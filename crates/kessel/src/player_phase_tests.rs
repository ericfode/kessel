use super::*;
use crate::player_phase_contract::Kind;
use crate::player_phase_state::{Ledger, Row};
#[test]
fn player_phase_compact_rows_capture_zero_and_capacity_atomic() {
    assert_eq!(std::mem::size_of::<Row>(), 6);
    let mut l = Ledger::default();
    for _ in 0..64 {
        l.append(Row::new(1, 0, Kind::ImperialDiscount, 0, 0, 0).unwrap())
            .unwrap();
    }
    let before = l;
    assert!(l
        .append(Row::new(1, 0, Kind::ImperialDiscount, 0, 0, 0).unwrap())
        .is_err());
    assert_eq!(l, before);
    let captured = l.capture(Kind::ImperialDiscount, 0);
    assert_eq!(captured.count(), 64);
    l.consume(captured).unwrap();
    assert_eq!(l.capture(Kind::ImperialDiscount, 0).count(), 0);
    l.consume(captured).unwrap();
    assert_eq!(l.len, 64);
    l.expire();
    assert_eq!(l.len, 0);
}
fn phase_catalogue() -> Value {
    let mut v: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    for n in 0..2 {
        let entry = crate::player_phase_contract::fixture(n);
        let code = entry["setCode"].clone();
        let slot = v["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["setCode"] == code)
            .unwrap();
        *slot = entry;
    }
    v
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&phase_catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, p: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        owner: p,
        def: e.by_code[code],
        ..Card::default()
    };
    id
}
fn board(e: &Engine, p: Seat) -> State {
    let mut s = State::default();
    s.active = p;
    s.action_player = Some(p);
    s.force_state_exact = true;
    s.hidden_history_exact = true;
    s.credit_state_exact = e.needs_credit_state();
    for p in [Seat::P1, Seat::P2] {
        let b = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).base = b;
        let l = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).leader = l;
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn install(e: &Engine, s: &mut State, id: CardId, ordinal: usize) {
    s.source = id;
    s.active = s.card(id).owner;
    s.ability_program = e.def(s, id).programs.card[ordinal] + 1;
    let kind = if e.def(s, id).code == "ASH_237" {
        Kind::ImperialDiscount
    } else {
        Kind::LowPrintedPowerReady
    };
    e.install_player_phase(s, kind).unwrap();
    s.ability_program = 0;
    s.source = 0;
}
#[test]
fn player_phase_exact_full_source_and_standalone_guards() {
    let e = engine();
    for (n, code) in [(0, "ASH_237"), (1, "ASH_248")] {
        assert!(!e.rejected_cards().unwrap().contains_key(code));
        let entry = crate::player_phase_contract::fixture(n);
        for field in [
            "extra",
            "amount",
            "cost",
            "trait",
            "type",
            "keyword",
            "source",
            "mix",
            "top",
            "target",
            "capabilities",
        ] {
            let mut bad = entry.clone();
            match field {
                "extra" => bad["abilities"]["card"][0]["effect"]["extra"] = json!(true),
                "amount" => {
                    bad["abilities"]["card"][0]["effect"]["effects"][0]["amount"] =
                        json!({"kind":"const","value":2})
                }
                "cost" => bad["abilities"]["card"][0]["costs"] = json!([{"kind":"exhaustSelf"}]),
                "trait" => bad["card"]["traits"] = json!(["spectre"]),
                "type" => bad["baseClass"] = json!("TokenUnitCard"),
                "keyword" => bad["card"]["keywords"] = json!(["support"]),
                "source" => bad["sourceHash"] = json!("unknown"),
                "top" => bad["unknownSourceMethod"] = json!({}),
                "target" => bad["abilities"]["card"][0]["effect"]["target"] = json!("enemyPlayer"),
                "capabilities" => bad["requiredCapabilities"] = json!(["unknown"]),
                _ => bad["abilities"]["leader"] = bad["abilities"]["card"].clone(),
            };
            assert!(
                crate::player_phase_contract::prepare(&bad).is_err(),
                "{field}"
            );
        }
        let p = e.program(
            e.by_code
                .get(code)
                .map(|d| e.definitions[*d as usize].programs.card[0])
                .unwrap(),
        );
        let effect = match &p.instructions[0] {
            crate::ability_ir::Instruction::Apply(effect) => effect.clone(),
            _ => panic!(),
        };
        let mut ability:crate::ability_ir::Ability=serde_json::from_value(json!({"trigger":"whenPlayed","title":"malicious","costs":[],"effect":{"kind":"noAction"}})).unwrap();
        ability.effect = effect;
        assert!(crate::ability_ir::Program::compile(&ability).is_err());
        assert!(crate::player_phase_contract::copied_program_allowed(p).is_err());
    }
}
#[test]
fn player_phase_all_captured_adjusters_consume_at_zero_without_late_filters() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_237");
        s.player_mut(seat).discard.push(source).unwrap();
        for _ in 0..4 {
            install(&e, &mut s, source, 0);
        }
        let target = add(&e, &mut s, seat, "ASH_237");
        s.player_mut(seat).hand.push(target).unwrap();
        assert_eq!(e.cost(&s, target), 0);
        let request = e.resource_request(
            &s,
            crate::resource_payment::PaymentPurpose::UnitPlay,
            target,
            0,
            0,
            false,
            true,
            false,
        );
        assert_eq!(request.phase_adjusters.count(), 4);
        // Captured handler runs despite source departure and an intervening state
        // which would make eligibility false; it never reruns a source/unused filter.
        s.card_mut(source).exhausted = true;
        e.pay_resource_cost(&mut s, request).unwrap();
        assert_eq!(s.player_phase.len, 4);
        assert_eq!(
            s.player_phase
                .capture(Kind::ImperialDiscount, seat.index() as u8)
                .count(),
            0
        );
        let effect = e.resource_request(
            &s,
            crate::resource_payment::PaymentPurpose::Effect,
            target,
            0,
            0,
            false,
            false,
            false,
        );
        assert_eq!(effect.phase_adjusters.count(), 0);
    }
}
#[test]
fn player_phase_ready_uses_printed_power_and_consumes_all_even_already_ready() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_248");
        s.player_mut(seat).discard.push(source).unwrap();
        install(&e, &mut s, source, 0);
        install(&e, &mut s, source, 1);
        assert_eq!(
            s.player_phase.rows[0].program, s.player_phase.rows[1].program,
            "one source registration despite two executable hooks"
        );
        let unit = add(&e, &mut s, seat, "IBH_007");
        s.card_mut(unit).phase_power = 20;
        s.card_mut(unit).exhausted = false;
        assert!(e.phase_ready_at_construction(&mut s, unit).unwrap());
        assert_eq!(
            s.player_phase
                .capture(Kind::LowPrintedPowerReady, seat.index() as u8)
                .count(),
            0
        );
    }
}
#[test]
fn player_phase_wire_spent_source_gone_roundtrip_and_strict_mutants() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_237");
        s.player_mut(seat).discard.push(source).unwrap();
        install(&e, &mut s, source, 0);
        let m = s
            .player_phase
            .capture(Kind::ImperialDiscount, seat.index() as u8);
        s.player_phase.consume(m).unwrap();
        let v = e.snapshot(&s);
        let imported = e.load_state(&v).unwrap();
        assert_eq!(e.snapshot(&imported), v);
        assert_eq!(imported.zobrist(), s.zobrist());
        for field in [
            "missing",
            "extra",
            "wrongKind",
            "wrongOwner",
            "wrongOrigin",
            "spent",
            "capacity",
            "pending",
        ] {
            let mut bad = v.clone();
            match field {
                "missing" => {
                    bad.as_object_mut().unwrap().remove("playerPhaseState");
                }
                "extra" => bad["playerPhaseState"]["effects"][0]["unknown"] = json!(true),
                "wrongKind" => {
                    bad["playerPhaseState"]["effects"][0]["kind"] = json!("lowPrintedPowerReady")
                }
                "wrongOwner" => {
                    bad["playerPhaseState"]["effects"][0]["target"] = json!(seat.other().as_str())
                }
                "wrongOrigin" => bad["playerPhaseState"]["effects"][0]["ordinal"] = json!(1),
                "spent" => bad["playerPhaseState"]["effects"][0]["used"] = json!([1, 1]),
                "pending" => bad["playerPhasePending"] = json!(false),
                _ => {
                    bad["playerPhaseState"]["effects"] =
                        json!(vec![v["playerPhaseState"]["effects"][0].clone(); 65])
                }
            };
            assert!(e.load_state(&bad).is_err(), "{field}");
        }
    }
}
#[test]
fn player_phase_copy_full_resolved_set_rejected_before_native_mutation() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let d = add(&e, &mut s, seat, "ASH_037");
        e.add_arena(&mut s, d).unwrap();
        let a = add(&e, &mut s, seat, "IBH_007");
        e.add_arena(&mut s, a).unwrap();
        let source = add(&e, &mut s, seat, "ASH_248");
        s.player_mut(seat).discard.push(source).unwrap();
        let p = e.def(&s, source).programs.card[1];
        s.source = d;
        s.support.len = 1;
        s.support.grants[0] = SupportGrant {
            target: d,
            donor: a,
            len: 1,
            ..SupportGrant::default()
        };
        s.support.grants[0].programs[0] = p;
        let before = s;
        assert!(e.begin_support_attack(&mut s, a).is_err());
        assert_eq!(s, before);
        // Synthetic raw grant references retained exact typed programme, not a real
        // source game. Rejection must happen after resolution and before publication.
        s.support = SupportState::default();
        s.source = 0;
        let mut v = e.snapshot(&s);
        v["supportGrants"] = json!([{"target":format!("c{}",a-1),"donor":format!("c{}",d-1),"programs":[serde_json::to_value(e.program(p)).unwrap()],"keywords":vec![0;9],"presence":0}]);
        assert!(e
            .load_state(&v)
            .unwrap_err()
            .contains("copied player phase"));
    }
}
#[test]
fn player_phase_capacity_preflight_preserves_mutable_hand_and_resources() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = add(&e, &mut s, Seat::P1, "ASH_237");
    s.player_mut(Seat::P1).hand.push(source).unwrap();
    for _ in 0..64 {
        install(&e, &mut s, source, 0);
    }
    let before = s;
    assert!(e.play_unit_with_payment(&mut s, source, false).is_err());
    assert_eq!(s, before);
    assert_eq!(e.snapshot(&s), e.snapshot(&before));
    assert_eq!(s.zobrist(), before.zobrist());
}
#[test]
fn player_phase_measured_layout() {
    eprintln!(
        "phase layout State={} Row={} Ledger={} Payment={} Context={} Frame={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<Row>(),
        std::mem::size_of::<Ledger>(),
        std::mem::size_of::<crate::resource_payment::ResourcePayment>(),
        std::mem::size_of::<ResolutionContext>(),
        std::mem::size_of::<DeathFrame>()
    );
}
#[test]
fn player_phase_normal_paid_play_and_actual_native_mandatory_handler() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e, seat);
            for _ in 0..8 {
                let r = add(&e, &mut s, seat, "IBH_007");
                s.player_mut(seat).resources.push(r).unwrap();
            }
            let mouse = add(&e, &mut s, seat, "ASH_237");
            s.player_mut(seat).hand.push(mouse).unwrap();
            assert!(e.play_unit_with_payment(&mut s, mouse, false).unwrap());
            assert_eq!(
                s.player_phase.len, 1,
                "mandatory source handler installs exactly once without confirmation"
            );
            assert!(e.in_arena(&s, mouse));
            assert_eq!(s.decision, Decision::Window);
            let second = add(&e, &mut s, seat, "ASH_237");
            s.player_mut(seat).hand.push(second).unwrap();
            let cost = e.cost(&s, second);
            s.active = seat;
            s.action_player = Some(seat);
            let ready = e.ready_resources(&s, seat);
            e.play_unit_with_payment(&mut s, second, false).unwrap();
            assert_eq!(e.ready_resources(&s, seat), ready - cost as usize);
            assert_eq!(s.player_phase.len, 2);
            assert!(!s.player_phase.rows[0].available());
            assert!(s.player_phase.rows[1].available());
        }
    }
}
#[test]
fn player_phase_spent_rows_survive_source_death_and_expire_at_actual_boundary() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = add(&e, &mut s, Seat::P1, "ASH_248");
    e.add_arena(&mut s, source).unwrap();
    install(&e, &mut s, source, 0);
    e.defeat(&mut s, source).unwrap();
    assert_eq!(s.player_phase.len, 1);
    s.player_mut(Seat::P1).out = true;
    s.player_mut(Seat::P2).out = true;
    e.advance(&mut s).unwrap();
    assert_eq!(s.player_phase.len, 0);
}
#[test]
fn player_phase_structural_alias_uniqueness_both_incoming_orders_and_opponent() {
    let mut c = phase_catalogue();
    let mut alias = crate::player_phase_contract::fixture(1);
    alias["setCode"] = json!("TEST_248");
    alias["card"]["id"] = json!("synthetic-physical-printing");
    c["cards"].as_array_mut().unwrap().push(alias);
    let e = Engine::from_catalogue_value(&c).unwrap();
    for seat in [Seat::P1, Seat::P2] {
        for reverse in [false, true] {
            let mut s = board(&e, seat);
            let a = add(
                &e,
                &mut s,
                seat,
                if reverse { "TEST_248" } else { "ASH_248" },
            );
            let b = add(
                &e,
                &mut s,
                seat,
                if reverse { "ASH_248" } else { "TEST_248" },
            );
            e.add_arena(&mut s, a).unwrap();
            e.add_arena(&mut s, b).unwrap();
            assert_eq!(e.duplicates(&s, a), 2);
            assert_eq!(e.duplicates(&s, b), 2);
            s.source = b;
            s.decision = Decision::Unique;
            let actions = e.legal_actions(&s);
            assert!(actions.contains(&e.card_action(&s, a)));
            assert!(actions.contains(&e.card_action(&s, b)));
            s.source = 0;
            s.decision = Decision::Window;
            let v = e.snapshot(&s);
            assert!(
                e.load_state(&v).is_err(),
                "unsettled alias pair cannot reload"
            );
            e.defeat(&mut s, a).unwrap();
            let other = add(&e, &mut s, seat.other(), "ASH_248");
            e.add_arena(&mut s, other).unwrap();
            assert_eq!(e.duplicates(&s, b), 1);
            assert_eq!(e.duplicates(&s, other), 1);
            let v = e.snapshot(&s);
            assert!(e.load_state(&v).is_ok());
        }
    }
}
#[test]
fn player_phase_uniqueness_cross_family_key_is_symmetric() {
    let mut e = engine();
    let phase = e.by_code["ASH_248"] as usize;
    let mut ordinary = e.definitions[phase].clone();
    ordinary.code = "syntheticNonphaseSameName".into();
    ordinary.programs = crate::ability_ir::CardPrograms::default();
    let index = e.definitions.len() as u16;
    e.definitions.push(ordinary);
    let mut s = board(&e, Seat::P1);
    let a = add(&e, &mut s, Seat::P1, "ASH_248");
    let b = s.next_id;
    s.next_id += 1;
    *s.card_mut(b) = Card {
        present: true,
        owner: Seat::P1,
        def: index,
        ..Card::default()
    };
    e.add_arena(&mut s, a).unwrap();
    e.add_arena(&mut s, b).unwrap();
    assert!(e.phase_unique_identity(&s, a, b));
    assert!(e.phase_unique_identity(&s, b, a));
    s.card_mut(b).owner = Seat::P2;
    assert_eq!(e.duplicates(&s, a), 1);
}
#[test]
fn player_phase_credit_pause_cancel_and_handler_capture() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_237");
        s.player_mut(seat).discard.push(source).unwrap();
        install(&e, &mut s, source, 0);
        for _ in 0..4 {
            let r = add(&e, &mut s, seat, "IBH_007");
            s.player_mut(seat).resources.push(r).unwrap();
        }
        e.create_credit_tokens(&mut s, seat, 2).unwrap();
        let incoming = add(&e, &mut s, seat, "ASH_237");
        s.player_mut(seat).hand.push(incoming).unwrap();
        assert!(!e.play_unit_with_payment(&mut s, incoming, false).unwrap());
        assert_eq!(s.decision, Decision::CreditMenu);
        assert_eq!(s.resource_payment.phase_adjusters.count(), 1);
        assert!(s.player_phase.rows[0].available());
        let snapshot = e.snapshot(&s);
        assert!(snapshot.get("directUnitPlayCreditState").is_some());
        assert_eq!(e.load_state(&snapshot).unwrap().zobrist(),s.zobrist());
        let mut unsupported=s;unsupported.ability_target=source;
        let unsupported=e.snapshot(&unsupported);assert_eq!(unsupported["playerPhasePending"],true);
        assert!(e.load_state(&unsupported).is_err());
        let actions = e.legal_actions(&s);
        let cancel = actions.iter().find(|a| a.label == "Cancel").unwrap();
        let cancelled = e.apply(&s, cancel).unwrap();
        assert!(cancelled.player_phase.rows[0].available());
        assert!(cancelled.player(seat).hand.contains(incoming));
        assert_eq!(cancelled.player(seat).credits.len(), 2);
        let mut paid = cancelled;
        paid.active = seat;
        paid.action_player = Some(seat);
        e.play_unit_with_payment(&mut paid, incoming, false)
            .unwrap();
        let skip = e
            .legal_actions(&paid)
            .into_iter()
            .find(|a| a.label == "Pay costs without Credit tokens")
            .unwrap();
        let paid = e.apply(&paid, &skip).unwrap();
        assert!(!paid.player_phase.rows[0].available());
        assert_eq!(paid.player_phase.len, 2);
    }
}
#[test]
fn player_phase_captured_incoming_unique_source_still_installs_when_defeated() {
    let e = engine();
    for keep_incoming in [false, true] {
        let mut s = board(&e, Seat::P1);
        for _ in 0..8 {
            let r = add(&e, &mut s, Seat::P1, "IBH_007");
            s.player_mut(Seat::P1).resources.push(r).unwrap();
        }
        let old = add(&e, &mut s, Seat::P1, "ASH_248");
        e.add_arena(&mut s, old).unwrap();
        install(&e, &mut s, old, 0);
        let incoming = add(&e, &mut s, Seat::P1, "ASH_248");
        s.player_mut(Seat::P1).hand.push(incoming).unwrap();
        e.play_unit_with_payment(&mut s, incoming, false).unwrap();
        assert_eq!(s.decision, Decision::Unique);
        e.settle_enter_triggers(&mut s, Some(if keep_incoming { old } else { incoming }))
            .unwrap();
        assert_eq!(s.player_phase.len, 2);
        assert_eq!(s.player_phase.rows[1].source, incoming);
        assert_eq!(e.in_arena(&s, incoming), keep_incoming);
    }
}
#[test]
fn player_phase_executable_closure_rejects_competing_program_roots() {
    use crate::ability_ir::{Effect, Instruction};
    let e = engine();
    let all = &e.catalogue.as_ref().unwrap().programs;
    crate::player_phase_contract::validate_direct_only_programs(all).unwrap();
    let attack = all
        .iter()
        .find(|p| {
            crate::player_phase_contract::program_operations(p)
                .unwrap()
                .attacks
                != 0
        })
        .unwrap();
    for trigger in [
        "whenDefeated",
        "onAttack",
        "onCardPlayed",
        "onUnitEntersPlay",
    ] {
        let mut bad = attack.clone();
        bad.trigger = trigger.into();
        assert!(crate::player_phase_contract::validate_direct_only_programs(&[bad]).is_err());
    }
    let mut bad = e
        .program(e.definitions[e.by_code["ASH_248"] as usize].programs.card[1])
        .clone();
    let scalar = match &bad.instructions[0] {
        Instruction::Apply(e) => e.clone(),
        _ => panic!(),
    };
    bad.instructions[0] = Instruction::Apply(Effect::Sequence {
        effects: vec![scalar],
    });
    assert!(crate::player_phase_contract::validate_direct_only_programs(&[bad]).is_err());
    let report = e.rules_backend_report();
    eprintln!(
        "phase actual programme report keys={:?}",
        report.as_object().unwrap().keys().collect::<Vec<_>>()
    );
}
#[test]
fn player_phase_capacity_precedes_parent_attack_commit_and_early_modifiers() {
    use crate::ability_ir::{Effect, Instruction};
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let neel = add(&e, &mut s, Seat::P1, "ASH_248");
    e.add_arena(&mut s, neel).unwrap();
    for _ in 0..64 {
        install(&e, &mut s, neel, 0);
    }
    let event = add(&e, &mut s, Seat::P1, "SOR_220");
    s.player_mut(Seat::P1).hand.push(event).unwrap();
    for _ in 0..8 {
        let r = add(&e, &mut s, Seat::P1, "IBH_007");
        s.player_mut(Seat::P1).resources.push(r).unwrap();
    }
    s.source = event;
    s.ability_program = e.def(&s, event).programs.card[0] + 1;
    s.ability_target = neel;
    let before = s;
    assert!(e.commit_program_mode(&mut s, false).is_err());
    assert_eq!(s, before);
    assert_eq!(s.zobrist(), before.zobrist());
    let p = e
        .program(s.ability_program - 1)
        .instructions
        .iter()
        .find_map(|i| {
            if let Instruction::Apply(Effect::InitiateAttack { properties, .. }) = i {
                Some(properties)
            } else {
                None
            }
        })
        .unwrap();
    assert!(e.begin_native_attack(&mut s, p, false).is_err());
    assert_eq!(s, before);
}
#[test]
fn player_phase_expiry_checks_captured_roots_before_identity_reuse() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = add(&e, &mut s, Seat::P1, "ASH_237");
    s.player_mut(Seat::P1).discard.push(source).unwrap();
    install(&e, &mut s, source, 0);
    let mask = s.player_phase.capture(Kind::ImperialDiscount, 0);
    s.resource_payment.phase_adjusters = mask;
    s.player_mut(Seat::P1).out = true;
    s.player_mut(Seat::P2).out = true;
    let before = s;
    assert!(e.advance(&mut s).is_err());
    assert_eq!(s, before);
    s.resource_payment = crate::resource_payment::ResourcePayment::default();
    s.death_depth = 1;
    s.death_frames[0].context.resource_payment.phase_adjusters = mask;
    let before = s;
    assert!(e.advance(&mut s).is_err());
    assert_eq!(s, before);
}
#[test]
fn player_phase_nonphase_support_control_and_legacy_presence_unchanged() {
    let mut base: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    base["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["setCode"] != "ASH_237" && e["setCode"] != "ASH_248");
    let e = Engine::from_catalogue_value(&base).unwrap();
    let mut s = board(&e, Seat::P1);
    let donor = add(&e, &mut s, Seat::P1, "ASH_037");
    e.add_arena(&mut s, donor).unwrap();
    let attacker = add(&e, &mut s, Seat::P1, "IBH_007");
    e.add_arena(&mut s, attacker).unwrap();
    s.source = donor;
    e.begin_support_attack(&mut s, attacker).unwrap();
    assert_eq!(s.support.len, 1);
    assert_eq!(s.player_phase.len, 0);
    s.support.pending = false;
    s.support.context = ResolutionContext::default();
    s.source = 0;
    s.ability_program = 0;
    s.decision = Decision::Window;
    s.action_player = Some(Seat::P1);
    let v = e.snapshot(&s);
    assert!(v.get("playerPhaseState").is_none());
    let mut explicit = v.clone();
    explicit["playerPhaseState"] = json!({"version":1,"effects":[]});
    let a = e.load_state(&v).unwrap();
    let b = e.load_state(&explicit).unwrap();
    assert_eq!(a.zobrist(), b.zobrist());
    assert_eq!(e.snapshot(&a), v);
    assert_eq!(e.snapshot(&b), explicit);
}
#[test]
fn player_phase_actual_admission_and_programme_closure_receipt() {
    use std::collections::BTreeSet;
    let e = engine();
    let baseline: Value = serde_json::from_slice(include_bytes!(
        "../../../compiler/generated/runtime-admission.json"
    ))
    .unwrap();
    let old = baseline["runtimeAdmittedCards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["setCode"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    let pool = e.compiled_pool().unwrap();
    let new = pool["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["setCode"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert!(old.is_subset(&new));
    // These source profiles must stay compiled as the catalogue grows.
    // Historical whole-catalogue gains are checked by the external CLI receipt.
    for index in 0..2 {
        let entry = crate::player_phase_contract::fixture(index);
        let code = entry["setCode"].as_str().unwrap();
        assert!(new.contains(code));
        let projection = crate::player_phase_contract::prepare(&entry).unwrap().unwrap();
        let (_, expected) = projection.compile(0).unwrap();
        let bindings = &e.definitions[e.by_code[code] as usize].programs.card;
        assert_eq!(bindings.len(), expected.len());
        for (&id, program) in bindings.iter().zip(&expected) {
            assert_eq!(serde_json::to_value(e.program(id)).unwrap(), serde_json::to_value(program).unwrap());
        }
    }
    let report = e.rules_backend_report();
    let bound = report["cards"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|s| s.as_object().unwrap().values())
        .flat_map(|p| p.as_array().unwrap())
        .map(|i| i.as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    let receipt = json!({"scope":"private experimental only; pending source ABI and whole proof not accepted","credit":0,"previousAdmissions":old.len(),"experimentalAdmissions":new.len(),"added":new.difference(&old).collect::<Vec<_>>(),"boundPrograms":bound.len(),"allProgramRows":report["programs"].as_array().unwrap().len(),"bindingRows":report["cards"].as_object().unwrap().len(),"stateBytes":std::mem::size_of::<State>()});
    eprintln!("phase actual closure {receipt}");
    if let Ok(dir) = std::env::var("SWU_PHASE_REPORT_DIR") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        for (name, value) in [
            ("closure.json", receipt),
            ("rules-full.json", report),
            ("catalogue.json", phase_catalogue()),
            ("experimental-pool.json", pool),
        ] {
            std::fs::write(dir.join(name), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
    }
}

#[test]
fn player_phase_actual_compiler_entries_use_exact_closed_source_contract() {
    let catalogue: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    for code in ["ASH_237", "ASH_248"] {
        let entry = catalogue["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == code)
            .unwrap();
        assert!(
            crate::player_phase_contract::prepare(entry)
                .unwrap()
                .is_some(),
            "actual compiler entry {code}"
        );
    }
}

#[test]
fn player_phase_full_copy_hashes_both_mask_banks_and_saved_payment_roots() {
    // Constructed storage witness: 64 zero-effective adjusters need no public
    // pause. The copied roots below test value ownership, not source reachability.
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = add(&e, &mut s, Seat::P1, "ASH_237");
    s.player_mut(Seat::P1).discard.push(source).unwrap();
    for _ in 0..64 {
        install(&e, &mut s, source, 0);
    }
    let mask = s.player_phase.capture(Kind::ImperialDiscount, 0);
    assert_eq!(mask.0, [u32::MAX, u32::MAX]);
    s.resource_payment = e.resource_request(
        &s,
        crate::resource_payment::PaymentPurpose::UnitPlay,
        source,
        0,
        0,
        false,
        true,
        false,
    );
    s.resource_payment.phase_adjusters = mask;
    s.death_depth = 1;
    s.death_frames[0].context.resource_payment = s.resource_payment;
    let mut copied = s;
    assert_eq!(copied.zobrist(), s.zobrist());
    copied.player_phase.consume(mask).unwrap();
    assert_ne!(copied.zobrist(), s.zobrist());
    assert_eq!(
        s.player_phase.capture(Kind::ImperialDiscount, 0).count(),
        64
    );
    let mut changed_root = s;
    changed_root.death_frames[0]
        .context
        .resource_payment
        .phase_adjusters
        .0[1] ^= 1 << 31;
    assert_ne!(changed_root.zobrist(), s.zobrist());
    assert_eq!(
        s.death_frames[0].context.resource_payment.phase_adjusters.0[1],
        u32::MAX
    );
}

#[test]
fn player_phase_retained_unselected_raw_support_import_rejects_typed_installer() {
    let full = engine();
    let mut pool = full.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["setCode"] != "ASH_237" && r["setCode"] != "ASH_248");
    let e = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&phase_catalogue()).unwrap(),
    )
    .unwrap();
    assert!(!e.needs_player_phase_state());
    assert!(e
        .catalogue
        .as_ref()
        .unwrap()
        .metadata
        .get("ASH_248")
        .is_none());
    let neel_program = e.definitions[e.by_code["ASH_248"] as usize].programs.card[1];
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let donor = add(&e, &mut s, seat, "ASH_037");
        e.add_arena(&mut s, donor).unwrap();
        let attacker = add(&e, &mut s, seat, "IBH_007");
        e.add_arena(&mut s, attacker).unwrap();
        let mut v = e.snapshot(&s);
        assert!(v.get("playerPhaseState").is_none());
        let row = |p| json!({"target":format!("c{}",attacker-1),"donor":format!("c{}",donor-1),"programs":[serde_json::to_value(e.program(p)).unwrap()],"keywords":vec![0;9],"presence":0});
        v["supportGrants"] = json!([row(neel_program)]);
        assert!(e
            .load_state(&v)
            .unwrap_err()
            .contains("copied player phase"));
        // Existing nonphase legacy raw programmes remain valid against the same
        // retained global catalogue and producer-free selected pool.
        let ordinary_program = e.def(&s, donor).programs.card[0];
        v["supportGrants"] = json!([row(ordinary_program)]);
        let loaded = e.load_state(&v).unwrap();
        assert_eq!(loaded.support.len, 1);
        assert_eq!(loaded.support.grants[0].programs[0], ordinary_program);
    }
}

#[test]
fn player_phase_unique_before_capture_is_explicitly_pending_even_after_winner_latch() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let neel = add(&e, &mut s, Seat::P1, "ASH_248");
    e.add_arena(&mut s, neel).unwrap();
    s.source = neel;
    s.decision = Decision::Unique;
    s.pending_programs = 0;
    s.ability_program = 0;
    s.outcome = 1;
    let wire = e.snapshot(&s);
    assert_eq!(wire["playerPhasePending"], true);
    assert!(e.load_state(&wire).is_err());
}

#[test]
fn player_phase_checked_trigger_card_identity_range_copy_and_atomic_boundary() {
    assert_eq!(std::mem::size_of::<CapturedCardId>(),1);
    assert_eq!(std::mem::size_of::<DeathTrigger>(),14);
    assert_eq!(std::mem::size_of::<DeathWindow>(),906);
    for id in [0,1,255] {
        let captured=CapturedCardId::checked(id).unwrap();
        assert_eq!(captured.get(),id);
        for slot in 0..MAX_DEATH_DEPTH+2 {
            let mut s=State::default();
            let w=match slot {0=>&mut s.death_pending,1=>&mut s.death_window,n=>&mut s.death_frames[n-2].window};
            w.entries[63]=DeathTrigger{event_card:captured,grantor:captured,..DeathTrigger::default()};
            let copy=s;assert_eq!(copy,s);assert_eq!(copy.zobrist(),s.zobrist());
            assert_eq!(w_identity(&copy,slot), (id,id));
        }
    }
    assert!(CapturedCardId::checked(256).is_err());
    assert!(CapturedCardId::checked(u16::MAX).is_err());
    let e=engine();let mut s=board(&e,Seat::P1);let before=s;
    assert!(e.capture_unit_entry_watchers(&mut s,&[256]).is_err());assert_eq!(s,before);
    assert!(e.capture_defeat_window(&mut s,&[(256,crate::defeat_cause::DefeatCause::Ability)]).is_err());assert_eq!(s,before);
    assert!(e.capture_round_limit_window(&mut s,&[(256,crate::round_limit_runtime::Event::AttackDeclared{attacker_controller:0,ground_attacker:true,any_base_target:true})]).is_err());assert_eq!(s,before);
    fn w_identity(s:&State,slot:usize)->(CardId,CardId) {let w=match slot {0=>&s.death_pending,1=>&s.death_window,n=>&s.death_frames[n-2].window};(w.entries[63].event_card.get(),w.entries[63].grantor.get())}
}

#[test]
fn player_phase_source_255_retained_after_departure_roundtrips_without_truncation() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);s.next_id=255;
        let source=add(&e,&mut s,seat,"ASH_237");assert_eq!(source,255);
        s.player_mut(seat).discard.push(source).unwrap();install(&e,&mut s,source,0);
        let value=e.snapshot(&s);assert_eq!(value["playerPhaseState"]["effects"][0]["source"],"c254");
        let loaded=e.load_state(&value).unwrap();assert_eq!(loaded.player_phase,s.player_phase);assert_eq!(loaded.zobrist(),s.zobrist());assert_eq!(e.snapshot(&loaded),value);
        let mut bad=value.clone();bad["playerPhaseState"]["effects"][0]["source"]=json!("c256");assert!(e.load_state(&bad).is_err());
        let mut bad=value.clone();bad["nextCardId"]=json!(257);assert!(e.load_state(&bad).is_err());
    }
}

#[test]
fn player_phase_mandatory_empty_wire_preserves_native_state_equality_and_hash() {
    let e=engine();let mut s=board(&e,Seat::P1);s.support_state_explicit=true;assert!(!s.player_phase.wire_present);
    let value=e.snapshot(&s);assert_eq!(value["playerPhaseState"],json!({"version":1,"effects":[]}));
    let loaded=e.load_state(&value).unwrap();assert_eq!(loaded,s);assert_eq!(loaded.zobrist(),s.zobrist());assert_eq!(e.snapshot(&loaded),value);
}

#[test]
fn player_phase_real_credit_use_skip_cancel_and_error_preserve_captured_rows() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1,Seat::P2] {
            let mut initial=board(&e,seat);
            for _ in 0..2 {let source=add(&e,&mut initial,seat,"ASH_237");initial.player_mut(seat).discard.push(source).unwrap();install(&e,&mut initial,source,0);}
            for _ in 0..16 {let r=add(&e,&mut initial,seat,"IBH_007");initial.player_mut(seat).resources.push(r).unwrap();}
            let incoming=add(&e,&mut initial,seat,"SOR_232");initial.player_mut(seat).hand.push(incoming).unwrap();
            e.create_credit_tokens(&mut initial,seat,1).unwrap();let amount=e.cost(&initial,incoming);assert!(amount>1);
            let ready=e.ready_resources(&initial,seat);
            let mut prompt=initial;assert!(!e.play_unit_with_payment(&mut prompt,incoming,false).unwrap());assert_eq!(prompt.decision,Decision::CreditMenu);assert_eq!(prompt.resource_payment.phase_adjusters.count(),2);
            let copy=prompt;assert_eq!(copy,prompt);assert_eq!(copy.zobrist(),prompt.zobrist());
            let cancel=e.legal_actions(&prompt).into_iter().find(|a|a.label=="Cancel").unwrap();let cancelled=e.apply(&prompt,&cancel).unwrap();assert_eq!(cancelled.player_phase,initial.player_phase);assert_eq!(cancelled.player(seat).credits.len(),1);assert_eq!(e.ready_resources(&cancelled,seat),ready);assert!(cancelled.player(seat).hand.contains(incoming));
            let skip=e.legal_actions(&prompt).into_iter().find(|a|a.label=="Pay costs without Credit tokens").unwrap();let skipped=e.apply(&prompt,&skip).unwrap();assert_eq!(skipped.player(seat).credits.len(),1);assert_eq!(e.ready_resources(&skipped,seat),ready-usize::from(amount));assert!(skipped.player_phase.rows[..2].iter().all(|r|!r.available()));
            let use_credit=e.legal_actions(&prompt).into_iter().find(|a|a.label.starts_with("Use 1 Credit")).unwrap();let used=e.apply(&prompt,&use_credit).unwrap();assert_eq!(used.player(seat).credits.len(),0);assert_eq!(e.ready_resources(&used,seat),ready-usize::from(amount-1));assert!(used.player_phase.rows[..2].iter().all(|r|!r.available()));assert!(e.in_arena(&used,incoming));
            // The native Credit definition has no own programme, is never an
            // arena unit/attached host, and no admitted defeat predicate sees it.
            let credit=initial.player(seat).credits.first().unwrap();let mut observation=initial;let before=observation.death_pending;e.capture_defeat_window(&mut observation,&[(credit,crate::defeat_cause::DefeatCause::Ability)]).unwrap();assert_eq!(observation.death_pending,before);assert!(observation.player_phase.rows[..2].iter().all(|r|r.available()));
            // Invalid native ownership fails before spending or publishing usage.
            let mut bad=initial;bad.card_mut(credit).owner=seat.other();let request=e.resource_request(&bad,crate::resource_payment::PaymentPurpose::UnitPlay,incoming,0,amount,false,true,false);let before=bad;assert!(e.pay_resource_cost(&mut bad,request).is_err());assert_eq!(bad,before);assert_eq!(bad.zobrist(),before.zobrist());assert_eq!(bad.player_phase,before.player_phase);assert_eq!(bad.player(seat).resources,before.player(seat).resources);assert_eq!(bad.player(seat).credits,before.player(seat).credits);assert_eq!(bad.player(seat).hand,before.player(seat).hand);
        }
    }
}

#[test]
fn player_phase_real_smuggle_play_consumes_neel_but_not_nonmatching_mouse() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=board(&e,seat);
        for code in ["ASH_237","ASH_248"] {let source=add(&e,&mut s,seat,code);s.player_mut(seat).discard.push(source).unwrap();install(&e,&mut s,source,0);}
        for _ in 0..16 {let r=add(&e,&mut s,seat,"IBH_007");s.player_mut(seat).resources.push(r).unwrap();}
        let incoming=add(&e,&mut s,seat,"SHD_097");s.player_mut(seat).resources.push(incoming).unwrap();let top=add(&e,&mut s,seat,"IBH_007");s.player_mut(seat).deck.push(top).unwrap();
        assert!(e.smuggle_cost(&s,incoming).is_some());assert_eq!(e.phase_cost_capture(&s,incoming).count(),0,"Freetown Backup is not Imperial");
        assert!(e.play_unit_with_payment(&mut s,incoming,true).unwrap());assert!(e.in_arena(&s,incoming));assert!(!s.card(incoming).exhausted);assert!(s.player_phase.rows[0].available());assert!(!s.player_phase.rows[1].available());assert!(s.player(seat).resources.contains(top));
    }
}

#[test]
fn player_phase_nested_player_limit_cannot_bypass_registration_limit_guard() {
    for n in 0..2 {for key in ["limit","roundLimit","round_limit"] {
        let mut entry=crate::player_phase_contract::fixture(n);
        entry["abilities"]["card"][0][key]=json!({"kind":"abilityLimit","max":1,"scope":"perPlayerPerRound"});
        assert!(crate::player_phase_contract::prepare(&entry).is_err());
        let code=entry["setCode"].as_str().unwrap().to_owned();let c=json!({"schema":crate::ability_ir::SCHEMA,"cards":[entry]});let loaded=crate::ability_ir::Catalogue::load(&c).unwrap();assert!(!loaded.cards.contains_key(&code));
    }}
}

#[path="direct_unit_credit_tests.rs"]
mod direct_unit_credit_tests;
