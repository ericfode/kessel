//! Source-derived midgame regressions. These are not forceteki differential
//! witnesses; source departure below is explicitly an internal capture diagnostic.
use super::*;

const CANDIDATES: &[&str] = &["JTL_198", "JTL_216", "LAW_071"];
fn source(code: &str) -> Value {
    committed_catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn cases(mut run: impl FnMut(&Engine, Seat)) {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            run(&e, seat);
        }
    }
}
fn midgame(e: &Engine, actor: Seat) -> State {
    let mut s = board(e);
    s.round = 4;
    s.active = actor;
    s.action_player = Some(actor);
    s.initiative = actor.other();
    s.prev_passed = true;
    for seat in [Seat::P1, Seat::P2] {
        deck_card(e, &mut s, seat);
        deck_card(e, &mut s, seat);
    }
    s
}
fn begin(e: &Engine, s: &State) -> State {
    named_button(e, s, "Pass")
}
fn trigger(e: &Engine, s: &State, source: CardId) -> State {
    assert_eq!(s.decision, Decision::DeathOrder);
    let group = e
        .death_groups(s)
        .iter()
        .position(|g| s.death_window.entries[g[0]].source == source)
        .unwrap();
    let action = e.legal_actions(s)[group].clone();
    e.apply(s, &action).unwrap()
}
fn credits(s: &State, seat: Seat) -> Vec<CardId> {
    s.player(seat)
        .credits
        .as_slice()
        .iter()
        .map(|id| u16::from(*id))
        .collect()
}
fn before_draw(e: &Engine, s: &State) {
    assert!(s.regroup);
    assert_eq!(s.round, 4);
    for seat in [Seat::P1, Seat::P2] {
        assert_eq!(s.player(seat).deck.len, 2);
        assert!(s.player(seat).hand.as_slice().is_empty());
    }
    assert_eq!(s.decision, Decision::DeathOrder);
    let copied = *s;
    assert_eq!(copied.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&copied), e.legal_actions(s));
    let snapshot = e.snapshot(s);
    assert_eq!(snapshot["regroupStartPending"], json!(true));
    assert!(
        e.load_state(&snapshot).is_err(),
        "pending regroup window cannot be reconstructed from a settled snapshot"
    );
}

#[test]
fn regroup_hook_complete_source_closure_and_typed_registration() {
    for code in CANDIDATES {
        let original = source(code);
        assert!(crate::regroup_hooks::entry(&original), "{code}");
        let ability: crate::ability_ir::Ability =
            serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
        assert!(crate::regroup_hooks::registration(&ability));
        crate::ability_ir::Program::compile(&ability).unwrap();
        let mut renamed = original.clone();
        renamed["setCode"] = json!("DIAGNOSTIC_REGROUP");
        renamed["id"] = json!("new source");
        renamed["className"] = json!("DiagnosticRegroup");
        renamed["card"]["title"] = json!("Diagnostic card");
        renamed["abilities"]["card"][0]["title"] = json!("Diagnostic trigger");
        renamed["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"] =
            json!("callback:diagnostic:0");
        assert!(crate::regroup_hooks::entry(&renamed));
        let mut reordered = original.clone();
        for key in ["aspects", "traits"] {
            reordered["card"][key].as_array_mut().unwrap().reverse();
        }
        assert!(crate::regroup_hooks::entry(&reordered));
    }
    for code in [
        "ASH_159", "LOF_055", "TS26_023", "TWI_067", "LAW_073", "HMW_004", "LOF_019", "SHD_015",
    ] {
        assert!(
            !crate::regroup_hooks::entry(&source(code)),
            "unreviewed full source {code}"
        );
    }
}

#[test]
fn regroup_hook_rejects_hidden_metadata_hooks_and_registration_changes() {
    for code in CANDIDATES {
        let original = source(code);
        for (path, value) in [
            ("/status", json!("unsupported")),
            ("/diagnostics", json!(["hidden rule"])),
            ("/baseClass", json!("LeaderUnitCard")),
            ("/card/types", json!(["unit", "token"])),
            (
                "/card/text",
                json!("When the regroup phase starts: Do something else."),
            ),
            ("/card/hp", json!(0)),
            ("/card/power", json!(9)),
            ("/card/cost", json!(0)),
            ("/card/aspects", json!(["cunning", "cunning"])),
            ("/card/keywords", json!(["shielded"])),
            ("/keywordValues", json!({"ambush":0})),
            ("/abilities/card/0/trigger", json!("onAttack")),
            ("/abilities/card/0/hooks/0/event", json!("onPhaseEnded")),
            ("/abilities/card/0/hooks/0/predicate/op", json!("!==")),
            (
                "/abilities/card/0/hooks/0/predicate/right/value",
                json!("action"),
            ),
            (
                "/abilities/card/0/hooks/0/predicate/left/root",
                json!(["context"]),
            ),
            (
                "/abilities/card/0/hooks/0/predicate/left/steps/0/optional",
                json!(true),
            ),
            (
                "/abilities/card/0/hooks/0/predicate/left/argumentBinding/readAt",
                json!("triggerCapture"),
            ),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert!(!crate::regroup_hooks::entry(&changed), "{code}: {path}");
        }
        for (key, value) in [
            ("optional", json!(false)),
            ("condition", Value::Null),
            ("targetResolver", Value::Null),
            (
                "then",
                json!({"effect":{"kind":"draw","target":"ownPlayer"}}),
            ),
            ("ifYouDo", Value::Null),
            ("attachedContext", json!("host")),
            ("unreviewed", json!(true)),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0][key] = value;
            assert!(!crate::regroup_hooks::entry(&changed), "{code}: raw {key}");
        }
        for key in [
            "stateWatchers",
            "classMethods",
            "privatePrograms",
            "historyHelpers",
            "inheritedSource",
        ] {
            let mut changed = original.clone();
            changed[key] = json!([{}]);
            assert!(!crate::regroup_hooks::entry(&changed), "{code}: {key}");
        }
        let mut changed = original.clone();
        changed["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(!crate::regroup_hooks::entry(&changed));
        let mut changed = original.clone();
        changed["abilities"]["unit"] = original["abilities"]["card"].clone();
        assert!(!crate::regroup_hooks::entry(&changed));
        for (key, value) in [
            ("target", json!("chosen")),
            ("amount", json!({"kind":"const","value":2})),
            ("unknown", json!(true)),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["effect"][key] = value;
            assert!(
                !crate::regroup_hooks::entry(&changed),
                "{code}: effect {key}"
            );
        }
        let mut ability: crate::ability_ir::Ability =
            serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
        ability.optional = true;
        assert!(!crate::regroup_hooks::registration(&ability));
        ability.optional = false;
        ability.attached_host = true;
        assert!(!crate::regroup_hooks::registration(&ability));
    }
}

#[test]
fn regroup_hook_midgame_all_three_effects_precede_draw_and_ready() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let band = unit(e, &mut s, actor, "LAW_071");
        let fireball = unit(e, &mut s, actor, "JTL_198");
        let hunter = unit(e, &mut s, actor, "JTL_216");
        assert!(e.has_keyword(&s, fireball, "ambush"));
        assert!(e.has_keyword(&s, hunter, "ambush"));
        for id in [band, fireball, hunter] {
            s.card_mut(id).exhausted = true;
        }
        let resource = add(e, &mut s, actor, "LOF_256");
        s.player_mut(actor).resources.push(resource).unwrap();
        s.card_mut(resource).exhausted = true;
        let next = s.next_id;
        let pending = begin(e, &s);
        before_draw(e, &pending);
        assert!(credits(&pending, actor).is_empty());
        assert_eq!(pending.card(fireball).damage, 0);
        assert!(e.in_arena(&pending, hunter));
        let credited = trigger(e, &pending, band);
        before_draw(e, &credited);
        assert_eq!(credits(&credited, actor), vec![next]);
        assert_eq!(credited.card(next).owner, actor);
        assert_eq!(credited.next_id, next + 1);
        assert!(credited.card(resource).exhausted);
        assert_eq!(credited.card(fireball).damage, 0);
        assert!(e.in_arena(&credited, hunter));
        let done = trigger(e, &credited, fireball);
        assert_eq!(done.decision, Decision::Resource);
        assert_eq!(done.card(fireball).damage, 1);
        assert!(done.player(actor).discard.contains(hunter));
        assert_eq!(credits(&done, actor), vec![next]);
        assert!(done.card(resource).exhausted);
        assert!(done.card(fireball).exhausted);
        assert!(done.card(band).exhausted);
        for seat in [Seat::P1, Seat::P2] {
            assert!(done.player(seat).deck.as_slice().is_empty());
            assert_eq!(done.player(seat).hand.len, 2);
        }
        assert_eq!(done.round, 4);
        assert_eq!(e.snapshot(&done)["regroupStartPending"], Value::Null);
    });
}

#[test]
fn regroup_hook_initiative_chooses_owner_order_before_draw_independent_of_last_actor() {
    cases(|e, initiative| {
        let mut s = midgame(e, initiative.other());
        s.initiative = initiative;
        let own = unit(e, &mut s, initiative, "LAW_071");
        unit(e, &mut s, initiative, "JTL_198");
        unit(e, &mut s, initiative.other(), "LAW_071");
        let next = s.next_id;
        let order = begin(e, &s);
        assert_eq!(order.decision, Decision::DeathPlayer);
        assert_eq!(order.active, initiative);
        assert_eq!(
            e.legal_actions(&order),
            vec![Action::button("You", 0), Action::button("Opponent", 1)]
        );
        let own_first = named_button(e, &order, "You");
        before_draw(e, &own_first);
        assert!(credits(&own_first, initiative.other()).is_empty());
        let enemy_first = named_button(e, &order, "Opponent");
        before_draw(e, &enemy_first);
        assert_eq!(enemy_first.active, initiative);
        assert_eq!(credits(&enemy_first, initiative.other()), vec![next]);
        assert!(credits(&enemy_first, initiative).is_empty());
        let done = trigger(e, &enemy_first, own);
        assert_eq!(done.decision, Decision::Resource);
        assert_eq!(credits(&done, initiative), vec![next + 1]);
        assert_eq!(done.next_id, next + 2);
    });
}

#[test]
fn regroup_hook_captured_credit_retains_controller_after_source_departure() {
    // Internal diagnostic: depart the already captured source without opening an
    // external action. The registration must not be rebound to the other owner.
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let band = unit(e, &mut s, actor, "LAW_071");
        unit(e, &mut s, actor, "JTL_198");
        unit(e, &mut s, actor, "JTL_216");
        let next = s.next_id;
        let mut pending = begin(e, &s);
        before_draw(e, &pending);
        e.leave_play(&mut pending, band).unwrap();
        assert!(pending.player(actor).discard.contains(band));
        let credited = trigger(e, &pending, band);
        before_draw(e, &credited);
        assert_eq!(credits(&credited, actor), vec![next]);
        assert!(credits(&credited, actor.other()).is_empty());
        assert_eq!(credited.card(next).owner, actor);
        assert_eq!(credited.next_id, next + 1);
    });
}

#[test]
fn regroup_hook_credit_requires_reviewed_native_definition() {
    for malformed in [false, true] {
        let mut catalogue = committed_catalogue().clone();
        let cards = catalogue["cards"].as_array_mut().unwrap();
        if malformed {
            cards
                .iter_mut()
                .find(|e| e["setCode"] == "TOKEN_CREDIT")
                .unwrap()["sourceHash"] = json!("unreviewed");
        } else {
            cards.retain(|e| e["setCode"] != "TOKEN_CREDIT");
        }
        let e = Engine::from_catalogue_value(&catalogue).unwrap();
        assert!(!e.by_code.contains_key("LAW_071"));
        for code in ["JTL_198", "JTL_216"] {
            assert!(e.by_code.contains_key(code));
        }
    }
}

#[test]
fn regroup_hook_expiry_removes_sources_before_capture_and_self_damage_can_be_lethal() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let band = unit(e, &mut s, actor, "LAW_071");
        s.card_mut(band).phase_hp = 1;
        s.card_mut(band).damage = 5;
        let fireball = unit(e, &mut s, actor, "JTL_198");
        s.card_mut(fireball).damage = 2;
        assert_eq!(e.hp(&s, band), 6);
        let next = s.next_id;
        let done = begin(e, &s);
        assert_eq!(done.decision, Decision::Resource);
        assert!(done.player(actor).discard.contains(band));
        assert!(done.player(actor).discard.contains(fireball));
        assert!(credits(&done, actor).is_empty());
        assert_eq!(done.next_id, next);
        for seat in [Seat::P1, Seat::P2] {
            assert_eq!(done.player(seat).hand.len, 2);
        }
        assert_eq!(done.death_depth, 0);
    });
}

#[test]
fn regroup_hook_pending_marker_any_presence_precedes_forged_phase_actor_and_winner() {
    let e = engine();
    let s = midgame(&e, Seat::P1);
    let snapshot = e.snapshot(&s);
    for marker in [json!(true), json!(false), Value::Null] {
        for winner in [Value::Null, json!("p1")] {
            let mut forged = snapshot.clone();
            forged["regroupStartPending"] = marker.clone();
            forged["phase"] = json!("action");
            forged["actor"] = json!("p2");
            forged["activePlayer"] = json!("p2");
            forged["winner"] = winner;
            let error = e.load_state(&forged).unwrap_err();
            assert!(
                error.contains("pending regroup-start"),
                "marker {marker}: {error}"
            );
        }
    }
}

#[test]
fn regroup_hook_expiry_prevents_capture_and_live_self_damage_defeats_once() {
    cases(|e, actor| {
        for expired in [false, true] {
            let mut s = midgame(e, actor);
            let fireball = unit(e, &mut s, actor, "JTL_198");
            s.card_mut(fireball).damage = if expired { 3 } else { 2 };
            if expired {
                s.card_mut(fireball).phase_hp = 1;
            }
            let band = unit(e, &mut s, actor, "LAW_071");
            unit(e, &mut s, actor, "JTL_216");
            let pending = begin(e, &s);
            before_draw(e, &pending);
            assert_eq!(pending.death_window.len, if expired { 2 } else { 3 });
            assert_eq!(
                pending.death_window.entries[..pending.death_window.len as usize]
                    .iter()
                    .any(|entry| entry.source == fireball),
                !expired
            );
            let after_damage = if expired {
                pending
            } else {
                trigger(e, &pending, fireball)
            };
            before_draw(e, &after_damage);
            assert!(after_damage.player(actor).discard.contains(fireball));
            // Physical departure clears damage; the captured-entry assertion
            // above distinguishes expiry from a live regroup damage event.
            assert_eq!(after_damage.card(fireball).damage, 0);
            let done = trigger(e, &after_damage, band);
            assert_eq!(done.decision, Decision::Resource);
            assert_eq!(
                done.player(actor)
                    .discard
                    .as_slice()
                    .iter()
                    .filter(|id| **id == fireball)
                    .count(),
                1
            );
            assert_eq!(credits(&done, actor).len(), 1);
            assert_eq!(done.death_depth, 0);
            for seat in [Seat::P1, Seat::P2] {
                assert_eq!(done.player(seat).hand.len, 2);
            }
        }
    });
}

#[test]
fn regroup_hook_child_defeat_target_resumes_parent_before_draw_once() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let band = unit(e, &mut s, actor, "LAW_071");
        let hunter = unit(e, &mut s, actor, "JTL_216");
        let hask = unit(e, &mut s, actor.other(), "SOR_036");
        let next = s.next_id;
        let pending = begin(e, &s);
        before_draw(e, &pending);
        let child = trigger(e, &pending, hunter);
        assert_eq!(child.decision, Decision::AbilityTarget);
        assert_eq!(child.active, actor.other());
        assert_eq!(child.death_depth, 2);
        assert!(child.player(actor).discard.contains(hunter));
        assert!(credits(&child, actor).is_empty());
        assert_eq!(e.snapshot(&child)["regroupStartPending"], json!(true));
        for seat in [Seat::P1, Seat::P2] {
            assert_eq!(child.player(seat).deck.len, 2);
            assert_eq!(child.player(seat).hand.len, 0);
        }
        let done = e.apply(&child, &e.card_action(&child, hask)).unwrap();
        assert_eq!(done.decision, Decision::Resource);
        assert_eq!(done.death_depth, 0);
        assert_eq!(e.token_count(&done, done.card(hask).experience), 1);
        assert_eq!(credits(&done, actor), vec![next + 1]);
        assert_eq!(done.next_id, next + 2);
        assert!(e.in_arena(&done, band));
        for seat in [Seat::P1, Seat::P2] {
            assert_eq!(done.player(seat).hand.len, 2);
            assert_eq!(done.player(seat).deck.len, 0);
        }
        assert_eq!(e.snapshot(&done)["regroupStartPending"], Value::Null);
    });
}

#[test]
fn regroup_hook_capture_capacity_and_missing_chronology_fail_before_mutation() {
    let e = engine();
    let mut s = board(&e);
    s.round = 4;
    let mut expected = vec![];
    for _ in 0..32 {
        for seat in [Seat::P2, Seat::P1] {
            expected.push(unit(&e, &mut s, seat, "JTL_216"));
        }
    }
    let before = s;
    let mut captured = s;
    e.capture_regroup_start(&mut captured).unwrap();
    assert_eq!(captured.death_pending.len, 64);
    assert_eq!(
        captured.death_pending.entries[..64]
            .iter()
            .map(|e| e.source)
            .collect::<Vec<_>>(),
        expected
    );
    let mut unchanged = before;
    unchanged.death_pending = captured.death_pending;
    assert_eq!(captured, unchanged);
    let mut unknown = before;
    unknown.registration_order.exact = false;
    let untouched = unknown;
    assert!(e
        .capture_regroup_start(&mut unknown)
        .unwrap_err()
        .contains("registrationOrder"));
    assert_eq!(unknown, untouched);
    let mut missing = before;
    missing.registration_order.remove(expected[0]);
    let untouched = missing;
    assert!(e
        .capture_regroup_start(&mut missing)
        .unwrap_err()
        .contains("registrationOrder"));
    assert_eq!(missing, untouched);
    unit(&e, &mut s, Seat::P1, "JTL_198");
    let untouched = s;
    assert!(e
        .capture_regroup_start(&mut s)
        .unwrap_err()
        .contains("capacity exceeded (64)"));
    assert_eq!(s, untouched);
}

#[test]
fn regroup_hook_selected_future_source_requires_explicit_registration_order() {
    let full = engine();
    let original_pool = full.compiled_pool().unwrap();
    for future in [false, true] {
        let mut pool = original_pool.clone();
        pool["cards"].as_array_mut().unwrap().retain(|c| {
            [
                "ASH_019",
                "SOR_014",
                "IBH_053",
                "LOF_256",
                "TOKEN_THE_FORCE",
                "TOKEN_CREDIT",
                "TOKEN_SHIELD",
                "TOKEN_EXPERIENCE",
            ]
            .iter()
            .any(|code| c["setCode"] == *code)
                || (future && c["setCode"] == "JTL_216")
        });
        let e = Engine::from_compiled_pool_bytes(
            &serde_json::to_vec(&pool).unwrap(),
            committed_catalogue_bytes(),
        )
        .unwrap();
        assert_eq!(e.needs_registration_order(), future);
        let s = board(&e);
        let mut snapshot = e.snapshot(&s);
        snapshot
            .as_object_mut()
            .unwrap()
            .remove("registrationOrder");
        if future {
            assert!(e
                .load_state(&snapshot)
                .unwrap_err()
                .contains("registrationOrder"));
        } else {
            assert!(!e.load_state(&snapshot).unwrap().registration_order.exact);
        }
    }
}

#[test]
fn regroup_hook_terminal_continuation_diagnostic_never_captures_or_draws() {
    // Internal continuation diagnostic: an earlier mandatory expiry/death effect
    // has already latched a winner before either regroup continuation resumes.
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        unit(e, &mut s, actor, "LAW_071");
        let base = s.player(actor.other()).base;
        let hp = e.hp(&s, base);
        s.card_mut(base).damage = hp;
        e.check_win(&mut s);
        assert!(s.is_terminal());
        let before = s;
        for draw_only in [false, true] {
            let mut terminal = before;
            if draw_only {
                e.draw_for_regroup(&mut terminal).unwrap();
            } else {
                e.advance_after_expiry(&mut terminal).unwrap();
            }
            assert_eq!(terminal.decision, Decision::Terminal);
            assert_eq!(terminal.outcome, before.outcome);
            assert_eq!(terminal.next_id, before.next_id);
            assert_eq!(terminal.death_pending.len, 0);
            assert_eq!(terminal.death_depth, 0);
            assert!(credits(&terminal, actor).is_empty());
            assert_eq!(terminal.round, 4);
            for seat in [Seat::P1, Seat::P2] {
                assert_eq!(terminal.player(seat).deck, before.player(seat).deck);
                assert_eq!(terminal.player(seat).hand, before.player(seat).hand);
            }
        }
    });
}
