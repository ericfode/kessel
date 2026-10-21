//! Native complete compiled definitions; source-engine comparisons are separate receipts.
use super::*;
fn engine(backend: RulesBackend) -> Engine {
    let v: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let mut e = Engine::from_catalogue_value(&v).unwrap();
    assert!(
        e.by_code.contains_key("ASH_231"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_231")
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
    s.credit_state_exact = true;
    s.force_state_exact = true;
    s.round_limit_wire_present = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
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
    let id = add(e, s, seat, "ASH_231");
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
fn advantage(e: &Engine, s: &State, id: CardId) -> Vec<CardId> {
    e.upgrades(s, id)
        .into_iter()
        .filter(|id| e.def(s, *id).code == "TOKEN_ADVANTAGE")
        .collect()
}
#[test]
fn named_exhaust_native_ready_and_selected_ineffective_last_event_both_seats() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for friendly_exhausted in [false, true] {
                for enemy_exhausted in [false, true] {
                    let mut s = board(&e, seat);
                    let friendly = unit(&e, &mut s, seat, "SOR_095");
                    let enemy = unit(&e, &mut s, seat.other(), "SOR_095");
                    let ready_friendly = unit(&e, &mut s, seat, "SOR_095");
                    let ready_enemy = unit(&e, &mut s, seat.other(), "SOR_095");
                    s.card_mut(friendly).exhausted = friendly_exhausted;
                    s.card_mut(enemy).exhausted = enemy_exhausted;
                    let card = event(&e, &mut s, seat);
                    s = choose(&e, &s, card);
                    assert_eq!(s.decision, Decision::AbilityTarget);
                    assert_eq!(
                        e.legal_actions(&s),
                        vec![
                            Action::button("Cancel", "cancel"),
                            e.card_action(&s, friendly),
                            e.card_action(&s, ready_friendly)
                        ]
                    );
                    assert_eq!(e.ready_resources(&s, seat), 24);
                    s = choose(&e, &s, friendly);
                    assert_eq!(s.ability_selection.ids[..2], [friendly, 0]);
                    assert_eq!(s.decision, Decision::AbilityTarget);
                    assert_eq!(
                        e.legal_actions(&s),
                        vec![
                            Action::button("Cancel", "cancel"),
                            e.card_action(&s, enemy),
                            e.card_action(&s, ready_enemy)
                        ]
                    );
                    assert_eq!(e.ready_resources(&s, seat), 24);
                    let copy = s;
                    assert_eq!(copy.zobrist(), s.zobrist());
                    let mut changed = s;
                    changed.ability_selection.ids[0] = ready_friendly;
                    assert_ne!(changed.zobrist(), s.zobrist());
                    let snapshot = e.snapshot(&s);
                    assert_eq!(snapshot["namedExhaustPending"], true);
                    assert!(e
                        .load_state(&snapshot)
                        .unwrap_err()
                        .contains("pending named exhaust"));
                    s = choose(&e, &s, enemy);
                    assert!(s.is_action_window());
                    assert!(s.player(seat).discard.contains(card));
                    assert_eq!(e.ready_resources(&s, seat), 21);
                    assert!(s.card(friendly).exhausted && s.card(enemy).exhausted);
                    assert!(!s.card(ready_friendly).exhausted && !s.card(ready_enemy).exhausted);
                    let tokens = advantage(&e, &s, friendly);
                    assert_eq!(tokens.len(), if enemy_exhausted { 0 } else { 2 });
                    assert!(advantage(&e, &s, enemy).is_empty());
                    if tokens.len() == 2 {
                        assert_ne!(tokens[0], tokens[1]);
                        assert!(tokens
                            .iter()
                            .all(|id| s.card(*id).owner == seat && s.card(*id).parent == friendly));
                    }
                }
            }
        }
    }
}
#[test]
fn named_exhaust_native_missing_domains_and_no_effect_confirmation_are_distinct() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let card = event(&e, &mut s, seat);
            let initial = s;
            s = choose(&e, &s, card);
            assert_eq!(s.decision, Decision::AbilityNoEffect);
            assert_eq!(
                e.legal_actions(&s)
                    .iter()
                    .map(|a| a.label.as_str())
                    .collect::<Vec<_>>(),
                ["Play anyway", "Cancel"]
            );
            let cancelled = button(&e, &s, "Cancel");
            assert_eq!(cancelled, initial);
            s = button(&e, &s, "Play anyway");
            assert!(s.is_action_window());
            assert_eq!(e.ready_resources(&s, seat), 21);
            assert!(s.player(seat).discard.contains(card));
            let mut s = board(&e, seat);
            let f = unit(&e, &mut s, seat, "SOR_095");
            let n = unit(&e, &mut s, seat.other(), "SOR_095");
            s.card_mut(f).exhausted = true;
            s.card_mut(n).exhausted = true;
            let card = event(&e, &mut s, seat);
            s = choose(&e, &s, card);
            assert!(s.is_action_window());
            assert_eq!(e.ready_resources(&s, seat), 21);
            assert!(advantage(&e, &s, f).is_empty());
            for own in [false, true] {
                let mut s = board(&e, seat);
                let who = if own { seat } else { seat.other() };
                let target = unit(&e, &mut s, who, "SOR_095");
                let card = event(&e, &mut s, seat);
                s = choose(&e, &s, card);
                if s.decision == Decision::AbilityTarget {
                    s = choose(&e, &s, target);
                }
                assert!(s.is_action_window());
                assert_eq!(e.ready_resources(&s, seat), 21);
                assert!(s.card(target).exhausted);
                assert!(advantage(&e, &s, target).is_empty());
            }
        }
    }
}
#[test]
fn named_exhaust_native_pretarget_cancel_restores_entire_state_and_can_retry() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for second in [false, true] {
                let mut s = board(&e, seat);
                let f = unit(&e, &mut s, seat, "SOR_095");
                unit(&e, &mut s, seat.other(), "SOR_095");
                let card = event(&e, &mut s, seat);
                let initial = s;
                s = choose(&e, &s, card);
                if second {
                    s = choose(&e, &s, f);
                }
                s = button(&e, &s, "Cancel");
                assert_eq!(s, initial);
                assert_eq!(s.zobrist(), initial.zobrist());
                assert!(e.legal_actions(&s).contains(&e.card_action(&s, card)));
            }
        }
    }
}
#[test]
fn named_exhaust_native_closed_program_and_raw_source_dependency_guards() {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let raw = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_231")
        .unwrap();
    assert!(crate::named_exhaust_contingent_contract::prepare(raw).unwrap());
    let a: crate::ability_ir::Ability =
        serde_json::from_value(raw["abilities"]["card"][0].clone()).unwrap();
    assert!(Program::compile(&a).is_err());
    let p = crate::named_exhaust_contingent_contract::compiled().unwrap();
    assert!(crate::named_exhaust_contingent_contract::program(&p));
    let mut bad = p.clone();
    bad.instructions.pop();
    assert!(!crate::named_exhaust_contingent_contract::program(&bad));
    let mut missing = c.clone();
    missing["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
    let e = Engine::from_catalogue_value(&missing).unwrap();
    assert!(!e.by_code.contains_key("ASH_231"));
    assert!(e.rejected_cards().unwrap()["ASH_231"].contains("complete reviewed native Advantage"));
}
#[test]
fn named_exhaust_native_binding_context_restore_capacity_and_import_fail_atomic() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let f = unit(&e, &mut s, seat, "SOR_095");
            let n = unit(&e, &mut s, seat.other(), "SOR_095");
            let card = event(&e, &mut s, seat);
            s = choose(&e, &s, card);
            s = choose(&e, &s, f);
            let before = s;
            let context = ResolutionContext::capture(&s);
            s.ability_selection = Selection::one(n);
            s.ability_target = n;
            s.ability_succeeded = true;
            context.restore(&mut s);
            assert_eq!(s, before);
            assert_eq!(s.zobrist(), before.zobrist());
            let mut no_capacity = s;
            no_capacity.next_id = MAX_CARDS as CardId - 1;
            let a = e.card_action(&no_capacity, n);
            let unchanged = no_capacity;
            assert!(e.apply(&no_capacity, &a).unwrap_err().contains("capacity"));
            assert_eq!(no_capacity, unchanged);
            for marker in [json!(true), json!(false), Value::Null] {
                let mut snapshot = e.snapshot(&s);
                snapshot["namedExhaustPending"] = marker;
                assert!(e
                    .load_state(&snapshot)
                    .unwrap_err()
                    .contains("pending named exhaust"));
            }
        }
    }
}
#[test]
fn named_exhaust_native_token_definition_unknown_payload_is_not_enriched_or_stripped() {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    for key in ["className", "nativeTokenGraph", "diagnostics", "unexpected"] {
        let mut bad = c.clone();
        let token = bad["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["setCode"] == "TOKEN_ADVANTAGE")
            .unwrap();
        token[key] = json!({"unknown":true});
        let e = Engine::from_catalogue_value(&bad).unwrap();
        assert!(!e.by_code.contains_key("ASH_231"), "{key}");
    }
}

#[test]
fn named_exhaust_standalone_program_zero_preserves_ordinary_group_or_success() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for initially_exhausted in [false, true] {
                let mut s = board(&e, seat);
                let source = unit(&e, &mut s, seat, "SOR_095");
                s.source = source;
                s.card_mut(source).exhausted = initially_exhausted;
                assert_eq!(s.ability_program, 0);
                s.event_group.active = true;
                s.event_group.initiator = seat;
                let group = Effect::Simultaneous { effects: vec![
                    Effect::Exhaust { target: Target::Name("self".into()) },
                    Effect::Exhaust { target: Target::Name("self".into()) },
                ] };
                e.generate_group(&mut s, &group).unwrap();
                assert_eq!(s.event_group.count, 2);
                e.resolve_group(&mut s, &group).unwrap();
                assert!(s.card(source).exhausted);
                assert_eq!(s.ability_succeeded, !initially_exhausted);
                assert_eq!(s.ability_result, if initially_exhausted {0} else {1});
            }
        }
    }
}
