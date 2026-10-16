//! Bounded native ongoing contracts shared by all execution backends.
//!
//! Every supported source form lowers to a closed native leaf. Unknown wrapper
//! fields, callbacks and dynamic values reject the complete definition.
use crate::ability_ir::{Ability, Effect, Sides};
use crate::admission::{plain_registration, SourceContextPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::OnceLock;

macro_rules! literal {
    ($name:ident, $variant:ident, $value:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            #[serde(rename=$value)]
            $variant,
        }
    };
}
literal!(Persistent, Persistent, "persistent");
literal!(CardScope, Card, "card");
literal!(ConstantMatch, ConstantMatch, "constantMatch");
literal!(StaticValue, Static, "static");
literal!(SourceContext, Refresh, "refreshFromSourceController");
literal!(ScopeDefaults, Pinned, "pinnedOngoingCardOrPlayerEffect");
literal!(DependencyPolicy, Live, "liveFixedPointRequired");
literal!(AnyZone, Any, "any");
literal!(UndefinedKind, Undefined, "undefined");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelfEffect {
    CannotAttack,
    CannotAttackBase,
    EntersPlayReady,
    CanAttackBaseWhileUsingAmbush,
    ModifyStats,
    GainTrait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Undefined {
    pub kind: UndefinedKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Properties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_target: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_zone_filter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_controller: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_card_type_filter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone_filter: Option<AnyZone>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum ValueMode { Static, Dynamic }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Modifier {
    OngoingContract {
        scope: CardScope,
        #[serde(rename = "effectType")]
        effect_type: SelfEffect,
        target: ConstantMatch,
        duration: Persistent,
        #[serde(rename = "valueMode")]
        value_mode: ValueMode,
        value: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        builder: Option<StaticValue>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Registration {
    pub properties: Properties,
    pub modifier: Modifier,
    pub duration: Persistent,
    pub context: SourceContext,
    pub scope_defaults: ScopeDefaults,
    pub dependency_policy: DependencyPolicy,
    #[serde(skip)]
    native: OnceLock<Result<NativeModifier, String>>,
}
impl Registration {
    pub fn effect_type(&self) -> SelfEffect {
        let Modifier::OngoingContract { effect_type, .. } = &self.modifier;
        *effect_type
    }
    pub fn validate(&self) -> Result<(), String> { self.native().map(|_| ()) }
    pub(crate) fn native(&self) -> Result<&NativeModifier, String> {
        self.native.get_or_init(|| self.lower()).as_ref().map_err(Clone::clone)
    }
    fn lower(&self) -> Result<NativeModifier, String> {
        let Modifier::OngoingContract { builder, value_mode, value, .. } = &self.modifier;
        let props = &self.properties;
        let own = props.target_controller.is_none() && props.target_card_type_filter.is_none() && props.target_zone_filter.is_none() && props.match_target.is_none();
        let condition = props.condition.as_ref().map(condition).transpose()?;
        let undefined = *value == json!({"kind":"undefined"});
        match self.effect_type() {
            SelfEffect::EntersPlayReady if own && props.source_zone_filter == Some(AnyZone::Any)
                && *builder == Some(StaticValue::Static) && *value_mode == ValueMode::Static && undefined =>
                    Ok(NativeModifier::SelfBoolean { kind:self.effect_type(), condition }),
            SelfEffect::CannotAttack | SelfEffect::CannotAttackBase if own && props.source_zone_filter.is_none()
                && builder.is_none() && *value_mode == ValueMode::Static && undefined && condition.is_none() =>
                    Ok(NativeModifier::SelfBoolean { kind:self.effect_type(), condition }),
            SelfEffect::CanAttackBaseWhileUsingAmbush if props.target_controller.as_deref() == Some("self")
                && props.target_card_type_filter.as_deref() == Some("unit") && props.source_zone_filter.is_none()
                && condition.is_none() && props.target_zone_filter.is_none() && props.match_target.is_none() && builder.is_none() && *value_mode == ValueMode::Static && undefined =>
                    Ok(NativeModifier::FriendlyAmbushBase),
            SelfEffect::GainTrait if condition.is_none() && builder.is_none() => {
                if own && props.source_zone_filter==Some(AnyZone::Any) && *value_mode==ValueMode::Dynamic {
                    return leader_trait_union(value).map(|excluded|NativeModifier::Traits(TraitRule::LeaderUnion{excluded}));
                }
                if props.source_zone_filter.is_none() && props.target_controller.as_deref()==Some("self") && props.target_card_type_filter.as_deref()==Some("unit") && props.target_zone_filter.as_deref()==Some("any") && *value_mode==ValueMode::Static {
                    if let (Some(predicate),Some(gained))=(&props.match_target,value["value"].as_str().filter(|t|!t.is_empty()&&t.len()<=64)) {
                        if *value==json!({"kind":"const","value":gained}) {
                            let required=unit_trait_aura(predicate)?;
                            if required!=gained {return Ok(NativeModifier::Traits(TraitRule::UnitAura{required,gained:gained.into()}));}
                        }
                    }
                }
                Err("unsupported trait gain scope, literal or dependency".into())
            },
            SelfEffect::ModifyStats if own && props.source_zone_filter.is_none() && condition.is_none()
                && builder.is_none() && *value_mode == ValueMode::Dynamic => stat_count(value).map(NativeModifier::StatCount),
            _ => Err("ongoing contract requires its reviewed scope, value and source-zone defaults".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NativeModifier {
    SelfBoolean { kind:SelfEffect, condition:Option<Condition> },
    FriendlyAmbushBase,
    StatCount(Count),
    Traits(TraitRule),
}
#[derive(Debug, Clone)]
pub(crate) enum Condition {
    RoundEquals(u16),
    ControlledTitle(String),
    FriendlyNonUnique,
    OpponentArenaEmpty(&'static str),
}
#[derive(Debug, Clone)]
pub(crate) enum Count { OtherFriendlySameTitle, DiscardUnitsWithTrait(String) }
#[derive(Debug,Clone)]
pub(crate) enum TraitRule { UnitAura{required:String,gained:String}, LeaderUnion{excluded:String} }
impl TraitRule {
    pub(crate) fn may_produce(&self,requested:Option<&str>)->bool {match self {
        Self::UnitAura{gained,..}=>requested.is_none_or(|t|t==gained),
        Self::LeaderUnion{excluded}=>requested.is_none_or(|t|t!=excluded),
    }}
}

fn name(v:&Value)->Result<&str,String> {
    v.as_str().filter(|s|!s.is_empty() && s.len()<=128).ok_or_else(||"invalid ongoing lexical binding".into())
}
fn reference(path:&[&str])->Value {json!({"kind":"ref","path":path})}
fn query(name:&str,receiver:Value,args:Vec<Value>)->Value {
    json!({"kind":"ongoingQuery","name":name,"receiver":receiver,"args":args,"optionalReceiver":false,"optionalCall":false,"continuesOptionalChain":false})
}
fn callback(root:&str,body:Value)->Value {json!({"kind":"attackCallback","parameters":[{"root":root,"role":"ongoingContext"}],"body":body})}
fn condition(v:&Value)->Result<Condition,String> {
    let root=name(&v["parameters"][0]["root"])?;
    let body=&v["body"];
    let number=body["right"]["value"].as_u64().filter(|n|*n<=u16::MAX as u64);
    if let Some(n)=number {
        let expected=callback(root,json!({"kind":"binary","op":"===","left":reference(&[root,"game","roundNumber"]),"right":{"kind":"const","value":n}}));
        if *v==expected {return Ok(Condition::RoundEquals(n as u16));}
    }
    if let Some(title)=body["args"][0]["value"].as_str().filter(|s|!s.is_empty()&&s.len()<=128) {
        let expected=callback(root,query("controlsLeaderUnitOrUpgradeWithTitle",reference(&[root,"player"]),vec![json!({"kind":"const","value":title})]));
        if *v==expected {return Ok(Condition::ControlledTitle(title.into()));}
    }
    for (arena,label) in [("groundArena","ground"),("spaceArena","space")] {
        let expected=callback(root,json!({"kind":"binary","op":"===","left":{"kind":"ongoingMember","receiver":query("getArenaUnits",reference(&[root,"player","opponent"]),vec![json!({"kind":"record","properties":{"arena":{"kind":"const","value":arena}}})]),"member":"length","optionalReceiver":false,"continuesOptionalChain":false},"right":{"kind":"const","value":0}}));
        if *v==expected {return Ok(Condition::OpponentArenaEmpty(label));}
    }
    let candidate=&body["args"][0];
    if let (Ok(slot),Ok(binding))=(name(&candidate["parameters"][0]["slot"]),name(&candidate["parameters"][0]["bindingPattern"]["name"])) {
        let expected=callback(root,json!({"kind":"ongoingCollection","name":"some","receiver":query("getArenaUnits",reference(&[root,"player"]),vec![]),"args":[pure_callback(slot,"candidate",binding,json!({"kind":"pureReturn","value":{"kind":"unary","op":"!","value":data_path(slot,&["unique"])}}))],"optionalReceiver":false,"optionalCall":false,"continuesOptionalChain":false}));
        if *v==expected {return Ok(Condition::FriendlyNonUnique);}
    }
    Err("unsupported ongoing condition or callback scope".into())
}
fn data_path(root:&str,steps:&[&str])->Value {
    json!({"kind":"dataPath","root":[root],"steps":steps.iter().map(|s|json!({"key":s,"optional":false})).collect::<Vec<_>>()})
}
fn pure_callback(slot:&str,argument:&str,binding:&str,body:Value)->Value {
    let parameter=PureBinding {slot,name:binding}.parameter(argument);
    json!({"kind":"pureCallback","parameters":[parameter],"body":body,"capture":"lexicalReferences","invokeAt":"nativeCallbackInvocation"})
}
/// A lexical slot and its source spelling form one callback binding. Parsing
/// names does not relax the complete wrapper comparisons at admission.
#[derive(Clone, Copy)]
struct PureBinding<'a> { slot: &'a str, name: &'a str }
impl<'a> PureBinding<'a> {
    fn read(value: &'a Value) -> Result<Self, String> {
        Ok(Self { slot: name(&value["slot"])?, name: name(&value["bindingPattern"]["name"])? })
    }
    fn parameter(self, argument: &str) -> Value {
        json!({"slot":self.slot,"argument":argument,"bindingPattern":{"name":self.name}})
    }
    fn callback(self, argument: &str, body: Value) -> Value {
        pure_callback(self.slot, argument, self.name, body)
    }
}
fn live_value(callback: Value) -> Value {
    json!({"kind":"ongoingValueFactory","callback":callback})
}
fn length(receiver: Value) -> Value {
    json!({"kind":"ongoingMember","receiver":receiver,"member":"length","optionalReceiver":false,"continuesOptionalChain":false})
}
fn returned(value: Value) -> Value { json!({"kind":"pureReturn","value":value}) }
fn power_only(power: Value) -> Value {
    json!({"kind":"dataRecord","fields":{"power":power,"hp":{"kind":"const","value":0}}})
}
fn stat_count(v:&Value)->Result<Count,String> {
    let c=&v["callback"];
    let target_binding=PureBinding::read(&c["parameters"][0])?;
    let target=target_binding.slot;
    // This is a pure local declaration evaluated at each native invocation; it
    // cannot capture a value from registration or an earlier callback.
    let declaration=&c["body"]["statements"][0]["bindings"][0];
    if let Ok(local_binding)=PureBinding::read(declaration) {
        let PureBinding {slot:local,name:local_name}=local_binding;
        let call=&declaration["value"]["receiver"];
        let candidate=&call["args"][0]["properties"]["condition"]["callback"];
        if let Ok(candidate_binding)=PureBinding::read(&candidate["parameters"][0]) {
            let slot=candidate_binding.slot;
            if slot!=target && slot!=local && local!=target {
                let equality=json!({"kind":"binary","op":"===","left":data_path(slot,&["name"]),"right":data_path(target,&["name"])});
                let count=length(query("getArenaUnits",data_path(target,&["controller"]),vec![json!({"kind":"record","properties":{"condition":live_value(candidate_binding.callback("candidate",returned(equality))),"otherThan":data_path(target,&[])}})]));
                let body=json!({"kind":"pureBlock","statements":[{"kind":"pureBindings","bindings":[{"slot":local,"mutable":false,"value":count,"bindingPattern":{"name":local_name},"evaluateAt":"declaration"}]},returned(power_only(data_path(local,&[])))]});
                if *v==live_value(target_binding.callback("targetCard",body)) {return Ok(Count::OtherFriendlySameTitle);}
            }
        }
    }
    let count=&c["body"]["value"]["fields"]["power"];
    let candidate=&count["receiver"]["args"][0];
    if let Ok(candidate_binding)=PureBinding::read(&candidate["parameters"][0]) {
        let slot=candidate_binding.slot;
        if slot!=target {
            if let Some(trait_name)=candidate["body"]["value"]["right"]["args"][0]["value"].as_str().filter(|s|!s.is_empty()&&s.len()<=64) {
                let predicate=json!({"kind":"binary","op":"&&","left":query("isUnit",data_path(slot,&[]),vec![]),"right":{"kind":"dataCall","method":"hasSomeTrait","receiver":data_path(slot,&[]),"args":[{"kind":"const","value":trait_name}],"optionalCall":false,"optionalReceiver":false}});
                let count=length(json!({"kind":"pureCollectionCall","method":"filter","receiver":data_path(target,&["controller","discardZone","cards"]),"args":[candidate_binding.callback("item",returned(predicate))],"optionalReceiver":false,"optionalCall":false}));
                let body=returned(power_only(count));
                if *v==live_value(target_binding.callback("targetCard",body)) {return Ok(Count::DiscardUnitsWithTrait(trait_name.into()));}
            }
        }
    }
    Err("unsupported live ongoing value or lexical capture".into())
}

fn unit_trait_aura(v:&Value)->Result<String,String> {
    let candidate=name(&v["parameters"][0]["root"])?;
    let context=name(&v["parameters"][1]["root"])?;
    if candidate==context {return Err("trait aura bindings must be distinct".into());}
    let required=name(&v["body"]["left"]["right"]["args"][0]["value"])?;
    if required.len()>64 {return Err("trait literal exceeds capacity".into());}
    let in_play=json!({"kind":"query","name":"isInPlay","receiver":reference(&[candidate]),"args":[]});
    let expected=json!({"kind":"attackCallback","parameters":[{"root":candidate,"role":"candidate"},{"root":context,"role":"ongoingContext"}],"body":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":query("canBeInPlay",reference(&[candidate]),vec![]),"right":{"kind":"query","name":"hasSomeTrait","receiver":reference(&[candidate]),"args":[{"kind":"const","value":required}]}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"&&","left":in_play,"right":{"kind":"binary","op":"===","left":reference(&[candidate,"controller"]),"right":reference(&[context,"player"])}},"right":{"kind":"binary","op":"&&","left":{"kind":"unary","op":"!","value":in_play},"right":{"kind":"binary","op":"===","left":reference(&[candidate,"owner"]),"right":reference(&[context,"player"])}}}}});
    if *v==expected {Ok(required.into())}else{Err("trait aura requires complete live in-play controller and nonplay owner predicate".into())}
}
fn leader_trait_union(v:&Value)->Result<String,String> {
    let c=&v["callback"];
    let target_binding=PureBinding::read(&c["parameters"][0])?;
    let target=target_binding.slot;
    let context_binding=PureBinding::read(&c["parameters"][1])?;
    let context=context_binding.slot;
    let statements=&c["body"]["statements"];
    let declaration=&statements[0]["bindings"][0];
    let local_binding=PureBinding::read(&declaration)?;
    let PureBinding {slot:local,name:local_name}=local_binding;
    let leader_loop=&statements[1];
    let leader_binding=PureBinding::read(&leader_loop)?;
    let PureBinding {slot:leader,name:leader_name}=leader_binding;
    let trait_loop=&leader_loop["body"]["statements"][0];
    let item_binding=PureBinding::read(&trait_loop)?;
    let PureBinding {slot:item,name:item_name}=item_binding;
    let excluded=name(&trait_loop["body"]["statements"][0]["condition"]["right"]["value"])?;
    if excluded.len()>64 || [target,context,local,leader,item].into_iter().collect::<std::collections::BTreeSet<_>>().len()!=5 {return Err("trait union bindings or exclusion invalid".into());}
    let trait_body=json!({"kind":"pureBlock","statements":[{"kind":"pureIf","condition":{"kind":"binary","op":"!==","left":data_path(item,&[]),"right":{"kind":"const","value":excluded}},"then":{"kind":"pureBlock","statements":[{"kind":"pureEvaluate","value":{"kind":"pureSetMutation","method":"add","receiver":data_path(local,&[]),"value":data_path(item,&[])}}]}}]});
    let outer=json!({"kind":"pureForOf","slot":leader,"bindingPattern":{"name":leader_name},"iterable":query("getLeaderCards",data_path(context,&["player"]),vec![]),"iterationScope":"perIteration","body":{"kind":"pureBlock","statements":[{"kind":"pureForOf","slot":item,"bindingPattern":{"name":item_name},"iterable":data_path(leader,&["traits"]),"iterationScope":"perIteration","body":trait_body}]}});
    let body=json!({"kind":"pureBlock","statements":[{"kind":"pureBindings","bindings":[{"slot":local,"mutable":false,"value":{"kind":"pureSet","implementation":"Set","values":{"kind":"array","items":[]}},"bindingPattern":{"name":local_name},"evaluateAt":"declaration"}]},outer,{"kind":"pureReturn","value":{"kind":"arrayFrom","value":data_path(local,&[])}}]});
    let mut expected=target_binding.callback("targetCard",body);
    expected["parameters"].as_array_mut().unwrap().push(context_binding.parameter("ongoingContext"));
    if *v==live_value(expected) {Ok(excluded.into())}else{Err("trait union requires live friendly leaders, bounded set insertion and literal exclusion".into())}
}
pub(crate) fn trait_dependencies(e:&Value)->Vec<(String,String)> {
    if e["status"]!="supported" || !e["abilities"]["card"].as_array().is_some_and(|a|a.iter().any(|a|a["effect"]["kind"]=="constantEffectRegistration" && a["effect"]["modifier"]["effectType"]=="gainTrait")) || !entry(e) {return vec![];}
    let Ok(sides)=serde_json::from_value::<Sides>(e["abilities"].clone()) else{return vec![];};
    sides.card.iter().filter_map(|a|match &a.effect {
        Effect::ConstantEffectRegistration(r)=>match r.native(){Ok(NativeModifier::Traits(TraitRule::UnitAura{required,gained}))=>Some((required.clone(),gained.clone())),_=>None},_=>None,
    }).collect()
}

pub(crate) fn marker(v: &Value) -> bool {
    v["kind"] == "constantEffectRegistration"
        || match v {
            Value::Object(o) => o.values().any(marker),
            Value::Array(a) => a.iter().any(marker),
            _ => false,
        }
}
pub(crate) fn registration(a: &Ability) -> bool {
    let Effect::ConstantEffectRegistration(r) = &a.effect else {
        return false;
    };
    a.trigger == "constant"
        && plain_registration(a) && a.target_resolver.is_none() && a.attached_context.is_none()
        && r.validate().is_ok()
}
pub(crate) fn has_leader_trait_union(p:&crate::ability_ir::Program)->bool {
    p.instructions.iter().any(|i|matches!(i,crate::ability_ir::Instruction::Apply(Effect::ConstantEffectRegistration(r)) if matches!(r.native(),Ok(NativeModifier::Traits(TraitRule::LeaderUnion{..})))))
}
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(sides) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    SourceContextPolicy::Closed.accepts(e)
        && e["baseClass"] == "NonLeaderUnitCard"
        && e["card"]["types"] == serde_json::json!(["unit"])
        && e["card"]["unique"].is_boolean()
        && ["cost", "hp", "power"]
            .iter()
            .all(|f| e["card"][*f].as_u64().is_some_and(|n| n <= 1000))
        && e["card"]["hp"].as_u64().is_some_and(|n| n > 0)
        && ["ground", "space"].iter().any(|a| e["card"]["arena"] == *a)
        && !sides.card.is_empty()
        && sides.card.len() <= 8
        && sides.unit.is_empty()
        && sides.leader.is_empty()
        && sides.card.iter().all(registration)
        && !(e["keywordValues"].get("support").is_some() && sides.card.iter().any(|a|matches!(&a.effect,Effect::ConstantEffectRegistration(r) if matches!(r.native(),Ok(NativeModifier::Traits(TraitRule::LeaderUnion{..}))))))
        && { let mut seen=std::collections::BTreeSet::new(); sides.card.iter().all(|a|seen.insert(serde_json::to_string(&a.effect).expect("typed ongoing serializes"))) }
        && e["piloting"].is_null()
        && e["attachCondition"].is_null()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
        && (e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
}
