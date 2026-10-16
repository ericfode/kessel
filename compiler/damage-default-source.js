'use strict';

// DamageSystem defaults the source player to context.player; an explicit card
// source instead uses that card's controller. These coincide for an ordinary
// unit action owned by that source's controller. Only recognize that default in an ordinary
// unit action's executing damage-property factory. Other source fields remain
// visible to the compiler's normal unknown-field rejection.
function normalizeDamageDefaultSource(obj, { ts, unwrap, expr }) {
    if (!obj.props.has('source')) return obj;
    const source = unwrap(obj.props.get('source'));
    if (!ts.isPropertyAccessExpression(source) || source.questionDotToken ||
        source.name.text !== 'source' || !ts.isIdentifier(source.expression)) return obj;

    let factory = source.parent;
    while (factory && !ts.isArrowFunction(factory) && !ts.isFunctionExpression(factory)) factory = factory.parent;
    if (!factory || factory.parameters.length !== 1 ||
        !ts.isIdentifier(factory.parameters[0].name) ||
        factory.parameters[0].name.text !== source.expression.text) return obj;
    const factoryCall = factory.parent;
    if (!factoryCall || !ts.isCallExpression(factoryCall) ||
        factoryCall.arguments.length !== 1 || factoryCall.arguments[0] !== factory ||
        !ts.isPropertyAccessExpression(factoryCall.expression) ||
        factoryCall.expression.name.text !== 'damage') return obj;

    let setup = factoryCall.parent;
    while (setup && !ts.isMethodDeclaration(setup)) setup = setup.parent;
    if (!setup || setup.name.getText(obj.ctx.source) !== 'setupCardAbilities' ||
        setup.parameters.length !== 2 || !ts.isIdentifier(setup.parameters[0].name)) return obj;
    const cls = setup.parent;
    if (!ts.isClassDeclaration(cls) || cls.heritageClauses?.length !== 1 ||
        cls.heritageClauses[0].token !== ts.SyntaxKind.ExtendsKeyword ||
        cls.heritageClauses[0].types.length !== 1 ||
        !ts.isIdentifier(cls.heritageClauses[0].types[0].expression) ||
        cls.heritageClauses[0].types[0].expression.text !== 'NonLeaderUnitCard') return obj;

    let registration = factoryCall.parent;
    while (registration && registration.parent !== setup.body) registration = registration.parent;
    if (!registration || !ts.isExpressionStatement(registration) ||
        !ts.isCallExpression(registration.expression)) return obj;
    const call = registration.expression;
    if (!ts.isPropertyAccessExpression(call.expression) || call.expression.questionDotToken ||
        call.expression.name.text !== 'addActionAbility' ||
        !ts.isIdentifier(call.expression.expression) ||
        call.expression.expression.text !== setup.parameters[0].name.text) return obj;

    // Resolve through the compiler's lexical scope rather than trusting a local
    // parameter's name. A chosen/named/parent source must never disappear.
    const value = expr(source, obj.ctx);
    if (value.kind !== 'ref' || value.path.length !== 1 || value.path[0] !== 'source') return obj;
    const props = new Map(obj.props);
    props.delete('source');
    return { ...obj, props };
}

module.exports = { normalizeDamageDefaultSource };
