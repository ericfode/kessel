//! Structural admission shared by the build-time and runtime loaders.
//!
//! These predicates describe envelopes, not executable capabilities. Callers
//! retain trigger, effect, resolver, provenance and family-specific flag checks.
use crate::ability_ir::Ability;
use serde_json::Value;

/// Exact ordinary source identity shared by selector and bounded-family gates.
/// This class/type pair does not grant a side, watcher, recipient or timing
/// capability; those contracts remain with each consuming family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrdinarySource {
    Unit,
    Event,
}

impl OrdinarySource {
    pub(crate) fn from_entry(entry: &Value) -> Option<Self> {
        match (
            entry["baseClass"].as_str(),
            entry["card"]["types"].as_array()?.as_slice(),
        ) {
            (Some("NonLeaderUnitCard"), [kind]) if kind == "unit" => Some(Self::Unit),
            (Some("EventCard"), [kind]) if kind == "event" => Some(Self::Event),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SourceContextPolicy {
    Closed,
    /// Only death-hook sources historically permit an empty object here.
    LegacyEmptyWatchers,
}

impl SourceContextPolicy {
    pub(crate) fn accepts(self, entry: &Value) -> bool {
        let watchers = &entry["stateWatchers"];
        closed_source_helpers(entry)
            && (watchers.is_null()
                || watchers.as_array().is_some_and(Vec::is_empty)
                || (matches!(self, Self::LegacyEmptyWatchers)
                    && watchers.as_object().is_some_and(serde_json::Map::is_empty)))
    }
}

/// Reviewed watcher families compose this with their exact declaration validator.
pub(crate) fn closed_source_helpers(entry: &Value) -> bool {
    [
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "privateClassConstants",
        "inheritedSource",
        "deploymentOverride",
        "setupValidationOverrides",
    ]
    .iter()
    .all(|key| entry[*key].is_null() || entry[*key].as_array().is_some_and(Vec::is_empty))
}

#[derive(Clone, Copy)]
pub(crate) enum Targeting {
    None,
    /// Resolver semantics remain with the effect-specific validator.
    OptionalSingle,
}

#[derive(Clone, Copy)]
pub(crate) enum HostContext {
    None,
    AttachedHost,
}

pub(crate) struct RegistrationShape {
    pub targeting: Targeting,
    pub host_context: HostContext,
}

impl RegistrationShape {
    /// Mandatory, unconditional, cost-free leaves with no named targeting or
    /// continuations. Internal attached_host and attacker_must_survive flags
    /// deliberately remain with callers: the existing families differ on them.
    pub(crate) fn accepts(&self, ability: &Ability) -> bool {
        !ability.optional
            && !ability.requires_confirmation
            && !ability.coordinate
            && ability.gained_host.is_none()
            && ability.costs.is_empty()
            && ability.context_title.is_empty()
            && ability.condition.is_none()
            && ability.target_resolvers.is_empty()
            && ability.then.is_none()
            && ability.if_you_do.is_none()
            && match self.targeting {
                Targeting::None => ability.target_resolver.is_none(),
                Targeting::OptionalSingle => true,
            }
            && match self.host_context {
                HostContext::None => ability.attached_context.is_none(),
                HostContext::AttachedHost => ability.attached_context.as_deref() == Some("host"),
            }
    }
}
