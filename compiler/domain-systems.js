'use strict';

// Pinned immediate-system contracts. These data programs deliberately have no
// runtime adapter; native event identity and sub-window rules are retained for
// the later executor phase.
const {costCallback,playerTarget,scalarProperty}=require('./cost-resource-systems');
const {compileCallback}=require('./private-programs');
const constant=value=>({kind:'const',value});
function fields(obj,allowed,api){for(const [key,value]of obj.props)if(!allowed.includes(key))api.fail(obj.ctx,value,'domain-system-field:'+key);}
function boolean(obj,key,fallback,api){const v=obj.props.has(key)?api.literal(obj.props.get(key),obj.ctx):fallback;if(typeof v!=='boolean')api.fail(obj.ctx,obj.node,'domain-system-boolean:'+key);return v;}
function required(obj,key,api){if(!obj.props.has(key))api.fail(obj.ctx,obj.node,'domain-system-required:'+key);return obj.props.get(key);}
function domainSystem(kind,obj,defaultTarget,api){
 const {props,ctx,node}=obj;
 const target=()=>api.target(props.get('target'),ctx,defaultTarget);
 if(kind==='noAction'&&props.has('hasLegalTarget')&&!ctx.namedTargets?.size){fields(obj,['hasLegalTarget'],api);return {kind:'explicitNoActionSystem',event:'noAction',target:target(),hasLegalTarget:scalarProperty(props.get('hasLegalTarget'),ctx,api),canAffect:'hasLegalTargetProperty',targetTypeValidation:'alwaysTrue',handler:'emptyNativeHandler'};}
 if(['moveUnitFromSpaceToGround','moveUnitFromGroundToSpace'].includes(kind)){
  fields(obj,['target'],api);const space=kind==='moveUnitFromSpaceToGround';
  return {kind:'moveArenaSystem',target:target(),from:space?'spaceArena':'groundArena',to:space?'groundArena':'spaceArena',targetType:'unit',event:'onCardMoved',initialization:'doNotInitialize',upgradeContingents:{collection:'attachedUpgrades',event:'onCardMoved',destination:'sameAsHostEvent',order:'parentOrderPlusOne',initialization:'doNotInitialize'}};
 }
 if(kind==='defeatBase'){
  fields(obj,['target'],api);return {kind:'defeatBaseSystem',target:target(),targetType:'base',event:'onBaseDefeated',eligibility:'baseAndNotAlreadyDefeated',handler:'defeatBase'};
 }
 if(kind==='winGame'||kind==='loseGame'){
  fields(obj,kind==='winGame'?['target','winReason']:['target'],api);
  return {kind:'endGameSystem',result:kind==='winGame'?'win':'lose',target:playerTarget(props.get('target'),ctx,'ownPlayer',api),event:kind==='winGame'?'gameWon':'gameLost',winner:kind==='winGame'?'eventPlayer':'eventPlayerOpponent',endReason:'contextSourceTitle',...(kind==='winGame'?{winReason:costCallback(required(obj,'winReason',api),ctx,['context'],'winMessageGeneration',api),message:{format:'{0} uses {1} to win the game {2}',args:['eventPlayer','contextSource','winReason']}}:{})};
 }
 if(kind==='rescue'){
  fields(obj,['target','enterPlayEffect'],api);
  return {kind:'rescueSystem',target:target(),targetType:'nonLeaderUnit',event:'onRescue',eligibility:'unitAndCaptured',baseEligibilityMustChangeState:'none',handler:'contingentOnly',contingent:{kind:'putIntoPlay',target:'rescuedPhysicalCard',controller:'cardOwner',entersReady:'ownerHasRescuedUnitsEnterPlayReady',entryType:'rescued',...(props.has('enterPlayEffect')?{enterPlayEffect:api.effect(props.get('enterPlayEffect'),ctx)}:{})}};
 }
 if(kind==='takeControlOfResource'){
  fields(obj,['target'],api);
  return {kind:'takeControlResourceSystem',target:playerTarget(props.get('target'),ctx,'enemyPlayer',api),targetMeans:'recipientPlayer',exactlyOnePlayer:true,event:'onTakeControl',eligibility:'opponentHasResourcesWhenStateChangeRequired',selectionStage:'eventPropertiesGeneration',rearrangeResourceExhaustState:{player:'recipientOpponent',exhaustedCount:'preserved',order:'shuffledWithGameRandomGenerator',priority:'exhaustSmuggleFirstThenFillOriginalExhaustedCount'},selection:'soleReadyElseRandomReadyElseRandomResource',randomSource:'gameRandomGenerator',newController:'eventPlayer',preserveExhaustion:true,assertions:['canBeExhausted','canChangeController'],rollbackConfirmation:'ifNewControllerIsActingPlayerAndControllerChanges'};
 }
 if(kind==='collectBounty'){
  fields(obj,['target','bountyProperties','bountySource','forceResolve'],api);
  return {kind:'collectBountySystem',target:target(),bountyProperties:api.expr(required(obj,'bountyProperties',api),ctx),bountySource:props.has('bountySource')?api.expr(props.get('bountySource'),ctx):{kind:'undefined'},forceResolve:boolean(obj,'forceResolve',false,api),targetType:'unit',event:'onBountyCollected',eligibility:'forceResolveOrTargetEqualsPropertyBountySource',stateChangeRequires:'nonemptyBountyPropertiesUnlessForced',eventBountySource:'propertyBountySourceOrEventCard',propertiesNormalization:'asArray',abilityConstruction:{kind:'bounty',source:'eventBountySource',optional:false,contextPlayer:'eventContextPlayer',contextEvent:'collectionEvent'},abilityOrder:'actingPlayerChoosesEachRemainingBounty',nonemptyAssertion:true,childEffectLegality:'deferredToSubwindow'};
 }
 if(kind==='useOnAttackAbility'){
  fields(obj,['target','resolvedAbilityEvent'],api);
  return {kind:'useOnAttackAbilitySystem',target:target(),resolvedAbilityEvent:props.has('resolvedAbilityEvent')?api.expr(props.get('resolvedAbilityEvent'),ctx):constant(null),targetType:'unit',event:'onUseOnAttack',eligibility:{withoutResolvedEvent:'arenaOrResourceAndCanRegisterTriggeredAbilitiesAndHasOnAttack',withResolvedEvent:'eventContextTriggeredAndAbilityOnAttack',mustChangeState:'abilityMeetsRequirementsAtPreTargetWithOriginalAttackDeclaredEvent'},eventAbilities:'currentOnAttackAbilitiesOrResolvedEventAbility',attackDeclaredEvent:'resolvedAbilityEventContextEventOrNull',multipleAbilities:'actingPlayerSelectsOneUnlessEventTriggerAll',abilityConstruction:{kind:'triggered',source:'targetPhysicalCard',properties:'copiedAbilityProperties',optional:false,target:'targetPhysicalCard',contextPlayer:'eventContextPlayer',contextEvent:'originalAttackDeclaredEventOrNull'},childEffectLegality:'deferredToSubwindow'};
 }
 if(kind==='damage'&&!props.has('source')&&['isUnpreventable','isIndirect','ignoreAbilityDamageIncrease'].some(key=>props.has(key))){
  fields(obj,['target','amount','type','isCost','isIndirect','isUnpreventable','ignoreAbilityDamageIncrease'],api);
  if(props.has('type')&&api.literal(props.get('type'),ctx)!=='ability')api.fail(ctx,node,'protected-damage-type');
  return {kind:'protectedAbilityDamageSystem',target:target(),amount:costCallback(required(obj,'amount',api),ctx,['damageRecipient'],'perRecipientEventGeneration',api),damageType:'ability',event:'onDamageDealt',isCost:boolean(obj,'isCost',false,api),isIndirect:boolean(obj,'isIndirect',false,api),isUnpreventable:boolean(obj,'isUnpreventable',false,api),ignoreAbilityDamageIncrease:boolean(obj,'ignoreAbilityDamageIncrease',false,api),sourcePlayer:'contextPlayer',sourceCard:'contextSource',triggeredDefeatController:'triggerEventLastKnownController',pendingAbilityDamageIncrease:'addPositivePendingIncreaseAtEventGenerationUnlessIgnored',targetTypes:['unit','base'],eligibility:'attackableZoneAndInheritedCardTargetRestrictions',stateChangeRequires:'nonzeroAmountAndReceiveDamageRestrictionUnlessIndirectOrUnpreventable',effectiveUnpreventable:'explicitOrIndirectOrDamageSourceCardOngoingEffect',lastKnownInformation:'captureForNonBaseTargetAtEventUpdate',preResolutionExcess:'amountMinusMinAmountRemainingHp',preResolutionWillDefeat:'amountAtLeastRemainingHpAndNoCannotBeDefeatedByDamage',damageApplication:'cardAddDamage',postResolutionExcess:'amountMinusActualDamageDealt'};
 }
 if(kind==='combatDamage'||kind==='excessDamage'){
  const combat=kind==='combatDamage';fields(obj,combat?['target','amount','sourceAttack','source','type']:['target','sourceEventForExcessDamage','type'],api);
  const type=combat?'combat':'excess';if(props.has('type')&&api.literal(props.get('type'),ctx)!==type)api.fail(ctx,node,'domain-damage-type');
  return {kind:'typedDamageSystem',damageType:type,target:target(),targetTypes:['unit','base'],event:'onDamageDealt',...(combat?{amount:scalarProperty(required(obj,'amount',api),ctx,api),sourceAttack:api.expr(required(obj,'sourceAttack',api),ctx),source:props.has('source')?api.expr(props.get('source'),ctx):{kind:'undefined'},damageSource:'attackWithContextPlayerAndExplicitSourceOrAttackParticipants',explicitSourcesMustBeUnits:true}:{sourceEvent:api.expr(required(obj,'sourceEventForExcessDamage',api),ctx),amount:'sourceEventAvailableExcessDamageAtResolution',damageSource:'abilityWithContextPlayerAndSource',consumeSourceExcessAtResolution:true}),eligibility:'attackableZoneAndInheritedCardTargetRestrictions',stateChangeRequires:'nonzeroAmountOrAvailableExcessAndCanReceiveDamage',preResolutionExcess:'amountMinusMinAmountRemainingHp',preResolutionWillDefeat:'amountAtLeastRemainingHpAndNoCannotBeDefeatedByDamage',damageApplication:'cardAddDamage',postResolutionExcess:'amountMinusActualDamageDealt',unpreventable:'damageDealtByThisCardIsUnpreventableOnAnyDamageSourceCard',lastKnownInformation:'captureForNonBaseTargetAtEventUpdate',defaults:{isIndirect:false,isUnpreventable:false}};
 }
 if(['distributeExperienceAmong','distributeWeaknessAmong'].includes(kind)){
  fields(obj,['amountToDistribute','controller','cardTypeFilter','zoneFilter','maxTargets','canChooseNoTargets','canDistributeLess','cardCondition','player','activePromptTitle'],api);
  required(obj,'canChooseNoTargets',api);const zero=boolean(obj,'canChooseNoTargets',null,api),less=boolean(obj,'canDistributeLess',false,api);if(less&&!zero)api.fail(ctx,node,'token-distribution-less-needs-zero');
  const chooser=props.has('player')?api.literal(props.get('player'),ctx):'self';if(!['self','opponent'].includes(chooser))api.fail(ctx,node,'token-distribution-chooser');
  return {kind:'distributeTokenUpgradeSystem',tokenType:kind==='distributeExperienceAmong'?'experience':'weakness',amount:costCallback(required(obj,'amountToDistribute',api),ctx,['context'],'distributionQuantity',api),targetResolver:api.targetResolverProperties(obj),canChooseNoTargets:zero,canDistributeLess:less,chooser,maxTargets:props.has('maxTargets')?api.literal(props.get('maxTargets'),ctx):null,...(props.has('activePromptTitle')?{promptTitleParts:api.contextTitle(props.get('activePromptTitle'),ctx)}:{}),selectorMode:'unlimited',eligibility:'oneTokenChildCanAffectAndConfiguredCardCondition',zeroAmount:'noEvents',noEnoughTargets:'noEvents',autoSingle:'singleLegalTargetAndNotCanChooseNoTargetsAndNotAbilityOptional',promptCanChooseNone:'canChooseNoTargetsOrAbilityOptional',event:'distributeTokenUpgrade',childEvent:'giveTokenUpgradePerRecipient',childEventOrder:'distributionOrderMinusOne',resultAmount:'sumResolvedIndividualEventsMatchingChildName',tokenOwnership:'recipientController',tokenCreation:'batchedPerRecipient'};
 }
 if(kind==='handler')return handlerSystem(obj,api);
 return null;
}
function handlerSystem(obj,api){
 const {ts,unwrap,fail}=api,{ctx,props,node}=obj;fields(obj,['handler','effectMessage','hasTargetsChosenByInitiatingPlayer'],api);
 const h=unwrap(required(obj,'handler',api));if(!ts.isArrowFunction(h)||h.modifiers?.length||h.parameters.length>1||h.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken))fail(ctx,h,'handler-function-shape');
 let body=unwrap(h.body),program;
 if(ts.isBlock(body)&&body.statements.length===0)program={kind:'emptyHandlerProgram'};
 else {
  if(ts.isBlock(body)){if(body.statements.length!==1||!ts.isExpressionStatement(body.statements[0]))fail(ctx,body,'handler-program-statement');body=unwrap(body.statements[0].expression);}
  if(!ts.isBinaryExpression(body)||body.operatorToken.kind!==ts.SyntaxKind.EqualsToken||!ts.isElementAccessExpression(body.left)||body.left.questionDotToken||!ts.isIdentifier(body.left.expression)||h.parameters[0]?.name.text!==body.left.expression.text||!ts.isStringLiteral(body.left.argumentExpression))fail(ctx,body,'handler-program-context-assignment');
  const slot=body.left.argumentExpression.text;if(!/^_[A-Za-z][A-Za-z0-9_]*$/.test(slot))fail(ctx,body,'handler-program-private-context-slot');
  const root='handlerContext:'+h.getStart(ctx.source),local={...ctx,scope:new Map(ctx.scope),constants:new Map(ctx.constants),dataRoots:new Set([...(ctx.dataRoots||[]),root])};local.scope.set(h.parameters[0].name.text,{kind:'ref',path:[root]});local.constants.delete(h.parameters[0].name.text);
  program={kind:'contextSlotAssignment',parameter:{name:root,role:'eventContext'},slot,value:api.expr(body.right,local),returns:'assignedValue',identity:'existingAbilityContext'};
 }
 return {kind:'executeHandlerSystem',event:'executeHandler',handler:program,hasLegalTarget:true,canAffect:true,hasTargetsChosenByInitiatingPlayer:boolean(obj,'hasTargetsChosenByInitiatingPlayer',false,api),...(props.has('effectMessage')?{effectMessage:compileCallback(props.get('effectMessage'),{...ctx,domainHandlerMessage:true},api,['context'])}:{})};
}
function domainMessageExpression(input,ctx,api){
 const {ts,unwrap,expr,fail,unshadowedEnum}=api,n=unwrap(input);
 if(!ctx.domainHandlerMessage||!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||!ts.isIdentifier(n.expression.expression)||n.expression.expression.text!=='TextHelper'||n.expression.name.text!=='aspect')return null;
 if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==1)fail(ctx,n,'handler-aspect-message-call');unshadowedEnum(n.expression.expression,ctx,'TextHelper');
 return {kind:'aspectDisplay',value:expr(n.arguments[0],ctx),testFormat:'capitalize',productionFormat:':value:',environment:'nativeNodeEnv'};
}
function bountyKeywordExpression(input,ctx,api){
 const {ts,unwrap,expr,fail}=api,n=unwrap(input);
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression))return null;
 const call=n.expression;
 if(call.name.text==='getBountyKeywords'){
  if(n.questionDotToken||call.questionDotToken||n.arguments.length)fail(ctx,n,'bounty-keyword-query-shape');
  return {kind:'bountyKeywordInventory',receiver:expr(call.expression,ctx),evaluation:'expressionEvaluation',inventory:'printedPlusGainedThenBlankFilterThenReplacementsThenDedupe',keyword:'bounty',filter:'fullyImplementedAbilityDefinitions',preserveInstancesAndOrder:true};
 }
 if(call.name.text!=='map'||n.arguments.length!==1)return null;
 const base=unwrap(call.expression);if(!ts.isCallExpression(base)||!ts.isPropertyAccessExpression(base.expression)||base.expression.name.text!=='getBountyKeywords')return null;
 const callback=unwrap(n.arguments[0]);if(n.questionDotToken||call.questionDotToken||!ts.isArrowFunction(callback)||callback.modifiers?.length||callback.parameters.length!==1||!ts.isIdentifier(callback.parameters[0].name)||callback.parameters[0].initializer||callback.parameters[0].dotDotDotToken||callback.parameters[0].questionToken)fail(ctx,n,'bounty-keyword-projection-callback');
 let body=unwrap(callback.body);if(ts.isBlock(body)){if(body.statements.length!==1||!ts.isReturnStatement(body.statements[0])||!body.statements[0].expression)fail(ctx,body,'bounty-keyword-projection-body');body=unwrap(body.statements[0].expression);}
 if(!ts.isPropertyAccessExpression(body)||body.questionDotToken||body.name.text!=='abilityProps'||!ts.isIdentifier(body.expression)||body.expression.text!==callback.parameters[0].name.text)fail(ctx,body,'bounty-keyword-projection-field');
 return {kind:'bountyAbilityProperties',keywords:bountyKeywordExpression(base,ctx,api),projection:'abilityProps',evaluationOrder:'nativeArrayMapOrder',snapshot:'propertyReferencesAtExpressionEvaluation'};
}
module.exports={domainSystem,domainMessageExpression,bountyKeywordExpression};
