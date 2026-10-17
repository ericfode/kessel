use super::*;

// Source-derived witnesses for UntilEndOfAttack stat modifications
// (`CardAttackLastingEffectSystem`). Printed sources are the seven converted
// cards; the synthetic programs exercise the gate and the attack-end sweep.

fn attack(e: &Engine, s: State, source: CardId, target: CardId) -> State {
    let a = e.card_action(&s, source);
    assert!(e.legal_actions(&s).contains(&a));
    let prompt = e.apply(&s, &a).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let a = e.card_action(&prompt, target);
    assert!(e.legal_actions(&prompt).contains(&a));
    e.apply(&prompt, &a).unwrap()
}
fn attach(e: &Engine, s: &mut State, seat: Seat, code: &str, host: CardId) -> CardId {
    let id = add(e, s, seat, code);
    e.attach(s, id, host);
    id
}
fn resources(e: &Engine, s: &mut State, seat: Seat, n: usize) {
    for _ in 0..n {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
}
/// Intrinsic keyword triggers (Saboteur) share the On Attack window; the engine
/// presents the order prompt whenever at least one registered trigger is
/// effective, so the witnesses choose the card's own ability by title.
fn choose_trigger(e: &Engine, s: &State, needle: &str) -> State {
    assert_eq!(s.decision, Decision::TriggerOrder);
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label.contains(needle))
        .unwrap_or_else(|| {
            panic!(
                "missing trigger {needle}: {:?}",
                e.legal_actions(s).iter().map(|a| a.label.clone()).collect::<Vec<_>>()
            )
        });
    e.apply(s, &a).unwrap()
}

#[test]
fn attack_duration_family_admission_and_exact_gate_boundaries() {
    let e = engine();
    for code in [
        "ASH_172", "IBH_010", "JTL_238", "LOF_138", "SHD_118", "SHD_151", "TWI_179",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    // Every converted program stores the printed UntilEndOfAttack duration.
    for code in ["ASH_172", "IBH_010", "JTL_238", "SHD_118", "SHD_151", "TWI_179"] {
        let d = &e.definitions[e.by_code[code] as usize];
        let found = d.programs.card.iter().any(|&p| {
            e.program(p).instructions.iter().any(|i| {
                matches!(i, Instruction::Apply(Effect::ModifyStats { duration, .. }) if duration == "attack")
            }) || e.program(p).instructions.iter().any(|i| {
                matches!(i, Instruction::IfResult{..}) && e.program(p).instructions.iter().any(|j| matches!(j, Instruction::Apply(Effect::ModifyStats { duration, .. }) if duration == "attack"))
            })
        });
        assert!(found, "{code} has no attack-duration stat instruction");
    }
    assert!(e.by_code.contains_key("LOF_138"));
    let compile = |value: Value| {
        let ability: crate::ability_ir::Ability = serde_json::from_value(value).unwrap();
        crate::ability_ir::Program::compile(&ability)
    };
    let grant = json!({"kind":"modifyStats","target":"self","duration":"attack","power":{"kind":"const","value":2},"hp":{"kind":"const","value":0}});
    let attack_targets = json!({"kind":"query","name":"getAllAttackTargets","receiver":{"kind":"ref","path":["event","attack"]},"args":[]});
    // Exact admitted shapes: self, the attack's captured target list, literal and
    // dynamic numeric values, and the printed negative literal.
    assert!(compile(json!({"trigger":"onAttack","title":"Self","effect":grant})).is_ok());
    assert!(compile(json!({"trigger":"onAttack","title":"Defender","effect":{"kind":"modifyStats","target":attack_targets,"duration":"attack","power":{"kind":"unary","op":"-","value":{"kind":"const","value":2}},"hp":{"kind":"const","value":0}}})).is_ok());
    assert!(compile(json!({"trigger":"onAttack","title":"Count","effect":{"kind":"modifyStats","target":"self","duration":"attack","power":{"kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["opponent"]},"args":[{"kind":"binary","op":">","left":{"kind":"ref","path":["card","damage"]},"right":{"kind":"const","value":0}}]},"hp":{"kind":"const","value":0}}})).is_ok());
    // Other triggers cannot observe the attack window.
    let err = compile(json!({"trigger":"whenPlayed","title":"Wrong window","effect":grant})).unwrap_err();
    assert!(err.contains("On Attack trigger"), "{err}");
    // Grouped stats keep their phase-stat prepared windows.
    let err = compile(json!({"trigger":"onAttack","title":"Grouped","effect":{"kind":"simultaneous","effects":[grant,{"kind":"damage","target":"enemyBase","amount":{"kind":"const","value":1}}]}})).unwrap_err();
    assert!(err.contains("grouped stat modifications require phase duration"), "{err}");
    // Arena lists are not one resolved card target.
    let err = compile(json!({"trigger":"onAttack","title":"Arena","effect":{"kind":"modifyStats","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"const","value":true}},"duration":"attack","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}}})).unwrap_err();
    assert!(err.contains("self or the current attack's target list") || err.contains("arena-list phase stats"), "{err}");
    // Engine possible target shapes only: CardAttackLastingEffectSystem
    // Contract-asserts `target.activeAttack`, so a chosen card, a base or an
    // arbitrary card/player reference has no active attack and stays rejected
    // instead of granting an effect the engine would not create.
    for (label, target) in [
        ("chosen", json!("chosen")),
        ("parent", json!("parent")),
        ("ownBase", json!("ownBase")),
        ("source ref", json!({"kind":"ref","path":["source"]})),
        ("card ref", json!({"kind":"ref","path":["card"]})),
        ("target ref", json!({"kind":"ref","path":["target"]})),
        ("own base ref", json!({"kind":"ref","path":["player","base"]})),
        ("opponent base ref", json!({"kind":"ref","path":["opponent","base"]})),
        ("conditional source", json!({"kind":"conditional","condition":{"kind":"const","value":true},"then":{"kind":"ref","path":["source"]},"else":{"kind":"const","value":null}})),
    ] {
        let err = compile(json!({"trigger":"onAttack","title":"Foreign target","effect":{"kind":"modifyStats","target":target,"duration":"attack","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}}})).unwrap_err();
        assert!(err.contains("self or the current attack's target list"), "{label}: {err}");
    }
    // The target list must be the attack event's, and it is not a number.
    let wrong_receiver = json!({"kind":"query","name":"getAllAttackTargets","receiver":{"kind":"ref","path":["player"]},"args":[]});
    let err = compile(json!({"trigger":"onAttack","title":"Bad receiver","effect":{"kind":"modifyStats","target":wrong_receiver,"duration":"attack","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}}})).unwrap_err();
    assert!(err.contains("self or the current attack's target list"), "{err}");
    // The attack target list has exactly one consumer: the unit-only lasting
    // stat system. A generic effect target would silently skip bases and
    // departed defenders, so every other consumer is an explicit rejection.
    for effect in [
        json!({"kind":"damage","target":attack_targets,"amount":{"kind":"const","value":1}}),
        json!({"kind":"defeat","target":attack_targets}),
        json!({"kind":"returnToHand","target":attack_targets}),
        json!({"kind":"ready","target":attack_targets}),
        json!({"kind":"modifyStats","target":attack_targets,"duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}}),
        json!({"kind":"simultaneous","effects":[{"kind":"damage","target":attack_targets,"amount":{"kind":"const","value":1}},{"kind":"noAction"}]}),
    ] {
        let err = compile(json!({"trigger":"onAttack","title":"Wrong consumer","effect":effect.clone()})).unwrap_err();
        assert!(
            err.contains("effect target must be a bounded card or player reference")
                || err.contains("self or the current attack")
                || err.contains("array targets require verified event batching"),
            "{effect}: {err}"
        );
    }
    // Same query outside the single admitted consumer is not even a valid
    // expression: the generic validator rejects the query name, so a future
    // amount/condition/target consumer cannot quietly use the attack snapshot.
    let err = compile(json!({"trigger":"onAttack","title":"Amount","effect":{"kind":"damage","target":"enemyBase","amount":attack_targets}})).unwrap_err();
    assert!(err.contains("unsupported runtime query getAllAttackTargets"), "{err}");
    let err = compile(json!({"trigger":"onAttack","title":"Condition","condition":attack_targets,"effect":{"kind":"noAction"}})).unwrap_err();
    assert!(err.contains("unsupported runtime query getAllAttackTargets"), "{err}");
}

#[test]
fn sith_trooper_counts_only_damaged_units_the_defending_player_controls() {
    let e = engine();
    for damaged in 1..=2 {
        let mut s = board(&e);
        s.round = 5;
        s.active = Seat::P1;
        s.action_player = Some(Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "JTL_238");
        let defender = unit(&e, &mut s, Seat::P2, "HMW_082");
        for _ in 0..damaged {
            let id = unit(&e, &mut s, Seat::P2, "HMW_082");
            s.card_mut(id).damage = 1;
        }
        // A damaged friendly unit and a damaged card outside the arena must not count.
        let friendly = unit(&e, &mut s, Seat::P1, "HMW_082");
        s.card_mut(friendly).damage = 1;
        let discarded = add(&e, &mut s, Seat::P2, "HMW_082");
        s.card_mut(discarded).damage = 2;
        s.player_mut(Seat::P2).discard.push(discarded).unwrap();
        let done = attack(&e, s, attacker, defender);
        assert_eq!(
            done.card(defender).damage,
            3 + damaged as u16,
            "attack power includes one per damaged defending unit"
        );
        assert_eq!(done.card(attacker).damage, 1, "defender's power is unchanged");
        assert_eq!(done.attack_stats.len, 0, "attack cleanup removes the grant");
        assert_eq!(e.power(&done, attacker, false), 3, "printed power after cleanup");
    }
}

#[test]
fn han_solo_debuffs_the_unit_defender_and_has_no_effect_on_a_base() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "IBH_010");
    let defender = unit(&e, &mut s, Seat::P2, "HMW_082");
    assert_eq!(e.power(&s, attacker, true), 6, "printed 4 + Raid 2 while attacking");
    let done = attack(&e, s, attacker, defender);
    assert_eq!(done.card(defender).damage, 6);
    assert_eq!(done.card(attacker).damage, 0, "defender's 1 power is reduced by 2 for this attack");
    assert_eq!(e.power(&done, defender, false), 1, "modifier expired with the attack");
    assert_eq!(done.attack_stats.len, 0);
    // A base defender is not a unit: the engine's unit-only canAffect drops the
    // effect before it creates any lasting effect.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "IBH_010");
    let base = s.player(Seat::P2).base;
    let done = attack(&e, s, attacker, base);
    assert_eq!(done.card(base).damage, 6);
    assert_eq!(done.attack_stats.len, 0);
}

#[test]
fn razor_crest_optional_discard_grants_power_and_empty_hand_resolves_without_one() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_172");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let discarded = add(&e, &mut s, Seat::P1, "LOF_254");
    s.player_mut(Seat::P1).hand.push(discarded).unwrap();
    let a = e.card_action(&s, attacker);
    let prompt = e.apply(&s, &a).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    let prompt = choose_trigger(&e, &prompt, "Discard a card from your hand");
    assert_eq!(prompt.decision, Decision::AbilityOptional, "You may discard a card");
    let prompt = named_button(&e, &prompt, "Trigger");
    assert_eq!(prompt.decision, Decision::AbilityDiscard);
    let done = e.apply(&prompt, &e.card_action(&prompt, discarded)).unwrap();
    assert!(done.player(Seat::P1).discard.contains(discarded));
    assert_eq!(done.card(defender).damage, 5, "printed 3 + 2 for this attack");
    assert_eq!(done.attack_stats.len, 0);
    // Empty hand: the triggered ability is still offered (the engine filters
    // triggered abilities at the trigger stage without checking effect targets),
    // the discard group produces no event, and the attack resolves unmodified.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_172");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    // Both registrations are ineffective (no card to discard, no Shield to
    // defeat), so the trigger window resolves without a prompt.
    assert_eq!(prompt.decision, Decision::Window);
    let done = prompt;
    assert_eq!(done.card(defender).damage, 3, "no discard, no bonus");
    assert_eq!(done.attack_stats.len, 0);
}

#[test]
fn kihraxz_optional_exhaust_grants_power_and_declined_or_ineffective_choices_do_not() {
    let e = engine();
    // Accept: another friendly unit is exhausted and the grant applies.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_118");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let ally = unit(&e, &mut s, Seat::P1, "LOF_254");
    let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    // Optional targeted abilities express the decline as the prompt's Pass button.
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    assert!(e.legal_actions(&prompt).iter().any(|a| a.label == "Pass"));
    assert!(!e.legal_actions(&prompt).iter().any(|a| a.id() == Some(attacker)), "source is excluded");
    let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
    assert!(done.card(ally).exhausted);
    assert_eq!(done.card(defender).damage, 6, "printed 3 + 3 for this attack");
    assert_eq!(done.attack_stats.len, 0);
    // Decline: no exhaustion and no grant.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_118");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let ally = unit(&e, &mut s, Seat::P1, "LOF_254");
    let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    let done = named_button(&e, &prompt, "Pass");
    assert!(!done.card(ally).exhausted);
    assert_eq!(done.card(defender).damage, 3);
    // Selected-but-ineffective: an already-exhausted ally stays selectable at
    // the selector stage, the exhaust event fails, and "If you do" does not run.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "SHD_118");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let ally = unit(&e, &mut s, Seat::P1, "LOF_254");
    let ready = unit(&e, &mut s, Seat::P1, "LOF_254");
    s.card_mut(ally).exhausted = true;
    let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    assert!(e.legal_actions(&prompt).iter().any(|a| a.id() == Some(ready)));
    assert!(e.legal_actions(&prompt).iter().any(|a| a.id() == Some(ally)));
    let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
    assert_eq!(done.card(defender).damage, 3, "failed exhaust skips the grant");
    assert_eq!(done.attack_stats.len, 0);
}

#[test]
fn valiant_assault_ship_condition_uses_live_resource_counts() {
    let e = engine();
    for (own, enemy, expected) in [(0, 2, 5), (2, 2, 3), (3, 2, 3)] {
        let mut s = board(&e);
        s.round = 5;
        s.active = Seat::P1;
        s.action_player = Some(Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "SHD_151");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
        resources(&e, &mut s, Seat::P1, own);
        resources(&e, &mut s, Seat::P2, enemy);
        let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
        let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
        // The conditional grant and the intrinsic Saboteur trigger share the
        // window only when the resource condition holds; otherwise both are
        // ineffective and the window resolves without a prompt.
        let done = if prompt.decision == Decision::TriggerOrder {
            choose_trigger(&e, &prompt, "more resources than you")
        } else {
            assert_eq!(prompt.decision, Decision::Window);
            prompt
        };
        assert_eq!(done.card(defender).damage, expected, "own/enemy resources {own}/{enemy}");
        assert_eq!(done.attack_stats.len, 0);
    }
}

#[test]
fn soulless_one_targets_only_a_friendly_droid_or_general_grievous() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "TWI_179");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_069");
    let droid = unit(&e, &mut s, Seat::P1, "ASH_029");
    let non_droid = unit(&e, &mut s, Seat::P1, "LOF_254");
    let enemy_droid = unit(&e, &mut s, Seat::P2, "ASH_029");
    let prompt = e.apply(&s, &e.card_action(&s, attacker)).unwrap();
    let prompt = e.apply(&prompt, &e.card_action(&prompt, defender)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    assert!(e.legal_actions(&prompt).iter().any(|a| a.label == "Pass"));
    let ids: Vec<_> = e.legal_actions(&prompt).iter().filter_map(Action::id).collect();
    assert!(ids.contains(&droid));
    assert!(!ids.contains(&non_droid), "a non-Droid friendly unit is not a legal target");
    assert!(!ids.contains(&enemy_droid), "the controller check excludes enemy units");
    let done = e.apply(&prompt, &e.card_action(&prompt, droid)).unwrap();
    assert!(done.card(droid).exhausted);
    assert_eq!(done.card(defender).damage, 3, "printed 1 + 2 for this attack");
    assert_eq!(done.attack_stats.len, 0);
}

#[test]
fn sith_holocron_grants_the_attached_unit_an_attack_duration_boost() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    // HMW_128 has no card programs of its own, so the only prompt is the
    // upgrade-granted On Attack ability. The host is a Force unit (attach
    // condition) and attacks the base so both combatants stay readable.
    let host = unit(&e, &mut s, Seat::P1, "HMW_128");
    let friendly = unit(&e, &mut s, Seat::P1, "HMW_082");
    let base = s.player(Seat::P2).base;
    attach(&e, &mut s, Seat::P1, "LOF_138", host);
    assert_eq!(e.power(&s, host, false), 5, "printed 4 + upgrade 1");
    let prompt = e.apply(&s, &e.card_action(&s, host)).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let prompt = e.apply(&prompt, &e.card_action(&prompt, base)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget, "gained On Attack ability");
    let done = e.apply(&prompt, &e.card_action(&prompt, friendly)).unwrap();
    assert_eq!(done.card(friendly).damage, 2, "2 damage to a friendly unit");
    assert_eq!(done.card(base).damage, 7, "host 5 power + 2 for this attack");
    assert_eq!(done.attack_stats.len, 0);
    assert_eq!(e.power(&done, host, false), 5, "expired with the attack");
}

#[test]
fn attack_duration_grants_expire_when_the_attack_ends_without_combat_damage() {
    // The synthetic registration mirrors `forThisAttackCardEffect` on the
    // defender followed by the attacker being defeated before combat. The
    // engine still runs AttackFlow.cleanUpAttack, so the defender's modifier
    // must be removed even though no combat damage event was generated.
    let mut e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let defender = unit(&e, &mut s, Seat::P2, "HMW_082");
    register_keyword_program(&mut e, &s, attacker, json!({
        "trigger":"onAttack","title":"Debuff the defender, then leave play",
        "effect":{"kind":"modifyStats","target":{"kind":"query","name":"getAllAttackTargets","receiver":{"kind":"ref","path":["event","attack"]},"args":[]},"duration":"attack","power":{"kind":"unary","op":"-","value":{"kind":"const","value":2}},"hp":{"kind":"const","value":0}},
        "then":{"title":"Defeat this unit","effect":{"kind":"defeat","target":"self"}}
    }));
    let done = attack(&e, s, attacker, defender);
    assert!(done.player(Seat::P1).discard.contains(attacker));
    assert!(e.in_arena(&done, defender));
    assert_eq!(done.attack_stats.len, 0, "attack cleanup removes the defender grant");
    assert_eq!(e.power(&done, defender, false), 1);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn attack_duration_store_is_bounded_hashed_and_target_scoped() {
    let e = engine();
    let mut s = board(&e);
    let a = unit(&e, &mut s, Seat::P1, "HMW_082");
    let b = unit(&e, &mut s, Seat::P2, "HMW_082");
    let outsider = unit(&e, &mut s, Seat::P1, "HMW_082");
    s.source = a;
    s.attack_source = a;
    s.attack_defender = b;
    let before = e.power(&s, a, false);
    let empty_hash = s.zobrist();
    e.grant_attack_stats(&mut s, a, 2, 0).unwrap();
    assert_eq!(e.power(&s, a, false), before + 2);
    assert_ne!(s.zobrist(), empty_hash);
    let copy = s;
    assert_eq!(copy.zobrist(), s.zobrist());
    // The engine asserts `target.activeAttack`: a foreign in-play unit is not a
    // participant of the resolving attack and cannot receive a grant, even
    // outside the admission gate.
    let err = e.grant_attack_stats(&mut s, outsider, 1, 0).unwrap_err();
    assert!(err.contains("participant of the resolving attack"), "{err}");
    let mut idle = board(&e);
    let idle_unit = unit(&e, &mut idle, Seat::P1, "HMW_082");
    idle.source = idle_unit;
    let err = e.grant_attack_stats(&mut idle, idle_unit, 1, 0).unwrap_err();
    assert!(err.contains("participant of the resolving attack"), "{err}");
    e.remove_attack_stats_target(&mut s, b);
    assert_eq!(e.power(&s, a, false), before + 2, "other targets keep their grants");
    e.remove_attack_stats_target(&mut s, a);
    assert_eq!(s.zobrist(), empty_hash);
    // Capacity is explicit and mutation is atomic on failure.
    assert!(e.grant_attack_stats(&mut s, 0, 1, 0).is_err());
    assert!(e.grant_attack_stats(&mut s, 99, 1, 0).is_err());
    assert!(e.grant_attack_stats(&mut s, a, 40000, 0).is_err());
    let mut full = board(&e);
    let target = unit(&e, &mut full, Seat::P1, "HMW_082");
    let defender = unit(&e, &mut full, Seat::P2, "HMW_082");
    full.source = target;
    full.attack_source = target;
    full.attack_defender = defender;
    for _ in 0..crate::model::MAX_ATTACK_STAT_GRANTS {
        e.grant_attack_stats(&mut full, target, 1, 0).unwrap();
    }
    let before = full;
    let err = e.grant_attack_stats(&mut full, target, 1, 0).unwrap_err();
    assert!(err.contains("capacity"), "{err}");
    assert_eq!(full, before);
    // Attack grants never outlive a clean attack: the sweep is global.
    e.clear_attack_stats(&mut full);
    assert_eq!(full.attack_stats.len, 0);
}
