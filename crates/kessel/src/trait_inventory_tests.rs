use super::*;
use crate::ability_ir::{Catalogue,Program};

// Expected from Card.getTraits: printed + every gain, then every loss, as a set.
// Malakili's complete source distinguishes controlled in-play units from owned
// nonplay units; the admitted importer still rejects foreign control. Zam uses
// Player.getLeaderCards and a Set union, excluding Force, in every source zone.
#[test]
fn trait_inventory_malakili_actual_source_covers_live_owner_zones_and_departure() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut s=board(&e);
            let creature=unit(&e,&mut s,seat,"LOF_257");
            let enemy=unit(&e,&mut s,seat.other(),"LOF_257");
            let noncreature=unit(&e,&mut s,seat,"LOF_256");
            let source=unit(&e,&mut s,seat,"LAW_212");
            let mut hidden=vec![];
            for zone in ["hand","discard","deck","resources"] {
                let id=add(&e,&mut s,seat,"LOF_257");
                match zone {"hand"=>s.player_mut(seat).hand.push(id).unwrap(),"discard"=>s.player_mut(seat).discard.push(id).unwrap(),"deck"=>s.player_mut(seat).deck.push(id).unwrap(),_=>s.player_mut(seat).resources.push(id).unwrap()};
                hidden.push(id);
            }
            assert!(e.card_has_trait(&s,creature,"underworld"));
            assert!(!e.card_has_trait(&s,enemy,"underworld"));
            assert!(!e.card_has_trait(&s,noncreature,"underworld"));
            assert!(e.card_has_trait(&s,source,"underworld"));
            for &id in &hidden {assert!(e.card_has_trait(&s,id,"underworld"));assert!(e.card_has_trait(&s,id,"creature"));}
            e.defeat(&mut s,creature).unwrap();
            assert!(s.player(seat).discard.contains(creature));
            assert!(e.card_has_trait(&s,creature,"underworld"),"owned nonplay Creature retains the live aura");
            let imported=e.load_state(&e.snapshot(&s)).unwrap();
            assert!(e.card_has_trait(&imported,creature,"underworld"));
            e.defeat(&mut s,source).unwrap();
            for id in hidden.into_iter().chain([creature]) {assert!(!e.card_has_trait(&s,id,"underworld"));assert!(e.card_has_trait(&s,id,"creature"));}
            assert!(e.card_has_trait(&s,source,"underworld"),"printed trait remains after source departure");
        }
    }
}
#[test]
fn trait_inventory_zam_actual_source_unions_friendly_leader_traits_in_all_zones_except_force() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            for zone in ["arena","hand","discard","deck","resources"] {
                let mut s=board(&e);
                let source=add(&e,&mut s,seat,"HMW_134");
                match zone {"arena"=>e.add_arena(&mut s,source).unwrap(),"hand"=>s.player_mut(seat).hand.push(source).unwrap(),"discard"=>s.player_mut(seat).discard.push(source).unwrap(),"deck"=>s.player_mut(seat).deck.push(source).unwrap(),_=>s.player_mut(seat).resources.push(source).unwrap()};
                let traits=e.card_traits(&s,source);
                assert!(traits.contains("underworld"));assert!(traits.contains("bounty hunter"));
                assert!(!traits.contains("force"));
                if seat==Seat::P1 {
                    for own in ["mandalorian","rebel","spectre"] {assert!(traits.contains(own));}
                    for enemy in ["imperial","sith"] {assert!(!traits.contains(enemy));}
                    assert_eq!(traits.len(),5);
                } else {
                    for own in ["imperial","sith"] {assert!(traits.contains(own));}
                    for enemy in ["mandalorian","rebel","spectre"] {assert!(!traits.contains(enemy));}
                    assert_eq!(traits.len(),4);
                }
                let copy=s;assert_eq!(e.card_traits(&copy,source),traits);
                let imported=e.load_state(&e.snapshot(&s)).unwrap();
                assert_eq!(e.card_traits(&imported,source),traits);
            }
        }
    }
}
fn diagnostic_parent_trait(e:&mut Engine,s:&State,upgrade:CardId,kind:&str,trait_name:&str) {
    death_program(e,s,upgrade,json!({"trigger":"constant","title":"Synthetic trait leaf","attachedContext":"upgrade","effect":{"kind":kind,"target":"parent","trait":trait_name,"duration":"persistent"}}));
}
#[test]
fn trait_inventory_shared_parent_leaves_apply_gain_then_loss_and_recompute_aura_dependencies() {
    for seat in [Seat::P1,Seat::P2] {
        let mut e=engine();let mut s=board(&e);
        let host=unit(&e,&mut s,seat,"LOF_256");
        let _source=unit(&e,&mut s,seat,"LAW_212");
        let gain=add(&e,&mut s,seat.other(),"SHD_070");e.attach(&mut s,gain,host);
        diagnostic_parent_trait(&mut e,&s,gain,"gainTrait","creature");
        assert!(e.card_has_trait(&s,host,"creature"));
        assert!(e.card_has_trait(&s,host,"underworld"));
        let loss=add(&e,&mut s,seat,"SOR_120");e.attach(&mut s,loss,host);
        diagnostic_parent_trait(&mut e,&s,loss,"loseTrait","creature");
        assert!(!e.card_has_trait(&s,host,"creature"));
        assert!(!e.card_has_trait(&s,host,"underworld"));
        e.defeat(&mut s,loss).unwrap();
        assert!(e.card_has_trait(&s,host,"creature"));
        assert!(e.card_has_trait(&s,host,"underworld"));
        e.defeat(&mut s,gain).unwrap();
        assert!(!e.card_has_trait(&s,host,"creature"));
        assert!(!e.card_has_trait(&s,host,"underworld"));
        assert_eq!(s.card(gain).owner,seat.other(),"upgrade controller stays separate from host");
    }
}
#[test]
fn trait_inventory_zam_uses_live_gained_and_lost_deployed_leader_traits() {
    for seat in [Seat::P1,Seat::P2] {
        let mut e=engine();let mut s=board(&e);
        let source=unit(&e,&mut s,seat,"HMW_134");
        let leader=s.player(seat).leader;s.card_mut(leader).deployed=true;s.card_mut(leader).epic_used=true;e.add_arena(&mut s,leader).unwrap();
        let gain=add(&e,&mut s,seat,"SHD_070");e.attach(&mut s,gain,leader);
        diagnostic_parent_trait(&mut e,&s,gain,"gainTrait","trooper");
        assert!(e.card_has_trait(&s,leader,"trooper"));assert!(e.card_has_trait(&s,source,"trooper"));
        let query:crate::ability_ir::Expr=serde_json::from_value(json!({"kind":"query","name":"leaderTraits","receiver":{"kind":"ref","path":["player"]},"args":[]})).unwrap();
        let Eval::Array(inventory)=e.eval(&s,source,0,&query) else {panic!("leader trait inventory")};
        assert!(inventory.contains(&Eval::Text("trooper".into())));
        let loss=add(&e,&mut s,seat.other(),"SOR_120");e.attach(&mut s,loss,leader);
        diagnostic_parent_trait(&mut e,&s,loss,"loseTrait","trooper");
        assert!(!e.card_has_trait(&s,source,"trooper"));
        let copied=s;assert!(!e.card_has_trait(&copied,source,"trooper"));
        e.defeat(&mut s,loss).unwrap();assert!(e.card_has_trait(&s,source,"trooper"));
        e.defeat(&mut s,gain).unwrap();assert!(!e.card_has_trait(&s,source,"trooper"));
    }
}
#[test]
fn trait_inventory_persistent_leaves_reject_event_mutation_and_dependency_scopes() {
    let plain=json!({"trigger":"constant","title":"Native trait","effect":{"kind":"gainTrait","target":"self","trait":"trooper","duration":"persistent"}});
    let compile=|v:Value|Program::compile(&serde_json::from_value(v).unwrap());
    assert!(compile(plain.clone()).is_ok());
    for mode in ["event","whenPlayed","sequence","conditional","cost","selector","condition","phase","dynamic","unknown"] {
        let mut v=plain.clone();
        match mode {
            "event"|"whenPlayed"=>v["trigger"]=json!(mode),
            "sequence"=>v["effect"]=json!({"kind":"sequence","effects":[v["effect"].clone()]}),
            "conditional"=>v["effect"]=json!({"kind":"conditional","condition":{"kind":"const","value":true},"onTrue":v["effect"].clone()}),
            "cost"=>v["costs"]=json!([{"kind":"exhaustSelf"}]),
            "selector"=>v["targetResolver"]=json!({"owner":"self","type":"unit","zone":"arena"}),
            "condition"=>v["condition"]=json!({"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["source"]},"args":[{"kind":"const","value":"creature"}]}),
            "phase"=>v["effect"]["duration"]=json!("phase"),
            "dynamic"=>v["effect"]["trait"]=json!({"kind":"const","value":"trooper"}),
            _=>v["effect"]["unmodeled"]=json!(true),
        }
        if matches!(mode,"dynamic"|"unknown") {assert!(serde_json::from_value::<crate::ability_ir::Ability>(v).is_err(),"{mode}");}
        else {assert!(compile(v).is_err(),"{mode}");}
    }
}
#[test]
fn trait_inventory_native_wrappers_reject_unknown_fields_wrong_roles_and_trait_cycles() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["HMW_134","LAW_212"] {
        let entry=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap();
        for mode in ["unknown","source","target","callback","leader","self-cycle"] {
            let mut v=entry.clone();let e=&mut v["abilities"]["card"][0]["effect"];
            match mode {
                "unknown"=>e["properties"]["unmodeled"]=json!(true),
                "source"=>e["properties"]["sourceZoneFilter"]=json!("hand"),
                "target"=>e["properties"]["targetController"]=json!("opponent"),
                "callback"=>if code=="HMW_134" {e["modifier"]["value"]["callback"]["parameters"][1]["argument"]=json!("targetCard");}else{e["properties"]["matchTarget"]["parameters"][1]["role"]=json!("candidate");},
                "leader"=>{v["baseClass"]=json!("LeaderUnitCard");v["card"]["types"]=json!(["leader"]);},
                _=>{if code=="HMW_134" {continue;}e["modifier"]["value"]["value"]=json!("creature");},
            }
            let c=Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap();
            assert!(c.rejected.contains_key(code),"{code}:{mode}");
        }
    }
    let original=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="LAW_212").unwrap().clone();
    let mut reverse=original.clone();reverse["setCode"]=json!("RAW_TRAIT_CYCLE");
    reverse["abilities"]["card"][0]["effect"]["properties"]["matchTarget"]["body"]["left"]["right"]["args"][0]["value"]=json!("underworld");
    reverse["abilities"]["card"][0]["effect"]["modifier"]["value"]["value"]=json!("creature");
    let c=Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[original,reverse]})).unwrap();
    assert_eq!(c.rejected.len(),2);assert!(c.rejected.values().all(|r|r.contains("dependency cycle")));
}
#[test]
fn trait_inventory_foreign_control_imports_remain_explicitly_rejected() {
    let e=engine();let mut s=board(&e);
    unit(&e,&mut s,Seat::P1,"LAW_212");
    let host=unit(&e,&mut s,Seat::P1,"LOF_257");
    let upgrade=add(&e,&mut s,Seat::P2,"SHD_070");e.attach(&mut s,upgrade,host);
    let snapshot=e.snapshot(&s);
    assert!(e.load_state(&snapshot).is_ok(),"enemy-owned attachment is a supported control identity");
    for field in ["owner","controller"] {
        let mut v=snapshot.clone();
        v["players"]["p1"]["ground"][0][field]=json!("p2");
        assert!(e.load_state(&v).is_err(),"unrecorded unit {field} must not invent ownership history");
    }
    let mut v=snapshot.clone();
    v["players"]["p1"]["ground"][1]["upgrades"][0]["owner"]=json!("p1");
    assert!(e.load_state(&v).is_err(),"foreign-controlled upgrade must preserve explicit rejection");
}
#[test]
fn trait_inventory_rejects_source_contexts_and_copying_leader_union_into_support() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["HMW_134","LAW_212","HMW_208"] {
        let entry=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap();
        for key in ["stateWatchers","historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"] {
            let mut v=entry.clone();v[key]=json!([{"kind":"unmodeled"}]);
            let c=Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap();
            assert!(c.rejected.contains_key(code),"{code}:{key}");
        }
    }
    let entry=original["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="HMW_134").unwrap();
    let mut v=entry.clone();v["keywordValues"]["support"]=json!(1);v["card"]["keywords"].as_array_mut().unwrap().push(json!("support"));v["card"]["text"]=json!("Support\nThis unit gains each friendly leader's traits except Force.");
    let c=Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]})).unwrap();assert!(c.rejected.contains_key("HMW_134"));
    let e=engine();let mut s=board(&e);
    let donor_code=e.by_code.iter().find_map(|(code,id)|(e.definitions[*id as usize].flags & SUPPORT!=0 && !e.definitions[*id as usize].info.is("leader")).then(||code.clone())).unwrap();
    let donor=unit(&e,&mut s,Seat::P1,&donor_code);
    let leader=s.player(Seat::P1).leader;s.card_mut(leader).deployed=true;s.card_mut(leader).epic_used=true;e.add_arena(&mut s,leader).unwrap();
    let zam_program=e.definitions[e.by_code["HMW_134"] as usize].programs.card[0];
    let mut snapshot=e.snapshot(&s);
    snapshot["supportGrants"]=json!([{"target":format!("c{}",leader-1),"donor":format!("c{}",donor-1),"programs":[serde_json::to_value(e.program(zam_program)).unwrap()],"keywords":[0,0,0,0,0,0,0,0,0],"presence":0}]);
    let err=e.load_state(&snapshot).unwrap_err();
    assert!(err.contains("copied leader-trait union"),"{err}");
}
#[test]
fn trait_inventory_generated_trait_predicate_uses_same_live_inventory_as_interpreter() {
    let mut e=engine();let mut s=board(&e);
    let creature=unit(&e,&mut s,Seat::P1,"LOF_257");
    let outer=unit(&e,&mut s,Seat::P2,"LOF_257");
    let expr_json=json!({"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"underworld"}]});
    let manifest:Vec<Value>=serde_json::from_str(GENERATED_EXPRESSIONS).unwrap();
    let row=manifest.iter().find(|v|v["expression"]==expr_json && v["index"].is_u64()).expect("canonical compiled trait predicate from actual admitted cards");
    let index=row["index"].as_u64().unwrap() as usize;
    let expression:crate::ability_ir::Expr=serde_json::from_value(expr_json).unwrap();
    let source=unit(&e,&mut s,Seat::P1,"LAW_212");
    let loss=add(&e,&mut s,Seat::P2,"SHD_070");e.attach(&mut s,loss,creature);
    diagnostic_parent_trait(&mut e,&s,loss,"loseTrait","underworld");
    for (stage,expected) in [(0,false),(1,true),(2,false)] {
        if stage==1 {e.defeat(&mut s,loss).unwrap();}
        if stage==2 {e.defeat(&mut s,source).unwrap();}
        e.set_rules_backend(RulesBackend::Interpreter).unwrap();
        assert_eq!(e.eval_with_binding(&s,source,outer,creature,0,&expression),Eval::Bool(expected));
        e.set_rules_backend(RulesBackend::Auto).unwrap();
        assert_eq!(e.generated_expression(&s,source,outer,creature,0,index),Eval::Bool(expected));
        assert_eq!(e.generated_expression(&s,source,creature,outer,0,index),Eval::Bool(false),"candidate stays distinct from outer target");
    }
}
