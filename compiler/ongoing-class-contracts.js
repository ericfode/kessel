'use strict';
function ongoingClassMethod(member,cls,context,baseClass,api) {
 const {ts,unwrap,fail}=api,name=member.name?.getText(context.source);
 if(baseClass!=='NonLeaderUnitCard'||!['getAbilityRegistrar','isClone'].includes(name))return null;
 if(!ts.isMethodDeclaration(member)||member.parameters.length||member.asteriskToken||member.modifiers?.some(m=>[ts.SyntaxKind.AsyncKeyword,ts.SyntaxKind.StaticKeyword].includes(m.kind))||!member.modifiers?.some(m=>m.kind===ts.SyntaxKind.PublicKeyword)||ts.getDecorators(member)?.length||cls.members.filter(m=>m.name?.getText(context.source)===name).length!==1||member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0])||!member.body.statements[0].expression)fail(context,member,'ongoing-class-method-shape');
 const returned=unwrap(member.body.statements[0].expression);
 if(name==='isClone'){if(returned.kind!==ts.SyntaxKind.TrueKeyword)fail(context,returned,'ongoing-clone-marker');return {kind:'cardClassMethod',name,implementation:'literalReturn',value:true};}
 if(!ts.isCallExpression(returned)||returned.questionDotToken||returned.arguments.length||!ts.isPropertyAccessExpression(returned.expression)||returned.expression.questionDotToken||returned.expression.name.text!==name||returned.expression.expression.kind!==ts.SyntaxKind.SuperKeyword)fail(context,returned,'ongoing-registrar-passthrough');
 return {kind:'cardClassMethod',name,implementation:'superMethodCall',arguments:[],visibility:'public',receiver:'this'};
}
module.exports={ongoingClassMethod};
