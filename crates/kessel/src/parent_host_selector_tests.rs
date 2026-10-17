//! Selector `card.parentCard` host filters for ordinary units that defeat upgrades.
use super::*;

fn attach(e: &Engine, s: &mut State, owner: Seat, code: &str, host: CardId) -> CardId {
    let id = add(e, s, owner, code);
    e.attach(s, id, host);
    id
}

fn program_for(e: &Engine, s: &State, source: CardId, trigger: &str) -> u16 {
    *e.def(s, source)
        .programs
        .card
        .iter()
        .find(|&&p| e.program(p).trigger == trigger)
        .expect(trigger)
}

fn run_optional_defeat(e: &Engine, mut s: State, source: CardId, trigger: &str, chosen: CardId) -> State {
    s.source = source;
    s.active = s.card(source).owner;
    let program = program_for(e, &s, source, trigger);
    e.begin_program(&mut s, program, 0).unwrap();
    assert_eq!(s.decision, Decision::AbilityTarget);
    let actions = e.legal_actions(&s);
    assert!(
        actions.iter().any(|a| a.id() == Some(chosen)),
        "expected {chosen} among {:?}",
        actions.iter().filter_map(|a| a.id()).collect::<Vec<_>>()
    );
    e.apply(&s, &e.card_action(&s, chosen)).unwrap()
}

#[test]
fn parent_host_selector_admits_reviewed_unit_sources() {
    let e = engine();
    for code in ["HMW_270", "LAW_195"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn wild_space_wanderer_defeats_only_upgrades_attached_to_bases() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.round = 5;
        s.active = seat;
        s.action_player = Some(seat);
        let wanderer = unit(&e, &mut s, seat, "HMW_270");
        let host = unit(&e, &mut s, seat, "LOF_254");
        let base = s.player(seat).base;
        let on_unit = attach(&e, &mut s, seat, "SOR_071", host);
        let on_base = attach(&e, &mut s, seat, "SOR_071", base);
        assert!(e.attached_in_play(&s, on_base));
        assert!(e.attached_in_play(&s, on_unit));

        s.source = wanderer;
        s.active = seat;
        let program = program_for(&e, &s, wanderer, "whenPlayed");
        e.begin_program(&mut s, program, 0).unwrap();
        assert_eq!(s.decision, Decision::AbilityTarget);
        let actions = e.legal_actions(&s);
        assert!(actions.iter().any(|a| a.id() == Some(on_base)));
        assert!(!actions.iter().any(|a| a.id() == Some(on_unit)));

        let done = e.apply(&s, &e.card_action(&s, on_base)).unwrap();
        assert_eq!(done.card(on_base).parent, 0);
        assert!(done.player(seat).discard.contains(on_base));
        assert_eq!(done.card(on_unit).parent, host);
    }
}

#[test]
fn overcharged_transport_defeats_only_space_host_upgrades() {
    let e = engine();
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.round = 5;
        s.active = seat;
        s.action_player = Some(seat);
        let transport = unit(&e, &mut s, seat, "LAW_195");
        let ground = unit(&e, &mut s, seat, "LOF_254");
        let space = unit(&e, &mut s, seat, "TOKEN_XWING");
        let on_ground = attach(&e, &mut s, seat, "SOR_071", ground);
        let on_space = attach(&e, &mut s, seat, "SOR_071", space);

        for trigger in ["whenPlayed", "whenDefeated"] {
            let mut prompt = s.clone();
            prompt.source = transport;
            prompt.active = seat;
            let program = program_for(&e, &prompt, transport, trigger);
            e.begin_program(&mut prompt, program, 0).unwrap();
            assert_eq!(prompt.decision, Decision::AbilityTarget, "{trigger}");
            let actions = e.legal_actions(&prompt);
            assert!(
                actions.iter().any(|a| a.id() == Some(on_space)),
                "{trigger} space"
            );
            assert!(
                !actions.iter().any(|a| a.id() == Some(on_ground)),
                "{trigger} ground"
            );
        }

        let done = run_optional_defeat(&e, s, transport, "whenPlayed", on_space);
        assert_eq!(done.card(on_space).parent, 0);
        assert!(done.player(seat).discard.contains(on_space));
        assert_eq!(done.card(on_ground).parent, ground);
    }
}

#[test]
fn card_parent_paths_validate_while_source_parent_still_requires_upgrade_class() {
    let card_parent: crate::ability_ir::Expr = serde_json::from_value(json!({
        "kind": "ref",
        "path": ["card", "parentCard", "zoneName"]
    }))
    .unwrap();
    assert!(card_parent.validate().is_ok());
    let is_base: crate::ability_ir::Expr = serde_json::from_value(json!({
        "kind": "query",
        "name": "isBase",
        "receiver": {"kind": "ref", "path": ["card", "parentCard"]},
        "args": []
    }))
    .unwrap();
    assert!(is_base.validate().is_ok());

    let catalog: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let mut entry = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "HMW_270")
        .unwrap()
        .clone();
    entry["abilities"]["card"][0]["targetResolver"]["predicate"]["right"]["receiver"]["path"] =
        json!(["source", "parentCard"]);
    let loaded = crate::ability_ir::Catalogue::load(&json!({
        "schema": crate::ability_ir::SCHEMA,
        "cards": [entry]
    }))
    .unwrap();
    assert!(
        !loaded.cards.contains_key("HMW_270"),
        "source.parentCard must still require UpgradeCard"
    );
}
