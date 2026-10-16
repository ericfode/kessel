'use strict';

// Attack callbacks bind different contexts from the enclosing ability. Keep
// lexical references and callback parameters distinct, and retain factory time.
function attackCallback(input, ctx, roles, api) {
    const {ts,unwrap,lambda,expr,fail}=api;
    const node=unwrap(input);
    if(!ts.isArrowFunction(node)&&!ts.isFunctionExpression(node))fail(ctx,node,'attack-callback-required');
    if(node.parameters.length>roles.length)fail(ctx,node,'attack-callback-arity');
    const namespaces=new Set(['AbilityHelper','abilityHelper','StaticAbilityHelper','OngoingEffectBuilder','EnumHelpers','TextHelper','Set',...Object.keys(ctx.enums).map(key=>key.split('.')[0])]);
    const bindings=n=>{
        if((ts.isParameter(n)||ts.isVariableDeclaration(n))&&ts.isIdentifier(n.name)&&namespaces.has(n.name.text))fail(ctx,n,'attack-callback-namespace-shadow');
        ts.forEachChild(n,bindings);
    };bindings(node);
    const roots=roles.map((role,i)=>`attackBinding${(ctx.attackBindingDepth||0)+1}_${i}`);
    const typedCallbackRoles=new Map(ctx.typedCallbackRoles||[]);roots.forEach((root,i)=>typedCallbackRoles.set(root,roles[i]));
    const child={...ctx,typedCallbackRoles,attackBindingDepth:(ctx.attackBindingDepth||0)+1,typedCallbackRoots:new Set([...(ctx.typedCallbackRoots||[]),...roots])};
    const lowered=lambda(node,child,roots);
    return {lowered,parameters:node.parameters.map((_,i)=>({root:roots[i],role:roles[i]})),body:()=>expr(lowered.node,lowered.ctx)};
}
function callbackValue(input,ctx,roles,api) {
    const callback=attackCallback(input,ctx,roles,api);
    return {kind:'attackCallback',parameters:callback.parameters,body:callback.body()};
}
function compileAttackProperties(input,context,defaultTarget,api) {
    const {ts,unwrap,object,knownProperties,literal,expr,target,modifierEffects,fail,displayText,contextTitle}=api;
    const node=unwrap(input),factory=ts.isArrowFunction(node)||ts.isFunctionExpression(node);
    const callback=factory?attackCallback(node,context,['systemContext'],api):null;
    const obj=object(callback?callback.lowered.node:node,callback?callback.lowered.ctx:context),{props,ctx}=obj;
    knownProperties(obj,['target','attacker','attackerCondition','targetCondition','ignoredRequirements','isAmbush','allowExhaustedAttacker','optional','attackerLastingEffects','defenderLastingEffects','attackerCombatDamageOverride','message','messageArgs','costHandler']);
    if(props.has('costHandler'))fail(ctx,props.get('costHandler'),'attack-imperative-cost-handler');
    const result={target:target(props.get('target'),ctx,defaultTarget)};
    if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),ctx);
    if(props.has('attacker'))result.attacker=expr(props.get('attacker'),ctx);
    for(const key of ['isAmbush','allowExhaustedAttacker','optional'])if(props.has(key)) {
        const value=literal(props.get(key),ctx);if(typeof value!=='boolean')fail(ctx,props.get(key),'attack-boolean:'+key);result[key]=value;
    }
    if(props.has('ignoredRequirements')) {
        const value=literal(props.get('ignoredRequirements'),ctx);
        if(!Array.isArray(value)||value.some(v=>typeof v!=='string'))fail(ctx,props.get('ignoredRequirements'),'attack-ignored-requirements');result.ignoredRequirements=value;
    }
    for(const key of ['attackerCondition','targetCondition'])if(props.has(key))result[key]=callbackValue(props.get(key),ctx,['candidate','systemContext'],api);
    if(props.has('attackerCombatDamageOverride'))result.attackerCombatDamageOverride=callbackValue(props.get('attackerCombatDamageOverride'),ctx,['attack','systemContext'],api);
    if(props.has('message'))result.message=displayText(props.get('message'),ctx);
    if(props.has('messageArgs'))result.messageArgs=callbackValue(props.get('messageArgs'),ctx,['attack','systemContext'],api);
    for(const participant of ['attacker','defender']) {
        const key=participant+'LastingEffects';if(!props.has(key))continue;
        const raw=unwrap(props.get(key)),entries=ts.isArrayLiteralExpression(raw)?raw.elements:[raw];
        if(!entries.length||entries.length>16)fail(ctx,raw,'attack-lasting-entry-bound');
        result[key]=entries.map(entry=>{
            const n=unwrap(entry),isFactory=ts.isArrowFunction(n)||ts.isFunctionExpression(n);
            const cb=isFactory?attackCallback(n,ctx,['systemContext','attack'],api):null;
            const e=object(cb?cb.lowered.node:n,cb?cb.lowered.ctx:ctx);
            knownProperties(e,['effect','condition']);if(!e.props.has('effect'))fail(e.ctx,e.node,'attack-lasting-without-effect');
            const out={kind:'attackLastingEntry',evaluation:participant==='attacker'&&!isFactory&&!e.props.has('condition')?'beforeDefenderSelection':'afterAttackCreated',
                effect:modifierEffects(e.props.get('effect'),e.ctx,'attack',{kind:'ref',path:['attackParticipant']})};
            if(e.props.has('condition'))out.condition=callbackValue(e.props.get('condition'),e.ctx,['attack','systemContext'],api);
            if(cb)out.factory={kind:'attackFactory',parameters:cb.parameters};
            return out;
        });
    }
    return callback?{kind:'attackPropertiesFactory',parameters:callback.parameters,properties:result}:{kind:'attackProperties',properties:result};
}
function compileAttackSystem(input,ctx,defaultTarget,api) {
    return {kind:'initiateAttack',properties:compileAttackProperties(input,ctx,defaultTarget,api)};
}
// This expression domain is available only under explicit attack callbacks.
// Keep full target lists and the queried player's identity, never a first-target
// or controller approximation.
function normalizeAttackExpression(node,ctx,api) {
    const {ts,unwrap,expr,object,fail}=api;node=unwrap(node);
    if(!ctx.typedCallbackRoles?.size)return null;
    if(ts.isArrayLiteralExpression(node)&&node.elements.length===1&&ts.isSpreadElement(node.elements[0])) {
        const set=unwrap(node.elements[0].expression);
        if(ts.isNewExpression(set)&&ts.isIdentifier(set.expression)&&set.expression.text==='Set'&&set.arguments?.length===1&&!ctx.scope.has('Set')&&!ctx.constants.has('Set'))return {kind:'distinctList',receiver:expr(set.arguments[0],ctx)};
    }
    if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.expression.getText(ctx.source)==='EnumHelpers'&&node.expression.name.text==='isArena') {
        if(node.arguments.length!==1||ctx.scope.has('EnumHelpers')||ctx.constants.has('EnumHelpers'))fail(ctx,node,'attack-arena-enum-query');
        return {kind:'query',name:'zoneIsArena',args:[expr(node.arguments[0],ctx)]};
    }
    if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='reduce') {
        const receiver=expr(node.expression.expression,ctx);
        if(receiver.kind==='query'&&receiver.name==='getAllAttackTargets') {
            if(node.arguments.length!==2)fail(ctx,node,'attack-target-fold-initial-value');
            return {kind:'attackTargetFold',receiver,initial:expr(node.arguments[1],ctx),callback:callbackValue(node.arguments[0],ctx,['accumulator','candidate','index','collection'],api)};
        }
    }
    if(ts.isPropertyAccessExpression(node)&&node.name.text==='length') {
        const n=unwrap(node.expression);
        if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='getArenaUnits') {
            const receiver=normalizeAttackExpression(n,ctx,api);
            if(receiver)return {kind:'attackListLength',receiver};
        }
    }
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
    const call=node.expression,name=call.name.text;
    if(!['getAllTargets','getDefendingPlayer','targetIsUnit','getArenaUnits','hasSomeArenaUnit','getPrintedPower'].includes(name))return null;
    const receiver=expr(call.expression,ctx);
    if(name==='getPrintedPower'&&receiver.kind==='ref'&&ctx.typedCallbackRoles.has(receiver.path[0])){if(node.questionDotToken||call.questionDotToken)fail(ctx,node,'optional-printed-power-query');if(node.arguments.length)fail(ctx,node,'printed-power-query-arity');return {kind:'query',name,receiver,args:[]};}
    const attack=receiver.kind==='ref'&&ctx.typedCallbackRoles.get(receiver.path[0])==='attack'&&receiver.path.length===1;
    const player=receiver.kind==='query'&&receiver.name==='attackDefendingPlayer'||receiver.kind==='ref'&&ctx.typedCallbackRoles.has(receiver.path[0])&&['attackingPlayer','player','opponent'].includes(receiver.path.at(-1));
    if(!attack&&!player)return null;
    if(node.questionDotToken||call.questionDotToken)fail(ctx,node,'optional-attack-query');
    if(attack&&['getAllTargets','getDefendingPlayer'].includes(name)) {
        if(node.arguments.length)fail(ctx,node,'attack-query-arity');
        return {kind:'query',name:name==='getAllTargets'?'getAllAttackTargets':'attackDefendingPlayer',receiver,args:[]};
    }
    if(attack&&name==='targetIsUnit') {
        if(node.arguments.length>1)fail(ctx,node,'attack-target-predicate-arity');
        return {kind:'query',name:'attackTargetIsUnit',receiver,args:node.arguments.length?[callbackValue(node.arguments[0],ctx,['candidate'],api)]:[]};
    }
    if(player&&['getArenaUnits','hasSomeArenaUnit'].includes(name)) {
        if(node.arguments.length>1)fail(ctx,node,'attack-arena-filter-arity');
        const filter={};if(node.arguments.length) {
            const obj=object(node.arguments[0],ctx);
            for(const [key,value]of obj.props) {
                if(!['arena','condition','otherThan','trait','aspect','keyword'].includes(key))fail(obj.ctx,value,'attack-arena-filter:'+key);
                filter[key]=key==='condition'?callbackValue(value,obj.ctx,['candidate'],api):expr(value,obj.ctx);
            }
        }
        return {kind:name==='getArenaUnits'?'attackArenaList':'attackArenaSome',receiver,filter};
    }
    return null;
}
module.exports={compileAttackSystem,callbackValue,normalizeAttackExpression,bindAttackCallback:attackCallback};
