//! Closed source proof for declaration history and attack-local damage queries.
//! No game state, runtime evaluator or generic callback decoding belongs here.
use serde_json::{json,Value};
use crate::ability_ir::{Ability,CardPrograms,Expr,Instruction,Program};
#[derive(Debug,Clone,Copy,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Query { OpponentDeclaredOwnBase,CurrentAttackBaseDamage,FriendlyAttackerHasBaseRecord,SourceInternalName }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum Kind { Declaration,Heal,Advantage }
const DECLARATION:&str=r###"{"abilities":{"card":[{"costs":[{"amount":{"kind":"const","value":1},"kind":"resources"},{"kind":"exhaustSelf"}],"effect":{"condition":{"args":[{"fields":{"controller":{"kind":"ref","path":["opponent"]},"filter":{"bindings":["data:1501:0"],"body":{"args":[{"kind":"ref","path":["player","base"]}],"kind":"dataCall","method":"includes","optionalCall":false,"optionalReceiver":false,"receiver":{"kind":"dataPath","root":["data:1501:0"],"steps":[{"key":"targets","optional":false}]}},"kind":"dataLambda"}},"kind":"historyOptions"}],"binding":"attacksThisPhaseWatcher","kind":"historyQuery","operation":"someUnitAttackedControlledByPlayer","readAt":"expressionEvaluation","watcher":"attacksThisPhase"},"kind":"conditional","onFalse":{"kind":"noAction"},"onTrue":{"amount":{"kind":"const","value":1},"entersReady":false,"kind":"createTokenUnit","target":"ownPlayer","tokenType":"mandalorian"}},"title":"Create a Mandalorian token","trigger":"action"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"ground","aspects":["command"],"cost":2,"deployBox":null,"epicAction":"","hp":3,"id":"4885389871","internalName":"greef-karga#introductions-are-in-order","keywords":[],"power":2,"rules":null,"setCodes":[{"number":119,"set":"ASH"}],"setId":{"number":119,"set":"ASH"},"subtitle":"Introductions are in Order","text":"Action [1 resource, Exhaust]: If your base was attacked this phase, create a Mandalorian token.","title":"Greef Karga","traits":["official"],"types":["unit"],"unique":true,"upgradeHp":null,"upgradePower":null},"className":"GreefKargaIntroductionsAreInOrder","dataHash":"5670978866b121ffda1d3a589fd6132479796e3b450fee542baf5f9f501236ad","diagnostics":[],"id":"4885389871","internalName":"greef-karga#introductions-are-in-order","keywordValues":{},"requiredCapabilities":["cardType:unit","ir:conditional","ir:const","ir:createTokenUnit","ir:dataCall","ir:dataLambda","ir:dataPath","ir:exhaustSelf","ir:historyOptions","ir:historyQuery","ir:noAction","ir:ref","ir:resources","ref:opponent","ref:player.base","trigger:action"],"setCode":"ASH_119","sourceHash":"38f17f62bcbb0125d06a0cd74ba39483574fee4e7ed8f39f2282a82f6f03164e","sourcePath":"game/cards/08_ASH/units/GreefKargaIntroductionsAreInOrder.ts","stateWatchers":[{"binding":"attacksThisPhaseWatcher","implementation":"AttacksThisPhaseWatcher","kind":"phaseHistory","queryContract":"pinnedWatcherApi","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded","sharedScope":"game","watcher":"attacksThisPhase"}],"status":"supported"}"###;
const HEAL:&str=r###"{"abilities":{"card":[{"contextTitle":[{"kind":"const","value":"Heal "},{"args":[{"kind":"ref","path":["source"]},{"binding":{"bindAt":"callbackInvocation","id":"callback:1253:0","readAt":"expressionEvaluation","reference":"lexicalContextObject","role":"contextTitle"},"kind":"historyAbilityContext"}],"binding":"damageDealtThisPhaseWatcher","kind":"historyQuery","operation":"getDamageDealtToBaseByUnitThisAttack","readAt":"expressionEvaluation","watcher":"damageDealtThisPhase"},{"kind":"const","value":" damage from your base"}],"costs":[],"effect":{"condition":{"kind":"binary","left":{"binding":"const:1517","captureAt":"declarationEvaluation","kind":"capturedHistory","owner":"callback:1466","steps":[],"value":{"args":[{"kind":"ref","path":["source"]},{"binding":{"bindAt":"callbackInvocation","id":"callback:1466:0","readAt":"expressionEvaluation","reference":"lexicalContextObject","role":"properties:conditional"},"kind":"historyAbilityContext"}],"binding":"damageDealtThisPhaseWatcher","kind":"historyQuery","operation":"getDamageDealtToBaseByUnitThisAttack","readAt":"expressionEvaluation","watcher":"damageDealtThisPhase"}},"op":">","right":{"kind":"const","value":0}},"kind":"conditional","onFalse":{"kind":"noAction"},"onTrue":{"amount":{"binding":"const:1517","captureAt":"declarationEvaluation","kind":"capturedHistory","owner":"callback:1466","steps":[],"value":{"args":[{"kind":"ref","path":["source"]},{"binding":{"bindAt":"callbackInvocation","id":"callback:1466:0","readAt":"expressionEvaluation","reference":"lexicalContextObject","role":"properties:conditional"},"kind":"historyAbilityContext"}],"binding":"damageDealtThisPhaseWatcher","kind":"historyQuery","operation":"getDamageDealtToBaseByUnitThisAttack","readAt":"expressionEvaluation","watcher":"damageDealtThisPhase"}},"kind":"heal","target":"ownBase"}},"title":"Heal damage from your base equal to damage you deal on opponent base","trigger":"whenAttackEnds"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"ground","aspects":["vigilance","aggression","heroism"],"cost":3,"deployBox":null,"epicAction":"","hp":4,"id":"5557320306","internalName":"hera-syndulla#renegade-general","keywords":[],"power":3,"rules":"\u201cCombat damage\u201d is only the damage dealt during the \u201cEnd attack\u201d step of an attack.\nWhen Attack Ends abilities trigger at the same time as units deal combat damage. Hera does not need to survive the attack in order to heal damage from your base.\n","setCodes":[{"number":31,"set":"ASH"}],"setId":{"number":31,"set":"ASH"},"subtitle":"Renegade General","text":"When Attack Ends: If this unit dealt combat damage to a base, heal that much damage from your base.","title":"Hera Syndulla","traits":["new republic","twi'lek","spectre"],"types":["unit"],"unique":true,"upgradeHp":null,"upgradePower":null},"className":"HeraSyndullaRenegadeGeneral","dataHash":"b41c27ecf6e489a39ee0f594712a96e2171b7219a1f5464cb292b5a0ca2689b0","diagnostics":[],"id":"5557320306","internalName":"hera-syndulla#renegade-general","keywordValues":{},"requiredCapabilities":["binary:>","cardType:unit","ir:binary","ir:capturedHistory","ir:conditional","ir:const","ir:heal","ir:historyAbilityContext","ir:historyQuery","ir:noAction","ir:ref","ref:source","trigger:whenAttackEnds"],"setCode":"ASH_031","sourceHash":"7b0fc28932b0ea22d3e37324755a98acc44d1c7ab73fb788ec30ad7c845c9b1f","sourcePath":"game/cards/08_ASH/units/HeraSyndullaRenegadeGeneral.ts","stateWatchers":[{"binding":"damageDealtThisPhaseWatcher","implementation":"DamageDealtThisPhaseWatcher","kind":"phaseHistory","queryContract":"pinnedWatcherApi","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded","sharedScope":"game","watcher":"damageDealtThisPhase"}],"status":"supported"}"###;
const ADVANTAGE:&str=r###"{"abilities":{"card":[{"contextTitle":[{"kind":"const","value":"If the attacker dealt combat damage to a base, give an Advantage token to "},{"kind":"ref","path":["source"]},{"kind":"const","value":""}],"costs":[],"effect":{"amount":{"kind":"const","value":1},"batch":"singleCreationEvent","createdBy":"self","kind":"giveAdvantage","target":"self","tokenOwner":"recipientController"},"hooks":[{"event":"onAttackEnd","predicate":{"kind":"binary","left":{"kind":"binary","left":{"argumentBinding":{"bindAt":"callbackInvocation","id":"callback:1426:0","readAt":"expressionEvaluation"},"kind":"dataPath","root":["event"],"steps":[{"key":"attack","optional":false},{"key":"attackingPlayer","optional":false}]},"op":"===","right":{"kind":"ref","path":["player"]}},"op":"&&","right":{"args":[{"argumentBinding":{"bindAt":"callbackInvocation","id":"callback:1426:0","readAt":"expressionEvaluation"},"kind":"dataPath","root":["event"],"steps":[{"key":"attack","optional":false},{"key":"attacker","optional":false}]},{"binding":{"bindAt":"callbackInvocation","id":"callback:1426:1","readAt":"expressionEvaluation","reference":"lexicalContextObject","role":"onAttackEnd"},"kind":"historyAbilityContext"}],"binding":"damageDealtThisPhaseWatcher","kind":"historyQuery","operation":"unitHasDealtCombatDamageToBaseThisAttack","readAt":"expressionEvaluation","watcher":"damageDealtThisPhase"}}}],"title":"If the attacker dealt combat damage to a base, give an Advantage token to this unit","trigger":"eventHooks"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"space","aspects":["aggression","villainy"],"cost":3,"deployBox":null,"epicAction":"","hp":4,"id":"3024525867","internalName":"vanes-snub-fighter#brash-and-proud","keywords":[],"power":2,"rules":"When Attack Ends abilities trigger at the same time as units deal combat damage. The attacking unit doesn't need to survive in order to give an Advantage token, so long as it dealt combat damage to a base.\n\u201cCombat damage\u201d is only the damage dealt during the \u201cEnd attack\u201d step of an attack.\n","setCodes":[{"number":144,"set":"ASH"}],"setId":{"number":144,"set":"ASH"},"subtitle":"Brash and Proud","text":"When a friendly unit's attack ends: If it dealt combat damage to a base, give an Advantage token to this unit.","title":"Vane's Snub Fighter","traits":["underworld","vehicle","fighter"],"types":["unit"],"unique":true,"upgradeHp":null,"upgradePower":null},"className":"VanesSnubFighterBrashAndProud","dataHash":"b862dcfda1554d64bfdab9f9ec0a7e71de745253d9b6ae6c768ba4c18bb321f4","diagnostics":[],"id":"3024525867","internalName":"vanes-snub-fighter#brash-and-proud","keywordValues":{},"requiredCapabilities":["binary:&&","binary:===","cardType:unit","ir:binary","ir:const","ir:dataPath","ir:giveAdvantage","ir:historyAbilityContext","ir:historyQuery","ir:ref","ref:player","ref:source","trigger:eventHooks"],"setCode":"ASH_144","sourceHash":"4cdf2acb18d484b1ed3e33c817d2b66ae5a1cb485856c8a08edb36b74e4ee8b4","sourcePath":"game/cards/08_ASH/units/VanesSnubFighterBrashAndProud.ts","stateWatchers":[{"binding":"damageDealtThisPhaseWatcher","implementation":"DamageDealtThisPhaseWatcher","kind":"phaseHistory","queryContract":"pinnedWatcherApi","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded","sharedScope":"game","watcher":"damageDealtThisPhase"}],"status":"supported"}"###;
pub(crate) fn profile(kind:Kind)->Value {
 serde_json::from_str(match kind {Kind::Declaration=>DECLARATION,Kind::Heal=>HEAL,Kind::Advantage=>ADVANTAGE}).expect("closed attack history source")
}
pub(crate) fn prepare(raw:&Value)->Result<Option<Kind>,String> {
 let kinds=[Kind::Declaration,Kind::Heal,Kind::Advantage];
 const IDENTITIES:&[(&str,&str)]=&[("setCode","ASH_119"),("sourcePath","game/cards/08_ASH/units/GreefKargaIntroductionsAreInOrder.ts"),("sourceHash","38f17f62bcbb0125d06a0cd74ba39483574fee4e7ed8f39f2282a82f6f03164e"),("className","GreefKargaIntroductionsAreInOrder"),("internalName","greef-karga#introductions-are-in-order"),("id","4885389871"),("setCode","ASH_031"),("sourcePath","game/cards/08_ASH/units/HeraSyndullaRenegadeGeneral.ts"),("sourceHash","7b0fc28932b0ea22d3e37324755a98acc44d1c7ab73fb788ec30ad7c845c9b1f"),("className","HeraSyndullaRenegadeGeneral"),("internalName","hera-syndulla#renegade-general"),("id","5557320306"),("setCode","ASH_144"),("sourcePath","game/cards/08_ASH/units/VanesSnubFighterBrashAndProud.ts"),("sourceHash","4cdf2acb18d484b1ed3e33c817d2b66ae5a1cb485856c8a08edb36b74e4ee8b4"),("className","VanesSnubFighterBrashAndProud"),("internalName","vanes-snub-fighter#brash-and-proud"),("id","3024525867")];
 let mut stack=vec![(raw,0usize)];let(mut seen,mut marked)=(0usize,false);
 while let Some((v,depth))=stack.pop() {
  seen+=1;if seen>65536||depth>128 {return Err("attack base history source inspection capacity exceeded".into());}
  marked|=IDENTITIES.iter().any(|(key,value)|v[*key].as_str()==Some(*value));
  marked|=v["kind"]=="attackBaseQuery"||matches!(v["operation"].as_str(),Some("someUnitAttackedControlledByPlayer"|"getDamageDealtToBaseByUnitThisAttack"|"unitHasDealtCombatDamageToBaseThisAttack"));
  match v {Value::Array(a)=>stack.extend(a.iter().map(|v|(v,depth+1))),Value::Object(o)=>stack.extend(o.values().map(|v|(v,depth+1))),_=>{}}
  if stack.len().saturating_add(seen)>65536 {return Err("attack base history source inspection capacity exceeded".into());}
 }
 if !marked {return Ok(None);}
 for kind in kinds {if raw==&profile(kind) {return Ok(Some(kind));}}
 Err("attack base history requires the complete canonical source, metadata and registration profile".into())
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn closed_source_provenance() {
  for kind in [Kind::Declaration,Kind::Heal,Kind::Advantage] {
   let p=profile(kind);assert_eq!(prepare(&p),Ok(Some(kind)));
   for key in ["setCode","sourcePath","sourceHash","className","baseClass","internalName","id","card","stateWatchers","abilities","keywordValues","diagnostics"] {
    let mut bad=p.clone();bad[key]=serde_json::json!("unreviewed");assert!(prepare(&bad).is_err(),"{key}");
   }
   let mut bad=p.clone();bad["unrecognized"]=true.into();assert!(prepare(&bad).is_err());
   assert!(prepare(&serde_json::json!({"ordinary":p})).is_err());
  }
  assert_eq!(prepare(&serde_json::json!({"abilities":{"card":[]}})),Ok(None));
 }
}

/// Neutral headers pass ordinary validation before only these closed nodes are
/// installed. Portable standalone expressions cannot authorize these queries.
pub(crate) fn compile(kind:Kind,start:usize)->Result<(CardPrograms,Vec<Program>),String> {
 let entry=profile(kind);let original=&entry["abilities"]["card"][0];let mut raw=original.clone();
 let q=|query|Expr::AttackBaseQuery{query};
 match kind {
  Kind::Declaration=>raw["effect"]["condition"]=json!({"kind":"const","value":true}),
  Kind::Heal=>{
   raw["contextTitle"][1]=json!({"kind":"const","value":0});
   raw["effect"]["condition"]=json!({"kind":"const","value":true});
   raw["effect"]["onTrue"]["amount"]=json!({"kind":"const","value":1});
  },
  Kind::Advantage=>{
   raw.as_object_mut().unwrap().remove("hooks");raw["trigger"]="whenAttackEnds".into();
   raw["contextTitle"][1]=json!({"kind":"const","value":"source"});
  }
 }
 let ability:Ability=serde_json::from_value(raw).map_err(|e|e.to_string())?;
 let mut p=Program::compile(&ability)?;
 match kind {
  Kind::Declaration|Kind::Heal=>{
   let Some(Instruction::Conditional{condition,..})=p.instructions.first_mut() else{return Err("attack history conditional shape drift".into());};
   *condition=if kind==Kind::Declaration {q(Query::OpponentDeclaredOwnBase)}else{Expr::Binary{op:">".into(),left:Box::new(q(Query::CurrentAttackBaseDamage)),right:Box::new(Expr::Const{value:0.into()})}};
   if kind==Kind::Heal {
    // No mutation, prompt, cost or callback occurs between this condition and
    // its scalar heal. Both reads therefore denote the factory's one captured
    // immutable attack-local amount. Scalar execution captures it before heal.
    let mut found=false;for i in &mut p.instructions {if let Instruction::Apply(crate::ability_ir::Effect::Heal{amount,..})=i {*amount=q(Query::CurrentAttackBaseDamage);found=true;}}
    if !found{return Err("attack history heal shape drift".into());}p.context_title[1]=q(Query::CurrentAttackBaseDamage);
   }
  },
  Kind::Advantage=>{p.condition=Some(q(Query::FriendlyAttackerHasBaseRecord));p.context_title[1]=q(Query::SourceInternalName);}
 }
 p.executable=Err("closed attack base history uses shared Rust interpreter");
 let id=u16::try_from(start).map_err(|_|"ability program capacity exceeded")?;
 Ok((CardPrograms{card:vec![id],..CardPrograms::default()},vec![p]))
}
pub(crate) fn requires(p:&Program,query:Query)->bool {
 let test=|e:&Expr|e.any(|e|match e {Expr::AttackBaseQuery{query:q}=>Some(*q==query),_=>None});
 p.condition.as_ref().is_some_and(test)||p.context_title.iter().any(test)||p.instructions.iter().any(|i|matches!(i,Instruction::Conditional{condition,..} if test(condition)))
}

/// No admitted producer grants Support to these programmes. The legacy raw
/// Support ABI carries no late-listener registration chronology for them.
pub(crate) fn copied_program_allowed(p:&Program)->Result<(),String> {
 if [Query::OpponentDeclaredOwnBase,Query::CurrentAttackBaseDamage,Query::FriendlyAttackerHasBaseRecord,Query::SourceInternalName].into_iter().any(|q|requires(p,q)) {return Err("copied attack base history query requires an unsupported registration chronology".into());}
 Ok(())
}

/// Inventory authorization belongs to these exact physical creator programmes,
/// independently of whether their optional/conditional token effect has fired.
pub(crate) fn inventory_program(p:&Program)->bool {
 for (kind,query) in [(Kind::Declaration,Query::OpponentDeclaredOwnBase),(Kind::Advantage,Query::FriendlyAttackerHasBaseRecord)] {
  if requires(p,query) {
   let Ok((_,known))=compile(kind,0) else{return false;};
   if serde_json::to_value(p).ok()==serde_json::to_value(&known[0]).ok() {return true;}
  }
 }
 false
}

/// The current executable copy graph has intrinsic donors only. A newly
/// introduced Support producer must get a chronology ABI before these source
/// programmes can coexist with it; reject this family instead of late paid Err.
pub(crate) fn validate_copy_closed_graph(programs:&[Program])->Result<(),String> {
 for p in programs {
  let value=serde_json::to_value(p).map_err(|e|e.to_string())?;let mut stack=vec![(&value,0usize)];let mut count=0usize;
  while let Some((v,depth))=stack.pop() {
   count+=1;if count>65536||depth>128 {return Err("copy graph inspection capacity exceeded".into());}
   if v["kind"]=="gainKeyword"&&v["keyword"]=="support"||v["kind"]=="copyKeywords"&&v["domain"].as_array().is_some_and(|a|a.iter().any(|v|v=="support")) {return Err("a Support producer requires reviewed attack-history copied registration chronology".into());}
   match v {Value::Array(a)=>stack.extend(a.iter().map(|v|(v,depth+1))),Value::Object(o)=>stack.extend(o.values().map(|v|(v,depth+1))),_=>{}}
   if stack.len().saturating_add(count)>65536 {return Err("copy graph inspection capacity exceeded".into());}
  }
 }
 Ok(())
}
pub(crate) fn has_query(p:&Program)->bool {[Query::OpponentDeclaredOwnBase,Query::CurrentAttackBaseDamage,Query::FriendlyAttackerHasBaseRecord,Query::SourceInternalName].into_iter().any(|q|requires(p,q))}
