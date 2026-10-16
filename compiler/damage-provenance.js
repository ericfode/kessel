'use strict';

const {bindAttackCallback,callbackValue}=require('./attack-initiation');
function compileDamageProvenance(input,context,defaultTarget,distribution,api) {
    const {ts,unwrap,object,expr,target,literal,fail,contextTitle}=api,node=unwrap(input),scope={...context,damageProvenanceScope:true};
    const factory=ts.isArrowFunction(node)||ts.isFunctionExpression(node),binding=factory?bindAttackCallback(node,scope,['systemContext'],api):null;
    const obj=object(binding?binding.lowered.node:node,binding?binding.lowered.ctx:scope),{props,ctx}=obj;
    const allowed=distribution?['amountToDistribute','source','target','canChooseNoTargets','canDistributeLess','activePromptTitle','player','controller','zoneFilter','cardTypeFilter','cardCondition','maxTargets']:['amount','source','target','type','isCost','isIndirect','isUnpreventable','ignoreAbilityDamageIncrease'];
    for(const [key,value]of props)if(!allowed.includes(key))fail(ctx,value,'damage-provenance-field:'+key);
    if(!props.has('source'))fail(ctx,obj.node,'damage-provenance-missing-source');
    const source=expr(props.get('source'),ctx);
    const result={kind:distribution?'distributeDamageFromCard':'damageFromCard',source,target:target(props.get('target'),ctx,defaultTarget),sourcePlayer:'sourceControllerOrContextPlayerAtEventCreation',sourceCard:'sourceOrContextSourceAtEventCreation',triggeredDefeatController:'triggerEventLastKnownController',requiresDamageableTarget:true};
    if(distribution) {
        if(!props.has('amountToDistribute')||!props.has('canChooseNoTargets'))fail(ctx,obj.node,'damage-distribution-required-fields');
        const amount=props.get('amountToDistribute');result.amount=ts.isArrowFunction(unwrap(amount))||ts.isFunctionExpression(unwrap(amount))?callbackValue(amount,ctx,['systemContext'],api):expr(amount,ctx);
        result.selector={mode:'unlimited',controller:props.has('controller')?literal(props.get('controller'),ctx):'any',cardTypeFilter:props.has('cardTypeFilter')?literal(props.get('cardTypeFilter'),ctx):'any',zoneFilter:props.has('zoneFilter')?expr(props.get('zoneFilter'),ctx):{kind:'const',value:'anyAttackable'}};
        if(props.has('cardCondition'))result.selector.condition=callbackValue(props.get('cardCondition'),ctx,['candidate','systemContext'],api);
        result.chooser=props.has('player')?literal(props.get('player'),ctx):'self';
        for(const key of ['canChooseNoTargets','canDistributeLess']){const v=props.has(key)?literal(props.get(key),ctx):false;if(typeof v!=='boolean')fail(ctx,obj.node,'damage-distribution-boolean');result[key]=v;}
        if(result.canDistributeLess&&!result.canChooseNoTargets)fail(ctx,obj.node,'damage-distribution-less-needs-zero');
        if(props.has('maxTargets'))result.maxTargets=literal(props.get('maxTargets'),ctx);
        if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),ctx);
        result.pendingAbilityDamageIncrease='onceBeforeDistribution';result.childEventsIgnoreAbilityDamageIncrease=true;
    } else {
        result.damageType=props.has('type')?literal(props.get('type'),ctx):'ability';
        if(result.damageType!=='ability')fail(ctx,obj.node,'damage-provenance-type');
        if(props.has('amount')) {const amount=props.get('amount');result.amount=ts.isArrowFunction(unwrap(amount))||ts.isFunctionExpression(unwrap(amount))?callbackValue(amount,ctx,['damageRecipient'],api):expr(amount,ctx);}
        else fail(ctx,obj.node,'damage-provenance-missing-amount');
        for(const key of ['isCost','isIndirect','isUnpreventable','ignoreAbilityDamageIncrease']){const v=props.has(key)?literal(props.get(key),ctx):false;if(typeof v!=='boolean')fail(ctx,obj.node,'damage-provenance-boolean');result[key]=v;}
    }
    return binding?{kind:'damagePropertiesFactory',parameters:binding.parameters,body:result}:result;
}
module.exports={compileDamageProvenance};
