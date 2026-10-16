use super::*;

fn live_board(e:&Engine,seat:Seat,code:&str)->(State,CardId) {
    let mut s=board(e);s.round=6;s.active=seat;s.action_player=Some(seat);
    let source=unit(e,&mut s,seat,code);
    let snapshot=e.snapshot(&s);let restored=e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored),snapshot);
    (restored,source)
}

fn attack(e:&Engine,s:State,source:CardId,target:CardId)->State {
    let a=e.card_action(&s,source);assert!(e.legal_actions(&s).contains(&a));let prompt=e.apply(&s,&a).unwrap();
    assert_eq!(prompt.decision,Decision::AttackTarget);let a=e.card_action(&prompt,target);assert!(e.legal_actions(&prompt).contains(&a));e.apply(&prompt,&a).unwrap()
}

#[test]
fn attack_end_complete_admission_and_backend_metadata() {
    let e=engine();
    for code in ["LAW_033","LOF_038","SHD_059","SOR_146"] {
        let d=&e.definitions[*e.by_code.get(code).unwrap_or_else(||panic!("{code}: {:?}",e.rejected_cards().unwrap().get(code))) as usize];
        assert_eq!(d.programs.card.len(),1);let p=e.program(d.programs.card[0]);
        assert_eq!(p.trigger,"whenAttackEnds");assert!(p.attacker_must_survive);assert!(p.executable.is_ok(),"{code}: {:?}",p.executable);
    }
    for code in ["HMW_067","LAW_054","LAW_074","LAW_215","LOF_156","SEC_096","SEC_107","SEC_174","SOR_192","TWI_053"] {assert!(!e.by_code.contains_key(code),"unsupported child contract admitted {code}");}
    assert!(e.by_code.contains_key("LAW_034"));
    let codes:Vec<_>=e.definitions.iter().filter(|d|d.programs.card.iter().any(|p|e.program(*p).trigger=="whenAttackEnds")).map(|d|d.code.as_str()).collect();for code in ["LAW_033","LAW_034","LOF_038","SHD_059","SOR_146"] {assert!(codes.contains(&code));}eprintln!("attack-end cards={codes:?}");
    eprintln!("phase2 attack-end admitted={} State={} ResolutionContext={}",e.definitions.len(),std::mem::size_of::<State>(),std::mem::size_of::<ResolutionContext>());
}

#[test]
fn attack_end_legal_midgame_optional_and_live_strict_boundaries_both_backends() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {for code in ["LAW_033","LOF_038"] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,seat,code);
        let low=unit(&e,&mut s,seat.other(),"LOF_256");let equal=unit(&e,&mut s,seat.other(),"LOF_256");
        if code=="LOF_038" {s.card_mut(low).damage=3;s.card_mut(equal).damage=2;}else{s.card_mut(equal).phase_power=4-e.power(&s,equal,false) as i32;}
        let base=s.player(seat.other()).base;let power=e.power(&s,source,false);
        let prompt=attack(&e,s,source,base);
        assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.card(base).damage,power);
        assert_eq!(prompt.attack_source,source);assert!(prompt.attack_end_captured);assert!(!prompt.attack_attacker_defeated);
        let actions=e.legal_actions(&prompt);assert!(actions.iter().any(|a|a.id()==Some(low)));assert!(!actions.iter().any(|a|a.id()==Some(equal)));assert!(!actions.iter().any(|a|a.id()==Some(base)));
        let declined=named_button(&e,&prompt,"Pass");assert!(e.in_arena(&declined,low));assert_eq!(declined.attack_source,0);assert!(!declined.attack_end_captured);assert!(!declined.attack_attacker_defeated);
        let done=e.apply(&prompt,&e.card_action(&prompt,low)).unwrap();assert!(done.player(seat.other()).discard.contains(low));assert!(e.in_arena(&done,equal));assert_eq!(done.attack_source,0);assert_eq!(done.death_depth,0);
    }}}
}

#[test]
fn attack_end_combat_defeat_suppresses_survival_trigger() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for code in ["LAW_033","LOF_038"] {
        let (mut s,source)=live_board(&e,seat,code);s.card_mut(source).damage=e.hp(&s,source)-1;
        let defender=unit(&e,&mut s,seat.other(),if code=="LAW_033" {"TOKEN_XWING"}else{"LOF_256"});let other=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(other).damage=1;
        let done=attack(&e,s,source,defender);assert!(done.player(seat).discard.contains(source));assert_eq!(done.decision,Decision::Window);assert!(e.in_arena(&done,other));assert_eq!(done.attack_source,0);
    }}
}

// Internal continuation diagnostic: same physical card is defeated, returned,
// then re-enters before its already-captured end trigger resolves. A live-zone
// check would incorrectly make the survival condition true again.
#[test]
fn attack_end_defeat_identity_latch_survives_nested_context_and_reentry() {
    let e=engine();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");let low=unit(&e,&mut s,Seat::P2,"LOF_256");
    s.source=source;s.attack_source=source;s.attack_defender=s.players[1].base;
    e.capture_attack_end(&mut s).unwrap();let captured=s.death_pending;
    let before=s.zobrist();s.death_depth=1;s.death_frames[0].context=ResolutionContext::capture(&s);
    e.defeat(&mut s,source).unwrap();assert!(s.attack_attacker_defeated);assert!(s.death_frames[0].context.attack_attacker_defeated);
    s.player_mut(Seat::P1).discard.remove(source);e.add_arena(&mut s,source).unwrap();assert!(e.in_arena(&s,source));
    let context=s.death_frames[0].context;s.death_depth=0;s.death_frames[0]=DeathFrame::default();context.restore(&mut s);
    assert_ne!(s.zobrist(),before);assert!(!e.death_has_effect(&s,captured.entries[0]));
    let program=captured.entries[0].program;e.begin_program(&mut s,program,1).unwrap();assert!(e.in_arena(&s,low));assert_eq!(s.decision,Decision::Window);
}

// Internal incarnation boundary: a non-defeat departure does not count as a
// defeat of the original in-play identity when a later incarnation is defeated.
#[test]
fn attack_end_nondefeat_departure_does_not_rebind_survival_identity() {
    let e=engine();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");let low=unit(&e,&mut s,Seat::P2,"LOF_256");
    s.source=source;s.attack_source=source;s.attack_defender=s.players[1].base;
    e.capture_attack_end(&mut s).unwrap();let captured=s.death_pending.entries[0];
    e.leave_play(&mut s,source).unwrap();assert!(s.attack_attacker_departed);assert!(!s.attack_attacker_defeated);
    s.player_mut(Seat::P1).discard.remove(source);e.add_arena(&mut s,source).unwrap();e.defeat(&mut s,source).unwrap();
    assert!(!s.attack_attacker_defeated,"defeat belongs to the new incarnation");
    s.player_mut(Seat::P1).discard.remove(source);e.add_arena(&mut s,source).unwrap();assert!(e.death_has_effect(&s,captured));assert!(e.in_arena(&s,low));
    s.card_mut(source).phase_keyword_values[0]=2;s.card_mut(source).add_phase_keyword(6);assert!(!e.is_attacking(&s,source));assert_eq!(e.power(&s,source,true),4,"the later incarnation must not inherit Raid from the old active attack");
}

#[test]
fn attack_end_retains_raid_until_target_selection_and_then_cleans_up() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");
        s.card_mut(source).phase_keyword_values[0]=2;s.card_mut(source).add_phase_keyword(6);
        let target=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(target).phase_power=4;assert_eq!(e.power(&s,target,false),5);
        let base=s.players[1].base;let prompt=attack(&e,s,source,base);assert_eq!(prompt.card(base).damage,6);assert_eq!(e.power(&prompt,source,true),6);
        assert!(e.program_menu_title(&prompt,source,e.card_programs(&prompt,source)[0]).contains("6"));assert!(e.legal_actions(&prompt).iter().any(|a|a.id()==Some(target)));
        let done=e.apply(&prompt,&e.card_action(&prompt,target)).unwrap();assert!(!e.in_arena(&done,target));assert_eq!(e.power(&done,source,false),4);assert_eq!(done.attack_source,0);
    }
}

#[test]
fn attack_end_combat_and_defeat_share_player_choice_and_live_survival_gate() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,seat,"LAW_033");
        let defender=unit(&e,&mut s,seat.other(),"JTL_104");s.card_mut(defender).damage=2;
        let victim=unit(&e,&mut s,seat.other(),"LOF_256");
        s.source=source;e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("self"),amount:Expr::number(1)}).unwrap();s.source=0;
        let window=attack(&e,s,source,defender);assert_eq!(window.decision,Decision::DeathPlayer);assert!(e.in_arena(&window,source));assert!(window.player(seat.other()).discard.contains(defender));assert_eq!(window.death_window.len,2);
        let first=named_button(&e,&window,"You");assert_eq!(first.source,source);assert_eq!(first.decision,Decision::AbilityTarget);
        let second=e.apply(&first,&e.card_action(&first,victim)).unwrap();assert!(second.player(seat.other()).discard.contains(victim));assert_eq!(second.source,defender);
        let done=e.apply(&second,&e.card_action(&second,source)).unwrap();assert!(done.player(seat).discard.contains(source));assert_eq!(done.decision,Decision::Window);assert_eq!(done.death_depth,0);
        let opponent=named_button(&e,&window,"Opponent");assert_eq!(opponent.source,defender);
        let done=e.apply(&opponent,&e.card_action(&opponent,source)).unwrap();assert!(done.player(seat).discard.contains(source));assert!(e.in_arena(&done,victim));assert_eq!(done.decision,Decision::Window);assert_eq!(done.death_depth,0);assert_eq!(done.attack_source,0);
    }}
}

#[test]
fn attack_end_optional_survival_false_captures_before_simultaneous_source_death() {
    let mut e=engine();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");
    let value=json!({"trigger":"whenAttackEnds","title":"Ungated end diagnostic","attackerMustSurvive":false,"optional":true,"targetResolver":{"owner":"opponent","type":"unit","zone":"anyAttackable"},"effect":{"kind":"defeat","target":"chosen"}});
    let program=death_program(&mut e,&s,source,value);e.definitions[s.card(source).def as usize].programs.card=vec![program];assert!(!e.program(program).attacker_must_survive);
    let defender=unit(&e,&mut s,Seat::P2,"TOKEN_XWING");let victim=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(source).damage=7;
    let prompt=attack(&e,s,source,defender);assert!(prompt.player(Seat::P1).discard.contains(source));assert_eq!(prompt.decision,Decision::AbilityTarget);assert_eq!(prompt.source,source);
    let done=e.apply(&prompt,&e.card_action(&prompt,victim)).unwrap();assert!(done.player(Seat::P2).discard.contains(victim));assert_eq!(done.decision,Decision::Window);
}

#[test]
fn attack_end_original_defender_combat_defeat_enables_zeb_and_embo() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {for code in ["SOR_146","SHD_059"] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,seat,code);
        let defender=unit(&e,&mut s,seat.other(),"LOF_254");let other=unit(&e,&mut s,seat.other(),"LOF_256");let ally=unit(&e,&mut s,seat,"LOF_256");s.card_mut(ally).damage=3;
        let prompt=attack(&e,s,source,defender);assert!(prompt.player(seat.other()).discard.contains(defender));assert!(prompt.attack_defender_defeated);
        let done=if code=="SOR_146" {assert_eq!(prompt.decision,Decision::AbilityTarget);e.apply(&prompt,&e.card_action(&prompt,other)).unwrap()}else{
            assert_eq!(prompt.decision,Decision::AbilityDistribute);
            let action=e.legal_actions(&prompt).into_iter().find(|a|a.arg==Some(json!([[format!("c{}",ally-1),2]]))).unwrap();e.apply(&prompt,&action).unwrap()
        };
        if code=="SOR_146" {assert!(done.player(seat.other()).discard.contains(other));}else{assert_eq!(done.card(ally).damage,1);assert!(e.in_arena(&done,other));}
        assert_eq!(done.decision,Decision::Window);assert_eq!(done.attack_source,0);assert!(!done.attack_defender_defeated);assert!(!done.attack_defender_departed);assert_eq!(done.death_depth,0);
    }}}
}

#[test]
fn attack_end_surviving_defender_and_base_do_not_qualify_history() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for code in ["SOR_146","SHD_059"] {for base_attack in [false,true] {
        let (mut s,source)=live_board(&e,seat,code);let defender=unit(&e,&mut s,seat.other(),"LOF_256");let ally=unit(&e,&mut s,seat,"LOF_256");s.card_mut(ally).damage=3;
        s.source=defender;e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("self"),amount:Expr::number(1)}).unwrap();s.source=0;
        let target=if base_attack{s.player(seat.other()).base}else{defender};let done=attack(&e,s,source,target);
        assert_eq!(done.decision,Decision::Window);assert!(e.in_arena(&done,defender));assert_eq!(done.card(ally).damage,3);assert_eq!(done.card(defender).damage,0);assert_eq!(done.attack_source,0);
    }}}
}

// Internal incarnation diagnostic; current arena membership and later defeats
// of the same physical card must not replace the attack's original in-play ID.
#[test]
fn attack_end_history_retains_original_defender_through_departure_and_nested_context() {
    let e=engine();for original_defeat in [false,true] {
        let (mut s,source)=live_board(&e,Seat::P1,"SOR_146");let defender=unit(&e,&mut s,Seat::P2,"LOF_256");s.source=source;s.attack_source=source;s.attack_defender=defender;
        let p=e.card_programs(&s,source)[0];let condition=e.program(p).instructions.iter().find_map(|i|if let Instruction::Conditional{condition,..}=i {Some(condition.clone())}else{None}).unwrap();
        assert!(!e.eval(&s,source,0,&condition).truth());let hash=s.zobrist();s.death_frames[0].context=ResolutionContext::capture(&s);s.death_depth=1;
        if original_defeat {e.defeat(&mut s,defender).unwrap();}else{e.leave_play(&mut s,defender).unwrap();}
        assert!(s.attack_defender_departed);assert_eq!(s.attack_defender_defeated,original_defeat);assert_eq!(s.death_frames[0].context.attack_defender_defeated,original_defeat);
        s.player_mut(Seat::P2).discard.remove(defender);e.add_arena(&mut s,defender).unwrap();e.defeat(&mut s,defender).unwrap();assert_eq!(s.attack_defender_defeated,original_defeat);
        s.player_mut(Seat::P2).discard.remove(defender);e.add_arena(&mut s,defender).unwrap();assert!(!e.is_defending(&s,defender));
        let c=s.death_frames[0].context;s.death_depth=0;s.death_frames[0]=DeathFrame::default();c.restore(&mut s);assert_eq!(e.eval(&s,source,0,&condition).truth(),original_defeat);assert_ne!(hash,s.zobrist());
    }
}

#[test]
fn attack_end_history_requires_exact_watcher_and_lexical_attack_contract() {
    let catalog:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    for mode in ["binding","scope","reset","extra","root","role","optional","helper","argumentIndex","registrationCondition"] {
        let mut value=catalog.clone();let entry=value["cards"].as_array_mut().unwrap().iter_mut().find(|c|c["setCode"]=="SOR_146").unwrap();
        match mode {
            "binding"=>entry["stateWatchers"][0]["binding"]=json!("different"),
            "scope"=>entry["stateWatchers"][0]["sharedScope"]=json!("controller"),
            "reset"=>entry["stateWatchers"][0]["resetEvent"]=json!("onRoundEnded"),
            "extra"=>{let extra=entry["stateWatchers"][0].clone();entry["stateWatchers"].as_array_mut().unwrap().push(extra);},
            "root"=>entry["abilities"]["card"][0]["effect"]["condition"]["attack"]["root"]=json!(["parentContext"]),
            "role"=>entry["abilities"]["card"][0]["effect"]["condition"]["attack"]["contextBinding"]["role"]=json!("targetResolver"),
            "optional"=>entry["abilities"]["card"][0]["effect"]["condition"]["attack"]["steps"][0]["optional"]=json!(true),
            "argumentIndex"=>entry["abilities"]["card"][0]["effect"]["condition"]["attack"]["contextBinding"]["id"]=json!("callback:1690:1"),
            "helper"=>entry["abilities"]["card"][0]["effect"]["condition"]["test"]=json!("allDefendersWereDefeated"),
            _=>entry["abilities"]["card"][0]["condition"]=json!({"kind":"const","value":true}),
        }
        let e=Engine::from_catalogue_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();assert!(!e.by_code.contains_key("SOR_146"),"{mode}");
    }
}

#[test]
fn attack_end_chewbacca_resolves_complete_group_and_suppresses_departed_source() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {for seat in [Seat::P1,Seat::P2] {for dies in [false,true] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,seat,"LAW_034");s.card_mut(source).damage=if dies{3}else{2};
        let defender=unit(&e,&mut s,seat.other(),"LOF_254");let other=unit(&e,&mut s,seat.other(),"LOF_256");let done=attack(&e,s,source,defender);
        assert!(done.player(seat.other()).discard.contains(defender));assert!(e.in_arena(&done,other));assert_eq!(done.decision,Decision::Window);assert_eq!(done.attack_source,0);
        if dies {assert!(done.player(seat).discard.contains(source));assert_eq!(e.token_count(&done,done.card(source).experience),0);}else{assert_eq!(e.token_count(&done,done.card(source).experience),1);assert_eq!(done.card(source).damage,0);assert_eq!(e.hp(&done,source),5);assert_eq!(e.power(&done,source,false),5);}
    }}}
}

#[test]
fn attack_end_generated_dispatch_matches_interpreter_exactly() {
    for code in ["LAW_033","LOF_038","SHD_059","SOR_146"] {
        let mut e=engine();e.set_rules_instrumentation(true);let (mut s,source)=live_board(&e,Seat::P1,code);
        let low=unit(&e,&mut s,Seat::P2,"LOF_256");s.card_mut(low).damage=3;
        let defender=unit(&e,&mut s,Seat::P2,"LOF_254");let ally=unit(&e,&mut s,Seat::P1,"LOF_256");s.card_mut(ally).damage=3;
        let target=if code=="LAW_033"||code=="LOF_038"{s.players[1].base}else{defender};
        let snapshot=e.snapshot(&s);s=e.load_state(&snapshot).unwrap();assert_eq!(e.snapshot(&s),snapshot);
        let program=e.card_programs(&s,source)[0] as usize;let aot=attack(&e,s,source,target);
        let hits=e.rules_hits.lock().unwrap()[&program];assert!(hits[0]>0,"{code}");assert_eq!(hits[1],0);
        let action=if code=="SHD_059" {e.legal_actions(&aot).into_iter().find(|a|a.arg==Some(json!([[format!("c{}",ally-1),2]]))).unwrap()}else{e.card_action(&aot,low)};
        let actual=e.apply(&aot,&action).unwrap();e.set_rules_backend(RulesBackend::Interpreter).unwrap();let interpreted=attack(&e,s,source,target);
        assert_eq!(aot,interpreted,"{code}");assert_eq!(e.legal_actions(&aot),e.legal_actions(&interpreted));let expected=e.apply(&interpreted,&action).unwrap();assert_eq!(actual,expected,"{code}");assert_eq!(actual.zobrist(),expected.zobrist());
    }
}

#[test]
fn attack_end_terminal_latches_pending_context_without_external_actions() {
    let e=engine();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");let _=unit(&e,&mut s,Seat::P2,"LOF_256");let base=s.players[1].base;s.card_mut(base).damage=29;
    let done=attack(&e,s,source,base);assert!(done.is_terminal());assert_eq!(done.outcome,1);assert!(e.legal_actions(&done).is_empty());assert_eq!(done.decision,Decision::AbilityTarget);assert_eq!(done.attack_source,source);assert_eq!(done.death_depth,1);
}

#[test]
fn attack_end_capture_capacity_and_context_hash_are_bounded() {
    let e=engine();let (mut s,source)=live_board(&e,Seat::P1,"LAW_033");s.source=source;s.attack_source=source;s.attack_defender=s.players[1].base;
    s.death_pending.len=MAX_DEATH_TRIGGERS as u8;let before=s;assert!(e.capture_attack_end(&mut s).unwrap_err().contains("capacity"));assert_eq!(s,before);
    s.death_pending=DeathWindow::default();let hash=s.zobrist();let c=ResolutionContext::capture(&s);s.attack_defender_departed=true;assert_ne!(s.zobrist(),hash);c.restore(&mut s);assert_eq!(s.zobrist(),hash);
    s.death_depth=1;s.death_frames[0].context=c;let hash=s.zobrist();s.death_frames[0].context.attack_attacker_departed=true;assert_ne!(s.zobrist(),hash);
}

#[test]
fn attack_end_embo_distribution_keeps_native_undamaged_targets_and_zero_choice() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();let (mut s,source)=live_board(&e,Seat::P1,"SHD_059");let defender=unit(&e,&mut s,Seat::P2,"LOF_254");
        s.source=source;e.apply_effect(&mut s,&Effect::GiveShield{target:Target::name("self"),amount:Expr::number(1)}).unwrap();s.source=0;
        let prompt=attack(&e,s,source,defender);assert_eq!(prompt.card(source).damage,0);assert_eq!(prompt.decision,Decision::AbilityDistribute);
        for arg in [json!([]),json!([[format!("c{}",source-1),1]]),json!([[format!("c{}",source-1),2]])] {
            let action=e.legal_actions(&prompt).into_iter().find(|a|a.arg.as_ref()==Some(&arg)).unwrap();let done=e.apply(&prompt,&action).unwrap();assert_eq!(done.card(source).damage,0);assert_eq!(done.decision,Decision::Window);assert_eq!(done.attack_source,0);
        }
    }
}
