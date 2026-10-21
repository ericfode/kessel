//! Native synthetic transition checks. Pinned-engine recordings remain separate evidence.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    assert!(
        e.by_code.contains_key("ASH_188"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_188")
    );
    e.set_rules_backend(backend).unwrap();
    e
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        owner,
        def: *e.by_code.get(code).unwrap(),
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.add_arena(s, id).unwrap();
    id
}
fn hand(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    s.player_mut(owner).hand.push(id).unwrap();
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
    s.phase_history.damage_exact = true;
    for owner in [Seat::P1, Seat::P2] {
        s.player_mut(owner).base = add(e, &mut s, owner, "SOR_023");
        s.player_mut(owner).leader = add(e, &mut s, owner, "SOR_014");
        s.player_mut(owner).force_id = s.next_id;
        s.next_id += 1;
        for _ in 0..12 {
            let id = add(e, &mut s, owner, "LOF_256");
            s.player_mut(owner).resources.push(id).unwrap();
        }
        for _ in 0..4 {
            let id = add(e, &mut s, owner, "LOF_256");
            s.player_mut(owner).deck.push(id).unwrap();
        }
    }
    e.load_state(&e.snapshot(&s)).unwrap()
}
fn fixed(e: &Engine, s: &State) {
    let v = e.snapshot(s);
    let t = e.load_state(&v).unwrap();
    assert_eq!(t, *s);
    assert_eq!(t.zobrist(), s.zobrist());
    assert_eq!(e.snapshot(&t), v);
    assert_eq!(e.legal_actions(&t), e.legal_actions(s));
    let copy = *s;
    assert_eq!(copy.zobrist(), s.zobrist());
}
fn marked(s: &State, id: CardId) -> bool {
    s.phase_history.damaged[id as usize / 64] & (1u64 << (id as usize % 64)) != 0
}
fn click(e: &Engine, s: &mut State, id: CardId) {
    let a = e.card_action(s, id);
    assert!(e.legal_actions(s).contains(&a), "{:?}", e.legal_actions(s));
    *s = e.apply(s, &a).unwrap();
}
#[test]
fn damaged_target_inline_size_measurement() {
    assert_eq!(std::mem::size_of::<State>(), 38_344);
    assert_eq!(std::mem::size_of::<Card>(), 40);
    eprintln!(
        "State={} PhaseHistory={} Card={}; added32 bytes physical history, no Card growth",
        std::mem::size_of::<State>(),
        std::mem::size_of::<crate::model::PhaseHistory>(),
        std::mem::size_of::<Card>()
    );
}
#[test]
fn damaged_target_actual_source_is_closed_and_raw_query_is_closed() {
    let p = crate::damaged_target_history_contract::profile();
    assert_eq!(
        crate::damaged_target_history_contract::prepare(&p),
        Ok(true)
    );
    for key in [
        "setCode",
        "sourceHash",
        "sourcePath",
        "className",
        "baseClass",
        "id",
        "card",
        "stateWatchers",
        "abilities",
        "diagnostics",
        "keywordValues",
    ] {
        let mut bad = p.clone();
        bad[key] = json!("altered");
        assert!(
            crate::damaged_target_history_contract::prepare(&bad).is_err(),
            "{key}"
        );
    }
    let mut bad = p.clone();
    bad["unknown"] = json!(true);
    assert!(crate::damaged_target_history_contract::prepare(&bad).is_err());
    let mut q = p["abilities"]["card"][0]["targetResolver"]["predicate"]["right"].clone();
    q.as_object_mut().unwrap().remove("kind");
    let parsed: crate::phase_history::Query = serde_json::from_value(q.clone()).unwrap();
    assert_eq!(
        parsed.kind(),
        Ok(crate::phase_history::QueryKind::DamagedCard)
    );
    for key in ["binding", "operation", "watcher", "args", "readAt"] {
        let mut bad = q.clone();
        bad[key] = json!("altered");
        assert!(
            serde_json::from_value::<crate::phase_history::Query>(bad)
                .map_or(true, |q| q.kind().is_err()),
            "{key}"
        );
    }
}
#[test]
fn damaged_target_ready_both_controllers_and_modes_after_heal() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for owner in [Seat::P1, Seat::P2] {
            for target_owner in [owner, owner.other()] {
                let e = engine(backend);
                let mut s = board(&e, owner);
                let event = hand(&e, &mut s, owner, "ASH_188");
                let target = unit(&e, &mut s, target_owner, "LOF_256");
                s.card_mut(target).exhausted = true;
                e.damage(&mut s, target, 1);
                assert!(marked(&s, target));
                assert_eq!(e.heal_card(&mut s, target, 1), 1);
                assert_eq!(s.card(target).damage, 0);
                fixed(&e, &s);
                click(&e, &mut s, event);
                click(&e, &mut s, target);
                assert!(!s.card(target).exhausted);
                assert!(marked(&s, target));
                assert!(s.player(owner).discard.contains(event));
                fixed(&e, &s);
            }
        }
    }
}
#[test]
fn damaged_target_shield_zero_event_and_physical_reentry() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        let mut s = board(&e, Seat::P1);
        let event = hand(&e, &mut s, Seat::P1, "ASH_188");
        let target = unit(&e, &mut s, Seat::P2, "LOF_256");
        e.damage(&mut s, target, 0);
        assert!(!marked(&s, target));
        let shield = add(&e, &mut s, Seat::P2, "TOKEN_SHIELD");
        s.card_mut(target).shield = shield;
        e.attach(&mut s, shield, target);
        e.damage(&mut s, target, 1);
        assert!(!marked(&s, target));
        e.damage(&mut s, target, 1);
        assert!(marked(&s, target));
        let other = unit(&e, &mut s, Seat::P2, "LOF_256");
        assert!(!marked(&s, other));
        e.leave_play(&mut s, target).unwrap();
        s.player_mut(Seat::P2).discard.remove(target);
        s.player_mut(Seat::P2).hand.push(target).unwrap();
        assert!(marked(&s, target));
        s.player_mut(Seat::P2).hand.remove(target);
        e.add_arena(&mut s, target).unwrap();
        s.card_mut(target).exhausted = true;
        click(&e, &mut s, event);
        click(&e, &mut s, target);
        assert!(!s.card(target).exhausted);
        let base = s.player(Seat::P1).base;
        let listeners = e.blade_base_damage_listeners(&s).unwrap();
        e.capture_blade_base_damage_event(&mut s, &listeners, base, true)
            .unwrap();
        assert!(
            marked(&s, base),
            "actual zero-valued resolved event retained"
        );
        let prev = s.zobrist();
        s.phase_history.reset();
        assert!(s.phase_history.damage_exact);
        assert!(!marked(&s, target));
        assert!(!marked(&s, base));
        assert_ne!(prev, s.zobrist());
    }
}
#[test]
fn damaged_target_wire_strict_presence_and_legacy_hash() {
    let e = engine(RulesBackend::Auto);
    let mut s = board(&e, Seat::P1);
    s.phase_history.damage_exact = false;
    let old = e.snapshot(&s);
    assert!(old.get("damagedTargetHistory").is_none());
    let h = s.zobrist();
    let base = s.player(Seat::P1).base;
    e.record_damaged_target(&mut s, base);
    assert_eq!(h, s.zobrist());
    let t = e.load_state(&old).unwrap();
    assert_eq!(t.zobrist(), h);
    let event = hand(&e, &mut s, Seat::P1, "ASH_188");
    assert!(e.state_needs_damaged_target_history(&s));
    let missing = e.snapshot(&s);
    assert!(e
        .load_state(&missing)
        .unwrap_err()
        .contains("damagedTargetHistory"));
    s.phase_history.damage_exact = true;
    fixed(&e, &s);
    let good = e.snapshot(&s);
    for wire in [
        json!({"version":2,"cardIds":[]}),
        json!({"version":1,"cardIds":[0]}),
        json!({"version":1,"cardIds":[1,1]}),
        json!({"version":1,"cardIds":[2,1]}),
        json!({"version":1,"cardIds":[s.next_id]}),
        json!({"version":1,"cardIds":[],"extra":true}),
    ] {
        let mut bad = good.clone();
        bad["damagedTargetHistory"] = wire;
        assert!(e.load_state(&bad).is_err());
    }
    s.player_mut(Seat::P1).hand.remove(event);
    s.player_mut(Seat::P1).deck.push(event).unwrap();
    assert!(e.state_needs_damaged_target_history(&s));
}
fn button(e: &Engine, s: &mut State, label: &str) {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)));
    *s = e.apply(s, &a).unwrap();
}
#[test]
fn damaged_target_ineffective_ready_restricted_and_cancel_protocol() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for scenario in ["none", "ready", "restricted"] {
                let mut s = board(&e, owner);
                let event = hand(&e, &mut s, owner, "ASH_188");
                if scenario != "none" {
                    let target = unit(&e, &mut s, owner.other(), "LOF_256");
                    e.damage(&mut s, target, 1);
                    if scenario == "restricted" {
                        s.card_mut(target).exhausted = true;
                        let upgrade = add(&e, &mut s, owner, "SHD_193");
                        e.attach(&mut s, upgrade, target);
                        assert!(e.card_restricted(
                            &s,
                            target,
                            crate::ability_ir::RestrictionVerb::Ready
                        ));
                    }
                }
                let before = s;
                let ready = e.ready_resources(&s, owner);
                let cost = e.cost(&s, event);
                click(&e, &mut s, event);
                if scenario == "ready" {
                    assert_eq!(s.decision, Decision::Window);
                    assert!(s.player(owner).discard.contains(event));
                    continue;
                }
                assert_eq!(s.decision, Decision::AbilityNoEffect, "{scenario}");
                assert_eq!(
                    e.legal_actions(&s),
                    vec![
                        Action::button("Play anyway", 0),
                        Action::button("Cancel", 1)
                    ]
                );
                let prompt = s;
                button(&e, &mut s, "Cancel");
                same(&s, &before);
                s = prompt;
                button(&e, &mut s, "Play anyway");
                assert!(s.player(owner).discard.contains(event));
                assert_eq!(e.ready_resources(&s, owner), ready - usize::from(cost));
                fixed(&e, &s);
            }
        }
    }
}
#[test]
fn damaged_target_space_leader_defeat_reentry_and_real_phase_boundaries() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            for kind in ["space", "leader", "defeat-reentry"] {
                let mut s = board(&e, owner);
                let event = hand(&e, &mut s, owner, "ASH_188");
                let target = if kind == "leader" {
                    let id = s.player(owner.other()).leader;
                    s.card_mut(id).deployed = true;
                    s.card_mut(id).epic_used = true;
                    e.add_arena(&mut s, id).unwrap();
                    id
                } else {
                    unit(
                        &e,
                        &mut s,
                        owner.other(),
                        if kind == "space" {
                            "HMW_178"
                        } else {
                            "LOF_256"
                        },
                    )
                };
                s.card_mut(target).exhausted = true;
                e.damage(&mut s, target, 1);
                if kind == "defeat-reentry" {
                    e.defeat(&mut s, target).unwrap();
                    assert!(marked(&s, target));
                    s.player_mut(owner.other()).discard.remove(target);
                    e.add_arena(&mut s, target).unwrap();
                    s.card_mut(target).exhausted = true;
                }
                fixed(&e, &s);
                click(&e, &mut s, event);
                click(&e, &mut s, target);
                assert!(!s.card(target).exhausted);
                assert!(marked(&s, target));
                s.players[0].out = true;
                s.players[1].out = true;
                e.advance(&mut s).unwrap();
                assert!(!marked(&s, target));
                assert!(s.phase_history.damage_exact);
                let base = s.player(owner).base;
                e.record_damaged_target(&mut s, base);
                assert!(marked(&s, base));
                e.end_regroup(&mut s).unwrap();
                assert!(!marked(&s, base));
                assert!(s.phase_history.damage_exact);
            }
        }
    }
}
#[test]
fn damaged_target_independent_wire_does_not_require_unrelated_phase_watchers() {
    let mut e = engine(RulesBackend::Auto);
    let mut s = board(&e, Seat::P1);
    let event = hand(&e, &mut s, Seat::P1, "ASH_188");
    let target = unit(&e, &mut s, Seat::P2, "LOF_256");
    e.damage(&mut s, target, 1);
    s.card_mut(target).exhausted = true;
    e.catalogue.as_mut().unwrap().metadata.retain(|code, _| {
        [
            "ASH_188",
            "SOR_023",
            "SOR_014",
            "LOF_256",
            "TOKEN_SHIELD",
            "TOKEN_THE_FORCE",
            "TOKEN_EXPERIENCE",
        ]
        .contains(&code.as_str())
    });
    assert!(!e.needs_phase_history());
    s.credit_state_exact = false;
    let mut v = e.snapshot(&s);
    v.as_object_mut().unwrap().remove("phaseHistory");
    let mut t = e.load_state(&v).unwrap();
    assert!(!t.phase_history.exact);
    assert!(t.phase_history.damage_exact);
    fixed(&e, &t);
    click(&e, &mut t, event);
    click(&e, &mut t, target);
    assert!(!t.card(target).exhausted);
}
#[test]
fn damaged_target_marked_ready_is_selectable_beside_effective_target_and_targetcancel_refunds() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for owner in [Seat::P1, Seat::P2] {
            let mut s = board(&e, owner);
            let event = hand(&e, &mut s, owner, "ASH_188");
            let ready = unit(&e, &mut s, owner.other(), "LOF_256");
            let exhausted = unit(&e, &mut s, owner, "LOF_256");
            s.card_mut(exhausted).exhausted = true;
            e.damage(&mut s, ready, 1);
            e.damage(&mut s, exhausted, 1);
            let before = s;
            click(&e, &mut s, event);
            let acts = e.legal_actions(&s);
            assert!(acts.contains(&e.card_action(&s, ready)));
            assert!(acts.contains(&e.card_action(&s, exhausted)));
            let prompt = s;
            button(&e, &mut s, "Cancel");
            same(&s, &before);
            s = prompt;
            click(&e, &mut s, ready);
            assert!(s.player(owner).discard.contains(event));
            assert!(!s.card(ready).exhausted);
            assert!(s.card(exhausted).exhausted);
            fixed(&e, &s);
        }
    }
}
#[test]
fn damaged_target_constructor_preserves_all_previous_resolved_bindings_and_closed_negatives() {
    let full = catalogue();
    let after = crate::ability_ir::Catalogue::load(&full).unwrap();
    let mut old = full.clone();
    old["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["setCode"] != "ASH_188");
    let before = crate::ability_ir::Catalogue::load(&old).unwrap();
    assert!(after.cards.contains_key("ASH_188"));
    for (code, bindings) in &before.cards {
        let new = &after.cards[code];
        let resolve = |c: &crate::ability_ir::Catalogue, p: &crate::ability_ir::CardPrograms| json!({"card":p.card.iter().map(|i|serde_json::to_value(&c.programs[*i as usize]).unwrap()).collect::<Vec<_>>(),"unit":p.unit.iter().map(|i|serde_json::to_value(&c.programs[*i as usize]).unwrap()).collect::<Vec<_>>(),"leader":p.leader.iter().map(|i|serde_json::to_value(&c.programs[*i as usize]).unwrap()).collect::<Vec<_>>(),"attachCondition":p.attach_condition});
        assert_eq!(resolve(&before, bindings), resolve(&after, new), "{code}");
    }
    let gained = after
        .cards
        .keys()
        .filter(|code| !before.cards.contains_key(*code))
        .collect::<Vec<_>>();
    assert_eq!(gained, vec![&"ASH_188".to_owned()]);
    for mutation in ["extra", "type", "keyword", "helper", "alias", "shape"] {
        let mut bad = full.clone();
        let entry = bad["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["setCode"] == "ASH_188")
            .unwrap();
        match mutation {
            "extra" => entry["unknown"] = json!(true),
            "type" => entry["card"]["types"] = json!(["unit"]),
            "keyword" => entry["keywordValues"] = json!({"hidden":1}),
            "helper" => entry["privatePrograms"] = json!([{"kind":"ready","target":"chosen"}]),
            "alias" => entry["setCode"] = json!("TST_188"),
            _ => {
                entry["abilities"]["card"][0]["targetResolver"]["predicate"]["right"]["args"] =
                    json!([{ "kind":"ref","path":["source"]}])
            }
        }
        let parsed = crate::ability_ir::Catalogue::load(&bad).unwrap();
        let code = if mutation == "alias" {
            "TST_188"
        } else {
            "ASH_188"
        };
        assert!(!parsed.cards.contains_key(code), "{mutation}");
        assert!(
            parsed.rejected[code].contains("damaged-target history"),
            "{mutation}: {}",
            parsed.rejected[code]
        );
    }
}

fn same(a: &State, b: &State) {
    let aa = format!("{a:#?}");
    let bb = format!("{b:#?}");
    let differences = aa
        .lines()
        .zip(bb.lines())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(12)
        .collect::<Vec<_>>();
    assert!(a == b, "first different debug rows: {differences:?}");
}

#[test]
fn damaged_target_public_selected_pool_complete_raw_metadata() {
    let mut c = catalogue();
    c["cards"].as_array_mut().unwrap().retain(|v| {
        v["setCode"].as_str().is_some_and(|s| {
            s.starts_with("TOKEN_")
                || ["ASH_188", "SOR_014", "SOR_023", "LOF_256"].contains(&s)
        })
    });
    let pool = Engine::from_catalogue_value(&c)
        .unwrap()
        .compiled_pool()
        .unwrap();
    let cb = serde_json::to_vec(&c).unwrap();
    assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &cb).is_ok());
    for code in ["ASH_188"] {
        for mutation in [
            "unknown",
            "rules",
            "id",
            "setId",
            "setCodes",
            "className",
            "helper",
            "programme",
            "ast",
            "missingRules",
            "missingId",
            "leader",
            "token",
            "upgrade",
            "mixed",
            "alias",
        ] {
            let mut bad = pool.clone();
            let row = bad["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|r| r["setCode"] == code)
                .unwrap();
            match mutation {
                "unknown" => row["unknown"] = json!(true),
                "rules" => row["rules"] = json!("poisoned"),
                "id" => row["id"] = json!("unknown physical identity"),
                "setId" => row["setId"] = json!({"set":"SOR","number":1}),
                "setCodes" => row["setCodes"] = json!([]),
                "className" => row["className"] = json!("unknown"),
                "helper" => row["privatePrograms"] = json!([]),
                "programme" => {
                    row["privatePrograms"] = json!([{ "instructions":[{"kind":"ready","target":"chosen"}],"executable":"external"}])
                }
                "ast" => row["abilities"] = json!({"card":[{"kind":"ready","target":"chosen"}]}),
                "missingRules" => {
                    row.as_object_mut().unwrap().remove("rules");
                }
                "missingId" => {
                    row.as_object_mut().unwrap().remove("id");
                }
                "leader" => row["types"] = json!(["leader"]),
                "token" => row["types"] = json!(["token"]),
                "upgrade" => row["types"] = json!(["upgrade"]),
                "mixed" => row["types"] = json!(["event", "unit"]),
                _ => row["setCode"] = json!("TST_188"),
            }
            assert!(
                Engine::from_compiled_pool_bytes(&serde_json::to_vec(&bad).unwrap(), &cb).is_err(),
                "{code}/{mutation}"
            );
        }
        for mutation in ["sourceHash", "helper", "ast", "alias"] {
            let mut bad = c.clone();
            let source = bad["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["setCode"] == code)
                .unwrap();
            match mutation {
                "sourceHash" => source["sourceHash"] = json!("0".repeat(64)),
                "helper" => source["privatePrograms"] = json!([]),
                "ast" => source["abilities"]["card"][0]["effect"] = json!({"kind":"noAction"}),
                _ => source["setCode"] = json!("TST_188"),
            }
            assert!(
                Engine::from_compiled_pool_bytes(
                    &serde_json::to_vec(&pool).unwrap(),
                    &serde_json::to_vec(&bad).unwrap()
                )
                .is_err(),
                "{code}/source/{mutation}"
            );
        }
    }
    let mut duplicate = pool.clone();
    let row = duplicate["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["setCode"] == "ASH_188")
        .unwrap()
        .clone();
    duplicate["cards"].as_array_mut().unwrap().push(row);
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&duplicate).unwrap(), &cb).is_err()
    );
    let mut legacy = pool.clone();
    legacy["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["setCode"] == "LOF_256")
        .unwrap()["unknown"] = json!(true);
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&legacy).unwrap(), &cb).is_ok(),
        "unrelated old-family metadata policy is unchanged"
    );
    // The whole unfiltered pool retains every pre-existing family's ordinary
    // metadata policy while the new closed family passes the actual API.
    let full = catalogue();
    let engine = Engine::from_catalogue_value(&full).unwrap();
    let pool = engine.compiled_pool().unwrap();
    let loaded = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&full).unwrap(),
    )
    .unwrap();
    assert_eq!(loaded.compiled_pool().unwrap(), pool);
}
