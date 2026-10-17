//! Source-derived self-play event cost witnesses. Diagnostic trait grants below
//! exercise the existing live trait executor; no reference-engine run is claimed.
use super::*;

fn event_catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn event_engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&event_catalogue()).unwrap();
    e.set_rules_backend(backend).unwrap();
    e
}
fn event_board(e: &Engine, seat: Seat) -> State {
    event_board_with_leader(e, seat, "SOR_014")
}
fn event_board_with_leader(e: &Engine, seat: Seat, own_leader: &str) -> State {
    let mut s = State::default();
    s.round = 7;
    s.active = seat;
    s.action_player = Some(seat);
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for owner in [Seat::P1, Seat::P2] {
        let base = add(e, &mut s, owner, "ASH_019");
        let leader = add(
            e,
            &mut s,
            owner,
            if owner == seat { own_leader } else { "SOR_014" },
        );
        s.player_mut(owner).base = base;
        s.player_mut(owner).leader = leader;
        s.player_mut(owner).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn event_hand(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn event_resources(e: &Engine, s: &mut State, seat: Seat, count: usize) {
    for _ in 0..count {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
fn event_deploy(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = s.player(seat).leader;
    s.card_mut(id).deployed = true;
    s.card_mut(id).epic_used = true;
    e.add_arena(s, id).unwrap();
    id
}
fn event_click(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}
fn event_button(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)));
    e.apply(s, &action).unwrap()
}
fn event_finish_empty(e: &Engine, s: State) -> State {
    match s.decision {
        Decision::AbilityTarget => event_button(e, &s, "Choose nothing"),
        Decision::AbilityNoEffect => event_button(e, &s, "Play anyway"),
        Decision::Window | Decision::Terminal => s,
        _ => panic!("unexpected empty-event decision {:?}", s.decision),
    }
}

#[test]
fn self_play_event_complete_sources_keep_their_executable_event() {
    let c = event_catalogue();
    for code in ["TS26_060", "HMW_173"] {
        let entry = c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["setCode"] == code)
            .unwrap();
        assert!(crate::self_play_cost::extract(entry).unwrap().is_some());
        let mut abilities = entry["abilities"].clone();
        crate::self_play_cost::normalize(entry, &mut abilities).unwrap();
        assert_eq!(
            abilities["card"],
            json!([entry["abilities"]["card"][1].clone()])
        );
        assert_eq!(abilities["card"][0]["trigger"], "event");
    }
    let e = event_engine(RulesBackend::Auto);
    for code in ["TS26_060", "HMW_173", "LAW_110", "JTL_163"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in [
        "JTL_101", "SEC_122", "TS26_047",
    ] {
        assert!(!e.by_code.contains_key(code), "incomplete companion {code}");
    }
}

#[test]
fn self_play_event_leader_count_uses_actual_arena_units_only() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = event_board(&e, seat);
            let source = event_hand(&e, &mut s, seat, "TS26_060");
            let original = e.cost(&s, source);
            assert_eq!(original, 5, "printed 3 plus missing Command penalty 2");
            event_deploy(&e, &mut s, seat.other());
            let host = unit(&e, &mut s, seat, "LAW_253");
            let pilot = add(&e, &mut s, seat, "JTL_045");
            e.attach(&mut s, pilot, host);
            assert_eq!(
                e.cost(&s, source),
                original,
                "enemy leader, own front leader and Pilot upgrade do not count"
            );
            let leader = event_deploy(&e, &mut s, seat);
            assert_eq!(e.cost(&s, source), original - 1);
            let duplicate_event = event_hand(&e, &mut s, seat, "TS26_060");
            assert_eq!(
                e.cost(&s, duplicate_event),
                original - 1,
                "one physical self registration per event"
            );
            e.defeat(&mut s, leader).unwrap();
            assert_eq!(
                e.cost(&s, source),
                original,
                "defeated front leader stops qualifying"
            );
        }
    }
}

#[test]
fn self_play_event_rebel_count_excludes_pilot_upgrade_hidden_cards_and_enemies() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = event_board(&e, seat);
            let source = event_hand(&e, &mut s, seat, "HMW_173");
            assert_eq!(
                e.cost(&s, source),
                3,
                "friendly Rebel front leader discounts printed 4"
            );
            let ground = unit(&e, &mut s, seat, "JTL_045");
            let host = unit(&e, &mut s, seat, "LAW_253");
            assert_eq!(e.cost(&s, source), 1);
            // Move the same physical Pilot from unit mode to attachment mode;
            // do not manufacture simultaneous unique Hera copies in play.
            e.leave_play(&mut s, ground).unwrap();
            s.player_mut(seat).discard.remove(ground);
            e.attach(&mut s, ground, host);
            unit(&e, &mut s, seat.other(), "LAW_253");
            for zone in ["hand", "deck", "discard", "resources"] {
                let id = add(&e, &mut s, seat, "JTL_045");
                match zone {
                    "hand" => s.player_mut(seat).hand.push(id).unwrap(),
                    "deck" => s.player_mut(seat).deck.push(id).unwrap(),
                    "discard" => s.player_mut(seat).discard.push(id).unwrap(),
                    _ => s.player_mut(seat).resources.push(id).unwrap(),
                }
            }
            assert_eq!(
                e.cost(&s, source),
                2,
                "physical Pilot upgrade and nonplay/enemy Rebels are excluded"
            );
            let leader = event_deploy(&e, &mut s, seat);
            assert_eq!(e.cost(&s, source), 2, "deployed leader is counted once");
            e.defeat(&mut s, ground).unwrap();
            assert_eq!(e.cost(&s, source), 2);
            e.defeat(&mut s, leader).unwrap();
            assert_eq!(
                e.cost(&s, source),
                2,
                "Rebel leader in base zone still qualifies"
            );
            for _ in 0..8 {
                unit(&e, &mut s, seat, "LAW_253");
            }
            assert_eq!(
                e.cost(&s, source),
                0,
                "live static reduction floors at zero"
            );
        }
    }
}

#[test]
fn self_play_event_rebel_discount_reads_live_trait_gain_and_loss() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = event_board(&e, seat);
            let source = event_hand(&e, &mut s, seat, "HMW_173");
            let host = unit(&e, &mut s, seat, "LOF_256");
            let initial = e.cost(&s, source);
            let gain = add(&e, &mut s, seat, "IC27_052");
            e.attach(&mut s, gain, host);
            // Synthetic diagnostic registrations use the established trait
            // executor; these are not printed-card witnesses for the upgrades.
            death_program(
                &mut e,
                &s,
                gain,
                json!({"trigger":"constant","title":"Diagnostic gain Rebel",
                "attachedContext":"upgrade","effect":{"kind":"gainTrait","target":"parent","trait":"rebel","duration":"persistent"}}),
            );
            assert_eq!(e.cost(&s, source), initial - 1);
            let loss = add(&e, &mut s, seat, "IC27_093");
            e.attach(&mut s, loss, host);
            death_program(
                &mut e,
                &s,
                loss,
                json!({"trigger":"constant","title":"Diagnostic lose Rebel",
                "attachedContext":"upgrade","effect":{"kind":"loseTrait","target":"parent","trait":"rebel","duration":"persistent"}}),
            );
            assert_eq!(e.cost(&s, source), initial);
            e.defeat(&mut s, loss).unwrap();
            assert_eq!(e.cost(&s, source), initial - 1);
            e.defeat(&mut s, gain).unwrap();
            assert_eq!(e.cost(&s, source), initial);
        }
    }
}

#[test]
fn self_play_event_rebel_operation_pays_live_cost_then_draws_two_or_fatigues() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for deck_size in [0, 1, 3] {
                let mut s = event_board(&e, seat);
                let source = event_hand(&e, &mut s, seat, "HMW_173");
                let other = event_hand(&e, &mut s, seat, "HMW_173");
                let mut deck = vec![];
                for _ in 0..deck_size {
                    let id = add(&e, &mut s, seat, "LOF_256");
                    s.player_mut(seat).deck.push(id).unwrap();
                    deck.push(id);
                }
                let cost = e.cost(&s, source) as usize;
                event_resources(&e, &mut s, seat, cost);
                let mut short = s;
                let resource = short.player(seat).resources.as_slice()[0];
                short.card_mut(resource).exhausted = true;
                assert!(!e
                    .legal_actions(&short)
                    .contains(&e.card_action(&short, source)));
                let base = s.player(seat).base;
                let before = s.card(base).damage;
                let done = event_click(&e, &s, source);
                assert_eq!(done.decision, Decision::Window);
                assert_eq!(e.ready_resources(&done, seat), 0);
                assert!(done.player(seat).discard.contains(source));
                assert!(done.player(seat).hand.contains(other));
                let drawn = deck_size.min(2);
                for id in &deck[..drawn] {
                    assert!(done.player(seat).hand.contains(*id));
                }
                assert_eq!(done.player(seat).hand.len as usize, drawn + 1);
                assert_eq!(done.player(seat).deck.as_slice(), &deck[drawn..]);
                assert_eq!(done.card(base).damage, before + 3 * (2 - drawn) as u16);
            }
        }
    }
}

#[test]
fn self_play_event_take_charge_prepares_before_payment_and_can_cancel() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = event_board(&e, seat);
            let source = event_hand(&e, &mut s, seat, "TS26_060");
            let own = unit(&e, &mut s, seat, "LOF_256");
            let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
            event_deploy(&e, &mut s, seat);
            let cost = e.cost(&s, source) as usize;
            event_resources(&e, &mut s, seat, cost);
            let prompt = event_click(&e, &s, source);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert_eq!(e.ready_resources(&prompt, seat), cost);
            assert!(prompt.player(seat).hand.contains(source));
            for id in [own, enemy] {
                assert!(e
                    .legal_actions(&prompt)
                    .contains(&e.card_action(&prompt, id)));
            }
            let selected = event_click(&e, &prompt, enemy);
            assert_eq!(e.ready_resources(&selected, seat), cost);
            let cancelled = event_button(&e, &selected, "Cancel");
            assert_eq!(cancelled.decision, Decision::Window);
            assert_eq!(e.ready_resources(&cancelled, seat), cost);
            assert!(cancelled.player(seat).hand.contains(source));
            assert_eq!(cancelled.card(enemy).experience, 0);
            let chosen = event_click(&e, &event_click(&e, &cancelled, source), enemy);
            let done = event_button(&e, &chosen, "Done");
            assert_eq!(e.ready_resources(&done, seat), 0);
            assert!(done.player(seat).discard.contains(source));
            assert_ne!(done.card(enemy).experience, 0);
            assert_eq!(done.card(done.card(enemy).experience).owner, seat.other());
            assert_eq!(done.card(own).experience, 0);
        }
    }
}

#[test]
fn self_play_event_take_charge_up_to_three_zero_toggle_and_token_ownership() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for count in 0..=3 {
                let mut s = event_board(&e, seat);
                let source = event_hand(&e, &mut s, seat, "TS26_060");
                let targets: Vec<_> = (0..4)
                    .map(|i| {
                        unit(
                            &e,
                            &mut s,
                            if i % 2 == 0 { seat } else { seat.other() },
                            "LOF_256",
                        )
                    })
                    .collect();
                let cost = e.cost(&s, source) as usize;
                event_resources(&e, &mut s, seat, cost);
                let mut prompt = event_click(&e, &s, source);
                // A chosen card is a legal toggle; choosing it twice restores
                // the empty set without spending resources or creating tokens.
                prompt = event_click(&e, &prompt, targets[0]);
                prompt = event_click(&e, &prompt, targets[0]);
                for &id in targets.iter().take(count) {
                    prompt = event_click(&e, &prompt, id);
                }
                if count == 3 {
                    assert!(!e
                        .legal_actions(&prompt)
                        .contains(&e.card_action(&prompt, targets[3])));
                    assert!(e
                        .legal_actions(&prompt)
                        .contains(&e.card_action(&prompt, targets[0])));
                }
                let next = prompt.next_id;
                let done = event_button(
                    &e,
                    &prompt,
                    if count == 0 { "Choose nothing" } else { "Done" },
                );
                assert_eq!(done.next_id, next + count as u16);
                assert_eq!(e.ready_resources(&done, seat), 0);
                assert!(done.player(seat).discard.contains(source));
                for (i, &id) in targets.iter().enumerate() {
                    let token = done.card(id).experience;
                    if i < count {
                        assert_ne!(token, 0);
                        assert_eq!(done.card(token).owner, done.card(id).owner);
                        assert_eq!(done.card(token).parent, id);
                        assert_eq!(e.token_count(&done, token), 1);
                    } else {
                        assert_eq!(token, 0);
                    }
                }
            }
            let mut s = event_board(&e, seat);
            let source = event_hand(&e, &mut s, seat, "TS26_060");
            let cost = e.cost(&s, source) as usize;
            event_resources(&e, &mut s, seat, cost);
            let done = event_finish_empty(&e, event_click(&e, &s, source));
            assert!(done.player(seat).discard.contains(source));
            assert_eq!(e.ready_resources(&done, seat), 0);
        }
    }
}

#[test]
fn self_play_event_experience_dependency_is_checked_in_catalogue_and_selected_pool() {
    let original = event_catalogue();
    let mut malformed = original.clone();
    malformed["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TOKEN_EXPERIENCE")
        .unwrap()["sourceHash"] = json!("unreviewed");
    let rejected = Engine::from_catalogue_value(&malformed).unwrap();
    assert!(!rejected.by_code.contains_key("TS26_060"));
    assert!(
        rejected.by_code.contains_key("HMW_173"),
        "draw event has no Experience dependency"
    );
    let mut missing = original.clone();
    missing["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_EXPERIENCE");
    assert!(!Engine::from_catalogue_value(&missing)
        .unwrap()
        .by_code
        .contains_key("TS26_060"));
    let e = event_engine(RulesBackend::Auto);
    let mut pool = e.compiled_pool().unwrap();
    let keep = [
        "ASH_019",
        "SOR_014",
        "TS26_060",
        "LOF_256",
        "TOKEN_SHIELD",
        "TOKEN_EXPERIENCE",
    ];
    pool["cards"].as_array_mut().unwrap().retain(|c| {
        c["setCode"]
            .as_str()
            .is_some_and(|code| keep.contains(&code))
    });
    let bytes = serde_json::to_vec(&original).unwrap();
    assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes).is_ok());
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_EXPERIENCE");
    let error = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), &bytes)
        .err()
        .unwrap();
    assert!(error.contains("Experience"), "{error}");
}

#[test]
fn self_play_event_leader_upgrade_and_is_leader_producers_stay_unreachable() {
    let e = event_engine(RulesBackend::Auto);
    for seat in [Seat::P1, Seat::P2] {
        let mut s = event_board(&e, seat);
        let host = unit(&e, &mut s, seat, "LAW_253");
        // A fresh physical leader used as an attachment is an invalid import,
        // independently of its printed traits. Do not approximate this source's
        // LeaderUpgrade filter branch as an ordinary Pilot or a base leader.
        let illegal = add(&e, &mut s, seat, "SOR_014");
        e.attach(&mut s, illegal, host);
        assert!(!crate::piloting_attached::metadata_is_pilot_attachment(
            &e.catalogue.as_ref().unwrap().metadata["SOR_014"]
        ));
        assert!(e.load_state(&e.snapshot(&s)).is_err());
    }
    let catalogue = event_catalogue();
    for code in ["SOR_014", "IC27_052"] {
        let mut mixed = catalogue["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["setCode"] == code)
            .unwrap()
            .clone();
        mixed["card"]["types"] = json!(["leader", "upgrade"]);
        let loaded =
            crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[mixed]}))
                .unwrap();
        assert!(
            loaded.rejected.contains_key(code),
            "mixed leader/upgrade {code}"
        );
    }
    let mut leader_shaped_pilot = e.catalogue.as_ref().unwrap().metadata["JTL_045"].clone();
    leader_shaped_pilot["types"] = json!(["leader"]);
    assert!(!crate::piloting_attached::metadata_is_pilot_attachment(
        &leader_shaped_pilot
    ));
    assert!(serde_json::from_value::<crate::ability_ir::Effect>(
        json!({"kind":"isLeader","target":"parent","duration":"persistent"})
    )
    .is_err());
}

#[test]
fn self_play_event_complete_contracts_reject_cost_callback_and_sibling_mutations() {
    let original = event_catalogue();
    for code in ["TS26_060", "HMW_173"] {
        let entry = original["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["setCode"] == code)
            .unwrap();
        let mut variants = vec![];
        for (field, value) in [
            ("sourceZone", json!("hand")),
            ("targetPlayers", json!("self")),
            ("matchSourceIdentity", json!(false)),
            ("matchPrintedType", json!(false)),
            ("stage", json!("payStage_5")),
            ("matchAbilityCosts", json!(true)),
            ("matchCardEffectResourcePayments", json!(true)),
            ("unexpected", json!(true)),
            (
                "attachTargetCondition",
                json!({"kind":"const","value":true}),
            ),
        ] {
            let mut changed = entry.clone();
            changed["abilities"]["card"][0]["effect"][field] = value;
            variants.push((format!("cost {field}"), changed));
        }
        for (field, value) in [
            ("parameters", json!(["otherCard", "adjusterPlayer"])),
            ("timing", json!("registration")),
            ("value", json!({"kind":"const","value":1})),
        ] {
            let mut changed = entry.clone();
            changed["abilities"]["card"][0]["effect"]["amount"][field] = value;
            variants.push((format!("callback {field}"), changed));
        }
        for (field, value) in [
            ("optional", json!(true)),
            ("condition", json!({"kind":"const","value":true})),
            (
                "limit",
                json!({"kind":"abilityLimit","factory":"perRound","args":[1]}),
            ),
            ("then", json!({"effect":{"kind":"noAction"}})),
        ] {
            let mut changed = entry.clone();
            changed["abilities"]["card"][1][field] = value;
            variants.push((format!("event {field}"), changed));
        }
        for (field, value) in [
            ("baseClass", json!("NonLeaderUnitCard")),
            ("stateWatchers", json!([{"watcher":"unreviewed"}])),
            ("inheritedSource", json!({"baseClass":"Unreviewed"})),
            ("privatePrograms", json!([{"name":"unreviewed"}])),
            ("historyHelpers", json!([{"name":"unreviewed"}])),
            ("piloting", json!({})),
        ] {
            let mut changed = entry.clone();
            changed[field] = value;
            variants.push((field.to_owned(), changed));
        }
        for (field, value) in [
            ("types", json!(["event", "unit"])),
            ("unique", json!(true)),
            ("pilotText", json!("Piloting [1]")),
            ("cost", json!(0)),
            ("aspects", json!([])),
            ("traits", json!(["unreviewed"])),
            ("power", json!(2)),
            ("hp", json!(3)),
            ("arena", json!("ground")),
            ("text", json!("Unreviewed cost and event text")),
        ] {
            let mut changed = entry.clone();
            changed["card"][field] = value;
            variants.push((format!("metadata {field}"), changed));
        }
        let mut extra = entry.clone();
        extra["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(json!({"trigger":"event","costs":[],"effect":{"kind":"noAction"}}));
        variants.push(("extra sibling".into(), extra));
        let mut erased = entry.clone();
        erased["abilities"]["card"][1]["effect"] = json!({"kind":"noAction"});
        variants.push(("erased event".into(), erased));
        for (name, changed) in variants {
            assert!(
                crate::self_play_cost::extract(&changed).is_err(),
                "{code}: {name}"
            );
            let input = json!({"schema":"swu-card-ir/v1","cards":[changed]});
            let loaded = crate::ability_ir::Catalogue::load(&input).unwrap();
            assert!(loaded.rejected.contains_key(code), "{code}: {name}");
        }
    }
}

#[test]
fn self_play_event_leader_query_exemption_does_not_admit_other_event_queries() {
    let original = event_catalogue();
    let entry = original["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["setCode"] == "TS26_060")
        .unwrap();
    assert!(crate::self_play_cost::reviewed_leader_count_event(entry));
    let mut changed = entry.clone();
    changed["abilities"]["card"][1]["effect"]["amount"] = json!({
        "kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["player"]},
        "args":[{"kind":"query","name":"isLeader","receiver":{"kind":"ref","path":["card"]},"args":[]}]
    });
    assert!(!crate::self_play_cost::reviewed_leader_count_event(
        &changed
    ));
    let mut input = original.clone();
    *input["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TS26_060")
        .unwrap() = changed;
    assert!(crate::ability_ir::Catalogue::load(&input)
        .unwrap()
        .rejected
        .contains_key("TS26_060"));
    let mut stripped = entry.clone();
    stripped["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    stripped["abilities"]["card"][0]["effect"]["amount"] = json!({
        "kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["player"]},
        "args":[{"kind":"query","name":"isLeader","receiver":{"kind":"ref","path":["card"]},"args":[]}]
    });
    assert!(!crate::self_play_cost::reviewed_leader_count_event(
        &stripped
    ));
    *input["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TS26_060")
        .unwrap() = stripped;
    assert!(crate::ability_ir::Catalogue::load(&input)
        .unwrap()
        .rejected
        .contains_key("TS26_060"));
}

#[test]
fn self_play_event_rebel_discount_removes_aspect_penalties_and_floors_total_at_zero() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = event_engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = event_board_with_leader(&e, seat, "IBH_053");
            let source = event_hand(&e, &mut s, seat, "HMW_173");
            assert_eq!(e.def(&s, source).info.cost(), 4);
            assert_eq!(
                e.cost(&s, source),
                6,
                "missing Heroism adds two before discount"
            );
            unit(&e, &mut s, seat, "LAW_253");
            assert_eq!(e.cost(&s, source), 5);
            for _ in 0..6 {
                unit(&e, &mut s, seat, "LAW_253");
            }
            assert_eq!(
                e.cost(&s, source),
                0,
                "reduction removes the entire cost including aspect penalty"
            );
            for _ in 0..2 {
                let id = add(&e, &mut s, seat, "LOF_256");
                s.player_mut(seat).deck.push(id).unwrap();
            }
            assert_eq!(e.ready_resources(&s, seat), 0);
            let done = event_click(&e, &s, source);
            assert!(done.player(seat).discard.contains(source));
            assert_eq!(done.player(seat).hand.len, 2);
            assert_eq!(e.ready_resources(&done, seat), 0);
        }
    }
}
