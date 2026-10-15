'use strict';

const reference = (...path) => ({ kind: 'ref', path });
const constant = value => ({ kind: 'const', value });
const query = (name, receiver, value) => ({ kind: 'query', name, receiver, args: [constant(value)] });

function syntax({ ts, unwrap }) {
    const path = (input, names) => {
        const node = unwrap(input);
        if (names.length === 1) return names[0] === 'this' ? node.kind === ts.SyntaxKind.ThisKeyword : ts.isIdentifier(node) && node.text === names[0];
        return ts.isPropertyAccessExpression(node) && !node.questionDotToken && node.name.text === names.at(-1) && path(node.expression, names.slice(0, -1));
    };
    const plainParameter = p => ts.isIdentifier(p.name) && !p.initializer && !p.dotDotDotToken && !p.questionToken && !p.modifiers?.length;
    const synchronous = n => !n.asteriskToken && !n.typeParameters?.length && !n.modifiers?.some(m => m.kind === ts.SyntaxKind.AsyncKeyword || m.kind === ts.SyntaxKind.DeclareKeyword) && !ts.getDecorators?.(n)?.length;
    const arrow = input => {
        const node = unwrap(input);
        return ts.isArrowFunction(node) && synchronous(node) && node.parameters.length === 1 && plainParameter(node.parameters[0]) && !ts.isBlock(node.body) ? node : null;
    };
    const call = (input, names, arity) => {
        const node = unwrap(input);
        return ts.isCallExpression(node) && !node.questionDotToken && node.arguments.length === arity && path(node.expression, names) ? node : null;
    };
    return { path, plainParameter, synchronous, arrow, call };
}

// The helper is synchronous, has no locals or captures, and reads the complete
// physical attachment list. Preserve it only in the complete ordinary leader
// source topology with one registration on each printed side.
function inspectPhysicalAttachmentHelpers(cls, ctx, api) {
    const { ts, unwrap, literal, unshadowedEnum } = api;
    const { path, plainParameter, synchronous, arrow, call } = syntax(api);
    const empty = () => new Map();
    if (!ts.isClassDeclaration(cls) || !synchronous(cls) || cls.members.length !== 4 ||
        cls.members.some(m => !ts.isMethodDeclaration(m) || !ts.isIdentifier(m.name) || !synchronous(m)) ||
        cls.heritageClauses?.length !== 1 || cls.heritageClauses[0].token !== ts.SyntaxKind.ExtendsKeyword ||
        cls.heritageClauses[0].types.length !== 1 || cls.heritageClauses[0].types[0].typeArguments?.length ||
        !path(cls.heritageClauses[0].types[0].expression, ['LeaderUnitCard'])) return empty();
    const names = cls.members.map(m => m.name.text);
    if (new Set(names).size !== 4) return empty();
    const identity = cls.members.find(m => m.name.text === 'getImplementationId');
    const front = cls.members.find(m => m.name.text === 'setupLeaderSideAbilities');
    const back = cls.members.find(m => m.name.text === 'setupLeaderUnitSideAbilities');
    const helper = cls.members.find(m => m !== identity && m !== front && m !== back);
    if (!identity || !front || !back || !helper || identity.parameters.length ||
        identity.body?.statements.length !== 1 || !ts.isReturnStatement(identity.body.statements[0])) return empty();
    for (const [method, registration] of [[front, 'addActionAbility'], [back, 'addOnAttackAbility']]) {
        if (method.parameters.length !== 2 || !method.parameters.every(plainParameter) ||
            method.parameters[0].name.text === method.parameters[1].name.text || method.body?.statements.length !== 1 ||
            !ts.isExpressionStatement(method.body.statements[0]) ||
            !call(method.body.statements[0].expression, [method.parameters[0].name.text, registration], 1)) return empty();
    }
    // This name is a card-local predicate, not an override of a framework hook
    // such as isUnit or setupCardAbilities. Broader helper names need review.
    if (helper.name.text !== 'isResistanceOrHasResistanceUpgrade' ||
        helper.parameters.length !== 1 || !helper.parameters.every(plainParameter) ||
        helper.parameters[0].name.text === 'Trait' || helper.body?.statements.length !== 1 ||
        !ts.isReturnStatement(helper.body.statements[0])) return empty();
    const parameter = helper.parameters[0].name.text;
    const body = unwrap(helper.body.statements[0].expression);
    if (!ts.isBinaryExpression(body) || body.operatorToken.kind !== ts.SyntaxKind.BarBarToken) return empty();
    const ownTrait = call(body.left, [parameter, 'hasSomeTrait'], 1);
    const some = call(body.right, [parameter, 'upgrades', 'some'], 1);
    if (!ownTrait || !some) return empty();
    const predicate = arrow(some.arguments[0]);
    if (!predicate || predicate.parameters[0].name.text === 'Trait') return empty();
    const attachedTrait = call(predicate.body, [predicate.parameters[0].name.text, 'hasSomeTrait'], 1);
    if (!attachedTrait) return empty();
    for (const arg of [ownTrait.arguments[0], attachedTrait.arguments[0]]) {
        const value = unwrap(arg);
        if (!ts.isPropertyAccessExpression(value) || value.questionDotToken || !path(value.expression, ['Trait'])) return empty();
        unshadowedEnum(value, ctx, 'Trait');
    }
    const trait = literal(ownTrait.arguments[0], ctx);
    if (trait !== 'resistance' || literal(attachedTrait.arguments[0], ctx) !== trait) return empty();
    return new Map([[helper.name.text, { member: helper, trait }]]);
}

function normalizePhysicalAttachmentExpression(input, ctx, api) {
    const { ts, unwrap, expr, literal, unshadowedEnum } = api;
    const { path, arrow, call } = syntax(api);
    const node = unwrap(input);
    if (!ctx.callbackStage || !ts.isCallExpression(node) || node.questionDotToken ||
        !ts.isPropertyAccessExpression(node.expression) || node.expression.questionDotToken) return null;
    const canonicalCard = n => {
        const value = expr(n, ctx);
        return value.kind === 'ref' && value.path.length === 1 && value.path[0] === 'card' ? value : null;
    };
    const helper = ctx.physicalAttachmentHelpers?.get(node.expression.name.text);
    if (helper && path(node.expression.expression, ['this']) && node.arguments.length === 1) {
        for (let ancestor = node.parent; ancestor && !ts.isMethodDeclaration(ancestor); ancestor = ancestor.parent) {
            if (ts.isFunctionExpression(ancestor)) return null;
        }
        const receiver = canonicalCard(node.arguments[0]);
        if (!receiver) return null;
        return { kind: 'binary', op: '||', left: query('hasSomeTrait', receiver, helper.trait), right: query('hasAttachedTrait', receiver, helper.trait) };
    }
    if (node.expression.name.text !== 'some' || node.arguments.length !== 1) return null;
    const collection = unwrap(node.expression.expression);
    if (!ts.isPropertyAccessExpression(collection) || collection.questionDotToken || collection.name.text !== 'upgrades') return null;
    const callback = arrow(node.arguments[0]);
    if (!callback) return null;
    const receiver = canonicalCard(collection.expression);
    if (!receiver) return null;
    const name = callback.parameters[0].name.text;
    if (call(callback.body, [name, 'isShield'], 0)) return query('hasAttachedNativeToken', receiver, 'shield');
    const traitCall = call(callback.body, [name, 'hasSomeTrait'], 1);
    if (!traitCall || name === 'Trait') return null;
    const arg = unwrap(traitCall.arguments[0]);
    if (!ts.isPropertyAccessExpression(arg) || arg.questionDotToken || !path(arg.expression, ['Trait'])) return null;
    unshadowedEnum(arg, ctx, 'Trait');
    const trait = literal(arg, ctx);
    return typeof trait === 'string' && trait.length ? query('hasAttachedTrait', receiver, trait) : null;
}

module.exports = { inspectPhysicalAttachmentHelpers, normalizePhysicalAttachmentExpression };
