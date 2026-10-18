//! Compiler-only contracts preserve source semantics without widening execution.
use kessel::ability_ir::{Catalogue, Effect, Instruction, Target, SCHEMA};
use serde_json::{json, Value};

#[test]
fn retained_source_contracts_preserve_execution_boundaries() {
    let fixtures: Vec<Value> = serde_json::from_str(include_str!("fixtures/source-only-contracts.json")).unwrap();
    for fixture in fixtures {
        let mut entry = fixture["compiled"].clone();
        entry["setCode"] = json!("PROBE_001");
        entry["card"] = json!({"types":["unit"],"traits":[],"aspects":[],"keywords":[],
            "hp":3,"power":2,"cost":2,"unique":false,"arena":"ground","text":"Source contract probe"});
        entry["keywordValues"] = json!({});
        // Establish that this complete ordinary source envelope itself admits.
        let mut control = entry.clone();
        control["abilities"] = json!({"leader":[],"unit":[],"card":[{"trigger":"whenPlayed",
            "title":"Draw","costs":[],"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}]});
        let control = Catalogue::load(&json!({"schema":SCHEMA,"cards":[control]})).unwrap();
        assert!(control.cards.contains_key("PROBE_001"), "{} control: {:?}", fixture["name"], control.rejected);
        if fixture["name"] == "live-constant" {
            // This sibling differs only by the unimplemented live condition;
            // its modifier and stricter ongoing source envelope already admit.
            let mut sibling = entry.clone();
            sibling["abilities"]["card"][0]["effect"]["properties"]
                .as_object_mut().unwrap().remove("condition");
            let sibling = Catalogue::load(&json!({"schema":SCHEMA,"cards":[sibling]})).unwrap();
            assert!(sibling.cards.contains_key("PROBE_001"), "constant sibling: {:?}", sibling.rejected);
        }
        let catalogue = Catalogue::load(&json!({"schema":SCHEMA,"cards":[entry.clone()]})).unwrap();
        if fixture["runtimeAdmission"].is_string() {
            // The checkpoint implements this exact selected-player scalar family;
            // its complete source remains intact and larger amounts stay held.
            let programs=&catalogue.cards["PROBE_001"];
            assert!(catalogue.programs[programs.card[0] as usize].instructions.iter().any(|i|
                matches!(i,Instruction::Apply(Effect::ReadyResources {target:Target::Name(n),..}) if n=="chosenPlayer")));
            entry["abilities"]["card"][0]["effect"]["properties"]["amount"]["value"]=json!(3);
            let rejected=Catalogue::load(&json!({"schema":SCHEMA,"cards":[entry]})).unwrap();
            assert!(!rejected.cards.contains_key("PROBE_001"));
            continue;
        }
        assert!(!catalogue.cards.contains_key("PROBE_001"), "{}", fixture["name"]);
        let reason = &catalogue.rejected["PROBE_001"];
        assert!(reason.contains(fixture["runtimeRejection"].as_str().unwrap()),
            "{}: {reason}", fixture["name"]);
    }
}
