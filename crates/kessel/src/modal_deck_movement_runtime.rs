//! One display boundary: the property factory captures a physical top card.
//! Deck choice precedes this capture; the next defeated source queries afresh.
//! Discard uses the existing prepared mutation owner, not a second zone mutator.
use super::*;
impl Engine {
    fn top_deck_side(&self, s: &State) -> Option<Seat> {
        if s.ability_program == 0 {
            return None;
        }
        match self
            .program(s.ability_program - 1)
            .instructions
            .get(s.ability_pc as usize)
        {
            Some(Instruction::LookTopDeckOptions { opponent }) => Some(if *opponent {
                s.card(s.source).owner.other()
            } else {
                s.card(s.source).owner
            }),
            _ => None,
        }
    }
    pub(super) fn top_deck_options_prompt(&self, s: &State) -> bool {
        s.decision == Decision::AbilityDisplay && self.top_deck_side(s).is_some()
    }
    pub(super) fn run_top_deck_options(&self, s: &mut State) -> Result<bool, String> {
        let side = self.top_deck_side(s).ok_or("top-deck instruction absent")?;
        if !crate::modal_deck_movement_contract::program(self.program(s.ability_program - 1)) {
            return Err("unproved top-deck programme".into());
        }
        let top = s.player(side).deck.as_slice().first().copied().unwrap_or(0);
        if top == 0 {
            s.ability_target = 0;
            s.ability_pc += 1;
            return Ok(false);
        }
        s.ability_target = top;
        s.active = s.card(s.source).owner;
        s.decision = Decision::AbilityDisplay;
        Ok(true)
    }
    pub(super) fn top_deck_options_actions(&self, s: &State) -> Vec<Action> {
        let id = s.ability_target;
        if id == 0 {
            return vec![];
        }
        [("Discard it", "discard"), ("Leave it on top", "leave")]
            .into_iter()
            .map(|(label, arg)| Action {
                kind: "perCardButton".into(),
                card: self.card_action(s, id).card,
                arg: Some(json!(arg)),
                method: None,
                label: format!("{label}:{}", self.def(s, id).info.internal_name),
            })
            .collect()
    }
    pub(super) fn apply_top_deck_options(&self, s: &State, a: &Action) -> Result<State, String> {
        if !self.top_deck_options_actions(s).contains(a) {
            return Err("illegal top-deck choice".into());
        }
        let side = self.top_deck_side(s).ok_or("top-deck side missing")?;
        if s.player(side).deck.as_slice().first().copied() != Some(s.ability_target) {
            return Err("captured top card left its deck".into());
        }
        let mut t = *s;
        if a.arg.as_ref().and_then(Value::as_str) == Some("discard") {
            if t.player(side).discard.len as usize >= MAX_ZONE {
                return Err("discard capacity exceeded".into());
            }
            let effect = Effect::DiscardDeck {
                target: Target::Name(
                    if side == s.card(s.source).owner {
                        "ownPlayer"
                    } else {
                        "enemyPlayer"
                    }
                    .into(),
                ),
                amount: Expr::Const { value: json!(1) },
            };
            t.event_group = EventGroupState::default();
            t.event_group.active = true;
            t.event_group.initiator = t.active;
            t.event_group.context = Selection::one(t.ability_target);
            if !self.generate_group(&mut t, &effect)? || !self.resolve_group(&mut t, &effect)? {
                return Err("closed top-deck discard unexpectedly paused".into());
            }
            t.event_group = EventGroupState::default();
        }
        t.ability_pc += 1;
        self.run_program(&mut t)?;
        Ok(t)
    }
    pub(crate) fn modal_deck_pending(&self, s: &State) -> bool {
        let context = |id: u16| {
            id != 0 && crate::modal_deck_movement_contract::requires(self.program(id - 1))
        };
        let pending = |source: CardId, bits: u16| {
            bits != 0
                && source != 0
                && self
                    .card_programs(s, source)
                    .iter()
                    .enumerate()
                    .any(|(i, id)| {
                        bits & (1 << i) != 0
                            && crate::modal_deck_movement_contract::requires(self.program(*id))
                    })
        };
        context(s.ability_program)
            || pending(s.source, s.pending_programs)
            || s.support.pending
                && (context(s.support.context.ability_program)
                    || pending(s.support.context.source, s.support.context.pending_programs))
            || s.death_frames[..s.death_depth as usize].iter().any(|f| {
                context(f.context.ability_program)
                    || pending(f.context.source, f.context.pending_programs)
            })
            || [&s.death_pending, &s.death_window]
                .into_iter()
                .chain(
                    s.death_frames[..s.death_depth as usize]
                        .iter()
                        .map(|f| &f.window),
                )
                .any(|w| {
                    w.entries[..w.len as usize].iter().any(|e| {
                        e.keyword == 0
                            && crate::modal_deck_movement_contract::requires(
                                self.program(e.program),
                            )
                    })
                })
    }
}
