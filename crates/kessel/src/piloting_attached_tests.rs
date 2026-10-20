//! Dormant source contracts remain checked; whole-card Piloting admission is rejected.
use super::*;
use serde_json::{json, Value};

#[test]
fn piloting_attached_keyword_rejects_partial_shapes() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_034")
        .unwrap()
        .clone();
    let admitted = |e: Value| crate::piloting_attached::entry(&e);
    assert!(admitted(original.clone()));
    for (path, value) in [
        ("/baseClass", json!("UpgradeCard")),
        ("/card/types", json!(["upgrade"])),
        ("/card/pilotText", json!(null)),
        ("/card/keywords", json!(["grit"])),
        ("/keywordValues", json!({"grit": 1, "raid": 1})),
        ("/abilities/card/0/effect/kind", json!("gainKeyword")),
        ("/abilities/card/0/effect/modifier/keyword", json!("ambush")),
        (
            "/abilities/card/0/effect/requiresGrantorInPlay",
            json!(false),
        ),
        ("/card/upgradePower", json!(1001)),
        (
            "/abilities/card/0/effect/gainCondition",
            json!({"kind": "const", "value": true}),
        ),
    ] {
        let mut bad = original.clone();
        let parts: Vec<_> = path.split('/').skip(1).collect();
        let mut target = &mut bad;
        for k in &parts[..parts.len() - 1] {
            target = if target.is_array() {
                &mut target[k.parse::<usize>().unwrap()]
            } else {
                &mut target[*k]
            };
        }
        target[parts[parts.len() - 1]] = value;
        assert!(!admitted(bad), "{path}");
    }
}
