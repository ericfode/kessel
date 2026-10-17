//! Strict ordinary-upgrade aliases. The registration is bound to the physical
//! host before an event; captured triggers retain that source/controller after
//! either host or grantor leaves play. This is not a persistent action inventory.
//!
//! Reviewed NonLeader Pilot units may use the same host-grant path with
//! `registrationMode: "piloting"` (Paige / Tam): the program still installs on
//! the physical host while the pilot is attached.
//!
//! `pilotingRegistration` / `addPilotingAbility` (Astromech / Victor Two / Backstabber /
//! Boba Fett / Mandalorian / Iden / Frisk / BB-8) keeps the program on the pilot and fires only
//! while attached. Reviewed shapes are WhenPlayed, or self-attach `onUpgradeAttached`
//! (`event.upgradeCard === source`) normalized to the same whenPlayed-while-attached path
//! (Iden parent Shield). Optional payResources→ifYouDo ready (BB-8) lowers the compiler
//! `registrationSystem` into `PayResources` then a trait-gated Ready. Host On Attack
//! `distributeHealingAmong` (Trace) lowers into portable `distributeHealing` with
//! `maxTargets` equal to the literal amount (bounded multi-target heal). Reviewed Pilots may also
//! carry one unit-side WhenPlayed companion that fires only while unattached (Mandalorian
//! exhaust up to 2 ground).
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target};
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

fn host_marker(v:&Value)->bool {
    v["kind"]=="attachedAbilityRegistration" || match v {
        Value::Object(o)=>o.values().any(host_marker),Value::Array(a)=>a.iter().any(host_marker),_=>false
    }
}
fn self_marker(v:&Value)->bool {
    v["kind"]=="pilotingRegistration" || match v {
        Value::Object(o)=>o.values().any(self_marker),Value::Array(a)=>a.iter().any(self_marker),_=>false
    }
}
fn marker(v:&Value)->bool { host_marker(v)||self_marker(v) }
fn internal(v:&Value)->bool {match v {
    Value::Object(o)=>o.contains_key("attachedHost")||o.values().any(internal),
    Value::Array(a)=>a.iter().any(internal),_=>false
}}
/// Smuggle is a resource-zone play cost mention (like Piloting's play mode), not a
/// host keyword grant. Require the same printed cost line the engine loader parses.
fn smuggle_text_ok(text:&str)->bool {
    let mut found=false;
    for (idx,_) in text.match_indices("Smuggle") {
        if idx>0 && !text[..idx].ends_with('\n') {continue;}
        let rest=text[idx+"Smuggle".len()..].trim_start();
        let Some(rest)=rest.strip_prefix('[') else {continue;};
        let Some((body,_))=rest.split_once(']') else {return false;};
        let body=body.trim();
        let Some((amount_text,aspects_text))=body.split_once("resources").map(|(a,b)|(a.trim(),b.trim())) else {return false;};
        if amount_text.parse::<u16>().ok().is_none_or(|n|n>1000) {return false;}
        let aspects_text=aspects_text.trim().trim_start_matches(',').trim();
        if aspects_text.is_empty()||aspects_text.contains(',') {return false;}
        if found {return false;}
        found=true;
    }
    found
}
fn mention_keywords(e:&Value)->bool {
    let Some(names)=e["card"]["keywords"].as_array() else{return false;};
    names.iter().all(|name|{
        let Some(n)=name.as_str() else {return false;};
        if n=="smuggle" {return smuggle_text_ok(e["card"]["text"].as_str().unwrap_or(""));}
        ["raid","restore","grit"].contains(&n)&&e["abilities"]["card"].as_array().is_some_and(|a|a.iter().any(|a|
            a["trigger"]=="constant"&&a["effect"]["kind"]=="gainKeyword"&&a["effect"]["target"]=="parent"
            &&a["effect"]["keyword"]==*name&&a["effect"]["duration"]=="persistent"
            &&a["effect"]["amount"]["value"].as_u64().is_some_and(|n|(1..=1000).contains(&n))))
    })
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
    {return false;}
    let Some(names)=e["card"]["keywords"].as_array() else{return false;};
    // Piloting is a play-mode mention. Shielded may appear as an intrinsic unit keyword
    // (Boba Fett / Iden); other printed keywords need a separate grant path.
    if !names.iter().any(|n|n=="piloting")
        || names.len()>2
        || !names.iter().all(|n|matches!(n.as_str(),Some("piloting"|"shielded")))
    {
        return false;
    }
    let expected_kv=if names.iter().any(|n|n=="shielded") {json!({"shielded":1})} else {json!({})};
    if e["keywordValues"]!=expected_kv {return false;}
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
    lower_distribute_healing_system(&mut p)?;
    let mut a:Ability=serde_json::from_value(p).map_err(|e|format!("attached program: {e}"))?;
    if a.gained_host.is_some()||a.attached_context.is_some()||a.coordinate||!a.costs.is_empty()
        ||!a.target_resolvers.is_empty()||a.attacker_must_survive {
        return Err("attached alias cannot reuse private host/side/payment contexts".into());
    }
    a.attached_host=true;
    Ok(a)
}
/// Host-trait gate used by Boba Fett's WhenPlayed damage amount / menu text.
fn parent_trait_query(e:&Expr,trait_name:&str)->bool {
    matches!(e,Expr::Query{name,receiver,args} if name=="hasSomeTrait"
        && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["source","parentCard"])
        && args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str()==Some(trait_name)))
}
fn parent_trait_amount(e:&Expr,trait_name:&str,high:u64,low:u64)->bool {
    matches!(e,Expr::Conditional{condition,then,r#else}
        if parent_trait_query(condition,trait_name)
            && matches!(then.as_ref(),Expr::Const{value} if value.as_u64()==Some(high))
            && matches!(r#else.as_ref(),Expr::Const{value} if value.as_u64()==Some(low)))
}
/// Menu / context fragments: static text plus one parent-trait amount (number or digit string).
pub(crate) fn parent_trait_scaled_title_parts(parts:&[Expr],trait_name:&str,high:u64,low:u64)->bool {
    if parts.len()!=3 {return false;}
    let amount_fragment=|e:&Expr|->bool {
        match e {
            Expr::Conditional{condition,then,r#else} if parent_trait_query(condition,trait_name) => {
                let digit=|v:&Value,n:u64| {
                    let label=match n {1=>Some("1"),2=>Some("2"),3=>Some("3"),_=>None};
                    v.as_u64()==Some(n) || label.is_some_and(|s|v.as_str()==Some(s))
                };
                matches!(then.as_ref(),Expr::Const{value} if digit(value,high))
                    && matches!(r#else.as_ref(),Expr::Const{value} if digit(value,low))
            }
            _ => false,
        }
    };
    matches!(&parts[0],Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64))
        && amount_fragment(&parts[1])
        && matches!(&parts[2],Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64))
}
fn piloting_self_titles_ok(a:&Ability)->bool {
    a.context_title.is_empty()
        || parent_trait_scaled_title_parts(&a.context_title,"transport",2,1)
}
/// Exact self-attach watcher: this upgrade is the one that just attached.
fn self_upgrade_attached_hooks(hooks:&Value)->bool {
    *hooks==json!([{
        "event":"onUpgradeAttached",
        "predicate":{
            "kind":"binary","op":"===",
            "left":{"kind":"ref","path":["event","upgradeCard"]},
            "right":{"kind":"ref","path":["source"]}
        }
    }])
}
fn parent_card_ref(e:&Expr)->bool {
    matches!(e,Expr::Ref{path} if path==&["source","parentCard"])
}
/// WhenPlayed / self-attach body on the pilot while attached; may read `source.parentCard`
/// for host filters or (Iden) give Shield directly to the host.
/// Piloting-registration lowering wraps zero-arg card guards as `registrationQuery`.
/// Reviewed `hasCost` on a card candidate is the same portable `query` the runtime evaluates.
fn normalize_registration_has_cost(v:&mut Value) {
    if v["kind"]=="registrationQuery"
        && v["name"]=="hasCost"
        && v["args"]==json!([])
        && v["optionalReceiver"]==false
        && v["optionalCall"]==false
        && v["receiver"]==json!({"kind":"ref","path":["card"]})
    {
        *v=json!({
            "kind":"query",
            "name":"hasCost",
            "receiver":{"kind":"ref","path":["card"]},
            "args":[]
        });
        return;
    }
    match v {
        Value::Object(o)=>for child in o.values_mut(){normalize_registration_has_cost(child)},
        Value::Array(a)=>for child in a {normalize_registration_has_cost(child)},
        _=>{}
    }
}
/// Strip compiler-only display fields from a program or nested continuation.
fn strip_display_fields(v:&mut Value) {
    if let Some(obj)=v.as_object_mut() {
        obj.remove("titleParts");
        if let Some(child)=obj.get_mut("ifYouDo") {strip_display_fields(child);}
        if let Some(child)=obj.get_mut("then") {strip_display_fields(child);}
    }
}
/// Lower reviewed `registrationSystem: payResources` into portable `PayResources`.
fn lower_pay_resources_system(v:&mut Value)->Result<(),String> {
    if v["kind"]=="registrationSystem" && v["system"]=="payResources" {
        let amount_ok=v["properties"]["amount"]["kind"]=="const"
            && v["properties"]["amount"]["value"].as_u64().is_some_and(|n|(1..=3).contains(&n));
        let shape_ok=v["defaults"]=="pinnedSystem"
            && v["contextTarget"]=="self"
            && v["properties"]["target"]==json!({"kind":"ref","path":["player"]})
            && v["properties"].as_object().is_some_and(|p|p.len()==2);
        if !amount_ok||!shape_ok {
            return Err("piloting payResources requires pinned self/player amount 1..=3".into());
        }
        let amount=v["properties"]["amount"].clone();
        *v=json!({
            "kind":"payResources",
            "amount":amount,
            "target":"ownPlayer"
        });
        return Ok(());
    }
    match v {
        Value::Object(o)=>for child in o.values_mut(){lower_pay_resources_system(child)?;},
        Value::Array(a)=>for child in a {lower_pay_resources_system(child)?;},
        _=>{}
    }
    Ok(())
}
/// Exact Trace host-grant shape: heal a literal 1..=2 across any units, choose-none
/// allowed, full amount required, unit-only `cardCondition`.
fn unit_card_condition(v:&Value)->bool {
    v==&json!({
        "kind":"query",
        "name":"isUnit",
        "receiver":{"kind":"ref","path":["card"]},
        "args":[]
    })
}
/// Lower reviewed `registrationSystem: distributeHealingAmong` into portable
/// `distributeHealing` (maxTargets equals the literal amount).
fn lower_distribute_healing_system(v:&mut Value)->Result<(),String> {
    if v["kind"]=="registrationSystem" && v["system"]=="distributeHealingAmong" {
        let props=&v["properties"];
        let amount=props["amountToDistribute"]["value"].as_u64()
            .filter(|_|props["amountToDistribute"]["kind"]=="const")
            .filter(|n|(1..=2).contains(n));
        let Some(amount)=amount else {
            return Err("piloting distributeHealingAmong requires literal amount 1..=2".into());
        };
        let keys_ok=props.as_object().is_some_and(|o|{
            o.len()==5
                && ["amountToDistribute","controller","canChooseNoTargets","canDistributeLess","cardCondition"]
                    .iter().all(|k|o.contains_key(*k))
        });
        let shape_ok=v["defaults"]=="pinnedSystem"
            && v["contextTarget"]=="self"
            && props["controller"]==json!({"kind":"const","value":"any"})
            && props["canChooseNoTargets"]==json!({"kind":"const","value":true})
            && props["canDistributeLess"]==json!({"kind":"const","value":false})
            && unit_card_condition(&props["cardCondition"])
            && keys_ok;
        if !shape_ok {
            return Err("piloting distributeHealingAmong requires reviewed any-unit full-amount shape".into());
        }
        *v=json!({
            "kind":"distributeHealing",
            "amount":{"kind":"const","value":amount},
            "maxTargets":amount,
            "canChooseNoTargets":true,
            "canDistributeLess":false,
            "targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"}
        });
        return Ok(());
    }
    match v {
        Value::Object(o)=>for child in o.values_mut(){lower_distribute_healing_system(child)?;},
        Value::Array(a)=>for child in a {lower_distribute_healing_system(child)?;},
        _=>{}
    }
    Ok(())
}
fn compile_self_when_played(mut p:Value,title:&str)->Result<Ability,String> {
    if p["title"]!=title {return Err("piloting registration requires matching ability title".into());}
    // Named "parent" grants belong on attachedAbilityRegistration. Expression
    // `source.parentCard` Shield is the reviewed Iden self-attach shape.
    if parent_effect_target(&p) {
        return Err("pilotingRegistration WhenPlayed cannot grant parent-targeted effects".into());
    }
    {
        let obj=p.as_object_mut().ok_or("piloting program must be an object")?;
        obj.remove("hooks");
    }
    strip_display_fields(&mut p);
    normalize_zone_names(&mut p);
    normalize_registration_has_cost(&mut p);
    crate::discard_return::normalize(&mut p);
    lower_pay_resources_system(&mut p)?;
    let mut a:Ability=serde_json::from_value(p).map_err(|e|format!("piloting program: {e}"))?;
    let pay_ready=piloting_pay_ready(&a);
    let discard_odd=piloting_discard_odd_return(&a);
    if a.trigger!="whenPlayed"||a.gained_host.is_some()||a.attached_context.is_some()||a.coordinate
        ||!a.costs.is_empty()||!a.target_resolvers.is_empty()||a.attacker_must_survive
        ||a.then.is_some()||a.condition.is_some()
        ||(!(pay_ready||discard_odd)&&a.if_you_do.is_some())
        ||!piloting_self_titles_ok(&a)
    {
        return Err("pilotingRegistration admits only standalone WhenPlayed without costs/continuations".into());
    }
    if !piloting_self_effect_ok(&a) {
        return Err("pilotingRegistration WhenPlayed requires reviewed heal/experience/token/damage/exhaust/shield/defeat/pay-ready/discard-return shapes".into());
    }
    a.piloting_ability=true;
    Ok(a)
}
fn parent_effect_target(v:&Value)->bool {
    v["target"]=="parent" || match v {
        Value::Object(o)=>o.values().any(parent_effect_target),
        Value::Array(a)=>a.iter().any(parent_effect_target),
        _=>false,
    }
}
fn static_prompt_parts(parts:&[Expr])->bool {
    !parts.is_empty()
        && parts.len()<=32
        && parts.iter().all(|e|matches!(e,Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=128)))
        && parts.iter().map(|e|match e {Expr::Const{value}=>value.as_str().map_or(0,str::len),_=>0}).sum::<usize>()<=1024
}
fn piloting_prompt_ok(parts:&[Expr])->bool {
    parts.is_empty()
        || static_prompt_parts(parts)
        || parent_trait_scaled_title_parts(parts,"transport",2,1)
}
/// Same-arena filter used by Mandalorian: candidate zone equals the attached host arena.
fn host_arena_zone_predicate(e:&Expr)->bool {
    matches!(e,Expr::Binary{op,left,right} if op=="==="
        && matches!(left.as_ref(),Expr::Ref{path} if path==&["card","zoneName"])
        && matches!(right.as_ref(),Expr::Ref{path} if path==&["source","parentCard","zoneName"]))
}
/// Frisk: upgrade candidate has a cost and printed cost ≤ 2.
fn cheap_upgrade_predicate(e:&Expr)->bool {
    matches!(e,Expr::Binary{op,left,right} if op=="&&"
        && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="hasCost"
            && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"])
            && args.is_empty())
        && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="<="
            && matches!(left.as_ref(),Expr::Ref{path} if path==&["card","cost"])
            && matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(2))))
}
/// Unit-side WhenPlayed companion: optional exhaust up to 2 ground units.
pub(crate) fn unit_ground_exhaust_companion(v:&Value)->bool {
    if v["trigger"]!="whenPlayed"||v["costs"]!=json!([])||host_marker(v)||self_marker(v)||crate::attached_constants::marker(v) {
        return false;
    }
    let Ok(a)=serde_json::from_value::<Ability>(v.clone()) else {return false;};
    !a.optional&&!a.requires_confirmation&&!a.coordinate&&a.attached_context.is_none()
        &&a.gained_host.is_none()&&!a.attached_host&&!a.piloting_ability&&a.context_title.is_empty()
        &&a.condition.is_none()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()
        && matches!(&a.effect,Effect::Exhaust{target:Target::Name(n)} if n=="chosen")
        && a.target_resolver.as_ref().is_some_and(|r|{
            r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("arena")
                &&r.arena.as_deref()==Some("ground")&&r.optional&&r.mode.as_deref()==Some("upTo")
                &&r.predicate.is_none()&&r.prompt_title.is_none()&&r.prompt_title_parts.is_empty()
                && matches!(&r.count,Some(Expr::Const{value}) if value.as_u64()==Some(2))
        })
}
fn compile_unit_ground_exhaust(v:&Value)->Result<Ability,String> {
    if !unit_ground_exhaust_companion(v) {
        return Err("piloting unit companion requires optional exhaust up to 2 ground units".into());
    }
    let mut a:Ability=serde_json::from_value(v.clone()).map_err(|e|format!("piloting unit companion: {e}"))?;
    a.pilot_unit_ability=true;
    Ok(a)
}
/// Resistance trait gate used by BB-8's pay-then-ready continuation.
fn resistance_trait_predicate(e:&Expr)->bool {
    matches!(e,Expr::Query{name,receiver,args} if name=="hasSomeTrait"
        && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"])
        && args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str()==Some("resistance")))
}
/// Boshek: discard 2 from deck; if you do, return each discarded card with odd printed cost.
fn piloting_discard_odd_return(a:&Ability)->bool {
    let amount_two=|e:&Expr| matches!(e,Expr::Const{value} if value.as_u64()==Some(2));
    !a.optional
        && a.target_resolver.is_none()
        && a.context_title.is_empty()
        && matches!(&a.effect,Effect::DiscardDeck{target:Target::Name(n),amount}
            if n=="ownPlayer"&&amount_two(amount))
        && a.if_you_do.as_ref().is_some_and(|next|{
            next.trigger.is_empty()&&!next.optional&&!next.requires_confirmation&&!next.coordinate
                && next.costs.is_empty()&&next.context_title.is_empty()
                && next.target_resolvers.is_empty()&&next.condition.is_none()
                && next.then.is_none()&&next.if_you_do.is_none()
                && next.if_you_do_condition.is_none()
                && next.target_resolver.is_none()
                && matches!(&next.effect,Effect::ReturnToHand{target:Target::Name(n)} if n=="oddCostDiscarded")
        })
}
/// BB-8: optional pay 2 resources; if you do, ready a Resistance unit.
fn piloting_pay_ready(a:&Ability)->bool {
    let amount_two=|e:&Expr| matches!(e,Expr::Const{value} if value.as_u64()==Some(2));
    a.optional
        && a.target_resolver.is_none()
        && a.context_title.is_empty()
        && matches!(&a.effect,Effect::PayResources{target:Target::Name(n),amount}
            if n=="ownPlayer"&&amount_two(amount))
        && a.if_you_do.as_ref().is_some_and(|next|{
            next.trigger.is_empty()&&!next.optional&&!next.requires_confirmation&&!next.coordinate
                && next.costs.is_empty()&&next.context_title.is_empty()
                && next.target_resolvers.is_empty()&&next.condition.is_none()
                && next.then.is_none()&&next.if_you_do.is_none()
                && matches!(&next.effect,Effect::Ready{target:Target::Name(n)} if n=="chosen")
                && next.target_resolver.as_ref().is_some_and(|r|{
                    r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")
                        &&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()
                        &&r.prompt_title.is_none()&&piloting_prompt_ok(&r.prompt_title_parts)
                        && r.predicate.as_ref().is_some_and(resistance_trait_predicate)
                })
        })
}
fn piloting_self_effect_ok(a:&Ability)->bool {
    let amount_one=|e:&Expr| matches!(e,Expr::Const{value} if value.as_u64()==Some(1));
    let amount_two=|e:&Expr| matches!(e,Expr::Const{value} if value.as_u64()==Some(2));
    let plain_unit_resolver=|r:&crate::ability_ir::TargetResolver|{
        r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")
            &&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()
            &&r.predicate.is_none()&&r.prompt_title.is_none()
            && piloting_prompt_ok(&r.prompt_title_parts)
    };
    let host_arena_enemy_resolver=|r:&crate::ability_ir::TargetResolver|{
        r.owner=="opponent"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")
            &&!r.optional&&r.arena.is_none()&&r.count.is_none()
            && r.mode.as_deref().is_none_or(|m|m=="single")
            &&r.prompt_title.is_none()&&piloting_prompt_ok(&r.prompt_title_parts)
            && r.predicate.as_ref().is_some_and(host_arena_zone_predicate)
    };
    if piloting_pay_ready(a) {return true;}
    if piloting_discard_odd_return(a) {return true;}
    match &a.effect {
        Effect::Heal{target:Target::Name(n),amount} if n=="chosen"&&amount_two(amount)
            && a.optional
            && a.target_resolver.as_ref().is_some_and(plain_unit_resolver) => true,
        Effect::CreateTokenUnit{token_type,target:Target::Name(n),amount,enters_ready}
            if token_type=="tieFighter"&&n=="ownPlayer"&&amount_one(amount)&&!*enters_ready
            && !a.optional && a.target_resolver.is_none() => true,
        Effect::GiveExperience{target:Target::Name(n),amount} if n=="chosen"&&amount_one(amount)
            && a.optional
            && a.target_resolver.as_ref().is_some_and(|r|{
                r.owner=="any"&&matches!(r.card_type.as_str(), "any"|"unit")
                    &&r.zone.as_deref()==Some("anyAttackable")&&!r.optional&&r.arena.is_none()
                    &&r.count.is_none()&&r.mode.is_none()&&r.prompt_title.is_none()
                    && piloting_prompt_ok(&r.prompt_title_parts)
                    && r.predicate.as_ref().is_some_and(|p| matches!(p,
                        Expr::Binary{op,left,right} if op=="!=="
                            && matches!(left.as_ref(), Expr::Ref{path} if path==&["card"])
                            && matches!(right.as_ref(), Expr::Ref{path} if path==&["source","parentCard"])
                    ))
            }) => true,
        // Boba Fett: optional WhenPlayed damage 1, or 2 when the host is a Transport.
        Effect::Damage{target:Target::Name(n),amount} if n=="chosen"
            && parent_trait_amount(amount,"transport",2,1)
            && a.optional
            && parent_trait_scaled_title_parts(&a.context_title,"transport",2,1)
            && a.target_resolver.as_ref().is_some_and(plain_unit_resolver) => true,
        // Mandalorian: exhaust one enemy unit in the attached host's arena.
        Effect::Exhaust{target:Target::Name(n)} if n=="chosen"
            && !a.optional
            && a.target_resolver.as_ref().is_some_and(host_arena_enemy_resolver) => true,
        // Iden: Give Shield 1 to the attached host when this upgrade attaches.
        Effect::GiveShield{target:Target::Expression(e),amount} if amount_one(amount)
            && parent_card_ref(e)
            && !a.optional
            && a.target_resolver.is_none()
            && a.context_title.is_empty() => true,
        // Frisk: optional WhenPlayed defeat an upgrade with printed cost ≤ 2.
        Effect::Defeat{target:Target::Name(n)} if n=="chosen"
            && a.optional
            && a.target_resolver.as_ref().is_some_and(|r|{
                r.owner=="any"&&r.card_type=="upgrade"&&r.zone.as_deref()==Some("anyAttackable")
                    &&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()
                    &&r.prompt_title.is_none()&&piloting_prompt_ok(&r.prompt_title_parts)
                    && r.predicate.as_ref().is_some_and(cheap_upgrade_predicate)
            }) => true,
        _ => false,
    }
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
/// Constant host aura granted while piloting (IG-88): registration.type constant.
fn piloting_constant_grant(g:&Grant)->bool {
    g.registration_mode=="piloting"&&g.target=="parent"&&g.target_controller=="any"
        &&g.source_binding=="attachedHost"&&g.controller_binding=="hostController"
        &&g.requires_grantor_in_play&&g.requires_attachable&&g.source_mode=="unitUpgrade"
        &&g.target_zone=="anyArena"&&g.target_types==["unit"]&&g.registration.r#type=="constant"
}
fn piloting_self_grant(g:&Grant)->bool {
    g.registration_mode=="piloting"&&g.target=="self"&&g.target_controller=="any"
        &&g.source_binding=="grantor"&&g.controller_binding=="grantorController"
        &&g.requires_grantor_in_play&&g.requires_attachable&&g.source_mode=="unitUpgrade"
        &&g.target_zone=="anyArena"&&g.target_types==["unit"]&&g.registration.r#type=="triggered"
}
/// Opponent has at least one damaged arena unit (`true && card.damage > 0`).
fn enemy_damaged_condition(e:&Expr)->bool {
    matches!(e,Expr::Query{name,receiver,args} if name=="hasSomeArenaUnit"
        && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["opponent"])
        && args.len()==1
        && matches!(&args[0],Expr::Binary{op,left,right} if op=="&&"
            && matches!(left.as_ref(),Expr::Const{value} if value.as_bool()==Some(true))
            && matches!(right.as_ref(),Expr::Binary{op,left,right} if op==">"
                && matches!(left.as_ref(),Expr::Ref{path} if path==&["card","damage"])
                && matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(0)))))
}
fn self_power_buff(e:&Effect,power:i64)->bool {
    matches!(e,Effect::ModifyStats{
        target:Target::Name(n),duration,power:Expr::Const{value:p},hp:Expr::Const{value:h}
    } if n=="self"&&duration=="persistent"&&p.as_i64()==Some(power)&&h.as_i64()==Some(0))
}
fn parent_power_buff(e:&Effect,power:i64)->bool {
    matches!(e,Effect::ModifyStats{
        target:Target::Name(n),duration,power:Expr::Const{value:p},hp:Expr::Const{value:h}
    } if n=="parent"&&duration=="persistent"&&p.as_i64()==Some(power)&&h.as_i64()==Some(0))
}
/// Reviewed IG-88 unit-side mirror: same enemy-damaged +3/+0 constant on self.
pub(crate) fn enemy_damaged_companion(v:&Value)->bool {
    if v["trigger"]!="constant"||v["costs"]!=json!([])||host_marker(v)||self_marker(v) {return false;}
    let Ok(a)=serde_json::from_value::<Ability>(v.clone()) else {return false;};
    !a.optional&&!a.requires_confirmation&&!a.coordinate&&a.attached_context.is_none()
        &&a.gained_host.is_none()&&!a.attached_host&&a.context_title.is_empty()
        &&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()
        &&a.condition.as_ref().is_some_and(enemy_damaged_condition)
        &&self_power_buff(&a.effect,3)
}
fn enemy_damaged_parent_stats(a:&Ability)->bool {
    a.trigger=="constant"&&a.condition.as_ref().is_some_and(enemy_damaged_condition)&&parent_power_buff(&a.effect,3)
}
/// Lower host-gained constant stats onto the pilot as a parent-targeted grant.
fn compile_host_constant_stats(mut p:Value,title:&str)->Result<Ability,String> {
    if invalid_host_scope(&p) {return Err("attached host program cannot read grantor/parent context".into());}
    if p["title"]!=title {return Err("attached registration requires matching ability title".into());}
    if let Some(hooks)=p.get("hooks") {
        if !hooks.is_null()&&hooks!=&json!([]) {
            return Err("piloting constant host grant does not use event hooks".into());
        }
    }
    p.as_object_mut().ok_or("attached program must be an object")?.remove("hooks");
    normalize_zone_names(&mut p);
    let mut a:Ability=serde_json::from_value(p).map_err(|e|format!("attached program: {e}"))?;
    if a.trigger!="constant"||a.optional||a.requires_confirmation||a.coordinate
        ||a.gained_host.is_some()||a.attached_context.is_some()||!a.costs.is_empty()
        ||!a.context_title.is_empty()||!a.target_resolvers.is_empty()||a.attacker_must_survive
        ||a.then.is_some()||a.if_you_do.is_some()||a.target_resolver.is_some()
        ||!a.condition.as_ref().is_some_and(enemy_damaged_condition)
        ||!self_power_buff(&a.effect,3)
    {
        return Err("piloting constant host requires reviewed enemy-damaged +3/+0 shape".into());
    }
    // Host-gain IR uses target self (ability lives on the host). Rewrite to parent so
    // the pilot's constant registration buffs the attached unit while attached.
    if let Effect::ModifyStats{target,..}=&mut a.effect {
        *target=Target::Name("parent".into());
    }
    Ok(a)
}
fn host_programs(v:&Value,entry:&Value)->Result<Vec<Ability>,String> {
    let w:Wrapper=serde_json::from_value(v.clone()).map_err(|e|format!("attached registration: {e}"))?;
    let g=w.effect;
    if w.trigger!="constant"||!w.costs.is_empty()||g.kind!="attachedAbilityRegistration"
        ||!(upgrade_grant(&g)||piloting_grant(&g)||piloting_constant_grant(&g)) {
        return Err("attached registration requires exact ordinary upgrade/host/controller contract".into());
    }
    if (piloting_grant(&g)||piloting_constant_grant(&g))&&!plain_pilot(entry) {
        return Err("piloting attached registration requires complete ordinary Pilot unit metadata".into());
    }
    if upgrade_grant(&g)&&!plain_upgrade(entry) {
        return Err("attached registration requires exact ordinary upgrade/host/controller contract".into());
    }
    if piloting_constant_grant(&g) {
        if defeat_text_override(entry) {
            return Err("piloting constant host grant does not use defeat-text validation overrides".into());
        }
        return Ok(vec![compile_host_constant_stats(g.registration.program,&w.title)?]);
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
fn self_programs(v:&Value,entry:&Value)->Result<Vec<Ability>,String> {
    let w:Wrapper=serde_json::from_value(v.clone()).map_err(|e|format!("piloting registration: {e}"))?;
    let g=w.effect;
    if w.trigger!="constant"||!w.costs.is_empty()||g.kind!="pilotingRegistration"||!piloting_self_grant(&g) {
        return Err("pilotingRegistration requires exact Pilot self/whenPlayed contract".into());
    }
    if !plain_pilot(entry) {
        return Err("pilotingRegistration requires complete ordinary Pilot unit metadata".into());
    }
    if defeat_text_override(entry) {
        return Err("pilotingRegistration does not use defeat-text validation overrides".into());
    }
    let mut p=g.registration.program;
    let trigger=p["trigger"].as_str().ok_or("piloting registration missing trigger")?.to_owned();
    let hooks=p["hooks"].clone();
    let when_played=hooks==json!([{"alias":"whenPlayed","enabled":true}]);
    let self_attach=self_upgrade_attached_hooks(&hooks);
    if trigger!="eventHooks"||!(when_played||self_attach) {
        return Err("pilotingRegistration admits only WhenPlayed or self-attach onUpgradeAttached".into());
    }
    // Self-attach (`upgradeCard === source`) is the attach-time WhenPlayed for pilots.
    p["trigger"]=json!("whenPlayed");
    Ok(vec![compile_self_when_played(p,&w.title)?])
}
pub(crate) fn sides(entry:&Value)->Result<Sides,String> {
    let raw=&entry["abilities"];
    if internal(raw) {return Err("attachedHost is an internal program marker".into());}
    if !marker(raw)&&!crate::attached_constants::marker(raw) {
        let mut abilities=raw.clone();
        crate::discard_return::normalize(&mut abilities);
        return serde_json::from_value(abilities).map_err(|e|format!("invalid ability IR: {e}"));
    }
    if !raw.as_object().is_some_and(|o|o.len()==3&&["leader","unit","card"].iter().all(|k|o.contains_key(*k)))||!source_meta(entry) {
        return Err("attached aliases require complete ordinary upgrade metadata and matching printed stats".into());
    }
    let card=raw["card"].as_array().ok_or("attached aliases need card-side registrations")?;
    if card.is_empty()||card.len()>8 {return Err("attached registration count exceeds bound".into());}
    let mut out=Sides::default();
    let mut saw_self=false;
    let mut saw_host=false;
    let mut saw_enemy_damaged_companion=false;
    let mut saw_enemy_damaged_host=false;
    let mut saw_unit_exhaust_companion=false;
    for a in card {
        if self_marker(a) {
            if saw_host {return Err("pilotingRegistration cannot mix with attachedAbilityRegistration".into());}
            saw_self=true;
            out.card.extend(self_programs(a,entry)?);
        } else if host_marker(a) {
            if saw_self {return Err("pilotingRegistration cannot mix with attachedAbilityRegistration".into());}
            saw_host=true;
            let programs=host_programs(a,entry)?;
            if programs.iter().any(enemy_damaged_parent_stats) {saw_enemy_damaged_host=true;}
            out.card.extend(programs);
        } else if crate::attached_constants::marker(a) {
            if plain_pilot(entry) {
                return Err("piloting attached registration does not admit companion constants".into());
            }
            out.card.push(crate::attached_constants::ability(a)?);
        } else if plain_pilot(entry) && unit_ground_exhaust_companion(a) {
            if saw_host {return Err("piloting unit exhaust companion cannot mix with attachedAbilityRegistration".into());}
            if saw_unit_exhaust_companion {
                return Err("piloting unit exhaust companion admits at most one registration".into());
            }
            saw_unit_exhaust_companion=true;
            out.card.push(compile_unit_ground_exhaust(a)?);
        } else {
            if plain_pilot(entry) {
                // IG-88: unit-side mirror of the constant host enemy-damaged aura.
                if !enemy_damaged_companion(a) {
                    return Err("piloting attached registration does not admit companion registrations".into());
                }
                if saw_enemy_damaged_companion {
                    return Err("piloting enemy-damaged companion admits at most one registration".into());
                }
                saw_enemy_damaged_companion=true;
            }
            out.card.push(crate::attached_constants::companion(a)?);
        }
    }
    if saw_enemy_damaged_companion&&!saw_enemy_damaged_host {
        return Err("enemy-damaged companion requires matching piloting constant host grant".into());
    }
    if saw_unit_exhaust_companion&&!saw_self {
        return Err("piloting unit exhaust companion requires matching pilotingRegistration WhenPlayed".into());
    }
    if out.card.len()>8 {return Err("attached registration count exceeds bound".into());}
    Ok(out)
}
/// Exact reviewed Pilot + piloting-mode host registration contract.
/// May include one reviewed unit-side companion that mirrors a constant host aura.
pub(crate) fn piloting_host_entry(entry:&Value)->bool {
    plain_pilot(entry)
        && entry["abilities"]["card"].as_array().is_some_and(|card|{
            !card.is_empty()
                && card.iter().any(host_marker)
                && card.iter().all(|a|host_marker(a)||enemy_damaged_companion(a))
        })
        && sides(entry).is_ok()
}
/// Exact reviewed Pilot + pilotingRegistration WhenPlayed-while-attached contract
/// (including Iden self-attach Shield and Frisk cheap-upgrade defeat). May include one
/// reviewed unit-side WhenPlayed exhaust companion (Mandalorian).
pub(crate) fn piloting_self_entry(entry:&Value)->bool {
    plain_pilot(entry)
        && entry["abilities"]["card"].as_array().is_some_and(|card|{
            !card.is_empty()
                && card.iter().any(self_marker)
                && card.iter().all(|a|self_marker(a)||unit_ground_exhaust_companion(a))
        })
        && sides(entry).is_ok()
}
