//! Closed attachment constants, reusing the shared effective-trait inventory.
use crate::ability_ir::{Ability, Effect, Expr, Target};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Wrapper {trigger:String,title:String,costs:Vec<Value>,effect:Contract}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Contract {
    kind:String,target:String,mode:String,target_controller:String,target_types:Vec<String>,target_zone:String,
    requires_grantor_in_play:bool,source_binding:String,gain_ability_source_binding:String,
    #[serde(default)]modifier:Option<Value>,#[serde(default)]program:Option<Value>,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct TraitModifier {kind:String,scope:String,effect_type:String,target:String,duration:String,value_mode:String,value:Value}
pub(crate) fn marker(v:&Value)->bool {v["kind"]=="attachedConstantContract"||match v {Value::Object(o)=>o.values().any(marker),Value::Array(a)=>a.iter().any(marker),_=>false}}
fn trait_name(v:&Value)->Option<&str> {let o=v.as_object()?;if o.len()!=2||v["kind"]!="const" {return None;}v["value"].as_str().filter(|n|!n.is_empty()&&n.len()<=64&&n.chars().all(|c|c.is_ascii_lowercase()||c=='-'||c==' '))}
fn trait_leaf(v:Value)->Result<Value,String> {
    let m:TraitModifier=serde_json::from_value(v).map_err(|e|format!("attached trait modifier: {e}"))?;
    if m.kind!="ongoingContract"||m.scope!="card"||!["gainTrait","loseTrait"].contains(&m.effect_type.as_str())||m.target!="parent"||m.duration!="persistent"||m.value_mode!="static" {return Err("attached constants support only static persistent parent trait leaves".into());}
    let name=trait_name(&m.value).ok_or("attached trait must be a bounded literal")?;
    Ok(json!({"kind":m.effect_type,"target":"parent","trait":name,"duration":"persistent"}))
}
fn reference(root:&str,path:&[&str])->Value {let mut p=vec![Value::String(root.into())];p.extend(path.iter().map(|s|Value::String((*s).into())));json!({"kind":"ref","path":p})}
fn predicate(candidate:&str,context:&str,name:&str)->Value {
    json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":reference(candidate,&[]),"right":reference(context,&["source"])},"right":{"kind":"binary","op":"===","left":reference(candidate,&["controller"]),"right":reference(context,&["player"])}},"right":{"kind":"query","name":"hasSomeTrait","receiver":reference(candidate,&[]),"args":[{"kind":"const","value":name}]}})
}
fn portable_predicate(name:&str)->Value {
    json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":reference("card",&[]),"right":reference("source",&[])},"right":{"kind":"binary","op":"===","left":reference("card",&["controller"]),"right":reference("player",&[])}},"right":{"kind":"query","name":"hasSomeTrait","receiver":reference("card",&[]),"args":[{"kind":"const","value":name}]}})
}
fn host_aura(p:&Value)->Result<Ability,String> {
    let name=trait_name(&p["effect"]["properties"]["matchTarget"]["body"]["right"]["args"][0]).ok_or("host aura requires literal trait predicate")?;
    let cb=&p["effect"]["properties"]["matchTarget"];
    let candidate=cb["parameters"][0]["root"].as_str().ok_or("host aura candidate missing")?;
    let context=cb["parameters"][1]["root"].as_str().ok_or("host aura context missing")?;
    if candidate==context||candidate.is_empty()||context.is_empty()||candidate.len()>128||context.len()>128 {return Err("host aura requires separate bounded callback parameters".into());}
    let props=&p["effect"]["modifier"]["value"]["properties"];
    let power=props["power"]["value"].as_i64().filter(|n|(-1000..=1000).contains(n)).ok_or("host aura bounded power missing")?;
    let hp=props["hp"]["value"].as_i64().filter(|n|(-1000..=1000).contains(n)).ok_or("host aura bounded HP missing")?;
    let expected=json!({"trigger":"constant","title":p["title"],"costs":[],"effect":{"kind":"constantEffectRegistration","properties":{"matchTarget":{"kind":"attackCallback","parameters":[{"root":candidate,"role":"candidate"},{"root":context,"role":"ongoingContext"}],"body":predicate(candidate,context,name)}},"modifier":{"kind":"ongoingContract","scope":"card","effectType":"modifyStats","target":"constantMatch","duration":"persistent","valueMode":"static","value":{"kind":"record","properties":{"power":{"kind":"const","value":power},"hp":{"kind":"const","value":hp}}}},"duration":"persistent","context":"refreshFromSourceController","scopeDefaults":"pinnedOngoingCardOrPlayerEffect","dependencyPolicy":"liveFixedPointRequired"}});
    if p!=&expected {return Err("host aura requires complete other-friendly-trait callback and static stats contract".into());}
    serde_json::from_value(json!({"trigger":"constant","title":p["title"],"costs":[],"attachedContext":"host","targetResolver":{"owner":"self","type":"unit","zone":"arena","predicate":portable_predicate(name)},"effect":{"kind":"modifyStats","target":"chosen","duration":"persistent","power":{"kind":"const","value":power},"hp":{"kind":"const","value":hp}}})).map_err(|e|e.to_string())
}
pub(crate) fn ability(v:&Value)->Result<Ability,String> {
    let w:Wrapper=serde_json::from_value(v.clone()).map_err(|e|format!("attached constant: {e}"))?;let c=w.effect;
    if w.trigger!="constant"||!w.costs.is_empty()||c.kind!="attachedConstantContract"||c.target!="parent"||c.target_controller!="any"||c.target_types!=["unit","base"]||c.target_zone!="anyAttackable"||!c.requires_grantor_in_play||c.gain_ability_source_binding!="upgrade" {return Err("attached constant requires exact parent/controller/presence contract".into());}
    match c.mode.as_str() {
        "direct" if c.source_binding=="upgrade"&&c.program.is_none()=>{
            let leaf=trait_leaf(c.modifier.ok_or("attached direct modifier missing")?)?;
            serde_json::from_value(json!({"trigger":"constant","title":w.title,"costs":[],"effect":leaf})).map_err(|e|e.to_string())
        }
        "gainConstant" if c.source_binding=="attachedHost"&&c.modifier.is_none()=>{
            let p=c.program.ok_or("attached host program missing")?;if p["title"]!=w.title {return Err("attached host title mismatch".into());}host_aura(&p)
        }
        _=>Err("attached constant mode/source binding is unsupported".into())
    }
}
fn optional_parent_title()->Value {json!({"kind":"conditional","condition":{"kind":"query","name":"isInPlay","receiver":reference("source",&[]),"args":[]},"then":reference("source",&["parentCard","title"]),"else":{"kind":"const","value":null}})}
pub(crate) fn companion(v:&Value)->Result<Ability,String> {
    let mut v=v.clone();
    let getter=json!({"kind":"memberPath","receiver":reference("source",&["parentCard"]),"steps":[{"property":"title","optional":true}]});
    if v["effect"]["condition"]["left"]==getter {
        v["effect"]["condition"]["left"]=optional_parent_title();
        let a:Ability=serde_json::from_value(v).map_err(|e|e.to_string())?;
        if !parent_title_play(&a) {return Err("optional parent title requires bounded play-time conditional Shield selection".into());}return Ok(a);
    }
    serde_json::from_value(v).map_err(|e|format!("companion registration: {e}"))
}
fn plain(a:&Ability)->bool {a.trigger=="constant"&&!a.optional&&!a.requires_confirmation&&!a.coordinate&&!a.attached_host&&a.gained_host.is_none()&&a.costs.is_empty()&&a.context_title.is_empty()&&a.condition.is_none()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()}
pub(crate) fn host_trait_aura(a:&Ability)->bool {
    if !plain(a)||a.attached_context.as_deref()!=Some("host") {return false;}
    let Some(r)=&a.target_resolver else{return false;};let Some(predicate)=&r.predicate else{return false;};let Ok(p)=serde_json::to_value(predicate) else{return false;};let Some(name)=trait_name(&p["right"]["args"][0]) else{return false;};
    r.owner=="self"&&r.card_type=="unit"&&r.zone.as_deref()==Some("arena")&&r.arena.is_none()&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.native.is_none()&&r.prompt_title.is_none()&&r.prompt_title_parts.is_empty()&&p==portable_predicate(name)
        &&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),duration,power:Expr::Const{value:p},hp:Expr::Const{value:h}} if n=="chosen"&&duration=="persistent"&&p.as_i64().is_some_and(|n|(-1000..=1000).contains(&n))&&h.as_i64().is_some_and(|n|(-1000..=1000).contains(&n)))
}
pub(crate) fn parent_title_play(a:&Ability)->bool {
    if a.trigger!="whenPlayed"||a.optional||a.requires_confirmation||a.coordinate||a.attached_host||a.gained_host.is_some()||a.attached_context.is_some()||!a.costs.is_empty()||!a.context_title.is_empty()||a.condition.is_some()||a.target_resolver.is_some()||!a.target_resolvers.is_empty()||a.then.is_some()||a.if_you_do.is_some() {return false;}
    let Ok(v)=serde_json::to_value(&a.effect) else{return false;};let Some(title)=v["condition"]["right"]["value"].as_str().filter(|s|!s.is_empty()&&s.len()<=128) else{return false;};
    let expected:Effect=serde_json::from_value(json!({"kind":"conditional","condition":{"kind":"binary","op":"===","left":optional_parent_title(),"right":{"kind":"const","value":title}},"onTrue":{"kind":"select","targetResolver":{"owner":"self","type":"any","zone":"anyAttackable"},"effect":{"kind":"giveShield","target":"chosen","amount":{"kind":"const","value":1}}},"onFalse":{"kind":"noAction"}})).expect("portable parent-title template");
    serde_json::to_value(expected).ok()==Some(v)
}
pub(crate) fn entry(e:&Value)->bool {marker(&e["abilities"])&&crate::attached_registration::plain_upgrade(e)&&crate::attached_registration::sides(e).is_ok()}
