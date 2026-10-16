//! Compatibility actions for strict pools loaded without a catalogue. Compiled
//! pools always use admitted programs, even when a leader has the same set code.
//! The fallback action for other historical strict-pool leaders is intentional.
use super::*;

impl Engine {
    pub(super) fn legacy_leader_available(&self, s: &State) -> bool {
        let id = s.player(s.active).leader;
        let c = s.card(id);
        !c.deployed
            && !c.exhausted
            && (self.def(s,id).code!="IBH_053" || self.ready_resources(s,s.active)>0)
    }
    pub(super) fn legacy_leader_title(&self, s: &State) -> &'static str {
        match self.def(s,s.player(s.active).leader).code.as_str() {
            "SOR_014"=>"Deal 1 damage to each base",
            "IBH_053"=>"Deal 1 damage to a base",
            _=>"Heal up to 2 damage from a unit. If you do, deal that much damage to your base",
        }
    }
    pub(super) fn use_legacy_leader(&self, s: &mut State) -> Result<(), String> {
        let id = s.player(s.active).leader;
        s.source = id;
        s.set_trigger_context(TriggerContext::Action);
        match self.def(s,id).code.as_str() {
            "SOR_014" => {
                s.card_mut(id).exhausted = true;
                for seat in [Seat::P1, Seat::P2] {
                    let base = s.player(seat).base;
                    s.card_mut(base).damage += 1;
                }
                self.check_win(s);
                self.finish(s)?;
            }
            "IBH_053" => s.decision = Decision::VaderBase,
            _ => {
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
