'use strict';

// Registration-time play costs are separate from action activation costs.
function playCostRegistration(registration,input,ctx,api){
    const obj=api.object(input,ctx),{props}=obj;
    for(const [key,value]of props)if(!['title','costName','cost','immediateEffect','targetResolver'].includes(key))api.fail(obj.ctx,value,'play-cost-field:'+key);
    const forms=['cost','immediateEffect','targetResolver'].filter(k=>props.has(k));
    if(forms.length!==1)api.fail(obj.ctx,obj.node,'play-cost-exclusive-form');
    const form=forms[0],alternate=registration==='addAlternatePlayCost';
    const result={trigger:alternate?'alternatePlayCost':'additionalPlayCost',title:props.has('title')?api.displayText(props.get('title'),obj.ctx):'',costs:[],effect:{kind:'playCostRegistration',form,blankedOutOfPlay:true,registrationTiming:'cardSetup',...(alternate?{playTypes:['playFromHand','playFromOutOfPlay'],printedCostAdjustment:'free',mergeOwnAdditionalCosts:false,opponentImposedAdditionalCosts:'stillApplied'}:{captureTiming:'playActionConstruction',preservedAcrossActionClone:true})}};
    if(props.has('costName')){
        if(form==='cost')api.fail(obj.ctx,obj.node,'direct-play-cost-name');
        const name=api.literal(props.get('costName'),obj.ctx);
        if(typeof name!=='string'||!name.length||name.length>128)api.fail(obj.ctx,obj.node,'play-cost-name');
        if(ctx.registeredPlayCostNames.has(name))api.fail(obj.ctx,obj.node,'duplicate-play-cost-name');
        ctx.registeredPlayCostNames.add(name);result.effect.costName=name;
    }
    if(form==='cost')result.effect.costs=api.costs(props.get('cost'),obj.ctx);
    else if(form==='immediateEffect')result.effect.cost={kind:'gameSystemCost',system:api.effect(props.get('immediateEffect'),obj.ctx),systemPropertiesUnchanged:true,ifPossible:false,canPay:'systemHasLegalTarget',resultBinding:result.effect.costName??'systemName'};
    else {
        // Reuse ordinary resolver lowering while retaining the cost-specific
        // selection and payment boundary as a distinct outer operation.
        const lowered=api.ability(input,obj.ctx,'action',{...obj,props:new Map([...props].filter(([k])=>k!=='costName'))});
        result.effect.cost={kind:'targetResolverPlayCost',resolver:lowered.targetResolver,system:lowered.effect,selectorIsCost:true,immediateSystemPropertiesUnchanged:true,canPay:'legalTargetOrSelectionAllowsNoCards',chooser:'ownPlayer',paymentControllerOverride:'self',cancellation:'costResultCanCancel',resultBinding:result.effect.costName??'cost',promptTitleParts:lowered.targetResolver?.promptTitleParts??[{kind:'const',value:result.title}]};
    }
    return result;
}
function preEnterPlayRegistration(input,ctx,api){
    const lowered=api.ability(input,ctx,'preEnterPlay');
    return {...lowered,registration:{kind:'preEnterPlayAbility',printedAbility:true,blankedOutOfPlay:true,abilityType:'triggered',timing:'enterEventPreResolution',costContext:'shallowCopyOfPlayCosts',entersReadyRecheckedAfterResolution:true}};
}
module.exports={playCostRegistration,preEnterPlayRegistration};
