// Included in generic_runtime so existing mutation and continuation ownership stays local.
impl Engine {
    /// Disclose commits the play before reading the hand, then suspends for the
    /// owner selection and opponent reveal acknowledgment. Only acknowledgment
    /// publishes success; cancellation and masked failure skip the continuation.
    /// Selection does not move cards or open child event windows, and is cleared
    /// before the ordinary continuation can select targets or resolve effects.
    fn disclose_missing(&self, s: &State, aspects: &[String], selected: &[CardId]) -> [u8; 6] {
        let mut missing = [0u8; 6];
        for name in aspects {
            missing[aspect(name).expect("validated disclose aspect")] += 1;
        }
        for &id in selected {
            for (i, count) in self.def(s, id).aspects.iter().enumerate() {
                missing[i] = missing[i].saturating_sub(*count);
            }
        }
        missing
    }
    fn can_disclose(&self, s: &State, player: Seat, aspects: &[String]) -> bool {
        self.disclose_missing(s, aspects, s.player(player).hand.as_slice())
            .iter()
            .all(|n| *n == 0)
    }
    fn disclose_aspects<'a>(&'a self, s: &State) -> &'a [String] {
        let Instruction::Apply(Effect::Disclose { aspects, .. }) =
            &self.program(s.ability_program - 1).instructions[s.ability_pc as usize]
        else {
            panic!("disclose prompt has no disclosure instruction")
        };
        aspects
    }
    fn disclose_complete(&self, s: &State) -> bool {
        self.disclose_missing(
            s,
            self.disclose_aspects(s),
            &s.disclose.selected.ids[..s.disclose.selected.len as usize],
        )
        .iter()
        .all(|n| *n == 0)
    }
    fn disclose_candidates(&self, s: &State) -> Vec<CardId> {
        let picks = &s.disclose.selected;
        let missing = self.disclose_missing(
            s,
            self.disclose_aspects(s),
            &picks.ids[..picks.len as usize],
        );
        s.player(s.disclose.player)
            .hand
            .as_slice()
            .iter()
            .copied()
            .filter(|id| {
                picks.ids[..picks.len as usize].contains(id)
                    || (!self.disclose_complete(s)
                        && self
                            .def(s, *id)
                            .aspects
                            .iter()
                            .enumerate()
                            .any(|(i, n)| *n > 0 && missing[i] > 0))
            })
            .collect()
    }
    fn finish_disclose(&self, s: &mut State, success: bool) -> Result<(), String> {
        s.active = s.disclose.player;
        s.disclose = DiscloseState::default();
        s.ability_result = u16::from(success);
        s.ability_succeeded = success;
        s.ability_pc += 1;
        self.run_program(s)
    }

    fn start_disclose(
        &self,
        s: &mut State,
        aspects: &[String],
        trigger: &str,
    ) -> Result<bool, String> {
        self.commit_program(s)?;
        s.ability_result = 0;
        s.ability_succeeded = false;
        let player = s.card(s.source).owner;
        s.disclose = DiscloseState {
            player,
            selected: Selection::default(),
        };
        s.active = player;
        if self.can_disclose(s, player, aspects) {
            s.decision = Decision::DiscloseSelect;
            return Ok(true);
        }
        if trigger != "event" {
            s.decision = Decision::DisclosePause;
            return Ok(true);
        }
        s.disclose = DiscloseState::default();
        s.ability_pc += 1;
        Ok(false)
    }
    fn disclose_actions(&self, s: &State) -> Vec<Action> {
        match s.decision {
            Decision::DisclosePause => vec![Action::button("Skip", "done")],
            Decision::DiscloseReveal => vec![Action::button("Done", "done")],
            Decision::DiscloseSelect => {
                let mut out = Vec::new();
                if s.disclose.selected.len == 0 {
                    out.push(Action::button("Choose nothing", "done"));
                } else if s.disclose.selected.len > 1 && self.disclose_complete(s) {
                    out.push(Action::button("Done", "done"));
                }
                out.push(Action::button("Cancel", "cancel"));
                for id in self.disclose_candidates(s) {
                    out.push(self.card_action(s, id));
                }
                out
            }
            _ => Vec::new(),
        }
    }
    fn apply_disclose_action(&self, s: &State, a: &Action) -> Result<State, String> {
        if !self.disclose_actions(s).contains(a) {
            return Err("invalid Disclose action".into());
        }
        let mut t = *s;
        match s.decision {
            Decision::DisclosePause => self.finish_disclose(&mut t, false)?,
            Decision::DiscloseReveal => self.finish_disclose(&mut t, true)?,
            Decision::DiscloseSelect => {
                if a.arg.as_ref().and_then(Value::as_str) == Some("cancel") {
                    self.finish_disclose(&mut t, false)?;
                } else if a.arg.as_ref().and_then(Value::as_str) == Some("done") {
                    if s.disclose.selected.len == 0 {
                        self.finish_disclose(&mut t, false)?;
                    } else {
                        if !self.disclose_complete(s) {
                            return Err("disclose requirements incomplete".into());
                        }
                        t.active = t.disclose.player.other();
                        t.decision = Decision::DiscloseReveal;
                    }
                } else {
                    let id = a.id().ok_or("disclose selection card missing")?;
                    t.disclose.selected.toggle(id)?;
                    if t.disclose.selected.len == 1 && self.disclose_complete(&t) {
                        t.active = t.disclose.player.other();
                        t.decision = Decision::DiscloseReveal;
                    }
                }
            }
            _ => return Err("not a Disclose decision".into()),
        }
        Ok(t)
    }
}
