use crate::round_limit_contract::{self as contract, Registration};
use crate::round_limit_state::{Handle, Instance, Ledger, MAX_NATIVE_INSTANCES};
use serde_json::{json, Value};
fn catalogue() -> Value {
    serde_json::from_slice(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
}
fn entry(code: &str) -> Value {
    catalogue()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
#[test]
fn round_limit_complete_five_profiles_project_into_whole_native_definitions() {
    let expected = [
        ("ASH_047", Registration::PlayedUpgradeOnSelf, 0),
        ("ASH_032", Registration::FriendlyUnitNonlethalDamage, 0),
        ("ASH_160", Registration::EnemyGroundAttackAtBase, 0),
        ("HMW_062", Registration::EnemyWeaknessUnitDefeated, 1),
        ("SOR_115", Registration::OtherUniqueUnitDefeated, 0),
    ];
    let runtime = crate::Engine::from_catalogue_value(&catalogue()).unwrap();
    for (code, kind, ordinal) in expected {
        let source = entry(code);
        let projection = contract::prepare(&source).unwrap().unwrap();
        assert_eq!(
            (projection.registration, projection.ordinal),
            (kind, ordinal)
        );
        assert_eq!(
            projection.abilities["card"].as_array().unwrap().len(),
            source["abilities"]["card"].as_array().unwrap().len()
        );
        assert!(!contract::marker(&projection.abilities).unwrap());
        assert!(!runtime.rejected_cards().unwrap().contains_key(code));
    }
}
#[test]
fn round_limit_whole_source_metadata_and_callback_boundaries_remain_closed() {
    for code in ["ASH_047", "ASH_032", "ASH_160", "HMW_062", "SOR_115"] {
        let source = entry(code);
        let ordinal = if code == "HMW_062" { 1 } else { 0 };
        for changed in [
            "max", "factory", "scope", "shared", "extra", "native", "side", "keyword", "helper",
            "callback",
        ] {
            let mut bad = source.clone();
            match changed {
                "max" => bad["abilities"]["card"][ordinal]["limit"]["args"] = json!([2]),
                "factory" => {
                    bad["abilities"]["card"][ordinal]["limit"]["factory"] = json!("perPhase")
                }
                "scope" => bad["abilities"]["card"][ordinal]["limit"]["scope"] = json!("card"),
                "shared" => {
                    bad["abilities"]["card"][ordinal]["limit"]["sharedAcrossHooks"] = json!(false)
                }
                "extra" => bad["abilities"]["card"][ordinal]["limit"]["unknown"] = json!(true),
                "native" => bad["card"]["pilotBox"] = json!({}),
                "side" => bad["abilities"]["leader"] = json!([{}]),
                "keyword" => bad["keywordValues"]["support"] = json!(1),
                "helper" => bad["privatePrograms"] = json!([{}]),
                "callback" => {
                    bad["abilities"]["card"][ordinal]["hooks"][0]["predicate"]["unknown"] =
                        json!(true)
                }
                _ => unreachable!(),
            }
            assert!(contract::prepare(&bad).is_err(), "{code} {changed}");
        }
        let mut bad = source;
        bad["card"]["roundLimit"] = json!({});
        assert!(contract::prepare(&bad).is_err());
    }
    // Existing collection limits bound execution, not ability use.
    for code in ["HMW_015", "JTL_180", "JTL_247"] {
        assert!(!contract::marker(&entry(code)).unwrap());
    }
    for raw in [
        json!({"limit":null}),
        json!({"kind":"abilityLimit"}),
        json!({"trigger":"roundLimitPlayedUpgrade"}),
        json!({"roundLimitHandle":1}),
    ] {
        assert!(contract::marker(&raw).unwrap());
        assert!(contract::prepare(&raw).is_err());
    }
}
#[test]
fn round_limit_shared_instance_decline_per_player_zone_clone_and_reset() {
    let mut ledger = Ledger::default();
    let old = ledger.native(4, 0).unwrap();
    assert_eq!(old, ledger.native(4, 0).unwrap());
    assert!(ledger.native(4,1).is_err());
    assert!(ledger.native(6,2).is_err());
    let other_registration = ledger.native(6, 1).unwrap();
    let other_copy = ledger.native(5, 0).unwrap();
    assert_ne!(old, other_registration);
    assert_ne!(old, other_copy);
    // Capture/decline is not consume. Sibling contexts share exactly this handle.
    assert!(ledger.available(old, 0).unwrap());
    ledger.consume(old, 0).unwrap();
    assert!(!ledger.available(old, 0).unwrap());
    assert!(ledger.available(old, 1).unwrap());
    ledger.consume(old, 1).unwrap();
    assert!(ledger.consume(old, 0).is_err());
    assert!(ledger.available(other_registration, 0).unwrap());
    assert!(ledger.available(other_copy, 0).unwrap());
    ledger.retire_native(4).unwrap();
    let fresh = ledger.native(4, 0).unwrap();
    assert_ne!(fresh, old);
    assert!(ledger.available(fresh, 0).unwrap());
    ledger.collect(&[old, fresh]).unwrap();
    assert!(!ledger.available(old, 0).unwrap());
    ledger.consume(fresh, 0).unwrap();
    ledger.round_ended();
    assert!(ledger.available(fresh, 0).unwrap());
    assert!(
        !ledger.available(old, 0).unwrap(),
        "unregistered old shared object does not reset"
    );
    ledger.collect(&[fresh]).unwrap();
    assert!(ledger.instance(old).is_err());
}
#[test]
fn round_limit_layout_copy_hash_capacity_and_quiescent_identity() {
    assert_eq!(MAX_NATIVE_INSTANCES, 905);
    assert_eq!(std::mem::size_of::<Instance>(), 4);
    assert_eq!(std::mem::size_of::<Ledger>(), 3622);
    let mut empty_hash = vec![];
    Ledger::default().hash_values(|v| empty_hash.push(v));
    assert!(empty_hash.is_empty());
    let mut ledger = Ledger::default();
    assert!(ledger.native(0, 0).is_err());
    assert!(ledger.native(256, 0).is_err());
    assert!(ledger.available(Handle::NONE, 0).is_err());
    let handle = ledger.native(255, 1).unwrap();
    ledger.consume(handle, 0).unwrap();
    let copy = ledger;
    let mut hash = vec![];
    copy.hash_values(|v| hash.push(v));
    let mut changed = copy;
    changed.consume(handle, 1).unwrap();
    let mut changed_hash = vec![];
    changed.hash_values(|v| changed_hash.push(v));
    assert_ne!(hash, changed_hash);
    assert_eq!(ledger, copy);
    assert!(ledger.normalize_quiescent(&[handle]).is_err());
    let mut alternate = Ledger::default();
    alternate.native(1, 0).unwrap();
    let alternate_handle = alternate.native(255, 1).unwrap();
    alternate.consume(alternate_handle, 0).unwrap();
    ledger.normalize_quiescent(&[]).unwrap();
    alternate.normalize_quiescent(&[]).unwrap();
    assert_eq!(ledger, alternate);
    let mut full = Ledger::default();
    for i in 0..MAX_NATIVE_INSTANCES {
        let handle = full.native(7, 0).unwrap();
        assert_eq!(usize::from(handle.raw()), i + 1);
        full.retire_native(7).unwrap();
    }
    let before = full;
    assert!(full.native(7, 0).unwrap_err().contains("capacity"));
    assert_eq!(full, before);
}

#[test]
fn round_limit_event_projection_keeps_exact_source_predicates() {
    use crate::round_limit_runtime::{qualifies, Event};
    use Registration::*;
    for player in [0, 1] {
        let other = 1 - player;
        let upgrade = Event::PlayedUpgrade {
            player,
            card_is_upgrade: true,
            attach_target: 7,
        };
        assert!(qualifies(PlayedUpgradeOnSelf, 7, player, upgrade, true).unwrap());
        assert!(!qualifies(PlayedUpgradeOnSelf, 8, player, upgrade, true).unwrap());
        assert!(!qualifies(PlayedUpgradeOnSelf, 7, other, upgrade, true).unwrap());
        let damage = Event::UnitDamage {
            controller: player,
            card_is_unit: true,
            will_defeat: false,
        };
        assert!(qualifies(FriendlyUnitNonlethalDamage, 7, player, damage, true).unwrap());
        assert!(!qualifies(FriendlyUnitNonlethalDamage, 7, player, damage, false).unwrap());
        assert!(!qualifies(FriendlyUnitNonlethalDamage, 7, other, damage, true).unwrap());
        let lethal = Event::UnitDamage {
            controller: player,
            card_is_unit: true,
            will_defeat: true,
        };
        assert!(!qualifies(FriendlyUnitNonlethalDamage, 7, player, lethal, true).unwrap());
        let base = Event::UnitDamage {
            controller: player,
            card_is_unit: false,
            will_defeat: false,
        };
        assert!(!qualifies(FriendlyUnitNonlethalDamage, 7, player, base, true).unwrap());
        let declaration = Event::AttackDeclared {
            attacker_controller: other,
            ground_attacker: true,
            any_base_target: true,
        };
        assert!(qualifies(EnemyGroundAttackAtBase, 7, player, declaration, true).unwrap());
        assert!(!qualifies(EnemyGroundAttackAtBase, 7, other, declaration, true).unwrap());
        let space = Event::AttackDeclared {
            attacker_controller: other,
            ground_attacker: false,
            any_base_target: true,
        };
        assert!(!qualifies(EnemyGroundAttackAtBase, 7, player, space, true).unwrap());
        let unit_attack = Event::AttackDeclared {
            attacker_controller: other,
            ground_attacker: true,
            any_base_target: false,
        };
        assert!(!qualifies(EnemyGroundAttackAtBase, 7, player, unit_attack, true).unwrap());
        let death = Event::Defeated {
            card: 9,
            controller: other,
            was_unit: true,
            unique: true,
            had_weakness: true,
        };
        assert!(qualifies(EnemyWeaknessUnitDefeated, 7, player, death, true).unwrap());
        assert!(!qualifies(EnemyWeaknessUnitDefeated, 7, other, death, true).unwrap());
        assert!(qualifies(OtherUniqueUnitDefeated, 7, player, death, true).unwrap());
        assert!(qualifies(OtherUniqueUnitDefeated, 7, other, death, true).unwrap());
        assert!(!qualifies(OtherUniqueUnitDefeated, 9, player, death, true).unwrap());
        let nonunique = Event::Defeated {
            card: 9,
            controller: other,
            was_unit: true,
            unique: false,
            had_weakness: false,
        };
        assert!(!qualifies(OtherUniqueUnitDefeated, 7, player, nonunique, true).unwrap());
        assert!(!qualifies(EnemyWeaknessUnitDefeated, 7, player, nonunique, true).unwrap());
    }
}

#[test]
fn round_limit_integrated_model_handles_copy_hash_and_layout() {
    use crate::model::{State,ResolutionContext,DeathFrame,DeathTrigger,MAX_DEATH_DEPTH};
    use std::mem::size_of;
    let base=State::default();let zero=base.zobrist();
    let mut seeded=base;let h=seeded.round_limits.native(7,0).unwrap();seeded.round_limits.consume(h,0).unwrap();
    assert_ne!(seeded.zobrist(),zero);
    for slot in 0..MAX_DEATH_DEPTH+4 {
        let mut control=seeded;if slot==1 {control.support.pending=true;}let mut s=control;match slot {0=>s.round_limit=h,1=>s.support.context.round_limit=h,2=>s.death_pending.entries[0].round_limit=h,3=>s.death_window.entries[0].round_limit=h,n=>s.death_frames[n-4].context.round_limit=h}
        assert_ne!(s.zobrist(),control.zobrist());let copy=s;assert_eq!(copy,s);assert_eq!(copy.zobrist(),s.zobrist());
        let captured=ResolutionContext::capture(&s);let mut restored=State::default();captured.restore(&mut restored);assert_eq!(restored.round_limit,s.round_limit);
    }
    for i in 0..MAX_DEATH_DEPTH {let mut s=seeded;s.death_frames[i].window.entries[0].round_limit=h;assert_ne!(s.zobrist(),seeded.zobrist());}
    println!("ROUND_LIMIT_ACTUAL_LAYOUT State={} Context={} Frame={} Trigger={} Ledger={}",size_of::<State>(),size_of::<ResolutionContext>(),size_of::<DeathFrame>(),size_of::<DeathTrigger>(),size_of::<Ledger>());
}

#[test]
fn round_limit_proven_programs_commit_once_after_selection_and_bind_typed_trigger() {
    use crate::ability_ir::Instruction;
    for code in ["ASH_047","ASH_032","ASH_160","HMW_062","SOR_115"] {
        let entry=entry(code);
        let Some(projection)=contract::prepare(&entry).unwrap() else {panic!("source profile absent")};
        let (_,programs)=projection.compile(0).unwrap();let p=&programs[projection.ordinal as usize];
        assert_eq!(contract::Registration::from_trigger(&p.trigger),Some(projection.registration));
        assert_eq!(p.instructions.iter().filter(|i|matches!(i,Instruction::Commit)).count(),1);
        let commit=p.instructions.iter().position(|i|matches!(i,Instruction::Commit)).unwrap();
        assert_eq!(commit,usize::from(matches!(p.instructions.first(),Some(Instruction::Select(_)))));
        assert!(p.executable.is_err());
    }
}

#[test]
fn round_limit_all_persistent_roots_block_reuse_and_invalid_roots_are_atomic() {
    use crate::model::{State,MAX_DEATH_DEPTH};
    use crate::round_limit_runtime as rt;
    let mut s=State::default();let h=s.round_limits.native(9,0).unwrap();
    s.round_limits.consume(h,0).unwrap();s.round_limits.retire_native(9).unwrap();
    s.round_limit=h;s.support.context.round_limit=h;
    s.death_pending.entries[63].round_limit=h;s.death_window.entries[63].round_limit=h;
    for f in &mut s.death_frames {f.context.round_limit=h;f.window.entries[63].round_limit=h;}
    assert_eq!(rt::roots(&s).len(),4+2*MAX_DEATH_DEPTH);
    rt::collect(&mut s).unwrap();let fresh=rt::native(&mut s,9,Registration::PlayedUpgradeOnSelf).unwrap();
    assert_ne!(fresh,h);assert!(!s.round_limits.available(h,0).unwrap());
    s.round_limits.consume(fresh,0).unwrap();let saved=crate::model::ResolutionContext::capture(&s);
    // Restoring the execution context cannot roll the shared consumed ledger back.
    s.round_limits.retire_native(9).unwrap();saved.restore(&mut s);assert!(!s.round_limits.available(fresh,0).unwrap());
    let mut invalid=State::default();invalid.round_limit=h;let before=invalid;
    assert!(rt::collect(&mut invalid).is_err());assert_eq!(invalid,before);
}

#[test]
fn round_limit_exact905_bound_retains650_roots_and_reuses_only_unrooted_retirement() {
    use crate::model::{State,MAX_DEATH_DEPTH};use crate::round_limit_runtime as rt;
    let mut s=State::default();let mut held=Vec::new();
    for _ in 0..650 {let h=s.round_limits.native(7,0).unwrap();s.round_limits.consume(h,0).unwrap();s.round_limits.retire_native(7).unwrap();held.push(h);}
    let mut i=0;
    for w in [&mut s.death_pending,&mut s.death_window].into_iter().chain(s.death_frames.iter_mut().map(|f|&mut f.window)) {for e in &mut w.entries {e.round_limit=held[i];i+=1;}}
    s.round_limit=held[i];i+=1;s.support.context.round_limit=held[i];i+=1;
    for f in &mut s.death_frames {f.context.round_limit=held[i];i+=1;}
    assert_eq!(i,650);assert_eq!(rt::roots(&s).len(),650);assert_eq!(MAX_DEATH_DEPTH,8);
    for source in 1..=255 {s.round_limits.native(source,0).unwrap();}
    assert_eq!(s.round_limits.len,905);rt::collect(&mut s).unwrap();
    for h in &held {assert!(!s.round_limits.available(*h,0).unwrap());}
    let replace=s.round_limits.current(42,0).unwrap();s.round_limits.retire_native(42).unwrap();rt::collect(&mut s).unwrap();
    let fresh=s.round_limits.native(42,0).unwrap();assert_eq!(replace,fresh);assert!(s.round_limits.available(fresh,0).unwrap());
    for h in held {assert!(!s.round_limits.available(h,0).unwrap());}
}
#[test]
fn round_limit_damage_fact_known_false_copy_hash_index_and_clear_boundaries() {
    use crate::round_limit_state::DamageFacts;
    let mut a=DamageFacts::default();assert!(a.get(0).is_err());let mut empty=Vec::new();a.hash_values(|n|empty.push(n));assert!(empty.is_empty());
    a.capture(0,false).unwrap();a.capture(255,true).unwrap();assert!(!a.get(0).unwrap());assert!(a.get(255).unwrap());assert!(a.get(254).is_err());let copy=a;assert_eq!(copy,a);let before=a;assert!(a.capture(256,true).is_err());assert_eq!(a,before);assert!(a.get(256).is_err());
    let mut words=Vec::new();a.hash_values(|n|words.push(n));let mut zero=Vec::new();DamageFacts::default().hash_values(|n|zero.push(n));assert_ne!(words,zero);
    let mut s=crate::State::default();let hash=s.zobrist();s.event_group.damage_facts=a;assert_ne!(s.zobrist(),hash);let copy=s;assert_eq!(copy.zobrist(),s.zobrist());s.event_group=Default::default();assert_eq!(s.zobrist(),hash);
}

#[test]
fn round_limit_printed_source_claim_cannot_be_erased_by_when_played_projection() {
    for code in ["ASH_047","ASH_032","ASH_160","HMW_062","SOR_115"] {
        let mut raw=entry(code);let ordinal=usize::from(code=="HMW_062");let ability=&mut raw["abilities"]["card"][ordinal];ability.as_object_mut().unwrap().remove("limit");ability.as_object_mut().unwrap().remove("hooks");ability["trigger"]=json!("whenPlayed");
        assert!(!contract::marker(&raw).unwrap());assert!(contract::prepare(&raw).is_err());let catalogue=json!({"schema":crate::ability_ir::SCHEMA,"cards":[raw]});let loaded=crate::ability_ir::Catalogue::load(&catalogue).unwrap();assert!(!loaded.cards.contains_key(code));
    }
}

// Exact actual ordinary139 initial state from the independent ROUND-WIRE-6
// diagnostic packet (result SHA800da796b22b85b57de6cddc8a3789d492c565095f2ddef2bf41a2237a2a03dd).
// The selected code set is copied from pool SHA637a4f8a0d4394e176b7b8bc956785b5ba521a69e3d4ca80ca97b01529548c28.
// Reconstructing its metadata below preserves selection, not the original pool bytes/hash.
pub(crate) fn ordinary139_wire_engine(backend:crate::RulesBackend,producer:bool)->crate::Engine {
    const CODES:&[&str]=&["ASH_019", "ASH_025", "ASH_034", "ASH_037", "ASH_041", "ASH_061", "ASH_065", "ASH_078", "ASH_083", "ASH_086", "ASH_087", "ASH_111", "ASH_116", "ASH_117", "ASH_126", "ASH_131", "ASH_141", "ASH_151", "ASH_158", "ASH_173", "ASH_179", "ASH_184", "ASH_190", "ASH_191", "ASH_197", "ASH_201", "ASH_207", "ASH_216", "ASH_226", "ASH_242", "ASH_260", "ASH_261", "HMW_041", "HMW_093", "HMW_116", "HMW_117", "HMW_155", "HMW_163", "HMW_178", "HMW_258", "HMW_268", "IBH_007", "IBH_008", "IBH_012", "IBH_022", "IBH_037", "IBH_055", "IBH_056", "IBH_057", "IBH_063", "IBH_075", "IBH_076", "IC27_052", "IC27_073", "IC27_097", "IC27_175", "IC27_178", "IC27_180", "IC27_189", "JTL_063", "JTL_069", "JTL_075", "JTL_085", "JTL_104", "JTL_212", "JTL_251", "LAW_033", "LAW_043", "LAW_097", "LAW_098", "LAW_143", "LAW_146", "LAW_164", "LAW_175", "LAW_180", "LAW_189", "LAW_253", "LOF_066", "LOF_084", "LOF_093", "LOF_112", "LOF_119", "LOF_144", "LOF_168", "LOF_192", "LOF_213", "LOF_230", "LOF_236", "LOF_254", "LOF_255", "LOF_256", "SEC_005", "SEC_028", "SEC_044", "SEC_080", "SEC_118", "SEC_161", "SEC_167", "SEC_214", "SEC_216", "SEC_237", "SEC_251", "SHD_037", "SHD_060", "SHD_079", "SHD_110", "SHD_130", "SHD_152", "SHD_200", "SHD_236", "SHD_257", "SOR_014", "SOR_023", "SOR_046", "SOR_095", "SOR_112", "SOR_135", "SOR_163", "SOR_168", "SOR_178", "SOR_210", "TOKEN_ADVANTAGE", "TOKEN_BATTLE_DROID", "TOKEN_BEAST", "TOKEN_EXPERIENCE", "TOKEN_MANDALORIAN", "TOKEN_SHIELD", "TOKEN_THE_FORCE", "TS26_024", "TWI_057", "TWI_077", "TWI_111", "TWI_133", "TWI_174", "TWI_185", "TWI_209", "TWI_226", "TWI_237", "TWI_253"];
    let raw=catalogue();let all=crate::Engine::from_catalogue_value(&raw).unwrap();
    let mut pool=all.compiled_pool().unwrap();
    pool["cards"].as_array_mut().unwrap().retain(|c|CODES.contains(&c["setCode"].as_str().unwrap()) || (producer&&c["setCode"]=="ASH_047"));
    assert_eq!(pool["cards"].as_array().unwrap().len(),139+usize::from(producer));
    let mut e=crate::Engine::from_compiled_pool_bytes(&serde_json::to_vec(&pool).unwrap(),&serde_json::to_vec(&raw).unwrap()).unwrap();
    e.set_rules_backend(backend).unwrap();e
}
pub(crate) fn ordinary139_wire_state()->Value {serde_json::from_str(r###"{"supportStateVersion":1,"supportGrants":[],"registrationOrder":{"version":1,"cards":["c1","c3","c60"]},"phaseHistory":{"version":1,"entered":[],"played":[],"playCounts":[0,0]},"unitEntryStateVersion":1,"inactiveUnitEntryStats":[],"forceStateVersion":1,"hiddenHistoryVersion":1,"arenaUnitOrder":{"ground":["c3"],"space":["c1","c60"]},"nextCardId":106,"round":1,"phase":"action","initiative":"p1","initiativeClaimed":false,"prevPassed":false,"activePlayer":"p1","actor":"p1","winner":null,"players":{"p1":{"base":{"id":"c50","setCode":"ASH_019","name":"fortress-of-the-great-mothers","damage":0,"hp":30,"epicUsed":false},"leader":{"id":"c51","setCode":"SOR_014","name":"sabine-wren#galvanized-revolutionary","deployed":false,"exhausted":false,"epicUsed":false},"hand":[{"id":"c0","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile"},{"id":"c9","setCode":"ASH_061","name":"strike-team-vanguard"},{"id":"c16","setCode":"ASH_190","name":"peridea-bandit"},{"id":"c32","setCode":"HMW_116","name":"ewok-brigade"}],"deck":[{"id":"c24","setCode":"ASH_261","name":"noti-mobile-pod"},{"id":"c7","setCode":"LAW_253","name":"alliance-xwing"},{"id":"c21","setCode":"ASH_242","name":"death-trooper-squad"},{"id":"c40","setCode":"HMW_178","name":"desperate-nantex"},{"id":"c36","setCode":"HMW_163","name":"champions-of-endor"},{"id":"c46","setCode":"HMW_268","name":"offworld-jawa"},{"id":"c33","setCode":"HMW_155","name":"filthy-dianoga"},{"id":"c17","setCode":"ASH_190","name":"peridea-bandit"},{"id":"c29","setCode":"HMW_093","name":"coastal-catamarans"},{"id":"c4","setCode":"SOR_095","name":"battlefield-marine"},{"id":"c25","setCode":"ASH_261","name":"noti-mobile-pod"},{"id":"c6","setCode":"LAW_253","name":"alliance-xwing"},{"id":"c47","setCode":"HMW_268","name":"offworld-jawa"},{"id":"c31","setCode":"HMW_116","name":"ewok-brigade"},{"id":"c5","setCode":"SOR_095","name":"battlefield-marine"},{"id":"c27","setCode":"HMW_093","name":"coastal-catamarans"},{"id":"c49","setCode":"IBH_007","name":"echo-coordinator"},{"id":"c39","setCode":"HMW_178","name":"desperate-nantex"}],"resources":[{"id":"c34","setCode":"HMW_155","name":"filthy-dianoga","exhausted":false},{"id":"c45","setCode":"HMW_268","name":"offworld-jawa","exhausted":false},{"id":"c35","setCode":"HMW_155","name":"filthy-dianoga","exhausted":false},{"id":"c12","setCode":"ASH_131","name":"dinosaur-turtle","exhausted":false},{"id":"c37","setCode":"HMW_163","name":"champions-of-endor","exhausted":false},{"id":"c26","setCode":"ASH_261","name":"noti-mobile-pod","exhausted":false},{"id":"c10","setCode":"ASH_061","name":"strike-team-vanguard","exhausted":false},{"id":"c8","setCode":"LAW_253","name":"alliance-xwing","exhausted":false},{"id":"c20","setCode":"ASH_201","name":"open-circle-ace","exhausted":false},{"id":"c41","setCode":"HMW_178","name":"desperate-nantex","exhausted":false},{"id":"c15","setCode":"ASH_190","name":"peridea-bandit","exhausted":false},{"id":"c30","setCode":"HMW_116","name":"ewok-brigade","exhausted":false},{"id":"c43","setCode":"HMW_258","name":"ryloth-revolutionary-rider","exhausted":false},{"id":"c22","setCode":"ASH_242","name":"death-trooper-squad","exhausted":false},{"id":"c28","setCode":"HMW_093","name":"coastal-catamarans","exhausted":false},{"id":"c18","setCode":"ASH_201","name":"open-circle-ace","exhausted":false},{"id":"c13","setCode":"ASH_131","name":"dinosaur-turtle","exhausted":false},{"id":"c19","setCode":"ASH_201","name":"open-circle-ace","exhausted":false},{"id":"c11","setCode":"ASH_061","name":"strike-team-vanguard","exhausted":false},{"id":"c38","setCode":"HMW_163","name":"champions-of-endor","exhausted":false},{"id":"c2","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile","exhausted":false},{"id":"c44","setCode":"HMW_258","name":"ryloth-revolutionary-rider","exhausted":false},{"id":"c23","setCode":"ASH_242","name":"death-trooper-squad","exhausted":false},{"id":"c48","setCode":"IBH_007","name":"echo-coordinator","exhausted":false},{"id":"c42","setCode":"HMW_258","name":"ryloth-revolutionary-rider","exhausted":false},{"id":"c14","setCode":"ASH_131","name":"dinosaur-turtle","exhausted":false}],"discard":[],"ground":[{"id":"c3","setCode":"SOR_095","name":"battlefield-marine","hiddenQualified":false,"damage":0,"exhausted":true,"power":3,"hp":3,"upgrades":[]}],"space":[{"id":"c1","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile","hiddenQualified":false,"damage":0,"exhausted":false,"power":3,"hp":1,"upgrades":[]}],"forceToken":{"id":"c52","held":false},"out":false},"p2":{"base":{"id":"c103","setCode":"ASH_019","name":"fortress-of-the-great-mothers","damage":0,"hp":30,"epicUsed":false},"leader":{"id":"c104","setCode":"SOR_014","name":"sabine-wren#galvanized-revolutionary","deployed":false,"exhausted":false,"epicUsed":false},"hand":[{"id":"c91","setCode":"HMW_163","name":"champions-of-endor"},{"id":"c93","setCode":"HMW_178","name":"desperate-nantex"},{"id":"c59","setCode":"LAW_253","name":"alliance-xwing"},{"id":"c56","setCode":"SOR_095","name":"battlefield-marine"}],"deck":[{"id":"c75","setCode":"ASH_242","name":"death-trooper-squad"},{"id":"c70","setCode":"ASH_190","name":"peridea-bandit"},{"id":"c98","setCode":"HMW_268","name":"offworld-jawa"},{"id":"c76","setCode":"ASH_242","name":"death-trooper-squad"},{"id":"c67","setCode":"ASH_131","name":"dinosaur-turtle"},{"id":"c79","setCode":"ASH_261","name":"noti-mobile-pod"},{"id":"c61","setCode":"LAW_253","name":"alliance-xwing"},{"id":"c65","setCode":"ASH_131","name":"dinosaur-turtle"},{"id":"c54","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile"},{"id":"c57","setCode":"SOR_095","name":"battlefield-marine"},{"id":"c77","setCode":"ASH_261","name":"noti-mobile-pod"},{"id":"c97","setCode":"HMW_258","name":"ryloth-revolutionary-rider"},{"id":"c72","setCode":"ASH_201","name":"open-circle-ace"},{"id":"c102","setCode":"IBH_007","name":"echo-coordinator"},{"id":"c66","setCode":"ASH_131","name":"dinosaur-turtle"},{"id":"c80","setCode":"HMW_093","name":"coastal-catamarans"},{"id":"c63","setCode":"ASH_061","name":"strike-team-vanguard"},{"id":"c68","setCode":"ASH_190","name":"peridea-bandit"},{"id":"c99","setCode":"HMW_268","name":"offworld-jawa"}],"resources":[{"id":"c95","setCode":"HMW_258","name":"ryloth-revolutionary-rider","exhausted":false},{"id":"c71","setCode":"ASH_201","name":"open-circle-ace","exhausted":false},{"id":"c62","setCode":"ASH_061","name":"strike-team-vanguard","exhausted":false},{"id":"c53","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile","exhausted":false},{"id":"c69","setCode":"ASH_190","name":"peridea-bandit","exhausted":false},{"id":"c90","setCode":"HMW_163","name":"champions-of-endor","exhausted":false},{"id":"c58","setCode":"SOR_095","name":"battlefield-marine","exhausted":false},{"id":"c73","setCode":"ASH_201","name":"open-circle-ace","exhausted":false},{"id":"c74","setCode":"ASH_242","name":"death-trooper-squad","exhausted":false},{"id":"c101","setCode":"IBH_007","name":"echo-coordinator","exhausted":false},{"id":"c92","setCode":"HMW_178","name":"desperate-nantex","exhausted":false},{"id":"c82","setCode":"HMW_093","name":"coastal-catamarans","exhausted":false},{"id":"c100","setCode":"HMW_268","name":"offworld-jawa","exhausted":false},{"id":"c86","setCode":"HMW_155","name":"filthy-dianoga","exhausted":false},{"id":"c64","setCode":"ASH_061","name":"strike-team-vanguard","exhausted":false},{"id":"c85","setCode":"HMW_116","name":"ewok-brigade","exhausted":false},{"id":"c87","setCode":"HMW_155","name":"filthy-dianoga","exhausted":false},{"id":"c81","setCode":"HMW_093","name":"coastal-catamarans","exhausted":false},{"id":"c89","setCode":"HMW_163","name":"champions-of-endor","exhausted":false},{"id":"c83","setCode":"HMW_116","name":"ewok-brigade","exhausted":false},{"id":"c78","setCode":"ASH_261","name":"noti-mobile-pod","exhausted":false},{"id":"c84","setCode":"HMW_116","name":"ewok-brigade","exhausted":false},{"id":"c94","setCode":"HMW_178","name":"desperate-nantex","exhausted":false},{"id":"c88","setCode":"HMW_155","name":"filthy-dianoga","exhausted":false},{"id":"c96","setCode":"HMW_258","name":"ryloth-revolutionary-rider","exhausted":false},{"id":"c55","setCode":"ASH_191","name":"shin-hatis-fiend-fighter#compact-and-agile","exhausted":false}],"discard":[],"ground":[],"space":[{"id":"c60","setCode":"LAW_253","name":"alliance-xwing","hiddenQualified":false,"damage":0,"exhausted":true,"power":2,"hp":3,"upgrades":[]}],"forceToken":{"id":"c105","held":false},"out":false}}}"###).unwrap()}
#[test]
fn round_limit_explicit_empty_wire_preserves_actual_ordinary139_state_and_legacy_hash() {
    use crate::RulesBackend;
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let e=ordinary139_wire_engine(backend,false);let legacy=ordinary139_wire_state();
        let old=e.load_state(&legacy).unwrap();assert_eq!(e.snapshot(&old),legacy);
        let mut modern=legacy.clone();modern["roundLimitState"]=json!({"version":1,"registrations":[]});
        let mut loaded=e.load_state(&modern).unwrap();assert_eq!(e.snapshot(&loaded),modern);
        assert_eq!(loaded.zobrist(),old.zobrist());assert_ne!(loaded,old);
        let copy=loaded;assert_eq!(e.snapshot(&copy),modern);assert_eq!(copy,loaded);
        crate::round_limit_runtime::collect(&mut loaded).unwrap();loaded.round_limits.round_ended();
        assert_eq!(e.snapshot(&loaded),modern);assert_eq!(loaded.zobrist(),old.zobrist());
        assert_eq!(e.load_state(&e.snapshot(&loaded)).unwrap(),loaded);
        assert_eq!(std::mem::size_of::<crate::State>(),38312);
        assert_eq!(std::mem::size_of::<crate::model::ResolutionContext>(),124);
    }
}

#[test]
fn round_limit_optional_wire_is_strict_and_survives_imported_rows_departing() {
    use crate::{RulesBackend,model::{Card,Seat}};
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let e=ordinary139_wire_engine(backend,false);let producer=ordinary139_wire_engine(backend,true);
        let legacy=ordinary139_wire_state();assert!(producer.load_state(&legacy).unwrap_err().contains("requires exact roundLimitState"));
        let mut modern=legacy.clone();modern["roundLimitState"]=json!({"version":1,"registrations":[]});
        for wire in [json!({"version":2,"registrations":[]}),json!({"version":"1","registrations":[]}),json!({"version":1}),json!({"version":1,"registrations":null}),json!({"version":1,"registrations":[],"extra":false}),json!({"version":1,"registrations":[{}]})] {
            let mut bad=modern.clone();bad["roundLimitState"]=wire;assert!(e.load_state(&bad).is_err());
        }
        let mut with_row=modern.clone();let wire_id=with_row["nextCardId"].as_u64().unwrap();
        with_row["nextCardId"]=json!(wire_id+1);
        with_row["players"]["p1"]["hand"].as_array_mut().unwrap().push(json!({"id":format!("c{wire_id}"),"setCode":"ASH_047","name":entry("ASH_047")["internalName"]}));
        with_row["roundLimitState"]["registrations"]=json!([{"card":format!("c{wire_id}"),"side":"card","ordinal":0,"used":[0,0]}]);
        let physical=producer.load_state(&with_row).unwrap();let id=(wire_id+1) as crate::model::CardId;
        let mut missing=with_row.clone();missing["roundLimitState"]["registrations"]=json!([]);
        assert!(producer.load_state(&missing).unwrap_err().contains("complete exact current native registration counters"));
        // Unknown physical sources remain rejected by the selected139 wire.
        assert!(e.load_state(&with_row).is_err());
        // Native cleanup boundary: start with the actual139 explicit-empty
        // import, temporarily retain a known physical registration, then retire
        // its last row. This artificial identity injection/deletion is not
        // an admitted gameplay history or a completed source-game witness.
        let mut imported=e.load_state(&modern).unwrap();assert!(imported.round_limit_wire_present);
        imported.next_id=id+1;*imported.card_mut(id)=*physical.card(id);
        imported.player_mut(Seat::P1).hand.push(id).unwrap();imported.round_limits.native(id,0).unwrap();
        assert_eq!(e.snapshot(&imported)["roundLimitState"]["registrations"].as_array().unwrap().len(),1);
        imported.player_mut(Seat::P1).hand.remove(id);*imported.card_mut(id)=Card::default();
        imported.round_limits.retire_native(id).unwrap();crate::round_limit_runtime::collect(&mut imported).unwrap();
        let empty=e.snapshot(&imported);assert_eq!(empty["roundLimitState"],json!({"version":1,"registrations":[]}));
        let loaded=e.load_state(&empty).unwrap();assert_eq!(e.snapshot(&loaded),empty);assert_eq!(loaded,imported);
        let mut without=imported;without.round_limit_wire_present=false;assert_eq!(without.zobrist(),imported.zobrist());
    }
}
