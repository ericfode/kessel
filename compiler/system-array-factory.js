'use strict';
const {isSpecializedFence}=require('./unsupported-recovery');
const {ongoingModifierConstruction}=require('./ongoing-generator-construction');
const {bindAttackCallback}=require('./attack-initiation');
const {readonlyFunctionValue}=require('./damage-recipient-amount');

// Bounded extension of the existing callback/capture contract. The source
// array is built completely before any child system is resolved.
function constructionFrame(input,context,roles,evaluateAt,api) {
    const {ts,unwrap,expr,literal,access,fail}=api,fn=unwrap(input);
    const callback=bindAttackCallback(fn,context,roles,api);
    const ctx={...callback.lowered.ctx,pendingSystemConstruction:true,scope:new Map(callback.lowered.ctx.scope),constants:new Map(callback.lowered.ctx.constants),constructionValues:new Map(context.constructionValues||[])};
    const captures=[];
    const capture=(node,role,constructedValue)=>{
        const n=unwrap(node),existing=ctx.constructionValues.get(n);if(existing)return existing;
        const value=constructedValue===undefined?expr(n,{...ctx,allowArenaList:true}):constructedValue;
        const name='systemCapture'+fn.getStart(context.source)+'_'+captures.length,reference={kind:'ref',path:['systemFactoryCaptures',name]};
        captures.push({name,role,value,evaluateAt,order:captures.length});ctx.constructionValues.set(n,reference);return reference;
    };
    // Readonly declarations preserve value or physical-object identity once,
    // rather than re-evaluating their initializer after an earlier child.
    for(const binding of ctx.bindings||[]) {
        if(ctx.constantStages?.get(binding.node)!==fn||ctx.constants.get(binding.name)!==binding.node)continue;
        const n=unwrap(binding.node);
        if(systemSyntax(n,ctx,api)) {
            const value=capture(n,'readonlySystemDeclaration',systemConstructionValue(n,ctx,ctx.constructionDefaultTarget||'self',api));
            ctx.constructedSystemBindings=new Map(ctx.constructedSystemBindings||[]);ctx.constructedSystemBindings.set(binding.name,value);
            ctx.scope.set(binding.name,value);ctx.constants.delete(binding.name);ctx.usedConstants.add(binding.node);continue;
        }
        if(readonlyFunctionValue(n,ctx,api)){ctx.usedConstants.add(binding.node);continue;}
        if(ts.isObjectLiteralExpression(n))fail(ctx,n,'system-factory-local-object-or-function');
        const value=capture(n,'readonlyDeclaration');ctx.scope.set(binding.name,value);ctx.constants.delete(binding.name);ctx.usedConstants.add(binding.node);
    }
    const walk=(input,depth=0)=>{
        if(depth>32)fail(ctx,input,'system-factory-construction-depth');
        const n=unwrap(input);
        if(ts.isIdentifier(n)&&ctx.constructedSystemBindings?.has(n.text))return;
        if(ts.isArrowFunction(n)||ts.isFunctionExpression(n))return; // deferred callback body
        if(ts.isArrayLiteralExpression(n)){for(const element of n.elements)walk(element,depth+1);return;}
        if(ts.isCallExpression(n)) {
            const parts=access(n.expression,ctx);
            if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||!['immediateEffects','ongoingEffects'].includes(parts[1]))fail(ctx,n,'system-factory-constructor-call');
            if(n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'system-factory-constructor-optional');
            if(parts[1]==='ongoingEffects') {
                if(n.arguments.length>8)fail(ctx,n,'system-factory-ongoing-arity');
                for(const arg of n.arguments){const value=unwrap(arg);if(ts.isArrowFunction(value)||ts.isFunctionExpression(value))continue;if(ts.isObjectLiteralExpression(value)||ts.isArrayLiteralExpression(value)){walk(value,depth+1);continue;}try{literal(value,ctx);}catch(error){if(!error.diagnostic)throw error;capture(value,'staticOngoingArgument');}}return;
            }
            if(n.arguments.length>1)fail(ctx,n,'system-factory-constructor-arity');
            if(!n.arguments.length)return;
            const properties=unwrap(n.arguments[0]);
            if(ts.isArrowFunction(properties)||ts.isFunctionExpression(properties))return; // constructed closure; body runs later
            if(ts.isObjectLiteralExpression(properties)||ts.isArrayLiteralExpression(properties))walk(properties,depth+1);
            else if(['simultaneous','sequential'].includes(parts[2])&&(ts.isCallExpression(properties)||ts.isConditionalExpression(properties)||ts.isBinaryExpression(properties)&&properties.operatorToken.kind===ts.SyntaxKind.QuestionQuestionToken))return;
            else {try{literal(properties,ctx);}catch(error){if(!error.diagnostic)throw error;fail(ctx,properties,'system-factory-nonconstant-constructor-argument');}}
            return;
        }
        if(!ts.isObjectLiteralExpression(n))fail(ctx,n,'system-factory-construction-object');
        for(const property of n.properties) {
            if(!ts.isPropertyAssignment(property)&&!ts.isShorthandPropertyAssignment(property))fail(ctx,property,'system-factory-property-form');
            const value=unwrap(ts.isShorthandPropertyAssignment(property)?property.name:property.initializer);
            if(readonlyFunctionValue(value,ctx,api))continue;
            if(ts.isObjectLiteralExpression(value)){walk(value,depth+1);continue;}
            if(ts.isCallExpression(value)&&ts.isPropertyAccessExpression(value.expression)&&['immediateEffects','ongoingEffects'].includes(value.expression.expression?.name?.text)){walk(value,depth+1);continue;}
            if(ts.isArrayLiteralExpression(value)&&value.elements.some(e=>ts.isCallExpression(unwrap(e))||ts.isObjectLiteralExpression(unwrap(e)))){walk(value,depth+1);continue;}
            try{literal(value,ctx);}catch(error){if(!error.diagnostic)throw error;capture(value,'staticProperty');}
        }
    };
    return {node:unwrap(callback.lowered.node),parameters:callback.parameters,ctx,captures,walk,capture};
}
function systemArrayFactory(input,context,mode,defaultTarget,api) {
    const {ts,unwrap,effect,fail}=api,fn=unwrap(input);
    if(!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))return null;
    let frame;try{frame=constructionFrame(fn,{...context,constructionDefaultTarget:defaultTarget},['systemFactoryContext'],'arrayFactoryInvocation',api);}catch(error){if(!error.diagnostic)throw error;if(isSpecializedFence(error))throw error;const callback=require('./private-programs').compileCallback(fn,{...context,pendingSystemConstruction:true,constructionProgram:true},api,['systemFactoryContext'],(value,local)=>({kind:'validatedSystemArray',value:systemConstructionValue(value,local,defaultTarget,api)}));return {kind:'systemArrayProgram',mode,callback,invocationPolicy:'eachNativePropertyGeneration',resolutionMode:'someGameSystemsMustBeLegal'};}
    const array=frame.node;
    if(!ts.isArrayLiteralExpression(array))return {kind:'systemArrayFactory',mode,parameters:frame.parameters,captures:frame.captures,constructionOrder:'readonlyDeclarationsThenReturnedConstruction',captureIdentity:'valueOrPhysicalObjectReference',callbacks:'deferredWithLexicalBindings',invocationPolicy:'eachNativePropertyGeneration',resolutionMode:'someGameSystemsMustBeLegal',systems:systemConstructionValue(array,frame.ctx,defaultTarget,api)};
    if(!array.elements.length||array.elements.length>16)fail(context,array,'system-factory-array-bound');
    array.elements.forEach(element=>frame.walk(element));
    const effects=array.elements.map(element=>effect(element,frame.ctx,defaultTarget));
    return {kind:'systemArrayFactory',mode,parameters:frame.parameters,captures:frame.captures,constructionOrder:'readonlyDeclarationsThenArrayLeftToRight',captureIdentity:'valueOrPhysicalObjectReference',callbacks:'deferredWithLexicalBindings',invocationPolicy:'eachNativePropertyGeneration',resolutionMode:'someGameSystemsMustBeLegal',effects};
}
function systemPropertiesFactory(call,context,defaultTarget,api) {
    const {ts,unwrap,effect,fail}=api;
    if(context.constructionFactorySuppressed===call||call.arguments.length!==1)return null;
    const fn=unwrap(call.arguments[0]);if(!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))return null;
    if(!context.pendingSystemConstruction) {
        try{return effect(call,{...context,constructionFactorySuppressed:call},defaultTarget);}
        catch(error){if(!error.diagnostic)throw error;if(isSpecializedFence(error))throw error;}
    }
    let frame;try{frame=constructionFrame(fn,{...context,constructionDefaultTarget:defaultTarget},['systemPropertiesContext'],'propertyFactoryInvocation',api);}catch(error){if(!error.diagnostic)throw error;if(isSpecializedFence(error))throw error;const program=require('./private-programs').compileSystemProperties(call,context,defaultTarget,api);if(program)return program;throw error;}
    if(!ts.isObjectLiteralExpression(frame.node))fail(frame.ctx,frame.node,'system-properties-factory-object');
    frame.walk(frame.node);
    const bound=ts.factory.updateCallExpression(call,call.expression,call.typeArguments,[frame.node]);
    return {kind:'systemPropertiesFactory',parameters:frame.parameters,captures:frame.captures,constructionOrder:'readonlyDeclarationsThenObjectLeftToRight',captureIdentity:'valueOrPhysicalObjectReference',invocationPolicy:'eachNativePropertyGeneration',effect:effect(bound,frame.ctx,defaultTarget)};
}

// A closed construction vocabulary: this describes allocation of validated DSL
// systems. It never executes or serializes arbitrary source expressions.
function systemSyntax(input,ctx,api) {
    const {ts,unwrap,access}=api,n=unwrap(input);
    if(ts.isIdentifier(n))return ctx.constructedSystemBindings?.has(n.text)||false;
    if(ts.isConditionalExpression(n))return systemSyntax(n.whenTrue,ctx,api)&&systemSyntax(n.whenFalse,ctx,api);
    if(ts.isCallExpression(n)){let p;try{p=access(n.expression,ctx);}catch(error){if(!error.diagnostic)throw error;return false;}return p.length===3&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])&&p[1]==='immediateEffects';}
    return false;
}
function systemConstructionValue(input,ctx,defaultTarget,api) {
    const {ts,unwrap,expr,effect,fail}=api,n=unwrap(input);
    if(ts.isIdentifier(n)&&ctx.constructedSystemBindings?.has(n.text))return {kind:'constructedSystemReference',reference:ctx.constructedSystemBindings.get(n.text)};
    if(ts.isArrayLiteralExpression(n)) {
        if(n.elements.length>128)fail(ctx,n,'system-construction-array-bound');
        return {kind:'constructedSystemArray',evaluationOrder:'leftToRight',items:n.elements.map(e=>systemConstructionValue(e,ctx,defaultTarget,api))};
    }
    if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.QuestionQuestionToken)return {kind:'nullishSystemConstruction',left:systemConstructionValue(n.left,ctx,defaultTarget,api),right:systemConstructionValue(n.right,ctx,defaultTarget,api),evaluation:'leftOnceThenRightOnlyIfNullOrUndefined'};
    if(ts.isConditionalExpression(n))return {kind:'conditionalSystemConstruction',condition:expr(n.condition,ctx),whenTrue:systemConstructionValue(n.whenTrue,{...ctx,constructionValues:new Map(ctx.constructionValues||[])},defaultTarget,api),whenFalse:systemConstructionValue(n.whenFalse,{...ctx,constructionValues:new Map(ctx.constructionValues||[])},defaultTarget,api),evaluation:'conditionThenChosenBranchOnly'};
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&ts.isIdentifier(n.expression.expression)&&n.expression.expression.text==='Array'&&n.expression.name.text==='from'){
        if(ctx.scope.has('Array')||ctx.constants.has('Array')||n.arguments.length!==2||n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'system-array-from-shape');
        const frame=constructionFrame(n.arguments[1],ctx,['systemElement','systemIndex'],'arrayFromCallbackInvocation',api);
        return {kind:'systemArrayFromConstruction',receiver:expr(n.arguments[0],{...ctx,pendingSystemConstruction:true}),parameters:frame.parameters,captures:frame.captures,value:systemConstructionValue(frame.node,frame.ctx,defaultTarget,api),receiverEvaluation:'onceBeforeCallbacks',iteration:'nativeArrayFromIteratorOrArrayLikeToLength',callbackArguments:['element','index']};
    }
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['map','flatMap'].includes(n.expression.name.text)) {
        if(n.arguments.length!==1)fail(ctx,n,'system-construction-map-arity');
        const frame=constructionFrame(n.arguments[0],ctx,['systemElement','systemIndex','systemCollection'],'collectionCallbackInvocation',api);
        return {kind:'systemCollectionConstruction',operation:n.expression.name.text,receiver:expr(n.expression.expression,{...ctx,allowArenaList:true}),optionalCall:!!n.questionDotToken,optionalReceiver:!!n.expression.questionDotToken,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain),iteration:'nativeArrayOrderAndHoles',receiverEvaluation:'onceBeforeCallbacks',parameters:frame.parameters,captures:frame.captures,value:systemConstructionValue(frame.node,frame.ctx,defaultTarget,api),callbacks:'lexicalBindingsSurviveConstruction',flattenDepth:n.expression.name.text==='flatMap'?1:0};
    }
    // Individual helper parsers retain responsibility for their complete fields
    // and native defaults. Static properties are captured at this expression's
    // evaluation; callback bodies remain deferred by those parsers.
    const local={...ctx,pendingSystemConstruction:true,constructionValues:new Map(ctx.constructionValues||[])};
    const captures=[];
    const walk=(v,depth=0)=>{
        if(depth>32)fail(local,v,'system-construction-depth');v=unwrap(v);
        if(ts.isArrowFunction(v)||ts.isFunctionExpression(v))return;
        if(ts.isCallExpression(v)){
            const p=api.access(v.expression,local);
            if(!systemSyntax(v,local,api)&&!(p.length===3&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])&&p[1]==='ongoingEffects'))return;
            for(const a of v.arguments)walk(a,depth+1);return;
        }
        if(ts.isArrayLiteralExpression(v)){for(const e of v.elements)walk(e,depth+1);return;}
        if(!ts.isObjectLiteralExpression(v))return;
        for(const p of v.properties){if(!ts.isPropertyAssignment(p))fail(local,p,'system-construction-property');const value=unwrap(p.initializer);
            if(readonlyFunctionValue(value,ctx,api))continue;
            if(ts.isObjectLiteralExpression(value)||ts.isArrayLiteralExpression(value)||systemSyntax(value,local,api)||ts.isCallExpression(value)&&ts.isPropertyAccessExpression(value.expression)&&value.expression.expression?.name?.text==='ongoingEffects'){walk(value,depth+1);continue;}
            try{api.literal(value,local);}catch(error){if(!error.diagnostic)throw error;const name='construction'+n.getStart(ctx.source)+'_'+captures.length;captures.push({name,value:expr(value,{...local,allowArenaList:true}),evaluateAt:'constructorArgumentEvaluation',order:captures.length});local.constructionValues.set(value,{kind:'ref',path:['systemFactoryCaptures',name]});}
        }
    };walk(n);
    return {kind:'systemConstruction',captures,effect:effect(n,local,defaultTarget),evaluation:'capturePropertiesThenConstruct',callbackEvaluation:'deferredWithLexicalBindings'};
}
function constructionExpression(input,ctx,api) {
    if(!ctx.pendingSystemConstruction)return null;
    const {ts,unwrap,expr,fail}=api,n=unwrap(input);
    const recordValue=value=>{value=unwrap(value);if(ts.isArrowFunction(value)||ts.isFunctionExpression(value))return {kind:'constructionClosure',callback:require('./private-programs').compileCallback(value,ctx,api,value.parameters.map((_,i)=>'nativeArgument'+i)),identity:'newFunctionPerExpression',evaluateBody:'nativeInvocationOnly'};return expr(value,ctx);};
    if(ts.isObjectLiteralExpression(n)){return {kind:'constructionRecord',properties:n.properties.map(p=>{if(!ts.isPropertyAssignment(p)&&!ts.isShorthandPropertyAssignment(p))fail(ctx,p,'construction-record-property');const key=p.name;if(!ts.isIdentifier(key)&&!ts.isStringLiteral(key)&&!ts.isNumericLiteral(key)&&!ts.isComputedPropertyName(key))fail(ctx,key,'construction-record-key');const name=ts.isComputedPropertyName(key)?api.literal(key.expression,ctx):key.text;if(!['string','number'].includes(typeof name))fail(ctx,key,'construction-record-key-type');return {key:String(name),value:recordValue(ts.isShorthandPropertyAssignment(p)?p.name:p.initializer)};}),evaluationOrder:'leftToRight',identity:'freshObjectPerExpression'};}
    if(ts.isIdentifier(n)&&n.text==='undefined'&&!ctx.scope.has(n.text)&&!ctx.constants.has(n.text))return {kind:'undefined'};
    if(api.modifierEffects){const generator=ongoingModifierConstruction(n,ctx,api);if(generator)return generator;}
    if(ts.isTemplateExpression(n)&&api.contextTitle)return {kind:'constructionText',parts:api.contextTitle(n,{...ctx,pendingSystemConstruction:false})};
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['filter','find','some','every','includes','indexOf','concat','slice'].includes(n.expression.name.text)) {
        const name=n.expression.name.text,predicate=['filter','find','some','every'].includes(name);
        if(n.arguments.length>(predicate?1:2)||predicate&&n.arguments.length!==1)fail(ctx,n,'construction-collection-arity');
        const args=n.arguments.map(a=>{if(!predicate)return expr(a,ctx);const cb=bindAttackCallback(a,ctx,['collectionElement','collectionIndex','collection'],api);return {kind:'constructionPredicate',parameters:cb.parameters,body:expr(cb.lowered.node,cb.lowered.ctx)};});
        return {kind:'constructionCollectionCall',method:name,receiver:expr(n.expression.expression,{...ctx,allowArenaList:true}),args,optionalCall:!!n.questionDotToken,optionalReceiver:!!n.expression.questionDotToken,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain),evaluation:'expressionEvaluation'};
    }
    if(ts.isPropertyAccessExpression(n)&&(n.questionDotToken||n.flags&ts.NodeFlags.OptionalChain||!ts.isIdentifier(unwrap(n.expression))&&!ts.isPropertyAccessExpression(unwrap(n.expression))))return {kind:'constructionMember',receiver:expr(n.expression,ctx),property:n.name.text,optional:!!n.questionDotToken,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain)};
    if(ts.isElementAccessExpression(n)&&(n.questionDotToken||n.flags&ts.NodeFlags.OptionalChain))return {kind:'constructionIndex',receiver:expr(n.expression,ctx),index:expr(n.argumentExpression,ctx),optional:!!n.questionDotToken,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain)};
    return null;
}
module.exports={systemArrayFactory,systemPropertiesFactory,constructionFrame,systemConstructionValue,constructionExpression};

