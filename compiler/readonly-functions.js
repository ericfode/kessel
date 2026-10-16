'use strict';

// A call through a readonly source-local function keeps the function's lexical
// environment and an explicit argument frame. The callee body still passes the
// finite pure/construction program compiler; arbitrary invocation is rejected.
function readonlyFunctionCall(input,ctx,api) {
 const {ts,unwrap,expr,readConstant,constantContext,fail}=api,n=unwrap(input);
 if(ts.isPropertyAccessExpression(n)&&n.name.text==='length'){const receiver=readonlyFunctionCall(n.expression,ctx,api);if(receiver)return {kind:'readonlyFunctionResultMember',receiver,member:'length',optional:!!n.questionDotToken,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain),evaluation:'getPropertyAfterCall'};}
 if(!ts.isCallExpression(n)||!ts.isIdentifier(n.expression)||!ctx.constants.has(n.expression.text))return null;
 const initializer=readConstant(n.expression.text,ctx),fn=unwrap(initializer);
 if(!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))return null;
 if(n.questionDotToken||n.arguments.length>16)fail(ctx,n,'readonly-function-call-shape');
 if(ctx.readonlyFunctionTrail?.has(fn)||(ctx.readonlyFunctionTrail?.size||0)>16)fail(ctx,n,'readonly-function-recursion');
 const lexical={...constantContext(initializer,ctx),readonlyFunctionTrail:new Set([...(ctx.readonlyFunctionTrail||[]),fn])};
 const callback=require('./private-programs').compileCallback(fn,lexical,api,fn.parameters.map((_,i)=>'argument'+i));
 const args=n.arguments.map(a=>{
  const v=unwrap(a),binding=ts.isIdentifier(v)&&ctx.scope.get(v.text);
  if(binding?.kind==='ref'&&binding.path.join('.')==='context')return {kind:'abilityContextValue',binding:require('./event-result-context').contextBinding(v,ctx,api)};
  if(binding?.kind==='ref'&&ctx.typedCallbackRoles?.get(binding.path[0])?.toLowerCase().includes('context'))return {kind:'abilityContextValue',value:binding,reference:'lexicalTypedContextObject'};
  return expr(a,ctx);
 });
 return {kind:'readonlyFunctionCall',binding:'readonlyFunction:'+fn.getStart(ctx.source),creation:'sourceReadonlyDeclaration',callback,args,argumentEvaluation:'leftToRightBeforeInvocation',invokeAt:'sourceCallExpression'};
}
module.exports={readonlyFunctionCall};
