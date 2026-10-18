//! Bounded first-strike and non-play-zone movement mechanisms.
//!
//! Sources are the pinned AttackFlow, ResourceCardSystem and MoveCardSystem.
//! First strike uses separate settled event windows; defender damage is live.
use super::*;

impl Engine {
    /// First attacker damage window. AttackComplete belongs to the second
    /// window, so attack-end registrations are deliberately not captured here.
    pub(in crate::simulator) fn combat_first_window(&self, s: &mut State) -> Result<(), String> {
        let source = s.attack_source;
        let target = s.attack_defender;
        if source == 0 || target == 0 || !self.in_arena(s, source) || !self.in_arena(s, target) {
            return Err("first-strike window requires live attack participants".into());
        }
        let attack = self.combat_damage(s, source, true);
        let before = s.card(target).damage;
        let remaining = self.hp(s, target).saturating_sub(before);
        // A Shield cancels the combat-damage parent and its Overwhelm contingent.
        let excess =
            if self.keyword_amount(s, source, "overwhelm") > 0 && s.card(target).shield == 0 {
                attack.saturating_sub(remaining)
            } else {
                0
            };
        self.damage(s, target, attack);
        if excess > 0 {
            let base = s.player(s.card(target).owner).base;
            self.damage(s, base, excess);
        }
        // Primary damage defeat precedes the framework stat sweep. Capture the
        // complete framework lethal set before any sibling leaves play.
        let mut dead = vec![];
        if s.card(target).damage > before && s.card(target).damage >= self.hp(s, target) {
            dead.push(target);
        }
        for id in self.arena_units(s) {
            if s.card(id).damage >= self.hp(s, id) && !dead.contains(&id) {
                dead.push(id);
            }
        }
        self.defeat_many(s, &dead)?;
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        if self.open_deaths(s, DeathContinuation::CombatAfterFirstDamage)? {
            return Ok(());
        }
        self.combat_after_first_window(s)
    }

    /// Generate defender damage only after the first event window and all its
    /// triggered abilities settle. This is a live power read (notably Grit).
    pub(in crate::simulator) fn combat_after_first_window(
        &self,
        s: &mut State,
    ) -> Result<(), String> {
        let source = s.attack_source;
        let target = s.attack_defender;
        if source == 0 || target == 0 {
            return Err("first-strike continuation lost attack context".into());
        }
        self.capture_attack_end(s)?;
        let before = s.card(source).damage;
        if self.in_arena(s, target) && self.in_arena(s, source) {
            let defend = self.combat_damage(s, target, false);
            self.damage(s, source, defend);
        }
        let mut dead = vec![];
        if self.in_arena(s, source)
            && s.card(source).damage > before
            && s.card(source).damage >= self.hp(s, source)
        {
            dead.push(source);
        }
        for id in self.arena_units(s) {
            if s.card(id).damage >= self.hp(s, id) && !dead.contains(&id) {
                dead.push(id);
            }
        }
        self.capture_advantage_end(s)?;
        self.defeat_many(s, &dead)?;
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        if self.open_deaths(s, DeathContinuation::CombatAfterDamage)? {
            return Ok(());
        }
        self.combat_after_damage(s)
    }

    /// First-strike state is either a live reviewed constant registration or an
    /// until-end-of-attack grant captured before defender selection. Both are
    /// attacker-only: the admitted source shapes all require the unit to be
    /// attacking, so a defender with this ability is never admitted.
    pub(in crate::simulator) fn deals_combat_damage_first(&self, s: &State, id: CardId) -> bool {
        if s.support.grants[..s.support.len as usize]
            .iter()
            .any(|g| g.target == id && g.deals_combat_damage_first)
        {
            return true;
        }
        self.constant_effects_matching(s, id, ConstantScope::CombatDamageFirst)
            .iter()
            .any(|(_, e)| matches!(e, Effect::DealsCombatDamageFirst { target: Target::Name(n), duration } if n == "self" && duration == "persistent"))
    }

    /// A reviewed selected card entering the owner's resource zone (hand or
    /// discard source shapes). Resource entry exhausts it.
    pub(in crate::simulator) fn resource_card_eligible(&self, s: &State, id: CardId) -> bool {
        if id == 0 || !s.card(id).present {
            return false;
        }
        let info = &self.def(s, id).info;
        if info.is("leader")
            || info.is("token")
            || info.is("base")
            || !(info.is("unit") || info.is("event") || info.is("upgrade"))
        {
            return false;
        }
        let owner = s.card(id).owner;
        if s.player(owner).resources.contains(id) {
            return false;
        }
        s.player(owner).hand.contains(id) || s.player(owner).discard.contains(id)
    }
    pub(in crate::simulator) fn resource_card_card(
        &self,
        s: &mut State,
        id: CardId,
    ) -> Result<u16, String> {
        if !self.resource_card_eligible(s, id) {
            return Ok(0);
        }
        let owner = s.card(id).owner;
        if s.player(owner).resources.len as usize >= crate::model::MAX_ZONE {
            return Err("resource zone capacity exceeded".into());
        }
        let p = s.player_mut(owner);
        p.hand.remove(id);
        p.discard.remove(id);
        p.resources.push(id)?;
        s.card_mut(id).exhausted = true;
        Ok(1)
    }

    /// Move one admitted card to the bottom of its owner's deck. The admission
    /// gate restricts this effect to a single chosen card from a hand or discard
    /// pile; a play-zone move would need the pinned engine's leaving-play window
    /// and stays an explicit runtime error rather than an approximation.
    pub(in crate::simulator) fn move_to_bottom_of_deck(
        &self,
        s: &mut State,
        id: CardId,
    ) -> Result<u16, String> {
        if id == 0 || !s.card(id).present {
            return Ok(0);
        }
        if self.in_arena(s, id) || self.attached_in_play(s, id) {
            return Err("moveToBottomOfDeck requires a card outside the play zone".into());
        }
        if !self.move_to_bottom_eligible(s, id) {
            return Ok(0);
        }
        let owner = s.card(id).owner;
        if s.player(owner).deck.len as usize >= crate::model::MAX_ZONE {
            return Err("deck capacity exceeded".into());
        }
        let p = s.player_mut(owner);
        p.hand.remove(id);
        p.discard.remove(id);
        p.deck.push(id)?;
        Ok(1)
    }
    /// Reviewed ResourceCardSystem target domain. The acting player's own
    /// resource zone only: the source card itself (e.g. a defeated unit in its
    /// discard) or the acting player's top deck card. Foreign resource zones
    /// would model take-control and stay rejected.
    pub(in crate::simulator) fn resource_card_system_card(
        &self,
        s: &State,
        source: CardId,
        target: &Target,
        target_player: &str,
    ) -> Result<Option<CardId>, String> {
        if target_player != "self" {
            return Err(
                "resourceCardSystem supports only the acting player's resource zone".into(),
            );
        }
        let owner = s.card(source).owner;
        match target {
            Target::Name(n) if n == "self" => Ok(Some(source)),
            Target::Expression(_) if crate::ability_ir::resource_card_system_target(target) => {
                Ok(s.player(owner).deck.as_slice().first().copied())
            }
            _ => Err("unreviewed resourceCardSystem target".into()),
        }
    }
    pub(in crate::simulator) fn resource_card_system_eligible(
        &self,
        s: &State,
        id: CardId,
    ) -> bool {
        if id == 0 || !s.card(id).present {
            return false;
        }
        let info = &self.def(s, id).info;
        if info.is("leader") || info.is("token") || info.is("base") {
            return false;
        }
        let owner = s.card(id).owner;
        if s.player(owner).resources.contains(id) {
            return false;
        }
        s.player(owner).hand.contains(id)
            || s.player(owner).deck.contains(id)
            || s.player(owner).discard.contains(id)
    }
    /// Resource entry exhausts the card; the contingent ready event (when
    /// requested) readies it afterwards. The ability source keeps its identity.
    pub(in crate::simulator) fn apply_resource_card_system(
        &self,
        s: &mut State,
        target: &Target,
        target_player: &str,
        ready: bool,
    ) -> Result<(u16, bool), String> {
        let Some(id) = self.resource_card_system_card(s, s.source, target, target_player)? else {
            return Ok((0, false));
        };
        if !self.resource_card_system_eligible(s, id) {
            return Ok((0, false));
        }
        let card_owner = s.card(id).owner;
        if s.player(card_owner).resources.len as usize >= crate::model::MAX_ZONE {
            return Err("resource zone capacity exceeded".into());
        }
        let p = s.player_mut(card_owner);
        p.hand.remove(id);
        p.deck.remove(id);
        p.discard.remove(id);
        p.resources.push(id)?;
        // ResourceCardSystem's primary move initializes exhaustion; its
        // contingent ReadySystem resolves before framework trigger processing.
        s.card_mut(id).exhausted = true;
        if ready {
            self.set_card_exhausted(s, id, false);
        }
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        Ok((1, true))
    }
    pub(in crate::simulator) fn move_to_bottom_eligible(&self, s: &State, id: CardId) -> bool {
        id != 0 && s.card(id).present && !self.in_arena(s, id) && !self.attached_in_play(s, id) && {
            let owner = s.card(id).owner;
            s.player(owner).hand.contains(id) || s.player(owner).discard.contains(id)
        }
    }
}
