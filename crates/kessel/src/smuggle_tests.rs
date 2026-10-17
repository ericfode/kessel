// Source-derived witnesses for the Smuggle keyword, not forceteki executions.
// Expected outcomes come from the pinned engine sources cited in
// `play_from_resources.rs` (PlayableOrDeployableCard.getPlayCardActions,
// buildCheapestAlternatePlayAction, PlayCardResourceCost, Player.exhaustResources,
// ResourceZone.rearrangeResourceExhaustState, PlayUnit/Upgrade/EventAction) and
// test/server/core/abilities/keyword/Smuggle.spec.ts.
use super::*;
use crate::simulator::play_from_resources::SmuggleCost;

fn resource(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).resources.push(id).unwrap();
    id
}
fn fillers(e: &Engine, s: &mut State, seat: Seat, count: usize) -> Vec<CardId> {
    (0..count).map(|_| resource(e, s, seat, "LOF_254")).collect()
}
fn deck_top(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).deck.push(id).unwrap();
    id
}
fn exhausted(s: &State, seat: Seat) -> usize {
    s.player(seat)
        .resources
        .as_slice()
        .iter()
        .filter(|&&id| s.card(id).exhausted)
        .count()
}
fn ready(s: &State, seat: Seat) -> usize {
    s.player(seat).resources.len as usize - exhausted(s, seat)
}
fn can_play(e: &Engine, s: &State, id: CardId) -> bool {
    e.legal_actions(s).contains(&e.card_action(s, id))
}

/// A resource-zone Smuggle card is a legal play action and is paid with the
/// keyword's cost and aspect list. `SHD_032` prints 4 with villainy+vigilance but
/// smuggles for 5 with vigilance+villainy; against the test deck one villainy
/// aspect is unmatched, so the play costs 7.
#[test]
fn smuggle_plays_unit_from_resources_for_smuggle_cost() {
    let e = engine();
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_032");
    let fill = fillers(&e, &mut s, Seat::P1, 10);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    assert!(can_play(&e, &s, source), "resource-zone Smuggle is a play action");
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(e.in_arena(&played, source));
    assert!(played.card(source).exhausted, "a played unit enters exhausted");
    assert!(!played.player(Seat::P1).resources.contains(source));
    // 11 resources - source + replacement. The exhausted source pays first, then
    // six ready resources; the replacement enters exhausted.
    assert_eq!(played.player(Seat::P1).resources.len, 11);
    assert_eq!(exhausted(&played, Seat::P1), 7);
    assert_eq!(ready(&played, Seat::P1), 4);
    assert!(played.player(Seat::P1).resources.contains(top));
    assert!(played.card(top).exhausted);
    assert_eq!(played.player(Seat::P1).deck.len, 0);
    assert_eq!(played.decision, Decision::Window);
    assert_eq!(played.active, Seat::P2);
    // Deterministic layout convention: the zone is not reordered, the played card
    // is removed, and the replacement is appended.
    let zone: Vec<_> = played
        .player(Seat::P1)
        .resources
        .as_slice()
        .iter()
        .copied()
        .collect();
    let mut expected = fill.clone();
    expected.push(top);
    assert_eq!(zone, expected);
    for (index, id) in fill.iter().enumerate() {
        assert_eq!(played.card(*id).exhausted, index < 6, "zone-order payment");
    }
}

/// Smuggle pays the keyword cost, not the printed cost: `SHD_050` prints 8 but
/// smuggles for 9 (aggression+heroism), both aspects matched by the test deck.
#[test]
fn smuggle_pays_keyword_cost_not_printed_cost() {
    let e = engine();
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_050");
    fillers(&e, &mut s, Seat::P1, 10);
    deck_top(&e, &mut s, Seat::P1, "LOF_256");
    assert!(can_play(&e, &s, source));
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(e.in_arena(&played, source));
    assert_eq!(exhausted(&played, Seat::P1), 9);
}

/// Synthetic diagnostic (not a card witness): the resource cost must be computed
/// from the keyword's aspect multiset, not the card's printed aspects. This card's
/// printed aspect is aggression; replacing only the derived keyword instance with
/// two cunning aspects makes both unmatched (4 penalty), so the play costs 3+4=7
/// although the printed cost would be 1+0=1.
#[test]
fn smuggle_cost_uses_keyword_aspect_multiset() {
    let mut e = engine();
    let def = e.by_code["SHD_160"];
    e.definitions[def as usize].smuggle = Some(SmuggleCost {
        cost: 3,
        aspects: [0, 0, 0, 2, 0, 0],
    });
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_160");
    fillers(&e, &mut s, Seat::P1, 8);
    deck_top(&e, &mut s, Seat::P1, "LOF_256");
    assert!(can_play(&e, &s, source));
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(e.in_arena(&played, source));
    assert_eq!(exhausted(&played, Seat::P1), 7);
}

/// Smuggle is only an alternate play for a card in the resource zone. The same
/// card in hand pays its printed cost instead: `SHD_097` costs 4 from hand in this
/// deck (2 + unmatched command) but 6 when smuggled, so four ready resources allow
/// the hand play; moving the card into the resource row raises the ready count to
/// five, still short of the smuggled cost.
#[test]
fn smuggle_from_hand_is_not_a_resource_play() {
    let e = engine();
    let mut s = board(&e);
    let card = add(&e, &mut s, Seat::P1, "SHD_097");
    s.player_mut(Seat::P1).hand.push(card).unwrap();
    fillers(&e, &mut s, Seat::P1, 4);
    assert!(can_play(&e, &s, card), "hand play for the printed cost");
    s.player_mut(Seat::P1).hand.remove(card);
    s.player_mut(Seat::P1).resources.push(card).unwrap();
    assert_eq!(ready(&s, Seat::P1), 5);
    assert!(
        !can_play(&e, &s, card),
        "the smuggled cost 6 is unaffordable with five ready resources"
    );
    // A sixth ready resource makes the smuggled play legal.
    let extra = add(&e, &mut s, Seat::P1, "LOF_254");
    s.player_mut(Seat::P1).resources.push(extra).unwrap();
    assert!(can_play(&e, &s, card));
}

/// Non-Smuggle resource cards are not playable from the resource zone; a ready
/// Smuggle source is only affordable while the ready count covers its cost.
#[test]
fn smuggle_requires_an_affordable_smuggle_source() {
    let e = engine();
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_160"); // smuggle 3 aggression
    let first = resource(&e, &mut s, Seat::P1, "LOF_254");
    let second = resource(&e, &mut s, Seat::P1, "LOF_254");
    assert_eq!(ready(&s, Seat::P1), 3);
    assert!(can_play(&e, &s, source));
    assert!(!can_play(&e, &s, first), "plain resources are not playable");
    assert!(!can_play(&e, &s, second));
    s.card_mut(source).exhausted = true;
    assert_eq!(ready(&s, Seat::P1), 2);
    assert!(!can_play(&e, &s, source), "an exhausted source cannot pay itself");
    let third = resource(&e, &mut s, Seat::P1, "LOF_254");
    assert_eq!(ready(&s, Seat::P1), 3);
    assert!(can_play(&e, &s, source));
    assert!(!can_play(&e, &s, third));
}

/// Smuggle's replacement is a sibling event of the play, so it happens before the
/// played card's When Played trigger. `SHD_050`'s When Played target prompt must
/// already see the replacement resource, and an empty deck must skip it.
#[test]
fn smuggle_replaces_before_when_played_and_skips_an_empty_deck() {
    let e = engine();
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_050");
    fillers(&e, &mut s, Seat::P1, 10);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let victim = unit(&e, &mut s, Seat::P2, "LOF_256"); // 4 HP <= 5
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(played.decision, Decision::AbilityTarget);
    assert!(played.player(Seat::P1).resources.contains(top));
    assert!(played.card(top).exhausted);
    assert_eq!(exhausted(&played, Seat::P1), 9);
    let resolved = e.apply(&played, &e.card_action(&played, victim)).unwrap();
    assert!(!e.in_arena(&resolved, victim));
    assert_eq!(resolved.decision, Decision::Window);
    assert_eq!(resolved.active, Seat::P2);

    // Empty deck: the play succeeds without a replacement and the resource count
    // shrinks by the played card.
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_160");
    let fill = fillers(&e, &mut s, Seat::P1, 3);
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(e.in_arena(&played, source));
    assert_eq!(played.player(Seat::P1).resources.len, 3);
    assert_eq!(exhausted(&played, Seat::P1), 2);
    assert_eq!(
        played.player(Seat::P1).resources.as_slice(),
        fill.as_slice()
    );
}

/// A smuggled event pays, moves to the discard pile and replaces itself before
/// its effect resolves; the deck's top card becomes an exhausted resource.
#[test]
fn smuggle_event_pays_replaces_and_resolves() {
    let e = engine();
    let mut s = board(&e);
    let source = resource(&e, &mut s, Seat::P1, "SHD_252"); // heal 3 base, smuggle 3 heroism
    let fill = fillers(&e, &mut s, Seat::P1, 3);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let base = s.player(Seat::P1).base;
    s.card_mut(base).damage = 10;
    assert!(can_play(&e, &s, source));
    let played = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(played.card(base).damage, 7);
    assert!(played.player(Seat::P1).discard.contains(source));
    assert!(!played.player(Seat::P1).resources.contains(source));
    assert!(played.player(Seat::P1).resources.contains(top));
    assert!(played.card(top).exhausted);
    assert_eq!(played.player(Seat::P1).resources.len, 4);
    assert_eq!(exhausted(&played, Seat::P1), 3);
    assert_eq!(
        played.player(Seat::P1).resources.as_slice()[..3],
        fill[..]
    );
    assert_eq!(played.decision, Decision::Window);
    assert_eq!(played.active, Seat::P2);
}
