'use strict';

// These are native value-wrapper constructions, not snapshots of another card's
// currently compiled ability array. Their registrations and removal run when
// the ongoing framework applies/unapplies the value to each recipient.
function ongoingValueAdapter(call,ctx,duration,target,api) {
 const {access,expr,fail}=api,p=access(call.expression,ctx);
 if(p.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])||p[1]!=='ongoingEffects')return null;
 const name=p[2],names={cloneUnit:'CloneUnitEffect',gainNonKeywordAbilitiesFromUnit:'GainNonKeywordAbilitiesFromUnitEffect',copyStandardTriggeredAbilities:'CopyStandardTriggeredAbilitiesEffect',replaceKeyword:'ReplaceKeyword'};
 if(!Object.hasOwn(names,name))return null;
 const arity=name==='copyStandardTriggeredAbilities'?2:1;
 if(call.questionDotToken||call.arguments.length!==arity)fail(ctx,call,'ongoing-value-adapter-arity');
 const value={kind:'nativeOngoingValue',implementation:names[name],arguments:call.arguments.map(n=>api.value?api.value(n,{...ctx,ongoingExpression:true},api):expr(n,{...ctx,ongoingExpression:true})),gameArgument:'currentGame',constructAt:'ongoingGeneratorInvocation',sourceReferences:'physicalCardObjects'};
 if(name==='replaceKeyword')value.lifecycle={apply:'replaceEveryPrintedOrGainedFromKeywordWithToKeyword',numericValues:'carryOver',subsequentlyGainedKeywords:'affectedWhileOngoingEffectActive',properties:['from','to']};
 if(name==='cloneUnit')value.lifecycle={constructor:'followFirstCloneEffectUntilOriginalUnit',sourceRequirement:'unitWithStandardAbilitySetup',targetRequirement:'cloneUnit',apply:'invokeSourceSetupWithRecipientRegistrarUnlessSameInternalName',snapshot:'recipientPrintedActionTriggeredConstantPreEnterPlayUuidsBeforeApply',unapply:'removePrintedAbilitiesAbsentFromPreApplySnapshot',abilityCategories:['action','triggered','constant','preEnterPlay']};
 if(name==='copyStandardTriggeredAbilities')value.lifecycle={constructor:'asArrayAndValidateUnitsWithStandardAbilitySetup',apply:'invokeEachSourceSetupWithFilteredRecipientRegistrar',snapshot:'recipientPrintedTriggeredUuidsBeforeAllSources',unapply:'removePrintedTriggeredAbilitiesAbsentFromPreApplySnapshot',shorthandMethodByType:{whenPlayed:'addWhenPlayedAbility',whenPlayedUsingSmuggle:'addWhenPlayedAbility',whenDefeated:'addWhenDefeatedAbility',onAttack:'addOnAttackAbility',onDefense:'addOnDefenseAbility'},triggerKeysByType:{whenPlayed:['whenPlayed','whenPlayedUsingSmuggle'],whenPlayedUsingSmuggle:['whenPlayed','whenPlayedUsingSmuggle'],whenDefeated:['whenDefeated'],onAttack:['onAttack'],onDefense:['onDefense']},otherRegistrarMethods:'noop',ordinaryTriggeredRegistration:'retainMatchingWhenKeysOnly'};
 if(name==='gainNonKeywordAbilitiesFromUnit')value.lifecycle={sourceRequirement:'unit',readAbilitiesAt:'apply',abilityCategories:['triggeredIncludingReplacementAndDamageModification','action','constant'],apply:'addGainedAbilityFromSourceProperties',gainAbilitySource:'donor',abilitySource:'recipient',unapply:'removeExactGainedUuidsRecordedPerRecipient'};
 return {kind:'ongoingContract',scope:'card',effectType:name,target,duration,valueMode:'static',value};
}
module.exports={ongoingValueAdapter};
