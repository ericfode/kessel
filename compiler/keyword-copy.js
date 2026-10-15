'use strict';
const DOMAIN=['sentinel','overwhelm','saboteur','ambush','grit','shielded','raid','restore','hidden'];
function normalizeKeywordCopy(input,ctx,{ts,unwrap,fail}) {
 const call=unwrap(input);
 if(!ts.isCallExpression(call)||!ts.isPropertyAccessExpression(call.expression)||call.expression.name.text!=='gainKeywords')return null;
 const reject=()=>fail(ctx,call,'keyword-copy-shape');
 if(call.questionDotToken||call.expression.questionDotToken||call.arguments.length!==1||!ts.isPropertyAccessExpression(call.expression.expression)||call.expression.expression.questionDotToken||call.expression.expression.name.text!=='ongoingEffects')reject();
 let setup=call;while(setup&&!ts.isMethodDeclaration(setup))setup=setup.parent;
 const helper=call.expression.expression.expression;
 if(!setup||setup.name.getText(ctx.source)!=='setupCardAbilities'||setup.parameters.length!==2||!ts.isIdentifier(helper)||!ts.isIdentifier(setup.parameters[1].name)||helper.text!==setup.parameters[1].name.text)reject();
 const fn=unwrap(call.arguments[0]);
 const param=p=>ts.isIdentifier(p.name)&&!p.initializer&&!p.questionToken&&!p.dotDotDotToken;
 if(!ts.isArrowFunction(fn)||fn.modifiers?.length||fn.parameters.length!==2||!fn.parameters.every(param)||fn.parameters[0].name.text===fn.parameters[1].name.text||ts.isBlock(fn.body))reject();
 const context=fn.parameters[1].name.text,body=unwrap(fn.body);
 const path=(n,parts)=>parts.length===1?ts.isIdentifier(n)&&n.text===parts[0]:ts.isPropertyAccessExpression(n)&&!n.questionDotToken&&n.name.text===parts.at(-1)&&path(n.expression,parts.slice(0,-1));
 if(!ts.isCallExpression(body)||body.questionDotToken||!path(body.expression,[context,'source','keywords','map'])||body.arguments.length!==1)reject();
 const map=unwrap(body.arguments[0]);
 if(!ts.isArrowFunction(map)||map.modifiers?.length||map.parameters.length!==1||!param(map.parameters[0])||ts.isBlock(map.body))reject();
 const value=unwrap(map.body);
 if(!ts.isCallExpression(value)||value.questionDotToken||value.arguments.length||!path(value.expression,[map.parameters[0].name.text,'toProperties']))reject();
 for(let a=call.parent;a&&a!==setup;a=a.parent)if(a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===helper.text)||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===helper.text)))reject();
 return {kind:'copyKeywords',target:'self',duration:'persistent',domain:DOMAIN};
}
module.exports={normalizeKeywordCopy};
