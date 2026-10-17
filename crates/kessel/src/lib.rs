//! Deterministic perfect-information rules for the explicitly supported SWU subset.
mod death_hooks;
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

pub mod card_restrictions;

mod disclose;
