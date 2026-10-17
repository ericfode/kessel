//! Reviewed piloting units that grant a persistent keyword to their host vehicle.
//!
//! Compiler IR wraps the grant as `pilotingAttachedKeyword` /
//! `pilotingAttachedModifier`. Runtime lowers that to the ordinary
//! `gainKeyword`→`parent` constant path upgrades already use, and treats the
//! `piloting` keyword tag as a play-mode mention (not an intrinsic combat flag).
use crate::ability_ir::{Ability, Effect, Expr, Target};
use serde_json::{json, Value};

const GRANTED: &[&str] = &["grit", "sentinel", "restore", "raid", "overwhelm"];

pub(crate) fn marker(v: &Value) -> bool {
    matches!(
        v["kind"].as_str(),
        Some("pilotingAttachedKeyword" | "pilotingAttachedModifier")
    ) || match v {
        Value::Object(o) => o.values().any(marker),
        Value::Array(a) => a.iter().any(marker),
        _ => false,
    }
}

fn wrapper(a: &Value) -> Option<&Value> {
    if a["trigger"] != "constant" || a["costs"] != json!([]) {
        return None;
    }
    let e = &a["effect"];
    if !matches!(
        e["kind"].as_str(),
        Some("pilotingAttachedKeyword" | "pilotingAttachedModifier")
    ) {
        return None;
    }
    if e["target"] != "parent"
        || e["targetController"] != "any"
        || e["requiresGrantorInPlay"] != true
        || !e.get("gainCondition").is_none_or(|v| v.is_null())
        || !e.get("condition").is_none_or(|v| v.is_null())
    {
        return None;
    }
    let m = &e["modifier"];
    if m["kind"] != "gainKeyword"
        || m["target"] != "parent"
        || m["duration"] != "persistent"
        || m["amount"] != json!({"kind":"const","value":1})
        || !m["keyword"]
            .as_str()
            .is_some_and(|k| GRANTED.contains(&k))
    {
        return None;
    }
    Some(m)
}

fn granted_keyword(entry: &Value) -> Option<&str> {
    let card = entry["abilities"]["card"].as_array()?;
    if card.len() != 1 {
        return None;
    }
    wrapper(&card[0])?["keyword"].as_str()
}

/// Exact reviewed NonLeaderUnitCard / Pilot shape for sole host-keyword grants.
pub(crate) fn entry(entry: &Value) -> bool {
    if entry["baseClass"] != "NonLeaderUnitCard"
        || entry["card"]["types"] != json!(["unit"])
        || !entry["card"]["unique"].is_boolean()
        || !entry["attachCondition"].is_null()
        || !entry["piloting"].is_null()
        || entry["card"]["pilotText"]
            .as_str()
            .is_none_or(|t| t.trim().is_empty() || t.len() > 512)
        || ["pilotBox", "pilotingBox", "deployBox", "deployCost"]
            .iter()
            .any(|k| !entry["card"][*k].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        || entry["abilities"]["leader"] != json!([])
        || entry["abilities"]["unit"] != json!([])
        || !entry["card"]["arena"]
            .as_str()
            .is_some_and(|a| a == "ground" || a == "space")
    {
        return false;
    }
    if !entry["card"]["traits"]
        .as_array()
        .is_some_and(|t| t.iter().any(|v| v == "pilot"))
    {
        return false;
    }
    for (printed, derived) in [("power", "upgradePower"), ("hp", "upgradeHp")] {
        if entry["card"][printed] != entry["card"][derived]
            || !entry["card"][printed]
                .as_i64()
                .is_some_and(|n| (0..=1000).contains(&n))
        {
            return false;
        }
    }
    let Some(keyword) = granted_keyword(entry) else {
        return false;
    };
    let Some(names) = entry["card"]["keywords"].as_array() else {
        return false;
    };
    if names.len() != 2
        || !names.iter().any(|n| n == keyword)
        || !names.iter().any(|n| n == "piloting")
    {
        return false;
    }
    let Some(values) = entry["keywordValues"].as_object() else {
        return false;
    };
    if values.len() != 1 || values.get(keyword).and_then(Value::as_u64) != Some(1) {
        return false;
    }
    let Some(card) = entry["abilities"]["card"].as_array() else {
        return false;
    };
    card.len() == 1 && wrapper(&card[0]).is_some()
}

pub(crate) fn normalize(card: &mut Value) -> Result<(), String> {
    if !entry(card) {
        return Err(
            "piloting attached keyword requires complete ordinary Pilot unit metadata and registration"
                .into(),
        );
    }
    let title = card["abilities"]["card"][0]["title"].clone();
    let modifier = card["abilities"]["card"][0]["effect"]["modifier"].clone();
    card["abilities"]["card"][0] = json!({
        "trigger": "constant",
        "title": title,
        "costs": [],
        "effect": modifier,
    });
    Ok(())
}

pub(crate) fn registration(a: &Ability) -> bool {
    a.trigger == "constant"
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && a.attached_context.is_none()
        && a.gained_host.is_none()
        && !a.attached_host
        && a.costs.is_empty()
        && a.context_title.is_empty()
        && a.condition.is_none()
        && a.target_resolver.is_none()
        && a.target_resolvers.is_empty()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && a.hooks.is_empty()
        && matches!(
            &a.effect,
            Effect::GainKeyword {
                target: Target::Name(n),
                keyword,
                amount: Expr::Const { value },
                duration,
            } if n == "parent"
                && duration == "persistent"
                && GRANTED.contains(&keyword.as_str())
                && value.as_u64() == Some(1)
        )
}

/// Catalogue metadata / definition helpers for attachment legality.
pub(crate) fn metadata_is_pilot_attachment(info: &Value) -> bool {
    let keywords = info["keywords"].as_array();
    let types = info["types"].as_array();
    keywords.is_some_and(|k| k.iter().any(|v| v == "piloting"))
        && types.is_some_and(|t| t.len() == 1 && t[0] == "unit")
        && info["upgradePower"]
            .as_i64()
            .is_some_and(|n| (0..=1000).contains(&n))
        && info["upgradeHp"]
            .as_i64()
            .is_some_and(|n| (0..=1000).contains(&n))
        && info["pilotText"]
            .as_str()
            .is_some_and(|t| !t.trim().is_empty())
}
