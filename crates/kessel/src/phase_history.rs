//! Exact projections of phase watchers for bounded existential/count queries.
//! Bitsets retain physical identity and controller-at-event independently of zones;
//! play counts retain repeated plays that an existential bitset intentionally merges.
use crate::ability_ir::{Ability, Cost, Effect, Expr, Sides, Target};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub watcher: Watcher,
    pub binding: String,
    pub operation: Operation,
    pub args: Vec<Argument>,
    pub read_at: ReadAt,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Watcher {
    #[serde(rename = "cardsEnteredPlayThisPhase")]
    Entered,
    #[serde(rename = "cardsPlayedThisPhase")]
    Played,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    #[serde(rename = "someCardEnteredPlay")]
    SomeEntered,
    #[serde(rename = "someCardPlayed")]
    SomePlayed,
    #[serde(rename = "getCardsPlayed")]
    PlayedList,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadAt {
    #[serde(rename = "expressionEvaluation")]
    Expression,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Argument {
    DataLambda { bindings: Vec<String>, body: Filter },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Filter {
    Binary {
        op: String,
        left: Box<Filter>,
        right: Box<Filter>,
    },
    DataPath {
        root: Vec<String>,
        steps: Vec<Step>,
    },
    Ref {
        path: Vec<String>,
    },
    Const {
        value: String,
    },
    DataCall {
        method: String,
        receiver: Box<Filter>,
        args: Vec<Filter>,
        #[serde(rename = "optionalCall")]
        optional_call: bool,
        #[serde(rename = "optionalReceiver")]
        optional_receiver: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    key: String,
    optional: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum List {
    HistoryQuery(Query),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Length {
    #[serde(rename = "length")]
    Length,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryKind {
    EnteredCard,
    EnteredOtherFriendly,
    /// Friendly card played this phase with an exact printed trait (Force, First Order, …).
    PlayedTrait(String),
    PlayedOpponentCount,
}
impl Query {
    /// Projection after kind() has validated the complete immutable filter.
    pub(crate) fn admitted_kind(&self) -> QueryKind {
        self.kind()
            .expect("history projection was validated at admission")
    }
    pub fn kind(&self) -> Result<QueryKind, String> {
        let invalid = || "unsupported phase history watcher/filter projection".to_string();
        if self.binding.is_empty() || self.binding.len() > 128 || self.args.len() != 1 {
            return Err(invalid());
        }
        let Argument::DataLambda { bindings, body } = &self.args[0];
        if bindings.len() != 1 || bindings[0].is_empty() || bindings[0].len() > 128 {
            return Err(invalid());
        }
        fn shape(f: &Filter, b: &str, depth: usize) -> Result<Value, String> {
            if depth > 8 {
                return Err("phase history predicate nesting exceeds capacity".into());
            }
            let mut v = serde_json::to_value(f).map_err(|e| e.to_string())?;
            match f {
                Filter::DataPath { root, .. } => {
                    if root != &[b] {
                        return Err("phase history predicate must bind its own entry".into());
                    }
                    v["root"] = json!(["entry"]);
                }
                Filter::Binary { left, right, .. } => {
                    v["left"] = shape(left, b, depth + 1)?;
                    v["right"] = shape(right, b, depth + 1)?;
                }
                Filter::DataCall { receiver, args, .. } => {
                    v["receiver"] = shape(receiver, b, depth + 1)?;
                    v["args"] = Value::Array(
                        args.iter()
                            .map(|f| shape(f, b, depth + 1))
                            .collect::<Result<_, _>>()?,
                    );
                }
                _ => {}
            }
            Ok(v)
        }
        let body = shape(body, &bindings[0], 0)?;
        let path = |key: &str| json!({"kind":"dataPath","root":["entry"],"steps":[{"key":key,"optional":false}]});
        let reference = |name: &str| json!({"kind":"ref","path":[name]});
        let binary = |op: &str, left: Value, right: Value| json!({"kind":"binary","op":op,"left":left,"right":right});
        let friendly = binary("===", path("playedBy"), reference("player"));
        let played_trait = |body: &Value| -> Option<String> {
            if body.get("kind") != Some(&json!("binary")) || body.get("op") != Some(&json!("&&")) {
                return None;
            }
            if body.get("left") != Some(&friendly) {
                return None;
            }
            let right = body.get("right")?;
            if right.get("kind") != Some(&json!("dataCall"))
                || right.get("method") != Some(&json!("hasSomeTrait"))
                || right.get("receiver") != Some(&path("card"))
                || right.get("optionalCall") != Some(&json!(false))
                || right.get("optionalReceiver") != Some(&json!(false))
            {
                return None;
            }
            let args = right.get("args")?.as_array()?;
            if args.len() != 1 || args[0].get("kind") != Some(&json!("const")) {
                return None;
            }
            let trait_name = args[0].get("value")?.as_str()?;
            if trait_name.is_empty() || trait_name.len() > 128 {
                return None;
            }
            Some(trait_name.to_owned())
        };
        match (&self.watcher, &self.operation) {
            (Watcher::Entered, Operation::SomeEntered)
                if body == binary("===", path("card"), reference("card")) =>
            {
                Ok(QueryKind::EnteredCard)
            }
            (Watcher::Entered, Operation::SomeEntered)
                if body
                    == binary(
                        "&&",
                        friendly.clone(),
                        binary("!==", path("card"), reference("source")),
                    ) =>
            {
                Ok(QueryKind::EnteredOtherFriendly)
            }
            (Watcher::Played, Operation::SomePlayed) => played_trait(&body)
                .map(QueryKind::PlayedTrait)
                .ok_or_else(invalid),
            (Watcher::Played, Operation::PlayedList)
                if body == binary("===", path("playedBy"), reference("opponent")) =>
            {
                Ok(QueryKind::PlayedOpponentCount)
            }
            _ => Err(invalid()),
        }
    }
}
pub(crate) fn marker(v: &Value) -> bool { crate::ability_ir::contains(v,|v|v["kind"] == "historyQuery") }
pub(crate) fn ability(a: &Ability) -> bool {
    if a.coordinate
        || a.requires_confirmation
        || a.gained_host.is_some()
        || a.attached_context.is_some()
        || !a.context_title.is_empty()
        || !a.target_resolvers.is_empty()
        || a.condition.is_some()
        || a.then.is_some()
        || a.if_you_do.is_some()
    {
        return false;
    }
    let literal =
        |e: &Expr, n: i64| matches!(e.original(),Expr::Const{value} if value.as_i64()==Some(n));
    let draw = |e: &Effect| matches!(e,Effect::Draw{target:Target::Name(t),amount} if t=="ownPlayer"&&literal(amount,1));
    let simple_resolver = |predicate: bool| {
        a.target_resolver.as_ref().is_some_and(|r|r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.arena.is_none()&&r.prompt_title.is_none()&&if predicate{matches!(&r.predicate,Some(Expr::HistoryQuery(q)) if q.kind()==Ok(QueryKind::EnteredCard))}else{r.predicate.is_none()})
    };
    if let Effect::Conditional {
        condition,
        on_true,
        on_false,
    } = &a.effect
    {
        if !matches!(on_false.as_deref(), Some(Effect::NoAction)) {
            return false;
        }
        if let Expr::HistoryQuery(q) = condition.original() {
            return match q.kind() {
                Ok(QueryKind::EnteredOtherFriendly) => {
                    a.trigger == "whenPlayed"
                        && !a.optional
                        && a.costs.is_empty()
                        && a.target_resolver.is_none()
                        && draw(on_true)
                }
                Ok(QueryKind::PlayedTrait(_)) => {
                    a.trigger == "action"
                        && !a.optional
                        && matches!(a.costs.as_slice(), [Cost::ExhaustSelf])
                        && a.target_resolver.is_none()
                        && draw(on_true)
                }
                _ => false,
            };
        }
        if let Expr::Binary { op, left, right } = condition.original() {
            return a.trigger == "whenPlayed"
                && a.optional
                && a.costs.is_empty()
                && simple_resolver(false)
                && op == ">="
                && literal(right, 2)
                && matches!(left.original(),Expr::DataMember{receiver:List::HistoryQuery(q),optional:false,..} if q.kind()==Ok(QueryKind::PlayedOpponentCount))
                && matches!(on_true.as_ref(),Effect::GiveWeakness{target:Target::Name(t),amount} if t=="chosen"&&literal(amount,2));
        }
    }
    let negative_four =
        |e: &Expr| matches!(e.original(),Expr::Unary{op,value} if op=="-"&&literal(value,4));
    a.trigger == "event"
        && !a.optional
        && a.costs.is_empty()
        && simple_resolver(true)
        && matches!(&a.effect,Effect::ModifyStats{target:Target::Name(t),duration,power,hp} if t=="chosen"&&duration=="phase"&&negative_four(power)&&negative_four(hp))
}
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(s) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    let event = e["baseClass"] == "EventCard" && e["card"]["types"] == json!(["event"]);
    let unit = e["baseClass"] == "NonLeaderUnitCard" && e["card"]["types"] == json!(["unit"]);
    // Exactly one history-bearing registration must match a bounded projection.
    // Ordinary companion abilities on the same card-side are validated elsewhere.
    let history = s
        .card
        .iter()
        .filter(|a| marker(&serde_json::to_value(a).unwrap_or(Value::Null)))
        .collect::<Vec<_>>();
    (event || unit)
        && !s.card.is_empty()
        && s.card.len() <= 8
        && s.leader.is_empty()
        && s.unit.is_empty()
        && history.len() == 1
        && ability(history[0])
        && (if event {
            history[0].trigger == "event"
        } else {
            ["whenPlayed", "action"].contains(&history[0].trigger.as_str())
        })
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
        .all(|k| e["card"][*k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
}
