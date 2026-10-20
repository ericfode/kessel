//! Expectations come from the actual pinned ASH_151 plays retained in the
//! source design and mirrored by the reference fixture. These tests exercise
//! shared Rust execution; engine/native/Wasm differential evidence is separate.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_151")
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
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards())),
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
    s.credit_state_exact = e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.arena_order_exact = true;
    for p in [seat, seat.other()] {
        let b = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).base = b;
        let l = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).leader = l;
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..20 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label}: {:?}", e.legal_actions(s)));
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn play(e: &Engine, s: &State) -> State {
    let mut s = *s;
    let actor = s.active;
    let id = add(e, &mut s, actor, "ASH_151");
    s.player_mut(actor).hand.push(id).unwrap();
    choose(e, &s, id)
}
fn admits(v: Value) -> bool {
    crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
        .unwrap()
        .cards
        .contains_key("ASH_151")
}
#[test]
fn cinder_closed_source_and_program_reject_malformed_or_partial_cards() {
    let original = entry();
    assert!(crate::cinder_contract::entry(&original));
    assert!(admits(original.clone()));
    let mut renamed = original.clone();
    renamed["setCode"] = json!("TEST_SAME_WINDOWS");
    renamed["internalName"] = json!("same-source-semantics");
    assert!(
        crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[renamed]}))
            .unwrap()
            .cards
            .contains_key("TEST_SAME_WINDOWS"),
        "structural semantics must not depend on selected card ID"
    );

    let a: crate::ability_ir::Ability =
        serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
    let p = crate::ability_ir::Program::compile(&a).unwrap();
    assert!(crate::cinder_contract::requires_global_order(&p));
    assert_eq!(p.instructions.len(), 3);
    for (pointer, value) in [
        ("/baseClass", json!("NonLeaderUnitCard")),
        ("/card/types", json!(["event", "unit"])),
        ("/card/keywords", json!(["support"])),
        ("/keywordValues", json!({"plot":1})),
        ("/card/cost", json!(7)),
        ("/abilities/card/0/effect/target", json!("enemyBase")),
        ("/abilities/card/0/effect/amount/value", json!(0)),
        ("/abilities/card/0/then/effect/amount/value", json!(4)),
        (
            "/abilities/card/0/then/effect/target/receiver/path",
            json!(["player"]),
        ),
        (
            "/abilities/card/0/then/effect/target/predicate/value",
            json!(false),
        ),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(!admits(bad), "{pointer}");
    }
    for field in [
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "stateWatchers",
    ] {
        let mut bad = original.clone();
        bad[field] = json!([{"unreviewed":true}]);
        assert!(!admits(bad), "{field}");
    }
    let mut nested_extra = original.clone();
    nested_extra["abilities"]["card"][0]["then"]["unknown"] = json!(true);
    assert!(!admits(nested_extra));
    let mut mixed = original.clone();
    let companion = mixed["abilities"]["card"][0].clone();
    mixed["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .push(companion);
    assert!(!admits(mixed));
    for field in ["optional", "extra", "coordinate"] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][field] = json!(true);
        assert!(!admits(bad), "{field}");
    }
    let mut bad = original.clone();
    bad["abilities"]["unit"] = json!([bad["abilities"]["card"][0].clone()]);
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["ifYouDo"] = bad["abilities"]["card"][0]["then"].take();
    bad["abilities"]["card"][0]
        .as_object_mut()
        .unwrap()
        .remove("then");
    assert!(!admits(bad));
    let mut scalar_fallback = original.clone();
    scalar_fallback["abilities"]["card"][0]["then"]["effect"]["target"] = json!("enemyBase");
    assert!(
        !admits(scalar_fallback),
        "malformed global leaf cannot fall through scalar admission"
    );
    let mut bad = original;
    bad["abilities"]["card"][0]["then"]["effect"]["amount"] =
        json!({"kind":"ref","path":["source","power"]});
    assert!(!admits(bad));
}
#[test]
fn cinder_empty_arena_still_damages_only_own_base_and_needs_exact_order() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let s = board(&e, seat);
        let out = play(&e, &s);
        assert_eq!(out.card(out.player(seat).base).damage, 5);
        assert_eq!(out.card(out.player(seat.other()).base).damage, 0);
        let mut missing = s;
        missing.arena_order_exact = false;
        let source = add(&e, &mut missing, seat, "ASH_151");
        missing.player_mut(seat).hand.push(source).unwrap();
        assert!(e.apply(&missing, &e.card_action(&missing, source)).is_err());
    }
}
#[test]
fn cinder_uniform_group_captures_order_and_shield_identity() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P2, "SOR_095");
    let shielded = unit(&e, &mut s, Seat::P1, "LAW_253");
    let b = unit(&e, &mut s, Seat::P1, "SOR_095");
    let c = unit(&e, &mut s, Seat::P2, "LAW_253");
    s.source = shielded;
    e.apply_effect(
        &mut s,
        &serde_json::from_value(
            json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":2}}),
        )
        .unwrap(),
    )
    .unwrap();
    let before = e.upgrades(&s, shielded);
    s.source = 0;
    let p = crate::ability_ir::Program::compile(
        &serde_json::from_value(entry()["abilities"]["card"][0].clone()).unwrap(),
    )
    .unwrap();
    s.source = shielded;
    let effect = match &p.instructions[1] {
        Instruction::Group(e) => e,
        _ => panic!(),
    };
    e.generate_uniform_damage(&mut s, effect).unwrap();
    assert_eq!(
        s.event_group.uniform_damage.records[..4]
            .iter()
            .map(|r| r.target())
            .collect::<Vec<_>>(),
        vec![a, b, shielded, c]
    );
    s.event_group = EventGroupState::default();
    s.source = 0;
    let out = play(&e, &s);
    assert!(!e.in_arena(&out, a) && !e.in_arena(&out, b) && !e.in_arena(&out, c));
    assert!(e.in_arena(&out, shielded));
    assert_eq!(out.card(shielded).damage, 0);
    let after = e.upgrades(&out, shielded);
    assert_eq!(after.len(), 1);
    assert!(!after.contains(&before[0]));
}
#[test]
fn cinder_aura_death_follows_all_primary_damage() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let aura = unit(&e, &mut s, seat, "JTL_085");
        let recipient = unit(&e, &mut s, seat, "ASH_078");
        assert_eq!(e.hp(&s, recipient), 6);
        let out = play(&e, &s);
        assert!(!e.in_arena(&out, aura));
        assert!(!e.in_arena(&out, recipient));
        assert!(out.player(seat).discard.contains(aura));
        assert!(out.player(seat).discard.contains(recipient));
    }
}
#[test]
fn cinder_latched_winner_keeps_then_and_mandatory_death_heal() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let base = s.player(seat).base;
        s.card_mut(base).damage = 25;
        let donor = unit(&e, &mut s, seat, "LAW_097");
        let out = play(&e, &s);
        assert_eq!(out.winner(), Some(seat.other()));
        assert_eq!(out.card(base).damage, 28);
        assert!(!e.in_arena(&out, donor));
    }
}
#[test]
fn cinder_latched_winner_preserves_mandatory_target_choice() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let base = s.player(seat).base;
        let enemy = s.player(seat.other()).base;
        s.card_mut(base).damage = 25;
        s.card_mut(enemy).damage = 28;
        let donor = unit(&e, &mut s, seat, "LAW_189");
        let out = play(&e, &s);
        assert_eq!(out.winner(), Some(seat.other()));
        assert!(!e.in_arena(&out, donor));
        assert!(
            e.legal_actions(&out).contains(&e.card_action(&out, enemy)),
            "mandatory death prompt must remain legal after winner latch: {:?}",
            out.decision
        );
        let done = choose(&e, &out, enemy);
        assert_eq!(done.card(base).damage, 30);
        assert_eq!(done.card(enemy).damage, 30);
        assert_eq!(done.winner(), Some(seat.other()));
        assert!(e.legal_actions(&done).is_empty());
    }
}

#[test]
fn cinder_both_owner_replacement_menus_preserve_alternate_shield_identities_after_winner() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let base = s.player(Seat::P1).base;
    s.card_mut(base).damage = 25;
    let units = [
        unit(&e, &mut s, Seat::P1, "SOR_095"),
        unit(&e, &mut s, Seat::P2, "LAW_253"),
        unit(&e, &mut s, Seat::P2, "SOR_095"),
        unit(&e, &mut s, Seat::P1, "LAW_253"),
    ];
    let mut shields = vec![];
    for id in units {
        s.source = id;
        e.apply_effect(
            &mut s,
            &serde_json::from_value(
                json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":2}}),
            )
            .unwrap(),
        )
        .unwrap();
        shields.push(e.upgrades(&s, id));
    }
    s.source = 0;
    let mut out = play(&e, &s);
    assert_eq!(out.winner(), Some(Seat::P2));
    assert_eq!(out.actor(), None);
    assert_eq!(
        e.legal_actions(&out)
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        ["You", "Opponent"]
    );
    out = button(&e, &out, "You");
    assert_eq!(
        e.legal_actions(&out)
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        ["Resolve next", "Resolve all (2)"]
    );
    out = button(&e, &out, "Resolve all (2)");
    assert_eq!(out.active, Seat::P2);
    out = button(&e, &out, "Resolve all (2)");
    // ReplacementEffectWindow consolidates stashed alternates before retained
    // contexts after each resolved replacement, including another owner's turn.
    // Source oracle: self ground oldest, self space alternate; enemy ground
    // oldest, enemy space alternate. Preserve identities, not just counts.
    for (i, id) in units.iter().enumerate() {
        assert!(e.in_arena(&out, *id));
        assert_eq!(out.card(*id).damage, 0);
        let consumed = if i == 0 || i == 2 {
            shields[i][0]
        } else {
            shields[i][1]
        };
        let kept = if consumed == shields[i][0] {
            shields[i][1]
        } else {
            shields[i][0]
        };
        assert_eq!(e.upgrades(&out, *id), vec![kept]);
        assert_eq!(out.card(consumed).parent, 0);
    }
    assert_eq!(out.winner(), Some(Seat::P2));
    assert!(e.legal_actions(&out).is_empty());
}
#[test]
fn cinder_native_auto_and_interpreter_share_windows_and_terminal_snapshot() {
    let auto = engine();
    let mut interpreter = engine();
    interpreter
        .set_rules_backend(RulesBackend::Interpreter)
        .unwrap();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&auto, seat);
        let base = s.player(seat).base;
        let enemy = s.player(seat.other()).base;
        s.card_mut(base).damage = 25;
        s.card_mut(enemy).damage = 28;
        unit(&auto, &mut s, seat, "LAW_189");
        let pending = play(&auto, &s);
        assert_eq!(pending.actor(), None);
        assert!(pending.is_terminal());
        assert!(pending.has_pending_resolution());
        assert_eq!(
            auto.snapshot(&pending)["winner"],
            json!(seat.other().as_str())
        );
        assert_eq!(auto.snapshot(&pending)["actor"], Value::Null);
        for engine in [&interpreter] {
            let other = play(engine, &s);
            assert_eq!(other, pending);
            assert_eq!(engine.legal_actions(&other), auto.legal_actions(&pending));
        }
        let done = choose(&auto, &pending, enemy);
        assert!(!done.has_pending_resolution());
        let snapshot = auto.snapshot(&done);
        assert_eq!(snapshot["winner"], json!(seat.other().as_str()));
        assert_eq!(snapshot["actor"], Value::Null);
        let loaded = auto.load_state(&snapshot).unwrap();
        assert_eq!(loaded.winner(), done.winner());
        assert_eq!(loaded.card(base).damage, 30);
        assert_eq!(loaded.card(enemy).damage, 30);
        assert!(auto.legal_actions(&loaded).is_empty());
        let mut stale = done;
        stale.decision = Decision::Window;
        stale.ability_program = 1;
        assert!(
            auto.legal_actions(&stale).is_empty(),
            "latched outcome must never reopen ordinary turn actions"
        );
        let mut no_order = snapshot;
        no_order.as_object_mut().unwrap().remove("arenaUnitOrder");
        assert!(
            auto.load_state(&no_order).is_err(),
            "reachable Cinder requires exact order even for empty final arena"
        );
    }
}

#[test]
fn cinder_shared_latched_protocol_keeps_real_committed_optional_root_discard() {
    let auto = engine();
    let mut interpreted = engine();
    interpreted
        .set_rules_backend(RulesBackend::Interpreter)
        .unwrap();
    for e in [&auto, &interpreted] {
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(e, seat);
            let base = s.player(seat).base;
            s.card_mut(base).damage = 29;
            assert_eq!(s.player(seat).deck.len, 0);
            let source = add(e, &mut s, seat, "ASH_260");
            let discard = add(e, &mut s, seat, "SOR_095");
            s.player_mut(seat).hand.push(source).unwrap();
            s.player_mut(seat).hand.push(discard).unwrap();
            let optional = choose(e, &s, source);
            assert_eq!(optional.decision, Decision::AbilityOptional);
            let pending = button(e, &optional, "Trigger");
            assert_eq!(pending.decision, Decision::AbilityDiscard);
            assert_eq!(pending.winner(), Some(seat.other()));
            assert_eq!(pending.actor(), None);
            assert!(
                e.program(pending.ability_program - 1).optional,
                "root stays optional after its draw is committed"
            );
            assert!(e
                .legal_actions(&pending)
                .contains(&e.card_action(&pending, discard)));
            let done = choose(e, &pending, discard);
            assert_eq!(done.winner(), Some(seat.other()));
            assert_eq!(done.card(base).damage, 30);
            assert_eq!(done.player(seat).hand.len, 0);
            assert!(done.player(seat).discard.contains(discard));
            assert!(e.legal_actions(&done).is_empty());
        }
    }
}
#[test]
fn cinder_real_raddus_after_latched_base_uses_public_death_target_and_lki() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let base = s.player(seat).base;
        s.card_mut(base).damage = 25;
        let donor = unit(&e, &mut s, seat, "JTL_104");
        s.card_mut(donor).damage = 3;
        let target = unit(&e, &mut s, seat.other(), "LOF_256");
        s.card_mut(target).phase_hp = 10;
        let pending = play(&e, &s);
        assert_eq!(pending.winner(), Some(seat.other()));
        assert_eq!(pending.source, donor);
        assert_eq!(pending.decision, Decision::AbilityTarget);
        assert_eq!(pending.card(target).damage, 5);
        assert_eq!(pending.actor(), None);
        let done = choose(&e, &pending, target);
        assert_eq!(done.card(target).damage, 13);
        assert_eq!(done.winner(), Some(seat.other()));
        assert!(e.legal_actions(&done).is_empty());
    }
}

#[test]
fn cinder_real_opponent_optional_death_prompt_remains_public_after_own_death_wins() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let enemy = seat.other();
        let enemy_base = s.player(enemy).base;
        s.card_mut(enemy_base).damage = 28;
        unit(&e, &mut s, seat, "LAW_189");
        unit(&e, &mut s, enemy, "JTL_063");
        let draw = add(&e, &mut s, enemy, "SOR_095");
        s.player_mut(enemy).deck.push(draw).unwrap();
        let order = play(&e, &s);
        assert_eq!(order.decision, Decision::DeathPlayer);
        let own = button(&e, &order, "You");
        assert_eq!(own.decision, Decision::AbilityTarget);
        let opponent = choose(&e, &own, enemy_base);
        assert_eq!(opponent.winner(), Some(seat));
        assert_eq!(opponent.active, enemy);
        assert_eq!(opponent.decision, Decision::AbilityOptional);
        assert_eq!(opponent.actor(), None);
        let accepted = button(&e, &opponent, "Trigger");
        assert!(accepted.player(enemy).hand.contains(draw));
        assert_eq!(accepted.winner(), Some(seat));
        assert!(e.legal_actions(&accepted).is_empty());
        let declined = button(&e, &opponent, "Pass");
        assert!(declined.player(enemy).deck.contains(draw));
        assert_eq!(declined.winner(), Some(seat));
        assert!(e.legal_actions(&declined).is_empty());
    }
}

#[test]
fn cinder_terminal_actor_ingress_validates_captured_chooser_without_reopening_turns() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let base = s.player(seat).base;
        let enemy_base = s.player(seat.other()).base;
        s.card_mut(base).damage = 25;
        unit(&e, &mut s, seat, "LAW_189");
        let pending = play(&e, &s);
        assert!(pending.is_terminal());
        assert_eq!(pending.actor(), None);
        assert_eq!(pending.active, seat);
        let mut restored = pending;
        let hash = restored.zobrist();
        e.set_actor(&mut restored, seat).unwrap();
        assert_eq!(restored, pending);
        assert_eq!(restored.zobrist(), hash);
        assert!(e.set_actor(&mut restored, seat.other()).is_err());
        assert_eq!(restored, pending);
        let done = choose(&e, &restored, enemy_base);
        assert_eq!(done.winner(), pending.winner());
        assert_eq!(done.actor(), None);
        for decision in [Decision::Window, Decision::Resource, Decision::AttackTarget] {
            let mut rejected = pending;
            rejected.decision = decision;
            rejected.resource_done = [false; 2];
            let before = rejected;
            for actor in [seat, seat.other()] {
                assert!(e.set_actor(&mut rejected, actor).is_err());
                assert_eq!(rejected, before);
            }
            assert!(e.legal_actions(&rejected).is_empty());
        }
        let mut completed = done;
        assert!(e.set_actor(&mut completed, seat).is_err());
        assert_eq!(completed, done);
    }
}

// Source expectations are bound to six actual regroup fatigue oracles: both
// initiative seats, either base lethal or both lethal. ResourcePrompt is an
// AllPlayerPrompt, so unfinished seats can interleave independently of active.
#[test]
fn cinder_terminal_regroup_fatigue_finishes_simultaneous_resources_and_ready() {
    let e = engine();
    for initiative in [Seat::P1, Seat::P2] {
        for lethal in [1, 2, 3] {
            let mut s = board(&e, initiative);
            s.initiative = initiative;
            for seat in [Seat::P1, Seat::P2] {
                let base = s.player(seat).base;
                s.card_mut(base).damage = if lethal & (1 << seat.index()) != 0 { 24 } else { 0 };
                let card = add(&e, &mut s, seat, "SOR_095");
                s.player_mut(seat).hand.push(card).unwrap();
                s.card_mut(s.player(seat).leader).exhausted = true;
            }
            let pending = button(&e, &button(&e, &s, "Pass"), "Pass");
            assert_eq!(pending.decision, Decision::Resource);
            assert!(pending.regroup && pending.is_terminal());
            assert_eq!(pending.actor(), None);
            assert_eq!(pending.outcome, if lethal == 3 { 3 } else if lethal == 1 { 2 } else { 1 });
            for seat in [Seat::P1, Seat::P2] {
                assert_eq!(pending.card(pending.player(seat).base).damage,
                    if lethal & (1 << seat.index()) != 0 { 30 } else { 6 });
            }
            let mut selected = pending;
            let ids = [selected.player(Seat::P1).hand.as_slice()[0], selected.player(Seat::P2).hand.as_slice()[0]];
            // Nonactive seat chooses first, then the other seat, before either confirms.
            for seat in [Seat::P2, Seat::P1] {
                let owner = selected.action_player;
                e.set_actor(&mut selected, seat).unwrap();
                assert_eq!(selected.action_player, owner);
                selected = choose(&e, &selected, ids[seat.index()]);
            }
            selected = button(&e, &selected, "Confirm Resources");
            let done_seat = Seat::P1;
            let before = selected;
            assert!(e.set_actor(&mut selected, done_seat).is_err());
            assert_eq!(selected, before);
            e.set_actor(&mut selected, Seat::P2).unwrap();
            let done = button(&e, &selected, "Confirm Resources");
            assert_eq!(done.round, s.round + 1);
            assert!(!done.regroup);
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.outcome, pending.outcome);
            assert_eq!(done.actor(), None);
            assert!(e.legal_actions(&done).is_empty());
            for seat in [Seat::P1, Seat::P2] {
                assert!(done.player(seat).resources.contains(ids[seat.index()]));
                assert!(!done.player(seat).hand.contains(ids[seat.index()]));
                for id in done.player(seat).resources.as_slice() { assert!(!done.card(*id).exhausted); }
                assert!(!done.card(done.player(seat).leader).exhausted);
            }
        }
    }
}

#[test]
fn cinder_expiry_winner_keeps_queued_regroup_draw_resource_and_ready() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        s.initiative = seat;
        let enemy_base = s.player(seat.other()).base;
        s.card_mut(enemy_base).damage = 28;
        let cavern = unit(&e, &mut s, seat, "LAW_189");
        e.create_token(&mut s, cavern, true).unwrap();
        let glory = add(&e, &mut s, seat, "SHD_130");
        s.player_mut(seat).hand.push(glory).unwrap();
        let buff = choose(&e, &choose(&e, &s, glory), cavern);
        assert_eq!(e.hp(&buff, cavern), 6);
        let own_turn = button(&e, &buff, "Pass");
        let after_cinder = play(&e, &own_turn);
        assert!(e.in_arena(&after_cinder, cavern));
        assert_eq!(after_cinder.card(cavern).damage, 5);
        let death = button(&e, &button(&e, &after_cinder, "Pass"), "Pass");
        assert_eq!(death.decision, Decision::AbilityTarget);
        assert!(!e.in_arena(&death, cavern));
        assert!(death.players.iter().all(|p| p.out));
        let pending = choose(&e, &death, enemy_base);
        assert_eq!(pending.winner(), Some(seat));
        assert_eq!(pending.decision, Decision::Resource);
        assert!(pending.regroup);
        assert_eq!(pending.card(pending.player(seat).base).damage, 11);
        assert_eq!(pending.card(enemy_base).damage, 30);
        let done = button(&e, &button(&e, &pending, "Skip Resourcing"), "Skip Resourcing");
        assert_eq!(done.winner(), Some(seat));
        assert_eq!(done.round, s.round + 1);
        assert_eq!(done.actor(), None);
        assert!(e.legal_actions(&done).is_empty());
    }
}

// Final Showdown is unsupported. This diagnostic mirrors only its independently
// observed *already captured* regroup-start continuation, never card admission.
#[test]
fn cinder_terminal_captured_regroup_draw_preserves_earlier_winner() {
    let e = engine();
    for earlier_winner in [Seat::P1, Seat::P2] {
        let mut s = board(&e, earlier_winner);
        s.regroup = true;
        s.outcome = if earlier_winner == Seat::P1 { 1 } else { 2 };
        s.decision = Decision::Terminal;
        for seat in [Seat::P1, Seat::P2] { s.card_mut(s.player(seat).base).damage = 25; }
        let mut uncaptured = s;
        uncaptured.regroup = false;
        e.draw_for_regroup(&mut uncaptured).unwrap();
        assert_eq!(uncaptured.card(uncaptured.player(Seat::P1).base).damage, 25);
        e.advance_after_expiry(&mut uncaptured).unwrap();
        assert!(!uncaptured.regroup);
        e.draw_for_regroup(&mut s).unwrap();
        assert_eq!(s.decision, Decision::Resource);
        assert_eq!(s.winner(), Some(earlier_winner));
        for seat in [Seat::P1, Seat::P2] { assert_eq!(s.card(s.player(seat).base).damage, 30); }
        let done = button(&e, &button(&e, &s, "Skip Resourcing"), "Skip Resourcing");
        assert_eq!(done.winner(), Some(earlier_winner));
        assert_eq!(done.round, s.round + 1);
        assert_eq!(done.actor(), None);
        assert!(e.legal_actions(&done).is_empty());
    }
}
