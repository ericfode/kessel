// Source-derived witnesses for the Plot keyword, not forceteki executions.
// Expected outcomes come from the pinned engine sources cited in
// `play_from_resources.rs` (PlotAbility.ts, PlayCardAction.ts, ResourceCost.ts,
// ResourceZone.rearrangeResourceExhaustState) and the recorded engine
// expectations in test/server/core/abilities/keyword/Plot.spec.ts.
use super::*;

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
fn deploy(e: &Engine, s: &State, seat: Seat) -> State {
    let leader = s.player(seat).leader;
    let menu = e.apply(s, &e.card_action(s, leader)).unwrap();
    assert_eq!(menu.decision, Decision::LeaderMenu);
    let action = e
        .legal_actions(&menu)
        .into_iter()
        .find(|a| a.label.starts_with("Deploy "))
        .unwrap_or_else(|| panic!("missing deploy action: {:?}", e.legal_actions(&menu)));
    e.apply(&menu, &action).unwrap()
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
fn labels(e: &Engine, s: &State) -> Vec<String> {
    e.legal_actions(s).into_iter().map(|a| a.label).collect()
}

/// PlotAbility: a plotted card in your resource zone is offered when *your*
/// leader deploys, paid with its own cost, and replaced by the exhausted top
/// card of your deck. `SEC_203` costs 3 printed with cunning+heroism; the test
/// deck (aggression+heroism leader, vigilance base) pays 2 for the unmatched
/// cunning aspect.
#[test]
fn plot_play_from_resources_pays_cost_and_replaces_with_exhausted_top_deck() {
    let e = engine();
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_203");
    let fill = fillers(&e, &mut s, Seat::P1, 10);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::PlotOptional);
    assert_eq!(labels(&e, &after), vec!["Trigger", "Pass"]);
    let played = e
        .apply(&after, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert!(e.in_arena(&played, plot), "plot card enters play");
    assert!(played.card(plot).exhausted, "a played unit enters exhausted");
    assert!(!played.player(Seat::P1).resources.contains(plot));
    // 11 resources - source + replacement = 11. Payment exhausts the source
    // first (5 = 3 printed + 2 aspect penalty), so 4 in-zone resources stay
    // exhausted and the replacement enters exhausted.
    assert_eq!(played.player(Seat::P1).resources.len, 11);
    assert_eq!(exhausted(&played, Seat::P1), 5);
    assert_eq!(ready(&played, Seat::P1), 6);
    for id in fill {
        assert!(played.player(Seat::P1).resources.contains(id));
    }
    assert!(played.player(Seat::P1).resources.contains(top));
    assert!(played.card(top).exhausted, "the replacement resource enters exhausted");
    assert_eq!(played.player(Seat::P1).deck.len, 0);
    assert_eq!(played.plot, PlotState::default());
    assert_eq!(played.decision, Decision::Window);
    assert_eq!(played.active, Seat::P2);
}

/// Plot's replacement is the ability's `ifYouDo`: an empty deck skips it and the
/// play still succeeds (Plot.spec.ts "should work even if there is no card to
/// replace it with from the deck").
#[test]
fn plot_empty_deck_still_plays_without_replacement() {
    let e = engine();
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_203");
    fillers(&e, &mut s, Seat::P1, 10);
    assert_eq!(s.player(Seat::P1).deck.len, 0);
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::PlotOptional);
    let played = e
        .apply(&after, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert!(e.in_arena(&played, plot));
    assert_eq!(played.player(Seat::P1).resources.len, 10);
    assert_eq!(exhausted(&played, Seat::P1), 4);
    assert_eq!(ready(&played, Seat::P1), 6);
    assert_eq!(played.decision, Decision::Window);
    assert_eq!(played.active, Seat::P2);
}

/// The Plot ability is optional: declining leaves the card in resources and does
/// not resource a deck card.
#[test]
fn plot_trigger_is_optional_and_declining_changes_nothing() {
    let e = engine();
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_203");
    fillers(&e, &mut s, Seat::P1, 10);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::PlotOptional);
    let declined = e.apply(&after, &Action::button("Pass", "pass")).unwrap();
    assert!(declined.player(Seat::P1).resources.contains(plot));
    assert!(!declined.card(plot).exhausted);
    assert!(declined.player(Seat::P1).deck.contains(top));
    assert_eq!(declined.player(Seat::P1).resources.len, 11);
    assert_eq!(exhausted(&declined, Seat::P1), 0);
    assert_eq!(declined.decision, Decision::Window);
    // The window closes instead of re-offering the declined trigger.
    assert_eq!(declined.active, Seat::P2);
}

/// A Plot trigger whose cost cannot be paid is not offered at all
/// (Plot.spec.ts "a triggered Plot that cannot be paid should not resource a
/// card"); the engine also never exports a prompt for it.
#[test]
fn plot_unaffordable_is_not_offered() {
    let e = engine();
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_203");
    for id in fillers(&e, &mut s, Seat::P1, 10) {
        s.card_mut(id).exhausted = true;
    }
    s.card_mut(plot).exhausted = true;
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::Window);
    assert_eq!(after.active, Seat::P2);
    assert!(after.player(Seat::P1).resources.contains(plot));
    assert_eq!(after.plot, PlotState::default());
}

/// onLeaderDeployed compares the deployed card's owner with the Plot source's
/// controller, so an opponent's deploy must not offer your Plot cards.
#[test]
fn plot_only_triggers_on_your_own_leader_deploy() {
    let e = engine();
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_203");
    fillers(&e, &mut s, Seat::P1, 3);
    fillers(&e, &mut s, Seat::P2, 8);
    s.active = Seat::P2;
    s.action_player = Some(Seat::P2);
    let after = deploy(&e, &s, Seat::P2);
    assert_eq!(after.decision, Decision::Window);
    assert_eq!(after.active, Seat::P1);
    assert!(after.player(Seat::P1).resources.contains(plot));
    assert!(!after.card(plot).exhausted);
    assert_eq!(after.plot, PlotState::default());
}

/// A card whose metadata mentions Plot but which is not in a resource zone has
/// no plot ability; the keyword instance only exists on the printed own line and
/// only a resource-zone instance can trigger.
#[test]
fn plot_card_outside_the_resource_zone_does_not_trigger() {
    let e = engine();
    let mut s = board(&e);
    let plot = add(&e, &mut s, Seat::P1, "SEC_203");
    s.player_mut(Seat::P1).hand.push(plot).unwrap();
    fillers(&e, &mut s, Seat::P1, 6);
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::Window);
    assert!(after.player(Seat::P1).hand.contains(plot));
    assert_eq!(after.plot, PlotState::default());
}

/// Timing witness from Plot.spec.ts: the deployed leader's own keyword trigger
/// (Shielded) shares the trigger window with the Plot abilities, a plot card's
/// play resolves its replacement before the played card's When Played trigger,
/// and the leader's remaining trigger is offered again afterwards.
#[test]
fn plot_window_shares_deploy_triggers_and_resumes_after_played_triggers() {
    let mut e = engine();
    let leader_def = e.by_code["SOR_014"];
    e.definitions[leader_def as usize].flags |= SHIELDED;
    let mut s = board(&e);
    let plot = resource(&e, &mut s, Seat::P1, "SEC_034"); // When Played: defeat <=2 HP unit
    let fill = fillers(&e, &mut s, Seat::P1, 10);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let victim = unit(&e, &mut s, Seat::P2, "LOF_254"); // 1 HP, so remaining HP <= 2
    let leader = s.player(Seat::P1).leader;
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::TriggerOrder);
    assert_eq!(
        labels(&e, &after),
        vec!["Play Cad Bane using Plot".to_string(), "Shielded".to_string()]
    );
    assert_eq!(e.ordered_trigger_titles(&after).len(), 2);

    // Choose the Plot ability, then Trigger on its optional prompt (cost 5
    // printed + 2 for the unmatched villainy aspect = 7).
    let chosen = e
        .legal_actions(&after)
        .into_iter()
        .find(|a| a.label == "Play Cad Bane using Plot")
        .unwrap();
    let optional = e.apply(&after, &chosen).unwrap();
    assert_eq!(optional.decision, Decision::PlotOptional);
    let played = e
        .apply(&optional, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert!(e.in_arena(&played, plot));
    assert_eq!(exhausted(&played, Seat::P1), 7);
    assert!(played.player(Seat::P1).resources.contains(top));
    assert!(played.card(top).exhausted);
    // The played card's own When Played target prompt comes before the leader's
    // still-pending deploy keyword; replacement already happened.
    assert_eq!(played.decision, Decision::AbilityTarget);
    assert_eq!(played.card(leader).shield, 0);
    let targets: Vec<_> = e.legal_actions(&played).iter().filter_map(Action::id).collect();
    assert_eq!(targets, vec![victim]);
    let resolved = e.apply(&played, &e.card_action(&played, victim)).unwrap();
    assert!(!e.in_arena(&resolved, victim), "When Played defeated the weak unit");
    // The resumed leader window resolves Shielded, then the window closes.
    assert_eq!(resolved.decision, Decision::Window);
    assert_eq!(resolved.active, Seat::P2);
    assert_ne!(resolved.card(leader).shield, 0, "leader Shielded resolved");
    assert_eq!(resolved.plot, PlotState::default());
    for id in fill {
        assert!(resolved.player(Seat::P1).resources.contains(id));
    }
}


/// Printed-text derivation boundaries from the pinned engine parser
/// (KeywordHelpers.getRegexForKeyword / isKeywordEnabled).
#[test]
fn plot_derivation_requires_one_own_line_occurrence() {
    use crate::simulator::play_from_resources::plot_instance;
    assert_eq!(plot_instance(1, &["Raid 1\nPlot (When you deploy a leader.)"]), Ok(true));
    assert_eq!(plot_instance(0, &["Raid 1\nPlot (When you deploy a leader.)"]), Ok(false));
    assert_eq!(plot_instance(1, &["Search for a card with Plot or attack."]), Ok(false));
    // The engine regex is a prefix match at a line start, so "Plotting" counts.
    assert_eq!(plot_instance(1, &["Plotting a course"]), Ok(true));
    assert_eq!(
        plot_instance(1, &["Plot (When you deploy a leader.)"]),
        Ok(true)
    );
    assert!(plot_instance(1, &["Plot (one)\nPlot (two)"])
        .unwrap_err()
        .contains("more than once"));
}

/// Admission boundaries: metadata mentions do not create a Plot instance, a
/// second printed line is rejected, and a leader cannot carry Plot.
#[test]
fn plot_admission_boundaries_are_explicit() {
    let mutate = |code: &str, change: fn(&mut serde_json::Value)| {
        let mut catalogue = committed_catalogue().clone();
        let entry = catalogue["cards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["setCode"] == code)
            .unwrap();
        change(entry);
        Engine::from_catalogue_value(&catalogue)
    };
    // A metadata mention without an own-printed Plot line is not a keyword
    // instance; the card stays admitted and gets no plot ability.
    let mentioned = mutate("SOR_073", |entry| {
        entry["card"]["keywords"] = json!(["plot"]);
        entry["card"]["text"] = json!("Give a shield token to a unit. Search for a card with Plot.");
    })
    .unwrap();
    assert!(mentioned.by_code.contains_key("SOR_073"));
    assert!(!mentioned.definitions[mentioned.by_code["SOR_073"] as usize].plot);
    // Two own-printed Plot lines are an engine error, so admission rejects them.
    let duplicate = mutate("SOR_073", |entry| {
        entry["card"]["keywords"] = json!(["plot"]);
        entry["card"]["text"] = json!("Plot (one)\nPlot (two)");
    })
    .unwrap();
    assert!(duplicate.rejected_cards().unwrap()["SOR_073"].contains("printed Plot appears more than once"));
    // PlotAbility asserts non-leaders, so a printed leader Plot is invalid data.
    let leader = mutate("SOR_014", |entry| {
        entry["card"]["keywords"] = json!(["plot"]);
        entry["card"]["deployBox"] = json!("When Deployed: draw a card.\nPlot");
    })
    .unwrap();
    assert!(leader.rejected_cards().unwrap()["SOR_014"].contains("cannot be printed on a leader"));
}
