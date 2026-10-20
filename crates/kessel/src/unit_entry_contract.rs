//! Captured unit-entry registrations. The event card is a physical binding,
//! independent of the ability source, chosen target, last result and defeat LKI.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program, Sides, Target};
use serde_json::{json, Value};
pub(crate) fn registration(a: &Ability) -> bool {
    let Ok(v) = serde_json::to_value(a) else {
        return false;
    };
    let mut expected:Ability=serde_json::from_value(json!({"trigger":"eventHooks","title":a.title,"costs":[],"contextTitle":[{"kind":"const","value":"Give "},{"kind":"ref","path":["unitEntryCard","title"]},{"kind":"const","value":" +1/+0 for this phase"}],"effect":{"kind":"modifyStats","target":{"kind":"ref","path":["unitEntryCard"]},"duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}},"hooks":[{"event":"onUnitEntersPlay","predicate":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"binding","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false},{"key":"controller","optional":false}]},"right":{"kind":"ref","path":["player"]}}}]})).expect("closed entry contract");
    // Callback identity is lexical provenance, not a semantic source offset.
    let id = &v["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"];
    if !id.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128) {
        return false;
    }
    let mut e = serde_json::to_value(&mut expected).unwrap();
    e["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"] = id.clone();
    a.title.len() <= 256 && v == e
}
pub(crate) fn entry(e: &Value, s: &Sides) -> bool {
    crate::admission::SourceContextPolicy::Closed.accepts(e)
        && e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && e["card"]["keywords"] == json!([])
        && e["keywordValues"] == json!({})
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
pub(crate) fn compile(a: &Ability) -> Result<Program, String> {
    if !registration(a) {
        return Err("unit entry requires the closed mandatory friendly captured-card phase-stat registration".into());
    }
    let mut n = a.clone();
    n.hooks.clear();
    n.trigger = "whenPlayed".into();
    n.context_title.clear();
    if let Effect::ModifyStats { target, .. } = &mut n.effect {
        *target = Target::Name("self".into());
    }
    let mut p = Program::compile(&n)?;
    p.trigger = "onUnitEntersPlay".into();
    p.hooks = a.hooks.clone();
    p.context_title = a.context_title.clone();
    for i in &mut p.instructions {
        if let Instruction::Apply(Effect::ModifyStats { target, .. }) = i {
            *target = Target::Expression(Expr::Ref {
                path: vec!["unitEntryCard".into()],
            });
        }
    }
    p.executable = Err("captured unit-entry event binding uses shared interpreter");
    Ok(p)
}
