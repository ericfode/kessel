//! Closed collective registrations. A batch is an actual captured event window,
//! never an action counter or all currently pending triggers.
use crate::ability_ir::{Ability, CardPrograms, Program};
use serde_json::{json, Value};
const TEMPLATES: [&str; 2] = [
    r##"{"trigger":"eventHooks","title":"Give an Advantage token to Axe Woves, Undaunted","costs":[],"contextTitle":[{"kind":"const","value":"Give an Advantage token to "},{"kind":"ref","path":["source","title"]},{"kind":"const","value":""}],"collectiveTrigger":true,"effect":{"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":1},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"},"hooks":[{"event":"onCardsDrawn","predicate":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:874:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"player","optional":false}]},"right":{"kind":"ref","path":["player"]}}}]}"##,
    r##"{"trigger":"eventHooks","title":"Exhaust a ground unit","costs":[],"optional":true,"collectiveTrigger":true,"targetResolver":{"owner":"any","type":"unit","zone":"arena","arena":"ground"},"effect":{"kind":"exhaust","target":"chosen"},"hooks":[{"event":"onUpgradeAttached","predicate":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:856:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"parentCard","optional":false}]},"right":{"kind":"ref","path":["source"]}}}]}"##,
];
/// Bounded iterative inspection also covers malformed raw marker values.
pub(crate) fn marker(v: &Value) -> Result<bool, String> {
    let mut todo = vec![(v, 0usize)];
    let mut count = 0usize;
    let mut found = false;
    while let Some((v, d)) = todo.pop() {
        count += 1;
        if count > 32768 || d > 64 {
            return Err("collective metadata inspection capacity exceeded".into());
        }
        match v {
            Value::Object(o) => {
                found |= o.contains_key("collectiveTrigger");
                if count + todo.len() + o.len() > 32768 {
                    return Err("collective metadata inspection capacity exceeded".into());
                }
                todo.extend(o.values().map(|x| (x, d + 1)));
            }
            Value::Array(a) => {
                if count + todo.len() + a.len() > 32768 {
                    return Err("collective metadata inspection capacity exceeded".into());
                }
                todo.extend(a.iter().map(|x| (x, d + 1)));
            }
            _ => {}
        }
    }
    Ok(found)
}
pub(crate) fn reject_pool(v: &Value) -> Result<(), String> {
    if marker(v)? {
        return Err("collectiveTrigger is a source registration, not pool metadata".into());
    }
    Ok(())
}
pub(crate) fn prepare(e: &Value) -> Result<Option<Program>, String> {
    if !marker(e)? {
        return Ok(None);
    }
    let fail = || {
        "collective triggers require the complete closed ordinary source registration and metadata"
            .to_string()
    };
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || e["baseClass"] != "NonLeaderUnitCard"
        || e["card"]["types"] != json!(["unit"])
        || e["card"]["unique"] != true
        || e["abilities"]["leader"] != json!([])
        || e["abilities"]["unit"] != json!([])
        || e["abilities"]["card"].as_array().map(Vec::len) != Some(1)
        || !e["attachCondition"].is_null()
        || !e["piloting"].is_null()
        || [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .any(|k| !e["card"][k].is_null())
        || !(e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
    {
        return Err(fail());
    }
    let a = &e["abilities"]["card"][0];
    let binding = &a["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"];
    if !binding
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
    {
        return Err(fail());
    }
    let kind = (0..2).find(|i| {
        let mut expected: Value = serde_json::from_str(TEMPLATES[*i]).unwrap();
        expected["title"] = a["title"].clone();
        expected["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"] = binding.clone();
        *a == expected
    });
    let Some(kind) = kind else {
        return Err(fail());
    };
    if e["abilities"].as_object().map(|o| o.len()) != Some(3)
        || e["card"]["arena"] != "ground"
        || !["cost", "hp", "power"]
            .iter()
            .all(|k| e["card"][k].as_u64().is_some_and(|n| n <= 1000))
        || e["keywordValues"]
            != if kind == 0 {
                json!({})
            } else {
                json!({"shielded":1})
            }
    {
        return Err(fail());
    }
    if !a["title"]
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 256)
        || e["card"]["keywords"]
            != if kind == 0 {
                json!([])
            } else {
                json!(["shielded"])
            }
    {
        return Err(fail());
    }
    if e["card"]["text"]
        != if kind == 0 {
            json!("When you draw 1 or more cards (including during the regroup phase): Give an Advantage token to this unit.")
        } else {
            json!("Shielded (When you play this unit, give a Shield token to her.)\nWhen 1 or more upgrades attach to this unit (including from Shielded): You may exhaust a ground unit.")
        }
    {
        return Err(fail());
    }
    let mut outside = e.clone();
    outside["abilities"]["card"] = json!([]);
    if marker(&outside)? {
        return Err(fail());
    }
    let mut normalized = a.clone();
    normalized
        .as_object_mut()
        .unwrap()
        .remove("collectiveTrigger");
    normalized.as_object_mut().unwrap().remove("hooks");
    normalized["trigger"] = json!("whenPlayed");
    let title = normalized.as_object_mut().unwrap().remove("contextTitle");
    let ability: Ability = serde_json::from_value(normalized).map_err(|x| x.to_string())?;
    let mut p = Program::compile(&ability)?;
    if let Some(title) = title {
        p.context_title = serde_json::from_value(title).map_err(|x| x.to_string())?;
    }
    p.trigger = if kind == 0 {
        "onCardsDrawn"
    } else {
        "onUpgradeAttached"
    }
    .into();
    p.executable = Err("closed collective event projection uses shared interpreter");
    Ok(Some(p))
}
pub(crate) fn programs(p: Program, id: u16) -> (CardPrograms, Vec<Program>) {
    (
        CardPrograms {
            card: vec![id],
            ..CardPrograms::default()
        },
        vec![p],
    )
}
