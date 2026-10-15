'use strict';

// The pinned Player helper searches every controlled in-play card, including
// the base zone. Its historical name does not impose a unit/leader type filter.
function normalizeControlledTitleCall(input, ctx, { ts, unwrap, expr, fail, unshadowedEnum }) {
    const node = unwrap(input);
    if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression) ||
        !['controlsLeaderUnitOrUpgradeWithTitle','controlsCardWithTrait'].includes(node.expression.name.text)) return null;
    const call = node.expression;
    if (node.questionDotToken || call.questionDotToken) fail(ctx, node, 'optional-controlled-title-query');
    if (!ctx.callbackStage) fail(ctx, node, 'controlled-title-evaluation-stage');
    if(call.name.text==='controlsCardWithTrait') {
        if(node.arguments.length!==3)fail(ctx,node,'controlled-trait-arity');
        const traitNode=unwrap(node.arguments[0]);
        if(ts.isPropertyAccessExpression(traitNode)&&ts.isIdentifier(traitNode.expression))unshadowedEnum(traitNode,ctx,traitNode.expression.text);
        const receiver=expr(call.expression,ctx),trait=expr(node.arguments[0],ctx),unique=expr(node.arguments[1],ctx),excluded=expr(node.arguments[2],ctx);
        if(receiver.kind!=='ref'||receiver.path.length!==1||!['player','opponent'].includes(receiver.path[0])||trait.kind!=='const'||typeof trait.value!=='string'||!trait.value.length||trait.value.length>64||unique.kind!=='const'||unique.value!==false||excluded.kind!=='ref'||excluded.path.join('.')!=='source')fail(ctx,node,'controlled-trait-literal-scope');
        return {kind:'query',name:'hasControlledInPlayTrait',receiver,args:[trait,excluded]};
    }
    if (node.arguments.length !== 1) fail(ctx, node, 'controlled-title-arity');
    const titleNode = unwrap(node.arguments[0]);
    if (!ts.isStringLiteral(titleNode) && !ts.isNoSubstitutionTemplateLiteral(titleNode)) fail(ctx, titleNode, 'controlled-title-literal');
    const title = titleNode.text;
    if (!title.length || Buffer.byteLength(title, 'utf8') > 128) fail(ctx, titleNode, 'controlled-title-bound');
    const receiver = expr(call.expression, ctx);
    if (receiver.kind !== 'ref' || receiver.path.length !== 1 || !['player', 'opponent'].includes(receiver.path[0])) fail(ctx, call.expression, 'controlled-title-player-receiver');
    return { kind: 'query', name: 'hasControlledInPlayTitle', receiver, args: [{ kind: 'const', value: title }] };
}

module.exports = { normalizeControlledTitleCall };
