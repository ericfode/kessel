'use strict';

// Retain the pinned registration contracts instead of rewriting prevention into
// damage subtraction or cancellation into a zero-result ordinary effect.
const REGISTRATIONS = new Set(['addDamageModificationAbility','addReplacementEffectAbility']);
function registration(name,input,context,api,validatedObject=null) {
    const {ts,unwrap,object,knownProperties,expr,literal,displayText,contextTitle,effect,fail,eventHistory}=api;
    const obj=validatedObject||object(input,{...context,eventHistory:true}),ctx=obj.ctx,props=obj.props;
    const damage=name==='addDamageModificationAbility';
    const common=['title','optional','condition','limit','sourceZoneFilter','collectiveTrigger','optionalMakesAbilityNonCancelable','appendOverrideTitle','showUnresolvable','canBeTriggeredBy','zoneFilter','onlyIfYouDoEffect'];
    knownProperties(obj,[...common,...damage?['modificationType','onlyFromPlayer','damageOfType','amount','replaceWithEffect','shouldCardHaveDamageModification','applyAtAbilityInitiation']:['when','aggregateWhen','replaceWith']]);
    const out={trigger:damage?'damageModification':'replacementEffect',kind:damage?'damageModificationRegistration':'replacementRegistration',contract:damage?'DamageModificationAbility':'ReplacementEffectAbility',title:props.has('title')?displayText(props.get('title'),ctx):''};
    Object.assign(out,require('./registration-metadata').metadata(obj,api,['canBeTriggeredBy','zoneFilter']));
    if(props.has('contextTitle'))out.contextTitle=contextTitle(props.get('contextTitle'),ctx);
    for(const key of ['optional','collectiveTrigger','optionalMakesAbilityNonCancelable','appendOverrideTitle','showUnresolvable','applyAtAbilityInitiation'])if(props.has(key)){const value=literal(props.get(key),ctx);if(typeof value!=='boolean')fail(ctx,props.get(key),'replacement-boolean:'+key);out[key]=value;}
    if(props.has('sourceZoneFilter'))out.sourceZoneFilter=literal(props.get('sourceZoneFilter'),ctx);
    if(props.has('limit'))out.limit=eventHistory.activationLimit(props.get('limit'),ctx,api);
    if(props.has('condition'))out.condition=expr(props.get('condition'),ctx);
    if(props.has('onlyIfYouDoEffect'))out.onlyIfYouDoEffect=effect(props.get('onlyIfYouDoEffect'),ctx,'self');
    if(damage) {
        if(!props.has('modificationType'))fail(ctx,obj.node,'damage-modification-type-required');
        out.modificationType=literal(props.get('modificationType'),ctx);
        if(!['cap','all','increase','multiply','reduce','replace'].includes(out.modificationType))fail(ctx,props.get('modificationType'),'damage-modification-type');
        if(props.has('amount'))out.amount=expr(props.get('amount'),ctx);
        if(props.has('onlyFromPlayer'))out.onlyFromPlayer=literal(props.get('onlyFromPlayer'),ctx);
        if(props.has('damageOfType'))out.damageOfType=literal(props.get('damageOfType'),ctx);
        if(props.has('shouldCardHaveDamageModification'))out.shouldCardHaveDamageModification=expr(props.get('shouldCardHaveDamageModification'),ctx,['card','context']);
        if(props.has('replaceWithEffect'))out.replaceWithEffect=effect(props.get('replaceWithEffect'),ctx,'self');
        // A missing predicate means self. At initiation the same callback tests
        // the initiating card, not a proposed damage recipient.
        out.triggerContract=out.applyAtAbilityInitiation?'abilityInitiationDamageSystems':'damageEventBeforeResolution';
    } else {
        if(props.has('when')===props.has('aggregateWhen'))fail(ctx,obj.node,'replacement-exactly-one-trigger');
        if(props.has('when')){out.hooks=eventHistory.hooks(input,ctx,api);if(!out.hooks)fail(ctx,props.get('when'),'replacement-explicit-hooks');}
        else out.aggregateWhen=eventHistory.predicate(props.get('aggregateWhen'),ctx,api,['events','context']);
        if(props.has('replaceWith')) {
            const r=object(props.get('replaceWith'),ctx);knownProperties(r,['replacementImmediateEffect','target']);
            out.replaceWith={kind:'replacementSystemProperties',evaluateAt:unwrap(props.get('replaceWith')).kind===ts.SyntaxKind.ObjectLiteralExpression?'abilitySetup':'replacementContext'};
            if(r.props.has('target'))out.replaceWith.target=expr(r.props.get('target'),r.ctx);
            if(r.props.has('effect')){out.replaceWith.effectParts=contextTitle(r.props.get('effect'),r.ctx);out.replaceWith.effectEvaluation='replacementPropertyGeneration';}
            if(r.props.has('replacementImmediateEffect'))out.replaceWith.replacementImmediateEffect=effect(r.props.get('replacementImmediateEffect'),r.ctx,'replacedEventCard');
        }
        // Omitting replaceWith is the source's ordinary event cancellation:
        // unlike damage prevention, the nullified event does not resolve.
        out.nullifiedEventCountsAsResolved=false;
    }
    return out;
}
module.exports={REGISTRATIONS,registration};
