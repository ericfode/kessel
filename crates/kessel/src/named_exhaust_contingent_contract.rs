//! Closed source projection and proposed lowering for two named exhaust bindings.
//! Source/typed/program closure; imports no runtime state. Named identities and event
//! outcomes are separate inputs to the continuation, never a changed-count heuristic.
use crate::ability_ir::{Ability, Effect, Instruction, Program};
use serde_json::Value;
const REASON: &str = "named exhaust contingency requires the complete closed event source profile";
const SOURCE_HASH: &str = "b96ffde5f3233eebfec1e36701fd04029982816e2476864943b639ddc0639cb4";
fn profile() -> Value {
    serde_json::from_str(r###"{"setCode":"ASH_231","sourcePath":"game/cards/08_ASH/events/DiplomaticPageantry.ts","sourceHash":"b96ffde5f3233eebfec1e36701fd04029982816e2476864943b639ddc0639cb4","abilities":{"leader":[],"unit":[],"card":[{"trigger":"event","title":"Exhaust a friendly unit and an enemy unit. If you do, give 2 Advantage tokens to that friendly unit.","costs":[],"targetResolvers":[{"name":"friendlyUnit","targetResolver":{"owner":"self","type":"unit","zone":"anyAttackable","promptTitle":"Choose a friendly unit to exhaust"},"chooser":"self","effect":{"kind":"exhaust","target":"chosen"}},{"name":"enemyUnit","targetResolver":{"owner":"opponent","type":"unit","zone":"anyAttackable","promptTitle":"Choose an enemy unit to exhaust"},"chooser":"self","effect":{"kind":"exhaust","target":"chosen"}}],"effect":{"kind":"noAction"},"ifYouDo":{"title":"Give 2 Advantage tokens to that friendly unit","costs":[],"effect":{"kind":"giveAdvantage","target":{"kind":"ref","path":["namedTargets","friendlyUnit"]},"amount":{"kind":"const","value":2},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"},"ifYouDoCondition":{"kind":"binary","op":"&&","left":{"kind":"ref","path":["namedTargets","friendlyUnit"]},"right":{"kind":"ref","path":["namedTargets","enemyUnit"]}}}}]},"className":"DiplomaticPageantry","baseClass":"EventCard","id":"6089557572","internalName":"diplomatic-pageantry","status":"supported","card":{"title":"Diplomatic Pageantry","subtitle":"","cost":1,"hp":null,"power":null,"text":"Exhaust a friendly unit and an enemy unit. If you do, give 2 Advantage tokens to that friendly unit.","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"6089557572","aspects":["cunning"],"traits":["law"],"keywords":[],"types":["event"],"setId":{"set":"ASH","number":231},"setCodes":[{"set":"ASH","number":231}],"internalName":"diplomatic-pageantry"},"dataHash":"24f9dd2b7bc066e3f4de88b4767417fff5b04bf2492ab29266eeb3df670e320e","keywordValues":{},"requiredCapabilities":["ability:ifYouDo","ability:ifYouDoCondition","binary:&&","cardType:event","ir:binary","ir:const","ir:exhaust","ir:giveAdvantage","ir:noAction","ir:ref","ref:namedTargets.enemyUnit","ref:namedTargets.friendlyUnit","targetType:unit","targetZone:anyAttackable","trigger:event"],"diagnostics":[]}"###).expect("reviewed source profile")
}

/// Inspect before any normalization or family early return. A marker nested in an
/// unfamiliar envelope is an error, not permission to strip that envelope.
pub(crate) fn prepare(raw: &Value) -> Result<bool, String> {
    let mut todo = vec![(raw, 0usize)];
    let mut visited = 0usize;
    let mut marked = false;
    while let Some((v, depth)) = todo.pop() {
        visited += 1;
        if visited > 65_536 || depth > 128 {
            return Err("named exhaust source inspection budget exceeded".into());
        }
        marked |= v["sourceHash"] == SOURCE_HASH
            || v["className"] == "DiplomaticPageantry"
            || v["internalName"] == "diplomatic-pageantry"
            || v["path"].as_array().is_some_and(|p| {
                p.len() == 2
                    && p[0] == "namedTargets"
                    && (p[1] == "friendlyUnit" || p[1] == "enemyUnit")
            });
        match v {
            Value::Array(a) => todo.extend(a.iter().map(|v| (v, depth + 1))),
            Value::Object(o) => todo.extend(o.values().map(|v| (v, depth + 1))),
            _ => {}
        }
        if todo.len().saturating_add(visited) > 65_536 {
            return Err("named exhaust source inspection budget exceeded".into());
        }
    }
    if !marked {
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

/// Typed checks supplement raw proof: serde deliberately omits attached/pilot flags.
pub(crate) fn registration(a: &Ability) -> bool {
    if a.attached_host || a.piloting_ability || a.pilot_unit_ability {
        return false;
    }
    let expected: Ability = serde_json::from_value(profile()["abilities"]["card"][0].clone())
        .expect("reviewed typed registration");
    serde_json::to_value(a).ok() == serde_json::to_value(expected).ok()
}

/// Only a proposed instruction payload. Both selectors retain their source order
/// and prompt titles. Root must integrate a closed instruction, not globally allow
/// these names/ref expressions in scalar evaluation or named-stat admission.
pub(crate) fn exhaust_group(a: &Ability) -> Option<Effect> {
    registration(a).then(|| Effect::Simultaneous {
        effects: a
            .target_resolvers
            .iter()
            .map(|d| Effect::Select {
                target_resolver: d.target_resolver.clone(),
                effect: Box::new(d.effect.clone()),
            })
            .collect(),
    })
}

/// Physical selected identities, independent of whether each exhaust changed state.
/// A zero slot represents an unbound/null resolver, not a synthetic card selection.
/// This transient value proposes reuse of existing saved Selection slots in runtime;
/// it makes no State-size or integrated lifecycle claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bindings {
    friendly: u16,
    enemy: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentOutcome {
    NoEvents,
    LastEvent {
        resolved_or_replacement_resolved: bool,
    },
}
/// Original emission order, including a generated event that later failed its
/// condition. GameEvent.canResolve initially checks CREATED status, not condition.
/// `None` is an absent event; `Some(false)` must never fall back to an earlier success.
pub(crate) fn parent_outcome(events: [Option<bool>; 2]) -> ParentOutcome {
    match events[1].or(events[0]) {
        Some(resolved_or_replacement_resolved) => ParentOutcome::LastEvent {
            resolved_or_replacement_resolved,
        },
        None => ParentOutcome::NoEvents,
    }
}

impl Bindings {
    pub(crate) fn new(friendly: u16, enemy: u16) -> Result<Self, String> {
        if friendly >= 256 || enemy >= 256 || friendly != 0 && friendly == enemy {
            return Err("named exhaust requires distinct checked physical unit bindings".into());
        }
        Ok(Self { friendly, enemy })
    }
    pub(crate) fn recipient(self, parent: ParentOutcome) -> Option<u16> {
        if self.friendly != 0
            && self.enemy != 0
            && matches!(
                parent,
                ParentOutcome::LastEvent {
                    resolved_or_replacement_resolved: true
                }
            )
        {
            Some(self.friendly)
        } else {
            None
        }
    }
}

pub(crate) fn typed_marker(a: &Ability) -> bool {
    serde_json::to_value(a).ok().is_some_and(|v| {
        crate::ability_ir::contains(&v, |v| {
            v["path"].as_array().is_some_and(|p| {
                p.len() == 2
                    && p[0] == "namedTargets"
                    && (p[1] == "friendlyUnit" || p[1] == "enemyUnit")
            })
        })
    })
}
pub(crate) fn dependencies(catalogue: &Value) -> Result<(), String> {
    let expected:Value=serde_json::from_str(r###"{"setCode":"TOKEN_ADVANTAGE","sourcePath":"game/cards/08_ASH/tokens/Advantage.ts","sourceHash":"1280861b9a94572ecce2c1d62fb5fd93c11b08609495c3d9f60e656d153e3119","abilities":{"leader":[],"unit":[],"card":[]},"status":"unsupported","id":"5844562972","internalName":"advantage","card":{"title":"Advantage","subtitle":"","cost":0,"hp":0,"power":1,"text":"When attached unit's attack or defense ends: Defeat this upgrade.","deployBox":null,"epicAction":"","unique":false,"rules":"When Attack Ends/When Defense Ends abilities trigger at the same time as units deal combat damage.\n","upgradePower":1,"upgradeHp":0,"id":"5844562972","aspects":[],"traits":["innate"],"keywords":[],"types":["token","upgrade"],"setId":{"set":"ASH"},"internalName":"advantage"},"dataHash":"9a81c2d083caa13d2a5a173bdeedf76e837d62fb413496e8539338b42d71ce94","keywordValues":{},"requiredCapabilities":["cardType:token","cardType:upgrade"],"diagnostics":[{"code":"UNSUPPORTED_SOURCE","pattern":"expected-one-card-class","file":"game/cards/08_ASH/tokens/Advantage.ts","line":1,"column":1,"message":"Found 2 classes"}]}"###).expect("reviewed complete native Advantage source");
    let found = catalogue["cards"]
        .as_array()
        .ok_or("named exhaust requires catalogue cards")?
        .iter()
        .filter(|v| v["setCode"] == "TOKEN_ADVANTAGE")
        .collect::<Vec<_>>();
    if found.len() != 1 || *found[0] != expected || !crate::advantage::reviewed_entry(found[0]) {
        return Err(
            "named exhaust requires the complete reviewed native Advantage definition".into(),
        );
    }
    Ok(())
}
/// Native State stores the catalogue index plus one; reserve that sentinel space.
pub(crate) fn checked_program_id(start: usize) -> Result<u16, String> {
    let id = u16::try_from(start).map_err(|_| "named exhaust program capacity exceeded")?;
    id.checked_add(1).ok_or("named exhaust program reservation exceeds stored program+1 capacity")?;
    Ok(id)
}

pub(crate) fn compiled() -> Result<Program, String> {
    let a: Ability = serde_json::from_value(profile()["abilities"]["card"][0].clone())
        .map_err(|e| e.to_string())?;
    let effect = exhaust_group(&a).ok_or(REASON)?;
    let mut skeleton = a.clone();
    skeleton.target_resolvers.clear();
    skeleton.if_you_do = None;
    let mut p = Program::compile(&skeleton)?;
    let advantage:Effect=serde_json::from_value(serde_json::json!({"kind":"giveAdvantage","target":"chosen","amount":{"kind":"const","value":2},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"})).map_err(|e|e.to_string())?;
    p.instructions = vec![
        Instruction::NamedExhaustContingent(effect),
        Instruction::IfResult { skip: 3 },
        Instruction::Apply(advantage),
    ];
    p.event_end = 3;
    p.select_ends.clear();
    p.executable = Err("named exhaust captured bindings use shared interpreter");
    Ok(p)
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions
        .iter()
        .any(|i| matches!(i, Instruction::NamedExhaustContingent(_)))
}
pub(crate) fn program(p: &Program) -> bool {
    p.select_ends.is_empty()
        && compiled().ok().is_some_and(|expected| {
            serde_json::to_value(p).ok() == serde_json::to_value(expected).ok()
        })
}

#[cfg(test)]
#[path = "named_exhaust_contingent_tests.rs"]
mod tests;
