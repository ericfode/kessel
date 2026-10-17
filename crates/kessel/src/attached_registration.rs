//! Strict ordinary-upgrade aliases. The registration is bound to the physical
//! host before an event; captured triggers retain that source/controller after
//! either host or grantor leaves play. This is not a persistent action inventory.
//!
//! Reviewed NonLeader Pilot units may use the same host-grant path with
//! `registrationMode: "piloting"` (Paige / Tam): the program still installs on
//! the physical host while the pilot is attached.
use crate::ability_ir::{Ability, Sides};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Wrapper { trigger:String, title:String, costs:Vec<Value>, effect:Grant }
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Grant {
    kind:String, registration_mode:String, target:String, target_controller:String,
    source_binding:String, controller_binding:String, requires_grantor_in_play:bool,
    requires_attachable:bool, source_mode:String, target_zone:String,
    target_types:Vec<String>, registration:Registration,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Registration { r#type:String, program:Value }

fn marker(v:&Value)->bool {
    v["kind"]=="attachedAbilityRegistration" || match v {
        Value::Object(o)=>o.values().any(marker),Value::Array(a)=>a.iter().any(marker),_=>false
    }
}
fn internal(v:&Value)->bool {match v {
    Value::Object(o)=>o.contains_key("attachedHost")||o.values().any(internal),
    Value::Array(a)=>a.iter().any(internal),_=>false
}}
fn mention_keywords(e:&Value)->bool {
    let Some(names)=e["card"]["keywords"].as_array() else{return false;};
    names.iter().all(|name|name.as_str().is_some_and(|n|["raid","restore","grit"].contains(&n))&&e["abilities"]["card"].as_array().is_some_and(|a|a.iter().any(|a|
        a["trigger"]=="constant"&&a["effect"]["kind"]=="gainKeyword"&&a["effect"]["target"]=="parent"
        &&a["effect"]["keyword"]==*name&&a["effect"]["duration"]=="persistent"
        &&a["effect"]["amount"]["value"].as_u64().is_some_and(|n|(1..=1000).contains(&n)))))
}
/// Engine-only card-text validation: dual On Attack / When Defeated host grants
/// trip forceteki's When Defeated source check; gameplay is unaffected.
fn defeat_text_override(e:&Value)->bool {
    e["setupValidationOverrides"]==json!([{
        "kind":"cardTextValidationOverride",
        "property":"disableWhenDefeatedCheck",
        "value":true,
        "applyAt":"orderedSourceSetupStatement",
        "default":false,
        "consumer":"InPlayCard.validateCardAbilities",
        "gameplayEffect":"none",
        "setupMethod":"setupCardAbilities",
        "statementIndex":0
    }])
}
fn empty_context(e:&Value,key:&str)->bool {
    e[key].is_null()||e[key]==json!([])
}
// The pending compiler preserves the zone object getter; the portable evaluator
// exposes its name directly. Only physical card roots receive this equivalence.
fn normalize_zone_names(v:&mut Value) {
    if v["kind"]=="ref"&&v["path"].as_array().is_some_and(|p|p.len()==3&&["source","card","target"].iter().any(|r|p[0]==*r)&&p[1]=="zone"&&p[2]=="name") {
        let root=v["path"][0].clone();v["path"]=json!([root,"zoneName"]);
    }
    // Arena equality often compares zone objects; portable IR uses zoneName.
    if v["kind"]=="ref"&&v["path"].as_array().is_some_and(|p|p.len()==2&&["source","card","target"].iter().any(|r|p[0]==*r)&&p[1]=="zone") {
        let root=v["path"][0].clone();v["path"]=json!([root,"zoneName"]);
    }
    match v {Value::Object(o)=>for child in o.values_mut(){normalize_zone_names(child)},Value::Array(a)=>for child in a {normalize_zone_names(child)},_=>{}}
}
pub(crate) fn plain_upgrade(e:&Value)->bool {
    let setup_ok=empty_context(e,"setupValidationOverrides")||defeat_text_override(e);
    if ["stateWatchers","historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride"].iter().any(|k|!empty_context(e,k))||!setup_ok {return false;}
    e["baseClass"]=="UpgradeCard"&&e["card"]["types"]==json!(["upgrade"])
    && e["card"]["unique"].is_boolean() && e["keywordValues"]==json!({})
    && mention_keywords(e) && e["piloting"].is_null()
    && ["pilotText","pilotBox","pilotingBox","deployBox","deployCost"].iter().all(|k|e["card"][*k].is_null())
    && (e["card"]["epicAction"].is_null()||e["card"]["epicAction"]=="")
    && [("power","upgradePower"),("hp","upgradeHp")].iter().all(|(a,b)|e["card"][*a]==e["card"][*b]&&e["card"][*a].as_i64().is_some_and(|n|(-1000..=1000).contains(&n)))
    && e["abilities"]["leader"]==json!([])&&e["abilities"]["unit"]==json!([])
}
/// Reviewed NonLeader Pilot that grants a host ability while attached via Piloting.
fn plain_pilot(e:&Value)->bool {
    if ["stateWatchers","historyHelpers","privatePrograms","classMethods","privateClassConstants","inheritedSource","deploymentOverride","setupValidationOverrides"].iter().any(|k|!empty_context(e,k)) {return false;}
    if e["baseClass"]!="NonLeaderUnitCard"||e["card"]["types"]!=json!(["unit"])
        ||!e["card"]["unique"].is_boolean()||!e["attachCondition"].is_null()||!e["piloting"].is_null()
        ||e["card"]["pilotText"].as_str().is_none_or(|t|t.trim().is_empty()||t.len()>512)
        ||["pilotBox","pilotingBox","deployBox","deployCost"].iter().any(|k|!e["card"][*k].is_null())
        ||!(e["card"]["epicAction"].is_null()||e["card"]["epicAction"]=="")
        ||e["abilities"]["leader"]!=json!([])||e["abilities"]["unit"]!=json!([])
        ||!e["card"]["arena"].as_str().is_some_and(|a|a=="ground"||a=="space")
        ||!e["card"]["traits"].as_array().is_some_and(|t|t.iter().any(|v|v=="pilot"))
        ||e["keywordValues"]!=json!({})
    {return false;}
    let Some(names)=e["card"]["keywords"].as_array() else{return false;};
    // Piloting is a play-mode mention; other printed keywords need a separate grant path.
    if names!=&[json!("piloting")] {return false;}
    for field in ["power","hp","upgradePower","upgradeHp"] {
        if !e["card"][field].as_i64().is_some_and(|n|(0..=1000).contains(&n)) {return false;}
    }
    true
}
fn source_meta(e:&Value)->bool {plain_upgrade(e)||plain_pilot(e)}
fn invalid_host_scope(v:&Value)->bool {
    v["target"]=="parent"||(v["kind"]=="ref"&&v["path"][0]=="source"&&matches!(v["path"][1].as_str(),Some("parentCard"|"parentUnit")))
        ||match v {Value::Object(o)=>o.values().any(invalid_host_scope),Value::Array(a)=>a.iter().any(invalid_host_scope),_=>false}
}
fn dual_attack_defeat_hooks(hooks:&Value)->bool {
    *hooks==json!([{"alias":"onAttack","enabled":true},{"alias":"whenDefeated","enabled":true}])
}
fn single_alias_hook(hooks:&Value)->Option<&'static str> {
    for alias in ["onAttack","onDefense","whenDefeated"] {
        if *hooks==json!([{"alias":alias,"enabled":true}]) {return Some(alias);}
    }
    None
}
fn compile_host_program(mut p:Value,title:&str)->Result<Ability,String> {
    if invalid_host_scope(&p) {return Err("attached host program cannot read grantor/parent context".into());}
    if p["title"]!=title {return Err("attached registration requires one exact attack/defense/defeat alias".into());}
    p.as_object_mut().ok_or("attached program must be an object")?.remove("hooks");
    normalize_zone_names(&mut p);
    let mut a:Ability=serde_json::from_value(p).map_err(|e|format!("attached program: {e}"))?;
    if a.gained_host.is_some()||a.attached_context.is_some()||a.coordinate||!a.costs.is_empty()
        ||!a.target_resolvers.is_empty()||a.attacker_must_survive {
        return Err("attached alias cannot reuse private host/side/payment contexts".into());
    }
    a.attached_host=true;
    Ok(a)
}
fn upgrade_grant(g:&Grant)->bool {
    g.registration_mode=="upgrade"&&g.target=="parent"&&g.target_controller=="any"
        &&g.source_binding=="attachedHost"&&g.controller_binding=="hostController"
        &&g.requires_grantor_in_play&&!g.requires_attachable&&g.source_mode=="upgrade"
        &&g.target_zone=="anyAttackable"&&g.target_types==["unit","base"]&&g.registration.r#type=="triggered"
}
fn piloting_grant(g:&Grant)->bool {
    g.registration_mode=="piloting"&&g.target=="parent"&&g.target_controller=="any"
        &&g.source_binding=="attachedHost"&&g.controller_binding=="hostController"
        &&g.requires_grantor_in_play&&g.requires_attachable&&g.source_mode=="unitUpgrade"
        &&g.target_zone=="anyArena"&&g.target_types==["unit"]&&g.registration.r#type=="triggered"
}
fn programs(v:&Value,entry:&Value)->Result<Vec<Ability>,String> {
    let w:Wrapper=serde_json::from_value(v.clone()).map_err(|e|format!("attached registration: {e}"))?;
    let g=w.effect;
    if w.trigger!="constant"||!w.costs.is_empty()||g.kind!="attachedAbilityRegistration"
        ||!(upgrade_grant(&g)||piloting_grant(&g)) {
        return Err("attached registration requires exact ordinary upgrade/host/controller contract".into());
    }
    if piloting_grant(&g)&&!plain_pilot(entry) {
        return Err("piloting attached registration requires complete ordinary Pilot unit metadata".into());
    }
    if upgrade_grant(&g)&&!plain_upgrade(entry) {
        return Err("attached registration requires exact ordinary upgrade/host/controller contract".into());
    }
    let mut p=g.registration.program;
    let mut trigger=p["trigger"].as_str().ok_or("attached registration missing trigger")?.to_owned();
    let hooks=p["hooks"].clone();
    if trigger=="eventHooks"&&dual_attack_defeat_hooks(&hooks) {
        if !defeat_text_override(entry) {
            return Err("dual attack/defeat attached aliases require reviewed defeat-text validation override".into());
        }
        let mut out=Vec::with_capacity(2);
        for alias in ["onAttack","whenDefeated"] {
            let mut one=p.clone();
            one["trigger"]=json!(alias);
            one["hooks"]=json!([{"alias":alias,"enabled":true}]);
            out.push(compile_host_program(one,&w.title)?);
        }
        return Ok(out);
    }
    if defeat_text_override(entry) {
        return Err("defeat-text validation override requires dual onAttack/whenDefeated attached aliases".into());
    }
    if trigger=="eventHooks" {
        let Some(alias)=single_alias_hook(&hooks) else {
            return Err("attached registration requires one exact attack/defense/defeat alias".into());
        };
        p["trigger"]=json!(alias);
        trigger=alias.to_owned();
    }
    if !["onAttack","onDefense","whenDefeated"].contains(&trigger.as_str())
        ||hooks!=json!([{"alias":trigger,"enabled":true}]) {
        return Err("attached registration requires one exact attack/defense/defeat alias".into());
    }
    Ok(vec![compile_host_program(p,&w.title)?])
}
pub(crate) fn sides(entry:&Value)->Result<Sides,String> {
    let raw=&entry["abilities"];
    if internal(raw) {return Err("attachedHost is an internal program marker".into());}
    if !marker(raw)&&!crate::attached_constants::marker(raw) {return serde_json::from_value(raw.clone()).map_err(|e|format!("invalid ability IR: {e}"));}
    if !raw.as_object().is_some_and(|o|o.len()==3&&["leader","unit","card"].iter().all(|k|o.contains_key(*k)))||!source_meta(entry) {
        return Err("attached aliases require complete ordinary upgrade metadata and matching printed stats".into());
    }
    let card=raw["card"].as_array().ok_or("attached aliases need card-side registrations")?;
    if card.is_empty()||card.len()>8 {return Err("attached registration count exceeds bound".into());}
    let mut out=Sides::default();
    for a in card {
        if marker(a) {
            out.card.extend(programs(a,entry)?);
        } else if crate::attached_constants::marker(a) {
            if plain_pilot(entry) {
                return Err("piloting attached registration does not admit companion constants".into());
            }
            out.card.push(crate::attached_constants::ability(a)?);
        } else {
            if plain_pilot(entry) {
                return Err("piloting attached registration does not admit companion registrations".into());
            }
            out.card.push(crate::attached_constants::companion(a)?);
        }
    }
    if out.card.len()>8 {return Err("attached registration count exceeds bound".into());}
    Ok(out)
}
/// Exact reviewed Pilot + piloting-mode host registration contract.
pub(crate) fn piloting_host_entry(entry:&Value)->bool {
    plain_pilot(entry)
        && entry["abilities"]["card"].as_array().is_some_and(|card|!card.is_empty()&&card.iter().all(marker))
        && sides(entry).is_ok()
}
