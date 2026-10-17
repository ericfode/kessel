//! Until-end-of-attack stat modifications (pinned `CardAttackLastingEffectSystem`
//! / `Duration.UntilEndOfAttack`).
//!
//! The engine's helper creates one `OngoingCardEffect` per targeted unit whose
//! `canAffectInternal` accepts units only. `AttackFlow.cleanUpAttack` calls
//! `OngoingEffectEngine.unregisterOnAttackEffects()`, which removes every
//! effect of that duration in one global sweep after the OnAttackEnd window;
//! a card that leaves play loses its own ongoing effect immediately. This store
//! is exactly that lifecycle and is separate from phase stats (`for this phase`)
//! because the two expire at different printed durations:
//! `card.phase_power/phase_hp` clear at phase end, these grants clear at attack
//! end. The applied values are the deltas resolved when the ability resolved;
//! live totals are composed in `raw_power`/`raw_hp` like every other modifier.
use super::*;
use crate::model::{AttackStatGrant, AttackStats, CardId, MAX_ATTACK_STAT_GRANTS, State};

impl Engine {
    pub(super) fn attack_stat_bonus(&self, s: &State, id: CardId) -> (i64, i64) {
        let mut power = 0i64;
        let mut hp = 0i64;
        for grant in &s.attack_stats.grants[..s.attack_stats.len as usize] {
            if grant.target == id {
                power += grant.power as i64;
                hp += grant.hp as i64;
            }
        }
        (power, hp)
    }
    /// Append one resolved stat delta. The pinned effect is unit-only, so a
    /// non-arena target is not an error: the caller only reaches this for
    /// in-play units (bases are filtered by the lasting effect's canAffect).
    pub(super) fn grant_attack_stats(
        &self,
        s: &mut State,
        target: CardId,
        power: i64,
        hp: i64,
    ) -> Result<(), String> {
        if target == 0 || !self.in_arena(s, target) {
            return Err("attack stats require an in-play unit target".into());
        }
        let power = i16::try_from(power)
            .map_err(|_| "attack stat power exceeds bounded capacity".to_string())?;
        let hp =
            i16::try_from(hp).map_err(|_| "attack stat HP exceeds bounded capacity".to_string())?;
        if s.attack_stats.len as usize == MAX_ATTACK_STAT_GRANTS {
            return Err("attack-duration lasting-effect capacity exceeded (32)".into());
        }
        let mut work = *s;
        work.attack_stats.grants[work.attack_stats.len as usize] = AttackStatGrant {
            target,
            power,
            hp,
        };
        work.attack_stats.len += 1;
        // Same compact-stat invariant as phase modifiers: a grant that would
        // push a live total past the bounded capacity fails before mutating.
        self.check_stat_capacity(&work)?;
        *s = work;
        Ok(())
    }
    /// Ongoing card effects are cancelled when their target leaves play, even
    /// though other grants of the same attack duration remain registered.
    pub(super) fn remove_attack_stats_target(&self, s: &mut State, target: CardId) {
        let mut len = 0;
        for i in 0..s.attack_stats.len as usize {
            if s.attack_stats.grants[i].target != target {
                s.attack_stats.grants[len] = s.attack_stats.grants[i];
                len += 1;
            }
        }
        s.attack_stats.grants[len..].fill(AttackStatGrant::default());
        s.attack_stats.len = len as u8;
    }
    /// One global sweep, like `unregisterOnAttackEffects`: nested attack cleanup
    /// removes the outer attack's grants too, so a single store is sufficient.
    pub(super) fn clear_attack_stats(&self, s: &mut State) {
        s.attack_stats = AttackStats::default();
    }
}
