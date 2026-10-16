//! Closed attachment constants, reusing the shared effective-trait inventory.
use crate::ability_ir::{Ability, Effect, Expr, Target};
use crate::admission::plain_registration;
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
pub(crate) fn marker(v:&Value)->bool { crate::ability_ir::contains(v,|v|v["kind"]=="attachedConstantContract") }
fn trait_name(v:&Value)->Option<&str> {let o=v.as_object()?;if o.len()!=2||v["kind"]!="const" {return None;}v["value"].as_str().filter(|n|!n.is_empty()&&n.len()<=64&&n.chars().all(|c|c.is_ascii_lowercase()||c=='-'||c==' '))}
fn trait_leaf(v:Value)->Result<Value,String> {
    let m:TraitModifier=serde_json::from_value(v).map_err(|e|format!("attached trait modifier: {e}"))?;
    if m.kind!="ongoingContract"||m.scope!="card"||!["gainTrait","loseTrait"].contains(&m.effect_type.as_str())||m.target!="parent"||m.duration!="persistent"||m.value_mode!="static" {return Err("attached constants support only static persistent parent trait leaves".into());}
    let name=trait_name(&m.value).ok_or("attached trait must be a bounded literal")?;
    Ok(json!({"kind":m.effect_type,"target":"parent","trait":name,"duration":"persistent"}))
}
/// Runtime source aliases are distinct from the physical grantor. The source
/// spelling differs between compiler wrappers and portable ability contexts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttachmentContext { Upgrade, Host }
impl AttachmentContext {
    fn parse(value: &str) -> Option<Self> {
        match value { "upgrade"=>Some(Self::Upgrade), "host"=>Some(Self::Host), _=>None }
    }
    pub(crate) fn matches_source_binding(self, value: &str) -> bool {
        value == match self { Self::Upgrade=>"upgrade", Self::Host=>"attachedHost" }
    }
    fn effect_target(self, selected: bool) -> &'static str {
        if selected { "chosen" } else { match self { Self::Upgrade=>"parent", Self::Host=>"self" } }
    }
    fn keyword_leaves(self, effect: &Effect, selected: bool) -> bool {
        match effect {
            Effect::GainTrait{target:Target::Name(n),trait_name,duration}
            | Effect::LoseTrait{target:Target::Name(n),trait_name,duration}=>duration=="persistent"
                && self==Self::Upgrade && !selected && n=="parent" && !trait_name.is_empty() && trait_name.len()<=64,
            Effect::GainKeyword{target:Target::Name(n),keyword,amount:Expr::Const{value},duration}=>duration=="persistent"
                && n==self.effect_target(selected) && ["grit","sentinel","restore","raid","overwhelm"].contains(&keyword.as_str())
                && value.as_u64().is_some_and(|n|(1..=2).contains(&n)),
            Effect::Simultaneous{effects}=>!effects.is_empty() && effects.len()<=8
                && effects.iter().all(|effect|self.keyword_leaves(effect,selected)),
            _=>false,
        }
    }
}
fn reference(root:&str,path:&[&str])->Value {let mut p=vec![Value::String(root.into())];p.extend(path.iter().map(|s|Value::String((*s).into())));json!({"kind":"ref","path":p})}
fn other_friendly(candidate:Value,source:Value,controller:Value,player:Value)->Value {
    json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":candidate,"right":source},"right":{"kind":"binary","op":"===","left":controller,"right":player}})
}
fn with_trait(scope:Value,candidate:Value,name:&str)->Value {
    json!({"kind":"binary","op":"&&","left":scope,"right":{"kind":"query","name":"hasSomeTrait","receiver":candidate,"args":[{"kind":"const","value":name}]}})
}
fn predicate(candidate:&str,context:&str,name:&str)->Value {
    with_trait(other_friendly(reference(candidate,&[]),reference(context,&["source"]),reference(candidate,&["controller"]),reference(context,&["player"])),reference(candidate,&[]),name)
}
fn portable_scope()->Value {
    other_friendly(reference("card",&[]),reference("source",&[]),reference("card",&["controller"]),reference("player",&[]))
}
fn portable_predicate(name:&str)->Value { with_trait(portable_scope(),reference("card",&[]),name) }
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
    let scope=crate::attached_registration::ParentScope { target:&c.target, controller:&c.target_controller,
        types:&c.target_types, zone:&c.target_zone, requires_grantor:c.requires_grantor_in_play };
    if w.trigger!="constant"||!w.costs.is_empty()||c.kind!="attachedConstantContract"||!scope.accepts()||!AttachmentContext::Upgrade.matches_source_binding(&c.gain_ability_source_binding) {return Err("attached constant requires exact parent/controller/presence contract".into());}
    match c.mode.as_str() {
        "direct" if AttachmentContext::Upgrade.matches_source_binding(&c.source_binding)&&c.program.is_none()=>{
            let leaf=trait_leaf(c.modifier.ok_or("attached direct modifier missing")?)?;
            serde_json::from_value(json!({"trigger":"constant","title":w.title,"costs":[],"effect":leaf})).map_err(|e|e.to_string())
        }
        "gainConstant" if AttachmentContext::Host.matches_source_binding(&c.source_binding)&&c.modifier.is_none()=>{
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
#[derive(Clone, Copy)]
enum AuraResolverPolicy { LegacyKeyword, Trait }
fn aura_resolver(r:&crate::ability_ir::TargetResolver,policy:AuraResolverPolicy)->bool {
    r.owner=="self" && r.card_type=="unit" && r.zone.as_deref()==Some("arena")
        && r.arena.is_none() && !r.optional && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none()
        && match policy {
            AuraResolverPolicy::LegacyKeyword=>true,
            AuraResolverPolicy::Trait=>r.native.is_none() && r.prompt_title_parts.is_empty(),
        }
}
pub(crate) fn host_trait_aura(a:&Ability)->bool {
    if a.trigger!="constant" || a.attached_host || !plain_registration(a) || a.attached_context.as_deref()!=Some("host") {return false;}
    let Some(r)=&a.target_resolver else{return false;};let Some(predicate)=&r.predicate else{return false;};let Ok(p)=serde_json::to_value(predicate) else{return false;};let Some(name)=trait_name(&p["right"]["args"][0]) else{return false;};
    aura_resolver(r,AuraResolverPolicy::Trait)&&p==portable_predicate(name)
        &&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),duration,power:Expr::Const{value:p},hp:Expr::Const{value:h}} if n=="chosen"&&duration=="persistent"&&p.as_i64().is_some_and(|n|(-1000..=1000).contains(&n))&&h.as_i64().is_some_and(|n|(-1000..=1000).contains(&n)))
}
/// Validate runtime alias capabilities after the complete source wrapper has
/// established grantor/host binding. This preserves the legacy keyword resolver
/// policy separately from the stricter trait-aura wrapper policy.
pub(crate) fn validate_context(ability:&Ability)->Result<(),String> {
    if let Some(context)=&ability.attached_context {
        let binding=AttachmentContext::parse(context);
        if binding.is_none() || ability.trigger!="constant" || ability.coordinate || ability.optional || ability.requires_confirmation || !ability.costs.is_empty() || !ability.context_title.is_empty() || !ability.target_resolvers.is_empty() || ability.then.is_some() || ability.if_you_do.is_some() {
            return Err("attached contexts require bounded constant keyword registrations".into());
        }
        let binding=binding.expect("checked attachment context");
        if !binding.keyword_leaves(&ability.effect,ability.target_resolver.is_some()) && !crate::attached_constants::host_trait_aura(ability) || (binding==AttachmentContext::Upgrade && ability.target_resolver.is_some()) {return Err("attached context supports only persistent literal keyword grants".into());}
        if binding==AttachmentContext::Host {
            if ability.condition.as_ref().is_some_and(|e|e.any(|e|match e {
                Expr::NativeTokenPredicate{..}|Expr::Compiled(_)=>Some(false),
                Expr::Ref{path}=>Some(path.get(1).is_some_and(|p|p=="parentCard"||p=="parentUnit")),
                _=>None,
            })) {return Err("delegated host condition cannot read an upgrade parent getter".into());}
            if let Some(r)=&ability.target_resolver {
                let expected=portable_scope();
                if !crate::attached_constants::host_trait_aura(ability) && (!aura_resolver(r,AuraResolverPolicy::LegacyKeyword) || r.predicate.as_ref().and_then(|p|serde_json::to_value(p).ok())!=Some(expected)) {return Err("delegated host aura requires other friendly arena units by host identity and controller".into());}
            }
        }
    }
    Ok(())
}
pub(crate) fn parent_title_play(a:&Ability)->bool {
    if a.trigger!="whenPlayed" || a.attached_host || !plain_registration(a) || a.target_resolver.is_some() || a.attached_context.is_some() {return false;}
    let Ok(v)=serde_json::to_value(&a.effect) else{return false;};let Some(title)=v["condition"]["right"]["value"].as_str().filter(|s|!s.is_empty()&&s.len()<=128) else{return false;};
    let expected:Effect=serde_json::from_value(json!({"kind":"conditional","condition":{"kind":"binary","op":"===","left":optional_parent_title(),"right":{"kind":"const","value":title}},"onTrue":{"kind":"select","targetResolver":{"owner":"self","type":"any","zone":"anyAttackable"},"effect":{"kind":"giveShield","target":"chosen","amount":{"kind":"const","value":1}}},"onFalse":{"kind":"noAction"}})).expect("portable parent-title template");
    serde_json::to_value(expected).ok()==Some(v)
}
pub(crate) fn entry(e:&Value)->bool {marker(&e["abilities"])&&crate::attached_registration::plain_upgrade(e)&&crate::attached_registration::sides(e).is_ok()}
