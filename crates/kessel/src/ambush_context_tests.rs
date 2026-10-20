use super::*;
use crate::ability_ir::ambush_context_contract as contract;
fn entry() -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let mut e = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_207")
        .unwrap()
        .clone();
    e["abilities"]["card"][0]["condition"] = json!({"kind":"query","name":"isAttackingUsingAmbush","receiver":{"kind":"ref","path":["source"]},"args":[]});
    e
}
fn engine() -> Engine {
    let mut c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let cards = c["cards"].as_array_mut().unwrap();
    let i = cards
        .iter()
        .position(|e| e["setCode"] == "ASH_207")
        .unwrap();
    cards[i] = entry();
    Engine::from_catalogue_bytes(&serde_json::to_vec(&c).unwrap()).unwrap()
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
fn board(e: &Engine, seat: Seat) -> (State, CardId, CardId) {
    let mut s = State::default();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for p in [Seat::P1, Seat::P2] {
        let base = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).base = base;
    }
    let a = add(e, &mut s, seat, "ASH_207");
    let d = add(e, &mut s, seat.other(), "JTL_069");
    e.add_arena(&mut s, a).unwrap();
    e.add_arena(&mut s, d).unwrap();
    (s, a, d)
}
#[test]
fn ambush_context_complete_metadata_and_raw_scope_are_closed() {
    let good = entry();
    assert!(contract::entry(&good));
    let admitted = |v: Value| {
        crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]}))
            .unwrap()
            .cards
            .contains_key("ASH_207")
    };
    assert!(admitted(good.clone()));
    for key in ["unknownSemantic", "historyHelpers", "deploymentOverride"] {
        let mut v = good.clone();
        v[key] = json!({"x":1});
        assert!(!admitted(v), "{key}");
    }
    for key in ["pilotText", "unknownSemantic"] {
        let mut v = good.clone();
        v["card"][key] = json!("new mechanic");
        assert!(!admitted(v), "{key}");
    }
    for (path, value) in [
        ("/card/types", json!(["unit", "leader"])),
        ("/baseClass", json!("EventCard")),
        ("/card/power", json!(4)),
        ("/card/text", json!("Ambush")),
        ("/keywordValues", json!({"ambush":1,"support":1})),
        ("/abilities/card/0/optional", json!(true)),
        ("/abilities/card/0/effect/hp/value", json!(1)),
        (
            "/abilities/card/0/condition/receiver/path",
            json!(["target"]),
        ),
        (
            "/abilities/card/0/condition/args",
            json!([{"kind":"const","value":1}]),
        ),
    ] {
        let mut e = good.clone();
        let keys: Vec<_> = path.split('/').skip(1).collect();
        let mut at = &mut e;
        for key in &keys[..keys.len() - 1] {
            at = if at.is_array() {
                &mut at[key.parse::<usize>().unwrap()]
            } else {
                &mut at[*key]
            };
        }
        at[keys[keys.len() - 1]] = value;
        assert!(!admitted(e), "{path}");
    }
    for key in ["condition", "contextTitle", "then", "ifYouDo"] {
        let mut a = good["abilities"]["card"][0].clone();
        let q = a["condition"].clone();
        a["effect"] = json!({"kind":"noAction"});
        a[key] = q;
        let rejected = serde_json::from_value::<crate::ability_ir::Ability>(a)
            .map(|a| crate::ability_ir::Program::compile(&a).is_err())
            .unwrap_or(true);
        assert!(rejected, "{key}");
    }
}
#[test]
fn ambush_context_uses_physical_source_and_matching_suspended_attack() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (mut s, a, d) = board(&e, seat);
        assert_eq!(e.evaluated_power(&s, a), 3);
        s.attack_source = a;
        s.attack_defender = d;
        s.ambush = true;
        assert!(e.is_attacking_using_ambush(&s, a));
        assert!(!e.is_attacking_using_ambush(&s, d));
        assert_eq!(e.evaluated_power(&s, a), 5);
        s.ambush = false;
        assert_eq!(e.evaluated_power(&s, a), 3);
        s.ambush = true;
        let context = crate::model::ResolutionContext::capture(&s);
        s.death_depth = 1;
        s.death_frames[0].context = context;
        s.attack_source = d;
        s.attack_defender = 0;
        s.ambush = false;
        assert!(e.is_attacking_using_ambush(&s, a));
        assert_eq!(e.evaluated_power(&s, a), 5);
        s.death_frames[0].context.ambush = false;
        s.ambush = true;
        assert!(!e.is_attacking_using_ambush(&s, a));
        assert_eq!(e.evaluated_power(&s, a), 3);
        s.attack_source = a;
        s.attack_defender = d;
        s.ambush = true;
        s.attack_attacker_departed = true;
        assert!(!e.is_attacking_using_ambush(&s, a));
    }
}
#[test]
fn ambush_context_legal_initial_wounds_and_nested_death_keep_live_bonus() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (mut s, a, d) = board(&e, seat);
        assert!(s.card(a).damage < e.evaluated_hp(&s, a) as u16);
        assert!(s.card(d).damage < e.evaluated_hp(&s, d) as u16);
        s.attack_source = a;
        s.attack_defender = d;
        s.ambush = true;
        s.card_mut(a).damage = 5;
        let context = crate::model::ResolutionContext::capture(&s);
        s.death_frames[0].context = context;
        s.death_depth = 1;
        s.attack_source = 0;
        s.attack_defender = 0;
        s.ambush = false;
        assert_eq!(e.evaluated_power(&s, a), 5);
        s.death_depth = 0;
        assert_eq!(e.evaluated_power(&s, a), 3);
    }
}

#[test]
fn ambush_context_real_native_attack_target_and_cleanup_boundaries() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let e = engine();
            for seat in [Seat::P1, Seat::P2] {
                let (mut s, a, d) = board(&e, seat);
                s.active = seat;
                s.source = a;
                s.decision = Decision::Ambush;
                s.card_mut(a).exhausted = true;
                let actions = e.legal_actions(&s);
                let trigger = actions
                    .iter()
                    .find(|a| a.arg == Some(json!("trigger")))
                    .unwrap();
                s = e.apply(&s, trigger).unwrap();
                assert_eq!(s.decision, Decision::AttackTarget);
                assert!(e
                    .legal_actions(&s)
                    .iter()
                    .all(|a| a.arg != Some(json!("cancel"))));
                s = e.apply(&s, &e.card_action(&s, d)).unwrap();
                assert_eq!(s.card(d).damage, 5);
                assert_eq!(s.card(a).damage, 4);
                assert_eq!(e.evaluated_power(&s, a), 3);
                assert!(!e.is_attacking_using_ambush(&s, a));
                // Ordinary target cancellation never creates an Ambush context or a bonus.
                s.active = seat;
                s.card_mut(a).exhausted = false;
                s = e.apply(&s, &e.card_action(&s, a)).unwrap();
                assert_eq!(s.decision, Decision::AttackTarget);
                let cancel = e
                    .legal_actions(&s)
                    .into_iter()
                    .find(|a| a.arg == Some(json!("cancel")))
                    .unwrap();
                s = e.apply(&s, &cancel).unwrap();
                assert!(!s.card(a).exhausted);
                assert_eq!(e.evaluated_power(&s, a), 3);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn ambush_context_auto_interpreter_aot_read_same_live_flag() {
    for backend in [
        RulesBackend::Auto,
        RulesBackend::Interpreter,
        RulesBackend::Aot,
    ] {
        let mut e = engine();
        e.rules_backend = backend;
        let (mut s, a, d) = board(&e, Seat::P2);
        s.attack_source = a;
        s.attack_defender = d;
        s.ambush = true;
        assert_eq!(e.evaluated_power(&s, a), 5);
        s.ambush = false;
        assert_eq!(e.evaluated_power(&s, a), 3);
    }
}
