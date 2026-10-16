use super::*;

fn attack(e:&Engine,s:State,source:CardId,target:CardId)->State {
    let a=e.card_action(&s,source);assert!(e.legal_actions(&s).contains(&a));
    let prompt=e.apply(&s,&a).unwrap();assert_eq!(prompt.decision,Decision::AttackTarget);
    e.apply(&prompt,&e.card_action(&prompt,target)).unwrap()
}
fn attach(e:&Engine,s:&mut State,owner:Seat,code:&str,host:CardId)->CardId {
    let id=add(e,s,owner,code);e.attach(s,id,host);id
}
#[test]
fn attached_alias_admission_census_and_preserved_rejections() {
    let e=engine();
    for code in ["ASH_134","ASH_210","JTL_073","SEC_039","TS26_052","TWI_169","TWI_218"] {
        assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));
    }
    for code in ["LOF_138","SHD_175","ASH_180","ASH_227","HMW_070","LAW_141","LAW_225","JTL_227","SHD_053","TS26_035","TWI_068","TWI_120","TWI_121"] {
        assert!(!e.by_code.contains_key(code),"unsupported attached child admitted {code}");
    }
}
#[test]
fn attached_alias_defense_uses_host_controller_and_exact_arena_before_combat() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;s.active=seat.other();s.action_player=Some(seat.other());
        let host=unit(&e,&mut s,seat,"LOF_254");let attacker=unit(&e,&mut s,seat.other(),"LOF_256");let space=unit(&e,&mut s,seat.other(),"TOKEN_XWING");
        let upgrade=attach(&e,&mut s,seat.other(),"ASH_210",host);
        let prompt=attack(&e,s,attacker,host);assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.source,host);assert_eq!(prompt.active,seat);
        assert_eq!(prompt.card(host).damage,0,"defense registration precedes combat");
        let actions=e.legal_actions(&prompt);assert!(actions.iter().any(|a|a.id()==Some(attacker)));assert!(!actions.iter().any(|a|a.id()==Some(space)));
        let done=e.apply(&prompt,&e.card_action(&prompt,attacker)).unwrap();assert!(done.card(attacker).exhausted);assert!(done.card(attacker).damage>=1);assert_eq!(done.card(upgrade).owner,seat.other());
    }}
}
#[test]
fn attached_alias_defeat_token_families_keep_captured_host_player_after_cleanup() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {for (code,token) in [("ASH_134","TOKEN_MANDALORIAN"),("TWI_169","TOKEN_CLONE_TROOPER"),("TWI_218","TOKEN_BATTLE_DROID")] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);s.round=5;
        let host=unit(&e,&mut s,seat,"LOF_254");let grantor=attach(&e,&mut s,seat.other(),code,host);
        e.defeat(&mut s,host).unwrap();assert_eq!(s.card(grantor).parent,0);assert!(s.player(seat.other()).discard.contains(grantor));assert!(s.player(seat).discard.contains(host));
        assert_eq!(s.death_pending.len,1);assert_eq!(s.death_pending.entries[0].source,host);assert_eq!(s.death_pending.entries[0].player,seat);assert_eq!(s.death_pending.entries[0].grantor,grantor);
        assert!(e.open_deaths(&mut s,DeathContinuation::Finish).unwrap());
        let created=e.arena_units(&s).into_iter().filter(|&id|s.card(id).owner==seat).filter(|&id|e.def(&s,id).code==token).collect::<Vec<_>>();assert_eq!(created.len(),1,"{code}");assert!(s.card(created[0]).exhausted);assert!(e.arena_units(&s).iter().all(|&id|s.card(id).owner==seat));
    }}}
}
#[test]
fn attached_alias_defeat_selector_remains_live_and_optional_after_parent_removal() {
    for code in ["JTL_073","SEC_039"] {for seat in [Seat::P1,Seat::P2] {
        let e=engine();let mut s=board(&e);s.round=5;let host=unit(&e,&mut s,seat,"LOF_254");
        let target=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(target).damage=e.hp(&s,target)-3;
        let high=unit(&e,&mut s,seat.other(),"LOF_256");
        let grantor=attach(&e,&mut s,seat.other(),code,host);e.defeat(&mut s,host).unwrap();assert!(e.open_deaths(&mut s,DeathContinuation::Finish).unwrap());
        assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.source,host);assert_eq!(s.active,seat);assert_eq!(s.card(grantor).parent,0);
        let legal=e.legal_actions(&s);assert!(legal.iter().any(|a|a.id()==Some(target)));if code=="SEC_039"{assert!(!legal.iter().any(|a|a.id()==Some(high)));}
        let declined=named_button(&e,&s,"Pass");assert!(e.in_arena(&declined,target));
        let done=e.apply(&s,&e.card_action(&s,target)).unwrap();if code=="JTL_073"{assert!(done.card(target).exhausted);}else{assert!(done.player(seat.other()).discard.contains(target));}
    }}
}
#[test]
fn attached_alias_sith_traditions_attack_and_death_are_separate_host_registrations() {
    for seat in [Seat::P1,Seat::P2] {
        let e=engine();let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");let ally=unit(&e,&mut s,seat,"LOF_254");let enemy=unit(&e,&mut s,seat.other(),"LOF_254");
        attach(&e,&mut s,seat.other(),"TS26_052",host);let base=s.player(seat.other()).base;
        let done=attack(&e,s,host,base);assert_eq!(e.upgrades(&done,host).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1);
        let mut dead=done;e.defeat(&mut dead,host).unwrap();assert_eq!(dead.death_pending.len,1);e.open_deaths(&mut dead,DeathContinuation::Finish).unwrap();assert_eq!(dead.decision,Decision::AbilityTarget);
        let actions=e.legal_actions(&dead);assert!(actions.iter().any(|a|a.id()==Some(ally)));assert!(!actions.iter().any(|a|a.id()==Some(enemy)));
        let done=e.apply(&dead,&e.card_action(&dead,ally)).unwrap();assert_eq!(e.upgrades(&done,ally).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1);
    }
}
#[test]
fn attached_alias_grantor_departure_and_host_reentry_do_not_restore_lost_grants() {
    let e=engine();let mut s=board(&e);let host=unit(&e,&mut s,Seat::P1,"LOF_254");let other=unit(&e,&mut s,Seat::P1,"LOF_254");
    let grantor=attach(&e,&mut s,Seat::P2,"TWI_218",host);
    e.leave_play_primary(&mut s,grantor).unwrap();e.defeat(&mut s,host).unwrap();assert_eq!(s.death_pending.len,0);
    s.player_mut(Seat::P1).discard.remove(host);e.add_arena(&mut s,host).unwrap();e.defeat(&mut s,host).unwrap();assert_eq!(s.death_pending.len,0);
    s.player_mut(Seat::P2).discard.remove(grantor);e.attach(&mut s,grantor,other);e.defeat(&mut s,other).unwrap();assert_eq!(s.death_pending.len,1);assert_eq!(s.death_pending.entries[0].source,other);
}

#[test]
fn attached_alias_both_players_share_one_declaration_window_and_host_prompt_order() {
    for first in ["You","Opponent"] {
        let e=engine();let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
        let attacker=unit(&e,&mut s,Seat::P1,"LOF_256");let defender=unit(&e,&mut s,Seat::P2,"LOF_254");
        attach(&e,&mut s,Seat::P2,"TS26_052",attacker);attach(&e,&mut s,Seat::P1,"ASH_210",defender);
        let prompt=attack(&e,s,attacker,defender);assert_eq!(prompt.decision,Decision::DeathPlayer);assert_eq!(prompt.active,Seat::P1);
        assert_eq!(prompt.card(attacker).damage,0);assert_eq!(prompt.card(defender).damage,0);
        assert_eq!(e.legal_actions(&prompt).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),vec!["You","Opponent"]);
        let defense=named_button(&e,&prompt,first);assert_eq!(defense.decision,Decision::AbilityTarget);assert_eq!(defense.active,Seat::P2);assert_eq!(defense.source,defender);
        let experience=e.upgrades(&defense,attacker).iter().filter(|&&id|e.def(&defense,id).code=="TOKEN_EXPERIENCE").count();assert_eq!(experience,usize::from(first=="You"));
        let done=e.apply(&defense,&e.card_action(&defense,attacker)).unwrap();assert_eq!(e.upgrades(&done,attacker).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1);
        assert_eq!(done.attack_source,0);assert_eq!(done.death_depth,0);
    }
}

#[test]
fn attached_alias_captured_defense_survives_grantor_removal_and_copy_hash() {
    let e=engine();let mut s=board(&e);let attacker=unit(&e,&mut s,Seat::P1,"LOF_256");let defender=unit(&e,&mut s,Seat::P2,"LOF_254");
    let grantor=attach(&e,&mut s,Seat::P1,"ASH_210",defender);
    let mut prompt=attack(&e,s,attacker,defender);assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.source,defender);
    let hash=prompt.zobrist();let copy=prompt;assert_eq!(copy.zobrist(),hash);assert_eq!(std::mem::size_of::<State>(),crate::model::EXPECTED_INLINE_STATE_BYTES);
    e.leave_play_primary(&mut prompt,grantor).unwrap();assert!(e.attached_host_grants(&prompt,defender,"onDefense").is_empty());assert_ne!(prompt.zobrist(),hash);
    let done=e.apply(&prompt,&e.card_action(&prompt,attacker)).unwrap();assert!(done.card(attacker).damage>=1);assert_eq!(done.card(grantor).parent,0);
}

#[test]
fn attached_alias_clone_raid_follows_only_current_host_and_not_upgrade_mention() {
    let e=engine();let mut s=board(&e);let host=unit(&e,&mut s,Seat::P1,"LOF_254");let other=unit(&e,&mut s,Seat::P2,"LOF_254");
    let grantor=attach(&e,&mut s,Seat::P2,"TWI_169",host);
    assert_eq!(e.keyword_amount(&s,host,"raid"),2);assert_eq!(e.keyword_amount(&s,grantor,"raid"),0);assert_eq!(e.keyword_amount(&s,other,"raid"),0);
    e.detach(&mut s,grantor);e.attach(&mut s,grantor,other);assert_eq!(e.keyword_amount(&s,host,"raid"),0);assert_eq!(e.keyword_amount(&s,other,"raid"),2);
}

#[test]
fn attached_alias_raw_metadata_hooks_fields_and_context_are_fail_closed() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="TWI_218").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("TWI_218");assert!(admitted(original.clone()));
    for key in ["stateWatchers","historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"] {
        let mut bad=original.clone();bad[key]=json!([{"kind":"unimplementedSourceContext"}]);assert!(!admitted(bad),"{key}");
    }
    for (path,value) in [
        ("/baseClass",json!("NonLeaderUnitCard")),("/card/types",json!(["upgrade","leader"])),("/card/unique",json!("false")),("/card/upgradeHp",json!(3)),("/keywordValues",json!({"raid":2})),("/card/keywords",json!(["smuggle"])),("/piloting",json!({})),
        ("/abilities/card/0/optional",json!(true)),("/abilities/card/0/effect/unknown",json!(true)),("/abilities/card/0/effect/requiresGrantorInPlay",json!(false)),("/abilities/card/0/effect/controllerBinding",json!("grantorController")),("/abilities/card/0/effect/sourceBinding",json!("grantor")),("/abilities/card/0/effect/registrationMode",json!("piloting")),
        ("/abilities/card/0/effect/registration/gainCondition",json!({"kind":"const","value":true})),("/abilities/card/0/effect/registration/limit",json!({"kind":"perRound","amount":1})),("/abilities/card/0/effect/registration/type",json!("action")),
        ("/abilities/card/0/effect/registration/program/hooks",json!([{"alias":"whenDefeated","enabled":false}])),("/abilities/card/0/effect/registration/program/hooks",json!([{"event":"onCardDefeated","predicate":{"kind":"const","value":true}}])),
        ("/abilities/card/0/effect/registration/program/trigger",json!("eventHooks")),("/abilities/card/0/effect/registration/program/attachedHost",json!(true)),("/abilities/card/0/effect/registration/program/condition",json!({"kind":"ref","path":["source","parentUnit"]})),("/abilities/card/0/effect/registration/program/effect/target",json!("parent")),
        ("/abilities/card/0/effect/registration/program/effect/tokenType",json!("credit")),("/abilities/card/0/effect/registration/program/then",json!({"title":"unsupported","effect":{"kind":"privateProgramCall","name":"arbitrary"}})),
    ] {
        let mut bad=original.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut target=&mut bad;
        for k in &parts[..parts.len()-1] {target=if target.is_array(){&mut target[k.parse::<usize>().unwrap()]}else{&mut target[*k]};}
        target[parts[parts.len()-1]]=value;assert!(!admitted(bad),"{path}");
    }
    let mut bad=original.clone();bad["abilities"]["card"][0]["attachedHost"]=json!(true);assert!(!admitted(bad));
    let mut nested=original.clone();let grant=nested["abilities"]["card"][0].clone();nested["abilities"]["card"][0]=json!({"trigger":"constant","title":"hidden wrapper","costs":[],"effect":{"kind":"sequence","effects":[grant["effect"].clone()]}});assert!(!admitted(nested));
}

#[test]
fn attached_alias_selected_pool_requires_each_created_token_definition() {
    let e=engine();let bytes=include_bytes!("../../../compiler/generated/catalog.json");
    for (code,token) in [("ASH_134","TOKEN_MANDALORIAN"),("TWI_169","TOKEN_CLONE_TROOPER"),("TWI_218","TOKEN_BATTLE_DROID"),("TS26_052","TOKEN_EXPERIENCE")] {
        let mut pool=e.compiled_pool().unwrap();pool["cards"].as_array_mut().unwrap().retain(|a|a["setCode"]==code||a["setCode"]==token||a["setCode"]=="TOKEN_SHIELD");
        let valid=Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),bytes);assert!(valid.is_ok(),"{code}: {:?}",valid.err());
        pool["cards"].as_array_mut().unwrap().retain(|a|a["setCode"]!=token);
        let error=Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),bytes).err().unwrap();assert!(error.contains(token),"{error}");if token!="TOKEN_EXPERIENCE" {assert!(error.contains(code),"{error}");}else{assert_eq!(error,"pool containing Experience creation must include reviewed TOKEN_EXPERIENCE");}
    }
}
#[test]
fn attached_alias_physical_duplicate_grants_and_capacity_are_bounded() {
    let e=engine();let mut s=board(&e);let host=unit(&e,&mut s,Seat::P1,"LOF_254");
    let mut grantors=vec![];for _ in 0..65 {grantors.push(attach(&e,&mut s,Seat::P2,"TWI_218",host));}
    let before=s;assert!(e.capture_death(&mut s,host).unwrap_err().contains("capacity"));assert_eq!(s,before);
    e.detach(&mut s,grantors[64]);e.capture_death(&mut s,host).unwrap();assert_eq!(s.death_pending.len,64);
    assert_eq!(s.death_pending.entries[..64].iter().map(|e|e.grantor).collect::<Vec<_>>(),grantors[..64]);assert!(s.death_pending.entries[..64].iter().all(|e|e.source==host&&e.player==Seat::P1));
}

#[test]
fn attached_alias_source_context_extras_are_not_silently_discarded() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="TWI_218").unwrap();
    for field in ["stateWatchers","historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"] {
        for extra in [json!([{"unsupported":"source context"}]),json!({"unknown":true}),json!(true)] {
            let mut bad=original.clone();bad[field]=extra;let c=crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[bad]})).unwrap();assert!(!c.cards.contains_key("TWI_218"),"{field}");
        }
    }
}
