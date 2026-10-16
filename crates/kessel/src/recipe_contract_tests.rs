use super::*;
use serde_json::json;

fn registration(code: &str) -> Ability {
    let catalogue: Value = serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();
    serde_json::from_value(catalogue["cards"].as_array().unwrap().iter()
        .find(|entry| entry["setCode"] == code).unwrap()["abilities"]["card"][0].clone()).unwrap()
}
fn compiled(effect: Value) -> Result<Program, String> {
    Program::compile(&serde_json::from_value(json!({"trigger":"event", "title":"Group contract", "effect":effect})).unwrap())
}
fn effect(kind: &str, target: &str) -> Value {
    json!({"kind":kind, "target":target, "amount":{"kind":"const", "value":1}})
}
fn group(effects: Vec<Value>) -> Value { json!({"kind":"simultaneous", "effects":effects}) }

#[test]
fn force_recipe_lowering_preserves_payment_result_and_capture_windows() {
    let drain = Program::compile(&registration("LOF_041")).unwrap();
    assert!(matches!(drain.instructions.as_slice(), [Instruction::Group(_), Instruction::EndStep]));
    assert_eq!(drain.event_end, 1);

    let wave = Program::compile(&registration("LOF_039")).unwrap();
    assert!(matches!(wave.instructions.as_slice(), [
        Instruction::Apply(Effect::UseForce { .. }), Instruction::IfResult { skip: 4 },
        Instruction::EndStep, Instruction::Apply(Effect::Defeat { target: Target::ArenaList(_) }),
        Instruction::EndStep,
    ]));
    assert_eq!(wave.event_end, 4);

    let shields = Program::compile(&registration("LOF_072")).unwrap();
    assert!(matches!(shields.instructions.as_slice(), [
        Instruction::Apply(Effect::UseForce { .. }), Instruction::IfResult { skip: 5 },
        Instruction::EndStep, Instruction::Select(_), Instruction::Group(Effect::GiveShield { .. }),
        Instruction::EndStep,
    ]));
    assert_eq!(shields.event_end, 5);
    for count in [4, 6] {
        let mut wrong = registration("LOF_072");
        wrong.if_you_do.as_mut().unwrap().target_resolver.as_mut().unwrap().count =
            Some(Expr::Const { value: count.into() });
        assert_eq!(Program::compile(&wrong).unwrap_err(), "selectors above three require exact Force Shield continuation");
    }
}

#[test]
fn weakness_recipes_keep_shared_recipient_and_separate_birth_selection() {
    let picador = Program::compile(&registration("HMW_199")).unwrap();
    assert!(matches!(picador.instructions.as_slice(), [
        Instruction::Group(Effect::CreateTokenUnit { .. }), Instruction::Select(_),
        Instruction::Apply(Effect::GiveWeakness { .. }), Instruction::EndStep,
    ]));
    for code in ["HMW_100", "HMW_202"] {
        let program = Program::compile(&registration(code)).unwrap();
        assert!(matches!(program.instructions.as_slice(), [
            Instruction::Select(_), Instruction::Group(_), Instruction::EndStep,
        ]), "{code}");
        let mut wrong = registration(code);
        wrong.target_resolver.as_mut().unwrap().optional = true;
        assert!(Program::compile(&wrong).is_err(), "optional captured recipient {code}");
    }
}

#[test]
fn conditional_group_storage_uses_largest_branch_and_keeps_capacity_diagnostic() {
    let eight = group(vec![effect("heal", "self"); 8]);
    let branch = group(vec![eight.clone(); 3]);
    let alternatives = json!({"kind":"conditional", "condition":{"kind":"const", "value":true},
        "onTrue":branch, "onFalse":branch});
    // Either branch captures 24 records. The independent eight fit the physical
    // budget exactly; summing mutually exclusive branches would reject this.
    assert!(compiled(group(vec![alternatives.clone(), eight.clone()])).is_ok());
    let error = compiled(group(vec![alternatives, eight, effect("heal", "self")])).unwrap_err();
    assert_eq!(error, "event group record budget exceeds capacity");
}

#[test]
fn nested_group_timing_fences_preserve_diagnostic_precedence() {
    let damage = effect("damage", "self");
    let draw = effect("draw", "ownPlayer");
    let shield = effect("giveShield", "chosen");
    let hand_branch = json!({"kind":"conditional", "condition":{"kind":"const", "value":false},
        "onTrue":draw, "onFalse":{"kind":"noAction"}});
    assert_eq!(compiled(group(vec![damage.clone(), hand_branch.clone()])).unwrap_err(),
        "mixed damage and hand effects require dedicated engine witnesses");
    assert_eq!(compiled(group(vec![damage, group(vec![hand_branch, shield])])).unwrap_err(),
        "mixed damage and newly attached tokens/lasting modifiers require dedicated engine witnesses");
}

#[test]
fn ordered_ground_scope_preserves_candidate_binding_and_exact_emitted_fence() {
    let scope = json!({"kind":"binary", "op":"&&", "left":{"kind":"const", "value":true},
        "right":{"kind":"binary", "op":"||", "left":{"kind":"unary", "op":"!", "value":{"kind":"const", "value":"groundArena"}},
            "right":{"kind":"binary", "op":"||", "left":{"kind":"binary", "op":"===", "left":{"kind":"const", "value":"groundArena"}, "right":{"kind":"const", "value":"anyArena"}},
                "right":{"kind":"binary", "op":"===", "left":{"kind":"ref", "path":["card", "zoneName"]}, "right":{"kind":"const", "value":"groundArena"}}}}});
    let list = json!({"kind":"arenaList", "receiver":{"kind":"ref", "path":["game"]},
        "predicate":scope, "controllerOrder":"playerFirst"});
    let exact: ArenaListTarget = serde_json::from_value(list.clone()).unwrap();
    assert!(exact.validate_ordered_ground().is_ok());
    // Equivalent predicates and outer-target aliases are separate contracts, even
    // when a particular board could happen to enumerate the same units.
    for predicate in [
        json!({"kind":"const", "value":true}),
        { let mut p = scope.clone(); p["right"]["right"]["right"]["left"]["path"] = json!(["target", "zoneName"]); p },
        { let mut p = scope.clone(); p["right"]["right"]["right"]["right"]["value"] = json!("spaceArena"); p },
    ] {
        let mut invalid = list.clone(); invalid["predicate"] = predicate;
        let invalid: ArenaListTarget = serde_json::from_value(invalid).unwrap();
        assert_eq!(invalid.validate_ordered_ground().unwrap_err(),
            "ordered lists require global controller partitions and exact ground scope");
    }
    for order in ["playerFirst", "opponentFirst", "seatOrder"] {
        let mut ordered = list.clone(); ordered["controllerOrder"] = json!(order);
        assert!(serde_json::from_value::<ArenaListTarget>(ordered).unwrap().validate_ordered_ground().is_ok());
    }
}
