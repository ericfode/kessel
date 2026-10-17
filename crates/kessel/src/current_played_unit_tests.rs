//! Source-derived SEC_236 checks; no reference-engine execution is claimed.
use super::*;
use crate::current_played_unit as contract;

fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn source(c: &Value) -> &Value {
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "SEC_236")
        .unwrap()
}
fn projection_engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    assert!(
        e.by_code.contains_key("SEC_236"),
        "{:?}",
        e.rejected_cards().unwrap().get("SEC_236")
    );
    e.set_rules_backend(backend).unwrap();
    e
}
fn position(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 4;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn hand_for(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn fund(e: &Engine, s: &mut State, seat: Seat) {
    for _ in 0..12 {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
fn play(e: &Engine, s: &State, id: CardId) -> State {
    let mut s = *s;
    s.active = s.card(id).owner;
    s.action_player = Some(s.active);
    e.apply(&s, &e.card_action(&s, id)).unwrap()
}
fn mark_played(s: &mut State, played_by: Seat, id: CardId) {
    s.phase_history.played(played_by, id).unwrap();
    s.card_mut(id).set_hidden_qualified(true);
}
fn played_predicate() -> Expr {
    serde_json::from_value(json!({"kind":"query","name":contract::QUERY,"receiver":{"kind":"ref","path":["card"]},"args":[]})).unwrap()
}
fn selected_pool(with_projection: bool) -> Engine {
    let c = catalogue();
    let mut pool: Value = serde_json::from_slice(include_bytes!("../data/pool.json")).unwrap();
    if with_projection {
        for code in ["SEC_236", "TOKEN_SPY"] {
            let entry = c["cards"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["setCode"] == code)
                .unwrap();
            let mut card = entry["card"].clone();
            card["setCode"] = json!(code);
            pool["cards"].as_array_mut().unwrap().push(card);
        }
    }
    Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    )
    .unwrap()
}

#[test]
fn current_played_unit_normalizes_only_the_complete_source_and_retains_provenance() {
    let c = catalogue();
    let original = source(&c).clone();
    let original_bytes = serde_json::to_vec(&original).unwrap();
    let abilities = contract::prepare(&original).unwrap().unwrap();
    assert_eq!(serde_json::to_vec(&original).unwrap(), original_bytes);
    assert_eq!(
        abilities["card"][0]["effect"],
        original["abilities"]["card"][0]["effect"]
    );
    assert_eq!(
        abilities["card"][0]["targetResolver"]["predicate"]["name"],
        contract::QUERY
    );
    let a: crate::ability_ir::Ability =
        serde_json::from_value(abilities["card"][0].clone()).unwrap();
    assert!(contract::ability(&a));
    let p = crate::ability_ir::Program::compile(&a).unwrap();
    assert!(contract::requires(&p));
    let loaded = crate::ability_ir::Catalogue::load(&c).unwrap();
    assert_eq!(loaded.metadata["SEC_236"], original["card"]);
    assert_eq!(loaded.cards["SEC_236"].card.len(), 1);
    assert!(!loaded.cards.contains_key("TS26_047"));
    // Alpha-renaming both declared watcher and its lambda is source-equivalent.
    let mut renamed = original.clone();
    let q = &mut renamed["abilities"]["card"][0]["targetResolver"]["predicate"];
    q["binding"] = json!("renamedWatcher");
    let old = q["args"][0]["bindings"][0].as_str().unwrap().to_owned();
    fn rename(v: &mut Value, old: &str) {
        if v["kind"] == "dataPath" && v["root"] == json!([old]) {
            v["root"] = json!(["renamedEntry"]);
        }
        match v {
            Value::Array(a) => {
                for v in a {
                    rename(v, old)
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    rename(v, old)
                }
            }
            _ => {}
        }
    }
    rename(q, &old);
    q["args"][0]["bindings"] = json!(["renamedEntry"]);
    renamed["stateWatchers"][0]["binding"] = json!("renamedWatcher");
    assert_eq!(contract::prepare(&renamed).unwrap().unwrap(), abilities);
}

#[test]
fn current_played_unit_source_mutations_do_not_bypass_scope_watchers_or_companions() {
    let original = source(&catalogue()).clone();
    for (pointer, value) in [
        ("/baseClass", json!("NonLeaderUnitCard")),
        ("/card/types", json!(["event", "leader"])),
        ("/card/unique", json!(true)),
        ("/piloting", json!({})),
        ("/privatePrograms", json!([{"kind":"noAction"}])),
        ("/stateWatchers/0/sharedScope", json!("player")),
        ("/stateWatchers/0/resetEvent", json!("onRoundEnded")),
        ("/abilities/card/0/optional", json!(true)),
        ("/abilities/card/0/coordinate", json!(true)),
        ("/abilities/card/0/ifYouDoCondition", json!({})),
        (
            "/abilities/card/0/condition",
            json!({"kind":"const","value":true}),
        ),
        ("/abilities/card/0/targetResolver/owner", json!("self")),
        ("/abilities/card/0/targetResolver/optional", json!(true)),
        (
            "/abilities/card/0/targetResolver/predicate/readAt",
            json!("cardPlay"),
        ),
        (
            "/abilities/card/0/effect/effects/1/onTrue/target",
            json!("enemyPlayer"),
        ),
        (
            "/abilities/card/0/effect/effects/1/onTrue/entersReady",
            json!(true),
        ),
        (
            "/abilities/card/0/effect/effects/1/condition/right/right/value",
            json!(4),
        ),
    ] {
        let mut bad = original.clone();
        let keys = pointer.split('/').skip(1).collect::<Vec<_>>();
        let mut at = &mut bad;
        for key in &keys[..keys.len() - 1] {
            at = if at.is_array() {
                &mut at[key.parse::<usize>().unwrap()]
            } else {
                &mut at[*key]
            };
        }
        at[keys[keys.len() - 1]] = value;
        assert!(contract::prepare(&bad).is_err(), "{pointer}");
    }
    let mut bad = original.clone();
    bad["stateWatchers"] = json!([]);
    assert!(contract::prepare(&bad).is_err());
    let mut bad = original.clone();
    bad["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .push(json!({"trigger":"event","title":"extra","effect":{"kind":"noAction"}}));
    assert!(contract::prepare(&bad).is_err());
    let normalized = contract::prepare(&original).unwrap().unwrap();
    let valid = normalized["card"][0].clone();
    for field in [
        "optional",
        "coordinate",
        "requiresConfirmation",
        "attackerMustSurvive",
    ] {
        let mut bad = valid.clone();
        bad[field] = json!(true);
        let a: crate::ability_ir::Ability = serde_json::from_value(bad).unwrap();
        assert!(crate::ability_ir::Program::compile(&a).is_err(), "{field}");
    }
    for flag in 0..3 {
        let mut a: crate::ability_ir::Ability = serde_json::from_value(valid.clone()).unwrap();
        match flag {
            0 => a.attached_host = true,
            1 => a.piloting_ability = true,
            _ => a.pilot_unit_ability = true,
        };
        assert!(crate::ability_ir::Program::compile(&a).is_err());
    }
    let mut forged = original;
    forged["abilities"] = normalized;
    assert!(
        contract::prepare(&forged).is_err(),
        "lowered query is not original watcher provenance"
    );
    let mut query = serde_json::to_value(played_predicate()).unwrap();
    query["receiver"]["path"] = json!(["target"]);
    let expression: Expr = serde_json::from_value(query).unwrap();
    assert!(expression.validate().is_err());
}

#[test]
fn current_played_unit_requires_both_histories_before_any_execution_mutation() {
    let e = projection_engine(RulesBackend::Interpreter);
    for missing in 0..3 {
        let mut s = position(&e, Seat::P1);
        let event = hand_for(&e, &mut s, Seat::P1, "SEC_236");
        s.source = event;
        if missing != 1 {
            s.phase_history.exact = false;
        }
        if missing != 0 {
            s.hidden_history_exact = false;
        }
        let program = e.card_programs(&s, event)[0];
        let before = s;
        assert!(e
            .begin_program(&mut s, program, 0)
            .unwrap_err()
            .contains("exact phase and Hidden history"));
        assert_eq!(s, before);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || e.current_played_unit_matches(&s, event)
        ))
        .is_err());
    }
}

#[test]
fn current_played_unit_real_ready_and_spy_are_independent_and_context_owned() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = projection_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for target_owner in [seat, seat.other()] {
                for ready in [false, true] {
                    let mut s = position(&e, seat);
                    fund(&e, &mut s, seat);
                    let target = unit(&e, &mut s, target_owner, "LOF_254");
                    mark_played(&mut s, target_owner, target);
                    s.card_mut(target).exhausted = !ready;
                    let event = hand_for(&e, &mut s, seat, "SEC_236");
                    let mut pending = play(&e, &s, event);
                    assert_eq!(pending.decision, Decision::AbilityTarget);
                    assert!(pending.player(seat).hand.contains(event));
                    assert!(e
                        .legal_actions(&pending)
                        .contains(&e.card_action(&pending, target)));
                    let allocated = pending.next_id;
                    pending = e.apply(&pending, &e.card_action(&pending, target)).unwrap();
                    assert!(!pending.card(target).exhausted);
                    assert!(pending.player(seat).discard.contains(event));
                    assert_eq!(pending.next_id, allocated + 1);
                    assert_eq!(e.def(&pending, allocated).code, "TOKEN_SPY");
                    assert_eq!(pending.card(allocated).owner, seat);
                    assert!(pending.player(seat).ground.contains(allocated));
                    assert!(pending.card(allocated).exhausted);
                    assert_eq!(pending.card(target).owner, target_owner);
                }
            }
            let mut s = position(&e, seat);
            fund(&e, &mut s, seat);
            let target = unit(&e, &mut s, seat.other(), "JTL_251");
            mark_played(&mut s, seat.other(), target);
            s.card_mut(target).exhausted = true;
            let event = hand_for(&e, &mut s, seat, "SEC_236");
            let pending = play(&e, &s, event);
            let next = pending.next_id;
            let out = e.apply(&pending, &e.card_action(&pending, target)).unwrap();
            assert!(!out.card(target).exhausted);
            assert_eq!(out.next_id, next, "printed cost6 creates no Spy");
        }
    }
}

#[test]
fn current_played_unit_real_hand_smuggle_placement_reentry_and_phase_paths() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = projection_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = position(&e, seat);
            fund(&e, &mut s, seat);
            let target = hand_for(&e, &mut s, seat, "LOF_254");
            s = play(&e, &s, target);
            assert!(e.current_played_unit_matches(&s, target));
            let placed = unit(&e, &mut s, seat, "LOF_256");
            assert!(!e.current_played_unit_matches(&s, placed));
            let old_id = target;
            e.leave_play_primary_to_discard(&mut s, target, false)
                .unwrap();
            assert!(!s.card(target).hidden_qualified());
            e.add_arena(&mut s, target).unwrap();
            assert!(
                !e.current_played_unit_matches(&s, target),
                "historical played bit cannot qualify placement"
            );
            e.leave_play_primary_to_discard(&mut s, target, false)
                .unwrap();
            s.player_mut(seat).hand.push(target).unwrap();
            s = play(&e, &s, target);
            assert_eq!(target, old_id);
            assert!(e.current_played_unit_matches(&s, target));
            // The controlled-in-play predicate is independent of which player played it.
            s.player_mut(seat).ground.remove(target);
            s.card_mut(target).owner = seat.other();
            s.player_mut(seat.other()).ground.push(target).unwrap();
            assert!(e.current_played_unit_matches(&s, target));
            // Native token creation and leader deployment qualify Hidden but record no play.
            let leader = s.player(seat).leader;
            s.card_mut(leader).deployed = true;
            s.card_mut(leader).set_hidden_qualified(true);
            e.add_arena(&mut s, leader).unwrap();
            let token = unit(&e, &mut s, seat, "TOKEN_SPY");
            s.card_mut(token).set_hidden_qualified(true);
            assert!(!e.current_played_unit_matches(&s, leader));
            assert!(!e.current_played_unit_matches(&s, token));
            // Defensive type guard also rejects malformed synthetic played bits.
            s.phase_history.played(seat, leader).unwrap();
            s.phase_history.played(seat, token).unwrap();
            assert!(!e.current_played_unit_matches(&s, leader));
            assert!(!e.current_played_unit_matches(&s, token));
            s.players[0].out = true;
            s.players[1].out = true;
            e.advance(&mut s).unwrap();
            assert!(!e.current_played_unit_matches(&s, target));
            assert!(!s.card(target).hidden_qualified());
            assert_eq!(s.phase_history.play_counts, [0, 0]);

            let mut s = position(&e, seat);
            fund(&e, &mut s, seat);
            let smuggle = add(&e, &mut s, seat, "SHD_097");
            s.player_mut(seat).resources.push(smuggle).unwrap();
            let replacement = add(&e, &mut s, seat, "LOF_254");
            s.player_mut(seat).deck.push(replacement).unwrap();
            e.begin_smuggle_play(&mut s, smuggle).unwrap();
            assert!(e.current_played_unit_matches(&s, smuggle));
            assert!(s.player(seat).resources.contains(replacement));
            assert!(s.player(seat).ground.contains(smuggle));
        }
    }
}

#[test]
fn current_played_unit_attachment_mode_is_not_an_arena_unit_incarnation() {
    let e = projection_engine(RulesBackend::Interpreter);
    let mut s = position(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    let pilot = unit(&e, &mut s, Seat::P1, "JTL_045");
    mark_played(&mut s, Seat::P1, pilot);
    assert!(e.current_played_unit_matches(&s, pilot));
    // Model a legal zone transition with ordinary departure cleanup first.
    e.leave_play_primary_to_discard(&mut s, pilot, false)
        .unwrap();
    e.attach(&mut s, pilot, host);
    assert!(!e.current_played_unit_matches(&s, pilot));
    s.card_mut(pilot).set_hidden_qualified(true);
    assert!(
        !e.current_played_unit_matches(&s, pilot),
        "attachment remains ineligible even with stale qualification"
    );
    e.leave_play_primary_to_discard(&mut s, pilot, false)
        .unwrap();
    e.add_arena(&mut s, pilot).unwrap();
    assert!(
        !e.current_played_unit_matches(&s, pilot),
        "plain re-entry after attachment cannot reuse earlier play"
    );
    e.leave_play_primary_to_discard(&mut s,pilot,false).unwrap();
    s.player_mut(Seat::P1).hand.push(pilot).unwrap();
    fund(&e, &mut s, Seat::P1);
    s = play(&e, &s, pilot);
    assert!(e.current_played_unit_matches(&s, pilot));
}

#[test]
fn current_played_unit_selected_pool_import_requires_both_exact_evidence_versions() {
    let e = selected_pool(true);
    assert!(e.needs_hidden_history());
    assert!(e.needs_phase_history());
    let mut s = position(&e, Seat::P1);
    s.force_state_exact = false;
    let snapshot = e.snapshot(&s);
    assert!(e.load_state(&snapshot).is_ok());
    for key in ["hiddenHistoryVersion", "phaseHistory"] {
        let mut bad = snapshot.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(e.load_state(&bad).is_err(), "missing {key}");
    }
    let legacy = selected_pool(false);
    assert!(
        legacy.by_code.contains_key("SEC_236"),
        "retained definitions alone do not demand history"
    );
    let mut s = position(&legacy, Seat::P1);
    s.hidden_history_exact = false;
    s.phase_history.exact = false;
    s.force_state_exact = false;
    let snapshot = legacy.snapshot(&s);
    let imported = legacy.load_state(&snapshot).unwrap();
    assert!(!imported.hidden_history_exact);
    assert!(!imported.phase_history.exact);
    assert_eq!(legacy.snapshot(&imported), snapshot);
}

#[test]
fn current_played_unit_imported_historical_play_does_not_requalify_new_incarnation() {
    let e = projection_engine(RulesBackend::Interpreter);
    for seat in [Seat::P1, Seat::P2] {
        let mut s = position(&e, seat);
        let target = unit(&e, &mut s, seat.other(), "LOF_254");
        s.phase_history.played(seat, target).unwrap();
        let snapshot = e.snapshot(&s);
        let restored = e.load_state(&snapshot).unwrap();
        assert!(!restored.card(target).hidden_qualified());
        assert!(!e.current_played_unit_matches(&restored, target));
        let mut changed = restored;
        changed.card_mut(target).set_hidden_qualified(true);
        assert!(e.current_played_unit_matches(&changed, target));
        assert_ne!(changed.zobrist(), restored.zobrist());
        let copy = changed;
        assert_eq!(copy, changed);
        assert_eq!(copy.zobrist(), changed.zobrist());
        let mut imported = snapshot;
        imported["players"][seat.other().as_str()]["ground"][0]["hiddenQualified"] = json!(true);
        assert!(e.current_played_unit_matches(&e.load_state(&imported).unwrap(), target));
        imported["phaseHistory"]["played"] = json!([]);
        imported["phaseHistory"]["playCounts"] = json!([0, 0]);
        assert!(
            !e.current_played_unit_matches(&e.load_state(&imported).unwrap(), target),
            "qualification alone is not evidence of play"
        );
    }
}

#[test]
fn current_played_unit_cancel_before_target_preserves_payment_and_event_identity() {
    let e = projection_engine(RulesBackend::Interpreter);
    let mut s = position(&e, Seat::P1);
    fund(&e, &mut s, Seat::P1);
    let target = unit(&e, &mut s, Seat::P2, "LOF_254");
    mark_played(&mut s, Seat::P2, target);
    let event = hand_for(&e, &mut s, Seat::P1, "SEC_236");
    let before = s.player(Seat::P1).resources;
    let pending = play(&e, &s, event);
    assert_eq!(pending.decision, Decision::AbilityTarget);
    let cancel = e
        .legal_actions(&pending)
        .into_iter()
        .find(|a| a.label == "Cancel")
        .unwrap();
    let out = e.apply(&pending, &cancel).unwrap();
    assert!(out.player(Seat::P1).hand.contains(event));
    assert!(!out.player(Seat::P1).discard.contains(event));
    for &id in before.as_slice() {
        assert!(!out.card(id).exhausted);
    }
    assert_eq!(out.next_id, pending.next_id);
}

#[test]
fn current_played_unit_spy_dependency_is_still_validated() {
    let mut c = catalogue();
    c["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["setCode"] != "TOKEN_SPY");
    assert!(!Engine::from_catalogue_value(&c)
        .unwrap()
        .by_code
        .contains_key("SEC_236"));
}

#[test]
fn current_played_unit_resumption_rejects_unknown_evidence_before_mutation() {
    let e = projection_engine(RulesBackend::Interpreter);
    let mut s = position(&e, Seat::P1);
    fund(&e, &mut s, Seat::P1);
    let target = unit(&e, &mut s, Seat::P2, "LOF_254");
    mark_played(&mut s, Seat::P2, target);
    let event = hand_for(&e, &mut s, Seat::P1, "SEC_236");
    let pending = play(&e, &s, event);
    for missing_hidden in [false, true] {
        let mut unknown = pending;
        if missing_hidden {
            unknown.hidden_history_exact = false;
        } else {
            unknown.phase_history.exact = false;
        }
        let before = unknown;
        assert!(e
            .apply_ability_action(&unknown, &e.card_action(&unknown, target))
            .unwrap_err()
            .contains("exact phase and Hidden history"));
        assert_eq!(unknown, before);
        assert!(e
            .run_program(&mut unknown)
            .unwrap_err()
            .contains("exact phase and Hidden history"));
        assert_eq!(unknown, before);
    }
}

#[test]
fn current_played_unit_typed_query_cannot_escape_into_source_attachment_or_helpers() {
    let c = catalogue();
    let foreign = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["setCode"] == "SOR_071")
        .unwrap();
    for slot in [
        "attachCondition",
        "piloting",
        "privatePrograms",
        "historyHelpers",
    ] {
        let mut bad = foreign.clone();
        bad[slot] = serde_json::to_value(played_predicate()).unwrap();
        assert!(contract::prepare(&bad).is_err(), "foreign {slot}");
        let mut mutated = c.clone();
        let row = mutated["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["setCode"] == "SOR_071")
            .unwrap();
        *row = bad;
        assert!(!Engine::from_catalogue_value(&mutated)
            .unwrap()
            .by_code
            .contains_key("SOR_071"));
    }
}
