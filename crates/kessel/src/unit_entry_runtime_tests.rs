//! Expectations come from the actual pinned ASH_041 plays retained in the
//! source design and mirrored by the reference fixture. These tests exercise
//! shared Rust execution; engine/native/Wasm differential evidence is separate.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_041")
        .unwrap()
        .clone()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e.by_code.get(code).unwrap_or_else(|| {
            panic!("{code}: {:?}", e.rejected_cards().and_then(|r| r.get(code)))
        }),
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
    s.credit_state_exact = e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.arena_order_exact = true;
    for p in [seat, seat.other()] {
        let b = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).base = b;
        let l = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).leader = l;
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..20 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label}: {:?}", e.legal_actions(s)));
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn play(e: &Engine, s: &State) -> State {
    let mut s = *s;
    let actor = s.active;
    let id = add(e, &mut s, actor, "ASH_041");
    s.player_mut(actor).hand.push(id).unwrap();
    choose(e, &s, id)
}
fn admits(v: Value) -> bool {
    crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
        .unwrap()
        .cards
        .contains_key("ASH_041")
}

#[test]
fn outcast_self_and_friendly_entry_bind_the_event_card() {
    let e = engine();
    assert!(
        e.by_code.contains_key("ASH_041"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_041")
    );
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let outcast = add(&e, &mut s, seat, "ASH_041");
        s.player_mut(seat).hand.push(outcast).unwrap();
        s = choose(&e, &s, outcast);
        assert_eq!(s.decision, Decision::Window);
        assert_eq!(e.power(&s, outcast, false), 2);
        let ally = add(&e, &mut s, seat, "LOF_254");
        s.player_mut(seat).hand.push(ally).unwrap();
        s.active = seat;
        s.action_player = Some(seat);
        s = choose(&e, &s, ally);
        assert_eq!(
            e.power(&s, ally, false),
            e.def(&s, ally).info.power.unwrap() as u16 + 1
        );
        assert_eq!(e.power(&s, outcast, false), 2);
    }
}
#[test]
fn outcast_capture_survives_unique_source_or_event_departure() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for keep_new in [false, true] {
            let mut s = board(&e, seat);
            let old = unit(&e, &mut s, seat, "ASH_041");
            let new = add(&e, &mut s, seat, "ASH_041");
            s.player_mut(seat).hand.push(new).unwrap();
            s = choose(&e, &s, new);
            assert_eq!(s.decision, Decision::Unique);
            s = choose(&e, &s, if keep_new { old } else { new });
            for _ in 0..8 {
                if s.decision == Decision::Window {
                    break;
                }
                let actions = e.legal_actions(&s);
                let a = actions
                    .iter()
                    .find(|a| a.label == "Resolve all")
                    .unwrap_or(&actions[0]);
                s = e.apply(&s, a).unwrap();
            }
            assert_eq!(s.decision, Decision::Window);
            if keep_new {
                assert_eq!(e.power(&s, new, false), 3);
                assert_eq!(s.card(old).inactive_entry_power, 0);
            } else {
                assert_eq!(e.power(&s, old, false), 1);
                assert_eq!(s.card(new).inactive_entry_power, 2);
                let mut snapshot = e.snapshot(&s);
                assert_eq!(snapshot["inactiveUnitEntryStats"][0]["power"], json!(2));
                let loaded = e.load_state(&snapshot).unwrap();
                assert_eq!(loaded.card(new).inactive_entry_power, 2);
                assert_eq!(e.snapshot(&loaded), snapshot);
                snapshot["inactiveUnitEntryStats"][0]["card"]["controller"] =
                    json!(seat.other().as_str());
                assert!(e.load_state(&snapshot).is_err());
            }
        }
    }
}
#[test]
fn entry_binding_and_inactive_effect_are_copied_hashed_and_clear_on_reentry() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_041");
    let target = unit(&e, &mut s, Seat::P1, "LOF_254");
    e.capture_unit_entry_watchers(&mut s, &[target]).unwrap();
    let capture = s.death_pending.entries[0];
    assert_eq!(capture.source, source);
    assert_eq!(capture.event_card, target);
    let before = s.zobrist();
    let copy = s;
    s.death_pending.entries[0].event_card = source;
    assert_ne!(before, s.zobrist());
    s = copy;
    e.defeat(&mut s, target).unwrap();
    s.card_mut(target).inactive_entry_power = 2;
    let before = s.zobrist();
    let copy = s;
    s.card_mut(target).inactive_entry_power = 1;
    assert_ne!(before, s.zobrist());
    s = copy;
    s.player_mut(Seat::P1).discard.remove(target);
    e.add_arena(&mut s, target).unwrap();
    assert_eq!(s.card(target).inactive_entry_power, 0);
    assert_eq!(
        e.power(&s, target, false),
        e.def(&s, target).info.power.unwrap() as u16
    );
}

fn settle(e: &Engine, mut s: State) -> State {
    for _ in 0..16 {
        if s.decision == Decision::Window {
            return s;
        }
        let a = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Resolve all")
            .unwrap_or_else(|| e.legal_actions(&s)[0].clone());
        s = e.apply(&s, &a).unwrap();
    }
    panic!("unsettled {:?}", s.decision)
}
#[test]
fn token_batches_keep_distinct_event_cards_after_framework_death() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        unit(&e, &mut s, seat, "ASH_041");
        unit(&e, &mut s, seat.other(), "SHD_037");
        let donor = add(&e, &mut s, seat, "ASH_111");
        s.player_mut(seat).hand.push(donor).unwrap();
        let first = s.next_id;
        s = settle(&e, choose(&e, &s, donor));
        assert_eq!(s.next_id, first + 2);
        for id in first..first + 2 {
            assert!(!e.in_arena(&s, id));
            assert_eq!(s.card(id).inactive_entry_power, 1);
            assert_eq!(s.card(id).shield, 0);
            assert_eq!(s.card(id).owner, seat);
        }
        let snapshot = e.snapshot(&s);
        assert_eq!(
            snapshot["inactiveUnitEntryStats"].as_array().unwrap().len(),
            2
        );
        let loaded = e.load_state(&snapshot).unwrap();
        assert_eq!(e.snapshot(&loaded), snapshot);
        for id in first..first + 2 {
            assert_eq!(loaded.card(id).def, s.card(id).def);
            assert_eq!(loaded.card(id).owner, seat);
        }
    }
}
#[test]
fn all_native_departed_zone_moves_clear_inactive_registrations() {
    let e = engine();
    for mode in 0..5 {
        let mut s = board(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "ASH_041");
        let id = unit(&e, &mut s, Seat::P1, "LOF_254");
        e.defeat(&mut s, id).unwrap();
        s.card_mut(id).inactive_entry_power = 2;
        s.source = source;
        match mode {
            0 => {
                s.ability_target = id;
                e.apply_effect(
                    &mut s,
                    &Effect::ReturnToHand {
                        target: Target::Name("chosen".into()),
                    },
                )
                .unwrap();
            }
            1 => {
                e.resource_card_card(&mut s, id).unwrap();
            }
            2 => {
                e.move_to_bottom_of_deck(&mut s, id).unwrap();
            }
            3 => {
                s.source = id;
                e.resource_source(&mut s, id).unwrap();
            }
            _ => {
                s.source = id;
                e.apply_resource_card_system(&mut s, &Target::Name("self".into()), "self", false)
                    .unwrap();
            }
        }
        assert_eq!(s.card(id).inactive_entry_power, 0, "move {mode}");
    }
}
#[test]
fn expiry_removes_inactive_effects_and_event_context_is_independent_of_result() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    unit(&e, &mut s, Seat::P1, "ASH_041");
    let id = unit(&e, &mut s, Seat::P1, "LOF_254");
    e.defeat(&mut s, id).unwrap();
    s.card_mut(id).inactive_entry_power = 2;
    s.entry_event_card = id;
    s.event_result.damage_dealt = 17;
    let context = ResolutionContext::capture(&s);
    s.entry_event_card = 0;
    s.event_result.damage_dealt = 0;
    context.restore(&mut s);
    assert_eq!(s.entry_event_card, id);
    assert_eq!(s.event_result.damage_dealt, 17);
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.card(id).inactive_entry_power, 0);
}
#[test]
fn unit_entry_registration_rejects_unknown_scope_and_metadata() {
    let good = entry();
    assert!(admits(good.clone()));
    for (path, value) in [
        ("optional", json!(true)),
        ("requiresConfirmation", json!(true)),
        ("condition", json!({"kind":"const","value":true})),
    ] {
        let mut bad = good.clone();
        bad["abilities"]["card"][0][path] = value;
        assert!(!admits(bad), "{path}");
    }
    for (key, value) in [
        ("types", json!(["unit", "token"])),
        ("keywords", json!(["support"])),
        ("pilotText", json!("Piloting [1]")),
    ] {
        let mut bad = good.clone();
        bad["card"][key] = value;
        assert!(!admits(bad), "{key}");
    }
    let mut bad = good.clone();
    bad["abilities"]["card"][0]["effect"]["target"]["extra"] = json!(true);
    assert!(!admits(bad));
    let mut bad = good;
    bad["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"] =
        json!("x".repeat(129));
    assert!(!admits(bad));
}

#[test]
fn unit_entry_history_and_offboard_identity_guards_reject_forgery() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_041");
    let id = unit(&e, &mut s, Seat::P1, "LOF_254");
    e.defeat(&mut s, id).unwrap();
    s.card_mut(id).inactive_entry_power = 2;
    let good = e.snapshot(&s);
    assert!(e.load_state(&good).is_ok());
    let mut bad = good.clone();
    bad.as_object_mut().unwrap().remove("unitEntryStateVersion");
    assert!(e.load_state(&bad).is_err());
    let mut bad = good.clone();
    bad.as_object_mut()
        .unwrap()
        .remove("inactiveUnitEntryStats");
    assert!(e.load_state(&bad).is_err());
    for (key, value) in [
        ("power", json!(0)),
        ("power", json!(65536)),
        ("power", json!(-1)),
        ("hp", json!(1)),
        ("unknown", json!(true)),
    ] {
        let mut bad = good.clone();
        bad["inactiveUnitEntryStats"][0][key] = value;
        assert!(e.load_state(&bad).is_err(), "{key}");
    }
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["id"] = json!(format!("c{}", source - 1));
    assert!(e.load_state(&bad).is_err());
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["name"] = json!("forged");
    assert!(e.load_state(&bad).is_err());
    let mut bad = good;
    let row = bad["inactiveUnitEntryStats"][0].clone();
    bad["inactiveUnitEntryStats"]
        .as_array_mut()
        .unwrap()
        .push(row);
    assert!(e.load_state(&bad).is_err());
}
#[test]
fn entry_capture_and_resolved_effect_overflow_leave_inputs_unchanged() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_041");
    let id = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.death_pending.len = MAX_DEATH_TRIGGERS as u8;
    let before = s;
    assert!(e.capture_unit_entry_watchers(&mut s, &[id]).is_err());
    assert_eq!(s, before);
    s.death_pending.len = 0;
    e.defeat(&mut s, id).unwrap();
    s.card_mut(id).inactive_entry_power = u16::MAX;
    s.source = source;
    s.entry_event_card = id;
    let p = *e.card_programs(&s, source).first().unwrap();
    s.ability_program = p + 1;
    let effect = e
        .program(p)
        .instructions
        .iter()
        .find_map(|i| {
            if let Instruction::Apply(effect) = i {
                Some(effect.clone())
            } else {
                None
            }
        })
        .unwrap();
    let before = s;
    assert!(e.apply_effect(&mut s, &effect).is_err());
    assert_eq!(s, before);
}
#[test]
fn leader_deploy_is_entry_but_attachment_and_snapshot_are_not() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        unit(&e, &mut s, seat, "ASH_041");
        let host = unit(&e, &mut s, seat, "LOF_192");
        // Whole-card Pilot lifecycle rejection applies to ordinary unit play
        // too. Keep the original unsanitized definition and assert that boundary.
        assert!(!e.by_code.contains_key("JTL_211"));
        assert!(e.rejected_cards().unwrap().contains_key("JTL_211"));
        let raw_pilot = catalogue()["cards"].as_array().unwrap().iter()
            .find(|c| c["setCode"] == "JTL_211").unwrap().clone();
        assert!(raw_pilot["card"]["keywords"].as_array().unwrap().iter().any(|k| k == "piloting"));
        // A native Shield attachment and a snapshot do not synthesize entry.
        e.create_token(&mut s, host, false).unwrap();
        assert_eq!(s.card(host).phase_power, 0);
        assert_eq!(s.death_pending.len, 0);
        assert_eq!(e.power(&s, host, false), 3);
        let snapshot = e.snapshot(&s);
        assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()), snapshot);
        assert_eq!(s.death_pending.len, 0);
        let leader = s.player(seat).leader;
        let old = e.power(&s, leader, false);
        s = choose(&e, &s, leader);
        if s.decision == Decision::LeaderMenu {
            s = button(&e, &s, "Deploy Sabine Wren");
        }
        s = settle(&e, s);
        assert!(s.card(leader).deployed);
        assert_eq!(e.power(&s, leader, false), old + 1);
    }
}

#[test]
fn absent_inactive_cards_require_exact_native_unit_token_identity_and_pool() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    unit(&e, &mut s, Seat::P1, "ASH_041");
    let id = add(&e, &mut s, Seat::P1, "TOKEN_MANDALORIAN");
    s.card_mut(id).present = false;
    s.card_mut(id).inactive_entry_power = 1;
    let good = e.snapshot(&s);
    assert_eq!(e.snapshot(&e.load_state(&good).unwrap()), good);
    for code in ["LOF_254", "ASH_019", "SOR_014", "TOKEN_SHIELD"] {
        let mut bad = good.clone();
        let def = &e.definitions[e.by_code[code] as usize];
        bad["inactiveUnitEntryStats"][0]["card"]["setCode"] = json!(code);
        bad["inactiveUnitEntryStats"][0]["card"]["name"] = json!(def.info.internal_name);
        assert!(e.load_state(&bad).is_err(), "absent {code}");
    }
    let mut smaller = catalogue();
    smaller["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_MANDALORIAN");
    let selected = Engine::from_catalogue_value(&smaller).unwrap();
    assert!(
        selected.load_state(&good).is_err(),
        "missing selected native token"
    );
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["name"] = json!("forged");
    assert!(e.load_state(&bad).is_err());
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["id"] = json!(format!("c{}", s.next_id - 1));
    assert!(e.load_state(&bad).is_err());
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["id"] = json!(format!("c0{}", id - 1));
    assert!(e.load_state(&bad).is_err());
    for value in [json!(1.5), json!(0), json!(-1), json!(65536)] {
        let mut bad = good.clone();
        bad["inactiveUnitEntryStats"][0]["power"] = value;
        assert!(e.load_state(&bad).is_err());
    }
    for value in [json!(0), json!(2), json!("1")] {
        let mut bad = good.clone();
        bad["unitEntryStateVersion"] = value;
        assert!(e.load_state(&bad).is_err());
    }
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["owner"] = json!("p2");
    assert!(e.load_state(&bad).is_err());
    let mut bad = good.clone();
    bad["inactiveUnitEntryStats"][0]["card"]["unknown"] = json!(true);
    assert!(e.load_state(&bad).is_err());
    let mut empty = good;
    empty["inactiveUnitEntryStats"] = json!([]);
    assert!(e.load_state(&empty).is_ok());
}

#[test]
fn unit_entry_wire_presence_is_retained_without_changing_gameplay_hashes() {
    let e = engine();
    let old = board(&e, Seat::P1);
    let legacy = e.snapshot(&old);
    assert!(legacy.get("unitEntryStateVersion").is_none());
    let loaded_old = e.load_state(&legacy).unwrap();
    assert!(!loaded_old.unit_entry_state_explicit);
    assert_eq!(e.snapshot(&loaded_old), legacy);
    assert_eq!(loaded_old.zobrist(), old.zobrist());
    let mut modern = legacy.clone();
    modern["unitEntryStateVersion"] = json!(1);
    modern["inactiveUnitEntryStats"] = json!([]);
    let loaded = e.load_state(&modern).unwrap();
    assert!(loaded.unit_entry_state_explicit);
    assert_ne!(loaded, loaded_old);
    assert_eq!(loaded.zobrist(), loaded_old.zobrist(), "presence is not gameplay");
    assert_eq!(e.legal_actions(&loaded), e.legal_actions(&loaded_old));
    let checkpoint = loaded;
    let after = button(&e, &loaded, "Pass");
    assert_eq!(e.snapshot(&after)["unitEntryStateVersion"], json!(1));
    assert_eq!(e.snapshot(&checkpoint), modern);
    assert_eq!(e.snapshot(&e.load_state(&modern).unwrap()), modern);

    let mut native = old;
    let outcast = unit(&e, &mut native, Seat::P1, "ASH_041");
    assert!(!native.unit_entry_state_explicit);
    let exported = e.snapshot(&native);
    let imported = e.load_state(&exported).unwrap();
    assert!(imported.unit_entry_state_explicit);
    assert_eq!(imported.zobrist(), native.zobrist());
    assert_eq!(e.legal_actions(&imported), e.legal_actions(&native));
    let mut departed = imported;
    // Preserve observed wire presence after the last source is removed, even
    // with no inactive-stat rows. This is a projection-presence diagnostic.
    e.leave_play(&mut departed, outcast).unwrap();
    departed.player_mut(Seat::P1).discard.remove(outcast);
    departed.card_mut(outcast).present = false;
    assert_eq!(e.snapshot(&departed)["inactiveUnitEntryStats"], json!([]));
    assert_eq!(e.snapshot(&departed)["unitEntryStateVersion"], json!(1));
    let mut changed = imported;
    changed.card_mut(outcast).inactive_entry_power = 1;
    assert_ne!(changed.zobrist(), imported.zobrist(), "actual card stats remain gameplay");
    let mut event = imported;
    event.entry_event_card = outcast;
    assert_ne!(event.zobrist(), imported.zobrist(), "captured event target remains gameplay");
}

fn selected_entry_engine(include_mandalorian: bool) -> Engine {
    let full = catalogue();
    let mut codes = vec!["ASH_019", "SOR_014", "LOF_254", "SOR_095", "TOKEN_SHIELD", "TOKEN_CREDIT"];
    if include_mandalorian { codes.push("TOKEN_MANDALORIAN"); }
    let cards = codes.iter().map(|code| {
        let mut card = full["cards"].as_array().unwrap().iter()
            .find(|c| c["setCode"] == *code).unwrap()["card"].clone();
        card["setCode"] = json!(code);
        card
    }).collect::<Vec<_>>();
    Engine::from_compiled_pool_bytes(&serde_json::to_vec(&json!({"schema":"omni-rs-pool-v1","cards":cards})).unwrap(),
        &serde_json::to_vec(&full).unwrap()).unwrap()
}

#[test]
fn unit_entry_inactive_native_token_cannot_escape_selected_pool_through_global_catalogue() {
    let included = selected_entry_engine(true);
    let mut s = board(&included, Seat::P1);
    let id = add(&included, &mut s, Seat::P1, "TOKEN_MANDALORIAN");
    s.card_mut(id).present = false;
    s.card_mut(id).inactive_entry_power = 1;
    let snapshot = included.snapshot(&s);
    let excluded = selected_entry_engine(false);
    // This is the important boundary: the global definition STILL EXISTS.
    assert!(excluded.by_code.contains_key("TOKEN_MANDALORIAN"));
    assert!(!excluded.catalogue.as_ref().unwrap().metadata.contains_key("TOKEN_MANDALORIAN"));
    let err = excluded.load_state(&snapshot).unwrap_err();
    assert!(err.contains("not admitted in selected pool"), "{err}");
    let good = included.load_state(&snapshot).unwrap();
    assert_eq!(included.snapshot(&good), snapshot);
    assert!(!good.card(id).present);
    assert_eq!(good.card(id).owner, Seat::P1);
    assert_eq!(good.next_id, s.next_id);
    for (key, value) in [("id", json!(format!("c{}", s.next_id - 1))), ("name", json!("forged")),
        ("owner", json!("p2")), ("controller", json!("p2"))] {
        let mut bad = snapshot.clone();
        bad["inactiveUnitEntryStats"][0]["card"][key] = value;
        assert!(included.load_state(&bad).is_err(), "forged {key}");
    }
}
