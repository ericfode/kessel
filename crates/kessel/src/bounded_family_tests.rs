//! Cross-family witnesses for the shared identity pair and distinct executable
//! scopes. These exercise the original consumers, not just the classifier.
use super::*;
use crate::ability_ir::{selector_contract, Expr, Sides};
use crate::admission::OrdinarySource;

fn catalog() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn source(catalog: &Value, code: &str) -> Value {
    catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["setCode"] == code)
        .unwrap()
        .clone()
}
fn legacy_pair(entry: &Value) -> Option<OrdinarySource> {
    match entry["baseClass"].as_str() {
        Some("NonLeaderUnitCard") if entry["card"]["types"] == json!(["unit"]) => {
            Some(OrdinarySource::Unit)
        }
        Some("EventCard") if entry["card"]["types"] == json!(["event"]) => {
            Some(OrdinarySource::Event)
        }
        _ => None,
    }
}

#[test]
fn bounded_family_ready_aspect_order_preserves_exact_membership() {
    let mut ready = source(&catalog(), "SHD_189");
    for aspects in [
        json!(["villainy", "cunning"]),
        json!(["cunning", "villainy"]),
    ] {
        ready["card"]["aspects"] = aspects;
        assert!(entry(&ready));
    }
    for aspects in [
        Value::Null,
        json!("cunning"),
        json!([]),
        json!(["cunning"]),
        json!(["cunning", "cunning"]),
        json!(["villainy", "villainy"]),
        json!(["cunning", "heroism"]),
        json!(["cunning", "villainy", "cunning"]),
    ] {
        ready["card"]["aspects"] = aspects;
        assert!(!entry(&ready));
    }
}

#[test]
fn bounded_family_source_identity_matrix_preserves_all_three_consumers_and_leader_exception() {
    let catalog = catalog();
    let ready = source(&catalog, "SHD_189");
    let damage = source(&catalog, "ASH_187");
    let advantage = source(&catalog, "ASH_251");
    let selector = source(&catalog, "SEC_188");
    let leader = source(&catalog, "TS26_005");
    let selector_sides: Sides = serde_json::from_value(selector["abilities"].clone()).unwrap();
    assert!(entry(&ready));
    assert!(entry(&damage));
    assert!(crate::advantage::source_entry(&advantage));
    assert!(selector_contract::native_source_entry(
        &selector,
        &selector_sides
    ));
    assert!(leader_entry(&leader));
    let mut checked = 0;
    for class in [
        Value::Null,
        json!(false),
        json!("NonLeaderUnitCard"),
        json!("EventCard"),
        json!("LeaderUnitCard"),
        json!("UpgradeCard"),
        json!("TokenUnitCard"),
        json!("TokenUpgradeCard"),
    ] {
        for types in [
            Value::Null,
            json!("unit"),
            json!({}),
            json!([]),
            json!(["unit"]),
            json!(["event"]),
            json!(["leader"]),
            json!(["token", "unit"]),
            json!(["unit", "leader"]),
            json!(["unit", "unit"]),
            json!(["event", "unit"]),
            json!([false]),
        ] {
            let mutate = |original: &Value| {
                let mut changed = original.clone();
                changed["baseClass"] = class.clone();
                changed["card"]["types"] = types.clone();
                changed
            };
            let identity = legacy_pair(&mutate(&ready));
            assert_eq!(OrdinarySource::from_entry(&mutate(&ready)), identity);
            assert_eq!(
                entry(&mutate(&ready)),
                identity == Some(OrdinarySource::Unit)
            );
            assert_eq!(
                entry(&mutate(&damage)),
                identity == Some(OrdinarySource::Event)
            );
            assert_eq!(
                crate::advantage::source_entry(&mutate(&advantage)),
                identity.is_some()
            );
            assert_eq!(
                selector_contract::native_source_entry(&mutate(&selector), &selector_sides),
                identity.is_some()
            );
            assert_eq!(
                leader_entry(&mutate(&leader)),
                class == "LeaderUnitCard" && types == json!(["leader"])
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 96);
    eprintln!("checked {checked} class/type pairs across reduction, Advantage and selector gates; leader reduction stays separate");
}

#[test]
fn bounded_family_shared_identity_does_not_grant_watcher_or_recipe_context() {
    let catalog = catalog();
    let history_source = source(&catalog, "ASH_036");
    assert_eq!(
        OrdinarySource::from_entry(&history_source),
        Some(OrdinarySource::Unit)
    );
    assert!(crate::advantage::source_entry(&history_source));
    // The exact phase-history declarations used by this Advantage source are
    // valid locally, while ordinary selectors require a closed watcher context.
    assert!(!crate::admission::SourceContextPolicy::Closed.accepts(&history_source));
    let sides: Sides = serde_json::from_value(history_source["abilities"].clone()).unwrap();
    assert!(!selector_contract::native_source_entry(
        &history_source,
        &sides
    ));
    for watchers in [
        Value::Null,
        json!([]),
        json!({}),
        json!([{"kind":"unknown"}]),
    ] {
        let mut changed = history_source.clone();
        changed["stateWatchers"] = watchers;
        assert_eq!(
            OrdinarySource::from_entry(&changed),
            Some(OrdinarySource::Unit)
        );
        assert!(!crate::advantage::source_entry(&changed));
    }
    for (code, side) in [
        ("SHD_189", "card"),
        ("ASH_187", "card"),
        ("TS26_005", "leader"),
    ] {
        let original = source(&catalog, code);
        for (field, value) in [
            ("trigger", json!("onAttack")),
            ("optional", json!(code != "SHD_189")),
            (
                "targetResolvers",
                json!([{"name":"extra","chooser":"self","targetResolver":{"type":"unit"},"effect":{"kind":"noAction"}}]),
            ),
        ] {
            let mut changed = original.clone();
            changed["abilities"][side][0][field] = value;
            assert_eq!(OrdinarySource::from_entry(&changed), legacy_pair(&original));
            assert!(!entry(&changed), "{code}/{field}");
        }
    }
}

#[test]
fn bounded_family_amount_bindings_and_factory_lifetimes_remain_distinct() {
    const ADVANTAGE_AMOUNT_ERROR: &str =
        "Advantage amount requires zero to five literal tokens or the live opposing unit count";
    for name in QUERIES {
        for owner in ["player", "opponent"] {
            let reduction: Expr = serde_json::from_value(q(name, owner)).unwrap();
            reduction.validate_number().unwrap();
            assert_eq!(
                crate::advantage::validate_amount(&reduction).unwrap_err(),
                ADVANTAGE_AMOUNT_ERROR
            );
        }
    }
    let count = json!({"kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["opponent"]},"args":[{"kind":"const","value":true}]});
    assert!(crate::advantage::validate_amount(
        &serde_json::from_value::<Expr>(count.clone()).unwrap()
    )
    .is_ok());
    assert!(!marker(&count));
    for (field, value) in [
        ("receiver", r(&["player"])),
        ("receiver", r(&["card"])),
        ("args", json!([])),
        ("args", json!([{"kind":"const","value":false}])),
    ] {
        let mut changed = count.clone();
        changed[field] = value;
        assert_eq!(
            crate::advantage::validate_amount(&serde_json::from_value::<Expr>(changed).unwrap())
                .unwrap_err(),
            ADVANTAGE_AMOUNT_ERROR
        );
    }
    let catalog = catalog();
    let mut ready = source(&catalog, "SHD_189");
    // Ready eligibility reads physical candidate power against upgrades attached
    // to opposing hosts, not the initiating source, outer target, or unit count.
    for (pointer, value) in [
        (
            "/abilities/card/0/targetResolver/predicate/right/right",
            count,
        ),
        (
            "/abilities/card/0/targetResolver/predicate/left/right/left/path",
            json!(["target"]),
        ),
        (
            "/abilities/card/0/targetResolver/predicate/right/left/receiver/path",
            json!(["source"]),
        ),
    ] {
        let mut changed = ready.clone();
        *changed.pointer_mut(pointer).expect(pointer) = value;
        assert!(!entry(&changed), "{pointer}");
    }
    // Moving a leaf into a factory/continuation does not preserve the fixed
    // registration's lifetime, even when its arithmetic and selected card match.
    let leaf = ready["abilities"]["card"][0]["effect"].clone();
    ready["abilities"]["card"][0]["effect"] = json!({"kind":"sequence","effects":[leaf]});
    assert!(!entry(&ready));
    let mut grant: Ability = serde_json::from_value({
        let mut raw = max_grant();
        raw["title"] = json!("ties");
        raw
    })
    .unwrap();
    assert!(power_maximum_grant(&grant));
    grant.target_resolver.as_mut().unwrap().owner = "opponent".into();
    assert!(!power_maximum_grant(&grant));
    // Advantage creation ownership and batch timing remain exact enum contracts,
    // not aliases for the persistent Overwhelm grant's recipient selection.
    let advantage = json!({"kind":"giveAdvantage","target":"chosen","amount":{"kind":"const","value":1},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"});
    assert!(serde_json::from_value::<crate::ability_ir::Effect>(advantage.clone()).is_ok());
    for (field, value) in [
        ("createdBy", json!("opponent")),
        ("batch", json!("perTarget")),
        ("tokenOwner", json!("abilityController")),
    ] {
        let mut changed = advantage.clone();
        changed[field] = value;
        assert!(
            serde_json::from_value::<crate::ability_ir::Effect>(changed).is_err(),
            "{field}"
        );
    }
}
