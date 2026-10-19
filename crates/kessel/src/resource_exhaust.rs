//! Reviewed plain resource exhaustion, deliberately separate from payment.
//!
//! Constant-one effects support direct player targets and one exact in-effect
//! player selector on ordinary units. Whole-source registration envelopes keep
//! other nested, modal, and payment variants closed.
use crate::ability_ir::{Effect, Expr, Target};
use serde_json::{json, Value};

pub(crate) fn amount(effect: &Effect) -> Result<u16, String> {
    let Effect::ExhaustResources {
        target,
        amount,
        payment_type,
        is_cost,
        allow_cost_adjustments,
    } = effect
    else {
        return Err("not plain resource exhaustion".into());
    };
    if payment_type != "exhaustResources" || *is_cost || *allow_cost_adjustments {
        return Err("exhaustResources requires the exact effect-only unadjusted contract".into());
    }
    if !matches!(target, Target::Name(n) if n == "ownPlayer" || n == "enemyPlayer") {
        return Err("exhaustResources requires a direct ownPlayer or enemyPlayer target".into());
    }
    if !matches!(amount.original(), Expr::Const { value } if value.as_u64() == Some(1)) {
        return Err("exhaustResources requires reviewed constant amount one".into());
    }
    Ok(1)
}

fn marker(v: &Value) -> bool {
    v["kind"] == "exhaustResources"
        || match v {
            Value::Array(a) => a.iter().any(marker),
            Value::Object(o) => o.values().any(marker),
            _ => false,
        }
}

fn leaf(target: &str) -> Value {
    json!({"kind":"exhaustResources","target":target,"amount":{"kind":"const","value":1},
        "paymentType":"exhaustResources","isCost":false,"allowCostAdjustments":false})
}

/// The retained wrapper is intentionally distinct from selectedPlayerSystem:
/// its target binding is installed by a new resolver after cost commitment.
fn selection_shape() -> Value {
    let mut inner = leaf("ownPlayer");
    inner["target"] = json!({"kind":"ref","path":["selectedPlayer"]});
    json!({"kind":"selectionSystem","system":"selectPlayer","name":"target",
        "resolver":{"kind":"playerSelection","mode":"player","chooser":"self",
        "resultBinding":"target","defaults":"pinnedTargetResolver","contextBinding":"target",
        "choosingPlayerOverride":"contextOverrideFirst","delayOpponentBeforeCosts":true,"resolverName":"target"},
        "innerSystem":inner,"contract":"SelectPlayerSystem.selectedPlayerDefaultTarget","legality":"innerSystemCanAffectPlayer"})
}

pub(crate) fn selection(effect: &Effect) -> Result<(), String> {
    if serde_json::to_value(effect).map_err(|e| e.to_string())? != selection_shape() {
        return Err("selectionSystem requires the exact nested single-player effect-only constant-one exhaustion contract".into());
    }
    Ok(())
}

pub(crate) fn validate_entry(entry: &Value) -> Result<(), String> {
    if !marker(&entry["abilities"]) {
        return Ok(());
    }
    if !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || crate::admission::OrdinarySource::from_entry(entry)
            != Some(crate::admission::OrdinarySource::Unit)
        || !matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
        || !entry["card"]["unique"].is_boolean()
        || !["cost", "hp", "power"]
            .iter()
            .all(|key| entry["card"][*key].as_u64().is_some_and(|n| n <= 1000))
        || !entry["card"]["hp"].as_u64().is_some_and(|n| n > 0)
        || !entry["piloting"].is_null()
        || !entry["attachCondition"].is_null()
        || ![
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|key| entry["card"][*key].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
    {
        return Err("exhaustResources requires closed ordinary nonleader unit metadata".into());
    }
    let mut sides = entry["abilities"].clone();
    // Titles are presentation data, but every other registration field binds the
    // audited source semantics, including absence of costs/conditions/helpers.
    let Some(registrations) = sides["card"].as_array_mut() else {
        return Err("exhaustResources requires complete ordinary registrations".into());
    };
    for registration in registrations {
        if !registration["title"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 1024)
        {
            return Err("exhaustResources requires bounded registration titles".into());
        }
        registration
            .as_object_mut()
            .ok_or("invalid resource-exhaustion registration")?
            .remove("title");
    }
    let death = json!({"leader":[],"unit":[],"card":[
        {"trigger":"whenDefeated","costs":[],"effect":leaf("ownPlayer")}
    ]});
    let attack = json!({"leader":[],"unit":[],"card":[
        {"trigger":"whenPlayed","costs":[],"optional":true,
         "targetResolver":{"owner":"opponent","type":"unit","zone":"anyAttackable"},
         "effect":{"kind":"exhaust","target":"chosen"}},
        {"trigger":"onAttack","costs":[],"optional":true,"effect":leaf("enemyPlayer")}
    ]});
    let selected = json!({"leader":[],"unit":[],"card":[
        {"trigger":"action","costs":[{"kind":"exhaustSelf"}],"effect":selection_shape()}
    ]});
    if sides != death && sides != attack && sides != selected {
        return Err("exhaustResources requires a complete reviewed constant-one direct death/attack or nested action registration envelope".into());
    }
    Ok(())
}
