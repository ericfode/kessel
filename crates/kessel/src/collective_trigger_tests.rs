use super::*;
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn unused_entry() -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "ASH_041")
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
        def: *e.by_code.get(code).unwrap_or_else(|| {
            panic!("{code}: {:?}", e.rejected_cards().and_then(|r| r.get(code)))
        }),
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
    s.force_state_exact = true;
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

fn token_count(e: &Engine, s: &State, parent: CardId, code: &str) -> usize {
    e.upgrades(s, parent)
        .iter()
        .filter(|id| e.def(s, **id).code == code)
        .count()
}
fn paid(e: &Engine, s: &mut State, code: &str) -> State {
    let seat = s.active;
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    choose(e, s, id)
}
#[test]
fn axe_actual_paid_draw_windows_owner_scope_and_empty_deck_event() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let axe = unit(&e, &mut s, seat, "ASH_169");
        for _ in 0..6 {
            let c = add(&e, &mut s, seat, "LOF_254");
            s.player_mut(seat).deck.push(c).unwrap();
        }
        let mut s = paid(&e, &mut s, "TWI_175");
        assert!(
            s.is_action_window(),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
        assert_eq!(token_count(&e, &s, axe, "TOKEN_ADVANTAGE"), 1);
        s.active = seat;
        s.action_player = Some(seat);
        s.prev_passed = false;
        let mut s = paid(&e, &mut s, "TWI_175");
        assert_eq!(token_count(&e, &s, axe, "TOKEN_ADVANTAGE"), 2);
        s.active = seat;
        s.action_player = Some(seat);
        s.prev_passed = false;
        let damage = s.card(s.player(seat).base).damage;
        let s = paid(&e, &mut s, "TWI_175");
        assert_eq!(s.card(s.player(seat).base).damage, damage + 9);
        assert_eq!(token_count(&e, &s, axe, "TOKEN_ADVANTAGE"), 3);
    }
}
#[test]
fn sabine_actual_shielded_optional_ground_selector() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let ground = unit(&e, &mut s, seat.other(), "LOF_254");
        let space = unit(&e, &mut s, seat.other(), "LOF_192");
        let s = paid(&e, &mut s, "ASH_208");
        let sabine = s
            .player(seat)
            .ground
            .as_slice()
            .iter()
            .copied()
            .find(|id| e.def(&s, *id).code == "ASH_208")
            .unwrap();
        assert_eq!(token_count(&e, &s, sabine, "TOKEN_SHIELD"), 1);
        let actions = e.legal_actions(&s);
        assert!(
            actions.contains(&e.card_action(&s, ground)),
            "{:?} {:?}",
            s.decision,
            actions
        );
        assert!(!actions.contains(&e.card_action(&s, space)));
        let s = choose(&e, &s, ground);
        assert!(s.card(ground).exhausted);
        assert!(
            s.is_action_window(),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
    }
}
#[test]
fn collective_batch_identity_is_ability_and_exact_window_not_pending_window() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let sabine = unit(&e, &mut s, Seat::P1, "ASH_208");
    let axe = unit(&e, &mut s, Seat::P1, "ASH_169");
    let batch = [
        CollectiveEvent::UpgradeAttached {
            parent: sabine,
            upgrade: 100,
        },
        CollectiveEvent::CardsDrawn(Seat::P1),
        CollectiveEvent::UpgradeAttached {
            parent: sabine,
            upgrade: 101,
        },
    ];
    e.capture_collective_window(&mut s, &batch).unwrap();
    assert_eq!(s.death_pending.len, 2);
    assert_eq!(s.death_pending.entries[0].source, axe);
    assert_eq!(s.death_pending.entries[1].source, sabine);
    assert_eq!(s.death_pending.entries[1].event_card, 100);
    e.capture_collective_window(&mut s, &batch).unwrap();
    assert_eq!(s.death_pending.len, 4);
    assert_eq!(s.death_pending.entries[3].source, sabine);
    let before = s;
    s.death_pending.len = MAX_DEATH_TRIGGERS as u8;
    let full = s;
    assert!(e.capture_collective_window(&mut s, &batch).is_err());
    assert_eq!(s, full);
    assert_eq!(before, *&before);
}
#[test]
fn whole_collective_shapes_reject_unknown_wrappers_markers_and_metadata() {
    let c = catalogue();
    for code in ["ASH_169", "ASH_208"] {
        let original = c["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["setCode"] == code)
            .unwrap();
        assert!(crate::collective_trigger_contract::prepare(original)
            .unwrap()
            .is_some());
        for n in 0..9 {
            let mut x = original.clone();
            match n {
                0 => x["abilities"]["card"][0]["collectiveTrigger"] = json!(false),
                1 => x["abilities"]["card"][0]["hooks"][0]["predicate"]["op"] = json!("!=="),
                2 => x["abilities"]["card"][0]["unknown"] = json!(true),
                3 => x["card"]["types"] = json!(["leader"]),
                4 => x["abilities"]["card"][0]["effect"]["target"] = json!("opponent"),
                5 => x["card"]["collectiveTrigger"] = json!(true),
                6 => x["baseClass"] = json!("UpgradeCard"),
                7 => {
                    x["abilities"]["card"][0]["hooks"][0]["predicate"]["left"]["steps"][0]
                        ["optional"] = json!(true)
                }
                _ => x["card"]["unique"] = json!(false),
            }
            assert!(
                crate::collective_trigger_contract::prepare(&x).is_err(),
                "{code} mutation{n}"
            );
        }
    }
    let mut deep = json!(false);
    for _ in 0..66 {
        deep = json!([deep]);
    }
    assert!(crate::collective_trigger_contract::marker(&deep).is_err());
    assert!(
        crate::collective_trigger_contract::marker(&Value::Array(vec![json!(false); 32769]))
            .is_err()
    );
    assert!(
        crate::collective_trigger_contract::marker(&json!({"collectiveTrigger":null})).unwrap()
    );
}

#[test]
fn sabine_actual_two_xp_creation_collects_once_before_ground_choice() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let sabine = unit(&e, &mut s, seat, "ASH_208");
        let ground = unit(&e, &mut s, seat.other(), "LOF_254");
        let s = paid(&e, &mut s, "IC27_184");
        let s = choose(&e, &s, sabine);
        assert_eq!(token_count(&e, &s, sabine, "TOKEN_EXPERIENCE"), 2);
        assert_eq!(
            s.death_window.entries[..s.death_window.len as usize]
                .iter()
                .filter(|x| x.source == sabine)
                .count(),
            0,
            "selected sole collective registration is executing, not duplicated"
        );
        assert!(
            e.legal_actions(&s).contains(&e.card_action(&s, ground)),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
        let s = choose(&e, &s, ground);
        assert!(s.card(ground).exhausted);
        assert!(
            s.is_action_window(),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
        assert_eq!(s.card(sabine).damage, 2);
    }
}
#[test]
fn sabine_resolved_weakness_attachment_capture_survives_source_stat_death() {
    let e = engine();
    for (seat, donor) in [Seat::P1, Seat::P2]
        .into_iter()
        .flat_map(|seat| ["HMW_100", "HMW_097"].map(move |donor| (seat, donor)))
    {
        let mut s = board(&e, seat);
        let sabine = unit(&e, &mut s, seat, "ASH_208");
        s.card_mut(sabine).damage = 4;
        let ground = unit(&e, &mut s, seat.other(), "LOF_254");
        let host = if donor == "HMW_097" {
            Some(unit(&e, &mut s, seat, "LOF_254"))
        } else {
            None
        };
        let s = paid(&e, &mut s, donor);
        let s = if let Some(host) = host {
            choose(&e, &s, host)
        } else {
            s
        };
        let s = choose(&e, &s, sabine);
        assert!(
            s.player(seat).discard.contains(sabine),
            "source must die before captured selector; hp={} damage={} upgrades={:?} decision={:?} actions={:?}",e.hp(&s,sabine),s.card(sabine).damage,e.upgrades(&s,sabine),s.decision,e.legal_actions(&s)
        );
        assert!(e.upgrades(&s, sabine).is_empty());
        assert!(
            e.legal_actions(&s).contains(&e.card_action(&s, ground)),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
        let s = choose(&e, &s, ground);
        assert!(s.card(ground).exhausted);
        assert!(s.is_action_window());
    }
}
#[test]
fn regroup_draw_batch_captures_both_owners_before_resource_prompts() {
    let e = engine();
    let mut s = board(&e, Seat::P1);
    let a = unit(&e, &mut s, Seat::P1, "ASH_169");
    let b = unit(&e, &mut s, Seat::P2, "ASH_169");
    s.regroup = true;
    s.action_player = None;
    e.draw_for_regroup(&mut s).unwrap();
    assert_eq!(s.decision, Decision::DeathPlayer);
    let first = e.legal_actions(&s)[0].clone();
    let mut s = e.apply(&s, &first).unwrap();
    for _ in 0..8 {
        if s.decision == Decision::Resource {
            break;
        }
        let actions = e.legal_actions(&s);
        assert_eq!(actions.len(), 1, "{:?} {:?}", s.decision, actions);
        s = e.apply(&s, &actions[0]).unwrap();
    }
    assert_eq!(s.decision, Decision::Resource);
    assert_eq!(token_count(&e, &s, a, "TOKEN_ADVANTAGE"), 1);
    assert_eq!(token_count(&e, &s, b, "TOKEN_ADVANTAGE"), 1);
    for seat in [Seat::P1, Seat::P2] {
        assert_eq!(s.card(s.player(seat).base).damage, 6);
    }
}
#[test]
fn cancelled_attachment_filter_precedes_dedup_and_keeps_first_resolved_context() {
    let e = engine();
    let mut before = board(&e, Seat::P1);
    let sabine = unit(&e, &mut before, Seat::P1, "ASH_208");
    let mut after = before;
    // Event100 was cancelled; event101 in the same window resolved. The
    // actual prepared handler supplies only its resolved physical events.
    e.capture_resolved_collective_window(
        &mut after,
        &before,
        0,
        &[CollectiveEvent::UpgradeAttached {
            parent: sabine,
            upgrade: 101,
        }],
    )
    .unwrap();
    assert_eq!(after.death_pending.len, 1);
    assert_eq!(after.death_pending.entries[0].event_card, 101);
    let mut none = before;
    e.capture_resolved_collective_window(&mut none, &before, 0, &[])
        .unwrap();
    assert_eq!(none, before);
}
#[test]
fn raw_collective_markers_cannot_bypass_specialized_or_pool_loaders() {
    for code in ["LOF_255", "ASH_191", "ASH_197"] {
        let mut c = catalogue();
        let changed = c["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x["setCode"] == code)
            .unwrap();
        changed["abilities"]["card"][0]["collectiveTrigger"] = json!(true);
        let e = Engine::from_catalogue_value(&c).unwrap();
        assert!(
            !e.by_code.contains_key(code),
            "raw marker laundered through {code} normalization"
        );
    }
    let original: Value = serde_json::from_str(include_str!("../data/pool.json")).unwrap();
    for value in [json!(null), json!(false), json!({})] {
        let mut p = original.clone();
        p["cards"][0]["collectiveTrigger"] = value;
        assert!(crate::validation::validate_pool(&p).is_err());
        assert!(Engine::from_pool_value(&p).is_err());
        let c = catalogue();
        assert!(Engine::from_compiled_pool_bytes(
            &serde_json::to_vec(&p).unwrap(),
            &serde_json::to_vec(&c).unwrap()
        )
        .is_err());
    }
}

#[test]
fn ordinary_upgrade_attachment_precedes_own_play_and_child_window_is_distinct() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e, seat);
        let sabine = unit(&e, &mut s, seat, "ASH_208");
        let first = unit(&e, &mut s, seat.other(), "LOF_254");
        let second = unit(&e, &mut s, seat.other(), "LOF_254");
        let s = paid(&e, &mut s, "HMW_097");
        let s = choose(&e, &s, sabine);
        let menu = e.legal_actions(&s);
        assert_eq!(menu.len(), 2);
        assert_eq!(menu[0], Action::button("Exhaust a ground unit", "0"));
        assert_eq!(
            menu[1],
            Action::button("Give a Weakness token to a unit", "1")
        );
        let s = e.apply(&s, &menu[1]).unwrap();
        let s = choose(&e, &s, sabine);
        let s = choose(&e, &s, first);
        assert!(s.card(first).exhausted);
        assert!(!s.card(second).exhausted);
        let s = choose(&e, &s, second);
        assert!(s.card(second).exhausted);
        assert!(
            s.is_action_window(),
            "{:?} {:?}",
            s.decision,
            e.legal_actions(&s)
        );
    }
}

#[test]
fn selected_collective_pool_requires_real_tokens_without_global_catalogue_leakage() {
    let c = catalogue();
    let e = Engine::from_catalogue_value(&c).unwrap();
    let mut p = e.compiled_pool().unwrap();
    p["cards"].as_array_mut().unwrap().retain(|x| {
        ["ASH_169", "ASH_208", "TOKEN_ADVANTAGE", "TOKEN_SHIELD"]
            .contains(&x["setCode"].as_str().unwrap_or(""))
    });
    let cb = serde_json::to_vec(&c).unwrap();
    assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&p).unwrap(), &cb).is_ok());
    for missing in ["TOKEN_ADVANTAGE", "TOKEN_SHIELD"] {
        let mut bad = p.clone();
        bad["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|x| x["setCode"] != missing);
        assert!(Engine::from_compiled_pool_bytes(&serde_json::to_vec(&bad).unwrap(), &cb).is_err());
    }
    p["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|x| x["setCode"] == "TOKEN_SHIELD");
    let selected = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&p).unwrap(), &cb).unwrap();
    assert!(!selected.needs_registration_order());
}

#[test]
fn combined_axe_blade_regroup_captures_both_families_before_either_draw_once() {
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();
        for initiative in [Seat::P1,Seat::P2] {for cards in [[0,0],[0,2],[2,0]] {
            let mut s=board(&e,Seat::P1);s.initiative=initiative;s.regroup=true;s.action_player=None;
            // Deliberately interleave entry chronology across families/owners.
            let blade2=unit(&e,&mut s,Seat::P2,"ASH_204");let axe1=unit(&e,&mut s,Seat::P1,"ASH_169");
            let blade1=unit(&e,&mut s,Seat::P1,"ASH_204");let axe2=unit(&e,&mut s,Seat::P2,"ASH_169");
            for seat in [Seat::P1,Seat::P2] {for _ in 0..cards[seat.index()] {let id=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).deck.push(id).unwrap();}}
            e.draw_for_regroup(&mut s).unwrap();assert_ne!(s.decision,Decision::Resource);
            let mut expected=vec![axe1,axe2];if cards[0]==0 {expected.push(blade1);}if cards[1]==0 {expected.push(blade2);}
            assert_eq!(s.death_window.entries[..s.death_window.len as usize].iter().map(|v|v.source).collect::<Vec<_>>(),expected,"pre-primary event positions must remain stable");
            for _ in 0..32 {if s.decision==Decision::Resource {break;}let action=e.legal_actions(&s).into_iter().next().unwrap();s=e.apply(&s,&action).unwrap();}
            assert_eq!(s.decision,Decision::Resource);
            for (seat,axe,blade) in [(Seat::P1,axe1,blade1),(Seat::P2,axe2,blade2)] {
                assert_eq!(token_count(&e,&s,axe,"TOKEN_ADVANTAGE"),1,"one draw event per owner, never double capture");
                assert_eq!(token_count(&e,&s,blade,"TOKEN_ADVANTAGE"),usize::from(cards[seat.index()]==0));
                assert_eq!(s.card(s.player(seat).base).damage,if cards[seat.index()]==0 {6}else{0});
            }
        }}
    }
}

#[test]
fn combined_prepared_draw_keeps_original_pending_position_and_departed_rosters() {
    let e=engine();let mut s=board(&e,Seat::P1);let blade=unit(&e,&mut s,Seat::P1,"ASH_204");let axe=unit(&e,&mut s,Seat::P1,"ASH_169");
    let event=add(&e,&mut s,Seat::P1,"ASH_151");s.source=event;s.ability_program=e.card_programs(&s,event)[0]+1;
    // An earlier event in this parent already owns the first pending position.
    let listeners=e.blade_base_damage_listeners(&s).unwrap();let base=s.player(Seat::P1).base;
    e.capture_blade_base_damage_event(&mut s,&listeners,base,true).unwrap();assert_eq!(s.death_pending.len,1);
    let effect:Effect=serde_json::from_value(json!({"kind":"simultaneous","effects":[{"kind":"defeat","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true}}},{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":2}}]})).unwrap();
    assert!(e.generate_group(&mut s,&effect).unwrap());assert!(e.resolve_group(&mut s,&effect).unwrap());
    assert!(!e.in_arena(&s,blade)&&!e.in_arena(&s,axe));
    assert_eq!(s.death_pending.entries[..s.death_pending.len as usize].iter().map(|v|v.source).collect::<Vec<_>>(),vec![blade,axe,blade]);
    assert_eq!(s.card(base).damage,6);
}
