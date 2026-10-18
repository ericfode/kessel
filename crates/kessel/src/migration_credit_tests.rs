//! Source-derived midgame regressions for complete ordinary Credit compositions.
//! Wire as a child of simulator::credit_action_tests to reuse its native ledger,
//! public-action, owner/backend and state-copy fixture helpers. These tests do
//! not constitute forceteki differential verification.
use super::*;
use crate::migration_credit_contract::{prepare, Composition};

const CANDIDATES: &[&str] = &["LAW_134", "LAW_155", "LAW_258"];
const HELD: &[&str] = &["LAW_053", "LAW_071", "LAW_080", "LAW_116", "LAW_247"];

fn entry(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["setCode"] == code)
        .unwrap()
        .clone()
}
fn midgame(e: &Engine, actor: Seat) -> State {
    let mut s = standard(e, actor);
    s.round = 4;
    s
}
fn attack(e: &Engine, s: &State, source: CardId, target: CardId) -> State {
    let targets = choose(e, s, source);
    assert_eq!(targets.decision, Decision::AttackTarget);
    choose(e, &targets, target)
}
fn roundtrip(e: &Engine, s: &State) {
    let copy = *s;
    assert_eq!(copy.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&copy), e.legal_actions(s));
    let snapshot = e.snapshot(s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    assert_eq!(loaded.zobrist(), s.zobrist());
}

#[test]
fn migration_credit_complete_contract_preserves_conditions_and_success_continuations() {
    for (code, kind) in [
        ("LAW_134", Composition::OtherFriendlyUnderworldOnPlay),
        ("LAW_155", Composition::FriendlyGroundOnAttack),
        ("LAW_258", Composition::OptionalAttackPayment),
    ] {
        let original = entry(code);
        let prepared = prepare(&original).unwrap().unwrap();
        assert_eq!(prepared.composition, kind);
        assert_eq!(prepared.normalized, original);
        let ability = serde_json::from_value::<crate::ability_ir::Ability>(
            prepared.normalized["abilities"]["card"][0].clone(),
        )
        .unwrap();
        crate::ability_ir::Program::compile(&ability).unwrap();
        let mut renamed = original;
        renamed["setCode"] = json!("DIAGNOSTIC_CREDIT_COMPOSITION");
        renamed["id"] = json!("diagnostic identity");
        renamed["className"] = json!("DiagnosticCompleteComposition");
        renamed["card"]["title"] = json!("Diagnostic composition");
        renamed["abilities"]["card"][0]["title"] = json!("Diagnostic trigger");
        assert_eq!(prepare(&renamed).unwrap().unwrap().composition, kind);
    }
    assert!(prepare(&entry("TWI_212")).unwrap().is_none());
    for code in HELD {
        assert!(
            prepare(&entry(code)).is_err(),
            "{code} needs a separate shared contract"
        );
    }
}

#[test]
fn migration_credit_contract_rejects_source_metadata_and_hidden_registration_mutations() {
    for code in CANDIDATES {
        let original = entry(code);
        for (path, value) in [
            ("/status", json!("unsupported")),
            ("/diagnostics", json!(["unconverted source"])),
            ("/baseClass", json!("LeaderUnitCard")),
            ("/card/types", json!(["unit", "token"])),
            ("/card/keywords", json!(["shielded"])),
            ("/keywordValues", json!({"shielded": 1})),
            ("/card/text", json!("Create a Credit token.")),
            ("/card/hp", json!(0)),
            ("/card/cost", json!(-1)),
            ("/card/power", json!(1001)),
            ("/card/arena", json!("base")),
            ("/card/upgradeHp", json!(0)),
            (
                "/abilities/unit",
                json!([{"trigger":"whenPlayed","effect":{"kind":"noAction"}}]),
            ),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert!(prepare(&changed).is_err(), "{code}: {path}");
        }
        for (field, value) in [
            ("piloting", json!({})),
            ("attachCondition", json!({})),
            ("historyHelpers", json!([{}])),
            ("privatePrograms", json!([{}])),
            ("inheritedSource", json!([{}])),
            ("deploymentOverride", json!([{}])),
            ("setupValidationOverrides", json!([{}])),
            ("stateWatchers", json!([{}])),
            ("classMethods", json!([{}])),
            ("privateClassConstants", json!([{}])),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            assert!(prepare(&changed).is_err(), "{code}: {field}");
        }
        for (field, value) in [
            ("coordinate", json!(true)),
            ("requiresConfirmation", json!(true)),
            ("condition", json!({"kind":"const","value":true})),
            ("targetResolver", json!({})),
            ("targetResolvers", json!([])),
            ("attachedContext", json!({})),
            ("limit", json!({})),
            ("hooks", json!([])),
            ("then", json!({"effect":{"kind":"noAction"}})),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0][field] = value;
            assert!(prepare(&changed).is_err(), "{code}: {field}");
        }
        let mut changed = original.clone();
        changed["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(prepare(&changed).is_err());
    }
}

#[test]
fn migration_credit_contract_rejects_effect_scope_timing_and_payment_mutations() {
    for code in CANDIDATES {
        let original = entry(code);
        let leaf = if *code == "LAW_258" {
            "/abilities/card/0/ifYouDo/effect"
        } else {
            "/abilities/card/0/effect/onTrue"
        };
        for (field, value) in [
            ("target", json!("enemyPlayer")),
            ("amount", json!({"kind":"const","value":2})),
            ("destination", json!("ground")),
            ("creationTiming", json!("handler")),
            ("unreviewed", json!(true)),
        ] {
            let mut changed = original.clone();
            changed.pointer_mut(leaf).unwrap()[field] = value;
            assert!(prepare(&changed).is_err(), "{code}: {field}");
        }
    }
    let original = entry("LAW_258");
    for (field, value) in [
        ("amount", json!({"kind":"const","value":0})),
        ("amount", json!({"kind":"const","value":1})),
        ("isCost", json!(true)),
        ("allowCostAdjustments", json!(false)),
        ("canCancelDuringPayment", json!(true)),
        ("target", json!("chosenPlayer")),
    ] {
        let mut changed = original.clone();
        changed["abilities"]["card"][0]["effect"][field] = value;
        assert!(prepare(&changed).is_err(), "payment {field}");
    }
    let mut mandatory = original.clone();
    mandatory["abilities"]["card"][0]["optional"] = json!(false);
    assert!(prepare(&mandatory).is_err());
    for code in ["LAW_134", "LAW_155"] {
        let mut changed = entry(code);
        changed["abilities"]["card"][0]["effect"]["condition"]["receiver"] =
            json!({"kind":"ref","path":["player","opponent"]});
        assert!(prepare(&changed).is_err(), "{code}: condition owner");
        let mut changed = entry(code);
        changed["abilities"]["card"][0]["effect"]["onFalse"] = json!({"kind":"createCreditToken","target":"ownPlayer","amount":{"kind":"const","value":1},"destination":"base","creationTiming":"eventUpdate"});
        assert!(prepare(&changed).is_err(), "{code}: false branch");
    }
}

#[test]
fn migration_credit_bib_requires_another_live_friendly_underworld_unit() {
    cases(|e, actor| {
        for setup in [
            "none",
            "enemy",
            "friendly",
            "friendlySpace",
            "nonarena",
            "gainedTrait",
            "lostTrait",
            "nontrait",
        ] {
            let mut s = midgame(e, actor);
            match setup {
                "enemy" => {
                    unit(e, &mut s, actor.other(), "LAW_262");
                }
                "friendly" => {
                    unit(e, &mut s, actor, "LAW_262");
                }
                "friendlySpace" => {
                    unit(e, &mut s, actor, "SEC_161");
                }
                "nonarena" => {
                    hand(e, &mut s, actor, "LAW_262");
                }
                "gainedTrait" | "lostTrait" => {
                    let host = unit(e, &mut s, actor, "LOF_256");
                    let disguise = add(e, &mut s, actor, "LAW_111");
                    e.attach(&mut s, disguise, host);
                    if setup == "lostTrait" {
                        e.defeat(&mut s, disguise).unwrap();
                    }
                }
                "nontrait" => {
                    unit(e, &mut s, actor, "LOF_256");
                }
                _ => (),
            }
            let source = hand(e, &mut s, actor, "LAW_134");
            let cost = usize::from(e.cost(&s, source));
            resources(e, &mut s, actor, cost);
            let next = s.next_id;
            let done = choose(e, &s, source);
            assert!(done.player(actor).ground.contains(source));
            assert!(done.card(source).exhausted);
            assert_eq!(
                live(&done, actor),
                if ["friendly", "friendlySpace", "gainedTrait"].contains(&setup) {
                    vec![next]
                } else {
                    vec![]
                },
                "{setup}"
            );
            assert!(live(&done, actor.other()).is_empty());
            assert_eq!(
                done.next_id,
                next + u16::from(["friendly", "friendlySpace", "gainedTrait"].contains(&setup))
            );
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.round, 4);
            roundtrip(e, &done);
        }
    });
}

#[test]
fn migration_credit_getaway_checks_friendly_ground_and_keeps_ordered_fresh_identity() {
    cases(|e, actor| {
        for setup in ["none", "enemyGround", "friendlySpace", "friendlyGround"] {
            let mut s = midgame(e, actor);
            let source = unit(e, &mut s, actor, "LAW_155");
            match setup {
                "enemyGround" => {
                    unit(e, &mut s, actor.other(), "LOF_256");
                }
                "friendlySpace" => {
                    unit(e, &mut s, actor, "LAW_155");
                }
                "friendlyGround" => {
                    unit(e, &mut s, actor, "LOF_256");
                }
                _ => (),
            }
            let previous = credits(e, &mut s, actor, 1);
            let target = s.player(actor.other()).base;
            let next = s.next_id;
            let done = attack(e, &s, source, target);
            let mut expected = previous;
            if setup == "friendlyGround" {
                expected.push(next);
            }
            assert_eq!(live(&done, actor), expected, "{setup}");
            assert!(live(&done, actor.other()).is_empty());
            assert_eq!(done.card(target).damage, 1);
            assert_eq!(done.next_id, next + u16::from(setup == "friendlyGround"));
            assert_eq!(done.decision, Decision::Window);
            roundtrip(e, &done);
        }
    });
}

#[test]
fn migration_credit_criminal_payment_zero_partial_and_all_credits_resume_once() {
    cases(|e, actor| {
        for (count, ready) in [(0, 2), (1, 1), (2, 0)] {
            let mut s = midgame(e, actor);
            let source = unit(e, &mut s, actor, "LAW_258");
            let rows = resources(e, &mut s, actor, ready);
            let old = credits(e, &mut s, actor, count);
            let target = s.player(actor.other()).base;
            let next = s.next_id;
            let optional = attack(e, &s, source, target);
            assert_eq!(optional.decision, Decision::AbilityOptional);
            assert_eq!(optional.active, actor);
            assert_eq!(
                e.legal_actions(&optional),
                vec![
                    Action::button("Trigger", "trigger"),
                    Action::button("Pass", "pass")
                ]
            );
            let declined = button(e, &optional, "Pass");
            assert_eq!(live(&declined, actor), old);
            assert_eq!(declined.next_id, next);
            assert_eq!(e.ready_resources(&declined, actor), ready);
            let payment = button(e, &optional, "Trigger");
            let done = match count {
                0 => payment,
                1 => {
                    menu(e, &payment, &["Use 1 Credit"]);
                    button(e, &payment, "Use 1 Credit")
                }
                2 => {
                    menu(e, &payment, &["Use 2 Credits"]);
                    button(e, &payment, "Use 2 Credits")
                }
                _ => unreachable!(),
            };
            assert_eq!(live(&done, actor), vec![next]);
            assert!(live(&done, actor.other()).is_empty());
            assert_eq!(done.next_id, next + 1);
            assert_eq!(done.card(next).owner, actor);
            assert_eq!(e.ready_resources(&done, actor), 0);
            for id in rows {
                assert!(done.card(id).exhausted);
            }
            for id in old {
                assert!(done.card(id).present);
                assert!(!live(&done, actor).contains(&id));
            }
            assert_eq!(done.card(target).damage, 1);
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(
                done.resource_payment,
                crate::resource_payment::ResourcePayment::default()
            );
            roundtrip(e, &done);
        }
    });
}

#[test]
fn migration_credit_criminal_unaffordable_payment_cannot_create_or_partially_pay() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let source = unit(e, &mut s, actor, "LAW_258");
        let rows = resources(e, &mut s, actor, 1);
        let target = s.player(actor.other()).base;
        let next = s.next_id;
        let done = attack(e, &s, source, target);
        assert_eq!(done.decision, Decision::Window);
        assert_eq!(done.next_id, next);
        assert!(live(&done, actor).is_empty());
        assert!(!done.card(rows[0]).exhausted);
        assert_eq!(done.card(target).damage, 1);
    });
}

#[test]
fn migration_credit_creation_still_requires_selected_native_provenance_and_exact_import() {
    let original = catalogue();
    for malformed in [false, true] {
        let mut changed = original.clone();
        if malformed {
            changed["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|entry| entry["setCode"] == "TOKEN_CREDIT")
                .unwrap()["sourceHash"] = json!("unreviewed");
        } else {
            changed["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|entry| entry["setCode"] != "TOKEN_CREDIT");
        }
        let e = Engine::from_catalogue_value(&changed).unwrap();
        for code in CANDIDATES {
            assert!(!e.by_code.contains_key(*code), "{code}");
        }
    }
    let e = engine(RulesBackend::Interpreter);
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|card| card["setCode"] != "TOKEN_CREDIT");
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&original).unwrap()
    )
    .err()
    .unwrap()
    .contains("TOKEN_CREDIT"));
    let s = midgame(&e, Seat::P2);
    let mut legacy = e.snapshot(&s);
    legacy.as_object_mut().unwrap().remove("creditStateVersion");
    for seat in ["p1", "p2"] {
        for field in ["credits", "retiredCredits"] {
            assert_eq!(legacy["players"][seat][field], json!([]));
            legacy["players"][seat].as_object_mut().unwrap().remove(field);
        }
    }
    let error = e.load_state(&legacy).unwrap_err();
    assert!(error.contains("Credit state version"), "{error}");
}

#[test]
fn migration_credit_bib_unique_replacement_checks_live_other_after_source_defeat() {
    cases(|e, actor| {
        for defeat_incoming in [false, true] {
            let mut s = midgame(e, actor);
            let previous = unit(e, &mut s, actor, "LAW_134");
            let incoming = hand(e, &mut s, actor, "LAW_134");
            let cost = usize::from(e.cost(&s, incoming));
            resources(e, &mut s, actor, cost);
            let next = s.next_id;
            let unique = choose(e, &s, incoming);
            assert_eq!(unique.decision, Decision::Unique);
            let defeated = if defeat_incoming { incoming } else { previous };
            let kept = if defeat_incoming { previous } else { incoming };
            let done = choose(e, &unique, defeated);
            assert!(done.player(actor).discard.contains(defeated));
            assert!(done.player(actor).ground.contains(kept));
            // A captured incoming trigger survives its own defeat. Its query is
            // still live: retaining only the incoming Bib leaves no OTHER unit.
            assert_eq!(
                live(&done, actor),
                if defeat_incoming { vec![next] } else { vec![] }
            );
            assert!(live(&done, actor.other()).is_empty());
            assert_eq!(done.decision, Decision::Window);
            roundtrip(e, &done);
        }
    });
}

#[test]
fn migration_credit_captured_criminal_uses_original_controller_after_source_defeat() {
    // Synthetic defender registration establishes the ordering boundary. The
    // attacker uses Criminal Contact's unchanged complete printed definition;
    // this diagnostic is not a forceteki differential witness.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            let mut s = midgame(&e, actor);
            let source = unit(&e, &mut s, actor, "LAW_258");
            let defender = unit(&e, &mut s, actor.other(), "LOF_256");
            let enemy_resources = resources(&e, &mut s, actor.other(), 2);
            credits(&e, &mut s, actor, 2);
            let ability = serde_json::from_value(json!({"trigger":"onDefense","title":"DIAGNOSTIC defeat attacker", "targetResolver":{"owner":"opponent","type":"unit","zone":"arena"},"effect":{"kind":"defeat","target":"chosen"}})).unwrap();
            let program = crate::ability_ir::Program::compile(&ability).unwrap();
            let catalogue = e.catalogue.as_mut().unwrap();
            let index = catalogue.programs.len() as u16;
            catalogue.programs.push(program);
            e.definitions[s.card(defender).def as usize].programs.card = vec![index];
            let next = s.next_id;
            let order = attack(&e, &s, source, defender);
            assert_eq!(order.decision, Decision::DeathPlayer);
            let removal = button(&e, &order, "Opponent");
            assert_eq!(removal.active, actor.other());
            assert_eq!(removal.decision, Decision::AbilityTarget);
            let captured = choose(&e, &removal, source);
            assert!(captured.player(actor).discard.contains(source));
            assert_eq!(captured.decision, Decision::AbilityOptional);
            assert_eq!(captured.active, actor);
            let payment = button(&e, &captured, "Trigger");
            menu(&e, &payment, &["Use 2 Credits"]);
            assert_eq!(payment.active, actor);
            let done = button(&e, &payment, "Use 2 Credits");
            assert!(done.player(actor).discard.contains(source));
            assert_eq!(live(&done, actor), vec![next]);
            assert!(live(&done, actor.other()).is_empty());
            assert_eq!(done.card(next).owner, actor);
            for id in enemy_resources {
                assert!(!done.card(id).exhausted);
            }
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.attack_source, 0);
            assert_eq!(done.death_depth, 0);
            roundtrip(&e, &done);
        }
    }
}
