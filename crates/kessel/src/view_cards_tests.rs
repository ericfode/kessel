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
fn look_at_with_then_or_select_stays_rejected() {
    let e = engine();
    for code in ["SEC_233", "SHD_202", "SOR_200"] {
        assert!(
            !e.by_code.contains_key(code),
            "{code} must stay rejected"
        );
        let reason = e.rejected_cards().unwrap()[code].clone();
        assert!(
            reason.contains("lookAt") || reason.contains("view-only"),
            "{code}: {reason}"
        );
    }
}
