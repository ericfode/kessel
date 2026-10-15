// Include as a child module of generic_runtime_tests.rs.
use super::*;

fn kalani_roundtrip(e: &Engine, s: &State) -> State {
    let snapshot = e.snapshot(s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(e.legal_actions(&restored), e.legal_actions(s));
    restored
}

#[test]
fn kalani_actual_attack_preserves_initiative_limits_zero_choices_and_phase_expiry() {
    let e = engine();
    assert!(e.by_code.contains_key("TWI_085"), "complete normalized card must pass unchanged admission");
    for seat in [Seat::P1, Seat::P2] {
        for has_initiative in [false, true] {
            for eligible in 0..=2 {
                for requested in 0..=2 {
                    let decline = requested == 0;
                    let mut s = board(&e);
                    s.round = 6;
                    s.active = seat;
                    s.action_player = Some(seat);
                    s.initiative = if has_initiative { seat } else { seat.other() };
                    let source = unit(&e, &mut s, seat, "TWI_085");
                    let targets: Vec<_> = (0..eligible).map(|i| unit(&e, &mut s, if i == 0 { seat } else { seat.other() }, "LOF_256")).collect();
                    let base = s.player(seat.other()).base;
                    let s = kalani_roundtrip(&e, &s);
                    let program = e.def(&s, source).programs.card[0];
                    let limit = if has_initiative { 2 } else { 1 };
                    let title = format!("Give +2/+2 for this phase to up to {limit} other {}", if limit == 1 { "unit" } else { "units" });
                    assert!(e.program_menu_title(&s, source, program).ends_with(&title));
                    let attack = e.card_action(&s, source);
                    assert!(e.legal_actions(&s).contains(&attack));
                    let choosing_defender = e.apply(&s, &attack).unwrap();
                    assert_eq!(choosing_defender.decision, Decision::AttackTarget);
                    let attack_base = e.card_action(&choosing_defender, base);
                    assert!(e.legal_actions(&choosing_defender).contains(&attack_base));
                    let mut resolving = e.apply(&choosing_defender, &attack_base).unwrap();
                    let chosen = requested.min(eligible).min(limit);
                    if eligible > 0 {
                        assert_eq!(resolving.decision, Decision::AbilityTarget);
                        let mut actual_targets = e.legal_actions(&resolving).iter().filter_map(Action::id).collect::<Vec<_>>();
                        let mut expected_targets = targets.clone();
                        actual_targets.sort_unstable();
                        expected_targets.sort_unstable();
                        assert_eq!(actual_targets, expected_targets);
                        let buttons = e.legal_actions(&resolving);
                        assert!(buttons.iter().any(|a| a.label == "Choose nothing"));
                        assert!(!buttons.iter().any(|a| ["Cancel", "Pass", "Choose no target"].contains(&a.label.as_str())));
                        if !decline {
                            // Both limits retain multi-selection click toggles, including limit one.
                            let first = e.card_action(&resolving, targets[0]);
                            resolving = e.apply(&resolving, &first).unwrap();
                            assert!(e.legal_actions(&resolving).iter().any(|a| a.id() == Some(targets[0])));
                            let toggle = e.card_action(&resolving, targets[0]);
                            resolving = e.apply(&resolving, &toggle).unwrap();
                            assert!(e.legal_actions(&resolving).iter().any(|a| a.label == "Choose nothing"));
                            for &target in targets.iter().take(chosen) {
                                let action = e.card_action(&resolving, target);
                                assert!(e.legal_actions(&resolving).contains(&action));
                                resolving = e.apply(&resolving, &action).unwrap();
                            }
                            if chosen == limit {
                                let mut at_limit = e.legal_actions(&resolving).iter().filter_map(Action::id).collect::<Vec<_>>();
                                let mut expected = targets[..chosen].to_vec();
                                at_limit.sort_unstable();
                                expected.sort_unstable();
                                assert_eq!(at_limit, expected);
                                for &extra in targets.iter().skip(chosen) {
                                    assert!(!e.legal_actions(&resolving).iter().any(|a| a.id() == Some(extra)));
                                    assert!(e.apply(&resolving, &e.card_action(&resolving, extra)).is_err());
                                }
                            }
                        }
                        // Clicks only select; modifiers apply together after Done.
                        for &pending in &targets {
                            assert_eq!(resolving.card(pending).phase_power, 0);
                            assert_eq!(resolving.card(pending).phase_hp, 0);
                        }
                        resolving = named_button(&e, &resolving, if decline { "Choose nothing" } else { "Done" });
                    }
                    assert_eq!(resolving.decision, Decision::Window);
                    assert_eq!(resolving.card(base).damage, 5);
                    assert!(resolving.card(source).exhausted);
                    for (index, &target) in targets.iter().enumerate() {
                        let bonus = if index < chosen { 2 } else { 0 };
                        assert_eq!(resolving.card(target).phase_power, bonus);
                        assert_eq!(resolving.card(target).phase_hp, bonus);
                        assert_eq!(e.power(&resolving, target, false), 1 + bonus as u16);
                        assert_eq!(e.hp(&resolving, target), 4 + bonus as u16);
                    }
                    let mut resolved = kalani_roundtrip(&e, &resolving);
                    // Two legal passes end the action phase and expire the grants.
                    for _ in 0..2 {
                        let pass = e.legal_actions(&resolved).into_iter().find(|a| a.label == "Pass").unwrap();
                        resolved = e.apply(&resolved, &pass).unwrap();
                    }
                    assert!(resolved.regroup);
                    for &target in &targets {
                        assert_eq!(resolved.card(target).phase_power, 0);
                        assert_eq!(resolved.card(target).phase_hp, 0);
                        assert_eq!(e.power(&resolved, target, false), 1);
                        assert_eq!(e.hp(&resolved, target), 4);
                    }
                }
            }
        }
    }
}
