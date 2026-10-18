//! Physical collection fixtures exercise committed card IR, never a reference engine.
use super::*;
use serde_json::{json, Value};

fn expr(value: Value) -> Expr { let e:Expr=serde_json::from_value(value).unwrap(); e.validate().unwrap(); e }
fn collection(name:&str, receiver:Value, argument:Value)->Value {json!({"kind":"collection","name":name,"receiver":receiver,"args":[argument],"limit":256})}
fn reference(path:&[&str])->Value {json!({"kind":"ref","path":path})}
fn constant(value:Value)->Value {json!({"kind":"const","value":value})}
fn query(name:&str,receiver:Value,args:Vec<Value>)->Value {json!({"kind":"query","name":name,"receiver":receiver,"args":args})}

#[test]
fn collection_query_admission_and_strict_boundaries() {
    let e=engine();
    for code in ["JTL_247"] {assert!(e.by_code.contains_key(code),"{code}: {:?}",e.rejected_cards().unwrap().get(code));}
    eprintln!("collection admission: {} executable; {} rejected",e.definitions.len(),e.rejected_cards().unwrap().len());
    let valid=collection("some",reference(&["card","upgrades"]),query("isTokenUpgrade",reference(&["card"]),vec![]));
    expr(valid.clone());
    for mode in ["method","arity","limit","receiver","callback-list","extra"] {
        let mut bad=valid.clone();
        match mode {"method"=>bad["name"]=json!("find"),"arity"=>bad["args"]=json!([]),"limit"=>bad["limit"]=json!(255),"receiver"=>bad["receiver"]=reference(&["card"]),"callback-list"=>bad["args"][0]=reference(&["card","upgrades"]),_=>bad["unbounded"]=json!(true)}
        assert!(serde_json::from_value::<Expr>(bad).map_or(true,|e|e.validate().is_err()),"{mode}");
    }
    let bad=collection("some",collection("map",reference(&["player","hand"]),reference(&["card","power"])),constant(json!(true)));
    assert!(serde_json::from_value::<Expr>(bad).unwrap().validate().is_err());
    assert!(expr(valid).validate_number().is_err());
    let list=collection("filter",reference(&["player","hand"]),constant(json!(false)));
    for bad in [
        json!({"kind":"binary","op":"===","left":{"kind":"conditional","condition":constant(json!(true)),"then":list.clone(),"else":constant(Value::Null)},"right":list.clone()}),
        json!({"kind":"unary","op":"-","value":list.clone()}),
        query("isUnit",list,vec![]),
    ] {assert!(serde_json::from_value::<Expr>(bad).unwrap().validate().is_err());}
}

#[test]
fn collection_query_methods_preserve_order_identity_outer_scope_and_empty_results() {
    let e=engine();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LOF_256");
    let target=unit(&e,&mut s,Seat::P2,"LOF_256");let first=add(&e,&mut s,Seat::P1,"HMW_235");let second=add(&e,&mut s,Seat::P1,"HMW_235");
    s.players[0].hand.push(first).unwrap();s.players[0].hand.push(second).unwrap();
    let inventory=reference(&["player","hand"]);
    let same_target=json!({"kind":"binary","op":"===","left":reference(&["card"]),"right":reference(&["target"])});
    for (name,expected) in [("some",false),("every",false)] {let x=expr(collection(name,inventory.clone(),same_target.clone()));assert_eq!(e.eval(&s,source,target,&x),Eval::Bool(expected));}
    let x=expr(collection("filter",inventory.clone(),same_target.clone()));assert_eq!(e.eval(&s,source,second,&x),Eval::Array(vec![Eval::Card(second)]));
    let x=expr(collection("map",inventory.clone(),reference(&["card"])));assert_eq!(e.eval(&s,source,target,&x),Eval::Array(vec![Eval::Card(first),Eval::Card(second)]));
    let x=expr(collection("includes",inventory.clone(),reference(&["target"])));assert_eq!(e.eval(&s,source,first,&x),Eval::Bool(true));assert_eq!(e.eval(&s,source,target,&x),Eval::Bool(false));
    let titles=collection("map",inventory.clone(),reference(&["card","title"]));
    let x=expr(collection("includes",titles,constant(json!(e.def(&s,first).info.title))));assert_eq!(e.eval(&s,source,target,&x),Eval::Bool(true));
    let source_only=json!({"kind":"binary","op":"===","left":reference(&["source"]),"right":reference(&["target"])});
    let x=expr(collection("every",inventory.clone(),source_only));assert_eq!(e.eval(&s,source,source,&x),Eval::Bool(true));assert_eq!(e.eval(&s,source,target,&x),Eval::Bool(false));
    s.players[0].hand=Default::default();
    for (name,expected) in [("some",false),("every",true)] {let x=expr(collection(name,inventory.clone(),constant(json!(true))));assert_eq!(e.eval(&s,source,target,&x),Eval::Bool(expected));}
    for name in ["filter","map"] {
        let raw=collection(name,inventory.clone(),reference(&["card"]));let x=expr(raw.clone());assert_eq!(e.eval(&s,source,target,&x),Eval::Array(vec![]));
        let not=expr(json!({"kind":"unary","op":"!","value":raw.clone()}));assert_eq!(e.eval(&s,source,target,&not),Eval::Bool(false),"empty JavaScript arrays are truthy");
        let conditional=expr(json!({"kind":"conditional","condition":raw,"then":constant(json!(1)),"else":constant(json!(2))}));assert_eq!(e.eval(&s,source,target,&conditional),Eval::Number(1));
    }
}

#[test]
fn collection_query_extracted_bossk_front_targets_tokens_and_deals_two_damage() {
    let mut observations=vec![];
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LOF_256");
        let plain=unit(&e,&mut s,Seat::P2,"LOF_256");let ordinary_host=unit(&e,&mut s,Seat::P2,"LOF_256");let upgrade=add(&e,&mut s,Seat::P1,"HMW_235");e.attach(&mut s,upgrade,ordinary_host);
        let xp_host=unit(&e,&mut s,Seat::P2,"LOF_256");e.create_token(&mut s,xp_host,true).unwrap();let shield_host=unit(&e,&mut s,Seat::P1,"LOF_256");e.create_token(&mut s,shield_host,false).unwrap();
        // Bossk's leader front still requires the separate native Weakness slice.
        // Compile only its unchanged unit registration onto a diagnostic host.
        let catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
        let ability=catalogue["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="HMW_015").unwrap()["abilities"]["unit"][0].clone();
        let program=death_program(&mut e,&s,source,ability);s.source=source;e.begin_program(&mut s,program,ProgramContinuation::Finish).unwrap();
        // On Attack is optional; the existing scheduler offers targets with Pass.
        assert_eq!(s.decision,Decision::AbilityTarget);
        let targets:Vec<_>=e.legal_actions(&s).iter().filter_map(Action::id).collect();assert!(targets.contains(&xp_host));assert!(targets.contains(&shield_host));assert!(!targets.contains(&plain));assert!(!targets.contains(&ordinary_host));assert!(!targets.contains(&source));
        let mut removed=s;e.leave_play_primary(&mut removed,s.card(xp_host).experience).unwrap();assert!(!e.legal_actions(&removed).iter().filter_map(Action::id).any(|id|id==xp_host));
        let done=e.apply(&s,&e.card_action(&s,xp_host)).unwrap();assert_eq!(done.card(xp_host).damage,2);assert_eq!(done.card(ordinary_host).damage,0);assert_eq!(done.card(source).damage,0);
        observations.push((e.snapshot(&done),done.zobrist()));
    }
    assert_eq!(observations[0],observations[1]);
}

#[test]
fn collection_query_xwing_live_pilot_aura_generated_and_interpreted() {
    // No admitted ordinary Pilot currently exists. Only this fixture upgrade's
    // trait/stat metadata is synthetic; the X-Wing's complete committed IR stays intact.
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {
        let mut catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
        let entry=catalogue["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]=="HMW_235").unwrap();entry["card"]["traits"]=json!(["pilot"]);entry["card"]["upgradePower"]=json!(0);entry["card"]["upgradeHp"]=json!(0);
        let mut e=Engine::from_catalogue_value(&catalogue).unwrap();e.set_rules_backend(backend).unwrap();e.set_rules_instrumentation(true);let mut s=board(&e);let host=unit(&e,&mut s,seat,"JTL_247");let other=unit(&e,&mut s,seat.other(),"JTL_247");
        let power=e.power(&s,host,false);let hp=e.hp(&s,host);e.create_token(&mut s,host,false).unwrap();assert_eq!(e.power(&s,host,false),power);assert_eq!(e.hp(&s,host),hp);
        let pilot=add(&e,&mut s,seat.other(),"HMW_235");e.attach(&mut s,pilot,host);*e.expression_hits.lock().unwrap()=[0,0];assert_eq!(e.power(&s,host,false),power+1);assert_eq!(e.hp(&s,host),hp+1);let hits=*e.expression_hits.lock().unwrap();match backend {RulesBackend::Auto=>{assert!(hits[0]>0);assert_eq!(hits[1],0);},RulesBackend::Interpreter=>{assert_eq!(hits[0],0);assert!(hits[1]>0);},_=>unreachable!()};
        assert_eq!(e.power(&s,other,false),power);e.detach(&mut s,pilot);assert_eq!(e.hp(&s,host),hp);e.attach(&mut s,pilot,host);assert_eq!(e.hp(&s,host),hp+1);
        s.card_mut(host).damage=hp;e.sweep_stat_defeats(&mut s).unwrap();assert!(e.in_arena(&s,host));e.leave_play_primary(&mut s,pilot).unwrap();e.sweep_stat_defeats(&mut s).unwrap();assert!(!e.in_arena(&s,host));assert!(s.player(seat).discard.contains(host));
    }}
}

fn attachment_engine(backend:RulesBackend,synthetic:bool)->Engine {
    let mut catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    if synthetic {
        // Distinguish native Shield identity from an ordinary upgrade's title,
        // and keep one unique attachment for the non-unique removal fixture.
        for entry in catalogue["cards"].as_array_mut().unwrap() {
            if entry["setCode"]=="HMW_235" {entry["card"]["title"]=json!("Shield");}
            if entry["setCode"]=="IC27_093" {entry["card"]["unique"]=json!(true);}
        }
    }
    let mut e=Engine::from_catalogue_value(&catalogue).unwrap();
    if backend==RulesBackend::Aot {for definition in &mut e.definitions {if definition.code!="JTL_180" {definition.programs=Default::default();}}}
    e.set_rules_backend(backend).unwrap();e
}

#[test]
fn collection_query_piercing_shot_removes_every_native_shield_before_damage_in_all_backends() {
    for seat in [Seat::P1,Seat::P2] {for shield_count in [0,1,3,40] {
        let mut observations=vec![];
        for backend in [RulesBackend::Auto,RulesBackend::Interpreter,RulesBackend::Aot] {
            let e=attachment_engine(backend,true);let mut s=board(&e);s.active=seat;s.action_player=Some(seat);s.round=5;
            let host=unit(&e,&mut s,seat.other(),"LOF_256");let ordinary=add(&e,&mut s,seat,"HMW_235");e.attach(&mut s,ordinary,host);e.create_token(&mut s,host,true).unwrap();let xp=s.card(host).experience;
            let mut shields=vec![];for _ in 0..shield_count {e.create_token(&mut s,host,false).unwrap();shields.push(s.card(host).shield);}
            let source=add(&e,&mut s,seat,"JTL_180");s.player_mut(seat).hand.push(source).unwrap();for _ in 0..8 {let r=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(r).unwrap();}
            let chosen=e.apply(&s,&e.card_action(&s,source)).unwrap();assert_eq!(chosen.decision,Decision::AbilityTarget);
            assert!(e.legal_actions(&chosen).contains(&e.card_action(&chosen,host)));let done=e.apply(&chosen,&e.card_action(&chosen,host)).unwrap();
            assert_eq!(done.card(host).damage,3);assert!(e.in_arena(&done,host));assert_eq!(done.card(host).shield,0);assert_eq!(done.card(host).experience,xp);assert_eq!(done.card(ordinary).parent,host);assert_eq!(e.upgrades(&done,host),vec![ordinary,xp]);
            for token in shields {assert_eq!(done.card(token).parent,0);assert!(!done.player(seat.other()).discard.contains(token));}
            assert!(done.player(seat).discard.contains(source));observations.push((without_empty_named_play_state(e.snapshot(&done)),done.zobrist()));
        }
        assert_eq!(observations[0],observations[1]);assert_eq!(observations[1],observations[2]);
    }}
}

#[test]
fn collection_query_extracted_nonunique_defeat_snapshots_order_then_sweeps_parent_once() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=attachment_engine(backend,true);let mut s=board(&e);let source=unit(&e,&mut s,Seat::P1,"LOF_256");let host=unit(&e,&mut s,Seat::P2,"LOF_256");
        let first=add(&e,&mut s,Seat::P2,"HMW_235");e.attach(&mut s,first,host);let unique=add(&e,&mut s,Seat::P2,"IC27_093");e.attach(&mut s,unique,host);let second=add(&e,&mut s,Seat::P2,"IC27_052");e.attach(&mut s,second,host);e.create_token(&mut s,host,true).unwrap();let xp=s.card(host).experience;e.create_token(&mut s,host,false).unwrap();let shield=s.card(host).shield;
        let before_damage=s;s.card_mut(host).damage=e.hp(&s,host)-1;
        // SEC149's Plot keyword remains unsupported: use its complete unchanged
        // When Played registration on a diagnostic ordinary-unit host only.
        let catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let ability=catalogue["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="SEC_149").unwrap()["abilities"]["card"][0].clone();
        let program=death_program(&mut e,&s,source,ability);s.source=source;e.begin_program(&mut s,program,ProgramContinuation::Finish).unwrap();assert_eq!(s.decision,Decision::AbilityTarget);let done=e.apply(&s,&e.card_action(&s,host)).unwrap();
        assert!(!e.in_arena(&done,host));assert_eq!(done.players[1].discard.as_slice(),&[first,second,unique,host]);for id in [first,unique,second,xp,shield] {assert_eq!(done.card(id).parent,0);}
        let mut alive=before_damage;alive.source=source;e.begin_program(&mut alive,program,ProgramContinuation::Finish).unwrap();let done=e.apply(&alive,&e.card_action(&alive,host)).unwrap();assert!(e.in_arena(&done,host));assert_eq!(e.upgrades(&done,host),vec![unique]);assert_eq!(done.players[1].discard.as_slice(),&[first,second]);
        // A full discard zone rejects the entire attachment window atomically.
        let mut full=before_damage;full.source=source;full.ability_target=host;for _ in 0..crate::model::MAX_ZONE {let id=add(&e,&mut full,Seat::P2,"LOF_254");full.players[1].discard.push(id).unwrap();}
        let original=full;let effect=e.program(program).instructions.iter().find_map(|i|if let crate::ability_ir::Instruction::Apply(effect)=i {Some(effect)}else{None}).unwrap();assert!(e.apply_effect(&mut full,effect).is_err());assert_eq!(full,original);assert_eq!(full.zobrist(),original.zobrist());
    }
}

#[test]
fn collection_query_attachment_defeat_rejects_unverified_groups_scopes_and_metadata() {
    let original:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let e=engine();assert!(e.by_code.contains_key("JTL_180"));assert!(!e.by_code.contains_key("SEC_149"),"Plot is not implemented");
    for mode in ["source-receiver","card-receiver","other-method","wrong-predicate","reverse-sequence","extra-step","opponent-selector","side","keyword"] {
        let mut catalogue=original.clone();let entry=catalogue["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]=="JTL_180").unwrap();
        match mode {
            "source-receiver"=>entry["abilities"]["card"][0]["effect"]["effects"][0]["target"]["receiver"]=reference(&["source","upgrades"]),
            "card-receiver"=>entry["abilities"]["card"][0]["effect"]["effects"][0]["target"]["receiver"]=reference(&["card","upgrades"]),
            "other-method"=>entry["abilities"]["card"][0]["effect"]["effects"][0]["target"]["name"]=json!("map"),
            "wrong-predicate"=>entry["abilities"]["card"][0]["effect"]["effects"][0]["target"]["args"][0]=constant(json!(true)),
            "reverse-sequence"=>entry["abilities"]["card"][0]["effect"]["effects"].as_array_mut().unwrap().reverse(),
            "extra-step"=>entry["abilities"]["card"][0]["effect"]["effects"].as_array_mut().unwrap().push(json!({"kind":"noAction"})),
            "opponent-selector"=>entry["abilities"]["card"][0]["targetResolver"]["owner"]=json!("opponent"),
            "side"=>entry["abilities"]["unit"]=entry["abilities"]["card"].clone(),
            _=>entry["card"]["keywords"]=json!(["plot"]),
        }
        let e=Engine::from_catalogue_value(&catalogue).unwrap();assert!(!e.by_code.contains_key("JTL_180"),"{mode}");
    }
}
