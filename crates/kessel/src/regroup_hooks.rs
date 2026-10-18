//! Complete ordinary regroup-start registrations, without a general phase
//! event evaluator. The event predicate is checked here and projected by the
//! shared trigger window; no lexical event context reaches an effect program.
use crate::ability_ir::Ability;
use serde_json::{json, Value};

#[derive(Clone, Copy)]
enum Shape {
    Damage,
    Defeat,
    Credit,
    Advantage,
    Experience,
    ClonesDamage,
    DebuffHeal,
}
const SHAPES: [Shape; 7] = [
    Shape::Damage,
    Shape::Defeat,
    Shape::Credit,
    Shape::Advantage,
    Shape::Experience,
    Shape::ClonesDamage,
    Shape::DebuffHeal,
];

fn other_non_vehicle_units() -> Value {
    json!({"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{
        "kind":"binary","op":"&&","left":{
            "kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{
                "kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"ref","path":["source"]}},
                "right":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}
            }
        },"right":{"kind":"unary","op":"!","value":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"vehicle"}]}}
    }})
}

fn enemy_ground_units() -> Value {
    json!({"kind":"arenaList","receiver":{"kind":"ref","path":["opponent"]},"predicate":{
        "kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{
            "kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"groundArena"}},"right":{
                "kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"groundArena"},"right":{"kind":"const","value":"anyArena"}},
                "right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"groundArena"}}
            }
        }
    }})
}

fn effect(shape: Shape) -> Value {
    match shape {
        Shape::Damage => {
            json!({"kind":"damage","target":"self","amount":{"kind":"const","value":1}})
        }
        Shape::Defeat => json!({"kind":"defeat","target":"self"}),
        Shape::Credit => {
            json!({"kind":"createCreditToken","target":"ownPlayer","amount":{"kind":"const","value":1},"destination":"base","creationTiming":"eventUpdate"})
        }
        Shape::Advantage => {
            json!({"kind":"giveAdvantage","target":"chosen","amount":{"kind":"const","value":1},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})
        }
        Shape::Experience => {
            json!({"kind":"giveExperience","target":other_non_vehicle_units(),"amount":{"kind":"const","value":1}})
        }
        Shape::ClonesDamage => {
            json!({"kind":"damage","target":"self","amount":{"kind":"const","value":4}})
        }
        Shape::DebuffHeal => {
            json!({"kind":"heal","target":"self","amount":{"kind":"const","value":5}})
        }
    }
}
fn template(shape: Shape, title: &str, binding: &str) -> Value {
    let mut registration = json!({"trigger":"eventHooks","title":title,"costs":[],"effect":effect(shape),"hooks":[
        {"event":"onPhaseStarted","predicate":{"kind":"binary","op":"===",
         "left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":binding,
            "bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"phase","optional":false}]},
         "right":{"kind":"const","value":"regroup"}}}
    ]});
    if matches!(shape, Shape::Advantage) {
        registration["targetResolver"] =
            json!({"owner":"any","type":"unit","zone":"anyAttackable"});
    }
    registration
}
/// These companions are part of the full source contract, not independent
/// permission to mix arbitrary When Played abilities with a phase hook.
fn companion(shape: Shape, title: &str) -> Option<Value> {
    let effect = match shape {
        Shape::ClonesDamage => {
            json!({"kind":"createTokenUnit","tokenType":"cloneTrooper","amount":{"kind":"const","value":2},"target":"ownPlayer","entersReady":false})
        }
        Shape::DebuffHeal => {
            json!({"kind":"modifyStats","target":enemy_ground_units(),"duration":"phase","power":{"kind":"unary","op":"-","value":{"kind":"const","value":5}},"hp":{"kind":"const","value":0}})
        }
        _ => return None,
    };
    Some(json!({"trigger":"whenPlayed","title":title,"costs":[],"effect":effect}))
}
fn title(a: &Value) -> Option<&str> {
    a["title"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 512)
}
fn identity(a: &Value) -> Option<(&str, &str)> {
    let title = title(a)?;
    let binding = a
        .pointer("/hooks/0/predicate/left/argumentBinding/id")?
        .as_str()?;
    (!binding.is_empty() && binding.len() <= 128).then_some((title, binding))
}
/// Typed compilation gate. Raw source closure is independently checked by
/// `entry`; typed comparisons account for omitted default fields.
fn typed_registration(a: &Ability, shape: Shape, normalized: bool) -> bool {
    if a.attached_host || a.piloting_ability || a.pilot_unit_ability {
        return false;
    }
    let Ok(actual) = serde_json::to_value(a) else {
        return false;
    };
    let identity = if normalized {
        title(&actual).map(|title| (title, "normalized"))
    } else {
        identity(&actual)
    };
    let Some((title, binding)) = identity else {
        return false;
    };
    let mut expected = template(shape, title, binding);
    if normalized {
        expected["trigger"] = json!("whenPlayed");
        expected.as_object_mut().unwrap().remove("hooks");
    }
    serde_json::from_value::<Ability>(expected)
        .ok()
        .and_then(|expected| serde_json::to_value(expected).ok())
        .is_some_and(|expected| actual == expected)
}
pub(crate) fn registration(a: &Ability) -> bool {
    SHAPES
        .iter()
        .any(|shape| typed_registration(a, *shape, false))
}
/// The shared hook compiler temporarily projects a verified phase hook to an
/// ordinary registration. Only this exact live list may cross the arena-token
/// compilation fence; `entry` still requires the original complete phase source.
pub(crate) fn arena_token_registration(a: &Ability) -> bool {
    typed_registration(a, Shape::Experience, false)
        || typed_registration(a, Shape::Experience, true)
}
fn metadata(shape: Shape) -> Value {
    let ambush_rules = "Ambush is an ability that triggers at the same time as “When Played” abilities. If there is no enemy unit that can be attacked, the unit does not ready.\n";
    let overwhelm_rules = "Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n";
    let (cost, hp, power, text, unique, arena, aspects, traits, keywords, rules) = match shape {
        Shape::Damage => (2,3,3,"Ambush (When you play this unit, it may attack an enemy unit.)\nWhen the regroup phase starts: Deal 1 damage to this unit.",true,"space",json!(["cunning","heroism"]),json!(["resistance","vehicle","fighter"]),json!(["ambush"]),json!(ambush_rules)),
        Shape::Defeat => (3,4,4,"Ambush (When you play this unit, it may attack an enemy unit.)\nWhen the regroup phase starts: Defeat this unit.",false,"ground",json!(["cunning"]),json!(["underworld","bounty hunter"]),json!(["ambush"]),json!(ambush_rules)),
        Shape::Credit => (3,5,1,"When the regroup phase starts: Create a Credit token.",true,"ground",json!(["command","cunning"]),json!(["underworld","musician"]),json!([]),Value::Null),
        Shape::Advantage => (5,6,5,"Overwhelm (When attacking an enemy unit, deal excess damage to the opponent's base.)\nWhen the regroup phase starts: Give an Advantage token to a unit.",true,"space",json!(["aggression","heroism"]),json!(["rebel","vehicle","transport"]),json!(["overwhelm"]),json!(overwhelm_rules)),
        Shape::Experience => (4,7,2,"When the regroup phase starts: Give an Experience token to each other friendly non-Vehicle unit.",true,"ground",json!(["vigilance","vigilance"]),json!(["force","creature"]),json!([]),Value::Null),
        Shape::ClonesDamage => (4,5,3,"When Played: Create 2 Clone Trooper tokens.\nWhen the regroup phase starts: Deal 4 damage to this unit.",false,"ground",json!(["command","aggression"]),json!(["republic","vehicle","transport"]),json!([]),Value::Null),
        Shape::DebuffHeal => (9,10,10,"When Played: Give each enemy ground unit –5/–0 for this phase.\nWhen the regroup phase starts: Heal 5 damage from this unit.",true,"ground",json!(["vigilance"]),json!(["creature"]),json!([]),Value::Null),
    };
    json!({"cost":cost,"hp":hp,"power":power,"text":text,"unique":unique,"arena":arena,
        "aspects":aspects,"traits":traits,"keywords":keywords,"rules":rules,
        "deployBox":null,"epicAction":if matches!(shape,Shape::DebuffHeal) {Value::Null}else{json!("")},"upgradePower":null,"upgradeHp":null})
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
/// Closed whole-card envelope. Token effects still require their exact reviewed
/// native definitions through the shared catalogue and selected-pool loaders.
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
    {
        return false;
    }
    let Some(abilities) = e["abilities"]["card"].as_array() else {
        return false;
    };
    let Some(a) = abilities.last() else {
        return false;
    };
    let Some((title, binding)) = identity(a) else {
        return false;
    };
    let Some(mut card) = e["card"].as_object().cloned() else {
        return false;
    };
    card.remove("types");
    let card = Value::Object(card);
    SHAPES.iter().any(|shape| {
        let mut expected = Vec::new();
        if matches!(shape, Shape::ClonesDamage | Shape::DebuffHeal) {
            let Some(companion_title) = abilities.first().and_then(self::title) else {
                return false;
            };
            expected.push(companion(*shape, companion_title).unwrap());
        }
        expected.push(template(*shape, title, binding));
        abilities == &expected
            && metadata_matches(&card, *shape)
            && e["keywordValues"]
                == match shape {
                    Shape::Damage | Shape::Defeat => json!({"ambush":1}),
                    Shape::Advantage => json!({"overwhelm":1}),
                    _ => json!({}),
                }
    })
}
