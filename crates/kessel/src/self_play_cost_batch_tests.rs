//! Source-derived midgame fixtures for complete live self-play reductions.
//! No reference-engine execution or historical-game replay is claimed.
use super::*;

const CODES: [&str; 7] = [
    "HMW_184", "JTL_204", "SOR_248", "TS26_071", "TWI_098", "TWI_197", "TWI_254",
];
fn raw(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn held(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    s.player_mut(owner).hand.push(id).unwrap();
    id
}
fn row(e: &Engine, s: &mut State, owner: Seat, count: usize) {
    for _ in 0..count {
        let id = add(e, s, owner, "LOF_254");
        s.player_mut(owner).resources.push(id).unwrap();
    }
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn qualify(e: &Engine, s: &mut State, owner: Seat, code: &str) -> u16 {
    match code {
        "HMW_184" => {
            s.initiative = owner;
            1
        }
        "JTL_204" => {
            for _ in 0..3 {
                unit(e, s, owner.other(), "SHD_189", 0);
            }
            3
        }
        "TWI_098" => {
            unit(e, s, owner.other(), "SHD_189", 0);
            unit(e, s, owner.other(), "LOF_254", 0);
            2
        }
        "TWI_197" => {
            for _ in 0..3 {
                unit(e, s, owner, "LOF_254", 0);
            }
            1
        }
        "SOR_248" | "TWI_254" => {
            unit(e, s, owner, "TOKEN_CLONE_TROOPER", 0);
            1
        }
        "TS26_071" => {
            let leader = s.player(owner).leader;
            s.card_mut(leader).deployed = true;
            s.card_mut(leader).epic_used = true;
            e.add_arena(s, leader).unwrap();
            1
        }
        _ => unreachable!(),
    }
}
fn stable_roundtrip(e: &Engine, s: &State) {
    let snapshot = e.snapshot(s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    assert_eq!(s.zobrist(), loaded.zobrist());
}

#[test]
fn self_play_batch_complete_contracts_and_alias_accounting() {
    let e = engine();
    for code in CODES {
        assert!(
            crate::self_play_cost::extract(&raw(code))
                .unwrap()
                .is_some(),
            "{code}"
        );
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        let mut renamed = raw(code);
        renamed["setCode"] = json!("DIAGNOSTIC_COST");
        renamed["card"]["title"] = json!("Diagnostic source");
        assert!(crate::self_play_cost::extract(&renamed).unwrap().is_some());
    }
    assert_eq!(raw("SOR_248")["sourceHash"], raw("TWI_254")["sourceHash"]);
    assert_eq!(
        e.catalogue.as_ref().unwrap().reprint_of["SOR_248"],
        "TWI_254"
    );
    assert_eq!(
        e.definitions[e.by_code["SOR_248"] as usize].self_play_cost,
        e.definitions[e.by_code["TWI_254"] as usize].self_play_cost
    );
    let pool = e.compiled_pool().unwrap();
    assert!(pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["setCode"] == "TWI_254"));
    assert!(!pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["setCode"] == "SOR_248"));
    for code in [
        "SOR_139", "TS26_047", "SEC_122", "JTL_101", "HMW_049", "HMW_125", "HMW_260", "TS26_014",
    ] {
        assert!(
            !e.by_code.contains_key(code),
            "unsupported complete companion {code}"
        );
    }
}

#[test]
fn self_play_batch_closed_registration_cost_and_source_context_fences() {
    for code in CODES {
        let original = raw(code);
        for (key, value) in [
            ("sourceZone", json!("hand")),
            ("targetPlayers", json!("self")),
            ("matchSourceIdentity", json!(false)),
            ("matchPrintedType", json!(false)),
            ("stage", json!("defeatCredits_7")),
            ("matchAbilityCosts", json!(true)),
            ("matchCardEffectResourcePayments", json!(true)),
            ("extra", json!(true)),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["effect"][key] = value;
            assert!(
                crate::self_play_cost::extract(&changed).is_err(),
                "{code}/{key}"
            );
        }
        for (key, value) in [
            ("optional", json!(true)),
            ("then", json!({"effect":{"kind":"noAction"}})),
            ("costs", json!([{"kind":"exhaustSelf"}])),
            ("targetResolver", json!({})),
            ("limit", json!({})),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0][key] = value;
            assert!(
                crate::self_play_cost::extract(&changed).is_err(),
                "{code}/{key}"
            );
        }
        for (key, value) in [
            ("baseClass", json!("LeaderUnitCard")),
            ("stateWatchers", json!([{}])),
            ("privatePrograms", json!([{}])),
            ("deploymentOverride", json!({})),
            ("attachCondition", json!({})),
            ("piloting", json!({})),
        ] {
            let mut changed = original.clone();
            changed[key] = value;
            assert!(
                crate::self_play_cost::extract(&changed).is_err(),
                "{code}/{key}"
            );
        }
        for (key, value) in [
            ("types", json!(["unit", "leader"])),
            ("keywords", json!(["shielded"])),
            ("text", json!("Costs less.")),
            ("upgradeHp", json!(1)),
        ] {
            let mut changed = original.clone();
            changed["card"][key] = value;
            assert!(
                crate::self_play_cost::extract(&changed).is_err(),
                "{code}/{key}"
            );
        }
        let mut changed = original.clone();
        changed["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(json!({"trigger":"whenPlayed","costs":[],"effect":{"kind":"noAction"}}));
        assert!(crate::self_play_cost::extract(&changed).is_err());
        let mut changed = original;
        changed["abilities"]["card"][0]["condition"] = json!({"kind":"const","value":true});
        assert!(crate::self_play_cost::extract(&changed).is_err());
    }
}

#[test]
fn self_play_batch_live_thresholds_use_source_controller_and_physical_arenas() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        for code in CODES {
            let mut s = board(&e, owner);
            s.initiative = owner.other();
            let source = held(&e, &mut s, owner, code);
            let full = e.cost(&s, source);
            // The active prompt actor may differ from the card's controller.
            s.active = owner.other();
            let discount = qualify(&e, &mut s, owner, code);
            assert_eq!(e.cost(&s, source), full.saturating_sub(discount), "{code}");
            s.initiative = owner.other();
            if code == "HMW_184" {
                assert_eq!(e.cost(&s, source), full);
                continue;
            }
            // Base damage, resources and the source in hand cannot replace an arena prerequisite.
            let qualifying = if matches!(code, "JTL_204" | "TWI_098") {
                owner.other()
            } else {
                owner
            };
            let ids = s
                .player(qualifying)
                .ground
                .as_slice()
                .iter()
                .chain(s.player(qualifying).space.as_slice())
                .copied()
                .collect::<Vec<_>>();
            for id in ids {
                s.player_mut(qualifying).ground.remove(id);
                s.player_mut(qualifying).space.remove(id);
                s.player_mut(qualifying).resources.push(id).unwrap();
            }
            assert_eq!(
                e.cost(&s, source),
                full,
                "{code}: base-zone/resource cards do not count"
            );
        }
        let mut s = board(&e, owner);
        let source = held(&e, &mut s, owner, "JTL_204");
        let full = e.cost(&s, source);
        for _ in 0..3 {
            unit(&e, &mut s, owner.other(), "LOF_254", 0);
        }
        for _ in 0..2 {
            unit(&e, &mut s, owner.other(), "SHD_189", 0);
        }
        assert_eq!(e.cost(&s, source), full);
        unit(&e, &mut s, owner.other(), "SHD_189", 0);
        assert_eq!(e.cost(&s, source), full - 3);
        let mut s = board(&e, owner);
        let source = held(&e, &mut s, owner, "TWI_197");
        let full = e.cost(&s, source);
        for _ in 0..2 {
            unit(&e, &mut s, owner, "LOF_254", 0);
        }
        s.player_mut(owner).hand.remove(source);
        e.add_arena(&mut s, source).unwrap();
        assert_eq!(
            e.cost(&s, source),
            full,
            "source cannot be the third other unit"
        );
        unit(&e, &mut s, owner, "SHD_189", 0);
        assert_eq!(e.cost(&s, source), full - 1);
    }
}

#[test]
fn self_play_batch_actual_unit_payment_affordability_keywords_and_no_registration_effect() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for owner in [Seat::P1, Seat::P2] {
            for code in [
                "HMW_184", "JTL_204", "TWI_098", "TWI_197", "SOR_248", "TWI_254",
            ] {
                let mut s = board(&e, owner);
                let source = held(&e, &mut s, owner, code);
                qualify(&e, &mut s, owner, code);
                let cost = e.cost(&s, source);
                row(&e, &mut s, owner, usize::from(cost));
                let next = s.next_id;
                let mut short = s;
                let resource = short.player(owner).resources.as_slice()[0];
                short.card_mut(resource).exhausted = true;
                assert!(
                    !e.legal_actions(&short)
                        .contains(&e.card_action(&short, source)),
                    "{code}"
                );
                let done = click(&e, &s, source);
                assert!(e.in_arena(&done, source));
                assert!(done.card(source).exhausted);
                assert!(!done.player(owner).hand.contains(source));
                assert_eq!(e.ready_resources(&done, owner), 0);
                assert_eq!(done.next_id, next);
                assert!(e
                    .available_programs(&done, source, "costAdjustment")
                    .is_empty());
                match code {
                    "JTL_204" => assert_eq!(done.decision, Decision::Ambush),
                    "TWI_098" => assert!(e.has_keyword(&done, source, "sentinel")),
                    "SOR_248" | "TWI_254" => assert_eq!(e.keyword_amount(&done, source, "raid"), 1),
                    _ => {}
                }
                if done.decision == Decision::Window {
                    stable_roundtrip(&e, &done);
                }
            }
        }
    }
}

#[test]
fn self_play_batch_enemy_count_includes_deployed_leader_and_floors_aspect_total() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        let mut s = board(&e, owner);
        let source = held(&e, &mut s, owner, "TWI_098");
        let full = e.cost(&s, source);
        let enemy = owner.other();
        let leader = s.player(enemy).leader;
        assert!(full > 11, "setup includes aspect penalties");
        unit(&e, &mut s, enemy, "LOF_254", 0);
        unit(&e, &mut s, enemy, "SHD_189", 0);
        assert_eq!(e.cost(&s, source), full - 2);
        s.card_mut(leader).deployed = true;
        s.card_mut(leader).epic_used = true;
        e.add_arena(&mut s, leader).unwrap();
        assert_eq!(e.cost(&s, source), full - 3);
        for _ in 0..full {
            unit(&e, &mut s, enemy, "LOF_254", 0);
        }
        assert_eq!(e.cost(&s, source), 0);
        assert_eq!(e.def(&s, source).info.cost(), 11);
    }
}

#[test]
fn self_play_batch_take_action_preserves_target_damage_and_leader_cost_body() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for owner in [Seat::P1, Seat::P2] {
            let mut s = board(&e, owner);
            let source = held(&e, &mut s, owner, "TS26_071");
            let full = e.cost(&s, source);
            let leader = s.player(owner.other()).leader;
            s.card_mut(leader).deployed = true;
            s.card_mut(leader).epic_used = true;
            e.add_arena(&mut s, leader).unwrap();
            assert_eq!(e.cost(&s, source), full, "enemy leader never reduces cost");
            qualify(&e, &mut s, owner, "TS26_071");
            assert_eq!(e.cost(&s, source), full - 1);
            let target = unit(&e, &mut s, owner.other(), "LOF_256", 0);
            row(&e, &mut s, owner, usize::from(full - 1));
            let select = click(&e, &s, source);
            assert_eq!(select.decision, Decision::AbilityTarget);
            let cancel = e
                .legal_actions(&select)
                .into_iter()
                .find(|a| a.label == "Cancel")
                .unwrap();
            let cancelled = e.apply(&select, &cancel).unwrap();
            assert!(cancelled.player(owner).hand.contains(source));
            assert_eq!(e.ready_resources(&cancelled, owner), usize::from(full - 1));
            assert_eq!(cancelled.card(target).damage, 0);
            let done = click(&e, &select, target);
            assert_eq!(done.card(target).damage, 3);
            assert!(done.player(owner).discard.contains(source));
            assert_eq!(e.ready_resources(&done, owner), 0);
            stable_roundtrip(&e, &done);
        }
    }
}

#[test]
fn self_play_batch_compiled_pool_roundtrip_retains_all_cost_rules_and_event_bodies() {
    let e = engine();
    let pool = e.compiled_pool().unwrap();
    let source = catalogue();
    let loaded = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    for owner in [Seat::P1, Seat::P2] {
        for code in CODES {
            let mut s = board(&loaded, owner);
            s.initiative = owner.other();
            let id = held(&loaded, &mut s, owner, code);
            let full = loaded.cost(&s, id);
            let reduction = qualify(&loaded, &mut s, owner, code);
            assert_eq!(loaded.cost(&s, id), full - reduction, "{code}");
            if code == "TS26_071" {
                assert_eq!(loaded.available_programs(&s, id, "event").len(), 1);
            }
        }
    }
}

#[test]
fn self_play_batch_diagnostic_live_trait_gain_and_loss_changes_discount() {
    // Synthetic attached constant registrations use the existing trait executor.
    // This is not a printed-card witness for these two setup upgrades.
    let mut e = engine();
    for owner in [Seat::P1, Seat::P2] {
        let mut s = board(&e, owner);
        let source = held(&e, &mut s, owner, "TWI_254");
        let candidate = unit(&e, &mut s, owner, "LOF_254", 0);
        let full = e.cost(&s, source);
        let mut attached = vec![];
        for (kind, code, reduction) in [("gainTrait", "IC27_052", 1), ("loseTrait", "IC27_093", 0)]
        {
            let upgrade = add(&e, &mut s, owner, code);
            e.attach(&mut s, upgrade, candidate);
            let ability = serde_json::from_value(json!({"trigger":"constant","title":"DIAGNOSTIC live Trooper","attachedContext":"upgrade","effect":{"kind":kind,"target":"parent","trait":"trooper","duration":"persistent"}})).unwrap();
            let program = crate::ability_ir::Program::compile(&ability).unwrap();
            let catalogue = e.catalogue.as_mut().unwrap();
            let index = catalogue.programs.len() as u16;
            catalogue.programs.push(program);
            e.definitions[s.card(upgrade).def as usize].programs.card = vec![index];
            assert_eq!(e.cost(&s, source), full - reduction);
            attached.push(upgrade);
        }
        e.defeat(&mut s, attached[1]).unwrap();
        assert_eq!(e.cost(&s, source), full - 1);
        e.defeat(&mut s, attached[0]).unwrap();
        assert_eq!(e.cost(&s, source), full);
    }
}
