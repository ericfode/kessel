//! Three complete ordinary regroup-start registrations, without a general phase
//! event evaluator. The event predicate is checked here and projected by the
//! shared trigger window; no lexical event context reaches an effect program.
use crate::ability_ir::Ability;
use serde_json::{json, Value};

#[derive(Clone, Copy)]
enum Shape {
    Damage,
    Defeat,
    Credit,
}
const SHAPES: [Shape; 3] = [Shape::Damage, Shape::Defeat, Shape::Credit];

fn effect(shape: Shape) -> Value {
    match shape {
        Shape::Damage => {
            json!({"kind":"damage","target":"self","amount":{"kind":"const","value":1}})
        }
        Shape::Defeat => json!({"kind":"defeat","target":"self"}),
        Shape::Credit => {
            json!({"kind":"createCreditToken","target":"ownPlayer","amount":{"kind":"const","value":1},"destination":"base","creationTiming":"eventUpdate"})
        }
    }
}
fn template(shape: Shape, title: &str, binding: &str) -> Value {
    json!({"trigger":"eventHooks","title":title,"costs":[],"effect":effect(shape),"hooks":[
        {"event":"onPhaseStarted","predicate":{"kind":"binary","op":"===",
         "left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":binding,
            "bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"phase","optional":false}]},
         "right":{"kind":"const","value":"regroup"}}}
    ]})
}
fn identity(a: &Value) -> Option<(&str, &str)> {
    let title = a["title"].as_str()?;
    let binding = a
        .pointer("/hooks/0/predicate/left/argumentBinding/id")?
        .as_str()?;
    (!title.is_empty() && title.len() <= 512 && !binding.is_empty() && binding.len() <= 128)
        .then_some((title, binding))
}
/// Typed compilation gate. Raw source closure is independently checked by
/// `entry`; typed comparisons account for omitted default fields.
pub(crate) fn registration(a: &Ability) -> bool {
    if a.attached_host || a.piloting_ability || a.pilot_unit_ability {
        return false;
    }
    let Ok(actual) = serde_json::to_value(a) else {
        return false;
    };
    let Some((title, binding)) = identity(&actual) else {
        return false;
    };
    SHAPES.iter().any(|shape| {
        serde_json::from_value::<Ability>(template(*shape, title, binding))
            .ok()
            .and_then(|expected| serde_json::to_value(expected).ok())
            .is_some_and(|expected| actual == expected)
    })
}
fn metadata(shape: Shape) -> Value {
    let ambush_rules = "Ambush is an ability that triggers at the same time as “When Played” abilities. If there is no enemy unit that can be attacked, the unit does not ready.\n";
    let (cost, hp, power, text, unique, arena, aspects, traits, keywords, rules) = match shape {
        Shape::Damage => (2,3,3,"Ambush (When you play this unit, it may attack an enemy unit.)\nWhen the regroup phase starts: Deal 1 damage to this unit.",true,"space",json!(["cunning","heroism"]),json!(["resistance","vehicle","fighter"]),json!(["ambush"]),json!(ambush_rules)),
        Shape::Defeat => (3,4,4,"Ambush (When you play this unit, it may attack an enemy unit.)\nWhen the regroup phase starts: Defeat this unit.",false,"ground",json!(["cunning"]),json!(["underworld","bounty hunter"]),json!(["ambush"]),json!(ambush_rules)),
        Shape::Credit => (3,5,1,"When the regroup phase starts: Create a Credit token.",true,"ground",json!(["command","cunning"]),json!(["underworld","musician"]),json!([]),Value::Null),
    };
    json!({"cost":cost,"hp":hp,"power":power,"text":text,"unique":unique,"arena":arena,
        "aspects":aspects,"traits":traits,"keywords":keywords,"rules":rules,
        "deployBox":null,"epicAction":"","upgradePower":null,"upgradeHp":null})
}
fn metadata_matches(card: &Value, shape: Shape) -> bool {
    let Some(mut actual) = card.as_object().cloned() else {
        return false;
    };
    for key in [
        "title",
        "subtitle",
        "id",
        "internalName",
        "setId",
        "setCodes",
    ] {
        actual.remove(key);
    }
    let mut actual = Value::Object(actual);
    let mut expected = metadata(shape);
    // These printed memberships have no order semantics. Sorting preserves
    // duplicate multiplicity instead of silently turning malformed data into sets.
    for key in ["aspects", "traits", "keywords"] {
        for value in [&mut actual, &mut expected] {
            let Some(values) = value[key].as_array_mut() else {
                return false;
            };
            if values.iter().any(|v| !v.is_string()) {
                return false;
            }
            values.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        }
    }
    actual == expected
}
/// Closed whole-card envelope. This does not grant native Credit provenance:
/// the common Credit loader must still require the reviewed selected token.
pub(crate) fn entry(e: &Value) -> bool {
    if e["status"] != "supported"
        || e["diagnostics"] != json!([])
        || !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || e["baseClass"] != "NonLeaderUnitCard"
        || e["card"]["types"] != json!(["unit"])
        || !e["piloting"].is_null()
        || !e["attachCondition"].is_null()
        || e["abilities"]["leader"] != json!([])
        || e["abilities"]["unit"] != json!([])
        || !e["abilities"].as_object().is_some_and(|o| o.len() == 3)
        || !e["abilities"]["card"]
            .as_array()
            .is_some_and(|a| a.len() == 1)
    {
        return false;
    }
    let a = &e["abilities"]["card"][0];
    let Some((title, binding)) = identity(a) else {
        return false;
    };
    SHAPES.iter().any(|shape| {
        let mut card = e["card"].clone();
        card.as_object_mut().unwrap().remove("types");
        a == &template(*shape, title, binding)
            && metadata_matches(&card, *shape)
            && e["keywordValues"]
                == match shape {
                    Shape::Credit => json!({}),
                    _ => json!({"ambush":1}),
                }
    })
}
