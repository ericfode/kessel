//! Pure, complete source proof for the optional ground-unit owner deck choice.
//! No normalization or executable placeholder is returned: admission must install
//! the complete target-preserving deck executor before consuming this contract.
use crate::migration_modal_contract::{self, Chooser, DependentChoice, Resolution};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Prepared {
    pub registration: Value,
    pub descriptor: DependentChoice,
}

pub(crate) fn prepare(entry: &Value) -> Result<Option<Prepared>, String> {
    // Leave ordinary registrations and root menus to their own closed adapters.
    let raw = &entry["abilities"]["card"][0];
    if raw["trigger"] != "whenDefeated" || raw["targetResolvers"].is_null() {
        return Ok(None);
    }
    let Some(descriptor) = migration_modal_contract::inspect(entry)? else {
        return Ok(None);
    };
    if descriptor.trigger != "whenDefeated"
        || descriptor.target_name != "unit"
        || descriptor.choice_name != "deck"
        || descriptor.card_type != "nonLeaderUnit"
        || descriptor.zone != "groundArena"
        || descriptor.controller != "any"
        || !descriptor.optional_target
        || descriptor.chooser != Chooser::SelectedOwner
        || descriptor.resolution != Resolution::OwnerDeckPlacement
    {
        return Ok(None);
    }
    Ok(Some(Prepared {
        registration: descriptor.registration.clone(),
        descriptor,
    }))
}
