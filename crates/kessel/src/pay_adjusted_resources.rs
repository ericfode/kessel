//! Bounded `payResources` card effects, separate from unadjusted resource payments.
//!
//! The pinned CardEffectResourcePaymentSystem allows cost adjustments and cannot
//! cancel once payment starts. The admitted pool has no Credit token or other
//! card-effect cost adjusters, so its whole constant charge is paid in resources.
//! Keep this contract distinct from `payResourcesWithoutAdjustment`, whose
//! partial payment and result semantics are different.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target};
use serde_json::{json, Value};

/// Validate the compiler's closed payment contract before using the existing
/// resource-exhaustion primitive. Affordability must be checked before that
/// primitive runs: it can otherwise exhaust a partial payment.
pub(crate) fn amount(effect: &Effect) -> Result<u16, String> {
    let Effect::PayAdjustedResources {
        target,
        amount,
        payment_type,
        is_cost,
        can_cancel_during_payment,
        allow_cost_adjustments,
    } = effect
    else {
        return Err("not a card-effect resource payment".into());
    };
    if payment_type != "cardEffectPayment" {
        return Err(format!(
            "unsupported payAdjustedResources paymentType {payment_type}"
        ));
    }
    if *is_cost {
        return Err("payAdjustedResources ability-cost flag is a different payment".into());
    }
    if *can_cancel_during_payment {
        return Err(
            "payAdjustedResources cancellation during payment needs a payment prompt".into(),
        );
    }
    if !*allow_cost_adjustments {
        return Err(
            "payAdjustedResources without adjustment permission is a different payment".into(),
        );
    }
    if !matches!(target, Target::Name(name) if name == "ownPlayer") {
        return Err("payAdjustedResources only pays the source controller".into());
    }
    match amount.original() {
        Expr::Const { value } => value
            .as_u64()
            .filter(|n| (1..=u16::MAX as u64).contains(n))
            .map(|n| n as u16)
            .ok_or_else(|| "payAdjustedResources amount must be a constant from 1 to 65535".into()),
        _ => Err("payAdjustedResources requires a constant amount".into()),
    }
}

pub(crate) fn marker(value: &Value) -> bool {
    value["kind"] == "payAdjustedResources"
        || match value {
            Value::Array(values) => values.iter().any(marker),
            Value::Object(fields) => fields.values().any(marker),
            _ => false,
        }
}

/// These payment registrations have only been reviewed on ordinary nonleader
/// units. Validate metadata together with source class; all companion abilities
/// still pass through ordinary complete-card deserialization and compilation.
pub(crate) fn validate_entry(entry: &Value) -> Result<(), String> {
    if !marker(&entry["abilities"]) {
        return Ok(());
    }
    fn ordinary_context(ability: &Ability) -> bool {
        !ability.attached_host
            && !ability.piloting_ability
            && !ability.pilot_unit_ability
            && !ability.attacker_must_survive
            && ability.hooks.is_empty()
            && ability.gained_host.is_none()
            && ability.attached_context.is_none()
            && ability.then.as_deref().is_none_or(ordinary_context)
            && ability.if_you_do.as_deref().is_none_or(ordinary_context)
    }
    let ordinary_sides =
        serde_json::from_value::<Sides>(entry["abilities"].clone()).is_ok_and(|sides| {
            sides.leader.is_empty()
                && sides.unit.is_empty()
                && !sides.card.is_empty()
                && sides.card.len() <= 8
                && sides.card.iter().all(ordinary_context)
        });
    if !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || entry["baseClass"] != "NonLeaderUnitCard"
        || entry["card"]["types"] != json!(["unit"])
        || !matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
        || !entry["card"]["unique"].is_boolean()
        || !["cost", "hp", "power"]
            .iter()
            .all(|field| entry["card"][*field].as_u64().is_some_and(|n| n <= 1000))
        || !entry["card"]["hp"].as_u64().is_some_and(|n| n > 0)
        || !ordinary_sides
        || !entry["piloting"].is_null()
        || !entry["attachCondition"].is_null()
        || ![
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|field| entry["card"][*field].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
    {
        return Err(
            "card-effect resource payment requires ordinary nonleader unit source metadata".into(),
        );
    }
    Ok(())
}
