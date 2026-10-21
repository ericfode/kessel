//! Public-position capacity boundaries; original source recordings live outside
//! these durable tests. Synthetic IR mutants exercise fail-closed proof paths.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    e.set_rules_backend(backend).unwrap();
    e
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        owner,
        def: e.by_code[code],
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
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.credit_state_exact = true;
    s.round_limit_wire_present = true;
    for owner in [Seat::P1, Seat::P2] {
        s.player_mut(owner).base = add(e, &mut s, owner, "SOR_023");
        s.player_mut(owner).leader = add(e, &mut s, owner, "SOR_014");
        s.player_mut(owner).force_id = s.next_id;
        s.next_id += 1;
        s.player_mut(owner).force_held = true;
        for _ in 0..12 {
            let id = add(e, &mut s, owner, "LOF_254");
            s.player_mut(owner).resources.push(id).unwrap();
        }
        for _ in 0..4 {
            let id = add(e, &mut s, owner, "LOF_254");
            s.player_mut(owner).deck.push(id).unwrap();
        }
    }
    e.load_state(&e.snapshot(&s)).unwrap()
}
fn saturate(e: &Engine, s: &mut State, free: u16) {
    while s.next_id < MAX_CARDS as u16 - free {
        add(e, s, Seat::P1, "TOKEN_EXPERIENCE");
    }
    *s = e.load_state(&e.snapshot(s)).unwrap();
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(e.legal_actions(s).contains(&a));
    e.apply(s, &a).unwrap()
}
fn fixed(e: &Engine, s: &State) {
    let v = e.snapshot(s);
    let t = e.load_state(&v).unwrap();
    assert_eq!(t, *s);
    assert_eq!(t.zobrist(), s.zobrist());
    assert_eq!(e.snapshot(&t), v);
    assert_eq!(e.legal_actions(&t), e.legal_actions(s));
}
fn unchanged_error(e: &Engine, s: &State, a: &Action, needle: &str) {
    let copy = *s;
    let hash = s.zobrist();
    let json = e.snapshot(s);
    let actions = e.legal_actions(s);
    let err = e.apply(s, a).unwrap_err();
    assert!(err.contains(needle), "{err}");
    assert_eq!(*s, copy);
    assert_eq!(s.zobrist(), hash);
    assert_eq!(e.snapshot(s), json);
    assert_eq!(e.legal_actions(s), actions);
}
fn drain(e: &Engine, mut s: State, last: bool) -> State {
    for _ in 0..80 {
        if !s.has_pending_resolution()
            && matches!(s.decision, Decision::Window | Decision::Terminal)
        {
            return s;
        }
        let actions = e.legal_actions(&s);
        let a = if last {
            actions.last()
        } else {
            actions.first()
        }
        .unwrap_or_else(|| panic!("stuck {:?}", s.decision));
        s = e.apply(&s, a).unwrap();
    }
    panic!("resolution did not drain")
}
#[test]
fn allocation_hera_vane_public_zero_rejects_before_commit_one_free_both_orders_terminal() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for last in [false, true] {
                for free in [0, 1] {
                    let mut s = board(&e, seat);
                    let hera = unit(&e, &mut s, seat, "ASH_031");
                    let vane = unit(&e, &mut s, seat, "ASH_144");
                    let own = s.player(seat).base;
                    let enemy = s.player(seat.other()).base;
                    s.card_mut(own).damage = 5;
                    s.card_mut(enemy).damage = 29;
                    saturate(&e, &mut s, free);
                    fixed(&e, &s);
                    let select = click(&e, &s, hera);
                    let target = e.card_action(&select, enemy);
                    assert!(e.legal_actions(&select).contains(&target));
                    assert!(e.legal_actions(&select).iter().any(|a| a.label == "Cancel"));
                    if free == 0 {
                        unchanged_error(
                            &e,
                            &select,
                            &target,
                            "unsupported attack position: allocation demand",
                        );
                        assert!(!select.card(hera).exhausted);
                        assert_eq!(select.card(enemy).damage, 29);
                    } else {
                        let pending = e.apply(&select, &target).unwrap();
                        assert_eq!(pending.decision, Decision::DeathOrder);
                        let done = drain(&e, pending, last);
                        assert!(done.is_terminal());
                        assert!(done.card(hera).exhausted);
                        assert_eq!(done.card(own).damage, 4);
                        assert_eq!(done.next_id, 256);
                        assert_eq!(e.upgrades(&done, vane).len(), 1);
                        assert!(e.legal_actions(&done).is_empty());
                    }
                }
            }
        }
    }
}
#[test]
fn allocation_friendly_occurrences_and_unrelated_enemy_source_do_not_share_budget() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let attacker = unit(&e, &mut s, seat, "LOF_254");
            unit(&e, &mut s, seat.other(), "ASH_144");
            let enemy = s.player(seat.other()).base;
            saturate(&e, &mut s, 0);
            let select = click(&e, &s, attacker);
            let done = drain(&e, click(&e, &select, enemy), false);
            assert_eq!(done.next_id, 256);
            assert!(done.card(attacker).exhausted);
            fixed(&e, &done);
        }
    }
}
#[test]
fn allocation_all_four_native_attack_roots_reject_before_event_payment_or_optional_attack() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for code in ["ASH_184", "IC27_175", "SOR_168", "SHD_236"] {
                let mut s = board(&e, seat);
                let attacker = unit(&e, &mut s, seat, "LOF_254");
                unit(&e, &mut s, seat, "ASH_144");
                let source = add(&e, &mut s, seat, code);
                s.player_mut(seat).hand.push(source).unwrap();
                saturate(&e, &mut s, 0);
                let mut selected = click(&e, &s, source);
                if code == "SHD_236" {
                    assert!(selected.player(seat).ground.contains(source));
                    if selected.decision == Decision::TriggerOrder {
                        let a = e
                            .legal_actions(&selected)
                            .into_iter()
                            .find(|a| a.label == "Trigger")
                            .unwrap();
                        selected = e.apply(&selected, &a).unwrap();
                    }
                    let decline = e.legal_actions(&selected).into_iter().find(|a| {
                        a.label == "Pass" || a.label == "Pass ability" || a.label == "Decline"
                    });
                    assert!(decline.is_some(), "{:?}", e.legal_actions(&selected));
                }
                assert_eq!(selected.decision, Decision::AbilityTarget, "{code}");
                let action = e.card_action(&selected, attacker);
                assert!(e.legal_actions(&selected).contains(&action));
                unchanged_error(&e, &selected, &action, "unsupported attack position");
                assert!(!selected.card(attacker).exhausted);
                if code != "SHD_236" {
                    assert!(selected.player(seat).hand.contains(source));
                    assert_eq!(
                        e.ready_resources(&selected, seat),
                        e.ready_resources(&s, seat)
                    );
                }
            }
        }
    }
}
#[test]
fn allocation_follow_me_suffix_shares_bound_with_vane_and_defender_alternatives_take_max() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for free in [3, 4] {
                let mut s = board(&e, seat);
                let attacker = unit(&e, &mut s, seat, "LOF_254");
                unit(&e, &mut s, seat, "ASH_144");
                unit(&e, &mut s, seat.other(), "LOF_254");
                unit(&e, &mut s, seat.other(), "LOF_254");
                let event = add(&e, &mut s, seat, "ASH_184");
                s.player_mut(seat).hand.push(event).unwrap();
                saturate(&e, &mut s, free);
                let selected = click(&e, &s, event);
                let a = e.card_action(&selected, attacker);
                if free == 3 {
                    unchanged_error(&e, &selected, &a, "allocation demand 4 exceeds 3");
                } else {
                    let target = e.apply(&selected, &a).unwrap();
                    assert_eq!(target.decision, Decision::AttackTarget);
                    assert!(target.player(seat).discard.contains(event));
                }
            }
        }
    }
}
#[test]
fn allocation_train_paz_mandalorians_shields_and_gideon_are_causal_prior_emissions() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for free in [1, 24] {
                let mut s = board(&e, seat);
                let train = unit(&e, &mut s, seat, "ASH_035");
                let skill = add(&e, &mut s, seat, "SOR_166");
                e.attach(&mut s, skill, train);
                let vane = unit(&e, &mut s, seat, "ASH_144");
                let gideon = unit(&e, &mut s, seat, "SOR_036");
                s.card_mut(gideon).exhausted = true;
                let paz = unit(&e, &mut s, seat.other(), "ASH_028");
                s.card_mut(paz).damage = 3;
                let base = s.player(seat.other()).base;
                saturate(&e, &mut s, free);
                fixed(&e, &s);
                let select = click(&e, &s, train);
                let a = e.card_action(&select, base);
                if free == 1 {
                    unchanged_error(&e, &select, &a, "unsupported attack position");
                    assert!(!select.card(train).exhausted);
                    assert!(e.in_arena(&select, paz));
                } else {
                    let mut t = e.apply(&select, &a).unwrap();
                    for _ in 0..100 {
                        if !t.has_pending_resolution()
                            && matches!(t.decision, Decision::Window | Decision::Terminal)
                        {
                            break;
                        }
                        let actions = e.legal_actions(&t);
                        let choice = if t.decision == Decision::AbilityTarget
                            && actions.contains(&e.card_action(&t, paz))
                        {
                            e.card_action(&t, paz)
                        } else {
                            actions
                                .iter()
                                .find(|a| a.label == "Trigger")
                                .or_else(|| actions.first())
                                .expect("mandatory continuation")
                                .clone()
                        };
                        t = e.apply(&t, &choice).unwrap();
                    }
                    assert!(t.player(seat.other()).discard.contains(paz));
                    let tokens = e
                        .arena_units(&t)
                        .into_iter()
                        .filter(|&id| e.def(&t, id).code == "TOKEN_MANDALORIAN")
                        .collect::<Vec<_>>();
                    assert_eq!(tokens.len(), 2);
                    assert!(tokens.iter().all(|&id| t.card(id).shield != 0));
                    assert!(e
                        .upgrades(&t, vane)
                        .iter()
                        .any(|&id| e.def(&t, id).code == "TOKEN_ADVANTAGE"));
                    assert!(e
                        .arena_units(&t)
                        .iter()
                        .any(|&id| t.card(id).experience != 0));
                    fixed(&e, &t);
                }
            }
        }
    }
}
#[test]
fn allocation_draw_axe_and_repeated_independent_emissions_preserve_multiplicity() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for free in [1, 2] {
                let mut s = board(&e, seat);
                let attacker = unit(&e, &mut s, seat, "LAW_107");
                let vane = unit(&e, &mut s, seat, "ASH_144");
                let axe = unit(&e, &mut s, seat, "ASH_169");
                let base = s.player(seat.other()).base;
                saturate(&e, &mut s, free);
                let select = click(&e, &s, attacker);
                let a = e.card_action(&select, base);
                if free == 1 {
                    unchanged_error(&e, &select, &a, "allocation demand 2 exceeds 1");
                } else {
                    let done = drain(&e, e.apply(&select, &a).unwrap(), false);
                    assert_eq!(done.next_id, 256);
                    assert_eq!(e.upgrades(&done, vane).len(), 1);
                    assert_eq!(e.upgrades(&done, axe).len(), 1);
                    fixed(&e, &done);
                }
            }
        }
    }
}
fn inject(e: &mut Engine, code: &str, trigger: &str, effect: Effect) -> u16 {
    let ability:crate::ability_ir::Ability=serde_json::from_value(json!({"trigger":"whenPlayed","title":"Diagnostic allocation closure","costs":[],"effect":{"kind":"noAction"}})).unwrap();
    let mut p = Program::compile(&ability).unwrap();
    p.instructions = vec![Instruction::Apply(effect), Instruction::EndStep];
    p.trigger = trigger.into();
    let c = e.catalogue.as_mut().unwrap();
    let id = c.programs.len() as u16;
    c.programs.push(p);
    e.definitions[e.by_code[code] as usize]
        .programs
        .card
        .push(id);
    id
}
#[test]
fn allocation_unknown_cycles_and_duplicate_copied_occurrences_reject_atomically() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for shape in ["unknown", "cycle", "duplicate"] {
            let mut e = engine(backend);
            if shape == "unknown" {
                inject(
                    &mut e,
                    "LOF_254",
                    "onAttack",
                    Effect::MaximumPowerExperience {
                        aspect: "command".into(),
                    },
                );
            }
            if shape == "cycle" {
                let draw = Effect::Draw {
                    amount: Expr::number(1),
                    target: Target::name("ownPlayer"),
                };
                inject(&mut e, "LOF_254", "onAttack", draw.clone());
                inject(&mut e, "ASH_144", "onCardsDrawn", draw);
            }
            if shape == "duplicate" {
                let def = e.by_code["ASH_144"] as usize;
                let p = e.definitions[def].programs.card[0];
                e.definitions[def].programs.card.push(p);
            }
            let mut s = board(&e, Seat::P1);
            let attacker = unit(&e, &mut s, Seat::P1, "LOF_254");
            unit(&e, &mut s, Seat::P1, "ASH_144");
            let base = s.player(Seat::P2).base;
            saturate(&e, &mut s, if shape == "duplicate" { 1 } else { 10 });
            let select = click(&e, &s, attacker);
            let a = e.card_action(&select, base);
            unchanged_error(
                &e,
                &select,
                &a,
                if shape == "unknown" {
                    "uncertified allocation closure"
                } else if shape == "cycle" {
                    "causal programme cycle"
                } else {
                    "allocation demand 2 exceeds 1"
                },
            );
        }
    }
}
#[test]
fn allocation_native_roots_with_enough_space_preserve_attack_and_suffix() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for code in ["ASH_184", "IC27_175", "SOR_168", "SHD_236"] {
                let mut s = board(&e, seat);
                let attacker = unit(&e, &mut s, seat, "LOF_254");
                let vane = unit(&e, &mut s, seat, "ASH_144");
                unit(&e, &mut s, seat.other(), "ASH_144");
                let source = add(&e, &mut s, seat, code);
                s.player_mut(seat).hand.push(source).unwrap();
                let free = if code == "ASH_184" { 4 } else { 1 };
                saturate(&e, &mut s, free);
                let selected = click(&e, &s, source);
                assert_eq!(selected.decision, Decision::AbilityTarget);
                let target = click(&e, &selected, attacker);
                assert_eq!(target.decision, Decision::AttackTarget);
                assert!(!target.card(attacker).exhausted);
                let base = target.player(seat.other()).base;
                let done = drain(&e, click(&e, &target, base), false);
                assert!(done.card(attacker).exhausted);
                assert_eq!(done.next_id, 256);
                assert!(!e.upgrades(&done, vane).is_empty());
                if code == "ASH_184" {
                    assert_eq!(
                        (1..done.next_id)
                            .filter(|&id| e.def(&done, id).code == "TOKEN_ADVANTAGE")
                            .count(),
                        4
                    );
                }
                fixed(&e, &done);
            }
        }
    }
}
#[test]
fn allocation_unreachable_programmes_and_hidden_vane_do_not_change_ordinary_positions() {
    let mut e = engine(RulesBackend::Auto);
    inject(
        &mut e,
        "LOF_093",
        "onAttack",
        Effect::MaximumPowerExperience {
            aspect: "command".into(),
        },
    );
    let mut s = board(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_254");
    let hidden = add(&e, &mut s, Seat::P1, "LOF_093");
    s.player_mut(Seat::P1).hand.push(hidden).unwrap();
    let vane = add(&e, &mut s, Seat::P1, "ASH_144");
    s.player_mut(Seat::P1).deck.push(vane).unwrap();
    let base = s.player(Seat::P2).base;
    saturate(&e, &mut s, 0);
    let target = click(&e, &s, attacker);
    let done = click(&e, &target, base);
    assert!(done.card(attacker).exhausted);
    assert_eq!(done.next_id, 256);
    fixed(&e, &done);
}
#[test]
fn allocation_ineffective_native_event_use_anyway_keeps_real_payment_without_suffix() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let vane = unit(&e, &mut s, seat, "ASH_144");
            s.card_mut(vane).exhausted = true;
            let source = add(&e, &mut s, seat, "ASH_184");
            s.player_mut(seat).hand.push(source).unwrap();
            saturate(&e, &mut s, 0);
            let selected = click(&e, &s, source);
            let a = e
                .legal_actions(&selected)
                .into_iter()
                .find(|a| a.label == "Play anyway")
                .unwrap_or_else(|| panic!("{:?}", e.legal_actions(&selected)));
            let before = e.ready_resources(&selected, seat);
            let done = e.apply(&selected, &a).unwrap();
            assert!(done.player(seat).discard.contains(source));
            assert!(e.ready_resources(&done, seat) < before);
            assert_eq!(done.next_id, 256);
            assert!(e.upgrades(&done, vane).is_empty());
            fixed(&e, &done);
        }
    }
}
#[test]
fn allocation_defense_creation_and_unique_death_draw_axe_are_in_same_attack_bound() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for code in ["TWI_049", "LAW_121", "ASH_119"] {
                for free in [1, 2] {
                    let mut s = board(&e, seat);
                    let attacker = unit(&e, &mut s, seat, "HMW_093");
                    s.card_mut(attacker).damage = 1;
                    let over = add(&e, &mut s, seat, "ASH_181");
                    e.attach(&mut s, over, attacker);
                    unit(&e, &mut s, seat, "ASH_144");
                    let victim = unit(&e, &mut s, seat.other(), code);
                    if code == "ASH_119" {
                        unit(&e, &mut s, seat, "SOR_115");
                        unit(&e, &mut s, seat, "ASH_169");
                    }
                    saturate(&e, &mut s, free);
                    let select = click(&e, &s, attacker);
                    let target = e.card_action(&select, victim);
                    if free == 1 {
                        unchanged_error(&e, &select, &target, "unsupported attack position");
                    } else {
                        let done = drain(&e, e.apply(&select, &target).unwrap(), false);
                        assert_eq!(done.next_id, 256, "{code}");
                        assert!(done.card(attacker).exhausted);
                        fixed(&e, &done);
                    }
                }
            }
        }
    }
}

// Regression from the independent V1 review: these are unmodified official
// programmes in a public fixedpoint, not a synthetic IR override. Discarding
// the last card must be visible to the operation's optional Draw capacity arm.
#[test]
fn allocation_discard_prefix_kallus_fatigue_blade_counts_both_peer_orders() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for free in [1, 2] {
                for kallus_first in [false, true] {
                    let mut s = board(&e, seat);
                    while s.player(seat).deck.len > 1 {
                        let id = s.player_mut(seat).deck.pop_front().unwrap();
                        s.player_mut(seat).hand.push(id).unwrap();
                    }
                    let last = s.player(seat).deck.as_slice()[0];
                    let attacker = unit(&e, &mut s, seat, "LAW_192");
                    s.card_mut(attacker).damage = 1;
                    let over = add(&e, &mut s, seat, "ASH_181");
                    e.attach(&mut s, over, attacker);
                    let vane = unit(&e, &mut s, seat, "ASH_144");
                    unit(&e, &mut s, seat, "SOR_115");
                    let blade = unit(&e, &mut s, seat, "ASH_204");
                    let defender = unit(&e, &mut s, seat.other(), "ASH_119");
                    saturate(&e, &mut s, free);
                    fixed(&e, &s);
                    let select = click(&e, &s, attacker);
                    let action = e.card_action(&select, defender);
                    assert!(e.legal_actions(&select).contains(&action));
                    if free == 1 {
                        assert!(e.apply(&select,&action).is_err(),"V1 certificate accepted one slot although discard -> optional Draw -> fatigue/Blade + Vane needs two");
                        unchanged_error(&e, &select, &action, "allocation demand 2 exceeds 1");
                        assert!(!select.card(attacker).exhausted);
                        assert_eq!(select.player(seat).deck.len, 1);
                        assert!(!select.player(seat).discard.contains(last));
                        assert!(e.in_arena(&select, defender));
                    } else {
                        let mut q = e.apply(&select, &action).unwrap();
                        let copy = e.apply(&select, &action).unwrap();
                        assert_eq!(q, copy);
                        assert_eq!(q.zobrist(), copy.zobrist());
                        for _ in 0..100 {
                            if !q.has_pending_resolution()
                                && matches!(q.decision, Decision::Window | Decision::Terminal)
                            {
                                break;
                            }
                            let actions = e.legal_actions(&q);
                            let a = actions
                                .iter()
                                .find(|a| {
                                    if kallus_first {
                                        a.label == "Draw a card"
                                    } else {
                                        a.label.contains("Advantage") && a.label.contains("Vane")
                                    }
                                })
                                .or_else(|| actions.iter().find(|a| a.label == "Trigger"))
                                .or_else(|| actions.iter().find(|a| !a.label.starts_with("Pass")))
                                .expect("mandatory continuation");
                            q = e.apply(&q, a).unwrap();
                        }
                        assert!(q.is_action_window());
                        assert!(
                            q.player(seat).discard.contains(attacker),
                            "the printed 3HP attacker also dies to Greef after its existing wound"
                        );
                        assert!(q.player(seat.other()).discard.contains(defender));
                        assert_eq!(q.player(seat).deck.len, 0);
                        assert!(q.player(seat).discard.contains(last));
                        assert_eq!(q.card(q.player(seat).base).damage, 3);
                        assert_eq!(q.next_id, 256);
                        assert_eq!(e.upgrades(&q, vane).len(), 1);
                        assert_eq!(e.upgrades(&q, blade).len(), 1);
                        fixed(&e, &q);
                    }
                }
            }
        }
    }
}

// Defensive typed-programme controls, not a claim that the vanilla host's raw
// source gained abilities: two independent registrations admit either order.
fn inject_compiled_control(e: &mut Engine, code: &str, title: &str, effect: Effect) {
    let a: crate::ability_ir::Ability = serde_json::from_value(
        json!({"trigger":"whenPlayed","title":title,"costs":[],"effect":effect}),
    )
    .unwrap();
    let mut p = Program::compile(&a).unwrap();
    p.trigger = "onAttack".into();
    let c = e.catalogue.as_mut().unwrap();
    let id = c.programs.len() as u16;
    c.programs.push(p);
    e.definitions[e.by_code[code] as usize]
        .programs
        .card
        .push(id);
}
#[test]
fn allocation_peer_depletion_orders_and_two_missing_cards_reserve_one_fatigue_event() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            for amount in [1, 2] {
                for discard_first in [false, true] {
                    for free in [1, 2] {
                        let mut e = engine(backend);
                        inject_compiled_control(
                            &mut e,
                            "LOF_254",
                            "Draw peer",
                            Effect::Draw {
                                target: Target::name("ownPlayer"),
                                amount: Expr::number(amount),
                            },
                        );
                        inject_compiled_control(
                            &mut e,
                            "LOF_254",
                            "Deplete peer",
                            Effect::DiscardDeck {
                                target: Target::name("ownPlayer"),
                                amount: Expr::number(1),
                            },
                        );
                        let mut s = board(&e, seat);
                        while s.player(seat).deck.len > 1 {
                            let id = s.player_mut(seat).deck.pop_front().unwrap();
                            s.player_mut(seat).hand.push(id).unwrap();
                        }
                        let attacker = unit(&e, &mut s, seat, "LOF_254");
                        let vane = unit(&e, &mut s, seat, "ASH_144");
                        let blade = unit(&e, &mut s, seat, "ASH_204");
                        let base = s.player(seat.other()).base;
                        saturate(&e, &mut s, free);
                        let target = click(&e, &s, attacker);
                        let action = e.card_action(&target, base);
                        if free == 1 {
                            unchanged_error(&e, &target, &action, "allocation demand 2 exceeds 1");
                        } else {
                            let mut q = e.apply(&target, &action).unwrap();
                            assert_eq!(q.decision, Decision::TriggerOrder);
                            let first = if discard_first {
                                "Deplete peer"
                            } else {
                                "Draw peer"
                            };
                            let a = e
                                .legal_actions(&q)
                                .into_iter()
                                .find(|a| a.label == first)
                                .unwrap();
                            q = e.apply(&q, &a).unwrap();
                            q = drain(&e, q, false);
                            let missing = if discard_first { amount } else { amount - 1 };
                            assert_eq!(q.card(q.player(seat).base).damage, 3 * missing as u16);
                            assert_eq!(e.upgrades(&q, blade).len(), usize::from(missing > 0));
                            assert_eq!(e.upgrades(&q, vane).len(), 1);
                            assert_eq!(q.next_id, 254 + 1 + u16::from(missing > 0));
                            fixed(&e, &q);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn allocation_no_vane_activation_keeps_depletion_and_draw_position_semantics() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for with_blade in [false, true] {
                let mut s = board(&e, seat);
                while s.player(seat).deck.len > 1 {
                    let id = s.player_mut(seat).deck.pop_front().unwrap();
                    s.player_mut(seat).hand.push(id).unwrap();
                }
                let attacker = unit(&e, &mut s, seat, "LAW_192");
                s.card_mut(attacker).damage = 1;
                let over = add(&e, &mut s, seat, "ASH_181");
                e.attach(&mut s, over, attacker);
                unit(&e, &mut s, seat, "SOR_115");
                let blade = if with_blade {
                    Some(unit(&e, &mut s, seat, "ASH_204"))
                } else {
                    None
                };
                let defender = unit(&e, &mut s, seat.other(), "ASH_119");
                saturate(&e, &mut s, u16::from(with_blade));
                let select = click(&e, &s, attacker);
                let mut q = click(&e, &select, defender);
                for _ in 0..100 {
                    if !q.has_pending_resolution()
                        && matches!(q.decision, Decision::Window | Decision::Terminal)
                    {
                        break;
                    }
                    let actions = e.legal_actions(&q);
                    let a = actions
                        .iter()
                        .find(|a| a.label == "Draw a card")
                        .or_else(|| actions.iter().find(|a| a.label == "Trigger"))
                        .or_else(|| actions.iter().find(|a| !a.label.starts_with("Pass")))
                        .unwrap();
                    q = e.apply(&q, a).unwrap();
                }
                assert!(q.is_action_window());
                assert!(q.player(seat).discard.contains(attacker));
                assert_eq!(q.next_id, 256);
                if let Some(blade) = blade {
                    assert_eq!(e.upgrades(&q, blade).len(), 1);
                }
                assert_eq!(q.card(q.player(seat).base).damage, 3);
                fixed(&e, &q);
            }
        }
    }
}
