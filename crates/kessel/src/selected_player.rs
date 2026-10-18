//! Bounded single-player selection feeding draw / ready-resources / hand-discard.
//!
//! Compiler IR uses `playerSelection` resolvers and `selectedPlayerSystem` effects.
//! Runtime stores the chosen player as that player's base id in `ability_target`
//! and executes the existing player-targeted effect primitives.
use crate::ability_ir::{Effect, Expr, Target, TargetResolver};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePlayerSelection {
    kind: String,
    mode: String,
    chooser: String,
    result_binding: String,
    defaults: String,
    context_binding: String,
    choosing_player_override: String,
    delay_opponent_before_costs: bool,
    #[serde(default)]
    prompt_title_parts: Vec<Expr>,
}

impl NativePlayerSelection {
    pub fn lift(self) -> Result<TargetResolver, String> {
        if self.kind != "playerSelection"
            || self.mode != "player"
            || self.chooser != "self"
            || self.result_binding != "selectedPlayer"
            || self.defaults != "pinnedTargetResolver"
            || self.context_binding != "target"
            || self.choosing_player_override != "contextOverrideFirst"
            || !self.delay_opponent_before_costs
        {
            return Err(
                "player selection requires pinned self-chooser single-player protocol".into(),
            );
        }
        let mut bytes = 0;
        for fragment in &self.prompt_title_parts {
            let Expr::Const { value } = fragment else {
                return Err("player selection prompt requires bounded static text".into());
            };
            bytes += value
                .as_str()
                .ok_or("player selection prompt requires string fragments")?
                .len();
        }
        if bytes > 1024 || self.prompt_title_parts.len() > 32 {
            return Err("player selection prompt exceeds capacity".into());
        }
        Ok(TargetResolver {
            prompt_title: None,
            prompt_title_parts: self.prompt_title_parts,
            owner: "any".into(),
            card_type: "player".into(),
            arena: None,
            zone: None,
            optional: false,
            predicate: None,
            mode: Some("player".into()),
            count: None,
            native: None,
            player_selection: true,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectedPlayerSystemIr {
    #[serde(default)]
    kind: Option<String>,
    system: String,
    target: Value,
    #[serde(default)]
    properties: SelectedPlayerProperties,
    defaults: String,
    #[serde(rename = "defaultTargetOverride")]
    default_target_override: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectedPlayerProperties {
    #[serde(default)]
    pub amount: Option<Expr>,
}

fn selected_player_ref(value: &Value) -> bool {
    value.get("kind").and_then(Value::as_str) == Some("ref")
        && value.get("path").and_then(Value::as_array).is_some_and(|path| {
            path.len() == 1 && path[0].as_str() == Some("selectedPlayer")
        })
}

/// Lower a reviewed `selectedPlayerSystem` object into an executable player effect.
pub fn lower_effect(value: &Value) -> Result<Effect, String> {
    let ir: SelectedPlayerSystemIr =
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if ir.kind.as_deref().is_some_and(|k| k != "selectedPlayerSystem") {
        return Err("selectedPlayerSystem kind mismatch".into());
    }
    lower_fields(
        &ir.system,
        &ir.target,
        &ir.properties,
        &ir.defaults,
        &ir.default_target_override,
    )
}

pub fn lower_fields(
    system: &str,
    target: &Value,
    properties: &SelectedPlayerProperties,
    defaults: &str,
    default_target_override: &Value,
) -> Result<Effect, String> {
    if defaults != "pinnedSystem"
        || !selected_player_ref(target)
        || !selected_player_ref(default_target_override)
    {
        return Err("selectedPlayerSystem requires pinned selectedPlayer target defaults".into());
    }
    let amount = properties
        .amount
        .clone()
        .unwrap_or_else(|| Expr::number(1));
    if !matches!(
        amount.original(),
        Expr::Const { value } if value.as_u64().is_some_and(|n| (1..=2).contains(&n))
    ) {
        return Err("selectedPlayerSystem amount must be literal 1..2".into());
    }
    let target = Target::Name("chosenPlayer".into());
    match system {
        "readyResources" => Ok(Effect::ReadyResources { amount, target }),
        "draw" => Ok(Effect::Draw { amount, target }),
        "discardCardsFromOwnHand" => Ok(Effect::DiscardOwnHand { amount, target }),
        other => Err(format!(
            "selectedPlayerSystem {other} is outside the reviewed ready/draw/discard family"
        )),
    }
}

pub fn marker(value: &Value) -> bool {
    (value.get("kind").and_then(Value::as_str) == Some("selectedPlayerSystem"))
        || (value.get("kind").and_then(Value::as_str) == Some("playerSelection"))
        || match value {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
