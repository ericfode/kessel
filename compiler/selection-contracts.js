'use strict';

function applyWaitingPromptTitle(result,obj,api){
 if(!obj.props.has('waitingPromptTitle'))return;
 const {ts,unwrap,literal,contextTitle,fail}=api,input=obj.props.get('waitingPromptTitle'),ctx=obj.ctx;
 try{result.waitingPromptTitle=literal(input,ctx);return;}catch(error){if(!error.diagnostic)throw error;}
 const root=unwrap(input);if(ts.isArrowFunction(root)||ts.isFunctionExpression(root))fail(ctx,root,'waiting-title-must-be-constructed-string');
 const visit=node=>{
  const n=unwrap(node);if(ctx.constructionValues?.has(n))return;
  if(ts.isIdentifier(n)&&ctx.constants.has(n.text)&&ctx.constantStages?.get(ctx.constants.get(n.text))){
   const p=n.parent;if(!(ts.isPropertyAccessExpression(p)&&p.name===n))fail(ctx,n,'waiting-title-saved-binding-requires-program');
  }
  ts.forEachChild(n,visit);
 };visit(root);
 result.waitingPromptTitleValue={kind:'constructedPromptText',parts:contextTitle(input,ctx),evaluateAt:'resolverPropertiesConstruction',result:'string'};
}
function numberResolver(obj,api){
 const {literal,expr,contextTitle,fail}=api,{props,node}=obj,ctx={...obj.ctx,pendingSelection:true,pendingChoice:true,pendingSelectionContract:true};
 if(!props.has('mode')||literal(props.get('mode'),ctx)!=='chooseNumber')return null;
 api.knownProperties(obj,['mode','min','max','condition','logSelection','choosingPlayer','immediateEffect','dependsOn','mustChangeGameState','hideIfNoLegalTargets','appendToDefaultTitle']);
 if(!props.has('min')||!props.has('max'))fail(ctx,node,'number-resolver-range-required');
 const result={kind:'numberSelection',mode:'chooseNumber',min:expr(props.get('min'),ctx),max:expr(props.get('max'),ctx),chooser:props.has('choosingPlayer')?expr(props.get('choosingPlayer'),ctx):{kind:'const',value:'self'},resultBinding:'select',resolverName:'target',namedResultStorage:'selects[name].choice',contextSelectAssignment:'onlyWhenNameIsTarget',resultRepresentation:'decimalString',rangeAssertions:['integerMinimum','integerMaximum','maximumAtLeastMinimum'],sameEndpoints:'automaticStringSelection',logSelection:props.has('logSelection')?literal(props.get('logSelection'),ctx):true,defaults:'NumberTargetResolver'};
 for(const field of ['dependsOn','mustChangeGameState','hideIfNoLegalTargets','appendToDefaultTitle'])if(props.has(field))result[field]=literal(props.get(field),ctx);
 if(props.has('condition'))result.condition=expr(props.get('condition'),ctx);
 if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),ctx);
 applyWaitingPromptTitle(result,{...obj,ctx},api);
 return result;
}
function selectionContractExpression(input,ctx,api){
 const {ts,unwrap,expr,fail,access,unshadowedEnum}=api,n=unwrap(input);
 if(ts.isCallExpression(n)&&ts.isIdentifier(n.expression)&&['parseInt','Number'].includes(n.expression.text)){
  const name=n.expression.text;if(n.questionDotToken||n.arguments.length<1||n.arguments.length>(name==='parseInt'?2:1))fail(ctx,n,'selection-number-coercion-arity');
  unshadowedEnum(n,ctx,name);
  for(const s of ctx.source.statements)if(ts.isImportDeclaration(s)&&s.importClause&&(s.importClause.name?.text===name||s.importClause.namedBindings?.name?.text===name||s.importClause.namedBindings?.elements?.some(e=>e.name.text===name)))fail(ctx,n,'selection-number-coercion-shadow');
  return {kind:'selectionNumberCoercion',operation:name,args:n.arguments.map(a=>expr(a,ctx)),semantics:name==='parseInt'?'javascriptParseInt':'javascriptNumber',invalidResult:'NaN'};
 }
 const chain=input=>{
  const node=unwrap(input);
  if(ts.isIdentifier(node))return {root:node.text,rootNode:node,path:[],optional:[]};
  if(ts.isPropertyAccessExpression(node)||ts.isElementAccessExpression(node)){
   const p=chain(node.expression);if(!p)return null;
   let key;if(ts.isPropertyAccessExpression(node))key=node.name.text;else{const x=unwrap(node.argumentExpression);if(ts.isStringLiteral(x))key=x.text;else if(ts.isNumericLiteral(x))key=Number(x.text);else return null;}
   const i=p.path.length;p.path.push(key);if(node.questionDotToken)p.optional.push(i);return p;
  }
  return null;
 };
 const p=chain(n);
 if(p){
  const binding=ctx.scope.get(p.root);
  if(ctx.selectionContextArgument&&!p.path.length&&binding?.kind==='ref'&&binding.path.join('.')==='context')return {kind:'selectionContextReference',contextBinding:require('./event-result-context').contextBinding(p.rootNode,ctx,api)};
  if(binding?.kind==='ref'&&binding.path.join('.')==='context'&&['controllingPlayer','selectedPromptCards'].includes(p.path[0]))return {kind:'selectionContextRead',contextBinding:require('./event-result-context').contextBinding(p.rootNode,ctx,api),path:p.path,optionalAccess:p.optional,semantics:p.path[0]==='controllingPlayer'?'attachmentController':'physicalPromptSelection'};
  if(ctx.extendedNamedSelection&&binding?.kind==='ref'){
   let path=[...binding.path,...p.path];if(path[0]==='context')path=path.slice(1);
   if(path[0]==='targets'){
    const slot=path[1];if(!ctx.namedTargets?.has(slot))fail(ctx,n,'selection-unknown-named-slot');
    return {kind:'namedSelectionRead',contextBinding:require('./event-result-context').contextBinding(p.rootNode,ctx,api),slot,path:path.slice(2),optionalAccess:p.optional,identity:'physicalValueOrSelectedList'};
   }
   if(path[0]==='namedTargets')return {kind:'namedSelectionRead',slot:path[1],path:path.slice(2),optionalAccess:p.optional,identity:'physicalValueOrSelectedList'};
   if(!p.path.length&&binding.path.join('.')==='context')return {kind:'selectionContextReference',contextBinding:require('./event-result-context').contextBinding(p.rootNode,ctx,api)};
  }
 }
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)){
  const method=n.expression.name.text,receiverPath=chain(n.expression.expression);
  const special=receiverPath&&ctx.scope.get(receiverPath.root)?.path?.join('.')==='context'&&['controllingPlayer','selectedPromptCards'].includes(receiverPath.path[0]);
  if((ctx.extendedNamedSelection||special||!ctx.namedTargets?.size&&['canAttach','canAttachPilot'].includes(method))&&['canAttach','canAttachPilot','hasCost','includes','isTokenUnit','isAttacking','isDefending','isUpgrade','isUnit','getPower','getHp','hasSomeTrait'].includes(method))return {kind:'selectionMethodCall',method,receiver:expr(n.expression.expression,ctx),args:n.arguments.map(a=>expr(a,{...ctx,selectionContextArgument:['canAttach','canAttachPilot'].includes(method)})),optionalReceiver:!!n.expression.questionDotToken,optionalCall:!!n.questionDotToken};
 }
 return null;
}
function selectionSystem(node,ctx,defaultTarget,api){
 const {access,object,literal,expr,effect,fail,ts}=api,p=access(node.expression,ctx),kind=p.at(-1);
 if(p.length!==3||p[1]!=='immediateEffects'||!['selectPlayer','chooseNumber','selectCard'].includes(kind))return null;
 if(node.arguments.length!==1)fail(ctx,node,'selection-system-properties');
 const obj=object(node.arguments[0],{...ctx,pendingSelection:true,pendingChoice:true,pendingSelectionContract:true});
 if(kind==='selectCard'){
  if(!['name','player','isCost','cancelIfNoTargets'].some(key=>obj.props.has(key)))return null;
  api.knownProperties(obj,['name','target','player','isCost','cancelIfNoTargets','mode','cardTypeFilter','controller','zoneFilter','cardCondition','optional','numCards','canChooseNoCards','immediateEffect','choosingPlayer',...api.EXTRA_FIELDS]);
  if(!obj.props.has('immediateEffect'))fail(ctx,node,'named-select-card-inner-required');
  const name=obj.props.has('name')?literal(obj.props.get('name'),obj.ctx):obj.props.has('isCost')&&literal(obj.props.get('isCost'),obj.ctx)?'cost':'target';if(typeof name!=='string'||!/^[_A-Za-z][_A-Za-z0-9]{0,63}$/.test(name))fail(ctx,node,'named-select-card-name');
  const mode=obj.props.has('mode')?literal(obj.props.get('mode'),obj.ctx):'single',slots=new Map(obj.ctx.namedTargets||[]);slots.set(name,{array:mode!=='single'});
  const child={...obj.ctx,namedTargets:slots,extendedNamedSelection:true},props=new Map(obj.props);
  for(const key of ['name','target','player','isCost','cancelIfNoTargets'])props.delete(key);
  if(!props.has('choosingPlayer')&&obj.props.has('player'))props.set('choosingPlayer',obj.props.get('player'));
  const resolver=api.richResolver({...obj,props,ctx:child},api,true);resolver.resultBinding=name;resolver.resolverName=name;if(resolver.kind==='choiceSelection'){resolver.namedResultBinding='selects.'+name+'.choice';resolver.contextSelectAssignment='onlyWhenNameIsTarget';}
  const properties={};for(const key of ['target','player','isCost','cancelIfNoTargets'])if(obj.props.has(key))properties[key]=expr(obj.props.get(key),obj.ctx);
  return {kind:'namedSelectCardSystem',name,resolver,properties,innerSystem:effect(obj.props.get('immediateEffect'),child,'chosen'),innerSystemDefaultTarget:{kind:'ref',path:['namedTargets',name]},contextTargetAssignment:'onlyWhenNameIsTarget',contract:'SelectCardSystem',legality:'CardTargetResolverWithCandidateContextCopy',defaults:{optional:false,cardCondition:true,name:'targetOrCostWhenIsCost',waitingPromptTitle:'Waiting for opponent',costSelection:'mandatoryMustFullyResolve',innerOptional:'makesSelectionOptional'},emptySelection:'skipUnlessSelectorAllowsZero',cancelIfNoTargetsDefault:false};
 }
 api.knownProperties(obj,['mode','min','max','name','target','choosingPlayer','immediateEffect','logSelection','effectChoices','mustChangeGameState','hideIfNoLegalTargets','appendToDefaultTitle']);
 if(!obj.props.has('immediateEffect'))fail(ctx,node,'selection-system-inner-required');
 const props=new Map(obj.props);props.set('mode',ts.factory.createStringLiteral(kind==='chooseNumber'?'chooseNumber':props.has('mode')?literal(props.get('mode'),obj.ctx):'player'));props.delete('name');props.delete('target');
 const resolver=kind==='chooseNumber'?numberResolver({...obj,props},api):api.richResolver({...obj,props},api,true);
 const name=obj.props.has('name')?literal(obj.props.get('name'),obj.ctx):'target';if(typeof name!=='string'||!name.length||name.length>64)fail(ctx,node,'selection-system-name');resolver.resultBinding=name;resolver.resolverName=name;if(resolver.kind==='choiceSelection'){resolver.namedResultBinding='selects.'+name+'.choice';resolver.contextSelectAssignment='onlyWhenNameIsTarget';}
 const child={...obj.ctx,...(kind==='selectPlayer'?{selectedPlayerTarget:{kind:'ref',path:['selectedPlayer']}}:{})};
 return {kind:'selectionSystem',system:kind,name,resolver,...(obj.props.has('target')?{target:expr(obj.props.get('target'),obj.ctx)}:{}),innerSystem:effect(obj.props.get('immediateEffect'),child,defaultTarget),contract:kind==='chooseNumber'?'ChooseNumberSystem.candidateContextCopiesThenResolve':'SelectPlayerSystem.selectedPlayerDefaultTarget',legality:kind==='chooseNumber'?'anyIntegerCandidateHasLegalInnerSystem':'innerSystemCanAffectPlayer'};
}
function extendedNamedResolvers(input,context,api){
 const {object,literal,expr,effect,fail}=api,obj=object(input,context),slots=new Map(),entries=[],dependents=new Set();
 if(context.namedTargets?.size)fail(context,input,'nested-extended-named-resolvers');
 if(!obj.props.size||obj.props.size>16)fail(context,input,'extended-named-target-bound');
 const modes=new Set(['single','exactly','upTo','unlimited','exactlyVariable','upToVariable','maxStat','betweenVariable','player','multiplePlayers','select','selectUnless','dropdownList','chooseNumber']);
 for(const[name,value]of obj.props){
  if(!/^[A-Za-z_][A-Za-z0-9_]{0,63}$/.test(name)||['__proto__','prototype','constructor'].includes(name))fail(context,value,'extended-named-target-name');
  const raw=object(value,obj.ctx),mode=raw.props.has('mode')?literal(raw.props.get('mode'),raw.ctx):'single';if(!modes.has(mode))fail(raw.ctx,value,'extended-named-target-mode');
  api.knownProperties(raw,['mode','cardTypeFilter','controller','zoneFilter','cardCondition','optional','numCards','canChooseNoCards','immediateEffect','choosingPlayer','dependsOn',...api.EXTRA_FIELDS,'choices','options','unlessEffect','defaultEffect','checkTarget','showUnresolvable','highlightCards','logSelection','min','max']);
  let dependsOn;if(raw.props.has('dependsOn')){dependsOn=literal(raw.props.get('dependsOn'),raw.ctx);if(!slots.has(dependsOn)||dependents.has(dependsOn))fail(raw.ctx,value,'extended-named-target-dependency');dependents.add(dependsOn);}
  const selection=['select','selectUnless','dropdownList','chooseNumber'].includes(mode),array=!['single','player'].includes(mode)&&!selection,scope=new Map(slots);scope.set(name,{array,selection});
  const ctx={...raw.ctx,pendingSelection:true,pendingChoice:true,extendedNamedSelection:true,pendingSelectionContract:true,namedTargets:scope},props=new Map(raw.props);props.delete('dependsOn');
  if(!selection)api.knownProperties({...raw,props,ctx},['mode','cardTypeFilter','controller','zoneFilter','cardCondition','optional','numCards','canChooseNoCards','immediateEffect','choosingPlayer',...api.EXTRA_FIELDS]);
  let resolver=numberResolver({...raw,props,ctx},api)||api.choiceResolver({...raw,props,ctx},api)||api.richResolver({...raw,props,ctx},api,true);
  if(!resolver)fail(ctx,value,'extended-named-target-resolver');resolver.resultBinding=name;resolver.resolverName=name;if(resolver.kind==='choiceSelection'){resolver.namedResultBinding='selects.'+name+'.choice';resolver.contextSelectAssignment='onlyWhenNameIsTarget';}
  const entry={name,targetResolver:resolver,chooser:resolver.chooser,...(dependsOn?{dependsOn}:{})};
  if(raw.props.has('immediateEffect'))entry.effect=effect(raw.props.get('immediateEffect'),{...ctx,...(resolver.kind==='playerSelection'?{selectedPlayerTarget:{kind:'ref',path:['namedTargets',name]}}:{})},'chosen');
  else if(selection&&!['dropdownList','chooseNumber'].includes(mode))entry.effect={kind:'resolveChoice',resultBinding:name};
  entries.push(entry);slots.set(name,{array,selection});
 }
 return {entries,ctx:{...obj.ctx,pendingSelection:true,pendingChoice:true,extendedNamedSelection:true,pendingSelectionContract:true,namedTargets:slots},contract:'PlayerOrCardAbility.namedTargetResolvers'};
}
function choiceFactoryMap(input,context,api){
 const {ts,unwrap,effect,contextTitle,fail}=api;
 const frame=require('./system-array-factory').constructionFrame(input,{...context,pendingChoice:true,pendingSelection:true},['choiceFactoryContext'],'choiceFactoryInvocation',api),node=frame.node;
 if(!ts.isObjectLiteralExpression(node)||!node.properties.length||node.properties.length>32)fail(frame.ctx,node,'choice-factory-map-object');
 const captures=frame.captures.slice(),entries=[];
 for(const property of node.properties){
  if(!ts.isPropertyAssignment(property))fail(frame.ctx,property,'choice-factory-property');
  const label=ts.isComputedPropertyName(property.name)?property.name.expression:ts.isIdentifier(property.name)?ts.factory.createStringLiteral(property.name.text):property.name;
  const labelParts=contextTitle(label,frame.ctx),start=frame.captures.length;
  frame.walk(property.initializer);
  entries.push({labelParts,keyEvaluation:'beforeValueConstruction',constructionCaptures:frame.captures.slice(start),effect:effect(property.initializer,frame.ctx,'self')});
 }
 return {kind:'choiceFactoryMap',parameters:frame.parameters,captures,entries,constructionOrder:'readonlyDeclarationsThenEachKeyAndValueLeftToRight',contextBinding:'resolverContextByReference',captureIdentity:'valueOrPhysicalObjectReference',invocationPolicy:'eachNativeChoiceGeneration',keyOrder:'javascriptPropertyOrder',keyCoercion:'javascriptPropertyKey',duplicatePolicy:'lastAssignmentWins',callbacks:'deferredWithLexicalBindings',factory:true};
}
function attachmentSelectionSystem(node,context,defaultTarget,api){
 const {access,object,expr,fail}=api,p=access(node.expression,context);
 if(p.length!==3||p[1]!=='immediateEffects'||p[2]!=='attachUpgrade'||node.arguments.length!==1)return null;
 const obj=object(node.arguments[0],context);if(!obj.props.has('newController')&&(context.pendingSelection||context.pendingRegistration))return null;
 api.knownProperties(obj,['target','upgrade','newController']);
 if(!obj.props.has('upgrade'))fail(obj.ctx,obj.node,'attachment-upgrade-required');
 return {kind:obj.props.has('newController')?'attachUpgradeWithController':'attachUpgradeSystem',upgrade:expr(obj.props.get('upgrade'),obj.ctx),target:obj.props.has('target')?expr(obj.props.get('target'),obj.ctx):defaultTarget,newController:obj.props.has('newController')?expr(obj.props.get('newController'),obj.ctx):{kind:'nativeUpgradeControllerDefault'},controllerResolution:'outOfPlayNonTokenAlwaysContextPlayerOtherwiseRelativeChoiceOrUpgradeController',contract:'AttachUpgradeSystem.getFinalController',event:'onUpgradeAttached',attachmentIdentity:'physicalUpgradeAndHost',eventRechecksCanAttach:true,cleanup:'InPlayCard.attachToAndUnitCleanupContingentEvents'};
}
module.exports={applyWaitingPromptTitle,attachmentSelectionSystem,choiceFactoryMap,numberResolver,selectionContractExpression,selectionSystem,extendedNamedResolvers};
