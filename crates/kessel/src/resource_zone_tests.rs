use super::*;

// Source-derived witnesses for Resource-zone targeting. Printed semantics come
// from the pinned engine: a targetResolver with `zoneFilter: ZoneName.Resource`
// and `controller: RelativePlayer.Self` selects the player's physical resource
// cards; DefeatCardSystem and MoveCardSystem(returnToHand) both run
// CardTargetSystem.leavesResourceZoneEventHandler first, which swaps the ready
// state of a ready friendly resource with the first exhausted resource before
// the card leaves the zone.

fn play(e: &Engine, s: &mut State, card: CardId) -> State {
    let a = e.card_action(s, card);
    assert!(e.legal_actions(s).contains(&a), "play action for {card}");
    e.apply(s, &a).unwrap()
}
fn register(e: &mut Engine, s: &State, source: CardId, value: Value) -> u16 {
    register_keyword_program(e, s, source, value);
    *e.def(s, source).programs.card.last().unwrap()
}
fn begin(e: &Engine, s: &mut State, source: CardId, program: u16) {
    s.source = source;
    e.begin_program(s, program, ProgramContinuation::Finish).unwrap();
}
fn resource(e: &Engine, s: &mut State, seat: Seat, exhausted: bool) -> CardId {
    let id = add(e, s, seat, "LOF_254");
    s.player_mut(seat).resources.push(id).unwrap();
    s.card_mut(id).exhausted = exhausted;
    id
}
fn ready_resources(e: &Engine, s: &State, seat: Seat) -> usize {
    s.player(seat)
        .resources
        .as_slice()
        .iter()
        .filter(|&&id| !s.card(id).exhausted)
        .count()
}

#[test]
fn resource_zone_family_admission_and_remaining_explicit_rejections() {
    let e = engine();
    assert!(
        e.by_code.contains_key("SHD_154"),
        "wrecker: {:?}",
        e.rejected_cards().unwrap().get("SHD_154")
    );
    // The two other printed resource-zone sources stay rejected on their next
    // unsupported semantic, not on the zone itself.
    let rejected = e.rejected_cards().unwrap();
    assert!(
        rejected["SHD_214"].contains("nested continuation costs, optional triggers"),
        "{}",
        rejected["SHD_214"]
    );
    assert!(
        rejected["SOR_197"].contains("event group contains an unverified operation"),
        "{}",
        rejected["SOR_197"]
    );
    let compile = |value: Value| {
        let ability: crate::ability_ir::Ability = serde_json::from_value(value).unwrap();
        crate::ability_ir::Program::compile(&ability)
    };
    let return_one = json!({"trigger":"whenPlayed","title":"Return a resource","targetResolver":{"owner":"self","type":"any","zone":"resource"},"effect":{"kind":"returnToHand","target":"chosen"}});
    assert!(compile(return_one.clone()).is_ok());
    let mut typed = return_one.clone();
    typed["targetResolver"]["type"] = json!("unit");
    let err = compile(typed).unwrap_err();
    assert!(err.contains("any card type"), "{err}");
    let mut arena = return_one.clone();
    arena["targetResolver"]["arena"] = json!("ground");
    let err = compile(arena).unwrap_err();
    assert!(err.contains("without an arena filter"), "{err}");
}

#[test]
fn wrecker_defeats_a_ready_resource_with_the_engine_ready_state_swap() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let wrecker = hand(&e, &mut s, "SHD_154");
    let enemy_ground = unit(&e, &mut s, Seat::P2, "HMW_093");
    let enemy_space = unit(&e, &mut s, Seat::P2, "LOF_069");
    let enemy_resource = resource(&e, &mut s, Seat::P2, false);
    let played = play(&e, &mut s, wrecker);
    // Wrecker costs 6 of the 8 test resources: ids 0..6 exhausted, 6..8 ready.
    let resources = played.player(Seat::P1).resources.as_slice().to_vec();
    assert_eq!(resources.len(), 8);
    let ready: Vec<_> = resources.iter().copied().filter(|&id| !played.card(id).exhausted).collect();
    assert_eq!(ready.len(), 2, "spend exhausts the first ready resources in zone order");
    assert_eq!(played.decision, Decision::AbilityTarget);
    let ids: Vec<_> = e.legal_actions(&played).iter().filter_map(Action::id).collect();
    assert!(ids.contains(&ready[1]), "friendly resources are candidates");
    assert!(!ids.contains(&enemy_resource), "the controller filter excludes enemy resources");
    assert!(!ids.contains(&enemy_ground) && !ids.contains(&enemy_space), "arena cards are not resources");
    let chosen = ready[1];
    let first_exhausted = resources[0];
    let prompt = e.apply(&played, &e.card_action(&played, chosen)).unwrap();
    // The ready resource leaves through the swap: it was exhausted, the first
    // exhausted resource became ready, then the chosen card moved to discard.
    assert!(prompt.player(Seat::P1).discard.contains(chosen));
    assert!(prompt.card(chosen).exhausted, "the departing ready resource was exhausted by the swap");
    assert!(!prompt.card(first_exhausted).exhausted, "the first exhausted resource became ready");
    assert!(!prompt.player(Seat::P1).resources.contains(chosen));
    // The swap exchanges two readiness states and the departing card was just
    // exhausted, so the total number of ready resources is unchanged.
    assert_eq!(ready_resources(&e, &prompt, Seat::P1), 2);
    // If you do: a ground unit of either player is selectable, space is not.
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let ids: Vec<_> = e.legal_actions(&prompt).iter().filter_map(Action::id).collect();
    assert!(ids.contains(&enemy_ground));
    assert!(ids.contains(&wrecker), "Wrecker is itself a ground unit");
    assert!(!ids.contains(&enemy_space));
    let done = e.apply(&prompt, &e.card_action(&prompt, enemy_ground)).unwrap();
    assert_eq!(done.card(enemy_ground).damage, 5);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn wrecker_choosing_an_exhausted_resource_does_not_swap_and_decline_does_nothing() {
    let e = engine();
    // Choose an already exhausted resource: no ready state moves.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let wrecker = hand(&e, &mut s, "SHD_154");
    let enemy_ground = unit(&e, &mut s, Seat::P2, "HMW_093");
    let played = play(&e, &mut s, wrecker);
    let resources = played.player(Seat::P1).resources.as_slice().to_vec();
    let ready_before = ready_resources(&e, &played, Seat::P1);
    let chosen = resources[2];
    assert!(played.card(chosen).exhausted);
    let prompt = e.apply(&played, &e.card_action(&played, chosen)).unwrap();
    assert!(prompt.player(Seat::P1).discard.contains(chosen));
    assert_eq!(ready_resources(&e, &prompt, Seat::P1), ready_before);
    assert!(prompt.card(resources[0]).exhausted, "an exhausted choice never swaps");
    let done = e.apply(&prompt, &e.card_action(&prompt, enemy_ground)).unwrap();
    assert_eq!(done.card(enemy_ground).damage, 5);
    // Decline the optional ability: no resource leaves and no damage follows.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let wrecker = hand(&e, &mut s, "SHD_154");
    let enemy_ground = unit(&e, &mut s, Seat::P2, "HMW_093");
    let played = play(&e, &mut s, wrecker);
    let resources_before = played.player(Seat::P1).resources.len;
    let done = named_button(&e, &played, "Pass");
    assert_eq!(done.player(Seat::P1).resources.len, resources_before);
    assert_eq!(done.card(enemy_ground).damage, 0);
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn return_to_hand_on_a_resource_uses_the_same_readiness_swap() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let ready = resource(&e, &mut s, Seat::P1, false);
    let exhausted = resource(&e, &mut s, Seat::P1, true);
    let mut e = e;
    let program = register(&mut e, &s, source, json!({
        "trigger":"whenPlayed","title":"Return a resource you control",
        "targetResolver":{"owner":"self","type":"any","zone":"resource"},
        "effect":{"kind":"returnToHand","target":"chosen"}
    }));
    let mut s = s;
    begin(&e, &mut s, source, program);
    assert_eq!(s.decision, Decision::AbilityTarget);
    let done = e.apply(&s, &e.card_action(&s, ready)).unwrap();
    assert!(done.player(Seat::P1).hand.contains(ready));
    assert!(!done.player(Seat::P1).resources.contains(ready));
    assert!(done.card(ready).exhausted, "departing ready resource was exhausted");
    assert!(!done.card(exhausted).exhausted, "first exhausted resource readied");
    assert_eq!(ready_resources(&e, &done, Seat::P1), 1);
    assert_eq!(done.decision, Decision::Window);
    // Without any exhausted resource there is no swap partner: the ready card
    // simply leaves and the ready count drops by one.
    let mut s2 = board(&e);
    s2.round = 5;
    s2.active = Seat::P1;
    s2.action_player = Some(Seat::P1);
    let source2 = unit(&e, &mut s2, Seat::P1, "LOF_254");
    let only = resource(&e, &mut s2, Seat::P1, false);
    let program2 = register(&mut e, &s2, source2, json!({
        "trigger":"whenPlayed","title":"Return a resource you control",
        "targetResolver":{"owner":"self","type":"any","zone":"resource"},
        "effect":{"kind":"returnToHand","target":"chosen"}
    }));
    begin(&e, &mut s2, source2, program2);
    assert_eq!(ready_resources(&e, &s2, Seat::P1), 1);
    let done2 = e.apply(&s2, &e.card_action(&s2, only)).unwrap();
    assert!(done2.player(Seat::P1).hand.contains(only));
    assert_eq!(ready_resources(&e, &done2, Seat::P1), 0);
}

#[test]
fn return_to_hand_of_an_exhausted_resource_leaves_readiness_untouched() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let ready = resource(&e, &mut s, Seat::P1, false);
    let exhausted = resource(&e, &mut s, Seat::P1, true);
    let mut e = e;
    let program = register(&mut e, &s, source, json!({
        "trigger":"whenPlayed","title":"Return a resource you control",
        "targetResolver":{"owner":"self","type":"any","zone":"resource"},
        "effect":{"kind":"returnToHand","target":"chosen"}
    }));
    let mut s = s;
    begin(&e, &mut s, source, program);
    let before = ready_resources(&e, &s, Seat::P1);
    let done = e.apply(&s, &e.card_action(&s, exhausted)).unwrap();
    assert!(done.player(Seat::P1).hand.contains(exhausted));
    assert_eq!(ready_resources(&e, &done, Seat::P1), before);
    assert!(!done.card(ready).exhausted);
}

#[test]
fn resource_departure_swap_only_applies_to_resources_the_ability_player_controls() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    let enemy_ready = resource(&e, &mut s, Seat::P2, false);
    let enemy_exhausted = resource(&e, &mut s, Seat::P2, true);
    let mut e = e;
    let program = register(&mut e, &s, source, json!({
        "trigger":"whenPlayed","title":"Defeat an enemy resource",
        "targetResolver":{"owner":"opponent","type":"any","zone":"resource"},
        "effect":{"kind":"defeat","target":"chosen"}
    }));
    let mut s = s;
    begin(&e, &mut s, source, program);
    let done = e.apply(&s, &e.card_action(&s, enemy_ready)).unwrap();
    assert!(done.player(Seat::P2).discard.contains(enemy_ready));
    assert!(done.card(enemy_exhausted).exhausted, "foreign controller: no ready-state swap");
    assert_eq!(done.decision, Decision::Window);
}

#[test]
fn resource_departure_swap_guards_on_the_ability_context_player() {
    // Pinned `CardTargetSystem.leavesResourceZoneEventHandler` returns unless the
    // resource's controller equals `context.player`. A Bounty program's context player
    // is the collector (the opponent of the defeated unit's controller), and the
    // admitted Bounty effect set has no resource-zone move, so this witnesses the
    // runtime boundary with that synthetic program context rather than an admitted
    // card: with the source-controller guard both directions below swap wrongly.
    let e = engine();
    let mut base = board(&e);
    base.round = 5;
    let source = unit(&e, &mut base, Seat::P1, "SHD_027");
    let own_ready = resource(&e, &mut base, Seat::P1, false);
    let own_exhausted = resource(&e, &mut base, Seat::P1, true);
    let collector_ready = resource(&e, &mut base, Seat::P2, false);
    let collector_exhausted = resource(&e, &mut base, Seat::P2, true);
    let bounty = *e
        .def(&base, source)
        .programs
        .card
        .iter()
        .find(|&&program| e.program(program).trigger == "bounty")
        .unwrap();
    let in_bounty_context = |s: State, target: CardId| {
        let mut s = s;
        s.source = source;
        s.ability_target = target;
        s.ability_program = bounty + 1; // running program index + 1
        s
    };
    let move_effect = crate::ability_ir::Effect::ReturnToHand {
        target: crate::ability_ir::Target::Name("chosen".into()),
    };
    // A ready resource in the collector's zone: the guard passes, and the readiness
    // swap stays inside that same player's resource zone.
    let mut collector = in_bounty_context(base, collector_ready);
    assert_eq!(e.apply_effect(&mut collector, &move_effect).unwrap(), (1, true));
    assert!(collector.player(Seat::P2).hand.contains(collector_ready));
    assert!(collector.card(collector_ready).exhausted, "the departing resource is exhausted");
    assert!(!collector.card(collector_exhausted).exhausted, "the collector's exhausted resource readied");
    assert!(!collector.card(own_ready).exhausted, "the source controller's resources are untouched");
    assert!(collector.card(own_exhausted).exhausted);
    // A ready resource of the defeated unit's controller: not the context player, so
    // the engine returns before swapping. The source-controller guard would have
    // exchanged readiness with own_exhausted here.
    let mut owner = in_bounty_context(base, own_ready);
    assert_eq!(e.apply_effect(&mut owner, &move_effect).unwrap(), (1, true));
    assert!(owner.player(Seat::P1).hand.contains(own_ready));
    assert!(!owner.card(own_ready).exhausted, "no readiness swap outside the context player");
    assert!(owner.card(own_exhausted).exhausted, "the source controller's exhausted resource stays exhausted");
    // Ordinary context (no Bounty program running): the source controller is the
    // ability player, so the reviewed family-D behavior is unchanged.
    let mut ordinary = base;
    ordinary.source = source;
    ordinary.ability_target = own_ready;
    assert_eq!(e.apply_effect(&mut ordinary, &move_effect).unwrap(), (1, true));
    assert!(ordinary.card(own_ready).exhausted, "ordinary departure still exhausts the ready resource");
    assert!(!ordinary.card(own_exhausted).exhausted, "ordinary departure still readies the first exhausted resource");
}
