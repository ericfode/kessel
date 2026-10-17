//! Reviewed deck-discard → conditional return-to-hand shapes (Kuiil / Boshek / Krennic).
//!
//! Compiler IR keeps `eventResult` / `dataCall` filter targets that portable `Target`
//! cannot deserialize. Normalize those leaves before Ability parse, capture discarded
//! card ids in `ability_selection` after `discardDeck`, and gate continuations with
//! reviewed ifYouDoCondition matchers.
use serde_json::{json, Value};

/// `events[0].card` from a discardDeck group (forceteki OnCardDiscarded then aggregate).
pub(crate) fn discarded_card_event_result(v: &Value) -> bool {
    v["kind"] == "eventResult"
        && v["collection"] == "events"
        && v["eventIndex"] == 0
        && v["path"] == json!(["card"])
        && (v["optionalAccess"].is_null() || v["optionalAccess"] == json!([]))
        && v["contextBinding"]["bindAt"] == "callbackInvocation"
        && v["contextBinding"]["readAt"] == "expressionEvaluation"
        && v["contextBinding"]["reference"] == "lexicalContextObject"
        && v["contextBinding"]["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("callback:") && id.len() <= 64)
}

/// Kuiil: discarded card shares an aspect with the acting player's base.
pub(crate) fn shares_base_aspect_condition(v: &Value) -> bool {
    v["kind"] == "resultSharesBaseAspect"
        && discarded_card_event_result(&v["card"])
        && v["base"]["kind"] == "contextRef"
        && v["base"]["path"] == json!(["player", "base"])
        && v["base"]["contextBinding"]["bindAt"] == "callbackInvocation"
        && v["base"]["contextBinding"]["readAt"] == "expressionEvaluation"
        && v["base"]["contextBinding"]["reference"] == "lexicalContextObject"
        && v["base"]["contextBinding"]["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("callback:") && id.len() <= 64)
}

/// Krennic: discarded card `isUnit()`.
pub(crate) fn discarded_is_unit_condition(v: &Value) -> bool {
    v["kind"] == "query"
        && v["name"] == "isUnit"
        && v["args"] == json!([])
        && discarded_card_event_result(&v["receiver"])
}

/// Boshek: map(filter(events, onCardDiscarded && cost%2===1), card).
fn odd_cost_discarded_target(v: &Value) -> bool {
    if v["kind"] != "dataCall"
        || v["method"] != "map"
        || v["optionalCall"] != false
        || v["optionalReceiver"] != false
    {
        return false;
    }
    let Some(map_args) = v["args"].as_array() else {
        return false;
    };
    if map_args.len() != 1 {
        return false;
    }
    let map_lambda = &map_args[0];
    if map_lambda["kind"] != "dataLambda" {
        return false;
    }
    let Some(map_bindings) = map_lambda["bindings"].as_array() else {
        return false;
    };
    if map_bindings.len() != 1 {
        return false;
    }
    let map_slot = map_bindings[0].as_str().unwrap_or("");
    if map_lambda["body"]
        != json!({"kind":"dataPath","root":[map_slot],"steps":[{"key":"card","optional":false}]})
    {
        return false;
    }
    let filter = &v["receiver"];
    if filter["kind"] != "dataCall"
        || filter["method"] != "filter"
        || filter["optionalCall"] != false
        || filter["optionalReceiver"] != false
    {
        return false;
    }
    let events = &filter["receiver"];
    if events["kind"] != "dataPath"
        || events["root"] != json!(["context"])
        || events["steps"] != json!([{"key":"events","optional":false}])
        || events["contextBinding"]["role"] != "ifYouDo"
        || events["contextBinding"]["bindAt"] != "callbackInvocation"
        || events["contextBinding"]["readAt"] != "expressionEvaluation"
        || events["contextBinding"]["reference"] != "lexicalContextObject"
        || !events["contextBinding"]["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("callback:") && id.len() <= 64)
    {
        return false;
    }
    let Some(filter_args) = filter["args"].as_array() else {
        return false;
    };
    if filter_args.len() != 1 {
        return false;
    }
    let filter_lambda = &filter_args[0];
    if filter_lambda["kind"] != "dataLambda" {
        return false;
    }
    let Some(filter_bindings) = filter_lambda["bindings"].as_array() else {
        return false;
    };
    if filter_bindings.len() != 1 {
        return false;
    }
    let slot = filter_bindings[0].as_str().unwrap_or("");
    filter_lambda["body"]
        == json!({
            "kind":"binary","op":"&&",
            "left":{
                "kind":"binary","op":"===",
                "left":{"kind":"dataPath","root":[slot],"steps":[{"key":"name","optional":false}]},
                "right":{"kind":"const","value":"onCardDiscarded"}
            },
            "right":{
                "kind":"binary","op":"===",
                "left":{
                    "kind":"binary","op":"%",
                    "left":{"kind":"dataPath","root":[slot],"steps":[
                        {"key":"card","optional":false},
                        {"key":"cost","optional":false}
                    ]},
                    "right":{"kind":"const","value":2}
                },
                "right":{"kind":"const","value":1}
            }
        })
}

/// Rewrite reviewed discard-return targets into portable Name targets before Ability parse.
pub(crate) fn normalize(v: &mut Value) {
    if v["kind"] == "returnToHand" {
        if discarded_card_event_result(&v["target"]) {
            v["target"] = json!("discardedCard");
        } else if odd_cost_discarded_target(&v["target"]) {
            v["target"] = json!("oddCostDiscarded");
        }
    }
    match v {
        Value::Object(o) => {
            for child in o.values_mut() {
                normalize(child);
            }
        }
        Value::Array(a) => {
            for child in a {
                normalize(child);
            }
        }
        _ => {}
    }
}
