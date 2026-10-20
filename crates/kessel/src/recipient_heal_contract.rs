//! Exact recipient callback contract; no general callback expression evaluator.
use crate::ability_ir::{Ability, Effect};
use serde_json::{json, Value};
pub(crate) fn target() -> Value {
    json!({"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"const","value":true}})
}
pub(crate) fn amount() -> Value {
    json!({"kind":"healRecipientAmount","callback":{"kind":"attackCallback","parameters":[{"root":"attackBinding1_0","role":"healedCard"}],"body":{"kind":"ref","path":["attackBinding1_0","damage"]}},"evaluateAt":"addPropertiesToEachHealEvent","legalityComparison":"nativePropertyFunctionComparedToZero"})
}
pub(crate) fn marker(v: &Value) -> bool {
    crate::ability_ir::contains(v, |v| {
        v["kind"] == "healRecipientAmount" || v["kind"] == "recipientHeal"
    })
}
pub(crate) fn effect(e: &Effect) -> Result<(), String> {
    if matches!(e,Effect::RecipientHeal{target:t,amount:a} if *t==target()&&*a==amount()) {
        Ok(())
    } else {
        Err("recipient heal requires exact friendly arena membership and captured recipient damage callback".into())
    }
}
pub(crate) fn ability(a: &Ability) -> bool {
    a.trigger == "whenPlayed"
        && crate::admission::plain_registration(a)
        && a.target_resolver.is_none()
        && a.attached_context.is_none()
        && a.hooks.is_empty()
        && !a.attacker_must_survive
        && !a.attached_host
        && !a.piloting_ability
        && !a.pilot_unit_ability
        && a.if_you_do_condition.is_none()
        && effect(&a.effect).is_ok()
}
pub(crate) fn entry(e: &Value) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || crate::admission::OrdinarySource::from_entry(e)
            != Some(crate::admission::OrdinarySource::Unit)
        || e["keywordValues"] != json!({"sentinel":1})
        || !e["piloting"].is_null()
        || !e["attachCondition"].is_null()
    {
        return false;
    }
    let profile = json!({"cost":8,"hp":10,"power":7,"arena":"space","unique":true,"keywords":["sentinel"],"traits":["rebel","vehicle","capital ship"],"aspects":["vigilance","heroism"],"text":"Sentinel (Enemy units in this arena must attack a Sentinel when they attack you.)\nWhen Played: Heal all damage from each friendly unit.","rules":null,"deployBox":null,"epicAction":"","upgradePower":null,"upgradeHp":null});
    if !profile
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
        || !["pilotText", "pilotBox", "pilotingBox", "deployCost"]
            .iter()
            .all(|k| e["card"][*k].is_null())
    {
        return false;
    }
    let mut sides = e["abilities"].clone();
    let Some(a) = sides["card"].as_array_mut().filter(|a| a.len() == 1) else {
        return false;
    };
    if !a[0]["title"]
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 1024)
    {
        return false;
    }
    a[0].as_object_mut().unwrap().remove("title");
    sides
        == json!({"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","costs":[],"effect":{"kind":"heal","target":target(),"amount":amount()}}]})
}
pub(crate) fn normalize(e: &Value, abilities: &mut Value) -> Result<(), String> {
    if marker(&e["abilities"]) {
        if !entry(e) || *abilities != e["abilities"] {
            return Err("recipient heal requires complete closed ordinary unit profile and original sole mandatory when-played registration".into());
        }
        abilities["card"][0]["effect"]["kind"] = json!("recipientHeal");
    }
    Ok(())
}
