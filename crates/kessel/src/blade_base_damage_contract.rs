//! Closed source projection for one own-base damage event -> one self Advantage.
//! This module validates data only. It does not admit a card or supply an event
//! producer; shared admission must also require the reviewed native token pool.
use crate::ability_ir::{Ability,Program};
use serde_json::{json,Value};

const MAX_SCAN_DEPTH:usize=128;
const MAX_SCAN_NODES:usize=65_536;
const SCAN_BUDGET_REASON:&str="own-base damage source inspection budget exceeded";
/// Raw callers can supply Values deeper/wider than the JSON parser accepts.
/// Count every scheduled child before extending the worklist, and keep scanning
/// after a match so a marker cannot bypass validation of its siblings' budget.
pub(crate) fn marker(root:&Value)->Result<bool,String> {
    let mut remaining=vec![(root,0usize)];
    let mut scheduled=1usize;
    let mut found=false;
    while let Some((v,depth))=remaining.pop() {
        if depth>MAX_SCAN_DEPTH {return Err(SCAN_BUDGET_REASON.into());}
        found|=v["event"]=="onDamageDealt"
            || v["text"]=="When your base is dealt damage: Give an Advantage token to this unit.";
        let children=match v {Value::Object(o)=>o.len(),Value::Array(a)=>a.len(),_=>0};
        scheduled=scheduled.checked_add(children).filter(|n|*n<=MAX_SCAN_NODES)
            .ok_or_else(||SCAN_BUDGET_REASON.to_string())?;
        if children>0 && depth==MAX_SCAN_DEPTH {return Err(SCAN_BUDGET_REASON.into());}
        match v {
            Value::Object(o)=>remaining.extend(o.values().map(|v|(v,depth+1))),
            Value::Array(a)=>remaining.extend(a.iter().map(|v|(v,depth+1))),
            _=>{},
        }
    }
    Ok(found)
}
fn binding(v:&Value)->Option<&str> {
    let id=v.as_str()?;let mut parts=id.split(':');
    (parts.next()==Some("callback") && parts.next().is_some_and(|n|!n.is_empty() && n.bytes().all(|c|c.is_ascii_digit()) && n.parse::<u32>().is_ok())
        && parts.next()==Some("0") && parts.next().is_none() && id.len()<=128).then_some(id)
}
fn ability(id:&str,title:&str)->Value {
    json!({"trigger":"eventHooks","title":title,"costs":[],"hooks":[{"event":"onDamageDealt","predicate":{
        "kind":"binary","op":"===","left":{"kind":"dataPath","root":["event"],
        "argumentBinding":{"id":id,"bindAt":"callbackInvocation","readAt":"expressionEvaluation"},
        "steps":[{"key":"card","optional":false}]},"right":{"kind":"ref","path":["player","base"]}}}],
        "effect":{"kind":"giveAdvantage","target":"self","amount":{"kind":"const","value":1},
        "createdBy":"self","batch":"singleCreationEvent","tokenOwner":"recipientController"}})
}
fn raw_registration(v:&Value)->bool {
    let Some(id)=binding(&v["hooks"][0]["predicate"]["left"]["argumentBinding"]["id"]) else {return false;};
    let Some(title)=v["title"].as_str().filter(|s|!s.is_empty() && s.len()<=512) else {return false;};
    *v==ability(id,title)
}
pub(crate) fn entry(e:&Value)->bool {
    if e["status"]!="supported" || e["baseClass"]!="NonLeaderUnitCard" || !crate::admission::closed_source_helpers(e)
        || !e["abilities"].as_object().is_some_and(|o|o.len()==3)
        || e["abilities"]["leader"]!=json!([]) || e["abilities"]["unit"]!=json!([])
        || !e["abilities"]["card"].as_array().is_some_and(|a|a.len()==1 && raw_registration(&a[0]))
        || e["keywordValues"]!=json!({}) {return false;}
    let profile=json!({"types":["unit"],"cost":3,"hp":4,"power":2,"arena":"space","unique":true,
        "aspects":["cunning","heroism"],"traits":["rebel","vehicle","fighter"],"keywords":[],
        "text":"When your base is dealt damage: Give an Advantage token to this unit.",
        "deployBox":null,"epicAction":"","rules":null,"upgradePower":null,"upgradeHp":null});
    profile.as_object().unwrap().iter().all(|(k,v)|e["card"][k]==*v)
        && ["piloting","attachCondition","stateWatchers"].iter().all(|k|e[*k].is_null())
        && ["pilotText","pilotBox","pilotingBox","deployCost"].iter().all(|k|e["card"][*k].is_null())
}
/// Standalone typed registrations must prove every header and leaf themselves.
/// Raw fallback Hook values are accepted ONLY by this exact closed projection;
/// there is no callback interpreter and no permissive event-name registration.
pub(crate) fn registration(a:&Ability)->bool {
    if a.trigger!="eventHooks" || !crate::admission::plain_registration(a)
        || a.target_resolver.is_some() || a.attached_context.is_some() || a.attached_host
        || a.piloting_ability || a.pilot_unit_ability || a.attacker_must_survive || a.if_you_do_condition.is_some() {return false;}
    let hooks=serde_json::to_value(&a.hooks).expect("typed hooks serialize");
    let Some(id)=binding(&hooks[0]["predicate"]["left"]["argumentBinding"]["id"]) else {return false;};
    let Ok(expected)=serde_json::from_value::<Ability>(ability(id,&a.title)) else {return false;};
    !a.title.is_empty() && a.title.len()<=512
        && serde_json::to_value(a).ok()==serde_json::to_value(expected).ok()
}
/// Local executable projection, not a source-admission shortcut. The unchanged
/// scalar compiler owns GiveAdvantage. Retain the exact hook and source binding
/// on the resulting program for the dedicated capture owner to check.
pub(crate) fn compile_registration(a:&Ability)->Result<Program,String> {
    if !registration(a) {return Err("own-base damage requires the complete mandatory source-only Advantage hook".into());}
    let mut scalar=a.clone();scalar.trigger="whenPlayed".into();scalar.hooks.clear();
    let mut program=Program::compile(&scalar)?;program.trigger=a.trigger.clone();program.hooks=a.hooks.clone();
    // Restoring source headers changes canonical program bytes. Rebind dispatch
    // to those final bytes instead of retaining the intermediate scalar hash.
    program.executable=crate::program_backend(&serde_json::to_vec(&program).map_err(|e|e.to_string())?);
    Ok(program)
}
pub(crate) fn compile_entry(e:&Value)->Result<Program,String> {
    if !entry(e) {return Err("own-base damage source profile is incomplete or changed".into());}
    let a=serde_json::from_value::<Ability>(e["abilities"]["card"][0].clone()).map_err(|e|e.to_string())?;
    compile_registration(&a)
}
pub(crate) fn program(p:&Program)->bool {
    let hooks=serde_json::to_value(&p.hooks).expect("typed hooks serialize");
    let Some(id)=binding(&hooks[0]["predicate"]["left"]["argumentBinding"]["id"]) else {return false;};
    let Ok(a)=serde_json::from_value::<Ability>(ability(id,&p.title)) else {return false;};
    compile_registration(&a).is_ok_and(|expected|serde_json::to_value(p).ok()==serde_json::to_value(expected).ok())
}
pub(crate) fn compiled_source(card:&Value,programs:&[&Program])->bool {
    if programs.len()!=1 || !program(programs[0]) {return false;}
    let hooks=serde_json::to_value(&programs[0].hooks).expect("typed hooks serialize");
    let Some(id)=binding(&hooks[0]["predicate"]["left"]["argumentBinding"]["id"]) else {return false;};
    entry(&json!({"status":"supported","baseClass":"NonLeaderUnitCard","card":card,"keywordValues":{},
        "abilities":{"leader":[],"unit":[],"card":[ability(id,&programs[0].title)]}}))
}
