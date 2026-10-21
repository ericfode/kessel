//! Continuation consumes only its scoped resolved creation identity. Entry and
//! framework state checks already ran; deferred Shielded registrations have not.
use super::*;
impl Engine {
    pub(super) fn run_created_token_advantage(&self, s: &mut State) -> Result<bool, String> {
        let Some(index) = s.ability_program.checked_sub(1) else {
            return Ok(false);
        };
        let p = self.program(index);
        let Some(Instruction::GrantCreatedTokenAdvantage { creation_pc }) =
            p.instructions.get(s.ability_pc as usize)
        else {
            return Ok(false);
        };
        if !crate::generated_token_continuation_contract::program(p)
            || !s
                .event_result
                .belongs_to(s.source, s.ability_program, *creation_pc)
        {
            return Err("generated token continuation result scope mismatch".into());
        }
        let id = s.event_result.generated_token;
        if id == 0 || id >= s.next_id || self.def(s, id).info.internal_name != "mandalorian" {
            return Err("resolved generated singleton identity missing".into());
        }
        if self.in_arena(s, id) {
            self.give_captured_advantage(s, vec![id], 1)?;
        }
        s.ability_pc += 1;
        Ok(true)
    }
    // Probe the SAME prepared creation/framework and captured attachment owners
    // before payment can publish a resource prompt. The copy is never published;
    // IDs remain allocated by the real creation, Advantage and Shield windows.
    pub(super) fn preflight_generated_token_branch(&self, s: &State) -> Result<(), String> {
        let Some(index) = s.ability_program.checked_sub(1) else {
            return Ok(());
        };
        let p = self.program(index);
        if !crate::generated_token_continuation_contract::requires(p) {
            return Ok(());
        }
        if !crate::generated_token_continuation_contract::program(p) {
            return Err("generated token program differs from complete source".into());
        }
        if s.ability_pc != 5 {
            return Ok(());
        }
        let Instruction::Conditional { condition, .. } = &p.instructions[6] else {
            unreachable!()
        };
        if !self.eval(s, s.source, 0, condition).truth() {
            return Ok(());
        }
        let Instruction::Group(effect) = &p.instructions[7] else {
            unreachable!()
        };
        let mut probe = *s;
        probe.ability_pc = 7;
        probe.event_group = EventGroupState::default();
        probe.event_group.active = true;
        self.generate_group(&mut probe, effect)?;
        if !self.resolve_group(&mut probe, effect)? {
            return Err("generated token preflight unexpectedly suspended".into());
        }
        let id = probe.event_result.generated_token;
        if id == 0 || id >= probe.next_id {
            return Err("generated token preflight lacks singleton".into());
        }
        if self.in_arena(&probe, id) {
            self.give_captured_advantage(&mut probe, vec![id], 1)?;
        }
        let shields = probe.death_pending.entries[..probe.death_pending.len as usize]
            .iter()
            .filter(|e| e.keyword == T_SHIELD && self.in_arena(&probe, e.source))
            .count();
        if (probe.next_id as usize)
            .checked_add(shields)
            .is_none_or(|n| n > MAX_CARDS)
        {
            return Err("card capacity exceeded by generated token continuation".into());
        }
        Ok(())
    }
    pub(crate) fn generated_token_continuation_pending(&self, s: &State) -> bool {
        let required =
            |p: u16| crate::generated_token_continuation_contract::requires(self.program(p));
        (s.ability_program != 0 && required(s.ability_program - 1))
            || (s.event_result.program != 0 && required(s.event_result.program - 1))
            || [&s.death_pending, &s.death_window]
                .into_iter()
                .chain(
                    s.death_frames[..s.death_depth as usize]
                        .iter()
                        .map(|f| &f.window),
                )
                .any(|w| {
                    w.entries[..w.len as usize]
                        .iter()
                        .any(|e| e.keyword == 0 && required(e.program))
                })
            || (s.support.context.event_result.program != 0
                && required(s.support.context.event_result.program - 1))
            || (s.support.context.ability_program != 0
                && required(s.support.context.ability_program - 1))
            || s.death_frames[..s.death_depth as usize].iter().any(|f| {
                (f.context.ability_program != 0 && required(f.context.ability_program - 1))
                    || (f.context.event_result.program != 0
                        && required(f.context.event_result.program - 1))
            })
    }
}
