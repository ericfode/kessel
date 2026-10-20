//! Shared per-event base-damage listener capture. The caller snapshots listeners before ANY
//! primary in a window, supplies each actual resolved onDamageDealt event once,
//! then uses the existing deferred trigger window after sibling handlers/deaths.
//! This module neither deals damage nor manufactures events from damage totals.
use super::*;

#[derive(Clone,Copy,Debug)]
pub(super) struct BaseDamageListener {
    base:CardId,
    trigger:DeathTrigger,
}
impl Engine {
    /// Capture controller/base identity, physical source, registration and LKI
    /// before the first handler can remove a source or change the board. Reuse
    /// the exact captured roster for all sibling events, including after deaths.
    pub(super) fn blade_base_damage_listeners(&self,s:&State)->Result<Vec<BaseDamageListener>,String> {
        let Some(catalogue)=self.catalogue.as_ref() else {return Ok(vec![]);};
        let mut listeners=vec![];
        for source in self.arena_units(s) {
            let definition=self.def(s,source);
            if !definition.programs.unit.is_empty() || !definition.programs.leader.is_empty() || definition.programs.attach_condition.is_some() {continue;}
            let Some(card)=catalogue.metadata.get(&definition.code) else {continue;};
            let programs=definition.programs.card.iter().map(|&p|&catalogue.programs[p as usize]).collect::<Vec<_>>();
            if !crate::blade_base_damage_contract::compiled_source(card,&programs) {continue;}
            if !s.registration_order.exact || s.registration_order.position(source).is_none() {
                return Err("own-base damage listener capture requires exact source registrationOrder".into());
            }
            let player=s.card(source).owner;
            listeners.push(BaseDamageListener {base:s.player(player).base,trigger:DeathTrigger {
                source,program:definition.programs.card[0],player,
                last_known_power:self.power(s,source,self.is_attacking(s,source)),
                after_pc:if s.ability_program!=0 {self.program(s.ability_program-1).event_end}else{0},
                capture:crate::defeat_cause::TriggerCapture::new(s.death_depth,crate::defeat_cause::DefeatCause::Unknown),..DeathTrigger::default()
            }});
        }
        listeners.sort_by_key(|l|s.registration_order.position(l.trigger.source).unwrap());
        Ok(listeners)
    }
    /// `resolved` is the framework's resolved-event fact, not `damage > 0` or
    /// numerical change. No call is made for a zero-power attack that emits no
    /// event. A real resolved zero-valued event still matches the printed hook.
    /// Cancelled/replaced unresolved events do not register triggers.
    pub(super) fn capture_blade_base_damage_event(&self,s:&mut State,listeners:&[BaseDamageListener],card:CardId,resolved:bool)->Result<(),String> {
        if !resolved {return Ok(());}
        let matching=listeners.iter().filter(|l|l.base==card).collect::<Vec<_>>();
        if s.death_pending.len as usize+matching.len()>MAX_DEATH_TRIGGERS {
            return Err("own-base damage trigger capacity exceeded (64)".into());
        }
        // Capacity is checked for the complete event before publishing any
        // capture. Do not recheck source liveness/controller after sibling deaths:
        // the captured trigger persists; GiveAdvantage checks its physical target
        // in play at resolution and cannot allocate onto a departed source.
        for listener in matching {
            let n=s.death_pending.len as usize;let mut trigger=listener.trigger;trigger.event_card=card;
            s.death_pending.entries[n]=trigger;s.death_pending.len+=1;
        }
        Ok(())
    }
}
