//! Synthetic native cases; the 52 source games remain independent Root conformance work.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    for c in ["ASH_045", "ASH_133"] {
        assert!(
            e.by_code.contains_key(c),
            "{c}: {:?}",
            e.rejected_cards().unwrap().get(c)
        );
    }
    e.set_rules_backend(backend).unwrap();
    e
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e.by_code.get(code).unwrap_or_else(|| panic!("{code}")),
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
    s.force_state_exact = true;
    s.credit_state_exact = true;
    s.round_limit_wire_present = true;
    for owner in [seat, seat.other()] {
        s.player_mut(owner).base = add(e, &mut s, owner, "SOR_023");
        s.player_mut(owner).leader = add(e, &mut s, owner, "SOR_014");
        s.player_mut(owner).force_id = add(e, &mut s, owner, "TOKEN_THE_FORCE");
        for _ in 0..24 {
            let id = add(e, &mut s, owner, "LOF_254");
            s.player_mut(owner).resources.push(id).unwrap();
        }
        for _ in 0..5 {
            let id = add(e, &mut s, owner, "LOF_254");
            s.player_mut(owner).deck.push(id).unwrap();
        }
    }
    s
}
fn click(e: &Engine, s: &mut State, label: &str) {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)));
    *s = e.apply(s, &a).unwrap();
}
fn start(e: &Engine, s: &mut State, source: CardId) {
    s.source = source;
    s.active = s.card(source).owner;
    let p = e.card_programs(s, source)[0];
    e.begin_program(s, p, ProgramContinuation::Finish).unwrap();
}

#[test]
fn modal_deck_closed_profiles_and_unknowns() {
    for k in [
        crate::modal_deck_movement_contract::Kind::Top,
        crate::modal_deck_movement_contract::Kind::Recovery,
    ] {
        let v = crate::modal_deck_movement_contract::profile(k);
        assert_eq!(
            crate::modal_deck_movement_contract::prepare(&v),
            Ok(Some(k))
        );
        let (_, ps) = crate::modal_deck_movement_contract::compile(k, 0).unwrap();
        for p in ps {
            assert!(crate::modal_deck_movement_contract::program(&p));
        }
        for path in [
            "/sourceHash",
            "/setCode",
            "/baseClass",
            "/card/title",
            "/card/cost",
            "/abilities/card/0/trigger",
        ] {
            let mut b = v.clone();
            *b.pointer_mut(path).unwrap() = json!("unreviewed");
            assert!(
                crate::modal_deck_movement_contract::prepare(&b).is_err(),
                "{path}"
            );
        }
        let mut erased = v.clone();
        for key in ["sourceHash", "className", "internalName"] {
            erased[key] = json!("foreign");
        }
        erased["abilities"]["card"] = json!([{"trigger":"whenPlayed","title":"erased","costs":[],"effect":{"kind":"noAction"}}]);
        assert!(crate::modal_deck_movement_contract::prepare(&erased).is_err());
        for path in ["", "/card", "/abilities/card/0"] {
            let mut b = v.clone();
            b.pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(true));
            assert!(crate::modal_deck_movement_contract::prepare(&b).is_err());
        }
        let mut b = v.clone();
        b["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","effect":{"kind":"heal","target":"ownBase","amount":{"kind":"const","value":1}}}));
        assert!(crate::modal_deck_movement_contract::prepare(&b).is_err());
    }
    assert_eq!(std::mem::size_of::<State>(), 38312);
}
#[test]
fn modal_deck_night_trooper_both_decks_buttons_empty_and_sequential_capture() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for enemy in [false, true] {
                for discard in [false, true] {
                    let mut s = board(&e, seat);
                    let source = unit(&e, &mut s, seat, "ASH_045");
                    let side = if enemy { seat.other() } else { seat };
                    let top = s.player(side).deck.as_slice()[0];
                    e.defeat(&mut s, source).unwrap();
                    e.finish(&mut s).unwrap();
                    assert_eq!(
                        e.legal_actions(&s)
                            .iter()
                            .map(|a| a.label.as_str())
                            .collect::<Vec<_>>(),
                        vec!["Your deck", "Opponent's deck"]
                    );
                    assert!(s.player(seat).discard.contains(source));
                    click(
                        &e,
                        &mut s,
                        if enemy {
                            "Opponent's deck"
                        } else {
                            "Your deck"
                        },
                    );
                    assert_eq!(s.decision, Decision::AbilityDisplay);
                    assert_eq!(s.active, seat);
                    let actions = e.legal_actions(&s);
                    assert_eq!(actions.len(), 2);
                    assert!(actions
                        .iter()
                        .all(|a| a.kind == "perCardButton"
                            && a.card == Some(format!("c{}", top - 1))));
                    let copied = s;
                    assert_eq!(copied.zobrist(), s.zobrist());
                    assert!(e.snapshot(&s)["dependentChoicePending"].as_bool().unwrap());
                    assert!(e.load_state(&e.snapshot(&s)).is_err());
                    let a = actions[usize::from(!discard)].clone();
                    s = e.apply(&s, &a).unwrap();
                    assert_eq!(s.player(side).discard.contains(top), discard);
                    assert_eq!(s.player(side).deck.contains(top), !discard);
                    assert_eq!(s.ability_program, 0);
                }
            }
            let mut s = board(&e, seat);
            s.player_mut(seat).deck = Default::default();
            let source = unit(&e, &mut s, seat, "ASH_045");
            e.defeat(&mut s, source).unwrap();
            e.finish(&mut s).unwrap();
            click(&e, &mut s, "Your deck");
            assert_eq!(s.ability_program, 0);
            assert_ne!(s.decision, Decision::AbilityDisplay);
            let mut s = board(&e, seat);
            let a = unit(&e, &mut s, seat, "ASH_045");
            let b = unit(&e, &mut s, seat, "ASH_045");
            let first = s.player(seat).deck.as_slice()[0];
            let second = s.player(seat).deck.as_slice()[1];
            e.defeat_events(
                &mut s,
                &[
                    (a, crate::defeat_cause::DefeatCause::FrameworkEffect),
                    (b, crate::defeat_cause::DefeatCause::FrameworkEffect),
                ],
            )
            .unwrap();
            e.finish(&mut s).unwrap();
            for expected in [first, second] {
                while s.decision != Decision::AbilityTarget {
                    let ac = e.legal_actions(&s)[0].clone();
                    s = e.apply(&s, &ac).unwrap();
                }
                click(&e, &mut s, "Your deck");
                assert_eq!(s.ability_target, expected);
                let ac = e.legal_actions(&s)[0].clone();
                s = e.apply(&s, &ac).unwrap();
            }
            assert!(
                s.player(seat).discard.contains(first) && s.player(seat).discard.contains(second)
            );
        }
    }
}
#[test]
fn modal_deck_trask_select_cost_owner_modal_both_triggers_and_heal_three() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for trigger in 0..2 {
                for hand in [false, true] {
                    for wound in [0, 1, 2, 3, 8] {
                        let mut s = board(&e, seat);
                        let source = unit(&e, &mut s, seat, "ASH_133");
                        let target = add(&e, &mut s, seat, "HMW_093");
                        s.player_mut(seat).discard.push(target).unwrap();
                        let high = add(&e, &mut s, seat, "ASH_133");
                        s.player_mut(seat).discard.push(high).unwrap();
                        let foreign = add(&e, &mut s, seat.other(), "HMW_093");
                        s.player_mut(seat.other()).discard.push(foreign).unwrap();
                        let base = s.player(seat).base;
                        s.card_mut(base).damage = wound;
                        s.source = source;
                        s.active = seat;
                        let p = e.card_programs(&s, source)[trigger];
                        e.begin_program(&mut s, p, ProgramContinuation::Finish)
                            .unwrap();
                        assert_eq!(s.decision, Decision::AbilityTarget);
                        let acts = e.legal_actions(&s);
                        assert!(acts.contains(&e.card_action(&s, target)));
                        assert!(
                            !acts.contains(&e.card_action(&s, high))
                                && !acts.contains(&e.card_action(&s, foreign))
                        );
                        s = e.apply(&s, &e.card_action(&s, target)).unwrap();
                        assert_eq!(s.ability_target, target);
                        let ac = e.legal_actions(&s);
                        assert_eq!(ac.len(), 2);
                        assert!(!ac.iter().any(|a| a.label.contains("Cancel")));
                        s = e.apply(&s, &ac[usize::from(hand)]).unwrap();
                        assert_eq!(s.player(seat).hand.contains(target), hand);
                        assert_eq!(s.player(seat).deck.contains(target), !hand);
                        assert!(!s.player(seat).discard.contains(target));
                        assert_eq!(
                            s.card(base).damage,
                            if hand { wound } else { wound.saturating_sub(3) }
                        );
                        if !hand {
                            assert_eq!(s.player(seat).deck.as_slice().last(), Some(&target));
                        }
                        assert_eq!(s.ability_program, 0);
                    }
                }
            }
        }
    }
}
#[test]
fn modal_deck_actual_complete_constructor_delta() {
    let e = engine(RulesBackend::Auto);
    let Ok(path) = std::env::var("SWU_MODAL_DECK_BASELINE_POOL") else {
        return;
    };
    let old: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let new = e.compiled_pool().unwrap();
    let codes = |p: &Value| {
        p["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["setCode"].as_str().unwrap().to_owned())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let (a, b) = (codes(&old), codes(&new));
    assert!(a.is_subset(&b));
    assert_eq!(
        b.difference(&a).map(String::as_str).collect::<Vec<_>>(),
        vec!["ASH_045", "ASH_133"]
    );
    for row in old["cards"].as_array().unwrap() {
        assert_eq!(
            Some(row),
            new["cards"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["setCode"] == row["setCode"])
        );
    }
    eprintln!("MODAL_DECK_CONSTRUCTOR {} -> {}", a.len(), b.len());
    if let Ok(out) = std::env::var("SWU_MODAL_DECK_REPORT_DIR") {
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            std::path::Path::new(&out).join("pool.json"),
            serde_json::to_vec_pretty(&new).unwrap(),
        )
        .unwrap();
        std::fs::write(
            std::path::Path::new(&out).join("rules.json"),
            serde_json::to_vec_pretty(&e.rules_backend_report()).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn modal_deck_normal_play_costs_and_empty_recovery_does_not_heal() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, cost) in [("ASH_045", 5), ("ASH_133", 8)] {
                let mut s = board(&e, seat);
                let source = add(&e, &mut s, seat, code);
                s.player_mut(seat).hand.push(source).unwrap();
                let base = s.player(seat).base;
                s.card_mut(base).damage = 7;
                let before = e.ready_resources(&s, seat);
                let a = e.card_action(&s, source);
                s = e.apply(&s, &a).unwrap();
                assert!(e.in_arena(&s, source));
                assert_eq!(e.ready_resources(&s, seat), before - cost);
                assert_eq!(s.card(base).damage, 7);
                assert_eq!(s.ability_program, 0);
                assert_eq!(s.decision, Decision::Window);
            }
        }
    }
}
#[test]
fn modal_deck_capture_drift_and_destination_capacity_are_atomic() {
    let e = engine(RulesBackend::Auto);
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_045");
    e.defeat(&mut s, source).unwrap();
    e.finish(&mut s).unwrap();
    click(&e, &mut s, "Your deck");
    let a = e.legal_actions(&s)[0].clone();
    let captured = s.ability_target;
    s.player_mut(Seat::P1).deck.remove(captured);
    s.player_mut(Seat::P1).discard.push(captured).unwrap();
    let before = s;
    assert!(e.apply(&s, &a).unwrap_err().contains("captured top"));
    assert_eq!(s, before);
    for hand in [false, true] {
        let mut s = board(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "ASH_133");
        let target = add(&e, &mut s, Seat::P1, "HMW_093");
        s.player_mut(Seat::P1).discard.push(target).unwrap();
        start(&e, &mut s, source);
        s = e.apply(&s, &e.card_action(&s, target)).unwrap();
        let a = e.legal_actions(&s)[usize::from(hand)].clone();
        while (if hand {
            s.players[0].hand.len
        } else {
            s.players[0].deck.len
        }) as usize
            != MAX_ZONE
        {
            let id = add(&e, &mut s, Seat::P1, "LOF_254");
            if hand {
                s.players[0].hand.push(id).unwrap();
            } else {
                s.players[0].deck.push(id).unwrap();
            }
        }
        let before = s;
        assert!(e.apply(&s, &a).is_err());
        assert_eq!(s, before);
        assert_eq!(s.zobrist(), before.zobrist());
    }
}
#[test]
fn modal_deck_exact_program_and_catalogue_alias_boundaries() {
    let mut c = catalogue();
    for k in [
        crate::modal_deck_movement_contract::Kind::Top,
        crate::modal_deck_movement_contract::Kind::Recovery,
    ] {
        let mut alias = crate::modal_deck_movement_contract::profile(k);
        let code = format!("SYNTH_MODAL_{k:?}");
        alias["setCode"] = json!(code);
        c["cards"].as_array_mut().unwrap().push(alias);
    }
    let e = Engine::from_catalogue_value(&c).unwrap();
    for code in ["SYNTH_MODAL_Top", "SYNTH_MODAL_Recovery"] {
        assert!(!e.by_code.contains_key(code));
    }
    for code in ["ASH_045", "ASH_133"] {
        let mut e = engine(RulesBackend::Auto);
        let mut s = board(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, code);
        let id = e.card_programs(&s, source)[0];
        e.catalogue.as_mut().unwrap().programs[id as usize].optional = true;
        s.source = source;
        let before = s;
        assert!(e
            .begin_program(&mut s, id, ProgramContinuation::Finish)
            .is_err());
        assert_eq!(s, before);
    }
    let top = crate::modal_deck_movement_contract::profile(
        crate::modal_deck_movement_contract::Kind::Top,
    );
    assert!(serde_json::from_value::<crate::ability_ir::Ability>(
        top["abilities"]["card"][0].clone()
    )
    .is_err());
    let recovery = crate::modal_deck_movement_contract::profile(
        crate::modal_deck_movement_contract::Kind::Recovery,
    );
    assert!(serde_json::from_value::<crate::ability_ir::Ability>(
        recovery["abilities"]["card"][0].clone()
    )
    .is_err());
}
#[test]
fn modal_deck_selected_pool_and_settled_full_hash_roundtrip() {
    let mut c = catalogue();
    c["cards"].as_array_mut().unwrap().retain(|v| {
        v["setCode"].as_str().is_some_and(|s| {
            s.starts_with("TOKEN_")
                || [
                    "SOR_023", "SOR_014", "LOF_254", "ASH_045", "ASH_133", "HMW_093",
                ]
                .contains(&s)
        })
    });
    let pool = Engine::from_catalogue_value(&c)
        .unwrap()
        .compiled_pool()
        .unwrap();
    let e = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    )
    .unwrap();
    for code in ["ASH_045", "ASH_133"] {
        let mut s = board(&e, Seat::P1);
        for seat in [Seat::P1, Seat::P2] {
            s.player_mut(seat).force_held = true;
        }
        s = e.load_state(&e.snapshot(&s)).unwrap();
        let initial = e.load_state(&e.snapshot(&s)).unwrap();
        assert_eq!(
            initial.zobrist(),
            s.zobrist(),
            "publicly imported fixture initial fixed point"
        );
        let source = unit(&e, &mut s, Seat::P1, code);
        if code == "ASH_045" {
            e.defeat(&mut s, source).unwrap();
            e.finish(&mut s).unwrap();
            click(&e, &mut s, "Your deck");
            let a = e.legal_actions(&s)[0].clone();
            s = e.apply(&s, &a).unwrap();
        } else {
            let target = add(&e, &mut s, Seat::P1, "HMW_093");
            s.player_mut(Seat::P1).discard.push(target).unwrap();
            start(&e, &mut s, source);
            s = e.apply(&s, &e.card_action(&s, target)).unwrap();
            let a = e.legal_actions(&s)[1].clone();
            s = e.apply(&s, &a).unwrap();
        }
        let wire = e.snapshot(&s);
        let reload = e.load_state(&wire).unwrap();
        assert_eq!(e.snapshot(&reload), wire);
        assert_eq!(reload.zobrist(), s.zobrist());
        let copy = s;
        assert_eq!(copy, s);
        assert_eq!(copy.zobrist(), s.zobrist());
    }
    let mut bad = c.clone();
    let at = bad["cards"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v["setCode"] == "ASH_045")
        .unwrap();
    bad["cards"][at]["privatePrograms"] = json!([]);
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&bad).unwrap()
    )
    .is_err());
}
#[test]
fn modal_deck_trask_real_native_attack_keeps_attack_context_through_recovery() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let source = unit(&e, &mut s, seat, "ASH_133");
            let target = add(&e, &mut s, seat, "HMW_093");
            s.player_mut(seat).discard.push(target).unwrap();
            let own = s.player(seat).base;
            let enemy = s.player(seat.other()).base;
            s.card_mut(own).damage = 5;
            s = e.apply(&s, &e.card_action(&s, source)).unwrap();
            assert_eq!(s.decision, Decision::AttackTarget);
            s = e.apply(&s, &e.card_action(&s, enemy)).unwrap();
            assert!(e.legal_actions(&s).contains(&e.card_action(&s, target)));
            s = e.apply(&s, &e.card_action(&s, target)).unwrap();
            let a = e.legal_actions(&s)[0].clone();
            s = e.apply(&s, &a).unwrap();
            assert_eq!(s.card(own).damage, 2);
            assert_eq!(s.card(enemy).damage, 5);
            assert!(s.player(seat).deck.contains(target));
            assert!(s.card(source).exhausted);
            assert_eq!(s.decision, Decision::Window);
        }
    }
}
#[test]
fn modal_deck_inline_layout_is_unchanged() {
    assert_eq!(std::mem::size_of::<State>(), 38312);
    eprintln!(
        "MODAL_DECK_LAYOUT State={} Context={} Frame={} Trigger={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<ResolutionContext>(),
        std::mem::size_of::<DeathFrame>(),
        std::mem::size_of::<DeathTrigger>()
    );
}

#[test]
fn modal_deck_night_trooper_both_empty_skips_menu_but_one_empty_keeps_both_choices() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for empty_other in [false, true] {
                let mut s = board(&e, seat);
                for side in [seat, seat.other()] {
                    if side == seat || empty_other {
                        for id in s.player(side).deck.as_slice().to_vec() {
                            s.player_mut(side).deck.remove(id);
                            s.player_mut(side).discard.push(id).unwrap();
                        }
                    }
                }
                let source = unit(&e, &mut s, seat, "ASH_045");
                e.defeat(&mut s, source).unwrap();
                e.finish(&mut s).unwrap();
                if empty_other {
                    assert_eq!(s.decision, Decision::Window);
                    assert_eq!(s.ability_program, 0);
                    assert!(!e
                        .legal_actions(&s)
                        .iter()
                        .any(|a| a.label == "Your deck" || a.label == "Opponent's deck"));
                } else {
                    assert_eq!(
                        e.legal_actions(&s)
                            .iter()
                            .map(|a| a.label.as_str())
                            .collect::<Vec<_>>(),
                        vec!["Your deck", "Opponent's deck"]
                    );
                    click(&e, &mut s, "Your deck");
                    assert_eq!(s.ability_program, 0);
                    assert_eq!(s.decision, Decision::Window);
                }
            }
        }
    }
}
