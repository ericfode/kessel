//! Closed source proof and lowering for an own-hand discard followed by a
//! self-only attack-lifetime power modifier. Mutation uses the shared typed
//! attack-grant owner; no phase-lifetime or physical identity is inferred here.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program, Target};
use serde_json::{json, Value};
const REASON: &str =
    "attack-lifetime discard requires the complete closed ordinary unit source profile";
fn profile() -> Value {
    serde_json::from_str(r###"{"setCode":"ASH_172","sourcePath":"game/cards/08_ASH/units/RazorCrestOutfittedArmament.ts","sourceHash":"ad7057b85f368cca178e6344e8b0d243e932a41589e19dcd8bbae9dcd3da9a57","abilities":{"leader":[],"unit":[],"card":[{"trigger":"onAttack","title":"Discard a card from your hand. If you do, this unit gets +2/+0 for this attack","costs":[],"optional":true,"effect":{"kind":"discardOwnHand","amount":{"kind":"const","value":1},"target":"ownPlayer"},"ifYouDo":{"title":"This unit gets +2/+0 for this attack","costs":[],"effect":{"kind":"modifyStats","target":"self","duration":"attack","power":{"kind":"const","value":2},"hp":{"kind":"const","value":0}}}}]},"className":"RazorCrestOutfittedArmament","baseClass":"NonLeaderUnitCard","id":"4046328280","internalName":"razor-crest#outfitted-armament","status":"supported","card":{"title":"Razor Crest","subtitle":"Outfitted Armament","cost":4,"hp":5,"power":3,"text":"Saboteur (When this unit attacks, ignore Sentinel and defeat the defender's Shields.)\nOn Attack: You may discard a card from your hand. If you do, this unit gets +2/+0 for this attack.","deployBox":null,"epicAction":"","unique":true,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"4046328280","aspects":["aggression"],"traits":["mandalorian","vehicle","transport"],"arena":"space","keywords":["saboteur"],"types":["unit"],"setId":{"set":"ASH","number":172},"setCodes":[{"set":"ASH","number":172}],"internalName":"razor-crest#outfitted-armament"},"dataHash":"d053b751e41e2f4c10f9d3d28f734d66d229ff670cd1cccf4cb60ae5c216fb4e","keywordValues":{"saboteur":1},"requiredCapabilities":["ability:ifYouDo","cardType:unit","ir:const","ir:discardOwnHand","ir:modifyStats","keyword:saboteur","trigger:onAttack"],"diagnostics":[]}"###).expect("reviewed complete source profile")
}
fn attack_power_two(v: &Value) -> bool {
    v["kind"] == "modifyStats"
        && v["duration"] == "attack"
        && v["power"] == json!({"kind":"const","value":2})
}
fn marked(v: &Value) -> bool {
    v["sourceHash"]=="ad7057b85f368cca178e6344e8b0d243e932a41589e19dcd8bbae9dcd3da9a57"
 ||v["className"]=="RazorCrestOutfittedArmament"||v["internalName"]=="razor-crest#outfitted-armament"
 // Existing closed InitiateAttack factories also contain attack-duration +2
 // descriptors. They are not this discarded-hand continuation or new wrapper.
 ||v["effect"]["kind"]=="discardOwnHand"&&attack_power_two(&v["ifYouDo"]["effect"])
 ||v["kind"]=="attackModifiers"&&v["effects"].as_array().is_some_and(|a|a.len()==1&&attack_power_two(&a[0]))
}
pub(crate) fn prepare(raw: &Value) -> Result<bool, String> {
    let mut pending = vec![(raw, 0usize)];
    let mut count = 0usize;
    let mut marker = false;
    while let Some((v, depth)) = pending.pop() {
        count += 1;
        if count > 65536 || depth > 128 {
            return Err("attack-lifetime source inspection budget exceeded".into());
        }
        marker |= marked(v);
        match v {
            Value::Object(o) => pending.extend(o.values().map(|v| (v, depth + 1))),
            Value::Array(a) => pending.extend(a.iter().map(|v| (v, depth + 1))),
            _ => {}
        }
        if count + pending.len() > 65536 {
            return Err("attack-lifetime source inspection budget exceeded".into());
        }
    }
    if !marker {
        return Ok(false);
    }
    let mut actual = raw.clone();
    let Some(object) = actual.as_object_mut() else {
        return Err(REASON.into());
    };
    if !object
        .get("setCode")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
    {
        return Err(REASON.into());
    }
    object.remove("setCode");
    let mut expected = profile();
    expected.as_object_mut().unwrap().remove("setCode");
    if actual != expected {
        return Err(REASON.into());
    }
    Ok(true)
}
pub(crate) fn typed_marker(a: &Ability) -> bool {
    let v = serde_json::to_value(a).expect("serializable ability");
    let mut pending = vec![&v];
    while let Some(v) = pending.pop() {
        if marked(v) {
            return true;
        }
        match v {
            Value::Object(o) => pending.extend(o.values()),
            Value::Array(a) => pending.extend(a),
            _ => {}
        }
    }
    false
}
pub(crate) fn modifier(e: &Effect) -> bool {
    matches!(e,Effect::AttackModifiers{effects} if effects.len()==1&&matches!(&effects[0],Effect::ModifyStats{target:Target::Name(n),duration,power,hp} if n=="self"&&duration=="attack"&&matches!(power.original(),Expr::Const{value} if value==&json!(2))&&matches!(hp.original(),Expr::Const{value} if value==&json!(0))))
}
pub(crate) fn compiled() -> Result<Program, String> {
    let mut ability: Ability = serde_json::from_value(profile()["abilities"]["card"][0].clone())
        .map_err(|e| e.to_string())?;
    let child = ability.if_you_do.take().expect("reviewed child");
    let mut program = Program::compile(&ability)?;
    if !matches!(program.instructions.pop(), Some(Instruction::EndStep)) {
        return Err("attack-lifetime continuation boundary missing".into());
    }
    let gate = program.instructions.len();
    program.instructions.push(Instruction::IfResult { skip: 0 });
    program
        .instructions
        .push(Instruction::Apply(Effect::AttackModifiers {
            effects: vec![child.effect],
        }));
    program.instructions[gate] = Instruction::IfResult {
        skip: program.instructions.len(),
    };
    program.event_end = u8::try_from(program.instructions.len())
        .map_err(|_| "attack-lifetime program exceeds capacity")?;
    program.instructions.push(Instruction::EndStep);
    program.executable =
        crate::program_backend(&serde_json::to_vec(&program).map_err(|e| e.to_string())?);
    Ok(program)
}
pub(crate) fn program(p: &Program) -> bool {
    compiled()
        .is_ok_and(|expected| serde_json::to_value(p).ok() == serde_json::to_value(expected).ok())
}

/// Program instructions store their catalogue index plus one. Reserve that
/// representation before the catalogue binds any program to this source.
pub(crate) fn checked_program_id(count: usize) -> Result<u16, String> {
    let stored = count
        .checked_add(1)
        .and_then(|n| u16::try_from(n).ok())
        .ok_or("attack-lifetime stored program index exceeds capacity")?;
    Ok(stored - 1)
}
