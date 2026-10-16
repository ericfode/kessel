// Pinned SupportAbility / InitiateAttackSystem / GainNonKeywordAbilitiesFromUnitEffect.
// Optional attack cancellation intentionally retains early UntilEndOfAttack effects:
// only an actual AttackFlow cleanup (or the target leaving play) removes them.
use super::*;
use crate::ability_ir::{Effect, Instruction, Program};

impl Engine {
    pub(super) fn support_programs(&self, s: &State, target: CardId) -> Vec<u16> {
        s.support.grants[..s.support.len as usize]
            .iter()
            .filter(|g| g.target == target)
            .flat_map(|g| g.programs[..g.len as usize].iter().copied())
            .collect()
    }
    pub(super) fn support_keyword(&self, s: &State, target: CardId, index: usize) -> i64 {
        if index >= 9 {
            return 0;
        }
        s.support.grants[..s.support.len as usize]
            .iter()
            .filter(|g| g.target == target)
            .map(|g| g.keywords[index] as i64)
            .sum()
    }
    pub(super) fn support_presence(&self, s: &State, target: CardId) -> u16 {
        s.support.grants[..s.support.len as usize]
            .iter()
            .filter(|g| g.target == target)
            .fold(0, |n, g| n | g.presence)
    }
    pub(super) fn support_targets(&self, s: &State) -> Vec<CardId> {
        let p = s.player(s.active);
        p.ground
            .as_slice()
            .iter()
            .chain(p.space.as_slice())
            .copied()
            .filter(|&id| {
                id != s.source
                    && !s.card(id).exhausted
                    && !self.attack_targets(s, id, false).is_empty()
            })
            .collect()
    }
    pub(super) fn begin_support_attack(
        &self,
        s: &mut State,
        attacker: CardId,
    ) -> Result<(), String> {
        if s.support.pending {
            return Err("nested Support attack is outside the admitted lifecycle".into());
        }
        if s.support.len as usize == MAX_SUPPORT_GRANTS {
            return Err("Support lasting-effect capacity exceeded (32)".into());
        }
        let mut grant = SupportGrant {
            target: attacker,
            donor: s.source,
            ..SupportGrant::default()
        };
        let mut programs = self.support_programs(s, s.source);
        programs.splice(0..0, self.card_programs(s, s.source).iter().copied());
        // An ordinary attached upgrade may grant an On Attack registration. Copy
        // its properties now; later removal of that upgrade does not remove the copy.
        for id in self.upgrades(s, s.source) {
            for &p in &self.def(s, id).programs.card {
                if self.program(p).gained_host == Some(true) {
                    programs.push(p);
                }
            }
        }
        if programs.len() > MAX_SUPPORT_PROGRAMS {
            return Err("Support copied-ability capacity exceeded (80)".into());
        }
        grant.len = programs.len() as u8;
        grant.programs[..programs.len()].copy_from_slice(&programs);
        for (i, name) in crate::ability_ir::CORE_KEYWORDS[..9].iter().enumerate() {
            if self.has_keyword(s, s.source, name) {
                grant.presence |= 1 << i;
            }
            grant.keywords[i] = self.keyword_amount(s, s.source, name);
        }
        s.support.context = ResolutionContext::capture(s);
        s.support.pending = true;
        s.support.grants[s.support.len as usize] = grant;
        s.support.len += 1;
        s.source = attacker;
        s.target = 0;
        s.pending_programs = 0;
        s.pending_triggers = 0;
        s.after_attack_programs = 0;
        s.after_attack_triggers = 0;
        s.trigger_context = 0;
        s.ambush = false;
        s.decision = Decision::AttackTarget;
        self.check_stat_capacity(s)?;
        Ok(())
    }
    pub(super) fn resume_support(&self, s: &mut State) -> Result<(), String> {
        let context = s.support.context;
        s.support.pending = false;
        s.support.context = ResolutionContext::default();
        context.restore(s);
        self.process_triggers(s)
    }
    pub(super) fn remove_support_target(&self, s: &mut State, target: CardId) {
        let mut len = 0;
        for i in 0..s.support.len as usize {
            if s.support.grants[i].target != target {
                s.support.grants[len] = s.support.grants[i];
                len += 1;
            }
        }
        s.support.grants[len..].fill(SupportGrant::default());
        s.support.len = len as u8;
    }
    pub(super) fn clear_support_grants(&self, s: &mut State) {
        s.support.grants.fill(SupportGrant::default());
        s.support.len = 0;
    }
    fn definition_needs_support_state(&self, definition: &Definition) -> bool {
        fn effect_needs_support(effect: &Effect) -> bool {
            match effect {
                Effect::GainKeyword { keyword, .. } => keyword == "support",
                Effect::CopyKeywords { domain, .. } => domain.iter().any(|k| k == "support"),
                Effect::Sequence { effects } | Effect::Simultaneous { effects }
                | Effect::PhaseModifiers { effects } => effects.iter().any(effect_needs_support),
                Effect::Select { effect, .. } => effect_needs_support(effect),
                Effect::Conditional { on_true, on_false, .. } => effect_needs_support(on_true)
                    || on_false.as_deref().is_some_and(effect_needs_support),
                _ => false,
            }
        }
        fn program_needs_support(program: &Program) -> bool {
            // Continuations and named selector effects have already been compiled
            // into this same instruction stream. Conditions do not erase potential.
            program.instructions.iter().any(|instruction| match instruction {
                Instruction::Apply(effect) => effect_needs_support(effect),
                other => other.group_effect().is_some_and(effect_needs_support),
            })
        }
        definition.flags & SUPPORT != 0 || self.catalogue.as_ref().is_some_and(|catalogue| {
            definition.programs.card.iter().chain(&definition.programs.unit)
                .chain(&definition.programs.leader)
                .any(|&p| program_needs_support(&catalogue.programs[p as usize]))
        })
    }
    pub(super) fn needs_support_state(&self) -> bool {
        // A compiled pool retains all admitted definitions for code lookup, but
        // catalogue metadata identifies the selected pool and its dependencies.
        self.catalogue.as_ref().map_or_else(
            || self.definitions.iter().any(|d| self.definition_needs_support_state(d)),
            |catalogue| catalogue.metadata.keys().any(|code| self.by_code.get(code)
                .is_some_and(|&i| self.definition_needs_support_state(&self.definitions[i as usize]))),
        )
    }
    pub(super) fn state_needs_support_state(&self, state: &State) -> bool {
        // Imported identities may resolve through the retained global by_code map.
        // Protect those actual cards too, even if absent from selected metadata.
        (1..state.next_id).any(|id| state.card(id).present && (
            state.card(id).phase_keyword_mask() & SUPPORT != 0
                || self.definition_needs_support_state(self.def(state, id))))
    }
    pub(super) fn support_snapshot(&self, s: &State) -> Value {
        let catalogue = self.catalogue.as_ref().expect("Support requires catalogue");
        json!(s.support.grants[..s.support.len as usize].iter().map(|g|json!({
            "target":format!("c{}",g.target-1),"donor":format!("c{}",g.donor-1),
            "programs":g.programs[..g.len as usize].iter().map(|&p|serde_json::to_value(&catalogue.programs[p as usize]).expect("serializable program")).collect::<Vec<_>>(),
            "keywords":g.keywords,"presence":g.presence
        })).collect::<Vec<_>>())
    }
    pub(super) fn load_support_state(&self, s: &mut State, v: &Value) -> Result<(), String> {
        if v.get("supportAttackPending").is_some() {
            return Err("pending Support attacks cannot be imported as action windows".into());
        }
        let Some(version) = v.get("supportStateVersion") else {
            if v.get("supportGrants").is_some() || self.needs_support_state()
                || self.state_needs_support_state(s) {
                return Err(
                    "selected Support pool requires explicit Support lasting-effect state".into(),
                );
            }
            return Ok(());
        };
        if version.as_u64() != Some(1) {
            return Err("unsupported supportStateVersion".into());
        }
        let grants = v["supportGrants"]
            .as_array()
            .ok_or("Support state requires grant list")?;
        if grants.len() > MAX_SUPPORT_GRANTS {
            return Err("Support lasting-effect capacity exceeded (32)".into());
        }
        let catalogue = self
            .catalogue
            .as_ref()
            .ok_or("Support grants require catalogue")?;
        for raw in grants {
            if !raw.as_object().is_some_and(|o| {
                o.len() == 5
                    && ["target", "donor", "programs", "keywords", "presence"]
                        .iter()
                        .all(|k| o.contains_key(*k))
            }) {
                return Err("invalid Support grant fields".into());
            }
            let id = |name: &str| -> Result<CardId, String> {
                let id = cid(raw[name].as_str().ok_or("Support grant identity missing")?)?;
                if id >= s.next_id || !s.card(id).present {
                    return Err("Support grant identity is absent".into());
                }
                Ok(id)
            };
            let mut g = SupportGrant {
                target: id("target")?,
                donor: id("donor")?,
                ..SupportGrant::default()
            };
            if !self.in_arena(s, g.target)
                || g.target == g.donor
                || s.card(g.target).owner != s.card(g.donor).owner
                || self.def(s, g.donor).flags & SUPPORT == 0
            {
                return Err(
                    "Support grant requires distinct friendly unit identities and intrinsic donor"
                        .into(),
                );
            }
            let programs = raw["programs"]
                .as_array()
                .ok_or("Support copied programs missing")?;
            if programs.len() > MAX_SUPPORT_PROGRAMS {
                return Err("Support copied-ability capacity exceeded (80)".into());
            }
            for (i, p) in programs.iter().enumerate() {
                let index = catalogue
                    .programs
                    .iter()
                    .position(|known| serde_json::to_value(known).ok().as_ref() == Some(p))
                    .ok_or("Support copied program is not admitted by this catalogue")?;
                // Granted programs may originate from earlier Support grants or
                // reviewed attachments. Every imported program must remain executable.
                if !self.definitions.iter().any(|d| {
                    d.programs
                        .card
                        .iter()
                        .chain(&d.programs.unit)
                        .any(|&id| id as usize == index)
                }) {
                    return Err("Support copied program is outside the selected pool".into());
                }
                let program = &catalogue.programs[index];
                if !matches!(program.trigger.as_str(), "constant" | "onAttack")
                    || program.coordinate
                    || program.attached_context.is_some()
                {
                    return Err(
                        "Support copied program exceeds the admitted donor lifecycle".into(),
                    );
                }
                g.programs[i] = index as u16;
            }
            g.len = programs.len() as u8;
            g.keywords = serde_json::from_value(raw["keywords"].clone())
                .map_err(|_| "Support keyword snapshot requires nine bounded integers")?;
            g.presence = raw["presence"]
                .as_u64()
                .filter(|n| *n < 512)
                .ok_or("invalid Support keyword presence")? as u16;
            for (i, &n) in g.keywords.iter().enumerate() {
                if (i != 6 && i != 7 && n > 1) || (n > 0 && g.presence & (1 << i) == 0) {
                    return Err("invalid Support keyword snapshot".into());
                }
            }
            s.support.grants[s.support.len as usize] = g;
            s.support.len += 1;
        }
        Ok(())
    }
}
