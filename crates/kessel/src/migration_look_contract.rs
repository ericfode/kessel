//! Closed ordinary event compositions containing private hand observations.
//!
//! This module only validates source/IR shapes. It does not evaluate callbacks,
//! allocate selection storage, lower a captured value to a live query, or grant
//! runtime admission. In particular, resource play, named-card restrictions and
//! selected-card play permissions remain outside this contract.
use crate::ability_ir::Ability;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    DiscardThenLook,
    DiscloseThenLook,
    ReciprocalLookThenDraw,
}

#[derive(Debug, Clone)]
pub(crate) struct Prepared {
    pub kind: Kind,
    /// Retain every observation, chooser, optionality and continuation field.
    /// The integration owner must compile the complete registration.
    pub registration: Value,
}

fn keys(v: &Value, allowed: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|o| o.keys().all(|key| allowed.contains(&key.as_str())))
}

fn empty_or_absent(v: &Value, field: &str) -> bool {
    v.get(field).is_none() || v[field].as_array().is_some_and(Vec::is_empty)
}

fn header(v: &Value, trigger: &str, continuation: bool) -> bool {
    keys(
        v,
        &[
            "trigger",
            "title",
            "costs",
            "effect",
            "ifYouDo",
            "then",
            "coordinate",
            "optional",
            "requiresConfirmation",
            "attackerMustSurvive",
            "contextTitle",
            "targetResolver",
            "targetResolvers",
            "condition",
            "hooks",
            "gainedHost",
            "attachedContext",
            "ifYouDoCondition",
        ],
    ) && v["title"].is_string()
        && (v["trigger"] == trigger || (trigger.is_empty() && v.get("trigger").is_none()))
        && [
            "coordinate",
            "optional",
            "requiresConfirmation",
            "attackerMustSurvive",
        ]
        .iter()
        .all(|f| v.get(*f).is_none() || v[*f] == false)
        && ["costs", "contextTitle", "targetResolvers", "hooks"]
            .iter()
            .all(|f| empty_or_absent(v, f))
        && [
            "targetResolver",
            "condition",
            "gainedHost",
            "attachedContext",
            "ifYouDoCondition",
            "then",
        ]
        .iter()
        .all(|f| v[*f].is_null())
        && (continuation || v["ifYouDo"].is_null())
}

/// A hand inventory and its chooser must be opposite players, each relative to
/// the retained source controller rather than the actor of an earlier prompt.
pub(crate) fn selection(v: &Value) -> bool {
    if !keys(
        v,
        &[
            "kind",
            "target",
            "player",
            "useDisplayPrompt",
            "maxCards",
            "canChooseFewer",
            "cardCondition",
            "selectedContextField",
            "immediateEffect",
            "activePromptTitleParts",
        ],
    ) || v["kind"] != "lookAtAndSelectCard"
        || v["useDisplayPrompt"] != true
        || v["maxCards"] != json!({"kind":"const","value":1})
        || ![true, false]
            .iter()
            .any(|b| v["canChooseFewer"] == json!({"kind":"const","value":b}))
        || v["selectedContextField"] != "selectedPromptCards"
        || v["immediateEffect"] != json!({"kind":"discardSpecificCard","target":"viewSelected"})
    {
        return false;
    }
    let opponent_hand = v["target"] == json!({"kind":"ref","path":["opponent","hand"]})
        && v["player"] == json!({"kind":"ref","path":["player"]});
    let own_hand = v["target"] == json!({"kind":"ref","path":["player","hand"]})
        && v["player"] == json!({"kind":"ref","path":["opponent"]});
    (opponent_hand || own_hand)
        && selection_predicate(&v["cardCondition"]).is_some()
        && v.get("activePromptTitleParts").is_none_or(|parts| {
            parts.as_array().is_some_and(|parts| {
                parts.len() <= 1
                    && parts.iter().all(|p| {
                        keys(p, &["kind", "value"])
                            && p["kind"] == "const"
                            && p["value"]
                                .as_str()
                                .is_some_and(|s| !s.is_empty() && s.len() <= 512)
                    })
            })
        })
}

/// Exact raw wrapper closure matters: `cardCondition` is an untyped JSON field
/// in the portable IR and must not silently discard additional callback fields.
pub(crate) fn selection_predicate(v: &Value) -> Option<crate::ability_ir::Expr> {
    let any = json!({"kind":"const","value":true});
    let unit = json!({"kind":"systemValue","evaluation":"viewSelectionFilter","value":{
        "kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]
    }});
    if v == &any {
        serde_json::from_value(any).ok()
    } else if v == &unit {
        serde_json::from_value(v["value"].clone()).ok()
    } else {
        None
    }
}

fn hand_selection(v: &Value, opponent: bool, optional: bool, unit: bool) -> bool {
    selection(v)
        && v["target"]
            == json!({"kind":"ref","path":if opponent {vec!["opponent","hand"]} else {vec!["player","hand"]}})
        && v["canChooseFewer"] == json!({"kind":"const","value":optional})
        && (v["cardCondition"] == json!({"kind":"const","value":true})) == !unit
}

fn disclose(v: &Value) -> bool {
    keys(
        v,
        &[
            "kind",
            "target",
            "aspects",
            "mode",
            "aspectMultiplicity",
            "selectedCardsEventField",
            "triggeredNoMatchPolicy",
            "selectionContext",
        ],
    ) && v["kind"] == "disclose"
        && v["target"] == "ownPlayer"
        && v["mode"] == "all"
        && v["aspectMultiplicity"] == true
        && v["selectedCardsEventField"] == "disclosedCards"
        && v["triggeredNoMatchPolicy"] == "maskWithPauseWithoutEvent"
        && v["selectionContext"] == "targetPlayer"
        && v["aspects"].as_array().is_some_and(|aspects| {
            (1..=3).contains(&aspects.len())
                && aspects.iter().all(|a| {
                    a.as_str().is_some_and(|a| {
                        [
                            "vigilance",
                            "command",
                            "aggression",
                            "cunning",
                            "heroism",
                            "villainy",
                        ]
                        .contains(&a)
                    })
                })
        })
}

pub(crate) fn classify(v: &Value) -> Option<Kind> {
    if !header(v, "event", true) {
        return None;
    }
    let next = &v["ifYouDo"];
    if header(next, "", false) {
        if v["effect"]
            == json!({"kind":"discardOwnHand","target":"ownPlayer","amount":{"kind":"const","value":1}})
            && hand_selection(&next["effect"], true, true, false)
        {
            return Some(Kind::DiscardThenLook);
        }
        if disclose(&v["effect"]) && hand_selection(&next["effect"], true, false, true) {
            return Some(Kind::DiscloseThenLook);
        }
    }
    let effect = &v["effect"];
    if next.is_null()
        && keys(effect, &["kind", "effects"])
        && effect["kind"] == "sequence"
        && effect["effects"].as_array().is_some_and(|effects| {
            effects.len() == 3
                && hand_selection(&effects[0], true, false, false)
                && hand_selection(&effects[1], false, false, false)
                && effects[2] == json!({"kind":"draw","target":"bothPlayers","amount":{"kind":"const","value":1}})
        })
    {
        return Some(Kind::ReciprocalLookThenDraw);
    }
    None
}

pub(crate) fn registration(a: &Ability) -> bool {
    // These trusted loader flags are deliberately absent from serialized IR.
    fn ordinary(a: &Ability) -> bool {
        !a.attached_host
            && !a.piloting_ability
            && !a.pilot_unit_ability
            && a.then.as_deref().is_none_or(ordinary)
            && a.if_you_do.as_deref().is_none_or(ordinary)
    }
    ordinary(a)
        && serde_json::to_value(a)
            .ok()
            .is_some_and(|v| classify(&v).is_some())
}

pub(crate) fn prepare(entry: &Value) -> Result<Prepared, String> {
    let sides = &entry["abilities"];
    if entry["status"] != "supported"
        || entry["baseClass"] != "EventCard"
        || entry["card"]["types"] != json!(["event"])
        || !entry["card"]["unique"].is_boolean()
        || !entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        || ![
            "hp",
            "power",
            "arena",
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|f| entry["card"][*f].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        || entry["card"]["keywords"] != json!([])
        || entry["keywordValues"] != json!({})
        || !entry["piloting"].is_null()
        || !entry["attachCondition"].is_null()
        || !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || !keys(sides, &["leader", "unit", "card"])
        || sides["leader"] != json!([])
        || sides["unit"] != json!([])
        || sides["card"].as_array().is_none_or(|a| a.len() != 1)
    {
        return Err("look continuation requires a complete ordinary event source".into());
    }
    let registration = sides["card"][0].clone();
    let kind = classify(&registration)
        .ok_or("look continuation requires a bounded hand observation composition; captured/play/lasting continuations remain unsupported")?;
    Ok(Prepared { kind, registration })
}

pub(crate) fn entry(entry: &Value) -> bool {
    prepare(entry).is_ok()
}
