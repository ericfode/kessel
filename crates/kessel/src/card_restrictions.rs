//! Closed card restriction contracts and their serialized phase-mask vocabulary.
use crate::ability_ir::{Ability, Effect, Sides, Target};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestrictionVerb {
    BeAttacked,
    BeHealed,
    Attack,
    Ready,
    DealCombatDamage,
    DoesNotReadyDuringRegroup,
}
impl RestrictionVerb {
    pub const ALL: [Self; 6] = [
        Self::BeAttacked,
        Self::BeHealed,
        Self::Attack,
        Self::Ready,
        Self::DealCombatDamage,
        Self::DoesNotReadyDuringRegroup,
    ];
    pub fn bit(self) -> u8 {
        1 << match self {
            Self::BeAttacked => 0,
            Self::BeHealed => 1,
            Self::Attack => 2,
            Self::Ready => 3,
            Self::DealCombatDamage => 4,
            Self::DoesNotReadyDuringRegroup => 5,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::BeAttacked => "beAttacked",
            Self::BeHealed => "beHealed",
            Self::Attack => "attack",
            Self::Ready => "ready",
            Self::DealCombatDamage => "dealCombatDamage",
            Self::DoesNotReadyDuringRegroup => "doesNotReadyDuringRegroup",
        }
    }
    pub fn parse(name: &str) -> Option<Self> {
        serde_json::from_value(Value::String(name.to_owned())).ok()
    }
}

/// Inspect all serialized positions, so a restriction hidden in a continuation
/// cannot evade the whole-source boundary.
pub(crate) fn marker(value: &Value) -> bool {
    value["kind"] == "cardRestriction"
        || match value {
            Value::Object(fields) => fields.values().any(marker),
            Value::Array(items) => items.iter().any(marker),
            _ => false,
        }
}

fn ordinary_header(a: &Ability) -> bool {
    a.costs.is_empty()
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && !a.attached_host
        && !a.piloting_ability
        && !a.pilot_unit_ability
        && a.gained_host.is_none()
        && a.hooks.is_empty()
        && !a.attacker_must_survive
        && a.context_title.is_empty()
        && a.target_resolvers.is_empty()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && a.if_you_do_condition.is_none()
        && (a.trigger == "constant" || a.condition.is_none())
}

pub(crate) fn attached_constant(a: &Ability) -> bool {
    ordinary_header(a)
        && a.trigger == "constant"
        && a.attached_context.as_deref() == Some("upgrade")
        && a.condition.is_none()
        && a.target_resolver.is_none()
        && matches!(&a.effect, Effect::CardRestriction {
            target: Target::Name(name), duration, condition: None, ..
        } if name == "parent" && duration == "persistent")
}

/// Restriction support does not confer support on another source side, helper,
/// keyword, cost, trigger or granted ability. Program compilation independently
/// checks every effect and resolver after this source guard passes.
pub(crate) fn source_entry(entry: &Value, sides: &Sides) -> bool {
    if !marker(&entry["abilities"])
        || !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || !sides.leader.is_empty()
        || !sides.unit.is_empty()
        || !(1..=2).contains(&sides.card.len())
        || !entry["piloting"].is_null()
        || !entry["card"]["unique"].is_boolean()
        || [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
        ]
        .iter()
        .any(|key| !entry["card"][*key].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        || !entry["card"]["keywords"]
            .as_array()
            .is_some_and(|keywords| {
                keywords.iter().all(|keyword| {
                    keyword
                        .as_str()
                        .is_some_and(|name| crate::ability_ir::CORE_KEYWORDS.contains(&name))
                })
            })
        || !entry["keywordValues"].as_object().is_some_and(|values| {
            values.iter().all(|(name, value)| {
                crate::ability_ir::CORE_KEYWORDS.contains(&name.as_str())
                    && value.as_u64().is_some_and(|n| n <= u16::MAX as u64)
            })
        })
        || !sides.card.iter().all(ordinary_header)
    {
        return false;
    }
    let class = entry["baseClass"].as_str();
    let types = &entry["card"]["types"];
    let upgrade = class == Some("UpgradeCard") && *types == json!(["upgrade"]);
    let unit = class == Some("NonLeaderUnitCard") && *types == json!(["unit"]);
    let event = class == Some("EventCard") && *types == json!(["event"]);
    if !(upgrade || unit || event) {
        return false;
    }
    if upgrade {
        if !["upgradePower", "upgradeHp"].iter().all(|key| {
            entry["card"][*key]
                .as_i64()
                .is_some_and(|n| (-1000..=1000).contains(&n))
        }) {
            return false;
        }
        let attach = &entry["attachCondition"];
        if !attach.is_null()
            && *attach
                != json!({
                    "kind":"query", "name":"isNonLeaderUnit",
                    "receiver":{"kind":"ref","path":["card"]},"args":[]
                })
        {
            return false;
        }
    } else if !entry["attachCondition"].is_null()
        || !entry["card"]["upgradePower"].is_null()
        || !entry["card"]["upgradeHp"].is_null()
    {
        return false;
    }
    sides.card.iter().all(|a| {
        let trigger = if event {
            a.trigger == "event"
        } else if upgrade {
            a.trigger == "whenPlayed" || attached_constant(a)
        } else {
            ["constant", "whenPlayed", "whenDefeated", "onAttack"].contains(&a.trigger.as_str())
        };
        trigger && (a.attached_context.is_none() || (upgrade && attached_constant(a)))
    })
}

pub(crate) fn attached_upgrade_entry(entry: &Value, sides: &Sides) -> bool {
    entry["baseClass"] == "UpgradeCard"
        && source_entry(entry, sides)
        && sides.card.len() == 2
        && sides.card.iter().filter(|a| attached_constant(a)).count() == 1
}
