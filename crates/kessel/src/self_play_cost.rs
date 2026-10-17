//! Native, live self-play reductions. These registrations are extracted into a
//! Definition cost rule, never executed as an ability or replaced with a no-op.
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelfPlayCost {
    FriendlyDamagedUnits,
    AllDamagedGroundUnits,
    FriendlyLeaderUnits,
    FriendlyRebelUnitsAndLeaders,
}

pub(crate) fn marker(v: &Value) -> bool {
    v["kind"] == "decreaseSelfPlayCost"
        || v["trigger"] == "costAdjustment"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}

fn r(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn c(v: Value) -> Value {
    json!({"kind":"const","value":v})
}
fn b(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn q(name: &str, receiver: Value, args: Vec<Value>) -> Value {
    json!({"kind":"query","name":name,"receiver":receiver,"args":args})
}
fn count(owner: &[&str], predicate: Value) -> Value {
    q("arenaUnitCount", r(owner), vec![predicate])
}
fn damaged(test: &str) -> Value {
    b(
        "&&",
        q(test, r(&["card"]), vec![]),
        b(">", r(&["card", "damage"]), c(json!(0))),
    )
}
fn ground() -> Value {
    let arena = c(json!("groundArena"));
    b(
        "||",
        json!({"kind":"unary","op":"!","value":arena}),
        b(
            "||",
            b("===", arena.clone(), c(json!("anyArena"))),
            b("===", r(&["card", "zoneName"]), arena),
        ),
    )
}
fn registration(rule: SelfPlayCost) -> Value {
    let amount = match rule {
        SelfPlayCost::FriendlyLeaderUnits => count(
            &["adjusterPlayer"],
            b(
                "&&",
                c(json!(true)),
                b(
                    "&&",
                    q("isUnit", r(&["card"]), vec![]),
                    q("isLeader", r(&["card"]), vec![]),
                ),
            ),
        ),
        SelfPlayCost::FriendlyRebelUnitsAndLeaders => q(
            "controlledInPlayCount",
            r(&["adjusterPlayer"]),
            vec![
                json!({"kind":"cardFilter","type":["unit","leader","leaderUpgrade"],"trait":"rebel"}),
            ],
        ),
        SelfPlayCost::FriendlyDamagedUnits => count(
            &["adjusterPlayer"],
            b("&&", c(json!(true)), damaged("isUnit")),
        ),
        SelfPlayCost::AllDamagedGroundUnits => {
            let predicate = b(
                "&&",
                b("&&", c(json!(true)), ground()),
                damaged("canBeDamaged"),
            );
            b(
                "+",
                count(&["adjusterPlayer"], predicate.clone()),
                count(&["adjusterPlayer", "opponent"], predicate),
            )
        }
    };
    json!({"trigger":"costAdjustment","costs":[],"effect":{
        "kind":"decreaseSelfPlayCost","sourceZone":"any","targetPlayers":"any",
        "matchSourceIdentity":true,"matchPrintedType":true,"stage":"standard_0",
        "amount":{"kind":"boundCallback","parameters":["adjustedCard","adjusterPlayer"],"timing":"costAdjustment","value":amount},
        "matchAbilityCosts":false,"matchCardEffectResourcePayments":false
    }})
}

/// Complete ordinary unit/event contract. No card names, IDs, or source hashes select
/// a rule. In particular an additional unsupported registration rejects the card.
pub(crate) fn extract(entry: &Value) -> Result<Option<SelfPlayCost>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    if let Some(rule) = event_rule(entry) {
        return Ok(Some(rule));
    }
    let plain = crate::static_ongoing::closed_source_context(entry)
        && entry["status"] == "supported"
        && entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && entry["card"]["unique"].is_boolean()
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
        && entry["abilities"]["leader"] == json!([])
        && entry["abilities"]["unit"] == json!([]);
    let mut abilities = entry["abilities"].clone();
    if let Some(list) = abilities["card"].as_array_mut() {
        for a in list {
            if let Some(o) = a.as_object_mut() {
                o.remove("title");
            }
        }
    }
    for rule in [
        SelfPlayCost::FriendlyDamagedUnits,
        SelfPlayCost::AllDamagedGroundUnits,
    ] {
        let (text, keywords, values) = match rule {
            SelfPlayCost::FriendlyDamagedUnits => ("This unit costs 1 resource less to play for each friendly damaged unit.", json!([]), json!({})),
            SelfPlayCost::AllDamagedGroundUnits => ("This unit costs 1 resource less to play for each damaged ground unit.\nOverwhelm (When attacking an enemy unit, deal excess damage to the opponent's base.)", json!(["overwhelm"]), json!({"overwhelm":1})),
            SelfPlayCost::FriendlyLeaderUnits | SelfPlayCost::FriendlyRebelUnitsAndLeaders => unreachable!("event rules were handled separately"),
        };
        if plain
            && abilities == json!({"leader":[],"unit":[],"card":[registration(rule)]})
            && entry["card"]["text"] == text
            && entry["card"]["keywords"] == keywords
            && entry["keywordValues"] == values
        {
            return Ok(Some(rule));
        }
    }
    Err("self-play cost requires a complete reviewed ordinary unit/event contract".into())
}

/// Only these complete ordinary event sources can retain an executable sibling.
fn event_rule(entry: &Value) -> Option<SelfPlayCost> {
    let plain = crate::static_ongoing::closed_source_context(entry)
        && entry["status"] == "supported"
        && entry["baseClass"] == "EventCard"
        && entry["card"]["types"] == json!(["event"])
        && entry["card"]["unique"] == false
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
            "power",
            "hp",
            "arena",
        ]
        .iter()
        .all(|k| entry["card"][*k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["card"]["keywords"] == json!([])
        && entry["keywordValues"] == json!({});
    if !plain {
        return None;
    }
    let mut abilities = entry["abilities"].clone();
    if let Some(list) = abilities["card"].as_array_mut() {
        for ability in list {
            if let Some(object) = ability.as_object_mut() {
                object.remove("title");
            }
        }
    }
    for rule in [
        SelfPlayCost::FriendlyLeaderUnits,
        SelfPlayCost::FriendlyRebelUnitsAndLeaders,
    ] {
        let (text, cost, aspects, traits, body) = match rule {
            SelfPlayCost::FriendlyLeaderUnits => (
                "This event costs 1 resource less to play for each friendly leader unit.\nGive an Experience token to each of up to 3 units.",
                3, json!(["command"]), json!(["tactic"]),
                json!({"trigger":"event","costs":[],"targetResolver":{
                    "owner":"any","type":"unit","zone":"anyAttackable","mode":"upTo",
                    "count":{"kind":"const","value":3},"optional":true},
                    "effect":{"kind":"giveExperience","target":"chosen","amount":{"kind":"const","value":1}}})),
            SelfPlayCost::FriendlyRebelUnitsAndLeaders => (
                "This card costs 1 resource less to play for each friendly Rebel unit and leader.\nDraw 2 cards.",
                4, json!(["aggression", "heroism"]), json!(["rebel", "plan"]),
                json!({"trigger":"event","costs":[],"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":2}}})),
            _ => unreachable!(),
        };
        if entry["card"]["text"] == text
            && entry["card"]["cost"] == cost
            && entry["card"]["aspects"] == aspects
            && entry["card"]["traits"] == traits
            && entry["card"]["rules"].is_null()
            && abilities == json!({"leader":[],"unit":[],"card":[registration(rule),body]})
        {
            return Some(rule);
        }
    }
    None
}

/// The raw leader-query source guard may bypass only the cost query in this
/// complete, validated source. Its retained event body has no leader queries.
pub(crate) fn reviewed_leader_count_event(entry: &Value) -> bool {
    matches!(event_rule(entry), Some(SelfPlayCost::FriendlyLeaderUnits))
}

/// Remove only a validated cost registration; executable event siblings must
/// survive normal program compilation and native token dependency validation.
pub(crate) fn normalize(entry: &Value, abilities: &mut Value) -> Result<(), String> {
    if extract(entry)?.is_some() {
        let list = abilities["card"]
            .as_array_mut()
            .ok_or("self-play source requires card registrations")?;
        let at = list
            .iter()
            .position(|a| a["trigger"] == "costAdjustment")
            .ok_or("validated self-play cost registration missing")?;
        list.remove(at);
    }
    Ok(())
}
