use super::*;

#[test]
fn active_side_transitions_preserve_base_buffs_and_printed_raid() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = basezone3_board(&e, seat, "TWI_019");
        let leader = s.player(seat).leader;
        let base = s.player(seat).base;
        let friendly = unit(&e, &mut s, seat, "ASH_261");
        let enemy = unit(&e, &mut s, seat.other(), "ASH_261");
        let front_programs = e.card_programs(&s, leader).to_vec();
        let base_programs = e.card_programs(&s, base).to_vec();
        assert_eq!(front_programs, e.def(&s, leader).programs.leader);
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 1);
        assert_eq!(e.keyword_amount(&s, enemy, "raid"), 0);

        s.card_mut(leader).deployed = true;
        e.add_arena(&mut s, leader).unwrap();
        assert_eq!(e.card_programs(&s, leader), e.def(&s, leader).programs.unit);
        assert_ne!(e.card_programs(&s, leader), front_programs);
        assert_eq!(e.card_programs(&s, base), base_programs);
        assert_eq!(e.hp(&s, leader), 6);
        assert_eq!(e.keyword_amount(&s, leader, "raid"), 1, "own grant must exclude printed Raid source");
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 1);

        s.disabled_ongoing_source = leader;
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 0);
        assert_eq!(e.keyword_amount(&s, leader, "raid"), 1, "disabling aura must retain printed Raid");
        assert_eq!(e.hp(&s, leader), 6, "base activation is independent of leader activation");
        s.disabled_ongoing_source = 0;
        e.defeat(&mut s, leader).unwrap();
        assert!(!s.card(leader).deployed);
        assert_eq!(e.card_programs(&s, leader), front_programs);
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 1, "front grant reactivates after defeat");
        assert_eq!(e.keyword_amount(&s, enemy, "raid"), 0);
    }
}

#[test]
fn side_selection_does_not_invent_base_zone_placement_or_unique_visibility() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = basezone3_board(&e, seat, "TWI_028");
        let leader = s.player(seat).leader;
        let friendly = unit(&e, &mut s, seat, "ASH_261");
        let front = e.card_programs(&s, leader).to_vec();
        // A physical card can retain its definition and active side after it
        // leaves the player slot; that alone must not keep the aura active.
        s.player_mut(seat).leader = 0;
        assert_eq!(e.card_programs(&s, leader), front);
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 0);
        s.player_mut(seat).leader = leader;
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 1);
        s.decision = Decision::Unique;
        s.source = leader;
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 0);
        s.decision = Decision::Window;
        assert_eq!(e.keyword_amount(&s, friendly, "raid"), 1);
    }
}
