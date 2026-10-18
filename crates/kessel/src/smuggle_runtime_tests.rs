use super::*;

fn fill_resources(e: &Engine, s: &mut State, seat: Seat, n: usize) {
    while (s.player(seat).resources.len as usize) < n {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}

fn resource_card(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).resources.push(id).unwrap();
    id
}

#[test]
fn smuggle_only_cards_admit_and_plot_stays_rejected() {
    let e = engine();
    for code in [
        "SHD_032", "SHD_050", "SHD_052", "SHD_075", "SHD_086", "SHD_097", "SHD_160", "SHD_201",
        "SHD_215", "SHD_252",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        assert!(e.definitions[*e.by_code.get(code).unwrap() as usize]
            .smuggle
            .is_some());
    }
    for code in [
        "SEC_034", "SEC_084", "SEC_099", "SEC_111", "SEC_140", "SEC_152", "SEC_172", "SEC_189",
        "SEC_203", "SEC_243", "SEC_255",
    ] {
        let reason = e.rejected_cards().unwrap()[code].clone();
        assert!(reason.contains("plot"), "{code}: {reason}");
    }
}

#[test]
fn smuggle_unit_pays_alternate_cost_and_replaces_from_deck() {
    let e = engine();
    let mut s = board(&e);
    fill_resources(&e, &mut s, Seat::P1, 6);
    let unit = resource_card(&e, &mut s, Seat::P1, "SHD_160");
    let top = add(&e, &mut s, Seat::P1, "LOF_256");
    let tail = add(&e, &mut s, Seat::P1, "LOF_254");
    s.players[0].deck.push(top).unwrap();
    s.players[0].deck.push(tail).unwrap();
    let before = s.players[0].resources.len;
    let cost = e.smuggle_cost(&s, unit).unwrap();
    assert_eq!(cost, 3);
    assert!(e.legal_actions(&s).contains(&e.card_action(&s, unit)));
    let done = e.apply(&s, &e.card_action(&s, unit)).unwrap();
    assert!(done.players[0].ground.contains(unit));
    assert!(done.card(unit).exhausted);
    assert!(!done.players[0].resources.contains(unit));
    assert!(done.players[0].resources.contains(top));
    assert!(done.card(top).exhausted);
    assert_eq!(done.players[0].deck.as_slice(), [tail]);
    assert_eq!(done.players[0].resources.len, before);
    assert_eq!(
        done.players[0]
            .resources
            .as_slice()
            .iter()
            .filter(|&&id| done.card(id).exhausted)
            .count(),
        cost as usize
    );
}

#[test]
fn smuggle_empty_deck_does_not_replace_and_reduces_resource_count() {
    let e = engine();
    let mut s = board(&e);
    fill_resources(&e, &mut s, Seat::P1, 5);
    let unit = resource_card(&e, &mut s, Seat::P1, "SHD_160");
    let before = s.players[0].resources.len;
    let done = e.apply(&s, &e.card_action(&s, unit)).unwrap();
    assert!(done.players[0].ground.contains(unit));
    assert_eq!(done.players[0].resources.len, before - 1);
    assert_eq!(done.players[0].deck.len, 0);
}

#[test]
fn smuggle_priority_exhaust_uses_ready_smuggled_card_in_payment() {
    let e = engine();
    let mut s = board(&e);
    fill_resources(&e, &mut s, Seat::P1, 4);
    let other = s.players[0].resources.as_slice()[0];
    s.card_mut(other).exhausted = true;
    let unit = resource_card(&e, &mut s, Seat::P1, "SHD_160");
    assert!(!s.card(unit).exhausted);
    let done = e.apply(&s, &e.card_action(&s, unit)).unwrap();
    // Pre-exhausted other + 3 payment exhausts (unit counted then removed) leave
    // 3 exhausted resources after replacement from an empty deck would be 3 among
    // remaining; with no deck top, remaining exhausted count is 3 (other + 2 paid).
    assert_eq!(
        done.players[0]
            .resources
            .as_slice()
            .iter()
            .filter(|&&id| done.card(id).exhausted)
            .count(),
        3
    );
    assert!(done.card(other).exhausted);
}

#[test]
fn smuggle_rejects_hand_play_from_resources_without_keyword() {
    let e = engine();
    let mut s = board(&e);
    fill_resources(&e, &mut s, Seat::P1, 6);
    let plain = resource_card(&e, &mut s, Seat::P1, "LOF_256");
    assert!(!e.legal_actions(&s).contains(&e.card_action(&s, plain)));
}

#[test]
fn smuggle_event_commits_alternate_cost_then_resolves() {
    let e = engine();
    let mut s = board(&e);
    s.card_mut(s.players[0].base).damage = 5;
    fill_resources(&e, &mut s, Seat::P1, 6);
    let event = resource_card(&e, &mut s, Seat::P1, "SHD_252");
    let top = add(&e, &mut s, Seat::P1, "LOF_256");
    s.players[0].deck.push(top).unwrap();
    let cost = e.smuggle_cost(&s, event).unwrap();
    assert_eq!(cost, 3);
    let done = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert!(done.players[0].discard.contains(event));
    assert!(!done.players[0].resources.contains(event));
    assert!(done.players[0].resources.contains(top));
    assert_eq!(done.card(done.players[0].base).damage, 2);
    assert_eq!(
        done.players[0]
            .resources
            .as_slice()
            .iter()
            .filter(|&&id| done.card(id).exhausted)
            .count(),
        cost as usize
    );
}

#[test]
fn smuggle_aspect_penalty_uses_smuggle_aspects_not_play_aspects() {
    let e = engine();
    let mut s = board(&e);
    // Sabine (aggression/heroism) + vigilance base: Chewbacca play aspects are
    // heroism/vigilance, but Smuggle is Aggression Heroism — covered, cost 9.
    fill_resources(&e, &mut s, Seat::P1, 12);
    let chewie = resource_card(&e, &mut s, Seat::P1, "SHD_050");
    assert_eq!(e.cost(&s, chewie), 8); // play cost with vigilance base covering vigilance
    assert_eq!(e.smuggle_cost(&s, chewie), Some(9));
    // Starfighter Smuggle is Cunning-only; neither leader nor base covers it.
    let ship = resource_card(&e, &mut s, Seat::P1, "SHD_215");
    assert_eq!(e.smuggle_cost(&s, ship), Some(6));
}

#[test]
fn smuggle_additional_cost_text_fails_admission() {
    let original: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let mut c = original.clone();
    let entry = c["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["setCode"] == "SHD_160")
        .unwrap();
    entry["card"]["text"] =
        json!("Smuggle [7 resources Vigilance Villainy, deal 4 damage to a friendly unit]");
    let e = Engine::from_catalogue_value(&c).unwrap();
    assert!(!e.by_code.contains_key("SHD_160"));
    assert!(e.rejected_cards().unwrap()["SHD_160"].contains("additional costs"));
}

#[test]
fn smuggle_when_played_hooks_admit_privateer_and_cassian() {
    let e = engine();
    for code in ["SHD_113", "SHD_148"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
        let def = &e.definitions[*e.by_code.get(code).unwrap() as usize];
        assert!(def.smuggle.is_some());
        assert!(
            def.programs
                .card
                .iter()
                .any(|&p| e.program(p).trigger == "whenPlayedUsingSmuggle"),
            "{code} missing smuggle when-played program"
        );
    }
}

#[test]
fn smuggle_privateer_gives_three_experience_only_from_resources() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e);
        fill_resources(&e, &mut s, Seat::P1, 8);
        let unit = resource_card(&e, &mut s, Seat::P1, "SHD_113");
        let done = e.apply(&s, &e.card_action(&s, unit)).unwrap();
        assert!(done.players[0].ground.contains(unit));
        assert_eq!(e.token_count(&done, done.card(unit).experience), 3);
        // Hand play does not fire WhenPlayedUsingSmuggle.
        let mut s = board(&e);
        fill_resources(&e, &mut s, Seat::P1, 8);
        let hand = hand(&e, &mut s, "SHD_113");
        let done = e.apply(&s, &e.card_action(&s, hand)).unwrap();
        assert!(done.players[0].ground.contains(hand));
        assert_eq!(done.card(hand).experience, 0);
    }
}

#[test]
fn smuggle_cassian_enters_ready_only_from_resources() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e);
        fill_resources(&e, &mut s, Seat::P1, 8);
        let unit = resource_card(&e, &mut s, Seat::P1, "SHD_148");
        let done = e.apply(&s, &e.card_action(&s, unit)).unwrap();
        assert!(done.players[0].ground.contains(unit));
        assert!(!done.card(unit).exhausted);
        let mut s = board(&e);
        fill_resources(&e, &mut s, Seat::P1, 8);
        let hand = hand(&e, &mut s, "SHD_148");
        let done = e.apply(&s, &e.card_action(&s, hand)).unwrap();
        assert!(done.players[0].ground.contains(hand));
        assert!(done.card(hand).exhausted);
    }
}

#[test]
fn hook_unsupported_shapes_reject_with_event_name() {
    let e = engine();
    // HMW_115 / LOF_142 / SHD_255 are reviewed OnCardPlayed; ASH_204 stays onDamageDealt.
    assert!(
        e.by_code.contains_key("HMW_115"),
        "{:?}",
        e.rejected_cards().unwrap().get("HMW_115")
    );
    assert!(
        e.by_code.contains_key("LOF_142"),
        "{:?}",
        e.rejected_cards().unwrap().get("LOF_142")
    );
    assert!(
        e.by_code.contains_key("SHD_255"),
        "{:?}",
        e.rejected_cards().unwrap().get("SHD_255")
    );
    let reason = e.rejected_cards().unwrap()["ASH_204"].clone();
    assert!(
        reason.contains("unimplemented event hook onDamageDealt"),
        "{reason}"
    );
}
