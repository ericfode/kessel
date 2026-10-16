use super::*;
use crate::ability_ir::Catalogue;
use serde_json::json;

fn catalogue() -> Value {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["setCode"] == code)
        .unwrap()
        .clone()
}
fn search(value: &Value) -> Option<Value> {
    if value["kind"] == "deckSearch" {
        return Some(value.clone());
    }
    match value {
        Value::Object(o) => o.values().find_map(search),
        Value::Array(a) => a.iter().find_map(search),
        _ => None,
    }
}
fn recruit() -> Value {
    search(&entry("SOR_123")["abilities"]).unwrap()
}

#[test]
fn deck_search_source_contracts_preserve_mandatory_randomness() {
    // These are native-helper/source contract tests, not executable card fixtures.
    // Full sources reviewed: Recruit, RemoteScout, RemnantReserves,
    // TrustYourself and SearchYourFeelings, together with the native systems.
    for (code, count, selected) in [
        ("SOR_123", 5, 1),
        ("HMW_085", 8, 1),
        ("SHD_093", 5, 3),
        ("HMW_101", 3, 1),
    ] {
        let raw = search(&entry(code)["abilities"]).unwrap();
        let parsed: Search = serde_json::from_value(raw).unwrap();
        let Search::DeckSearch {
            search_count,
            select_count,
            ..
        } = &parsed;
        assert_eq!(search_count.as_ref().unwrap().value(), count);
        assert_eq!(select_count.value(), selected);
        assert!(parsed.blocker().unwrap().contains("random order"), "{code}");
    }
    let raw = search(&entry("SOR_042")["abilities"]).unwrap();
    let parsed: Search = serde_json::from_value(raw).unwrap();
    assert!(parsed.blocker().unwrap().contains("always shuffles"));
}

#[test]
fn deck_search_take_nothing_and_small_deck_do_not_remove_whole_card_requirement() {
    let mut raw = recruit();
    raw["canChooseFewer"] = json!({"kind":"const","value":false});
    // Native `canChooseFewer || true` still offers take-nothing. Even selecting
    // all eligible cards cannot make every possible search completion deterministic.
    raw["selectCount"]["value"] = json!(5);
    let parsed: Search = serde_json::from_value(raw.clone()).unwrap();
    assert!(parsed.blocker().unwrap().contains("including take-nothing"));
    // A state with no deck, no matches, or a one-card deck does not authorize
    // admitting this card in its other legal states. No State-dependent bypass.
    assert!(reject(&raw).unwrap_err().contains("chance semantics"));
    raw["searchCount"]["value"] = json!(1);
    assert!(reject(&raw)
        .unwrap_err()
        .contains("continuations are unimplemented"));
    raw["shuffleWhenDone"]["value"] = json!(true);
    assert!(reject(&raw).unwrap_err().contains("final shuffle"));
}

#[test]
fn deck_search_unknown_fields_and_dynamic_contexts_stay_rejected() {
    let raw = recruit();
    for key in [
        "selectedCardsHandler",
        "remainingCardsHandler",
        "remainingCardsImmediateEffect",
        "chooseNothingImmediateEffect",
        "choosingPlayer",
        "unknown",
    ] {
        let mut changed = raw.clone();
        changed[key] = json!({"kind":"noAction"});
        assert!(
            serde_json::from_value::<Search>(changed.clone()).is_err(),
            "{key}"
        );
        assert!(reject(&changed).is_err());
    }
    for key in [
        "searchCount",
        "selectCount",
        "searchWholeDeck",
        "shuffleWhenDone",
    ] {
        let mut changed = raw.clone();
        changed[key] = json!({"kind":"ref","path":["privateCapturedValue"]});
        assert!(serde_json::from_value::<Search>(changed).is_err(), "{key}");
    }
    for key in ["switch", "shuffle", "changePlayer"] {
        let mut changed = search(&entry("HMW_101")["abilities"]).unwrap();
        changed["selectedCardsImmediateEffect"][key] = json!(true);
        let parsed: Search = serde_json::from_value(changed).unwrap();
        assert!(parsed.blocker().unwrap_err().contains("cannot erase"));
    }
    let mut changed = raw.clone();
    changed["cardCondition"]["value"]["receiver"]["path"] = json!(["source"]);
    let parsed: Search = serde_json::from_value(changed).unwrap();
    assert!(parsed
        .blocker()
        .unwrap_err()
        .contains("physical search card"));
    let mut changed = raw;
    changed["cardCondition"]["value"]["args"] = json!([{"kind":"const","value":"droid"}]);
    let parsed: Search = serde_json::from_value(changed).unwrap();
    assert!(parsed
        .blocker()
        .unwrap_err()
        .contains("exact literal argument"));
}

#[test]
fn deck_search_native_markers_bounds_and_completion_order_are_not_erased() {
    let raw = recruit();
    for n in [0, 65, 255] {
        let mut changed = raw.clone();
        changed["searchCount"]["value"] = json!(n);
        let parsed: Search = serde_json::from_value(changed).unwrap();
        assert!(parsed.blocker().is_err(), "count {n}");
    }
    for (key, value) in [
        ("wholeDeckAlwaysShuffles", json!(false)),
        (
            "multiSelectCondition",
            json!({"kind":"const","value":false}),
        ),
        (
            "completionOrder",
            json!(["selected", "remaining", "shuffle"]),
        ),
        ("searchWholeDeck", json!({"kind":"const","value":true})),
        ("targetMode", json!("unlimited")),
    ] {
        let mut changed = raw.clone();
        changed[key] = value;
        let parsed: Search = serde_json::from_value(changed).unwrap();
        assert!(parsed.blocker().is_err(), "{key}");
    }
    let mut changed = raw;
    changed["remainingCardsDefault"] = json!("keepOriginalOrder");
    assert!(serde_json::from_value::<Search>(changed).is_err());
}

#[test]
fn deck_search_entire_card_stays_rejected_including_nested_and_raw_metadata_mutations() {
    for code in ["SOR_123", "HMW_085", "SHD_093", "HMW_101", "SOR_042"] {
        for variant in 0..5 {
            let mut c = entry(code);
            match variant {
                1 => c["card"]["types"] = json!(["unit", "leader"]),
                2 => c["baseClass"] = json!("TokenUnitCard"),
                3 => c["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":7}}})),
                4 => c["piloting"] = json!({"kind":"unimplemented"}),
                _ => {},
            }
            let one = json!({"schema":"swu-card-ir/v1","cards":[c]});
            let loaded = Catalogue::load(&one).unwrap();
            assert!(!loaded.cards.contains_key(code), "{code}/{variant}");
            assert!(loaded.rejected.contains_key(code));
        }
    }
    // Unknown wrappers never make the nested known search executable.
    assert!(reject(&json!({"kind":"privateCallback","child":recruit()})).is_err());
    // Ordinary unaffected programs continue to the normal admission machinery.
    assert!(reject(
        &json!({"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}})
    )
    .is_ok());
}

#[test]
fn deck_search_execution_boundaries_keep_completion_precedence_and_reject_every_scope() {
    let mut raw = recruit();
    for (count, final_shuffle, boundary) in [
        (1, false, "deck-search display/selection/completion continuations are unimplemented"),
        (2, false, "deck search bottoms unchosen cards in random order, including take-nothing; exact RNG snapshot or chance semantics are unimplemented"),
        (1, true, "deck search requests a final shuffle; exact RNG snapshot or chance semantics are unimplemented"),
        (2, true, "deck search requests a final shuffle; exact RNG snapshot or chance semantics are unimplemented"),
    ] {
        raw["searchCount"]["value"] = json!(count);
        raw["shuffleWhenDone"]["value"] = json!(final_shuffle);
        let parsed: Search = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(parsed.blocker().unwrap(), boundary);
        assert_eq!(reject(&raw).unwrap_err(), format!("unimplemented deck-search contract: {}", boundary));
    }
    raw["searchWholeDeck"]["value"] = json!(true);
    raw.as_object_mut().unwrap().remove("searchCount");
    let parsed: Search = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(parsed.blocker().unwrap(), "whole-deck search always shuffles; exact RNG snapshot or chance semantics are unimplemented");
    assert_eq!(reject(&raw).unwrap_err(), format!("unimplemented deck-search contract: {}", "whole-deck search always shuffles; exact RNG snapshot or chance semantics are unimplemented"));
}
