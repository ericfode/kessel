use super::*;

// Pure retained descriptor checks only. This does not construct an Engine or
// grant whole-card admission while native Piloting play remains unavailable.
fn piloting_source_recipe(e: &Value) -> bool {
    (crate::attached_registration::piloting_host_entry(e)
        || crate::attached_registration::piloting_self_entry(e))
        && crate::attached_registration::sides(e).is_ok_and(|s| {
            s.card
                .iter()
                .chain(&s.unit)
                .chain(&s.leader)
                .all(|a| Program::compile(a).is_ok())
        })
}
fn attack(e: &Engine, s: State, source: CardId, target: CardId) -> State {
    let a = e.card_action(&s, source);
    assert!(e.legal_actions(&s).contains(&a));
    let prompt = e.apply(&s, &a).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    e.apply(&prompt, &e.card_action(&prompt, target)).unwrap()
}
fn attach(e: &Engine, s: &mut State, owner: Seat, code: &str, host: CardId) -> CardId {
    let id = add(e, s, owner, code);
    e.attach(s, id, host);
    id
}
#[test]
fn attached_alias_admission_census_and_preserved_rejections() {
    let e = engine();
    for code in [
        "ASH_134", "ASH_210", "JTL_073", "SEC_039", "TS26_052", "TWI_169", "TWI_218", "IC27_085",
        "SHD_104", "SHD_175",
    ] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
    for code in [
        "LOF_138", "ASH_180", "ASH_227", "HMW_070", "LAW_141", "LAW_225", "JTL_227", "SHD_053",
        "TS26_035", "TWI_068", "TWI_120", "TWI_121",
    ] {
        assert!(
            !e.by_code.contains_key(code),
            "unsupported attached child admitted {code}"
        );
    }
}
#[test]
fn attached_alias_inspiring_mentor_expands_dual_hooks_like_separate_registrations() {
    for code in ["IC27_085", "SHD_104"] {
        for seat in [Seat::P1, Seat::P2] {
            let e = engine();
            let mut s = board(&e);
            s.round = 5;
            s.active = seat;
            s.action_player = Some(seat);
            let host = unit(&e, &mut s, seat, "LOF_254");
            let ally = unit(&e, &mut s, seat, "LOF_254");
            let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
            attach(&e, &mut s, seat.other(), code, host);
            let base = s.player(seat.other()).base;
            let prompt = attack(&e, s, host, base);
            assert_eq!(
                prompt.decision,
                Decision::AbilityTarget,
                "{code} onAttack prompt"
            );
            assert_eq!(prompt.source, host);
            assert_eq!(prompt.active, seat);
            let attack_actions = e.legal_actions(&prompt);
            assert!(attack_actions.iter().any(|a| a.id() == Some(ally)));
            assert!(!attack_actions
                .iter()
                .any(|a| a.id() == Some(host) || a.id() == Some(enemy)));
            let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
            assert_eq!(
                e.upgrades(&done, ally)
                    .iter()
                    .filter(|&&id| e.def(&done, id).code == "TOKEN_EXPERIENCE")
                    .count(),
                1,
                "{code} onAttack"
            );
            let mut dead = done;
            e.defeat(&mut dead, host).unwrap();
            assert_eq!(dead.death_pending.len, 1);
            e.open_deaths(&mut dead, DeathContinuation::Finish).unwrap();
            assert_eq!(dead.decision, Decision::AbilityTarget);
            let actions = e.legal_actions(&dead);
            assert!(actions.iter().any(|a| a.id() == Some(ally)));
            assert!(!actions.iter().any(|a| a.id() == Some(enemy)));
            let done = e.apply(&dead, &e.card_action(&dead, ally)).unwrap();
            assert_eq!(
                e.upgrades(&done, ally)
                    .iter()
                    .filter(|&&id| e.def(&done, id).code == "TOKEN_EXPERIENCE")
                    .count(),
                2,
                "{code} whenDefeated stacks"
            );
        }
    }
}
#[test]
fn attached_alias_dual_hooks_require_defeat_text_override_and_reject_partial_shapes() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "SHD_104")
        .unwrap()
        .clone();
    let admitted = |e: Value| {
        crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]}))
            .unwrap()
            .cards
            .contains_key("SHD_104")
    };
    assert!(admitted(original.clone()));
    let mut no_override = original.clone();
    no_override["setupValidationOverrides"] = json!([]);
    assert!(!admitted(no_override));
    let mut bad_override = original.clone();
    bad_override["setupValidationOverrides"][0]["gameplayEffect"] = json!("changesRules");
    assert!(!admitted(bad_override));
    let mut single = original.clone();
    single["abilities"]["card"][0]["effect"]["registration"]["program"]["trigger"] =
        json!("onAttack");
    single["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"] =
        json!([{"alias":"onAttack","enabled":true}]);
    assert!(!admitted(single), "override without dual hooks");
    let mut reversed = original.clone();
    reversed["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"] =
        json!([{"alias":"whenDefeated","enabled":true},{"alias":"onAttack","enabled":true}]);
    assert!(
        !admitted(reversed),
        "hook order is part of the reviewed contract"
    );
}
#[test]
fn attached_alias_defense_uses_host_controller_and_exact_arena_before_combat() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            let mut e = engine();
            e.set_rules_backend(backend).unwrap();
            let mut s = board(&e);
            s.round = 5;
            s.active = seat.other();
            s.action_player = Some(seat.other());
            let host = unit(&e, &mut s, seat, "LOF_254");
            let attacker = unit(&e, &mut s, seat.other(), "LOF_256");
            let space = unit(&e, &mut s, seat.other(), "TOKEN_XWING");
            let upgrade = attach(&e, &mut s, seat.other(), "ASH_210", host);
            let prompt = attack(&e, s, attacker, host);
            assert_eq!(prompt.decision, Decision::AbilityTarget);
            assert_eq!(prompt.source, host);
            assert_eq!(prompt.active, seat);
            assert_eq!(
                prompt.card(host).damage,
                0,
                "defense registration precedes combat"
            );
            let actions = e.legal_actions(&prompt);
            assert!(actions.iter().any(|a| a.id() == Some(attacker)));
            assert!(!actions.iter().any(|a| a.id() == Some(space)));
            let done = e.apply(&prompt, &e.card_action(&prompt, attacker)).unwrap();
            assert!(done.card(attacker).exhausted);
            assert!(done.card(attacker).damage >= 1);
            assert_eq!(done.card(upgrade).owner, seat.other());
        }
    }
}
#[test]
fn attached_alias_defeat_token_families_keep_captured_host_player_after_cleanup() {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        for seat in [Seat::P1, Seat::P2] {
            for (code, token) in [
                ("ASH_134", "TOKEN_MANDALORIAN"),
                ("TWI_169", "TOKEN_CLONE_TROOPER"),
                ("TWI_218", "TOKEN_BATTLE_DROID"),
            ] {
                let mut e = engine();
                e.set_rules_backend(backend).unwrap();
                let mut s = board(&e);
                s.round = 5;
                let host = unit(&e, &mut s, seat, "LOF_254");
                let grantor = attach(&e, &mut s, seat.other(), code, host);
                e.defeat(&mut s, host).unwrap();
                assert_eq!(s.card(grantor).parent, 0);
                assert!(s.player(seat.other()).discard.contains(grantor));
                assert!(s.player(seat).discard.contains(host));
                assert_eq!(s.death_pending.len, 1);
                assert_eq!(s.death_pending.entries[0].source, host);
                assert_eq!(s.death_pending.entries[0].player, seat);
                assert_eq!(s.death_pending.entries[0].grantor, grantor);
                assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
                let created = e
                    .arena_units(&s)
                    .into_iter()
                    .filter(|&id| s.card(id).owner == seat)
                    .filter(|&id| e.def(&s, id).code == token)
                    .collect::<Vec<_>>();
                assert_eq!(created.len(), 1, "{code}");
                assert!(s.card(created[0]).exhausted);
                assert!(e.arena_units(&s).iter().all(|&id| s.card(id).owner == seat));
            }
        }
    }
}
#[test]
fn attached_alias_defeat_selector_remains_live_and_optional_after_parent_removal() {
    for code in ["JTL_073", "SEC_039"] {
        for seat in [Seat::P1, Seat::P2] {
            let e = engine();
            let mut s = board(&e);
            s.round = 5;
            let host = unit(&e, &mut s, seat, "LOF_254");
            let target = unit(&e, &mut s, seat.other(), "LOF_256");
            s.card_mut(target).damage = e.hp(&s, target) - 3;
            let high = unit(&e, &mut s, seat.other(), "LOF_256");
            let grantor = attach(&e, &mut s, seat.other(), code, host);
            e.defeat(&mut s, host).unwrap();
            assert!(e.open_deaths(&mut s, DeathContinuation::Finish).unwrap());
            assert_eq!(s.decision, Decision::AbilityTarget);
            assert_eq!(s.source, host);
            assert_eq!(s.active, seat);
            assert_eq!(s.card(grantor).parent, 0);
            let legal = e.legal_actions(&s);
            assert!(legal.iter().any(|a| a.id() == Some(target)));
            if code == "SEC_039" {
                assert!(!legal.iter().any(|a| a.id() == Some(high)));
            }
            let declined = named_button(&e, &s, "Pass");
            assert!(e.in_arena(&declined, target));
            let done = e.apply(&s, &e.card_action(&s, target)).unwrap();
            if code == "JTL_073" {
                assert!(done.card(target).exhausted);
            } else {
                assert!(done.player(seat.other()).discard.contains(target));
            }
        }
    }
}
#[test]
fn attached_alias_sith_traditions_attack_and_death_are_separate_host_registrations() {
    for seat in [Seat::P1, Seat::P2] {
        let e = engine();
        let mut s = board(&e);
        s.round = 5;
        s.active = seat;
        s.action_player = Some(seat);
        let host = unit(&e, &mut s, seat, "LOF_254");
        let ally = unit(&e, &mut s, seat, "LOF_254");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_254");
        attach(&e, &mut s, seat.other(), "TS26_052", host);
        let base = s.player(seat.other()).base;
        let done = attack(&e, s, host, base);
        assert_eq!(
            e.upgrades(&done, host)
                .iter()
                .filter(|&&id| e.def(&done, id).code == "TOKEN_EXPERIENCE")
                .count(),
            1
        );
        let mut dead = done;
        e.defeat(&mut dead, host).unwrap();
        assert_eq!(dead.death_pending.len, 1);
        e.open_deaths(&mut dead, DeathContinuation::Finish).unwrap();
        assert_eq!(dead.decision, Decision::AbilityTarget);
        let actions = e.legal_actions(&dead);
        assert!(actions.iter().any(|a| a.id() == Some(ally)));
        assert!(!actions.iter().any(|a| a.id() == Some(enemy)));
        let done = e.apply(&dead, &e.card_action(&dead, ally)).unwrap();
        assert_eq!(
            e.upgrades(&done, ally)
                .iter()
                .filter(|&&id| e.def(&done, id).code == "TOKEN_EXPERIENCE")
                .count(),
            1
        );
    }
}
#[test]
fn attached_alias_grantor_departure_and_host_reentry_do_not_restore_lost_grants() {
    let e = engine();
    let mut s = board(&e);
    let host = unit(&e, &mut s, Seat::P1, "LOF_254");
    let other = unit(&e, &mut s, Seat::P1, "LOF_254");
    let grantor = attach(&e, &mut s, Seat::P2, "TWI_218", host);
    e.leave_play_primary(&mut s, grantor).unwrap();
    e.defeat(&mut s, host).unwrap();
    assert_eq!(s.death_pending.len, 0);
    s.player_mut(Seat::P1).discard.remove(host);
    e.add_arena(&mut s, host).unwrap();
    e.defeat(&mut s, host).unwrap();
    assert_eq!(s.death_pending.len, 0);
    s.player_mut(Seat::P2).discard.remove(grantor);
    e.attach(&mut s, grantor, other);
    e.defeat(&mut s, other).unwrap();
    assert_eq!(s.death_pending.len, 1);
    assert_eq!(s.death_pending.entries[0].source, other);
}

#[test]
fn attached_alias_both_players_share_one_declaration_window_and_host_prompt_order() {
    for first in ["You", "Opponent"] {
        let e = engine();
        let mut s = board(&e);
        s.round = 5;
        s.active = Seat::P1;
        s.action_player = Some(Seat::P1);
        let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
        let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
        attach(&e, &mut s, Seat::P2, "TS26_052", attacker);
        attach(&e, &mut s, Seat::P1, "ASH_210", defender);
        let prompt = attack(&e, s, attacker, defender);
        assert_eq!(prompt.decision, Decision::DeathPlayer);
        assert_eq!(prompt.active, Seat::P1);
        assert_eq!(prompt.card(attacker).damage, 0);
        assert_eq!(prompt.card(defender).damage, 0);
        assert_eq!(
            e.legal_actions(&prompt)
                .iter()
                .map(|a| a.label.as_str())
                .collect::<Vec<_>>(),
            vec!["You", "Opponent"]
        );
        let defense = named_button(&e, &prompt, first);
        assert_eq!(defense.decision, Decision::AbilityTarget);
        assert_eq!(defense.active, Seat::P2);
        assert_eq!(defense.source, defender);
        let experience = e
            .upgrades(&defense, attacker)
            .iter()
            .filter(|&&id| e.def(&defense, id).code == "TOKEN_EXPERIENCE")
            .count();
        assert_eq!(experience, usize::from(first == "You"));
        let done = e
            .apply(&defense, &e.card_action(&defense, attacker))
            .unwrap();
        assert_eq!(
            e.upgrades(&done, attacker)
                .iter()
                .filter(|&&id| e.def(&done, id).code == "TOKEN_EXPERIENCE")
                .count(),
            1
        );
        assert_eq!(done.attack_source, 0);
        assert_eq!(done.death_depth, 0);
    }
}

#[test]
fn attached_alias_captured_defense_survives_grantor_removal_and_copy_hash() {
    let e = engine();
    let mut s = board(&e);
    let attacker = unit(&e, &mut s, Seat::P1, "LOF_256");
    let defender = unit(&e, &mut s, Seat::P2, "LOF_254");
    let grantor = attach(&e, &mut s, Seat::P1, "ASH_210", defender);
    let mut prompt = attack(&e, s, attacker, defender);
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    assert_eq!(prompt.source, defender);
    let hash = prompt.zobrist();
    let copy = prompt;
    assert_eq!(copy.zobrist(), hash);
    assert_eq!(
        std::mem::size_of::<State>(),
        crate::model::EXPECTED_INLINE_STATE_BYTES
    );
    e.leave_play_primary(&mut prompt, grantor).unwrap();
    assert!(e
        .attached_host_grants(&prompt, defender, "onDefense")
        .is_empty());
    assert_ne!(prompt.zobrist(), hash);
    let done = e.apply(&prompt, &e.card_action(&prompt, attacker)).unwrap();
    assert!(done.card(attacker).damage >= 1);
    assert_eq!(done.card(grantor).parent, 0);
}

#[test]
fn attached_alias_clone_raid_follows_only_current_host_and_not_upgrade_mention() {
    let e = engine();
    let mut s = board(&e);
    let host = unit(&e, &mut s, Seat::P1, "LOF_254");
    let other = unit(&e, &mut s, Seat::P2, "LOF_254");
    let grantor = attach(&e, &mut s, Seat::P2, "TWI_169", host);
    assert_eq!(e.keyword_amount(&s, host, "raid"), 2);
    assert_eq!(e.keyword_amount(&s, grantor, "raid"), 0);
    assert_eq!(e.keyword_amount(&s, other, "raid"), 0);
    e.detach(&mut s, grantor);
    e.attach(&mut s, grantor, other);
    assert_eq!(e.keyword_amount(&s, host, "raid"), 0);
    assert_eq!(e.keyword_amount(&s, other, "raid"), 2);
}

#[test]
fn attached_alias_raw_metadata_hooks_fields_and_context_are_fail_closed() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "TWI_218")
        .unwrap()
        .clone();
    let admitted = |e: Value| {
        crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]}))
            .unwrap()
            .cards
            .contains_key("TWI_218")
    };
    assert!(admitted(original.clone()));
    for key in [
        "stateWatchers",
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "privateClassConstants",
        "inheritedSource",
        "deploymentOverride",
        "setupValidationOverrides",
    ] {
        let mut bad = original.clone();
        bad[key] = json!([{"kind":"unimplementedSourceContext"}]);
        assert!(!admitted(bad), "{key}");
    }
    for (path, value) in [
        ("/baseClass", json!("NonLeaderUnitCard")),
        ("/card/types", json!(["upgrade", "leader"])),
        ("/card/unique", json!("false")),
        ("/card/upgradeHp", json!(3)),
        ("/keywordValues", json!({"raid":2})),
        ("/card/keywords", json!(["smuggle"])),
        ("/piloting", json!({})),
        ("/abilities/card/0/optional", json!(true)),
        ("/abilities/card/0/effect/unknown", json!(true)),
        (
            "/abilities/card/0/effect/requiresGrantorInPlay",
            json!(false),
        ),
        (
            "/abilities/card/0/effect/controllerBinding",
            json!("grantorController"),
        ),
        ("/abilities/card/0/effect/sourceBinding", json!("grantor")),
        (
            "/abilities/card/0/effect/registrationMode",
            json!("piloting"),
        ),
        (
            "/abilities/card/0/effect/registration/gainCondition",
            json!({"kind":"const","value":true}),
        ),
        (
            "/abilities/card/0/effect/registration/limit",
            json!({"kind":"perRound","amount":1}),
        ),
        (
            "/abilities/card/0/effect/registration/type",
            json!("action"),
        ),
        (
            "/abilities/card/0/effect/registration/program/hooks",
            json!([{"alias":"whenDefeated","enabled":false}]),
        ),
        (
            "/abilities/card/0/effect/registration/program/hooks",
            json!([{"event":"onCardDefeated","predicate":{"kind":"const","value":true}}]),
        ),
        // Bare eventHooks without a reviewed alias rewrite stays rejected; matching
        // single-alias eventHooks (Paige) is normalized separately.
        (
            "/abilities/card/0/effect/registration/program/hooks",
            json!([]),
        ),
        (
            "/abilities/card/0/effect/registration/program/trigger",
            json!("whenPlayed"),
        ),
        (
            "/abilities/card/0/effect/registration/program/attachedHost",
            json!(true),
        ),
        (
            "/abilities/card/0/effect/registration/program/condition",
            json!({"kind":"ref","path":["source","parentUnit"]}),
        ),
        (
            "/abilities/card/0/effect/registration/program/effect/target",
            json!("parent"),
        ),
        (
            "/abilities/card/0/effect/registration/program/effect/tokenType",
            json!("credit"),
        ),
        (
            "/abilities/card/0/effect/registration/program/then",
            json!({"title":"unsupported","effect":{"kind":"privateProgramCall","name":"arbitrary"}}),
        ),
    ] {
        let mut bad = original.clone();
        let parts = path.split('/').skip(1).collect::<Vec<_>>();
        let mut target = &mut bad;
        for k in &parts[..parts.len() - 1] {
            target = if target.is_array() {
                &mut target[k.parse::<usize>().unwrap()]
            } else {
                &mut target[*k]
            };
        }
        target[parts[parts.len() - 1]] = value;
        assert!(!admitted(bad), "{path}");
    }
    let mut bad = original.clone();
    bad["abilities"]["card"][0]["attachedHost"] = json!(true);
    assert!(!admitted(bad));
    let mut nested = original.clone();
    let grant = nested["abilities"]["card"][0].clone();
    nested["abilities"]["card"][0] = json!({"trigger":"constant","title":"hidden wrapper","costs":[],"effect":{"kind":"sequence","effects":[grant["effect"].clone()]}});
    assert!(!admitted(nested));
}

#[test]
fn attached_alias_selected_pool_requires_each_created_token_definition() {
    let e = engine();
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"));
    // SHD_104 is a reprint alias of IC27_085 and is excluded from compiled_pool.
    for (code, token) in [
        ("ASH_134", "TOKEN_MANDALORIAN"),
        ("TWI_169", "TOKEN_CLONE_TROOPER"),
        ("TWI_218", "TOKEN_BATTLE_DROID"),
        ("TS26_052", "TOKEN_EXPERIENCE"),
        ("IC27_085", "TOKEN_EXPERIENCE"),
    ] {
        let mut pool = e.compiled_pool().unwrap();
        pool["cards"].as_array_mut().unwrap().retain(|a| {
            a["setCode"] == code || a["setCode"] == token || a["setCode"] == "TOKEN_SHIELD"
        });
        let valid = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes);
        assert!(valid.is_ok(), "{code}: {:?}", valid.err());
        pool["cards"]
            .as_array_mut()
            .unwrap()
            .retain(|a| a["setCode"] != token);
        let error = Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(), bytes)
            .err()
            .unwrap();
        assert!(error.contains(token), "{error}");
        if token != "TOKEN_EXPERIENCE" {
            assert!(error.contains(code), "{error}");
        } else {
            assert_eq!(
                error,
                "pool containing Experience creation must include reviewed TOKEN_EXPERIENCE"
            );
        }
    }
}
#[test]
fn attached_alias_physical_duplicate_grants_and_capacity_are_bounded() {
    let e = engine();
    let mut s = board(&e);
    let host = unit(&e, &mut s, Seat::P1, "LOF_254");
    let mut grantors = vec![];
    for _ in 0..65 {
        grantors.push(attach(&e, &mut s, Seat::P2, "TWI_218", host));
    }
    let before = s;
    assert!(e
        .capture_death(&mut s, host)
        .unwrap_err()
        .contains("capacity"));
    assert_eq!(s, before);
    e.detach(&mut s, grantors[64]);
    e.capture_death(&mut s, host).unwrap();
    assert_eq!(s.death_pending.len, 64);
    assert_eq!(
        s.death_pending.entries[..64]
            .iter()
            .map(|e| e.grantor)
            .collect::<Vec<_>>(),
        grantors[..64]
    );
    assert!(s.death_pending.entries[..64]
        .iter()
        .all(|e| e.source == host && e.player == Seat::P1));
}

#[test]
fn attached_alias_source_context_extras_are_not_silently_discarded() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "TWI_218")
        .unwrap();
    for field in [
        "stateWatchers",
        "historyHelpers",
        "privatePrograms",
        "classMethods",
        "privateClassConstants",
        "inheritedSource",
        "deploymentOverride",
        "setupValidationOverrides",
    ] {
        for extra in [
            json!([{"unsupported":"source context"}]),
            json!({"unknown":true}),
            json!(true),
        ] {
            let mut bad = original.clone();
            bad[field] = extra;
            let c = crate::ability_ir::Catalogue::load(
                &json!({"schema":crate::ability_ir::SCHEMA,"cards":[bad]}),
            )
            .unwrap();
            assert!(!c.cards.contains_key("TWI_218"), "{field}");
        }
    }
}

#[test]
fn piloting_host_rejects_leaders_and_extra_keywords() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_046")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    for (path, value) in [
        ("/baseClass", json!("LeaderUnitCard")),
        ("/card/pilotText", json!(null)),
        ("/card/keywords", json!(["piloting", "grit"])),
        (
            "/abilities/card/0/effect/registrationMode",
            json!("upgrade"),
        ),
        ("/abilities/card/0/effect/requiresAttachable", json!(false)),
        (
            "/abilities/card/0/effect/registration/program/hooks",
            json!([{"alias":"whenPlayed","enabled":true}]),
        ),
        (
            "/abilities/unit",
            json!([{"trigger":"onAttack","title":"extra","costs":[],"effect":{"kind":"noAction"}}]),
        ),
    ] {
        let mut bad = original.clone();
        let parts = path.split('/').skip(1).collect::<Vec<_>>();
        let mut target = &mut bad;
        for k in &parts[..parts.len() - 1] {
            target = if target.is_array() {
                &mut target[k.parse::<usize>().unwrap()]
            } else {
                &mut target[*k]
            };
        }
        target[parts[parts.len() - 1]] = value;
        assert!(!admitted(bad), "{path}");
    }
}

fn fire_attached_when_played(e: &Engine, s: &mut State, pilot: CardId) {
    s.source = pilot;
    s.trigger_context = 1;
    s.decision = Decision::Window;
    e.queue_programs(s, &["whenPlayed"]);
    e.process_all_triggers(s).unwrap();
}

#[test]
fn piloting_host_ig88_rejects_mismatched_companion_and_hooks() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_141")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    // Host-only (no companion) still admits via constant parent grant.
    let mut host_only = original.clone();
    host_only["abilities"]["card"] = json!([original["abilities"]["card"][1].clone()]);
    assert!(admitted(host_only), "host-only constant grant");
    // Turn the constant grant into an onAttack hook: unit companion is no longer a reviewed mirror.
    let mut hooked = original.clone();
    hooked["abilities"]["card"][1]["effect"]["registration"]["type"] = json!("triggered");
    hooked["abilities"]["card"][1]["effect"]["registration"]["program"]["trigger"] =
        json!("eventHooks");
    hooked["abilities"]["card"][1]["effect"]["registration"]["program"]["hooks"] =
        json!([{"alias":"onAttack","enabled":true}]);
    assert!(!admitted(hooked), "onAttack host + companion");
    // Wrong buff amount on the host program.
    let mut wrong = original.clone();
    wrong["abilities"]["card"][1]["effect"]["registration"]["program"]["effect"]["power"]
        ["value"] = json!(2);
    assert!(!admitted(wrong), "mismatched host amount");
    // Companion amount drift while host stays reviewed.
    let mut companion_wrong = original.clone();
    companion_wrong["abilities"]["card"][0]["effect"]["power"]["value"] = json!(1);
    assert!(!admitted(companion_wrong), "mismatched companion amount");
}

#[test]
fn piloting_self_rejects_non_when_played_and_companions() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_057")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    for (path, value) in [
        (
            "/abilities/card/0/effect/registration/program/hooks",
            json!([{"alias":"onAttack","enabled":true}]),
        ),
        (
            "/abilities/card/0/effect/kind",
            json!("attachedAbilityRegistration"),
        ),
        ("/abilities/card/0/effect/target", json!("parent")),
        (
            "/abilities/card/0/effect/sourceBinding",
            json!("attachedHost"),
        ),
        ("/card/keywords", json!(["piloting", "ambush"])),
        (
            "/abilities/card",
            json!([
                original["abilities"]["card"][0].clone(),
                {"trigger":"whenPlayed","title":"extra","costs":[],"effect":{"kind":"noAction"}}
            ]),
        ),
    ] {
        let mut bad = original.clone();
        if path == "/abilities/card" {
            bad["abilities"]["card"] = value;
        } else {
            let parts = path.split('/').skip(1).collect::<Vec<_>>();
            let mut target = &mut bad;
            for k in &parts[..parts.len() - 1] {
                target = if target.is_array() {
                    &mut target[k.parse::<usize>().unwrap()]
                } else {
                    &mut target[*k]
                };
            }
            target[parts[parts.len() - 1]] = value;
        }
        assert!(!admitted(bad), "{path}");
    }
}

#[test]
fn piloting_self_iden_rejects_partial_attach_and_shield_shapes() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_036")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Watcher must be self-attach, not an unrelated upgrade attach.
    let mut other_attach = original.clone();
    other_attach["abilities"]["card"][0]["effect"]["registration"]["program"]["hooks"] = json!([{
        "event":"onUpgradeAttached",
        "predicate":{"kind":"binary","op":"===",
            "left":{"kind":"ref","path":["event","parentCard"]},
            "right":{"kind":"ref","path":["source","parentCard"]}}
    }]);
    assert!(!admitted(other_attach), "non-self attach predicate");
    // Amount drift.
    let mut amount = original.clone();
    amount["abilities"]["card"][0]["effect"]["registration"]["program"]["effect"]["amount"]
        ["value"] = json!(2);
    assert!(!admitted(amount), "shield amount");
    // Named parent grant belongs on attachedAbilityRegistration, not Expression rewrite.
    let mut named_parent = original.clone();
    named_parent["abilities"]["card"][0]["effect"]["registration"]["program"]["effect"]["target"] =
        json!("parent");
    assert!(!admitted(named_parent), "named parent target");
    // Optional would change the attach contract.
    let mut optional = original.clone();
    optional["abilities"]["card"][0]["effect"]["registration"]["program"]["optional"] = json!(true);
    assert!(!admitted(optional), "optional");
}

#[test]
fn piloting_self_mandalorian_rejects_mismatched_companion_and_predicate() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_210")
        .unwrap()
        .clone();
    let source_recipe_supported = |e: Value| piloting_source_recipe(&e);
    let complete_recipe_supported = |e: Value| piloting_source_recipe(&e);
    assert!(source_recipe_supported(original.clone()));
    assert!(complete_recipe_supported(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Drop the pilotingRegistration: unit companion alone is not a piloting_self entry,
    // and the printed piloting keyword stays unimplemented without a reviewed path.
    let mut unit_only = original.clone();
    unit_only["abilities"]["card"] = json!([original["abilities"]["card"][0].clone()]);
    assert!(!crate::attached_registration::piloting_self_entry(
        &unit_only
    ));
    assert!(
        !complete_recipe_supported(unit_only),
        "companion without pilotingRegistration"
    );
    // Widen arena filter away from host parentCard zone.
    let mut bad_pred = original.clone();
    bad_pred["abilities"]["card"][1]["effect"]["registration"]["program"]["targetResolver"]
        ["predicate"] = json!({
        "kind":"binary","op":"===",
        "left":{"kind":"ref","path":["card","zoneName"]},
        "right":{"kind":"const","value":"groundArena"}
    });
    assert!(
        !source_recipe_supported(bad_pred),
        "static arena instead of host arena"
    );
    // Extra keyword breaks plain_pilot.
    let mut bad_kw = original.clone();
    bad_kw["card"]["keywords"] = json!(["piloting", "ambush"]);
    assert!(!source_recipe_supported(bad_kw), "extra keyword");
}

#[test]
fn piloting_self_frisk_rejects_cost_and_registration_query_drift() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_148")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Cost ceiling is part of the reviewed Frisk contract.
    let mut high = original.clone();
    high["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]
        ["predicate"]["right"]["right"]["value"] = json!(3);
    assert!(!admitted(high), "cost <= 3");
    // Optional is required; mandatory defeat would change the prompt protocol.
    let mut mandatory = original.clone();
    mandatory["abilities"]["card"][0]["effect"]["registration"]["program"]["optional"] =
        json!(false);
    assert!(!admitted(mandatory), "mandatory");
    // Unit domain is not the Frisk shape.
    let mut unit_type = original.clone();
    unit_type["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]
        ["type"] = json!("unit");
    assert!(!admitted(unit_type), "unit type");
    // Drop hasCost: bare cost comparison is not the reviewed guard.
    let mut no_has_cost = original.clone();
    no_has_cost["abilities"]["card"][0]["effect"]["registration"]["program"]["targetResolver"]
        ["predicate"] = json!({
        "kind":"binary","op":"<=",
        "left":{"kind":"ref","path":["card","cost"]},
        "right":{"kind":"const","value":2}
    });
    assert!(!admitted(no_has_cost), "missing hasCost");
}

fn stock_ready_resources(e: &Engine, s: &mut State, seat: Seat, n: usize) {
    for _ in 0..n {
        let id = add(e, s, seat, "LOF_254");
        s.player_mut(seat).resources.push(id).unwrap();
        s.card_mut(id).exhausted = false;
    }
}

#[test]
fn piloting_self_bb8_rejects_pay_ready_drift() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_145")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_self_entry(&original));
    // Amount is part of the reviewed BB-8 contract.
    let mut amount = original.clone();
    amount["abilities"]["card"][0]["effect"]["registration"]["program"]["effect"]["properties"]
        ["amount"]["value"] = json!(3);
    assert!(!admitted(amount), "pay 3");
    // Trait gate is Resistance only.
    let mut trait_name = original.clone();
    trait_name["abilities"]["card"][0]["effect"]["registration"]["program"]["ifYouDo"]
        ["targetResolver"]["predicate"]["args"][0]["value"] = json!("imperial");
    assert!(!admitted(trait_name), "non-resistance trait");
    // Mandatory payment would change the optional protocol.
    let mut mandatory = original.clone();
    mandatory["abilities"]["card"][0]["effect"]["registration"]["program"]["optional"] =
        json!(false);
    assert!(!admitted(mandatory), "mandatory");
}

#[test]
fn attached_alias_smuggle_mention_admits_armed_to_the_teeth() {
    let e = engine();
    assert!(
        e.by_code.contains_key("SHD_175"),
        "{:?}",
        e.rejected_cards().unwrap().get("SHD_175")
    );
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.round = 5;
        s.active = seat;
        s.action_player = Some(seat);
        let host = unit(&e, &mut s, seat, "LOF_254");
        let ally = unit(&e, &mut s, seat, "LOF_256");
        let enemy = unit(&e, &mut s, seat.other(), "LOF_256");
        let printed = e.power(&s, ally, false);
        attach(&e, &mut s, seat, "SHD_175", host);
        let base = s.player(seat.other()).base;
        let prompt = attack(&e, s, host, base);
        assert_eq!(prompt.decision, Decision::AbilityTarget, "{seat:?}");
        assert_eq!(prompt.source, host);
        let actions = e.legal_actions(&prompt);
        assert!(
            actions.iter().any(|a| a.id() == Some(ally)),
            "{seat:?} ally"
        );
        assert!(
            !actions
                .iter()
                .any(|a| a.id() == Some(host) || a.id() == Some(enemy)),
            "{seat:?} excludes host/enemy"
        );
        let done = e.apply(&prompt, &e.card_action(&prompt, ally)).unwrap();
        assert_eq!(e.power(&done, ally, false), printed + 2, "{seat:?}");
    }
}

#[test]
fn attached_alias_smuggle_mention_requires_printed_cost_text() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "SHD_175")
        .unwrap()
        .clone();
    let admitted = |e: Value| {
        crate::ability_ir::Catalogue::load(&json!({"schema":crate::ability_ir::SCHEMA,"cards":[e]}))
            .unwrap()
            .cards
            .contains_key("SHD_175")
    };
    assert!(admitted(original.clone()));
    let mut no_text = original.clone();
    no_text["card"]["text"] =
        json!("Attached unit gains: “On Attack: Give another friendly unit +2/+0 for this phase.”");
    assert!(!admitted(no_text), "smuggle without printed cost");
    let mut bare = original.clone();
    bare["card"]["keywords"] = json!(["smuggle", "raid"]);
    assert!(!admitted(bare), "raid mention still needs a parent grant");
}

#[test]
fn piloting_host_vader_rejects_missing_will_defeat_condition() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_142")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_host_entry(&original));
    let mut no_cond = original.clone();
    no_cond["abilities"]["card"][0]["effect"]["registration"]["program"]["ifYouDo"]
        .as_object_mut()
        .unwrap()
        .remove("ifYouDoCondition");
    assert!(
        !admitted(no_cond),
        "missing willDefeat condition would fire on any damage"
    );
    let mut bad_path = original.clone();
    bad_path["abilities"]["card"][0]["effect"]["registration"]["program"]["ifYouDo"]
        ["ifYouDoCondition"]["path"] = json!(["damageDealt"]);
    assert!(!admitted(bad_path), "non-willDefeat eventResult");
}

#[test]
fn will_defeat_justifier_and_scout_walker_give_advantage() {
    let e = engine();
    assert!(
        e.by_code.contains_key("ASH_146"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_146")
    );
    assert!(
        e.by_code.contains_key("ASH_176"),
        "{:?}",
        e.rejected_cards().unwrap().get("ASH_176")
    );
    // Justifier When Played: lethal damage grants Advantage to a chosen unit.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_146");
    let fragile = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(fragile).damage = e.hp(&s, fragile).saturating_sub(1);
    let ally = unit(&e, &mut s, Seat::P1, "LOF_256");
    s.source = source;
    s.trigger_context = 1;
    s.decision = Decision::Window;
    e.queue_programs(&mut s, &["whenPlayed"]);
    e.process_all_triggers(&mut s).unwrap();
    assert_eq!(s.decision, Decision::AbilityTarget);
    let follow = e.apply(&s, &e.card_action(&s, fragile)).unwrap();
    assert!(follow.player(Seat::P2).discard.contains(fragile));
    assert_eq!(follow.decision, Decision::AbilityTarget);
    let done = e.apply(&follow, &e.card_action(&follow, ally)).unwrap();
    assert_eq!(
        e.upgrades(&done, ally)
            .iter()
            .filter(|&&id| e.def(&done, id).code == "TOKEN_ADVANTAGE")
            .count(),
        1
    );

    // Scout Walker: ground-only 3 damage; self gains 3 Advantage on willDefeat.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "ASH_176");
    let ground = unit(&e, &mut s, Seat::P2, "LOF_256");
    s.card_mut(ground).damage = e.hp(&s, ground).saturating_sub(3);
    let space = unit(&e, &mut s, Seat::P2, "TOKEN_XWING");
    s.source = source;
    s.trigger_context = 1;
    s.decision = Decision::Window;
    e.queue_programs(&mut s, &["whenPlayed"]);
    e.process_all_triggers(&mut s).unwrap();
    assert_eq!(s.decision, Decision::AbilityTarget);
    let actions = e.legal_actions(&s);
    assert!(actions.iter().any(|a| a.id() == Some(ground)));
    assert!(
        !actions.iter().any(|a| a.id() == Some(space)),
        "space excluded"
    );
    let done = e.apply(&s, &e.card_action(&s, ground)).unwrap();
    assert!(done.player(Seat::P2).discard.contains(ground));
    assert_eq!(
        e.upgrades(&done, source)
            .iter()
            .filter(|&&id| e.def(&done, id).code == "TOKEN_ADVANTAGE")
            .count(),
        3
    );
}

#[test]
fn will_defeat_shoot_down_optional_base_followup() {
    let e = engine();
    assert!(
        e.by_code.contains_key("JTL_176"),
        "{:?}",
        e.rejected_cards().unwrap().get("JTL_176")
    );
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    // Stock resources to play the event (cost 2).
    for _ in 0..2 {
        let id = add(&e, &mut s, Seat::P1, "LOF_254");
        s.player_mut(Seat::P1).resources.push(id).unwrap();
        s.card_mut(id).exhausted = false;
    }
    let event = hand(&e, &mut s, "JTL_176");
    let space = unit(&e, &mut s, Seat::P2, "TOKEN_XWING");
    s.card_mut(space).damage = e.hp(&s, space).saturating_sub(3);
    let ground = unit(&e, &mut s, Seat::P2, "LOF_256");
    let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let actions = e.legal_actions(&prompt);
    assert!(actions.iter().any(|a| a.id() == Some(space)));
    assert!(
        !actions.iter().any(|a| a.id() == Some(ground)),
        "ground excluded"
    );
    let follow = e.apply(&prompt, &e.card_action(&prompt, space)).unwrap();
    // Token fighters leave play without a discard pile entry.
    assert!(!e.in_arena(&follow, space));
    assert!(follow.event_result.will_defeat);
    assert_eq!(
        follow.decision,
        Decision::AbilityTarget,
        "optional base select"
    );
    let enemy_base = follow.player(Seat::P2).base;
    let before = follow.card(enemy_base).damage;
    let done = e
        .apply(&follow, &e.card_action(&follow, enemy_base))
        .unwrap();
    assert_eq!(done.card(enemy_base).damage, before + 2);
}

#[test]
fn discard_return_admits_kuiil_krennic() {
    let e = engine();
    for code in ["SHD_041", "SEC_090"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn piloting_host_trace_rejects_unreviewed_distribute_shapes() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json")))
            .unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_066")
        .unwrap()
        .clone();
    let admitted = |e: Value| piloting_source_recipe(&e);
    assert!(admitted(original.clone()));
    assert!(crate::attached_registration::piloting_host_entry(&original));
    for (path,value) in [
        ("/abilities/card/0/effect/registration/program/effect/properties/amountToDistribute/value",json!(3)),
        ("/abilities/card/0/effect/registration/program/effect/properties/canDistributeLess/value",json!(true)),
        ("/abilities/card/0/effect/registration/program/effect/properties/controller/value",json!("self")),
        ("/abilities/card/0/effect/registration/program/effect/properties/cardCondition",json!({"kind":"const","value":true})),
    ] {
        let mut bad=original.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut target=&mut bad;
        for k in &parts[..parts.len()-1] {target=if target.is_array(){&mut target[k.parse::<usize>().unwrap()]}else{&mut target[*k]};}
        target[parts[parts.len()-1]]=value;assert!(!admitted(bad),"{path}");
    }
}

#[test]
fn kuiil_returns_discarded_card_when_aspects_match() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "SHD_041");
    s.card_mut(source).exhausted = false;
    // ASH_019 is vigilance; ASH_068 shares vigilance and will return.
    let matching = add(&e, &mut s, Seat::P1, "ASH_068");
    s.players[0].deck.push(matching).unwrap();
    let before_hand = s.players[0].hand.len;
    let mut done = attack(&e, s, source, s.players[1].base);
    // Non-optional On Attack enters TriggerOrder; resolve the ability.
    if done.decision == Decision::TriggerOrder {
        let trigger = e
            .legal_actions(&done)
            .into_iter()
            .find(|a| a.label.contains("Discard") || a.label == "Trigger")
            .unwrap_or_else(|| panic!("missing Kuiil trigger: {:?}", e.legal_actions(&done)));
        done = e.apply(&done, &trigger).unwrap();
    }
    assert!(
        done.players[0].hand.contains(matching),
        "matching aspect returns"
    );
    assert!(!done.players[0].discard.contains(matching));
    assert_eq!(done.players[0].hand.len, before_hand + 1);
    assert_eq!(done.decision, Decision::Window);

    // No shared aspect: discarded card stays in discard.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    let source = unit(&e, &mut s, Seat::P1, "SHD_041");
    s.card_mut(source).exhausted = false;
    let mismatch = add(&e, &mut s, Seat::P1, "LOF_256"); // no aspects
    s.players[0].deck.push(mismatch).unwrap();
    let mut done = attack(&e, s, source, s.players[1].base);
    if done.decision == Decision::TriggerOrder {
        let trigger = e
            .legal_actions(&done)
            .into_iter()
            .find(|a| a.label.contains("Discard") || a.label == "Trigger")
            .unwrap_or_else(|| panic!("missing Kuiil trigger: {:?}", e.legal_actions(&done)));
        done = e.apply(&done, &trigger).unwrap();
    }
    assert!(
        done.players[0].discard.contains(mismatch),
        "no shared aspect stays discarded"
    );
    assert!(!done.players[0].hand.contains(mismatch));
}

#[test]
fn krennic_optional_return_of_discarded_unit_on_defense() {
    let e = engine();
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P2;
    s.action_player = Some(Seat::P2);
    let defender = unit(&e, &mut s, Seat::P1, "SEC_090");
    let attacker = unit(&e, &mut s, Seat::P2, "LOF_254");
    s.card_mut(attacker).exhausted = false;
    let discarded = add(&e, &mut s, Seat::P1, "ASH_068");
    s.players[0].deck.push(discarded).unwrap();
    let prompt = attack(&e, s, attacker, defender);
    assert_eq!(
        prompt.decision,
        Decision::AbilityOptional,
        "unit discard offers confirm"
    );
    assert!(prompt.players[0].discard.contains(discarded));
    let done = e
        .apply(&prompt, &Action::button("Trigger", "trigger"))
        .unwrap();
    assert!(
        done.players[0].hand.contains(discarded),
        "Trigger returns unit"
    );
    assert!(!done.players[0].discard.contains(discarded));

    // Pass leaves the unit in discard.
    let mut s = board(&e);
    s.round = 5;
    s.active = Seat::P2;
    s.action_player = Some(Seat::P2);
    let defender = unit(&e, &mut s, Seat::P1, "SEC_090");
    let attacker = unit(&e, &mut s, Seat::P2, "LOF_254");
    s.card_mut(attacker).exhausted = false;
    let discarded = add(&e, &mut s, Seat::P1, "ASH_068");
    s.players[0].deck.push(discarded).unwrap();
    let prompt = attack(&e, s, attacker, defender);
    let done = e.apply(&prompt, &Action::button("Pass", "pass")).unwrap();
    assert!(done.players[0].discard.contains(discarded));
    assert!(!done.players[0].hand.contains(discarded));
}
