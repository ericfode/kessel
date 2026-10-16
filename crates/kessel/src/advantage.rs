//! Native Advantage is a physical +1/+0 token upgrade. Its manual attack-end
//! ability is registered after damage handlers/state checks, before pending physical
//! defeats move the involved hosts (UnitProperties.ts / EventWindow.ts).
use crate::ability_ir::Expr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Creator {
    #[serde(rename = "self")]
    SelfPlayer,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Batch {
    #[serde(rename = "singleCreationEvent")]
    SingleCreationEvent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Owner {
    #[serde(rename = "recipientController")]
    RecipientController,
}
pub fn marker(v: &Value) -> bool {
    v["kind"] == "giveAdvantage"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
pub fn validate_amount(amount: &Expr) -> Result<(), String> {
    // One bounded native creation batch. Dynamic values other than the exact
    // live opposing unit count need a separate expression/timing contract.
    if matches!(amount,Expr::Const{value} if value.as_u64().is_some_and(|n|n<=5)) {
        return Ok(());
    }
    if matches!(amount,Expr::Query{name,receiver,args} if name=="arenaUnitCount"&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["opponent"])&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value==true))
    {
        return amount.validate_number();
    }
    Err(
        "Advantage amount requires zero to five literal tokens or the live opposing unit count"
            .into(),
    )
}
pub fn source_entry(entry: &Value) -> bool {
    crate::admission::closed_source_helpers(entry)
        && crate::attack_history::declarations(entry)
        && crate::admission::OrdinarySource::from_entry(entry).is_some()
        && entry["card"]["unique"].is_boolean()
        && entry["abilities"]["leader"] == serde_json::json!([])
        && entry["abilities"]["unit"] == serde_json::json!([])
        && entry["attachCondition"].is_null()
        && entry["piloting"].is_null()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| entry["card"][*k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
}
pub fn reviewed_entry(entry: &Value) -> bool {
    let expected:Value=serde_json::from_str(r##"{
  "title": "Advantage",
  "subtitle": "",
  "cost": 0,
  "hp": 0,
  "power": 1,
  "text": "When attached unit's attack or defense ends: Defeat this upgrade.",
  "deployBox": null,
  "epicAction": "",
  "unique": false,
  "rules": "When Attack Ends/When Defense Ends abilities trigger at the same time as units deal combat damage.\n",
  "upgradePower": 1,
  "upgradeHp": 0,
  "id": "5844562972",
  "aspects": [],
  "traits": [
    "innate"
  ],
  "keywords": [],
  "types": [
    "token",
    "upgrade"
  ],
  "setId": {
    "set": "ASH"
  },
  "internalName": "advantage"
}"##).expect("reviewed native Advantage metadata");
    entry["setCode"] == "TOKEN_ADVANTAGE"
        && entry["id"] == "5844562972"
        && entry["internalName"] == "advantage"
        && entry["sourcePath"] == "game/cards/08_ASH/tokens/Advantage.ts"
        && entry["sourceHash"] == "1280861b9a94572ecce2c1d62fb5fd93c11b08609495c3d9f60e656d153e3119"
        && entry["baseClass"].is_null()
        && entry["card"] == expected
        && entry["keywordValues"] == serde_json::json!({})
        && entry["abilities"] == serde_json::json!({"leader":[],"unit":[],"card":[]})
        && entry["attachCondition"].is_null()
        && entry["piloting"].is_null()
}
