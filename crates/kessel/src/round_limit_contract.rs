//! Whole-source proof for five native registration-scoped per-round triggers.
//! This module only recognizes source envelopes and returns data projections.
//! Runtime limit ownership and event timing remain separate responsibilities.
use serde_json::{json, Value};
use std::collections::BTreeMap;
const TEMPLATES: [&str; 5] = [
    r##"{"leader":[],"unit":[],"card":[{"trigger":"eventHooks","title":"Create a Mandalorian token","costs":[],"optional":true,"limit":{"kind":"abilityLimit","factory":"perRound","args":[1],"scope":"registration","sharedAcrossHooks":true},"effect":{"kind":"createTokenUnit","tokenType":"mandalorian","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},"hooks":[{"event":"onCardPlayed","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:738:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"player","optional":false}]},"right":{"kind":"ref","path":["player"]}},"right":{"kind":"dataCall","method":"isUpgrade","receiver":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:738:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false}]},"args":[],"optionalCall":false,"optionalReceiver":false}},"right":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:738:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"attachTarget","optional":false}]},"right":{"kind":"ref","path":["source"]}}}}]}]}"##,
    r##"{"leader":[],"unit":[],"card":[{"trigger":"eventHooks","title":"Deal 1 damage to any number of bases","costs":[],"limit":{"kind":"abilityLimit","factory":"perRound","args":[1],"scope":"registration","sharedAcrossHooks":true},"targetResolver":{"owner":"any","type":"base","zone":"anyAttackable","mode":"unlimited","optional":true},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}},"hooks":[{"event":"onDamageDealt","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"unary","op":"!","value":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:787:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"willDefeat","optional":false}]}},"right":{"kind":"dataCall","method":"isUnit","receiver":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:787:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false}]},"args":[],"optionalCall":false,"optionalReceiver":false}},"right":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:787:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false},{"key":"controller","optional":false}]},"right":{"kind":"ref","path":["player"]}}}}]}]}"##,
    r##"{"leader":[],"unit":[],"card":[{"trigger":"eventHooks","title":"Ready Kachirho Militia","costs":[],"limit":{"kind":"abilityLimit","factory":"perRound","args":[1],"scope":"registration","sharedAcrossHooks":true},"effect":{"kind":"ready","target":"self"},"hooks":[{"event":"onAttackDeclared","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:821:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"attackerLastKnownInformation","optional":false},{"key":"controller","optional":false}]},"right":{"kind":"ref","path":["player"]}},"right":{"kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:821:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"attackerLastKnownInformation","optional":false},{"key":"arena","optional":false}]},"right":{"kind":"const","value":"groundArena"}}},"right":{"kind":"dataCall","method":"some","receiver":{"kind":"dataCall","method":"getAllTargets","receiver":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:821:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"attack","optional":false}]},"args":[],"optionalCall":false,"optionalReceiver":false},"args":[{"kind":"dataLambda","bindings":["data:1072:0"],"body":{"kind":"dataCall","method":"isBase","receiver":{"kind":"dataPath","root":["data:1072:0"],"steps":[]},"args":[],"optionalCall":false,"optionalReceiver":false}}],"optionalCall":false,"optionalReceiver":false}}}]}]}"##,
    r##"{"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","title":"Give a Weakness token to a unit","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"giveWeakness","target":"chosen","amount":{"kind":"const","value":1}}},{"trigger":"eventHooks","title":"Give a Weakness token to a unit","costs":[],"optional":true,"limit":{"kind":"abilityLimit","factory":"perRound","args":[1],"scope":"registration","sharedAcrossHooks":true},"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"giveWeakness","target":"chosen","amount":{"kind":"const","value":1}},"hooks":[{"event":"onCardDefeated","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"enumTypeTest","test":"isUnit","args":[{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:1271:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"lastKnownInformation","optional":false},{"key":"type","optional":false}]}]},"right":{"kind":"binary","op":"!==","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:1271:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"lastKnownInformation","optional":false},{"key":"controller","optional":false}]},"right":{"kind":"ref","path":["player"]}}},"right":{"kind":"dataCall","method":"some","receiver":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:1271:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"lastKnownInformation","optional":false},{"key":"upgrades","optional":false}]},"args":[{"kind":"dataLambda","bindings":["data:1508:0"],"body":{"kind":"dataCall","method":"isWeakness","receiver":{"kind":"dataPath","root":["data:1508:0"],"steps":[]},"args":[],"optionalCall":false,"optionalReceiver":false}}],"optionalCall":false,"optionalReceiver":true}}}]}]}"##,
    r##"{"leader":[],"unit":[],"card":[{"trigger":"eventHooks","title":"Draw a card","costs":[],"optional":true,"limit":{"kind":"abilityLimit","factory":"perRound","args":[1],"scope":"registration","sharedAcrossHooks":true},"effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}},"hooks":[{"event":"onCardDefeated","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"enumTypeTest","test":"isUnit","args":[{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:826:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"lastKnownInformation","optional":false},{"key":"type","optional":false}]}]},"right":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:826:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false},{"key":"unique","optional":false}]}},"right":{"kind":"binary","op":"!==","left":{"kind":"dataPath","root":["event"],"argumentBinding":{"id":"callback:826:0","bindAt":"callbackInvocation","readAt":"expressionEvaluation"},"steps":[{"key":"card","optional":false}]},"right":{"kind":"ref","path":["source"]}}}}]}]}"##,
];
const METADATA: &str = r##"[{"text":"When you play an upgrade on this unit: You may create a Mandalorian token. Use this ability only once each round.","unique":true,"keywords":[],"title":"Gar Saxon","subtitle":"Coveting Power","rules":"Attaching a token upgrade is not considered playing an upgrade and does not trigger Gar Saxon's ability.\n","types":["unit"],"arena":"ground","traits":["mandalorian","trooper"],"aspects":["vigilance","villainy"],"keywordValues":{}},{"text":"When a friendly unit is dealt damage and survives: Deal 1 damage to any number of bases. Use this ability only once each round.","unique":false,"keywords":[],"title":"Rancor Keeper","subtitle":"","rules":"All damage dealt by Rancor Keeper's ability is dealt simultaneously.\n","types":["unit"],"arena":"ground","traits":["underworld"],"aspects":["vigilance","aggression"],"keywordValues":{}},{"text":"Hidden (This unit can't be attacked if it was played this phase.)\nWhen an enemy ground unit attacks your base: Ready this unit. Use this ability only once each round.","unique":false,"keywords":["hidden"],"title":"Kachirho Militia","subtitle":"","rules":"If a unit has Hidden and Sentinel at the same time, it can be attacked.\n","types":["unit"],"arena":"ground","traits":["wookiee","trooper"],"aspects":["aggression","heroism"],"keywordValues":{"hidden":1}},{"text":"When Played: You may give a Weakness token to a unit.\nWhen an enemy unit with a Weakness token on it is defeated: You may give a Weakness token to a unit. Use this ability only once each round.","unique":true,"keywords":[],"title":"Nuvo Vindi","subtitle":"Blue Shadow Perfected","rules":null,"types":["unit"],"arena":"ground","traits":["separatist"],"aspects":["vigilance","villainy"],"keywordValues":{}},{"text":"Ambush (After you play this unit, he may ready and attack an enemy unit.)\n\nWhen another unique unit is defeated: You may draw a card. Use this ability only once each round.","unique":true,"keywords":["ambush"],"title":"Agent Kallus","subtitle":"Seeking the Rebels","rules":"If Agent Kallus is defeated simultaneously with other unique units, you still may draw a card.\nAmbush is an ability that triggers at the same time as “When Played” abilities. If there is no enemy unit that can be attacked, the unit does not ready.\n(ERRATA) Templating update: “Unique” becomes “<uq>”.\n","types":["unit"],"arena":"ground","traits":["imperial","trooper"],"aspects":["command"],"keywordValues":{"ambush":1}}]"##;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Registration {
    PlayedUpgradeOnSelf,
    FriendlyUnitNonlethalDamage,
    EnemyGroundAttackAtBase,
    EnemyWeaknessUnitDefeated,
    OtherUniqueUnitDefeated,
}
impl Registration {
    pub(crate) fn trigger(self)->&'static str {match self {
        Self::PlayedUpgradeOnSelf=>"roundLimit.playedUpgradeOnSelf",
        Self::FriendlyUnitNonlethalDamage=>"roundLimit.friendlyUnitNonlethalDamage",
        Self::EnemyGroundAttackAtBase=>"roundLimit.enemyGroundAttackAtBase",
        Self::EnemyWeaknessUnitDefeated=>"roundLimit.enemyWeaknessUnitDefeated",
        Self::OtherUniqueUnitDefeated=>"roundLimit.otherUniqueUnitDefeated",
    }}
    pub(crate) fn from_trigger(trigger:&str)->Option<Self> {Some(match trigger {
        "roundLimit.playedUpgradeOnSelf"=>Self::PlayedUpgradeOnSelf,
        "roundLimit.friendlyUnitNonlethalDamage"=>Self::FriendlyUnitNonlethalDamage,
        "roundLimit.enemyGroundAttackAtBase"=>Self::EnemyGroundAttackAtBase,
        "roundLimit.enemyWeaknessUnitDefeated"=>Self::EnemyWeaknessUnitDefeated,
        "roundLimit.otherUniqueUnitDefeated"=>Self::OtherUniqueUnitDefeated,
        _=>return None,
    })}
    pub(crate) fn ordinal(self)->u8 {u8::from(self==Self::EnemyWeaknessUnitDefeated)}
}
#[derive(Debug, Clone)]
pub(crate) struct Projection {
    pub abilities: Value,
    pub registration: Registration,
    pub ordinal: u8,
}
/// Bounded inspection also detects malformed registration-limit values before
/// source families can discard fields. A collection's numeric execution bound
/// is a different existing IR contract, never an ability-use budget.
pub(crate) fn marker(value: &Value) -> Result<bool, String> {
    let mut todo = vec![(value, 0usize)];
    let mut count = 0usize;
    let mut found = false;
    while let Some((v, depth)) = todo.pop() {
        count += 1;
        if count > 32768 || depth > 64 {
            return Err("round-limit inspection capacity exceeded".into());
        }
        match v {
            Value::Object(o) => {
                let collection_bound = o.len() == 5
                    && o.get("kind") == Some(&json!("collection"))
                    && o.contains_key("name")
                    && o.contains_key("receiver")
                    && o.contains_key("args")
                    && o.get("limit").is_some_and(|n| n.as_u64() == Some(256));
                found |= o.get("kind") == Some(&json!("abilityLimit"))
                    || (o.contains_key("limit") && !collection_bound)
                    || o.keys()
                        .any(|k| k.starts_with("roundLimit") || k.starts_with("round_limit"))
                    || o.get("trigger")
                        .and_then(Value::as_str)
                        .is_some_and(|s| s.starts_with("roundLimit"));
                if count + todo.len() + o.len() > 32768 {
                    return Err("round-limit inspection capacity exceeded".into());
                }
                todo.extend(o.values().map(|x| (x, depth + 1)));
            }
            Value::Array(items) => {
                if count + todo.len() + items.len() > 32768 {
                    return Err("round-limit inspection capacity exceeded".into());
                }
                todo.extend(items.iter().map(|x| (x, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(found)
}
fn alpha_normalize(value: &Value) -> Value {
    fn collect(
        v: &Value,
        callback: &mut BTreeMap<String, String>,
        data: &mut BTreeMap<String, String>,
    ) {
        match v {
            Value::Object(o) => {
                if let Some(id) = o
                    .get("argumentBinding")
                    .and_then(|b| b.get("id"))
                    .and_then(Value::as_str)
                {
                    if !callback.contains_key(id) {
                        callback.insert(id.into(), format!("callback:closed:{}", callback.len()));
                    }
                }
                if o.get("kind") == Some(&json!("dataLambda")) {
                    if let Some(bindings) = o.get("bindings").and_then(Value::as_array) {
                        for binding in bindings {
                            if let Some(id) = binding.as_str() {
                                if !data.contains_key(id) {
                                    data.insert(id.into(), format!("data:closed:{}", data.len()));
                                }
                            }
                        }
                    }
                }
                for item in o.values() {
                    collect(item, callback, data);
                }
            }
            Value::Array(a) => {
                for item in a {
                    collect(item, callback, data);
                }
            }
            _ => {}
        }
    }
    fn replace(
        v: &mut Value,
        callbacks: &BTreeMap<String, String>,
        data: &BTreeMap<String, String>,
    ) {
        match v {
            Value::Object(o) => {
                if let Some(b) = o.get_mut("argumentBinding") {
                    if let Some(id) = b
                        .get("id")
                        .and_then(Value::as_str)
                        .and_then(|id| callbacks.get(id))
                        .cloned()
                    {
                        b["id"] = json!(id);
                    }
                }
                if o.get("kind") == Some(&json!("dataLambda")) {
                    if let Some(a) = o.get_mut("bindings").and_then(Value::as_array_mut) {
                        for b in a {
                            if let Some(id) = b.as_str().and_then(|id| data.get(id)).cloned() {
                                *b = json!(id);
                            }
                        }
                    }
                }
                if let Some(a) = o.get_mut("root").and_then(Value::as_array_mut) {
                    for b in a {
                        if let Some(id) = b.as_str().and_then(|id| data.get(id)).cloned() {
                            *b = json!(id);
                        }
                    }
                }
                for item in o.values_mut() {
                    replace(item, callbacks, data);
                }
            }
            Value::Array(a) => {
                for item in a {
                    replace(item, callbacks, data);
                }
            }
            _ => {}
        }
    }
    let (mut callbacks, mut data) = (BTreeMap::new(), BTreeMap::new());
    collect(value, &mut callbacks, &mut data);
    let mut out = value.clone();
    replace(&mut out, &callbacks, &data);
    out
}
pub(crate) fn prepare(entry: &Value) -> Result<Option<Projection>, String> {
    let printed_claim=entry["card"]["text"].as_str().is_some_and(|text|text.to_ascii_lowercase().contains("once each round"));
    if !marker(entry)?&&!printed_claim {
        return Ok(None);
    }
    let fail = || {
        "per-round limits require the complete closed native source registration and metadata"
            .to_string()
    };
    if entry["status"] != "supported"
        || !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || entry["baseClass"] != "NonLeaderUnitCard"
        || entry["card"]["types"] != json!(["unit"])
        || entry["card"]["arena"] != "ground"
        || !entry["piloting"].is_null()
        || !entry["attachCondition"].is_null()
        || entry["abilities"].as_object().map(|o| o.len()) != Some(3)
        || !["cost", "power", "hp"]
            .iter()
            .all(|k| entry["card"][k].as_u64().is_some_and(|n| n <= 1000))
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
        .any(|k| !entry["card"][k].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
    {
        return Err(fail());
    }
    fn valid_bindings(value: &Value) -> bool {
        match value {
            Value::Object(object) => {
                let valid = |value: &Value| {
                    value
                        .as_str()
                        .is_some_and(|id| !id.is_empty() && id.len() <= 128)
                };
                if object
                    .get("argumentBinding")
                    .is_some_and(|binding| !valid(&binding["id"]))
                {
                    return false;
                }
                if object.get("kind") == Some(&json!("dataLambda"))
                    && !object
                        .get("bindings")
                        .and_then(Value::as_array)
                        .is_some_and(|items| items.iter().all(valid))
                {
                    return false;
                }
                object.values().all(valid_bindings)
            }
            Value::Array(items) => items.iter().all(valid_bindings),
            _ => true,
        }
    }
    if !valid_bindings(&entry["abilities"]) {
        return Err(fail());
    }
    let expected_keys: Vec<&str> = vec![
        "arena",
        "aspects",
        "cost",
        "deployBox",
        "epicAction",
        "hp",
        "id",
        "internalName",
        "keywords",
        "power",
        "rules",
        "setCodes",
        "setId",
        "subtitle",
        "text",
        "title",
        "traits",
        "types",
        "unique",
        "upgradeHp",
        "upgradePower",
    ];
    if entry["card"]
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect::<Vec<_>>())
        != Some(expected_keys)
    {
        return Err(fail());
    }
    let actual = alpha_normalize(&entry["abilities"]);
    let Some(kind) = TEMPLATES.iter().position(|template| {
        alpha_normalize(&serde_json::from_str::<Value>(template).expect("closed limit template"))
            == actual
    }) else {
        return Err(fail());
    };
    let profiles: Value = serde_json::from_str(METADATA).expect("closed metadata profiles");
    let profile = &profiles[kind];
    if [
        "text", "unique", "keywords", "title", "subtitle", "rules", "types", "arena", "traits",
        "aspects",
    ]
    .iter()
    .any(|k| entry["card"][k] != profile[k])
        || entry["keywordValues"] != profile["keywordValues"]
    {
        return Err(fail());
    }
    let mut outside = entry.clone();
    outside["abilities"] = json!({"card":[],"unit":[],"leader":[]});
    if marker(&outside)? {
        return Err(fail());
    }
    let ordinal = if kind == 3 { 1 } else { 0 };
    let mut abilities = entry["abilities"].clone();
    let registration = &mut abilities["card"][ordinal];
    registration.as_object_mut().unwrap().remove("limit");
    registration.as_object_mut().unwrap().remove("hooks");
    registration["trigger"] = json!("whenPlayed");
    let registration = [
        Registration::PlayedUpgradeOnSelf,
        Registration::FriendlyUnitNonlethalDamage,
        Registration::EnemyGroundAttackAtBase,
        Registration::EnemyWeaknessUnitDefeated,
        Registration::OtherUniqueUnitDefeated,
    ][kind];
    Ok(Some(Projection {
        abilities,
        registration,
        ordinal: ordinal as u8,
    }))
}

impl Projection {
    /// Called only with the complete source proof. Internal trigger tags are
    /// serialized into program identity and rejected in every raw source input.
    pub(crate) fn compile(&self, first_id:usize)->Result<(crate::ability_ir::CardPrograms,Vec<crate::ability_ir::Program>),String> {
        use crate::ability_ir::{Ability,CardPrograms,Instruction,Program};
        if self.ordinal!=self.registration.ordinal() {return Err("round-limit projection ordinal differs from closed registration kind".into());}
        let mut card=CardPrograms::default();let mut programs=Vec::new();
        let values=self.abilities["card"].as_array().ok_or("round-limit projection missing registrations")?;
        for (ordinal,value) in values.iter().enumerate() {
            let ability:Ability=serde_json::from_value(value.clone()).map_err(|e|e.to_string())?;
            let mut program=Program::compile(&ability)?;
            if ordinal==usize::from(self.ordinal) {
                let commit=usize::from(matches!(program.instructions.first(),Some(Instruction::Select(_))));
                if program.instructions.len()!=commit+2 || !matches!(program.instructions[commit],Instruction::Apply(_)|Instruction::Group(_)) || !matches!(program.instructions.last(),Some(Instruction::EndStep)) {return Err("round-limit source projection requires one complete scalar handler".into());}
                program.instructions.insert(commit,Instruction::Commit);
                program.event_end=program.event_end.checked_add(1).ok_or("round-limit program boundary overflow")?;
                program.trigger=self.registration.trigger().into();
                program.executable=Err("registration limit uses shared Rust interpreter");
            }
            card.card.push(u16::try_from(first_id+programs.len()).map_err(|_|"round-limit program capacity exceeded")?);
            programs.push(program);
        }
        Ok((card,programs))
    }
}
