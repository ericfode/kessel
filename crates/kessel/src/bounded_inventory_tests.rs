use super::*;
fn engine()->Engine {Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()}
fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;*s.card_mut(id)=Card{present:true,def:*e.by_code.get(code).unwrap_or_else(||panic!("{code}: {:?}",e.rejected_cards().unwrap().get(code))),owner:seat,..Card::default()};id}
fn unit(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=add(e,s,seat,code);e.add_arena(s,id).unwrap();id}
fn board(e:&Engine)->State {let mut s=State::default();s.hidden_history_exact=true;s.force_state_exact=true;for(seat,code)in[(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")]{let base=add(e,&mut s,seat,"ASH_019");let leader=add(e,&mut s,seat,code);s.player_mut(seat).base=base;s.player_mut(seat).leader=leader;s.player_mut(seat).force_id=s.next_id;s.next_id+=1;}s}
#[test]fn bounded_inventory_leader_traits_use_controller_presence_deployment_and_union(){
 let e=engine();assert!(e.by_code.contains_key("LAW_152"));
 for seat in [Seat::P1,Seat::P2]{let mut s=board(&e);let source=unit(&e,&mut s,seat,"LAW_152");let q=Expr::Query{name:"leaderTraits".into(),receiver:Box::new(Expr::Ref{path:vec!["player".into()]}),args:vec![]};
 let leader=s.player(seat).leader;let expected=e.def(&s,leader).traits.iter().cloned().map(Eval::Text).collect::<Vec<_>>();let Eval::Array(actual)=e.eval(&s,source,0,&q)else{panic!()};assert!(expected.iter().all(|t|actual.contains(t)));assert_eq!(actual.len(),expected.len());
 s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();assert_eq!(e.eval(&s,source,0,&q),Eval::Array(actual.clone()));
 let enemy=s.player(seat.other()).leader;s.card_mut(enemy).deployed=true;e.add_arena(&mut s,enemy).unwrap();s.player_mut(seat.other()).ground.remove(enemy);s.card_mut(enemy).owner=seat;s.player_mut(seat).ground.push(enemy).unwrap();let Eval::Array(union)=e.eval(&s,source,0,&q)else{panic!()};for t in &e.def(&s,enemy).traits{assert!(union.contains(&Eval::Text(t.clone())));}
 e.leave_play_primary(&mut s,enemy).unwrap();s.card_mut(enemy).owner=seat.other();assert_eq!(e.eval(&s,source,0,&q),Eval::Array(actual));
 }
}
#[test]fn bounded_inventory_leader_traits_real_attack_grants_enemy_shared_trait_and_excludes_source(){
 let e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LAW_152");let recipient=unit(&e,&mut s,Seat::P2,"SOR_095");let leader=s.players[0].leader;
 let cat:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let value=&cat["cards"].as_array().unwrap().iter().find(|c|c["setCode"]=="LAW_152").unwrap()["abilities"]["card"][0];let ability:crate::ability_ir::Ability=serde_json::from_value(value.clone()).unwrap();let r=ability.target_resolver.unwrap();let targets=e.matching_targets(&s,source,&r);assert!(targets.contains(&recipient));assert!(!targets.contains(&source));assert!(!targets.contains(&leader));
 s.active=Seat::P1;s.source=source;let p=e.def(&s,source).programs.card[0];e.begin_program(&mut s,p,ProgramContinuation::Finish).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);let prompt=s;let token=prompt.next_id;let done=e.apply(&prompt,&e.card_action(&prompt,recipient)).unwrap();assert_eq!(done.card(token).parent,recipient);assert_eq!(done.card(token).owner,Seat::P2);assert_eq!(e.token_count(&done,done.card(recipient).experience),1);
}
#[test]fn bounded_inventory_leader_traits_reject_wrong_placement_and_source_topology(){
 let original:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();for(field,value)in[("optional",json!(false)),("trigger",json!("whenPlayed")),("condition",json!({"kind":"query","name":"leaderTraits","receiver":{"kind":"ref","path":["player"]},"args":[]})),("then",json!({"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}))]{let mut c=original.clone();let entry=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="LAW_152").unwrap();entry["abilities"]["card"][0][field]=value;assert!(!Engine::from_catalogue_value(&c).unwrap().by_code.contains_key("LAW_152"),"{field}");}
 let mut c=original.clone();let entry=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="LAW_152").unwrap();entry["baseClass"]=json!("TokenUnitCard");entry["card"]["types"]=json!(["token","unit"]);assert!(!Engine::from_catalogue_value(&c).unwrap().by_code.contains_key("LAW_152"));
}

#[test]fn bounded_inventory_leader_traits_distinct_deployed_side_is_live(){
 let mut c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let leader=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="SOR_014").unwrap();leader["card"]["backSideTraits"]=json!(["separatist","sith"]);let e=Engine::from_catalogue_value(&c).unwrap();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LAW_152");let id=s.players[0].leader;let q=Expr::Query{name:"leaderTraits".into(),receiver:Box::new(Expr::Ref{path:vec!["player".into()]}),args:vec![]};let before=e.eval(&s,source,0,&q);s.card_mut(id).deployed=true;e.add_arena(&mut s,id).unwrap();assert_eq!(e.eval(&s,source,0,&q),Eval::Array(vec![Eval::Text("separatist".into()),Eval::Text("sith".into())]));e.leave_play_primary(&mut s,id).unwrap();assert_eq!(e.eval(&s,source,0,&q),before);
}

#[test]fn bounded_inventory_ghost_copies_live_keywords_numeric_presence_loss_and_source_identity(){
 let e=engine();assert!(e.by_code.contains_key("JTL_053"),"{:?}",e.rejected_cards().unwrap().get("JTL_053"));
 for seat in[Seat::P1,Seat::P2]{let mut s=board(&e);let source=unit(&e,&mut s,seat,"JTL_053");let leader=unit(&e,&mut s,seat,"LOF_046");let other=unit(&e,&mut s,seat.other(),"JTL_053");
 assert!(!e.has_keyword(&s,leader,"sentinel"));let shield=add(&e,&mut s,seat,"TOKEN_SHIELD");e.attach(&mut s,shield,source);assert!(e.has_keyword(&s,leader,"sentinel"));assert!(!e.has_keyword(&s,other,"sentinel"));
 for (index,amount)in[(6,0),(7,3)]{s.card_mut(source).add_phase_keyword(index);s.card_mut(source).phase_keyword_values[crate::ability_ir::numeric_keyword_slot(index).unwrap()]=amount;}
 assert!(e.has_keyword(&s,leader,"raid"));assert_eq!(e.raw_keyword_amount(&s,leader,"raid"),e.def(&s,leader).raid as i64);assert_eq!(e.raw_keyword_amount(&s,leader,"restore"),e.def(&s,leader).restore as i64+3);
 s.card_mut(leader).add_lost_keyword(7);assert!(!e.has_keyword(&s,leader,"restore"));assert_eq!(e.raw_keyword_amount(&s,leader,"restore"),0);
 e.leave_play_primary(&mut s,shield).unwrap();assert!(!e.has_keyword(&s,leader,"sentinel"));s.card_mut(source).phase_keyword_values=[0;2];s.card_mut(source).phase_keyword_presence=0;assert!(!e.has_keyword(&s,leader,"raid"));
 s.card_mut(source).add_phase_keyword(4);assert!(e.has_keyword(&s,leader,"grit"));e.leave_play_primary(&mut s,source).unwrap();assert!(!e.has_keyword(&s,leader,"grit"));assert_ne!(source,other);
 }
}
#[test]fn bounded_inventory_ghost_rejects_unknown_domain_cycles_and_raw_capture_expansion(){
 let original:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();for(domain,value)in[(true,json!(["sentinel","mystery"])),(false,json!("phase"))]{let mut c=original.clone();let card=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="JTL_053").unwrap();card["abilities"]["card"][0]["effect"][if domain{"domain"}else{"duration"}]=value;assert!(!Engine::from_catalogue_value(&c).unwrap().by_code.contains_key("JTL_053"));}
 let e=engine();let mut s=board(&e);let first=unit(&e,&mut s,Seat::P1,"JTL_053");let second=unit(&e,&mut s,Seat::P1,"JTL_053");assert!(e.check_keyword_copy_graph(&s).unwrap_err().contains("cycle"));assert_ne!(first,second);
 // Unknown inventory cannot be silently dropped even when metadata bypasses normal admission in a synthetic engine.
 let mut e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"JTL_053");let leader=s.players[0].leader;s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();e.definitions[s.card(leader).def as usize].traits=vec!["spectre".into()];e.catalogue.as_mut().unwrap().metadata.get_mut("JTL_053").unwrap()["keywords"]=json!(["smuggle"]);assert!(e.check_keyword_copy_graph(&s).unwrap_err().contains("unsupported inventory"));assert_eq!(s.card(source).owner,Seat::P1);
}

#[test]fn bounded_inventory_copy_expiry_stacking_and_snapshot_preserve_exact_presence(){
 let e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"JTL_053");let recipient=unit(&e,&mut s,Seat::P1,"LOF_046");s.source=source;s.ability_target=source;
 for(keyword,amount)in[("raid",0),("raid",2),("raid",3),("sentinel",1),("sentinel",1)]{let effect=Effect::GainKeyword{target:Target::name("chosen"),keyword:keyword.into(),amount:Expr::Const{value:json!(amount)},duration:"phase".into()};e.apply_effect(&mut s,&effect).unwrap();}
 assert_eq!(e.raw_keyword_amount(&s,recipient,"raid"),5);assert_eq!(e.raw_keyword_amount(&s,recipient,"sentinel"),1);assert!(e.has_keyword(&s,recipient,"raid"));assert_eq!(e.keyword_inventory_len(&s,recipient),2);let copied=s;assert_eq!(copied.zobrist(),s.zobrist());
 // Phase expiry clears the source inventories; no copied lasting effect is installed on the recipient.
 s.card_mut(source).phase_keyword_presence=0;s.card_mut(source).phase_keyword_values=[0;2];assert_eq!(e.keyword_inventory_len(&s,recipient),0);assert!(!e.has_keyword(&s,recipient,"raid"));assert_ne!(s.zobrist(),copied.zobrist());
}

#[test]fn bounded_inventory_leader_traits_rejects_missing_predicate_leaves(){
 let original:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();for which in[0,1]{let mut c=original.clone();let entry=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="LAW_152").unwrap();let predicate=&mut entry["abilities"]["card"][0]["targetResolver"]["predicate"];match which{0=>predicate["left"]["left"]=json!({"kind":"const","value":true}),1=>predicate["left"]["right"]=json!({"kind":"const","value":true}),_=>predicate["right"]=json!({"kind":"const","value":true})};assert!(!Engine::from_catalogue_value(&c).unwrap().by_code.contains_key("LAW_152"),"leaf {which}");}
}
