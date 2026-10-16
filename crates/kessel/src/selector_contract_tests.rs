use super::*;
use crate::ability_ir::{Ability, Sides};
use serde_json::json;

fn without_titles(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter(|(k, _)| k != "title")
                .map(|(k, v)| (k, without_titles(v)))
                .collect(),
        ),
        Value::Array(array) => Value::Array(array.into_iter().map(without_titles).collect()),
        other => other,
    }
}
fn mutations(value: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let mut removed = object.clone();
                removed.remove(key);
                out.push(Value::Object(removed));
                for changed in mutations(child) {
                    let mut copy = object.clone();
                    copy.insert(key.clone(), changed);
                    out.push(Value::Object(copy));
                }
            }
        }
        Value::Array(array) => {
            out.push(json!([]));
            if array.len() > 1 {
                let mut reversed = array.clone();
                reversed.reverse();
                out.push(Value::Array(reversed));
            }
            for (index, child) in array.iter().enumerate() {
                for changed in mutations(child) {
                    let mut copy = array.clone();
                    copy[index] = changed;
                    out.push(Value::Array(copy));
                }
            }
        }
        Value::Bool(value) => out.push(json!(!value)),
        Value::Number(value) => out.push(json!(value.as_i64().unwrap_or(0) + 1)),
        Value::String(_) => out.push(json!("changed")),
        Value::Null => out.push(json!(false)),
    }
    out
}

#[test]
fn reviewed_arena_queries_preserve_normalized_ability_fingerprints_and_bindings() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();
    let mut originals = Vec::new();
    for entry in catalog["cards"].as_array().unwrap() {
        let Some(kind) = crate::ability_ir::arena_query_source_kind(entry) else {
            continue;
        };
        let sides: Sides = serde_json::from_value(entry["abilities"].clone()).unwrap();
        let ability = if kind == 2 {
            &sides.leader[0]
        } else {
            &sides.card[0]
        };
        assert!(reviewed_arena_query_ability(ability), "kind {kind}");
        originals.push(serde_json::to_value(ability).unwrap());
    }
    assert_eq!(originals.len(), 3);
    let expected = originals
        .iter()
        .cloned()
        .map(without_titles)
        .collect::<Vec<_>>();
    let mut checked = 0;
    for original in &originals {
        let mut candidates = mutations(original);
        for (field, value) in [
            ("gainedHost", json!(false)),
            ("attachedContext", json!("host")),
            ("attackerMustSurvive", json!(true)),
            ("requiresConfirmation", json!(true)),
            ("contextTitle", json!([{"kind":"const","value":"context"}])),
            (
                "targetResolvers",
                json!([{"name":"extra","chooser":"self","targetResolver":{"type":"unit"},"effect":{"kind":"noAction"}}]),
            ),
            (
                "then",
                json!({"title":"continuation","effect":{"kind":"noAction"}}),
            ),
            (
                "ifYouDo",
                json!({"title":"continuation","effect":{"kind":"noAction"}}),
            ),
            (
                "targetResolver",
                json!({"kind":"cardSelection","mode":"single","chooser":"self","resultBinding":"target","defaults":"pinnedTargetResolver","contextBinding":"target","choosingPlayerOverride":"contextOverrideFirst","delayOpponentBeforeCosts":true,"cardTypes":{"kind":"const","value":"unit"},"zones":{"kind":"const","value":"anyAttackable"},"controller":{"kind":"const","value":"any"}}),
            ),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            candidates.push(changed);
        }
        for changed in candidates {
            if let Ok(ability) = serde_json::from_value::<Ability>(changed) {
                let legacy =
                    expected.contains(&without_titles(serde_json::to_value(&ability).unwrap()));
                assert_eq!(
                    reviewed_arena_query_ability(&ability),
                    legacy,
                    "{}",
                    serde_json::to_string(&ability).unwrap()
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 100, "checked {checked} normalized mutations");
    eprintln!("checked {checked} normalized arena registration mutations");
}

#[test]
fn selector_source_context_keeps_ordinary_sides_and_legacy_empty_fields() {
    let baseline =
        json!({"baseClass":"NonLeaderUnitCard","card":{"types":["unit"],"unique":false}});
    let sides = Sides::default();
    assert!(native_source_entry(&baseline, &sides));
    for (base, kinds, accepted) in [
        ("NonLeaderUnitCard", json!(["unit"]), true),
        ("EventCard", json!(["event"]), true),
        ("LeaderUnitCard", json!(["leader"]), false),
        ("TokenUnitCard", json!(["unit"]), false),
        ("UpgradeCard", json!(["upgrade"]), false),
        ("NonLeaderUnitCard", json!(["unit", "leader"]), false),
        ("NonLeaderUnitCard", json!("unit"), false),
        ("EventCard", json!(["unit"]), false),
    ] {
        let mut entry = baseline.clone();
        entry["baseClass"] = json!(base);
        entry["card"]["types"] = kinds;
        assert_eq!(native_source_entry(&entry, &sides), accepted, "{base}");
    }
    for field in [
        "stateWatchers",
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "inheritedSource",
        "privateClassConstants",
        "deploymentOverride",
        "setupValidationOverrides",
    ] {
        for (value, accepted) in [
            (Value::Null, true),
            (json!([]), true),
            (json!({}), false),
            (json!([false]), false),
        ] {
            let mut entry = baseline.clone();
            entry[field] = value;
            assert_eq!(native_source_entry(&entry, &sides), accepted, "{field}");
        }
    }
    let ability = json!({"title":"scope","trigger":"whenPlayed","effect":{"kind":"noAction"}});
    for side in ["leader", "unit", "card"] {
        let mut raw = json!({"leader":[],"unit":[],"card":[]});
        raw[side] = json!([ability.clone()]);
        let sides: Sides = serde_json::from_value(raw).unwrap();
        assert_eq!(
            native_source_entry(&baseline, &sides),
            side == "card",
            "{side}"
        );
    }
}

#[test]
fn named_selector_contracts_preserve_scalar_modes_bindings_and_domain_boundaries() {
    for (name, chooser, accepted) in [
        ("first", "self", true),
        ("1_first", "self", true),
        ("target", "self", false),
        ("first", "opponent", false),
        ("", "self", false),
        ("first.dot", "self", false),
    ] {
        assert_eq!(own_named_binding(name, chooser), accepted);
    }
    let baseline = json!({"type":"unit","zone":"arena"});
    for (field, value, accepted) in [
        ("zone", json!("anyAttackable"), true),
        ("zone", json!("discard"), false),
        ("arena", json!("ground"), true),
        ("arena", json!("space"), true),
        ("arena", json!("mystery"), false),
        ("type", json!("leaderUnit"), false),
        ("mode", json!("single"), false),
        ("mode", json!("upTo"), false),
        ("count", json!({"kind":"const","value":1}), false),
    ] {
        let mut value_with_field = baseline.clone();
        value_with_field[field] = value;
        let resolver: TargetResolver = serde_json::from_value(value_with_field).unwrap();
        assert_eq!(scalar_arena_unit(&resolver), accepted, "{field}");
    }
}

#[test]
fn emitted_arena_scope_keeps_exact_ground_and_space_operands() {
    let mut checked = 0;
    for arena in ["groundArena", "spaceArena"] {
        let raw = json!({"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":arena}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":arena},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":arena}}}}});
        let expression: Expr = serde_json::from_value(raw.clone()).unwrap();
        assert!(crate::ability_ir::emitted_arena_scope(&expression, arena));
        let other = if arena == "groundArena" {
            "spaceArena"
        } else {
            "groundArena"
        };
        assert!(!crate::ability_ir::emitted_arena_scope(&expression, other));
        for changed in mutations(&raw) {
            if let Ok(expression) = serde_json::from_value::<Expr>(changed) {
                assert_eq!(
                    crate::ability_ir::emitted_arena_scope(&expression, arena),
                    serde_json::to_value(&expression).unwrap() == raw
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
    eprintln!("checked {checked} exact emitted arena-scope mutations across ground and space");
}

#[test]
fn expr_search_outer_candidate_preserves_collection_and_compiled_pruning() {
    let target=Expr::Ref{path:vec!["target".into()]};
    let compiled=Expr::Compiled(crate::ability_ir::CompiledExpr{original:Box::new(target.clone()),index:0});
    let collection=Expr::Collection{name:"some".into(),receiver:Box::new(target.clone()),args:vec![],limit:256};
    assert!(!captures_outer_candidate(&compiled));assert!(!captures_outer_candidate(&collection));
    let native=Expr::NativeTokenPredicate{operation:"hasWeakness".into(),receiver:Box::new(target.clone()),domain:"physicalAttachments".into(),token_type:"weakness".into(),optional_receiver:false,optional_call:false};
    assert!(captures_outer_candidate(&native));
    assert!(captures_outer_candidate(&Expr::ArenaList{receiver:Box::new(Expr::number(0)),predicate:Box::new(target)}));
}

#[test]
fn contextual_stat_read_keeps_selector_scope_and_first_error() {
    let descriptor = |predicate| {
        let mut selection:NativeCardSelection=serde_json::from_value(json!({"kind":"cardSelection","mode":"single","chooser":"self","resultBinding":"target","defaults":"pinnedTargetResolver","contextBinding":"target","choosingPlayerOverride":"contextOverrideFirst","delayOpponentBeforeCosts":true,"cardTypes":{"kind":"const","value":"base"},"zones":{"kind":"const","value":"base"},"controller":{"kind":"const","value":"any"}})).unwrap();
        selection.predicate = Some(predicate);
        selection
    };
    let power = Expr::Ref {
        path: vec!["selectionCandidate".into(), "power".into()],
    };
    let expected = "native selector unit-stat predicate requires an exclusively unit type domain";
    for expr in [
        power.clone(),
        Expr::NativeTokenPredicate {
            operation: "hasWeakness".into(),
            receiver: Box::new(power.clone()),
            domain: "physicalAttachments".into(),
            token_type: "weakness".into(),
            optional_receiver: false,
            optional_call: false,
        },
        Expr::Conditional {
            condition: Box::new(Expr::number(0)),
            then: Box::new(Expr::number(0)),
            r#else: Box::new(power.clone()),
        },
    ] {
        assert_eq!(descriptor(expr).lift().unwrap_err(), expected);
    }
    let compiled = Expr::Compiled(crate::ability_ir::CompiledExpr {
        original: Box::new(power.clone()),
        index: 0,
    });
    assert_eq!(
        descriptor(compiled).lift().unwrap_err(),
        "native selector input cannot contain an executable binding"
    );
    let collection = Expr::Collection {
        name: "some".into(),
        receiver: Box::new(power),
        args: vec![],
        limit: 256,
    };
    assert_eq!(
        descriptor(collection).lift().unwrap_err(),
        "native selector requires a separately reviewed collection/history candidate scope"
    );
}
