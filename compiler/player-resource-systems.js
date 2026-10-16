'use strict';

// Source conversion only. These operations retain player identity, event shape,
// and property timing without expanding executable runtime admission.
const {costCallback,playerTarget,scalarProperty}=require('./cost-resource-systems');
const constant=value=>({kind:'const',value});
function fields(obj,allowed,api){for(const [k,v]of obj.props)if(!allowed.includes(k))api.fail(obj.ctx,v,'player-resource-field:'+k);}
function playerResourceSystem(kind,obj,api){
    const {props,ctx,node}=obj;
    const affected=()=>playerTarget(props.get('target'),ctx,'enemyPlayer',api);
    if(kind==='indirectDamageToPlayer'){
        fields(obj,['amount','target'],api);
        if(!props.has('amount'))api.fail(ctx,node,'indirect-damage-missing-amount');
        return {kind:'indirectDamagePlayerSystem',target:affected(),amount:scalarProperty(props.get('amount'),ctx,api),event:'onIndirectDamageDealtToPlayer',parentEventAmount:'unmodifiedProperty',distribution:{kind:'indirectDamageDistribution',targetPlayer:'firstSystemTarget',amount:'propertyPlusPendingAbilityDamageIncreasePlusActingPlayerModifiers',modifierOpponentCheck:'actingPlayerDiffersFromFirstTarget',chooser:'targetPlayerUnlessActingPlayerOrSourceUnitAssignsIndirectDamage',canChooseNoTargets:false,canDistributeLess:false,maxTargets:null,damageType:'ability',isIndirect:true,ignoreAbilityDamageIncrease:true},eligibility:'propertyAmountPositive',queueOrder:['parentEvent','distributionEvents']};
    }
    if(kind==='discardEntireHand'){
        fields(obj,['target'],api);
        return {kind:'discardEntireHandSystem',target:affected(),perPlayer:true,cardOrder:'handZoneOrder',events:['discardSpecificCardPerCard','entireHandDiscardedPerAffectedPlayer'],checkCanAffectBeforeGenerating:true};
    }
    if(kind==='takeControlOfCreditToken'){
        fields(obj,['target','amount','newController'],api);
        if(!props.has('newController'))api.fail(ctx,node,'credit-control-missing-controller');
        return {kind:'takeControlCreditTokenSystem',target:affected(),amount:props.has('amount')?scalarProperty(props.get('amount'),ctx,api):constant(1),newController:playerTarget(props.get('newController'),ctx,null,api),selection:'firstCreditsInBaseZoneOrder',amountClamp:'availableCreditCount',ownership:'changesWithController',destination:'base',event:'onTakeControl'};
    }
    return null;
}
function playerDiscardSystem(kind,obj,api){
    const {props,ctx,node}=obj;
    if(kind==='discardFromDeck'){
        fields(obj,['target','amount'],api);
        if(!props.has('amount'))api.fail(ctx,node,'deck-discard-missing-amount');
        return {kind:'discardDeckSystem',target:playerTarget(props.get('target'),ctx,'enemyPlayer',api),amount:scalarProperty(props.get('amount'),ctx,api),assertAmountNonNegative:true,selection:'topCardsInDeckOrder',amountClamp:'availableDeckSize',events:['discardSpecificCardPerCard','overallDiscardEvent'],zeroAmount:'emitOverallEventAndReturnFromAllPlayerLoop',overallEventCondition:'alwaysTrue',mustFullyResolveRequiresFullAmount:true};
    }
    if(!['discardCardsFromOwnHand','discardCardsFromOpponentsHand'].includes(kind))return null;
    fields(obj,['target','amount','random','cardTypeFilter','cardCondition'],api);
    const random=props.has('random')?api.literal(props.get('random'),ctx):false;
    if(typeof random!=='boolean')api.fail(ctx,node,'discard-random-boolean');
    return {kind:'discardHandSystem',target:playerTarget(props.get('target'),ctx,'enemyPlayer',api),amount:props.has('amount')?costCallback(props.get('amount'),ctx,['affectedPlayer'],'perAffectedPlayerDiscardAmount',api):constant(1),random,cardTypeFilter:props.has('cardTypeFilter')?api.literal(props.get('cardTypeFilter'),ctx):'any',cardCondition:props.has('cardCondition')?costCallback(props.get('cardCondition'),ctx,['card','context'],'discardEligibilityAndSelection',api):constant(true),wrapperChoosingPlayer:kind==='discardCardsFromOwnHand'?'self':'opponent',actualChooser:'affectedPlayer',availabilityFilter:'cardConditionAndCardType',generationFilter:'cardConditionOnly',assertAmountNonNegative:true,amountClamp:'availableFilteredHand',zeroAmount:'noEvents',autoSingleTarget:'allAvailableWhenAmountAtLeastAvailableCount',randomSource:'gameRandomGenerator',events:['discardSpecificCardPerChosenCard','overallDiscardPerNonemptyPlayer'],overallEventCondition:'alwaysTrue',mustFullyResolveRequiresFullAmount:true};
}
module.exports={playerResourceSystem,playerDiscardSystem};
