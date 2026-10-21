//! Direct-only installers and captured player-owned phase consumers.
use super::*;
use crate::ability_ir::{Effect, Instruction};
use crate::player_phase_contract::Kind;
use crate::player_phase_state::{Mask, Row};
impl Engine {
    pub(super) fn install_player_phase(&self, s: &mut State, kind: Kind) -> Result<(), String> {
        let program = s
            .ability_program
            .checked_sub(1)
            .ok_or("phase installer missing captured program")?;
        let p = self.program(program);
        // Physical identity remains allocated after source departure. Never require
        // live aura membership for an already captured mandatory own trigger.
        if s.source == 0
            || !self.def(s, s.source).programs.card.contains(&program)
            || s.active != s.card(s.source).owner
            || !matches!(p.instructions.as_slice(),[Instruction::Apply(Effect::InstallPlayerPhase{phase_kind:k}),Instruction::EndStep] if *k==kind)
            || !matches!(
                (kind, p.trigger.as_str()),
                (Kind::ImperialDiscount, "whenPlayed")
                    | (Kind::LowPrintedPowerReady, "whenPlayed" | "onAttack")
            )
        {
            return Err("phase installer lacks captured direct own-source origin".into());
        }
        let origin = *self
            .def(s, s.source)
            .programs
            .card
            .first()
            .ok_or("phase source registration missing")?;
        s.player_phase.append(Row::new(
            s.source,
            origin,
            kind,
            s.active.index() as u8,
            s.active.index() as u8,
            0,
        )?)?;
        Ok(())
    }
    pub(super) fn phase_cost_capture(&self, s: &State, id: CardId) -> Mask {
        if s.player_phase.len == 0 {
            return Mask::default();
        }
        if self.def(s, id).info.is("unit") && self.card_has_trait(s, id, "imperial") {
            s.player_phase
                .capture(Kind::ImperialDiscount, s.card(id).owner.index() as u8)
        } else {
            Mask::default()
        }
    }
    pub(super) fn phase_ready_at_construction(
        &self,
        s: &mut State,
        id: CardId,
    ) -> Result<bool, String> {
        if s.player_phase.len == 0 {
            return Ok(false);
        }
        if !self.def(s, id).info.is("unit") || self.def(s, id).info.power.unwrap_or(0) > 1 {
            return Ok(false);
        }
        let mask = s
            .player_phase
            .capture(Kind::LowPrintedPowerReady, s.card(id).owner.index() as u8);
        s.player_phase.consume(mask)?;
        Ok(mask.count() != 0)
    }
    pub(super) fn phase_direct_required(&self, s: &State, id: CardId, trigger: &str) -> usize {
        self.def(s, id)
            .programs
            .card
            .iter()
            .filter(|&&p| {
                self.program(p).trigger == trigger
                    && self
                        .program(p)
                        .instructions
                        .iter()
                        .any(|i| matches!(i, Instruction::Apply(Effect::InstallPlayerPhase { .. })))
            })
            .count()
    }
    pub(super) fn phase_preflight_parent(
        &self,
        s: &State,
        p: &crate::ability_ir::Program,
    ) -> Result<(), String> {
        // The whole-program closure caps one parent's attack instructions at 64;
        // each exact direct source contributes at most one installer per attack.
        if s.player_phase.len == 0 {
            return Ok(());
        }
        let attacks = crate::player_phase_contract::program_operations(p)?.attacks;
        if attacks != 0 {
            let required = (1..s.next_id)
                .filter(|id| self.in_arena(s, *id))
                .map(|id| self.phase_direct_required(s, id, "onAttack"))
                .max()
                .unwrap_or(0);
            s.player_phase.preflight(
                required
                    .checked_mul(attacks)
                    .ok_or("phase attack bound overflow")?,
            )?;
        }
        Ok(())
    }

    pub(super) fn phase_preflight_attack(
        &self,
        s: &State,
        attacker: CardId,
        defender: CardId,
    ) -> Result<(), String> {
        s.player_phase
            .preflight(self.phase_direct_required(s, attacker, "onAttack"))?;
        for grant in self
            .gained_attack_grants(s, attacker)
            .into_iter()
            .chain(self.attached_host_grants(s, defender, "onDefense"))
        {
            crate::player_phase_contract::copied_program_allowed(self.program(grant.program))?;
        }
        Ok(())
    }

    pub(super) fn phase_source(&self, s: &State, id: CardId) -> bool {
        self.def(s, id).programs.card.iter().any(|&p| {
            self.program(p)
                .instructions
                .iter()
                .any(|i| matches!(i, Instruction::Apply(Effect::InstallPlayerPhase { .. })))
        })
    }
    /// Pinned InPlayCard uniqueness compares title/subtitle/controller and excludes
    /// nonunique clones. Apply this symmetric identity rule to pairs involving a
    /// validated phase producer; ordinary-family behavior stays on its old key.
    pub(super) fn phase_unique_identity(&self, s: &State, a: CardId, b: CardId) -> bool {
        if !self.phase_source(s, a) && !self.phase_source(s, b) {
            return s.card(a).def == s.card(b).def;
        }
        let da = self.def(s, a);
        let db = self.def(s, b);
        da.info.unique
            && db.info.unique
            && da.info.title == db.info.title
            && da.unique_subtitle == db.unique_subtitle
    }

    pub(super) fn phase_unique_candidates(&self, s: &State, source: CardId) -> Option<Vec<CardId>> {
        let live = |id| self.in_arena(s, id) || self.attached_in_play(s, id);
        let involved = self.phase_source(s, source)
            || (1..s.next_id).any(|id| {
                live(id)
                    && s.card(id).owner == s.card(source).owner
                    && self.phase_source(s, id)
                    && self.phase_unique_identity(s, id, source)
            });
        if !involved {
            return None;
        }
        let mut out = vec![];
        // Engine AllArenasZone visits ground then space, retaining physical entry
        // order inside each zone. Tokens cannot be unique in this closed source set.
        for space in [false, true] {
            for &physical in s.registration_order.as_slice() {
                let id = u16::from(physical);
                if s.card(id).owner != s.card(source).owner
                    || !live(id)
                    || !self.phase_unique_identity(s, id, source)
                {
                    continue;
                }
                let host = if self.attached_in_play(s, id) {
                    s.card(id).parent
                } else {
                    id
                };
                if (self.def(s, host).info.arena.as_deref() == Some("space")) == space {
                    out.push(id);
                }
            }
        }
        Some(out)
    }
}
