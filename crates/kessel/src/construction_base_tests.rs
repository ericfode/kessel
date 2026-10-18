use super::*;

// Source-derived witnesses for bases whose printed text is a deck-construction
// rule. Pinned DataVault.ts / ThermalOscillator.ts are plain BaseCard subclasses
// whose source says "no implementation here, the 'implementation' is in
// DeckValidator"; the simulator starts from a caller-supplied deck, so the only
// in-game behaviour of these bases is a plain base with the printed HP.

fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn admitted(value: Value, code: &str) -> bool {
    Engine::from_catalogue_value(&value)
        .map(|e| e.by_code.contains_key(code))
        .unwrap_or(false)
}
fn entry<'a>(catalogue: &'a Value, code: &str) -> &'a Value {
    catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
}

#[test]
fn construction_only_bases_admit_with_plain_base_metadata_and_no_programs() {
    let e = engine();
    for (code, hp, text) in [
        (
            "JTL_024",
            33,
            "Your minimum deck size is increased by 10 cards.",
        ),
        (
            "JTL_025",
            27,
            "Your minimum deck size is decreased by 5 cards.",
        ),
    ] {
        let d = &e.definitions[e.by_code[code] as usize];
        assert!(d.info.is("base"));
        assert_eq!(d.info.hp, Some(hp));
        assert_eq!(d.info.text.as_deref(), Some(text));
        assert!(d.programs.card.is_empty() && d.programs.unit.is_empty() && d.programs.leader.is_empty());
        assert!(!d.info.unique);
    }
    // The committed raw entries match the reviewed construction family exactly.
    let c = catalogue();
    for code in ["JTL_024", "JTL_025"] {
        let raw = entry(&c, code);
        assert_eq!(raw["baseClass"], json!("BaseCard"));
        assert_eq!(raw["card"]["types"], json!(["base"]));
        assert_eq!(raw["card"]["unique"], json!(false));
        assert_eq!(raw["card"]["keywords"], json!([]));
        assert_eq!(raw["keywordValues"], json!({}));
        assert_eq!(raw["abilities"], json!({"leader":[],"unit":[],"card":[]}));
        assert_eq!(raw["piloting"], Value::Null);
        assert_eq!(raw["attachCondition"], Value::Null);
    }
}

#[test]
fn construction_only_bases_use_their_printed_hp_for_the_winner_threshold() {
    let e = engine();
    for (code, hp) in [("JTL_024", 33u16), ("JTL_025", 27u16)] {
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e);
            let base = add(&e, &mut s, seat, code);
            s.player_mut(seat).base = base;
            s.card_mut(base).damage = hp - 1;
            e.check_win(&mut s);
            assert_eq!(s.winner(), None, "{code} survives at {} damage", hp - 1);
            s.card_mut(base).damage = hp;
            e.check_win(&mut s);
            assert_eq!(s.winner(), Some(seat.other()), "{code} is lethal at {hp} damage");
        }
    }
}

#[test]
fn any_other_base_text_or_program_stays_an_explicit_rejection() {
    let c = catalogue();
    assert!(admitted(c.clone(), "JTL_024"));
    // Text outside the two reviewed construction sentences is not admitted.
    let mut bad = c.clone();
    let raw = bad["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "JTL_024")
        .unwrap();
    raw["card"]["text"] = json!("Your minimum deck size is 40 cards.");
    assert!(!admitted(bad, "JTL_024"));
    // A construction sentence beside a real registration is not admitted.
    let mut bad = c.clone();
    let raw = bad["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "JTL_024")
        .unwrap();
    raw["abilities"]["card"] = json!([{"trigger":"whenPlayed","title":"Draw a card","costs":[],"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}]);
    assert!(!admitted(bad, "JTL_024"));
    // A base whose text changes the game keeps the existing base-zone review.
    let mut bad = c.clone();
    let raw = bad["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "JTL_025")
        .unwrap();
    raw["card"]["text"] = json!("Each leader unit you control gets +1/+0.");
    assert!(!admitted(bad, "JTL_025"));
    // HP bounds still apply to the construction family.
    let mut bad = c.clone();
    let raw = bad["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "JTL_024")
        .unwrap();
    raw["card"]["hp"] = json!(0);
    assert!(!admitted(bad, "JTL_024"));
}
