'use strict';

// One complete source family, not a general variable-count selector. Both
// branches retain the engine's array-selection shape even at a limit of one.
function normalizeInitiativeUpTo(input, context, trigger, api) {
    if (trigger !== 'onAttack') return null;
    const { ts, unwrap, object, literal, displayText, contextTitle, expr, effect } = api;
    const raw = unwrap(input);
    if (!ts.isObjectLiteralExpression(raw)) return null;
    const obj = object(raw, context);
    const keys = (o, expected) => [...o.props.keys()].sort().join(',') === [...expected].sort().join(',');
    if (!keys(obj, ['title', 'contextTitle', 'targetResolver'])) return null;
    const resolverNode = unwrap(obj.props.get('targetResolver'));
    if (!ts.isObjectLiteralExpression(resolverNode)) return null;
    const resolver = object(resolverNode, obj.ctx);
    if (!keys(resolver, ['mode', 'cardTypeFilter', 'zoneFilter', 'cardCondition', 'canChooseNoCards', 'numCardsFunc', 'immediateEffect'])) return null;
    let setup = raw.parent;
    while (setup && !ts.isMethodDeclaration(setup)) setup = setup.parent;
    if (!setup || setup.name.getText(context.source) !== 'setupCardAbilities' || setup.parameters.length !== 2 ||
        setup.parameters.some(p => !ts.isIdentifier(p.name) || p.initializer || p.dotDotDotToken || p.questionToken || p.modifiers?.length) ||
        setup.parameters[0].name.text === setup.parameters[1].name.text || setup.body?.statements.length !== 1) return null;
    const path = node => {
        node = unwrap(node);
        if (ts.isIdentifier(node)) return [node.text];
        if (ts.isPropertyAccessExpression(node) && !node.questionDotToken) {
            const base = path(node.expression);
            return base && [...base, node.name.text];
        }
        return null;
    };
    const isPath = (node, expected) => path(node)?.join('.') === expected.join('.');
    const statement = setup.body.statements[0];
    if (!ts.isExpressionStatement(statement) || !ts.isCallExpression(statement.expression) || statement.expression.questionDotToken ||
        statement.expression.arguments.length !== 1 || statement.expression.arguments[0] !== raw ||
        !isPath(statement.expression.expression, [setup.parameters[0].name.text, 'addOnAttackAbility'])) return null;
    for (const [field, namespace, member] of [['mode', 'TargetMode', 'UpToVariable'], ['cardTypeFilter', 'WildcardCardType', 'Unit'], ['zoneFilter', 'WildcardZoneName', 'AnyArena']]) {
        if (!isPath(resolver.props.get(field), [namespace, member]) || setup.parameters.some(p => p.name.text === namespace) ||
            obj.ctx.scope.has(namespace) || obj.ctx.constants.has(namespace)) return null;
    }
    if (literal(resolver.props.get('canChooseNoCards'), resolver.ctx) !== true) return null;
    const countCallback = unwrap(resolver.props.get('numCardsFunc'));
    const predicateCallback = unwrap(resolver.props.get('cardCondition'));
    const simple = (fn, count) => ts.isArrowFunction(fn) && !fn.modifiers?.length && !fn.asteriskToken && !fn.typeParameters?.length &&
        fn.parameters.length === count && fn.parameters.every(p => ts.isIdentifier(p.name) && !p.initializer && !p.dotDotDotToken && !p.questionToken && !p.modifiers?.length) && !ts.isBlock(fn.body);
    if (!simple(countCallback, 1) || !simple(predicateCallback, 2)) return null;
    const count = expr(countCallback, resolver.ctx);
    const predicate = expr(predicateCallback, resolver.ctx, ['card', 'context']);
    const ref = (...path) => ({ kind: 'ref', path });
    const constant = value => ({ kind: 'const', value });
    const condition = { kind: 'query', name: 'hasInitiative', receiver: ref('player'), args: [] };
    const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
    if (!same(count, { kind: 'conditional', condition, then: constant(2), else: constant(1) }) ||
        !same(predicate, { kind: 'binary', op: '!==', left: ref('card'), right: ref('source') })) return null;
    const helper = setup.parameters[1].name.text;
    const phase = unwrap(resolver.props.get('immediateEffect'));
    if (!ts.isCallExpression(phase) || phase.questionDotToken || phase.arguments.length !== 1 ||
        !isPath(phase.expression, [helper, 'immediateEffects', 'forThisPhaseCardEffect']) || !ts.isObjectLiteralExpression(unwrap(phase.arguments[0]))) return null;
    const phaseProps = object(phase.arguments[0], resolver.ctx);
    if (!keys(phaseProps, ['effect'])) return null;
    const stats = unwrap(phaseProps.props.get('effect'));
    if (!ts.isCallExpression(stats) || stats.questionDotToken || stats.arguments.length !== 1 ||
        !isPath(stats.expression, [helper, 'ongoingEffects', 'modifyStats']) || !ts.isObjectLiteralExpression(unwrap(stats.arguments[0]))) return null;
    const statsProps = object(stats.arguments[0], phaseProps.ctx);
    if (!keys(statsProps, ['power', 'hp']) || literal(statsProps.props.get('power'), statsProps.ctx) !== 2 || literal(statsProps.props.get('hp'), statsProps.ctx) !== 2) return null;
    const compiledEffect = effect(phase, resolver.ctx, 'chosen');
    if (!same(compiledEffect, { kind: 'modifyStats', target: 'chosen', duration: 'phase', power: constant(2), hp: constant(2) })) return null;
    const selected = limit => ({ kind: 'select', targetResolver: { owner: 'any', type: 'unit', zone: 'arena', predicate,
        mode: 'upTo', count: constant(limit), optional: true }, effect: compiledEffect });
    return { trigger, title: displayText(obj.props.get('title'), obj.ctx), costs: [],
        contextTitle: contextTitle(obj.props.get('contextTitle'), obj.ctx),
        effect: { kind: 'conditional', condition, onTrue: selected(2), onFalse: selected(1) } };
}

module.exports = { normalizeInitiativeUpTo };
