'use strict';
const privatePrograms=require('./private-programs');

// DamageSystem invokes a function-valued amount with the actual event card.
// It is not an AbilityContext lambda or a snapshot of the selected target.
function nativeDamageRecipientAmount(input,context,api) {
 const {ts,unwrap,fail}=api,n=unwrap(input);
 if(!ts.isArrowFunction(n)&&!ts.isFunctionExpression(n))fail(context,n,'damage-recipient-function-required');
 if(n.parameters.length>1)fail(context,n,'damage-recipient-function-arity');
 if(context.constructionFactorySuppressed&&!context.pendingSystemConstruction)fail(context,n,'damage-recipient-requires-properties-frame');
 const callback=privatePrograms.compileCallback(n,{...context,damageRecipientScope:false,eventHistory:true},api,['damagedCard']);
 return {kind:'nativeDamageRecipientAmount',callback,evaluateAt:'addAbilityDamagePropertiesToEachEvent',arguments:['actualEventRecipient'],capture:'lexicalValuesAndPhysicalObjectIdentity',propertyFactory:'eachNativePropertyGeneration',functionAllocation:'oncePerReadonlyInitializerOrInlineExpressionAtPropertyGeneration',legality:{ordinary:'doNotInvokeAmountForZeroCheck',costOrRequiredStateChange:'assertRecipientUnitThenInvokeAndRejectStrictlyZero',subsequentZeroCheck:'compareFunctionObjectToNumericZero',propertiesSource:'generatePropertiesFromContextWithoutAdditionalProperties',requireAttackableZone:true,superclassChecks:'CardTargetSystem'},eventAmount:{store:'callbackResultAtEventGeneration',pendingAbilityDamageIncrease:'addIfPositiveUnlessIgnoreAbilityDamageIncrease',normalization:'nativeNumberNoExtraClampOrRounding'},eventResolution:'useStoredEventAmount'};
}
// Resolve only readonly aliases to function expressions. This is constructor
// value classification; it never invokes the function body as a context lambda.
function readonlyFunctionValue(input,context,api) {
 const {ts,unwrap,fail}=api;let n=unwrap(input),ctx=context;const seen=new Set();
 while(ts.isIdentifier(n)&&ctx.constants.has(n.text)) {
  const initializer=ctx.constants.get(n.text);if(seen.has(initializer))fail(ctx,n,'readonly-function-alias-cycle');seen.add(initializer);ctx.usedConstants?.add(initializer);
  ctx={...ctx,scope:ctx.constantScopes?.get(initializer)||ctx.scope,constants:ctx.constantEnvs?.get(initializer)||ctx.constants};n=unwrap(initializer);
 }
 return ts.isArrowFunction(n)||ts.isFunctionExpression(n)?{node:n,ctx}:null;
}
module.exports={nativeDamageRecipientAmount,readonlyFunctionValue};
