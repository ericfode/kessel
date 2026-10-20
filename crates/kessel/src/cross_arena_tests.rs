//! Source-derived focused runtime witnesses; oracle comparisons remain separate.
use super::*;
fn engine()->Engine {Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap()}
fn add(e:&Engine,s:&mut State,owner:Seat,code:&str)->CardId {
    let id=s.next_id;s.next_id+=1;
    *s.card_mut(id)=Card{present:true,owner,def:*e.by_code.get(code).unwrap_or_else(||panic!("missing {code}")),..Card::default()};id
}
fn unit(e:&Engine,s:&mut State,owner:Seat,space:bool)->CardId {
    let def=e.definitions.iter().position(|d|d.info.is("unit") && d.info.arena.as_deref()==Some(if space{"space"}else{"ground"}) && d.programs.card.is_empty() && d.flags==0).unwrap();
    let id=s.next_id;s.next_id+=1;*s.card_mut(id)=Card{present:true,owner,def:def as u16,..Card::default()};e.add_arena(s,id).unwrap();id
}
fn board(e:&Engine,seat:Seat)->State {
    let mut s=State::default();s.active=seat;s.action_player=Some(seat);s.round=6;
    s.hidden_history_exact=true;s.force_state_exact=true;s.credit_state_exact=e.needs_credit_state();
    for p in [Seat::P1,Seat::P2] {let b=add(e,&mut s,p,"ASH_019");s.player_mut(p).base=b;let l=add(e,&mut s,p,if p==Seat::P1{"SOR_014"}else{"IBH_053"});s.player_mut(p).leader=l;s.player_mut(p).force_id=s.next_id;s.next_id+=1;}
    s
}
fn donor(e:&Engine,s:&mut State,seat:Seat)->CardId {let id=add(e,s,seat,"ASH_037");e.add_arena(s,id).unwrap();id}
#[test]
fn cross_arena_complete_profile_and_negative_raw_contracts() {
    let c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    let e=c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="ASH_037").unwrap();
    assert!(crate::cross_arena_contract::entry(e));
    for patch in [json!({"support":0}),json!({}),json!({"support":1,"ambush":1})] {let mut e=e.clone();e["keywordValues"]=patch;assert!(!crate::cross_arena_contract::entry(&e));}
    for (field,value) in [("baseClass",json!("LeaderUnitCard")),("piloting",json!({})),("stateWatchers",json!([]))] {let mut e=e.clone();e[field]=value;assert!(!crate::cross_arena_contract::entry(&e));}
    for (field,value) in [("permission",json!("canAttackAnything")),("target",json!("chosen")),("duration",json!("phase")),("extra",json!(true))] {let mut e=e.clone();e["abilities"]["card"][0]["effect"]["effects"][0][field]=value;assert!(!crate::cross_arena_contract::entry(&e));}
}
#[test]
fn cross_arena_printed_and_support_both_directions_preserve_sentinel_arena() {
    let e=engine();
    for seat in [Seat::P1,Seat::P2] {for space in [false,true] {
        let mut s=board(&e,seat);let d=donor(&e,&mut s,seat);let a=unit(&e,&mut s,seat,space);
        let same=unit(&e,&mut s,seat.other(),space);let opposite=unit(&e,&mut s,seat.other(),!space);let base=s.player(seat.other()).base;
        assert!(!e.attack_targets(&s,a,false).contains(&opposite));
        assert!(e.attack_targets(&s,d,false).contains(&same));assert!(e.attack_targets(&s,d,false).contains(&opposite));
        s.source=d;assert!(e.support_targets(&s).contains(&a));e.begin_support_attack(&mut s,a).unwrap();
        let both=if space {vec![opposite,same,base]}else{vec![same,opposite,base]};
        assert_eq!(e.attack_targets(&s,a,false),both);
        assert!(!e.attack_targets(&s,a,true).contains(&base));
        s.card_mut(opposite).add_phase_keyword(8);s.card_mut(opposite).set_hidden_qualified(true);
        assert!(!e.attack_targets(&s,a,false).contains(&opposite));
        s.card_mut(opposite).clear_hidden_phase_effects();
        // An opposite-arena Sentinel alone cannot constrain this attacker.
        s.card_mut(opposite).phase_keyword_presence|=1;
        assert_eq!(e.attack_targets(&s,a,false),both);
        s.card_mut(same).phase_keyword_presence|=1;
        assert_eq!(e.attack_targets(&s,a,false),both[..2]);
        s.card_mut(a).phase_keyword_presence|=4;
        assert_eq!(e.attack_targets(&s,a,false),both);
        // Donor departure retains the copied constant, recipient departure removes it.
        e.defeat(&mut s,d).unwrap();assert!(e.attack_targets(&s,a,false).contains(&opposite));
        e.clear_support_grants(&mut s);assert!(!e.attack_targets(&s,a,false).contains(&opposite));
    }}
}
#[test]
fn cross_arena_support_pregrant_only_opposite_defender_and_restrictions() {
    use crate::ability_ir::RestrictionVerb;
    let e=engine();for space in [false,true] {
        let mut s=board(&e,Seat::P1);let d=donor(&e,&mut s,Seat::P1);let a=if space {unit(&e,&mut s,Seat::P1,true)}else{let a=add(&e,&mut s,Seat::P1,"ASH_034");e.add_arena(&mut s,a).unwrap();a};let enemy=unit(&e,&mut s,Seat::P2,!space);
        // Ground uses actual Wicket's printed cannotAttackBase. The space
        // branch is a labeled synthetic base-restriction boundary probe.
        let base=s.player(Seat::P2).base;if space {s.card_mut(base).phase_restrictions|=RestrictionVerb::BeAttacked.bit();}s.source=d;
        assert!(!e.has_attack_target(&s,a,false));assert!(!e.support_targets(&s).contains(&a));
        // The pinned resolver checks an eligible attack before copying Support.
        // Add a same-arena pregrant defender, then copy before it departs.
        s.card_mut(base).phase_restrictions=0;
        let same=unit(&e,&mut s,Seat::P2,space);assert!(e.support_targets(&s).contains(&a));
        s.card_mut(a).exhausted=true;assert!(!e.support_targets(&s).contains(&a));s.card_mut(a).exhausted=false;
        s.card_mut(a).phase_restrictions|=RestrictionVerb::Attack.bit();assert!(!e.support_targets(&s).contains(&a));s.card_mut(a).phase_restrictions=0;
        e.begin_support_attack(&mut s,a).unwrap();e.defeat(&mut s,same).unwrap();
        let base=s.player(Seat::P2).base;s.card_mut(base).phase_restrictions|=RestrictionVerb::BeAttacked.bit();
        assert_eq!(e.attack_targets(&s,a,false),vec![enemy]);
        s.card_mut(enemy).phase_restrictions|=RestrictionVerb::BeAttacked.bit();assert!(e.attack_targets(&s,a,false).is_empty());
        s.card_mut(enemy).phase_keyword_presence|=1;assert_eq!(e.attack_targets(&s,a,false),vec![enemy]);
        e.defeat(&mut s,a).unwrap();assert_eq!(s.support.len,0);
    }
}
#[test]
fn cross_arena_support_completed_attack_and_reentry_clear_snapshot() {
    let e=engine();for space in [false,true] {
        let mut s=board(&e,Seat::P1);let d=donor(&e,&mut s,Seat::P1);let a=unit(&e,&mut s,Seat::P1,space);let enemy=unit(&e,&mut s,Seat::P2,!space);let base=s.player(Seat::P2).base;
        s.source=d;e.begin_support_attack(&mut s,a).unwrap();
        // An actual AttackFlow cleanup, rather than merely clearing a local flag.
        let done=e.apply(&s,&e.card_action(&s,base)).unwrap();
        assert_eq!(done.support.len,0);assert!(!done.support.pending);
        assert!(!e.attack_targets(&done,a,false).contains(&enemy));
        let mut s=board(&e,Seat::P1);let d=donor(&e,&mut s,Seat::P1);let a=unit(&e,&mut s,Seat::P1,space);let enemy=unit(&e,&mut s,Seat::P2,!space);
        s.source=d;e.begin_support_attack(&mut s,a).unwrap();e.defeat(&mut s,a).unwrap();
        assert_eq!(s.support.len,0);s.player_mut(Seat::P1).discard.remove(a);e.add_arena(&mut s,a).unwrap();
        assert!(!e.attack_targets(&s,a,false).contains(&enemy));
    }
}
#[test]
fn cross_arena_import_copied_program_is_exact_and_selected_pool_bound() {
    let e=engine();let mut s=board(&e,Seat::P1);let d=donor(&e,&mut s,Seat::P1);let a=unit(&e,&mut s,Seat::P1,false);
    s.source=d;e.begin_support_attack(&mut s,a).unwrap();
    assert!(e.load_state(&e.snapshot(&s)).unwrap_err().contains("pending Support"));
    // The pinned engine retains early effects on cancellation. The resulting
    // action window can import the exact grant, but never a foreign program.
    let cancel=e.legal_actions(&s).into_iter().find(|a|a.label=="Pass attack").unwrap();
    let s=e.apply(&s,&cancel).unwrap();assert_eq!(s.support.len,1);
    let raw=e.snapshot(&s);let imported=e.load_state(&raw).unwrap();
    assert!(e.cross_arena_permission(&imported,a,crate::cross_arena_contract::SPACE));
    let mut bad=raw;bad["supportGrants"][0]["sourcePrograms"]["side"]=json!("unit");
    assert!(e.load_state(&bad).unwrap_err().contains("source reference"));
}
#[test]
fn cross_arena_standalone_typed_headers_and_leaves_fail_closed() {
    use crate::ability_ir::{Ability,Program};
    let valid=json!({"trigger":"constant","title":"Arena access","costs":[],"effect":crate::cross_arena_contract::effect(crate::cross_arena_contract::SPACE)});
    let compile=|v:Value|->Result<Program,String>{let a:Ability=serde_json::from_value(v).map_err(|e|e.to_string())?;Program::compile(&a)};
    assert!(compile(valid.clone()).is_ok());
    for trigger in ["event","action","whenPlayed"] {let mut a=valid.clone();a["trigger"]=json!(trigger);assert!(compile(a).is_err());}
    for (key,value) in [("condition",json!({"kind":"const","value":true})),("optional",json!(true)),("requiresConfirmation",json!(true)),("costs",json!([{"kind":"exhaustSelf"}]))] {let mut a=valid.clone();a[key]=value;assert!(compile(a).is_err(),"{key}");}
    for (key,value) in [("target",json!("chosen")),("target",json!("parent")),("duration",json!("phase")),("permission",json!("canAttackAnything")),("unexpected",json!(true))] {let mut a=valid.clone();a["effect"][key]=value;assert!(compile(a).is_err(),"{key}");}
}

fn residual(e:&Engine,seat:Seat,space:bool)->(State,CardId,CardId) {
    let mut s=board(e,seat);let d=donor(e,&mut s,seat);let a=unit(e,&mut s,seat,space);
    s.source=d;e.begin_support_attack(&mut s,a).unwrap();
    let pass=e.legal_actions(&s).into_iter().find(|a|a.label=="Pass attack").unwrap();
    (e.apply(&s,&pass).unwrap(),d,a)
}
#[test]
fn cross_arena_source_wire_reference_roundtrip_and_departure_preserve_captured_set() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for space in [false,true] {
        let (mut s,d,a)=residual(&e,seat,space);
        let expected=json!({"kind":"sourcePrograms","target":format!("c{}",a-1),"donor":format!("c{}",d-1),
            "sourcePrograms":{"setCode":"ASH_037","side":"card"},"keywords":[0,0,0,0,0,0,0,0,0],"presence":0});
        assert_eq!(e.support_snapshot(&s),json!([expected]));
        let raw=e.snapshot(&s);let restored=e.load_state(&raw).unwrap();assert_eq!(e.snapshot(&restored),raw);
        assert_eq!(restored.zobrist(),s.zobrist());assert_eq!(e.legal_actions(&restored),e.legal_actions(&s));
        // Stable physical definition is valid even after the donor leaves play.
        e.defeat(&mut s,d).unwrap();assert_eq!(e.support_snapshot(&s),json!([expected]));
        let restored=e.load_state(&e.snapshot(&s)).unwrap();assert!(e.cross_arena_permission(&restored,a,crate::cross_arena_contract::SPACE));
        assert_eq!(e.support_snapshot(&restored),json!([expected]));
        // Later donor state cannot rewrite a captured keyword snapshot.
        s.card_mut(d).add_phase_keyword(6);s.card_mut(d).phase_keyword_values[0]=9;
        assert_eq!(e.support_snapshot(&s),json!([expected]));
        e.defeat(&mut s,a).unwrap();assert_eq!(e.support_snapshot(&s),json!([]));
        s.player_mut(seat).discard.remove(a);e.add_arena(&mut s,a).unwrap();
        assert!(!e.cross_arena_permission(&s,a,if space {crate::cross_arena_contract::GROUND}else{crate::cross_arena_contract::SPACE}));
    }}
}
#[test]
fn cross_arena_source_wire_strict_union_reference_identity_keywords_and_bounds() {
    let e=engine();let (s,d,a)=residual(&e,Seat::P1,false);let raw=e.snapshot(&s);
    let row=&raw["supportGrants"][0];
    for (key,value) in [("kind",json!("unknown")),("programs",json!([])),("unknown",json!(0)),("sourcePrograms",json!(0)),
        ("target",json!(format!("c{}",d-1))),("donor",json!(format!("c{}",a-1))),("donor",json!("c255")),
        ("keywords",json!([0,0])),("keywords",json!([0,0,0,0,0,0,0,0,-1])),("keywords",json!([0,0,0,0,0,0,65536,0,0])),
        ("keywords",json!([2,0,0,0,0,0,0,0,0])),("keywords",json!([1,0,0,0,0,0,0,0,0])),("presence",json!(512)),("presence",json!(-1)),("presence",json!(0.5))] {
        let mut bad=raw.clone();bad["supportGrants"][0][key]=value;assert!(e.load_state(&bad).is_err(),"{key}");
    }
    for key in ["kind","target","donor","sourcePrograms","keywords","presence"] {let mut bad=raw.clone();bad["supportGrants"][0].as_object_mut().unwrap().remove(key);assert!(e.load_state(&bad).is_err(),"missing {key}");}
    for (key,value) in [("setCode",json!("ASH_074")),("setCode",json!(0)),("side",json!("unit")),("programs",json!([0])),("index",json!(0)),("unknown",json!(true))] {
        let mut bad=raw.clone();bad["supportGrants"][0]["sourcePrograms"][key]=value;assert!(e.load_state(&bad).is_err(),"ref {key}");
    }
    for key in ["setCode","side"] {let mut bad=raw.clone();bad["supportGrants"][0]["sourcePrograms"].as_object_mut().unwrap().remove(key);assert!(e.load_state(&bad).is_err());}
    for pending in [json!(true),json!(false),Value::Null] {let mut bad=raw.clone();bad["supportAttackPending"]=pending;assert!(e.load_state(&bad).unwrap_err().contains("pending Support"));}
    let mut bad=raw.clone();bad["supportGrants"]=json!(vec![row.clone();MAX_SUPPORT_GRANTS+1]);assert!(e.load_state(&bad).is_err());
    let mut wrong=s;wrong.card_mut(d).owner=Seat::P2;wrong.support.len=0;assert!(e.load_support_state(&mut wrong,&raw).is_err());
    let mut absent=s;absent.card_mut(d).present=false;absent.support.len=0;assert!(e.load_support_state(&mut absent,&raw).is_err());
    let mut out=s;e.defeat(&mut out,a).unwrap();let mut bad=e.snapshot(&out);bad["supportGrants"]=json!([row]);assert!(e.load_state(&bad).is_err());
    // Numeric zero can have presence, while supported captured numeric values
    // remain bounded and independent of the donor's current keyword state.
    let mut valid=raw;valid["supportGrants"][0]["keywords"]=json!([0,0,0,0,0,0,2,0,0]);valid["supportGrants"][0]["presence"]=json!((1<<6)|(1<<7));
    let restored=e.load_state(&valid).unwrap();assert_eq!(e.snapshot(&restored),valid);
}
#[test]
fn cross_arena_source_wire_legacy_canonicalization_and_nonqualifying_rows_stay_legacy() {
    let e=engine();let (mut s,d,_)=residual(&e,Seat::P1,false);let canonical=e.snapshot(&s);
    let programs=s.support.grants[0].programs[..s.support.grants[0].len as usize].iter().map(|p|serde_json::to_value(e.program(*p)).unwrap()).collect::<Vec<_>>();
    let mut legacy=canonical.clone();let row=legacy["supportGrants"][0].as_object_mut().unwrap();row.remove("kind");row.remove("sourcePrograms");row.insert("programs".into(),json!(programs));
    // Qualifying historical representations normalize, not byte-roundtrip.
    let imported=e.load_state(&legacy).unwrap();assert_eq!(e.snapshot(&imported),canonical);
    // Exact captured-set matching includes cardinality, order and multiplicity.
    let original=s.support.grants[0];s.support.grants[0].len=0;assert!(e.support_snapshot(&s)[0].get("kind").is_none());
    s.support.grants[0]=original;s.support.grants[0].len=2;s.support.grants[0].programs[1]=original.programs[0];
    assert!(e.support_snapshot(&s)[0].get("kind").is_none());
    s.support.grants[0]=original;let other=e.catalogue.as_ref().unwrap().programs.iter().position(|p|p.trigger=="onAttack").unwrap() as u16;
    s.support.grants[0].len=2;s.support.grants[0].programs[1]=other;assert!(e.support_snapshot(&s)[0].get("kind").is_none());
    s.support.grants[0].len=1;s.support.grants[0].programs[0]=other;assert!(e.support_snapshot(&s)[0].get("kind").is_none());
    s.support.grants[0]=original;
    // An unrelated ordinary Support donor keeps its old empty program row.
    let plain=add(&e,&mut s,Seat::P1,"ASH_074");e.add_arena(&mut s,plain).unwrap();
    s.support.grants[1]=SupportGrant{donor:plain,target:original.target,..SupportGrant::default()};s.support.len=2;
    let raw=e.snapshot(&s);assert_eq!(raw["supportGrants"][1],json!({"target":format!("c{}",original.target-1),"donor":format!("c{}",plain-1),"programs":[],"keywords":[0,0,0,0,0,0,0,0,0],"presence":0}));
    assert_eq!(e.snapshot(&e.load_state(&raw).unwrap()),raw);
    // A native modified attack keeps its existing tagged program-free encoding
    // beside the new reference and an ordinary legacy serialized grant.
    let (code,modifiers)=e.definitions.iter().find_map(|definition|definition.programs.card.iter().find_map(|p|e.program(*p).instructions.iter().find_map(|i|match i {
        crate::ability_ir::Instruction::Apply(crate::ability_ir::Effect::InitiateAttack{properties,..})=>properties.modifiers().ok().filter(|m|m.power!=0||m.presence!=0||m.deals_combat_damage_first).map(|m|(definition.code.clone(),m)),_=>None
    }))).unwrap();
    let native=add(&e,&mut s,Seat::P1,&code);s.player_mut(Seat::P1).discard.push(native).unwrap();
    s.support.grants[2]=SupportGrant{donor:native,target:original.target,native_attack:true,power:modifiers.power,keywords:modifiers.keywords,presence:modifiers.presence,deals_combat_damage_first:modifiers.deals_combat_damage_first,..SupportGrant::default()};s.support.len=3;
    let raw=e.snapshot(&s);assert_eq!(raw["supportGrants"][2]["kind"],"modifiedAttack");assert!(raw["supportGrants"][2].get("sourcePrograms").is_none());
    assert_eq!(e.snapshot(&e.load_state(&raw).unwrap()),raw);
    assert_ne!(d,plain);
}
#[test]
fn cross_arena_source_wire_compiled_profile_rejects_metadata_and_program_mutants() {
    let e=engine();let (s,d,_)=residual(&e,Seat::P1,false);let c=e.catalogue.as_ref().unwrap();let card=c.metadata.get(&e.def(&s,d).code).unwrap();let p=e.program(s.support.grants[0].programs[0]);
    assert!(crate::cross_arena_contract::source_programs(card,&[p]));
    assert!(!crate::cross_arena_contract::source_programs(card,&[]));assert!(!crate::cross_arena_contract::source_programs(card,&[p,p]));
    for (key,value) in [("types",json!(["leader","unit"])),("cost",json!(8)),("traits",json!([])),("aspects",json!([])),("keywords",json!([]))] {let mut card=card.clone();card[key]=value;assert!(!crate::cross_arena_contract::source_programs(&card,&[p]),"{key}");}
    for mutant in 0..4 {let mut p=p.clone();match mutant {0=>p.trigger="whenPlayed".into(),1=>p.coordinate=true,2=>p.optional=true,_=>p.instructions.clear()};assert!(!crate::cross_arena_contract::source_programs(card,&[&p]),"{mutant}");}
}
#[test]
fn cross_arena_source_wire_import_rechecks_actual_definition_and_whole_program_binding() {
    let mut e=engine();let (s,d,_)=residual(&e,Seat::P1,false);let raw=e.snapshot(&s);
    let index=s.card(d).def as usize;let original=e.definitions[index].programs.clone();let p=original.card[0];
    for mutant in 0..4 {
        e.definitions[index].programs=original.clone();
        match mutant {0=>e.definitions[index].programs.card.clear(),1=>e.definitions[index].programs.card.push(p),2=>e.definitions[index].programs.unit.push(p),_=>e.definitions[index].programs.leader.push(p)};
        let mut candidate=s;candidate.support.len=0;assert!(e.load_support_state(&mut candidate,&raw).is_err(),"program set {mutant}");
    }
    e.definitions[index].programs=original;
    let code=e.definitions[index].code.clone();let metadata=e.catalogue.as_ref().unwrap().metadata[&code].clone();
    e.catalogue.as_mut().unwrap().metadata.get_mut(&code).unwrap()["keywords"]=json!([]);
    let mut candidate=s;candidate.support.len=0;assert!(e.load_support_state(&mut candidate,&raw).is_err());
    *e.catalogue.as_mut().unwrap().metadata.get_mut(&code).unwrap()=metadata;
    // Matching code text alone cannot substitute a different physical donor.
    let mut candidate=s;let other=add(&e,&mut candidate,Seat::P1,"ASH_074");e.add_arena(&mut candidate,other).unwrap();candidate.support.len=0;
    let mut wrong=raw;wrong["supportGrants"][0]["donor"]=json!(format!("c{}",other-1));wrong["supportGrants"][0]["sourcePrograms"]["setCode"]=json!("ASH_074");
    assert!(e.load_support_state(&mut candidate,&wrong).is_err());
}
#[test]
fn cross_arena_paid_support_nested_menu_preserves_commit_and_optional_pass() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let mut s=board(&e,seat);
        let attacker=unit(&e,&mut s,seat,true);let defender=unit(&e,&mut s,seat.other(),false);
        let donor=add(&e,&mut s,seat,"ASH_037");s.player_mut(seat).hand.push(donor).unwrap();
        for _ in 0..16 {let resource=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(resource).unwrap();}
        let s=e.load_state(&e.snapshot(&s)).unwrap();
        // A normal attack has not paid an outer ability: its defender picker
        // retains the actual rollback Cancel boundary.
        let normal=e.apply(&s,&e.card_action(&s,attacker)).unwrap();
        assert_eq!(normal.decision,Decision::AttackTarget);
        assert_eq!(e.legal_actions(&normal),vec![Action::button("Cancel","cancel"),e.card_action(&normal,s.player(seat.other()).base)]);
        let cancelled=e.apply(&normal,&Action::button("Cancel","cancel")).unwrap();
        assert_eq!(e.snapshot(&cancelled),e.snapshot(&s));
        let support=e.apply(&s,&e.card_action(&s,donor)).unwrap();assert_eq!(support.decision,Decision::Support);
        assert_eq!(e.legal_actions(&support),vec![Action::button("Pass","passAbility"),e.card_action(&support,attacker)]);
        assert!(!support.player(seat).hand.contains(donor));assert!(support.card(donor).exhausted);
        let paid=support.player(seat).resources.as_slice().iter().filter(|id|support.card(**id).exhausted).count();assert!(paid>=7);
        let choosing=e.apply(&support,&e.card_action(&support,attacker)).unwrap();
        // Exact pinned menu: Pass attack plus ground defender and base. No
        // rollback Cancel belongs to this nested already-committed resolver.
        assert_eq!(e.legal_actions(&choosing),vec![Action::button("Pass attack","passAbility"),e.card_action(&choosing,defender),e.card_action(&choosing,s.player(seat.other()).base)]);
        assert!(e.apply(&choosing,&Action::button("Cancel","cancel")).is_err());
        let passed=e.apply(&choosing,&Action::button("Pass attack","passAbility")).unwrap();
        assert!(!passed.support.pending);assert_eq!(passed.support.len,1);assert!(!passed.card(attacker).exhausted);
        assert!(!passed.player(seat).hand.contains(donor));assert!(passed.card(donor).exhausted);
        assert_eq!(passed.player(seat).resources.as_slice().iter().filter(|id|passed.card(**id).exhausted).count(),paid);
        assert!(e.cross_arena_permission(&passed,attacker,crate::cross_arena_contract::GROUND));
        let wire=e.snapshot(&passed);assert_eq!(e.snapshot(&e.load_state(&wire).unwrap()),wire);
    }}
}
