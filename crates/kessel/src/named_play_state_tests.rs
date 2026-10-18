//! Bounded state/import regressions, separate from source contract and play witnesses.
use super::*;
fn restricted_engine(named: bool) -> Engine {
    let catalogue = committed_catalogue();
    let mut codes = vec![
        "SOR_014",
        "IBH_053",
        "ASH_019",
        "LOF_254",
        "LOF_256",
        "TOKEN_SHIELD",
    ];
    if named {
        codes.extend(["SOR_062", "ASH_077"]);
    }
    let cards = codes
        .iter()
        .map(|code| {
            let entry = catalogue["cards"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["setCode"] == *code)
                .unwrap();
            let mut card = entry["card"].clone();
            card["setCode"] = json!(code);
            card
        })
        .collect::<Vec<_>>();
    Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&json!({"schema":"omni-rs-pool-v1","cards":cards})).unwrap(),
        committed_catalogue_bytes(),
    )
    .unwrap()
}
fn locked(e: &Engine) -> (State, CardId) {
    let mut s = board(e);
    s.round = 5;
    let source = add(e, &mut s, Seat::P1, "SOR_062");
    e.add_arena(&mut s, source).unwrap();
    s.source = source;
    s.active = Seat::P1;
    let title = e
        .naming_options()
        .into_iter()
        .find(|(_, t)| *t == "Luke Skywalker")
        .unwrap()
        .0;
    e.install_named_play(&mut s, title).unwrap();
    s.source = 0;
    s.active = Seat::P2;
    s.action_player = Some(Seat::P2);
    (s, source)
}
#[test]
fn named_play_registry_preserves_absent_titles_order_and_rejects_forged_data() {
    let e = restricted_engine(true);
    let options = e.naming_options();
    assert_eq!(options.len(), 1931);
    assert_eq!(
        e.new_named_play_state().unwrap().registry,
        DEFAULT_NAMED_PLAY_REGISTRY
    );
    assert_eq!(options[327].1, "Commander Pyre");
    assert!(!e
        .catalogue
        .as_ref()
        .unwrap()
        .metadata
        .values()
        .any(|c| c["title"] == "Commander Pyre"));
    let original = committed_catalogue();
    for alteration in 0..4 {
        let mut c = original.clone();
        match alteration {
            0 => c["playableCardTitles"]["titles"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            1 => c["playableCardTitles"]["titles"][327] = json!("Forged"),
            2 => c["playableCardTitles"]["sourceHash"] = json!("0".repeat(64)),
            _ => c["playableCardTitles"]["unknown"] = json!(false),
        }
        assert!(Engine::from_catalogue_value(&c).is_err());
    }
    let mut legacy = original.clone();
    legacy.as_object_mut().unwrap().remove("playableCardTitles");
    let legacy = Engine::from_catalogue_value(&legacy).unwrap();
    for code in ["ASH_077", "SOR_062"] {
        assert!(legacy.rejected_cards().unwrap()[code].contains("registry"));
    }
}
#[test]
fn named_play_snapshot_roundtrip_and_legacy_guards_follow_selected_and_imported_cards() {
    let e = restricted_engine(true);
    let (s, _) = locked(&e);
    let snapshot = e.snapshot(&s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(loaded.named_play, s.named_play);
    assert_eq!(loaded.zobrist(), s.zobrist());
    let mut legacy = snapshot.clone();
    legacy.as_object_mut().unwrap().remove("namedPlayState");
    assert!(e.load_state(&legacy).unwrap_err().contains("named Play"));
    let old = restricted_engine(false);
    let mut fresh = board(&old);
    fresh.force_state_exact = false;
    fresh.credit_state_exact = false;
    let snapshot = old.snapshot(&fresh);
    assert!(
        old.load_state(&snapshot).is_ok(),
        "unrelated legacy pool remains compatible"
    );
    // Raw imports outside the selected pool are rejected by the earlier pool
    // gate. Trusted/directly constructed positions must also satisfy evidence
    // checks when they use a retained definition omitted by that selection.
    assert!(old
        .load_state(&legacy)
        .unwrap_err()
        .contains("unsupported card"));
    let mut imported = s;
    imported.named_play.exact = false;
    imported.named_play.len = 0;
    assert!(old
        .check_named_play(&imported)
        .unwrap_err()
        .contains("named Play"));
}
#[test]
fn named_play_snapshot_malformed_partial_and_unknown_evidence_rejects() {
    let e = restricted_engine(true);
    let (s, _) = locked(&e);
    let original = e.snapshot(&s);
    for alteration in 0..9 {
        let mut snapshot = original.clone();
        match alteration {
            0 => snapshot["namedPlayState"] = Value::Null,
            1 => snapshot["namedPlayState"]["version"] = json!(2),
            2 => {
                snapshot["namedPlayState"]
                    .as_object_mut()
                    .unwrap()
                    .remove("locks");
            }
            3 => snapshot["namedPlayState"]["registry"] = json!("0".repeat(64)),
            4 => snapshot["namedPlayState"]["locks"][0]["source"] = json!("c254"),
            5 => snapshot["namedPlayState"]["locks"][0]["title"] = json!("Not a playable title"),
            6 => snapshot["namedPlayState"]["locks"][0]["recipient"] = json!("p3"),
            7 => snapshot["namedPlayState"]["locks"][0]["unknown"] = json!(true),
            _ => {
                snapshot["namedPlayState"]["locks"] =
                    json!(vec![original["namedPlayState"]["locks"][0].clone(); 65])
            }
        }
        assert!(e.load_state(&snapshot).is_err(), "alteration {alteration}");
    }
    for marker in [Value::Null, json!(false), json!(true)] {
        let mut snapshot = original.clone();
        snapshot["namedPlayPending"] = marker;
        snapshot["actor"] = json!("p1");
        snapshot["winner"] = json!("p1");
        assert!(e
            .load_state(&snapshot)
            .unwrap_err()
            .contains("pending card naming"));
    }
}
#[test]
fn named_play_configuration_empty_single_auto_and_registry_binding() {
    for count in [0, 1] {
        let mut e = restricted_engine(true);
        e.set_legal_card_titles(&if count == 0 {
            vec![]
        } else {
            vec!["Commander Pyre".into()]
        })
        .unwrap();
        let mut s = board(&e);
        s.named_play = e.new_named_play_state().unwrap();
        let source = add(&e, &mut s, Seat::P1, "SOR_062");
        e.add_arena(&mut s, source).unwrap();
        s.source = source;
        let p = e.def(&s, source).programs.card[0];
        // The native dropdown registration is legal even with an empty list;
        // it resolves without selection rather than disappearing from a window.
        assert!(e.program_has_effect(&s, source, p));
        e.begin_program(&mut s, p, ProgramContinuation::Triggers)
            .unwrap();
        assert_ne!(s.decision, Decision::AbilityTarget);
        assert_eq!(s.named_play.len, count);
    }
    let mut e = restricted_engine(true);
    let (s, _) = locked(&e);
    let snapshot = e.snapshot(&s);
    e.set_legal_card_titles(&["Luke Skywalker".into()]).unwrap();
    assert!(e.load_state(&snapshot).unwrap_err().contains("registry"));
    assert!(e
        .apply(&s, &Action::button("Pass", "pass"))
        .unwrap_err()
        .contains("registry"));
}
#[test]
fn named_play_ledger_capacity_is_atomic_and_state_hash_covers_captured_values() {
    let e = restricted_engine(true);
    let (mut s, source) = locked(&e);
    s.source = source;
    s.active = Seat::P1;
    let lock = s.named_play.locks[0];
    s.named_play.locks.fill(lock);
    s.named_play.len = MAX_NAMED_PLAY_LOCKS as u8;
    let before = s;
    assert!(e
        .install_named_play(&mut s, lock.title)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(s, before);
    for variation in 0..4 {
        let mut changed = s;
        match variation {
            0 => changed.named_play.locks[0].title += 1,
            1 => changed.named_play.locks[0].recipient = Seat::P1,
            2 => changed.named_play.locks[0].source += 1,
            _ => changed.named_play.registry[0] ^= 1,
        }
        assert_ne!(changed.zobrist(), s.zobrist());
    }
}
#[test]
fn named_play_blocks_units_events_upgrades_and_smuggle_before_payment() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for code in ["SHD_160", "SHD_252", "IC27_052"] {
            for resource in [false, true] {
                let mut s = board(&e);
                s.active = seat;
                s.action_player = Some(seat);
                let source = add(&e, &mut s, seat.other(), "SOR_062");
                e.add_arena(&mut s, source).unwrap();
                let target = add(&e, &mut s, seat, code);
                if resource && e.def(&s, target).smuggle.is_none() {
                    continue;
                }
                for _ in 0..20 {
                    let id = add(&e, &mut s, seat, "LOF_254");
                    s.player_mut(seat).resources.push(id).unwrap();
                }
                let host = add(&e, &mut s, seat, "LOF_254");
                e.add_arena(&mut s, host).unwrap();
                if resource {
                    s.player_mut(seat).resources.push(target).unwrap();
                } else {
                    s.player_mut(seat).hand.push(target).unwrap();
                }
                s.source = source;
                s.active = seat.other();
                let title = e
                    .naming_options()
                    .into_iter()
                    .find(|(_, t)| *t == e.def(&s, target).info.title)
                    .unwrap()
                    .0;
                e.install_named_play(&mut s, title).unwrap();
                s.source = 0;
                s.active = seat;
                assert!(
                    !e.legal_actions(&s).contains(&e.card_action(&s, target)),
                    "{code} resource={resource}"
                );
                assert!(e.apply(&s, &e.card_action(&s, target)).is_err());
                let before = s;
                if e.def(&s, target).info.is("unit") {
                    assert!(e.play_unit_with_payment(&mut s, target, resource).is_err());
                } else if e.def(&s, target).info.is("upgrade") {
                    assert!(e.play_upgrade_with_payment(&mut s, target, host).is_err());
                } else {
                    s.source = target;
                    s.ability_program = e.def(&s, target).programs.card[0] + 1;
                    s.ability_committed = false;
                    let event_before = s;
                    assert!(e.commit_program_mode(&mut s, false).is_err());
                    assert_eq!(s, event_before);
                    s = before;
                }
                assert_eq!(s, before, "rejected play must not pay or mutate");
            }
        }
    }
}
