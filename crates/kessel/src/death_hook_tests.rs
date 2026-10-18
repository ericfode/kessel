// Source-derived actual-card witnesses. No source engine execution.
use super::*;
use crate::ability_ir::{Ability, Catalogue, Program};
fn entry(code: &str) -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn admits(e: Value) -> bool {
    Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[e.clone()]}))
        .unwrap()
        .cards
        .contains_key(e["setCode"].as_str().unwrap())
}
fn midgame(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn attack(e: &Engine, s: State, source: CardId, target: CardId) -> State {
    let snapshot = e.snapshot(&s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    let a = e.card_action(&loaded, source);
    assert!(e.legal_actions(&loaded).contains(&a));
    let prompt = e.apply(&loaded, &a).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let a = e.card_action(&prompt, target);
    assert!(e.legal_actions(&prompt).contains(&a));
    e.apply(&prompt, &a).unwrap()
}
fn batch(e: &Engine, mut s: State) -> State {
    for _ in 0..8 {
        if s.decision != Decision::DeathBatch {
            return s;
        }
        let a = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.arg == Some(json!("all")))
            .unwrap();
        s = e.apply(&s, &a).unwrap();
    }
    panic!("unexpected unbounded trigger batch")
}
#[test]
fn death_hook_complete_admission_and_fail_closed_boundaries() {
    let e = engine();
    for code in ["SOR_036", "SEC_051", "ASH_127", "TWI_033", "TS26_013"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in ["SOR_115", "SHD_137", "HMW_062", "SEC_088", "SOR_149"] {
        assert!(
            !e.by_code.contains_key(code),
            "unsupported provenance/limit {code}"
        );
    }
    let original = entry("SOR_036");
    assert!(admits(original.clone()));
    for key in ["historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"] {
        let mut bad=original.clone();bad[key]=json!([{"kind":"unimplementedSourceContext"}]);assert!(!admits(bad),"{key}");
    }
    for key in ["limit", "collectiveTrigger", "body", "cannotTargetFirst"] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][key] = json!(true);
        assert!(!admits(bad), "{key}");
    }
    for trigger in ["onCardPlayed", "onDamageDealt", "onAttackEnd"] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0]["hooks"][0]["event"] = json!(trigger);
        assert!(!admits(bad));
    }
    for key in ["optional", "requiresConfirmation", "coordinate"] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][key] = json!(true);
        assert!(!admits(bad));
    }
    for kind in ["leader", "upgrade", "token"] {
        let mut bad = original.clone();
        bad["card"]["types"] = json!([kind]);
        assert!(!admits(bad));
    }
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][0]["predicate"]["op"] = json!("||");
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["args"][0]["steps"][0]
        ["optional"] = json!(true);
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][0]["predicate"]["right"]["left"]["argumentBinding"]
        ["id"] = json!("foreign-callback");
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][0]["predicate"]["right"]["left"]["steps"][0]["key"] =
        json!("card");
    assert!(!admits(bad));
    let hook = original["abilities"]["card"][0].clone();
    let direct: Ability = serde_json::from_value(hook.clone()).unwrap();
    assert!(Program::compile(&direct).is_ok());
    let nested: Ability = serde_json::from_value(
        json!({"trigger":"whenPlayed","title":"outer","effect":{"kind":"noAction"},"then":hook}),
    )
    .unwrap();
    assert!(Program::compile(&nested).is_err());
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
}
#[test]
fn death_hook_gideon_actual_combat_live_friendly_target_both_backends_and_owners() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let source = unit(&e, &mut s, seat, "SOR_036");
            let ally = unit(&e, &mut s, seat, "LOF_256");
            let victim = unit(&e, &mut s, seat.other(), "LOF_254");
            let prompt = attack(&e, s, source, victim);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert!(prompt.player(seat.other()).discard.contains(victim));
            let ids: Vec<_> = e
                .legal_actions(&prompt)
                .iter()
                .filter_map(Action::id)
                .collect();
            assert!(ids.contains(&source) && ids.contains(&ally));
            assert!(!ids.contains(&prompt.player(seat).base));
            let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
            assert_eq!(e.upgrades(&done, ally).len(), 1);
            assert_eq!(e.power(&done, ally, false), 2);
            assert_eq!(done.death_depth, 0);
        }
    }
}
#[test]
fn death_hook_twins_actual_friendly_combat_death_heals_base_and_enemy_death_does_not() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat.other());
            let source = unit(&e, &mut s, seat, "ASH_127");
            let victim = unit(&e, &mut s, seat, "LOF_254");
            let attacker = unit(&e, &mut s, seat.other(), "LOF_256");
            let base = s.player(seat).base;
            s.card_mut(base).damage = 4;
            let done = attack(&e, s, attacker, victim);
            assert_eq!(done.card(base).damage, 3);
            assert!(done.player(seat).discard.contains(victim));
            assert!(e.in_arena(&done, source));
            let mut s = midgame(&e, seat);
            unit(&e, &mut s, seat, "ASH_127");
            let source = unit(&e, &mut s, seat, "LOF_256");
            let victim = unit(&e, &mut s, seat.other(), "LOF_254");
            let base = s.player(seat).base;
            s.card_mut(base).damage = 4;
            let done = attack(&e, s, source, victim);
            assert_eq!(done.card(base).damage, 4);
        }
    }
}
#[test]
fn death_hook_magnaguard_actual_token_death_gains_sentinel_and_when_played_alias() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P2);
        let source = unit(&e, &mut s, Seat::P1, "TWI_033");
        let victim = unit(&e, &mut s, Seat::P1, "TOKEN_BATTLE_DROID");
        let attacker = unit(&e, &mut s, Seat::P2, "LOF_256");
        assert!(!e.has_keyword(&s, source, "sentinel"));
        let done = attack(&e, s, attacker, victim);
        assert!(e.has_keyword(&done, source, "sentinel"));
        assert!(!e.in_arena(&done, victim));
        let mut s = midgame(&e, Seat::P1);
        let source = hand(&e, &mut s, "TWI_033");
        let done = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert!(e.in_arena(&done, source));
        assert!(e.has_keyword(&done, source, "sentinel"));
    }
}
#[test]
fn death_hook_bokatan_play_stat_defeat_triggers_experience_after_enemy_departure() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P1);
        let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
        let survivor = unit(&e, &mut s, Seat::P2, "LOF_256");
        let source = hand(&e, &mut s, "SEC_051");
        for _ in 0..6 {
            let id = add(&e, &mut s, Seat::P1, "LOF_254");
            s.players[0].resources.push(id).unwrap();
        }
        let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        assert!(prompt.player(Seat::P2).discard.contains(victim));
        assert!(e.in_arena(&prompt, survivor));
        assert_eq!(e.hp(&prompt, survivor), 1);
        let done = e.apply(&prompt, &e.card_action(&prompt, source)).unwrap();
        assert_eq!(e.upgrades(&done, source).len(), 1);
        assert_eq!(e.power(&done, source, false), 9);
    }
}
#[test]
fn death_hook_sidious_actual_simultaneous_self_and_enemy_defeat_creates_two_and_token_death_zero() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let source = unit(&e, &mut s, seat, "TS26_013");
            let victim = unit(&e, &mut s, seat.other(), "LOF_254");
            s.card_mut(source).damage = e.hp(&s, source) - 1;
            let done = batch(&e, attack(&e, s, source, victim));
            assert!(done.player(seat).discard.contains(source));
            assert!(done.player(seat.other()).discard.contains(victim));
            let droids: Vec<_> = done
                .player(seat)
                .ground
                .as_slice()
                .iter()
                .copied()
                .filter(|id| e.def(&done, *id).code == "TOKEN_BATTLE_DROID")
                .collect();
            assert_eq!(droids.len(), 2);
            assert!(droids.iter().all(|id| done.card(*id).exhausted));
            let mut s = midgame(&e, seat);
            unit(&e, &mut s, seat, "TS26_013");
            let attacker = unit(&e, &mut s, seat, "LOF_256");
            let victim = unit(&e, &mut s, seat.other(), "TOKEN_BATTLE_DROID");
            let done = attack(&e, s, attacker, victim);
            assert_eq!(
                done.player(seat).ground.len,
                2,
                "token death must not create a droid"
            );
        }
    }
}

#[test]
fn death_hook_gideon_simultaneous_source_death_retains_captured_friendly_selection() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "SOR_036");
        let ally = unit(&e, &mut s, Seat::P1, "LOF_256");
        let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
        s.card_mut(source).damage = e.hp(&s, source) - 1;
        let prompt = attack(&e, s, source, victim);
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        assert!(prompt.player(Seat::P1).discard.contains(source));
        assert!(prompt.player(Seat::P2).discard.contains(victim));
        assert_eq!(
            e.legal_actions(&prompt)
                .iter()
                .filter_map(Action::id)
                .collect::<Vec<_>>(),
            vec![ally]
        );
        let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
        assert_eq!(e.upgrades(&done, ally).len(), 1);
    }
}
#[test]
fn death_hook_opposing_captured_sources_share_owner_order_then_advance_action_owner() {
    let e = engine();
    let mut s = midgame(&e, Seat::P2);
    let source = unit(&e, &mut s, Seat::P2, "SOR_036");
    let defender = unit(&e, &mut s, Seat::P1, "SOR_036");
    let ally1 = unit(&e, &mut s, Seat::P1, "LOF_256");
    let ally2 = unit(&e, &mut s, Seat::P2, "LOF_256");
    let prompt = attack(&e, s, source, defender);
    assert_eq!(prompt.decision, Decision::DeathPlayer);
    assert_eq!(prompt.active, Seat::P2);
    assert_eq!(
        e.legal_actions(&prompt),
        vec![Action::button("You", 0), Action::button("Opponent", 1)]
    );
    let first = named_button(&e, &prompt, "Opponent");
    assert_eq!(first.active, Seat::P1);
    assert_eq!(first.decision, Decision::AbilityTarget);
    let second = e.apply(&first, &e.card_action(&first, ally1)).unwrap();
    assert_eq!(second.active, Seat::P2);
    let done = e.apply(&second, &e.card_action(&second, ally2)).unwrap();
    assert_eq!(e.upgrades(&done, ally1).len(), 1);
    assert_eq!(e.upgrades(&done, ally2).len(), 1);
    assert_eq!(done.death_depth, 0);
    assert_eq!(done.action_player, Some(Seat::P1));
    assert_eq!(done.active, Seat::P1);
}
#[test]
fn death_hook_twins_self_defeat_is_excluded_and_magnaguard_enemy_death_is_excluded() {
    let e = engine();
    let mut s = midgame(&e, Seat::P2);
    let source = unit(&e, &mut s, Seat::P1, "ASH_127");
    let attacker = unit(&e, &mut s, Seat::P2, "LOF_256");
    let base = s.player(Seat::P1).base;
    s.card_mut(base).damage = 4;
    s.card_mut(source).damage = e.hp(&s, source) - 1;
    let done = attack(&e, s, attacker, source);
    assert_eq!(done.card(base).damage, 4);
    assert!(done.player(Seat::P1).discard.contains(source));
    let mut s = midgame(&e, Seat::P1);
    let guard = unit(&e, &mut s, Seat::P1, "TWI_033");
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
    let done = attack(&e, s, attacker, victim);
    assert!(!e.has_keyword(&done, guard, "sentinel"));
}
#[test]
fn death_hook_sidious_live_aura_and_hidden_remain_part_of_complete_definition() {
    let e = engine();
    let mut s = midgame(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "TS26_013");
    let droid = unit(&e, &mut s, Seat::P1, "TOKEN_BATTLE_DROID");
    let enemy = unit(&e, &mut s, Seat::P2, "TOKEN_BATTLE_DROID");
    assert_eq!(e.power(&s, droid, false), 2);
    assert_eq!(e.power(&s, enemy, false), 1);
    assert!(e.has_keyword(&s, source, "hidden"));
    e.defeat(&mut s, source).unwrap();
    assert_eq!(e.power(&s, droid, false), 1);
    assert_eq!(
        s.death_pending.len, 1,
        "self non-token defeat is independently captured"
    );
}

#[test]
fn death_hook_return_is_not_defeat_and_lki_unit_type_includes_deployed_leaders() {
    let e = engine();
    let mut s = midgame(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "SOR_036");
    let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.leave_play(&mut s, victim).unwrap();
    assert_eq!(
        s.death_pending.len, 0,
        "non-defeat departure must not emit defeat hooks"
    );
    let leader = s.players[1].leader;
    e.add_arena(&mut s, leader).unwrap();
    e.capture_death(&mut s, leader).unwrap();
    assert_eq!(s.death_pending.len, 1);
    assert_eq!(s.death_pending.entries[0].source, source);
    assert_eq!(s.death_pending.entries[0].player, Seat::P1);
}
#[test]
fn death_hook_unknown_fields_predicate_paths_and_duplicated_aliases_reject() {
    let original = entry("TWI_033");
    for field in ["stateWatchers", "piloting", "attachCondition"] {
        let mut bad = original.clone();
        bad[field] = json!({"unknown":true});
        assert!(!admits(bad));
    }
    for field in [
        "deployBox",
        "deployCost",
        "upgradePower",
        "upgradeHp",
        "pilotText",
    ] {
        let mut bad = original.clone();
        bad["card"][field] = json!(1);
        assert!(!admits(bad));
    }
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][0]["enabled"] = json!(false);
    assert!(!admits(bad));
    let mut bad = original.clone();
    let alias = bad["abilities"]["card"][0]["hooks"][0].clone();
    bad["abilities"]["card"][0]["hooks"]
        .as_array_mut()
        .unwrap()
        .push(alias);
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["hooks"][1]["predicate"]["unknown"] = json!(true);
    assert!(!admits(bad));
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["effect"]["target"] = json!({"kind":"dataPath","root":["context"],"steps":[{"key":"event","optional":false},{"key":"card","optional":false}]});
    assert!(!admits(bad));
}

#[test]
fn death_hook_twins_existing_play_and_attack_sentinel_siblings_keep_optional_targeting() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P1);
        let ally = unit(&e, &mut s, Seat::P1, "LOF_256");
        let source = hand(&e, &mut s, "ASH_127");
        let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        assert!(e.legal_actions(&prompt).iter().any(|a| a.label == "Pass"));
        assert!(!e
            .legal_actions(&prompt)
            .iter()
            .any(|a| a.id() == Some(source)));
        let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
        assert!(e.has_keyword(&done, ally, "sentinel"));
        let mut s = midgame(&e, Seat::P1);
        let source = unit(&e, &mut s, Seat::P1, "ASH_127");
        let ally = unit(&e, &mut s, Seat::P1, "LOF_256");
        let base = s.players[1].base;
        let prompt = attack(&e, s, source, base);
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
        assert!(e.has_keyword(&done, ally, "sentinel"));
        assert_eq!(done.card(base).damage, 2);
    }
}
#[test]
fn death_hook_sidious_actual_play_keeps_hidden_entry_qualification_and_live_aura() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P1);
        let droid = unit(&e, &mut s, Seat::P1, "TOKEN_BATTLE_DROID");
        let source = hand(&e, &mut s, "TS26_013");
        for _ in 0..6 {
            let id = add(&e, &mut s, Seat::P1, "LOF_254");
            s.players[0].resources.push(id).unwrap();
        }
        let done = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert!(e.in_arena(&done, source));
        assert!(e.has_keyword(&done, source, "hidden"));
        assert!(done.card(source).hidden_qualified());
        assert_eq!(e.power(&done, droid, false), 2);
    }
}

#[test]
fn death_hook_registration_cross_arena_source_order_uses_entry_not_stable_ids() {
    let e = engine();
    for twins_first in [true, false] {
        let mut s = midgame(&e, Seat::P1);
        let twins = add(&e, &mut s, Seat::P1, "ASH_127");
        let shuttle = add(&e, &mut s, Seat::P1, "JTL_063");
        if twins_first {
            e.add_arena(&mut s, twins).unwrap();
            e.add_arena(&mut s, shuttle).unwrap();
        } else {
            e.add_arena(&mut s, shuttle).unwrap();
            e.add_arena(&mut s, twins).unwrap();
        }
        s.card_mut(s.players[0].base).damage = 3;
        deck_card(&e, &mut s, Seat::P1);
        let snapshot = e.snapshot(&s);
        let mut s = e.load_state(&snapshot).unwrap();
        assert_eq!(e.snapshot(&s), snapshot);
        e.defeat(&mut s, shuttle).unwrap();
        let sources = s.death_pending.entries[..s.death_pending.len as usize]
            .iter()
            .map(|e| e.source)
            .collect::<Vec<_>>();
        assert_eq!(
            sources,
            if twins_first {
                vec![twins, shuttle]
            } else {
                vec![shuttle, twins]
            }
        );
        assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
        assert_eq!(s.decision, Decision::DeathOrder);
        let labels = e
            .legal_actions(&s)
            .into_iter()
            .map(|a| a.label)
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            if twins_first {
                vec![
                    "When another friendly unit is defeated, heal 1 damage from your base",
                    "Draw a card",
                ]
            } else {
                vec![
                    "Draw a card",
                    "When another friendly unit is defeated, heal 1 damage from your base",
                ]
            }
        );
    }
}
#[test]
fn death_hook_registration_reentry_and_upgrade_interleaving_preserve_physical_chronology() {
    let e = engine();
    let mut s = midgame(&e, Seat::P1);
    let host = unit(&e, &mut s, Seat::P1, "LOF_256");
    let first = add(&e, &mut s, Seat::P1, "LOF_051");
    e.attach(&mut s, first, host);
    let twins = unit(&e, &mut s, Seat::P1, "ASH_127");
    let second = add(&e, &mut s, Seat::P1, "LOF_051");
    e.attach(&mut s, second, host);
    let snapshot = e.snapshot(&s);
    assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()), snapshot);
    let mut before_host = snapshot.clone();
    before_host["registrationOrder"]["cards"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert!(e
        .load_state(&before_host)
        .unwrap_err()
        .contains("attachment chronology"));
    let mut reversed_grants = snapshot.clone();
    reversed_grants["registrationOrder"]["cards"]
        .as_array_mut()
        .unwrap()
        .swap(1, 3);
    assert!(e
        .load_state(&reversed_grants)
        .unwrap_err()
        .contains("attachment chronology"));
    let mut missing_grant = snapshot.clone();
    missing_grant["registrationOrder"]["cards"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(e.load_state(&missing_grant).is_err());
    // Synthetic captured entries only: root integrates actual granted-death aliases.
    s.death_pending.len = 3;
    s.death_pending.entries[0] = DeathTrigger {
        source: host,
        grantor: second,
        ..DeathTrigger::default()
    };
    s.death_pending.entries[1] = DeathTrigger {
        source: twins,
        ..DeathTrigger::default()
    };
    s.death_pending.entries[2] = DeathTrigger {
        source: host,
        grantor: first,
        ..DeathTrigger::default()
    };
    e.sort_captured_registrations(&mut s, 0).unwrap();
    assert_eq!(
        s.death_pending.entries[..3]
            .iter()
            .map(|e| if e.grantor == 0 { e.source } else { e.grantor })
            .collect::<Vec<_>>(),
        vec![first, twins, second]
    );
    e.detach(&mut s, first);
    e.attach(&mut s, first, host);
    assert!(s.registration_order.position(first) > s.registration_order.position(second));
    e.leave_play(&mut s, twins).unwrap();
    assert!(s.registration_order.position(twins).is_none());
    s.players[0].discard.remove(twins);
    e.add_arena(&mut s, twins).unwrap();
    assert!(s.registration_order.position(twins) > s.registration_order.position(first));
}
#[test]
fn death_hook_registration_import_is_versioned_complete_and_hashes_cross_arena_chronology() {
    let e = engine();
    let mut s = midgame(&e, Seat::P1);
    let ground = unit(&e, &mut s, Seat::P1, "ASH_127");
    let space = unit(&e, &mut s, Seat::P1, "JTL_063");
    let snapshot = e.snapshot(&s);
    let copy = e.load_state(&snapshot).unwrap();
    assert_eq!(copy.zobrist(), s.zobrist());
    assert_eq!((*s.registration_order.as_slice()).len(), 2);
    let mut swapped = snapshot.clone();
    swapped["registrationOrder"]["cards"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let swapped = e.load_state(&swapped).unwrap();
    assert_eq!(swapped.arena_unit_order, s.arena_unit_order);
    assert_ne!(swapped.zobrist(), s.zobrist());
    assert_eq!(
        swapped.registration_order.as_slice(),
        &[space as u8, ground as u8]
    );
    let mut variants = vec![];
    let mut v = snapshot.clone();
    v.as_object_mut().unwrap().remove("registrationOrder");
    variants.push(v);
    let mut v = snapshot.clone();
    v["registrationOrder"]["version"] = json!(2);
    variants.push(v);
    let mut v = snapshot.clone();
    v["registrationOrder"]["unknown"] = json!(true);
    variants.push(v);
    let mut v = snapshot.clone();
    v["registrationOrder"]["cards"] = json!([]);
    variants.push(v);
    let mut v = snapshot.clone();
    let first = v["registrationOrder"]["cards"][0].clone();
    v["registrationOrder"]["cards"]
        .as_array_mut()
        .unwrap()
        .push(first);
    variants.push(v);
    let mut v = snapshot.clone();
    v["registrationOrder"]["cards"][0] = json!("c0004");
    variants.push(v);
    for v in variants {
        assert!(
            e.load_state(&v).is_err(),
            "invalid registration evidence accepted {v}"
        );
    }
}
#[test]
fn death_hook_registration_legacy_compatibility_uses_selected_pool_and_future_card_guard() {
    let mut e = engine();
    let mut s = midgame(&e, Seat::P1);
    unit(&e, &mut s, Seat::P1, "LOF_254");
    let mut snapshot = without_empty_credit_state(e.snapshot(&s));
    snapshot
        .as_object_mut()
        .unwrap()
        .remove("registrationOrder");
    let codes = ["ASH_019", "SOR_014", "IBH_053", "LOF_254"];
    e.catalogue
        .as_mut()
        .unwrap()
        .metadata
        .retain(|code, _| codes.contains(&code.as_str()));
    assert!(!e.needs_registration_order());
    let legacy = e.load_state(&snapshot).unwrap();
    assert!(!legacy.registration_order.exact);
    assert!(legacy.registration_order.as_slice().is_empty());
    assert!(e.snapshot(&legacy).get("registrationOrder").is_none());
    let future = entry("ASH_127")["card"].clone();
    e.catalogue
        .as_mut()
        .unwrap()
        .metadata
        .insert("ASH_127".into(), future);
    assert!(e.needs_registration_order());
    assert!(e
        .load_state(&snapshot)
        .unwrap_err()
        .contains("registrationOrder"));
}
#[test]
fn death_hook_registration_capacity_and_copy_hash_are_bounded() {
    assert_eq!(std::mem::size_of::<RegistrationOrder>(), 260);
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
    let mut order = RegistrationOrder::default();
    for id in 1..MAX_CARDS as u16 {
        order.push(id).unwrap();
    }
    assert_eq!(order.len, 255);
    assert!(order.push(0).is_err());
    assert!(order.push(256).is_err());
    assert!(order.push(7).is_err());
    order.remove(4);
    order.push(4).unwrap();
    assert_eq!(order.as_slice().last(), Some(&4));
    let mut s = State::default();
    s.registration_order = order;
    let copied = s;
    assert_eq!(s.zobrist(), copied.zobrist());
    s.registration_order.exact = false;
    assert_ne!(s.zobrist(), copied.zobrist());
    eprintln!(
        "registration State={} compact={} delta=256",
        std::mem::size_of::<State>(),
        std::mem::size_of::<RegistrationOrder>()
    );
}

#[test]
fn death_hook_actual_twins_and_droid_cohort_share_physical_registration_order() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {for grant_first in [true,false] {
            let mut s=midgame(&e,seat);
            // Allocate the observer first, but register it on either side of the
            // attachment. Stable card IDs cannot establish the listener order.
            let observer=add(&e,&mut s,seat,"ASH_127");
            let base=s.players[seat.index()].base; s.card_mut(base).damage=1;
            let host=unit(&e,&mut s,seat,"LOF_254");
            let grantor=add(&e,&mut s,seat.other(),"TWI_218");
            if grant_first {e.attach(&mut s,grantor,host);e.add_arena(&mut s,observer).unwrap();}
            else {e.add_arena(&mut s,observer).unwrap();e.attach(&mut s,grantor,host);}
            let snapshot=e.snapshot(&s);s=e.load_state(&snapshot).unwrap();
            e.defeat(&mut s,host).unwrap();
            let captured=s.death_pending.entries[..s.death_pending.len as usize].iter().map(|t|(t.source,t.grantor)).collect::<Vec<_>>();
            let expected=if grant_first {vec![(host,grantor),(observer,0)]}else{vec![(observer,0),(host,grantor)]};
            assert_eq!(captured,expected,"{backend:?}/{seat:?}/{grant_first}");
            assert!(e.open_deaths(&mut s,DeathContinuation::Finish).unwrap());
            for _ in 0..12 {
                if !e.is_ability_decision(s.decision) {break;}
                let actions=e.legal_actions(&s);assert!(!actions.is_empty());s=e.apply(&s,&actions[0]).unwrap();
            }
            assert!(!e.is_ability_decision(s.decision));
            assert_eq!(s.card(base).damage,0);
            let tokens=e.arena_units(&s).into_iter().filter(|&id|e.def(&s,id).code=="TOKEN_BATTLE_DROID").collect::<Vec<_>>();
            assert_eq!(tokens.len(),1);assert_eq!(s.card(tokens[0]).owner,seat);
        }}
    }
}

#[test]
fn card_played_hook_admission_maz_yularen_leia_force_sentinel_adi_poggle_and_selectors() {
    let e = engine();
    for code in [
        "SHD_096", "SOR_109", "HMW_115", "LOF_096", "LOF_142", "TWI_080", "SHD_255", "SOR_143",
        "SOR_182", "LOF_087", "LOF_229", "LOF_249", "SHD_005", "SHD_008", "SHD_018",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn card_played_maz_gains_experience_on_other_friendly_unit_play() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let maz = unit(&e, &mut s, seat, "SHD_096");
            assert_eq!(e.upgrades(&s, maz).len(), 0);
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            assert!(e.in_arena(&done, played));
            assert_eq!(e.upgrades(&done, maz).len(), 1);
            assert_eq!(e.power(&done, maz, false), e.def(&done, maz).info.power.unwrap_or(0) as u16 + 1);
            // Own play does not grant Experience.
            let mut s = midgame(&e, seat);
            let maz = hand_for(&e, &mut s, seat, "SHD_096");
            let done = e.apply(&s, &e.card_action(&s, maz)).unwrap();
            assert!(e.in_arena(&done, maz));
            assert_eq!(e.upgrades(&done, maz).len(), 0);
            // Enemy play does not grant Experience.
            let mut s = midgame(&e, seat);
            let maz = unit(&e, &mut s, seat, "SHD_096");
            let enemy = hand_for(&e, &mut s, seat.other(), "ASH_034");
            s.active = seat.other();
            s.action_player = Some(seat.other());
            let done = e.apply(&s, &e.card_action(&s, enemy)).unwrap();
            assert_eq!(e.upgrades(&done, maz).len(), 0);
        }
    }
}

#[test]
fn card_played_yularen_heals_on_command_unit_including_self() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e, Seat::P1);
        let yularen = unit(&e, &mut s, Seat::P1, "SOR_109");
        let base = s.players[0].base;
        s.card_mut(base).damage = 3;
        let command = hand(&e, &mut s, "SHD_096"); // command/heroism unit
        let done = e.apply(&s, &e.card_action(&s, command)).unwrap();
        assert_eq!(done.card(base).damage, 2);
        assert!(e.in_arena(&done, yularen));
        // Self-play also heals.
        let mut s = midgame(&e, Seat::P1);
        let base = s.players[0].base;
        s.card_mut(base).damage = 2;
        let yularen = hand(&e, &mut s, "SOR_109");
        let done = e.apply(&s, &e.card_action(&s, yularen)).unwrap();
        assert_eq!(done.card(base).damage, 1);
    }
}

fn drive_ability_prompts(e: &Engine, mut s: State, limit: usize) -> State {
    for _ in 0..limit {
        if !e.is_ability_decision(s.decision)
            && !matches!(
                s.decision,
                Decision::DeathPlayer | Decision::DeathOrder | Decision::DeathBatch | Decision::AbilityOptional
            )
        {
            return s;
        }
        let actions = e.legal_actions(&s);
        assert!(!actions.is_empty(), "{:?}", s.decision);
        // Prefer resolving the effect: trigger confirmations, then card targets,
        // never the leading Pass/passAbility decline when a real target exists.
        let preferred = actions
            .iter()
            .find(|a| {
                matches!(
                    a.arg.as_ref().and_then(Value::as_str),
                    Some("trigger" | "all")
                ) || a.label == "Trigger"
                    || a.label.starts_with("You")
            })
            .or_else(|| actions.iter().find(|a| a.id().is_some()))
            .cloned()
            .unwrap_or_else(|| actions[0].clone());
        s = e.apply(&s, &preferred).unwrap();
    }
    s
}

#[test]
fn card_played_adi_damages_enemy_base_when_opponent_plays_event() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let adi = unit(&e, &mut s, seat, "LOF_142");
            let enemy_base = s.players[seat.other().index()].base;
            assert_eq!(s.card(enemy_base).damage, 0);
            // Heal is effective so the event does not take AbilityNoEffect.
            s.card_mut(enemy_base).damage = 2;
            let event = hand_for(&e, &mut s, seat.other(), "SHD_252");
            s.active = seat.other();
            s.action_player = Some(seat.other());
            let mut done = e.apply(&s, &e.card_action(&s, event)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(e.in_arena(&done, adi));
            assert!(done.players[seat.other().index()].discard.contains(event));
            // Heal 3 from 2 → 0, then Adi deals 1 to that base.
            assert_eq!(done.card(enemy_base).damage, 1, "{backend:?}/{seat:?}");
            // Friendly event play does not trigger Adi.
            let mut s = midgame(&e, seat);
            let _adi = unit(&e, &mut s, seat, "LOF_142");
            let foe_base = s.players[seat.other().index()].base;
            let own_base = s.players[seat.index()].base;
            s.card_mut(own_base).damage = 2;
            let event = hand_for(&e, &mut s, seat, "SHD_252");
            let mut done = e.apply(&s, &e.card_action(&s, event)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert_eq!(done.card(foe_base).damage, 0, "friendly event {backend:?}/{seat:?}");
            assert_eq!(done.card(own_base).damage, 0, "heal only {backend:?}/{seat:?}");
        }
    }
}

#[test]
fn card_played_lady_proxima_may_damage_base_on_other_underworld_play() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let proxima = unit(&e, &mut s, seat, "SHD_255");
            let own_base = s.players[seat.index()].base;
            let foe_base = s.players[seat.other().index()].base;
            // Maz is Underworld; playing her should offer Proxima's optional base damage.
            let played = hand_for(&e, &mut s, seat, "SHD_096");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(e.in_arena(&done, proxima));
            assert!(e.in_arena(&done, played));
            let damaged = done.card(own_base).damage + done.card(foe_base).damage;
            assert_eq!(damaged, 1, "{backend:?}/{seat:?}");
            // Self-play does not trigger (other filter).
            let mut s = midgame(&e, seat);
            let proxima = hand_for(&e, &mut s, seat, "SHD_255");
            let mut done = e.apply(&s, &e.card_action(&s, proxima)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert_eq!(done.card(s.players[seat.index()].base).damage, 0);
            assert_eq!(done.card(s.players[seat.other().index()].base).damage, 0);
            // Non-Underworld Aggression unit does not trigger.
            let mut s = midgame(&e, seat);
            let _proxima = unit(&e, &mut s, seat, "SHD_255");
            let played = hand_for(&e, &mut s, seat, "ASH_034"); // Wicket: aggression, not underworld
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert_eq!(done.card(s.players[seat.index()].base).damage, 0);
            assert_eq!(done.card(s.players[seat.other().index()].base).damage, 0);
        }
    }
}

#[test]
fn card_played_fighters_may_damage_base_on_other_aggression_including_upgrade() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            // Unit Aggression play.
            let mut s = midgame(&e, seat);
            let fighters = unit(&e, &mut s, seat, "SOR_143");
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(e.in_arena(&done, fighters));
            let own_base = done.players[seat.index()].base;
            let foe_base = done.players[seat.other().index()].base;
            assert_eq!(
                done.card(own_base).damage + done.card(foe_base).damage,
                1,
                "unit play {backend:?}/{seat:?}"
            );
            // Aggression upgrade play also captures (non-unit play path).
            let mut s = midgame(&e, seat);
            let fighters = unit(&e, &mut s, seat, "SOR_143");
            let host = unit(&e, &mut s, seat, "ASH_034");
            let upgrade = hand_for(&e, &mut s, seat, "SOR_166"); // Infiltrator's Skill (aggression)
            let mut done = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            assert_eq!(done.decision, Decision::UpgradeTarget);
            let attach = e
                .legal_actions(&done)
                .into_iter()
                .find(|a| a.id() == Some(host))
                .expect("attach host");
            done = e.apply(&done, &attach).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(e.in_arena(&done, fighters));
            let own_base = done.players[seat.index()].base;
            let foe_base = done.players[seat.other().index()].base;
            assert_eq!(
                done.card(own_base).damage + done.card(foe_base).damage,
                1,
                "upgrade play {backend:?}/{seat:?}"
            );
        }
    }
}

#[test]
fn card_played_bossk_may_damage_unit_when_friendly_event_played() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let bossk = unit(&e, &mut s, seat, "SOR_182");
            let victim = unit(&e, &mut s, seat.other(), "ASH_034");
            let event = hand_for(&e, &mut s, seat, "SHD_252");
            // Pre-damage so Heal from Smuggler's Aid does not take AbilityNoEffect alone.
            s.card_mut(s.players[seat.index()].base).damage = 2;
            let mut done = e.apply(&s, &e.card_action(&s, event)).unwrap();
            // Prefer Bossk's damage prompt when both event and watcher abilities fire.
            for _ in 0..24 {
                if !e.is_ability_decision(done.decision)
                    && !matches!(
                        done.decision,
                        Decision::DeathPlayer
                            | Decision::DeathOrder
                            | Decision::DeathBatch
                            | Decision::AbilityOptional
                    )
                {
                    break;
                }
                let actions = e.legal_actions(&done);
                assert!(!actions.is_empty(), "{:?}", done.decision);
                let preferred = actions
                    .iter()
                    .find(|a| a.id() == Some(victim))
                    .or_else(|| {
                        actions.iter().find(|a| {
                            matches!(
                                a.arg.as_ref().and_then(Value::as_str),
                                Some("trigger" | "all")
                            ) || a.label == "Trigger"
                                || a.label.starts_with("Deal")
                                || a.label.starts_with("You")
                        })
                    })
                    .cloned()
                    .unwrap_or_else(|| actions[0].clone());
                done = e.apply(&done, &preferred).unwrap();
            }
            assert!(e.in_arena(&done, bossk));
            assert_eq!(done.card(victim).damage, 2, "{backend:?}/{seat:?}");
            // Enemy event does not trigger Bossk.
            let mut s = midgame(&e, seat);
            let _bossk = unit(&e, &mut s, seat, "SOR_182");
            let victim = unit(&e, &mut s, seat, "ASH_034");
            let event = hand_for(&e, &mut s, seat.other(), "SHD_252");
            s.card_mut(s.players[seat.other().index()].base).damage = 2;
            s.active = seat.other();
            s.action_player = Some(seat.other());
            let mut done = e.apply(&s, &e.card_action(&s, event)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert_eq!(done.card(victim).damage, 0, "enemy event {backend:?}/{seat:?}");
        }
    }
}

fn drive_force_optional(e: &Engine, mut s: State, watcher: CardId, limit: usize) -> State {
    for _ in 0..limit {
        if !e.is_ability_decision(s.decision)
            && !matches!(
                s.decision,
                Decision::DeathPlayer
                    | Decision::DeathOrder
                    | Decision::DeathBatch
                    | Decision::AbilityOptional
                    | Decision::AbilityTarget
            )
        {
            return s;
        }
        let actions = e.legal_actions(&s);
        assert!(!actions.is_empty(), "{:?}", s.decision);
        // Prefer a card target that is not the watcher (Eighth Brother buffs another unit).
        let preferred = actions
            .iter()
            .find(|a| a.id().is_some_and(|id| id != watcher) && s.source == watcher)
            .or_else(|| actions.iter().find(|a| a.id().is_some() && s.source == watcher))
            .or_else(|| {
                actions.iter().find(|a| {
                    s.source == watcher
                        && matches!(
                            a.arg.as_ref().and_then(Value::as_str),
                            Some("trigger" | "all")
                        )
                })
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a.arg.as_ref().and_then(Value::as_str),
                        Some("trigger" | "all")
                    ) || a.label == "Trigger"
                        || a.label.starts_with("You")
                })
            })
            .or_else(|| actions.iter().find(|a| a.id().is_some()))
            .cloned()
            .unwrap_or_else(|| actions[0].clone());
        s = e.apply(&s, &preferred).unwrap();
    }
    s
}

#[test]
fn card_played_eighth_brother_may_use_force_for_plus_two_plus_two() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            s.player_mut(seat).force_held = true;
            let brother = unit(&e, &mut s, seat, "LOF_087");
            let ally = unit(&e, &mut s, seat, "ASH_068");
            let before = e.power(&s, ally, false);
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_force_optional(&e, done, brother, 24);
            assert!(e.in_arena(&done, brother));
            assert!(!done.player(seat).force_held, "{backend:?}/{seat:?}");
            assert_eq!(
                e.power(&done, ally, false),
                before + 2,
                "{backend:?}/{seat:?}"
            );
            // Without the Force, the optional window does not grant stats.
            let mut s = midgame(&e, seat);
            assert!(!s.player(seat).force_held);
            let brother = unit(&e, &mut s, seat, "LOF_087");
            let ally = unit(&e, &mut s, seat, "ASH_068");
            let before = e.power(&s, ally, false);
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert_eq!(e.power(&done, ally, false), before);
            assert!(e.in_arena(&done, brother));
        }
    }
}

#[test]
fn card_played_luke_may_use_force_for_experience_and_shield_on_unique_play() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            s.player_mut(seat).force_held = true;
            let luke = unit(&e, &mut s, seat, "LOF_249");
            assert_eq!(e.upgrades(&s, luke).len(), 0);
            let played = hand_for(&e, &mut s, seat, "ASH_034"); // unique
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_force_optional(&e, done, luke, 24);
            assert!(e.in_arena(&done, luke));
            assert!(!done.player(seat).force_held, "{backend:?}/{seat:?}");
            assert_eq!(e.upgrades(&done, luke).len(), 2, "{backend:?}/{seat:?}");
            // Non-unique friendly unit does not trigger.
            let mut s = midgame(&e, seat);
            s.player_mut(seat).force_held = true;
            let luke = unit(&e, &mut s, seat, "LOF_249");
            let played = hand_for(&e, &mut s, seat, "ASH_068"); // not unique
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(done.player(seat).force_held, "force unused {backend:?}/{seat:?}");
            assert_eq!(e.upgrades(&done, luke).len(), 0);
        }
    }
}

#[test]
fn card_played_kylo_may_use_force_to_draw_when_upgrade_attaches() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            s.player_mut(seat).force_held = true;
            let kylo = unit(&e, &mut s, seat, "LOF_229");
            let deck_card = add(&e, &mut s, seat, "ASH_068");
            s.player_mut(seat).deck.push(deck_card).unwrap();
            let hand_before = s.player(seat).hand.len;
            let upgrade = hand_for(&e, &mut s, seat, "SOR_166");
            let mut done = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            assert_eq!(done.decision, Decision::UpgradeTarget);
            let attach = e
                .legal_actions(&done)
                .into_iter()
                .find(|a| a.id() == Some(kylo))
                .expect("attach kylo");
            done = e.apply(&done, &attach).unwrap();
            done = drive_force_optional(&e, done, kylo, 24);
            assert!(e.in_arena(&done, kylo));
            assert!(!done.player(seat).force_held, "{backend:?}/{seat:?}");
            assert_eq!(
                done.player(seat).hand.len,
                hand_before + 1,
                "{backend:?}/{seat:?}"
            );
            // Attaching the same upgrade to another friendly unit does not fire Kylo.
            let mut s = midgame(&e, seat);
            s.player_mut(seat).force_held = true;
            let kylo = unit(&e, &mut s, seat, "LOF_229");
            let other = unit(&e, &mut s, seat, "ASH_068");
            let deck_card = add(&e, &mut s, seat, "ASH_068");
            s.player_mut(seat).deck.push(deck_card).unwrap();
            let hand_before = s.player(seat).hand.len;
            let upgrade = hand_for(&e, &mut s, seat, "SOR_166");
            let mut done = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            let attach = e
                .legal_actions(&done)
                .into_iter()
                .find(|a| a.id() == Some(other))
                .expect("attach other");
            done = e.apply(&done, &attach).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(done.player(seat).force_held, "force unused {backend:?}/{seat:?}");
            assert_eq!(done.player(seat).hand.len, hand_before);
            assert!(e.in_arena(&done, kylo));
        }
    }
}

#[test]
fn card_played_poggle_may_exhaust_to_create_battle_droid() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            let poggle = unit(&e, &mut s, seat, "TWI_080");
            assert!(!s.card(poggle).exhausted);
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(e.in_arena(&done, played));
            assert!(done.card(poggle).exhausted, "{backend:?}/{seat:?}");
            let tokens = e
                .arena_units(&done)
                .into_iter()
                .filter(|&id| e.def(&done, id).code == "TOKEN_BATTLE_DROID")
                .collect::<Vec<_>>();
            assert_eq!(tokens.len(), 1, "{backend:?}/{seat:?}");
            assert_eq!(done.card(tokens[0]).owner, seat);
            // Passing declines the Battle Droid.
            let mut s = midgame(&e, seat);
            let poggle = unit(&e, &mut s, seat, "TWI_080");
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            for _ in 0..16 {
                if done.decision == Decision::AbilityOptional && done.source == poggle {
                    let pass = e
                        .legal_actions(&done)
                        .into_iter()
                        .find(|a| a.arg.as_ref().and_then(Value::as_str) == Some("pass"))
                        .expect("Pass");
                    done = e.apply(&done, &pass).unwrap();
                    break;
                }
                if !e.is_ability_decision(done.decision)
                    && !matches!(
                        done.decision,
                        Decision::DeathPlayer | Decision::DeathOrder | Decision::DeathBatch
                    )
                {
                    break;
                }
                let actions = e.legal_actions(&done);
                done = e.apply(&done, &actions[0]).unwrap();
            }
            assert!(!done.card(poggle).exhausted);
            assert_eq!(
                e.arena_units(&done)
                    .into_iter()
                    .filter(|&id| e.def(&done, id).code == "TOKEN_BATTLE_DROID")
                    .count(),
                0
            );
        }
    }
}

fn hand_for(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    for _ in 0..8 {
        let resource = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(resource).unwrap();
    }
    id
}

fn set_leader(e: &Engine, s: &mut State, seat: Seat, code: &str) {
    let leader = s.player(seat).leader;
    s.card_mut(leader).def = *e.by_code.get(code).unwrap_or_else(|| {
        panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))
    });
}

fn deploy_leader(e: &Engine, s: &mut State, seat: Seat) {
    let leader = s.player(seat).leader;
    s.card_mut(leader).deployed = true;
    s.card_mut(leader).epic_used = true;
    e.add_arena(s, leader).unwrap();
}

#[test]
fn card_played_mando_front_may_exhaust_to_exhaust_low_hp_enemy_on_upgrade() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_018");
            let leader = s.player(seat).leader;
            assert!(!s.card(leader).deployed);
            assert!(!s.card(leader).exhausted);
            let host = unit(&e, &mut s, seat, "LOF_254");
            let enemy = unit(&e, &mut s, seat.other(), "ASH_034"); // hp 3 ≤ 4
            assert!(!s.card(enemy).exhausted);
            let upgrade = hand_for(&e, &mut s, seat, "SOR_166");
            let mut done = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            assert_eq!(done.decision, Decision::UpgradeTarget);
            let attach = e
                .legal_actions(&done)
                .into_iter()
                .find(|a| a.id() == Some(host))
                .expect("attach host");
            done = e.apply(&done, &attach).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(done.card(leader).exhausted, "{backend:?}/{seat:?}");
            assert!(done.card(enemy).exhausted, "{backend:?}/{seat:?}");
        }
    }
}

#[test]
fn card_played_mando_deployed_may_exhaust_enemy_on_upgrade_without_self_exhaust() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_018");
            deploy_leader(&e, &mut s, seat);
            let leader = s.player(seat).leader;
            assert!(!s.card(leader).exhausted);
            let host = unit(&e, &mut s, seat, "LOF_254");
            let enemy = unit(&e, &mut s, seat.other(), "ASH_034");
            let upgrade = hand_for(&e, &mut s, seat, "SOR_166");
            let mut done = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            let attach = e
                .legal_actions(&done)
                .into_iter()
                .find(|a| a.id() == Some(host))
                .expect("attach host");
            done = e.apply(&done, &attach).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(!done.card(leader).exhausted, "{backend:?}/{seat:?}");
            assert!(done.card(enemy).exhausted, "{backend:?}/{seat:?}");
        }
    }
}

#[test]
fn card_played_boba_front_may_exhaust_to_buff_friendly_on_keyword_unit() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_008");
            let leader = s.player(seat).leader;
            let ally = unit(&e, &mut s, seat, "LOF_254");
            let before = e.power(&s, ally, false);
            let played = hand_for(&e, &mut s, seat, "ASH_034"); // saboteur
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(done.card(leader).exhausted, "{backend:?}/{seat:?}");
            assert_eq!(
                e.power(&done, ally, false),
                before + 1,
                "{backend:?}/{seat:?}"
            );
            // Non-keyword play does not open the window.
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_008");
            let leader = s.player(seat).leader;
            let played = hand_for(&e, &mut s, seat, "LOF_254");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(!done.card(leader).exhausted, "plain unit {backend:?}/{seat:?}");
        }
    }
}

#[test]
fn card_played_boba_deployed_aura_buffs_other_keyword_units() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_008");
            deploy_leader(&e, &mut s, seat);
            let keyworded = unit(&e, &mut s, seat, "ASH_034");
            let plain = unit(&e, &mut s, seat, "LOF_254");
            let enemy = unit(&e, &mut s, seat.other(), "ASH_034");
            assert_eq!(
                e.power(&s, keyworded, false),
                e.def(&s, keyworded).info.power.unwrap_or(0) as u16 + 1,
                "{backend:?}/{seat:?}"
            );
            assert_eq!(
                e.power(&s, plain, false),
                e.def(&s, plain).info.power.unwrap_or(0) as u16,
                "{backend:?}/{seat:?}"
            );
            assert_eq!(
                e.power(&s, enemy, false),
                e.def(&s, enemy).info.power.unwrap_or(0) as u16,
                "{backend:?}/{seat:?}"
            );
        }
    }
}

#[test]
fn card_played_hondo_front_may_exhaust_to_give_experience_on_smuggle() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_005");
            let leader = s.player(seat).leader;
            let ally = unit(&e, &mut s, seat, "LOF_254");
            assert_eq!(e.upgrades(&s, ally).len(), 0);
            for _ in 0..6 {
                let resource = add(&e, &mut s, seat, "LOF_254");
                s.player_mut(seat).resources.push(resource).unwrap();
            }
            let smuggled = add(&e, &mut s, seat, "SHD_160");
            s.player_mut(seat).resources.push(smuggled).unwrap();
            let top = add(&e, &mut s, seat, "LOF_256");
            s.player_mut(seat).deck.push(top).unwrap();
            let mut done = e.apply(&s, &e.card_action(&s, smuggled)).unwrap();
            done = drive_ability_prompts(&e, done, 24);
            assert!(done.card(leader).exhausted, "{backend:?}/{seat:?}");
            assert_eq!(e.upgrades(&done, ally).len(), 1, "{backend:?}/{seat:?}");
            // Ordinary hand play does not fire Hondo.
            let mut s = midgame(&e, seat);
            set_leader(&e, &mut s, seat, "SHD_005");
            let leader = s.player(seat).leader;
            let played = hand_for(&e, &mut s, seat, "ASH_034");
            let mut done = e.apply(&s, &e.card_action(&s, played)).unwrap();
            done = drive_ability_prompts(&e, done, 16);
            assert!(!done.card(leader).exhausted, "hand play {backend:?}/{seat:?}");
        }
    }
}

