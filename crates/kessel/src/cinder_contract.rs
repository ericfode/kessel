//! Closed source contract for scalar own-base damage followed by a uniform arena
//! window. This module describes source/IR only; runtime state stays with the
//! existing scalar and prepared-window owners.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program, Target};
use serde_json::{json, Value};

pub(crate) fn registration() -> Value {
    json!({"trigger":"event","title":"Deal 5 damage to your base","costs":[],
        "effect":{"kind":"damage","target":"ownBase","amount":{"kind":"const","value":5}},
        "then":{"title":"Deal 5 damage to each unit","costs":[],"effect":{"kind":"damage",
            "target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true}},
            "amount":{"kind":"const","value":5}}}})
}
fn without_titles(mut v: Value) -> Option<Value> {
    for node in [&[][..], &["then"][..]] {
        let mut at = &mut v;
        for key in node {
            at = at.get_mut(*key)?;
        }
        let object = at.as_object_mut()?;
        let title = object.remove("title")?;
        if !title
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 1024)
        {
            return None;
        }
    }
    Some(v)
}
pub(crate) fn ability(a: &Ability) -> bool {
    if a.attached_host
        || a.piloting_ability
        || a.pilot_unit_ability
        || a.then.as_deref().is_some_and(|next| {
            next.attached_host || next.piloting_ability || next.pilot_unit_ability
        })
    {
        return false;
    }
    let expected: Ability = serde_json::from_value(registration()).expect("closed damage contract");
    without_titles(serde_json::to_value(a).expect("ability serializes"))
        == without_titles(serde_json::to_value(expected).expect("ability serializes"))
}
/// Printed semantics identify this source family even when a malformed raw
/// registration removes its arena leaf and could otherwise enter scalar gates.
pub(crate) fn marker(e: &Value) -> bool {
    e["card"]["text"] == "Deal 5 damage to your base. Then, deal 5 damage to each unit."
}
pub(crate) fn entry(e: &Value) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || crate::admission::OrdinarySource::from_entry(e)
            != Some(crate::admission::OrdinarySource::Event)
        || e["keywordValues"] != json!({})
        || !e["piloting"].is_null()
        || !e["attachCondition"].is_null()
        || e["abilities"]["leader"] != json!([])
        || e["abilities"]["unit"] != json!([])
        || e["abilities"].as_object().is_none_or(|o| o.len() != 3)
    {
        return false;
    }
    let profile = json!({"cost":6,"hp":null,"power":null,"unique":false,"keywords":[],
        "traits":["disaster","plan"],"aspects":["aggression","villainy"],
        "text":"Deal 5 damage to your base. Then, deal 5 damage to each unit.",
        "rules":"All damage dealt to units by Operation Cinder is dealt simultaneously.\n",
        "deployBox":null,"epicAction":"","upgradePower":null,"upgradeHp":null});
    if !profile
        .as_object()
        .unwrap()
        .iter()
        .all(|(k, v)| e["card"][k] == *v)
        || ![
            "arena",
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployCost",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
    {
        return false;
    }
    let Some(cards) = e["abilities"]["card"].as_array().filter(|a| a.len() == 1) else {
        return false;
    };
    without_titles(cards[0].clone()) == without_titles(registration())
}
/// Reachability test over immutable instructions, also used before payment.
/// The group captures the physical global list only after the scalar base window.
pub(crate) fn requires_global_order(p: &Program) -> bool {
    let [Instruction::Apply(Effect::Damage {
        target: Target::Name(n),
        amount,
    }), Instruction::Group(Effect::Damage {
        target: Target::ArenaList(list),
        amount: next,
    }), Instruction::EndStep] = p.instructions.as_slice()
    else {
        return false;
    };
    n == "ownBase"
        && matches!(amount.original(),Expr::Const{value} if value.as_u64()==Some(5))
        && matches!(next.original(),Expr::Const{value} if value.as_u64()==Some(5))
        && list.controller_order.is_none()
        && matches!(list.receiver.original(),Expr::Ref{path} if path==&["game"])
        && matches!(list.predicate.original(),Expr::Const{value} if value.as_bool()==Some(true))
}
