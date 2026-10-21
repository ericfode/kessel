//! Synthetic native boundary tests. Actual source conformance is separate.
use super::*;
fn fixture(e:&Engine,seat:Seat,credits:u16)->(State,CardId){
    let mut s=board(e,seat);s.support_state_explicit=true;s.unit_entry_state_explicit=true;
    for _ in 0..2 {let id=add(e,&mut s,seat,"ASH_237");s.player_mut(seat).discard.push(id).unwrap();install(e,&mut s,id,0);}
    for _ in 0..16 {let id=add(e,&mut s,seat,"IBH_007");s.player_mut(seat).resources.push(id).unwrap();}
    let id=add(e,&mut s,seat,"SOR_232");s.player_mut(seat).hand.push(id).unwrap();
    e.create_credit_tokens(&mut s,seat,credits).unwrap();(s,id)
}
#[test]
fn direct_credit_native_reached_roundtrip_and_all_continuations(){
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter]{let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {for credits in [1,3] {
            let(mut reached,id)=fixture(&e,seat,credits);assert!(!e.play_unit_with_payment(&mut reached,id,false).unwrap());
            let wire=e.snapshot(&reached);assert!(wire.get("directUnitPlayCreditState").is_some(),"{wire}");
            assert!(wire.get("creditPaymentPending").is_none());assert!(wire.get("playerPhasePending").is_none());
            let loaded=e.load_state(&wire).unwrap();assert_eq!(loaded.zobrist(),reached.zobrist());assert_eq!(e.snapshot(&loaded),wire);assert_eq!(e.legal_actions(&loaded),e.legal_actions(&reached));
            for action in e.legal_actions(&reached){let a=e.apply(&reached,&action).unwrap();let b=e.apply(&loaded,&action).unwrap();assert_eq!(a.zobrist(),b.zobrist());assert_eq!(e.snapshot(&a),e.snapshot(&b));
                if a.decision==Decision::CreditAmount {let number_wire=e.snapshot(&a);assert_eq!(number_wire["directUnitPlayCreditState"]["menu"]["minimum"],0);assert_eq!(number_wire["directUnitPlayCreditState"]["menu"]["canCancel"],true);let x=e.load_state(&number_wire).unwrap();assert_eq!(x.zobrist(),a.zobrist());assert_eq!(e.legal_actions(&x),e.legal_actions(&a));for n in e.legal_actions(&a){assert_eq!(e.apply(&x,&n).unwrap().zobrist(),e.apply(&a,&n).unwrap().zobrist());}}
            }
        }}
    }
}
#[test]
fn direct_credit_complete_mask_shape_and_other_pending_reject(){
    let e=engine();let(mut s,id)=fixture(&e,Seat::P1,3);e.play_unit_with_payment(&mut s,id,false).unwrap();let wire=e.snapshot(&s);assert!(wire.get("directUnitPlayCreditState").is_some());
    for key in ["playerPhasePending","creditPaymentPending","resourcePaymentPending","numberSelectionPending","supportAttackPending","defeatCausePending"] {let mut v=wire.clone();v[key]=json!(false);assert!(e.load_state(&v).is_err(),"{key}");}
    for ids in [json!([]),json!([1]),json!([2,1]),json!([1,1]),json!([1,2,3]),json!([0]),json!([65])]{let mut v=wire.clone();v["directUnitPlayCreditState"]["capturedMouseIds"]=ids;assert!(e.load_state(&v).is_err());}
    for (key,value) in [("preCreditAmount",json!(0)),("preCreditAmount",json!(65536)),("version",json!(2)),("extra",json!(true)),("stage",json!("paid")),("source",json!("c01")),("payer",json!("p2"))]{let mut v=wire.clone();v["directUnitPlayCreditState"][key]=value;assert!(e.load_state(&v).is_err(),"{key}");}
    for value in [json!(null),json!(false),json!({})]{let mut v=wire.clone();v["directUnitPlayCreditState"]=value;assert!(e.load_state(&v).is_err());}
    for key in ["playerPhaseState","creditStateVersion","actor","winner","phase","activePlayer"]{let mut v=wire.clone();v.as_object_mut().unwrap().remove(key);assert!(e.load_state(&v).is_err());}
    let mut foreign=s;foreign.ability_target=id;assert!(e.snapshot(&foreign).get("directUnitPlayCreditState").is_none());
    let mut foreign=s;foreign.death_frames[7].context.source=id;assert!(e.snapshot(&foreign).get("directUnitPlayCreditState").is_none());
}
#[test]
fn direct_credit_high_mask_bank_and_spent_rows_survive_hash_copy(){
    let e=engine();let(mut s,id)=fixture(&e,Seat::P2,3);let origins=[s.player_phase.rows[0],s.player_phase.rows[1]];
    let mut ledger=Ledger::default();for _ in 0..32 {let r=origins[0];ledger.append(Row::new(r.source,r.program,Kind::ImperialDiscount,1,1,2).unwrap()).unwrap();}
    for r in origins {ledger.append(r).unwrap();}s.player_phase=ledger;
    e.play_unit_with_payment(&mut s,id,false).unwrap();assert_eq!(s.resource_payment.phase_adjusters.0,[0,3]);
    let wire=e.snapshot(&s);assert_eq!(wire["directUnitPlayCreditState"]["capturedMouseIds"],json!([33,34]));
    let x=e.load_state(&wire).unwrap();assert_eq!(x.zobrist(),s.zobrist());assert_eq!(x.player_phase,s.player_phase);
    let mut changed=s;changed.resource_payment.phase_adjusters.0[1]^=1;assert_ne!(changed.zobrist(),s.zobrist());assert!(e.snapshot(&changed).get("directUnitPlayCreditState").is_none());
    for ids in [json!([1,33,34]),json!([33]),json!([34])]{let mut v=wire.clone();v["directUnitPlayCreditState"]["capturedMouseIds"]=ids;assert!(e.load_state(&v).is_err());}
}
#[test]
fn direct_credit_positive_minimum_forced_use_and_quote_mutants(){
    let e=engine();let(mut s,id)=fixture(&e,Seat::P1,3);let amount=e.cost(&s,id);assert_eq!(amount,6); // includes this fixture's aspect penalty before two Mouse decreases.
    let resources=s.player(Seat::P1).resources.as_slice().to_vec();for r in resources.into_iter().skip(usize::from(amount-3)){s.card_mut(r).exhausted=true;}
    e.play_unit_with_payment(&mut s,id,false).unwrap();let wire=e.snapshot(&s);assert_eq!(wire["directUnitPlayCreditState"]["menu"]["minimum"],3);assert_eq!(wire["directUnitPlayCreditState"]["menu"]["maximum"],3);
    let loaded=e.load_state(&wire).unwrap();assert_eq!(loaded.zobrist(),s.zobrist());let actions=e.legal_actions(&s);assert_eq!(actions.len(),2);assert!(actions[0].label.starts_with("Use 3 Credit"));assert_eq!(actions[1].label,"Cancel");
    for (path,value) in [("minimum",json!(0)),("maximum",json!(2)),("canCancel",json!(false)),("extra",json!(1))]{let mut v=wire.clone();v["directUnitPlayCreditState"]["menu"][path]=value;assert!(e.load_state(&v).is_err());}
    let mut v=wire.clone();v["directUnitPlayCreditState"]["preCreditAmount"]=json!(amount+1);assert!(e.load_state(&v).is_err());
    let mut v=wire;v["directUnitPlayCreditState"]["stage"]=json!("amount");assert!(e.load_state(&v).is_err());
}
/// Explicit external, immutable source evidence lane; never substitutes for
/// terminal whole-card conformance. Run by name with the bound receipt path.
#[test]
#[ignore]
fn direct_credit_source_readbacks_reached_imported_exact(){
    use sha2::{Digest,Sha256};
    let root=std::path::PathBuf::from(std::env::var("SWU_PHASE_CREDIT_READBACKS").expect("set immutable source readback directory"));
    let receipt:Value=serde_json::from_slice(&std::fs::read(root.join("receipt.json")).unwrap()).unwrap();
    let catalog=serde_json::to_vec(&phase_catalogue()).unwrap();
    let mut count=0;
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter]{
        let mut engines=std::collections::HashMap::new();
        for (index,row) in receipt["observedStates"].as_array().unwrap().iter().enumerate(){
            let hash=row["pool"]["sha256"].as_str().expect("actual bound pool hash");
            if !engines.contains_key(hash){let bytes=std::fs::read(root.join("pools").join(format!("{hash}.json"))).unwrap();assert_eq!(format!("{:x}",Sha256::digest(&bytes)),hash);let mut e=Engine::from_compiled_pool_bytes(&bytes,&catalog).unwrap();e.set_rules_backend(backend).unwrap();engines.insert(hash.to_owned(),e);}
            let e=&engines[hash];let source=&row["state"];
            let loaded=e.load_state(source).unwrap_or_else(|err|panic!("source row {index}: {err}"));
            assert_eq!(e.snapshot(&loaded),*source,"source snapshot row {index}");
            let payer=loaded.active;
            let expected=row["menus"].as_array().unwrap().iter().find(|m|m["actor"]==payer.as_str()).unwrap()["actions"].as_array().unwrap().iter().map(|a|{let mut a=a.clone();a.as_object_mut().unwrap().remove("key");a}).collect::<Vec<_>>();
            assert_eq!(serde_json::to_value(e.legal_actions(&loaded)).unwrap(),json!(expected),"source menu row {index}");
            // Independently REACH the boundary by ordinary native play from the
            // source-proven unchanged prepayment board. The loader itself does
            // not execute this action or reconstruct a lexical source closure.
            let mut base=source.clone();base.as_object_mut().unwrap().remove("directUnitPlayCreditState");base["actor"]=json!(payer.as_str());
            let before=e.load_state(&base).unwrap();let action=e.legal_actions(&before).into_iter().find(|a|a.id()==Some(loaded.source)).unwrap();
            let mut reached=e.apply(&before,&action).unwrap();
            if loaded.decision==Decision::CreditAmount {let select=e.legal_actions(&reached).into_iter().find(|a|a.label=="Select amount").unwrap();reached=e.apply(&reached,&select).unwrap();}
            assert_eq!(reached.zobrist(),loaded.zobrist(),"reached/imported full hash row {index}");assert_eq!(e.snapshot(&reached),*source,"reached source snapshot row {index}");
            for action in e.legal_actions(&loaded){let a=e.apply(&loaded,&action).unwrap();let b=e.apply(&reached,&action).unwrap();assert_eq!(a.zobrist(),b.zobrist(),"successor hash row {index}");assert_eq!(e.snapshot(&a),e.snapshot(&b));}
            println!("{}",json!({"sourceRow":index,"backend":format!("{backend:?}"),"poolSha256":hash,"fullHash":loaded.zobrist(),"sourceSnapshot":true,"sourceMenu":true,"reachedEqualsImported":true}));count+=1;
        }
    }
    let rows=std::env::var("SWU_PHASE_CREDIT_EXPECTED_ROWS").unwrap_or_else(|_|"88".into()).parse::<usize>().unwrap();
    assert_eq!(count,rows*2);
}

fn phase_free_credit_engine(backend:RulesBackend)->Engine {
    let original=engine();let mut pool=original.compiled_pool().unwrap();
    pool["cards"].as_array_mut().unwrap().retain(|r|r["setCode"]!="ASH_237"&&r["setCode"]!="ASH_248");
    let mut e=Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),&serde_json::to_vec(&phase_catalogue()).unwrap()).unwrap();
    e.set_rules_backend(backend).unwrap();assert!(!e.needs_player_phase_state());e
}
fn phase_free_credit_fixedpoint(e:&Engine,s:&State,present:bool){
    let wire=e.snapshot(s);assert_eq!(wire.get("playerPhaseState").is_some(),present);
    let loaded=e.load_state(&wire).unwrap();assert_eq!(e.snapshot(&loaded),wire);assert_eq!(loaded.zobrist(),s.zobrist());
    assert_eq!(e.legal_actions(&loaded),e.legal_actions(s));let copy=s.clone();assert_eq!(e.snapshot(&copy),wire);assert_eq!(copy.zobrist(),s.zobrist());
}
#[test]
fn direct_credit_phase_free_natural_pending_and_optional_presence_all_branches(){
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter]{let e=phase_free_credit_engine(backend);
  for seat in [Seat::P1,Seat::P2]{for explicit in [false,true]{
   let mut s=board(&e,seat);s.support_state_explicit=true;s.unit_entry_state_explicit=true;
   for _ in 0..8{let r=add(&e,&mut s,seat,"IBH_007");s.player_mut(seat).resources.push(r).unwrap();}
   let id=add(&e,&mut s,seat,"SOR_095");s.player_mut(seat).hand.push(id).unwrap();e.create_credit_tokens(&mut s,seat,2).unwrap();
   let mut wire=e.snapshot(&s);assert!(wire.get("playerPhaseState").is_none());
   if explicit{wire["playerPhaseState"]=json!({"version":1,"effects":[]});}
   let initial=e.load_state(&wire).unwrap();phase_free_credit_fixedpoint(&e,&initial,explicit);
   let play=e.legal_actions(&initial).into_iter().find(|a|a.id()==Some(id)).unwrap();
   let reached=e.apply(&initial,&play).unwrap();let pending=e.snapshot(&reached);
   assert!(pending.get("directUnitPlayCreditState").is_some(),"natural direct Credit pending must not require Phase presence");
   assert!(pending.get("creditPaymentPending").is_none());assert!(pending.get("playerPhasePending").is_none());
   phase_free_credit_fixedpoint(&e,&reached,explicit);let imported=e.load_state(&pending).unwrap();
   for action in e.legal_actions(&reached){let a=e.apply(&reached,&action).unwrap();let b=e.apply(&imported,&action).unwrap();assert_eq!(a.zobrist(),b.zobrist());assert_eq!(e.snapshot(&a),e.snapshot(&b));phase_free_credit_fixedpoint(&e,&a,explicit);
    if action.label=="Cancel"{assert_eq!(e.snapshot(&a),wire);assert_eq!(a.zobrist(),initial.zobrist());}
    if a.decision==Decision::CreditAmount{let loaded=e.load_state(&e.snapshot(&a)).unwrap();for number in e.legal_actions(&a){let x=e.apply(&a,&number).unwrap();let y=e.apply(&loaded,&number).unwrap();assert_eq!(x.zobrist(),y.zobrist());assert_eq!(e.snapshot(&x),e.snapshot(&y));phase_free_credit_fixedpoint(&e,&x,explicit);}}
   }
  }}
 }
}

/// Fresh actual Phase-free source rows and complete source continuations.
/// The prepayment board used to reach pending is an explicitly derived unchanged board.
#[test]
#[ignore]
fn direct_credit_phase_free_actual_source_rows_and_terminal(){
 phase_free_credit_actual_source_check(true);
}
#[test]
#[ignore]
fn direct_credit_phase_free_actual_live_boundaries_and_terminal_json(){
 phase_free_credit_actual_source_check(false);
}
fn phase_free_credit_actual_source_check(strict_terminal:bool){
 use sha2::{Digest,Sha256};
 let root=std::path::PathBuf::from(std::env::var("SWU_PHASE_FREE_READBACKS").expect("frozen source packet root"));
 let bytes=std::fs::read(root.join("phase-free-credit.json")).unwrap();
 assert_eq!(format!("{:x}",Sha256::digest(&bytes)),std::env::var("SWU_PHASE_FREE_READBACKS_SHA256").unwrap());
 let cases:Value=serde_json::from_slice(&bytes).unwrap();let cat=serde_json::to_vec(&phase_catalogue()).unwrap();let mut rows=0;let mut terminals=0;
 for backend in [RulesBackend::Auto,RulesBackend::Interpreter]{
  for case in cases.as_array().unwrap(){let states=case["states"].as_array().unwrap();if states.is_empty(){continue;}
   let mut last=None;
   for row in states{let hash=row["pool"]["sha256"].as_str().unwrap();let pool=std::fs::read(root.join("pools").join(format!("{hash}.json"))).unwrap();assert_eq!(format!("{:x}",Sha256::digest(&pool)),hash);
    let mut e=Engine::from_compiled_pool_bytes(&pool,&cat).unwrap();e.set_rules_backend(backend).unwrap();assert!(!e.needs_player_phase_state());
    let wire=&row["state"];assert!(wire.get("playerPhaseState").is_none());let loaded=e.load_state(wire).unwrap();phase_free_credit_fixedpoint(&e,&loaded,false);assert_eq!(e.snapshot(&loaded),*wire);
    let expected=row["actions"].as_array().unwrap().iter().map(|a|{let mut a=a.clone();a.as_object_mut().unwrap().remove("key");a}).collect::<Vec<_>>();assert_eq!(serde_json::to_value(e.legal_actions(&loaded)).unwrap(),json!(expected));
    let mut before=wire.clone();before.as_object_mut().unwrap().remove("directUnitPlayCreditState");before["actor"]=json!(loaded.active.as_str());let initial=e.load_state(&before).unwrap();assert_eq!(e.snapshot(&initial),before);
    let play=e.legal_actions(&initial).into_iter().find(|a|a.id()==Some(loaded.source)).unwrap();let mut reached=e.apply(&initial,&play).unwrap();if loaded.decision==Decision::CreditAmount{let a=e.legal_actions(&reached).into_iter().find(|a|a.label=="Select amount").unwrap();reached=e.apply(&reached,&a).unwrap();}
    assert_eq!(e.snapshot(&reached),*wire);assert_eq!(reached.zobrist(),loaded.zobrist());
    for action in e.legal_actions(&loaded){let a=e.apply(&loaded,&action).unwrap();let b=e.apply(&reached,&action).unwrap();assert_eq!(e.snapshot(&a),e.snapshot(&b));assert_eq!(a.zobrist(),b.zobrist());phase_free_credit_fixedpoint(&e,&a,false);}
    let key=row["chosenAction"]["key"].as_str().unwrap();let action=e.legal_actions(&loaded).into_iter().find(|a|a.key()==key).unwrap();let successor=e.apply(&loaded,&action).unwrap();assert_eq!(e.snapshot(&successor),row["successor"]["state"]);phase_free_credit_fixedpoint(&e,&successor,false);
    for menu in row["successor"]["menus"].as_array().unwrap(){if menu["actor"]==successor.active.as_str(){let expected=menu["actions"].as_array().unwrap().iter().map(|a|{let mut a=a.clone();a.as_object_mut().unwrap().remove("key");a}).collect::<Vec<_>>();assert_eq!(serde_json::to_value(e.legal_actions(&successor)).unwrap(),json!(expected));}}
    eprintln!("{}",json!({"seat":case["seat"],"mode":case["mode"],"stage":wire["directUnitPlayCreditState"]["stage"],"backend":format!("{backend:?}"),"reachedFullHash":format!("{:016x}",reached.zobrist()),"importedFullHash":format!("{:016x}",loaded.zobrist()),"successorFullHash":format!("{:016x}",successor.zobrist()),"pendingSourceExact":true,"successorSourceExact":true,"cloneReloadFixedpoint":true}));last=Some((e,successor));rows+=1;
   }
   let (e,mut state)=last.unwrap();for record in case["forward"]["actions"].as_array().unwrap(){let key=record["action"]["key"].as_str().unwrap();let action=e.legal_actions(&state).into_iter().find(|a|a.key()==key).unwrap();state=e.apply(&state,&action).unwrap();}
   let wire=e.snapshot(&state);assert_eq!(wire,case["forward"]["finalState"]);
   let copy=state.clone();assert_eq!(e.snapshot(&copy),wire);assert_eq!(copy.zobrist(),state.zobrist());
   let reloaded=e.load_state(&wire).unwrap();assert_eq!(e.snapshot(&reloaded),wire);
   assert!(state.is_terminal()&&reloaded.is_terminal());assert!(e.legal_actions(&state).is_empty()&&e.legal_actions(&reloaded).is_empty());
   // This original strict assertion intentionally remains an executable known
   // counterexample: inherited Window versus Terminal is absent from the wire.
   if strict_terminal {phase_free_credit_fixedpoint(&e,&state,false);}
   eprintln!("{}",json!({"seat":case["seat"],"mode":case["mode"],"backend":format!("{backend:?}"),"terminalFullHash":format!("{:016x}",state.zobrist()),"terminalReloadFullHash":format!("{:016x}",reloaded.zobrist()),"terminalSourceExact":true,"terminalCloneFullHashExact":true,"terminalReloadHashPositive":false,"reachedDecision":format!("{:?}",state.decision),"reloadedDecision":format!("{:?}",reloaded.decision)}));terminals+=1;
  }
 }
 assert_eq!(rows,16);assert_eq!(terminals,12);eprintln!("actual Phase-free source rows {rows}; complete terminal continuations {terminals}");
}
