//! Bounded native ongoing contracts shared by all execution backends.
//!
//! Every supported source form lowers to a closed native leaf. Unknown wrapper
//! fields, callbacks and dynamic values reject the complete definition.
use crate::ability_ir::{Ability, Effect, Sides};
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
        let own = props.target_controller.is_none() && props.target_card_type_filter.is_none();
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
                && condition.is_none() && builder.is_none() && *value_mode == ValueMode::Static && undefined =>
                    Ok(NativeModifier::FriendlyAmbushBase),
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
    json!({"kind":"pureCallback","parameters":[{"slot":slot,"argument":argument,"bindingPattern":{"name":binding}}],"body":body,"capture":"lexicalReferences","invokeAt":"nativeCallbackInvocation"})
}
fn stat_count(v:&Value)->Result<Count,String> {
    let c=&v["callback"];
    let target=name(&c["parameters"][0]["slot"])?;
    let binding=name(&c["parameters"][0]["bindingPattern"]["name"])?;
    // This is a pure local declaration evaluated at each native invocation; it
    // cannot capture a value from registration or an earlier callback.
    let declaration=&c["body"]["statements"][0]["bindings"][0];
    if let (Ok(local),Ok(local_name))=(name(&declaration["slot"]),name(&declaration["bindingPattern"]["name"])) {
        let call=&declaration["value"]["receiver"];
        let candidate=&call["args"][0]["properties"]["condition"]["callback"];
        if let (Ok(slot),Ok(candidate_name))=(name(&candidate["parameters"][0]["slot"]),name(&candidate["parameters"][0]["bindingPattern"]["name"])) {
            if slot!=target && slot!=local && local!=target {
                let equality=json!({"kind":"binary","op":"===","left":data_path(slot,&["name"]),"right":data_path(target,&["name"])});
                let count=json!({"kind":"ongoingMember","receiver":query("getArenaUnits",data_path(target,&["controller"]),vec![json!({"kind":"record","properties":{"condition":{"kind":"ongoingValueFactory","callback":pure_callback(slot,"candidate",candidate_name,json!({"kind":"pureReturn","value":equality}))},"otherThan":data_path(target,&[])}})]),"member":"length","optionalReceiver":false,"continuesOptionalChain":false});
                let body=json!({"kind":"pureBlock","statements":[{"kind":"pureBindings","bindings":[{"slot":local,"mutable":false,"value":count,"bindingPattern":{"name":local_name},"evaluateAt":"declaration"}]},{"kind":"pureReturn","value":{"kind":"dataRecord","fields":{"power":data_path(local,&[]),"hp":{"kind":"const","value":0}}}}]});
                if *v==json!({"kind":"ongoingValueFactory","callback":pure_callback(target,"targetCard",binding,body)}) {return Ok(Count::OtherFriendlySameTitle);}
            }
        }
    }
    let count=&c["body"]["value"]["fields"]["power"];
    let candidate=&count["receiver"]["args"][0];
    if let (Ok(slot),Ok(candidate_name))=(name(&candidate["parameters"][0]["slot"]),name(&candidate["parameters"][0]["bindingPattern"]["name"])) {
        if slot!=target {
            if let Some(trait_name)=candidate["body"]["value"]["right"]["args"][0]["value"].as_str().filter(|s|!s.is_empty()&&s.len()<=64) {
                let predicate=json!({"kind":"binary","op":"&&","left":query("isUnit",data_path(slot,&[]),vec![]),"right":{"kind":"dataCall","method":"hasSomeTrait","receiver":data_path(slot,&[]),"args":[{"kind":"const","value":trait_name}],"optionalCall":false,"optionalReceiver":false}});
                let count=json!({"kind":"ongoingMember","receiver":{"kind":"pureCollectionCall","method":"filter","receiver":data_path(target,&["controller","discardZone","cards"]),"args":[pure_callback(slot,"item",candidate_name,json!({"kind":"pureReturn","value":predicate}))],"optionalReceiver":false,"optionalCall":false},"member":"length","optionalReceiver":false,"continuesOptionalChain":false});
                let body=json!({"kind":"pureReturn","value":{"kind":"dataRecord","fields":{"power":count,"hp":{"kind":"const","value":0}}}});
                if *v==json!({"kind":"ongoingValueFactory","callback":pure_callback(target,"targetCard",binding,body)}) {return Ok(Count::DiscardUnitsWithTrait(trait_name.into()));}
            }
        }
    }
    Err("unsupported live ongoing value or lexical capture".into())
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
        && !a.optional
        && !a.requires_confirmation
        && !a.coordinate
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && a.costs.is_empty()
        && a.context_title.is_empty()
        && a.target_resolver.is_none()
        && a.target_resolvers.is_empty()
        && a.condition.is_none()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && r.validate().is_ok()
}
pub(crate) fn entry(e: &Value) -> bool {
    let Ok(sides) = serde_json::from_value::<Sides>(e["abilities"].clone()) else {
        return false;
    };
    e["baseClass"] == "NonLeaderUnitCard"
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
