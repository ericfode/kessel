'use strict';

// Explicit pure programs, never opaque source/AST nodes. Object reads and finite
// query methods use the ordinary expression compiler; local mutation is scoped
// to these program frames and cannot write engine objects or caller bindings.
function synchronous(n,{ts}) {return !n.asteriskToken&&!n.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)&&!(ts.canHaveDecorators(n)&&ts.getDecorators(n)?.length);}
function inspect(cls,ctx,api) {
 const {ts}=api,result=new Map(),reserved=['historyHelpers','resultHelpers','maximumHelpers','defeatedUnitCountHelpers','reductionHelpers','arenaDamageHelpers','physicalAttachmentHelpers','selectedCostHelpers','constantRegistrarHelpers','choiceHelpers'];
 for(const m of cls.members){
  const name=m.name&&ts.isIdentifier(m.name)&&m.name.text;
  if(!name||!ts.isMethodDeclaration(m)||!m.modifiers?.some(x=>x.kind===ts.SyntaxKind.PrivateKeyword)||m.modifiers.some(x=>x.kind===ts.SyntaxKind.StaticKeyword)||!synchronous(m,api)||!m.body||m.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.modifiers?.length||ts.getDecorators(p)?.length)||cls.members.filter(x=>x.name?.getText(ctx.source)===name).length!==1||reserved.some(k=>ctx[k]?.has(name)))continue;
  let constructors=false,registration=false;const walk=n=>{if(ts.isPropertyAccessExpression(n)&&['immediateEffects','ongoingEffects','costs'].includes(n.name.text)&&ts.isIdentifier(n.expression)&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(n.expression.text))constructors=true;if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&/^add|^set/.test(n.expression.name.text)&&ts.isIdentifier(n.expression.expression)&&m.parameters.some(p=>p.name.text===n.expression.expression.text))registration=true;ts.forEachChild(n,walk);};walk(m.body);
  if(!registration)result.set(name,{member:m,name,construction:constructors,objectTemplate:m.parameters.every(p=>p.type?.getText(ctx.source).includes('IAbilityHelper'))&&m.body.statements.length===1&&ts.isReturnStatement(m.body.statements[0])&&ts.isObjectLiteralExpression(api.unwrap(m.body.statements[0].expression))});
 }
 return result;
}
function frame(ctx) {return {...ctx,scope:new Map(ctx.scope),constants:new Map(ctx.constants),dataRoots:new Set(ctx.dataRoots||[]),programLocals:new Map(ctx.programLocals||[]),ownedCollections:new Map(ctx.ownedCollections||[]),pureProgram:true,eventHistory:true};}
function bind(pattern,path,ctx,api,mutable=false) {
 const {ts,fail}=api;
 if(ts.isIdentifier(pattern)){ctx.scope.set(pattern.text,{kind:'ref',path});ctx.constants.delete(pattern.text);ctx.dataRoots.add(path[0]);ctx.programLocals.set(path.join('.'),mutable);return;}
 if(!ts.isObjectBindingPattern(pattern)&&!ts.isArrayBindingPattern(pattern))fail(ctx,pattern,'private-program-binding');
 pattern.elements.forEach((p,i)=>{if(ts.isOmittedExpression(p))return;if(p.dotDotDotToken||p.initializer)fail(ctx,p,'private-program-binding-default-or-rest');const name=p.propertyName||p.name;const key=ts.isArrayBindingPattern(pattern)?i:ts.isIdentifier(name)||ts.isStringLiteral(name)?name.text:null;if(key===null)fail(ctx,p,'private-program-binding-key');bind(p.name,[...path,key],ctx,api,mutable);});
}
function compileBlock(node,context,api) {
 const {ts,unwrap,expr,fail}=api;const local=frame(context);
 const value=(n,c)=>expr(n,c);
 const statement=(n,c)=>{
  if(ts.isBlock(n)){const next=frame(c);return {kind:'pureBlock',statements:n.statements.map(s=>statement(s,next))};}
  if(ts.isVariableStatement(n)){
   if(!(n.declarationList.flags&(ts.NodeFlags.Const|ts.NodeFlags.Let)))fail(c,n,'private-program-var');
   const mutable=!!(n.declarationList.flags&ts.NodeFlags.Let);
   return {kind:'pureBindings',bindings:n.declarationList.declarations.map(d=>{const v=d.initializer?value(d.initializer,c):{kind:'undefined'},slot='data:local:'+d.getStart(c.source);bind(d.name,[slot],c,api,mutable);if(ts.isIdentifier(d.name)&&['pureSet','array','pureArray'].includes(v.kind))c.ownedCollections.set(slot,v.kind==='pureSet'?'set':'array');return {slot,mutable,value:v,bindingPattern:bindingShape(d.name,api),evaluateAt:'declaration'};})};
  }
  if(ts.isReturnStatement(n))return {kind:'pureReturn',value:n.expression?value(n.expression,c):{kind:'undefined'}};
  if(ts.isIfStatement(n))return {kind:'pureIf',condition:value(n.expression,c),then:statement(n.thenStatement,frame(c)),...(n.elseStatement?{else:statement(n.elseStatement,frame(c))}:{})};
  if(ts.isForOfStatement(n)){
   if(n.awaitModifier||!ts.isVariableDeclarationList(n.initializer)||n.initializer.declarations.length!==1||!(n.initializer.flags&(ts.NodeFlags.Const|ts.NodeFlags.Let)))fail(c,n,'private-program-for-of');
   const iterable=value(n.expression,c),next=frame(c),decl=n.initializer.declarations[0],slot='data:iteration:'+decl.getStart(c.source);if(decl.initializer)fail(c,decl,'private-program-for-of-initializer');bind(decl.name,[slot],next,api,!!(n.initializer.flags&ts.NodeFlags.Let));return {kind:'pureForOf',slot,bindingPattern:bindingShape(decl.name,api),iterable,iterationScope:'perIteration',body:statement(n.statement,next)};
  }
  if(ts.isSwitchStatement(n)){const next=frame(c);return {kind:'pureSwitch',value:value(n.expression,c),clauses:n.caseBlock.clauses.map(cl=>({...ts.isCaseClause(cl)?{value:value(cl.expression,next)}:{default:true},statements:cl.statements.map(s=>statement(s,next))}))};}
  if(ts.isBreakStatement(n)&&!n.label)return {kind:'pureBreak'};
  if(ts.isContinueStatement(n)&&!n.label)return {kind:'pureContinue'};
  if(ts.isExpressionStatement(n))return {kind:'pureEvaluate',value:value(n.expression,c)};
  fail(c,n,'private-program-statement:'+ts.SyntaxKind[n.kind]);
 };
 return statement(node,local);
}
function bindingShape(p,{ts}) {if(ts.isIdentifier(p))return {name:p.text};return {kind:ts.isArrayBindingPattern(p)?'array':'object',entries:p.elements.map((e,i)=>ts.isOmittedExpression(e)?null:{key:ts.isArrayBindingPattern(p)?i:(e.propertyName||e.name).text,binding:bindingShape(e.name,{ts})})};}
function compileCallback(input,context,api,names=['context']) {
 const {ts,unwrap,fail,expr}=api,n=unwrap(input);if(!ts.isArrowFunction(n)&&!ts.isFunctionExpression(n))return null;
 if(!synchronous(n,api)||n.parameters.some(p=>p.initializer||p.dotDotDotToken||p.modifiers?.length||ts.getDecorators(p)?.length))fail(context,n,'private-program-callback');
 const ctx=frame(context),parameters=n.parameters.map((p,i)=>{const slot='data:callback:'+n.getStart(ctx.source)+':'+i;bind(p.name,[slot],ctx,api,true);return {slot,argument:names[i]||'argument'+i,bindingPattern:bindingShape(p.name,api)};});ctx.callbackStage=n;
 return {kind:'pureCallback',parameters,body:ts.isBlock(n.body)?compileBlock(n.body,ctx,api):{kind:'pureReturn',value:expr(n.body,ctx)},capture:'lexicalReferences',invokeAt:'nativeCallbackInvocation'};
}
function capabilityArgument(n,ctx,api){const {ts,fail}=api;if(ctx.scope.has(n.text)||ctx.constants.has(n.text))fail(ctx,n,'private-helper-capability-shadow');for(let a=n.parent;a;a=a.parent){if(ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===n.text)))fail(ctx,n,'private-helper-capability-shadow');const p=a.parameters?.find(p=>ts.isIdentifier(p.name)&&p.name.text===n.text);if(p){if(!p.type?.getText(ctx.source).includes('IAbilityHelper'))fail(ctx,n,'private-helper-capability-binding');return {kind:'lexicalCapability',capability:'AbilityHelper',name:n.text};}}fail(ctx,n,'private-helper-capability-origin');}
function callArgument(n,ctx,api) {const {ts,unwrap}=api;n=unwrap(n);if(ts.isIdentifier(n)&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(n.text))return capabilityArgument(n,ctx,api);if(ts.isIdentifier(n)&&ctx.scope.get(n.text)?.path?.join('.')==='context')return {kind:'abilityContextValue',binding:require('./event-result-context').contextBinding(n,ctx,api)};return api.expr(n,ctx);}
function normalizeExpression(input,ctx,api) {
 const {ts,unwrap,expr,fail,unshadowedEnum}=api,n=unwrap(input);
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.expression.kind===ts.SyntaxKind.ThisKeyword&&ctx.privatePrograms?.has(n.expression.name.text)){
  const helper=ctx.privatePrograms.get(n.expression.name.text),min=helper.member.parameters.filter(p=>!p.questionToken).length;
  if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length<min||n.arguments.length>helper.member.parameters.length)fail(ctx,n,'private-program-call-arity');
  for(let a=n.parent;a;a=a.parent)if(ts.isFunctionExpression(a))fail(ctx,n,'private-program-call-this');
  return {kind:helper.construction?'privateConstructionCall':'privateProgramCall',name:helper.name,args:n.arguments.map(a=>callArgument(a,ctx,api)),source:'physicalAbilitySource',invokeAt:'expressionEvaluation'};
 }
 if(!ctx.pureProgram)return null;
 if(ctx.constructionProgram&&ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.expression?.name?.text==='immediateEffects'){return api.systemConstructionValue?api.systemConstructionValue(n,ctx,'self',api):{kind:'systemConstructionValue',system:api.effect(n,ctx),evaluateAt:'programExpression',callbacks:'deferredWithLexicalBindings',captureIdentity:'valueOrPhysicalObjectReference'};}
 if(ts.isIdentifier(n)&&n.text==='undefined'&&!ctx.scope.has(n.text)&&!ctx.constants.has(n.text))return {kind:'undefined'};
 if(ts.isNewExpression(n)){
  const name=n.expression.getText(ctx.source);if(!['Set','Helpers.IntersectingSet'].includes(name)||(n.arguments?.length||0)>1)fail(ctx,n,'private-program-constructor:'+name);unshadowedEnum(n.expression,ctx,name.split('.')[0]);return {kind:'pureSet',implementation:name,values:n.arguments?.length?expr(n.arguments[0],ctx):{kind:'array',items:[]}};
 }
 if(ts.isArrayLiteralExpression(n)&&n.elements.some(ts.isSpreadElement))return {kind:'pureArray',items:n.elements.map(e=>ts.isSpreadElement(e)?{spread:expr(e.expression,ctx)}:{value:expr(e,ctx)}),order:'leftToRight'};
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)){
  const name=n.expression.name.text,receiver=n.expression.expression;
  if(['forEach','some','every','filter','find','map','flatMap','reduce'].includes(name))return {kind:'pureCollectionCall',method:name,receiver:expr(receiver,ctx),args:n.arguments.map(a=>{const v=unwrap(a);return ts.isArrowFunction(v)||ts.isFunctionExpression(v)?compileCallback(v,ctx,api,name==='reduce'?['accumulator','item','index','collection']:['item','index','collection']):expr(a,ctx);}),optionalReceiver:!!n.expression.questionDotToken,optionalCall:!!n.questionDotToken};
  if(name==='push'){if(!ts.isIdentifier(receiver)||!ctx.scope.has(receiver.text)||ctx.ownedCollections.get(ctx.scope.get(receiver.text).path.join('.'))!=='array'||n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'private-program-owned-array');return {kind:'localArrayAppend',receiver:expr(receiver,ctx),values:n.arguments.map(a=>expr(a,ctx)),evaluationOrder:'leftToRight',returns:'newLength'};}
  if(['add','delete','intersect'].includes(name)){
   if(!ts.isIdentifier(receiver)||!ctx.scope.has(receiver.text))fail(ctx,n,'private-program-set-receiver');const ref=ctx.scope.get(receiver.text);if(ctx.ownedCollections.get(ref.path.join('.'))!=='set'||n.arguments.length!==1||n.questionDotToken||n.expression.questionDotToken)fail(ctx,n,'private-program-local-set');return {kind:'pureSetMutation',method:name,receiver:expr(receiver,ctx),value:expr(n.arguments[0],ctx)};
  }
  if(receiver.getText(ctx.source)==='Helpers'&&['asArray','setIntersection'].includes(name)){unshadowedEnum(receiver,ctx,'Helpers');return {kind:'pureUtility',utility:'Helpers.'+name,args:n.arguments.map(a=>expr(a,ctx))};}
  if(receiver.getText(ctx.source)==='Math'&&['min','max','floor','ceil','abs'].includes(name)){unshadowedEnum(receiver,ctx,'Math');return {kind:'math',name,args:n.arguments.map(a=>expr(a,ctx))};}
 }
 if(ts.isBinaryExpression(n)&&['=','+=','-=','*=','/='].includes(n.operatorToken.getText(ctx.source))){
  if(!ts.isIdentifier(n.left)||ctx.scope.get(n.left.text)?.kind!=='ref'||ctx.programLocals.get(ctx.scope.get(n.left.text).path.join('.'))!==true)fail(ctx,n,'private-program-local-write');return {kind:'pureLocalWrite',operation:n.operatorToken.getText(ctx.source),target:ctx.scope.get(n.left.text),value:expr(n.right,ctx)};
 }
 return null;
}
function definitions(ctx,api){
 const result=[];
 for(const[name,h]of ctx.privatePrograms||[]){if(h.objectTemplate)continue;const local=frame({...ctx,constructionProgram:h.construction}),parameters=h.member.parameters.map((p,i)=>{const slot='data:private:'+name+':'+i;if(p.type?.getText(ctx.source).includes('IAbilityHelper')){local.helperCapabilities=new Set([...(local.helperCapabilities||[]),p.name.text]);return {slot,optional:!!p.questionToken,capability:'AbilityHelper',bindingName:p.name.text};}bind(p.name,[slot],local,api,true);return {slot,optional:!!p.questionToken};});result.push({kind:h.construction?'constructionProgram':'privateProgram',name,parameters,body:compileBlock(h.member.body,local,api),source:'physicalAbilitySource'});}
 return result;
}
function effectCall(input,ctx,defaultTarget,api){const {ts,unwrap}=api,n=unwrap(input);if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||n.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||!ctx.privatePrograms?.get(n.expression.name.text)?.construction)return null;return {...normalizeExpression(n,ctx,api),defaultTarget,resultType:'gameSystem',invocationPolicy:'sourceMethodCall'};}
function objectTemplate(input,ctx,api){const {ts,unwrap,fail}=api,n=unwrap(input);if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||n.expression.expression.kind!==ts.SyntaxKind.ThisKeyword)return null;const h=ctx.privatePrograms?.get(n.expression.name.text);if(!h?.objectTemplate)return null;if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==h.member.parameters.length||n.arguments.some(a=>!ts.isIdentifier(a)||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(a.text)))fail(ctx,n,'private-object-capability-arguments');n.arguments.forEach(a=>capabilityArgument(a,ctx,api));h.used=true;return {node:h.member.body.statements[0].expression,ctx:{...ctx,privateObjectTemplate:true}};}
function computedKey(input,ctx,api){const {ts,unwrap,literal,unshadowedEnum,fail}=api,n=unwrap(input);let root=n;while(ts.isPropertyAccessExpression(root))root=root.expression;if(ts.isIdentifier(root)&&ts.isPropertyAccessExpression(n))unshadowedEnum(root,ctx,root.text);const key=literal(n,ctx);if(typeof key!=='string'&&typeof key!=='number')fail(ctx,n,'private-computed-key');return String(key);}
module.exports={computedKey,inspect,compileBlock,compileCallback,normalizeExpression,definitions,effectCall,objectTemplate};
