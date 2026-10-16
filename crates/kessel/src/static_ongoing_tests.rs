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
fn static_ongoing_admission_delta_is_exact_and_conditional_ready_stays_rejected() {
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
    for code in ["HMW_208", "LAW_210", "LAW_223", "SEC_170"] {
        assert!(
            !admitted.by_code.contains_key(code),
            "conditional all-zone readiness remains rejected: {code}"
        );
    }
    eprintln!(
        "static_ongoing runtime-admitted delta: {:?}; count {} -> {}",
        added,
        baseline.by_code.len(),
        admitted.by_code.len()
    );
}
