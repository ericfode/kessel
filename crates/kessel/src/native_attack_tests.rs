use super::*;
use crate::ability_ir::{Ability, Catalogue, Effect, Program};

// Full pinned card sources + InitiateAttackSystem / InitiateAttackAction /
// AttackStepsSystem establish these outcomes before executing the Rust fixtures.
// These are direct named-card midgame assertions, not JS differential evidence.
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
        "missing {id} {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn play(e: &Engine, s: &State, code: &str) -> (State, CardId) {
    let mut s = *s;
    let seat = s.active;
    let id = add(e, &mut s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    for _ in 0..12 {
        let r = add(e, &mut s, seat, "LOF_093");
        s.player_mut(seat).resources.push(r).unwrap();
    }
    (click(e, &s, id), id)
}
fn attacker(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    unit(e, s, seat, code)
}
fn finish_triggers(e: &Engine, mut s: State) -> State {
    for _ in 0..20 {
        if s.decision == Decision::Window || s.decision == Decision::Terminal {
            return s;
        }
        let a = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Trigger")
            .or_else(|| {
                e.legal_actions(&s)
                    .into_iter()
                    .find(|a| a.label == "Restore 1")
            })
            .unwrap_or_else(|| panic!("unexpected {:?} {:?}", s.decision, e.legal_actions(&s)));
        s = e.apply(&s, &a).unwrap();
    }
    panic!("trigger loop")
}
fn roundtrip(e: &Engine, s: &State) {
    let v = e.snapshot(s);
    let t = e.load_state(&v).unwrap();
    assert_eq!(e.snapshot(&t), v);
    assert_eq!(t.zobrist(), s.zobrist());
}

#[test]
fn native_attack_literal_event_cards_apply_power_once_and_cleanup_after_combat() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            for (code, bonus) in [("IBH_021", 2), ("SHD_231", 3), ("SHD_179", 2)] {
                let mut s = start(&e, seat);
                let a = attacker(&e, &mut s, seat, "LOF_093");
                let enemy = attacker(&e, &mut s, seat.other(), "LOF_256");
                s.card_mut(a).damage = 1;
                let before = e.power(&s, a, true);
                let (select, event) = play(&e, &s, code);
                assert_eq!(select.decision, Decision::AbilityTarget);
                assert!(select.player(seat).hand.contains(event));
                let choose = click(&e, &select, a);
                assert_eq!(choose.decision, Decision::AttackTarget);
                assert_eq!(e.power(&choose, a, true), before + bonus);
                assert!(choose.player(seat).discard.contains(event));
                assert_eq!(choose.support.context.source, event);
                assert!(e
                    .legal_actions(&choose)
                    .iter()
                    .all(|x| x.label != "Pass attack"));
                let done = finish_triggers(&e, click(&e, &choose, enemy));
                assert!(done.card(a).exhausted);
                assert_eq!(done.support.len, 0);
                assert!(!done.support.pending);
                assert_eq!(e.power(&done, a, false), before);
                assert_eq!(done.active, seat.other());
                roundtrip(&e, &done);
            }
        }
    }
}
#[test]
fn native_attack_breaking_in_saboteur_applies_before_defender_selection() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        let mut s = start(&e, Seat::P1);
        let a = attacker(&e, &mut s, Seat::P1, "LOF_093");
        let sentinel = attacker(&e, &mut s, Seat::P2, "LOF_044");
        let base = s.player(Seat::P2).base;
        let (select, _) = play(&e, &s, "TWI_224");
        let choose = click(&e, &select, a);
        assert_eq!(e.keyword_amount(&choose, a, "saboteur"), 1);
        assert!(e
            .legal_actions(&choose)
            .iter()
            .any(|x| x.id() == Some(base)));
        assert!(e
            .legal_actions(&choose)
            .iter()
            .any(|x| x.id() == Some(sentinel)));
        let done = finish_triggers(&e, click(&e, &choose, base));
        assert_eq!(done.card(base).damage, 4);
        assert_eq!(e.keyword_amount(&done, a, "saboteur"), 0);
        assert_eq!(e.power(&done, a, false), 2);
    }
}
#[test]
fn native_attack_optional_hoth_keeps_exhaustion_and_parent_trigger_identity() {
    for code in ["IBH_064"] {
        let e = engine();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat);
            let a = attacker(&e, &mut s, seat, "LOF_093");
            let base = s.player(seat.other()).base;
            let (mut select, source) = play(&e, &s, code);
            if select.decision == Decision::AbilityOptional {
                select = named_button(&e, &select, "Trigger");
            }
            assert_eq!(select.decision, Decision::AbilityTarget);
            assert!(!e
                .legal_actions(&select)
                .iter()
                .any(|x| x.id() == Some(source)));
            let choose = click(&e, &select, a);
            assert!(choose.card(source).exhausted);
            assert!(e
                .legal_actions(&choose)
                .iter()
                .any(|x| x.label == "Pass attack"));
            let passed = named_button(&e, &choose, "Pass attack");
            assert!(!passed.card(a).exhausted);
            assert!(!passed.support.pending);
            assert_eq!(
                e.power(&passed, a, false),
                2 + if code == "IBH_064" { 2 } else { 0 }
            );
            roundtrip(&e, &passed);
            let done = finish_triggers(&e, click(&e, &choose, base));
            assert_eq!(
                done.card(base).damage,
                2 + if code == "IBH_064" { 2 } else { 0 }
            );
            assert_eq!(done.active, seat.other());
            assert!(!done.support.pending);
        }
    }
}
#[test]
fn native_attack_cancel_before_payment_is_atomic_and_paid_attack_is_committed() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = start(&e, seat);
            let a = attacker(&e, &mut s, seat, "LOF_093");
            let base = s.player(seat.other()).base;
            // An ordinary external attack remains cancellable before exhaustion.
            let ordinary = click(&e, &s, a);
            assert_eq!(e.legal_actions(&ordinary), vec![Action::button("Cancel", "cancel"), e.card_action(&ordinary, base)]);
            let cancelled = named_button(&e, &ordinary, "Cancel");
            assert!(!cancelled.card(a).exhausted);
            let (select, event) = play(&e, &s, "IBH_021");
            let ready = e.ready_resources(&select, seat);
            let cancelled = named_button(&e, &select, "Cancel");
            assert!(cancelled.player(seat).hand.contains(event));
            assert_eq!(e.ready_resources(&cancelled, seat), ready);
            assert_eq!(e.power(&cancelled, a, false), 2);
            let choose = click(&e, &select, a);
            assert_eq!(e.legal_actions(&choose), vec![e.card_action(&choose, base)]);
            assert!(e.apply(&choose, &Action::button("Cancel", "cancel")).is_err());
            assert!(e.apply(&choose, &Action::button("Pass attack", "passAbility")).is_err());
            assert!(!choose.card(a).exhausted);
            assert!(choose.player(seat).discard.contains(event));
            assert!(e.ready_resources(&choose, seat) < ready);
            assert_eq!(e.power(&choose, a, false), 4);
            let done = finish_triggers(&e, click(&e, &choose, base));
            assert_eq!(done.card(base).damage, 4);
            assert!(done.card(a).exhausted);
            assert_eq!(e.power(&done, a, false), 2);
            assert_eq!(done.support.len, 0);
        }
    }
}

#[test]
fn native_attack_grim_resolve_live_grit_and_pounce_trait_selection() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let a = attacker(&e, &mut s, Seat::P1, "LOF_093");
    s.card_mut(a).damage = 1;
    let base = s.player(Seat::P2).base;
    let (select, _) = play(&e, &s, "TWI_172");
    let choose = click(&e, &select, a);
    assert_eq!(e.keyword_amount(&choose, a, "grit"), 1);
    assert_eq!(e.power(&choose, a, true), 3);
    let done = finish_triggers(&e, click(&e, &choose, base));
    assert_eq!(done.card(base).damage, 3);
    assert_eq!(e.keyword_amount(&done, a, "grit"), 0);
    let mut s = start(&e, Seat::P1);
    let beast = attacker(&e, &mut s, Seat::P1, "TOKEN_BEAST");
    let other = attacker(&e, &mut s, Seat::P1, "LOF_093");
    let enemy = attacker(&e, &mut s, Seat::P2, "TOKEN_BEAST");
    let (select, _) = play(&e, &s, "LOF_224");
    let ids = e
        .legal_actions(&select)
        .iter()
        .filter_map(Action::id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![beast]);
    assert!(!ids.contains(&other));
    assert!(!ids.contains(&enemy));
    let choose = click(&e, &select, beast);
    let base = choose.player(Seat::P2).base;
    let done = finish_triggers(&e, click(&e, &choose, base));
    assert_eq!(done.card(base).damage, 7);
}
#[test]
fn native_attack_action_senator_targeting_payment_and_self_exhaustion_boundary() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = start(&e, seat);
        let actor = attacker(&e, &mut s, seat, "TWI_105");
        let a = attacker(&e, &mut s, seat, "LOF_093");
        for _ in 0..3 {
            let r = add(&e, &mut s, seat, "LOF_093");
            s.player_mut(seat).resources.push(r).unwrap();
        }
        let menu = click(&e, &s, actor);
        let title = e.program(e.card_programs(&s, actor)[0]).title.clone();
        let select = named_button(&e, &menu, &title);
        let ready = e.ready_resources(&select, seat);
        let cancel = named_button(&e, &select, "Cancel");
        assert_eq!(e.ready_resources(&cancel, seat), ready);
        assert!(!cancel.card(actor).exhausted);
        let choose = click(&e, &select, a);
        assert!(choose.card(actor).exhausted);
        assert_eq!(e.ready_resources(&choose, seat), ready - 2);
        let base = choose.player(seat.other()).base;
        let done = finish_triggers(&e, click(&e, &choose, base));
        assert_eq!(done.card(base).damage, 4);
        assert!(done.card(a).exhausted);
        // Selecting the cost source is legal early, but payment exhausts it before
        // InitiateAttackSystem.checkEventCondition, so the attack event cannot resolve.
        let done = click(&e, &select, actor);
        assert_eq!(done.decision, Decision::Window);
        assert!(done.card(actor).exhausted);
        assert_eq!(done.support.len, 0);
        assert_eq!(done.active, seat.other());
    }
}
#[test]
fn native_attack_direct_source_condition_families_and_suspend_on_attack_trigger() {
    let e = engine();
    for (code, target_code, bonus) in [
        ("IBH_064", "LOF_093", 2),
        ("TWI_091", "HMW_146", 2),
        ("LOF_111", "LOF_093", 2),
    ] {
        let mut s = start(&e, Seat::P1);
        let a = attacker(&e, &mut s, Seat::P1, target_code);
        let base = s.player(Seat::P2).base;
        let (select, _) = play(&e, &s, code);
        let select = if select.decision == Decision::AbilityOptional {
            named_button(&e, &select, "Trigger")
        } else {
            select
        };
        let before = e.power(&select, a, true);
        let choose = click(&e, &select, a);
        let done = finish_triggers(&e, click(&e, &choose, base));
        assert_eq!(done.card(base).damage, before + bonus, "{code}");
    }
    // Benthic's actual On Attack selector suspends combat. Native initiation must
    // restore its event source only after that window and combat have completed.
    let mut s = start(&e, Seat::P1);
    let a = attacker(&e, &mut s, Seat::P1, "LAW_057");
    let victim = attacker(&e, &mut s, Seat::P2, "LOF_093");
    let base = s.player(Seat::P2).base;
    let (select, event) = play(&e, &s, "IBH_021");
    let choose = click(&e, &select, a);
    let trigger = click(&e, &choose, base);
    assert!(trigger.support.pending);
    assert_ne!(trigger.decision, Decision::Window);
    assert_eq!(trigger.support.context.source, event);
    let trigger = if trigger.decision == Decision::AbilityOptional {
        named_button(&e, &trigger, "Trigger")
    } else {
        trigger
    };
    let trigger = if trigger.decision == Decision::AbilityTarget {
        click(&e, &trigger, victim)
    } else {
        trigger
    };
    let done = finish_triggers(&e, trigger);
    assert!(!done.support.pending);
    assert_eq!(done.active, Seat::P2);
    assert!(done.card(a).exhausted);
}
#[test]
fn native_attack_raw_contracts_reject_unknowns_late_callbacks_and_partial_sources() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["setCode"] == "TWI_224")
        .unwrap()
        .clone();
    let admitted = |entry: Value| {
        let code = entry["setCode"].as_str().unwrap().to_owned();
        let c = Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[entry]})).unwrap();
        assert!(c.rejected.contains_key(&code));
    };
    assert!(Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[original.clone()]})).unwrap().cards.contains_key("TWI_224"));
    for key in ["stateWatchers", "historyHelpers", "privatePrograms", "classMethods", "privateClassConstants", "inheritedSource", "deploymentOverride", "setupValidationOverrides"] {
        let mut bad=original.clone();
        bad[key]=json!([{"kind":"unimplementedSourceContext"}]);
        admitted(bad);
    }
    for field in [
        "targetCondition",
        "allowExhaustedAttacker",
        "ignoredRequirements",
        "isAmbush",
        "defenderLastingEffects",
        "attackerCombatDamageOverride",
        "optional",
    ] {
        let mut x = original.clone();
        x["abilities"]["card"][0]["effect"]["properties"]["properties"][field] = json!(false);
        admitted(x);
    }
    for mode in [
        "factory",
        "late",
        "target",
        "hp",
        "source",
        "side",
        "continuation",
        "nested",
    ] {
        let mut x = original.clone();
        match mode {
            "factory" => {
                x["abilities"]["card"][0]["effect"]["properties"]["kind"] =
                    json!("attackPropertiesFactory")
            }
            "late" => {
                x["abilities"]["card"][0]["effect"]["properties"]["properties"]
                    ["attackerLastingEffects"][0]["evaluation"] = json!("afterAttackCreated")
            }
            "target" => {
                x["abilities"]["card"][0]["effect"]["properties"]["properties"]["target"] =
                    json!("self")
            }
            "hp" => {
                x["abilities"]["card"][0]["effect"]["properties"]["properties"]
                    ["attackerLastingEffects"][1]["effect"]["hp"] =
                    json!({"kind":"const","value":2})
            }
            "source" => x["card"]["types"] = json!(["leader"]),
            "side" => x["abilities"]["unit"] = x["abilities"]["card"].clone(),
            "continuation" => {
                x["abilities"]["card"][0]["then"] = json!({"title":"extra","effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}})
            }
            _ => {
                let e = x["abilities"]["card"][0]["effect"].clone();
                x["abilities"]["card"][0]["effect"] = json!({"kind":"sequence","effects":[e]});
            }
        }
        admitted(x);
    }
    let a: Ability = serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();
    assert!(Program::compile(&a).is_ok());
    let Effect::InitiateAttack { properties, .. } = &a.effect else {
        panic!()
    };
    assert_eq!(properties.modifiers().unwrap().power, 2);
}

#[test]
fn native_attack_additional_complete_action_and_play_sources() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for (code, target_code, bonus) in [("IBH_023", "LOF_093", 2), ("JTL_146", "TOKEN_XWING", 2)]
        {
            let mut s = start(&e, seat);
            let actor = attacker(&e, &mut s, seat, code);
            let a = attacker(&e, &mut s, seat, target_code);
            let wrong = attacker(&e, &mut s, seat, "LOF_254");
            let enemy = attacker(&e, &mut s, seat.other(), target_code);
            let base = s.player(seat.other()).base;
            let menu = click(&e, &s, actor);
            let title = e.program(e.card_programs(&s, actor)[0]).title.clone();
            let select = if menu.decision == Decision::AbilityMenu {
                named_button(&e, &menu, &title)
            } else {
                menu
            };
            let choices = e
                .legal_actions(&select)
                .iter()
                .filter_map(Action::id)
                .collect::<Vec<_>>();
            assert_eq!(choices, vec![a], "{code}");
            assert!(!choices.contains(&wrong));
            assert!(!choices.contains(&enemy));
            assert!(!choices.contains(&actor));
            let before = e.power(&select, a, true);
            let choose = click(&e, &select, a);
            assert!(choose.card(actor).exhausted);
            let done = finish_triggers(&e, click(&e, &choose, base));
            assert_eq!(done.card(base).damage, before + bonus, "{code}");
            assert!(done.card(a).exhausted);
        }
        let mut s = start(&e, seat);
        let a = attacker(&e, &mut s, seat, "IBH_004");
        let before = e.power(&s, a, true);
        let base = s.player(seat.other()).base;
        let (select, _) = play(&e, &s, "JTL_231");
        let choose = click(&e, &select, a);
        let done = finish_triggers(&e, click(&e, &choose, base));
        assert_eq!(done.card(base).damage, before + 2);
        let mut s = start(&e, seat);
        let a = attacker(&e, &mut s, seat, "LOF_093");
        let base = s.player(seat.other()).base;
        let (select, maul) = play(&e, &s, "TS26_030");
        let select = if select.decision == Decision::AbilityOptional {
            named_button(&e, &select, "Trigger")
        } else {
            select
        };
        assert!(!e
            .legal_actions(&select)
            .iter()
            .any(|a| a.id() == Some(maul)));
        let choose = click(&e, &select, a);
        let done = finish_triggers(&e, click(&e, &choose, base));
        assert_eq!(done.card(base).damage, 2);
        assert_eq!(e.keyword_amount(&done, maul, "sentinel"), 1);
        assert!(done.card(maul).exhausted);
    }
}
#[test]
fn native_attack_immutable_copy_hash_capacity_and_forged_import_boundaries() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let a = attacker(&e, &mut s, Seat::P1, "LOF_093");
    let (select, _) = play(&e, &s, "IBH_064");
    let select = if select.decision == Decision::AbilityOptional { named_button(&e, &select, "Trigger") } else { select };
    let choose = click(&e, &select, a);
    let copy = choose;
    assert_eq!(copy, choose);
    assert_eq!(copy.zobrist(), choose.zobrist());
    let mut mutated = copy;
    mutated.support.grants[0].power += 1;
    assert_ne!(copy.zobrist(), mutated.zobrist());
    let mut mutated = copy;
    mutated.support.optional_attack = !copy.support.optional_attack;
    assert_ne!(copy.zobrist(), mutated.zobrist());
    let passed = named_button(&e, &choose, "Pass attack");
    roundtrip(&e, &passed);
    for field in ["power", "presence", "donor", "programs"] {
        let mut snap = e.snapshot(&passed);
        match field {
            "power" => snap["supportGrants"][0]["power"] = json!(17),
            "presence" => snap["supportGrants"][0]["presence"] = json!(17),
            "donor" => snap["supportGrants"][0]["donor"] = json!(format!("c{}", a - 1)),
            _ => snap["supportGrants"][0]["programs"] = json!([{}]),
        };
        assert!(e.load_state(&snap).is_err(), "forged {field}");
    }
    let p = copy.support.context.ability_program - 1;
    let properties = e.program(p).instructions.iter().find_map(|instruction| match instruction {
        crate::ability_ir::Instruction::Apply(Effect::InitiateAttack { properties, .. }) => Some(properties),
        _ => None,
    }).unwrap();
    let mut full = passed;
    full.support.len = 32;
    full.support.grants.fill(passed.support.grants[0]);
    full.source = passed.support.grants[0].donor;
    full.ability_program = p + 1;
    full.ability_target = a;
    let before = full;
    assert!(e
        .begin_native_attack(&mut full, properties, false)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(full, before);
    assert!(!e.by_code.contains_key("SEC_172"));
    assert!(e.rejected_cards().unwrap()["SEC_172"].contains("plot"));
}
#[test]
fn native_attack_inline_layout_budget_preserves_card_size() {
    assert_eq!(std::mem::size_of::<Card>(), 40);
    eprintln!(
        "native attack State={} SupportState={} SupportGrant={} ResolutionContext={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<SupportState>(),
        std::mem::size_of::<SupportGrant>(),
        std::mem::size_of::<ResolutionContext>()
    );
    assert_eq!(std::mem::size_of::<State>(), EXPECTED_INLINE_STATE_BYTES);
}

#[test]
fn native_attack_attacker_death_and_illegal_attacker_domains_keep_parent_lifecycle() {
    let e = engine();
    let mut s = start(&e, Seat::P1);
    let a = attacker(&e, &mut s, Seat::P1, "LOF_254");
    let exhausted = attacker(&e, &mut s, Seat::P1, "LOF_093");
    s.card_mut(exhausted).exhausted = true;
    let restricted = attacker(&e, &mut s, Seat::P1, "LOF_044");
    let enemy = attacker(&e, &mut s, Seat::P2, "HMW_089");
    let (select, event) = play(&e, &s, "IBH_021");
    assert_eq!(
        e.legal_actions(&select)
            .iter()
            .filter_map(Action::id)
            .collect::<Vec<_>>(),
        vec![a]
    );
    for id in [exhausted, restricted, enemy, select.player(Seat::P1).base] {
        assert!(e.apply(&select, &e.card_action(&select, id)).is_err());
    }
    let choose = click(&e, &select, a);
    let done = finish_triggers(&e, click(&e, &choose, enemy));
    assert!(done.player(Seat::P1).discard.contains(a));
    assert_eq!(done.card(enemy).damage, 3);
    assert!(done.player(Seat::P1).discard.contains(event));
    assert_eq!(done.support.len, 0);
    assert!(!done.support.pending);
    assert!(!done.support.native_attack);
    assert_eq!(done.active, Seat::P2);
    roundtrip(&e, &done);
}

#[test]
fn native_attack_early_keyword_window_settles_stat_deaths_before_defender_prompt() {
    // Synthetic dependency diagnostic: a live HP registration whose predicate
    // reads Grit. This is separate from the 14 actual-card behavioral fixtures.
    let mut catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let donor = catalog["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|x| x["setCode"] == "LOF_093")
        .unwrap();
    donor["abilities"]["card"] = json!([{"trigger":"constant","title":"Synthetic HP until Grit","condition":{"kind":"unary","op":"!","value":{"kind":"query","name":"hasSomeKeyword","receiver":{"kind":"ref","path":["source"]},"args":[{"kind":"const","value":"grit"}]}},"effect":{"kind":"modifyStats","target":"self","duration":"persistent","power":{"kind":"const","value":0},"hp":{"kind":"const","value":2}}}]);
    let e = Engine::from_catalogue_value(&catalog).unwrap();
    let mut s = start(&e, Seat::P1);
    let a = attacker(&e, &mut s, Seat::P1, "LOF_093");
    s.card_mut(a).damage = 5;
    assert_eq!(e.hp(&s, a), 7);
    let (select, event) = play(&e, &s, "TWI_172");
    let done = click(&e, &select, a);
    assert_eq!(done.decision, Decision::Window);
    assert!(done.player(Seat::P1).discard.contains(a));
    assert!(done.player(Seat::P1).discard.contains(event));
    assert_eq!(done.support.len, 0);
    assert!(!done.support.pending);
    assert_eq!(done.active, Seat::P2);
}
