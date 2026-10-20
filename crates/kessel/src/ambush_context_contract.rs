//! Closed source envelope for the whole live Ambush attack condition.
use super::Ability;
use serde_json::{json, Value};
pub(crate) fn marker(v: &Value) -> bool {
    super::contains(v, |v| {
        v["kind"] == "query" && v["name"] == "isAttackingUsingAmbush"
    })
}
pub(crate) fn registration(a: &Ability) -> bool {
    let Ok(value) = serde_json::to_value(a) else {
        return false;
    };
    let Ok(expected) = serde_json::from_value::<Ability>(registration_value()) else {
        return false;
    };
    // Titles are legal-action text; preserve a bounded nonempty source title.
    let mut expected = serde_json::to_value(expected).unwrap();
    if a.title.is_empty() || a.title.len() > 256 {
        return false;
    }
    expected["title"] = value["title"].clone();
    value == expected
}
fn registration_value() -> Value {
    serde_json::from_str(r###"{"condition":{"kind":"query","name":"isAttackingUsingAmbush","receiver":{"kind":"ref","path":["source"]},"args":[]},"costs":[],"effect":{"duration":"persistent","hp":{"kind":"const","value":0},"kind":"modifyStats","power":{"kind":"const","value":2},"target":"self"},"title":"While attacking with Ambush, this unit gets +2/+0","trigger":"constant"}"###).unwrap()
}
pub(crate) fn entry(e: &Value) -> bool {
    const ENTRY_KEYS: &[&str] = &[
        "setCode",
        "id",
        "internalName",
        "sourcePath",
        "sourceHash",
        "status",
        "className",
        "baseClass",
        "abilities",
        "card",
        "dataHash",
        "keywordValues",
        "requiredCapabilities",
        "diagnostics",
        "attachCondition",
        "piloting",
    ];
    const CARD_KEYS: &[&str] = &[
        "title",
        "subtitle",
        "cost",
        "hp",
        "power",
        "text",
        "deployBox",
        "epicAction",
        "unique",
        "rules",
        "upgradePower",
        "upgradeHp",
        "id",
        "aspects",
        "traits",
        "arena",
        "keywords",
        "types",
        "setId",
        "setCodes",
        "internalName",
    ];
    if !e
        .as_object()
        .is_some_and(|o| o.keys().all(|k| ENTRY_KEYS.contains(&k.as_str())))
        || !e["card"]
            .as_object()
            .is_some_and(|o| o.keys().all(|k| CARD_KEYS.contains(&k.as_str())))
        || !e["abilities"].as_object().is_some_and(|o| {
            o.keys()
                .all(|k| ["leader", "unit", "card"].contains(&k.as_str()))
        })
        || e["status"] != "supported"
        || e["diagnostics"] != json!([])
        || !e["sourcePath"].as_str().is_some_and(|s| !s.is_empty())
        || !["sourceHash", "dataHash"].iter().all(|k| {
            e[*k].as_str().is_some_and(|s| {
                s.len() == 64
                    && s.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
        })
    {
        return false;
    }
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || crate::admission::OrdinarySource::from_entry(e)
            != Some(crate::admission::OrdinarySource::Unit)
        || !e["attachCondition"].is_null()
        || !e["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox", "deployCost"]
            .iter()
            .any(|k| !e["card"][*k].is_null())
        || e["keywordValues"] != json!({"ambush":1})
    {
        return false;
    }
    let profile:Value=serde_json::from_str(r###"{"arena":"space","aspects":["cunning","heroism"],"cost":5,"deployBox":null,"epicAction":"","hp":6,"keywords":["ambush"],"power":3,"rules":"Ambush is an ability that triggers at the same time as “When Played” abilities. The unit does not have to be ready in order to attack.\n","text":"Ambush (When you play this unit, it may attack an enemy unit.)\nWhile attacking using Ambush, this unit gets +2/+0.","traits":["creature"],"types":["unit"],"unique":false,"upgradeHp":null,"upgradePower":null}"###).unwrap();
    if !profile
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
    {
        return false;
    }
    if e["abilities"]["leader"] != json!([]) || e["abilities"]["unit"] != json!([]) {
        return false;
    }
    let Some(cards) = e["abilities"]["card"].as_array() else {
        return false;
    };
    cards.len() == 1
        && serde_json::from_value::<Ability>(cards[0].clone())
            .ok()
            .is_some_and(|a| registration(&a))
}
