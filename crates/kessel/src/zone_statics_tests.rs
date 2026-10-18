//! Bounded zone-static families: first-strike combat ordering,
//! deterministic bottom-of-deck moves, and reviewed resource entry.
use super::*;
use crate::ability_ir::{Ability, Effect, Instruction, Program};

fn start(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "missing action for {id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn attack(e: &Engine, s: &mut State, attacker: CardId, defender: CardId) -> State {
    let prompt = click(e, s, attacker);
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let a = e.card_action(&prompt, defender);
    assert!(
        e.legal_actions(&prompt).contains(&a),
        "missing attack target {defender}: {:?}",
        e.legal_actions(&prompt)
    );
    e.apply(&prompt, &a).unwrap()
}
fn settle(e: &Engine, mut s: State) -> State {
    for _ in 0..64 {
        if matches!(s.decision, Decision::Window | Decision::Terminal) {
            return s;
        }
        let actions = e.legal_actions(&s);
        let action = actions
            .iter()
            .find(|a| a.label == "Trigger")
            .cloned()
            .or_else(|| actions.iter().find(|a| a.label == "Play anyway").cloned())
            .or_else(|| actions.iter().find(|a| a.label == "Pass").cloned())
            .unwrap_or_else(|| panic!("unexpected decision {:?}: {:?}", s.decision, actions));
        s = e.apply(&s, &action).unwrap();
    }
    panic!("settle loop did not reach a window");
}
fn compile(ability: Value) -> Result<Program, String> {
    let ability: Ability =
        serde_json::from_value(ability).map_err(|e| format!("invalid ability IR: {e}"))?;
    Program::compile(&ability)
}

#[test]
fn zone_statics_first_strike_constant_defeats_defender_without_return_damage() {
    // SHD_234 Incinerator Trooper: 2/2 "While attacking, deals combat damage
    // before the defender." LOF_254 is a 1/1, so it is defeated before its own
    // combat damage event is generated.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "SHD_234");
            let defender = unit(&e, &mut s, seat.other(), "LOF_254");
            let done = settle(&e, attack(&e, &mut s, attacker, defender));
            assert!(e.in_arena(&done, attacker));
            assert!(!e.in_arena(&done, defender));
            assert!(done.player(seat.other()).discard.contains(defender));
            assert_eq!(
                done.card(attacker).damage,
                0,
                "defeated defender must not deal damage"
            );
            assert!(done.card(attacker).exhausted);
            assert_eq!((done.attack_source, done.attack_defender), (0, 0));
        }
    }
}
#[test]
fn zone_statics_simultaneous_combat_control_still_takes_return_damage() {
    // LOF_093 is a plain 2/5: it defeats LOF_254 in normal simultaneous combat
    // and still takes the defender's combat damage.
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_093");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
    let done = settle(&e, attack(&e, &mut s, attacker, defender));
    assert!(!e.in_arena(&done, defender));
    assert_eq!(done.card(attacker).damage, 1);
}
#[test]
fn zone_statics_first_strike_surviving_defender_deals_damage_back() {
    // A defender that survives the first window still resolves its own combat
    // damage event: 2 power against a 1/4 leaves it alive and it deals 1 back.
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_234");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_256");
    let done = settle(&e, attack(&e, &mut s, attacker, defender));
    assert!(e.in_arena(&done, defender));
    assert_eq!(done.card(defender).damage, 2);
    assert_eq!(done.card(attacker).damage, 1);
}
#[test]
fn zone_statics_first_strike_shield_windows_are_sequential_not_simultaneous() {
    // Both combatants are shielded. Simultaneous combat would open a shield
    // replacement order prompt; first strike resolves two sequential windows, so
    // each shield replaces exactly its own window and no damage is dealt.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "SHD_234");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        e.create_token(&mut s, attacker, false).unwrap();
        e.create_token(&mut s, defender, false).unwrap();
        let done = settle(&e, attack(&e, &mut s, attacker, defender));
        assert_eq!(done.card(attacker).shield, 0);
        assert_eq!(done.card(defender).shield, 0);
        assert_eq!(done.card(attacker).damage, 0);
        assert_eq!(done.card(defender).damage, 0);
        assert!(e.in_arena(&done, defender));
    }
    // A defender-only shield replaces the first window, not the return damage.
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_234");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.create_token(&mut s, defender, false).unwrap();
    let done = settle(&e, attack(&e, &mut s, attacker, defender));
    assert_eq!(done.card(defender).damage, 0);
    assert_eq!(done.card(attacker).damage, 1);
}
#[test]
fn zone_statics_shoot_first_grants_power_and_first_strike_to_the_chosen_attacker() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let own = unit(&e, &mut s, Seat::P1, "LOF_093");
        let own_other = unit(&e, &mut s, Seat::P1, "LOF_256");
        let enemy_ready = unit(&e, &mut s, Seat::P2, "LOF_256");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        let event = hand(&e, &mut s, "SOR_217");
        let cost = e.cost(&s, event);
        let ready_before = e.ready_resources(&s, Seat::P1);
        let choosing = click(&e, &s, event);
        assert_eq!(choosing.decision, Decision::AbilityTarget);
        let ids: Vec<_> = e
            .legal_actions(&choosing)
            .iter()
            .filter_map(Action::id)
            .collect();
        // "Attack with a unit": the event's callback predicate restricts the
        // chosen attacker to units controlled by the initiating player.
        assert!(ids.contains(&own));
        assert!(ids.contains(&own_other));
        assert!(!ids.contains(&enemy_ready));
        assert!(!ids.contains(&defender));
        let choosing = click(&e, &choosing, own);
        assert_eq!(choosing.decision, Decision::AttackTarget);
        assert_eq!(
            e.power(&choosing, own, true),
            3,
            "Shoot First grants +1/+0 for the attack"
        );
        assert_eq!(choosing.support.len, 1);
        // The until-end-of-attack first-strike grant is part of the exported
        // support state, not only of an in-memory flag. A pending attack window
        // cannot be imported mid-prompt, so the round trip is checked after the
        // attack window has cleaned up.
        let snapshot = e.snapshot(&choosing);
        assert_eq!(snapshot["supportGrants"][0]["firstStrike"], json!(true));
        let a = e.card_action(&choosing, defender);
        assert!(e.legal_actions(&choosing).contains(&a));
        let done = settle(&e, e.apply(&choosing, &a).unwrap());
        assert!(!e.in_arena(&done, defender));
        assert_eq!(
            done.card(own).damage,
            0,
            "first-strike attacker must not take return damage"
        );
        assert_eq!(
            e.power(&done, own, false),
            2,
            "the attack bonus expires with the attack"
        );
        assert_eq!(done.support.len, 0);
        assert!(done.player(Seat::P1).discard.contains(event));
        assert_eq!(
            e.ready_resources(&done, Seat::P1),
            ready_before - cost as usize
        );
        let snapshot = e.snapshot(&done);
        let restored = e.load_state(&snapshot).unwrap();
        assert_eq!(e.snapshot(&restored), snapshot);
        assert_eq!(restored.zobrist(), done.zobrist());
    }
}
#[test]
fn zone_statics_first_strike_overwhelm_excess_uses_the_first_window_damage() {
    // ASH_143 is a 3/2 Overwhelm unit. Shoot First makes the attack 4 power,
    // which defeats a 1/1 defender in the first window: no return damage and
    // the three excess damage go to the defending base.
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let own = unit(&e, &mut s, Seat::P1, "ASH_143");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
    let enemy_base = s.player(Seat::P2).base;
    let event = hand(&e, &mut s, "SOR_217");
    let choosing = click(&e, &s, event);
    let choosing = click(&e, &choosing, own);
    assert_eq!(choosing.decision, Decision::AttackTarget);
    assert_eq!(e.power(&choosing, own, true), 4);
    let a = e.card_action(&choosing, defender);
    assert!(e.legal_actions(&choosing).contains(&a));
    let done = settle(&e, e.apply(&choosing, &a).unwrap());
    assert!(!e.in_arena(&done, defender));
    assert_eq!(done.card(own).damage, 0);
    assert_eq!(done.card(enemy_base).damage, 3);
}
#[test]
fn zone_statics_support_copied_first_strike_ability_applies_to_the_attacker() {
    // ASH_202 Carson Teva has Support and "While attacking, deals combat damage
    // first". The Support attack copies both the constant registration and the
    // intrinsic keyword presence to the chosen attacker.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let donor = unit(&e, &mut s, Seat::P1, "ASH_202");
        let attacker = unit(&e, &mut s, Seat::P1, "ASH_143");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        s.source = donor;
        e.begin_support_attack(&mut s, attacker).unwrap();
        assert_eq!(s.decision, Decision::AttackTarget);
        assert!(e.support_programs(&s, attacker).iter().any(|p| e
            .program(*p)
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Apply(Effect::DealsCombatDamageFirst { .. })))));
        let a = e.card_action(&s, defender);
        assert!(e.legal_actions(&s).contains(&a));
        let done = settle(&e, e.apply(&s, &a).unwrap());
        assert!(!e.in_arena(&done, defender));
        assert_eq!(done.card(attacker).damage, 0);
        assert_eq!(done.support.len, 0);
    }
}
#[test]
fn zone_statics_commence_patrol_bottoms_any_discard_card_and_creates_xwing() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let own_discard = add(&e, &mut s, Seat::P1, "LOF_093");
        s.player_mut(Seat::P1).discard.push(own_discard).unwrap();
        let enemy_discard = add(&e, &mut s, Seat::P2, "LOF_256");
        s.player_mut(Seat::P2).discard.push(enemy_discard).unwrap();
        let event = hand(&e, &mut s, "JTL_205");
        let choosing = click(&e, &s, event);
        assert_eq!(choosing.decision, Decision::AbilityTarget);
        let ids: Vec<_> = e
            .legal_actions(&choosing)
            .iter()
            .filter_map(Action::id)
            .collect();
        // "another card in a discard pile": either player's discard is legal,
        // but the played event itself is not in a discard yet.
        assert!(ids.contains(&own_discard));
        assert!(ids.contains(&enemy_discard));
        assert!(!ids.contains(&event));
        let done = settle(&e, click(&e, &choosing, enemy_discard));
        assert!(!done.player(Seat::P2).discard.contains(enemy_discard));
        assert_eq!(
            done.player(Seat::P2).deck.as_slice().last(),
            Some(&enemy_discard)
        );
        assert_eq!(done.player(Seat::P2).deck.len as usize, 1);
        assert!(done.player(Seat::P1).discard.contains(event));
        let xwing = done
            .player(Seat::P1)
            .space
            .as_slice()
            .iter()
            .copied()
            .find(|id| e.def(&done, *id).code == "TOKEN_XWING")
            .expect("Commence Patrol's ifYouDo creates an X-Wing token");
        // Engine CreateTokenUnitSystem defaults entersReady to false.
        assert!(done.card(xwing).exhausted);
    }
}
#[test]
fn zone_statics_zero_zero_zero_bottoms_only_an_aggression_discard_card() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let ooo = unit(&e, &mut s, Seat::P1, "LAW_174");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_256");
        let aggression = add(&e, &mut s, Seat::P1, "ASH_141");
        s.player_mut(Seat::P1).discard.push(aggression).unwrap();
        let plain = add(&e, &mut s, Seat::P1, "LOF_093");
        s.player_mut(Seat::P1).discard.push(plain).unwrap();
        let enemy_base = s.player(Seat::P2).base;
        let base_damage = s.card(enemy_base).damage;
        let prompt = click(&e, &s, ooo);
        assert_eq!(prompt.decision, Decision::AttackTarget);
        let a = e.card_action(&prompt, defender);
        assert!(e.legal_actions(&prompt).contains(&a));
        // The optional On Attack registration owns a root target resolver, so
        // the runtime offers the discard selection directly (declining is the
        // selection's Pass action) rather than a separate optional prompt.
        let targeting = e.apply(&prompt, &a).unwrap();
        assert_eq!(targeting.decision, Decision::AbilityTarget);
        let ids: Vec<_> = e
            .legal_actions(&targeting)
            .iter()
            .filter_map(Action::id)
            .collect();
        assert_eq!(ids, vec![aggression]);
        let done = settle(&e, click(&e, &targeting, aggression));
        assert!(!done.player(Seat::P1).discard.contains(aggression));
        assert_eq!(
            done.player(Seat::P1).deck.as_slice().last(),
            Some(&aggression)
        );
        assert!(done.player(Seat::P1).discard.contains(plain));
        assert_eq!(done.card(enemy_base).damage, base_damage + 1);
    }
}
#[test]
fn zone_statics_ima_gun_di_resources_selected_hand_card_then_top_deck() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let ima = unit(&e, &mut s, Seat::P1, "HMW_044");
        let hand_card = add(&e, &mut s, Seat::P1, "LOF_093");
        s.player_mut(Seat::P1).hand.push(hand_card).unwrap();
        let top = add(&e, &mut s, Seat::P1, "LOF_256");
        s.player_mut(Seat::P1).deck.push(top).unwrap();
        let enemy_resource = add(&e, &mut s, Seat::P2, "LOF_254");
        s.player_mut(Seat::P2)
            .resources
            .push(enemy_resource)
            .unwrap();
        assert_eq!(s.player(Seat::P1).resources.len, 0);
        assert_eq!(s.player(Seat::P2).resources.len, 1);
        // "If you have fewer resources than an opponent" holds, so the defeated
        // unit's mandatory trigger offers its optional hand selection.
        e.defeat(&mut s, ima).unwrap();
        assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
        assert_eq!(s.decision, Decision::AbilityTarget);
        let ids: Vec<_> = e.legal_actions(&s).iter().filter_map(Action::id).collect();
        assert_eq!(ids, vec![hand_card]);
        let done = settle(&e, click(&e, &s, hand_card));
        assert!(!done.player(Seat::P1).hand.contains(hand_card));
        assert!(done.player(Seat::P1).resources.contains(hand_card));
        assert!(done.card(hand_card).exhausted);
        assert!(!done.player(Seat::P1).deck.contains(top));
        assert!(done.player(Seat::P1).resources.contains(top));
        assert!(done.card(top).exhausted);
        assert_eq!(done.player(Seat::P1).resources.len, 2);
        assert_eq!(done.decision, Decision::Window);
    }
    // Boundary: with equal or more resources the conditional takes no action and
    // neither card is resourced.
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let ima = unit(&e, &mut s, Seat::P1, "HMW_044");
    let hand_card = add(&e, &mut s, Seat::P1, "LOF_093");
    s.player_mut(Seat::P1).hand.push(hand_card).unwrap();
    let top = add(&e, &mut s, Seat::P1, "LOF_256");
    s.player_mut(Seat::P1).deck.push(top).unwrap();
    let own_resource = add(&e, &mut s, Seat::P1, "LOF_254");
    s.player_mut(Seat::P1).resources.push(own_resource).unwrap();
    e.defeat(&mut s, ima).unwrap();
    e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
    let done = settle(&e, s);
    assert!(done.player(Seat::P1).hand.contains(hand_card));
    assert!(done.player(Seat::P1).deck.contains(top));
    assert_eq!(done.player(Seat::P1).resources.len, 1);
}
#[test]
fn zone_statics_superlaser_technician_resources_itself_ready_from_discard() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let technician = unit(&e, &mut s, Seat::P1, "SHD_085");
        let resources = s.player(Seat::P1).resources.len;
        e.defeat(&mut s, technician).unwrap();
        assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
        assert_eq!(s.decision, Decision::AbilityOptional);
        let triggered = named_button(&e, &s, "Trigger");
        let done = settle(&e, triggered);
        assert!(!e.in_arena(&done, technician));
        assert!(!done.player(Seat::P1).discard.contains(technician));
        assert!(done.player(Seat::P1).resources.contains(technician));
        assert_eq!(done.player(Seat::P1).resources.len, resources + 1);
        assert!(
            !done.card(technician).exhausted,
            "readyResource readies the resourced card"
        );
        assert_eq!(done.decision, Decision::Window);
    }
}
#[test]
fn zone_statics_reviewed_shapes_compile_and_unreviewed_shapes_stay_rejected() {
    // Positive: the exact committed registrations compile.
    for (code, side) in [
        ("SHD_234", "card"),
        ("ASH_202", "card"),
        ("SOR_217", "card"),
        ("JTL_205", "card"),
        ("LAW_174", "card"),
        ("SHD_085", "card"),
        ("HMW_044", "card"),
        ("SOR_198", "card"),
    ] {
        let catalog: Value =
            serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
                .unwrap();
        let entry = catalog["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == code)
            .unwrap();
        let sides: crate::ability_ir::Sides =
            serde_json::from_value(entry["abilities"].clone()).unwrap();
        let abilities = match side {
            "card" => &sides.card,
            _ => unreachable!(),
        };
        assert!(!abilities.is_empty(), "{code}");
        for ability in abilities {
            assert!(
                Program::compile(ability).is_ok(),
                "{code}: {:?}",
                Program::compile(ability)
            );
        }
    }
    // Negative first-strike shapes: the constant family only executes while
    // attacking, and its exact condition/duration/target are mandatory.
    let first_strike = |effect: Value, condition: Value| {
        json!({
            "trigger":"constant","title":"While attacking, deals combat damage first","costs":[],
            "effect":effect,"condition":condition
        })
    };
    let attacking = json!({"kind":"query","name":"isAttacking","receiver":{"kind":"ref","path":["source"]},"args":[]});
    assert!(compile(first_strike(
        json!({"kind":"dealsCombatDamageFirst","target":"self","duration":"persistent"}),
        attacking.clone()
    ))
    .is_ok());
    for (effect, condition) in [
        (
            json!({"kind":"dealsCombatDamageFirst","target":"self","duration":"attack"}),
            attacking.clone(),
        ),
        (
            json!({"kind":"dealsCombatDamageFirst","target":"chosen","duration":"persistent"}),
            attacking.clone(),
        ),
        (
            json!({"kind":"dealsCombatDamageFirst","target":"self","duration":"persistent"}),
            json!({"kind":"const","value":true}),
        ),
        (
            json!({"kind":"dealsCombatDamageFirst","target":"self","duration":"persistent","damageFirst":true}),
            attacking.clone(),
        ),
    ] {
        assert!(compile(first_strike(effect, condition)).is_err());
    }
    // Negative bottom-of-deck shapes: only a single chosen card from a hand or
    // discard pile is admitted, and unknown ordering fields fail closed.
    let bottom = |trigger: &str, resolver: Option<Value>, effect: Value| {
        let mut value = json!({"trigger":trigger,"title":"Bottom","costs":[],"effect":effect});
        if let Some(resolver) = resolver {
            value["targetResolver"] = resolver;
        }
        value
    };
    assert!(compile(bottom(
        "event",
        Some(json!({"owner":"any","type":"any","zone":"discard"})),
        json!({"kind":"moveToBottomOfDeck","target":"chosen"})
    ))
    .is_ok());
    for ability in [
        bottom(
            "event",
            Some(json!({"owner":"any","type":"unit","zone":"anyAttackable"})),
            json!({"kind":"moveToBottomOfDeck","target":"chosen"}),
        ),
        bottom(
            "event",
            Some(json!({"owner":"any","type":"any","zone":"discard"})),
            json!({"kind":"moveToBottomOfDeck","target":"self"}),
        ),
        bottom(
            "event",
            Some(json!({"owner":"any","type":"any","zone":"discard"})),
            json!({"kind":"moveToBottomOfDeck","target":"chosen","shuffle":true}),
        ),
    ] {
        assert!(compile(ability).is_err());
    }
    // Negative resource-entry shapes: foreign resource zones, plural top-deck
    // amounts, unreviewed timings and unknown fields remain rejected.
    let resource = |target: Value, player: &str| {
        json!({
            "trigger":"event","title":"Resource","costs":[],
            "effect":{"kind":"resourceCardSystem","target":target,"targetPlayer":player,"readyResource":false,
                "readyTiming":"contingentAfterResourceEvent","readinessTargets":"completeSystemTargetList"}
        })
    };
    let top = json!({"kind":"query","name":"topDeckCard","receiver":{"kind":"ref","path":["player"]},"args":[]});
    assert!(compile(resource(json!("self"), "self")).is_ok());
    assert!(compile(resource(top.clone(), "self")).is_ok());
    for ability in [
        resource(json!("self"), "opponent"),
        resource(
            json!({"kind":"query","name":"topDeckCards","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"const","value":2}]}),
            "self",
        ),
        resource(
            json!({"kind":"query","name":"topDeckCard","receiver":{"kind":"ref","path":["opponent"]},"args":[]}),
            "self",
        ),
    ] {
        assert!(compile(ability).is_err());
    }
    let mut extra = resource(json!("self"), "self");
    extra["effect"]["readyTiming"] = json!("immediate");
    assert!(compile(extra).is_err());
}

#[test]
fn zone_statics_first_strike_reads_surviving_grit_power_after_first_damage() {
    // Pinned AttackFlow creates defender damage after the first EventWindow.
    // Mos Eisley Modifier is 1/4 with Grit: two damage makes its return power 3,
    // defeating the 2-HP Incinerator. Capturing power before the hit saves it.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "SHD_234");
            let defender = unit(&e, &mut s, seat.other(), "ASH_074");
            assert_eq!(e.power(&s, defender, false), 1);
            let mut resilient = s;
            resilient.card_mut(attacker).phase_hp = 2;
            let done = settle(&e, attack(&e, &mut s, attacker, defender));
            assert!(e.in_arena(&done, defender));
            assert_eq!(done.card(defender).damage, 2);
            assert_eq!(e.power(&done, defender, false), 3);
            assert!(!e.in_arena(&done, attacker));
            assert!(done.player(seat).discard.contains(attacker));
            assert_eq!(done.card(attacker).damage, 0, "leaving play clears damage");

            // Keep a second attacker alive so the exact return damage remains
            // observable after resolution, independently of discard cleanup.
            let survived = settle(&e, attack(&e, &mut resilient, attacker, defender));
            assert!(e.in_arena(&survived, attacker));
            assert_eq!(survived.card(attacker).damage, 3);
            assert_eq!(survived.card(defender).damage, 2);
        }
    }
}

#[test]
fn zone_statics_first_window_death_trigger_finishes_before_attack_complete() {
    // Superlaser Technician's real optional death ability must resolve between
    // attacker damage and AttackComplete. It never gets to return combat damage.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "SHD_234");
        let technician = unit(&e, &mut s, Seat::P2, "SHD_085");
        let prompt = attack(&e, &mut s, attacker, technician);
        assert_eq!(prompt.decision, Decision::AbilityOptional);
        assert_eq!(prompt.death_depth, 1);
        assert_eq!(prompt.death_frames[0].resume, 8);
        assert_eq!(prompt.attack_source, attacker);
        assert_eq!(prompt.attack_defender, technician);
        assert_eq!(prompt.card(attacker).damage, 0);
        assert!(prompt.player(Seat::P2).discard.contains(technician));
        let done = settle(&e, named_button(&e, &prompt, "Trigger"));
        assert!(done.player(Seat::P2).resources.contains(technician));
        assert!(!done.card(technician).exhausted);
        assert_eq!(done.card(attacker).damage, 0);
        assert_eq!(done.death_depth, 0);
        assert_eq!(done.death_frames[0], DeathFrame::default());
        assert_eq!((done.attack_source, done.attack_defender), (0, 0));
    }
}

#[test]
fn zone_statics_first_strike_shield_cancels_overwhelm_contingent() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_143");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
    e.create_token(&mut s, defender, false).unwrap();
    let base = s.player(Seat::P2).base;
    let event = hand(&e, &mut s, "SOR_217");
    let choosing = click(&e, &click(&e, &s, event), attacker);
    let done = settle(&e, click(&e, &choosing, defender));
    assert!(e.in_arena(&done, defender));
    assert_eq!(done.card(defender).shield, 0);
    assert_eq!(done.card(defender).damage, 0);
    assert_eq!(done.card(attacker).damage, 1);
    assert_eq!(done.card(base).damage, 0);
}

#[test]
fn zone_statics_all_three_intrinsic_first_strike_cards_use_live_constant() {
    let e = engine();
    for code in ["ASH_202", "SHD_234", "SOR_198"] {
        let mut s = start(&e, Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, code);
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        assert!(
            !e.deals_combat_damage_first(&s, attacker),
            "{code} is not attacking yet"
        );
        let done = settle(&e, attack(&e, &mut s, attacker, defender));
        assert!(e.in_arena(&done, attacker), "{code}");
        assert!(!e.in_arena(&done, defender), "{code}");
        assert_eq!(done.card(attacker).damage, 0, "{code}");
        assert!(!e.deals_combat_damage_first(&done, attacker));
    }
}

#[test]
fn zone_statics_resource_optional_decline_does_not_resource_top_deck() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let ima = unit(&e, &mut s, Seat::P1, "HMW_044");
    let hand_card = add(&e, &mut s, Seat::P1, "LOF_093");
    s.player_mut(Seat::P1).hand.push(hand_card).unwrap();
    let top = add(&e, &mut s, Seat::P1, "LOF_256");
    s.player_mut(Seat::P1).deck.push(top).unwrap();
    let enemy_resource = add(&e, &mut s, Seat::P2, "LOF_254");
    s.player_mut(Seat::P2)
        .resources
        .push(enemy_resource)
        .unwrap();
    e.defeat(&mut s, ima).unwrap();
    assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
    assert_eq!(s.decision, Decision::AbilityTarget);
    let done = settle(&e, named_button(&e, &s, "Choose nothing"));
    assert!(done.player(Seat::P1).hand.contains(hand_card));
    assert!(done.player(Seat::P1).deck.contains(top));
    assert_eq!(done.player(Seat::P1).resources.len, 0);
}

#[test]
fn zone_statics_complete_cards_and_imported_continuations_fail_closed() {
    let e = engine();
    for code in [
        "ASH_202", "SHD_234", "SOR_198", "SOR_217", "HMW_044", "SHD_085", "JTL_205", "LAW_174",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    // Bottom movement alone must not admit the Sarlacc's unimplemented named
    // selection contract or physical selected-value read.
    assert!(!e.by_code.contains_key("LAW_163"));
    let s = start(&e, Seat::P1);
    for field in ["deathFrames", "deathContinuation", "deathDepth"] {
        let mut snapshot = e.snapshot(&s);
        snapshot[field] = json!(9);
        assert!(e.load_state(&snapshot).is_err(), "{field}");
    }
    let invalid = s;
    assert!(DeathContinuation::from_raw(9).is_none());
    assert_eq!(invalid, s);
}

fn betrayed_trust_marks(e: &Engine, s: &mut State, caster: Seat, target: CardId) -> State {
    // Play the complete card and choose its real enemy-unit resolver. This
    // creates the sibling restriction mechanism's phase mark through its IR.
    s.active = caster;
    s.action_player = Some(caster);
    let event = add(e, s, caster, "LAW_130");
    s.player_mut(caster).hand.push(event).unwrap();
    for _ in 0..8 {
        let resource = add(e, s, caster, "LOF_254");
        s.player_mut(caster).resources.push(resource).unwrap();
    }
    let prompt = click(e, s, event);
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let marked = settle(e, click(e, &prompt, target));
    assert!(marked.player(caster).discard.contains(event));
    assert!(e.card_restricted(
        &marked,
        target,
        crate::ability_ir::RestrictionVerb::DealCombatDamage
    ));
    marked
}

#[test]
fn zone_statics_betrayed_trust_first_strike_attacker_deals_no_damage_or_overwhelm() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat.other());
            let attacker = unit(&e, &mut s, seat, "SHD_234");
            let defender = unit(&e, &mut s, seat.other(), "LOF_254");
            // Isolate the Overwhelm interaction using the existing phase grant
            // primitive; the restriction itself comes from playing LAW_130.
            s.source = attacker;
            let overwhelm: Effect = serde_json::from_value(json!({
                "kind":"gainKeyword", "target":"self", "duration":"phase",
                "keyword":"overwhelm", "amount":{"kind":"const","value":1}
            }))
            .unwrap();
            assert_eq!(e.apply_effect(&mut s, &overwhelm).unwrap(), (1, true));
            s.source = 0;
            let mut marked = betrayed_trust_marks(&e, &mut s, seat.other(), attacker);
            assert_eq!(marked.active, seat);
            assert_eq!(e.power(&marked, attacker, true), 2);
            assert_eq!(e.combat_damage(&marked, attacker, true), 0);
            assert_eq!(e.keyword_amount(&marked, attacker, "overwhelm"), 1);
            let base = marked.player(seat.other()).base;
            let base_before = marked.card(base).damage;
            let done = settle(&e, attack(&e, &mut marked, attacker, defender));
            assert!(e.in_arena(&done, attacker));
            assert!(e.in_arena(&done, defender));
            assert_eq!(done.card(defender).damage, 0);
            assert_eq!(done.card(attacker).damage, 1);
            assert_eq!(done.card(base).damage, base_before);
        }
    }
}

#[test]
fn zone_statics_betrayed_trust_surviving_grit_defender_deals_no_return_damage() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat);
            let attacker = unit(&e, &mut s, seat, "SHD_234");
            let defender = unit(&e, &mut s, seat.other(), "ASH_074");
            let marked = betrayed_trust_marks(&e, &mut s, seat, defender);
            // The other player passes after the event, returning a legal action
            // window to the attacker while the phase restriction remains live.
            let mut marked = settle(&e, named_button(&e, &marked, "Pass"));
            assert_eq!(marked.active, seat);
            let done = settle(&e, attack(&e, &mut marked, attacker, defender));
            assert!(e.in_arena(&done, attacker));
            assert!(e.in_arena(&done, defender));
            assert_eq!(done.card(defender).damage, 2);
            assert_eq!(e.power(&done, defender, false), 3);
            assert_eq!(e.combat_damage(&done, defender, false), 0);
            assert_eq!(done.card(attacker).damage, 0);
        }
    }
}

#[test]
fn zone_statics_full_catalogue_rejects_unimplemented_source_context() {
    let catalogue: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let focus = [
        "ASH_202", "SHD_234", "SOR_198", "SOR_217", "HMW_044", "SHD_085", "JTL_205", "LAW_174",
    ];
    let original = crate::ability_ir::Catalogue::load(&catalogue).unwrap();
    for code in focus {
        assert!(
            original.cards.contains_key(code),
            "{code} legitimate complete source"
        );
    }
    for (field, value) in [
        ("stateWatchers", json!([{"watcher":"unreviewed"}])),
        ("inheritedSource", json!({"baseClass":"Unreviewed"})),
    ] {
        let mut changed = catalogue.clone();
        for entry in changed["cards"].as_array_mut().unwrap() {
            if entry["setCode"]
                .as_str()
                .is_some_and(|code| focus.contains(&code))
            {
                entry[field] = value.clone();
            }
        }
        let loaded = crate::ability_ir::Catalogue::load(&changed).unwrap();
        for code in focus {
            assert!(loaded.rejected.contains_key(code), "{code}:{field}");
            assert!(!loaded.cards.contains_key(code), "{code}:{field}");
        }
    }
}
