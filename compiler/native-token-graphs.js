'use strict';

// Complete, data-only source graphs for token classes with constructor state
// and locally declared triggered-ability classes. Every member and call has a
// checked semantic operation; unrecognized members reject the entire graph.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const ts=require(require.resolve('typescript',{paths:[path.resolve(__dirname,'../forceteki')]}));
const hash=s=>crypto.createHash('sha256').update(s).digest('hex');
const constant=value=>({kind:'const',value});
const ref=(...path)=>({kind:'ref',path});
function compileNativeTokenGraph(text,file,sourceRoot,enums,inheritance){
 if(require('./source-exclusions').sourceExclusion(file)||!file.split('/').includes('tokens'))return null;
 const source=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true);
 const classes=source.statements.filter(ts.isClassDeclaration);
 if(source.parseDiagnostics.length||classes.length<1||classes.length>2)return null;
 const primary=classes.find(c=>c.modifiers?.some(m=>m.kind===ts.SyntaxKind.DefaultKeyword));
 if(!primary||ts.getDecorators(primary)?.length||primary.heritageClauses?.length!==1||primary.heritageClauses[0].types.length!==1||primary.heritageClauses[0].types[0].expression.getText(source)!=='TokenUpgradeCard')return null;
 const reject=()=>{throw new Error('unsupported-native-token-graph');};
 const imports=new Map(),classNames=new Map(classes.map(c=>[c.name?.text,c]));
 const imported=(name,module)=>{const v=imports.get(name);return v&&!v.typeOnly&&v.name===name&&path.posix.normalize(path.posix.join(path.posix.dirname(file),v.module))===module;};
 const plainName=n=>n&&(ts.isIdentifier(n)||ts.isStringLiteral(n))?n.text:null;
 const methodName=m=>plainName(m.name);
 const fields=new Map();
 let markerName=null;
 const bind=(parameters,roles,types,optionalLast=false)=>{
  if(parameters.length!==roles.length)reject();const scope=new Map();
  parameters.forEach((p,i)=>{if(!ts.isIdentifier(p.name)||scope.has(p.name.text)||p.initializer||p.dotDotDotToken||p.modifiers?.length||ts.getDecorators(p)?.length||!!p.questionToken!==(optionalLast&&i===parameters.length-1)||p.type?.getText(source)!==types[i])reject();scope.set(p.name.text,ref(roles[i]));});return scope;
 };
 const accessPath=n=>{if(ts.isIdentifier(n))return [n.text];if(n.kind===ts.SyntaxKind.ThisKeyword)return ['this'];if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){const p=accessPath(n.expression);if(p)return [...p,n.name.text];}return null;};
 const returnExpression=m=>{if(m.body?.statements.length!==1||!ts.isReturnStatement(m.body.statements[0])||!m.body.statements[0].expression)reject();return m.body.statements[0].expression;};
 const expr=(n,scope)=>{
  if(ts.isParenthesizedExpression(n))return expr(n.expression,scope);
  if(n.kind===ts.SyntaxKind.ThisKeyword)return ref('instance');
  if(n.kind===ts.SyntaxKind.TrueKeyword)return constant(true);
  if(n.kind===ts.SyntaxKind.FalseKeyword)return constant(false);
  if(ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n))return constant(n.text);
  if(ts.isIdentifier(n)&&scope.has(n.text))return scope.get(n.text);
  if(ts.isPropertyAccessExpression(n)){
   if(ts.isIdentifier(n.expression)&&['TokenUpgradeName','DamageModificationType'].includes(n.expression.text)){
    if(n.questionDotToken||!imported(n.expression.text,'game/core/Constants'))reject();const value=enums[n.expression.text+'.'+n.name.text];if(value===undefined)reject();return constant(value);
   }
   const parent=expr(n.expression,scope),p=parent.kind==='ref'?parent.path.join('.'):null;
   const allowed={instance:['title','game'],context:['source'],event:['attack'],'event.attack':['attacker'],'context.source':['parentCard'],'context.source.parentCard':['title'],additionalProperties:['highPriorityRemoval']};
   if(!p||!allowed[p]?.includes(n.name.text)||n.questionDotToken&&p!=='additionalProperties')reject();
   return n.questionDotToken?{kind:'optionalMember',receiver:parent,property:n.name.text,missingValue:'undefined'}:ref(...parent.path,n.name.text);
  }
  if(ts.isPrefixUnaryExpression(n)&&n.operator===ts.SyntaxKind.ExclamationToken)return {kind:'unary',op:'!',value:expr(n.operand,scope)};
  if(ts.isBinaryExpression(n)){
   const op=ts.tokenToString(n.operatorToken.kind);if(!['===','&&','||'].includes(op))reject();return {kind:'binary',op,left:expr(n.left,scope),right:expr(n.right,scope),...(['&&','||'].includes(op)?{evaluation:'shortCircuit'}:{})};
  }
  if(ts.isConditionalExpression(n))return {kind:'conditional',condition:expr(n.condition,scope),then:expr(n.whenTrue,scope),else:expr(n.whenFalse,scope),evaluation:'chosenBranchOnly'};
  if(ts.isTemplateExpression(n))return {kind:'textTemplate',parts:[constant(n.head.text),...n.templateSpans.flatMap(s=>[expr(s.expression,scope),constant(s.literal.text)])]};
  if(ts.isCallExpression(n)&&!n.questionDotToken&&!n.typeArguments?.length&&ts.isPropertyAccessExpression(n.expression)&&!n.expression.questionDotToken){
   const callee=n.expression,name=callee.name.text;
   if(['isInPlay','isUpgrade',markerName].includes(name)&&n.arguments.length===0){const receiver=expr(callee.expression,scope);if(receiver.kind!=='ref'||!['context.source','card'].includes(receiver.path.join('.')))reject();return {kind:'query',name,receiver,args:[]};}
   if(name==='getAllTargets'&&!n.arguments.length){const receiver=expr(callee.expression,scope);if(receiver.kind!=='ref'||receiver.path.join('.')!=='event.attack')reject();return {kind:'attackTargetList',receiver,order:'nativeAttackTargetOrder'};}
   if(name==='includes'&&n.arguments.length===1){const receiver=expr(callee.expression,scope);if(receiver.kind!=='attackTargetList')reject();return {kind:'arrayIncludesIdentity',receiver,value:expr(n.arguments[0],scope)};}
  }
  reject();
 };
 const callback=(n,roles,scope,lexicalThis)=>{
  if(!ts.isArrowFunction(n)||n.typeParameters?.length||n.modifiers?.length||n.parameters.length!==roles.length)reject();
  const local=new Map(scope);
  n.parameters.forEach((p,i)=>{if(!ts.isIdentifier(p.name)||local.has(p.name.text)||p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length||ts.getDecorators(p)?.length)reject();local.set(p.name.text,ref(roles[i]));});
  if(ts.isBlock(n.body))reject();return {kind:'boundCallback',parameters:roles,timing:'nativeAbilityCallbackInvocation',lexicalThis,value:expr(n.body,local)};
 };
 const object=(n,allowed)=>{
  if(!ts.isObjectLiteralExpression(n))reject();const props=new Map();
  for(const p of n.properties){if(!ts.isPropertyAssignment(p))reject();const key=plainName(p.name);if(!key||props.has(key)||!allowed.includes(key))reject();props.set(key,p.initializer);}return props;
 };
 const defeatSystem=(n,scope)=>{
  if(!ts.isCallExpression(n)||n.questionDotToken||n.arguments.length||n.typeArguments?.length)reject();
  const p=accessPath(n.expression);if(!p||p.at(-1)!=='defeat'||p.at(-2)!=='immediateEffects')reject();
  const helper=p.length===3&&scope.get(p[0])?.path?.join('.')==='abilityHelper';
  const game=p.length===4&&p[1]==='abilityHelper'&&scope.get(p[0])?.path?.join('.')==='game';
  if(!helper&&!game)reject();return {kind:'defeat',target:'self'};
 };
 const abilityProperties=(n,scope,damage,lexicalThis)=>{
  const props=object(n,damage?['title','contextTitle','modificationType','shouldCardHaveDamageModification','replaceWithEffect']:['title','when','immediateEffect']);
  const required=damage?['title','contextTitle','modificationType','shouldCardHaveDamageModification','replaceWithEffect']:['title','when','immediateEffect'];if(required.some(k=>!props.has(k)))reject();
  const result={kind:damage?'damageModificationRegistration':'triggeredAbilityProperties',title:expr(props.get('title'),scope)};
  if(damage){result.contextTitle=callback(props.get('contextTitle'),['context'],scope,lexicalThis);result.modificationType=expr(props.get('modificationType'),scope);if(result.modificationType.value!=='replace')reject();result.shouldCardHaveDamageModification=callback(props.get('shouldCardHaveDamageModification'),['damageCandidate','context'],scope,lexicalThis);result.replaceWithEffect=defeatSystem(props.get('replaceWithEffect'),scope);result.contract='DamageModificationAbility';result.triggerContract='damageEventBeforeResolution';}
  else {const hooks=object(props.get('when'),['onAttackEnd']);if(hooks.size!==1)reject();result.hooks=[{event:'onAttackEnd',predicate:callback(hooks.get('onAttackEnd'),['event','context'],scope,lexicalThis)}];result.immediateEffect=defeatSystem(props.get('immediateEffect'),scope);}
  return result;
 };
 try{
  for(const s of source.statements){if(ts.isImportDeclaration(s)){const c=s.importClause;if(c?.name)reject();if(c?.namedBindings){if(!ts.isNamedImports(c.namedBindings))reject();for(const e of c.namedBindings.elements){if(imports.has(e.name.text))reject();imports.set(e.name.text,{name:e.propertyName?.text??e.name.text,module:s.moduleSpecifier.text,typeOnly:!!(c.isTypeOnly||e.isTypeOnly)});}}}else if(!classes.includes(s))reject();}
  if(!imported('TokenUpgradeCard','game/core/card/TokenCards'))reject();
  for(const m of primary.members){if(ts.isMethodDeclaration(m)&&/^is[A-Z]/.test(methodName(m))){if(markerName)reject();markerName=methodName(m);}}
  if(!markerName)reject();
  const methods=[],constructorSteps=[];let identity=null,hasConstructor=false;
  for(const m of primary.members){
   if(ts.getDecorators(m)?.length||m.modifiers?.some(v=>v.kind===ts.SyntaxKind.AsyncKeyword))reject();
   const name=methodName(m);
   if(ts.isConstructorDeclaration(m)){
    if(hasConstructor||ts.getDecorators(m)?.length)reject();hasConstructor=true;
    const scope=bind(m.parameters,['owner','cardData','additionalProperties'],['Player','ICardDataJson','any'],true);
    if(m.body?.statements.length!==2)reject();
    for(const statement of m.body.statements){
     if(!ts.isExpressionStatement(statement))reject();const n=statement.expression;
     if(ts.isCallExpression(n)&&n.expression.kind===ts.SyntaxKind.SuperKeyword&&!n.questionDotToken&&!n.typeArguments?.length&&n.arguments.length===2){const args=n.arguments.map(a=>expr(a,scope));if(JSON.stringify(args)!==JSON.stringify([ref('owner'),ref('cardData')]))reject();constructorSteps.push({kind:'superConstructor',baseClass:'TokenUpgradeCard',args});}
     else if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsToken&&ts.isPropertyAccessExpression(n.left)&&n.left.expression.kind===ts.SyntaxKind.ThisKeyword&&!n.left.questionDotToken&&fields.has(n.left.name.text)){constructorSteps.push({kind:'initializeTokenField',field:n.left.name.text,value:expr(n.right,scope)});}
     else reject();
    }
    if(constructorSteps[0]?.kind!=='superConstructor'||constructorSteps[1]?.kind!=='initializeTokenField')reject();continue;
   }
   if(ts.isPropertyDeclaration(m)){if(name!=='highPriorityRemoval'||m.initializer||m.type?.kind!==ts.SyntaxKind.BooleanKeyword||!m.modifiers?.some(v=>v.kind===ts.SyntaxKind.ReadonlyKeyword)||fields.has(name))reject();fields.set(name,{name,type:'boolean',readonly:true});continue;}
   if(!ts.isMethodDeclaration(m)&&!ts.isGetAccessorDeclaration(m)||m.typeParameters?.length||m.asteriskToken)reject();
   if(name==='getImplementationId'){if(m.parameters.length)reject();const props=object(returnExpression(m),['id','internalName']);if(props.size!==2)reject();identity={};for(const[k,v]of props){const value=expr(v,new Map());if(value.kind!=='const'||typeof value.value!=='string')reject();identity[k]=value.value;}continue;}
   if(name===markerName){if(m.parameters.length||returnExpression(m).kind!==ts.SyntaxKind.TrueKeyword)reject();methods.push({name,kind:'typePredicate',body:[{kind:'return',value:constant(true)}]});continue;}
   if(name==='tokenName'&&ts.isGetAccessorDeclaration(m)){if(m.parameters.length)reject();methods.push({name,kind:'getter',body:[{kind:'return',value:expr(returnExpression(m),new Map())}]});continue;}
   if(name==='setupCardAbilities'){
    const scope=bind(m.parameters,['registrar','abilityHelper'],['IUpgradeAbilityRegistrar','IAbilityHelper']);
    if(m.body?.statements.length!==1||!ts.isExpressionStatement(m.body.statements[0]))reject();const call=m.body.statements[0].expression;
    if(!ts.isCallExpression(call)||call.questionDotToken||call.typeArguments?.length||call.arguments.length!==1||!ts.isPropertyAccessExpression(call.expression)||call.expression.questionDotToken||!ts.isIdentifier(call.expression.expression)||scope.get(call.expression.expression.text)?.path?.join('.')!=='registrar'||call.expression.name.text!=='addDamageModificationAbility')reject();
    methods.push({name,kind:'abilitySetup',parameters:[{name:'registrar',type:'upgradeAbilityRegistrar'},{name:'abilityHelper',type:'abilityHelper'}],body:[{kind:'registerAbility',registration:'addDamageModificationAbility',properties:abilityProperties(call.arguments[0],scope,true,{kind:'ref',path:['instance']})}]});continue;
   }
   if(name==='buildWhenAttackOrDefenseEndsAbilities'){
    if(m.parameters.length)reject();const arr=returnExpression(m);if(!ts.isArrayLiteralExpression(arr)||arr.elements.length!==1)reject();const create=arr.elements[0];if(!ts.isNewExpression(create)||create.typeArguments?.length||!ts.isIdentifier(create.expression)||!classNames.has(create.expression.text)||create.expression.text===primary.name.text||create.arguments?.length!==2)reject();
    const args=create.arguments.map(a=>expr(a,new Map()));if(JSON.stringify(args)!==JSON.stringify([ref('instance','game'),ref('instance')]))reject();methods.push({name,kind:'attackBoundaryAbilityFactory',body:[{kind:'return',value:{kind:'constructedArray',items:[{kind:'constructLocalAbility',className:create.expression.text,args,evaluationOrder:'leftToRight'}]}}]});continue;
   }
   reject();
  }
  if(!identity||!methods.some(m=>m.name==='tokenName')||fields.size&&!hasConstructor)reject();
  const auxiliary=[];
  for(const cls of classes.filter(c=>c!==primary)){
   if(cls.heritageClauses?.length!==1||cls.heritageClauses[0].types.length!==1||cls.heritageClauses[0].types[0].expression.getText(source)!=='TriggeredAbilityBase'||!imported('TriggeredAbilityBase','game/core/ability/TriggeredAbility'))reject();
   const decorators=ts.getDecorators(cls)||[];if(decorators.length!==1||!ts.isCallExpression(decorators[0].expression)||decorators[0].expression.arguments.length||!ts.isIdentifier(decorators[0].expression.expression)||decorators[0].expression.expression.text!=='registerState'||!imported('registerState','game/core/GameObjectUtils'))reject();
   const constructors=cls.members.filter(ts.isConstructorDeclaration),factories=cls.members.filter(ts.isMethodDeclaration);if(constructors.length!==1||factories.length!==1||cls.members.length!==2)reject();
   const factory=factories[0],factoryName=methodName(factory);if(!factoryName||!factory.modifiers?.some(m=>m.kind===ts.SyntaxKind.StaticKeyword)||!factory.modifiers?.some(m=>m.kind===ts.SyntaxKind.PrivateKeyword)||factory.typeParameters?.length||factory.asteriskToken||ts.getDecorators(factory)?.length||factory.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword))reject();
   const factoryScope=bind(factory.parameters,['game'],['Game']),properties=abilityProperties(returnExpression(factory),factoryScope,false,{kind:'localClassReference',className:cls.name.text});
   const ctor=constructors[0];if(ts.getDecorators(ctor)?.length||ctor.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword))reject();const scope=bind(ctor.parameters,['game','card'],['Game',primary.name.text]),steps=[];
   if(ctor.body?.statements.length!==3)reject();
   for(const statement of ctor.body.statements){
    if(!ts.isExpressionStatement(statement)||!ts.isCallExpression(statement.expression))reject();const call=statement.expression;if(call.questionDotToken||call.typeArguments?.length)reject();const p=accessPath(call.expression);
    if(p?.join('.')==='Contract.assertTrue'&&imported('Contract','game/core/utils/Contract')&&call.arguments.length===1){steps.push({kind:'assert',condition:expr(call.arguments[0],scope),failure:'throw'});continue;}
    if(call.expression.kind===ts.SyntaxKind.SuperKeyword&&call.arguments.length===3){const propsCall=call.arguments[2];if(!ts.isCallExpression(propsCall)||propsCall.questionDotToken||propsCall.typeArguments?.length||propsCall.arguments.length!==1||accessPath(propsCall.expression)?.join('.')!==cls.name.text+'.'+factoryName)reject();const args=call.arguments.slice(0,2).map(a=>expr(a,scope));if(JSON.stringify(args)!==JSON.stringify([ref('game'),ref('card')])||JSON.stringify(expr(propsCall.arguments[0],scope))!==JSON.stringify(ref('game')))reject();steps.push({kind:'superConstructor',baseClass:'TriggeredAbilityBase',args:[...args,{kind:'invokeLocalStaticFactory',className:cls.name.text,method:factoryName,args:[ref('game')]}],defaultAbilityType:'triggered'});continue;}
    reject();
   }
   if(steps[0]?.kind!=='assert'||steps[1]?.kind!=='assert'||steps[2]?.kind!=='superConstructor')reject();
   auxiliary.push({className:cls.name.text,baseClass:'TriggeredAbilityBase',stateRegistration:'registerState',constructor:{parameters:[{name:'game',type:'game'},{name:'card',type:'tokenInstance'}],steps},staticFactories:[{name:factoryName,parameters:[{name:'game',type:'game'}],body:[{kind:'return',value:properties}]}]});
  }
  if(methods.some(m=>m.kind==='attackBoundaryAbilityFactory')!==!!auxiliary.length||hasConstructor&&auxiliary.length||!hasConstructor&&!auxiliary.length)reject();
  const inherited=inheritance(sourceRoot,enums,'TokenUpgradeCard');if(!inherited)reject();
  const dependencies=[...inherited.dependencies];
  if(auxiliary.length){for(const dependency of ['game/core/ability/TriggeredAbility.ts','game/core/card/baseClasses/InPlayCard.ts'])dependencies.push({sourcePath:dependency,sourceHash:hash(fs.readFileSync(path.join(sourceRoot,dependency),'utf8'))});}
  else {const dependency='game/core/ability/DamageModificationAbility.ts';dependencies.push({sourcePath:dependency,sourceHash:hash(fs.readFileSync(path.join(sourceRoot,dependency),'utf8'))});}
  return {role:'staged-native-token-lifecycle',sourcePath:file,sourceHash:hash(text),className:primary.name.text,identity,status:'lifecycle-complete',activeCardEntryUnchanged:true,baseClass:'TokenUpgradeCard',inheritedBase:'InPlayCard',tokenMixin:inherited.tokenMixin,fields:[...fields.values()],constructor:hasConstructor?{parameters:[{name:'owner',type:'player'},{name:'cardData',type:'cardMetadata'},{name:'additionalProperties',type:'record',optional:true}],steps:constructorSteps}:{kind:'inheritedConstructor',baseClass:'TokenUpgradeCard'},methods,localClasses:auxiliary,...(auxiliary.length?{manualRegistration:{entryPoint:'InPlayCard.checkRegisterWhenAttackOrDefenseEndsAbilities',invokes:'buildWhenAttackOrDefenseEndsAbilities',stores:'whenAttackOrDefenseEndsAbilities',registerEvents:true,cleanup:'parentEventCleanupHandlerUnregistersAndClearsStoredAbilities',assertNoPreviousRegistration:true}}:{}),dependencies};
 }catch(error){if(error.message==='unsupported-native-token-graph')return null;throw error;}
}
module.exports={compileNativeTokenGraph};
