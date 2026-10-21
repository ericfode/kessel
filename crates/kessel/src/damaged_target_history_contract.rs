//! Closed actual-source envelope for the physical damage-target watcher query.
use crate::ability_ir::{Ability, Expr, Instruction, Program};
use serde_json::Value;
const PROFILE: &str = r###"{"setCode":"ASH_188","sourcePath":"game/cards/08_ASH/events/GalvanizedLeap.ts","sourceHash":"c9dce44bd1ecb19201aa8b4ec7004058ffa053d609e226fbc211cfcc9104837d","abilities":{"leader":[],"unit":[],"card":[{"trigger":"event","title":"Ready a unit that was damaged this phase","costs":[],"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","predicate":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"historyQuery","watcher":"damageDealtThisPhase","binding":"damageDealtThisPhaseWatcher","operation":"wasUnitDamagedThisPhase","args":[{"kind":"ref","path":["card"]}],"readAt":"expressionEvaluation"}}},"effect":{"kind":"ready","target":"chosen"}}]},"className":"GalvanizedLeap","baseClass":"EventCard","stateWatchers":[{"kind":"phaseHistory","binding":"damageDealtThisPhaseWatcher","watcher":"damageDealtThisPhase","sharedScope":"game","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded","implementation":"DamageDealtThisPhaseWatcher","queryContract":"pinnedWatcherApi"}],"id":"9496039852","internalName":"galvanized-leap","status":"supported","card":{"title":"Galvanized Leap","subtitle":"","cost":4,"hp":null,"power":null,"text":"Ready a unit that was damaged this phase.","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"9496039852","aspects":["aggression"],"traits":["force"],"keywords":[],"types":["event"],"setId":{"set":"ASH","number":188},"setCodes":[{"set":"ASH","number":188}],"internalName":"galvanized-leap"},"dataHash":"4983f86b6ec1eeea03c876edb0befecbc008af7b79c97a3300910e9fd1fe452e","keywordValues":{},"requiredCapabilities":["binary:&&","cardType:event","ir:binary","ir:historyQuery","ir:query","ir:ready","ir:ref","query:isUnit","ref:card","targetType:unit","targetZone:anyAttackable","trigger:event"],"diagnostics":[]}"###;
pub(crate) fn profile() -> Value {
    serde_json::from_str(PROFILE).expect("closed damage history source")
}
pub(crate) fn marker(v: &Value) -> bool {
    crate::ability_ir::contains(v, |v| {
        v["operation"] == "wasUnitDamagedThisPhase"
            || v["className"] == "GalvanizedLeap"
            || v["sourcePath"] == "game/cards/08_ASH/events/GalvanizedLeap.ts"
            || v["setCode"] == "ASH_188"
            || v["id"] == "9496039852"
            || v["internalName"] == "galvanized-leap"
    })
}
pub(crate) fn prepare(raw: &Value) -> Result<bool, String> {
    if !marker(raw) {
        return Ok(false);
    }
    if raw == &profile() {
        Ok(true)
    } else {
        Err("damaged-target history requires the complete canonical source, metadata and registration profile".into())
    }
}
pub(crate) fn ability(a: &Ability) -> bool {
    if !marker(&serde_json::to_value(a).unwrap_or(Value::Null)) {
        return false;
    }
    let expected: Ability = serde_json::from_value(profile()["abilities"]["card"][0].clone())
        .expect("closed damage ability");
    serde_json::to_value(a).ok() == serde_json::to_value(expected).ok()
}
pub(crate) fn requires(p: &Program) -> bool {
    fn expression(e: &Expr) -> bool {
        match e.original() {
            Expr::HistoryQuery(q) => q.kind() == Ok(crate::phase_history::QueryKind::DamagedCard),
            Expr::Binary { left, right, .. } => expression(left) || expression(right),
            _ => false,
        }
    }
    p.instructions.iter().any(|i| match i {
        Instruction::Select(r) => r.predicate.as_ref().is_some_and(expression),
        Instruction::Conditional { condition, .. } => expression(condition),
        _ => false,
    })
}

/// The complete damaged-target source profile authorize strict physical pool rows. The
/// printing wrapper is the sole field compiled_pool adds to original metadata;
/// compare before typed metadata projection so null omissions and extras survive.
pub(crate) fn validate_selected_pool_metadata(source: &Value, row: &Value) -> Result<(), String> {
    let damaged = prepare(source)?;
    if !damaged {
        if marker(row) {
            return Err(
                "selected damaged-target identity requires complete canonical source".into(),
            );
        }
        return Ok(());
    }
    let mut expected = source["card"].clone();
    expected["setCode"] = source["setCode"].clone();
    if row != &expected {
        return Err(
            "selected closed-source pool row requires complete canonical raw metadata".into(),
        );
    }
    Ok(())
}
