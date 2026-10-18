use super::*;
fn disclose_test_engine(required: &[&str], trigger: &str, leaf: Value) -> Engine {
    let mut c: Value = committed_catalogue().clone();
    let row = c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["setCode"] == "LOF_254")
        .unwrap();
    row["baseClass"] = json!("NonLeaderUnitCard");
    row["abilities"]["card"] = json!([{"trigger":trigger,"title":"Disclose diagnostic","effect":{"kind":"disclose","aspectMultiplicity":true,"selectedCardsEventField":"disclosedCards","triggeredNoMatchPolicy":"maskWithPauseWithoutEvent","selectionContext":"targetPlayer","target":"ownPlayer","aspects":required,"mode":"all"},"ifYouDo":{"title":"Draw diagnostic","effect":leaf}}]);
    let mut e = Engine::from_catalogue_value(&c).unwrap();
    assert!(
        e.by_code.contains_key("LOF_254"),
        "{:?}",
        e.rejected_cards().unwrap().get("LOF_254")
    );
    e.rules_backend = RulesBackend::Interpreter;
    for code in ["ASH_131", "ASH_061"] {
        let index = e.by_code[code] as usize;
        e.definitions[index].aspects = [0; 6];
    }
    e
}
fn disclose_draw() -> Value {
    json!({"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}})
}
#[test]
fn disclose_all_multiset_toggle_limit_acknowledgement_and_copy_hash() {
    for seat in [Seat::P1, Seat::P2] {
        let mut e =
            disclose_test_engine(&["vigilance", "vigilance"], "whenPlayed", disclose_draw());
        let def = e.by_code["ASH_131"];
        e.definitions[def as usize].aspects[0] = 1;
        let mut s = board(&e);
        s.active = seat;
        let source = unit(&e, &mut s, seat, "LOF_254");
        let first = add(&e, &mut s, seat, "ASH_131");
        let second = add(&e, &mut s, seat, "ASH_131");
        let surplus = add(&e, &mut s, seat, "ASH_131");
        let wrong = add(&e, &mut s, seat, "ASH_061");
        for id in [first, second, surplus, wrong] {
            s.player_mut(seat).hand.push(id).unwrap();
        }
        let draw = add(&e, &mut s, seat, "ASH_061");
        s.player_mut(seat).deck.push(draw).unwrap();
        s.source = source;
        let p = e.card_programs(&s, source)[0];
        e.begin_program(&mut s, p, ProgramContinuation::Finish)
            .unwrap();
        assert_eq!(s.decision, Decision::DiscloseSelect);
        assert!(e
            .legal_actions(&s)
            .contains(&Action::button("Choose nothing", "done")));
        assert_eq!(s.actor(), Some(seat));
        assert!(!e
            .legal_actions(&s)
            .iter()
            .any(|a| a.id() == Some(wrong) || a.label == "Done"));
        let empty_hash = s.zobrist();
        s = e.apply(&s, &e.card_action(&s, first)).unwrap();
        assert_ne!(s.zobrist(), empty_hash);
        assert_eq!(s.disclose.selected, Selection::one(first));
        assert!(!e.legal_actions(&s).iter().any(|a| a.label == "Done"));
        s = e.apply(&s, &e.card_action(&s, second)).unwrap();
        assert_eq!(s.decision, Decision::DiscloseSelect);
        assert!(e.legal_actions(&s).iter().any(|a| a.label == "Done"));
        assert!(!e.legal_actions(&s).iter().any(|a| a.id() == Some(surplus)));
        let copy = s;
        assert_eq!(copy, s);
        assert_eq!(copy.zobrist(), s.zobrist());
        s = e.apply(&s, &e.card_action(&s, first)).unwrap();
        assert_eq!(s.disclose.selected, Selection::one(second));
        assert!(!e.legal_actions(&s).iter().any(|a| a.label == "Done"));
        s = e.apply(&s, &e.card_action(&s, first)).unwrap();
        let done = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Done")
            .unwrap();
        s = e.apply(&s, &done).unwrap();
        assert_eq!(s.decision, Decision::DiscloseReveal);
        assert_eq!(s.actor(), Some(seat.other()));
        assert_eq!(e.legal_actions(&s), vec![Action::button("Done", "done")]);
        assert_eq!(
            s.player(seat).hand.as_slice(),
            &[first, second, surplus, wrong]
        );
        assert!(!s.player(seat).hand.contains(draw));
        s = e.apply(&s, &Action::button("Done", "done")).unwrap();
        assert!(s.player(seat).hand.contains(draw));
        assert_eq!(s.disclose, DiscloseState::default());
        assert_eq!(s.decision, Decision::Window);
    }
}
#[test]
fn disclose_all_double_icons_single_card_autocompletes_without_done() {
    for aspects in [vec!["vigilance", "vigilance"], vec!["vigilance", "command"]] {
        let mut e = disclose_test_engine(&aspects, "whenPlayed", disclose_draw());
        let def = e.by_code["ASH_131"];
        for name in &aspects {
            e.definitions[def as usize].aspects[aspect(name).unwrap()] += 1;
        }
        let mut s = board(&e);
        let source = unit(&e, &mut s, Seat::P1, "LOF_254");
        let donor = add(&e, &mut s, Seat::P1, "ASH_131");
        s.players[0].hand.push(donor).unwrap();
        s.source = source;
        let program = e.card_programs(&s, source)[0];
        e.begin_program(&mut s, program, ProgramContinuation::Finish)
            .unwrap();
        let out = e.apply(&s, &e.card_action(&s, donor)).unwrap();
        assert_eq!(out.decision, Decision::DiscloseReveal);
        assert_eq!(out.disclose.selected, Selection::one(donor));
        assert_eq!(out.active, Seat::P2);
    }
}
#[test]
fn disclose_all_impossible_trigger_masks_and_decline_never_runs_continuation() {
    let mut e = disclose_test_engine(&["vigilance"], "whenDefeated", disclose_draw());
    let mut s = board(&e);
    let source = add(&e, &mut s, Seat::P2, "LOF_254");
    s.players[1].discard.push(source).unwrap();
    let draw = add(&e, &mut s, Seat::P2, "ASH_061");
    s.players[1].deck.push(draw).unwrap();
    s.source = source;
    s.active = Seat::P2;
    s.ability_succeeded = true;
    s.ability_result = 42;
    let program = e.card_programs(&s, source)[0];
    e.begin_program(&mut s, program, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::DisclosePause);
    assert_eq!(e.legal_actions(&s), vec![Action::button("Skip", "done")]);
    let out = e.apply(&s, &Action::button("Skip", "done")).unwrap();
    assert!(!out.players[1].hand.contains(draw));
    assert!(out.players[1].discard.contains(source));
    assert_eq!(out.decision, Decision::Window);
    let def = e.by_code["ASH_131"];
    e.definitions[def as usize].aspects[0] = 1;
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let donor = add(&e, &mut s, Seat::P1, "ASH_131");
    s.players[0].hand.push(donor).unwrap();
    let draw = add(&e, &mut s, Seat::P1, "ASH_061");
    s.players[0].deck.push(draw).unwrap();
    s.source = source;
    let program = e.card_programs(&s, source)[0];
    e.begin_program(&mut s, program, ProgramContinuation::Finish)
        .unwrap();
    let out = e.apply(&s, &Action::button("Cancel", "cancel")).unwrap();
    assert!(out.players[0].hand.contains(donor));
    assert!(!out.players[0].hand.contains(draw));
    assert_eq!(out.decision, Decision::Window);
}
#[test]
fn disclose_all_event_impossible_uses_normal_confirmation_without_masking() {
    let mut c: Value = committed_catalogue().clone();
    let row = c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["setCode"] == "ASH_067")
        .unwrap();
    row["abilities"]["card"] = json!([{"trigger":"event","title":"Disclose diagnostic event","effect":{"kind":"disclose","aspectMultiplicity":true,"selectedCardsEventField":"disclosedCards","triggeredNoMatchPolicy":"maskWithPauseWithoutEvent","selectionContext":"targetPlayer","target":"ownPlayer","aspects":["vigilance","vigilance","vigilance"],"mode":"all"},"ifYouDo":{"title":"Draw diagnostic","effect":disclose_draw()}}]);
    let e = Engine::from_catalogue_value(&c).unwrap();
    assert!(
        e.by_code.contains_key("ASH_067"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_067")
    );
    let mut s = board(&e);
    let event = hand(&e, &mut s, "ASH_067");
    let draw = add(&e, &mut s, Seat::P1, "ASH_131");
    s.players[0].deck.push(draw).unwrap();
    s.source = event;
    let program = e.card_programs(&s, event)[0];
    e.begin_program(&mut s, program, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityNoEffect);
    let actions = e.legal_actions(&s);
    let use_it = actions.into_iter().find(|a| a.label != "Cancel").unwrap();
    let out = e.apply(&s, &use_it).unwrap();
    assert_eq!(out.decision, Decision::Window);
    assert!(out.players[0].discard.contains(event));
    assert!(!out.players[0].hand.contains(draw));
    assert_eq!(out.disclose, DiscloseState::default());
}
#[test]
fn disclose_all_raw_ir_modes_fields_continuations_and_source_metadata_fail_closed() {
    let base = json!({"trigger":"whenPlayed","title":"bounded Disclose","effect":{"kind":"disclose","aspectMultiplicity":true,"selectedCardsEventField":"disclosedCards","triggeredNoMatchPolicy":"maskWithPauseWithoutEvent","selectionContext":"targetPlayer","target":"ownPlayer","aspects":["vigilance","vigilance"],"mode":"all"},"ifYouDo":{"title":"Draw","effect":disclose_draw()}});
    let compile = |v: Value| {
        serde_json::from_value::<crate::ability_ir::Ability>(v)
            .ok()
            .is_some_and(|a| crate::ability_ir::Program::compile(&a).is_ok())
    };
    assert!(compile(base.clone()));
    for (field, value) in [
        ("aspects", json!([])),
        (
            "aspects",
            json!(["vigilance", "vigilance", "vigilance", "vigilance"]),
        ),
        ("aspects", json!(["neutral"])),
        ("mode", json!("any")),
        ("mode", json!("some")),
        ("target", json!("enemyPlayer")),
        ("controller", json!("opponent")),
    ] {
        let mut bad = base.clone();
        bad["effect"][field] = value;
        assert!(!compile(bad), "effect field {field}");
    }
    for field in ["optional", "coordinate", "requiresConfirmation"] {
        let mut bad = base.clone();
        bad[field] = json!(true);
        assert!(!compile(bad), "root {field}");
    }
    let mut bad = base.clone();
    bad["trigger"] = json!("action");
    assert!(!compile(bad));
    let mut bad = base.clone();
    bad["ifYouDo"]["effect"]["amount"] =
        json!({"kind":"ref","path":["event","disclosedCards","length"]});
    assert!(!compile(bad));
    let mut bad = base.clone();
    bad["then"] = bad["ifYouDo"].clone();
    assert!(!compile(bad));
    let mut bad = base.clone();
    bad["ifYouDo"]["targetResolver"] = json!({"owner":"any","type":"any","zone":"anyAttackable","mode":"upTo","count":{"kind":"const","value":2},"optional":true});
    bad["ifYouDo"]["effect"] =
        json!({"kind":"giveExperience","target":"chosen","amount":{"kind":"const","value":1}});
    assert!(!compile(bad));
    let original: Value = committed_catalogue().clone();
    for class in ["LeaderUnitCard", "UpgradeCard", "TokenUnitCard"] {
        let mut c = original.clone();
        let row = c["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["setCode"] == "LOF_254")
            .unwrap();
        row["baseClass"] = json!(class);
        row["abilities"]["card"] = json!([base.clone()]);
        assert!(
            !Engine::from_catalogue_value(&c)
                .unwrap()
                .by_code
                .contains_key("LOF_254"),
            "{class}"
        );
    }
}
#[test]
fn disclose_all_event_first_spy_survives_impossible_or_declined_second_spy() {
    for decline_mode in 0..3 {
        let decline = decline_mode != 0;
        let mut c: Value = committed_catalogue().clone();
        let row = c["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["setCode"] == "ASH_067")
            .unwrap();
        row["abilities"]["card"] = json!([{"trigger":"event","title":"Spy then Disclose","effect":{"kind":"createTokenUnit","tokenType":"spy","target":"ownPlayer","amount":{"kind":"const","value":1},"entersReady":false},"then":{"title":"Disclose for another Spy","effect":{"kind":"disclose","aspectMultiplicity":true,"selectedCardsEventField":"disclosedCards","triggeredNoMatchPolicy":"maskWithPauseWithoutEvent","selectionContext":"targetPlayer","target":"ownPlayer","aspects":["aggression"],"mode":"all"},"ifYouDo":{"title":"Create another Spy","effect":{"kind":"createTokenUnit","tokenType":"spy","target":"ownPlayer","amount":{"kind":"const","value":1},"entersReady":false}}}}]);
        let mut e = Engine::from_catalogue_value(&c).unwrap();
        assert!(
            e.by_code.contains_key("ASH_067"),
            "{:?}",
            e.rejected_cards().unwrap().get("ASH_067")
        );
        e.rules_backend = RulesBackend::Interpreter;
        let def = e.by_code["ASH_131"];
        e.definitions[def as usize].aspects = [0; 6];
        e.definitions[def as usize].aspects[2] = 1;
        let mut s = board(&e);
        let event = hand(&e, &mut s, "ASH_067");
        if decline {
            let donor = add(&e, &mut s, Seat::P1, "ASH_131");
            s.players[0].hand.push(donor).unwrap();
        }
        let mut out = e.apply(&s, &e.card_action(&s, event)).unwrap();
        if decline {
            assert_eq!(out.decision, Decision::DiscloseSelect);
            out = e
                .apply(
                    &out,
                    &if decline_mode == 1 {
                        Action::button("Cancel", "cancel")
                    } else {
                        Action::button("Choose nothing", "done")
                    },
                )
                .unwrap();
        }
        assert_eq!(out.decision, Decision::Window);
        assert!(out.players[0].discard.contains(event));
        assert_eq!(out.players[0].ground.len, 1);
        let spy = out.players[0].ground.as_slice()[0];
        assert_eq!(e.def(&out, spy).code, "TOKEN_SPY");
        assert!(out.card(spy).exhausted);
        assert_eq!(out.disclose, DiscloseState::default());
    }
}

#[test]
fn disclose_real_catalogue_keeps_reviewed_sources_without_admitting_extended_modes() {
    let e = engine();
    for code in [
        "SEC_059", "SEC_062", "SEC_076", "SEC_094", "SEC_098", "SEC_120", "SEC_141", "SEC_148",
        "SEC_153", "SEC_181", "SEC_182", "SEC_190", "SEC_230", "SEC_248",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    let original = committed_catalogue();
    for row in original["cards"].as_array().unwrap() {
        if row["abilities"]
            .to_string()
            .contains("\"kind\":\"disclose\"")
            && ![
                "SEC_059", "SEC_062", "SEC_076", "SEC_094", "SEC_098", "SEC_120", "SEC_141",
                "SEC_148", "SEC_153", "SEC_181", "SEC_182", "SEC_190", "SEC_230", "SEC_248",
            ]
            .contains(&row["setCode"].as_str().unwrap())
        {
            assert!(
                !e.by_code.contains_key(row["setCode"].as_str().unwrap()),
                "extended disclose unexpectedly admitted: {}",
                row["setCode"]
            );
        }
    }
    eprintln!("restored catalogue admits {} cards", e.definitions.len());
}

#[test]
fn disclose_real_ornithopter_cannot_reveal_itself_and_draw_waits_for_opponent() {
    let e = engine();
    let mut s = board(&e);
    let source = hand(&e, &mut s, "SEC_062");
    let draw = add(&e, &mut s, Seat::P1, "ASH_131");
    s.players[0].deck.push(draw).unwrap();
    let out = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(out.decision, Decision::DisclosePause);
    assert!(!out.players[0].hand.contains(source));
    assert!(e.in_arena(&out, source));
    let out = e.apply(&out, &Action::button("Skip", "done")).unwrap();
    assert!(!out.players[0].hand.contains(draw));

    let mut s = board(&e);
    let source = hand(&e, &mut s, "SEC_062");
    let donor = add(&e, &mut s, Seat::P1, "SEC_062");
    s.players[0].hand.push(donor).unwrap();
    let draw = add(&e, &mut s, Seat::P1, "ASH_131");
    s.players[0].deck.push(draw).unwrap();
    let out = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(out.decision, Decision::DiscloseSelect);
    assert_eq!(
        e.legal_actions(&out)
            .iter()
            .filter_map(Action::id)
            .collect::<Vec<_>>(),
        vec![donor]
    );
    let out = e.apply(&out, &e.card_action(&out, donor)).unwrap();
    assert_eq!(out.actor(), Some(Seat::P2));
    assert!(!out.players[0].hand.contains(draw));
    let out = e.apply(&out, &Action::button("Done", "done")).unwrap();
    assert!(out.players[0].hand.contains(draw));
    assert!(out.players[0].hand.contains(donor));
}

#[test]
fn disclose_real_treason_selects_target_only_after_reveal_and_commits_event_first() {
    let e = engine();
    let mut s = board(&e);
    let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
    let event = hand(&e, &mut s, "SEC_182");
    let a = add(&e, &mut s, Seat::P1, "SEC_182");
    let b = add(&e, &mut s, Seat::P1, "SEC_182");
    for id in [a, b] {
        s.players[0].hand.push(id).unwrap();
    }
    let mut out = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert_eq!(out.decision, Decision::DiscloseSelect);
    assert!(out.players[0].discard.contains(event));
    assert_eq!(out.card(victim).damage, 0);
    for id in [a, b] {
        out = e.apply(&out, &e.card_action(&out, id)).unwrap();
    }
    out = e.apply(&out, &Action::button("Done", "done")).unwrap();
    assert_eq!(out.decision, Decision::DiscloseReveal);
    assert_eq!(out.card(victim).damage, 0);
    out = e.apply(&out, &Action::button("Done", "done")).unwrap();
    assert_eq!(out.decision, Decision::AbilityTarget);
    assert!(e.legal_actions(&out).iter().any(|a| a.id() == Some(victim)));
    out = e.apply(&out, &e.card_action(&out, victim)).unwrap();
    assert!(out.players[1].discard.contains(victim));
    assert!(out.players[0].hand.contains(a) && out.players[0].hand.contains(b));
}

#[test]
fn disclose_metadata_and_new_registration_contexts_are_not_silently_ignored() {
    let base = committed_catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["setCode"] == "SEC_062")
        .unwrap()["abilities"]["card"][0]
        .clone();
    let compile = |v: Value| {
        serde_json::from_value::<crate::ability_ir::Ability>(v)
            .ok()
            .is_some_and(|a| crate::ability_ir::Program::compile(&a).is_ok())
    };
    for (field, value) in [
        ("aspectMultiplicity", json!(false)),
        ("selectedCardsEventField", json!("other")),
        ("triggeredNoMatchPolicy", json!("resolveImmediately")),
        ("selectionContext", json!("opponent")),
    ] {
        let mut bad = base.clone();
        bad["effect"][field] = value;
        assert!(!compile(bad), "{field}");
        let mut bad = base.clone();
        bad["effect"].as_object_mut().unwrap().remove(field);
        assert!(!compile(bad), "missing {field}");
    }
    for (field, value) in [
        ("gainedHost", json!(false)),
        ("attachedContext", json!("parent")),
        ("attackerMustSurvive", json!(true)),
    ] {
        let mut bad = base.clone();
        bad[field] = value;
        assert!(!compile(bad.clone()), "root {field}");
        let mut bad = base.clone();
        // Rebuild independently: root must remain a valid ordinary registration.
        bad["ifYouDo"][field] = match field {
            "gainedHost" => json!(false),
            "attachedContext" => json!("parent"),
            _ => json!(true),
        };
        assert!(!compile(bad), "continuation {field}");
    }
    let mut catalogue = committed_catalogue().clone();
    let row = catalogue["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["setCode"] == "SEC_062")
        .unwrap();
    row["privatePrograms"] = json!([{"unreviewed":true}]);
    assert!(!Engine::from_catalogue_value(&catalogue)
        .unwrap()
        .by_code
        .contains_key("SEC_062"));
}

#[test]
fn disclose_state_preserves_inline_layout_and_clear_action_boundaries() {
    eprintln!(
        "Disclose inline State bytes={}",
        std::mem::size_of::<State>()
    );
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    let e = engine();
    let mut s = board(&e);
    s.disclose.player = Seat::P2;
    s.disclose.selected = Selection::one(1);
    e.cancel(&mut s);
    assert_eq!(s.disclose, DiscloseState::default());
}

#[test]
fn disclose_conversion_contexts_and_mixed_source_types_fail_closed() {
    let catalogue = committed_catalogue();
    let original = catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["setCode"] == "SEC_062")
        .unwrap()["abilities"]["card"][0]
        .clone();
    for field in ["ifYouDoCondition"] {
        for child in [false, true] {
            let mut value = original.clone();
            if child {
                value["ifYouDo"][field] = json!({"kind":"eventResult"});
            } else {
                value[field] = json!({"kind":"eventResult"});
            }
            let ability: crate::ability_ir::Ability = serde_json::from_value(value).unwrap();
            assert!(crate::ability_ir::Program::compile(&ability).is_err());
        }
    }
    for flag in [0, 1] {
        let mut ability: crate::ability_ir::Ability =
            serde_json::from_value(original.clone()).unwrap();
        if flag == 0 {
            ability.piloting_ability = true;
        } else {
            ability.pilot_unit_ability = true;
        }
        assert!(crate::ability_ir::Program::compile(&ability).is_err());
    }
    for types in [
        json!(["unit", "leader"]),
        json!(["unit", "token"]),
        json!(["unit", "upgrade"]),
    ] {
        let mut value = catalogue.clone();
        let row = value["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["setCode"] == "SEC_062")
            .unwrap();
        row["card"]["types"] = types;
        assert!(!Engine::from_catalogue_value(&value)
            .unwrap()
            .by_code
            .contains_key("SEC_062"));
    }
}

#[test]
fn disclose_pending_snapshot_and_invalid_clicks_fail_closed() {
    let mut e = disclose_test_engine(&["vigilance", "vigilance"], "whenPlayed", disclose_draw());
    let definition = e.by_code["ASH_131"];
    e.definitions[definition as usize].aspects[0] = 1;
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let first = add(&e, &mut s, Seat::P1, "ASH_131");
    let second = add(&e, &mut s, Seat::P1, "ASH_131");
    let wrong = add(&e, &mut s, Seat::P1, "ASH_061");
    for id in [first, second, wrong] {
        s.players[0].hand.push(id).unwrap();
    }
    s.source = source;
    let p = e.card_programs(&s, source)[0];
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["disclosePending"], json!(true));
    assert!(e
        .load_state(&snapshot)
        .unwrap_err()
        .contains("pending Disclose"));
    let before = s;
    assert!(e
        .apply_disclose_action(&s, &e.card_action(&s, wrong))
        .is_err());
    assert!(e
        .apply_disclose_action(&s, &Action::button("Done", "done"))
        .is_err());
    assert_eq!(s, before);
    let selected = e.apply(&s, &e.card_action(&s, first)).unwrap();
    assert!(e
        .apply_disclose_action(&selected, &Action::button("Done", "done"))
        .is_err());
    let second_selected = e
        .apply(&selected, &e.card_action(&selected, second))
        .unwrap();
    let complete = e
        .apply(&second_selected, &Action::button("Done", "done"))
        .unwrap();
    assert!(e
        .apply_disclose_action(&complete, &e.card_action(&complete, first))
        .is_err());
    let snapshot = e.snapshot(&complete);
    assert!(e
        .load_state(&snapshot)
        .unwrap_err()
        .contains("pending Disclose"));
}

#[test]
fn disclose_spy_dependency_rejects_absent_or_malformed_native_token() {
    for malformed in [false, true] {
        let mut catalogue = committed_catalogue().clone();
        let rows = catalogue["cards"].as_array_mut().unwrap();
        if malformed {
            rows.iter_mut()
                .find(|r| r["setCode"] == "TOKEN_SPY")
                .unwrap()["card"]["traits"] = json!(["trooper"]);
        } else {
            rows.retain(|r| r["setCode"] != "TOKEN_SPY");
        }
        let e = Engine::from_catalogue_value(&catalogue).unwrap();
        for code in ["SEC_141", "SEC_148", "SEC_181"] {
            assert!(!e.by_code.contains_key(code), "{code}");
        }
        assert!(e.by_code.contains_key("SEC_062"));
    }
}

#[test]
fn disclose_real_spies_preserve_native_traits_readiness_and_physical_identity() {
    for (code, count, ready, aspects) in [
        ("SEC_141", 3, false, [0, 0, 2, 0, 0, 1]),
        ("SEC_148", 1, true, [0, 0, 1, 0, 1, 0]),
    ] {
        let mut e = engine();
        e.rules_backend = RulesBackend::Interpreter;
        let donor_definition = e.by_code["ASH_131"];
        e.definitions[donor_definition as usize].aspects = aspects;
        let mut s = board(&e);
        let source = add(&e, &mut s, Seat::P1, code);
        if code == "SEC_148" {
            s.players[0].discard.push(source).unwrap();
        } else {
            e.add_arena(&mut s, source).unwrap();
        }
        let donor = add(&e, &mut s, Seat::P1, "ASH_131");
        s.players[0].hand.push(donor).unwrap();
        s.source = source;
        let program = e.card_programs(&s, source)[0];
        e.begin_program(&mut s, program, ProgramContinuation::Finish)
            .unwrap();
        let reveal = e.apply(&s, &e.card_action(&s, donor)).unwrap();
        assert_eq!(reveal.decision, Decision::DiscloseReveal);
        let next = reveal.next_id;
        let out = e.apply(&reveal, &Action::button("Done", "done")).unwrap();
        assert_eq!(out.next_id, next + count);
        let spies = out.players[0]
            .ground
            .as_slice()
            .iter()
            .copied()
            .filter(|id| e.def(&out, *id).code == "TOKEN_SPY")
            .collect::<Vec<_>>();
        assert_eq!(spies, (next..next + count).collect::<Vec<_>>());
        for id in spies {
            assert_eq!(out.card(id).exhausted, !ready);
            assert_eq!(e.keyword_amount(&out, id, "raid"), 2);
        }
        assert!(out.players[0].hand.contains(donor));
        if code == "SEC_148" {
            assert!(out.players[0].discard.contains(source));
        }
    }
}
