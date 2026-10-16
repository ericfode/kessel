//! Closed projections of defeat-event registrations. The predicate is evaluated
//! against the event's pre-handler unit/type/controller snapshot; effect programs
//! cannot read or retain an unmodeled event context.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Hook {
    Alias(Alias),
    Event(Event),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    pub alias: AliasName,
    pub enabled: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AliasName {
    #[serde(rename = "whenPlayed")]
    WhenPlayed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub event: EventName,
    pub predicate: Predicate,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventName {
    #[serde(rename = "onCardDefeated")]
    Defeated,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Predicate {
    Binary {
        op: String,
        left: Box<Predicate>,
        right: Box<Predicate>,
    },
    EnumTypeTest {
        test: String,
        args: Vec<Predicate>,
    },
    DataPath {
        root: Vec<String>,
        #[serde(rename = "argumentBinding")]
        argument_binding: Binding,
        steps: Vec<Step>,
    },
    Ref {
        path: Vec<String>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    id: String,
    bind_at: BindAt,
    read_at: ReadAt,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindAt {
    #[serde(rename = "callbackInvocation")]
    Invocation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadAt {
    #[serde(rename = "expressionEvaluation")]
    Expression,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    key: String,
    optional: bool,
}
#[derive(Default)]
struct Projection {
    unit: bool,
    non_token: bool,
    controller: Option<bool>,
    other: bool,
}
impl Predicate {
    fn path(&self, wanted: &[&str], binding: &mut Option<String>) -> bool {
        let Self::DataPath {
            root,
            argument_binding: b,
            steps,
        } = self
        else {
            return false;
        };
        if root != &["event"]
            || b.id.is_empty()
            || b.id.len() > 128
            || steps.len() != wanted.len()
            || !steps
                .iter()
                .zip(wanted)
                .all(|(s, k)| !s.optional && s.key == *k)
        {
            return false;
        }
        if binding.as_ref().is_some_and(|id| id != &b.id) {
            return false;
        }
        *binding = Some(b.id.clone());
        true
    }
    fn reference(&self, wanted: &[&str]) -> bool {
        matches!(self,Self::Ref{path} if path.iter().map(String::as_str).eq(wanted.iter().copied()))
    }
    fn project(&self, out: &mut Projection, binding: &mut Option<String>, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        match self {
            Self::Binary { op, left, right } if op == "&&" => {
                left.project(out, binding, depth + 1) && right.project(out, binding, depth + 1)
            }
            Self::EnumTypeTest { test, args }
                if ["isUnit", "isNonTokenUnit"].contains(&test.as_str())
                    && args.len() == 1
                    && args[0].path(&["lastKnownInformation", "type"], binding)
                    && !out.unit =>
            {
                out.unit = true;
                out.non_token = test == "isNonTokenUnit";
                true
            }
            Self::Binary { op, left, right }
                if ["===", "!=="].contains(&op.as_str())
                    && left.path(&["lastKnownInformation", "controller"], binding)
                    && right.reference(&["player"])
                    && out.controller.is_none() =>
            {
                out.controller = Some(op == "===");
                true
            }
            Self::Binary { op, left, right }
                if op == "!=="
                    && left.path(&["card"], binding)
                    && right.reference(&["source"])
                    && !out.other =>
            {
                out.other = true;
                true
            }
            _ => false,
        }
    }
    fn projection(&self) -> Option<Projection> {
        let mut p = Projection::default();
        self.project(&mut p, &mut None, 0)
            .then_some(p)
            .filter(|p| p.unit)
    }
    pub(crate) fn matches(&self, token: bool, same_controller: bool, same_card: bool) -> bool {
        self.projection().is_some_and(|p| {
            (!p.non_token || !token)
                && p.controller.is_none_or(|same| same == same_controller)
                && (!p.other || !same_card)
        })
    }
}
pub(crate) fn registration(a: &Ability) -> bool {
    let shape = crate::admission::RegistrationShape {
        targeting: crate::admission::Targeting::OptionalSingle,
        host_context: crate::admission::HostContext::None,
    };
    if a.trigger != "eventHooks"
        || a.hooks.is_empty()
        || a.hooks.len() > 2
        || !shape.accepts(a)
        || a.attacker_must_survive
    {
        return false;
    }
    let mut deaths = 0;
    let mut plays = 0;
    for hook in &a.hooks {
        match hook {
            Hook::Alias(v) if v.enabled => plays += 1,
            Hook::Event(v) if v.predicate.projection().is_some() => deaths += 1,
            _ => return false,
        }
    }
    if deaths != 1 || plays > 1 {
        return false;
    }
    let one = |e: &Expr| matches!(e,Expr::Const{value} if value.as_u64()==Some(1));
    match &a.effect {
        Effect::GiveExperience {
            target: Target::Name(t),
            amount,
        } if t == "chosen" && one(amount) => a.target_resolver.as_ref().is_some_and(|r| {
            serde_json::to_value(r).ok().is_some_and(|v| {
                let expected: crate::ability_ir::TargetResolver = serde_json::from_value(
                    serde_json::json!({"owner":"self","type":"any","zone":"anyAttackable"}),
                )
                .unwrap();
                v == serde_json::to_value(expected).unwrap()
            })
        }),
        Effect::Heal {
            target: Target::Name(t),
            amount,
        } => t == "ownBase" && one(amount) && a.target_resolver.is_none(),
        Effect::GainKeyword {
            target: Target::Name(t),
            keyword,
            amount,
            duration,
        } => {
            t == "self"
                && keyword == "sentinel"
                && duration == "phase"
                && one(amount)
                && a.target_resolver.is_none()
        }
        Effect::CreateTokenUnit {
            token_type,
            amount,
            target: Target::Name(t),
            enters_ready,
        } => {
            token_type == "battleDroid"
                && one(amount)
                && t == "ownPlayer"
                && !enters_ready
                && a.target_resolver.is_none()
        }
        _ => false,
    }
}
pub(crate) fn marker(v: &Value) -> bool {
    v.get("hooks").is_some()
        || v["trigger"] == "eventHooks"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
pub(crate) fn entry(e: &Value, sides: &Sides) -> bool {
    crate::admission::SourceContextPolicy::LegacyEmptyWatchers.accepts(e)
        && e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == serde_json::json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && !sides.card.is_empty()
        && sides.card.len() <= 8
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
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
        .all(|key| e["card"][key].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
        && sides
            .card
            .iter()
            .filter(|a| marker(&serde_json::to_value(a).expect("serializable ability")))
            .all(registration)
}
