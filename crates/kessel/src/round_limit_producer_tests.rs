use super::*;
use crate::ability_ir::Effect;
use crate::round_limit_contract::Registration;
fn engine(backend:RulesBackend)->Engine {let v:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();let mut e=Engine::from_catalogue_value(&v).unwrap();e.set_rules_backend(backend).unwrap();e}
fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;*s.card_mut(id)=Card{present:true,def:*e.by_code.get(code).unwrap_or_else(||panic!("{code}: {:?}",e.rejected_cards().unwrap().get(code))),owner:seat,..Card::default()};id}
fn unit(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=add(e,s,seat,code);e.add_arena(s,id).unwrap();id}
fn board(e:&Engine,seat:Seat)->State {let mut s=State::default();s.active=seat;s.action_player=Some(seat);s.hidden_history_exact=true;s.credit_state_exact=true;s.force_state_exact=true;for p in [seat,seat.other()] {s.player_mut(p).base=add(e,&mut s,p,"SOR_023");s.player_mut(p).leader=add(e,&mut s,p,"SOR_014");s.player_mut(p).force_id=s.next_id;s.next_id+=1;for _ in 0..24 {let id=add(e,&mut s,p,"LOF_254");s.player_mut(p).resources.push(id).unwrap();}for _ in 0..6 {let id=add(e,&mut s,p,"LOF_254");s.player_mut(p).deck.push(id).unwrap();}}s}
fn arg(e:&Engine,s:&State,arg:&str)->State {let actions=e.legal_actions(s);let a=actions.iter().find(|a|a.arg.as_ref().and_then(Value::as_str)==Some(arg)).unwrap_or_else(||panic!("{arg} at {:?}: {actions:?}",s.decision));e.apply(s,a).unwrap_or_else(|error|panic!("arg {arg} source={} program={} round={:?}: {error}",s.source,s.ability_program,s.round_limit))}
fn choose(e:&Engine,s:&State,id:CardId)->State {let a=e.card_action(s,id);assert!(e.legal_actions(s).contains(&a),"{:?}: {:?}",s.decision,e.legal_actions(s));e.apply(s,&a).unwrap()}
fn used(s:&State,source:CardId,r:Registration,seat:Seat)->bool {s.round_limits.current(source,r.ordinal()).is_some_and(|h|!s.round_limits.available(h,seat.index() as u8).unwrap())}
fn damage()->Effect {serde_json::from_value(json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":3}})).unwrap()}
fn open(e:&Engine,s:&mut State) {assert!(e.open_deaths(s,DeathContinuation::Finish).unwrap());}
#[test]
fn round_limit_native_keeper_choose_none_commits_and_declined_gar_does_not() {for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
    let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");let victim=unit(&e,&mut s,seat,"LOF_164");let source=unit(&e,&mut s,seat.other(),"SOR_095");s.source=source;s.ability_target=victim;e.apply_effect(&mut s,&damage()).unwrap();open(&e,&mut s);
    assert_eq!(s.decision,Decision::AbilityTarget);let actions=e.legal_actions(&s);assert_eq!(actions.len(),3);assert_eq!(actions[0],Action::button("Choose nothing","done"));assert!(actions.contains(&e.card_action(&s,s.player(seat).base)));assert!(actions.contains(&e.card_action(&s,s.player(seat.other()).base)));
    s=arg(&e,&s,"done");assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));assert_eq!(s.card(s.player(seat).base).damage,0);assert_eq!(s.card(s.player(seat.other()).base).damage,0);
    let mut s=board(&e,seat);let gar=unit(&e,&mut s,seat,"ASH_047");let upgrade=add(&e,&mut s,seat,"IC27_052");s.player_mut(seat).hand.push(upgrade).unwrap();s.source=upgrade;assert!(e.play_upgrade_with_payment(&mut s,upgrade,gar).unwrap());
    assert_eq!(s.decision,Decision::AbilityOptional);assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),["Trigger","Pass"]);s=arg(&e,&s,"pass");assert!(!used(&s,gar,Registration::PlayedUpgradeOnSelf,seat));
    s.active=seat;s.action_player=Some(seat);let upgrade2=add(&e,&mut s,seat,"IC27_052");s.player_mut(seat).hand.push(upgrade2).unwrap();s.source=upgrade2;assert!(e.play_upgrade_with_payment(&mut s,upgrade2,gar).unwrap());s=arg(&e,&s,"trigger");assert!(used(&s,gar,Registration::PlayedUpgradeOnSelf,seat));assert_eq!(e.arena_units(&s).iter().filter(|id|e.def(&s,**id).code=="TOKEN_MANDALORIAN").count(),1);
}}}
#[test]
fn round_limit_native_militia_declaration_and_zero_damage_are_separate() {for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
    let mut s=board(&e,seat);let militia=unit(&e,&mut s,seat,"ASH_160");s.card_mut(militia).exhausted=true;let attacker=unit(&e,&mut s,seat.other(),"SOR_095");s.active=seat.other();s.action_player=Some(seat.other());s=choose(&e,&s,attacker);assert_eq!(s.decision,Decision::AttackTarget);let base=s.player(seat).base;s=choose(&e,&s,base);assert!(!s.card(militia).exhausted);assert!(used(&s,militia,Registration::EnemyGroundAttackAtBase,seat));
    for zero_attacker in [true,false] {let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");unit(&e,&mut s,seat,"SHD_037");let marine=unit(&e,&mut s,seat,"SOR_095");let zero=unit(&e,&mut s,seat.other(),"IBH_063");assert_eq!(e.power(&s,zero,true),0);let (attacker,target,actor)=if zero_attacker {(zero,marine,seat.other())}else{(marine,zero,seat)};s.active=actor;s.action_player=Some(actor);s=choose(&e,&s,attacker);s=choose(&e,&s,target);assert_eq!(s.card(marine).damage,0);assert!(!used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));assert_eq!(s.card(s.player(seat).base).damage,0);assert!(!e.in_arena(&s,zero));}
}}}
#[test]
fn round_limit_native_nuvo_lki_and_kallus_captured_source_death() {for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
    let mut s=board(&e,seat);let nuvo=unit(&e,&mut s,seat,"HMW_062");let recipient=unit(&e,&mut s,seat,"SOR_095");let victim=unit(&e,&mut s,seat.other(),"LOF_164");s.source=victim;let weakness:Effect=serde_json::from_value(json!({"kind":"giveWeakness","target":"self","amount":{"kind":"const","value":1}})).unwrap();e.apply_effect(&mut s,&weakness).unwrap();e.defeat(&mut s,victim).unwrap();open(&e,&mut s);assert_eq!(s.decision,Decision::AbilityTarget);assert!(e.legal_actions(&s).iter().any(|a|a.arg.as_ref().and_then(Value::as_str)==Some("passAbility")));s=choose(&e,&s,recipient);assert!(e.attached_native_token(&s,recipient,"weakness"));assert!(used(&s,nuvo,Registration::EnemyWeaknessUnitDefeated,seat));
    let mut s=board(&e,seat);let kallus=unit(&e,&mut s,seat,"SOR_115");let other=unit(&e,&mut s,seat.other(),"ASH_036");let cards=s.player(seat).deck.len;e.defeat_many(&mut s,&[kallus,other]).unwrap();assert!(!e.in_arena(&s,kallus));open(&e,&mut s);assert_eq!(s.decision,Decision::AbilityOptional);s=arg(&e,&s,"trigger");assert_eq!(s.player(seat).deck.len,cards-1);let roots=crate::round_limit_runtime::roots(&s);assert!(roots.is_empty());let wire=e.snapshot(&s);assert!(wire["roundLimitState"]["registrations"].as_array().unwrap().iter().any(|r|r["card"]==format!("c{}",kallus-1)&&r["used"]==json!([0,0])),"offboard current clone is fresh; consumed captured old object is retired");
}}}

#[test]
fn round_limit_native_will_defeat_is_generation_latch_and_missing_fact_is_atomic() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");let victim=unit(&e,&mut s,seat,"LOF_164");let source=add(&e,&mut s,seat.other(),"TWI_174");s.source=source;s.ability_target=victim;
        let catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
        let raw=catalogue["cards"].as_array().unwrap().iter().find(|v|v["setCode"]=="TWI_174").unwrap();
        let effect:Effect=serde_json::from_value(raw["abilities"]["card"][0]["effect"].clone()).unwrap();
        s.event_group.context=Selection::one(victim);assert!(e.generate_group(&mut s,&effect).unwrap());assert_eq!(s.event_group.count,1);assert!(!s.event_group.damage_facts.get(0).unwrap());
        let mut missing=s;missing.event_group.damage_facts=Default::default();let before=missing;
        assert!(e.resolve_group(&mut missing,&effect).unwrap_err().contains("willDefeat is missing"));assert_eq!(missing,before);
        // Diagnostic interleaving: a generation-time false remains false even
        // if remaining HP changes before handlers. No actual engine oracle is
        // claimed for this staged divergence; the66 completed source games do
        // not establish this interleaving as reachable.
        s.card_mut(victim).damage=e.hp(&s,victim)-1;
        assert!(e.resolve_group(&mut s,&effect).unwrap());assert!(!e.in_arena(&s,victim));s.event_group=EventGroupState::default();open(&e,&mut s);
        assert_eq!(s.decision,Decision::AbilityTarget);s=arg(&e,&s,"done");assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));
    }}
}
#[test]
fn round_limit_native_wire_is_complete_offboard_and_selected_proof_is_required() {
    let e=engine(RulesBackend::Interpreter);
    for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let gar=unit(&e,&mut s,seat,"ASH_047");
        let keeper=add(&e,&mut s,seat,"ASH_032");s.player_mut(seat).hand.push(keeper).unwrap();
        let militia=add(&e,&mut s,seat.other(),"ASH_160");s.player_mut(seat.other()).deck.push(militia).unwrap();
        let nuvo=add(&e,&mut s,seat,"HMW_062");s.player_mut(seat).discard.push(nuvo).unwrap();
        let kallus=add(&e,&mut s,seat.other(),"SOR_115");s.player_mut(seat.other()).resources.push(kallus).unwrap();
        let h=s.round_limits.native(gar,0).unwrap();s.round_limits.consume(h,seat.index() as u8).unwrap();
        let raw=e.snapshot(&s);let rows=raw["roundLimitState"]["registrations"].as_array().unwrap();assert_eq!(rows.len(),5);assert_eq!(rows[3]["ordinal"],1);assert_eq!(rows[0]["used"][seat.index()],1);
        let loaded=e.load_state(&raw).unwrap();assert_eq!(e.snapshot(&loaded)["roundLimitState"],raw["roundLimitState"]);
        assert!(used(&loaded,gar,Registration::PlayedUpgradeOnSelf,seat));
        let mut missing=raw.clone();missing.as_object_mut().unwrap().remove("roundLimitState");assert!(e.load_state(&missing).unwrap_err().contains("requires exact roundLimitState"));
        for mutation in 0..8 {let mut bad=raw.clone();match mutation {
            0=>{bad["roundLimitState"]["registrations"].as_array_mut().unwrap().pop();},
            1=>bad["roundLimitState"]["registrations"][3]["ordinal"]=json!(0),
            2=>bad["roundLimitState"]["registrations"][0]["used"]=json!([true,0]),
            3=>bad["roundLimitState"]["registrations"][0]["used"]=json!([2,0]),
            4=>bad["roundLimitState"]["registrations"][0]["extra"]=json!(false),
            5=>bad["roundLimitState"]["version"]=json!("1"),
            6=>bad["roundLimitState"]["registrations"][0]["card"]=rows[1]["card"].clone(),
            _=>bad["roundLimitPending"]=json!(false),
        }assert!(e.load_state(&bad).is_err(),"mutation {mutation}");}
        let empty=board(&e,seat);let mut absent=e.snapshot(&empty);assert_eq!(absent["roundLimitState"]["registrations"],json!([]));absent.as_object_mut().unwrap().remove("roundLimitState");assert!(e.load_state(&absent).is_err());
    }
    let mut c:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();c["cards"].as_array_mut().unwrap().retain(|v|v["setCode"].as_str().is_some_and(|code|code.starts_with("TOKEN_")||["SOR_023","SOR_014","LOF_254"].contains(&code)));
    let producer_free=Engine::from_catalogue_value(&c).unwrap();assert!(!producer_free.needs_round_limit_state());let s=board(&producer_free,Seat::P1);let raw=producer_free.snapshot(&s);assert!(raw.get("roundLimitState").is_none());assert!(producer_free.load_state(&raw).is_ok());
}
#[test]
fn round_limit_native_captured_handles_reject_foreign_registration_and_controller() {
    let e=engine(RulesBackend::Interpreter);let mut s=board(&e,Seat::P1);let keeper=unit(&e,&mut s,Seat::P1,"ASH_032");let copy=unit(&e,&mut s,Seat::P1,"ASH_032");let victim=unit(&e,&mut s,Seat::P1,"LOF_164");s.source=victim;s.ability_target=victim;e.apply_effect(&mut s,&damage()).unwrap();assert_eq!(s.death_pending.len,2);
    let before=s;let h=s.death_pending.entries[0].round_limit;s.death_pending.entries[1].round_limit=h;assert!(e.validate_round_limit_handles(&s).unwrap_err().contains("physical registration"));s=before;s.card_mut(keeper).owner=Seat::P2;assert!(e.validate_round_limit_handles(&s).unwrap_err().contains("controller change"));s=before;s.death_pending.entries[0].round_limit=crate::round_limit_state::Handle::NONE;assert!(e.validate_round_limit_handles(&s).unwrap_err().contains("instance is missing"));assert_ne!(keeper,copy);
}

#[test]
fn round_limit_native_imported_retained_definition_also_requires_counter_proof() {
    let mut e=engine(RulesBackend::Interpreter);let mut s=board(&e,Seat::P1);unit(&e,&mut s,Seat::P1,"ASH_032");
    e.catalogue.as_mut().unwrap().metadata.retain(|code,_|!["ASH_047","ASH_032","ASH_160","HMW_062","SOR_115"].contains(&code.as_str()));
    assert!(!e.needs_round_limit_state());let mut raw=e.snapshot(&s);assert!(raw.get("roundLimitState").is_some());raw.as_object_mut().unwrap().remove("roundLimitState");assert!(e.load_state(&raw).is_err());assert!(e.load_round_limit_state(&mut s,&raw).unwrap_err().contains("imported pool requires exact roundLimitState"));
}

#[test]
fn round_limit_native_old_siblings_and_reentry_use_distinct_shared_instances() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let kallus=unit(&e,&mut s,seat,"SOR_115");let a=unit(&e,&mut s,seat.other(),"ASH_036");let b=unit(&e,&mut s,seat.other(),"SHD_037");let c=unit(&e,&mut s,seat.other(),"HMW_062");let deck=s.player(seat).deck.len;
        e.defeat(&mut s,a).unwrap();e.defeat(&mut s,b).unwrap();assert_eq!(s.death_pending.len,2);let old=s.death_pending.entries[0].round_limit;assert_eq!(old,s.death_pending.entries[1].round_limit);
        // Deliberately staged pending-window lifecycle witness, distinct from
        // the sealed source-only games: old captures survive a physical clone.
        e.leave_play(&mut s,kallus).unwrap();s.player_mut(seat).discard.remove(kallus);e.add_arena(&mut s,kallus).unwrap();e.defeat(&mut s,c).unwrap();assert_eq!(s.death_pending.len,3);assert_ne!(old,s.death_pending.entries[2].round_limit);
        open(&e,&mut s);
        for _ in 0..20 {if s.is_action_window(){break;}let actions=e.legal_actions(&s);let action=match s.decision {
            Decision::AbilityOptional=>actions.iter().find(|a|a.arg.as_ref().and_then(Value::as_str)==Some("trigger")),
            Decision::DeathBatch=>actions.iter().find(|a|a.arg.as_ref().and_then(Value::as_str)==Some("all")),
            Decision::DeathOrder=>actions.first(),
            _=>panic!("unexpected pending lifecycle prompt {:?}: {actions:?}",s.decision),
        }.unwrap();s=e.apply(&s,action).unwrap();}
        assert!(s.is_action_window());assert_eq!(s.player(seat).deck.len,deck-2,"two old contexts share one use, fresh clone gets one separate use");assert!(used(&s,kallus,Registration::OtherUniqueUnitDefeated,seat));
        let round=s.round;e.advance_after_expiry(&mut s).unwrap();assert!(s.regroup);assert_eq!(s.decision,Decision::Resource);assert!(used(&s,kallus,Registration::OtherUniqueUnitDefeated,seat),"phase boundary does not reset per-round use");s=arg(&e,&s,"done");s=arg(&e,&s,"done");assert_eq!(s.round,round+1);assert!(!used(&s,kallus,Registration::OtherUniqueUnitDefeated,seat));
    }}
}

#[test]
fn round_limit_native_ready_militia_retains_shared_window_owner_bucket() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let militia=unit(&e,&mut s,seat,"ASH_160");let pershing=unit(&e,&mut s,seat.other(),"ASH_072");let deck=s.player(seat.other()).deck.len;
        s.active=seat.other();s.action_player=Some(seat.other());s=choose(&e,&s,pershing);let base=s.player(seat).base;s=choose(&e,&s,base);
        assert_eq!(s.decision,Decision::DeathPlayer);assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),["You","Opponent"]);
        let you=e.legal_actions(&s)[0].clone();s=e.apply(&s,&you).unwrap();assert!(s.is_action_window());assert_eq!(s.player(seat.other()).deck.len,deck-1);assert!(!used(&s,militia,Registration::EnemyGroundAttackAtBase,seat));assert!(!s.card(militia).exhausted);
    }}
}
#[test]
fn round_limit_native_retired_parent_roots_survive_all_eight_frames_and_overflow_is_atomic() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for parents in 0..8 {
        let mut s=board(&e,seat);let mut old=Vec::new();
        // Structural native suspension witness. Parent frames are staged from
        // real completed Militia handlers, not claimed as fresh engine games.
        for i in 0..parents {let source=unit(&e,&mut s,seat,"ASH_160");let p=e.card_programs(&s,source)[0];let h=s.round_limits.native(source,0).unwrap();s.round_limits.consume(h,seat.index() as u8).unwrap();
            let mut context=ResolutionContext::capture(&s);context.active=seat;context.source=source;context.round_limit=h;context.ability_program=p+1;context.ability_pc=e.program(p).event_end;context.ability_return=if i==0 {ProgramContinuation::Finish.raw()} else {ProgramContinuation::Deaths.raw()};context.ability_committed=true;
            s.death_frames[i]=DeathFrame{context,window:DeathWindow::default(),resume:DeathContinuation::Program.raw()};s.death_depth=(i+1) as u8;
            e.leave_play(&mut s,source).unwrap();s.player_mut(seat).discard.remove(source);e.add_arena(&mut s,source).unwrap();old.push(h);
        }
        let gar=unit(&e,&mut s,seat,"ASH_047");let upgrade=add(&e,&mut s,seat,"IC27_052");s.player_mut(seat).hand.push(upgrade).unwrap();s.source=upgrade;assert!(e.play_upgrade_with_payment(&mut s,upgrade,gar).unwrap());assert_eq!(s.decision,Decision::AbilityOptional);
        assert_eq!(s.death_depth as usize,parents+1);assert!(!old.contains(&s.round_limit));for h in &old {assert!(!s.round_limits.available(*h,seat.index() as u8).unwrap());}e.validate_round_limit_handles(&s).unwrap();
        let trigger=e.legal_actions(&s).into_iter().find(|a|a.arg.as_ref().and_then(Value::as_str)==Some("trigger")).unwrap();let before=s;
        if parents==7 {assert!(e.apply(&s,&trigger).unwrap_err().contains("nested defeat window capacity"));assert_eq!(s,before);assert!(!used(&s,gar,Registration::PlayedUpgradeOnSelf,seat));}
        else {s=e.apply(&s,&trigger).unwrap();assert!(s.is_action_window());assert!(used(&s,gar,Registration::PlayedUpgradeOnSelf,seat));assert_eq!(s.death_depth,0);}
    }}}
}

#[test]
fn round_limit_native_keeper_combat_and_each_first_strike_window_prune_shields() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for window in 0..3 {for shielded in [false,true] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");
        let friendly=unit(&e,&mut s,seat,"ASH_197");let enemy=unit(&e,&mut s,seat.other(),"ASH_197");
        if shielded {e.create_token(&mut s,friendly,false).unwrap();}
        let (actor,attacker,target)=if window==1 {(seat.other(),enemy,friendly)}else{(seat,friendly,enemy)};
        s.active=actor;s.action_player=Some(actor);
        if window!=0 {
            let event=add(&e,&mut s,actor,"IC27_175");s.player_mut(actor).hand.push(event).unwrap();s=choose(&e,&s,event);
            assert!(e.legal_actions(&s).iter().any(|a|a.label=="Cancel"));s=choose(&e,&s,attacker);
            let mut expected=vec![e.card_action(&s,target).key(),e.card_action(&s,s.player(actor.other()).base).key()];expected.sort();let mut actual=e.legal_actions(&s).into_iter().map(|a|a.key()).collect::<Vec<_>>();actual.sort();assert_eq!(actual,expected);
        } else {s=choose(&e,&s,attacker);}
        s=choose(&e,&s,target);
        if shielded {assert!(s.is_action_window());assert_eq!(s.card(friendly).damage,0);assert!(!used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));}
        else {
            assert_eq!(s.decision,Decision::AbilityTarget);assert!(s.card(friendly).damage>0);assert!(e.in_arena(&s,friendly));
            let mut expected=vec![Action::button("Choose nothing","done").key(),e.card_action(&s,s.player(seat).base).key(),e.card_action(&s,s.player(seat.other()).base).key()];expected.sort();let mut actual=e.legal_actions(&s).into_iter().map(|a|a.key()).collect::<Vec<_>>();actual.sort();assert_eq!(actual,expected);
            s=arg(&e,&s,"done");assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));assert!(s.is_action_window());
        }
    }}}}
}

#[test]
fn round_limit_native_distinct_events_keep_optional_contexts_with_shared_counter() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        for source_dies in [false,true] {for all in [false,true] {for pass_first in [false,true] {
        let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,"SOR_115");let a=unit(&e,&mut s,seat.other(),"ASH_036");let b=unit(&e,&mut s,seat.other(),"SHD_037");let deck=s.player(seat).deck.len;
        let ids=if source_dies {vec![source,a,b]}else{vec![a,b]};e.defeat_many(&mut s,&ids).unwrap();assert_eq!(s.death_pending.len,2);assert_eq!(s.death_pending.entries[0].event_card,a);assert_eq!(s.death_pending.entries[1].event_card,b);let shared=s.death_pending.entries[0].round_limit;assert_eq!(shared,s.death_pending.entries[1].round_limit);
        open(&e,&mut s);assert_eq!(s.decision,Decision::DeathBatch);assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),["Resolve next","Resolve all (2)"]);
        s=arg(&e,&s,if all {"all"}else{"next"});
        if pass_first {s=arg(&e,&s,"pass");assert_eq!(s.decision,Decision::AbilityOptional);assert!(s.round_limits.available(shared,seat.index() as u8).unwrap());}
        s=arg(&e,&s,"trigger");assert_eq!(s.player(seat).deck.len,deck-1);assert!(s.is_action_window());
        assert_eq!(used(&s,source,Registration::OtherUniqueUnitDefeated,seat),!source_dies,"source death clones a fresh offboard current registration; captured old object is consumed");
        }}}
        let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,"HMW_062");let recipient=unit(&e,&mut s,seat,"SOR_095");let a=unit(&e,&mut s,seat.other(),"LOF_164");let b=unit(&e,&mut s,seat.other(),"LOF_164");
        let weakness:Effect=serde_json::from_value(json!({"kind":"giveWeakness","target":"self","amount":{"kind":"const","value":1}})).unwrap();for id in [a,b] {s.source=id;e.apply_effect(&mut s,&weakness).unwrap();}s.source=0;e.defeat_many(&mut s,&[a,b]).unwrap();assert_eq!(s.death_pending.len,2);assert_eq!(s.death_pending.entries[0].round_limit,s.death_pending.entries[1].round_limit);
        open(&e,&mut s);assert_eq!(s.decision,Decision::DeathBatch);s=arg(&e,&s,"next");s=arg(&e,&s,"passAbility");assert_eq!(s.decision,Decision::AbilityTarget);assert!(!used(&s,source,Registration::EnemyWeaknessUnitDefeated,seat));s=choose(&e,&s,recipient);assert!(used(&s,source,Registration::EnemyWeaknessUnitDefeated,seat));assert!(e.attached_native_token(&s,recipient,"weakness"));assert!(s.is_action_window());
    }}
}
#[test]
fn round_limit_native_distinct_context_capacity_preflights_without_allocating_or_mutating() {
    let e=engine(RulesBackend::Interpreter);let mut s=board(&e,Seat::P1);let source=unit(&e,&mut s,Seat::P1,"SOR_115");
    let events=(0..65).map(|i|((i+100) as CardId,crate::round_limit_runtime::Event::Defeated{card:(i+100) as CardId,controller:1,was_unit:true,unique:true,had_weakness:false})).collect::<Vec<_>>();
    let before=s;assert!(e.capture_round_limit_window(&mut s,&events).unwrap_err().contains("trigger capture capacity"));assert_eq!(s,before);
    e.capture_round_limit_window(&mut s,&events[..64]).unwrap();assert_eq!(s.death_pending.len,64);let h=s.death_pending.entries[0].round_limit;assert!(s.death_pending.entries.iter().all(|entry|entry.source==source&&entry.round_limit==h));for (entry,(event_card,_)) in s.death_pending.entries.iter().zip(&events) {assert_eq!(entry.event_card,*event_card);}
    let before=s;assert!(e.capture_round_limit_window(&mut s,&events[..1]).is_err());assert_eq!(s,before);
}
#[test]
fn round_limit_native_deployed_leaders_are_units_in_captured_damage_and_death_facts() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");let leader=s.player(seat).leader;assert!(!e.round_limit_live_unit(&s,leader));s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();assert!(e.round_limit_live_unit(&s,leader));s.source=leader;s.ability_target=leader;e.apply_effect(&mut s,&damage()).unwrap();open(&e,&mut s);assert_eq!(s.decision,Decision::AbilityTarget);s=arg(&e,&s,"done");assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));
        for nuvo in [false,true] {let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,if nuvo {"HMW_062"}else{"SOR_115"});let leader=s.player(seat.other()).leader;s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();if nuvo {s.source=leader;let weakness:Effect=serde_json::from_value(json!({"kind":"giveWeakness","target":"self","amount":{"kind":"const","value":1}})).unwrap();e.apply_effect(&mut s,&weakness).unwrap();}s.source=0;e.defeat_many(&mut s,&[leader]).unwrap();assert_eq!(s.death_pending.len,1);assert_eq!(s.death_pending.entries[0].event_card,leader);open(&e,&mut s);if nuvo {assert_eq!(s.decision,Decision::AbilityTarget);s=choose(&e,&s,source);assert!(used(&s,source,Registration::EnemyWeaknessUnitDefeated,seat));}else{assert_eq!(s.decision,Decision::AbilityOptional);s=arg(&e,&s,"trigger");assert!(used(&s,source,Registration::OtherUniqueUnitDefeated,seat));}assert!(s.is_action_window());}
    }}
}

#[test]
fn round_limit_native_shared_registration_order_is_required_before_future_events() {
    let mut raw:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    raw["cards"].as_array_mut().unwrap().retain(|v|v["setCode"].as_str().is_some_and(|code|code.starts_with("TOKEN_")||["SOR_023","SOR_014","LOF_254","ASH_047","ASH_032","ASH_160","HMW_062","SOR_115"].contains(&code)));
    let e=Engine::from_catalogue_value(&raw).unwrap();assert!(e.needs_registration_order());let s=board(&e,Seat::P1);let snapshot=e.snapshot(&s);assert_eq!(snapshot["roundLimitState"]["registrations"],json!([]));assert!(e.load_state(&snapshot).is_ok());let mut missing=snapshot;missing.as_object_mut().unwrap().remove("registrationOrder");assert!(e.load_state(&missing).unwrap_err().contains("registrationOrder"));
    raw["cards"].as_array_mut().unwrap().retain(|v|!["ASH_047","ASH_032","ASH_160","HMW_062","SOR_115"].contains(&v["setCode"].as_str().unwrap()));let legacy=Engine::from_catalogue_value(&raw).unwrap();assert!(!legacy.needs_registration_order());let mut snapshot=legacy.snapshot(&board(&legacy,Seat::P1));snapshot.as_object_mut().unwrap().remove("registrationOrder");assert!(legacy.load_state(&snapshot).is_ok());
}
#[test]
fn round_limit_native_ordinary_and_limited_death_listeners_share_event_registration_order() {
    let mut raw:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();
    raw["cards"].as_array_mut().unwrap().retain(|v|v["setCode"].as_str().is_some_and(|code|code.starts_with("TOKEN_")||["SOR_023","SOR_014","LOF_254","SOR_115","ASH_153","SOR_095","ASH_191"].contains(&code)));
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let mut e=Engine::from_catalogue_value(&raw).unwrap();e.set_rules_backend(backend).unwrap();for seat in [Seat::P1,Seat::P2] {for ordinary_first in [false,true] {
        let mut s=board(&e,seat);let (green,kallus)=if ordinary_first {let green=unit(&e,&mut s,seat,"ASH_153");let kallus=unit(&e,&mut s,seat,"SOR_115");(green,kallus)}else{let kallus=unit(&e,&mut s,seat,"SOR_115");let green=unit(&e,&mut s,seat,"ASH_153");(green,kallus)};let marine=unit(&e,&mut s,seat.other(),"SOR_095");let deck=s.player(seat).deck.len;
        e.defeat_many(&mut s,&[green]).unwrap();assert_eq!(s.death_pending.len,2);assert_eq!(s.death_pending.entries[..2].iter().map(|e|e.source).collect::<Vec<_>>(),if ordinary_first {vec![green,kallus]}else{vec![kallus,green]});
        open(&e,&mut s);assert_eq!(s.decision,Decision::DeathOrder);assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),if ordinary_first {vec!["Deal 2 damage to a unit","Draw a card"]}else{vec!["Draw a card","Deal 2 damage to a unit"]});s=arg(&e,&s,"0");
        if ordinary_first {s=choose(&e,&s,marine);s=arg(&e,&s,"trigger");}else{s=arg(&e,&s,"trigger");s=choose(&e,&s,marine);}assert!(s.is_action_window());assert_eq!(s.card(marine).damage,2);assert_eq!(s.player(seat).deck.len,deck-1);
    }}}
    // Native structural two-event witness: no family-priority append may move
    // an event-2 listener ahead of event-1 ordinary registrations.
    let e=Engine::from_catalogue_value(&raw).unwrap();let mut s=board(&e,Seat::P1);let green=unit(&e,&mut s,Seat::P1,"ASH_153");let kallus=unit(&e,&mut s,Seat::P1,"SOR_115");let fiend=unit(&e,&mut s,Seat::P1,"ASH_191");assert!(e.def(&s,fiend).info.unique);e.defeat_many(&mut s,&[green,fiend]).unwrap();assert_eq!(s.death_pending.entries[..s.death_pending.len as usize].iter().map(|e|e.source).collect::<Vec<_>>(),vec![green,kallus,kallus,fiend]);
}

#[test]
fn round_limit_native_prepared_and_uniform_damage_capture_deployed_leader_unit_facts() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for uniform in [false,true] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");let leader=s.player(seat).leader;s.card_mut(leader).deployed=true;e.add_arena(&mut s,leader).unwrap();s.source=leader;s.ability_target=leader;
        let effect:Effect=if uniform {serde_json::from_value(json!({"kind":"damage","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true},"controllerOrder":"seatOrder"},"amount":{"kind":"const","value":1}})).unwrap()}else{
            let catalogue:Value=serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"),"/catalogue.json"))).unwrap();let raw=catalogue["cards"].as_array().unwrap().iter().find(|v|v["setCode"]=="TWI_174").unwrap();serde_json::from_value(raw["abilities"]["card"][0]["effect"].clone()).unwrap()
        };
        s.event_group.context=Selection::one(leader);assert!(e.generate_group(&mut s,&effect).unwrap());assert_eq!(s.event_group.uniform_damage.active,uniform);assert!(e.resolve_group(&mut s,&effect).unwrap());
        assert!(s.death_pending.entries[..s.death_pending.len as usize].iter().any(|entry|entry.event_card==leader));assert_eq!(s.death_pending.len,if uniform {2}else{1});s.event_group=EventGroupState::default();open(&e,&mut s);if uniform {assert_eq!(s.decision,Decision::DeathBatch);s=arg(&e,&s,"all");}s=arg(&e,&s,"done");assert!(s.is_action_window());assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));
    }}}
}
#[test]
fn round_limit_native_common_death_capture_capacity_failure_is_atomic_across_families() {
    let e=engine(RulesBackend::Interpreter);let mut s=board(&e,Seat::P1);let kallus=unit(&e,&mut s,Seat::P1,"SOR_115");let mut victims=Vec::new();for _ in 0..33 {victims.push(unit(&e,&mut s,Seat::P1,"ASH_153"));}
    // Structural capacity position intentionally stages repeated unique copies;
    // it proves error atomicity, not a reachable completed engine game.
    let events=victims.iter().map(|id|(*id,crate::defeat_cause::DefeatCause::Ability)).collect::<Vec<_>>();let before=s;assert!(e.capture_defeat_window(&mut s,&events).is_err());assert_eq!(s,before);
    e.capture_defeat_window(&mut s,&events[..32]).unwrap();assert_eq!(s.death_pending.len,64);for (i,victim) in victims[..32].iter().enumerate() {assert_eq!(s.death_pending.entries[i*2].source,kallus);assert_eq!(s.death_pending.entries[i*2+1].source,*victim);}let before=s;assert!(e.capture_defeat_window(&mut s,&events[..1]).is_err());assert_eq!(s,before);
}

#[test]
fn round_limit_native_distributed_six_keeper_contexts_follow_sorted_damage_emission() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat.other(),"ASH_032");let a=unit(&e,&mut s,seat.other(),"ASH_061");let b=unit(&e,&mut s,seat.other(),"ASH_131");assert!(e.hp(&s,a)>3&&e.hp(&s,b)>3);
        let source=add(&e,&mut s,seat,"SOR_135");s.player_mut(seat).hand.push(source).unwrap();let resources=e.ready_resources(&s,seat);s=choose(&e,&s,source);assert_eq!(s.decision,Decision::AbilityDistribute);assert!(e.in_arena(&s,source));assert!(e.ready_resources(&s,seat)<resources);
        let allocation=json!([[format!("c{}",a-1),3],[format!("c{}",b-1),3]]);let action=e.legal_actions(&s).into_iter().find(|action|action.arg.as_ref()==Some(&allocation)).expect("exact legal six-damage allocation");s=e.apply(&s,&action).unwrap();
        assert_eq!(s.decision,Decision::DeathBatch);assert_eq!(s.death_window.len,2);let entries=&s.death_window.entries[..2];assert_eq!(entries.iter().map(|entry|entry.event_card).collect::<Vec<_>>(),vec![b,a]);assert!(entries.iter().all(|entry|entry.source==keeper));assert_eq!(entries[0].round_limit,entries[1].round_limit);
        assert_eq!(e.legal_actions(&s).iter().map(|action|action.label.as_str()).collect::<Vec<_>>(),["Resolve next","Resolve all (2)"]);s=arg(&e,&s,"next");let target=s.player(seat).base;s=choose(&e,&s,target);assert_eq!(s.decision,Decision::AbilityTarget);assert!(!used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat.other()));s=arg(&e,&s,"done");assert!(s.is_action_window());assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat.other()));assert_eq!(s.card(a).damage,3);assert_eq!(s.card(b).damage,3);assert_eq!(s.card(keeper).damage,0);assert_eq!(s.card(s.player(seat).base).damage,1);assert_eq!(s.card(s.player(seat.other()).base).damage,0);
    }}
}

#[test]
fn round_limit_native_distributed_shields_keep_replacement_insertion_order_and_filter_contexts() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for shields in [1,2] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat.other(),"ASH_032");let a=unit(&e,&mut s,seat.other(),"ASH_061");let b=unit(&e,&mut s,seat.other(),"ASH_131");e.create_token(&mut s,a,false).unwrap();if shields==2 {e.create_token(&mut s,b,false).unwrap();}
        let source=add(&e,&mut s,seat,"SOR_135");s.player_mut(seat).hand.push(source).unwrap();s=choose(&e,&s,source);assert_eq!(s.decision,Decision::AbilityDistribute);let allocation=json!([[format!("c{}",a-1),3],[format!("c{}",b-1),3]]);let action=e.legal_actions(&s).into_iter().find(|action|action.arg.as_ref()==Some(&allocation)).unwrap();s=e.apply(&s,&action).unwrap();
        if shields==2 {assert_eq!(s.decision,Decision::AbilityGroupShield);assert_eq!(s.event_group.records[..2].iter().map(|r|r.target).collect::<Vec<_>>(),vec![a,b]);let Instruction::Apply(effect)=&e.program(s.ability_program-1).instructions[s.ability_pc as usize] else {panic!("missing distribution")};assert_eq!(e.group_shield_targets(&s,effect,seat.other()),vec![a,b]);s=arg(&e,&s,"all");assert!(s.is_action_window());assert!(!used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat.other()));assert_eq!(s.card(b).damage,0);}
        else {assert_eq!(s.decision,Decision::AbilityTarget);assert_eq!(s.entry_event_card,b);assert_eq!(s.death_window.len,0);s=arg(&e,&s,"done");assert!(s.is_action_window());assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat.other()));assert_eq!(s.card(b).damage,3);}
        assert_eq!(s.card(a).damage,0);assert_eq!(s.card(a).shield,0);assert_eq!(s.card(keeper).damage,0);
    }}}
}
#[test]
fn round_limit_native_uniform_zero_order_preserves_forward_context_order() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);let keeper=unit(&e,&mut s,seat,"ASH_032");let a=unit(&e,&mut s,seat,"ASH_061");let b=unit(&e,&mut s,seat,"ASH_131");s.source=a;
        // Native uniform-order control; the runtime effect has order zero,
        // unlike distributed-six negative-order children. No source game claim.
        let effect:Effect=serde_json::from_value(json!({"kind":"damage","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true},"controllerOrder":"seatOrder"},"amount":{"kind":"const","value":1}})).unwrap();assert!(e.generate_uniform_damage(&mut s,&effect).unwrap());assert!(e.resolve_group(&mut s,&effect).unwrap());assert_eq!(s.death_pending.entries[..s.death_pending.len as usize].iter().map(|entry|entry.event_card).collect::<Vec<_>>(),vec![keeper,a,b]);let h=s.death_pending.entries[0].round_limit;assert!(s.death_pending.entries[..3].iter().all(|entry|entry.round_limit==h));s.event_group=EventGroupState::default();open(&e,&mut s);s=arg(&e,&s,"all");s=arg(&e,&s,"done");assert!(s.is_action_window());assert!(used(&s,keeper,Registration::FriendlyUnitNonlethalDamage,seat));
    }}
}

#[test]
fn round_limit_native_consumed_contexts_retain_batch_menu_before_resolver_skip() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {let e=engine(backend);for seat in [Seat::P1,Seat::P2] {for keeper in [false,true] {for all in [false,true] {
        let mut s=board(&e,seat);let source=unit(&e,&mut s,seat,if keeper {"ASH_032"}else{"SOR_115"});let deck=s.player(seat).deck.len;
        if keeper {
            let attacker=unit(&e,&mut s,seat.other(),"SOR_095");
            for _ in 0..3 {let victim=unit(&e,&mut s,seat,"LOF_164");s.source=attacker;s.ability_target=victim;e.apply_effect(&mut s,&damage()).unwrap();}
        } else {
            let victims=(0..3).map(|_|unit(&e,&mut s,seat.other(),"ASH_036")).collect::<Vec<_>>();e.defeat_many(&mut s,&victims).unwrap();
        }
        assert_eq!(s.death_pending.len,3);let handle=s.death_pending.entries[0].round_limit;assert!(s.death_pending.entries[..3].iter().all(|entry|entry.round_limit==handle));
        open(&e,&mut s);assert_eq!(s.decision,Decision::DeathBatch);assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),["Resolve next","Resolve all (3)"]);
        s=arg(&e,&s,"next");
        if keeper {assert_eq!(s.decision,Decision::AbilityTarget);s=arg(&e,&s,"done");}else{assert_eq!(s.decision,Decision::AbilityOptional);s=arg(&e,&s,"trigger");}
        assert!(!s.round_limits.available(handle,seat.index() as u8).unwrap());
        assert_eq!(s.decision,Decision::DeathBatch,"consumed registrations remain event contexts until resolver requirement checks; {backend:?} {seat:?} keeper={keeper}");
        assert_eq!(s.active,seat);assert_eq!(s.death_window.len,2);
        assert_eq!(e.legal_actions(&s).iter().map(|a|a.label.as_str()).collect::<Vec<_>>(),["Resolve next","Resolve all (2)"]);
        s=arg(&e,&s,if all {"all"}else{"next"});assert!(s.is_action_window(),"spent contexts skip before optional/target prompts");
        assert_eq!(s.player(seat).deck.len,deck-if keeper {0}else{1});assert_eq!(s.card(s.player(seat).base).damage,0);assert_eq!(s.card(s.player(seat.other()).base).damage,0);
    }}}}
}
