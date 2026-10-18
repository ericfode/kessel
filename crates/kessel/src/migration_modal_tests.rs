//! Source-derived contract checks and existing-runtime modal controls.
//! The assigned cards remain unadmitted; these are not differential witnesses.
use super::*;
use crate::migration_modal_contract::{self as contract, Chooser, Resolution};

const DESCRIBED: &[&str] = &["HMW_218", "IC27_185", "LOF_200", "SHD_227", "SOR_233"];
const HELD: &[&str] = &[
    "ASH_077", "HMW_218", "IC27_185", "LAW_086", "LAW_243", "LOF_200", "LOF_222", "SEC_220",
    "SHD_227", "SOR_062", "SOR_233",
];
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(catalogue: &Value, code: &str) -> Value {
    catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["setCode"] == code)
        .unwrap()
        .clone()
}
fn set(value: &mut Value, pointer: &str, replacement: Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let parent = value.pointer_mut(parent).unwrap();
    if let Some(object) = parent.as_object_mut() {
        object.insert(key.into(), replacement);
    } else {
        parent[key.parse::<usize>().unwrap()] = replacement;
    }
}
fn labels(e: &Engine, s: &State) -> Vec<String> {
    e.legal_actions(s).into_iter().map(|a| a.label).collect()
}
fn choose(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", labels(e, s)));
    e.apply(s, &action).unwrap()
}

#[test]
fn migration_modal_describes_complete_dependent_sources_without_erasing_the_registration() {
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        let bytes = serde_json::to_vec(&original).unwrap();
        let description = contract::inspect(&original).unwrap().unwrap();
        assert_eq!(serde_json::to_vec(&original).unwrap(), bytes);
        assert_eq!(description.registration, original["abilities"]["card"][0]);
        assert_eq!(
            description.registration["targetResolvers"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        match *code {
            "IC27_185" | "SOR_233" => {
                assert_eq!(
                    description.resolution,
                    Resolution::DamageOrDraw { damage: 7, draw: 3 }
                );
                assert_eq!(description.chooser, Chooser::Opponent);
                assert_eq!(description.controller, "opponent");
                assert!(!description.optional_target);
            }
            "SHD_227" => {
                assert_eq!(
                    description.resolution,
                    Resolution::ExhaustOrAdjustedPayment { amount: 2 }
                );
                assert_eq!(description.chooser, Chooser::SelectedController);
                assert_eq!(description.controller, "any");
            }
            "HMW_218" | "LOF_200" => {
                assert_eq!(description.resolution, Resolution::OwnerDeckPlacement);
                assert_eq!(description.chooser, Chooser::SelectedOwner);
                assert_eq!(description.card_type, "nonLeaderUnit");
                assert_eq!(description.controller, "any");
                assert_eq!(description.optional_target, *code == "LOF_200");
                assert_eq!(
                    description.zone,
                    if *code == "LOF_200" {
                        "groundArena"
                    } else {
                        "anyAttackable"
                    }
                );
                assert_eq!(
                    description.trigger,
                    if *code == "LOF_200" {
                        "whenDefeated"
                    } else {
                        "event"
                    }
                );
            }
            _ => unreachable!(),
        }
    }
    let father = contract::inspect(&entry(&c, "IC27_185")).unwrap().unwrap();
    let alias = contract::inspect(&entry(&c, "SOR_233")).unwrap().unwrap();
    assert_eq!(
        father, alias,
        "alternate printing must not become a separate semantic family"
    );
}

#[test]
fn migration_modal_alpha_names_are_nonsemantic_but_capture_references_remain_bound() {
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        let mut renamed = original.clone();
        fn rename_callbacks(value: &mut Value) {
            if value["kind"] == "namedSelectionRead" {
                value["contextBinding"]["id"] = json!("renamedLexicalCallback");
            }
            match value {
                Value::Array(items) => {
                    for item in items {
                        rename_callbacks(item)
                    }
                }
                Value::Object(fields) => {
                    for item in fields.values_mut() {
                        rename_callbacks(item)
                    }
                }
                _ => {}
            }
        }
        rename_callbacks(&mut renamed["abilities"]);
        if ["HMW_218", "LOF_200"].contains(code) {
            let factory = &mut renamed["abilities"]["card"][0]["targetResolvers"][1]
                ["targetResolver"]["choices"];
            factory["parameters"][0]["root"] = json!("renamedFactory");
            for (i, branch) in factory["entries"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                let name = format!("renamedCapture{i}");
                branch["constructionCaptures"][0]["name"] = json!(name);
                branch["constructionCaptures"][0]["value"]["path"][0] = json!("renamedFactory");
                branch["effect"]["target"]["path"][1] = json!(name);
            }
        }
        let expected = contract::inspect(&original).unwrap().unwrap();
        let mut actual = contract::inspect(&renamed).unwrap().unwrap();
        assert_eq!(actual.registration, renamed["abilities"]["card"][0]);
        actual.registration = expected.registration.clone();
        assert_eq!(actual, expected, "alpha-equivalent source {code}");
        // Selection slot names are part of the actual context protocol.
        renamed["abilities"]["card"][0]["targetResolvers"][0]["name"] = json!("differentSlot");
        assert!(contract::inspect(&renamed).is_err());
    }
}

#[test]
fn migration_modal_unknown_fields_at_every_registration_depth_fail_closed() {
    fn objects(value: &Value, path: String, out: &mut Vec<String>) {
        match value {
            Value::Object(fields) => {
                out.push(path.clone());
                for (key, value) in fields {
                    objects(value, format!("{path}/{key}"), out);
                }
            }
            Value::Array(items) => {
                for (i, value) in items.iter().enumerate() {
                    objects(value, format!("{path}/{i}"), out)
                }
            }
            _ => {}
        }
    }
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        let mut paths = Vec::new();
        objects(
            &original["abilities"]["card"][0],
            "/abilities/card/0".into(),
            &mut paths,
        );
        for path in paths {
            let mut mutated = original.clone();
            mutated
                .pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unreviewed".into(), json!(true));
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted unknown field at {path}"
            );
        }
    }
}

#[test]
fn migration_modal_malformed_nested_containers_reject_without_panicking() {
    fn containers(value: &Value, path: String, out: &mut Vec<String>) {
        match value {
            Value::Object(fields) => {
                out.push(path.clone());
                for (key, value) in fields {
                    containers(value, format!("{path}/{key}"), out);
                }
            }
            Value::Array(items) => {
                out.push(path.clone());
                for (index, value) in items.iter().enumerate() {
                    containers(value, format!("{path}/{index}"), out);
                }
            }
            _ => {}
        }
    }
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        let mut paths = Vec::new();
        containers(&original["abilities"], "/abilities".into(), &mut paths);
        for path in paths {
            for replacement in [
                Value::Null,
                json!(true),
                json!(1),
                json!("malformed"),
                json!([]),
                json!({}),
            ] {
                let mut mutated = original.clone();
                if mutated.pointer(&path) == Some(&replacement) {
                    continue;
                }
                set(&mut mutated, &path, replacement.clone());
                let result = std::panic::catch_unwind(|| contract::inspect(&mutated));
                assert!(result.is_ok(), "{code}: panic at {path}={replacement}");
                assert!(
                    !matches!(result.unwrap(), Ok(Some(_))),
                    "{code}: described malformed {path}={replacement}"
                );
            }
        }
    }
}

#[test]
fn migration_modal_incompatible_metadata_headers_and_source_companions_fail_closed() {
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        for (pointer, value) in [
            ("/status", json!("unsupported")),
            ("/baseClass", json!("LeaderUnitCard")),
            ("/card/types", json!(["unit", "leader"])),
            ("/card/unique", json!("true")),
            ("/card/cost", json!(-1)),
            ("/card/pilotText", json!("Pilot")),
            ("/card/epicAction", json!("deploy")),
            ("/attachCondition", json!({})),
            ("/piloting", json!({})),
            ("/abilities/extraSide", json!([])),
            ("/abilities/card/0/optional", json!(true)),
            ("/abilities/card/0/coordinate", json!(false)),
            ("/abilities/card/0/hooks", json!([])),
            ("/abilities/card/0/requiresConfirmation", json!(true)),
            (
                "/abilities/card/0/condition",
                json!({"kind":"const","value":true}),
            ),
            ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
            (
                "/abilities/card/0/then",
                json!({"title":"later","effect":{"kind":"noAction"}}),
            ),
            (
                "/abilities/card/0/ifYouDo",
                json!({"title":"later","effect":{"kind":"noAction"}}),
            ),
        ] {
            let mut mutated = original.clone();
            set(&mut mutated, pointer, value);
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted {pointer}"
            );
        }
        for field in [
            "stateWatchers",
            "historyHelpers",
            "privatePrograms",
            "classMethods",
            "privateClassConstants",
            "inheritedSource",
            "deploymentOverride",
            "setupValidationOverrides",
        ] {
            let mut mutated = original.clone();
            mutated[field] = json!([{}]);
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted {field}"
            );
        }
        let mut mutated = original.clone();
        mutated["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(
            contract::inspect(&mutated).is_err(),
            "{code}: accepted extra registration"
        );
    }
}

#[test]
fn migration_modal_target_chooser_payment_and_unless_timing_mutations_fail_closed() {
    let c = catalogue();
    for code in DESCRIBED {
        let original = entry(&c, code);
        for (pointer, value) in [
            (
                "/abilities/card/0/targetResolvers/0/targetResolver/delayOpponentBeforeCosts",
                json!(false),
            ),
            (
                "/abilities/card/0/targetResolvers/0/targetResolver/choosingPlayerOverride",
                json!("ignored"),
            ),
            (
                "/abilities/card/0/targetResolvers/0/targetResolver/contextBinding",
                json!("chosen"),
            ),
            (
                "/abilities/card/0/targetResolvers/0/targetResolver/cardTypes/value",
                json!("any"),
            ),
            (
                "/abilities/card/0/targetResolvers/0/targetResolver/controller/value",
                json!("self"),
            ),
            (
                "/abilities/card/0/targetResolvers/1/dependsOn",
                json!("otherTarget"),
            ),
            (
                "/abilities/card/0/targetResolvers/1/targetResolver/contextSelectAssignment",
                json!("always"),
            ),
            (
                "/abilities/card/0/targetResolvers/1/targetResolver/showUnresolvable",
                json!(true),
            ),
            (
                "/abilities/card/0/targetResolvers/1/effect/resultBinding",
                json!("select"),
            ),
        ] {
            let mut mutated = original.clone();
            set(&mut mutated, pointer, value);
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted {pointer}"
            );
        }
    }
    for code in ["IC27_185", "SHD_227"] {
        let original = entry(&c, code);
        for (key, value) in [
            ("unlessLegalityMode", json!("none")),
            ("defaultAutomaticallyWhenUnlessIllegal", json!(false)),
            ("mode", json!("select")),
        ] {
            let mut mutated = original.clone();
            mutated["abilities"]["card"][0]["targetResolvers"][1]["targetResolver"][key] = value;
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted {key}"
            );
        }
    }
    let mut payment = entry(&c, "SHD_227");
    payment["abilities"]["card"][0]["targetResolvers"][1]["targetResolver"]["unlessEffect"]
        ["effect"]["system"] = json!("payResourcesWithoutAdjustment");
    assert!(
        contract::inspect(&payment).is_err(),
        "adjusted payment cannot become resource exhaustion"
    );
    let mut draw = entry(&c, "IC27_185");
    draw["abilities"]["card"][0]["targetResolvers"][1]["targetResolver"]["unlessEffect"]
        ["effect"]["target"] = json!("enemyPlayer");
    assert!(
        contract::inspect(&draw).is_err(),
        "the chooser does not become the draw recipient"
    );
}

#[test]
fn migration_modal_deck_capture_owner_and_optional_target_boundaries_fail_closed() {
    let c = catalogue();
    for code in ["HMW_218", "LOF_200"] {
        let original = entry(&c, code);
        let factory = "/abilities/card/0/targetResolvers/1/targetResolver/choices";
        for (suffix, value) in [
            ("/invocationPolicy", json!("onceAtRegistration")),
            ("/captureIdentity", json!("currentZoneLookup")),
            (
                "/entries/0/constructionCaptures/0/evaluateAt",
                json!("effectResolution"),
            ),
            ("/entries/0/constructionCaptures/0/order", json!(1)),
            (
                "/entries/0/constructionCaptures/0/value/path/2",
                json!("anotherUnit"),
            ),
            ("/entries/0/effect/target/path/1", json!("unboundCapture")),
            ("/entries/0/effect/kind", json!("returnToHand")),
        ] {
            let mut mutated = original.clone();
            set(&mut mutated, &format!("{factory}{suffix}"), value);
            assert!(
                contract::inspect(&mutated).is_err(),
                "{code}: accepted {suffix}"
            );
        }
        let mut changed = original.clone();
        changed["abilities"]["card"][0]["targetResolvers"][1]["chooser"]["condition"]["left"]
            ["path"] = json!(["controller"]);
        changed["abilities"]["card"][0]["targetResolvers"][1]["targetResolver"]["chooser"] =
            changed["abilities"]["card"][0]["targetResolvers"][1]["chooser"].clone();
        assert!(
            contract::inspect(&changed).is_err(),
            "owner choice cannot follow current controller"
        );
        let mut optional = original.clone();
        optional["abilities"]["card"][0]["targetResolvers"][0]["targetResolver"]["optional"] =
            json!(code != "LOF_200");
        assert!(
            contract::inspect(&optional).is_err(),
            "{code}: changed optionality"
        );
    }
}

#[test]
fn migration_modal_contracts_do_not_admit_pending_or_unreviewed_families() {
    let c = catalogue();
    let e = Engine::from_catalogue_value(&c).unwrap();
    for code in HELD {
        assert!(
            !e.by_code.contains_key(*code),
            "pending modal source unexpectedly executable: {code}"
        );
        assert!(e.rejected_cards().unwrap().contains_key(*code));
        if !DESCRIBED.contains(code) {
            assert!(
                contract::inspect(&entry(&c, code)).is_err(),
                "unsupported family {code}"
            );
        }
    }
    assert!(
        contract::inspect(&entry(&c, "SEC_046")).is_err(),
        "Plot must not be erased"
    );
    assert_eq!(contract::inspect(&entry(&c, "LOF_254")).unwrap(), None);
}

// Existing-executor controls: actual Lightmaker defeat, not a newly admitted
// assigned-card witness. Friendly targets never create an opposing-arena mode.
#[test]
fn migration_modal_midgame_zero_one_two_modes_follow_enemy_membership_and_skip_extra_prompt() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for owner in [Seat::P1, Seat::P2] {
            for count in 0..=2 {
                let mut s = board(&e);
                s.round = 4;
                s.arena_order_exact = true;
                s.active = owner;
                s.action_player = Some(owner);
                let source = unit(&e, &mut s, owner, "SEC_207");
                let own_ground = unit(&e, &mut s, owner, "LOF_256");
                let own_space = unit(&e, &mut s, owner, "LOF_034");
                let enemy_ground = (count > 0).then(|| unit(&e, &mut s, owner.other(), "LOF_256"));
                let enemy_space = (count == 2).then(|| unit(&e, &mut s, owner.other(), "LOF_034"));
                e.defeat(&mut s, source).unwrap();
                e.finish(&mut s).unwrap();
                assert!(s.player(owner).discard.contains(source));
                if count == 2 {
                    assert_eq!(labels(&e, &s), ["Space", "Ground"]);
                    assert_eq!(s.active, owner);
                    assert!(!labels(&e, &s).iter().any(|label| label == "Cancel"));
                    s = choose(&e, &s, "Ground");
                }
                assert_eq!(s.ability_program, 0);
                assert!(!s.card(own_ground).exhausted);
                assert!(!s.card(own_space).exhausted);
                if let Some(id) = enemy_ground {
                    assert!(s.card(id).exhausted);
                }
                if let Some(id) = enemy_space {
                    assert!(!s.card(id).exhausted);
                }
            }
        }
    }
}

// Source controller chooses the captured death mode; opponent chooses a hand
// identity. Neither stage can cancel, and only the selected identity moves.
#[test]
fn migration_modal_midgame_departed_source_preserves_actor_and_discard_chooser() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for owner in [Seat::P1, Seat::P2] {
            let mut s = board(&e);
            s.round = 4;
            s.active = owner.other();
            s.action_player = Some(owner.other());
            let source = unit(&e, &mut s, owner, "SOR_145");
            let first = add(&e, &mut s, owner.other(), "LOF_254");
            let second = add(&e, &mut s, owner.other(), "LOF_256");
            s.player_mut(owner.other()).hand.push(first).unwrap();
            s.player_mut(owner.other()).hand.push(second).unwrap();
            let own_base_damage = s.card(s.player(owner).base).damage;
            let enemy_base_damage = s.card(s.player(owner.other()).base).damage;
            e.defeat(&mut s, source).unwrap();
            e.finish(&mut s).unwrap();
            assert!(s.player(owner).discard.contains(source));
            assert_eq!(s.active, owner);
            assert_eq!(
                labels(&e, &s),
                [
                    "Deal 3 damage to opponent's base",
                    "The opponent discards a card"
                ]
            );
            let copy = s;
            assert_eq!(copy.zobrist(), s.zobrist());
            let picking = choose(&e, &copy, "The opponent discards a card");
            assert_eq!(picking.decision, Decision::AbilityDiscard);
            assert_eq!(picking.active, owner.other());
            assert!(!labels(&e, &picking).iter().any(|label| label == "Cancel"));
            let done = e.apply(&picking, &e.card_action(&picking, second)).unwrap();
            assert!(done.player(owner.other()).discard.contains(second));
            assert!(done.player(owner.other()).hand.contains(first));
            assert_eq!(done.card(done.player(owner).base).damage, own_base_damage);
            assert_eq!(
                done.card(done.player(owner.other()).base).damage,
                enemy_base_damage
            );
            assert_eq!(done.ability_program, 0);
        }
    }
}
