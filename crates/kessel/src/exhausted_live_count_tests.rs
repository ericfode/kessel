//! Native unit boundaries for the prospective shared integration. These tests
//! are not source-engine recordings or admission/conformance receipts.
use super::*;
use crate::exhausted_live_count_contract as contract;
fn raw_entry()->Value {committed_catalogue()["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="ASH_035").unwrap().clone()}
fn native(backend:RulesBackend)->Engine {let mut e=engine();e.set_rules_backend(backend).unwrap();assert!(e.by_code.contains_key("ASH_035"),"{:?}",e.rejected_cards().unwrap().get("ASH_035"));e}
fn count(e:&Engine,s:&State,source:CardId)->i64 {match e.eval(s,source,0,&contract::count_expr()){Eval::Number(n)=>n,other=>panic!("{other:?}")}}
fn prompt_read(e:&Engine,s:&State,source:CardId)->String {contract::title_parts().iter().map(|p|match e.eval(s,source,0,p){Eval::Text(t)=>t,Eval::Number(n)=>n.to_string(),other=>panic!("{other:?}")}).collect()}
fn choose(e:&Engine,s:&State,id:CardId)->State {let a=e.card_action(s,id);assert!(e.legal_actions(s).contains(&a),"{id}: {:?}",e.legal_actions(s));e.apply(s,&a).unwrap()}
#[test]
fn exhausted_live_count_complete_raw_pair_and_typed_program_guards() {
    let original=raw_entry();assert!(contract::entry(&original));assert!(contract::prepare(&original).unwrap().is_some());
    // No external card-code behavior dispatch; the complete source and metadata
    // profile, not this opaque catalogue key, owns the projection.
    let mut renamed=original.clone();renamed["setCode"]=json!("ZZZ_999");assert!(contract::entry(&renamed));
    for (pointer,value) in [
        ("/baseClass",json!("LeaderCard")),("/card/types",json!(["unit","upgrade"])),
        ("/card/cost",json!(6)),("/card/power",json!(9)),("/card/hp",json!(6)),("/card/traits",json!([])),
        ("/card/keywords",json!(["sentinel"])),("/keywordValues",json!({"support":1})),
        ("/sourceHash",json!("0000000000000000000000000000000000000000000000000000000000000000")),
        ("/diagnostics",json!(["unreviewed"])),("/historyHelpers",json!([])),
        ("/abilities/card/0/condition/right/value",json!(1)),("/abilities/card/1/targetResolver/arena",json!("space")),
        ("/abilities/card/1/targetResolver/owner",json!("self")),("/abilities/card/1/costs",json!([{"kind":"exhaustSelf"}])),
        ("/historyHelpers/0/body/statements/0/value/receiver/filter/condition/body/path/1",json!("ready")),
        ("/historyHelpers/0/body/statements/0/value/receiver/receiver/steps/0/key",json!("opponent")),
        ("/historyHelpers/1/body/statements/0/value/left/value",json!(3)),
    ] {let mut bad=original.clone();*bad.pointer_mut(pointer).unwrap()=value;assert!(!contract::entry(&bad),"{pointer}");assert!(contract::prepare(&bad).is_err(),"{pointer}");}
    for pointer in ["", "/card", "/abilities/card/0", "/historyHelpers/0/body"] {let mut bad=original.clone();bad.pointer_mut(pointer).unwrap().as_object_mut().unwrap().insert("unknown".into(),json!(false));assert!(!contract::entry(&bad),"{pointer}");}
    for side in ["leader","unit"] {let mut bad=original.clone();bad["abilities"][side]=json!([bad["abilities"]["card"][1].clone()]);assert!(contract::prepare(&bad).is_err());}
    for n in [0,1] {let mut bad=original.clone();bad["abilities"]["card"].as_array_mut().unwrap().remove(n);assert!(contract::prepare(&bad).is_err());}
    let sides=contract::normalized_sides();let programs=sides.card.iter().map(|a|crate::ability_ir::Program::compile(a).unwrap()).collect::<Vec<_>>();
    assert!(programs[1].executable.is_ok(),"prospective generated attack path: {:?}",programs[1].executable);
    assert!(contract::compiled_source(&original["card"],&programs.iter().collect::<Vec<_>>()));
    assert!(!contract::compiled_source(&original["card"],&[&programs[0]]));assert!(!contract::compiled_source(&original["card"],&[&programs[1],&programs[0]]));
    for a in &sides.card {assert!(contract::registration(a));let mut bad=a.clone();bad.optional=true;assert!(!contract::registration(&bad));assert!(crate::ability_ir::Program::compile(&bad).is_err());}
    let mut bad=serde_json::to_value(&sides.card[1]).unwrap();bad["effect"]["amount"]["left"]["value"]=json!(3);let bad:crate::ability_ir::Ability=serde_json::from_value(bad).unwrap();assert!(!contract::registration(&bad));assert!(crate::ability_ir::Program::compile(&bad).is_err());
}
#[test]
fn exhausted_live_count_scan_is_bounded_before_any_marker_route() {
    let deep=(0..256).fold(Value::Null,|v,_|json!([v]));let wide=Value::Array(vec![Value::Null;65_536]);
    for payload in [deep,wide] {assert!(contract::marker(&json!([raw_entry(),payload])).is_err());}
}
#[test]
fn exhausted_live_count_reads_current_controller_both_arenas_and_unit_inventory() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=native(backend);for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);let source=unit(&e,&mut s,owner,"ASH_035");let ground=unit(&e,&mut s,owner,"LOF_256");let space=unit(&e,&mut s,owner,"ASH_201");let enemy=unit(&e,&mut s,owner.other(),"LOF_256");s.card_mut(enemy).exhausted=true;
        let resource=add(&e,&mut s,owner,"LOF_254");s.card_mut(resource).exhausted=true;s.player_mut(owner).resources.push(resource).unwrap();
        let attached=add(&e,&mut s,owner,"LOF_254");s.card_mut(attached).exhausted=true;e.attach(&mut s,attached,source);
        let leader=s.player(owner).leader;s.card_mut(leader).exhausted=true;
        assert_eq!(count(&e,&s,source),0);s.card_mut(source).exhausted=true;assert_eq!(count(&e,&s,source),1);
        s.card_mut(space).exhausted=true;assert_eq!(count(&e,&s,source),2);s.card_mut(ground).exhausted=true;assert_eq!(count(&e,&s,source),3);
        let token=unit(&e,&mut s,owner,"TOKEN_MANDALORIAN");s.card_mut(token).exhausted=true;assert_eq!(count(&e,&s,source),4);
        s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();assert_eq!(count(&e,&s,source),5);
        s.active=owner.other();assert_eq!(count(&e,&s,source),5,"prompt actor is not controller");
        // Native control-boundary diagnostic: physical controller changes while
        // captured prompt actor remains the old owner. No engine witness claimed.
        s.player_mut(owner).ground.remove(source);s.card_mut(source).owner=owner.other();s.player_mut(owner.other()).ground.push(source).unwrap();s.active=owner;
        assert_eq!(count(&e,&s,source),2);s.card_mut(enemy).exhausted=false;assert_eq!(count(&e,&s,source),1);
    }}}
#[test]
fn exhausted_live_count_be_attacked_uses_live_threshold_and_sentinel_overrides_saboteur() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=native(backend);for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner.other();s.action_player=Some(owner.other());let source=unit(&e,&mut s,owner,"ASH_035");let helper=unit(&e,&mut s,owner,"ASH_201");let attacker=unit(&e,&mut s,owner.other(),"ASH_035");
        let can=|s:&State| {let target=choose(&e,s,attacker);e.legal_actions(&target).contains(&e.card_action(&target,source))};
        assert!(can(&s));s.card_mut(source).exhausted=true;assert!(can(&s));s.card_mut(helper).exhausted=true;assert!(!can(&s));
        s.card_mut(attacker).add_phase_keyword(2);assert!(!can(&s),"Saboteur cannot bypass naked beAttacked restriction");
        s.card_mut(source).add_phase_keyword(0);assert!(can(&s),"effective Sentinel overrides restriction even against Saboteur");
        s.card_mut(source).phase_keyword_presence=0;s.card_mut(helper).exhausted=false;assert!(can(&s));
    }}}
#[test]
fn exhausted_live_count_attack_exhausts_before_on_attack_and_factory_reads_again() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=native(backend);for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner;s.action_player=Some(owner);let source=unit(&e,&mut s,owner,"ASH_035");let helper=unit(&e,&mut s,owner,"ASH_201");let target=unit(&e,&mut s,owner.other(),"ASH_035");
        let victim=unit(&e,&mut s,owner,"ASH_035");let declaration=choose(&e,&s,source);assert_eq!(declaration.decision,Decision::AttackTarget);let mut prompt=choose(&e,&declaration,target);assert_eq!(prompt.decision,Decision::AbilityTarget);
        assert!(prompt.card(source).exhausted);assert_eq!(count(&e,&prompt,source),1);assert_eq!(prompt_read(&e,&prompt,source),"Choose a ground unit to deal 2 damage to");
        // Separate typed prompt/factory reads, not a legal external action during
        // the prompt and not a completed source-engine timing witness.
        prompt.card_mut(helper).exhausted=true;assert_eq!(prompt_read(&e,&prompt,source),"Choose a ground unit to deal 4 damage to");
        let done=choose(&e,&prompt,victim);assert_eq!(done.card(victim).damage,4,"factory rereads the count separately from prompt");assert!(done.player(owner.other()).discard.contains(target));
    }}}
#[test]
fn exhausted_live_count_prepared_amount_is_captured_once_and_source_can_die_before_combat() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=native(backend);for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner;s.action_player=Some(owner);let source=unit(&e,&mut s,owner,"ASH_035");let helper=unit(&e,&mut s,owner,"ASH_201");let target=unit(&e,&mut s,owner.other(),"ASH_035");s.card_mut(source).exhausted=true;s.card_mut(helper).exhausted=true;
        s.source=source;s.ability_target=target;s.ability_program=e.def(&s,source).programs.card.iter().copied().find(|p|e.program(*p).trigger=="onAttack").unwrap()+1;
        let effect=crate::ability_ir::Effect::Damage{target:crate::ability_ir::Target::name("chosen"),amount:contract::damage_expr()};
        e.capture_group_leaf(&mut s,0,&effect).unwrap();assert_eq!(s.event_group.records[0].amount,4);s.card_mut(source).exhausted=false;s.card_mut(helper).exhausted=false;assert_eq!(count(&e,&s,source),0);assert_eq!(s.event_group.records[0].amount,4);
        let group=crate::ability_ir::Effect::Simultaneous{effects:vec![effect]};e.resolve_group(&mut s,&group).unwrap();assert_eq!(s.card(target).damage,4);
        let mut lethal=board(&e);lethal.active=owner;lethal.action_player=Some(owner);let source=unit(&e,&mut lethal,owner,"ASH_035");let defender=unit(&e,&mut lethal,owner.other(),"LOF_256");lethal.card_mut(source).damage=5;
        let declaration=choose(&e,&lethal,source);let prompt=choose(&e,&declaration,defender);assert_eq!(prompt.decision,Decision::AbilityTarget);let done=choose(&e,&prompt,source);
        assert!(!e.in_arena(&done,source));assert!(e.in_arena(&done,defender));assert_eq!(done.card(defender).damage,0);assert_eq!(done.attack_source,0);
    }}}

#[test]
fn exhausted_live_count_captured_player_survives_controller_change_actor_and_nested_frame() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=native(backend);for captured in [Seat::P1,Seat::P2] {
        let current=captured.other();let mut s=board(&e);s.action_player=Some(captured);s.active=current;
        let source=unit(&e,&mut s,current,"ASH_035");s.card_mut(source).exhausted=true;
        let a=unit(&e,&mut s,captured,"LOF_256");let b=unit(&e,&mut s,captured,"ASH_201");s.card_mut(a).exhausted=true;s.card_mut(b).exhausted=true;
        let target=unit(&e,&mut s,current,"ASH_035");
        assert_eq!(count(&e,&s,source),1,"constant is refreshed to live controller");
        let p=e.def(&s,source).programs.card.iter().copied().find(|p|e.program(*p).trigger=="onAttack").unwrap();
        // Native captured-context diagnostic: the source was captured for the
        // other player before its controller changed. This is not a legal source
        // control-change game or a public pending-state import.
        s.death_pending.entries[0]=DeathTrigger{source,program:p,player:captured,..DeathTrigger::default()};s.death_pending.len=1;
        assert!(e.open_deaths(&mut s,DeathContinuation::Finish).unwrap());
        assert_eq!(s.active,captured);assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.death_depth,1);
        assert_eq!(prompt_read(&e,&s,source),"Choose a ground unit to deal 4 damage to");
        assert!(e.set_actor(&mut s,current).is_err(),"foreign actor cannot override this chooser");
        let parent=ResolutionContext::capture(&s);let mut foreign=s;foreign.active=current;foreign.source=target;
        parent.restore(&mut foreign);assert_eq!(foreign.active,captured);assert_eq!(foreign.source,source);
        assert_eq!(prompt_read(&e,&foreign,source),"Choose a ground unit to deal 4 damage to");
        let done=choose(&e,&foreign,target);assert_eq!(done.card(target).damage,4);assert_eq!(done.death_depth,0);
        assert_eq!(std::mem::size_of::<State>(),EXPECTED_INLINE_STATE_BYTES);
    }}
}
