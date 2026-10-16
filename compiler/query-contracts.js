'use strict';
const {bindAttackCallback}=require('./attack-initiation');
// Pending executor contracts. Each node stays at its source expression's
// evaluation site; lexical construction frames own any earlier snapshots.
function queryContract(input,ctx,api) {
 const {ts,unwrap,expr,object,fail,unshadowedEnum}=api,n=unwrap(input);
 const sourceModule=moduleName=>require('path').posix.resolve('/forceteki/server',require('path').posix.dirname(ctx.file),moduleName).replace(/\.js$/,'');
 const namespace=(node,name)=>{unshadowedEnum(node,ctx,name);const imported=ctx.source.statements.some(s=>ts.isImportDeclaration(s)&&sourceModule(s.moduleSpecifier.text)==='/forceteki/server/game/core/utils/'+name&&s.importClause&&!s.importClause.isTypeOnly&&s.importClause.namedBindings&&ts.isNamedImports(s.importClause.namedBindings)&&s.importClause.namedBindings.elements.some(e=>e.name.text===name&&!e.isTypeOnly&&(e.propertyName?.text||e.name.text)===name));if(!imported)fail(ctx,node,'query-contract-namespace-import:'+name);};
 const stage=()=>{if(!ctx.callbackStage&&!ctx.pureProgram&&!ctx.eventHistory)fail(ctx,n,'query-contract-evaluation-stage');};
 const args=(arity,requiresStage=true)=>{if(requiresStage)stage();if(n.arguments.length!==arity||n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'query-contract-arity-or-optional');};
 const receiver=v=>expr(v,{...ctx,arenaReceiver:true});
 const valuePath=v=>v.kind==='ref'?v.path:v.kind==='dataPath'?[...v.root,...v.steps.map(s=>s.key)]:['constructionMember','queryMember'].includes(v.kind)&&v.property?[...valuePath(v.receiver),v.property]:[];
 const cb=(v,roles)=>{const b=bindAttackCallback(v,ctx,roles,api);return {kind:'queryCallback',parameters:b.parameters,body:expr(b.lowered.node,b.lowered.ctx)};};
 // Explicit lexical context reads retain the callback object that produced
 // an earlier result. Optional chaining is kept step by step, including its
 // continuation flag, instead of substituting the current target register.
 const contextPath=(v)=>{if(ts.isIdentifier(v))return {root:v,steps:[]};if(ts.isPropertyAccessExpression(v)){const p=contextPath(v.expression);return p&&{...p,steps:[...p.steps,{property:v.name.text,optional:!!v.questionDotToken,continuesOptionalChain:!!(v.flags&ts.NodeFlags.OptionalChain)}]};}if(ts.isElementAccessExpression(v)){const p=contextPath(v.expression);return p&&{...p,steps:[...p.steps,{index:v.argumentExpression,optional:!!v.questionDotToken,continuesOptionalChain:!!(v.flags&ts.NodeFlags.OptionalChain)}]};}return null;};
 const lexical=contextPath(n);
 if(lexical&&ctx.scope.get(lexical.root.text)?.path?.join('.')==='context'&&!ctx.constants.has(lexical.root.text)) {
  const first=lexical.steps[0]?.property;
  if(['costs','activeAttackId'].includes(first)||['source','target','targets','events','resolvedEvents'].includes(first)&&lexical.steps.some(s=>s.optional||s.property==='lastKnownInformation')) {
   const allowed=new Set(['costs','activeAttackId','source','target','targets','events','resolvedEvents','lastKnownInformation','card','arena','zoneName','zone','aspects','traits','title','subtitle','cost','power','hp','type','name','length']);
   if(lexical.steps.length>12||lexical.steps.some((s,i)=>s.property&&!allowed.has(s.property)&&!(['targets','costs'].includes(lexical.steps[i-1]?.property)&&/^[A-Za-z0-9_]{1,64}$/.test(s.property))))fail(ctx,n,'lexical-query-context-property');
   return {kind:'lexicalContextQuery',contextBinding:require('./event-result-context').contextBinding(lexical.root,ctx,api),steps:lexical.steps.map(s=>s.index?{...s,index:expr(s.index,ctx)}:s),evaluation:'expressionEvaluation'};
  }
 }
 if(ts.isElementAccessExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&ts.isIdentifier(n.expression.expression)&&n.expression.expression.text==='EnumHelpers'&&n.expression.name.text==='tokenTitle') {
  namespace(n.expression.expression,'EnumHelpers');if(n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'enum-token-title-optional');
  const labels={'TokenUnitName.BattleDroid':'Battle Droid','TokenUnitName.CloneTrooper':'Clone Trooper','TokenUnitName.XWing':'X-Wing','TokenUnitName.TIEFighter':'TIE Fighter','TokenUnitName.Spy':'Spy','TokenUnitName.Mandalorian':'Mandalorian','TokenUnitName.Beast':'Beast','TokenUpgradeName.Shield':'Shield','TokenUpgradeName.Experience':'Experience','TokenUpgradeName.Advantage':'Advantage','TokenUpgradeName.Weakness':'Weakness','TokenCardName.Credit':'Credit','TokenCardName.Force':'The Force'},entries={};for(const[key,label]of Object.entries(labels)){const value=ctx.enums[key];if(typeof value!=='string')fail(ctx,n,'enum-token-title-dependency');entries[value]=label;}
  return {kind:'enumRecordLookup',record:'EnumHelpers.tokenTitle',entries,index:expr(n.argumentExpression,ctx),propertyKeyCoercion:'native',missing:'undefined',evaluation:'expressionEvaluation'};
 }
 // These constructors and adapters preserve object identity. They are values,
 // never eager system execution or a replacement for construction captures.
 if(ts.isNewExpression(n)&&ts.isIdentifier(n.expression)&&['Set','Map'].includes(n.expression.text)) {
  unshadowedEnum(n.expression,ctx,n.expression.text);if((n.arguments?.length||0)>1)fail(ctx,n,'query-set-constructor-arity');
  return {kind:n.expression.text==='Set'?'querySetConstruction':'queryMapConstruction',...(n.expression.text==='Map'?{entries:'iterablePairs',duplicateKey:'lastValueFirstKeyInsertionOrder'}:{}),values:n.arguments?.length?expr(n.arguments[0],{...ctx,allowArenaList:true}):{kind:'array',items:[]},nullishInput:'empty',iteration:'nativeIterableOrder',equality:'sameValueZero',identity:'freshPerExpressionEvaluation'};
 }
 if(ts.isPropertyAccessExpression(n)&&['isInitiativeClaimed','currentPhase','roundNumber','actionNumber'].includes(n.name.text)) {
  const game=receiver(n.expression),p=game.kind==='ref'?game.path:[];
  if(p.at(-1)==='game')return {kind:'gamePropertyQuery',game,property:n.name.text,evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(n)&&n.text==='undefined'&&!ctx.scope.has('undefined')&&!ctx.constants.has('undefined')){unshadowedEnum(n,ctx,'undefined');return {kind:'undefined'};}
 if(ts.isArrayLiteralExpression(n)&&n.elements.some(e=>ts.isSpreadElement(e))){return {kind:'queryArrayConstruction',items:n.elements.map(e=>ts.isSpreadElement(e)?{kind:'spread',value:expr(e.expression,{...ctx,allowArenaList:true})}:{kind:'element',value:expr(e,ctx)}),evaluationOrder:'leftToRight'};}
 if(ts.isPropertyAccessExpression(n)&&n.name.text==='length'){const list=expr(n.expression,{...ctx,allowArenaList:true});if(['scopedArenaUnitsQuery','controlledInPlayListQuery','gamePlayersQuery','gamePlayerCollection'].includes(list.kind)||list.kind==='query'&&list.name==='getCardsInZone')return {kind:'queryListLength',receiver:list,evaluation:'expressionEvaluation'};}
 if(ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n)) {
  const member=ts.isPropertyAccessExpression(n)?n.name.text:null;
  const allowed=['length','size','title','subtitle','cost','power','hp','aspects','traits','zoneName','zone','exhausted','damage','remainingHp','parentCard','controller','owner','type','name','card','lastKnownInformation','selectedTargets'];
  if(!member||allowed.includes(member)) {
   const base=expr(n.expression,{...ctx,allowArenaList:true});
   const family=['arrayAdapterQuery','queryMember','querySetConstruction','queryMapConstruction','pureSet','pureMap','setAlgebraQuery','bountyKeywordListQuery','privateProgramCall','privateConstructionCall','pureCollectionCall','pureUtility','dataCall','dataMember','dataPath','queryCollectionCall','arrayFromQuery','arenaList','arenaFilter','controlledInPlayListQuery','lexicalContextQuery','scopedArenaUnitsQuery','gamePlayersQuery','gamePlayerCollection'];
   const optional=!!n.questionDotToken;
   if(family.includes(base.kind)||optional&&base.kind==='ref')return {kind:'queryMember',receiver:base,...(member?{property:member}:{index:expr(n.argumentExpression,ctx)}),optionalReceiver:optional,continuesOptionalChain:!!(n.flags&ts.NodeFlags.OptionalChain),evaluation:'expressionEvaluation'};
  }
 }
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression))return null;
 const call=n.expression,name=call.name.text,base=unwrap(call.expression);
 if(ts.isIdentifier(base)&&base.text==='Math'&&['min','max'].includes(name)&&n.arguments.some(a=>ts.isSpreadElement(a))) {
  unshadowedEnum(base,ctx,'Math');if(n.questionDotToken||call.questionDotToken)fail(ctx,n,'query-math-spread-optional');
  return {kind:'mathSpreadQuery',operation:name,arguments:n.arguments.map(a=>ts.isSpreadElement(a)?{spread:expr(a.expression,{...ctx,allowArenaList:true})}:{value:expr(a,ctx)}),evaluationOrder:'argumentsAndIteratorsLeftToRightThenNumericCoercionLeftToRight',emptyResult:name==='min'?'positiveInfinity':'negativeInfinity',nan:'propagate',signedZero:'nativeMinMax',argumentLimit:'nativeArgumentListLimit'};
 }
 if(name==='flat') {
  args(0,false);return {kind:'arrayFlatQuery',receiver:expr(base,{...ctx,allowArenaList:true}),depth:1,order:'nativeArrayIndexOrder',holes:'removedAtFlattenedLevels',identity:'freshArray',evaluation:'expressionEvaluation'};
 }
 if(name==='createCopy') {
  args(1,false);if(!ts.isIdentifier(base))fail(ctx,n,'query-context-copy-receiver');
  const binding=ctx.scope.get(base.text),slot=binding?.path?.join('.');
  let typedContext=false;for(let ancestor=n.parent;ancestor;ancestor=ancestor.parent){const parameter=ancestor.parameters?.find(p=>ts.isIdentifier(p.name)&&p.name.text===base.text);if(parameter){typedContext=parameter.type?.getText(ctx.source).includes('AbilityContext')||binding?.path?.join('.')==='context';break;}}
  if(!typedContext||!slot)fail(ctx,n,'query-context-copy-proof');const o=object(n.arguments[0],ctx);for(const[key,v]of o.props)if(key!=='player')fail(o.ctx,v,'query-context-copy-field:'+key);
  return {kind:'abilityContextCreateCopy',context:expr(base,ctx),overrides:Object.fromEntries([...o.props].map(([key,v])=>[key,expr(v,o.ctx)])),constructs:'sameNativeContextClassByVirtualCreateCopyDispatch',identity:'freshContext',getProps:{shared:['game','source','player','ability','limit','ongoingEffect','costAspects','events','stage','targetAbility','playType','selectedPromptCards'],shallowCopies:['costs','targets','selects']},derivedContextFields:{TriggeredAbilityContext:['event','overrideTitle','retriggeredByAbility'],ReplacementEffectContext:['event','overrideTitle','retriggeredByAbility','replacementEffectWindow']},allowedClasses:['AbilityContext','TriggeredAbilityContext','ReplacementEffectContext'],unknownClass:'reject',copyMethod:'createCopyNotCopy',notCopied:['target','select','subResolution','choosingPlayerOverride','gameActionsResolutionChain','activeAttackId','cardStateWhenInitiated','pendingAbilityDamageIncrease'],resetDefaults:{target:'undefined',select:'undefined',subResolution:false,choosingPlayerOverride:null,gameActionsResolutionChain:'freshArray',activeAttackId:'undefined',cardStateWhenInitiated:null,pendingAbilityDamageIncrease:0},evaluationOrder:'receiverThenOverrideObjectThenGetPropsThenObjectAssignThenConstructor',constructorDefaults:'pinnedAbilityContext',evaluation:'expressionEvaluation'};
 }
 if(name==='hasLegalTarget') {
  args(1,false);if(!ts.isIdentifier(base))fail(ctx,n,'query-system-legality-receiver');
  let declaration=null;for(let ancestor=n.parent;ancestor;ancestor=ancestor.parent){if(ancestor.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===base.text))break;if(ts.isBlock(ancestor)){for(const statement of ancestor.statements)if(ts.isVariableStatement(statement)&&statement.declarationList.flags&ts.NodeFlags.Const){const found=statement.declarationList.declarations.find(d=>ts.isIdentifier(d.name)&&d.name.text===base.text);if(found){declaration=found;break;}}if(declaration)break;}}
  const binding=ctx.scope.get(base.text);if(!declaration?.initializer||binding?.path?.join('.')!=='data:local:'+declaration.getStart(ctx.source)||ctx.programLocals?.get(binding.path.join('.'))!==false)fail(ctx,n,'query-system-legality-provenance');
  const init=unwrap(declaration.initializer);if(!ts.isCallExpression(init)||!ts.isPropertyAccessExpression(init.expression)||init.expression.name.text!=='playCardFromOutOfPlay'||!ts.isPropertyAccessExpression(init.expression.expression)||init.expression.expression.name.text!=='immediateEffects'||!ts.isIdentifier(init.expression.expression.expression)||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(init.expression.expression.expression.text))fail(ctx,n,'query-system-legality-dispatch');
  return {kind:'constructedSystemLegalityQuery',system:expr(base,ctx),context:expr(n.arguments[0],ctx),additionalProperties:{kind:'record',properties:{}},mustChangeGameState:'none',dispatch:{system:'PlayCardSystem',method:'GameSystem.hasLegalTarget',targets:'validateContextThenGeneratePropertiesThenAsArray',targetFilter:'falsyTargetsRemoved',targetGenerationError:'propagate',iteration:'targetOrderShortCircuitAny',emptyResult:false,perTarget:'PlayCardSystem.canAffect',canAffectError:'reportErrorToContextGameThenFalse',cardTypeFilter:['basicUnit','basicUpgrade','event'],playableRequired:true,pendingDefeatUnit:'reject',playAsType:'pinnedWildcardAndPilotingRules',playActions:'nativePlayActionsForPlayType',actionRequirements:'createContextForPassedPlayerThenMeetsRequirementsWithIgnoredRequirements'},receiverIdentity:'existingConstructedSystemObject',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='Array'&&name==='from') {
  args(1,false);unshadowedEnum(base,ctx,'Array');return {kind:'arrayFromQuery',value:expr(n.arguments[0],{...ctx,allowArenaList:true}),input:'iterableOrArrayLike',order:'nativeIteratorOrIndexOrder',identity:'freshArray',nullishInput:'throwTypeError',evaluation:'expressionEvaluation'};
 }
 if(name==='has') {
  args(1,false);const set=expr(base,ctx);const helper=set.kind==='privateProgramCall'&&ctx.privatePrograms?.get(set.name);
  const declaredSet=helper?.member?.type?.getText(ctx.source).startsWith('Set<');
  const setPath=valuePath(set);const ownedSet=setPath.length&&ctx.ownedCollections?.get(setPath.join('.'))==='set';
  if(!['querySetConstruction','pureSet','setAlgebraQuery'].includes(set.kind)&&!declaredSet&&!ownedSet)return null;
  return {kind:'setMembershipQuery',set,value:expr(n.arguments[0],ctx),equality:'sameValueZero',evaluationOrder:'receiverThenArgument',evaluation:'expressionEvaluation'};
 }
 if(['map','filter','some','every','find','includes','indexOf','concat','slice'].includes(name)) {
  const list=expr(base,{...ctx,allowArenaList:true});
  if(['scopedArenaUnitsQuery','controlledInPlayListQuery','arenaList','arenaFilter','attackQueryContract','arrayFromQuery','arrayAdapterQuery','queryCollectionCall','queryMember','lexicalContextQuery'].includes(list.kind)) {
   const predicate=['map','filter','some','every','find'].includes(name);
   if(predicate&&n.arguments.length!==1||!predicate&&n.arguments.length>2)fail(ctx,n,'query-collection-arity');
   return {kind:'queryCollectionCall',operation:name,receiver:list,args:n.arguments.map(a=>predicate?cb(a,['element','index','collection']):expr(a,ctx)),optionalCall:!!n.questionDotToken,optionalReceiver:!!call.questionDotToken,order:'nativeArrayOrderAndHoles',evaluation:'expressionEvaluation'};
  }
 }
 if(ts.isIdentifier(base)&&base.text==='Contract'&&['assertTrue','assertFalse','assertNotNullLike','assertArraySize','assertHasKey','assertHasProperty','assertEqual','assertNotEqual'].includes(name)) {
  if(!ctx.pureProgram&&!ctx.eventHistory)fail(ctx,n,'query-assertion-program-scope');namespace(base,'Contract');
  const count=['assertArraySize','assertHasKey','assertHasProperty','assertEqual','assertNotEqual'].includes(name)?2:1;
  if(n.arguments.length<count||n.arguments.length>count+1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'query-assertion-arity');
  const values=n.arguments.slice(0,count).map(a=>expr(a,ctx));
  let message={kind:'absent'};if(n.arguments.length>count){const raw=unwrap(n.arguments[count]);if((ts.isArrowFunction(raw)||ts.isFunctionExpression(raw))&&!['assertTrue','assertFalse','assertNotNullLike'].includes(name))fail(ctx,raw,'assertion-message-callback-type');message=ts.isArrowFunction(raw)||ts.isFunctionExpression(raw)?{kind:'lazyAssertionMessage',callback:require('./private-programs').compileCallback(raw,ctx,api,[]),invoke:'onlyOnFailure'}:{kind:'eagerAssertionMessage',value:expr(raw,ctx)};}
  const arg=index=>({argument:index}),length=index=>({argumentLength:index});
  const defaults={assertTrue:['False condition'],assertFalse:['True condition'],assertNotNullLike:['Null-like object value: ',arg(0)],assertArraySize:['Array size ',length(0),' does not match expected size ',arg(1)],assertHasKey:['Map does not contain key ',arg(1)],assertHasProperty:["Object does not have property '",arg(1),"'"],assertEqual:['Value ',arg(0),' is not equal to ',arg(1)],assertNotEqual:['Value ',arg(0),' is equal to ',arg(1)]};
  const precondition=['assertArraySize','assertHasKey','assertHasProperty'].includes(name)?'firstArgumentNotNullLike':null;
  const predicate={assertTrue:'truthy',assertFalse:'falsy',assertNotNullLike:'notNullLike',assertArraySize:'lengthStrictEqualsSecondArgument',assertHasKey:'objectPrototypeHasOwnPropertyCall',assertHasProperty:'secondArgumentInFirstArgument',assertEqual:'strictEqual',assertNotEqual:'strictNotEqual'}[name];
  return {kind:'contractAssertion',assertion:name,arguments:values,message,predicate,...(precondition?{precondition,preconditionFailure:{message:['Null-like object value: ',arg(0)],customMessage:'ignored'}}:{}),evaluationOrder:'argumentsLeftToRightThenAssertion',failure:'throwError',errorPrefix:'Contract assertion failure: ',defaultMessage:defaults[name],messageFallback:name==='assertNotNullLike'?'truthyMessageOtherwiseDefault':'nullishMessageOtherwiseDefault',returnValue:'undefined',engineObjectMutation:false};
 }
 if(ts.isIdentifier(base)&&base.text==='EnumHelpers'&&name==='checkConvertToEnum') {
  args(2,false);namespace(base,'EnumHelpers');const en=unwrap(n.arguments[1]);if(!ts.isIdentifier(en))fail(ctx,en,'query-enum-object');unshadowedEnum(en,ctx,en.text);
  const entries=Object.entries(ctx.enums).filter(([key])=>key.startsWith(en.text+'.')).map(([key,value])=>({key:key.slice(en.text.length+1),value}));if(!entries.length||entries.some(e=>typeof e.value!=='string'))fail(ctx,en,'query-string-enum-required');
  return {kind:'checkedEnumConversion',values:expr(n.arguments[0],ctx),enumObject:en.text,entries,arrayNormalization:'Helpers.asArray',matching:'lowercaseString',duplicateLookup:'lastEnumValueWins',inputOrder:'preservedIncludingDuplicates',invalidValue:'throwError',invalidMessagePrefix:'Invalid value for enum: ',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='Helpers'&&['setUnion','setIntersection'].includes(name)) {
  args(2,false);namespace(base,'Helpers');return {kind:'setAlgebraQuery',operation:name,args:n.arguments.map(a=>expr(a,ctx)),equality:'sameValueZero',order:name==='setUnion'?'firstSetThenNewMembersOfSecond':'firstSetOrderFilteredBySecondMembership',identity:'freshSet',evaluationOrder:'argumentsLeftToRightThenSetIteration'};
 }
 if(name==='getBountyKeywords') {
  args(0,false);return {kind:'bountyKeywordListQuery',card:receiver(base),source:'currentEffectiveKeywordInstances',keyword:'bounty',filter:'isFullyImplemented',order:'effectiveKeywordInstanceOrder',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='Helpers'&&name==='asArray') {
  args(1,false);namespace(base,'Helpers');return {kind:'arrayAdapterQuery',value:expr(n.arguments[0],{...ctx,allowArenaList:true}),arrayInput:'sameObjectReference',nullishInput:'freshEmptyArray',otherInput:'freshSingleElementArray',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='CostHelpers'&&name==='getExploitedUnits') {
  args(1);unshadowedEnum(base,ctx,'CostHelpers');const imported=ctx.source.statements.some(s=>ts.isImportDeclaration(s)&&s.moduleSpecifier.text==='../../../core/cost/CostHelpers'&&s.importClause&&!s.importClause.isTypeOnly&&s.importClause.namedBindings&&ts.isNamespaceImport(s.importClause.namedBindings)&&s.importClause.namedBindings.name.text==='CostHelpers');if(!imported)fail(ctx,n,'query-cost-helper-import');
  return {kind:'exploitedUnitsQuery',playEvent:expr(n.arguments[0],ctx),path:['costs','exploit','selectedTargets'],optionalPath:[true,true],missing:'emptyArray',normalization:'asArrayPreservingArrayIdentity',valueType:'lastKnownInformation',evaluation:'expressionEvaluation'};
 }
 if(['getAllTargets','attackerDealsCombatDamageFirst'].includes(name)) {
  args(0,false);const attack=receiver(base);
  const path=attack.kind==='ref'?attack.path:attack.kind==='dataPath'?[...attack.root,...attack.steps.map(s=>s.key)]:[];
  if(path.at(-1)!=='attack'&&path.at(-1)!=='activeAttack'&&path.at(-1)!=='currentAttack')return null;
  return {kind:'attackQueryContract',operation:name,attack,...(name==='getAllTargets'?{returnIdentity:'nativeTargetsArray'}:{effect:'attackerHasDealsCombatDamageFirst'}),evaluation:'expressionEvaluation'};
 }
 if(name==='flatMap') {
  if(n.arguments.length!==1)fail(ctx,n,'query-flat-map-arity');return {kind:'queryCollectionCall',operation:'flatMap',receiver:expr(base,{...ctx,allowArenaList:true}),callback:cb(n.arguments[0],['element','index','collection']),flattenDepth:1,optionalCall:!!n.questionDotToken,optionalReceiver:!!call.questionDotToken,order:'nativeArrayOrderAndHoles',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='Helpers'&&['countUniqueAspects','countUniqueKeywords'].includes(name)) {
  args(1);namespace(base,'Helpers');return {kind:'distinctCardAttributeCount',attribute:name==='countUniqueAspects'?'aspects':'keywordNames',cards:expr(n.arguments[0],{...ctx,allowArenaList:true}),input:'singleCardOrArray',aggregation:'set',evaluation:'expressionEvaluation'};
 }
 if(ts.isIdentifier(base)&&base.text==='EnumHelpers'&&['arenaName','cardTypeMatches','asRelativePlayer','isArena'].includes(name)) {
  // Existing isArena lowering remains authoritative unless this pending domain
  // needs the query outside the attack/event-specific vocabulary.
  if(name==='isArena')return null;
  if(n.arguments.length!==(['cardTypeMatches','asRelativePlayer'].includes(name)?2:1)||n.questionDotToken||call.questionDotToken)fail(ctx,n,'enum-query-contract-arity');namespace(base,'EnumHelpers');return {kind:'enumQueryContract',operation:name,args:n.arguments.map(a=>expr(a,ctx)),...(name==='arenaName'?{values:{groundArena:'ground arena',spaceArena:'space arena'},otherwise:'undefined'}:name==='cardTypeMatches'?{filter:'singleTypeOrUnion',matching:'pinnedWildcardCardTypeMembership'}:{comparison:'playerIdentityToContextPlayer',values:{same:'self',different:'opponent'}})};
 }
 if(['getPlayers','getOtherPlayer'].includes(name)) {
  args(name==='getPlayers'?0:1);const game=receiver(base);
  const gamePath=valuePath(game);
  if(!(gamePath.join('.')==='game'||gamePath.join('.')==='source.game'||(ctx.typedCallbackRoles?.has(gamePath[0])||ctx.dataRoots?.has(gamePath[0]))&&gamePath.at(-1)==='game'))return null;
  return name==='getPlayers'?{kind:'gamePlayersQuery',game,order:'playersAndSpectatorsObjectValues',filter:'isPlayer',evaluation:'expressionEvaluation'}:{kind:'otherPlayerQuery',game,player:expr(n.arguments[0],ctx),selection:'firstPlayerWithDifferentName',missing:'undefined',evaluation:'expressionEvaluation'};
 }
 if(['some','every','map','filter','find','reduce'].includes(name)&&ts.isCallExpression(base)&&ts.isPropertyAccessExpression(base.expression)&&base.expression.name.text==='getPlayers') {
  stage();if(n.questionDotToken||call.questionDotToken||n.arguments.length!==(name==='reduce'?2:1))fail(ctx,n,'game-player-collection-arity');
  return {kind:'gamePlayerCollection',operation:name,receiver:expr(base,ctx),callback:cb(n.arguments[0],name==='reduce'?['accumulator','player','index','collection']:['player','index','collection']),...(name==='reduce'?{initial:expr(n.arguments[1],ctx)}:{}),order:'receiverArrayOrder',evaluation:'expressionEvaluation'};
 }
 if(['getPrintedPower','getPrintedHp','isAdvantage','isCard','isTokenUnit','isLeaderUnit','canBeExhausted','isResource'].includes(name)) {
  args(0,false);const value=receiver(base);return {kind:'cardQueryContract',operation:name,receiver:value,evaluation:'expressionEvaluation',...(n.flags&ts.NodeFlags.OptionalChain?{continuesOptionalChain:true}:{}),...(name.startsWith('getPrinted')?{printedAttributesOverride:true}:{} )};
 }
 if(name==='getInPlayCards') {
  stage();if(n.arguments.length>1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'in-play-query-contract-arity');
  const player=receiver(base),p=valuePath(player);if(!(['player','opponent','source.controller'].includes(p.join('.'))||ctx.typedCallbackRoots?.has(p[0])&&p.at(-1)==='player'))return null;
  const o=n.arguments.length?object(n.arguments[0],ctx):{props:new Map(),ctx},filter={};for(const[key,v]of o.props){if(!['aspect','keyword','trait','type','otherThan','condition','controller'].includes(key))fail(o.ctx,v,'in-play-query-contract-filter:'+key);filter[key]=key==='condition'?cb(v,['candidate']):expr(v,o.ctx);}
  return {kind:'controlledInPlayListQuery',player,filter,arenaController:'queriedPlayerOverridesFilter',baseZoneController:'zoneOwnerNoAdditionalControllerFilter',order:['controlledGroundArenaCards','controlledSpaceArenaCards','baseZoneCards'],baseZoneOrder:['base','forceToken','leaders','credits','baseUpgrades'],evaluation:'expressionEvaluation'};
 }
 if(name==='controlsCardWithTrait') {
  stage();if(n.arguments.length<1||n.arguments.length>3||n.questionDotToken||call.questionDotToken)fail(ctx,n,'controlled-trait-query-contract-arity');
  return {kind:'controlledInPlayTraitSomeQuery',receiver:receiver(base),trait:expr(n.arguments[0],ctx),onlyUnique:n.arguments[1]?expr(n.arguments[1],ctx):{kind:'const',value:false},otherThan:n.arguments[2]?expr(n.arguments[2],ctx):{kind:'undefined'},domain:'allControlledInPlayCardsIncludingBaseZone',evaluation:'expressionEvaluation'};
 }
 if(name==='hasSomeCard') {
  args(1);const zone=receiver(base);if(zone.kind!=='ref'||!['discardZone','handZone','deckZone','resourceZone','baseZone','allArenas'].includes(zone.path.at(-1)))fail(ctx,n,'zone-query-contract-receiver');
  const o=object(n.arguments[0],ctx),filter={};for(const [key,v]of o.props){if(!['aspect','keyword','trait','type','otherThan','condition'].includes(key))fail(o.ctx,v,'zone-query-contract-filter:'+key);filter[key]=key==='condition'?cb(v,['candidate']):expr(v,o.ctx);}
  return {kind:'zoneSomeQuery',zone,filter,fieldCombination:'and',attributeArrayCombination:'or',condition:'shortCircuitAfterAttributeFilters',evaluation:'expressionEvaluation'};
 }
 // A computed player or explicitly typed callback player cannot use the legacy
 // single player/card register. Keep its physical receiver and predicate role.
 if(['getArenaUnits','hasSomeArenaUnit','hasSomeArenaCard','hasSomeArenaUpgrade'].includes(name)) {
  const value=receiver(base),p=valuePath(value);
  // Typed roots are only arena controllers when they name a player seat — not
  // source/target/card identities that happen to live in the callback scope.
  const typed=(ctx.typedCallbackRoles?.has(p[0])||ctx.dataRoots?.has(p[0]))&&['player','opponent'].includes(p.at(-1));
  const computed=value.kind==='otherPlayerQuery';
  const candidateController=['card','selectionCandidate','attachTarget'].includes(p[0])&&p.at(-1)==='controller';
  const ordinary=value.kind==='ref'&&['player','opponent','game','source.controller','source.controller.opponent'].includes(p.join('.'));
  if(!typed&&!computed&&!candidateController&&!ordinary)return null;
  stage();if(n.arguments.length>1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'arena-query-contract-arity');
  const o=n.arguments.length?object(n.arguments[0],ctx):{props:new Map(),ctx},filter={};
  for(const[key,v]of o.props){if(!['arena','trait','aspect','keyword','otherThan','condition',...(name==='hasSomeArenaCard'?['type']:[])].includes(key))fail(o.ctx,v,'arena-query-contract-filter:'+key);filter[key]=key==='condition'?cb(v,['candidate']):expr(v,o.ctx);}
  return {kind:name==='getArenaUnits'?'scopedArenaUnitsQuery':'scopedArenaSomeQuery',receiver:value,filter,type:name==='hasSomeArenaCard'?'any':name==='hasSomeArenaUpgrade'?'upgrade':'unit',controller:p.at(-1)==='game'?'any':'receiverIdentity',order:'groundThenSpacePhysicalOrder',fieldCombination:'and',attributeArrayCombination:'or',predicateOrder:['aspect','keyword','trait','type','otherThan','condition'],evaluation:'expressionEvaluation'};
 }
 if(['isTraitInPlay','isAspectInPlay','isKeywordInPlay'].includes(name)) {
  stage();if(n.arguments.length<1||n.arguments.length>2||n.questionDotToken||call.questionDotToken)fail(ctx,n,'arena-attribute-query-contract-arity');
  const owner=receiver(base),game=valuePath(owner).at(-1)==='game';if(game&&(name!=='isTraitInPlay'||n.arguments.length!==1))fail(ctx,n,'game-attribute-query-contract-signature');
  return {kind:'arenaAttributeSomeQuery',receiver:owner,attribute:{isTraitInPlay:'trait',isAspectInPlay:'aspect',isKeywordInPlay:'keyword'}[name],value:expr(n.arguments[0],ctx),otherThan:n.arguments[1]?expr(n.arguments[1],ctx):{kind:'const',value:null},type:'unit',controller:game?'any':'receiverIdentity',...(game?{iteration:'gamePlayersThenEachControlledArena'}:{}),attributeArrayCombination:'or',evaluation:'expressionEvaluation'};
 }
 return null;
}
function queryCollectionContract(n,ctx,api){
 if(ctx.liveStats||ctx.legacyKeywordValue||ctx.constructionFactorySuppressed&&!ctx.pendingSystemConstruction)return null;
 const list=api.expr(n.expression.expression,{...ctx,allowArenaList:true});
 // Existing scoped collections retain their established representation. New
 // Array adapters and previously rejected arena algebra need separate slots.
 if(!['arrayFromQuery','arrayAdapterQuery'].includes(list.kind))return null;
 return queryContract(n,ctx,api);
}
module.exports={queryContract,queryCollectionContract};
