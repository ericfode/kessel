'use strict';

// Keep the triggering attack's complete target list. A list is not the first
// defender, and the event's attack is not the game's later current attack.
function normalizeAttackEventCall(node, ctx, { ts, unwrap, expr, fail }) {
    node = unwrap(node);
    if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression) || node.expression.name.text !== 'getAllTargets') return null;
    const receiver = expr(node.expression.expression, ctx);
    if (receiver.kind !== 'ref' || receiver.path.join('.') !== 'event.attack') return null;
    if (node.questionDotToken || node.expression.questionDotToken || node.arguments.length) fail(ctx,node,'attack-target-list-call');
    return {kind:'query',name:'getAllAttackTargets',receiver,args:[]};
}

module.exports = { normalizeAttackEventCall };
