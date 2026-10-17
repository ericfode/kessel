// Source-derived witnesses for the cardRestriction IR family. The pinned
// forceteki implementations are the expected behavior; unsupported shapes stay
// explicit catalogue rejections instead of being approximated.
use super::*;

fn restriction_effect(v: Value) -> Effect { serde_json::from_value(v).unwrap() }

fn phase_mark(e: &Engine, s: &mut State, source: CardId, target: Value, verb: &str) -> (u16, bool) {
    s.source = source;
    let effect = restriction_effect(json!({
        "kind":"cardRestriction","target":target,"duration":"phase","restriction":verb
    }));
    e.apply_effect(s, &effect).unwrap()
}

fn attack(e: &Engine, mut s: State, source: CardId, target: CardId) -> State {
    s.active = s.card(source).owner;
    s.action_player = Some(s.active);
    let prompt = e.apply(&s, &e.card_action(&s, source)).unwrap();
    assert_eq!(prompt.decision, Decision::AttackTarget);
    let action = e.card_action(&prompt, target);
    assert!(e.legal_actions(&prompt).contains(&action));
    e.apply(&prompt, &action).unwrap()
}

fn expire_phase(e: &Engine, s: &mut State) {
    s.players[0].out = true;
    s.players[1].out = true;
    e.advance(s).unwrap();
}

fn last_instruction_effect(e: &Engine, code: &str, side: usize) -> Effect {
    let programs = &e.definitions[*e.by_code.get(code).unwrap() as usize].programs;
    let program = &e.catalogue.as_ref().unwrap().programs[programs.card[side] as usize];
    program.instructions.iter().rev().find_map(|i| match i {
        Instruction::Apply(effect) | Instruction::Group(effect) => Some(effect.clone()),
        _ => None,
    }).expect("restriction program")
}

#[test]
fn card_restriction_family_admission_is_reviewed_and_shapes_stay_closed() {
    let e = engine();
    let admitted = [
        "HMW_159","JTL_182","LAW_130","LAW_185","LAW_197","LOF_211","LOF_262",
        "SEC_135","SHD_193","SOR_160","TWI_132","TWI_219",
    ];
    for code in admitted {
        assert!(e.by_code.contains_key(code), "{code}: {:?}", e.rejected_cards().unwrap().get(code));
    }
    // Shapes blocked by other unimplemented families stay rejected; none is
    // admitted through a partially executed restriction.
    for code in [
        "LAW_073","TWI_039","LOF_063","TWI_195","LOF_098","HMW_095","HMW_121",
        "SEC_137","TS26_031","SOR_186","SEC_037","ASH_035","SEC_012",
    ] {
        assert!(!e.by_code.contains_key(code), "{code} must stay explicitly rejected");
    }
    // Every admitted card lowers its restriction into executable instructions.
    for code in admitted {
        let programs = &e.definitions[*e.by_code.get(code).unwrap() as usize].programs;
        let found = programs.card.iter().chain(&programs.unit).chain(&programs.leader).any(|&p| {
            e.catalogue.as_ref().unwrap().programs[p as usize].instructions.iter().any(|i| match i {
                Instruction::Apply(Effect::CardRestriction { .. }) => true,
                Instruction::Group(effect) => {
                    fn contains(effect: &Effect) -> bool {
                        match effect {
                            Effect::CardRestriction { .. } => true,
                            Effect::Simultaneous { effects } | Effect::PhaseModifiers { effects } =>
                                effects.iter().any(contains),
                            Effect::Select { effect, .. } => contains(effect),
                            Effect::Sequence { effects } => effects.iter().any(contains),
                            _ => false,
                        }
                    }
                    contains(effect)
                }
                _ => false,
            })
        });
        assert!(found, "{code} lost its restriction registration");
    }
}

#[test]
fn persistent_base_heal_restriction_blocks_both_bases_until_source_departs() {
    let e = engine();
    let mut s = board(&e);
    let own = s.player(Seat::P1).base;
    let enemy = s.player(Seat::P2).base;
    s.card_mut(own).damage = 5;
    s.card_mut(enemy).damage = 6;
    let source = unit(&e, &mut s, Seat::P1, "HMW_159");
    let heal_own = restriction_effect(json!({"kind":"heal","target":"ownBase","amount":{"kind":"const","value":3}}));
    let heal_enemy = restriction_effect(json!({"kind":"heal","target":"enemyBase","amount":{"kind":"const","value":3}}));
    s.source = source;
    let before = s;
    assert_eq!(e.apply_effect(&mut s, &heal_own).unwrap(), (0, false));
    assert_eq!(e.apply_effect(&mut s, &heal_enemy).unwrap(), (0, false));
    assert_eq!(s, before);
    assert!(e.card_restricted(&s, own, RestrictionVerb::BeHealed));
    assert!(e.card_restricted(&s, enemy, RestrictionVerb::BeHealed));
    // Unit healing is unaffected by a bases-only restriction.
    let friend = unit(&e, &mut s, Seat::P1, "ASH_061");
    s.card_mut(friend).damage = 3;
    assert_eq!(e.heal_card(&mut s, friend, 2), 2);
    assert_eq!(s.card(friend).damage, 1);
    // The Restore keyword path shares the same heal owner.
    assert_eq!(e.heal_card(&mut s, own, 5), 0);
    // Departure ends the live restriction for both bases.
    e.defeat(&mut s, source).unwrap();
    assert!(!e.card_restricted(&s, own, RestrictionVerb::BeHealed));
    assert_eq!(e.heal_card(&mut s, own, 3), 3);
    assert_eq!(e.heal_card(&mut s, enemy, 3), 3);
    assert_eq!(s.card(own).damage, 2);
    assert_eq!(s.card(enemy).damage, 3);
}

#[test]
fn conditional_restriction_constants_follow_live_source_state() {
    let e = engine();
    let mut s = board(&e);
    // "While this unit is ready, it can't be attacked."
    let crab = unit(&e, &mut s, Seat::P1, "SEC_135");
    assert!(e.card_restricted(&s, crab, RestrictionVerb::BeAttacked));
    s.card_mut(crab).exhausted = true;
    assert!(!e.card_restricted(&s, crab, RestrictionVerb::BeAttacked));
    s.card_mut(crab).exhausted = false;
    assert!(e.card_restricted(&s, crab, RestrictionVerb::BeAttacked));
    let attacker = unit(&e, &mut s, Seat::P2, "ASH_061");
    let targets = e.attack_targets(&s, attacker, false);
    assert!(!targets.contains(&crab));
    assert!(targets.contains(&s.player(Seat::P1).base));
    s.card_mut(crab).exhausted = true;
    assert!(e.attack_targets(&s, attacker, false).contains(&crab));
    s.card_mut(crab).exhausted = false;
    // "Doesn't ready during regroup unless its power is 4 or more."
    let rampart = unit(&e, &mut s, Seat::P2, "JTL_182");
    assert!(e.card_restricted(&s, rampart, RestrictionVerb::DoesNotReadyDuringRegroup));
    s.card_mut(rampart).phase_power = 1;
    assert_eq!(e.power(&s, rampart, false), 4);
    assert!(!e.card_restricted(&s, rampart, RestrictionVerb::DoesNotReadyDuringRegroup));
}

#[test]
#[test]
fn phase_be_attacked_mark_removes_targets_sentinel_overrides_and_expiry_clears() {
    let e = engine();
    let mut s = board(&e);
    let exposed = unit(&e, &mut s, Seat::P2, "LOF_256");
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_061");
    // Mark the enemy unit itself; the second application is idempotent.
    assert_eq!(phase_mark(&e, &mut s, exposed, json!("self"), "beAttacked"), (1, true));
    assert_eq!(phase_mark(&e, &mut s, exposed, json!("self"), "beAttacked"), (0, false));
    assert!(e.card_restricted(&s, exposed, RestrictionVerb::BeAttacked));
    let targets = e.attack_targets(&s, attacker, false);
    assert!(!targets.contains(&exposed));
    assert!(targets.contains(&s.player(Seat::P2).base));
    // A marked Sentinel unit stays attackable: Sentinel still demands the swing
    // (CR 2.0 7.5.11.D), so the restriction cannot hide it.
    let sentinel = unit(&e, &mut s, Seat::P2, "ASH_030");
    assert!(e.keyword_amount(&s, sentinel, "sentinel") > 0);
    assert_eq!(phase_mark(&e, &mut s, sentinel, json!("self"), "beAttacked"), (1, true));
    assert!(e.card_restricted(&s, sentinel, RestrictionVerb::BeAttacked));
    assert!(e.attack_targets(&s, attacker, false).contains(&sentinel));
    // Canonical snapshot round-trips the mask; transient source is not state.
    let hash = s.zobrist();
    let snapshot = e.snapshot(&s);
    let loaded = e.load_state(&snapshot).unwrap();
    assert_eq!(e.snapshot(&loaded), snapshot);
    assert_ne!(loaded.card(exposed).phase_restrictions, 0);
    assert!(snapshot.to_string().contains("phaseRestrictions"));
    expire_phase(&e, &mut s);
    assert!(!e.card_restricted(&s, exposed, RestrictionVerb::BeAttacked));
    assert!(!e.card_restricted(&s, sentinel, RestrictionVerb::BeAttacked));
    assert_ne!(s.zobrist(), hash);
    e.defeat(&mut s, sentinel).unwrap();
    assert!(e.attack_targets(&s, attacker, false).contains(&exposed));
}

#[test]
fn phase_deal_combat_damage_prevents_attack_and_defense_damage_only() {
    let e = engine();
    let mut s = board(&e);
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_061");
    let defender = unit(&e, &mut s, Seat::P2, "ASH_030");
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    assert_eq!(phase_mark(&e, &mut s, source, json!("self"), "dealCombatDamage"), (1, true));
    s.source = defender;
    let mark = restriction_effect(json!({"kind":"cardRestriction","target":"self","duration":"phase","restriction":"dealCombatDamage"}));
    assert_eq!(e.apply_effect(&mut s, &mark).unwrap(), (1, true));
    let attack_power = e.power(&s, attacker, true);
    let defend_power = e.power(&s, defender, false);
    assert!(attack_power > 0 && defend_power > 0);
    let done = attack(&e, s, attacker, defender);
    assert_eq!(done.card(defender).damage, attack_power);
    assert_eq!(done.card(attacker).damage, 0);
    // An attacker under the same verb deals no combat damage in the other direction.
    let mut s = board(&e);
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_061");
    let defender = unit(&e, &mut s, Seat::P2, "ASH_030");
    s.source = attacker;
    let mark = restriction_effect(json!({"kind":"cardRestriction","target":"self","duration":"phase","restriction":"dealCombatDamage"}));
    assert_eq!(e.apply_effect(&mut s, &mark).unwrap(), (1, true));
    let defend = e.power(&s, defender, false);
    let done = attack(&e, s, attacker, defender);
    assert_eq!(done.card(defender).damage, 0);
    assert_eq!(done.card(attacker).damage, defend);
}

#[test]
fn phase_attack_restriction_removes_the_declaration_for_the_phase() {
    let e = engine();
    let mut s = board(&e);
    let bystander = unit(&e, &mut s, Seat::P2, "ASH_061");
    let source = unit(&e, &mut s, Seat::P1, "LOF_254");
    assert_eq!(phase_mark(&e, &mut s, source, json!("self"), "attack"), (1, true));
    assert!(e.card_restricted(&s, source, RestrictionVerb::Attack));
    assert!(e.attack_targets(&s, source, false).is_empty());
    s.active = Seat::P1;
    s.action_player = Some(Seat::P1);
    assert!(!e.legal_actions(&s).contains(&e.card_action(&s, source)));
    assert!(!e.attack_targets(&s, bystander, false).is_empty());
    expire_phase(&e, &mut s);
    assert!(!e.card_restricted(&s, source, RestrictionVerb::Attack));
    assert!(!e.attack_targets(&s, source, false).is_empty());
    // The post-expiry phase transition itself remains the framework's ordinary
    // resource/regroup flow, unrelated to the restriction mask.
    assert_eq!(s.decision, Decision::Resource);
}

#[test]
fn attached_ready_restriction_blocks_ready_effects_and_regroup_ready() {
    let e = engine();
    let mut s = board(&e);
    upgrade_resources(&e, &mut s, Seat::P1);
    let host = unit(&e, &mut s, Seat::P2, "ASH_030");
    let upgrade = hand(&e, &mut s, "SHD_193");
    let played = play_upgrade(&e, &s, upgrade, host);
    assert!(played.card(host).exhausted, "Frozen in Carbonite exhausts its attached unit");
    assert!(e.card_restricted(&played, host, RestrictionVerb::Ready));
    let mut s = played;
    let ready = restriction_effect(json!({"kind":"ready","target":"self"}));
    s.source = host;
    assert_eq!(e.apply_effect(&mut s, &ready).unwrap(), (0, false));
    assert!(s.card(host).exhausted);
    e.end_regroup(&mut s).unwrap();
    assert!(s.card(host).exhausted);
    // Detaching lifts the live restriction before the next regroup ready step.
    e.defeat(&mut s, upgrade).unwrap();
    assert!(!e.card_restricted(&s, host, RestrictionVerb::Ready));
    e.end_regroup(&mut s).unwrap();
    assert!(!s.card(host).exhausted);
}

#[test]
fn phase_modifier_window_applies_its_restriction_leaf_with_the_stats() {
    // One reviewed PhaseModifiers window carries stat and restriction leaves.
    // No admitted card reaches this path yet (TWI_039 is owned by the Exploit
    // family), so the runtime primitive is witnessed directly.
    let e = engine();
    let mut s = board(&e);
    let target = unit(&e, &mut s, Seat::P2, "ASH_030");
    let effect = restriction_effect(json!({
        "kind":"phaseModifiers",
        "effects":[
            {"kind":"modifyStats","target":"self","duration":"phase",
             "power":{"kind":"const","value":-4},"hp":{"kind":"const","value":0}},
            {"kind":"cardRestriction","target":"self","duration":"phase","restriction":"attack"}
        ]
    }));
    s.source = target;
    let (count, resolved) = e.apply_effect(&mut s, &effect).unwrap();
    assert!(count > 0 && resolved);
    assert_eq!(s.card(target).phase_power, -4);
    assert!(e.card_restricted(&s, target, RestrictionVerb::Attack));
    assert!(e.attack_targets(&s, target, false).is_empty());
}

#[test]
fn grouped_and_arena_list_restriction_leaves_mark_their_recipients() {
    let e = engine();
    // LAW_185: one Simultaneous window readies an ally and marks it beAttacked.
    let mut s = board(&e);
    upgrade_resources(&e, &mut s, Seat::P1);
    let ally = unit(&e, &mut s, Seat::P1, "ASH_061");
    let enemy = unit(&e, &mut s, Seat::P2, "ASH_061");
    s.card_mut(ally).exhausted = true;
    let ben = hand(&e, &mut s, "LAW_185");
    let prompt = e.apply(&s, &e.card_action(&s, ben)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let action = e.card_action(&prompt, ally);
    assert!(e.legal_actions(&prompt).contains(&action));
    let done = e.apply(&prompt, &action).unwrap();
    assert!(!done.card(ally).exhausted);
    assert!(e.card_restricted(&done, ally, RestrictionVerb::BeAttacked));
    assert!(!e.attack_targets(&done, enemy, false).contains(&ally));
    // The restriction registration is a real grouped primitive, not a fallback.
    let effect = last_instruction_effect(&e, "LAW_185", 0);
    assert!(matches!(effect, Effect::Simultaneous { .. }));

    // LOF_211: the arena-list snapshot marks every friendly Hidden unit.
    let mut s = board(&e);
    upgrade_resources(&e, &mut s, Seat::P1);
    let hidden_friend = unit(&e, &mut s, Seat::P1, "LAW_185");
    let plain_friend = unit(&e, &mut s, Seat::P1, "ASH_061");
    let dooku = hand(&e, &mut s, "LOF_211");
    let done = e.apply(&s, &e.card_action(&s, dooku)).unwrap();
    assert!(e.card_restricted(&done, dooku, RestrictionVerb::BeAttacked));
    assert!(e.card_restricted(&done, hidden_friend, RestrictionVerb::BeAttacked));
    assert!(!e.card_restricted(&done, plain_friend, RestrictionVerb::BeAttacked));
}

#[test]
fn root_phase_base_heal_mark_from_wolffe_expires_after_the_phase() {
    let e = engine();
    let mut s = board(&e);
    upgrade_resources(&e, &mut s, Seat::P1);
    let enemy_base = s.player(Seat::P2).base;
    s.card_mut(enemy_base).damage = 4;
    let wolffe = hand(&e, &mut s, "SOR_160");
    let played = e.apply(&s, &e.card_action(&s, wolffe)).unwrap();
    let own_base = played.player(Seat::P1).base;
    assert!(e.card_restricted(&played, own_base, RestrictionVerb::BeHealed));
    assert!(e.card_restricted(&played, enemy_base, RestrictionVerb::BeHealed));
    let mut s = played;
    assert_eq!(e.heal_card(&mut s, own_base, 2), 0);
    expire_phase(&e, &mut s);
    assert!(!e.card_restricted(&s, own_base, RestrictionVerb::BeHealed));
    assert_eq!(e.heal_card(&mut s, enemy_base, 2), 2);
}

#[test]
fn event_restriction_marks_the_chosen_recipient_for_the_phase() {
    let e = engine();
    let mut s = board(&e);
    let event = hand(&e, &mut s, "LOF_262");
    let victim = unit(&e, &mut s, Seat::P2, "LOF_256");
    let attacker = unit(&e, &mut s, Seat::P1, "ASH_061");
    let prompt = e.apply(&s, &e.card_action(&s, event)).unwrap();
    assert_eq!(prompt.decision, Decision::AbilityTarget);
    let action = e.card_action(&prompt, victim);
    assert!(e.legal_actions(&prompt).contains(&action));
    let done = e.apply(&prompt, &action).unwrap();
    assert!(s.players[0].hand.contains(event) || done.players[0].discard.contains(event));
    assert!(e.card_restricted(&done, victim, RestrictionVerb::BeAttacked));
    assert!(!e.attack_targets(&done, attacker, false).contains(&victim));
}

#[test]
fn raw_restriction_shapes_fail_closed_with_explicit_reasons() {
    // Root phase restrictions can only execute for the phase.
    for duration in ["round", "persistent", "whileSourceInPlay"] {
        let ability: crate::ability_ir::Ability = serde_json::from_value(json!({
            "trigger":"event","title":"bad",
            "effect":{"kind":"cardRestriction","target":"chosen","duration":duration,"restriction":"ready"}
        })).unwrap();
        let err = crate::ability_ir::Program::compile(&ability).unwrap_err();
        assert!(err.contains("card restrictions execute as persistent constants or a root phase mark"), "{duration}: {err}");
    }
    // doesNotReadyDuringRegroup is a live constant only.
    let ability: crate::ability_ir::Ability = serde_json::from_value(json!({
        "trigger":"whenPlayed","title":"bad",
        "effect":{"kind":"cardRestriction","target":"self","duration":"phase","restriction":"doesNotReadyDuringRegroup"}
    })).unwrap();
    assert!(crate::ability_ir::Program::compile(&ability).unwrap_err().contains("doesNotReadyDuringRegroup only lives on a persistent constant"));
    // Effect-level action conditions stay on the constant ability gate.
    let ability: crate::ability_ir::Ability = serde_json::from_value(json!({
        "trigger":"event","title":"bad",
        "effect":{"kind":"cardRestriction","target":"chosen","duration":"phase","restriction":"ready",
                  "condition":{"kind":"const","value":true}}
    })).unwrap();
    assert!(crate::ability_ir::Program::compile(&ability).unwrap_err().contains("conditions stay on the constant ability"));
    // Constant restriction gates reject round durations, unbound chosen targets
    // and effect-level conditions.
    for (duration, target) in [("round","self"),("persistent","chosen")] {
        let ability: crate::ability_ir::Ability = serde_json::from_value(json!({
            "trigger":"constant","title":"bad",
            "effect":{"kind":"cardRestriction","target":target,"duration":duration,"restriction":"beAttacked"}
        })).unwrap();
        assert!(crate::ability_ir::Program::compile(&ability).is_err(), "{duration}/{target}");
    }
    // Imported phase masks require the reviewed verbs and survive snapshots.
    let e = engine();
    let s = board(&e);
    let snapshot = e.snapshot(&s);
    let mut with_mark = snapshot.clone();
    with_mark["players"]["p1"]["base"]["phaseRestrictions"] = json!(["beHealed"]);
    let loaded = e.load_state(&with_mark).unwrap();
    assert!(e.card_restricted(&loaded, loaded.player(Seat::P1).base, RestrictionVerb::BeHealed));
    let mut with_bad_verb = snapshot;
    with_bad_verb["players"]["p1"]["base"]["phaseRestrictions"] = json!(["notAVerb"]);
    assert!(e.load_state(&with_bad_verb).is_err());
    // Default state stays off the restriction hash stream.
    let reset = State::default();
    e.card_restricted(&reset, 0, RestrictionVerb::BeHealed);
}