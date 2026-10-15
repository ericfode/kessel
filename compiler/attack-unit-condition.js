'use strict';

// Normalize only the synchronous, live damaged-defender condition. This module
// emits existing scalar IR; it does not introduce attack objects or collections.
function normalizeAttackUnitCondition(node, ctx, { ts, unwrap, expr }) {
    node = unwrap(node);
    if (!ctx.callbackStage || !ts.isBinaryExpression(node) ||
        node.operatorToken.kind !== ts.SyntaxKind.AmpersandAmpersandToken) return null;
    const left = unwrap(node.left), right = unwrap(node.right);
    const property = (value, name, optional = false) =>
        ts.isPropertyAccessExpression(value) && !!value.questionDotToken === optional && value.name.text === name;
    if (!ts.isCallExpression(left) || left.questionDotToken || left.arguments.length ||
        !property(left.expression, 'isAttacking')) return null;
    if (!ts.isCallExpression(right) || right.questionDotToken || right.arguments.length !== 1 ||
        !property(right.expression, 'targetIsUnit', true)) return null;
    const attack = unwrap(right.expression.expression);
    if (!property(attack, 'activeAttack')) return null;
    const callback = unwrap(right.arguments[0]);
    if (!ts.isArrowFunction(callback) || callback.modifiers?.length || callback.asteriskToken ||
        callback.typeParameters?.length || callback.parameters.length !== 1) return null;
    const parameter = callback.parameters[0];
    if (!ts.isIdentifier(parameter.name) || parameter.modifiers?.length || parameter.initializer || parameter.dotDotDotToken || parameter.questionToken) return null;
    const body = unwrap(callback.body);
    if (!ts.isBinaryExpression(body) || body.operatorToken.kind !== ts.SyntaxKind.GreaterThanToken ||
        !property(unwrap(body.left), 'damage') || !ts.isIdentifier(unwrap(body.left).expression) ||
        unwrap(body.left).expression.text !== parameter.name.text ||
        !ts.isNumericLiteral(unwrap(body.right)) || Number(unwrap(body.right).text) !== 0) return null;
    const source = expr(left.expression.expression, ctx);
    const attackSource = expr(attack.expression, ctx);
    const isSource = value => value.kind === 'ref' && value.path.length === 1 && value.path[0] === 'source';
    if (!isSource(source) || !isSource(attackSource)) return null;
    const ref = (...path) => ({ kind: 'ref', path });
    const query = (name, receiver, args = []) => ({ kind: 'query', name, receiver, args });
    const binary = (op, left, right) => ({ kind: 'binary', op, left, right });
    const predicate = () => binary('&&',
        binary('&&', query('isDefending', ref('card')),
            binary('===', ref('card', 'activeAttack', 'attacker'), ref('source'))),
        binary('>', ref('card', 'damage'), { kind: 'const', value: 0 }));
    // Both controller inventories preserve defender identity if control changes.
    // Arena membership supplies targetIsUnit's live unit/in-play checks.
    return binary('&&', query('isAttacking', ref('source')),
        binary('||', query('hasSomeArenaUnit', ref('player'), [predicate()]),
            query('hasSomeArenaUnit', ref('opponent'), [predicate()])));
}

module.exports = { normalizeAttackUnitCondition };
