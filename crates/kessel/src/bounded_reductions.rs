//! Fixed arena reductions: each scans at most the validated card inventory.
//! No reducer callback, list capture, variable seed, or nested traversal is IR.
use crate::ability_ir::Ability;
use serde_json::{json, Value};
pub(crate) const QUERIES: [&str; 3] = ["arenaDamageSum", "arenaUpgradeSum", "arenaPowerMax"];
fn q(name: &str, owner: &str) -> Value {
    json!({"kind":"query","name":name,"receiver":{"kind":"ref","path":[owner]},"args":[]})
}
fn r(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn b(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn c(n: u8) -> Value {
    json!({"kind":"const","value":n})
}
fn unit() -> Value {
    q("isUnit", "card")
}
fn grant(predicate: Value) -> Value {
    json!({"trigger":"constant","costs":[],"targetResolver":{"owner":"self","type":"unit","zone":"arena","predicate":predicate},"effect":{"kind":"gainKeyword","target":"chosen","keyword":"overwhelm","amount":c(1),"duration":"persistent"}})
}
fn max_grant() -> Value {
    grant(b(
        "&&",
        unit(),
        b("===", q("getPower", "card"), q("arenaPowerMax", "player")),
    ))
}
fn normalized(a: &Ability) -> Value {
    // Deserialize defaults back to a minimal canonical registration by comparing
    // typed values, retaining every semantic field and ignoring only display title.
    let mut a = a.clone();
    a.title.clear();
    serde_json::to_value(a).expect("ability serializes")
}
fn same(a: &Ability, mut expected: Value) -> bool {
    expected["title"] = json!("");
    serde_json::from_value::<Ability>(expected).is_ok_and(|v| normalized(a) == normalized(&v))
}
pub(crate) fn power_maximum_grant(a: &Ability) -> bool {
    same(a, max_grant())
}
pub(crate) fn marker(v: &Value) -> bool { crate::ability_ir::contains(v,|v|v["kind"] == "query" && v["name"].as_str().is_some_and(|s| QUERIES.contains(&s))) }
pub(crate) fn leader_entry(e: &Value) -> bool {
    let Ok(s) = serde_json::from_value::<crate::ability_ir::Sides>(e["abilities"].clone()) else {
        return false;
    };
    let other = b(
        "&&",
        b(
            "&&",
            unit(),
            b("===", r(&["card", "controller"]), r(&["player"])),
        ),
        b("!==", r(&["card"]), r(&["source"])),
    );
    leader_metadata(e)
        && e["baseClass"] == "LeaderUnitCard"
        && e["card"]["types"] == json!(["leader"])
        && e["keywordValues"] == json!({"raid":3,"overwhelm":1})
        && plain(e)
        && s.card.is_empty()
        && s.leader.len() == 1
        && s.unit.len() == 1
        && power_maximum_grant(&s.leader[0])
        && same(&s.unit[0], grant(other))
}
fn plain(e: &Value) -> bool {
    e["attachCondition"].is_null()
        && e["piloting"].is_null()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
}
pub(crate) fn entry(e: &Value) -> bool {
    if leader_entry(e) {
        return true;
    }
    if !plain(e)
        || !e["card"]["unique"].is_boolean()
        || !e["card"]["deployBox"].is_null()
        || !e["card"]["deployCost"].is_null()
        || !(e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
        || e["keywordValues"] != json!({})
        || e["card"]["keywords"] != json!([])
    {
        return false;
    }
    let Ok(s) = serde_json::from_value::<crate::ability_ir::Sides>(e["abilities"].clone()) else {
        return false;
    };
    if s.card.len() != 1 || !s.leader.is_empty() || !s.unit.is_empty() {
        return false;
    }
    let a = &s.card[0];
    let ready = json!({"trigger":"whenPlayed","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"any","zone":"arena","predicate":b("&&",b("&&",unit(),b("!==",r(&["card"]),r(&["source"]))),b("<=",q("getPower","card"),q("arenaUpgradeSum","opponent")))},"effect":{"kind":"ready","target":"chosen"}});
    let damage = json!({"trigger":"event","costs":[],"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","promptTitleParts":[{"kind":"const","value":"Choose a unit to deal "},q("arenaDamageSum","player"),{"kind":"const","value":" damage to"}]},"effect":{"kind":"conditional","condition":b(">",q("arenaDamageSum","player"),c(0)),"onTrue":{"kind":"damage","target":"chosen","amount":q("arenaDamageSum","player")},"onFalse":{"kind":"noAction"}}});
    (ready_metadata(e)
        && crate::admission::OrdinarySource::from_entry(e)
            == Some(crate::admission::OrdinarySource::Unit)
        && same(a, ready))
        || (damage_metadata(e)
            && crate::admission::OrdinarySource::from_entry(e)
                == Some(crate::admission::OrdinarySource::Event)
            && same(a, damage))
}

// Reject composed stat -> Overwhelm -> power cycles before attaching executable
// programs. This conservative catalog-wide guard considers every source-
// converted stat registration, including sources that later admission may reject.
// A future selected-pool dependency solver may narrow this explicit rejection.
pub(crate) fn power_feedback(e: &Value) -> bool {
    fn keyword(v: &Value) -> bool { crate::ability_ir::contains(v,|v|v["kind"] == "query" && v["name"] == "hasSomeKeyword" && crate::ability_ir::contains(&v["args"],|v|v["kind"] == "const" && v["value"] == "overwhelm")
            || v["kind"] == "ref" && v["path"] == json!(["card", "keywords", "length"])) }
    e["status"] == "supported"
        && ["card", "leader", "unit"].iter().any(|side| {
            e["abilities"][*side].as_array().is_some_and(|a| {
                a.iter().any(|a| {
                    a["trigger"] == "constant"
                        && crate::ability_ir::contains(&a["effect"],|v|v["kind"] == "modifyStats")
                        && (keyword(&a["condition"]) || keyword(&a["targetResolver"]))
                })
            })
        })
}

fn ready_metadata(e: &Value) -> bool {
    let expected:Value=serde_json::from_str(r##"{"cost":5,"hp":5,"power":4,"text":"When Played: You may ready another unit with power equal to or less than the number of upgrades on enemy units.","deployBox":null,"epicAction":null,"unique":false,"rules":"Abilities that refer to a card’s power include temporary modifiers.\n","upgradePower":null,"upgradeHp":null,"aspects":["villainy","cunning"],"traits":["underworld","vehicle","transport"],"arena":"space","keywords":[]}"##).expect("reviewed semantic metadata");
    expected
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
}

fn damage_metadata(e: &Value) -> bool {
    let expected:Value=serde_json::from_str(r##"{"cost":3,"hp":null,"power":null,"text":"Deal damage to a unit equal to the total amount of damage on all units you control.","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"aspects":["aggression"],"traits":["plan"],"arena":null,"keywords":[]}"##).expect("reviewed semantic metadata");
    expected
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
}

fn leader_metadata(e: &Value) -> bool {
    let expected:Value=serde_json::from_str(r##"{"cost":6,"hp":7,"power":3,"text":"Each friendly unit with the most power among friendly units gains Overwhelm. (When attacking an enemy unit, deal excess damage to the opponent's base.)","deployBox":"Raid 3 (This unit gets +3/+0 while attacking.)\nOverwhelm\nEach other friendly unit gains Overwhelm.","epicAction":"Epic Action: If you control 6 or more resources, deploy this leader.","unique":true,"rules":"Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\nAbilities that refer to a card’s power include temporary modifiers.\nIf multiple friendly units are tied for the most power, they each gain Overwhelm.\n","upgradePower":null,"upgradeHp":null,"aspects":["aggression","villainy"],"traits":["force","underworld"],"arena":"ground","keywords":["overwhelm","raid"]}"##).expect("reviewed semantic metadata");
    expected
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
}

#[cfg(test)]
#[path = "bounded_family_tests.rs"]
mod tests;
