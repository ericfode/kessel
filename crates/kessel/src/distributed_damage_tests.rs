use super::*;
fn engine() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
        .unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: e.by_code[code],
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine) -> State {
    let mut s = State::default();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for seat in [Seat::P1, Seat::P2] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, "SOR_014");
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
    }
    s
}
fn distribution() -> Effect {
    serde_json::from_value(json!({"kind":"distributeDamage","amount":{"kind":"const","value":6},"canChooseNoTargets":false,"canDistributeLess":false,"targetResolver":{"owner":"opponent","type":"unit","zone":"anyAttackable"}})).unwrap()
}
#[test]
fn distributed_damage_captures_nonuniform_amounts_and_simultaneous_lethals() {
    let e = engine();
    let mut s = board(&e);
    let source = add(&e, &mut s, Seat::P1, "SOR_135");
    e.add_arena(&mut s, source).unwrap();
    s.source = source;
    let a = add(&e, &mut s, Seat::P2, "SHD_056");
    let b = add(&e, &mut s, Seat::P2, "SHD_056");
    e.add_arena(&mut s, a).unwrap();
    e.add_arena(&mut s, b).unwrap();
    let hp = e.hp(&s, a);
    s.card_mut(a).damage = hp - 1;
    e.capture_distribution(&mut s, &[(a, 4), (b, 2)]).unwrap();
    assert_eq!(s.event_group.records[0].amount, 4);
    assert!(e.resolve_group(&mut s, &distribution()).unwrap());
    assert!(!e.in_arena(&s, a));
    assert_eq!(s.card(b).damage, 2);
    assert_eq!(s.ability_result, 3);
    assert_eq!(s.event_result.damage_dealt, 3);
    assert!(s.ability_succeeded);
}
#[test]
fn distributed_damage_shields_cancel_children_but_meta_resolves() {
    let e = engine();
    for count in [1, 2] {
        let mut s = board(&e);
        let source = add(&e, &mut s, Seat::P1, "SOR_135");
        e.add_arena(&mut s, source).unwrap();
        s.source = source;
        let mut targets = vec![];
        for _ in 0..count {
            let id = add(&e, &mut s, Seat::P2, "SHD_056");
            e.add_arena(&mut s, id).unwrap();
            s.ability_target = id;
            e.apply_effect(
                &mut s,
                &Effect::GiveShield {
                    target: Target::name("chosen"),
                    amount: Expr::number(1),
                },
            )
            .unwrap();
            targets.push(id);
        }
        let pairs = targets
            .iter()
            .map(|id| (*id, 6 / count))
            .collect::<Vec<_>>();
        e.capture_distribution(&mut s, &pairs).unwrap();
        let effect = distribution();
        if !e.resolve_group(&mut s, &effect).unwrap() {
            assert_eq!(s.decision, Decision::AbilityGroupShield);
            for id in &targets {
                e.replace_group_damage(&mut s, &effect, *id);
            }
            s.event_group.shield_mask = 0;
            assert!(e.resolve_group(&mut s, &effect).unwrap());
        }
        for id in targets {
            assert_eq!(s.card(id).damage, 0);
            assert_eq!(s.card(id).shield, 0);
        }
        assert_eq!(s.ability_result, 0);
        assert_eq!(s.event_result.damage_dealt, 0);
        assert!(s.ability_succeeded);
    }
}
fn ready(e: &Engine, s: &mut State, code: &str, controller: Seat) -> CardId {
    let source = add(e, s, controller, code);
    e.add_arena(s, source).unwrap();
    s.source = source;
    s.active = controller;
    let p = e.card_programs(s, source)[0];
    s.ability_program = p + 1;
    s.ability_pc = 0;
    s.ability_committed = true;
    source
}
#[test]
fn distributed_damage_exact_prompt_and_invalid_injected_allocations() {
    let e = engine();
    for code in ["SOR_135", "LOF_213"] {
        let mut s = board(&e);
        ready(&e, &mut s, code, Seat::P1);
        let mut targets = vec![];
        for _ in 0..3 {
            let id = add(&e, &mut s, Seat::P2, "SHD_056");
            e.add_arena(&mut s, id).unwrap();
            targets.push(id);
        }
        e.run_program(&mut s).unwrap();
        assert_eq!(s.decision, Decision::AbilityDistribute);
        let actions = e.distribution_actions(&s).unwrap();
        assert_eq!(actions.len(), 28);
        for arg in [
            json!([]),
            json!([[format!("c{}", targets[0] - 1), 5]]),
            json!([
                [format!("c{}", targets[0] - 1), 0],
                [format!("c{}", targets[1] - 1), 6]
            ]),
            json!([
                [format!("c{}", targets[0] - 1), 3],
                [format!("c{}", targets[0] - 1), 3]
            ]),
            json!([[format!("c{}", s.source - 1), 6]]),
            json!([[format!("c{}", targets[0] - 1), 6.5]]),
        ] {
            let action = Action {
                arg: Some(arg),
                ..actions[0].clone()
            };
            let before = s;
            assert!(e.apply_ability_action(&s, &action).is_err());
            assert_eq!(s, before);
        }
    }
}
#[test]
fn distributed_damage_defeated_stolen_source_uses_captured_controller() {
    let e = engine();
    let mut s = board(&e);
    let source = ready(&e, &mut s, "LOF_213", Seat::P2);
    s.player_mut(Seat::P2).space.remove(source);
    s.player_mut(Seat::P2).ground.remove(source);
    s.player_mut(Seat::P1).discard.push(source).unwrap();
    s.card_mut(source).owner = Seat::P1;
    s.last_known_source = source;
    s.active = Seat::P2;
    let mut enemies = vec![];
    for _ in 0..2 {
        let id = add(&e, &mut s, Seat::P1, "SHD_056");
        e.add_arena(&mut s, id).unwrap();
        enemies.push(id);
    }
    let friend = add(&e, &mut s, Seat::P2, "SHD_056");
    e.add_arena(&mut s, friend).unwrap();
    e.run_program(&mut s).unwrap();
    assert_eq!(s.decision, Decision::AbilityDistribute);
    let actions = e.distribution_actions(&s).unwrap();
    assert_eq!(actions.len(), 7);
    assert!(actions.iter().all(|a| a
        .arg
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|p| enemies.contains(&cid(p[0].as_str().unwrap()).unwrap()))));
}
#[test]
fn distributed_damage_physical_capture_and_negative_child_order_are_distinct_from_canonical_keys() {
    let e = engine();
    let mut s = board(&e);
    ready(&e, &mut s, "SOR_135", Seat::P1);
    let low = add(&e, &mut s, Seat::P2, "SHD_056");
    while s.next_id < 11 {
        add(&e, &mut s, Seat::P1, "SHD_056");
    }
    let high = add(&e, &mut s, Seat::P2, "SHD_056");
    assert!(format!("c{}", high - 1) < format!("c{}", low - 1));
    e.add_arena(&mut s, low).unwrap();
    e.add_arena(&mut s, high).unwrap();
    for id in [low, high] {
        s.card_mut(id).damage = e.hp(&s, id) - 1;
    }
    let canonical = vec![(high, 3), (low, 3)];
    e.capture_distribution(&mut s, &canonical).unwrap();
    assert_eq!(
        s.event_group.records[..2]
            .iter()
            .map(|r| r.target)
            .collect::<Vec<_>>(),
        [low, high]
    );
    assert!(e.resolve_group(&mut s, &distribution()).unwrap());
    assert_eq!(s.players[1].discard.as_slice(), [high, low]);
    assert_eq!(s.ability_result, 2);
}
#[test]
fn distributed_damage_no_targets_clears_prior_result_and_one_target_autoassigns() {
    let e = engine();
    for count in [0, 1] {
        let mut s = board(&e);
        ready(&e, &mut s, "SOR_135", Seat::P1);
        s.ability_result = 17;
        s.ability_succeeded = true;
        s.event_result.damage_dealt = 17;
        let mut target = 0;
        if count == 1 {
            target = add(&e, &mut s, Seat::P2, "SHD_056");
            e.add_arena(&mut s, target).unwrap();
            s.card_mut(target).damage = e.hp(&s, target) - 1;
        }
        e.run_program(&mut s).unwrap();
        assert_ne!(s.decision, Decision::AbilityDistribute);
        assert_eq!(s.ability_program, 0);
        assert_eq!(
            s.ability_result, 0,
            "finish clears the completed action context"
        );
        assert!(!s.ability_succeeded);
        assert_eq!(s.event_result.damage_dealt, 0);
        if count == 1 {
            assert!(!e.in_arena(&s, target));
        }
    }
}
#[test]
fn distributed_damage_six_recipients_and_overflow_fail_explicitly() {
    let e = engine();
    for count in [6, 10] {
        let mut s = board(&e);
        ready(&e, &mut s, "SOR_135", Seat::P1);
        let mut targets = vec![];
        for _ in 0..count {
            let id = add(&e, &mut s, Seat::P2, "SHD_056");
            e.add_arena(&mut s, id).unwrap();
            targets.push(id);
        }
        let before = s;
        if count == 10 {
            assert!(e.run_program(&mut s).unwrap_err().contains("capacity"));
            assert_eq!(s, before);
        } else {
            e.run_program(&mut s).unwrap();
            let actions = e.distribution_actions(&s).unwrap();
            assert_eq!(actions.len(), 462);
            let all = actions
                .into_iter()
                .find(|a| a.arg.as_ref().unwrap().as_array().unwrap().len() == 6)
                .unwrap();
            s = e.apply_ability_action(&s, &all).unwrap();
            assert!(targets.into_iter().all(|id| s.card(id).damage == 1));
            assert_eq!(s.ability_program, 0);
        }
    }
}
#[test]
fn distributed_damage_foreign_controller_multiple_shields_restore_initiator_and_meta_result() {
    let e = engine();
    let mut s = board(&e);
    let source = ready(&e, &mut s, "LOF_213", Seat::P2);
    s.players[1].space.remove(source);
    s.card_mut(source).owner = Seat::P1;
    s.players[0].discard.push(source).unwrap();
    s.last_known_source = source;
    let mut targets = vec![];
    let mut heads = vec![];
    let mut tails = vec![];
    for _ in 0..2 {
        let target = add(&e, &mut s, Seat::P1, "SHD_056");
        e.add_arena(&mut s, target).unwrap();
        s.ability_target = target;
        e.apply_effect(
            &mut s,
            &Effect::GiveShield {
                target: Target::name("chosen"),
                amount: Expr::number(2),
            },
        )
        .unwrap();
        targets.push(target);
        heads.push(s.card(target).shield);
        tails.push(s.card(s.card(target).shield).next_token);
    }
    e.capture_distribution(&mut s, &[(targets[0], 2), (targets[1], 4)])
        .unwrap();
    let effect = distribution();
    assert!(!e.resolve_group(&mut s, &effect).unwrap());
    assert_eq!(s.active, Seat::P1);
    assert_eq!(s.event_group.initiator, Seat::P2);
    for target in &targets {
        e.replace_group_damage(&mut s, &effect, *target);
    }
    assert!(e.resolve_group(&mut s, &effect).unwrap());
    assert_eq!(s.active, Seat::P2);
    assert!(s.ability_succeeded);
    assert_eq!(s.ability_result, 0);
    assert_eq!(s.event_result.damage_dealt, 0);
    assert_eq!(s.card(targets[0]).shield, heads[0]);
    assert_eq!(s.card(targets[1]).shield, tails[1]);
    assert_eq!(s.card(heads[0]).next_token, 0);
    assert_eq!(s.card(tails[1]).next_token, 0);
}
#[test]
fn distributed_damage_synthetic_continuation_observes_step_success_and_result_after_shield_resume()
{
    // This synthetic continuation exposes the intermediate result before normal
    // finish() clears action context. It does not expand source admission.
    for mode in [0, 1, 2] {
        let mut e = engine();
        e.rules_backend = RulesBackend::Interpreter;
        let mut s = board(&e);
        let source = ready(&e, &mut s, "SOR_135", Seat::P1);
        s.card_mut(source).damage = 1;
        let ability=serde_json::from_value(json!({"trigger":"whenPlayed","title":"Synthetic distribution result diagnostic","effect":serde_json::to_value(distribution()).unwrap(),"then":{"title":"Pause to inspect result","targetResolver":{"owner":"self","type":"unit","zone":"arena"},"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":1}}}})).unwrap();
        let program = Program::compile(&ability).unwrap();
        let catalogue = e.catalogue.as_mut().unwrap();
        let index = catalogue.programs.len();
        catalogue.programs.push(program);
        s.ability_program = index as u16 + 1;
        let mut targets = vec![];
        if mode > 0 {
            for _ in 0..2 {
                let id = add(&e, &mut s, Seat::P2, "SHD_056");
                e.add_arena(&mut s, id).unwrap();
                if mode == 1 {
                    s.ability_target = id;
                    e.apply_effect(
                        &mut s,
                        &Effect::GiveShield {
                            target: Target::name("chosen"),
                            amount: Expr::number(1),
                        },
                    )
                    .unwrap();
                }
                targets.push(id);
            }
        }
        s.ability_result = 17;
        s.ability_succeeded = true;
        s.event_result.damage_dealt = 17;
        e.run_program(&mut s).unwrap();
        if mode > 0 {
            assert_eq!(s.decision, Decision::AbilityDistribute);
            let action = e
                .distribution_actions(&s)
                .unwrap()
                .into_iter()
                .find(|a| {
                    a.arg
                        == Some(json!([
                            [format!("c{}", targets[0] - 1), 2],
                            [format!("c{}", targets[1] - 1), 4]
                        ]))
                })
                .unwrap();
            s = e.apply_ability_action(&s, &action).unwrap();
            if mode == 1 {
                assert_eq!(s.decision, Decision::AbilityGroupShield);
                let all = Action::button("Resolve all (2)", "all");
                s = e.apply_ability_action(&s, &all).unwrap();
            }
        }
        assert_eq!(s.decision, Decision::AbilityTarget);
        assert_eq!(s.ability_succeeded, mode > 0);
        assert_eq!(s.ability_result, if mode == 2 { 5 } else { 0 });
        assert_eq!(s.event_result.damage_dealt, if mode == 2 { 5 } else { 0 });
    }
}
