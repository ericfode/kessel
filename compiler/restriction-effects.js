'use strict';

const {callbackValue}=require('./attack-initiation');
// Restrictions retain the attempted-action context separately from the effect
// source. A condition is evaluated at the restriction check, not grant time.
function restrictionModifier(call,ctx,duration,target,api) {
    const {access,unwrap,object,literal,expr,fail}=api;
    const parts=access(call.expression,ctx),name=parts[2];
    if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='ongoingEffects'||!['cardCannot','playerCannot'].includes(name))return null;
    if(call.arguments.length!==1)fail(ctx,call,'restriction-modifier-arity');
    const raw=unwrap(call.arguments[0]),out={kind:name==='cardCannot'?'cardRestriction':'playerRestriction',target,duration};
    if(api.ts.isObjectLiteralExpression(raw)) {
        const obj=object(raw,ctx),allowed=name==='cardCannot'?['cannot','restrictedActionCondition']:['cannot','restrictedActionCondition','applyingPlayer','source'];
        for(const [key,v]of obj.props)if(!allowed.includes(key))fail(obj.ctx,v,'restriction-field:'+key);
        if(!obj.props.has('cannot'))fail(obj.ctx,raw,'restriction-missing-type');out.restriction=literal(obj.props.get('cannot'),obj.ctx);
        if(obj.props.has('restrictedActionCondition'))out.condition=callbackValue(obj.props.get('restrictedActionCondition'),obj.ctx,['attemptedActionContext','restrictionSource'],api);
        for(const key of ['applyingPlayer','source'])if(obj.props.has(key))out[key]=expr(obj.props.get(key),obj.ctx);
    } else out.restriction=literal(raw,ctx);
    if(typeof out.restriction!=='string'||!out.restriction)fail(ctx,raw,'restriction-type');
    return out;
}
function simpleAttackModifier(call,ctx,duration,target,api) {
    const {access,literal,fail}=api,parts=access(call.expression,ctx);
    if(parts.join('.')==='OngoingEffectBuilder.card.static'&&!ctx.scope.has('OngoingEffectBuilder')&&!ctx.constants.has('OngoingEffectBuilder')&&call.arguments.length===1&&literal(call.arguments[0],ctx)==='canAttackGroundArenaFromSpaceArena')return {kind:'attackPermission',permission:'canAttackGroundArenaFromSpaceArena',target,duration};
    const kind=parts[2];
    if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='ongoingEffects'||!['dealsCombatDamageFirst','loseAllAbilities'].includes(kind))return null;
    if(call.arguments.length)fail(ctx,call,'attack-marker-arity');return {kind,target,duration};
}
module.exports={restrictionModifier,simpleAttackModifier};
