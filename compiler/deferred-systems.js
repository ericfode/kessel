'use strict';
const history=require('./event-history');
// Representation only: disclose prompts and delayed scheduling remain pending.
function deferredSystem(kind,obj,defaultTarget,api) {
    const {literal,expr,target,effect,contextTitle,fail}=api;
    const check=fields=>{for(const [key,value]of obj.props)if(!fields.includes(key))fail(obj.ctx,value,'unsupported-'+kind+'-field:'+key);};
    if(kind==='optional'||kind==='randomSelection') {
        const random=kind==='randomSelection';check(random?['target','count','innerSystem']:['target','title','innerSystem']);
        if(!obj.props.has('innerSystem')||!random&&!obj.props.has('title'))fail(obj.ctx,obj.node,'aggregate-required-field');
        const aggregateTarget=obj.props.has('target')?target(obj.props.get('target'),obj.ctx):{kind:'inheritedSystemTarget',binding:defaultTarget==='self'?null:defaultTarget,fallback:'empty'};
        const childDefault=random?'randomTarget':{kind:'aggregateChildDefault',enclosingTarget:aggregateTarget,fallback:'nativeSystemDefault'};
        return {kind,target:aggregateTarget,effect:effect(obj.props.get('innerSystem'),obj.ctx,childDefault),...(random?{count:obj.props.has('count')?expr(obj.props.get('count'),obj.ctx):{kind:'const',value:1},randomProtocol:'pinnedHelpers.getRandomArrayElements',resultBinding:'randomTarget',sampleTiming:'firstPropertyEvaluationUnlessCached',requiresRollbackConfirmationFor:'bothPlayers'}:{titleParts:contextTitle(obj.props.get('title'),obj.ctx),choosingPlayer:'contextPlayer',legality:'innerSystem',declineResult:'noEvents'})};
    }
    if(kind==='disclose') {
        check(['target','activePromptTitle','aspects','mode']);
        if(!obj.props.has('aspects'))fail(obj.ctx,obj.node,'disclose-missing-aspects');
        const aspects=literal(obj.props.get('aspects'),obj.ctx),mode=obj.props.has('mode')?literal(obj.props.get('mode'),obj.ctx):'all';
        if(!Array.isArray(aspects)||!aspects.length||aspects.some(a=>!['vigilance','command','aggression','cunning','heroism','villainy'].includes(a))||!['any','some','all'].includes(mode))fail(obj.ctx,obj.node,'disclose-aspect-mode');
        return {kind:'disclose',target:target(obj.props.get('target'),obj.ctx,'ownPlayer'),aspects,mode,aspectMultiplicity:true,selectedCardsEventField:'disclosedCards',triggeredNoMatchPolicy:'maskWithPauseWithoutEvent',selectionContext:'targetPlayer',...(obj.props.has('activePromptTitle')?{promptParts:contextTitle(obj.props.get('activePromptTitle'),obj.ctx)}:{})};
    }
    if(!['delayedCardEffect','delayedPlayerEffect'].includes(kind))return null;
    check(['target','title','when','duration','limit','immediateEffect','effectDescription']);
    for(const key of ['title','when','immediateEffect'])if(!obj.props.has(key))fail(obj.ctx,obj.node,'delayed-missing-'+key);
    const duration=obj.props.has('duration')?literal(obj.props.get('duration'),obj.ctx):'persistent';
    if(!['persistent','untilEndOfAttack','untilEndOfPhase','untilEndOfRound'].includes(duration))fail(obj.ctx,obj.node,'delayed-duration');
    const hooks=history.hooks(obj.node,obj.ctx,api);if(!hooks||hooks.some(h=>h.alias))fail(obj.ctx,obj.node,'delayed-named-event-hooks');
    const limit=obj.props.has('limit')?history.activationLimit(obj.props.get('limit'),obj.ctx,api):{kind:'abilityLimit',factory:'perPlayerPerGame',args:[1],scope:'delayedApplication',sharedAcrossHooks:true};
    return {kind:'delayedEffect',effectType:kind==='delayedCardEffect'?'card':'player',target:obj.props.has('target')?target(obj.props.get('target'),obj.ctx):{kind:'inheritedSystemTarget',binding:defaultTarget==='self'?null:defaultTarget,fallback:'empty'},titleParts:contextTitle(obj.props.get('title'),obj.ctx),duration,hooks,limit,contextPolicy:'retainApplicationContextAndAddTriggerEvents',childTargetDefault:'matchedDelayedTargets',effect:effect(obj.props.get('immediateEffect'),{...obj.ctx,eventHistory:true},'delayedTargets'),...(obj.props.has('effectDescription')?{descriptionParts:contextTitle(obj.props.get('effectDescription'),obj.ctx)}:{})};
}
module.exports={deferredSystem};
