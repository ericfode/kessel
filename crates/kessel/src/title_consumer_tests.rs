//! Canonical consumers of the existing controlled in-play title leaf.
use super::*;

fn midgame(e:&Engine,seat:Seat)->State {let mut s=board(e);s.round=6;s.active=seat;s.action_player=Some(seat);s}
fn resources(e:&Engine,s:&mut State,seat:Seat,n:usize) {for _ in 0..n {let id=add(e,s,seat,"LOF_256");s.player_mut(seat).resources.push(id).unwrap();}}
fn exact_roundtrip(e:&Engine,s:&State)->State {let v=e.snapshot(s);let copy=e.load_state(&v).unwrap();assert_eq!(e.snapshot(&copy),v);assert_eq!(e.legal_actions(&copy),e.legal_actions(s));assert_eq!(copy.zobrist(),s.zobrist());copy}
fn button(e:&Engine,s:&State,label:&str)->Action {e.legal_actions(s).into_iter().find(|a|a.label==label).unwrap_or_else(||panic!("missing {label}: {:?}",e.legal_actions(s)))}
fn title_engine(code:&str,title:&str)->Engine {let mut c:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();c["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]==code).unwrap()["card"]["title"]=json!(title);Engine::from_catalogue_value(&c).unwrap()}

#[test]
fn title_consumers_firespray_paid_entry_reads_live_jango_and_controlled_enemy_host_upgrade() {
    for seat in [Seat::P1,Seat::P2] {for witness in [0,1,2,3] {
        // Actual Jango is canonical; the title-only upgrade carrier is labeled
        // synthetic because no admitted printed Boba title source exists.
        let e=if witness==3 {title_engine("HMW_235","Boba Fett")}else{engine()};
        let mut s=midgame(&e,seat);resources(&e,&mut s,seat,12);
        if witness==1||witness==2 {unit(&e,&mut s,if witness==1 {seat}else{seat.other()},"LAW_087");}
        if witness==3 {let host=unit(&e,&mut s,seat.other(),"LOF_256");let upgrade=add(&e,&mut s,seat,"HMW_235");e.attach(&mut s,upgrade,host);}
        let source=add(&e,&mut s,seat,"SOR_184");s.player_mut(seat).hand.push(source).unwrap();
        let s=exact_roundtrip(&e,&s);let before=e.ready_resources(&s,seat);let cost=e.cost(&s,source) as usize;
        let play=e.card_action(&s,source);assert!(e.legal_actions(&s).contains(&play));let done=e.apply(&s,&play).unwrap();
        assert!(done.player(seat).space.contains(source));assert_eq!(e.ready_resources(&done,seat),before-cost);
        assert_eq!(done.card(source).exhausted,witness==0||witness==2);exact_roundtrip(&e,&done);
    }}
}

#[test]
fn title_consumers_firespray_action_cost_targets_any_nonunique_unit() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=midgame(&e,seat);resources(&e,&mut s,seat,2);
        let source=unit(&e,&mut s,seat,"SOR_184");let target=unit(&e,&mut s,seat.other(),"LOF_256");
        let unique=unit(&e,&mut s,seat,"LAW_087");
        let s=exact_roundtrip(&e,&s);let opened=e.apply(&s,&e.card_action(&s,source)).unwrap();
        let choosing=if opened.decision==Decision::AbilityTarget {opened}else{e.apply(&opened,&button(&e,&opened,"Exhaust a non-unique unit")).unwrap()};
        assert_eq!(choosing.decision,Decision::AbilityTarget);assert_eq!(e.ready_resources(&choosing,seat),2);
        assert!(e.legal_actions(&choosing).contains(&e.card_action(&choosing,target)));assert!(!e.legal_actions(&choosing).contains(&e.card_action(&choosing,unique)));
        let cancelled=e.apply(&choosing,&button(&e,&choosing,"Cancel")).unwrap();assert_eq!(e.ready_resources(&cancelled,seat),2);assert!(!cancelled.card(target).exhausted);
        let done=e.apply(&choosing,&e.card_action(&choosing,target)).unwrap();assert_eq!(e.ready_resources(&done,seat),0);assert!(done.card(target).exhausted);assert!(!done.card(source).exhausted);exact_roundtrip(&e,&done);
    }
}

#[test]
fn title_consumers_black_one_optional_damage_and_upgraded_bonus_remain_separate() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for presence in [false,true] {for accept in [false,true] {
        let mut s=midgame(&e,seat);let source=unit(&e,&mut s,seat,"JTL_147");
        let damaged=unit(&e,&mut s,seat.other(),"SOR_082");
        if presence {unit(&e,&mut s,seat,"ASH_040");}else{unit(&e,&mut s,seat.other(),"ASH_040");}
        let base=s.player(seat.other()).base;let base_power=e.power(&s,source,false);
        let token=add(&e,&mut s,seat,"TOKEN_EXPERIENCE");s.card_mut(source).experience=token;e.attach(&mut s,token,source);
        assert_eq!(e.power(&s,source,false),base_power+2,"physical XP plus independent upgraded bonus");
        let s=exact_roundtrip(&e,&s);let choosing=e.apply(&s,&e.card_action(&s,source)).unwrap();assert_eq!(choosing.decision,Decision::AttackTarget);
        let mut done=e.apply(&choosing,&e.card_action(&choosing,base)).unwrap();
        if presence {
            assert_eq!(done.decision,Decision::AbilityTarget);
            assert!(e.legal_actions(&done).contains(&e.card_action(&done,damaged)));
            done=if accept {e.apply(&done,&e.card_action(&done,damaged)).unwrap()}else{e.apply(&done,&button(&e,&done,"Pass")).unwrap()};
        }
        assert_eq!(done.card(damaged).damage,u16::from(presence&&accept));assert_eq!(done.card(base).damage,(base_power+2) as u16);assert!(done.card(source).exhausted);exact_roundtrip(&e,&done);
    }}}
}

#[test]
fn title_consumers_shuttle_death_uses_live_title_and_can_grant_enemy_experience() {
    // Source, Restore keyword and XP effect remain canonical; admitted inert
    // carrier is renamed solely to exercise unavailable printed Chancellor title.
    let e=title_engine("LOF_254","Chancellor Palpatine");
    for seat in [Seat::P1,Seat::P2] {for title_alive in [false,true] {for accept in [false,true] {
        let mut s=midgame(&e,seat);let source=unit(&e,&mut s,seat,"SEC_027");let carrier=unit(&e,&mut s,seat,"LOF_254");let target=unit(&e,&mut s,seat.other(),"LOF_256");
        assert_eq!(e.keyword_amount(&s,source,"restore"),1);
        e.defeat(&mut s,source).unwrap();assert!(s.player(seat).discard.contains(source));
        if !title_alive {e.defeat(&mut s,carrier).unwrap();}
        e.finish(&mut s).unwrap();
        if title_alive {assert_eq!(s.decision,Decision::AbilityOptional);s=e.apply(&s,&button(&e,&s,if accept {"Trigger"}else{"Pass"})).unwrap();
            if accept {assert_eq!(s.decision,Decision::AbilityTarget);assert!(e.legal_actions(&s).contains(&e.card_action(&s,target)));s=e.apply(&s,&e.card_action(&s,target)).unwrap();}
        }
        assert_eq!(e.token_count(&s,s.card(target).experience),u16::from(title_alive&&accept));
        assert_eq!(s.card(source).experience,0);exact_roundtrip(&e,&s);
    }}}
}

#[test]
fn title_consumers_full_source_and_registration_mutations_remain_rejected() {
    let original:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for code in ["JTL_147","SEC_027","SOR_184"] {for mutation in ["pilot","type","side","condition","then","optional","token-amount"] {
        let mut c=original.clone();let entry=c["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]==code).unwrap();
        let index=if code=="JTL_147" {1}else{0};let a=&mut entry["abilities"]["card"][index];
        match mutation {
            "condition"=>a["condition"]=json!({"kind":"const","value":true}),
            "then"=>a["then"]=json!({"title":"unreviewed","effect":{"kind":"noAction"}}),
            "optional"=>a["optional"]=json!(code=="SOR_184"),
            "token-amount"=>{if code=="SEC_027" {a["effect"]["onTrue"]["effect"]["amount"]["value"]=json!(2);}else{a["effect"]["onFalse"]=json!({"kind":"ready","target":"self"});}},
            "pilot"=>entry["card"]["pilotText"]=json!("unsupported"),
            "type"=>entry["card"]["types"]=json!(["unit","token"]),
            _=>entry["abilities"]["unit"]=json!([{"trigger":"onAttack","title":"extra","effect":{"kind":"noAction"}}]),
        }
        let e=Engine::from_catalogue_value(&c).unwrap();assert!(!e.by_code.contains_key(code),"admitted {code} mutation {mutation}");
    }}
    let mut missing=original;missing["cards"].as_array_mut().unwrap().retain(|e|e["setCode"]!="TOKEN_EXPERIENCE");
    let e=Engine::from_catalogue_value(&missing).unwrap();assert!(!e.by_code.contains_key("SEC_027"));assert!(e.by_code.contains_key("SOR_184"));
}
