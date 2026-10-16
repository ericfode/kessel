//! Capability witnesses for structural admission. Run before and after extraction;
//! gate-level exceptions are distinct from complete-card runtime admission.
use crate::ability_ir::{Ability, Catalogue, SCHEMA};
use serde_json::{json, Value};

fn entry(catalog: &Value, code: &str) -> Value {
    catalog["cards"].as_array().unwrap().iter().find(|e| e["setCode"] == code).unwrap().clone()
}

struct Witness {
    name: &'static str,
    ability: Ability,
    accepts: fn(&Ability) -> bool,
}

fn witnesses() -> Vec<Witness> {
    let catalog: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    vec![
        Witness {
            name: "self ongoing",
            ability: serde_json::from_value(entry(&catalog, "ASH_034")["abilities"]["card"][0].clone()).unwrap(),
            accepts: crate::static_ongoing::registration,
        },
        Witness {
            name: "death hook",
            ability: serde_json::from_value(entry(&catalog, "SOR_036")["abilities"]["card"][0].clone()).unwrap(),
            accepts: crate::death_hooks::registration,
        },
        Witness {
            name: "host aura",
            ability: crate::attached_constants::ability(&entry(&catalog, "LAW_150")["abilities"]["card"][1]).unwrap(),
            accepts: crate::attached_constants::host_trait_aura,
        },
        Witness {
            name: "parent title play",
            ability: crate::attached_constants::companion(&entry(&catalog, "LAW_111")["abilities"]["card"][1]).unwrap(),
            accepts: crate::attached_constants::parent_title_play,
        },
    ]
}

#[test]
fn registration_envelopes_reject_controls_bindings_and_continuations() {
    let child = json!({"trigger":"","title":"Unsupported continuation","effect":{"kind":"noAction"}});
    let mutations = [
        ("optional", json!(true)),
        ("requiresConfirmation", json!(true)),
        ("coordinate", json!(true)),
        ("gainedHost", json!(false)),
        ("costs", json!([{"kind":"exhaustSelf"}])),
        ("contextTitle", json!([{"kind":"const","value":"dynamic menu"}])),
        ("condition", json!({"kind":"const","value":true})),
        ("targetResolvers", json!([{"name":"extra","chooser":"self","targetResolver":{"owner":"any","type":"unit"},"effect":{"kind":"noAction"}}])),
        ("then", child.clone()),
        ("ifYouDo", child),
    ];
    for witness in witnesses() {
        assert!((witness.accepts)(&witness.ability), "{} baseline", witness.name);
        for (field, value) in &mutations {
            let mut raw = serde_json::to_value(&witness.ability).unwrap();
            raw[*field] = value.clone();
            let changed: Ability = serde_json::from_value(raw).unwrap();
            assert!(!(witness.accepts)(&changed), "{}: {field}", witness.name);
        }
        for context in [None, Some("host"), Some("upgrade"), Some("unknown")] {
            let mut changed = witness.ability.clone();
            changed.attached_context = context.map(str::to_owned);
            assert_eq!((witness.accepts)(&changed), context == witness.ability.attached_context.as_deref(), "{}: context {context:?}", witness.name);
        }
    }
}

#[test]
fn registration_envelopes_keep_targeting_and_internal_flags_family_specific() {
    for witness in witnesses() {
        let accepts = witness.accepts;
        let mut changed = witness.ability.clone();
        changed.target_resolver = None;
        assert_eq!(accepts(&changed), matches!(witness.name, "self ongoing" | "parent title play"), "{}: absent selector", witness.name);
        changed = witness.ability.clone();
        changed.attached_host = true;
        // This flag is serde-skipped and only set by the strict attached loader.
        assert_eq!(accepts(&changed), matches!(witness.name, "self ongoing" | "death hook"), "{}: internal attached host", witness.name);
        changed = witness.ability.clone();
        changed.attacker_must_survive = true;
        // Complete Program::compile separately rejects this flag outside attack-end.
        assert_eq!(accepts(&changed), witness.name != "death hook", "{}: survival gate", witness.name);
    }
}

#[test]
fn closed_source_contracts_keep_watcher_shapes_and_side_contexts_explicit() {
    let catalog: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    for code in ["ASH_034", "SOR_036", "LAW_150"] {
        let original = entry(&catalog, code);
        let admits = |value: Value| Catalogue::load(&json!({"schema":SCHEMA,"cards":[value]})).unwrap().cards.contains_key(code);
        assert!(admits(original.clone()));
        for field in ["stateWatchers", "historyHelpers", "privatePrograms", "classMethods", "privateClassConstants", "inheritedSource", "deploymentOverride", "setupValidationOverrides"] {
            for value in [Value::Null, json!([]), json!({}), json!([{"kind":"unsupported"}]), json!(false)] {
                let allowed = value.is_null() || value == json!([])
                    || (code == "SOR_036" && field == "stateWatchers" && value == json!({}));
                let mut changed = original.clone();
                changed[field] = value.clone();
                assert_eq!(admits(changed), allowed, "{code}: {field}={value}");
            }
        }
    }
}
