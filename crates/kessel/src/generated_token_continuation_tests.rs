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
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 3;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.credit_state_exact = e.needs_credit_state();
    for p in [Seat::P1, Seat::P2] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        let force = add(e, &mut s, p, "TOKEN_THE_FORCE");
        s.player_mut(p).force_id = force;
    }
    let v = e.snapshot(&s);
    let s = e.load_state(&v).unwrap();
    assert_eq!(e.snapshot(&s), v);
    let reload = e.load_state(&e.snapshot(&s)).unwrap();
    assert_eq!(s.zobrist(), reload.zobrist());
    s
}
fn setup_event(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "ASH_257");
    s.player_mut(seat).hand.push(id).unwrap();
    for _ in 0..6 {
        let r = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    id
}
fn play(e: &Engine, s: &mut State, event: CardId) {
    let a = e.card_action(s, event);
    assert!(e.legal_actions(s).contains(&a));
    *s = e.apply(s, &a).unwrap();
}
fn choose(e: &Engine, s: &mut State, index: usize) {
    let actions = e.legal_actions(s);
    assert_eq!(actions.len(), 2, "{:?}", actions);
    assert!(actions.iter().all(|a| a.label != "Cancel"));
    *s = e.apply(s, &actions[index]).unwrap();
}
fn drain(e: &Engine, s: &mut State) {
    for _ in 0..100 {
        if matches!(s.decision, Decision::Window | Decision::Terminal) {
            return;
        }
        let a = e
            .legal_actions(s)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no action {:?}", s.decision));
        *s = e.apply(s, &a).unwrap();
    }
    panic!("not drained")
}
fn spent(s: &State, seat: Seat) -> usize {
    s.player(seat)
        .resources
        .as_slice()
        .iter()
        .filter(|id| s.card(**id).exhausted)
        .count()
}
fn mandos(e: &Engine, s: &State, seat: Seat) -> Vec<CardId> {
    s.player(seat)
        .ground
        .as_slice()
        .iter()
        .copied()
        .filter(|id| e.def(s, *id).info.internal_name == "mandalorian")
        .collect()
}
#[test]
fn generated_token_program_is_closed_and_reserves_stored_sentinel() {
    let p = crate::generated_token_continuation_contract::compiled().unwrap();
    assert!(crate::generated_token_continuation_contract::program(&p));
    assert_eq!(std::mem::size_of::<State>(), 38344);
    assert_eq!(std::mem::size_of::<ResolutionContext>(), 124);
    assert!(crate::generated_token_continuation_contract::checked_program_id(65534).is_ok());
    for i in [65535, 65536, usize::MAX] {
        assert!(crate::generated_token_continuation_contract::checked_program_id(i).is_err());
    }
}
#[test]
fn generated_token_paid_force_healthy_and_unavailable_branches() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        assert!(
            e.by_code.contains_key("ASH_257"),
            "{:?}",
            e.rejected_cards().unwrap().get("ASH_257")
        );
        for seat in [Seat::P1, Seat::P2] {
            for (damage, branch, expected) in [(9, 0, 4), (2, 0, 0), (0, 0, 0), (9, 1, 9)] {
                let mut s = board(&e, seat);
                unit(&e, &mut s, seat, "HMW_128");
                let b = s.player(seat).base;
                s.card_mut(b).damage = damage;
                let event = setup_event(&e, &mut s, seat);
                play(&e, &mut s, event);
                assert_eq!(spent(&s, seat), 0);
                assert!(s.player(seat).hand.contains(event));
                assert_eq!(s.decision, Decision::AbilityTarget);
                let v = e.snapshot(&s);
                assert_eq!(v["generatedTokenContinuationPending"], true);
                assert!(e.load_state(&v).is_err());
                choose(&e, &mut s, branch);
                drain(&e, &mut s);
                assert_eq!(s.card(b).damage, expected);
                assert_eq!(spent(&s, seat), 2);
                assert!(s.player(seat).discard.contains(event));
                assert!(mandos(&e, &s, seat).is_empty());
            }
        }
    }
}
#[test]
fn generated_token_paid_singleton_advantage_before_shield_and_snoke_death() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for snoke in [false, true] {
                let mut s = board(&e, seat);
                let donor = unit(&e, &mut s, seat, "SHD_169");
                if snoke {
                    unit(&e, &mut s, seat.other(), "SHD_037");
                }
                let event = setup_event(&e, &mut s, seat);
                let next = s.next_id;
                play(&e, &mut s, event);
                choose(&e, &mut s, 1);
                drain(&e, &mut s);
                assert_eq!(spent(&s, seat), 2);
                assert!(s.player(seat).discard.contains(event));
                assert!(e
                    .upgrades(&s, donor)
                    .iter()
                    .all(|id| e.def(&s, *id).info.internal_name != "advantage"));
                if snoke {
                    assert_eq!(s.next_id, next + 1);
                    assert!(mandos(&e, &s, seat).is_empty());
                    assert!(!e.in_arena(&s, next));
                    let v = e.snapshot(&s);
                    assert_eq!(v["inactivePhaseKeywordStateVersion"], 1);
                    assert!(v["inactiveAllocatedCards"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|v| v["id"] == format!("c{}", next - 1)));
                } else {
                    let ids = mandos(&e, &s, seat);
                    assert_eq!(ids, vec![next]);
                    assert!(s.card(next).exhausted);
                    assert_eq!(e.power(&s, next, false), 3);
                    assert_eq!(e.hp(&s, next), 2);
                    let advantage = e
                        .upgrades(&s, next)
                        .into_iter()
                        .find(|id| e.def(&s, *id).info.internal_name == "advantage")
                        .unwrap();
                    let shield = s.card(next).shield;
                    assert_eq!(advantage, next + 1);
                    assert_eq!(shield, next + 2);
                    assert_eq!(s.next_id, next + 3);
                    assert_eq!(s.card(advantage).owner, seat);
                    assert_eq!(s.card(shield).owner, seat);
                }
            }
        }
    }
}
#[test]
fn generated_token_no_trait_paid_anyway_and_cancel_are_distinct() {
    for seat in [Seat::P1, Seat::P2] {
        let e = engine();
        let mut s = board(&e, seat);
        let event = setup_event(&e, &mut s, seat);
        let original = s;
        play(&e, &mut s, event);
        assert_eq!(s.decision, Decision::AbilityNoEffect);
        let actions = e.legal_actions(&s);
        assert!(actions.iter().any(|a| a.label == "Cancel"));
        let cancel = actions.iter().find(|a| a.label == "Cancel").unwrap();
        let restored = e.apply(&s, cancel).unwrap();
        assert_eq!(restored, original);
        let play = actions.iter().find(|a| a.label != "Cancel").unwrap();
        s = e.apply(&s, play).unwrap();
        drain(&e, &mut s);
        assert_eq!(spent(&s, seat), 2);
        assert!(s.player(seat).discard.contains(event));
    }
}
#[test]
fn generated_token_source_factory_raw_and_dependency_closure() {
    let c = catalogue();
    let raw = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["setCode"] == "ASH_257")
        .unwrap();
    assert!(crate::generated_token_continuation_contract::prepare(raw, &c).unwrap());
    for(path,value)in [("/setCode",json!("OPAQUE_ALIAS")),("/sourceHash",json!("changed")),("/card/types",json!(["unit"])),("/card/keywords",json!(["support"])),("/abilities/card/0/targetResolver/showUnresolvable",json!(false)),("/abilities/card/0/targetResolver/choices/factory",json!(true)),("/abilities/card/0/targetResolver/choices/entries/1/effect/onTrue/effects/1/properties/target/eventIndex",json!(1))]{let mut bad=raw.clone();if let Some(slot)=bad.pointer_mut(path){*slot=value;}else{panic!("missing mutation {path}")};assert!(crate::generated_token_continuation_contract::prepare(&bad,&c).is_err(),"{path}");}
    for code in ["TOKEN_MANDALORIAN", "TOKEN_ADVANTAGE", "TOKEN_SHIELD"] {
        let mut bad = c.clone();
        bad["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v["setCode"] != code);
        assert!(crate::generated_token_continuation_contract::prepare(raw, &bad).is_err());
    }
    let e = engine();
    assert!(!e.by_code.contains_key("ASH_094"));
    assert!(!e.by_code.contains_key("TWI_203"));
}
#[test]
fn generated_token_scoped_result_copy_hash_and_no_fallback() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let event = setup_event(&e, &mut s, Seat::P1);
    let old = unit(&e, &mut s, Seat::P1, "TOKEN_MANDALORIAN");
    let p = e.card_programs(&s, event)[0];
    s.source = event;
    s.ability_program = p + 1;
    s.ability_pc = 8;
    s.event_result = StepEventResult {
        generated_token: old,
        ..StepEventResult::for_state(&s)
    };
    s.event_result.pc = 7;
    let context = ResolutionContext::capture(&s);
    let hash = s.zobrist();
    s.event_result.generated_token = 0;
    assert_ne!(hash, s.zobrist());
    context.restore(&mut s);
    assert_eq!(hash, s.zobrist());
    assert!(s.event_result.belongs_to(event, p + 1, 7));
    s.event_result.pc = 6;
    let original = s;
    assert!(e.run_created_token_advantage(&mut s).is_err());
    assert_eq!(s, original);
}
#[test]
fn generated_token_capacity_preflight_is_whole_allocation_atomic() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for snoke in [false, true] {
            for free in [1, 2, 3] {
                let mut s = board(&e, seat);
                unit(&e, &mut s, seat, "SHD_169");
                if snoke {
                    unit(&e, &mut s, seat.other(), "SHD_037");
                }
                let event = setup_event(&e, &mut s, seat);
                while (s.next_id as usize) < MAX_CARDS - free {
                    add(&e, &mut s, seat, "LOF_254");
                }
                play(&e, &mut s, event);
                let action = e.legal_actions(&s)[1].clone();
                let before = s;
                let hash = s.zobrist();
                match e.apply(&s, &action) {
                    Ok(mut t) => {
                        assert!(snoke || free == 3);
                        drain(&e, &mut t);
                        assert_eq!(spent(&t, seat), 2);
                        assert_eq!(
                            t.next_id as usize,
                            MAX_CARDS - free + if snoke { 1 } else { 3 }
                        );
                    }
                    Err(reason) => {
                        assert!(!snoke && free < 3, "{reason}");
                        assert_eq!(s, before);
                        assert_eq!(s.zobrist(), hash);
                        assert_eq!(spent(&s, seat), 0);
                        assert!(s.player(seat).hand.contains(event));
                    }
                }
            }
        }
    }
}
#[test]
fn generated_token_selected_pool_requires_full_native_dependencies() {
    let c = catalogue();
    let e = engine();
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"].as_array_mut().unwrap().retain(|v| {
        [
            "ASH_257",
            "TOKEN_MANDALORIAN",
            "TOKEN_ADVANTAGE",
            "TOKEN_SHIELD",
        ]
        .iter()
        .any(|code| v["setCode"] == *code)
    });
    let bytes = serde_json::to_vec(&c).unwrap();
    assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes).is_ok());
    for code in ["TOKEN_MANDALORIAN", "TOKEN_ADVANTAGE", "TOKEN_SHIELD"] {
        let mut bad = pool.clone();
        bad["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v["setCode"] != code);
        assert!(
            Engine::from_compiled_pool_bytes(&serde_json::to_vec(&bad).unwrap(), &bytes).is_err(),
            "{code}"
        );
    }
}
#[test]
fn generated_token_two_copies_regroup_attack_expiry_and_wire_fixedpoints() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            unit(&e, &mut s, seat, "SHD_169");
            let first = setup_event(&e, &mut s, seat);
            play(&e, &mut s, first);
            choose(&e, &mut s, 1);
            drain(&e, &mut s);
            let pass = e
                .legal_actions(&s)
                .into_iter()
                .find(|a| a.label == "Pass")
                .unwrap();
            s = e.apply(&s, &pass).unwrap();
            let second = setup_event(&e, &mut s, seat);
            play(&e, &mut s, second);
            choose(&e, &mut s, 1);
            drain(&e, &mut s);
            let ids = mandos(&e, &s, seat);
            assert_eq!(ids.len(), 2);
            assert_ne!(first, second);
            let advantages = ids
                .iter()
                .map(|id| {
                    e.upgrades(&s, *id)
                        .into_iter()
                        .find(|u| e.def(&s, *u).info.internal_name == "advantage")
                        .unwrap()
                })
                .collect::<Vec<_>>();
            assert_ne!(advantages[0], advantages[1]);
            s.players[0].out = true;
            s.players[1].out = true;
            e.advance(&mut s).unwrap();
            drain(&e, &mut s);
            assert!(ids.iter().all(|id| !s.card(*id).exhausted
                && e.power(&s, *id, false) == 3
                && s.card(*id).shield != 0));
            s.active = seat;
            s.action_player = Some(seat);
            let v = e.snapshot(&s);
            let loaded = e.load_state(&v).unwrap();
            assert_eq!(e.snapshot(&loaded), v);
            assert_eq!(loaded.zobrist(), s.zobrist());
            let a = e.card_action(&s, ids[0]);
            s = e.apply(&s, &a).unwrap();
            if s.decision == Decision::AbilityMenu {
                let attack = e
                    .legal_actions(&s)
                    .into_iter()
                    .find(|a| a.label == "Attack")
                    .unwrap();
                s = e.apply(&s, &attack).unwrap();
            }
            assert_eq!(s.decision, Decision::AttackTarget);
            let base = s.player(seat.other()).base;
            let before_damage = s.card(base).damage;
            let a = e.card_action(&s, base);
            s = e.apply(&s, &a).unwrap();
            drain(&e, &mut s);
            assert_eq!(s.card(base).damage, before_damage + 3);
            assert_eq!(e.power(&s, ids[0], false), 2);
            assert_eq!(e.power(&s, ids[1], false), 3);
            assert_eq!(s.card(advantages[0]).parent, 0);
            assert_eq!(s.card(advantages[1]).parent, ids[1]);
            assert!(ids.iter().all(|id| s.card(*id).shield != 0));
            let v = e.snapshot(&s);
            let loaded = e.load_state(&v).unwrap();
            assert_eq!(e.snapshot(&loaded), v);
            assert_eq!(loaded.zobrist(), s.zobrist());
        }
    }
}
#[test]
fn generated_token_enemy_traits_do_not_enable_a_branch_and_pending_claims_are_strict() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        unit(&e, &mut s, seat.other(), "HMW_128");
        unit(&e, &mut s, seat.other(), "SHD_169");
        let event = setup_event(&e, &mut s, seat);
        play(&e, &mut s, event);
        assert_eq!(s.decision, Decision::AbilityNoEffect);
        let v = e.snapshot(&s);
        for value in [json!(true), json!(false), Value::Null] {
            let mut bad = v.clone();
            bad["generatedTokenContinuationPending"] = value;
            assert!(e.load_state(&bad).is_err());
        }
    }
    let mut p = crate::generated_token_continuation_contract::compiled().unwrap();
    p.instructions
        .retain(|i| !matches!(i, Instruction::GrantCreatedTokenAdvantage { .. }));
    assert!(crate::generated_token_continuation_contract::requires(&p));
    assert!(!crate::generated_token_continuation_contract::program(&p));
}
#[test]
fn generated_token_pending_saved_frame_support_and_result_roots_are_not_importable() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = setup_event(&e, &mut s, Seat::P1);
    let p = e.card_programs(&s, source)[0] + 1;
    for root in 0..5 {
        let mut t = s;
        let mut context = ResolutionContext::default();
        context.source = source;
        context.ability_program = p;
        context.ability_pc = 8;
        match root {
            0 => {
                t.support.context = context;
            }
            1 => {
                t.death_depth = 1;
                t.death_frames[0].context = context;
            }
            2 => {
                t.event_result.program = p;
                t.event_result.source = source;
                t.event_result.pc = 7;
            }
            3 => {
                t.support.context.event_result.program = p;
            }
            _ => {
                t.death_depth = 1;
                t.death_frames[0].context.event_result.program = p;
            }
        }
        assert!(e.generated_token_continuation_pending(&t));
        let v = e.snapshot(&t);
        assert_eq!(v["generatedTokenContinuationPending"], true);
        assert!(e.load_state(&v).is_err());
    }
}
