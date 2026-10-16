'use strict';

// Pending selection operations preserve chooser identity and resolver contracts.
// They are intentionally separate from the executable scalar card resolver.
const EXTRA_FIELDS=['numCardsFunc','multiSelectCardCondition','minNumCardsFunc','maxNumCardsFunc','useSingleSelectModeFunc','cardStat','maxStat','capturedByFilter','condition','effectChoices','mustChangeGameState','hideIfNoLegalTargets','appendToDefaultTitle'];
function richResolver(obj,api,force=false) {
 const {ts,unwrap,literal,expr,contextTitle,fail}=api,{props,ctx,node}=obj;
 let mode='single';if(props.has('mode'))mode=literal(props.get('mode'),ctx);
 const player=['player','multiplePlayers'].includes(mode);
 // Preserve the already frozen piloting-player representation.
 if(player&&ctx.pendingRegistration)return null;
 let chooser='self';
 if(props.has('choosingPlayer')){const v=unwrap(props.get('choosingPlayer'));chooser=ts.isArrowFunction(v)||ts.isFunctionExpression(v)?null:literal(v,ctx);}
 const literalOrNull=input=>{try{return literal(input,ctx);}catch(error){if(!error.diagnostic)throw error;return null;}};
 const types=props.has('cardTypeFilter')?literalOrNull(props.get('cardTypeFilter')):'any';
 const zone=props.has('zoneFilter')?literalOrNull(props.get('zoneFilter')):'anyAttackable';
 const supportedTypes=new Set(['unit','nonLeaderUnit','non-leader unit','basicUnit','base','any','upgrade','event','leaderUnit','tokenUnit','token','basicUpgrade','tokenUpgrade','nonTokenUnit','nonUnit']);
 const typeUnion=Array.isArray(types)&&!(types.length===2&&types.includes('unit')&&types.includes('base'));
 const rich=player||chooser!=='self'||typeUnion||!Array.isArray(types)&&!supportedTypes.has(types)||Array.isArray(zone)||zone===null||zone==='capture'||EXTRA_FIELDS.some(key=>props.has(key));
 if(!rich&&!force)return null;
 const local={...ctx,pendingSelection:true};
 const result={kind:player?'playerSelection':'cardSelection',mode,chooser:chooser===null?expr(props.get('choosingPlayer'),local):chooser,resultBinding:player?'selectedPlayer':'target',defaults:'pinnedTargetResolver'};
 if(![null,'self','opponent'].includes(chooser))fail(ctx,node,'selection-chooser');
 result.contextBinding='target';result.choosingPlayerOverride='contextOverrideFirst';result.delayOpponentBeforeCosts=true;
 if(!player){
  result.cardTypes=props.has('cardTypeFilter')?expr(props.get('cardTypeFilter'),local):{kind:'const',value:'any'};
  result.zones=props.has('zoneFilter')?expr(props.get('zoneFilter'),local):{kind:'const',value:'anyAttackable'};
  result.controller=props.has('controller')?expr(props.get('controller'),local):{kind:'const',value:'any'};
  if(props.has('cardCondition'))result.predicate=expr(props.get('cardCondition'),local,['selectionCandidate','context']);
 }
 const fields={numCards:['context'],numCardsFunc:['context','selectedCards'],multiSelectCardCondition:['selectionCandidate','selectedCards','context'],minNumCardsFunc:['context','selectedCards'],maxNumCardsFunc:['context','selectedCards'],useSingleSelectModeFunc:['selectionCandidate','selectedCards','context'],cardStat:['selectionCandidate'],maxStat:[],capturedByFilter:['context'],condition:['context'],effectChoices:['relativePlayer','context']};
 for(const[key,params]of Object.entries(fields))if(props.has(key))result[key]=expr(props.get(key),local,params);
 for(const key of ['optional','canChooseNoCards','hideIfNoLegalTargets'])if(props.has(key)){const value=literal(props.get(key),ctx);if(typeof value!=='boolean')fail(ctx,node,'selection-boolean:'+key);result[key]=value;}
 for(const key of ['mustChangeGameState','appendToDefaultTitle','waitingPromptTitle'])if(props.has(key))result[key]=literal(props.get(key),ctx);
 if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),local);
 return result;
}
function selectedPlayerEffect(input,ctx,api) {
 if(!ctx.selectedPlayerTarget)return null;
 const {ts,unwrap,access,object,expr,effect,fail}=api,node=unwrap(input);
 if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
 const p=access(node.expression,ctx),kind=p.at(-1);
 if(p.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])||p[1]!=='immediateEffects')return null;
 const allowed={draw:['amount','target'],readyResources:['amount','target'],indirectDamageToPlayer:['amount','target'],discardCardsFromOwnHand:['amount','target'],discardFromDeck:['amount','target'],discardResources:['amount','target'],payResources:['amount','target'],createCreditToken:['amount','target']};
 if(!allowed[kind])return null;
 if(node.arguments.length>1)fail(ctx,node,'selected-player-system-arity');
 const obj=node.arguments.length?object(node.arguments[0],ctx):{props:new Map(),ctx,node};api.knownProperties(obj,allowed[kind]);
 const properties={};for(const[key,value]of obj.props)properties[key]=expr(value,{...obj.ctx,pendingSelection:true});
 return {kind:'selectedPlayerSystem',system:kind,target:properties.target||ctx.selectedPlayerTarget,properties,defaults:'pinnedSystem',defaultTargetOverride:ctx.selectedPlayerTarget};
}
function selectionExpression(input,ctx,api){
 if(!ctx.pendingSelection)return null;
 const {ts,unwrap,expr,fail}=api,node=unwrap(input);
 if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)){
  const name=node.expression.name.text;
  if(['hasCost','isTokenUnit','isAttacking','isDefending'].includes(name)){if(node.arguments.length||node.questionDotToken||node.expression.questionDotToken)fail(ctx,node,'selection-card-query');return {kind:'selectionCardQuery',name,receiver:expr(node.expression.expression,ctx)};}
  if(name==='reduce'){
   if(node.questionDotToken||node.expression.questionDotToken||node.arguments.length!==2)fail(ctx,node,'selection-reduce-shape');
   const fn=unwrap(node.arguments[0]);if(!ts.isArrowFunction(fn)||fn.parameters.length!==2)fail(ctx,node,'selection-reduce-callback');
   return {kind:'selectionReduce',receiver:expr(node.expression.expression,ctx),step:expr(fn,ctx,['selectionAccumulator','selectedCard']),seed:expr(node.arguments[1],ctx),limit:256};
  }
  if(ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='EnumHelpers'&&['asConcretePlayer','asRelativePlayer','isArena'].includes(name)){
   if(node.arguments.length!==(name==='isArena'?1:2)||node.questionDotToken||node.expression.questionDotToken)fail(ctx,node,'selection-relative-player-call');
   return {kind:'selectionPlayerConversion',operation:name,args:node.arguments.map(a=>expr(a,ctx))};
  }
 }
 return null;
}
function richNamedResolvers(input,context,api){
 const {object,literal,expr,effect,fail}=api,obj=object(input,context),slots=new Map(),entries=[],dependents=new Set();
 if(context.namedTargets?.size)fail(context,input,'nested-rich-named-resolvers');
 if(!obj.props.size||obj.props.size>8)fail(context,input,'rich-named-target-bound');
 const modes=new Set(['single','exactly','upTo','unlimited','exactlyVariable','upToVariable','maxStat','betweenVariable','player','multiplePlayers']);
 for(const[name,value]of obj.props){
  if(!/^[A-Za-z_][A-Za-z0-9_]{0,31}$/.test(name)||['__proto__','prototype','constructor'].includes(name))fail(context,value,'rich-named-target-name');
  const raw=object(value,obj.ctx),mode=raw.props.has('mode')?literal(raw.props.get('mode'),raw.ctx):'single';if(!modes.has(mode))fail(raw.ctx,value,'rich-named-target-mode');
  api.knownProperties(raw,['mode','cardTypeFilter','controller','zoneFilter','cardCondition','optional','numCards','canChooseNoCards','immediateEffect','choosingPlayer','dependsOn',...EXTRA_FIELDS]);
  let dependsOn;if(raw.props.has('dependsOn')){dependsOn=literal(raw.props.get('dependsOn'),raw.ctx);if(!slots.has(dependsOn)||dependents.has(dependsOn))fail(raw.ctx,value,'rich-named-target-dependency');dependents.add(dependsOn);}
  const array=!['single','player'].includes(mode),scope=new Map(slots);scope.set(name,{array});
  const ctx={...raw.ctx,pendingSelection:true,namedTargets:scope},props=new Map(raw.props);props.delete('dependsOn');
  const resolver=richResolver({...raw,props,ctx},api,true);if(!resolver)fail(ctx,value,'rich-named-target-resolver');resolver.resultBinding=name;
  const entry={name,targetResolver:resolver,chooser:resolver.chooser,...(dependsOn?{dependsOn}:{})};
  if(raw.props.has('immediateEffect'))entry.effect=effect(raw.props.get('immediateEffect'),{...ctx,...(resolver.kind==='playerSelection'?{selectedPlayerTarget:{kind:'ref',path:['namedTargets',name]}}:{})},'chosen');
  entries.push(entry);slots.set(name,{array});
 }
 return {entries,ctx:{...obj.ctx,pendingSelection:true,namedTargets:slots}};
}
module.exports={EXTRA_FIELDS,richResolver,richNamedResolvers,selectedPlayerEffect,selectionExpression};
