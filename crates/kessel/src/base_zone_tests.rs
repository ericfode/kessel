//! Base-zone ongoing leader auras and reviewed base epic actions.
use super::*;

#[test]
fn krennic_damaged_unit_aura_applies_from_base_and_deployed() {
    let e = engine();
    assert!(e.by_code.contains_key("SOR_001"), "{:?}", e.rejected_cards().unwrap().get("SOR_001"));
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        let leader = s.player(seat).leader;
        s.card_mut(leader).def = e.by_code["SOR_001"];
        s.active = seat;
        s.action_player = Some(seat);
        let healthy = unit(&e, &mut s, seat, "ASH_034");
        let damaged = unit(&e, &mut s, seat, "ASH_034");
        s.card_mut(damaged).damage = 1;
        let enemy = unit(&e, &mut s, seat.other(), "ASH_034");
        s.card_mut(enemy).damage = 1;
        assert_eq!(e.power(&s, healthy, false), e.def(&s, healthy).info.power.unwrap_or(0) as u16);
        assert_eq!(
            e.power(&s, damaged, false),
            e.def(&s, damaged).info.power.unwrap_or(0) as u16 + 1
        );
        assert_eq!(e.power(&s, enemy, false), e.def(&s, enemy).info.power.unwrap_or(0) as u16);
        s.card_mut(leader).deployed = true;
        s.card_mut(leader).epic_used = true;
        e.add_arena(&mut s, leader).unwrap();
        assert_eq!(
            e.power(&s, damaged, false),
            e.def(&s, damaged).info.power.unwrap_or(0) as u16 + 1
        );
        assert_eq!(e.power(&s, healthy, false), e.def(&s, healthy).info.power.unwrap_or(0) as u16);
    }
}

#[test]
fn security_complex_epic_gives_shield_and_spends() {
    let e = engine();
    assert!(e.by_code.contains_key("SOR_019"), "{:?}", e.rejected_cards().unwrap().get("SOR_019"));
    let mut s = board(&e);
    let base = s.players[0].base;
    s.card_mut(base).def = e.by_code["SOR_019"];
    let target = unit(&e, &mut s, Seat::P2, "ASH_034");
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    assert!(e.legal_actions(&s).iter().any(|a| a.id() == Some(base)));
    let choose = e.apply(&s, &e.card_action(&s, base)).unwrap();
    assert_eq!(choose.decision, Decision::AbilityTarget);
    assert!(!choose.card(base).epic_used);
    let done = e.apply(&choose, &e.card_action(&choose, target)).unwrap();
    assert!(done.card(base).epic_used);
    assert_ne!(done.card(target).shield, 0);
    assert!(!e.legal_actions(&done).iter().any(|a| a.id() == Some(base)));
}

#[test]
fn tarkintown_epic_damages_only_damaged_non_leaders() {
    let e = engine();
    assert!(e.by_code.contains_key("SOR_025"), "{:?}", e.rejected_cards().unwrap().get("SOR_025"));
    let mut s = board(&e);
    let base = s.players[0].base;
    s.card_mut(base).def = e.by_code["SOR_025"];
    let healthy = unit(&e, &mut s, Seat::P2, "ASH_131");
    let damaged = unit(&e, &mut s, Seat::P2, "ASH_131");
    s.card_mut(damaged).damage = 1;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let choose = e.apply(&s, &e.card_action(&s, base)).unwrap();
    assert_eq!(choose.decision, Decision::AbilityTarget);
    let actions = e.legal_actions(&choose);
    assert!(actions.iter().any(|a| a.id() == Some(damaged)));
    assert!(!actions.iter().any(|a| a.id() == Some(healthy)));
    let done = e.apply(&choose, &e.card_action(&choose, damaged)).unwrap();
    assert_eq!(done.card(damaged).damage, 4);
    assert!(e.in_arena(&done, damaged));
    assert!(done.card(base).epic_used);
}

#[test]
fn jedha_city_epic_applies_phase_power_penalty() {
    let e = engine();
    assert!(e.by_code.contains_key("SOR_028"), "{:?}", e.rejected_cards().unwrap().get("SOR_028"));
    let mut s = board(&e);
    let base = s.players[0].base;
    s.card_mut(base).def = e.by_code["SOR_028"];
    let target = unit(&e, &mut s, Seat::P2, "ASH_034");
    let printed = e.def(&s, target).info.power.unwrap_or(0) as u16;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let choose = e.apply(&s, &e.card_action(&s, base)).unwrap();
    let done = e.apply(&choose, &e.card_action(&choose, target)).unwrap();
    assert_eq!(done.card(target).phase_power, -4);
    assert_eq!(e.power(&done, target, false), printed.saturating_sub(4));
    assert!(done.card(base).epic_used);
}

#[test]
fn epic_action_cancel_before_commit_does_not_spend() {
    let e = engine();
    let mut s = board(&e);
    let base = s.players[0].base;
    s.card_mut(base).def = e.by_code["SOR_019"];
    let _target = unit(&e, &mut s, Seat::P1, "ASH_034");
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let choose = e.apply(&s, &e.card_action(&s, base)).unwrap();
    let cancel = e
        .legal_actions(&choose)
        .into_iter()
        .find(|a| a.label == "Cancel")
        .expect("cancel");
    let back = e.apply(&choose, &cancel).unwrap();
    assert!(!back.card(base).epic_used);
    assert_eq!(back.decision, Decision::Window);
    assert!(e.legal_actions(&back).iter().any(|a| a.id() == Some(base)));
}

#[test]
fn epic_action_no_targets_can_spend_with_use_anyway() {
    let e = engine();
    let mut s = board(&e);
    let base = s.players[0].base;
    s.card_mut(base).def = e.by_code["SOR_019"];
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let prompt = e.apply(&s, &e.card_action(&s, base)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityNoEffect);
    let use_anyway = e
        .legal_actions(&prompt)
        .into_iter()
        .find(|a| a.label.contains("anyway"))
        .expect("use anyway");
    let done = e.apply(&prompt, &use_anyway).unwrap();
    assert!(done.card(base).epic_used);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn play_card_and_force_base_shapes_stay_rejected() {
    let e = engine();
    for code in ["SOR_022", "LAW_020", "LOF_022", "TWI_025"] {
        assert!(!e.by_code.contains_key(code), "{code} must stay rejected");
        let reason = e.rejected_cards().unwrap()[code].clone();
        assert!(
            reason.contains("base-zone") || reason.contains("playCard") || reason.contains("event hook") || reason.contains("invalid") || reason.contains("inherited"),
            "{code}: {reason}"
        );
    }
}
