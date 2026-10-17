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
    for code in ["ASH_134","ASH_210","JTL_073","SEC_039","TS26_052","TWI_169","TWI_218","IC27_085","SHD_104","JTL_035","JTL_046","JTL_036","JTL_057","JTL_084","JTL_086","JTL_141","JTL_148","JTL_189","JTL_210","SHD_175"] {
        assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));
    }
    for code in ["LOF_138","ASH_180","ASH_227","HMW_070","LAW_141","LAW_225","JTL_227","SHD_053","TS26_035","TWI_068","TWI_120","TWI_121","JTL_066","JTL_142"] {
        assert!(!e.by_code.contains_key(code),"unsupported attached child admitted {code}");
    }
}
#[test]
fn attached_alias_inspiring_mentor_expands_dual_hooks_like_separate_registrations() {
    for code in ["IC27_085","SHD_104"] {for seat in [Seat::P1,Seat::P2] {
        let e=engine();let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");let ally=unit(&e,&mut s,seat,"LOF_254");let enemy=unit(&e,&mut s,seat.other(),"LOF_254");
        attach(&e,&mut s,seat.other(),code,host);let base=s.player(seat.other()).base;
        let prompt=attack(&e,s,host,base);assert_eq!(prompt.decision,Decision::AbilityTarget,"{code} onAttack prompt");
        assert_eq!(prompt.source,host);assert_eq!(prompt.active,seat);
        let attack_actions=e.legal_actions(&prompt);assert!(attack_actions.iter().any(|a|a.id()==Some(ally)));assert!(!attack_actions.iter().any(|a|a.id()==Some(host)||a.id()==Some(enemy)));
        let done=e.apply(&prompt,&e.card_action(&prompt,ally)).unwrap();assert_eq!(e.upgrades(&done,ally).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1,"{code} onAttack");
        let mut dead=done;e.defeat(&mut dead,host).unwrap();assert_eq!(dead.death_pending.len,1);e.open_deaths(&mut dead,3).unwrap();assert_eq!(dead.decision,Decision::AbilityTarget);
        let actions=e.legal_actions(&dead);assert!(actions.iter().any(|a|a.id()==Some(ally)));assert!(!actions.iter().any(|a|a.id()==Some(enemy)));
        let done=e.apply(&dead,&e.card_action(&dead,ally)).unwrap();assert_eq!(e.upgrades(&done,ally).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),2,"{code} whenDefeated stacks");
    }}
}
#[test]
fn attached_alias_dual_hooks_require_defeat_text_override_and_reject_partial_shapes() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="SHD_104").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("SHD_104");
    assert!(admitted(original.clone()));
    let mut no_override=original.clone();no_override["setupValidationOverrides"]=json!([]);assert!(!admitted(no_override));
    let mut bad_override=original.clone();bad_override["setupValidationOverrides"][0]["gameplayEffect"]=json!("changesRules");assert!(!admitted(bad_override));
    let mut single=original.clone();single["abilities"]["card"][0]["effect"]["registration"]["program"]["trigger"]=json!("onAttack");
    single["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"]=json!([{"alias":"onAttack","enabled":true}]);
    assert!(!admitted(single),"override without dual hooks");
    let mut reversed=original.clone();reversed["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"]=json!([{"alias":"whenDefeated","enabled":true},{"alias":"onAttack","enabled":true}]);
    assert!(!admitted(reversed),"hook order is part of the reviewed contract");
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
        assert!(e.open_deaths(&mut s,3).unwrap());
        let created=e.arena_units(&s).into_iter().filter(|&id|s.card(id).owner==seat).filter(|&id|e.def(&s,id).code==token).collect::<Vec<_>>();assert_eq!(created.len(),1,"{code}");assert!(s.card(created[0]).exhausted);assert!(e.arena_units(&s).iter().all(|&id|s.card(id).owner==seat));
    }}}
}
#[test]
fn attached_alias_defeat_selector_remains_live_and_optional_after_parent_removal() {
    for code in ["JTL_073","SEC_039"] {for seat in [Seat::P1,Seat::P2] {
        let e=engine();let mut s=board(&e);s.round=5;let host=unit(&e,&mut s,seat,"LOF_254");
        let target=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(target).damage=e.hp(&s,target)-3;
        let high=unit(&e,&mut s,seat.other(),"LOF_256");
        let grantor=attach(&e,&mut s,seat.other(),code,host);e.defeat(&mut s,host).unwrap();assert!(e.open_deaths(&mut s,3).unwrap());
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
        let mut dead=done;e.defeat(&mut dead,host).unwrap();assert_eq!(dead.death_pending.len,1);e.open_deaths(&mut dead,3).unwrap();assert_eq!(dead.decision,Decision::AbilityTarget);
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
        // Bare eventHooks without a reviewed alias rewrite stays rejected; matching
        // single-alias eventHooks (Paige) is normalized separately.
        ("/abilities/card/0/effect/registration/program/hooks",json!([])),("/abilities/card/0/effect/registration/program/trigger",json!("whenPlayed")),("/abilities/card/0/effect/registration/program/attachedHost",json!(true)),("/abilities/card/0/effect/registration/program/condition",json!({"kind":"ref","path":["source","parentUnit"]})),("/abilities/card/0/effect/registration/program/effect/target",json!("parent")),
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
    // SHD_104 is a reprint alias of IC27_085 and is excluded from compiled_pool.
    for (code,token) in [("ASH_134","TOKEN_MANDALORIAN"),("TWI_169","TOKEN_CLONE_TROOPER"),("TWI_218","TOKEN_BATTLE_DROID"),("TS26_052","TOKEN_EXPERIENCE"),("IC27_085","TOKEN_EXPERIENCE")] {
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

#[test]
fn piloting_host_paige_on_attack_experience_then_self_damage() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_046"),"{:?}",e.rejected_cards().unwrap().get("JTL_046"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        // Blank host: avoid stacking a unit onAttack beside the piloting grant.
        let host=unit(&e,&mut s,seat,"LOF_254");
        let printed_hp=e.hp(&s,host);
        attach(&e,&mut s,seat,"JTL_046",host);
        let base=s.player(seat.other()).base;
        let done=attack(&e,s,host,base);
        assert_eq!(e.upgrades(&done,host).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1,"{seat:?} experience");
        assert_eq!(done.card(host).damage,1,"{seat:?} self damage after experience");
        assert!(e.hp(&done,host)>=printed_hp,"{seat:?} experience raises hp before damage");
    }
}

#[test]
fn piloting_host_paige_as_unit_does_not_fire_attached_grant() {
    let e=engine();
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_046");
    let base=s.player(Seat::P2).base;
    let done=attack(&e,s,pilot,base);
    assert_eq!(e.upgrades(&done,pilot).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),0);
    assert_eq!(done.card(pilot).damage,0);
}

#[test]
fn piloting_host_tam_debuffs_enemy_in_same_arena() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_035"),"{:?}",e.rejected_cards().unwrap().get("JTL_035"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
        let space=unit(&e,&mut s,seat.other(),"TOKEN_XWING");
        let printed=e.power(&s,enemy,false);
        let printed_hp=e.hp(&s,enemy);
        attach(&e,&mut s,seat,"JTL_035",host);
        let base=s.player(seat.other()).base;
        let prompt=attack(&e,s,host,base);
        assert_eq!(prompt.decision,Decision::AbilityTarget,"{seat:?}");
        assert_eq!(prompt.source,host);
        let actions=e.legal_actions(&prompt);
        assert!(actions.iter().any(|a|a.id()==Some(enemy)));
        assert!(!actions.iter().any(|a|a.id()==Some(space)||a.id()==Some(host)));
        let done=e.apply(&prompt,&e.card_action(&prompt,enemy)).unwrap();
        assert_eq!(e.power(&done,enemy,false),printed.saturating_sub(1));
        assert_eq!(e.hp(&done,enemy),printed_hp.saturating_sub(1));
    }
}

#[test]
fn piloting_host_rejects_leaders_and_extra_keywords() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_046").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_046");
    assert!(admitted(original.clone()));
    for (path,value) in [
        ("/baseClass",json!("LeaderUnitCard")),
        ("/card/pilotText",json!(null)),
        ("/card/keywords",json!(["piloting","grit"])),
        ("/abilities/card/0/effect/registrationMode",json!("upgrade")),
        ("/abilities/card/0/effect/requiresAttachable",json!(false)),
        ("/abilities/card/0/effect/registration/program/hooks",json!([{"alias":"whenPlayed","enabled":true}])),
        ("/abilities/unit",json!([{"trigger":"onAttack","title":"extra","costs":[],"effect":{"kind":"noAction"}}])),
    ] {
        let mut bad=original.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut target=&mut bad;
        for k in &parts[..parts.len()-1] {target=if target.is_array(){&mut target[k.parse::<usize>().unwrap()]}else{&mut target[*k]};}
        target[parts[parts.len()-1]]=value;assert!(!admitted(bad),"{path}");
    }
}

fn fire_attached_when_played(e:&Engine,s:&mut State,pilot:CardId) {
    s.source=pilot;s.trigger_context=1;s.decision=Decision::Window;
    e.queue_programs(s,&["whenPlayed"]);
    e.process_all_triggers(s).unwrap();
}

#[test]
fn piloting_host_ig88_enemy_damaged_buffs_unit_and_attached_host() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_141"),"{:?}",e.rejected_cards().unwrap().get("JTL_141"));
    for seat in [Seat::P1,Seat::P2] {
        // As a unit: +3/+0 while an enemy unit is damaged.
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let ig=unit(&e,&mut s,seat,"JTL_141");
        let printed=e.power(&s,ig,false);
        assert_eq!(e.power(&s,ig,false),printed,"{seat:?} no enemy damage yet");
        let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
        s.card_mut(enemy).damage=1;
        assert_eq!(e.power(&s,ig,false),printed+3,"{seat:?} unit buff");

        // While attached: host gains the aura; the pilot does not stay a unit.
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let host_printed=e.power(&s,host,false);
        let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
        attach(&e,&mut s,seat,"JTL_141",host);
        assert_eq!(e.power(&s,host,false),host_printed,"{seat:?} attached without damage");
        s.card_mut(enemy).damage=1;
        assert_eq!(e.power(&s,host,false),host_printed+3,"{seat:?} host buff while attached");
    }
}

#[test]
fn piloting_host_ig88_rejects_mismatched_companion_and_hooks() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_141").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_141");
    assert!(admitted(original.clone()));
    // Host-only (no companion) still admits via constant parent grant.
    let mut host_only=original.clone();
    host_only["abilities"]["card"]=json!([original["abilities"]["card"][1].clone()]);
    assert!(admitted(host_only),"host-only constant grant");
    // Turn the constant grant into an onAttack hook: unit companion is no longer a reviewed mirror.
    let mut hooked=original.clone();
    hooked["abilities"]["card"][1]["effect"]["registration"]["type"]=json!("triggered");
    hooked["abilities"]["card"][1]["effect"]["registration"]["program"]["trigger"]=json!("eventHooks");
    hooked["abilities"]["card"][1]["effect"]["registration"]["program"]["hooks"]=json!([{"alias":"onAttack","enabled":true}]);
    assert!(!admitted(hooked),"onAttack host + companion");
    // Wrong buff amount on the host program.
    let mut wrong=original.clone();
    wrong["abilities"]["card"][1]["effect"]["registration"]["program"]["effect"]["power"]["value"]=json!(2);
    assert!(!admitted(wrong),"mismatched host amount");
    // Companion amount drift while host stays reviewed.
    let mut companion_wrong=original.clone();
    companion_wrong["abilities"]["card"][0]["effect"]["power"]["value"]=json!(1);
    assert!(!admitted(companion_wrong),"mismatched companion amount");
}

#[test]
fn piloting_self_astromech_heals_when_attached_not_as_unit() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_057"),"{:?}",e.rejected_cards().unwrap().get("JTL_057"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let damaged=unit(&e,&mut s,seat,"LOF_256");s.card_mut(damaged).damage=3;
        let pilot=attach(&e,&mut s,seat,"JTL_057",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?}");
        assert_eq!(s.source,pilot);
        assert!(e.legal_actions(&s).iter().any(|a|a.id()==Some(damaged)));
        let done=e.apply(&s,&e.card_action(&s,damaged)).unwrap();
        assert_eq!(done.card(damaged).damage,1,"{seat:?}");
    }
    // Played as a unit: WhenPlayed must not fire.
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_057");
    let damaged=unit(&e,&mut s,Seat::P1,"LOF_256");s.card_mut(damaged).damage=3;
    fire_attached_when_played(&e,&mut s,pilot);
    assert_eq!(s.decision,Decision::Window);
    assert_eq!(s.card(damaged).damage,3);
}

#[test]
fn piloting_self_victor_two_creates_tie_when_attached() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_084"),"{:?}",e.rejected_cards().unwrap().get("JTL_084"));
    assert!(e.by_code.contains_key("TOKEN_TIE_FIGHTER"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"TOKEN_XWING");
        let before=e.arena_units(&s).len();
        let pilot=attach(&e,&mut s,seat,"JTL_084",host);
        fire_attached_when_played(&e,&mut s,pilot);
        let ties:Vec<_>=e.arena_units(&s).into_iter().filter(|&id|e.def(&s,id).code=="TOKEN_TIE_FIGHTER"&&s.card(id).owner==seat).collect();
        assert_eq!(ties.len(),1,"{seat:?}");
        assert!(s.card(ties[0]).exhausted);
        assert_eq!(e.arena_units(&s).len(),before+1);
    }
}

#[test]
fn piloting_self_backstabber_experience_excludes_host() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_086"),"{:?}",e.rejected_cards().unwrap().get("JTL_086"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let ally=unit(&e,&mut s,seat,"LOF_256");
        let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
        let pilot=attach(&e,&mut s,seat,"JTL_086",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?}");
        let actions=e.legal_actions(&s);
        assert!(actions.iter().any(|a|a.id()==Some(ally)));
        assert!(actions.iter().any(|a|a.id()==Some(enemy)));
        assert!(!actions.iter().any(|a|a.id()==Some(host)),"host excluded via parentCard");
        let done=e.apply(&s,&e.card_action(&s,ally)).unwrap();
        assert_eq!(e.upgrades(&done,ally).iter().filter(|&&id|e.def(&done,id).code=="TOKEN_EXPERIENCE").count(),1);
    }
}

#[test]
fn piloting_self_rejects_non_when_played_and_companions() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_057").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_057");
    assert!(admitted(original.clone()));
    for (path,value) in [
        ("/abilities/card/0/effect/registration/program/hooks",json!([{"alias":"onAttack","enabled":true}])),
        ("/abilities/card/0/effect/kind",json!("attachedAbilityRegistration")),
        ("/abilities/card/0/effect/target",json!("parent")),
        ("/abilities/card/0/effect/sourceBinding",json!("attachedHost")),
        ("/card/keywords",json!(["piloting","ambush"])),
        ("/abilities/card",json!([
            original["abilities"]["card"][0].clone(),
            {"trigger":"whenPlayed","title":"extra","costs":[],"effect":{"kind":"noAction"}}
        ])),
    ] {
        let mut bad=original.clone();
        if path=="/abilities/card" {
            bad["abilities"]["card"]=value;
        } else {
            let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut target=&mut bad;
            for k in &parts[..parts.len()-1] {target=if target.is_array(){&mut target[k.parse::<usize>().unwrap()]}else{&mut target[*k]};}
            target[parts[parts.len()-1]]=value;
        }
        assert!(!admitted(bad),"{path}");
    }
}

#[test]
fn piloting_self_iden_gives_host_shield_when_attached() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_036"),"{:?}",e.rejected_cards().unwrap().get("JTL_036"));
    assert!(e.by_code.contains_key("TOKEN_SHIELD"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        assert_eq!(e.upgrades(&s,host).iter().filter(|&&id|e.def(&s,id).code=="TOKEN_SHIELD").count(),0);
        let pilot=attach(&e,&mut s,seat,"JTL_036",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::Window,"{seat:?} auto-resolves");
        assert_eq!(e.upgrades(&s,host).iter().filter(|&&id|e.def(&s,id).code=="TOKEN_SHIELD").count(),1,"{seat:?} host shield");
        assert!(e.upgrades(&s,host).contains(&pilot),"{seat:?} pilot still attached");
    }
    // Played as a unit: attach watcher must not fire (Shielded is intrinsic separately).
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_036");
    let before=e.upgrades(&s,pilot).len();
    fire_attached_when_played(&e,&mut s,pilot);
    assert_eq!(s.decision,Decision::Window);
    assert_eq!(e.upgrades(&s,pilot).len(),before,"unit play does not run attach Shield");
}

#[test]
fn piloting_self_iden_rejects_partial_attach_and_shield_shapes() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_036").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_036");
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Watcher must be self-attach, not an unrelated upgrade attach.
    let mut other_attach=original.clone();
    other_attach["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"]=json!([{
        "event":"onUpgradeAttached",
        "predicate":{"kind":"binary","op":"===",
            "left":{"kind":"ref","path":["event","parentCard"]},
            "right":{"kind":"ref","path":["source","parentCard"]}}
    }]);
    assert!(!admitted(other_attach),"non-self attach predicate");
    // Amount drift.
    let mut amount=original.clone();
    amount["abilities"]["card"][0]["effect"]["registration"]["program"]["effect"]["amount"]["value"]=json!(2);
    assert!(!admitted(amount),"shield amount");
    // Named parent grant belongs on attachedAbilityRegistration, not Expression rewrite.
    let mut named_parent=original.clone();
    named_parent["abilities"]["card"][0]["effect"]["registration"]["program"]["effect"]["target"]=json!("parent");
    assert!(!admitted(named_parent),"named parent target");
    // Optional would change the attach contract.
    let mut optional=original.clone();
    optional["abilities"]["card"][0]["effect"]["registration"]["program"]["optional"]=json!(true);
    assert!(!admitted(optional),"optional");
}

#[test]
fn piloting_self_boba_damages_by_host_transport_trait() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_189"),"{:?}",e.rejected_cards().unwrap().get("JTL_189"));
    for seat in [Seat::P1,Seat::P2] {
        // Non-Transport host: 1 damage.
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"TOKEN_XWING");
        let victim=unit(&e,&mut s,seat.other(),"LOF_256");
        let pilot=attach(&e,&mut s,seat,"JTL_189",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?} non-transport");
        let done=e.apply(&s,&e.card_action(&s,victim)).unwrap();
        assert_eq!(done.card(victim).damage,1,"{seat:?} non-transport");

        // Transport host: 2 damage.
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"ASH_254");
        let victim=unit(&e,&mut s,seat.other(),"LOF_256");
        let pilot=attach(&e,&mut s,seat,"JTL_189",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?} transport");
        let done=e.apply(&s,&e.card_action(&s,victim)).unwrap();
        assert_eq!(done.card(victim).damage,2,"{seat:?} transport");
    }
    // Played as a unit: WhenPlayed must not fire.
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_189");
    let victim=unit(&e,&mut s,Seat::P2,"LOF_256");
    fire_attached_when_played(&e,&mut s,pilot);
    assert_eq!(s.decision,Decision::Window);
    assert_eq!(s.card(victim).damage,0);
}

#[test]
fn piloting_self_mandalorian_exhausts_enemy_in_host_arena_when_attached() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_210"),"{:?}",e.rejected_cards().unwrap().get("JTL_210"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        // Ground host: only ground enemies in the same arena are legal.
        let host=unit(&e,&mut s,seat,"LOF_254");
        let enemy_ground=unit(&e,&mut s,seat.other(),"LOF_256");
        let enemy_space=unit(&e,&mut s,seat.other(),"TOKEN_XWING");
        let ally=unit(&e,&mut s,seat,"LOF_256");
        let pilot=attach(&e,&mut s,seat,"JTL_210",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?} attached");
        assert_eq!(s.source,pilot);
        let actions=e.legal_actions(&s);
        assert!(actions.iter().any(|a|a.id()==Some(enemy_ground)),"{seat:?} ground enemy");
        assert!(!actions.iter().any(|a|a.id()==Some(enemy_space)||a.id()==Some(ally)||a.id()==Some(host)),"{seat:?} arena/owner filter");
        let done=e.apply(&s,&e.card_action(&s,enemy_ground)).unwrap();
        assert!(done.card(enemy_ground).exhausted,"{seat:?} exhausted");

        // Space host: ground enemy is excluded.
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"TOKEN_XWING");
        let enemy_ground=unit(&e,&mut s,seat.other(),"LOF_256");
        let enemy_space=unit(&e,&mut s,seat.other(),"TOKEN_TIE_FIGHTER");
        let pilot=attach(&e,&mut s,seat,"JTL_210",host);
        fire_attached_when_played(&e,&mut s,pilot);
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?} space host");
        let actions=e.legal_actions(&s);
        assert!(actions.iter().any(|a|a.id()==Some(enemy_space)),"{seat:?} space enemy");
        assert!(!actions.iter().any(|a|a.id()==Some(enemy_ground)),"{seat:?} excludes other arena");
    }
}

#[test]
fn piloting_self_mandalorian_unit_companion_fires_only_as_unit() {
    let e=engine();
    // As a unit: exhaust up to 2 ground units (multi-select + Done).
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_210");
    let g1=unit(&e,&mut s,Seat::P2,"LOF_256");
    let g2=unit(&e,&mut s,Seat::P2,"LOF_254");
    let space=unit(&e,&mut s,Seat::P2,"TOKEN_XWING");
    s.source=pilot;s.trigger_context=1;s.decision=Decision::Window;
    e.queue_programs(&mut s,&["whenPlayed"]);
    e.process_all_triggers(&mut s).unwrap();
    assert_eq!(s.decision,Decision::AbilityTarget);
    let actions=e.legal_actions(&s);
    assert!(actions.iter().any(|a|a.id()==Some(g1)));
    assert!(actions.iter().any(|a|a.id()==Some(g2)));
    assert!(!actions.iter().any(|a|a.id()==Some(space)),"space excluded from ground upTo");
    let picked=e.apply(&s,&e.card_action(&s,g1)).unwrap();
    assert_eq!(picked.decision,Decision::AbilityTarget,"upTo waits for Done");
    assert!(!picked.card(g1).exhausted,"not exhausted until Done");
    let done=named_button(&e,&picked,"Done");
    assert!(done.card(g1).exhausted);
    // While attached, unit companion must not fire; only host-arena exhaust does.
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let host=unit(&e,&mut s,Seat::P1,"LOF_254");
    let enemy=unit(&e,&mut s,Seat::P2,"LOF_256");
    let pilot=attach(&e,&mut s,Seat::P1,"JTL_210",host);
    fire_attached_when_played(&e,&mut s,pilot);
    assert_eq!(s.decision,Decision::AbilityTarget);
    assert_eq!(s.source,pilot);
    let actions=e.legal_actions(&s);
    assert!(actions.iter().any(|a|a.id()==Some(enemy)));
    assert!(!actions.iter().any(|a|a.id()==Some(host)),"unit upTo must not replace host-arena exhaust");
}

#[test]
fn piloting_self_mandalorian_rejects_mismatched_companion_and_predicate() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_210").unwrap().clone();
    let catalogue_admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_210");
    let engine_admitted=|e:Value|{
        let mut full=catalog.clone();
        let cards=full["cards"].as_array_mut().unwrap();
        let slot=cards.iter_mut().find(|c|c["setCode"]=="JTL_210").unwrap();
        *slot=e;
        Engine::from_catalogue_value(&full).unwrap().by_code.contains_key("JTL_210")
    };
    assert!(catalogue_admitted(original.clone()));
    assert!(engine_admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Drop the pilotingRegistration: unit companion alone is not a piloting_self entry,
    // and the printed piloting keyword stays unimplemented without a reviewed path.
    let mut unit_only=original.clone();
    unit_only["abilities"]["card"]=json!([original["abilities"]["card"][0].clone()]);
    assert!(!crate::attached_registration::piloting_self_entry(&unit_only));
    assert!(!engine_admitted(unit_only),"companion without pilotingRegistration");
    // Widen arena filter away from host parentCard zone.
    let mut bad_pred=original.clone();
    bad_pred["abilities"]["card"][1]["effect"]["registration"]["program"]["targetResolver"]["predicate"]=json!({
        "kind":"binary","op":"===",
        "left":{"kind":"ref","path":["card","zoneName"]},
        "right":{"kind":"const","value":"groundArena"}
    });
    assert!(!catalogue_admitted(bad_pred),"static arena instead of host arena");
    // Extra keyword breaks plain_pilot.
    let mut bad_kw=original.clone();
    bad_kw["card"]["keywords"]=json!(["piloting","ambush"]);
    assert!(!catalogue_admitted(bad_kw),"extra keyword");
}

#[test]
fn piloting_self_frisk_defeats_cheap_upgrade_when_attached() {
    let e=engine();
    assert!(e.by_code.contains_key("JTL_148"),"{:?}",e.rejected_cards().unwrap().get("JTL_148"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let ally=unit(&e,&mut s,seat,"LOF_256");
        let cheap=attach(&e,&mut s,seat,"ASH_210",ally);
        let expensive=attach(&e,&mut s,seat.other(),"ASH_134",ally);
        assert_eq!(e.def(&s,cheap).info.cost(),1);
        assert_eq!(e.def(&s,expensive).info.cost(),3);
        let pilot=attach(&e,&mut s,seat,"JTL_148",host);
        fire_attached_when_played(&e,&mut s,pilot);
        // Optional + Select opens the target window (same as Astromech / Backstabber).
        assert_eq!(s.decision,Decision::AbilityTarget,"{seat:?}");
        assert_eq!(s.source,pilot);
        let actions=e.legal_actions(&s);
        assert!(actions.iter().any(|a|a.id()==Some(cheap)),"{seat:?} cheap upgrade");
        assert!(!actions.iter().any(|a|a.id()==Some(expensive)),"{seat:?} cost>2 excluded");
        let done=e.apply(&s,&e.card_action(&s,cheap)).unwrap();
        assert!(done.player(seat).discard.contains(cheap),"{seat:?}");
        assert_eq!(done.card(expensive).parent,ally,"{seat:?} expensive remains");
    }
    // Played as a unit: WhenPlayed must not fire.
    let mut s=board(&e);s.round=5;s.active=Seat::P1;s.action_player=Some(Seat::P1);
    let ally=unit(&e,&mut s,Seat::P1,"LOF_256");
    let cheap=attach(&e,&mut s,Seat::P1,"ASH_210",ally);
    let pilot=unit(&e,&mut s,Seat::P1,"JTL_148");
    fire_attached_when_played(&e,&mut s,pilot);
    assert_eq!(s.decision,Decision::Window);
    assert_eq!(s.card(cheap).parent,ally);
}

#[test]
fn piloting_self_frisk_rejects_cost_and_registration_query_drift() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="JTL_148").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("JTL_148");
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Cost ceiling is part of the reviewed Frisk contract.
    let mut high=original.clone();
    high["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]["predicate"]["right"]["right"]["value"]=json!(3);
    assert!(!admitted(high),"cost <= 3");
    // Optional is required; mandatory defeat would change the prompt protocol.
    let mut mandatory=original.clone();
    mandatory["abilities"]["card"][0]["effect"]["registration"]["program"]["optional"]=json!(false);
    assert!(!admitted(mandatory),"mandatory");
    // Unit domain is not the Frisk shape.
    let mut unit_type=original.clone();
    unit_type["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]["type"]=json!("unit");
    assert!(!admitted(unit_type),"unit type");
    // Drop hasCost: bare cost comparison is not the reviewed guard.
    let mut no_has_cost=original.clone();
    no_has_cost["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]["predicate"]=json!({
        "kind":"binary","op":"<=",
        "left":{"kind":"ref","path":["card","cost"]},
        "right":{"kind":"const","value":2}
    });
    assert!(!admitted(no_has_cost),"missing hasCost");
}

#[test]
fn attached_alias_smuggle_mention_admits_armed_to_the_teeth() {
    let e=engine();
    assert!(e.by_code.contains_key("SHD_175"),"{:?}",e.rejected_cards().unwrap().get("SHD_175"));
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e);s.round=5;s.active=seat;s.action_player=Some(seat);
        let host=unit(&e,&mut s,seat,"LOF_254");
        let ally=unit(&e,&mut s,seat,"LOF_256");
        let enemy=unit(&e,&mut s,seat.other(),"LOF_256");
        let printed=e.power(&s,ally,false);
        attach(&e,&mut s,seat,"SHD_175",host);
        let base=s.player(seat.other()).base;
        let prompt=attack(&e,s,host,base);
        assert_eq!(prompt.decision,Decision::AbilityTarget,"{seat:?}");
        assert_eq!(prompt.source,host);
        let actions=e.legal_actions(&prompt);
        assert!(actions.iter().any(|a|a.id()==Some(ally)),"{seat:?} ally");
        assert!(!actions.iter().any(|a|a.id()==Some(host)||a.id()==Some(enemy)),"{seat:?} excludes host/enemy");
        let done=e.apply(&prompt,&e.card_action(&prompt,ally)).unwrap();
        assert_eq!(e.power(&done,ally,false),printed+2,"{seat:?}");
    }
}

#[test]
fn attached_alias_smuggle_mention_requires_printed_cost_text() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original=catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="SHD_175").unwrap().clone();
    let admitted=|e:Value|crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]})).unwrap().cards.contains_key("SHD_175");
    assert!(admitted(original.clone()));
    let mut no_text=original.clone();
    no_text["card"]["text"]=json!("Attached unit gains: “On Attack: Give another friendly unit +2/+0 for this phase.”");
    assert!(!admitted(no_text),"smuggle without printed cost");
    let mut bare=original.clone();
    bare["card"]["keywords"]=json!(["smuggle","raid"]);
    assert!(!admitted(bare),"raid mention still needs a parent grant");
}
