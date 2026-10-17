//! Source-derived witnesses for the `cardsDefeatedThisPhase` projections.
//!
//! Pinned engine source (`forceteki` @ the card-conversion pin):
//! `server/game/stateWatchers/CardsDefeatedThisPhaseWatcher.ts` appends one
//! entry per `onCardDefeated` whose last-known type is a unit or an upgrade:
//! `{card, inPlayId, controlledBy: LKI controller, defeatedBy,
//! wasDefeatedWhileAttacking, lastKnownInformation}`. The while-attacking value
//! is computed in `DefeatCardSystem.addDefeatSourceToEvent` as
//! `card.isUnit() && card.isInPlay() && card.isAttacking()` before the move.
//! `someDefeatedUnitControlledByPlayer` filters `isUnit && controlledBy===controller`;
//! `getDefeatedUnitsControlledByPlayer(...).length` counts those entries (one per
//! defeat event, repeats included); `someDefeatedWhileAttackingUnitControlledByPlayer`
//! adds the truthy while-attacking flag. Every query reads the live list, so zone
//! departure does not erase history, and the list resets at `onPhaseEnded`.
use super::*;
use crate::model::PhaseHistory;
use crate::phase_history::{Argument, Length, List, Query, QueryKind};

fn defeated_query(operation: &str, controller: &str) -> Query {
    serde_json::from_value(json!({
        "watcher": "cardsDefeatedThisPhase",
        "binding": "cardsDefeatedThisPhaseWatcher",
        "operation": operation,
        "args": [{"kind": "ref", "path": [controller]}],
        "readAt": "expressionEvaluation",
    }))
    .unwrap()
}
fn defeated_count(controller: &str) -> Expr {
    Expr::DataMember {
        receiver: List::HistoryQuery(defeated_query(
            "getDefeatedUnitsControlledByPlayer",
            controller,
        )),
        key: Length::Length,
        optional: false,
    }
}
fn history_count(e: &Engine, s: &State, source: CardId, controller: &str) -> i64 {
    e.eval(s, source, 0, &defeated_count(controller)).number()
}
fn draw_into_deck(e: &Engine, s: &mut State, seat: Seat, n: usize) {
    for _ in 0..n {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).deck.push(id).unwrap();
    }
}
fn play(e: &Engine, s: &State, id: CardId) -> State {
    played_from_hand(e, s, s.card(id).owner, id)
}
/// A resolved-without-effect ability keeps its play uncommitted behind the
/// prompt; follow it exactly like the ability-action protocol does.
fn resolve_no_effect(e: &Engine, s: State) -> State {
    let mut s = s;
    while s.decision == Decision::AbilityNoEffect {
        let action = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Play anyway" || a.label == "Use it anyway")
            .expect("no-effect prompt offers a commit action");
        s = e.apply(&s, &action).unwrap();
    }
    s
}
fn create_token(e: &Engine, s: &mut State, source: CardId) {
    let effect = Effect::CreateTokenUnit {
        token_type: "battleDroid".into(),
        amount: Expr::number(1),
        target: Target::Name("ownPlayer".into()),
        enters_ready: false,
    };
    s.source = source;
    e.generate_group(s, &effect).unwrap();
    e.resolve_group(s, &effect).unwrap();
    s.source = 0;
}
fn played_from_hand(e: &Engine, s: &State, seat: Seat, id: CardId) -> State {
    let mut s = *s;
    s.active = seat;
    s.action_player = Some(seat);
    // Resources stay in the zone after being spent, so top the player up with a
    // full ready set rather than counting exhausted resources as payable.
    for _ in 0..8 {
        let r = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    e.apply(&s, &e.card_action(&s, id)).unwrap()
}

#[test]
fn phase_history_defeated_projection_counts_units_by_controller_at_defeat() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        // No defeat: the whenPlayed condition is false and nothing heals.
        let mut s = board(&e);
        let base = s.player(Seat::P1).base;
        s.card_mut(base).damage = 5;
        let card = hand(&e, &mut s, "LAW_109");
        let played = resolve_no_effect(&e, play(&e, &s, card));
        assert_eq!(played.card(base).damage, 5, "{backend:?}");
        assert_eq!(played.phase_history.defeated_counts, [0, 0]);
        // A friendly unit defeat is retained after the unit reaches discard.
        let mut s = board(&e);
        let base = s.player(Seat::P1).base;
        s.card_mut(base).damage = 5;
        let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
        e.defeat(&mut s, friendly).unwrap();
        assert!(!e.in_arena(&s, friendly) && s.players[0].discard.contains(friendly));
        assert_eq!(s.phase_history.defeated_counts, [1, 0]);
        let card = hand(&e, &mut s, "LAW_109");
        let played = play(&e, &s, card);
        assert_eq!(played.card(base).damage, 1, "{backend:?}");
        // An enemy defeat is not a friendly defeat for the same card.
        let mut s = board(&e);
        let base = s.player(Seat::P1).base;
        s.card_mut(base).damage = 5;
        let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
        e.defeat(&mut s, enemy).unwrap();
        let card = hand(&e, &mut s, "LAW_109");
        let played = play(&e, &s, card);
        assert_eq!(played.phase_history.defeated_counts, [0, 1]);
        assert_eq!(played.card(base).damage, 5, "{backend:?}");
    }
}

#[test]
fn phase_history_defeated_projection_separates_tokens_upgrades_and_non_defeat_moves() {
    let e = engine();
    // A token unit is a unit defeat (`EnumHelpers.isUnit(TokenUnit)`).
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    create_token(&e, &mut s, source);
    let token = *s.players[0]
        .ground
        .as_slice()
        .iter()
        .find(|id| **id != source)
        .unwrap();
    assert!(e.def(&s, token).info.is("token"));
    e.defeat(&mut s, token).unwrap();
    assert_eq!(s.phase_history.defeated_counts, [1, 0]);
    assert_eq!(s.phase_history.defeated_while_attacking_counts, [0, 0]);
    // An upgrade defeat is an entry in the pinned watcher but not a *unit*
    // entry: the unit projections must ignore it.
    let mut s = board(&e);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    let upgrade = add(&e, &mut s, Seat::P1, "SOR_071");
    e.attach(&mut s, upgrade, host);
    e.defeat(&mut s, upgrade).unwrap();
    assert_eq!(s.phase_history.defeated_counts, [0, 0]);
    assert!(e.in_arena(&s, host), "host survives an upgrade defeat");
    // A non-defeat departure (return to hand) is not a defeat.
    let mut s = board(&e);
    let returned = unit(&e, &mut s, Seat::P1, "LOF_256");
    e.leave_play_primary_to_discard(&mut s, returned, false)
        .unwrap();
    s.players[0].hand.push(returned).unwrap();
    assert_eq!(s.phase_history.defeated_counts, [0, 0]);
}

#[test]
fn phase_history_defeated_while_attacking_projects_attacker_only() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for scenario in ["attacker", "token-attacker", "defender", "bystander"] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = board(&e);
            draw_into_deck(&e, &mut s, Seat::P1, 5);
            let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
            let defender = unit(&e, &mut s, Seat::P2, "LOF_256");
            let bystander = unit(&e, &mut s, Seat::P1, "LOF_256");
            let token = if scenario == "token-attacker" {
                create_token(&e, &mut s, attacker);
                *s.players[0]
                    .ground
                    .as_slice()
                    .iter()
                    .find(|id| **id != attacker && **id != bystander)
                    .unwrap()
            } else {
                0
            };
            let attacker = if scenario == "token-attacker" {
                token
            } else {
                attacker
            };
            s.attack_source = attacker;
            s.attack_defender = defender;
            let defeated = match scenario {
                "attacker" | "token-attacker" => attacker,
                "defender" => defender,
                "bystander" => bystander,
                _ => unreachable!(),
            };
            e.defeat(&mut s, defeated).unwrap();
            let defeated_seat = s.card(defeated).owner;
            assert_eq!(
                s.phase_history.defeated_counts,
                if defeated_seat == Seat::P1 { [1, 0] } else { [0, 1] },
                "{scenario}"
            );
            let while_attacking =
                matches!(scenario, "attacker" | "token-attacker") as u16;
            assert_eq!(
                s.phase_history.defeated_while_attacking_counts,
                if defeated_seat == Seat::P1 {
                    [while_attacking, 0]
                } else {
                    [0, while_attacking]
                },
                "{scenario}"
            );
            s.attack_source = 0;
            s.attack_defender = 0;
            let card = hand(&e, &mut s, "SEC_158");
            let hand_before = s.players[0].hand.len as i64 - 1;
            let played = resolve_no_effect(&e, play(&e, &s, card));
            let draws = if while_attacking == 1 { 3 } else { 0 };
            assert_eq!(
                played.players[0].hand.len as i64,
                hand_before + draws,
                "{backend:?}:{scenario}"
            );
            let q = defeated_query("someDefeatedWhileAttackingUnitControlledByPlayer", "player");
            assert_eq!(
                e.history_boolean(&played, played.players[0].base, 0, &q),
                while_attacking == 1,
                "{backend:?}:{scenario}"
            );
        }
    }
}

#[test]
fn phase_history_defeated_projection_counts_repeats_and_resolves_controller_side() {
    let e = engine();
    let mut s = board(&e);
    let replayed = add(&e, &mut s, Seat::P2, "LOF_256");
    s.players[1].hand.push(replayed).unwrap();
    let source = s.player(Seat::P1).base;
    // Two defeats of the same physical card in one phase, each a separate
    // incarnation after returning it to hand and replaying it.
    for _ in 0..2 {
        let played = played_from_hand(&e, &s, Seat::P2, replayed);
        let mut defeated = played;
        e.defeat(&mut defeated, replayed).unwrap();
        s = defeated;
        s.players[1].discard.remove(replayed);
        s.players[1].hand.push(replayed).unwrap();
    }
    assert_eq!(s.phase_history.defeated_counts, [0, 2]);
    // A physical-identity bitset would report one; the pinned `.length` counts
    // one entry per defeat event.
    assert_eq!(history_count(&e, &s, source, "player"), 0);
    assert_eq!(history_count(&e, &s, source, "opponent"), 2);
    let friendly = defeated_query("someDefeatedUnitControlledByPlayer", "player");
    let enemy = defeated_query("someDefeatedUnitControlledByPlayer", "opponent");
    assert!(!e.history_boolean(&s, source, 0, &friendly));
    assert!(e.history_boolean(&s, source, 0, &enemy));
    // The same query resolved from the defeated card's own controller: for that
    // controller the defeated unit is its own "player", so the projection holds.
    assert!(e.history_boolean(&s, replayed, 0, &friendly));
    assert_eq!(
        defeated_query("getDefeatedUnitsControlledByPlayer", "opponent").kind(),
        Ok(QueryKind::DefeatedUnitCount(
            crate::phase_history::PlayerRef::Opponent
        ))
    );
    // Exact controller-reference and shape gates stay rejections.
    for bad in [
        json!({"watcher":"cardsDefeatedThisPhase","binding":"b","operation":"someDefeatedUnitControlledByPlayer","args":[{"kind":"ref","path":["game"]}],"readAt":"expressionEvaluation"}),
        json!({"watcher":"cardsDefeatedThisPhase","binding":"b","operation":"someDefeatedUnitControlledByPlayer","args":[{"kind":"ref","path":["player","base"]}],"readAt":"expressionEvaluation"}),
        json!({"watcher":"cardsDefeatedThisPhase","binding":"b","operation":"someDefeatedUnitControlledByPlayer","args":[{"kind":"dataLambda","bindings":["entry"],"body":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["entry"],"steps":[{"key":"controlledBy","optional":false}]},"right":{"kind":"ref","path":["player"]}}}],"readAt":"expressionEvaluation"}),
        json!({"watcher":"cardsDefeatedThisPhase","binding":"b","operation":"getDefeatedUnitsControlledByPlayer","args":[{"kind":"ref","path":["player"]},{"kind":"ref","path":["opponent"]}],"readAt":"expressionEvaluation"}),
        json!({"watcher":"cardsDefeatedThisPhase","binding":"","operation":"someDefeatedUnitControlledByPlayer","args":[{"kind":"ref","path":["player"]}],"readAt":"expressionEvaluation"}),
        json!({"watcher":"cardsPlayedThisPhase","binding":"b","operation":"someDefeatedUnitControlledByPlayer","args":[{"kind":"ref","path":["player"]}],"readAt":"expressionEvaluation"}),
    ] {
        let q: Query = serde_json::from_value(bad.clone()).unwrap();
        assert!(q.kind().is_err(), "{bad}");
    }
    assert!(matches!(
        defeated_query("getDefeatedUnitsControlledByPlayer", "player").args[0],
        Argument::Ref { .. }
    ));
    assert!(defeated_count("player").validate().is_ok());
}

#[test]
fn phase_history_defeated_projection_resets_exports_and_bounds_capacity() {
    let e = engine();
    let mut s = board(&e);
    let first = unit(&e, &mut s, Seat::P1, "LOF_256");
    let second = unit(&e, &mut s, Seat::P1, "LOF_256");
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    e.defeat(&mut s, first).unwrap();
    e.defeat(&mut s, second).unwrap();
    s.attack_source = attacker;
    e.defeat(&mut s, attacker).unwrap();
    s.attack_source = 0;
    assert_eq!(s.phase_history.defeated_counts, [3, 0]);
    assert_eq!(s.phase_history.defeated_while_attacking_counts, [1, 0]);
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["phaseHistory"]["defeatsVersion"], json!(1));
    assert_eq!(snapshot["phaseHistory"]["defeatedCounts"], json!([3, 0]));
    assert_eq!(
        snapshot["phaseHistory"]["defeatedWhileAttackingCounts"],
        json!([1, 0])
    );
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(restored.phase_history, s.phase_history);
    assert_eq!(restored.zobrist(), s.zobrist());
    // A legacy export without the defeat capability must not invent zero
    // defeats for a pool that reads them.
    let mut legacy = snapshot.clone();
    let history = legacy["phaseHistory"].as_object_mut().unwrap();
    history.remove("defeatsVersion");
    history.remove("defeatedCounts");
    history.remove("defeatedWhileAttackingCounts");
    assert!(e
        .load_state(&legacy)
        .unwrap_err()
        .contains("defeat phase history version"));
    // Counts without their marker are not a v1 phase history either, and the
    // while-attacking subset can never exceed the unit-defeat count.
    let mut value = snapshot["phaseHistory"].clone();
    value.as_object_mut().unwrap().remove("defeatsVersion");
    assert!(PhaseHistory::parse(&value, s.next_id).is_err());
    let mut value = snapshot["phaseHistory"].clone();
    value["defeatedCounts"] = json!([0, 0]);
    value["defeatedWhileAttackingCounts"] = json!([1, 0]);
    assert_eq!(
        PhaseHistory::parse(&value, s.next_id).unwrap_err(),
        "phaseHistory while-attacking defeats exceed unit defeats"
    );
    let mut value = snapshot["phaseHistory"].clone();
    value["defeatsVersion"] = json!(2);
    assert_eq!(
        PhaseHistory::parse(&value, s.next_id).unwrap_err(),
        "unsupported defeatsVersion"
    );
    // Counters are bounded and a refused capture leaves history unchanged.
    let mut full = s;
    full.phase_history.defeated_counts[0] = u16::MAX;
    let before = full.phase_history;
    assert_eq!(
        full.phase_history.defeated(Seat::P1, false),
        Err("phase unit-defeat count capacity exceeded".into())
    );
    assert_eq!(full.phase_history, before);
    full.phase_history.defeated_counts[0] = 0;
    full.phase_history.defeated_while_attacking_counts[0] = u16::MAX;
    assert_eq!(
        full.phase_history.defeated(Seat::P1, true),
        Err("phase while-attacking defeat count capacity exceeded".into())
    );
    assert_eq!(full.phase_history.defeated_counts[0], 0);
    // Phase end resets the watcher, keeping the export capability.
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.phase_history, PhaseHistory::default());
    assert!(s.phase_history.defeat_exact && s.phase_history.exact);
}

#[test]
fn phase_history_defeated_projection_gates_the_constant_self_buff_live() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        // The unit entered before any defeat, so a snapshot at registration time
        // would miss the buff the engine's live constant condition applies.
        let mut s = board(&e);
        let jyn = unit(&e, &mut s, Seat::P1, "TWI_143");
        let base_power = e.power(&s, jyn, false);
        let base_hp = e.hp(&s, jyn);
        assert!(!e.has_keyword(&s, jyn, "saboteur"), "{backend:?}");
        // A friendly defeat is not an enemy defeat.
        let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
        e.defeat(&mut s, friendly).unwrap();
        assert_eq!(e.power(&s, jyn, false), base_power);
        assert!(!e.has_keyword(&s, jyn, "saboteur"));
        // An enemy defeat turns the passive buff on for the same in-play unit.
        let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
        e.defeat(&mut s, enemy).unwrap();
        assert_eq!(e.power(&s, jyn, false), base_power + 1);
        assert_eq!(e.hp(&s, jyn), base_hp);
        assert!(e.has_keyword(&s, jyn, "saboteur"), "{backend:?}");
        // Phase end clears the watcher, so the buff disappears even though the
        // unit stays in play.
        s.players[0].out = true;
        s.players[1].out = true;
        e.advance(&mut s).unwrap();
        assert_eq!(s.phase_history.defeated_counts, [0, 0]);
        assert_eq!(e.power(&s, jyn, false), base_power);
        assert!(!e.has_keyword(&s, jyn, "saboteur"));
    }
}

#[test]
fn phase_history_defeated_family_keeps_unimplemented_projections_rejected() {
    let e = engine();
    // Admitted here: single-registration ordinary unit/event shapes.
    for code in [
        "LAW_109", "SEC_083", "SOR_051", "TWI_031", "LOF_263", "SEC_158", "TWI_143",
    ] {
        assert!(e.by_code.contains_key(code), "{code}");
    }
    // Kept explicit: identity/incarnation reads, leader and constant
    // registrations, multi-registration entries and unrelated runtime gaps.
    let rejected = e.rejected_cards().unwrap();
    for code in [
        "SHD_105", "SOR_091", "TWI_086", "ASH_093", "SOR_002", "TWI_002", "SEC_035", "ASH_079",
        "LOF_007",
    ] {
        assert!(
            rejected.get(code).is_some_and(|reason| reason.ends_with(
                "phase history requires complete ordinary unit/event source metadata and registration"
            )),
            "{code}: {:?}",
            rejected.get(code)
        );
    }
    // TWI_186 (San Hill) needs this family's `getDefeatedUnitsControlledByPlayer`
    // count projection *and* the Exploit keyword; the integrated batch tree
    // implements both, so the card admits and finally exercises the count
    // consumer end-to-end. The lane-internal tree kept it rejected while
    // Exploit was missing, so this assertion moved with the batch integration.
    assert!(e.by_code.contains_key("TWI_186"), "TWI_186");
}

