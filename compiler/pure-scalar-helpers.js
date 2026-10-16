'use strict';

// Add only single-return scalar/count helpers to the existing typed helper
// program backend. No imperative bodies, arbitrary calls or cyclic definitions.
function inspectPureScalarHelpers(cls,ctx,{ts,unwrap}) {
    const result=new Map(),candidates=new Map(),dependencies=new Map();
    const plain=p=>ts.isIdentifier(p.name)&&!p.initializer&&!p.dotDotDotToken&&!p.questionToken&&!p.modifiers?.length&&!ts.getDecorators(p)?.length;
    const methodName=n=>n.name&&ts.isIdentifier(n.name)?n.name.text:null;
    for(const m of cls.members) {
        const name=methodName(m);
        if(!name||!(/^(get[A-Z]\w*(Amount|Count|Damage)|readyResourcesCount|republicLeaderCount)$/.test(name))||!ts.isMethodDeclaration(m)||m.modifiers?.length!==1||m.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||m.asteriskToken||m.questionToken||m.typeParameters?.length||ts.getDecorators(m)?.length||m.parameters.length!==1||!plain(m.parameters[0])||m.body?.statements.length!==1||!ts.isReturnStatement(m.body.statements[0])||!m.body.statements[0].expression||cls.members.filter(x=>methodName(x)===name).length!==1)continue;
        candidates.set(name,m);
    }
    const fields=new Set(['condition','trait','aspect','keyword','otherThan','arena','type']);
    const queries=new Set(['getArenaUnits','getLeaderCards','filter','hasSomeTrait','isLeaderUnit','isUnit']);
    const valid=(input,env,deps,depth=0)=>{
        if(depth>24)return false;const n=unwrap(input),next=x=>valid(x,env,deps,depth+1);
        if(ts.isNumericLiteral(n)||ts.isStringLiteral(n)||[ts.SyntaxKind.TrueKeyword,ts.SyntaxKind.FalseKeyword,ts.SyntaxKind.NullKeyword].includes(n.kind))return true;
        if(ts.isIdentifier(n))return env.has(n.text);
        if(ts.isPropertyAccessExpression(n))return !n.questionDotToken&&(Object.hasOwn(ctx.enums,n.getText(ctx.source))||next(n.expression));
        if(ts.isBinaryExpression(n))return ['+','-','*','/','%','===','!==','>','>=','<','<=','&&','||','??'].includes(n.operatorToken.getText(ctx.source))&&next(n.left)&&next(n.right);
        if(ts.isConditionalExpression(n))return next(n.condition)&&next(n.whenTrue)&&next(n.whenFalse);
        if(ts.isPrefixUnaryExpression(n))return [ts.SyntaxKind.ExclamationToken,ts.SyntaxKind.MinusToken,ts.SyntaxKind.PlusToken].includes(n.operator)&&next(n.operand);
        if(ts.isArrayLiteralExpression(n))return n.elements.length<=64&&n.elements.every(next);
        if(ts.isObjectLiteralExpression(n))return n.properties.length<=8&&n.properties.every(p=>ts.isPropertyAssignment(p)&&ts.isIdentifier(p.name)&&fields.has(p.name.text)&&next(p.initializer));
        if(ts.isArrowFunction(n)) {
            if(n.modifiers?.length||n.typeParameters?.length||n.parameters.length!==1||!plain(n.parameters[0])||ts.isBlock(n.body))return false;
            return valid(n.body,new Set([...env,n.parameters[0].name.text]),deps,depth+1);
        }
        if(ts.isCallExpression(n)&&!n.questionDotToken&&ts.isPropertyAccessExpression(n.expression)&&!n.expression.questionDotToken) {
            const receiver=n.expression.expression,name=n.expression.name.text;
            if(receiver.kind===ts.SyntaxKind.ThisKeyword&&candidates.has(name)) {if(n.arguments.length!==1)return false;deps.add(name);return n.arguments.every(next);}
            return queries.has(name)&&next(receiver)&&n.arguments.length<=1&&n.arguments.every(next);
        }
        return false;
    };
    for(const [name,m]of candidates) {
        const deps=new Set();if(valid(m.body.statements[0].expression,new Set([m.parameters[0].name.text]),deps)){result.set(name,{member:m,name});dependencies.set(name,deps);}
    }
    const admitted=(name,trail=new Set())=>{
        if(!result.has(name)||trail.has(name))return false;
        const next=new Set([...trail,name]);return [...dependencies.get(name)].every(dep=>admitted(dep,next));
    };
    for(const name of [...result.keys()])if(!admitted(name))result.delete(name);
    return result;
}
function installPureScalarHelpers(cls,ctx,api) {
    if(!ctx.historyHelpers)return;
    const existing=['resultHelpers','maximumHelpers','defeatedUnitCountHelpers','reductionHelpers','arenaDamageHelpers','physicalAttachmentHelpers'];
    for(const [name,helper]of inspectPureScalarHelpers(cls,ctx,api))if(!ctx.historyHelpers.has(name)&&!existing.some(key=>ctx[key]?.has(name)))ctx.historyHelpers.set(name,helper);
}
module.exports={inspectPureScalarHelpers,installPureScalarHelpers};
