//! Closed ordinary simultaneous upgrade return and independent unit Shield.
//! Selection/event storage and mutation remain in the shared prepared executor.
use crate::ability_ir::{Ability, Effect, Instruction, Program};
use serde_json::Value;
fn profile() -> Value {
    serde_json::from_str(r###"{"setCode":"ASH_232","sourcePath":"game/cards/08_ASH/events/FullOfSurprises.ts","sourceHash":"3cb0333b17b8ceecc49539d3fe48bfda6c3ea32a05ff4595f5095fcddd7af280","abilities":{"leader":[],"unit":[],"card":[{"trigger":"event","title":"Return an upgrade that costs 2 or less to its owner's hand. Give a Shield token to a unit","costs":[],"effect":{"kind":"simultaneous","effects":[{"kind":"select","targetResolver":{"owner":"any","type":"any","zone":"anyAttackable","predicate":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUpgrade","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"binary","op":"<=","left":{"kind":"ref","path":["card","cost"]},"right":{"kind":"const","value":2}}},"promptTitle":"Return an upgrade that costs 2 or less to its owner's hand"},"effect":{"kind":"returnToHand","target":"chosen"}},{"kind":"select","targetResolver":{"owner":"any","type":"any","zone":"anyAttackable","predicate":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"promptTitle":"Give a Shield token to a unit"},"effect":{"kind":"giveShield","target":"chosen","amount":{"kind":"const","value":1}}}]}}]},"className":"FullOfSurprises","baseClass":"EventCard","id":"4547747622","internalName":"full-of-surprises","status":"supported","card":{"title":"Full of Surprises","subtitle":"","cost":2,"hp":null,"power":null,"text":"Return an upgrade that costs 2 or less to its owner's hand.\nGive a Shield token to a unit.","deployBox":null,"epicAction":"","unique":false,"rules":"Abilities that return a card to hand must choose a card in play unless otherwise specified.\nAbilities that refer to a card\u2019s cost always refer to its printed cost, regardless of modifiers.\n","upgradePower":null,"upgradeHp":null,"id":"4547747622","aspects":["cunning"],"traits":["trick"],"keywords":[],"types":["event"],"setId":{"set":"ASH","number":232},"setCodes":[{"set":"ASH","number":232}],"internalName":"full-of-surprises"},"dataHash":"da677e84423dc75fa9ea7ae5c6ed157a1c448caa60632390ef277fa82016b101","keywordValues":{},"requiredCapabilities":["binary:&&","binary:<=","cardType:event","ir:binary","ir:const","ir:giveShield","ir:query","ir:ref","ir:returnToHand","ir:select","ir:simultaneous","query:isUnit","query:isUpgrade","ref:card","ref:card.cost","targetType:any","targetZone:anyAttackable","trigger:event"],"diagnostics":[]}"###).expect("reviewed Full of Surprises source")
}
pub(crate) fn prepare(raw: &Value) -> Result<bool, String> {
    let mut todo = vec![(raw, 0usize)];
    let mut count = 0usize;
    let mut marked = false;
    while let Some((v, depth)) = todo.pop() {
        count += 1;
        if count > 65536 || depth > 128 {
            return Err("upgrade return source inspection budget exceeded".into());
        }
        marked |= v["sourceHash"]
            == "3cb0333b17b8ceecc49539d3fe48bfda6c3ea32a05ff4595f5095fcddd7af280"
            || v["className"] == "FullOfSurprises"
            || v["internalName"] == "full-of-surprises"
            || v["kind"] == "simultaneous" && contains_return(v);
        match v {
            Value::Array(xs) => todo.extend(xs.iter().map(|v| (v, depth + 1))),
            Value::Object(xs) => todo.extend(xs.values().map(|v| (v, depth + 1))),
            _ => {}
        }
        if todo.len().saturating_add(count) > 65536 {
            return Err("upgrade return source inspection budget exceeded".into());
        }
    }
    if !marked {
        return Ok(false);
    }
    let mut actual = raw.clone();
    let mut expected = profile();
    let o = actual
        .as_object_mut()
        .ok_or("upgrade return requires complete closed source profile")?;
    if !o
        .get("setCode")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
    {
        return Err("upgrade return requires valid source identity".into());
    }
    o.remove("setCode");
    expected.as_object_mut().unwrap().remove("setCode");
    if actual != expected {
        return Err("upgrade return requires complete closed source profile".into());
    }
    Ok(true)
}
fn contains_return(v: &Value) -> bool {
    let mut todo = vec![v];
    let mut n = 0;
    while let Some(v) = todo.pop() {
        n += 1;
        if n > 65536 {
            return true;
        }
        if v["kind"] == "returnToHand" {
            return true;
        }
        match v {
            Value::Array(a) => todo.extend(a),
            Value::Object(o) => todo.extend(o.values()),
            _ => {}
        }
    }
    false
}
pub(crate) fn compiled() -> Result<Program, String> {
    let mut a: Ability = serde_json::from_value(profile()["abilities"]["card"][0].clone())
        .map_err(|e| e.to_string())?;
    let mut effect = a.effect.clone();
    if let Effect::Simultaneous { effects } = &mut effect {
        for e in effects {
            if let Effect::Select {
                target_resolver, ..
            } = e
            {
                target_resolver.prompt_title = None;
            }
        }
    }
    a.effect = Effect::NoAction;
    let mut p = Program::compile(&a)?;
    p.instructions = vec![Instruction::Group(effect)];
    p.event_end = 1;
    p.select_ends.clear();
    p.executable = crate::program_backend(&serde_json::to_vec(&p).map_err(|e| e.to_string())?);
    Ok(p)
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions.iter().any(|i| {
        i.group_effect().is_some_and(|e| {
            serde_json::to_value(e)
                .ok()
                .is_some_and(|v| contains_return(&v))
        })
    })
}
pub(crate) fn program(p: &Program) -> bool {
    compiled()
        .ok()
        .is_some_and(|x| serde_json::to_value(p).ok() == serde_json::to_value(x).ok())
}

/// One registration plus State's stored-program+1 sentinel must both fit.
pub(crate) fn checked_program_id(start: usize) -> Result<u16, String> {
    let id = u16::try_from(start).map_err(|_| "upgrade return program capacity exceeded")?;
    id.checked_add(1)
        .ok_or("upgrade return stored program+1 capacity exceeded")?;
    Ok(id)
}
