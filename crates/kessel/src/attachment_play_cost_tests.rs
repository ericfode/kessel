//! Source-derived witnesses for physical host-dependent Standard_0 prices.
//! Synthetic trait grants exercise the admitted live trait executor; they are
//! diagnostic registrations, not additional claims about printed card text.
use super::*;
use crate::attachment_play_cost::{self, AttachmentPlayCost};

const ATTACHMENT_CASES: [(&str, &str, u16, AttachmentPlayCost); 3] = [
    ("ASH_262", "JTL_058", 2, AttachmentPlayCost::Imperial),
    ("ASH_263", "SHD_258", 2, AttachmentPlayCost::Mandalorian),
    ("LAW_129", "ASH_202", 4, AttachmentPlayCost::Unique),
];
fn attachment_catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn attachment_engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&attachment_catalogue()).unwrap();
    e.set_rules_backend(backend).unwrap();
    e
}
fn attachment_board(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 7;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn attachment_hand(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn attachment_resources(e: &Engine, s: &mut State, seat: Seat, count: u16) {
    for _ in 0..count {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
fn attachment_click(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "{:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}
fn attachment_entry(catalogue: &Value, code: &str) -> Value {
    catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["setCode"] == code)
        .unwrap()
        .clone()
}
fn attachment_replace(catalogue: &mut Value, code: &str, entry: Value) {
    *catalogue["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["setCode"] == code)
        .unwrap() = entry;
}

#[test]
fn attachment_play_cost_complete_sources_and_closed_native_rules() {
    let c = attachment_catalogue();
    let e = attachment_engine(RulesBackend::Auto);
    for (code, _, _, rule) in ATTACHMENT_CASES {
        let mut entry = attachment_entry(&c, code);
        assert_eq!(attachment_play_cost::extract(&entry).unwrap(), Some(rule));
        assert_eq!(
            e.definitions[e.by_code[code] as usize].attachment_play_cost,
            Some(rule)
        );
        assert!(e.definitions[e.by_code[code] as usize]
            .self_play_cost
            .is_none());
        let mut abilities = entry["abilities"].clone();
        attachment_play_cost::normalize(&entry, &mut abilities).unwrap();
        assert_eq!(abilities, json!({"leader":[],"unit":[],"card":[]}));
        entry["setCode"] = json!("DIAGNOSTIC_001");
        entry["id"] = json!("different");
        entry["internalName"] = json!("different");
        entry["abilities"]["card"][0]["title"] = json!("different");
        assert_eq!(attachment_play_cost::extract(&entry).unwrap(), Some(rule));
        assert_eq!(rule.apply(0, true), 0);
        assert_eq!(rule.apply(1, true), 0);
        assert_eq!(rule.apply(2, true), 1);
        assert_eq!(rule.apply(2, false), 2);
    }
    for code in ["LAW_110", "JTL_163", "HMW_173", "TS26_060"] {
        assert!(e.by_code.contains_key(code));
        assert_eq!(
            attachment_play_cost::extract(&attachment_entry(&c, code)).unwrap(),
            None
        );
    }
    for code in ["JTL_101", "SEC_122", "TS26_047"] {
        assert!(!e.by_code.contains_key(code), "unreviewed companion {code}");
    }
}

#[test]
fn attachment_play_cost_existence_and_menu_use_only_affordable_legal_hosts() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, qualifying, full, _) in ATTACHMENT_CASES {
                let mut s = attachment_board(&e, seat);
                let source = attachment_hand(&e, &mut s, seat, code);
                let plain = unit(&e, &mut s, seat, "LOF_256");
                attachment_resources(&e, &mut s, seat, full - 1);
                assert_eq!(
                    e.cost(&s, source),
                    full,
                    "target-free cost remains undiscounted"
                );
                assert_eq!(e.upgrade_play_cost(&s, source, plain), Some(full));
                assert!(!e.legal_actions(&s).contains(&e.card_action(&s, source)));
                let cheap = unit(&e, &mut s, seat, qualifying);
                assert_eq!(e.upgrade_play_cost(&s, source, cheap), Some(full - 1));
                let pending = attachment_click(&e, &s, source);
                assert_eq!(pending.decision, Decision::UpgradeTarget);
                assert_eq!(pending.source, source);
                let actions = e.legal_actions(&pending);
                assert!(actions.contains(&e.card_action(&pending, cheap)));
                assert!(!actions.contains(&e.card_action(&pending, plain)));
                assert!(!actions.contains(&e.card_action(&pending, pending.player(seat).leader)));
                assert!(!actions.contains(&e.card_action(&pending, pending.player(seat).base)));
                assert!(e.apply(&pending, &e.card_action(&pending, plain)).is_err());
                let done = attachment_click(&e, &pending, cheap);
                assert_eq!(done.card(source).parent, cheap);
                assert_eq!(e.ready_resources(&done, seat), 0);
                assert!(!done.player(seat).hand.contains(source));
            }
        }
    }
}

#[test]
fn attachment_play_cost_final_payment_prices_selected_host_without_stacking_copies() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, qualifying, full, _) in ATTACHMENT_CASES {
                for choose_qualifying in [false, true] {
                    let mut s = attachment_board(&e, seat);
                    let source = attachment_hand(&e, &mut s, seat, code);
                    let other = attachment_hand(&e, &mut s, seat, code);
                    let plain = unit(&e, &mut s, seat, "LOF_256");
                    let cheap = unit(&e, &mut s, seat, qualifying);
                    attachment_resources(&e, &mut s, seat, full);
                    let pending = attachment_click(&e, &s, source);
                    assert!(e
                        .legal_actions(&pending)
                        .contains(&e.card_action(&pending, plain)));
                    assert!(e
                        .legal_actions(&pending)
                        .contains(&e.card_action(&pending, cheap)));
                    let chosen = if choose_qualifying { cheap } else { plain };
                    let done = attachment_click(&e, &pending, chosen);
                    assert_eq!(done.card(source).parent, chosen);
                    assert_eq!(
                        e.ready_resources(&done, seat),
                        usize::from(choose_qualifying)
                    );
                    assert!(done.player(seat).hand.contains(other));
                    assert_eq!(e.cost(&done, other), full);
                    // Price query requires acting ownership, so restore the
                    // actor after the completed action before querying copy 2.
                    let mut next = done;
                    next.active = seat;
                    assert_eq!(e.upgrade_play_cost(&next, other, cheap), Some(full - 1));
                    assert_eq!(e.upgrade_play_cost(&next, other, plain), Some(full));
                }
            }
        }
    }
}

#[test]
fn attachment_play_cost_preparation_cancel_and_resume_spend_only_at_chosen_host() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, qualifying, full, _) in ATTACHMENT_CASES {
                let mut s = attachment_board(&e, seat);
                let source = attachment_hand(&e, &mut s, seat, code);
                let host = unit(&e, &mut s, seat, qualifying);
                attachment_resources(&e, &mut s, seat, full);
                let pending = attachment_click(&e, &s, source);
                assert_eq!(e.ready_resources(&pending, seat), usize::from(full));
                assert!(pending.player(seat).hand.contains(source));
                assert_eq!(pending.card(source).parent, 0);
                let resumed = pending; // Existing inline prompt state; no new payment prompt.
                assert_eq!(e.legal_actions(&resumed), e.legal_actions(&pending));
                let cancelled = e
                    .apply(&pending, &Action::button("Cancel", "cancel"))
                    .unwrap();
                assert_eq!(cancelled.decision, Decision::Window);
                assert_eq!(cancelled.source, 0);
                assert_eq!(cancelled.active, seat);
                assert_eq!(e.ready_resources(&cancelled, seat), usize::from(full));
                assert!(cancelled.player(seat).hand.contains(source));
                let reopened = attachment_click(&e, &cancelled, source);
                let done = attachment_click(&e, &reopened, host);
                assert_eq!(done.card(source).parent, host);
                assert_eq!(e.ready_resources(&done, seat), 1);
                assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
            }
        }
    }
}

#[test]
fn attachment_play_cost_enemy_units_and_actual_deployed_leaders_are_legal_hosts() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for (code, qualifying, full, _) in ATTACHMENT_CASES {
                let mut s = attachment_board(&e, seat);
                let source = attachment_hand(&e, &mut s, seat, code);
                let host = unit(&e, &mut s, seat.other(), qualifying);
                attachment_resources(&e, &mut s, seat, full - 1);
                let pending = attachment_click(&e, &s, source);
                let done = attachment_click(&e, &pending, host);
                assert_eq!(
                    done.card(source).owner,
                    seat,
                    "actor controls upgrade on enemy host"
                );
                assert_eq!(done.card(host).owner, seat.other());
                assert_eq!(done.card(source).parent, host);
                assert_eq!(e.ready_resources(&done, seat), 0);
                assert_eq!(e.ready_resources(&done, seat.other()), 0);
            }
            for (code, leader_code, full) in [
                ("ASH_262", "IBH_053", 2),
                ("ASH_263", "SOR_014", 2),
                ("LAW_129", "SOR_014", 4),
            ] {
                let mut s = attachment_board(&e, seat);
                let leader = add(&e, &mut s, seat.other(), leader_code);
                s.player_mut(seat.other()).leader = leader;
                let source = attachment_hand(&e, &mut s, seat, code);
                attachment_resources(&e, &mut s, seat, full - 1);
                assert_eq!(e.upgrade_play_cost(&s, source, leader), None);
                assert!(!e.legal_actions(&s).contains(&e.card_action(&s, source)));
                s.card_mut(leader).deployed = true;
                s.card_mut(leader).epic_used = true;
                e.add_arena(&mut s, leader).unwrap();
                assert_eq!(e.upgrade_play_cost(&s, source, leader), Some(full - 1));
                let pending = attachment_click(&e, &s, source);
                let done = attachment_click(&e, &pending, leader);
                assert_eq!(done.card(source).parent, leader);
                assert_eq!(done.card(source).owner, seat);
            }
        }
    }
}

#[test]
fn attachment_play_cost_live_trait_changes_refresh_menu_and_revalidate_final_payment() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            for (code, native_host, full, rule) in ATTACHMENT_CASES.into_iter().take(2) {
                let mut e = attachment_engine(backend);
                let mut s = attachment_board(&e, seat);
                let source = attachment_hand(&e, &mut s, seat, code);
                let host = unit(&e, &mut s, seat, "LOF_256");
                let trait_name = if rule == AttachmentPlayCost::Imperial {
                    "imperial"
                } else {
                    "mandalorian"
                };
                attachment_resources(&e, &mut s, seat, full - 1);
                let gain = add(&e, &mut s, seat, "IC27_052");
                e.attach(&mut s, gain, host);
                death_program(
                    &mut e,
                    &s,
                    gain,
                    json!({"trigger":"constant","title":"Diagnostic gain",
                    "attachedContext":"upgrade","effect":{"kind":"gainTrait","target":"parent","trait":trait_name,"duration":"persistent"}}),
                );
                assert_eq!(e.upgrade_play_cost(&s, source, host), Some(full - 1));
                let mut pending = attachment_click(&e, &s, source);
                let loss = add(&e, &mut pending, seat, "IC27_093");
                e.attach(&mut pending, loss, host);
                death_program(
                    &mut e,
                    &pending,
                    loss,
                    json!({"trigger":"constant","title":"Diagnostic lose",
                    "attachedContext":"upgrade","effect":{"kind":"loseTrait","target":"parent","trait":trait_name,"duration":"persistent"}}),
                );
                assert_eq!(e.upgrade_play_cost(&pending, source, host), Some(full));
                assert!(!e
                    .legal_actions(&pending)
                    .contains(&e.card_action(&pending, host)));
                assert!(e.apply(&pending, &e.card_action(&pending, host)).is_err());
                assert_eq!(e.ready_resources(&pending, seat), usize::from(full - 1));
                // Another matching host cannot discount the originally chosen
                // target. With enough resources the lost-trait host pays full.
                let other = unit(&e, &mut pending, seat, native_host);
                attachment_resources(&e, &mut pending, seat, 1);
                assert_eq!(e.upgrade_play_cost(&pending, source, other), Some(full - 1));
                let done = attachment_click(&e, &pending, host);
                assert_eq!(done.card(source).parent, host);
                assert_eq!(e.ready_resources(&done, seat), 0);
                // Removing the loss restores the gained trait in the existing
                // prompt, without paying anything during preparation.
                let mut restored = pending;
                e.leave_play(&mut restored, loss).unwrap();
                assert_eq!(e.upgrade_play_cost(&restored, source, host), Some(full - 1));
                let done = attachment_click(&e, &restored, host);
                assert_eq!(e.ready_resources(&done, seat), 1);
                let mut removed = restored;
                e.leave_play(&mut removed, gain).unwrap();
                assert_eq!(e.upgrade_play_cost(&removed, source, host), Some(full));
            }
        }
    }
}

#[test]
fn attachment_play_cost_inventory_excludes_pilot_mode_and_nonarena_cards() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = attachment_board(&e, seat);
            let source = attachment_hand(&e, &mut s, seat, "ASH_262");
            let pilot = unit(&e, &mut s, seat, "JTL_058");
            let host = unit(&e, &mut s, seat, "LAW_253");
            attachment_resources(&e, &mut s, seat, 1);
            assert_eq!(e.upgrade_play_targets(&s, source), vec![pilot]);
            e.leave_play(&mut s, pilot).unwrap();
            s.player_mut(seat).discard.remove(pilot);
            e.attach(&mut s, pilot, host);
            assert_eq!(e.upgrade_play_cost(&s, source, pilot), None);
            assert_eq!(e.upgrade_play_cost(&s, source, host), Some(2));
            assert!(e.upgrade_play_targets(&s, source).is_empty());
            assert!(!e.legal_actions(&s).contains(&e.card_action(&s, source)));
            let hidden = attachment_hand(&e, &mut s, seat, "JTL_058");
            assert_eq!(e.upgrade_play_cost(&s, source, hidden), None);
            assert_eq!(e.upgrade_play_cost(&s, source, s.player(seat).base), None);
            assert_eq!(e.upgrade_play_cost(&s, source, s.player(seat).leader), None);
            assert_eq!(
                e.cost(&s, source),
                2,
                "upgrade's own Imperial trait is not a host"
            );
        }
    }
}

#[test]
fn attachment_play_cost_aspect_penalty_precedes_selected_host_reduction() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = attachment_board(&e, seat);
            s.player_mut(seat).base = add(&e, &mut s, seat, "SOR_030");
            let source = attachment_hand(&e, &mut s, seat, "LAW_129");
            let unique = unit(&e, &mut s, seat, "ASH_202");
            let plain = unit(&e, &mut s, seat, "LOF_256");
            attachment_resources(&e, &mut s, seat, 5);
            assert_eq!(e.cost(&s, source), 6);
            assert_eq!(e.upgrade_play_cost(&s, source, unique), Some(5));
            assert_eq!(e.upgrade_play_cost(&s, source, plain), Some(6));
            let pending = attachment_click(&e, &s, source);
            assert!(!e
                .legal_actions(&pending)
                .contains(&e.card_action(&pending, plain)));
            let done = attachment_click(&e, &pending, unique);
            assert_eq!(e.ready_resources(&done, seat), 0);
        }
    }
}

#[test]
fn attachment_play_cost_forged_source_contexts_and_departed_host_reject() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = attachment_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = attachment_board(&e, seat);
            let source = attachment_hand(&e, &mut s, seat, "ASH_262");
            let host = unit(&e, &mut s, seat, "JTL_058");
            attachment_resources(&e, &mut s, seat, 2);
            let pending = attachment_click(&e, &s, source);
            let mut departed = pending;
            e.leave_play(&mut departed, host).unwrap();
            assert_eq!(e.upgrade_play_cost(&departed, source, host), None);
            assert!(e.apply(&departed, &e.card_action(&departed, host)).is_err());
            for zone in ["discard", "resource", "foreign"] {
                let mut forged = pending;
                forged.player_mut(seat).hand.remove(source);
                match zone {
                    "discard" => forged.player_mut(seat).discard.push(source).unwrap(),
                    "resource" => forged.player_mut(seat).resources.push(source).unwrap(),
                    _ => {
                        forged.card_mut(source).owner = seat.other();
                        forged.player_mut(seat.other()).hand.push(source).unwrap();
                    }
                }
                assert_eq!(e.upgrade_play_cost(&forged, source, host), None, "{zone}");
                assert_eq!(
                    e.legal_actions(&forged),
                    vec![Action::button("Cancel", "cancel")]
                );
                assert!(e.apply(&forged, &e.card_action(&forged, host)).is_err());
                assert_eq!(forged.card(source).parent, 0);
            }
            assert_eq!(e.upgrade_play_cost(&s, 0, host), None);
            assert_eq!(e.upgrade_play_cost(&s, host, source), None);
        }
    }
}

#[test]
fn attachment_play_cost_complete_contract_mutations_fail_closed() {
    let catalogue = attachment_catalogue();
    for (code, _, _, _) in ATTACHMENT_CASES {
        let original = attachment_entry(&catalogue, code);
        for (pointer, value) in [
            ("/baseClass", json!("NonLeaderUnitCard")),
            ("/status", json!("partial")),
            ("/card/types", json!(["upgrade", "unit"])),
            ("/card/unique", json!(true)),
            ("/card/cost", json!(0)),
            ("/card/power", json!(9)),
            ("/card/hp", json!(9)),
            ("/card/upgradePower", json!(9)),
            ("/card/upgradeHp", json!(9)),
            ("/card/aspects", json!(["command"])),
            ("/card/traits", json!(["rebel"])),
            ("/card/text", json!("Unreviewed text")),
            ("/card/keywords", json!(["bounty"])),
            ("/keywordValues", json!({"grit":1})),
            ("/attachCondition", json!({"kind":"const","value":true})),
            ("/piloting", json!({"cost":1})),
            ("/abilities/card/0/effect/sourceZone", json!("hand")),
            ("/abilities/card/0/effect/targetPlayers", json!("self")),
            ("/abilities/card/0/effect/matchSourceIdentity", json!(false)),
            ("/abilities/card/0/effect/matchPrintedType", json!(false)),
            ("/abilities/card/0/effect/stage", json!("increase_1")),
            (
                "/abilities/card/0/effect/amount",
                json!({"kind":"const","value":2}),
            ),
            ("/abilities/card/0/effect/matchAbilityCosts", json!(true)),
            (
                "/abilities/card/0/effect/matchCardEffectResourcePayments",
                json!(true),
            ),
            (
                "/abilities/card/0/effect/attachTargetCondition/parameters",
                json!(["target"]),
            ),
            (
                "/abilities/card/0/effect/attachTargetCondition/timing",
                json!("resolution"),
            ),
            (
                "/abilities/card/0/effect/attachTargetCondition/value",
                json!({"kind":"const","value":true}),
            ),
        ] {
            let mut bad = original.clone();
            if let Some(slot) = bad.pointer_mut(pointer) {
                *slot = value;
            } else {
                // Optional top-level metadata is absent in the pinned catalog.
                assert_eq!(pointer.matches('/').count(), 1);
                bad[&pointer[1..]] = value;
            }
            assert!(
                attachment_play_cost::extract(&bad).is_err(),
                "{code} {pointer}"
            );
        }
        for key in [
            "arena",
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "rules",
            "epicAction",
        ] {
            let mut bad = original.clone();
            bad["card"][key] = json!("unknown");
            assert!(attachment_play_cost::extract(&bad).is_err(), "{code} {key}");
        }
        for key in ["condition", "optional", "limit", "unknown"] {
            let mut bad = original.clone();
            bad["abilities"]["card"][0][key] = json!(true);
            assert!(attachment_play_cost::extract(&bad).is_err(), "{code} {key}");
        }
        for key in [
            "stateWatchers",
            "historyHelpers",
            "privatePrograms",
            "classMethods",
            "privateClassConstants",
            "inheritedSource",
            "deploymentOverride",
            "setupValidationOverrides",
        ] {
            let mut bad = original.clone();
            bad[key] = json!([{"kind":"unknown"}]);
            assert!(attachment_play_cost::extract(&bad).is_err(), "{code} {key}");
        }
        for side in ["card", "unit", "leader"] {
            let mut bad = original.clone();
            bad["abilities"][side]
                .as_array_mut()
                .unwrap()
                .push(json!({"trigger":"whenPlayed","costs":[],"effect":{"kind":"noAction"}}));
            assert!(
                attachment_play_cost::extract(&bad).is_err(),
                "{code} {side}"
            );
        }
        let mut bad = original.clone();
        bad["abilities"]["card"][0]["title"] =
            json!({"kind":"boundCallback","value":{"kind":"unknown"}});
        assert!(attachment_play_cost::extract(&bad).is_err());
        let mut abilities = bad["abilities"].clone();
        let before = abilities.clone();
        assert!(attachment_play_cost::normalize(&bad, &mut abilities).is_err());
        assert_eq!(
            abilities, before,
            "failed validation cannot erase a registration"
        );
        let mut bad = original.clone();
        bad["abilities"]["card"][0]["effect"]["unexpected"] = json!(true);
        assert!(attachment_play_cost::extract(&bad).is_err());
        let mut bad = original.clone();
        bad["abilities"]["card"][0]["effect"]["attachTargetCondition"]["unexpected"] = json!(true);
        assert!(attachment_play_cost::extract(&bad).is_err());
    }
}

#[test]
fn attachment_play_cost_full_catalogue_rejects_forged_watchers_and_inherited_sources() {
    for code in ["ASH_262", "ASH_263", "LAW_129"] {
        for key in ["stateWatchers", "inheritedSource"] {
            let mut catalogue = attachment_catalogue();
            let mut bad = attachment_entry(&catalogue, code);
            bad[key] = json!([{"kind":"unknown"}]);
            attachment_replace(&mut catalogue, code, bad);
            let e = Engine::from_catalogue_value(&catalogue).unwrap();
            assert!(!e.by_code.contains_key(code), "{code} {key}");
            for (other, _, _, _) in ATTACHMENT_CASES {
                if other != code {
                    assert!(e.by_code.contains_key(other));
                }
            }
        }
    }
}

#[test]
fn attachment_play_cost_selected_pool_round_trip_retains_native_host_rule() {
    let c = attachment_catalogue();
    let e = attachment_engine(RulesBackend::Auto);
    let pool = e.compiled_pool().unwrap();
    let loaded = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    )
    .unwrap();
    for seat in [Seat::P1, Seat::P2] {
        for (code, qualifying, full, _) in ATTACHMENT_CASES {
            let mut s = attachment_board(&loaded, seat);
            let source = attachment_hand(&loaded, &mut s, seat, code);
            let host = unit(&loaded, &mut s, seat, qualifying);
            attachment_resources(&loaded, &mut s, seat, full - 1);
            assert_eq!(loaded.cost(&s, source), full);
            assert_eq!(loaded.upgrade_play_cost(&s, source, host), Some(full - 1));
            let pending = attachment_click(&loaded, &s, source);
            let done = attachment_click(&loaded, &pending, host);
            assert_eq!(done.card(source).parent, host);
            assert_eq!(loaded.ready_resources(&done, seat), 0);
        }
    }
}
