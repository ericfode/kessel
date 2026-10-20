//! Closed selector input contracts from the pinned CardTargetResolver and
//! EnumHelpers. This module is shared by the build-time and runtime IR loaders.
use super::{Expr, TargetResolver};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CardTypeFilter {
    Any,
    Unit,
    NonLeaderUnit,
    LeaderUnit,
    NonTokenUnit,
    NonUnit,
    Upgrade,
    NonLeaderUpgrade,
    UnitUpgrade,
    Token,
    Playable,
    Base,
    BasicUnit,
    BasicUpgrade,
    Event,
    Leader,
    NonTokenLeaderUnit,
    TokenLeaderUnit,
    LeaderUpgrade,
    TokenUnit,
    TokenUpgrade,
    TokenCard,
    NonLeaderUnitUpgrade,
}
impl CardTypeFilter {
    pub fn parse(value: &str) -> Result<Self, String> {
        serde_json::from_value(Value::String(value.into()))
            .map_err(|_| format!("unsupported target type {value}"))
    }
    pub fn matches(self, actual: Self) -> bool {
        use CardTypeFilter::*;
        match self {
            Any => true,
            Unit => matches!(
                actual,
                BasicUnit | NonTokenLeaderUnit | TokenLeaderUnit | TokenUnit
            ),
            NonLeaderUnit => matches!(actual, BasicUnit | TokenUnit),
            LeaderUnit => matches!(actual, NonTokenLeaderUnit | TokenLeaderUnit),
            NonTokenUnit => matches!(actual, BasicUnit | NonTokenLeaderUnit),
            NonUnit => !Unit.matches(actual),
            Upgrade => matches!(
                actual,
                BasicUpgrade | LeaderUpgrade | TokenUpgrade | NonLeaderUnitUpgrade
            ),
            NonLeaderUpgrade => {
                matches!(actual, BasicUpgrade | TokenUpgrade | NonLeaderUnitUpgrade)
            }
            UnitUpgrade => matches!(actual, LeaderUpgrade | NonLeaderUnitUpgrade),
            Token => matches!(actual, TokenUpgrade | TokenUnit | TokenCard),
            Playable => matches!(
                actual,
                Event | BasicUnit | BasicUpgrade | NonLeaderUnitUpgrade
            ),
            concrete => concrete == actual,
        }
    }
    pub fn upgrade_only(self) -> bool {
        use CardTypeFilter::*;
        [
            BasicUpgrade,
            TokenUpgrade,
            LeaderUpgrade,
            NonLeaderUnitUpgrade,
        ]
        .into_iter()
        .any(|t| self.matches(t))
            && [
                BasicUnit,
                TokenUnit,
                NonTokenLeaderUnit,
                TokenLeaderUnit,
                Event,
                Leader,
                Base,
                TokenCard,
            ]
            .into_iter()
            .all(|t| !self.matches(t))
    }
    pub fn legal_in_discard(self) -> bool {
        [
            Self::BasicUnit,
            Self::BasicUpgrade,
            Self::Event,
            Self::NonLeaderUnitUpgrade,
            Self::LeaderUpgrade,
        ]
        .into_iter()
        .any(|t| self.matches(t))
    }
    pub fn legal_in_arena(self) -> bool {
        [
            Self::BasicUnit,
            Self::BasicUpgrade,
            Self::Event,
            Self::NonTokenLeaderUnit,
            Self::TokenLeaderUnit,
            Self::LeaderUpgrade,
            Self::TokenUnit,
            Self::TokenUpgrade,
            Self::NonLeaderUnitUpgrade,
        ]
        .into_iter()
        .any(|t| self.matches(t))
    }
    pub fn legal_in_base(self) -> bool {
        [Self::Base, Self::Leader, Self::TokenCard]
            .into_iter()
            .any(|t| self.matches(t))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorZone {
    AnyAttackable,
    AnyArena,
    GroundArena,
    SpaceArena,
    Base,
    Discard,
}
impl SelectorZone {
    /// Ordered inventory partitions, not an unordered union. The caller retains
    /// the literal zone list, including repeated zones, and visits seats first.
    pub fn includes_ground(self) -> bool {
        matches!(
            self,
            Self::AnyArena | Self::AnyAttackable | Self::GroundArena
        )
    }
    pub fn includes_space(self) -> bool {
        matches!(
            self,
            Self::AnyArena | Self::AnyAttackable | Self::SpaceArena
        )
    }
    pub fn includes_base(self) -> bool {
        matches!(self, Self::AnyAttackable | Self::Base)
    }
}

/// Native selectors currently execute only in ordinary card-side contexts.
/// This does not grant attached, leader, gained, or history candidate scopes.
fn ordinary_source_sides(sides: &super::Sides) -> bool {
    sides.unit.is_empty()
        && sides.leader.is_empty()
        && sides.card.iter().all(|a| {
            ["event", "action", "whenPlayed", "onAttack", "whenDefeated"]
                .contains(&a.trigger.as_str())
        })
}

/// Shared shape only: callers retain their own owner, optionality, predicate,
/// prompt, and timing requirements. `single` is deliberately not equivalent to
/// an omitted mode at these reviewed named-selector boundaries.
pub(super) fn scalar_arena_unit(resolver: &TargetResolver) -> bool {
    resolver.card_type == "unit"
        && resolver.mode.is_none()
        && resolver.count.is_none()
        && matches!(resolver.zone.as_deref(), Some("arena" | "anyAttackable"))
        && resolver
            .arena
            .as_deref()
            .is_none_or(|a| ["ground", "space"].contains(&a))
}

pub(super) fn own_named_binding(name: &str, chooser: &str) -> bool {
    !name.is_empty()
        && name != "target"
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && chooser == "self"
}

fn reference(expr: &Expr, expected: &[&str]) -> bool {
    matches!(expr.original(), Expr::Ref { path } if path.iter().map(String::as_str).eq(expected.iter().copied()))
}
fn literal_value(expr: &Expr, expected: &Value) -> bool {
    matches!(expr.original(), Expr::Const { value } if value == expected)
}
fn unary(expr: &Expr, operator: &str, value: impl FnOnce(&Expr) -> bool) -> bool {
    matches!(expr.original(), Expr::Unary { op, value: inner } if op == operator && value(inner))
}
fn binary(
    expr: &Expr,
    operator: &str,
    left_matches: impl FnOnce(&Expr) -> bool,
    right_matches: impl FnOnce(&Expr) -> bool,
) -> bool {
    matches!(expr.original(), Expr::Binary { op, left, right } if op == operator && left_matches(left) && right_matches(right))
}
fn candidate_is_unit(expr: &Expr) -> bool {
    matches!(expr.original(), Expr::Query { name, receiver, args } if name == "isUnit" && reference(receiver, &["card"]) && args.is_empty())
}
fn other_than_source(expr: &Expr) -> bool {
    binary(
        expr,
        "!==",
        |e| reference(e, &["card"]),
        |e| reference(e, &["source"]),
    )
}
fn nullable_other_than_source(expr: &Expr) -> bool {
    binary(
        expr,
        "||",
        |e| unary(e, "!", |e| reference(e, &["source"])),
        other_than_source,
    )
}
fn arena_list(expr: &Expr, owner: &str, predicate_matches: impl FnOnce(&Expr) -> bool) -> bool {
    matches!(expr.original(), Expr::ArenaList { receiver, predicate } if reference(receiver, &[owner]) && predicate_matches(predicate))
}
fn named_target(target: &super::Target, name: &str) -> bool {
    matches!(target, super::Target::Name(actual) if actual == name)
}

/// The three reviewed arena-query registrations share explicit physical-card,
/// source, and player binders. This replaces serialization-based ability matching;
/// the separate raw whole-source fingerprint remains a provenance boundary.
pub(super) fn reviewed_arena_query_ability(ability: &super::Ability) -> bool {
    use super::{Cost, Effect};
    if !ability.hooks.is_empty()
        || ability.gained_host.is_some()
        || ability.attached_context.is_some()
        || !ability.target_resolvers.is_empty()
        || ability.attacker_must_survive
        || ability.coordinate
        || !ability.context_title.is_empty()
        || ability.requires_confirmation
        || ability.then.is_some()
        || ability.if_you_do.is_some()
    {
        return false;
    }
    match ability.trigger.as_str() {
        "constant" => {
            !ability.optional && ability.costs.is_empty() && ability.target_resolver.is_none()
                && matches!(&ability.effect, Effect::GainKeyword { target, duration, keyword, amount }
                    if named_target(target, "self") && duration == "persistent" && keyword == "ambush" && literal_value(amount, &Value::from(1)))
                && ability.condition.as_ref().is_some_and(|e| {
                    matches!(e.original(), Expr::ArenaSome { receiver, predicate }
                        if arena_list(receiver, "player", |e| binary(e, "&&", |e| literal_value(e, &Value::Bool(true)), nullable_other_than_source))
                            && unary(predicate, "!", |e| reference(e, &["card", "unique"])))
                })
        }
        "whenPlayed" => {
            ability.optional && ability.costs.is_empty() && ability.condition.is_none()
                && ability.target_resolver.as_ref().is_some_and(|r|
                    super::RecipeSelector::one("any", "unit", "anyAttackable").matches(r)
                        && r.predicate.is_none() && r.prompt_title_parts.is_empty() && r.native.is_none())
                && matches!(&ability.effect, Effect::Conditional { condition, on_true, on_false }
                    if binary(condition, ">", |e| matches!(e.original(), Expr::ArenaLength { receiver }
                        if matches!(receiver.original(), Expr::ArenaFilter { receiver, predicate }
                            if arena_list(receiver, "player", |e| super::emitted_arena_scope(e, "spaceArena")) && other_than_source(predicate))),
                        |e| literal_value(e, &Value::from(0)))
                    && matches!(on_true.as_ref(), Effect::Exhaust { target } if named_target(target, "chosen"))
                    && matches!(on_false.as_deref(), Some(Effect::NoAction)))
        }
        "action" => {
            !ability.optional && ability.target_resolver.is_none() && ability.condition.is_none()
                && matches!(ability.costs.as_slice(), [Cost::ExhaustSelf, Cost::Resources { amount }] if literal_value(amount, &Value::from(1)))
                && matches!(&ability.effect, Effect::Conditional { condition, on_true, on_false }
                    if binary(condition, ">=", |e| matches!(e.original(), Expr::ArenaLength { receiver }
                        if arena_list(receiver, "game", |e| binary(e, "&&", |e| literal_value(e, &Value::Bool(true)),
                            |e| binary(e, "&&", candidate_is_unit, |e| reference(e, &["card", "exhausted"]))))),
                        |e| literal_value(e, &Value::from(4)))
                    && matches!(on_true.as_ref(), Effect::CreateTokenUnit { token_type, amount, target, enters_ready }
                        if token_type == "spy" && literal_value(amount, &Value::from(1)) && named_target(target, "ownPlayer") && !enters_ready)
                    && matches!(on_false.as_deref(), Some(Effect::NoAction)))
        }
        _ => false,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorController {
    #[serde(rename = "self")]
    Own,
    Opponent,
    Any,
}
impl SelectorController {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Own => "self",
            Self::Opponent => "opponent",
            Self::Any => "any",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorChooser {
    #[serde(rename = "self")]
    Own,
    Opponent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeCardSelection {
    kind: String,
    mode: String,
    pub chooser: SelectorChooser,
    result_binding: String,
    defaults: String,
    context_binding: String,
    choosing_player_override: String,
    delay_opponent_before_costs: bool,
    pub card_types: Expr,
    pub zones: Expr,
    pub controller: Expr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    predicate: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    optional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    can_choose_no_cards: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    prompt_title_parts: Vec<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolver_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    condition: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    must_change_game_state: Option<String>,
}
fn literal<T: serde::de::DeserializeOwned>(expr: &Expr, what: &str) -> Result<T, String> {
    let Expr::Const { value } = expr else {
        return Err(format!("native selector {what} requires a static literal"));
    };
    serde_json::from_value(value.clone())
        .map_err(|_| format!("unsupported native selector {what}: {value}"))
}
fn literals<T: serde::de::DeserializeOwned>(expr: &Expr, what: &str) -> Result<Vec<T>, String> {
    let values = if let Expr::Array { items } = expr {
        items
            .iter()
            .map(|e| literal(e, what))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![literal(expr, what)?]
    };
    if values.is_empty() || values.len() > 8 {
        return Err(format!(
            "native selector {what} list must contain one to eight entries"
        ));
    }
    Ok(values)
}
impl NativeCardSelection {
    pub fn types(&self) -> Result<Vec<CardTypeFilter>, String> {
        literals(&self.card_types, "cardTypes")
    }
    pub fn zone_filters(&self) -> Result<Vec<SelectorZone>, String> {
        literals(&self.zones, "zones")
    }
    pub fn controller_filter(&self) -> Result<SelectorController, String> {
        literal(&self.controller, "controller")
    }
    pub fn is_opponent(&self) -> bool {
        self.chooser == SelectorChooser::Opponent
    }
    pub fn is_root_binding(&self) -> bool {
        self.result_binding == "target"
    }
    pub fn matches_named_binding(&self, name: &str, chooser: &str) -> bool {
        self.result_binding == name
            && chooser
                == if self.is_opponent() {
                    "opponent"
                } else {
                    "self"
                }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.kind != "cardSelection"
            || self.mode != "single"
            || self.defaults != "pinnedTargetResolver"
            || self.context_binding != "target"
            || self.choosing_player_override != "contextOverrideFirst"
            || !self.delay_opponent_before_costs
        {
            return Err(
                "native card selector requires pinned single-card context/default/chooser protocol"
                    .into(),
            );
        }
        if self.result_binding.is_empty()
            || self.result_binding.len() > 32
            || !self
                .result_binding
                .chars()
                .enumerate()
                .all(|(i, c)| c.is_ascii_alphabetic() || c == '_' || (i > 0 && c.is_ascii_digit()))
            || ["__proto__", "prototype", "constructor"].contains(&self.result_binding.as_str())
            || self
                .resolver_name
                .as_ref()
                .is_some_and(|name| name != &self.result_binding)
        {
            return Err("invalid native selector result binding".into());
        }
        if self.condition.is_some() {
            return Err("native selector resolution-time condition requires a separate skip/result contract".into());
        }
        if self.must_change_game_state.is_some() {
            return Err("native selector explicit mustChangeGameState is not implemented".into());
        }
        let types = self.types()?;
        let zones = self.zone_filters()?;
        self.controller_filter()?;
        if zones.iter().any(|z| *z != SelectorZone::Discard)
            && types.iter().any(|t| {
                t.matches(CardTypeFilter::BasicUpgrade) || t.matches(CardTypeFilter::TokenUpgrade)
            })
        {
            return Err("native selector arena upgrade inventory/order requires a separately reviewed selector contract".into());
        }
        if !types.iter().any(|t| {
            zones.iter().any(|z| match z {
                SelectorZone::Discard => t.legal_in_discard(),
                SelectorZone::Base => t.legal_in_base(),
                SelectorZone::AnyAttackable => t.legal_in_base() || t.legal_in_arena(),
                _ => t.legal_in_arena(),
            })
        }) {
            return Err("native selector type/zone filters have no legal overlap".into());
        }
        // Force is a virtual base-zone identity in the current model; Credits and
        // Fortify are not represented here. Do not fabricate their raw inventory.
        if zones
            .iter()
            .any(|z| matches!(z, SelectorZone::AnyAttackable | SelectorZone::Base))
            && types.iter().any(|t| t.matches(CardTypeFilter::TokenCard))
        {
            return Err(
                "native selector base-token inventory requires physical base-zone tokens".into(),
            );
        }
        let mut bytes = 0;
        for fragment in &self.prompt_title_parts {
            let Expr::Const { value } = fragment else {
                return Err("native selector prompt requires bounded static text".into());
            };
            bytes += value
                .as_str()
                .ok_or("native selector prompt requires string fragments")?
                .len();
        }
        if bytes > 1024 || self.prompt_title_parts.len() > 32 {
            return Err("native selector prompt exceeds capacity".into());
        }
        Ok(())
    }
    fn lift(self) -> Result<TargetResolver, String> {
        self.validate()?;
        let types = self.types()?;
        let zones = self.zone_filters()?;
        let card_type = if types.len() == 1 {
            serde_json::to_value(types[0])
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        } else {
            "any".into()
        };
        // A concrete non-unit has no unit-stat API merely because its metadata
        // has printed stats. Typed control-flow refinement is a separate contract.
        if let Some(predicate) = &self.predicate {
            fn unit_read(e: &Expr) -> bool {
                e.any(|e| match e {
                    Expr::Compiled(_) | Expr::Collection { .. } => Some(false),
                    Expr::Ref { path } => Some(
                        path.first().is_some_and(|s| {
                            ["selectionCandidate", "target"].contains(&s.as_str())
                        }) && path.iter().skip(1).any(|s| {
                            ["power", "hp", "remainingHp", "damage", "exhausted"]
                                .contains(&s.as_str())
                        }),
                    ),
                    Expr::Query { name, receiver, .. }
                        if ["getPower", "getHp"].contains(&name.as_str())
                            && matches!(receiver.as_ref(), Expr::Ref { path }
                                if path.first().is_some_and(|s| ["selectionCandidate", "target"].contains(&s.as_str()))) =>
                    {
                        Some(true)
                    }
                    _ => None,
                })
            }
            let non_unit = [
                CardTypeFilter::Base,
                CardTypeFilter::Leader,
                CardTypeFilter::Event,
                CardTypeFilter::BasicUpgrade,
                CardTypeFilter::LeaderUpgrade,
                CardTypeFilter::NonLeaderUnitUpgrade,
                CardTypeFilter::TokenUpgrade,
                CardTypeFilter::TokenCard,
            ];
            if unit_read(predicate)
                && types
                    .iter()
                    .any(|t| non_unit.iter().any(|actual| t.matches(*actual)))
            {
                return Err(
                    "native selector unit-stat predicate requires an exclusively unit type domain"
                        .into(),
                );
            }
        }
        let mut predicate = self.predicate.clone();
        if let Some(predicate) = &mut predicate {
            bind_candidate(predicate, self.is_root_binding())?;
            predicate.validate()?;
        }
        let (zone, arena) = if zones.len() == 1 {
            match zones[0] {
                SelectorZone::AnyAttackable => ("anyAttackable", None),
                SelectorZone::AnyArena => ("arena", None),
                SelectorZone::GroundArena => ("arena", Some("ground")),
                SelectorZone::SpaceArena => ("arena", Some("space")),
                SelectorZone::Base => ("base", None),
                SelectorZone::Discard => ("discard", None),
            }
        } else {
            ("anyAttackable", None)
        };
        Ok(TargetResolver {
            prompt_title: None,
            prompt_title_parts: self.prompt_title_parts.clone(),
            owner: self.controller_filter()?.as_str().into(),
            card_type,
            arena: arena.map(str::to_owned),
            zone: Some(zone.into()),
            optional: self.optional.unwrap_or(false) || self.can_choose_no_cards.unwrap_or(false),
            predicate,
            mode: None,
            count: None,
            native: Some(self),
            player_selection: false,
        })
    }
}
fn captures_outer_candidate(e: &Expr) -> bool {
    e.any(|e|match e {
        Expr::Collection{..}|Expr::Compiled(_)=>Some(false),
        Expr::Ref{path}=>Some(path.first().is_some_and(|s|["selectionCandidate","target","targets","namedTargets"].contains(&s.as_str()))),
        _=>None,
    })
}
fn bind_candidate(expr: &mut Expr, root_binding: bool) -> Result<(), String> {
    match expr {
        Expr::Ref { path } => {
            if path.first().is_some_and(|s| s == "selectionCandidate") {
                path[0] = "card".into();
            } else if path.first().is_some_and(|s| s == "card") {
                return Err("native selector cardCondition must use its declared selectionCandidate binding".into());
            } else if !root_binding && path.first().is_some_and(|s| s == "target") {
                return Err(
                    "named native selector cannot alias outer context.target to its candidate"
                        .into(),
                );
            }
        }
        Expr::Binary { left, right, .. } => {
            bind_candidate(left, root_binding)?;
            bind_candidate(right, root_binding)?;
        }
        Expr::Unary { value, .. } => bind_candidate(value, root_binding)?,
        Expr::Conditional {
            condition,
            then,
            r#else,
        } => {
            bind_candidate(condition, root_binding)?;
            bind_candidate(then, root_binding)?;
            bind_candidate(r#else, root_binding)?;
        }
        Expr::Array { items } | Expr::Math { args: items, .. } => {
            for item in items {
                bind_candidate(item, root_binding)?;
            }
        }
        Expr::Query {
            name,
            receiver,
            args,
        } => {
            bind_candidate(receiver, root_binding)?;
            if ["hasSomeArenaUnit", "arenaUnitCount", "hasSomeArenaCard"].contains(&name.as_str()) {
                if args.iter().any(captures_outer_candidate) {
                    return Err(
                        "native selector nested arena predicate requires outer-candidate capture"
                            .into(),
                    );
                }
            } else {
                for arg in args {
                    bind_candidate(arg, root_binding)?;
                }
            }
        }
        Expr::NativeTokenPredicate { receiver, .. } | Expr::ArenaLength { receiver } => {
            bind_candidate(receiver, root_binding)?
        }
        // Arena query predicates introduce their own `card` binder. Do not
        // rewrite or reject that nested lexical scope as a selector candidate.
        Expr::ArenaList {
            receiver,
            predicate,
        }
        | Expr::ArenaFilter {
            receiver,
            predicate,
        }
        | Expr::ArenaSome {
            receiver,
            predicate,
        } => {
            if captures_outer_candidate(predicate) {
                return Err(
                    "native selector nested arena predicate requires outer-candidate capture"
                        .into(),
                );
            }
            bind_candidate(receiver, root_binding)?;
        }
        Expr::Collection { .. }
        | Expr::HistoryQuery(_)
        | Expr::DataMember { .. }
        | Expr::HistoryAttackCheck { .. } => {
            return Err(
                "native selector requires a separately reviewed collection/history candidate scope"
                    .into(),
            );
        }
        Expr::DefeatCauseAmount {..} | Expr::Const { .. } => {}
        Expr::Compiled(_) => {
            return Err("native selector input cannot contain an executable binding".into())
        }
    }
    Ok(())
}
// Keep the exact old portable representation and defaults byte-for-byte for all
// already admitted programs. Native descriptors serialize as their typed source
// contract; their candidate-bound execution fields are derived, not hidden IR.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PortableResolver {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prompt_title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    prompt_title_parts: Vec<Expr>,
    #[serde(default = "any")]
    owner: String,
    #[serde(rename = "type", default = "any")]
    card_type: String,
    #[serde(default)]
    arena: Option<String>,
    #[serde(default)]
    zone: Option<String>,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    predicate: Option<Expr>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    count: Option<Expr>,
}
fn any() -> String {
    "any".into()
}
impl<'de> Deserialize<'de> for TargetResolver {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.get("kind").and_then(Value::as_str) == Some("playerSelection") {
            let native: crate::selected_player::NativePlayerSelection =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            return native.lift().map_err(serde::de::Error::custom);
        }
        if value.get("kind").is_some() {
            let native: NativeCardSelection =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            return native.lift().map_err(serde::de::Error::custom);
        }
        let p: PortableResolver =
            serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            prompt_title: p.prompt_title,
            prompt_title_parts: p.prompt_title_parts,
            owner: p.owner,
            card_type: p.card_type,
            arena: p.arena,
            zone: p.zone,
            optional: p.optional,
            predicate: p.predicate,
            mode: p.mode,
            count: p.count,
            native: None,
            player_selection: false,
        })
    }
}
impl Serialize for TargetResolver {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(native) = &self.native {
            return native.serialize(serializer);
        }
        if self.player_selection {
            return serde_json::json!({
                "kind":"playerSelection",
                "mode":"player",
                "chooser":"self",
                "resultBinding":"selectedPlayer",
                "defaults":"pinnedTargetResolver",
                "contextBinding":"target",
                "choosingPlayerOverride":"contextOverrideFirst",
                "delayOpponentBeforeCosts":true,
                "promptTitleParts":self.prompt_title_parts,
            })
            .serialize(serializer);
        }
        PortableResolver {
            prompt_title: self.prompt_title.clone(),
            prompt_title_parts: self.prompt_title_parts.clone(),
            owner: self.owner.clone(),
            card_type: self.card_type.clone(),
            arena: self.arena.clone(),
            zone: self.zone.clone(),
            optional: self.optional,
            predicate: self.predicate.clone(),
            mode: self.mode.clone(),
            count: self.count.clone(),
        }
        .serialize(serializer)
    }
}

pub fn native_selector_marker(value: &Value) -> bool {
    (value["kind"] == "cardSelection")
        || match value {
            Value::Object(o) => o.values().any(native_selector_marker),
            Value::Array(a) => a.iter().any(native_selector_marker),
            _ => false,
        }
}
pub fn expanded_selector_marker(value: &Value) -> bool {
    if value["kind"] == "playerSelection" || value["targetResolver"]["kind"] == "playerSelection" {
        return match value {
            Value::Object(o) => o.iter().any(|(k, v)| k != "targetResolver" && expanded_selector_marker(v)),
            Value::Array(a) => a.iter().any(expanded_selector_marker),
            _ => false,
        };
    }
    native_selector_marker(value)
        || value["targetResolver"]["type"].as_str().is_some_and(|s| {
            ![
                "unit",
                "base",
                "any",
                "nonLeaderUnit",
                "leaderUnit",
                "upgrade",
                "tokenUnit",
            ]
            .contains(&s)
        })
        || match value {
            Value::Object(o) => o.values().any(expanded_selector_marker),
            Value::Array(a) => a.iter().any(expanded_selector_marker),
            _ => false,
        }
}
pub fn native_source_entry(entry: &Value, sides: &super::Sides) -> bool {
    crate::admission::OrdinarySource::from_entry(entry).is_some()
        && ordinary_source_sides(sides)
        && entry["card"]["unique"].is_boolean()
        && entry["attachCondition"].is_null()
        && entry["piloting"].is_null()
        && crate::admission::SourceContextPolicy::Closed.accepts(entry)
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
}

#[cfg(test)]
#[path = "selector_contract_tests.rs"]
mod tests;
