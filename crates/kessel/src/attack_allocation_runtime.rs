//! A bounded position certificate for attacks which can emit the new friendly
//! base-record allocation. This is a read-only causal event walk, not a search
//! through decisions. It reserves no identities and changes no legal menus.
//!
//! The initiation owner calls before costs/early effects. The proof covers its
//! whole programme, the nested attack, and the suffix which resumes afterwards.
//! Independent emissions add; mutually exclusive attackers/defenders take max.
//! Unknown shapes/cycles are explicit unsupported positions, never zero effects.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum UnsupportedAttackPosition {
    Uncertified(&'static str),
    Capacity { required: usize, available: usize },
}
impl std::fmt::Display for UnsupportedAttackPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncertified(why)=>write!(f,"unsupported attack position: uncertified allocation closure ({why})"),
            Self::Capacity{required,available}=>write!(f,"unsupported attack position: allocation demand {required} exceeds {available} available card slots"),
        }
    }
}
type Proof<T> = Result<T, UnsupportedAttackPosition>;
fn unknown<T>(why: &'static str) -> Proof<T> {
    Err(UnsupportedAttackPosition::Uncertified(why))
}
fn plus(a: usize, b: usize) -> Proof<usize> {
    a.checked_add(b)
        .ok_or(UnsupportedAttackPosition::Uncertified(
            "allocation arithmetic overflow",
        ))
}
fn times(a: usize, b: usize) -> Proof<usize> {
    a.checked_mul(b)
        .ok_or(UnsupportedAttackPosition::Uncertified(
            "allocation arithmetic overflow",
        ))
}

struct Causal<'a> {
    engine: &'a Engine,
    state: &'a State,
    // A recursion-stack detects a causal cycle. It is deliberately not a global
    // visited set: repeated independent events must pay for every occurrence.
    stack: Vec<(CardId, u16)>,
    work: usize,
    entries: usize,
}
impl<'a> Causal<'a> {
    fn new(engine: &'a Engine, state: &'a State) -> Self {
        Self {
            engine,
            state,
            stack: vec![],
            work: 0,
            entries: 0,
        }
    }
    fn tick(&mut self) -> Proof<()> {
        self.work += 1;
        if self.work > 8192 {
            return unknown("causal summary work budget");
        }
        Ok(())
    }
    fn literal(&self, e: &Expr) -> Proof<usize> {
        match e.original() {
            Expr::Const { value } => value.as_u64().and_then(|n| usize::try_from(n).ok()).ok_or(
                UnsupportedAttackPosition::Uncertified("nonnegative literal allocation amount"),
            ),
            _ => unknown("dynamic allocation/event multiplicity"),
        }
    }
    fn roster(&mut self, roster: Vec<(CardId, u16)>) -> Proof<usize> {
        let mut total = 0;
        for (source, program) in roster {
            total = plus(total, self.program(source, program, 0)?)?;
        }
        Ok(total)
    }
    fn program(&mut self, source: CardId, program: u16, start: usize) -> Proof<usize> {
        self.tick()?;
        if self.stack.contains(&(source, program)) {
            return unknown("causal programme cycle");
        }
        if self.stack.len() >= MAX_DEATH_DEPTH {
            return unknown("causal nesting depth");
        }
        self.stack.push((source, program));
        let result = self.instructions(source, program, start);
        self.stack.pop();
        result
    }
    fn instructions(&mut self, source: CardId, program: u16, start: usize) -> Proof<usize> {
        let p = self.engine.program(program);
        let mut selected = usize::from(self.state.ability_selection.len.max(1));
        let mut total = 0;
        // Walking all forward conditional/optional arms is conservative. Jumps
        // never cause a second execution here; executable IR has no backedges.
        for instruction in p.instructions.iter().skip(start) {
            self.tick()?;
            match instruction {
                Instruction::Select(r) => {
                    selected = r.max_targets().map_err(|_| {
                        UnsupportedAttackPosition::Uncertified("selector multiplicity")
                    })?;
                }
                Instruction::Apply(effect)
                | Instruction::Group(effect)
                | Instruction::NamedGroup(effect)
                | Instruction::NamedExhaustContingent(effect)
                | Instruction::NamedChooserGroup { effect, .. } => {
                    total = plus(total, self.effect(source, effect, selected)?)?;
                }
                Instruction::GrantCreatedTokenAdvantage { .. } => {
                    total = plus(total, plus(1, self.framework()?)?)?;
                }
                Instruction::GrantCreatedTokenPhaseKeyword { .. } => {
                    total = plus(total, self.framework()?)?;
                }
                Instruction::EndStep
                | Instruction::Commit
                | Instruction::Conditional { .. }
                | Instruction::IfResult { .. }
                | Instruction::IfWillDefeat { .. }
                | Instruction::IfDiscardedSharesAspect { .. }
                | Instruction::IfDiscardedIsUnit { .. }
                | Instruction::OptionalConfirm { .. }
                | Instruction::Jump { .. } => {}
                _ => return unknown("instruction/event shape"),
            }
        }
        Ok(total)
    }
    fn targets(&self, source: CardId, target: &Target) -> Vec<CardId> {
        match target {
            Target::Name(n) if n == "self" => vec![source],
            Target::Name(n) if n == "ownBase" => {
                vec![self.state.player(self.state.card(source).owner).base]
            }
            Target::Name(n) if n == "enemyBase" => vec![
                self.state
                    .player(self.state.card(source).owner.other())
                    .base,
            ],
            Target::Name(n) if n == "bothBases" => vec![
                self.state.player(Seat::P1).base,
                self.state.player(Seat::P2).base,
            ],
            // A selector may change eligibility after earlier effects. All arena
            // identities are a sound recipient superset; do not freeze its menu.
            _ => {
                let selectors = self
                    .stack
                    .last()
                    .map(|&(_, p)| {
                        self.engine
                            .program(p)
                            .instructions
                            .iter()
                            .filter_map(|i| {
                                if let Instruction::Select(r) = i {
                                    Some(r)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let units_only =
                    !selectors.is_empty() && selectors.iter().all(|r| r.card_type == "unit");
                let bases_only =
                    !selectors.is_empty() && selectors.iter().all(|r| r.card_type == "base");
                let mut ids = if bases_only {
                    vec![]
                } else {
                    self.engine.arena_units(self.state)
                };
                if !units_only {
                    ids.extend([
                        self.state.player(Seat::P1).base,
                        self.state.player(Seat::P2).base,
                    ]);
                }
                ids
            }
        }
    }
    fn multiplicity(&self, target: &Target, selected: usize) -> Proof<usize> {
        match target {
            Target::Name(n) if n == "chosen" => Ok(selected),
            Target::Name(n) if n == "bothBases" || n == "bothPlayers" => Ok(2),
            Target::Name(_) | Target::Expression(_) => Ok(1),
            Target::ArenaList(_) => unknown("arena allocation multiplicity across entry effects"),
            Target::PlayerList(_) => Ok(2),
        }
    }
    fn attachments(
        &mut self,
        source: CardId,
        target: &Target,
        amount: usize,
        selected: usize,
    ) -> Proof<usize> {
        let count = times(amount, self.multiplicity(target, selected)?)?;
        if count == 0 {
            return Ok(0);
        }
        let mut peer = 0;
        for host in self.targets(source, target) {
            let roster = self
                .engine
                .card_programs(self.state, host)
                .iter()
                .copied()
                .filter(|&p| self.engine.program(p).trigger == "onUpgradeAttached")
                .map(|p| (host, p))
                .collect();
            peer = peer.max(self.roster(roster)?);
        }
        plus(plus(count, times(count, peer)?)?, self.framework()?)
    }
    fn constants(&self) -> Vec<(CardId, u16)> {
        let e = self.engine;
        let s = self.state;
        let mut out = vec![];
        for source in 1..s.next_id {
            let basezone = s
                .players
                .iter()
                .any(|p| p.base == source || (p.leader == source && !s.card(source).deployed));
            if !(e.in_arena(s, source) || e.attached_in_play(s, source) || basezone) {
                continue;
            }
            out.extend(
                e.card_programs(s, source)
                    .iter()
                    .copied()
                    .chain(e.support_programs(s, source))
                    .filter(|&p| e.program(p).trigger == "constant")
                    .map(|p| (source, p)),
            );
        }
        out
    }
    fn stat_aura(&self) -> bool {
        self.constants().iter().any(|&(_, p)| {
            self.engine.program(p).instructions.iter().any(|i| match i {
                Instruction::Apply(Effect::ModifyStats { hp, .. }) => {
                    !matches!(hp.original(),Expr::Const{value} if value.as_i64()==Some(0))
                }
                _ => false,
            })
        })
    }
    fn framework(&mut self) -> Proof<usize> {
        if !self.stat_aura() {
            return Ok(0);
        }
        let mut total = 0;
        for id in self.engine.arena_units(self.state) {
            total = plus(total, self.defeated(id, true)?)?;
        }
        Ok(total)
    }
    fn defeated(&mut self, id: CardId, ability: bool) -> Proof<usize> {
        self.tick()?;
        if !self.engine.in_arena(self.state, id) {
            return Ok(0);
        }
        let e = self.engine;
        let s = self.state;
        let mut roster = e
            .card_programs(s, id)
            .iter()
            .copied()
            .filter(|&p| !e.program(p).attached_host && e.program(p).trigger == "whenDefeated")
            .map(|p| (id, p))
            .collect::<Vec<_>>();
        roster.extend(
            e.attached_host_grants(s, id, "whenDefeated")
                .into_iter()
                .map(|g| (id, g.program)),
        );
        let token = e.def(s, id).info.is("token");
        for owner in [Seat::P1, Seat::P2] {
            for source in e.arena_units(s) {
                if s.card(source).owner != owner {
                    continue;
                }
                for &program in e.card_programs(s, source) {
                    if e.program(program).hooks.iter().any(|hook|matches!(hook,crate::death_hooks::Hook::Event(h) if matches!(h.event,crate::death_hooks::EventName::Defeated)&&h.predicate.matches(token,owner==s.card(id).owner,source==id))) {roster.push((source,program));}
                }
            }
        }
        // Round-limited listeners still emit at most once for this event. Their
        // finite ledger cannot increase this conservative per-event demand.
        for (source, program, registration) in e
            .round_limit_roster(s)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("round-limit roster"))?
        {
            // Weakness can be attached by an earlier peer callback. Both values
            // are possible here; count one occurrence when either qualifies.
            let mut qualifies = false;
            for had_weakness in [false, true] {
                let event = crate::round_limit_runtime::Event::Defeated {
                    card: id,
                    controller: s.card(id).owner.index() as u8,
                    was_unit: true,
                    unique: e.def(s, id).info.unique,
                    had_weakness,
                };
                qualifies |= crate::round_limit_runtime::qualifies(
                    registration,
                    source,
                    s.card(source).owner.index() as u8,
                    event,
                    true,
                )
                .map_err(|_| {
                    UnsupportedAttackPosition::Uncertified("defeat observer projection")
                })?;
            }
            if qualifies {
                roster.push((source, program));
            }
        }
        let _ = ability; // Conditions/cause gates are conservatively included.
        self.roster(roster)
    }
    fn unit_damage_observers(&mut self, id: CardId) -> Proof<usize> {
        let mut roster = vec![];
        for (observer, program, registration) in self
            .engine
            .round_limit_roster(self.state)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("round-limit roster"))?
        {
            let event = crate::round_limit_runtime::Event::UnitDamage {
                controller: self.state.card(id).owner.index() as u8,
                card_is_unit: true,
                will_defeat: false,
            };
            if crate::round_limit_runtime::qualifies(
                registration,
                observer,
                self.state.card(observer).owner.index() as u8,
                event,
                true,
            )
            .map_err(|_| UnsupportedAttackPosition::Uncertified("damage observer projection"))?
            {
                roster.push((observer, program));
            }
        }
        self.roster(roster)
    }
    fn damage(&mut self, source: CardId, target: &Target) -> Proof<usize> {
        let mut total = 0;
        let mut possible = self.targets(source, target);
        if self.stat_aura()
            && possible
                .iter()
                .any(|&id| self.engine.in_arena(self.state, id))
        {
            let bases = possible
                .iter()
                .copied()
                .filter(|&id| self.engine.def(self.state, id).info.is("base"))
                .collect::<Vec<_>>();
            possible = self.engine.arena_units(self.state);
            possible.extend(bases);
        }
        for id in possible {
            if self.engine.def(self.state, id).info.is("base") {
                total = plus(total, self.base_damage(self.state.card(id).owner)?)?;
            }
            total = plus(total, self.defeated(id, true)?)?;
            if self.engine.in_arena(self.state, id) {
                total = plus(total, self.unit_damage_observers(id)?)?;
            }
        }
        Ok(total)
    }
    fn base_damage(&mut self, owner: Seat) -> Proof<usize> {
        let e = self.engine;
        let s = self.state;
        let mut roster = vec![];
        if let Some(catalogue) = e.catalogue.as_ref() {
            for source in e.arena_units(s) {
                if s.card(source).owner != owner {
                    continue;
                }
                let d = e.def(s, source);
                if !d.programs.unit.is_empty()
                    || !d.programs.leader.is_empty()
                    || d.programs.attach_condition.is_some()
                {
                    continue;
                }
                if let Some(card) = catalogue.metadata.get(&d.code) {
                    let programs = d
                        .programs
                        .card
                        .iter()
                        .map(|&p| e.program(p))
                        .collect::<Vec<_>>();
                    if crate::blade_base_damage_contract::compiled_source(card, &programs) {
                        roster.push((source, d.programs.card[0]));
                    }
                }
            }
        }
        plus(self.roster(roster)?, self.framework()?)
    }
    fn draw(&mut self, source: CardId, target: &Target) -> Proof<usize> {
        let owner = self.state.card(source).owner;
        let players = match target {
            Target::Name(n) if n == "ownPlayer" => vec![owner],
            Target::Name(n) if n == "enemyPlayer" => vec![owner.other()],
            _ => vec![Seat::P1, Seat::P2],
        };
        let events = players
            .iter()
            .copied()
            .map(CollectiveEvent::CardsDrawn)
            .collect::<Vec<_>>();
        let captures = self
            .engine
            .collective_captures(self.state, &events)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("draw observer roster"))?;
        let mut total = self.roster(
            captures
                .into_iter()
                .map(|t| (t.source, t.program))
                .collect(),
        )?;
        // A reachable discard, top-deck resource move, or an independently
        // ordered peer can deplete the deck before this Draw resolves. Initial
        // deck length (even minus earlier Draws in this walk) is not a lower
        // bound at that event. Conservatively include its possible fatigue arm.
        // Pinned DrawSystem creates ONE contingent player-base damage event per
        // positive Draw/player, with 3*missing damage; not one event per card.
        for player in players {
            total = plus(total, self.base_damage(player)?)?;
        }
        plus(total, self.framework()?)
    }
    fn create_units(
        &mut self,
        source: CardId,
        token_type: &str,
        amount: usize,
        target: &Target,
    ) -> Proof<usize> {
        self.entries = plus(self.entries, amount)?;
        let code = token_unit_code(token_type)
            .ok_or(UnsupportedAttackPosition::Uncertified("native unit type"))?;
        let def = *self
            .engine
            .by_code
            .get(code)
            .ok_or(UnsupportedAttackPosition::Uncertified(
                "native unit unavailable",
            ))?;
        let owner = self.state.card(source).owner;
        let owner = match target {
            Target::Name(n) if n == "ownPlayer" => owner,
            Target::Name(n) if n == "enemyPlayer" => owner.other(),
            _ => return unknown("unit owner domain"),
        };
        let d = &self.engine.definitions[def as usize];
        if !d.programs.card.is_empty() || !d.programs.unit.is_empty() {
            return unknown("created unit programmes");
        }
        // Shielded is boolean and fires at most once per entered native unit.
        // Include a possible effective grant conservatively, even if its live
        // conditional/trait gate would not match this particular token.
        let constants = self.constants();
        let shielded=d.flags & SHIELDED != 0 || constants.iter().any(|&(_,p)|self.engine.program(p).instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::GainKeyword{keyword,..}) if keyword=="shielded")||matches!(i,Instruction::Apply(Effect::CopyKeywords{..}))));
        if d.flags & (AMBUSH | SUPPORT) != 0 {
            return unknown("created native attack/Support entry");
        }
        if constants.iter().any(|&(_,p)|self.engine.program(p).instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::GainKeyword{keyword,..}) if keyword=="ambush"||keyword=="support")||matches!(i,Instruction::Apply(Effect::CopyKeywords{..}|Effect::ConstantEffectRegistration(_))))) {return unknown("effective native entry attack/Support registration");}
        let mut per = 1 + usize::from(shielded);
        let mut roster = vec![];
        for observer in self.engine.arena_units(self.state) {
            if self.state.card(observer).owner == owner {
                for &p in self.engine.card_programs(self.state, observer) {
                    if self.engine.program(p).trigger == "onUnitEntersPlay" {
                        roster.push((observer, p));
                    }
                }
            }
        }
        per = plus(per, self.roster(roster)?)?;
        // New native bodies can subsequently be defeated by queued effects.
        // Include each matching token-death observer once per created body.
        let mut roster = vec![];
        for observer in self.engine.arena_units(self.state) {
            for &p in self.engine.card_programs(self.state, observer) {
                if self.engine.program(p).hooks.iter().any(|hook|matches!(hook,crate::death_hooks::Hook::Event(h) if matches!(h.event,crate::death_hooks::EventName::Defeated)&&h.predicate.matches(true,self.state.card(observer).owner==owner,false))) {roster.push((observer,p));}
            }
        }
        for (observer, program, registration) in self
            .engine
            .round_limit_roster(self.state)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("native defeat observer roster"))?
        {
            // All seven native unit profiles are nonunique. A peer can attach
            // Weakness before a later defeat, so include that possible event.
            let event = crate::round_limit_runtime::Event::Defeated {
                card: self.state.next_id.min(255),
                controller: owner.index() as u8,
                was_unit: true,
                unique: false,
                had_weakness: true,
            };
            if crate::round_limit_runtime::qualifies(
                registration,
                observer,
                self.state.card(observer).owner.index() as u8,
                event,
                true,
            )
            .map_err(|_| {
                UnsupportedAttackPosition::Uncertified("native defeat observer projection")
            })? {
                roster.push((observer, program));
            }
        }
        per = plus(per, self.roster(roster)?)?;
        plus(times(amount, per)?, self.framework()?)
    }
    fn effect(&mut self, source: CardId, effect: &Effect, selected: usize) -> Proof<usize> {
        self.tick()?;
        match effect {
            Effect::GiveShield { amount, target }
            | Effect::GiveExperience { amount, target }
            | Effect::GiveAdvantage { amount, target, .. } => {
                self.attachments(source, target, self.literal(amount)?, selected)
            }
            Effect::GiveWeakness { amount, target } => plus(
                self.attachments(source, target, self.literal(amount)?, selected)?,
                self.damage(source, target)?,
            ),
            Effect::CreateTokenUnit {
                token_type,
                amount,
                target,
                ..
            } => self.create_units(source, token_type, self.literal(amount)?, target),
            Effect::CreateCreditToken { amount, target, .. } => plus(
                times(self.literal(amount)?, self.multiplicity(target, selected)?)?,
                self.framework()?,
            ),
            Effect::Draw { amount, target } => {
                if self.literal(amount)? == 0 {
                    Ok(0)
                } else {
                    self.draw(source, target)
                }
            }
            Effect::Damage { target, amount } => {
                if matches!(amount.original(),Expr::Const{value} if value.as_i64()==Some(0)) {
                    Ok(0)
                } else {
                    self.damage(source, target)
                }
            }
            Effect::Defeat { target } => self.damage(source, target),
            Effect::DistributeDamage { .. } => self.damage(source, &Target::name("chosen")),
            Effect::DistributeTokenUpgrade { amount, .. } => {
                self.attachments(source, &Target::name("chosen"), self.literal(amount)?, 1)
            }
            Effect::Sequence { effects }
            | Effect::Simultaneous { effects }
            | Effect::PhaseModifiers { effects }
            | Effect::AttackModifiers { effects } => {
                let mut total = 0;
                for child in effects {
                    total = plus(total, self.effect(source, child, selected)?)?;
                }
                Ok(total)
            }
            Effect::Conditional {
                on_true, on_false, ..
            } => {
                let yes = self.effect(source, on_true, selected)?;
                let no = if let Some(e) = on_false {
                    self.effect(source, e, selected)?
                } else {
                    0
                };
                plus(yes, no)
            }
            Effect::Select {
                target_resolver,
                effect,
            } => self.effect(
                source,
                effect,
                target_resolver.max_targets().map_err(|_| {
                    UnsupportedAttackPosition::Uncertified("nested selector multiplicity")
                })?,
            ),
            Effect::Optional { effect, .. } => self.effect(source, effect, selected),
            Effect::SelectionSystem { inner_system, .. } => {
                self.effect(source, inner_system, selected)
            }
            Effect::InitiateAttack { properties, .. } => self.native_attack(source, properties),
            // These operations cannot allocate or introduce an ordinary unit.
            // A modifier/removal can cause framework defeats; include their LKI
            // registrations without assuming a recipient survives the attack.
            Effect::ModifyStats { target, .. }
            | Effect::GainKeyword { target, .. }
            | Effect::LoseKeyword { target, .. }
            | Effect::CopyKeywords { target, .. }
            | Effect::ReturnToHand { target }
            | Effect::MoveSelectedArenaUnitToOwnerDeck { target, .. } => {
                self.damage(source, target)
            }
            Effect::Heal { .. }
            | Effect::RecipientHeal { .. }
            | Effect::DistributeHealing { .. }
            | Effect::UseForce { .. }
            | Effect::GainForce { .. }
            | Effect::Ready { .. }
            | Effect::Exhaust { .. }
            | Effect::ReadyResources { .. }
            | Effect::ExhaustResources { .. }
            | Effect::PayResources { .. }
            | Effect::PayAdjustedResources { .. }
            | Effect::DiscardOwnHand { .. }
            | Effect::DiscardDeck { .. }
            | Effect::DiscardSpecificCard { .. }
            | Effect::ResourceCard { .. }
            | Effect::ResourceCardSystem { .. } => self.framework(),
            Effect::NoAction
            | Effect::InstallPlayerPhase { .. }
            | Effect::MoveToBottomOfDeck { .. }
            | Effect::DealsCombatDamageFirst { .. }
            | Effect::AttackPermission { .. }
            | Effect::CardRestriction { .. }
            | Effect::LookAt { .. } => Ok(0),
            _ => unknown("effect/event shape"),
        }
    }
    fn native_attack(
        &mut self,
        source: CardId,
        p: &crate::native_attack::Properties,
    ) -> Proof<usize> {
        if self.entries != 0 {
            return unknown("prior native entry changes attack selection");
        }
        p.modifiers()
            .map_err(|_| UnsupportedAttackPosition::Uncertified("attack modifiers"))?;
        p.conditional_power()
            .map_err(|_| UnsupportedAttackPosition::Uncertified("conditional attack modifiers"))?;
        let candidates = if self.state.ability_target != 0 {
            vec![self.state.ability_target]
        } else {
            self.engine.arena_units(self.state)
        };
        let mut max = 0;
        for attacker in candidates {
            if self
                .engine
                .native_attack_eligible(self.state, source, attacker, p)
            {
                // Early handlers can remove a Sentinel or change its keyword.
                // Include same-arena physical alternatives as well as the real
                // original legal set; never charge alternatives cumulatively.
                let s = self.state;
                let owner = s.card(attacker).owner;
                let ground = s.player(owner).ground.contains(attacker);
                let mut defenders = self.engine.attack_targets(s, attacker, false);
                for id in self.engine.arena_units(s) {
                    if s.card(id).owner != owner
                        && s.player(s.card(id).owner).ground.contains(id) == ground
                        && !defenders.contains(&id)
                    {
                        defenders.push(id);
                    }
                }
                let base = s.player(owner.other()).base;
                if !defenders.contains(&base) {
                    defenders.push(base);
                }
                for defender in defenders {
                    max = max.max(self.attack(attacker, defender)?);
                }
            }
        }
        Ok(max)
    }
    fn attack(&mut self, attacker: CardId, defender: CardId) -> Proof<usize> {
        self.tick()?;
        let e = self.engine;
        let s = self.state;
        let mut total = self.framework()?;
        let mut on_attack = e
            .card_programs(s, attacker)
            .iter()
            .copied()
            .chain(e.support_programs(s, attacker))
            .filter(|&p| e.program(p).trigger == "onAttack" && !e.program(p).attached_host)
            .map(|p| (attacker, p))
            .collect::<Vec<_>>();
        on_attack.extend(
            e.card_programs(s, defender)
                .iter()
                .copied()
                .filter(|&p| e.program(p).trigger == "onDefense" && !e.program(p).attached_host)
                .map(|p| (defender, p)),
        );
        for (host, trigger) in [(attacker, "onAttack"), (defender, "onDefense")] {
            for grantor in e.upgrades(s, host) {
                for &program in &e.def(s, grantor).programs.card {
                    let p = e.program(program);
                    if p.trigger == trigger
                        && (p.attached_host
                            || (trigger == "onAttack" && p.gained_host == Some(true)))
                    {
                        on_attack.push((host, program));
                    }
                }
            }
        }
        total = plus(total, self.roster(on_attack)?)?;
        // Shared PRE/POST occurrence multiset; no prediction from later damage.
        let roster = e
            .attack_base_end_potential(s, attacker, true)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("attack-end roster"))?;
        let owner = s.card(attacker).owner;
        let roster = roster
            .into_iter()
            .filter(|&(source, p)| {
                !is_friendly_allocation(e.program(p)) || s.card(source).owner == owner
            })
            .collect();
        total = plus(total, self.roster(roster)?)?;
        if e.in_arena(s, defender) {
            // Framework/aura deaths can include peers; no retirement frees IDs.
            let possible = if self.stat_aura() {
                e.arena_units(s)
            } else {
                vec![attacker, defender]
            };
            for id in possible {
                total = plus(total, self.defeated(id, false)?)?;
                total = plus(total, self.unit_damage_observers(id)?)?;
            }
        }
        total = plus(total, self.base_damage(owner.other())?)?;
        // Round-limit attack-declaration and damage/defeat observers may allocate.
        for (source, program, registration) in e
            .round_limit_roster(s)
            .map_err(|_| UnsupportedAttackPosition::Uncertified("round-limit roster"))?
        {
            let event = crate::round_limit_runtime::Event::AttackDeclared {
                attacker_controller: owner.index() as u8,
                ground_attacker: s.player(owner).ground.contains(attacker),
                any_base_target: e.def(s, defender).info.is("base"),
            };
            if crate::round_limit_runtime::qualifies(
                registration,
                source,
                s.card(source).owner.index() as u8,
                event,
                true,
            )
            .map_err(|_| UnsupportedAttackPosition::Uncertified("attack observer projection"))?
            {
                total = plus(total, self.program(source, program, 0)?)?;
            }
        }
        Ok(total)
    }
}
fn is_friendly_allocation(p: &Program) -> bool {
    matches!(
        p.condition.as_ref(),
        Some(Expr::AttackBaseQuery {
            query: crate::attack_base_history_contract::Query::FriendlyAttackerHasBaseRecord
        })
    )
}
impl Engine {
    fn allocation_listener_present(&self, s: &State) -> bool {
        // Current closed IR can enter only the seven native TokenUnit profiles;
        // none carries this registration. Ordinary unit plays are outside the
        // attack operation. Hidden Vane definitions alone cannot gain a listener.
        self.arena_units(s).into_iter().any(|id| {
            self.card_programs(s, id)
                .iter()
                .copied()
                .chain(self.support_programs(s, id))
                .any(|p| is_friendly_allocation(self.program(p)))
        })
    }
    fn check_attack_demand(&self, s: &State, demand: Proof<usize>) -> Result<(), String> {
        let required = demand.map_err(|e| e.to_string())?;
        let available = MAX_CARDS - usize::from(s.next_id);
        if required > available {
            return Err(UnsupportedAttackPosition::Capacity {
                required,
                available,
            }
            .to_string());
        }
        Ok(())
    }
    pub(in crate::simulator) fn preflight_attack_allocation(
        &self,
        s: &State,
        attacker: CardId,
        defender: CardId,
    ) -> Result<(), String> {
        if !self.allocation_listener_present(s) {
            return Ok(());
        }
        self.check_attack_demand(s, Causal::new(self, s).attack(attacker, defender))
    }
    pub(super) fn preflight_native_attack_allocation(
        &self,
        s: &State,
        p: &crate::native_attack::Properties,
    ) -> Result<(), String> {
        if !self.allocation_listener_present(s) {
            return Ok(());
        }
        self.check_attack_demand(s, Causal::new(self, s).native_attack(s.source, p))
    }
    pub(super) fn preflight_attack_parent_allocation(
        &self,
        s: &State,
        end_program: bool,
    ) -> Result<(), String> {
        if !self.allocation_listener_present(s) || s.ability_program == 0 {
            return Ok(());
        }
        let p = self.program(s.ability_program - 1);
        if !p
            .instructions
            .iter()
            .skip(s.ability_pc as usize)
            .any(|i| matches!(i, Instruction::Apply(Effect::InitiateAttack { .. })))
        {
            return Ok(());
        }
        let mut proof = Causal::new(self, s);
        let result = (|| {
            if p.costs.iter().any(|c| matches!(c, Cost::SelectCardCost(_))) {
                return unknown("discard-cost callbacks");
            }
            let mut demand = proof.framework()?;
            if self.def(s, s.source).info.is("event")
                && (s.player(s.card(s.source).owner).hand.contains(s.source)
                    || s.player(s.card(s.source).owner)
                        .resources
                        .contains(s.source))
            {
                // Capture-only fact collection on a private value, using the real
                // typed play/Smuggle predicates. No handler/cost/effect executes.
                let mut context = *s;
                let old = context.death_pending.len;
                self.capture_card_played_watchers(
                    &mut context,
                    s.player(s.card(s.source).owner)
                        .resources
                        .contains(s.source),
                )
                .map_err(|_| UnsupportedAttackPosition::Uncertified("play observer capture"))?;
                let roster = context.death_pending.entries
                    [old as usize..context.death_pending.len as usize]
                    .iter()
                    .map(|t| (t.source, t.program))
                    .collect();
                demand = plus(demand, proof.roster(roster)?)?;
            }
            // Play/Smuggle callbacks are the causal prefix. A native entry
            // there needs a projected defender contract, explicitly unsupported
            // here. Use Anyway commits costs only, so has no attack/suffix.
            if !end_program {
                demand = plus(
                    demand,
                    proof.program(s.source, s.ability_program - 1, s.ability_pc as usize)?,
                )?;
            }
            Ok(demand)
        })();
        self.check_attack_demand(s, result)
    }
}
