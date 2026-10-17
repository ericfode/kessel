//! Bounded ordinary "attack with a unit" contract. Source callbacks stay closed:
//! only the candidate and the initiating physical source may be read.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target, CORE_KEYWORDS};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Properties {
    kind: String,
    properties: Values,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Values {
    target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attacker_condition: Option<Callback>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attacker_lasting_effects: Vec<Lasting>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Callback {
    kind: String,
    parameters: Vec<Parameter>,
    body: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    root: String,
    role: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lasting {
    kind: String,
    evaluation: String,
    effect: Box<Effect>,
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub power: i16,
    pub keywords: [u16; 9],
    pub presence: u16,
    pub deals_combat_damage_first: bool,
}
impl Properties {
    pub(crate) fn predicate(&self) -> Result<Option<Expr>, String> {
        let Some(c) = &self.properties.attacker_condition else {
            return Ok(None);
        };
        if c.kind != "attackCallback"
            || c.parameters.is_empty()
            || c.parameters.len() > 2
            || c.parameters[0].role != "candidate"
            || c.parameters[0].root.is_empty()
            || c.parameters.get(1).is_some_and(|p| {
                p.role != "systemContext" || p.root.is_empty() || p.root == c.parameters[0].root
            })
        {
            return Err(
                "native attack condition needs candidate and optional initiating context".into(),
            );
        }
        fn normalize(v: &mut Value, c: &Callback) -> Result<(), String> {
            match v["kind"].as_str() {
                Some("ref") => {
                    let path = v["path"]
                        .as_array()
                        .ok_or("attack callback reference path missing")?;
                    if path.first().and_then(Value::as_str) == Some(c.parameters[0].root.as_str()) {
                        // The candidate's controller is its owner: admitted state
                        // imports do not model control changes.
                        if path.len() == 2 && path[1] == "controller" {
                            v["path"] = serde_json::json!(["card", "controller"]);
                        } else if path.len() != 1 {
                            return Err(
                                "native attack candidate member reads are not implemented".into()
                            );
                        } else {
                            v["path"] = serde_json::json!(["card"]);
                        }
                    } else if c.parameters.get(1).is_some_and(|p| {
                        path.first().and_then(Value::as_str) == Some(p.root.as_str())
                    }) && path.len() == 2
                        && path[1] == "source"
                    {
                        v["path"] = serde_json::json!(["source"]);
                    } else if c.parameters.get(1).is_some_and(|p| {
                        path.first().and_then(Value::as_str) == Some(p.root.as_str())
                    }) && path.len() == 2
                        && path[1] == "player"
                    {
                        // systemContext.player is the initiating player; the
                        // physical source's controller is that same player.
                        v["path"] = serde_json::json!(["source", "controller"]);
                    } else {
                        return Err(
                            "native attack condition cannot capture outer callback values".into(),
                        );
                    }
                }
                Some("query") => {
                    let name = v["name"].as_str().ok_or("attack condition query missing")?;
                    let args = v["args"]
                        .as_array()
                        .ok_or("attack condition query args missing")?;
                    if !((matches!(name, "isUnit" | "isNonLeaderUnit") && args.is_empty())
                        || (matches!(name, "hasSomeTrait" | "hasSomeAspect")
                            && args.len() == 1
                            && args[0]["kind"] == "const"
                            && args[0]["value"].as_str().is_some_and(|n| !n.is_empty())))
                    {
                        return Err(
                            "native attack condition query is outside the candidate domain".into(),
                        );
                    }
                    normalize(&mut v["receiver"], c)?;
                }
                Some("binary") => {
                    if !matches!(v["op"].as_str(), Some("&&" | "!==" | "===")) {
                        return Err("native attack condition operator unsupported".into());
                    }
                    normalize(&mut v["left"], c)?;
                    normalize(&mut v["right"], c)?;
                }
                _ => return Err("native attack condition expression unsupported".into()),
            }
            Ok(())
        }
        let mut body = c.body.clone();
        normalize(&mut body, c)?;
        let e: Expr =
            serde_json::from_value(body).map_err(|e| format!("invalid attack predicate: {e}"))?;
        e.validate()?;
        Ok(Some(e))
    }
    pub(crate) fn modifiers(&self) -> Result<Modifiers, String> {
        if self.kind != "attackProperties"
            || self.properties.target != "chosen"
            || self.properties.attacker_lasting_effects.len() > 8
        {
            return Err("native attack needs static properties and one chosen attacker".into());
        }
        let mut out = Modifiers::default();
        for entry in &self.properties.attacker_lasting_effects {
            if entry.kind != "attackLastingEntry" || entry.evaluation != "beforeDefenderSelection" {
                return Err(
                    "native attack supports only unconditional early attacker modifiers".into(),
                );
            }
            let participant = |t: &Target| matches!(t,Target::Expression(e) if matches!(e,Expr::Ref{path} if path==&["attackParticipant"]));
            match entry.effect.as_ref() {
                Effect::ModifyStats {
                    target,
                    duration,
                    power: Expr::Const { value },
                    hp: Expr::Const { value: hp },
                } if participant(target)
                    && duration == "attack"
                    && hp.as_i64() == Some(0)
                    && value.as_i64().is_some_and(|n| (0..=10).contains(&n)) =>
                {
                    out.power = out
                        .power
                        .checked_add(value.as_i64().unwrap() as i16)
                        .ok_or("native attack power capacity exceeded")?;
                }
                Effect::GainKeyword {
                    target,
                    duration,
                    keyword,
                    amount: Expr::Const { value },
                } if participant(target)
                    && duration == "attack"
                    && ["grit", "saboteur", "overwhelm", "raid", "restore"]
                        .contains(&keyword.as_str()) =>
                {
                    let n = value
                        .as_u64()
                        .filter(|n| (1..=10).contains(n))
                        .ok_or("native attack keyword amount must be 1..10")?
                        as u16;
                    if !["raid", "restore"].contains(&keyword.as_str()) && n != 1 {
                        return Err("native attack boolean keyword amount must be one".into());
                    }
                    let i = CORE_KEYWORDS.iter().position(|k| *k == keyword).unwrap();
                    out.presence |= 1 << i;
                    out.keywords[i] = if i == 6 || i == 7 {
                        out.keywords[i]
                            .checked_add(n)
                            .ok_or("native attack keyword capacity exceeded")?
                    } else {
                        1
                    };
                }
                Effect::DealsCombatDamageFirst { target, duration }
                    if participant(target) && duration == "attack" =>
                {
                    out.deals_combat_damage_first = true;
                }
                _ => return Err(
                    "native attack lasting effect requires literal power, reviewed core keyword or first-strike"
                        .into(),
                ),
            }
        }
        Ok(out)
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.modifiers()?;
        self.predicate()?;
        Ok(())
    }
}
pub(crate) fn ability(a: &Ability) -> bool {
    matches!(&a.effect,Effect::InitiateAttack{properties,..} if properties.validate().is_ok())
        && matches!(a.trigger.as_str(), "event" | "whenPlayed" | "action")
        && a.then.is_none()
        && a.if_you_do.is_none()
        && !a.coordinate
        && !a.attacker_must_survive
        && a.attached_context.is_none()
        && a.gained_host.is_none()
        && a.target_resolvers.is_empty()
        && a.condition.is_none()
        && !a.requires_confirmation
        && a.context_title.is_empty()
        && a.target_resolver.as_ref().is_some_and(|r| {
            r.native.is_none()
                && r.owner == "any"
                && r.card_type == "any"
                && r.zone.as_deref() == Some("anyAttackable")
                && r.arena.is_none()
                && !r.optional
                && !r.is_multiple()
                && r.mode.is_none()
                && r.count.is_none()
                && r.prompt_title.is_none()
                && r.prompt_title_parts.is_empty()
        })
}
pub(crate) fn marker(v: &Value) -> bool { crate::ability_ir::contains(v,|v|v.as_object().is_some_and(|o|o.get("kind").is_some_and(|k|k=="initiateAttack"))) }
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(s) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    crate::admission::SourceContextPolicy::Closed.accepts(e)
        && s.card.len() == 1
        && s.leader.is_empty()
        && s.unit.is_empty()
        && ability(&s.card[0])
        && ((e["baseClass"] == "EventCard"
            && e["card"]["types"] == serde_json::json!(["event"])
            && s.card[0].trigger == "event"
            && !s.card[0].optional)
            || (e["baseClass"] == "NonLeaderUnitCard"
                && e["card"]["types"] == serde_json::json!(["unit"])
                && ["whenPlayed", "action"].contains(&s.card[0].trigger.as_str())))
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && e["card"]["unique"].is_boolean()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
}
