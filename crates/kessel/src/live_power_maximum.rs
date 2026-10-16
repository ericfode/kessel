//! Live, unseeded maximum comparison and textual display. Empty is false for
//! eligibility and "-Infinity" in the source's selector-header metadata.
use crate::ability_ir::Ability;
use serde_json::{json, Value};
pub(crate) const DISPLAY_QUERY: &str = "arenaPowerMaximumText";
const PRINTED_ACTION:&str="Action [Exhaust]: Choose a unit with less power than a friendly unit. It gets +2/+0 for this phase.";
fn q(name: &str, root: &str) -> Value {
    json!({"kind":"query","name":name,"receiver":{"kind":"ref","path":[root]},"args":[]})
}
fn b(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn title(value: Value) -> Value {
    json!([{"kind":"const","value":"Give a unit with less than "},value,{"kind":"const","value":" power +2/+0 for this phase"}])
}
pub(crate) fn prompt_parts(parts: &[crate::ability_ir::Expr]) -> bool {
    let v = serde_json::to_value(parts).expect("prompt expressions serialize");
    v == title(q(DISPLAY_QUERY, "player")) || v == title(q("getPower", "source"))
}
pub(crate) fn predicate() -> Value {
    let greater = b(">", q("getPower", "card"), q("getPower", "target"));
    b(
        "&&",
        q("isUnit", "card"),
        json!({"kind":"query","name":"hasSomeArenaUnit","receiver":{"kind":"ref","path":["player"]},"args":[greater]}),
    )
}
pub(crate) fn registration(deployed: bool) -> Ability {
    let predicate = if deployed {
        b(
            "&&",
            q("isUnit", "card"),
            b("<", q("getPower", "card"), q("getPower", "source")),
        )
    } else {
        predicate()
    };
    let mut a = json!({"title":"","trigger":if deployed{"onAttack"}else{"action"},"costs":if deployed{json!([])}else{json!([{"kind":"exhaustSelf"}])},"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","predicate":predicate,"promptTitleParts":title(q(if deployed{"getPower"}else{DISPLAY_QUERY},if deployed{"source"}else{"player"}))},"effect":{"kind":"modifyStats","target":"chosen","duration":"phase","power":{"kind":"const","value":2},"hp":{"kind":"const","value":0}}});
    if deployed {
        a["optional"] = json!(true);
        a["contextTitle"] = title(q("getPower", "source"));
    }
    serde_json::from_value(a).expect("reviewed complete registration")
}
fn same(a: &Ability, expected: &Ability) -> bool {
    let mut a = a.clone();
    a.title.clear();
    serde_json::to_value(a).ok() == serde_json::to_value(expected).ok()
}
pub(crate) fn marker(v: &Value) -> bool { crate::ability_ir::contains(v,|v|v["kind"] == "query" && v["name"] == DISPLAY_QUERY
        || v["text"] == PRINTED_ACTION) }
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(s) = serde_json::from_value::<crate::ability_ir::Sides>(e["abilities"].clone()) else {
        return false;
    };
    if e["baseClass"] != "LeaderUnitCard"
        || e["card"]["types"] != json!(["leader"])
        || e["keywordValues"] != json!({"support":1})
        || !e["attachCondition"].is_null()
        || !e["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox", "deployCost"]
            .iter()
            .any(|k| !e["card"][*k].is_null())
        || !s.card.is_empty()
        || s.leader.len() != 1
        || s.unit.len() != 1
        || !same(&s.leader[0], &registration(false))
        || !same(&s.unit[0], &registration(true))
    {
        return false;
    }
    let expected:Value=serde_json::from_str(r##"{"cost":6,"hp":6,"power":5,"text":"Action [Exhaust]: Choose a unit with less power than a friendly unit. It gets +2/+0 for this phase.","deployBox":"Support (When you deploy this leader, you may attack with another unit. It gains this unit's other abilities for this attack.)\nOn Attack: You may give a unit with less power than this unit +2/+0 for this phase.","epicAction":"Epic Action: If you control 6 or more resources, deploy this leader.","unique":true,"rules":"Abilities that refer to a card’s power include temporary modifiers.\n","upgradePower":null,"upgradeHp":null,"aspects":["command","heroism"],"traits":["force","jedi"],"arena":"ground","keywords":["support"]}"##).expect("reviewed semantic metadata");
    expected
        .as_object()
        .unwrap()
        .iter()
        .all(|(key, value)| e["card"][key] == *value)
}
