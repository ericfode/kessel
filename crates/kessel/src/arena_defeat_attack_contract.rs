//! Closed on-attack defeat of every other physical space unit. Target capture,
//! simultaneous defeat, attachment cleanup and attack continuation stay with
//! the existing shared runtime; this module authorizes no new mutation owner.
use crate::ability_ir::{Ability, ArenaListTarget, Effect, Instruction, Program, Sides};
use serde_json::{json, Value};
fn reference(name: &str) -> Value {
    json!({"kind":"ref","path":[name]})
}
fn constant(value: Value) -> Value {
    json!({"kind":"const","value":value})
}
fn binary(op: &str, left: Value, right: Value) -> Value {
    json!({"kind":"binary","op":op,"left":left,"right":right})
}
/// Exact emitted getArenaUnits({arena:SpaceArena,otherThan:source}) projection.
/// The nullable branch excludes one physical source, never every same-title copy.
fn target_value() -> Value {
    let space = constant(json!("spaceArena"));
    let scope = binary(
        "&&",
        constant(json!(true)),
        binary(
            "||",
            json!({"kind":"unary","op":"!","value":space.clone()}),
            binary(
                "||",
                binary("===", space.clone(), constant(json!("anyArena"))),
                binary(
                    "===",
                    json!({"kind":"ref","path":["card","zoneName"]}),
                    space,
                ),
            ),
        ),
    );
    let other = binary(
        "||",
        json!({"kind":"unary","op":"!","value":reference("source")}),
        binary("!==", reference("card"), reference("source")),
    );
    json!({"kind":"arenaList","receiver":reference("game"),"predicate":binary("&&",scope,other)})
}
pub(crate) fn scope(list: &ArenaListTarget) -> bool {
    serde_json::to_value(list).is_ok_and(|v| v == target_value())
}
pub(crate) fn registration(a: &Ability) -> bool {
    crate::admission::plain_registration(a)
        && a.trigger == "onAttack"
        && !a.attacker_must_survive
        && a.hooks.is_empty()
        && a.target_resolver.is_none()
        && a.attached_context.is_none()
        && a.if_you_do_condition.is_none()
        && matches!(&a.effect,Effect::Defeat{target:crate::ability_ir::Target::ArenaList(list)} if scope(list))
}
pub(crate) fn marker(v: &Value) -> bool {
    v["trigger"] == "onAttack"
        && v["effect"]["kind"] == "defeat"
        && v["effect"]["target"]["kind"] == "arenaList"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(s) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    crate::admission::SourceContextPolicy::Closed.accepts(e)
        && e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == json!(["unit"])
        && e["card"]["arena"] == "space"
        && e["card"]["unique"] == false
        && e["card"]["keywords"] == json!(["sentinel"])
        && e["keywordValues"] == json!({"sentinel":1})
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && s.leader.is_empty()
        && s.unit.is_empty()
        && s.card.len() == 1
        && registration(&s.card[0])
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| e["card"][k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
}
pub(crate) fn requires_global_order(p: &Program) -> bool {
    p.trigger=="onAttack"&&p.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::Defeat{target:crate::ability_ir::Target::ArenaList(list)}) if scope(list)))
}
