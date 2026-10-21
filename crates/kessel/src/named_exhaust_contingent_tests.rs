use super::*;
use serde_json::json;
fn ability() -> Ability {
    serde_json::from_value(profile()["abilities"]["card"][0].clone()).unwrap()
}
#[test]
fn named_exhaust_complete_source_and_typed_projection_agree() {
    let raw = profile();
    assert_eq!(prepare(&raw), Ok(true));
    assert!(registration(&ability()));
    let Some(Effect::Simultaneous { effects }) = exhaust_group(&ability()) else {
        panic!("missing closed group")
    };
    assert_eq!(effects.len(), 2);
    for (index, effect) in effects.iter().enumerate() {
        let Effect::Select {
            target_resolver,
            effect,
        } = effect
        else {
            panic!("missing selector")
        };
        assert_eq!(
            target_resolver.owner,
            if index == 0 { "self" } else { "opponent" }
        );
        assert_eq!(target_resolver.card_type, "unit");
        assert_eq!(target_resolver.zone.as_deref(), Some("anyAttackable"));
        assert_eq!(
            target_resolver.prompt_title.as_deref(),
            Some(if index == 0 {
                "Choose a friendly unit to exhaust"
            } else {
                "Choose an enemy unit to exhaust"
            })
        );
        assert!(matches!(effect.as_ref(), Effect::Exhaust { .. }));
    }
}
#[test]
fn named_exhaust_raw_header_metadata_helpers_and_all_registrations_are_closed() {
    for key in [
        "setCode",
        "sourcePath",
        "sourceHash",
        "className",
        "baseClass",
        "id",
        "internalName",
        "status",
        "card",
        "dataHash",
        "keywordValues",
        "requiredCapabilities",
        "diagnostics",
        "abilities",
    ] {
        let mut raw = profile();
        raw.as_object_mut().unwrap().remove(key);
        assert!(prepare(&raw).is_err(), "missing {key}");
        let mut raw = profile();
        raw[key] = if key == "setCode" {
            json!(null)
        } else {
            json!("unknown")
        };
        assert!(prepare(&raw).is_err(), "modified {key}");
    }
    for key in [
        "historyHelpers",
        "attachCondition",
        "piloting",
        "unfamiliar",
    ] {
        let mut raw = profile();
        raw[key] = json!({});
        assert!(prepare(&raw).is_err());
    }
    for key in [
        "types",
        "keywords",
        "cost",
        "aspects",
        "traits",
        "text",
        "arena",
        "pilotText",
    ] {
        let mut raw = profile();
        raw["card"][key] = json!(["unsupported"]);
        assert!(prepare(&raw).is_err(), "metadata {key}");
    }
    let mut raw = profile();
    raw["abilities"]["card"]
        .as_array_mut()
        .unwrap()
        .push(json!({"trigger":"onAttack","effect":{"kind":"noAction"}}));
    assert!(prepare(&raw).is_err());
    let mut raw = profile();
    raw["abilities"]["card"][0]["effect"]["hiddenPayload"] = json!({});
    assert!(
        prepare(&raw).is_err(),
        "raw noAction payload cannot be erased by unit enum serde"
    );
    assert!(prepare(&json!({"unexpectedEnvelope":profile()})).is_err());
    assert_eq!(
        prepare(&json!({"unrelated":"ordinary producer-free source"})),
        Ok(false)
    );
}
#[test]
fn named_exhaust_raw_aliases_and_binding_shapes_cannot_bypass_source_proof() {
    let mut raw = profile();
    raw["setCode"] = json!("ALT_001");
    raw["card"]["setId"] = json!({"set":"ALT","number":1});
    assert!(prepare(&raw).is_err());
    for path in ["friendlyUnit", "enemyUnit"] {
        assert!(prepare(&json!({"effect":{"kind":"giveAdvantage","target":{"kind":"ref","path":["namedTargets",path]}}})).is_err());
    }
    let mut raw = profile();
    raw["abilities"]["card"][0]["targetResolvers"][0]["name"] = json!("otherBinding");
    assert!(prepare(&raw).is_err());
    let mut raw = profile();
    raw["abilities"]["card"][0]["ifYouDo"]["ifYouDoCondition"] =
        json!({"kind":"const","value":true});
    assert!(prepare(&raw).is_err());
}
#[test]
fn named_exhaust_typed_hidden_flags_and_continuation_mutations_reject() {
    let original = ability();
    for flag in 0..3 {
        let mut a = original.clone();
        match flag {
            0 => a.attached_host = true,
            1 => a.piloting_ability = true,
            _ => a.pilot_unit_ability = true,
        };
        assert!(!registration(&a));
        assert!(exhaust_group(&a).is_none());
    }
    for key in [
        "ifYouDo",
        "targetResolvers",
        "effect",
        "costs",
        "condition",
        "optional",
        "requiresConfirmation",
        "coordinate",
        "trigger",
        "contextTitle",
    ] {
        let mut raw = profile()["abilities"]["card"][0].clone();
        raw[key] = match key {
            "ifYouDo" => json!(null),
            "targetResolvers" | "costs" | "contextTitle" => json!([]),
            "effect" => json!({"kind":"ready","target":"chosen"}),
            "condition" => json!({"kind":"const","value":true}),
            "trigger" => json!("onAttack"),
            _ => json!(true),
        };
        // costs/contextTitle were empty in the valid registration; give them real content.
        if key == "costs" {
            raw[key] = json!([{"kind":"exhaustSelf"}]);
        }
        if key == "contextTitle" {
            raw[key] = json!([{"kind":"const","value":"changed"}]);
        }
        if let Ok(a) = serde_json::from_value::<Ability>(raw) {
            assert!(!registration(&a), "{key}");
        }
    }
    let mut a = ability();
    a.if_you_do.as_mut().unwrap().if_you_do_condition = Some(json!({"kind":"const","value":true}));
    assert!(!registration(&a));
}
#[test]
fn named_exhaust_binding_presence_and_last_event_outcome_are_independent() {
    use ParentOutcome::*;
    for friendly in [0, 17] {
        for enemy in [0, 29] {
            for parent in [
                NoEvents,
                LastEvent {
                    resolved_or_replacement_resolved: false,
                },
                LastEvent {
                    resolved_or_replacement_resolved: true,
                },
            ] {
                let actual = Bindings::new(friendly, enemy).unwrap().recipient(parent);
                assert_eq!(
                    actual,
                    if friendly != 0
                        && enemy != 0
                        && matches!(
                            parent,
                            LastEvent {
                                resolved_or_replacement_resolved: true
                            }
                        )
                    {
                        Some(17)
                    } else {
                        None
                    }
                );
            }
        }
    }
    // Both selections may exist even when only one exhaust actually resolved.
    // The LAST generated event controls the continuation, not any success/count.
    let selected = Bindings::new(17, 29).unwrap();
    assert_eq!(
        selected.recipient(LastEvent {
            resolved_or_replacement_resolved: true
        }),
        Some(17)
    );
    assert_eq!(selected.recipient(NoEvents), None);
    assert!(Bindings::new(17, 17).is_err());
    assert!(Bindings::new(256, 29).is_err());
    assert!(Bindings::new(17, u16::MAX).is_err());
}
#[test]
fn named_exhaust_raw_inspection_is_bounded() {
    let mut v = json!("leaf");
    for _ in 0..130 {
        v = json!([v]);
    }
    assert!(prepare(&v).is_err());
    assert!(prepare(&Value::Array(vec![Value::Null; 65_537])).is_err());
}

#[test]
fn named_exhaust_actual_source_asymmetric_selected_exhausted_cases() {
    // Source engine both-seat witnesses: ready alternatives allow selecting an
    // exhausted card while preserving its named binding. Generated failed events
    // stay in the parent event array used by the ifYouDo callback.
    let b = Bindings::new(17, 29).unwrap();
    assert_eq!(
        b.recipient(parent_outcome([Some(false), Some(true)])),
        Some(17),
        "exhausted friendly; enemy event succeeds"
    );
    assert_eq!(
        b.recipient(parent_outcome([Some(true), Some(false)])),
        None,
        "exhausted enemy; last generated event fails"
    );
    assert_eq!(
        b.recipient(parent_outcome([Some(false), Some(false)])),
        None
    );
    assert_eq!(
        b.recipient(parent_outcome([Some(true), Some(true)])),
        Some(17)
    );
    assert_eq!(
        parent_outcome([Some(true), None]),
        ParentOutcome::LastEvent {
            resolved_or_replacement_resolved: true
        }
    );
    assert_eq!(parent_outcome([None, None]), ParentOutcome::NoEvents);
    assert_eq!(
        Bindings::new(17, 0)
            .unwrap()
            .recipient(parent_outcome([Some(true), None])),
        None
    );
    assert_eq!(
        Bindings::new(0, 29)
            .unwrap()
            .recipient(parent_outcome([None, Some(true)])),
        None
    );
}

#[test]
fn named_exhaust_lookup_code_is_opaque_but_metadata_aliases_remain_closed() {
    let mut raw = profile();
    raw["setCode"] = json!("ZZZ_999");
    assert_eq!(prepare(&raw), Ok(true));
    raw["aliasOf"] = json!("ASH_231");
    assert!(prepare(&raw).is_err());
    raw.as_object_mut().unwrap().remove("aliasOf");
    raw["card"]["setId"] = json!({"set":"ZZZ","number":999});
    assert!(prepare(&raw).is_err());
}

#[test]
fn named_exhaust_reserves_stored_program_sentinel_at_capacity() {
    assert_eq!(checked_program_id(0), Ok(0));
    assert_eq!(checked_program_id(u16::MAX as usize - 1), Ok(u16::MAX - 1));
    assert!(checked_program_id(u16::MAX as usize).is_err());
    assert!(checked_program_id(u16::MAX as usize + 1).is_err());
    assert!(checked_program_id(usize::MAX).is_err());
}
