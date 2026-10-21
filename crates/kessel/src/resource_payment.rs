//! Prepared resource-cost state shared by card, ability and effect payments.
//! A pause retains physical source/host identity and the already evaluated cost.
use crate::model::{CardId, Seat};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaymentPurpose {
    #[default]
    None,
    UnitPlay,
    UpgradePlay,
    ProgramPlay,
    AbilityCost,
    Effect,
    CostWindow,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaymentStage {
    #[default]
    None,
    Credits,
    Amount,
    Paid,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePayment {
    pub(crate) phase_adjusters:crate::player_phase_state::Mask,
    pub purpose: PaymentPurpose,
    pub stage: PaymentStage,
    pub payer: Seat,
    pub actor: Seat,
    pub source: CardId,
    pub target: CardId,
    pub amount: u16,
    pub from_resources: bool,
    pub can_cancel: bool,
    pub program: u16,
    pub pc: u8,
    pub cost_index: u16,
    pub minimum: u16,
    pub maximum: u16,
    pub credits_used: u16,
    pub end_program: bool,
    pub group_saved: bool,
}
impl ResourcePayment {
    pub(crate) fn active(&self) -> bool {
        self.purpose != PaymentPurpose::None
    }
    pub(crate) fn same_boundary(&self, other: &Self) -> bool {
        self.purpose == other.purpose
            && self.payer == other.payer
            && self.source == other.source
            && self.target == other.target
            && self.from_resources == other.from_resources
            && self.program == other.program
            && self.pc == other.pc
            && self.cost_index == other.cost_index
    }
    pub(crate) fn hash_values(&self, put: &mut impl FnMut(u64)) {
        if !self.active() {
            debug_assert_eq!(*self, Self::default(), "inactive payment must be cleared");
            return;
        }
        put(0x5041594d454e54);
        if self.phase_adjusters.count()!=0 {put(0x50484153454d);put(self.phase_adjusters.0[0]as u64);put(self.phase_adjusters.0[1]as u64);}
        for value in [
            self.purpose as u64,
            self.stage as u64,
            self.payer.index() as u64,
            self.actor.index() as u64,
            self.source as u64,
            self.target as u64,
            self.amount as u64,
            self.from_resources as u64,
            self.can_cancel as u64,
            self.program as u64,
            self.pc as u64,
            self.cost_index as u64,
            self.minimum as u64,
            self.maximum as u64,
            self.credits_used as u64,
            self.end_program as u64,
            self.group_saved as u64,
        ] {
            put(value);
        }
    }
}
