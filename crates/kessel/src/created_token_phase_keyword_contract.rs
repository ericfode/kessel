//! Closed creation-event result followed by a phase keyword on its exact token.
//! Factory defaults, raw metadata, continuation binding and native dependencies
//! are checked before lowering; no generic eventResult evaluator is introduced.
use crate::ability_ir::{Ability, Instruction, Program};
use serde_json::{json, Value};
const SOURCE_HASH: &str = "6658f3027c842eac57033101b423e236e43e10654adc2f7dab4703675a6d2c32";
const SOURCE: &str = r###"{"setCode":"ASH_091","sourcePath":"game/cards/08_ASH/events/BuyTime.ts","sourceHash":"6658f3027c842eac57033101b423e236e43e10654adc2f7dab4703675a6d2c32","abilities":{"leader":[],"unit":[],"card":[{"trigger":"event","title":"Create a Mandalorian token and give it Sentinel for this Phase","costs":[],"effect":{"kind":"createTokenUnit","tokenType":"mandalorian","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},"then":{"title":"Give it Sentinel for this phase","costs":[],"effect":{"kind":"gainKeyword","target":{"kind":"eventResult","contextBinding":{"id":"callback:925:0","role":"then","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"},"collection":"resolvedEvents","eventIndex":0,"path":["generatedTokens"],"optionalAccess":[2]},"duration":"phase","keyword":"sentinel","amount":{"kind":"const","value":1}}}}]},"className":"BuyTime","baseClass":"EventCard","id":"6201936455","internalName":"buy-time","status":"supported","card":{"title":"Buy Time","subtitle":"","cost":3,"hp":null,"power":null,"text":"Create a Mandalorian token and give it Sentinel for this phase. (Enemy units in its arena must attack a Sentinel when they attack you.)","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"6201936455","aspects":["vigilance"],"traits":["tactic"],"keywords":["sentinel"],"types":["event"],"setId":{"set":"ASH","number":91},"setCodes":[{"set":"ASH","number":91}],"internalName":"buy-time"},"dataHash":"ba5b4af5b78917688967c869fade1d899bfda21893526cbf4b31ff69283d0566","keywordValues":{},"requiredCapabilities":["ability:then","cardType:event","ir:const","ir:createTokenUnit","ir:eventResult","ir:gainKeyword","keyword:sentinel","trigger:event"],"diagnostics":[]}"###;
fn invalid() -> String {
    "created-token phase keyword requires complete source, resolved creation binding and native token profiles".into()
}
pub(crate) fn marker(v: &Value) -> bool {
    v["sourcePath"] == "game/cards/08_ASH/events/BuyTime.ts"
        || v["sourceHash"] == SOURCE_HASH
        || crate::ability_ir::contains(v, |n| {
            n["kind"] == "eventResult" && n["path"] == json!(["generatedTokens"])
        })
}
fn compiled() -> Result<Program, String> {
    let mut a: Value = serde_json::from_str(SOURCE).expect("source proof");
    let ability = a["abilities"]["card"][0].as_object_mut().unwrap();
    ability.remove("then");
    let a: Ability =
        serde_json::from_value(a["abilities"]["card"][0].clone()).map_err(|e| e.to_string())?;
    let mut p = Program::compile(&a)?;
    let pc = p
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::Group(_)))
        .ok_or_else(invalid)?;
    if p.instructions.len() != pc + 2
        || !matches!(p.instructions.last(), Some(Instruction::EndStep))
    {
        return Err(invalid());
    }
    p.instructions.insert(
        pc + 1,
        Instruction::GrantCreatedTokenPhaseKeyword {
            creation_pc: pc as u8,
            keyword: "sentinel".into(),
        },
    );
    p.event_end += 1;
    p.executable = Err("created-token continuation uses shared interpreter");
    Ok(p)
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions
        .iter()
        .any(|i| matches!(i, Instruction::GrantCreatedTokenPhaseKeyword { .. }))
}
pub(crate) fn program(p: &Program) -> bool {
    compiled()
        .ok()
        .is_some_and(|expected| serde_json::to_value(p).ok() == serde_json::to_value(expected).ok())
}
pub(crate) fn prepare(entry: &Value, catalogue: &Value) -> Result<Option<Program>, String> {
    if !marker(entry) {
        return Ok(None);
    }
    if !crate::admission::SourceContextPolicy::Closed.accepts(entry) {
        return Err(invalid());
    }
    let mut actual = entry.clone();
    let mut expected: Value = serde_json::from_str(SOURCE).expect("source proof");
    if !actual["setCode"]
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
    {
        return Err(invalid());
    }
    actual
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("setCode");
    expected.as_object_mut().unwrap().remove("setCode");
    if actual != expected {
        return Err(invalid());
    }
    crate::defeat_token_creation_contract::dependencies(catalogue)?;
    Ok(Some(compiled()?))
}

#[cfg(test)]
#[path = "created_token_phase_keyword_tests.rs"]
mod tests;
