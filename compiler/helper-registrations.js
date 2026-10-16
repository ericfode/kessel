'use strict';

// These hooks augment the inherited registrar; they do not replace ordinary
// leader deployment or unit entry systems with a live-board approximation.
function inspect(cls,ctx,api) {
 const {ts,unwrap}=api,helpers=new Map(),members=new Set();
 for(const m of cls.members){
  if(!ts.isMethodDeclaration(m)||!ts.isIdentifier(m.name)||!m.body||m.asteriskToken||m.modifiers?.some(x=>x.kind===ts.SyntaxKind.AsyncKeyword||x.kind===ts.SyntaxKind.StaticKeyword)||ts.getDecorators(m)?.length||m.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken)||cls.members.filter(x=>x.name?.getText(ctx.source)===m.name.text).length!==1)continue;
  const st=m.body.statements,returned=st.length===1&&ts.isReturnStatement(st[0])&&st[0].expression;
  if(ctx.constantRegistrarHelpers?.has(m.name.text)||!m.modifiers?.some(x=>x.kind===ts.SyntaxKind.PrivateKeyword)||st.length!==1||!ts.isExpressionStatement(st[0]))continue;
  const call=unwrap(st[0].expression);
  if(!ts.isCallExpression(call)||call.questionDotToken||call.arguments.length!==1||!ts.isPropertyAccessExpression(call.expression)||call.expression.questionDotToken||!ts.isIdentifier(call.expression.expression)||call.expression.expression.text!==m.parameters[0]?.name.text||!['addActionAbility','addTriggeredAbility','addConstantAbility'].includes(call.expression.name.text)||!ts.isObjectLiteralExpression(unwrap(call.arguments[0])))continue;
  helpers.set(m.name.text,{member:m,registration:call.expression.name.text,input:call.arguments[0]});members.add(m);
 }
 return {helpers,members};
}
function setupCall(call,ctx,setup,api){
 const {ts,unwrap,fail,bindConstant,ability,standardTriggeredAbilities,constantAbility}=api,n=unwrap(call);
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||n.expression.expression.kind!==ts.SyntaxKind.ThisKeyword)return null;
 const h=ctx.helperRegistrations?.helpers.get(n.expression.name.text);if(!h)return null;
 if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==h.member.parameters.length||n.arguments.length<2||!ts.isIdentifier(n.arguments[0])||n.arguments[0].text!==setup.parameters[0]?.name.text||!ts.isIdentifier(n.arguments[1])||n.arguments[1].text!==setup.parameters[1]?.name.text||!h.member.parameters[1].type?.getText(ctx.source).includes('IAbilityHelper'))fail(ctx,n,'setup-helper-capabilities');
 const child={...ctx,constants:new Map(ctx.constants),scope:new Map(ctx.scope),eventHistory:true,pendingRegistration:true};
 for(let i=2;i<n.arguments.length;i++)bindConstant(child,h.member.parameters[i].name.text,n.arguments[i]);
 // A scalar/cost template is built during setup, once per helper invocation.
 const registrations=h.registration==='addTriggeredAbility'?standardTriggeredAbilities(h.input,child):[h.registration==='addConstantAbility'?constantAbility(h.input,child):ability(h.input,child,'action')];
 return registrations.map(registration=>({trigger:'setupHelper',title:registration.title,costs:[],effect:{kind:'setupHelperRegistration',helper:h.member.name.text,evaluation:'orderedArgumentsThenRegistration',registration}}));
}
function setupValidation(statement,ctx,api){const {ts,literal,fail}=api;if(!ts.isExpressionStatement(statement)||!ts.isBinaryExpression(statement.expression))return null;const n=statement.expression;if(n.operatorToken.kind!==ts.SyntaxKind.EqualsToken||!ts.isPropertyAccessExpression(n.left)||n.left.questionDotToken||n.left.expression.kind!==ts.SyntaxKind.ThisKeyword||!['disableWhenDefeatedCheck','disableOnAttackCheck'].includes(n.left.name.text))return null;const value=literal(n.right,ctx);if(typeof value!=='boolean')fail(ctx,n,'setup-validation-boolean');return {kind:'cardTextValidationOverride',property:n.left.name.text,value,applyAt:'orderedSourceSetupStatement',default:false,consumer:'InPlayCard.validateCardAbilities',gameplayEffect:'none'};}
module.exports={inspect,setupCall,setupValidation};
