//! Source-derived midgame naming regressions; no differential verification claim.
use super::*;
fn fixture(e:&Engine,actor:Seat)->State {
    let mut s=board(e);s.named_play=e.new_named_play_state().unwrap();s.round=4;s.active=actor;s.action_player=Some(actor);
    for seat in [actor,actor.other()] {for _ in 0..20 {
        let id=add(e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(id).unwrap();
    }}
    s
}
fn in_hand(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {
    let id=add(e,s,seat,code);s.player_mut(seat).hand.push(id).unwrap();id
}
fn select(e:&Engine,s:&State,id:CardId)->State {
    let action=e.card_action(s,id);
    assert!(e.legal_actions(s).contains(&action),"card {id} not offered: {:?}",e.legal_actions(s));
    e.apply(s,&action).unwrap()
}
fn name(e:&Engine,s:&State,title:&str)->State {
    let action=Action::button(title,title);
    assert!(e.legal_actions(s).contains(&action),"title {title} not offered");
    e.apply(s,&action).unwrap()
}
fn can_select(e:&Engine,s:&State,id:CardId)->bool {
    e.legal_actions(s).contains(&e.card_action(s,id))
}
#[test]
fn named_play_midgame_actual_unit_play_names_exact_string_and_blocks_only_opponent_title() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        e.set_legal_card_titles(&["Luke Skywalker".into(),"Regional Governor".into()]).unwrap();
        for actor in [Seat::P1,Seat::P2] {for code in ["ASH_077","SOR_062"] {
            let mut s=fixture(&e,actor);
            let source=in_hand(&e,&mut s,actor,code);
            let own=in_hand(&e,&mut s,actor,"SOR_051");
            let enemy=in_hand(&e,&mut s,actor.other(),"SOR_051");
            let other=in_hand(&e,&mut s,actor.other(),"LOF_254");
            let ready=e.ready_resources(&s,actor);let cost=e.cost(&s,source);
            let naming=select(&e,&s,source);
            assert!(e.in_arena(&naming,source));
            assert!(!naming.player(actor).hand.contains(source));
            assert_eq!(e.ready_resources(&naming,actor),ready-usize::from(cost));
            assert_eq!(naming.active,actor);
            let snapshot=e.snapshot(&naming);
            assert_eq!(snapshot["namedPlayPending"],json!(true));
            for marker in [json!(true),json!(false),Value::Null] {
                let mut forged=snapshot.clone();forged["namedPlayPending"]=marker;
                forged["actor"]=forged["activePlayer"].clone();
                assert!(e.load_state(&forged).is_err());
                forged["winner"]=json!("p1");assert!(e.load_state(&forged).is_err());
            }
            assert_eq!(e.legal_actions(&naming),vec![Action::button("Luke Skywalker","Luke Skywalker"),Action::button("Regional Governor","Regional Governor")]);
            assert!(!e.legal_actions(&naming).iter().any(|a|a.label=="Cancel"));
            let done=name(&e,&naming,"Luke Skywalker");
            assert_eq!(done.active,actor.other());
            assert!(!can_select(&e,&done,enemy));assert!(can_select(&e,&done,other));
            assert!(e.apply(&done,&e.card_action(&done,enemy)).is_err());
            // Fixture-only actor projection checks the fixed recipient without
            // playing the named card and introducing its independent ability.
            let mut own_turn=done;own_turn.active=actor;own_turn.action_player=Some(actor);
            assert!(can_select(&e,&own_turn,own));
        }}
    }
}
#[test]
fn named_play_captured_when_played_still_names_after_source_departure_without_live_lock() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        e.set_legal_card_titles(&["Luke Skywalker".into(),"Regional Governor".into()]).unwrap();
        for actor in [Seat::P1,Seat::P2] {
            let mut s=fixture(&e,actor);
            let source=in_hand(&e,&mut s,actor,"SOR_062");
            let enemy=in_hand(&e,&mut s,actor.other(),"SOR_051");
            let mut naming=select(&e,&s,source);
            // Direct departure isolates the already captured entry continuation.
            e.leave_play(&mut naming,source).unwrap();
            assert!(!e.in_arena(&naming,source));
            let done=name(&e,&naming,"Luke Skywalker");
            assert!(can_select(&e,&done,enemy));
        }
    }
}
#[test]
fn named_play_overlapping_sources_leave_independently_and_reentry_does_not_revive_lock() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        e.set_legal_card_titles(&["Luke Skywalker".into(),"Regional Governor".into()]).unwrap();
        for actor in [Seat::P1,Seat::P2] {
            let mut s=fixture(&e,actor);
            let first=in_hand(&e,&mut s,actor,"SOR_062");let second=in_hand(&e,&mut s,actor,"SOR_062");
            let enemy=in_hand(&e,&mut s,actor.other(),"SOR_051");
            let naming=select(&e,&s,first);let mut done=name(&e,&naming,"Luke Skywalker");
            // Project the next same-seat action window to isolate overlapping
            // persistent effects without unrelated opponent actions.
            done.active=actor;done.action_player=Some(actor);
            let naming=select(&e,&done,second);let mut done=name(&e,&naming,"Luke Skywalker");
            assert!(!can_select(&e,&done,enemy));
            e.leave_play(&mut done,first).unwrap();assert!(!can_select(&e,&done,enemy));
            e.leave_play(&mut done,second).unwrap();assert!(can_select(&e,&done,enemy));
            // Direct reentry deliberately does not generate a new When Played.
            done.player_mut(actor).discard.remove(first);e.add_arena(&mut done,first).unwrap();
            assert!(can_select(&e,&done,enemy));
        }
    }
}
#[test]
fn named_play_title_equality_covers_subtitles_and_alias_printings() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        e.set_legal_card_titles(&["Luke Skywalker".into(),"Regional Governor".into()]).unwrap();
        for actor in [Seat::P1,Seat::P2] {
            let mut s=fixture(&e,actor);let source=in_hand(&e,&mut s,actor,"SOR_062");
            let codes:Vec<String>=e.definitions.iter().filter(|d|d.info.title=="Luke Skywalker"&&d.info.is("unit")).map(|d|d.code.clone()).collect();
            assert!(codes.len()>=2,"need admitted distinct Luke subtitles/printings");
            let targets:Vec<_>=codes.iter().map(|code|in_hand(&e,&mut s,actor.other(),code)).collect();
            let naming=select(&e,&s,source);let done=name(&e,&naming,"Luke Skywalker");
            for target in targets {assert!(!can_select(&e,&done,target));}
        }
    }
}
