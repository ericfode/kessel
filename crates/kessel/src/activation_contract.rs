//! Reviewed base-zone constant sources. Runtime side selection and physical
//! activation use their existing state directly; admission needs only source kind.
use super::{Ability, Effect, Expr, RecipeSelector, Sides, Target, TargetResolver};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source { Base, Leader }

// Keep this family's original header policy. Program admission owns later
// confirmation/attachment diagnostics, so this check must not absorb those guards.
fn constant<'a>(a: &'a Ability, recipient: &str) -> Option<&'a TargetResolver> {
    if a.trigger != "constant" || a.optional || !a.costs.is_empty() || a.condition.is_some()
        || a.coordinate || a.then.is_some() || a.if_you_do.is_some()
        || !a.context_title.is_empty() || !a.target_resolvers.is_empty() { return None; }
    a.target_resolver.as_ref().filter(|r|RecipeSelector::one("self", recipient, "arena").matches(r))
}
fn reference(e: &Expr, expected: &[&str]) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(expected.iter().copied()))
}
fn number(e: &Expr, expected: u64) -> bool { matches!(e, Expr::Const { value } if value.as_u64() == Some(expected)) }
fn cost_three(e: &Expr) -> bool {
    matches!(e, Expr::Binary { op, left, right } if op == "&&"
        && matches!(left.as_ref(), Expr::Query { name, receiver, args } if name == "isUnit" && reference(receiver, &["card"]) && args.is_empty())
        && matches!(right.as_ref(), Expr::Binary { op, left, right } if op == ">=" && reference(left, &["card", "cost"]) && number(right, 3)))
}
fn raid(a: &Ability, exclude_source: bool) -> bool {
    let Some(resolver) = constant(a, "unit") else { return false; };
    matches!(&a.effect, Effect::GainKeyword { target: Target::Name(name), keyword, amount, duration }
        if name == "chosen" && keyword == "raid" && number(amount, 1) && duration == "persistent")
        && resolver.predicate.as_ref().is_some_and(|p| {
            if exclude_source {
                matches!(p, Expr::Binary { op, left, right } if op == "&&" && cost_three(left)
                    && matches!(right.as_ref(), Expr::Binary { op, left, right } if op == "!=="
                        && reference(left, &["card"]) && reference(right, &["source"])))
            } else { cost_three(p) }
        })
}

pub fn base_zone_source(entry: &Value) -> Option<Source> {
    if crate::bounded_reductions::leader_entry(entry) { return Some(Source::Leader); }
    let sides: Sides = serde_json::from_value(entry["abilities"].clone()).ok()?;
    if !entry["card"]["unique"].is_boolean() || !entry["attachCondition"].is_null() || !entry["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox"].iter().any(|key| !entry["card"][key].is_null()) { return None; }
    if entry["baseClass"] == "BaseCard" && entry["card"]["types"] == json!(["base"])
        && entry["card"]["unique"] == false && entry["card"]["hp"] == 26 && entry["card"]["cost"].is_null()
        && entry["card"]["power"].is_null() && entry["keywordValues"] == json!({}) && entry["card"]["keywords"] == json!([])
        && sides.card.len() == 1 && sides.leader.is_empty() && sides.unit.is_empty() {
        let a = &sides.card[0];
        let hp_bonus = matches!(&a.effect, Effect::ModifyStats { power, hp, .. } if number(power, 0) && number(hp, 1));
        let text = if hp_bonus { "Each leader unit you control gets +0/+1." } else { "Each leader unit you control gets +1/+0." };
        let grant = matches!(&a.effect, Effect::ModifyStats { target: Target::Name(name), power, hp, duration }
            if name == "chosen" && duration == "persistent"
                && ((number(power, 0) && number(hp, 1)) || (number(power, 1) && number(hp, 0))));
        return (entry["card"]["text"] == text && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
            && ["rules", "deployBox", "upgradePower", "upgradeHp"].iter().all(|key| entry["card"][key].is_null())
            && grant && constant(a, "leaderUnit").is_some_and(|r|r.predicate.is_none())).then_some(Source::Base);
    }
    (entry["card"]["text"] == "Friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["deployBox"] == "Raid 1\nOther friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["epicAction"] == "Epic Action: If you control 6 or more resources, deploy this leader."
        && entry["card"]["rules"] == "Abilities that refer to a card’s cost always refer to its printed cost, regardless of modifiers.\n"
        && entry["card"]["upgradePower"].is_null() && entry["card"]["upgradeHp"].is_null()
        && entry["baseClass"] == "LeaderUnitCard" && entry["card"]["types"] == json!(["leader"])
        && entry["card"]["unique"] == true && entry["card"]["cost"] == 6 && entry["card"]["hp"] == 5
        && entry["card"]["power"] == 5 && entry["card"]["arena"] == "ground"
        && entry["keywordValues"] == json!({"raid":1}) && entry["card"]["keywords"] == json!(["raid"])
        && sides.card.is_empty() && sides.leader.len() == 1 && sides.unit.len() == 1
        && raid(&sides.leader[0], false) && raid(&sides.unit[0], true)).then_some(Source::Leader)
}


#[cfg(test)]
#[path = "activation_contract_tests.rs"]
mod tests;
