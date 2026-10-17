//! Native lifecycle and stable Credit snapshot tests. Payment menus are root-owned.
use super::*;

fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn engine() -> Engine {
    Engine::from_catalogue_value(&catalogue()).unwrap()
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
fn board(e: &Engine) -> State {
    let mut s = State::default();
    s.credit_state_exact = true;
    s.force_state_exact = true;
    s.hidden_history_exact = true;
    for seat in [Seat::P1, Seat::P2] {
        s.player_mut(seat).base = add(e, &mut s, seat, "ASH_019");
        s.player_mut(seat).leader = add(e, &mut s, seat, "SOR_014");
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn live(s: &State, seat: Seat) -> Vec<u16> {
    s.player(seat)
        .credits
        .as_slice()
        .iter()
        .map(|id| u16::from(*id))
        .collect()
}
fn selected(e: &Engine, codes: &[&str]) -> Engine {
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| codes.iter().any(|code| c["setCode"] == *code));
    Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&catalogue()).unwrap(),
    )
    .unwrap()
}

#[test]
fn credit_native_lifecycle_keeps_order_ownership_and_fresh_identity_for_both_seats() {
    let e = engine();
    assert!(e.needs_credit_state());
    for payer in [Seat::P1, Seat::P2] {
        let other = payer.other();
        let mut s = board(&e);
        let history = s.phase_history;
        let registration = s.registration_order;
        let next = s.next_id;
        assert_eq!(e.create_credit_tokens(&mut s, payer, 3).unwrap(), 3);
        assert_eq!(e.create_credit_tokens(&mut s, other, 1).unwrap(), 1);
        assert_eq!(live(&s, payer), vec![next, next + 1, next + 2]);
        assert_eq!(
            e.transfer_credit_tokens(&mut s, payer, other, 2).unwrap(),
            2
        );
        assert_eq!(live(&s, payer), vec![next + 2]);
        assert_eq!(live(&s, other), vec![next + 3, next, next + 1]);
        assert_eq!(s.card(next).owner, other);
        assert_eq!(s.card(next + 1).owner, other);
        assert_eq!(
            e.transfer_credit_tokens(&mut s, other, other, 5).unwrap(),
            0
        );
        e.defeat_credits_for_payment(&mut s, other, 2).unwrap();
        assert_eq!(live(&s, other), vec![next + 1]);
        for id in [next + 3, next] {
            assert!(s.card(id).present);
            assert_eq!(s.card(id).owner, other);
            assert_eq!(s.card(id).def, e.credit_def.unwrap());
            assert!(!s.player(other).discard.contains(id));
        }
        let fresh = s.next_id;
        e.create_credit_tokens(&mut s, other, 1).unwrap();
        assert_eq!(fresh, next + 4);
        assert_eq!(live(&s, other), vec![next + 1, fresh]);
        assert_eq!(s.phase_history, history);
        assert_eq!(s.registration_order, registration);
        assert_eq!(s, s.clone());
        assert_eq!(s.zobrist(), s.clone().zobrist());
    }
}

#[test]
fn credit_allocation_and_invalid_payment_fail_before_any_mutation() {
    let e = engine();
    let mut s = board(&e);
    s.next_id = (MAX_CARDS - 1) as u16;
    let before = s;
    assert!(e.create_credit_tokens(&mut s, Seat::P1, 2).is_err());
    assert_eq!(s, before);
    e.create_credit_tokens(&mut s, Seat::P1, 1).unwrap();
    assert_eq!(s.next_id as usize, MAX_CARDS);
    let before = s;
    assert!(e.create_credit_tokens(&mut s, Seat::P1, 1).is_err());
    assert!(e.defeat_credits_for_payment(&mut s, Seat::P1, 2).is_err());
    assert_eq!(s, before);
    let mut invalid = board(&e);
    invalid.credit_state_exact = false;
    let before = invalid;
    assert!(e.create_credit_tokens(&mut invalid, Seat::P1, 1).is_err());
    assert_eq!(invalid, before);
    let mut reused = board(&e);
    let next = reused.next_id;
    *reused.card_mut(next) = Card {
        present: true,
        ..Card::default()
    };
    let before = reused;
    assert!(e.create_credit_tokens(&mut reused, Seat::P1, 1).is_err());
    assert_eq!(reused, before);
}

#[test]
fn credit_lifecycle_rejects_wrong_holding_owner_and_modified_native_tokens() {
    let e = engine();
    let mut s = board(&e);
    e.create_credit_tokens(&mut s, Seat::P1, 2).unwrap();
    let id = live(&s, Seat::P1)[0];
    for field in ["owner", "parent", "exhausted", "phaseRestrictions"] {
        let mut changed = s;
        match field {
            "owner" => changed.card_mut(id).owner = Seat::P2,
            "parent" => changed.card_mut(id).parent = changed.player(Seat::P1).base,
            "exhausted" => changed.card_mut(id).exhausted = true,
            _ => changed.card_mut(id).phase_restrictions = 1,
        }
        let before = changed;
        assert!(
            e.defeat_credits_for_payment(&mut changed, Seat::P1, 1)
                .is_err(),
            "{field}"
        );
        assert!(
            e.transfer_credit_tokens(&mut changed, Seat::P1, Seat::P2, 1)
                .is_err(),
            "{field}"
        );
        assert_eq!(changed, before);
    }
}

#[test]
fn credit_stable_snapshot_roundtrips_live_and_retired_identities_and_full_hash() {
    let e = engine();
    let mut s = board(&e);
    e.create_credit_tokens(&mut s, Seat::P1, 4).unwrap();
    e.create_credit_tokens(&mut s, Seat::P2, 2).unwrap();
    e.transfer_credit_tokens(&mut s, Seat::P1, Seat::P2, 2)
        .unwrap();
    e.defeat_credits_for_payment(&mut s, Seat::P2, 3).unwrap();
    e.create_credit_tokens(&mut s, Seat::P1, 1).unwrap();
    let snapshot = e.snapshot(&s);
    assert_eq!(snapshot["creditStateVersion"], 1);
    assert_eq!(
        snapshot["players"]["p2"]["retiredCredits"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let imported = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&imported), snapshot);
    assert_eq!(live(&imported, Seat::P1), live(&s, Seat::P1));
    assert_eq!(live(&imported, Seat::P2), live(&s, Seat::P2));
    assert_eq!(imported.next_id, s.next_id);
    assert_eq!(imported.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&imported), e.legal_actions(&s));
}

#[test]
fn credit_import_rejects_missing_version_pending_and_incomplete_retired_projection() {
    let e = engine();
    let s = board(&e);
    let original = e.snapshot(&s);
    for field in ["creditStateVersion", "retiredCredits", "credits", "pending"] {
        let mut changed = original.clone();
        match field {
            "creditStateVersion" => {
                changed.as_object_mut().unwrap().remove(field);
            }
            "pending" => changed["creditPaymentPending"] = json!(true),
            _ => {
                changed["players"]["p1"]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
            }
        }
        assert!(e.load_state(&changed).is_err(), "{field}");
    }
    let mut changed = original;
    changed["creditStateVersion"] = json!(2);
    assert!(e.load_state(&changed).is_err());
}

#[test]
fn credit_import_rejects_duplicate_out_of_zone_force_and_unknown_effect_identity() {
    let e = engine();
    let mut s = board(&e);
    e.create_credit_tokens(&mut s, Seat::P1, 3).unwrap();
    e.defeat_credits_for_payment(&mut s, Seat::P1, 2).unwrap();
    let original = e.snapshot(&s);
    let current = original["players"]["p1"]["credits"][0].clone();
    for mutation in [
        "duplicate",
        "crossPlayer",
        "retiredOverlap",
        "base",
        "force",
        "next",
        "noncanonical",
        "definition",
        "name",
        "blank",
        "owner",
        "retiredOrder",
    ] {
        let mut changed = original.clone();
        match mutation {
            "duplicate" => changed["players"]["p1"]["credits"]
                .as_array_mut()
                .unwrap()
                .push(current.clone()),
            "crossPlayer" => changed["players"]["p2"]["credits"]
                .as_array_mut()
                .unwrap()
                .push(current.clone()),
            "retiredOverlap" => changed["players"]["p1"]["retiredCredits"]
                .as_array_mut()
                .unwrap()
                .push(current.clone()),
            "base" => {
                changed["players"]["p1"]["credits"][0]["id"] =
                    changed["players"]["p1"]["base"]["id"].clone()
            }
            "force" => {
                changed["players"]["p1"]["credits"][0]["id"] =
                    changed["players"]["p1"]["forceToken"]["id"].clone()
            }
            "next" => {
                changed["players"]["p1"]["credits"][0]["id"] = json!(format!("c{}", s.next_id - 1))
            }
            "noncanonical" => changed["players"]["p1"]["credits"][0]["id"] = json!("c01"),
            "definition" => {
                changed["players"]["p1"]["credits"][0]["setCode"] = json!("TOKEN_SHIELD")
            }
            "name" => changed["players"]["p1"]["credits"][0]["name"] = json!("shield"),
            "blank" => changed["players"]["p1"]["credits"][0]["blanked"] = json!(true),
            "owner" => changed["players"]["p1"]["credits"][0]["owner"] = json!("p2"),
            _ => changed["players"]["p1"]["retiredCredits"]
                .as_array_mut()
                .unwrap()
                .reverse(),
        }
        assert!(e.load_state(&changed).is_err(), "{mutation}");
    }
}

#[test]
fn credit_native_provenance_is_required_in_the_selected_pool() {
    let full = engine();
    let codes = ["ASH_019", "SOR_014", "TOKEN_SHIELD"];
    let e = selected(&full, &codes);
    assert!(
        !e.needs_credit_state(),
        "retained catalogue native token is not selected provenance"
    );
    let mut s = board(&e);
    s.credit_state_exact = false;
    let before = s;
    assert!(e.create_credit_tokens(&mut s, Seat::P1, 1).is_err());
    assert_eq!(s, before);
    let snapshot = e.snapshot(&s);
    assert!(snapshot.get("creditStateVersion").is_none());
    let loaded = e.load_state(&snapshot).unwrap();
    assert!(!loaded.credit_state_exact);
    assert_eq!(loaded.zobrist(), s.zobrist());
    let mut bad = catalogue();
    bad["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "TOKEN_CREDIT")
        .unwrap()["sourceHash"] = json!("unreviewed");
    let rejected = Engine::from_catalogue_value(&bad).unwrap();
    assert!(!rejected.needs_credit_state());
    let mut s = board(&rejected);
    assert!(rejected.create_credit_tokens(&mut s, Seat::P2, 1).is_err());
}
