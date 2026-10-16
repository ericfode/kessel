'use strict';

// Shared typed AsToken overrides used by separately staged native lifecycles.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const ts=require(require.resolve('typescript',{paths:[path.resolve(__dirname,'../forceteki')]}));
const hash=s=>crypto.createHash('sha256').update(s).digest('hex');
const constant=value=>({kind:'const',value});
function compileTokenInheritance(sourceRoot,enums,base){
 if(!['TokenCard','TokenUpgradeCard'].includes(base))return null;
 const reject=()=>{throw new Error('unsupported-token-inheritance');};
 const tokenFile='game/core/card/propertyMixins/Token.ts',tokenText=fs.readFileSync(path.join(sourceRoot,tokenFile),'utf8');
 const source=ts.createSourceFile(tokenFile,tokenText,ts.ScriptTarget.Latest,true),imports=new Map();
 const bound=(name,module)=>{const v=imports.get(name);return v&&!v.typeOnly&&v.name===name&&path.posix.normalize(path.posix.join(path.posix.dirname(tokenFile),v.module))===module;};
 try{
  if(source.parseDiagnostics.length)reject();
  for(const s of source.statements)if(ts.isImportDeclaration(s)&&s.importClause?.namedBindings&&ts.isNamedImports(s.importClause.namedBindings))for(const e of s.importClause.namedBindings.elements){if(imports.has(e.name.text))reject();imports.set(e.name.text,{name:e.propertyName?.text??e.name.text,module:s.moduleSpecifier.text,typeOnly:!!(s.importClause.isTypeOnly||e.isTypeOnly)});}
  const fn=source.statements.find(n=>ts.isFunctionDeclaration(n)&&n.name?.text==='AsToken'),inner=fn?.body?.statements.find(ts.isClassDeclaration);
  if(!fn||!inner||fn.parameters.length!==1||fn.parameters[0].name.getText(source)!=='BaseClass'||fn.parameters[0].initializer||fn.parameters[0].dotDotDotToken||fn.parameters[0].questionToken||inner.heritageClauses?.[0]?.types?.[0]?.expression.getText(source)!=='BaseClass'||fn.body.statements.length!==2||!ts.isReturnStatement(fn.body.statements[1])||fn.body.statements[1].expression?.getText(source)!=='AsToken')reject();
  const decorators=ts.getDecorators(inner)||[];if(decorators.length!==1||!ts.isCallExpression(decorators[0].expression)||decorators[0].expression.arguments.length||!ts.isIdentifier(decorators[0].expression.expression)||decorators[0].expression.expression.text!=='registerStateBase'||!bound('registerStateBase','game/core/GameObjectUtils'))reject();
  const expr=n=>{
   if(n.kind===ts.SyntaxKind.ThisKeyword)return {kind:'ref',path:['source']};
   if(n.kind===ts.SyntaxKind.TrueKeyword)return constant(true);
   if(n.kind===ts.SyntaxKind.FalseKeyword)return constant(false);
   if(n.kind===ts.SyntaxKind.NullKeyword)return constant(null);
   if(ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n))return constant(n.text);
   if(ts.isTemplateExpression(n))return {kind:'textTemplate',parts:[constant(n.head.text),...n.templateSpans.flatMap(s=>[expr(s.expression),constant(s.literal.text)])]};
   if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){
    if(ts.isIdentifier(n.expression)&&n.expression.text==='ZoneName'&&bound('ZoneName','game/core/Constants')){const value=enums['ZoneName.'+n.name.text];if(value===undefined)reject();return constant(value);}
    const p=expr(n.expression);if(p.kind!=='ref'||!['zone','name','internalName'].includes(n.name.text))reject();return {kind:'ref',path:[...p.path,n.name.text]};
   }
   if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsEqualsEqualsToken)return {kind:'binary',op:'===',left:expr(n.left),right:expr(n.right)};
   reject();
  };
  const methods=[];
  for(const m of inner.members){
   if(!ts.isMethodDeclaration(m)||m.parameters.length||m.typeParameters?.length||m.asteriskToken||ts.getDecorators(m)?.length||m.modifiers?.some(v=>v.kind===ts.SyntaxKind.AsyncKeyword)||!m.body||m.body.statements.length>3)reject();
   const name=m.name.getText(source);if(!['removeFromGame','isToken','isPlayable'].includes(name)||methods.some(x=>x.name===name))reject();
   const body=m.body.statements.map(s=>{
    if(ts.isReturnStatement(s)&&s.expression)return {kind:'return',value:expr(s.expression)};
    if(!ts.isExpressionStatement(s))reject();const n=s.expression;
    if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsToken){const target=expr(n.left),value=expr(n.right);if(target.kind!=='ref'||target.path.join('.')!=='source.zone'||value.value!==null)reject();return {kind:'writeTokenState',target,value};}
    if(!ts.isCallExpression(n)||n.questionDotToken||n.typeArguments?.length||!ts.isPropertyAccessExpression(n.expression)||n.expression.questionDotToken)reject();const c=n.expression;
    if(ts.isIdentifier(c.expression)&&c.expression.text==='Contract'&&c.name.text==='assertTrue'&&bound('Contract','game/core/utils/Contract')&&[1,2].includes(n.arguments.length))return {kind:'assert',condition:expr(n.arguments[0]),failure:'throw',...(n.arguments[1]?{message:expr(n.arguments[1])}:{})};
    const receiver=expr(c.expression);if(receiver.kind==='ref'&&receiver.path.join('.')==='source.zone'&&c.name.text==='removeCard'&&n.arguments.length===1&&n.arguments[0].kind===ts.SyntaxKind.ThisKeyword)return {kind:'removeTokenFromCurrentZone'};
    reject();
   });
   if(name==='removeFromGame'?(body.length!==3||body[0].kind!=='assert'||body[1].kind!=='removeTokenFromCurrentZone'||body[2].kind!=='writeTokenState'):(body.length!==1||body[0].kind!=='return'||body[0].value.value!==(name==='isToken')))reject();
   methods.push({name,body});
  }
  if(methods.length!==3)reject();
  const parentFile='game/core/card/TokenCards.ts',parentText=fs.readFileSync(path.join(sourceRoot,parentFile),'utf8'),parentSource=ts.createSourceFile(parentFile,parentText,ts.ScriptTarget.Latest,true);
  const parent=parentSource.statements.find(n=>ts.isClassDeclaration(n)&&n.name?.text===base);
  if(!parent||parent.members.some(m=>!(ts.isPropertyDeclaration(m)&&m.name.getText(parentSource)==='state')&&!(base==='TokenUpgradeCard'&&(ts.isMethodDeclaration(m)&&m.name.getText(parentSource)==='isTokenUpgrade'||ts.isGetAccessorDeclaration(m)&&m.name.getText(parentSource)==='tokenName'))))reject();
  if(base==='TokenUpgradeCard'){const marker=parent.members.find(m=>m.name?.getText(parentSource)==='isTokenUpgrade');if(!marker||marker.parameters.length||marker.body?.statements.length!==1||marker.body.statements[0].expression?.kind!==ts.SyntaxKind.TrueKeyword)reject();methods.push({name:'isTokenUpgrade',body:[{kind:'return',value:constant(true)}]});}
  return {tokenMixin:{name:'AsToken',stateRegistration:'stateBase',methods},dependencies:[{sourcePath:tokenFile,sourceHash:hash(tokenText)},{sourcePath:parentFile,sourceHash:hash(parentText)}]};
 }catch(error){if(error.message==='unsupported-token-inheritance')return null;throw error;}
}
module.exports={compileTokenInheritance};
