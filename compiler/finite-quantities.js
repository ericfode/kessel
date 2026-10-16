'use strict';

// Pure collection projections and quantity factories. These recognize syntax as
// data; no source callback, Set constructor, or Math function is executed.
function syntax(api) {
    const { ts, unwrap } = api;
    const path = input => {
        const n = unwrap(input);
        if (ts.isIdentifier(n)) return [n.text];
        if (ts.isPropertyAccessExpression(n) && !n.questionDotToken) {
            const p = path(n.expression); return p && [...p, n.name.text];
        }
        return null;
    };
    const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
    const call = (n, name) => ts.isCallExpression(n) && !n.questionDotToken &&
        ts.isPropertyAccessExpression(n.expression) && !n.expression.questionDotToken && n.expression.name.text === name;
    const unary = input => {
        const n = unwrap(input);
        if (!ts.isArrowFunction(n) || n.modifiers?.length || n.typeParameters?.length || n.parameters.length !== 1 || ts.isBlock(n.body)) return null;
        const p = n.parameters[0];
        return ts.isIdentifier(p.name) && !p.initializer && !p.dotDotDotToken && !p.questionToken && !p.modifiers?.length && !ts.getDecorators(p)?.length ? n : null;
    };
    return { path, same, call, unary };
}
function unshadowedBuiltin(node, ctx, root, api) {
    api.unshadowedEnum(node, ctx, root);
    if (ctx.source.statements.some(s => api.ts.isImportDeclaration(s) && s.importClause &&
        (s.importClause.name?.text === root || s.importClause.namedBindings?.name?.text === root || s.importClause.namedBindings?.elements?.some(e => e.name.text === root)))) api.fail(ctx, node, 'shadowed-finite-quantity-builtin');
}
function normalizeFiniteQuantity(node, ctx, api) {
    const { ts, unwrap, expr, literal, fail, unshadowedEnum } = api;
    const { path, same, call, unary } = syntax(api);
    const constant = value => ({ kind: 'const', value });
    const reject = () => fail(ctx, node, 'finite-quantity-shape');
    const project = (collection, field, method) => {
        if (!call(collection, method) || collection.arguments.length !== 1) return null;
        const fn = unary(collection.arguments[0]);
        if (!fn || !same(path(fn.body), [fn.parameters[0].name.text, field])) return null;
        return unwrap(collection.expression.expression);
    };
    const arena = base => {
        if (!call(base, 'getArenaUnits') || base.arguments.length) return null;
        const receiver = expr(base.expression.expression, { ...ctx, arenaReceiver: true });
        if (receiver.kind !== 'ref' || !['player', 'opponent'].includes(receiver.path.join('.'))) reject();
        return receiver;
    };
    const checked = result => {
        if (!ctx.callbackStage) fail(ctx, node, 'finite-quantity-evaluation-stage');
        return result;
    };
    if (ts.isPropertyAccessExpression(node) && !node.questionDotToken && node.name.text === 'size') {
        const set = unwrap(node.expression);
        if (!ts.isNewExpression(set) || !ts.isIdentifier(set.expression) || set.expression.text !== 'Set' || set.arguments?.length !== 1 || set.typeArguments?.length) return null;
        unshadowedBuiltin(set, ctx, 'Set', api);
        const collection = unwrap(set.arguments[0]);
        for (const [field, method, name] of [['aspects', 'flatMap', 'arenaDistinctAspectCount'], ['title', 'map', 'arenaDistinctTitleCount']]) {
            const base = project(collection, field, method);
            if (!base) continue;
            const receiver = arena(base); if (!receiver) reject();
            return checked({ kind: 'query', name, receiver, args: [] });
        }
        return null;
    }
    if (ts.isPropertyAccessExpression(node) && !node.questionDotToken && node.name.text === 'length') {
        const filter = unwrap(node.expression);
        if (!call(filter, 'filter') || filter.arguments.length !== 1) return null;
        const base = project(unwrap(filter.expression.expression), 'aspects', 'flatMap');
        if (!base) return null;
        const fn = unary(filter.arguments[0]); if (!fn) reject();
        const test = unwrap(fn.body);
        if (!ts.isBinaryExpression(test) || test.operatorToken.kind !== ts.SyntaxKind.EqualsEqualsEqualsToken || !same(path(test.left), [fn.parameters[0].name.text])) reject();
        const aspectNode = unwrap(test.right);
        if (!ts.isPropertyAccessExpression(aspectNode) || !same(path(aspectNode.expression), ['Aspect']) || aspectNode.questionDotToken) reject();
        unshadowedEnum(aspectNode, ctx, 'Aspect');
        const aspect = literal(aspectNode, ctx);
        if (!['vigilance', 'command', 'aggression', 'cunning', 'heroism', 'villainy'].includes(aspect)) reject();
        let receiver = arena(base), name = 'arenaAspectIconCount', args = [constant(aspect)];
        if (!receiver) {
            if (!call(base, 'getInPlayCards') || base.arguments.length !== 1) reject();
            const filterObject = unwrap(base.arguments[0]);
            if (!ts.isObjectLiteralExpression(filterObject) || filterObject.properties.length !== 1) reject();
            const property = filterObject.properties[0];
            if (!ts.isPropertyAssignment(property) || !ts.isIdentifier(property.name) || property.name.text !== 'type') reject();
            const types = unwrap(property.initializer);
            if (!ts.isArrayLiteralExpression(types) || types.elements.length !== 2 ||
                !same(types.elements.map(path), [['WildcardCardType', 'Unit'], ['WildcardCardType', 'Upgrade']])) reject();
            unshadowedEnum(types, ctx, 'WildcardCardType');
            receiver = expr(base.expression.expression, { ...ctx, arenaReceiver: true });
            if (receiver.kind !== 'ref' || !['player', 'opponent'].includes(receiver.path.join('.'))) reject();
            name = 'inPlayAspectIconCount';
            args = [{ kind: 'array', items: [constant('unit'), constant('upgrade')] }, ...args];
        }
        return checked({ kind: 'query', name, receiver, args });
    }
    return null;
}
function resourceTokenAmount(input, ctx, api) {
    const { ts, unwrap, expr, literal, fail, unshadowedEnum } = api;
    const { path, same, call } = syntax(api);
    const n = unwrap(input);
    if (!call(n, 'floor') || !same(path(n.expression.expression), ['Math']) || n.arguments.length !== 1) return null;
    unshadowedBuiltin(n, ctx, 'Math', api);
    const divided = unwrap(n.arguments[0]);
    if (!ctx.callbackStage || !ts.isBinaryExpression(divided) || divided.operatorToken.kind !== ts.SyntaxKind.SlashToken) fail(ctx, n, 'token-resource-quotient-shape');
    const denominator = literal(divided.right, ctx), count = expr(divided.left, ctx);
    if (!Number.isSafeInteger(denominator) || denominator < 1 || denominator > 64 || count.kind !== 'ref' || !['player.resources.length', 'opponent.resources.length'].includes(count.path.join('.'))) fail(ctx, n, 'token-resource-quotient-shape');
    return { kind: 'math', name: 'floor', args: [{ kind: 'binary', op: '/', left: count, right: { kind: 'const', value: denominator } }] };
}
function perPlayerHandDiscard(obj, api) {
    const { ts, unwrap, expr, literal, fail, unshadowedEnum } = api;
    const { path, same, call, unary } = syntax(api);
    if (!obj.props.has('amount')) return null;
    const fn = unary(obj.props.get('amount')); if (!fn) return null;
    const node = unwrap(fn.body), ctx = obj.ctx;
    const reject = () => fail(ctx, node, 'per-player-hand-discard-shape');
    if (!ctx.callbackStage || !call(node, 'max') || !same(path(node.expression.expression), ['Math']) || node.arguments.length !== 2) reject();
    unshadowedBuiltin(node, ctx, 'Math', api);
    const zero = unwrap(node.arguments[0]), subtract = unwrap(node.arguments[1]);
    if (!ts.isNumericLiteral(zero) || Number(zero.text) !== 0 || !ts.isBinaryExpression(subtract) || subtract.operatorToken.kind !== ts.SyntaxKind.MinusToken || !same(path(subtract.left), [fn.parameters[0].name.text, 'hand', 'length'])) reject();
    const leave = literal(subtract.right, ctx);
    if (!Number.isSafeInteger(leave) || leave < 0 || leave > 64) reject();
    if (!obj.props.has('target')) reject();
    const players = unwrap(obj.props.get('target'));
    if (!call(players, 'getPlayers') || players.arguments.length) reject();
    const receiver = expr(players.expression.expression, { ...ctx, arenaReceiver: true });
    if (receiver.kind !== 'ref' || receiver.path.join('.') !== 'game') reject();
    return {
        kind: 'discardOwnHandPerPlayer',
        target: { kind: 'collection', name: 'players', receiver, args: [] },
        amount: { kind: 'perPlayerAmount', parameter: 'affectedPlayer', value: { kind: 'math', name: 'max', args: [
            { kind: 'const', value: 0 }, { kind: 'binary', op: '-', left: { kind: 'ref', path: ['affectedPlayer', 'hand', 'length'] }, right: { kind: 'const', value: leave } }
        ] } }
    };
}
module.exports = { normalizeFiniteQuantity, resourceTokenAmount, perPlayerHandDiscard };
