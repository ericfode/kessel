use super::*;
#[test]
fn created_token_phase_keyword_source_and_dependency_guards() {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let e = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_091")
        .unwrap();
    let p = prepare(e, &c).unwrap().unwrap();
    assert!(program(&p));
    let mut renamed = e.clone();
    renamed["setCode"] = json!("TEST_001");
    assert!(prepare(&renamed, &c).unwrap().is_some());
    for field in ["extra", "card", "abilities", "sourceHash", "diagnostics"] {
        let mut bad = e.clone();
        bad[field] = json!("changed");
        assert!(prepare(&bad, &c).is_err(), "{field}");
    }
    for code in ["TOKEN_MANDALORIAN", "TOKEN_SHIELD"] {
        let mut bad = c.clone();
        bad["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|e| e["setCode"] != code);
        assert!(prepare(e, &bad).is_err());
    }
}

#[test]
fn created_token_phase_keyword_nested_binding_and_native_profile_mutants() {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let e = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_091")
        .unwrap();
    let mutations = [
        ("/abilities/card/0/effect/amount/value", json!(2)),
        ("/abilities/card/0/effect/entersReady", json!(true)),
        (
            "/abilities/card/0/then/effect/target/collection",
            json!("events"),
        ),
        ("/abilities/card/0/then/effect/target/eventIndex", json!(1)),
        ("/abilities/card/0/then/effect/target/path", json!(["card"])),
        (
            "/abilities/card/0/then/effect/target/optionalAccess",
            json!([]),
        ),
        (
            "/abilities/card/0/then/effect/target/contextBinding/role",
            json!("ifYouDo"),
        ),
        (
            "/abilities/card/0/then/effect/duration",
            json!("persistent"),
        ),
        ("/abilities/card/0/then/effect/keyword", json!("shielded")),
        ("/card/types", json!(["unit"])),
        ("/card/keywords", json!([])),
        ("/keywordValues", json!({"sentinel":1})),
    ];
    for (path, value) in mutations {
        let mut bad = e.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(prepare(&bad, &c).is_err(), "{path}");
    }
    let mut bad = e.clone();
    bad["abilities"]["card"][0]["optional"] = json!(true);
    assert!(prepare(&bad, &c).is_err());
    for code in ["TOKEN_MANDALORIAN", "TOKEN_SHIELD"] {
        let mut bad = c.clone();
        let token = bad["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["setCode"] == code)
            .unwrap();
        token["extra"] = json!(true);
        assert!(prepare(e, &bad).is_err());
    }
    assert!(
        serde_json::from_value::<Ability>(e["abilities"]["card"][0].clone()).is_err(),
        "raw eventResult target must remain unrepresentable"
    );
    let mut p = prepare(e, &c).unwrap().unwrap();
    if let Instruction::GrantCreatedTokenPhaseKeyword { creation_pc, .. } = &mut p.instructions[1] {
        *creation_pc = 1;
    } else {
        panic!("typed continuation missing");
    }
    assert!(!program(&p));
}
