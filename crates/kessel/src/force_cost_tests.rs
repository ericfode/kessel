// Source-derived Rust midgame assertions; no reference-engine execution.
use super::*;

fn start(e: &Engine, seat: Seat, code: &str) -> (State, CardId) {
    let mut s = board(e);
    s.active = seat;
    s.action_player = Some(seat);
    let source = if code == "LOF_178" { unit(e, &mut s, seat, code) } else {
        let id = s.player(seat).leader;
        s.card_mut(id).def = e.by_code[code];
        id
    };
    s.player_mut(seat).force_held = true;
    (s, source)
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(e.legal_actions(s).contains(&a), "{id}: {:?}", e.legal_actions(s));
    e.apply(s, &a).unwrap()
}
fn begin_action(e: &Engine, s: &State, id: CardId) -> State {
    let t = choose(e, s, id);
    if t.decision == Decision::AbilityMenu {
        named_button(e, &t, "Exhaust a unit")
    } else { t }
}

#[test]
fn force_cost_actual_leaders_target_cancel_pay_and_preserve_identity() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine(); e.set_rules_backend(backend).unwrap();
        for seat in [Seat::P1, Seat::P2] { for code in ["LOF_002", "LOF_003"] {
            let (mut s, source) = start(&e, seat, code);
            let own = unit(&e, &mut s, seat, "SHD_189");
            let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
            let p = e.available_programs(&s, source, "action")[0];
            assert!(e.program(p).executable.is_ok(), "canonical Force-cost action must have generated dispatch");
            let force_id = s.player(seat).force_id;
            let next = s.next_id;
            let selected = begin_action(&e, &s, source);
            assert_eq!(selected.decision, Decision::AbilityTarget);
            assert!(selected.player(seat).force_held);
            assert!(!selected.card(source).exhausted);
            assert_eq!(named_button(&e, &selected, "Cancel"), s);
            let ids: Vec<_> = e.legal_actions(&selected).iter().filter_map(Action::id).collect();
            assert!(ids.contains(&own)); assert_eq!(ids.contains(&enemy), code == "LOF_002");
            let done = choose(&e, &selected, own);
            assert!(!done.player(seat).force_held); assert!(done.card(source).exhausted);
            assert_eq!(done.player(seat).force_id, force_id); assert_eq!(done.next_id, next);
            if code == "LOF_002" { assert_eq!((done.card(own).phase_power, done.card(own).phase_hp), (-1,-1)); }
            else { assert!(e.has_keyword(&done, own, "sentinel")); assert!(!e.has_keyword(&done, enemy, "sentinel")); }
            assert!(e.available_programs(&done, source, "action").is_empty());
            let mut no_force = s; no_force.player_mut(seat).force_held = false;
            assert!(e.available_programs(&no_force, source, "action").is_empty());
            let restored = e.load_state(&e.snapshot(&done)).unwrap();
            assert_eq!(e.snapshot(&done), e.snapshot(&restored));
            assert_eq!(done.zobrist(), restored.zobrist());
        }}
    }
}

#[test]
fn force_cost_adept_exhausts_enemy_and_no_effect_confirmation_still_pays() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let (mut s, source) = start(&e, seat, "LOF_178");
        let target = unit(&e, &mut s, seat.other(), "LOF_256");
        let selected = begin_action(&e, &s, source);
        let done = choose(&e, &selected, target);
        assert!(done.card(target).exhausted && done.card(source).exhausted);
        assert!(!done.player(seat).force_held);
        assert_eq!(done.player(seat.other()).force_held, s.player(seat.other()).force_held);
        // Ahsoka has no friendly arena unit: confirmation is before all costs.
        let (empty, leader) = start(&e, seat, "LOF_003");
        let prompt = begin_action(&e, &empty, leader);
        assert_eq!(prompt.decision, Decision::AbilityNoEffect);
        assert_eq!(named_button(&e, &prompt, "Cancel"), empty);
        let paid = named_button(&e, &prompt, "Use it anyway");
        assert!(!paid.player(seat).force_held && paid.card(leader).exhausted);
    }
}

#[test]
fn force_cost_raw_contract_duplicate_and_atomic_payment_boundaries() {
    let cost = json!({"kind":"useForceCost","target":"ownPlayer","timing":"abilityCostPayment","mustFullyResolve":true});
    let ability = json!({"trigger":"action","title":"Diagnostic Force payment","costs":[cost.clone()],"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}}});
    let compile = |v: Value| serde_json::from_value::<crate::ability_ir::Ability>(v).map_err(|e|e.to_string()).and_then(|a|crate::ability_ir::Program::compile(&a));
    assert!(compile(ability.clone()).is_ok());
    for (key,value) in [("target",json!("enemyPlayer")),("timing",json!("afterEffect")),("mustFullyResolve",json!(false)),("ignoredField",json!(true))] {
        let mut bad = ability.clone(); bad["costs"][0][key] = value;
        assert!(compile(bad).is_err(), "{key}");
    }
    let mut duplicate = ability.clone(); duplicate["costs"] = json!([cost.clone(),cost]);
    assert!(compile(duplicate).unwrap_err().contains("duplicate Force"));
    let e = engine(); let (mut s, source) = start(&e, Seat::P1, "LOF_002");
    let target = unit(&e, &mut s, Seat::P1, "LOF_254");
    s = begin_action(&e, &s, source);
    s.ability_target = target; s.player_mut(Seat::P1).force_held = false;
    let before = s;
    assert!(e.commit_program(&mut s).unwrap_err().contains("Force cost"));
    assert_eq!(s, before); // failed Force payment cannot exhaust the source first.
}

#[test]
fn force_cost_native_dependency_and_snapshot_history_are_required() {
    let original: Value = serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let mut missing = original.clone();
    missing["cards"].as_array_mut().unwrap().retain(|v|v["setCode"]!="TOKEN_THE_FORCE");
    let e = Engine::from_catalogue_value(&missing).unwrap();
    for code in ["LOF_002","LOF_003","LOF_178"] { assert!(!e.by_code.contains_key(code)); }
    let e = engine(); let (mut s, source) = start(&e, Seat::P1, "LOF_003");
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"].as_array_mut().unwrap().retain(|c| c["setCode"] != "TOKEN_THE_FORCE");
    assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&original).unwrap()).err().unwrap().contains("Force"));
    let mut snapshot = e.snapshot(&s); snapshot.as_object_mut().unwrap().remove("forceStateVersion");
    assert!(e.load_state(&snapshot).is_err());
    s.force_state_exact = false;
    assert!(e.available_programs(&s, source, "action").is_empty());
}

#[test]
fn force_cost_leader_unit_attacks_keep_optional_free_effect_and_lethal_stat_change() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] { for code in ["LOF_002", "LOF_003"] {
        let (mut s, source) = start(&e, seat, code);
        s.player_mut(seat).force_held = false;
        s.card_mut(source).deployed = true; e.add_arena(&mut s, source).unwrap();
        let target = unit(&e, &mut s, seat, "LOF_254");
        let base = s.player(seat.other()).base;
        let attack = choose(&e, &s, source);
        let triggered = choose(&e, &attack, base);
        assert_eq!(triggered.decision, Decision::AbilityTarget);
        let done = choose(&e, &triggered, target);
        assert!(!done.player(seat).force_held);
        if code == "LOF_002" {
            assert!(!e.in_arena(&done, target));
            assert!(done.player(seat).discard.contains(target));
        } else { assert!(e.has_keyword(&done, target, "sentinel")); }
        assert!(done.card(base).damage > 0);
    }}
}
