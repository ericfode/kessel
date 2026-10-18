//! Include under generic_runtime_tests. Expected results come from the pinned
//! complete card sources, AttackStepsSystem, CardLastingEffectSystem and
//! OngoingEffect. These are Rust midgame regressions, not differential evidence.
use super::*;
use crate::migration_attack_contract::{self as contract, Condition};

const PRINTINGS: [&str; 8] = [
    "LAW_157", "LAW_202", "SOR_168", "SHD_101", "SOR_227", "SHD_236", "SOR_240", "HMW_266",
];

fn ma_entry(code: &str) -> Value {
    let catalog: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn ma_late(entry: &Value) -> Value {
    entry["abilities"]["card"][0]["effect"]["properties"]["properties"]["attackerLastingEffects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["evaluation"] == "afterAttackCreated")
        .unwrap()
        .clone()
}
fn ma_engine(backend: RulesBackend) -> Engine {
    let mut e = engine();
    e.set_rules_backend(backend).unwrap();
    e
}
fn ma_start(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn ma_resources(e: &Engine, s: &mut State, seat: Seat, count: usize) {
    for _ in 0..count {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
fn ma_click(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "missing {id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}
fn ma_play(e: &Engine, s: &mut State, code: &str) -> (State, CardId) {
    let seat = s.active;
    let source = add(e, s, seat, code);
    s.player_mut(seat).hand.push(source).unwrap();
    let next = ma_click(e, s, source);
    let next = if next.decision == Decision::AbilityOptional {
        named_button(e, &next, "Trigger")
    } else {
        next
    };
    assert_eq!(next.decision, Decision::AbilityTarget, "{code}");
    (next, source)
}
fn ma_finish(e: &Engine, mut s: State) -> State {
    for _ in 0..24 {
        if matches!(s.decision, Decision::Window | Decision::Terminal) {
            return s;
        }
        let action = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Trigger")
            .unwrap_or_else(|| panic!("unexpected {:?}: {:?}", s.decision, e.legal_actions(&s)));
        s = e.apply(&s, &action).unwrap();
    }
    panic!("attack trigger loop")
}
fn ma_roundtrip(e: &Engine, s: &State) {
    let copy = *s;
    assert_eq!(copy.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&copy), e.legal_actions(s));
    let snapshot = e.snapshot(s);
    let imported = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&imported), snapshot);
    assert_eq!(e.legal_actions(&imported), e.legal_actions(s));
    assert_eq!(imported.zobrist(), s.zobrist());
}

#[test]
fn migration_attack_contract_normalizes_seven_canonical_candidates_and_alias() {
    for (code, expected) in [
        (
            "LAW_157",
            Condition::AttackerHasTrait("bounty hunter".into()),
        ),
        ("LAW_202", Condition::InitiatingPlayerHasFewerResources),
        ("SOR_168", Condition::AttackerHasTrait("trooper".into())),
        ("SHD_101", Condition::InitiatingPlayerHasInitiative),
        ("SHD_236", Condition::AttackerHasTrait("imperial".into())),
        ("SOR_227", Condition::AttackerHasTrait("imperial".into())),
        ("SOR_240", Condition::AttackerHasTrait("rebel".into())),
        ("HMW_266", Condition::SharesTraitWithOtherFriendlyUnit),
    ] {
        let entry = ma_entry(code);
        assert!(contract::entry(&entry), "{code}");
        let late = contract::normalize_lasting_entry(&ma_late(&entry)).unwrap();
        assert_eq!(late.condition, expected);
        assert_eq!(late.power, 2);
        // Behavior is structural, independent of printing, class and title IDs.
        let mut renamed = entry.clone();
        renamed["setCode"] = json!("DIAGNOSTIC_ATTACK_SOURCE");
        renamed["id"] = json!("diagnostic-id");
        renamed["className"] = json!("DiagnosticAttackSource");
        renamed["card"]["title"] = json!("Diagnostic attack source");
        assert!(contract::entry(&renamed), "{code}");
    }
    assert_eq!(ma_entry("SOR_227")["aliasOf"], json!("SHD_236"));
    assert_eq!(
        ma_entry("SOR_227")["abilities"],
        ma_entry("SHD_236")["abilities"]
    );
}

#[test]
fn migration_attack_contract_rejects_late_effect_and_callback_scope_mutations() {
    for code in PRINTINGS {
        let original = ma_late(&ma_entry(code));
        for (pointer, value) in [
            ("/unknown", json!(true)),
            ("/evaluation", json!("beforeDefenderSelection")),
            ("/effect/unknown", json!(true)),
            ("/effect/duration", json!("phase")),
            ("/effect/target/path", json!(["source"])),
            (
                "/effect/power",
                json!({"kind":"ref","path":["source","power"]}),
            ),
            ("/effect/power/value", json!(-2)),
            ("/effect/power/value", json!(11)),
            ("/effect/power/value", json!("2")),
            ("/effect/hp/value", json!(1)),
            ("/condition/kind", json!("callback")),
            ("/condition/parameters/0/role", json!("candidate")),
            ("/condition/parameters/0/root", json!("")),
            ("/condition/body/unknown", json!(true)),
        ] {
            let mut bad = original.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            bad.pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert(key.into(), value);
            assert!(
                contract::normalize_lasting_entry(&bad).is_err(),
                "{code}/{pointer}"
            );
        }
        let mut capture = original.clone();
        capture["condition"]["body"] = json!({"kind":"query","name":"hasSomeTrait",
            "receiver":{"kind":"ref","path":["outerContext","source"]},
            "args":[{"kind":"const","value":"imperial"}]});
        assert!(contract::normalize_lasting_entry(&capture).is_err());
        let mut extra_parameter = original.clone();
        extra_parameter["condition"]["parameters"]
            .as_array_mut()
            .unwrap()
            .push(json!({"root":"captured","role":"attack"}));
        assert!(contract::normalize_lasting_entry(&extra_parameter).is_err());
    }
    let mut same_root = ma_late(&ma_entry("LAW_202"));
    same_root["condition"]["parameters"][1]["root"] =
        same_root["condition"]["parameters"][0]["root"].clone();
    assert!(contract::normalize_lasting_entry(&same_root).is_err());
    for (code, pointer, replacement) in [
        ("LAW_202", "/condition/body/op", json!("<=")),
        (
            "LAW_202",
            "/condition/body/left/path",
            json!(["attackBinding1_1", "player", "readyResourceCount"]),
        ),
        (
            "SHD_101",
            "/condition/body/receiver/path",
            json!(["attackBinding1_1", "player", "opponent"]),
        ),
        (
            "HMW_266",
            "/condition/body/filter/otherThan/path",
            json!(["attackBinding1_0", "defender"]),
        ),
        (
            "HMW_266",
            "/condition/body/filter/trait/receiver/path",
            json!(["attackBinding1_0", "defender", "traits"]),
        ),
    ] {
        let mut bad = ma_late(&ma_entry(code));
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            contract::normalize_lasting_entry(&bad).is_err(),
            "{code}/{pointer}"
        );
    }
}

#[test]
fn migration_attack_contract_closes_complete_headers_source_helpers_and_metadata() {
    for code in PRINTINGS {
        let original = ma_entry(code);
        for (key, value) in [
            ("costs", json!([{"kind":"exhaustSelf"}])),
            ("trigger", json!("onAttack")),
            ("condition", json!({"kind":"const","value":true})),
            ("then", json!({"effect":{"kind":"noAction"}})),
            ("ifYouDo", json!({"effect":{"kind":"noAction"}})),
            ("coordinate", json!(true)),
            ("requiresConfirmation", json!(true)),
            ("unreviewedHeader", json!(true)),
            (
                "targetResolver",
                json!({"owner":"opponent","type":"unit","zone":"arena"}),
            ),
        ] {
            let mut bad = original.clone();
            bad["abilities"]["card"][0][key] = value;
            assert!(!contract::entry(&bad), "{code}/{key}");
        }
        for (pointer, value) in [
            ("/baseClass", json!("LeaderUnitCard")),
            ("/card/types", json!(["unit", "leader"])),
            ("/card/deployBox", json!("Another ability")),
            ("/card/upgradePower", json!(0)),
            ("/piloting", json!({})),
            ("/stateWatchers", json!([{}])),
            ("/historyHelpers", json!([{}])),
            ("/privatePrograms", json!([{}])),
        ] {
            let mut bad = original.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            bad.pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert(key.into(), value);
            assert!(!contract::entry(&bad), "{code}/{pointer}");
        }
        let mut sibling = original.clone();
        sibling["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(!contract::entry(&sibling));
        let mut side = original.clone();
        side["abilities"]["unit"] = original["abilities"]["card"].clone();
        assert!(!contract::entry(&side));
    }
}

#[test]
fn migration_attack_contract_malformed_containers_return_false_without_panicking() {
    let original = ma_entry("LAW_157");
    for malformed in [
        json!(null),
        json!([]),
        json!({}),
        json!(true),
        json!(7),
        json!("invalid"),
    ] {
        assert!(!contract::entry(&malformed));
        for pointer in [
            "/abilities",
            "/abilities/card",
            "/abilities/card/0",
            "/abilities/card/0/effect",
            "/abilities/card/0/effect/properties",
            "/abilities/card/0/effect/properties/properties",
            "/abilities/card/0/effect/properties/properties/attackerLastingEffects",
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = malformed.clone();
            assert!(!contract::entry(&changed), "{pointer}/{malformed}");
        }
    }
}

#[test]
fn migration_attack_runtime_printings_have_equivalent_programs_without_duplicate_admission() {
    let e = engine();
    for code in PRINTINGS {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    assert_eq!(e.definitions[e.by_code["SOR_227"] as usize].code, "SOR_227");
    assert_eq!(e.definitions[e.by_code["SHD_236"] as usize].code, "SHD_236");
    // First-class catalogue alias rows have separate numeric indices; their
    // full serialized programs and canonical-only export must remain equivalent.
    let alias = &e.definitions[e.by_code["SOR_227"] as usize].programs.card;
    let canonical = &e.definitions[e.by_code["SHD_236"] as usize].programs.card;
    assert_eq!(alias.len(), canonical.len());
    for (&alias, &canonical) in alias.iter().zip(canonical) {
        assert_eq!(
            serde_json::to_value(e.program(alias)).unwrap(),
            serde_json::to_value(e.program(canonical)).unwrap()
        );
    }
    let pool = e.compiled_pool().unwrap();
    assert!(pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["setCode"] == "SHD_236"));
    assert!(!pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["setCode"] == "SOR_227"));
}

#[test]
fn migration_attack_runtime_trait_bonuses_read_selected_attacker_and_cleanup() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (source_code, matching_code) in [
                ("LAW_157", "HMW_134"),
                ("SOR_227", "SHD_236"),
                ("SHD_236", "SHD_236"),
                ("SOR_240", "SOR_240"),
            ] {
                for matching in [false, true] {
                    let mut s = ma_start(&e, seat);
                    ma_resources(&e, &mut s, seat, 12);
                    let attacker = unit(
                        &e,
                        &mut s,
                        seat,
                        if matching { matching_code } else { "LOF_093" },
                    );
                    let base = s.player(seat.other()).base;
                    let before = e.power(&s, attacker, true);
                    let (select, source) = ma_play(&e, &mut s, source_code);
                    assert!(
                        !e.legal_actions(&select)
                            .contains(&e.card_action(&select, source)),
                        "incoming source is exhausted"
                    );
                    let defender = ma_click(&e, &select, attacker);
                    assert_eq!(defender.decision, Decision::AttackTarget);
                    assert_eq!(defender.support.context.source, source);
                    assert_eq!(
                        e.power(&defender, attacker, true),
                        before,
                        "conditional effect is late"
                    );
                    assert!(!defender.card(attacker).exhausted);
                    let done = ma_finish(&e, ma_click(&e, &defender, base));
                    assert_eq!(
                        done.card(base).damage as i64,
                        i64::from(before) + if matching { 2 } else { 0 },
                        "{source_code}/{matching}"
                    );
                    assert!(done.card(attacker).exhausted);
                    assert_eq!(done.attack_source, 0);
                    assert_eq!(done.support.len, 0);
                    assert!(!done.support.pending);
                    assert_eq!(e.power(&done, attacker, false), before);
                    ma_roundtrip(&e, &done);
                }
            }
        }
    }
}

#[test]
fn migration_attack_runtime_initiative_and_total_resource_boundaries() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for initiative in [seat, seat.other()] {
                let mut s = ma_start(&e, seat);
                s.initiative = initiative;
                ma_resources(&e, &mut s, seat, 12);
                let attacker = unit(&e, &mut s, seat, "LOF_093");
                let base = s.player(seat.other()).base;
                let (select, _) = ma_play(&e, &mut s, "SHD_101");
                let defender = ma_click(&e, &select, attacker);
                assert_eq!(e.power(&defender, attacker, true), 2);
                let done = ma_finish(&e, ma_click(&e, &defender, base));
                assert_eq!(
                    done.card(base).damage,
                    if initiative == seat { 4 } else { 2 }
                );
            }
            for opponent_resources in [11, 12, 13] {
                let mut s = ma_start(&e, seat);
                ma_resources(&e, &mut s, seat, 12);
                ma_resources(&e, &mut s, seat.other(), opponent_resources);
                let enemy_resources = s.player(seat.other()).resources.as_slice().to_vec();
                for resource in enemy_resources {
                    s.card_mut(resource).exhausted = true;
                }
                let attacker = unit(&e, &mut s, seat, "LOF_093");
                let sentinel = unit(&e, &mut s, seat.other(), "LOF_044");
                let base = s.player(seat.other()).base;
                let (select, _) = ma_play(&e, &mut s, "LAW_202");
                let defender = ma_click(&e, &select, attacker);
                assert_eq!(e.keyword_amount(&defender, attacker, "saboteur"), 1);
                assert!(e
                    .legal_actions(&defender)
                    .contains(&e.card_action(&defender, base)));
                assert!(e
                    .legal_actions(&defender)
                    .contains(&e.card_action(&defender, sentinel)));
                assert_eq!(e.power(&defender, attacker, true), 2);
                assert_eq!(defender.player(seat).resources.len, 12);
                let done = ma_finish(&e, ma_click(&e, &defender, base));
                assert_eq!(
                    done.card(base).damage,
                    if opponent_resources > 12 { 4 } else { 2 }
                );
                assert_eq!(e.keyword_amount(&done, attacker, "saboteur"), 0);
                ma_roundtrip(&e, &done);
            }
        }
    }
}

#[test]
fn migration_attack_runtime_precision_fire_grants_early_saboteur_and_late_trooper_power() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for trooper in [false, true] {
                let mut s = ma_start(&e, seat);
                ma_resources(&e, &mut s, seat, 12);
                let attacker = unit(
                    &e,
                    &mut s,
                    seat,
                    if trooper { "SHD_236" } else { "LOF_093" },
                );
                unit(&e, &mut s, seat.other(), "LOF_044");
                let base = s.player(seat.other()).base;
                let (select, source) = ma_play(&e, &mut s, "SOR_168");
                let defender = ma_click(&e, &select, attacker);
                assert!(defender.player(seat).discard.contains(source));
                assert_eq!(e.power(&defender, attacker, true), 2);
                assert_eq!(e.keyword_amount(&defender, attacker, "saboteur"), 1);
                assert!(e
                    .legal_actions(&defender)
                    .contains(&e.card_action(&defender, base)));
                let done = ma_finish(&e, ma_click(&e, &defender, base));
                assert_eq!(done.card(base).damage, if trooper { 4 } else { 2 });
                assert_eq!(e.power(&done, attacker, false), 2);
                assert_eq!(e.keyword_amount(&done, attacker, "saboteur"), 0);
            }
        }
    }
}

#[test]
fn migration_attack_runtime_cancel_retains_only_early_grants_and_snapshot_provenance() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "SHD_236");
            let (select, source) = ma_play(&e, &mut s, "SOR_168");
            let ready = e.ready_resources(&select, seat);
            let unpaid = named_button(&e, &select, "Cancel");
            assert!(unpaid.player(seat).hand.contains(source));
            assert_eq!(e.ready_resources(&unpaid, seat), ready);
            assert_eq!(e.keyword_amount(&unpaid, attacker, "saboteur"), 0);
            let defender = ma_click(&e, &select, attacker);
            let cancelled = named_button(&e, &defender, "Cancel");
            assert!(!cancelled.card(attacker).exhausted);
            assert!(cancelled.player(seat).discard.contains(source));
            assert_eq!(e.keyword_amount(&cancelled, attacker, "saboteur"), 1);
            assert_eq!(
                e.power(&cancelled, attacker, false),
                2,
                "no late attack object was created"
            );
            assert!(!cancelled.support.pending);
            ma_roundtrip(&e, &cancelled);
            let mut missing = e.snapshot(&cancelled);
            missing
                .as_object_mut()
                .unwrap()
                .remove("supportStateVersion");
            assert!(e.load_state(&missing).is_err());
            let mut forged = e.snapshot(&cancelled);
            forged["supportGrants"][0]["unreviewedCondition"] = json!(true);
            assert!(e.load_state(&forged).is_err());
            // A subsequent ordinary attack consumes the retained early grant,
            // without inheriting the cancelled conditional power factory.
            let mut again = cancelled;
            again.active = seat;
            again.action_player = Some(seat);
            let base = again.player(seat.other()).base;
            let defender = ma_click(&e, &again, attacker);
            let done = ma_finish(&e, ma_click(&e, &defender, base));
            assert_eq!(done.card(base).damage, 2);
            assert_eq!(e.keyword_amount(&done, attacker, "saboteur"), 0);
            assert_eq!(done.support.len, 0);
        }
    }
}

#[test]
fn migration_attack_runtime_familiar_requires_another_friendly_matching_unit() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for ally in [None, Some("LOF_254"), Some("LOF_256")] {
                let mut s = ma_start(&e, seat);
                ma_resources(&e, &mut s, seat, 12);
                let attacker = unit(&e, &mut s, seat, "LOF_093");
                unit(&e, &mut s, seat.other(), "LOF_256");
                if let Some(code) = ally {
                    unit(&e, &mut s, seat, code);
                }
                let base = s.player(seat.other()).base;
                let (select, _) = ma_play(&e, &mut s, "HMW_266");
                let defender = ma_click(&e, &select, attacker);
                assert_eq!(e.power(&defender, attacker, true), 2);
                let done = ma_finish(&e, ma_click(&e, &defender, base));
                assert_eq!(
                    done.card(base).damage,
                    if ally == Some("LOF_256") { 4 } else { 2 }
                );
                assert_eq!(e.power(&done, attacker, false), 2);
            }
        }
    }
}

#[test]
fn migration_attack_runtime_familiar_reads_live_gained_traits_and_both_arenas() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            // Malakili supplies Underworld live to a Creature. Neither printed
            // trait lists alone nor the initiating event's Tactic trait match.
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "LOF_254");
            unit(&e, &mut s, seat, "LAW_212");
            assert!(e.card_has_trait(&s, attacker, "underworld"));
            let base = s.player(seat.other()).base;
            let (select, _) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let done = ma_finish(&e, ma_click(&e, &defender, base));
            assert_eq!(done.card(base).damage, 3);

            // A space attacker can match an exhausted friendly ground unit.
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "SHD_101");
            let ally = unit(&e, &mut s, seat, "HMW_213");
            s.card_mut(ally).exhausted = true;
            let base = s.player(seat.other()).base;
            let (select, _) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let done = ma_finish(&e, ma_click(&e, &defender, base));
            assert_eq!(done.card(base).damage, 6);
            assert!(done.card(ally).exhausted);

            // Native unit tokens count in the same live friendly inventory.
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "SHD_236");
            let ally = unit(&e, &mut s, seat, "TOKEN_CLONE_TROOPER");
            s.card_mut(ally).exhausted = true;
            let base = s.player(seat.other()).base;
            let (select, _) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let done = ma_finish(&e, ma_click(&e, &defender, base));
            assert_eq!(done.card(base).damage, 4);
        }
    }
}

#[test]
fn migration_attack_runtime_registered_familiar_condition_rechecks_after_ally_departure() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "SOR_131");
            let ally = unit(&e, &mut s, seat, "SHD_085");
            let base = s.player(seat.other()).base;
            let (select, source) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let declaration = ma_click(&e, &defender, base);
            assert!(declaration.support.pending);
            assert_eq!(declaration.support.context.source, source);
            assert_eq!(
                e.power(&declaration, attacker, true),
                4,
                "initially sharing Imperial installs +2"
            );
            let trigger = if declaration.decision == Decision::AbilityOptional {
                named_button(&e, &declaration, "Trigger")
            } else {
                declaration
            };
            assert_eq!(trigger.decision, Decision::AbilityTarget);
            let after_damage = ma_click(&e, &trigger, ally);
            assert!(!e.in_arena(&after_damage, ally));
            assert_eq!(after_damage.card(attacker).damage, 1);
            assert_eq!(
                e.power(&after_damage, attacker, true),
                3,
                "Raid 1 remains; live shared-trait +2 stops"
            );
            let done = ma_finish(&e, after_damage);
            assert_eq!(
                done.card(base).damage,
                3,
                "a frozen bonus would incorrectly deal 5"
            );
            assert_eq!(e.power(&done, attacker, false), 2);
            assert!(done.player(seat).discard.contains(source));
            ma_roundtrip(&e, &done);
        }
    }
}

#[test]
fn migration_attack_runtime_false_initial_gate_never_installs_later_shared_trait_bonus() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "HMW_064");
            let ally = unit(&e, &mut s, seat, "TWI_043");
            let experience = add(&e, &mut s, seat, "TOKEN_EXPERIENCE");
            s.card_mut(ally).experience = experience;
            e.attach(&mut s, experience, ally);
            s.card_mut(ally).damage = 3;
            let base = s.player(seat.other()).base;
            let (select, _) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let declaration = ma_click(&e, &defender, base);
            assert_eq!(
                e.power(&declaration, attacker, true),
                3,
                "no shared trait at installation"
            );
            let trigger = if declaration.decision == Decision::AbilityOptional {
                named_button(&e, &declaration, "Trigger")
            } else {
                declaration
            };
            assert_eq!(trigger.decision, Decision::AbilityTarget);
            let after_damage = ma_click(&e, &trigger, ally);
            let done = ma_finish(&e, after_damage);
            assert!(!e.in_arena(&done, ally));
            assert!(done
                .player(seat)
                .ground
                .as_slice()
                .iter()
                .any(|id| e.def(&done, *id).code == "TOKEN_CLONE_TROOPER"));
            assert_eq!(
                done.card(base).damage,
                3,
                "new matching Clone/Trooper does not revive a failed initial gate"
            );
            assert_eq!(e.power(&done, attacker, false), 3);
            // The existing snapshot format omits retired upgrade-token slots.
            // Check the observable roundtrip and all other hashed state after
            // projecting only that inert Experience slot to its import default.
            let snapshot = e.snapshot(&done);
            let imported = e.load_state(&snapshot).unwrap();
            assert_eq!(e.snapshot(&imported), snapshot);
            assert_eq!(e.legal_actions(&imported), e.legal_actions(&done));
            assert_eq!(done.card(experience).parent, 0);
            let mut projected = done;
            *projected.card_mut(experience) = Card::default();
            assert_eq!(imported.zobrist(), projected.zobrist());
        }
    }
}

#[test]
fn migration_attack_runtime_registered_resource_condition_uses_live_total_counts() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            ma_resources(&e, &mut s, seat.other(), 13);
            let attacker = unit(&e, &mut s, seat, "SOR_131");
            let ally = unit(&e, &mut s, seat, "SHD_085");
            let base = s.player(seat.other()).base;
            let (select, _) = ma_play(&e, &mut s, "LAW_202");
            let defender = ma_click(&e, &select, attacker);
            let declaration = ma_click(&e, &defender, base);
            assert_eq!(e.power(&declaration, attacker, true), 4);
            let trigger = if declaration.decision == Decision::AbilityOptional {
                named_button(&e, &declaration, "Trigger")
            } else {
                declaration
            };
            let after_damage = ma_click(&e, &trigger, ally);
            let done = ma_finish(&e, after_damage);
            assert!(done.player(seat).resources.contains(ally));
            assert_eq!(done.player(seat).resources.len, 13);
            assert_eq!(
                done.card(base).damage,
                3,
                "equal total resources disables +2; Raid 1 remains"
            );
            assert_eq!(e.keyword_amount(&done, attacker, "saboteur"), 0);
            ma_roundtrip(&e, &done);
        }
    }
}

#[test]
fn migration_attack_runtime_live_reactivation_and_physical_departure_boundaries() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = ma_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = ma_start(&e, seat);
            ma_resources(&e, &mut s, seat, 12);
            let attacker = unit(&e, &mut s, seat, "SOR_131");
            let ally = unit(&e, &mut s, seat, "SHD_085");
            let base = s.player(seat.other()).base;
            let (select, donor) = ma_play(&e, &mut s, "HMW_266");
            let defender = ma_click(&e, &select, attacker);
            let mut declaration = ma_click(&e, &defender, base);
            assert_eq!(e.power(&declaration, attacker, true), 4);
            // Exercise actual departure and re-entry helpers while the captured
            // declaration window is paused. The same installed grant reactivates.
            e.leave_play(&mut declaration, ally).unwrap();
            assert_eq!(e.power(&declaration, attacker, true), 2);
            assert!(declaration.player_mut(seat).discard.remove(ally));
            e.add_arena(&mut declaration, ally).unwrap();
            assert_eq!(e.power(&declaration, attacker, true), 4);
            // The event donor is already in discard; its absence from play
            // never replaces the selected physical attacker in the predicate.
            assert!(declaration.player(seat).discard.contains(donor));
            assert!(!e.in_arena(&declaration, donor));
            e.leave_play(&mut declaration, attacker).unwrap();
            assert_eq!(declaration.support.len, 0);
            assert_eq!(e.native_attack_power(&declaration, attacker), 0);
        }
    }
}

#[test]
fn migration_attack_runtime_bounded_provenance_hash_and_pending_import_guards() {
    let e = ma_engine(RulesBackend::Auto);
    let mut s = ma_start(&e, Seat::P1);
    ma_resources(&e, &mut s, Seat::P1, 12);
    let attacker = unit(&e, &mut s, Seat::P1, "SOR_131");
    unit(&e, &mut s, Seat::P1, "SHD_085");
    let base = s.player(Seat::P2).base;
    let (select, _) = ma_play(&e, &mut s, "HMW_266");
    let defender = ma_click(&e, &select, attacker);
    let declared = ma_click(&e, &defender, base);
    let index = declared.support.grants[..declared.support.len as usize]
        .iter()
        .position(|g| g.conditional_mask != 0)
        .unwrap();
    assert_eq!(
        std::mem::size_of::<State>(),
        crate::model::EXPECTED_INLINE_STATE_BYTES
    );
    let original = declared.zobrist();
    for field in 0..3 {
        let mut changed = declared;
        let g = &mut changed.support.grants[index];
        match field {
            0 => g.conditional_program += 1,
            1 => g.conditional_pc += 1,
            _ => g.conditional_mask ^= 2,
        }
        assert_ne!(
            changed.zobrist(),
            original,
            "conditional provenance field {field}"
        );
    }
    let snapshot = e.snapshot(&declared);
    assert!(snapshot["supportGrants"][index]
        .get("conditionalAttack")
        .is_some());
    for marker in [json!(true), json!(false), json!(null)] {
        let mut forged = snapshot.clone();
        forged["actor"] = json!("p1");
        forged["winner"] = json!("p1");
        forged["supportAttackPending"] = marker;
        assert!(e.load_state(&forged).is_err());
    }
    let mut forged = snapshot;
    forged["actor"] = json!("p1");
    forged["winner"] = json!("p1");
    forged
        .as_object_mut()
        .unwrap()
        .remove("supportAttackPending");
    for descriptor in [json!(false), json!(null)] {
        forged["supportGrants"][index]["conditionalAttack"] = descriptor;
        assert!(e.load_state(&forged).is_err());
    }
}

#[test]
fn migration_attack_runtime_late_capacity_failure_is_transactional() {
    let e = ma_engine(RulesBackend::Auto);
    let mut s = ma_start(&e, Seat::P1);
    ma_resources(&e, &mut s, Seat::P1, 12);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_236");
    let base = s.player(Seat::P2).base;
    let (select, _) = ma_play(&e, &mut s, "SOR_168");
    let mut defender = ma_click(&e, &select, attacker);
    // Fill the existing bounded grant storage before late registration. The
    // public immutable transition must return an error and preserve its input.
    let early = defender.support.grants[0];
    defender.support.grants.fill(early);
    defender.support.len = MAX_SUPPORT_GRANTS as u8;
    let before = e.snapshot(&defender);
    let hash = defender.zobrist();
    let error = e
        .apply(&defender, &e.card_action(&defender, base))
        .unwrap_err();
    assert!(
        error.contains("lasting-effect capacity exceeded"),
        "{error}"
    );
    assert_eq!(e.snapshot(&defender), before);
    assert_eq!(defender.zobrist(), hash);
    assert!(!defender.card(attacker).exhausted);
    let mut oversized = ma_click(&e, &select, attacker);
    oversized.card_mut(attacker).phase_power = 65_533;
    let original = oversized;
    let error = e
        .apply(&oversized, &e.card_action(&oversized, base))
        .unwrap_err();
    assert!(
        error.contains("computed stats exceed compact capacity"),
        "{error}"
    );
    assert_eq!(oversized, original);
}
