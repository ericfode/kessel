//! Closed source and selector envelopes for chosen physical hand discard.
//! Runtime positions and mutation are deliberately absent from this contract.
use super::{Ability, Effect, Expr, Target, TargetResolver};
use serde_json::{json, Value};
pub const KEEPER_PROMPT: &str =
    "Choose 2 cards named Keeper of Skara Nal from your hand to discard";
pub(crate) fn marker(v: &Value) -> bool {
    // View-card programs have their own closed nested discard enum. Its exact
    // viewSelected binding is not the physical own-hand operation added here.
    super::contains(v, |v| {
        v["kind"] == "attackModifiers"
            || v["kind"] == "discardSpecificCard" && v["target"] != "viewSelected"
    })
}
pub fn hand_selector(r: &TargetResolver) -> bool {
    if r.owner != "self"
        || r.card_type != "any"
        || r.zone.as_deref() != Some("hand")
        || r.arena.is_some()
        || r.optional
        || r.native.is_some()
        || r.player_selection
        || !r.prompt_title_parts.is_empty()
    {
        return false;
    }
    if r.mode.is_none() {
        return r.count.is_none() && r.predicate.is_none() && r.prompt_title.is_none();
    }
    r.mode.as_deref()==Some("exactly") && matches!(r.count.as_ref().map(Expr::original),Some(Expr::Const{value}) if value==&json!(2))
        && r.prompt_title.as_deref()==Some(KEEPER_PROMPT)
        && r.predicate.as_ref().is_some_and(|p|serde_json::to_value(p).ok()==Some(json!({"kind":"binary","op":"===","left":{"kind":"ref","path":["card","title"]},"right":{"kind":"const","value":"Keeper of Skara Nal"}})))
}
pub fn attack_modifiers(e: &Effect) -> bool {
    matches!(e,Effect::AttackModifiers{effects} if effects.len()==2
    && matches!(&effects[0],Effect::ModifyStats{target:Target::Name(n),duration,power,hp} if n=="self"&&duration=="attack"&&matches!(power.original(),Expr::Const{value} if value==&json!(15))&&matches!(hp.original(),Expr::Const{value} if value==&json!(0)))
    && matches!(&effects[1],Effect::GainKeyword{target:Target::Name(n),duration,keyword,amount} if n=="self"&&duration=="attack"&&keyword=="overwhelm"&&matches!(amount.original(),Expr::Const{value} if value==&json!(1))))
}
fn common(a: &Ability) -> bool {
    !a.attached_host
        && !a.piloting_ability
        && !a.pilot_unit_ability
        && a.hooks.is_empty()
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && a.target_resolvers.is_empty()
        && !a.attacker_must_survive
        && !a.coordinate
        && a.context_title.is_empty()
        && a.costs.is_empty()
        && !a.requires_confirmation
        && a.condition.is_none()
        && a.then.is_none()
        && a.if_you_do_condition.is_none()
        && !a.title.is_empty()
        && a.title.len() <= 1024
}
pub fn registration(a: &Ability) -> bool {
    if !common(a)
        || !a.optional
        || !matches!(a.effect,Effect::DiscardSpecificCard{target:Target::Name(ref n)} if n=="chosen")
    {
        return false;
    }
    let Some(r) = &a.target_resolver else {
        return false;
    };
    if !hand_selector(r) {
        return false;
    }
    let Some(next) = a.if_you_do.as_deref() else {
        return false;
    };
    if !common(next) || next.optional || !next.trigger.is_empty() || next.if_you_do.is_some() {
        return false;
    }
    if a.trigger == "whenPlayed" && r.mode.is_none() {
        return next.target_resolver.as_ref().is_some_and(|r| {
            r.owner == "any"
                && r.card_type == "unit"
                && r.zone.as_deref() == Some("anyAttackable")
                && r.arena.is_none()
                && r.predicate.is_none()
                && !r.optional
                && r.mode.is_none()
                && r.count.is_none()
                && r.native.is_none()
                && !r.player_selection
                && r.prompt_title.is_none()
                && r.prompt_title_parts.is_empty()
        }) && matches!(&next.effect,Effect::Damage{target:Target::Name(n),amount} if n=="chosen"&&matches!(amount.original(),Expr::Const{value} if value==&json!(3)));
    }
    a.trigger == "onAttack"
        && r.mode.as_deref() == Some("exactly")
        && next.target_resolver.is_none()
        && attack_modifiers(&next.effect)
}
pub fn entry(e: &Value) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || crate::admission::OrdinarySource::from_entry(e)
            != Some(crate::admission::OrdinarySource::Unit)
        || !e["attachCondition"].is_null()
        || !e["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox", "deployCost"]
            .iter()
            .any(|k| !e["card"][*k].is_null())
    {
        return false;
    }
    {
        let profile:Value=serde_json::from_str(r###"{"title":"Qi'ra","subtitle":"Master of Teräs Käsi","cost":7,"hp":7,"power":9,"text":"This unit gets –1/–0 for each card in your hand.\nWhen Played: You may discard a card from your hand. If you do, deal 3 damage to a unit.","deployBox":null,"epicAction":"","unique":true,"rules":null,"upgradePower":null,"upgradeHp":null,"aspects":["cunning"],"traits":["underworld"],"arena":"ground","keywords":[],"types":["unit"]}"###).unwrap();
        let abilities:Value=serde_json::from_str(r###"{"leader":[],"unit":[],"card":[{"trigger":"constant","title":"This unit gets -1/-0 for each card in your hand.","costs":[],"effect":{"kind":"modifyStats","target":"self","duration":"persistent","power":{"kind":"binary","op":"*","left":{"kind":"unary","op":"-","value":{"kind":"const","value":1}},"right":{"kind":"ref","path":["player","hand","length"]}},"hp":{"kind":"const","value":0}}},{"trigger":"whenPlayed","title":"Discard a card from your hand to deal 3 damage to a unit","costs":[],"optional":true,"targetResolver":{"owner":"self","type":"any","zone":"hand"},"effect":{"kind":"discardSpecificCard","target":"chosen"},"ifYouDo":{"title":"Deal 3 damage to a unit","costs":[],"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":3}}}}]}"###).unwrap();
        if profile
            .as_object()
            .unwrap()
            .iter()
            .all(|(k, v)| e["card"][k] == *v)
            && e["abilities"] == abilities
            && e["keywordValues"] == json!({})
        {
            return true;
        }
    }
    {
        let profile:Value=serde_json::from_str(r###"{"title":"Keeper of Skara Nal","subtitle":"Awoken","cost":6,"hp":8,"power":5,"text":"Restore 2\nOn Attack: You may discard 2 cards named Keeper of Skara Nal from your hand. If you do, this unit gets +15/+0 and gains Overwhelm for this attack.","deployBox":null,"epicAction":"","unique":true,"rules":"Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n","upgradePower":null,"upgradeHp":null,"aspects":["vigilance","aggression"],"traits":["droid","vehicle","walker"],"arena":"ground","keywords":["restore","overwhelm"],"types":["unit"]}"###).unwrap();
        let abilities:Value=serde_json::from_str(r###"{"leader":[],"unit":[],"card":[{"trigger":"onAttack","title":"Discard 2 cards named Keeper of Skara Nal from your hand. If you do, this unit gets +15/+0 and gains Overwhelm for this attack.","costs":[],"optional":true,"targetResolver":{"owner":"self","type":"any","zone":"hand","predicate":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","title"]},"right":{"kind":"const","value":"Keeper of Skara Nal"}},"mode":"exactly","count":{"kind":"const","value":2},"promptTitle":"Choose 2 cards named Keeper of Skara Nal from your hand to discard"},"effect":{"kind":"discardSpecificCard","target":"chosen"},"ifYouDo":{"title":"This unit gets +15/+0 and gains Overwhelm for this attack","costs":[],"effect":{"kind":"attackModifiers","effects":[{"kind":"modifyStats","target":"self","duration":"attack","power":{"kind":"const","value":15},"hp":{"kind":"const","value":0}},{"kind":"gainKeyword","target":"self","duration":"attack","keyword":"overwhelm","amount":{"kind":"const","value":1}}]}}}]}"###).unwrap();
        if profile
            .as_object()
            .unwrap()
            .iter()
            .all(|(k, v)| e["card"][k] == *v)
            && e["abilities"] == abilities
            && e["keywordValues"] == json!({"restore":2})
        {
            return true;
        }
    }
    false
}
