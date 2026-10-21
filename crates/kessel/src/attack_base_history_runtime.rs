//! Queries consume captured facts; scalar heal/token mutation stays in its owner.
use super::*;
use crate::attack_base_history_contract::Query;
impl Engine {
 pub(super) fn attack_base_query(&self,s:&State,source:CardId,query:Query)->Eval {
  match query {
   Query::OpponentDeclaredOwnBase=>Eval::Bool(source!=0&&s.attack_base_history.opponent_attacked(s.card(source).owner)),
   Query::SourceInternalName=>if source==0 {Eval::Null}else{Eval::Text(self.def(s,source).info.internal_name.clone())},
   Query::CurrentAttackBaseDamage=>{
    let public=source!=0&&!s.player(s.card(source).owner).hand.contains(source)&&!s.player(s.card(source).owner).deck.contains(source)&&!s.player(s.card(source).owner).resources.contains(source);
    Eval::Number(if s.attack_source==source&&s.attack_flags.enabled()&&!s.attack_flags.reentered()&&public {i64::from(s.attack_base_damage)}else{0})
   },
   Query::FriendlyAttackerHasBaseRecord=>Eval::Bool(source!=0&&s.attack_source!=0&&!s.player(s.card(s.attack_source).owner).hand.contains(s.attack_source)&&!s.player(s.card(s.attack_source).owner).deck.contains(s.attack_source)&&!s.player(s.card(s.attack_source).owner).resources.contains(s.attack_source)&&s.attack_flags.enabled()&&s.attack_flags.record_exists()&&!s.attack_flags.reentered()&&s.attack_flags.attacking_player()==s.card(source).owner),
  }
 }
 pub(in crate::simulator) fn start_attack_base_context(&self,s:&mut State,attacker:CardId,target:CardId) {
  s.attack_base_damage=0;s.attack_flags.reset_history();
  let history=self.needs_attack_base_history()||(1..s.next_id).any(|id|s.card(id).present&&self.card_programs(s,id).iter().any(|&p|crate::attack_base_history_contract::requires(self.program(p),Query::OpponentDeclaredOwnBase)));
  if history {
   for seat in [Seat::P1,Seat::P2] {if target==s.player(seat).base&&s.card(attacker).owner==seat.other() {s.attack_base_history.record(seat);}}
  }
  let local=|p:u16|[Query::CurrentAttackBaseDamage,Query::FriendlyAttackerHasBaseRecord].into_iter().any(|q|crate::attack_base_history_contract::requires(self.program(p),q));
  let enabled=self.selected_definition_any(|d|d.programs.card.iter().chain(&d.programs.unit).chain(&d.programs.leader).copied().any(local))||
   (1..s.next_id).any(|id|s.card(id).present&&(self.card_programs(s,id).iter().copied().any(local)||self.support_programs(s,id).into_iter().any(local)));
  if enabled {s.attack_flags.enable();s.attack_flags.set_attacking_player(s.card(attacker).owner);}
 }
 /// A real entry, unlike public discard, gives this physical object a new
 /// incarnation. Every suspended attack keeps its own irreversible mismatch.
 pub(in crate::simulator) fn attack_base_reentry(&self,s:&mut State,id:CardId) {
  if s.attack_source==id&&s.attack_flags.enabled() {s.attack_flags.mark_reentered();}
  for f in &mut s.death_frames[..s.death_depth as usize] {if f.context.attack_source==id&&f.context.attack_flags.enabled() {f.context.attack_flags.mark_reentered();}}
  if s.support.context.attack_source==id&&s.support.context.attack_flags.enabled() {s.support.context.attack_flags.mark_reentered();}
 }
 /// Caller supplies the resolved Combat-base/Overwhelm record's applied amount,
 /// before changing base damage. A missing/zero-power event makes no call.
 pub(in crate::simulator) fn record_attack_base_damage(&self,s:&mut State,amount:u16)->Result<(),String> {
  if !s.attack_flags.enabled()||s.attack_flags.reentered() {return Ok(());}
  let sum=s.attack_base_damage.checked_add(u32::from(amount)).ok_or("attack base damage aggregate capacity exceeded")?;
  s.attack_base_damage=sum;s.attack_flags.set_record_exists();Ok(())
 }

 /// The closed conditional registration can match at either emitter epoch.
 /// Between these two epochs this combat owner performs damage/Shield removal
 /// and queues observers; it does not execute an ability or enter a unit. Thus
 /// the registration multiset is stable, including pending-defeat units. Count
 /// occurrences rather than collapsing duplicate copied registrations by ID.
 pub(in crate::simulator) fn capture_attack_base_end(&self,s:&mut State,post:bool)->Result<(),String> {
  if !s.attack_flags.enabled()||s.attack_source==0 {return Ok(());}
  if post&&s.attack_flags.post_captured()||!post&&s.attack_flags.end_captured() {return Ok(());}
  let global=|p:u16|matches!(self.program(p).condition.as_ref(),Some(Expr::AttackBaseQuery{query:Query::FriendlyAttackerHasBaseRecord}));
  let mut potential=vec![];
  if !post&&self.in_arena(s,s.attack_source) {
   for p in self.card_programs(s,s.attack_source).iter().copied().chain(self.support_programs(s,s.attack_source)) {
    if self.program(p).trigger=="whenAttackEnds"&&!global(p) {
     if self.program(p).condition.is_some() {return Err("conditional attack-end capture requires the closed base-record registration".into());}
     potential.push((s.attack_source,p));
    }
   }
  }
  for source in self.arena_units(s) {
   for p in self.card_programs(s,source).iter().copied().chain(self.support_programs(s,source)) {
    if self.program(p).trigger=="whenAttackEnds"&&global(p) {potential.push((source,p));}
   }
  }
  if !post {
   // Reserve the full possible post-handler union as well as the existing
   // base-damage and Advantage listeners sharing this physical event window.
   // No listener is executed between reservation and post capture.
   let advantages=[s.attack_source,s.attack_defender].into_iter().filter(|&id|self.in_arena(s,id)).flat_map(|id|self.upgrades(s,id)).filter(|&id|Some(s.card(id).def)==self.advantage_def).count();
   // Preserve the existing independent Advantage bound diagnostic before
   // reserving the larger cross-family capture union.
   if (s.death_pending.len as usize).saturating_add(advantages)>MAX_DEATH_TRIGGERS {return Err("Advantage attack-end trigger capacity exceeded (64)".into());}
   let base_listeners=self.blade_base_damage_listeners(s)?.len();
   if (s.death_pending.len as usize).saturating_add(potential.len()).saturating_add(advantages).saturating_add(base_listeners)>MAX_DEATH_TRIGGERS {return Err("attack-end union capacity exceeded (64) before damage".into());}
  }
  let mut append=vec![];let mut occurrences=std::collections::BTreeMap::new();
  for (source,program) in potential {
   if global(program)&&!self.attack_base_query(s,source,Query::FriendlyAttackerHasBaseRecord).truth() {continue;}
   let count=occurrences.entry((source,program)).or_insert(0usize);*count+=1;
   let already=if post {s.death_pending.entries[..s.death_pending.len as usize].iter().filter(|e|e.source==source&&e.program==program).count()}else{0};
   if *count>already {append.push(DeathTrigger{source,program,player:s.card(source).owner,..DeathTrigger::default()});}
  }
  if s.death_pending.len as usize+append.len()>MAX_DEATH_TRIGGERS {return Err("attack-end union capacity exceeded (64)".into());}
  if append.iter().any(|e|s.registration_order.position(e.source).is_none()) {return Err("attack-end registration missing physical chronology".into());}
  if !s.registration_order.exact {return Err("attack-end union requires exact registrationOrder".into());}
  append.sort_by_key(|e|s.registration_order.position(e.source).unwrap());
  for entry in append {let n=s.death_pending.len as usize;s.death_pending.entries[n]=entry;s.death_pending.len+=1;}
  if post {s.attack_flags.set_post_captured();}else{s.attack_flags.set_end_captured(true);}
  Ok(())
 }

}
