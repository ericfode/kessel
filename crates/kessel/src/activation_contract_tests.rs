use super::*;
use crate::ability_ir::{Catalogue, SCHEMA};

fn entry(code: &str) -> Value {
    let catalogue: Value = serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();
    catalogue["cards"].as_array().unwrap().iter().find(|e| e["setCode"] == code).unwrap().clone()
}
fn diagnostic(entry: Value) -> String {
    let code = entry["setCode"].as_str().unwrap().to_owned();
    let prefix = format!("{}: ", entry["internalName"].as_str().unwrap());
    Catalogue::load(&json!({"schema":SCHEMA, "cards":[entry]})).unwrap().rejected[&code]
        .strip_prefix(&prefix).expect("catalogue diagnostics preserve the source name").to_owned()
}

#[test]
fn source_contract_keeps_side_asymmetry_and_printed_raid_separate() {
    for code in ["TWI_019", "TWI_028"] {
        let original = entry(code);
        assert_eq!(base_zone_source(&original), Some(Source::Base));
        let mut wrong = original;
        wrong["abilities"]["leader"] = wrong["abilities"]["card"].take();
        wrong["abilities"]["card"] = json!([]);
        assert_eq!(diagnostic(wrong), "base-zone ongoing source requires exact reviewed base or two-side leader shape");
    }
    let original = entry("HMW_007");
    assert_eq!(base_zone_source(&original), Some(Source::Leader));
    for policy in ["swapped-side", "printed-keyword", "printed-value"] {
        let mut wrong = original.clone();
        match policy {
            "swapped-side" => {
                let front = wrong["abilities"]["leader"].take();
                wrong["abilities"]["leader"] = wrong["abilities"]["unit"].take();
                wrong["abilities"]["unit"] = front;
            }
            "printed-keyword" => wrong["card"]["keywords"] = json!([]),
            _ => wrong["keywordValues"]["raid"] = json!(0),
        }
        assert_eq!(diagnostic(wrong), "base-zone ongoing source requires exact reviewed base or two-side leader shape", "{policy}");
    }
}

#[test]
fn source_contract_preserves_later_confirmation_diagnostic() {
    for (code, side) in [("TWI_019", "card"), ("HMW_007", "leader")] {
        let mut source = entry(code);
        source["abilities"][side][0]["requiresConfirmation"] = json!(true);
        assert!(base_zone_source(&source).is_some(), "confirmation is owned by Program admission");
        assert_eq!(diagnostic(source), "explicit confirmation semantics are not implemented");
    }
}
