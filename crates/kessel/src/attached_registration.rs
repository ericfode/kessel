//! Strict ordinary-upgrade aliases. The registration is bound to the physical
//! host before an event; captured triggers retain that source/controller after
//! either host or grantor leaves play. This is not a persistent action inventory.
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

/// Shared physical-parent scope for triggered aliases and constant wrappers.
/// Source/controller rebinding is a separate contract: the grantor stays the
/// upgrade even when a registration executes with its host as source.
pub(crate) struct ParentScope<'a> {
    pub target: &'a str,
    pub controller: &'a str,
    pub types: &'a [String],
    pub zone: &'a str,
    pub requires_grantor: bool,
}
impl ParentScope<'_> {
    pub(crate) fn accepts(&self) -> bool {
        self.target == "parent" && self.controller == "any"
            && self.types == ["unit", "base"] && self.zone == "anyAttackable"
            && self.requires_grantor
    }
}

/// Metadata shared by whole-upgrade contracts. Legacy delegated contexts have
/// historically checked fewer printed fields; that remains an explicit policy.
#[derive(Clone, Copy)]
pub(crate) enum UpgradeMetadata { Complete, LegacyContext }
impl UpgradeMetadata {
    pub(crate) fn accepts(self, entry: &Value) -> bool {
        entry["baseClass"] == "UpgradeCard" && entry["card"]["types"] == json!(["upgrade"])
            && entry["piloting"].is_null() && entry["keywordValues"] == json!({})
            && ["pilotText", "pilotBox", "pilotingBox"].iter().all(|key|entry["card"][*key].is_null())
            && match self {
                Self::LegacyContext => true,
                Self::Complete => ["deployBox", "deployCost"].iter().all(|key|entry["card"][*key].is_null())
                    && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
                    && [("power", "upgradePower"), ("hp", "upgradeHp")].iter().all(|(printed, derived)|
                        entry["card"][*printed] == entry["card"][*derived]
                            && entry["card"][*printed].as_i64().is_some_and(|n|(-1000..=1000).contains(&n))),
            }
    }
}
pub(crate) fn complete_single_upgrade(entry: &Value, sides: &Sides) -> bool {
    UpgradeMetadata::Complete.accepts(entry) && sides.card.len() == 1
        && sides.leader.is_empty() && sides.unit.is_empty()
        && entry["card"]["keywords"] == json!([])
}
pub(crate) fn delegated_upgrade_metadata(entry: &Value, sides: &Sides) -> bool {
    UpgradeMetadata::LegacyContext.accepts(entry) && sides.card.len() == 1
        && sides.leader.is_empty() && sides.unit.is_empty()
        && entry["card"]["unique"].is_boolean()
        && entry["card"]["keywords"].as_array().is_some_and(|keywords|
            keywords.iter().all(|k|k.as_str().is_some_and(|k|crate::ability_ir::CORE_KEYWORDS.contains(&k))))
}

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
// The pending compiler preserves the zone object getter; the portable evaluator
// exposes its name directly. Only physical card roots receive this equivalence.
fn normalize_zone_names(v:&mut Value) {
    if v["kind"]=="ref"&&v["path"].as_array().is_some_and(|p|p.len()==3&&["source","card","target"].iter().any(|r|p[0]==*r)&&p[1]=="zone"&&p[2]=="name") {
        let root=v["path"][0].clone();v["path"]=json!([root,"zoneName"]);
    }
    match v {Value::Object(o)=>for child in o.values_mut(){normalize_zone_names(child)},Value::Array(a)=>for child in a {normalize_zone_names(child)},_=>{}}
}
pub(crate) fn plain_upgrade(e:&Value)->bool {
    if !crate::admission::SourceContextPolicy::Closed.accepts(e) {return false;}
    UpgradeMetadata::Complete.accepts(e)
    && e["card"]["unique"].is_boolean() && mention_keywords(e)
    && e["abilities"]["leader"]==json!([])&&e["abilities"]["unit"]==json!([])
}
fn invalid_host_scope(v:&Value)->bool {
    v["target"]=="parent"||(v["kind"]=="ref"&&v["path"][0]=="source"&&matches!(v["path"][1].as_str(),Some("parentCard"|"parentUnit")))
        ||match v {Value::Object(o)=>o.values().any(invalid_host_scope),Value::Array(a)=>a.iter().any(invalid_host_scope),_=>false}
}
fn program(v:&Value)->Result<Ability,String> {
    let w:Wrapper=serde_json::from_value(v.clone()).map_err(|e|format!("attached registration: {e}"))?;
    let g=w.effect;
    let scope=ParentScope { target:&g.target, controller:&g.target_controller,
        types:&g.target_types, zone:&g.target_zone, requires_grantor:g.requires_grantor_in_play };
    if w.trigger!="constant"||!w.costs.is_empty()||g.kind!="attachedAbilityRegistration"
        ||g.registration_mode!="upgrade"||!scope.accepts()
        ||!crate::attached_constants::AttachmentContext::Host.matches_source_binding(&g.source_binding)||g.controller_binding!="hostController"
        ||g.requires_attachable||g.source_mode!="upgrade"||g.registration.r#type!="triggered" {
        return Err("attached registration requires exact ordinary upgrade/host/controller contract".into());
    }
    let mut p=g.registration.program;
    if invalid_host_scope(&p) {return Err("attached host program cannot read grantor/parent context".into());}
    let trigger=p["trigger"].as_str().ok_or("attached registration missing trigger")?.to_owned();
    if !["onAttack","onDefense","whenDefeated"].contains(&trigger.as_str())
        ||p["title"]!=w.title||p["hooks"]!=json!([{"alias":trigger,"enabled":true}]) {
        return Err("attached registration requires one exact attack/defense/defeat alias".into());
    }
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
pub(crate) fn sides(entry:&Value)->Result<Sides,String> {
    let raw=&entry["abilities"];
    if internal(raw) {return Err("attachedHost is an internal program marker".into());}
    if !marker(raw)&&!crate::attached_constants::marker(raw) {return serde_json::from_value(raw.clone()).map_err(|e|format!("invalid ability IR: {e}"));}
    if !raw.as_object().is_some_and(|o|o.len()==3&&["leader","unit","card"].iter().all(|k|o.contains_key(*k)))||!plain_upgrade(entry) {return Err("attached aliases require complete ordinary upgrade metadata and matching printed stats".into());}
    let card=raw["card"].as_array().ok_or("attached aliases need card-side registrations")?;
    if card.is_empty()||card.len()>8 {return Err("attached registration count exceeds bound".into());}
    let mut out=Sides::default();
    for a in card {
        out.card.push(if marker(a) {program(a)?}else if crate::attached_constants::marker(a) {crate::attached_constants::ability(a)?}else{crate::attached_constants::companion(a)?});
    }
    Ok(out)
}
