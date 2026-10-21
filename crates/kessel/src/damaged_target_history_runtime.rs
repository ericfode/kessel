//! Physical target history survives movement/healing; only phase end clears it.
use super::*;
impl Engine {
    pub(super) fn state_needs_damaged_target_history(&self, s: &State) -> bool {
        self.catalogue.is_some()
            && (1..s.next_id).any(|id| {
                (s.card(id).present || s.card(id).def != 0) && {
                    let p = &self.def(s, id).programs;
                    p.card
                        .iter()
                        .chain(&p.unit)
                        .chain(&p.leader)
                        .any(|p| crate::damaged_target_history_contract::requires(self.program(*p)))
                }
            })
    }
    pub(super) fn record_damaged_target(&self, s: &mut State, id: CardId) {
        // Unknown old histories cannot become exact by tracking a suffix.
        if s.phase_history.damage_exact {
            s.phase_history.damaged[id as usize / 64] |= 1u64 << (id as usize % 64);
        }
    }
}
