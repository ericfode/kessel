use super::*;
use crate::ability_ir::chosen_hand_discard_contract as contract;
fn engine() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
        .unwrap()
}
fn entry(code: &str) -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: e.by_code[code],
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine) -> State {
    let mut s = State::default();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for seat in [Seat::P1, Seat::P2] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, "SOR_014");
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn hand(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn start(e: &Engine, s: &mut State, source: CardId, trigger: &str) {
    s.source = source;
    s.active = s.card(source).owner;
    let p = e
        .card_programs(s, source)
        .iter()
        .copied()
        .find(|p| e.program(*p).trigger == trigger)
        .unwrap();
    e.begin_program(s, p, ProgramContinuation::Triggers)
        .unwrap();
}
fn click(e: &Engine, s: &mut State, id: CardId) {
    *s = e.apply_ability_action(s, &e.card_action(s, id)).unwrap();
}
fn done(e: &Engine, s: &mut State) {
    let action = e
        .ability_actions(s)
        .into_iter()
        .find(|a| a.arg == Some(json!("done")))
        .expect("Done or Choose nothing must be legal");
    *s = e.apply_ability_action(s, &action).unwrap();
}
#[test]
fn chosen_hand_closed_complete_profiles_and_raw_mutations() {
    for code in ["ASH_226", "HMW_041"] {
        let original = entry(code);
        assert!(contract::entry(&original));
        let admitted = |v: Value| {
            let c = crate::ability_ir::Catalogue::load(
                &json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]}),
            )
            .unwrap();
            assert!(c.cards.contains_key(code), "{:?}", c.rejected);
        };
        admitted(original.clone());
        let index = if code == "ASH_226" { 1 } else { 0 };
        for (path, value) in [
            ("/baseClass", json!("EventCard")),
            ("/card/types", json!(["unit", "leader"])),
            ("/card/unique", json!(false)),
            ("/card/power", json!(22)),
            ("/keywordValues", json!({"overwhelm":1})),
            ("/stateWatchers", json!([{}])),
        ] {
            let mut v = original.clone();
            let keys = path.split('/').skip(1).collect::<Vec<_>>();
            let mut node = &mut v;
            for k in &keys[..keys.len() - 1] {
                node = &mut node[*k];
            }
            node[keys[keys.len() - 1]] = value;
            let c = crate::ability_ir::Catalogue::load(
                &json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]}),
            )
            .unwrap();
            assert!(!c.cards.contains_key(code), "{code} {path}");
        }
        for (key, value) in [
            ("optional", json!(false)),
            ("costs", json!([{"kind":"exhaustSelf"}])),
            ("condition", json!({"kind":"const","value":true})),
            ("requiresConfirmation", json!(true)),
            (
                "then",
                json!({"title":"extra","effect":{"kind":"noAction"}}),
            ),
        ] {
            let mut v = original.clone();
            v["abilities"]["card"][index][key] = value;
            let c = crate::ability_ir::Catalogue::load(
                &json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]}),
            )
            .unwrap();
            assert!(!c.cards.contains_key(code), "{code} {key}");
        }
        for (key, value) in [
            ("owner", json!("opponent")),
            ("type", json!("unit")),
            ("zone", json!("discard")),
            ("optional", json!(true)),
            ("promptTitle", json!("wrong")),
            ("count", json!({"kind":"const","value":1})),
        ] {
            let mut v = original.clone();
            v["abilities"]["card"][index]["targetResolver"][key] = value;
            let c = crate::ability_ir::Catalogue::load(
                &json!({"schema":crate::ability_ir::SCHEMA,"cards":[v]}),
            )
            .unwrap();
            assert!(!c.cards.contains_key(code), "{code} resolver {key}");
        }
    }
    for raw in [
        json!({"kind":"discardSpecificCard","target":"chosen","amount":{"kind":"const","value":2}}),
        json!({"kind":"attackModifiers","effects":[],"duration":"phase"}),
    ] {
        assert!(serde_json::from_value::<Effect>(raw).is_err());
    }
}
#[test]
fn chosen_hand_qira_single_protocol_discard_and_live_power() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, seat, "ASH_226");
        let first = hand(&e, &mut s, seat, "JTL_075");
        let second = hand(&e, &mut s, seat, "SOR_095");
        let target = unit(&e, &mut s, seat.other(), "SOR_095");
        assert_eq!(e.power(&s, source, false), 7);
        start(&e, &mut s, source, "whenPlayed");
        assert_eq!(s.decision, Decision::AbilityTarget);
        let actions = e.ability_actions(&s);
        assert!(actions.iter().any(|a| a.arg == Some(json!("passAbility"))));
        assert!(!actions.iter().any(|a| matches!(
            a.arg.as_ref().and_then(Value::as_str),
            Some("done" | "noTarget")
        )));
        assert!(e
            .apply_ability_action(&s, &Action::button("Choose nothing", "done"))
            .is_err());
        click(&e, &mut s, first);
        assert_eq!(s.decision, Decision::AbilityTarget);
        assert_eq!(s.player(seat).hand.as_slice(), [second]);
        assert_eq!(s.player(seat).discard.as_slice(), [first]);
        assert_eq!(e.power(&s, source, false), 8);
        assert_eq!(s.ability_selection.len, 0);
        assert!(e.ability_actions(&s).iter().any(|a| a.id() == Some(source)));
        click(&e, &mut s, target);
        assert!(!e.in_arena(&s, target));
        assert_eq!(e.power(&s, source, false), 8);
        let drawn = hand(&e, &mut s, seat, "JTL_075");
        assert_eq!(e.power(&s, source, false), 7);
        s.player_mut(seat).hand.remove(drawn);
        for _ in 0..11 {
            hand(&e, &mut s, seat, "JTL_075");
        }
        assert_eq!(e.power(&s, source, false), 0);
    }
}
#[test]
fn chosen_hand_keeper_exact_cardinality_toggles_and_commit_order() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, seat, "HMW_041");
        let a = hand(&e, &mut s, seat, "HMW_041");
        let b = hand(&e, &mut s, seat, "HMW_041");
        let c = hand(&e, &mut s, seat, "HMW_041");
        let wrong = hand(&e, &mut s, seat, "JTL_075");
        s.attack_source = source;
        s.attack_defender = s.player(seat.other()).base;
        start(&e, &mut s, source, "onAttack");
        assert_eq!(s.decision, Decision::AbilityTarget);
        assert!(e
            .ability_actions(&s)
            .iter()
            .any(|a| a.label == "Choose nothing" && a.arg == Some(json!("done"))));
        assert!(!e.ability_actions(&s).iter().any(|x| x.id() == Some(wrong)));
        assert!(e
            .apply_ability_action(&s, &e.card_action(&s, wrong))
            .is_err());
        click(&e, &mut s, a);
        assert!(!e
            .ability_actions(&s)
            .iter()
            .any(|a| a.arg == Some(json!("done"))));
        assert!(e
            .apply_ability_action(&s, &Action::button("Done", "done"))
            .is_err());
        click(&e, &mut s, b);
        assert_eq!(s.decision, Decision::AbilityTarget);
        assert!(!e.ability_actions(&s).iter().any(|x| x.id() == Some(c)));
        assert!(e.apply_ability_action(&s, &e.card_action(&s, c)).is_err());
        let copy = s;
        assert_eq!(copy, s);
        assert_eq!(copy.zobrist(), s.zobrist());
        click(&e, &mut s, a);
        click(&e, &mut s, c);
        done(&e, &mut s);
        assert_eq!(s.player(seat).hand.as_slice(), [a, wrong]);
        assert_eq!(s.player(seat).discard.as_slice(), [b, c]);
        assert_eq!(e.power(&s, source, true), 20);
        assert!(e.has_keyword(&s, source, "overwhelm"));
        assert_eq!(s.support.len, 1);
        assert_eq!(s.support.grants[0].power, 15);
    }
}
#[test]
fn chosen_hand_last_event_success_and_empty_reset_share_scalar_owner() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "ASH_226");
    s.source = source;
    let valid = hand(&e, &mut s, Seat::P1, "JTL_075");
    let stale = add(&e, &mut s, Seat::P1, "JTL_075");
    s.players[0].discard.push(stale).unwrap();
    let effect = Effect::DiscardSpecificCard {
        target: Target::name("chosen"),
    };
    s.event_result.damage_dealt = 17;
    let (n, success) = e
        .apply_scalar_effect(
            &mut s,
            &effect,
            vec![Eval::Card(valid), Eval::Card(stale)],
            1,
            None,
        )
        .unwrap();
    assert_eq!(n, 1);
    assert!(
        !success,
        "last captured event failed even though an earlier event moved a card"
    );
    assert_eq!(s.event_result.damage_dealt, 0);
    let valid2 = hand(&e, &mut s, Seat::P1, "JTL_075");
    let (n, success) = e
        .apply_scalar_effect(
            &mut s,
            &effect,
            vec![Eval::Card(stale), Eval::Card(valid2)],
            1,
            None,
        )
        .unwrap();
    assert_eq!((n, success), (1, true));
    s.ability_result = 9;
    s.ability_succeeded = true;
    let (n, success) = e
        .apply_scalar_effect(&mut s, &effect, vec![], 1, None)
        .unwrap();
    assert_eq!((n, success), (0, false));
    assert_eq!(s.players[0].discard.as_slice(), [stale, valid, valid2]);
}
#[test]
fn chosen_hand_keeper_insufficient_and_decline_grant_nothing() {
    let e = engine();
    for count in [0, 1, 2] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, Seat::P1, "HMW_041");
        for _ in 0..count {
            hand(&e, &mut s, Seat::P1, "HMW_041");
        }
        s.attack_source = source;
        s.attack_defender = s.players[1].base;
        start(&e, &mut s, source, "onAttack");
        if count == 2 {
            done(&e, &mut s);
        } else {
            assert_ne!(s.decision, Decision::AbilityTarget);
        }
        assert_eq!(s.players[0].hand.len, count);
        assert_eq!(s.support.len, 0);
        assert!(!e.has_keyword(&s, source, "overwhelm"));
        assert_eq!(e.power(&s, source, false), 5);
    }
}
#[test]
fn chosen_hand_actual_attack_restore_overwhelm_shield_and_expiry() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let e = engine();
            for seat in [Seat::P1, Seat::P2] {
                for shield in [false, true] {
                    for custom_first in [false, true] {
                        let mut s = board(&e);
                        s.active = seat;
                        s.action_player = Some(seat);
                        let source = unit(&e, &mut s, seat, "HMW_041");
                        let a = hand(&e, &mut s, seat, "HMW_041");
                        let b = hand(&e, &mut s, seat, "HMW_041");
                        let defender = unit(&e, &mut s, seat.other(), "SOR_095");
                        let base = s.player(seat).base;
                        s.card_mut(base).damage = 5;
                        if shield {
                            s.source = source;
                            s.ability_target = defender;
                            e.apply_effect(
                                &mut s,
                                &Effect::GiveShield {
                                    target: Target::name("chosen"),
                                    amount: Expr::number(1),
                                },
                            )
                            .unwrap();
                            s.source = 0;
                            s.ability_target = 0;
                        }
                        s = e.apply(&s, &e.card_action(&s, source)).unwrap();
                        s = e.apply(&s, &e.card_action(&s, defender)).unwrap();
                        let actions = e.legal_actions(&s);
                        let order = actions
                            .iter()
                            .find(|a| {
                                if custom_first {
                                    a.label.starts_with("Discard 2 cards")
                                } else {
                                    a.label == "Restore 2"
                                }
                            })
                            .unwrap_or_else(|| panic!("trigger order actions {:?}", actions));
                        s = e.apply(&s, order).unwrap();
                        let mut clicks = 0;
                        for _ in 0..20 {
                            if s.decision == Decision::Window {
                                break;
                            }
                            let actions = e.legal_actions(&s);
                            let action = if s.decision == Decision::AbilityTarget {
                                if clicks == 0 {
                                    clicks += 1;
                                    e.card_action(&s, a)
                                } else if clicks == 1 {
                                    clicks += 1;
                                    e.card_action(&s, b)
                                } else {
                                    actions
                                        .iter()
                                        .find(|a| a.arg == Some(json!("done")))
                                        .unwrap()
                                        .clone()
                                }
                            } else {
                                actions
                                    .iter()
                                    .find(|a| {
                                        a.label.starts_with("Discard 2 cards")
                                            || a.label == "Restore 2"
                                            || a.label == "Trigger"
                                    })
                                    .unwrap_or_else(|| {
                                        panic!(
                                            "unexpected attack continuation {:?} {:?}",
                                            s.decision, actions
                                        )
                                    })
                                    .clone()
                            };
                            s = e.apply(&s, &action).unwrap();
                        }
                        assert_eq!(s.decision, Decision::Window);
                        assert_eq!(s.card(base).damage, 3);
                        assert_eq!(s.player(seat).hand.len, 0);
                        assert_eq!(s.support.len, 0);
                        assert_eq!(e.power(&s, source, false), 5);
                        assert!(!e.has_keyword(&s, source, "overwhelm"));
                        assert_eq!(
                            s.card(s.player(seat.other()).base).damage,
                            if shield { 0 } else { 17 }
                        );
                        assert_eq!(e.in_arena(&s, defender), shield);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
#[test]
fn chosen_hand_attack_grant_copy_hash_stacking_and_retained_boundary() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "HMW_041");
    s.source = source;
    s.attack_source = source;
    s.attack_defender = s.players[1].base;
    let p = e.card_programs(&s, source)[0];
    let pc = e
        .program(p)
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::Apply(Effect::AttackModifiers { .. })))
        .unwrap();
    s.ability_program = p + 1;
    s.ability_pc = pc as u8;
    let Instruction::Apply(effect) = &e.program(p).instructions[pc] else {
        unreachable!()
    };
    let baseline = s;
    let oldhash = s.zobrist();
    e.apply_on_attack_modifiers(&mut s, effect).unwrap();
    e.apply_on_attack_modifiers(&mut s, effect).unwrap();
    assert_eq!(e.power(&s, source, true), 35);
    let copy = s;
    assert_eq!(copy, s);
    assert_eq!(copy.zobrist(), s.zobrist());
    assert_ne!(oldhash, s.zobrist());
    let mut changed = s;
    changed.support.grants[0].power = 14;
    assert_ne!(changed.zobrist(), s.zobrist());
    changed = s;
    changed.support.grants[0].conditional_pc += 1;
    assert_ne!(changed.zobrist(), s.zobrist());
    let mut old_neutral = baseline;
    old_neutral.support.grants[0].power = 15;
    assert_eq!(
        old_neutral.zobrist(),
        oldhash,
        "unused inline grant storage stays old-state hash neutral"
    );
    s.outcome = 1;
    s.decision = Decision::Terminal;
    let snap = e.snapshot(&s);
    assert_eq!(snap["onAttackModifiersPending"], true);
    assert!(
        e.load_state(&snap)
            .unwrap_err()
            .contains("onAttackModifiersPending"),
        "retained native diagnostic does not invent supported engine-wire provenance"
    );
    e.clear_support_grants(&mut s);
    assert!(e.snapshot(&s).get("onAttackModifiersPending").is_none());
}
#[test]
fn chosen_hand_keeper_title_predicate_allows_other_identity_and_copied_program_recipient() {
    let mut e = engine();
    let mut s = board(&e);
    let donor = unit(&e, &mut s, Seat::P1, "HMW_041");
    let recipient = unit(&e, &mut s, Seat::P1, "SOR_095");
    // Synthetic alternate-title identity proves the source predicate reads the
    // printed title, rather than recognizing a focus set code/internal name.
    let alternate = e.by_code["JTL_075"] as usize;
    e.definitions[alternate].info.title = "Keeper of Skara Nal".into();
    e.definitions[alternate].info.internal_name = "keeper-of-skara-nal#another-subtitle".into();
    let a = hand(&e, &mut s, Seat::P1, "HMW_041");
    let b = hand(&e, &mut s, Seat::P1, "JTL_075");
    s.attack_source = donor;
    s.attack_defender = s.players[1].base;
    start(&e, &mut s, donor, "onAttack");
    assert!(e.ability_actions(&s).iter().any(|x| x.id() == Some(b)));
    click(&e, &mut s, a);
    click(&e, &mut s, b);
    done(&e, &mut s);
    assert_eq!(s.players[0].discard.as_slice(), [a, b]);
    e.clear_support_grants(&mut s);
    s.attack_source = 0;
    s.attack_defender = 0;
    s.source = donor;
    let support:crate::ability_ir::Ability=serde_json::from_value(json!({"trigger":"action","title":"Synthetic Support grant diagnostic","costs":[],"effect":{"kind":"gainKeyword","target":"self","duration":"phase","keyword":"support","amount":{"kind":"const","value":1}}})).unwrap();
    assert!(Program::compile(&support).is_ok());
    e.apply_effect(&mut s, &support.effect).unwrap();
    assert!(e.has_keyword(&s, donor, "support"));
    e.begin_support_attack(&mut s, recipient).unwrap();
    let p = e.card_programs(&s, donor)[0];
    assert!(e.support_programs(&s, recipient).contains(&p));
    let pc = e
        .program(p)
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::Apply(Effect::AttackModifiers { .. })))
        .unwrap();
    s.source = recipient;
    s.attack_source = recipient;
    s.attack_defender = s.players[1].base;
    s.ability_program = p + 1;
    s.ability_pc = pc as u8;
    let Instruction::Apply(effect) = &e.program(p).instructions[pc] else {
        unreachable!()
    };
    e.apply_on_attack_modifiers(&mut s, effect).unwrap();
    assert_eq!(e.power(&s, recipient, true), 18);
    assert_eq!(e.power(&s, donor, false), 5);
    assert!(e.has_keyword(&s, recipient, "overwhelm"));
    assert!(!e.has_keyword(&s, donor, "overwhelm"));
    assert_eq!(s.support.grants[1].donor, recipient);
    assert_eq!(s.support.grants[1].conditional_program, p + 1);
    e.defeat(&mut s, donor).unwrap();
    assert!(e.has_keyword(&s, recipient, "overwhelm"));
    assert!(e.support_programs(&s, recipient).contains(&p));
}
#[test]
fn chosen_hand_exact_selector_and_attack_envelope_standalone_guards_reject_mutants() {
    let keeper = entry("HMW_041");
    let original = keeper["abilities"]["card"][0].clone();
    for (pointer, value) in [
        ("/targetResolver/count/value", json!(3)),
        ("/targetResolver/mode", json!("upTo")),
        ("/targetResolver/predicate/right/value", json!("Other")),
        ("/ifYouDo/effect/effects/0/power/value", json!(14)),
        ("/ifYouDo/effect/effects/0/duration", json!("phase")),
        ("/ifYouDo/effect/effects/1/keyword", json!("raid")),
        ("/ifYouDo/effect/effects/1/amount/value", json!(2)),
    ] {
        let mut value_root = original.clone();
        *value_root.pointer_mut(pointer).unwrap() = value;
        let a: crate::ability_ir::Ability = serde_json::from_value(value_root).unwrap();
        assert!(!contract::registration(&a), "{pointer}");
        assert!(Program::compile(&a).is_err(), "{pointer}");
    }
    let a: crate::ability_ir::Ability = serde_json::from_value(original).unwrap();
    let mut effect = a.if_you_do.unwrap().effect;
    assert!(contract::attack_modifiers(&effect));
    if let Effect::AttackModifiers { effects } = &mut effect {
        effects.reverse();
    }
    assert!(!contract::attack_modifiers(&effect));
}
#[test]
fn chosen_hand_explicit_aot_fallback_matches_interpreter_and_never_skips_discard() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.rules_backend = backend;
        let mut s = board(&e);
        let source = unit(&e, &mut s, Seat::P1, "ASH_226");
        let card = hand(&e, &mut s, Seat::P1, "JTL_075");
        let p = e
            .card_programs(&s, source)
            .iter()
            .copied()
            .find(|p| e.program(*p).trigger == "whenPlayed")
            .unwrap();
        assert!(e.program(p).executable.is_err());
        start(&e, &mut s, source, "whenPlayed");
        click(&e, &mut s, card);
        assert_eq!(s.players[0].discard.as_slice(), [card]);
        assert_eq!(s.decision, Decision::AbilityTarget);
        assert_eq!(s.ability_result, 1);
        assert!(s.ability_succeeded);
    }
    let mut e = engine();
    e.rules_backend = RulesBackend::Aot;
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "ASH_226");
    hand(&e, &mut s, Seat::P1, "JTL_075");
    s.source = source;
    let p = e
        .card_programs(&s, source)
        .iter()
        .copied()
        .find(|p| e.program(*p).trigger == "whenPlayed")
        .unwrap();
    assert!(e
        .begin_program(&mut s, p, ProgramContinuation::Triggers)
        .unwrap_err()
        .contains("AOT backend unavailable"));
}
#[test]
fn chosen_hand_qira_pass_empty_and_complete_secondary_target_domain() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        let source = unit(&e, &mut s, seat, "ASH_226");
        start(&e, &mut s, source, "whenPlayed");
        assert_ne!(s.decision, Decision::AbilityTarget);
        assert_eq!(e.power(&s, source, false), 9);
        let card = hand(&e, &mut s, seat, "JTL_075");
        start(&e, &mut s, source, "whenPlayed");
        s = e
            .apply_ability_action(&s, &Action::button("Pass", "passAbility"))
            .unwrap();
        assert_eq!(s.player(seat).hand.as_slice(), [card]);
        assert_eq!(s.card(source).damage, 0);
        let own = unit(&e, &mut s, seat, "SOR_095");
        let enemy = unit(&e, &mut s, seat.other(), "SOR_095");
        let own_space = unit(&e, &mut s, seat, "SOR_112");
        let enemy_space = unit(&e, &mut s, seat.other(), "SOR_112");
        start(&e, &mut s, source, "whenPlayed");
        click(&e, &mut s, card);
        let mut actual = e
            .ability_actions(&s)
            .iter()
            .filter_map(Action::id)
            .collect::<Vec<_>>();
        actual.sort();
        let mut expected = vec![source, own, enemy, own_space, enemy_space];
        expected.sort();
        assert_eq!(actual, expected);
        click(&e, &mut s, source);
        assert_eq!(s.card(source).damage, 3);
        assert_eq!(e.power(&s, source, false), 9);
    }
}
#[test]
fn chosen_hand_existing_view_discard_namespace_is_unchanged() {
    for code in [
        "ASH_220", "JTL_207", "LOF_226", "SEC_230", "SHD_184", "SOR_200", "SOR_201", "TS26_080",
        "TWI_223",
    ] {
        let original = entry(code);
        assert!(!contract::marker(&original["abilities"]));
        let c = crate::ability_ir::Catalogue::load(
            &json!({"schema":crate::ability_ir::SCHEMA,"cards":[original]}),
        )
        .unwrap();
        assert!(c.cards.contains_key(code), "{code}: {:?}", c.rejected);
    }
}
