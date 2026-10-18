//! Source-derived midgame tests, not forceteki differential evidence.
use super::*;
use crate::migration_modal_deck_contract::prepare;

fn deck_entry() -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "LOF_200")
        .unwrap()
        .clone()
}
#[test]
fn migration_modal_deck_complete_source_proof_retains_registration() {
    let entry = deck_entry();
    let p = prepare(&entry).unwrap().unwrap();
    assert_eq!(p.registration, entry["abilities"]["card"][0]);
    assert_eq!(p.descriptor.zone, "groundArena");
    assert!(p.descriptor.optional_target);
    let mut renamed = entry.clone();
    renamed["setCode"] = json!("DIAGNOSTIC_DECK");
    renamed["card"]["title"] = json!("Diagnostic unit");
    assert_eq!(prepare(&renamed).unwrap(), Some(p));
}
#[test]
fn migration_modal_deck_raw_headers_and_factory_capture_mutations_fail_closed() {
    let original = deck_entry();
    for (path,value) in [
        ("/abilities/card/0/optional",json!(true)),
        ("/abilities/card/0/costs",json!([{"kind":"exhaustSelf"}])),
        ("/abilities/card/0/targetResolvers/0/targetResolver/optional",json!(false)),
        ("/abilities/card/0/targetResolvers/0/targetResolver/zones/value",json!("anyAttackable")),
        ("/abilities/card/0/targetResolvers/1/targetResolver/choices/entries/0/constructionCaptures/0/evaluateAt",json!("effectResolution")),
        ("/abilities/card/0/targetResolvers/1/targetResolver/choices/entries/1/effect/target/path/1",json!("unbound")),
        ("/abilities/card/0/targetResolvers/1/targetResolver/choices/invocationPolicy",json!("onceAtRegistration")),
        ("/abilities/card/0/targetResolvers/1/targetResolver/unknown",json!(false)),
        ("/stateWatchers",json!([{}])),
        ("/abilities/unit",json!([{}])),
    ] {
        let mut changed=original.clone();
        let (parent,key)=path.rsplit_once('/').unwrap();
        let parent=changed.pointer_mut(parent).unwrap();
        if let Some(object)=parent.as_object_mut() {object.insert(key.into(),value);} else {parent[key.parse::<usize>().unwrap()]=value;}
        assert!(prepare(&changed).is_err(),"accepted {path}");
    }
}
fn select(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "missing selected physical card {id}"
    );
    e.apply(s, &action).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let actions = e.legal_actions(s);
    let action = actions
        .iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {actions:?}"));
    e.apply(s, action).unwrap()
}
fn trigger(e: &Engine, actor: Seat) -> (State, CardId) {
    let mut s = board(e);
    s.active = actor;
    s.action_player = Some(actor);
    s.round = 4;
    s.phase_history.exact = true;
    s.phase_history.defeats_exact = true;
    let source = unit(e, &mut s, actor, "LOF_200");
    (s, source)
}
#[test]
fn migration_modal_deck_midgame_owner_choice_order_cleanup_and_no_unit_defeat() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for actor in [Seat::P1, Seat::P2] {
            for owner in [actor, actor.other()] {
                for placement in ["Top", "Bottom"] {
                    let (mut s, source) = trigger(&e, actor);
                    let target = unit(&e, &mut s, owner, "LOF_254");
                    let upgrade = add(&e, &mut s, owner, "IC27_052");
                    e.attach(&mut s, upgrade, target);
                    let first = add(&e, &mut s, owner, "LOF_254");
                    let last = add(&e, &mut s, owner, "LOF_256");
                    s.player_mut(owner).deck.push(first).unwrap();
                    s.player_mut(owner).deck.push(last).unwrap();
                    e.defeat(&mut s, source).unwrap();
                    e.finish(&mut s).unwrap();
                    let before = s.phase_history.defeated_unit_counts;
                    let menu = select(&e, &s, target);
                    assert_eq!(menu.active, owner);
                    assert_eq!(menu.ability_target, target);
                    assert_eq!(
                        e.legal_actions(&menu),
                        vec![Action::button("Top", 0), Action::button("Bottom", 1)]
                    );
                    let done = button(&e, &menu, placement);
                    assert_eq!(
                        done.player(owner).deck.as_slice(),
                        if placement == "Top" {
                            vec![target, first, last]
                        } else {
                            vec![first, last, target]
                        }
                    );
                    assert!(done.player(owner).discard.contains(upgrade));
                    assert!(!done.player(owner).discard.contains(target));
                    assert_eq!(done.card(upgrade).parent, 0);
                    assert_eq!(done.phase_history.defeated_unit_counts, before);
                    assert_eq!(done.ability_program, 0);
                }
            }
        }
    }
}
#[test]
fn migration_modal_deck_optional_none_and_token_leave_game() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for actor in [Seat::P1, Seat::P2] {
            let (mut s, source) = trigger(&e, actor);
            let target = unit(&e, &mut s, actor.other(), "LOF_254");
            e.defeat(&mut s, source).unwrap();
            e.finish(&mut s).unwrap();
            let actions = e.legal_actions(&s);
            let decline = actions
                .iter()
                .find(|a| a.arg == Some(json!("noTarget")))
                .unwrap();
            let done = e.apply(&s, decline).unwrap();
            assert_eq!(done.ability_program, 0);
            assert!(e.in_arena(&done, target));
            for owner in [actor, actor.other()] {
                let (mut s, source) = trigger(&e, actor);
                let token = unit(&e, &mut s, owner, "TOKEN_CLONE_TROOPER");
                e.defeat(&mut s, source).unwrap();
                e.finish(&mut s).unwrap();
                let before = s.phase_history.defeated_unit_counts;
                let menu = select(&e, &s, token);
                let done = button(&e, &menu, "Bottom");
                assert!(!e.in_arena(&done, token));
                assert!(!done.player(owner).deck.contains(token));
                assert!(!done.player(owner).discard.contains(token));
                assert_eq!(done.phase_history.defeated_unit_counts, before);
            }
        }
    }
}

#[test]
fn migration_modal_deck_pending_frames_reject_forged_or_terminal_imports() {
    let e = engine();
    let (mut s, source) = trigger(&e, Seat::P1);
    let target = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.defeat(&mut s, source).unwrap();
    e.finish(&mut s).unwrap();
    for pending in [s, select(&e, &s, target)] {
        let snapshot = e.snapshot(&pending);
        assert_eq!(snapshot["dependentChoicePending"], json!(true));
        assert_eq!(e.legal_actions(&pending), e.legal_actions(&pending.clone()));
        for marker in [json!(true), json!(false), Value::Null] {
            let mut forged = snapshot.clone();
            forged["dependentChoicePending"] = marker;
            forged["actor"] = forged["activePlayer"].clone();
            assert!(e.load_state(&forged).is_err());
            forged["winner"] = json!("p1");
            assert!(e.load_state(&forged).is_err());
        }
    }
}
#[test]
fn migration_modal_deck_capacity_preflight_and_sealed_scalar_boundary() {
    use crate::ability_ir::{DeckPosition, Program};
    let e = engine();
    for full_discard in [false, true] {
        // Synthetic storage diagnostic: a failed bounded move leaves the
        // physical recipient and its attachments untouched in either case.
        let mut s = board(&e);
        let target = unit(&e, &mut s, Seat::P2, "LOF_254");
        let upgrade = add(&e, &mut s, Seat::P2, "IC27_052");
        e.attach(&mut s, upgrade, target);
        for _ in 0..MAX_ZONE {
            let card = add(&e, &mut s, Seat::P2, "LOF_254");
            if full_discard {
                s.players[1].discard.push(card).unwrap();
            } else {
                s.players[1].deck.push(card).unwrap();
            }
        }
        let before = e.snapshot(&s);
        let hash = s.zobrist();
        assert!(e
            .move_arena_unit_to_owner_deck(&mut s, target, DeckPosition::Top)
            .is_err());
        assert_eq!(e.snapshot(&s), before);
        assert_eq!(s.zobrist(), hash);
        assert!(e.in_arena(&s, target));
        assert_eq!(s.card(upgrade).parent, target);
    }
    let raw = json!({"trigger":"whenDefeated","title":"Sealed movement diagnostic","costs":[],"effect":{
        "kind":"moveSelectedArenaUnitToOwnerDeck","target":"self","position":"top"}});
    let ability = serde_json::from_value(raw).unwrap();
    assert!(Program::compile(&ability)
        .unwrap_err()
        .contains("prepared dependent-choice contract"));
}
#[test]
fn migration_modal_deck_departure_does_not_capture_target_when_defeated() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let (mut s, source) = trigger(&e, Seat::P1);
        let target = unit(&e, &mut s, Seat::P2, "LAW_116");
        e.defeat(&mut s, source).unwrap();
        e.finish(&mut s).unwrap();
        let before = s.phase_history.defeated_unit_counts;
        let menu = select(&e, &s, target);
        let done = button(&e, &menu, "Top");
        assert_eq!(done.players[1].deck.as_slice(), &[target]);
        assert_eq!(done.phase_history.defeated_unit_counts, before);
        assert_eq!(e.available_credit_count(&done, Seat::P1), 0);
        assert_eq!(e.available_credit_count(&done, Seat::P2), 0);
        assert_eq!(done.ability_program, 0);
    }
}
