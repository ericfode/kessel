use super::*;
use crate::attack_lifetime_contract as contract;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_172")
        .unwrap()
        .clone()
}
fn engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    assert!(
        e.by_code.contains_key("ASH_172"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_172")
    );
    e.set_rules_backend(backend).unwrap();
    e
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
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.active = seat;
    s.action_player = Some(seat);
    s.hidden_history_exact = true;
    s.credit_state_exact = true;
    s.force_state_exact = true;
    s.round_limit_wire_present = true;
    for p in [seat, seat.other()] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
        for _ in 0..12 {
            let id = add(e, &mut s, p, "LOF_254");
            s.player_mut(p).resources.push(id).unwrap();
        }
        for _ in 0..5 {
            let id = add(e, &mut s, p, "LOF_254");
            s.player_mut(p).deck.push(id).unwrap();
        }
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{:?}: {:?}",
        s.decision,
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn arg(e: &Engine, s: &State, arg: &str) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.arg == Some(json!(arg)))
        .unwrap_or_else(|| panic!("{arg}: {:?} {:?}", s.decision, e.legal_actions(s)));
    e.apply(s, &a).unwrap()
}
#[test]
fn attack_lifetime_closed_raw_and_typed_envelopes() {
    let original = entry();
    assert_eq!(contract::prepare(&original).unwrap(), true);
    assert!(Program::compile(
        &serde_json::from_value::<crate::ability_ir::Ability>(
            original["abilities"]["card"][0].clone()
        )
        .unwrap()
    )
    .is_err());
    for field in ["rawUnknown", "aliasOf", "pilotText"] {
        let mut raw = original.clone();
        raw[field] = json!(true);
        assert!(contract::prepare(&raw).is_err());
    }
    for (path, value) in [
        (vec!["card", "types"], json!(["leader"])),
        (vec!["baseClass"], json!("LeaderUnitCard")),
        (vec!["keywordValues"], json!({})),
        (
            vec!["abilities", "leader"],
            json!([original["abilities"]["card"][0].clone()]),
        ),
        (vec!["diagnostics"], json!(["unknown"])),
    ] {
        let mut raw = original.clone();
        let mut v = &mut raw;
        for p in &path[..path.len() - 1] {
            v = &mut v[*p];
        }
        v[path[path.len() - 1]] = value;
        assert!(contract::prepare(&raw).is_err());
    }
    let mut raw = original.clone();
    raw["setCode"] = json!("ZZZ_999");
    assert!(contract::prepare(&raw).unwrap());
    let p = contract::compiled().unwrap();
    assert!(contract::program(&p));
    let mut bad = p.clone();
    bad.event_end -= 1;
    assert!(!contract::program(&bad));
    let mut nested = json!({"unfamiliar":original});
    assert!(contract::prepare(&nested).is_err());
    for _ in 0..129 {
        nested = json!([nested]);
    }
    assert!(contract::prepare(&nested).unwrap_err().contains("budget"));
}
#[test]
fn attack_lifetime_native_discard_decline_empty_and_single_hand_both_seats() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            for mode in 0..4 {
                let mut s = board(&e, seat);
                let source = unit(&e, &mut s, seat, "ASH_172");
                let count = if mode == 2 {
                    0
                } else if mode == 3 {
                    1
                } else {
                    2
                };
                let mut cards = vec![];
                for _ in 0..count {
                    cards.push(hand(&e, &mut s, seat, "SOR_095"));
                }
                let base = s.player(seat.other()).base;
                s = choose(&e, &s, source);
                s = choose(&e, &s, base);
                if mode == 2 {
                    assert!(s.is_action_window());
                    assert_eq!(s.card(base).damage, 3);
                } else {
                    assert_eq!(s.decision, Decision::AbilityOptional);
                    assert_eq!(
                        e.legal_actions(&s),
                        vec![
                            Action::button("Trigger", "trigger"),
                            Action::button("Pass", "pass")
                        ]
                    );
                    if mode == 1 {
                        s = arg(&e, &s, "pass");
                        assert_eq!(s.card(base).damage, 3);
                        assert_eq!(s.player(seat).hand.len, count);
                    } else {
                        s = arg(&e, &s, "trigger");
                        assert_eq!(s.decision, Decision::AbilityDiscard);
                        let actions = e.legal_actions(&s);
                        assert_eq!(actions.len(), count as usize);
                        assert!(actions.iter().all(|a| a.kind == "card"));
                        s = choose(&e, &s, cards[0]);
                        assert!(s.is_action_window());
                        assert_eq!(s.card(base).damage, 5);
                        assert!(s.player(seat).discard.contains(cards[0]));
                    }
                }
                assert_eq!(s.support.len, 0);
                assert_eq!(e.power(&s, source, false), 3);
                assert_eq!(s.card(source).phase_power, 0);
            }
        }
    }
}
#[test]
fn attack_lifetime_native_saboteur_and_next_attack_cleanup() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let source = unit(&e, &mut s, seat, "ASH_172");
            let sentinel = unit(&e, &mut s, seat.other(), "SHD_237");
            let defender = unit(&e, &mut s, seat.other(), "ASH_201");
            let card = hand(&e, &mut s, seat, "SOR_095");
            assert!(e.has_keyword(&s, source, "saboteur"));
            assert!(e.has_keyword(&s, sentinel, "sentinel"));
            assert!(e
                .attack_targets(&s, source, false)
                .contains(&s.player(seat.other()).base));
            s.source = defender;
            e.apply_effect(&mut s,&serde_json::from_value(json!({"kind":"giveShield","target":"self","amount":{"kind":"const","value":1}})).unwrap()).unwrap();
            s.source = 0;
            s = choose(&e, &s, source);
            s = choose(&e, &s, defender);
            assert_eq!(s.decision, Decision::TriggerOrder);
            let order = e.legal_actions(&s);
            assert_eq!(order.len(), 2);
            let saboteur = order
                .into_iter()
                .find(|a| a.label == "Saboteur: defeat all shields")
                .unwrap();
            s = e.apply(&s, &saboteur).unwrap();
            assert_eq!(s.card(defender).shield, 0);
            assert_eq!(s.decision, Decision::AbilityOptional);
            s = arg(&e, &s, "trigger");
            s = choose(&e, &s, card);
            assert!(!e.in_arena(&s, defender));
            assert_eq!(s.card(defender).shield, 0);
            assert_eq!(s.card(source).damage, 2);
            assert_eq!(s.support.len, 0);
            assert_eq!(e.power(&s, source, false), 3);
            // Synthetic readiness isolates the next declared attack; actual source games
            // separately establish natural rounds and real repeated attacks.
            s.active = seat;
            s.action_player = Some(seat);
            s.card_mut(source).exhausted = false;
            let base = s.player(seat.other()).base;
            s = choose(&e, &s, source);
            s = choose(&e, &s, base);
            assert!(s.is_action_window());
            assert_eq!(s.card(base).damage, 3);
            assert_eq!(s.support.len, 0);
        }
    }
}
fn modifier_context(e: &Engine, s: &mut State, source: CardId) -> Effect {
    let p = e
        .by_code
        .get("ASH_172")
        .map(|d| e.definitions[*d as usize].programs.card[0])
        .unwrap();
    let (pc, effect) = e
        .program(p)
        .instructions
        .iter()
        .enumerate()
        .find_map(|(pc, i)| match i {
            Instruction::Apply(e) if contract::modifier(e) => Some((pc, e.clone())),
            _ => None,
        })
        .unwrap();
    s.source = source;
    s.target = s.player(s.card(source).owner.other()).base;
    s.attack_source = source;
    s.attack_defender = s.target;
    s.ability_program = p + 1;
    s.ability_pc = pc as u8;
    effect
}
#[test]
fn attack_lifetime_native_retained_copy_hash_pending_import_and_physical_departure() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let donor = unit(&e, &mut s, seat, "ASH_172");
            let recipient = unit(&e, &mut s, seat, "ASH_201");
            let effect = modifier_context(&e, &mut s, recipient);
            // A captured copied program resolves with the recipient as its source. There
            // is no printed Support on Razor Crest; this is a native lifecycle diagnostic.
            let copied = s.ability_program - 1;
            s.support.grants[0] = SupportGrant {
                target: recipient,
                donor,
                programs: {
                    let mut a = [0; MAX_SUPPORT_PROGRAMS];
                    a[0] = copied;
                    a
                },
                len: 1,
                ..SupportGrant::default()
            };
            s.support.len = 1;
            e.leave_play(&mut s, donor).unwrap();
            assert_eq!(e.support_programs(&s, recipient), vec![copied]);
            let before = s.zobrist();
            assert_eq!(e.apply_effect(&mut s, &effect).unwrap(), (1, true));
            assert_eq!(e.power(&s, recipient, true), 4);
            assert_eq!(e.power(&s, donor, false), 3);
            assert_ne!(s.zobrist(), before);
            let copy = s;
            assert_eq!(copy, s);
            assert_eq!(copy.zobrist(), s.zobrist());
            let mut changed = s;
            changed.support.grants[1].power = 3;
            assert_ne!(changed.zobrist(), s.zobrist());
            let outer = ResolutionContext::capture(&s);
            s.source = donor;
            s.ability_program = 0;
            outer.restore(&mut s);
            assert_eq!(e.power(&s, recipient, true), 4);
            assert_eq!(s.support.len, 2);
            let wire = e.snapshot(&s);
            assert!(wire["supportGrants"]
                .as_array()
                .unwrap()
                .iter()
                .any(|g| g["kind"] == "onAttackModifiersPending"));
            assert!(e.load_state(&wire).is_err());
            // Capturing attack-end registrations must not expire the grant: the actual
            // engine removes it only after that event's trigger window.
            e.capture_attack_end(&mut s).unwrap();
            assert_eq!(e.power(&s, recipient, true), 4);
            assert_eq!(s.support.len, 2);
            e.leave_play(&mut s, recipient).unwrap();
            assert_eq!(s.support.len, 0);
            s.player_mut(seat).discard.remove(recipient);
            e.add_arena(&mut s, recipient).unwrap();
            assert!(e.attack_participant_departed(&s, recipient));
            assert_eq!(e.apply_effect(&mut s, &effect).unwrap(), (0, false));
            assert_eq!(e.power(&s, recipient, false), 2);
            assert_eq!(s.support.len, 0);
        }
    }
}
#[test]
fn attack_lifetime_native_capacity_and_origin_fail_before_mutation() {
    let e = engine(RulesBackend::Interpreter);
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_172");
    let effect = modifier_context(&e, &mut s, source);
    s.support.len = MAX_SUPPORT_GRANTS as u8;
    let before = s;
    assert!(e
        .apply_effect(&mut s, &effect)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(s, before);
    s.support.len = 0;
    s.ability_program = 0;
    let before = s;
    assert!(e.apply_effect(&mut s, &effect).is_err());
    assert_eq!(s, before);
}
#[test]
fn attack_lifetime_native_paid_play_and_combat_death_clear_at_completed_boundaries() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for seat in [Seat::P1, Seat::P2] {
            let mut paid = board(&e, seat);
            let source = hand(&e, &mut paid, seat, "ASH_172");
            let ready = e.ready_resources(&paid, seat);
            paid = choose(&e, &paid, source);
            assert!(e.in_arena(&paid, source));
            assert!(paid.card(source).exhausted);
            assert_eq!(e.ready_resources(&paid, seat), ready - 4);
            assert!(e.has_keyword(&paid, source, "saboteur"));
            assert_eq!(e.power(&paid, source, false), 3);
            let mut s = board(&e, seat);
            let source = unit(&e, &mut s, seat, "ASH_172");
            let defender = unit(&e, &mut s, seat.other(), "JTL_251");
            let card = hand(&e, &mut s, seat, "SOR_095");
            let resources = e.ready_resources(&s, seat);
            s = choose(&e, &s, source);
            s = choose(&e, &s, defender);
            assert_eq!(s.decision, Decision::TriggerOrder);
            let order = e.legal_actions(&s);
            assert_eq!(order.len(), 2);
            let custom = order
                .into_iter()
                .find(|a| a.label.starts_with("Discard a card from your hand."))
                .unwrap();
            s = e.apply(&s, &custom).unwrap();
            s = arg(&e, &s, "trigger");
            s = choose(&e, &s, card);
            assert!(s.is_action_window());
            assert!(!e.in_arena(&s, source));
            assert!(s.player(seat).discard.contains(source));
            assert_eq!(s.support.len, 0);
            assert_eq!(s.card(source).phase_power, 0);
            assert_eq!(e.ready_resources(&s, seat), resources);
            assert_eq!(s.attack_source, 0);
            // This asserts the completed death boundary only. Event-order probes from the
            // source engine separately retain its pre-cleanup OnAttackEnd power.
        }
    }
}
#[test]
fn attack_lifetime_selected_pool_wire_dependency_full_hash_reload_and_pending_rejection() {
    let mut value = catalogue();
    let refs = [
        "ASH_172",
        "SOR_014",
        "SOR_023",
        "SOR_095",
        "LOF_254",
        "TOKEN_SHIELD",
        "TOKEN_ADVANTAGE",
        "TOKEN_THE_FORCE",
        "TOKEN_CREDIT",
    ];
    value["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| refs.contains(&c["setCode"].as_str().unwrap()));
    // This selected-pool test deliberately excludes unrelated producer protocols;
    // every selected definition retains its complete original source metadata.
    let e = Engine::from_catalogue_value(&value).unwrap();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_172");
    let card = hand(&e, &mut s, Seat::P1, "SOR_095");
    let base = s.player(Seat::P2).base;
    let wire = e.snapshot(&s);
    let loaded = e.load_state(&wire).unwrap();
    assert_eq!(e.snapshot(&loaded), wire);
    assert_eq!(loaded.zobrist(), s.zobrist());
    for field in ["supportStateVersion", "supportGrants"] {
        let mut missing = wire.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(e.load_state(&missing).is_err());
    }
    s = choose(&e, &s, source);
    s = choose(&e, &s, base);
    s = arg(&e, &s, "trigger");
    s = choose(&e, &s, card);
    let wire = e.snapshot(&s);
    let loaded = e.load_state(&wire).unwrap();
    assert_eq!(e.snapshot(&loaded), wire);
    assert_eq!(loaded.zobrist(), s.zobrist());
    assert_eq!(loaded.support.len, 0);
    for value in [json!(true), json!(false), Value::Null] {
        let mut pending = wire.clone();
        pending["onAttackModifiersPending"] = value;
        assert!(e
            .load_state(&pending)
            .unwrap_err()
            .contains("onAttackModifiersPending"));
    }
}

#[test]
fn attack_lifetime_program_id_capacity_reserves_stored_plus_one() {
    assert_eq!(contract::checked_program_id(0).unwrap(), 0);
    assert_eq!(contract::checked_program_id(65534).unwrap(), 65534);
    for count in [65535, 65536, usize::MAX] {
        assert!(contract::checked_program_id(count).is_err());
    }
}
#[test]
fn attack_lifetime_synthetic_copied_raddus_retains_predeparture_lki_power_five() {
    // Native diagnostic only: Razor Crest has no intrinsic Support, and this
    // copied Raddus registration is not an actual printed-source oracle construction.
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine(backend);
        let donor_def = e.by_code["JTL_104"] as usize;
        let p = e.definitions[donor_def]
            .programs
            .card
            .iter()
            .copied()
            .find(|p| e.program(*p).trigger == "whenDefeated")
            .unwrap();
        // Configure the test-only copied registration before creating any State.
        // No admitted source or Support wire exposes this synthetic definition.
        let def = e.by_code["ASH_172"] as usize;
        e.definitions[def].programs.card.push(p);
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            let source = unit(&e, &mut s, seat, "ASH_172");
            let target = unit(&e, &mut s, seat.other(), "ASH_131");
            let effect = modifier_context(&e, &mut s, source);
            e.apply_effect(&mut s, &effect).unwrap();
            assert_eq!(e.power(&s, source, true), 5);
            s.ability_program = 0;
            s.ability_pc = 0;
            e.defeat(&mut s, source).unwrap();
            assert!(!e.in_arena(&s, source));
            assert_eq!(s.support.len, 0);
            let captured = s.death_pending.entries[..s.death_pending.len as usize]
                .iter()
                .find(|t| t.source == source && t.program == p)
                .unwrap();
            assert_eq!(captured.last_known_power, 5);
            assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
            assert_eq!(s.decision, Decision::AbilityTarget);
            s = choose(&e, &s, target);
            assert_eq!(s.card(target).damage, 5);
        }
    }
}
#[test]
fn attack_lifetime_copied_support_when_defeated_remains_outside_public_import() {
    let e = engine(RulesBackend::Interpreter);
    let mut s = board(&e, Seat::P1);
    let target = unit(&e, &mut s, Seat::P1, "ASH_172");
    let donor = unit(&e, &mut s, Seat::P1, "JTL_104");
    let p = e
        .card_programs(&s, donor)
        .iter()
        .copied()
        .find(|p| e.program(*p).trigger == "whenDefeated")
        .unwrap();
    let row = json!({"target":format!("c{}",target-1),"donor":format!("c{}",donor-1),"programs":[serde_json::to_value(e.program(p)).unwrap()],"keywords":vec![0u16;9],"presence":0});
    let wire = json!({"supportStateVersion":1,"supportGrants":[row]});
    let before = s;
    assert!(e
        .load_support_state(&mut s, &wire)
        .unwrap_err()
        .contains("intrinsic donor"));
    assert_eq!(s, before);
    // Even with a genuine admitted intrinsic Support donor, this registration
    // kind remains outside the public copied-ability import lifecycle.
    let intrinsic = unit(&e, &mut s, Seat::P1, "ASH_037");
    let mut wire = wire;
    wire["supportGrants"][0]["donor"] = json!(format!("c{}", intrinsic - 1));
    let before = s;
    assert!(e
        .load_support_state(&mut s, &wire)
        .unwrap_err()
        .contains("Support copied program exceeds the admitted donor lifecycle"));
    assert_eq!(s, before);
}
#[test]
fn attack_lifetime_preserves_existing_modified_attack_profiles_and_foreign_raw_boundary() {
    let e = engine(RulesBackend::Auto);
    for code in [
        "HMW_266", "IBH_021", "IBH_023", "IBH_064", "JTL_146", "JTL_231", "LAW_157", "LAW_202",
        "LOF_111", "SHD_101", "SHD_179", "SHD_236", "SOR_168", "SOR_240", "TWI_091", "TWI_105",
        "TWI_224",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "prior modified attack lost: {code}"
        );
        let raw = catalogue()["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["setCode"] == code)
            .unwrap()
            .clone();
        assert!(!contract::prepare(&raw).unwrap());
    }
    let mut foreign = catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["setCode"] == "SOR_168")
        .unwrap()
        .clone();
    let p = contract::compiled().unwrap();
    let new_effect = p
        .instructions
        .iter()
        .find_map(|i| match i {
            Instruction::Apply(e) if contract::modifier(e) => {
                Some(serde_json::to_value(e).unwrap())
            }
            _ => None,
        })
        .unwrap();
    foreign["foreignPayload"] = new_effect;
    assert!(contract::prepare(&foreign).is_err());
    let parsed = crate::ability_ir::Catalogue::load(
        &json!({"schema":crate::ability_ir::SCHEMA,"cards":[foreign]}),
    )
    .unwrap();
    assert!(
        parsed.rejected.contains_key("SOR_168"),
        "new wrapper cannot bypass raw proof via native attack early return"
    );
}
