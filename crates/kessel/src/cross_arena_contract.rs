//! Closed registration envelope for persistent, self-only arena permissions.
//! This module validates data only; source identity never drives combat behavior.
use serde_json::{json,Value};
pub const GROUND: &str = "canAttackGroundArenaFromSpaceArena";
pub const SPACE: &str = "canAttackSpaceArenaFromGroundArena";
pub fn effect(permission:&str)->Value {
    json!({"kind":"attackPermission","permission":permission,"target":"self","duration":"persistent"})
}
pub fn marker(v:&Value)->bool {
    let mut pending=vec![(v,0usize)];let mut budget=2048usize;
    while let Some((v,depth))=pending.pop() {
        if budget==0 || depth>24 {return true;} budget-=1;
        if v["kind"]=="attackPermission" {return true;}
        match v {Value::Object(o)=>pending.extend(o.values().map(|v|(v,depth+1))),Value::Array(a)=>pending.extend(a.iter().map(|v|(v,depth+1))),_=>{}}
        if pending.len()>2048 {return true;}
    } false
}
pub fn entry(e:&Value)->bool {
    if e["status"]!="supported" || e["baseClass"]!="NonLeaderUnitCard"
        || !crate::admission::closed_source_helpers(e) {return false;}
    let mut a=e["abilities"]["card"].clone();
    let Some(title)=a.as_array_mut().filter(|a|a.len()==1).and_then(|a|a[0].as_object_mut()).and_then(|o|o.remove("title")) else {return false;};
    if !title.as_str().is_some_and(|s| !s.is_empty() && s.len()<=512)
        || a!=json!([{"trigger":"constant","costs":[],"effect":{"kind":"simultaneous","effects":[effect(GROUND),effect(SPACE)]}}])
        || !e["abilities"].as_object().is_some_and(|o|o.len()==3)
        || e["abilities"]["leader"]!=json!([]) || e["abilities"]["unit"]!=json!([])
        || e["keywordValues"]!=json!({"support":1}) {return false;}
    // Complete printed rules profile includes intrinsic Support, never an
    // invented keyword-free donor. Names, IDs and set numbers are not semantics.
    let profile=json!({"types":["unit"],"arena":"space","cost":7,"hp":6,"power":6,
        "keywords":["support"],"traits":["rebel","vehicle","fighter"],
        "aspects":["command","cunning","heroism"],"unique":true,
        "text":"Support (When you play this unit, you may attack with another unit. It gains this unit's other abilities for this attack.)\nThis unit may attack units in either arena.",
        "rules":null,"deployBox":null,"epicAction":"","upgradePower":null,"upgradeHp":null});
    if !profile.as_object().unwrap().iter().all(|(k,v)|e["card"][k]==*v) {return false;}
    ["piloting","attachCondition","stateWatchers"].iter().all(|k|e[*k].is_null())
        && ["pilotText","pilotBox","pilotingBox","deployCost"].iter().all(|k|e["card"][*k].is_null())
}

/// Check the complete admitted definition's retained metadata and canonical
/// compiled registration set. Catalogue loading has already checked the raw
/// source envelope with `entry`; this guard independently prevents a program
/// reference from selecting a partial, appended or modified registration set.
/// No runtime state, numeric program selection or card identity is accepted.
pub(crate) fn source_programs(card:&Value, programs:&[&crate::ability_ir::Program])->bool {
    use crate::ability_ir::{Ability,Program};
    let ability=json!({"trigger":"constant","title":"This unit can attack units in either arena","costs":[],
        "effect":{"kind":"simultaneous","effects":[effect(GROUND),effect(SPACE)]}});
    let envelope=json!({"status":"supported","baseClass":"NonLeaderUnitCard","card":card,
        "keywordValues":{"support":1},"abilities":{"leader":[],"unit":[],"card":[ability.clone()]}});
    if !entry(&envelope) || programs.len()!=1 {return false;}
    let Ok(ability)=serde_json::from_value::<Ability>(ability) else {return false;};
    let Ok(expected)=Program::compile(&ability) else {return false;};
    serde_json::to_value(programs[0]).ok()==serde_json::to_value(expected).ok()
}
