//! Source-derived public-action payment fixtures. DefeatCreditTokensCostAdjuster
//! fixes menu order, bounds, and cancellation; ResourceCost exhausts the remaining
//! total (including aspect penalties) after Credit defeat. Its resource event
//! resolves even at zero, so an all-Credit payResources effect runs ifYouDo.
//! Printed creators are LAW232/262 (one after unit entry), LAW244 (one), LAW248
//! (three). TWI056 pays ExhaustSelf before Resources; HMW003 reverses that order.
//! ResourceCost's Smuggle branch prioritizes the played resource itself.
use super::*;

fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn engine(backend: RulesBackend) -> Engine {
    let mut e = Engine::from_catalogue_value(&catalogue()).unwrap();
    e.set_rules_backend(backend).unwrap();
    e
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = s.next_id;
    s.next_id += 1;
    *s.card_mut(id) = Card {
        present: true,
        def: e.by_code[code],
        owner: seat,
        ..Card::default()
    };
    id
}
fn board(e: &Engine, actor: Seat, base_code: &str, leader_code: &str) -> State {
    let mut s = State::default();
    s.active = actor;
    s.action_player = Some(actor);
    s.initiative = actor;
    s.credit_state_exact = true;
    s.force_state_exact = true;
    s.hidden_history_exact = true;
    for seat in [actor, actor.other()] {
        s.player_mut(seat).base = add(e, &mut s, seat, base_code);
        s.player_mut(seat).leader = add(e, &mut s, seat, leader_code);
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
    }
    s
}
fn standard(e: &Engine, actor: Seat) -> State {
    board(e, actor, "SOR_029", "SOR_014")
}
fn hand(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).hand.push(id).unwrap();
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    e.add_arena(s, id).unwrap();
    id
}
fn deck(e: &Engine, s: &mut State, seat: Seat) -> CardId {
    let id = add(e, s, seat, "LOF_254");
    s.player_mut(seat).deck.push(id).unwrap();
    id
}
fn resources(e: &Engine, s: &mut State, seat: Seat, count: usize) -> Vec<CardId> {
    (0..count)
        .map(|_| {
            let id = add(e, s, seat, "LOF_254");
            s.player_mut(seat).resources.push(id).unwrap();
            id
        })
        .collect()
}
fn credits(e: &Engine, s: &mut State, seat: Seat, count: u16) -> Vec<CardId> {
    let first = s.next_id;
    e.create_credit_tokens(s, seat, count).unwrap();
    (first..s.next_id).collect()
}
fn live(s: &State, seat: Seat) -> Vec<CardId> {
    s.player(seat)
        .credits
        .as_slice()
        .iter()
        .map(|id| u16::from(*id))
        .collect()
}
fn choose(e: &Engine, s: &State, id: CardId) -> State {
    let a = e.card_action(s, id);
    assert!(
        e.legal_actions(s).contains(&a),
        "card {id} missing from {:?}",
        e.legal_actions(s)
    );
    e.apply(s, &a).unwrap()
}
fn button(e: &Engine, s: &State, title: &str) -> State {
    let actions = e.legal_actions(s);
    let a = actions
        .iter()
        .find(|a| a.label == title)
        .unwrap_or_else(|| panic!("{title}: {actions:?}"));
    e.apply(s, a).unwrap()
}
fn menu(e: &Engine, s: &State, expected: &[&str]) {
    assert_eq!(s.decision, Decision::CreditMenu);
    let expected = expected
        .iter()
        .enumerate()
        .map(|(i, label)| Action::button(*label, i))
        .collect::<Vec<_>>();
    assert_eq!(e.legal_actions(s), expected);
    assert_pending_copy(e, s);
}
fn amount(e: &Engine, s: &State, expected: std::ops::RangeInclusive<u16>) {
    assert_eq!(s.decision, Decision::CreditAmount);
    assert_eq!(
        e.legal_actions(s),
        expected
            .map(|n| Action {kind:"option".into(),card:None,arg:Some(Value::String(n.to_string())),method:None,label:format!("#{n}")})
            .collect::<Vec<_>>()
    );
    assert_pending_copy(e, s);
}
fn assert_pending_copy(e: &Engine, s: &State) {
    let copied = *s;
    assert_eq!(copied, *s);
    assert_eq!(copied.zobrist(), s.zobrist());
    assert_eq!(e.legal_actions(&copied), e.legal_actions(s));
    let snapshot = e.snapshot(s);
    if snapshot.get("directUnitPlayCreditState").is_some() {
        assert!(snapshot.get("creditPaymentPending").is_none());
        let imported=e.load_state(&snapshot).unwrap();
        assert_eq!(imported.zobrist(),s.zobrist());
        assert_eq!(e.snapshot(&imported),snapshot);
        assert_eq!(e.legal_actions(&imported),e.legal_actions(s));
        for action in e.legal_actions(s) {
            let reached=e.apply(s,&action).unwrap();let loaded=e.apply(&imported,&action).unwrap();
            assert_eq!(reached.zobrist(),loaded.zobrist());
            assert_eq!(e.snapshot(&reached),e.snapshot(&loaded));
            assert_eq!(e.legal_actions(&reached),e.legal_actions(&loaded));
        }
    } else {
        assert!(snapshot.get("creditPaymentPending").is_some());
        assert!(e.load_state(&snapshot).unwrap_err().contains("pending Credit"));
    }
}
fn cases(mut run: impl FnMut(&Engine, Seat)) {
    for backend in [RulesBackend::Auto, RulesBackend::Interpreter] {
        let e = engine(backend);
        for actor in [Seat::P1, Seat::P2] {
            run(&e, actor);
        }
    }
}

#[test]
fn credit_card_payment_all_credits_creates_fresh_identity_after_event_cost() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let event = hand(e, &mut s, actor, "LAW_244");
        assert_eq!(e.cost(&s, event), 1);
        let spent = credits(e, &mut s, actor, 1)[0];
        let fresh = s.next_id;
        let prompt = choose(e, &s, event);
        menu(e, &prompt, &["Use 1 Credit", "Cancel"]);
        assert!(prompt.player(actor).hand.contains(event));
        assert_eq!(live(&prompt, actor), vec![spent]);
        let done = button(e, &prompt, "Use 1 Credit");
        assert!(done.player(actor).discard.contains(event));
        assert!(!done.player(actor).hand.contains(event));
        assert_eq!(live(&done, actor), vec![fresh]);
        assert_eq!(done.next_id, fresh + 1);
        assert!(done.card(spent).present);
        assert_eq!(e.ready_resources(&done, actor), 0);
        assert_eq!(done.decision, Decision::Window);
        assert_eq!(done.active, actor.other());
        assert_eq!(
            live(&prompt, actor),
            vec![spent],
            "late continuation cannot mutate the earlier copy"
        );
    });
}

#[test]
fn credit_mixed_payment_fixed_and_number_menus_consume_exact_resource_remainder() {
    cases(|e, actor| {
        for (ready, count, chosen, expected_ready, expected_live) in
            [(3, 2, 2, 0, 3), (4, 3, 2, 1, 4)]
        {
            let mut s = standard(e, actor);
            let event = hand(e, &mut s, actor, "LAW_248");
            assert_eq!(e.cost(&s, event), 5);
            let row = resources(e, &mut s, actor, ready);
            let original_credits = credits(e, &mut s, actor, count);
            let fresh = s.next_id;
            let prompt = choose(e, &s, event);
            let done = if count == 2 {
                menu(e, &prompt, &["Use 2 Credits", "Cancel"]);
                button(e, &prompt, "Use 2 Credits")
            } else {
                menu(e, &prompt, &["Select amount", "Cancel"]);
                let number = button(e, &prompt, "Select amount");
                amount(e, &number, 1..=3);
                let before = number.zobrist();
                let label=format!("#{chosen}");
                let option=e.legal_actions(&number).into_iter().find(|a|a.label==label).unwrap();
                assert_eq!(option.kind,"option");assert_eq!(option.arg,Some(json!(chosen.to_string())));
                assert_eq!(option.card,None);assert_eq!(option.method,None);
                let done = e.apply(&number,&option).unwrap();
                assert_eq!(number.zobrist(), before);
                assert_eq!(live(&number, actor), original_credits);
                done
            };
            assert_eq!(e.ready_resources(&done, actor), expected_ready);
            assert_eq!(done.player(actor).credits.len(), expected_live);
            assert_eq!(done.next_id, fresh + 3);
            assert!(done.player(actor).discard.contains(event));
            for (i, id) in row.iter().enumerate() {
                assert_eq!(done.card(*id).exhausted, i < 5 - chosen as usize);
            }
        }
    });
}

#[test]
fn credit_skip_preserves_tokens_and_cancel_preserves_card_and_resources() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let event = hand(e, &mut s, actor, "LAW_244");
        let row = resources(e, &mut s, actor, 1);
        let old = credits(e, &mut s, actor, 1)[0];
        let fresh = s.next_id;
        let prompt = choose(e, &s, event);
        menu(
            e,
            &prompt,
            &["Use 1 Credit", "Pay costs without Credit tokens", "Cancel"],
        );
        let cancelled = button(e, &prompt, "Cancel");
        assert!(cancelled.player(actor).hand.contains(event));
        assert_eq!(live(&cancelled, actor), vec![old]);
        assert!(!cancelled.card(row[0]).exhausted);
        assert_eq!(cancelled.active, actor);
        assert_eq!(
            cancelled, s,
            "cancelling before the first cost preserves the whole position"
        );
        let done = button(e, &prompt, "Pay costs without Credit tokens");
        assert_eq!(live(&done, actor), vec![old, fresh]);
        assert!(done.card(row[0]).exhausted);
        assert!(done.player(actor).discard.contains(event));
    });
}

#[test]
fn credit_payment_applies_after_aspect_penalties_and_standard_cost_floor() {
    cases(|e, actor| {
        let mut s = board(e, actor, "ASH_019", "SOR_014");
        let event = hand(e, &mut s, actor, "LAW_248");
        // No Cunning aspect: 5 printed + 2 penalty, then Credits lower total.
        assert_eq!(e.cost(&s, event), 7);
        resources(e, &mut s, actor, 5);
        credits(e, &mut s, actor, 2);
        let prompt = choose(e, &s, event);
        menu(e, &prompt, &["Use 2 Credits", "Cancel"]);
        let done = button(e, &prompt, "Use 2 Credits");
        assert_eq!(e.ready_resources(&done, actor), 0);
        assert_eq!(live(&done, actor).len(), 3);

        let mut s = standard(e, actor);
        // AT-DP is 4 printed and Sabine supplies Aggression. Four damaged
        // ground units floor its reviewed standard-stage reduction at zero.
        for _ in 0..4 {
            let id = unit(e, &mut s, actor, "TWI_056");
            s.card_mut(id).damage = 1;
        }
        let source = hand(e, &mut s, actor, "JTL_163");
        let retained = credits(e, &mut s, actor, 2);
        assert_eq!(e.cost(&s, source), 0);
        let done = choose(e, &s, source);
        assert!(done.player(actor).ground.contains(source));
        assert_eq!(
            live(&done, actor),
            retained,
            "zero remaining cost creates no Credit menu"
        );
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_unit_card_payment_precedes_entry_and_when_played_creation() {
    cases(|e, actor| {
        for (code, charge) in [("LAW_232", 3), ("LAW_262", 6)] {
            let mut s = standard(e, actor);
            let source = hand(e, &mut s, actor, code);
            assert_eq!(e.cost(&s, source), charge);
            credits(e, &mut s, actor, charge);
            let fresh = s.next_id;
            let prompt = choose(e, &s, source);
            let label = format!("Use {charge} Credits");
            menu(e, &prompt, &[&label, "Cancel"]);
            assert!(prompt.player(actor).hand.contains(source));
            assert!(!prompt.player(actor).ground.contains(source));
            assert_eq!(
                prompt.next_id, fresh,
                "enter trigger cannot create before payment"
            );
            let done = button(e, &prompt, &label);
            assert!(done.player(actor).ground.contains(source));
            assert!(!done.player(actor).hand.contains(source));
            assert!(done.card(source).exhausted);
            assert_eq!(live(&done, actor), vec![fresh]);
            assert_eq!(done.next_id, fresh + 1);
            assert_eq!(done.decision, Decision::Window);
        }
    });
}

#[test]
fn credit_ability_cost_order_preserves_cancellation_before_and_after_exhaust_self() {
    cases(|e, actor| {
        let mut s = board(e, actor, "SOR_029", "HMW_003");
        let leader = s.player(actor).leader;
        let target = unit(e, &mut s, actor.other(), "SOR_239");
        let old = credits(e, &mut s, actor, 1);
        let targets = choose(e, &s, leader);
        assert_eq!(targets.decision, Decision::AbilityTarget);
        let prompt = choose(e, &targets, target);
        menu(e, &prompt, &["Use 1 Credit", "Cancel"]);
        assert!(
            !prompt.card(leader).exhausted,
            "resource-first source has not exhausted yet"
        );
        let cancelled = button(e, &prompt, "Cancel");
        assert!(!cancelled.card(leader).exhausted);
        assert_eq!(live(&cancelled, actor), old);

        let mut s = standard(e, actor);
        let source = unit(e, &mut s, actor, "TWI_056");
        let base = s.player(actor).base;
        s.card_mut(base).damage = 3;
        credits(e, &mut s, actor, 2);
        let abilities = choose(e, &s, source);
        assert_eq!(abilities.decision, Decision::AbilityMenu);
        let targets = button(e, &abilities, "Heal 2 damage from a unit or base");
        let prompt = choose(e, &targets, base);
        menu(e, &prompt, &["Use 2 Credits"]);
        assert!(
            prompt.card(source).exhausted,
            "earlier ExhaustSelf cost is already paid"
        );
        assert_eq!(prompt.card(base).damage, 3);
        let done = button(e, &prompt, "Use 2 Credits");
        assert_eq!(done.card(base).damage, 1);
        assert!(done.card(source).exhausted);
        assert!(live(&done, actor).is_empty());
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_adjusted_effect_payment_is_noncancellable_and_zero_resource_event_succeeds() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let source = unit(e, &mut s, actor, "SOR_206");
        let target = s.player(actor.other()).base;
        let drawn = deck(e, &mut s, actor);
        credits(e, &mut s, actor, 2);
        let attack = choose(e, &s, source);
        assert_eq!(attack.decision, Decision::AttackTarget);
        let optional = choose(e, &attack, target);
        assert_eq!(optional.decision, Decision::AbilityOptional);
        assert_eq!(
            e.legal_actions(&optional),
            vec![
                Action::button("Trigger", "trigger"),
                Action::button("Pass", "pass")
            ]
        );
        let declined = button(e, &optional, "Pass");
        assert_eq!(live(&declined, actor).len(), 2);
        assert!(declined.player(actor).deck.contains(drawn));
        let prompt = button(e, &optional, "Trigger");
        menu(e, &prompt, &["Use 2 Credits"]);
        assert!(prompt.player(actor).deck.contains(drawn));
        let done = button(e, &prompt, "Use 2 Credits");
        assert!(
            done.player(actor).hand.contains(drawn),
            "ifYouDo follows resolved payment event even with zero exhausted resources"
        );
        assert!(live(&done, actor).is_empty());
        assert_eq!(e.ready_resources(&done, actor), 0);
        assert_eq!(done.card(target).damage, 1);
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_upgrade_payment_retains_selected_host_until_attachment() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let host = unit(e, &mut s, actor, "SOR_239");
        let upgrade = hand(e, &mut s, actor, "SHD_175");
        assert_eq!(e.cost(&s, upgrade), 2);
        let row = resources(e, &mut s, actor, 1);
        credits(e, &mut s, actor, 1);
        let hosts = choose(e, &s, upgrade);
        assert_eq!(hosts.decision, Decision::UpgradeTarget);
        let prompt = choose(e, &hosts, host);
        menu(e, &prompt, &["Use 1 Credit", "Cancel"]);
        assert_eq!(prompt.card(upgrade).parent, 0);
        let done = button(e, &prompt, "Use 1 Credit");
        assert_eq!(done.card(upgrade).parent, host);
        assert!(done.card(row[0]).exhausted);
        assert!(live(&done, actor).is_empty());
        assert!(!done.player(actor).hand.contains(upgrade));
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_smuggle_unit_prioritizes_played_resource_and_preserves_replacement_order() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let row = resources(e, &mut s, actor, 6);
        let source = add(e, &mut s, actor, "SHD_086");
        s.player_mut(actor).resources.push(source).unwrap();
        let replacement = deck(e, &mut s, actor);
        assert_eq!(
            e.smuggle_cost(&s, source),
            Some(8),
            "4 + missing Command and Villainy penalties"
        );
        credits(e, &mut s, actor, 2);
        let prompt = choose(e, &s, source);
        menu(e, &prompt, &["Select amount", "Cancel"]);
        let number = button(e, &prompt, "Select amount");
        amount(e, &number, 1..=2);
        let done = button(e, &number, "#2");
        assert!(done.player(actor).ground.contains(source));
        assert!(!done.player(actor).resources.contains(source));
        assert!(row[..5].iter().all(|id| done.card(*id).exhausted));
        assert!(
            !done.card(row[5]).exhausted,
            "source-last resource must pay before this earlier row card"
        );
        assert_eq!(
            done.player(actor).resources.as_slice().last(),
            Some(&replacement)
        );
        assert!(done.card(replacement).exhausted);
        assert!(!done.player(actor).deck.contains(replacement));
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_smuggle_upgrade_preserves_host_priority_and_resource_replacement() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let host = unit(e, &mut s, actor, "SOR_239");
        let row = resources(e, &mut s, actor, 2);
        let upgrade = add(e, &mut s, actor, "SHD_175");
        s.player_mut(actor).resources.push(upgrade).unwrap();
        let replacement = deck(e, &mut s, actor);
        assert_eq!(e.smuggle_cost(&s, upgrade), Some(4));
        credits(e, &mut s, actor, 2);
        let hosts = choose(e, &s, upgrade);
        assert_eq!(hosts.decision, Decision::UpgradeTarget);
        let prompt = choose(e, &hosts, host);
        menu(e, &prompt, &["Select amount", "Cancel"]);
        let number = button(e, &prompt, "Select amount");
        amount(e, &number, 1..=2);
        let done = button(e, &number, "#2");
        assert_eq!(done.card(upgrade).parent, host);
        assert!(done.card(row[0]).exhausted);
        assert!(!done.card(row[1]).exhausted);
        // UpgradeCard disables its exhaust property on leaving Resource;
        // priority is observable through the earlier resource left ready.
        assert!(!done.player(actor).resources.contains(upgrade));
        assert_eq!(
            done.player(actor).resources.as_slice(),
            &[row[0], row[1], replacement]
        );
        assert!(done.card(replacement).exhausted);
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_creation_actions_reject_missing_exact_state_and_selected_native_definition() {
    let e = engine(RulesBackend::Interpreter);
    let mut s = standard(&e, Seat::P1);
    let event = hand(&e, &mut s, Seat::P1, "LAW_244");
    resources(&e, &mut s, Seat::P1, 1);
    s.credit_state_exact = false;
    let original = s;
    assert!(e
        .apply(&s, &e.card_action(&s, event))
        .unwrap_err()
        .contains("exact Credit"));
    assert_eq!(s, original);
    let mut pool = e.compiled_pool().unwrap();
    pool["cards"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["setCode"] != "TOKEN_CREDIT");
    let error = Engine::from_compiled_pool_bytes(
        &serde_json::to_vec(&pool).unwrap(),
        &serde_json::to_vec(&catalogue()).unwrap(),
    )
    .err()
    .expect("selected creator without native Credit must reject");
    assert!(error.contains("TOKEN_CREDIT"), "{error}");
}

#[test]
fn credit_disclose_event_pays_before_opening_hand_selection() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let event = hand(e, &mut s, actor, "SEC_076");
        let first = hand(e, &mut s, actor, "SEC_032");
        let second = hand(e, &mut s, actor, "SEC_032");
        let target = unit(e, &mut s, actor.other(), "SOR_239");
        s.card_mut(target).damage = 1;
        let charge = e.cost(&s, event);
        credits(e, &mut s, actor, charge);
        let prompt = choose(e, &s, event);
        let label = format!("Use {charge} Credits");
        menu(e, &prompt, &[&label, "Cancel"]);
        assert_eq!(prompt.disclose, DiscloseState::default());
        let selected = button(e, &prompt, &label);
        assert_eq!(selected.decision, Decision::DiscloseSelect);
        assert!(selected.player(actor).discard.contains(event));
        assert!(live(&selected, actor).is_empty());
        let selected = choose(e, &selected, first);
        let revealed = choose(e, &selected, second);
        let revealed = button(e, &revealed, "Done");
        assert_eq!(revealed.decision, Decision::DiscloseReveal);
        let targets = button(e, &revealed, "Done");
        assert_eq!(targets.decision, Decision::AbilityTarget);
        let done = choose(e, &targets, target);
        assert!(done.player(actor.other()).discard.contains(target));
    });
}

#[test]
fn resource_cost_death_window_restores_parent_before_the_selected_effect() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        s.player_mut(actor).force_held = true;
        let source = unit(e, &mut s, actor, "LOF_178");
        let sentinel = unit(e, &mut s, actor, "LOF_196");
        unit(e, &mut s, actor, "SEC_032");
        let printed = e.def(&s, sentinel).info.hp.unwrap() as u16;
        s.card_mut(sentinel).damage = printed;
        assert!(e.hp(&s, sentinel) > printed);
        let watcher = unit(e, &mut s, actor.other(), "SOR_036");
        let target = unit(e, &mut s, actor.other(), "SOR_239");
        let menu_state = choose(e, &s, source);
        let targets = button(e, &menu_state, "Exhaust a unit");
        let child = choose(e, &targets, target);
        assert!(child.player(actor).discard.contains(sentinel));
        assert!(!child.player(actor).force_held);
        assert!(child.card(source).exhausted);
        assert!(
            !child.card(target).exhausted,
            "selected effect waits for the cost event window"
        );
        assert_eq!(child.source, watcher);
        assert_eq!(child.decision, Decision::AbilityTarget);
        assert_pending_copy(e, &child);
        let done = choose(e, &child, watcher);
        assert!(done.card(watcher).experience != 0);
        assert!(done.card(target).exhausted);
        assert_eq!(done.decision, Decision::Window);
        assert_eq!(
            done.resource_payment,
            crate::resource_payment::ResourcePayment::default()
        );
        assert_eq!(done.payment_group, EventGroupState::default());
    });
}

#[test]
fn credit_attachment_discount_and_payment_share_the_chosen_physical_host() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let upgrade = hand(e, &mut s, actor, "ASH_262");
        let cheap = unit(e, &mut s, actor, "ASH_143");
        let plain = unit(e, &mut s, actor, "LOF_256");
        let full = e.cost(&s, upgrade);
        let reduced = full - 1;
        assert_eq!(e.upgrade_play_cost(&s, upgrade, cheap), Some(reduced));
        credits(e, &mut s, actor, reduced);
        let hosts = choose(e, &s, upgrade);
        assert!(e
            .legal_actions(&hosts)
            .contains(&e.card_action(&hosts, cheap)));
        assert!(!e
            .legal_actions(&hosts)
            .contains(&e.card_action(&hosts, plain)));
        let prompt = choose(e, &hosts, cheap);
        let label = if reduced == 1 {
            "Use 1 Credit".into()
        } else {
            format!("Use {reduced} Credits")
        };
        menu(e, &prompt, &[&label, "Cancel"]);
        assert_eq!(prompt.resource_payment.target, cheap);
        assert_eq!(prompt.resource_payment.amount, reduced);
        assert_eq!(prompt.card(upgrade).parent, 0);
        let done = button(e, &prompt, &label);
        assert_eq!(done.card(upgrade).parent, cheap);
        assert!(live(&done, actor).is_empty());
        assert_eq!(done.decision, Decision::Window);
    });
}

#[test]
fn credit_named_event_retains_both_pretargets_and_allocates_shield_after_payment() {
    cases(|e, actor| {
        let mut s = standard(e, actor);
        let event = hand(e, &mut s, actor, "SOR_218");
        let enemy = unit(e, &mut s, actor.other(), "LOF_256");
        let friendly = unit(e, &mut s, actor, "LOF_254");
        let charge = e.cost(&s, event);
        credits(e, &mut s, actor, charge);
        let next = s.next_id;
        let first = choose(e, &s, event);
        assert_eq!(first.decision, Decision::AbilityTarget);
        let second = choose(e, &first, enemy);
        assert_eq!(second.decision, Decision::AbilityTarget);
        let prompt = choose(e, &second, friendly);
        let label = if charge == 1 {
            "Use 1 Credit".into()
        } else {
            format!("Use {charge} Credits")
        };
        menu(e, &prompt, &[&label, "Cancel"]);
        assert!(prompt.event_group.active);
        assert_eq!(prompt.next_id, next, "Shield allocation follows payment");
        assert!(!prompt.card(enemy).exhausted);
        assert_eq!(prompt.card(friendly).shield, 0);
        assert_eq!(button(e, &prompt, "Cancel"), s);
        let done = button(e, &prompt, &label);
        assert!(done.player(actor).discard.contains(event));
        assert!(done.card(enemy).exhausted);
        assert_eq!(done.card(friendly).shield, next);
        assert_eq!(done.card(next).owner, actor);
        assert!(live(&done, actor).is_empty());
        assert_eq!(done.event_group, EventGroupState::default());
        assert_eq!(done.decision, Decision::Window);
    });
}

#[path = "credit_trigger_tests.rs"]
mod credit_trigger_tests;

#[path = "migration_credit_tests.rs"]
mod migration_credit_tests;
