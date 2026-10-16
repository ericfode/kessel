// Source-derived Rust midgame witnesses; no source-engine execution.
use super::*;
use crate::ability_ir::{selector_contract::CardTypeFilter, Ability, Program};
fn native(types: Value, zones: Value, controller: &str) -> Value {
    json!({"kind":"cardSelection","mode":"single","chooser":"self","resultBinding":"target","defaults":"pinnedTargetResolver","contextBinding":"target","choosingPlayerOverride":"contextOverrideFirst","delayOpponentBeforeCosts":true,"cardTypes":types,"zones":zones,"controller":{"kind":"const","value":controller}})
}
fn constant(value: &str) -> Value {
    json!({"kind":"const","value":value})
}
fn source_entry(code: &str) -> Value {
    let c: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    c["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["setCode"] == code)
        .unwrap()
        .clone()
}
fn canonical(e: &Engine, s: &State) -> State {
    let snap = e.snapshot(s);
    let loaded = e.load_state(&snap).unwrap();
    assert_eq!(e.snapshot(&loaded), snap);
    assert_eq!(loaded.zobrist(), s.zobrist());
    loaded
}
fn seat_board(e: &Engine, seat: Seat) -> State {
    let mut s = board(e);
    s.round = 6;
    s.active = seat;
    s.action_player = Some(seat);
    s
}
fn in_hand(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    for _ in 0..15 {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    id
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let loaded = if s.is_action_window() {
        canonical(e, s)
    } else {
        // Canonical import is intentionally action-window-only. Prompt frames
        // are fixed-size State copies; do not invent an external frame format.
        assert!(e.load_state(&e.snapshot(s)).is_err());
        let copied = *s;
        assert_eq!(copied.zobrist(), s.zobrist());
        copied
    };
    assert_eq!(e.legal_actions(&loaded), e.legal_actions(s));
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id} absent from {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn ids(e: &Engine, s: &State) -> Vec<CardId> {
    e.legal_actions(s).iter().filter_map(Action::id).collect()
}
#[test]
fn selector_type_filters_match_concrete_types_and_unions_without_wildcard_substitution() {
    use CardTypeFilter::*;
    for (filter, actual, expected) in [
        (Unit, TokenUnit, true),
        (Unit, NonTokenLeaderUnit, true),
        (Unit, Leader, false),
        (BasicUnit, TokenUnit, false),
        (BasicUnit, BasicUnit, true),
        (NonLeaderUnit, NonTokenLeaderUnit, false),
        (NonTokenUnit, NonTokenLeaderUnit, true),
        (NonTokenUnit, TokenUnit, false),
        (Upgrade, LeaderUpgrade, true),
        (NonLeaderUpgrade, LeaderUpgrade, false),
        (UnitUpgrade, BasicUpgrade, false),
        (UnitUpgrade, NonLeaderUnitUpgrade, true),
        (Token, TokenCard, true),
        (Playable, TokenUnit, false),
        (Playable, BasicUnit, true),
        (NonUnit, Base, true),
        (NonUnit, NonTokenLeaderUnit, false),
    ] {
        assert_eq!(filter.matches(actual), expected, "{filter:?}/{actual:?}");
    }
    let e = engine();
    let mut s = seat_board(&e, Seat::P2);
    let source = unit(&e, &mut s, Seat::P2, "LOF_256");
    let token = unit(&e, &mut s, Seat::P2, "TOKEN_CLONE_TROOPER");
    let enemy = unit(&e, &mut s, Seat::P1, "LOF_254");
    let r: TargetResolver = serde_json::from_value(native(
        json!({"kind":"array","items":[constant("basicUnit"),constant("tokenUnit")]}),
        constant("anyArena"),
        "any",
    ))
    .unwrap();
    r.validate().unwrap();
    assert_eq!(
        e.matching_targets(&s, source, &r),
        vec![source, token, enemy],
        "context player partitions first even in P2"
    );
    let bases: TargetResolver = serde_json::from_value(native(
        json!({"kind":"array","items":[constant("unit"),constant("base")]}),
        constant("anyAttackable"),
        "self",
    ))
    .unwrap();
    assert_eq!(
        e.matching_targets(&s, source, &bases),
        vec![source, token, s.players[1].base]
    );
    let basic: TargetResolver =
        serde_json::from_value(json!({"type":"basicUnit","owner":"any","zone":"arena"})).unwrap();
    basic.validate().unwrap();
    assert_eq!(e.matching_targets(&s, source, &basic), vec![enemy, source]);
}
#[test]
fn selector_native_zone_owner_and_candidate_context_are_separate_and_raw_round_trip() {
    let e = engine();
    let mut s = seat_board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "LOF_256");
    let own = unit(&e, &mut s, Seat::P1, "LOF_256");
    let enemy = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(own).damage = 1;
    s.card_mut(enemy).damage = 1;
    let mut raw = native(constant("unit"), constant("anyArena"), "any");
    raw["predicate"] = json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["selectionCandidate"]},"right":{"kind":"ref","path":["source"]}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["selectionCandidate","damage"]},"right":{"kind":"ref","path":["target","damage"]}}});
    let r: TargetResolver = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(&r).unwrap(), raw);
    assert_eq!(e.matching_targets(&s, source, &r), vec![own, enemy]);
    raw["controller"] = constant("opponent");
    let r: TargetResolver = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(e.matching_targets(&s, source, &r), vec![enemy]);
    raw["zones"] = constant("spaceArena");
    let r: TargetResolver = serde_json::from_value(raw).unwrap();
    assert!(e.matching_targets(&s, source, &r).is_empty());
}
#[test]
fn selector_native_unsupported_fields_scopes_modes_and_type_zone_pairs_fail_closed() {
    let raw = native(constant("unit"), constant("anyAttackable"), "any");
    for (field, value) in [
        ("cardTypes", constant("mysteryType")),
        ("cardTypes", json!({"kind":"array","items":[]})),
        ("cardTypes", json!({"kind":"ref","path":["target"]})),
        ("zones", constant("hand")),
        ("zones", constant("capture")),
        ("controller", constant("owner")),
        ("mode", json!("exactlyVariable")),
        ("numCardsFunc", json!({"kind":"const","value":2})),
        (
            "multiSelectCardCondition",
            json!({"kind":"const","value":true}),
        ),
        ("condition", json!({"kind":"const","value":false})),
        ("mustChangeGameState", json!("none")),
        ("delayOpponentBeforeCosts", json!(false)),
        ("defaults", json!("anything")),
        ("contextBinding", json!("outerTarget")),
        ("choosingPlayerOverride", json!("ignore")),
        ("unknown", json!(true)),
    ] {
        let mut bad = raw.clone();
        bad[field] = value;
        assert!(
            serde_json::from_value::<TargetResolver>(bad).is_err(),
            "accepted {field}"
        );
    }
    for (ty, zone) in [
        ("leader", "anyArena"),
        ("tokenUnit", "discard"),
        ("base", "discard"),
        ("any", "anyAttackable"),
        ("upgrade", "anyArena"),
    ] {
        assert!(
            serde_json::from_value::<TargetResolver>(native(constant(ty), constant(zone), "any"))
                .is_err(),
            "{ty}/{zone}"
        );
    }
    let mut bad = raw.clone();
    bad["predicate"] = json!({"kind":"ref","path":["card"]});
    assert!(serde_json::from_value::<TargetResolver>(bad).is_err());
    let mut bad = raw;
    bad["predicate"] = json!({"kind":"query","name":"hasSomeArenaUnit","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"binary","op":"===","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["selectionCandidate"]}}]});
    assert!(serde_json::from_value::<TargetResolver>(bad).is_err());
    let mut bad = native(constant("event"), constant("discard"), "any");
    bad["predicate"] = json!({"kind":"query","name":"getPower","receiver":{"kind":"ref","path":["selectionCandidate"]},"args":[]});
    assert!(serde_json::from_value::<TargetResolver>(bad).is_err());
    let mut bad = native(constant("unit"), constant("anyArena"), "any");
    bad["predicate"] = json!({"kind":"arenaSome","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"const","value":true}},"predicate":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["selectionCandidate"]}}});
    assert!(serde_json::from_value::<TargetResolver>(bad).is_err());
    let legacy: TargetResolver = serde_json::from_value(json!({"type":"unit"})).unwrap();
    assert_eq!(
        serde_json::to_value(&legacy).unwrap(),
        json!({"owner":"any","type":"unit","arena":null,"zone":null,"optional":false,"predicate":null,"mode":null,"count":null})
    );
    for raw in [
        json!({"type":"mystery","zone":"arena"}),
        json!({"type":"base","zone":"discard"}),
        json!({"type":"unit","zone":"base"}),
        json!({"type":"token","zone":"anyAttackable"}),
        json!({"type":"unit","owner":"owner"}),
        json!({"type":"unit","zone":"capture"}),
    ] {
        let r: TargetResolver = serde_json::from_value(raw).unwrap();
        assert!(r.validate().is_err());
    }
}
#[test]
fn selector_opponent_event_pays_before_choice_preserves_owner_and_resumes_initiator() {
    for seat in [Seat::P1, Seat::P2] {
        for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = seat_board(&e, seat);
            let own = unit(&e, &mut s, seat, "LOF_256");
            let chosen = unit(&e, &mut s, seat.other(), "LOF_254");
            let alternate = unit(&e, &mut s, seat.other(), "LOF_256");
            let card = in_hand(&e, &mut s, seat, "SOR_041");
            let s = canonical(&e, &s);
            let ready = e.ready_resources(&s, seat);
            let cost = e.cost(&s, card);
            let prompt = click(&e, &s, card);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert_eq!(prompt.active, seat.other());
            assert_eq!(prompt.source, card);
            assert!(prompt.player(seat).discard.contains(card));
            assert_eq!(e.ready_resources(&prompt, seat), ready - cost as usize);
            assert_eq!(ids(&e, &prompt), vec![chosen, alternate]);
            assert!(!ids(&e, &prompt).contains(&own));
            assert!(!e.legal_actions(&prompt).iter().any(|a| a.label == "Cancel"));
            let done = click(&e, &prompt, chosen);
            assert_eq!(done.active, seat.other());
            assert_eq!(done.decision, Decision::Window);
            assert!(done.player(seat.other()).discard.contains(chosen));
            assert!(e.in_arena(&done, alternate));
            assert!(e.in_arena(&done, own));
            canonical(&e, &done);
        }
    }
}
#[test]
fn selector_opponent_entry_and_attack_defeat_exclude_deployed_leaders_and_no_target_is_safe() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = seat_board(&e, seat);
        let target = unit(&e, &mut s, seat.other(), "LOF_254");
        let leader = s.player(seat.other()).leader;
        s.card_mut(leader).deployed = true;
        s.card_mut(leader).epic_used = true;
        e.add_arena(&mut s, leader).unwrap();
        let source = in_hand(&e, &mut s, seat, "SOR_040");
        let s = canonical(&e, &s);
        let entered = click(&e, &s, source);
        assert_eq!(entered.decision, Decision::AbilityTarget);
        assert_eq!(entered.active, seat.other());
        assert_eq!(ids(&e, &entered), vec![target]);
        let done = click(&e, &entered, target);
        assert!(e.in_arena(&done, leader));
        assert!(done.card(source).exhausted);
        assert_eq!(done.active, seat.other());
        let mut attack = done;
        attack.active = seat;
        attack.action_player = Some(seat);
        attack.card_mut(source).exhausted = false;
        let victim = unit(&e, &mut attack, seat.other(), "LOF_256");
        let attack = canonical(&e, &attack);
        let declared = click(&e, &attack, source);
        let triggered = click(&e, &declared, attack.player(seat.other()).base);
        assert_eq!(triggered.decision, Decision::AbilityTarget);
        assert_eq!(triggered.active, seat.other());
        assert_eq!(ids(&e, &triggered), vec![victim]);
        let resolved = click(&e, &triggered, victim);
        assert!(!e.in_arena(&resolved, victim));
        assert!(e.in_arena(&resolved, leader));
        assert!(resolved.card(resolved.player(seat.other()).base).damage > 0);
        let mut empty = seat_board(&e, seat);
        let event = in_hand(&e, &mut empty, seat, "SOR_041");
        let empty = canonical(&e, &empty);
        let prompt = click(&e, &empty, event);
        assert_eq!(prompt.decision, Decision::AbilityNoEffect);
        assert!(prompt.player(seat).hand.contains(event));
        let done = named_button(&e, &prompt, "Play anyway");
        assert_eq!(done.decision, Decision::Window);
        assert!(done.player(seat).discard.contains(event));
    }
}
#[test]
fn selector_opponent_damage_and_weakness_keep_the_source_and_recipient_controllers() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = seat_board(&e, seat);
        let attacker = unit(&e, &mut s, seat, "TS26_066");
        let own = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        let s = canonical(&e, &s);
        let started = click(&e, &s, attacker);
        let prompt = click(&e, &started, s.player(seat.other()).base);
        assert_eq!(prompt.active, seat.other());
        assert_eq!(ids(&e, &prompt), vec![own, attacker, enemy]);
        let done = click(&e, &prompt, own);
        assert_eq!(done.card(own).damage, 1);
        assert_eq!(done.card(enemy).damage, 0);
        assert_eq!(done.active, seat.other());
        let mut s = seat_board(&e, seat);
        let victim = unit(&e, &mut s, seat.other(), "LOF_256");
        let wrong = unit(&e, &mut s, seat, "LOF_256");
        let cid = in_hand(&e, &mut s, seat, "HMW_197");
        let s = canonical(&e, &s);
        let hp = e.hp(&s, victim);
        let prompt = click(&e, &s, cid);
        assert_eq!(prompt.active, seat.other());
        assert_eq!(ids(&e, &prompt), vec![victim]);
        let done = click(&e, &prompt, victim);
        assert_eq!(e.hp(&done, victim), hp - 1);
        assert_eq!(e.hp(&done, wrong), e.hp(&s, wrong));
        let tokens = e.upgrades(&done, victim);
        assert_eq!(tokens.len(), 1);
        assert_eq!(done.card(tokens[0]).owner, seat.other());
        canonical(&e, &done);
    }
}
#[test]
fn selector_front_leader_ready_and_discard_event_return_are_canonical() {
    for backend in [
        RulesBackend::Auto,
        RulesBackend::Interpreter,
        RulesBackend::Aot,
    ] {
        let mut e = engine();
        // Forced AOT checks this program in isolation, like the existing direct
        // AOT tests; unrelated admitted programs retain interpreter fallbacks.
        if backend == RulesBackend::Aot {
            for d in &mut e.definitions {
                if d.code != "SEC_188" {
                    d.programs = Default::default();
                }
            }
        }
        e.set_rules_instrumentation(true);
        e.set_rules_backend(backend).unwrap();
        let mut s = seat_board(&e, Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "SEC_188");
        let leader = s.players[0].leader;
        s.card_mut(leader).exhausted = true;
        let other = s.players[1].leader;
        s.card_mut(other).deployed = true;
        s.card_mut(other).epic_used = true;
        e.add_arena(&mut s, other).unwrap();
        let s = canonical(&e, &s);
        let started = click(&e, &s, attacker);
        let prompt = click(&e, &started, s.players[1].base);
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        let declined = named_button(&e, &prompt, "Pass");
        assert!(declined.card(leader).exhausted);
        assert_eq!(ids(&e, &prompt), vec![leader]);
        let done = click(&e, &prompt, leader);
        assert!(!done.card(leader).exhausted);
        assert!(done.card(attacker).exhausted);
        assert!(e.in_arena(&done, other));
        canonical(&e, &done);
        if backend == RulesBackend::Aot {
            let index = e.def(&done, attacker).programs.card[0] as usize;
            let hits = e.rules_hits.lock().unwrap()[&index];
            assert!(hits[0] > 0);
            assert_eq!(hits[1], 0);
        }
    }
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = seat_board(&e, seat);
        let own = add(&e, &mut s, seat, "SOR_041");
        s.player_mut(seat).discard.push(own).unwrap();
        let opposing = add(&e, &mut s, seat.other(), "SOR_073");
        s.player_mut(seat.other()).discard.push(opposing).unwrap();
        let wrong = add(&e, &mut s, seat, "LOF_254");
        s.player_mut(seat).discard.push(wrong).unwrap();
        let source = in_hand(&e, &mut s, seat, "SOR_183");
        let s = canonical(&e, &s);
        let entered = click(&e, &s, source);
        assert_eq!(entered.decision, Decision::TriggerOrder);
        let prompt = named_button(&e, &entered, "Return an event from a discard pile");
        assert_eq!(prompt.decision, Decision::AbilityTarget);
        let declined = named_button(&e, &prompt, "Pass");
        assert!(declined.player(seat.other()).discard.contains(opposing));
        assert!(declined.card(source).exhausted);
        assert_eq!(
            ids(&e, &prompt),
            if seat == Seat::P1 {
                vec![own, opposing]
            } else {
                vec![opposing, own]
            }
        );
        let done = click(&e, &prompt, opposing);
        assert!(done.player(seat.other()).hand.contains(opposing));
        assert!(done.player(seat).discard.contains(own));
        assert!(done.player(seat).discard.contains(wrong));
        assert!(done.card(source).exhausted);
        canonical(&e, &done);
    }
}
#[test]
fn selector_complete_cards_are_admitted_and_compound_contracts_still_reject() {
    let e = engine();
    for code in [
        "SOR_041", "SOR_040", "TS26_066", "HMW_197", "SEC_188", "SOR_183",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in [
        "IBH_001", "IBH_005", "JTL_173", "HMW_218", "JTL_233", "TWI_153", "SEC_196", "LOF_145",
    ] {
        assert!(!e.by_code.contains_key(code), "partial admission {code}");
    }
    let entry = source_entry("SOR_041");
    let a: Ability = serde_json::from_value(entry["abilities"]["card"][0].clone()).unwrap();
    let p = Program::compile(&a).unwrap();
    assert!(
        matches!(&p.instructions[0],Instruction::NamedChooserGroup{choosers,..} if choosers.len()==1)
    );
    let mut bad = entry["abilities"]["card"][0].clone();
    bad["then"] = json!({"title":"unreviewed continuation","effect":{"kind":"noAction"}});
    assert!(Program::compile(&serde_json::from_value::<Ability>(bad).unwrap()).is_err());
    let mut bad = entry["abilities"]["card"][0].clone();
    let r = bad["targetResolver"].take();
    bad.as_object_mut().unwrap().remove("targetResolver");
    bad["effect"] =
        json!({"kind":"select","targetResolver":r,"effect":{"kind":"defeat","target":"chosen"}});
    assert!(Program::compile(&serde_json::from_value::<Ability>(bad).unwrap()).is_err());
}

#[test]
fn selector_expanded_source_metadata_and_selected_pool_token_dependencies_fail_closed() {
    let original: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    for code in ["SEC_188", "SOR_183"] {
        for (field, value) in [
            ("baseClass", json!("LeaderUnitCard")),
            ("card.types", json!(["unit", "leader"])),
            ("privatePrograms", json!([{"kind":"unknownProgram"}])),
            ("classMethods", json!([{"kind":"unknownOverride"}])),
        ] {
            let mut catalog = original.clone();
            let entry = catalog["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["setCode"] == code)
                .unwrap();
            if field == "card.types" {
                entry["card"]["types"] = value;
            } else {
                entry[field] = value;
            }
            let e = Engine::from_catalogue_value(&catalog).unwrap();
            assert!(!e.by_code.contains_key(code), "{code}/{field}");
            assert!(e.rejected_cards().unwrap()[code].contains("expanded selectors"));
        }
    }
    let e = engine();
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"].as_array_mut().unwrap().retain(|c| {
        ["ASH_019", "SOR_014", "IBH_053", "TOKEN_SHIELD", "HMW_197"]
            .contains(&c["setCode"].as_str().unwrap())
    });
    assert!(Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))
    )
    .err()
    .unwrap()
    .contains("GiveWeakness cards must include"));
    let mut bad = source_entry("HMW_197")["abilities"]["card"][0].clone();
    bad["effect"] = json!({"kind":"simultaneous","effects":[bad["effect"].clone(),{"kind":"damage","amount":{"kind":"const","value":1},"target":"chosen"}]});
    assert!(Program::compile(&serde_json::from_value::<Ability>(bad).unwrap()).is_err());
}

#[test]
fn selector_literal_zone_capabilities_keep_seat_zone_and_repeated_inventory_order() {
    let e = engine();
    let space_code = e.definitions.iter().find(|d| d.info.is("unit") && !d.info.is("leader") && d.info.arena.as_deref() == Some("space")).unwrap().code.clone();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = seat_board(&e, seat);
        let source = unit(&e, &mut s, seat, "LOF_256");
        let own_space = unit(&e, &mut s, seat, &space_code);
        let opposing_ground = unit(&e, &mut s, seat.other(), "LOF_256");
        let opposing_space = unit(&e, &mut s, seat.other(), &space_code);
        let own_event = add(&e, &mut s, seat, "SOR_041");
        let opposing_event = add(&e, &mut s, seat.other(), "SOR_041");
        s.player_mut(seat).discard.push(own_event).unwrap();
        s.player_mut(seat.other()).discard.push(opposing_event).unwrap();
        let mut raw = native(
            json!({"kind":"array","items":[constant("unit"),constant("base"),constant("event"),constant("leader")]}),
            json!({"kind":"array","items":[constant("spaceArena"),constant("groundArena"),constant("spaceArena"),constant("base"),constant("discard")]}),
            "any",
        );
        raw["chooser"] = json!("opponent");
        let resolver: TargetResolver = serde_json::from_value(raw).unwrap();
        assert_eq!(e.matching_targets(&s, source, &resolver), vec![
            own_space, source, own_space, s.player(seat).base, s.player(seat).leader, own_event,
            opposing_space, opposing_ground, opposing_space, s.player(seat.other()).base, s.player(seat.other()).leader, opposing_event,
        ]);
    }
}
