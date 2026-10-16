// Pinned SupportAbility / InitiateAttackSystem / GainNonKeywordAbilitiesFromUnitEffect.
// Optional attack cancellation intentionally retains early UntilEndOfAttack effects:
// only an actual AttackFlow cleanup (or the target leaving play) removes them.
use super::*;
use crate::ability_ir::{Effect, Instruction, Program};

impl Engine {
    pub(super) fn native_attack_power(&self,s:&State,target:CardId)->i64 {
        s.support.grants[..s.support.len as usize].iter().filter(|g|g.target==target).map(|g|g.power as i64).sum()
    }
    pub(super) fn native_attack_eligible(&self,s:&State,source:CardId,target:CardId,p:&crate::native_attack::Properties)->bool {
        target!=0 && self.in_arena(s,target) && s.card(target).owner==s.card(source).owner
            && !s.card(target).exhausted && !self.attack_targets(s,target,false).is_empty()
            && p.predicate().ok().is_some_and(|predicate|predicate.is_none_or(|e|self.native_attack_predicate(s,source,target,&e)))
    }
    pub(super) fn begin_native_attack(&self,s:&mut State,p:&crate::native_attack::Properties,optional:bool)->Result<(),String> {
        if s.support.pending {return Err("nested modified attack exceeds the admitted lifecycle".into());}
        let modifiers=p.modifiers()?;
        if (modifiers.power!=0||modifiers.presence!=0) && s.support.len as usize==MAX_SUPPORT_GRANTS {return Err("modified attack lasting-effect capacity exceeded (32)".into());}
        // Targeting and parent costs have completed. The early event applies its
        // captured literal modifiers before the defender prompt, and cancellation
        // retains them exactly like the existing Support grant lifecycle.
        if modifiers.power!=0||modifiers.presence!=0 {
            s.support.grants[s.support.len as usize]=SupportGrant {target:s.ability_target,donor:s.source,native_attack:true,power:modifiers.power,keywords:modifiers.keywords,presence:modifiers.presence,..SupportGrant::default()};
            s.support.len+=1;
        }
        s.ability_pc+=1;s.ability_succeeded=true;s.ability_result=1;
        s.support.context=ResolutionContext::capture(s);s.support.pending=true;s.support.native_attack=true;s.support.optional_attack=optional;
        s.source=s.ability_target;s.target=0;s.ability_program=0;s.ability_target=0;s.ability_pc=0;
        s.pending_programs=0;s.pending_triggers=0;s.after_attack_programs=0;s.after_attack_triggers=0;s.trigger_context=0;s.ambush=false;
        s.decision=Decision::AttackTarget;self.check_stat_capacity(s)?;self.sweep_stat_defeats(s)?;
        if self.open_deaths(s,7)? {return Ok(());}
        self.native_attack_after_early_effects(s)
    }
    pub(super) fn native_attack_after_early_effects(&self,s:&mut State)->Result<(),String> {
        // Early lasting effects are a real framework event window. A grant can
        // change a keyword-dependent HP aura; complete its deaths before the
        // nested attack resolver checks its physical attacker again.
        if !self.in_arena(s,s.source)||s.card(s.source).exhausted||self.attack_targets(s,s.source,false).is_empty() {return self.resume_support(s);}
        s.decision=Decision::AttackTarget;Ok(())
    }
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
                    && self.has_attack_target(s, id, false)
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
        let native=s.support.native_attack;
        s.support.native_attack=false;s.support.optional_attack=false;
        s.support.pending = false;
        s.support.context = ResolutionContext::default();
        context.restore(s);
        if native {self.run_program(s)} else {self.process_triggers(s)}
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
                Effect::InitiateAttack{..}=>true,
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
        self.selected_definition_any(|definition| self.definition_needs_support_state(definition))
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
        json!(s.support.grants[..s.support.len as usize].iter().map(|g|{
            let mut value=json!({
                "target":format!("c{}",g.target-1),"donor":format!("c{}",g.donor-1),
                "programs":g.programs[..g.len as usize].iter().map(|&p|serde_json::to_value(&catalogue.programs[p as usize]).expect("serializable program")).collect::<Vec<_>>(),
                "keywords":g.keywords,"presence":g.presence
            });
            if g.native_attack {value["kind"]=json!("modifiedAttack");value["power"]=json!(g.power);}
            value
        }).collect::<Vec<_>>())
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
                (o.len() == 5 || o.len()==7&&o.get("kind")==Some(&json!("modifiedAttack"))&&o.contains_key("power"))
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
            let native=raw["kind"]=="modifiedAttack";
            if !native && (!self.in_arena(s, g.target)
                || g.target == g.donor
                || s.card(g.target).owner != s.card(g.donor).owner
                || self.def(s, g.donor).flags & SUPPORT == 0)
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
            if native {
                g.native_attack=true;g.power=raw["power"].as_i64().and_then(|n|i16::try_from(n).ok()).ok_or("invalid modified attack power")?;
                if !self.in_arena(s,g.target)||s.card(g.target).owner!=s.card(g.donor).owner||g.len!=0 {return Err("modified attack grant requires friendly in-play recipient and no copied programs".into());}
                let expected=crate::native_attack::Modifiers{power:g.power,keywords:g.keywords,presence:g.presence};
                let valid=(expected.power!=0||expected.presence!=0)&&self.card_programs(s,g.donor).iter().any(|p|self.program(*p).instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::InitiateAttack{properties,..}) if properties.modifiers().ok().as_ref()==Some(&expected))));
                if !valid {return Err("modified attack grant does not match its admitted physical donor".into());}
            }
            s.support.grants[s.support.len as usize] = g;
            s.support.len += 1;
        }
        Ok(())
    }
}
