//! Closed paired source projection for a live friendly exhausted-unit count.
//! Data validation only: no history watcher, storage, mutation, or callback evaluator.
use crate::ability_ir::{Ability,CardPrograms,Expr,Program,Sides};
use serde_json::{json,Value};

const SOURCE_HASH:&str="7b34e5913965588ea82110944db0b13aefe73d978abfaf1f343eaf774b68e727";
const REASON:&str="live exhausted count requires the complete closed paired source profile";
fn raw_profile()->Value {serde_json::from_str(r###"{"setCode":"ASH_035","sourcePath":"game/cards/08_ASH/units/TatooineRepulsorTrain.ts","sourceHash":"7b34e5913965588ea82110944db0b13aefe73d978abfaf1f343eaf774b68e727","abilities":{"leader":[],"unit":[],"card":[{"trigger":"constant","title":"This unit can't be attacked while you control 2 or more exhausted units","costs":[],"effect":{"kind":"cardRestriction","target":"self","duration":"persistent","restriction":"beAttacked"},"condition":{"kind":"binary","op":">=","left":{"kind":"historyHelperCall","name":"getFriendlyExhaustedUnitCount","args":[{"kind":"historyAbilityContext","binding":{"id":"callback:839:0","role":"condition","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"}}]},"right":{"kind":"const","value":2}}},{"trigger":"onAttack","title":"Deal 2 damage to a ground unit for each friendly exhausted unit","costs":[],"targetResolver":{"owner":"any","type":"unit","zone":"arena","arena":"ground"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"historyHelperCall","name":"getAttackDamage","args":[{"kind":"historyAbilityContext","binding":{"id":"callback:1467:0","role":"properties:damage","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"}}]}}}]},"className":"TatooineRepulsorTrain","baseClass":"NonLeaderUnitCard","id":"9629363501","internalName":"tatooine-repulsor-train","historyHelpers":[{"kind":"historyHelper","name":"getFriendlyExhaustedUnitCount","parameters":["data:helper:getFriendlyExhaustedUnitCount:0"],"body":{"kind":"dataBlock","statements":[{"kind":"dataReturn","value":{"kind":"dataMember","receiver":{"kind":"scopedArenaUnitsQuery","receiver":{"kind":"dataPath","root":["data:helper:getFriendlyExhaustedUnitCount:0"],"steps":[{"key":"player","optional":false}]},"filter":{"condition":{"kind":"queryCallback","parameters":[{"root":"attackBinding1_0","role":"candidate"}],"body":{"kind":"ref","path":["attackBinding1_0","exhausted"]}}},"type":"unit","controller":"receiverIdentity","order":"groundThenSpacePhysicalOrder","fieldCombination":"and","attributeArrayCombination":"or","predicateOrder":["aspect","keyword","trait","type","otherThan","condition"],"evaluation":"expressionEvaluation"},"key":"length","optional":false}}]}},{"kind":"historyHelper","name":"getAttackDamage","parameters":["data:helper:getAttackDamage:0"],"body":{"kind":"dataBlock","statements":[{"kind":"dataReturn","value":{"kind":"binary","op":"*","left":{"kind":"const","value":2},"right":{"kind":"historyHelperCall","name":"getFriendlyExhaustedUnitCount","args":[{"kind":"dataPath","root":["data:helper:getAttackDamage:0"],"steps":[]}]}}}]}}],"status":"supported","card":{"title":"Tatooine Repulsor Train","subtitle":"","cost":7,"hp":7,"power":8,"text":"This unit can't be attacked while you control 2 or more exhausted units (unless it gains Sentinel).\nOn Attack: Deal 2 damage to a ground unit for each friendly exhausted unit.","deployBox":null,"epicAction":"","unique":false,"rules":"(ERRATA) On Attack: Choose a ground unit. Deal 2 damage to it for each friendly exhausted unit.\nIf Tatooine Repulsor Train gains Sentinel, it can be attacked, even while you control 2 or more exhausted units.\n","upgradePower":null,"upgradeHp":null,"id":"9629363501","aspects":["command","aggression"],"traits":["underworld","vehicle","speeder"],"arena":"ground","keywords":[],"types":["unit"],"setId":{"set":"ASH","number":35},"setCodes":[{"set":"ASH","number":35}],"internalName":"tatooine-repulsor-train"},"dataHash":"257442c8045023339c9656ca70d8d6ee8eb3878502a1cc503618a887ba3596c5","keywordValues":{},"requiredCapabilities":["binary:>=","cardType:unit","ir:binary","ir:cardRestriction","ir:const","ir:damage","ir:historyAbilityContext","ir:historyHelperCall","targetType:unit","targetZone:arena","trigger:constant","trigger:onAttack"],"diagnostics":[]}"###).expect("reviewed full source profile")}

/// Scan all children before admission, including siblings after a marker. The
/// aggregate scheduling bound is checked before extending the work list.
pub(crate) fn marker(root:&Value)->Result<bool,String> {
    let mut pending=vec![(root,0usize)];let mut scheduled=1usize;let mut found=false;
    while let Some((v,depth))=pending.pop() {
        if depth>128 {return Err("live exhausted source inspection budget exceeded".into());}
        found|=v["sourceHash"]==SOURCE_HASH || v["className"]=="TatooineRepulsorTrain"
            || matches!(v["name"].as_str(),Some("getFriendlyExhaustedUnitCount"|"getAttackDamage"))
            || (v["kind"]=="ref" && v["path"]==json!(["abilityPlayer"]))
            || (v["kind"]=="arenaLength" && v["receiver"]["kind"]=="arenaList" && v["receiver"]["predicate"]==json!({"kind":"ref","path":["card","exhausted"]}));
        let children=match v {Value::Array(a)=>a.len(),Value::Object(o)=>o.len(),_=>0};
        scheduled=scheduled.checked_add(children).filter(|n|*n<=65_536).ok_or("live exhausted source inspection budget exceeded")?;
        if children>0 && depth==128 {return Err("live exhausted source inspection budget exceeded".into());}
        match v {Value::Array(a)=>pending.extend(a.iter().map(|x|(x,depth+1))),Value::Object(o)=>pending.extend(o.values().map(|x|(x,depth+1))),_=>{}}
    }Ok(found)
}
fn callback(v:&Value)->Option<&str> {
    let id=v.as_str()?;let parts=id.split(':').collect::<Vec<_>>();
    (parts.len()==3 && parts[0]=="callback" && parts[2]=="0" && !parts[1].is_empty()
        && parts[1].bytes().all(|b|b.is_ascii_digit()) && parts[1].parse::<u32>().is_ok() && id.len()<=128).then_some(id)
}
fn replace(v:&mut Value,old:&str,new:&str) {match v {Value::String(s) if s==old=>*s=new.into(),Value::Array(a)=>a.iter_mut().for_each(|v|replace(v,old,new)),Value::Object(o)=>o.values_mut().for_each(|v|replace(v,old,new)),_=>{}}}
/// The fingerprint covers the source prompt callback omitted by current raw IR;
/// it is additional evidence, never a substitute for complete AST/metadata equality.
/// The external printing key is not a behavior dispatch key. Every metadata and
/// helper field remains present and compared; unknown fields are rejected.
pub(crate) fn entry(e:&Value)->bool {
    if marker(e).is_err() {return false;}
    let Some(code)=e["setCode"].as_str().filter(|s|s.len()<=32 && s.split_once('_').is_some_and(|(a,b)|!a.is_empty()&&a.bytes().all(|x|x.is_ascii_uppercase()||x.is_ascii_digit())&&b.len()==3&&b.bytes().all(|x|x.is_ascii_digit()))) else{return false;};
    let Some(condition)=callback(&e["abilities"]["card"][0]["condition"]["left"]["args"][0]["binding"]["id"]) else{return false;};
    let Some(damage)=callback(&e["abilities"]["card"][1]["effect"]["amount"]["args"][0]["binding"]["id"]) else{return false;};
    if condition==damage {return false;}
    let mut expected=raw_profile();expected["setCode"]=json!(code);
    replace(&mut expected,"callback:839:0",condition);replace(&mut expected,"callback:1467:0",damage);
    *e==expected
}
pub(crate) fn count_expr()->Expr {Expr::ArenaLength{receiver:Box::new(Expr::ArenaList{receiver:Box::new(Expr::Ref{path:vec!["player".into()]}),predicate:Box::new(Expr::Ref{path:vec!["card".into(),"exhausted".into()]})})}}
/// Captured OnAttack context player, distinct from the refreshed constant owner.
/// Root's shared evaluator/codegen glue must supply this privileged receiver.
pub(crate) fn attack_count_expr()->Expr {Expr::ArenaLength{receiver:Box::new(Expr::ArenaList{receiver:Box::new(Expr::Ref{path:vec!["abilityPlayer".into()]}),predicate:Box::new(Expr::Ref{path:vec!["card".into(),"exhausted".into()]})})}}
pub(crate) fn damage_expr()->Expr {Expr::Binary{op:"*".into(),left:Box::new(Expr::Const{value:json!(2)}),right:Box::new(attack_count_expr())}}
pub(crate) fn title_parts()->Vec<Expr> {vec![Expr::Const{value:json!("Choose a ground unit to deal ")},damage_expr(),Expr::Const{value:json!(" damage to")} ]}
pub(crate) fn prompt_parts(parts:&[Expr])->bool {serde_json::to_value(parts).ok()==serde_json::to_value(title_parts()).ok()}
fn normalized_value()->Value {
    let mut sides=raw_profile()["abilities"].clone();
    sides["card"][0]["condition"]["left"]=serde_json::to_value(count_expr()).unwrap();
    sides["card"][1]["effect"]["amount"]=serde_json::to_value(damage_expr()).unwrap();
    sides["card"][1]["targetResolver"]["promptTitleParts"]=serde_json::to_value(title_parts()).unwrap();
    sides
}
pub(crate) fn normalized_sides()->Sides {serde_json::from_value(normalized_value()).expect("typed live exhausted projection")}
/// Raw source and metadata are never rewritten or enriched. The caller retains
/// the original entry while compiling these separately projected typed sides.
pub(crate) fn prepare(e:&Value)->Result<Option<Sides>,String> {
    if !marker(e)? {return Ok(None);}
    if !entry(e) {return Err(REASON.into());}
    Ok(Some(normalized_sides()))
}
pub(crate) fn registration(a:&Ability)->bool {
    normalized_sides().card.iter().any(|expected|serde_json::to_value(expected).ok()==serde_json::to_value(a).ok())
}
pub(crate) fn typed_marker(a:&Ability)->bool {marker(&serde_json::to_value(a).expect("typed ability")).unwrap_or(true)}
pub(crate) fn compiled_source(card:&Value,programs:&[&Program])->bool {
    if *card!=raw_profile()["card"] || programs.len()!=2 {return false;}
    normalized_sides().card.iter().zip(programs).all(|(a,p)|Program::compile(a).is_ok_and(|expected|serde_json::to_value(expected).ok()==serde_json::to_value(p).ok()))
}

/// Complete paired compilation after `prepare` has proved the unmodified source.
/// The caller must not pass arbitrary typed sides to this specialized route.
pub(crate) fn compile(sides:&Sides,start:usize)->Result<(CardPrograms,Vec<Program>),String> {
    if serde_json::to_value(sides).ok()!=serde_json::to_value(normalized_sides()).ok() {return Err(REASON.into());}
    let id=u16::try_from(start).map_err(|_|"ability program capacity exceeded")?;
    let next=id.checked_add(1).ok_or("ability program capacity exceeded")?;
    let programs=sides.card.iter().map(Program::compile).collect::<Result<Vec<_>,_>>()?;
    Ok((CardPrograms{card:vec![id,next],..CardPrograms::default()},programs))
}
