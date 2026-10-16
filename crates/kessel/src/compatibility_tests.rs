use super::*;
fn strict() -> Engine {
    Engine::from_pool_bytes(include_bytes!("../data/pool.json")).unwrap()
}
fn compiled() -> Engine {
    Engine::from_compiled_pool_bytes(
        include_bytes!("../data/pool.json"),
        include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")),
    )
    .unwrap()
}
fn runtime() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
        .unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: *e.by_code.get(code).unwrap(),
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine) -> State {
    let mut s = State::default();
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (seat, code) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, code);
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}

#[test]
fn compatibility_leaders_keep_strict_actions_and_compiled_program_boundary() {
    let e = strict();
    let mut s = board(&e);
    let leader = s.player(s.active).leader;
    assert_eq!(e.leader_available(&s), (false, true));
    e.use_leader(&mut s).unwrap();
    assert_eq!(s.card(s.player(Seat::P1).base).damage, 1);
    assert_eq!(s.card(s.player(Seat::P2).base).damage, 1);
    assert!(s.card(leader).exhausted);
    let mut s = board(&e);
    s.active = Seat::P2;
    assert_eq!(e.leader_available(&s), (false, false));
    let resource = add(&e, &mut s, Seat::P2, "LOF_254");
    s.player_mut(Seat::P2).resources.push(resource).unwrap();
    assert_eq!(e.leader_available(&s), (false, true));
    e.use_leader(&mut s).unwrap();
    assert_eq!(s.decision, Decision::VaderBase);
    assert!(
        !s.card(s.player(Seat::P2).leader).exhausted,
        "payment remains after base choice"
    );
    let mut s = board(&e);
    let leader = s.player(s.active).leader;
    s.card_mut(leader).def = *e.by_code.get("SEC_005").unwrap();
    e.use_leader(&mut s).unwrap();
    assert_eq!(s.decision, Decision::NoEffect);
    let mut e = compiled();
    let mut s = board(&e);
    let leader = s.player(s.active).leader;
    let index = s.card(leader).def as usize;
    e.definitions[index].programs.leader.clear();
    assert_eq!(e.leader_available(&s), (false, false));
    assert_eq!(
        e.use_leader(&mut s),
        Err("leader has no executable action".into())
    );
    assert_eq!(
        s.card(s.player(Seat::P1).base).damage,
        0,
        "compiled source must never use legacy fallback"
    );
}

#[test]
fn compatibility_deployment_contract_is_shared_without_changing_action_exhaustion() {
    for e in [strict(), compiled()] {
        let mut s = board(&e);
        let leader = s.player(s.active).leader;
        for _ in 0..6 {
            let id = add(&e, &mut s, Seat::P1, "LOF_254");
            s.player_mut(Seat::P1).resources.push(id).unwrap();
        }
        s.card_mut(leader).exhausted = true;
        assert!(
            e.leader_available(&s).0,
            "deploy depends on resources/epic/deployed, not exhaustion"
        );
        s.card_mut(leader).epic_used = true;
        assert!(!e.leader_available(&s).0);
        s.card_mut(leader).epic_used = false;
        s.card_mut(leader).deployed = true;
        assert_eq!(e.leader_available(&s), (false, false));
    }
}

#[test]
fn compatibility_snapshot_preserves_exact_evidence_and_unknown_legacy_history() {
    let e = strict();
    let mut s = board(&e);
    let snapshot = e.snapshot(&s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(restored.zobrist(), s.zobrist());
    for name in [
        "forceStateVersion",
        "hiddenHistoryVersion",
        "phaseHistory",
        "registrationOrder",
        "arenaUnitOrder",
    ] {
        assert!(snapshot.get(name).is_some(), "missing {name}");
    }
    s.force_state_exact = false;
    s.hidden_history_exact = false;
    s.phase_history.exact = false;
    s.registration_order.exact = false;
    s.arena_order_exact = false;
    let snapshot = e.snapshot(&s);
    let mut restored = e.load_state(&snapshot).unwrap();
    for name in [
        "forceStateVersion",
        "hiddenHistoryVersion",
        "phaseHistory",
        "registrationOrder",
        "arenaUnitOrder",
    ] {
        assert!(snapshot.get(name).is_none(), "invented {name}");
    }
    let prior = restored.phase_history;
    restored.phase_history.entered(Seat::P1, 3);
    restored.phase_history.played(Seat::P1, 3).unwrap();
    assert_eq!(
        restored.phase_history, prior,
        "observing a suffix cannot establish exact prior history"
    );
    assert_eq!(e.snapshot(&restored), snapshot);
}

#[test]
fn compatibility_required_evidence_and_pending_support_still_fail_closed() {
    let e = runtime();
    let s = board(&e);
    let snapshot = e.snapshot(&s);
    assert_eq!(e.snapshot(&e.load_state(&snapshot).unwrap()), snapshot);
    for (field,diagnostic) in [
        ("hiddenHistoryVersion","selected pool requires explicit Hidden phase history"),
        ("phaseHistory","selected pool requires explicit phaseHistory version"),
        ("registrationOrder","defeat-hook/granted-death sources require explicit registrationOrder including future cards in selected pool"),
        ("forceStateVersion","selected pool requires explicit Force state version"),
    ] {
        let mut bad=snapshot.clone();bad.as_object_mut().unwrap().remove(field);
        if field=="forceStateVersion" {for seat in ["p1","p2"] {bad["players"][seat].as_object_mut().unwrap().remove("forceToken");}}
        assert_eq!(e.load_state(&bad).unwrap_err(),diagnostic,"{field}");
    }
    for field in ["hiddenHistoryVersion", "forceStateVersion"] {
        let mut bad = snapshot.clone();
        bad[field] = json!(2);
        assert_eq!(
            e.load_state(&bad).unwrap_err(),
            format!("unsupported {field}")
        );
    }
    let mut bad = snapshot.clone();
    bad["supportAttackPending"] = json!(true);
    assert_eq!(
        e.load_state(&bad).unwrap_err(),
        "pending Support attacks cannot be imported as action windows"
    );
    let mut bad = snapshot;
    bad.as_object_mut().unwrap().remove("arenaUnitOrder");
    assert_eq!(
        e.load_state(&bad).unwrap_err(),
        "registrationOrder requires explicit arenaUnitOrder evidence"
    );
}
