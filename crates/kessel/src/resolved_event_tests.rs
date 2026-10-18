// Pinned-source semantic expectations: direct context.events defeat counts,
// pre-resolution LKI, and Damage.addDamage's returned (HP-capped) amount.
use super::*;

fn query_count()->Expr {serde_json::from_value(json!({"kind":"query","name":"resolvedDefeatedUnitCount","receiver":{"kind":"ref","path":["opponent"]},"args":[]})).unwrap()}
fn lki()->Expr {serde_json::from_value(json!({"kind":"ref","path":["event","lastKnownInformation","power"]})).unwrap()}
fn damage(target:&str,amount:u16)->Effect {Effect::Damage{target:Target::name(target),amount:Expr::number(amount as i64)}}
fn catalogue_entry(code:&str)->Value {let c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap().clone()}

#[test]
fn resolved_event_named_cards_admit_complete_sources_and_preserve_ir() {
    let e=engine();for code in ["LAW_044","JTL_104"] {assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));}
    let law=catalogue_entry("LAW_044");assert_eq!(law["abilities"]["card"][0]["then"]["effect"]["amount"]["name"],"resolvedDefeatedUnitCount");
    let raddus=catalogue_entry("JTL_104");assert_eq!(raddus["abilities"]["card"].as_array().unwrap().len(),2);assert_eq!(raddus["abilities"]["card"][1]["effect"]["amount"]["path"],json!(["event","lastKnownInformation","power"]));
    eprintln!("resolved-event State={} Card={} StepEventResult={} DeathTrigger={}",std::mem::size_of::<State>(),std::mem::size_of::<Card>(),std::mem::size_of::<StepEventResult>(),std::mem::size_of::<DeathTrigger>());
}

#[test]
fn resolved_event_actual_damage_is_zero_prevented_partial_overkill_and_terminal() {
    let e=engine();for (amount,prior,expected) in [(0,0,0),(2,0,2),(9,29,1),(9,30,0),(9,34,0)] {
        let mut s=board(&e);s.source=s.players[0].leader;s.ability_pc=9;let base=s.players[1].base;s.card_mut(base).damage=prior;
        let (legacy,_) = e.apply_effect(&mut s,&damage("enemyBase",amount)).unwrap();
        assert_eq!(s.event_result.damage_dealt,expected);assert_eq!(s.event_result.source,s.source);assert_eq!(s.event_result.pc,9);assert_eq!(s.event_result.defeated_units,[0,0]);
        assert_eq!(legacy,amount,"legacy pre-clamp delta and actual result have distinct meanings");assert_eq!(s.card(base).damage,(prior+amount).min(30),"winner checks clamp the base storage counter");
        if prior+amount>=30 {assert!(s.is_terminal());}
    }
    for (amount,shield,expected) in [(1,false,1),(9,false,4),(9,true,0)] {
        let mut s=board(&e);s.source=s.players[0].leader;let target=unit(&e,&mut s,Seat::P2,"LOF_256");s.ability_target=target;
        assert_eq!(e.hp(&s,target),4);if shield {e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("chosen"),amount:Expr::number(1)}).unwrap();}
        e.apply_effect(&mut s,&damage("chosen",amount)).unwrap();assert_eq!(s.event_result.damage_dealt,expected);
        assert_eq!(e.in_arena(&s,target),shield||amount<4);
    }
}

#[test]
fn resolved_event_group_aggregates_leaves_without_stale_result_and_uniform_prevention() {
    let e=engine();let mut s=board(&e);s.source=s.players[0].leader;s.event_result.damage_dealt=99;
    let group=Effect::Simultaneous{effects:vec![damage("ownBase",2),damage("enemyBase",3)]};s.event_group.active=true;
    assert!(e.generate_group(&mut s,&group).unwrap());assert!(e.resolve_group(&mut s,&group).unwrap());assert_eq!(s.event_result.damage_dealt,5);
    s.event_group=EventGroupState::default();let empty=Effect::Simultaneous{effects:vec![Effect::NoAction]};s.event_group.active=true;assert!(e.generate_group(&mut s,&empty).unwrap());assert!(e.resolve_group(&mut s,&empty).unwrap());assert_eq!(s.event_result.damage_dealt,0);
    s.event_group=EventGroupState::default();let a=unit(&e,&mut s,Seat::P1,"LOF_254");let b=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(b).damage=2;s.ability_target=a;e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("chosen"),amount:Expr::number(1)}).unwrap();
    let uniform:Effect=serde_json::from_value(json!({"kind":"damage","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true},"controllerOrder":"seatOrder"},"amount":{"kind":"const","value":3}})).unwrap();
    s.event_group.active=true;assert!(e.generate_group(&mut s,&uniform).unwrap());assert!(s.event_group.uniform_damage.active);assert!(e.resolve_group(&mut s,&uniform).unwrap());assert_eq!(s.event_result.damage_dealt,2);assert!(e.in_arena(&s,a));assert!(!e.in_arena(&s,b));
}

#[test]
fn resolved_event_neutral_steps_clear_actual_results_and_resolution_context_restores() {
    let e=engine();let mut s=board(&e);s.source=s.players[0].leader;s.ability_pc=7;e.apply_effect(&mut s,&damage("enemyBase",2)).unwrap();
    s.last_known_source=s.source;s.last_known_power=13;let saved=ResolutionContext::capture(&s);let hash=s.zobrist();
    s.source=s.players[1].leader;s.ability_pc=11;e.apply_effect(&mut s,&Effect::Draw{target:Target::name("ownPlayer"),amount:Expr::number(0)}).unwrap();assert_eq!(s.event_result.damage_dealt,0);assert_ne!(s.zobrist(),hash);
    saved.restore(&mut s);assert_eq!(s.event_result.damage_dealt,2);assert_eq!(s.event_result.pc,7);assert_eq!(s.last_known_power,13);assert_eq!(s.zobrist(),hash);
    let copy=s;assert_eq!(copy,s);let before=s.zobrist();s.death_pending.entries[0].last_known_power=17;assert_ne!(before,s.zobrist());
}

#[test]
fn resolved_event_single_reactor_counts_enemy_units_tokens_and_deployed_leader_only() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=seat;let source=add(&e,&mut s,seat,"LAW_044");s.player_mut(seat).discard.push(source).unwrap();s.source=source;
        let own=unit(&e,&mut s,seat,"LOF_254");let enemy=unit(&e,&mut s,seat.other(),"LOF_256");let token=unit(&e,&mut s,seat.other(),"TOKEN_XWING");let leader=s.player(seat.other()).leader;s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();
        s.ability_target=enemy;e.apply_effect(&mut s,&Effect::GiveExperience{target:Target::name("chosen"),amount:Expr::number(2)}).unwrap();e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("chosen"),amount:Expr::number(1)}).unwrap();
        let program=e.card_programs(&s,source)[0];e.begin_program(&mut s,program,ProgramContinuation::Triggers).unwrap();
        assert!(!e.in_arena(&s,own));assert!(!e.in_arena(&s,enemy));assert!(!e.in_arena(&s,token));assert!(!e.in_arena(&s,leader));assert!(!s.card(leader).deployed);assert_eq!(s.card(s.player(seat.other()).base).damage,3);assert_eq!(s.card(s.player(seat).base).damage,0);
    }
}

#[test]
fn resolved_event_direct_defeat_summary_ignores_framework_deaths_and_already_departed() {
    let e=engine();let mut s=board(&e);let source=add(&e,&mut s,Seat::P1,"LAW_044");s.source=source;
    let a=unit(&e,&mut s,Seat::P1,"LOF_254");let b=unit(&e,&mut s,Seat::P2,"LOF_256");let gone=unit(&e,&mut s,Seat::P2,"LOF_256");e.defeat(&mut s,gone).unwrap();
    let effect:Effect=serde_json::from_value(catalogue_entry("LAW_044")["abilities"]["card"][0]["effect"].clone()).unwrap();e.apply_effect(&mut s,&effect).unwrap();assert_eq!(s.event_result.defeated_units,[1,1]);assert_eq!(e.eval(&s,source,0,&query_count()),Eval::Number(1));assert!(!e.in_arena(&s,a));assert!(!e.in_arena(&s,b));
    // A later damage-driven defeat belongs to a new window and must not retain count.
    let later=unit(&e,&mut s,Seat::P2,"LOF_256");s.ability_target=later;e.apply_effect(&mut s,&damage("chosen",3)).unwrap();assert_eq!(s.event_result.defeated_units,[0,0]);
}

#[test]
fn resolved_event_raddus_captures_complete_pre_removal_power_for_each_batch_order() {
    let e=engine();for reverse in [false,true] {for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=seat;let raddus=unit(&e,&mut s,seat,"JTL_104");let supremacy=unit(&e,&mut s,seat,"LOF_089");let enemy=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(enemy).phase_hp=30;
        s.source=raddus;s.ability_target=raddus;e.apply_effect(&mut s,&Effect::GiveExperience{target:Target::name("chosen"),amount:Expr::number(2)}).unwrap();s.card_mut(raddus).phase_power=3;assert_eq!(e.power(&s,raddus,false),19);
        let ids=if reverse {vec![raddus,supremacy]}else{vec![supremacy,raddus]};e.defeat_many(&mut s,&ids).unwrap();
        let captured=s.death_pending.entries[..s.death_pending.len as usize].iter().find(|x|x.source==raddus).unwrap();assert_eq!(captured.last_known_power,19);assert_eq!(e.power(&s,raddus,false),8);
        let c=e.captured_trigger_context(&s,*captured);assert_eq!(e.eval(&c,raddus,0,&lki()),Eval::Number(19));assert!(e.death_title(&s,*captured,false).contains("19 damage"));
        e.finish(&mut s).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.source,raddus);let action=e.card_action(&s,enemy);s=e.apply(&s,&action).unwrap();assert_eq!(s.card(enemy).damage,19);assert!(s.player(seat).discard.contains(raddus));
    }}
}

#[test]
fn resolved_event_raddus_zero_power_and_attack_raid_use_capture_not_discard_stats() {
    let e=engine();for (phase,raid,expected) in [(-50,0,0),(0,3,11)] {
        let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"JTL_104");let enemy=unit(&e,&mut s,Seat::P2,"LOF_256");s.source=source;s.attack_source=source;s.attack_defender=enemy;s.card_mut(source).phase_power=phase;
        if raid>0 {e.apply_effect(&mut s,&Effect::GainKeyword{target:Target::name("self"),keyword:"raid".into(),amount:Expr::number(raid),duration:"phase".into()}).unwrap();}
        e.defeat(&mut s,source).unwrap();let t=s.death_pending.entries[0];assert_eq!(t.last_known_power,expected);let c=e.captured_trigger_context(&s,t);assert_eq!(e.eval(&c,source,0,&lki()),Eval::Number(expected as i64));
        if expected==0 {assert!(e.death_has_effect(&s,t),"target feasibility is evaluated without must-change-state");assert!(!e.program_has_effect(&c,source,t.program),"resolving zero damage requires a state change");}
    }
}

#[test]
fn resolved_event_controlled_trait_includes_base_front_leader_and_enemy_host_upgrade() {
    let mut e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"JTL_104");assert!(!e.has_keyword(&s,source,"sentinel"));
    let base=s.players[0].base;let definition=s.card(base).def as usize;e.definitions[definition].traits.push("resistance".into());assert!(e.has_keyword(&s,source,"sentinel"));e.definitions[definition].traits.retain(|t|t!="resistance");
    let leader=s.players[0].leader;let definition=s.card(leader).def as usize;e.definitions[definition].traits.push("resistance".into());assert!(e.has_keyword(&s,source,"sentinel"));
    e.catalogue.as_mut().unwrap().metadata.get_mut("SOR_014").unwrap()["backSideTraits"]=json!(["jedi"]);s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();assert!(!e.has_keyword(&s,source,"sentinel"));
    let host=unit(&e,&mut s,Seat::P2,"LOF_256");let upgrade=add(&e,&mut s,Seat::P1,"IC27_052");let definition=s.card(upgrade).def as usize;e.definitions[definition].traits.push("resistance".into());e.attach(&mut s,upgrade,host);assert!(e.has_keyword(&s,source,"sentinel"));
    s.card_mut(upgrade).owner=Seat::P2;assert!(!e.has_keyword(&s,source,"sentinel"));s.card_mut(upgrade).owner=Seat::P1;e.leave_play_primary(&mut s,upgrade).unwrap();assert!(!e.has_keyword(&s,source,"sentinel"));
}

#[test]
fn resolved_event_raw_ir_rejects_unbound_capture_and_incomplete_registration() {
    let law=catalogue_entry("LAW_044")["abilities"]["card"][0].clone();let compile=|v:Value|crate::ability_ir::Program::compile(&serde_json::from_value(v).unwrap());assert!(compile(law.clone()).is_ok());
    for field in ["effect","then"] {let mut bad=law.clone();if field=="effect" {bad["effect"]["target"]["receiver"]["path"]=json!(["player"]);}else{bad["then"]["then"]=json!({"title":"Unsupported next","effect":{"kind":"noAction"}});}assert!(compile(bad).is_err());}
    let raddus=catalogue_entry("JTL_104")["abilities"]["card"][1].clone();assert!(compile(raddus.clone()).is_ok());for trigger in ["event","whenPlayed","onAttack"] {let mut bad=raddus.clone();bad["trigger"]=json!(trigger);assert!(compile(bad).is_err());}
    let mut bad=raddus;bad["effect"]["amount"]["path"]=json!(["event","lastKnownInformation","hp"]);assert!(compile(bad).is_err());
}

#[test]
fn resolved_event_terminal_window_keeps_mandatory_raddus_damage_and_winner() {
    // Synthetic supported simultaneous damage window, exercising the actual
    // printed Raddus registration after its controller's base is defeated.
    let e=engine();let mut s=board(&e);let raddus=unit(&e,&mut s,Seat::P1,"JTL_104");let enemy=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(enemy).phase_hp=20;s.source=s.players[0].leader;s.ability_target=raddus;
    let group=Effect::Simultaneous{effects:vec![damage("ownBase",30),damage("chosen",6)]};s.event_group.active=true;s.event_group.context=Selection::one(raddus);assert!(e.generate_group(&mut s,&group).unwrap());assert!(e.resolve_group(&mut s,&group).unwrap());s.event_group=EventGroupState::default();assert!(s.is_terminal());let winner=s.outcome;
    e.finish(&mut s).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.source,raddus);assert_eq!(s.outcome,winner);assert!(e.legal_actions(&s).is_empty(),"a terminal prompt must not allow a new external action");assert_eq!(e.eval(&s,raddus,enemy,&lki()),Eval::Number(8));
    // Internal continuation diagnostic only: the public action protocol remains terminal.
    let action=e.card_action(&s,enemy);s=e.apply_ability_action(&s,&action).unwrap();assert_eq!(s.card(enemy).damage,8);assert_eq!(s.outcome,winner);assert_eq!(s.decision,Decision::Terminal);
}

#[test]
fn resolved_event_damage_continuation_uses_actual_result_and_nested_death_frame() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.rules_backend=backend;let mut s=board(&e);let source=s.players[0].leader;let raddus=unit(&e,&mut s,Seat::P2,"JTL_104");let enemy=unit(&e,&mut s,Seat::P1,"LOF_256");s.card_mut(enemy).phase_hp=20;s.card_mut(raddus).damage=5;s.source=source;s.ability_target=raddus;
        let program=death_program(&mut e,&s,source,json!({"trigger":"action","title":"Result diagnostic","targetResolver":{"owner":"opponent","type":"unit","zone":"arena"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":7}},"then":{"title":"Actual result","effect":{"kind":"damage","target":"enemyBase","amount":{"kind":"ref","path":["event","damageDealt"]}}}}));
        e.begin_program(&mut s,program,ProgramContinuation::Triggers).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);let action=e.card_action(&s,raddus);s=e.apply(&s,&action).unwrap();assert_eq!(s.card(s.players[1].base).damage,1);assert_eq!(s.source,raddus);assert_eq!(s.decision,Decision::AbilityTarget);assert!(s.death_depth>0);
        let parent=s.death_frames[s.death_depth as usize-1].context;assert_eq!(parent.event_result.damage_dealt,1);assert_eq!(parent.event_result.source,source);assert_eq!(parent.event_result.program,program+1);
        let action=e.card_action(&s,enemy);s=e.apply(&s,&action).unwrap();assert_eq!(s.card(enemy).damage,8);
    }
}

#[test]
fn resolved_event_scalar_defeat_excludes_framework_sibling_and_duplicate_event() {
    let e=engine();let mut s=board(&e);let source=s.players[0].leader;s.source=source;let aura=unit(&e,&mut s,Seat::P2,"LOF_089");let target=unit(&e,&mut s,Seat::P2,"ASH_201");s.card_mut(target).damage=5;assert!(e.hp(&s,target)>5);s.ability_target=aura;
    e.apply_effect(&mut s,&Effect::Defeat{target:Target::name("chosen")}).unwrap();assert!(!e.in_arena(&s,aura));assert!(!e.in_arena(&s,target));assert_eq!(s.event_result.defeated_units,[0,1]);
    e.apply_effect(&mut s,&Effect::Defeat{target:Target::name("chosen")}).unwrap();assert_eq!(s.event_result.defeated_units,[0,0]);
}

#[test]
fn resolved_event_raddus_auto_interpreter_and_generated_damage_match() {
    let mut states=vec![];for backend in [RulesBackend::Auto,RulesBackend::Interpreter,RulesBackend::Aot] {let mut e=engine();e.rules_backend=backend;let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"JTL_104");let target=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(target).phase_hp=20;s.card_mut(source).phase_power=2;e.defeat(&mut s,source).unwrap();e.finish(&mut s).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);let action=e.card_action(&s,target);s=e.apply(&s,&action).unwrap();assert_eq!(s.card(target).damage,10);states.push(s);}
    assert_eq!(states[0],states[1]);assert_eq!(states[1],states[2]);
}

#[test]
fn resolved_event_damage_sum_covers_full_bounded_group_without_u16_saturation() {
    let e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LOF_256");let target=unit(&e,&mut s,Seat::P2,"LOF_256");s.source=source;s.ability_target=target;for id in [source,target] {s.card_mut(id).phase_hp=65531;assert_eq!(e.hp(&s,id),65535);}e.check_stat_capacity(&s).unwrap();
    let group=Effect::Simultaneous{effects:vec![damage("self",65535),damage("chosen",65535)]};s.event_group.active=true;s.event_group.context=Selection::one(target);assert!(e.generate_group(&mut s,&group).unwrap());assert!(e.resolve_group(&mut s,&group).unwrap());assert_eq!(s.event_result.damage_dealt,131070);assert!(!e.in_arena(&s,source));assert!(!e.in_arena(&s,target));assert_eq!(s.event_result.source,source);
}

#[test]
fn resolved_event_neutral_arena_keyword_and_phase_batches_replace_preceding_result() {
    let e=engine();let mut s=board(&e);let source=add(&e,&mut s,Seat::P1,"JTL_077");s.players[0].discard.push(source).unwrap();let target=unit(&e,&mut s,Seat::P1,"LOF_112");s.source=source;
    e.apply_effect(&mut s,&damage("enemyBase",3)).unwrap();assert_eq!(s.event_result.damage_dealt,3);e.apply_effect(&mut s,&arena_keyword4_effect("JTL_077")).unwrap();assert_eq!(s.event_result.damage_dealt,0);assert_eq!(s.event_result.defeated_units,[0,0]);
    e.apply_effect(&mut s,&damage("enemyBase",2)).unwrap();assert_eq!(s.event_result.damage_dealt,2);s.ability_target=target;let phase:Effect=serde_json::from_value(json!({"kind":"phaseModifiers","effects":[{"kind":"modifyStats","target":"chosen","duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":1}}]})).unwrap();e.apply_effect(&mut s,&phase).unwrap();assert_eq!(s.event_result.damage_dealt,0);assert_eq!(s.event_result.defeated_units,[0,0]);
}

#[test]
fn resolved_event_distribution_completion_clears_prior_actual_result_before_next_prompt() {
    // Synthetic typed program diagnostic for a prompted neutral instruction.
    let mut e=engine();let mut s=board(&e);let source=s.players[0].leader;let first=unit(&e,&mut s,Seat::P1,"LOF_256");let second=unit(&e,&mut s,Seat::P1,"LOF_256");for id in [first,second] {s.card_mut(id).damage=1;}s.source=source;
    let program=death_program(&mut e,&s,source,json!({"trigger":"action","title":"Neutral distribution diagnostic","effect":{"kind":"distributeHealing","amount":{"kind":"const","value":1},"maxTargets":1,"targetResolver":{"owner":"self","type":"unit","zone":"arena"}},"then":{"title":"Next target","targetResolver":{"owner":"self","type":"unit","zone":"arena"},"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":1}}}}));e.begin_program(&mut s,program,ProgramContinuation::Triggers).unwrap();assert_eq!(s.decision,Decision::AbilityDistribute);s.event_result.damage_dealt=17;
    let action=e.legal_actions(&s).into_iter().find(|a|a.kind=="distribute"&&a.arg==Some(json!([[format!("c{}",first-1),1]]))).unwrap();s=e.apply(&s,&action).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.event_result.damage_dealt,0);assert_eq!(s.event_result.defeated_units,[0,0]);assert_eq!(s.event_result.source,source);assert_eq!(s.card(first).damage,0);assert_eq!(s.card(second).damage,1);
}

#[test]
fn resolved_event_payload_is_not_available_before_its_originating_step() {
    let mut law=catalogue_entry("LAW_044")["abilities"]["card"][0].clone();law["contextTitle"]=json!([{"kind":"query","name":"resolvedDefeatedUnitCount","receiver":{"kind":"ref","path":["opponent"]},"args":[]}]);let a:crate::ability_ir::Ability=serde_json::from_value(law).unwrap();assert!(crate::ability_ir::Program::compile(&a).is_err());
    let mut damage=json!({"trigger":"action","title":"Unbound diagnostic","effect":{"kind":"damage","target":"enemyBase","amount":{"kind":"const","value":1}},"then":{"title":"Use captured result","effect":{"kind":"damage","target":"ownBase","amount":{"kind":"ref","path":["event","damageDealt"]}}}});let valid:crate::ability_ir::Ability=serde_json::from_value(damage.clone()).unwrap();assert!(crate::ability_ir::Program::compile(&valid).is_ok());damage["condition"]=json!({"kind":"binary","op":">","left":{"kind":"ref","path":["event","damageDealt"]},"right":{"kind":"const","value":0}});let invalid:crate::ability_ir::Ability=serde_json::from_value(damage).unwrap();assert!(crate::ability_ir::Program::compile(&invalid).is_err());
}
