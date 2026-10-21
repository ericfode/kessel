//! Native complete compiled definitions; source-engine comparisons are separate receipts.
use super::*;
fn engine(backend: RulesBackend) -> Engine {
    let v: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let mut v = v;
    v["cards"].as_array_mut().unwrap().retain(|c| {
        [
            "ASH_232",
            "TOKEN_SHIELD",
            "TOKEN_EXPERIENCE",
            "TOKEN_ADVANTAGE",
            "TOKEN_WEAKNESS",
            "TOKEN_THE_FORCE",
            "SOR_023",
            "SOR_014",
            "LOF_254",
            "SOR_095",
            "IC27_093",
        ]
        .contains(&c["setCode"].as_str().unwrap())
    });
    let mut e = Engine::from_catalogue_value(&v).unwrap();
    assert!(
        e.by_code.contains_key("ASH_232"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_232")
    );
    e.set_rules_backend(backend).unwrap();
    e
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e.by_code.get(code).unwrap(),
        owner: seat,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.active = seat;
    s.action_player = Some(seat);
    s.hidden_history_exact = true;
    s.credit_state_exact = false;
    s.force_state_exact = true;
    s.round_limit_wire_present = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = add(e, &mut s, p, "TOKEN_THE_FORCE");
        for _ in 0..24 {
            let id = add(e, &mut s, p, "LOF_254");
            s.player_mut(p).resources.push(id).unwrap();
        }
        for _ in 0..6 {
            let id = add(e, &mut s, p, "LOF_254");
            s.player_mut(p).deck.push(id).unwrap();
        }
    }
    s
}
fn event(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "ASH_232");
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{:?} {:?}",
        s.decision,
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label} at {:?}: {:?}", s.decision, e.legal_actions(s)));
    e.apply(s, &a).unwrap()
}

fn shield(e: &Engine, s: &State, id: CardId) -> Vec<CardId> {
    e.upgrades(s, id)
        .into_iter()
        .filter(|u| e.def(s, *u).code == "TOKEN_SHIELD")
        .collect()
}
fn attached(e: &Engine, s: &mut State, owner: Seat, host: CardId, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.attach(s, id, host);
    id
}
#[test]
fn surprises_paid_before_independent_choices_capture_and_copy_both_seats_backends() {
    for backend in [
        RulesBackend::Auto,
        RulesBackend::Interpreter,
        RulesBackend::Aot,
    ] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let host = unit(&e, &mut s, seat, "SOR_095");
            let other = unit(&e, &mut s, seat.other(), "SOR_095");
            let upgrade = attached(&e, &mut s, seat.other(), host, "IC27_093");
            let card = event(&e, &mut s, seat);
            s = choose(&e, &s, card);
            assert_eq!(s.decision, Decision::AbilityTarget);
            assert!(s.ability_committed);
            assert!(e.ready_resources(&s, seat) < 24);
            assert!(!e.legal_actions(&s).iter().any(|a| a.label == "Cancel"));
            s = choose(&e, &s, upgrade);
            assert!(e.attached_in_play(&s, upgrade));
            assert_eq!(s.decision, Decision::AbilityTarget);
            assert!(e.legal_actions(&s).contains(&e.card_action(&s, host)));
            assert!(e.legal_actions(&s).contains(&e.card_action(&s, other)));
            let copy = s;
            assert_eq!(copy.zobrist(), s.zobrist());
            let snapshot = e.snapshot(&s);
            assert!(
                e.load_state(&snapshot).unwrap_err().contains("actor"),
                "pending grouped decisions have no action-window actor"
            );
            let next = s.next_id;
            s = choose(&e, &copy, host);
            assert!(s.is_action_window());
            assert!(s.player(seat.other()).hand.contains(upgrade));
            assert!(!e.attached_in_play(&s, upgrade));
            assert_eq!(shield(&e, &s, host), vec![next]);
            assert!(shield(&e, &s, other).is_empty());
            let snapshot = e.snapshot(&s);
            let restored = e
                .load_state(&snapshot)
                .expect("completed action-window snapshot imports");
            assert_eq!(e.snapshot(&restored), snapshot);
        }
    }
}
#[test]
fn surprises_native_token_return_is_outside_play_and_new_shield_has_fresh_identity() {
    for backend in [
        RulesBackend::Auto,
        RulesBackend::Interpreter,
        RulesBackend::Aot,
    ] {
        let e = engine(backend);
        for code in [
            "TOKEN_SHIELD",
            "TOKEN_EXPERIENCE",
            "TOKEN_ADVANTAGE",
            "TOKEN_WEAKNESS",
        ] {
            for seat in [Seat::P1, Seat::P2] {
                let mut s = board(&e, seat);
                let host = unit(&e, &mut s, seat, "SOR_095");
                let other = unit(&e, &mut s, seat.other(), "SOR_095");
                let old = attached(&e, &mut s, seat.other(), host, code);
                let card = event(&e, &mut s, seat);
                s = choose(&e, &s, card);
                s = choose(&e, &s, old);
                let next = s.next_id;
                s = choose(&e, &s, other);
                assert!(!e.attached_in_play(&s, old));
                assert!(!s.player(seat.other()).hand.contains(old));
                assert!(!s.player(seat.other()).discard.contains(old));
                assert_eq!(shield(&e, &s, other), vec![next]);
                assert_ne!(next, old);
            }
        }
    }
}
#[test]
fn surprises_empty_upgrade_still_shields_and_no_targets_does_not_crash() {
    let e = engine(RulesBackend::Interpreter);
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let host = unit(&e, &mut s, seat, "SOR_095");
        let card = event(&e, &mut s, seat);
        s = choose(&e, &s, card);
        if s.decision == Decision::AbilityTarget {
            s = choose(&e, &s, host);
        }
        assert_eq!(shield(&e, &s, host).len(), 1);
        assert!(s.is_action_window());
        let mut s = board(&e, seat);
        let card = event(&e, &mut s, seat);
        let actions = e.legal_actions(&s);
        if actions.contains(&e.card_action(&s, card)) {
            s = choose(&e, &s, card);
            for _ in 0..3 {
                if s.is_action_window() {
                    break;
                }
                let acts = e.legal_actions(&s);
                if let Some(a) = acts
                    .iter()
                    .find(|a| a.label == "Play anyway" || a.label == "Play Anyway")
                {
                    s = e.apply(&s, a).unwrap()
                } else {
                    break;
                }
            }
            assert!(s.is_action_window());
        }
    }
}
#[test]
fn surprises_closed_source_unknown_fields_alias_and_program_guards() {
    let raw: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let entry = raw["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["setCode"] == "ASH_232")
        .unwrap();
    assert_eq!(
        crate::upgrade_return_shield_contract::prepare(entry),
        Ok(true)
    );
    let mut alias = entry.clone();
    alias["setCode"] = serde_json::json!("TEST_232");
    assert_eq!(
        crate::upgrade_return_shield_contract::prepare(&alias),
        Ok(true)
    );
    for field in ["unknown", "piloting", "attachedContext"] {
        let mut x = entry.clone();
        x[field] = serde_json::json!(true);
        assert!(crate::upgrade_return_shield_contract::prepare(&x).is_err());
    }
    for field in ["cost", "keywords", "types", "text"] {
        let mut x = entry.clone();
        x["card"][field] = serde_json::json!(null);
        assert!(crate::upgrade_return_shield_contract::prepare(&x).is_err());
    }
    let mut p = crate::upgrade_return_shield_contract::compiled().unwrap();
    assert!(crate::upgrade_return_shield_contract::program(&p));
    p.optional = true;
    assert!(!crate::upgrade_return_shield_contract::program(&p));
    assert_eq!(std::mem::size_of::<State>(), 39088);
}

fn full_engine() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
        .unwrap()
}
#[test]
fn surprises_hp_loss_defeats_host_only_after_new_shield_contingent() {
    let e = full_engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let host = unit(&e, &mut s, seat, "SOR_095");
        let other = unit(&e, &mut s, seat.other(), "SOR_095");
        let upgrade = attached(&e, &mut s, seat, host, "ASH_086");
        s.card_mut(host).damage = 3;
        assert_eq!(e.hp(&s, host), 4);
        let card = event(&e, &mut s, seat);
        s = choose(&e, &s, card);
        s = choose(&e, &s, upgrade);
        assert_eq!(e.hp(&s, host), 4);
        assert!(e.in_arena(&s, host));
        let next = s.next_id;
        s = choose(&e, &s, host);
        assert!(!e.in_arena(&s, host));
        assert!(s.player(seat).hand.contains(upgrade));
        assert!(s.player(seat).discard.contains(host));
        assert_eq!(s.next_id, next + 1);
        assert_eq!(e.def(&s, next).code, "TOKEN_SHIELD");
        assert!(!e.attached_in_play(&s, next));
        assert!(!s.player(seat).hand.contains(next) && !s.player(seat).discard.contains(next));
        assert!(shield(&e, &s, other).is_empty());
    }
}
#[test]
fn surprises_cost_three_upgrade_ineligible_and_existing_shields_stack() {
    let e = full_engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let host = unit(&e, &mut s, seat, "SOR_095");
        let other = unit(&e, &mut s, seat.other(), "SOR_095");
        let expensive = attached(&e, &mut s, seat, host, "ASH_087");
        let cheap = attached(&e, &mut s, seat.other(), host, "IC27_093");
        let old = attached(&e, &mut s, seat.other(), other, "TOKEN_SHIELD");
        let card = event(&e, &mut s, seat);
        s = choose(&e, &s, card);
        let actions = e.legal_actions(&s);
        assert!(actions.contains(&e.card_action(&s, cheap)));
        assert!(actions.contains(&e.card_action(&s, old)));
        assert!(!actions.contains(&e.card_action(&s, expensive)));
        s = choose(&e, &s, cheap);
        let next = s.next_id;
        s = choose(&e, &s, other);
        assert_eq!(shield(&e, &s, other), vec![old, next]);
        assert!(e.attached_in_play(&s, expensive));
    }
}
#[test]
fn surprises_missing_shield_or_changed_native_source_is_rejected() {
    let v: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    for mode in 0..2 {
        let mut c = v.clone();
        if mode == 0 {
            c["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|e| e["setCode"] != "TOKEN_SHIELD");
        } else {
            let shield = c["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["setCode"] == "TOKEN_SHIELD")
                .unwrap();
            shield["sourceHash"] = serde_json::json!("changed");
        }
        match Engine::from_catalogue_value(&c) {
            Ok(e) => assert!(!e.by_code.contains_key("ASH_232")),
            Err(e) => assert!(e.contains("Shield")),
        }
    }
}
#[test]
fn surprises_grouped_return_remains_closed_to_other_programs() {
    let raw: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let entry = raw["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["setCode"] == "ASH_232")
        .unwrap();
    let mut a: crate::ability_ir::Ability =
        serde_json::from_value(entry["abilities"]["card"][0].clone()).unwrap();
    assert!(crate::ability_ir::Program::compile(&a).is_err());
    a.effect = Effect::NoAction;
    let mut p = crate::ability_ir::Program::compile(&a).unwrap();
    p.instructions = crate::upgrade_return_shield_contract::compiled()
        .unwrap()
        .instructions;
    p.title = "different envelope".into();
    assert!(crate::upgrade_return_shield_contract::requires(&p));
    assert!(!crate::upgrade_return_shield_contract::program(&p));
}

#[test]
fn surprises_missing_upgrade_cost_guard_is_scoped_to_selected_and_copied_graphs() {
    let mut c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let bad = c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|v| v["setCode"] == "IC27_093")
        .unwrap();
    bad["card"]["cost"] = Value::Null;
    assert!(Engine::from_catalogue_value(&c)
        .err()
        .unwrap()
        .contains("WithCost upgrade"));
    let bytes = serde_json::to_vec(&c).unwrap();
    let raw = |id: &str| {
        let mut v = c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == id)
            .unwrap()["card"]
            .clone();
        v["setCode"] = serde_json::json!(id);
        v
    };
    let plain = serde_json::json!({"schema":"omni-rs-pool-v1","cards":[raw("TOKEN_SHIELD"),raw("SOR_023"),raw("SOR_014"),raw("LOF_254"),raw("SOR_095"),raw("IC27_093")]});
    let e = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&plain).unwrap(), &bytes).unwrap();
    let mut relevant = plain.clone();
    relevant["cards"]
        .as_array_mut()
        .unwrap()
        .push(raw("ASH_232"));
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&relevant).unwrap(), &bytes)
            .err()
            .unwrap()
            .contains("WithCost upgrade")
    );
    let mut unselected = plain.clone();
    unselected["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["setCode"] != "IC27_093");
    unselected["cards"]
        .as_array_mut()
        .unwrap()
        .push(raw("ASH_232"));
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&unselected).unwrap(), &bytes).is_ok()
    );
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "SOR_095");
    let _bad = attached(&e, &mut s, Seat::P1, host, "IC27_093");
    let source = event(&e, &mut s, Seat::P1);
    assert!(e
        .validate_upgrade_return_graph(Some(&s))
        .unwrap_err()
        .contains("WithCost upgrade"));
    let before = s;
    assert!(e.apply(&s, &e.card_action(&s, source)).is_err());
    assert_eq!(s.zobrist(), before.zobrist());
}

#[test]
fn surprises_program_reservation_includes_stored_plus_one_sentinel() {
    for (start, expected) in [
        (0, Some(0)),
        (65534, Some(65534)),
        (65535, None),
        (65536, None),
        (usize::MAX, None),
    ] {
        assert_eq!(
            crate::upgrade_return_shield_contract::checked_program_id(start).ok(),
            expected
        );
    }
}

#[test]
fn surprises_insufficient_resources_and_shield_capacity_fail_before_payment() {
    let e = engine(RulesBackend::Interpreter);
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "SOR_095");
    let upgrade = attached(&e, &mut s, Seat::P1, host, "IC27_093");
    let source = event(&e, &mut s, Seat::P1);
    let a = e.card_action(&s, source);
    for id in s.player(Seat::P1).resources.as_slice().to_vec() {
        s.card_mut(id).exhausted = true;
    }
    assert!(!e.legal_actions(&s).contains(&a));
    for id in s.player(Seat::P1).resources.as_slice().to_vec() {
        s.card_mut(id).exhausted = false;
    }
    s.next_id = MAX_CARDS as u16;
    let before = s;
    assert!(e.apply(&s, &a).unwrap_err().contains("capacity"));
    assert_eq!(s.zobrist(), before.zobrist());
    assert!(e.attached_in_play(&s, upgrade));
    assert_eq!(e.ready_resources(&s, Seat::P1), 24);
}
