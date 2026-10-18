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
    //
    // CONVENTION (hidden information, not engine-determined): forceteki shuffles
    // the resource zone in rearrangeResourceExhaustState when a Plot play pays and
    // reassigns which resources are exhausted. CR 1.7.4 makes that identity a
    // legal player choice, so Kessel fixes one legal deterministic assignment
    // (played card first, then Kessel zone order, counts preserved, no reorder).
    // The counts below assert that convention, not parity with a recording.
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
    // CONVENTION: the exhausted counts use Kessel's deterministic hidden-resource
    // assignment (see the payment/replacement test above), not the engine shuffle.
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

/// SYNTHETIC diagnostic (no printed Plot event is admitted, so the dormant plot
/// event continuation is exercised by marking an admitted event's Definition):
/// PlayEventAction resolves a mandatory event ability's targets before the play
/// cost, the nested plot play has no Cancel, and the plot `ifYouDo` replacement
/// happens after the effect.
#[test]
fn plot_event_resolves_early_target_before_payment_without_cancel() {
    let mut e = engine();
    let def = e.by_code["SOR_073"]; // 1-cost vigilance event, mandatory unit target
    e.definitions[def as usize].plot = true;
    let mut s = board(&e);
    let event = resource(&e, &mut s, Seat::P1, "SOR_073");
    fillers(&e, &mut s, Seat::P1, 3);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let target = unit(&e, &mut s, Seat::P1, "LOF_254");
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::PlotOptional);
    let prompt = e
        .apply(&after, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    // The plot cost (1, vigilance matched by the base) is paid only after the
    // target is chosen; the card is still in the resource zone at the prompt.
    assert!(prompt.player(Seat::P1).resources.contains(event));
    assert_eq!(exhausted(&prompt, Seat::P1), 0);
    assert!(prompt.player(Seat::P1).deck.contains(top));
    let labels = labels(&e, &prompt);
    assert!(
        !labels.iter().any(|label| label == "Cancel"),
        "the nested plot play is not cancellable: {labels:?}"
    );
    assert!(
        e.apply(&prompt, &Action::button("Cancel", "cancel")).is_err(),
        "Cancel is not a legal plot target action"
    );
    let resolved = e.apply(&prompt, &e.card_action(&prompt, target)).unwrap();
    assert_ne!(resolved.card(target).shield, 0, "plotted event gave a Shield");
    assert!(resolved.player(Seat::P1).discard.contains(event));
    assert!(!resolved.player(Seat::P1).resources.contains(event));
    assert!(resolved.player(Seat::P1).resources.contains(top));
    assert!(resolved.card(top).exhausted);
    // CONVENTION: source-first payment then the ifYouDo replacement, using
    // Kessel's deterministic hidden-resource assignment.
    assert_eq!(resolved.player(Seat::P1).resources.len, 4);
    assert_eq!(exhausted(&resolved, Seat::P1), 1);
    assert_eq!(resolved.decision, Decision::Window);
    assert_eq!(resolved.active, Seat::P2);
    assert_eq!(resolved.plot, PlotState::default());
}

/// SYNTHETIC diagnostic (every printed Plot upgrade stays rejected, so this marks
/// an admitted plain upgrade's Definition as Plot): PlayUpgradeAction resolves the
/// attach target before paying, the nested plot continuation has no Cancel, and a
/// plot upgrade can therefore never be abandoned after payment into no zone.
#[test]
fn plot_upgrade_attach_is_mandatory_and_pays_on_the_chosen_target() {
    let mut e = engine();
    let def = e.by_code["HMW_235"]; // 1-cost cunning upgrade, no abilities
    e.definitions[def as usize].plot = true;
    let mut s = board(&e);
    let upgrade = resource(&e, &mut s, Seat::P1, "HMW_235");
    fillers(&e, &mut s, Seat::P1, 5);
    let top = deck_top(&e, &mut s, Seat::P1, "LOF_256");
    let leader = s.player(Seat::P1).leader;
    let after = deploy(&e, &s, Seat::P1);
    assert_eq!(after.decision, Decision::PlotOptional);
    let prompt = e
        .apply(&after, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert_eq!(prompt.decision, Decision::UpgradeTarget);
    assert!(
        prompt.player(Seat::P1).resources.contains(upgrade),
        "the attach target is chosen before the plot cost is paid"
    );
    assert_eq!(exhausted(&prompt, Seat::P1), 0);
    let labels = labels(&e, &prompt);
    assert!(
        !labels.iter().any(|label| label == "Cancel"),
        "the plot upgrade attach is not cancellable: {labels:?}"
    );
    assert!(
        e.apply(&prompt, &Action::button("Cancel", "cancel")).is_err(),
        "a paid plot upgrade can never be abandoned into a cardless state"
    );
    let attached = e.apply(&prompt, &e.card_action(&prompt, leader)).unwrap();
    assert!(e.upgrades(&attached, leader).contains(&upgrade));
    assert!(!attached.player(Seat::P1).resources.contains(upgrade));
    assert!(attached.player(Seat::P1).resources.contains(top));
    assert!(attached.card(top).exhausted);
    // Cost 3 = 1 printed + 2 for the unmatched cunning aspect; source first then
    // two ready fillers, then the replacement (CONVENTION hidden-resource layout).
    assert_eq!(attached.player(Seat::P1).resources.len, 6);
    assert_eq!(exhausted(&attached, Seat::P1), 3);
    assert_eq!(attached.decision, Decision::Window);
    assert_eq!(attached.active, Seat::P2);
}
