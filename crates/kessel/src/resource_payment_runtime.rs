//! A single yielding resource-cost boundary for card, ability and effect costs.
use super::*;
use crate::credits::{MenuChoice, PaymentPlan};
use crate::resource_payment::{PaymentPurpose, PaymentStage, ResourcePayment};

impl Engine {
    pub(super) fn available_credit_count(&self, s: &State, payer: Seat) -> u16 {
        if s.credit_state_exact && self.needs_credit_state() {
            // Modified/blanked Credits have no admitted producer. Execution
            // validates the native ledger before offering the pinned stage.
            PaymentPlan::available(s.player(payer).credits.len(), false)
        } else {
            0
        }
    }
    pub(super) fn can_pay_resources(&self, s: &State, payer: Seat, amount: u16) -> bool {
        usize::from(amount)
            <= self
                .ready_resources(s, payer)
                .saturating_add(usize::from(self.available_credit_count(s, payer)))
    }
    pub(super) fn resource_request(
        &self,
        s: &State,
        purpose: PaymentPurpose,
        source: CardId,
        target: CardId,
        amount: u16,
        from_resources: bool,
        can_cancel: bool,
        end_program: bool,
    ) -> ResourcePayment {
        ResourcePayment {
            purpose,
            payer: s.card(source).owner,
            actor: s.active,
            source,
            target,
            amount,
            from_resources,
            can_cancel,
            end_program,
            program: s.ability_program,
            pc: s.ability_pc,
            cost_index: s.resource_cost_index,
            ..ResourcePayment::default()
        }
    }
    /// Some is the resolved resource event's amount (possibly zero); None is a
    /// retained payment prompt or child cost window. The amount was prepared
    /// once and is never recomputed after that cost window changes the board.
    pub(super) fn pay_resource_cost(
        &self,
        s: &mut State,
        request: ResourcePayment,
    ) -> Result<Option<u16>, String> {
        if s.resource_payment.active() {
            if !s.resource_payment.same_boundary(&request)
                || s.resource_payment.stage != PaymentStage::Paid
            {
                return Err("resource payment continuation does not match prepared cost".into());
            }
            let paid = s.resource_payment.amount - s.resource_payment.credits_used;
            s.resource_payment = ResourcePayment::default();
            return Ok(Some(paid));
        }
        if !s.player(request.payer).credits.is_empty() {
            self.validate_credit_account(s, request.payer)?;
        }
        let ready = u16::try_from(self.ready_resources(s, request.payer))
            .map_err(|_| "ready resource count exceeds payment bound")?;
        let plan = PaymentPlan::new(
            request.amount,
            ready,
            self.available_credit_count(s, request.payer),
            false,
            request.can_cancel,
        )?;
        s.resource_payment = request;
        if let Some(plan) = plan {
            s.resource_payment.stage = PaymentStage::Credits;
            s.resource_payment.minimum = plan.minimum;
            s.resource_payment.maximum = plan.maximum;
            s.active = request.payer;
            s.decision = Decision::CreditMenu;
            return Ok(None);
        }
        self.resolve_resource_cost(s, 0)?;
        if self.open_payment_deaths(s)? {
            return Ok(None);
        }
        let paid = s.resource_payment.amount;
        s.resource_payment = ResourcePayment::default();
        Ok(Some(paid))
    }
    fn resolve_resource_cost(&self, s: &mut State, credits: u16) -> Result<(), String> {
        let request = s.resource_payment;
        if !request.active() || request.stage == PaymentStage::Paid || credits > request.amount {
            return Err("invalid prepared resource payment".into());
        }
        let remaining = request.amount - credits;
        if self.ready_resources(s, request.payer) < usize::from(remaining) {
            return Err("resource payment became unaffordable; recovery is unsupported".into());
        }
        if credits != 0 {
            self.defeat_credits_for_payment(s, request.payer, credits)?;
        }
        s.active = request.payer;
        if request.from_resources {
            self.spend_resource_play(s, remaining, request.source);
        } else {
            self.spend(s, request.payer, remaining);
        }
        s.active = request.actor;
        s.resource_payment.stage = PaymentStage::Paid;
        s.resource_payment.credits_used = credits;
        s.resource_payment.can_cancel = false;
        self.settle_payment_state(s)?;
        Ok(())
    }
    pub(super) fn settle_cost_window(
        &self,
        s: &mut State,
        end_program: bool,
    ) -> Result<bool, String> {
        self.settle_payment_state(s)?;
        if s.death_pending.len == 0 {
            return Ok(false);
        }
        s.resource_payment = self.resource_request(
            s,
            PaymentPurpose::CostWindow,
            s.source,
            s.ability_target,
            0,
            false,
            false,
            end_program,
        );
        s.resource_payment.stage = PaymentStage::Paid;
        let opened = self.open_payment_deaths(s)?;
        if !opened {
            s.resource_payment = ResourcePayment::default();
        }
        Ok(opened)
    }
    fn settle_payment_state(&self, s: &mut State) -> Result<(), String> {
        let start = s.death_pending.len as usize;
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        for death in &mut s.death_pending.entries[start..] {
            if death.source == 0 {
                break;
            }
            death.after_pc = if s.ability_program == 0 {
                0
            } else {
                s.ability_pc
            };
            death.capture.set_depth(s.death_depth);
        }
        Ok(())
    }
    fn open_payment_deaths(&self, s: &mut State) -> Result<bool, String> {
        if !s.death_pending.entries[..s.death_pending.len as usize]
            .iter()
            .any(|entry| {
                entry.after_pc == 0
                    || (entry.capture.depth() == s.death_depth
                        && entry.after_pc == s.ability_pc
                        && s.ability_program != 0)
            })
        {
            return Ok(false);
        }
        if s.event_group.active {
            if s.payment_group.active {
                return Err("nested prepared payment group capacity exceeded".into());
            }
            s.payment_group = s.event_group;
            s.event_group = EventGroupState::default();
            s.resource_payment.group_saved = true;
        }
        self.open_deaths(s, DeathContinuation::ResourcePayment)
    }
    pub(super) fn resume_resource_payment(&self, s: &mut State) -> Result<(), String> {
        let request = s.resource_payment;
        if request.group_saved {
            if !s.payment_group.active || s.event_group.active {
                return Err("prepared payment group continuation is inconsistent".into());
            }
            s.event_group = s.payment_group;
            s.payment_group = EventGroupState::default();
            s.resource_payment.group_saved = false;
        }
        match request.purpose {
            PaymentPurpose::UnitPlay => {
                self.play_unit_with_payment(s, request.source, request.from_resources)?;
            }
            PaymentPurpose::UpgradePlay => {
                self.play_upgrade_with_payment(s, request.source, request.target)?;
            }
            PaymentPurpose::ProgramPlay
            | PaymentPurpose::AbilityCost
            | PaymentPurpose::Effect
            | PaymentPurpose::CostWindow => {
                if request.purpose == PaymentPurpose::CostWindow {
                    s.resource_payment = ResourcePayment::default();
                }
                if request.end_program {
                    if self.commit_program_mode(s, true)? {
                        self.end_program(s)?;
                    }
                } else {
                    self.run_program(s)?;
                }
            }
            PaymentPurpose::None => return Err("resource payment continuation is missing".into()),
        }
        Ok(())
    }
    fn payment_plan(&self, s: &State) -> PaymentPlan {
        PaymentPlan {
            minimum: s.resource_payment.minimum,
            maximum: s.resource_payment.maximum,
            can_cancel: s.resource_payment.can_cancel,
        }
    }
    pub(super) fn credit_payment_actions(&self, s: &State) -> Vec<Action> {
        let plan = self.payment_plan(s);
        match s.decision {
            Decision::CreditMenu => plan
                .choices()
                .into_iter()
                .enumerate()
                .map(|(index, choice)| Action::button(choice.label(), index))
                .collect(),
            Decision::CreditAmount => plan
                .number_range()
                .into_iter()
                .flatten()
                .map(|n| Action::button(n.to_string(), n.to_string()))
                .collect(),
            _ => vec![],
        }
    }
    pub(super) fn apply_credit_payment_action(
        &self,
        s: &State,
        a: &Action,
    ) -> Result<State, String> {
        let mut t = *s;
        let chosen = if s.decision == Decision::CreditMenu {
            let index = a
                .arg
                .as_ref()
                .and_then(Value::as_u64)
                .ok_or("Credit menu index missing")?;
            let choice = *self
                .payment_plan(s)
                .choices()
                .get(index as usize)
                .ok_or("Credit menu index invalid")?;
            match choice {
                MenuChoice::Cancel => {
                    t.active = t.resource_payment.payer;
                    self.cancel(&mut t);
                    return Ok(t);
                }
                MenuChoice::SelectAmount => {
                    t.resource_payment.stage = PaymentStage::Amount;
                    t.decision = Decision::CreditAmount;
                    return Ok(t);
                }
                MenuChoice::Skip => 0,
                MenuChoice::Use(amount) => amount,
            }
        } else {
            let n = a
                .arg
                .as_ref()
                .and_then(Value::as_str)
                .ok_or("Credit number missing")?
                .parse::<u16>()
                .map_err(|_| "invalid Credit number")?;
            self.payment_plan(s).validate_amount(n)?;
            n
        };
        self.resolve_resource_cost(&mut t, chosen)?;
        if !self.open_payment_deaths(&mut t)? {
            self.resume_resource_payment(&mut t)?;
        }
        // Match the ordinary public action boundary, including unique-entry prompts.
        self.sweep_stat_defeats(&mut t)?;
        Ok(t)
    }
    pub(super) fn run_adjusted_resource_payment(
        &self,
        s: &mut State,
        effect: &crate::ability_ir::Effect,
    ) -> Result<bool, String> {
        let amount = crate::pay_adjusted_resources::amount(effect)?;
        let request = self.resource_request(
            s,
            PaymentPurpose::Effect,
            s.source,
            s.ability_target,
            amount,
            false,
            false,
            false,
        );
        if !s.resource_payment.active() && !self.can_pay_resources(s, request.payer, amount) {
            s.ability_result = 0;
            s.ability_succeeded = false;
            s.event_result = StepEventResult::default();
            return Ok(true);
        }
        let Some(resources) = self.pay_resource_cost(s, request)? else {
            return Ok(false);
        };
        s.ability_result = resources;
        s.ability_succeeded = true;
        s.event_result = StepEventResult::default();
        Ok(true)
    }
    pub(super) fn play_unit_with_payment(
        &self,
        s: &mut State,
        id: CardId,
        from_resources: bool,
    ) -> Result<bool, String> {
        if !self.named_play_allowed(s,id) {return Err("named card cannot be played".into());}
        let seat = s.card(id).owner;
        if !(if from_resources {
            s.player(seat).resources.contains(id)
        } else {
            s.player(seat).hand.contains(id)
        }) {
            return Err("unit left its prepared payment origin".into());
        }
        let cost = if from_resources {
            self.smuggle_cost(s, id).ok_or("smuggle cost missing")?
        } else {
            self.cost(s, id)
        };
        s.phase_history.preflight_play(seat)?;
        s.source = id;
        let request = self.resource_request(
            s,
            PaymentPurpose::UnitPlay,
            id,
            0,
            cost,
            from_resources,
            true,
            false,
        );
        if self.pay_resource_cost(s, request)?.is_none() {
            return Ok(false);
        }
        if from_resources {
            s.player_mut(seat).resources.remove(id);
        } else {
            s.player_mut(seat).hand.remove(id);
        }
        s.card_mut(id).exhausted =
            !self.self_static_ongoing(s, id, crate::static_ongoing::SelfEffect::EntersPlayReady);
        s.card_mut(id).set_hidden_qualified(true);
        self.add_arena(s, id)?;
        s.phase_history.entered(seat, id);
        s.phase_history.played(seat, id)?;
        if from_resources {
            self.resource_deck_top_after_play(s, seat)?;
            s.enter_smuggle = true;
        }
        s.source = id;
        s.trigger_context = 1;
        if self.def(s, id).info.unique && self.duplicates(s, id) > 1 {
            s.decision = Decision::Unique;
        } else {
            self.enter_triggers(s)?;
        }
        Ok(true)
    }
    pub(super) fn play_upgrade_with_payment(
        &self,
        s: &mut State,
        upgrade: CardId,
        host: CardId,
    ) -> Result<bool, String> {
        let seat = s.card(upgrade).owner;
        let from_resources = s.player(seat).resources.contains(upgrade);
        let cost = self
            .upgrade_play_cost(s, upgrade, host)
            .ok_or("illegal attachment play context")?;
        s.phase_history.preflight_play(seat)?;
        let request = self.resource_request(
            s,
            PaymentPurpose::UpgradePlay,
            upgrade,
            host,
            cost,
            from_resources,
            true,
            false,
        );
        if self.pay_resource_cost(s, request)?.is_none() {
            return Ok(false);
        }
        if !self.upgrade_targets(s, upgrade).contains(&host) {
            return Err("prepared attachment host became illegal during payment".into());
        }
        if from_resources {
            s.player_mut(seat).resources.remove(upgrade);
        } else {
            s.player_mut(seat).hand.remove(upgrade);
        }
        self.attach(s, upgrade, host);
        s.phase_history.played(seat, upgrade)?;
        s.trigger_context = 1;
        s.enter_smuggle = from_resources;
        if from_resources {
            self.resource_deck_top_after_play(s, seat)?;
        }
        if self.def(s, upgrade).info.unique && self.duplicates(s, upgrade) > 1 {
            let same = self
                .upgrades(s, host)
                .into_iter()
                .filter(|other| {
                    s.card(*other).def == s.card(upgrade).def && s.card(*other).owner == seat
                })
                .collect::<Vec<_>>();
            if same.len() == self.duplicates(s, upgrade) && same.len() > 1 {
                self.settle_enter_triggers(s, Some(same[0]))?;
            } else {
                s.decision = Decision::Unique;
            }
        } else {
            self.enter_triggers(s)?;
        }
        Ok(true)
    }
}
