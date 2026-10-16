'use strict';

// A resolved ability event is a physical event/context/ability reference. It
// cannot be replaced with a new live scan of the source card's abilities.
function compileTriggeredInventorySystem(call,context,defaultTarget,api) {
 const {ts,unwrap,object,expr,target,fail}=api,kind=call.expression.name.text;
 if(call.arguments.length!==1)fail(context,call,'triggered-inventory-effect-arity');
 const input=unwrap(call.arguments[0]),factory=ts.isArrowFunction(input)||ts.isFunctionExpression(input);
 const frame=factory?require('./system-array-factory').constructionFrame(input,context,['triggeredInventoryContext'],'propertyFactoryInvocation',api):null;
 if(frame)frame.walk(frame.node);
 const obj=object(frame?frame.node:input,frame?frame.ctx:context),{props,ctx}=obj;
 for(const [name,value]of props){
  if(!['target','triggerAll','resolvedAbilityEvent'].includes(name))fail(ctx,value,'triggered-inventory-property:'+name);
  const n=unwrap(value);if(ts.isArrowFunction(n)||ts.isFunctionExpression(n))fail(ctx,n,'triggered-inventory-function-valued-property');
 }
 const defeated=kind==='useWhenDefeatedAbility',category=defeated?'whenDefeated':'whenPlayed';
 const result={kind:'triggeredAbilityInventorySystem',category,target:target(props.get('target'),ctx,defaultTarget),triggerAll:props.has('triggerAll')?expr(props.get('triggerAll'),ctx):{kind:'const',value:false},resolvedAbilityEvent:props.has('resolvedAbilityEvent')?expr(props.get('resolvedAbilityEvent'),ctx):{kind:'const',value:null},nativeSystem:defeated?'UseWhenDefeatedSystem':'UseWhenPlayedSystem',event:defeated?'onUseWhenDefeated':'onUseWhenPlayed',targetTypes:['unit','upgrade'],propertyEvaluation:'eachGeneratePropertiesFromContext',resolvedEventBranch:'strictNullUsesLiveInventoryOtherwiseDereferenceSuppliedEvent',eligibility:{withoutResolvedEvent:{zones:['groundArena','spaceArena','resource'],requires:'canRegisterTriggeredAbilitiesAndHasCategory'},withResolvedEvent:'eventContextIsTriggeredAndReferencedAbilityHasCategory',requiredStateChange:'createOriginalAbilityContextAtPreTargetAndRequireMeetsRequirementsEmpty'},inventoryCaptureAt:'addPropertiesToEvent',eventReferences:{source:'targetPhysicalCard',abilities:'currentRegisteredAbilitiesFilteredByCategory',resolvedAbility:'resolvedAbilityEvent.ability',originalTriggerEvent:'resolvedAbilityEvent.context.event'},selection:{originalEventNullish:'capturedCategoryInventory',otherwise:'referencedResolvedAbilityOnly',singleOrTriggerAll:'resolveEveryListedAbilityInArrayOrder',multiple:'actingPlayerChoosesOneByAbilityTitle'},abilityConstruction:{kind:'TriggeredAbility',source:'targetPhysicalCard',propertiesCopy:'shallowCopySelectedAbilityProperties',overrides:{optional:false,target:'targetPhysicalCard'},conditionsCostsAndLimits:'preservedFromCopiedProperties',limitIdentity:'sharedLimitObjectIfPresentThenNativeConstructorRegistersAndRebindsAbility',contextPlayer:'effectContextPlayer',contextStage:'trigger',retriggeredByAbility:true},resolution:'game.resolveAbilityInNativeSubwindow',childLegality:'deferredExceptRequiredStateChangePreTargetCheck'};
 result.triggerEvent=defeated?{reuseOriginal:'onlyWhenOriginalExistsAndSourceNoLongerInPlay',inPlayTest:'canBeInPlayAndIsInPlayAtUseTime',otherwise:'newDefeatCardSystemWithCopiedPropertiesGenerateEvent(effectContext,source,true)',lastKnownInformation:'freshForStillInPlaySourceOtherwiseRetainOriginalEventSnapshot'}:{reuseOriginal:'whenPresent',otherwise:'newPlayCardSystem({playAsType:Any}).generateEvent(effectContext)'};
 return frame?{kind:'triggeredInventoryPropertiesFactory',parameters:frame.parameters,captures:frame.captures,constructionOrder:'readonlyDeclarationsThenObjectLeftToRight',captureIdentity:'valueOrPhysicalObjectReference',invocationPolicy:'eachNativePropertyGeneration',effect:result}:result;
}
module.exports={compileTriggeredInventorySystem};
