'use strict';

const {callbackValue,bindAttackCallback}=require('./attack-initiation');
const {restrictionModifier}=require('./restriction-effects');
function playerLimit(input,ctx,clonePerApplication,api) {
    const {ts,unwrap,access,literal,fail}=api,n=unwrap(input);
    if(!ts.isCallExpression(n)||n.arguments.length>1)fail(ctx,n,'player-modifier-limit');
    const parts=access(n.expression,ctx);if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='limit'||!['perPhase','perRound','perGame','perPlayerPerGame'].includes(parts[2]))fail(ctx,n,'player-modifier-limit');
    const max=n.arguments.length?literal(n.arguments[0],ctx):1;
    if(!Number.isSafeInteger(max)||max<1||max>64)fail(ctx,n,'player-modifier-limit-bound');
    return {kind:'playerModifierLimit',scope:parts[2],max,clonePerApplication};
}
function playerPhaseModifier(input,ctx,api) {
    const {ts,unwrap,access,object,literal,expr,fail}=api,call=unwrap(input);
    if(!ts.isCallExpression(call))return null;
    const restriction=restrictionModifier(call,ctx,'phase','effectPlayer',api);if(restriction)return restriction;
    const parts=access(call.expression,ctx),name=parts[2];
    if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='ongoingEffects'||!['unitsEnterPlayReady','addExploit'].includes(name))return null;
    if(call.arguments.length!==1)fail(ctx,call,'player-entry-modifier-arity');
    const obj=object(call.arguments[0],ctx),{props}=obj,entry=name==='unitsEnterPlayReady';
    const allowed=entry?['cardTypeFilter','entryType','match','limit']:['cardTypeFilter','match','limit','playType','attachTargetCondition','matchAbilityCosts','matchCardEffectResourcePayments','exploitKeywordAmount'];
    for(const [key,v]of props)if(!allowed.includes(key))fail(obj.ctx,v,'player-entry-modifier-field:'+key);
    const out={kind:entry?'unitsEnterPlayReady':'addExploit',cardTypeFilter:props.has('cardTypeFilter')?literal(props.get('cardTypeFilter'),obj.ctx):entry?'unit':'any'};
    if(props.has('match'))out.match=callbackValue(props.get('match'),obj.ctx,entry?['enteringUnit','entryContext']:['adjustedCard','adjusterSource'],api);
    if(props.has('limit'))out.limit=playerLimit(props.get('limit'),obj.ctx,entry,api);
    if(entry) {
        if(!props.has('entryType'))fail(obj.ctx,obj.node,'entry-ready-missing-entry-type');out.entryType=literal(props.get('entryType'),obj.ctx);
        out.matchTiming='beforeUnitEntry';out.limitIncrementTiming='successfulMatcherApplication';
    } else {
        if(!props.has('exploitKeywordAmount'))fail(obj.ctx,obj.node,'exploit-missing-amount');out.amount=expr(props.get('exploitKeywordAmount'),obj.ctx);out.stage='exploit_2';out.sourceRequiresCost=true;
        if(props.has('playType'))out.playType=literal(props.get('playType'),obj.ctx);
        if(props.has('attachTargetCondition'))out.attachTargetCondition=callbackValue(props.get('attachTargetCondition'),obj.ctx,['attachTarget','playContext','adjusterSource'],api);
        for(const key of ['matchAbilityCosts','matchCardEffectResourcePayments']) {const value=props.has(key)?literal(props.get(key),obj.ctx):false;if(typeof value!=='boolean')fail(obj.ctx,obj.node,'exploit-modifier-boolean');out[key]=value;}
        if(!out.limit)out.limit={kind:'playerModifierLimit',scope:'perPlayerPerGame',max:1,clonePerApplication:false};
    }
    return out;
}
function compilePlayerPhaseEffect(input,context,api) {
    const {ts,unwrap,object,target,fail,expr}=api,node=unwrap(input);
    // Cost adjusters are owned by the separate cost module. Return null before
    // parsing them, so its existing one-application/limit contract is retained.
    const factory=ts.isArrowFunction(node)||ts.isFunctionExpression(node),binding=factory?bindAttackCallback(node,context,['systemContext'],api):null;
    const obj=object(binding?binding.lowered.node:node,binding?binding.lowered.ctx:context);if(!obj.props.has('effect'))return null;
    const raw=unwrap(obj.props.get('effect')),nodes=ts.isArrayLiteralExpression(raw)?raw.elements:[raw];
    const effects=nodes.map(n=>playerPhaseModifier(n,obj.ctx,api));
    if(effects.every(e=>e===null))return null;
    if(effects.some(e=>e===null))fail(obj.ctx,raw,'mixed-unconverted-player-phase-modifier');
    for(const [key,value]of obj.props)if(!['effect','target','condition','title','ongoingEffectDescription','ongoingEffectTargetDescription','effectArgs'].includes(key))fail(obj.ctx,value,'player-phase-field:'+key);
    if(!effects.length||effects.length>16)fail(obj.ctx,raw,'player-phase-effect-bound');
    const result={kind:'phasePlayerEffect',target:target(obj.props.get('target'),obj.ctx,'ownPlayer'),duration:'phase',effects};
    if(obj.props.has('condition'))result.condition=callbackValue(obj.props.get('condition'),obj.ctx,['effectContext'],api);
    return binding?{kind:'playerPhasePropertiesFactory',parameters:binding.parameters,body:result}:result;
}
module.exports={playerPhaseModifier,compilePlayerPhaseEffect};
