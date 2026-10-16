'use strict';

const {callbackValue}=require('./attack-initiation');
const chain=(node,ts)=>({optionalReceiver:!!node.expression?.questionDotToken,optionalCall:!!node.questionDotToken,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)});
const attackValue=(v,ctx)=>['liveActiveAttack','attackMember','attackValueQuery','attackCollection'].includes(v?.kind)||v?.kind==='ref'&&(v.path.join('.')==='event.attack'||v.path.includes('activeAttack')||ctx.typedCallbackRoles?.get(v.path[0])==='attack'&&v.path.length===1);
function normalizeActiveAttackValue(input,ctx,api) {
    const {ts,unwrap,expr,fail}=api,node=unwrap(input);
    if(ts.isPropertyAccessExpression(node)) {
        // Preserve the existing admitted guarded participant reference.
        if(node.name.text==='attacker'&&!node.questionDotToken&&ctx.attackDefenders?.has('card')) {
            try{if(api.access(node,ctx).join('.')==='card.activeAttack.attacker')return null;}catch(error){if(!error.diagnostic)throw error;}
        }
        if(node.name.text==='activeAttack')return {kind:'liveActiveAttack',card:expr(node.expression,ctx),optionalReceiver:!!node.questionDotToken};
        const text=node.getText(ctx.source);
        if(text.includes('activeAttack')||['getSingleTarget','getDefendingPlayer'].some(name=>text.includes(name+'('))) {
            const receiver=expr(node.expression,ctx);
            if(attackValue(receiver,ctx)){if(!['attacker','attackingPlayer','id','isAmbush','attackerInPlayId','previousAttack','controller','owner','base','cost','zoneName','title','length'].includes(node.name.text))fail(ctx,node,'active-attack-member:'+node.name.text);return {kind:'attackMember',receiver,member:node.name.text,optionalReceiver:!!node.questionDotToken,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)};}
        }
    }
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
    const call=node.expression,name=call.name.text;
    const known=['getAllTargets','getSingleTarget','getDefendingPlayer','targetIsUnit','some','every','filter','map','includes','isUnit','isBase','isInPlay','hasSomeTrait','hasSomeKeyword'];
    if(!known.includes(name))return null;
    if(['isUnit','isBase','isInPlay','hasSomeTrait','hasSomeKeyword'].includes(name)&&!(node.flags&ts.NodeFlags.OptionalChain))return null;
    const text=call.expression.getText(ctx.source);
    if(!['getAllTargets','getSingleTarget','getDefendingPlayer','targetIsUnit'].includes(name)&&!text.includes('activeAttack')&&!text.includes('.attack'))return null;
    const receiver=expr(call.expression,ctx);
    if(!attackValue(receiver,ctx))return null;
    // The triggering event's complete-list query already has exact old IR.
    if(receiver.kind==='ref'&&receiver.path.join('.')==='event.attack'&&name==='getAllTargets')return null;
    if(['getAllTargets','getSingleTarget','getDefendingPlayer'].includes(name)) {
        if(node.arguments.length)fail(ctx,node,'active-attack-query-arity');
        return {kind:'attackValueQuery',name,receiver,args:[],...chain(node,ts)};
    }
    if(name==='targetIsUnit') {
        if(node.arguments.length>2)fail(ctx,node,'active-attack-predicate-arity');
        const args=node.arguments.length?[callbackValue(node.arguments[0],ctx,['candidate'],api)]:[];
        if(node.arguments.length===2){const v=api.literal(node.arguments[1],ctx);if(typeof v!=='boolean')fail(ctx,node,'active-attack-multiple-match-boolean');args.push({kind:'const',value:v});}
        return {kind:'attackValueQuery',name,receiver,args,...chain(node,ts)};
    }
    if(['some','every','filter','map'].includes(name)) {
        if(node.arguments.length!==1)fail(ctx,node,'attack-collection-arity');return {kind:'attackCollection',name,receiver,callback:callbackValue(node.arguments[0],ctx,['candidate','index','collection'],api),...chain(node,ts)};
    }
    return {kind:'attackValueQuery',name,receiver,args:node.arguments.map(n=>expr(n,ctx)),...chain(node,ts)};
}
// Optional source reads are only enabled inside the new provenance contract.
function provenanceOptionalValue(input,ctx,api) {
    if(!ctx.damageProvenanceScope)return null;
    const {ts,expr,fail}=api,node=input;
    if(ts.isPropertyAccessExpression(node)&&node.questionDotToken)return {kind:'optionalMember',receiver:expr(node.expression,ctx),member:node.name.text,continuesOptionalChain:!!(node.flags&ts.NodeFlags.OptionalChain)};
    if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&(node.questionDotToken||node.expression.questionDotToken||node.flags&ts.NodeFlags.OptionalChain)) {
        const name=node.expression.name.text;if(!['getPower','getHp','getNumericKeywordTotal','isUnit','isInPlay'].includes(name))fail(ctx,node,'provenance-optional-query:'+name);
        return {kind:'optionalValueQuery',name,receiver:expr(node.expression.expression,ctx),args:node.arguments.map(n=>expr(n,ctx)),...chain(node,ts)};
    }
    return null;
}
function attackPermissionModifier(call,ctx,duration,target,api) {
    const parts=api.access(call.expression,ctx);
    if(parts.join('.')!=='OngoingEffectBuilder.card.static'||call.arguments.length!==1||ctx.scope.has('OngoingEffectBuilder')||ctx.constants.has('OngoingEffectBuilder'))return null;
    if(api.literal(call.arguments[0],ctx)!=='canAttackSpaceArenaFromGroundArena')return null;
    return {kind:'attackPermission',permission:'canAttackSpaceArenaFromGroundArena',target,duration};
}
module.exports={normalizeActiveAttackValue,provenanceOptionalValue,attackPermissionModifier};
