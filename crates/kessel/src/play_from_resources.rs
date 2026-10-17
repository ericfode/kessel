//! Alternate play from the resource zone: the pinned engine's **Plot** and
//! **Smuggle** keywords.
//!
//! Sources (read-only pinned checkout, forceteki):
//! - `server/game/abilities/keyword/PlotAbility.ts` — optional triggered ability
//!   watching `onLeaderDeployed`, `zoneFilter: ZoneName.Resource`, then
//!   `PlayCardSystem({ playType: PlayType.Plot, canPlayFromAnyZone: true,
//!   target: context.source })` and `ifYouDo: ResourceCardSystem(top of deck)`.
//! - `server/game/core/ability/PlayableOrDeployableCard.ts` — a resource-zone
//!   card with Smuggle gains a Smuggle play action; `getCheapestPlayAction`
//!   drops Smuggle instances that carry additional costs.
//! - `server/game/core/ability/PlayCardAction.ts` and
//!   `server/costs/PlayCardResourceCost.ts` — alternate play costs use the
//!   keyword's own cost and aspect list; Plot pays the printed cost/aspects.
//! - `server/game/core/player/Player.ts::exhaustResources` and
//!   `server/game/core/zone/ResourceZone.ts::rearrangeResourceExhaustState` —
//!   the played card is exhausted first when ready, then ready resources are
//!   exhausted until the cost is paid; the zone's exhausted count is preserved.
//! - `server/game/actions/PlayUnitAction.ts`, `PlayUpgradeAction.ts`,
//!   `PlayEventAction.ts` — Smuggle's "replace it with the top card of your
//!   deck" is a sibling of the play events (before When Played triggers);
//!   Plot's replacement is the ability's `ifYouDo`.
//! - `test/server/core/abilities/keyword/{Plot,Smuggle}.spec.ts` — recorded
//!   engine behavior for affordability, replacement timing and empty decks.
//!
//! Printed-text parser: the compiler leaves `keywordValues` empty for both
//! keywords, so admission derives the exact instance from the printed text with
//! the engine's own regexes (`KeywordHelpers.getRegexForKeyword`):
//! - Plot: `/(?:^|(?:\n))Plot/g`
//! - Smuggle: `/(?:\n)?Smuggle\s\[\s*(\d+)\s+resources(?:,\s*|\s+)([\w\s]+)(,.*)?\]/g`
//! Metadata `card.keywords` is a *mention* list (e.g. "search for a card with
//! Plot" or "defeat each resource with Smuggle"), so a mention alone is not an
//! instance; only the own-printed-text parse creates one. The engine throws on
//! more than one instance of a keyword in the same text, so duplicate text is
//! rejected instead of guessed.
//!
//! ## Deterministic resource layout
//!
//! `rearrangeResourceExhaustState` shuffles the resource zone and re-assigns
//! which resources are exhausted (keeping the count), for the hidden-information
//! representation only. CR 1.7.4 lets a player rearrange their own resources at
//! any time until a specific resource is chosen, so the identity of the other
//! exhausted resources is a legal player choice, not a random rule outcome.
//! Kessel fixes one legal deterministic assignment: the played card is exhausted
//! first when ready, then other ready resources in Kessel zone order; the zone
//! order is never rewritten and the exhausted count is preserved. This module
//! never reads resource-zone order/exhaustion as an engine-determined fact.
//! Consequently a forceteki recording that exercises Plot or Smuggle can differ
//! from Kessel only in that hidden layout/exhaustion assignment (documented in
//! the batch receipt as unverified conformance, not as a rule approximation).

use super::*;
use crate::model::*;

/// One intrinsic printed Smuggle instance: `Smuggle [N resources Aspect ...]`.
/// `aspects` counts the keyword's own aspect list, which is the multiset used
/// for the aspect penalty (not the card's printed aspects).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SmuggleCost {
    pub cost: u16,
    pub aspects: [u8; 6],
}

pub(crate) fn aspect_index(name: &str) -> Option<usize> {
    match name {
        "vigilance" => Some(0),
        "command" => Some(1),
        "aggression" => Some(2),
        "cunning" => Some(3),
        "heroism" => Some(4),
        "villainy" => Some(5),
        _ => None,
    }
}

fn is_js_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn is_js_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Occurrences of the engine's Plot regex `/(?:^|(?:\n))Plot/g`: the literal
/// starts the text or a line.
pub(crate) fn plot_occurrences(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut from = 0;
    while let Some(offset) = text[from..].find("Plot") {
        let at = from + offset;
        if at == 0 || bytes[at - 1] == b'\n' {
            count += 1;
        }
        from = at + "Plot".len();
    }
    count
}

/// Matches the engine's Smuggle regex. Structural mismatches are simply not
/// occurrences (the regex does not match); a structurally matched instance
/// whose aspect list cannot be converted to aspects is an engine error, which
/// admission reports as a rejection.
pub(crate) fn smuggle_occurrences(text: &str) -> Result<Vec<SmuggleCost>, String> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find("Smuggle") {
        let mut i = from + offset + "Smuggle".len();
        from = from + offset + "Smuggle".len();
        // `\s\[`: exactly one whitespace, then '['.
        if i >= bytes.len() || !is_js_space(bytes[i]) || bytes.get(i + 1) != Some(&b'[') {
            continue;
        }
        i += 2;
        while bytes.get(i).is_some_and(|c| is_js_space(*c)) {
            i += 1;
        }
        let digits = i;
        while bytes.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        if i == digits {
            continue;
        }
        let cost_text = &text[digits..i];
        let spaces = i;
        while bytes.get(i).is_some_and(|c| is_js_space(*c)) {
            i += 1;
        }
        if i == spaces || !text[i..].starts_with("resources") {
            continue;
        }
        i += "resources".len();
        // `(?:,\s*|\s+)`
        if bytes.get(i) == Some(&b',') {
            i += 1;
            while bytes.get(i).is_some_and(|c| is_js_space(*c)) {
                i += 1;
            }
        } else {
            let spaces = i;
            while bytes.get(i).is_some_and(|c| is_js_space(*c)) {
                i += 1;
            }
            if i == spaces {
                continue;
            }
        }
        // `([\w\s]+)` is greedy; the following `(,.*)?\]` only matches on the
        // same line, so take the maximal word/space run and require ',' or ']'.
        let aspects_start = i;
        while bytes.get(i).is_some_and(|c| is_js_word(*c) || is_js_space(*c)) {
            i += 1;
        }
        if i == aspects_start {
            continue;
        }
        let aspect_text = &text[aspects_start..i];
        let additional_costs = match bytes.get(i) {
            Some(b']') => false,
            Some(b',') => {
                // `,.*\]` on the rest of this line.
                match text[i + 1..].find(']') {
                    Some(end) if !text[i + 1..i + 1 + end].contains('\n') => {
                        true
                    }
                    _ => continue,
                }
            }
            _ => continue,
        };
        let cost: u16 = cost_text
            .parse()
            .ok()
            .filter(|n| *n <= 1000)
            .ok_or_else(|| "Smuggle cost must be an integer in 0..=1000".to_string())?;
        let mut aspects = [0u8; 6];
        let mut seen = 0;
        for word in aspect_text.split(' ') {
            let index = aspect_index(&word.to_ascii_lowercase()).ok_or_else(|| {
                format!("invalid printed Smuggle aspect {word:?}; the pinned engine throws here")
            })?;
            aspects[index] += 1;
            seen += 1;
        }
        if seen == 0 {
            continue;
        }
        if additional_costs {
            return Err("Smuggle additional costs are not implemented".into());
        }
        found.push(SmuggleCost { cost, aspects });
    }
    Ok(found)
}

/// Intrinsic Plot instance from one printed side. `mentions` is the number of
/// "plot" entries in the card's metadata keyword list (the engine only parses a
/// keyword that the metadata lists).
pub(crate) fn plot_instance(mentions: usize, sides: &[&str]) -> Result<bool, String> {
    let matches: usize = sides.iter().map(|side| plot_occurrences(side)).sum();
    if matches > 1 {
        return Err("printed Plot appears more than once per side; the pinned engine throws".into());
    }
    Ok(mentions >= 1 && matches == 1)
}

/// Intrinsic Smuggle instance. Cost-bearing keywords are not deduped by the
/// engine, so duplicate metadata mentions would produce duplicate play actions
/// with identical keys; reject that card data instead of guessing.
pub(crate) fn smuggle_instance(mentions: usize, sides: &[&str]) -> Result<Option<SmuggleCost>, String> {
    let mut instances = Vec::new();
    for side in sides {
        instances.extend(smuggle_occurrences(side)?);
    }
    if mentions == 0 {
        return Ok(None);
    }
    if mentions > 1 {
        return Err("duplicate Smuggle keyword metadata would produce duplicate play actions".into());
    }
    match instances.len() {
        0 => Ok(None),
        1 => Ok(instances.pop()),
        _ => Err("printed Smuggle appears more than once; the pinned engine throws".into()),
    }
}

impl Engine {
    /// Aspect penalty for an explicit aspect multiset. Mirrors `cost`: every
    /// cost aspect consumes one deck aspect (leader plus base, forceteki
    /// `Player.getPenaltyAspects`) and each unmatched one costs 2.
    pub(super) fn aspect_cost(&self, s: &State, seat: Seat, base: u16, aspects: &[u8; 6]) -> u16 {
        let p = s.player(seat);
        let leader = self.def(s, p.leader);
        let base_def = self.def(s, p.base);
        let penalty: u16 = (0..6)
            .map(|i| {
                u16::from(
                    aspects[i].saturating_sub(leader.aspects[i].saturating_add(base_def.aspects[i])),
                ) * 2
            })
            .sum();
        base + penalty
    }

    pub(super) fn smuggle_cost(&self, s: &State, id: CardId) -> Option<u16> {
        self.def(s, id)
            .smuggle
            .map(|smuggle| self.aspect_cost(s, s.active, smuggle.cost, &smuggle.aspects))
    }

    /// Card types the runtime can play from a resource-zone card. Events need
    /// their executable program, upgrades need a legal attach target; this
    /// matches the hand-play availability rules.
    fn resource_play_available(&self, s: &State, id: CardId) -> bool {
        let def = self.def(s, id);
        if def.info.is("event") {
            return !self.available_programs(s, id, "event").is_empty();
        }
        if def.info.is("upgrade") {
            return !self.upgrade_targets(s, id).is_empty();
        }
        def.info.is("unit")
    }

    /// The engine checks `readyResourceCount >= requiredReadyResources` without
    /// special-casing the played card: a ready source resource is itself part of
    /// that count and is exhausted first.
    fn card_affordable(&self, s: &State, seat: Seat, cost: u16) -> bool {
        cost as usize <= self.ready_resources(s, seat)
    }

    pub(super) fn plot_playable(&self, s: &State, id: CardId) -> bool {
        let def = self.def(s, id);
        // Control changes of resource cards are not admitted; the played card is
        // always owned by the player whose resource zone holds it.
        def.plot
            && s.card(id).owner == s.active
            && self.card_affordable(s, s.active, self.cost_for(s, s.active, id))
            && self.resource_play_available(s, id)
    }

    pub(super) fn smuggle_playable(&self, s: &State, id: CardId) -> bool {
        let def = self.def(s, id);
        def.smuggle.is_some()
            && s.card(id).owner == s.active
            && self
                .smuggle_cost(s, id)
                .is_some_and(|cost| self.card_affordable(s, s.active, cost))
            && self.resource_play_available(s, id)
    }

    /// Plot abilities in the deploying player's resource zone, in Kessel zone
    /// order. The engine offers one trigger per controlled Plot instance; the
    /// menu order is the engine's registration order (recorded Plot spec menus
    /// list resource-zone order before the leader's own deployed keyword).
    pub(super) fn plot_candidates(&self, s: &State) -> Vec<CardId> {
        if !s.plot.window_open() {
            return vec![];
        }
        s.player(s.active)
            .resources
            .as_slice()
            .iter()
            .copied()
            .filter(|id| !s.plot.is_done(*id) && self.plot_playable(s, *id))
            .collect()
    }

    pub(super) fn plot_trigger_title(&self, s: &State, id: CardId) -> String {
        format!("Play {} using Plot", self.def(s, id).info.title)
    }

    /// Suspends the leader-deploy window and offers the Plot ability's optional
    /// Trigger/Pass prompt. The outer window's remaining triggers are stored in
    /// the plot state; no card rule reads them while the play resolves.
    pub(super) fn begin_plot_trigger(&self, s: &mut State, id: CardId) -> Result<(), String> {
        if !s.plot.window_open() || s.plot.is_done(id) || !self.plot_playable(s, id) {
            return Err("chosen Plot trigger is no longer playable".into());
        }
        s.plot.card = id;
        s.plot.leader_pending = s.pending_triggers;
        s.plot.leader_programs = s.pending_programs;
        s.plot.stage = PlotState::OFFERED;
        s.plot.mark_done(id);
        s.pending_triggers = 0;
        s.pending_programs = 0;
        s.decision = Decision::PlotOptional;
        Ok(())
    }

    pub(super) fn pass_plot_trigger(&self, s: &mut State) -> Result<(), String> {
        let plot = s.plot;
        // The declined instance stays resolved for this window (the engine
        // removes it from the trigger window); the leader and its remaining
        // triggers stay suspended.
        s.plot.card = 0;
        s.plot.stage = 0;
        s.plot.leader_pending = 0;
        s.plot.leader_programs = 0;
        s.source = plot.leader;
        s.set_trigger_context(TriggerContext::LeaderDeploy);
        s.pending_triggers = plot.leader_pending;
        s.pending_programs = plot.leader_programs;
        self.process_triggers(s)
    }

    /// Pays exactly `amount` ready resources, exhausting the played resource-zone
    /// card first when it is ready (`PlayCardAction.checkAndRearrangeResources`
    /// and `ResourceCost.getExhaustResourceEvent` prioritize the played card),
    /// then other ready resources in zone order. The zone order is never
    /// rewritten; CR 1.7.4 makes this a legal deterministic layout choice.
    pub(super) fn exhaust_played_resource(
        &self,
        s: &mut State,
        seat: Seat,
        amount: u16,
        source: CardId,
    ) {
        let mut left = amount;
        if left > 0
            && s.player(seat).resources.contains(source)
            && !s.card(source).exhausted
        {
            s.card_mut(source).exhausted = true;
            left -= 1;
        }
        if left > 0 {
            let ids = s.player(seat).resources;
            for &id in ids.as_slice() {
                if left > 0 && !s.card(id).exhausted {
                    s.card_mut(id).exhausted = true;
                    left -= 1;
                }
            }
        }
        debug_assert_eq!(left, 0, "validated resource payment must be affordable");
    }

    /// The engine's replacement: the top card of the payer's deck enters the
    /// resource zone exhausted (`Player.resourceCard`/`initializeForCurrentZone`),
    /// or nothing happens when the deck is empty and the play still succeeds.
    pub(super) fn resource_top_of_deck(&self, s: &mut State, seat: Seat) -> Result<(), String> {
        let Some(top) = s.player_mut(seat).deck.pop_front() else {
            return Ok(());
        };
        s.player_mut(seat).resources.push(top)?;
        s.card_mut(top).exhausted = true;
        Ok(())
    }

    /// The Trigger half of the Plot ability's optional prompt.
    pub(super) fn confirm_plot_trigger(&self, s: &mut State) -> Result<(), String> {
        let plot = s.plot;
        let seat = s.active;
        if plot.card == 0 || !s.player(seat).resources.contains(plot.card) {
            return Err("Plot play requires the chosen card in the resource zone".into());
        }
        if !self.plot_playable(s, plot.card) {
            return Err("chosen Plot play is no longer legal".into());
        }
        let id = plot.card;
        s.phase_history.preflight_play(seat)?;
        let cost = self.cost_for(s, seat, id);
        self.exhaust_played_resource(s, seat, cost, id);
        s.player_mut(seat).resources.remove(id);
        let def = self.def(s, id);
        if def.info.is("event") {
            s.player_mut(seat).discard.push(id)?;
            s.phase_history.played(seat, id)?;
            s.source = id;
            s.set_trigger_context(TriggerContext::Action);
            let programs = self.available_programs(s, id, "event");
            let program = *programs
                .first()
                .ok_or("Plot event has no executable ability")?;
            return self.begin_program(s, program, ProgramContinuation::Plot);
        }
        if def.info.is("upgrade") {
            // PlayUpgradeAction resolves its attach target before paying; there is
            // no cancel once the triggered ability resolves, so the attach prompt
            // has no Cancel (AbilityResolver canCancel = false from PlotAbility).
            s.source = id;
            s.plot.stage = PlotState::ATTACH;
            s.decision = Decision::UpgradeTarget;
            return Ok(());
        }
        s.card_mut(id).exhausted = !self.self_static_ongoing(
            s,
            id,
            crate::static_ongoing::SelfEffect::EntersPlayReady,
        );
        s.card_mut(id).set_hidden_qualified(true);
        self.add_arena(s, id)?;
        s.phase_history.entered(seat, id);
        s.phase_history.played(seat, id)?;
        s.source = id;
        self.plot_settle_enter(s, None)
    }

    /// Completes a Plot play: the engine's `ifYouDo` resources the top card of
    /// the deck, then the played card's own triggers resolve (its play event
    /// window passed them to the Plot ability window), then the suspended
    /// leader-deploy window resumes. A unique replacement choice resolves before
    /// the `ifYouDo` in the engine, with its triggers captured beforehand.
    pub(super) fn plot_settle_enter(
        &self,
        s: &mut State,
        unique_defeat: Option<CardId>,
    ) -> Result<(), String> {
        let seat = s.active;
        s.decision = Decision::Window;
        s.disabled_ongoing_source = unique_defeat.unwrap_or(0);
        self.capture_enter_triggers(s);
        s.disabled_ongoing_source = 0;
        if let Some(id) = unique_defeat {
            self.defeat(s, id)?;
        }
        self.sweep_stat_defeats(s)?;
        self.resource_top_of_deck(s, seat)?;
        s.plot.stage = PlotState::INNER;
        self.process_triggers(s)
    }

    /// After the played card's enter triggers finish, restore exactly the leader
    /// window that was suspended when the Plot trigger was chosen.
    pub(super) fn resume_leader_window(&self, s: &mut State) -> Result<(), String> {
        let plot = s.plot;
        // Keep the window marker and the resolved-instance set: playing one plot
        // card does not re-offer it, and the window may still hold other plots or
        // the leader's own deployed keywords.
        s.plot.card = 0;
        s.plot.stage = 0;
        s.plot.leader_pending = 0;
        s.plot.leader_programs = 0;
        s.source = plot.leader;
        s.set_trigger_context(TriggerContext::LeaderDeploy);
        s.pending_triggers = plot.leader_pending;
        s.pending_programs = plot.leader_programs;
        self.process_all_triggers(s)
    }

    /// Smuggle play from the resource zone. Units and upgrades attach/enter
    /// before the replacement (the replacement event is a sibling of the play
    /// events), so their When Played triggers resolve after it. Events resolve
    /// their effect after payment, then the replacement, then triggers — this
    /// implementation pays and moves the event before beginning its program.
    pub(super) fn begin_smuggle_play(
        &self,
        s: &mut State,
        seat: Seat,
        id: CardId,
    ) -> Result<(), String> {
        if !self.smuggle_playable(s, id) {
            return Err("illegal Smuggle play".into());
        }
        let def = self.def(s, id);
        let cost = self.smuggle_cost(s, id).ok_or("missing Smuggle cost")?;
        if def.info.is("upgrade") {
            // The play action resolves its attach target before paying.
            s.source = id;
            s.decision = Decision::UpgradeTarget;
            return Ok(());
        }
        s.phase_history.preflight_play(seat)?;
        self.exhaust_played_resource(s, seat, cost, id);
        s.player_mut(seat).resources.remove(id);
        if def.info.is("event") {
            s.player_mut(seat).discard.push(id)?;
            s.phase_history.played(seat, id)?;
            self.resource_top_of_deck(s, seat)?;
            s.source = id;
            s.set_trigger_context(TriggerContext::Action);
            let programs = self.available_programs(s, id, "event");
            let program = *programs
                .first()
                .ok_or("Smuggle event has no executable ability")?;
            return self.begin_program(s, program, ProgramContinuation::Finish);
        }
        s.card_mut(id).exhausted = !self.self_static_ongoing(
            s,
            id,
            crate::static_ongoing::SelfEffect::EntersPlayReady,
        );
        s.card_mut(id).set_hidden_qualified(true);
        self.add_arena(s, id)?;
        s.phase_history.entered(seat, id);
        s.phase_history.played(seat, id)?;
        self.resource_top_of_deck(s, seat)?;
        s.source = id;
        s.set_trigger_context(TriggerContext::Enter);
        if self.def(s, id).info.unique && self.duplicates(s, id) > 1 {
            s.decision = Decision::Unique;
            Ok(())
        } else {
            self.enter_triggers(s)
        }
    }
}
