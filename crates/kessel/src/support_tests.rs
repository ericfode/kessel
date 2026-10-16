// Source-derived expectations. These are Rust-only legal midgame witnesses, not
// forceteki executions or differential recordings.
use super::*;
fn start(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn play(e: &Engine, s: &State, seat: Seat, code: &str) -> (State, CardId) {
    let mut s = *s;
    let id = add(e, &mut s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    for _ in 0..10 {
        let r = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    let s = e.load_state(&e.snapshot(&s)).unwrap();
    let a = e.card_action(&s, id);
    assert!(e.legal_actions(&s).contains(&a));
    (e.apply(&s, &a).unwrap(), id)
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "missing card {id} in {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn roundtrip(e: &Engine, s: &State) -> State {
    let snap = e.snapshot(s);
    let t = e.load_state(&snap).unwrap();
    assert_eq!(e.snapshot(&t), snap);
    assert_eq!(e.legal_actions(&t), e.legal_actions(s));
    assert_eq!(s.zobrist(), t.zobrist());
    t
}
#[test]
fn support_plain_family_copies_keywords_and_removes_after_actual_attack() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for (code, kw, amount) in [
            ("ASH_074", "grit", 1),
            ("ASH_095", "restore", 1),
            ("ASH_121", "overwhelm", 1),
            ("ASH_130", "support", 0),
            ("ASH_154", "raid", 1),
            ("ASH_215", "support", 0),
            ("ASH_222", "saboteur", 1),
        ] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            let base = s.player(seat.other()).base;
            s.card_mut(s.player(seat).base).damage = 5;
            let (s, donor) = play(&e, &s, seat, code);
            assert_eq!(s.decision, Decision::Support, "{code}");
            assert!(s.card(donor).exhausted);
            assert_eq!(
                e.legal_actions(&s)
                    .iter()
                    .filter_map(Action::id)
                    .collect::<Vec<_>>(),
                vec![attacker]
            );
            let s = click(&e, &s, attacker);
            assert!(s.support.pending);
            assert_eq!(s.source, attacker);
            assert_eq!(e.keyword_amount(&s, attacker, kw), amount, "{code}");
            assert_eq!(e.keyword_amount(&s, attacker, "support"), 0);
            assert!(e
                .load_state(&e.snapshot(&s))
                .unwrap_err()
                .contains("pending Support"));
            let done = click(&e, &s, base);
            assert_eq!(done.support.len, 0);
            assert!(!done.support.pending);
            assert!(done.card(attacker).exhausted);
            assert!(done.card(donor).exhausted);
            assert_eq!(
                done.card(base).damage,
                if kw == "raid" { 2 } else { 1 },
                "{code}"
            );
            assert_eq!(
                done.card(done.player(seat).base).damage,
                if kw == "restore" { 4 } else { 5 }
            );
            assert_eq!(done.active, seat.other());
            roundtrip(&e, &done);
        }
    }
}
#[test]
fn support_passes_persist_stack_and_any_later_attack_cleans_all_grants() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = start(&e, seat);
        let attacker = unit(&e, &mut s, seat, "LOF_256");
        let other = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
        let base = s.player(seat.other()).base;
        let (s, donor) = play(&e, &s, seat, "ASH_154");
        let choosing = click(&e, &s, attacker);
        assert_eq!(
            e.legal_actions(&choosing)
                .iter()
                .filter(|a| a.kind == "button")
                .map(|a| (a.label.as_str(), a.arg.clone()))
                .collect::<Vec<_>>(),
            vec![
                ("Cancel", Some(json!("cancel"))),
                ("Pass attack", Some(json!("passAbility")))
            ]
        );
        let passed = named_button(&e, &choosing, "Pass attack");
        assert_eq!(passed.support.len, 1);
        assert!(!passed.support.pending);
        assert!(!passed.card(attacker).exhausted);
        assert_eq!(e.keyword_amount(&passed, attacker, "raid"), 1);
        let mut passed = roundtrip(&e, &passed);
        e.leave_play_primary(&mut passed, donor).unwrap();
        assert_eq!(
            e.keyword_amount(&passed, attacker, "raid"),
            1,
            "donor departure retains snapshot"
        );
        // Real opponent action gives the Support player the next action.
        let passed = named_button(&e, &passed, "Pass");
        let (again, _) = play(&e, &passed, seat, "ASH_154");
        let again = click(&e, &again, attacker);
        let again = named_button(&e, &again, "Cancel");
        assert_eq!(again.support.len, 2);
        assert_eq!(e.keyword_amount(&again, attacker, "raid"), 2);
        roundtrip(&e, &again);
        let passed = named_button(&e, &again, "Pass");
        let select = click(&e, &passed, other);
        let done = click(&e, &select, base);
        assert_eq!(done.card(base).damage, 1);
        assert_eq!(done.support.len, 0);
        assert_eq!(e.keyword_amount(&done, attacker, "raid"), 0);
        assert!(e.in_arena(&done, enemy));
    }
}
#[test]
fn support_target_domain_decline_and_source_exclusion_are_exact() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = start(&e, seat);
        let ready = unit(&e, &mut s, seat, "LOF_256");
        let exhausted = unit(&e, &mut s, seat, "LOF_256");
        s.card_mut(exhausted).exhausted = true;
        unit(&e, &mut s, seat.other(), "LOF_254");
        let (s, donor) = play(&e, &s, seat, "ASH_130");
        assert_eq!(
            e.legal_actions(&s)
                .iter()
                .filter_map(Action::id)
                .collect::<Vec<_>>(),
            vec![ready]
        );
        assert_eq!(
            e.legal_actions(&s)
                .iter()
                .find(|a| a.kind == "button")
                .unwrap(),
            &Action::button("Pass", "passAbility")
        );
        let done = named_button(&e, &s, "Pass");
        assert_eq!(done.support.len, 0);
        assert!(!done.card(ready).exhausted);
        assert!(done.card(donor).exhausted);
        let s = start(&e, seat);
        let (done, _) = play(&e, &s, seat, "ASH_130");
        assert_eq!(
            done.decision,
            Decision::Window,
            "no candidate skips optional Support"
        );
    }
}
#[test]
fn support_marrok_copied_constant_rebinds_source_and_tracks_actual_defender() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for damage in [0, 1] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
            s.card_mut(enemy).damage = damage;
            let (s, donor) = play(&e, &s, seat, "ASH_241");
            let select = click(&e, &s, attacker);
            assert_eq!(e.power(&select, attacker, false), 1);
            let done = click(&e, &select, enemy);
            assert_eq!(done.card(attacker).damage, 1);
            assert_eq!(done.support.len, 0);
            assert!(e.in_arena(&done, donor));
            if damage == 1 {
                assert!(done.player(seat.other()).discard.contains(enemy));
                assert_eq!(done.card(done.player(seat.other()).base).damage, 0);
            } else {
                assert_eq!(done.card(enemy).damage, 1);
            }
        }
    }
}
#[test]
fn support_lothcat_suppresses_registration_but_never_has_intrinsic_support() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = start(&e, seat);
        let cat = unit(&e, &mut s, seat.other(), "ASH_068");
        let attacker = unit(&e, &mut s, seat, "LOF_256");
        assert!(!e.has_keyword(&s, cat, "support"));
        let (done, donor) = play(&e, &s, seat, "ASH_130");
        assert_eq!(done.decision, Decision::Window);
        assert_eq!(e.keyword_amount(&done, donor, "support"), 0);
        assert!(!done.card(attacker).exhausted);
        let mut done = done;
        e.leave_play_primary(&mut done, cat).unwrap();
        assert_eq!(e.keyword_amount(&done, donor, "support"), 1);
        assert_eq!(
            done.decision,
            Decision::Window,
            "later gain doesn't create entry trigger"
        );
    }
}
#[test]
fn support_source_onattack_copies_context_and_continuation_uses_attacker() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = start(&e, seat);
        let attacker = unit(&e, &mut s, seat, "LOF_254");
        let base = s.player(seat.other()).base;
        s.card_mut(s.player(seat).base).damage = 4;
        let (s, donor) = play(&e, &s, seat, "ASH_059");
        let select = click(&e, &s, attacker);
        let triggered = click(&e, &select, base);
        assert_eq!(triggered.decision, Decision::AbilityOptional);
        let done = named_button(&e, &triggered, "Trigger");
        assert!(done.player(seat).discard.contains(attacker));
        assert_eq!(done.card(attacker).damage, 0);
        assert_eq!(done.card(donor).damage, 0);
        assert_eq!(done.card(done.player(seat).base).damage, 2);
        assert_eq!(done.support.len, 0);
    }
}
#[test]
fn support_saboteur_is_present_before_defender_legality_and_survives_donor_keyword_loss() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let sentinel = unit(&e, &mut s, Seat::P2, "SOR_063");
    let other = unit(&e, &mut s, Seat::P2, "LOF_254");
    let (s, donor) = play(&e, &s, Seat::P1, "ASH_222");
    let mut select = click(&e, &s, attacker);
    assert!(e
        .legal_actions(&select)
        .iter()
        .filter_map(Action::id)
        .any(|id| id == other));
    assert!(e
        .attack_targets(&select, attacker, false)
        .contains(&select.player(Seat::P2).base));
    select.card_mut(donor).add_lost_keyword(2);
    assert_eq!(e.keyword_amount(&select, attacker, "saboteur"), 1);
    assert!(e.in_arena(&select, sentinel));
}
#[test]
fn support_storage_is_copy_hashed_capacity_checked_and_import_is_versioned() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let (s, _) = play(&e, &s, Seat::P1, "ASH_154");
    let choosing = click(&e, &s, attacker);
    let passed = named_button(&e, &choosing, "Pass attack");
    let copy = passed;
    assert_eq!(copy, passed);
    let mut changed = passed;
    changed.support.grants[0].keywords[6] += 1;
    assert_ne!(changed.zobrist(), passed.zobrist());
    changed = passed;
    changed.support.grants[0].programs[0] += 1;
    changed.support.grants[0].len = 1;
    assert_ne!(changed.zobrist(), passed.zobrist());
    let snap = e.snapshot(&passed);
    let mut bad = snap.clone();
    bad.as_object_mut().unwrap().remove("supportStateVersion");
    assert!(e.load_state(&bad).is_err());
    let mut bad = snap.clone();
    bad["supportStateVersion"] = json!(2);
    assert!(e.load_state(&bad).is_err());
    let mut bad = snap.clone();
    bad["supportGrants"][0]["target"] = bad["supportGrants"][0]["donor"].clone();
    assert!(e.load_state(&bad).is_err());
    let mut bad = snap.clone();
    bad["supportGrants"][0]["presence"] = json!(512);
    assert!(e.load_state(&bad).is_err());
    let mut bad = snap.clone();
    bad["supportGrants"][0]["keywords"][0] = json!(2);
    assert!(e.load_state(&bad).is_err());
    let mut full = choosing;
    full.support.pending = false;
    full.support.len = MAX_SUPPORT_GRANTS as u8;
    let before = full;
    assert!(e
        .begin_support_attack(&mut full, attacker)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(full, before);
    let mut gone = passed;
    e.leave_play_primary(&mut gone, attacker).unwrap();
    assert_eq!(gone.support.len, 0);
    assert_eq!(
        gone.support.grants,
        [SupportGrant::default(); MAX_SUPPORT_GRANTS]
    );
    eprintln!(
        "Support State={} SupportState={} Card={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<SupportState>(),
        std::mem::size_of::<Card>()
    );
}
#[test]
fn support_admission_rejects_unsupported_donors_and_arbitrary_grants() {
    let e = engine();
    assert!(
        !e.by_code.contains_key("ASH_050"),
        "source departure WD donor remains bounded rejection"
    );
    let original: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for variant in 0..4 {
        let mut c = original.clone();
        let entry = c["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["setCode"] == "ASH_241")
            .unwrap();
        match variant {0=>entry["abilities"]["card"][0]["trigger"]=json!("whenDefeated"),1=>entry["abilities"]["card"][0]["coordinate"]=json!(true),2=>entry["card"]["types"]=json!(["token","unit"]),_=>entry["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"onAttack","effect":{"kind":"gainKeyword","duration":"phase","target":"self","keyword":"support","amount":{"kind":"const","value":1}}}))};
        assert!(!Engine::from_catalogue_value(&c)
            .unwrap()
            .by_code
            .contains_key("ASH_241"));
    }
}
#[test]
fn support_diagnostic_copies_attached_registration_then_retains_snapshot() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let donor = unit(&e, &mut s, Seat::P1, "ASH_130");
    let upgrade = add(&e, &mut s, Seat::P1, "LOF_139");
    e.attach(&mut s, upgrade, donor);
    let discarded = add(&e, &mut s, Seat::P1, "LOF_254");
    s.players[0].hand.push(discarded).unwrap();
    let mut s = roundtrip(&e, &s);
    // The midgame board is legal. This direct captured-trigger diagnostic isolates
    // attachment-copy semantics, without claiming a canonical play/entry witness.
    s.source = donor;
    s.trigger_context = 1;
    s.pending_triggers = T_SUPPORT;
    e.process_triggers(&mut s).unwrap();
    let mut s = click(&e, &s, attacker);
    e.leave_play_primary(&mut s, upgrade).unwrap();
    let base = s.players[1].base;
    let trigger = click(&e, &s, base);
    assert_eq!(trigger.decision, Decision::AbilityDiscard);
    let done = click(&e, &trigger, discarded);
    assert!(done.players[0].discard.contains(discarded));
    assert_eq!(done.card(base).damage, 1);
    assert_eq!(done.support.len, 0);
}
#[test]
fn support_diagnostic_entry_keywords_share_order_and_outer_window_resumes() {
    let mut c: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let entry = c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "ASH_130")
        .unwrap();
    entry["card"]["text"] = json!("Ambush\nShielded\nSupport");
    entry["card"]["keywords"] = json!(["ambush", "shielded", "support"]);
    entry["keywordValues"] = json!({"ambush":1,"shielded":1,"support":1});
    let e = Engine::from_catalogue_value(&c).unwrap();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let enemy = unit(&e, &mut s, Seat::P2, "ASH_130");
    let base = s.players[1].base;
    let (s, donor) = play(&e, &s, Seat::P1, "ASH_130");
    assert_eq!(s.decision, Decision::TriggerOrder);
    assert_eq!(
        e.legal_actions(&s)
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Ambush", "Shielded", "Support"]
    );
    let s = named_button(&e, &s, "Support");
    let s = click(&e, &s, attacker);
    let s = click(&e, &s, base);
    assert_eq!(s.source, donor);
    assert_eq!(s.decision, Decision::TriggerOrder);
    assert_eq!(s.card(donor).shield, 0);
    let s = named_button(&e, &s, "Shielded");
    assert!(s.card(donor).shield != 0);
    assert_eq!(
        s.card(attacker).shield,
        0,
        "copied Shielded does not make an entry event"
    );
    assert_eq!(s.decision, Decision::Ambush);
    let s = named_button(&e, &s, "Trigger");
    let done = click(&e, &s, enemy);
    assert_eq!(done.support.len, 0);
    assert_eq!(done.decision, Decision::Window);
}
#[test]
fn support_cancellation_history_survives_phase_expiry_until_actual_attack() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let (s, _) = play(&e, &s, Seat::P1, "ASH_154");
    let s = click(&e, &s, attacker);
    let mut s = named_button(&e, &s, "Pass attack");
    // Explicit phase-boundary diagnostic keeps the retained UntilEndOfAttack
    // effect distinct from phase stats; no claim of a full regroup recording.
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.support.len, 1);
    assert_eq!(e.keyword_amount(&s, attacker, "raid"), 1);
}
#[test]
fn support_admission_inventory_is_explicit() {
    let e = engine();
    let mut codes = e
        .definitions
        .iter()
        .filter(|d| d.flags & SUPPORT != 0)
        .map(|d| d.code.as_str())
        .collect::<Vec<_>>();
    codes.sort();
    eprintln!("Support intrinsic admissions: {codes:?}");
    assert!(
        e.by_code.contains_key("ASH_009"),
        "complete live maximum programs and Support dependency are admitted"
    );
}
#[test]
fn support_pershing_remaining_hp_and_gozanti_phase_gain_use_recipient() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for remaining in [2, 3] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            s.card_mut(attacker).damage = 4 - remaining;
            let draw = add(&e, &mut s, seat, "LOF_254");
            s.player_mut(seat).deck.push(draw).unwrap();
            let base = s.player(seat.other()).base;
            let (s, _) = play(&e, &s, seat, "ASH_072");
            let s = click(&e, &s, attacker);
            let done = click(&e, &s, base);
            assert_eq!(done.player(seat).hand.contains(draw), remaining >= 3);
            assert_eq!(done.card(base).damage, 1);
            assert_eq!(done.support.len, 0);
        }
        let mut s = start(&e, seat);
        let attacker = unit(&e, &mut s, seat, "LOF_256");
        let base = s.player(seat.other()).base;
        let (s, donor) = play(&e, &s, seat, "ASH_099");
        let s = click(&e, &s, attacker);
        let done = click(&e, &s, base);
        assert_eq!(e.keyword_amount(&done, attacker, "sentinel"), 1);
        assert_eq!(e.keyword_amount(&done, donor, "sentinel"), 0);
        assert_eq!(done.support.len, 0);
        roundtrip(&e, &done);
    }
}
#[test]
fn support_scion_defender_aura_binds_attacker_and_stat_defeat_retains_attack() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for damage in [0, 3] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            let defender = unit(&e, &mut s, seat.other(), "LOF_256");
            s.card_mut(defender).damage = damage;
            let (s, donor) = play(&e, &s, seat, "ASH_046");
            let s = click(&e, &s, attacker);
            let done = click(&e, &s, defender);
            assert_eq!(
                done.card(attacker).damage,
                0,
                "copied debuff reduces defender power"
            );
            assert_eq!(done.support.len, 0);
            assert!(e.in_arena(&done, donor));
            if damage == 3 {
                assert!(done.player(seat.other()).discard.contains(defender));
            } else {
                assert_eq!(done.card(defender).damage, 1);
                assert_eq!(e.hp(&done, defender), 4, "grant expires after attack");
            }
        }
    }
}
fn support_experience(e: &Engine, s: &mut State, id: CardId) {
    let owner = s.card(id).owner;
    let token = add(e, s, owner, "TOKEN_EXPERIENCE");
    s.card_mut(id).experience = token;
    e.attach(s, token, id);
}
#[test]
fn support_ezra_and_yellow_aces_attachment_condition_reads_recipient() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for upgraded in [false, true] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            let enemy = unit(&e, &mut s, seat.other(), "SOR_130");
            if upgraded {
                support_experience(&e, &mut s, attacker);
            }
            let base = s.player(seat.other()).base;
            let (s, donor) = play(&e, &s, seat, "ASH_209");
            let s = click(&e, &s, attacker);
            let s = click(&e, &s, base);
            let done = if upgraded {
                assert_eq!(s.decision, Decision::AbilityTarget);
                click(&e, &s, enemy)
            } else {
                s
            };
            assert_eq!(e.power(&done, enemy, false), if upgraded { 0 } else { 2 });
            assert!(e.upgrades(&done, donor).is_empty());
            assert_eq!(done.card(base).damage, if upgraded { 2 } else { 1 });
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            if upgraded {
                support_experience(&e, &mut s, attacker);
            }
            let base = s.player(seat.other()).base;
            let (s, _) = play(&e, &s, seat, "ASH_253");
            let s = click(&e, &s, attacker);
            let s = click(&e, &s, base);
            let done = if upgraded {
                assert_eq!(s.decision, Decision::AbilityTarget);
                click(&e, &s, base)
            } else {
                s
            };
            assert_eq!(done.card(base).damage, if upgraded { 4 } else { 1 });
            assert_eq!(done.support.len, 0);
        }
    }
}
#[test]
fn support_captured_entry_survives_source_departure_before_snapshot() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(defender).damage = 1;
    let (mut s, donor) = play(&e, &s, Seat::P1, "ASH_241");
    // Source-departure diagnostic between captured entry and resolution. Printed
    // ability properties still copy, and Source means the chosen attacker.
    e.leave_play_primary(&mut s, donor).unwrap();
    let s = click(&e, &s, attacker);
    let done = click(&e, &s, defender);
    assert!(done.players[1].discard.contains(defender));
    assert!(done.players[0].discard.contains(donor));
    assert_eq!(done.card(attacker).damage, 1);
    assert_eq!(done.support.len, 0);
}
#[test]
fn support_numeric_snapshots_reject_overflow_and_lost_keyword_bits_hash_without_hidden() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let (s, _) = play(&e, &s, Seat::P1, "ASH_154");
    let s = click(&e, &s, attacker);
    let mut s = named_button(&e, &s, "Pass attack");
    let grant = s.support.grants[0];
    s.support.grants[1] = grant;
    s.support.len = 2;
    s.support.grants[0].keywords[6] = 40000;
    s.support.grants[1].keywords[6] = 40000;
    assert!(e
        .check_stat_capacity(&s)
        .unwrap_err()
        .contains("keyword capacity"));
    let mut s = State::default();
    s.next_id = 2;
    s.cards[1].present = true;
    let h = s.zobrist();
    s.cards[1].add_lost_keyword(9);
    assert_ne!(h, s.zobrist());
    assert_eq!(s.cards[1].lost_keyword_mask(), 1 << 9);
    s.cards[1].clear_hidden_phase_effects();
    assert_eq!(h, s.zobrist());
}

// Saved semantic payload is a hashing invariant; this diagnostic does not claim
// every nonzero field is reachable through today's admitted Support donors.
#[test]
fn support_saved_context_hash_includes_event_lki_and_maximum_payload() {
    let mut base=State::default();base.support.pending=true;
    for variant in 0..6 {
        let mut before=base;
        if variant==2 {before.support.context.last_known_source=1;}
        let mut changed=before;
        match variant {
            0=>changed.support.context.event_result.damage_dealt=1,
            1=>changed.support.context.last_known_source=1,
            2=>changed.support.context.last_known_power=3,
            3=>changed.support.context.maximum_capture.power=2,
            4=>changed.support.context.maximum_capture.sole=1,
            _=>changed.support.context.maximum_capture.count=2,
        }
        assert_ne!(before.zobrist(),changed.zobrist(),"saved context field {variant}");
        let mut restored=State::default();changed.support.context.restore(&mut restored);
        assert_eq!(ResolutionContext::capture(&restored),changed.support.context);
    }
}

#[test]
fn support_import_requirement_uses_selected_pool_and_actual_dependencies() {
    let mut e = Engine::from_compiled_pool_bytes(
        include_bytes!("../data/pool.json"),
        include_bytes!("../../../compiler/generated/catalog.json"),
    ).unwrap();
    assert!(e.definitions.iter().any(|d| d.flags & SUPPORT != 0));
    assert!(!e.needs_support_state(), "unselected definitions do not impose state");
    let mut s = start(&e, Seat::P1);
    let snap = e.snapshot(&s);
    assert!(snap.get("supportStateVersion").is_none());
    assert_eq!(e.snapshot(&e.load_state(&snap).unwrap()), snap);
    let mut bad = snap.clone();
    bad["supportGrants"] = json!([]);
    assert!(e.load_state(&bad).unwrap_err().contains("explicit Support"));
    let mut bad = snap.clone();
    bad["supportStateVersion"] = json!(2);
    bad["supportGrants"] = json!([]);
    assert!(e.load_state(&bad).unwrap_err().contains("unsupported supportStateVersion"));

    // An internally assembled out-of-pool donor must impose the same state
    // boundary. Full import separately rejects its unselected identity first.
    unit(&e, &mut s, Seat::P1, "ASH_130");
    assert!(!e.needs_support_state());
    assert!(e.state_needs_support_state(&s));
    let snap = e.snapshot(&s);
    assert_eq!(snap["supportStateVersion"], 1);
    let mut missing = snap.clone();
    missing.as_object_mut().unwrap().remove("supportStateVersion");
    missing.as_object_mut().unwrap().remove("supportGrants");
    assert!(e.load_state(&missing).is_err(), "unselected identity remains rejected");
    let error = e.load_support_state(&mut s, &missing).unwrap_err();
    assert!(error.contains("explicit Support"), "{error}");
    e.load_support_state(&mut s, &snap).unwrap();

    // Selecting a Support donor requires versioned state even before any donor
    // is present. Metadata membership is what from_compiled_pool_bytes filters.
    e.catalogue.as_mut().unwrap().metadata.insert("ASH_130".into(), json!({}));
    assert!(e.needs_support_state());
    assert!(e.load_state(&missing).is_err());
    let mut empty = State::default();
    assert!(e.load_support_state(&mut empty, &json!({})).is_err());
    e.load_support_state(&mut empty, &json!({"supportStateVersion":1,"supportGrants":[]})).unwrap();
}
#[test]
fn support_dependency_detection_includes_nested_grants_and_copy_domains() {
    let mut e = Engine::from_compiled_pool_bytes(
        include_bytes!("../data/pool.json"),
        include_bytes!("../../../compiler/generated/catalog.json"),
    ).unwrap();
    // Dependency diagnostics only: admission still rejects these unreviewed
    // mechanics. Install typed instructions after loading to test the guard
    // independently of today's bounded source admissions.
    let code = "SOR_014";
    let index = e.catalogue.as_ref().unwrap().cards[code].leader[0] as usize;
    let grant: crate::ability_ir::Effect = serde_json::from_value(json!({
        "kind":"conditional","condition":{"kind":"const","value":false},
        "onTrue":{"kind":"sequence","effects":[{"kind":"gainKeyword","target":"self", "keyword":"support", "duration":"phase"}]}
    })).unwrap();
    e.catalogue.as_mut().unwrap().programs[index].instructions.push(crate::ability_ir::Instruction::Apply(grant));
    assert!(e.needs_support_state(), "a false branch still has a lifecycle dependency");
    e.catalogue.as_mut().unwrap().metadata.remove(code);
    assert!(!e.needs_support_state(), "unselected grant sources do not impose state");
    let s = start(&e, Seat::P1);
    assert!(e.state_needs_support_state(&s), "an actual grant source still imposes state");
    let mut empty = State::default();
    assert!(e.load_support_state(&mut empty, &json!({})).is_ok());
    assert!(e.load_support_state(&mut { s }, &json!({})).is_err());
    e.catalogue.as_mut().unwrap().programs[index].instructions.pop();
    let copy: crate::ability_ir::Effect = serde_json::from_value(json!({
        "kind":"copyKeywords","target":"chosen","duration":"persistent", "domain":["grit", "support"]
    })).unwrap();
    e.catalogue.as_mut().unwrap().programs[index].instructions.push(crate::ability_ir::Instruction::Apply(copy));
    e.catalogue.as_mut().unwrap().metadata.insert(code.into(), json!({}));
    assert!(e.needs_support_state(), "a Support-capable copy domain also imposes state");
}
#[test]
fn support_free_selected_pool_preserves_committed_vader_trace() {
    let e = Engine::from_compiled_pool_bytes(
        include_bytes!("../data/pool.json"),
        include_bytes!("../../../compiler/generated/catalog.json"),
    ).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/vader-dual-lethal-2001.jsonl.gz");
    let result = crate::check::check(&e, &path).unwrap();
    assert!(result.matched);
}
