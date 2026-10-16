'use strict';

// Promote legacy token leaves only when an ability also reads its triggering
// event. Native systems allocate a fresh creation event; that event's context
// still points at the original ability context and its original trigger event.
function bindTokenEventFactories(ability){
 let promoted=0;
 const visit=value=>{
  if(!value||typeof value!=='object')return;
  if(value.kind==='createTokenUnit'){
   const properties={tokenType:value.tokenType,amount:value.amount,controllers:value.target,entersReady:value.entersReady};
   const replacement={
    kind:'nativeTokenUnitEventFactory',properties,defaults:{enterPlayEffect:{kind:'undefinedValue'}},
    inputContext:{kind:'nativeSystemInvocationContext',scope:'currentAbilityInvocation',source:'enclosingAbilitySource',player:'enclosingAbilityPlayer',triggerEvent:'context.event',triggerEventIdentity:'retainOriginal',propertyFactories:'nativeSystemPropertyGeneration'},
    creationEvent:{kind:'freshGameEvent',name:'onTokensCreated',identity:'newPerTargetPlayer',targetIteration:'nativeTargetArrayOrder',context:'inputContext',fields:{player:'capturedTargetPlayer',amount:'generatedProperties.amount',tokenType:'generatedProperties.tokenType',entersReady:'generatedProperties.entersReady',enterPlayEffect:'generatedProperties.enterPlayEffect'},fieldBindingStage:'superUpdateEventAddPropertiesToEvent',triggerEventRegister:'unchanged'},
    updateEvent:{after:'superUpdateEvent',regenerateProperties:true,generatedTokens:{initialValue:[],allocationStage:'eventUpdateBeforeResolution',loop:{indexInitial:0,condition:'indexLessThanGeneratedPropertiesAmount',increment:1},factory:'game.generateToken',owner:'capturedTargetPlayer',tokenType:'generatedProperties.tokenType'},cancelledEventTokens:'retainUntilRoundCleanup'},
    eventHandler:{kind:'emptySourceHandler'},
    contingentEventFactory:{registeredAt:'eventUpdate',input:'creationEvent',iteration:'creationEvent.generatedTokensInOrder',system:'PutIntoPlaySystem',properties:{controller:'capturedTargetPlayer',target:'iteratedGeneratedToken',entersReady:{kind:'logicalOr',left:'creationEvent.entersReady',right:'capturedTargetPlayerHasTokenUnitsEnterPlayReady'},entryType:'created',enterPlayEffect:'creationEvent.enterPlayEffect'},context:'creationEvent.context',contextEvent:'originalInputContextEvent',order:'creationEvent.orderPlusOne'},
    resultEvent:{identity:'creationEvent',generatedTokensField:'creationEvent.generatedTokens',separateFrom:'inputContext.event',continuationBinding:'existingResolvedEventContextProtocol'}
   };
   for(const k of Object.keys(value))delete value[k];Object.assign(value,replacement);promoted++;return;
  }
  for(const child of Object.values(value))visit(child);
 };
 visit(ability);return promoted;
}
module.exports={bindTokenEventFactories};
