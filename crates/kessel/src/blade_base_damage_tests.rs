//! Source guards and native event-window regression tests. Actual-engine/Wasm
//! differential acceptance remains a separate gate.
use super::*;
fn entry()->Value {
    let c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="ASH_204").unwrap().clone()
}
fn engine()->Engine {
    let e=Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    assert!(e.by_code.contains_key("ASH_204"),"complete source and native event pipeline must be admitted");e
}
fn add(e:&Engine,s:&mut State,owner:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;*s.card_mut(id)=Card{present:true,owner,def:e.by_code[code],..Default::default()};id}
fn board(e:&Engine)->State {
    let mut s=State::default();s.registration_order.exact=true;s.arena_order_exact=true;s.round=4;
    s.active=Seat::P1;s.action_player=Some(Seat::P1);s.hidden_history_exact=true;s.force_state_exact=true;s.credit_state_exact=e.needs_credit_state();
    for owner in [Seat::P1,Seat::P2] {let base=add(e,&mut s,owner,"ASH_019");s.player_mut(owner).base=base;let leader=add(e,&mut s,owner,"SOR_014");s.player_mut(owner).leader=leader;s.player_mut(owner).force_id=s.next_id;s.next_id+=1;}s
}
fn blade(e:&Engine,s:&mut State,owner:Seat)->CardId {let id=add(e,s,owner,"ASH_204");e.add_arena(s,id).unwrap();id}
fn advantage(e:&Engine,s:&State,id:CardId)->usize {e.upgrades(s,id).iter().filter(|id|e.def(s,**id).code=="TOKEN_ADVANTAGE").count()}
fn drain(e:&Engine,s:&mut State) {
    e.open_deaths(s,DeathContinuation::Finish).unwrap();
    for _ in 0..32 {if s.death_depth==0 && s.death_pending.len==0 {return;}let action=e.legal_actions(s).into_iter().next().expect("pending mandatory trigger must remain actionable");*s=e.apply(s,&action).unwrap();}
    panic!("mandatory window did not drain");
}
#[test]
fn blade_source_complete_semantic_contract_rejects_partial_profiles_and_hook_mutants() {
    let original=entry();assert!(crate::blade_base_damage_contract::entry(&original));
    let mut renamed=original.clone();renamed["setCode"]=json!("TEST_999");renamed["id"]=json!("foreign identity");renamed["card"]["id"]=json!("foreign identity");
    assert!(crate::blade_base_damage_contract::entry(&renamed),"identity is not behavior");
    for (key,value) in [("types",json!(["leader","unit"])),("power",json!(3)),("hp",json!(5)),("keywords",json!(["support"])),("traits",json!([])),("aspects",json!([]))] {let mut e=original.clone();e["card"][key]=value;assert!(!crate::blade_base_damage_contract::entry(&e),"{key}");}
    for (key,value) in [("baseClass",json!("TokenUnitCard")),("keywordValues",json!({"support":1})),("piloting",json!({})),("classMethods",json!([{}]))] {let mut e=original.clone();e[key]=value;assert!(!crate::blade_base_damage_contract::entry(&e),"{key}");}
    for (key,value) in [("optional",json!(true)),("requiresConfirmation",json!(true)),("condition",json!({"kind":"const","value":true})),("targetResolver",json!({"owner":"self","type":"unit","zone":"anyAttackable"}))] {let mut e=original.clone();e["abilities"]["card"][0][key]=value;assert!(!crate::blade_base_damage_contract::entry(&e));let a:crate::ability_ir::Ability=serde_json::from_value(e["abilities"]["card"][0].clone()).unwrap();assert!(crate::blade_base_damage_contract::compile_registration(&a).is_err());}
    for (key,value) in [("op",json!("==")),("right",json!({"kind":"ref","path":["opponent","base"]})),("left",json!({"kind":"ref","path":["source"]}))] {let mut e=original.clone();e["abilities"]["card"][0]["hooks"][0]["predicate"][key]=value;assert!(!crate::blade_base_damage_contract::entry(&e),"{key}");}
    let mut extra=original.clone();extra["abilities"]["card"].as_array_mut().unwrap().push(original["abilities"]["card"][0].clone());assert!(!crate::blade_base_damage_contract::entry(&extra));
    for (key,value) in [("target",json!("chosen")),("createdBy",json!("opponent")),("amount",json!({"kind":"const","value":2})),("extra",json!(true))] {let mut e=original.clone();e["abilities"]["card"][0]["effect"][key]=value;assert!(!crate::blade_base_damage_contract::entry(&e));}
    for id in ["callback:1:0","callback:129999:0"] {let mut e=original.clone();e["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"]=json!(id);assert!(crate::blade_base_damage_contract::entry(&e));}
    for id in ["callback:1:1","callback:x:0","callback:1:0:extra",""] {let mut e=original.clone();e["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"]=json!(id);assert!(!crate::blade_base_damage_contract::entry(&e));}
}
#[test]
fn blade_per_event_not_per_damage_and_never_other_targets_or_cancelled_events() {
    let e=engine();for owner in [Seat::P1,Seat::P2] {let mut s=board(&e);let source=blade(&e,&mut s,owner);let listeners=e.blade_base_damage_listeners(&s).unwrap();let base=s.player(owner).base;
        e.capture_blade_base_damage_event(&mut s,&listeners,base,false).unwrap();let enemy_base=s.player(owner.other()).base;
        e.capture_blade_base_damage_event(&mut s,&listeners,enemy_base,true).unwrap();e.capture_blade_base_damage_event(&mut s,&listeners,source,true).unwrap();assert_eq!(s.death_pending.len,0);
        s.card_mut(base).damage=3;e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();assert_eq!(s.death_pending.len,1);drain(&e,&mut s);assert_eq!(advantage(&e,&s,source),1);
        // A second distinct event is another token, even in the same parent
        // window. There is deliberately no amount argument or positive test.
        let listeners=e.blade_base_damage_listeners(&s).unwrap();e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();assert_eq!(s.death_pending.len,2);drain(&e,&mut s);assert_eq!(advantage(&e,&s,source),3);
    }
}
#[test]
fn blade_listener_lki_survives_sibling_departure_but_cannot_give_to_discard() {
    let e=engine();let mut s=board(&e);let source=blade(&e,&mut s,Seat::P1);let base=s.player(Seat::P1).base;
    let listeners=e.blade_base_damage_listeners(&s).unwrap();e.defeat(&mut s,source).unwrap();
    e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();assert_eq!(s.death_pending.len,1);
    let captured=s.death_pending.entries[0];assert_eq!((captured.source,captured.player,captured.last_known_power,captured.event_card.get()),(source,Seat::P1,2,base));
    let next=s.next_id;drain(&e,&mut s);assert_eq!(s.next_id,next);assert_eq!(advantage(&e,&s,source),0);
    assert!(e.blade_base_damage_listeners(&s).unwrap().is_empty());
}
#[test]
fn blade_snapshot_controller_base_not_live_mutation_and_capacity_preflight_is_atomic() {
    let e=engine();let mut s=board(&e);let source=blade(&e,&mut s,Seat::P1);let base=s.player(Seat::P1).base;let listeners=e.blade_base_damage_listeners(&s).unwrap();
    // Synthetic LKI mutation diagnostic only: foreign control is not admitted.
    s.card_mut(source).owner=Seat::P2;
    e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();assert_eq!(s.death_pending.entries[0].player,Seat::P1);
    s.death_pending.len=MAX_DEATH_TRIGGERS as u8;let before=s.death_pending;
    assert!(e.capture_blade_base_damage_event(&mut s,&listeners,base,true).is_err());assert_eq!(s.death_pending.entries,before.entries);assert_eq!(s.death_pending.len,before.len);
    let mut s=board(&e);blade(&e,&mut s,Seat::P1);s.registration_order.exact=false;assert!(e.blade_base_damage_listeners(&s).is_err());
}
#[test]
fn blade_terminal_mandatory_owner_choices_drain_through_existing_shared_window() {
    let e=engine();let mut s=board(&e);let p1=blade(&e,&mut s,Seat::P1);let p2=blade(&e,&mut s,Seat::P2);let listeners=e.blade_base_damage_listeners(&s).unwrap();
    let base2=s.player(Seat::P2).base;s.card_mut(base2).damage=1;e.capture_blade_base_damage_event(&mut s,&listeners,base2,true).unwrap();
    let base1=s.player(Seat::P1).base;s.card_mut(base1).damage=30;e.capture_blade_base_damage_event(&mut s,&listeners,base1,true).unwrap();e.check_win(&mut s);assert_eq!(s.winner(),Some(Seat::P2));
    drain(&e,&mut s);assert_eq!(s.winner(),Some(Seat::P2));assert_eq!(advantage(&e,&s,p1),1);assert_eq!(advantage(&e,&s,p2),1);assert!(e.legal_actions(&s).is_empty());
}
#[test]
fn blade_mixed_effective_owners_keep_the_shared_player_order_prompt() {
    let e=engine();let mut s=board(&e);let gone=blade(&e,&mut s,Seat::P1);let live=blade(&e,&mut s,Seat::P2);
    let listeners=e.blade_base_damage_listeners(&s).unwrap();
    for owner in [Seat::P1,Seat::P2] {let base=s.player(owner).base;e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();}
    e.defeat(&mut s,gone).unwrap();assert_eq!(s.death_pending.len,2);
    e.open_deaths(&mut s,DeathContinuation::Finish).unwrap();assert_eq!(s.decision,Decision::DeathPlayer);
    assert_eq!(e.legal_actions(&s),vec![Action::button("You",0),Action::button("Opponent",1)]);
    s=e.apply(&s,&Action::button("Opponent",1)).unwrap();
    assert_eq!(s.death_depth,0);assert_eq!(advantage(&e,&s,gone),0);assert_eq!(advantage(&e,&s,live),1);
}

fn scalar_damage(e:&Engine,s:&mut State,id:CardId,amount:u16) {
    let effect:Effect=serde_json::from_value(json!({"kind":"damage","target":"self","amount":{"kind":"const","value":amount}})).unwrap();
    e.apply_scalar_effect(s,&effect,vec![Eval::Card(id)],amount,None).unwrap();
}
#[test]
fn blade_native_scalar_events_both_seats_backends_and_source_identity() {
    let mut results=vec![];
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        let mut done=vec![];
        for owner in [Seat::P1,Seat::P2] {
            let mut s=board(&e);let source=blade(&e,&mut s,owner);let base=s.player(owner).base;
            scalar_damage(&e,&mut s,base,3);assert_eq!(s.death_pending.len,1);drain(&e,&mut s);
            assert_eq!(advantage(&e,&s,source),1);assert_eq!(e.power(&s,source,false),3);
            scalar_damage(&e,&mut s,base,0);let enemy=s.player(owner.other()).base;scalar_damage(&e,&mut s,enemy,2);assert_eq!(s.death_pending.len,0);
            let shield=add(&e,&mut s,owner,"TOKEN_SHIELD");e.attach(&mut s,shield,source);s.card_mut(source).shield=shield;scalar_damage(&e,&mut s,source,1);assert_eq!(s.death_pending.len,0);assert_eq!(s.card(source).damage,0);
            done.push(s);
        }
        results.push(done);
    }
    assert_eq!(results[0],results[1]);
}
#[test]
fn blade_native_combat_base_and_separate_overwhelm_event_before_framework_defeat() {
    let e=engine();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);let source=blade(&e,&mut s,owner);let attacker=add(&e,&mut s,owner.other(),"LOF_254");e.add_arena(&mut s,attacker).unwrap();
        s.active=owner.other();s.source=attacker;s.attack_source=attacker;s.target=s.player(owner).base;s.attack_defender=s.target;
        e.combat(&mut s).unwrap();assert_eq!(advantage(&e,&s,source),1,"combat must emit one event rather than tokens per point");
        let mut s=board(&e);let source=blade(&e,&mut s,owner);let attacker=add(&e,&mut s,owner.other(),"LOF_254");e.add_arena(&mut s,attacker).unwrap();
        let defender=add(&e,&mut s,owner,"LOF_256");e.add_arena(&mut s,defender).unwrap();
        s.card_mut(attacker).phase_power=10;s.card_mut(attacker).add_phase_keyword(1);
        s.active=owner.other();s.source=attacker;s.attack_source=attacker;s.target=defender;s.attack_defender=defender;
        e.combat(&mut s).unwrap();assert_eq!(advantage(&e,&s,source),1);assert!(!e.in_arena(&s,defender));assert!(s.card(s.player(owner).base).damage>0);
    }
}
#[test]
fn blade_native_draw_fatigue_is_one_event_per_draw_and_regroup_drains_before_resources() {
    let e=engine();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);let source=blade(&e,&mut s,owner);e.draw_cards(&mut s,owner,2).unwrap();assert_eq!(s.death_pending.len,1);assert_eq!(s.card(s.player(owner).base).damage,6);drain(&e,&mut s);assert_eq!(advantage(&e,&s,source),1);
    }
    let mut s=board(&e);let a=blade(&e,&mut s,Seat::P1);let b=blade(&e,&mut s,Seat::P2);s.regroup=true;s.initiative=Seat::P2;let base=s.player(Seat::P1).base;s.card_mut(base).damage=29;
    e.draw_for_regroup(&mut s).unwrap();assert_ne!(s.decision,Decision::Resource);assert!(s.death_depth>0);assert_eq!(s.death_frames[0].continuation(),Some(DeathContinuation::RegroupResources));
    for _ in 0..16 {if s.decision==Decision::Resource {break;}let action=e.legal_actions(&s).into_iter().next().unwrap();s=e.apply(&s,&action).unwrap();}
    assert_eq!(s.decision,Decision::Resource);assert_eq!(advantage(&e,&s,a),1);assert_eq!(advantage(&e,&s,b),1);assert_eq!(s.active,Seat::P1);assert_eq!(s.winner(),Some(Seat::P2));
}
#[test]
fn blade_actual_cinder_paid_play_captures_before_later_unit_death() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let mut e=engine();e.set_rules_backend(backend).unwrap();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner;s.action_player=Some(owner);let source=blade(&e,&mut s,owner);
        for _ in 0..12 {let r=add(&e,&mut s,owner,"LOF_254");s.player_mut(owner).resources.push(r).unwrap();}
        let cinder=add(&e,&mut s,owner,"ASH_151");s.player_mut(owner).hand.push(cinder).unwrap();let action=e.card_action(&s,cinder);assert!(e.legal_actions(&s).contains(&action));
        s=e.apply(&s,&action).unwrap();
        for _ in 0..24 {if s.decision==Decision::Window && s.death_depth==0 {break;}let actions=e.legal_actions(&s);assert!(!actions.is_empty(),"{:?}",s.decision);s=e.apply(&s,&actions[0]).unwrap();}
        assert!(!e.in_arena(&s,source));assert!(s.player(owner).discard.contains(source));assert_eq!(advantage(&e,&s,source),0);assert_eq!(s.card(s.player(owner).base).damage,5);assert_eq!(s.death_pending.len,0);
    }}
}
#[test]
fn blade_whole_entry_guards_precede_other_family_early_returns_and_no_id_dispatch() {
    fn load(v:Value)->crate::ability_ir::Catalogue {crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap()}
    let raw=entry();assert!(load(raw.clone()).cards.contains_key("ASH_204"));let mut renamed=raw.clone();renamed["setCode"]=json!("TEST_OWN_BASE");renamed["id"]=json!("foreign");renamed["internalName"]=json!("foreign");assert!(load(renamed).cards.contains_key("TEST_OWN_BASE"));
    for (pointer,value) in [("/card/power",json!(3)),("/abilities/card/0/effect/target",json!("chosen")),("/abilities/card/0/hooks/0/event",json!("onCardDefeated")),("/abilities/unit",json!([raw["abilities"]["card"][0].clone()]))] {let mut v=raw.clone();*v.pointer_mut(pointer).unwrap_or_else(||panic!("{pointer}"))=value;assert!(load(v).rejected.contains_key("ASH_204"),"{pointer}");}
    let mut extra=raw.clone();extra["abilities"]["card"][0]["effect"]=json!({"kind":"noAction"});assert!(load(extra).rejected.contains_key("ASH_204"));
}
#[test]
fn blade_prepared_events_snapshot_before_explicit_sibling_departure_and_preserve_multiplicity() {
    let e=engine();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner;let source=blade(&e,&mut s,owner);let event=add(&e,&mut s,owner,"ASH_151");s.source=event;s.ability_program=e.card_programs(&s,event)[0]+1;
        let effect:Effect=serde_json::from_value(json!({"kind":"simultaneous","effects":[{"kind":"defeat","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true}}},{"kind":"damage","target":"ownBase","amount":{"kind":"const","value":2}}]})).unwrap();
        assert!(e.generate_group(&mut s,&effect).unwrap());assert!(e.resolve_group(&mut s,&effect).unwrap());
        assert!(!e.in_arena(&s,source));assert_eq!(s.death_pending.len,1,"pre-primary listener must survive earlier explicit defeat");assert_eq!(s.death_pending.entries[0].source,source);
        s.event_group=EventGroupState::default();s.ability_pc=e.program(s.ability_program-1).event_end;drain(&e,&mut s);assert_eq!(advantage(&e,&s,source),0);
        let mut s=board(&e);s.active=owner;let source=blade(&e,&mut s,owner);let event=add(&e,&mut s,owner,"ASH_151");s.source=event;s.ability_program=e.card_programs(&s,event)[0]+1;
        let effect:Effect=serde_json::from_value(json!({"kind":"simultaneous","effects":[{"kind":"damage","target":"ownBase","amount":{"kind":"const","value":2}},{"kind":"damage","target":"ownBase","amount":{"kind":"const","value":3}}]})).unwrap();
        assert!(e.generate_group(&mut s,&effect).unwrap());assert!(e.resolve_group(&mut s,&effect).unwrap());assert_eq!(s.death_pending.len,2);assert_eq!(s.card(s.player(owner).base).damage,5);s.event_group=EventGroupState::default();s.ability_pc=e.program(s.ability_program-1).event_end;drain(&e,&mut s);assert_eq!(advantage(&e,&s,source),2);
    }
}
#[test]
fn blade_actual_chaos_query_zero_base_event_is_cancelled_and_positive_sibling_resolves() {
    // Actual H294 oracle: SHD159 is the only own-hand card before paid play;
    // opponent hand has one. Both-seat recordings observe base0/token0 vs
    // base1/token1. CardTargetSystem.checkEventCondition requires full resolution,
    // so DamageSystem rejects the zero event before handler/trigger cleanup.
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let mut e=engine();e.set_rules_backend(backend).unwrap();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.active=owner;s.action_player=Some(owner);let own=blade(&e,&mut s,owner);let enemy=blade(&e,&mut s,owner.other());
        for _ in 0..12 {let id=add(&e,&mut s,owner,"LOF_254");s.player_mut(owner).resources.push(id).unwrap();}
        let card=add(&e,&mut s,owner,"SHD_159");s.player_mut(owner).hand.push(card).unwrap();let other=add(&e,&mut s,owner.other(),"LOF_254");s.player_mut(owner.other()).hand.push(other).unwrap();
        let action=e.card_action(&s,card);assert!(e.legal_actions(&s).contains(&action));s=e.apply(&s,&action).unwrap();
        for _ in 0..16 {if s.decision==Decision::Window && s.death_depth==0 {break;}let action=e.legal_actions(&s).into_iter().next().unwrap();s=e.apply(&s,&action).unwrap();}
        assert_eq!((s.player(owner).hand.len,s.player(owner.other()).hand.len),(0,1));assert_eq!((s.card(s.player(owner).base).damage,s.card(s.player(owner.other()).base).damage),(0,1));assert_eq!((advantage(&e,&s,own),advantage(&e,&s,enemy)),(0,1));
    }}
}
#[test]
fn blade_first_strike_overwhelm_captures_once_before_typed_defeat_batch() {
    let e=engine();for owner in [Seat::P1,Seat::P2] {
        let mut s=board(&e);let source=blade(&e,&mut s,owner);let attacker=add(&e,&mut s,owner.other(),"LOF_254");e.add_arena(&mut s,attacker).unwrap();let defender=add(&e,&mut s,owner,"LOF_256");e.add_arena(&mut s,defender).unwrap();s.card_mut(attacker).phase_power=10;s.card_mut(attacker).add_phase_keyword(1);
        s.active=owner.other();s.source=attacker;s.attack_source=attacker;s.target=defender;s.attack_defender=defender;e.combat_first_window(&mut s).unwrap();
        assert!(!e.in_arena(&s,defender));assert_eq!(advantage(&e,&s,source),1);assert!(s.card(s.player(owner).base).damage>0);
    }
}

fn nested_raw(depth:usize)->Value {(0..depth).fold(Value::Null,|v,_|Value::Array(vec![v]))}
#[test]
fn blade_marker_budget_limits_programmatic_depth_width_and_aggregate_children() {
    let scan=crate::blade_base_damage_contract::marker;
    assert_eq!(scan(&nested_raw(128)).unwrap(),false);
    assert!(scan(&nested_raw(129)).unwrap_err().contains("inspection budget"));
    assert!(scan(&nested_raw(256)).is_err());
    assert!(!scan(&Value::Array(vec![Value::Null;65_535])).unwrap());
    for width in [65_536,65_537,1_000_000] {
        assert!(scan(&Value::Array(vec![Value::Null;width])).unwrap_err().contains("inspection budget"),"width{width}");
    }
    // Both child arrays fit individually. The cumulative scheduled-node budget
    // must still reject their combined children before appending the second set.
    assert!(!scan(&json!([vec![Value::Null;32_766],vec![Value::Null;32_767]])).unwrap());
    assert!(scan(&json!([vec![Value::Null;32_767],vec![Value::Null;32_767]])).is_err());
    let malformed=json!({"event":"onDamageDealt","predicate":{"unreviewed":true}});
    assert!(scan(&malformed).unwrap());
    // Finding a marker early must never conceal a deep or huge sibling.
    for payload in [nested_raw(256),Value::Array(vec![Value::Null;65_536])] {
        assert!(scan(&json!([malformed.clone(),payload])).is_err());
    }
}
#[test]
fn blade_raw_loader_rejects_budget_exhaustion_before_other_normalizers_and_routes() {
    let original=entry();
    for payload in [nested_raw(256),Value::Array(vec![Value::Null;65_537]),Value::Array(vec![Value::Null;1_000_000])] {
        let mut raw=original.clone();raw["inspectionPayload"]=payload;
        // Piloting would otherwise reject first with its lifecycle reason.
        raw["piloting"]=json!({"unimplemented":true});
        let reason=format!("{}: own-base damage source inspection budget exceeded",raw["internalName"].as_str().unwrap());
        let c=json!({"schema":"swu-card-ir/v1","cards":[raw.clone()]});
        let loaded=crate::ability_ir::Catalogue::load(&c).unwrap();
        assert_eq!(loaded.rejected["ASH_204"],reason);
        // The public Engine loader also requires its real native token roster.
        let mut full:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
        *full["cards"].as_array_mut().unwrap().iter_mut().find(|v|v["setCode"]=="ASH_204").unwrap()=raw;
        let e=Engine::from_catalogue_value(&full).unwrap();
        assert_eq!(e.rejected_cards().unwrap()["ASH_204"],reason);
    }
    let mut raw=original;raw["abilities"]["card"][0]["hooks"][0]["predicate"]=json!({"kind":"const","value":true});
    let loaded=crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[raw]})).unwrap();
    assert!(loaded.rejected["ASH_204"].contains("complete closed mandatory source Advantage"));
}
#[test]
fn blade_bounded_marker_preserves_all_real_catalogue_classifications_and_admissions() {
    let c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    let cards=c["cards"].as_array().unwrap();assert_eq!(cards.len(),2605);
    // Reference only on the finite real catalogue, never adversarial Values.
    fn original(v:&Value)->bool {v["event"]=="onDamageDealt" || v["text"]=="When your base is dealt damage: Give an Advantage token to this unit." || match v {Value::Array(a)=>a.iter().any(original),Value::Object(o)=>o.values().any(original),_=>false}}
    let mut markers=0;
    for raw in cards {let result=crate::blade_base_damage_contract::marker(raw).unwrap();assert_eq!(result,original(raw),"{}",raw["setCode"]);markers+=usize::from(result);}
    // The baseline is the actual unchanged input, not a mutable report from a
    // previous increment. A deliberately poisoned independent control must be
    // the sole admission change caused by the bounded whole-source marker.
    let e=Engine::from_catalogue_value(&c).unwrap();
    let codes=|e:&Engine|e.catalogue.as_ref().unwrap().cards.keys().filter(|code|!e.catalogue.as_ref().unwrap().reprint_of.contains_key(*code)).cloned().collect::<std::collections::BTreeSet<_>>();
    let baseline=codes(&e);assert!(baseline.contains("LOF_254"));
    let mut changed=c.clone();let raw=changed["cards"].as_array_mut().unwrap().iter_mut().find(|v|v["setCode"]=="LOF_254").unwrap();raw["card"]["text"]=json!("When your base is dealt damage: Give an Advantage token to this unit.");
    assert!(crate::blade_base_damage_contract::marker(raw).unwrap());let changed=Engine::from_catalogue_value(&changed).unwrap();let mut expected=baseline.clone();expected.remove("LOF_254");assert_eq!(codes(&changed),expected);
    assert!(changed.rejected_cards().unwrap()["LOF_254"].contains("complete closed mandatory source Advantage"));
    eprintln!("Blade bounded scan: all{} real entries compatible; markers{}; actual baseline{}; poisoned control sole loss; depth128 nodes65536",cards.len(),markers,baseline.len());
}
