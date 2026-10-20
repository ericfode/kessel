//! Captured defeat-event cause. Combat context is deliberately irrelevant:
//! ability damage during an attack remains noncombat damage.
use serde_json::{json, Value};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DefeatCause {
    #[default]
    Unknown = 0,
    Attack = 1,
    NonCombatDamage = 2,
    Ability = 3,
    FrameworkEffect = 4,
    UniqueRule = 5,
}
impl DefeatCause {
    pub fn from_raw(raw: u8) -> Option<Self> {
        Some(match raw {
            0 => Self::Unknown,
            1 => Self::Attack,
            2 => Self::NonCombatDamage,
            3 => Self::Ability,
            4 => Self::FrameworkEffect,
            5 => Self::UniqueRule,
            _ => return None,
        })
    }
    pub fn advantage_amount(self) -> Option<u16> {
        match self {
            Self::Unknown => None,
            Self::Attack => Some(2),
            _ => Some(3),
        }
    }
}
/// One byte holds independent depth (0..8) and event cause (0..5). Packing
/// avoids growing each of the 640 inline captured registrations. Accessors never
/// reinterpret a depth as a cause; invalid wire values are not constructed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TriggerCapture(u8);
impl TriggerCapture {
    pub fn new(depth: u8, cause: DefeatCause) -> Self {
        assert!(depth <= 8);
        Self(depth | ((cause as u8) << 4))
    }
    pub fn depth(self) -> u8 {
        self.0 & 15
    }
    pub fn cause(self) -> DefeatCause {
        DefeatCause::from_raw(self.0 >> 4).expect("typed defeat cause")
    }
    pub fn set_depth(&mut self, depth: u8) {
        *self = Self::new(depth, self.cause());
    }
}
pub(crate) fn marker(v: &Value) -> bool {
    crate::ability_ir::contains(v, |v| {
        v["kind"] == "defeatCauseAmount"
            || (v["kind"] == "historyHelperCall" && v["name"] == "getAdvantageTokenAmount")
            || (v["kind"] == "historyHelper" && v["name"] == "getAdvantageTokenAmount")
            || (v["kind"] == "dataPath"
                && v["steps"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|s| s["key"] == "defeatSource")))
    })
}
/// Verify the whole source/metadata envelope before normalization or any family
/// early return. Portable/raw callers cannot supply the lowered leaf directly.
pub(crate) fn prepare(entry: &Value) -> Result<Option<Value>, String> {
    if !marker(entry) {
        return Ok(None);
    }
    let invalid = || {
        "defeat cause requires the complete closed optional unit death source and exact event-cause helper".to_string()
    };
    let expected:Value=serde_json::from_str(r###"{"abilities":{"leader":[],"unit":[],"card":[{"trigger":"whenDefeated","title":"Give 2 Advantage tokens to a unit, or 3 instead if this unit wasn't defeated by combat damage","costs":[],"contextTitle":[{"kind":"const","value":"Give "},{"kind":"historyHelperCall","name":"getAdvantageTokenAmount","args":[{"kind":"historyAbilityContext","binding":{"id":"callback:1033:0","role":"contextTitle","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"}}]},{"kind":"const","value":" Advantage tokens to a unit"}],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"giveAdvantage","target":"chosen","amount":{"kind":"historyHelperCall","name":"getAdvantageTokenAmount","args":[{"kind":"historyAbilityContext","binding":{"id":"callback:1313:0","role":"properties:giveAdvantage","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"}}]},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"}}]},"historyHelpers":[{"kind":"historyHelper","name":"getAdvantageTokenAmount","parameters":["data:helper:getAdvantageTokenAmount:0"],"body":{"kind":"dataBlock","statements":[{"kind":"dataReturn","value":{"kind":"conditional","condition":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["data:helper:getAdvantageTokenAmount:0"],"steps":[{"key":"event","optional":false},{"key":"defeatSource","optional":false},{"key":"type","optional":false}]},"right":{"kind":"const","value":"attack"}},"then":{"kind":"const","value":2},"else":{"kind":"const","value":3}}}]}}]}"###).expect("source contract");
    let card:Value=serde_json::from_str(r###"{"title":"Shin Hati's Fiend Fighter","subtitle":"Compact and Agile","cost":2,"hp":1,"power":3,"text":"When Defeated: You may give 2 Advantage tokens to a unit. If this unit wasn't defeated by combat damage, you may give 3 Advantage tokens to that unit instead.","deployBox":null,"epicAction":"","unique":true,"rules":"\u201cCombat damage\u201d is only the damage dealt during the \u201cEnd attack\u201d step of an attack.\n","upgradePower":null,"upgradeHp":null,"aspects":["cunning","villainy"],"traits":["vehicle","fighter"],"arena":"space","keywords":[],"types":["unit"]}"###).expect("printed metadata");
    if entry["baseClass"] != "NonLeaderUnitCard"
        || entry["className"] != "ShinHatisFiendFighterCompactAndAgile"
        || entry["abilities"] != expected["abilities"]
        || entry["historyHelpers"] != expected["historyHelpers"]
        || entry["keywordValues"] != json!({})
    {
        return Err(invalid());
    }
    if !crate::admission::OrdinarySource::from_entry(entry).is_some() {
        return Err(invalid());
    }
    for (k, v) in card.as_object().unwrap() {
        if !["id", "internalName", "setId", "setCodes"].contains(&k.as_str())
            && entry["card"][k] != *v
        {
            return Err(invalid());
        }
    }
    let mut normalized = entry.clone();
    normalized
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("historyHelpers");
    if !crate::admission::closed_source_helpers(&normalized)
        || !normalized["attachCondition"].is_null()
        || !normalized["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox"]
            .iter()
            .any(|k| !normalized["card"][k].is_null())
    {
        return Err(invalid());
    }
    normalized["abilities"]["card"][0]["contextTitle"][1] = json!({"kind":"defeatCauseAmount"});
    normalized["abilities"]["card"][0]["effect"]["amount"] = json!({"kind":"defeatCauseAmount"});
    Ok(Some(normalized))
}

pub(crate) fn requires(program: &crate::ability_ir::Program) -> bool {
    program.context_title.iter().any(|e| {
        e.any(|e| matches!(e, crate::ability_ir::Expr::DefeatCauseAmount { .. }).then_some(true))
    })
}

pub(crate) fn registration(a: &crate::ability_ir::Ability) -> bool {
    let value:Value=serde_json::from_str(r###"{"trigger":"whenDefeated","title":"Give 2 Advantage tokens to a unit, or 3 instead if this unit wasn't defeated by combat damage","costs":[],"contextTitle":[{"kind":"const","value":"Give "},{"kind":"defeatCauseAmount"},{"kind":"const","value":" Advantage tokens to a unit"}],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"giveAdvantage","target":"chosen","amount":{"kind":"defeatCauseAmount"},"createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"}}"###).expect("closed normalized death registration");
    let expected: crate::ability_ir::Ability =
        serde_json::from_value(value).expect("typed death registration");
    serde_json::to_value(a).unwrap() == serde_json::to_value(expected).unwrap()
}
