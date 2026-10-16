use super::native_predicate_tests::{add,board,button,choose,engine,front,has,unit,weakness};
use super::*;

#[test]
fn native_heal_weakness_actual_bossk_heals_before_contingent_and_preserves_enemy_ownership() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e,seat,"HMW_015");let source=s.player(seat).leader;
            let enemy=unit(&e,&mut s,seat.other(),"ASH_096");let healthy=unit(&e,&mut s,seat.other(),"ASH_096");let own=unit(&e,&mut s,seat,"ASH_096");s.card_mut(enemy).damage=3;s.card_mut(own).damage=1;
            let first=front(&e,&s);assert_eq!(first.decision,Decision::AbilityTarget);assert!(!first.card(source).exhausted);
            assert!(e.legal_actions(&first).contains(&e.card_action(&first,enemy)));for id in [healthy,own,s.player(seat.other()).base] {assert!(!e.legal_actions(&first).contains(&e.card_action(&first,id)));}
            assert_eq!(button(&e,&first,"Cancel"),s);let token=first.next_id;let mut done=choose(&e,&first,enemy);
            assert!(done.card(source).exhausted);assert_eq!(done.card(enemy).damage,2);assert_eq!(e.hp(&done,enemy),3);assert!(e.in_arena(&done,enemy));assert_eq!(done.card(token).parent,enemy);assert_eq!(done.card(token).owner,seat.other());assert!(has(&e,&done,enemy,"weakness"));
            // The front condition is damage, not absence of an existing Weakness.
            done.active=seat;done.action_player=Some(seat);done.card_mut(source).exhausted=false;let second=front(&e,&done);let second=choose(&e,&second,enemy);assert_eq!(second.card(enemy).damage,1);assert_eq!(e.hp(&second,enemy),2);assert_eq!(e.upgrades(&second,enemy).len(),2);
            let snapshot=e.snapshot(&second);assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()),snapshot);
        }
    }
}

#[test]
fn native_heal_weakness_actual_bossk_attack_uses_any_physical_token_upgrade_and_can_pass() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {for token in ["experience","shield","weakness"] {
            let mut s=board(&e,seat,"HMW_015");let source=s.player(seat).leader;s.card_mut(source).deployed=true;e.add_arena(&mut s,source).unwrap();
            let target=unit(&e,&mut s,seat.other(),"ASH_096");let bare=unit(&e,&mut s,seat,"ASH_096");
            if token=="weakness" {weakness(&e,&mut s,target);}else {e.create_token(&mut s,target,token=="experience").unwrap();}
            let ordinary=e.definitions.iter().find(|d|d.info.is("upgrade")&&!d.info.is("token")).unwrap().code.clone();let upgrade=add(&e,&mut s,seat,&ordinary);e.attach(&mut s,upgrade,bare);
            let attack=choose(&e,&s,source);let prompt=choose(&e,&attack,s.player(seat.other()).base);assert_eq!(prompt.decision,Decision::AbilityTarget);
            assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,target)));assert!(!e.legal_actions(&prompt).contains(&e.card_action(&prompt,bare)));
            let passed=button(&e,&prompt,"Pass");assert_eq!(passed.card(target).damage,0);
            let done=choose(&e,&prompt,target);assert_eq!(done.card(target).damage,if token=="shield" {0}else{2});assert_eq!(has(&e,&done,target,"shield"),false);
        }}
    }
}

#[test]
fn native_heal_weakness_exact_group_and_complete_side_boundaries_reject() {
    let original:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let entry=original["cards"].as_array().unwrap().iter().find(|v|v["setCode"]=="HMW_015").unwrap();
    let reject=|entry:Value|{let c=crate::ability_ir::Catalogue::load(&json!({"schema":original["schema"],"cards":[entry]})).unwrap();assert!(c.rejected.contains_key("HMW_015"));};
    let mut bad=entry.clone();bad["abilities"]["unit"]=json!([]);reject(bad);
    let mut bad=entry.clone();bad["abilities"]["leader"][0]["effect"]["effects"].as_array_mut().unwrap().swap(0,1);reject(bad);
    for amount in [0,2] {let mut bad=entry.clone();bad["abilities"]["leader"][0]["effect"]["effects"][0]["amount"]["value"]=json!(amount);reject(bad);}
    let mut bad=entry.clone();bad["abilities"]["leader"][0]["targetResolver"]["owner"]=json!("any");reject(bad);
    let mut bad=entry.clone();bad["abilities"]["unit"][0]["targetResolver"]["predicate"]["right"]["args"][0]["name"]=json!("isUpgrade");reject(bad);
    let mut bad=entry.clone();bad["baseClass"]=json!("NonLeaderUnitCard");reject(bad);
}
