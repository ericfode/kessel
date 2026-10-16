//! Base-zone constant admission and active-side selection share this topology.
//! Printed keyword inventory stays separate from the grants on the active side.
use super::{Ability, CardPrograms, Effect, Expr, RecipeSelector, Sides, Target};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Card, Leader, Unit }
impl Side {
    pub fn select<'a, T>(self, card: &'a [T], leader: &'a [T], unit: &'a [T]) -> &'a [T] {
        match self { Self::Card => card, Self::Leader => leader, Self::Unit => unit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology { CardOnly, LeaderSides }
impl Topology {
    pub fn for_source(leader: bool) -> Self { if leader { Self::LeaderSides } else { Self::CardOnly } }
    fn accepts_single_registration_per_side(self, sides: &Sides) -> bool {
        let counts = (sides.card.len(), sides.leader.len(), sides.unit.len());
        counts == match self { Self::CardOnly => (1, 0, 0), Self::LeaderSides => (0, 1, 1) }
    }
    pub fn activate(self, deployed: bool) -> Activation { Activation { topology: self, deployed } }
}

/// Side selection depends on deployment; base-zone placement additionally uses
/// the actual player slots. Keeping those inputs distinct preserves legacy
/// placement behavior without inferring a leader role from partial metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activation { topology: Topology, deployed: bool }
impl Activation {
    pub fn side(self) -> Side {
        match (self.topology, self.deployed) {
            (Topology::CardOnly, _) => Side::Card,
            (Topology::LeaderSides, false) => Side::Leader,
            (Topology::LeaderSides, true) => Side::Unit,
        }
    }
    pub fn programs(self, programs: &CardPrograms) -> &[u16] {
        self.side().select(&programs.card, &programs.leader, &programs.unit)
    }
    pub fn in_base_zone(self, base_slot: bool, leader_slot: bool) -> bool {
        base_slot || (leader_slot && !self.deployed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatAxis { Power, Hp }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseZoneContract { LeaderStats(StatAxis), RaidLeader, MaximumLeader }
impl BaseZoneContract {
    pub fn topology(self) -> Topology {
        match self { Self::LeaderStats(_) => Topology::CardOnly, Self::RaidLeader | Self::MaximumLeader => Topology::LeaderSides }
    }
    fn policy(self, side: Side) -> Option<ConstantPolicy> {
        match (self, side) {
            (Self::LeaderStats(axis), Side::Card) => Some(ConstantPolicy { recipient: "leaderUnit", predicate: RecipientPredicate::All, grant: Grant::Stats(axis) }),
            (Self::RaidLeader, Side::Leader) => Some(ConstantPolicy { recipient: "unit", predicate: RecipientPredicate::PrintedCost { exclude_source: false }, grant: Grant::Raid }),
            (Self::RaidLeader, Side::Unit) => Some(ConstantPolicy { recipient: "unit", predicate: RecipientPredicate::PrintedCost { exclude_source: true }, grant: Grant::Raid }),
            _ => None,
        }
    }
    fn accepts(self, sides: &Sides) -> bool {
        self.topology().accepts_single_registration_per_side(sides) && [Side::Card, Side::Leader, Side::Unit].into_iter().all(|side| {
            side.select(&sides.card, &sides.leader, &sides.unit).iter().all(|ability| {
                self.policy(side).is_some_and(|policy| policy.accepts(ability))
            })
        })
    }
}
#[derive(Clone, Copy)]
enum RecipientPredicate { All, PrintedCost { exclude_source: bool } }
#[derive(Clone, Copy)]
enum Grant { Stats(StatAxis), Raid }
struct ConstantPolicy { recipient: &'static str, predicate: RecipientPredicate, grant: Grant }
impl ConstantPolicy {
    fn accepts(self, a: &Ability) -> bool {
        // Preserve this family's original header policy. Other loader layers
        // retain confirmation, attachment and source-context diagnostics.
        if a.trigger != "constant" || a.optional || !a.costs.is_empty() || a.condition.is_some()
            || a.coordinate || a.then.is_some() || a.if_you_do.is_some()
            || !a.context_title.is_empty() || !a.target_resolvers.is_empty() { return false; }
        let Some(resolver) = &a.target_resolver else { return false; };
        if !RecipeSelector::one("self", self.recipient, "arena").matches(resolver) { return false; }
        let predicate = match self.predicate {
            RecipientPredicate::All => resolver.predicate.is_none(),
            RecipientPredicate::PrintedCost { exclude_source } => resolver.predicate.as_ref().is_some_and(|p| {
                if exclude_source {
                    matches!(p, Expr::Binary { op, left, right } if op == "&&" && cost_three(left)
                        && matches!(right.as_ref(), Expr::Binary { op, left, right } if op == "!=="
                            && reference(left, &["card"]) && reference(right, &["source"])))
                } else { cost_three(p) }
            }),
        };
        predicate && match (self.grant, &a.effect) {
            (Grant::Stats(axis), Effect::ModifyStats { target: Target::Name(name), power, hp, duration }) =>
                name == "chosen" && duration == "persistent" && match axis {
                    StatAxis::Power => number(power, 1) && number(hp, 0),
                    StatAxis::Hp => number(power, 0) && number(hp, 1),
                },
            (Grant::Raid, Effect::GainKeyword { target: Target::Name(name), keyword, amount, duration }) =>
                name == "chosen" && keyword == "raid" && number(amount, 1) && duration == "persistent",
            _ => false,
        }
    }
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

/// Returns the activation contract only after complete family metadata and each
/// side's target/grant policy have matched. This does not admit generic auras.
pub fn base_zone_contract(entry: &Value) -> Option<BaseZoneContract> {
    if crate::bounded_reductions::leader_entry(entry) { return Some(BaseZoneContract::MaximumLeader); }
    let sides: Sides = serde_json::from_value(entry["abilities"].clone()).ok()?;
    if !entry["card"]["unique"].is_boolean() || !entry["attachCondition"].is_null() || !entry["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox"].iter().any(|key| !entry["card"][key].is_null()) { return None; }
    if entry["baseClass"] == "BaseCard" && entry["card"]["types"] == json!(["base"])
        && entry["card"]["unique"] == false && entry["card"]["hp"] == 26 && entry["card"]["cost"].is_null()
        && entry["card"]["power"].is_null() && entry["keywordValues"] == json!({}) && entry["card"]["keywords"] == json!([])
        && Topology::CardOnly.accepts_single_registration_per_side(&sides) {
        let axis = match &sides.card[0].effect {
            Effect::ModifyStats { power, hp, .. } if number(power, 0) && number(hp, 1) => StatAxis::Hp,
            _ => StatAxis::Power,
        };
        let text = match axis { StatAxis::Hp => "Each leader unit you control gets +0/+1.", StatAxis::Power => "Each leader unit you control gets +1/+0." };
        let contract = BaseZoneContract::LeaderStats(axis);
        return (entry["card"]["text"] == text && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
            && ["rules", "deployBox", "upgradePower", "upgradeHp"].iter().all(|key| entry["card"][key].is_null())
            && contract.accepts(&sides)).then_some(contract);
    }
    let contract = BaseZoneContract::RaidLeader;
    (entry["card"]["text"] == "Friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["deployBox"] == "Raid 1\nOther friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["epicAction"] == "Epic Action: If you control 6 or more resources, deploy this leader."
        && entry["card"]["rules"] == "Abilities that refer to a card’s cost always refer to its printed cost, regardless of modifiers.\n"
        && entry["card"]["upgradePower"].is_null() && entry["card"]["upgradeHp"].is_null()
        && entry["baseClass"] == "LeaderUnitCard" && entry["card"]["types"] == json!(["leader"])
        && entry["card"]["unique"] == true && entry["card"]["cost"] == 6 && entry["card"]["hp"] == 5
        && entry["card"]["power"] == 5 && entry["card"]["arena"] == "ground"
        && entry["keywordValues"] == json!({"raid":1}) && entry["card"]["keywords"] == json!(["raid"])
        && contract.accepts(&sides)).then_some(contract)
}

#[cfg(test)]
#[path = "activation_contract_tests.rs"]
mod tests;
