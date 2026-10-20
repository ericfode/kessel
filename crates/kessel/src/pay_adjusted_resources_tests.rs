//! Runtime and admission boundaries for the card-effect resource payment family
//! ("You may pay N resources. If you do, ..."). The engine charge is a constant,
//! non-cancellable card-effect payment by the ability source's controller; any
//! other payment shape must stay an explicit rejection.
use super::*;
use crate::ability_ir::{Ability, Effect, Instruction, Program};
use std::sync::OnceLock;

fn committed_catalogue() -> &'static serde_json::Value {
    static CATALOG: OnceLock<serde_json::Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .expect("committed compiler catalogue")
    })
}
fn committed_catalogue_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES
        .get_or_init(|| serde_json::to_vec(committed_catalogue()).expect("serialize catalogue"))
        .as_slice()
}
fn engine() -> Engine {
    Engine::from_catalogue_bytes(committed_catalogue_bytes()).unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e
            .by_code
            .get(code)
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))),
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine) -> State {
    let mut s = State::default();
    // This builder creates a fresh position with an explicitly empty Credit ledger.
    s.credit_state_exact = e.needs_credit_state();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (seat, leader) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, leader);
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
/// Adds physical resource cards to a player's zone in the given ready/exhausted
/// order and returns them, so exhaustion order is observable.
fn resources(e: &Engine, s: &mut State, seat: Seat, order: &[bool]) -> Vec<CardId> {
    let mut ids = vec![];
    for exhausted in order {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
        s.card_mut(id).exhausted = *exhausted;
        ids.push(id);
    }
    ids
}
fn deck_card(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "LOF_254");
    s.player_mut(seat).deck.push(id).unwrap();
    id
}
fn program(e: &Engine, s: &State, id: CardId, trigger: &str) -> u16 {
    e.card_programs(s, id)
        .iter()
        .copied()
        .find(|p| e.program(*p).trigger == trigger)
        .unwrap_or_else(|| panic!("{trigger} program on {:?}", e.def(s, id).code))
}
/// The exact compiler lowering of AbilityHelper.immediateEffects.payResources.
fn lowered(amount: u64) -> serde_json::Value {
    json!({
        "kind": "payAdjustedResources",
        "target": "ownPlayer",
        "amount": {"kind": "const", "value": amount},
        "paymentType": "cardEffectPayment",
        "isCost": false,
        "canCancelDuringPayment": false,
        "allowCostAdjustments": true
    })
}
fn effect(amount: u64) -> Effect {
    serde_json::from_value(lowered(amount)).unwrap()
}
fn optional_payment_ability(effect: serde_json::Value) -> serde_json::Value {
    json!({
        "trigger": "whenPlayed",
        "title": "Pay to draw a card",
        "costs": [],
        "optional": true,
        "effect": effect,
        "ifYouDo": {"title": "Draw a card", "costs": [], "effect": {"kind": "draw", "target": "ownPlayer", "amount": {"kind": "const", "value": 1}}}
    })
}

#[test]
fn payment_contract_accepts_only_the_lowered_compiler_shape() {
    let effect = effect(1);
    assert_eq!(crate::ability_ir::card_effect_payment(&effect).unwrap(), 1);
    let ability: Ability = serde_json::from_value(optional_payment_ability(lowered(2))).unwrap();
    let program = Program::compile(&ability).unwrap();
    assert!(matches!(
        program.instructions[0],
        Instruction::Apply(Effect::PayAdjustedResources { .. })
    ));
    // The ifYouDo continuation is gated on the payment's success result.
    assert!(matches!(
        program.instructions[1],
        Instruction::IfResult { .. }
    ));
    assert!(matches!(
        program.instructions[2],
        Instruction::Apply(Effect::Draw { .. })
    ));

    for (mutate, needle) in [
        (json!({"isCost": true}), "ability-cost"),
        (json!({"canCancelDuringPayment": true}), "cancellation"),
        (
            json!({"allowCostAdjustments": false}),
            "adjustment permission",
        ),
        (json!({"paymentType": "exhaustResources"}), "paymentType"),
        (json!({"target": "enemyPlayer"}), "source controller"),
        (json!({"target": "chosen"}), "source controller"),
        (
            json!({"amount": {"kind": "const", "value": 0}}),
            "constant from 1",
        ),
        (
            json!({"amount": {"kind": "const", "value": -1}}),
            "constant from 1",
        ),
        (
            json!({"amount": {"kind": "ref", "path": ["player", "resources", "length"]}}),
            "constant amount",
        ),
    ] {
        let mut value = lowered(1);
        for (key, replacement) in mutate.as_object().unwrap() {
            value[key] = replacement.clone();
        }
        let effect: Effect = serde_json::from_value(value).unwrap();
        assert!(
            crate::ability_ir::card_effect_payment(&effect).is_err(),
            "{mutate} must not satisfy the payment contract"
        );
        let ability: Ability = serde_json::from_value(optional_payment_ability(
            serde_json::to_value(&effect).unwrap(),
        ))
        .unwrap();
        let error = Program::compile(&ability).unwrap_err();
        assert!(error.contains(needle), "{mutate}: {error}");
    }
}

#[test]
fn payment_in_an_event_group_stays_rejected() {
    let ability: Ability = serde_json::from_value(json!({
        "trigger": "event",
        "title": "Grouped payment",
        "effect": {"kind": "simultaneous", "effects": [
            lowered(1),
            {"kind": "damage", "target": "enemyBase", "amount": {"kind": "const", "value": 1}}
        ]}
    }))
    .unwrap();
    assert!(Program::compile(&ability)
        .unwrap_err()
        .contains("unverified operation"));
}

#[test]
fn admitted_payment_cards_compile_and_unsupported_companions_stay_rejected() {
    let e = engine();
    for code in [
        "LAW_113", "LAW_148", "LAW_198", "LAW_214", "LAW_227", "LAW_257", "SOR_206", "TS26_077",
        "TWI_212",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in ["ASH_014", "ASH_219", "JTL_096", "LAW_193"] {
        assert!(
            !e.by_code.contains_key(code),
            "unsupported companion admitted on {code}"
        );
        assert!(
            e.rejected_cards().unwrap().contains_key(code),
            "{code} needs an explicit rejection"
        );
    }
}

#[test]
fn payment_charges_the_source_controller_not_the_active_player() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let own = resources(&e, &mut s, Seat::P1, &[false, false, true]);
    let other = resources(&e, &mut s, Seat::P2, &[false, false]);
    s.source = source;
    s.active = Seat::P2;
    assert_eq!(e.ready_resources(&s, Seat::P1), 2);
    assert_eq!(e.apply_effect(&mut s, &effect(2)).unwrap(), (2, true));
    assert!(s.card(own[0]).exhausted);
    assert!(s.card(own[1]).exhausted);
    assert!(s.card(own[2]).exhausted);
    assert!(!s.card(other[0]).exhausted);
    assert!(!s.card(other[1]).exhausted);
    assert_eq!(e.ready_resources(&s, Seat::P1), 0);
    assert_eq!(e.ready_resources(&s, Seat::P2), 2);
    assert_eq!(s.active, Seat::P2);
    assert!(
        !s.ability_succeeded,
        "direct effect application leaves ability result ownership to the caller"
    );
}

#[test]
fn payment_fails_atomically_without_enough_ready_resources() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let own = resources(&e, &mut s, Seat::P1, &[false, true]);
    s.source = source;
    let before = s.zobrist();
    assert_eq!(e.apply_effect(&mut s, &effect(2)).unwrap(), (0, false));
    assert!(!s.card(own[0]).exhausted);
    assert!(s.card(own[1]).exhausted);
    assert_eq!(e.ready_resources(&s, Seat::P1), 1);
    assert_eq!(s.zobrist(), before, "a failed charge must not mutate state");
}

#[test]
fn payment_rejects_unmodeled_payers_and_missing_sources() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let own = resources(&e, &mut s, Seat::P1, &[false, false]);
    s.source = source;
    let enemy: Effect = serde_json::from_value(json!({
        "kind": "payAdjustedResources", "target": "enemyPlayer",
        "amount": {"kind": "const", "value": 1}, "paymentType": "cardEffectPayment",
        "isCost": false, "canCancelDuringPayment": false, "allowCostAdjustments": true
    }))
    .unwrap();
    let error = e.apply_effect(&mut s, &enemy).unwrap_err();
    assert!(error.contains("source controller"), "{error}");
    assert!(!s.card(own[0]).exhausted);
    s.source = 0;
    let error = e.apply_effect(&mut s, &effect(1)).unwrap_err();
    assert!(error.contains("requires an ability source"), "{error}");
    assert!(!s.card(own[0]).exhausted);
}

#[test]
fn optional_when_played_payment_charges_then_runs_its_if_you_do() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LAW_227");
    let own = resources(&e, &mut s, Seat::P1, &[false, false]);
    let p = program(&e, &s, source, "whenPlayed");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    let actions = e.legal_actions(&s);
    assert!(actions.contains(&Action::button("Trigger", "trigger")));
    assert!(actions.contains(&Action::button("Pass", "pass")));

    let declined = e.apply(&s, &Action::button("Pass", "pass")).unwrap();
    assert_eq!(e.token_count(&declined, declined.card(source).shield), 0);
    assert!(!declined.card(own[0]).exhausted);
    assert_eq!(e.ready_resources(&declined, Seat::P1), 2);
    assert_eq!(declined.decision, Decision::Window);

    let done = e.apply(&s, &Action::button("Trigger", "trigger")).unwrap();
    assert!(done.card(own[0]).exhausted);
    assert!(!done.card(own[1]).exhausted);
    assert_eq!(e.ready_resources(&done, Seat::P1), 1);
    assert_eq!(e.token_count(&done, done.card(source).shield), 1);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn optional_payment_prompt_needs_an_affordable_charge() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LAW_227");
    // Exactly one ready resource is enough for a one-resource charge.
    resources(&e, &mut s, Seat::P1, &[false]);
    let p = program(&e, &s, source, "whenPlayed");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);

    // With no ready resource the ability has no legal effect, so it stays a
    // no-effect prompt and "using it anyway" cannot charge or continue.
    let mut poor = board(&e);
    let poor_source = unit(&e, &mut poor, Seat::P1, "LAW_227");
    let poor_own = resources(&e, &mut poor, Seat::P1, &[true]);
    let p = program(&e, &poor, poor_source, "whenPlayed");
    poor.source = poor_source;
    e.begin_program(&mut poor, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(poor.decision, Decision::AbilityNoEffect);
    let actions = e.legal_actions(&poor);
    assert!(actions.iter().any(|a| a.label == "Cancel"));
    let use_anyway = actions.into_iter().find(|a| a.label != "Cancel").unwrap();
    let done = e.apply(&poor, &use_anyway).unwrap();
    assert!(done.card(poor_own[0]).exhausted);
    assert_eq!(e.token_count(&done, done.card(poor_source).shield), 0);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn payment_failure_after_the_prompt_skips_the_if_you_do_continuation() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SOR_206");
    let own = resources(&e, &mut s, Seat::P1, &[false, false]);
    let drawn = deck_card(&e, &mut s, Seat::P1);
    let p = program(&e, &s, source, "onAttack");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    // The charge becomes unpayable between the prompt and its resolution, e.g. a
    // replacement or later trigger consuming the resource. The payment must then
    // fail and the gated draw must not run.
    s.card_mut(own[0]).exhausted = true;
    s.card_mut(own[1]).exhausted = true;
    let done = e.apply(&s, &Action::button("Trigger", "trigger")).unwrap();
    assert!(!done.ability_succeeded);
    assert_eq!(done.ability_result, 0);
    assert!(!done.players[0].hand.contains(drawn));
    assert!(done.players[0].deck.contains(drawn));
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn two_resource_payment_charges_and_draws_on_attack() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "SOR_206");
    let own = resources(&e, &mut s, Seat::P1, &[false, false, false, false]);
    let drawn = deck_card(&e, &mut s, Seat::P1);
    let p = program(&e, &s, source, "onAttack");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    let done = e.apply(&s, &Action::button("Trigger", "trigger")).unwrap();
    assert!(done.card(own[0]).exhausted);
    assert!(done.card(own[1]).exhausted);
    assert!(!done.card(own[2]).exhausted);
    assert!(!done.card(own[3]).exhausted);
    assert!(done.players[0].hand.contains(drawn));
    assert_eq!(done.decision, Decision::Window);
    assert_eq!(e.ready_resources(&done, Seat::P1), 2);
}

/// Runs the admitted Mining Guild TIE Fighter On Attack payment through its
/// optional prompt and the scripted trigger choice.
fn attack_payment_scenario(e: &Engine) -> State {
    let mut s = board(e);
    let source = unit(e, &mut s, Seat::P1, "SOR_206");
    resources(e, &mut s, Seat::P1, &[false, false, false, false]);
    deck_card(e, &mut s, Seat::P1);
    let p = program(e, &s, source, "onAttack");
    s.source = source;
    e.begin_program(&mut s, p, ProgramContinuation::Finish)
        .unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    e.apply(&s, &Action::button("Trigger", "trigger")).unwrap()
}

#[test]
fn payment_program_matches_between_interpreter_and_generated_backend() {
    let interpreter = engine();
    let expected = attack_payment_scenario(&interpreter);
    let mut generated = engine();
    // Isolate the payment program so the generated backend must lower it.
    for definition in &mut generated.definitions {
        if definition.code != "SOR_206" {
            definition.programs = Default::default();
        }
    }
    generated.set_rules_instrumentation(true);
    if let Err(reason) = generated.set_rules_backend(RulesBackend::Aot) {
        panic!("AOT payment program unavailable: {reason}");
    }
    let actual = attack_payment_scenario(&generated);
    assert_eq!(
        actual.zobrist(),
        expected.zobrist(),
        "generated and interpreted payment programs must agree"
    );
    let definition = &generated.definitions[generated.by_code["SOR_206"] as usize];
    let index = definition
        .programs
        .card
        .iter()
        .copied()
        .find(|p| generated.program(*p).trigger == "onAttack")
        .unwrap() as usize;
    let hits = generated.rules_hits.lock().unwrap()[&index];
    assert!(
        hits[0] > 0,
        "payment program must execute through the generated backend"
    );
    assert_eq!(
        hits[1], 0,
        "payment program must not fall back to the interpreter step"
    );
}

#[test]
fn payment_source_metadata_rejects_incompatible_card_types_and_classes() {
    let original = committed_catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["setCode"] == "LAW_227")
        .unwrap();
    crate::pay_adjusted_resources::validate_entry(original).unwrap();
    for replacement in [
        json!(["leader"]),
        json!(["upgrade"]),
        json!(["unit", "token"]),
        json!(["unit", "upgrade"]),
    ] {
        let mut entry = original.clone();
        entry["card"]["types"] = replacement;
        assert!(crate::pay_adjusted_resources::validate_entry(&entry).is_err());
    }
    let mut entry = original.clone();
    entry["baseClass"] = json!("UpgradeCard");
    assert!(crate::pay_adjusted_resources::validate_entry(&entry).is_err());
}

fn trigger_payment(e: &Engine, s: &mut State, source: CardId, trigger: &str) -> State {
    let p = program(e, s, source, trigger);
    s.source = source;
    e.begin_program(s, p, ProgramContinuation::Finish).unwrap();
    assert_eq!(s.decision, Decision::AbilityOptional);
    e.apply(s, &Action::button("Trigger", "trigger")).unwrap()
}

#[test]
fn each_selected_payment_payload_waits_until_after_payment() {
    // Pinned sources use an unqualified Unit selector except the ground-only
    // damage cards and Supplier's explicit source exclusion. The entering unit
    // remains a target when the source does not exclude it.
    for (code, trigger, charge, damage, shield, experience, excludes_self, ground_only) in [
        ("LAW_113", "whenPlayed", 1, 0, 1, 0, false, false),
        ("LAW_198", "whenPlayed", 1, 2, 0, 0, false, true),
        ("LAW_214", "whenPlayed", 1, 3, 0, 0, false, true),
        ("LAW_214", "onAttack", 1, 3, 0, 0, false, true),
        ("LAW_257", "whenPlayed", 1, 0, 0, 1, true, false),
        ("TWI_212", "whenPlayed", 2, 2, 0, 0, false, false),
    ] {
        let e = engine();
        let mut s = board(&e);
        let source = unit(&e, &mut s, Seat::P1, code);
        // Dogged Pursuers is a ground unit with 5 HP, so every table effect
        // can target it and the largest (3) damage payload remains nonlethal.
        let target = unit(&e, &mut s, Seat::P2, "LAW_198");
        let space = unit(&e, &mut s, Seat::P2, "SOR_206");
        assert!(s.player(Seat::P2).ground.contains(target));
        assert!(s.player(Seat::P2).space.contains(space));
        assert!(s.player(Seat::P1).ground.contains(source));
        resources(&e, &mut s, Seat::P1, &[false, false, false]);
        let paid = trigger_payment(&e, &mut s, source, trigger);
        assert_eq!(paid.decision, Decision::AbilityTarget, "{code}/{trigger}");
        assert_eq!(e.ready_resources(&paid, Seat::P1), 3 - charge, "{code}");
        assert_eq!(paid.card(target).damage, 0);
        assert_eq!(e.token_count(&paid, paid.card(target).shield), 0);
        assert_eq!(e.token_count(&paid, paid.card(target).experience), 0);
        let actions = e.legal_actions(&paid);
        assert!(actions.contains(&e.card_action(&paid, target)), "{code}");
        assert_eq!(
            actions.contains(&e.card_action(&paid, source)),
            !excludes_self,
            "{code}"
        );
        assert_eq!(
            actions.contains(&e.card_action(&paid, space)),
            !ground_only,
            "{code}"
        );
        let done = e.apply(&paid, &e.card_action(&paid, target)).unwrap();
        assert_eq!(done.card(target).damage, damage, "{code}/{trigger}");
        assert_eq!(
            e.token_count(&done, done.card(target).shield),
            shield,
            "{code}"
        );
        assert_eq!(
            e.token_count(&done, done.card(target).experience),
            experience,
            "{code}"
        );
        assert_eq!(e.ready_resources(&done, Seat::P1), 3 - charge);
    }
}

#[test]
fn paid_self_effects_grant_phase_stats_and_both_tokens() {
    let e = engine();
    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "LAW_148");
    resources(&e, &mut s, Seat::P1, &[false, false, false]);
    let before = (e.power(&s, source, false), e.hp(&s, source));
    let done = trigger_payment(&e, &mut s, source, "whenPlayed");
    assert_eq!(e.ready_resources(&done, Seat::P1), 2);
    assert_eq!(
        (e.power(&done, source, false), e.hp(&done, source)),
        (before.0 + 1, before.1 + 1)
    );

    let mut s = board(&e);
    let source = unit(&e, &mut s, Seat::P1, "TS26_077");
    resources(&e, &mut s, Seat::P1, &[false, false, false]);
    let done = trigger_payment(&e, &mut s, source, "whenPlayed");
    assert_eq!(e.ready_resources(&done, Seat::P1), 1);
    assert_eq!(e.token_count(&done, done.card(source).experience), 1);
    assert_eq!(e.token_count(&done, done.card(source).shield), 1);
}

#[test]
fn payment_catalogue_rejects_unreviewed_source_envelopes() {
    use crate::ability_ir::Catalogue;
    let entries = committed_catalogue()["cards"].as_array().unwrap();
    let load = |entry: Value| {
        Catalogue::load(&json!({"schema": crate::ability_ir::SCHEMA, "cards": [entry]})).unwrap()
    };
    // Check the complete catalogue admission path on every restored source,
    // including Boba's two registrations and the Ambush companions' metadata.
    for code in [
        "LAW_113", "LAW_148", "LAW_198", "LAW_214", "LAW_227", "LAW_257", "SOR_206", "TS26_077",
        "TWI_212",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["setCode"] == code)
            .unwrap();
        let loaded = load(entry.clone());
        assert!(
            loaded.cards.contains_key(code),
            "{code}: {:?}",
            loaded.rejected.get(code)
        );
    }
    let original = entries
        .iter()
        .find(|entry| entry["setCode"] == "LAW_227")
        .unwrap();
    let reject = |entry: Value, mutation: &str| {
        let loaded = load(entry);
        assert!(!loaded.cards.contains_key("LAW_227"), "admitted {mutation}");
        let reason = loaded
            .rejected
            .get("LAW_227")
            .expect("named source-envelope rejection");
        assert!(
            if ["piloting", "pilotText", "pilotBox", "pilotingBox"].contains(&mutation)
                || mutation.starts_with("pilotingAbility ")
                || mutation.starts_with("pilotUnitAbility ")
            {
                reason.contains(crate::piloting_lifecycle::REASON)
            } else {
                reason.contains(
                    "card-effect resource payment requires ordinary nonleader unit source metadata",
                )
            },
            "{mutation}: {reason}"
        );
    };
    for field in [
        "stateWatchers",
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "privateClassConstants",
        "inheritedSource",
        "deploymentOverride",
        "setupValidationOverrides",
    ] {
        let mut entry = original.clone();
        entry[field] = json!([{"kind": "unreviewedSourceContext"}]);
        reject(entry, field);
    }
    for field in ["piloting", "attachCondition"] {
        let mut entry = original.clone();
        entry[field] = json!({});
        reject(entry, field);
    }
    for field in [
        "pilotText",
        "pilotBox",
        "pilotingBox",
        "deployBox",
        "deployCost",
        "upgradePower",
        "upgradeHp",
        "epicAction",
    ] {
        let mut entry = original.clone();
        entry["card"][field] = json!("unreviewed");
        reject(entry, field);
    }
    for (field, value) in [
        ("unique", json!("yes")),
        ("types", json!(["unit", "upgrade"])),
        ("arena", json!("base")),
        ("cost", json!(-1)),
        ("hp", json!(0)),
        ("power", json!(1001)),
    ] {
        let mut entry = original.clone();
        entry["card"][field] = value;
        reject(entry, field);
    }
    let mut entry = original.clone();
    entry["baseClass"] = json!("UpgradeCard");
    reject(entry, "baseClass");
    for side in ["leader", "unit"] {
        let mut entry = original.clone();
        entry["abilities"][side] = json!([entry["abilities"]["card"][0].clone()]);
        reject(entry, side);
    }
    for continuation in [false, true] {
        for (field, value) in [
            ("gainedHost", json!(false)),
            ("attachedContext", json!("parent")),
            ("attachedHost", json!(true)),
            ("pilotingAbility", json!(true)),
            ("pilotUnitAbility", json!(true)),
            ("attackerMustSurvive", json!(true)),
            (
                "hooks",
                json!([{"event": "onCardDefeated", "predicate": {"kind": "const", "value": true}}]),
            ),
        ] {
            let mut entry = original.clone();
            let ability = &mut entry["abilities"]["card"][0];
            if continuation {
                ability["ifYouDo"][field] = value;
            } else {
                ability[field] = value;
            }
            reject(entry, &format!("{field} in continuation={continuation}"));
        }
    }
    let mut entry = original.clone();
    entry["abilities"]["card"][0]["ifYouDo"]["then"] = json!({
        "title": "Unreviewed nested context", "effect": {"kind": "noAction"},
        "attachedContext": "parent"
    });
    reject(entry, "nested then context");
}
