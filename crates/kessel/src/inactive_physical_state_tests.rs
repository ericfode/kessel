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
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn drain(e: &Engine, s: &mut State) {
    for _ in 0..100 {
        if s.is_action_window() {
            return;
        }
        let a = e
            .legal_actions(s)
            .into_iter()
            .next()
            .expect("mandatory choice");
        *s = e.apply(s, &a).unwrap();
    }
    panic!("pending resolution");
}
fn paid_dead(e: &Engine, seat: Seat, outcast: bool) -> (State, CardId) {
    let mut s = board(e, seat);
    unit(e, &mut s, seat.other(), "SHD_037");
    if outcast {
        unit(e, &mut s, seat, "ASH_041");
    }
    let event = add(e, &mut s, seat, "ASH_091");
    s.player_mut(seat).hand.push(event).unwrap();
    for _ in 0..10 {
        let r = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    let initial = e.snapshot(&s);
    assert_eq!(initial["inactivePhaseKeywordStateVersion"], 1);
    assert_eq!(initial["inactiveAllocatedCards"], json!([]));
    let imported = e.load_state(&initial).unwrap();
    assert_eq!(imported.zobrist(), s.zobrist());
    let action = e.card_action(&s, event);
    assert!(e.legal_actions(&s).contains(&action));
    s = e.apply(&s, &action).unwrap();
    drain(e, &mut s);
    let id = (1..s.next_id)
        .find(|id| s.card(*id).present && e.def(&s, *id).info.internal_name == "mandalorian")
        .unwrap();
    assert!(!e.in_arena(&s, id));
    assert!(e.has_keyword(&s, id, "sentinel"));
    (s, id)
}
#[test]
fn inactive_physical_keyword_exact_roundtrip_copy_hash_and_expiry() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (mut s, id) = paid_dead(&e, seat, false);
        let copy = s;
        assert_eq!(s, copy);
        let v = e.snapshot(&s);
        assert_eq!(v["inactiveAllocatedCards"].as_array().unwrap().len(), 1);
        assert_eq!(v["inactiveAllocatedCards"][0]["id"], format!("c{}", id - 1));
        assert_eq!(
            v["inactiveAllocatedCards"][0]["phaseKeywords"],
            json!({"sentinel":1})
        );
        let loaded = e.load_state(&v).unwrap();
        assert_eq!(loaded.card(id).def, s.card(id).def);
        assert_eq!(loaded.card(id).owner, seat);
        assert_eq!(
            loaded.card(id).phase_keyword_mask(),
            s.card(id).phase_keyword_mask()
        );
        assert_eq!(loaded.zobrist(), s.zobrist());
        assert_eq!(e.snapshot(&loaded), v);
        assert_eq!(loaded.next_id, s.next_id);
        s.players[0].out = true;
        s.players[1].out = true;
        e.advance(&mut s).unwrap();
        drain(&e, &mut s);
        assert!(!e.has_keyword(&s, id, "sentinel"));
        let expired = e.snapshot(&s);
        assert_eq!(
            expired["inactiveAllocatedCards"][0]["id"],
            format!("c{}", id - 1)
        );
        assert_eq!(
            expired["inactiveAllocatedCards"][0]["phaseKeywords"],
            json!({})
        );
        let imported = e.load_state(&expired).unwrap();
        assert_eq!(imported.zobrist(), s.zobrist());
        assert_eq!(e.snapshot(&imported), expired);
    }
}
#[test]
fn inactive_physical_keyword_versions_gaps_and_conflicts_reject() {
    let e = engine();
    let (s, _) = paid_dead(&e, Seat::P1, false);
    let v = e.snapshot(&s);
    for field in ["inactivePhaseKeywordStateVersion", "inactiveAllocatedCards"] {
        let mut bad = v.clone();
        bad.as_object_mut().unwrap().remove(field);
        assert!(e.load_state(&bad).is_err(), "{field}");
    }
    let mut missing = v.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("inactivePhaseKeywordStateVersion");
    missing
        .as_object_mut()
        .unwrap()
        .remove("inactiveAllocatedCards");
    assert!(e.load_state(&missing).is_err());
    for value in [json!(0), json!(2), json!(true), json!("1")] {
        let mut bad = v.clone();
        bad["inactivePhaseKeywordStateVersion"] = value;
        assert!(e.load_state(&bad).is_err());
    }
    let mut no_row = v.clone();
    no_row["inactiveAllocatedCards"] = json!([]);
    assert!(e.load_state(&no_row).is_err());
    let mut duplicate = v.clone();
    let row = duplicate["inactiveAllocatedCards"][0].clone();
    duplicate["inactiveAllocatedCards"]
        .as_array_mut()
        .unwrap()
        .push(row);
    assert!(e.load_state(&duplicate).is_err());
    for field in [
        "id",
        "name",
        "setCode",
        "owner",
        "controller",
        "phaseKeywords",
    ] {
        let mut bad = v.clone();
        bad["inactiveAllocatedCards"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(e.load_state(&bad).is_err(), "{field}");
    }
    for (field, value) in [
        ("id", json!("c00")),
        ("id", json!(format!("c{}", s.next_id - 1))),
        ("id", v["players"]["p1"]["forceToken"]["id"].clone()),
        ("id", v["players"]["p1"]["base"]["id"].clone()),
        ("name", json!("invented")),
        ("setCode", json!("TOKEN_THE_FORCE")),
        ("setCode", json!("SOR_095")),
        ("owner", json!("p2")),
        ("controller", json!("p2")),
        ("phaseKeywords", json!({"sentinel":2})),
        ("phaseKeywords", json!({"ambush":1})),
        ("phaseKeywords", json!(null)),
    ] {
        let mut bad = v.clone();
        bad["inactiveAllocatedCards"][0][field] = value;
        assert!(e.load_state(&bad).is_err(), "{field}");
    }
    let mut owner = v.clone();
    owner["inactiveAllocatedCards"][0]["owner"] = json!("p2");
    owner["inactiveAllocatedCards"][0]["controller"] = json!("p2");
    assert!(
        e.load_state(&owner).is_err(),
        "entry history independently binds current token owner"
    );
    let mut extra = v.clone();
    extra["inactiveAllocatedCards"][0]["source"] = json!("c1");
    assert!(e.load_state(&extra).is_err());
}
#[test]
fn inactive_physical_keyword_composes_with_entry_power_without_guard_changes() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (s, id) = paid_dead(&e, seat, true);
        assert_eq!(s.card(id).inactive_entry_power, 1);
        let v = e.snapshot(&s);
        assert_eq!(v["inactiveUnitEntryStats"][0]["power"], 1);
        assert_eq!(
            v["inactiveAllocatedCards"][0]["phaseKeywords"],
            json!({"sentinel":1})
        );
        let loaded = e.load_state(&v).unwrap();
        assert_eq!(loaded.card(id).inactive_entry_power, 1);
        assert!(e.has_keyword(&loaded, id, "sentinel"));
        assert_eq!(loaded.zobrist(), s.zobrist());
        assert_eq!(e.snapshot(&loaded), v);
        let mut bad = v.clone();
        bad["inactiveUnitEntryStats"][0]["power"] = json!(0);
        assert!(e.load_state(&bad).is_err());
    }
}
#[test]
fn inactive_physical_keyword_legacy_and_allocated_empty_rows() {
    let e = engine();
    let s = board(&e, Seat::P1);
    let v = e.snapshot(&s);
    assert!(v.get("inactivePhaseKeywordStateVersion").is_none());
    e.load_state(&v).unwrap();
    let mut orphan = v.clone();
    orphan["inactivePhaseKeywordStateVersion"] = json!(1);
    orphan["inactiveAllocatedCards"] = json!([]);
    assert!(e.load_state(&orphan).is_err());
    let (mut s, first) = paid_dead(&e, Seat::P1, false);
    let old = add(&e, &mut s, Seat::P2, "TOKEN_SHIELD");
    let mut v = e.snapshot(&s);
    assert_eq!(
        v["inactiveAllocatedCards"][0]["id"],
        format!("c{}", first - 1)
    );
    assert_eq!(
        v["inactiveAllocatedCards"][1]["id"],
        format!("c{}", old - 1)
    );
    assert_eq!(v["inactiveAllocatedCards"][1]["phaseKeywords"], json!({}));
    let loaded = e.load_state(&v).unwrap();
    assert_eq!(loaded.zobrist(), s.zobrist());
    v["inactiveAllocatedCards"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert!(e.load_state(&v).is_err());
}
#[test]
fn inactive_physical_keyword_selected_pool_definition_guard() {
    let all = engine();
    let (s, _) = paid_dead(&all, Seat::P1, false);
    let v = all.snapshot(&s);
    let mut metadata = all.catalogue.as_ref().unwrap().metadata.clone();
    metadata.remove("TOKEN_MANDALORIAN");
    assert!(crate::validation::inactive_allocated_rows(&v, &metadata).is_err());
    let mut bad = v.clone();
    bad["inactiveAllocatedCards"][0]["name"] = json!("shield");
    bad["inactiveAllocatedCards"][0]["setCode"] = json!("TOKEN_SHIELD");
    assert!(
        all.load_state(&bad).is_err(),
        "nonempty Sentinel cannot move to a different native token type"
    );
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    assert_eq!(std::mem::size_of::<crate::model::ResolutionContext>(), 124);
}
#[test]
fn inactive_physical_keyword_deployed_leader_alias_roundtrip() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (mut s, _) = paid_dead(&e, seat, false);
        if s.active != seat {
            let pass = e
                .legal_actions(&s)
                .into_iter()
                .find(|a| a.label == "Pass")
                .unwrap();
            s = e.apply(&s, &pass).unwrap();
        }
        let leader_action = e.card_action(&s, s.player(seat).leader);
        assert!(e.legal_actions(&s).contains(&leader_action));
        s = e.apply(&s, &leader_action).unwrap();
        let deploy = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label.starts_with("Deploy "))
            .unwrap();
        s = e.apply(&s, &deploy).unwrap();
        drain(&e, &mut s);
        let leader = s.player(seat).leader;
        assert!(s.card(leader).deployed && e.in_arena(&s, leader));
        let v = e.snapshot(&s);
        let loaded = e.load_state(&v).unwrap();
        assert_eq!(loaded.zobrist(), s.zobrist());
        assert_eq!(e.snapshot(&loaded), v);
        let seat_key = if seat == Seat::P1 { "p1" } else { "p2" };
        let mut bad = v.clone();
        let duplicate = bad["players"][seat_key]["ground"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == format!("c{}", leader - 1))
            .unwrap()
            .clone();
        bad["players"][seat_key]["ground"]
            .as_array_mut()
            .unwrap()
            .push(duplicate.clone());
        assert!(crate::validation::allocated_live_ids(&bad).is_err());
        let mut bad = v.clone();
        bad["players"][seat_key]["discard"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(crate::validation::allocated_live_ids(&bad).is_err());
        let mut bad = v.clone();
        bad["players"][seat_key]["leader"]["deployed"] = json!(false);
        assert!(crate::validation::allocated_live_ids(&bad).is_err());
        let mut bad = v.clone();
        bad["players"][seat_key]["leader"]["name"] = json!("wrong");
        assert!(crate::validation::allocated_live_ids(&bad).is_err());
    }
}

// Shared inventory expectations were written before these tests. These native
// constructions are not claimed as pinned-engine or terminal trace evidence.
fn assert_exact_inventory_roundtrip(e: &Engine, s: &State) -> Value {
    let v = e.snapshot(s);
    let live = crate::validation::allocated_live_ids(&v).unwrap();
    let expected = (1..s.next_id)
        .filter(|id| !live.contains(id))
        .map(|id| format!("c{}", id - 1))
        .collect::<Vec<_>>();
    assert_eq!(v["inactivePhaseKeywordStateVersion"], 1);
    assert_eq!(
        v["inactiveAllocatedCards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>(),
        expected
    );
    let loaded = e.load_state(&v).unwrap();
    assert_eq!(e.snapshot(&loaded), v);
    assert_eq!(s.zobrist(), loaded.zobrist());
    eprintln!("INACTIVE_WIRE_ROUNDTRIP {}",json!({"nativeHash":s.zobrist().to_string(),"importedHash":loaded.zobrist().to_string(),"nextCardId":s.next_id-1,"inactiveAllocatedCards":v["inactiveAllocatedCards"],"jsonEqual":e.snapshot(&loaded)==v}));
    assert_eq!(s.next_id, loaded.next_id);
    assert_eq!(e.legal_actions(s), e.legal_actions(&loaded));
    let copy = *s;
    assert_eq!(s.zobrist(), copy.zobrist());
    v
}
fn helgait_six_then_expire(e: &Engine, s: &mut State, seat: Seat, host: CardId) -> Vec<CardId> {
    s.active = seat;
    s.action_player = Some(seat);
    s.prev_passed = false;
    let source = unit(e, s, seat, "ASH_195");
    let initial = assert_exact_inventory_roundtrip(e, s);
    assert!(initial["inactiveAllocatedCards"].is_array());
    let delta = 6 - i64::from(e.power(s, source, false));
    s.card_mut(source).phase_power = i32::try_from(delta).unwrap();
    e.defeat_events(s, &[(source, crate::defeat_cause::DefeatCause::Ability)])
        .unwrap();
    assert_eq!(
        s.death_pending.entries[..s.death_pending.len as usize]
            .iter()
            .find(|r| r.source == source)
            .unwrap()
            .last_known_power,
        6
    );
    e.open_deaths(s, DeathContinuation::Finish).unwrap();
    assert_eq!(s.decision, Decision::AbilityDistribute);
    let arg = json!([[format!("c{}", host - 1), 6]]);
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.kind == "distribute" && a.arg == Some(arg.clone()))
        .unwrap();
    let first = s.next_id;
    *s = e.apply(s, &a).unwrap();
    let ids = (first..first + 6).collect::<Vec<_>>();
    assert!(ids
        .iter()
        .all(|id| s.card(*id).parent == host && s.card(*id).owner == seat));
    assert_exact_inventory_roundtrip(e, s);
    if s.active != seat {
        *s = e.apply(s, &Action::button("Pass", "pass")).unwrap();
    }
    *s = e.apply(s, &e.card_action(s, host)).unwrap();
    if s.decision == Decision::AbilityMenu {
        let a = e
            .legal_actions(s)
            .into_iter()
            .find(|a| a.label == "Attack")
            .unwrap();
        *s = e.apply(s, &a).unwrap();
    }
    assert_eq!(s.decision, Decision::AttackTarget);
    let base = s.player(seat.other()).base;
    *s = e.apply(s, &e.card_action(s, base)).unwrap();
    assert_eq!(s.decision, Decision::DeathBatch);
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == "Resolve all (6)")
        .unwrap();
    *s = e.apply(s, &a).unwrap();
    drain(e, s);
    assert!(ids
        .iter()
        .all(|id| s.card(*id).parent == 0 && s.card(*id).owner == seat));
    let v = assert_exact_inventory_roundtrip(e, s);
    for id in &ids {
        let row = v["inactiveAllocatedCards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == format!("c{}", id - 1))
            .unwrap();
        assert_eq!(row["setCode"], "TOKEN_ADVANTAGE");
        assert_eq!(row["owner"], seat.as_str());
        assert_eq!(row["phaseKeywords"], json!({}));
    }
    ids
}
#[test]
fn shared_inactive_inventory_helgait_six_actual_expiry_both_seats_backends() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let host = unit(&e, &mut s, seat, "SOR_095");
            let ids = helgait_six_then_expire(&e, &mut s, seat, host);
            assert_eq!(ids.len(), 6);
            assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
        }
    }
}
#[test]
fn shared_inactive_inventory_coexists_buytime_outcast_both_chronological_orders() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for buytime_first in [true, false] {
            let mut s = if buytime_first {
                paid_dead(&e, seat, true).0
            } else {
                board(&e, seat)
            };
            let host = unit(&e, &mut s, seat, "SOR_095");
            let ids = helgait_six_then_expire(&e, &mut s, seat, host);
            if !buytime_first {
                unit(&e, &mut s, seat.other(), "SHD_037");
                unit(&e, &mut s, seat, "ASH_041");
                s.active = seat;
                s.action_player = Some(seat);
                s.prev_passed = false;
                let event = add(&e, &mut s, seat, "ASH_091");
                s.player_mut(seat).hand.push(event).unwrap();
                for _ in 0..10 {
                    let r = add(&e, &mut s, seat, "LOF_254");
                    s.player_mut(seat).resources.push(r).unwrap();
                }
                s = e.apply(&s, &e.card_action(&s, event)).unwrap();
                drain(&e, &mut s);
            }
            let mando = (1..s.next_id)
                .find(|id| e.def(&s, *id).code == "TOKEN_MANDALORIAN")
                .unwrap();
            assert!(!e.in_arena(&s, mando));
            assert!(e.has_keyword(&s, mando, "sentinel"));
            assert_eq!(s.card(mando).inactive_entry_power, 1);
            let v = assert_exact_inventory_roundtrip(&e, &s);
            assert_eq!(v["inactiveAllocatedCards"].as_array().unwrap().len(), 7);
            assert!(ids.iter().all(|id| v["inactiveAllocatedCards"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["id"] == format!("c{}", id - 1))));
            s.players[0].out = true;
            s.players[1].out = true;
            e.advance(&mut s).unwrap();
            drain(&e, &mut s);
            let expired = assert_exact_inventory_roundtrip(&e, &s);
            assert_eq!(
                expired["inactiveAllocatedCards"].as_array().unwrap().len(),
                7
            );
            assert!(expired["inactiveAllocatedCards"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["phaseKeywords"] == json!({})));
        }
    }
}
#[test]
fn shared_inactive_inventory_helgait_versions_rows_ownership_and_selected_tokens_fail_closed() {
    let e = engine();
    let mut s = board(&e, Seat::P2);
    let host = unit(&e, &mut s, Seat::P2, "SOR_095");
    helgait_six_then_expire(&e, &mut s, Seat::P2, host);
    let v = e.snapshot(&s);
    for field in ["inactivePhaseKeywordStateVersion", "inactiveAllocatedCards"] {
        let mut bad = v.clone();
        bad.as_object_mut().unwrap().remove(field);
        assert!(e.load_state(&bad).is_err(), "{field}");
    }
    let mut bad = v.clone();
    bad.as_object_mut()
        .unwrap()
        .remove("inactivePhaseKeywordStateVersion");
    bad.as_object_mut()
        .unwrap()
        .remove("inactiveAllocatedCards");
    assert!(e.load_state(&bad).is_err());
    for (field, value) in [
        ("id", json!("c999")),
        ("id", v["players"]["p2"]["forceToken"]["id"].clone()),
        ("id", v["players"]["p2"]["base"]["id"].clone()),
        ("owner", json!("p1")),
        ("owner", json!("bogus")),
        ("controller", json!("p1")),
        ("name", json!("weakness")),
        ("setCode", json!("TOKEN_THE_FORCE")),
        ("phaseKeywords", json!({"sentinel":1})),
        ("phaseKeywords", json!({"ambush":1})),
        ("phaseKeywords", json!({"sentinel":2})),
        ("phaseKeywords", Value::Null),
    ] {
        let mut bad = v.clone();
        bad["inactiveAllocatedCards"][0][field] = value;
        assert!(e.load_state(&bad).is_err(), "{field}");
    }
    for mode in 0..6 {
        let mut bad = v.clone();
        let a = bad["inactiveAllocatedCards"].as_array_mut().unwrap();
        match mode {
            0 => {
                a.remove(0);
            }
            1 => a.clear(),
            2 => a.push(a[0].clone()),
            3 => a.swap(0, 1),
            4 => {
                a[0]["unknown"] = json!(1);
            }
            _ => {
                a[0].as_object_mut().unwrap().remove("phaseKeywords");
            }
        }
        assert!(e.load_state(&bad).is_err(), "mode{mode}");
    }
    let mut metadata = e.catalogue.as_ref().unwrap().metadata.clone();
    metadata.remove("TOKEN_ADVANTAGE");
    assert!(crate::validation::inactive_allocated_rows(&v, &metadata).is_err());
}
#[test]
fn shared_inactive_inventory_helgait_cannot_authorize_inactive_sentinel() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = add(&e, &mut s, Seat::P1, "ASH_195");
    s.player_mut(Seat::P1).hand.push(source).unwrap();
    let mando = unit(&e, &mut s, Seat::P1, "TOKEN_MANDALORIAN");
    s.phase_history.entered(Seat::P1, mando);
    e.defeat(&mut s, mando).unwrap();
    let mut v = assert_exact_inventory_roundtrip(&e, &s);
    v["inactiveAllocatedCards"][0]["phaseKeywords"] = json!({"sentinel":1});
    let err = e.load_state(&v).unwrap_err();
    assert!(err.contains("played physical creation source"), "{err}");
}
#[test]
fn shared_inactive_inventory_all_retired_native_token_classes_keep_metadata_and_hash() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_195");
        s.player_mut(seat).hand.push(source).unwrap();
        let host = unit(&e, &mut s, seat, "SOR_095");
        s.source = host;
        s.ability_target = host;
        // Stage physical upgrades; exercise the real shared detach/defeat path.
        // Generation is covered by native producer tests, not claimed here.
        for (code, n) in [
            ("TOKEN_SHIELD", 2),
            ("TOKEN_EXPERIENCE", 2),
            ("TOKEN_WEAKNESS", 1),
            ("TOKEN_ADVANTAGE", 2),
        ] {
            for _ in 0..n {
                let id = add(&e, &mut s, seat, code);
                if code == "TOKEN_SHIELD" {
                    s.card_mut(id).next_token = s.card(host).shield;
                    s.card_mut(host).shield = id;
                }
                if code == "TOKEN_EXPERIENCE" {
                    s.card_mut(id).next_token = s.card(host).experience;
                    s.card_mut(host).experience = id;
                }
                e.attach(&mut s, id, host);
            }
        }
        let mut ids = e.upgrades(&s, host);
        e.defeat(&mut s, host).unwrap();
        for code in [
            "TOKEN_MANDALORIAN",
            "TOKEN_SPY",
            "TOKEN_BEAST",
            "TOKEN_CLONE_TROOPER",
            "TOKEN_BATTLE_DROID",
            "TOKEN_TIE_FIGHTER",
            "TOKEN_XWING",
        ] {
            let id = unit(&e, &mut s, seat, code);
            ids.push(id);
            e.defeat(&mut s, id).unwrap();
        }
        s.source = 0;
        s.ability_target = 0;
        let v = assert_exact_inventory_roundtrip(&e, &s);
        assert_eq!(
            v["inactiveAllocatedCards"].as_array().unwrap().len(),
            ids.len()
        );
        let loaded = e.load_state(&v).unwrap();
        for id in ids {
            assert_eq!(s.card(id).def, loaded.card(id).def);
            assert_eq!(s.card(id).owner, loaded.card(id).owner);
            assert_eq!(s.card(id).parent, 0);
            assert_eq!(s.card(id).next_token, 0);
            assert_eq!(s.card(id).next_upgrade, 0);
        }
    }
}
#[test]
fn shared_inactive_inventory_required_before_creation_in_each_physical_source_zone() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for zone in ["hand","deck","resources","discard","arena"] {let mut s=board(&e,seat);let id=add(&e,&mut s,seat,"ASH_195");match zone {"hand"=>s.player_mut(seat).hand.push(id).unwrap(),"deck"=>s.player_mut(seat).deck.push(id).unwrap(),"resources"=>s.player_mut(seat).resources.push(id).unwrap(),"discard"=>s.player_mut(seat).discard.push(id).unwrap(),_=>e.add_arena(&mut s,id).unwrap()};let v=assert_exact_inventory_roundtrip(&e,&s);assert_eq!(v["inactiveAllocatedCards"],json!([]));let mut missing=v;missing.as_object_mut().unwrap().remove("inactivePhaseKeywordStateVersion");missing.as_object_mut().unwrap().remove("inactiveAllocatedCards");assert!(e.load_state(&missing).is_err(),"{seat:?} {zone}");}}
}

// Each event/attack starts from a public-import fixed point. Initial import may
// establish the wire's documented flags; after gameplay no state is repaired.
fn four_family_engine(backend: RulesBackend, source: &str) -> Engine {
    let mut v = catalogue();
    v["cards"].as_array_mut().unwrap().retain(|c| [
        "ASH_172", "ASH_232", "SOR_023", "SOR_014", "LOF_254", "SOR_095",
        "ASH_201", "IC27_093", "TOKEN_SHIELD", "TOKEN_EXPERIENCE",
        "TOKEN_ADVANTAGE", "TOKEN_WEAKNESS", "TOKEN_THE_FORCE", "TOKEN_CREDIT"
    ].contains(&c["setCode"].as_str().unwrap()));
    v["cards"].as_array_mut().unwrap().retain(|c| !["ASH_172","ASH_232"].contains(&c["setCode"].as_str().unwrap()) || c["setCode"]==source);
    let mut e=Engine::from_catalogue_value(&v).unwrap();
    e.set_rules_backend(backend).unwrap();
    assert!(e.by_code.contains_key(source));
    assert!(!e.by_code.contains_key("ASH_091"));
    assert!(!e.by_code.contains_key("ASH_195"));
    e
}
fn four_family_choose(e:&Engine,s:&State,id:CardId)->State {
    let a=e.card_action(s,id);assert!(e.legal_actions(s).contains(&a),"{:?}: {:?}",s.decision,e.legal_actions(s));e.apply(s,&a).unwrap()
}
fn four_family_initial(e:&Engine,s:&State)->State {
    let wire=e.snapshot(s);let initial=e.load_state(&wire).unwrap();
    assert_eq!(e.snapshot(&initial),wire);
    let again=e.load_state(&e.snapshot(&initial)).unwrap();
    assert_eq!(initial.zobrist(),again.zobrist(),"pre-action supported-import full hash fixed point");
    assert_eq!(e.snapshot(&initial),e.snapshot(&again));
    assert_exact_inventory_roundtrip(e,&initial);
    initial
}
#[test]
fn inactive_four_family_paid_token_return_and_ordinary_controls_fixed_points() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter,RulesBackend::Aot] {
        let e=four_family_engine(backend,"ASH_232");
        for seat in [Seat::P1,Seat::P2] {
            for code in ["IC27_093","TOKEN_SHIELD","TOKEN_EXPERIENCE","TOKEN_ADVANTAGE","TOKEN_WEAKNESS"] {
                let mut s=board(&e,seat);
                let host=unit(&e,&mut s,seat,"SOR_095");
                let other=unit(&e,&mut s,seat.other(),"SOR_095");
                let token_owner=if code=="IC27_093" {seat.other()} else {seat};
                let old=add(&e,&mut s,token_owner,code);e.attach(&mut s,old,host);
                // Native attachment and scalar token lists have separate owners.
                // Maintain both when staging the public import position.
                if code=="TOKEN_SHIELD" {s.card_mut(host).shield=old;}
                if code=="TOKEN_EXPERIENCE" {s.card_mut(host).experience=old;}
                let event=add(&e,&mut s,seat,"ASH_232");s.player_mut(seat).hand.push(event).unwrap();
                for _ in 0..12 {let id=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(id).unwrap();}
                s=four_family_initial(&e,&s);
                let ready=e.ready_resources(&s,seat);
                s=four_family_choose(&e,&s,event);
                assert!(s.ability_committed);assert!(e.ready_resources(&s,seat)<ready);
                // A real imported position can export a pending grouped choice;
                // importing arbitrary pending resolution remains unsupported.
                let pending=e.snapshot(&s);
                assert_eq!(pending["inactivePhaseKeywordStateVersion"],1);
                assert!(e.load_state(&pending).unwrap_err().contains("actor"));
                let copy=s;assert_eq!(copy.zobrist(),s.zobrist());
                s=four_family_choose(&e,&s,old);let fresh=s.next_id;
                s=four_family_choose(&e,&s,other);
                assert!(s.is_action_window());assert!(!e.attached_in_play(&s,old));
                assert_eq!(s.player(seat.other()).hand.contains(old),code=="IC27_093");
                assert!(!s.player(seat.other()).discard.contains(old));
                assert!(e.upgrades(&s,other).contains(&fresh));assert_ne!(old,fresh);
                let wire=assert_exact_inventory_roundtrip(&e,&s);
                let rows=wire["inactiveAllocatedCards"].as_array().unwrap();
                if code=="IC27_093" {assert!(rows.is_empty());} else {
                    assert_eq!(rows.len(),1);assert_eq!(rows[0]["id"],format!("c{}",old-1));
                    assert_eq!(rows[0]["setCode"],code);assert_eq!(rows[0]["owner"],token_owner.as_str());
                    assert_eq!(rows[0]["controller"],token_owner.as_str());assert_eq!(rows[0]["phaseKeywords"],json!({}));
                    let mut bad=wire.clone();bad["inactiveAllocatedCards"][0]["phaseKeywords"]=json!({"sentinel":1});
                    assert!(e.load_state(&bad).is_err(),"232 does not authorize inactive Sentinel");
                }
                for key in ["inactivePhaseKeywordStateVersion","inactiveAllocatedCards"] {
                    let mut bad=wire.clone();bad.as_object_mut().unwrap().remove(key);assert!(e.load_state(&bad).is_err());
                }
            }
        }
    }
}
#[test]
fn inactive_four_family_saboteur_shield_attack_fixed_points_and_expiry() {
    let mut strict=four_family_engine(RulesBackend::Auto,"ASH_172");
    assert!(strict.set_rules_backend(RulesBackend::Aot).unwrap_err().contains("requires interpreter"));
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let e=four_family_engine(backend,"ASH_172");
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e,seat);
            let source=unit(&e,&mut s,seat,"ASH_172");
            let defender=unit(&e,&mut s,seat.other(),"ASH_201");
            let token=add(&e,&mut s,seat.other(),"TOKEN_SHIELD");e.attach(&mut s,token,defender);
            let discarded=add(&e,&mut s,seat,"SOR_095");s.player_mut(seat).hand.push(discarded).unwrap();
            s=four_family_initial(&e,&s);
            s=four_family_choose(&e,&s,source);s=four_family_choose(&e,&s,defender);
            let a=e.legal_actions(&s).into_iter().find(|a|a.label=="Saboteur: defeat all shields").unwrap();s=e.apply(&s,&a).unwrap();
            assert!(!e.attached_in_play(&s,token));assert_eq!(s.decision,Decision::AbilityOptional);
            let a=e.legal_actions(&s).into_iter().find(|a|a.arg==Some(json!("trigger"))).unwrap();s=e.apply(&s,&a).unwrap();s=four_family_choose(&e,&s,discarded);
            assert!(s.is_action_window());assert_eq!(s.support.len,0);assert_eq!(e.power(&s,source,false),3);
            assert!(s.player(seat).discard.contains(discarded));assert!(!e.in_arena(&s,defender));
            let wire=assert_exact_inventory_roundtrip(&e,&s);let rows=wire["inactiveAllocatedCards"].as_array().unwrap();
            assert_eq!(rows.len(),1);assert_eq!(rows[0]["id"],format!("c{}",token-1));assert_eq!(rows[0]["owner"],seat.other().as_str());
            let mut bad=wire.clone();bad["inactiveAllocatedCards"][0]["phaseKeywords"]=json!({"sentinel":1});assert!(e.load_state(&bad).is_err(),"172 does not authorize inactive Sentinel");
        }
    }
}
#[test]
fn inactive_four_family_new_sources_do_not_authorize_mandalorian_sentinel() {
    let e=engine();
    for seat in [Seat::P1,Seat::P2] {
        for code in ["ASH_172","ASH_232"] {
            let mut s=board(&e,seat);
            let source=add(&e,&mut s,seat,code);s.player_mut(seat).hand.push(source).unwrap();
            let mando=unit(&e,&mut s,seat,"TOKEN_MANDALORIAN");
            s.phase_history.entered(seat,mando);e.defeat(&mut s,mando).unwrap();
            let mut wire=assert_exact_inventory_roundtrip(&e,&s);
            wire["inactiveAllocatedCards"][0]["phaseKeywords"]=json!({"sentinel":1});
            let error=e.load_state(&wire).unwrap_err();
            assert!(error.contains("played physical creation source"),"{code}: {error}");
        }
    }
}
