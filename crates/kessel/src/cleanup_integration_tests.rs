// Verbatim append-only tests from reviewed task-3 e5c77f5, isolated from card fixtures.
use super::*;

#[test]
fn group_shield_scheduler_preserves_both_layouts_and_controller_resumption() {
    let e = engine();
    let effect: Effect = serde_json::from_value(json!({
        "kind":"damage", "amount":{"kind":"const","value":1},
        "target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},
                  "predicate":{"kind":"const","value":true}}
    })).unwrap();
    for uniform in [false, true] {
        for initiator in [Seat::P1, Seat::P2] {
            for first in [Seat::P1, Seat::P2] {
                let mut s = board(&e);
                s.active = initiator;
                let source = unit(&e, &mut s, initiator, "LOF_256");
                s.source = source;
                let mut recipients = vec![];
                for seat in [Seat::P1, Seat::P2] {
                    for _ in 0..2 {
                        let target = unit(&e, &mut s, seat, "LOF_256");
                        let mut shields = vec![];
                        for _ in 0..2 {
                            shields.push(s.next_id);
                            e.create_token(&mut s, target, false).unwrap();
                        }
                        recipients.push((target, shields));
                    }
                }
                s.event_group.active = true;
                if uniform {
                    e.generate_uniform_damage(&mut s, &effect).unwrap();
                } else {
                    for (i, target) in std::iter::once(source)
                        .chain(recipients.iter().map(|(id, _)| *id)).enumerate()
                    {
                        s.event_group.records[i] = PreparedEvent {
                            target, amount: 1, ..PreparedEvent::default()
                        };
                        s.event_group.count += 1;
                    }
                }
                assert!(!e.resolve_group(&mut s, &effect).unwrap());
                assert_eq!(s.active, initiator);
                assert_eq!(s.event_group.shield_count, 0);
                assert_eq!(e.legal_actions(&s), vec![Action::button("You", 0), Action::button("Opponent", 1)]);
                let paused = s;
                assert!(!e.resolve_group(&mut s, &effect).unwrap());
                assert_eq!(s, paused, "resuming without a choice must not consume events");
                s.event_group.shield_priority = first.index() as u8 + 1;
                assert!(!e.resolve_group(&mut s, &effect).unwrap());
                assert_eq!(s.active, first);
                assert_eq!(s.event_group.shield_count, 2);
                assert_eq!(e.legal_actions(&s), vec![Action::button("Resolve next", "next"), Action::button("Resolve all (2)", "all")]);
                let target = e.group_shield_targets(&s, &effect, first)[0];
                e.replace_group_damage(&mut s, &effect, target);
                assert!(!e.resolve_group(&mut s, &effect).unwrap());
                assert_eq!(s.active, first.other());
                assert_eq!(s.event_group.shield_count, 2);
                for target in e.group_shield_targets(&s, &effect, first.other()) {
                    e.replace_group_damage(&mut s, &effect, target);
                }
                s.event_group.shield_priority = first.other().index() as u8 + 1;
                assert!(e.resolve_group(&mut s, &effect).unwrap());
                assert_eq!(s.active, initiator);
                assert_eq!(s.source, source);
                assert_eq!(s.card(source).damage, 1);
                assert_eq!(s.ability_result, 1);
                for (target, shields) in recipients {
                    assert_eq!(s.card(target).damage, 0);
                    assert_eq!(e.token_count(&s, s.card(target).shield), 1);
                    assert_eq!(shields.iter().filter(|id| s.card(**id).parent == target).count(), 1);
                    assert!(shields.contains(&s.card(target).shield));
                }
            }
        }
    }
}

#[test]
fn shared_heal_and_exhaustion_primitives_match_generated_steps_at_boundaries() {
    let mut e = engine();
    let programs = e.catalogue.as_ref().unwrap().programs.clone();
    let mut exercised = [0usize; 3];
    for (program_id, p) in programs.iter().enumerate() {
        let Ok(executable) = p.executable else { continue; };
        for (pc, instruction) in p.instructions.iter().enumerate() {
            let Instruction::Apply(effect) = instruction else { continue; };
            let kind = match effect {
                Effect::Heal { .. } => 0,
                Effect::Ready { .. } => 1,
                Effect::Exhaust { .. } => 2,
                _ => continue,
            };
            let encoded = serde_json::to_string(effect).unwrap();
            let needs_parent = encoded.contains("parentCard") || encoded.contains("parentUnit");
            for damage in [0, 1, 3] {
                for exhausted in [false, true] {
                    for target_zone in ["arena", "base", "discard"] {
                        let mut s = board(&e);
                        let host = unit(&e, &mut s, Seat::P1, "LOF_256");
                        let source = if needs_parent {
                            // Parent-target steps now share generated dispatch. Supply
                            // their required live attachment rather than a bare unit.
                            let code = e.definitions.iter().find(|d| d.info.is("upgrade") && !d.info.is("token")).unwrap().code.clone();
                            let id = add(&e, &mut s, Seat::P1, &code);
                            e.attach(&mut s, id, host);
                            s.card_mut(host).damage = damage;
                            s.card_mut(host).exhausted = exhausted;
                            id
                        } else { host };
                        let target = if target_zone == "base" {
                            s.players[1].base
                        } else {
                            let id = add(&e, &mut s, Seat::P2, "LOF_256");
                            if target_zone == "arena" { e.add_arena(&mut s, id).unwrap(); }
                            else { s.players[1].discard.push(id).unwrap(); }
                            id
                        };
                        s.source = source;
                        s.ability_target = target;
                        s.card_mut(target).damage = damage;
                        s.card_mut(target).exhausted = exhausted;
                        s.ability_program = program_id as u16 + 1;
                        s.ability_pc = pc as u8;
                        s.ability_committed = true;
                        e.set_rules_backend(RulesBackend::Interpreter).unwrap();
                        let mut expected = s;
                        let (result, succeeded) = e.apply_effect(&mut expected, effect).unwrap();
                        expected.ability_result = result;
                        expected.ability_succeeded = succeeded;
                        expected.ability_pc += 1;
                        e.set_rules_backend(RulesBackend::Auto).unwrap();
                        let mut actual = s;
                        assert!(!e.generated_step(&mut actual, p, executable).unwrap());
                        assert_eq!(actual, expected, "program {program_id}, pc {pc}, {target_zone}, damage {damage}, exhausted {exhausted}");
                        assert_eq!(e.legal_actions(&actual), e.legal_actions(&expected));
                        exercised[kind] += 1;
                    }
                }
            }
        }
    }
    assert!(exercised.iter().all(|count| *count > 0), "all shared primitive kinds need generated coverage: {exercised:?}");
}

#[test]
fn shared_primitives_preserve_card_domains_and_noop_results() {
    let e = engine();
    for zone in ["arena", "base", "discard"] {
        let mut s = board(&e);
        s.source = unit(&e, &mut s, Seat::P1, "LOF_256");
        let target = if zone == "base" { s.players[1].base } else {
            let id = add(&e, &mut s, Seat::P2, "LOF_256");
            if zone == "arena" { e.add_arena(&mut s, id).unwrap(); }
            else { s.players[1].discard.push(id).unwrap(); }
            id
        };
        s.ability_target = target;
        s.card_mut(target).damage = 3;
        s.card_mut(target).exhausted = true;
        let chosen = Target::Name("chosen".into());
        let partial = Effect::Heal { target: chosen.clone(), amount: Expr::number(2) };
        assert_eq!(e.apply_effect(&mut s, &partial).unwrap(), if zone == "discard" { (0, false) } else { (2, true) });
        assert_eq!(s.card(target).damage, if zone == "discard" { 3 } else { 1 });
        let excess = Effect::Heal { target: chosen.clone(), amount: Expr::number(9) };
        assert_eq!(e.apply_effect(&mut s, &excess).unwrap(), if zone == "discard" { (0, false) } else { (1, true) });
        assert_eq!(e.apply_effect(&mut s, &excess).unwrap(), (0, false));
        let ready = Effect::Ready { target: chosen.clone() };
        assert_eq!(e.apply_effect(&mut s, &ready).unwrap(), if zone == "arena" { (1, true) } else { (0, false) });
        assert_eq!(s.card(target).exhausted, zone != "arena");
        assert_eq!(e.apply_effect(&mut s, &ready).unwrap(), (0, false));
        let exhaust = Effect::Exhaust { target: chosen };
        assert_eq!(e.apply_effect(&mut s, &exhaust).unwrap(), if zone == "arena" { (1, true) } else { (0, false) });
        assert!(s.card(target).exhausted);
        assert_eq!(e.apply_effect(&mut s, &exhaust).unwrap(), (0, false));
    }
}

#[test]
fn token_requirements_keep_nested_creation_order_and_native_state_distinct() {
    let e = engine();
    let mut program = e.catalogue.as_ref().unwrap().programs[0].clone();
    program.condition = None;
    program.context_title.clear();
    program.costs.clear();
    let effect: Effect = serde_json::from_value(json!({"kind":"sequence","effects":[
        {"kind":"giveShield","target":"chosen"},
        {"kind":"conditional","condition":{"kind":"const","value":true},
         "onTrue":{"kind":"giveExperience","target":"chosen"},
         "onFalse":{"kind":"select","targetResolver":{},"effect":{
             "kind":"simultaneous","effects":[
                 {"kind":"giveShield","target":"chosen"},
                 {"kind":"createTokenUnit","tokenType":"xwing","target":"ownPlayer","amount":{"kind":"const","value":1}},
                 {"kind":"giveWeakness","target":"chosen","amount":{"kind":"const","value":1}},
                 {"kind":"useForce","target":"ownPlayer"}
             ]}}},
        {"kind":"phaseModifiers","effects":[
            {"kind":"createTokenUnit","tokenType":"spy","target":"ownPlayer","amount":{"kind":"const","value":1}},
            {"kind":"createTokenUnit","tokenType":"beast","target":"ownPlayer","amount":{"kind":"const","value":1}},
            {"kind":"createTokenUnit","tokenType":"unknown","target":"ownPlayer","amount":{"kind":"const","value":1}}
        ]}
    ]})).unwrap();
    // Discovery inspects typed trees; this deliberately broad tree is not an admission claim.
    for instruction in [Instruction::Apply(effect.clone()), Instruction::Group(effect.clone()),
        Instruction::NamedGroup(effect.clone()), Instruction::NamedChooserGroup {
            effect: effect.clone(), choosers: vec![crate::ability_ir::NamedChooser::Opponent],
        }]
    {
        program.instructions = vec![instruction];
        let requirements = TokenRequirements::for_program(&program);
        assert_eq!(requirements.created, ["TOKEN_SHIELD", "TOKEN_EXPERIENCE", "TOKEN_SHIELD",
            "TOKEN_XWING", "TOKEN_SPY", "TOKEN_BEAST", "unsupported token unit unknown"]);
        for token in ["TOKEN_SHIELD", "TOKEN_EXPERIENCE", "TOKEN_WEAKNESS", "TOKEN_THE_FORCE", "TOKEN_SPY", "TOKEN_BEAST"] {
            assert!(requirements.needs(token), "{token}");
        }
    }
    program.instructions = vec![Instruction::Apply(Effect::NoAction)];
    for position in ["condition", "contextTitle", "cost", "selector"] {
        program.condition = None;
        program.context_title.clear();
        program.costs.clear();
        program.instructions.truncate(1);
        let presence = Expr::Ref { path: vec!["player".into(), "hasTheForce".into()] };
        match position {
            "condition" => program.condition = Some(presence),
            "contextTitle" => program.context_title.push(presence),
            "cost" => program.costs.push(Cost::Resources { amount: presence }),
            _ => program.instructions.push(Instruction::Select(serde_json::from_value(json!({
                "predicate":{"kind":"query","name":"hasTheForce","receiver":{"kind":"ref","path":["player"]},"args":[]}
            })).unwrap())),
        }
        let requirements = TokenRequirements::for_program(&program);
        assert!(requirements.force, "presence in {position}");
        assert!(!requirements.weakness);
        assert!(requirements.created.is_empty(), "presence must not become token creation");
    }
}

#[test]
fn token_requirements_preserve_selected_pool_diagnostics() {
    let e = engine();
    let full = e.compiled_pool().unwrap();
    for (token, expected) in [
        ("TOKEN_SHIELD", "pool must include TOKEN_SHIELD"),
        ("TOKEN_WEAKNESS", "pool containing GiveWeakness cards must include reviewed TOKEN_WEAKNESS"),
        ("TOKEN_THE_FORCE", "pool containing Force gain/use/presence cards must include reviewed TOKEN_THE_FORCE"),
        ("TOKEN_SPY", "pool containing Spy creation must include reviewed TOKEN_SPY"),
        ("TOKEN_BEAST", "pool containing Beast creation must include reviewed TOKEN_BEAST"),
        ("TOKEN_EXPERIENCE", "pool containing Experience creation must include reviewed TOKEN_EXPERIENCE"),
    ] {
        let mut pool = full.clone();
        pool["cards"].as_array_mut().unwrap().retain(|card| card["setCode"] != token);
        let error = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),
            include_bytes!("../../../compiler/generated/catalog.json")).err().unwrap();
        assert_eq!(error, expected);
    }
    let mut pool = full;
    pool["cards"].as_array_mut().unwrap().retain(|card| !["TOKEN_WEAKNESS", "TOKEN_THE_FORCE", "TOKEN_EXPERIENCE"].contains(&card["setCode"].as_str().unwrap()));
    let error = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),
        include_bytes!("../../../compiler/generated/catalog.json")).err().unwrap();
    assert_eq!(error, "pool containing GiveWeakness cards must include reviewed TOKEN_WEAKNESS");
}

#[test]
fn action_context_reset_preserves_cancellation_completion_and_terminal_boundaries() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        let mut baseline = board(&e);
        baseline.active = actor;
        baseline.action_player = Some(actor);
        baseline.prev_passed = true;
        baseline.after_attack_triggers = 7;
        baseline.attack_restore = 3;
        let source = unit(&e, &mut baseline, actor, "LOF_256");
        let target = baseline.player(actor.other()).base;
        // Leave dormant death-frame provenance intact through cancellation/completion.
        baseline.death_frames[0].context.source = source;
        let mut pending = baseline;
        pending.source = source;
        pending.target = target;
        pending.attack_source = source;
        pending.attack_defender = target;
        pending.ambush = true;
        pending.attack_shield_opponent_first = true;
        pending.pending_triggers = 1;
        pending.trigger_context = 2;
        pending.pending_programs = 1;
        pending.after_attack_programs = 1;
        pending.ability_program = 1;
        pending.ability_pc = 1;
        pending.ability_target = target;
        pending.ability_selection = Selection::one(target);
        pending.event_group.active = true;
        pending.ability_result = 3;
        pending.ability_succeeded = true;
        pending.ability_return = 1;
        pending.ability_committed = true;
        pending.decision = Decision::AbilityOptional;
        let mut cancelled = pending;
        e.cancel(&mut cancelled);
        assert_eq!(cancelled, baseline, "cancel preserves actor, pass state, after-attack keywords and death provenance");
        let mut completed = pending;
        e.finish(&mut completed).unwrap();
        let mut expected = baseline;
        expected.prev_passed = false;
        expected.after_attack_triggers = 0;
        expected.active = actor.other();
        expected.action_player = Some(actor.other());
        assert_eq!(completed, expected, "completion clears context then advances priority");
        let mut terminal = pending;
        terminal.outcome = 1;
        let mut expected_terminal = terminal;
        expected_terminal.prev_passed = false;
        expected_terminal.active = actor.other();
        expected_terminal.action_player = Some(actor.other());
        expected_terminal.decision = Decision::Terminal;
        e.finish(&mut terminal).unwrap();
        assert_eq!(terminal, expected_terminal, "terminal completion returns before clearing retained resolution context");
    }
}
