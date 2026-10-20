//! Source-derived heal batch diagnostics; engine witnesses are separate.
use super::*;
use crate::ability_ir::Ability;
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
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_065")
        .unwrap()
        .clone()
}
fn admitted(v: Value) -> bool {
    crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
        .unwrap()
        .cards
        .contains_key("ASH_065")
}
fn effect() -> Effect {
    serde_json::from_value(json!({"kind":"recipientHeal","target":crate::recipient_heal_contract::target(),"amount":crate::recipient_heal_contract::amount()})).unwrap()
}
fn home(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    unit(e, s, seat, "ASH_065")
}
#[test]
fn recipient_heal_full_source_and_raw_callback_contract_are_closed() {
    let original = entry();
    assert!(admitted(original.clone()));
    let e = engine();
    assert!(e.by_code.contains_key("ASH_065"));
    let mut renamed = original.clone();
    renamed["setCode"] = json!("TEST_HOME");
    renamed["internalName"] = json!("same-semantics");
    assert!(crate::ability_ir::Catalogue::load(
        &json!({"schema":"swu-card-ir/v1","cards":[renamed]})
    )
    .unwrap()
    .cards
    .contains_key("TEST_HOME"));
    for (path, value) in [
        ("/baseClass", json!("EventCard")),
        ("/card/types", json!(["unit", "leader"])),
        ("/card/unique", json!(false)),
        ("/card/arena", json!("ground")),
        ("/card/hp", json!(11)),
        ("/keywordValues/sentinel", json!(0)),
        ("/abilities/card/0/trigger", json!("onAttack")),
        ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
        (
            "/abilities/card/0/effect/target/receiver/path",
            json!(["opponent"]),
        ),
        (
            "/abilities/card/0/effect/target/predicate/value",
            json!(false),
        ),
        (
            "/abilities/card/0/effect/amount/callback/body/path",
            json!(["attackBinding1_0", "hp"]),
        ),
        (
            "/abilities/card/0/effect/amount/callback/parameters/0/role",
            json!("context"),
        ),
        (
            "/abilities/card/0/effect/amount/evaluateAt",
            json!("handler"),
        ),
        (
            "/abilities/card/0/effect/amount/legalityComparison",
            json!("callbackResultComparedToZero"),
        ),
        ("/abilities/card/0/effect/kind", json!("recipientHeal")),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(!admitted(bad), "{path}");
    }
    for (key, value) in [
        ("optional", json!(true)),
        ("requiresConfirmation", json!(true)),
        ("condition", json!({"kind":"const","value":true})),
        ("unknown", json!(true)),
    ] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][key] = value;
        assert!(!admitted(bad), "{key}");
    }
    for key in ["classMethods", "stateWatchers", "privatePrograms"] {
        let mut bad = original.clone();
        bad[key] = json!([{"unreviewed":true}]);
        assert!(!admitted(bad));
    }
    for side in ["leader", "unit", "card"] {
        let mut bad = original.clone();
        bad["abilities"][side]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(!admitted(bad));
    }
    let mut a = original["abilities"]["card"][0].clone();
    a["effect"]["kind"] = json!("recipientHeal");
    let valid: Ability = serde_json::from_value(a.clone()).unwrap();
    assert!(Program::compile(&valid).is_ok());
    for (key, value) in [
        ("optional", json!(true)),
        ("trigger", json!("event")),
        (
            "targetResolver",
            json!({"owner":"self","type":"unit","zone":"arena"}),
        ),
        (
            "then",
            json!({"title":"extra","effect":{"kind":"noAction"}}),
        ),
    ] {
        let mut bad = a.clone();
        bad[key] = value;
        assert!(Program::compile(&serde_json::from_value(bad).unwrap()).is_err());
    }
    for container in ["target", "amount"] {
        let mut bad = a.clone();
        bad["effect"][container]["extra"] = json!(true);
        assert!(Program::compile(&serde_json::from_value(bad).unwrap()).is_err());
    }
    let mut bad = a.clone();
    bad["effect"] = json!({"kind":"simultaneous","effects":[a["effect"].clone()]});
    assert!(Program::compile(&serde_json::from_value(bad).unwrap()).is_err());
}
#[test]
fn recipient_heal_capacity_all_recipients_distinct_damage_no_selector_or_record_limit() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for count in [0, 1, 40, 127] {
            let mut s = board(&e, seat);
            let source = home(&e, &mut s, seat);
            let mut ids = vec![];
            for n in 0..count {
                let code = if n % 2 == 0 { "LOF_256" } else { "JTL_251" };
                let id = unit(&e, &mut s, seat, code);
                s.card_mut(id).damage = (n % 3 + 1) as u16;
                ids.push(id);
            }
            let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
            s.card_mut(enemy).damage = 2;
            s.source = source;
            let expected = ids.iter().map(|id| s.card(*id).damage).sum::<u16>();
            let captured = e.recipient_heal_capture(&s, source).unwrap();
            assert_eq!(captured.len(), count + 1);
            // Captured order is friendly ground, then friendly space, including source.
            let p = s.player(seat);
            assert_eq!(
                captured.iter().map(|x| x.0).collect::<Vec<_>>(),
                p.ground
                    .as_slice()
                    .iter()
                    .chain(p.space.as_slice())
                    .copied()
                    .collect::<Vec<_>>()
            );
            let result = e.apply_effect(&mut s, &effect()).unwrap();
            assert_eq!(result.0, expected);
            assert_eq!(result.1, count > 0);
            for id in ids {
                assert_eq!(s.card(id).damage, 0);
                assert!(e.in_arena(&s, id));
            }
            assert_eq!(s.card(enemy).damage, 2);
            assert_eq!(s.card(source).damage, 0);
        }
    }
}
#[test]
fn recipient_heal_captured_amount_live_clamp_restriction_and_membership() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = home(&e, &mut s, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "LOF_256");
    let b = unit(&e, &mut s, Seat::P1, "LOF_256");
    let blocked = unit(&e, &mut s, Seat::P1, "LOF_256");
    s.card_mut(a).damage = 2;
    s.card_mut(b).damage = 3;
    s.card_mut(blocked).damage = 2;
    let captured = e.recipient_heal_capture(&s, source).unwrap();
    s.card_mut(a).damage = 4;
    s.card_mut(b).damage = 1;
    s.card_mut(blocked).phase_restrictions |= RestrictionVerb::BeHealed.bit();
    let late = unit(&e, &mut s, Seat::P1, "LOF_256");
    s.card_mut(late).damage = 2;
    let (amount, resolved) = e.recipient_heal_resolve(&mut s, &captured).unwrap();
    assert_eq!((amount, resolved), (3, true));
    assert_eq!(s.card(a).damage, 2);
    assert_eq!(s.card(b).damage, 0);
    assert_eq!(s.card(blocked).damage, 2);
    assert_eq!(s.card(late).damage, 2);
}
#[test]
fn recipient_heal_actual_play_uniqueness_retains_dead_source_and_sentinel_is_arena_scoped() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for keep_old in [false, true] {
            let mut s = board(&e, seat);
            let old = home(&e, &mut s, seat);
            s.card_mut(old).damage = 4;
            let friendly = unit(&e, &mut s, seat, "LOF_256");
            s.card_mut(friendly).damage = 2;
            let incoming = add(&e, &mut s, seat, "ASH_065");
            s.player_mut(seat).hand.push(incoming).unwrap();
            let unique = choose(&e, &s, incoming);
            assert_eq!(unique.decision, Decision::Unique);
            let victim = if keep_old { incoming } else { old };
            let done = choose(&e, &unique, victim);
            assert!(done.player(seat).discard.contains(victim));
            assert_eq!(done.card(friendly).damage, 0);
            let survivor = if keep_old { old } else { incoming };
            assert!(e.has_keyword(&done, survivor, "sentinel"));
            assert_eq!(done.card(survivor).damage, 0);
            let mut board = done;
            let ground = unit(&e, &mut board, seat.other(), "LOF_256");
            let space = unit(&e, &mut board, seat.other(), "JTL_251");
            assert!(!e
                .attack_targets(&board, space, false)
                .contains(&board.player(seat).base));
            assert!(e
                .attack_targets(&board, ground, false)
                .contains(&board.player(seat).base));
        }
    }
}
#[test]
fn recipient_heal_unknown_damage_healed_listener_profiles_remain_rejected() {
    let e = engine();
    for code in ["JTL_062", "LAW_047", "TS26_047"] {
        assert!(!e.by_code.contains_key(code));
        assert!(e.rejected_cards().unwrap().contains_key(code));
    }
}
#[test]
fn recipient_heal_one_final_sweep_synthetic_damage_dependent_aura() {
    for blocked in [false, true] {
        let mut e = engine();
        let mut s = board(&e, Seat::P1);
        let source = home(&e, &mut s, Seat::P1);
        let aura = unit(&e, &mut s, Seat::P1, "SOR_230");
        let victim = unit(&e, &mut s, Seat::P1, "ASH_061");
        // Synthetic conditional on an actual aura program, deliberately bypassing
        // source admission solely to distinguish one sweep from per-heal sweeps.
        let p = e.card_programs(&s, aura)[0];
        e.catalogue.as_mut().unwrap().programs[p as usize].condition = Some(Expr::Ref {
            path: vec!["source".into(), "damage".into()],
        });
        for instruction in &mut e.catalogue.as_mut().unwrap().programs[p as usize].instructions {
            if let Instruction::Select(resolver) = instruction {
                resolver.predicate = None;
            }
        }

        s.card_mut(aura).damage = 1;
        let hp = e.hp(&s, victim);
        s.card_mut(victim).damage = hp - 1;
        let unbuffed = e.def(&s, victim).info.hp.unwrap() as u16;
        assert!(s.card(victim).damage >= unbuffed);
        if blocked {
            s.card_mut(victim).phase_restrictions |= RestrictionVerb::BeHealed.bit();
        }
        s.source = source;
        e.apply_effect(&mut s, &effect()).unwrap();
        assert_eq!(e.in_arena(&s, victim), !blocked);
        assert_eq!(s.card(aura).damage, 0);
        if blocked {
            assert!(s.player(Seat::P1).discard.contains(victim));
        } else {
            assert_eq!(s.card(victim).damage, 0);
        }
    }
}
#[test]
fn recipient_heal_extreme_aggregate_applies_every_physical_heal_before_protocol_saturation() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = home(&e, &mut s, Seat::P1);
    let mut recipients = vec![];
    // Synthetic high-HP state; not an ordinary printed-card setup or an engine
    // claim about aggregate return values. More than u16::MAX damage is removed.
    for n in 0..127 {
        let id = unit(
            &e,
            &mut s,
            Seat::P1,
            if n % 2 == 0 { "LOF_256" } else { "JTL_251" },
        );
        s.card_mut(id).phase_hp = 65000;
        s.card_mut(id).damage = 64000;
        recipients.push(id);
    }
    s.source = source;
    assert_eq!(e.apply_effect(&mut s, &effect()).unwrap(), (u16::MAX, true));
    for id in recipients {
        assert_eq!(s.card(id).damage, 0);
        assert!(e.in_arena(&s, id));
    }
    // A separate program resets the bounded numeric protocol before it can read
    // any prior result; the source contract forbids then/ifYouDo/sequence/selector.
    let p = e.card_programs(&s, source)[0];
    s.ability_result = u16::MAX;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.ability_result, 0);
}
#[test]
fn recipient_heal_empty_undamaged_and_restricted_feasibility_remains_distinct() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = home(&e, &mut s, Seat::P1);
    s.source = source;
    assert!(e.effect_can_affect(&s, source, 0, &effect(), false));
    assert!(!e.effect_can_affect(&s, source, 0, &effect(), true));
    assert_eq!(e.apply_effect(&mut s, &effect()).unwrap(), (0, false));
    s.card_mut(source).damage = 3;
    s.card_mut(source).phase_restrictions |= RestrictionVerb::BeHealed.bit();
    assert!(!e.effect_can_affect(&s, source, 0, &effect(), true));
    assert_eq!(e.apply_effect(&mut s, &effect()).unwrap(), (0, false));
    assert_eq!(s.card(source).damage, 3);
    e.defeat(&mut s, source).unwrap();
    assert_eq!(e.apply_effect(&mut s, &effect()).unwrap(), (0, false));
}

#[test]
fn recipient_heal_capture_rejects_corrupt_membership_without_mutation() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = home(&e, &mut s, Seat::P1);
    s.source = source;
    s.card_mut(source).damage = 2;
    s.player_mut(Seat::P1).space.push(source).unwrap();
    let before = s;
    assert!(e.apply_effect(&mut s, &effect()).is_err());
    assert_eq!(s, before);
}

#[test]
fn recipient_heal_auto_and_interpreter_share_explicit_batch_fallback() {
    let auto = engine();
    let mut interpreted = engine();
    interpreted
        .set_rules_backend(RulesBackend::Interpreter)
        .unwrap();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&auto, seat);
        let target = unit(&auto, &mut s, seat, "LOF_256");
        s.card_mut(target).damage = 2;
        let source = add(&auto, &mut s, seat, "ASH_065");
        s.player_mut(seat).hand.push(source).unwrap();
        let expected = choose(&interpreted, &s, source);
        let actual = choose(&auto, &s, source);
        assert_eq!(actual, expected);
        assert_eq!(actual.card(target).damage, 0);
        let p = auto.card_programs(&actual, source)[0];
        assert!(
            auto.program(p).executable.is_err(),
            "batch deliberately remains interpreter-owned"
        );
    }
}

#[test]
fn recipient_heal_intrinsic_sentinel_survives_real_phase_cleanup() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = home(&e, &mut s, Seat::P1);
    s.source = source;
    s.card_mut(source).damage = 2;
    e.apply_effect(&mut s, &effect()).unwrap();
    s.card_mut(source).phase_power = 1;
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.card(source).phase_power, 0);
    assert!(e.has_keyword(&s, source, "sentinel"));
    assert_eq!(s.card(source).damage, 0);
}
