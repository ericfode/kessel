'use strict';

function randomResourceValue(node,ctx,api){
 const {ts,expr,fail}=api;
 if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.name.text!=='getRandomResources')return null;
 if(node.questionDotToken||node.expression.questionDotToken||node.typeArguments?.length||node.arguments.length!==2||!ctx.callbackStage)fail(ctx,node,'random-resources-call-shape');
 const receiver=expr(node.expression.expression,{...ctx,arenaReceiver:true}),arg=node.arguments[0],context=ts.isIdentifier(arg)&&!ctx.constants.has(arg.text)?ctx.scope.get(arg.text):null;
 if(receiver.kind!=='ref'||!['player','opponent','source.controller','source.controller.opponent'].includes(receiver.path.join('.'))||!context||context.kind!=='ref'||context.path.join('.')!=='context')fail(ctx,node,'random-resources-receiver-or-context');
 return {kind:'randomResourcesMutation',receiver,context,amount:expr(node.arguments[1],ctx),evaluation:'eachSourceCall',steps:[{kind:'captureExhaustedResourceCount'},{kind:'readyAllResources'},{kind:'shuffleResourceOrder',algorithm:'durstenfeld',randomSource:'contextGameRandomGenerator',draws:'onePerIndexDescendingFromLengthMinusOneThroughOne'},{kind:'replaceResourceZoneOrder'},{kind:'exhaustFirstResources',amount:'capturedExhaustedResourceCount'},{kind:'returnFirstResources',count:'argumentAmount',operation:'spliceOnFilteredCopy'}],resourceIdentityAndCountPreserved:true};
}
module.exports={randomResourceValue};
