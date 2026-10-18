//! Complete pinned Credit registrations through existing action/declaration paths.
//! Source-derived fixtures, not differential replays or historical-game replicas.
use super::*;

fn entry(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn start_attack(e: &Engine, s: &State, attacker: CardId, defender: CardId) -> State {
    let targets = choose(e, s, attacker);
    assert_eq!(targets.decision, Decision::AttackTarget);
    choose(e, &targets, defender)
}
fn cast_action(e: &Engine, s: &State, source: CardId) -> State {
    let selected = choose(e, s, source);
    assert_eq!(selected.decision, Decision::AbilityMenu);
    button(e, &selected, "Create a Credit token")
}
fn roundtrip(e: &Engine, s: &State) {
    let copied = *s;
    assert_eq!(copied.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&copied), e.legal_actions(s));
    let snapshot = e.snapshot(s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    assert_eq!(loaded.zobrist(), s.zobrist());
}

#[test]
fn credit_trigger_complete_source_contracts_extend_only_the_two_closed_roles() {
    for code in ["LAW_235", "LAW_121"] {
        let original = entry(code);
        let contract = crate::credits::creation_source(&original).unwrap().unwrap();
        assert_eq!(contract.amount, 1);
        assert!(!contract.event);
        // Source shape, not an identity/name allowlist.
        let mut renamed = original.clone();
        renamed["setCode"] = json!("DIAGNOSTIC_CREDIT_TRIGGER");
        renamed["card"]["title"] = json!("Diagnostic complete source");
        assert_eq!(
            crate::credits::creation_source(&renamed).unwrap(),
            Some(contract)
        );
    }
    // No broader admission of other Credit registrations accompanies this change.
    for code in [
        "LAW_071", "LAW_134", "LAW_155", "LAW_161", "LAW_258", "LAW_006",
    ] {
        assert!(
            crate::credits::creation_source(&entry(code)).is_err(),
            "{code}"
        );
    }
}

#[test]
fn credit_trigger_source_guards_reject_cost_timing_side_and_keyword_mutations() {
    for code in ["LAW_235", "LAW_121"] {
        let original = entry(code);
        let mut mutations = vec![];
        for (key, value) in [
            ("optional", json!(true)),
            ("condition", json!({"kind":"const","value":true})),
            ("then", json!({"effect":{"kind":"noAction"}})),
            ("targetResolver", json!({})),
            ("attachedContext", json!({})),
            ("limit", json!({})),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0][key] = value;
            mutations.push(changed);
        }
        for trigger in [
            "onAttack",
            "whenDefeated",
            "whenRegroupStarts",
            "epicAction",
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["trigger"] = json!(trigger);
            mutations.push(changed);
        }
        for costs in [
            json!([{"kind":"resources","amount":{"kind":"const","value":1}}]),
            json!([{"kind":"exhaustSelf"},{"kind":"exhaustSelf"}]),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["costs"] = costs;
            mutations.push(changed);
        }
        let mut wrong_cost = original.clone();
        wrong_cost["abilities"]["card"][0]["costs"] = if code == "LAW_235" {
            json!([])
        } else {
            json!([{"kind":"exhaustSelf"}])
        };
        mutations.push(wrong_cost);
        for (key, value) in [
            ("amount", json!({"kind":"const","value":2})),
            ("target", json!("enemyPlayer")),
            ("destination", json!("ground")),
            ("creationTiming", json!("handler")),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["effect"][key] = value;
            mutations.push(changed);
        }
        for (key, value) in [
            ("keywords", json!(["shielded"])),
            ("text", json!("Create a Credit token.")),
            ("types", json!(["leader", "unit"])),
            ("upgradeHp", json!(0)),
        ] {
            let mut changed = original.clone();
            changed["card"][key] = value;
            mutations.push(changed);
        }
        let mut extra = original.clone();
        extra["abilities"]["unit"] = extra["abilities"]["card"].clone();
        mutations.push(extra);
        let mut extra = original.clone();
        extra["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(json!({"trigger":"constant","effect":{"kind":"noAction"}}));
        mutations.push(extra);
        let mut extra = original.clone();
        extra["stateWatchers"] = json!([{}]);
        mutations.push(extra);
        let mut extra = original.clone();
        extra["baseClass"] = json!("UpgradeCard");
        mutations.push(extra);
        let mut extra = original.clone();
        extra["keywordValues"] = json!({"sentinel":2});
        mutations.push(extra);
        for changed in mutations {
            assert!(
                crate::credits::creation_source(&changed).is_err(),
                "{code}: {changed}"
            );
        }
    }
}

#[test]
fn credit_trigger_dependencies_require_exact_selected_native_credit_definition() {
    let original = catalogue();
    for malformed in [false, true] {
        let mut changed = original.clone();
        if malformed {
            let native = changed["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["setCode"] == "TOKEN_CREDIT")
                .unwrap();
            native["sourceHash"] = json!("different source");
        } else {
            changed["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|e| e["setCode"] != "TOKEN_CREDIT");
        }
        let e = Engine::from_catalogue_value(&changed).unwrap();
        for code in ["LAW_235", "LAW_121"] {
            assert!(!e.by_code.contains_key(code), "{code}");
        }
    }
    let e = engine(RulesBackend::Interpreter);
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_CREDIT");
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&original).unwrap()
    )
    .err()
    .unwrap()
    .contains("TOKEN_CREDIT"));
}

#[test]
fn credit_trigger_exhaust_action_creates_one_without_spending_resources() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let source = unit(e, &mut s, actor, "LAW_235");
        let row = resources(e, &mut s, actor, 2);
        let next = s.next_id;
        let history = s.phase_history;
        let program = e.available_programs(&s, source, "action")[0];
        assert!(e.catalogue.as_ref().unwrap().programs[program as usize]
            .executable
            .is_ok());
        let done = cast_action(e, &s, source);
        assert!(done.card(source).exhausted);
        assert_eq!(live(&done, actor), vec![next]);
        assert!(live(&done, actor.other()).is_empty());
        assert_eq!(done.next_id, next + 1);
        assert_eq!(done.phase_history, history);
        assert_eq!(done.decision, Decision::Window);
        for id in row {
            assert!(!done.card(id).exhausted);
        }
        roundtrip(e, &done);
        // A supplied later midgame readiness state can pay the cost again; the
        // source has no once-per-phase/round limit beyond its exhaust cost.
        let mut again = done;
        again.active = actor;
        again.action_player = Some(actor);
        assert!(!e
            .legal_actions(&again)
            .contains(&e.card_action(&again, source)));
        again.card_mut(source).exhausted = false;
        let repeated = cast_action(e, &again, source);
        assert_eq!(live(&repeated, actor), vec![next, next + 1]);
        assert_eq!(repeated.round, s.round);
    });
}

#[test]
fn credit_trigger_action_is_not_an_entry_effect_and_failure_is_transactional() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let source = hand(e, &mut s, actor, "LAW_235");
        let charge = e.cost(&s, source);
        resources(e, &mut s, actor, usize::from(charge));
        let next = s.next_id;
        let done = choose(e, &s, source);
        assert!(done.card(source).exhausted);
        assert_eq!(done.next_id, next);
        assert!(live(&done, actor).is_empty());
        let mut invalid = done;
        invalid.active = actor;
        invalid.action_player = Some(actor);
        invalid.card_mut(source).exhausted = false;
        invalid.credit_state_exact = false;
        let selected = choose(e, &invalid, source);
        let action = e
            .legal_actions(&selected)
            .into_iter()
            .find(|a| a.label.ends_with("Create a Credit token"))
            .unwrap();
        assert!(e
            .apply(&selected, &action)
            .unwrap_err()
            .contains("exact Credit"));
        assert!(!selected.card(source).exhausted);
        assert_eq!(selected.next_id, next);
    });
}

#[test]
fn credit_trigger_defense_is_mandatory_target_specific_and_repeatable_while_exhausted() {
    cases(|e, actor| {
        let defender_owner = actor.other();
        let mut s = standard(e, actor);
        let first = unit(e, &mut s, actor, "LOF_254");
        let second = unit(e, &mut s, actor, "LOF_254");
        let defender = unit(e, &mut s, defender_owner, "LAW_121");
        let other = unit(e, &mut s, defender_owner, "LAW_121");
        s.card_mut(defender).exhausted = true;
        let targets = choose(e, &s, first);
        let ids = e
            .legal_actions(&targets)
            .iter()
            .filter_map(Action::id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![defender, other]); // Sentinel excludes the base.
        let next = s.next_id;
        let done = choose(e, &targets, defender);
        assert_eq!(done.decision, Decision::Window); // No optional trigger prompt.
        assert_eq!(live(&done, defender_owner), vec![next]);
        assert!(live(&done, actor).is_empty());
        assert_eq!(done.card(defender).damage, 1);
        assert!(done.card(defender).exhausted);
        assert_eq!(done.card(other).damage, 0);
        roundtrip(e, &done);
        let passed = button(e, &done, "Pass");
        let repeated = start_attack(e, &passed, second, defender);
        assert_eq!(live(&repeated, defender_owner), vec![next, next + 1]);
        assert_eq!(repeated.card(defender).damage, 2);
        assert_eq!(repeated.round, s.round);
    });
}

#[test]
fn credit_trigger_defense_creation_survives_combat_defeat_of_its_source() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let attacker = unit(e, &mut s, actor, "LOF_256");
        let defender = unit(e, &mut s, actor.other(), "LAW_121");
        s.card_mut(defender).damage = 4;
        let next = s.next_id;
        let done = start_attack(e, &s, attacker, defender);
        assert!(done.player(actor.other()).discard.contains(defender));
        assert_eq!(live(&done, actor.other()), vec![next]);
        assert_eq!(done.card(next).owner, actor.other());
        assert_eq!(done.decision, Decision::Window);
        roundtrip(e, &done);
    });
}

#[test]
fn credit_trigger_defense_owner_priority_preserves_attacker_optional_window() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        // Boba is a ground attacker with an optional On Attack payment.
        let attacker = unit(e, &mut s, actor, "LAW_214");
        let defender = unit(e, &mut s, actor.other(), "LAW_121");
        resources(e, &mut s, actor, 2);
        deck(e, &mut s, actor);
        let next = s.next_id;
        let window = start_attack(e, &s, attacker, defender);
        assert_eq!(window.decision, Decision::DeathPlayer);
        assert_eq!(window.active, actor);
        for priority in ["You", "Opponent"] {
            let optional = button(e, &window, priority);
            assert_eq!(optional.decision, Decision::AbilityOptional);
            assert_eq!(optional.active, actor);
            assert_eq!(
                live(&optional, actor.other()).len(),
                usize::from(priority == "Opponent")
            );
            assert!(live(&optional, actor).is_empty());
            let done = button(e, &optional, "Pass");
            assert_eq!(live(&done, actor.other()), vec![next]);
            assert_eq!(done.death_depth, 0);
            assert_eq!(done.attack_source, 0);
        }
    });
}

#[test]
fn credit_trigger_captured_defense_uses_original_controller_after_diagnostic_source_removal() {
    // Synthetic attacker registration tests capture/settlement only. Canto's
    // complete printed definition remains unchanged; no new attacker admission.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            let mut s = standard(&e, actor);
            let attacker = unit(&e, &mut s, actor, "LOF_256");
            let defender = unit(&e, &mut s, actor.other(), "LAW_121");
            let ability = serde_json::from_value(json!({"trigger":"onAttack","title":"DIAGNOSTIC remove defender","targetResolver":{"owner":"opponent","type":"unit","zone":"arena"},"effect":{"kind":"defeat","target":"chosen"}})).unwrap();
            let program = crate::ability_ir::Program::compile(&ability).unwrap();
            let catalogue = e.catalogue.as_mut().unwrap();
            let index = catalogue.programs.len() as u16;
            catalogue.programs.push(program);
            e.definitions[s.card(attacker).def as usize].programs.card = vec![index];
            let next = s.next_id;
            let window = start_attack(&e, &s, attacker, defender);
            let selected = button(&e, &window, "You");
            assert_eq!(selected.decision, Decision::AbilityTarget);
            let done = choose(&e, &selected, defender);
            assert!(done.player(actor.other()).discard.contains(defender));
            assert_eq!(live(&done, actor.other()), vec![next]);
            assert_eq!(done.card(next).owner, actor.other());
            assert_eq!(done.attack_source, 0);
            assert_eq!(done.death_depth, 0);
        }
    }
}
