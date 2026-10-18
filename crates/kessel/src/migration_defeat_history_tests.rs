//! Source-derived Rust midgame regressions; no forceteki differential execution.
//! Wire below generic_runtime_tests so the existing actual-card builders apply.
use super::*;
use crate::migration_defeat_history_contract::{
    self as contract, Projection, Registration, RelativePlayer,
};

const CANDIDATES: [&str; 9] = [
    "ASH_079", "ASH_093", "IC27_016", "LAW_109", "LOF_263", "SEC_035", "SEC_083", "SOR_051",
    "TWI_143",
];
fn catalog() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(code: &str) -> Value {
    catalog()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn find_query(v: &mut Value) -> Option<&mut Value> {
    if v["kind"] == "historyQuery" {
        return Some(v);
    }
    match v {
        Value::Object(o) => o.values_mut().find_map(find_query),
        Value::Array(a) => a.iter_mut().find_map(find_query),
        _ => None,
    }
}
fn midgame(e: &Engine) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    s
}
fn fund(e: &Engine, s: &mut State, id: CardId) {
    while e.ready_resources(s, Seat::P1) < usize::from(e.cost(s, id)) {
        let r = add(e, s, Seat::P1, "LOF_254");
        s.players[0].resources.push(r).unwrap();
    }
}
fn play(e: &Engine, mut s: State, id: CardId) -> State {
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    fund(e, &mut s, id);
    e.apply(&s, &e.card_action(&s, id)).unwrap()
}
fn resolve(e: &Engine, mut s: State, target: Option<CardId>) -> State {
    for _ in 0..12 {
        if matches!(
            s.decision,
            Decision::Window | Decision::Terminal | Decision::Resource
        ) {
            return s;
        }
        let actions = e.legal_actions(&s);
        let chosen = target
            .and_then(|id| actions.iter().find(|a| a.id() == Some(id)))
            .or_else(|| {
                actions.iter().find(|a| {
                    ["Trigger", "Use it anyway", "Play anyway", "Resolve"]
                        .contains(&a.label.as_str())
                })
            })
            .or_else(|| actions.iter().find(|a| a.arg == Some(json!("all"))));
        let Some(action) = chosen else {
            panic!("unhandled actual prompt {:?}: {actions:?}", s.decision);
        };
        s = e.apply(&s, action).unwrap();
    }
    panic!("unbounded actual-card continuation")
}
fn prior(e: &Engine, s: &mut State, kind: &str) -> Option<CardId> {
    if kind == "none" {
        return None;
    }
    let seat = if kind == "enemy" { Seat::P2 } else { Seat::P1 };
    let id = add(e, s, seat, "LOF_254");
    match kind {
        "discard-placement" => s.player_mut(seat).discard.push(id).unwrap(),
        "departed-without-defeat" => {
            e.add_arena(s, id).unwrap();
            e.leave_play(s, id).unwrap();
        }
        "resource" => {
            // Negative helper diagnostic: defeat outside play is a no-op here;
            // this is not an oracle witness of an engine resource-defeat event.
            s.player_mut(seat).resources.push(id).unwrap();
            e.defeat(s, id).unwrap();
        }
        _ => {
            e.add_arena(s, id).unwrap();
            e.defeat(s, id).unwrap();
        }
    }
    Some(id)
}
#[test]
fn migration_defeat_history_contract_recognizes_nine_canonical_shapes_and_alias() {
    for code in CANDIDATES {
        let c = contract::contract(&entry(code)).unwrap().unwrap();
        assert_eq!(c.captured_power_return, code == "SEC_035");
        assert_eq!(
            c.projection,
            match code {
                "ASH_093" => Projection::SomeLeaderUnit,
                "TWI_143" => Projection::SomeControlledUnit(RelativePlayer::Opponent),
                "SEC_035" => Projection::ControlledUnitCount(RelativePlayer::Opponent),
                _ => Projection::SomeControlledUnit(RelativePlayer::Own),
            }
        );
        if code == "ASH_079" {
            assert_eq!(c.token, Some("mandalorian"));
        }
        if code == "SEC_083" {
            assert_eq!(c.token, Some("spy"));
        }
        if code == "SEC_035" {
            assert_eq!(
                c.registration,
                Registration::EnemyCountExperienceAndCapturedReturn
            );
        }
    }
    let alias = entry("TWI_031");
    assert_eq!(alias["aliasOf"], "IC27_016");
    assert_eq!(
        contract::contract(&alias).unwrap(),
        contract::contract(&entry("IC27_016")).unwrap()
    );
}
#[test]
fn migration_defeat_history_contract_rejects_partial_sources_and_unknown_headers() {
    for code in CANDIDATES {
        let original = entry(code);
        for mutation in [
            "printed-keywords",
            "intrinsic-keywords",
            "class",
            "types",
            "mixed-types",
            "leader-side",
            "extra-ability",
            "helper",
            "attach",
            "pilot",
            "watcher",
            "watcher-binding",
            "query-field",
            "read-time",
            "operation",
            "receiver",
            "registration-field",
            "continuation",
            "forged-projection",
        ] {
            let mut bad = original.clone();
            match mutation {
                "printed-keywords" => bad["card"]["keywords"] = json!(["grit"]),
                "intrinsic-keywords" => bad["keywordValues"] = json!({"raid":99}),
                "class" => bad["baseClass"] = json!("LeaderUnitCard"),
                "types" => bad["card"]["types"] = json!(["leader"]),
                "mixed-types" => bad["card"]["types"] = json!(["unit", "token"]),
                "leader-side" => bad["abilities"]["leader"] = bad["abilities"]["card"].clone(),
                "extra-ability" => {
                    let a = bad["abilities"]["card"][0].clone();
                    bad["abilities"]["card"].as_array_mut().unwrap().push(a);
                }
                "helper" => bad["historyHelpers"] = json!([{"kind":"unreviewedCapture"}]),
                "attach" => bad["attachCondition"] = json!({"kind":"const","value":true}),
                "pilot" => bad["card"]["pilotText"] = json!("not an ordinary unit"),
                "watcher" => bad["stateWatchers"][0]["resetEvent"] = json!("onRoundEnded"),
                "watcher-binding" => bad["stateWatchers"][0]["binding"] = json!("foreignWatcher"),
                "registration-field" => bad["abilities"]["card"][0]["limit"] = json!(1),
                "continuation" => {
                    bad["abilities"]["card"][0]["then"] =
                        json!({"title":"extra","effect":{"kind":"noAction"}})
                }
                other => {
                    let q = find_query(&mut bad["abilities"]).unwrap();
                    match other {
                        "query-field" => q["ignored"] = json!(true),
                        "read-time" => q["readAt"] = json!("registration"),
                        "operation" => q["operation"] = json!("getCurrentValue"),
                        "receiver" => {
                            q["args"] = json!([{"kind":"ref","path":["source","controller"]}])
                        }
                        "forged-projection" => q["defeatProjection"] = json!("friendly"),
                        _ => unreachable!(),
                    }
                }
            }
            assert!(!contract::entry(&bad), "{code}:{mutation}");
        }
    }
}
#[test]
fn migration_defeat_history_contract_alpha_bindings_and_last_known_type_are_closed() {
    let mut good = entry("ASH_093");
    let q = find_query(&mut good["abilities"]).unwrap();
    q["binding"] = json!("differentWatcher");
    q["args"][0]["bindings"] = json!(["local"]);
    q["args"][0]["body"]["args"][0]["root"] = json!(["local"]);
    good["stateWatchers"][0]["binding"] = json!("differentWatcher");
    assert!(contract::entry(&good));
    for mutation in [
        "foreign-root",
        "live-type",
        "optional-type",
        "extra-lambda",
        "unknown-test",
        "long-binding",
    ] {
        let mut bad = good.clone();
        let q = find_query(&mut bad["abilities"]).unwrap();
        match mutation {
            "foreign-root" => q["args"][0]["body"]["args"][0]["root"] = json!(["outer"]),
            "live-type" => {
                q["args"][0]["body"]["args"][0]["steps"] =
                    json!([{"key":"card","optional":false},{"key":"type","optional":false}])
            }
            "optional-type" => {
                q["args"][0]["body"]["args"][0]["steps"][1]["optional"] = json!(true)
            }
            "extra-lambda" => q["args"][0]["bindings"] = json!(["local", "other"]),
            "unknown-test" => q["args"][0]["body"]["test"] = json!("isUnit"),
            "long-binding" => q["binding"] = json!("x".repeat(129)),
            _ => unreachable!(),
        }
        assert!(!contract::entry(&bad), "{mutation}");
    }
    for pointer in [
        "/abilities/card/0/effect/amount/optional",
        "/abilities/card/1/optional",
    ] {
        let mut bad = entry("SEC_035");
        if let Some(v) = bad.pointer_mut(pointer) {
            *v = json!(true);
        } else {
            bad["abilities"]["card"][1]["optional"] = json!(true);
        }
        assert!(!contract::entry(&bad));
    }
    let mut bad = entry("SEC_035");
    bad["abilities"]["card"][1]["effect"]["condition"]["left"]["contextBinding"]["id"] =
        json!("callback:1720:1");
    assert!(!contract::entry(&bad));
}
#[test]
fn migration_defeat_history_core_is_admitted_and_sion_remains_explicitly_held() {
    let e = engine();
    for code in CANDIDATES
        .into_iter()
        .filter(|code| *code != "SEC_035")
        .chain(["TWI_031"])
    {
        assert!(e.by_code.contains_key(code), "exact core must admit {code}");
    }
    assert!(!e.by_code.contains_key("SEC_035"));
    assert!(e.rejected_cards().unwrap().contains_key("SEC_035"));
}
#[test]
fn migration_defeat_history_actual_conditional_heal_and_tokens_require_a_friendly_defeat() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for kind in [
            "none",
            "discard-placement",
            "departed-without-defeat",
            "resource",
            "enemy",
            "friendly",
        ] {
            for (code, token) in [
                ("LAW_109", None),
                ("SEC_083", Some("TOKEN_SPY")),
                ("ASH_079", Some("TOKEN_MANDALORIAN")),
            ] {
                let mut s = midgame(&e);
                prior(&e, &mut s, kind);
                let base = s.players[0].base;
                s.card_mut(base).damage = 6;
                let source = hand(&e, &mut s, code);
                let next = s.next_id;
                let out = resolve(&e, play(&e, s, source), None);
                assert!(e.in_arena(&out, source));
                if let Some(token) = token {
                    let created = (next..out.next_id)
                        .filter(|id| e.def(&out, *id).code == token)
                        .collect::<Vec<_>>();
                    assert_eq!(
                        created.len(),
                        usize::from(kind == "friendly"),
                        "{backend:?}:{code}:{kind}"
                    );
                    for id in created {
                        assert_eq!(out.card(id).owner, Seat::P1);
                        assert!(out.card(id).exhausted);
                        assert!(e.in_arena(&out, id));
                        if token == "TOKEN_MANDALORIAN" {
                            assert_eq!(e.power(&out, id, false), 2);
                            assert_eq!(e.hp(&out, id), 2);
                            let shields = e
                                .upgrades(&out, id)
                                .into_iter()
                                .filter(|upgrade| e.def(&out, *upgrade).code == "TOKEN_SHIELD")
                                .count();
                            assert_eq!(shields, 1, "native Shielded creation must resolve");
                        }
                    }
                    if code == "ASH_079" {
                        assert_eq!(e.has_keyword(&out, source, "sentinel"), kind == "friendly");
                    }
                } else {
                    assert_eq!(
                        out.card(base).damage,
                        if kind == "friendly" { 2 } else { 6 },
                        "{backend:?}:{kind}"
                    );
                }
            }
        }
    }
}
#[test]
fn migration_defeat_history_actual_luke_rune_and_last_words_change_the_chosen_unit() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for friendly in [false, true] {
            for code in ["SOR_051", "IC27_016", "TWI_031", "LOF_263"] {
                let mut s = midgame(&e);
                if friendly {
                    prior(&e, &mut s, "friendly");
                }
                let target = unit(&e, &mut s, Seat::P2, "LAW_109");
                let hp = e.hp(&s, target);
                let power = e.power(&s, target, false);
                let source = hand(&e, &mut s, code);
                let out = resolve(&e, play(&e, s, source), Some(target));
                let delta = match code {
                    "SOR_051" => {
                        if friendly {
                            -6
                        } else {
                            -3
                        }
                    }
                    "LOF_263" => {
                        if friendly {
                            2
                        } else {
                            0
                        }
                    }
                    _ => {
                        if friendly {
                            -1
                        } else {
                            0
                        }
                    }
                };
                assert_eq!(
                    i32::from(e.hp(&out, target)),
                    i32::from(hp) + delta,
                    "{backend:?}:{code}:{friendly}"
                );
                assert_eq!(
                    i32::from(e.power(&out, target, false)),
                    (i32::from(power) + delta).max(0)
                );
                if code == "LOF_263" {
                    assert_eq!(e.upgrades(&out, target).len(), if friendly { 2 } else { 0 });
                }
            }
        }
    }
}
#[test]
fn migration_defeat_history_actual_jyn_is_live_and_phase_reset_removes_both_bonuses() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = midgame(&e);
            let jyn = unit(&e, &mut s, seat, "TWI_143");
            assert_eq!(e.power(&s, jyn, false), 3);
            assert!(!e.has_keyword(&s, jyn, "saboteur"));
            let friendly = unit(&e, &mut s, seat, "LOF_254");
            e.defeat(&mut s, friendly).unwrap();
            assert_eq!(e.power(&s, jyn, false), 3);
            let dead = unit(&e, &mut s, seat.other(), "LOF_254");
            e.defeat(&mut s, dead).unwrap();
            assert_eq!(e.power(&s, jyn, false), 4);
            assert!(e.has_keyword(&s, jyn, "saboteur"));
            s.player_mut(seat.other()).discard.remove(dead);
            s.player_mut(seat.other()).hand.push(dead).unwrap();
            assert_eq!(
                e.power(&s, jyn, false),
                4,
                "leaving discard must not erase defeat history"
            );
            let snap = e.snapshot(&s);
            let restored = e.load_state(&snap).unwrap();
            assert_eq!(restored.zobrist(), s.zobrist());
            assert_eq!(e.power(&restored, jyn, false), 4);
            // Synthetic projection diagnostic only: foreign-control gameplay and
            // imports remain unsupported. A later mutable identity/controller
            // value must not rewrite the recorded controller of this defeat.
            let mut changed = s;
            changed.player_mut(seat.other()).hand.remove(dead);
            changed.card_mut(dead).owner = seat;
            e.add_arena(&mut changed, dead).unwrap();
            assert_eq!(
                e.power(&changed, jyn, false),
                4,
                "event controller must be a snapshot"
            );
            s.players[0].out = true;
            s.players[1].out = true;
            e.advance(&mut s).unwrap();
            assert_eq!(e.power(&s, jyn, false), 3);
            assert!(!e.has_keyword(&s, jyn, "saboteur"));
        }
    }
}
#[test]
fn migration_defeat_history_actual_pellaeon_uses_defeated_leader_unit_lki_after_undeployment() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = midgame(&e);
            let source = unit(&e, &mut s, Seat::P1, "ASH_093");
            let leader = s.player(seat).leader;
            e.defeat(&mut s, leader).unwrap();
            assert_eq!(
                e.power(&s, source, true),
                2,
                "undeployed is not a unit defeat"
            );
            prior(&e, &mut s, "enemy");
            assert_eq!(
                e.power(&s, source, true),
                2,
                "ordinary defeat is not leader defeat"
            );
            s.card_mut(leader).deployed = true;
            e.add_arena(&mut s, leader).unwrap();
            e.defeat(&mut s, leader).unwrap();
            assert!(!s.card(leader).deployed);
            assert_eq!(e.power(&s, source, false), 2);
            assert_eq!(e.power(&s, source, true), 5);
            let restored = e.load_state(&e.snapshot(&s)).unwrap();
            assert_eq!(e.power(&restored, source, true), 5);
            s.players[0].out = true;
            s.players[1].out = true;
            e.advance(&mut s).unwrap();
            assert_eq!(e.power(&s, source, true), 2);
        }
    }
}
#[test]
#[ignore = "SEC_035 held: captured-power return continuation is not admitted"]
fn migration_defeat_history_actual_sion_counts_events_including_reentered_identity_and_token() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for count in [0, 1, 2] {
            let mut s = midgame(&e);
            prior(&e, &mut s, "friendly");
            let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
            for n in 0..count {
                if n > 0 {
                    s.players[1].discard.remove(victim);
                    e.add_arena(&mut s, victim).unwrap();
                }
                e.defeat(&mut s, victim).unwrap();
                e.defeat(&mut s, victim).unwrap(); // duplicate invocation outside play creates no new event.
            }
            if count > 0 {
                let token = unit(&e, &mut s, Seat::P2, "TOKEN_BATTLE_DROID");
                e.defeat(&mut s, token).unwrap();
            }
            let source = hand(&e, &mut s, "SEC_035");
            let out = resolve(&e, play(&e, s, source), None);
            let expected = if count == 0 { 0 } else { count + 1 };
            assert_eq!(
                e.upgrades(&out, source).len(),
                expected,
                "{backend:?}:{count}"
            );
            assert_eq!(usize::from(e.power(&out, source, false)), 5 + expected);
        }
    }
}
#[test]
#[ignore = "SEC_035 held: captured-power return continuation is not admitted"]
fn migration_defeat_history_actual_sion_return_uses_captured_power_and_current_discard_zone() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for bonus in [1, 2] {
            let mut s = midgame(&e);
            let source = unit(&e, &mut s, Seat::P1, "SEC_035");
            s.card_mut(source).phase_power = bonus;
            assert_eq!(e.power(&s, source, false), 5 + bonus as u16);
            e.defeat(&mut s, source).unwrap();
            assert_eq!(s.card(source).phase_power, 0);
            let before = e.snapshot(&s);
            s = e.load_state(&before).unwrap();
            assert_eq!(
                e.snapshot(&s),
                before,
                "pending captured-power return roundtrip"
            );
            e.finish(&mut s).unwrap();
            let out = resolve(&e, s, None);
            assert_eq!(out.players[0].hand.contains(source), bonus == 2);
            assert_eq!(out.players[0].discard.contains(source), bonus != 2);
        }
        let mut s = midgame(&e);
        let source = unit(&e, &mut s, Seat::P1, "SEC_035");
        s.card_mut(source).phase_power = 2;
        e.defeat(&mut s, source).unwrap();
        s.players[0].discard.remove(source);
        s.players[0].resources.push(source).unwrap();
        e.finish(&mut s).unwrap();
        let out = resolve(&e, s, None);
        assert!(out.players[0].resources.contains(source));
        assert!(!out.players[0].hand.contains(source));
    }
}
#[test]
fn migration_defeat_history_actual_removed_play_source_keeps_its_friendly_projection() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = midgame(&e);
        let snoke = unit(&e, &mut s, Seat::P2, "SHD_037");
        // Snoke's -2/-2 defeats the incoming 2-HP Rune after entry. Its captured
        // When Played still resolves and can observe its own friendly defeat.
        let target = unit(&e, &mut s, Seat::P2, "LAW_109");
        let source = hand(&e, &mut s, "IC27_016");
        let out = resolve(&e, play(&e, s, source), Some(target));
        assert!(e.in_arena(&out, snoke));
        assert!(out.players[0].discard.contains(source));
        assert_eq!(e.hp(&out, target), 7);
    }
}
#[test]
fn migration_defeat_history_selected_pool_requires_native_tokens_and_history_version() {
    let e = engine();
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"));
    for (code, token) in [
        ("ASH_079", "TOKEN_MANDALORIAN"),
        ("SEC_083", "TOKEN_SPY"),
        ("LOF_263", "TOKEN_EXPERIENCE"),
    ] {
        let mut pool = e.compiled_pool().unwrap();
        pool["cards"].as_array_mut().unwrap().retain(|r| {
            r["setCode"] == code || r["setCode"] == token || r["setCode"] == "TOKEN_SHIELD"
        });
        assert!(
            Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes).is_ok(),
            "{code}"
        );
        pool["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["setCode"] != token);
        let error = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes)
            .err()
            .unwrap();
        assert!(error.contains(token), "{code}: {error}");
    }
    for (code, token) in [
        ("ASH_079", "TOKEN_MANDALORIAN"),
        ("SEC_083", "TOKEN_SPY"),
        ("LOF_263", "TOKEN_EXPERIENCE"),
    ] {
        let mut bad = catalog();
        let native = bad["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["setCode"] == token)
            .unwrap();
        native["card"]["text"] = json!("unreviewed native-token lifecycle");
        let rejected = Engine::from_catalogue_value(&bad).unwrap();
        assert!(
            !rejected.by_code.contains_key(code),
            "{code} must reject malformed {token}"
        );
    }
    for code in ["TWI_143", "ASH_093", "LAW_109"] {
        let mut pool = e.compiled_pool().unwrap();
        pool["cards"].as_array_mut().unwrap().retain(|r| {
            [
                "ASH_019",
                "SOR_014",
                "IBH_053",
                "LOF_254",
                "TOKEN_SHIELD",
                "TOKEN_EXPERIENCE",
                code,
            ]
            .contains(&r["setCode"].as_str().unwrap())
        });
        let selected =
            Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes).unwrap();
        let s = midgame(&selected);
        let snap = selected.snapshot(&s);
        for shape in ["unknown-field", "partial", "unknown-version"] {
            let mut bad = snap.clone();
            match shape {
                "unknown-field" => bad["phaseHistory"]["ignored"] = json!(true),
                "partial" => {
                    bad["phaseHistory"] = json!({"version":snap["phaseHistory"]["version"]})
                }
                "unknown-version" => bad["phaseHistory"]["version"] = json!(999),
                _ => unreachable!(),
            }
            assert!(selected.load_state(&bad).is_err(), "{code}: {shape}");
        }
        for legacy in [
            None,
            Some(json!({"version":1,"entered":[],"played":[],"playCounts":[0,0]})),
        ] {
            let mut old = snap.clone();
            if let Some(history) = legacy {
                old["phaseHistory"] = history;
            } else {
                old.as_object_mut().unwrap().remove("phaseHistory");
            }
            assert!(
                selected.load_state(&old).is_err(),
                "missing defeat evidence for {code}"
            );
            pool["cards"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["setCode"] != code);
            let unrelated =
                Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes)
                    .unwrap();
            assert!(
                unrelated.load_state(&old).is_ok(),
                "unrelated pool retains bounded legacy compatibility: {code}"
            );
        }
    }
}

#[test]
fn migration_defeat_history_reentered_identity_and_native_token_are_distinct_events() {
    let e = engine();
    let mut s = midgame(&e);
    let victim = unit(&e, &mut s, Seat::P2, "LOF_254");
    for expected in 1..=2 {
        e.defeat(&mut s, victim).unwrap();
        assert_eq!(s.phase_history.defeated_unit_counts, [0, expected]);
        e.defeat(&mut s, victim).unwrap();
        assert_eq!(s.phase_history.defeated_unit_counts, [0, expected]);
        if expected == 1 {
            s.players[1].discard.remove(victim);
            e.add_arena(&mut s, victim).unwrap();
        }
    }
    let token = unit(&e, &mut s, Seat::P2, "TOKEN_BATTLE_DROID");
    e.defeat(&mut s, token).unwrap();
    assert_eq!(s.phase_history.defeated_unit_counts, [0, 3]);
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["phaseHistory"]["version"], 2);
    assert_eq!(
        snapshot["phaseHistory"]["defeatedUnitCounts"],
        json!([0, 3])
    );
    assert_eq!(snapshot["phaseHistory"]["defeatedLeaderCount"], 0);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(restored.phase_history, s.phase_history);
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.phase_history.defeated_unit_counts, [0, 0]);
    assert_eq!(s.phase_history.defeated_leader_count, 0);
    assert!(s.phase_history.defeats_exact);
}

#[test]
fn migration_defeat_history_snapshot_counts_are_hashed_and_strictly_bounded() {
    let e = engine();
    let mut s = midgame(&e);
    prior(&e, &mut s, "enemy");
    let snapshot = e.snapshot(&s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(restored.zobrist(), s.zobrist());
    let mut changed = s;
    changed.phase_history.defeated_unit_counts[1] += 1;
    assert_ne!(changed.zobrist(), s.zobrist());
    changed = s;
    changed.phase_history.defeated_leader_count = 1;
    assert_ne!(changed.zobrist(), s.zobrist());
    for invalid in [
        json!([-1, 0]),
        json!([65536, 0]),
        json!([0]),
        json!([0, 0, 0]),
        json!(["1", 0]),
        json!(null),
    ] {
        let mut bad = snapshot.clone();
        bad["phaseHistory"]["defeatedUnitCounts"] = invalid;
        assert!(e.load_state(&bad).is_err());
    }
    for invalid in [json!(-1), json!(65536), json!(2), json!(null)] {
        let mut bad = snapshot.clone();
        bad["phaseHistory"]["defeatedLeaderCount"] = invalid;
        assert!(e.load_state(&bad).is_err());
    }
    for field in ["defeatedUnitCounts", "defeatedLeaderCount"] {
        let mut bad = snapshot.clone();
        bad["phaseHistory"].as_object_mut().unwrap().remove(field);
        assert!(e.load_state(&bad).is_err(), "missing {field}");
    }
}

#[test]
fn migration_defeat_history_overflow_rejects_before_removing_the_unit() {
    let e = engine();
    for leader_unit in [false, true] {
        let mut s = midgame(&e);
        let victim = if leader_unit {
            let leader = s.players[1].leader;
            s.card_mut(leader).deployed = true;
            e.add_arena(&mut s, leader).unwrap();
            leader
        } else {
            unit(&e, &mut s, Seat::P2, "LOF_254")
        };
        s.phase_history.defeated_unit_counts[1] = u16::MAX;
        if leader_unit {
            s.phase_history.defeated_leader_count = u16::MAX;
        }
        let before = e.snapshot(&s);
        let hash = s.zobrist();
        assert!(e.defeat(&mut s, victim).is_err());
        assert_eq!(e.snapshot(&s), before);
        assert_eq!(s.zobrist(), hash);
        assert!(e.in_arena(&s, victim));
    }
}

#[test]
fn migration_defeat_history_duplicate_simultaneous_identity_records_one_event() {
    let e = engine();
    let mut s = midgame(&e);
    let victim = unit(&e, &mut s, Seat::P1, "LOF_254");
    e.defeat_many(&mut s, &[victim, victim]).unwrap();
    assert_eq!(s.phase_history.defeated_unit_counts, [1, 0]);
    assert_eq!(s.phase_history.defeated_leader_count, 0);
}
#[test]
fn migration_defeat_history_unknown_history_rejects_before_live_stat_evaluation() {
    let e = engine();
    for code in ["TWI_143", "ASH_093"] {
        let mut s = midgame(&e);
        unit(&e, &mut s, Seat::P1, code);
        let action = e.legal_actions(&s).into_iter().next().unwrap();
        for phase_exact in [false, true] {
            let mut unknown = s;
            unknown.phase_history.exact = phase_exact;
            unknown.phase_history.defeats_exact = false;
            assert!(e
                .apply(&unknown, &action)
                .unwrap_err()
                .contains("exact defeat phase history"));
        }
    }
}

// Exercise the public event-play continuation, not only Engine::defeat helpers.
#[test]
fn migration_defeat_history_actual_vanquish_activates_jyn_through_public_play() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for code in ["SOR_078", "TWI_077"] {
                assert!(
                    e.by_code.contains_key(code),
                    "native Vanquish printing {code}"
                );
                let mut s = midgame(&e);
                let jyn = unit(&e, &mut s, seat, "TWI_143");
                let victim = unit(&e, &mut s, seat.other(), "LOF_254");
                let event = add(&e, &mut s, seat, code);
                s.player_mut(seat).hand.push(event).unwrap();
                while e.ready_resources(&s, seat) < usize::from(e.cost(&s, event)) {
                    let resource = add(&e, &mut s, seat, "LOF_254");
                    s.player_mut(seat).resources.push(resource).unwrap();
                }
                s.active = seat;
                s.action_player = Some(seat);
                assert_eq!(e.power(&s, jyn, false), 3);
                assert!(!e.has_keyword(&s, jyn, "saboteur"));
                let played = e.apply(&s, &e.card_action(&s, event)).unwrap();
                let out = resolve(&e, played, Some(victim));
                assert!(out.player(seat.other()).discard.contains(victim));
                assert_eq!(
                    out.phase_history.defeated_unit_counts[seat.other().index()],
                    1
                );
                assert_eq!(out.phase_history.defeated_unit_counts[seat.index()], 0);
                assert_eq!(e.power(&out, jyn, false), 4, "{backend:?}:{seat:?}:{code}");
                assert!(e.has_keyword(&out, jyn, "saboteur"));
            }
        }
    }
}

#[test]
fn migration_defeat_history_actual_takedown_captures_leader_unit_before_undeployment() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        assert!(e.by_code.contains_key("SOR_077"));
        for seat in [Seat::P1, Seat::P2] {
            let mut s = midgame(&e);
            let pellaeon = unit(&e, &mut s, seat, "ASH_093");
            let leader = s.player(seat.other()).leader;
            s.card_mut(leader).deployed = true;
            e.add_arena(&mut s, leader).unwrap();
            s.card_mut(leader).damage = e.hp(&s, leader).saturating_sub(5);
            let event = add(&e, &mut s, seat, "SOR_077");
            s.player_mut(seat).hand.push(event).unwrap();
            while e.ready_resources(&s, seat) < usize::from(e.cost(&s, event)) {
                let resource = add(&e, &mut s, seat, "LOF_254");
                s.player_mut(seat).resources.push(resource).unwrap();
            }
            s.active = seat;
            s.action_player = Some(seat);
            assert_eq!(e.power(&s, pellaeon, true), 2);
            let played = e.apply(&s, &e.card_action(&s, event)).unwrap();
            let out = resolve(&e, played, Some(leader));
            assert!(!out.card(leader).deployed);
            assert_eq!(
                out.phase_history.defeated_unit_counts[seat.other().index()],
                1
            );
            assert_eq!(out.phase_history.defeated_leader_count, 1);
            assert_eq!(e.power(&out, pellaeon, false), 2);
            assert_eq!(e.power(&out, pellaeon, true), 5, "{backend:?}:{seat:?}");
        }
    }
}

#[test]
fn migration_defeat_history_synthetic_grouped_explicit_defeats_share_event_accounting() {
    // Synthetic shared-capability diagnostic: this is not a native-card witness.
    let e = engine();
    let mut s = midgame(&e);
    let first = unit(&e, &mut s, Seat::P1, "LOF_254");
    let second = unit(&e, &mut s, Seat::P2, "LOF_254");
    s.source = first;
    s.ability_target = second;
    s.event_group.active = true;
    s.event_group.context = Selection::one(second);
    s.event_group.initiator = Seat::P1;
    let group = Effect::Simultaneous {
        effects: vec![
            Effect::Defeat {
                target: Target::name("self"),
            },
            Effect::Defeat {
                target: Target::name("chosen"),
            },
        ],
    };
    e.generate_group(&mut s, &group).unwrap();
    e.resolve_group(&mut s, &group).unwrap();
    assert_eq!(s.phase_history.defeated_unit_counts, [1, 1]);
    assert_eq!(s.phase_history.defeated_leader_count, 0);
    assert!(s.players[0].discard.contains(first));
    assert!(s.players[1].discard.contains(second));
}

#[test]
fn migration_defeat_history_group_overflow_preserves_both_victims_and_attachments() {
    let e = engine();
    let mut s = midgame(&e);
    let first = unit(&e, &mut s, Seat::P2, "LOF_254");
    let second = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.create_token(&mut s, first, false).unwrap();
    e.create_token(&mut s, second, false).unwrap();
    let shields = [s.card(first).shield, s.card(second).shield];
    s.source = first;
    s.ability_target = second;
    s.event_group.active = true;
    s.event_group.context = Selection::one(second);
    s.event_group.initiator = Seat::P2;
    let group = Effect::Simultaneous { effects: vec![
        Effect::Defeat { target: Target::name("self") },
        Effect::Defeat { target: Target::name("chosen") },
    ]};
    e.generate_group(&mut s, &group).unwrap();
    s.phase_history.defeated_unit_counts[1] = u16::MAX - 1;
    let before = e.snapshot(&s);
    let hash = s.zobrist();
    assert!(e.resolve_group(&mut s, &group).unwrap_err().contains("count capacity"));
    assert_eq!(e.snapshot(&s), before);
    assert_eq!(s.zobrist(), hash);
    for (victim, shield) in [first, second].into_iter().zip(shields) {
        assert!(e.in_arena(&s, victim));
        assert_eq!(s.card(victim).shield, shield);
        assert_eq!(s.card(shield).parent, victim);
    }
}
