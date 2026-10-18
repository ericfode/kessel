//! Complete ordinary Credit compositions over the existing exact native ledger.
//!
//! This module validates source/IR only. It never pays a cost, creates an identity,
//! evaluates a condition, or admits an event-hook approximation. The original
//! condition and success continuation remain intact for ordinary compilation.
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Composition {
    OtherFriendlyUnderworldOnPlay,
    FriendlyGroundOnAttack,
    OptionalAttackPayment,
}

#[derive(Debug, Clone)]
pub(crate) struct Prepared {
    pub normalized: Value,
    pub composition: Composition,
}

pub(crate) fn marker(value: &Value) -> bool {
    value["kind"] == "createCreditToken"
        || match value {
            Value::Array(values) => values.iter().any(marker),
            Value::Object(fields) => fields.values().any(marker),
            _ => false,
        }
}

fn reference(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn binary(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn create() -> Value {
    json!({"kind":"createCreditToken","target":"ownPlayer",
        "amount":{"kind":"const","value":1},"destination":"base",
        "creationTiming":"eventUpdate"})
}
fn condition(composition: Composition) -> Value {
    let candidate = match composition {
        Composition::OtherFriendlyUnderworldOnPlay => binary(
            "&&",
            binary(
                "&&",
                json!({"kind":"const","value":true}),
                json!({"kind":"query","name":"hasSomeTrait","receiver":reference(&["card"]),
                    "args":[{"kind":"const","value":"underworld"}]}),
            ),
            binary(
                "||",
                json!({"kind":"unary","op":"!","value":reference(&["source"])}),
                binary("!==", reference(&["card"]), reference(&["source"])),
            ),
        ),
        Composition::FriendlyGroundOnAttack => {
            let ground = json!({"kind":"const","value":"groundArena"});
            binary(
                "&&",
                json!({"kind":"const","value":true}),
                binary(
                    "||",
                    json!({"kind":"unary","op":"!","value":ground.clone()}),
                    binary(
                        "||",
                        binary(
                            "===",
                            ground.clone(),
                            json!({"kind":"const","value":"anyArena"}),
                        ),
                        binary("===", reference(&["card", "zoneName"]), ground),
                    ),
                ),
            )
        }
        Composition::OptionalAttackPayment => unreachable!("payment is not a conditional"),
    };
    json!({"kind":"query","name":"hasSomeArenaUnit","receiver":reference(&["player"]),"args":[candidate]})
}
fn registration(composition: Composition) -> Value {
    match composition {
        Composition::OtherFriendlyUnderworldOnPlay | Composition::FriendlyGroundOnAttack => {
            json!({"trigger":if composition==Composition::OtherFriendlyUnderworldOnPlay {"whenPlayed"} else {"onAttack"},
                "costs":[],"effect":{"kind":"conditional","condition":condition(composition),
                    "onTrue":create(),"onFalse":{"kind":"noAction"}}})
        }
        Composition::OptionalAttackPayment => {
            json!({"trigger":"onAttack","costs":[],"optional":true,
            "effect":{"kind":"payAdjustedResources","target":"ownPlayer","amount":{"kind":"const","value":2},
                "paymentType":"cardEffectPayment","isCost":false,"canCancelDuringPayment":false,"allowCostAdjustments":true},
            "ifYouDo":{"title":"Create a Credit token","costs":[],"effect":create()}})
        }
    }
}
fn ordinary(entry: &Value) -> bool {
    entry["status"] == "supported"
        && entry["diagnostics"] == json!([])
        && crate::static_ongoing::closed_source_context(entry)
        && entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && entry["card"]["unique"].is_boolean()
        && entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        && entry["card"]["hp"]
            .as_u64()
            .is_some_and(|n| (1..=1000).contains(&n))
        && entry["card"]["power"].as_u64().is_some_and(|n| n <= 1000)
        && matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
        && entry["card"]["keywords"] == json!([])
        && entry["keywordValues"] == json!({})
        && entry["abilities"]["leader"] == json!([])
        && entry["abilities"]["unit"] == json!([])
        && entry["abilities"]
            .as_object()
            .is_some_and(|sides| sides.len() == 3)
        && entry["abilities"]["card"]
            .as_array()
            .is_some_and(|abilities| abilities.len() == 1)
        && entry["piloting"].is_null()
        && entry["attachCondition"].is_null()
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
        .all(|key| entry["card"][*key].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
}

/// Called only as an extension of `credits::creation_source`: the original
/// standalone creator contracts continue through that validator unchanged.
/// `Some` preserves the full source entry; no guard, optionality or continuation
/// is erased to make a registration compile. Hook, multi-owner, modal and live
/// Credit-count compositions remain explicit errors until their shared runtime
/// contracts are implemented and verified.
pub(crate) fn prepare(entry: &Value) -> Result<Option<Prepared>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let invalid = || {
        "Credit composition requires a complete reviewed ordinary conditional or optional-payment registration".to_owned()
    };
    if !ordinary(entry) {
        return Err(invalid());
    }
    let mut actual = entry["abilities"]["card"][0].clone();
    let title = actual
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("title")
        .ok_or_else(invalid)?;
    if !title
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 512)
    {
        return Err(invalid());
    }
    for (composition, text) in [
        (
            Composition::OtherFriendlyUnderworldOnPlay,
            "When Played: If you control another Underworld unit, create a Credit token.",
        ),
        (
            Composition::FriendlyGroundOnAttack,
            "On Attack: If you control a ground unit, create a Credit token.",
        ),
        (
            Composition::OptionalAttackPayment,
            "On Attack: You may pay 2 resources. If you do, create a Credit token.",
        ),
    ] {
        if actual == registration(composition) && entry["card"]["text"] == text {
            return Ok(Some(Prepared {
                normalized: entry.clone(),
                composition,
            }));
        }
    }
    Err(invalid())
}
