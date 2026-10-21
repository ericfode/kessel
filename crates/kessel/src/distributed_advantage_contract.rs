//! Closed source contract for captured-defeat Advantage allocation. Runtime has
//! no card identity dispatch; raw profile equality prevents widened wrappers.
use serde_json::Value;
const SOURCE: &str = r###"{"abilities":{"card":[{"contextTitle":[{"kind":"const","value":"Distribute "},{"kind":"ref","path":["event","lastKnownInformation","power"]},{"kind":"const","value":" Advantage tokens among friendly units"}],"costs":[],"effect":{"amount":{"kind":"ref","path":["event","lastKnownInformation","power"]},"canChooseNoTargets":true,"canDistributeLess":false,"chooser":"self","creationEvents":"perRecipient","kind":"distributeTokenUpgrade","targetResolver":{"owner":"self","type":"unit","zone":"anyAttackable"},"tokenType":"advantage"},"title":"Distribute a number of Advantage tokens equal to this unit's power among friendly units","trigger":"whenDefeated"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"ground","aspects":["cunning","villainy"],"cost":5,"deployBox":null,"epicAction":"","hp":4,"id":"9419144933","internalName":"helgait#dooku-was-a-visionary","keywords":[],"power":6,"rules":"Abilities that refer to a card’s power include temporary modifiers.\n","setCodes":[{"number":195,"set":"ASH"}],"setId":{"number":195,"set":"ASH"},"subtitle":"Dooku Was a Visionary","text":"When Defeated: You may distribute a number of Advantage tokens equal to this unit's power among friendly units (divided as you choose).","title":"Helgait","traits":["separatist"],"types":["unit"],"unique":true,"upgradeHp":null,"upgradePower":null},"className":"HelgaitDookuWasAVisionary","dataHash":"f5988763a096e10465c99788c44907c71790d10b078116ebb67d8fbbf6dfa13d","diagnostics":[],"id":"9419144933","internalName":"helgait#dooku-was-a-visionary","keywordValues":{},"requiredCapabilities":["cardType:unit","ir:const","ir:distributeTokenUpgrade","ir:ref","ref:event.lastKnownInformation.power","targetType:unit","targetZone:anyAttackable","trigger:whenDefeated"],"setCode":"ASH_195","sourceHash":"19281e10cd7a6648ed5b8162cce726777a3dc1cc4f6bd5e9f776832bee06e39f","sourcePath":"game/cards/08_ASH/units/HelgaitDookuWasAVisionary.ts","status":"supported"}"###;
fn original() -> Value {
    serde_json::from_str(SOURCE).expect("reviewed source profile")
}
pub(super) fn marker(e: &Value) -> bool {
    e["sourceHash"] == "19281e10cd7a6648ed5b8162cce726777a3dc1cc4f6bd5e9f776832bee06e39f"
        || super::contains(e, |n| n["kind"] == "distributeTokenUpgrade")
}
pub(super) fn entry(e: &Value) -> bool {
    e == &original()
}
pub(super) fn registration(a: &super::Ability) -> bool {
    serde_json::to_value(a).ok().is_some_and(|v| {
        // Typed serde emits absent default fields. Compare exact semantic payload
        // separately; complete raw source equality above rejects their presence.
        a.trigger == "whenDefeated"
            && !a.optional
            && !a.coordinate
            && !a.requires_confirmation
            && a.costs.is_empty()
            && a.condition.is_none()
            && a.then.is_none()
            && a.if_you_do.is_none()
            && a.target_resolver.is_none()
            && a.target_resolvers.is_empty()
            && a.hooks.is_empty()
            && a.gained_host.is_none()
            && a.attached_context.is_none()
            && !a.attached_host
            && !a.piloting_ability
            && !a.pilot_unit_ability
            && !a.attacker_must_survive
            && a.if_you_do_condition.is_none()
            && v["title"] == original()["abilities"]["card"][0]["title"]
            && v["contextTitle"] == original()["abilities"]["card"][0]["contextTitle"]
            && typed_effect(&a.effect)
    })
}
pub(super) fn typed_effect(e: &super::Effect) -> bool {
    serde_json::to_value(e).ok().is_some_and(|v| {
        let expected=original()["abilities"]["card"][0]["effect"].clone();
        let super::Effect::DistributeTokenUpgrade {token_type,amount,can_choose_no_targets,can_distribute_less,chooser,creation_events,target_resolver:r}=e else{return false;};
        token_type=="advantage" && *can_choose_no_targets && !can_distribute_less && chooser=="self" && creation_events=="perRecipient"
        && matches!(amount.original(),super::Expr::Ref{path} if path==&["event","lastKnownInformation","power"])
        && r.owner=="self" && r.card_type=="unit" && r.zone.as_deref()==Some("anyAttackable")
        && r.arena.is_none() && r.predicate.is_none() && !r.optional && r.native.is_none() && !r.player_selection
        && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none() && r.prompt_title_parts.is_empty()
        && v["amount"]==expected["amount"]
    })
}
