//! Closed Standard_0 self-play reductions whose eligibility depends on the
//! physical attachment host. This module performs no board or prompt traversal.
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttachmentPlayCost {
    Imperial,
    Mandalorian,
    Unique,
}
impl AttachmentPlayCost {
    /// Caller supplies live host facts only after ordinary attachment legality
    /// has been established. The upgrade controller need not own the host.
    pub(crate) fn qualifies(self, unique: bool, has_trait: impl Fn(&str) -> bool) -> bool {
        match self {
            Self::Imperial => has_trait("imperial"),
            Self::Mandalorian => has_trait("mandalorian"),
            Self::Unique => unique,
        }
    }
    /// `total` already includes printed cost and aspect penalties. No chosen
    /// host is inferred from another matching unit or from an ability context.
    pub(crate) fn apply(self, total: u16, qualifies: bool) -> u16 {
        if qualifies {
            total.saturating_sub(1)
        } else {
            total
        }
    }
}

fn cost_marker(v: &Value) -> bool {
    v["kind"] == "decreaseSelfPlayCost"
        || v["trigger"] == "costAdjustment"
        || match v {
            Value::Object(o) => o.values().any(cost_marker),
            Value::Array(a) => a.iter().any(cost_marker),
            _ => false,
        }
}
fn attachment_marker(v: &Value) -> bool {
    match v {
        Value::Object(o) => {
            o.contains_key("attachTargetCondition") || o.values().any(attachment_marker)
        }
        Value::Array(a) => a.iter().any(attachment_marker),
        _ => false,
    }
}
fn registration(rule: AttachmentPlayCost) -> Value {
    let value = match rule {
        AttachmentPlayCost::Imperial | AttachmentPlayCost::Mandalorian => json!({
            "kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["attachTarget"]},
            "args":[{"kind":"const","value":if rule == AttachmentPlayCost::Imperial {"imperial"} else {"mandalorian"}}]
        }),
        AttachmentPlayCost::Unique => json!({"kind":"ref","path":["attachTarget","unique"]}),
    };
    json!({"trigger":"costAdjustment","costs":[],"effect":{
        "kind":"decreaseSelfPlayCost","sourceZone":"any","targetPlayers":"any",
        "matchSourceIdentity":true,"matchPrintedType":true,"stage":"standard_0",
        "amount":{"kind":"const","value":1},"matchAbilityCosts":false,
        "matchCardEffectResourcePayments":false,"attachTargetCondition":{
            "kind":"boundCallback","parameters":["attachTarget"],"timing":"costAttachmentEligibility","value":value
        }
    }})
}

/// Extract the complete reviewed ordinary-upgrade contract. Names, set codes,
/// class names and hashes do not choose the runtime rule. Unknown registrations,
/// source contexts, modes and callback fields reject the entire card.
pub(crate) fn extract(entry: &Value) -> Result<Option<AttachmentPlayCost>, String> {
    let upgrade = entry["baseClass"] == "UpgradeCard"
        || entry["card"]["types"]
            .as_array()
            .is_some_and(|types| types.iter().any(|t| t == "upgrade"));
    if !cost_marker(&entry["abilities"]) || !(upgrade || attachment_marker(&entry["abilities"])) {
        return Ok(None);
    }
    let plain = crate::admission::SourceContextPolicy::Closed.accepts(entry)
        && entry["status"] == "supported"
        && entry["baseClass"] == "UpgradeCard"
        && entry["card"]["types"] == json!(["upgrade"])
        && entry["card"]["unique"] == false
        && entry["attachCondition"].is_null()
        && entry["piloting"].is_null()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "arena",
        ]
        .iter()
        .all(|key| entry["card"][*key].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["card"]["rules"].is_null()
        && entry["card"]["keywords"] == json!([])
        && entry["keywordValues"] == json!({});
    let mut abilities = entry["abilities"].clone();
    if let Some(list) = abilities["card"].as_array_mut() {
        for ability in list {
            if let Some(object) = ability.as_object_mut() {
                if object.get("title").is_some_and(|title| !title.is_string()) {
                    return Err("attachment cost requires a plain registration title".into());
                }
                object.remove("title");
            }
        }
    }
    for rule in [
        AttachmentPlayCost::Imperial,
        AttachmentPlayCost::Mandalorian,
        AttachmentPlayCost::Unique,
    ] {
        let (text, cost, power, hp, aspects, traits) = match rule {
            AttachmentPlayCost::Imperial => (
                "This upgrade costs 1 resource less to play on an Imperial unit.",
                2,
                1,
                2,
                json!([]),
                json!(["imperial"]),
            ),
            AttachmentPlayCost::Mandalorian => (
                "This upgrade costs 1 resource less to play on a Mandalorian unit.",
                2,
                2,
                0,
                json!([]),
                json!(["mandalorian"]),
            ),
            AttachmentPlayCost::Unique => (
                "This upgrade costs 1 resource less to play on a <uq> unit.",
                4,
                3,
                3,
                json!(["vigilance"]),
                json!(["learned"]),
            ),
        };
        if plain
            && entry["card"]["text"] == text
            && entry["card"]["cost"] == cost
            && entry["card"]["power"] == power
            && entry["card"]["upgradePower"] == power
            && entry["card"]["hp"] == hp
            && entry["card"]["upgradeHp"] == hp
            && entry["card"]["aspects"] == aspects
            && entry["card"]["traits"] == traits
            && abilities == json!({"leader":[],"unit":[],"card":[registration(rule)]})
        {
            return Ok(Some(rule));
        }
    }
    Err("attachment self-play cost requires a complete reviewed ordinary upgrade and exact host predicate".into())
}

/// Called instead of target-free self-play normalization only after the closed
/// extraction succeeds. The native Definition field retains the actual rule.
pub(crate) fn normalize(entry: &Value, abilities: &mut Value) -> Result<(), String> {
    if extract(entry)?.is_some() {
        let list = abilities["card"]
            .as_array_mut()
            .ok_or("attachment cost requires card registrations")?;
        let at = list
            .iter()
            .position(|a| a["trigger"] == "costAdjustment")
            .ok_or("validated attachment cost registration missing")?;
        list.remove(at);
    }
    Ok(())
}
