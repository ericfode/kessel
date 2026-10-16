use super::*;
use serde_json::json;

fn entry(code: &str) -> Value {
    let catalogue: Value = serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    catalogue["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==code).unwrap().clone()
}
fn outcome(entry: Value) -> Result<(), String> {
    let code = entry["setCode"].as_str().unwrap().to_owned();
    let prefix = format!("{}: ", entry["internalName"].as_str().unwrap());
    let catalogue = Catalogue::load(&json!({"schema":SCHEMA,"cards":[entry]})).unwrap();
    if catalogue.cards.contains_key(&code) { Ok(()) }
    else { Err(catalogue.rejected[&code].strip_prefix(&prefix).expect("source-prefixed diagnostic").to_owned()) }
}

#[test]
fn source_guard_type_and_side_diagnostics_dominate_family_metadata() {
    for code in ["JTL_119", "SEC_130"] {
        let source = entry(code);
        assert!(outcome(source.clone()).is_ok());
        for types in [Value::Null, json!(false), json!({}), json!(["unit","unit"]), json!(["upgrade"])] {
            let mut wrong = source.clone();
            wrong["card"]["types"] = types;
            wrong["piloting"] = json!({});
            wrong["abilities"]["leader"] = json!([{"title":"extra","effect":{"kind":"noAction"}}]);
            assert_eq!(outcome(wrong).unwrap_err(), "source class and card type metadata disagree", "{code}");
        }
        for side in ["leader", "unit"] {
            let mut wrong = source.clone();
            wrong["abilities"][side] = json!([{"title":"extra","effect":{"kind":"noAction"}}]);
            wrong["card"]["deployCost"] = json!(9);
            assert_eq!(outcome(wrong).unwrap_err(), "ordinary sources require card-side registrations", "{code}/{side}");
        }
    }
}

#[test]
fn source_guard_complete_metadata_retains_absence_null_and_type_rules() {
    for code in ["JTL_119", "SEC_130"] {
        let source = entry(code);
        for key in ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp"] {
            let mut absent = source.clone(); absent["card"].as_object_mut().unwrap().remove(key);
            assert!(outcome(absent).is_ok(), "absent {code}/{key}");
            let mut null = source.clone(); null["card"][key] = Value::Null;
            assert!(outcome(null).is_ok(), "null {code}/{key}");
            for value in [json!(false), json!(0), json!(""), json!([]), json!({})] {
                let mut wrong = source.clone(); wrong["card"][key] = value;
                assert!(outcome(wrong).is_err(), "nonnull {code}/{key}");
            }
        }
        for key in ["piloting", "attachCondition"] {
            let mut absent = source.clone(); absent.as_object_mut().unwrap().remove(key);
            assert!(outcome(absent).is_ok(), "absent root {code}/{key}");
            let mut wrong = source.clone(); wrong[key] = json!({});
            assert!(outcome(wrong).is_err(), "non-null root {code}/{key}");
        }
        for value in [json!(true), json!(false)] {
            let mut valid = source.clone(); valid["card"]["unique"] = value;
            let allowed = code == "JTL_119" || valid["card"]["unique"] == false;
            assert_eq!(outcome(valid).is_ok(), allowed, "family uniqueness policy {code}");
        }
        for value in [Value::Null, json!("")] {
            let mut valid = source.clone(); valid["card"]["epicAction"] = value;
            assert!(outcome(valid).is_ok(), "empty epic {code}");
        }
        let mut no_epic = source.clone(); no_epic["card"].as_object_mut().unwrap().remove("epicAction");
        assert!(outcome(no_epic).is_ok(), "absent epic {code}");
        for value in [json!(false), json!(0), json!([]), json!({}), json!("extra")] {
            let mut wrong = source.clone(); wrong["card"]["epicAction"] = value;
            assert!(outcome(wrong).is_err(), "nonempty epic {code}");
        }
        let mut absent = source.clone(); absent["card"].as_object_mut().unwrap().remove("unique");
        assert!(outcome(absent).is_err(), "absent unique {code}");
        for value in [Value::Null, json!(0), json!("false"), json!([]), json!({})] {
            let mut wrong = source.clone(); wrong["card"]["unique"] = value;
            assert!(outcome(wrong).is_err(), "nonboolean unique {code}");
        }
    }
}

#[test]
fn source_guard_partial_families_and_raw_headers_keep_their_distinct_boundaries() {
    // Force continuation sources historically constrain pilot/attachment fields,
    // while their printed deployment/upgrade/epic metadata is a separate concern.
    let source = entry("LOF_041");
    for key in ["deployBox","deployCost","upgradePower","upgradeHp","epicAction","unique"] {
        let mut partial = source.clone(); partial["card"][key] = json!({});
        assert!(outcome(partial).is_ok(), "do not impose complete metadata on Force/{key}");
    }
    for key in ["pilotText","pilotBox","pilotingBox"] {
        let mut wrong = source.clone(); wrong["card"][key] = json!(false);
        assert_eq!(outcome(wrong).unwrap_err(), "Force continuation requires sole ordinary source registration without side mechanics");
    }
    let mut typed_header = entry("JTL_119");
    typed_header["abilities"]["card"][0].as_object_mut().unwrap().remove("costs");
    assert!(outcome(typed_header).is_ok());
    let mut raw_header = entry("SEC_130");
    raw_header["abilities"]["card"][0].as_object_mut().unwrap().remove("costs");
    assert_eq!(outcome(raw_header).unwrap_err(), "target-arena damage and resource hand groups require a sole complete ordinary event registration");
}
