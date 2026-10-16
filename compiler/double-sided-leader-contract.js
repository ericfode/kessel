'use strict';

// Finite typed method graph for the non-deploying, two-sided leader base.
// All its own methods, state writes, callbacks and inherited calls are encoded.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const ts=require(require.resolve('typescript',{paths:[path.resolve(__dirname,'../forceteki')]}));
const hash=s=>crypto.createHash('sha256').update(s).digest('hex');
const constant=value=>({kind:'const',value});
const ref=(...path)=>({kind:'ref',path});
function compileDoubleSidedLeaderContract(sourceRoot,enums){
 const file='game/core/card/DoubleSidedLeaderCard.ts';if(!fs.existsSync(path.join(sourceRoot,file)))return null;
 const text=fs.readFileSync(path.join(sourceRoot,file),'utf8'),source=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true);
 const reject=()=>{throw new Error('unsupported-double-sided-leader-contract');};
 const imports=new Map(),aliases=new Map();
 const bound=(name,module)=>{const p=imports.get(name);return p&&!p.typeOnly&&p.name===name&&path.posix.normalize(path.posix.join(path.posix.dirname(file),p.module))===module;};
 const thisFields=new Set(['setupLeaderBackSide','_onStartingSide','onStartingSide','_aspects','_backSideAspects','_title','_backSideTitle','printedTraits','backsidePrintedTraits','game','exhausted']);
 const inheritedMethods=new Set(['onInitialize','getAbilityRegistrar','initializeForStartZone','getSummary','createActionAbility','createConstantAbility','createTriggeredAbility','createReplacementEffectAbility','initializeForCurrentZone']);
 const ownCalls=new Set(['setupLeaderBackSideAbilities','getAbilityRegistrar','setupLeaderSideAbilities','setExhaustEnabled','resolveAbilitiesForNewZone']);
 const methods=new Set(['onInitialize','onStartingSide','aspects','title','isDoubleSidedLeader','getPrintedTraits','getAbilityRegistrar','callSetupLeaderWithRegistrar','setupLeaderSideAbilities','setupLeaderBackSideAbilities','flipLeader','initializeForStartZone','getSummary','createActionAbility','createConstantAbility','createTriggeredAbility','createReplacementEffectAbility','initializeForCurrentZone']);
 const unwrap=n=>{while(ts.isParenthesizedExpression(n)||ts.isAsExpression(n)||ts.isTypeAssertionExpression(n))n=n.expression;return n;};
 const expr=(input,scope)=>{
  const n=unwrap(input);
  if(n.kind===ts.SyntaxKind.ThisKeyword)return ref('instance');
  if(n.kind===ts.SyntaxKind.TrueKeyword)return constant(true);
  if(n.kind===ts.SyntaxKind.FalseKeyword)return constant(false);
  if(ts.isIdentifier(n)&&scope.has(n.text))return scope.get(n.text);
  if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){
   if(ts.isIdentifier(n.expression)&&n.expression.text==='ZoneName'&&bound('ZoneName','game/core/Constants')){const value=enums['ZoneName.'+n.name.text];if(value===undefined)reject();return constant(value);}
   const p=expr(n.expression,scope);if(p.kind!=='ref'||!(p.path.join('.')==='instance'&&thisFields.has(n.name.text)||p.path.join('.')==='instance.game'&&n.name.text==='abilityHelper'))reject();return ref(...p.path,n.name.text);
  }
  if(ts.isPrefixUnaryExpression(n)&&n.operator===ts.SyntaxKind.ExclamationToken)return {kind:'unary',op:'!',value:expr(n.operand,scope)};
  if(ts.isConditionalExpression(n))return {kind:'conditional',condition:expr(n.condition,scope),then:expr(n.whenTrue,scope),else:expr(n.whenFalse,scope),evaluation:'chosenBranchOnly'};
  if(ts.isArrowFunction(n)&&n.parameters.length===0&&!n.typeParameters?.length&&!n.modifiers?.length&&!ts.isBlock(n.body))return {kind:'classBoundCallback',parameters:[],lexicalThis:'instance',value:expr(n.body,scope),evaluation:'callbackInvocation'};
  if(ts.isNewExpression(n)&&ts.isIdentifier(n.expression)&&n.expression.text==='Set'&&!imports.has('Set')&&!scope.has('Set')&&!aliases.has('Set')&&n.arguments?.length===1&&!n.typeArguments?.length)return {kind:'copySet',value:expr(n.arguments[0],scope),iteration:'sourceSetOrder'};
  if(ts.isObjectLiteralExpression(n))return {kind:'constructObject',evaluationOrder:'leftToRight',properties:n.properties.map(p=>{if(ts.isSpreadAssignment(p))return {kind:'spreadProperties',value:expr(p.expression,scope)};if(!ts.isPropertyAssignment(p)||!ts.isIdentifier(p.name)||!['onStartingSide','zoneFilter','sourceZoneFilter','condition'].includes(p.name.text))reject();return {kind:'property',name:p.name.text,value:expr(p.initializer,scope)};})};
  if(ts.isCallExpression(n)&&!n.questionDotToken&&!n.typeArguments?.length){
   if(n.expression.kind===ts.SyntaxKind.SuperKeyword)return {kind:'superConstructor',args:n.arguments.map(a=>expr(a,scope))};
   if(ts.isPropertyAccessExpression(n.expression)&&!n.expression.questionDotToken){const c=n.expression,name=c.name.text;
    if(c.expression.kind===ts.SyntaxKind.SuperKeyword&&inheritedMethods.has(name))return {kind:'inheritedMethodCall',method:name,args:n.arguments.map(a=>expr(a,scope)),evaluationOrder:'leftToRight'};
    if(c.expression.kind===ts.SyntaxKind.ThisKeyword&&ownCalls.has(name))return {kind:'classMethodCall',method:name,args:n.arguments.map(a=>expr(a,scope)),evaluationOrder:'leftToRight'};
   }
  }
  reject();
 };
 const body=(block,scope)=>{
  if(!block||block.statements.length>4)reject();const out=[];
  for(const s of block.statements){
   if(ts.isVariableStatement(s)&&s.declarationList.flags&ts.NodeFlags.Const&&s.declarationList.declarations.length===1){const d=s.declarationList.declarations[0];if(!ts.isIdentifier(d.name)||!d.initializer||scope.has(d.name.text))reject();out.push({kind:'bindImmutable',name:d.name.text,value:expr(d.initializer,scope)});scope.set(d.name.text,ref('locals',d.name.text));continue;}
   if(ts.isReturnStatement(s)){out.push(s.expression?{kind:'return',value:expr(s.expression,scope)}:{kind:'returnUndefined'});continue;}
   if(ts.isExpressionStatement(s)){
    const n=unwrap(s.expression);
    if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsToken&&ts.isPropertyAccessExpression(n.left)&&!n.left.questionDotToken&&n.left.expression.kind===ts.SyntaxKind.ThisKeyword&&['setupLeaderBackSide','_onStartingSide','exhausted'].includes(n.left.name.text)){out.push({kind:'writeClassField',field:n.left.name.text,value:expr(n.right,scope)});continue;}
    out.push({kind:'evaluate',value:expr(n,scope)});continue;
   }
   reject();
  }
  return out;
 };
 const inheritance=n=>{
  n=unwrap(n);if(ts.isIdentifier(n)&&n.text==='PlayableOrDeployableCard'&&bound(n.text,'game/core/card/baseClasses/PlayableOrDeployableCard'))return {kind:'frameworkBaseClass',name:n.text};
  if(ts.isCallExpression(n)&&ts.isIdentifier(n.expression)&&n.arguments.length===1&&!n.typeArguments?.length&&!n.questionDotToken){const name=n.expression.text,target=name==='WithLeaderProperties'?'game/core/card/propertyMixins/LeaderProperties':name==='WithAllAbilityTypes'?'game/core/card/propertyMixins/AllAbilityTypeRegistrations':null;if(!target||!bound(name,target))reject();return {kind:'frameworkMixinApplication',name,base:inheritance(n.arguments[0])};}
  reject();
 };
 try{
  if(source.parseDiagnostics.length)reject();let cls=null;
  for(const s of source.statements){
   if(ts.isImportDeclaration(s)){const b=s.importClause?.namedBindings;if(s.importClause?.name&&!s.importClause.isTypeOnly||b&&!ts.isNamedImports(b))reject();for(const e of b?.elements||[]){if(imports.has(e.name.text))reject();imports.set(e.name.text,{name:e.propertyName?.text??e.name.text,module:s.moduleSpecifier.text,typeOnly:!!(s.importClause.isTypeOnly||e.isTypeOnly)});}continue;}
   if(ts.isInterfaceDeclaration(s))continue;
   if(ts.isVariableStatement(s)&&s.declarationList.flags&ts.NodeFlags.Const){for(const d of s.declarationList.declarations){if(!ts.isIdentifier(d.name)||!d.initializer||aliases.has(d.name.text))reject();aliases.set(d.name.text,d.initializer);}continue;}
   if(ts.isClassDeclaration(s)&&!cls){cls=s;continue;}reject();
  }
  if(!cls||cls.name?.text!=='DoubleSidedLeaderCard'||aliases.size!==1)reject();const base=cls.heritageClauses?.[0]?.types?.[0]?.expression;if(!base||!ts.isIdentifier(base)||!aliases.has(base.text)||cls.heritageClauses[0].types.length!==1)reject();
  const decorators=ts.getDecorators(cls)||[];if(decorators.length!==1||!ts.isCallExpression(decorators[0].expression)||decorators[0].expression.arguments.length||!ts.isIdentifier(decorators[0].expression.expression)||decorators[0].expression.expression.text!=='registerStateBase'||!bound('registerStateBase','game/core/GameObjectUtils'))reject();
  const contracts=[],seen=new Set();let constructor=null;const fields=[];
  for(const m of cls.members){
   if(ts.getDecorators(m)?.length||m.modifiers?.some(v=>v.kind===ts.SyntaxKind.AsyncKeyword)||m.asteriskToken||m.questionToken)reject();
   if(ts.isPropertyDeclaration(m)){if(m.name.getText(source)!=='setupLeaderBackSide'||m.initializer?.kind!==ts.SyntaxKind.FalseKeyword||fields.length)reject();fields.push({name:'setupLeaderBackSide',storage:'plainInstanceField',initialValue:constant(false)});continue;}
   if(!ts.isConstructorDeclaration(m)&&!ts.isMethodDeclaration(m)&&!ts.isGetAccessorDeclaration(m))reject();
   const name=ts.isConstructorDeclaration(m)?'constructor':m.name.getText(source);if(seen.has(name)||name!=='constructor'&&!methods.has(name))reject();seen.add(name);
   const scope=new Map(),parameters=[];for(const p of m.parameters){if(!ts.isIdentifier(p.name)||scope.has(p.name.text)||p.dotDotDotToken||p.modifiers?.length||ts.getDecorators(p)?.length)reject();const defaultValue=p.initializer?expr(p.initializer,scope):null;parameters.push({name:p.name.text,optional:!!p.questionToken,...(p.initializer?{defaultValue}:{})});scope.set(p.name.text,ref('parameters',p.name.text));}
   const program=body(m.body,scope);
   if(name==='constructor'){if(constructor)reject();constructor={parameters,body:program};}else contracts.push({kind:'classMethodOverride',name,memberKind:ts.isGetAccessorDeclaration(m)?'getter':'method',parameters,body:program});
  }
  if(!constructor||fields.length!==1||contracts.length!==methods.size||[...methods].some(n=>!seen.has(n)))reject();
  const dependencies=['game/core/card/propertyMixins/LeaderProperties.ts','game/core/card/propertyMixins/AllAbilityTypeRegistrations.ts','game/core/card/baseClasses/PlayableOrDeployableCard.ts'].map(p=>{if(!fs.existsSync(path.join(sourceRoot,p)))reject();return {sourcePath:p,sourceHash:hash(fs.readFileSync(path.join(sourceRoot,p),'utf8'))};});
  return {kind:'finiteClassContract',className:cls.name.text,sourcePath:file,sourceHash:hash(text),stateRegistration:'stateBase',inheritance:inheritance(aliases.get(base.text)),fields,constructor,methods:contracts,dependencies};
 }catch(error){if(error.message==='unsupported-double-sided-leader-contract')return null;throw error;}
}
module.exports={compileDoubleSidedLeaderContract};
