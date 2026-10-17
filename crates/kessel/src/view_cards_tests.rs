use super::*;

#[test]
fn look_at_cluster_admits_viper_probe_and_imperial_defector() {
    let e = engine();
    for code in ["SEC_239", "ASH_250"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    // SOR reprint shares the SEC_239 programme (catalogue aliasOf).
    assert!(
        e.by_code.contains_key("SOR_228"),
        "SOR_228 should resolve via reprint alias of SEC_239"
    );
}

#[test]
fn look_at_select_cluster_admits_spark_bodhi_jam_and_tip() {
    let e = engine();
    for code in ["SOR_200", "SOR_201", "JTL_207", "LOF_226"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn viper_probe_droid_looks_at_opponent_hand_then_done() {
    let e = engine();
    let mut s = board(&e);
    for _ in 0..3 {
        let id = add(&e, &mut s, Seat::P2, "LOF_256");
        s.players[1].hand.push(id).unwrap();
    }
    let source = hand(&e, &mut s, "SEC_239");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(prompt.players[0].ground.contains(source));
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    assert_eq!(prompt.active, Seat::P1);
    assert_eq!(prompt.ability_result, 3);
    assert_eq!(
        e.legal_actions(&prompt),
        vec![Action::button("Done", "done")]
    );
    let done = e
        .apply(&prompt, &Action::button("Done", "done"))
        .unwrap();
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(done.active, Seat::P2);
    assert_eq!(done.players[1].hand.len, 3);
}

#[test]
fn viper_probe_droid_empty_opponent_hand_skips_display() {
    let e = engine();
    let mut s = board(&e);
    let source = hand(&e, &mut s, "SEC_239");
    let done = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(done.players[0].ground.contains(source));
    assert_ne!(done.decision, Decision::AbilityDisplay);
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(done.active, Seat::P2);
}

#[test]
fn imperial_defector_looks_at_opponent_hand() {
    let e = engine();
    let mut s = board(&e);
    let card = add(&e, &mut s, Seat::P2, "LOF_254");
    s.players[1].hand.push(card).unwrap();
    let source = hand(&e, &mut s, "ASH_250");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    assert_eq!(prompt.ability_result, 1);
    let done = e
        .apply(&prompt, &Action::button("Done", "done"))
        .unwrap();
    assert_eq!(done.decision, Decision::Window);
    assert!(done.players[1].hand.contains(card));
}

#[test]
fn spark_of_rebellion_discards_selected_opponent_hand_card() {
    let e = engine();
    let mut s = board(&e);
    let unit = add(&e, &mut s, Seat::P2, "LOF_256");
    let event = add(&e, &mut s, Seat::P2, "ASH_115");
    s.players[1].hand.push(unit).unwrap();
    s.players[1].hand.push(event).unwrap();
    let source = hand(&e, &mut s, "SOR_200");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(prompt.players[0].discard.contains(source));
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    assert_eq!(prompt.active, Seat::P1);
    let actions = e.legal_actions(&prompt);
    assert!(actions.contains(&e.card_action(&prompt, unit)));
    assert!(actions.contains(&e.card_action(&prompt, event)));
    assert!(!actions.iter().any(|a| a.label == "Done"));
    let done = e.apply(&prompt, &e.card_action(&prompt, event)).unwrap();
    assert!(done.players[1].discard.contains(event));
    assert!(done.players[1].hand.contains(unit));
    assert!(!done.players[1].hand.contains(event));
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn spark_of_rebellion_empty_hand_is_play_anyway() {
    let e = engine();
    let mut s = board(&e);
    let source = hand(&e, &mut s, "SOR_200");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityNoEffect);
    assert!(prompt.players[0].hand.contains(source));
}

#[test]
fn bodhi_rook_discards_only_non_unit_from_opponent_hand() {
    let e = engine();
    let mut s = board(&e);
    let unit = add(&e, &mut s, Seat::P2, "LOF_256");
    let event = add(&e, &mut s, Seat::P2, "ASH_115");
    s.players[1].hand.push(unit).unwrap();
    s.players[1].hand.push(event).unwrap();
    let source = hand(&e, &mut s, "SOR_201");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(prompt.players[0].ground.contains(source));
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    let actions = e.legal_actions(&prompt);
    assert!(actions.contains(&e.card_action(&prompt, event)));
    assert!(!actions.contains(&e.card_action(&prompt, unit)));
    assert!(!actions.iter().any(|a| a.label == "Done"));
    let done = e.apply(&prompt, &e.card_action(&prompt, event)).unwrap();
    assert!(done.players[1].discard.contains(event));
    assert!(done.players[1].hand.contains(unit));
}

#[test]
fn bodhi_rook_unit_only_hand_acknowledges_with_done() {
    let e = engine();
    let mut s = board(&e);
    let unit = add(&e, &mut s, Seat::P2, "LOF_256");
    s.players[1].hand.push(unit).unwrap();
    let source = hand(&e, &mut s, "SOR_201");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    assert_eq!(
        e.legal_actions(&prompt),
        vec![Action::button("Done", "done")]
    );
    let done = e
        .apply(&prompt, &Action::button("Done", "done"))
        .unwrap();
    assert!(done.players[1].hand.contains(unit));
    assert!(!done.players[1].discard.contains(unit));
}

#[test]
fn jam_communications_discards_only_events() {
    let e = engine();
    let mut s = board(&e);
    let unit = add(&e, &mut s, Seat::P2, "LOF_256");
    let event = add(&e, &mut s, Seat::P2, "ASH_115");
    s.players[1].hand.push(unit).unwrap();
    s.players[1].hand.push(event).unwrap();
    let source = hand(&e, &mut s, "JTL_207");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    let actions = e.legal_actions(&prompt);
    assert!(actions.contains(&e.card_action(&prompt, event)));
    assert!(!actions.contains(&e.card_action(&prompt, unit)));
    let done = e.apply(&prompt, &e.card_action(&prompt, event)).unwrap();
    assert!(done.players[1].discard.contains(event));
}

#[test]
fn look_at_select_optional_discard_draw_admits_remnant_and_bazine() {
    let e = engine();
    for code in ["ASH_220", "SHD_184"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn remnant_lookouts_may_discard_then_opponent_draws() {
    let e = engine();
    let mut s = board(&e);
    let card = add(&e, &mut s, Seat::P2, "LOF_256");
    s.players[1].hand.push(card).unwrap();
    let deck_card = add(&e, &mut s, Seat::P2, "LOF_254");
    s.players[1].deck.push(deck_card).unwrap();
    let hand_before = s.players[1].hand.len;
    let deck_before = s.players[1].deck.len;
    let source = hand(&e, &mut s, "ASH_220");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(prompt.players[0].ground.contains(source));
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    let actions = e.legal_actions(&prompt);
    assert!(actions.contains(&e.card_action(&prompt, card)));
    assert!(actions.contains(&Action::button("Done", "done")));
    let done = e.apply(&prompt, &e.card_action(&prompt, card)).unwrap();
    assert!(done.players[1].discard.contains(card));
    assert!(!done.players[1].hand.contains(card));
    assert_eq!(done.players[1].hand.len, hand_before); // discarded 1, drew 1
    assert_eq!(done.players[1].deck.len, deck_before - 1);
    assert!(done.players[1].hand.contains(deck_card));
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn remnant_lookouts_done_looks_without_discard() {
    let e = engine();
    let mut s = board(&e);
    let card = add(&e, &mut s, Seat::P2, "LOF_256");
    s.players[1].hand.push(card).unwrap();
    let deck_card = add(&e, &mut s, Seat::P2, "LOF_254");
    s.players[1].deck.push(deck_card).unwrap();
    let source = hand(&e, &mut s, "ASH_220");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    let done = e
        .apply(&prompt, &Action::button("Done", "done"))
        .unwrap();
    assert!(done.players[1].hand.contains(card));
    assert!(!done.players[1].discard.contains(card));
    assert!(done.players[1].deck.contains(deck_card));
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn bazine_netal_optional_discard_draws_for_opponent() {
    let e = engine();
    let mut s = board(&e);
    let card = add(&e, &mut s, Seat::P2, "ASH_115");
    s.players[1].hand.push(card).unwrap();
    let deck_card = add(&e, &mut s, Seat::P2, "LOF_254");
    s.players[1].deck.push(deck_card).unwrap();
    let source = hand(&e, &mut s, "SHD_184");
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    let done = e.apply(&prompt, &e.card_action(&prompt, card)).unwrap();
    assert!(done.players[1].discard.contains(card));
    assert!(done.players[1].hand.contains(deck_card));
}

#[test]
fn look_at_select_wrapped_and_play_shapes_stay_rejected() {
    let e = engine();
    // Mother Talzin: ifYouDo play-from-discard. Unmasking: ifYouDo wrapper.
    // Tear This Ship: playCardSystem immediate. Beguile: then returnToHand.
    for code in ["TS26_026", "TWI_223", "LAW_066", "SEC_233"] {
        assert!(
            !e.by_code.contains_key(code),
            "{code} must stay rejected"
        );
        let reason = e.rejected_cards().unwrap()[code].clone();
        assert!(
            reason.contains("lookAt") || reason.contains("view-only") || reason.contains("invalid"),
            "{code}: {reason}"
        );
    }
}
