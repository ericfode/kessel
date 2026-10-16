'use strict';

// A live trait union from Player.getLeaderCards, never a saved outer list.
// Admit only the exact collection pipeline, without arbitrary flatMap callbacks.
function normalizeLeaderTraits(input, ctx, {ts, unwrap, expr, fail}) {
    const node=unwrap(input);
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.name.text!=='flatMap')return null;
    const leaders=unwrap(node.expression.expression);
    if(!ts.isCallExpression(leaders)||!ts.isPropertyAccessExpression(leaders.expression)||leaders.expression.name.text!=='getLeaderCards')return null;
    const reject=()=>fail(ctx,node,'leader-trait-union-shape');
    if(node.questionDotToken||node.expression.questionDotToken||leaders.questionDotToken||leaders.expression.questionDotToken||leaders.arguments.length||node.arguments.length!==1||!ctx.callbackStage)reject();
    const fn=unwrap(node.arguments[0]);
    if(!ts.isArrowFunction(fn)||fn.modifiers?.length||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name)||fn.parameters[0].initializer||fn.parameters[0].dotDotDotToken||fn.parameters[0].questionToken||ts.isBlock(fn.body))reject();
    const body=unwrap(fn.body),name=fn.parameters[0].name.text;
    if(name==='Array'||!ts.isCallExpression(body)||body.questionDotToken||body.arguments.length!==1||!ts.isPropertyAccessExpression(body.expression)||body.expression.questionDotToken||!ts.isIdentifier(body.expression.expression)||body.expression.expression.text!=='Array'||body.expression.name.text!=='from')reject();
    const value=unwrap(body.arguments[0]);
    if(!ts.isPropertyAccessExpression(value)||value.questionDotToken||value.name.text!=='traits'||!ts.isIdentifier(value.expression)||value.expression.text!==name)reject();
    // Array must resolve to the built-in, not an import or setup/callback binding.
    for(const statement of ctx.source.statements)if(ts.isImportDeclaration(statement)&&statement.importClause){const c=statement.importClause;if(c.name?.text==='Array'||c.namedBindings&&(ts.isNamespaceImport(c.namedBindings)?c.namedBindings.name.text==='Array':c.namedBindings.elements.some(e=>e.name.text==='Array')))reject();}
    for(let a=body;a;a=a.parent)if(a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text==='Array')||ts.isBlock(a)&&a.statements.some(s=>ts.isVariableStatement(s)&&s.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text==='Array')))reject();
    const receiver=expr(leaders.expression.expression,ctx);
    if(receiver.kind!=='ref'||receiver.path.length!==1||!['player','opponent'].includes(receiver.path[0]))reject();
    return {kind:'query',name:'leaderTraits',receiver,args:[]};
}
module.exports={normalizeLeaderTraits};
