//! Native, live self-play reductions. These registrations are extracted into a
//! Definition cost rule, never executed as an ability or replaced with a no-op.
use crate::model::{CardId, Seat, State};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelfPlayCost {
    FriendlyDamagedUnits,
    AllDamagedGroundUnits,
}

pub(crate) fn marker(v: &Value) -> bool {
    v["kind"] == "decreaseSelfPlayCost"
        || v["trigger"] == "costAdjustment"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}

fn r(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn c(v: Value) -> Value {
    json!({"kind":"const","value":v})
}
fn b(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
fn q(name: &str, receiver: Value, args: Vec<Value>) -> Value {
    json!({"kind":"query","name":name,"receiver":receiver,"args":args})
}
fn count(owner: &[&str], predicate: Value) -> Value {
    q("arenaUnitCount", r(owner), vec![predicate])
}
fn damaged(test: &str) -> Value {
    b(
        "&&",
        q(test, r(&["card"]), vec![]),
        b(">", r(&["card", "damage"]), c(json!(0))),
    )
}
fn ground() -> Value {
    let arena = c(json!("groundArena"));
    b(
        "||",
        json!({"kind":"unary","op":"!","value":arena}),
        b(
            "||",
            b("===", arena.clone(), c(json!("anyArena"))),
            b("===", r(&["card", "zoneName"]), arena),
        ),
    )
}
fn registration(rule: SelfPlayCost) -> Value {
    let amount = match rule {
        SelfPlayCost::FriendlyDamagedUnits => count(
            &["adjusterPlayer"],
            b("&&", c(json!(true)), damaged("isUnit")),
        ),
        SelfPlayCost::AllDamagedGroundUnits => {
            let predicate = b(
                "&&",
                b("&&", c(json!(true)), ground()),
                damaged("canBeDamaged"),
            );
            b(
                "+",
                count(&["adjusterPlayer"], predicate.clone()),
                count(&["adjusterPlayer", "opponent"], predicate),
            )
        }
    };
    json!({"trigger":"costAdjustment","costs":[],"effect":{
        "kind":"decreaseSelfPlayCost","sourceZone":"any","targetPlayers":"any",
        "matchSourceIdentity":true,"matchPrintedType":true,"stage":"standard_0",
        "amount":{"kind":"boundCallback","parameters":["adjustedCard","adjusterPlayer"],"timing":"costAdjustment","value":amount},
        "matchAbilityCosts":false,"matchCardEffectResourcePayments":false
    }})
}

/// Complete ordinary-unit contract. No card names, IDs, or source hashes select
/// a rule. In particular an additional unsupported registration rejects the card.
pub(crate) fn extract(entry: &Value) -> Result<Option<SelfPlayCost>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let plain = crate::static_ongoing::closed_source_context(entry)
        && entry["status"] == "supported"
        && entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && entry["card"]["unique"].is_boolean()
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
        ]
        .iter()
        .all(|k| entry["card"][*k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["abilities"]["leader"] == json!([])
        && entry["abilities"]["unit"] == json!([]);
    let mut abilities = entry["abilities"].clone();
    if let Some(list) = abilities["card"].as_array_mut() {
        for a in list {
            if let Some(o) = a.as_object_mut() {
                o.remove("title");
            }
        }
    }
    for rule in [
        SelfPlayCost::FriendlyDamagedUnits,
        SelfPlayCost::AllDamagedGroundUnits,
    ] {
        let (text, keywords, values) = match rule {
            SelfPlayCost::FriendlyDamagedUnits => ("This unit costs 1 resource less to play for each friendly damaged unit.", json!([]), json!({})),
            SelfPlayCost::AllDamagedGroundUnits => ("This unit costs 1 resource less to play for each damaged ground unit.\nOverwhelm (When attacking an enemy unit, deal excess damage to the opponent's base.)", json!(["overwhelm"]), json!({"overwhelm":1})),
        };
        if plain
            && abilities == json!({"leader":[],"unit":[],"card":[registration(rule)]})
            && entry["card"]["text"] == text
            && entry["card"]["keywords"] == keywords
            && entry["keywordValues"] == values
        {
            return Ok(Some(rule));
        }
    }
    Err(
        "self-play cost requires a complete ordinary unit and reviewed live damaged-unit count"
            .into(),
    )
}

/// Called before typed ability deserialization. The removed registration is
/// retained by extract(entry) on the Definition and consulted for every play.
pub(crate) fn normalize(entry: &Value, abilities: &mut Value) -> Result<(), String> {
    if extract(entry)?.is_some() {
        abilities["card"] = json!([]);
    }
    Ok(())
}

impl SelfPlayCost {
    pub(crate) fn reduction(self, state: &State, source: CardId) -> u16 {
        let owner = state.card(source).owner;
        let seats: &[Seat] = match self {
            Self::FriendlyDamagedUnits => std::slice::from_ref(&owner),
            Self::AllDamagedGroundUnits => &[Seat::P1, Seat::P2],
        };
        seats
            .iter()
            .map(|seat| {
                let player = state.player(*seat);
                player
                    .ground
                    .as_slice()
                    .iter()
                    .chain(
                        if self == Self::FriendlyDamagedUnits {
                            player.space.as_slice()
                        } else {
                            &[]
                        }
                        .iter(),
                    )
                    .filter(|&&id| state.card(id).present && state.card(id).damage > 0)
                    .count() as u16
            })
            .sum()
    }
}
