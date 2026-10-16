//! Compatibility actions for strict pools loaded without a catalogue. Compiled
//! pools always use admitted programs, even when a leader has the same set code.
//! The fallback action for other historical strict-pool leaders is intentional.
use super::*;

#[derive(Clone, Copy)]
enum LegacyLeaderAction {
    BothBases,
    PaidBase,
    HealThenOwnBase,
}
impl LegacyLeaderAction {
    fn for_code(code: &str) -> Self {
        match code {
            "SOR_014" => Self::BothBases,
            "IBH_053" => Self::PaidBase,
            _ => Self::HealThenOwnBase,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::BothBases => "Deal 1 damage to each base",
            Self::PaidBase => "Deal 1 damage to a base",
            Self::HealThenOwnBase => {
                "Heal up to 2 damage from a unit. If you do, deal that much damage to your base"
            }
        }
    }
}
impl Engine {
    pub(super) fn legacy_leader_available(&self, s: &State) -> bool {
        let id = s.player(s.active).leader;
        let c = s.card(id);
        !c.deployed
            && !c.exhausted
            && match LegacyLeaderAction::for_code(&self.def(s, id).code) {
                LegacyLeaderAction::PaidBase => self.ready_resources(s, s.active) > 0,
                _ => true,
            }
    }
    pub(super) fn legacy_leader_title(&self, s: &State) -> &'static str {
        LegacyLeaderAction::for_code(&self.def(s, s.player(s.active).leader).code).title()
    }
    pub(super) fn use_legacy_leader(&self, s: &mut State) -> Result<(), String> {
        let id = s.player(s.active).leader;
        s.source = id;
        s.set_trigger_context(TriggerContext::Action);
        match LegacyLeaderAction::for_code(&self.def(s, id).code) {
            LegacyLeaderAction::BothBases => {
                s.card_mut(id).exhausted = true;
                for seat in [Seat::P1, Seat::P2] {
                    let base = s.player(seat).base;
                    s.card_mut(base).damage += 1;
                }
                self.check_win(s);
                self.finish(s)?;
            }
            LegacyLeaderAction::PaidBase => s.decision = Decision::VaderBase,
            LegacyLeaderAction::HealThenOwnBase => {
                s.decision = if s.players.iter().all(|p| p.ground.len + p.space.len == 0) {
                    Decision::NoEffect
                } else {
                    Decision::SatineHeal
                };
            }
        }
        Ok(())
    }
}
