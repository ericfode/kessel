use super::*;

fn choose(e: &Engine, s: &State, label: &str) -> State {
    let action = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)));
    e.apply(s, &action).unwrap()
}
fn start(e: &Engine, s: &mut State, source: CardId) {
    s.active = s.card(source).owner;
    s.source = source;
    let program = e.def(s, source).programs.card[0];
    e.begin_program(s, program, 0).unwrap();
}
fn labels(e: &Engine, s: &State) -> Vec<String> {
    e.legal_actions(s).into_iter().map(|a| a.label).collect()
}
fn catalog_entry(code: &str) -> Value {
    let value: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    value["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["setCode"] == code)
        .unwrap()
        .clone()
}

#[test]
fn modal_choices_admit_complete_closed_sources_and_reject_other_families() {
    let e = engine();
    for code in [
        "SOR_145", "SOR_189", "LAW_067", "ASH_147", "LOF_065", "TWI_102",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in [
        "SHD_191", "SEC_193", "SEC_220", "LAW_080", "LAW_086", "LAW_101",
        "JTL_131", "HMW_054", "HMW_188", "ASH_045", "ASH_257",
    ] {
        assert!(
            !e.by_code.contains_key(code),
            "unreviewed modal family {code}"
        );
    }
}

// K-2SO's controller chooses the mode even after K-2SO leaves play. The
// opponent, rather than the controller, chooses the discarded physical card.
#[test]
fn modal_k2so_death_preserves_owner_and_both_branch_effects() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        for discard in [false, true] {
            let mut s = board(&e);
            let source = unit(&e, &mut s, owner, "SOR_145");
            let first = add(&e, &mut s, owner.other(), "LOF_256");
            let second = add(&e, &mut s, owner.other(), "LOF_254");
            s.player_mut(owner.other()).hand.push(first).unwrap();
            s.player_mut(owner.other()).hand.push(second).unwrap();
            e.defeat(&mut s, source).unwrap();
            e.finish(&mut s).unwrap();
            assert!(s.player(owner).discard.contains(source));
            assert_eq!(s.active, owner);
            assert_eq!(
                labels(&e, &s),
                [
                    "Deal 3 damage to opponent's base",
                    "The opponent discards a card"
                ]
            );
            for (index, a) in e.legal_actions(&s).iter().enumerate() {
                assert_eq!(a.arg, Some(json!(index)));
            }
            let before = s.card(s.player(owner.other()).base).damage;
            if discard {
                let picking = choose(&e, &s, "The opponent discards a card");
                assert_eq!(picking.decision, Decision::AbilityDiscard);
                assert_eq!(picking.active, owner.other());
                assert!(e
                    .legal_actions(&picking)
                    .contains(&e.card_action(&picking, second)));
                let done = e.apply(&picking, &e.card_action(&picking, second)).unwrap();
                assert!(done.player(owner.other()).discard.contains(second));
                assert!(done.player(owner.other()).hand.contains(first));
                assert_eq!(done.card(done.player(owner.other()).base).damage, before);
            } else {
                let done = choose(&e, &s, "Deal 3 damage to opponent's base");
                assert_eq!(
                    done.card(done.player(owner.other()).base).damage,
                    before + 3
                );
                assert_eq!(done.player(owner.other()).hand.len, 2);
            }
        }
    }
}

// SelectTargetResolver tests branch legality without requiring a state change.
// An empty hand remains a legal mode while the base-damage mode can resolve.
#[test]
fn modal_k2so_empty_hand_keeps_discard_label_without_inheriting_damage() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SOR_145");
    e.defeat(&mut s, source).unwrap();
    e.finish(&mut s).unwrap();
    assert_eq!(labels(&e, &s).len(), 2);
    let before = s.card(s.players[1].base).damage;
    let done = choose(&e, &s, "The opponent discards a card");
    assert_eq!(done.card(done.players[1].base).damage, before);
    assert!(done.players[1].hand.len == 0);
    assert_eq!(done.ability_program, 0);
}

#[test]
fn modal_leia_modes_ready_the_named_player_and_exhaust_the_selected_unit() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        for mode in 0..3 {
            let mut s = board(&e);
            let source = unit(&e, &mut s, owner, "SOR_189");
            let enemy = unit(&e, &mut s, owner.other(), "LOF_256");
            let own_resource = add(&e, &mut s, owner, "LOF_254");
            let enemy_resource = add(&e, &mut s, owner.other(), "LOF_254");
            for (seat, id) in [(owner, own_resource), (owner.other(), enemy_resource)] {
                s.player_mut(seat).resources.push(id).unwrap();
                s.card_mut(id).exhausted = true;
            }
            start(&e, &mut s, source);
            let expected = [
                "Ready a friendly resource",
                "Ready an enemy resource",
                "Exhaust a unit",
            ];
            assert_eq!(labels(&e, &s), expected);
            let mut done = choose(&e, &s, expected[mode]);
            if mode == 2 {
                assert_eq!(done.active, owner);
                assert!(!labels(&e, &done).iter().any(|s| s == "Cancel"));
                done = e.apply(&done, &e.card_action(&done, enemy)).unwrap();
            }
            assert_eq!(done.card(own_resource).exhausted, mode != 0);
            assert_eq!(done.card(enemy_resource).exhausted, mode != 1);
            assert_eq!(done.card(enemy).exhausted, mode == 2);
        }
    }
}

#[test]
fn modal_jyn_experience_and_selected_but_ineffective_exhaust_are_distinct() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LAW_067");
    let target = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(source).exhausted = true;
    s.card_mut(target).exhausted = true;
    start(&e, &mut s, source);
    assert_eq!(
        labels(&e, &s),
        ["Give an Experience token to a unit", "Exhaust a unit"]
    );
    let before = s.next_id;
    let unchanged = choose(&e, &s, "Exhaust a unit");
    assert_eq!(unchanged.next_id, before);
    assert_eq!(unchanged.ability_program, 0);
    let picking = choose(&e, &s, "Give an Experience token to a unit");
    let done = e.apply(&picking, &e.card_action(&picking, target)).unwrap();
    assert_eq!(done.next_id, before + 1);
    assert_eq!(done.card(before).parent, target);
    assert_eq!(done.card(before).owner, Seat::P2);
}

#[test]
fn modal_cyborg_filters_modes_auto_resolves_one_and_keeps_ground_damage_partition() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "ASH_147");
    let target = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(target).phase_hp = 20;
    let space = unit(&e, &mut s, Seat::P2, "LOF_034");
    s.card_mut(space).damage = 1;
    start(&e, &mut s, source);
    // Only undamaged ground units exist: the one legal mode auto-selects,
    // leaving a card prompt, with no redundant mode button.
    assert_eq!(s.decision, Decision::AbilityTarget);
    assert!(e.legal_actions(&s).contains(&e.card_action(&s, target)));
    assert!(!labels(&e, &s)
        .iter()
        .any(|s| s.starts_with("Deal 2 damage")));
    assert!(!e.legal_actions(&s).contains(&e.card_action(&s, space)));
    let done = e.apply(&s, &e.card_action(&s, target)).unwrap();
    assert_eq!(done.card(target).damage, 2);
    let mut second = done;
    start(&e, &mut second, source);
    assert_eq!(
        labels(&e, &second),
        [
            "Deal 2 damage to an undamaged ground unit",
            "Deal 5 damage to a damaged ground unit"
        ]
    );
    let picking = choose(&e, &second, "Deal 5 damage to a damaged ground unit");
    assert!(!e
        .legal_actions(&picking)
        .contains(&e.card_action(&picking, source)));
    let final_state = e.apply(&picking, &e.card_action(&picking, target)).unwrap();
    assert_eq!(final_state.card(target).damage, 7);
}

#[test]
fn modal_watto_opponent_chooses_mode_controller_chooses_target_and_draws() {
    let e = engine();
    for owner in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, owner, "LOF_065");
        let ally = unit(&e, &mut s, owner, "LOF_256");
        let enemy = unit(&e, &mut s, owner.other(), "LOF_256");
        let drawn = add(&e, &mut s, owner, "LOF_254");
        s.player_mut(owner).deck.push(drawn).unwrap();
        start(&e, &mut s, source);
        assert_eq!(s.active, owner.other());
        assert_eq!(
            labels(&e, &s),
            [
                "Opponent give an Experience token to a friendly unit",
                "Opponent draws a card"
            ]
        );
        let copied = s;
        assert_eq!(copied.zobrist(), s.zobrist());
        let picking = choose(
            &e,
            &copied,
            "Opponent give an Experience token to a friendly unit",
        );
        assert_eq!(picking.active, owner);
        assert!(e
            .legal_actions(&picking)
            .contains(&e.card_action(&picking, ally)));
        assert!(!e
            .legal_actions(&picking)
            .contains(&e.card_action(&picking, enemy)));
        let done = e.apply(&picking, &e.card_action(&picking, ally)).unwrap();
        assert_ne!(done.card(ally).experience, 0);
        let draw = choose(&e, &s, "Opponent draws a card");
        assert!(draw.player(owner).hand.contains(drawn));
        assert!(draw.player(owner.other()).hand.len == 0);
    }
}

#[test]
fn modal_manufactured_soldiers_choose_before_payment_then_create_exact_exhausted_batch() {
    let e = engine();
    for (label, count, token) in [
        ("Create 2 Clone Trooper tokens", 2, "TOKEN_CLONE_TROOPER"),
        ("Create 3 Battle Droid tokens", 3, "TOKEN_BATTLE_DROID"),
    ] {
        let mut s = board(&e);
        let source = hand(&e, &mut s, "TWI_102");
        let next = s.next_id;
        let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert!(prompt.players[0].hand.contains(source));
        assert_eq!(
            e.ready_resources(&prompt, Seat::P1),
            e.ready_resources(&s, Seat::P1)
        );
        assert_eq!(
            labels(&e, &prompt),
            [
                "Create 2 Clone Trooper tokens",
                "Create 3 Battle Droid tokens"
            ]
        );
        let done = choose(&e, &prompt, label);
        assert!(done.players[0].discard.contains(source));
        assert_eq!(done.next_id, next + count);
        assert_eq!(
            e.ready_resources(&done, Seat::P1),
            e.ready_resources(&s, Seat::P1) - usize::from(e.cost(&s, source))
        );
        for id in next..done.next_id {
            assert_eq!(e.def(&done, id).code, token);
            assert_eq!(done.card(id).owner, Seat::P1);
            assert!(done.card(id).exhausted);
            assert!(done.players[0].ground.contains(id));
        }
    }
}

#[test]
fn modal_closed_contract_rejects_pending_branches_wrong_bindings_and_partial_cards() {
    let original = catalog_entry("SOR_145");
    for (pointer, replacement) in [
        (
            "/abilities/card/0/targetResolver/resultBinding",
            json!("wrong"),
        ),
        (
            "/abilities/card/0/targetResolver/chooser",
            json!({"kind":"ref","path":["player"]}),
        ),
        (
            "/abilities/card/0/targetResolver/choices/contextBinding",
            json!("snapshot"),
        ),
        (
            "/abilities/card/0/targetResolver/choices/entries/0/labelParts/0",
            json!({"kind":"ref","path":["source","title"]}),
        ),
        (
            "/abilities/card/0/targetResolver/choices/entries/0/effect",
            json!({"kind":"capture","target":"chosen"}),
        ),
        (
            "/abilities/card/0/ifYouDo",
            json!({"title":"later","effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}),
        ),
        ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
        ("/baseClass", json!("LeaderUnitCard")),
        ("/card/types", json!(["unit", "leader"])),
    ] {
        let mut entry = original.clone();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        let parent = entry.pointer_mut(parent).unwrap();
        if let Some(object) = parent.as_object_mut() {
            object.insert(key.to_owned(), replacement);
        } else {
            parent[key.parse::<usize>().unwrap()] = replacement;
        }
        assert!(
            crate::modal_choices::prepare(&entry).is_err(),
            "accepted mutation {pointer}"
        );
    }
    let mut sibling = original.clone();
    sibling["abilities"]["card"].as_array_mut().unwrap().push(
        json!({"trigger":"whenPlayed","title":"unsupported sibling","effect":{"kind":"noAction"}}),
    );
    assert!(crate::modal_choices::prepare(&sibling).is_err());
    let mut extra = original;
    extra["abilities"]["card"][0]["targetResolver"]["showUnresolvable"] = json!(true);
    assert!(crate::modal_choices::prepare(&extra).is_err());
}

#[test]
fn modal_property_order_duplicates_and_text_bounds_follow_pinned_contract() {
    let mut entry = catalog_entry("SOR_145");
    let entries = entry["abilities"]["card"][0]["targetResolver"]["choices"]["entries"]
        .as_array_mut()
        .unwrap();
    let mut replacement = entries[0].clone();
    entries[0]["labelParts"][0]["value"] = json!("2");
    entries[1]["labelParts"][0]["value"] = json!("1");
    replacement["labelParts"][0]["value"] = json!("2");
    replacement["effect"]["amount"]["value"] = json!(1);
    entries.push(replacement);
    let prepared = crate::modal_choices::prepare(&entry).unwrap().unwrap();
    let ability =
        serde_json::from_value(prepared.normalized["abilities"]["card"][0].clone()).unwrap();
    let program = prepared.compile(&ability).unwrap();
    let Instruction::ModalChoice(menu) = &program.instructions[0] else {
        panic!("missing modal instruction")
    };
    assert_eq!(
        menu.branches
            .iter()
            .map(|b| b.label.as_str())
            .collect::<Vec<_>>(),
        ["1", "2"]
    );
    let branch = &menu.branches[1];
    assert!(program.instructions[branch.start..branch.end].iter().any(|instruction| matches!(instruction,
        Instruction::Apply(Effect::Damage { amount: Expr::Const { value }, .. }) if value.as_u64() == Some(1))));
    let mut invalid = catalog_entry("SOR_145");
    invalid["abilities"]["card"][0]["targetResolver"]["choices"]["entries"][0]["labelParts"][0]
        ["value"] = json!("x".repeat(1025));
    assert!(crate::modal_choices::prepare(&invalid).is_err());
}

#[test]
fn modal_leia_root_still_prompts_when_all_legal_modes_are_ineffective() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SOR_189");
    s.card_mut(source).exhausted = true;
    // No resources can ready, and the only unit is already exhausted. The
    // pinned modal resolver still finds legal player/unit targets at None.
    start(&e, &mut s, source);
    assert_eq!(
        labels(&e, &s),
        [
            "Ready a friendly resource",
            "Ready an enemy resource",
            "Exhaust a unit"
        ]
    );
    let next_id = s.next_id;
    let done = choose(&e, &s, "Exhaust a unit");
    assert_eq!(done.ability_program, 0);
    assert_eq!(done.next_id, next_id);
    assert!(done.card(source).exhausted);
    let mut event = catalog_entry("TWI_102");
    event["abilities"]["card"][0]["targetResolver"]["chooser"]["value"] = json!("opponent");
    assert!(crate::modal_choices::prepare(&event).is_err());
}

#[test]
fn modal_catalogue_rejects_unexecuted_source_context_before_normalization() {
    for code in [
        "SOR_145", "SOR_189", "LAW_067", "ASH_147", "LOF_065", "TWI_102",
    ] {
        let original = catalog_entry(code);
        let load = |entry: Value| {
            crate::ability_ir::Catalogue::load(&json!({
                "schema": crate::ability_ir::SCHEMA,
                "cards": [entry]
            }))
            .unwrap()
        };
        assert!(
            load(original.clone()).cards.contains_key(code),
            "baseline {code}"
        );
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
            // These fields carry complete source contracts beyond the emitted
            // ability. Leaving one unused must reject the containing card.
            for context in [
                json!([{"kind":"unreviewedWatcher"}]),
                json!([{}]),
                json!({}),
                json!(true),
            ] {
                let mut entry = original.clone();
                entry[key] = context.clone();
                let loaded = load(entry);
                assert!(
                    !loaded.cards.contains_key(code),
                    "{code}: accepted {key}={context}"
                );
                assert!(
                    loaded.rejected.contains_key(code),
                    "{code}: rejection missing for {key}"
                );
            }
            let mut empty = original.clone();
            empty[key] = json!([]);
            assert!(load(empty).cards.contains_key(code), "{code}: empty {key}");
        }
    }
}
