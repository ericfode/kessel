'use strict';

function isCall(input,ctx,{ts,unwrap}) {
 const n=unwrap(input);
 return ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.expression.kind===ts.SyntaxKind.ThisKeyword&&ctx.privatePrograms?.get(n.expression.name.text)?.returnRole==='ability'&&!ctx.privatePrograms.get(n.expression.name.text).objectTemplate;
}
function ownProperties(input,ctx,api) {
 const {ts,unwrap}=api,n=unwrap(input);
 if(!ts.isObjectLiteralExpression(n))return null;
 if(n.properties.some(ts.isSpreadAssignment))return {kind:'constructedSourceOwnProperties',from:'abilityObjectConstruction'};
 return n.properties.map(p=>ts.isComputedPropertyName(p.name)?api.literal(p.name.expression,ctx):p.name.text);
}
// Keep source object spreads separate from normalized ability defaults. An
// omitted title in a helper's result must not erase an earlier title property.
function merge(input,ctx,trigger,api) {
 const {ts,unwrap,fail,expr,literal,contextTitle,ability}=api,n=unwrap(input);
 if(!ts.isObjectLiteralExpression(n)||!n.properties.some(p=>ts.isSpreadAssignment(p)&&isCall(p.expression,ctx,api)))return null;
 const entries=n.properties.map(p=>{
  if(ts.isSpreadAssignment(p)){
   if(!isCall(p.expression,ctx,api))fail(ctx,p,'ability-object-spread-source');
   return {kind:'abilityPropertiesSpread',value:ability(p.expression,ctx)};
  }
  if(!ts.isPropertyAssignment(p)||(!ts.isIdentifier(p.name)&&!ts.isStringLiteral(p.name)))fail(ctx,p,'ability-object-property-form');
  const key=p.name.text;
  if(key==='title')return {kind:'abilityOwnProperty',name:key,value:{kind:'displayTextValue',parts:contextTitle(p.initializer,ctx)}};
  if(['optional','optionalMakesAbilityNonCancelable'].includes(key)){const value=literal(p.initializer,ctx);if(typeof value!=='boolean')fail(ctx,p,'ability-object-boolean');return {kind:'abilityOwnProperty',name:key,value:{kind:'const',value}};}
  if(['condition','thenCondition','ifYouDoCondition'].includes(key))return {kind:'abilityOwnProperty',name:key,value:expr(p.initializer,ctx)};
  if(['then','ifYouDo','ifYouDoNot'].includes(key))return {kind:'abilityOwnProperty',name:key,value:ability(p.initializer,ctx)};
  fail(ctx,p,'ability-object-explicit-property:'+key);
 });
 return {kind:'abilityObjectConstruction',...(trigger?{trigger}:{}),entries,evaluation:'leftToRightOnce',spread:'ownEnumerableSourceProperties',duplicates:'lastPropertyWins',normalization:'applyAbilityDefaultsAfterAllSourceProperties',validation:'eachFragmentUsesOrdinaryAbilityParser'};
}
module.exports={isCall,ownProperties,merge};
