//! Source-bound selected own-hand discard ability cost. Pure admission only.
use crate::ability_ir::{Ability,CardPrograms,Cost,Program};
use serde::{Serialize,Deserialize};
use serde_json::Value;
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandSelector { pub owner:String, #[serde(rename="type")] pub card_type:String, pub zone:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscardCostEffect {pub kind:String,pub target:String,#[serde(rename="isCost")]pub is_cost:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedHandCost {
 pub selector:HandSelector,pub effect:DiscardCostEffect,pub timing:String,
 #[serde(rename="selectionName")]pub selection_name:String,pub chooser:String,
 #[serde(rename="mustFullyResolve")]pub must_fully_resolve:bool,
 #[serde(rename="promptTitleParts")]pub prompt_title_parts:Vec<crate::ability_ir::Expr>,
}
const PROFILE:&str=r###"{"setCode":"ASH_217","sourcePath":"game/cards/08_ASH/units/MayorsMajordomoNoProblemGroveling.ts","sourceHash":"f9b390c677b6a1cc25aed5de6fc58f422b9cb6b50ae1f1d252aa19af57ff66fb","abilities":{"leader":[],"unit":[],"card":[{"trigger":"action","title":"Exhaust a unit","costs":[{"kind":"exhaustSelf"},{"kind":"selectCardCost","selector":{"owner":"self","type":"any","zone":"hand"},"effect":{"kind":"discardSpecificCard","target":"costSelection","isCost":true},"timing":"abilityCostPayment","selectionName":"cost","chooser":"ownPlayer","mustFullyResolve":true,"promptTitleParts":[{"kind":"const","value":"Choose a card to discard"}]}],"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","predicate":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}},"effect":{"kind":"exhaust","target":"chosen"}}]},"className":"MayorsMajordomoNoProblemGroveling","baseClass":"NonLeaderUnitCard","id":"6565211534","internalName":"mayors-majordomo#no-problem-groveling","status":"supported","card":{"title":"Mayor's Majordomo","subtitle":"No Problem Groveling","cost":2,"hp":4,"power":1,"text":"Action [Exhaust, discard a card from your hand]: Exhaust a unit.","deployBox":null,"epicAction":"","unique":true,"rules":"ERRATA: Traits: Official, Twi'lek\n","upgradePower":null,"upgradeHp":null,"id":"6565211534","aspects":["cunning"],"traits":["official","twi'lek"],"arena":"ground","keywords":[],"types":["unit"],"setId":{"set":"ASH","number":217},"setCodes":[{"set":"ASH","number":217}],"internalName":"mayors-majordomo#no-problem-groveling"},"dataHash":"d2f76be2692f3ea1807d0a86ebccaad68a0b6a9b5822adfc7b3f9a00b1bb00e1","keywordValues":{},"requiredCapabilities":["binary:!==","cardType:unit","ir:binary","ir:const","ir:discardSpecificCard","ir:exhaust","ir:exhaustSelf","ir:ref","ir:selectCardCost","ref:card","ref:source","targetType:unit","targetZone:anyAttackable","trigger:action"],"diagnostics":[]}"###;
pub(crate) fn profile()->Value {serde_json::from_str(PROFILE).expect("closed selected-cost profile")}
const REASON:&str="selected hand-card cost requires complete closed ordinary source and cancellable meta-cost protocol";
/// Inspect whole raw entry before any family returns. Bounded inspection remains
/// after Blade's existing outer input budget. Common discard effects do not mark.
pub(crate) fn prepare(raw:&Value)->Result<bool,String>{
 let mut pending=vec![(raw,0usize)];let mut seen=0;let mut marked=false;
 while let Some((v,d))=pending.pop(){seen+=1;if seen>65536||d>128{return Err("selected hand-cost source inspection budget exceeded".into())}
  marked|=v["sourceHash"]=="f9b390c677b6a1cc25aed5de6fc58f422b9cb6b50ae1f1d252aa19af57ff66fb"||v["kind"]=="selectCardCost"||v["className"]=="MayorsMajordomoNoProblemGroveling"||v["internalName"]=="mayors-majordomo#no-problem-groveling";
  match v {Value::Object(o)=>pending.extend(o.values().map(|v|(v,d+1))),Value::Array(a)=>pending.extend(a.iter().map(|v|(v,d+1))),_=>{}}
 }
 if !marked{return Ok(false)}
 // A second source binding would get a different native definition index,
 // which is outside this ordinary Unique card's reviewed identity scope.
 // Keep the exact canonical profile including setCode; do not erase identity.
 if raw!=&profile(){return Err(REASON.into())}Ok(true)
}
pub(crate) fn checked_program_id(count:usize)->Result<u16,String>{
 u16::try_from(count.checked_add(1).ok_or("selected hand-cost program capacity exceeded")?).map(|stored|stored-1).map_err(|_|"selected hand-cost program capacity exceeded".into())
}
pub(crate) fn compiled()->Result<Program,String>{
 let mut raw=profile()["abilities"]["card"][0].clone();let cost=raw["costs"][1].clone();raw["costs"].as_array_mut().unwrap().remove(1);
 let mut p=Program::compile(&serde_json::from_value::<Ability>(raw).map_err(|e|e.to_string())?)?;
 // Pinned AbilityResolver.payCosts stable-partitions meta-action costs first.
 p.costs.insert(0,serde_json::from_value::<Cost>(cost).map_err(|e|e.to_string())?);
 p.executable=crate::program_backend(&serde_json::to_vec(&p).map_err(|e|e.to_string())?);Ok(p)
}
pub(crate) fn compile(count:usize)->Result<(CardPrograms,Vec<Program>),String>{
 let id=checked_program_id(count)?;Ok((CardPrograms{card:vec![id],..Default::default()},vec![compiled()?]))
}

pub(crate) fn requires(p:&Program)->bool {p.costs.iter().any(|c|matches!(c,Cost::SelectCardCost(_)))}
pub(crate) fn program(p:&Program)->bool {compiled().ok().is_some_and(|expected|serde_json::to_value(p).ok()==serde_json::to_value(expected).ok())}
