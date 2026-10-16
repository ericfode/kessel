'use strict';

// Recognize a complete asserted fold over the supplied selected-card list.
// Imperative source is inspected as syntax, never executed or retained as text.
function inspectSelectedCostHelpers(cls,ctx,{ts,unwrap}) {
    const result=new Map();
    const path=input=>{const n=unwrap(input);if(ts.isIdentifier(n))return [n.text];if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){const p=path(n.expression);return p&&[...p,n.name.text];}return null;};
    const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
    const plain=p=>ts.isIdentifier(p.name)&&!p.initializer&&!p.dotDotDotToken&&!p.questionToken&&!p.modifiers?.length&&!ts.getDecorators(p)?.length;
    const call=(input,p,args)=>{const n=unwrap(input);return ts.isCallExpression(n)&&!n.questionDotToken&&same(path(n.expression),p)&&n.arguments.length===args?n:null;};
    for(const m of cls.members) {
        if(!ts.isMethodDeclaration(m)||!ts.isIdentifier(m.name)||m.name.text!=='costSum'||m.modifiers?.length!==1||m.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||m.asteriskToken||m.questionToken||m.typeParameters?.length||ts.getDecorators(m)?.length||m.parameters.length!==1||!plain(m.parameters[0])||m.body?.statements.length!==3||cls.members.filter(x=>x.name?.getText(ctx.source)===m.name.text).length!==1)continue;
        const [initialize,loop,returned]=m.body.statements;
        if(!ts.isVariableStatement(initialize)||initialize.modifiers?.length||initialize.declarationList.flags!==ts.NodeFlags.Let||initialize.declarationList.declarations.length!==1)continue;
        const total=initialize.declarationList.declarations[0];
        if(!ts.isIdentifier(total.name)||!total.initializer||!ts.isNumericLiteral(unwrap(total.initializer))||Number(unwrap(total.initializer).text)!==0)continue;
        if(!ts.isForOfStatement(loop)||loop.awaitModifier||!ts.isVariableDeclarationList(loop.initializer)||loop.initializer.flags!==ts.NodeFlags.Const||loop.initializer.declarations.length!==1||!ts.isBlock(loop.statement)||loop.statement.statements.length!==2)continue;
        const item=loop.initializer.declarations[0],cards=m.parameters[0].name.text;
        if(!ts.isIdentifier(item.name)||item.initializer||!same(path(loop.expression),[cards])||new Set([cards,total.name.text,item.name.text,'Contract']).size!==4)continue;
        const [assertion,addition]=loop.statement.statements;
        if(!ts.isExpressionStatement(assertion)||!ts.isExpressionStatement(addition))continue;
        const assertCall=call(assertion.expression,['Contract','assertTrue'],1);
        if(!assertCall||!call(assertCall.arguments[0],[item.name.text,'isUnit'],0))continue;
        const sum=unwrap(addition.expression);
        if(!ts.isBinaryExpression(sum)||sum.operatorToken.kind!==ts.SyntaxKind.PlusEqualsToken||!same(path(sum.left),[total.name.text])||!same(path(sum.right),[item.name.text,'cost'])||!ts.isReturnStatement(returned)||!returned.expression||!same(path(returned.expression),[total.name.text]))continue;
        // The assertion is meaningful only with the pinned Contract binding.
        const bindings=[];
        for(const s of ctx.source.statements)if(ts.isImportDeclaration(s)&&s.importClause){
            const c=s.importClause;if(c.name?.text==='Contract')bindings.push(false);
            if(c.namedBindings&&ts.isNamespaceImport(c.namedBindings)&&c.namedBindings.name.text==='Contract')bindings.push(false);
            if(c.namedBindings&&ts.isNamedImports(c.namedBindings))for(const e of c.namedBindings.elements)if(e.name.text==='Contract')bindings.push(!c.isTypeOnly&&!e.isTypeOnly&&(e.propertyName?.text||e.name.text)==='Contract'&&ts.isStringLiteral(s.moduleSpecifier)&&s.moduleSpecifier.text==='../../../core/utils/Contract');
        }
        if(bindings.length!==1||!bindings[0])continue;
        result.set(m.name.text,{member:m,used:false});
    }
    return result;
}
function selectedCostHelperCall(node,ctx,{ts,expr,fail}) {
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
    const helper=ctx.selectedCostHelpers?.get(node.expression.name.text);if(!helper)return null;
    if(!ctx.pendingSelection||!ctx.callbackStage||node.questionDotToken||node.expression.questionDotToken||node.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||node.arguments.length!==1)fail(ctx,node,'selected-cost-helper-call-scope');
    for(let p=node.parent;p;p=p.parent)if(ts.isFunctionExpression(p))fail(ctx,node,'selected-cost-helper-nonlexical-this');
    const cards=expr(node.arguments[0],ctx),ref=(v,name)=>v?.kind==='ref'&&v.path.length===1&&v.path[0]===name;
    if(cards.kind!=='collection'||cards.name!=='concat'||!ref(cards.receiver,'selectedCards')||cards.args.length!==1||!['card','selectionCandidate'].some(name=>ref(cards.args[0],name)))fail(ctx,node,'selected-cost-helper-list-domain');
    helper.used=true;
    return {kind:'assertedCardCostSum',cards,seed:0,itemBinding:'costSumItem',assertion:{kind:'query',name:'isUnit',receiver:{kind:'ref',path:['costSumItem']},args:[]},value:{kind:'ref',path:['costSumItem','cost']},assertionFailure:'throw',order:'suppliedListOrder',limit:256};
}
module.exports={inspectSelectedCostHelpers,selectedCostHelperCall};
