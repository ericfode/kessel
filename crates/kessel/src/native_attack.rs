//! Bounded ordinary "attack with a unit" contract. Source callbacks stay closed:
//! only the candidate and the initiating physical source may be read.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target, CORE_KEYWORDS};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    condition: Option<Value>,
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
            if entry.evaluation == "afterAttackCreated" {
                crate::migration_attack_contract::normalize_lasting_entry(
                    &serde_json::to_value(entry).map_err(|e| e.to_string())?,
                )?;
                continue;
            }
            if entry.condition.is_some()
                || entry.kind != "attackLastingEntry"
                || entry.evaluation != "beforeDefenderSelection"
            {
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
    /// Late descriptors retain their complete callback until the physical
    /// attack is created. Only successfully gated entries become live grants.
    pub(crate) fn conditional_power(
        &self,
    ) -> Result<Vec<crate::migration_attack_contract::ConditionalPower>, String> {
        if self.properties.attacker_lasting_effects.len() > 8 {
            return Err("native attack lasting-effect entries exceed capacity (8)".into());
        }
        self.properties
            .attacker_lasting_effects
            .iter()
            .filter(|e| e.evaluation == "afterAttackCreated")
            .map(|e| {
                crate::migration_attack_contract::normalize_lasting_entry(
                    &serde_json::to_value(e).map_err(|error| error.to_string())?,
                )
            })
            .collect()
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
        && (a.if_you_do.is_none() || advantage_continuation(a))
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
    // Keep late compositions under the reviewed complete ordinary header.
    // Its early-only validation clone returns here with no late entries.
    if e["abilities"]["card"].as_array().is_some_and(|a| {
        a.iter().any(|a| {
            a["effect"]["properties"]["properties"]["attackerLastingEffects"]
                .as_array()
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|e| e["evaluation"] == "afterAttackCreated")
                })
        })
    }) {
        return crate::migration_attack_contract::entry(e);
    }
    let Ok(s) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    if leader_composition(e, &s) {
        return true;
    }
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
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
}

// Compare typed registrations so omitted default fields remain equivalent while
// every supported header, controller, continuation and resolver field is closed.
fn same_registration(actual: &Ability, expected: Value) -> bool {
    let Ok(expected) = serde_json::from_value::<Ability>(expected) else {
        return false;
    };
    let (Ok(actual), Ok(expected)) = (serde_json::to_value(actual), serde_json::to_value(expected))
    else {
        return false;
    };
    actual == expected
}

/// One resolved initiation MetaEvent may be followed by a fresh scalar Advantage
/// selection. This does not permit another attack, callback factory or `then`.
/// The nested defender picker is mandatory; its parent resolver owns cancellation.
fn advantage_continuation(a: &Ability) -> bool {
    let Some(next) = a.if_you_do.as_deref() else {
        return false;
    };
    same_registration(
        a,
        json!({
            "trigger":"event", "title":a.title, "costs":[],
            "targetResolver":{"owner":"any","type":"any","zone":"anyAttackable"},
            "effect":{"kind":"initiateAttack","initiationProperty":true,
                "properties":{"kind":"attackProperties","properties":{"target":"chosen"}}},
            "ifYouDo":{"title":next.title,"costs":[],
                "targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},
                "effect":{"kind":"giveAdvantage","target":"chosen","amount":{"kind":"const","value":3},
                    "createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"}}
        }),
    )
}

/// A base-side native action and an ordinary deployed keyword aura use the
/// existing physical leader's side switch. Only the two reviewed static shapes
/// are composed; neither arbitrary leaders nor attack-end attacks are admitted.
fn leader_composition(e: &Value, s: &Sides) -> bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e)
        || e["baseClass"] != "LeaderUnitCard"
        || e["card"]["types"] != json!(["leader"])
        || e["card"]["unique"] != true
        || !e["card"]["hp"]
            .as_u64()
            .is_some_and(|n| (1..=1000).contains(&n))
        || !e["card"]["power"].as_u64().is_some_and(|n| n <= 1000)
        || e["card"]["arena"] != "ground"
        || !s.card.is_empty()
        || s.leader.len() != 1
        || s.unit.len() != 1
        || !e["piloting"].is_null()
        || !e["attachCondition"].is_null()
        || [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .any(|key| !e["card"][*key].is_null())
    {
        return false;
    }
    let a = &s.leader[0];
    let aura = &s.unit[0];
    let grit = e["keywordValues"] == json!({}) && e["card"]["keywords"] == json!(["grit"])
        && e["card"]["cost"] == 5
        && e["card"]["text"] == "Action [1 resource, Exhaust]: Attack with a Heroism unit. It gains Grit for this attack. (It gets +1/+0 for each damage on it.)"
        && e["card"]["deployBox"] == "Other friendly Heroism units gain Grit."
        && e["card"]["epicAction"] == "Epic Action: If you control 5 or more resources, deploy this leader."
        && e["card"]["rules"] == "If you use Omega’s leader ability, you must attack with a unit, if able. Units must be ready in order to attack.\n";
    let overwhelm = e["keywordValues"] == json!({"overwhelm":1}) && e["card"]["keywords"] == json!(["overwhelm"])
        && e["card"]["cost"] == 6
        && e["card"]["text"] == "Action [Exhaust]: Attack with a unit. It gains Overwhelm for this attack. (When attacking an enemy unit, deal excess damage to the opponent's base.)"
        && e["card"]["deployBox"] == "Overwhelm\nEach other friendly unit gains Overwhelm."
        && e["card"]["epicAction"] == "Epic Action: If you control 6 or more resources, deploy this leader."
        && e["card"]["rules"] == "If you use Maul's leader ability, you must attack with a unit, if able. Units must be ready in order to attack.\nShields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n";
    if !grit && !overwhelm {
        return false;
    }
    let keyword = if grit { "grit" } else { "overwhelm" };
    let mut properties = json!({"target":"chosen", "attackerLastingEffects":[{
        "kind":"attackLastingEntry", "evaluation":"beforeDefenderSelection", "effect":{
            "kind":"gainKeyword", "target":{"kind":"ref","path":["attackParticipant"]},
            "duration":"attack", "keyword":keyword, "amount":{"kind":"const","value":1}}}]});
    if grit {
        properties["attackerCondition"] = json!({"kind":"attackCallback",
        "parameters":[{"root":"attackBinding1_0","role":"candidate"}],
        "body":{"kind":"query","name":"hasSomeAspect","receiver":{"kind":"ref","path":["attackBinding1_0"]},"args":[{"kind":"const","value":"heroism"}]}});
    }
    let costs = if grit {
        json!([{"kind":"resources","amount":{"kind":"const","value":1}},{"kind":"exhaustSelf"}])
    } else {
        json!([{"kind":"exhaustSelf"}])
    };
    let self_ref = json!({"kind":"ref","path":["source"]});
    let card_ref = json!({"kind":"ref","path":["card"]});
    let is_unit = json!({"kind":"query","name":"isUnit","receiver":card_ref,"args":[]});
    let not_self = json!({"kind":"binary","op":"!==","left":card_ref,"right":self_ref});
    let predicate = if grit {
        json!({"kind":"binary","op":"&&",
        "left":{"kind":"binary","op":"&&","left":not_self,"right":is_unit},
        "right":{"kind":"query","name":"hasSomeAspect","receiver":card_ref,"args":[{"kind":"const","value":"heroism"}]}})
    } else {
        json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":is_unit,
            "right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","controller"]},"right":{"kind":"ref","path":["player"]}}},"right":not_self})
    };
    ability(a)
        && same_registration(
            a,
            json!({"trigger":"action","title":a.title,"costs":costs,
        "targetResolver":{"owner":"any","type":"any","zone":"anyAttackable"},
        "effect":{"kind":"initiateAttack","initiationProperty":true,"properties":{"kind":"attackProperties","properties":properties}}}),
        )
        && same_registration(
            aura,
            json!({"trigger":"constant","title":aura.title,"costs":[],
            "targetResolver":{"owner":"self","type":"unit","zone":"arena","predicate":predicate},
            "effect":{"kind":"gainKeyword","target":"chosen","duration":"persistent","keyword":keyword,"amount":{"kind":"const","value":1}}}),
        )
}
