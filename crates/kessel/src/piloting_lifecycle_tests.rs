//! Regression for the actual missing Normal/Piloting mode choice. A printed
//! Pilot cannot be offered only its ordinary play or accepted only when attached.
use super::*;
use std::collections::BTreeSet;
const PILOTS: [&str; 19] = [
    "JTL_034", "JTL_035", "JTL_036", "JTL_045", "JTL_046", "JTL_057", "JTL_058", "JTL_066",
    "JTL_084", "JTL_086", "JTL_141", "JTL_142", "JTL_145", "JTL_148", "JTL_150", "JTL_189",
    "JTL_210", "JTL_211", "JTL_215",
];
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn strict() -> Value {
    serde_json::from_str(include_str!("../data/pool.json")).unwrap()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn entry(c: &Value, code: &str) -> Value {
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn replacement(c: &Value, e: Value) -> Value {
    let mut c = c.clone();
    let code = e["setCode"].clone();
    *c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == code)
        .unwrap() = e;
    c
}
fn code_set(pool: &Value) -> BTreeSet<String> {
    pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["setCode"].as_str().unwrap().into())
        .collect()
}
fn native(reason: &str) {
    assert!(reason.contains("Piloting play lifecycle"), "{reason}");
}
#[test]
fn all_nineteen_whole_cards_reject_without_erasing_ir_or_other_admissions() {
    let c = catalogue();
    let before = c.clone();
    let loaded = Catalogue::load(&c).unwrap();
    let e = engine();
    for code in PILOTS {
        let raw = entry(&c, code);
        assert_eq!(raw["status"], json!("supported"));
        assert!(!raw["abilities"]["card"].as_array().unwrap().is_empty());
        assert!(!loaded.cards.contains_key(code));
        native(&loaded.rejected[code]);
        assert!(!e.by_code.contains_key(code));
        native(&e.rejected_cards().unwrap()[code]);
    }
    assert_eq!(c, before, "admission must not mutate source conversion");
    let report: Value = serde_json::from_str(include_str!(
        "../../../compiler/generated/runtime-admission.json"
    ))
    .unwrap();
    let expected: BTreeSet<String> = report["runtimeAdmittedCards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["setCode"].as_str().unwrap())
        .filter(|id| !PILOTS.contains(id))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        code_set(&e.compiled_pool().unwrap()),
        expected,
        "unrelated admitted definitions must stay unchanged"
    );
    assert_eq!(strict()["cards"].as_array().unwrap().len(), 100);
    assert!(crate::validation::validate_pool(&strict()).is_ok());
    assert!(Engine::from_pool_value(&strict()).is_ok());
}
fn raw_mutations() -> Vec<(String, Value)> {
    let mut variants = Vec::new();
    for field in ["pilotText", "pilotBox", "pilotingBox"] {
        for value in [
            json!("Piloting [0]"),
            json!(" "),
            json!({}),
            json!([]),
            json!(0),
            json!(false),
        ] {
            variants.push((field.into(), value));
        }
    }
    variants.extend([
        ("piloting".into(), json!({"cost":0})),
        ("piloting".into(), json!(false)),
        ("pilotTag".into(), json!(0)),
        ("keywords".into(), json!(["piloting"])),
        ("keywords".into(), json!("piloting")),
        ("keywords".into(), json!({"piloting":0})),
        ("keywordValues".into(), json!({"piloting":0})),
        ("keywordValues".into(), json!({"piloting":null})),
        ("keywordValues".into(), json!(["piloting"])),
    ]);
    for field in ["pilotingAbility", "pilotUnitAbility"] {
        for value in [json!(true), json!(0), json!({}), json!("false")] {
            variants.push((field.into(), value));
        }
    }
    for kind in [
        "pilotingAttachedKeyword",
        "pilotingAttachedModifier",
        "pilotingRegistration",
        "pilotTag",
    ] {
        variants.push(("abilities".into(), json!({"card":[{"kind":kind}]})));
    }
    variants.push(("abilities".into(),json!({"card":[{"kind":"attachedAbilityRegistration","registrationMode":"piloting","cost":0}]})));
    variants
}
#[test]
fn each_raw_marker_is_rejected_before_projection_at_every_pool_boundary() {
    let c = catalogue();
    let e = engine();
    let pool = e.compiled_pool().unwrap();
    let strict = strict();
    let bytes = serde_json::to_vec(&c).unwrap();
    for (field, value) in raw_mutations() {
        let mut raw = entry(&c, "LOF_254");
        raw["card"][&field] = value.clone();
        let changed = replacement(&c, raw);
        let loaded = Catalogue::load(&changed).unwrap();
        native(&loaded.rejected["LOF_254"]);
        let e = Engine::from_catalogue_value(&changed).unwrap();
        assert!(!e.by_code.contains_key("LOF_254"), "Engine field {field}");
        let mut p = pool.clone();
        p["cards"][0][&field] = value.clone();
        let result = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&p).unwrap(), &bytes);
        assert!(result.is_err(), "compiled pool {field}");
        native(&result.err().unwrap());
        let mut p = strict.clone();
        p["cards"][0][&field] = value;
        native(&crate::validation::validate_pool(&p).unwrap_err());
        assert!(Engine::from_pool_value(&p).is_err(), "strict pool {field}");
    }
}
#[test]
fn entry_headers_and_aliases_cannot_launder_native_piloting() {
    let c = catalogue();
    for field in [
        "piloting",
        "pilotText",
        "pilotBox",
        "pilotingBox",
        "pilotTag",
        "keywordValues",
    ] {
        let mut raw = entry(&c, "LOF_254");
        raw[field] = if field == "keywordValues" {
            json!({"piloting":0})
        } else {
            json!({})
        };
        let changed = replacement(&c, raw);
        let loaded = Catalogue::load(&changed).unwrap();
        native(&loaded.rejected["LOF_254"]);
    }
    let mut c = c;
    let mut alias = entry(&c, "LOF_254");
    alias["setCode"] = json!("TEST_001");
    alias["aliasOf"] = json!("JTL_211");
    c["cards"].as_array_mut().unwrap().push(alias);
    let e = Engine::from_catalogue_value(&c).unwrap();
    assert!(!e.by_code.contains_key("TEST_001"));
    native(&e.rejected_cards().unwrap()["TEST_001"]);
    let mut pilot = entry(&c, "JTL_211");
    pilot["card"]["setCodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"set":"TEST","number":2}));
    let c = replacement(&c, pilot);
    let e = Engine::from_catalogue_value(&c).unwrap();
    assert!(!e.by_code.contains_key("TEST_002"));
    let mut p = e.compiled_pool().unwrap();
    let mut stripped = entry(&c, "LOF_254")["card"].clone();
    stripped["setCode"] = json!("TEST_001");
    p["cards"].as_array_mut().unwrap().push(stripped);
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&p).unwrap(),
        &serde_json::to_vec(&c).unwrap()
    )
    .is_err());
}
#[test]
fn pilot_trait_alone_and_inert_native_metadata_remain_non_piloting() {
    let c = catalogue();
    let mut plain = entry(&c, "LOF_254");
    plain["card"]["traits"]
        .as_array_mut()
        .unwrap()
        .push(json!("pilot"));
    for field in ["pilotText", "pilotBox", "pilotingBox"] {
        plain["card"][field] = json!("");
    }
    plain["card"]["piloting"] = Value::Null;
    assert!(crate::piloting_lifecycle::reject_card(&plain["card"]).is_ok());
    let c = replacement(&c, plain);
    let e = Engine::from_catalogue_value(&c).unwrap();
    assert!(e.by_code.contains_key("LOF_254"));
    let mut selected = e.compiled_pool().unwrap();
    selected["cards"].as_array_mut().unwrap().retain(|r| {
        ["LOF_254", "TOKEN_SHIELD"]
            .iter()
            .any(|id| r["setCode"] == *id)
    });
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&selected).unwrap(),
        &serde_json::to_vec(&c).unwrap()
    )
    .is_ok());
    let mut ordinary = strict();
    for field in ["pilotText", "pilotBox", "pilotingBox", "piloting"] {
        ordinary["cards"][0][field] = Value::Null;
    }
    assert!(Engine::from_pool_value(&ordinary).is_ok());
}
#[test]
fn rejected_pilot_identity_cannot_enter_an_attached_state_or_selected_pool() {
    let c = catalogue();
    let e = engine();
    let mut selected = e.compiled_pool().unwrap();
    let mut raw = entry(&c, "JTL_211")["card"].clone();
    raw["setCode"] = json!("JTL_211");
    raw.as_object_mut().unwrap().remove("pilotText");
    raw["keywords"] = json!([]);
    selected["cards"].as_array_mut().unwrap().push(raw);
    let result = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&selected).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    );
    assert!(result.is_err());
    native(&result.err().unwrap());
    // Build a valid unrelated physical state, then ask to import a rejected Pilot
    // as an attachment. Identity resolution must fail before link/stat execution.
    let mut s = State::default();
    s.round = 7;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    s.arena_order_exact = true;
    s.force_state_exact = e.needs_force_state();
    s.credit_state_exact = e.needs_credit_state();
    s.hidden_history_exact = true;
    let add = |s: &mut State, owner: Seat, code: &str| {
        let id = s.next_id;
        s.next_id += 1;
        *s.card_mut(id) = Card {
            present: true,
            owner,
            def: e.by_code[code],
            ..Card::default()
        };
        id
    };
    for seat in [Seat::P1, Seat::P2] {
        let base = add(&mut s, seat, "ASH_019");
        s.player_mut(seat).base = base;
        let leader = add(&mut s, seat, "SOR_014");
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    let host = add(&mut s, Seat::P1, "LOF_192");
    e.add_arena(&mut s, host).unwrap();
    let mut state = e.snapshot(&s);
    let id = s.next_id;
    state["nextCardId"] = json!(id);
    state["players"]["p1"]["space"][0]["upgrades"].as_array_mut().unwrap().push(json!({"id":format!("c{}",id-1),"setCode":"JTL_211","name":"independent-smuggler","owner":"p1","controller":"p1","token":false}));
    assert!(e.load_state(&state).is_err());
}

#[test]
fn early_special_family_returns_do_not_drop_raw_native_headers_or_normalized_flags() {
    let c = catalogue();
    let source = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| matches!(crate::named_play_contract::prepare(entry), Ok(Some(_))))
        .expect("reviewed early naming family")
        .clone();
    let code = source["setCode"].as_str().unwrap();
    assert!(Catalogue::load(&c).unwrap().cards.contains_key(code));
    for field in ["pilotBox", "pilotingBox"] {
        let mut raw = source.clone();
        raw["card"][field] = json!({});
        let changed = replacement(&c, raw);
        native(&Catalogue::load(&changed).unwrap().rejected[code]);
    }
    for field in ["pilotingAbility", "pilotUnitAbility"] {
        for value in [json!(true), json!(0), json!({})] {
            let mut raw = source.clone();
            raw["abilities"]["card"][0][field] = value;
            let changed = replacement(&c, raw);
            native(&Catalogue::load(&changed).unwrap().rejected[code]);
        }
    }
}

#[test]
fn raw_value_scans_reject_excess_depth_and_width_explicitly() {
    let mut deep = json!(null);
    for _ in 0..256 {
        deep = Value::Array(vec![deep]);
    }
    let deep = json!({"keywordValues":deep});
    let wide = json!({"keywords":vec![Value::Null; 65_537]});
    for value in [&deep, &wide] {
        let error = crate::piloting_lifecycle::reject_card(value)
            .expect_err("raw scan budget must reject oversized values");
        assert!(error.contains("inspection budget"), "{error}");
        assert!(crate::piloting_lifecycle::reject_entry(value).is_err());
    }
}
