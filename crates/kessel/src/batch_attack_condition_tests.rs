// Include as a child module of generic_runtime's existing tests module.
use super::*;

fn snowtrooper_roundtrip(e: &Engine, s: &State) -> State {
    let snapshot = e.snapshot(s);
    let restored = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&restored), snapshot);
    assert_eq!(e.legal_actions(&restored), e.legal_actions(s));
    restored
}

// Synthetic internal live-expression diagnostic. The initial mid-game board
// round-trips publicly; the direct attack-field mutations below are not a legal
// attack witness and are kept separate from the legal-combat scenario.
#[test]
fn snowtrooper_synthetic_internal_live_expression_is_attack_specific() {
    let e = engine();
    assert!(e.by_code.contains_key("SOR_130"), "normalized complete card must be admitted by the unchanged runtime");
    assert!(e.by_code.contains_key("ASH_241"), "bounded Support lifecycle now admits the complete Marrok card");
    assert!(!e.rejected_cards().unwrap().contains_key("ASH_241"));
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.round = 6;
        s.active = seat;
        s.action_player = Some(seat);
        let source = unit(&e, &mut s, seat, "SOR_130");
        let defender = unit(&e, &mut s, seat.other(), "LOF_254");
        let unrelated = unit(&e, &mut s, seat.other(), "LOF_256");
        let other_attacker = unit(&e, &mut s, seat, "LOF_254");
        s.card_mut(unrelated).damage = 1;
        let mut s = snowtrooper_roundtrip(&e, &s);
        assert_eq!(e.power(&s, source, false), 2);
        assert!(!e.has_keyword(&s, source, "overwhelm"));
        s.attack_source = source;
        s.attack_defender = defender;
        assert_eq!(e.power(&s, source, true), 2, "unrelated damage must not qualify");
        s.card_mut(defender).damage = 1;
        assert_eq!(e.power(&s, source, true), 4);
        assert!(e.has_keyword(&s, source, "overwhelm"));
        s.card_mut(defender).damage = 0;
        assert_eq!(e.power(&s, source, true), 2, "healing removes the live bonus");
        assert!(!e.has_keyword(&s, source, "overwhelm"));
        s.card_mut(defender).damage = 1;
        s.attack_source = other_attacker;
        assert_eq!(e.power(&s, source, false), 2, "another attack does not qualify");
        s.attack_source = source;
        let base = s.player(seat.other()).base;
        s.attack_defender = base;
        s.card_mut(base).damage = 1;
        assert_eq!(e.power(&s, source, true), 2, "a damaged base is not a unit");
        assert!(!e.has_keyword(&s, source, "overwhelm"));
        s.attack_defender = defender;
        e.leave_play_primary(&mut s, defender).unwrap();
        assert_eq!(e.power(&s, source, true), 2, "a departed defender is not in play");
        assert!(!e.has_keyword(&s, source, "overwhelm"));
        s.attack_defender = unrelated;
        assert_eq!(e.power(&s, source, true), 4);
        e.leave_play_primary(&mut s, source).unwrap();
        assert_eq!(e.power(&s, source, true), 2, "a departed source no longer contributes its constant even while attack identity is retained");
        assert!(!e.has_keyword(&s, source, "overwhelm"));
    }
}

#[test]
fn snowtrooper_legal_combat_distinguishes_initial_defender_damage() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        for initial_damage in [0, 1] {
            let mut s = board(&e);
            s.round = 6;
            s.active = seat;
            s.action_player = Some(seat);
            let source = unit(&e, &mut s, seat, "SOR_130");
            let defender = unit(&e, &mut s, seat.other(), "LOF_256");
            let base = s.player(seat.other()).base;
            s.card_mut(defender).damage = initial_damage;
            let s = snowtrooper_roundtrip(&e, &s);
            assert_eq!(e.power(&s, source, false), 2);
            assert!(!e.has_keyword(&s, source, "overwhelm"));
            let attack_action = e.card_action(&s, source);
            assert!(e.legal_actions(&s).contains(&attack_action));
            let choosing = e.apply(&s, &attack_action).unwrap();
            assert_eq!(choosing.decision, Decision::AttackTarget);
            let defender_action = e.card_action(&choosing, defender);
            assert!(e.legal_actions(&choosing).contains(&defender_action));
            let resolved = e.apply(&choosing, &defender_action).unwrap();
            assert_eq!(resolved.card(source).damage, 1);
            assert!(resolved.card(source).exhausted);
            assert_eq!(resolved.attack_source, 0);
            assert_eq!(resolved.attack_defender, 0);
            if initial_damage == 0 {
                assert!(e.in_arena(&resolved, defender));
                assert_eq!(resolved.card(defender).damage, 2);
                assert_eq!(resolved.card(base).damage, 0);
            } else {
                assert!(!e.in_arena(&resolved, defender));
                assert!(resolved.player(seat.other()).discard.contains(defender));
                assert_eq!(resolved.card(base).damage, 1);
            }
            assert_eq!(e.power(&resolved, source, false), 2);
            assert!(!e.has_keyword(&resolved, source, "overwhelm"));
            let restored = snowtrooper_roundtrip(&e, &resolved);
            assert!(!e.legal_actions(&restored).contains(&e.card_action(&restored, source)));
        }
    }
}
