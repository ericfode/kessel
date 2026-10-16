'use strict';
const {bindAttackCallback}=require('./attack-initiation');

// Bounded extension of the existing callback/capture contract. The source
// array is built completely before any child system is resolved.
function systemArrayFactory(input,context,mode,defaultTarget,api) {
    const {ts,unwrap,expr,effect,literal,access,fail}=api,fn=unwrap(input);
    if(!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))return null;
    const callback=bindAttackCallback(fn,context,['systemFactoryContext'],api);
    const array=unwrap(callback.lowered.node);
    if(!ts.isArrayLiteralExpression(array))return null;
    if(!array.elements.length||array.elements.length>16)fail(context,array,'system-factory-array-bound');
    const ctx={...callback.lowered.ctx,pendingSystemConstruction:true,scope:new Map(callback.lowered.ctx.scope),constants:new Map(callback.lowered.ctx.constants),constructionValues:new Map(context.constructionValues||[])};
    const captures=[];
    const capture=(node,role)=>{
        const n=unwrap(node),existing=ctx.constructionValues.get(n);if(existing)return existing;
        const value=expr(n,{...ctx,allowArenaList:true});
        const name='systemCapture'+fn.getStart(context.source)+'_'+captures.length,reference={kind:'ref',path:['systemFactoryCaptures',name]};
        captures.push({name,role,value,evaluateAt:'arrayFactoryInvocation',order:captures.length});ctx.constructionValues.set(n,reference);return reference;
    };
    // Readonly declarations preserve value or physical-object identity once,
    // rather than re-evaluating their initializer after an earlier child.
    for(const binding of ctx.bindings||[]) {
        if(ctx.constantStages?.get(binding.node)!==fn||ctx.constants.get(binding.name)!==binding.node)continue;
        const n=unwrap(binding.node);
        if(ts.isArrowFunction(n)||ts.isFunctionExpression(n)||ts.isObjectLiteralExpression(n))fail(ctx,n,'system-factory-local-object-or-function');
        const value=capture(n,'readonlyDeclaration');ctx.scope.set(binding.name,value);ctx.constants.delete(binding.name);ctx.usedConstants.add(binding.node);
    }
    const walk=(input,depth=0)=>{
        if(depth>32)fail(ctx,input,'system-factory-construction-depth');
        const n=unwrap(input);
        if(ts.isArrowFunction(n)||ts.isFunctionExpression(n))return; // deferred callback body
        if(ts.isArrayLiteralExpression(n)){for(const element of n.elements)walk(element,depth+1);return;}
        if(ts.isCallExpression(n)) {
            const parts=access(n.expression,ctx);
            if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||!['immediateEffects','ongoingEffects'].includes(parts[1]))fail(ctx,n,'system-factory-constructor-call');
            if(n.questionDotToken||n.arguments.length>1)fail(ctx,n,'system-factory-constructor-arity');
            if(!n.arguments.length)return;
            const properties=unwrap(n.arguments[0]);
            if(ts.isArrowFunction(properties)||ts.isFunctionExpression(properties))fail(ctx,properties,'system-factory-nested-property-factory');
            if(ts.isObjectLiteralExpression(properties)||ts.isArrayLiteralExpression(properties))walk(properties,depth+1);
            else {try{literal(properties,ctx);}catch(error){if(!error.diagnostic)throw error;fail(ctx,properties,'system-factory-nonconstant-constructor-argument');}}
            return;
        }
        if(!ts.isObjectLiteralExpression(n))fail(ctx,n,'system-factory-construction-object');
        for(const property of n.properties) {
            if(!ts.isPropertyAssignment(property))fail(ctx,property,'system-factory-property-form');
            const value=unwrap(property.initializer);
            if(ts.isArrowFunction(value)||ts.isFunctionExpression(value))continue;
            if(ts.isObjectLiteralExpression(value)){walk(value,depth+1);continue;}
            if(ts.isCallExpression(value)&&ts.isPropertyAccessExpression(value.expression)&&['immediateEffects','ongoingEffects'].includes(value.expression.expression?.name?.text)){walk(value,depth+1);continue;}
            if(ts.isArrayLiteralExpression(value)&&value.elements.some(e=>ts.isCallExpression(unwrap(e)))){walk(value,depth+1);continue;}
            try{literal(value,ctx);}catch(error){if(!error.diagnostic)throw error;capture(value,'staticProperty');}
        }
    };
    array.elements.forEach(element=>walk(element));
    const effects=array.elements.map(element=>effect(element,ctx,defaultTarget));
    return {kind:'systemArrayFactory',mode,parameters:callback.parameters,captures,constructionOrder:'readonlyDeclarationsThenArrayLeftToRight',captureIdentity:'valueOrPhysicalObjectReference',callbacks:'deferredWithLexicalBindings',invocationPolicy:'eachNativePropertyGeneration',resolutionMode:'someGameSystemsMustBeLegal',effects};
}
module.exports={systemArrayFactory};
