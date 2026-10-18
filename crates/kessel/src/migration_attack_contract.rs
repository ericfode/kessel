//! Closed source normalization for conditional native attack power bonuses.
//!
//! This module does not evaluate predicates or admit runtime state. The pinned
//! InitiateAttackSystem applies conditional entries after defender selection.
//! CardLastingEffectSystem first gates registration on the condition; an effect
//! registered then keeps its condition live through attack cleanup. Consumers
//! must preserve both stages and the initiating physical source/player.
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Condition {
    AttackerHasTrait(String),
    InitiatingPlayerHasInitiative,
    InitiatingPlayerHasFewerResources,
    SharesTraitWithOtherFriendlyUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionalPower {
    pub condition: Condition,
    pub power: i16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Callback {
    kind: String,
    parameters: Vec<Parameter>,
    body: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    root: String,
    role: String,
}

fn binding(root: &str) -> bool {
    !root.is_empty()
        && root.len() <= 64
        && root
            .bytes()
            .enumerate()
            .all(|(i, c)| c.is_ascii_alphabetic() || c == b'_' || (i != 0 && c.is_ascii_digit()))
}

/// Normalize only the four closed source predicates. Equality against complete
/// expression objects deliberately rejects unknown keys at every nested level,
/// alternate receivers, optional chains and outer lexical captures.
pub(crate) fn normalize_condition(value: &Value) -> Result<Condition, String> {
    let c: Callback = serde_json::from_value(value.clone())
        .map_err(|e| format!("invalid conditional attack callback: {e}"))?;
    if c.kind != "attackCallback"
        || !(1..=2).contains(&c.parameters.len())
        || c.parameters[0].role != "attack"
        || !binding(&c.parameters[0].root)
        || c.parameters.get(1).is_some_and(|p| {
            p.role != "systemContext" || !binding(&p.root) || p.root == c.parameters[0].root
        })
    {
        return Err("conditional attack needs an attack and optional initiating context".into());
    }
    let attack = &c.parameters[0].root;
    if c.parameters.len() == 1 {
        if let Some(trait_name) = c.body["args"][0]["value"].as_str() {
            let literal_trait = !trait_name.is_empty()
                && trait_name.len() <= 64
                && trait_name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b == b' ' || b == b'-');
            if literal_trait
                && c.body
                    == json!({"kind":"query","name":"hasSomeTrait",
                    "receiver":{"kind":"ref","path":[attack,"attacker"]},
                    "args":[{"kind":"const","value":trait_name}]})
            {
                return Ok(Condition::AttackerHasTrait(trait_name.into()));
            }
        }
        if c.body
            == json!({"kind":"attackArenaSome",
            "receiver":{"kind":"ref","path":[attack,"attackingPlayer"]},
            "filter":{
                "otherThan":{"kind":"ref","path":[attack,"attacker"]},
                "trait":{"kind":"distinctList","receiver":{
                    "kind":"ref","path":[attack,"attacker","traits"]}}
            }})
        {
            return Ok(Condition::SharesTraitWithOtherFriendlyUnit);
        }
    } else {
        let context = &c.parameters[1].root;
        if c.body
            == json!({"kind":"query","name":"hasInitiative",
            "receiver":{"kind":"ref","path":[context,"player"]},"args":[]})
        {
            return Ok(Condition::InitiatingPlayerHasInitiative);
        }
        if c.body
            == json!({"kind":"binary","op":"<",
            "left":{"kind":"ref","path":[context,"player","resources","length"]},
            "right":{"kind":"ref","path":[context,"player","opponent","resources","length"]}})
        {
            return Ok(Condition::InitiatingPlayerHasFewerResources);
        }
    }
    Err("conditional attack predicate is outside the closed source domain".into())
}

/// A late entry is literal positive power only. Early keyword/stat effects stay
/// with native_attack's existing contract and must not be moved to this stage.
pub(crate) fn normalize_lasting_entry(value: &Value) -> Result<ConditionalPower, String> {
    let power = value["effect"]["power"]["value"]
        .as_i64()
        .filter(|n| (1..=10).contains(n))
        .ok_or("conditional attack power must be a literal in 1..10")? as i16;
    let condition = normalize_condition(&value["condition"])?;
    if *value
        != json!({"kind":"attackLastingEntry","evaluation":"afterAttackCreated",
        "effect":{"kind":"modifyStats",
            "target":{"kind":"ref","path":["attackParticipant"]},
            "duration":"attack","power":{"kind":"const","value":power},
            "hp":{"kind":"const","value":0}},
        "condition":value["condition"]})
    {
        return Err("conditional attack requires a complete late attacker power entry".into());
    }
    Ok(ConditionalPower { condition, power })
}

/// Validate the complete ordinary source around normalized late entries. This
/// does not erase any unsupported sibling, header, helper or metadata field.
/// The clone is used only for validation by the established early contract;
/// callers must keep the original late entries for execution.
pub(crate) fn entry(value: &Value) -> bool {
    let mut early = value.clone();
    let Some(abilities) = early
        .get_mut("abilities")
        .and_then(|sides| sides.get_mut("card"))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };
    if abilities.len() != 1 {
        return false;
    }
    let ability = &mut abilities[0];
    let Some(entries) =
        ability["effect"]["properties"]["properties"]["attackerLastingEffects"].as_array()
    else {
        return false;
    };
    if entries.len() > 8 {
        return false;
    }
    let mut late_count = 0;
    let mut before = Vec::new();
    for effect in entries {
        if effect["evaluation"] == "afterAttackCreated" {
            if normalize_lasting_entry(effect).is_err() {
                return false;
            }
            late_count += 1;
        } else {
            before.push(effect.clone());
        }
    }
    if late_count == 0 {
        return false;
    }
    ability["effect"]["properties"]["properties"]["attackerLastingEffects"] = json!(before);
    // Compare typed headers so omitted defaults remain equivalent, while no
    // unrelated action, costs, continuation or attacker filter is introduced.
    let Ok(actual) = serde_json::from_value::<crate::ability_ir::Ability>(ability.clone()) else {
        return false;
    };
    let unit = value["baseClass"] == "NonLeaderUnitCard";
    let Ok(expected) = serde_json::from_value::<crate::ability_ir::Ability>(json!({
        "trigger":if unit {"whenPlayed"} else {"event"},
        "title":actual.title,"costs":[],"optional":unit,
        "targetResolver":{"owner":"any","type":"any","zone":"anyAttackable"},
        "effect":{"kind":"initiateAttack","initiationProperty":true,
            "properties":{"kind":"attackProperties","properties":{
                "target":"chosen","attackerLastingEffects":before}}}
    })) else {
        return false;
    };
    serde_json::to_value(actual).ok() == serde_json::to_value(expected).ok()
        && crate::native_attack::entry(&early)
}
