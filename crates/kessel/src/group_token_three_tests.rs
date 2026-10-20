//! Source-derived runtime tests; differential witnesses are recorded separately.
use super::*;
use crate::ability_ir::Ability;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
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
    s.credit_state_exact = e.needs_credit_state();
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
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_158")
        .unwrap()
        .clone()
}
fn admitted(v: Value) -> bool {
    crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
        .unwrap()
        .cards
        .contains_key("ASH_158")
}
fn staged(e: &Engine, seat: Seat) -> (State, CardId, CardId, CardId) {
    let mut s = board(e, seat);
    let friendly = unit(e, &mut s, seat, "LOF_254");
    let enemy = unit(e, &mut s, seat.other(), "LOF_254");
    let han = add(e, &mut s, seat, "ASH_158");
    s.player_mut(seat).hand.push(han).unwrap();
    (s, han, friendly, enemy)
}
#[test]
fn token_three_actual_card_is_admitted_and_whole_source_stays_closed() {
    let e = engine();
    assert!(
        e.by_code.contains_key("ASH_158"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_158")
    );
    let original = entry();
    assert!(admitted(original.clone()));
    let mut renamed = original.clone();
    renamed["setCode"] = json!("TEST_HAN");
    renamed["internalName"] = json!("same-semantics");
    assert!(crate::ability_ir::Catalogue::load(
        &json!({"schema":"swu-card-ir/v1","cards":[renamed]})
    )
    .unwrap()
    .cards
    .contains_key("TEST_HAN"));
    for (pointer, value) in [
        ("/abilities/card/0/trigger", json!("onAttack")),
        ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
        ("/abilities/card/0/effect/effects/0/target", json!("chosen")),
        ("/abilities/card/0/effect/effects/0/amount/value", json!(2)),
        (
            "/abilities/card/0/effect/effects/1/targetResolver/owner",
            json!("self"),
        ),
        (
            "/abilities/card/0/effect/effects/1/targetResolver/zone",
            json!("arena"),
        ),
        (
            "/abilities/card/0/effect/effects/1/effect/effects/0/amount/value",
            json!(4),
        ),
        (
            "/abilities/card/0/effect/effects/1/effect/effects/0/target",
            json!("self"),
        ),
        (
            "/abilities/card/0/effect/effects/1/effect/effects/0/createdBy",
            json!("opponent"),
        ),
        ("/baseClass", json!("EventCard")),
        ("/card/types", json!(["unit", "token"])),
        ("/card/hp", json!(8)),
        ("/card/keywords", json!([])),
        ("/keywordValues/saboteur", json!(0)),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(!admitted(bad), "{pointer}");
    }
    for (key, value) in [
        ("optional", json!(true)),
        ("requiresConfirmation", json!(true)),
        ("condition", json!({"kind":"const","value":true})),
        ("unknown", json!(true)),
    ] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][key] = value;
        assert!(!admitted(bad), "{key}");
    }
    for key in ["privatePrograms", "classMethods", "stateWatchers"] {
        let mut bad = original.clone();
        bad[key] = json!([{"extra":true}]);
        assert!(!admitted(bad));
    }
    for side in ["leader", "unit", "card"] {
        let mut bad = original.clone();
        bad["abilities"][side]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(!admitted(bad));
    }
    let a: Ability = serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
    assert!(Program::compile(&a).is_ok());
    for (key, value) in [
        ("optional", json!(true)),
        ("trigger", json!("event")),
        ("condition", json!({"kind":"const","value":true})),
    ] {
        let mut bad = original["abilities"]["card"][0].clone();
        bad[key] = value;
        assert!(Program::compile(&serde_json::from_value(bad).unwrap()).is_err());
    }
    let mut bad = original["abilities"]["card"][0].clone();
    bad["effect"]["effects"][1]["effect"]["effects"][0]["extra"] = json!(true);
    assert!(serde_json::from_value::<Ability>(bad).is_err());
}
#[test]
fn token_three_play_selects_incoming_own_enemy_and_leader_and_allocates_in_order() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for choice in 0..5 {
            let (mut s, han, friendly, enemy) = staged(&e, seat);
            let leader = s.player(seat.other()).leader;
            s.card_mut(leader).deployed = true;
            e.add_arena(&mut s, leader).unwrap(); // Synthetic predeployed setup, not an extra action.
            give(&e, &mut s, enemy, 1);
            let previous = tokens(&e, &s, enemy);
            let space = unit(&e, &mut s, seat.other(), "JTL_251");
            let target = [han, friendly, enemy, leader, space][choice];
            let prompt = choose(&e, &s, han);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert_eq!(
                prompt.card(han).damage,
                0,
                "capture waits for mandatory selection"
            );
            let actions = e.legal_actions(&prompt);
            for id in [han, friendly, enemy, leader, space] {
                assert!(
                    actions.contains(&e.card_action(&prompt, id)),
                    "missing {id}: {actions:?}"
                );
            }
            assert!(!actions
                .iter()
                .any(|a| a.label == "Pass" || a.label == "Cancel"));
            let next = prompt.next_id;
            let done = choose(&e, &prompt, target);
            assert_eq!(done.card(han).damage, 3);
            assert_eq!(done.next_id, next + 3);
            let created = (next..next + 3).collect::<Vec<_>>();
            let actual = tokens(&e, &done, target);
            let expected = if target == enemy {
                previous
                    .iter()
                    .copied()
                    .chain(created.iter().copied())
                    .collect()
            } else {
                created.clone()
            };
            assert_eq!(actual, expected);
            for token in created {
                assert_eq!(done.card(token).owner, done.card(target).owner);
                assert_eq!(done.card(token).parent, target);
            }
            assert_eq!(
                e.power(&done, target, false),
                e.power(&prompt, target, false) + 3
            );
            assert_eq!(e.hp(&done, target), e.hp(&prompt, target));
            assert!(e.has_keyword(&done, han, "saboteur"));
        }
    }
}
#[test]
fn token_three_self_damage_shield_does_not_cancel_creation_and_lethal_cleanup_allocates_all() {
    let e = engine();
    for shield in [false, true] {
        let (mut s, han, _, enemy) = staged(&e, Seat::P1);
        // Synthetic damaged incoming source: isolates deferred lethal ordering.
        s.card_mut(han).damage = 5;
        let mut prompt = choose(&e, &s, han);
        if shield {
            prompt.source = han;
            e.apply_effect(&mut prompt,&serde_json::from_value(json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":1}})).unwrap()).unwrap();
        }
        let next = prompt.next_id;
        let done = choose(&e, &prompt, han);
        assert_eq!(
            done.next_id,
            next + 3,
            "all three identities allocated before damage death"
        );
        if shield {
            assert!(e.in_arena(&done, han));
            assert_eq!(done.card(han).damage, 5);
            assert_eq!(tokens(&e, &done, han).len(), 3);
            assert_eq!(done.card(han).shield, 0);
        } else {
            assert!(!e.in_arena(&done, han));
            for id in next..next + 3 {
                assert_eq!(done.card(id).parent, 0);
                assert!(done.card(id).present);
            }
            assert!(done.player(Seat::P1).discard.contains(han));
        }
        assert!(tokens(&e, &done, enemy).is_empty());
    }
}

#[test]
fn token_three_unique_discarded_incoming_source_keeps_captured_mandatory_grant() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for keep_old in [false, true] {
            let (mut s, han, friendly, _) = staged(&e, seat);
            let old = unit(&e, &mut s, seat, "ASH_158");
            let unique = choose(&e, &s, han);
            assert_eq!(unique.decision, Decision::Unique);
            let victim = if keep_old { han } else { old };
            let prompt = choose(&e, &unique, victim);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert!(!e
                .legal_actions(&prompt)
                .contains(&e.card_action(&prompt, victim)));
            let done = choose(&e, &prompt, friendly);
            assert_eq!(tokens(&e, &done, friendly).len(), 3);
            assert!(done.player(seat).discard.contains(victim));
            assert_eq!(done.card(han).damage, if keep_old { 0 } else { 3 });
        }
    }
}
#[test]
fn token_three_requires_exact_native_token_even_in_selected_pool() {
    let c = catalogue();
    for mutation in ["omit", "power", "hash", "types"] {
        let mut v = c.clone();
        let cards = v["cards"].as_array_mut().unwrap();
        if mutation == "omit" {
            cards.retain(|e| e["setCode"] != "TOKEN_ADVANTAGE");
        } else {
            let token = cards
                .iter_mut()
                .find(|e| e["setCode"] == "TOKEN_ADVANTAGE")
                .unwrap();
            match mutation {
                "power" => token["card"]["upgradePower"] = json!(2),
                "hash" => token["sourceHash"] = json!("changed"),
                _ => token["card"]["types"] = json!(["upgrade"]),
            }
        }
        let e = Engine::from_catalogue_value(&v).unwrap();
        assert!(!e.by_code.contains_key("ASH_158"), "{mutation}");
        assert!(e.rejected_cards().unwrap()["ASH_158"].contains("Advantage"));
    }
    let e = engine();
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["setCode"] != "TOKEN_ADVANTAGE");
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&c).unwrap()
    )
    .is_err());
}
#[test]
fn token_three_han_native_saboteur_bypasses_sentinel_and_removes_shield() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let han = unit(&e, &mut s, seat, "ASH_158");
        let sentinel = unit(&e, &mut s, seat.other(), "ASH_252");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
        assert!(e.has_keyword(&s, sentinel, "sentinel"));
        s.source = enemy;
        e.apply_effect(
            &mut s,
            &serde_json::from_value(
                json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":1}}),
            )
            .unwrap(),
        )
        .unwrap();
        s.source = 0;
        let targets = e.attack_targets(&s, han, false);
        assert!(targets.contains(&enemy));
        assert!(targets.contains(&s.player(seat.other()).base));
        let prompt = choose(&e, &s, han);
        let done = choose(&e, &prompt, enemy);
        assert_eq!(done.card(enemy).shield, 0);
        assert!(
            !e.in_arena(&done, enemy),
            "Saboteur's attack damages through the removed Shield"
        );
    }
}

#[test]
fn token_three_standalone_group_mutants_preserve_general_mixed_guards() {
    let original = entry()["abilities"]["card"][0].clone();
    for kind in ["giveExperience", "giveShield", "giveWeakness"] {
        let mut bad = original.clone();
        let token = &mut bad["effect"]["effects"][1]["effect"]["effects"][0];
        *token = json!({"kind":kind,"target":"chosen","amount":{"kind":"const","value":3}});
        let a: Ability = serde_json::from_value(bad).unwrap();
        assert!(Program::compile(&a).is_err(), "{kind}");
    }
    for amount in [0, 1, 2, 4, 5, 33] {
        let mut bad = original.clone();
        bad["effect"]["effects"][1]["effect"]["effects"][0]["amount"]["value"] = json!(amount);
        assert!(
            Program::compile(&serde_json::from_value(bad).unwrap()).is_err(),
            "{amount}"
        );
    }
    for which in ["reverse", "duplicate", "flatten", "nested"] {
        let mut bad = original.clone();
        let effects = bad["effect"]["effects"].as_array_mut().unwrap();
        match which {
            "reverse" => effects.reverse(),
            "duplicate" => effects.push(effects[0].clone()),
            "flatten" => effects[1]["effect"] = effects[1]["effect"]["effects"][0].clone(),
            _ => {
                let effect = bad["effect"].clone();
                bad["effect"] = json!({"kind":"simultaneous","effects":[effect]});
            }
        }
        assert!(
            Program::compile(&serde_json::from_value(bad).unwrap()).is_err(),
            "{which}"
        );
    }
}

#[test]
fn token_three_granted_advantage_changes_combat_and_expires() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (s, han, friendly, _) = staged(&e, seat);
        let prompt = choose(&e, &s, han);
        let mut played = choose(&e, &prompt, friendly);
        let created = tokens(&e, &played, friendly);
        assert_eq!(created.len(), 3);
        // Synthetic turn handoff isolates the recipient's next legal attack.
        played.active = seat;
        played.action_player = Some(seat);
        let base = played.player(seat.other()).base;
        let damage = played.card(base).damage;
        let power = e.power(&played, friendly, true);
        let attack = choose(&e, &played, friendly);
        let pending = choose(&e, &attack, base);
        assert_eq!(pending.card(base).damage, damage + power);
        assert_eq!(pending.decision, Decision::DeathBatch);
        let done = button(&e, &pending, "Resolve all (3)");
        assert!(tokens(&e, &done, friendly).is_empty());
        for id in created {
            assert_eq!(done.card(id).parent, 0);
        }
    }
}
