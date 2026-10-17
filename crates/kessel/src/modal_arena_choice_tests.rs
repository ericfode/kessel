//! Source-derived tests for complete Lightmaker and Persecutor modal sources.
use super::*;

fn arena_engine(backend: RulesBackend) -> Engine {
    let mut e = engine();
    e.set_rules_backend(backend).unwrap();
    e
}
fn arena_board(e: &Engine, actor: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = actor;
    s.action_player = Some(actor);
    s
}
fn choice(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)));
    e.apply(s, &action).unwrap()
}
fn menu_labels(e: &Engine, s: &State) -> Vec<String> {
    e.legal_actions(s).into_iter().map(|a| a.label).collect()
}
fn program(e: &Engine, s: &State, source: CardId, trigger: &str) -> u16 {
    *e.def(s, source)
        .programs
        .card
        .iter()
        .find(|id| e.program(**id).trigger == trigger)
        .unwrap()
}
fn start(e: &Engine, s: &mut State, source: CardId, trigger: &str) {
    s.source = source;
    s.active = s.card(source).owner;
    let id = program(e, s, source, trigger);
    e.begin_program(s, id, 0).unwrap();
}
fn entry(code: &str) -> Value {
    let catalogue: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn admitted(raw: Value) -> bool {
    let code = raw["setCode"].as_str().unwrap().to_owned();
    crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[raw]}))
        .unwrap()
        .cards
        .contains_key(&code)
}
fn finish_shields(e: &Engine, mut s: State) -> State {
    for _ in 0..12 {
        if s.decision != Decision::AbilityGroupShield {
            return s;
        }
        let actions = e.legal_actions(&s);
        let action = actions
            .iter()
            .find(|a| a.label.starts_with("Resolve all"))
            .or_else(|| actions.iter().find(|a| a.label == "You"))
            .or_else(|| actions.first())
            .unwrap();
        s = e.apply(&s, action).unwrap();
    }
    panic!("Shield replacements did not complete")
}

#[test]
fn modal_arena_sources_admit_both_complete_program_topologies() {
    let e = arena_engine(RulesBackend::Auto);
    for (code, triggers) in [
        ("SEC_207", vec!["whenDefeated"]),
        ("LAW_178", vec!["whenPlayed", "onAttack"]),
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        let d = &e.definitions[e.by_code[code] as usize];
        assert_eq!(
            d.programs
                .card
                .iter()
                .map(|id| e.program(*id).trigger.as_str())
                .collect::<Vec<_>>(),
            triggers
        );
        for id in &d.programs.card {
            assert!(matches!(
                e.program(*id).instructions.first(),
                Some(Instruction::ModalChoice(_))
            ));
            assert_eq!(
                crate::modal_arena_choices::requires_global_order(e.program(*id)),
                code == "LAW_178"
            );
        }
    }
    assert_eq!(entry("SEC_207")["keywordValues"], json!({"raid":4}));
    assert_eq!(entry("LAW_178")["keywordValues"], json!({}));
    assert!(
        !e.by_code.contains_key("LAW_101"),
        "handler/stat composition remains unsupported"
    );
}

#[test]
fn modal_arena_lightmaker_actual_death_uses_departed_source_controller_and_selected_arena() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for selected in ["Space", "Ground"] {
                let mut s = arena_board(&e, owner.other());
                let source = unit(&e, &mut s, owner, "SEC_207");
                assert_eq!(
                    e.power(&s, source, true),
                    7,
                    "printed Raid 4 remains active while attacking"
                );
                let ground = unit(&e, &mut s, owner.other(), "LOF_256");
                let space = unit(&e, &mut s, owner.other(), "LOF_034");
                let friendly = unit(&e, &mut s, owner, "LOF_256");
                e.defeat(&mut s, source).unwrap();
                e.finish(&mut s).unwrap();
                assert!(s.player(owner).discard.contains(source));
                assert_eq!(s.active, owner);
                assert_eq!(menu_labels(&e, &s), ["Space", "Ground"]);
                for (index, action) in e.legal_actions(&s).iter().enumerate() {
                    assert_eq!(action.arg, Some(json!(index)));
                }
                let done = choice(&e, &s, selected);
                assert_eq!(done.card(ground).exhausted, selected == "Ground");
                assert_eq!(done.card(space).exhausted, selected == "Space");
                assert!(!done.card(friendly).exhausted);
                assert_eq!(done.ability_program, 0);
            }
        }
    }
}

#[test]
fn modal_arena_lightmaker_empty_sole_and_entirely_ineffective_modes_preserve_protocol() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for count in 0..=2 {
                let mut s = arena_board(&e, owner);
                let source = unit(&e, &mut s, owner, "SEC_207");
                let ground = (count > 0).then(|| unit(&e, &mut s, owner.other(), "LOF_256"));
                if let Some(id) = ground {
                    s.card_mut(id).exhausted = true;
                }
                if count == 2 {
                    let space = unit(&e, &mut s, owner.other(), "LOF_034");
                    s.card_mut(space).exhausted = true;
                }
                e.defeat(&mut s, source).unwrap();
                e.finish(&mut s).unwrap();
                if count == 2 {
                    assert_eq!(menu_labels(&e, &s), ["Space", "Ground"]);
                    s = choice(&e, &s, "Ground");
                    assert!(!s.ability_succeeded);
                    assert_eq!(s.ability_result, 0);
                }
                assert_eq!(
                    s.ability_program, 0,
                    "zero modes skip; one legal mode auto-selects"
                );
                if let Some(id) = ground {
                    assert!(s.card(id).exhausted);
                }
            }
        }
    }
}

#[test]
fn modal_arena_lightmaker_exhausts_more_than_prepared_record_capacity_without_truncation() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P2);
        let source = unit(&e, &mut s, Seat::P2, "SEC_207");
        let targets: Vec<_> = (0..40)
            .map(|_| unit(&e, &mut s, Seat::P1, "LOF_256"))
            .collect();
        e.defeat(&mut s, source).unwrap();
        e.finish(&mut s).unwrap();
        assert_eq!(
            s.ability_program, 0,
            "one Ground mode resolves automatically"
        );
        assert!(targets.iter().all(|id| s.card(*id).exhausted));
        assert_eq!(s.event_group.count, 0);
    }
}

#[test]
fn modal_arena_persecutor_both_aliases_damage_all_controllers_in_only_chosen_arena() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for trigger in ["whenPlayed", "onAttack"] {
                for selected in ["Space", "Ground"] {
                    let mut s = arena_board(&e, owner);
                    let source = unit(&e, &mut s, owner, "LAW_178");
                    let own_ground = unit(&e, &mut s, owner, "LOF_256");
                    let enemy_ground = unit(&e, &mut s, owner.other(), "LOF_256");
                    let enemy_space = unit(&e, &mut s, owner.other(), "LOF_034");
                    s.card_mut(enemy_space).phase_hp = 10;
                    start(&e, &mut s, source, trigger);
                    assert_eq!(menu_labels(&e, &s), ["Space", "Ground", "Pass"]);
                    let done = choice(&e, &s, selected);
                    for id in [own_ground, enemy_ground] {
                        assert_eq!(
                            done.card(id).damage,
                            if selected == "Ground" { 3 } else { 0 }
                        );
                    }
                    for id in [source, enemy_space] {
                        assert_eq!(
                            done.card(id).damage,
                            if selected == "Space" { 3 } else { 0 }
                        );
                    }
                    assert_eq!(done.card(done.player(owner.other()).base).damage, 0);
                    assert_eq!(done.ability_program, 0);
                }
            }
        }
    }
}

#[test]
fn modal_arena_persecutor_actual_play_targets_after_unit_payment_and_pass_keeps_result_false() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P1);
        let source = hand(&e, &mut s, "LAW_178");
        for _ in 0..4 {
            let resource = add(&e, &mut s, Seat::P1, "LOF_254");
            s.players[0].resources.push(resource).unwrap();
        }
        let resources = e.ready_resources(&s, Seat::P1);
        let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert!(prompt.players[0].space.contains(source));
        assert!(!prompt.players[0].hand.contains(source));
        assert_eq!(
            e.ready_resources(&prompt, Seat::P1),
            resources - usize::from(e.cost(&s, source))
        );
        assert_eq!(menu_labels(&e, &prompt), ["Space", "Pass"]);
        let mut dirty = prompt;
        dirty.ability_result = 77;
        dirty.ability_succeeded = true;
        dirty.event_result.damage_dealt = 7;
        let prepared = e.program(dirty.ability_program - 1);
        let Instruction::ModalChoice(menu) = &prepared.instructions[dirty.ability_pc as usize]
        else {
            panic!("expected prepared modal menu");
        };
        let pass = menu
            .branches
            .iter()
            .find(|branch| branch.label == "Pass")
            .unwrap();
        let Instruction::Apply(leaf) = &prepared.instructions[pass.start] else {
            panic!("expected Pass leaf");
        };
        assert!(matches!(leaf, Effect::NoAction));
        let mut leaf_state = dirty;
        assert_eq!(e.apply_effect(&mut leaf_state, leaf).unwrap(), (0, false));
        // The shared neutral-leaf wrapper publishes no event identity.
        assert_eq!(leaf_state.event_result, StepEventResult::default());
        let done = choice(&e, &dirty, "Pass");
        assert_eq!(done.ability_result, 0);
        assert!(!done.ability_succeeded);
        assert_eq!(done.event_result.damage_dealt, 0);
        assert_eq!(done.event_result.defeated_units, [0, 0]);
        assert_eq!(done.event_result, StepEventResult::default());
        assert_eq!(done.card(source).damage, 0);
    }
}

#[test]
fn modal_arena_persecutor_pass_resumes_combat_and_self_lethal_mode_preserves_attack_cleanup() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for lethal in [false, true] {
                let mut s = arena_board(&e, owner);
                let source = unit(&e, &mut s, owner, "LAW_178");
                if lethal {
                    s.card_mut(source).damage = e.hp(&s, source) - 2;
                }
                let base = s.player(owner.other()).base;
                let power = e.power(&s, source, true);
                let prompt = declaration(&e, s, source, base);
                assert_eq!(menu_labels(&e, &prompt), ["Space", "Pass"]);
                let done = choice(&e, &prompt, if lethal { "Space" } else { "Pass" });
                assert_eq!(done.card(base).damage, if lethal { 0 } else { power });
                assert_eq!(done.player(owner).discard.contains(source), lethal);
                assert_eq!(done.attack_source, 0);
                assert_eq!(done.ability_program, 0);
            }
        }
    }
}

#[test]
fn modal_arena_persecutor_empty_damage_domains_auto_pass_even_for_departed_source() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P1);
        // Source-derived helper boundary: a captured When Played registration
        // may outlive the entering source; no arena targets remain.
        let source = add(&e, &mut s, Seat::P1, "LAW_178");
        s.players[0].discard.push(source).unwrap();
        let next = s.next_id;
        start(&e, &mut s, source, "whenPlayed");
        assert_eq!(s.ability_program, 0);
        assert_eq!(s.next_id, next);
        assert_eq!(s.ability_result, 0);
        assert!(!s.ability_succeeded);
    }
}

#[test]
fn modal_arena_uniform_damage_snapshots_global_order_and_handles_more_than_32_targets() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P2);
        let source = unit(&e, &mut s, Seat::P2, "LAW_178");
        let arrivals: Vec<_> = (0..40)
            .map(|index| {
                unit(
                    &e,
                    &mut s,
                    if index % 2 == 0 { Seat::P2 } else { Seat::P1 },
                    "LOF_256",
                )
            })
            .collect();
        let p = program(&e, &s, source, "whenPlayed");
        let effect = e
            .program(p)
            .instructions
            .iter()
            .find_map(|instruction| match instruction {
                Instruction::Group(
                    effect @ Effect::Damage {
                        target: Target::ArenaList(list),
                        ..
                    },
                ) if serde_json::to_value(&list.predicate)
                    .unwrap()
                    .to_string()
                    .contains("groundArena") =>
                {
                    Some(effect)
                }
                _ => None,
            })
            .unwrap();
        let mut prepared = s;
        prepared.source = source;
        assert!(e.generate_uniform_damage(&mut prepared, effect).unwrap());
        assert_eq!(
            prepared.event_group.uniform_damage.records[..40]
                .iter()
                .map(|record| record.target())
                .collect::<Vec<_>>(),
            arrivals
        );
        assert_eq!(prepared.event_group.count, 0);
        start(&e, &mut s, source, "whenPlayed");
        let done = choice(&e, &s, "Ground");
        assert!(arrivals
            .iter()
            .all(|id| done.card(*id).damage == 3 && e.in_arena(&done, *id)));
    }
}

#[test]
fn modal_arena_shields_keep_exact_token_identity_and_ability_damage_never_overwhelms() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "LAW_178");
        let first = unit(&e, &mut s, Seat::P2, "LOF_254");
        let second = unit(&e, &mut s, Seat::P2, "LOF_254");
        let next = s.next_id;
        for id in [first, second] {
            e.create_token(&mut s, id, false).unwrap();
            e.create_token(&mut s, id, false).unwrap();
        }
        s.source = source;
        // Synthetic keyword interaction on the actual printed damage program:
        // Overwhelm never converts ability damage into excess base damage.
        e.apply_effect(
            &mut s,
            &Effect::GainKeyword {
                target: Target::name("self"),
                keyword: "overwhelm".into(),
                amount: Expr::number(1),
                duration: "phase".into(),
            },
        )
        .unwrap();
        start(&e, &mut s, source, "whenPlayed");
        let done = finish_shields(&e, choice(&e, &s, "Ground"));
        assert_eq!(done.card(first).damage, 0);
        assert_eq!(done.card(second).damage, 0);
        assert_eq!(
            done.card(first).shield,
            next + 1,
            "first replacement removes oldest Shield"
        );
        assert_eq!(
            done.card(second).shield,
            next + 2,
            "remaining contexts rotate before second replacement"
        );
        assert_eq!(done.card(done.players[1].base).damage, 0);
    }
}

#[test]
fn modal_arena_lethal_batch_captures_target_before_snoke_aura_leaves() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = arena_engine(backend);
        let mut s = arena_board(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "LAW_178");
        let snoke = unit(&e, &mut s, Seat::P2, "SHD_037");
        let target = unit(&e, &mut s, Seat::P1, "LOF_256");
        s.card_mut(snoke).damage = e.hp(&s, snoke) - 2;
        assert_eq!(e.hp(&s, target), 2);
        start(&e, &mut s, source, "whenPlayed");
        let done = choice(&e, &s, "Ground");
        assert!(done.players[1].discard.contains(snoke));
        assert!(
            done.players[0].discard.contains(target),
            "aura departure must not rescue already lethal target"
        );
    }
}

#[test]
fn modal_arena_import_guard_uses_selected_pool_and_execution_rejects_missing_global_order() {
    let e = arena_engine(RulesBackend::Auto);
    let mut s = arena_board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "LAW_178");
    let snapshot = e.snapshot(&s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(restored.zobrist(), s.zobrist());
    let mut legacy = snapshot.clone();
    legacy.as_object_mut().unwrap().remove("arenaUnitOrder");
    assert!(e
        .load_state(&legacy)
        .unwrap_err()
        .contains("arenaUnitOrder"));
    let id = program(&e, &s, source, "whenPlayed");
    s.source = source;
    s.arena_order_exact = false;
    let before = s.zobrist();
    assert!(e
        .begin_program(&mut s, id, 0)
        .unwrap_err()
        .contains("recorded arena order"));
    assert_eq!(s.zobrist(), before, "guard precedes ability field mutation");
    s.arena_order_exact = true;
    e.begin_program(&mut s, id, 0).unwrap();
    s.arena_order_exact = false;
    assert!(e
        .run_program(&mut s)
        .unwrap_err()
        .contains("recorded arena order"));
    let mut selected = e.compiled_pool().unwrap();
    selected["cards"].as_array_mut().unwrap().retain(|card| {
        let code = card["setCode"].as_str().unwrap();
        ["ASH_019", "SOR_014", "IBH_053", "LOF_254"].contains(&code) || code.starts_with("TOKEN_")
    });
    let strict = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&selected).unwrap(),
        include_bytes!("../../../compiler/generated/catalog.json"),
    )
    .unwrap();
    let strict_state = arena_board(&strict, Seat::P1);
    let mut strict_snapshot = strict.snapshot(&strict_state);
    assert!(!strict.needs_modal_arena_order());
    assert!(strict
        .catalogue
        .as_ref()
        .unwrap()
        .programs
        .iter()
        .any(crate::modal_arena_choices::requires_global_order));
    // A legacy snapshot omits both chronology projections and Support evidence;
    // keeping registrationOrder while omitting arenaUnitOrder contradicts it.
    for key in [
        "arenaUnitOrder",
        "registrationOrder",
        "supportStateVersion",
        "supportGrants",
    ] {
        strict_snapshot.as_object_mut().unwrap().remove(key);
    }
    let imported = strict.load_state(&strict_snapshot);
    assert!(
        imported.is_ok(),
        "retained unused programs do not broaden selected-pool reachability: {imported:?}"
    );
}

#[test]
fn modal_arena_full_catalogue_rejects_metadata_alias_context_and_branch_contract_mutations() {
    for code in ["SEC_207", "LAW_178"] {
        let original = entry(code);
        assert!(admitted(original.clone()));
        for key in [
            "stateWatchers",
            "historyHelpers",
            "privatePrograms",
            "classMethods",
            "privateClassConstants",
            "inheritedSource",
            "deploymentOverride",
            "setupValidationOverrides",
        ] {
            let mut bad = original.clone();
            bad[key] = json!([{}]);
            assert!(!admitted(bad), "{code}: {key}");
        }
        for mutation in [
            "leader",
            "mixed-type",
            "pilot",
            "keyword",
            "extra-sibling",
            "cost",
            "continuation",
            "receiver",
            "predicate",
            "chooser",
            "branch-extra",
            "label",
        ] {
            let mut bad = original.clone();
            match mutation {
                "leader" => bad["baseClass"] = json!("LeaderUnitCard"),
                "mixed-type" => bad["card"]["types"] = json!(["unit", "leader"]),
                "pilot" => bad["card"]["pilotText"] = json!("Pilot ability"),
                "keyword" => bad["keywordValues"] = json!({"raid":2}),
                "extra-sibling" => bad["abilities"]["card"].as_array_mut().unwrap().push(
                    json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}),
                ),
                "cost" => bad["abilities"]["card"][0]["costs"] = json!([{"kind":"exhaustSelf"}]),
                "continuation" => {
                    bad["abilities"]["card"][0]["then"] = json!({"effect":{"kind":"noAction"}})
                }
                "receiver" => {
                    bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][0]
                        ["effect"]["target"]["receiver"]["path"] = json!(["player"])
                }
                "predicate" => {
                    bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][0]
                        ["effect"]["target"]["predicate"] = json!({"kind":"const","value":true})
                }
                "chooser" => {
                    bad["abilities"]["card"][0]["targetResolver"]["chooser"]["value"] =
                        json!("opponent")
                }
                "branch-extra" => {
                    bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][0]
                        ["effect"]["extra"] = json!(true)
                }
                _ => {
                    bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][0]
                        ["labelParts"][0]["value"] = json!("Changed")
                }
            }
            assert!(!admitted(bad), "{code}: {mutation}");
        }
    }
    let original = entry("LAW_178");
    for mutation in [
        "missing-alias",
        "wrong-alias",
        "mismatched-title",
        "pass-false",
        "pass-dynamic",
        "damage-amount",
    ] {
        let mut bad = original.clone();
        match mutation {
            "missing-alias" => {
                bad["abilities"]["card"].as_array_mut().unwrap().pop();
            }
            "wrong-alias" => bad["abilities"]["card"][1]["trigger"] = json!("whenDefeated"),
            "mismatched-title" => bad["abilities"]["card"][1]["title"] = json!("Different"),
            "pass-false" => {
                bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][2]["effect"]
                    ["hasLegalTarget"]["value"] = json!(false)
            }
            "pass-dynamic" => {
                bad["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][2]["effect"]
                    ["hasLegalTarget"] = json!({"kind":"ref","path":["source"]})
            }
            _ => {
                bad["abilities"]["card"][1]["targetResolver"]["choices"]["entries"][0]["effect"]
                    ["amount"]["value"] = json!(2)
            }
        }
        assert!(!admitted(bad), "LAW_178: {mutation}");
    }
}
