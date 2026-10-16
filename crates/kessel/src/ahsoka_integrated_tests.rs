//! Complete generated Ahsoka leader, including the Support dependency.
use super::*;
fn start(e:&Engine,seat:Seat)->State {let mut s=board(e);s.round=8;s.active=seat;s.action_player=Some(seat);let leader=s.player(seat).leader;s.card_mut(leader).def=e.by_code["ASH_009"];s}
fn click(e:&Engine,s:&State,id:CardId)->State {let action=e.card_action(s,id);assert!(e.legal_actions(s).contains(&action),"missing {id}: {:?}",e.legal_actions(s));e.apply(s,&action).unwrap()}
fn stable(e:&Engine,s:&State) {let v=e.snapshot(s);let r=e.load_state(&v).unwrap();assert_eq!(e.snapshot(&r),v);assert_eq!(e.legal_actions(&r),e.legal_actions(s));assert_eq!(r.zobrist(),s.zobrist());}
#[test]
fn ahsoka_integrated_actual_front_exhaust_cost_any_owner_and_empty_maximum() {
 let e=engine();assert!(e.by_code.contains_key("ASH_009"));
 for seat in [Seat::P1,Seat::P2] {
  let mut empty=start(&e,seat);let leader=empty.player(seat).leader;unit(&e,&mut empty,seat.other(),"LOF_254");let no_effect=click(&e,&empty,leader);assert_eq!(no_effect.decision,Decision::AbilityNoEffect);assert!(!no_effect.card(leader).exhausted);let cancelled=named_button(&e,&no_effect,"Cancel");assert!(!cancelled.card(leader).exhausted);
  let mut s=start(&e,seat);let leader=s.player(seat).leader;let high=unit(&e,&mut s,seat,"SHD_189");let low=unit(&e,&mut s,seat.other(),"LOF_254");stable(&e,&s);
  let choosing=click(&e,&s,leader);assert_eq!(choosing.decision,Decision::AbilityTarget);assert!(!choosing.card(leader).exhausted);
  assert!(e.legal_actions(&choosing).contains(&e.card_action(&choosing,low)));assert!(!e.legal_actions(&choosing).contains(&e.card_action(&choosing,high)));
  let cancelled=named_button(&e,&choosing,"Cancel");assert!(!cancelled.card(leader).exhausted);assert_eq!(cancelled.card(low).phase_power,0);
  let done=click(&e,&choosing,low);assert!(done.card(leader).exhausted);assert_eq!(done.card(low).phase_power,2);assert_eq!(done.card(high).phase_power,0);stable(&e,&done);
 }
}
#[test]
fn ahsoka_integrated_actual_deployment_support_copies_attack_with_recipient_power_binding() {
 let e=engine();for seat in [Seat::P1,Seat::P2] {
  let mut s=start(&e,seat);let leader=s.player(seat).leader;
  for _ in 0..10 {let id=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).resources.push(id).unwrap();}
  let attacker=unit(&e,&mut s,seat,"LOF_256");let low=unit(&e,&mut s,seat,"LOF_254");s.card_mut(low).phase_power=-1;let too_high=unit(&e,&mut s,seat.other(),"LOF_254");s.card_mut(too_high).phase_power=1;
  let enemy=s.player(seat.other()).base;stable(&e,&s);let menu=click(&e,&s,leader);assert_eq!(menu.decision,Decision::LeaderMenu);
  let deployed=named_button(&e,&menu,"Deploy Ahsoka Tano");assert!(deployed.card(leader).deployed);assert_eq!(deployed.decision,Decision::Support);assert!(e.has_keyword(&deployed,leader,"support"));
  let targeting=click(&e,&deployed,attacker);assert_eq!(targeting.decision,Decision::AttackTarget);assert!(!e.support_programs(&targeting,attacker).is_empty());
  let choosing=click(&e,&targeting,enemy);assert_eq!(choosing.decision,Decision::AbilityTarget);assert!(e.legal_actions(&choosing).contains(&e.card_action(&choosing,low)));assert!(!e.legal_actions(&choosing).contains(&e.card_action(&choosing,too_high)),"copied source is the 1-power attacker, not Ahsoka");
  let done=click(&e,&choosing,low);assert_eq!(done.card(low).phase_power,1);assert_eq!(done.card(too_high).phase_power,1);assert_eq!(done.card(enemy).damage,1);assert!(done.card(attacker).exhausted);assert!(e.support_programs(&done,attacker).is_empty());stable(&e,&done);
 }
}
