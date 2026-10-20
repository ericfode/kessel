//! Executor for the closed selected-number payment contract. No cost adjustment,
//! Credit menu, payment-result coercion, or card-ID selection is involved.
use super::*;
impl Engine {
    fn number_max(&self, s: &State) -> u16 {
        self.ready_resources(s, s.card(s.source).owner).min(6) as u16
    }
    pub(super) fn number_actions(&self, s: &State) -> Vec<Action> {
        (0..=self.number_max(s))
            .map(|n| Action {
                kind: "option".into(),
                card: None,
                arg: Some(Value::String(n.to_string())),
                method: None,
                label: format!("#{n}"),
            })
            .collect()
    }
    pub(super) fn apply_number_action(&self, s: &State, a: &Action) -> Result<State, String> {
        if s.ability_program == 0
            || !matches!(
                self.program(s.ability_program - 1)
                    .instructions
                    .get(s.ability_pc as usize),
                Some(Instruction::ChooseNumber)
            )
            || s.selected_number != 0
            || s.active != s.card(s.source).owner
        {
            return Err("invalid selected-number frame".into());
        }
        let text = a
            .arg
            .as_ref()
            .and_then(Value::as_str)
            .ok_or("number choice requires decimal string")?;
        let n = text.parse::<u8>().map_err(|_| "invalid number choice")?;
        if text != n.to_string() || u16::from(n) > self.number_max(s) {
            return Err("number choice outside live inclusive range".into());
        }
        if !self.number_actions(s).contains(a) {
            return Err("illegal number action".into());
        }
        let mut work = *s;
        work.selected_number = n + 1;
        work.ability_pc += 1;
        self.run_program(&mut work)?;
        Ok(work)
    }
    /// Returns true only while waiting for the mandatory number prompt.
    pub(super) fn run_number_instruction(&self, s: &mut State) -> Result<bool, String> {
        let instruction = self
            .program(s.ability_program - 1)
            .instructions
            .get(s.ability_pc as usize)
            .ok_or("number instruction missing")?
            .clone();
        if matches!(instruction, Instruction::ChooseNumber) {
            if !self.commit_program(s)? {
                return Ok(true);
            }
            if s.selected_number != 0 {
                return Err("number binding already initialized".into());
            }
            s.active = s.card(s.source).owner;
            if self.number_max(s) == 0 {
                s.selected_number = 1;
                s.ability_pc += 1;
                return Ok(false);
            }
            s.decision = Decision::AbilityNumber;
            return Ok(true);
        }
        let n = s
            .selected_number
            .checked_sub(1)
            .filter(|n| *n <= 6)
            .ok_or("selected number missing or invalid")?;
        let effect = match instruction {
            Instruction::PayUnadjustedSelectedResources => {
                if usize::from(n) > self.ready_resources(s, s.card(s.source).owner) {
                    return Err("unadjusted resource cost is unavailable".into());
                }
                Effect::PayResources {
                    target: Target::Name("ownPlayer".into()),
                    amount: Expr::number(i64::from(n)),
                }
            }
            Instruction::GiveSelectedExperience => Effect::GiveExperience {
                target: Target::Name("self".into()),
                amount: Expr::number(i64::from(n)),
            },
            _ => return Err("invalid number instruction".into()),
        };
        let (amount, resolved) = self.apply_effect(s, &effect)?;
        s.ability_result = amount;
        s.ability_succeeded = resolved;
        s.ability_pc += 1;
        Ok(false)
    }
}
