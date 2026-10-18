//! Include as a child of generic_runtime_tests. Real-card expectations below are
//! derived from pinned card/helper sources; these are Rust midgame regressions,
//! not forceteki differential recordings or evidence of admission by themselves.
use super::*;
use crate::migration_look_contract::{self as contract, Kind};

fn look_catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn look_row(catalogue: &Value, code: &str) -> Value {
    catalogue["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["setCode"] == code)
        .unwrap()
        .clone()
}
fn look_board(e: &Engine, actor: Seat) -> State {
    let mut s = board(e);
    s.round = 5;
    s.active = actor;
    s.action_player = Some(actor);
    s.phase_history.exact = true;
    s.registration_order.exact = true;
    s.arena_order_exact = true;
    for _ in 0..8 {
        let id = add(e, &mut s, actor, "LOF_254");
        s.player_mut(actor).resources.push(id).unwrap();
    }
    s
}
fn look_hand(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    s.player_mut(owner).hand.push(id).unwrap();
    id
}
fn look_button(e: &Engine, s: &State, label: &str) -> Action {
    e.legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)))
}
fn look_engine(backend: RulesBackend) -> Engine {
    let mut e = engine();
    e.rules_backend = backend;
    e
}
fn look_finish_disclose(e: &Engine, s: &State, donors: &[CardId]) -> State {
    let mut out = *s;
    for &id in donors {
        out = e.apply(&out, &e.card_action(&out, id)).unwrap();
    }
    if out.decision == Decision::DiscloseSelect {
        out = e.apply(&out, &look_button(e, &out, "Done")).unwrap();
    }
    assert_eq!(out.decision, Decision::DiscloseReveal);
    out
}

#[test]
fn migration_look_contract_prepares_three_complete_programs_without_erasing_fields() {
    let catalogue = look_catalogue();
    for (code, kind) in [
        ("TWI_223", Kind::DiscardThenLook),
        ("SEC_230", Kind::DiscloseThenLook),
        ("TS26_080", Kind::ReciprocalLookThenDraw),
    ] {
        let row = look_row(&catalogue, code);
        let prepared = contract::prepare(&row).unwrap();
        assert_eq!(prepared.kind, kind);
        assert_eq!(prepared.registration, row["abilities"]["card"][0]);
        assert!(contract::entry(&row));
        // Card identity and source hash are provenance, never a rules switch.
        let mut renamed = row.clone();
        renamed["setCode"] = json!("SYNTHETIC_001");
        renamed["internalName"] = json!("renamed-complete-source");
        renamed["sourceHash"] = json!("different-source-hash");
        assert!(contract::entry(&renamed));
    }
    for code in ["LAW_066", "LAW_217", "SHD_202", "TS26_026"] {
        assert!(
            contract::prepare(&look_row(&catalogue, code)).is_err(),
            "{code}"
        );
    }
}

#[test]
fn migration_look_integration_admits_complete_candidates_and_rejects_partial_sources() {
    let e = engine();
    for code in ["TWI_223", "SEC_230", "TS26_080"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in ["LAW_066", "LAW_217", "SHD_202", "TS26_026"] {
        assert!(
            !e.by_code.contains_key(code),
            "unsupported companion must remain rejected: {code}"
        );
    }
    let original = look_catalogue();
    for mode in 0..3 {
        let mut catalogue = original.clone();
        let row = catalogue["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["setCode"] == "SEC_230")
            .unwrap();
        match mode {
            0 => row["card"]["types"] = json!(["event", "leader"]),
            1 => {
                row["abilities"]["card"][0]["ifYouDo"]["effect"]["cardCondition"]
                    ["capturedContext"] = json!("unreviewed")
            }
            _ => row["privatePrograms"] = json!([{"unreviewed":true}]),
        }
        let e = Engine::from_catalogue_value(&catalogue).unwrap();
        assert!(!e.by_code.contains_key("SEC_230"), "mutation {mode}");
    }
}

#[test]
fn migration_look_contract_closes_root_and_continuation_headers() {
    let catalogue = look_catalogue();
    let mutations = [
        ("trigger", json!("action")),
        ("optional", json!(true)),
        ("coordinate", json!(true)),
        ("requiresConfirmation", json!(true)),
        ("attackerMustSurvive", json!(true)),
        ("gainedHost", json!(false)),
        ("attachedContext", json!("parent")),
        ("attachedHost", json!(true)),
        ("pilotingAbility", json!(true)),
        ("pilotUnitAbility", json!(true)),
        ("costs", json!([{"kind":"exhaustSelf"}])),
        ("contextTitle", json!([{"kind":"const","value":"changed"}])),
        ("hooks", json!([{"event":"onCardDefeated"}])),
        ("targetResolver", json!({"owner":"any","type":"unit"})),
        ("targetResolvers", json!([{"name":"other"}])),
        ("condition", json!({"kind":"const","value":true})),
        ("ifYouDoCondition", json!({"kind":"eventResult"})),
        (
            "then",
            json!({"title":"extra","effect":{"kind":"noAction"}}),
        ),
        ("futureHeader", json!(false)),
    ];
    for code in ["TWI_223", "SEC_230"] {
        let original = look_row(&catalogue, code);
        for child in [false, true] {
            for (field, value) in &mutations {
                let mut row = original.clone();
                let root = &mut row["abilities"]["card"][0];
                let node = if child { &mut root["ifYouDo"] } else { root };
                node[*field] = value.clone();
                assert!(!contract::entry(&row), "{code} child={child} {field}");
            }
        }
        let mut typed: crate::ability_ir::Ability =
            serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
        assert!(contract::registration(&typed));
        typed.attached_host = true;
        assert!(!contract::registration(&typed));
        typed.attached_host = false;
        typed.piloting_ability = true;
        assert!(!contract::registration(&typed));
        typed.piloting_ability = false;
        typed.pilot_unit_ability = true;
        assert!(!contract::registration(&typed));
        typed.pilot_unit_ability = false;
        for flag in 0..3 {
            let mut bad = typed.clone();
            let child = bad.if_you_do.as_deref_mut().unwrap();
            match flag {
                0 => child.attached_host = true,
                1 => child.piloting_ability = true,
                _ => child.pilot_unit_ability = true,
            }
            assert!(!contract::registration(&bad), "typed child flag {flag}");
        }
    }
}

#[test]
fn migration_look_contract_closes_source_types_companions_and_helper_context() {
    let catalogue = look_catalogue();
    for code in ["TWI_223", "SEC_230", "TS26_080"] {
        let original = look_row(&catalogue, code);
        for types in [
            json!(["unit"]),
            json!(["event", "leader"]),
            json!(["event", "upgrade"]),
            json!(["event", "token"]),
        ] {
            let mut row = original.clone();
            row["card"]["types"] = types;
            assert!(!contract::entry(&row));
        }
        for class in [
            "LeaderUnitCard",
            "TokenUnitCard",
            "UpgradeCard",
            "NonLeaderUnitCard",
        ] {
            let mut row = original.clone();
            row["baseClass"] = json!(class);
            assert!(!contract::entry(&row));
        }
        for field in [
            "historyHelpers",
            "privatePrograms",
            "classMethods",
            "privateClassConstants",
            "inheritedSource",
            "deploymentOverride",
            "setupValidationOverrides",
            "stateWatchers",
        ] {
            let mut row = original.clone();
            row[field] = json!([{"unreviewed":true}]);
            assert!(!contract::entry(&row), "{field}");
        }
        for side in ["card", "unit", "leader"] {
            let mut row = original.clone();
            let extra = row["abilities"]["card"][0].clone();
            row["abilities"][side].as_array_mut().unwrap().push(extra);
            assert!(!contract::entry(&row), "extra {side}");
        }
        for (field, value) in [
            ("cost", json!(-1)),
            ("cost", json!(1001)),
            ("power", json!(0)),
            ("hp", json!(1)),
            ("arena", json!("ground")),
            ("unique", json!(null)),
            ("keywords", json!(["smuggle"])),
        ] {
            let mut row = original.clone();
            row["card"][field] = value;
            assert!(!contract::entry(&row), "{field}");
        }
    }
}

#[test]
fn migration_look_contract_rejects_wrong_selection_identity_optionality_and_hidden_results() {
    let catalogue = look_catalogue();
    for code in ["TWI_223", "SEC_230"] {
        let row = look_row(&catalogue, code);
        let base = row["abilities"]["card"][0]["ifYouDo"]["effect"].clone();
        assert!(contract::selection(&base));
        for (field, value) in [
            (
                "target",
                json!({"kind":"ref","path":["opponent","resources"]}),
            ),
            ("player", json!({"kind":"ref","path":["opponent"]})),
            ("maxCards", json!({"kind":"const","value":2})),
            ("maxCards", json!({"kind":"const","value":0})),
            ("canChooseFewer", json!({"kind":"ref","path":["player"]})),
            ("useDisplayPrompt", json!(false)),
            ("selectedContextField", json!("target")),
            (
                "immediateEffect",
                json!({"kind":"discardSpecificCard","target":"chosen"}),
            ),
            (
                "cardCondition",
                json!({"kind":"systemValue","evaluation":"viewSelectionFilter","value":{"kind":"ref","path":["target","aspects"]}}),
            ),
            (
                "activePromptTitleParts",
                json!([{"kind":"ref","path":["target"]}]),
            ),
            ("newCallback", json!({"kind":"const","value":true})),
        ] {
            let mut bad = base.clone();
            bad[field] = value;
            assert!(!contract::selection(&bad), "{field}");
        }
        let mut row = row.clone();
        row["abilities"]["card"][0]["ifYouDo"]["effect"]["canChooseFewer"]["value"] =
            json!(code != "TWI_223");
        assert!(
            !contract::entry(&row),
            "source optionality must not be inferred from printed prose"
        );
        let mut bad = base.clone();
        bad["cardCondition"]["capturedContext"] = json!("other");
        assert!(!contract::selection(&bad));
    }
    let base = look_row(&catalogue, "TS26_080");
    for slot in [0usize, 1, 2] {
        let mut row = base.clone();
        row["abilities"]["card"][0]["effect"]["effects"][slot]["target"] = json!("ownPlayer");
        assert!(!contract::entry(&row), "reciprocal slot {slot}");
    }
}

#[test]
fn migration_look_unmasking_commits_own_discard_then_selects_or_declines_for_both_seats() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = look_engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            for decline in [false, true] {
                let mut s = look_board(&e, actor);
                let event = look_hand(&e, &mut s, actor, "TWI_223");
                let own = look_hand(&e, &mut s, actor, "LOF_254");
                let enemy = look_hand(&e, &mut s, actor.other(), "LOF_256");
                let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
                assert_eq!(prompt.decision, Decision::AbilityDiscard);
                assert_eq!(prompt.active, actor);
                assert!(prompt.player(actor).discard.contains(event));
                assert_eq!(
                    e.legal_actions(&prompt)
                        .iter()
                        .filter_map(Action::id)
                        .collect::<Vec<_>>(),
                    vec![own]
                );
                let prompt = e.apply(&prompt, &e.card_action(&prompt, own)).unwrap();
                assert_eq!(prompt.decision, Decision::AbilityDisplay);
                assert_eq!(prompt.active, actor);
                assert!(prompt.player(actor).discard.contains(own));
                assert!(prompt.player(actor).discard.contains(event));
                assert!(e
                    .legal_actions(&prompt)
                    .contains(&Action::button("Take nothing", "done")));
                assert!(e.apply(&prompt, &e.card_action(&prompt, event)).is_err());
                let choice = if decline {
                    look_button(&e, &prompt, "Take nothing")
                } else {
                    e.card_action(&prompt, enemy)
                };
                let out = e.apply(&prompt, &choice).unwrap();
                assert_eq!(out.decision, Decision::Window);
                assert_eq!(out.active, actor.other());
                assert_eq!(out.player(actor.other()).hand.contains(enemy), decline);
                assert_eq!(out.player(actor.other()).discard.contains(enemy), !decline);
                assert!(out.player(actor).discard.contains(event));
            }
        }
    }
}

#[test]
fn migration_look_unmasking_empty_hands_do_not_invent_success() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        let mut s = look_board(&e, actor);
        let event = look_hand(&e, &mut s, actor, "TWI_223");
        let enemy = look_hand(&e, &mut s, actor.other(), "LOF_256");
        // Feasibility sees this event in hand before payment. Commitment removes
        // it before discard resolution; no card remains to discard or look at.
        let out = e.apply(&s, &e.card_action(&s, event)).unwrap();
        assert_eq!(out.decision, Decision::Window);
        assert!(out.player(actor).discard.contains(event));
        assert!(out.player(actor.other()).hand.contains(enemy));
        let mut s = look_board(&e, actor);
        let event = look_hand(&e, &mut s, actor, "TWI_223");
        let own = look_hand(&e, &mut s, actor, "LOF_254");
        let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
        let out = e.apply(&prompt, &e.card_action(&prompt, own)).unwrap();
        assert_eq!(out.decision, Decision::Window);
        assert!(out.player(actor).discard.contains(own));
        assert_eq!(out.player(actor.other()).hand.len, 0);
    }
}

#[test]
fn migration_look_charged_reveal_acknowledgment_precedes_mandatory_unit_selection() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = look_engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            let mut s = look_board(&e, actor);
            let event = look_hand(&e, &mut s, actor, "SEC_230");
            let donors = [
                look_hand(&e, &mut s, actor, "SEC_230"),
                look_hand(&e, &mut s, actor, "SEC_230"),
            ];
            let unit = look_hand(&e, &mut s, actor.other(), "LOF_256");
            let nonunit = look_hand(&e, &mut s, actor.other(), "ASH_115");
            let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
            assert_eq!(prompt.decision, Decision::DiscloseSelect);
            assert!(prompt.player(actor).discard.contains(event));
            assert!(!e
                .legal_actions(&prompt)
                .iter()
                .any(|a| a.id() == Some(event)));
            let reveal = look_finish_disclose(&e, &prompt, &donors);
            assert_eq!(reveal.active, actor.other());
            assert!(reveal.player(actor.other()).hand.contains(unit));
            assert_eq!(
                e.legal_actions(&reveal),
                vec![Action::button("Done", "done")]
            );
            let prompt = e.apply(&reveal, &look_button(&e, &reveal, "Done")).unwrap();
            assert_eq!(prompt.decision, Decision::AbilityDisplay);
            assert_eq!(prompt.active, actor);
            assert_eq!(e.legal_actions(&prompt), vec![e.card_action(&prompt, unit)]);
            assert!(e.apply(&prompt, &Action::button("Done", "done")).is_err());
            assert!(e.apply(&prompt, &e.card_action(&prompt, nonunit)).is_err());
            let out = e.apply(&prompt, &e.card_action(&prompt, unit)).unwrap();
            assert!(out.player(actor.other()).discard.contains(unit));
            assert!(out.player(actor.other()).hand.contains(nonunit));
            assert!(donors.iter().all(|id| out.player(actor).hand.contains(*id)));
            assert_eq!(out.disclose, DiscloseState::default());
        }
    }
}

#[test]
fn migration_look_charged_single_double_cunning_card_auto_advances_to_reveal() {
    let e = engine();
    let mut s = look_board(&e, Seat::P2);
    let event = look_hand(&e, &mut s, Seat::P2, "SEC_230");
    // Bodhi Rook has two printed Cunning icons; disclosure counts icons, not cards.
    let donor = look_hand(&e, &mut s, Seat::P2, "SOR_201");
    let enemy = look_hand(&e, &mut s, Seat::P1, "LOF_256");
    let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
    let reveal = e.apply(&prompt, &e.card_action(&prompt, donor)).unwrap();
    assert_eq!(reveal.decision, Decision::DiscloseReveal);
    assert_eq!(reveal.active, Seat::P1);
    assert!(reveal.players[1].discard.contains(event));
    assert!(reveal.players[1].hand.contains(donor));
    let prompt = e.apply(&reveal, &look_button(&e, &reveal, "Done")).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityDisplay);
    assert_eq!(prompt.active, Seat::P2);
    let out = e.apply(&prompt, &e.card_action(&prompt, enemy)).unwrap();
    assert!(out.players[0].discard.contains(enemy));
    assert_eq!(out.decision, Decision::Window);
}

#[test]
fn migration_look_charged_decline_and_source_departure_do_not_leak_opponent_hand() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        for decline in ["Cancel", "Choose nothing"] {
            let mut s = look_board(&e, actor);
            let event = look_hand(&e, &mut s, actor, "SEC_230");
            look_hand(&e, &mut s, actor, "SEC_230");
            look_hand(&e, &mut s, actor, "SEC_230");
            let enemy = look_hand(&e, &mut s, actor.other(), "LOF_256");
            let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
            let out = e
                .apply(&prompt, &look_button(&e, &prompt, decline))
                .unwrap();
            assert_eq!(out.decision, Decision::Window);
            assert!(out.player(actor).discard.contains(event));
            assert!(out.player(actor.other()).hand.contains(enemy));
        }
        // Before commit, source plus donor meet two Cunning. After commit the
        // source cannot disclose itself; the continuation must stay skipped.
        let mut s = look_board(&e, actor);
        let event = look_hand(&e, &mut s, actor, "SEC_230");
        let donor = look_hand(&e, &mut s, actor, "SEC_230");
        let enemy = look_hand(&e, &mut s, actor.other(), "LOF_256");
        let out = e.apply(&s, &e.card_action(&s, event)).unwrap();
        assert_eq!(out.decision, Decision::Window);
        assert!(out.player(actor).discard.contains(event));
        assert!(out.player(actor).hand.contains(donor));
        assert!(out.player(actor.other()).hand.contains(enemy));
    }
}

#[test]
fn migration_look_charged_empty_and_nonunit_only_hands_keep_distinct_prompts() {
    let e = engine();
    for nonunit in [false, true] {
        let mut s = look_board(&e, Seat::P1);
        let event = look_hand(&e, &mut s, Seat::P1, "SEC_230");
        let donors = [
            look_hand(&e, &mut s, Seat::P1, "SEC_230"),
            look_hand(&e, &mut s, Seat::P1, "SEC_230"),
        ];
        let card = nonunit.then(|| look_hand(&e, &mut s, Seat::P2, "ASH_115"));
        let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
        let reveal = look_finish_disclose(&e, &prompt, &donors);
        let out = e.apply(&reveal, &look_button(&e, &reveal, "Done")).unwrap();
        if let Some(card) = card {
            assert_eq!(out.decision, Decision::AbilityDisplay);
            assert_eq!(e.legal_actions(&out), vec![Action::button("Done", "done")]);
            let done = e.apply(&out, &look_button(&e, &out, "Done")).unwrap();
            assert!(done.players[1].hand.contains(card));
            assert_eq!(done.decision, Decision::Window);
        } else {
            assert_eq!(out.decision, Decision::Window);
        }
    }
}

#[test]
fn migration_look_reveal_intentions_switches_chooser_then_draws_for_both_once() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = look_engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            let mut s = look_board(&e, actor);
            let event = look_hand(&e, &mut s, actor, "TS26_080");
            let own = look_hand(&e, &mut s, actor, "LOF_254");
            let enemy = look_hand(&e, &mut s, actor.other(), "LOF_256");
            let own_draw = add(&e, &mut s, actor, "ASH_131");
            let enemy_draw = add(&e, &mut s, actor.other(), "ASH_131");
            s.player_mut(actor).deck.push(own_draw).unwrap();
            s.player_mut(actor.other()).deck.push(enemy_draw).unwrap();
            let first = e.apply(&s, &e.card_action(&s, event)).unwrap();
            assert_eq!(first.decision, Decision::AbilityDisplay);
            assert_eq!(first.active, actor);
            assert_eq!(e.legal_actions(&first), vec![e.card_action(&first, enemy)]);
            let second = e.apply(&first, &e.card_action(&first, enemy)).unwrap();
            assert_eq!(second.decision, Decision::AbilityDisplay);
            assert_eq!(second.active, actor.other());
            assert_eq!(e.legal_actions(&second), vec![e.card_action(&second, own)]);
            assert!(second.player(actor.other()).discard.contains(enemy));
            assert!(second.player(actor).deck.contains(own_draw));
            assert!(second.player(actor.other()).deck.contains(enemy_draw));
            let out = e.apply(&second, &e.card_action(&second, own)).unwrap();
            assert_eq!(out.decision, Decision::Window);
            assert_eq!(out.active, actor.other());
            assert!(out.player(actor).discard.contains(own));
            assert!(out.player(actor).discard.contains(event));
            assert!(out.player(actor).hand.contains(own_draw));
            assert!(out.player(actor.other()).hand.contains(enemy_draw));
            assert_eq!(out.player(actor).deck.len, 0);
            assert_eq!(out.player(actor.other()).deck.len, 0);
        }
    }
}

#[test]
fn migration_look_reciprocal_empty_short_and_full_sets_do_not_inherit_prior_result() {
    let e = engine();
    for (own_count, enemy_count) in [
        (0usize, 0usize),
        (0, 1),
        (1, 0),
        (1, 1),
        (crate::model::MAX_ZONE - 1, crate::model::MAX_ZONE),
    ] {
        let mut s = look_board(&e, Seat::P1);
        let event = look_hand(&e, &mut s, Seat::P1, "TS26_080");
        let own: Vec<_> = (0..own_count)
            .map(|_| look_hand(&e, &mut s, Seat::P1, "LOF_254"))
            .collect();
        let enemy: Vec<_> = (0..enemy_count)
            .map(|_| look_hand(&e, &mut s, Seat::P2, "LOF_256"))
            .collect();
        for seat in [Seat::P1, Seat::P2] {
            let draw = add(&e, &mut s, seat, "ASH_131");
            s.player_mut(seat).deck.push(draw).unwrap();
        }
        let mut out = e.apply(&s, &e.card_action(&s, event)).unwrap();
        for (actor, cards) in [(Seat::P1, &enemy), (Seat::P2, &own)] {
            if cards.is_empty() {
                continue;
            }
            assert_eq!(out.decision, Decision::AbilityDisplay);
            assert_eq!(out.active, actor);
            assert_eq!(
                e.legal_actions(&out)
                    .iter()
                    .filter_map(Action::id)
                    .collect::<Vec<_>>(),
                *cards
            );
            out = e.apply(&out, &e.card_action(&out, cards[0])).unwrap();
        }
        assert_eq!(out.decision, Decision::Window);
        assert_eq!(
            out.players[0].hand.len as usize,
            own_count.saturating_sub(1) + 1
        );
        assert_eq!(
            out.players[1].hand.len as usize,
            enemy_count.saturating_sub(1) + 1
        );
        assert_eq!(out.players[0].deck.len, 0);
        assert_eq!(out.players[1].deck.len, 0);
    }
}

#[test]
fn migration_look_pending_snapshot_cannot_be_reimported_as_fresh_or_terminal_window() {
    let e = engine();
    let mut s = look_board(&e, Seat::P1);
    let event = look_hand(&e, &mut s, Seat::P1, "TS26_080");
    look_hand(&e, &mut s, Seat::P1, "LOF_254");
    let enemy = look_hand(&e, &mut s, Seat::P2, "LOF_256");
    let first = e.apply(&s, &e.card_action(&s, event)).unwrap();
    let second = e.apply(&first, &e.card_action(&first, enemy)).unwrap();
    for pending in [first, second] {
        let snapshot = e.snapshot(&pending);
        assert_eq!(snapshot["viewSelectionPending"], json!(true));
        assert!(e.load_state(&snapshot).is_err());
        for marker in [json!(true), json!(false), Value::Null] {
            let mut forged = snapshot.clone();
            forged["viewSelectionPending"] = marker;
            forged["actor"] = forged["activePlayer"].clone();
            assert!(e.load_state(&forged).is_err());
            forged["winner"] = json!("p1");
            assert!(e.load_state(&forged).is_err());
        }
    }
    let snapshot = e.snapshot(&s);
    assert!(snapshot.get("viewSelectionPending").is_none());
    assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()), snapshot);
}
