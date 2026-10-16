use super::*;
use crate::ability_ir::Catalogue;

// Expected outcomes are read from the pinned complete four card definitions,
// OngoingCardEffect's source-only default, UnitProperties.effectsPreventAttack,
// and PutIntoPlaySystem's unconditional EntersPlayReady check before entry.
#[test]
fn static_ongoing_canonical_self_attack_restrictions_and_saboteur() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e);
            s.active = seat;
            let wicket = unit(&e, &mut s, seat, "ASH_034");
            let sentinel = unit(&e, &mut s, seat.other(), "LOF_044");
            let ordinary = unit(&e, &mut s, seat.other(), "LOF_256");
            // Saboteur ignores Sentinel, but it cannot bypass its own base ban.
            assert_eq!(
                e.attack_targets(&s, wicket, false),
                vec![sentinel, ordinary]
            );
            assert_eq!(e.attack_targets(&s, wicket, true), vec![sentinel, ordinary]);
            let prompt = e.apply(&s, &e.card_action(&s, wicket)).unwrap();
            assert!(!e
                .legal_actions(&prompt)
                .iter()
                .any(|a| a.id() == Some(s.player(seat.other()).base)));
            assert!(e
                .apply(
                    &prompt,
                    &e.card_action(&prompt, s.player(seat.other()).base)
                )
                .is_err());
            let fought = e.apply(&prompt, &e.card_action(&prompt, ordinary)).unwrap();
            assert_eq!(fought.card(ordinary).damage, 3);
            assert_eq!(fought.card(wicket).damage, 1);
            assert!(fought.card(wicket).exhausted);
            assert_eq!(fought.card(s.player(seat.other()).base).damage, 0);
            // The self restriction does not affect another friendly attacker.
            let ally = unit(&e, &mut s, seat, "LOF_256");
            assert_eq!(e.attack_targets(&s, ally, false), vec![sentinel]);
            s.card_mut(sentinel).damage = 0;
            e.defeat(&mut s, sentinel).unwrap();
            assert!(e
                .attack_targets(&s, ally, false)
                .contains(&s.player(seat.other()).base));
            assert!(!e
                .attack_targets(&s, wicket, false)
                .contains(&s.player(seat.other()).base));
        }
    }
}
#[test]
fn static_ongoing_base_attack_ban_does_not_prevent_overwhelm_damage() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e);
        let wicket = unit(&e, &mut s, Seat::P1, "ASH_034");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        // A legal granted keyword is shared fixture setup, not printed Wicket text.
        s.card_mut(wicket).add_phase_keyword(
            crate::ability_ir::CORE_KEYWORDS
                .iter()
                .position(|k| *k == "overwhelm")
                .unwrap(),
        );
        let prompt = e.apply(&s, &e.card_action(&s, wicket)).unwrap();
        let fought = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
        assert!(fought.players[1].discard.contains(defender));
        assert_eq!(fought.card(fought.players[1].base).damage, 2);
    }
}
#[test]
fn static_ongoing_cannot_attack_still_defends_and_keeps_sentinel() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for code in ["JTL_059", "LOF_044"] {
            let mut s = board(&e);
            let blocked = unit(&e, &mut s, Seat::P1, code);
            assert!(e.attack_targets(&s, blocked, false).is_empty());
            assert!(e.attack_targets(&s, blocked, true).is_empty());
            // InitiateAttackAction::meetsRequirements requires a legal target;
            // a blocked attacker has no action to click, even while ready.
            assert!(!e.legal_actions(&s).contains(&e.card_action(&s, blocked)));
            assert!(e.apply(&s, &e.card_action(&s, blocked)).is_err());
            // A new enemy in the same arena can attack this unit. Its own
            // restriction is not a blanket immunity or a removed Sentinel.
            let attacker = unit(
                &e,
                &mut s,
                Seat::P2,
                if code == "JTL_059" {
                    "TWI_253"
                } else {
                    "LOF_256"
                },
            );
            assert!(e.attack_targets(&s, attacker, false).contains(&blocked));
            if code == "LOF_044" {
                assert_eq!(e.attack_targets(&s, attacker, false), vec![blocked]);
            }
            s.active = Seat::P2;
            let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
            let fought = e.apply(&prompt, &e.card_action(&prompt, blocked)).unwrap();
            assert_eq!(fought.card(blocked).damage, 1);
            assert_eq!(fought.card(attacker).damage, 3);
        }
    }
}
#[test]
fn static_ongoing_enters_ready_hand_play_and_unique_boundary() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for duplicate in [false, true] {
            let mut s = board(&e);
            let old = duplicate.then(|| unit(&e, &mut s, Seat::P1, "HMW_203"));
            let incoming = hand(&e, &mut s, "HMW_203");
            // This fixture's bases/leaders do not provide Cunning or Villainy;
            // include the source's two aspect penalties in payment setup.
            while s.players[0].resources.len < e.cost(&s, incoming) as u8 {
                let resource = add(&e, &mut s, Seat::P1, "LOF_254");
                s.players[0].resources.push(resource).unwrap();
            }
            s.card_mut(incoming).exhausted = true;
            let mut played = e.apply(&s, &e.card_action(&s, incoming)).unwrap();
            assert!(e.in_arena(&played, incoming));
            assert!(!played.card(incoming).exhausted);
            assert!(!played.players[0].hand.contains(incoming));
            assert!(played.players[0]
                .resources
                .as_slice()
                .iter()
                .any(|id| played.card(*id).exhausted));
            if let Some(old) = old {
                assert_eq!(played.decision, Decision::Unique);
                played = e.apply(&played, &e.card_action(&played, old)).unwrap();
                assert!(played.players[0].discard.contains(old));
                assert!(!played.card(incoming).exhausted);
            }
            assert!(e
                .attack_targets(&played, incoming, false)
                .contains(&played.players[1].base));
        }
        // Vanilla units still enter exhausted; readiness is not globally granted.
        let mut s = board(&e);
        let vanilla = hand(&e, &mut s, "LOF_256");
        let played = e.apply(&s, &e.card_action(&s, vanilla)).unwrap();
        assert!(played.card(vanilla).exhausted);
    }
}
#[test]
fn static_ongoing_import_and_plain_placement_preserve_explicit_exhaustion() {
    let e = engine();
    let mut s = board(&e);
    let id = unit(&e, &mut s, Seat::P1, "HMW_203");
    s.card_mut(id).exhausted = true;
    let snapshot = e.snapshot(&s);
    let imported = e.load_state(&snapshot).unwrap();
    assert!(imported.card(id).exhausted);
    assert_eq!(e.snapshot(&imported), snapshot);
    // add_arena is a placement helper, not a hand-play readiness event.
    let id = add(&e, &mut s, Seat::P2, "HMW_203");
    s.card_mut(id).exhausted = true;
    e.add_arena(&mut s, id).unwrap();
    assert!(s.card(id).exhausted);
}
#[test]
fn static_ongoing_raw_contract_rejects_unknown_dynamic_and_widened_scopes() {
    let original: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["ASH_034", "JTL_059", "LOF_044", "HMW_203"] {
        let entry = original["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == code)
            .unwrap();
        let load = |entry: Value| {
            Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[entry]})).unwrap()
        };
        assert!(load(entry.clone()).cards.contains_key(code));
        for path in [
            "unknown-wrapper",
            "unknown-properties",
            "unknown-modifier",
            "unknown-value",
            "dynamic",
            "scope",
            "target",
            "duration",
            "context",
            "defaults",
            "dependency",
            "source-zone",
            "builder",
            "value",
            "aura",
            "nested",
            "condition",
            "optional",
            "cost",
            "selector",
            "extra-registration",
            "extra-side",
            "leader",
            "mixed",
            "attachment",
            "pilot",
        ] {
            let mut v = entry.clone();
            let a = &mut v["abilities"]["card"][0];
            match path {
                "unknown-wrapper" => a["effect"]["unimplemented"] = json!(true),
                "unknown-properties" => a["effect"]["properties"]["unimplemented"] = json!(true),
                "unknown-modifier" => a["effect"]["modifier"]["unimplemented"] = json!(true),
                "unknown-value" => a["effect"]["modifier"]["value"]["unimplemented"] = json!(true),
                "dynamic" => a["effect"]["modifier"]["valueMode"] = json!("dynamic"),
                "scope" => a["effect"]["modifier"]["scope"] = json!("player"),
                "target" => a["effect"]["modifier"]["target"] = json!("self"),
                "duration" => a["effect"]["duration"] = json!("phase"),
                "context" => a["effect"]["context"] = json!("snapshot"),
                "defaults" => a["effect"]["scopeDefaults"] = json!("none"),
                "dependency" => a["effect"]["dependencyPolicy"] = json!("ignore"),
                "source-zone" => {
                    a["effect"]["properties"] = if code == "HMW_203" {
                        json!({})
                    } else {
                        json!({"sourceZoneFilter":"any"})
                    }
                }
                "builder" => {
                    if code == "HMW_203" {
                        a["effect"]["modifier"]
                            .as_object_mut()
                            .unwrap()
                            .remove("builder");
                    } else {
                        a["effect"]["modifier"]["builder"] = json!("static");
                    }
                }
                "value" => a["effect"]["modifier"]["value"] = json!({"kind":"const","value":true}),
                "aura" => a["effect"]["properties"]["targetController"] = json!("self"),
                "nested" => {
                    a["effect"] = json!({"kind":"simultaneous","effects":[a["effect"].clone()]})
                }
                "condition" => a["condition"] = json!({"kind":"const","value":true}),
                "optional" => a["optional"] = json!(true),
                "cost" => a["costs"] = json!([{"kind":"exhaustSelf"}]),
                "selector" => {
                    a["targetResolver"] = json!({"owner":"self","type":"unit","zone":"arena"})
                }
                "extra-registration" => {
                    let extra = a.clone();
                    v["abilities"]["card"].as_array_mut().unwrap().push(extra);
                }
                "extra-side" => v["abilities"]["unit"] = v["abilities"]["card"].clone(),
                "leader" => {
                    v["baseClass"] = json!("LeaderUnitCard");
                    v["card"]["types"] = json!(["leader"]);
                }
                "mixed" => v["card"]["types"] = json!(["unit", "token"]),
                "attachment" => v["attachCondition"] = json!({"kind":"const","value":true}),
                _ => v["piloting"] = json!({"cost":1}),
            }
            let c = load(v);
            assert!(c.rejected.contains_key(code), "{code}:{path}");
        }
    }
}

#[test]
fn static_ongoing_original_admission_delta_remains_exact() {
    let original: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let admitted = Engine::from_catalogue_value(&original).unwrap();
    let mut baseline = original.clone();
    let focus = ["ASH_034", "JTL_059", "LOF_044", "HMW_203"];
    for entry in baseline["cards"].as_array_mut().unwrap() {
        if entry["setCode"]
            .as_str()
            .is_some_and(|code| focus.contains(&code))
        {
            entry["status"] = json!("unsupported");
        }
    }
    let baseline = Engine::from_catalogue_value(&baseline).unwrap();
    let added: std::collections::BTreeSet<_> = admitted
        .by_code
        .keys()
        .filter(|code| !baseline.by_code.contains_key(*code))
        .map(String::as_str)
        .collect();
    assert_eq!(added, focus.into_iter().collect());
    assert!(baseline
        .by_code
        .keys()
        .all(|code| admitted.by_code.contains_key(code)));
    eprintln!(
        "static_ongoing runtime-admitted delta: {:?}; count {} -> {}",
        added,
        baseline.by_code.len(),
        admitted.by_code.len()
    );
}

// Source expectations: PutIntoPlaySystem checks these conditions immediately
// before entry, with the entering controller. Arena membership excludes the
// entering card. Card::name is an alias for its printed title. Discard counts
// use only that controller's live discard pile, including type+trait filters.
fn ongoing_paid_hand(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {
    s.active=seat;
    let id=add(e,s,seat,code);
    s.player_mut(seat).hand.push(id).unwrap();
    while s.player(seat).resources.len < e.cost(s,id) as u8 {
        let resource=add(e,s,seat,"LOF_254");
        s.player_mut(seat).resources.push(resource).unwrap();
    }
    id
}
#[test]
fn static_ongoing_conditional_ready_round_and_opponent_arena_are_live() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine(); e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            for round in [1,2,3] {
                let mut s=board(&e);s.round=round;
                let source=ongoing_paid_hand(&e,&mut s,seat,"HMW_208");
                let played=e.apply(&s,&e.card_action(&s,source)).unwrap();
                assert!(e.in_arena(&played,source));
                assert_eq!(played.card(source).exhausted,round!=1);
                assert_eq!(e.power(&played,source,false),1);
                assert_eq!(e.power(&played,source,true),2);
            }
            for enemy_ground in [false,true] {
                let mut s=board(&e);
                let _space=unit(&e,&mut s,seat.other(),"TWI_253");
                let _friendly_ground=unit(&e,&mut s,seat,"LOF_256");
                if enemy_ground {unit(&e,&mut s,seat.other(),"LOF_256");}
                let source=ongoing_paid_hand(&e,&mut s,seat,"SEC_170");
                let played=e.apply(&s,&e.card_action(&s,source)).unwrap();
                assert!(e.in_arena(&played,source));
                assert_eq!(played.card(source).exhausted,enemy_ground);
                // Readiness grants no lasting attack or stat modifier.
                assert_eq!(e.power(&played,source,false),5);
                assert_eq!(e.hp(&played,source),5);
            }
        }
    }
}
#[test]
fn static_ongoing_rose_ready_checks_control_and_does_not_count_incoming_unit() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            for friendly_nonunique in [false,true] {
                let mut s=board(&e);
                // Enemy nonunique units never satisfy a friendly predicate.
                unit(&e,&mut s,seat.other(),"LOF_256");
                if friendly_nonunique {unit(&e,&mut s,seat,"LOF_256");}
                let source=ongoing_paid_hand(&e,&mut s,seat,"LAW_223");
                let played=e.apply(&s,&e.card_action(&s,source)).unwrap();
                assert!(e.in_arena(&played,source));
                assert_eq!(played.card(source).exhausted,!friendly_nonunique);
                assert_eq!(e.hp(&played,source),5);
            }
        }
    }
}
#[test]
fn static_ongoing_salacious_title_checks_in_play_and_current_controller() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        // Setup diagnostic only: no executable Jabba source is admitted yet.
        // A vanilla unit supplies the exact printed-title predicate without
        // pretending to implement any unsupported Jabba card ability.
        let setup_def=e.by_code["LOF_256"];
        e.definitions[setup_def as usize].info.title="Jabba the Hutt".into();
        for seat in [Seat::P1,Seat::P2] {
            for location in ["friendly","enemy","hand","discard","absent"] {
                let mut s=board(&e);
                match location {
                    "friendly"=>{unit(&e,&mut s,seat,"LOF_256");},
                    "enemy"=>{unit(&e,&mut s,seat.other(),"LOF_256");},
                    "hand"|"discard"=>{let setup=add(&e,&mut s,seat,"LOF_256");if location=="hand" {s.player_mut(seat).hand.push(setup).unwrap();}else{s.player_mut(seat).discard.push(setup).unwrap();}},
                    _=>{},
                }
                let source=ongoing_paid_hand(&e,&mut s,seat,"LAW_210");
                let played=e.apply(&s,&e.card_action(&s,source)).unwrap();
                assert!(e.in_arena(&played,source));
                assert_eq!(played.card(source).exhausted,location!="friendly");
                assert_eq!(e.power(&played,source,true),2);
            }
        }
    }
}
#[test]
fn static_ongoing_swarm_counts_other_friendly_same_title_and_updates_after_departure() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e);
            let source=unit(&e,&mut s,seat,"JTL_256");
            let original=e.power(&s,source,false);
            let friend=unit(&e,&mut s,seat,"JTL_256");
            let friend2=unit(&e,&mut s,seat,"JTL_256");
            unit(&e,&mut s,seat.other(),"JTL_256");
            unit(&e,&mut s,seat,"TWI_253");
            assert_eq!(e.power(&s,source,false),original+2);
            assert_eq!(e.power(&s,friend,false),original+2);
            assert_eq!(e.power(&s,friend2,false),original+2);
            let hp=e.hp(&s,source);
            e.defeat(&mut s,friend).unwrap();
            assert_eq!(e.power(&s,source,false),original+1);
            assert_eq!(e.hp(&s,source),hp);
            // A full copy/import retains the live count, not a cached stat value.
            let copy=s;assert_eq!(e.power(&copy,source,false),original+1);
            let imported=e.load_state(&e.snapshot(&copy)).unwrap();
            assert_eq!(e.power(&imported,source,false),original+1);
        }
    }
}
#[test]
fn static_ongoing_enoch_live_discard_count_checks_owner_type_trait_and_movement() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e);
            let source=unit(&e,&mut s,seat,"LOF_083");
            let original=e.power(&s,source,false);
            let trooper=unit(&e,&mut s,seat,"SOR_128");
            let other=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).discard.push(other).unwrap();
            let enemy=add(&e,&mut s,seat.other(),"SOR_128");s.player_mut(seat.other()).discard.push(enemy).unwrap();
            let hidden=add(&e,&mut s,seat,"SOR_128");s.player_mut(seat).hand.push(hidden).unwrap();
            assert_eq!(e.power(&s,source,false),original);
            e.defeat(&mut s,trooper).unwrap();
            assert_eq!(e.power(&s,source,false),original+1);
            // Trooper upgrades/events would not be units: the source filters both.
            let setup=e.by_code["LOF_256"];
            e.definitions[setup as usize].traits=vec!["trooper".into()];
            let metadata=json!({"title":"Synthetic Trooper Event","types":["event"]});
            e.definitions[setup as usize].info=serde_json::from_value(metadata).unwrap();
            assert_eq!(e.power(&s,source,false),original+1);
            s.player_mut(seat).discard.remove(trooper);s.player_mut(seat).hand.push(trooper).unwrap();
            assert_eq!(e.power(&s,source,false),original);
        }
    }
}
#[test]
fn static_ongoing_firespray_ambush_base_permission_is_friendly_live_and_respects_sentinel() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e);
            let source=unit(&e,&mut s,seat,"LOF_256");
            let own_base=s.player(seat).base;let enemy_base=s.player(seat.other()).base;
            assert!(!e.attack_targets(&s,source,true).contains(&enemy_base));
            let firespray=unit(&e,&mut s,seat,"HMW_053");
            assert!(e.attack_targets(&s,source,true).contains(&enemy_base));
            assert!(e.attack_targets(&s,firespray,true).contains(&enemy_base));
            let restricted=unit(&e,&mut s,seat,"ASH_034");
            assert!(!e.attack_targets(&s,restricted,true).contains(&enemy_base));
            let unable=unit(&e,&mut s,seat,"LOF_044");
            assert!(e.attack_targets(&s,unable,true).is_empty());
            let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
            assert!(!e.attack_targets(&s,enemy,true).contains(&own_base));
            let sentinel=unit(&e,&mut s,seat.other(),"LOF_044");
            assert_eq!(e.attack_targets(&s,source,true),vec![sentinel]);
            e.defeat(&mut s,sentinel).unwrap();
            // Actual mid-game attack resolves using the Ambush target protocol.
            s.active=seat;s.source=source;s.decision=Decision::AttackTarget;s.ambush=true;
            let fought=e.apply(&s,&e.card_action(&s,enemy_base)).unwrap();
            assert_eq!(fought.card(enemy_base).damage,1);
            e.defeat(&mut s,firespray).unwrap();
            assert!(!e.attack_targets(&s,source,true).contains(&enemy_base));
        }
    }
}
#[test]
fn static_ongoing_new_contracts_reject_scope_capture_and_unknown_mutations() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["HMW_208","LAW_210","LAW_223","SEC_170","JTL_256","LOF_083","HMW_053"] {
        let entry=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap();
        let load=|v|Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap();
        assert!(load(entry.clone()).cards.contains_key(code),"{code}");
        for mutation in ["properties","value","native","mode","source","target","condition","extra-side","metadata"] {
            let mut v=entry.clone();let a=&mut v["abilities"]["card"][0];
            match mutation {
                "properties"=>a["effect"]["properties"]["unmodeled"]=json!(true),
                "value"=>a["effect"]["modifier"]["value"]["unmodeled"]=json!(true),
                "native"=>a["effect"]["native"]=json!({"forged":true}),
                "mode"=>a["effect"]["modifier"]["valueMode"]=json!("unmodeled"),
                "source"=>a["effect"]["properties"]["sourceZoneFilter"]=json!("discard"),
                "target"=>a["effect"]["properties"]["targetZoneFilter"]=json!("any"),
                "condition"=>a["effect"]["properties"]["condition"]=json!({"kind":"const","value":true}),
                "extra-side"=>v["abilities"]["unit"]=v["abilities"]["card"].clone(),
                _=>v["card"]["types"]=json!(["unit","leader"]),
            }
            assert!(load(v).rejected.contains_key(code),"{code}:{mutation}");
        }
    }
}
#[test]
fn static_ongoing_firespray_opens_real_hand_play_ambush_against_an_empty_board() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            for aura in [false,true] {
                let mut s=board(&e);
                if aura {unit(&e,&mut s,seat,"HMW_053");}
                let source=ongoing_paid_hand(&e,&mut s,seat,"LOF_257");
                let entered=e.apply(&s,&e.card_action(&s,source)).unwrap();
                if aura {
                    assert_eq!(entered.decision,Decision::Ambush);
                    assert_eq!(e.legal_actions(&entered),vec![Action::button("Trigger","trigger"),Action::button("Pass","pass")]);
                    let choosing=e.apply(&entered,&Action::button("Trigger","trigger")).unwrap();
                    assert_eq!(choosing.decision,Decision::AttackTarget);
                    let base=s.player(seat.other()).base;
                    assert_eq!(e.legal_actions(&choosing),vec![e.card_action(&choosing,base)]);
                    let fought=e.apply(&choosing,&e.card_action(&choosing,base)).unwrap();
                    assert_eq!(fought.card(base).damage,2);
                    assert!(fought.card(source).exhausted);
                } else {
                    assert_eq!(entered.decision,Decision::Window);
                    assert_eq!(entered.card(entered.player(seat.other()).base).damage,0);
                }
            }
        }
    }
}
#[test]
fn static_ongoing_rejects_nested_callback_capture_and_binding_mutations() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["JTL_256","LOF_083","LAW_223"] {
        let entry=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap();
        fn mutate(v:&mut Value,key:&str,value:Value)->bool {
            if let Some(o)=v.as_object_mut() {
                if o.contains_key(key) {o.insert(key.into(),value);return true;}
                for child in o.values_mut() {if mutate(child,key,value.clone()) {return true;}}
            } else if let Some(a)=v.as_array_mut() {for child in a {if mutate(child,key,value.clone()) {return true;}}}
            false
        }
        for (key,value) in [("capture",json!("registrationSnapshot")),("invokeAt",json!("setup")),("bindingPattern",json!({"name":"x","unsupported":true})),("optionalReceiver",json!(true)),("role",json!("candidate"))] {
            let mut v=entry.clone();
            if !mutate(&mut v["abilities"],key,value) {continue;}
            let c=Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap();
            assert!(c.rejected.contains_key(code),"{code}:{key}");
        }
    }
}
#[test]
fn static_ongoing_conditional_and_live_count_admission_delta_is_complete() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let focus=["HMW_053","HMW_208","JTL_256","LAW_210","LAW_223","LOF_083","SEC_170"];
    let admitted=Engine::from_catalogue_value(&original).unwrap();
    let mut baseline=original.clone();
    for entry in baseline["cards"].as_array_mut().unwrap() {
        if entry["setCode"].as_str().is_some_and(|code|focus.contains(&code)) {entry["status"]=json!("unsupported");}
    }
    let baseline=Engine::from_catalogue_value(&baseline).unwrap();
    let delta:std::collections::BTreeSet<_>=admitted.by_code.keys().filter(|code|!baseline.by_code.contains_key(*code)).map(String::as_str).collect();
    assert_eq!(delta,focus.into_iter().collect());
    assert!(baseline.by_code.keys().all(|code|admitted.by_code.contains_key(code)));
    eprintln!("ongoing conditional/live-count admission delta: {delta:?}; {} -> {}",baseline.by_code.len(),admitted.by_code.len());
}
