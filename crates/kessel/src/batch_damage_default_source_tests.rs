use super::*;
use serde_json::{json, Value};

fn engine() -> Engine {
    Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card { present: true, def: *e.by_code.get(code).unwrap(), owner: seat, ..Card::default() };
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn board(e: &Engine, actor: Seat, resources: usize) -> State {
    let mut s = State::default();
    // This builder creates a fresh position with an explicitly empty Credit ledger.
    s.credit_state_exact=e.needs_credit_state();
    s.round = 6;
    s.hidden_history_exact = true;
    s.force_state_exact = true;
    for (seat, leader) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let leader = add(e, &mut s, seat, leader);
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = leader;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    for _ in 0..resources {
        let id = add(e, &mut s, actor, "LOF_254");
        s.player_mut(actor).resources.push(id).unwrap();
    }
    s.active = actor;
    s.action_player = Some(actor);
    s
}
fn action(e: &Engine, s: &State, label: &str) -> Action {
    e.legal_actions(s).into_iter().find(|a| a.label == label).unwrap_or_else(|| panic!("missing {label}: {:?}", e.legal_actions(s)))
}
fn menu(e: &Engine, s: &State, source: CardId, label: &str) -> State {
    let opened = e.apply(s, &e.card_action(s, source)).unwrap();
    assert_eq!(opened.decision, Decision::AbilityMenu);
    e.apply(&opened, &action(e, &opened, label)).unwrap()
}
const BUFF: &str = "Get +1/+0 for this phase";
const SHOT: &str = "Deal damage equal to his power to an enemy ground unit";

#[test]
fn batch_crosshair_complete_actions_use_existing_ir() {
    let catalogue: Value = serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();
    let entry = catalogue["cards"].as_array().unwrap().iter().find(|e| e["setCode"] == "SHD_087").unwrap();
    assert_eq!(entry["status"], "supported");
    assert_eq!(entry["baseClass"], "NonLeaderUnitCard");
    assert_eq!(entry["card"]["types"], json!(["unit"]));
    assert_eq!(entry["abilities"]["card"].as_array().unwrap().len(), 2);
    assert_eq!(entry["abilities"]["card"][0], json!({
        "trigger":"action", "title":BUFF, "costs":[{"kind":"resources","amount":{"kind":"const","value":2}}],
        "effect":{"kind":"modifyStats","target":"self","duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}}
    }));
    let shot = &entry["abilities"]["card"][1];
    assert_eq!(shot, &json!({
        "trigger":"action", "title":SHOT, "costs":[{"kind":"exhaustSelf"}],
        "targetResolver":{"owner":"opponent","type":"any","zone":"arena","arena":"ground"},
        "effect":{"kind":"damage","target":"chosen","amount":{"kind":"query","name":"getPower","receiver":{"kind":"ref","path":["source"]},"args":[]}}
    }));
    let e = engine();
    assert!(e.by_code.contains_key("SHD_087"));
    // No new source slot is smuggled into raw IR admission.
    let mut raw = shot.clone();
    raw["effect"]["source"] = json!({"kind":"ref","path":["source"]});
    assert!(serde_json::from_value::<crate::ability_ir::Ability>(raw).is_err());
}

#[test]
fn batch_crosshair_paid_repeated_buffs_feed_live_damage_in_both_seats() {
    let e = engine();
    for actor in [Seat::P1, Seat::P2] {
        let mut s = board(&e, actor, 4);
        let source = unit(&e, &mut s, actor, "SHD_087");
        let target = unit(&e, &mut s, actor.other(), "LOF_256");
        let mut s = e.load_state(&e.snapshot(&s)).unwrap();
        assert_eq!(e.power(&s, source, false), 2);
        for expected in [3, 4] {
            let resources = e.ready_resources(&s, actor);
            s = menu(&e, &s, source, BUFF);
            assert_eq!(e.ready_resources(&s, actor), resources - 2);
            assert!(!s.card(source).exhausted, "the resource-only action never exhausts Crosshair");
            assert_eq!(e.power(&s, source, false), expected);
            assert_eq!(s.card(source).phase_hp, 0);
            assert_eq!(s.active, actor.other());
            s = e.apply(&s, &action(&e, &s, "Pass")).unwrap();
            assert_eq!(s.active, actor);
            s = e.load_state(&e.snapshot(&s)).unwrap();
        }
        let selecting = menu(&e, &s, source, SHOT);
        assert_eq!(selecting.decision, Decision::AbilityTarget);
        assert!(!selecting.card(source).exhausted, "target selection precedes exhaustion payment");
        let shot = e.apply(&selecting, &e.card_action(&selecting, target)).unwrap();
        assert!(shot.card(source).exhausted);
        assert_eq!(e.ready_resources(&shot, actor), 0);
        assert!(shot.player(actor.other()).discard.contains(target), "live power four defeats the four-HP target; printed power two would not");
        assert!(!shot.player(actor.other()).ground.contains(target));
        assert_eq!(shot.card(source).damage, 0);
        e.load_state(&e.snapshot(&shot)).unwrap();
    }
}

#[test]
fn batch_crosshair_shot_filters_targets_and_cancellation_preserves_costs() {
    let e = engine();
    let actor = Seat::P1;
    let mut s = board(&e, actor, 1);
    let source = unit(&e, &mut s, actor, "SHD_087");
    let friendly = unit(&e, &mut s, actor, "LOF_256");
    let enemy_ground = unit(&e, &mut s, actor.other(), "LOF_256");
    let enemy_space = unit(&e, &mut s, actor.other(), "ASH_201");
    let enemy_leader = s.player(actor.other()).leader;
    s.card_mut(enemy_leader).deployed = true;
    s.card_mut(enemy_leader).epic_used = true;
    e.add_arena(&mut s, enemy_leader).unwrap();
    let s = e.load_state(&e.snapshot(&s)).unwrap();
    let opened = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert!(!e.legal_actions(&opened).iter().any(|a| a.label == BUFF), "one resource cannot pay the two-resource action");
    let selecting = e.apply(&opened, &action(&e, &opened, SHOT)).unwrap();
    let targets: Vec<_> = e.legal_actions(&selecting).iter().filter_map(Action::id).collect();
    assert_eq!(targets, vec![enemy_ground, enemy_leader]);
    for excluded in [source, friendly, enemy_space, s.player(actor).base, s.player(actor.other()).base] {
        assert!(!targets.contains(&excluded));
    }
    let cancelled = e.apply(&selecting, &action(&e, &selecting, "Cancel")).unwrap();
    assert!(!cancelled.card(source).exhausted);
    assert_eq!(e.ready_resources(&cancelled, actor), 1);
    assert_eq!(cancelled.card(enemy_ground).damage, 0);
    let selecting = menu(&e, &cancelled, source, SHOT);
    let shot = e.apply(&selecting, &e.card_action(&selecting, enemy_ground)).unwrap();
    assert_eq!(shot.card(enemy_ground).damage, 2);
    assert!(shot.card(source).exhausted);
    assert_eq!(e.ready_resources(&shot, actor), 1, "the shot has no resource cost");
    let mut next = e.apply(&shot, &action(&e, &shot, "Pass")).unwrap();
    assert_eq!(next.active, actor);
    assert!(!e.legal_actions(&next).iter().any(|a| a.id() == Some(source)), "neither action is available when exhausted with one resource");
    next = e.load_state(&e.snapshot(&next)).unwrap();
    assert_eq!(next.card(enemy_ground).damage, 2);
}
