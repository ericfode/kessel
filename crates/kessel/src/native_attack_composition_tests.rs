//! Source-derived composition witnesses. InitiateAttackSystem resolves its
//! MetaEvent before its child defender picker; CardAbilityStep's ifYouDo tests
//! that resolved event, rather than whether a later combat actually completed.
use super::*;
use crate::ability_ir::Program;

fn nc_engine(backend: RulesBackend) -> Engine {
    let mut e = engine();
    e.set_rules_backend(backend).unwrap();
    e
}
fn nc_start(e: &Engine, seat: Seat, leader: Option<&str>) -> State {
    let mut s = board(e);
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 6;
    if let Some(code) = leader {
        let id = s.player(seat).leader;
        s.card_mut(id).def = e.by_code[code];
    }
    s
}
fn nc_resources(e: &Engine, s: &mut State, seat: Seat, n: usize) {
    for _ in 0..n {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
fn nc_click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn nc_leader_action(e: &Engine, s: &State) -> State {
    let leader = s.player(s.active).leader;
    let menu = nc_click(e, s, leader);
    if menu.decision == Decision::LeaderMenu {
        let title = e.program(e.card_programs(s, leader)[0]).title.clone();
        named_button(e, &menu, &title)
    } else {
        menu
    }
}
fn nc_event(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    nc_resources(e, s, seat, 12);
    let id = add(e, s, seat, "ASH_184");
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn nc_advantages(e: &Engine, s: &State, host: CardId) -> Vec<CardId> {
    e.upgrades(s, host)
        .into_iter()
        .filter(|id| e.def(s, *id).code == "TOKEN_ADVANTAGE")
        .collect()
}
fn nc_roundtrip(e: &Engine, s: &State) {
    let snapshot = e.snapshot(s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    assert_eq!(s.zobrist(), loaded.zobrist());
}
fn nc_source(code: &str) -> Value {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}

#[test]
fn native_composition_admits_only_complete_reviewed_shapes() {
    let e = engine();
    for code in ["TWI_009", "HMW_006", "ASH_184"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        let source = nc_source(code);
        assert!(crate::native_attack::entry(&source));
        let sides = crate::attached_registration::sides(&source).unwrap();
        for a in sides.card.iter().chain(&sides.leader).chain(&sides.unit) {
            assert!(Program::compile(a).is_ok(), "{code}");
        }
    }
    // These require another attack lifecycle, a captured event predicate,
    // unimplemented attack modifiers, or a broader leader source topology.
    for code in ["HMW_104", "JTL_228", "LOF_221", "TWI_011", "SOR_150"] {
        assert!(
            !e.by_code.contains_key(code),
            "{code} remains outside this composition"
        );
    }
}

#[test]
fn native_composition_leader_actions_pay_physical_source_costs_and_clean_grants() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, keyword, cost) in [("TWI_009", "overwhelm", 0), ("HMW_006", "grit", 1)] {
                let mut s = nc_start(&e, seat, Some(code));
                nc_resources(&e, &mut s, seat, cost);
                let leader = s.player(seat).leader;
                let attacker = unit(&e, &mut s, seat, "LOF_093");
                s.card_mut(attacker).damage = 1;
                let wrong = unit(&e, &mut s, seat, "LOF_256");
                let enemy = unit(&e, &mut s, seat.other(), "LOF_093");
                let ready = e.ready_resources(&s, seat);
                let before = e.power(&s, attacker, true);
                let select = nc_leader_action(&e, &s);
                assert_eq!(select.decision, Decision::AbilityTarget);
                let targets = e.legal_actions(&select);
                assert!(targets.contains(&e.card_action(&select, attacker)));
                assert!(!targets.contains(&e.card_action(&select, leader)));
                assert!(!targets.contains(&e.card_action(&select, enemy)));
                assert_eq!(
                    targets.contains(&e.card_action(&select, wrong)),
                    code == "TWI_009"
                );
                let cancelled = named_button(&e, &select, "Cancel");
                assert_eq!(e.ready_resources(&cancelled, seat), ready);
                assert!(!cancelled.card(leader).exhausted);
                assert_eq!(e.keyword_amount(&cancelled, attacker, keyword), 0);
                let choose = nc_click(&e, &select, attacker);
                assert_eq!(choose.decision, Decision::AttackTarget);
                assert!(choose.card(leader).exhausted);
                assert!(!choose.card(attacker).exhausted);
                assert_eq!(e.ready_resources(&choose, seat), ready - cost);
                assert_eq!(choose.support.context.source, leader);
                assert_eq!(e.keyword_amount(&choose, attacker, keyword), 1);
                assert_eq!(
                    e.power(&choose, attacker, true),
                    before + if code == "HMW_006" { 1 } else { 0 }
                );
                assert!(!e
                    .legal_actions(&choose)
                    .iter()
                    .any(|a| a.label == "Pass attack"));
                let base = choose.player(seat.other()).base;
                let expected = e.power(&choose, attacker, true);
                let done = nc_click(&e, &choose, base);
                assert_eq!(done.card(base).damage, expected);
                assert!(done.card(attacker).exhausted);
                assert!(done.card(leader).exhausted);
                assert_eq!(e.keyword_amount(&done, attacker, keyword), 0);
                assert!(!done.support.pending);
                assert_eq!(done.support.len, 0);
                nc_roundtrip(&e, &done);
            }
        }
    }
}

#[test]
fn native_composition_omega_requires_resource_and_no_target_cancellation_is_atomic() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = nc_start(&e, seat, Some("HMW_006"));
            let leader = s.player(seat).leader;
            unit(&e, &mut s, seat, "LOF_093");
            assert!(!e.legal_actions(&s).contains(&e.card_action(&s, leader)));
            nc_resources(&e, &mut s, seat, 1);
            let friendly = s.player(seat).ground.as_slice()[0];
            s.card_mut(friendly).exhausted = true;
            let no_effect = nc_leader_action(&e, &s);
            assert_eq!(no_effect.decision, Decision::AbilityNoEffect);
            let cancelled = named_button(&e, &no_effect, "Cancel");
            assert!(!cancelled.card(leader).exhausted);
            assert_eq!(e.ready_resources(&cancelled, seat), 1);
            let used = named_button(&e, &no_effect, "Use it anyway");
            assert!(used.card(leader).exhausted);
            assert_eq!(e.ready_resources(&used, seat), 0);
            assert!(!used.support.pending);
        }
    }
}

#[test]
fn native_composition_deployment_switches_action_to_live_other_friendly_aura() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, keyword) in [("TWI_009", "overwhelm"), ("HMW_006", "grit")] {
                let mut s = nc_start(&e, seat, Some(code));
                nc_resources(&e, &mut s, seat, 6);
                let leader = s.player(seat).leader;
                let friend = unit(&e, &mut s, seat, "LOF_093");
                let neutral = unit(&e, &mut s, seat, "LOF_256");
                let enemy = unit(&e, &mut s, seat.other(), "LOF_093");
                assert_eq!(e.keyword_amount(&s, friend, keyword), 0);
                let menu = nc_click(&e, &s, leader);
                assert_eq!(menu.decision, Decision::LeaderMenu);
                let deploy = e
                    .legal_actions(&menu)
                    .into_iter()
                    .find(|a| a.label.starts_with("Deploy "))
                    .unwrap();
                let mut deployed = e.apply(&menu, &deploy).unwrap();
                assert!(deployed.card(leader).deployed);
                assert!(e.available_programs(&deployed, leader, "action").is_empty());
                assert_eq!(e.keyword_amount(&deployed, friend, keyword), 1);
                assert_eq!(
                    e.keyword_amount(&deployed, neutral, keyword),
                    if code == "TWI_009" { 1 } else { 0 }
                );
                assert_eq!(e.keyword_amount(&deployed, enemy, keyword), 0);
                assert_eq!(
                    e.keyword_amount(&deployed, leader, keyword),
                    if code == "TWI_009" { 1 } else { 0 }
                );
                if code == "HMW_006" {
                    let power = e.power(&deployed, friend, false);
                    deployed.card_mut(friend).damage = 2;
                    assert_eq!(e.power(&deployed, friend, false), power + 2);
                }
                nc_roundtrip(&e, &deployed);
                e.defeat(&mut deployed, leader).unwrap();
                assert!(!deployed.card(leader).deployed);
                assert!(deployed.card(leader).exhausted);
                assert_eq!(e.keyword_amount(&deployed, friend, keyword), 0);
                assert_eq!(e.card_programs(&deployed, leader).len(), 1);
                assert!(e.available_programs(&deployed, leader, "action").is_empty());
            }
        }
    }
}

#[test]
fn native_composition_follow_me_waits_for_deaths_then_selects_fresh_recipient() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = nc_start(&e, seat, None);
            let attacker = unit(&e, &mut s, seat, "LOF_093");
            let defender = unit(&e, &mut s, seat.other(), "SOR_145");
            s.card_mut(defender).damage = 3;
            let recipient = unit(&e, &mut s, seat.other(), "LOF_256");
            let event = nc_event(&e, &mut s, seat);
            let ready = e.ready_resources(&s, seat);
            let cost = e.cost(&s, event) as usize;
            let select = nc_click(&e, &s, event);
            let choose = nc_click(&e, &select, attacker);
            assert_eq!(choose.decision, Decision::AttackTarget);
            assert_eq!(e.ready_resources(&choose, seat), ready - cost);
            assert!(choose.player(seat).discard.contains(event));
            let pending = nc_click(&e, &choose, defender);
            assert!(pending.player(seat.other()).discard.contains(defender));
            assert!(
                pending.support.pending,
                "attack death ability precedes the parent continuation"
            );
            assert!(nc_advantages(&e, &pending, recipient).is_empty());
            let fresh = named_button(&e, &pending, "Deal 3 damage to opponent's base");
            assert_eq!(fresh.card(fresh.player(seat).base).damage, 3);
            assert_eq!(fresh.decision, Decision::AbilityTarget);
            assert_eq!(fresh.active, seat);
            assert_eq!(fresh.source, event);
            assert_eq!(fresh.attack_source, 0);
            assert!(!fresh.support.pending);
            let targets = e.legal_actions(&fresh);
            assert!(targets.contains(&e.card_action(&fresh, recipient)));
            assert!(targets.contains(&e.card_action(&fresh, attacker)));
            assert!(!targets.contains(&e.card_action(&fresh, defender)));
            let next = fresh.next_id;
            let power = e.power(&fresh, recipient, false);
            let done = nc_click(&e, &fresh, recipient);
            let tokens = nc_advantages(&e, &done, recipient);
            assert_eq!(tokens.len(), 3);
            assert_eq!(done.next_id, next + 3);
            assert_eq!(tokens, [next, next + 1, next + 2]);
            for token in tokens {
                assert_eq!(done.card(token).owner, seat.other());
                assert_eq!(done.card(token).parent, recipient);
            }
            assert_eq!(e.power(&done, recipient, false), power + 3);
            assert!(nc_advantages(&e, &done, attacker).is_empty());
            e.validate_attachment_links(&done).unwrap();
            nc_roundtrip(&e, &done);
        }
    }
}

#[test]
fn native_composition_follow_me_if_you_do_uses_initiation_event_resolution() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = nc_start(&e, seat, None);
            let attacker = unit(&e, &mut s, seat, "LOF_093");
            let event = nc_event(&e, &mut s, seat);
            let ready = e.ready_resources(&s, seat);
            let next = s.next_id;
            let select = nc_click(&e, &s, event);
            let cancelled = named_button(&e, &select, "Cancel");
            assert!(cancelled.player(seat).hand.contains(event));
            assert_eq!(e.ready_resources(&cancelled, seat), ready);
            assert_eq!(cancelled.next_id, next);
            // The MetaEvent has resolved before this child picker exists. A
            // child attack is mandatory: only its completed attack reaches ifYouDo.
            let choose = nc_click(&e, &select, attacker);
            assert_eq!(e.legal_actions(&choose), vec![e.card_action(&choose, s.player(seat.other()).base)]);
            assert!(e.apply(&choose, &Action::button("Cancel", "cancel")).is_err());
            let declined = nc_click(&e, &choose, s.player(seat.other()).base);
            assert_eq!(declined.decision, Decision::AbilityTarget);
            assert_eq!(declined.source, event);
            assert!(declined.player(seat).discard.contains(event));
            assert!(declined.card(attacker).exhausted);
            assert_eq!(declined.card(s.player(seat.other()).base).damage, 2);
            assert!(!declined.support.pending);
            assert!(nc_advantages(&e, &declined, attacker).is_empty());
            let done = nc_click(&e, &declined, attacker);
            assert_eq!(nc_advantages(&e, &done, attacker).len(), 3);
            // No eligible attacker means no initiating event; an enemy unit is
            // still a legal Advantage recipient but cannot bypass ifYouDo.
            let mut empty = nc_start(&e, seat, None);
            let enemy = unit(&e, &mut empty, seat.other(), "LOF_256");
            let event = nc_event(&e, &mut empty, seat);
            let no_attack = nc_click(&e, &empty, event);
            assert_eq!(no_attack.decision, Decision::AbilityNoEffect);
            let done = named_button(&e, &no_attack, "Play anyway");
            assert!(done.player(seat).discard.contains(event));
            assert!(nc_advantages(&e, &done, enemy).is_empty());
            assert!(!done.support.pending);
            assert_eq!(done.decision, Decision::Window);
        }
    }
}

#[test]
fn native_composition_whole_metadata_headers_and_other_continuations_stay_closed() {
    for code in ["TWI_009", "HMW_006", "ASH_184"] {
        let original = nc_source(code);
        assert!(crate::native_attack::entry(&original));
        for (field, value) in [
            ("baseClass", json!("UpgradeCard")),
            ("piloting", json!({})),
            ("privatePrograms", json!([{}])),
            ("stateWatchers", json!([{}])),
            ("inheritedSource", json!({})),
            ("setupValidationOverrides", json!([{}])),
        ] {
            let mut bad = original.clone();
            bad[field] = value;
            assert!(!crate::native_attack::entry(&bad), "{code}:{field}");
        }
        for (field, value) in [
            ("types", json!(["unit", "leader"])),
            ("unique", Value::Null),
            ("pilotText", json!("Piloting [1]")),
            ("upgradePower", json!(0)),
            ("upgradeHp", json!(0)),
            ("deployCost", json!(1)),
            ("epicAction", json!("Draw 1 card")),
        ] {
            let mut bad = original.clone();
            bad["card"][field] = value;
            assert!(!crate::native_attack::entry(&bad), "{code}:{field}");
        }
        let side = if code == "ASH_184" { "card" } else { "leader" };
        for (field, value) in [
            ("optional", json!(true)),
            ("coordinate", json!(true)),
            ("requiresConfirmation", json!(true)),
            ("condition", json!({"kind":"const","value":true})),
            (
                "costs",
                json!([{"kind":"exhaustSelf"},{"kind":"exhaustSelf"}]),
            ),
        ] {
            let mut bad = original.clone();
            bad["abilities"][side][0][field] = value;
            assert!(!crate::native_attack::entry(&bad), "{code}:{field}");
        }
        let mut bad = original.clone();
        bad["abilities"][side]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"][side][0].clone());
        assert!(!crate::native_attack::entry(&bad));
    }
    let source = nc_source("ASH_184");
    for field in ["then", "ifYouDo"] {
        let mut bad = source.clone();
        let attack = bad["abilities"]["card"][0]["effect"].clone();
        bad["abilities"]["card"][0][field] = json!({"title":"nested attack","effect":attack});
        assert!(!crate::native_attack::entry(&bad));
    }
    for (field, value) in [
        ("optional", json!(true)),
        ("costs", json!([{"kind":"exhaustSelf"}])),
        ("ifYouDoCondition", json!({"kind":"const","value":true})),
    ] {
        let mut bad = source.clone();
        bad["abilities"]["card"][0]["ifYouDo"][field] = value;
        assert!(!crate::native_attack::entry(&bad));
    }
    for code in ["TWI_009", "HMW_006"] {
        let mut bad = nc_source(code);
        bad["abilities"]["unit"][0]["targetResolver"]["owner"] = json!("opponent");
        assert!(!crate::native_attack::entry(&bad));
        let mut bad = nc_source(code);
        bad["abilities"]["unit"][0]["effect"]["amount"]["value"] = json!(2);
        assert!(!crate::native_attack::entry(&bad));
        let mut bad = nc_source(code);
        bad["card"]["deployBox"] = json!("When Deployed: Draw 2 cards");
        assert!(!crate::native_attack::entry(&bad));
    }
}

#[test]
fn native_composition_follow_me_requires_reviewed_advantage_in_selected_pool() {
    let e = engine();
    let catalogue = include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"));
    let pool = e.compiled_pool().unwrap();
    let mut complete = pool.clone();
    complete["cards"].as_array_mut().unwrap().retain(|c| {
        [
            "ASH_019",
            "SOR_014",
            "IBH_053",
            "TOKEN_SHIELD",
            "TOKEN_ADVANTAGE",
            "ASH_184",
        ]
        .contains(&c["setCode"].as_str().unwrap())
    });
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&complete).unwrap(), catalogue)
            .is_ok()
    );
    let mut absent = complete.clone();
    absent["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_ADVANTAGE");
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&absent).unwrap(), catalogue)
            .err()
            .unwrap()
            .contains("TOKEN_ADVANTAGE")
    );
    let mut malformed = complete;
    malformed["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TOKEN_ADVANTAGE")
        .unwrap()["traits"] = json!([]);
    assert!(
        Engine::from_compiled_pool_bytes(&serde_json::to_vec(&malformed).unwrap(), catalogue)
            .is_err()
    );
    let mut changed: Value = serde_json::from_slice(catalogue).unwrap();
    changed["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TOKEN_ADVANTAGE")
        .unwrap()["sourceHash"] = json!("unreviewed");
    let altered = Engine::from_catalogue_value(&changed).unwrap();
    assert!(!altered.by_code.contains_key("ASH_184"));
}

#[test]
fn native_composition_maul_grant_keeps_shield_overwhelm_and_cleanup_boundaries() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = nc_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for shield in [false, true] {
                let mut s = nc_start(&e, seat, Some("TWI_009"));
                let attacker = unit(&e, &mut s, seat, "LOF_093");
                let defender = unit(&e, &mut s, seat.other(), "LOF_254");
                if shield {
                    e.create_token(&mut s, defender, false).unwrap();
                }
                let select = nc_leader_action(&e, &s);
                let choose = nc_click(&e, &select, attacker);
                let base = choose.player(seat.other()).base;
                let done = nc_click(&e, &choose, defender);
                assert_eq!(done.card(base).damage, if shield { 0 } else { 1 });
                assert_eq!(e.in_arena(&done, defender), shield);
                if shield {
                    assert_eq!(done.card(defender).shield, 0);
                    assert_eq!(done.card(defender).damage, 0);
                }
                assert_eq!(e.keyword_amount(&done, attacker, "overwhelm"), 0);
                assert_eq!(done.support.len, 0);
                assert!(!done.support.pending);
            }
        }
    }
}
