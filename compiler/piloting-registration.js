'use strict';

// Source-conversion IR only. These wrappers describe registration/host rebinding
// and deliberately require a future executor, rather than installing unit bodies
// on the upgrade or treating Piloting as an ordinary When Played ability.
const replacementRegistration=require('./replacement-registration');
const eventHistory=require('./event-history');
const ATTACHED_TRIGGERS = {
 addGainOnAttackAbilityTargetingAttached:'onAttack',
 addGainOnDefenseAbilityTargetingAttached:'onDefense',
 addGainWhenDefeatedAbilityTargetingAttached:'whenDefeated',
 addGainWhenAttackEndsAbilityTargetingAttached:'whenAttackEnds'
};
const REGISTRATIONS = new Set([
 ...Object.keys(ATTACHED_TRIGGERS),'addGainTriggeredAbilityTargetingAttached',
 'addGainActionAbilityTargetingAttached','addPilotingAbility',
 'addPilotingGainAbilityTargetingAttached','addPilotingGainTriggeredAbilityTargetingAttached',
 'addPilotingConstantAbilityTargetingAttached','addPilotingGainKeywordTargetingAttached',
 'addPilotDeploy'
]);
function registrationHooks(input,ctx,api) {
 const {object,unwrap,ts,expr,literal,fail}=api,obj=object(input,ctx),hooks=[];
 if(!obj.props.size||obj.props.size>16)fail(ctx,input,'attached-trigger-hook-bound');
 const aliases=new Set(['whenPlayed','whenDefeated','onAttack','onDefense','whenPlayedUsingSmuggle']);
 const events=new Set(['onCardReadied','onPhaseStarted','onCardDefeated','onCardPlayed','onUpgradeAttached','onUpgradeUnattached','onDamageDealt','onLeaderDeployed','onAttackEnd','onAttackDeclared','onCardMoved','onAttackCompleted']);
 for(const [event,value]of obj.props){
  const n=unwrap(value);
  if(aliases.has(event)){if(literal(n,obj.ctx)!==true)fail(ctx,n,'attached-trigger-alias-true');hooks.push({alias:event,enabled:true});}
  else {if(!events.has(event)||!(ts.isArrowFunction(n)||ts.isFunctionExpression(n)))fail(ctx,n,'attached-trigger-hook:'+event);hooks.push({event,predicate:expr(value,{...obj.ctx,pendingRegistration:true},['event','context'])});}
 }
 return hooks;
}
function registrationLimit(input,ctx,api) {
 const {unwrap,ts,access,literal,fail}=api,n=unwrap(input);
 if(!ts.isCallExpression(n)||n.questionDotToken||n.arguments.length>1)fail(ctx,n,'attached-ability-limit');
 const p=access(n.expression,ctx);if(p.length!==3||!['AbilityHelper','abilityHelper'].includes(p[0])||p[1]!=='limit'||!['perRound','perPhase','perGame','perPlayerPerGame'].includes(p[2]))fail(ctx,n,'attached-ability-limit');
 const amount=n.arguments.length?literal(n.arguments[0],ctx):1;if(!Number.isSafeInteger(amount)||amount<1||amount>64)fail(ctx,n,'attached-ability-limit-bound');
 return {kind:p[2],amount};
}
function compileGrantedAbility(input,context,api,options={}) {
 const {object,literal,ability,constantAbility,fail,displayText,effect}=api;
 const obj=object(input,{...context,pendingRegistration:true}),ctx=obj.ctx,props=new Map(obj.props);
 const type=props.has('type')?literal(props.get('type'),ctx):options.type;
 if(!['triggered','constant','action','replacementEffect','damageModification'].includes(type))fail(ctx,obj.node,'attached-ability-type:'+type);
 props.delete('type');
 const extra={};
 if(props.has('gainCondition')){extra.gainCondition=api.expr(props.get('gainCondition'),ctx);props.delete('gainCondition');extra.gainConditionSource='grantor';extra.gainConditionRequiresGrantorInPlay=true;}
 for(const key of ['zoneFilter','sourceZoneFilter'])if(props.has(key)){extra[key]=literal(props.get(key),ctx);props.delete(key);}
 if(props.has('limit')){extra.limit=type==='damageModification'?eventHistory.activationLimit(props.get('limit'),ctx,api):registrationLimit(props.get('limit'),ctx,api);props.delete('limit');}
 for(const key of ['collectiveTrigger','triggerOnce','requiresConfirmation','cannotTargetFirst'])if(props.has(key)){const v=literal(props.get(key),ctx);if(typeof v!=='boolean')fail(ctx,obj.node,'attached-ability-boolean:'+key);extra[key]=v;props.delete(key);}
 let hooks;
 if(options.trigger){if(props.has('when'))fail(ctx,obj.node,'attached-standard-trigger-extra-when');hooks=options.trigger==='whenAttackEnds'?[{event:'onAttackEnd',predicate:{kind:'binary',op:'===',left:{kind:'ref',path:['event','attack','attacker']},right:{kind:'ref',path:['source']}}}]:[{alias:options.trigger,enabled:true}];}
 else if(props.has('when')){hooks=registrationHooks(props.get('when'),ctx,api);props.delete('when');}
 if(type==='triggered'&&!hooks)fail(ctx,obj.node,'attached-trigger-without-when');
 let program;
 if(type==='damageModification')program=replacementRegistration.registration('addDamageModificationAbility',input,ctx,{...api,eventHistory},{...obj,props,ctx:{...ctx,eventHistory:true}});
 else if(type==='constant')program=constantAbility(input,ctx,{...obj,props,ctx});
 else if(type==='replacementEffect'){
  if(!hooks)fail(ctx,obj.node,'attached-replacement-without-when');
  const allowed=['title','optional','condition','replaceWith'];api.knownProperties({...obj,props},allowed);
  program={trigger:'replacement',title:props.has('title')?displayText(props.get('title'),ctx):'',costs:[]};
  if(props.has('optional'))program.optional=literal(props.get('optional'),ctx);
  if(props.has('condition'))program.condition=api.expr(props.get('condition'),ctx);
  if(props.has('replaceWith')){const replace=object(props.get('replaceWith'),ctx);api.knownProperties(replace,['replacementImmediateEffect']);if(!replace.props.has('replacementImmediateEffect'))fail(ctx,replace.node,'attached-replacement-effect');program.replacement=effect(replace.props.get('replacementImmediateEffect'),replace.ctx,'self');}
  else program.replacement={kind:'cancelMatchedEvent'};
 } else program=ability(input,ctx,options.trigger||(type==='action'?'action':'eventHooks'),{...obj,props,ctx});
 if(hooks)program.hooks=hooks;
 return {type,program,...extra};
}
function pilotingRegistration(name,call,context,details,api) {
 if(!REGISTRATIONS.has(name))return null;
 const {fail,object,literal,expr,modifierEffects,displayText}=api,{baseClass,side}=details;
 const pilot=name.startsWith('addPilot');
 if(name==='addPilotDeploy'){
  if(baseClass!=='LeaderUnitCard'||side!=='leader'||call.arguments.length)fail(context,call,'pilot-deploy-source-or-arity');
  return {trigger:'action',title:'Deploy [value] as a Pilot',contextTitle:[{kind:'const',value:'Deploy '},{kind:'ref',path:['source','title']},{kind:'const',value:' as a Pilot'}],costs:[],effect:{kind:'pilotDeploy',source:'self',sourceZone:'base',requiresConfirmation:true,limit:{kind:'sharedLeaderDeployEpicAction'},resourceCondition:{kind:'binary',op:'>=',left:{kind:'ref',path:['player','resources','length']},right:{kind:'ref',path:['source','cost']}},targetResolver:{owner:'self',type:'unit',zone:'anyAttackable',traits:['vehicle'],requiresCanAttachPilot:true},deployType:'leaderUpgrade',makeHostLeader:true,requiresPrintedUpgradeStats:true}};
 }
 if(call.arguments.length!==1)fail(context,call,'attached-registration-arity');
 if(pilot?!['LeaderUnitCard','NonLeaderUnitCard'].includes(baseClass)||!['card','unit'].includes(side):baseClass!=='UpgradeCard'||side!=='card')fail(context,call,'attached-registration-source-side');
 const input=call.arguments[0],ctx={...context,pendingRegistration:true};
 if(name==='addPilotingGainKeywordTargetingAttached'){
  const obj=object(input,ctx);api.knownProperties(obj,['keyword','amount','gainCondition']);
  const properties=new Map(obj.props);properties.delete('gainCondition');
  const synthetic=api.ts.factory.createCallExpression(api.ts.factory.createPropertyAccessExpression(api.ts.factory.createPropertyAccessExpression(api.ts.factory.createIdentifier('AbilityHelper'),'ongoingEffects'),'gainKeyword'),undefined,[api.ts.factory.createObjectLiteralExpression([...properties].map(([k,v])=>api.ts.factory.createPropertyAssignment(k,v)))]);
  const effect=modifierEffects(synthetic,ctx,'persistent','parent');
  const result={trigger:'constant',title:'Give keyword to attached card',costs:[],effect:{kind:'pilotingAttachedKeyword',target:'parent',modifier:effect,requiresGrantorInPlay:true,targetController:'any'}};
  if(obj.props.has('gainCondition'))result.effect.gainCondition=expr(obj.props.get('gainCondition'),obj.ctx);
  return result;
 }
 if(name==='addPilotingConstantAbilityTargetingAttached'){
  const obj=object(input,ctx);api.knownProperties(obj,['title','condition','ongoingEffect']);if(!obj.props.has('ongoingEffect'))fail(ctx,call,'piloting-attached-without-effect');
  return {trigger:'constant',title:obj.props.has('title')?displayText(obj.props.get('title'),obj.ctx):'',costs:[],effect:{kind:'pilotingAttachedModifier',target:'parent',targetController:'any',requiresGrantorInPlay:true,...(obj.props.has('condition')?{condition:expr(obj.props.get('condition'),obj.ctx)}:{}),modifier:modifierEffects(obj.props.get('ongoingEffect'),obj.ctx,'persistent','parent')}};
 }
 const trigger=ATTACHED_TRIGGERS[name],type=trigger||name==='addGainTriggeredAbilityTargetingAttached'||name==='addPilotingGainTriggeredAbilityTargetingAttached'?'triggered':name==='addGainActionAbilityTargetingAttached'?'action':null;
 const registration=compileGrantedAbility(input,ctx,api,{type,trigger});
 return {trigger:'constant',title:registration.program.title,costs:[],effect:{kind:name==='addPilotingAbility'?'pilotingRegistration':'attachedAbilityRegistration',registrationMode:pilot?'piloting':'upgrade',target:name==='addPilotingAbility'?'self':'parent',targetController:'any',sourceBinding:name==='addPilotingAbility'?'grantor':'attachedHost',controllerBinding:name==='addPilotingAbility'?'grantorController':'hostController',requiresGrantorInPlay:true,requiresAttachable:pilot,sourceMode:pilot?'unitUpgrade':'upgrade',targetZone:pilot?'anyArena':'anyAttackable',targetTypes:pilot?['unit']:['unit','base'],registration}};
}
// Pure calls unavailable to the current engine retain their identity and optional
// call boundaries in typed pending IR. Only the scoped registration grammar uses
// these nodes; existing accepted expressions stay unchanged.
function registrationExpression(node,ctx,api){
 if(!ctx.pendingRegistration)return null;
 const {ts,unwrap,expr,access,fail}=api,n=unwrap(node);
 if(ts.isPropertyAccessExpression(n)){let root=n;const path=[];while(ts.isPropertyAccessExpression(root)&&!root.questionDotToken){path.unshift(root.name.text);root=root.expression;}if(root.kind===ts.SyntaxKind.ThisKeyword)return {kind:'registrationGrantor',path};}
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)){
  const call=n.expression,name=call.name.text;
  const methods=new Set(['isAttached','isAttacking','isDefending','isTokenUnit','hasCost','targetIsUnit','getSingleTarget','getAllTargets','getDefendingPlayer','isBase','hasSomeTrait','hasSomeAspect','canAttachPilot']);
  if(methods.has(name)&&(call.questionDotToken||n.questionDotToken||!['isBase','hasSomeTrait','hasSomeAspect'].includes(name))){if(n.arguments.length>2)fail(ctx,n,'registration-query-arity');return {kind:'registrationQuery',name,receiver:expr(call.expression,ctx),args:n.arguments.map(a=>expr(a,ctx)),optionalReceiver:!!call.questionDotToken,optionalCall:!!n.questionDotToken};}
  let p=[];try{p=access(call,ctx);}catch(error){if(!error.diagnostic)throw error;}if(p.join('.')==='EnumHelpers.isUnit'||p.join('.')==='DefeatCardSystem.defeatSourceCard'){if(n.arguments.length!==1)fail(ctx,n,'registration-static-query-arity');return {kind:'registrationStaticQuery',name:p.join('.'),args:n.arguments.map(a=>expr(a,ctx))};}
 }
 if(ts.isPropertyAccessExpression(n)){
  try{const p=access(n,ctx);if(p.length>=3&&p[1]==='game'&&p[2]==='currentPhase'){const r=ctx.scope.get(p[0]);if(r?.path?.join('.')==='context')return {kind:'registrationContext',path:['game',...p.slice(2)]};}}catch(error){if(!error.diagnostic)throw error;}
 }
 return null;
}
function registrationEffect(node,ctx,defaultTarget,api){
 if((!ctx.pendingRegistration&&!ctx.pendingSelection)||!api.ts.isCallExpression(node))return null;
 const {access,object,expr,fail}=api,p=access(node.expression,ctx),kind=p.at(-1);
 if(p.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])||p[1]!=='immediateEffects'||!['attachUpgrade','detachPilot','flipAndAttachPilotLeader','deployAndAttachPilotLeader','indirectDamageToPlayer','distributeDamageAmong','distributeHealingAmong','payResources','giveAdvantage','rescue'].includes(kind))return null;
 if(node.arguments.length>1)fail(ctx,node,'registration-effect-arity');
 const obj=node.arguments.length?object(node.arguments[0],ctx):{props:new Map(),ctx,node};
 const allowed={rescue:['target'],payResources:['amount','target'],giveAdvantage:['amount','target'],attachUpgrade:['upgrade','target','controller','takeControl'],detachPilot:['target'],flipAndAttachPilotLeader:['leaderPilotCard','target'],deployAndAttachPilotLeader:['leaderPilotCard','target'],indirectDamageToPlayer:['amount','target'],distributeDamageAmong:['amountToDistribute','canChooseNoTargets','canDistributeLess','controller','cardTypeFilter','zoneFilter','cardCondition'],distributeHealingAmong:['amountToDistribute','canChooseNoTargets','canDistributeLess','controller','cardTypeFilter','zoneFilter','cardCondition']}[kind];
 api.knownProperties(obj,allowed);
 const properties={};for(const [key,value]of obj.props)properties[key]=expr(value,obj.ctx,key==='cardCondition'?['card','context']:['context']);
 return {kind:'registrationSystem',system:kind,contextTarget:defaultTarget,defaults:'pinnedSystem',properties};
}
function registrationModifier(call,ctx,duration,target,api){
 if(!ctx.pendingRegistration)return null;
 const {ts,access,expr,fail,literal}=api;if(!ts.isCallExpression(call))return null;const p=access(call.expression,ctx),kind=p.at(-1);
 if(p.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])||p[1]!=='ongoingEffects'||!['gainTrait','cardCannot','modifyPilotingLimit','ignorePilotingPilotLimit','loseAllAbilities','canPlayFromDiscard','loseAllAbilitiesExceptFromSource','isLeader'].includes(kind))return null;
 if(call.arguments.length>1)fail(ctx,call,'registration-modifier-arity');
 let value=null;if(call.arguments.length){if(['gainTrait','cardCannot'].includes(kind))value=literal(call.arguments[0],ctx);else{const obj=api.object(call.arguments[0],ctx);api.knownProperties(obj,['amount']);value={};for(const[k,v]of obj.props)value[k]=expr(v,obj.ctx);}}
 return {kind:'registrationModifier',modifier:kind,target,duration,value};
}
module.exports={REGISTRATIONS,compileGrantedAbility,pilotingRegistration,registrationExpression,registrationEffect,registrationModifier};
