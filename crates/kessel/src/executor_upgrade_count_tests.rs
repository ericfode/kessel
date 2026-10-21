use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine() -> Engine {
    let e = Engine::from_catalogue_value(&catalogue()).unwrap();
    assert!(
        e.by_code.contains_key("ASH_197"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_197")
    );
    e
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
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.credit_state_exact = e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..12 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn attached(e: &Engine, s: &mut State, host: CardId, code: &str) -> CardId {
    let id = add(e, s, s.card(host).owner, code);
    e.attach(s, id, host);
    id
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(e.legal_actions(s).contains(&a), "{:?}", e.legal_actions(s));
    e.apply(s, &a).unwrap()
}
#[test]
fn executor_complete_source_and_only_new_admission() {
    let e = engine();
    let report: Value = serde_json::from_slice(include_bytes!(
        "../../../compiler/generated/runtime-admission.json"
    ))
    .unwrap();
    let old = report["runtimeAdmittedCards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["setCode"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let admitted = e
        .catalogue
        .as_ref()
        .unwrap()
        .cards
        .keys()
        .filter(|code|!e.catalogue.as_ref().unwrap().reprint_of.contains_key(*code))
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let added = admitted.difference(&old).copied().collect::<Vec<_>>();
    let removed = old.difference(&admitted).copied().collect::<Vec<_>>();
    assert!(added.is_empty() || added == vec!["ASH_197"], "{added:?}");
    assert!(removed.is_empty(), "{removed:?}");
}
#[test]
fn executor_live_physical_attachments_both_arenas_source_identity_and_copy_hash() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "ASH_197");
        let other = unit(&e, &mut s, seat, "ASH_197");
        let ground = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        assert_eq!(e.power(&s, source, false), 5);
        let a = attached(&e, &mut s, other, "TOKEN_ADVANTAGE");
        attached(&e, &mut s, ground, "TOKEN_EXPERIENCE");
        attached(&e, &mut s, ground, "TOKEN_SHIELD");
        attached(&e, &mut s, ground, "IC27_052");
        attached(&e, &mut s, enemy, "TOKEN_ADVANTAGE");
        assert_eq!(e.power(&s, source, false), 9);
        attached(&e, &mut s, source, "TOKEN_ADVANTAGE");
        assert_eq!(e.power(&s, source, false), 10);
        assert_eq!(e.power(&s, other, false), 10);
        let h = s.zobrist();
        let copy = s;
        assert_eq!(copy, s);
        assert_eq!(copy.zobrist(), h);
        e.detach(&mut s, a);
        assert_eq!(e.power(&s, source, false), 9);
        assert_ne!(s.zobrist(), h);
    }
}
#[test]
fn executor_paid_play_captures_unique_old_and_new() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for keep_new in [false, true] {
            let mut s = board(&e, seat);
            let old = unit(&e, &mut s, seat, "ASH_197");
            let ground = unit(&e, &mut s, seat, "LOF_256");
            let space = unit(&e, &mut s, seat, "SOR_237");
            attached(&e, &mut s, ground, "TOKEN_ADVANTAGE");
            attached(&e, &mut s, space, "TOKEN_ADVANTAGE");
            let incoming = add(&e, &mut s, seat, "ASH_197");
            s.player_mut(seat).hand.push(incoming).unwrap();
            s = choose(&e, &s, incoming);
            assert_eq!(e.ready_resources(&s, seat), 0);
            s = choose(&e, &s, if keep_new { old } else { incoming });
            let survivor = if keep_new { incoming } else { old };
            assert!(e.in_arena(&s, survivor));
            assert!(!e.in_arena(&s, if keep_new { old } else { incoming }));
            assert_eq!(e.upgrades(&s, ground).len(), 2);
            assert_eq!(e.upgrades(&s, space).len(), 2);
            assert_eq!(e.power(&s, survivor, false), if keep_new { 9 } else { 10 });
        }
    }
}
#[test]
fn executor_raw_ast_mutations_and_aliases_reject_whole_card() {
    let original = catalogue();
    let i = original["cards"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v["setCode"] == "ASH_197")
        .unwrap();
    for mutation in 0..10 {
        let mut c = original.clone();
        let e = &mut c["cards"][i];
        match mutation {
            0 => e["abilities"]["card"]
                .as_array_mut()
                .unwrap()
                .pop()
                .map(|_| ())
                .unwrap(),
            1 => {
                e["abilities"]["card"][0]["effect"]["modifier"]["value"]["callback"]["body"]
                    ["statements"][0]["bindings"][0]["mutable"] = json!(true);
            }
            2 => {
                e["card"]["unique"] = json!(false);
            }
            3 => {
                e["card"]["keywords"] = json!(["pilot"]);
            }
            4 => {
                e["classMethods"] = json!([{"name":"unreviewed"}]);
            }
            5 => {
                e["abilities"]["card"][1]["optional"] = json!(true);
            }
            6 => {
                e["abilities"]["card"][0]["effect"] = json!({"kind":"modifyStats","target":"self","duration":"persistent","power":{"kind":"query","name":crate::executor_upgrade_count::QUERY,"receiver":{"kind":"ref","path":["modifierTarget"]},"args":[]},"hp":{"kind":"const","value":0}});
            }
            7 => {
                e["abilities"]["card"][0]["hooks"] = json!([{}]);
            }
            8 => {
                e["stateWatchers"] = json!([{}]);
            }
            _ => {
                e["baseClass"] = json!("LeaderUnitCard");
            }
        }
        e["setCode"] = json!("TEST_EXECUTOR_ALIAS");
        let loaded = Engine::from_catalogue_value(&c).unwrap();
        assert!(
            !loaded.by_code.contains_key("TEST_EXECUTOR_ALIAS"),
            "mutation {mutation}"
        );
    }
}
#[test]
fn executor_all_native_unit_recipient_definitions_and_deployed_leader() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let leader = s.player(seat).leader;
        s.card_mut(leader).deployed = true;
        e.add_arena(&mut s, leader).unwrap();
        let mut recipients = vec![leader];
        for code in [
            "TOKEN_BATTLE_DROID",
            "TOKEN_BEAST",
            "TOKEN_CLONE_TROOPER",
            "TOKEN_MANDALORIAN",
            "TOKEN_SPY",
            "TOKEN_TIE_FIGHTER",
            "TOKEN_XWING",
        ] {
            if e.by_code.contains_key(code) {
                recipients.push(unit(&e, &mut s, seat, code));
            }
        }
        let enemy = unit(&e, &mut s, seat.other(), "TOKEN_BATTLE_DROID");
        let source = add(&e, &mut s, seat, "ASH_197");
        s.player_mut(seat).hand.push(source).unwrap();
        let s = choose(&e, &s, source);
        assert_eq!(e.power(&s, source, false), 5 + recipients.len() as u16);
        assert!(s.card(source).exhausted);
        for id in recipients {
            assert_eq!(e.upgrades(&s, id).len(), 1);
            assert_eq!(
                e.upgrades(&s, id)
                    .iter()
                    .all(|&token| Some(s.card(token).def) == e.advantage_def),
                true
            );
        }
        assert_eq!(e.upgrades(&s, enemy).len(), 0);
        assert_eq!(e.upgrades(&s, source).len(), 0);
    }
}
#[test]
fn executor_native_attack_expiry_refresh_and_shield_consumption() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "ASH_197");
        let host = unit(&e, &mut s, seat, "LOF_256");
        s.source = host;
        let give:Effect=serde_json::from_value(json!({"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":1},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).unwrap();
        e.apply_effect(&mut s, &give).unwrap();
        s.source = 0;
        assert_eq!(e.power(&s, source, false), 6);
        s = choose(&e, &s, host);
        if s.decision == Decision::AbilityMenu {
            let a = e
                .legal_actions(&s)
                .into_iter()
                .find(|a| a.label == "Attack")
                .unwrap();
            s = e.apply(&s, &a).unwrap();
        }
        let base = s.player(seat.other()).base;
        s = choose(&e, &s, base);
        assert_eq!(e.upgrades(&s, host).len(), 0);
        assert_eq!(e.power(&s, source, false), 5);
        // Native Shield uses the token creation scalar, then real damage consumes it.
        s.source = host;
        let shield: Effect = serde_json::from_value(
            json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":1}}),
        )
        .unwrap();
        e.apply_effect(&mut s, &shield).unwrap();
        assert_eq!(e.power(&s, source, false), 6);
        let damage: Effect = serde_json::from_value(
            json!({"kind":"damage","target":"self","amount":{"kind":"const","value":1}}),
        )
        .unwrap();
        e.apply_effect(&mut s, &damage).unwrap();
        assert_eq!(s.card(host).damage, 0);
        assert_eq!(e.power(&s, source, false), 5);
        s.source = 0;
    }
}
#[test]
fn executor_batch_token_capacity_failure_is_atomic_and_inline_sizes_stable() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_197");
    unit(&e, &mut s, Seat::P1, "LOF_256");
    unit(&e, &mut s, Seat::P1, "LOF_254");
    s.source = source;
    s.next_id = (MAX_CARDS - 1) as CardId;
    let before = s;
    let c = catalogue();
    let focus = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["setCode"] == "ASH_197")
        .unwrap();
    let effect: Effect =
        serde_json::from_value(focus["abilities"]["card"][1]["effect"].clone()).unwrap();
    assert!(e
        .apply_effect(&mut s, &effect)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(s, before);
    eprintln!(
        "Executor storage State={} Card={} ResolutionContext={} DeathFrame={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<Card>(),
        std::mem::size_of::<ResolutionContext>(),
        std::mem::size_of::<DeathFrame>()
    );
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    assert_eq!(std::mem::size_of::<Card>(), 40);
}
#[test]
fn executor_missing_or_malformed_advantage_dependency_rejects_and_alpha_slots_are_lexical() {
    let c = catalogue();
    let index = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v["setCode"] == "ASH_197")
        .unwrap();
    let original = c["cards"][index].clone();
    let mut changed = original.clone();
    fn rename(v: &mut Value) {
        match v {
            Value::Object(o) => {
                if let Some(Value::String(slot)) = o.get_mut("slot") {
                    *slot = format!("renamed:{slot}");
                }
                if o.get("kind") == Some(&json!("dataPath")) {
                    if let Some(root) = o.get_mut("root").and_then(Value::as_array_mut) {
                        for v in root {
                            if let Some(s) = v.as_str() {
                                *v = json!(format!("renamed:{s}"));
                            }
                        }
                    }
                }
                for v in o.values_mut() {
                    rename(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    rename(v);
                }
            }
            _ => {}
        }
    }
    rename(&mut changed["abilities"]);
    assert!(crate::executor_upgrade_count::prepare(&changed)
        .unwrap()
        .is_some());
    for malformed in [false, true] {
        let mut broken = c.clone();
        if malformed {
            let token = broken["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|v| v["setCode"] == "TOKEN_ADVANTAGE")
                .unwrap();
            token["card"]["power"] = json!(2);
        } else {
            broken["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
        }
        let e = Engine::from_catalogue_value(&broken).unwrap();
        assert!(!e.by_code.contains_key("ASH_197"));
    }
}

#[test]fn executor_standalone_program_and_query_guards_reject_composed_mutations(){let c=catalogue();let raw=c["cards"].as_array().unwrap().iter().find(|v|v["setCode"]=="ASH_197").unwrap();let normalized=crate::executor_upgrade_count::prepare(raw).unwrap().unwrap();for index in 0..2 {let good:crate::ability_ir::Ability=serde_json::from_value(normalized["card"][index].clone()).unwrap();assert!(Program::compile(&good).is_ok());for field in ["optional","coordinate","requiresConfirmation"]{let mut bad=normalized["card"][index].clone();bad[field]=json!(true);let bad=serde_json::from_value(bad).unwrap();assert!(Program::compile(&bad).is_err(),"{index}:{field}");}}let mut expression=normalized["card"][0]["effect"]["power"].clone();expression["receiver"]=json!({"kind":"ref","path":["player"]});let bad:Expr=serde_json::from_value(expression).unwrap();assert!(bad.validate().is_err());}
