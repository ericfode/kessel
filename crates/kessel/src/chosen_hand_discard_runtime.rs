//! Chosen hand selection owns physical identities. The shared scalar mutation
//! owner moves that captured ordered list; its last event controls continuation.
use super::*;
use crate::ability_ir::chosen_hand_discard_contract as contract;
impl Engine {
    pub(super) fn chosen_hand_prompt(&self, s: &State, p: &Program) -> bool {
        s.ability_pc == 0
            && matches!(
                p.instructions.get(2),
                Some(Instruction::Apply(Effect::DiscardSpecificCard { .. }))
            )
            && matches!(p.instructions.first(),Some(Instruction::Select(r)) if contract::hand_selector(r))
    }
    pub(super) fn apply_chosen_hand_selection(
        &self,
        s: &State,
        a: &Action,
    ) -> Result<State, String> {
        if !self
            .ability_actions(s)
            .iter()
            .any(|legal| legal.key() == a.key())
        {
            return Err("illegal chosen hand action".into());
        }
        let p = self.program(s.ability_program - 1);
        let Instruction::Select(r) = &p.instructions[0] else {
            return Err("chosen hand selector missing".into());
        };
        let mut t = *s;
        if a.arg.as_ref().and_then(Value::as_str) == Some("passAbility") {
            if !p.optional {
                return Err("chosen hand ability is not optional".into());
            }
            self.end_program(&mut t)?;
            return Ok(t);
        }
        if a.arg.as_ref().and_then(Value::as_str) == Some("cancel") {
            if s.ability_committed || s.ability_return != ProgramContinuation::Finish.raw() {
                return Err("chosen hand selection cannot cancel".into());
            }
            self.cancel(&mut t);
            return Ok(t);
        }
        if let Some(id) = a.id() {
            if !self
                .program_targets(s, s.source, s.ability_program - 1, 0, r, false)
                .contains(&id)
            {
                return Err("illegal chosen hand card".into());
            }
            if r.is_multiple() {
                if !t.ability_selection.ids[..t.ability_selection.len as usize].contains(&id)
                    && t.ability_selection.len as usize >= r.max_targets()?
                {
                    return Err("chosen hand selection exceeds exact count".into());
                }
                t.ability_selection.toggle(id)?;
                return Ok(t);
            }
            t.ability_selection = Selection::one(id);
        } else {
            if !r.is_multiple() || a.arg.as_ref().and_then(Value::as_str) != Some("done") {
                return Err("chosen hand selection requires card or Done".into());
            }
            let count = t.ability_selection.len as usize;
            if count != 0 && (!r.is_multiple() || count != r.max_targets()?) {
                return Err("chosen hand Done requires zero or exact count".into());
            }
        }
        t.ability_target = t.ability_selection.ids[0];
        t.ability_pc += 1;
        self.run_program(&mut t)?;
        Ok(t)
    }
    pub(super) fn execute_chosen_hand_discard(
        &self,
        s: &mut State,
        e: &Effect,
    ) -> Result<(u16, bool), String> {
        let selection = s.ability_selection;
        if selection.len > 2 {
            return Err("chosen hand discard count exceeds reviewed bound".into());
        }
        let p = self.program(s.ability_program - 1);
        let Instruction::Select(r) = &p.instructions[0] else {
            return Err("chosen hand discard source selector missing".into());
        };
        if selection.len != 0 && selection.len as usize != r.max_targets()? {
            return Err("chosen hand discard requires exact selected cardinality".into());
        }
        if selection.ids[..selection.len as usize]
            .iter()
            .enumerate()
            .any(|(i, id)| selection.ids[..i].contains(id))
        {
            return Err("chosen hand discard contains duplicate identities".into());
        }
        let targets = selection.ids[..selection.len as usize]
            .iter()
            .copied()
            .map(Eval::Card)
            .collect::<Vec<_>>();
        let result = self.apply_scalar_effect(s, e, targets, 1, None)?;
        s.ability_selection = Selection::default();
        s.ability_target = 0;
        Ok(result)
    }
}
