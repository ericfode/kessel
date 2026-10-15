'use strict';

// Recognize a bounded, pure reduction of the immediately preceding event window.
// This is intentionally not a lowering of general arrays or arbitrary event fields.
function defeatedUnitCountHelper(member, { ts, unwrap }) {
    if (!ts.isMethodDeclaration(member) || member.asteriskToken || member.questionToken || ts.canHaveDecorators(member)&&ts.getDecorators(member)?.length || !ts.isIdentifier(member.name) ||
        member.modifiers?.length !== 1 || member.modifiers[0].kind !== ts.SyntaxKind.PrivateKeyword ||
        member.parameters.length !== 1 || !ts.isIdentifier(member.parameters[0].name) ||
        member.parameters[0].initializer || member.parameters[0].questionToken || member.parameters[0].dotDotDotToken ||
        member.body?.statements.length !== 1) return null;
    const statement = member.body.statements[0];
    if (!ts.isReturnStatement(statement) || !statement.expression) return null;
    const q = unwrap(statement.expression), context = member.parameters[0].name.text;
    const path = n => { n = unwrap(n); if (ts.isIdentifier(n)) return n.text;
        if (ts.isPropertyAccessExpression(n) && !n.questionDotToken) {const p=path(n.expression);return p && p+'.'+n.name.text;} return null; };
    if (!ts.isCallExpression(q) || q.questionDotToken || path(q.expression) !== context+'.events.reduce' ||
        q.arguments.length !== 2 || !ts.isNumericLiteral(q.arguments[1]) || q.arguments[1].text !== '0') return null;
    const callback = unwrap(q.arguments[0]);
    if (!ts.isArrowFunction(callback) || callback.modifiers?.length || callback.parameters.length !== 2 ||
        callback.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.questionToken||p.dotDotDotToken) ||
        !ts.isBlock(callback.body) || callback.body.statements.length !== 2) return null;
    const [sum,event]=callback.parameters.map(p=>p.name.text);
    if (new Set([context,sum,event,'EnumHelpers','EventName']).size !== 5) return null;
    const [conditional, fallback]=callback.body.statements;
    if (!ts.isIfStatement(conditional) || conditional.elseStatement || !ts.isBlock(conditional.thenStatement) ||
        conditional.thenStatement.statements.length !== 1 || !ts.isReturnStatement(fallback) || !fallback.expression || path(fallback.expression)!==sum) return null;
    const success=conditional.thenStatement.statements[0];
    if (!ts.isReturnStatement(success) || !success.expression) return null;
    const addition=unwrap(success.expression);
    if (!ts.isBinaryExpression(addition) || addition.operatorToken.kind!==ts.SyntaxKind.PlusToken ||
        path(addition.left)!==sum || !ts.isNumericLiteral(addition.right) || addition.right.text!=='1') return null;
    // Exact conjunction and enum/helper calls; unlike textual matching this rejects
    // optional calls, extra statements, alternate operators and side effects.
    const terms=[];const split=n=>{n=unwrap(n);if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.AmpersandAmpersandToken){split(n.left);split(n.right);}else terms.push(n);};split(conditional.expression);
    if(terms.length!==4)return null;
    const eq=(n,a,b)=>ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsEqualsEqualsToken&&path(n.left)===a&&path(n.right)===b;
    const unit=terms[3];
    if(!ts.isPrefixUnaryExpression(terms[0])||terms[0].operator!==ts.SyntaxKind.ExclamationToken||path(terms[0].operand)!==event+'.isCancelled'||
        !eq(terms[1],event+'.name','EventName.OnCardDefeated')||!eq(terms[2],event+'.lastKnownInformation.controller',context+'.player.opponent')||
        !ts.isCallExpression(unit)||unit.questionDotToken||path(unit.expression)!=='EnumHelpers.isUnit'||unit.arguments.length!==1||path(unit.arguments[0])!==event+'.lastKnownInformation.type')return null;
    return { member, parameter:context, used:false };
}
function normalizeDefeatedUnitCountCall(node,ctx,{ts,expr,fail}) {
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
    const helper=ctx.defeatedUnitCountHelpers?.get(node.expression.name.text);if(!helper)return null;
    const arg=node.arguments[0];
    if(!ctx.callbackStage||node.questionDotToken||node.expression.questionDotToken||node.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||
        node.arguments.length!==1||!ts.isIdentifier(arg)||ctx.constants.has(arg.text)||ctx.scope.get(arg.text)?.path?.join('.')!=='context')fail(ctx,node,'resolved-defeat-helper-scope');
    for(let a=node.parent;a;a=a.parent)if(ts.isFunctionExpression(a))fail(ctx,node,'resolved-defeat-helper-nonlexical-this');
    helper.used=true;
    return {kind:'query',name:'resolvedDefeatedUnitCount',receiver:{kind:'ref',path:['opponent']},args:[]};
}
module.exports={defeatedUnitCountHelper,normalizeDefeatedUnitCountCall};
