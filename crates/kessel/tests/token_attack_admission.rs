use kessel::ability_ir::{Ability, Catalogue, Program};
use serde_json::{json, Value};

fn entry() -> Value {
    let catalog: Value = serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();
    catalog["cards"].as_array().unwrap().iter().find(|c| c["setCode"] == "TWI_084").unwrap().clone()
}
fn attack() -> Value { entry()["abilities"]["card"][1].clone() }
fn compile(value: Value) -> Result<Program, String> {
    let ability: Ability = serde_json::from_value(value).map_err(|e|e.to_string())?;
    Program::compile(&ability)
}
fn admitted(value: Value) -> bool {
    let code = value["setCode"].as_str().unwrap().to_owned();
    let catalog = Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[value]})).unwrap();
    catalog.cards.contains_key(&code)
}

#[test]
fn friendly_token_attack_retains_complete_source_and_has_no_card_id_exception() {
    assert!(compile(attack()).is_ok());
    assert!(admitted(entry()));
    let mut renamed = entry();
    renamed["setCode"] = json!("PROBE_001");
    renamed["internalName"] = json!("independent-token-unit-commander");
    assert!(admitted(renamed));
}

#[test]
fn token_attack_rejects_owner_filter_alias_and_stat_scope_changes() {
    for receiver in [json!(["opponent"]), json!(["game"]), json!(["source","controller"])] {
        let mut value=attack(); value["effect"]["target"]["receiver"]["path"]=receiver;
        assert!(compile(value).is_err());
    }
    for name in ["isToken", "isUnit", "isUpgrade"] {
        let mut value=attack(); value["effect"]["target"]["predicate"]["right"]["name"]=json!(name);
        assert!(compile(value).is_err(), "{name}");
    }
    for receiver in ["source", "target"] {
        let mut value=attack(); value["effect"]["target"]["predicate"]["right"]["receiver"]["path"]=json!([receiver]);
        assert!(compile(value).is_err());
    }
    for stat in ["power", "hp"] {for amount in [-1, 0, 2] {
        let mut value=attack(); value["effect"][stat]["value"]=json!(amount);
        assert!(compile(value).is_err());
    }}
    for duration in ["persistent", "attack"] {
        let mut value=attack();value["effect"]["duration"]=json!(duration);assert!(compile(value).is_err());
    }
}

#[test]
fn token_attack_rejects_nested_group_selectors_continuations_and_optional_headers() {
    for group in ["sequence", "simultaneous"] {
        let mut value=attack(); value["effect"]=json!({"kind":group,"effects":[value["effect"].clone()]});
        assert!(compile(value).is_err());
    }
    for (key, replacement) in [
        ("optional", json!(true)), ("requiresConfirmation", json!(true)),
        ("condition", json!({"kind":"const","value":true})),
        ("targetResolver", json!({"type":"unit"})),
        ("then", json!({"title":"Continuation","effect":{"kind":"noAction"}})),
        ("ifYouDo", json!({"title":"Continuation","effect":{"kind":"noAction"}})),
        ("costs", json!([{"kind":"exhaustSelf"}]))
    ] {
        let mut value=attack();value[key]=replacement;assert!(compile(value).is_err(),"{key}");
    }
    // This increment does not widen token-list stat effects to other timings.
    for trigger in ["event", "whenPlayed", "whenDefeated", "constant"] {
        let mut value=attack();value["trigger"]=json!(trigger);assert!(compile(value).is_err());
    }
}

#[test]
fn token_attack_requires_ordinary_unit_card_side_metadata() {
    for types in [json!(["unit","leader"]),json!(["unit","token"]),json!(["event"]),json!(["upgrade"])] {
        let mut value=entry();value["card"]["types"]=types;assert!(!admitted(value));
    }
    for class in ["LeaderUnitCard","TokenUnitCard","EventCard","UpgradeCard"] {
        let mut value=entry();value["baseClass"]=json!(class);assert!(!admitted(value));
    }
    for side in ["leader","unit"] {
        let mut value=entry();value["abilities"][side]=json!([attack()]);assert!(!admitted(value));
        let mut value=entry();value["abilities"][side]=json!([{"trigger":"whenPlayed","title":"Unused scalar side","effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}]);assert!(!admitted(value));
    }
    for key in ["piloting","attachCondition"] {
        let mut value=entry();value[key]=json!({});assert!(!admitted(value));
    }
    for key in ["pilotText","pilotBox","pilotingBox"] {
        let mut value=entry();value["card"][key]=json!("Pilot");assert!(!admitted(value));
    }
}
