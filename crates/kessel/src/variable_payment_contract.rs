//! Closed bounded choose-number, unadjusted-payment, self-Experience source contract.
//! Source JSON is retained; only the complete reviewed registration is prepared.
use crate::ability_ir::{Instruction, Program};
use serde_json::{json, Value};
pub(crate) const REASON:&str="variable payment requires complete ordinary bounded-number/unadjusted-payment/self-Experience registration";
const EXPECTED: &str = r###"{"trigger":"whenPlayed","title":"Pay up to 6 resources. For each resource paid this way, give an Experience token to this unit","costs":[],"targetResolver":{"kind":"numberSelection","mode":"chooseNumber","min":{"kind":"const","value":0},"max":{"kind":"math","name":"min","args":[{"kind":"const","value":6},{"kind":"ref","path":["player","readyResourceCount"]}]},"chooser":{"kind":"const","value":"self"},"resultBinding":"select","resolverName":"target","namedResultStorage":"selects[name].choice","contextSelectAssignment":"onlyWhenNameIsTarget","resultRepresentation":"decimalString","rangeAssertions":["integerMinimum","integerMaximum","maximumAtLeastMinimum"],"sameEndpoints":"automaticStringSelection","logSelection":true,"defaults":"NumberTargetResolver"},"effect":{"kind":"sequence","effects":[{"kind":"payUnadjustedResources","target":"ownPlayer","amount":{"kind":"selectionNumberCoercion","operation":"parseInt","args":[{"kind":"ref","path":["select"]}],"semantics":"javascriptParseInt","invalidResult":"NaN"},"paymentType":"exhaustResources","isCost":true,"allowCostAdjustments":false},{"kind":"giveExperience","target":"self","amount":{"kind":"selectionNumberCoercion","operation":"parseInt","args":[{"kind":"ref","path":["select"]}],"semantics":"javascriptParseInt","invalidResult":"NaN"}}]}}"###;
pub(crate) fn prepare(entry: &Value) -> Result<Option<Program>, String> {
    let abilities = &entry["abilities"];
    if !crate::ability_ir::contains(abilities, |v| {
        matches!(
            v["kind"].as_str(),
            Some("numberSelection" | "payUnadjustedResources" | "selectionNumberCoercion")
        )
    }) {
        return Ok(None);
    }
    let source = entry["status"] == "supported"
        && crate::admission::SourceContextPolicy::Closed.accepts(entry)
        && entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && entry["card"]["unique"].is_boolean()
        && entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        && entry["card"]["hp"]
            .as_u64()
            .is_some_and(|n| (1..=1000).contains(&n))
        && entry["card"]["power"].as_u64().is_some_and(|n| n <= 1000)
        && matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
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
        .all(|k| entry["card"][k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["card"]["keywords"] == json!([])
        && entry["keywordValues"] == json!({});
    if !source
        || abilities.as_object().is_none_or(|o| o.len() != 3)
        || abilities["leader"] != json!([])
        || abilities["unit"] != json!([])
        || abilities["card"].as_array().is_none_or(|a| a.len() != 1)
        || abilities["card"][0]
            != serde_json::from_str::<Value>(EXPECTED).expect("reviewed number contract")
    {
        return Err(REASON.into());
    }
    Ok(Some(Program {
        attached_host: false,
        piloting_ability: false,
        pilot_unit_ability: false,
        hooks: vec![],
        gained_host: None,
        attached_context: None,
        title: abilities["card"][0]["title"].as_str().unwrap().into(),
        context_title: vec![],
        trigger: "whenPlayed".into(),
        attacker_must_survive: false,
        coordinate: false,
        costs: vec![],
        optional: false,
        requires_confirmation: false,
        condition: None,
        instructions: vec![
            Instruction::ChooseNumber,
            Instruction::PayUnadjustedSelectedResources,
            Instruction::EndStep,
            Instruction::GiveSelectedExperience,
            Instruction::EndStep,
        ],
        event_end: 5,
        executable: Err("number selection requires shared bounded executor"),
        select_ends: vec![],
    }))
}
