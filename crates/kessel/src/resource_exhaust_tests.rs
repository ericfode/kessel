//! Source-derived witnesses; these do not claim real-engine conformance.
use super::*;
use crate::ability_ir::{Ability, Catalogue, Effect, Program};
use std::sync::OnceLock;

fn catalogue() -> &'static Value {
    static C: OnceLock<Value> = OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
    })
}
fn entry(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn admits(entry: Value) -> bool {
    Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[entry.clone()]}))
        .unwrap()
        .cards
        .contains_key(entry["setCode"].as_str().unwrap())
}
fn engine() -> Engine {
    Engine::from_catalogue_value(catalogue()).unwrap()
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
fn board(e: &Engine, seat: Seat) -> State {
    let mut s = State::default();
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s.credit_state_exact = e.needs_credit_state();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (owner, code) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, owner, "ASH_019");
        let leader = add(e, &mut s, owner, code);
        s.player_mut(owner).base = base;
        s.player_mut(owner).leader = leader;
    }
    s
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn resources(e: &Engine, s: &mut State, seat: Seat, order: &[bool]) -> Vec<CardId> {
    order
        .iter()
        .map(|&exhausted| {
            let id = add(e, s, seat, "LOF_254");
            s.player_mut(seat).resources.push(id).unwrap();
            s.card_mut(id).exhausted = exhausted;
            id
        })
        .collect()
}
fn program(e: &Engine, s: &State, source: CardId, trigger: &str) -> u16 {
    e.card_programs(s, source)
        .iter()
        .copied()
        .find(|&p| e.program(p).trigger == trigger)
        .unwrap()
}
fn leaf(target: &str) -> Value {
    json!({"kind":"exhaustResources","target":target,"amount":{"kind":"const","value":1},"paymentType":"exhaustResources","isCost":false,"allowCostAdjustments":false})
}

#[test]
fn resource_exhaust_admits_exactly_the_reviewed_current_sources() {
    let e = engine();
    for code in ["ASH_216", "TWI_185"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in ["JTL_250", "SEC_216", "SEC_235", "SHD_191", "SOR_188"] {
        assert!(!e.by_code.contains_key(code), "unreviewed source {code}");
        assert!(e.rejected_cards().unwrap().contains_key(code));
    }
    // Card identity is not an admission switch.
    let mut renamed = entry("ASH_216");
    renamed["setCode"] = json!("TEST_001");
    renamed["internalName"] = json!("same-semantics");
    assert!(admits(renamed));
}

#[test]
fn resource_exhaust_rejects_payment_target_amount_and_nested_drift() {
    let original = entry("ASH_216");
    assert!(admits(original.clone()));
    for change in [
        json!({"isCost":true}),
        json!({"allowCostAdjustments":true}),
        json!({"paymentType":"cardEffectPayment"}),
        json!({"amount":{"kind":"const","value":0}}),
        json!({"amount":{"kind":"const","value":2}}),
        json!({"amount":{"kind":"const","value":-1}}),
        json!({"amount":{"kind":"ref","path":["player","resources","length"]}}),
        json!({"target":"chosenPlayer"}),
        json!({"target":"bothPlayers"}),
        json!({"target":{"kind":"registrationSnapshot","value":{"kind":"ref","path":["source","controller"]}}}),
        json!({"canCancelDuringPayment":false}),
    ] {
        let mut bad = original.clone();
        for (k, v) in change.as_object().unwrap() {
            bad["abilities"]["card"][0]["effect"][k] = v.clone();
        }
        assert!(!admits(bad), "{change}");
    }
    for nested in [
        json!({"kind":"optional","effect":leaf("ownPlayer")}),
        json!({"kind":"sequence","effects":[leaf("ownPlayer")]}),
        json!({"kind":"simultaneous","effects":[leaf("ownPlayer")]}),
        json!({"kind":"conditional","condition":{"kind":"const","value":true},"onTrue":leaf("ownPlayer")}),
        json!({"kind":"select","targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":leaf("ownPlayer")}),
    ] {
        let mut bad = original.clone();
        bad["abilities"]["card"][0]["effect"] = nested;
        assert!(!admits(bad));
    }
    // Envelopes must preserve mandatory death, optional attack, and the full
    // companion unit selector, not merely recognize a supported leaf somewhere.
    for (code, index, key, value) in [
        ("ASH_216", 0, "optional", json!(true)),
        ("ASH_216", 0, "trigger", json!("whenPlayed")),
        ("ASH_216", 0, "costs", json!([{"kind":"exhaustSelf"}])),
        (
            "ASH_216",
            0,
            "condition",
            json!({"kind":"const","value":true}),
        ),
        (
            "ASH_216",
            0,
            "then",
            json!({"title":"extra","effect":{"kind":"noAction"}}),
        ),
        ("TWI_185", 1, "optional", json!(false)),
        (
            "TWI_185",
            0,
            "effect",
            json!({"kind":"ready","target":"chosen"}),
        ),
    ] {
        let mut bad = entry(code);
        bad["abilities"]["card"][index][key] = value;
        assert!(!admits(bad), "{code}/{key}");
    }
}

#[test]
fn resource_exhaust_rejects_foreign_metadata_helpers_and_side_registrations() {
    let original = entry("ASH_216");
    for key in [
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "privateClassConstants",
        "inheritedSource",
        "deploymentOverride",
        "setupValidationOverrides",
        "stateWatchers",
    ] {
        let mut bad = original.clone();
        bad[key] = json!([{"unreviewed":true}]);
        assert!(!admits(bad), "{key}");
    }
    for types in [
        json!(["leader"]),
        json!(["upgrade"]),
        json!(["unit", "token"]),
        json!(["unit", "upgrade"]),
    ] {
        let mut bad = original.clone();
        bad["card"]["types"] = types;
        assert!(!admits(bad));
    }
    for (key, value) in [
        ("arena", json!("base")),
        ("hp", json!(0)),
        ("cost", json!(-1)),
        ("power", json!(null)),
        ("unique", json!(null)),
        ("pilotText", json!("extra")),
        ("deployCost", json!(4)),
        ("upgradeHp", json!(1)),
        ("epicAction", json!("extra")),
    ] {
        let mut bad = original.clone();
        bad["card"][key] = value;
        assert!(!admits(bad), "{key}");
    }
    for key in ["piloting", "attachCondition"] {
        let mut bad = original.clone();
        bad[key] = json!({"extra":true});
        assert!(!admits(bad));
    }
    let mut bad = original.clone();
    bad["baseClass"] = json!("LeaderCard");
    assert!(!admits(bad));
    for side in ["leader", "unit"] {
        let mut bad = original.clone();
        bad["abilities"][side] = json!([original["abilities"]["card"][0].clone()]);
        assert!(!admits(bad));
    }
    let mut bad = original.clone();
    bad["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}}));
    assert!(!admits(bad));
}

#[test]
fn resource_exhaust_leaf_validation_is_strict_and_scalar_order_is_physical() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for target in ["ownPlayer", "enemyPlayer"] {
            let mut s = board(&e, seat.other());
            let source = unit(&e, &mut s, seat, "ASH_216");
            s.source = source;
            let recipient = if target == "ownPlayer" {
                seat
            } else {
                seat.other()
            };
            let own = resources(&e, &mut s, recipient, &[true, false, false, true, false]);
            let other = resources(&e, &mut s, recipient.other(), &[false]);
            e.create_credit_tokens(&mut s, recipient, 2).unwrap();
            let credits = s.player(recipient).credits;
            let effect: Effect = serde_json::from_value(leaf(target)).unwrap();
            assert!(e.effect_can_affect(&s, source, 0, &effect, true));
            assert_eq!(e.apply_effect(&mut s, &effect).unwrap(), (1, true));
            assert!(s.card(own[0]).exhausted && s.card(own[1]).exhausted);
            assert!(
                !s.card(own[2]).exhausted
                    && !s.card(own[4]).exhausted
                    && !s.card(other[0]).exhausted
            );
            assert_eq!(e.ready_resources(&s, recipient), 2);
            assert_eq!(s.active, seat.other());
            assert!(!s.resource_payment.active());
            assert_eq!(s.player(recipient).credits, credits);
            for id in own {
                s.card_mut(id).exhausted = true;
            }
            assert!(!e.effect_can_affect(&s, source, 0, &effect, true));
            assert_eq!(e.apply_effect(&mut s, &effect).unwrap(), (0, false));
            assert_eq!(
                s.player(recipient).credits,
                credits,
                "plain effects cannot spend Credits"
            );
        }
    }
    let ability: Ability =
        serde_json::from_value(entry("ASH_216")["abilities"]["card"][0].clone()).unwrap();
    assert!(Program::compile(&ability).is_ok());
    for value in [0, 2, 65536] {
        let mut raw = leaf("ownPlayer");
        raw["amount"]["value"] = json!(value);
        let effect: Effect = serde_json::from_value(raw).unwrap();
        assert!(crate::resource_exhaust::amount(&effect).is_err());
    }
}

fn attack_prompt(e: &Engine, seat: Seat) -> (State, CardId, Vec<CardId>, Vec<CardId>) {
    let mut s = board(e, seat);
    let source = unit(e, &mut s, seat, "TWI_185");
    let own = resources(e, &mut s, seat, &[false, false]);
    let enemy = resources(e, &mut s, seat.other(), &[true, false, false]);
    let p = program(e, &s, source, "onAttack");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    (s, source, own, enemy)
}
#[test]
fn resource_exhaust_ziro_optional_attack_trigger_decline_and_zero_both_owners() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (s, _, own, enemy) = attack_prompt(&e, seat);
        let declined = e.apply(&s, &Action::button("Pass", "pass")).unwrap();
        assert!(!declined.card(enemy[1]).exhausted);
        assert!(!declined.card(own[0]).exhausted);
        let done = e.apply(&s, &Action::button("Trigger", "trigger")).unwrap();
        assert!(done.card(enemy[1]).exhausted);
        assert!(!done.card(enemy[2]).exhausted);
        assert!(!done.card(own[0]).exhausted);
        assert_eq!(done.decision, Decision::Window);
        let (mut s, _, _, enemy) = attack_prompt(&e, seat);
        for id in enemy {
            s.card_mut(id).exhausted = true;
        }
        let done = e.apply(&s, &Action::button("Trigger", "trigger")).unwrap();
        assert_eq!(e.ready_resources(&done, seat.other()), 0);
        assert_eq!(done.decision, Decision::Window);
    }
}

#[test]
fn resource_exhaust_ziro_complete_when_played_enemy_unit_selector() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "TWI_185");
        let friendly = unit(&e, &mut s, seat, "LOF_254");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
        let p = program(&e, &s, source, "whenPlayed");
        s.source = source;
        e.begin_program(&mut s, p, ProgramContinuation::Finish)
            .unwrap();
        assert_eq!(s.decision, Decision::AbilityTarget);
        let declined = e.apply(&s, &Action::button("Pass", "passAbility")).unwrap();
        assert!(!declined.card(enemy).exhausted);
        let prompt = s;
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        let legal = e.legal_actions(&prompt);
        assert!(legal.contains(&e.card_action(&prompt, enemy)));
        assert!(!legal.contains(&e.card_action(&prompt, friendly)));
        assert!(!legal.contains(&e.card_action(&prompt, source)));
        let done = e.apply(&prompt, &e.card_action(&prompt, enemy)).unwrap();
        assert!(done.card(enemy).exhausted);
        assert!(!done.card(friendly).exhausted);
    }
}

#[test]
fn resource_exhaust_scout_mandatory_death_uses_captured_controller_after_departure() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat.other());
        let source = unit(&e, &mut s, seat, "ASH_216");
        let own = resources(&e, &mut s, seat, &[true, false, false]);
        let other = resources(&e, &mut s, seat.other(), &[false]);
        e.defeat(&mut s, source).unwrap();
        assert!(!e.in_arena(&s, source));
        assert!(s.player(seat).discard.contains(source));
        assert_eq!(s.death_pending.len, 1);
        assert_eq!(s.death_pending.entries[0].player, seat);
        e.open_deaths(&mut s, DeathContinuation::Finish).unwrap();
        for _ in 0..8 {
            if matches!(s.decision, Decision::Window) {
                break;
            }
            assert_ne!(
                s.decision,
                Decision::AbilityOptional,
                "Scout cannot be declined"
            );
            let actions = e.legal_actions(&s);
            let a = actions
                .iter()
                .find(|a| a.arg == Some(json!("all")))
                .unwrap_or_else(|| actions.first().unwrap());
            s = e.apply(&s, a).unwrap();
        }
        assert_eq!(s.decision, Decision::Window);
        assert!(s.card(own[1]).exhausted);
        assert!(!s.card(own[2]).exhausted);
        assert!(!s.card(other[0]).exhausted);
    }
}

#[test]
fn resource_exhaust_generated_dispatch_and_interpreter_share_the_state_change() {
    let mut interpreter = engine();
    interpreter
        .set_rules_backend(RulesBackend::Interpreter)
        .unwrap();
    let mut generated = engine();
    for d in &mut generated.definitions {
        if !["ASH_216", "TWI_185"].contains(&d.code.as_str()) {
            d.programs = Default::default();
        }
    }
    generated.set_rules_instrumentation(true);
    generated.set_rules_backend(RulesBackend::Aot).unwrap();
    for seat in [Seat::P1, Seat::P2] {
        let (si, _, _, _) = attack_prompt(&interpreter, seat);
        let (sg, _, _, _) = attack_prompt(&generated, seat);
        let expected = interpreter
            .apply(&si, &Action::button("Trigger", "trigger"))
            .unwrap();
        let actual = generated
            .apply(&sg, &Action::button("Trigger", "trigger"))
            .unwrap();
        assert_eq!(actual.zobrist(), expected.zobrist());
    }
    assert!(generated
        .rules_hits
        .lock()
        .unwrap()
        .values()
        .any(|hits| hits[0] > 0));
    assert!(generated
        .rules_hits
        .lock()
        .unwrap()
        .values()
        .all(|hits| hits[1] == 0));
}
