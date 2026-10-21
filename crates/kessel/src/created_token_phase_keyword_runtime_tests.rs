use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(c: &Value) -> Value {
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_028")
        .unwrap()
        .clone()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e
            .by_code
            .get(code)
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))),
        owner,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 3;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.credit_state_exact = e.needs_credit_state();
    for p in [Seat::P1, Seat::P2] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn drain(e: &Engine, s: &mut State) {
    for _ in 0..100 {
        if matches!(s.decision, Decision::Window) {
            return;
        }
        let actions = e.legal_actions(s);
        let a = actions
            .first()
            .unwrap_or_else(|| panic!("missing prompt action {:?}", s.decision));
        *s = e.apply(s, a).unwrap();
    }
    panic!("unfinished captured window {:?}", s.decision);
}
fn mandos(e: &Engine, s: &State, seat: Seat) -> Vec<CardId> {
    s.player(seat)
        .ground
        .as_slice()
        .iter()
        .copied()
        .filter(|id| e.def(s, *id).info.internal_name == "mandalorian")
        .collect()
}

fn paid(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let event = add(e, s, seat, "ASH_091");
    s.player_mut(seat).hand.push(event).unwrap();
    for _ in 0..10 {
        let r = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    let action = e.card_action(s, event);
    assert!(e.legal_actions(s).contains(&action));
    *s = e.apply(s, &action).unwrap();
    event
}
#[test]
fn created_token_phase_keyword_state_budget() {
    assert_eq!(std::mem::size_of::<State>(), 39088);
    assert_eq!(std::mem::size_of::<crate::model::ResolutionContext>(), 112);
}
#[test]
fn created_token_phase_keyword_paid_identity_shield_order_and_backends() {
    let auto = engine();
    let mut interp = engine();
    interp.set_rules_backend(RulesBackend::Interpreter).unwrap();
    for seat in [Seat::P1, Seat::P2] {
        let mut before = board(&auto, seat);
        let old = unit(&auto, &mut before, seat, "TOKEN_MANDALORIAN");
        let shield = add(&auto, &mut before, seat, "TOKEN_SHIELD");
        before.card_mut(old).shield = shield;
        auto.attach(&mut before, shield, old);
        let mut s = before;
        let event = paid(&auto, &mut s, seat);
        let mut t = before;
        assert_eq!(paid(&interp, &mut t, seat), event);
        assert_eq!(s, t);
        assert_eq!(
            s.player(seat)
                .resources
                .as_slice()
                .iter()
                .filter(|id| s.card(**id).exhausted)
                .count(),
            5
        );
        assert!(!s.player(seat).hand.contains(event));
        let ids = mandos(&auto, &s, seat);
        assert_eq!(ids.len(), 2);
        let new = *ids.iter().find(|id| **id != old).unwrap();
        assert!(auto.has_keyword(&s, new, "sentinel"));
        assert!(!auto.has_keyword(&s, old, "sentinel"));
        assert_eq!(s.card(old).shield, shield);
        assert!(new > shield);
        assert_eq!(s.actor(), Some(seat.other()));
        drain(&auto, &mut s);
        drain(&interp, &mut t);
        assert_eq!(s, t);
        assert!(s.card(new).shield > new);
        assert!(s.card(new).exhausted);
        assert!(auto.has_keyword(&s, new, "sentinel"));
        assert_eq!(
            s.event_result.generated_token, 0,
            "completed action clears result"
        );
    }
}
#[test]
fn created_token_phase_keyword_hash_copy_scope_and_suspend() {
    let mut s = State::default();
    s.source = 5;
    s.ability_program = 7;
    s.ability_pc = 1;
    s.event_result = StepEventResult {
        generated_token: 11,
        ..StepEventResult::for_state(&s)
    };
    let h = s.zobrist();
    let c = crate::model::ResolutionContext::capture(&s);
    s.event_result.generated_token = 12;
    assert_ne!(s.zobrist(), h);
    c.restore(&mut s);
    assert_eq!(s.zobrist(), h);
    assert_eq!(s.event_result.generated_token, 11);
    assert!(!s.event_result.belongs_to(5, 8, 1));
}
#[test]
fn created_token_phase_keyword_repeat_expiry_import_and_arena_targets() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let ground = unit(&e, &mut s, seat.other(), "SOR_095");
        let space = unit(&e, &mut s, seat.other(), "SOR_178");
        paid(&e, &mut s, seat);
        drain(&e, &mut s);
        let first = mandos(&e, &s, seat)[0];
        assert_eq!(e.attack_targets(&s, ground, false), vec![first]);
        assert!(e
            .attack_targets(&s, space, false)
            .contains(&s.player(seat).base));
        let pass = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Pass")
            .unwrap();
        s = e.apply(&s, &pass).unwrap();
        assert_eq!(s.active, seat);
        paid(&e, &mut s, seat);
        drain(&e, &mut s);
        let ids = mandos(&e, &s, seat);
        assert_eq!(ids.len(), 2);
        assert!(ids.iter().all(|id| e.has_keyword(&s, *id, "sentinel")));
        let v = e.snapshot(&s);
        let imported = e.load_state(&v).unwrap();
        assert_eq!(e.snapshot(&imported), v);
        assert_eq!(imported.event_result.generated_token, 0);
        s.players[0].out = true;
        s.players[1].out = true;
        e.advance(&mut s).unwrap();
        assert!(ids.iter().all(|id| !e.has_keyword(&s, *id, "sentinel")));
    }
}
#[test]
fn created_token_phase_keyword_departure_and_new_creation_do_not_retarget() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        paid(&e, &mut s, seat);
        drain(&e, &mut s);
        let old = mandos(&e, &s, seat)[0];
        e.leave_play(&mut s, old).unwrap();
        assert_eq!(s.card(old).phase_keyword_presence, 0);
        // Staged native lifecycle boundary; real Waylay source games are separate.
        s.active = seat;
        s.action_player = Some(seat);
        paid(&e, &mut s, seat);
        drain(&e, &mut s);
        let new = mandos(&e, &s, seat)[0];
        assert_ne!(old, new);
        assert!(e.has_keyword(&s, new, "sentinel"));
        assert_eq!(s.card(old).phase_keyword_presence, 0);
    }
}
#[test]
fn created_token_phase_keyword_current_window_capacity_is_atomic() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for slots in [1, 2] {
            let mut s = board(&e, seat);
            let event = add(&e, &mut s, seat, "ASH_091");
            s.player_mut(seat).hand.push(event).unwrap();
            for _ in 0..8 {
                let r = add(&e, &mut s, seat, "LOF_254");
                s.player_mut(seat).resources.push(r).unwrap();
            }
            s.next_id = (MAX_CARDS - slots) as CardId;
            let first = s.next_id;
            let before = s;
            let action = e.card_action(&s, event);
            assert!(e.legal_actions(&s).contains(&action));
            let result = e.apply(&s, &action);
            assert_eq!(s, before);
            if slots == 1 {
                assert!(result.is_err());
            } else {
                let mut done = result.unwrap();
                assert_eq!(mandos(&e, &done, seat), vec![first]);
                assert!(e.has_keyword(&done, first, "sentinel"));
                assert!(done.next_id == first + 1 || done.next_id == first + 2);
                drain(&e, &mut done);
                assert_eq!(done.card(first).shield, first + 1);
            }
        }
    }
}

#[test]
fn created_token_phase_keyword_framework_dead_physical_token_keeps_phase_effect() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        unit(&e, &mut s, seat.other(), "SHD_037");
        let old = unit(&e, &mut s, seat, "TOKEN_MANDALORIAN");
        // Old friendly token in this synthetic position is also framework-lethal;
        // only the newly generated physical recipient gains the lasting keyword.
        let first = s.next_id + 11;
        paid(&e, &mut s, seat);
        drain(&e, &mut s);
        assert!(!e.in_arena(&s, first));
        assert!(e.has_keyword(&s, first, "sentinel"));
        assert_eq!(s.card(first).shield, 0);
        assert!(!e.has_keyword(&s, old, "sentinel"));
        s.players[0].out = true;
        s.players[1].out = true;
        e.advance(&mut s).unwrap();
        assert!(!e.has_keyword(&s, first, "sentinel"));
    }
}

#[test]
fn created_token_phase_keyword_captured_boundary_and_stale_scope_reject() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_091");
        let p = e.card_programs(&s, source)[0];
        s.source = source;
        s.ability_program = p + 1;
        s.ability_pc = 0;
        s.ability_committed = true;
        let Instruction::Group(effect) = e.program(p).instructions[0].clone() else {
            panic!("creation missing")
        };
        assert!(e.generate_group(&mut s, &effect).unwrap());
        assert!(e.resolve_group(&mut s, &effect).unwrap());
        let token = s.event_result.generated_token;
        assert_ne!(token, 0);
        assert_eq!(s.card(token).shield, 0);
        assert!(!e.has_keyword(&s, token, "sentinel"));
        assert!(s.death_pending.len > 0);
        s.event_group = EventGroupState::default();
        s.ability_pc = 1;
        let pending = s.death_pending;
        let mut invalid = s;
        invalid.event_result.program += 1;
        let before = invalid;
        assert!(e.run_created_token_phase_keyword(&mut invalid).is_err());
        assert_eq!(invalid, before);
        assert!(e.run_created_token_phase_keyword(&mut s).unwrap());
        assert!(e.has_keyword(&s, token, "sentinel"));
        assert_eq!(s.card(token).shield, 0);
        assert_eq!(s.death_pending, pending);
        e.run_program(&mut s).unwrap();
        drain(&e, &mut s);
        assert_ne!(s.card(token).shield, 0);
    }
}
