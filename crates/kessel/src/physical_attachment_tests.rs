// Include as a cfg(test) child module of generic_runtime or its tests module.
use super::*;
use serde_json::{json, Value};

fn engine() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
// No currently catalogued ordinary upgrade has the Resistance trait. These
// explicitly synthetic raw-catalog variants exercise the physical query branch
// without claiming an unsupported pilot is a playable ordinary upgrade.
fn synthetic_upgrade_engine(traits: &[&str], title: Option<&str>) -> Engine {
    let mut catalogue: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let entry = catalogue["cards"].as_array_mut().unwrap().iter_mut().find(|e| e["setCode"] == "HMW_235").unwrap();
    entry["card"]["traits"] = json!(traits);
    if let Some(title) = title { entry["card"]["title"] = json!(title); }
    Engine::from_catalogue_value(&catalogue).unwrap()
}
fn add(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card { present: true, def: *e.by_code.get(code).unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards().unwrap().get(code))), owner, ..Card::default() };
    id
}
fn unit(e: &Engine, s: &mut State, owner: Seat, code: &str) -> CardId {
    let id = add(e, s, owner, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, actor: Seat, holdo: bool, resources: usize) -> State {
    let mut s = State::default();
    s.round = 6;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (seat, fallback) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, if holdo && seat == actor { "JTL_007" } else { fallback });
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..resources {
        let id = add(e, &mut s, actor, "LOF_254");
        s.player_mut(actor).resources.push(id).unwrap();
    }
    s.active = actor;
    s.action_player = Some(actor);
    s
}
fn token(e: &Engine, s: &mut State, parent: CardId, code: &str) -> CardId {
    let owner = s.card(parent).owner;
    let id = add(e, s, owner, code);
    if code == "TOKEN_SHIELD" {
        s.card_mut(id).next_token = s.card(parent).shield;
        s.card_mut(parent).shield = id;
    } else {
        assert_eq!(code, "TOKEN_EXPERIENCE");
        s.card_mut(id).next_token = s.card(parent).experience;
        s.card_mut(parent).experience = id;
    }
    e.attach(s, id, parent);
    id
}
fn attached(e: &Engine, s: &mut State, parent: CardId, owner: Seat) -> CardId {
    let id = add(e, s, owner, "HMW_235");
    e.attach(s, id, parent);
    id
}
fn action(e: &Engine, s: &State, label: &str) -> Action {
    e.legal_actions(s).into_iter().find(|a| a.label == label).unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)))
}
fn roundtrip(e: &Engine, s: &State) -> State {
    let snapshot = e.snapshot(s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(restored.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&restored), e.legal_actions(s));
    restored
}

#[test]
fn physical_attachment_queries_include_native_tokens_once_and_ignore_upgrade_owner() {
    let e = synthetic_upgrade_engine(&["resistance"], None);
    for actor in [Seat::P1, Seat::P2] {
        let mut s = board(&e, actor, true, 1);
        let source = s.player(actor).leader;
        let host = unit(&e, &mut s, actor.other(), "LOF_256");
        let ordinary = attached(&e, &mut s, host, actor);
        let shield = token(&e, &mut s, host, "TOKEN_SHIELD");
        let xp = token(&e, &mut s, host, "TOKEN_EXPERIENCE");
        let mut s = roundtrip(&e, &s);
        assert_eq!(e.upgrades(&s, host), vec![ordinary, shield, xp]);
        let query = |name: &str, value: &str| serde_json::from_value::<crate::ability_ir::Expr>(json!({"kind":"query","name":name,"receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":value}]})).unwrap();
        for trait_name in ["resistance", "armor", "learned"] {
            let q = query("hasAttachedTrait", trait_name);
            assert!(q.validate().is_ok());
            assert!(e.eval_with_card(&s, source, 0, host, &q).truth(), "physical trait {trait_name}");
        }
        let shield_query = query("hasAttachedNativeToken", "shield");
        assert!(shield_query.validate().is_ok());
        assert!(e.eval_with_card(&s, source, 0, host, &shield_query).truth());
        assert_eq!(s.card(ordinary).owner, actor);
        assert_eq!(s.card(host).owner, actor.other());
        assert_eq!(s.card(shield).owner, actor.other());
        e.detach(&mut s, shield);
        assert!(!e.eval_with_card(&s, source, 0, host, &shield_query).truth());
        assert!(e.eval_with_card(&s, source, 0, host, &query("hasAttachedTrait", "resistance")).truth());
        e.detach(&mut s, ordinary);
        assert!(!e.eval_with_card(&s, source, 0, host, &query("hasAttachedTrait", "resistance")).truth());
        assert_eq!(e.upgrades(&s, host), vec![xp]);
    }
}

#[test]
fn holdo_front_pays_one_resource_and_exhausts_after_any_owner_selection_then_expires() {
    let e = synthetic_upgrade_engine(&["resistance"], None);
    for actor in [Seat::P1, Seat::P2] {
        let mut s = board(&e, actor, true, 1);
        let source = s.player(actor).leader;
        let intrinsic = unit(&e, &mut s, actor, "SEC_251");
        let enemy_host = unit(&e, &mut s, actor.other(), "LOF_256");
        let upgrade = attached(&e, &mut s, enemy_host, actor);
        let plain = unit(&e, &mut s, actor, "LOF_256");
        token(&e, &mut s, plain, "TOKEN_EXPERIENCE");
        token(&e, &mut s, plain, "TOKEN_SHIELD");
        let s = roundtrip(&e, &s);
        // With fewer than six resources, this is the leader's sole available
        // action and clicking the leader opens its target selector directly.
        let selecting = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert_eq!(selecting.decision, Decision::AbilityTarget);
        let mut targets = e.legal_actions(&selecting).iter().filter_map(Action::id).collect::<Vec<_>>();
        targets.sort();
        assert_eq!(targets, vec![intrinsic, enemy_host]);
        assert!(!selecting.card(source).exhausted);
        assert_eq!(e.ready_resources(&selecting, actor), 1);
        let cancelled = e.apply(&selecting, &action(&e, &selecting, "Cancel")).unwrap();
        assert!(!cancelled.card(source).exhausted);
        assert_eq!(e.ready_resources(&cancelled, actor), 1);
        let selecting = e.apply(&cancelled, &e.card_action(&cancelled, source)).unwrap();
        let power = e.power(&selecting, enemy_host, false);
        let hp = e.hp(&selecting, enemy_host);
        let mut resolved = e.apply(&selecting, &e.card_action(&selecting, enemy_host)).unwrap();
        assert!(resolved.card(source).exhausted);
        assert_eq!(e.ready_resources(&resolved, actor), 0);
        assert_eq!(e.power(&resolved, enemy_host, false), power + 2);
        assert_eq!(e.hp(&resolved, enemy_host), hp + 2);
        assert_eq!(resolved.card(upgrade).owner, actor, "attachment owner differs from its recipient");
        assert_eq!(resolved.card(intrinsic).phase_power, 0);
        resolved = roundtrip(&e, &resolved);
        resolved = e.apply(&resolved, &action(&e, &resolved, "Pass")).unwrap();
        resolved = e.apply(&resolved, &action(&e, &resolved, "Pass")).unwrap();
        assert_eq!(resolved.card(enemy_host).phase_power, 0);
        assert_eq!(resolved.card(enemy_host).phase_hp, 0);
        assert_eq!(e.power(&resolved, enemy_host, false), power);
        assert_eq!(e.hp(&resolved, enemy_host), hp);
    }
}

#[test]
fn holdo_deployed_attack_excludes_self_and_optional_decline_resumes_combat() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        let mut s = board(&e, actor, true, 6);
        let source = s.player(actor).leader;
        let own_resistance = unit(&e, &mut s, actor, "SEC_251");
        let enemy_resistance = unit(&e, &mut s, actor.other(), "SEC_251");
        let plain = unit(&e, &mut s, actor, "LOF_256");
        let s = roundtrip(&e, &s);
        let menu = e.apply(&s, &e.card_action(&s, source)).unwrap();
        let deployed = e.apply(&menu, &action(&e, &menu, "Deploy Admiral Holdo")).unwrap();
        assert!(deployed.card(source).deployed);
        assert_eq!(e.ready_resources(&deployed, actor), 6);
        let deployed = e.apply(&deployed, &action(&e, &deployed, "Pass")).unwrap();
        let target_step = e.apply(&deployed, &e.card_action(&deployed, source)).unwrap();
        assert_eq!(target_step.decision, Decision::AttackTarget);
        let base = target_step.player(actor.other()).base;
        let trigger = e.apply(&target_step, &e.card_action(&target_step, base)).unwrap();
        assert_eq!(trigger.decision, Decision::AbilityTarget);
        let mut targets = e.legal_actions(&trigger).iter().filter_map(Action::id).collect::<Vec<_>>();
        targets.sort();
        assert_eq!(targets, vec![own_resistance, enemy_resistance]);
        assert!(!targets.contains(&source));
        assert!(!targets.contains(&plain));
        let pass = e.legal_actions(&trigger).into_iter().find(|a| a.arg.as_ref().is_some_and(|v| v == "passAbility")).unwrap();
        let declined = e.apply(&trigger, &pass).unwrap();
        assert_eq!(declined.card(own_resistance).phase_power, 0);
        assert_eq!(declined.card(enemy_resistance).phase_hp, 0);
        assert_eq!(declined.card(base).damage, 3);
        assert!(declined.card(source).exhausted);
        assert_eq!(e.ready_resources(&declined, actor), 6);
        let buffed = e.apply(&trigger, &e.card_action(&trigger, enemy_resistance)).unwrap();
        assert_eq!(buffed.card(enemy_resistance).phase_power, 2);
        assert_eq!(buffed.card(enemy_resistance).phase_hp, 2);
        assert_eq!(buffed.card(source).phase_power, 0);
        assert_eq!(buffed.card(base).damage, 3);
        roundtrip(&e, &buffed);
    }
}

#[test]
fn fambaa_paid_play_filters_physical_shields_and_allocates_complete_batch_in_order() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        let mut s = board(&e, actor, false, 10);
        let plain = unit(&e, &mut s, actor, "LOF_256");
        let shielded = unit(&e, &mut s, actor, "LOF_256");
        let old_shield = token(&e, &mut s, shielded, "TOKEN_SHIELD");
        let xp_host = unit(&e, &mut s, actor, "LOF_256");
        let old_xp = token(&e, &mut s, xp_host, "TOKEN_EXPERIENCE");
        let ordinary_host = unit(&e, &mut s, actor, "LOF_256");
        let ordinary = attached(&e, &mut s, ordinary_host, actor.other());
        let token_unit = unit(&e, &mut s, actor, "TOKEN_CLONE_TROOPER");
        let consumed_shielded = unit(&e, &mut s, actor, "SOR_064");
        let consumed = token(&e, &mut s, consumed_shielded, "TOKEN_SHIELD");
        e.damage(&mut s, consumed_shielded, 1);
        assert!(e.has_keyword(&s, consumed_shielded, "shielded"));
        assert_eq!(s.card(consumed_shielded).shield, 0);
        assert_eq!(s.card(consumed).parent, 0);
        // Damage leaves the consumed token in an unreachable internal slot.
        // A public recorded state omits that slot; canonicalize this diagnostic
        // fixture once before checking stable snapshot/hash identity below.
        let mut s = e.load_state(&e.snapshot(&s)).unwrap();
        let space = unit(&e, &mut s, actor, "ASH_201");
        let enemy = unit(&e, &mut s, actor.other(), "LOF_256");
        let source = add(&e, &mut s, actor, "HMW_080");
        s.player_mut(actor).hand.push(source).unwrap();
        let s = roundtrip(&e, &s);
        let first_token = s.next_id;
        let resources = e.ready_resources(&s, actor);
        let cost = usize::from(e.cost(&s, source));
        assert!(cost > 0 && cost <= resources);
        let action = e.card_action(&s, source);
        assert!(e.legal_actions(&s).contains(&action));
        let played = e.apply(&s, &action).unwrap();
        assert!(played.card(source).exhausted);
        assert!(played.player(actor).ground.contains(source));
        assert_eq!(e.ready_resources(&played, actor), resources - cost);
        let recipients = [plain, xp_host, ordinary_host, token_unit, consumed_shielded, source];
        assert_eq!(played.next_id, first_token + recipients.len() as CardId);
        for (offset, recipient) in recipients.into_iter().enumerate() {
            let id = first_token + offset as CardId;
            assert_eq!(played.card(recipient).shield, id, "recipient order maps to allocation order");
            assert_eq!(e.token_count(&played, played.card(recipient).shield), 1);
            assert_eq!(played.card(id).parent, recipient);
            assert_eq!(played.card(id).owner, actor);
            assert_eq!(e.def(&played, id).code, "TOKEN_SHIELD");
            assert_eq!(e.upgrades(&played, recipient).last(), Some(&id));
        }
        assert_eq!(played.card(shielded).shield, old_shield);
        assert_eq!(e.upgrades(&played, shielded), vec![old_shield]);
        assert_eq!(played.card(xp_host).experience, old_xp);
        assert_eq!(e.upgrades(&played, xp_host), vec![old_xp, first_token + 1]);
        assert_eq!(e.upgrades(&played, ordinary_host), vec![ordinary, first_token + 2]);
        assert_eq!(played.card(ordinary).owner, actor.other());
        assert_ne!(played.card(consumed_shielded).shield, 0, "printed Shielded with its native Shield consumed still receives a new token");
        assert_eq!(played.card(space).shield, 0);
        assert_eq!(played.card(enemy).shield, 0);
        roundtrip(&e, &played);
    }
}

#[test]
fn fambaa_ordinary_upgrade_named_shield_does_not_count_as_native_shield() {
    let e = synthetic_upgrade_engine(&["armor"], Some("Shield"));
    let actor = Seat::P1;
    let mut s = board(&e, actor, false, 10);
    let host = unit(&e, &mut s, actor, "LOF_256");
    let ordinary = attached(&e, &mut s, host, actor.other());
    let source = add(&e, &mut s, actor, "HMW_080");
    s.player_mut(actor).hand.push(source).unwrap();
    let s = roundtrip(&e, &s);
    let next = s.next_id;
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(played.card(host).shield, next);
    assert_eq!(e.upgrades(&played, host), vec![ordinary, next]);
    assert_eq!(played.card(source).shield, next + 1);
    assert_eq!(played.next_id, next + 2);
    roundtrip(&e, &played);
}

#[test]
fn physical_attachment_leaf_arguments_and_fambaa_metadata_fail_closed() {
    let query = json!({"kind":"query","name":"hasAttachedNativeToken","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"shield"}]});
    for mutation in ["player", "nested-card", "empty-args", "many-args", "numeric-arg", "other-token", "array-receiver"] {
        let mut bad = query.clone();
        match mutation {
            "player" => bad["receiver"]["path"] = json!(["player"]),
            "nested-card" => bad["receiver"]["path"] = json!(["card", "parentCard"]),
            "empty-args" => bad["args"] = json!([]),
            "many-args" => bad["args"] = json!([{"kind":"const","value":"shield"},{"kind":"const","value":"shield"}]),
            "numeric-arg" => bad["args"][0]["value"] = json!(1),
            "other-token" => bad["args"][0]["value"] = json!("experience"),
            _ => bad["receiver"] = json!({"kind":"array","items":[]}),
        }
        let expression: crate::ability_ir::Expr = serde_json::from_value(bad).unwrap();
        assert!(expression.validate().is_err(), "{mutation}");
    }
    for value in [String::new(), "x".repeat(65)] {
        let mut bad = query.clone();
        bad["name"] = json!("hasAttachedTrait");
        bad["args"][0]["value"] = json!(value);
        let expression: crate::ability_ir::Expr = serde_json::from_value(bad).unwrap();
        assert!(expression.validate().is_err());
    }
    let mut extra = query.clone();
    extra["includeHand"] = json!(true);
    assert!(serde_json::from_value::<crate::ability_ir::Expr>(extra).is_err());

    let original: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    for mutation in ["event-class", "mixed-type", "unique", "keyword", "intrinsic-keyword", "extra-side", "pilot", "attach", "optional", "cost", "onAttack", "two-shields", "experience", "opponent", "controller-order", "extra-registration"] {
        let mut catalogue = original.clone();
        let entry = catalogue["cards"].as_array_mut().unwrap().iter_mut().find(|e| e["setCode"] == "HMW_080").unwrap();
        match mutation {
            "event-class" => entry["baseClass"] = json!("EventCard"),
            "mixed-type" => entry["card"]["types"] = json!(["unit","leader"]),
            "unique" => entry["card"]["unique"] = json!(true),
            "keyword" => entry["keywordValues"] = json!({"sentinel":1}),
            "intrinsic-keyword" => entry["card"]["keywords"] = json!(["sentinel"]),
            "extra-side" => entry["abilities"]["unit"] = entry["abilities"]["card"].clone(),
            "pilot" => entry["piloting"] = json!({"cost":1}),
            "attach" => entry["attachCondition"] = json!({"kind":"const","value":true}),
            "optional" => entry["abilities"]["card"][0]["optional"] = json!(true),
            "cost" => entry["abilities"]["card"][0]["costs"] = json!([{"kind":"exhaustSelf"}]),
            "onAttack" => entry["abilities"]["card"][0]["trigger"] = json!("onAttack"),
            "two-shields" => entry["abilities"]["card"][0]["effect"]["amount"]["value"] = json!(2),
            "experience" => entry["abilities"]["card"][0]["effect"]["kind"] = json!("giveExperience"),
            "opponent" => entry["abilities"]["card"][0]["effect"]["target"]["receiver"]["path"] = json!(["opponent"]),
            "controller-order" => entry["abilities"]["card"][0]["effect"]["target"]["controllerOrder"] = json!("seatOrder"),
            _ => { let duplicate = entry["abilities"]["card"][0].clone(); entry["abilities"]["card"].as_array_mut().unwrap().push(duplicate); }
        }
        let e = Engine::from_catalogue_value(&catalogue).unwrap();
        assert!(!e.by_code.contains_key("HMW_080"), "{mutation}");
    }
}

#[test]
fn holdo_auto_interpreter_and_aot_match_physical_upgrade_target_scenario() {
    let mut observations = vec![];
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter, RulesBackend::Aot] {
        let mut e = synthetic_upgrade_engine(&["resistance"], None);
        // Only these focused programs execute; unrelated catalog fallbacks do
        // not prevent forcing the AOT backend for the two Holdo programs.
        for definition in &mut e.definitions { if definition.code != "JTL_007" { definition.programs = Default::default(); } }
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e, Seat::P2, true, 1);
        let source = s.player(Seat::P2).leader;
        let host = unit(&e, &mut s, Seat::P1, "LOF_256");
        attached(&e, &mut s, host, Seat::P2);
        unit(&e, &mut s, Seat::P2, "LOF_256");
        let s = roundtrip(&e, &s);
        let choosing = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert_eq!(choosing.decision, Decision::AbilityTarget);
        let actions = e.legal_actions(&choosing);
        let done = e.apply(&choosing, &e.card_action(&choosing, host)).unwrap();
        assert_eq!(done.card(host).phase_power, 2);
        assert_eq!(done.card(host).phase_hp, 2);
        observations.push((e.snapshot(&choosing), actions, e.snapshot(&done), done.zobrist()));
    }
    assert_eq!(observations[0], observations[1]);
    assert_eq!(observations[1], observations[2]);
}

#[test]
fn fambaa_auto_and_interpreter_match_and_aot_remains_an_explicit_fallback() {
    let mut observations = vec![];
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let mut e = engine();
        for definition in &mut e.definitions { if definition.code != "HMW_080" { definition.programs = Default::default(); } }
        e.set_rules_backend(backend).unwrap();
        let mut s = board(&e, Seat::P2, false, 10);
        let host = unit(&e, &mut s, Seat::P2, "LOF_256");
        token(&e, &mut s, host, "TOKEN_EXPERIENCE");
        let shielded = unit(&e, &mut s, Seat::P2, "LOF_256");
        token(&e, &mut s, shielded, "TOKEN_SHIELD");
        let source = add(&e, &mut s, Seat::P2, "HMW_080");
        s.player_mut(Seat::P2).hand.push(source).unwrap();
        let s = roundtrip(&e, &s);
        let done = e.apply(&s, &e.card_action(&s, source)).unwrap();
        assert_ne!(done.card(host).shield, 0);
        assert_ne!(done.card(source).shield, 0);
        observations.push((e.snapshot(&done), e.legal_actions(&done), done.zobrist()));
        assert!(e.set_rules_backend(RulesBackend::Aot).unwrap_err().contains("requires interpreter"));
    }
    assert_eq!(observations[0], observations[1]);
}
