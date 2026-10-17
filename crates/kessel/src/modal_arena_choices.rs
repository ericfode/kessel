//! Closed arena exhaustion and damage branches inside ordinary modal abilities.
//! This contract proves whole-source topology before reusing existing executors.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program, Target};
use crate::modal_choices::Chooser;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct Prepared {
    pub normalized: Value,
    programs: Vec<(String, Program)>,
}
impl Prepared {
    pub fn compile(&self, ability: &Ability) -> Result<Program, String> {
        self.programs
            .iter()
            .find(|(trigger, _)| trigger == &ability.trigger)
            .map(|(_, program)| program.clone())
            .ok_or_else(|| "arena modal registration has no prepared program".into())
    }
}
fn marker(value: &Value) -> bool {
    (value["kind"] == "choiceSelection"
        && value["choices"]["entries"]
            .as_array()
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    matches!(entry["effect"]["kind"].as_str(), Some("damage" | "exhaust"))
                        && entry["effect"]["target"]["kind"] == "arenaList"
                })
            }))
        || match value {
            Value::Object(fields) => fields.values().any(marker),
            Value::Array(items) => items.iter().any(marker),
            _ => false,
        }
}
fn arena_target(receiver: &str, arena: &str) -> Value {
    json!({"kind":"arenaList","receiver":{"kind":"ref","path":[receiver]},"predicate":{
        "kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{
            "kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":arena}},"right":{
                "kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":arena},"right":{"kind":"const","value":"anyArena"}},
                "right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":arena}}
            }
        }
    }})
}
fn expected_resolver(damage: bool) -> Value {
    let mut entries: Vec<Value> = [("Space", "spaceArena"), ("Ground", "groundArena")].into_iter().map(|(label, arena)| {
        let effect = if damage {
            json!({"kind":"damage","target":arena_target("game",arena),"amount":{"kind":"const","value":3}})
        } else {
            json!({"kind":"exhaust","target":arena_target("opponent",arena)})
        };
        json!({"labelParts":[{"kind":"const","value":label}],"effect":effect})
    }).collect();
    if damage {
        entries.push(json!({"labelParts":[{"kind":"const","value":"Pass"}],"effect":{"kind":"choiceNoAction","hasLegalTarget":{"kind":"const","value":true}}}));
    }
    json!({"kind":"choiceSelection","mode":"select","resultBinding":"select","namedResultBinding":"selects.target.choice","defaults":"pinnedTargetResolver",
        "chooser":{"kind":"const","value":"self"},"promptTitleParts":[{"kind":"const","value":"Choose an arena"}],
        "choices":{"entries":entries,"contextBinding":"resolverContextByReference","keyOrder":"javascriptPropertyOrder","duplicatePolicy":"lastAssignmentWins","factory":false}})
}

pub fn prepare(entry: &Value) -> Result<Option<Prepared>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let cards = entry["abilities"]["card"]
        .as_array()
        .ok_or("arena modal registrations missing")?;
    let damage = cards
        .first()
        .and_then(|raw| raw["targetResolver"]["choices"]["entries"][0]["effect"]["kind"].as_str())
        == Some("damage");
    if !crate::static_ongoing::closed_source_context(entry)
        || entry["baseClass"] != "NonLeaderUnitCard"
        || entry["card"]["types"] != json!(["unit"])
        || !entry["card"]["unique"].is_boolean()
        || !entry["attachCondition"].is_null()
        || !entry["piloting"].is_null()
        || ["unit", "leader"].iter().any(|side| {
            !entry["abilities"][side]
                .as_array()
                .is_some_and(Vec::is_empty)
        })
        || [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .any(|key| !entry["card"][key].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        || if damage {
            cards.len() != 2
                || entry["keywordValues"] != json!({})
                || entry["card"]["keywords"] != json!([])
        } else {
            cards.len() != 1
                || entry["keywordValues"] != json!({"raid":4})
                || entry["card"]["keywords"] != json!(["raid"])
        }
    {
        return Err(
            "arena modal requires complete ordinary death-exhaust or play/attack damage source"
                .into(),
        );
    }
    let expected = expected_resolver(damage);
    let mut normalized = entry.clone();
    for (index, raw) in cards.iter().enumerate() {
        let trigger = if damage {
            if index == 0 {
                "whenPlayed"
            } else {
                "onAttack"
            }
        } else {
            "whenDefeated"
        };
        if raw.as_object().is_none_or(|object| {
            object.len() != 5
                || object.keys().any(|key| {
                    !["trigger", "title", "costs", "targetResolver", "effect"]
                        .contains(&key.as_str())
                })
        }) || raw["trigger"] != trigger
            || !raw["title"]
                .as_str()
                .is_some_and(|title| !title.is_empty() && title.len() <= 1024)
            || raw["costs"] != json!([])
            || raw["targetResolver"] != expected
            || raw["effect"] != json!({"kind":"resolveChoice","resultBinding":"select"})
        {
            return Err(
                "arena modal resolver must retain exact closed arena branches and root header"
                    .into(),
            );
        }
        if damage && index == 1 {
            let mut first = cards[0].clone();
            let mut second = raw.clone();
            first.as_object_mut().unwrap().remove("trigger");
            second.as_object_mut().unwrap().remove("trigger");
            if first != second {
                return Err("arena damage aliases must agree in every field except trigger".into());
            }
        }
        let object = normalized["abilities"]["card"][index]
            .as_object_mut()
            .unwrap();
        object.remove("targetResolver");
        object.insert("effect".into(), json!({"kind":"noAction"}));
    }
    let mut programs = Vec::new();
    for (index, raw) in cards.iter().enumerate() {
        let ability: Ability =
            serde_json::from_value(normalized["abilities"]["card"][index].clone())
                .map_err(|error| error.to_string())?;
        let mut branches = Vec::new();
        for branch in raw["targetResolver"]["choices"]["entries"]
            .as_array()
            .unwrap()
        {
            let label = branch["labelParts"][0]["value"]
                .as_str()
                .unwrap()
                .to_owned();
            let pass = branch["effect"]["kind"] == "choiceNoAction";
            let effect = if pass {
                Effect::NoAction
            } else {
                serde_json::from_value(branch["effect"].clone())
                    .map_err(|error| error.to_string())?
            };
            let mut child = ability.clone();
            child.title = label.clone();
            let program = if !damage {
                // compile_effect deliberately rejects general death-time arena
                // exhaustion. Only the complete contract above may emit this leaf.
                let mut program = Program::compile(&child)?;
                program.instructions = vec![Instruction::Apply(effect), Instruction::EndStep];
                program.event_end = 1;
                program.select_ends.clear();
                program
            } else {
                child.effect = effect;
                Program::compile(&child)?
            };
            branches.push((label, program, pass));
        }
        let program = crate::modal_choices::compile_branches(
            &ability,
            Chooser::Own,
            "Choose an arena",
            &branches,
        )?;
        programs.push((ability.trigger.clone(), program));
    }
    Ok(Some(Prepared {
        normalized,
        programs,
    }))
}

/// Exact order already lives in State; modal global lists must never use the
/// compatibility enumeration for snapshots missing that evidence.
pub fn requires_global_order(program: &Program) -> bool {
    program
        .instructions
        .iter()
        .any(|instruction| matches!(instruction, Instruction::ModalChoice(_)))
        && program.instructions.iter().any(|instruction| {
            matches!(instruction,
            Instruction::Group(Effect::Damage{target:Target::ArenaList(list),..})
                if matches!(&list.receiver,Expr::Ref{path} if path==&["game"]))
        })
}
