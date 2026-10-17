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
    #[serde(rename = "cardsDefeatedThisPhase")]
    Defeated,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    #[serde(rename = "someCardEnteredPlay")]
    SomeEntered,
    #[serde(rename = "someCardPlayed")]
    SomePlayed,
    #[serde(rename = "getCardsPlayed")]
    PlayedList,
    #[serde(rename = "someDefeatedUnitControlledByPlayer")]
    SomeDefeatedUnit,
    #[serde(rename = "someDefeatedWhileAttackingUnitControlledByPlayer")]
    SomeDefeatedWhileAttackingUnit,
    #[serde(rename = "getDefeatedUnitsControlledByPlayer")]
    DefeatedUnitsList,
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
    /// The pinned watchers that take a controller argument receive one exact
    /// `context.player`/`context.opponent` reference, never a lambda.
    Ref { path: Vec<String> },
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
/// Relative controller argument of a defeat query: the pinned operations take
/// `context.player`/`context.opponent`, resolved against the ability's controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRef {
    Player,
    Opponent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    EnteredCard,
    EnteredOtherFriendly,
    PlayedForce,
    PlayedOpponentCount,
    /// `someDefeatedUnitControlledByPlayer(controller)`.
    DefeatedUnit(PlayerRef),
    /// `someDefeatedWhileAttackingUnitControlledByPlayer(controller)`.
    DefeatedWhileAttacking(PlayerRef),
    /// `getDefeatedUnitsControlledByPlayer(controller)` consumed as `length`.
    DefeatedUnitCount(PlayerRef),
}
impl QueryKind {
    /// Kinds whose value is a count and may only be read through a `length`
    /// consumer, never as a boolean predicate.
    pub(crate) fn count(self) -> bool {
        matches!(self, Self::PlayedOpponentCount | Self::DefeatedUnitCount(_))
    }
    /// Kinds that read the `cardsDefeatedThisPhase` watcher.
    pub(crate) fn defeat_projection(self) -> bool {
        matches!(
            self,
            Self::DefeatedUnit(_) | Self::DefeatedWhileAttacking(_) | Self::DefeatedUnitCount(_)
        )
    }
}
/// One exact controller reference: `ref player` or `ref opponent`.
fn player_ref(args: &[Argument]) -> Option<PlayerRef> {
    let [Argument::Ref { path }] = args else {
        return None;
    };
    match path.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["player"] => Some(PlayerRef::Player),
        ["opponent"] => Some(PlayerRef::Opponent),
        _ => None,
    }
}
impl Query {
    /// O(1) projection after kind() has validated the complete immutable filter.
    pub(crate) fn admitted_kind(&self) -> QueryKind {
        let player_ref = || player_ref(&self.args).expect("controller argument validated at admission");
        match (&self.watcher, &self.operation) {
            (Watcher::Entered, Operation::SomeEntered) => {
                let Argument::DataLambda { body, .. } = &self.args[0] else {
                    unreachable!("entry projection was validated at admission")
                };
                if matches!(body,Filter::Binary{op,..} if op=="&&") {
                    QueryKind::EnteredOtherFriendly
                } else {
                    QueryKind::EnteredCard
                }
            }
            (Watcher::Played, Operation::SomePlayed) => QueryKind::PlayedForce,
            (Watcher::Played, Operation::PlayedList) => QueryKind::PlayedOpponentCount,
            (Watcher::Defeated, Operation::SomeDefeatedUnit) => {
                QueryKind::DefeatedUnit(player_ref())
            }
            (Watcher::Defeated, Operation::SomeDefeatedWhileAttackingUnit) => {
                QueryKind::DefeatedWhileAttacking(player_ref())
            }
            (Watcher::Defeated, Operation::DefeatedUnitsList) => {
                QueryKind::DefeatedUnitCount(player_ref())
            }
            _ => unreachable!("history projection was validated at admission"),
        }
    }
    pub fn kind(&self) -> Result<QueryKind, String> {
        let invalid = || "unsupported phase history watcher/filter projection".to_string();
        if self.binding.is_empty() || self.binding.len() > 128 || self.args.len() != 1 {
            return Err(invalid());
        }
        // The three defeat projections take one exact controller reference and
        // read the live per-controller defeat counters. Every other combination,
        // including a lambda or an extra argument, stays rejected.
        if let Some(side) = player_ref(&self.args) {
            return match (&self.watcher, &self.operation) {
                (Watcher::Defeated, Operation::SomeDefeatedUnit) => {
                    Ok(QueryKind::DefeatedUnit(side))
                }
                (Watcher::Defeated, Operation::SomeDefeatedWhileAttackingUnit) => {
                    Ok(QueryKind::DefeatedWhileAttacking(side))
                }
                (Watcher::Defeated, Operation::DefeatedUnitsList) => {
                    Ok(QueryKind::DefeatedUnitCount(side))
                }
                _ => Err(invalid()),
            };
        }
        let Argument::DataLambda { bindings, body } = &self.args[0] else {
            return Err(invalid());
        };
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
            (Watcher::Played, Operation::SomePlayed)
                if body
                    == binary(
                        "&&",
                        friendly,
                        json!({"kind":"dataCall","method":"hasSomeTrait","receiver":path("card"),"args":[{"kind":"const","value":"force"}],"optionalCall":false,"optionalReceiver":false}),
                    ) =>
            {
                Ok(QueryKind::PlayedForce)
            }
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
    // `cardsDefeatedThisPhase` existence projections. Each clause is one exact
    // ordinary registration whose only condition reads the live watcher through
    // one controller reference; the branches are the same supported effects the
    // ordinary runtime already executes for the same trigger.
    let negative =
        |e: &Expr, n: i64| matches!(e.original(),Expr::Unary{op,value} if op=="-"&&literal(value,n));
    let phase_stats = |e: &Effect, n: i64| {
        matches!(e,Effect::ModifyStats{target:Target::Name(t),duration,power,hp} if t=="chosen"&&duration=="phase"&&negative(power,n)&&negative(hp,n))
    };
    let unit_resolver = |owner: &str| {
        a.target_resolver.as_ref().is_some_and(|r|r.owner==owner&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.arena.is_none()&&r.prompt_title.is_none()&&r.prompt_title_parts.is_empty()&&r.predicate.is_none()&&r.native.is_none())
    };
    // For each friendly unit that was defeated this phase, ready a friendly
    // resource: the count consumer of `getDefeatedUnitsControlledByPlayer`.
    if a.trigger == "onAttack"
        && !a.optional
        && a.costs.is_empty()
        && a.target_resolver.is_none()
        && matches!(&a.effect,Effect::ReadyResources{target:Target::Name(t),amount}
            if t=="ownPlayer"&&matches!(amount.original(),Expr::DataMember{receiver:List::HistoryQuery(q),key:Length::Length,optional:false} if q.kind()==Ok(QueryKind::DefeatedUnitCount(PlayerRef::Player))))
    {
        return true;
    }
    if let Effect::Conditional {
        condition,
        on_true,
        on_false,
    } = &a.effect
    {
        let history = |kind: QueryKind| {
            matches!(condition.original(),Expr::HistoryQuery(q) if q.kind()==Ok(kind))
        };
        let no_action = matches!(on_false.as_deref(),Some(Effect::NoAction));
        // If a friendly unit was defeated this phase: heal 4 from your base.
        if a.trigger == "whenPlayed"
            && !a.optional
            && a.costs.is_empty()
            && a.target_resolver.is_none()
            && history(QueryKind::DefeatedUnit(PlayerRef::Player))
            && no_action
            && matches!(on_true.as_ref(),Effect::Heal{target:Target::Name(t),amount} if t=="ownBase"&&literal(amount,4))
        {
            return true;
        }
        // If a friendly unit was defeated this phase: create a Spy token.
        if a.trigger == "whenPlayed"
            && !a.optional
            && a.costs.is_empty()
            && a.target_resolver.is_none()
            && history(QueryKind::DefeatedUnit(PlayerRef::Player))
            && no_action
            && matches!(on_true.as_ref(),Effect::CreateTokenUnit{token_type,amount,target,enters_ready} if token_type=="spy"&&literal(amount,1)&&matches!(target,Target::Name(t) if t=="ownPlayer")&&!enters_ready)
        {
            return true;
        }
        // If a friendly unit was defeated this phase, you may give a unit -1/-1.
        if a.trigger == "whenPlayed"
            && a.optional
            && a.costs.is_empty()
            && unit_resolver("any")
            && history(QueryKind::DefeatedUnit(PlayerRef::Player))
            && no_action
            && phase_stats(on_true, 1)
        {
            return true;
        }
        // If a friendly unit was defeated this phase: give a unit 2 Experience.
        if a.trigger == "event"
            && !a.optional
            && a.costs.is_empty()
            && unit_resolver("any")
            && history(QueryKind::DefeatedUnit(PlayerRef::Player))
            && no_action
            && matches!(on_true.as_ref(),Effect::GiveExperience{target:Target::Name(t),amount} if t=="chosen"&&literal(amount,2))
        {
            return true;
        }
        // Give an enemy unit -3/-3, or -6/-6 if a friendly unit was defeated this phase.
        if a.trigger == "whenPlayed"
            && !a.optional
            && a.costs.is_empty()
            && unit_resolver("opponent")
            && history(QueryKind::DefeatedUnit(PlayerRef::Player))
            && phase_stats(on_true, 6)
            && on_false.as_deref().is_some_and(|e| phase_stats(e, 3))
        {
            return true;
        }
        // If a friendly unit was defeated while attacking this phase: draw 3.
        if a.trigger == "event"
            && !a.optional
            && a.costs.is_empty()
            && a.target_resolver.is_none()
            && history(QueryKind::DefeatedWhileAttacking(PlayerRef::Player))
            && no_action
            && matches!(on_true.as_ref(),Effect::Draw{target:Target::Name(t),amount} if t=="ownPlayer"&&literal(amount,3))
        {
            return true;
        }
    }
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
                Ok(QueryKind::PlayedForce) => {
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
    (event || unit)
        && s.card.len() == 1
        && s.leader.is_empty()
        && s.unit.is_empty()
        && ability(&s.card[0])
        && (if event {
            s.card[0].trigger == "event"
        } else {
            ["whenPlayed", "action", "onAttack"].contains(&s.card[0].trigger.as_str())
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
