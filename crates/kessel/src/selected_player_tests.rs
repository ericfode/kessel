use super::*;

#[test]
fn selected_player_cluster_admits_reviewed_ready_draw_discard_cards() {
    let e = engine();
    for code in ["SOR_171", "SHD_181", "ASH_189", "SHD_199", "HMW_209"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn mission_briefing_chooses_player_and_draws_two() {
    let e = engine();
    let mut s = board(&e);
    for _ in 0..3 {
        let id = add(&e, &mut s, Seat::P2, "LOF_256");
        s.players[1].deck.push(id).unwrap();
    }
    let before = s.players[1].hand.len;
    let event = hand(&e, &mut s, "SOR_171");
    let choice = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert_eq!(choice.decision, Decision::AbilityTarget);
    let actions = e.legal_actions(&choice);
    let labels: Vec<_> = actions.iter().map(|a| a.label.as_str()).collect();
    assert!(labels.contains(&"You"));
    assert!(labels.contains(&"Opponent"));
    let done = e.apply(&choice, &Action::button("Opponent", 1)).unwrap();
    assert_eq!(done.players[1].hand.len, before + 2);
    assert!(done.players[0].discard.contains(event));
}

#[test]
fn pillage_forces_chosen_player_to_discard_two() {
    let e = engine();
    let mut s = board(&e);
    for _ in 0..3 {
        let id = add(&e, &mut s, Seat::P2, "LOF_256");
        s.players[1].hand.push(id).unwrap();
    }
    let before = s.players[1].hand.len;
    let event = hand(&e, &mut s, "SHD_181");
    let choice = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert_eq!(choice.decision, Decision::AbilityTarget);
    let picking = e.apply(&choice, &Action::button("Opponent", 1)).unwrap();
    assert_eq!(picking.decision, Decision::AbilityDiscard);
    assert_eq!(picking.event_group.discard_count, 2);
    assert_eq!(picking.active, Seat::P2);
    let first = picking.players[1].hand.as_slice()[0];
    let second = picking.players[1].hand.as_slice()[1];
    let mid = e.apply(&picking, &e.card_action(&picking, first)).unwrap();
    let after = e.apply(&mid, &e.card_action(&mid, second)).unwrap();
    let done = if after.decision == Decision::AbilityDiscard {
        e.apply(&after, &Action::button("Done", "done")).unwrap()
    } else {
        after
    };
    assert_eq!(done.players[1].hand.len, before - 2);
    assert!(done.players[1].discard.contains(first));
    assert!(done.players[1].discard.contains(second));
}

#[test]
fn emperors_messenger_on_attack_readies_chosen_players_resource() {
    let e = engine();
    let mut s = board(&e);
    for seat in [Seat::P1, Seat::P2] {
        for _ in 0..3 {
            let id = add(&e, &mut s, seat, "LOF_254");
            s.player_mut(seat).resources.push(id).unwrap();
            s.card_mut(id).exhausted = true;
        }
    }
    let source = unit(&e, &mut s, Seat::P1, "ASH_189");
    s.card_mut(source).exhausted = false;
    let attack = e.apply(&s, &e.card_action(&s, source)).unwrap();
    let declared = e
        .apply(&attack, &e.card_action(&attack, attack.players[1].base))
        .unwrap();
    let mut prompt = declared;
    for _ in 0..12 {
        if prompt.decision == Decision::AbilityTarget
            && e.legal_actions(&prompt).iter().any(|a| a.label == "Opponent")
        {
            break;
        }
        let actions = e.legal_actions(&prompt);
        let next = actions
            .iter()
            .find(|a| ["Trigger", "You", "Opponent"].contains(&a.label.as_str()) || a.kind == "card")
            .cloned()
            .or_else(|| actions.first().cloned())
            .expect("progress toward player selection");
        prompt = e.apply(&prompt, &next).unwrap();
    }
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let before = e.ready_resources(&prompt, Seat::P2);
    let done = e.apply(&prompt, &Action::button("Opponent", 1)).unwrap();
    assert_eq!(e.ready_resources(&done, Seat::P2), before + 1);
}

#[test]
fn selected_player_system_lowers_only_reviewed_family() {
    let ready = crate::selected_player::lower_fields(
        "readyResources",
        &json!({"kind":"ref","path":["selectedPlayer"]}),
        &crate::selected_player::SelectedPlayerProperties {
            amount: Some(crate::ability_ir::Expr::number(1)),
        },
        "pinnedSystem",
        &json!({"kind":"ref","path":["selectedPlayer"]}),
    )
    .unwrap();
    assert!(matches!(
        ready,
        Effect::ReadyResources { target: Target::Name(n), .. } if n == "chosenPlayer"
    ));
    assert!(crate::selected_player::lower_fields(
        "indirectDamageToPlayer",
        &json!({"kind":"ref","path":["selectedPlayer"]}),
        &crate::selected_player::SelectedPlayerProperties::default(),
        "pinnedSystem",
        &json!({"kind":"ref","path":["selectedPlayer"]}),
    )
    .is_err());
}

#[test]
fn player_selection_rejects_multiple_players_mode() {
    assert!(
        serde_json::from_value::<crate::ability_ir::TargetResolver>(json!({
            "kind":"playerSelection",
            "mode":"multiplePlayers",
            "chooser":"self",
            "resultBinding":"selectedPlayer",
            "defaults":"pinnedTargetResolver",
            "contextBinding":"target",
            "choosingPlayerOverride":"contextOverrideFirst",
            "delayOpponentBeforeCosts":true
        }))
        .is_err()
    );
}
