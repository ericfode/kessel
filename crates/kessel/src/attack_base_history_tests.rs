//! Synthetic native cases; actual source terminal games are independent evidence.
use super::*;
fn catalogue()->Value {serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap()}
fn engine(backend:RulesBackend)->Engine {let mut e=Engine::from_catalogue_value(&catalogue()).unwrap();for code in ["ASH_119","ASH_031","ASH_144"] {assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));}e.set_rules_backend(backend).unwrap();e}
fn add(e:&Engine,s:&mut State,owner:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;*s.card_mut(id)=Card{present:true,owner,def:*e.by_code.get(code).unwrap_or_else(||panic!("{code}")),..Card::default()};id}
fn unit(e:&Engine,s:&mut State,owner:Seat,code:&str)->CardId {let id=add(e,s,owner,code);e.add_arena(s,id).unwrap();id}
fn board(e:&Engine,seat:Seat)->State {let mut s=State::default();s.active=seat;s.action_player=Some(seat);s.hidden_history_exact=true;s.force_state_exact=true;s.credit_state_exact=true;s.round_limit_wire_present=true;for owner in [Seat::P1,Seat::P2] {s.player_mut(owner).base=add(e,&mut s,owner,"SOR_023");s.player_mut(owner).leader=add(e,&mut s,owner,"SOR_014");s.player_mut(owner).force_id=s.next_id;s.next_id+=1;s.player_mut(owner).force_held=true;for _ in 0..12 {let id=add(e,&mut s,owner,"LOF_254");s.player_mut(owner).resources.push(id).unwrap();}for _ in 0..4 {let id=add(e,&mut s,owner,"LOF_254");s.player_mut(owner).deck.push(id).unwrap();}}let t=e.load_state(&e.snapshot(&s)).unwrap();assert_eq!(t.zobrist(),s.zobrist(),"builder canonicalizes only evidence-presence flags");t}
fn click(e:&Engine,s:&mut State,label:&str) {let a=e.legal_actions(s).into_iter().find(|a|a.label==label).unwrap_or_else(||panic!("missing {label}: {:?}",e.legal_actions(s)));*s=e.apply(s,&a).unwrap();}
fn attack(e:&Engine,s:&mut State,source:CardId,target:CardId) {let a=e.card_action(s,source);assert!(e.legal_actions(s).contains(&a));*s=e.apply(s,&a).unwrap();assert_eq!(s.decision,Decision::AttackTarget);let a=e.card_action(s,target);assert!(e.legal_actions(s).contains(&a));*s=e.apply(s,&a).unwrap();}
fn fixed(e:&Engine,s:&State) {let v=e.snapshot(s);let t=e.load_state(&v).unwrap();assert_eq!(e.snapshot(&t),v);for id in 1..s.next_id {assert_eq!(t.card(id),s.card(id),"physical card{id}");}assert_eq!(t.players,s.players);assert_eq!(t.zobrist(),s.zobrist());assert_eq!(t,*s);}
#[test]
fn attack_base_greef_pre_entry_history_and_mandatory_wire() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat.other());fixed(&e,&s);let mut missing=e.snapshot(&s);missing.as_object_mut().unwrap().remove("attackBaseHistory");assert!(e.load_state(&missing).is_err());
  let attacker=unit(&e,&mut s,seat.other(),"LOF_254");let base=s.player(seat).base;attack(&e,&mut s,attacker,base);assert!(s.attack_base_history.opponent_attacked(seat));
  s.active=seat;s.action_player=Some(seat);let greef=unit(&e,&mut s,seat,"ASH_119");fixed(&e,&s);let ready=e.ready_resources(&s,seat);
  let a=e.card_action(&s,greef);s=e.apply(&s,&a).unwrap();click(&e,&mut s,"Create a Mandalorian token");assert!(s.card(greef).exhausted);assert_eq!(e.ready_resources(&s,seat),ready-1);
  let tokens=e.arena_units(&s).into_iter().filter(|&id|e.def(&s,id).code=="TOKEN_MANDALORIAN").collect::<Vec<_>>();assert_eq!(tokens.len(),1);assert_ne!(s.card(tokens[0]).shield,0);fixed(&e,&s);
 }}
}
#[test]
fn attack_base_hera_actual_capped_damage_after_winner() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let hera=unit(&e,&mut s,seat,"ASH_031");let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=5;s.card_mut(enemy).damage=29;fixed(&e,&s);
  attack(&e,&mut s,hera,enemy);assert_eq!(s.card(own).damage,4);assert_eq!(s.card(enemy).damage,30);assert!(s.is_terminal());
 }}
}
#[test]
fn attack_base_vane_friendly_base_record_and_opponent_negative() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat,"LOF_254");let base=s.player(seat.other()).base;attack(&e,&mut s,attacker,base);assert_eq!(e.upgrades(&s,vane).iter().filter(|&&id|e.def(&s,id).code=="TOKEN_ADVANTAGE").count(),1);fixed(&e,&s);
  let mut s=board(&e,seat.other());let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat.other(),"LOF_254");let base=s.player(seat).base;attack(&e,&mut s,attacker,base);assert!(e.upgrades(&s,vane).is_empty());fixed(&e,&s);
 }}
}
#[test]
fn attack_base_wire_producer_free_absence_optional_empty_and_strict_shapes() {
 let mut raw=catalogue();raw["cards"].as_array_mut().unwrap().retain(|v|!["ASH_119","ASH_031","ASH_144"].contains(&v["setCode"].as_str().unwrap_or("")));
 let e=Engine::from_catalogue_value(&raw).unwrap();let mut s=board(&e,Seat::P1);assert!(e.snapshot(&s).get("attackBaseHistory").is_none());
 let unit=unit(&e,&mut s,Seat::P1,"LOF_254");let base=s.player(Seat::P2).base;attack(&e,&mut s,unit,base);assert!(e.snapshot(&s).get("attackBaseHistory").is_none());fixed(&e,&s);
 let mut explicit=e.snapshot(&s);explicit["attackBaseHistory"]=json!({"version":1,"baseAttackedByOpponent":[false,false]});let t=e.load_state(&explicit).unwrap();assert_eq!(t.zobrist(),s.zobrist());assert_ne!(t,s);fixed(&e,&t);
 for bad in [json!(null),json!({"version":2,"baseAttackedByOpponent":[false,false]}),json!({"version":1,"baseAttackedByOpponent":[false]}),json!({"version":1,"baseAttackedByOpponent":[false,0]}),json!({"version":1,"baseAttackedByOpponent":[false,true]}),json!({"version":1,"baseAttackedByOpponent":[false,false],"extra":false})] {let mut v=e.snapshot(&s);v["attackBaseHistory"]=bad;assert!(e.load_state(&v).is_err());}
 for value in [json!(false),json!(null),json!({})] {let mut v=e.snapshot(&s);v["attackBaseHistoryPending"]=value;assert!(e.load_state(&v).is_err());}
}
#[test]
fn attack_base_greef_credit_ability_cost_use_skip_cancel_both_seats() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for choice in ["Use 1 Credit","Cancel","Pay costs without Credit tokens"] {
  let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,"ASH_119");s.attack_base_history.record(seat);e.create_credit_tokens(&mut s,seat,1).unwrap();let resources=s.player(seat).resources.as_slice().to_vec();for &id in &resources {s.card_mut(id).exhausted=true;}if choice=="Pay costs without Credit tokens" {s.card_mut(resources[0]).exhausted=false;}fixed(&e,&s);
  let a=e.card_action(&s,source);s=e.apply(&s,&a).unwrap();click(&e,&mut s,"Create a Mandalorian token");assert_eq!(s.decision,Decision::CreditMenu);assert_eq!(s.resource_payment.purpose,crate::resource_payment::PaymentPurpose::AbilityCost);assert_eq!(s.resource_payment.payer,seat);assert_eq!(s.resource_payment.amount,1);assert!(!s.card(source).exhausted);assert_eq!(s.player(seat).credits.len(),1);
  let saved=s;let hash=s.zobrist();let action=e.legal_actions(&s).into_iter().find(|a|a.label==choice).unwrap();let copy=e.apply(&s,&action).unwrap();assert_eq!(s,saved);assert_eq!(s.zobrist(),hash);s=e.apply(&s,&action).unwrap();assert_eq!(s,copy);
  if choice=="Cancel" {assert!(!s.card(source).exhausted);assert_eq!(s.player(seat).credits.len(),1);assert!(!e.arena_units(&s).iter().any(|&id|e.def(&s,id).code=="TOKEN_MANDALORIAN"));}
  else {assert!(s.card(source).exhausted);assert_eq!(e.ready_resources(&s,seat),0);assert_eq!(s.player(seat).credits.len(),u16::from(choice!="Use 1 Credit"));let tokens=e.arena_units(&s).into_iter().filter(|&id|e.def(&s,id).code=="TOKEN_MANDALORIAN").collect::<Vec<_>>();assert_eq!(tokens.len(),1);assert_ne!(s.card(tokens[0]).shield,0);}
  fixed(&e,&s);
 }}}
}
#[test]
fn attack_base_hera_public_dead_incarnation_heals_and_vane_pending_defeat_captures() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  // Synthetic native analogue of the actual source Overwhelm/death boundaries.
  let mut s=board(&e,seat);let hera=unit(&e,&mut s,seat,"ASH_031");s.card_mut(hera).damage=1;let upgrade=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,upgrade,hera);let defender=unit(&e,&mut s,seat.other(),"HMW_093");s.card_mut(defender).damage=6;let base=s.player(seat).base;s.card_mut(base).damage=5;assert_eq!(e.keyword_amount(&s,hera,"overwhelm"),1);fixed(&e,&s);attack(&e,&mut s,hera,defender);assert!(s.player(seat).discard.contains(hera));assert_eq!(s.card(base).damage,2);fixed(&e,&s);
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");s.card_mut(vane).damage=2;let upgrade=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,upgrade,vane);let defender=unit(&e,&mut s,seat.other(),"HMW_178");fixed(&e,&s);
  let mut probe=s;probe.attack_source=vane;probe.attack_defender=defender;e.start_attack_base_context(&mut probe,vane,defender);e.capture_attack_end(&mut probe).unwrap();assert_eq!(probe.death_pending.len,0);e.record_attack_base_damage(&mut probe,1).unwrap();probe.card_mut(vane).damage=4;e.capture_attack_base_end(&mut probe,true).unwrap();assert_eq!(probe.death_pending.len,1);assert_eq!(probe.death_pending.entries[0].source,vane);assert_eq!(probe.death_pending.entries[0].player,seat);
  let before=s.next_id;attack(&e,&mut s,vane,defender);assert!(s.player(seat).discard.contains(vane));assert_eq!(s.next_id,before,"captured dead recipient cannot receive an Advantage");assert_eq!(s.card(s.player(seat.other()).base).damage,1);fixed(&e,&s);
 }}
}
#[test]
fn attack_base_scoped_incarnation_copy_and_separate_record_zero() {
 let e=engine(RulesBackend::Auto);let mut s=board(&e,Seat::P1);let source=unit(&e,&mut s,Seat::P1,"ASH_031");let base=s.player(Seat::P2).base;s.attack_source=source;s.attack_defender=base;e.start_attack_base_context(&mut s,source,base);e.record_attack_base_damage(&mut s,0).unwrap();assert!(s.attack_flags.record_exists());assert_eq!(s.attack_base_damage,0);
 s.death_frames[0].context=ResolutionContext::capture(&s);s.death_depth=1;s.support.context=ResolutionContext::capture(&s);s.support.pending=true;let saved=s;assert_eq!(saved.zobrist(),s.zobrist());e.attack_base_reentry(&mut s,source);assert!(s.attack_flags.reentered());assert!(s.death_frames[0].context.attack_flags.reentered());assert!(s.support.context.attack_flags.reentered());assert_ne!(s.zobrist(),saved.zobrist());assert!(!saved.attack_flags.reentered());
 assert!(e.load_state(&e.snapshot(&s)).is_err());let before=s;s.attack_base_damage=u32::MAX;let saturated=s;assert!(e.record_attack_base_damage(&mut s,1).is_ok(),"reentered old attack ignores new records");assert_eq!(s,saturated);s=before;s.attack_flags=Default::default();s.attack_flags.enable();s.attack_base_damage=u32::MAX;let before=s;assert!(e.record_attack_base_damage(&mut s,1).is_err());assert_eq!(s,before);
}
#[test]
fn attack_base_zero_power_declaration_and_shield_no_record() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat,"LOF_254");s.card_mut(attacker).phase_power=-1;let enemy=s.player(seat.other()).base;attack(&e,&mut s,attacker,enemy);assert_eq!(s.card(enemy).damage,0);assert!(e.upgrades(&s,vane).is_empty());assert!(s.attack_base_history.opponent_attacked(seat.other()),"declaration exists despite no damage event");fixed(&e,&s);
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");s.card_mut(vane).damage=1;let upgrade=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,upgrade,vane);let target=unit(&e,&mut s,seat.other(),"HMW_178");let shield=add(&e,&mut s,seat.other(),"TOKEN_SHIELD");e.attach(&mut s,shield,target);s.card_mut(target).shield=shield;fixed(&e,&s);attack(&e,&mut s,vane,target);assert_eq!(s.card(s.player(seat.other()).base).damage,0);assert!(!e.upgrades(&s,vane).iter().any(|&id|e.def(&s,id).code=="TOKEN_ADVANTAGE"));fixed(&e,&s);
 }}
}
#[test]
fn attack_base_capture_capacity_is_atomic_and_post_epoch_deduplicated() {
 let e=engine(RulesBackend::Auto);
 for count in [63,64] {
  let mut s=board(&e,Seat::P1);let attacker=unit(&e,&mut s,Seat::P1,"LOF_254");let vane=unit(&e,&mut s,Seat::P1,"ASH_144");let hera=unit(&e,&mut s,Seat::P1,"ASH_031");let base=s.player(Seat::P2).base;s.attack_source=attacker;s.attack_defender=base;e.start_attack_base_context(&mut s,attacker,base);
  // Adversarial bounded queued entries: this tests representation atomicity,
  // not a claim that 63 copies of this registration arise in a source game.
  let program=e.card_programs(&s,hera)[0];for i in 0..count {s.death_pending.entries[i]=DeathTrigger{source:hera,program,player:Seat::P1,..Default::default()};}s.death_pending.len=count as u8;
  let before=s;let result=e.capture_attack_end(&mut s);
  if count==64 {assert!(result.is_err());assert_eq!(s,before);e.record_attack_base_damage(&mut s,1).unwrap();let before=s;assert!(e.capture_attack_base_end(&mut s,true).is_err());assert_eq!(s,before);}
  else {result.unwrap();assert_eq!(s.death_pending.len,63);e.record_attack_base_damage(&mut s,1).unwrap();e.capture_attack_base_end(&mut s,true).unwrap();assert_eq!(s.death_pending.len,64);assert_eq!(s.death_pending.entries[63].source,vane);let captured=s;e.capture_attack_base_end(&mut s,true).unwrap();assert_eq!(s,captured);}
 }
 // First-strike can make the condition true in the pre epoch already.
 let mut s=board(&e,Seat::P1);let attacker=unit(&e,&mut s,Seat::P1,"LOF_254");let vane=unit(&e,&mut s,Seat::P1,"ASH_144");let base=s.player(Seat::P2).base;s.attack_source=attacker;s.attack_defender=base;e.start_attack_base_context(&mut s,attacker,base);e.record_attack_base_damage(&mut s,1).unwrap();e.capture_attack_end(&mut s).unwrap();assert_eq!(s.death_pending.len,1);e.capture_attack_base_end(&mut s,true).unwrap();assert_eq!(s.death_pending.len,1);assert_eq!(s.death_pending.entries[0].source,vane);
}
#[test]
fn attack_base_copy_boundary_resolves_complete_set_before_publication() {
 let e=engine(RulesBackend::Auto);let mut s=board(&e,Seat::P1);let donor=unit(&e,&mut s,Seat::P1,"ASH_037");let recipient=unit(&e,&mut s,Seat::P1,"LOF_254");let row=json!({"kind":"sourcePrograms","target":format!("c{}",recipient-1),"donor":format!("c{}",donor-1),"sourcePrograms":{"setCode":"ASH_037","side":"card"},"keywords":[0,0,0,0,0,0,0,0,1],"presence":256});
 let mut positive=s;e.load_support_state(&mut positive,&json!({"supportStateVersion":1,"supportGrants":[row.clone()]})).unwrap();assert_eq!(positive.support.len,1);fixed(&e,&positive);
 for code in ["ASH_119","ASH_031","ASH_144"] {
  let id=e.by_code[code];let p=e.definitions[id as usize].programs.card[0];let mut bad=row.clone();bad.as_object_mut().unwrap().remove("kind");bad.as_object_mut().unwrap().remove("sourcePrograms");bad["programs"]=json!([serde_json::to_value(e.program(p)).unwrap()]);
  let mut target=s;let before=target;assert!(e.load_support_state(&mut target,&json!({"supportStateVersion":1,"supportGrants":[row.clone(),bad.clone()]})).is_err());assert_eq!(target,before,"later bad row must not publish earlier valid row");
  let mut snapshot=e.snapshot(&s);snapshot["supportGrants"]=json!([bad]);assert!(e.load_state(&snapshot).is_err());
  let mut reference=row.clone();reference["sourcePrograms"]["setCode"]=code.into();let mut target=s;assert!(e.load_support_state(&mut target,&json!({"supportStateVersion":1,"supportGrants":[reference]})).is_err());assert_eq!(target,s);
  // Existing grants and printed/attachment programmes are one resolved set.
  let mut target=s;target.source=donor;target.support.len=1;target.support.grants[0]=SupportGrant{target:donor,donor:recipient,len:1,..Default::default()};target.support.grants[0].programs[0]=p;let before=target;assert!(e.begin_support_attack(&mut target,recipient).is_err());assert_eq!(target,before);
 }
 for marker in ["supportAttackPending","attackBaseHistoryPending"] {let mut v=e.snapshot(&s);v[marker]=false.into();assert!(e.load_state(&v).is_err());}
}
#[test]
fn attack_base_standalone_queries_and_alias_profiles_cannot_authorize() {
 use crate::attack_base_history_contract::{Kind,Query};
 for query in [Query::OpponentDeclaredOwnBase,Query::CurrentAttackBaseDamage,Query::FriendlyAttackerHasBaseRecord,Query::SourceInternalName] {
  let value=serde_json::to_value(Expr::AttackBaseQuery{query}).unwrap();let raw=json!({"trigger":"action","title":"unscoped","costs":[],"condition":value,"effect":{"kind":"noAction"}});let ability:crate::ability_ir::Ability=serde_json::from_value(raw).unwrap();assert!(crate::ability_ir::Program::compile(&ability).is_err());
 }
 let base=catalogue();let canonical=Engine::from_catalogue_value(&base).unwrap().compiled_pool().unwrap();
 for kind in [Kind::Declaration,Kind::Heal,Kind::Advantage] {
  let profile=crate::attack_base_history_contract::profile(kind);let code=profile["setCode"].as_str().unwrap();let mut alias=profile.clone();alias["setCode"]="UNREVIEWED_ALIAS".into();let mut input=base.clone();input["cards"].as_array_mut().unwrap().push(alias);let e=Engine::from_catalogue_value(&input).unwrap();assert!(!e.by_code.contains_key("UNREVIEWED_ALIAS"));assert_eq!(e.compiled_pool().unwrap(),canonical);
  let mut pool=canonical.clone();let mut metadata=pool["cards"].as_array().unwrap().iter().find(|v|v["setCode"]==code).unwrap().clone();metadata["setCode"]="UNREVIEWED_ALIAS".into();pool["cards"].as_array_mut().unwrap().push(metadata);assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),&serde_json::to_vec(&input).unwrap()).is_err());
  for key in ["card","abilities"] {let mut bad=profile.clone();bad[key]["injected"]=true.into();assert!(crate::attack_base_history_contract::prepare(&bad).is_err());}
 }
}
#[test]
fn attack_base_new_creator_retirement_fullhash_diagnostic() {
 let e=engine(RulesBackend::Auto);let seat=Seat::P1;
 let mut s=board(&e,seat);let greef=unit(&e,&mut s,seat,"ASH_119");s.attack_base_history.record(seat);let a=e.card_action(&s,greef);s=e.apply(&s,&a).unwrap();click(&e,&mut s,"Create a Mandalorian token");let mando=e.arena_units(&s).into_iter().find(|&id|e.def(&s,id).code=="TOKEN_MANDALORIAN").unwrap();let shield=s.card(mando).shield;let attacker=unit(&e,&mut s,seat.other(),"LOF_254");fixed(&e,&s);attack(&e,&mut s,attacker,mando);assert_eq!(s.card(mando).shield,0);assert_eq!(s.card(shield).parent,0);let loaded=e.load_state(&e.snapshot(&s)).unwrap();assert_eq!(e.snapshot(&loaded),e.snapshot(&s));eprintln!("GREEF_RETIRED_SHIELD id={shield} native={:?} imported={:?} hashes={:x}/{:x}",s.card(shield),loaded.card(shield),s.zobrist(),loaded.zobrist());assert_eq!(loaded.zobrist(),s.zobrist(),"new actual native creator must retain its retired Shield physical identity");
}
#[test]
fn attack_base_vane_retired_advantage_fullhash_diagnostic() {
 let e=engine(RulesBackend::Auto);let seat=Seat::P1;let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat,"LOF_254");let base=s.player(seat.other()).base;attack(&e,&mut s,attacker,base);let old=e.upgrades(&s,vane).into_iter().find(|&id|e.def(&s,id).code=="TOKEN_ADVANTAGE").unwrap();fixed(&e,&s);click(&e,&mut s,"Pass");attack(&e,&mut s,vane,base);
 for _ in 0..8 {if s.is_action_window()||s.is_terminal(){break;}let action=e.legal_actions(&s).into_iter().find(|a|a.label!="Cancel").unwrap();s=e.apply(&s,&action).unwrap();}
 assert!(s.is_action_window());assert_eq!(s.card(old).parent,0);let live=e.upgrades(&s,vane).into_iter().filter(|&id|e.def(&s,id).code=="TOKEN_ADVANTAGE").collect::<Vec<_>>();assert_eq!(live.len(),1);assert_ne!(live[0],old);let loaded=e.load_state(&e.snapshot(&s)).unwrap();assert_eq!(e.snapshot(&loaded),e.snapshot(&s));eprintln!("VANE_RETIRED_ADVANTAGE id={old} native={:?} imported={:?} hashes={:x}/{:x}",s.card(old),loaded.card(old),s.zobrist(),loaded.zobrist());assert_eq!(loaded.zobrist(),s.zobrist(),"new actual native creator must retain its retired Advantage physical identity");
}
#[test]
fn attack_base_canonical_unique_both_owners_and_incoming_choices() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for code in ["ASH_119","ASH_031","ASH_144"] {for keep_old in [false,true] {
  let mut s=board(&e,seat);let old=unit(&e,&mut s,seat,code);let opponent=unit(&e,&mut s,seat.other(),code);let incoming=add(&e,&mut s,seat,code);s.player_mut(seat).hand.push(incoming).unwrap();fixed(&e,&s);let a=e.card_action(&s,incoming);assert!(e.legal_actions(&s).contains(&a));s=e.apply(&s,&a).unwrap();assert_eq!(s.decision,Decision::Unique);assert!(e.in_arena(&s,old)&&e.in_arena(&s,incoming));assert!(e.load_state(&e.snapshot(&s)).is_err());let remove=if keep_old {incoming}else{old};let a=e.card_action(&s,remove);assert!(e.legal_actions(&s).contains(&a));s=e.apply(&s,&a).unwrap();assert!(!e.in_arena(&s,remove));assert!(e.in_arena(&s,if keep_old {old}else{incoming}));assert!(e.in_arena(&s,opponent));fixed(&e,&s);
 }}}}
}
#[test]
fn attack_base_inventory_authority_is_physical_exact_not_selected_or_copied() {
 let mut e=engine(RulesBackend::Auto);let mut s=board(&e,Seat::P1);assert!(e.snapshot(&s).get("inactiveAllocatedCards").is_none(),"selected creator is not a physical creator");let donor=unit(&e,&mut s,Seat::P1,"ASH_037");let recipient=unit(&e,&mut s,Seat::P1,"LOF_254");let p=e.definitions[e.by_code["ASH_144"] as usize].programs.card[0];s.support.len=1;s.support.grants[0]=SupportGrant{target:recipient,donor,len:1,..Default::default()};s.support.grants[0].programs[0]=p;let v=e.snapshot(&s);assert!(v.get("inactiveAllocatedCards").is_none());assert!(e.load_state(&v).is_err(),"unscoped copied history programme must reject");s.support=Default::default();
 let source=add(&e,&mut s,Seat::P1,"ASH_119");s.player_mut(Seat::P1).hand.push(source).unwrap();let v=e.snapshot(&s);assert_eq!(v["inactivePhaseKeywordStateVersion"],json!(1));assert_eq!(v["inactiveAllocatedCards"],json!([]));fixed(&e,&s);let mut absent=v.clone();absent.as_object_mut().unwrap().remove("inactivePhaseKeywordStateVersion");absent.as_object_mut().unwrap().remove("inactiveAllocatedCards");assert!(e.load_state(&absent).is_err());
 // In-process adversarial definition mutation is not a constructible source
 // printing; it must not authorize this source-backed inventory protocol.
 let p=e.card_programs(&s,source)[0];e.catalogue.as_mut().unwrap().programs[p as usize].title.push_str(" unreviewed");assert!(e.snapshot(&s).get("inactiveAllocatedCards").is_none());
}
#[test]
fn attack_base_no_history_confirmation_phase_expiry_and_nonplayed_phase_effects() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  for confirm in [false,true] {
   let mut s=board(&e,seat);let greef=unit(&e,&mut s,seat,"ASH_119");let ready=e.ready_resources(&s,seat);let a=e.card_action(&s,greef);s=e.apply(&s,&a).unwrap();click(&e,&mut s,"(No effect) Create a Mandalorian token");click(&e,&mut s,if confirm {"Use it anyway"}else{"Cancel"});assert_eq!(s.card(greef).exhausted,confirm);assert_eq!(e.ready_resources(&s,seat),ready-usize::from(confirm));assert!(!e.arena_units(&s).into_iter().any(|id|e.def(&s,id).code=="TOKEN_MANDALORIAN"));fixed(&e,&s);
  }
  let mut s=board(&e,seat);let greef=unit(&e,&mut s,seat,"ASH_119");
  for (code,kind) in [("ASH_237",crate::player_phase_contract::Kind::ImperialDiscount),("ASH_248",crate::player_phase_contract::Kind::LowPrintedPowerReady)] {let source=unit(&e,&mut s,seat,code);s.source=source;s.ability_program=e.card_programs(&s,source)[0]+1;e.install_player_phase(&mut s,kind).unwrap();}
  s.source=0;s.ability_program=0;s.attack_base_history.record(seat);let rows=s.player_phase;let a=e.card_action(&s,greef);s=e.apply(&s,&a).unwrap();click(&e,&mut s,"Create a Mandalorian token");assert_eq!(s.player_phase,rows,"ability payment and Created entry do not consume Played-unit effects");let mando=e.arena_units(&s).into_iter().find(|&id|e.def(&s,id).code=="TOKEN_MANDALORIAN").unwrap();assert!(s.card(mando).exhausted);fixed(&e,&s);
  click(&e,&mut s,"Pass");click(&e,&mut s,"Pass");assert!(!s.attack_base_history.opponent_attacked(seat));assert_eq!(s.player_phase.len,0);
 }}
}
// These native builders exercise the actual printed on-attack/on-defense
// programmes. Independent paid source traces are replayed separately.
#[test]
fn attack_base_departed_defender_enfys_bounce_overwhelm_records_vane() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for overwhelm in [false,true] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat,"TWI_198");s.card_mut(attacker).damage=1;if overwhelm {let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,attacker);}let defender=unit(&e,&mut s,seat.other(),"SOR_095");fixed(&e,&s);let original=s;let before=s.next_id;attack(&e,&mut s,attacker,defender);
  if s.decision==Decision::TriggerOrder {click(&e,&mut s,"Return an enemy non-leader unit with less power than this unit to its owner's hand");}
  assert!(e.legal_actions(&s).contains(&e.card_action(&s,defender)));let a=e.card_action(&s,defender);s=e.apply(&s,&a).unwrap();assert!(s.player(seat.other()).hand.contains(defender));assert_eq!(s.card(s.player(seat.other()).base).damage,if overwhelm {7}else{0});assert_eq!(s.next_id,before+u16::from(overwhelm));assert_eq!(e.upgrades(&s,vane).iter().filter(|&&id|e.def(&s,id).code=="TOKEN_ADVANTAGE").count(),usize::from(overwhelm));fixed(&e,&s);assert_eq!(original.card(defender).damage,0);assert!(e.in_arena(&original,defender));
 }}}
}
#[test]
fn attack_base_departed_defender_train_on_attack_defeat_records_vane() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let attacker=unit(&e,&mut s,seat,"ASH_035");s.card_mut(attacker).damage=1;let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,attacker);let friendly=unit(&e,&mut s,seat,"LOF_254");s.card_mut(friendly).exhausted=true;let defender=unit(&e,&mut s,seat.other(),"SOR_095");fixed(&e,&s);let power=e.power(&s,attacker,true);attack(&e,&mut s,attacker,defender);assert!(e.legal_actions(&s).contains(&e.card_action(&s,defender)));let a=e.card_action(&s,defender);s=e.apply(&s,&a).unwrap();assert!(s.player(seat.other()).discard.contains(defender));assert_eq!(s.card(s.player(seat.other()).base).damage,power);assert!(e.upgrades(&s,vane).iter().any(|&id|e.def(&s,id).code=="TOKEN_ADVANTAGE"));fixed(&e,&s);
 }}
}
#[test]
fn attack_base_departed_defender_grievous_self_return_hera_actual_cap() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for clipped in [false,true] {
  let mut s=board(&e,seat);let hera=unit(&e,&mut s,seat,"ASH_031");s.card_mut(hera).damage=1;let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,hera);let defender=unit(&e,&mut s,seat.other(),"SEC_187");let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=8;if clipped {s.card_mut(enemy).damage=29;}fixed(&e,&s);attack(&e,&mut s,hera,defender);assert!(s.player(seat.other()).hand.contains(defender));assert_eq!(s.card(own).damage,if clipped {7}else{3});assert_eq!(s.card(enemy).damage,if clipped {30}else{5});assert_eq!(s.is_terminal(),clipped);if !clipped {fixed(&e,&s);}else {let copied=s;assert_eq!(copied,s);assert_eq!(copied.zobrist(),s.zobrist());} // Terminal reload is an existing separate protocol boundary, not asserted here.
 }}}
}
#[test]
fn attack_base_departed_defender_zero_damage_and_capacity_preflight() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let hera=unit(&e,&mut s,seat,"ASH_031");s.card_mut(hera).damage=1;let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,hera);s.card_mut(hera).phase_power=-5;let defender=unit(&e,&mut s,seat.other(),"SEC_187");let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=6;fixed(&e,&s);attack(&e,&mut s,hera,defender);assert!(s.player(seat.other()).hand.contains(defender));assert_eq!(s.card(own).damage,6);assert_eq!(s.card(enemy).damage,0);assert!(e.upgrades(&s,vane).is_empty());fixed(&e,&s);
  // Construct the internal departure continuation at its explicit bounded
  // window edge. An overflow must precede primary damage or capture mutation.
  let mut s=board(&e,seat);let hera=unit(&e,&mut s,seat,"ASH_031");s.card_mut(hera).damage=1;let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,hera);let defender=add(&e,&mut s,seat.other(),"SEC_187");s.player_mut(seat.other()).hand.push(defender).unwrap();fixed(&e,&s);s.source=hera;s.target=defender;s.attack_source=hera;s.attack_defender=defender;s.set_trigger_context(TriggerContext::Attack);e.start_attack_base_context(&mut s,hera,defender);s.death_pending.len=64;let before=s;assert!(e.process_all_triggers(&mut s).unwrap_err().contains("capacity"));assert_eq!(s,before);
 }}
}
#[test]
fn attack_base_departed_defender_pre_hera_post_earlier_vane_epoch_order() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for clipped in [false,true] {
  let mut s=board(&e,seat);let vane=unit(&e,&mut s,seat,"ASH_144");let hera=unit(&e,&mut s,seat,"ASH_031");s.card_mut(hera).damage=1;let mark=add(&e,&mut s,seat,"ASH_181");e.attach(&mut s,mark,hera);let defender=unit(&e,&mut s,seat.other(),"SEC_187");let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=8;if clipped {s.card_mut(enemy).damage=29;}assert!(s.registration_order.position(vane)<s.registration_order.position(hera));fixed(&e,&s);attack(&e,&mut s,hera,defender);assert_eq!(s.death_window.len,2);assert_eq!(s.death_window.entries[0].source,hera,"PRE unconditional listener precedes POST condition even with later registration");assert_eq!(s.death_window.entries[1].source,vane);let menus=e.legal_actions(&s);assert!(menus[0].label.contains("Heal"));assert!(menus[1].label.contains("vanes-snub-fighter"));
  for _ in 0..8 {if s.is_action_window() || s.death_depth==0 {break;}let a=e.legal_actions(&s).into_iter().find(|a|a.label!="Cancel"&&a.label!="Pass").unwrap();s=e.apply(&s,&a).unwrap();}assert_eq!(s.death_depth,0);assert_eq!(s.card(own).damage,if clipped {7}else{3});assert_eq!(s.card(enemy).damage,if clipped {30}else{5});assert_eq!(e.upgrades(&s,vane).iter().filter(|&&id|e.def(&s,id).code=="TOKEN_ADVANTAGE").count(),1);if !clipped {fixed(&e,&s);}else {assert!(s.is_terminal());}
 }}}
}

/// Completed ordinary attacks must be canonical action windows even when the
/// captured Restore amount was positive and combat itself dealt zero damage.
#[test]
fn restore_cleanup_public_zero_power_and_ordinary_controls_both_pools_seats_backends() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for producers in [false,true] {
  let mut raw=catalogue();if !producers {raw["cards"].as_array_mut().unwrap().retain(|v|!["ASH_119","ASH_031","ASH_144"].contains(&v["setCode"].as_str().unwrap_or("")));}
  let mut e=Engine::from_catalogue_value(&raw).unwrap();e.set_rules_backend(backend).unwrap();
  for seat in [Seat::P1,Seat::P2] {for code in ["SHD_055","LOF_254"] {
   let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,code);let own=s.player(seat).base;let enemy=s.player(seat.other()).base;
   s.card_mut(own).damage=5;let restore=e.keyword_amount(&s,source,"restore");let power=e.power(&s,source,true);
   assert_eq!(restore,if code=="SHD_055"{2}else{0});if restore==2 {assert_eq!(power,0);}
   fixed(&e,&s);let original=s;let original_hash=s.zobrist();let source_action=e.card_action(&s,source);
   let targeting=e.apply(&s,&source_action).unwrap();assert!(s==original);assert_eq!(s.zobrist(),original_hash);
   let base_action=e.card_action(&targeting,enemy);assert!(e.legal_actions(&targeting).contains(&base_action));
   let done=e.apply(&targeting,&base_action).unwrap();let copied_result=e.apply(&targeting,&base_action).unwrap();assert!(done==copied_result);
   assert_eq!(done.decision,Decision::Window);assert_eq!(done.attack_source,0);assert_eq!(done.attack_defender,0);
   assert_eq!(done.card(own).damage,5-restore);assert_eq!(done.card(enemy).damage,power as u16);assert!(done.card(source).exhausted);
   assert_eq!(done.attack_restore,0,"completed Restore amount must not leak into a public window");
   fixed(&e,&done);let imported=e.load_state(&e.snapshot(&done)).unwrap();assert_eq!(e.legal_actions(&imported),e.legal_actions(&done));
  }}
 }}
}

/// Restore stays captured while declaration choices and saved contexts are live;
/// the cleanup is owned by actual attack completion, not by healing itself.
#[test]
fn restore_cleanup_preserves_pending_restore_and_saved_attack_context() {
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
  let mut s=board(&e,seat);let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=15;
  let source=unit(&e,&mut s,seat,"TWI_150");let upgrade=add(&e,&mut s,seat,"HMW_096");e.attach(&mut s,upgrade,source);
  let ground=unit(&e,&mut s,seat.other(),"ASH_061");fixed(&e,&s);
  let targeting=e.apply(&s,&e.card_action(&s,source)).unwrap();let mut declared=e.apply(&targeting,&e.card_action(&targeting,enemy)).unwrap();
  assert_eq!(declared.decision,Decision::TriggerOrder);assert_eq!(declared.attack_restore,2);assert_ne!(declared.attack_source,0);assert_eq!(declared.card(own).damage,15);
  let before=declared;let before_hash=declared.zobrist();let context=ResolutionContext::capture(&declared);
  assert_eq!(context.attack_restore,2);let mut restored=declared;restored.attack_restore=0;context.restore(&mut restored);assert!(restored==declared);assert_eq!(restored.zobrist(),before_hash);
  assert!(e.legal_actions(&declared).iter().any(|a|a.label=="Restore 2"));assert!(declared==before);assert_eq!(declared.zobrist(),before_hash);
  click(&e,&mut declared,"Restore 2");
  // Healing first drops Saw below his condition before his other registration.
  assert_eq!(declared.card(own).damage,13);assert_eq!(declared.card(ground).damage,0);assert_eq!(declared.decision,Decision::Window);
  assert_eq!(declared.attack_restore,0);fixed(&e,&declared);
 }}
}
