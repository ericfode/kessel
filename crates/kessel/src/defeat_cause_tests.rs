use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e
            .by_code
            .get(code)
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))),
        owner,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    // This builder creates a fresh position with an explicitly empty Credit ledger.
    s.credit_state_exact = e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..12 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label}: {:?}", e.legal_actions(s)));
    e.apply(s, &action).unwrap()
}
fn give(e: &Engine, s: &mut State, host: CardId, amount: u16) {
    s.source = host;
    e.apply_effect(s,&serde_json::from_value(json!({"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":amount},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).unwrap()).unwrap();
    s.source = 0;
}
fn tokens(e: &Engine, s: &State, host: CardId) -> Vec<CardId> {
    e.upgrades(s, host)
        .into_iter()
        .filter(|id| Some(s.card(*id).def) == e.advantage_def)
        .collect()
}
fn play(e: &Engine, s: &State, code: &str) -> (State, CardId) {
    let mut state = *s;
    let source = add(e, &mut state, s.active, code);
    state.player_mut(s.active).hand.push(source).unwrap();
    let result = choose(e, &state, source);
    (result, source)
}
fn attack(e: &Engine, s: &State, source: CardId, target: CardId) -> State {
    let mut state = choose(e, s, source);
    if state.decision == Decision::AbilityMenu {
        state = button(e, &state, "Attack");
    }
    assert_eq!(state.decision, Decision::AttackTarget);
    choose(e, &state, target)
}

use crate::defeat_cause::{DefeatCause, TriggerCapture};
#[test]
fn cause_admission_is_exact_and_no_raw_leaf_bypass() {
    let value = catalogue();
    let e = Engine::from_catalogue_value(&value).unwrap();
    assert!(
        e.by_code.contains_key("ASH_191"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_191")
    );
    let admitted = value["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_191")
        .unwrap();
    for pointer in [
        "/card/types",
        "/baseClass",
        "/historyHelpers/0/body/statements/0/value/condition/right/value",
        "/abilities/card/0/optional",
        "/abilities/card/0/targetResolver/owner",
    ] {
        let mut bad = admitted.clone();
        *bad.pointer_mut(pointer).unwrap() = json!("bad");
        let c =
            crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[bad]}))
                .unwrap();
        assert!(!c.cards.contains_key("ASH_191"), "{pointer}");
    }
    let lowered = crate::defeat_cause::prepare(admitted).unwrap().unwrap();
    let c =
        crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[lowered]}))
            .unwrap();
    assert!(!c.cards.contains_key("ASH_191"));
}
#[test]
fn cause_storage_copy_hash_and_legacy_layout() {
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    assert_eq!(std::mem::size_of::<DeathTrigger>(), 14);
    eprintln!(
        "Fiend layouts State={} Card={} DeathFrame={} ResolutionContext={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<Card>(),
        std::mem::size_of::<DeathFrame>(),
        std::mem::size_of::<ResolutionContext>()
    );
    for cause in [
        DefeatCause::Unknown,
        DefeatCause::Attack,
        DefeatCause::NonCombatDamage,
        DefeatCause::Ability,
        DefeatCause::FrameworkEffect,
        DefeatCause::UniqueRule,
    ] {
        let mut capture = TriggerCapture::new(8, cause);
        assert_eq!(capture.depth(), MAX_DEATH_DEPTH as u8);
        assert_eq!(capture.cause(), cause);
        capture.set_depth(1);
        assert_eq!(capture.cause(), cause);
        let mut s = State::default();
        s.defeat_cause = cause;
        let context = ResolutionContext::capture(&s);
        s.defeat_cause = DefeatCause::Unknown;
        context.restore(&mut s);
        assert_eq!(s.defeat_cause, cause);
        if cause != DefeatCause::Unknown {
            assert_ne!(s.zobrist(), State::default().zobrist());
        }
        let mut other = s;
        other.defeat_cause = DefeatCause::Unknown;
        other.death_frames[0].context.defeat_cause = cause;
        assert_eq!(other.death_frames[0].context.defeat_cause, cause);
    }
    let mut a = State::default();
    a.death_pending.entries[0].capture = TriggerCapture::new(0, DefeatCause::Ability);
    let mut b = a;
    b.death_pending.entries[0].capture = TriggerCapture::new(0, DefeatCause::FrameworkEffect);
    assert_ne!(a.zobrist(), b.zobrist());
}
#[test]
fn captured_cause_controls_exact_prompt_amount_and_recipient_ownership() {
    for seat in [Seat::P1, Seat::P2] {
        for cause in [
            DefeatCause::Attack,
            DefeatCause::NonCombatDamage,
            DefeatCause::Ability,
            DefeatCause::FrameworkEffect,
            DefeatCause::UniqueRule,
        ] {
            for enemy in [false, true] {
                let e = engine();
                let mut s = board(&e, seat);
                let fiend = unit(&e, &mut s, seat, "ASH_191");
                let host = unit(
                    &e,
                    &mut s,
                    if enemy { seat.other() } else { seat },
                    "LOF_254",
                );
                s.attack_source = host;
                s.source = host; // An active attack cannot override captured cause.
                e.defeat_events(&mut s, &[(fiend, cause)]).unwrap();
                assert!(!e.in_arena(&s, fiend));
                assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
                let amount = cause.advantage_amount().unwrap();
                let expected = format!("Give {amount} Advantage tokens to a unit");
                assert_eq!(s.decision, Decision::AbilityTarget);
                assert_eq!(
                    e.program_menu_title(&s, fiend, s.ability_program - 1),
                    expected
                );
                assert!(e.legal_actions(&s).iter().any(|a| a.label == "Pass"));
                let before = s.next_id;
                s = choose(&e, &s, host);
                let ids = tokens(&e, &s, host);
                assert_eq!(ids.len(), amount as usize);
                assert_eq!(ids, (before..before + amount).collect::<Vec<_>>());
                for id in ids {
                    assert_eq!(s.card(id).owner, s.card(host).owner);
                }
            }
        }
    }
}
#[test]
fn scalar_damage_during_attack_captures_noncombat_while_stat_death_is_framework() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let fiend = unit(&e, &mut s, Seat::P1, "ASH_191");
    let other = unit(&e, &mut s, Seat::P2, "LOF_254");
    s.source = other;
    s.attack_source = other;
    s.ability_target = fiend;
    e.apply_effect(
        &mut s,
        &serde_json::from_value(
            json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        s.death_pending.entries[0].capture.cause(),
        DefeatCause::NonCombatDamage
    );
    let mut s = board(&e, Seat::P1);
    let fiend = unit(&e, &mut s, Seat::P1, "ASH_191");
    s.card_mut(fiend).phase_hp = -1;
    e.sweep_stat_defeats(&mut s).unwrap();
    assert_eq!(
        s.death_pending.entries[0].capture.cause(),
        DefeatCause::FrameworkEffect
    );
}
#[test]
fn unknown_captured_cause_is_rejected_before_resolution() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let fiend = unit(&e, &mut s, Seat::P1, "ASH_191");
    let host = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.source = host;
    e.capture_death_cause(&mut s, fiend, DefeatCause::Unknown)
        .unwrap();
    let error = e
        .open_deaths(&mut s, DeathContinuation::Finish)
        .unwrap_err();
    assert!(error.contains("cause is missing"), "{error}");
}

#[test]
fn actual_combat_death_and_jedi_attack_ability_are_distinct() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let fiend = unit(&e, &mut s, seat.other(), "ASH_191");
        let fighter = unit(&e, &mut s, seat, "JTL_247");
        let host = unit(&e, &mut s, seat.other(), "LOF_254");
        let pending = attack(&e, &s, fighter, fiend);
        assert_eq!(pending.decision, Decision::AbilityTarget);
        assert_eq!(pending.defeat_cause, DefeatCause::Attack);
        assert_eq!(
            e.program_menu_title(&pending, fiend, pending.ability_program - 1),
            "Give 2 Advantage tokens to a unit"
        );
        let done = choose(&e, &pending, host);
        assert_eq!(tokens(&e, &done, host).len(), 2);
        let mut s = board(&e, seat);
        let fiend = unit(&e, &mut s, seat.other(), "ASH_191");
        let jedi = unit(&e, &mut s, seat, "LOF_144");
        let host = unit(&e, &mut s, seat.other(), "LOF_254");
        let base = s.player(seat.other()).base;
        let mut pending = attack(&e, &s, jedi, base);
        assert_eq!(pending.decision, Decision::AbilityTarget);
        pending = choose(&e, &pending, fiend);
        assert_eq!(pending.decision, Decision::AbilityTarget);
        assert_eq!(pending.attack_source, jedi);
        assert_eq!(pending.defeat_cause, DefeatCause::NonCombatDamage);
        assert_eq!(
            e.program_menu_title(&pending, fiend, pending.ability_program - 1),
            "Give 3 Advantage tokens to a unit"
        );
        let done = choose(&e, &pending, host);
        assert_eq!(tokens(&e, &done, host).len(), 3);
    }
}
#[test]
fn optional_decline_creates_nothing_and_recipient_tokens_persist_to_its_attack_end() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let fiend = unit(&e, &mut s, Seat::P1, "ASH_191");
    let host = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.defeat_events(&mut s, &[(fiend, DefeatCause::Ability)])
        .unwrap();
    e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
    let next = s.next_id;
    let declined = button(&e, &s, "Pass");
    assert_eq!(declined.next_id, next);
    assert!(tokens(&e, &declined, host).is_empty());
    let mut done = choose(&e, &s, host);
    let given = tokens(&e, &done, host);
    assert_eq!(given.len(), 3);
    done.players[0].out = true;
    done.players[1].out = true;
    e.advance_after_expiry(&mut done).unwrap();
    assert_eq!(tokens(&e, &done, host), given);
    e.end_regroup(&mut done).unwrap();
    done.active = Seat::P2;
    done.action_player = Some(Seat::P2);
    let base = done.player(Seat::P1).base;
    let pending = attack(&e, &done, host, base);
    assert_eq!(pending.decision, Decision::DeathBatch);
    let done = button(&e, &pending, "Resolve all (3)");
    assert!(tokens(&e, &done, host).is_empty());
}

#[test]
fn batch_causes_stay_with_physical_event_and_unknown_pending_wire_rejects() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "ASH_191");
    let b = unit(&e, &mut s, Seat::P2, "ASH_191");
    e.defeat_events(
        &mut s,
        &[(a, DefeatCause::Attack), (b, DefeatCause::FrameworkEffect)],
    )
    .unwrap();
    assert_eq!(
        s.death_pending.entries[..2]
            .iter()
            .map(|e| (e.source, e.capture.cause()))
            .collect::<Vec<_>>(),
        vec![(a, DefeatCause::Attack), (b, DefeatCause::FrameworkEffect)]
    );
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["defeatCausePending"], true);
    assert!(e
        .load_state(&snapshot)
        .unwrap_err()
        .contains("captured defeat cause"));
    for raw in 0..=255 {
        assert_eq!(DefeatCause::from_raw(raw).is_some(), raw <= 5);
    }
}
#[test]
fn native_auto_and_interpreter_share_the_captured_leaf_and_report_fallback() {
    let mut results = Vec::new();
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        e.set_rules_instrumentation(true);
        let mut s = board(&e, Seat::P1);
        let fiend = unit(&e, &mut s, Seat::P1, "ASH_191");
        let host = unit(&e, &mut s, Seat::P2, "LOF_254");
        let program = e.card_programs(&s, fiend)[0];
        assert!(e.program(program).executable.is_err());
        e.defeat_events(&mut s, &[(fiend, DefeatCause::Ability)])
            .unwrap();
        e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
        let done = choose(&e, &s, host);
        assert_eq!(tokens(&e, &done, host).len(), 3);
        results.push(done);
        let report = e.rules_backend_report();
        let p = &report["programs"][program as usize];
        assert_eq!(
            p["backend"],
            if backend == RulesBackend::Auto {
                "interpreterFallback"
            } else {
                "interpreter"
            }
        );
        assert!(p["executedSteps"][1].as_u64().unwrap() > 0);
        assert_eq!(p["executedSteps"][0], 0);
    }
    assert_eq!(results[0], results[1]);
}
#[test]
fn raw_expression_extra_fields_and_unreviewed_registration_shapes_reject() {
    assert!(
        serde_json::from_value::<Expr>(json!({"kind":"defeatCauseAmount","fallback":3})).is_err()
    );
    let value = catalogue();
    let raw = value["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_191")
        .unwrap();
    let lowered = crate::defeat_cause::prepare(raw).unwrap().unwrap();
    let mut ability: crate::ability_ir::Ability =
        serde_json::from_value(lowered["abilities"]["card"][0].clone()).unwrap();
    assert!(crate::ability_ir::Program::compile(&ability).is_ok());
    ability.optional = false;
    assert!(crate::ability_ir::Program::compile(&ability)
        .unwrap_err()
        .contains("exact optional Advantage"));
}
#[test]
fn cause_capture_preserves_outcast_physical_event_and_flock_number_binding() {
    let mut s = State::default();
    s.selected_number = 7;
    s.entry_event_card = 23;
    s.defeat_cause = DefeatCause::NonCombatDamage;
    let context = ResolutionContext::capture(&s);
    s.selected_number = 0;
    s.entry_event_card = 0;
    s.defeat_cause = DefeatCause::Unknown;
    context.restore(&mut s);
    assert_eq!(
        (s.selected_number, s.entry_event_card, s.defeat_cause),
        (7, 23, DefeatCause::NonCombatDamage)
    );
    let mut pending = State::default();
    pending.death_pending.entries[0] = DeathTrigger {
        event_card: 23,
        capture: TriggerCapture::new(2, DefeatCause::Attack),
        ..DeathTrigger::default()
    };
    let copy = pending;
    assert_eq!(copy.death_pending.entries[0].event_card, 23);
    assert_eq!(copy.death_pending.entries[0].capture.depth(), 2);
    assert_eq!(
        copy.death_pending.entries[0].capture.cause(),
        DefeatCause::Attack
    );
    let mut changed = pending;
    changed.death_pending.entries[0].event_card = 24;
    assert_ne!(changed.zobrist(), pending.zobrist());
    changed = pending;
    changed.death_pending.entries[0].capture = TriggerCapture::new(2, DefeatCause::Ability);
    assert_ne!(changed.zobrist(), pending.zobrist());
}

fn shoot_first(e: &Engine, s: &State, source: CardId, target: CardId) -> State {
    let (choosing, event) = play(e, s, "IC27_175");
    assert!(e.legal_actions(&choosing).iter().any(|a| a.label == "Cancel"));
    let ready = e.ready_resources(&choosing, s.active);
    assert_eq!(choosing.decision, Decision::AbilityTarget);
    let choosing = choose(e, &choosing, source);
    assert_eq!(choosing.decision, Decision::AttackTarget);
    assert!(choosing.player(s.active).discard.contains(event));
    assert!(e.ready_resources(&choosing, s.active) < ready);
    let base = s.player(s.active.other()).base;
    let mut expected = [target, base].into_iter().map(|id| e.card_action(&choosing, id).key()).collect::<Vec<_>>();
    let mut actual = e.legal_actions(&choosing).into_iter().map(|action| action.key()).collect::<Vec<_>>();
    expected.sort(); actual.sort();
    assert_eq!(actual, expected);
    assert!(e.apply(&choosing, &Action::button("Cancel", "cancel")).is_err());
    choose(e, &choosing, target)
}
fn assert_first_strike_advantage(e: &Engine, s: &State, fiend: CardId, host: CardId) {
    assert_eq!(s.decision, Decision::AbilityTarget);
    assert!(!e.in_arena(s, fiend));
    assert_eq!(s.defeat_cause, DefeatCause::Attack);
    assert_eq!(e.program_menu_title(s, fiend, s.ability_program - 1), "Give 2 Advantage tokens to a unit");
    let done = choose(e, s, host);
    assert_eq!(tokens(e, &done, host).len(), 2);
}
#[test]
fn shoot_first_first_damage_captures_attack_cause() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let attacker = unit(&e, &mut s, seat, "JTL_247");
            let fiend = unit(&e, &mut s, seat.other(), "ASH_191");
            let host = unit(&e, &mut s, seat.other(), "LOF_254");
            let pending = shoot_first(&e, &s, attacker, fiend);
            assert_eq!(pending.card(attacker).damage, 0);
            assert_first_strike_advantage(&e, &pending, fiend, host);
        }
    }
}
#[test]
fn shoot_first_return_damage_captures_attack_cause() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let fiend = unit(&e, &mut s, seat, "ASH_191");
            let defender = unit(&e, &mut s, seat.other(), "ASH_197");
            let host = unit(&e, &mut s, seat, "LOF_254");
            let pending = shoot_first(&e, &s, fiend, defender);
            assert!(e.in_arena(&pending, defender));
            assert_eq!(pending.card(defender).damage, 4);
            assert_first_strike_advantage(&e, &pending, fiend, host);
        }
    }
}
#[test]
fn shoot_first_shields_cancel_each_damage_window_without_fiend_trigger() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for fiend_attacks in [false, true] {
                let mut s = board(&e, seat);
                let fiend = unit(&e, &mut s, if fiend_attacks { seat } else { seat.other() }, "ASH_191");
                let other = unit(&e, &mut s, if fiend_attacks { seat.other() } else { seat }, "ASH_197");
                e.create_token(&mut s, fiend, false).unwrap();
                let (source, target) = if fiend_attacks { (fiend, other) } else { (other, fiend) };
                let done = shoot_first(&e, &s, source, target);
                assert!(e.in_arena(&done, fiend));
                assert_eq!(done.card(fiend).damage, 0);
                assert_eq!(done.card(fiend).shield, 0);
                assert_eq!(done.death_pending.len, 0);
                assert_eq!(done.decision, Decision::Window);
                assert!(tokens(&e, &done, other).is_empty());
            }
        }
    }
}
#[test]
fn first_strike_framework_peer_and_zero_damage_are_not_attack_defeats() {
    // Boundary positions intentionally inject an already framework-lethal peer.
    // This is a native producer test, not a legal engine-game witness.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for return_window in [false, true] {
                let mut s = board(&e, seat);
                let source = unit(&e, &mut s, seat, "ASH_197");
                let target = unit(&e, &mut s, seat.other(), "ASH_197");
                let peer = unit(&e, &mut s, seat, "ASH_191");
                s.card_mut(peer).phase_hp = -1;
                s.attack_source = source;
                s.attack_defender = target;
                // Zero combat damage must not classify an unrelated lethal
                // participant as a primary damage defeat either.
                s.card_mut(source).phase_power = -20;
                s.card_mut(target).phase_power = -20;
                if return_window { e.combat_after_first_window(&mut s).unwrap(); }
                else { e.combat_first_window(&mut s).unwrap(); }
                assert_eq!(s.decision, Decision::AbilityTarget);
                assert_eq!(s.defeat_cause, DefeatCause::FrameworkEffect);
                assert_eq!(e.program_menu_title(&s, peer, s.ability_program - 1), "Give 3 Advantage tokens to a unit");
                assert_eq!(s.card(source).damage, 0);
                assert_eq!(s.card(target).damage, 0);
                let done = choose(&e, &s, source);
                assert_eq!(tokens(&e, &done, source).len(), 3);
            }
        }
    }
}
#[test]
fn first_strike_mixed_physical_batch_retains_each_cause() {
    // Native producer boundary: the peer is already framework-lethal when
    // the actual combat damage recipient also dies in this same window.
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for return_window in [false, true] {
            let mut s = board(&e, seat);
            let primary = unit(&e, &mut s, seat.other(), "ASH_191");
            let other = unit(&e, &mut s, seat, "ASH_197");
            let peer = unit(&e, &mut s, seat.other(), "ASH_191");
            s.card_mut(peer).phase_hp = -1;
            let (source, target) = if return_window { (primary, other) } else { (other, primary) };
            s.attack_source = source;
            s.attack_defender = target;
            if return_window { e.combat_after_first_window(&mut s).unwrap(); }
            else { e.combat_first_window(&mut s).unwrap(); }
            assert_eq!(s.decision, Decision::DeathBatch);
            assert_eq!(s.death_window.len, 2);
            let events = &s.death_window.entries[..2];
            assert_eq!((events[0].source, events[0].capture.cause()), (primary, DefeatCause::Attack));
            assert_eq!((events[1].source, events[1].capture.cause()), (peer, DefeatCause::FrameworkEffect));
        }
    }
}
