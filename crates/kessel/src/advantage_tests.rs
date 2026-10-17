use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
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
    // This builder creates a fresh position with an explicitly empty Credit ledger.
    s.credit_state_exact=e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..12 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label}: {:?}", e.legal_actions(s)));
    e.apply(s, &action).unwrap()
}
fn give(e: &Engine, s: &mut State, host: CardId, amount: u16) {
    s.source = host;
    e.apply_effect(s,&serde_json::from_value(json!({"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":amount},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).unwrap()).unwrap();
    s.source = 0;
}
fn tokens(e: &Engine, s: &State, host: CardId) -> Vec<CardId> {
    e.upgrades(s, host)
        .into_iter()
        .filter(|id| Some(s.card(*id).def) == e.advantage_def)
        .collect()
}
fn play(e: &Engine, s: &State, code: &str) -> (State, CardId) {
    let mut state = *s;
    let source = add(e, &mut state, s.active, code);
    state.player_mut(s.active).hand.push(source).unwrap();
    let result = choose(e, &state, source);
    (result, source)
}
fn attack(e: &Engine, s: &State, source: CardId, target: CardId) -> State {
    let mut state = choose(e, s, source);
    if state.decision == Decision::AbilityMenu {
        state = button(e, &state, "Attack");
    }
    assert_eq!(state.decision, Decision::AttackTarget);
    choose(e, &state, target)
}
#[test]
fn advantage_complete_admission_census() {
    let e = engine();
    assert!(e.by_code.contains_key("TOKEN_ADVANTAGE"));
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../compiler/generated/runtime-admission.json"
    ))
    .unwrap();
    let known = original["runtimeAdmittedCards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["setCode"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let new = e
        .by_code
        .keys()
        .filter(|code| !known.contains(code.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    eprintln!("Advantage additions {}: {new:?}", new.len());

    for code in [
        "ASH_036", "ASH_157", "ASH_167", "ASH_178", "ASH_205", "ASH_218", "ASH_221", "ASH_238",
        "ASH_251", "ASH_254", "ASH_264",
    ] {
        eprintln!(
            "{code}: {} {:?}",
            e.by_code.contains_key(code),
            e.rejected_cards().unwrap().get(code)
        );
    }
}
#[test]
fn advantage_native_physical_stats_owner_capacity_and_snapshot() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let host = unit(&e, &mut s, seat.other(), "LOF_256");
        let power = e.power(&s, host, false);
        let hp = e.hp(&s, host);
        let before = s.zobrist();
        let first = s.next_id;
        give(&e, &mut s, host, 4);
        assert_eq!(e.power(&s, host, false), power + 4);
        assert_eq!(e.hp(&s, host), hp);
        assert_eq!(tokens(&e, &s, host), (first..first + 4).collect::<Vec<_>>());
        assert!(tokens(&e, &s, host)
            .iter()
            .all(|id| s.card(*id).owner == seat.other()));
        assert_ne!(before, s.zobrist());
        let snapshot = e.snapshot(&s);
        let loaded = e.load_state(&snapshot).unwrap();
        assert_eq!(snapshot, e.snapshot(&loaded));
        assert_eq!(s.zobrist(), loaded.zobrist());
        assert_eq!(s, loaded);
        let mut full = s;
        full.next_id = (MAX_CARDS - 2) as CardId;
        let copy = full;
        let effect:Effect=serde_json::from_value(json!({"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":4},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).unwrap();
        full.source = host;
        let before_full = full;
        assert!(e
            .apply_effect(&mut full, &effect)
            .unwrap_err()
            .contains("capacity"));
        assert_eq!(full, before_full);
        assert_eq!(copy.next_id, full.next_id);
        let first = tokens(&e, &s, host)[0];
        e.defeat(&mut s, first).unwrap();
        assert_eq!(tokens(&e, &s, host).len(), 3);
        assert_eq!(e.power(&s, host, false), power + 3);
        assert!(!s.player(seat.other()).discard.contains(first));
        e.defeat(&mut s, host).unwrap();
        assert!(tokens(&e, &s, host).is_empty());
        assert!((first..first + 4).all(|id| s.card(id).parent == 0));
        assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    }
}
#[test]
fn advantage_actual_self_play_ferry_and_zealous_both_backends_seats() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for (code, n) in [("ASH_218", 4), ("ASH_251", 1)] {
                let s = board(&e, seat);
                let (s, source) = play(&e, &s, code);
                assert_eq!(s.decision, Decision::Window);
                assert_eq!(tokens(&e, &s, source).len(), n);
                assert_eq!(
                    e.power(&s, source, false),
                    e.def(&s, source).info.power.unwrap() as u16 + n as u16
                );
            }
        }
    }
}
#[test]
fn advantage_native_survivors_share_owner_window_and_each_token_resolves_once() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let a = unit(&e, &mut s, seat, "LOF_256");
            let d = unit(&e, &mut s, seat.other(), "LOF_256");
            give(&e, &mut s, a, 2);
            give(&e, &mut s, d, 1);
            let next = s.next_id;
            let pending = attack(&e, &s, a, d);
            assert_eq!(pending.decision, Decision::DeathPlayer);
            assert_eq!(pending.card(a).damage, 2);
            assert_eq!(pending.card(d).damage, 3);
            assert_eq!(tokens(&e, &pending, a).len(), 2);
            assert_eq!(tokens(&e, &pending, d).len(), 1);
            assert_eq!(pending.death_window.len, 3);
            let mut done = button(&e, &pending, "You");
            assert_eq!(done.decision, Decision::DeathBatch);
            done = button(&e, &done, "Resolve all (2)");
            assert_eq!(done.decision, Decision::Window);
            assert!(tokens(&e, &done, a).is_empty());
            assert!(tokens(&e, &done, d).is_empty());
            assert_eq!(done.next_id, next);
            assert_eq!(e.power(&done, a, false), 1);
            assert_eq!(e.power(&done, d, false), 1);
            assert_eq!(done.attack_source, 0);
            assert_eq!(done.death_depth, 0);
            assert_eq!(done.death_pending.len, 0);
            let other = button(&e, &pending, "Opponent");
            assert_eq!(other.decision, Decision::DeathBatch);
            assert!(tokens(&e, &other, d).is_empty());
            let other = button(&e, &other, "Resolve all (2)");
            assert_eq!(done, other);
        }
    }
}
#[test]
fn advantage_native_dead_host_keeps_captured_ineffective_owner_bucket() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let a = unit(&e, &mut s, seat, "LOF_256");
        let d = unit(&e, &mut s, seat.other(), "LOF_256");
        s.card_mut(d).damage = 3;
        give(&e, &mut s, a, 1);
        give(&e, &mut s, d, 2);
        let dead_tokens = tokens(&e, &s, d);
        let pending = attack(&e, &s, a, d);
        assert_eq!(pending.decision, Decision::DeathPlayer);
        assert_eq!(pending.death_window.len, 3);
        let done = button(&e, &pending, "Opponent");
        assert_eq!(done.decision, Decision::Window);
        assert!(e.in_arena(&done, a));
        assert!(!e.in_arena(&done, d));
        assert_eq!(done.card(a).damage, 3);
        assert!(tokens(&e, &done, a).is_empty());
        assert!(
            dead_tokens
                .into_iter()
                .all(|id| done.card(id).parent == 0
                    && !done.player(seat.other()).discard.contains(id))
        );
        assert_eq!(done.death_depth, 0);
    }
}
#[test]
fn advantage_native_and_ordinary_attack_end_programs_share_one_order_window() {
    let e = engine();
    let seat = Seat::P1;
    let mut s = board(&e, seat);
    let a = unit(&e, &mut s, seat, "LAW_033");
    let low = unit(&e, &mut s, seat.other(), "LOF_256");
    give(&e, &mut s, a, 1);
    let base = s.player(seat.other()).base;
    let pending = attack(&e, &s, a, base);
    assert_eq!(pending.decision, Decision::DeathOrder);
    let labels = e
        .legal_actions(&pending)
        .iter()
        .map(|a| a.label.clone())
        .collect::<Vec<_>>();
    assert!(labels.iter().any(|s| s == "Defeat Advantage token"));
    let ordinary = labels.iter().find(|s| !s.contains("Advantage")).unwrap();
    let chosen = button(&e, &pending, ordinary);
    assert_eq!(chosen.decision, Decision::AbilityTarget);
    assert_eq!(tokens(&e, &chosen, a).len(), 1);
    let done = choose(&e, &chosen, low);
    assert!(done.player(seat.other()).discard.contains(low));
    assert!(tokens(&e, &done, a).is_empty());
    assert_eq!(done.attack_source, 0);
    assert_eq!(done.death_depth, 0);
    let first = button(&e, &pending, "Defeat Advantage token");
    assert!(tokens(&e, &first, a).is_empty());
    assert_eq!(first.decision, Decision::AbilityTarget);
    let done = choose(&e, &first, low);
    assert!(done.player(seat.other()).discard.contains(low));
    assert_eq!(done.attack_source, 0);
}
#[test]
fn advantage_missing_malformed_native_and_selected_pool_dependencies_fail_closed() {
    let c = catalogue();
    let full = engine();
    assert!(full.by_code.contains_key("ASH_251"));
    for mutation in [
        "omit",
        "hash",
        "power",
        "traits",
        "abilities",
        "identity",
        "type",
    ] {
        let mut v = c.clone();
        let cards = v["cards"].as_array_mut().unwrap();
        if mutation == "omit" {
            cards.retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
        } else {
            let token = cards
                .iter_mut()
                .find(|v| v["setCode"] == "TOKEN_ADVANTAGE")
                .unwrap();
            match mutation {
                "hash" => token["sourceHash"] = json!("unreviewed"),
                "power" => token["card"]["upgradePower"] = json!(2),
                "traits" => token["card"]["traits"] = json!([]),
                "abilities" => token["abilities"]["card"] = json!([{}]),
                "identity" => token["card"]["id"] = json!("different"),
                _ => token["card"]["types"] = json!(["upgrade"]),
            }
        }
        let e = Engine::from_catalogue_value(&v).unwrap();
        assert!(!e.by_code.contains_key("TOKEN_ADVANTAGE"), "{mutation}");
        for code in ["ASH_251", "ASH_218", "ASH_264", "ASH_184"] {
            assert!(!e.by_code.contains_key(code), "{mutation}: {code}");
            assert!(e.rejected_cards().unwrap()[code].contains("Advantage"));
        }
    }
    let mut pool = full.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["setCode"] != "TOKEN_ADVANTAGE");
    let bytes = serde_json::to_vec(&c).unwrap();
    let err = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes)
        .err()
        .unwrap();
    assert!(err.contains("TOKEN_ADVANTAGE"), "{err}");
    pool["cards"].as_array_mut().unwrap().retain(|v| {
        ![
            "ASH_036", "ASH_146", "ASH_157", "ASH_167", "ASH_176", "ASH_178", "ASH_205", "ASH_218",
            "ASH_221", "ASH_238", "ASH_251", "ASH_254", "ASH_264",
        ]
        .iter()
        .any(|code| v["setCode"] == *code)
    });
    // Follow Me is another physical Advantage creator in the selected pool.
    // Removing the older creators must not bypass its native-token dependency.
    let err = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes)
        .err()
        .unwrap();
    assert!(err.contains("TOKEN_ADVANTAGE"), "{err}");
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["setCode"] != "ASH_184");
    Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes).unwrap();
}
#[test]
fn advantage_raw_contract_rejects_unknown_or_nondefault_creation_semantics() {
    let original = json!({"kind":"giveAdvantage","amount":{"kind":"const","value":1},"target":"chosen","createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"});
    for (key, value) in [
        ("createdBy", json!("opponent")),
        ("batch", json!("perTarget")),
        ("tokenOwner", json!("abilityController")),
        ("highPriorityRemoval", json!(true)),
        ("amount", json!({"kind":"const","value":6})),
        ("target", json!("bothPlayers")),
    ] {
        let mut v = original.clone();
        v[key] = value;
        let ability = json!({"trigger":"whenPlayed","costs":[],"effect":v});
        assert!(
            serde_json::from_value::<crate::ability_ir::Ability>(ability)
                .map_err(|e| e.to_string())
                .and_then(|a| Program::compile(&a))
                .is_err(),
            "{key}"
        );
    }
    for key in ["createdBy", "batch", "tokenOwner"] {
        let mut v = original.clone();
        v.as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<Effect>(v).is_err());
    }
    let entries=catalogue();
    for code in ["ASH_251", "ASH_036"] {
        let original_entry=entries["cards"].as_array().unwrap().iter().find(|v|v["setCode"]==code).unwrap().clone();
        let admits=|entry:Value| crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[entry]})).unwrap().cards.contains_key(code);
        assert!(admits(original_entry.clone()),"{code}");
        for key in ["stateWatchers", "historyHelpers", "privatePrograms", "classMethods", "privateClassConstants", "inheritedSource", "deploymentOverride", "setupValidationOverrides"] {
            let mut bad=original_entry.clone();
            bad[key]=json!([{"kind":"unimplementedSourceContext"}]);
            assert!(!admits(bad),"{code}/{key}");
        }
    }
    for class in [
        "LeaderUnitCard",
        "UpgradeCard",
        "TokenUnitCard",
        "EventCard",
    ] {
        let mut v = catalogue();
        let entry = v["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["setCode"] == "ASH_251")
            .unwrap();
        entry["baseClass"] = json!(class);
        let e = Engine::from_catalogue_value(&v).unwrap();
        assert!(!e.by_code.contains_key("ASH_251"), "{class}");
    }
}
#[test]
fn advantage_actual_optional_targeted_play_and_flarestar_death_preserve_controller() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let ground = unit(&e, &mut s, seat.other(), "LOF_256");
            let space = unit(&e, &mut s, seat.other(), "JTL_247");
            let (prompt, source) = play(&e, &s, "ASH_238");
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert!(!e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, ground)));
            assert!(e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, space)));
            assert!(!e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, source)));
            let next = prompt.next_id;
            let pass = button(&e, &prompt, "Pass");
            assert_eq!(pass.next_id, next);
            let done = choose(&e, &prompt, space);
            assert_eq!(tokens(&e, &done, space).len(), 2);
            assert_eq!(done.card(next).owner, seat.other());
            let (prompt, shuttle) = play(&e, &s, "ASH_167");
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            let mut done = choose(&e, &prompt, ground);
            assert_eq!(tokens(&e, &done, ground).len(), 1);
            assert_eq!(done.active, seat.other());
            done.card_mut(space).phase_power = 5;
            let died = attack(&e, &done, space, shuttle);
            assert_eq!(died.decision, Decision::AbilityTarget);
            assert!(died.player(seat).discard.contains(shuttle));
            assert_eq!(died.active, seat);
            let next = died.next_id;
            let done = choose(&e, &died, ground);
            assert_eq!(tokens(&e, &done, ground).len(), 2);
            assert_eq!(done.card(next).owner, seat.other());
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.attack_source, 0);
        }
    }
}
#[test]
fn advantage_actual_danger_wingmen_attack_excludes_source_and_consumes_defender_token() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let a = unit(&e, &mut s, seat, "ASH_157");
            let d = unit(&e, &mut s, seat.other(), "JTL_059");
            let ally = unit(&e, &mut s, seat, "LOF_256");
            let prompt = attack(&e, &s, a, d);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert!(!e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, a)));
            assert!(e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, ally)));
            let pass = button(&e, &prompt, "Pass");
            assert_eq!(pass.card(a).damage, 3);
            assert!(tokens(&e, &pass, d).is_empty());
            let done = choose(&e, &prompt, d);
            assert_eq!(done.card(a).damage, 4);
            assert_eq!(done.card(d).damage, 4);
            assert!(tokens(&e, &done, d).is_empty());
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.next_id, prompt.next_id + 1);
            let done = choose(&e, &prompt, ally);
            assert_eq!(tokens(&e, &done, ally).len(), 1);
            assert_eq!(done.card(a).damage, 3);
            assert_eq!(done.decision, Decision::Window);
        }
    }
}
#[test]
fn advantage_actual_gallofree_mandatory_friendly_death_selector() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat.other());
            let host = unit(&e, &mut s, seat, "ASH_254");
            let ally = unit(&e, &mut s, seat, "LOF_256");
            let enemy = unit(&e, &mut s, seat.other(), "JTL_247");
            s.card_mut(enemy).phase_power = 5;
            let prompt = attack(&e, &s, enemy, host);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert_eq!(prompt.active, seat);
            assert!(prompt.player(seat).discard.contains(host));
            assert!(!e.legal_actions(&prompt).iter().any(|a| a.label == "Pass"));
            assert!(!e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, enemy)));
            let done = choose(&e, &prompt, ally);
            assert_eq!(tokens(&e, &done, ally).len(), 2);
            assert_eq!(done.decision, Decision::Window);
            assert_eq!(done.attack_source, 0);
        }
    }
}
#[test]
fn advantage_actual_spider_counts_both_enemy_arenas_and_helix_branches_live() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for count in [0, 1, 3] {
                let mut s = board(&e, seat);
                for i in 0..count {
                    unit(
                        &e,
                        &mut s,
                        seat.other(),
                        if i % 2 == 0 { "LOF_256" } else { "JTL_059" },
                    );
                }
                let friendly = unit(&e, &mut s, seat, "LOF_256");
                let (s, spider) = play(&e, &s, "ASH_178");
                assert_eq!(s.decision, Decision::Window);
                assert_eq!(tokens(&e, &s, spider).len(), count);
                assert!(tokens(&e, &s, friendly).is_empty());
                assert_eq!(e.power(&s, spider, false), 5 + count as u16);
                assert!(s.card(spider).hidden_qualified());
            }
            for space in [false, true] {
                let mut s = board(&e, seat);
                unit(
                    &e,
                    &mut s,
                    seat.other(),
                    if space { "JTL_059" } else { "LOF_256" },
                );
                let (s, helix) = play(&e, &s, "ASH_221");
                assert_eq!(s.decision, Decision::Window);
                assert_eq!(tokens(&e, &s, helix).len(), if space { 0 } else { 2 });
                assert_eq!(s.card(helix).shield != 0, space);
            }
        }
    }
}
#[test]
fn advantage_actual_up_to_selectors_zero_toggle_limit_order_and_ownership() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for (code, limit) in [("ASH_205", 3), ("ASH_264", 2)] {
                let mut s = board(&e, seat);
                let mut units = vec![];
                for i in 0..4 {
                    let u = unit(
                        &e,
                        &mut s,
                        if i % 2 == 0 { seat } else { seat.other() },
                        if i % 2 == 0 { "LOF_256" } else { "JTL_059" },
                    );
                    s.card_mut(u).exhausted = true;
                    units.push(u);
                }
                let ready = unit(&e, &mut s, seat, "LOF_256");
                let (prompt, source) = play(&e, &s, code);
                assert_eq!(prompt.decision, Decision::AbilityTarget);
                if code == "ASH_205" {
                    assert!(!e
                        .legal_actions(&prompt)
                        .contains(&e.card_action(&prompt, ready)));
                    assert!(e
                        .legal_actions(&prompt)
                        .contains(&e.card_action(&prompt, source)));
                }
                let zero = button(&e, &prompt, "Choose nothing");
                assert_eq!(zero.next_id, prompt.next_id);
                for id in &units {
                    assert!(tokens(&e, &zero, *id).is_empty());
                }
                let mut selected = choose(&e, &prompt, units[2]);
                selected = choose(&e, &selected, units[2]);
                assert_eq!(selected.ability_selection.len, 0);
                let order = units.iter().rev().take(limit).copied().collect::<Vec<_>>();
                for id in &order {
                    selected = choose(&e, &selected, *id);
                }
                assert_eq!(selected.next_id, prompt.next_id);
                assert!(!e
                    .legal_actions(&selected)
                    .contains(&e.card_action(&selected, units[0])));
                assert!(e
                    .apply(&selected, &e.card_action(&selected, units[0]))
                    .is_err());
                let done = button(&e, &selected, "Done");
                assert_eq!(done.next_id, prompt.next_id + limit as CardId);
                for (i, id) in order.iter().enumerate() {
                    assert_eq!(tokens(&e, &done, *id), vec![prompt.next_id + i as CardId]);
                    assert_eq!(
                        done.card(prompt.next_id + i as CardId).owner,
                        done.card(*id).owner
                    );
                }
                assert_eq!(done.decision, Decision::Window);
            }
        }
    }
}
#[test]
fn advantage_actual_support_donated_attack_end_rukh_thrawn_and_halo() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for code in ["ASH_036", "ASH_033", "ASH_223"] {
                for remove_donor in [false, true] {
                    let mut s = board(&e, seat);
                    let a = unit(&e, &mut s, seat, "LOF_256");
                    let d = unit(&e, &mut s, seat.other(), "LOF_256");
                    s.card_mut(d).damage = 3;
                    let (support, donor) = play(&e, &s, code);
                    assert_eq!(support.decision, Decision::Support);
                    let mut declared = choose(&e, &support, a);
                    assert_eq!(declared.decision, Decision::AttackTarget);
                    assert!(declared.support.grants[..declared.support.len as usize]
                        .iter()
                        .any(|g| g.target == a && g.donor == donor));
                    // Internal lifecycle boundary after an actual card's Support copy: removing
                    // the donor cannot revoke its already snapshotted granted registration.
                    if remove_donor {
                        e.defeat(&mut declared, donor).unwrap();
                    }
                    let result = choose(&e, &declared, d);
                    assert!(result.player(seat.other()).discard.contains(d));
                    assert_eq!(result.card(a).damage, 1);
                    let done = if code == "ASH_036" {
                        assert_eq!(result.decision, Decision::AbilityTarget);
                        let next = result.next_id;
                        let t = choose(&e, &result, a);
                        assert_eq!(tokens(&e, &t, a).len(), 3);
                        assert_eq!(t.next_id, next + 3);
                        t
                    } else {
                        assert_eq!(result.decision, Decision::Window);
                        if code == "ASH_033" {
                            assert!(!result.card(a).exhausted);
                        } else {
                            assert_ne!(result.card(a).shield, 0);
                        }
                        result
                    };
                    assert_eq!(done.decision, Decision::Window);
                    assert_eq!(done.attack_source, 0);
                    assert_eq!(done.death_depth, 0);
                    assert_eq!(done.support.len, 0);
                    if !remove_donor {
                        assert!(e.in_arena(&done, donor));
                        assert!(done.card(donor).exhausted);
                        assert!(tokens(&e, &done, donor).is_empty());
                        assert_eq!(done.card(donor).shield, 0);
                    }
                }
            }
        }
    }
}
#[test]
fn advantage_actual_rukh_thrawn_halo_own_attack_end_and_rukh_decline() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for code in ["ASH_036", "ASH_033", "ASH_223"] {
                let mut s = board(&e, seat);
                let a = unit(&e, &mut s, seat, code);
                let space = e.def(&s, a).info.arena.as_deref() == Some("space");
                let d = unit(
                    &e,
                    &mut s,
                    seat.other(),
                    if space { "JTL_247" } else { "LOF_256" },
                );
                s.card_mut(d).damage = e.hp(&s, d) - 1;
                let result = attack(&e, &s, a, d);
                assert!(result.player(seat.other()).discard.contains(d));
                let done = if code == "ASH_036" {
                    assert_eq!(result.decision, Decision::AbilityTarget);
                    let before = result.next_id;
                    let pass = button(&e, &result, "Pass");
                    assert_eq!(pass.next_id, before);
                    assert!(tokens(&e, &pass, a).is_empty());
                    let t = choose(&e, &result, a);
                    assert_eq!(tokens(&e, &t, a).len(), 3);
                    assert_eq!(t.next_id, before + 3);
                    t
                } else {
                    assert_eq!(result.decision, Decision::Window);
                    if code == "ASH_033" {
                        assert!(!result.card(a).exhausted);
                    } else {
                        assert_ne!(result.card(a).shield, 0);
                    }
                    result
                };
                assert_eq!(done.decision, Decision::Window);
                assert_eq!(done.attack_source, 0);
            }
        }
    }
}
#[test]
fn advantage_support_retained_attack_end_program_roundtrips_and_rejects_unreviewed_programs() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "LOF_256");
    let d = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(d).damage = 3;
    let (support, _) = play(&e, &s, "ASH_036");
    let declared = choose(&e, &support, a);
    let passed = button(&e, &declared, "Pass attack");
    assert_eq!(passed.decision, Decision::Window);
    assert_eq!(passed.support.len, 1);
    let snap = e.snapshot(&passed);
    let restored = e.load_state(&snap).unwrap();
    assert_eq!(restored, passed);
    assert_eq!(restored.zobrist(), passed.zobrist());
    let mut bad = snap.clone();
    bad["supportGrants"][0]["programs"][0] = json!("unknown-program-hash");
    assert!(e.load_state(&bad).is_err());
    let mut bad = snap.clone();
    bad["supportGrants"][0]["programs"] = json!(vec![
        snap["supportGrants"][0]["programs"][0]
            .clone();
        MAX_SUPPORT_PROGRAMS + 1
    ]);
    assert!(e.load_state(&bad).is_err());
    let restored = button(&e, &restored, "Pass");
    let result = attack(&e, &restored, a, d);
    assert_eq!(result.decision, Decision::AbilityTarget);
    let done = choose(&e, &result, a);
    assert_eq!(tokens(&e, &done, a).len(), 3);
    assert_eq!(done.support.len, 0);
    assert_eq!(done.attack_source, 0);
}
#[test]
fn advantage_native_trigger_capacity_is_explicit_and_attack_is_atomic_on_overflow() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "LOF_256");
    for _ in 0..13 {
        give(&e, &mut s, a, 5);
    }
    let declared = choose(&e, &s, a);
    let before = declared;
    let action = e.card_action(&declared, declared.players[1].base);
    let err = e.apply(&declared, &action).unwrap_err();
    assert!(
        err.contains("Advantage attack-end trigger capacity"),
        "{err}"
    );
    assert_eq!(declared, before);
}
#[test]
fn advantage_support_copied_rukh_trigger_survives_attacker_death_and_grant_cleanup() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "LOF_256");
    let d = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(a).damage = 3;
    s.card_mut(d).damage = 3;
    give(&e, &mut s, d, 1);
    let (support, donor) = play(&e, &s, "ASH_036");
    let declared = choose(&e, &support, a);
    let pending = choose(&e, &declared, d);
    assert_eq!(pending.decision, Decision::DeathPlayer);
    assert!(pending.player(Seat::P1).discard.contains(a));
    assert!(pending.player(Seat::P2).discard.contains(d));
    assert_eq!(pending.support.len, 0);
    assert!(pending.attack_attacker_defeated);
    assert!(pending.attack_defender_defeated);
    let target = button(&e, &pending, "You");
    assert_eq!(target.decision, Decision::AbilityTarget);
    let done = choose(&e, &target, donor);
    assert_eq!(tokens(&e, &done, donor).len(), 3);
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(done.death_depth, 0);
    assert_eq!(done.attack_source, 0);
}
#[test]
fn advantage_actual_rukh_granted_tokens_survive_the_same_attack_end_window() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "ASH_036");
    let d = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(d).damage = 3;
    give(&e, &mut s, a, 1);
    let old = tokens(&e, &s, a)[0];
    let first_new = s.next_id;
    let order = attack(&e, &s, a, d);
    assert_eq!(order.decision, Decision::DeathOrder);
    let target = button(&e, &order, "Give 3 Advantage tokens to a unit");
    assert_eq!(target.decision, Decision::AbilityTarget);
    let done = choose(&e, &target, a);
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(
        tokens(&e, &done, a),
        (first_new..first_new + 3).collect::<Vec<_>>()
    );
    assert_eq!(done.card(old).parent, 0);
    assert_eq!(done.next_id, first_new + 3);
    assert_eq!(done.attack_source, 0);
}
#[test]
fn advantage_actual_follower_upgraded_hp_disappears_on_native_defeat_and_sweeps_unit() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "SHD_056");
    let printed_hp = e.def(&s, host).info.hp.unwrap() as u16;
    let printed_power = e.def(&s, host).info.power.unwrap() as u16;
    give(&e, &mut s, host, 1);
    assert_eq!(e.hp(&s, host), printed_hp + 1);
    assert_eq!(e.power(&s, host, false), printed_power + 2);
    s.card_mut(host).damage = printed_hp;
    let done = attack(&e, &s, host, s.players[1].base);
    assert_eq!(done.card(s.players[1].base).damage, printed_power + 2);
    assert!(done.player(Seat::P1).discard.contains(host));
    assert!(tokens(&e, &done, host).is_empty());
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(done.attack_source, 0);
}
