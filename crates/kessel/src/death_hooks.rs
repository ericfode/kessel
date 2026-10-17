//! Closed projections of defeat-event and smuggle-play registrations. Defeat
//! predicates are evaluated against the event's pre-handler unit/type/controller
//! snapshot; effect programs cannot read or retain an unmodeled event context.
//! Unsupported hook JSON deserializes as [`Hook::Unsupported`] and stays rejected.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Hook {
    Alias(Alias),
    Event(Event),
    /// Any hook shape that is not a reviewed Alias/Event projection.
    Unsupported(Value),
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
    #[serde(rename = "whenPlayedUsingSmuggle")]
    WhenPlayedUsingSmuggle,
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
    #[serde(rename = "onLeaderDeployed")]
    LeaderDeployed,
    #[serde(rename = "onCardPlayed")]
    CardPlayed,
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
    Const {
        value: Value,
    },
    DataCall {
        method: String,
        receiver: Box<Predicate>,
        args: Vec<Predicate>,
        #[serde(rename = "optionalCall")]
        optional_call: bool,
        #[serde(rename = "optionalReceiver")]
        optional_receiver: bool,
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
/// Reviewed OnCardPlayed watcher filter over the entering card.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayProjection {
    /// `cardTypeWhenInPlay === "basicUnit"` or `isUnit(cardTypeWhenInPlay)`.
    pub unit: bool,
    /// Friendly controller (`event.player === player`).
    pub same_controller: bool,
    /// Exclude the watcher (`event.card !== source`).
    pub other: bool,
    /// Optional printed aspect filter on the played card.
    pub aspect: Option<String>,
    /// Optional printed trait filter on the played card.
    pub trait_name: Option<String>,
    /// Optional printed cost upper bound (`event.card.cost <= N`).
    pub max_cost: Option<u64>,
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
    /// Base watchers: `event.card.controller === player` (friendly leader deploy).
    pub(crate) fn leader_deploy_friendly_controller(&self) -> bool {
        let mut binding = None;
        matches!(
            self,
            Self::Binary { op, left, right }
                if op == "==="
                    && left.path(&["card", "controller"], &mut binding)
                    && right.reference(&["player"])
        )
    }
    fn const_str(&self, wanted: &str) -> bool {
        matches!(self, Self::Const { value } if value.as_str() == Some(wanted))
    }
    fn play_unit_type(&self, binding: &mut Option<String>) -> bool {
        match self {
            Self::Binary { op, left, right }
                if op == "==="
                    && left.path(&["cardTypeWhenInPlay"], binding)
                    && right.const_str("basicUnit") =>
            {
                true
            }
            Self::EnumTypeTest { test, args }
                if test == "isUnit"
                    && args.len() == 1
                    && args[0].path(&["cardTypeWhenInPlay"], binding) =>
            {
                true
            }
            _ => false,
        }
    }
    fn play_data_call(
        &self,
        method: &str,
        binding: &mut Option<String>,
    ) -> Option<String> {
        let Self::DataCall {
            method: name,
            receiver,
            args,
            optional_call: false,
            optional_receiver: false,
        } = self
        else {
            return None;
        };
        if name != method || args.len() != 1 || !receiver.path(&["card"], binding) {
            return None;
        }
        match &args[0] {
            Self::Const { value } => value.as_str().filter(|s| !s.is_empty() && s.len() <= 128).map(str::to_owned),
            _ => None,
        }
    }
    fn play_project(
        &self,
        out: &mut PlayProjection,
        binding: &mut Option<String>,
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        match self {
            Self::Binary { op, left, right } if op == "&&" => {
                left.play_project(out, binding, depth + 1)
                    && right.play_project(out, binding, depth + 1)
            }
            _ if self.play_unit_type(binding) && !out.unit => {
                out.unit = true;
                true
            }
            Self::Binary { op, left, right }
                if op == "==="
                    && left.path(&["player"], binding)
                    && right.reference(&["player"])
                    && !out.same_controller =>
            {
                out.same_controller = true;
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
            Self::Binary { op, left, right }
                if op == "<="
                    && left.path(&["card", "cost"], binding)
                    && out.max_cost.is_none() =>
            {
                if let Self::Const { value } = right.as_ref() {
                    if let Some(n) = value.as_u64().filter(|n| (0..=20).contains(n)) {
                        out.max_cost = Some(n);
                        return true;
                    }
                }
                false
            }
            _ => {
                if out.aspect.is_none() {
                    if let Some(aspect) = self.play_data_call("hasSomeAspect", binding) {
                        out.aspect = Some(aspect);
                        return true;
                    }
                }
                if out.trait_name.is_none() {
                    if let Some(trait_name) = self.play_data_call("hasSomeTrait", binding) {
                        out.trait_name = Some(trait_name);
                        return true;
                    }
                }
                false
            }
        }
    }
    pub(crate) fn play_projection(&self) -> Option<PlayProjection> {
        let mut p = PlayProjection::default();
        self.play_project(&mut p, &mut None, 0)
            .then_some(p)
            .filter(|p| p.unit && p.same_controller)
    }
}
fn plain_hook_header(a: &Ability) -> bool {
    a.trigger == "eventHooks"
        && !a.hooks.is_empty()
        && a.hooks.len() <= 2
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && !a.attacker_must_survive
        && a.costs.is_empty()
        && a.context_title.is_empty()
        && a.target_resolvers.is_empty()
        && a.condition.is_none()
        && a.then.is_none()
        && a.if_you_do.is_none()
}
pub(crate) fn registration(a: &Ability) -> bool {
    if !plain_hook_header(a) {
        return false;
    }
    let mut deaths = 0;
    let mut plays = 0;
    for hook in &a.hooks {
        match hook {
            Hook::Alias(v) if v.enabled && matches!(v.alias, AliasName::WhenPlayed) => plays += 1,
            Hook::Event(v)
                if matches!(v.event, EventName::Defeated) && v.predicate.projection().is_some() =>
            {
                deaths += 1
            }
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
/// Sole WhenPlayedUsingSmuggle alias with self Experience 1..=3 or Ready self.
pub(crate) fn smuggle_play_registration(a: &Ability) -> bool {
    if !plain_hook_header(a) || a.hooks.len() != 1 || a.target_resolver.is_some() {
        return false;
    }
    match &a.hooks[0] {
        Hook::Alias(v) if v.enabled && matches!(v.alias, AliasName::WhenPlayedUsingSmuggle) => {}
        _ => return false,
    }
    match &a.effect {
        Effect::GiveExperience {
            target: Target::Name(t),
            amount: Expr::Const { value },
        } => t == "self" && value.as_u64().is_some_and(|n| (1..=3).contains(&n)),
        Effect::Ready {
            target: Target::Name(t),
        } => t == "self",
        _ => false,
    }
}
/// Sole OnLeaderDeployed watcher: friendly controller deploy → Battle Droid×2 or Draw 1.
pub(crate) fn leader_deploy_base_registration(a: &Ability) -> bool {
    if !plain_hook_header(a) || a.hooks.len() != 1 || a.target_resolver.is_some() {
        return false;
    }
    match &a.hooks[0] {
        Hook::Event(v)
            if matches!(v.event, EventName::LeaderDeployed)
                && v.predicate.leader_deploy_friendly_controller() => {}
        _ => return false,
    }
    match &a.effect {
        Effect::CreateTokenUnit {
            token_type,
            amount: Expr::Const { value },
            target: Target::Name(t),
            enters_ready,
        } => {
            token_type == "battleDroid"
                && value.as_u64() == Some(2)
                && t == "ownPlayer"
                && !*enters_ready
        }
        Effect::Draw {
            target: Target::Name(t),
            amount: Expr::Const { value },
        } => t == "ownPlayer" && value.as_u64() == Some(1),
        _ => false,
    }
}
/// Sole OnCardPlayed watcher: friendly unit play → self Experience, own-base heal, or phase Sentinel.
pub(crate) fn card_played_registration(a: &Ability) -> bool {
    if !plain_hook_header(a) || a.hooks.len() != 1 || a.target_resolver.is_some() {
        return false;
    }
    match &a.hooks[0] {
        Hook::Event(v)
            if matches!(v.event, EventName::CardPlayed) && v.predicate.play_projection().is_some() => {}
        _ => return false,
    }
    let one = |e: &Expr| matches!(e, Expr::Const { value } if value.as_u64() == Some(1));
    match &a.effect {
        Effect::GiveExperience {
            target: Target::Name(t),
            amount,
        } => t == "self" && one(amount),
        Effect::Heal {
            target: Target::Name(t),
            amount,
        } => t == "ownBase" && one(amount),
        Effect::GainKeyword {
            target: Target::Name(t),
            keyword,
            amount,
            duration,
        } => t == "self" && keyword == "sentinel" && duration == "phase" && one(amount),
        _ => false,
    }
}
pub(crate) fn reject_reason(a: &Ability) -> String {
    for hook in &a.hooks {
        if let Hook::Unsupported(v) = hook {
            if let Some(event) = v.get("event").and_then(Value::as_str) {
                return format!("unimplemented event hook {event}");
            }
            if let Some(alias) = v.get("alias").and_then(Value::as_str) {
                return format!("unimplemented hook alias {alias}");
            }
            return "unsupported event hook shape".into();
        }
        if let Hook::Event(v) = hook {
            if matches!(v.event, EventName::LeaderDeployed) {
                return "leader-deploy requires exact reviewed base registration".into();
            }
            if matches!(v.event, EventName::CardPlayed) {
                return "onCardPlayed requires exact reviewed unit registration".into();
            }
        }
    }
    if a.hooks.iter().any(|h| {
        matches!(
            h,
            Hook::Alias(v) if matches!(v.alias, AliasName::WhenPlayedUsingSmuggle)
        )
    }) {
        return "smuggle when-played requires complete ordinary self Experience/Ready registration"
            .into();
    }
    "defeat hooks require one bounded typed root registration".into()
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
fn ordinary_unit_hook_source(e: &Value, sides: &Sides) -> bool {
    ["historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"].iter().all(|k|e[*k].is_null()||e[*k].as_array().is_some_and(Vec::is_empty))
        && e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == serde_json::json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && !sides.card.is_empty()
        && sides.card.len() <= 8
        && (e["stateWatchers"].is_null()
            || e["stateWatchers"] == serde_json::json!([])
            || e["stateWatchers"] == serde_json::json!({}))
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
}
pub(crate) fn entry(e: &Value, sides: &Sides) -> bool {
    ordinary_unit_hook_source(e, sides)
        && sides
            .card
            .iter()
            .filter(|a| marker(&serde_json::to_value(a).expect("serializable ability")))
            .all(registration)
}
pub(crate) fn smuggle_play_entry(e: &Value, sides: &Sides) -> bool {
    ordinary_unit_hook_source(e, sides)
        && e["card"]["keywords"]
            .as_array()
            .is_some_and(|k| k.iter().any(|v| v == "smuggle"))
        && sides.card.len() == 1
        && smuggle_play_registration(&sides.card[0])
}
pub(crate) fn card_played_entry(e: &Value, sides: &Sides) -> bool {
    ordinary_unit_hook_source(e, sides)
        && sides
            .card
            .iter()
            .filter(|a| marker(&serde_json::to_value(a).expect("serializable ability")))
            .all(card_played_registration)
}
pub(crate) fn leader_deploy_base_entry(e: &Value, sides: &Sides) -> bool {
    ["historyHelpers","privatePrograms","classMethods","privateClassConstants","deploymentOverride","setupValidationOverrides"].iter().all(|k|e[*k].is_null()||e[*k].as_array().is_some_and(Vec::is_empty))
        && e["baseClass"] == "BaseCard"
        && e["card"]["types"] == serde_json::json!(["base"])
        && e["card"]["unique"] == false
        && e["card"]["cost"].is_null()
        && e["card"]["power"].is_null()
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
        && e["card"]["rules"].is_null()
        && e["card"]["deployBox"].is_null()
        && e["card"]["upgradePower"].is_null()
        && e["card"]["upgradeHp"].is_null()
        && e["keywordValues"] == serde_json::json!({})
        && e["card"]["keywords"] == serde_json::json!([])
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && (e["stateWatchers"].is_null()
            || e["stateWatchers"] == serde_json::json!([])
            || e["stateWatchers"] == serde_json::json!({}))
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && sides.card.len() == 1
        && leader_deploy_base_registration(&sides.card[0])
        && crate::ability_ir::basezone_source_kind(e) == Some(false)
}
