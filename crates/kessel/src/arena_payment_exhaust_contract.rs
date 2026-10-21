//! Closed ordinary optional entry payment followed by one live arena choice.
//! Full raw source proof only; execution uses shared resource payment, modal
//! selection and scalar arena exhaustion. No additional inline State storage.
use crate::ability_ir::{Ability,CardPrograms,Effect,Instruction,Program};
use serde_json::Value;
const PROFILE:&str=r###"{"setCode":"ASH_219","sourcePath":"game/cards/08_ASH/units/JodNaNawoodKeepingSecrets.ts","sourceHash":"298f2cc245afb02013ae7d8b42ceba6b476a06683d9a54d4f48acbd5c507da07","abilities":{"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","title":"Pay 4 resources to exhaust each unit in an arena","costs":[],"optional":true,"effect":{"kind":"payAdjustedResources","target":"ownPlayer","amount":{"kind":"const","value":4},"paymentType":"cardEffectPayment","isCost":false,"canCancelDuringPayment":false,"allowCostAdjustments":true},"ifYouDo":{"title":"Exhaust each unit in an arena","costs":[],"targetResolver":{"kind":"choiceSelection","mode":"select","resultBinding":"select","namedResultBinding":"selects.target.choice","defaults":"pinnedTargetResolver","chooser":{"kind":"const","value":"self"},"promptTitleParts":[{"kind":"const","value":"Choose an arena"}],"choices":{"entries":[{"labelParts":[{"kind":"const","value":"Space"}],"effect":{"kind":"exhaust","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"spaceArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"spaceArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"spaceArena"}}}}}}}},{"labelParts":[{"kind":"const","value":"Ground"}],"effect":{"kind":"exhaust","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"groundArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"groundArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"groundArena"}}}}}}}}],"contextBinding":"resolverContextByReference","keyOrder":"javascriptPropertyOrder","duplicatePolicy":"lastAssignmentWins","factory":false}},"effect":{"kind":"resolveChoice","resultBinding":"select"}}}]},"className":"JodNaNawoodKeepingSecrets","baseClass":"NonLeaderUnitCard","id":"8838303580","internalName":"jod-na-nawood#keeping-secrets","status":"supported","card":{"title":"Jod Na Nawood","subtitle":"Keeping Secrets","cost":3,"hp":3,"power":4,"text":"Sentinel\nWhen Played: You may pay 4 resources. If you do, choose an arena. Exhaust each unit in that arena.","deployBox":null,"epicAction":"","unique":true,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"8838303580","aspects":["cunning"],"traits":["force","underworld"],"arena":"ground","keywords":["sentinel"],"types":["unit"],"setId":{"set":"ASH","number":219},"setCodes":[{"set":"ASH","number":219}],"internalName":"jod-na-nawood#keeping-secrets"},"dataHash":"1e82cbfe8cf64ab02e4a1a64c25bef54995516303953d84d45a26833ab0cf88b","keywordValues":{"sentinel":1},"requiredCapabilities":["ability:ifYouDo","binary:&&","binary:===","binary:||","cardType:unit","ir:arenaList","ir:binary","ir:choiceSelection","ir:const","ir:exhaust","ir:payAdjustedResources","ir:ref","ir:resolveChoice","ir:unary","keyword:sentinel","ref:card.zoneName","ref:game","targetMode:select","trigger:whenPlayed","unary:!"],"diagnostics":[]}"###;
pub(crate) fn profile()->Value {serde_json::from_str(PROFILE).expect("closed arena payment source")}
pub(crate) fn prepare(raw:&Value)->Result<bool,String>{
 let expected=profile();let mut todo=vec![(raw,0usize)];let(mut n,mut marked,mut payment,mut choice)=(0,false,false,false);
 while let Some((v,depth))=todo.pop(){n+=1;if n>65536||depth>128{return Err("arena payment source inspection capacity exceeded".into());}
  marked|=v["sourceHash"]==expected["sourceHash"]||v["className"]==expected["className"]||v["internalName"]==expected["internalName"];
  payment|=v["kind"]=="payAdjustedResources";choice|=v["kind"]=="choiceSelection";
  match v {Value::Array(a)=>todo.extend(a.iter().map(|v|(v,depth+1))),Value::Object(o)=>todo.extend(o.values().map(|v|(v,depth+1))),_=>{}}
  if todo.len().saturating_add(n)>65536{return Err("arena payment source inspection capacity exceeded".into());}
 }
 if !(marked||payment&&choice){return Ok(false);}
 // Only the reviewed physical source binding is admitted. A foreign setCode
 // would create a second definition and evade ordinary same-definition Unique.
 if raw!=&expected{return Err("arena payment requires complete closed optional payment and arena source profile".into());}Ok(true)
}
pub(crate) fn compile(start:usize)->Result<(CardPrograms,Vec<Program>),String>{
 let entry=profile();let mut raw=entry["abilities"]["card"][0].clone();let next=raw.as_object_mut().unwrap().remove("ifYouDo").unwrap();
 let ability:Ability=serde_json::from_value(raw).map_err(|e|e.to_string())?;
 let mut p=Program::compile(&ability)?;
 if p.instructions.len()!=2{return Err("arena payment parent shape drift".into());}
 let branches=next["targetResolver"]["choices"]["entries"].as_array().ok_or("arena choices absent")?;
 let space:Effect=serde_json::from_value(branches[0]["effect"].clone()).map_err(|e|e.to_string())?;
 let ground:Effect=serde_json::from_value(branches[1]["effect"].clone()).map_err(|e|e.to_string())?;
 // ifYouDo executes inside the original initiating window, before the final
 // EndStep. Its success is the resolved payment event, not the resource remainder.
 p.instructions=vec![Instruction::Apply(ability.effect),Instruction::IfResult{skip:7},Instruction::ModalChoice(crate::modal_choices::Menu{
  chooser:crate::modal_choices::Chooser::Own,prompt_title:"Choose an arena".into(),
  branches:vec![crate::modal_choices::Branch{label:"Space".into(),start:3,end:4,always_legal:false},crate::modal_choices::Branch{label:"Ground".into(),start:5,end:6,always_legal:false}],end:7,preserve_target:false,unless_branch:None}),
  Instruction::Apply(space),Instruction::Jump{to:7},Instruction::Apply(ground),Instruction::Jump{to:7},Instruction::EndStep];
 p.event_end=7;p.executable=Err("closed arena payment uses shared Rust interpreter");
 let id=u16::try_from(start).map_err(|_|"ability program capacity exceeded")?;
 Ok((CardPrograms{card:vec![id],..CardPrograms::default()},vec![p]))
}

/// Structural obligation, never card-id dispatch. Only a fully closed source
/// can construct this modal arena-exhaust programme in the first place.
pub(crate) fn requires_global_order(p:&Program)->bool {
 p.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::PayAdjustedResources{..})))&&p.instructions.iter().any(|i|matches!(i,Instruction::ModalChoice(_)))&&p.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::Exhaust{target:crate::ability_ir::Target::ArenaList(list)}) if list.controller_order.is_none()))
}
pub(crate) fn requires(p:&Program)->bool {requires_global_order(p)}
pub(crate) fn program(p:&Program)->bool {
 let Ok((_,programs))=compile(0) else{return false;};serde_json::to_value(p).ok()==serde_json::to_value(&programs[0]).ok()
}
