//! Reviewed piloting units that grant persistent host effects while attached.
//!
//! Compiler IR wraps grants as `pilotingAttachedKeyword` /
//! `pilotingAttachedModifier`. Runtime lowers those to ordinary parent-targeted
//! constants (keyword / stats) that upgrades already evaluate, and treats the
//! `piloting` keyword tag as a play-mode mention (not an intrinsic combat flag).
//! Optional `gainCondition` / `condition` is limited to a live host-trait query.
use crate::ability_ir::{Ability, Effect, Expr, Target};
use serde_json::{json, Value};

const GRANTED: &[&str] = &["grit", "sentinel", "restore", "raid", "overwhelm"];
const TRAITS: &[&str] = &[
    "fighter", "speeder", "transport", "vehicle", "capital ship", "force", "jedi",
];

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

fn parent_trait_condition(v: &Value) -> bool {
    let Some(trait_name) = v["args"].as_array().and_then(|a| {
        (a.len() == 1)
            .then(|| a[0]["value"].as_str())
            .flatten()
            .filter(|t| TRAITS.contains(t))
    }) else {
        return false;
    };
    v == &json!({
        "kind": "query",
        "name": "hasSomeTrait",
        "receiver": {"kind": "ref", "path": ["source", "parentCard"]},
        "args": [{"kind": "const", "value": trait_name}]
    })
}

fn wrapper_gate(e: &Value) -> Option<Option<Value>> {
    if e["target"] != "parent"
        || e["targetController"] != "any"
        || e["requiresGrantorInPlay"] != true
    {
        return None;
    }
    let gain = e.get("gainCondition").filter(|v| !v.is_null());
    let cond = e.get("condition").filter(|v| !v.is_null());
    match (gain, cond) {
        (None, None) => Some(None),
        (Some(c), None) | (None, Some(c)) if parent_trait_condition(c) => Some(Some(c.clone())),
        _ => None,
    }
}

fn keyword_modifier(m: &Value) -> bool {
    m["kind"] == "gainKeyword"
        && m["target"] == "parent"
        && m["duration"] == "persistent"
        && m["amount"] == json!({"kind": "const", "value": 1})
        && m["keyword"]
            .as_str()
            .is_some_and(|k| GRANTED.contains(&k))
}

fn stats_modifier(m: &Value) -> bool {
    m["kind"] == "modifyStats"
        && m["target"] == "parent"
        && m["duration"] == "persistent"
        && m["power"]
            .as_object()
            .is_some_and(|o| o.get("kind") == Some(&json!("const")))
        && m["hp"]
            .as_object()
            .is_some_and(|o| o.get("kind") == Some(&json!("const")))
        && m["power"]["value"]
            .as_i64()
            .is_some_and(|n| (-1000..=1000).contains(&n))
        && m["hp"]["value"]
            .as_i64()
            .is_some_and(|n| (-1000..=1000).contains(&n))
        && !(m["power"]["value"] == 0 && m["hp"]["value"] == 0)
}

fn wrapper(a: &Value) -> Option<(Value, Option<Value>)> {
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
    let condition = wrapper_gate(e)?;
    let m = &e["modifier"];
    if !(keyword_modifier(m) || stats_modifier(m)) {
        return None;
    }
    // Keyword wrappers must use the keyword IR kind; stat wrappers use modifier.
    match e["kind"].as_str() {
        Some("pilotingAttachedKeyword") if !keyword_modifier(m) => return None,
        Some("pilotingAttachedModifier") if !(keyword_modifier(m) || stats_modifier(m)) => {
            return None
        }
        _ => {}
    }
    Some((m.clone(), condition))
}

fn pilot_metadata(entry: &Value) -> bool {
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
    for field in ["power", "hp", "upgradePower", "upgradeHp"] {
        if !entry["card"][field]
            .as_i64()
            .is_some_and(|n| (0..=1000).contains(&n))
        {
            return false;
        }
    }
    true
}

fn keyword_metadata(entry: &Value, grants: &[(Value, Option<Value>)]) -> bool {
    let Some(names) = entry["card"]["keywords"].as_array() else {
        return false;
    };
    if !names.iter().any(|n| n == "piloting") || names.len() > 8 {
        return false;
    }
    let granted: Vec<&str> = grants
        .iter()
        .filter_map(|(m, _)| {
            (m["kind"] == "gainKeyword")
                .then(|| m["keyword"].as_str())
                .flatten()
        })
        .collect();
    for name in names {
        let Some(n) = name.as_str() else {
            return false;
        };
        if n == "piloting" {
            continue;
        }
        if !granted.contains(&n) {
            return false;
        }
    }
    let Some(values) = entry["keywordValues"].as_object() else {
        return false;
    };
    if values.len() > granted.len() {
        return false;
    }
    for (key, value) in values {
        // Intrinsic keywordValues only for unconditional keyword grants.
        let ok = grants.iter().any(|(m, condition)| {
            m["kind"] == "gainKeyword"
                && m["keyword"] == *key
                && condition.is_none()
                && value.as_u64() == Some(1)
        });
        if !ok {
            return false;
        }
    }
    true
}

/// Exact reviewed NonLeaderUnitCard / Pilot shape for persistent parent grants.
pub(crate) fn entry(entry: &Value) -> bool {
    if !pilot_metadata(entry) {
        return false;
    }
    let Some(card) = entry["abilities"]["card"].as_array() else {
        return false;
    };
    if card.is_empty() || card.len() > 8 {
        return false;
    }
    let mut grants = Vec::with_capacity(card.len());
    for ability in card {
        let Some(parsed) = wrapper(ability) else {
            return false;
        };
        grants.push(parsed);
    }
    keyword_metadata(entry, &grants)
}

pub(crate) fn normalize(card: &mut Value) -> Result<(), String> {
    if !entry(card) {
        return Err(
            "piloting attached keyword requires complete ordinary Pilot unit metadata and registration"
                .into(),
        );
    }
    let raw = card["abilities"]["card"]
        .as_array()
        .cloned()
        .ok_or("piloting attached abilities missing")?;
    let mut out = Vec::with_capacity(raw.len());
    for ability in raw {
        let title = ability["title"].clone();
        let (modifier, condition) = wrapper(&ability).ok_or("piloting attached wrapper")?;
        let mut normalized = json!({
            "trigger": "constant",
            "title": title,
            "costs": [],
            "effect": modifier,
        });
        if let Some(condition) = condition {
            normalized["condition"] = condition;
        }
        out.push(normalized);
    }
    card["abilities"]["card"] = Value::Array(out);
    Ok(())
}

fn parent_trait_expr(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Query {
            name,
            receiver,
            args
        } if name == "hasSomeTrait"
            && matches!(receiver.as_ref(), Expr::Ref { path } if path.iter().map(String::as_str).eq(["source", "parentCard"]))
            && args.len() == 1
            && matches!(&args[0], Expr::Const { value } if value.as_str().is_some_and(|t| TRAITS.contains(&t)))
    )
}

pub(crate) fn registration(a: &Ability) -> bool {
    if a.trigger != "constant"
        || a.optional
        || a.requires_confirmation
        || a.coordinate
        || a.attached_context.is_some()
        || a.gained_host.is_some()
        || a.attached_host
        || !a.costs.is_empty()
        || !a.context_title.is_empty()
        || a.target_resolver.is_some()
        || !a.target_resolvers.is_empty()
        || a.then.is_some()
        || a.if_you_do.is_some()
        || !a.hooks.is_empty()
    {
        return false;
    }
    if a.condition.as_ref().is_some_and(|c| !parent_trait_expr(c)) {
        return false;
    }
    match &a.effect {
        Effect::GainKeyword {
            target: Target::Name(n),
            keyword,
            amount: Expr::Const { value },
            duration,
        } => {
            n == "parent"
                && duration == "persistent"
                && GRANTED.contains(&keyword.as_str())
                && value.as_u64() == Some(1)
        }
        Effect::ModifyStats {
            target: Target::Name(n),
            power: Expr::Const { value: power },
            hp: Expr::Const { value: hp },
            duration,
        } => {
            n == "parent"
                && duration == "persistent"
                && power.as_i64().is_some_and(|v| (-1000..=1000).contains(&v))
                && hp.as_i64().is_some_and(|v| (-1000..=1000).contains(&v))
                && !(power.as_i64() == Some(0) && hp.as_i64() == Some(0))
        }
        _ => false,
    }
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
