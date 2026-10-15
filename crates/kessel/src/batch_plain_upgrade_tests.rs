// Included beneath generic_runtime_tests so the existing board/action helpers
// exercise actual paid play. Expectations come from the pinned default
// UpgradeCard and PlayUpgradeAction, not generated ability programs.
use super::*;

fn plain_upgrade_roundtrip(e: &Engine, state: &State) -> State {
    let snapshot = e.snapshot(state);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    loaded
}

fn plain_upgrade_paid_attachment(code: &str, base: &str, cost: usize, power: u16, hp: u16) {
    let e = engine();
    let metadata = &e.catalogue.as_ref().unwrap().metadata[code];
    assert_eq!(metadata["types"], json!(["upgrade"]));
    let definition = &e.definitions[e.by_code[code] as usize];
    assert!(definition.programs.card.is_empty());
    assert!(definition.programs.leader.is_empty());
    assert!(definition.programs.unit.is_empty());

    for owner in [Seat::P1, Seat::P2] {
        for exhausted in [false, true] {
            let mut s = board(&e);
            s.round = 6;
            let base_id = s.player(Seat::P1).base;
            s.card_mut(base_id).def = e.by_code[base];
            let host = unit(&e, &mut s, owner, "LOF_254");
            s.card_mut(host).exhausted = exhausted;
            let initial_power = e.power(&s, host, false);
            let initial_hp = e.hp(&s, host);
            let upgrade = add(&e, &mut s, Seat::P1, code);
            s.player_mut(Seat::P1).hand.push(upgrade).unwrap();
            for _ in 0..6 {
                let resource = add(&e, &mut s, Seat::P1, "LOF_254");
                s.player_mut(Seat::P1).resources.push(resource).unwrap();
            }
            s = plain_upgrade_roundtrip(&e, &s);
            assert_eq!(e.cost(&s, upgrade) as usize, cost);
            let ready = e.ready_resources(&s, Seat::P1);

            let choice = e.apply(&s, &e.card_action(&s, upgrade)).unwrap();
            // UpgradeTarget is an internal selection prompt, not a serializable
            // action-window snapshot. Stable states before and after still round-trip.
            e.validate_attachment_links(&choice).unwrap();
            assert_eq!(choice.decision, Decision::UpgradeTarget);
            let actions = e.legal_actions(&choice);
            assert!(actions.contains(&e.card_action(&choice, host)));
            for seat in [Seat::P1, Seat::P2] {
                assert!(!actions.contains(&e.card_action(&choice, choice.player(seat).base)));
                assert!(!actions.contains(&e.card_action(&choice, choice.player(seat).leader)));
            }
            assert_eq!(e.ready_resources(&choice, Seat::P1), ready);
            let cancelled = e
                .apply(&choice, &Action::button("Cancel", "cancel"))
                .unwrap();
            let cancelled = plain_upgrade_roundtrip(&e, &cancelled);
            assert!(cancelled.player(Seat::P1).hand.contains(upgrade));
            assert_eq!(e.ready_resources(&cancelled, Seat::P1), ready);

            let mut attached = e.apply(&choice, &e.card_action(&choice, host)).unwrap();
            attached = plain_upgrade_roundtrip(&e, &attached);
            assert_eq!(e.ready_resources(&attached, Seat::P1), ready - cost);
            assert!(!attached.player(Seat::P1).hand.contains(upgrade));
            assert_eq!(attached.card(upgrade).owner, Seat::P1);
            assert_eq!(attached.card(upgrade).parent, host);
            assert_eq!(e.upgrades(&attached, host), vec![upgrade]);
            assert_eq!(e.power(&attached, host, false), initial_power + power);
            assert_eq!(e.hp(&attached, host), initial_hp + hp);
            assert_eq!(attached.card(host).exhausted, exhausted);
            assert_eq!(attached.card(host).owner, owner);
            for seat in [Seat::P1, Seat::P2] {
                assert!(!attached.player(seat).ground.contains(upgrade));
                assert!(!attached.player(seat).space.contains(upgrade));
            }

            e.defeat(&mut attached, host).unwrap();
            attached = plain_upgrade_roundtrip(&e, &attached);
            assert!(attached.player(owner).discard.contains(host));
            assert!(attached.player(Seat::P1).discard.contains(upgrade));
            assert_eq!(attached.card(upgrade).parent, 0);
            assert!(e.upgrades(&attached, host).is_empty());
            e.validate_attachment_links(&attached).unwrap();
        }
    }
}

#[test]
fn batch_plain_academy_training_pays_two_for_two_two_on_either_host() {
    plain_upgrade_paid_attachment("SOR_120", "SOR_023", 2, 2, 2);
}

#[test]
fn batch_plain_resilient_pays_one_for_zero_three_on_either_host() {
    plain_upgrade_paid_attachment("SHD_070", "ASH_019", 1, 0, 3);
}

#[test]
fn batch_plain_champion_pays_five_for_four_four_on_either_host() {
    plain_upgrade_paid_attachment("TS26_045", "ASH_019", 5, 4, 4);
}
