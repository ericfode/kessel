//! Closed captured-defeat predicate followed by native token-unit creation.
//! Raw source/helper defaults and both native dependencies are checked before
//! lowering. The false branch skips the creation instruction entirely: a zero-
//! amount creation window is not equivalent to the printed absent branch.
use crate::ability_ir::{Ability, Expr, Instruction, Program};
use crate::defeat_cause::DefeatCause;
use serde_json::{json, Value};

pub(crate) const QUERY: &str = "capturedDefeatWasNonAttack";
const SOURCE_HASH: &str = "abb6d32d10d0a69a179075857626851bcf5ce2a315985c7033c2c9c4c898bd10";
const SOURCE: &str = r###"{"setCode":"ASH_028","sourcePath":"game/cards/08_ASH/units/PazVizslaForABrighterFuture.ts","sourceHash":"abb6d32d10d0a69a179075857626851bcf5ce2a315985c7033c2c9c4c898bd10","abilities":{"leader":[],"unit":[],"card":[{"trigger":"whenDefeated","title":"If this unit wasn't defeated by combat damage, create 2 Mandalorian tokens","costs":[],"effect":{"kind":"conditional","condition":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["event","defeatSource","type"]},"right":{"kind":"const","value":"attack"}},"onTrue":{"kind":"nativeTokenUnitEventFactory","properties":{"tokenType":"mandalorian","amount":{"kind":"const","value":2},"controllers":"ownPlayer","entersReady":false},"defaults":{"enterPlayEffect":{"kind":"undefinedValue"}},"inputContext":{"kind":"nativeSystemInvocationContext","scope":"currentAbilityInvocation","source":"enclosingAbilitySource","player":"enclosingAbilityPlayer","triggerEvent":"context.event","triggerEventIdentity":"retainOriginal","propertyFactories":"nativeSystemPropertyGeneration"},"creationEvent":{"kind":"freshGameEvent","name":"onTokensCreated","identity":"newPerTargetPlayer","targetIteration":"nativeTargetArrayOrder","context":"inputContext","fields":{"player":"capturedTargetPlayer","amount":"generatedProperties.amount","tokenType":"generatedProperties.tokenType","entersReady":"generatedProperties.entersReady","enterPlayEffect":"generatedProperties.enterPlayEffect"},"fieldBindingStage":"superUpdateEventAddPropertiesToEvent","triggerEventRegister":"unchanged"},"updateEvent":{"after":"superUpdateEvent","regenerateProperties":true,"generatedTokens":{"initialValue":[],"allocationStage":"eventUpdateBeforeResolution","loop":{"indexInitial":0,"condition":"indexLessThanGeneratedPropertiesAmount","increment":1},"factory":"game.generateToken","owner":"capturedTargetPlayer","tokenType":"generatedProperties.tokenType"},"cancelledEventTokens":"retainUntilRoundCleanup"},"eventHandler":{"kind":"emptySourceHandler"},"contingentEventFactory":{"registeredAt":"eventUpdate","input":"creationEvent","iteration":"creationEvent.generatedTokensInOrder","system":"PutIntoPlaySystem","properties":{"controller":"capturedTargetPlayer","target":"iteratedGeneratedToken","entersReady":{"kind":"logicalOr","left":"creationEvent.entersReady","right":"capturedTargetPlayerHasTokenUnitsEnterPlayReady"},"entryType":"created","enterPlayEffect":"creationEvent.enterPlayEffect"},"context":"creationEvent.context","contextEvent":"originalInputContextEvent","order":"creationEvent.orderPlusOne"},"resultEvent":{"identity":"creationEvent","generatedTokensField":"creationEvent.generatedTokens","separateFrom":"inputContext.event","continuationBinding":"existingResolvedEventContextProtocol"}},"onFalse":{"kind":"noAction"}}}]},"className":"PazVizslaForABrighterFuture","baseClass":"NonLeaderUnitCard","id":"7107655724","internalName":"paz-vizsla#for-a-brighter-future","status":"supported","card":{"title":"Paz Vizsla","subtitle":"For a Brighter Future","cost":5,"hp":7,"power":4,"text":"Sentinel\nWhen Defeated: If this unit wasn't defeated by combat damage, create 2 Mandalorian tokens.","deployBox":null,"epicAction":"","unique":true,"rules":"“Combat damage” is only the damage dealt during the “End attack” step of an attack.\n","upgradePower":null,"upgradeHp":null,"id":"7107655724","aspects":["vigilance","command","heroism"],"traits":["mandalorian"],"arena":"ground","keywords":["sentinel"],"types":["unit"],"setId":{"set":"ASH","number":28},"setCodes":[{"set":"ASH","number":28}],"internalName":"paz-vizsla#for-a-brighter-future"},"dataHash":"07c813fa631018b2608e32d220498347299a7227e33068ee90d9855852584f0b","keywordValues":{"sentinel":1},"requiredCapabilities":["binary:!==","cardType:unit","ir:binary","ir:conditional","ir:const","ir:emptySourceHandler","ir:freshGameEvent","ir:logicalOr","ir:nativeSystemInvocationContext","ir:nativeTokenUnitEventFactory","ir:noAction","ir:ref","ir:undefinedValue","keyword:sentinel","ref:event.defeatSource.type","trigger:whenDefeated"],"diagnostics":[]}"###;
const MANDALORIAN: &str = r###"{"setCode":"TOKEN_MANDALORIAN","sourcePath":"game/cards/08_ASH/tokens/Mandalorian.ts","sourceHash":"8f6beea828b30ca108831ac3ec8ffbe7a5f7178808aabfc9ca521d1185fbeba9","abilities":{"leader":[],"unit":[],"card":[]},"className":"Mandalorian","baseClass":"TokenUnitCard","id":"8192010342","internalName":"mandalorian","status":"supported","card":{"title":"Mandalorian","subtitle":"","cost":0,"hp":2,"power":2,"text":"Shielded (When you create this token, give a Shield token to it.)","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"8192010342","aspects":["vigilance"],"traits":["mandalorian"],"arena":"ground","keywords":["shielded"],"types":["token","unit"],"setId":{"set":"ASH"},"internalName":"mandalorian"},"dataHash":"5b4ef3b80503a664b91f945e1558638f30ef4bf108631017b5d8fc5009798dda","keywordValues":{"shielded":1},"requiredCapabilities":["cardType:token","cardType:unit","keyword:shielded"],"diagnostics":[]}"###;
const SHIELD: &str = r###"{"setCode":"TOKEN_SHIELD","sourcePath":"game/cards/01_SOR/tokens/Shield.ts","sourceHash":"39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f","abilities":{"leader":[],"unit":[],"card":[]},"className":"Shield","baseClass":"TokenUpgradeCard","id":"8752877738","internalName":"shield","status":"unsupported","card":{"title":"Shield","subtitle":null,"cost":0,"hp":0,"power":0,"text":"If damage would be dealt to attached unit, prevent that damage. If you do, defeat a Shield token on it.","deployBox":null,"epicAction":null,"unique":false,"rules":null,"upgradePower":0,"upgradeHp":0,"id":"8752877738","aspects":[],"traits":["armor"],"keywords":[],"types":["token","upgrade"],"setId":{"set":"SOR"},"internalName":"shield"},"dataHash":"3fe37b5b4c214e1a6971c2ea5e7729a7cad9bf983a384c20b56c404cbf41208b","keywordValues":{},"requiredCapabilities":["cardType:token","cardType:upgrade"],"diagnostics":[{"code":"UNSUPPORTED_SOURCE","pattern":"class-state:highPriorityRemoval","file":"game/cards/01_SOR/tokens/Shield.ts","line":10,"column":1,"message":"Class-local persistent state requires conversion"},{"code":"UNSUPPORTED_SOURCE","pattern":"class-member:Constructor","file":"game/cards/01_SOR/tokens/Shield.ts","line":19,"column":1,"message":"Additional class behavior requires conversion"},{"code":"UNSUPPORTED_SOURCE","pattern":"method:isShield","file":"game/cards/01_SOR/tokens/Shield.ts","line":29,"column":1,"message":"Additional class method requires conversion"},{"code":"UNSUPPORTED_SOURCE","pattern":"class-member:GetAccessor","file":"game/cards/01_SOR/tokens/Shield.ts","line":33,"column":1,"message":"Additional class behavior requires conversion"},{"code":"UNSUPPORTED_SOURCE","pattern":"registrar:addDamageModificationAbility","message":"registrar:addDamageModificationAbility","file":"game/cards/01_SOR/tokens/Shield.ts","line":38,"column":9}]}"###;

fn source() -> Value {
    serde_json::from_str(SOURCE).expect("closed source proof")
}
fn invalid() -> String {
    "noncombat defeat token creation requires the complete mandatory Sentinel unit source and native Mandalorian/Shield profiles".into()
}

/// Marker detection is independent of a catalogue code or display title. Raw
/// source provenance keeps damaged variants from falling into another family.
pub(crate) fn marker(v: &Value) -> bool {
    v["sourceHash"] == SOURCE_HASH
        || crate::ability_ir::contains(v, |n| {
            (n["kind"] == "query" && n["name"] == QUERY)
                || (n["kind"] == "ref" && n["path"] == json!(["event", "defeatSource", "type"]))
                || (n["trigger"] == "whenDefeated"
                    && crate::ability_ir::contains(&n["effect"], |e| {
                        e["kind"] == "nativeTokenUnitEventFactory"
                    }))
        })
}

/// The event cause is captured before the physical source leaves play. Active
/// attack state and the current ability source are deliberately not inputs.
pub(crate) fn evaluate(cause: DefeatCause) -> Result<bool, String> {
    match cause {
        DefeatCause::Unknown => Err("captured defeat cause is missing".into()),
        DefeatCause::Attack => Ok(false),
        _ => Ok(true),
    }
}

pub(crate) fn predicate() -> Expr {
    Expr::Query {
        name: QUERY.into(),
        receiver: Box::new(Expr::Ref {
            path: vec!["source".into()],
        }),
        args: vec![],
    }
}
pub(crate) fn query(name: &str, receiver: &Expr, args: &[Expr]) -> bool {
    name == QUERY
        && args.is_empty()
        && matches!(receiver, Expr::Ref { path } if path == &["source"])
}

/// All raw fields are paired with their complete pinned source proof, including
/// diagnostics and native helper factory ordering/defaults. Only the lookup code
/// is irrelevant to semantics; no card-ID switch dispatches this contract.
fn source_entry(entry: &Value) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(entry) {
        return false;
    }
    let mut actual = entry.clone();
    let Some(object) = actual.as_object_mut() else {
        return false;
    };
    if !object
        .get("setCode")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
    {
        return false;
    }
    object.remove("setCode");
    let mut expected = source();
    expected.as_object_mut().unwrap().remove("setCode");
    actual == expected
}

/// Selected pools must provide exactly one copy of each full reviewed native
/// definition, including Shield's explicit native-only unsupported diagnostics.
pub(crate) fn dependencies(catalogue: &Value) -> Result<(), String> {
    let cards = catalogue["cards"].as_array().ok_or_else(invalid)?;
    for (code, text) in [("TOKEN_MANDALORIAN", MANDALORIAN), ("TOKEN_SHIELD", SHIELD)] {
        let found = cards
            .iter()
            .filter(|v| v["setCode"] == code)
            .collect::<Vec<_>>();
        let expected: Value = serde_json::from_str(text).expect("native dependency proof");
        if found.len() != 1 || *found[0] != expected {
            return Err(invalid());
        }
    }
    Ok(())
}

/// Compile only the already reviewed native creation leaf. The typed cause
/// condition is inserted as a closed instruction, not exposed as a generally
/// executable arbitrary event path or converted into a create-zero effect.
fn compiled() -> Result<Program, String> {
    let a: Ability = serde_json::from_value(json!({
        "trigger":"whenDefeated",
        "title":"If this unit wasn't defeated by combat damage, create 2 Mandalorian tokens",
        "costs":[],
        "effect":{"kind":"createTokenUnit","tokenType":"mandalorian",
            "amount":{"kind":"const","value":2},"target":"ownPlayer","entersReady":false}
    }))
    .map_err(|e| e.to_string())?;
    let mut p = Program::compile(&a)?;
    let end = p.instructions.len();
    p.instructions.insert(
        0,
        Instruction::Conditional {
            condition: predicate(),
            skip: end,
        },
    );
    p.event_end = u8::try_from(end).map_err(|_| "token creation program capacity exceeded")?;
    p.executable = Err("captured defeat token creation uses shared interpreter");
    Ok(p)
}

/// Standalone compiled-program/import closure. Merely possessing a query marker
/// does not permit an optional, nested or modified creation registration.
pub(crate) fn program(p: &Program) -> bool {
    compiled()
        .ok()
        .is_some_and(|expected| serde_json::to_value(p).ok() == serde_json::to_value(expected).ok())
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions.iter().any(|instruction| match instruction {
        Instruction::Conditional { condition, .. } => condition
            .any(|e| matches!(e, Expr::Query { name, .. } if name == QUERY).then_some(true)),
        _ => false,
    })
}

pub(crate) fn prepare(entry: &Value, catalogue: &Value) -> Result<Option<Program>, String> {
    if !marker(entry) {
        return Ok(None);
    }
    if !source_entry(entry) {
        return Err(invalid());
    }
    dependencies(catalogue)?;
    Ok(Some(compiled()?))
}
