//! Closed current-incarnation play projection for an ordinary ready/Spy event.
//!
//! A physical played-ID bit records play (including an opponent's play); the
//! Hidden qualification bit records whether this ordinary unit's current
//! incarnation was played this phase. Deployment and token creation set the
//! latter bit alone and are expressly outside the playable-card domain.
use crate::ability_ir::{Ability, Expr, Instruction, Program};
use serde_json::{json, Value};

pub(crate) const QUERY: &str = "isCurrentPlayedUnit";

fn reference(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn binary(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn query_value() -> Value {
    json!({"kind":"query","name":QUERY,"receiver":reference(&["card"]),"args":[]})
}
fn body(predicate: Value) -> Value {
    json!({"trigger":"event","costs":[],"targetResolver":{
    "owner":"any","type":"unit","zone":"anyAttackable","predicate":predicate},
    "effect":{"kind":"simultaneous","effects":[
        {"kind":"ready","target":"chosen"},
        {"kind":"conditional","condition":binary("&&",
            json!({"kind":"query","name":"isUnit","receiver":reference(&["target"]),"args":[]}),
            binary("<=",reference(&["target","cost"]),json!({"kind":"const","value":3}))),
         "onTrue":{"kind":"createTokenUnit","tokenType":"spy","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},
         "onFalse":{"kind":"noAction"}}
    ]}})
}
fn raw_predicate(watcher_binding: &str, lambda_binding: &str) -> Value {
    let entry = |key: &str| json!({"kind":"dataPath","root":[lambda_binding],"steps":[{"key":key,"optional":false}]});
    json!({"kind":"historyQuery","watcher":"cardsPlayedThisPhase","binding":watcher_binding,
        "operation":"someCardPlayed","args":[{"kind":"dataLambda","bindings":[lambda_binding],
            "body":binary("&&",binary("&&",
                binary("===",entry("card"),reference(&["card"])),
                json!({"kind":"dataCall","method":"canBeInPlay","receiver":reference(&["card"]),"args":[],"optionalCall":false,"optionalReceiver":false})),
                binary("===",entry("inPlayId"),reference(&["card","inPlayId"])))}],
        "readAt":"expressionEvaluation"})
}
fn watcher(binding: &str) -> Value {
    json!({"kind":"phaseHistory","binding":binding,"watcher":"cardsPlayedThisPhase",
        "sharedScope":"game","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded",
        "implementation":"CardsPlayedThisPhaseWatcher","queryContract":"pinnedWatcherApi"})
}
fn binding(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| !s.is_empty() && s.len() <= 128)
}
fn has_incarnation_reference(v: &Value) -> bool {
    (v["kind"] == "ref" && v["path"][1] == "inPlayId")
        || match v {
            Value::Object(o) => o.values().any(has_incarnation_reference),
            Value::Array(a) => a.iter().any(has_incarnation_reference),
            _ => false,
        }
}
/// Detect both original query contracts and attempts to supply the lowered query.
/// General history query admission is unchanged.
pub(crate) fn marker(v: &Value) -> bool {
    (v["kind"] == "query" && v["name"] == QUERY)
        || (v["kind"] == "historyQuery"
            && v["watcher"] == "cardsPlayedThisPhase"
            && v["operation"] == "someCardPlayed"
            && has_incarnation_reference(v))
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}

/// Return only normalized executable abilities. The original entry, source
/// provenance, watcher declaration, and catalogue metadata remain unchanged.
pub(crate) fn prepare(entry: &Value) -> Result<Option<Value>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let invalid = || {
        "current played-unit query requires the complete ordinary ready/conditional-Spy event and declared watcher".to_owned()
    };
    let mut closed = entry.clone();
    closed
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("stateWatchers");
    let plain = crate::static_ongoing::closed_source_context(&closed)
        && entry["status"] == "supported"
        && entry["baseClass"] == "EventCard"
        && entry["card"]["types"] == json!(["event"])
        && entry["card"]["unique"] == false
        && entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        && entry["card"]["keywords"] == json!([])
        && entry["keywordValues"] == json!({})
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
            "hp",
            "power",
            "arena",
        ]
        .iter()
        .all(|k| entry["card"][*k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["abilities"]["leader"] == json!([])
        && entry["abilities"]["unit"] == json!([])
        && entry["abilities"]["card"]
            .as_array()
            .is_some_and(|a| a.len() == 1)
        && entry["abilities"].as_object().is_some_and(|o| o.len() == 3);
    if !plain {
        return Err(invalid());
    }
    let original = &entry["abilities"]["card"][0];
    let predicate = &original["targetResolver"]["predicate"];
    let watcher_binding = binding(&predicate["binding"]).ok_or_else(invalid)?;
    let lambda = &predicate["args"][0];
    let lambda_binding = binding(&lambda["bindings"][0]).ok_or_else(invalid)?;
    if lambda["bindings"].as_array().is_none_or(|a| a.len() != 1)
        || predicate != &raw_predicate(watcher_binding, lambda_binding)
        || entry["stateWatchers"] != json!([watcher(watcher_binding)])
    {
        return Err(invalid());
    }
    let mut shape = original.clone();
    let title = shape
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("title")
        .ok_or_else(invalid)?;
    if !title
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 512)
        || shape != body(predicate.clone())
    {
        return Err(invalid());
    }
    let mut normalized = body(query_value());
    normalized["title"] = title;
    Ok(Some(json!({"leader":[],"unit":[],"card":[normalized]})))
}

pub(crate) fn query(e: &Expr) -> bool {
    matches!(e.original(),Expr::Query{name,receiver,args} if name==QUERY&&args.is_empty()&&matches!(receiver.original(),Expr::Ref{path} if path==&["card"]))
}
/// Raw Program compilation cannot reuse this query in a foreign registration,
/// callback, scalar amount, continuation, or attached/piloting context.
pub(crate) fn ability(a: &Ability) -> bool {
    if a.attached_host || a.piloting_ability || a.pilot_unit_ability {
        return false;
    }
    let mut expected = body(query_value());
    expected["title"] = json!("diagnostic");
    let Ok(expected) = serde_json::from_value::<Ability>(expected) else {
        return false;
    };
    let Ok(mut actual) = serde_json::to_value(a) else {
        return false;
    };
    let Ok(mut expected) = serde_json::to_value(expected) else {
        return false;
    };
    actual.as_object_mut().unwrap().remove("title");
    expected.as_object_mut().unwrap().remove("title");
    actual == expected
}
pub(crate) fn requires(program: &Program) -> bool {
    program
        .instructions
        .iter()
        .any(|i| matches!(i,Instruction::Select(r) if r.predicate.as_ref().is_some_and(query)))
}
