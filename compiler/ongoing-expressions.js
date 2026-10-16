'use strict';
const privatePrograms=require('./private-programs');
function ongoingExpression(node,ctx,api) {
 const {ts,expr,fail}=api;
 if((ctx.ongoingExpression||ctx.explicitPrintedQuery)&&ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&['getPrintedPower','getPrintedHp','getPrintedTraits'].includes(node.expression.name.text)){if(node.arguments.length)fail(ctx,node,'printed-card-query-arity');return {kind:'printedCardQuery',name:node.expression.name.text,receiver:expr(node.expression.expression,ctx),args:[],lookup:'lastNonNullPrintedOverrideThenIntrinsic',traitsResult:'freshSet',optionalReceiver:!!node.expression.questionDotToken,optionalCall:!!node.questionDotToken};}
 if(!ctx.ongoingExpression)return null;
 if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)) {
  const call=node.expression,name=call.name.text;
  if(name==='toProperties'){if(node.arguments.length)fail(ctx,node,'keyword-properties-arity');return {kind:'keywordInstanceProperties',receiver:expr(call.expression,ctx),dispatch:'runtimeKeywordSubtype',abilityDefinition:'requireImplementedAbilityProperties',optionalReceiver:!!call.questionDotToken,optionalCall:!!node.questionDotToken};}
  if(name==='duplicate'){if(node.arguments.length!==1)fail(ctx,node,'keyword-duplicate-arity');return {kind:'keywordInstanceDuplicate',receiver:expr(call.expression,ctx),card:expr(node.arguments[0],ctx),retain:['name','numericValue','cost','aspects','additionalCosts'],abilityDefinition:'clearAndRequireRegistrarSetup',optionalReceiver:!!call.questionDotToken,optionalCall:!!node.questionDotToken};}
  if(['some','every','filter','map','flatMap','reduce'].includes(name))return {kind:'ongoingCollection',name,receiver:expr(call.expression,ctx),args:node.arguments.map(n=>(ts.isArrowFunction(n)||ts.isFunctionExpression(n))?privatePrograms.compileCallback(n,ctx,api,name==='reduce'?['accumulator','candidate','index','collection']:['candidate','index','collection']):expr(n,ctx)),optionalReceiver:!!call.questionDotToken,optionalCall:!!node.questionDotToken,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)};
  if(['getLeaderCards','hasSomeLeaderCard','isKeywordInPlay','controlsLeaderUnitOrUpgradeWithTitle','canBeInPlay','getPlayers','canAttack','isAttacking','isDefending','getArenaUnits','hasSomeArenaUnit','hasSomeKeyword','hasEveryKeyword','getNumericKeywordTotal','getPower','getHp','isUnit','isBase'].includes(name))return {kind:'ongoingQuery',name,receiver:expr(call.expression,ctx),args:node.arguments.map(n=>api.value(n,ctx)),optionalReceiver:!!call.questionDotToken,optionalCall:!!node.questionDotToken,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)};
  let p=[];try{p=api.access(call,ctx);}catch(e){if(!e.diagnostic)throw e;}
  if(p.length===3&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])&&p[1]==='costs'){const cost=api.additionalCost(node,p[2],ctx,api)||api.costs(node,ctx)[0];if(cost)return {kind:'costConstruction',cost};}
 }
 if(ts.isPropertyAccessExpression(node)&&(node.questionDotToken||ts.isCallExpression(node.expression)||ts.isNewExpression(node.expression)))return {kind:'ongoingMember',receiver:expr(node.expression,ctx),member:node.name.text,optionalReceiver:!!node.questionDotToken,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)};
 return null;
}
module.exports={ongoingExpression};
