//! Stable phase declaration evidence. Attack-local frames remain nonportable.
use super::*;
use crate::attack_base_history_contract::Query;
impl Engine {
 fn definition_has_attack_query(&self,d:&Definition,query:Query)->bool {
  d.programs.card.iter().chain(&d.programs.unit).chain(&d.programs.leader).any(|&p|crate::attack_base_history_contract::requires(self.program(p),query))
 }
 pub(super) fn needs_attack_base_history(&self)->bool {
  self.selected_definition_any(|d|self.definition_has_attack_query(d,Query::OpponentDeclaredOwnBase))
 }
 fn physical_attack_base_history(&self,s:&State)->bool {
  (1..s.next_id).any(|id|s.card(id).present&&self.definition_has_attack_query(self.def(s,id),Query::OpponentDeclaredOwnBase))
 }
 fn needs_attack_base_order(&self,s:&State)->bool {
  self.selected_definition_any(|d|self.definition_has_attack_query(d,Query::FriendlyAttackerHasBaseRecord))||
  (1..s.next_id).any(|id|s.card(id).present&&self.definition_has_attack_query(self.def(s,id),Query::FriendlyAttackerHasBaseRecord))
 }
 pub(super) fn load_attack_base_history(&self,s:&mut State,v:&Value)->Result<(),String> {
  if v.get("attackBaseHistoryPending").is_some() {return Err("pending attack base history is not importable".into());}
  if !s.is_terminal()&&(1..s.next_id).any(|id|self.in_arena(s,id)&&self.def(s,id).info.unique&&self.card_programs(s,id).iter().any(|&p|crate::attack_base_history_contract::has_query(self.program(p)))&&self.duplicates(s,id)>1) {return Err("unsettled attack-history unique source at action window".into());}
  if self.needs_attack_base_order(s)&&!s.registration_order.exact {return Err("global attack-end registration requires exact registrationOrder".into());}
  let required=self.needs_attack_base_history()||self.physical_attack_base_history(s);
  let Some(raw)=v.get("attackBaseHistory") else {
   if required {return Err("selected declaration history source requires attackBaseHistory".into());}
   return Ok(());
  };
  let o=raw.as_object().ok_or("attackBaseHistory must be an object")?;
  if o.len()!=2||o.keys().any(|k|!["version","baseAttackedByOpponent"].contains(&k.as_str()))||raw["version"]!=1 {return Err("attackBaseHistory requires exact version1 fields".into());}
  let a=raw["baseAttackedByOpponent"].as_array().ok_or("attackBaseHistory requires two booleans")?;
  if a.len()!=2||a.iter().any(|v|!v.is_boolean()) {return Err("attackBaseHistory requires two booleans".into());}
  if !required&&a.iter().any(|v|v.as_bool()==Some(true)) {return Err("nonempty declaration history requires a selected or retained source".into());}
  let mut history=crate::attack_base_history_state::History::default();if !required {history.set_explicit();}
  for (i,value) in a.iter().enumerate() {if value.as_bool()==Some(true) {history.record(if i==0 {Seat::P1}else{Seat::P2});}}
  s.attack_base_history=history;Ok(())
 }
 pub(super) fn snapshot_attack_base_history(&self,s:&State,v:&mut Value) {
  if self.needs_attack_base_history()||self.physical_attack_base_history(s)||s.attack_base_history.explicit()||s.attack_base_history.bits()!=0 {
   v["attackBaseHistory"]=json!({"version":1,"baseAttackedByOpponent":[s.attack_base_history.opponent_attacked(Seat::P1),s.attack_base_history.opponent_attacked(Seat::P2)]});
  }
  if s.attack_flags.enabled()||s.death_frames[..s.death_depth as usize].iter().any(|f|f.context.attack_flags.enabled())||s.support.context.attack_flags.enabled() {
   v["attackBaseHistoryPending"]=true.into();
  }
 }
}
