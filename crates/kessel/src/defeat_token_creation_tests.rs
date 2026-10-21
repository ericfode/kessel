//! Native unit tests on staged positions, not source-engine conformance games.
use super::*;
use crate::ability_ir::{Effect, Instruction, Program};
use crate::defeat_cause::DefeatCause;
use crate::defeat_token_creation_contract as contract;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(c: &Value) -> Value {
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_028")
        .unwrap()
        .clone()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e
            .by_code
            .get(code)
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))),
        owner,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 3;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    s.credit_state_exact = e.needs_credit_state();
    for p in [Seat::P1, Seat::P2] {
        s.player_mut(p).base = add(e, &mut s, p, "SOR_023");
        s.player_mut(p).leader = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn drain(e: &Engine, s: &mut State) {
    for _ in 0..100 {
        if matches!(s.decision, Decision::Window) {
            return;
        }
        let actions = e.legal_actions(s);
        let a = actions
            .first()
            .unwrap_or_else(|| panic!("missing prompt action {:?}", s.decision));
        *s = e.apply(s, a).unwrap();
    }
    panic!("unfinished captured window {:?}", s.decision);
}
fn mandos(e: &Engine, s: &State, seat: Seat) -> Vec<CardId> {
    s.player(seat)
        .ground
        .as_slice()
        .iter()
        .copied()
        .filter(|id| e.def(s, *id).info.internal_name == "mandalorian")
        .collect()
}
#[test]
fn defeat_token_creation_exact_source_and_no_code_dispatch() {
    let c = catalogue();
    let e = entry(&c);
    let p = contract::prepare(&e, &c).unwrap().unwrap();
    assert!(contract::program(&p));
    assert!(contract::requires(&p));
    let mut renamed = e.clone();
    renamed["setCode"] = json!("TEST_SAME_SOURCE");
    assert!(contract::prepare(&renamed, &c).unwrap().is_some());
    let other = c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "SOR_095")
        .unwrap();
    assert!(contract::prepare(other, &c).unwrap().is_none());
}
#[test]
fn defeat_token_creation_full_raw_factory_and_metadata_mutants_reject() {
    let c = catalogue();
    let original = entry(&c);
    for (pointer, value) in [
        ("/baseClass", json!("EventCard")),
        ("/className", json!("Other")),
        ("/card/types", json!(["token", "unit"])),
        ("/card/keywords", json!([])),
        ("/keywordValues/sentinel", json!(0)),
        ("/card/hp", json!(8)),
        ("/diagnostics", json!([{"code":"bad"}])),
        ("/abilities/card/0/costs", json!([{"kind":"exhaustSelf"}])),
        ("/abilities/card/0/effect/condition/op", json!("===")),
        (
            "/abilities/card/0/effect/condition/left/path",
            json!(["source", "activeAttack"]),
        ),
        (
            "/abilities/card/0/effect/onFalse/kind",
            json!("nativeTokenUnitEventFactory"),
        ),
        (
            "/abilities/card/0/effect/onTrue/properties/amount/value",
            json!(0),
        ),
        (
            "/abilities/card/0/effect/onTrue/properties/controllers",
            json!("enemyPlayer"),
        ),
        (
            "/abilities/card/0/effect/onTrue/properties/entersReady",
            json!(true),
        ),
        (
            "/abilities/card/0/effect/onTrue/creationEvent/triggerEventRegister",
            json!("overwrite"),
        ),
        (
            "/abilities/card/0/effect/onTrue/updateEvent/cancelledEventTokens",
            json!("discard"),
        ),
        (
            "/abilities/card/0/effect/onTrue/contingentEventFactory/contextEvent",
            json!("creationEvent"),
        ),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(contract::prepare(&bad, &c).is_err(), "{pointer}");
    }
    for key in ["optional", "unknown"] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0][key] = json!(true);
        assert!(contract::prepare(&bad, &c).is_err());
    }
    for key in ["historyHelpers", "stateWatchers", "privatePrograms"] {
        let mut bad = original.clone();
        bad[key] = json!([{"extra":true}]);
        assert!(contract::prepare(&bad, &c).is_err());
    }
    let mut bad = original.clone();
    bad["card"]["pilotText"] = json!("Piloting [1]");
    assert!(contract::prepare(&bad, &c).is_err());
    let mut bad = original.clone();
    bad["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .push(original["abilities"]["card"][0].clone());
    assert!(contract::prepare(&bad, &c).is_err());
}
#[test]
fn defeat_token_creation_selected_dependencies_missing_or_changed_reject() {
    let c = catalogue();
    let original = entry(&c);
    for code in ["TOKEN_MANDALORIAN", "TOKEN_SHIELD"] {
        let mut missing = c.clone();
        missing["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|e| e["setCode"] != code);
        assert!(contract::prepare(&original, &missing).is_err());
        for field in ["sourceHash", "baseClass", "diagnostics"] {
            let mut bad = c.clone();
            let token = bad["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["setCode"] == code)
                .unwrap();
            token[field] = json!("wrong");
            assert!(contract::prepare(&original, &bad).is_err());
        }
        let mut duplicate = c.clone();
        let token = duplicate["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["setCode"] == code)
            .unwrap()
            .clone();
        duplicate["cards"].as_array_mut().unwrap().push(token);
        assert!(contract::prepare(&original, &duplicate).is_err());
    }
    let selected = json!({"schema":"swu-card-ir/v1","cards":c["cards"].as_array().unwrap().iter().filter(|e|["ASH_028","TOKEN_MANDALORIAN","TOKEN_SHIELD"].iter().any(|id|e["setCode"]==*id)).cloned().collect::<Vec<_>>()});
    assert!(contract::prepare(&original, &selected).unwrap().is_some());
    assert!(Engine::from_catalogue_value(&selected)
        .unwrap()
        .by_code
        .contains_key("ASH_028"));
}
#[test]
fn defeat_token_creation_captured_cause_predicate_and_false_branch_no_creation_instruction() {
    for cause in [
        DefeatCause::Attack,
        DefeatCause::NonCombatDamage,
        DefeatCause::Ability,
        DefeatCause::FrameworkEffect,
        DefeatCause::UniqueRule,
    ] {
        assert_eq!(
            contract::evaluate(cause).unwrap(),
            cause != DefeatCause::Attack
        );
    }
    assert!(contract::evaluate(DefeatCause::Unknown).is_err());
    let p = contract::prepare(&entry(&catalogue()), &catalogue())
        .unwrap()
        .unwrap();
    let Instruction::Conditional { condition, skip } = &p.instructions[0] else {
        panic!("missing branch")
    };
    assert!(
        matches!(condition,crate::ability_ir::Expr::Query{name,receiver,args} if contract::query(name,receiver,args))
    );
    assert!(matches!(p.instructions[*skip], Instruction::EndStep));
    assert!(
        matches!(&p.instructions[1],Instruction::Group(Effect::CreateTokenUnit{token_type,amount:crate::ability_ir::Expr::Const{value},enters_ready:false,..}) if token_type=="mandalorian"&&value.as_u64()==Some(2))
    );
    let mut bad = p.clone();
    bad.optional = true;
    assert!(!contract::program(&bad));
    let mut bad = p.clone();
    bad.instructions.remove(0);
    assert!(!contract::program(&bad));
    assert!(Program::compile(&serde_json::from_value(json!({"trigger":"whenDefeated","title":"untrusted","effect":{"kind":"conditional","condition":contract::predicate(),"onTrue":{"kind":"createTokenUnit","tokenType":"mandalorian","amount":{"kind":"const","value":2},"target":"ownPlayer"},"onFalse":{"kind":"noAction"}}})).unwrap()).is_err());
}
#[test]
fn defeat_token_creation_all_captured_causes_both_seats_and_offboard_shielded_batch() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for cause in [
            DefeatCause::Attack,
            DefeatCause::NonCombatDamage,
            DefeatCause::Ability,
            DefeatCause::FrameworkEffect,
            DefeatCause::UniqueRule,
        ] {
            let mut s = board(&e, seat);
            let paz = unit(&e, &mut s, seat, "ASH_028");
            let attacker = unit(&e, &mut s, seat.other(), "SOR_095");
            s.attack_source = attacker;
            s.source = attacker;
            let first = s.next_id;
            e.defeat_events(&mut s, &[(paz, cause)]).unwrap();
            assert!(!e.in_arena(&s, paz));
            let _ = e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
            drain(&e, &mut s);
            let ids = mandos(&e, &s, seat);
            if cause == DefeatCause::Attack {
                assert!(ids.is_empty());
                assert_eq!(s.next_id, first);
                assert!(!s.event_group.active);
            } else {
                assert_eq!(ids, vec![first, first + 1]);
                assert_eq!(s.next_id, first + 4);
                for id in ids {
                    assert_eq!(s.card(id).owner, seat);
                    assert!(s.card(id).exhausted);
                    assert_eq!(e.power(&s, id, false), 2);
                    assert_eq!(e.hp(&s, id), 2);
                    assert_ne!(s.card(id).shield, 0);
                    assert_eq!(s.card(s.card(id).shield).owner, seat);
                }
            }
            assert!(s.player(seat).discard.contains(paz));
        }
    }
}
#[test]
fn defeat_token_creation_sibling_sources_capture_separate_players_and_causes() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "ASH_028");
    let b = unit(&e, &mut s, Seat::P2, "ASH_028");
    e.defeat_events(
        &mut s,
        &[(a, DefeatCause::Attack), (b, DefeatCause::NonCombatDamage)],
    )
    .unwrap();
    e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
    drain(&e, &mut s);
    assert!(mandos(&e, &s, Seat::P1).is_empty());
    assert_eq!(mandos(&e, &s, Seat::P2).len(), 2);
}
#[test]
fn defeat_token_creation_unknown_context_rejected_without_mutation() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let paz = unit(&e, &mut s, Seat::P1, "ASH_028");
    s.source = paz;
    let p = e.card_programs(&s, paz)[0];
    let before = s;
    assert!(e
        .begin_program(&mut s, p, ProgramContinuation::Finish)
        .is_err());
    assert_eq!(s, before);
}
#[test]
fn defeat_token_creation_sentinel_targeting() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let paz = unit(&e, &mut s, Seat::P1, "ASH_028");
    let friendly = unit(&e, &mut s, Seat::P1, "SOR_095");
    let attacker = unit(&e, &mut s, Seat::P2, "SOR_095");
    let targets = e.attack_targets(&s, attacker, false);
    assert!(targets.contains(&paz));
    assert!(!targets.contains(&friendly));
    assert!(!targets.contains(&s.player(Seat::P1).base));
}

#[test]
fn defeat_token_creation_retained_catalogue_cannot_supply_omitted_selected_tokens() {
    let c = catalogue();
    let e = engine();
    for code in ["TOKEN_MANDALORIAN", "TOKEN_SHIELD"] {
        let mut pool = e.compiled_pool().unwrap();
        pool["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v["setCode"] != code);
        assert!(
            Engine::from_compiled_pool_bytes(
                &serde_json::to_vec(&pool).unwrap(),
                &serde_json::to_vec(&c).unwrap()
            )
            .is_err(),
            "{code}"
        );
    }
}

#[test]
fn defeat_token_creation_suspended_physical_source_player_and_cause_restore() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let paz = unit(&e, &mut s, seat, "ASH_028");
        let other = unit(&e, &mut s, seat.other(), "SOR_095");
        s.source = paz;
        s.defeat_cause = DefeatCause::NonCombatDamage;
        let saved = ResolutionContext::capture(&s);
        let before_hash = s.zobrist();
        s.source = other;
        s.active = seat.other();
        s.defeat_cause = DefeatCause::Attack;
        saved.restore(&mut s);
        assert_eq!(s.source, paz);
        assert_eq!(s.active, seat);
        assert_eq!(s.defeat_cause, DefeatCause::NonCombatDamage);
        assert_eq!(s.zobrist(), before_hash);
        assert!(contract::evaluate(s.defeat_cause).unwrap());
    }
}

#[test]
fn defeat_token_creation_actual_native_ability_damage_during_attack_is_noncombat() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat.other());
        let paz = unit(&e, &mut s, seat, "ASH_028");
        let attacker = unit(&e, &mut s, seat.other(), "ASH_131");
        s.source = attacker;
        s.attack_source = attacker;
        s.attack_defender = paz;
        s.ability_target = paz;
        let effect: Effect = serde_json::from_value(
            json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":7}}),
        )
        .unwrap();
        e.apply_effect(&mut s, &effect).unwrap();
        e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
        drain(&e, &mut s);
        assert_eq!(mandos(&e, &s, seat).len(), 2);
        assert!(s.player(seat).discard.contains(paz));
    }
}

#[test]
fn defeat_token_creation_actual_native_combat_negative_uses_real_attack_actions() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat.other());
        let paz = unit(&e, &mut s, seat, "ASH_028");
        let attacker = unit(&e, &mut s, seat.other(), "ASH_131");
        let first = s.next_id;
        let attack = e.card_action(&s, attacker);
        assert!(e.legal_actions(&s).contains(&attack));
        s = e.apply(&s, &attack).unwrap();
        let target = e.card_action(&s, paz);
        assert!(e.legal_actions(&s).contains(&target));
        s = e.apply(&s, &target).unwrap();
        drain(&e, &mut s);
        assert!(s.player(seat).discard.contains(paz));
        assert!(mandos(&e, &s, seat).is_empty());
        assert_eq!(s.next_id, first);
    }
}

#[test]
fn defeat_token_creation_pending_unknown_cause_rejects_before_window_cleanup() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let paz = unit(&e, &mut s, seat, "ASH_028");
        e.defeat_events(&mut s, &[(paz, DefeatCause::Unknown)])
            .unwrap();
        let before = s;
        assert!(e.open_deaths(&mut s, DeathContinuation::Finish).is_err());
        assert_eq!(s, before);
    }
}

#[test]
fn defeat_token_creation_current_window_native_shields_preflight_before_publication() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for slots in [4, 3] {
            let actor = seat.other();
            let mut s = board(&e, actor);
            let paz = unit(&e, &mut s, seat, "ASH_028");
            let event = add(&e, &mut s, actor, "TWI_077");
            s.player_mut(actor).hand.push(event).unwrap();
            for _ in 0..24 {
                let resource = add(&e, &mut s, actor, "LOF_254");
                s.player_mut(actor).resources.push(resource).unwrap();
            }
            // Synthetic bounded native position with earlier token identities
            // outside play; this is not an actual source-engine game claim.
            s.next_id = (MAX_CARDS - slots) as CardId;
            let first = s.next_id;
            let play = e.card_action(&s, event);
            assert!(e.legal_actions(&s).contains(&play));
            s = e.apply(&s, &play).unwrap();
            let target = e.card_action(&s, paz);
            assert!(e.legal_actions(&s).contains(&target));
            let before = s;
            let result = e.apply(&s, &target);
            assert_eq!(s, before, "public apply never mutates its input");
            if slots == 3 {
                assert!(result.is_err(), "current window cannot publish two mandatory native Shield choices with only one physical ID remaining");
            } else {
                let mut done = result.unwrap();
                // Creation allocates both Mandalorians in order, while Shield
                // allocation remains at each native Shielded resolver.
                assert_eq!(done.next_id, first + 2);
                assert_eq!(mandos(&e, &done, seat), vec![first, first + 1]);
                assert!(mandos(&e, &done, seat)
                    .iter()
                    .all(|id| done.card(*id).shield == 0));
                drain(&e, &mut done);
                assert_eq!(done.next_id, first + 4);
                assert!(mandos(&e, &done, seat)
                    .iter()
                    .all(|id| done.card(*id).shield != 0));
            }
        }
    }
}

#[test]
fn defeat_token_creation_framework_dead_tokens_need_no_native_shield_slots() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let paz = unit(&e, &mut s, seat, "ASH_028");
        unit(&e, &mut s, seat.other(), "SHD_037");
        s.next_id = (MAX_CARDS - 2) as CardId;
        let first = s.next_id;
        e.defeat_events(&mut s, &[(paz, DefeatCause::Ability)])
            .unwrap();
        e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
        drain(&e, &mut s);
        assert!(mandos(&e, &s, seat).is_empty());
        assert_eq!(
            s.next_id,
            first + 2,
            "framework-dead created units allocate no native Shields"
        );
        assert_eq!(s.card(first).shield, 0);
        assert_eq!(s.card(first + 1).shield, 0);
    }
}
