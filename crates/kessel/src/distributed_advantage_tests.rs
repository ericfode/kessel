use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: e.by_code[code],
        owner: seat,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.credit_state_exact = e.needs_credit_state();
    for p in [Seat::P1, Seat::P2] {
        let b = add(e, &mut s, p, "ASH_019");
        let l = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).base = b;
        s.player_mut(p).leader = l;
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn tokens(e: &Engine, s: &State, host: CardId) -> Vec<CardId> {
    e.upgrades(s, host)
        .into_iter()
        .filter(|id| Some(s.card(*id).def) == e.advantage_def)
        .collect()
}
fn pending(e: &Engine, s: &mut State, seat: Seat, n: u16) -> CardId {
    let source = unit(e, s, seat, "ASH_195");
    e.defeat_events(s, &[(source, crate::defeat_cause::DefeatCause::Ability)])
        .unwrap();
    assert!(!e.in_arena(s, source));
    for d in &mut s.death_pending.entries[..s.death_pending.len as usize] {
        if d.source == source {
            d.last_known_power = n;
        }
    }
    e.open_deaths(s, DeathContinuation::Finish).unwrap();
    source
}
fn allocation(e: &Engine, s: &State, pairs: &[(CardId, u16)]) -> Action {
    let arg = json!(pairs
        .iter()
        .map(|(id, n)| json!([format!("c{}", id - 1), n]))
        .collect::<Vec<_>>());
    e.legal_actions(s)
        .into_iter()
        .find(|a| a.arg.as_ref() == Some(&arg))
        .unwrap_or_else(|| panic!("missing {arg}: {:?}", e.legal_actions(s)))
}
#[test]
fn advantage_distribution_complete_source_raw_guards_and_token_closure() {
    let original = catalogue();
    let index = original["cards"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v["setCode"] == "ASH_195")
        .unwrap();
    assert!(engine().by_code.contains_key("ASH_195"));
    for (pointer, value) in [
        ("/sourceHash", json!("bad")),
        ("/baseClass", json!("LeaderUnitCard")),
        ("/card/types", json!(["unit", "token"])),
        ("/card/keywords", json!(["support"])),
        ("/abilities/card/0/optional", json!(true)),
        ("/abilities/card/0/contextTitle", json!([])),
        (
            "/abilities/card/0/effect/amount",
            json!({"kind":"const","value":6}),
        ),
        ("/abilities/card/0/effect/tokenType", json!("experience")),
        ("/abilities/card/0/effect/chooser", json!("opponent")),
        (
            "/abilities/card/0/effect/creationEvents",
            json!("singleBatch"),
        ),
        ("/abilities/card/0/effect/canChooseNoTargets", json!(false)),
        ("/abilities/card/0/effect/canDistributeLess", json!(true)),
        (
            "/abilities/card/0/effect/targetResolver/owner",
            json!("any"),
        ),
        (
            "/abilities/card/0/effect/targetResolver/promptTitle",
            json!("Changed"),
        ),
        ("/abilities/card/0/effect/maxTargets", json!(6)),
        ("/historyHelpers", json!([])),
    ] {
        let mut c = original.clone();
        let mut v = &mut c["cards"][index];
        let keys = pointer.split('/').skip(1).collect::<Vec<_>>();
        for key in &keys[..keys.len() - 1] {
            v = if v.is_array() {
                &mut v[key.parse::<usize>().unwrap()]
            } else {
                &mut v[*key]
            };
        }
        v[keys[keys.len() - 1]] = value;
        let c = crate::ability_ir::Catalogue::load(
            &json!({"schema":"swu-card-ir/v1","cards":[c["cards"][index].clone()]}),
        )
        .unwrap();
        assert!(!c.cards.contains_key("ASH_195"), "{pointer}");
    }
    for mutate in [false, true] {
        let mut c = original.clone();
        if mutate {
            let token = c["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|v| v["setCode"] == "TOKEN_ADVANTAGE")
                .unwrap();
            token["card"]["power"] = json!(2);
        } else {
            c["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
        }
        let e = Engine::from_catalogue_value(&c).unwrap();
        assert!(!e.by_code.contains_key("ASH_195"));
    }
}
#[test]
fn advantage_distribution_exact_ordered_actions_and_copy_hash() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let a = unit(&e, &mut s, seat, "LOF_256");
        let b = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        let source = pending(&e, &mut s, seat, 6);
        assert_eq!(s.decision, Decision::AbilityDistribute);
        assert_eq!(e.legal_actions(&s).len(), 13);
        assert!(e.legal_actions(&s).iter().all(|a| a.kind == "distribute"
            && a.card.is_none()
            && a.method.is_none()
            && a.label == "distribute 6"));
        assert!(!e.legal_actions(&s).iter().any(|a| a
            .arg
            .as_ref()
            .unwrap()
            .to_string()
            .contains(&format!("\"c{}\"", enemy - 1))));
        let copied = s;
        assert_eq!(s.zobrist(), copied.zobrist());
        let mut changed = s;
        changed.last_known_power = 7;
        assert_ne!(s.zobrist(), changed.zobrist());
        let first = s.next_id;
        let ab = e.apply(&s, &allocation(&e, &s, &[(a, 2), (b, 4)])).unwrap();
        let ba = e
            .apply(&copied, &allocation(&e, &copied, &[(b, 4), (a, 2)]))
            .unwrap();
        assert_eq!(tokens(&e, &ab, a), (first..first + 2).collect::<Vec<_>>());
        assert_eq!(tokens(&e, &ba, b), (first..first + 4).collect::<Vec<_>>());
        assert_ne!(ab.zobrist(), ba.zobrist());
        assert_eq!(e.power(&ab, a, false), 3);
        assert_eq!(e.power(&ab, b, false), 5);
        assert_eq!(ab.card(source).parent, 0);
        assert!(ab.player(seat).discard.contains(source));
    }
}
#[test]
fn advantage_distribution_single_recipient_still_prompts_and_decline_allocates_nothing() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    pending(&e, &mut s, Seat::P1, 6);
    assert_eq!(e.legal_actions(&s).len(), 2);
    let first = s.next_id;
    let out = e.apply(&s, &allocation(&e, &s, &[])).unwrap();
    assert_eq!(out.next_id, first);
    assert!(tokens(&e, &out, host).is_empty());
    assert_eq!(out.decision, Decision::Window);
}
#[test]
fn advantage_distribution_zero_and_no_recipients_have_no_prompt() {
    let e = engine();
    for n in [0, 6] {
        let mut s = board(&e, Seat::P2);
        if n == 0 {
            unit(&e, &mut s, Seat::P2, "LOF_256");
        }
        pending(&e, &mut s, Seat::P2, n);
        assert_ne!(s.decision, Decision::AbilityDistribute);
        assert_eq!(s.last_known_source, 0);
    }
}
#[test]
fn advantage_distribution_injected_partial_duplicate_wrong_owner_rejected() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
    pending(&e, &mut s, Seat::P1, 6);
    for pairs in [
        vec![(host, 5)],
        vec![(host, 0)],
        vec![(host, 3), (host, 3)],
        vec![(enemy, 6)],
    ] {
        let mut a = allocation(&e, &s, &[]);
        a.arg = Some(json!(pairs
            .iter()
            .map(|(id, n)| json!([format!("c{}", id - 1), n]))
            .collect::<Vec<_>>()));
        assert!(e.apply(&s, &a).is_err());
    }
    assert!(tokens(&e, &s, host).is_empty());
}
#[test]
fn advantage_distribution_prepared_events_original_context_and_empty_meta() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    let source = pending(&e, &mut s, Seat::P1, 6);
    let effect = e.program(s.ability_program - 1).instructions[s.ability_pc as usize]
        .group_effect()
        .unwrap()
        .clone();
    let event = s.event_result;
    let first = s.next_id;
    e.capture_advantage_distribution(&mut s, &[(host, 6)])
        .unwrap();
    assert_eq!(s.last_known_source, source);
    assert_eq!(s.last_known_power, 6);
    assert!(s.event_group.records[..6]
        .iter()
        .enumerate()
        .all(|(i, r)| r.target == host && r.amount == 6 && r.token == first + i as u16));
    let copy = s;
    assert_eq!(s.zobrist(), copy.zobrist());
    e.resolve_group(&mut s, &effect).unwrap();
    assert_eq!(s.ability_result, 6);
    assert!(s.ability_succeeded);
    assert_eq!(s.last_known_source, source);
    assert_eq!(s.event_result, StepEventResult::for_state(&s));
    assert_eq!(event.damage_dealt, 0);
    s.event_group = EventGroupState::default();
    s.ability_result = 99;
    s.ability_succeeded = false;
    e.capture_advantage_distribution(&mut s, &[]).unwrap();
    e.resolve_group(&mut s, &effect).unwrap();
    assert_eq!(s.ability_result, 0);
    assert!(s.ability_succeeded);
}
#[test]
fn advantage_distribution_explicit_amount_action_and_physical_capacity_errors() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    pending(&e, &mut s, Seat::P1, 6);
    let original = s;
    s.last_known_source = 0;
    assert!(e.distribution_actions(&s).unwrap_err().contains("captured"));
    s = original;
    s.last_known_power = 33;
    let before = s;
    assert!(e
        .capture_advantage_distribution(&mut s, &[(host, 33)])
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(before, s);
    s = original;
    s.next_id = (MAX_CARDS - 3) as u16;
    let before = s;
    assert!(e
        .capture_advantage_distribution(&mut s, &[(host, 6)])
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(before, s);
    s = original;
    for _ in 0..6 {
        unit(&e, &mut s, Seat::P1, "LOF_256");
    }
    assert!(e.distribution_actions(&s).unwrap_err().contains("capacity"));
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
}
#[test]
fn advantage_distribution_captures_live_modified_power_before_departure() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        unit(&e, &mut s, seat, "LOF_256");
        let source = unit(&e, &mut s, seat, "ASH_195");
        e.modify_phase_stats(&mut s, source, PhaseStatDelta { power: 2, hp: 0 })
            .unwrap();
        s.source = source;
        e.apply_effect(&mut s,&serde_json::from_value(json!({"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":1},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).unwrap()).unwrap();
        assert_eq!(e.power(&s, source, false), 9);
        e.defeat_events(
            &mut s,
            &[(source, crate::defeat_cause::DefeatCause::NonCombatDamage)],
        )
        .unwrap();
        assert_eq!(s.death_pending.entries[0].last_known_power, 9);
        assert_ne!(e.power(&s, source, false), 9);
        e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
        assert_eq!(s.last_known_power, 9);
        assert_eq!(e.legal_actions(&s).len(), 2);
        assert!(e
            .legal_actions(&s)
            .iter()
            .all(|a| a.label == "distribute 9"));
    }
}
#[test]
fn advantage_distribution_native_attack_end_retires_physical_tokens_and_reload() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let host = unit(&e, &mut s, seat, "LOF_256");
        pending(&e, &mut s, seat, 6);
        let s = e.apply(&s, &allocation(&e, &s, &[(host, 6)])).unwrap();
        let ids = tokens(&e, &s, host);
        let snap = e.snapshot(&s);
        let imported = e.load_state(&snap).unwrap();
        assert_eq!(snap, e.snapshot(&imported));
        assert_eq!(s.zobrist(), imported.zobrist());
        let next = s.next_id;
        let s = if s.active != seat {
            e.apply(&s, &Action::button("Pass", "pass")).unwrap()
        } else {
            s
        };
        let start = e.apply(&s, &e.card_action(&s, host)).unwrap();
        let target = start.player(seat.other()).base;
        let mut out = e.apply(&start, &e.card_action(&start, target)).unwrap();
        assert_eq!(out.decision, Decision::DeathBatch);
        let action = e
            .legal_actions(&out)
            .into_iter()
            .find(|a| a.label == "Resolve all (6)")
            .unwrap();
        out = e.apply(&out, &action).unwrap();
        assert_eq!(out.decision, Decision::Window);
        assert!(tokens(&e, &out, host).is_empty());
        assert_eq!(out.next_id, next);
        assert!(ids.iter().all(|id| out.card(*id).parent == 0));
        assert_eq!(e.power(&out, host, false), 1);
        let retired = e.snapshot(&out);
        let restored = e.load_state(&retired).unwrap();
        assert_eq!(retired, e.snapshot(&restored));
        assert_eq!(out.zobrist(), restored.zobrist());
    }
}
#[test]
fn advantage_distribution_actual_attack_death_preserves_raid_lki_and_attacker_context() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let host = unit(&e, &mut s, seat, "LOF_256");
            let source = unit(&e, &mut s, seat, "ASH_195");
            let defender = unit(&e, &mut s, seat.other(), "LOF_256");
            s.source = source;
            e.apply_effect(
                &mut s,
                &Effect::GainKeyword {
                    target: Target::name("self"),
                    keyword: "raid".into(),
                    amount: Expr::number(3),
                    duration: "phase".into(),
                },
            )
            .unwrap();
            s.source = 0;
            s.card_mut(source).damage = 3;
            let start = e.apply(&s, &e.card_action(&s, source)).unwrap();
            let pending = e.apply(&start, &e.card_action(&start, defender)).unwrap();
            assert_eq!(pending.decision, Decision::AbilityDistribute);
            assert_eq!(pending.attack_source, source);
            assert_eq!(pending.last_known_source, source);
            assert_eq!(pending.last_known_power, 9);
            assert!(!e.in_arena(&pending, source));
            assert!(!e.in_arena(&pending, defender));
            assert_eq!(e.legal_actions(&pending).len(), 2);
            let out = e
                .apply(&pending, &allocation(&e, &pending, &[(host, 9)]))
                .unwrap();
            assert_eq!(tokens(&e, &out, host).len(), 9);
            assert_eq!(out.attack_source, 0);
            assert_eq!(out.decision, Decision::Window);
        }
    }
}
#[test]
fn advantage_distribution_selected_pool_closure_replacement_boundary_and_admission_delta() {
    let c = catalogue();
    let e = engine();
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&c).unwrap()
    )
    .err()
    .unwrap()
    .contains("TOKEN_ADVANTAGE"));
    let moff = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["internalName"] == "moff-jerjerrod#we-shall-redouble-our-efforts")
        .unwrap();
    assert!(
        !e.by_code.contains_key(moff["setCode"].as_str().unwrap()),
        "token creation replacement is outside the admitted pool"
    );
    // Canonical pool membership is durable. Per-increment gains/losses are
    // measured by the pinned CLI receipt, not against a commit-volatile count.
    let admitted = e.compiled_pool().unwrap();
    assert!(admitted["cards"].as_array().unwrap().iter().any(|c| c["setCode"] == "ASH_195"));
}
#[test]
fn advantage_distribution_full_three_recipient_ui_includes_deployed_leader_and_token() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let ordinary = unit(&e, &mut s, seat, "LOF_256");
        let token = unit(&e, &mut s, seat, "TOKEN_SPY");
        let leader = s.player(seat).leader;
        s.card_mut(leader).deployed = true;
        e.add_arena(&mut s, leader).unwrap();
        pending(&e, &mut s, seat, 6);
        let actions = e.legal_actions(&s);
        assert_eq!(actions.len(), 94);
        for order in [
            [ordinary, token, leader],
            [leader, ordinary, token],
            [token, leader, ordinary],
        ] {
            assert!(actions.contains(&allocation(
                &e,
                &s,
                &order.into_iter().map(|id| (id, 2)).collect::<Vec<_>>()
            )));
        }
        let out = e
            .apply(
                &s,
                &allocation(&e, &s, &[(leader, 2), (token, 2), (ordinary, 2)]),
            )
            .unwrap();
        assert_eq!(tokens(&e, &out, leader).len(), 2);
        assert_eq!(tokens(&e, &out, token).len(), 2);
        assert_eq!(tokens(&e, &out, ordinary).len(), 2);
    }
}
#[test]
fn advantage_distribution_has_no_64_recipient_shortcut_and_rejects_pending_defeats() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let doomed = unit(&e, &mut s, Seat::P1, "LOF_256");
    s.card_mut(doomed).damage = e.hp(&s, doomed);
    for _ in 0..35 {
        unit(&e, &mut s, Seat::P1, "LOF_256");
        unit(&e, &mut s, Seat::P1, "LOF_192");
    }
    let source = unit(&e, &mut s, Seat::P1, "ASH_195");
    s.source = source;
    s.last_known_source = source;
    s.last_known_power = 1;
    s.ability_program = e.card_programs(&s, source)[0] + 1;
    s.ability_pc = 0;
    let actions = e.distribution_actions(&s).unwrap();
    assert_eq!(actions.len(), 72);
    assert!(!actions
        .iter()
        .any(|a| a.arg == Some(json!([[format!("c{}", doomed - 1), 1]]))));
}
#[test]
fn advantage_distribution_marker_precedes_specialized_early_returns_and_keeps_scan_bounds() {
    let c = catalogue();
    for code in ["ASH_208", "ASH_169"] {
        let original = c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == code)
            .unwrap()
            .clone();
        let load = |v: Value| {
            crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
                .unwrap()
        };
        assert!(load(original.clone()).cards.contains_key(code));
        let mut injected = original.clone();
        injected["unreviewedHelpers"] = json!({"kind":"distributeTokenUpgrade"});
        assert!(!load(injected).cards.contains_key(code));
        let mut oversized = original;
        oversized["unreviewedHelpers"] = json!(vec![Value::Null; 32769]);
        let loaded = load(oversized);
        assert!(!loaded.cards.contains_key(code));
        assert!(loaded.rejected[code].contains("capacity"));
    }
}
#[test]
fn advantage_distribution_all_attach_contingents_share_one_collective_window() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let sabine = unit(&e, &mut s, seat, "ASH_208");
        let host = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
        pending(&e, &mut s, seat, 6);
        let out = e
            .apply(&s, &allocation(&e, &s, &[(sabine, 3), (host, 3)]))
            .unwrap();
        assert_eq!(tokens(&e, &out, sabine).len(), 3);
        assert_eq!(tokens(&e, &out, host).len(), 3);
        assert_eq!(out.source, sabine);
        assert_eq!(out.decision, Decision::AbilityTarget);
        let done = e.apply(&out, &e.card_action(&out, enemy)).unwrap();
        assert!(done.card(enemy).exhausted);
        assert_eq!(
            done.decision,
            Decision::Window,
            "one collective trigger for all three attachments, not three windows"
        );
    }
}
#[test]
fn advantage_distribution_original_creation_parent_order_condition_and_requested_result() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "LOF_256");
    let b = unit(&e, &mut s, Seat::P1, "LOF_256");
    pending(&e, &mut s, Seat::P1, 6);
    let effect = e.program(s.ability_program - 1).instructions[s.ability_pc as usize]
        .group_effect()
        .unwrap()
        .clone();
    let first = s.next_id;
    e.capture_advantage_distribution(&mut s, &[(a, 2), (b, 4)])
        .unwrap();
    assert_eq!(e.advantage_creation_events(&s).unwrap(), (vec![b, a], 6));
    s.card_mut(a).damage = e.hp(&s, a);
    assert_eq!(e.advantage_creation_events(&s).unwrap(), (vec![b], 4));
    e.resolve_group(&mut s, &effect).unwrap();
    assert_eq!(s.ability_result, 4);
    assert!(s.ability_succeeded);
    assert!(tokens(&e, &s, a).is_empty());
    assert_eq!(
        tokens(&e, &s, b),
        (first + 2..first + 6).collect::<Vec<_>>()
    );
    assert!((first..first + 2).all(|id| s.card(id).parent == 0));
    let mut malformed = s;
    malformed.event_group.records[0].amount = 3;
    assert!(e
        .advantage_creation_events(&malformed)
        .unwrap_err()
        .contains("original"));
}
