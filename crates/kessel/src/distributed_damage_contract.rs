//! Source-only envelope for the reviewed native damage distribution helper.
//! No simulator state or card identities participate in admission.
use serde_json::{json, Value};
pub(super) fn effect(
    amount: &super::Expr,
    empty: bool,
    less: bool,
    r: &super::TargetResolver,
) -> bool {
    matches!(amount.original(),super::Expr::Const{value} if value==&json!(6))
        && !empty
        && !less
        && r.owner == "opponent"
        && r.card_type == "unit"
        && r.zone.as_deref() == Some("anyAttackable")
        && r.arena.is_none()
        && r.predicate.is_none()
        && !r.optional
        && r.native.is_none()
        && !r.player_selection
        && r.mode.is_none()
        && r.count.is_none()
        && r.prompt_title.is_none()
        && r.prompt_title_parts.is_empty()
}
pub(super) fn marker(e: &Value) -> bool {
    ["abilities", "attachCondition", "piloting"]
        .iter()
        .any(|k| super::contains(&e[*k], |v| v["kind"] == "distributeDamage"))
}
pub(super) fn entry(e: &Value) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || crate::admission::OrdinarySource::from_entry(e)
            != Some(crate::admission::OrdinarySource::Unit)
    {
        return false;
    }
    let card = &e["card"];
    if !card["unique"].is_boolean()
        || !card["cost"].is_u64()
        || !card["hp"].is_u64()
        || !card["power"].is_u64()
        || !matches!(card["arena"].as_str(), Some("ground" | "space"))
        || !card["traits"].is_array()
        || !card["aspects"].is_array()
    {
        return false;
    }
    let Some(registrations) = e["abilities"]["card"].as_array().filter(|a| a.len() == 1) else {
        return false;
    };
    let a = &registrations[0];
    let keywords = match a["trigger"].as_str() {
        Some("whenPlayed") => {
            e["keywordValues"] == json!({"overwhelm":1}) && card["keywords"] == json!(["overwhelm"])
        }
        Some("whenDefeated") => e["keywordValues"] == json!({}) && card["keywords"] == json!([]),
        _ => false,
    };
    if !keywords
        || !e["abilities"]["unit"].as_array().is_some_and(Vec::is_empty)
        || !e["abilities"]["leader"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return false;
    }
    let Some(mut registration) = a.as_object().cloned() else {
        return false;
    };
    if !registration
        .remove("title")
        .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 1024))
    {
        return false;
    }
    let trigger = registration.remove("trigger");
    if !matches!(
        trigger.as_ref().and_then(Value::as_str),
        Some("whenPlayed" | "whenDefeated")
    ) {
        return false;
    }
    Value::Object(registration)
        == json!({"costs":[],"effect":{"kind":"distributeDamage","amount":{"kind":"const","value":6},"canChooseNoTargets":false,"canDistributeLess":false,"targetResolver":{"owner":"opponent","type":"unit","zone":"anyAttackable"}}})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn original(code: &str) -> Value {
        let c: Value =
            serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json")))
                .unwrap();
        c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["setCode"] == code)
            .unwrap()
            .clone()
    }
    fn admitted(v: Value) -> bool {
        let code = v["setCode"].as_str().unwrap().to_owned();
        {
            let c =
                super::super::Catalogue::load(&json!({"schema":super::super::SCHEMA,"cards":[v]}))
                    .unwrap();
            if !c.cards.contains_key(&code) {
                eprintln!("rejection {code}: {:?}", c.rejected);
            }
            c.cards.contains_key(&code)
        }
    }
    #[test]
    fn distributed_damage_complete_source_and_raw_ir_closed() {
        for code in ["SOR_135", "LOF_213"] {
            let original = original(code);
            assert!(entry(&original));
            assert!(admitted(original.clone()), "{code}");
            for (pointer, value) in [
                ("/baseClass", json!("EventCard")),
                ("/card/types", json!(["unit", "leader"])),
                ("/card/unique", Value::Null),
                ("/card/deployBox", json!("new")),
                ("/abilities/card/0/optional", json!(false)),
                ("/abilities/card/0/effect/maxTargets", json!(6)),
                (
                    "/abilities/card/0/effect/amount",
                    json!({"kind":"const","value":5}),
                ),
                ("/abilities/card/0/effect/canChooseNoTargets", json!(true)),
                ("/abilities/card/0/effect/canDistributeLess", json!(true)),
                (
                    "/abilities/card/0/effect/targetResolver/owner",
                    json!("any"),
                ),
                (
                    "/abilities/card/0/effect/targetResolver/type",
                    json!("base"),
                ),
                ("/abilities/card/0/effect/source", json!("self")),
                ("/stateWatchers", json!([{}])),
            ] {
                let mut v = original.clone();
                let keys = pointer.split('/').skip(1).collect::<Vec<_>>();
                let mut node = &mut v;
                for key in &keys[..keys.len() - 1] {
                    node = if node.is_array() {
                        &mut node[key.parse::<usize>().unwrap()]
                    } else {
                        &mut node[*key]
                    };
                }
                node[keys[keys.len() - 1]] = value;
                assert!(!admitted(v), "{code} {pointer}");
            }
        }
    }
}
