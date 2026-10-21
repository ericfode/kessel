//! Selected hand-card COST: early effect target → cancellable card selection →
//! discard event commitment/settlement → remaining exhaust cost → effect.
//! Cost identity uses the existing captured physical-hand binding; it never
//! overwrites the effect recipient. Cost cursor and lock already survive frames.
use super::*;
impl Engine {
 /// Shared physical movement only. Callers own event purpose and settlement:
 /// scalar effects settle their effect window; costs advance their retained
 /// progress and then open the distinct cost window before the next payment.
 pub(super) fn discard_physical_hand_card(&self,s:&mut State,payer:Seat,id:CardId)->Result<bool,String>{
  if id==0||id>=s.next_id||!s.card(id).present||s.card(id).owner!=payer||!s.player(payer).hand.contains(id){return Ok(false)}
  if s.player(payer).discard.len as usize>=MAX_ZONE{return Err("physical hand discard destination capacity exceeded".into())}
  s.player_mut(payer).hand.remove(id);s.player_mut(payer).discard.push(id)?;Ok(true)
 }
 /// AbilityResolver.resolveTargets runs after all costs. Keep a valid selected
 /// target even if now exhausted; otherwise repeat the original selector with
 /// cancellation disabled, or finish without refund if none can resolve.
 pub(super) fn revalidate_selected_hand_cost_target(&self,s:&mut State)->Result<bool,String>{
  let p=s.ability_program.checked_sub(1).ok_or("selected cost program missing")?;
  let Some(Instruction::Select(r))=self.program(p).instructions.first()else{return Err("selected cost target resolver missing".into())};
  let legal=self.program_targets(s,s.source,p,0,r,false);
  if legal.is_empty(){self.end_program(s)?;return Ok(false)}
  // An ineffective early resolver publishes a valid null binding. The paid
  // no-legal-target confirmation instead retains the end continuation: its
  // target resolver was delayed, so it must run now if targets appeared.
  let delayed=s.ability_pc as usize==self.program(p).instructions.len();
  if !delayed&&(s.ability_target==0||legal.contains(&s.ability_target)){return Ok(true)}
  let effective=self.program_targets(s,s.source,p,0,r,true);
  s.ability_target=0;s.ability_selection=Selection::default();
  if effective.is_empty(){self.end_program(s)?;return Ok(false)}
  s.ability_pc=0;s.decision=Decision::AbilityTarget;Ok(false)
 }

 pub(super) fn selected_hand_cost_prompt(&self,s:&State)->bool {
  s.decision==Decision::AbilityDiscard && s.ability_program!=0 && !s.ability_committed &&
  matches!(self.program(s.ability_program-1).costs.get(s.resource_cost_index as usize),Some(Cost::SelectCardCost(_)))
 }
 pub(super) fn selected_hand_cost_actions(&self,s:&State)->Vec<Action>{
  if !self.selected_hand_cost_prompt(s){return vec![]}
  let mut actions=Vec::new();if !s.resource_cost_locked&&s.ability_return==ProgramContinuation::Finish.raw(){actions.push(Action::button("Cancel","cancel"));}
  actions.extend(s.player(s.active).hand.as_slice().iter().map(|id|self.card_action(s,*id)));actions
 }
 pub(super) fn apply_selected_hand_cost(&self,s:&State,a:&Action)->Result<State,String>{
  if !self.selected_hand_cost_actions(s).contains(a){return Err("illegal selected hand-card cost choice".into())}
  let mut t=*s;
  if a.arg.as_ref().and_then(Value::as_str)==Some("cancel"){self.cancel(&mut t);return Ok(t)}
  let id=a.id().ok_or("selected cost requires physical card")?;
  t.captured_hand_card=crate::captured_hand_cost_contract::PhysicalHandCard::capture(id)?;
  // The active resolver player is captured in ResolutionContext, independent
  // of the physical source's controller. Never borrow the effect target slot.
  let payer=t.active;
  let resolved=self.discard_physical_hand_card(&mut t,payer,id)?;
  if !resolved{return Err("selected hand-card cost did not fully resolve".into())}
  t.resource_cost_index=t.resource_cost_index.checked_add(1).ok_or("ability cost progress capacity exceeded")?;
  t.resource_cost_locked=true;
  let finishing=t.ability_pc as usize==self.program(t.ability_program-1).instructions.len();
  if self.settle_cost_window(&mut t,finishing)? {return Ok(t)}
  if finishing {if self.commit_program_mode(&mut t,true)? {self.end_program(&mut t)?;}} else {self.run_program(&mut t)?;}Ok(t)
 }
}
