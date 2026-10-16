'use strict';

// Limits are identity-bearing constructed objects, rather than numeric values.
// In particular an unlimited limit still records usage, and perGame is shared
// across players for that one constructed instance.
function limitFactoryValue(input,ctx,api){
 const {ts,unwrap,literal,fail}=api,n=unwrap(input);
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||!['perGame','perPlayerPerGame','unlimited'].includes(n.expression.name.text))return null;
 const receiver=n.expression.expression;
 if(!ts.isPropertyAccessExpression(receiver)||receiver.name.text!=='limit'||!ts.isIdentifier(receiver.expression))return null;
 const root=receiver.expression.text,method=n.expression.name.text;
 if(!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(root))return null;
 if(n.questionDotToken||n.expression.questionDotToken||receiver.questionDotToken||n.typeArguments?.length||n.arguments.length!==(method==='unlimited'?0:1)||ctx.constants.has(root))fail(ctx,n,'limit-factory-shape');
 let bound=false;
 for(let p=n.parent;p;p=p.parent){if(!p.parameters)continue;const parameter=p.parameters.find(x=>ts.isIdentifier(x.name)&&x.name.text===root);if(!parameter)continue;bound=parameter.type?.getText(ctx.source)==='IAbilityHelper'&&!parameter.initializer&&!parameter.dotDotDotToken&&!parameter.questionToken;break;}
 if(!bound)fail(ctx,n,'limit-factory-helper-binding');
 const result={kind:'abilityLimitFactory',factory:method,allocation:'eachSourceExpressionEvaluation',gameBinding:'helperBoundGame',lifetime:'constructedObjectInstance',reset:'zeroUsage',clone:'newInstanceWithZeroUsage',usageTracking:method==='perGame'?'singleCounterSharedAcrossPlayers':'mapByPlayerName',isRepeatable:method==='unlimited',isAtMax:method==='unlimited'?'alwaysFalse':method==='perPlayerPerGame'?'currentForPlayerAtLeastModifiedMax':'usageAtLeastMax'};
 if(method!=='unlimited'){const max=literal(n.arguments[0],ctx);if(!Number.isSafeInteger(max)||max<1||max>64)fail(ctx,n,'limit-factory-max-bound');result.max=max;}
 if(method==='perPlayerPerGame')result.modifiedMax='abilityCardGetModifiedAbilityLimitMaxOtherwisePrintedMax';
 return result;
}
function limitInstanceReference(input,ctx,api){
 const {ts,unwrap}=api;let n=unwrap(input),local=ctx;const seen=new Set(),aliases=[];
 while(ts.isIdentifier(n)&&local.constants?.has(n.text)){
  const initializer=local.constants.get(n.text);if(seen.has(initializer))api.fail(ctx,n,'shared-limit-alias-cycle');seen.add(initializer);aliases.push(initializer);
  local={...local,scope:ctx.constantScopes?.get(initializer)||local.scope,constants:ctx.constantEnvs?.get(initializer)||local.constants};n=unwrap(initializer);
 }
 if(!aliases.length)return null;
 const definition=limitFactoryValue(n,local,api);if(!definition)return null;
 // Construction captures live elsewhere; this contract owns readonly setup
 // declarations and all later lexical reads of that exact object.
 const declaration=aliases.at(-1),stage=ctx.constantStages?.get(declaration);
 if(stage)api.fail(ctx,input,'shared-limit-requires-setup-declaration');
 for(const initializer of aliases)ctx.usedConstants?.add(initializer);
 return {kind:'readonlyLimitReference',binding:'limit:'+declaration.getStart(ctx.source),owner:'lexicalCardSetupInvocation',definition,allocation:'originalReadonlyDeclarationEvaluationOnce',read:'sameObjectIdentity'};
}
function limitObjectExpression(input,ctx,api){
 const {ts,unwrap,expr,fail}=api,n=unwrap(input);
 const reference=limitInstanceReference(n,ctx,api);if(reference)return reference;
 if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken&&n.name.text==='max'){
  const receiver=limitInstanceReference(n.expression,ctx,api);if(receiver){if(receiver.definition.factory==='unlimited')fail(ctx,n,'unlimited-limit-has-no-max');return {kind:'limitInstanceProperty',receiver,property:'max',semantics:'printedConstructorMax'};}
 }
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='currentForPlayer'){
  const receiver=limitInstanceReference(n.expression.expression,ctx,api);if(!receiver)return null;
  const perGame=receiver.definition.factory==='perGame';if(n.questionDotToken||n.expression.questionDotToken||n.typeArguments?.length||n.arguments.length!==(perGame?0:1))fail(ctx,n,'limit-current-for-player-shape');
  return {kind:'limitInstanceQuery',receiver,method:'currentForPlayer',args:n.arguments.map(a=>expr(a,ctx)),missingPlayerUsage:0,tracking:receiver.definition.usageTracking};
 }
 return null;
}
function sharedActivationLimit(input,ctx,api){
 const instance=limitInstanceReference(input,ctx,api);if(!instance)return null;
 return {kind:'abilityLimitReference',instance,sharedAcrossHooks:true,registrationUsesExistingObject:true};
}
module.exports={limitFactoryValue,limitInstanceReference,limitObjectExpression,sharedActivationLimit};
