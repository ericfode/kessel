//! Tests against pinned number/payment source semantics and raw admission boundaries.
use super::*;
use crate::ability_ir::Catalogue;
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

fn play(e: &Engine, s: &State, id: CardId) -> State {
    e.apply(s, &e.card_action(s, id)).unwrap()
}
fn settle(e: &Engine, mut s: State) -> State {
    for _ in 0..20 {
        if s.decision == Decision::AbilityNumber || s.is_action_window() || s.is_terminal() {
            return s;
        }
        let actions = e.legal_actions(&s);
        let action = actions
            .iter()
            .find(|a| a.arg.as_ref().is_some_and(|v| v == "all"))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| a.arg.as_ref().is_some_and(|v| v == "trigger"))
            })
            .or_else(|| {
                if actions.len() == 1 {
                    actions.first()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("unhandled {:?}: {:?}", s.decision, actions));
        s = e.apply(&s, action).unwrap();
    }
    panic!("settle overflow")
}
#[test]
fn native_number_payment_range_order_and_same_count_xp_both_seats_and_backends() {
    for backend in [
        RulesBackend::Interpreter,
        RulesBackend::Auto,
        RulesBackend::Aot,
    ] {
        let full = engine();
        let mut pool = full.compiled_pool().unwrap();
        pool["cards"].as_array_mut().unwrap().retain(|c| {
            [
                "ASH_019",
                "SOR_014",
                "IBH_053",
                "LOF_254",
                "LOF_255",
                "TOKEN_EXPERIENCE",
                "TOKEN_SHIELD",
                "TOKEN_THE_FORCE",
            ]
            .contains(&c["setCode"].as_str().unwrap())
        });
        let codes = pool["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["setCode"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let mut catalogue = committed_catalogue().clone();
        catalogue["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|e| codes.iter().any(|code| e["setCode"] == *code));
        let mut e = Engine::from_compiled_pool_bytes(
            &serde_json::to_vec(&pool).unwrap(),
            &serde_json::to_vec(&catalogue).unwrap(),
        )
        .unwrap();
        e.set_rules_backend(backend).unwrap();
        e.set_rules_instrumentation(true);
        for seat in [Seat::P1, Seat::P2] {
            for (ready, n) in [(10, 0), (10, 1), (10, 3), (10, 6), (1, 0), (3, 2), (7, 6)] {
                let mut s = board(&e);
                s.active = seat;
                s.action_player = Some(seat);
                let source = add(&e, &mut s, seat, "LOF_255");
                s.player_mut(seat).hand.push(source).unwrap();
                // Already exhausted gaps prove cost and effect use physical ready-list order.
                let mut order = vec![false; ready];
                order.insert(0, true);
                order.insert(2, true);
                let ids = resources(&e, &mut s, seat, &order);
                let pending = settle(&e, play(&e, &s, source));
                assert_eq!(
                    e.ready_resources(&pending, seat),
                    ready - 1,
                    "ordinary card cost precedes range"
                );
                assert!(
                    pending.card(ids[1]).exhausted,
                    "ordinary cost first ready resource"
                );
                let done = if ready == 1 {
                    assert!(pending.is_action_window());
                    pending
                } else {
                    assert_eq!(pending.decision, Decision::AbilityNumber);
                    let actions = e.legal_actions(&pending);
                    assert_eq!(actions.len(), (ready - 1).min(6) + 1);
                    for (i, a) in actions.iter().enumerate() {
                        assert_eq!(a.kind, "option");
                        assert_eq!(a.label, format!("#{i}"));
                        assert_eq!(a.arg, Some(json!(i.to_string())));
                    }
                    assert!(e
                        .load_state(&e.snapshot(&pending))
                        .unwrap_err()
                        .contains("pending number"));
                    let action = actions
                        .into_iter()
                        .find(|a| a.arg == Some(json!(n.to_string())))
                        .unwrap();
                    settle(&e, e.apply(&pending, &action).unwrap())
                };
                assert!(done.is_action_window());
                assert_eq!(done.selected_number, 0);
                assert_eq!(e.ready_resources(&done, seat), ready - 1 - n);
                let before_ready = ids
                    .iter()
                    .filter(|id| !pending.card(**id).exhausted)
                    .copied()
                    .collect::<Vec<_>>();
                for (i, id) in before_ready.iter().enumerate() {
                    assert_eq!(done.card(*id).exhausted, i < n);
                }
                assert_eq!(e.power(&done, source, false), 1 + n as u16);
                assert_eq!(e.hp(&done, source), 1 + n as u16);
                assert_eq!(e.upgrades(&done, source).len(), n);
                assert_eq!(done.next_id, pending.next_id + n as u16);
                let program = e.card_programs(&done, source)[0] as usize;
                let hits = e.rules_hits.lock().unwrap()[&program];
                if backend == RulesBackend::Interpreter {
                    assert!(hits[1] > 0);
                    assert_eq!(hits[0], 0);
                } else {
                    assert!(hits[0] > 0);
                    assert_eq!(hits[1], 0);
                }
            }
        }
    }
}
#[test]
fn captured_when_played_pays_after_snoke_kills_source_and_creates_no_xp() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.active = seat;
        s.action_player = Some(seat);
        unit(&e, &mut s, seat.other(), "SHD_037");
        let source = add(&e, &mut s, seat, "LOF_255");
        s.player_mut(seat).hand.push(source).unwrap();
        resources(&e, &mut s, seat, &[false; 10]);
        let pending = settle(&e, play(&e, &s, source));
        assert_eq!(pending.decision, Decision::AbilityNumber);
        assert!(pending.player(seat).discard.contains(source));
        assert!(!e.in_arena(&pending, source));
        let before = pending.next_id;
        let a = e
            .legal_actions(&pending)
            .into_iter()
            .find(|a| a.arg == Some(json!("3")))
            .unwrap();
        let done = settle(&e, e.apply(&pending, &a).unwrap());
        assert_eq!(e.ready_resources(&done, seat), 6);
        assert_eq!(done.next_id, before);
        assert!(e.upgrades(&done, source).is_empty());
    }
}
#[test]
fn numeric_binding_copy_hash_context_restore_and_atomic_invalid_choices() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_255");
    s.source = source;
    resources(&e, &mut s, Seat::P1, &[false; 3]);
    let p = e.card_programs(&s, source)[0];
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityNumber);
    let before = s;
    for invalid in [
        json!("-1"),
        json!("4"),
        json!("01"),
        json!("1.5"),
        json!(1),
        json!("cancel"),
    ] {
        let a = Action {
            kind: "option".into(),
            card: None,
            arg: Some(invalid),
            method: None,
            label: "#1".into(),
        };
        assert!(e.apply(&s, &a).is_err());
        assert_eq!(s, before);
    }
    let mut capacity = s;
    capacity.next_id = (MAX_CARDS - 1) as u16;
    let before_capacity = capacity;
    let a = e
        .legal_actions(&capacity)
        .into_iter()
        .find(|a| a.arg == Some(json!("3")))
        .unwrap();
    assert!(e.apply(&capacity, &a).is_err());
    assert_eq!(capacity, before_capacity);
    let mut wrong_actor = s;
    wrong_actor.active = Seat::P2;
    assert!(e.apply_number_action(&wrong_actor, &a).is_err());
    let mut missing_marker = e.snapshot(&s);
    missing_marker
        .as_object_mut()
        .unwrap()
        .remove("numberSelectionPending");
    assert!(
        e.load_state(&missing_marker).is_err(),
        "actor-null prompt cannot silently import as action state"
    );
    let mut bound = s;
    bound.selected_number = 4;
    bound.entry_event_card = source;
    let hash = bound.zobrist();
    assert_ne!(hash, s.zobrist());
    let mut copy = bound;
    assert_eq!(copy.zobrist(), hash);
    let context = ResolutionContext::capture(&copy);
    copy.selected_number = 1;
    copy.entry_event_card = 0;
    context.restore(&mut copy);
    assert_eq!(copy.selected_number, 4);
    assert_eq!(copy.entry_event_card, source, "Outcast physical event binding coexists with the number binding");
    assert_eq!(copy.zobrist(), hash);
    for slot in 0..MAX_DEATH_DEPTH {
        let mut other = State::default();
        let zero = other.zobrist();
        other.death_frames[slot].context.selected_number = 1;
        assert_ne!(zero, other.zobrist());
    }
    let mut support = State::default();
    support.support.pending = true;
    let zero = support.zobrist();
    support.support.context.selected_number = 1;
    assert_ne!(zero, support.zobrist());
    bound.ability_pc = 1;
    for id in bound.player(Seat::P1).resources.as_slice().to_vec() {
        bound.card_mut(id).exhausted = true;
    }
    let before = bound;
    assert!(e.run_number_instruction(&mut bound).is_err());
    assert_eq!(bound, before);
    println!(
        "State={} Context={} Frame={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<ResolutionContext>(),
        std::mem::size_of::<DeathFrame>()
    );
}
#[test]
fn exact_source_and_all_entry_boundaries_reject_mutations_and_other_number_families() {
    let c = committed_catalogue();
    let loaded = Catalogue::load(c).unwrap();
    assert!(loaded.cards.contains_key("LOF_255"));
    assert!(loaded.rejected.contains_key("HMW_036"));
    // The target-capturing event is a distinct complete recipe; it must never
    // be treated as this bounded self-XP primitive or as generic callback IR.
    assert!(loaded.cards.contains_key("SEC_040"));
    let event=c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="SEC_040").unwrap();
    assert_eq!(crate::resource_number_experience_contract::prepare(event),Ok(true));
    assert!(crate::variable_payment_contract::prepare(event).is_err());
    let mut near=event.clone();near["abilities"]["card"][0]["then"]["body"]["effect"]["effects"][0]["allowCostAdjustments"]=json!(true);
    assert!(crate::resource_number_experience_contract::prepare(&near).is_err());
    let mut altered=c.clone();let at=altered["cards"].as_array().unwrap().iter().position(|e|e["setCode"]=="SEC_040").unwrap();altered["cards"][at]=near;let rejected=Catalogue::load(&altered).unwrap();assert!(rejected.rejected.contains_key("SEC_040"));assert!(rejected.cards.contains_key("LOF_255"));assert!(rejected.rejected.contains_key("HMW_036"));
    let entry = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "LOF_255")
        .unwrap();
    let mutations = vec![
        ("/baseClass", json!("LeaderUnitCard")),
        ("/card/types", json!(["unit", "upgrade"])),
        ("/card/keywords", json!(["shielded"])),
        ("/abilities/card/0/optional", json!(true)),
        ("/abilities/card/0/unknown", json!(true)),
        ("/abilities/card/0/targetResolver/min/value", json!(-1)),
        (
            "/abilities/card/0/targetResolver/max/args/0/value",
            json!(7),
        ),
        (
            "/abilities/card/0/targetResolver/chooser/value",
            json!("opponent"),
        ),
        (
            "/abilities/card/0/targetResolver/resultBinding",
            json!("other"),
        ),
        (
            "/abilities/card/0/effect/effects/0/allowCostAdjustments",
            json!(true),
        ),
        ("/abilities/card/0/effect/effects/0/isCost", json!(false)),
        (
            "/abilities/card/0/effect/effects/0/target",
            json!("enemyPlayer"),
        ),
        ("/abilities/card/0/effect/effects/1/target", json!("chosen")),
        (
            "/abilities/card/0/effect/effects/1/amount/args/0/path",
            json!(["event", "lastResult"]),
        ),
    ];
    for (path, value) in mutations {
        let mut bad = entry.clone();
        let pointer = bad.pointer_mut(path);
        if let Some(p) = pointer {
            *p = value;
        } else {
            bad["abilities"]["card"][0][path.rsplit('/').next().unwrap()] = value;
        }
        assert!(
            crate::variable_payment_contract::prepare(&bad).is_err(),
            "{path}"
        );
        let mut changed = c.clone();
        *changed["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["setCode"] == "LOF_255")
            .unwrap() = bad;
        let cat = Catalogue::load(&changed).unwrap();
        assert!(cat.rejected.contains_key("LOF_255"), "{path}");
        let engine = Engine::from_catalogue_value(&changed).unwrap();
        assert!(!engine.by_code.contains_key("LOF_255"), "{path}");
    }
    // Native Experience is mandatory in both standalone and selected-pool loads.
    let mut missing = c.clone();
    missing["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["setCode"] != "TOKEN_EXPERIENCE");
    let e = Engine::from_catalogue_value(&missing).unwrap();
    assert!(!e.by_code.contains_key("LOF_255"));
    let e = engine();
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["setCode"] != "TOKEN_EXPERIENCE");
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        committed_catalogue_bytes()
    )
    .is_err());
}
