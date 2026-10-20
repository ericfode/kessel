//! A live participant reads the flag from its own attack, including suspended death windows.
use super::*;
impl Engine {
    pub(super) fn is_attacking_using_ambush(&self, s: &State, id: CardId) -> bool {
        if !self.is_attacking(s, id) {
            return false;
        }
        // Do not take a flag from an unrelated nested attack or ability source.
        if s.attack_source != 0 && (s.attack_source == id || s.attack_defender == id) {
            return s.ambush;
        }
        s.death_frames[..s.death_depth as usize]
            .iter()
            .rev()
            .find(|f| {
                f.context.attack_source != 0
                    && (f.context.attack_source == id || f.context.attack_defender == id)
            })
            .is_some_and(|f| f.context.ambush)
    }
}
