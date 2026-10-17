//! Source-derived witnesses for the Bounty keyword family.
//!
//! Pinned engine reading order: `abilities/keyword/BountyAbility.ts` (who may
//! trigger, optionality, always-collect), `core/card/propertyMixins/UnitProperties.ts`
//! (registration at defeat/capture, one binding per printed instance),
//! `core/gameSteps/abilityWindow/TriggerWindowBase.ts` (player buckets, SWU 7.6.10
//! active-player order), `core/ability/PlayerOrCardAbility.ts` (context controller)
//! and `compiler/bounty-registration.js` (the reviewed IR metadata).
use super::*;

fn decks(e: &Engine, s: &mut State, seat: Seat, count: usize) -> Vec<CardId> {
    (0..count)
        .map(|_| {
            let id = add(e, s, seat, "LOF_254");
            s.player_mut(seat).deck.push(id).unwrap();
            id
        })
        .collect()
}

fn experience(e: &Engine, s: &State, id: CardId) -> u16 {
    e.token_count(s, s.card(id).experience)
}

fn defeated(e: &Engine, mut s: State, id: CardId) -> State {
    e.defeat(&mut s, id).unwrap();
    e.finish(&mut s).unwrap();
    s
}

#[test]
fn bounty_family_admits_printed_registrations_and_keeps_mentions_metadata_only() {
    let e = engine();
    for code in [
        "SHD_027", "SHD_058", "SHD_095", "SHD_116", "SHD_134", "SHD_167", "SHD_185", "SHD_195",
        "SHD_211",
    ] {
        assert!(e.by_code.contains_key(code), "{code}: {:?}", e.rejected_cards().unwrap().get(code));
        let d = &e.definitions[*e.by_code.get(code).unwrap() as usize];
        let programs: Vec<_> = d.programs.card.iter().map(|program| e.program(*program)).collect();
        let bounty: Vec<_> = programs.iter().filter(|program| program.trigger == "bounty").collect();
        assert_eq!(bounty.len(), 1, "{code}");
        assert!(bounty[0].title.starts_with("Collect Bounty: "), "{code}");
        assert_eq!(bounty[0].optional, code != "SHD_185", "{code}");
        assert!(bounty[0].costs.is_empty(), "{code}");
    }
    for code in ["SHD_139", "SHD_140", "SHD_171"] {
        assert!(e.by_code.contains_key(code), "{code}: {:?}", e.rejected_cards().unwrap().get(code));
        let d = &e.definitions[*e.by_code.get(code).unwrap() as usize];
        assert!(!d.programs.card.iter().any(|program| e.program(*program).trigger == "bounty"), "{code}");
    }
    // The provider keyword list mentions bounty on unrelated cards; the intrinsic
    // instance follows the printed own-line keyword, exactly like the engine parser.
    let mut s = board(&e);
    let hylobon = unit(&e, &mut s, Seat::P1, "SHD_027");
    let hunters = unit(&e, &mut s, Seat::P2, "SHD_140");
    let krrsantan = unit(&e, &mut s, Seat::P1, "SHD_139");
    let rivals = unit(&e, &mut s, Seat::P2, "SHD_171");
    assert!(e.has_keyword(&s, hylobon, "bounty"));
    assert!(e.has_keyword(&s, hylobon, "grit"), "neighbouring printed keywords stay intact");
    assert!(!e.has_keyword(&s, hunters, "bounty"), "a condition mention is not the keyword");
    assert!(!e.has_keyword(&s, krrsantan, "bounty"));
    assert!(!e.has_keyword(&s, rivals, "bounty"));
}

#[test]
fn bounty_defeat_prompt_belongs_to_the_opponent_and_draws_for_the_collector() {
    let e = engine();
    let mut s = board(&e);
    let p1_deck = decks(&e, &mut s, Seat::P1, 2);
    let p2_deck = decks(&e, &mut s, Seat::P2, 2);
    let source = unit(&e, &mut s, Seat::P1, "SHD_027");
    unit(&e, &mut s, Seat::P2, "LOF_256");
    let (p1_hand, p2_hand) = (s.players[0].hand.len, s.players[1].hand.len);

    let prompt = defeated(&e, s, source);
    assert_eq!(prompt.decision, Decision::AbilityOptional);
    assert_eq!(prompt.active, Seat::P2, "the opponent of the defeated unit's controller collects");
    assert_eq!(prompt.source, source);
    assert!(e.legal_actions(&prompt).iter().any(|action| action.label == "Trigger"));
    assert!(e.legal_actions(&prompt).iter().any(|action| action.label == "Pass"));

    let declined = named_button(&e, &prompt, "Pass");
    assert_eq!((declined.players[0].hand.len, declined.players[1].hand.len), (p1_hand, p2_hand));
    assert_eq!(declined.players[1].deck.as_slice(), p2_deck.as_slice());

    let collected = named_button(&e, &prompt, "Trigger");
    assert_eq!(collected.players[0].hand.len, p1_hand, "the defeated unit's controller does not draw");
    assert_eq!(collected.players[1].hand.len, p2_hand + 1, "the collector draws");
    assert_eq!(collected.players[1].deck.as_slice(), &p2_deck[1..]);
    assert_eq!(collected.players[0].deck.as_slice(), p1_deck.as_slice());
}

#[test]
fn bounty_mandatory_collection_readies_collector_resources_without_a_decline_prompt() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SHD_185");
    let resources: Vec<_> = (0..3)
        .map(|_| {
            let id = add(&e, &mut s, Seat::P2, "LOF_254");
            s.player_mut(Seat::P2).resources.push(id).unwrap();
            s.card_mut(id).exhausted = true;
            id
        })
        .collect();
    let mut exhausted = s;
    exhausted.active = Seat::P1;
    let resolved = defeated(&e, exhausted, source);
    assert_eq!(resolved.decision, Decision::Window, "an overridden non-optional bounty never asks to decline");
    for id in resources {
        assert!(!resolved.card(id).exhausted, "the collector readies its own resources");
    }
    // The same mandatory registration with nothing to change still collects without a
    // prompt (canResolveWithoutLegalTargets / hasAnyLegalEffects true, not a force).
    let mut idle = board(&e);
    let source = unit(&e, &mut idle, Seat::P1, "SHD_185");
    idle.active = Seat::P1;
    let quiet = defeated(&e, idle, source);
    assert_eq!(quiet.decision, Decision::Window);
}

#[test]
fn bounty_effect_shapes_use_the_collector_as_context_player() {
    let e = engine();
    // Damage (Wanted Insurgents) and exhaust (Fugitive Wookiee) select for the
    // collector, not for the defeated unit's controller.
    for (code, damage) in [("SHD_167", 2u16), ("SHD_211", 0)] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, Seat::P1, code);
        let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
        let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
        let prompt = defeated(&e, s, source);
        assert_eq!(prompt.decision, Decision::AbilityOptional, "{code}");
        let targeted = named_button(&e, &prompt, "Trigger");
        assert_eq!(targeted.decision, Decision::AbilityTarget, "{code}");
        assert_eq!(targeted.active, Seat::P2, "{code}");

        let picked = e.apply(&targeted, &e.card_action(&targeted, friendly)).unwrap();
        if damage > 0 {
            assert_eq!(picked.card(friendly).damage, damage, "{code}");
            assert_eq!(picked.card(enemy).damage, 0, "{code}");
        } else {
            assert!(picked.card(friendly).exhausted, "{code}");
            assert!(!picked.card(enemy).exhausted, "{code}");
        }
    }
    // Resource the top card of the collector's deck (Outlaw Corona).
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SHD_116");
    let p1_deck = decks(&e, &mut s, Seat::P1, 1);
    let p2_deck = decks(&e, &mut s, Seat::P2, 1);
    let p1_resources = s.players[0].resources.len;
    let prompt = defeated(&e, s, source);
    assert_eq!(prompt.decision, Decision::AbilityOptional);
    let collected = named_button(&e, &prompt, "Trigger");
    assert_eq!(collected.players[1].resources.as_slice(), p2_deck.as_slice(), "collector resources");
    assert!(collected.players[1].deck.as_slice().is_empty());
    assert_eq!(collected.players[0].resources.len, p1_resources, "bounty owner keeps its resources");
    assert_eq!(collected.players[0].deck.as_slice(), p1_deck.as_slice());
}

#[test]
fn bounty_collector_is_the_opponent_of_the_defeated_unit_even_on_its_own_turn() {
    // The defeated unit's controller is the non-active player, so the collector is the
    // active player. Collection still follows the defeated unit's controller, not the
    // turn: the active player's opponent only acts if it is that opponent.
    let e = engine();
    let mut s = board(&e);
    s.active = Seat::P1;
    let p1_deck = decks(&e, &mut s, Seat::P1, 1);
    let p2_deck = decks(&e, &mut s, Seat::P2, 1);
    let source = unit(&e, &mut s, Seat::P2, "SHD_027");
    let prompt = defeated(&e, s, source);
    assert_eq!(prompt.decision, Decision::AbilityOptional);
    assert_eq!(prompt.active, Seat::P1, "the source's controller's opponent collects");
    let collected = named_button(&e, &prompt, "Trigger");
    assert_eq!(collected.players[0].deck.as_slice(), &p1_deck[1..]);
    assert_eq!(collected.players[1].deck.as_slice(), p2_deck.as_slice());
}

#[test]
fn simultaneous_bounty_collections_are_ordered_and_grouped_by_the_collector() {
    let e = engine();
    // Two different bounty titles: the collector chooses which collected bounty
    // resolves first (the window has one player bucket, so no player-order prompt).
    let mut s = board(&e);
    s.active = Seat::P1;
    let draw_bounty = unit(&e, &mut s, Seat::P1, "SHD_027");
    let damage_bounty = unit(&e, &mut s, Seat::P1, "SHD_167");
    let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
    let p2_deck = decks(&e, &mut s, Seat::P2, 1);
    e.defeat_many(&mut s, &[draw_bounty, damage_bounty]).unwrap();
    e.finish(&mut s).unwrap();
    assert_eq!(s.decision, Decision::DeathOrder);
    assert_eq!(s.active, Seat::P2);
    let titles: Vec<_> = e.legal_actions(&s).iter().map(|action| action.label.clone()).collect();
    assert!(titles.iter().any(|title| title == "Collect Bounty: Draw a card"), "{titles:?}");
    assert!(titles.iter().any(|title| title == "Collect Bounty: Deal 2 damage to a unit"), "{titles:?}");
    let damage_first = e
        .legal_actions(&s)
        .into_iter()
        .find(|action| action.label == "Collect Bounty: Deal 2 damage to a unit")
        .unwrap();
    let chosen = e.apply(&s, &damage_first).unwrap();
    assert_eq!(chosen.decision, Decision::AbilityOptional);
    let targeted = named_button(&e, &chosen, "Trigger");
    let struck = e.apply(&targeted, &e.card_action(&targeted, friendly)).unwrap();
    assert_eq!(struck.card(friendly).damage, 2, "the chosen bounty resolved first");
    assert_eq!(struck.decision, Decision::AbilityOptional, "the other bounty follows");
    assert_eq!(struck.active, Seat::P2);
    let drawn = named_button(&e, &struck, "Trigger");
    assert_eq!(drawn.players[1].deck.as_slice(), &p2_deck[1..]);

    // Two identical bounty sources are grouped: the collector resolves the next
    // instance or all remaining instances of that group (SWU 7.6.11).
    let mut s = board(&e);
    s.active = Seat::P1;
    let first = unit(&e, &mut s, Seat::P1, "SHD_027");
    let second = unit(&e, &mut s, Seat::P1, "SHD_027");
    e.defeat_many(&mut s, &[first, second]).unwrap();
    e.finish(&mut s).unwrap();
    assert_eq!(s.decision, Decision::DeathBatch);
    let labels: Vec<_> = e.legal_actions(&s).iter().map(|action| action.label.clone()).collect();
    assert!(labels.iter().any(|label| label == "Resolve next"), "{labels:?}");
    assert!(labels.iter().any(|label| label.starts_with("Resolve all")), "{labels:?}");
    let next = named_button(&e, &s, "Resolve next");
    assert_eq!(next.decision, Decision::AbilityOptional);
    let collected = named_button(&e, &next, "Trigger");
    assert_eq!(collected.decision, Decision::AbilityOptional, "the second grouped instance asks next");
    let final_state = named_button(&e, &collected, "Trigger");
    assert_eq!(final_state.decision, Decision::Window);
}

#[test]
fn bounty_and_when_defeated_share_the_window_and_the_active_player_orders_the_players() {
    let e = engine();
    let mut s = board(&e);
    s.active = Seat::P1;
    let val = unit(&e, &mut s, Seat::P1, "SHD_058");
    let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
    let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
    e.defeat(&mut s, val).unwrap();
    e.finish(&mut s).unwrap();
    // Both players own a defeated Val trigger: When Defeated (Val's controller) and
    // Bounty (the collector). SWU 7.6.10 lets the active player choose the order.
    assert_eq!(s.decision, Decision::DeathPlayer);
    assert_eq!(s.active, Seat::P1);
    let labels: Vec<_> = e.legal_actions(&s).iter().map(|action| action.label.clone()).collect();
    assert_eq!(labels, vec!["You".to_string(), "Opponent".to_string()]);

    let opponent_first = named_button(&e, &s, "Opponent");
    assert_eq!(opponent_first.decision, Decision::AbilityOptional, "the collector's bounty is first");
    assert_eq!(opponent_first.active, Seat::P2);
    let targeted = named_button(&e, &opponent_first, "Trigger");
    assert_eq!(targeted.decision, Decision::AbilityTarget);
    let after_damage = e.apply(&targeted, &e.card_action(&targeted, friendly)).unwrap();
    assert_eq!(after_damage.card(friendly).damage, 3, "the collector's bounty resolved first");
    assert_eq!(experience(&e, &after_damage, friendly), 0);
    assert_eq!(after_damage.decision, Decision::AbilityTarget, "Val's When Defeated follows");
    assert_eq!(after_damage.active, Seat::P1);
    let after_xp = e.apply(&after_damage, &e.card_action(&after_damage, friendly)).unwrap();
    assert_eq!(experience(&e, &after_xp, friendly), 2);
    assert_eq!(after_xp.card(enemy).damage, 0);

    // The other order resolves Val's When Defeated before the bounty collection.
    let mut s = board(&e);
    s.active = Seat::P1;
    let val = unit(&e, &mut s, Seat::P1, "SHD_058");
    let friendly = unit(&e, &mut s, Seat::P1, "LOF_256");
    e.defeat(&mut s, val).unwrap();
    e.finish(&mut s).unwrap();
    assert_eq!(s.decision, Decision::DeathPlayer);
    let owner_first = named_button(&e, &s, "You");
    assert_eq!(owner_first.decision, Decision::AbilityTarget);
    assert_eq!(owner_first.active, Seat::P1);
    let gifted = e.apply(&owner_first, &e.card_action(&owner_first, friendly)).unwrap();
    assert_eq!(experience(&e, &gifted, friendly), 2);
    assert_eq!(gifted.decision, Decision::AbilityOptional, "the bounty follows");
    assert_eq!(gifted.active, Seat::P2);
}

#[test]
fn bounty_keyword_queries_drive_printed_conditions_and_target_filters() {
    let e = engine();
    // Trandoshan Hunters: "When Played: If an enemy unit has a Bounty, give an
    // Experience token to this unit." The condition reads the intrinsic keyword.
    for bounty in [true, false] {
        let mut s = board(&e);
        if bounty {
            unit(&e, &mut s, Seat::P2, "SHD_027");
        } else {
            let plain = unit(&e, &mut s, Seat::P2, "LOF_256");
            assert!(!e.has_keyword(&s, plain, "bounty"));
        }
        let hunters = hand(&e, &mut s, "SHD_140");
        let played = e.apply(&s, &e.card_action(&s, hunters)).unwrap();
        assert_eq!(experience(&e, &played, hunters), u16::from(bounty), "bounty={bounty}");
    }
    // Covetous Rivals: "When Played/On Attack: Deal 2 damage to a unit with a
    // Bounty." The selector predicate reads the same intrinsic keyword, so a
    // mention-only Trandoshan Hunters is not selectable but Hylobon Enforcer is.
    let mut s = board(&e);
    unit(&e, &mut s, Seat::P2, "SHD_140");
    let marked = unit(&e, &mut s, Seat::P1, "SHD_027");
    let plain = unit(&e, &mut s, Seat::P2, "LOF_256");
    let rivals = hand(&e, &mut s, "SHD_171");
    let played = e.apply(&s, &e.card_action(&s, rivals)).unwrap();
    assert_eq!(played.decision, Decision::AbilityTarget);
    let offered: Vec<_> = e.legal_actions(&played).iter().filter_map(Action::id).collect();
    assert_eq!(offered, vec![marked], "only the printed-bounty unit is selectable");
    let struck = e.apply(&played, &e.card_action(&played, marked)).unwrap();
    assert_eq!(struck.card(marked).damage, 2);
    assert_eq!(struck.card(plain).damage, 0);
}

#[test]
fn bounty_optional_prompt_survives_a_selector_with_no_legal_target() {
    // The engine sets canResolveWithoutLegalTargets and always has a legal effect, so
    // the collector is still asked and may collect with nothing to choose.
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SHD_167");
    let prompt = defeated(&e, s, source);
    assert_eq!(prompt.decision, Decision::AbilityOptional);
    assert_eq!(prompt.active, Seat::P2);
    let collected = named_button(&e, &prompt, "Trigger");
    assert_eq!(collected.decision, Decision::Window, "collecting with no legal target resolves");
}

#[test]
fn bounty_source_mutants_stay_explicit_rejections() {
    let catalogue: Value = serde_json::from_slice(committed_catalogue_bytes()).unwrap();
    let source = |catalogue: &Value| {
        catalogue["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["setCode"] == "SHD_027")
            .unwrap()
            .clone()
    };
    let rejected = |catalogue: Value, label: &str| -> String {
        let engine = Engine::from_catalogue_value(&catalogue).unwrap();
        assert!(!engine.by_code.contains_key("SHD_027"), "{label}");
        let reason = engine.rejected_cards().unwrap()["SHD_027"].clone();
        assert!(!reason.is_empty(), "{label}");
        reason
    };
    let bounty_index = |entry: &Value| {
        entry["abilities"]["card"]
            .as_array()
            .unwrap()
            .iter()
            .position(|ability| ability["trigger"] == "bounty")
            .unwrap()
    };
    let program = |entry: &Value| entry["abilities"]["card"].as_array().unwrap()[bounty_index(entry)].clone();
    assert!(program(&source(&catalogue))["bounty"].is_object());

    for (label, mutate) in [
        ("printed without registration", Box::new(|entry: &mut Value| {
            let index = entry["abilities"]["card"]
                .as_array()
                .unwrap()
                .iter()
                .position(|ability| ability["trigger"] == "bounty")
                .unwrap();
            entry["abilities"]["card"].as_array_mut().unwrap().remove(index);
        }) as Box<dyn Fn(&mut Value)>),
        ("registration without printed keyword", Box::new(|entry: &mut Value| {
            entry["card"]["text"] = serde_json::json!("Grit (This unit gets +1/+0 for each damage on it.)");
        })),
        ("trigger disagrees with metadata", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["trigger"] = serde_json::json!("whenDefeated");
        })),
        ("capture-only event list", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["bounty"]["events"] = serde_json::json!(["onCardDefeated"]);
        })),
        ("collector is not the opponent", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["bounty"]["canBeTriggeredBy"] = serde_json::json!("self");
        })),
        ("context player is not the collector", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["bounty"]["contextPlayer"] = serde_json::json!("source");
        })),
        ("unknown metadata field", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["bounty"]["extra"] = serde_json::json!(true);
        })),
        ("not an optional flag override", Box::new(|entry: &mut Value| {
            let index = bounty_index(entry);
            entry["abilities"]["card"][index]["optional"] = serde_json::json!("yes");
        })),
        ("registration on a leader source", Box::new(|entry: &mut Value| {
            let ability = program(entry);
            entry["baseClass"] = serde_json::json!("LeaderUnitCard");
            entry["card"]["types"] = serde_json::json!(["leader"]);
            entry["abilities"]["card"] = serde_json::json!([]);
            entry["abilities"]["unit"] = serde_json::json!([ability]);
        })),
        ("second registration for one printed instance", Box::new(|entry: &mut Value| {
            let ability = program(entry);
            entry["abilities"]["card"].as_array_mut().unwrap().push(ability);
        })),
    ] {
        let mut entry = source(&catalogue);
        mutate(&mut entry);
        let mut catalogue = catalogue.clone();
        catalogue["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|candidate| candidate["setCode"] == "SHD_027")
            .map(|candidate| *candidate = entry);
        let reason = rejected(catalogue, label);
        // Serde-shaped mutants surface as typed IR errors; every family-shape mutant
        // must say Bounty so the rejection is actionable.
        let family_shape = !matches!(
            label,
            "not an optional flag override" | "unknown metadata field"
        );
        if family_shape {
            assert!(reason.contains("Bounty") || reason.contains("bounty"), "{label}: {reason}");
        }
    }
    // The untouched committed entry still admits, so the mutants above are the cause.
    assert!(Engine::from_catalogue_value(&catalogue)
        .unwrap()
        .by_code
        .contains_key("SHD_027"));
}

#[test]
fn bounty_metadata_is_never_serialized_into_a_runtime_program() {
    // The executable program content hash covers instruction dispatch only; the
    // reviewed metadata is admission-time input, so two catalogues that differ only
    // in an unused metadata value cannot execute differently (the loader rejects
    // the changed value instead).
    let e = engine();
    let d = &e.definitions[*e.by_code.get("SHD_027").unwrap() as usize];
    let program = d.programs.card.iter().map(|p| e.program(*p)).find(|p| p.trigger == "bounty").unwrap();
    let value = serde_json::to_value(program).unwrap();
    assert!(value.get("bounty").is_none());
}
