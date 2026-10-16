use super::*;
fn attached(e:&Engine,s:&mut State,owner:Seat,code:&str,parent:CardId)->CardId {let id=add(e,s,owner,code);e.attach(s,id,parent);id}
#[test]
fn attached_trait_complete_admission_keeps_unimplemented_companions_rejected() {
    let e=engine();for code in ["SHD_069","TS26_037","LAW_111","LAW_150"] {assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));}
    for code in ["ASH_135","SEC_054","ASH_183","ASH_230","LOF_052"] {assert!(!e.by_code.contains_key(code),"unimplemented companion admitted {code}");}
}
#[test]
fn attached_trait_foundling_live_physical_host_and_duplicate_removal_both_backends() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for owner in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;
        let host=unit(&e,&mut s,owner,"LOF_254");let other=unit(&e,&mut s,owner.other(),"LOF_254");assert!(!e.card_has_trait(&s,host,"mandalorian"));
        let a=attached(&e,&mut s,owner.other(),"SHD_069",host);let b=attached(&e,&mut s,owner,"SHD_069",host);
        assert!(e.card_has_trait(&s,host,"mandalorian"));assert!(e.card_traits(&s,host).contains("creature"));assert!(!e.card_has_trait(&s,other,"mandalorian"));
        e.leave_play_primary(&mut s,a).unwrap();assert!(e.card_has_trait(&s,host,"mandalorian"));e.detach(&mut s,b);e.attach(&mut s,b,other);assert!(!e.card_has_trait(&s,host,"mandalorian"));assert!(e.card_has_trait(&s,other,"mandalorian"));
        e.defeat(&mut s,other).unwrap();assert!(!e.card_has_trait(&s,other,"mandalorian"));assert_eq!(s.card(b).parent,0);
        s.player_mut(owner.other()).discard.remove(other);e.add_arena(&mut s,other).unwrap();assert!(!e.card_has_trait(&s,other,"mandalorian"));
    }}
}
#[test]
fn attached_trait_abandoned_order_live_loss_and_restore_use_host_controller() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for owner in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;let host=unit(&e,&mut s,owner,"LOF_048");
        assert!(e.card_has_trait(&s,host,"jedi"));let a=attached(&e,&mut s,owner.other(),"TS26_037",host);
        assert!(!e.card_has_trait(&s,host,"jedi"));assert!(e.card_has_trait(&s,host,"force"));assert_eq!(e.keyword_amount(&s,host,"restore"),1);assert_eq!(e.keyword_amount(&s,a,"restore"),0);
        let own=s.player(owner).base;let opposite=s.player(owner.other()).base;s.card_mut(own).damage=3;s.card_mut(opposite).damage=3;
        let power=e.power(&s,host,true);let mut s=declaration(&e,s,host,opposite);assert_eq!(s.card(own).damage,2);assert_eq!(s.card(opposite).damage,3+power);
        e.leave_play_primary(&mut s,a).unwrap();assert!(e.card_has_trait(&s,host,"jedi"));assert_eq!(e.keyword_amount(&s,host,"restore"),0);
    }}
}
#[test]
fn attached_trait_leia_disguise_play_uses_upgrade_player_and_current_parent_title() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for enemy_host in [false,true] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;
        let host=unit(&e,&mut s,if enemy_host{Seat::P2}else{Seat::P1},"ASH_059");let own=unit(&e,&mut s,Seat::P1,"LOF_254");let enemy=unit(&e,&mut s,Seat::P2,"LOF_254");let upgrade=hand(&e,&mut s,"LAW_111");
        let prompt=e.apply(&s,&e.card_action(&s,upgrade)).unwrap();assert_eq!(prompt.decision,Decision::UpgradeTarget);assert!(e.legal_actions(&prompt).iter().any(|a|a.id()==Some(host)));
        let prompt=e.apply(&prompt,&e.card_action(&prompt,host)).unwrap();assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.active,Seat::P1);assert_eq!(prompt.source,upgrade);assert!(e.card_has_trait(&prompt,host,"underworld"));
        let legal=e.legal_actions(&prompt);assert!(legal.iter().any(|a|a.id()==Some(own)));assert!(!legal.iter().any(|a|a.id()==Some(enemy)));if enemy_host {assert!(!legal.iter().any(|a|a.id()==Some(host)));}
        let done=e.apply(&prompt,&e.card_action(&prompt,own)).unwrap();assert_ne!(done.card(own).shield,0);assert_eq!(done.card(enemy).shield,0);
    }}
}
#[test]
fn attached_trait_leia_disguise_wrong_or_missing_parent_has_no_shield_and_no_dereference() {
    let e=engine();let mut s=board(&e);let host=unit(&e,&mut s,Seat::P1,"LOF_254");let upgrade=hand(&e,&mut s,"LAW_111");
    let prompt=e.apply(&s,&e.card_action(&s,upgrade)).unwrap();let done=e.apply(&prompt,&e.card_action(&prompt,host)).unwrap();assert_eq!(done.decision,Decision::Window);assert_eq!(done.card(host).shield,0);assert!(e.card_has_trait(&done,host,"underworld"));
    let mut missing=done;e.leave_play_primary(&mut missing,upgrade).unwrap();let program=e.def(&missing,upgrade).programs.card.iter().copied().find(|&p|e.program(p).trigger=="whenPlayed").unwrap();missing.source=upgrade;e.begin_program(&mut missing,program,0).unwrap();assert_eq!(missing.card(host).shield,0);assert!(!e.card_has_trait(&missing,host,"underworld"));
}
#[test]
fn attached_trait_fulcrum_grants_rebel_and_host_relative_aura_excluding_host() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for owner in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;
        let host=unit(&e,&mut s,owner,"LOF_254");let rebel=unit(&e,&mut s,owner,"SOR_095");let other=unit(&e,&mut s,owner,"LOF_254");let enemy=unit(&e,&mut s,owner.other(),"SOR_095");
        let before=(e.power(&s,rebel,false),e.hp(&s,rebel));let hp=e.hp(&s,host);let attack=e.power(&s,host,false);
        let a=attached(&e,&mut s,owner.other(),"LAW_150",host);
        assert!(e.card_has_trait(&s,host,"rebel"));assert_eq!((e.power(&s,rebel,false),e.hp(&s,rebel)),(before.0+2,before.1+2));assert_eq!(e.power(&s,enemy,false),before.0);assert_eq!(e.hp(&s,other),1);
        assert_eq!(e.power(&s,host,false),attack+e.def(&s,a).info.upgrade_power.unwrap() as u16);assert_eq!(e.hp(&s,host),hp+e.def(&s,a).info.upgrade_hp.unwrap() as u16);
        e.leave_play_primary(&mut s,a).unwrap();assert!(!e.card_has_trait(&s,host,"rebel"));assert_eq!((e.power(&s,rebel,false),e.hp(&s,rebel)),before);
    }}
}

#[test]
fn attached_trait_abandoned_order_play_optional_return_and_host_departure_cleanup() {
    let e=engine();let mut s=board(&e);s.round=5;let host=unit(&e,&mut s,Seat::P2,"LOF_048");let other=unit(&e,&mut s,Seat::P1,"LOF_254");let upgrade=hand(&e,&mut s,"TS26_037");
    let target=e.apply(&s,&e.card_action(&s,upgrade)).unwrap();let prompt=e.apply(&target,&e.card_action(&target,host)).unwrap();assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.source,upgrade);assert_eq!(prompt.active,Seat::P1);assert!(!e.card_has_trait(&prompt,host,"jedi"));assert_eq!(e.keyword_amount(&prompt,host,"restore"),1);
    let declined=named_button(&e,&prompt,"Pass");assert!(e.in_arena(&declined,host));assert_eq!(declined.card(upgrade).parent,host);
    let returned=e.apply(&prompt,&e.card_action(&prompt,other)).unwrap();assert!(returned.players[0].hand.contains(other));assert_eq!(returned.card(upgrade).parent,host);
    let own_host=e.apply(&prompt,&e.card_action(&prompt,host)).unwrap();assert!(own_host.players[1].hand.contains(host));assert!(own_host.players[0].discard.contains(upgrade));assert_eq!(own_host.card(upgrade).parent,0);assert!(e.card_has_trait(&own_host,host,"jedi"));assert_eq!(e.keyword_amount(&own_host,host,"restore"),0);
}

#[test]
fn attached_trait_fulcrum_live_gained_rebels_and_stat_death_after_grantor_removal() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
    let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);let a=unit(&e,&mut s,Seat::P1,"LOF_254");let b=unit(&e,&mut s,Seat::P1,"LOF_254");let rebel=unit(&e,&mut s,Seat::P1,"SOR_095");
    let first=attached(&e,&mut s,Seat::P2,"LAW_150",a);let base_hp=e.hp(&s,b);let p=e.power(&s,a,false);let second=attached(&e,&mut s,Seat::P1,"LAW_150",b);
    assert!(e.card_has_trait(&s,b,"rebel"));assert_eq!(e.power(&s,a,false),p+2);assert_eq!(e.hp(&s,b),base_hp+e.def(&s,second).info.upgrade_hp.unwrap() as u16+2);
    let expected=e.hp(&s,rebel);s.card_mut(rebel).damage=expected-1;assert!(s.card(rebel).damage<e.hp(&s,rebel));
    e.leave_play_primary(&mut s,first).unwrap();e.sweep_stat_defeats(&mut s).unwrap();assert!(s.players[0].discard.contains(rebel));assert!(!e.card_has_trait(&s,a,"rebel"));assert!(e.card_has_trait(&s,b,"rebel"));
    }
}

#[test]
fn attached_trait_raw_wrapper_binding_modifier_and_host_aura_fields_fail_closed() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["SHD_069","LAW_150"] {
        let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap();
        let admitted=|v:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]})).unwrap().cards.contains_key(code);assert!(admitted(original.clone()));
        for (path,value) in [("/baseClass",json!("NonLeaderUnitCard")),("/card/types",json!(["unit"])),("/card/upgradePower",json!(99)),("/stateWatchers",json!([{"unimplemented":true}])),("/privatePrograms",json!({"unknown":true})),("/abilities/card/0/optional",json!(true)),("/abilities/card/0/effect/targetController",json!("self")),("/abilities/card/0/effect/sourceBinding",json!("attachedHost")),("/abilities/card/0/effect/requiresGrantorInPlay",json!(false)),("/abilities/card/0/effect/gainAbilitySourceBinding",json!("host")),("/abilities/card/0/effect/condition",json!({"kind":"const","value":true})),("/abilities/card/0/effect/modifier/valueMode",json!("dynamic")),("/abilities/card/0/effect/modifier/scope",json!("player")),("/abilities/card/0/effect/modifier/target",json!("self")),("/abilities/card/0/effect/modifier/unknown",json!(true))] {
            let mut bad=original.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut node=&mut bad;for k in &parts[..parts.len()-1] {node=if node.is_array(){&mut node[k.parse::<usize>().unwrap()]}else{&mut node[*k]};}node[parts[parts.len()-1]]=value;assert!(!admitted(bad),"{code} {path}");
        }
        if code=="LAW_150" {
            let normalized=crate::attached_constants::ability(&original["abilities"]["card"][1]).unwrap();
            let mut naked=original.clone();naked["abilities"]["card"]=json!([serde_json::to_value(normalized).unwrap()]);assert!(!admitted(naked),"naked host stat aura must not widen old attachedContext guards");
            let mut bad=original.clone();bad["abilities"]["card"][1]["effect"]["sourceBinding"]=json!("upgrade");assert!(!admitted(bad));
            let mut bad=original.clone();bad["abilities"]["card"][1]["effect"]["program"]["effect"]["properties"]["matchTarget"]["parameters"][1]["root"]=json!("attackBinding1_0");assert!(!admitted(bad));
            let mut bad=original.clone();bad["abilities"]["card"][1]["effect"]["program"]["effect"]["properties"]["matchTarget"]["body"]["left"]["right"]["op"]=json!("!==");assert!(!admitted(bad));
        }
    }
}
