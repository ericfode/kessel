// Expected outcomes from the pinned addDecreaseCostAbility helpers, card sources,
// and SimpleAdjustedCost. No reference-engine execution is claimed.
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
        def: e.by_code[code],
        owner,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str, damage: u16) -> CardId {
    let id = add(e, s, owner, code);
    s.card_mut(id).damage = damage;
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, owner: Seat) -> State {
    let mut s = State::default();
    s.active = owner;
    s.action_player = Some(owner);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for seat in [Seat::P1, Seat::P2] {
        s.player_mut(seat).base = add(e, &mut s, seat, "ASH_019");
        s.player_mut(seat).leader = add(e, &mut s, seat, "SOR_014");
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}

#[test]
fn self_play_cost_complete_units_and_companion_rejections() {
    let e = engine();
    for code in ["LAW_110", "JTL_163"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in [
        "ASH_262", "ASH_263", "HMW_173", "JTL_101", "LAW_129", "SEC_122", "TS26_047", "TS26_060",
    ] {
        assert!(
            !e.by_code.contains_key(code),
            "incomplete family member {code} admitted"
        );
    }
}

#[test]
fn self_play_cost_live_controller_arena_damage_and_total_floor() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        for code in ["LAW_110", "JTL_163"] {
            let mut s = board(&e, owner);
            let source = add(&e, &mut s, owner, code);
            s.player_mut(owner).hand.push(source).unwrap();
            let initial = e.cost(&s, source);
            let ground = unit(&e, &mut s, owner, "LOF_256", 1);
            unit(&e, &mut s, owner, "SHD_189", 1);
            let enemy = unit(&e, &mut s, owner.other(), "LOF_256", 1);
            unit(&e, &mut s, owner.other(), "SHD_189", 1);
            unit(&e, &mut s, owner, "LOF_256", 0);
            // Friendly ground+space for Phoenix; friendly+enemy ground for AT-DP.
            assert_eq!(e.cost(&s, source), initial.saturating_sub(2));
            s.card_mut(ground).damage = 0;
            assert_eq!(e.cost(&s, source), initial.saturating_sub(1));
            // Damaged deployed leaders are units; a damaged undeployed leader,
            // a base, and a card in hand are outside the arena inventory.
            let leader = s.player(owner).leader;
            s.card_mut(leader).damage = 1;
            s.card_mut(s.player(owner).base).damage = 1;
            s.card_mut(source).damage = 1;
            assert_eq!(e.cost(&s, source), initial.saturating_sub(1));
            s.card_mut(leader).deployed = true;
            e.add_arena(&mut s, leader).unwrap();
            assert_eq!(e.cost(&s, source), initial.saturating_sub(2));
            s.player_mut(owner).ground.remove(leader);
            s.card_mut(leader).deployed = false;
            s.player_mut(owner.other()).ground.remove(enemy);
            s.card_mut(enemy).present = false;
            assert_eq!(
                e.cost(&s, source),
                initial.saturating_sub(if code == "LAW_110" { 1 } else { 0 })
            );
            // Reduction applies after aspect penalties and can remove them too.
            for _ in 0..initial + 2 {
                unit(&e, &mut s, owner, "LOF_256", 1);
            }
            assert_eq!(e.cost(&s, source), 0);
            assert_eq!(
                e.def(&s, source).info.cost(),
                if code == "LAW_110" { 8 } else { 4 }
            );
        }
    }
}

#[test]
fn self_play_cost_actual_play_affordability_payment_and_same_printing_identity() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for owner in [Seat::P1, Seat::P2] {
            for code in ["LAW_110", "JTL_163"] {
                let mut s = board(&e, owner);
                let source = add(&e, &mut s, owner, code);
                s.player_mut(owner).hand.push(source).unwrap();
                let other = add(&e, &mut s, owner, code);
                s.player_mut(owner).hand.push(other).unwrap();
                unit(&e, &mut s, owner, "LOF_256", 1);
                let cost = e.cost(&s, source);
                assert_eq!(e.cost(&s, other), cost); // own physical registration, once per copy
                for _ in 0..cost {
                    let id = add(&e, &mut s, owner, "LOF_254");
                    s.player_mut(owner).resources.push(id).unwrap();
                }
                let action = e.card_action(&s, source);
                assert!(e.legal_actions(&s).contains(&action));
                let mut short = s;
                let resource = short.player(owner).resources.as_slice()[0];
                short.card_mut(resource).exhausted = true;
                assert!(!e.legal_actions(&short).contains(&action));
                let done = e.apply(&s, &action).unwrap();
                assert!(e.in_arena(&done, source));
                assert!(done.card(source).exhausted);
                assert!(!done.player(owner).hand.contains(source));
                assert!(done.player(owner).hand.contains(other));
                assert_eq!(e.ready_resources(&done, owner), 0);
                assert_eq!(done.next_id, s.next_id);
                assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
                if code == "JTL_163" {
                    assert!(e.has_keyword(&done, source, "overwhelm"));
                }
            }
        }
    }
}

#[test]
fn self_play_cost_raw_contract_mutations_reject_complete_card() {
    let original = catalogue();
    let entry = original["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["setCode"] == "LAW_110")
        .unwrap();
    for (key, value) in [
        ("sourceZone", json!("hand")),
        ("targetPlayers", json!("self")),
        ("matchSourceIdentity", json!(false)),
        ("matchPrintedType", json!(false)),
        ("stage", json!("defeatCredits_7")),
        ("matchAbilityCosts", json!(true)),
        ("matchCardEffectResourcePayments", json!(true)),
        ("unexpected", json!(true)),
    ] {
        let mut bad = entry.clone();
        bad["abilities"]["card"][0]["effect"][key] = value;
        assert!(crate::self_play_cost::extract(&bad).is_err(), "{key}");
        let mut input = original.clone();
        let slot = input["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["setCode"] == "LAW_110")
            .unwrap();
        *slot = bad;
        assert!(!Engine::from_catalogue_value(&input)
            .unwrap()
            .by_code
            .contains_key("LAW_110"));
    }
    for key in ["condition", "optional", "limit"] {
        let mut bad = entry.clone();
        bad["abilities"]["card"][0][key] = json!(true);
        assert!(crate::self_play_cost::extract(&bad).is_err());
    }
    let mut bad = entry.clone();
    bad["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .push(json!({"trigger":"whenPlayed","costs":[],"effect":{"kind":"noAction"}}));
    assert!(crate::self_play_cost::extract(&bad).is_err());
    for (key, value) in [
        ("baseClass", json!("LeaderUnitCard")),
        ("stateWatchers", json!([{"kind":"unknown"}])),
    ] {
        let mut bad = entry.clone();
        bad[key] = value;
        assert!(crate::self_play_cost::extract(&bad).is_err());
    }
    let mut renamed = entry.clone();
    renamed["setCode"] = json!("DIAGNOSTIC_001");
    renamed["id"] = json!("different");
    renamed["internalName"] = json!("different");
    assert!(crate::self_play_cost::extract(&renamed).unwrap().is_some());
}

#[test]
fn self_play_cost_selected_pool_round_trip_retains_native_reduction() {
    let e = engine();
    let pool = e.compiled_pool().unwrap();
    let source = catalogue();
    let loaded = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    for code in ["LAW_110", "JTL_163"] {
        let mut s = board(&loaded, Seat::P2);
        let id = add(&loaded, &mut s, Seat::P2, code);
        let cost = loaded.cost(&s, id);
        unit(&loaded, &mut s, Seat::P2, "LOF_256", 1);
        assert_eq!(loaded.cost(&s, id), cost - 1);
    }
}
