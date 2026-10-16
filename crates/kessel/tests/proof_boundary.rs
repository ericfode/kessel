//! Compile the real prover with only two-player identity, serde and the clock.
//! This is intentionally a separate crate with no simulator/model/IR modules:
//! adding a rules dependency to search.rs or seat.rs makes this target fail.
#![allow(dead_code)]

#[path = "../src/search.rs"]
mod search;
#[path = "../src/seat.rs"]
mod seat;

#[test]
fn participant_wire_identity_stays_compatible() {
    use seat::Seat;
    for (participant, wire) in [(Seat::P1, "p1"), (Seat::P2, "p2")] {
        assert_eq!(serde_json::to_value(participant).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<Seat>(wire.into()).unwrap(),
            participant
        );
        assert_eq!(Seat::parse(wire).unwrap(), participant);
        assert_eq!(participant.other().other(), participant);
    }
    assert!(Seat::parse("P1").is_err());
    assert!(serde_json::from_value::<Seat>("p3".into()).is_err());
}
