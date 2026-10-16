'use strict';

// TextHelper is a closed formatter vocabulary. Keep dynamic strings as values,
// with the same headless formatting contract used by existing literal labels.
function textProgramExpression(input,ctx,api) {
 const {ts,unwrap,expr,fail,unshadowedEnum}=api,n=unwrap(input);
 if(ts.isPropertyAccessExpression(n)&&n.getText(ctx.source).startsWith('TextHelper.')&&api.titleHelper){const value=api.titleHelper(n,ctx);if(value!==null)return {kind:'const',value};}
 if(ts.isTemplateExpression(n))return {kind:'templateString',parts:[{kind:'const',value:n.head.text},...n.templateSpans.flatMap(s=>[expr(s.expression,ctx),{kind:'const',value:s.literal.text}])],evaluationOrder:'leftToRight',conversion:'nativeTemplateSubstitutionToString'};
 if(ctx.pureProgram&&(ts.isPostfixUnaryExpression(n)||ts.isPrefixUnaryExpression(n))&&[ts.SyntaxKind.PlusPlusToken,ts.SyntaxKind.MinusMinusToken].includes(n.operator)){const target=ts.isIdentifier(n.operand)?ctx.scope.get(n.operand.text):null;if(target?.kind!=='ref'||ctx.programLocals?.get(target.path.join('.'))!==true)fail(ctx,n,'program-update-owned-mutable-local');return {kind:'pureLocalUpdate',target,delta:n.operator===ts.SyntaxKind.PlusPlusToken?1:-1,coercion:'ToNumeric',result:ts.isPostfixUnaryExpression(n)?'previousValue':'updatedValue'};}
 if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||!ts.isIdentifier(n.expression.expression)||n.expression.expression.text!=='TextHelper'||!['resource','keyword','trait','aspect'].includes(n.expression.name.text))return null;
 unshadowedEnum(n.expression.expression,ctx,'TextHelper');
 if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==1)fail(ctx,n,'text-program-formatter-arity');
 return {kind:'textFormatter',formatter:n.expression.name.text,value:expr(n.arguments[0],ctx),environment:'test',resourcePlural:'amountNotStrictlyOne',keyword:'capitalizeNameAndOptionalAmountProperty',trait:'titleCaseEachSpaceSeparatedWord',aspect:'capitalize',evaluateAt:'stringExpressionEvaluation'};
}
module.exports={textProgramExpression};
