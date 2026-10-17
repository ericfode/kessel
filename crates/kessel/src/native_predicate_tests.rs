use super::*;

fn catalogue()->Value {serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()}
pub(super) fn engine()->Engine {Engine::from_catalogue_value(&catalogue()).unwrap()}
pub(super) fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {
    let id=s.next_id;s.next_id+=1;
    *s.card_mut(id)=Card{present:true,def:*e.by_code.get(code).unwrap_or_else(||panic!("{code}: {:?}",e.rejected_cards().unwrap().get(code))),owner:seat,..Card::default()};id
}
pub(super) fn unit(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=add(e,s,seat,code);e.add_arena(s,id).unwrap();id}
pub(super) fn board(e:&Engine,actor:Seat,code:&str)->State {
    let mut s=State::default();s.credit_state_exact=e.needs_credit_state();s.hidden_history_exact=true;s.force_state_exact=true;s.active=actor;s.action_player=Some(actor);
    for seat in [Seat::P1,Seat::P2] {
        let base=add(e,&mut s,seat,"ASH_019");let leader=add(e,&mut s,seat,if seat==actor {code}else{"SOR_014"});
        s.player_mut(seat).base=base;s.player_mut(seat).leader=leader;s.player_mut(seat).force_id=s.next_id;s.next_id+=1;
    }
    for _ in 0..2 {let r=add(e,&mut s,actor,"LOF_254");s.player_mut(actor).resources.push(r).unwrap();}
    s
}
fn predicate(token:&str)->Value {json!({"kind":"nativeTokenPredicate","operation":match token {"experience"=>"hasExperience","weakness"=>"hasWeakness",_=>"hasShield"},"receiver":{"kind":"ref","path":["card"]},"domain":"physicalAttachments","tokenType":token,"optionalReceiver":false,"optionalCall":false})}
pub(super) fn has(e:&Engine,s:&State,id:CardId,token:&str)->bool {
    let p:Expr=serde_json::from_value(predicate(token)).unwrap();e.eval_with_card(s,s.player(s.active).leader,id,id,&p).truth()
}

#[test]
fn native_predicate_contract_rejects_unknown_identity_optional_and_numeric_shapes() {
    for token in ["shield","experience","weakness"] {
        let original=predicate(token);let p:Expr=serde_json::from_value(original.clone()).unwrap();p.validate().unwrap();assert!(p.validate_number().is_err());
        for (key,value) in [("operation",json!("isCreditToken")),("tokenType",json!("advantage")),("domain",json!("cardIdentity")),("optionalCall",json!(true)),("optionalReceiver",json!(true)),("receiver",json!({"kind":"ref","path":["player"]}))] {
            let mut invalid=original.clone();invalid[key]=value;assert!(serde_json::from_value::<Expr>(invalid).unwrap().validate().is_err(),"{token} {key}");
        }
        let mut invalid=original.clone();invalid["unexpected"]=json!(true);assert!(serde_json::from_value::<Expr>(invalid).is_err());
        let mut invalid=original.clone();invalid["optionalCall"]=json!(0);assert!(serde_json::from_value::<Expr>(invalid).is_err());
        let mut invalid=original.clone();invalid.as_object_mut().unwrap().remove("optionalReceiver");assert!(serde_json::from_value::<Expr>(invalid).is_err());
    }
}

#[test]
fn native_predicate_uses_live_physical_identity_and_removal_not_stat_bonuses() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat,"SOR_014");let host=unit(&e,&mut s,seat.other(),"ASH_096");let other=unit(&e,&mut s,seat,"ASH_096");
        s.card_mut(host).phase_power=3;s.card_mut(host).phase_hp=3;
        for token in ["shield","experience","weakness"] {assert!(!has(&e,&s,host,token));}
        let ordinary=e.definitions.iter().find(|d|d.info.is("upgrade")&&!d.info.is("token")).unwrap().code.clone();
        let ordinary=add(&e,&mut s,seat.other(),&ordinary);e.attach(&mut s,ordinary,host);
        assert!(!has(&e,&s,host,"experience"));assert!(!has(&e,&s,host,"shield"));
        let xp=s.next_id;e.create_token(&mut s,host,true).unwrap();let shield=s.next_id;e.create_token(&mut s,host,false).unwrap();
        s.source=s.player(seat).leader;s.ability_target=host;
        let weakness=s.next_id;let effect:Effect=serde_json::from_value(json!({"kind":"giveWeakness","target":"chosen","amount":{"kind":"const","value":1}})).unwrap();e.apply_effect(&mut s,&effect).unwrap();
        for token in ["shield","experience","weakness"] {assert!(has(&e,&s,host,token));assert!(!has(&e,&s,other,token));}
        let cloned=s;assert_eq!(cloned.zobrist(),s.zobrist());assert!(has(&e,&cloned,host,"weakness"));
        for (token,id) in [("experience",xp),("shield",shield),("weakness",weakness)] {e.defeat(&mut s,id).unwrap();assert!(!has(&e,&s,host,token));assert_eq!(s.card(id).parent,0);assert!(!s.player(seat.other()).discard.contains(id));}
        e.create_token(&mut s,host,true).unwrap();assert!(has(&e,&s,host,"experience"));e.defeat(&mut s,host).unwrap();assert!(!has(&e,&s,host,"experience"));assert!(e.upgrades(&s,host).is_empty());
    }
}

#[test]
fn native_predicate_staged_identity_reconciliation_distinguishes_native_adapters_from_conversions() {
    let c=catalogue();let staged=c["sourceSupport"].as_array().unwrap().iter().filter(|s|s["role"]=="staged-native-token-lifecycle").collect::<Vec<_>>();assert_eq!(staged.len(),6);
    let cards=c["cards"].as_array().unwrap();
    for stage in staged {let card=cards.iter().find(|c|c["id"]==stage["identity"]["id"]).unwrap();assert_eq!(card["internalName"],stage["identity"]["internalName"]);assert_eq!(card["sourcePath"],stage["sourcePath"]);assert_eq!(card["sourceHash"],stage["sourceHash"]);assert_eq!(card["status"],"unsupported");}
    assert!(!c["sourceSupport"].as_array().unwrap().iter().any(|s|s["role"]=="source-without-card-data" && s["sourcePath"]=="game/cards/08_ASH/tokens/Advantage.ts"));
    let e=engine();for code in ["TOKEN_SHIELD","TOKEN_EXPERIENCE","TOKEN_WEAKNESS","TOKEN_THE_FORCE","TOKEN_ADVANTAGE"] {assert!(e.by_code.contains_key(code));}
    assert!(e.by_code.contains_key("TOKEN_CREDIT"));
    assert!(!e.rejected_cards().unwrap().contains_key("TOKEN_CREDIT"));
    let baseline:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/runtime-admission.json")).unwrap();
    let known=baseline["runtimeAdmittedCards"].as_array().unwrap().iter().map(|c|c["setCode"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
    let new=e.by_code.keys().filter(|code|!known.contains(code.as_str())).collect::<std::collections::BTreeSet<_>>();
    eprintln!("native predicate admission: {}; converted: {}; additions versus admission report: {:?}",e.definitions.len(),cards.iter().filter(|c|c["status"]=="supported").count(),new);
}

pub(super) fn choose(e:&Engine,s:&State,id:CardId)->State {
    let a=e.card_action(s,id);assert!(e.legal_actions(s).contains(&a),"{id}: {:?}",e.legal_actions(s));e.apply(s,&a).unwrap()
}
pub(super) fn button(e:&Engine,s:&State,label:&str)->State {
    let a=e.legal_actions(s).into_iter().find(|a|a.label==label).unwrap_or_else(||panic!("{label}: {:?}",e.legal_actions(s)));e.apply(s,&a).unwrap()
}
pub(super) fn front(e:&Engine,s:&State)->State {
    let mut t=choose(e,s,s.player(s.active).leader);
    if t.decision==Decision::AbilityMenu {let a=e.legal_actions(&t).into_iter().find(|a|a.label.starts_with("Give ")).unwrap();t=e.apply(&t,&a).unwrap();}t
}
pub(super) fn weakness(e:&Engine,s:&mut State,host:CardId) {
    s.source=s.player(s.active).leader;s.ability_target=host;
    let effect:Effect=serde_json::from_value(json!({"kind":"giveWeakness","target":"chosen","amount":{"kind":"const","value":1}})).unwrap();e.apply_effect(s,&effect).unwrap();
    s.source=0;s.ability_target=0;
}

#[test]
fn native_predicate_actual_leader_actions_select_before_payment_and_filter_live_attachments() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {for code in ["HMW_003","LOF_008"] {
            let mut s=board(&e,seat,code);let leader=s.player(seat).leader;s.player_mut(seat).force_held=true;
            let eligible=unit(&e,&mut s,seat.other(),"ASH_096");let existing=unit(&e,&mut s,seat,"ASH_096");
            if code=="LOF_008" {e.create_token(&mut s,existing,true).unwrap();}else {weakness(&e,&mut s,existing);}
            let token=if code=="LOF_008" {"experience"}else{"weakness"};let before=s;
            let prompt=front(&e,&s);assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(e.ready_resources(&prompt,seat),2);assert!(!prompt.card(leader).exhausted);assert!(prompt.player(seat).force_held);
            assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,eligible)));assert!(!e.legal_actions(&prompt).contains(&e.card_action(&prompt,existing)));
            for player in [seat,seat.other()] {assert!(!e.legal_actions(&prompt).contains(&e.card_action(&prompt,prompt.player(player).base)));}
            assert_eq!(button(&e,&prompt,"Cancel"),before);
            let next=prompt.next_id;let done=choose(&e,&prompt,eligible);assert!(has(&e,&done,eligible,token));assert_eq!(done.card(next).parent,eligible);assert_eq!(done.card(next).owner,seat.other());assert!(done.card(leader).exhausted);
            assert_eq!(e.ready_resources(&done,seat),if code=="HMW_003" {1}else{2});assert_eq!(done.player(seat).force_held,code=="HMW_003");assert_eq!(done.player(seat).force_id,s.player(seat).force_id);
            let mut removed=s;if code=="LOF_008" {e.defeat(&mut removed,s.card(existing).experience).unwrap();}else {let id=e.upgrades(&removed,existing).into_iter().find(|id|Some(removed.card(*id).def)==e.weakness_def).unwrap();e.defeat(&mut removed,id).unwrap();}
            let prompt=front(&e,&removed);assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,existing)));
        }}
    }
}

#[test]
fn native_predicate_actual_attack_sides_preserve_optional_pass_source_exclusion_and_stacking() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {for code in ["HMW_003","LOF_008"] {
            let mut s=board(&e,seat,code);let leader=s.player(seat).leader;s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();
            let target=unit(&e,&mut s,seat.other(),"ASH_096");let existing=unit(&e,&mut s,seat,"ASH_096");
            if code=="LOF_008" {e.create_token(&mut s,existing,true).unwrap();}else {weakness(&e,&mut s,existing);}
            let declared=choose(&e,&s,leader);assert_eq!(declared.decision,Decision::AttackTarget);
            let prompt=choose(&e,&declared,s.player(seat.other()).base);assert_eq!(prompt.decision,Decision::AbilityTarget);
            assert!(e.legal_actions(&prompt).iter().any(|a|a.label=="Pass"));assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,target)));
            assert_eq!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,leader)),code=="HMW_003");assert_eq!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,existing)),code=="HMW_003");
            let next=prompt.next_id;let passed=button(&e,&prompt,"Pass");assert_eq!(passed.next_id,next);assert_eq!(passed.card(target).experience,0);
            let chosen=if code=="HMW_003" {existing}else{target};let done=choose(&e,&prompt,chosen);assert_eq!(done.card(next).parent,chosen);assert_eq!(done.card(next).owner,done.card(chosen).owner);
            if code=="HMW_003" {assert_eq!(e.upgrades(&done,chosen).into_iter().filter(|id|Some(done.card(*id).def)==e.weakness_def).count(),2);}else {assert!(has(&e,&done,chosen,"experience"));}
        }}
    }
}

#[test]
fn native_predicate_presence_only_requires_reviewed_and_selected_native_identity() {
    for (token,code) in [("experience","TOKEN_EXPERIENCE"),("weakness","TOKEN_WEAKNESS")] {
        let mut c=catalogue();let diagnostic=c["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="SOR_073").unwrap();
        // Raw-IR boundary fixture: presence requires the native definition even
        // though this program only creates a Shield, never this queried token.
        diagnostic["abilities"]["card"][0]["targetResolver"]["predicate"]=predicate(token);
        let e=Engine::from_catalogue_value(&c).unwrap();assert!(e.by_code.contains_key("SOR_073"));
        let mut pool=e.compiled_pool().unwrap();pool["cards"].as_array_mut().unwrap().retain(|c|["SOR_073","TOKEN_SHIELD",code].contains(&c["setCode"].as_str().unwrap()));
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),&serde_json::to_vec(&c).unwrap()).unwrap();
        pool["cards"].as_array_mut().unwrap().retain(|c|c["setCode"]!=code);
        assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),&serde_json::to_vec(&c).unwrap()).err().unwrap().contains(code));
        for remove in [true,false] {let mut bad=c.clone();if remove {bad["cards"].as_array_mut().unwrap().retain(|c|c["setCode"]!=code);}else {bad["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]==code).unwrap()["sourceHash"]=json!("changed");}
            let bad=Engine::from_catalogue_value(&bad).unwrap();assert!(!bad.by_code.contains_key("SOR_073"));assert!(bad.rejected_cards().unwrap()["SOR_073"].contains("token unavailable"));}
    }
}

#[test]
fn native_predicate_leader_weakness_complete_registration_and_native_boundaries() {
    let original=catalogue();let source=original["cards"].as_array().unwrap().iter().find(|c|c["setCode"]=="HMW_003").unwrap();
    let reject=|entry:Value| {let loaded=crate::ability_ir::Catalogue::load(&json!({"schema":original["schema"],"cards":[entry]})).unwrap();assert!(loaded.rejected.contains_key("HMW_003"));};
    for class in ["NonLeaderUnitCard","LeaderCard","UpgradeCard"] {let mut entry=source.clone();entry["baseClass"]=json!(class);reject(entry);}
    for types in [json!(["unit"]),json!(["leader","unit"]),json!(["leader","token"])] {let mut entry=source.clone();entry["card"]["types"]=types;reject(entry);}
    for side in ["leader","unit"] {let mut entry=source.clone();entry["abilities"][side]=json!([]);reject(entry);}
    let mut entry=source.clone();entry["abilities"]["leader"][0]["costs"][0]["amount"]["value"]=json!(2);reject(entry);
    let mut entry=source.clone();entry["abilities"]["unit"][0]["optional"]=json!(false);reject(entry);
    let mut entry=source.clone();entry["abilities"]["leader"][0]["targetResolver"]["predicate"]["right"]["value"]["tokenType"]=json!("experience");reject(entry);
    for code in ["TOKEN_WEAKNESS","TOKEN_EXPERIENCE","TOKEN_THE_FORCE"] {let mut c=original.clone();c["cards"].as_array_mut().unwrap().retain(|c|c["setCode"]!=code);let e=Engine::from_catalogue_value(&c).unwrap();let dependent=if code=="TOKEN_WEAKNESS" {"HMW_003"}else{"LOF_008"};assert!(!e.by_code.contains_key(dependent));}
}

#[test]
fn native_predicate_actual_zillo_aura_attack_weakness_and_source_departure_compose() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e,seat,"SOR_014");let own=unit(&e,&mut s,seat,"ASH_096");let tiny=unit(&e,&mut s,seat,"LOF_254");let enemy=unit(&e,&mut s,seat.other(),"ASH_096");
            let zillo=add(&e,&mut s,seat,"HMW_065");s.player_mut(seat).hand.push(zillo).unwrap();
            for _ in 0..8 {let r=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(r).unwrap();}
            let mut played=choose(&e,&s,zillo);assert!(e.in_arena(&played,zillo));assert_eq!(e.hp(&played,zillo),6);assert_eq!(e.power(&played,zillo,false),6);
            assert_eq!(e.hp(&played,own),2);assert_eq!(e.power(&played,own,false),1);assert!(played.player(seat).discard.contains(tiny));assert_eq!(e.hp(&played,enemy),4);assert_eq!(e.power(&played,enemy,false),3);
            played.active=seat;played.action_player=Some(seat);played.card_mut(zillo).exhausted=false;
            let attack=choose(&e,&played,zillo);assert_eq!(attack.decision,Decision::AttackTarget);let prompt=choose(&e,&attack,played.player(seat.other()).base);assert_eq!(prompt.decision,Decision::AbilityTarget);
            assert!(e.legal_actions(&prompt).iter().any(|a|a.label=="Pass"));assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,own)));assert!(e.legal_actions(&prompt).contains(&e.card_action(&prompt,enemy)));
            let next=prompt.next_id;let mut done=choose(&e,&prompt,own);assert_eq!(done.card(next).parent,own);assert_eq!(done.card(next).owner,seat);assert!(has(&e,&done,own,"weakness"));assert_eq!(e.hp(&done,own),1);assert_eq!(e.power(&done,own,false),0);
            let snapshot=e.snapshot(&done);assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()),snapshot);
            e.defeat(&mut done,zillo).unwrap();assert_eq!(e.hp(&done,own),3);assert_eq!(e.power(&done,own,false),2);assert!(has(&e,&done,own,"weakness"));
        }
    }
}
