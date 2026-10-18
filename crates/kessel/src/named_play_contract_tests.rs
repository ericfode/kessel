use super::*;
fn entry(code: &str) -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn replace(entry: &mut Value, path: &str, value: Value) {
    let (parent, key) = path.rsplit_once('/').unwrap();
    let parent = entry.pointer_mut(parent).unwrap();
    if let Some(o) = parent.as_object_mut() {
        o.insert(key.into(), value);
    } else {
        parent[key.parse::<usize>().unwrap()] = value;
    }
}
#[test]
fn named_play_contract_complete_identical_sources_preserve_original_registration() {
    let mut descriptors = Vec::new();
    for code in ["ASH_077", "SOR_062"] {
        let original = entry(code);
        let bytes = serde_json::to_vec(&original).unwrap();
        let prepared = prepare(&original).unwrap().unwrap();
        assert_eq!(prepared.registration, original["abilities"]["card"][0]);
        assert_eq!(serde_json::to_vec(&original).unwrap(), bytes);
        let mut diagnostic = original.clone();
        diagnostic["setCode"] = json!("DIAGNOSTIC_NAMED_PLAY");
        diagnostic["card"]["title"] = json!("Diagnostic governor");
        assert_eq!(prepare(&diagnostic).unwrap(), Some(prepared.clone()));
        descriptors.push(prepared.descriptor);
    }
    assert_eq!(descriptors[0], descriptors[1]);
    assert_eq!(REGISTRY_COUNT, 1931);
    assert_eq!(
        REGISTRY_SHA256,
        "3c793f2120a9b07d451e4166fa3841832775f8371a102b60ea0329f79852b717"
    );
}
#[test]
fn named_play_contract_alpha_renaming_requires_bound_declarations_and_exact_roles() {
    let mut e = entry("SOR_062");
    fn rename(v: &mut Value) {
        if v["kind"] == "ref" {
            if v["path"][0] == "attackBinding1_0" {
                v["path"][0] = json!("systemRenamed");
            } else if v["path"][0] == "attackBinding2_0" {
                v["path"][0] = json!("attemptRenamed");
            }
        }
        match v {
            Value::Array(a) => {
                for v in a {
                    rename(v)
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    rename(v)
                }
            }
            _ => {}
        }
    }
    rename(&mut e["abilities"]);
    let effect = &mut e["abilities"]["card"][0]["then"]["effect"];
    effect["parameters"][0]["root"] = json!("systemRenamed");
    effect["body"]["effects"][0]["condition"]["parameters"][0]["root"] = json!("attemptRenamed");
    assert!(prepare(&e).unwrap().is_some());
    let mut wrong = e.clone();
    wrong["abilities"]["card"][0]["then"]["effect"]["body"]["effects"][0]["condition"]["body"]
        ["left"]["path"][0] = json!("systemRenamed");
    assert!(prepare(&wrong).is_err());
    let mut wrong = e.clone();
    wrong["abilities"]["card"][0]["then"]["effect"]["body"]["effects"][0]["condition"]
        ["parameters"][0]["role"] = json!("systemContext");
    assert!(prepare(&wrong).is_err());
}
#[test]
fn named_play_contract_source_headers_registry_lifetime_and_title_capture_fail_closed() {
    for code in ["ASH_077", "SOR_062"] {
        for (path, value) in [
            ("/baseClass", json!("EventCard")),
            ("/status", json!("unsupported")),
            ("/card/types", json!(["unit", "token"])),
            ("/card/hp", json!(0)),
            ("/keywordValues", json!({"restore":2})),
            ("/card/pilotBox", json!(false)),
            ("/stateWatchers", json!([{}])),
            ("/abilities/unit", json!([{}])),
            ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
            ("/abilities/card/0/optional", json!(true)),
            (
                "/abilities/card/0/targetResolver/options/order",
                json!("sortedDefinitions"),
            ),
            (
                "/abilities/card/0/targetResolver/options/capture",
                json!("currentHand"),
            ),
            (
                "/abilities/card/0/targetResolver/logSelection",
                json!(false),
            ),
            (
                "/abilities/card/0/then/effect/body/duration",
                json!("untilEndOfPhase"),
            ),
            (
                "/abilities/card/0/then/effect/body/requiresSourceInPlayAtApplication",
                json!(true),
            ),
            (
                "/abilities/card/0/then/effect/body/sourceLifetime",
                json!("independent"),
            ),
            (
                "/abilities/card/0/then/effect/body/target/path/2",
                json!("self"),
            ),
            (
                "/abilities/card/0/then/effect/body/effects/0/restriction",
                json!("attack"),
            ),
            (
                "/abilities/card/0/then/effect/body/effects/0/condition/body/left/path/3",
                json!("subtitle"),
            ),
            (
                "/abilities/card/0/then/effect/body/effects/0/condition/body/right/path/0",
                json!("target"),
            ),
        ] {
            let mut e = entry(code);
            replace(&mut e, path, value);
            assert!(prepare(&e).is_err(), "{code} accepted mutation {path}");
        }
    }
    assert!(prepare(&entry("LAW_243")).is_err());
    assert!(prepare(&entry("SHD_202")).unwrap().is_none());
}
#[test]
fn named_play_contract_every_registration_object_rejects_unknown_fields() {
    fn objects(value: &Value, path: String, out: &mut Vec<String>) {
        match value {
            Value::Object(o) => {
                out.push(path.clone());
                for (k, v) in o {
                    objects(v, format!("{path}/{k}"), out)
                }
            }
            Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    objects(v, format!("{path}/{i}"), out)
                }
            }
            _ => {}
        }
    }
    for code in ["ASH_077", "SOR_062"] {
        let original = entry(code);
        let mut paths = Vec::new();
        objects(
            &original["abilities"]["card"][0],
            "/abilities/card/0".into(),
            &mut paths,
        );
        for path in paths {
            let mut e = original.clone();
            e.pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(false));
            assert!(prepare(&e).is_err(), "{code} accepted unknown at {path}");
        }
    }
}
#[test]
fn named_play_contract_malformed_containers_never_panic_or_pass() {
    let original = entry("SOR_062");
    for path in [
        "/abilities",
        "/abilities/card",
        "/abilities/card/0",
        "/abilities/card/0/then/effect/parameters",
        "/abilities/card/0/then/effect/body/effects/0/condition/parameters",
    ] {
        for value in [
            Value::Null,
            json!(true),
            json!(1),
            json!("bad"),
            json!([]),
            json!({}),
        ] {
            let mut e = original.clone();
            replace(&mut e, path, value);
            let result = std::panic::catch_unwind(|| prepare(&e));
            assert!(result.is_ok(), "panic at {path}");
            assert!(!matches!(result.unwrap(), Ok(Some(_))), "accepted {path}");
        }
    }
}
