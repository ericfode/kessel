//! Bounded view-only lookAt of an opponent's hand.
//!
//! Matches forceteki LookAtSystem + ViewCardSystem viewOnly with an explicit
//! display prompt: nonempty hands pause on Done; empty hands generate no event.
use crate::ability_ir::{Ability, Effect, Expr, Sides};
use serde_json::{json, Value};

fn opponent_hand(e: &Expr) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(["opponent", "hand"]))
}
fn self_player(e: &Expr) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(["player"]))
}

/// Exact reviewed leaf emitted for Viper Probe Droid / Imperial Defector.
pub(crate) fn reviewed_look_at(e: &Effect) -> bool {
    matches!(
        e,
        Effect::LookAt {
            target,
            player,
            interact_mode,
            use_display_prompt: Some(true),
        } if interact_mode == "viewOnly" && opponent_hand(target) && self_player(player)
    )
}

pub(crate) fn marker(v: &Value) -> bool {
    v["kind"] == "lookAt"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}

pub(crate) fn ability(a: &Ability) -> bool {
    a.trigger == "whenPlayed"
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && a.hooks.is_empty()
        && a.context_title.is_empty()
        && a.costs.is_empty()
        && a.target_resolver.is_none()
        && a.target_resolvers.is_empty()
        && a.condition.is_none()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && reviewed_look_at(&a.effect)
}

pub(crate) fn entry(e: &Value) -> bool {
    let Ok(sides) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    let look_at = sides
        .card
        .iter()
        .filter(|a| marker(&serde_json::to_value(a).unwrap_or(Value::Null)))
        .collect::<Vec<_>>();
    e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && ["cost", "hp", "power"]
            .iter()
            .all(|f| e["card"][*f].as_u64().is_some_and(|n| n <= 1000))
        && e["card"]["hp"].as_u64().is_some_and(|n| n > 0)
        && ["ground", "space"].iter().any(|a| e["card"]["arena"] == *a)
        && sides.card.len() == 1
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && look_at.len() == 1
        && ability(look_at[0])
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && e["keywordValues"] == json!({})
        && e["card"]["keywords"] == json!([])
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
        .all(|k| e["card"][*k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
        && crate::static_ongoing::closed_source_context(e)
}
