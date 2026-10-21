//! Deterministic perfect-information rules for the explicitly supported SWU subset.
mod attack_base_history_state;
mod attack_base_history_contract;
mod arena_payment_exhaust_contract;
mod modal_deck_movement_contract;
mod resource_number_experience_contract;

mod upgrade_return_shield_contract;
mod named_exhaust_contingent_contract;
mod captured_hand_cost_contract;
mod attack_lifetime_contract;
mod direct_unit_credit_contract;
mod player_phase_contract;
mod player_phase_state;
mod selected_hand_cost_contract;
mod round_limit_contract;
mod exhausted_live_count_contract;
mod defeat_token_creation_contract;
mod created_token_phase_keyword_contract;
mod generated_token_continuation_contract;
mod round_limit_state;
mod round_limit_runtime;
#[cfg(test)]
mod round_limit_tests;
mod collective_trigger_contract;
mod admission;
mod death_hooks;
mod regroup_hooks;
pub mod model;
mod seat;
pub mod search;
mod swu_search;
pub mod simulator;
pub use model::{Action, Seat, State};
pub use simulator::{Engine, RulesBackend};
pub mod ability_ir;
mod bounded_reductions;
pub mod static_ongoing;
pub mod phase_history;
mod live_power_maximum;
pub mod catalogue_io;
pub mod check;
pub mod validation;
pub mod cards;

fn program_backend(bytes:&[u8])->Result<usize,&'static str> {
    use sha2::{Digest,Sha256};
    simulator::program_backend(&format!("{:x}",Sha256::digest(bytes)),bytes)
}

fn expression_backend(bytes: &[u8]) -> Option<usize> {
    use sha2::{Digest, Sha256};
    simulator::expression_backend(&format!("{:x}", Sha256::digest(bytes)), bytes)
}

mod attack_history;

mod attached_registration;

mod piloting_attached;

mod native_attack;

mod advantage;

mod attached_constants;

mod deck_search;

mod selected_player;

mod discard_return;

mod view_cards;

mod pay_adjusted_resources;
mod resource_exhaust;

pub mod card_restrictions;

mod disclose;

mod self_play_cost;
mod self_play_cost_runtime;

mod modal_choices;
mod modal_arena_choices;

mod attachment_play_cost;

mod credits;
mod migration_credit_contract;
mod migration_look_contract;
mod resource_payment;
#[cfg(test)]
#[path = "credit_payment_tests.rs"]
mod credit_payment_tests;

mod current_played_unit;

mod migration_modal_contract;
mod migration_attack_contract;

mod migration_defeat_history_contract;
#[cfg(test)]
mod admission_tests;

mod migration_modal_deck_contract;
mod named_play_contract;
mod named_play;

mod cross_arena_contract;
mod recipient_heal_contract;

mod cinder_contract;
mod blade_base_damage_contract;
mod arena_defeat_attack_contract;
mod piloting_lifecycle;
mod unit_entry_contract;

mod variable_payment_contract;

#[path="defeat_cause_contract.rs"]
mod defeat_cause;

#[path="executor_upgrade_count_contract.rs"]
mod executor_upgrade_count;
