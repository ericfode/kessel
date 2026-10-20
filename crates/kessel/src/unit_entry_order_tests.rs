//! Pinned PlayUnitAction emits UnitEntersPlay before CardPlayed. Expectations
//! are also witnessed by the frozen both-seat Outcast/Flock interaction games.
use super::*;

use std::sync::OnceLock;

fn committed_catalogue() -> &'static serde_json::Value {
    static CATALOG: OnceLock<serde_json::Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .expect("committed compiler catalogue")
    })
}
fn committed_catalogue_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES
        .get_or_init(|| serde_json::to_vec(committed_catalogue()).expect("serialize catalogue"))
        .as_slice()
}
fn engine() -> Engine {
    Engine::from_catalogue_bytes(committed_catalogue_bytes()).unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e
            .by_code
            .get(code)
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))),
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine) -> State {
    let mut s = State::default();
    // This builder creates a fresh position with an explicitly empty Credit ledger.
    s.credit_state_exact = e.needs_credit_state();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (seat, leader) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, leader);
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
/// Adds physical resource cards to a player's zone in the given ready/exhausted
/// order and returns them, so exhaustion order is observable.
fn resources(e: &Engine, s: &mut State, seat: Seat, order: &[bool]) -> Vec<CardId> {
    let mut ids = vec![];
    for exhausted in order {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
        s.card_mut(id).exhausted = *exhausted;
        ids.push(id);
    }
    ids
}
fn deck_card(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "LOF_254");
    s.player_mut(seat).deck.push(id).unwrap();
    id
}

#[test]
fn ordinary_entry_event_precedes_when_played_and_both_resolution_orders_retain_dead_target() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for entry_first in [true, false] {
            let mut s = board(&e);
            s.active = seat;
            s.action_player = Some(seat);
            let flock = add(&e, &mut s, seat, "LOF_255");
            s.player_mut(seat).hand.push(flock).unwrap();
            let observer = unit(&e, &mut s, seat, "ASH_041");
            unit(&e, &mut s, seat.other(), "SHD_037");
            resources(&e, &mut s, seat, &[false; 10]);
            let observer_power = e.power(&s, observer, false);
            let pending = e.apply(&s, &e.card_action(&s, flock)).unwrap();
            assert!(pending.player(seat).discard.contains(flock));
            assert_eq!(pending.decision, Decision::DeathOrder);
            let menu = e.legal_actions(&pending);
            assert_eq!(menu.len(), 2);
            assert_eq!(
                menu[0],
                Action::button("Give Curious Flock +1/+0 for this phase", "0")
            );
            assert_eq!(menu[1],Action::button("Pay up to 6 resources. For each resource paid this way, give an Experience token to this unit","1"));
            let mut state = e.apply(&pending, &menu[usize::from(!entry_first)]).unwrap();
            for _ in 0..12 {
                if state.is_action_window() {
                    break;
                }
                let actions = e.legal_actions(&state);
                let chosen = if state.decision == Decision::AbilityNumber {
                    actions.iter().find(|a| a.arg == Some(json!("3"))).unwrap()
                } else {
                    assert_eq!(actions.len(), 1, "{:?}: {:?}", state.decision, actions);
                    &actions[0]
                };
                state = e.apply(&state, chosen).unwrap();
            }
            assert!(state.is_action_window());
            assert_eq!(e.ready_resources(&state, seat), 6);
            assert_eq!(state.next_id, pending.next_id);
            assert_eq!(state.card(flock).inactive_entry_power, 1);
            assert!(e.upgrades(&state, flock).is_empty());
            assert_eq!(e.power(&state, observer, false), observer_power);
            assert_eq!(state.selected_number, 0);
            let snapshot = e.snapshot(&state);
            assert!(snapshot.get("numberSelectionPending").is_none());
            assert_eq!(
                snapshot["inactiveUnitEntryStats"][0]["card"]["id"],
                json!(format!("c{}", flock - 1))
            );
            assert_eq!(snapshot["inactiveUnitEntryStats"][0]["power"], json!(1));
        }
    }
}
#[test]
fn entry_merge_only_promotes_the_current_ordinary_event_boundary() {
    let e = engine();
    let mut s = board(&e);
    let observer = unit(&e, &mut s, Seat::P1, "ASH_041");
    let source = unit(&e, &mut s, Seat::P1, "LOF_255");
    let other = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.source = source;
    s.trigger_context = TriggerContext::Enter.raw();
    e.queue_programs(&mut s, &["whenPlayed"]);
    let program = e.card_programs(&s, observer)[0];
    let current = DeathTrigger {
        event_card: source,
        source: observer,
        program,
        player: Seat::P1,
        ..DeathTrigger::default()
    };
    let unrelated = DeathTrigger {
        event_card: other,
        ..current
    };
    let future = DeathTrigger {
        capture: crate::defeat_cause::TriggerCapture::new(1,crate::defeat_cause::DefeatCause::Unknown),
        ..current
    };
    let later_step = DeathTrigger {
        after_pc: 1,
        ..current
    };
    s.death_pending.entries[..4].copy_from_slice(&[unrelated, current, future, later_step]);
    s.death_pending.len = 4;
    e.merge_enter_deaths(&mut s).unwrap();
    assert_eq!(s.death_pending.entries[0], current);
    assert_eq!(s.death_pending.entries[1].source, source);
    assert_eq!(s.death_pending.entries[2], unrelated);
    assert_eq!(s.death_pending.entries[3], future);
    assert_eq!(s.death_pending.entries[4], later_step);
    // Leaders and native tokens have different source event layouts. The
    // ordinary unit-play proof must not reorder either of those windows.
    let leader = s.player(Seat::P1).leader;
    let token = unit(&e, &mut s, Seat::P1, "TOKEN_MANDALORIAN");
    for incoming in [leader, token] {
        let mut other = s;
        other.source = incoming;
        other.pending_programs = 0;
        other.pending_triggers = T_SHIELD;
        let capture = DeathTrigger {
            event_card: incoming,
            ..current
        };
        other.death_pending = DeathWindow::default();
        other.death_pending.entries[0] = capture;
        other.death_pending.len = 1;
        e.merge_enter_deaths(&mut other).unwrap();
        assert_eq!(other.death_pending.entries[0].keyword, T_SHIELD);
        assert_eq!(other.death_pending.entries[1], capture);
    }
}
