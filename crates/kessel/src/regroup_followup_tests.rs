//! Source-derived regressions; these are not reference-engine comparisons.
use super::*;

const FOLLOWUP: &[&str] = &["ASH_159", "LOF_055", "TS26_023", "TWI_067"];

fn click(e: &Engine, s: &State, id: CardId) -> State {
    let action = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&action),
        "{id}: {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &action).unwrap()
}

fn play(e: &Engine, s: &State, seat: Seat, code: &str) -> (State, CardId) {
    let mut s = *s;
    s.active = seat;
    s.action_player = Some(seat);
    s.prev_passed = false;
    for _ in 0..14 {
        let id = add(e, &mut s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
    }
    let id = add(e, &mut s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    (click(e, &s, id), id)
}

fn advantages(e: &Engine, s: &State, host: CardId) -> Vec<CardId> {
    e.upgrades(s, host)
        .into_iter()
        .filter(|id| Some(s.card(*id).def) == e.advantage_def)
        .collect()
}

#[test]
fn regroup_followup_complete_source_closure_preserves_siblings_and_alpha_renames() {
    for code in FOLLOWUP.iter().chain(CANDIDATES.iter()) {
        let original = source(code);
        assert!(crate::regroup_hooks::entry(&original), "{code}");
        let index = original["abilities"]["card"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["trigger"] == "eventHooks")
            .unwrap();
        let ability = serde_json::from_value(original["abilities"]["card"][index].clone()).unwrap();
        assert!(crate::regroup_hooks::registration(&ability), "{code}");
        crate::ability_ir::Program::compile(&ability).unwrap();
        let mut renamed = original.clone();
        renamed["setCode"] = json!("DIAGNOSTIC_REGROUP_FOLLOWUP");
        renamed["className"] = json!("DiagnosticFollowup");
        renamed["card"]["title"] = json!("Diagnostic followup");
        renamed["abilities"]["card"][index]["title"] = json!("Diagnostic trigger");
        renamed["abilities"]["card"][index]["hooks"][0]["predicate"]["left"]["argumentBinding"]
            ["id"] = json!("callback:diagnostic:0");
        assert!(
            crate::regroup_hooks::entry(&renamed),
            "{code}: alpha rename"
        );
        for key in ["aspects", "traits", "keywords"] {
            renamed["card"][key].as_array_mut().unwrap().reverse();
        }
        assert!(crate::regroup_hooks::entry(&renamed), "{code}: memberships");
    }
    for code in ["LAW_073", "HMW_004", "LOF_019", "SHD_015"] {
        assert!(!crate::regroup_hooks::entry(&source(code)), "held {code}");
    }
}

#[test]
fn regroup_followup_source_closure_rejects_partial_cards_and_hidden_fields() {
    for code in FOLLOWUP {
        let original = source(code);
        for (path, value) in [
            ("/card/types", json!(["unit", "token"])),
            ("/card/power", json!(77)),
            ("/card/text", json!("Additional printed rule")),
            ("/abilities/unit", json!([{}])),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert!(!crate::regroup_hooks::entry(&changed), "{code}: {path}");
        }
        let index = original["abilities"]["card"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["trigger"] == "eventHooks")
            .unwrap();
        for key in ["optional", "condition", "then", "unknown"] {
            let mut changed = original.clone();
            changed["abilities"]["card"][index][key] = Value::Null;
            assert!(!crate::regroup_hooks::entry(&changed), "{code}: {key}");
        }
        let mut changed = original.clone();
        changed["abilities"]["card"][index]["effect"]["unknown"] = json!(true);
        assert!(!crate::regroup_hooks::entry(&changed));
        let mut changed = original.clone();
        changed["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .remove(index);
        assert!(!crate::regroup_hooks::entry(&changed));
        if index > 0 {
            let mut changed = original.clone();
            changed["abilities"]["card"]
                .as_array_mut()
                .unwrap()
                .remove(0);
            assert!(
                !crate::regroup_hooks::entry(&changed),
                "{code}: missing WP sibling"
            );
            let mut changed = original.clone();
            changed["abilities"]["card"][0]["effect"]["unknown"] = json!(true);
            assert!(
                !crate::regroup_hooks::entry(&changed),
                "{code}: changed WP sibling"
            );
        }
    }
    let mut catalogue = committed_catalogue().clone();
    let dume = catalogue["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["setCode"] == "LOF_055")
        .unwrap();
    dume["abilities"]["card"][0]["trigger"] = json!("whenPlayed");
    dume["abilities"]["card"][0]
        .as_object_mut()
        .unwrap()
        .remove("hooks");
    assert!(!crate::regroup_hooks::entry(dume));
    let changed = Engine::from_catalogue_value(&catalogue).unwrap();
    assert!(
        !changed.by_code.contains_key("LOF_055"),
        "normalized execution leaf must not admit a new source registration"
    );
}

#[test]
fn regroup_followup_uwing_targets_enemy_and_gives_native_recipient_owned_advantage() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let uwing = unit(e, &mut s, actor, "ASH_159");
        let band = unit(e, &mut s, actor, "LAW_071");
        unit(e, &mut s, actor, "JTL_198");
        let enemy = unit(e, &mut s, actor.other(), "LOF_256");
        s.card_mut(uwing).exhausted = true;
        assert!(e.has_keyword(&s, uwing, "overwhelm"));
        let pending = begin(e, &s);
        before_draw(e, &pending);
        let target = trigger(e, &pending, uwing);
        assert_eq!(target.decision, Decision::AbilityTarget);
        assert_eq!(target.active, actor);
        let ids: Vec<_> = e
            .legal_actions(&target)
            .iter()
            .filter_map(Action::id)
            .collect();
        for id in [uwing, band, enemy] {
            assert!(ids.contains(&id));
        }
        for seat in [actor, actor.other()] {
            assert!(!ids.contains(&target.player(seat).base));
        }
        let power = e.power(&target, enemy, false);
        let hp = e.hp(&target, enemy);
        let next = target.next_id;
        let awarded = click(e, &target, enemy);
        before_draw(e, &awarded);
        assert!(awarded.card(uwing).exhausted);
        assert_eq!(advantages(e, &awarded, enemy), vec![next]);
        assert_eq!(awarded.card(next).owner, actor.other());
        assert_eq!(awarded.card(next).parent, enemy);
        assert_eq!(e.power(&awarded, enemy, false), power + 1);
        assert_eq!(e.hp(&awarded, enemy), hp);
        assert!(advantages(e, &awarded, uwing).is_empty());
        let done = trigger(e, &awarded, band);
        assert_eq!(done.decision, Decision::Resource);
        assert!(done.card(uwing).exhausted);
        assert_eq!(done.player(actor).hand.len, 2);
        let done = named_button(e, &done, "Skip Resourcing");
        let done = named_button(e, &done, "Skip Resourcing");
        assert_eq!(done.decision, Decision::Window);
        assert!(!done.card(uwing).exhausted);
    });
}

#[test]
fn regroup_followup_dume_live_other_friendly_nonvehicle_units_in_both_arenas() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let dume = unit(e, &mut s, actor, "LOF_055");
        let ground = unit(e, &mut s, actor, "LOF_256");
        let space = unit(e, &mut s, actor, "LOF_109");
        let departed = unit(e, &mut s, actor, "LOF_254");
        let vehicle = unit(e, &mut s, actor, "TOKEN_XWING");
        let enemy = unit(e, &mut s, actor.other(), "LOF_256");
        let fireball = unit(e, &mut s, actor, "JTL_198");
        let lander = unit(e, &mut s, actor, "TS26_023");
        let mut pending = begin(e, &s);
        before_draw(e, &pending);
        // Internal timing diagnostic: alter membership after trigger capture.
        e.leave_play(&mut pending, departed).unwrap();
        let late = unit(e, &mut pending, actor, "LOF_254");
        let next = pending.next_id;
        let awarded = trigger(e, &pending, dume);
        before_draw(e, &awarded);
        assert_eq!(awarded.next_id, next + 3);
        for id in [ground, space, late] {
            assert_eq!(e.token_count(&awarded, awarded.card(id).experience), 1);
            assert_eq!(awarded.card(awarded.card(id).experience).owner, actor);
        }
        for id in [dume, departed, vehicle, enemy, fireball, lander] {
            assert_eq!(awarded.card(id).experience, 0, "excluded {id}");
        }
        assert_eq!(trigger(e, &awarded, fireball).decision, Decision::Resource);
    });
}

#[test]
fn regroup_followup_dume_no_other_eligible_units_creates_nothing() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        unit(e, &mut s, actor, "LOF_055");
        unit(e, &mut s, actor, "TOKEN_XWING");
        unit(e, &mut s, actor.other(), "LOF_256");
        let next = s.next_id;
        let done = begin(e, &s);
        assert_eq!(done.decision, Decision::Resource);
        assert_eq!(done.next_id, next);
        assert!(e
            .arena_units(&done)
            .iter()
            .all(|id| done.card(*id).experience == 0));
    });
}

#[test]
fn regroup_followup_laat_actual_play_creates_two_exhausted_clones_then_self_damage() {
    cases(|e, actor| {
        let s = midgame(e, actor);
        let (mut played, lander) = play(e, &s, actor, "TS26_023");
        let clones: Vec<_> = e
            .arena_units(&played)
            .into_iter()
            .filter(|id| e.def(&played, *id).code == "TOKEN_CLONE_TROOPER")
            .collect();
        assert_eq!(clones.len(), 2);
        for id in &clones {
            assert_eq!(played.card(*id).owner, actor);
            assert!(played.card(*id).exhausted);
            assert!(played.player(actor).ground.contains(*id));
        }
        let band = unit(e, &mut played, actor, "LAW_071");
        unit(e, &mut played, actor, "JTL_198");
        played.prev_passed = true;
        let pending = begin(e, &played);
        before_draw(e, &pending);
        let damaged = trigger(e, &pending, lander);
        before_draw(e, &damaged);
        assert_eq!(damaged.card(lander).damage, 4);
        assert!(damaged.card(lander).exhausted);
        assert!(e.in_arena(&damaged, lander));
        let done = trigger(e, &damaged, band);
        assert_eq!(done.decision, Decision::Resource);
        for id in &clones {
            assert!(done.card(*id).exhausted);
        }
        let done = named_button(e, &done, "Skip Resourcing");
        let done = named_button(e, &done, "Skip Resourcing");
        assert_eq!(done.decision, Decision::Window);
        for id in clones {
            assert!(!done.card(id).exhausted);
        }
    });
}

#[test]
fn regroup_followup_zillo_actual_play_debuff_snapshot_expires_before_self_heal() {
    cases(|e, actor| {
        let mut s = midgame(e, actor);
        let friendly = unit(e, &mut s, actor, "LOF_256");
        let ground = unit(e, &mut s, actor.other(), "LOF_256");
        let space = unit(e, &mut s, actor.other(), "TOKEN_XWING");
        let (mut played, zillo) = play(e, &s, actor, "TWI_067");
        assert_eq!(
            (
                played.card(ground).phase_power,
                played.card(ground).phase_hp
            ),
            (-5, 0)
        );
        assert_eq!(e.power(&played, ground, false), 0);
        for id in [friendly, space, zillo] {
            assert_eq!(played.card(id).phase_power, 0);
        }
        let late = unit(e, &mut played, actor.other(), "LOF_256");
        assert_eq!(played.card(late).phase_power, 0);
        played.card_mut(zillo).damage = 8;
        played.card_mut(friendly).damage = 2;
        let band = unit(e, &mut played, actor, "LAW_071");
        unit(e, &mut played, actor, "JTL_198");
        played.prev_passed = true;
        let pending = begin(e, &played);
        before_draw(e, &pending);
        assert_eq!(pending.card(ground).phase_power, 0);
        assert_eq!(pending.card(zillo).damage, 8);
        let healed = trigger(e, &pending, zillo);
        before_draw(e, &healed);
        assert_eq!(healed.card(zillo).damage, 3);
        assert_eq!(healed.card(friendly).damage, 2);
        assert!(healed.card(zillo).exhausted);
        assert_eq!(trigger(e, &healed, band).decision, Decision::Resource);
    });
}

#[test]
fn regroup_followup_self_effect_boundaries_lethal_damage_and_clamped_heal() {
    cases(|e, actor| {
        for damage in [0, 1] {
            let mut s = midgame(e, actor);
            let lander = unit(e, &mut s, actor, "TS26_023");
            s.card_mut(lander).damage = damage;
            let done = begin(e, &s);
            assert_eq!(done.decision, Decision::Resource);
            assert_eq!(done.player(actor).discard.contains(lander), damage == 1);
            if damage == 0 {
                assert_eq!(done.card(lander).damage, 4);
            }
        }
        for damage in [0, 2] {
            let mut s = midgame(e, actor);
            let zillo = unit(e, &mut s, actor, "TWI_067");
            s.card_mut(zillo).damage = damage;
            let done = begin(e, &s);
            assert_eq!(done.decision, Decision::Resource);
            assert_eq!(done.card(zillo).damage, 0);
            assert!(e.in_arena(&done, zillo));
        }
    });
}

#[test]
fn regroup_followup_native_dependencies_fail_closed_in_catalogue_and_selected_pool() {
    let full = engine();
    for (dependent, token) in [
        ("ASH_159", "TOKEN_ADVANTAGE"),
        ("LOF_055", "TOKEN_EXPERIENCE"),
        ("TS26_023", "TOKEN_CLONE_TROOPER"),
    ] {
        let mut selected = full.compiled_pool().unwrap();
        selected["cards"].as_array_mut().unwrap().retain(|c| {
            [
                "ASH_019",
                "SOR_014",
                "IBH_053",
                "TOKEN_SHIELD",
                "TOKEN_EXPERIENCE",
                "TOKEN_THE_FORCE",
                dependent,
                token,
            ]
            .contains(&c["setCode"].as_str().unwrap())
        });
        Engine::from_compiled_pool_bytes(
            &serde_json::to_vec(&selected).unwrap(),
            committed_catalogue_bytes(),
        )
        .unwrap();
        for malformed in [false, true] {
            let mut catalogue = committed_catalogue().clone();
            let cards = catalogue["cards"].as_array_mut().unwrap();
            if malformed {
                cards.iter_mut().find(|c| c["setCode"] == token).unwrap()["sourceHash"] =
                    json!("unreviewed");
            } else {
                cards.retain(|c| c["setCode"] != token);
            }
            let e = Engine::from_catalogue_value(&catalogue).unwrap();
            assert!(
                !e.by_code.contains_key(dependent),
                "{dependent}: {token}, malformed={malformed}"
            );
            assert!(e.by_code.contains_key("TWI_067"));
            let mut changed = selected.clone();
            let cards = changed["cards"].as_array_mut().unwrap();
            if malformed {
                cards.iter_mut().find(|c| c["setCode"] == token).unwrap()["traits"] =
                    json!(["unreviewed"]);
            } else {
                cards.retain(|c| c["setCode"] != token);
            }
            assert!(
                Engine::from_compiled_pool_bytes(
                    &serde_json::to_vec(&changed).unwrap(),
                    committed_catalogue_bytes()
                )
                .is_err(),
                "{dependent}: selected {token}, malformed={malformed}"
            );
        }
    }
}

#[test]
fn regroup_followup_live_vehicle_traits_and_native_shield_cancellation() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for actor in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = midgame(&e, actor);
            let dume = unit(&e, &mut s, actor, "LOF_055");
            let gains_vehicle = unit(&e, &mut s, actor, "LOF_256");
            let loses_vehicle = unit(&e, &mut s, actor, "TOKEN_XWING");
            let lander = unit(&e, &mut s, actor, "TS26_023");
            let band = unit(&e, &mut s, actor, "LAW_071");
            s.card_mut(lander).damage = 1;
            e.create_token(&mut s, lander, false).unwrap();
            let shield = s.card(lander).shield;
            let gain = add(&e, &mut s, actor, "IC27_052");
            let loss = add(&e, &mut s, actor, "IC27_093");
            for (upgrade, kind) in [(gain, "gainTrait"), (loss, "loseTrait")] {
                death_program(
                    &mut e,
                    &s,
                    upgrade,
                    json!({
                        "trigger":"constant", "title":"Synthetic live Vehicle trait diagnostic",
                        "attachedContext":"upgrade", "effect":{
                            "kind":kind, "target":"parent", "trait":"vehicle", "duration":"persistent"
                        }
                    }),
                );
            }
            assert!(!e.card_has_trait(&s, gains_vehicle, "vehicle"));
            assert!(e.card_has_trait(&s, loses_vehicle, "vehicle"));
            let mut pending = begin(&e, &s);
            before_draw(&e, &pending);
            // Internal timing diagnostic: attach synthetic shared trait leaves
            // after regroup trigger capture, without an external action window.
            e.attach(&mut pending, gain, gains_vehicle);
            e.attach(&mut pending, loss, loses_vehicle);
            assert!(e.card_has_trait(&pending, gains_vehicle, "vehicle"));
            assert!(!e.card_has_trait(&pending, loses_vehicle, "vehicle"));
            let next = pending.next_id;
            let awarded = trigger(&e, &pending, dume);
            before_draw(&e, &awarded);
            assert_eq!(awarded.next_id, next + 2);
            assert_eq!(awarded.card(gains_vehicle).experience, 0);
            assert_eq!(
                e.token_count(&awarded, awarded.card(loses_vehicle).experience),
                1
            );
            assert_eq!(e.token_count(&awarded, awarded.card(band).experience), 1);
            assert_eq!(awarded.card(dume).experience, 0);
            assert_eq!(awarded.card(lander).shield, shield);
            let done = trigger(&e, &awarded, lander);
            assert_eq!(done.decision, Decision::Resource);
            assert_eq!(
                done.card(lander).damage,
                1,
                "native Shield cancels all 4 damage"
            );
            assert!(
                e.in_arena(&done, lander),
                "prevented damage cannot defeat LAAT"
            );
            assert_eq!(done.card(lander).shield, 0);
            assert_eq!(done.card(shield).parent, 0);
        }
    }
}
