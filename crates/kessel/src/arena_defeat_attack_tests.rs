//! Expectations come from the actual pinned ASH_083 plays retained in the
//! source design and mirrored by the reference fixture. These tests exercise
//! shared Rust execution; engine/native/Wasm differential evidence is separate.
use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_083")
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
            .unwrap_or_else(|| panic!("{code}: {:?}", e.rejected_cards())),
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
    s.credit_state_exact = e.needs_credit_state();
    s.active = seat;
    s.action_player = Some(seat);
    s.round = 7;
    s.hidden_history_exact = true;
    s.force_state_exact = e.needs_force_state();
    s.arena_order_exact = true;
    for p in [seat, seat.other()] {
        let b = add(e, &mut s, p, "ASH_019");
        s.player_mut(p).base = b;
        let l = add(e, &mut s, p, "SOR_014");
        s.player_mut(p).leader = l;
        s.player_mut(p).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..20 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    s
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn button(e: &Engine, s: &State, label: &str) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.label == label)
        .unwrap_or_else(|| panic!("{label}: {:?}", e.legal_actions(s)));
    let mut ingress = *s;
    e.set_actor(&mut ingress, s.active).unwrap();
    assert_eq!(ingress, *s, "resolver ingress must not mutate state");
    e.apply(&ingress, &a).unwrap()
}
fn play(e: &Engine, s: &State) -> State {
    let mut s = *s;
    let actor = s.active;
    let id = add(e, &mut s, actor, "ASH_083");
    s.player_mut(actor).hand.push(id).unwrap();
    choose(e, &s, id)
}
fn admits(v: Value) -> bool {
    crate::ability_ir::Catalogue::load(&json!({"schema":"swu-card-ir/v1","cards":[v]}))
        .unwrap()
        .cards
        .contains_key("ASH_083")
}

fn attack(e: &Engine, s: &State, source: CardId, target: CardId) -> State {
    let s = choose(e, s, source);
    assert_eq!(s.decision, Decision::AttackTarget);
    choose(e, &s, target)
}
fn settle(e: &Engine, mut s: State) -> State {
    for _ in 0..20 {
        if matches!(s.decision, Decision::Window | Decision::Terminal) {
            return s;
        }
        let a = e
            .legal_actions(&s)
            .into_iter()
            .find(|a| a.label == "Resolve all")
            .unwrap_or_else(|| e.legal_actions(&s)[0].clone());
        s = e.apply(&s, &a).unwrap();
    }
    panic!("unsettled {:?}", s.decision)
}
#[test]
fn summa_excludes_only_physical_source_and_defeats_both_players_space_copies() {
    let e = engine();
    assert!(
        e.by_code.contains_key("ASH_083"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_083")
    );
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "ASH_083");
        let own = unit(&e, &mut s, seat, "ASH_083");
        let enemy = unit(&e, &mut s, seat.other(), "ASH_083");
        let own_ground = unit(&e, &mut s, seat, "LOF_254");
        let enemy_ground = unit(&e, &mut s, seat.other(), "LOF_254");
        s.card_mut(source).exhausted = false;
        let base = s.player(seat.other()).base;
        let done = settle(&e, attack(&e, &s, source, enemy));
        assert!(e.in_arena(&done, source));
        assert_eq!(done.card(source).damage, 0);
        for id in [own, enemy] {
            assert!(!e.in_arena(&done, id));
            assert!(done.player(s.card(id).owner).discard.contains(id));
        }
        for id in [own_ground, enemy_ground] {
            assert!(e.in_arena(&done, id));
        }
        assert_eq!(done.card(base).damage, 0);
        assert_eq!(done.attack_source, 0);
        assert_eq!(done.attack_defender, 0);
        assert_eq!(done.death_depth, 0);
    }
}
#[test]
fn summa_captures_global_entry_and_reverse_attachment_cleanup_order() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "ASH_083");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_192");
        let own = unit(&e, &mut s, seat, "LOF_192");
        let enemy2 = unit(&e, &mut s, seat.other(), "LOF_192");
        let u1 = add(&e, &mut s, seat.other(), "HMW_235");
        e.attach(&mut s, u1, enemy);
        let u2 = add(&e, &mut s, seat, "HMW_235");
        e.attach(&mut s, u2, own);
        let u3 = add(&e, &mut s, seat.other(), "HMW_235");
        e.attach(&mut s, u3, enemy2);
        e.create_token(&mut s, enemy, false).unwrap();
        s.card_mut(source).exhausted = false;
        let done = settle(&e, attack(&e, &s, source, enemy));
        assert_eq!(
            done.player(seat.other()).discard.as_slice(),
            &[u3, u1, enemy, enemy2]
        );
        assert_eq!(done.player(seat).discard.as_slice(), &[u2, own]);
        assert_eq!(done.card(source).damage, 0);
        assert_eq!(done.card(enemy).shield, 0);
        assert_eq!(done.death_depth, 0);
    }
}
#[test]
fn captured_summa_ability_retains_departed_physical_source() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_083");
    let copy = unit(&e, &mut s, Seat::P1, "ASH_083");
    let enemy = unit(&e, &mut s, Seat::P2, "LOF_192");
    let p = *e.card_programs(&s, source).first().unwrap();
    let effect = e
        .program(p)
        .instructions
        .iter()
        .find_map(|i| {
            if let Instruction::Apply(effect) = i {
                Some(effect.clone())
            } else {
                None
            }
        })
        .unwrap();
    e.defeat(&mut s, source).unwrap();
    s.source = source;
    s.ability_program = p + 1;
    e.apply_effect(&mut s, &effect).unwrap();
    for id in [copy, enemy] {
        assert!(!e.in_arena(&s, id));
    }
    assert!(s.player(Seat::P1).discard.contains(source));
    assert_eq!(s.source, source);
}
#[test]
fn summa_attack_requires_global_order_at_import_and_execution() {
    let mut c = catalogue();
    let wanted = [
        "ASH_083",
        "ASH_019",
        "SOR_014",
        "LOF_192",
        "LOF_254",
        "TOKEN_SHIELD",
        "TOKEN_EXPERIENCE",
    ];
    c["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| wanted.iter().any(|id| c["setCode"] == *id));
    let e = Engine::from_catalogue_value(&c).unwrap();
    let mut s = board(&e, Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_083");
    let enemy = unit(&e, &mut s, Seat::P2, "LOF_192");
    let good = e.snapshot(&s);
    assert!(e.load_state(&good).is_ok());
    let mut bad = good;
    bad.as_object_mut().unwrap().remove("arenaUnitOrder");
    assert!(e.load_state(&bad).unwrap_err().contains("arenaUnitOrder"));
    s.arena_order_exact = false;
    let p = *e.card_programs(&s, source).first().unwrap();
    let effect = e
        .program(p)
        .instructions
        .iter()
        .find_map(|i| {
            if let Instruction::Apply(effect) = i {
                Some(effect.clone())
            } else {
                None
            }
        })
        .unwrap();
    s.source = source;
    s.ability_program = p + 1;
    let before = s;
    assert!(e
        .apply_effect(&mut s, &effect)
        .unwrap_err()
        .contains("global arena order"));
    assert_eq!(s, before);
    assert!(e.in_arena(&s, enemy));
}
#[test]
fn summa_direct_and_source_admission_guards_stay_closed() {
    let good = entry();
    assert!(admits(good.clone()));
    let a: crate::ability_ir::Ability =
        serde_json::from_value(good["abilities"]["card"][0].clone()).unwrap();
    assert!(Program::compile(&a).is_ok());
    for (key, value) in [
        ("trigger", json!("event")),
        ("optional", json!(true)),
        ("condition", json!({"kind":"const","value":true})),
        ("costs", json!([{"kind":"exhaustSelf"}])),
        ("contextTitle", json!([{"kind":"const","value":"other"}])),
        ("attackerMustSurvive", json!(true)),
        (
            "targetResolver",
            json!({"owner":"any","type":"unit","zone":"anyAttackable"}),
        ),
    ] {
        let mut bad = good.clone();
        bad["abilities"]["card"][0][key] = value;
        let a: crate::ability_ir::Ability =
            serde_json::from_value(bad["abilities"]["card"][0].clone()).unwrap();
        assert!(Program::compile(&a).is_err(), "direct {key}");
        assert!(!admits(bad), "entry {key}");
    }
    for (key, value) in [
        ("types", json!(["leader"])),
        ("arena", json!("ground")),
        ("keywords", json!(["support"])),
        ("pilotText", json!("Piloting [1]")),
    ] {
        let mut bad = good.clone();
        bad["card"][key] = value;
        assert!(!admits(bad), "metadata {key}");
    }
    let mut bad = good.clone();
    bad["baseClass"] = json!("LeaderUnitCard");
    assert!(!admits(bad));
    let mut bad = good.clone();
    bad["abilities"]["card"].as_array_mut().unwrap().push(
        json!({"trigger":"whenPlayed","title":"extra","costs":[],"effect":{"kind":"noAction"}}),
    );
    assert!(!admits(bad));
    let mut bad = good;
    bad["abilities"]["card"][0]["effect"]["target"]["controllerOrder"] = json!("playerFirst");
    assert!(!admits(bad));
    assert!(!e_rejected_phantom());
}
fn e_rejected_phantom() -> bool {
    engine().by_code.contains_key("SHD_187")
}

#[test]
fn captured_defense_survives_defender_removal_and_preserves_owner() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for first in ["You", "Opponent"] {
            let mut s = board(&e, seat);
            let source = unit(&e, &mut s, seat, "ASH_083");
            let defender = unit(&e, &mut s, seat.other(), "TS26_024");
            let base = s.player(seat.other()).base;
            s.card_mut(source).exhausted = false;
            let choosing = attack(&e, &s, source, defender);
            assert_eq!(choosing.decision, Decision::DeathPlayer);
            assert_eq!(choosing.attack_source, source);
            let done = settle(&e, button(&e, &choosing, first));
            assert_eq!(done.card(base).damage, 1);
            assert!(!e.in_arena(&done, defender));
            assert_eq!(done.card(source).damage, 0);
            assert_eq!(done.attack_source, 0);
            assert_eq!(done.attack_defender, 0);
        }
    }
}
#[test]
fn death_effect_source_does_not_replace_attack_source_and_can_defeat_summa() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat, "ASH_083");
        s.card_mut(source).damage = 14;
        let donor = unit(&e, &mut s, seat.other(), "SHD_164");
        let defender = unit(&e, &mut s, seat.other(), "LOF_192");
        s.card_mut(source).exhausted = false;
        let choosing = attack(&e, &s, source, defender);
        assert_eq!(choosing.decision, Decision::AbilityTarget);
        assert_eq!(choosing.source, donor);
        assert_eq!(choosing.active, seat.other());
        assert_eq!(choosing.attack_source, source);
        assert!(!e.in_arena(&choosing, donor));
        assert!(!e.in_arena(&choosing, defender));
        let done = settle(&e, choose(&e, &choosing, source));
        assert!(!e.in_arena(&done, source));
        assert!(done.player(seat).discard.contains(source));
        assert_eq!(done.attack_source, 0);
        assert_eq!(done.attack_defender, 0);
    }
}
#[test]
fn summa_ordinary_paid_entry_and_intrinsic_sentinel_are_preserved() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let source = add(&e, &mut s, seat, "ASH_083");
        s.player_mut(seat).hand.push(source).unwrap();
        let done = choose(&e, &s, source);
        assert!(e.in_arena(&done, source));
        assert!(done.card(source).exhausted);
        assert!(e.has_keyword(&done, source, "sentinel"));
        assert_eq!(e.power(&done, source, false), 15);
        assert_eq!(e.hp(&done, source), 15);
        let mut s = board(&e, seat);
        let source = unit(&e, &mut s, seat.other(), "ASH_083");
        let other = unit(&e, &mut s, seat.other(), "LOF_192");
        let attacker = unit(&e, &mut s, seat, "LOF_192");
        assert_eq!(e.attack_targets(&s, attacker, false), vec![source]);
        s.card_mut(attacker).add_phase_keyword(
            crate::ability_ir::CORE_KEYWORDS
                .iter()
                .position(|k| *k == "saboteur")
                .unwrap(),
        );
        let targets = e.attack_targets(&s, attacker, false);
        assert!(targets.contains(&source));
        assert!(targets.contains(&other));
        assert!(targets.contains(&s.player(seat.other()).base));
    }
}
