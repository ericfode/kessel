// Include as a child module of generic_runtime_tests.rs. Canonical card behavior
// tests retain SOR_082 and SEC_201 source/metadata. Labeled diagnostic carriers
// rename admitted fixture definitions; isolated rejection/dependency tests make
// explicit raw-IR mutations rather than claiming canonical behavior witnesses.
use super::*;

fn controlled_title_board(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}

fn controlled_title_roundtrip(e: &Engine, s: &State) -> State {
    let snapshot = e.snapshot(s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(e.legal_actions(&restored), e.legal_actions(s));
    restored
}

fn diagnostic_title_engine(changes: &[(&str, &str)]) -> Engine {
    let mut catalog: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    for &(code, title) in changes {
        assert!(!["SOR_082", "SEC_201"].contains(&code));
        let entry = catalog["cards"].as_array_mut().unwrap().iter_mut().find(|entry| entry["setCode"] == code).unwrap();
        entry["card"]["title"] = json!(title);
    }
    Engine::from_catalogue_value(&catalog).unwrap()
}

fn title_query(e: &Engine, s: &State, source: CardId, opponent: bool, title: &str) -> bool {
    let expr: crate::ability_ir::Expr = serde_json::from_value(json!({"kind":"query", "name":"hasControlledInPlayTitle",
        "receiver":{"kind":"ref","path":[if opponent {"opponent"} else {"player"}]},
        "args":[{"kind":"const","value":title}]})).unwrap();
    expr.validate().unwrap();
    e.eval(s, source, 0, &expr).truth()
}

#[test]
fn controlled_title_actual_padme_preserves_anakin_raid_hidden_and_real_attack() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for friendly in [false, true] {
            let mut s = controlled_title_board(&e, seat);
            let anakin = unit(&e, &mut s, seat, "SEC_201");
            let padme = unit(&e, &mut s, if friendly { seat } else { seat.other() }, "TWI_192");
            let base = s.player(seat.other()).base;
            let s = controlled_title_roundtrip(&e, &s);
            assert_eq!(e.def(&s, padme).info.title, "Padmé Amidala");
            assert_eq!(e.keyword_amount(&s, anakin, "hidden"), 1);
            assert_eq!(e.keyword_amount(&s, anakin, "raid"), if friendly { 2 } else { 0 });
            assert_eq!(e.power(&s, anakin, false), 3);
            assert_eq!(e.power(&s, anakin, true), if friendly { 5 } else { 3 });
            let attack = e.card_action(&s, anakin);
            assert!(e.legal_actions(&s).contains(&attack));
            let attack_target = e.apply(&s, &attack).unwrap();
            assert_eq!(attack_target.decision, Decision::AttackTarget);
            let target = e.card_action(&attack_target, base);
            assert!(e.legal_actions(&attack_target).contains(&target));
            let done = e.apply(&attack_target, &target).unwrap();
            assert_eq!(done.card(base).damage, if friendly { 5 } else { 3 });
            controlled_title_roundtrip(&e, &done);
        }
        // Canonical ordinary play still records Hidden qualification independently
        // of the new title condition and its current Raid grant.
        let mut s = controlled_title_board(&e, seat);
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        unit(&e, &mut s, seat, "TWI_192");
        let anakin = add(&e, &mut s, seat, "SEC_201");
        s.player_mut(seat).hand.push(anakin).unwrap();
        for _ in 0..8 {
            let resource = add(&e, &mut s, seat, "LOF_256");
            s.player_mut(seat).resources.push(resource).unwrap();
        }
        let s = controlled_title_roundtrip(&e, &s);
        let play = e.card_action(&s, anakin);
        assert!(e.legal_actions(&s).contains(&play));
        let played = e.apply(&s, &play).unwrap();
        assert!(played.card(anakin).hidden_qualified());
        assert_eq!(e.keyword_amount(&played, anakin, "hidden"), 1);
        assert_eq!(e.keyword_amount(&played, anakin, "raid"), 2);
        assert!(!e.attack_targets(&played, enemy, false).contains(&anakin));
        controlled_title_roundtrip(&e, &played);
    }
}

#[test]
fn controlled_title_diagnostic_unit_enables_guard_hp_and_loss_is_lethal_live() {
    let e = diagnostic_title_engine(&[("LOF_254", "Emperor Palpatine")]);
    for seat in [Seat::P1, Seat::P2] {
        let mut s = controlled_title_board(&e, seat);
        let guard = unit(&e, &mut s, seat, "SOR_082");
        assert_eq!(e.hp(&s, guard), 4);
        assert!(!e.has_keyword(&s, guard, "sentinel"));
        let palpatine = unit(&e, &mut s, seat, "LOF_254");
        assert_eq!(e.hp(&s, guard), 5);
        assert!(!e.has_keyword(&s, guard, "sentinel"), "a title does not invent Official");
        s.card_mut(guard).damage = 4;
        let mut s = controlled_title_roundtrip(&e, &s);
        assert!(e.in_arena(&s, guard));
        e.defeat(&mut s, palpatine).unwrap();
        assert_eq!(e.hp(&s, guard), 4);
        e.sweep_stat_defeats(&mut s).unwrap();
        assert!(!e.in_arena(&s, guard));
        assert!(s.player(seat).discard.contains(guard));
        controlled_title_roundtrip(&e, &s);
    }
}

#[test]
fn controlled_title_guard_official_condition_stays_independent() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = controlled_title_board(&e, seat);
        let guard = unit(&e, &mut s, seat, "SOR_082");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        let official = unit(&e, &mut s, seat, "TOKEN_SPY");
        let mut s = controlled_title_roundtrip(&e, &s);
        assert!(e.has_keyword(&s, guard, "sentinel"));
        assert_eq!(e.hp(&s, guard), 4);
        assert_eq!(e.attack_targets(&s, enemy, false), vec![guard]);
        e.defeat(&mut s, official).unwrap();
        assert!(!e.has_keyword(&s, guard, "sentinel"));
        assert!(e.attack_targets(&s, enemy, false).contains(&s.player(seat).base));
        controlled_title_roundtrip(&e, &s);
    }
}

#[test]
fn controlled_title_diagnostic_leader_counts_undeployed_deployed_and_reverted() {
    for (title, source_code) in [("Emperor Palpatine", "SOR_082"), ("Padmé Amidala", "SEC_201")] {
        let e = diagnostic_title_engine(&[("SOR_014", title)]);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = controlled_title_board(&e, seat);
            let leader = s.player(seat).leader;
            let opposing_leader = s.player(seat.other()).leader;
            s.card_mut(leader).def = e.by_code["SOR_014"];
            s.card_mut(opposing_leader).def = e.by_code["IBH_053"];
            let source = unit(&e, &mut s, seat, source_code);
            let mut s = controlled_title_roundtrip(&e, &s);
            assert!(title_query(&e, &s, source, false, title));
            assert!(!title_query(&e, &s, source, true, title));
            if source_code == "SOR_082" { assert_eq!(e.hp(&s, source), 5); }
            else { assert_eq!(e.keyword_amount(&s, source, "raid"), 2); }
            s.card_mut(leader).deployed = true;
            s.card_mut(leader).epic_used = true;
            e.add_arena(&mut s, leader).unwrap();
            let mut s = controlled_title_roundtrip(&e, &s);
            assert!(title_query(&e, &s, source, false, title));
            e.defeat(&mut s, leader).unwrap();
            assert!(!s.card(leader).deployed);
            assert!(title_query(&e, &s, source, false, title), "leader returns to base zone");
            controlled_title_roundtrip(&e, &s);
        }
    }
}

#[test]
fn controlled_title_diagnostic_upgrade_uses_its_controller_on_either_host() {
    for (title, source_code) in [("Emperor Palpatine", "SOR_082"), ("Padmé Amidala", "SEC_201")] {
        let e = diagnostic_title_engine(&[("HMW_096", title)]);
        for seat in [Seat::P1, Seat::P2] {
            for upgrade_controller in [seat, seat.other()] {
                for host_controller in [seat, seat.other()] {
                    let mut s = controlled_title_board(&e, seat);
                    let source = unit(&e, &mut s, seat, source_code);
                    let host = unit(&e, &mut s, host_controller, "LOF_256");
                    let upgrade = add(&e, &mut s, upgrade_controller, "HMW_096");
                    e.attach(&mut s, upgrade, host);
                    let mut s = controlled_title_roundtrip(&e, &s);
                    let own = upgrade_controller == seat;
                    assert_eq!(title_query(&e, &s, source, false, title), own);
                    assert_eq!(title_query(&e, &s, source, true, title), !own);
                    if source_code == "SOR_082" { assert_eq!(e.hp(&s, source), if own { 5 } else { 4 }); }
                    else { assert_eq!(e.keyword_amount(&s, source, "raid"), if own { 2 } else { 0 }); }
                    e.leave_play_primary(&mut s, upgrade).unwrap();
                    assert!(!title_query(&e, &s, source, false, title));
                    assert!(!title_query(&e, &s, source, true, title));
                    controlled_title_roundtrip(&e, &s);
                }
            }
        }
    }
}

#[test]
fn controlled_title_diagnostic_hidden_zones_wrong_owner_and_exact_accent() {
    for (title, source_code) in [("Emperor Palpatine", "SOR_082"), ("Padmé Amidala", "SEC_201")] {
        let e = diagnostic_title_engine(&[("LOF_254", title)]);
        for seat in [Seat::P1, Seat::P2] {
            for zone in ["hand", "discard", "deck", "resource"] {
                let mut s = controlled_title_board(&e, seat);
                let source = unit(&e, &mut s, seat, source_code);
                let carrier = add(&e, &mut s, seat, "LOF_254");
                match zone {
                    "hand" => s.player_mut(seat).hand.push(carrier).unwrap(),
                    "discard" => s.player_mut(seat).discard.push(carrier).unwrap(),
                    "deck" => s.player_mut(seat).deck.push(carrier).unwrap(),
                    _ => s.player_mut(seat).resources.push(carrier).unwrap(),
                }
                let s = controlled_title_roundtrip(&e, &s);
                assert!(!title_query(&e, &s, source, false, title), "excluded zone {zone}");
                if source_code == "SOR_082" { assert_eq!(e.hp(&s, source), 4); }
                else { assert_eq!(e.keyword_amount(&s, source, "raid"), 0); }
            }
            let mut s = controlled_title_board(&e, seat);
            let source = unit(&e, &mut s, seat, source_code);
            unit(&e, &mut s, seat.other(), "LOF_254");
            let s = controlled_title_roundtrip(&e, &s);
            assert!(!title_query(&e, &s, source, false, title));
            assert!(title_query(&e, &s, source, true, title));
        }
    }
    for wrong in ["Padme Amidala", "PADMÉ AMIDALA", "Padmé Amidala ", "Padme\u{301} Amidala"] {
        let e = diagnostic_title_engine(&[("LOF_254", wrong)]);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = controlled_title_board(&e, seat);
            let source = unit(&e, &mut s, seat, "SEC_201");
            unit(&e, &mut s, seat, "LOF_254");
            let s = controlled_title_roundtrip(&e, &s);
            assert!(!title_query(&e, &s, source, false, "Padmé Amidala"));
            assert_eq!(e.keyword_amount(&s, source, "raid"), 0);
        }
    }
}

#[test]
fn controlled_title_diagnostic_base_and_force_presence_are_in_play_domain() {
    let e = diagnostic_title_engine(&[("ASH_019", "DIAGNOSTIC base title")]);
    for seat in [Seat::P1, Seat::P2] {
        let mut s = controlled_title_board(&e, seat);
        let source = unit(&e, &mut s, seat, "SOR_082");
        let s = controlled_title_roundtrip(&e, &s);
        assert!(title_query(&e, &s, source, false, "DIAGNOSTIC base title"));
        assert!(!title_query(&e, &s, source, false, "The Force"));
        let mut held = s;
        held.player_mut(seat).force_held = true;
        let held = controlled_title_roundtrip(&e, &held);
        assert!(title_query(&e, &held, source, false, "The Force"));
        assert!(!title_query(&e, &held, source, true, "The Force"));
    }
}

#[test]
fn controlled_title_leaf_validation_rejects_malformed_domains_and_values() {
    let leaf = json!({"kind":"query","name":"hasControlledInPlayTitle","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"const","value":"Padmé Amidala"}]});
    for root in ["player", "opponent"] {
        let mut valid = leaf.clone();
        valid["receiver"]["path"] = json!([root]);
        let expr: crate::ability_ir::Expr = serde_json::from_value(valid).unwrap();
        expr.validate().unwrap();
        assert!(expr.validate_number().is_err(), "existence is boolean, not an amount");
    }
    let mut invalid = Vec::new();
    for path in [json!(["source"]), json!(["card"]), json!(["target"]), json!(["player", "base"]), json!(["source", "controller"]), json!(["modifierTarget", "controller"])] {
        let mut value = leaf.clone(); value["receiver"]["path"] = path; invalid.push(value);
    }
    for value in [json!(""), json!("x".repeat(129)), json!("é".repeat(65)), json!(false), json!(1), Value::Null] {
        let mut invalid_value = leaf.clone(); invalid_value["args"][0]["value"] = value; invalid.push(invalid_value);
    }
    for args in [json!([]), json!([{"kind":"const","value":"Padmé Amidala"},{"kind":"const","value":"extra"}]), json!([{"kind":"ref","path":["source","title"]}]), json!([{"kind":"conditional","condition":{"kind":"const","value":true},"then":{"kind":"const","value":"Padmé Amidala"},"else":{"kind":"const","value":"Emperor Palpatine"}}])] {
        let mut value = leaf.clone(); value["args"] = args; invalid.push(value);
    }
    let mut invalid_receiver = leaf.clone(); invalid_receiver["receiver"] = json!({"kind":"const","value":"player"}); invalid.push(invalid_receiver);
    for value in invalid {
        let expr: crate::ability_ir::Expr = serde_json::from_value(value.clone()).unwrap();
        assert!(expr.validate().is_err(), "malformed title query admitted: {value}");
    }
    // UTF-8 bytes, not scalar count, bind the same 128-byte runtime/compiler limit.
    let mut boundary = leaf; boundary["args"][0]["value"] = json!("é".repeat(64));
    serde_json::from_value::<crate::ability_ir::Expr>(boundary).unwrap().validate().unwrap();
}

#[test]
fn controlled_title_boolean_query_rejects_numeric_phase_stats_and_resources() {
    let query = json!({"kind":"query","name":"hasControlledInPlayTitle","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"const","value":"Padmé Amidala"}]});
    for effect in [
        json!({"kind":"modifyStats","target":"self","duration":"phase","power":query.clone(),"hp":{"kind":"const","value":0}}),
        json!({"kind":"readyResources","target":"ownPlayer","amount":query.clone()}),
        json!({"kind":"damage","target":"self","amount":query.clone()}),
        json!({"kind":"heal","target":"self","amount":query.clone()}),
        json!({"kind":"giveShield","target":"self","amount":query.clone()}),
        json!({"kind":"giveExperience","target":"self","amount":query.clone()}),
    ] {
        let ability: crate::ability_ir::Ability = serde_json::from_value(json!({"trigger":"whenPlayed","title":"DIAGNOSTIC boolean amount","costs":[],"effect":effect})).unwrap();
        assert!(crate::ability_ir::Program::compile(&ability).is_err());
    }
    let numeric = json!({"kind":"conditional","condition":query,"then":{"kind":"const","value":2},"else":{"kind":"const","value":1}});
    let ability: crate::ability_ir::Ability = serde_json::from_value(json!({"trigger":"whenPlayed","title":"DIAGNOSTIC numeric branches","costs":[],
        "effect":{"kind":"modifyStats","target":"self","duration":"phase","power":numeric,"hp":{"kind":"const","value":0}}})).unwrap();
    crate::ability_ir::Program::compile(&ability).unwrap();
}

#[test]
fn controlled_title_admission_fence_accepts_reviewed_consumers_and_rejects_scope_mutations() {
    let e = engine();
    for code in ["JTL_147", "SEC_027", "SOR_184"] {
        assert!(e.by_code.contains_key(code), "reviewed scalar title consumer is admitted: {code}");
    }
    let original: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    for mutation in ["trigger", "optional", "coordinate", "target", "duration", "selector", "continuation", "nested-condition", "pilot", "epic", "type", "side"] {
        let mut catalog = original.clone();
        let entry = catalog["cards"].as_array_mut().unwrap().iter_mut().find(|entry| entry["setCode"] == "SOR_082").unwrap();
        let a = &mut entry["abilities"]["card"][1];
        match mutation {
            "trigger" => a["trigger"] = json!("whenPlayed"),
            "optional" => a["optional"] = json!(true),
            "coordinate" => a["coordinate"] = json!(true),
            "target" => a["effect"]["target"] = json!("chosen"),
            "duration" => a["effect"]["duration"] = json!("phase"),
            "selector" => a["targetResolver"] = json!({"owner":"self","type":"unit","zone":"arena"}),
            "continuation" => a["then"] = json!({"title":"DIAGNOSTIC extra","effect":{"kind":"noAction"}}),
            "nested-condition" => { let condition = a["condition"].clone(); a["condition"] = json!({"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":condition}); },
            "pilot" => entry["card"]["pilotText"] = json!("DIAGNOSTIC pilot"),
            "epic" => entry["card"]["epicAction"] = json!("DIAGNOSTIC epic"),
            "type" => entry["card"]["types"] = json!(["unit","token"]),
            _ => entry["abilities"]["unit"] = json!([{"trigger":"onAttack","title":"DIAGNOSTIC side","effect":{"kind":"noAction"}}]),
        }
        let rejected = Engine::from_catalogue_value(&catalog).unwrap();
        assert!(!rejected.by_code.contains_key("SOR_082"), "title-family fence admitted {mutation}");
    }
}

#[test]
fn controlled_title_force_literal_requires_native_definition_and_explicit_state() {
    // Synthetic diagnostic mutation of Royal Guard's complete ordinary source:
    // replace only its Palpatine condition with The Force. This is a query
    // dependency boundary, not a canonical Royal Guard behavior witness.
    let mut catalog: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let entry = catalog["cards"].as_array_mut().unwrap().iter_mut().find(|entry| entry["setCode"] == "SOR_082").unwrap();
    entry["abilities"]["card"][1]["title"] = json!("DIAGNOSTIC title The Force presence");
    entry["abilities"]["card"][1]["condition"]["args"][0]["value"] = json!("The Force");
    let mut e = Engine::from_catalogue_value(&catalog).unwrap();
    assert!(e.by_code.contains_key("SOR_082"), "diagnostic source rejected: {:?}", e.rejected_cards().unwrap().get("SOR_082"));
    let mut missing = catalog.clone();
    missing["cards"].as_array_mut().unwrap().retain(|entry| entry["setCode"] != "TOKEN_THE_FORCE");
    let rejected = Engine::from_catalogue_value(&missing).unwrap();
    assert!(!rejected.by_code.contains_key("SOR_082"), "title-only Force dependency must require the native definition");
    assert!(rejected.rejected_cards().unwrap()["SOR_082"].contains("Force"), "unexpected rejection: {:?}", rejected.rejected_cards().unwrap().get("SOR_082"));
    let bytes = serde_json::to_vec(&catalog).unwrap();
    let metadata = &e.catalogue.as_ref().unwrap().metadata;
    let pool = |codes: &[&str]| serde_json::to_vec(&json!({"schema":"omni-rs-pool-v1","cards":codes.iter().map(|code| {
        let mut card = metadata[*code].clone(); card["setCode"] = json!(code); card
    }).collect::<Vec<_>>()})).unwrap();
    let missing_pool = pool(&["ASH_019","SOR_014","IBH_053","TOKEN_SHIELD","SOR_082"]);
    assert!(Engine::from_compiled_pool_bytes(&missing_pool, &bytes).err().unwrap().contains("TOKEN_THE_FORCE"));
    let complete_pool = pool(&["ASH_019","SOR_014","IBH_053","TOKEN_SHIELD","SOR_082","TOKEN_THE_FORCE"]);
    assert!(Engine::from_compiled_pool_bytes(&complete_pool, &bytes).is_ok());
    let s = controlled_title_board(&e, Seat::P1);
    let mut snapshot = without_empty_credit_state(e.snapshot(&s));
    snapshot.as_object_mut().unwrap().remove("forceStateVersion");
    for seat in ["p1", "p2"] { snapshot["players"][seat].as_object_mut().unwrap().remove("forceToken"); }
    e.catalogue.as_mut().unwrap().metadata.retain(|code, _| ["ASH_019","SOR_014","IBH_053","TOKEN_THE_FORCE","SOR_082"].contains(&code.as_str()));
    assert!(e.load_state(&snapshot).unwrap_err().contains("Force"));
    e.catalogue.as_mut().unwrap().metadata.remove("SOR_082");
    assert!(e.load_state(&snapshot).is_ok(), "unrelated selected metadata retains the legacy compatibility boundary");
}

#[test]
fn controlled_title_auto_and_interpreter_match_actual_attack_and_live_hp_with_explicit_aot_fallback() {
    let mut observations = Vec::new();
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        // Palpatine is an explicitly diagnostic inert carrier. Both focus
        // definitions and canonical Padmé TWI_192 remain unchanged.
        let mut e = diagnostic_title_engine(&[("LOF_254", "Emperor Palpatine")]);
        // Remove unrelated programs so the forced-AOT rejection identifies
        // the focused constants' existing live-modifier program boundary.
        for definition in &mut e.definitions {
            if !["SOR_082", "SEC_201"].contains(&definition.code.as_str()) {
                definition.programs = Default::default();
            }
        }
        e.set_rules_backend(backend).unwrap();
        if backend == RulesBackend::Auto {
            let reason = e.set_rules_backend(RulesBackend::Aot).unwrap_err();
            assert!(reason.contains("constant") && reason.contains("live modifier"), "unexpected AOT rejection: {reason}");
        }
        e.set_rules_instrumentation(true);
        let mut scenarios = Vec::new();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = controlled_title_board(&e, seat);
            let anakin = unit(&e, &mut s, seat, "SEC_201");
            let guard = unit(&e, &mut s, seat, "SOR_082");
            let padme = unit(&e, &mut s, seat, "TWI_192");
            let palpatine = unit(&e, &mut s, seat, "LOF_254");
            let base = s.player(seat.other()).base;
            let s = controlled_title_roundtrip(&e, &s);
            assert_eq!(e.keyword_amount(&s, anakin, "raid"), 2);
            assert_eq!(e.hp(&s, guard), 5);
            // Measure the exact new query leaves independently of other
            // constant/query evaluations in snapshots and combat.
            for (source, slot, title) in [(anakin, 0, "Padmé Amidala"), (guard, 1, "Emperor Palpatine")] {
                let program = e.def(&s, source).programs.card[slot];
                let condition = e.program(program).condition.as_ref().unwrap().clone();
                assert!(matches!(&condition, crate::ability_ir::Expr::Compiled(_)), "new title query lacks generated expression binding");
                assert!(matches!(condition.original(), crate::ability_ir::Expr::Query {name,args,..} if name == "hasControlledInPlayTitle" && matches!(&args[0],crate::ability_ir::Expr::Const {value} if value.as_str() == Some(title))));
                *e.expression_hits.lock().unwrap() = [0, 0];
                assert!(e.eval(&s, source, 0, &condition).truth());
                let hits = *e.expression_hits.lock().unwrap();
                if backend == RulesBackend::Interpreter {
                    assert_eq!(hits[0], 0);
                    assert!(hits[1] > 0, "interpreter title leaf not exercised");
                } else {
                    assert!(hits[0] > 0, "generated title leaf not exercised");
                    assert_eq!(hits[1], 0);
                }
            }
            let before = (e.snapshot(&s), e.legal_actions(&s), s.zobrist());
            let attack = e.card_action(&s, anakin);
            assert!(e.legal_actions(&s).contains(&attack));
            let choosing = e.apply(&s, &attack).unwrap();
            assert_eq!(choosing.decision, Decision::AttackTarget);
            let target = e.card_action(&choosing, base);
            assert!(e.legal_actions(&choosing).contains(&target));
            let targeting = (e.snapshot(&choosing), e.legal_actions(&choosing), choosing.zobrist());
            let done = e.apply(&choosing, &target).unwrap();
            assert_eq!(done.card(base).damage, 5);
            let mut done = controlled_title_roundtrip(&e, &done);
            let after = (e.snapshot(&done), e.legal_actions(&done), done.zobrist());
            e.defeat(&mut done, padme).unwrap();
            e.defeat(&mut done, palpatine).unwrap();
            e.sweep_stat_defeats(&mut done).unwrap();
            assert_eq!(e.keyword_amount(&done, anakin, "raid"), 0);
            assert_eq!(e.hp(&done, guard), 4);
            let lost = controlled_title_roundtrip(&e, &done);
            scenarios.push((before, targeting, after, e.snapshot(&lost), e.legal_actions(&lost), lost.zobrist()));
        }
        observations.push(scenarios);
    }
    assert_eq!(observations[0], observations[1]);
}
