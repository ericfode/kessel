use super::*;
use crate::model::PhaseHistory;
use crate::phase_history::Query;
fn query(code: &str) -> Query {
    fn find(v: &Value) -> Option<Value> {
        if v["kind"] == "historyQuery" {
            return Some(v.clone());
        }
        match v {
            Value::Object(o) => o.values().find_map(find),
            Value::Array(a) => a.iter().find_map(find),
            _ => None,
        }
    }
    let c: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let entry = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["setCode"] == code)
        .unwrap();
    let mut q = find(&entry["abilities"]).unwrap();
    q.as_object_mut().unwrap().remove("kind");
    serde_json::from_value(q).unwrap()
}
fn fund(e: &Engine, s: &mut State, seat: Seat, id: CardId) {
    while s.player(seat).resources.len < e.cost(s, id) as u8 {
        let r = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(r).unwrap();
    }
}
fn play(e: &Engine, s: &State, id: CardId) -> State {
    let mut s = *s;
    s.active = s.card(id).owner;
    s.action_player = Some(s.active);
    let seat = s.card(id).owner;
    fund(e, &mut s, seat, id);
    e.apply(&s, &e.card_action(&s, id)).unwrap()
}
fn draw_card(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "LOF_254");
    s.player_mut(seat).deck.push(id).unwrap();
    id
}
#[test]
fn phase_history_disposable_draws_for_other_friendly_entry_not_placement_or_self() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for prior in [
            "none",
            "placement",
            "self",
            "enemy",
            "friendly",
            "departed-friendly",
        ] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = board(&e);
            let b1 = hand(&e, &mut s, "HMW_103");
            let drawn = draw_card(&e, &mut s, Seat::P1);
            if prior == "self" {
                s.phase_history.entered(Seat::P1, b1);
            }
            if ["placement", "enemy", "friendly", "departed-friendly"].contains(&prior) {
                let owner = if prior == "enemy" { Seat::P2 } else { Seat::P1 };
                let prior_card = add(&e, &mut s, owner, "LOF_254");
                s.player_mut(owner).hand.push(prior_card).unwrap();
                if prior == "placement" {
                    s.player_mut(owner).hand.remove(prior_card);
                    e.add_arena(&mut s, prior_card).unwrap();
                } else {
                    s = play(&e, &s, prior_card);
                    if prior == "departed-friendly" {
                        e.defeat(&mut s, prior_card).unwrap();
                    }
                }
            }
            let played = play(&e, &s, b1);
            assert_eq!(
                played.players[0].hand.contains(drawn),
                ["friendly", "departed-friendly"].contains(&prior),
                "{backend:?}:{prior}"
            );
            assert!(PhaseHistory::contains(
                &played.phase_history.entered,
                Seat::P1,
                b1
            ));
        }
    }
}
#[test]
fn phase_history_hello_there_selects_both_controllers_entries_and_changes_stats() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e);
        let untouched = unit(&e, &mut s, Seat::P1, "LOF_256");
        let entered = add(&e, &mut s, Seat::P2, "LOF_256");
        s.players[1].hand.push(entered).unwrap();
        s = play(&e, &s, entered);
        let event = hand(&e, &mut s, "TWI_052");
        s.active = Seat::P1;
        s.action_player = Some(Seat::P1);
        let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
        let ids = e
            .legal_actions(&prompt)
            .iter()
            .filter_map(Action::id)
            .collect::<Vec<_>>();
        assert!(ids.contains(&entered));
        assert!(!ids.contains(&untouched));
        let resolved = e.apply(&prompt, &e.card_action(&prompt, entered)).unwrap();
        assert!(resolved.players[1].discard.contains(entered));
        assert!(e.in_arena(&resolved, untouched));
        assert_eq!(e.hp(&resolved, untouched), 4);
        assert_eq!(resolved.phase_history.play_counts, [1, 1]);
    }
}
#[test]
fn phase_history_caretaker_uses_played_by_snapshot_and_live_force_trait() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for prior in [
            "placement",
            "enemy-play",
            "friendly-play",
            "friendly-departed",
        ] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = board(&e);
            let matron = unit(&e, &mut s, Seat::P1, "LOF_243");
            let drawn = draw_card(&e, &mut s, Seat::P1);
            let owner = if prior == "enemy-play" {
                Seat::P2
            } else {
                Seat::P1
            };
            let force = add(&e, &mut s, owner, "LOF_256");
            if prior == "placement" {
                e.add_arena(&mut s, force).unwrap();
            } else {
                s.player_mut(owner).hand.push(force).unwrap();
                s = play(&e, &s, force);
            }
            if prior == "friendly-departed" {
                e.defeat(&mut s, force).unwrap();
            }
            s.active = Seat::P1;
            s.action_player = Some(Seat::P1);
            s.source = matron;
            let p = e.available_programs(&s, matron, "action")[0];
            e.begin_program(&mut s, p, 0).unwrap();
            if s.decision == Decision::AbilityNoEffect {
                let action = e
                    .legal_actions(&s)
                    .into_iter()
                    .find(|a| a.label == "Use it anyway")
                    .unwrap();
                s = e.apply(&s, &action).unwrap();
            }
            assert!(s.card(matron).exhausted);
            assert_eq!(
                s.players[0].hand.contains(drawn),
                ["friendly-play", "friendly-departed"].contains(&prior),
                "{prior}"
            );
            // Shared projection diagnostic: a historical record's card getter is live.
            let q = query("LOF_243");
            let mut live = s;
            live.phase_history.played(Seat::P1, force).unwrap();
            assert!(e.history_boolean(&live, matron, 0, &q));
            e.definitions[live.card(force).def as usize]
                .traits
                .retain(|t| t != "force");
            assert!(!e.history_boolean(&live, matron, 0, &q));
        }
    }
}
#[test]
fn phase_history_talzin_repeated_opponent_plays_count_and_two_weakness_resolve() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for count in [0, 1, 2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = board(&e);
            let target = unit(&e, &mut s, Seat::P2, "LOF_256");
            let replayed = add(&e, &mut s, Seat::P2, "LOF_254");
            for n in 0..count {
                if n > 0 {
                    e.leave_play(&mut s, replayed).unwrap();
                    s.players[1].discard.remove(replayed);
                }
                s.players[1].hand.push(replayed).unwrap();
                s = play(&e, &s, replayed);
            }
            assert_eq!(s.phase_history.play_counts[1], count);
            let shuttle = hand(&e, &mut s, "HMW_040");
            s.active = Seat::P1;
            s.action_player = Some(Seat::P1);
            let prompt = play(&e, &s, shuttle);
            let mut resolved = prompt;
            for _ in 0..5 {
                let actions = e.legal_actions(&resolved);
                if let Some(a) = actions.iter().find(|a| a.id() == Some(target)).cloned() {
                    resolved = e.apply(&resolved, &a).unwrap();
                    break;
                }
                if let Some(a) = actions
                    .iter()
                    .find(|a| a.label == "Trigger" || a.label == "Use it anyway")
                    .cloned()
                {
                    resolved = e.apply(&resolved, &a).unwrap();
                } else {
                    break;
                }
            }
            let weaknesses = e
                .upgrades(&resolved, target)
                .into_iter()
                .filter(|id| Some(resolved.card(*id).def) == e.weakness_def)
                .count();
            assert_eq!(
                weaknesses,
                if count >= 2 { 2 } else { 0 },
                "{backend:?}:{count}; {:?}",
                e.legal_actions(&resolved)
            );
            if count >= 2 {
                assert_eq!(e.hp(&resolved, target), 2);
            }
        }
    }
}
#[test]
fn phase_history_deploy_and_token_entry_are_entries_not_card_plays() {
    let e = engine();
    let mut s = board(&e);
    let leader = s.players[0].leader;
    for _ in 0..8 {
        let r = add(&e, &mut s, Seat::P1, "LOF_254");
        s.players[0].resources.push(r).unwrap();
    }
    let menu = e.apply(&s, &e.card_action(&s, leader)).unwrap();
    let action = e
        .legal_actions(&menu)
        .into_iter()
        .find(|a| a.label.starts_with("Deploy "))
        .unwrap();
    let deployed = e.apply(&menu, &action).unwrap();
    assert!(PhaseHistory::contains(
        &deployed.phase_history.entered,
        Seat::P1,
        leader
    ));
    assert_eq!(deployed.phase_history.play_counts, [0, 0]);
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.source = source;
    let effect = Effect::CreateTokenUnit {
        token_type: "battleDroid".into(),
        amount: Expr::number(2),
        target: Target::Name("ownPlayer".into()),
        enters_ready: false,
    };
    e.generate_group(&mut s, &effect).unwrap();
    e.resolve_group(&mut s, &effect).unwrap();
    let tokens = s.players[0]
        .ground
        .as_slice()
        .iter()
        .copied()
        .filter(|id| *id != source)
        .collect::<Vec<_>>();
    assert_eq!(tokens.len(), 2);
    for id in tokens {
        assert!(PhaseHistory::contains(
            &s.phase_history.entered,
            Seat::P1,
            id
        ));
    }
    assert_eq!(s.phase_history.play_counts, [0, 0]);
    assert!(!PhaseHistory::contains(
        &s.phase_history.entered,
        Seat::P1,
        source
    ));
}
#[test]
fn phase_history_import_hash_copy_reset_and_overflow_are_explicit() {
    let e = engine();
    let mut s = board(&e);
    let id = unit(&e, &mut s, Seat::P1, "LOF_256");
    s.phase_history.entered(Seat::P2, id);
    s.phase_history.played(Seat::P2, id).unwrap();
    s.phase_history.played(Seat::P2, id).unwrap();
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["phaseHistory"]["playCounts"], json!([0, 2]));
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(restored.phase_history, s.phase_history);
    assert_eq!(restored.zobrist(), s.zobrist());
    let mut copied = s;
    copied.phase_history.entered(Seat::P1, id);
    assert_ne!(copied.zobrist(), s.zobrist());
    assert_ne!(copied.phase_history, s.phase_history);
    let mut missing = snapshot.clone();
    missing.as_object_mut().unwrap().remove("phaseHistory");
    assert!(e.load_state(&missing).unwrap_err().contains("phaseHistory"));
    for bad in [
        json!({"version":2,"entered":[],"played":[],"playCounts":[0,0]}),
        json!({"version":1,"entered":[],"played":[],"playCounts":[0,0],"ignored":true}),
        json!({"version":1,"entered":[{"card":"c999","playedBy":"p1"}],"played":[],"playCounts":[0,0]}),
        json!({"version":1,"entered":[],"played":[{"card":format!("c{}",id-1),"playedBy":"p2"}],"playCounts":[0,0]}),
    ] {
        let mut bad_snapshot = snapshot.clone();
        bad_snapshot["phaseHistory"] = bad;
        assert!(e.load_state(&bad_snapshot).is_err());
    }
    let mut overflow = s;
    overflow.phase_history.play_counts[0] = u16::MAX;
    let before = overflow.phase_history;
    assert!(overflow.phase_history.played(Seat::P1, id).is_err());
    assert_eq!(overflow.phase_history, before);
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.phase_history, PhaseHistory::default());
    eprintln!(
        "phase_history State bytes {}; PhaseHistory bytes {}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<PhaseHistory>()
    );
}
#[test]
fn phase_history_admits_four_complete_cards_and_rejects_unknown_filter_scope() {
    let c: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let e = engine();
    for code in ["HMW_103", "TWI_052", "LOF_243", "HMW_040"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        let entry = c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["setCode"] == code)
            .unwrap();
        for mutation in [
            "extra-side",
            "extra-ability",
            "unknown-query",
            "wrong-read-time",
            "lambda-root",
            "leader",
            "attach",
            "continuation",
        ] {
            let mut entry = entry.clone();
            fn mutate_query(v: &mut Value, mutation: &str) -> bool {
                if v["kind"] == "historyQuery" {
                    match mutation {
                        "unknown-query" => v["ignored"] = json!(true),
                        "wrong-read-time" => v["readAt"] = json!("cardCreation"),
                        _ => v["args"][0]["bindings"] = json!(["wrongBinding"]),
                    }
                    return true;
                }
                match v {
                    Value::Object(o) => o.values_mut().any(|v| mutate_query(v, mutation)),
                    Value::Array(a) => a.iter_mut().any(|v| mutate_query(v, mutation)),
                    _ => false,
                }
            }
            match mutation {
                "extra-side" => entry["abilities"]["unit"] = entry["abilities"]["card"].clone(),
                "extra-ability" => {
                    let a = entry["abilities"]["card"][0].clone();
                    entry["abilities"]["card"].as_array_mut().unwrap().push(a);
                }
                "leader" => {
                    entry["baseClass"] = json!("LeaderUnitCard");
                    entry["card"]["types"] = json!(["leader"]);
                }
                "attach" => entry["attachCondition"] = json!({"kind":"const","value":true}),
                "continuation" => {
                    entry["abilities"]["card"][0]["then"] =
                        json!({"title":"extra","effect":{"kind":"noAction"}})
                }
                _ => {
                    assert!(mutate_query(&mut entry, mutation));
                }
            }
            let loaded = crate::ability_ir::Catalogue::load(
                &json!({"schema":"swu-card-ir/v1","cards":[entry]}),
            )
            .unwrap();
            assert!(!loaded.cards.contains_key(code), "{code}:{mutation}");
        }
    }
}

#[test]
fn phase_history_departed_token_identity_and_legacy_selected_pool_roundtrip() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.source = source;
    let effect = Effect::CreateTokenUnit {
        token_type: "battleDroid".into(),
        amount: Expr::number(1),
        target: Target::Name("ownPlayer".into()),
        enters_ready: false,
    };
    e.generate_group(&mut s, &effect).unwrap();
    e.resolve_group(&mut s, &effect).unwrap();
    let token = s.next_id - 1;
    e.defeat(&mut s, token).unwrap();
    s.source = 0;
    let snapshot = e.snapshot(&s);
    let restored = e.load_state(&snapshot).unwrap();
    assert!(PhaseHistory::contains(
        &restored.phase_history.entered,
        Seat::P1,
        token
    ));
    assert!(!restored.card(token).present);
    assert_eq!(e.snapshot(&restored), snapshot);
    // The legacy strict pool cannot reach new history mechanics. Its missing
    // history stays unknown across actions, copies and exports, with no suffix
    // fabricated from its current arena or from newly observed events.
    let legacy = Engine::from_compiled_pool_bytes(
        include_bytes!("../data/pool.json"),
        include_bytes!("../../../compiler/generated/catalog.json"),
    )
    .unwrap();
    let mut s = board(&legacy);
    s.phase_history.exact = false;
    let snapshot = legacy.snapshot(&s);
    assert!(snapshot.get("phaseHistory").is_none());
    let restored = legacy.load_state(&snapshot).unwrap();
    assert!(!restored.phase_history.exact);
    assert_eq!(legacy.snapshot(&restored), snapshot);
    let mut history = restored.phase_history;
    history.entered(Seat::P1, source);
    history.played(Seat::P1, source).unwrap();
    assert_eq!(history, restored.phase_history);
}
#[test]
fn phase_history_talzin_two_weakness_are_allocated_before_lethal_stat_sweep() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e);
        let prior = unit(&e, &mut s, Seat::P2, "LOF_256");
        s.phase_history.played(Seat::P2, prior).unwrap();
        s.phase_history.played(Seat::P2, prior).unwrap();
        let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
        let shuttle = hand(&e, &mut s, "HMW_040");
        let mut prompt = play(&e, &s, shuttle);
        if prompt.decision == Decision::AbilityOptional {
            let action = e
                .legal_actions(&prompt)
                .into_iter()
                .find(|a| a.label == "Trigger")
                .unwrap();
            prompt = e.apply(&prompt, &action).unwrap();
        }
        let next = prompt.next_id;
        let resolved = e.apply(&prompt, &e.card_action(&prompt, victim)).unwrap();
        assert!(resolved.players[1].discard.contains(victim));
        assert_eq!(resolved.next_id, next + 2);
        for id in next..next + 2 {
            assert_eq!(Some(resolved.card(id).def), e.weakness_def);
            assert_eq!(resolved.card(id).parent, 0);
        }
    }
}
