//! Bounded self-only, unconditional static ongoing contracts.
//!
//! Target filters are deliberately absent: the pinned OngoingCardEffect defaults
//! to its source only in that case. No dynamic values or aura domains are admitted.
use crate::ability_ir::{Ability, Effect, Sides};
use serde::{Deserialize, Serialize};
use serde_json::Value;

macro_rules! literal {
    ($name:ident, $variant:ident, $value:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            #[serde(rename=$value)]
            $variant,
        }
    };
}
literal!(Persistent, Persistent, "persistent");
literal!(CardScope, Card, "card");
literal!(ConstantMatch, ConstantMatch, "constantMatch");
literal!(StaticValue, Static, "static");
literal!(SourceContext, Refresh, "refreshFromSourceController");
literal!(ScopeDefaults, Pinned, "pinnedOngoingCardOrPlayerEffect");
literal!(DependencyPolicy, Live, "liveFixedPointRequired");
literal!(AnyZone, Any, "any");
literal!(UndefinedKind, Undefined, "undefined");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelfEffect {
    CannotAttack,
    CannotAttackBase,
    EntersPlayReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Undefined {
    pub kind: UndefinedKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Properties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone_filter: Option<AnyZone>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Modifier {
    OngoingContract {
        scope: CardScope,
        #[serde(rename = "effectType")]
        effect_type: SelfEffect,
        target: ConstantMatch,
        duration: Persistent,
        #[serde(rename = "valueMode")]
        value_mode: StaticValue,
        value: Undefined,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        builder: Option<StaticValue>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Registration {
    pub properties: Properties,
    pub modifier: Modifier,
    pub duration: Persistent,
    pub context: SourceContext,
    pub scope_defaults: ScopeDefaults,
    pub dependency_policy: DependencyPolicy,
}
impl Registration {
    pub fn effect_type(&self) -> SelfEffect {
        let Modifier::OngoingContract { effect_type, .. } = &self.modifier;
        *effect_type
    }
    pub fn validate(&self) -> Result<(), String> {
        let Modifier::OngoingContract { builder, .. } = &self.modifier;
        let exact_scope = match self.effect_type() {
            SelfEffect::EntersPlayReady => {
                self.properties.source_zone_filter == Some(AnyZone::Any)
                    && *builder == Some(StaticValue::Static)
            }
            _ => self.properties.source_zone_filter.is_none() && builder.is_none(),
        };
        if exact_scope {
            Ok(())
        } else {
            Err("static self ongoing contract requires its reviewed source-zone and builder defaults".into())
        }
    }
}
pub(crate) fn marker(v: &Value) -> bool {
    v["kind"] == "constantEffectRegistration"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
pub(crate) fn registration(a: &Ability) -> bool {
    let Effect::ConstantEffectRegistration(r) = &a.effect else {
        return false;
    };
    a.trigger == "constant"
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && a.costs.is_empty()
        && a.context_title.is_empty()
        && a.target_resolver.is_none()
        && a.target_resolvers.is_empty()
        && a.condition.is_none()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && r.validate().is_ok()
}
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(sides) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == serde_json::json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && ["cost", "hp", "power"]
            .iter()
            .all(|f| e["card"][*f].as_u64().is_some_and(|n| n <= 1000))
        && e["card"]["hp"].as_u64().is_some_and(|n| n > 0)
        && ["ground", "space"].iter().any(|a| e["card"]["arena"] == *a)
        && sides.card.len() == 1
        && sides.unit.is_empty()
        && sides.leader.is_empty()
        && registration(&sides.card[0])
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
