'use strict';
const {compileCallback}=require('./private-programs');

// EventCard installs these all-zone constants immediately during setup rather
// than waiting for the card to enter an arena. Its override of constant-ability
// discovery suppresses them only for out-of-play blanking.
function playRestrictionAbility(input,context,api){
 const obj=api.object(input,context),{props,ctx,node}=obj;
 for(const [key,value]of props)if(!['title','restrictedActionCondition'].includes(key))api.fail(ctx,value,'play-restriction-field:'+key);
 if(!props.has('title'))api.fail(ctx,node,'play-restriction-title-required');
 let predicate={kind:'const',value:true};
 if(props.has('restrictedActionCondition')){predicate=compileCallback(props.get('restrictedActionCondition'),ctx,api,['restrictionActionContext','restrictionSource']);if(!predicate)api.fail(ctx,props.get('restrictedActionCondition'),'play-restriction-callback');if(predicate.parameters.length>2)api.fail(ctx,props.get('restrictedActionCondition'),'play-restriction-callback-arity');}
 return {trigger:'constant',title:api.displayText(props.get('title'),ctx),costs:[],effect:{kind:'eventPlayRestrictionRegistration',sourceZone:'any',target:'physicalSource',duration:'persistent',registerAt:'setupImmediately',effectType:'abilityRestrictions',restrictionType:'play',predicate,predicateContext:'actionBeingChecked',predicateSource:'ongoingEffectContextSource',missingPredicate:'alwaysRestricted',predicateRequiresNonNullContext:props.has('restrictedActionCondition'),ongoingContext:'refreshFromSourceController',suppression:'eventIsBlankOutOfPlay',activeWhile:'registeredEffectBelongsToCurrentConstantAbility'}};
}
function selectionOnlyStep(result,props){
 if(!result.targetResolver||result.effect||!props.has('then'))return null;
 return {kind:'selectionOnlyAbilityStep',systems:'selectedResolverSystems',directSystem:'absent',resetEventsAtExecution:true,emptyEventWindow:'resolveThenImmediately',resolutionContext:'preserveSelectedPhysicalTargets',stateCheck:'queuedAfterAbilityHandler'};
}
module.exports={playRestrictionAbility,selectionOnlyStep};
