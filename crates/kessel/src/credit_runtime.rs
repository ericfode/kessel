//! Native Credit lifecycle. Payment prompts and source admission are coordinated
//! centrally; these helpers operate only on reviewed, selected native identities.
use super::*;
use crate::credits::CreditList;

impl Engine {
    pub(super) fn needs_credit_state(&self) -> bool {
        self.catalogue.as_ref().is_some_and(|catalogue| {
            catalogue.metadata.contains_key("TOKEN_CREDIT")
                && self.credit_def.is_some_and(|index| {
                    self.by_code.get("TOKEN_CREDIT") == Some(&index)
                        && self
                            .definitions
                            .get(index as usize)
                            .is_some_and(|d| d.code == "TOKEN_CREDIT")
                })
        })
    }
    fn selected_credit_definition(&self) -> Result<u16, String> {
        if !self.needs_credit_state() {
            return Err(
                "Credit operation requires a reviewed selected native Credit definition".into(),
            );
        }
        self.credit_def
            .ok_or_else(|| "reviewed selected native Credit definition missing".into())
    }
    fn validate_credit_identity(
        &self,
        s: &State,
        seat: Seat,
        id: CardId,
        definition: u16,
    ) -> Result<(), String> {
        if id == 0 || id >= s.next_id || id as usize >= MAX_CARDS {
            return Err("Credit identity outside the allocated physical range".into());
        }
        let card = s.card(id);
        if !card.present
            || card.def != definition
            || card.owner != seat
            || card.parent != 0
            || card.upgrades != 0
            || card.shield != 0
            || card.experience != 0
            || s.players.iter().any(|p| {
                p.base == id
                    || p.leader == id
                    || p.force_id == id
                    || p.hand.contains(id)
                    || p.deck.contains(id)
                    || p.resources.contains(id)
                    || p.discard.contains(id)
                    || p.ground.contains(id)
                    || p.space.contains(id)
            })
            || s.player(seat.other()).credits.contains(id)
            || s.registration_order.position(id).is_some()
            || s.arena_unit_order
                .iter()
                .any(|order| order.as_slice().contains(&id))
        {
            return Err(
                "Credit identity conflicts with native definition, holding owner, or another zone"
                    .into(),
            );
        }
        // No admitted effect can blank or otherwise modify a Credit. The pinned
        // adjuster reads the first token's blanking state; inventing such state
        // without a supported lasting-effect producer would lose provenance.
        if card.damage != 0
            || card.exhausted
            || card.deployed
            || card.epic_used
            || card.phase_power != 0
            || card.phase_hp != 0
            || card.phase_keyword_values != [0; 2]
            || card.phase_keyword_presence != 0
            || card.phase_lost_keywords != 0
            || card.phase_restrictions != 0
            || card.hidden_state != 0
            || card.next_token != 0
            || card.next_upgrade != 0
        {
            return Err("unsupported modified or blanked native Credit state".into());
        }
        Ok(())
    }
    pub(super) fn validate_credit_account(&self, s: &State, seat: Seat) -> Result<(), String> {
        let definition = self.selected_credit_definition()?;
        if !s.credit_state_exact {
            return Err("Credit payment requires exact Credit state".into());
        }
        for id in s.player(seat).credits.as_slice() {
            self.validate_credit_identity(s, seat, u16::from(*id), definition)?;
        }
        Ok(())
    }
    pub(super) fn create_credit_tokens(
        &self,
        s: &mut State,
        seat: Seat,
        amount: u16,
    ) -> Result<u16, String> {
        let definition = self.selected_credit_definition()?;
        if !s.credit_state_exact {
            return Err("Credit creation requires exact Credit state".into());
        }
        for id in s.player(seat).credits.as_slice() {
            self.validate_credit_identity(s, seat, u16::from(*id), definition)?;
        }
        let end = usize::from(s.next_id)
            .checked_add(usize::from(amount))
            .ok_or("Credit allocation overflow")?;
        if s.next_id == 0 || end > MAX_CARDS {
            return Err("card capacity exceeded creating Credits".into());
        }
        let mut credits = s.player(seat).credits;
        for id in s.next_id..end as u16 {
            if s.card(id).present || s.players.iter().any(|p| p.force_id == id) {
                return Err("Credit allocation would reuse an existing physical identity".into());
            }
            credits.append(id)?;
        }
        // All capacity/identity checks precede allocation or BaseZone entry.
        for id in s.next_id..end as u16 {
            *s.card_mut(id) = Card {
                present: true,
                def: definition,
                owner: seat,
                ..Card::default()
            };
        }
        s.next_id = end as u16;
        s.player_mut(seat).credits = credits;
        Ok(amount)
    }
    pub(super) fn transfer_credit_tokens(
        &self,
        s: &mut State,
        from: Seat,
        to: Seat,
        amount: u16,
    ) -> Result<u16, String> {
        let definition = self.selected_credit_definition()?;
        if !s.credit_state_exact {
            return Err("Credit transfer requires exact Credit state".into());
        }
        for seat in [from, to] {
            for id in s.player(seat).credits.as_slice() {
                self.validate_credit_identity(s, seat, u16::from(*id), definition)?;
            }
        }
        if from == to {
            return Ok(0);
        }
        let mut donor = s.player(from).credits;
        let mut recipient = s.player(to).credits;
        let ids = donor.prefix(amount.min(donor.len()))?.to_vec();
        let transferred = donor.transfer_prefix_to(&mut recipient, amount)?;
        for id in ids {
            s.card_mut(u16::from(id)).owner = to;
        }
        s.player_mut(from).credits = donor;
        s.player_mut(to).credits = recipient;
        Ok(transferred)
    }
    pub(super) fn defeat_credits_for_payment(
        &self,
        s: &mut State,
        payer: Seat,
        count: u16,
    ) -> Result<(), String> {
        let definition = self.selected_credit_definition()?;
        if !s.credit_state_exact {
            return Err("Credit payment requires exact Credit state".into());
        }
        if count == 0 {
            return Err("Credit payment must defeat a positive number of tokens".into());
        }
        let ids = s.player(payer).credits.prefix(count)?.to_vec();
        for id in &ids {
            self.validate_credit_identity(s, payer, u16::from(*id), definition)?;
        }
        // The parent OnDefeatCreditsToPayCost captures a front-of-zone prefix.
        // Its negative-order contingents execute in reverse insertion order in
        // the pinned comparator. No admitted hook matches token-only defeats.
        for id in ids.into_iter().rev() {
            s.player_mut(payer).credits.remove(u16::from(id))?;
            // Retain allocated physical metadata outside every zone, matching
            // existing token defeat. No unit-entry/death/registration history.
            *s.card_mut(u16::from(id)) = Card {
                present: true,
                def: definition,
                owner: payer,
                ..Card::default()
            };
        }
        Ok(())
    }
    /// Called after ordinary zone, Force and nextCardId validation/loading.
    /// Stable frames only: pending payment menus cannot be action-window input.
    pub(super) fn load_credit_state(&self, s: &mut State, value: &Value) -> Result<(), String> {
        if value.get("creditPaymentPending").is_some() {
            return Err("pending Credit payment cannot be imported as an action window".into());
        }
        let Some(version) = value.get("creditStateVersion") else {
            if self.needs_credit_state() {
                return Err("selected pool requires explicit Credit state version".into());
            }
            if s.players.iter().any(|p| !p.credits.is_empty())
                || [Seat::P1, Seat::P2].iter().any(|seat| {
                    value["players"][seat.as_str()].get("credits").is_some()
                        || value["players"][seat.as_str()]
                            .get("retiredCredits")
                            .is_some()
                })
            {
                return Err("Credit identities require creditStateVersion".into());
            }
            s.credit_state_exact = false;
            return Ok(());
        };
        if version.as_u64() != Some(1) {
            return Err("unsupported creditStateVersion".into());
        }
        let definition = self.selected_credit_definition()?;
        if s.next_id == 0 || usize::from(s.next_id) > MAX_CARDS {
            return Err("invalid Credit physical identity bound".into());
        }
        let next = value["nextCardId"]
            .as_u64()
            .filter(|next| *next < MAX_CARDS as u64)
            .ok_or("Credit state requires bounded nextCardId")?;
        let mut lists = [CreditList::default(); 2];
        let mut loaded = Vec::new();
        for seat in [Seat::P1, Seat::P2] {
            if !s.player(seat).credits.is_empty() {
                return Err("Credit import requires empty destination lists".into());
            }
            for kind in ["credits", "retiredCredits"] {
                let refs = value["players"][seat.as_str()][kind].as_array().ok_or("versioned Credit state requires live and retired Credit lists for both players")?;
                let mut previous_retired = 0;
                for reference in refs {
                    let object = reference
                        .as_object()
                        .ok_or("Credit reference must be an object")?;
                    if !object
                        .keys()
                        .all(|key| ["id", "setCode", "name"].contains(&key.as_str()))
                    {
                        return Err("unsupported Credit snapshot field (including blanking or lasting effects)".into());
                    }
                    let text = reference["id"]
                        .as_str()
                        .ok_or("Credit reference missing identity")?;
                    let id = cid(text)?;
                    if text != format!("c{}", id - 1) || u64::from(id) > next || id >= s.next_id {
                        return Err("Credit reference exceeds exact nextCardId or has noncanonical identity".into());
                    }
                    if reference["setCode"] != "TOKEN_CREDIT"
                        || reference["name"] != "credit"
                        || s.card(id).present
                        || loaded.iter().any(|(_, prior)| *prior == id)
                        || s.players.iter().any(|p| p.force_id == id)
                    {
                        return Err("Credit reference conflicts with native identity, allocated card, or Force".into());
                    }
                    if kind == "credits" {
                        lists[seat.index()].append(id)?;
                    } else {
                        if id <= previous_retired {
                            return Err("retired Credit references must have canonical ascending identity order".into());
                        }
                        previous_retired = id;
                    }
                    loaded.push((seat, id));
                }
            }
        }
        for (seat, id) in loaded {
            *s.card_mut(id) = Card {
                present: true,
                def: definition,
                owner: seat,
                ..Card::default()
            };
        }
        for seat in [Seat::P1, Seat::P2] {
            s.player_mut(seat).credits = lists[seat.index()];
        }
        s.credit_state_exact = true;
        Ok(())
    }
    pub(super) fn snapshot_credit_state(&self, s: &State, value: &mut Value) {
        if !s.credit_state_exact {
            assert!(
                s.players.iter().all(|p| p.credits.is_empty())
                    && !(1..s.next_id).any(|id| {
                        s.card(id).present && Some(s.card(id).def) == self.credit_def
                    }),
                "Credit snapshot requires exact evidence for live and retired identities"
            );
            return;
        }
        assert!(
            self.needs_credit_state(),
            "exact Credit snapshot requires selected native provenance"
        );
        let definition = self.credit_def.expect("selected Credit definition");
        for seat in [Seat::P1, Seat::P2] {
            for id in s.player(seat).credits.as_slice() {
                self.validate_credit_identity(s, seat, u16::from(*id), definition)
                    .expect("valid native Credit snapshot identity");
            }
        }
        for id in 1..s.next_id {
            let card = s.card(id);
            if card.present && card.def == definition {
                self.validate_credit_identity(s, card.owner, id, definition)
                    .expect("valid retained native Credit snapshot identity");
            }
        }
        value["creditStateVersion"] = json!(1);
        for seat in [Seat::P1, Seat::P2] {
            value["players"][seat.as_str()]["credits"] = Value::Array(
                s.player(seat)
                    .credits
                    .as_slice()
                    .iter()
                    .map(|id| self.card_ref(s, u16::from(*id)))
                    .collect(),
            );
            // Retained defeated-token slots already participate in State hashes.
            // Export them explicitly rather than infer metadata from ID gaps or
            // make their hash contribution disappear after defeat.
            value["players"][seat.as_str()]["retiredCredits"] = Value::Array(
                (1..s.next_id)
                    .filter(|id| {
                        let card = s.card(*id);
                        card.present
                            && card.def == definition
                            && card.owner == seat
                            && !s.players.iter().any(|p| p.credits.contains(*id))
                    })
                    .map(|id| self.card_ref(s, id))
                    .collect(),
            );
        }
    }
}

#[cfg(test)]
#[path = "credit_lifecycle_tests.rs"]
mod credit_lifecycle_tests;
