//! Closed modal token creation followed by an exact resolved singleton recipient.
//! Raw complete-source proof precedes lowering; generated identity is never a query.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program};
use serde_json::{json, Value};
const REASON: &str =
    "generated token continuation requires complete closed modal source and token profiles";
fn profile() -> Value {
    serde_json::from_str(r###"{"setCode":"ASH_257","sourcePath":"game/cards/08_ASH/events/ChooseYourPath.ts","sourceHash":"df620d876e2203b0aac3db5c0e2ce6f8ced4c916dc5638d66542aac229409f4a","abilities":{"leader":[],"unit":[],"card":[{"trigger":"event","title":"Choose one: If you control a Force Unit, heal 5 damage from your base. If you control a Mandalorian unit, create a Mandalorian token and give an Advantage token to it.","costs":[],"targetResolver":{"kind":"choiceSelection","mode":"select","resultBinding":"select","namedResultBinding":"selects.target.choice","defaults":"pinnedTargetResolver","chooser":{"kind":"const","value":"self"},"showUnresolvable":true,"choices":{"entries":[{"labelParts":[{"kind":"const","value":"If you control a "},{"kind":"const","value":"Force"},{"kind":"const","value":" unit, heal 5 damage from your base"}],"effect":{"kind":"conditional","condition":{"kind":"query","name":"hasSomeArenaUnit","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"force"}]}}]},"onTrue":{"kind":"heal","target":"ownBase","amount":{"kind":"const","value":5}},"onFalse":{"kind":"noAction"}}},{"labelParts":[{"kind":"const","value":"If you control a "},{"kind":"const","value":"Mandalorian"},{"kind":"const","value":" unit, create a Mandalorian token and give an Advantage token to it"}],"effect":{"kind":"conditional","condition":{"kind":"query","name":"hasSomeArenaUnit","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"mandalorian"}]}}]},"onTrue":{"kind":"sequence","effects":[{"kind":"createTokenUnit","tokenType":"mandalorian","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},{"kind":"registrationSystem","system":"giveAdvantage","contextTarget":"self","defaults":"pinnedSystem","properties":{"target":{"kind":"eventResult","contextBinding":{"id":"callback:2143:0","role":"properties:giveAdvantage","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"},"collection":"resolvedEvents","eventIndex":0,"path":["generatedTokens"],"optionalAccess":[2]}}}]},"onFalse":{"kind":"noAction"}}}],"contextBinding":"resolverContextByReference","keyOrder":"javascriptPropertyOrder","duplicatePolicy":"lastAssignmentWins","factory":false}},"effect":{"kind":"resolveChoice","resultBinding":"select"}}]},"className":"ChooseYourPath","baseClass":"EventCard","id":"2613847155","internalName":"choose-your-path","status":"supported","card":{"title":"Choose Your Path","subtitle":"","cost":2,"hp":null,"power":null,"text":"Choose one:\nIf you control a Force unit, heal 5 damage from your base.\nIf you control a Mandalorian unit, create a Mandalorian token and give an Advantage token to it.\n","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"2613847155","aspects":["heroism"],"traits":["force","mandalorian"],"keywords":[],"types":["event"],"setId":{"set":"ASH","number":257},"setCodes":[{"set":"ASH","number":257}],"internalName":"choose-your-path"},"dataHash":"75e56e44abb274b0a9ed765940d8bea97a269ab798982b04c0b1c189b5250417","keywordValues":{},"requiredCapabilities":["binary:&&","cardType:event","ir:binary","ir:choiceSelection","ir:conditional","ir:const","ir:createTokenUnit","ir:eventResult","ir:heal","ir:noAction","ir:query","ir:ref","ir:registrationSystem","ir:resolveChoice","ir:sequence","query:hasSomeArenaUnit","query:hasSomeTrait","ref:card","ref:player","targetMode:select","trigger:event"],"diagnostics":[]}"###).expect("reviewed complete source")
}
pub(crate) fn prepare(raw: &Value, catalogue: &Value) -> Result<bool, String> {
    let mut todo = vec![(raw, 0usize)];
    let mut visited = 0;
    let mut marked = false;
    while let Some((v, depth)) = todo.pop() {
        visited += 1;
        if visited > 65536 || depth > 128 {
            return Err("generated token source inspection budget exceeded".into());
        }
        marked |= v["sourceHash"]
            == "df620d876e2203b0aac3db5c0e2ce6f8ced4c916dc5638d66542aac229409f4a"
            || v["className"] == "ChooseYourPath"
            || v["internalName"] == "choose-your-path"
            || (v["collection"] == "resolvedEvents"
                && v["path"] == json!(["generatedTokens"])
                && v["contextBinding"]["role"] == "properties:giveAdvantage");
        match v {
            Value::Array(a) => todo.extend(a.iter().map(|v| (v, depth + 1))),
            Value::Object(o) => todo.extend(o.values().map(|v| (v, depth + 1))),
            _ => {}
        }
        if todo.len().saturating_add(visited) > 65536 {
            return Err("generated token source inspection budget exceeded".into());
        }
    }
    if !marked {
        return Ok(false);
    }
    if !crate::admission::SourceContextPolicy::Closed.accepts(raw) {
        return Err(REASON.into());
    }
    if *raw != profile() {
        return Err(REASON.into());
    }
    crate::defeat_token_creation_contract::dependencies(catalogue)?;
    crate::named_exhaust_contingent_contract::dependencies(catalogue)?;
    Ok(true)
}
pub(crate) fn checked_program_id(start: usize) -> Result<u16, String> {
    let id = u16::try_from(start).map_err(|_| "generated token program capacity exceeded")?;
    id.checked_add(1)
        .ok_or("generated token program+1 capacity exceeded")?;
    Ok(id)
}
pub(crate) fn compiled() -> Result<Program, String> {
    let raw = profile();
    let a = &raw["abilities"]["card"][0];
    let choices = a["targetResolver"]["choices"]["entries"]
        .as_array()
        .ok_or(REASON)?;
    let mut skeleton = a.clone();
    skeleton.as_object_mut().unwrap().remove("targetResolver");
    skeleton["effect"] = json!({"kind":"noAction"});
    let a: Ability = serde_json::from_value(skeleton).map_err(|e| e.to_string())?;
    let mut p = Program::compile(&a)?;
    let force: Expr = serde_json::from_value(choices[0]["effect"]["condition"].clone())
        .map_err(|e| e.to_string())?;
    let mando: Expr = serde_json::from_value(choices[1]["effect"]["condition"].clone())
        .map_err(|e| e.to_string())?;
    let heal: Effect = serde_json::from_value(choices[0]["effect"]["onTrue"].clone())
        .map_err(|e| e.to_string())?;
    let create: Effect =
        serde_json::from_value(choices[1]["effect"]["onTrue"]["effects"][0].clone())
            .map_err(|e| e.to_string())?;
    let label = |i: usize| {
        choices[i]["labelParts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["value"].as_str().unwrap())
            .collect::<String>()
    };
    p.instructions = vec![
        Instruction::ModalChoice(crate::modal_choices::Menu {
            chooser: crate::modal_choices::Chooser::Own,
            prompt_title: "Select one".into(),
            branches: vec![
                crate::modal_choices::Branch {
                    label: label(0),
                    start: 1,
                    end: 4,
                    always_legal: false,
                },
                crate::modal_choices::Branch {
                    label: label(1),
                    start: 5,
                    end: 9,
                    always_legal: false,
                },
            ],
            end: 10,
            preserve_target: false,
            unless_branch: None,
            show_unresolvable: true,
        }),
        Instruction::Commit,
        Instruction::Conditional {
            condition: force,
            skip: 4,
        },
        Instruction::Apply(heal),
        Instruction::Jump { to: 10 },
        Instruction::Commit,
        Instruction::Conditional {
            condition: mando,
            skip: 9,
        },
        Instruction::Group(create),
        Instruction::GrantCreatedTokenAdvantage { creation_pc: 7 },
        Instruction::Jump { to: 10 },
        Instruction::EndStep,
    ];
    p.event_end = 10;
    p.select_ends.clear();
    p.executable = Err("generated token continuation uses shared interpreter");
    Ok(p)
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions.iter().any(|i| {
        matches!(i, Instruction::GrantCreatedTokenAdvantage { .. })
            || matches!(i,Instruction::ModalChoice(menu) if menu.show_unresolvable)
    })
}
pub(crate) fn program(p: &Program) -> bool {
    compiled()
        .ok()
        .is_some_and(|expected| serde_json::to_value(p).ok() == serde_json::to_value(expected).ok())
}
