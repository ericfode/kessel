//! Bounded opponent-hand lookAt / lookAtAndSelectCard effects.
//!
//! View-only `lookAt` (Viper Probe / Imperial Defector): nonempty hands pause on
//! Done; empty hands generate no event.
//!
//! `lookAtAndSelectCard` + `discardSpecificCard` (Spark of Rebellion, Bodhi Rook,
//! Jam Communications, Tip the Scale): look at opponent hand; when the reviewed
//! filter selects at least one card and `canChooseFewer` is false, the chooser
//! must discard one matching card; when the hand is nonempty but no card matches,
//! acknowledge with Done (look only); empty hands skip / make events illegal.
//!
//! Optional discard then opponent draw (Remnant Lookouts / Bazine Netal): same
//! look, but `canChooseFewer` is true and selecting runs
//! `discardSpecificCard` then `draw(enemyPlayer, 1)`; Done looks without discard.
use crate::ability_ir::{Ability, Effect, Expr, Sides, ViewSelectImmediate, ViewSelectStep};
use serde_json::{json, Value};

fn opponent_hand(e: &Expr) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(["opponent", "hand"]))
}
fn self_player(e: &Expr) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(["player"]))
}

fn const_u64(e: &Expr, n: u64) -> bool {
    matches!(e, Expr::Const { value } if value.as_u64() == Some(n))
}
fn const_bool(e: &Expr, b: bool) -> bool {
    matches!(e, Expr::Const { value } if value.as_bool() == Some(b))
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

/// Reviewed cardCondition shapes for opponent-hand discard selection.
pub(crate) fn reviewed_card_condition(v: &Value) -> Option<Expr> {
    if v == &json!({"kind": "const", "value": true}) {
        return Some(Expr::Const {
            value: Value::Bool(true),
        });
    }
    if v["kind"] == "systemValue" && v["evaluation"] == "viewSelectionFilter" {
        let inner = &v["value"];
        // !card.isUnit()
        if inner["kind"] == "unary"
            && inner["op"] == "!"
            && inner["value"]["kind"] == "query"
            && inner["value"]["name"] == "isUnit"
            && inner["value"]["receiver"] == json!({"kind": "ref", "path": ["card"]})
            && inner["value"]["args"] == json!([])
        {
            return serde_json::from_value(inner.clone()).ok();
        }
        // card.isEvent()
        if inner["kind"] == "query"
            && inner["name"] == "isEvent"
            && inner["receiver"] == json!({"kind": "ref", "path": ["card"]})
            && inner["args"] == json!([])
        {
            return serde_json::from_value(inner.clone()).ok();
        }
    }
    None
}

fn reviewed_discard_only(imm: &ViewSelectImmediate) -> bool {
    matches!(
        imm,
        ViewSelectImmediate::DiscardSpecificCard { target } if target == "viewSelected"
    )
}

fn reviewed_discard_then_opponent_draw(imm: &ViewSelectImmediate) -> bool {
    match imm {
        ViewSelectImmediate::Sequence { effects } if effects.len() == 2 => {
            matches!(
                &effects[0],
                ViewSelectStep::DiscardSpecificCard { target } if target == "viewSelected"
            ) && matches!(
                &effects[1],
                ViewSelectStep::Draw {
                    target,
                    amount,
                } if target == "enemyPlayer" && const_u64(amount, 1)
            )
        }
        _ => false,
    }
}

/// Exact reviewed lookAtAndSelectCard shapes:
/// - mandatory discard (`canChooseFewer: false`) with discardSpecificCard
/// - optional discard + opponent draw (`canChooseFewer: true`) with the reviewed sequence
pub(crate) fn reviewed_look_at_select(e: &Effect) -> bool {
    match e {
        Effect::LookAtAndSelectCard {
            target,
            player,
            use_display_prompt: true,
            max_cards,
            can_choose_fewer,
            card_condition,
            selected_context_field,
            immediate_effect,
        } => {
            if !(opponent_hand(target)
                && self_player(player)
                && const_u64(max_cards, 1)
                && selected_context_field == "selectedPromptCards"
                && reviewed_card_condition(card_condition).is_some())
            {
                return false;
            }
            if const_bool(can_choose_fewer, false) && reviewed_discard_only(immediate_effect) {
                return true;
            }
            if const_bool(can_choose_fewer, true)
                && reviewed_discard_then_opponent_draw(immediate_effect)
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Whether the current lookAtAndSelectCard allows Done without discarding.
pub(crate) fn can_choose_fewer(e: &Effect) -> bool {
    matches!(
        e,
        Effect::LookAtAndSelectCard {
            can_choose_fewer,
            ..
        } if const_bool(can_choose_fewer, true)
    )
}

/// Whether selecting a card also draws one for the opponent.
pub(crate) fn draws_for_opponent(e: &Effect) -> bool {
    matches!(
        e,
        Effect::LookAtAndSelectCard {
            immediate_effect,
            ..
        } if reviewed_discard_then_opponent_draw(immediate_effect)
    )
}

/// Predicate for live filtering. `card_condition` stays a raw Value (not Compiled).
pub(crate) fn selection_predicate(e: &Effect) -> Option<Expr> {
    match e {
        Effect::LookAtAndSelectCard { card_condition, .. } => {
            reviewed_card_condition(card_condition)
        }
        _ => None,
    }
}

pub(crate) fn marker(v: &Value) -> bool {
    matches!(
        v["kind"].as_str(),
        Some("lookAt" | "lookAtAndSelectCard")
    ) || match v {
        Value::Object(o) => o.values().any(marker),
        Value::Array(a) => a.iter().any(marker),
        _ => false,
    }
}

fn plain_ability_header(a: &Ability) -> bool {
    !a.optional
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
}

pub(crate) fn look_at_ability(a: &Ability) -> bool {
    a.trigger == "whenPlayed" && plain_ability_header(a) && reviewed_look_at(&a.effect)
}

pub(crate) fn look_at_select_ability(a: &Ability) -> bool {
    matches!(a.trigger.as_str(), "whenPlayed" | "event")
        && plain_ability_header(a)
        && reviewed_look_at_select(&a.effect)
}

fn printed_keywords_ok(e: &Value) -> bool {
    let keywords = &e["card"]["keywords"];
    keywords == &json!([])
        || keywords == &json!(["smuggle"])
}

fn closed_unit_or_event(e: &Value, sides: &Sides, trigger: &str) -> bool {
    let (base, types) = match trigger {
        "whenPlayed" => ("NonLeaderUnitCard", json!(["unit"])),
        "event" => ("EventCard", json!(["event"])),
        _ => return false,
    };
    let stats_ok = if trigger == "event" {
        e["card"]["cost"]
            .as_u64()
            .is_some_and(|n| n <= 1000)
            && e["card"]["hp"].is_null()
            && e["card"]["power"].is_null()
            && (e["card"]["arena"].is_null() || e["card"].get("arena").is_none())
    } else {
        ["cost", "hp", "power"]
            .iter()
            .all(|f| e["card"][*f].as_u64().is_some_and(|n| n <= 1000))
            && e["card"]["hp"].as_u64().is_some_and(|n| n > 0)
            && ["ground", "space"].iter().any(|a| e["card"]["arena"] == *a)
    };
    e["baseClass"] == base
        && e["card"]["types"] == types
        && e["card"]["unique"].is_boolean()
        && stats_ok
        && sides.card.len() == 1
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && e["keywordValues"] == json!({})
        && printed_keywords_ok(e)
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

pub(crate) fn entry(e: &Value) -> bool {
    let Ok(sides) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    let marked = sides
        .card
        .iter()
        .filter(|a| marker(&serde_json::to_value(a).unwrap_or(Value::Null)))
        .collect::<Vec<_>>();
    if marked.len() != 1 {
        return false;
    }
    let ability = marked[0];
    if look_at_ability(ability) {
        return closed_unit_or_event(e, &sides, "whenPlayed");
    }
    if look_at_select_ability(ability) {
        return closed_unit_or_event(e, &sides, ability.trigger.as_str());
    }
    false
}
