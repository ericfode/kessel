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

/// Shared mandatory leaf fields; trigger, targeting and host binding stay with
/// each consumer instead of being encoded in a separate policy object.
pub(crate) fn plain_registration(a: &Ability) -> bool {
    !a.optional && !a.requires_confirmation && !a.coordinate && a.gained_host.is_none()
        && a.costs.is_empty() && a.context_title.is_empty() && a.condition.is_none()
        && a.target_resolvers.is_empty() && a.then.is_none() && a.if_you_do.is_none()
}
