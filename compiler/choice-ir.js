'use strict';

function choiceContext(context){return {...context,pendingChoice:true,pendingSelection:true};}
function choiceMap(input,context,api){
 const {ts,unwrap,lambda,effect,contextTitle,fail}=api,lowered=lambda(input,choiceContext(context)),ctx=lowered.ctx,node=unwrap(lowered.node);
 const original=unwrap(input);let captured=false;
 if(ts.isArrowFunction(original)||ts.isFunctionExpression(original)){
  const outer=original.parameters[0]?.name;
  if(outer&&ts.isIdentifier(outer)){
   const search=(n,bound=true)=>{
    if(n!==original&&(ts.isArrowFunction(n)||ts.isFunctionExpression(n))&&n.parameters.some(p=>ts.isIdentifier(p.name)&&p.name.text===outer.text))bound=false;
    if(bound&&(ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n))){let root=n,first;while(ts.isPropertyAccessExpression(root)||ts.isElementAccessExpression(root)){first=ts.isPropertyAccessExpression(root)?root.name.text:ts.isStringLiteral(root.argumentExpression)?root.argumentExpression.text:null;root=root.expression;}if(ts.isIdentifier(root)&&root.text===outer.text&&['target','targets','select','selects'].includes(first))captured=true;}
    ts.forEachChild(n,c=>search(c,bound));
   };search(original.body);
  }
 }

 if(captured)return require('./selection-contracts').choiceFactoryMap(input,context,api);
 if(!ts.isObjectLiteralExpression(node))fail(ctx,node,'choice-map-object');
 if(!node.properties.length||node.properties.length>32)fail(ctx,node,'choice-map-bound');
 const entries=[];
 for(const p of node.properties){
  if(!ts.isPropertyAssignment(p))fail(ctx,p,'choice-map-property');
  const label=ts.isComputedPropertyName(p.name)?p.name.expression:ts.isIdentifier(p.name)?ts.factory.createStringLiteral(p.name.text):p.name;
  entries.push({labelParts:contextTitle(label,ctx),effect:effect(p.initializer,ctx,'self')});
 }
 return {entries,contextBinding:'resolverContextByReference',keyOrder:'javascriptPropertyOrder',duplicatePolicy:'lastAssignmentWins',factory:ts.isArrowFunction(unwrap(input))||ts.isFunctionExpression(unwrap(input))};
}
function choiceResolver(obj,api){
 const {props,node}=obj,ctx=choiceContext(obj.ctx),{literal,expr,knownProperties,contextTitle,fail}=api;
 if(!props.has('mode'))return null;const mode=literal(props.get('mode'),ctx);
 if(!['select','selectUnless','dropdownList'].includes(mode))return null;
 knownProperties(obj,['mode','choices','options','unlessEffect','defaultEffect','condition','checkTarget','showUnresolvable','highlightCards','logSelection','choosingPlayer','mustChangeGameState','hideIfNoLegalTargets','appendToDefaultTitle']);
 const result={kind:'choiceSelection',mode,resultBinding:'select',namedResultBinding:'selects.target.choice',defaults:'pinnedTargetResolver'};
 result.chooser=props.has('choosingPlayer')?expr(props.get('choosingPlayer'),ctx):{kind:'const',value:'self'};
 for(const field of ['condition','highlightCards'])if(props.has(field))result[field]=expr(props.get(field),ctx);
 for(const field of ['checkTarget','showUnresolvable','logSelection','hideIfNoLegalTargets'])if(props.has(field)){const value=literal(props.get(field),ctx);if(typeof value!=='boolean')fail(ctx,node,'choice-boolean:'+field);result[field]=value;}
 for(const field of ['mustChangeGameState','appendToDefaultTitle','waitingPromptTitle'])if(props.has(field))result[field]=literal(props.get(field),ctx);
 if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),ctx);
 if(mode==='select'){
  if(!props.has('choices'))fail(ctx,node,'choice-map-required');result.choices=choiceMap(props.get('choices'),ctx,api);
 }else if(mode==='dropdownList'){
  if(!props.has('options'))fail(ctx,node,'dropdown-options-required');result.options=expr(props.get('options'),ctx);result.logSelection=result.logSelection??true;
 }else{
  for(const key of ['defaultEffect','unlessEffect']){
   if(!props.has(key))fail(ctx,node,'choice-unless-pair');const entry=api.object(props.get(key),ctx);knownProperties(entry,['effect','promptButtonText']);if(!entry.props.has('effect')||!entry.props.has('promptButtonText'))fail(ctx,entry.node,'choice-unless-fields');
   result[key]={label:literal(entry.props.get('promptButtonText'),entry.ctx),effect:api.effect(entry.props.get('effect'),entry.ctx,'self')};
  }
  result.unlessLegalityMode='mustFullyOrPartiallyResolve';result.defaultAutomaticallyWhenUnlessIllegal=true;
 }
 return result;
}
function choiceExpression(input,ctx,api){
 if(!ctx.pendingChoice)return null;
 const {ts,unwrap,access}=api,node=unwrap(input);
 if(ts.isPropertyAccessExpression(node)){
  let p;try{p=access(node,ctx);}catch(error){if(!error.diagnostic)throw error;return null;}
  if(p[0]==='this'&&p[1]==='game'&&p.length===3&&['playableCardTitles','allNonLeaderCardTitles','traitNames'].includes(p[2]))return {kind:'choiceRegistry',registry:p[2],order:'engineRegistry',capture:'setupRegistryReference'};
 }
 return null;
}
function lookAtOptions(node,context,defaultTarget,api){
 const {ts,unwrap,access,object,expr,effect,contextTitle,literal,fail}=api;
 const p=access(node.expression,context);if(p.length!==3||p[1]!=='immediateEffects'||!['lookAtAndChooseOption','revealAndChooseOption'].includes(p[2]))return null;
 if(node.arguments.length!==1)fail(context,node,'look-at-options-arity');
 const factory=unwrap(node.arguments[0]),obj=object(factory,choiceContext(context));let ctx=obj.ctx;
 api.knownProperties(obj,['target','player','useDisplayPrompt','perCardButtons','message',...(p[2]==='revealAndChooseOption'?['promptedPlayer']:[])]);
 if(!obj.props.has('perCardButtons'))fail(ctx,obj.node,'look-at-buttons-required');
 const captures=[],scope=new Map(ctx.scope),constants=new Map(ctx.constants);
 ctx={...ctx,scope,constants};
 const stage=ctx.callbackStage;
 if(!(ts.isArrowFunction(factory)||ts.isFunctionExpression(factory))&&[...ctx.constants.values()].some(n=>ctx.constantStages?.get(n)===stage))fail(ctx,node,'choice-external-capture-stage');
 // Scalar/card declarations run once in the property factory. Button objects
 // remain declarative syntax, with references redirected to these captures.
 for(const binding of ctx.bindings||[]){
  if(ctx.constantStages?.get(binding.node)!==stage||constants.get(binding.name)!==binding.node)continue;
  const init=unwrap(binding.node);
  if(ts.isObjectLiteralExpression(init)||ts.isArrayLiteralExpression(init)||ts.isArrowFunction(init)||ts.isFunctionExpression(init)||ts.isConditionalExpression(init)&&[init.whenTrue,init.whenFalse].every(x=>ts.isArrayLiteralExpression(unwrap(x))))continue;
  const name='c'+binding.node.pos;
  captures.push({name,value:expr(binding.node,ctx)});scope.set(binding.name,{kind:'ref',path:['choiceCaptures',name]});constants.delete(binding.name);ctx.usedConstants.add(binding.node);
 }
 const resolve=input=>{
  let n=unwrap(input),seen=new Set();
  while(ts.isIdentifier(n)&&ctx.constants.has(n.text)){
   if(seen.has(n.text))fail(ctx,n,'choice-button-alias-cycle');seen.add(n.text);const value=ctx.constants.get(n.text);
   if(ctx.constantStages?.get(value)!==stage)fail(ctx,n,'choice-button-cross-stage-alias');ctx.usedConstants.add(value);n=unwrap(value);
  }
  return n;
 };
 const buttons=input=>{
  const n=resolve(input);
  if(ts.isConditionalExpression(n))return {kind:'conditionalButtons',condition:expr(n.condition,ctx),whenTrue:buttons(n.whenTrue),whenFalse:buttons(n.whenFalse)};
  if(!ts.isArrayLiteralExpression(n)||!n.elements.length||n.elements.length>16)fail(ctx,n,'choice-button-list');
  return {kind:'buttonList',buttons:n.elements.map(input=>{
   const button=object(resolve(input),ctx);api.knownProperties(button,['text','arg','immediateEffect']);
   if(!['text','arg','immediateEffect'].every(k=>button.props.has(k)))fail(ctx,button.node,'choice-button-fields');
   const arg=literal(button.props.get('arg'),ctx);if(typeof arg!=='string'||!arg.length||arg.length>128)fail(ctx,button.node,'choice-button-arg');
   return {textParts:contextTitle(button.props.get('text'),ctx),arg,effect:effect(button.props.get('immediateEffect'),ctx,'viewedCard')};
  })};
 };
 const reveal=p[2]==='revealAndChooseOption';
 const result={kind:reveal?'revealOptions':'lookAtOptions',interactMode:'perCardButtons',captures,captureStage:'propertyFactory',target:obj.props.has('target')?expr(obj.props.get('target'),ctx):defaultTarget,buttons:buttons(obj.props.get('perCardButtons')),defaults:'pinnedLookAtSystem',buttonDefaultTargetOverride:'clickedPhysicalCard',resolveWindow:'afterDisplayComplete'};
 if(reveal){result.revealContract={event:'onCardRevealed',legalZones:['deck','hand','resource'],checkEventCondition:'allCardsStillRevealable',revealedFromZone:'sharedZoneIfAllSame',defaultPromptedPlayer:'self'};if(obj.props.has('promptedPlayer'))result.promptedPlayer=expr(obj.props.get('promptedPlayer'),ctx);}
 if(obj.props.has('player'))result.player=expr(obj.props.get('player'),ctx);
 if(obj.props.has('useDisplayPrompt'))result.useDisplayPrompt=literal(obj.props.get('useDisplayPrompt'),ctx);
 if(obj.props.has('activePromptTitle'))result.promptTitleParts=contextTitle(obj.props.get('activePromptTitle'),ctx);
 return result;
}
function choiceNoAction(node,ctx,api){
 if(!ctx.pendingChoice)return null;
 const {access,object,expr,fail}=api,p=access(node.expression,ctx);if(p.length!==3||p[1]!=='immediateEffects'||p[2]!=='noAction')return null;
 if(node.arguments.length>1)fail(ctx,node,'choice-no-action-arity');const obj=node.arguments.length?object(node.arguments[0],ctx):{props:new Map(),ctx,node};api.knownProperties(obj,['hasLegalTarget']);
 return {kind:'choiceNoAction',...(obj.props.has('hasLegalTarget')?{hasLegalTarget:expr(obj.props.get('hasLegalTarget'),obj.ctx)}:{})};
}
function inspectChoiceHelpers(cls,api){
 const {ts,unwrap}=api,out=new Map();
 for(const member of cls.members){
  if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||member.parameters.length<1||member.parameters.length>2||member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0])||!member.body.statements[0].expression||member.asteriskToken||member.typeParameters?.length||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||ts.getDecorators(member)?.length||member.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||ts.getDecorators(p)?.length))continue;
  const body=unwrap(member.body.statements[0].expression);if(!ts.isCallExpression(body)||!ts.isPropertyAccessExpression(body.expression)||!ts.isPropertyAccessExpression(body.expression.expression)||!ts.isIdentifier(body.expression.expression.expression))continue;
  const helper=member.parameters.at(-1).name.text;if(!['AbilityHelper','abilityHelper'].includes(helper)||body.expression.expression.expression.text!==helper||body.expression.expression.name.text!=='immediateEffects')continue;
  if(cls.members.filter(m=>m.name?.getText()===member.name.text).length!==1)continue;
  out.set(member.name.text,{member,body,helper,used:false});
 }
 return out;
}
function choiceHelperCall(node,ctx,defaultTarget,api){
 if(!ctx.pendingChoice)return null;
 const {ts,unwrap,literal,effect,fail}=api,n=unwrap(node);
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||n.expression.expression.kind!==ts.SyntaxKind.ThisKeyword)return null;
 const helper=ctx.choiceHelpers?.get(n.expression.name.text);if(!helper)return null;
 if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==helper.member.parameters.length)fail(ctx,n,'choice-helper-call-shape');
 const actual=n.arguments.at(-1);if(!ts.isIdentifier(actual)||!['AbilityHelper','abilityHelper'].includes(actual.text)||ctx.constants.has(actual.text)||ctx.scope.has(actual.text))fail(ctx,n,'choice-helper-argument');
 const constants=new Map(ctx.constants),scopes=new Map(ctx.constantScopes),envs=new Map(ctx.constantEnvs),stages=new Map(ctx.constantStages);
 for(let i=0;i<n.arguments.length-1;i++){literal(n.arguments[i],ctx);const name=helper.member.parameters[i].name.text,value=n.arguments[i];constants.set(name,value);scopes.set(value,new Map(ctx.scope));envs.set(value,new Map(ctx.constants));stages.set(value,ctx.callbackStage||null);}
 helper.used=true;return effect(helper.body,{...ctx,constants,constantScopes:scopes,constantEnvs:envs,constantStages:stages},defaultTarget);
}
module.exports={choiceResolver,choiceExpression,lookAtOptions,choiceNoAction,inspectChoiceHelpers,choiceHelperCall};
