'use strict';

// Compiler-only contracts for native view/deck and token-option systems. None
// of these operations is an alias for an executable scalar runtime effect.
function viewDeckSystem(kind,obj,defaultTarget,api,call) {
 const {ts,unwrap,expr,effect,target,literal,contextTitle,fail}=api,{props,ctx}=obj;
 const c=value=>({kind:'const',value});
 const fields=names=>{for(const [key,value]of props)if(!names.includes(key))fail(ctx,value,'view-deck-field:'+kind+':'+key);};
 const value=(key,fallback)=>props.has(key)?expr(props.get(key),ctx):fallback;
 const captureBoundary=()=>{
  const visit=(node,shadowed=new Set())=>{
   const n=unwrap(node);
   if(ts.isArrowFunction(n)||ts.isFunctionExpression(n)){shadowed=new Set(shadowed);for(const p of n.parameters)if(ts.isIdentifier(p.name))shadowed.add(p.name.text);}
   if(ts.isIdentifier(n)&&!shadowed.has(n.text)&&ctx.constants.has(n.text)&&ctx.constantStages?.get(ctx.constants.get(n.text))){
    const parent=n.parent,propertyName=(ts.isPropertyAccessExpression(parent)&&parent.name===n)||(ts.isPropertyAssignment(parent)&&parent.name===n);
    if(!propertyName)fail(ctx,n,'view-deck-saved-binding-requires-construction-frame');
   }
   ts.forEachChild(n,c=>visit(c,shadowed));
  };
  for(const node of props.values())visit(node);
 };

 if(['lookAt','reveal'].includes(kind)&&props.has('displayTextByCardUuid')) {
  captureBoundary();fields(['target','player','useDisplayPrompt','activePromptTitle','message','messageArgs','displayTextByCardUuid',...(kind==='reveal'?['promptedPlayer']:[])]);
  const reveal=kind==='reveal';
  const result={kind:reveal?'revealWithCardDisplayText':'lookAtWithCardDisplayText',target:target(props.get('target'),ctx,defaultTarget),player:value('player',{kind:'ref',path:['player']}),displayTextByCardUuid:value('displayTextByCardUuid'),useDisplayPrompt:value('useDisplayPrompt',c(null)),interactMode:'viewOnly',nativeSystem:'LookAtSystem',event:'onLookAtCard',defaultDisplayPrompt:false,displayTextKey:'physicalCardUuid',displayTextIdentity:'originalMapReference',eventCondition:'alwaysTrue'};
  if(reveal){result.nativeSystem='RevealSystem';result.event='onCardRevealed';result.eventCondition='allCardsStillRevealable';result.legalZones=['deck','hand','resource'];result.revealedFromZone='sharedZoneIfAllSame';result.promptedPlayer=value('promptedPlayer',c('self'));}
  for(const key of ['activePromptTitle','message'])if(props.has(key))result[key+'Parts']=contextTitle(props.get(key),ctx);
  if(props.has('messageArgs'))result.messageArgs=expr(props.get('messageArgs'),ctx,['viewCards']);
  return result;
 }
 if(kind==='returnToHand'&&props.has('attachedUpgradeOverrideHandler')) {
  captureBoundary();fields(['target','attachedUpgradeOverrideHandler']);
  const binding=require('./attack-initiation').bindAttackCallback(props.get('attachedUpgradeOverrideHandler'),ctx,['attachedUpgrade','attachmentCleanupContext'],api);
  const systemOrNull=input=>{
   const n=unwrap(input);
   if(n.kind===ts.SyntaxKind.NullKeyword)return {kind:'noAttachmentOverride'};
   if(ts.isConditionalExpression(n))return {kind:'conditionalAttachmentOverride',condition:expr(n.condition,binding.lowered.ctx),whenTrue:systemOrNull(n.whenTrue),whenFalse:systemOrNull(n.whenFalse),evaluation:'chosenBranchOnly'};
   return {kind:'attachmentOverrideSystem',system:effect(n,binding.lowered.ctx,'attachedUpgrade')};
  };
  return {kind:'returnToHandWithAttachmentOverride',target:target(props.get('target'),ctx,defaultTarget),destination:'hand',nativeSystem:'MoveCardSystem',event:'onCardMoved',override:{parameters:binding.parameters,body:systemOrNull(binding.lowered.node),invokeAt:'contingentEventGenerationForEachAttachedUpgrade',context:'leavingUnitEffectContext'},noOverride:'frameworkDefeatWithFrameworkContext',contingentOrder:'parentEventOrderMinusOne',attachmentIteration:'physicalAttachmentArrayOrder',leavesPlay:'nativeCleanupAndLeaderOrTokenReplacement'};
 }
 if(kind==='lookMoveDeckCardsTopOrBottom') {
  captureBoundary();
  fields(['amount','target']);if(!props.has('amount'))fail(ctx,obj.node,'look-move-amount-required');
  return {kind:'lookMoveDeckCards',amount:value('amount'),target:target(props.get('target'),ctx,defaultTarget==='self'?'ownPlayer':defaultTarget),nativeSystem:'LookMoveDeckCardsTopOrBottomSystem',targetAssertion:'exactlyOnePlayer',viewer:'contextPlayer',captureAt:'queueGenerateEventGameSteps',targetCards:'targetPlayer.drawDeck.slice(0,min(amount,deckLength))',singleCardDeck:'lookAtOnlyRegardlessOfAmount',emptyDeck:'emptyDisplayPrompt',choices:[{text:'Put on top',arg:'top',destination:'deck top'},{text:'Put on bottom',arg:'bottom',destination:'deck bottom'}],promptTitle:'Select card to move to the top or bottom of the deck',moveEventCreation:'eachPromptClickInClickOrder',rollbackConfirmation:'contextPlayerWhenDeckNonempty',legality:'emptyDeckIllegalOnlyWhenStateChangeRequired'};
 }
 if(kind==='revealAndSelectCard'||kind==='lookAtAndSelectCard'&&props.has('displayTextByCardUuid')) {
  captureBoundary();
  fields(['target','player','promptedPlayer','useDisplayPrompt','activePromptTitle','canChooseFewer','maxCards','cardCondition','immediateEffect','noSelectedCardsButtonText','message','messageArgs','displayTextByCardUuid']);
  const result={kind:'revealSelectCards',target:target(props.get('target'),ctx,defaultTarget),promptedPlayer:value('promptedPlayer',c('self')),maxCards:value('maxCards',c(1)),canChooseFewer:value('canChooseFewer',c(true)),useDisplayPrompt:value('useDisplayPrompt',c(true)),cardCondition:props.has('cardCondition')?expr(props.get('cardCondition'),ctx,['card','context']):c(true),nativeSystem:'RevealSystem',interactMode:'selectSingle',legalZones:['deck','hand','resource'],event:'onCardRevealed',eventRecheck:'allCardsStillRevealable',revealedFromZone:'sharedZoneIfAllSame',selectedContextField:'selectedPromptCards',innerDefaultTarget:'selectedPhysicalCardList',innerResolution:'nonemptySelectionThenChildEventWindow',displayAssertion:'cannotDisableSelectionDisplayPrompt',filter:'predicateAndInnerSystemCanAffect'};
  if(kind==='lookAtAndSelectCard'){result.kind='lookAtSelectCardsWithDisplayText';result.nativeSystem='LookAtSystem';result.event='onLookAtCard';result.eventRecheck='alwaysTrue';delete result.promptedPlayer;delete result.legalZones;delete result.revealedFromZone;result.player=value('player',{kind:'ref',path:['player']});}
  if(props.has('displayTextByCardUuid')){result.displayTextByCardUuid=value('displayTextByCardUuid');result.displayTextKey='physicalCardUuid';result.displayTextIdentity='originalMapReference';}
  if(props.has('immediateEffect'))result.immediateEffect=effect(props.get('immediateEffect'),{...ctx,pendingSelection:true},'viewSelected');
  if(props.has('player'))result.player=value('player'); // retained property; RevealSystem chooses via promptedPlayer
  for(const key of ['activePromptTitle','message','noSelectedCardsButtonText'])if(props.has(key))result[key+'Parts']=contextTitle(props.get(key),ctx);
  if(props.has('messageArgs'))result.messageArgs=expr(props.get('messageArgs'),ctx,['viewCards']);
  return result;
 }
 if(kind==='moveToBottomOfDeck'&&props.has('shuffleMovedCards')) {
  captureBoundary();
  fields(['target','shuffleMovedCards','shuffle']);
  const original=ts.getOriginalNode(call),input=original.arguments[0]&&unwrap(original.arguments[0]);
  const factory=!!input&&(ts.isArrowFunction(input)||ts.isFunctionExpression(input));
  return {kind:'moveDeckCards',target:target(props.get('target'),ctx,defaultTarget),destination:'deck bottom',shuffle:value('shuffle',c(false)),shuffleMovedCards:value('shuffleMovedCards'),nativeSystem:'MoveCardSystem',propertySource:factory?'propertyFactory':'staticProperties',targetProcessing:{shuffleRead:'constructorStaticPropertiesOnly',factoryShuffleMovedCardsIgnored:true,mutation:'inPlaceGeneratedTargetArray',randomGenerator:'game.randomGenerator',timing:'beforeTargetIterationAndEventCreation'},destinationPlayer:'eachCardOwner',event:'onCardMoved',leavesPlay:'nativeMoveCardCleanupAndLeaderReplacement',eventTargetOrder:'processedTargetArrayOrder'};
 }
 if(kind==='useTheForce'&&(defaultTarget!=='self'||props.size)) {
  captureBoundary();
  fields(['target','isCost','cannotBeCancelled']);
  return {kind:'useForceSystem',target:props.has('target')?expr(props.get('target'),ctx):ctx.selectedPlayerTarget||{kind:'inheritedSystemTarget',binding:defaultTarget==='self'?null:defaultTarget,fallback:'nativeUseForceTargets'},nativeDefaultTargets:'contextPlayerOnlyWhenHasTheForce',isCost:value('isCost',c(false)),cannotBeCancelled:value('cannotBeCancelled',c(false)),nativeSystem:'UseTheForceSystem',legalTarget:'player',requiresForce:'costOrRequiredStateChange',eventUpdateAssertion:'targetPlayerHasTheForce',event:'onCardLeavesPlay',card:'targetPlayer.baseZone.forceToken',lastKnownInformation:'forceTokenAtEventUpdate',destination:'outsideTheGame',contingentEvent:'onForceUsed'};
 }
 const tokens={giveShield:'shield',giveExperience:'experience',giveAdvantage:'advantage'};
 if(tokens[kind]&&(props.has('createdBy')||props.has('highPriorityRemoval'))&&(kind!=='giveAdvantage'||ctx.pendingSelection||ctx.pendingRegistration)) {
  captureBoundary();
  fields(['target','amount','createdBy','highPriorityRemoval']);
  return {kind:'giveTokenUpgradeOptions',tokenType:tokens[kind],target:target(props.get('target'),ctx,defaultTarget),amount:value('amount',c(1)),createdBy:value('createdBy',c('self')),highPriorityRemoval:value('highPriorityRemoval',c(false)),priorityAppliesTo:'shieldOnly',nativeSystem:'GiveTokenUpgradeSystem',event:'onTokensCreated',eventBatch:'allLegalTargetUnitsOneCreationEvent',eventPlayer:'opponentOnlyWhenCreatedByOpponentAndOpponentExists',owner:'recipientUnitController',generationTiming:'eventUpdateBeforeContingentAttachment',eventRecheck:'anyTargetStillAffectable',tokenEventField:'generatedTokens',attachmentContingents:'targetOrderThenPerTargetTokenOrderAtParentOrderPlusOne',amountZero:'cannotAffect'};
 }
 return null;
}
function modalChoiceDefinitions(input,context,api,defaultTarget='self'){
 const {ts,unwrap,lambda,contextTitle,effect,fail}=api;
 const n=unwrap(input),factory=ts.isArrowFunction(n)||ts.isFunctionExpression(n);
 const frame=factory?require('./system-array-factory').constructionFrame(input,context,['modalChoiceContext'],'modalChoiceFactoryInvocation',api):null;
 const lowered=frame?{node:frame.node,ctx:frame.ctx}:lambda(input,context),node=unwrap(lowered.node),ctx=lowered.ctx;
 if(!ts.isObjectLiteralExpression(node)||!node.properties.length||node.properties.length>32)fail(ctx,node,'modal-choice-map-object');
 const readonlyCaptures=frame?frame.captures.slice():[],entries=[];
 for(const p of node.properties){
  if(!ts.isPropertyAssignment(p))fail(ctx,p,'modal-choice-property');
  const label=ts.isComputedPropertyName(p.name)?p.name.expression:ts.isIdentifier(p.name)?ts.factory.createStringLiteral(p.name.text):p.name;
  const labelParts=contextTitle(label,ctx),start=frame?.captures.length;
  if(frame)frame.walk(p.initializer);
  entries.push({labelParts,...(frame?{constructionCaptures:frame.captures.slice(start)}:{}),effect:effect(p.initializer,ctx,defaultTarget)});
 }
 return {kind:'modalChoiceDefinitions',entries,keyOrder:'javascriptPropertyOrder',duplicatePolicy:'lastAssignmentWins',evaluationOrder:'eachKeyThenValueLeftToRight',keyCoercion:'javascriptPropertyKey',keyEvaluation:'choicesObjectConstruction',...(frame?{parameters:frame.parameters,captures:readonlyCaptures,invocationPolicy:'eachNativeChoiceGeneration'}:{})};
}
module.exports={viewDeckSystem,modalChoiceDefinitions};
