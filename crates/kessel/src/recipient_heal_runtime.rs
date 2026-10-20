// Home One's non-pausing window owns three distinct phases: snapshot every
// friendly recipient and callback amount; recheck each captured event and mutate
// without state checks; then run one framework state sweep. It cannot use the
// scalar loop once per card, which would expose intermediate lethal boards.
// No onDamageHealed listeners/replacements are admitted by the current runtime;
// this extension does not synthesize their event windows or prompt ownership.
impl Engine {
    fn recipient_heal_capture(
        &self,
        s: &State,
        source: CardId,
    ) -> Result<Vec<(CardId, u16)>, String> {
        if source == 0 || !s.card(source).present {
            return Err("recipient heal requires retained source identity".into());
        }
        let p = s.player(s.card(source).owner);
        // Engine getArenaUnits filters controller within ground, then space.
        // The per-controller lists preserve exactly these subsequences even in
        // legacy states without global inter-controller arena order evidence.
        let mut captured = Vec::with_capacity(p.ground.len as usize + p.space.len as usize);
        for &id in p.ground.as_slice().iter().chain(p.space.as_slice()) {
            if id == 0
                || !s.card(id).present
                || !self.in_arena(s, id)
                || s.card(id).owner != s.card(source).owner
                || captured.iter().any(|(prior, _)| *prior == id)
                || !(self.def(s, id).info.is("unit")
                    || (self.def(s, id).info.is("leader") && s.card(id).deployed))
            {
                return Err("recipient heal arena membership is inconsistent".into());
            }
            captured.push((id, s.card(id).damage));
        }
        if captured.len() >= MAX_CARDS {
            return Err("recipient heal batch exceeds physical card capacity".into());
        }
        Ok(captured)
    }
    fn recipient_heal_can_affect(&self, s: &State, source: CardId, require_change: bool) -> bool {
        self.recipient_heal_capture(s, source).is_ok_and(|v| {
            v.iter().any(|(id, _)| {
                !require_change
                    || (s.card(*id).damage > 0
                        && !self.card_restricted(s, *id, RestrictionVerb::BeHealed))
            })
        })
    }
    fn recipient_heal_resolve(
        &self,
        s: &mut State,
        captured: &[(CardId, u16)],
    ) -> Result<(u16, bool), String> {
        // Check aggregate capacity before mutations, including diagnostic captures.
        let total = captured
            .iter()
            .try_fold(0u32, |n, (_, amount)| n.checked_add(u32::from(*amount)))
            .ok_or("recipient heal total exceeds capacity")?;
        let _ = total;
        let mut healed = 0u32;
        let mut resolved = false;
        for &(id, amount) in captured {
            // Captured event.amount is fixed. Current damage and BeHealed remain
            // live at the per-event condition/handler boundary, including zero.
            if self.in_arena(s, id)
                && s.card(id).damage > 0
                && !self.card_restricted(s, id, RestrictionVerb::BeHealed)
            {
                healed = healed
                    .checked_add(u32::from(self.heal_card(s, id, amount)))
                    .ok_or("recipient heal result exceeds capacity")?;
                resolved = true;
            }
        }
        self.finish_scalar_effect(s)?;
        // Admission and standalone validation forbid then/ifYouDo, nested groups,
        // selectors and any surrounding sequence. No admitted onDamageHealed
        // listener consumes this value; begin_program resets ability_result before
        // another registration, and retained parent frames restore their own value.
        // This closed family has no result-consuming continuation. Preserve every
        // physical heal and saturate only the existing bounded numeric protocol.
        Ok((u16::try_from(healed).unwrap_or(u16::MAX), resolved))
    }
    fn apply_recipient_heal(&self, s: &mut State, e: &Effect) -> Result<(u16, bool), String> {
        crate::recipient_heal_contract::effect(e)?;
        if s.event_group.active || s.event_group.resolving {
            return Err("recipient heal cannot execute inside a prepared group".into());
        }
        let captured = self.recipient_heal_capture(s, s.source)?;
        // Keep failure atomic, including the final stat/capacity checks.
        let mut work = *s;
        let result = self.recipient_heal_resolve(&mut work, &captured)?;
        *s = work;
        Ok(result)
    }
}
