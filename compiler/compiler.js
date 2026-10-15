'use strict';

const fs = require('fs');
const { normalizeFiniteQuantity, resourceTokenAmount, perPlayerHandDiscard } = require('./finite-quantities');
const {normalizeLivePowerMaximum,hasRawLivePowerMaximum}=require('./live-power-maximum');
const {normalizeBoundedReduction,inspectReductionHelpers,reductionHelperCall,powerMaximumGrant}=require('./bounded-reductions');
const path = require('path');
const crypto = require('crypto');
const { normalizeAttackUnitCondition } = require('./attack-unit-condition');
const { normalizeAttackEventCall } = require('./attack-duration');
const { normalizeDamageDefaultSource } = require('./damage-default-source');
const { normalizeControlledTitleCall } = require('./controlled-title');
const { defeatedUnitCountHelper, normalizeDefeatedUnitCountCall } = require('./resolved-events');
const {normalizeEventResultExpression,inspectResultHelpers,normalizeResultHelperCall,isEventResultAccess}=require('./event-result-context');
const { inspectPhysicalAttachmentHelpers, normalizePhysicalAttachmentExpression } = require('./physical-attachments');
const { inspectMaximumHelpers, maximumHelperCall } = require('./maximum-snapshot');
const { normalizeKeywordCopy } = require('./keyword-copy');
const { normalizeInventoryExpression, pendingInventoryEffect } = require('./inventory-ir');
const { normalizeLeaderTraits } = require('./leader-traits');
const { normalizeInitiativeUpTo } = require('./initiative-up-to');
const ts = require(require.resolve('typescript', { paths: [path.resolve(__dirname, '../forceteki')] }));
const SCHEMA = 'swu-card-ir/v1';
const VERSION = 1;
const hash = (text) => crypto.createHash('sha256').update(text).digest('hex');
const constant = (value) => ({ kind: 'const', value });
const reference = (...parts) => ({ kind: 'ref', path: parts });
const BASIC_EFFECTS = new Set(['damage', 'heal', 'draw', 'ready', 'exhaust', 'defeat', 'giveShield', 'giveExperience', 'returnToHand', 'discardSpecificCard', 'resourceCard', 'readyResources', 'noAction']);
const { bountyAbility, bountyGrant } = require('./bounty-registration');

const TRIGGERS = { addActionAbility: 'action', addWhenPlayedAbility: 'whenPlayed', addOnAttackAbility: 'onAttack', addWhenDefeatedAbility: 'whenDefeated', setEventAbility: 'event', setEpicActionAbility: 'epicAction', addOnDefenseAbility: 'onDefense', addWhenAttackEndsAbility: 'whenAttackEnds' };
const SIDE_METHODS = { setupCardAbilities: 'card', setupLeaderSideAbilities: 'leader', setupLeaderUnitSideAbilities: 'unit', setupLeaderBackSideAbilities: 'unit' };
const QUERY_NAMES = new Set(['isShield', 'isExperience', 'isWeakness', 'isTokenUpgrade', 'canRegisterTriggeredAbilities', 'isUnit', 'isBase', 'isLeader', 'isNonLeaderUnit', 'isUpgrade', 'isToken', 'isEvent', 'hasSomeTrait', 'hasEveryTrait', 'hasSomeAspect', 'hasEveryAspect', 'hasSomeKeyword', 'hasEveryKeyword', 'getPower', 'getHp', 'getCost', 'getPrintedCost', 'hasInitiative', 'getAllUnitsInPlay', 'getUnitsInPlay', 'getCardsInZone', 'getOtherUnitsInPlay', 'getNumericKeywordTotal', 'isInPlay', 'isDeployed', 'isUpgraded', 'canBeDamaged', 'canBeExhausted']);
const DISPLAY_FIELDS = new Set(['effect', 'effectArgs', 'activePromptTitle', 'waitingPromptTitle', 'contextTitle', 'effectMessage', 'ongoingEffectDescription']);

class Unsupported extends Error {
    constructor(ctx, node, pattern, message = pattern) {
        super(message);
        const location = ctx.source.getLineAndCharacterOfPosition(node.getStart(ctx.source));
        this.diagnostic = { code: 'UNSUPPORTED_SOURCE', pattern, message, file: ctx.file, line: location.line + 1, column: location.character + 1 };
    }
}
function fail(ctx, node, pattern, message) { throw new Unsupported(ctx, node, pattern, message); }
function unwrap(node) {
    if (!node) throw new Error('Missing AST node in card definition');
    while (ts.isParenthesizedExpression(node) || ts.isAsExpression(node) || ts.isTypeAssertionExpression(node) || ts.isNonNullExpression(node) || ts.isSatisfiesExpression(node)) node = node.expression;
    return node;
}
function access(node, ctx) {
    node = unwrap(node);
    if (node.questionDotToken) fail(ctx, node, 'optional-reference');
    if (ts.isIdentifier(node)) return [node.text];
    if (node.kind === ts.SyntaxKind.ThisKeyword) return ['this'];
    if (ts.isPropertyAccessExpression(node)) return [...access(node.expression, ctx), node.name.text];
    if (ts.isElementAccessExpression(node)) {
        const parent = access(node.expression, ctx);
        const index = literal(node.argumentExpression, ctx);
        if (!['number', 'string'].includes(typeof index)) fail(ctx, node, 'dynamic-property-index');
        return [...parent, index];
    }
    fail(ctx, node, 'non-path-reference', node.getText(ctx.source));
}
function readConstant(name, ctx) {
    const node = ctx.constants.get(name);
    ctx.usedConstants?.add(node);
    return node;
}
function bindConstant(ctx, name, node) {
    ctx.constants.set(name, node);
    ctx.constantScopes?.set(node, new Map(ctx.scope));
    ctx.constantEnvs?.set(node, new Map(ctx.constants));
    ctx.constantStages?.set(node, ctx.callbackStage || null);
    ctx.bindings?.push({ name, node });
}
function literal(node, ctx) {
    node = unwrap(node);
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return node.text;
    if (ts.isNumericLiteral(node)) return Number(node.text);
    if (ts.isArrayLiteralExpression(node)) return node.elements.map((element) => literal(element, ctx));
    if (ts.isPrefixUnaryExpression(node) && [ts.SyntaxKind.MinusToken, ts.SyntaxKind.PlusToken].includes(node.operator)) {
        const value = literal(node.operand, ctx);
        if (typeof value === 'number') return node.operator === ts.SyntaxKind.MinusToken ? -value : value;
    }
    if (node.kind === ts.SyntaxKind.TrueKeyword) return true;
    if (node.kind === ts.SyntaxKind.FalseKeyword) return false;
    if (node.kind === ts.SyntaxKind.NullKeyword) return null;
    if (ts.isPropertyAccessExpression(node)) {
        const name = node.getText(ctx.source);
        if (Object.hasOwn(ctx.enums, name)) return ctx.enums[name];
    }
    if (ts.isIdentifier(node) && ctx.constants?.has(node.text)) { const initializer=readConstant(node.text,ctx); return literal(initializer,constantContext(initializer,ctx)); }
    fail(ctx, node, 'nonconstant-value', node.getText(ctx.source));
}
function titleHelper(node, ctx) {
    node=unwrap(node);
    const text=node.getText(ctx.source);
    if(ts.isPropertyAccessExpression(node)&&text.startsWith('TextHelper.')) {
        const trait=/^TextHelper\.Trait\.([A-Za-z]+)$/.exec(text);
        if(trait) {
            const value=ctx.enums['Trait.'+trait[1]];
            if(typeof value!=='string')fail(ctx,node,'unsupported-title-helper');
            // Pinned Helpers.titleCase: capitalize each space-separated enum word.
            return value.split(' ').map(word=>word.charAt(0).toUpperCase()+word.slice(1)).join(' ');
        }
        const name=node.name.text;
        if(!/^TextHelper\.[A-Za-z]+$/.test(text)||!['Vigilance','Command','Aggression','Cunning','Villainy','Heroism','Keyword','Keywords','Ambush','Grit','Overwhelm','Saboteur','Sentinel','Shielded','Bounty','Smuggle','Coordinate','Piloting','Hidden','Plot','Support','Fortify'].includes(name))fail(ctx,node,'unsupported-title-helper');
        return name;
    }
    if(ts.isCallExpression(node)&&node.expression.getText(ctx.source).startsWith('TextHelper.')) {
        if(node.expression.getText(ctx.source)==='TextHelper.resource') {
            if(node.arguments.length!==1)fail(ctx,node,'unsupported-title-helper');
            const amount=literal(node.arguments[0],ctx);
            if(!Number.isSafeInteger(amount)||amount<0)fail(ctx,node,'title-helper-amount');
            // Env defaults NODE_ENV to test: preserve the real headless labels.
            return `${amount} resource${amount!==1?'s':''}`;
        }
        if(node.expression.getText(ctx.source)==='TextHelper.keyword') {
            if(node.arguments.length!==1)fail(ctx,node,'unsupported-title-helper');
            const input=unwrap(node.arguments[0]);
            let keyword, amount;
            if(ts.isObjectLiteralExpression(input)) {
                const obj=object(input,ctx);
                for(const key of obj.props.keys())if(!['keyword','amount'].includes(key))fail(ctx,input,'unsupported-title-helper');
                if(!obj.props.has('keyword'))fail(ctx,input,'unsupported-title-helper');
                keyword=literal(obj.props.get('keyword'),obj.ctx);
                if(obj.props.has('amount')) {
                    amount=literal(obj.props.get('amount'),obj.ctx);
                    if(!Number.isSafeInteger(amount)||amount<0)fail(ctx,input,'title-helper-amount');
                }
            } else keyword=literal(input,ctx);
            const keywords=Object.entries(ctx.enums).filter(([key])=>key.startsWith('KeywordName.')).map(([,value])=>value);
            if(typeof keyword!=='string'||!keywords.includes(keyword))fail(ctx,input,'unsupported-title-helper');
            return keyword.charAt(0).toUpperCase()+keyword.slice(1)+(amount===undefined?'':` ${amount}`);
        }
        if(node.expression.getText(ctx.source)==='TextHelper.aspectList') {
            if(node.arguments.length<1||node.arguments.length>2)fail(ctx,node,'unsupported-title-helper');
            const values=literal(node.arguments[0],ctx), conjunction=node.arguments.length===2?literal(node.arguments[1],ctx):null;
            if(!Array.isArray(values)||values.length>8||values.some(v=>!['vigilance','command','aggression','cunning','villainy','heroism'].includes(v))||![null,'and','or'].includes(conjunction))fail(ctx,node,'unsupported-title-helper');
            const labels=values.map(v=>v.charAt(0).toUpperCase()+v.slice(1));
            if(!conjunction)return labels.join(', ');
            return labels.map((v,i)=>labels.length>1&&i===labels.length-1?conjunction+' '+v:v).join(labels.length>2?', ':' ');
        }
        const numeric=/^TextHelper\.(Raid|Restore|Exploit)$/.exec(node.expression.getText(ctx.source));
        if(!numeric||node.arguments.length!==1)fail(ctx,node,'unsupported-title-helper');
        const amount=literal(node.arguments[0],ctx);
        if(!Number.isSafeInteger(amount)||amount<0)fail(ctx,node,'title-helper-amount');
        return numeric[1]+' '+amount;
    }
    return null;
}
// Titles are display text, never executable rules. Dynamic title fragments are
// rendered as placeholders; their expressions are deliberately never evaluated.
function displayText(node, ctx) {
    node = unwrap(node);
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return node.text;
    if (ts.isTemplateExpression(node)) return node.head.text + node.templateSpans.map((span) => {
        const helper=titleHelper(span.expression,ctx);
        return (helper ?? '[value]')+span.literal.text;
    }).join('');
    return 'Resolve ability';
}
// Unlike cosmetic effect text, contextTitle is part of the legal action protocol.
// Its expressions remain pure IR and are evaluated in the current ability context.
function contextTitle(input, context) {
    const {node,ctx}=lambda(input,context);
    const part=(n)=>{
        n=unwrap(n);
        if(ts.isPropertyAccessExpression(n)&&n.questionDotToken) {
            // Only nullable attached-card title access is supported here.
            const receiver=expr(n.expression,ctx);
            if(n.name.text!=='title'||receiver.kind!=='ref'||receiver.path.join('.')!=='source.parentUnit')fail(ctx,n,'unsupported-context-title-optional');
            return reference(...receiver.path,'title');
        }
        if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.QuestionQuestionToken)return {kind:'binary',op:'??',left:part(n.left),right:part(n.right)};
        const helper=titleHelper(n,ctx);
        if(helper!==null)return constant(helper);
        return expr(n,ctx);
    };
    const fragments=ts.isTemplateExpression(node)?[constant(node.head.text),...node.templateSpans.flatMap(span=>[part(span.expression),constant(span.literal.text)])]:[part(node)];
    let literalBytes=0; const count=(v)=>{if(!v||typeof v!=='object')return;if(v.kind==='const'&&typeof v.value==='string')literalBytes+=Buffer.byteLength(v.value);Object.values(v).forEach(count);};count(fragments);
    if(fragments.length>32||literalBytes>1024)fail(ctx,node,'context-title-bound');
    return fragments;
}
function isNonSynchronous(node) {
    return !!node.asteriskToken || !!node.modifiers?.some(m => m.kind === ts.SyntaxKind.AsyncKeyword);
}
function hasDecorators(node) {
    return ts.canHaveDecorators(node) && !!ts.getDecorators(node)?.length;
}
function unsupportedParameterBinding(parameter) {
    return !!parameter.dotDotDotToken || !!parameter.initializer || hasDecorators(parameter);
}
function lambda(node, ctx, names = ['context']) {
    node = unwrap(node);
    if (!ts.isArrowFunction(node) && !ts.isFunctionExpression(node)) return { node, ctx };
    if (isNonSynchronous(node)) fail(ctx,node,'non-synchronous-callback');
    const scope = new Map(ctx.scope);
    const constants = new Map(ctx.constants);
    node.parameters.forEach((p, i) => {
        if (!ts.isIdentifier(p.name)) fail(ctx, p, 'destructured-callback-parameter');
        if (unsupportedParameterBinding(p)) fail(ctx,p,'unsupported-callback-parameter-binding');
        scope.set(p.name.text, reference(names[i] || `arg${i}`));
        constants.delete(p.name.text);
    });
    // A nested callback can bind a new candidate to the same IR root. An
    // outer candidate's active-attack guard must not guard that new candidate.
    const attackDefenders = new Set(ctx.attackDefenders || []);
    node.parameters.forEach((_,i)=>attackDefenders.delete(names[i] || `arg${i}`));
    const arenaLeaderCandidate = !!ctx.syncArenaPredicate && node.parameters.length === 1;
    const child = { ...ctx, scope, constants, attackDefenders, callbackStage: ctx.syncArenaPredicate ? ctx.callbackStage : node, syncArenaPredicate: false, arenaLeaderCandidate };
    if (!ts.isBlock(node.body)) return { node: unwrap(node.body), ctx: child };
    let returned = null;
    for (const statement of node.body.statements) {
        if (returned) fail(child, statement, 'statement-after-callback-return');
        if (ts.isVariableStatement(statement)) {
            if (!(statement.declarationList.flags & ts.NodeFlags.Const)) fail(child, statement, 'mutable-callback-local');
            for (const d of statement.declarationList.declarations) {
                if (!ts.isIdentifier(d.name) || !d.initializer) fail(child, d, 'complex-callback-binding');
                bindConstant(child, d.name.text, d.initializer);
            }
        } else if (ts.isReturnStatement(statement) && statement.expression && !returned) returned = statement.expression;
        else fail(child, statement, 'callback-statement:' + ts.SyntaxKind[statement.kind]);
    }
    if (!returned) fail(child, node, 'callback-without-return');
    return { node: unwrap(returned), ctx: child };
}
function normalizeRef(parts) {
    if (parts[0] === 'context') parts = parts.slice(1);
    if (parts[0] === 'player' && parts[1] === 'opponent') parts = ['opponent', ...parts.slice(2)];
    if (parts[0] === 'events' && parts[1] === 0) parts = ['event', ...parts.slice(2)];
    if (parts[0] === 'this') parts = ['source', ...parts.slice(1)];
    return parts;
}
// Readonly is about assignment, not evaluation time. An arena query initializer
// holds a snapshot; replacing it with a live query in a later callback is wrong.
function capturesArenaSnapshot(node, ctx, seen = new Set(), controllerOnly = false) {
    node = unwrap(node);
    if (ts.isArrowFunction(node) || ts.isFunctionExpression(node)) return false;
    if (!controllerOnly && ts.isPropertyAccessExpression(node) && node.name.text==='upgrades') return true;
    if (controllerOnly && ts.isPropertyAccessExpression(node) && node.name.text === 'controller') return true;
    if (!controllerOnly && ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && (['getArenaUnits','getInPlayCards','getPlayers','getTopCardOfDeck','hasSomeArenaCard','hasSomeArenaUpgrade','hasSomeArenaUnit','isTraitInPlay','isAspectInPlay','isKeywordInPlay','controlsCardWithTrait','controlsLeaderUnitOrUpgradeWithTitle','getLeaderCards','getConstantAbilities','getActionAbilities','getTriggeredAbilities'].includes(node.expression.name.text) || ctx.arenaDamageHelpers?.has(node.expression.name.text) || ctx.reductionHelpers?.has(node.expression.name.text))) return true;
    if (ts.isIdentifier(node) && ctx.constants.has(node.text) && !(ts.isPropertyAccessExpression(node.parent) && node.parent.name === node)) {
        const initializer = ctx.constants.get(node.text);
        if (seen.has(initializer)) return false; // Normal alias resolution diagnoses cycles.
        const next = new Set(seen); next.add(initializer);
        return capturesArenaSnapshot(initializer, { ...ctx, constants: ctx.constantEnvs?.get(initializer) || ctx.constants }, next, controllerOnly);
    }
    let captured = false;
    ts.forEachChild(node, child => { if (capturesArenaSnapshot(child, ctx, seen, controllerOnly)) captured = true; });
    return captured;
}
function constantContext(node, ctx) {
    const trail = new Set(ctx.constantTrail || []);
    if (trail.has(node)) fail(ctx, node, 'cyclic-readonly-binding');
    trail.add(node);
    const lexicalConstants = ctx.constantEnvs?.get(node) || ctx.constants;
    if (capturesArenaSnapshot(node, { ...ctx, constants: lexicalConstants }) && (!ctx.callbackStage || ctx.constantStages?.get(node) !== ctx.callbackStage)) fail(ctx, node, 'arena-query-snapshot-capture');
    if (ctx.arenaReceiver && capturesArenaSnapshot(node, { ...ctx, constants: lexicalConstants }, new Set(), true) && (!ctx.callbackStage || ctx.constantStages?.get(node) !== ctx.callbackStage)) fail(ctx, node, 'arena-controller-snapshot-capture');
    return { ...ctx, scope: ctx.constantScopes?.get(node) || ctx.scope, constants: ctx.constantEnvs?.get(node) || ctx.constants, constantTrail: trail };
}
// A private numeric helper is accepted only as a complete, pure program.
// The bounded namespace has no pinned EventCard hooks; body and call binding
// checks establish semantics rather than trusting its method name.
function arenaDamageHelper(member) {
    const pathOf=n=>{n=unwrap(n);if(ts.isIdentifier(n))return [n.text];if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){const p=pathOf(n.expression);return p&&[...p,n.name.text];}return null;};
    if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||!/^get[A-Z]\w*FromContext$/.test(member.name.text)||isNonSynchronous(member)||hasDecorators(member)||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||member.parameters.length!==1)return null;
    const parameter=member.parameters[0];if(!ts.isIdentifier(parameter.name)||unsupportedParameterBinding(parameter))return null;
    const statements=member.body?.statements;if(statements?.length!==3)return null;
    const declaration=s=>ts.isVariableStatement(s)&&(s.declarationList.flags&ts.NodeFlags.Const)&&s.declarationList.declarations.length===1&&ts.isIdentifier(s.declarationList.declarations[0].name)?s.declarationList.declarations[0]:null;
    const arenaName=declaration(statements[0]),arena=declaration(statements[1]);if(!arenaName?.initializer||!arena?.initializer||!ts.isReturnStatement(statements[2])||!statements[2].expression)return null;
    if(new Set([parameter.name.text,arenaName.name.text,arena.name.text]).size!==3)return null;
    if(JSON.stringify(pathOf(arenaName.initializer))!==JSON.stringify([parameter.name.text,'target','zoneName']))return null;
    const query=unwrap(arena.initializer);if(!ts.isCallExpression(query)||query.questionDotToken||JSON.stringify(pathOf(query.expression))!==JSON.stringify([parameter.name.text,'player','getArenaUnits'])||query.arguments.length!==1)return null;
    const filter=unwrap(query.arguments[0]);if(!ts.isObjectLiteralExpression(filter)||filter.properties.length!==1)return null;
    const p=filter.properties[0];if(!ts.isPropertyAssignment(p)||!ts.isIdentifier(p.name)||p.name.text!=='arena'||!ts.isIdentifier(unwrap(p.initializer))||unwrap(p.initializer).text!==arenaName.name.text)return null;
    const returned=unwrap(statements[2].expression);if(JSON.stringify(pathOf(returned))===JSON.stringify([arena.name.text,'length']))return {member,parameter,arenaName,arena,used:false,multiplier:1};
    if(!ts.isBinaryExpression(returned)||returned.operatorToken.kind!==ts.SyntaxKind.AsteriskToken||JSON.stringify(pathOf(returned.left))!==JSON.stringify([arena.name.text,'length'])||!ts.isNumericLiteral(unwrap(returned.right))||Number(unwrap(returned.right).text)!==2)return null;
    return {member,parameter,arenaName,arena,used:false,multiplier:2};
}
function spaceCountComparisonHelper(member) {
    if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||!/^controls[A-Z]\w*$/.test(member.name.text)||isNonSynchronous(member)||hasDecorators(member)||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||member.parameters.length!==1)return null;
    const parameter=member.parameters[0];if(!ts.isIdentifier(parameter.name)||parameter.name.text==='ZoneName'||unsupportedParameterBinding(parameter)||member.body?.statements.length!==1)return null;
    const statement=member.body.statements[0];if(!ts.isReturnStatement(statement)||!statement.expression)return null;
    const comparison=unwrap(statement.expression);if(!ts.isBinaryExpression(comparison)||comparison.operatorToken.kind!==ts.SyntaxKind.LessThanToken)return null;
    const count=(n,opponent)=>{n=unwrap(n);if(!ts.isPropertyAccessExpression(n)||n.questionDotToken||n.name.text!=='length')return false;const q=unwrap(n.expression);if(!ts.isCallExpression(q)||q.questionDotToken||!ts.isPropertyAccessExpression(q.expression)||q.expression.questionDotToken||q.expression.name.text!=='getArenaUnits'||q.arguments.length!==1)return false;const owner=q.expression.expression.getText().replace(/\s/g,'');if(owner!==parameter.name.text+'.player'+(opponent?'.opponent':''))return false;const filter=unwrap(q.arguments[0]);if(!ts.isObjectLiteralExpression(filter)||filter.properties.length!==1)return false;const p=filter.properties[0];return ts.isPropertyAssignment(p)&&ts.isIdentifier(p.name)&&p.name.text==='arena'&&ts.isPropertyAccessExpression(p.initializer)&&!p.initializer.questionDotToken&&ts.isIdentifier(p.initializer.expression)&&p.initializer.expression.text==='ZoneName'&&p.initializer.name.text==='SpaceArena';};
    if(!count(comparison.left,false)||!count(comparison.right,true))return null;
    return {member,parameter,spaceComparison:comparison,used:false};
}
function filteredArenaCountHelper(member) {
    if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||!/^(get[A-Z]\w*Amount|opponentHas[A-Z]\w*)$/.test(member.name.text)||isNonSynchronous(member)||hasDecorators(member)||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||member.parameters.length!==1)return null;
    const parameter=member.parameters[0];if(!ts.isIdentifier(parameter.name)||unsupportedParameterBinding(parameter))return null;
    const statements=member.body?.statements;if(!statements||![1,2].includes(statements.length))return null;
    let list,comparison=false;
    if(statements.length===1){const r=statements[0];if(!ts.isReturnStatement(r)||!r.expression)return null;const length=unwrap(r.expression);if(!ts.isPropertyAccessExpression(length)||length.questionDotToken||length.name.text!=='length')return null;list=unwrap(length.expression);}
    else {const d=statements[0],r=statements[1];if(!ts.isVariableStatement(d)||!(d.declarationList.flags&ts.NodeFlags.Const)||d.declarationList.declarations.length!==1||!ts.isReturnStatement(r)||!r.expression)return null;const local=d.declarationList.declarations[0];if(!ts.isIdentifier(local.name)||local.name.text===parameter.name.text||!local.initializer)return null;const b=unwrap(r.expression);if(!ts.isBinaryExpression(b)||b.operatorToken.kind!==ts.SyntaxKind.GreaterThanEqualsToken||!ts.isNumericLiteral(b.right)||Number(b.right.text)!==3||!ts.isPropertyAccessExpression(b.left)||b.left.questionDotToken||!ts.isIdentifier(b.left.expression)||b.left.expression.text!==local.name.text||b.left.name.text!=='length')return null;list=unwrap(local.initializer);comparison=true;}
    if(!ts.isCallExpression(list)||list.questionDotToken||!ts.isPropertyAccessExpression(list.expression)||list.expression.questionDotToken||list.expression.name.text!=='filter'||list.arguments.length!==1)return null;
    const query=unwrap(list.expression.expression),filter=unwrap(list.arguments[0]);
    if(!ts.isCallExpression(query)||query.questionDotToken||!ts.isPropertyAccessExpression(query.expression)||query.expression.questionDotToken||query.expression.name.text!=='getArenaUnits'||query.arguments.length!==0||!ts.isArrowFunction(filter)||isNonSynchronous(filter)||hasDecorators(filter)||filter.parameters.length!==1||!ts.isIdentifier(filter.parameters[0].name)||unsupportedParameterBinding(filter.parameters[0]))return null;
    const owner=query.expression.expression.getText().replace(/\s/g,'');if(owner!==parameter.name.text+'.player'+(comparison?'.opponent':''))return null;
    const candidate=filter.parameters[0].name.text,body=unwrap(filter.body);
    const property=(n,name)=>ts.isPropertyAccessExpression(n)&&!n.questionDotToken&&ts.isIdentifier(n.expression)&&n.expression.text===candidate&&n.name.text===name;
    if(!comparison){if(!ts.isBinaryExpression(body)||body.operatorToken.kind!==ts.SyntaxKind.GreaterThanEqualsToken||!property(body.left,'remainingHp')||!ts.isNumericLiteral(body.right)||Number(body.right.text)!==7)return null;}
    else {if(!ts.isBinaryExpression(body)||body.operatorToken.kind!==ts.SyntaxKind.AmpersandAmpersandToken||!ts.isCallExpression(body.left)||body.left.questionDotToken||body.left.arguments.length!==0||!property(body.left.expression,'isUnit')||!ts.isBinaryExpression(body.right)||body.right.operatorToken.kind!==ts.SyntaxKind.EqualsEqualsEqualsToken||!property(body.right.left,'exhausted')||body.right.right.kind!==ts.SyntaxKind.TrueKeyword)return null;}
    return {member,parameter,filteredCount:{query,filter,comparison},used:false};
}
function arenaDamageHelperCall(node,ctx,helper) {
    const call=node.expression,arg=node.arguments[0];
    // Only arrows retain the card receiver through property-factory callbacks.
    for(let ancestor=node.parent;ancestor;ancestor=ancestor.parent)if(ts.isFunctionExpression(ancestor))fail(ctx,node,'arena-damage-helper-nonlexical-this');
    if(!ctx.callbackStage||(!helper.spaceComparison&&!helper.filteredCount&&!ctx.damageRecipientScope)||node.questionDotToken||call.questionDotToken||call.expression.kind!==ts.SyntaxKind.ThisKeyword||node.arguments.length!==1||!ts.isIdentifier(arg)||ctx.constants.has(arg.text)||ctx.scope.get(arg.text)?.path?.join('.')!=='context')fail(ctx,node,'arena-damage-helper-call-scope');
    const child={...ctx,scope:new Map(),constants:new Map()};
    child.scope.set(helper.parameter.name.text,reference('context'));
    if(helper.spaceComparison){helper.used=true;return expr(helper.spaceComparison,child);}
    if(helper.filteredCount){
        const {query,filter,comparison}=helper.filteredCount;
        const predicate=expr(filter,{...child,syncArenaPredicate:true},['card']);arenaPredicate(predicate,child,filter);
        const count=arenaQuery(query,child,'arenaUnitCount');count.args[0]={kind:'binary',op:'&&',left:count.args[0],right:predicate};helper.used=true;
        return comparison?{kind:'binary',op:'>=',left:count,right:constant(3)}:count;
    }
    bindConstant(child,helper.arenaName.name.text,helper.arenaName.initializer);bindConstant(child,helper.arena.name.text,helper.arena.initializer);
    const query=readConstant(helper.arena.name.text,child);helper.used=true;
    const count=arenaQuery(query,constantContext(query,child),'arenaUnitCount');
    return helper.multiplier===1?count:{kind:'binary',op:'*',left:count,right:constant(2)};
}
const isArenaList = (value) => ['arenaList', 'arenaFilter'].includes(value.kind);
function arenaCapture(node, ctx, bound = new Set(), seen = new Set()) {
    if (ts.isArrowFunction(node) || ts.isFunctionExpression(node)) {
        const local = new Set(bound);
        for (const parameter of node.parameters) if (ts.isIdentifier(parameter.name)) local.add(parameter.name.text);
        arenaCapture(node.body, ctx, local, seen);
        return;
    }
    if (ts.isIdentifier(node) && !bound.has(node.text) && !(ts.isPropertyAccessExpression(node.parent) && node.parent.name === node)) {
        if (ctx.scope.get(node.text)?.path?.[0] === 'card') fail(ctx, node, 'arena-query-card-capture');
        if (ctx.constants.has(node.text)) {
            const initializer = readConstant(node.text, ctx);
            if (seen.has(initializer)) fail(ctx, node, 'cyclic-readonly-binding');
            const next = new Set(seen); next.add(initializer);
            arenaCapture(initializer, { ...ctx, scope: ctx.constantScopes?.get(initializer) || ctx.scope, constants: ctx.constantEnvs?.get(initializer) || ctx.constants }, bound, next);
            return;
        }
    }
    ts.forEachChild(node, (child) => arenaCapture(child, ctx, bound, seen));
}
// List predicates use one candidate register. Nested list algebra, arbitrary
// collections and array-selected outer targets need additional typed registers.
function arenaPredicate(value, ctx, node, depth = 0) {
    if (depth > 24) fail(ctx, node, 'arena-predicate-bound');
    if (value.kind === 'ref') {
        const [root, ...parts] = value.path;
        const cardFields = ['power','hp','damage','remainingHp','cost','printedCost','exhausted','unique','zoneName','controller','title'];
        const validCard = ['card','source','target'].includes(root) && (parts.length === 0 || (parts.length === 1 && cardFields.includes(parts[0])));
        const validPlayer = ['player','opponent'].includes(root) && (parts.length === 0 || ['base.damage','hand.length','resources.length','readyResourceCount','exhaustedResourceCount'].includes(parts.join('.')));
        if (!validCard && !validPlayer) fail(ctx, node, 'arena-predicate-reference');
        return;
    }
    if (!['const','binary','unary','conditional','query','math','array'].includes(value.kind)) fail(ctx, node, 'arena-predicate-expression');
    if (value.kind === 'query' && !['isUnit','isBase','isLeader','isNonLeaderUnit','isUpgrade','isToken','isTokenUnit','isEvent','hasSomeTrait','hasEveryTrait','hasSomeAspect','hasEveryAspect','hasSomeKeyword','hasEveryKeyword','getPower','getHp','getCost','getPrintedCost','hasInitiative','getNumericKeywordTotal','hasAttachedTrait','hasAttachedNativeToken','isInPlay','isDeployed','isUpgraded','canBeDamaged','canBeExhausted','hasCost'].includes(value.name)) fail(ctx, node, 'arena-predicate-query');
    for (const child of Object.values(value)) {
        if (Array.isArray(child)) { for (const item of child) if (item && typeof item === 'object') arenaPredicate(item, ctx, node, depth + 1); }
        else if (child && typeof child === 'object') arenaPredicate(child, ctx, node, depth + 1);
    }
}
function arenaListNode(node, ctx, depth = 0) {
    node = unwrap(node);
    if (depth > 24) fail(ctx, node, 'arena-list-bound');
    if (ts.isIdentifier(node) && ctx.constants.has(node.text)) {
        const initializer = readConstant(node.text, ctx);
        return arenaListNode(initializer, constantContext(initializer, ctx), depth + 1);
    }
    if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression)) return null;
    const name = node.expression.name.text;
    if (name === 'getArenaUnits') {
        if (node.questionDotToken || node.expression.questionDotToken) fail(ctx, node, 'optional-arena-list');
        const query = arenaQuery(node, ctx, 'arenaList');
        arenaPredicate(query.args[0], ctx, node);
        return { kind: 'arenaList', receiver: query.receiver, predicate: query.args[0] };
    }
    if (name === 'filter') {
        const receiver = arenaListNode(node.expression.expression, ctx, depth + 1);
        if (!receiver) return null;
        if (node.questionDotToken || node.expression.questionDotToken) fail(ctx, node, 'optional-arena-list');
        if (node.arguments.length !== 1) fail(ctx, node, 'arena-filter-arity');
        const callback = unwrap(node.arguments[0]);
        if ((!ts.isArrowFunction(callback) && !ts.isFunctionExpression(callback)) || callback.parameters.length !== 1) fail(ctx, callback, 'arena-predicate-shape');
        arenaCapture(callback, ctx);
        const predicate = expr(callback, { ...ctx, allowArenaList: false, syncArenaPredicate: true }, ['card']);
        arenaPredicate(predicate, ctx, callback);
        return { kind: 'arenaFilter', receiver, predicate };
    }
    return null;
}
function arenaQuery(node, ctx, name, filterProps = null) {
    if (!ctx.callbackStage) fail(ctx, node, 'arena-query-evaluation-stage');
    if (!filterProps && node.arguments.length > 1) fail(ctx, node, 'arena-query-arity');
    let receiver = expr(node.expression.expression, { ...ctx, arenaReceiver: true });
    if (name === 'arenaList' && receiver.kind === 'ref') {
        if (receiver.path.join('.') === 'source.controller') receiver = reference('player');
        if (receiver.path.join('.') === 'source.controller.opponent') receiver = reference('opponent');
    }
    const playerReceiver = receiver.kind === 'ref' && ((receiver.path.length === 1 && ['player', 'opponent'].includes(receiver.path[0])) || (ctx.liveStats && receiver.path.join('.') === 'modifierTarget.controller'));
    const gameReceiver = name === 'arenaList' && receiver.kind === 'ref' && receiver.path.join('.') === 'game';
    if (!playerReceiver && !gameReceiver) fail(ctx, node, 'arena-query-receiver');
    const obj = filterProps ? { props: filterProps, ctx, node } : node.arguments.length ? object(node.arguments[0], ctx) : { props: new Map(), ctx, node };
    for (const value of obj.props.values()) arenaCapture(value, obj.ctx);
    // Player.getArenaUnits fixes controller and unit type; zone filter fields
    // intersect, while trait/aspect/keyword arrays each use OR semantics.
    for (const [key, value] of obj.props) if (!['arena', 'trait', 'aspect', 'keyword', 'otherThan', 'condition'].includes(key)) fail(obj.ctx, value, 'arena-query-filter:' + key);
    let predicate = constant(true);
    const intersect = (term) => { predicate = { kind: 'binary', op: '&&', left: predicate, right: term }; };
    if (obj.props.has('arena')) {
        const arena = expr(obj.props.get('arena'), obj.ctx);
        intersect({ kind: 'binary', op: '||', left: { kind: 'unary', op: '!', value: arena }, right: { kind: 'binary', op: '||', left: { kind: 'binary', op: '===', left: arena, right: constant('anyArena') }, right: { kind: 'binary', op: '===', left: reference('card', 'zoneName'), right: arena } } });
    }
    for (const [field, query] of [['trait', 'hasSomeTrait'], ['aspect', 'hasSomeAspect'], ['keyword', 'hasSomeKeyword']]) if (obj.props.has(field)) {
        if(field==='trait'){const dynamic=normalizeLeaderTraits(unwrap(obj.props.get(field)),obj.ctx,{ts,unwrap,expr,fail});if(dynamic){intersect({kind:'query',name:query,receiver:reference('card'),args:[dynamic]});continue;}}
        const value = literal(obj.props.get(field), obj.ctx);
        if (typeof value !== 'string' && !(Array.isArray(value) && value.every((v) => typeof v === 'string'))) fail(ctx, obj.props.get(field), 'arena-query-nonconstant-filter');
        if (value !== '') intersect({ kind: 'query', name: query, receiver: reference('card'), args: [Array.isArray(value) ? { kind: 'array', items: value.map(constant) } : constant(value)] });
    }
    if (obj.props.has('otherThan')) {
        const excluded = expr(obj.props.get('otherThan'), obj.ctx);
        const containsCard = (value) => value && typeof value === 'object' && ((value.kind === 'ref' && value.path[0] === 'card') || Object.values(value).some(containsCard));
        if (containsCard(excluded)) fail(ctx, obj.props.get('otherThan'), 'arena-query-card-capture');
        intersect({ kind: 'binary', op: '||', left: { kind: 'unary', op: '!', value: excluded }, right: { kind: 'binary', op: '!==', left: reference('card'), right: excluded } });
    }
    if (obj.props.has('condition')) {
        if (name === 'arenaList') {
            const callback = unwrap(obj.props.get('condition'));
            if ((!ts.isArrowFunction(callback) && !ts.isFunctionExpression(callback)) || callback.parameters.length !== 1) fail(ctx, callback, 'arena-predicate-shape');
        }
        intersect(expr(obj.props.get('condition'), { ...obj.ctx, syncArenaPredicate: true }, ['card']));
    }
    return { kind: 'query', name, receiver, args: [predicate] };
}
// Finite counts lower to existing scalar expressions, never general list/Set values.
function finiteCollectionCount(node,ctx) {
    const checked=result=>{
        if(!ctx.callbackStage)fail(ctx,node,'finite-count-evaluation-stage');
        const visit=n=>{if(n.questionDotToken)fail(ctx,n,'optional-finite-count');if((ts.isArrowFunction(n)||ts.isFunctionExpression(n))&&(n.asteriskToken||n.modifiers?.length||n.parameters.some(p=>p.initializer||p.dotDotDotToken||p.questionToken)))fail(ctx,n,'finite-count-callback-shape');ts.forEachChild(n,visit);};visit(node);return result;
    };
    const unaryCallback=(fn,field)=> {fn=unwrap(fn);return (ts.isArrowFunction(fn)||ts.isFunctionExpression(fn))&&fn.parameters.length===1&&ts.isIdentifier(fn.parameters[0].name)&&ts.isPropertyAccessExpression(unwrap(fn.body))&&unwrap(fn.body).name.text===field&&ts.isIdentifier(unwrap(fn.body).expression)&&unwrap(fn.body).expression.text===fn.parameters[0].name.text;};
    if(ts.isPropertyAccessExpression(node)&&node.name.text==='size'&&ts.isNewExpression(unwrap(node.expression))) {
        const set=unwrap(node.expression);if(!ts.isIdentifier(set.expression)||set.expression.text!=='Set'||set.arguments?.length!==1)return null;if(ctx.constants.has('Set')||ctx.scope.has('Set'))fail(ctx,node,'shadowed-finite-set');
        let collection=unwrap(set.arguments[0]);let mapper;
        if(ts.isCallExpression(collection)&&ts.isPropertyAccessExpression(collection.expression)&&collection.expression.name.text==='flat'&&collection.arguments.length===0){collection=unwrap(collection.expression.expression);if(ts.isCallExpression(collection)&&ts.isPropertyAccessExpression(collection.expression)&&collection.expression.name.text==='map'&&collection.arguments.length===1)mapper=collection.arguments[0];}
        if(!mapper||!unaryCallback(mapper,'aspects'))return null;
        const base=unwrap(collection.expression.expression);if(!ts.isCallExpression(base)||!ts.isPropertyAccessExpression(base.expression)||base.expression.name.text!=='getArenaUnits'||base.arguments.length)return null;
        const receiver=expr(base.expression.expression,{...ctx,arenaReceiver:true});if(receiver.kind!=='ref'||receiver.path.join('.')!=='player')fail(ctx,node,'finite-aspect-union-receiver');
        const terms=['vigilance','command','aggression','cunning','heroism','villainy'].map(aspect=>({kind:'conditional',condition:{kind:'query',name:'hasSomeArenaUnit',receiver,args:[{kind:'binary',op:'&&',left:constant(true),right:{kind:'query',name:'hasSomeAspect',receiver:reference('card'),args:[constant(aspect)]}}]},then:constant(1),else:constant(0)}));
        const balanced=xs=>xs.length===1?xs[0]:{kind:'binary',op:'+',left:balanced(xs.slice(0,Math.ceil(xs.length/2))),right:balanced(xs.slice(Math.ceil(xs.length/2)))};return checked(balanced(terms));
    }
    if(ts.isPropertyAccessExpression(node)&&node.name.text==='length') {
        const f=unwrap(node.expression);if(!ts.isCallExpression(f)||!ts.isPropertyAccessExpression(f.expression)||f.expression.name.text!=='filter'||f.arguments.length!==1)return null;
        const base=unwrap(f.expression.expression);if(!ts.isCallExpression(base)||!ts.isPropertyAccessExpression(base.expression)||base.expression.name.text!=='getPlayers'||base.arguments.length)return null;
        const game=expr(base.expression.expression,{...ctx,arenaReceiver:true});if(game.kind!=='ref'||game.path.join('.')!=='game')return null;
        const fn=unwrap(f.arguments[0]);if(!ts.isArrowFunction(fn)||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name))return null;
        const b=unwrap(fn.body);if(!ts.isBinaryExpression(b)||b.operatorToken.getText(ctx.source)!=='>='||!ts.isPropertyAccessExpression(b.left)||b.left.name.text!=='damage'||!ts.isPropertyAccessExpression(b.left.expression)||b.left.expression.name.text!=='base'||!ts.isIdentifier(b.left.expression.expression)||b.left.expression.expression.text!==fn.parameters[0].name.text)return null;
        const threshold=literal(b.right,ctx);if(threshold!==15)fail(ctx,node,'two-player-base-count-bound');
        const term=player=>({kind:'conditional',condition:{kind:'binary',op:'>=',left:reference(player,'base','damage'),right:constant(threshold)},then:constant(1),else:constant(0)});
        return checked({kind:'binary',op:'+',left:term('player'),right:term('opponent')});
    }
    return null;
}
function arenaCardExistence(node,ctx,upgrade=false) {
    if(node.questionDotToken||node.expression.questionDotToken)fail(ctx,node,'optional-arena-card-query');
    const obj=node.arguments.length?object(node.arguments[0],ctx):{props:new Map(),ctx,node};if(node.arguments.length>1)fail(ctx,node,'arena-card-query-arity');
    for(const [key,value]of obj.props)if(!(upgrade?['trait']:['trait','type']).includes(key))fail(ctx,value,'arena-card-query-filter:'+key);
    const props=new Map(obj.props);let type=upgrade?'upgrade':null;if(props.has('type')){type=literal(props.get('type'),obj.ctx);props.delete('type');if(!['leaderUnit','nonLeaderUnit'].includes(type))fail(ctx,node,'arena-card-query-type');}
    if(upgrade&&props.has('trait')){const trait=literal(props.get('trait'),obj.ctx);if(typeof trait!=='string'||!trait.length||trait.length>64)fail(ctx,node,'arena-upgrade-query-trait');}
    if(!type&&!props.has('trait'))fail(ctx,node,'arena-card-query-bounded-filter');
    const result=arenaQuery(node,ctx,'hasSomeArenaUnit',props);result.name='hasSomeArenaCard';
    if(type){const predicate=type==='nonLeaderUnit'?{kind:'query',name:'isNonLeaderUnit',receiver:reference('card'),args:[]}:type==='upgrade'?{kind:'query',name:'isUpgrade',receiver:reference('card'),args:[]}:{kind:'binary',op:'&&',left:{kind:'query',name:'isLeader',receiver:reference('card'),args:[]},right:{kind:'query',name:'isDeployed',receiver:reference('card'),args:[]}};result.args[0]={kind:'binary',op:'&&',left:result.args[0],right:predicate};}
    return result;
}
// A singleton's first element is the source exactly when the filtered arena
// contains one unit and contains the source. Both reads must use the same local
// snapshot in the executing callback; no general array indexing is admitted.
function singletonArenaMembership(node, ctx) {
    const isEqual = n => ts.isBinaryExpression(n) && n.operatorToken.kind === ts.SyntaxKind.EqualsEqualsEqualsToken;
    if (!ts.isBinaryExpression(node) || node.operatorToken.kind !== ts.SyntaxKind.AmpersandAmpersandToken) return null;
    const count = unwrap(node.left), member = unwrap(node.right);
    if (!isEqual(count) || !isEqual(member)) return null;
    const length = unwrap(count.left), one = unwrap(count.right), first = unwrap(member.left);
    if (!ts.isPropertyAccessExpression(length) || length.questionDotToken || length.name.text !== 'length' ||
        !ts.isNumericLiteral(one) || one.text !== '1' || !ts.isElementAccessExpression(first) || first.questionDotToken ||
        !ts.isNumericLiteral(first.argumentExpression) || first.argumentExpression.text !== '0' ||
        !ts.isIdentifier(length.expression) || !ts.isIdentifier(first.expression) || length.expression.text !== first.expression.text ||
        !ctx.constants.has(length.expression.text)) return null;
    const initializer = readConstant(length.expression.text, ctx);
    const local = constantContext(initializer, ctx), query = unwrap(initializer);
    if (!ts.isCallExpression(query) || !ts.isPropertyAccessExpression(query.expression) || query.expression.name.text !== 'getArenaUnits') return null;
    if (query.questionDotToken || query.expression.questionDotToken) fail(ctx, query, 'optional-singleton-arena-query');
    const source = expr(member.right, ctx);
    if (source.kind !== 'ref' || source.path.join('.') !== 'source') return null;
    const units = arenaQuery(query, local, 'arenaUnitCount');
    const containsSource = {kind:'query', name:'hasSomeArenaUnit', receiver:units.receiver, args:[
        {kind:'binary', op:'&&', left:units.args[0], right:{kind:'binary', op:'===', left:reference('card'), right:source}}
    ]};
    return {kind:'binary', op:'&&', left:{kind:'binary', op:'===', left:units, right:constant(1)}, right:containsSource};
}
function expr(input, context, params = ['context']) {
    let { node, ctx } = lambda(input, context, params);
    node = unwrap(node);
    // Lexical event reads take precedence over generic optional member paths.
    const explicitResult=normalizeEventResultExpression(node,ctx,{ts,unwrap,expr,fail,unshadowedEnum});if(explicitResult)return explicitResult;
    const inventory=normalizeInventoryExpression(node,ctx,{ts,unwrap,expr,fail});if(inventory)return inventory;
    const leaderTraits=normalizeLeaderTraits(node,ctx,{ts,unwrap,expr,fail});if(leaderTraits)return leaderTraits;
    const liveMaximum=normalizeLivePowerMaximum(node,ctx,{ts,unwrap,expr,fail,unshadowedEnum});if(liveMaximum)return liveMaximum;
    const reduction=normalizeBoundedReduction(node,ctx,{ts,unwrap,expr,fail});if(reduction)return reduction;
    if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ctx.reductionHelpers?.has(node.expression.name.text))return reductionHelperCall(node,ctx,ctx.reductionHelpers.get(node.expression.name.text),{ts,fail});
    const resultHelper=normalizeResultHelperCall(node,ctx,{ts,unwrap,expr,fail,unshadowedEnum});if(resultHelper)return resultHelper;
    const eventQuery = normalizeDefeatedUnitCountCall(node, ctx, { ts, expr, fail });
    if (eventQuery) return eventQuery;
    const titleQuery = normalizeControlledTitleCall(node, ctx, { ts, unwrap, expr, fail, unshadowedEnum });
    if (titleQuery) return titleQuery;
    const attachmentQuery = normalizePhysicalAttachmentExpression(node, ctx, { ts, unwrap, expr, literal, unshadowedEnum });
    if (attachmentQuery) return attachmentQuery;
    const attackCondition = normalizeAttackUnitCondition(node, ctx, { ts, unwrap, expr });
    if (attackCondition) return attackCondition;
    const attackEvent = normalizeAttackEventCall(node, ctx, { ts, unwrap, expr, fail });
    if (attackEvent) return attackEvent;
    const singleton = singletonArenaMembership(node, ctx); if (singleton) return singleton;
    const quantity=normalizeFiniteQuantity(node,ctx,{ts,unwrap,expr,literal,fail,unshadowedEnum});if(quantity)return quantity;
    const finite=finiteCollectionCount(node,ctx);if(finite)return finite;
    if(ctx.attachedParentTitle&&ts.isPropertyAccessExpression(node)&&node.questionDotToken&&node.name.text==='title'&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='parentCard') {const parent=expr(node.expression,ctx);if(parent.kind==='ref'&&parent.path.join('.')==='source.parentCard')return reference('source','parentCard','title');fail(ctx,node,'attached-parent-title-scope');}

    if (ts.isIdentifier(node) && ctx.constants?.has(node.text)) { const initializer = readConstant(node.text, ctx); return expr(initializer, constantContext(initializer, ctx)); }
    if (ts.isPropertyAccessExpression(node) && node.name.text === 'length') {
        // Preserve the established direct Player count representation byte-for-byte.
        if (ts.isCallExpression(node.expression) && ts.isPropertyAccessExpression(node.expression.expression) && node.expression.expression.name.text === 'getArenaUnits') {
            const receiver = expr(node.expression.expression.expression, { ...ctx, arenaReceiver: true });
            if (receiver.kind === 'ref' && ['player', 'opponent', 'modifierTarget.controller'].includes(receiver.path.join('.'))) return arenaQuery(node.expression, ctx, 'arenaUnitCount');
        }
        const receiver = arenaListNode(node.expression, ctx);
        if (receiver) return { kind: 'arenaLength', receiver };
    }
    if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && node.expression.name.text === 'some') {
        const receiver = arenaListNode(node.expression.expression, ctx);
        if (receiver) {
            if (node.questionDotToken || node.expression.questionDotToken) fail(ctx, node, 'optional-arena-list');
            if (node.arguments.length !== 1) fail(ctx, node, 'arena-some-arity');
            const callback = unwrap(node.arguments[0]);
            if ((!ts.isArrowFunction(callback) && !ts.isFunctionExpression(callback)) || callback.parameters.length !== 1) fail(ctx, callback, 'arena-predicate-shape');
            arenaCapture(callback, ctx);
            const predicate = expr(callback, { ...ctx, allowArenaList: false, syncArenaPredicate: true }, ['card']);
            arenaPredicate(predicate, ctx, callback);
            return { kind: 'arenaSome', receiver, predicate };
        }
    }
    if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && ['getArenaUnits', 'filter'].includes(node.expression.name.text)) {
        const list = arenaListNode(node, ctx);
        if (list) {
            if (!ctx.allowArenaList) fail(ctx, node, 'arena-list-scalar');
            return list;
        }
    }
    if (ts.isStringLiteral(node) || ts.isNumericLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword, ts.SyntaxKind.NullKeyword].includes(node.kind)) return constant(literal(node, ctx));
    if (ts.isIdentifier(node) || ts.isPropertyAccessExpression(node) || ts.isElementAccessExpression(node) || node.kind === ts.SyntaxKind.ThisKeyword) {
        const enumValue = ctx.enums[node.getText(ctx.source)];
        if (enumValue !== undefined) return constant(enumValue);
        let parts = access(node, ctx);
        let resolvedAlias = false;
        if (ctx.constants.has(parts[0])) {
            const initializer = readConstant(parts[0], ctx);
            const value = expr(initializer, constantContext(initializer, ctx));
            if (value.kind !== 'ref') fail(ctx, node, 'property-of-nonreference-local');
            parts = [...value.path, ...parts.slice(1)];
            resolvedAlias = true;
        }
        const binding = resolvedAlias ? null : ctx.scope.get(parts[0]);
        if (binding) {
            if (binding.kind !== 'ref') fail(ctx, node, 'property-of-nonreference-local');
            parts = [...binding.path, ...parts.slice(1)];
        } else if (!resolvedAlias && parts[0] !== 'this') fail(ctx, node, 'unbound-reference:' + parts[0]);
        parts = normalizeRef(parts);
        if (ctx.namedTargets && parts[0]==='targets') {
            const name=parts[1],slot=ctx.namedTargets.get(name);
            if (!slot) fail(ctx,node,'unknown-named-target:'+name);
            if (parts.length>2 && (slot.array || parts.length!==3 || !['power','hp','damage','remainingHp','cost','printedCost','exhausted','unique','zoneName','controller','title'].includes(parts[2]))) fail(ctx,node,'unsupported-named-target-property');
            parts=['namedTargets',...parts.slice(1)];
        } else if(ctx.namedTargets && parts[0]==='target') {
            const slot=ctx.namedTargets.get('target');
            if(!slot) fail(ctx,node,'unnamed-target-in-named-scope');
            if(parts.length>1 && (slot.array || parts.length!==2 || !['power','hp','damage','remainingHp','cost','printedCost','exhausted','unique','zoneName','controller','title'].includes(parts[1]))) fail(ctx,node,'unsupported-named-target-property');
            parts=['namedTargets','target',...parts.slice(1)];
        }
        if (parts.includes('activeAttack') && !(parts.join('.')==='card.activeAttack.attacker' && ctx.attackDefenders?.has('card'))) fail(ctx,node,'unsupported-attack-reference');
        if (ctx.liveStats && parts.length===3 && parts[0]==='card' && parts[1]==='zone' && parts[2]==='name') parts=['card','zoneName'];
        const roots = new Set(['source', 'player', 'opponent', 'target', 'targets', 'card', 'event', 'attack']);
        if (ctx.arenaReceiver) roots.add('game');
        if (ctx.attachPredicate) roots.add('attachTarget');
        if (ctx.liveStats) roots.add('modifierTarget');
        if (ctx.namedTargets) roots.add('namedTargets');
        if (!roots.has(parts[0])) fail(ctx, node, 'unimplemented-context-field:' + parts[0]);
        return reference(...parts);
    }
    if (ts.isArrayLiteralExpression(node)) return { kind: 'array', items: node.elements.map((e) => expr(e, { ...ctx, allowArenaList: false })) };
    if (ts.isPrefixUnaryExpression(node)) {
        const op = ts.tokenToString(node.operator);
        if (!['!', '-', '+', '~'].includes(op)) fail(ctx, node, 'unsupported-unary:' + op);
        return { kind: 'unary', op, value: expr(node.operand, { ...ctx, allowArenaList: false }) };
    }
    if (ts.isBinaryExpression(node)) {
        const op = node.operatorToken.getText(ctx.source);
        if (!['+', '-', '*', '/', '%', '===', '!==', '==', '!=', '<', '<=', '>', '>=', '&&', '||', '??', '&', '|'].includes(op)) fail(ctx, node, 'side-effecting-expression:' + op);
        ctx = { ...ctx, allowArenaList: false };
        const left = expr(node.left,ctx);
        let rightContext=ctx;
        if(op==='&&') {
            const attackDefenders=new Set(ctx.attackDefenders || []);
            const guards=v=> {
                if(v.kind==='query'&&v.name==='isDefending'&&v.receiver.kind==='ref'&&v.receiver.path.length===1) attackDefenders.add(v.receiver.path[0]);
                if(v.kind==='binary'&&v.op==='&&') {guards(v.left);guards(v.right);}
            };
            guards(left);rightContext={...ctx,attackDefenders};
        }
        return { kind:'binary',op,left,right:expr(node.right,rightContext) };
    }
    if (ts.isConditionalExpression(node)) return { kind: 'conditional', condition: expr(node.condition, { ...ctx, allowArenaList: false }), then: expr(node.whenTrue, { ...ctx, allowArenaList: false }), else: expr(node.whenFalse, { ...ctx, allowArenaList: false }) };
    if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression)) {
        const call = node.expression;
        const name = call.name.text;
        if(ctx.arenaDamageHelpers?.has(name))return arenaDamageHelperCall(node,ctx,ctx.arenaDamageHelpers.get(name));
        if (name === 'hasCost') {
            if(node.questionDotToken || call.questionDotToken)fail(ctx,node,'optional-cost-query');
            if(node.arguments.length)fail(ctx,node,'cost-query-arity');
            const receiver=expr(call.expression,ctx);
            const roots=['source','card','target'];
            if(ctx.attachPredicate)roots.push('attachTarget');
            if(receiver.kind!=='ref'||receiver.path.length!==1||!roots.includes(receiver.path[0]))fail(ctx,node,'cost-query-receiver');
            return {kind:'query',name,receiver,args:[]};
        }
        if (['isAttacking','isDefending'].includes(name)) {
            if(node.questionDotToken || call.questionDotToken) fail(ctx,node,'optional-attack-query');
            if(node.arguments.length) fail(ctx,node,'attack-query-arity');
            const receiver=expr(call.expression,ctx);
            if(receiver.kind!=='ref'||receiver.path.length!==1||!['source','card','target'].includes(receiver.path[0])) fail(ctx,node,'attack-query-receiver');
            return {kind:'query',name,receiver,args:[]};
        }
        if (name === 'hasSomeArenaUnit') return arenaQuery(node, ctx, name);
        if (name === 'isLeaderUnit') {
            if (node.questionDotToken || call.questionDotToken) fail(ctx,node,'optional-arena-leader-query');
            if (node.arguments.length) fail(ctx,node,'arena-leader-query-arity');
            const receiver=expr(call.expression,ctx);
            if (!ctx.arenaLeaderCandidate || receiver.kind!=='ref' || receiver.path.length!==1 || receiver.path[0]!=='card') fail(ctx,node,'arena-leader-query-scope');
            return {kind:'binary',op:'&&',left:{kind:'query',name:'isUnit',receiver,args:[]},right:{kind:'query',name:'isLeader',receiver,args:[]}};
        }
        if (name === 'hasSomeArenaCard') return arenaCardExistence(node,ctx);
        if (name === 'hasSomeArenaUpgrade') return arenaCardExistence(node,ctx,true);
        if (name === 'isTokenUnit') {
            if (node.questionDotToken || call.questionDotToken) fail(ctx,node,'optional-token-unit-query');
            if (node.arguments.length) fail(ctx,node,'token-unit-query-arity');
            const receiver = expr(call.expression,ctx);
            if (receiver.kind !== 'ref' || receiver.path.length !== 1 || !['source','card','target'].includes(receiver.path[0])) fail(ctx,node,'token-unit-query-receiver');
            return {kind:'query',name,receiver,args:[]};
        }
        if (['isTraitInPlay', 'isAspectInPlay', 'isKeywordInPlay'].includes(name)) {
            if (node.arguments.length < 1 || node.arguments.length > 2) fail(ctx, node, 'arena-query-alias-arity');
            const field = { isTraitInPlay: 'trait', isAspectInPlay: 'aspect', isKeywordInPlay: 'keyword' }[name];
            const filters = new Map([[field, node.arguments[0]]]);
            if (node.arguments.length === 2) filters.set('otherThan', node.arguments[1]);
            return arenaQuery(node, ctx, 'hasSomeArenaUnit', filters);
        }
        if (ctx.liveStats && call.expression.getText(ctx.source) === 'Math' && name === 'floor') {
            if (node.arguments.length !== 1) fail(ctx,node,'live-stat-resource-pairs-shape');
            const divided = unwrap(node.arguments[0]);
            if (!ts.isBinaryExpression(divided) || divided.operatorToken.kind !== ts.SyntaxKind.SlashToken || literal(divided.right,ctx) !== 2) fail(ctx,node,'live-stat-resource-pairs-shape');
            const count = expr(divided.left,ctx);
            if (count.kind !== 'ref' || !['modifierTarget.controller.resources.length','player.resources.length','opponent.resources.length'].includes(count.path.join('.'))) fail(ctx,node,'live-stat-resource-pairs-shape');
            return {kind:'query',name:'resourcePairs',receiver:reference(...count.path.slice(0,-2)),args:[]};
        }
        if (call.expression.getText(ctx.source) === 'Math' && ['min', 'max', 'floor', 'ceil', 'abs'].includes(name)) return { kind: 'math', name, args: node.arguments.map((a) => expr(a, ctx)) };
        if (QUERY_NAMES.has(name)) return { kind: 'query', name, receiver: expr(call.expression, ctx), args: node.arguments.map((a) => expr(a, ctx)) };
        if(ctx.aspectIncludesAlias&&name==='includes'&&ts.isPropertyAccessExpression(call.expression)&&call.expression.name.text==='aspects') {
            if(node.arguments.length!==1||node.questionDotToken||call.questionDotToken||call.expression.questionDotToken)fail(ctx,node,'aspect-includes-shape');
            const receiver=expr(call.expression.expression,ctx),arg=unwrap(node.arguments[0]);
            if(receiver.kind!=='ref'||receiver.path.join('.')!=='card'||!ts.isPropertyAccessExpression(arg)||!ts.isIdentifier(arg.expression)||arg.expression.text!=='Aspect'||arg.questionDotToken)fail(ctx,node,'aspect-includes-shape');
            for(const namespace of ['Aspect','WildcardZoneName','TextHelper'])unshadowedEnum(arg,ctx,namespace);
            const value=literal(arg,ctx);if(!['vigilance','command','aggression','cunning','villainy','heroism'].includes(value))fail(ctx,node,'aspect-includes-value');
            return {kind:'query',name:'hasSomeAspect',receiver,args:[constant(value)]};
        }
        if (['some', 'every', 'filter', 'map', 'find', 'includes', 'concat', 'slice'].includes(name)) {
            const receiver = expr(call.expression, ctx);
            if (isArenaList(receiver)) fail(ctx, node, 'unsupported-arena-collection:' + name);
            const args = node.arguments.map((a) => expr(a, ctx, ['card', 'index']));
            return { kind: 'collection', name, receiver, args, limit: 256 };
        }
        fail(ctx, node, 'unsupported-query:' + name);
    }
    fail(ctx, node, 'expression:' + ts.SyntaxKind[node.kind], node.getText(ctx.source));
}
function object(input, context) {
    let { node, ctx } = lambda(input, context);
    node = unwrap(node);
    if (ts.isIdentifier(node) && ctx.constants.has(node.text)) { const initializer=readConstant(node.text,ctx); return object(initializer,constantContext(initializer,ctx)); }
    if (!ts.isObjectLiteralExpression(node)) fail(ctx, node, 'expected-declarative-object');
    const props = new Map();
    for (const p of node.properties) {
        if (ts.isPropertyAssignment(p)) {
            const name = ts.isIdentifier(p.name) || ts.isStringLiteral(p.name) ? p.name.text : null;
            if (name === null) fail(ctx, p, 'computed-object-key');
            if (props.has(name)) fail(ctx, p, 'duplicate-object-key:' + name);
            props.set(name, p.initializer);
        } else if (ts.isShorthandPropertyAssignment(p) && ctx.constants.has(p.name.text)) props.set(p.name.text, readConstant(p.name.text, ctx));
        else fail(ctx, p, 'object-property:' + ts.SyntaxKind[p.kind]);
    }
    return { props, ctx, node };
}
function knownProperties(obj, names) {
    for (const [key, node] of obj.props) if (!names.includes(key) && !DISPLAY_FIELDS.has(key)) fail(obj.ctx, node, 'unsupported-field:' + key);
}
// Only effect targets gain this bounded list union. Scalar list algebra keeps
// its existing rejection path, and existing arenaList targets omit ordering.
function orderedArenaTarget(input, context, depth=0) {
    if(depth>24)fail(context,input,'ordered-arena-alias-bound');
    let node=unwrap(input),ctx=context;
    if(ts.isIdentifier(node)&&ctx.constants.has(node.text)){
        const initializer=readConstant(node.text,ctx);
        return orderedArenaTarget(initializer,constantContext(initializer,ctx),depth+1);
    }
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||!['concat','reduce'].includes(node.expression.name.text))return null;
    const call=node.expression,name=call.name.text;
    if(!ctx.callbackStage||node.questionDotToken||call.questionDotToken)fail(ctx,node,'ordered-arena-target-stage-or-call');
    const resolve=(value,scope)=>{
        let n=unwrap(value),child=scope,steps=0;
        while(ts.isIdentifier(n)&&child.constants.has(n.text)){
            if(++steps>24)fail(child,n,'ordered-arena-alias-bound');
            const initializer=readConstant(n.text,child);child=constantContext(initializer,child);n=unwrap(initializer);
        }
        return {node:n,ctx:child};
    };
    const ground=(value,scope)=>{
        const resolved=resolve(value,scope),q=resolved.node,child=resolved.ctx;
        if(!ts.isCallExpression(q)||q.questionDotToken||!ts.isPropertyAccessExpression(q.expression)||q.expression.questionDotToken||q.expression.name.text!=='getArenaUnits'||q.arguments.length!==1)fail(child,q,'ordered-arena-ground-query');
        const ownerPath=access(q.expression.expression,child),root=ownerPath[0],binding=child.scope.get(root);
        const directContext=!child.constants.has(root)&&binding?.kind==='ref'&&binding.path.join('.')==='context'&&['player','player.opponent'].includes(ownerPath.slice(1).join('.'));
        const directPlayer=!child.constants.has(root)&&ownerPath.length===1&&binding?.kind==='ref'&&['player','opponent'].includes(binding.path.join('.'));
        if(!directContext&&!directPlayer)fail(child,q,'ordered-arena-direct-controller');
        const filter=unwrap(q.arguments[0]);if(!ts.isObjectLiteralExpression(filter)||filter.properties.length!==1)fail(child,filter,'ordered-arena-ground-filter');
        const p=filter.properties[0];if(!ts.isPropertyAssignment(p)||!ts.isIdentifier(p.name)||p.name.text!=='arena')fail(child,p,'ordered-arena-ground-filter');
        const arena=unwrap(p.initializer);
        if(!ts.isStringLiteral(arena)&&!ts.isNoSubstitutionTemplateLiteral(arena)&&!ts.isPropertyAccessExpression(arena))fail(child,arena,'ordered-arena-static-filter');
        if(ts.isPropertyAccessExpression(arena)&&ts.isIdentifier(arena.expression)&&(child.scope.has(arena.expression.text)||child.constants.has(arena.expression.text)))fail(child,arena,'ordered-arena-shadowed-enum');
        if(literal(arena,child)!=='groundArena')fail(child,arena,'ordered-arena-ground-only');
        const query=arenaQuery(q,child,'arenaList');
        if(query.receiver.kind!=='ref'||query.receiver.path.length!==1||!['player','opponent'].includes(query.receiver.path[0]))fail(child,q,'ordered-arena-controller');
        return {owner:query.receiver.path[0],predicate:query.args[0]};
    };
    if(name==='concat'){
        if(node.arguments.length!==1||ts.isSpreadElement(node.arguments[0]))fail(ctx,node,'ordered-arena-concat-arity');
        const left=ground(call.expression,ctx),right=ground(node.arguments[0],ctx);
        if(left.owner===right.owner||JSON.stringify(left.predicate)!==JSON.stringify(right.predicate))fail(ctx,node,'ordered-arena-complementary-controllers');
        return {kind:'arenaList',receiver:reference('game'),predicate:left.predicate,controllerOrder:left.owner==='player'?'playerFirst':'opponentFirst'};
    }
    if(node.arguments.length!==2||!ts.isArrayLiteralExpression(node.arguments[1])||node.arguments[1].elements.length!==0)fail(ctx,node,'ordered-arena-reduce-initializer');
    const players=unwrap(call.expression);
    if(!ts.isCallExpression(players)||players.questionDotToken||!ts.isPropertyAccessExpression(players.expression)||players.expression.questionDotToken||players.expression.name.text!=='getPlayers'||players.arguments.length!==0)fail(ctx,players,'ordered-arena-players-query');
    const gamePath=access(players.expression.expression,ctx),gameRoot=gamePath[0],gameBinding=ctx.scope.get(gameRoot);
    if(ctx.constants.has(gameRoot)||gameBinding?.kind!=='ref'||gameBinding.path.join('.')!=='context'||gamePath.slice(1).join('.')!=='game')fail(ctx,players,'ordered-arena-players-receiver');
    const game=expr(players.expression.expression,{...ctx,arenaReceiver:true});if(game.kind!=='ref'||game.path.join('.')!=='game')fail(ctx,players,'ordered-arena-players-receiver');
    const reducer=unwrap(node.arguments[0]);
    if(!ts.isArrowFunction(reducer)||isNonSynchronous(reducer)||hasDecorators(reducer)||reducer.parameters.length!==2||reducer.parameters.some(p=>!ts.isIdentifier(p.name)||p.questionToken||unsupportedParameterBinding(p)))fail(ctx,reducer,'ordered-arena-reducer-binding');
    const [accumulator,player]=reducer.parameters.map(p=>p.name.text);if(accumulator===player)fail(ctx,reducer,'ordered-arena-reducer-binding');
    const body=unwrap(reducer.body);
    if(!ts.isCallExpression(body)||body.questionDotToken||!ts.isPropertyAccessExpression(body.expression)||body.expression.questionDotToken||body.expression.name.text!=='concat'||!ts.isIdentifier(body.expression.expression)||body.expression.expression.text!==accumulator||body.arguments.length!==1)fail(ctx,body,'ordered-arena-reducer-body');
    const query=unwrap(body.arguments[0]);
    if(!ts.isCallExpression(query)||!ts.isPropertyAccessExpression(query.expression)||!ts.isIdentifier(query.expression.expression)||query.expression.expression.text!==player)fail(ctx,query,'ordered-arena-reducer-player');
    const child={...ctx,scope:new Map(ctx.scope),constants:new Map(ctx.constants)};
    child.scope.set(accumulator,reference('card'));child.scope.set(player,reference('player'));child.constants.delete(accumulator);child.constants.delete(player);
    const list=ground(query,child);
    return {kind:'arenaList',receiver:reference('game'),predicate:list.predicate,controllerOrder:'seatOrder'};
}
function ownTopDeckResourceTarget(input,context,depth=0) {
    if(depth>24)fail(context,input,'top-deck-resource-alias-bound');
    let node=unwrap(input),ctx=context;
    if(ts.isIdentifier(node)&&ctx.constants.has(node.text)){
        const initializer=readConstant(node.text,ctx);
        return ownTopDeckResourceTarget(initializer,constantContext(initializer,ctx),depth+1);
    }
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.name.text!=='getTopCardOfDeck')return null;
    if(!ctx.callbackStage||node.questionDotToken||node.expression.questionDotToken||node.arguments.length)fail(ctx,node,'top-deck-resource-call');
    const parts=access(node.expression.expression,ctx),binding=ctx.scope.get(parts[0]);
    if(ctx.constants.has(parts[0])||binding?.kind!=='ref'||binding.path.join('.')!=='context'||parts.slice(1).join('.')!=='player')fail(ctx,node,'top-deck-resource-context');
    return 'ownTopDeck';
}
// Enum spellings are constants only when their imported namespace remains
// unshadowed. Apply this extra fence to the newly admitted continuation roots.
function topDeckContinuationNamespaces(node,ctx) {
    const namespaces=new Set(['Trait','RelativePlayer','WildcardCardType','TextHelper']);
    const names=(binding,root)=>ts.isIdentifier(binding)?binding.text===root:binding.elements?.some(e=>e.name&&names(e.name,root));
    const visit=n=>{
        if(ts.isPropertyAccessExpression(n)&&ts.isIdentifier(n.expression)&&namespaces.has(n.expression.text)) {
            const root=n.expression.text;
            for(let a=n.parent;a;a=a.parent) {
                if(a.parameters?.some(p=>names(p.name,root))||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>names(d.name,root))))fail(ctx,n,'top-deck-resource-enum-binding');
            }
        }
        ts.forEachChild(n,visit);
    };
    visit(node);
}
function topDeckResourceSourceShape(out) {
    const keys=(value,names)=>Object.keys(value||{}).sort().join(',')===[...names].sort().join(',');
    const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
    if(out.abilities.leader.length||out.abilities.unit.length||out.abilities.card.length!==1)return false;
    const a=out.abilities.card[0],resource={kind:'resourceCard',target:'ownTopDeck'};
    if(out.baseClass==='UpgradeCard'&&a.trigger==='constant'&&a.effect?.kind==='gainBounty'&&a.effect.target==='parent'&&a.effect.duration==='persistent'&&same(a.effect.ability.effect,resource))return true;
    const fewer=(zone)=>({kind:'binary',op:'<',left:reference('player',zone,'length'),right:reference('opponent',zone,'length')});
    const conditional=(condition,onTrue)=>({kind:'conditional',condition,onTrue,onFalse:{kind:'noAction'}});
    if(out.baseClass==='EventCard') {
        if(keys(a,['trigger','title','costs','effect'])&&a.trigger==='event'&&same(a.costs,[])&&same(a.effect,{kind:'simultaneous',effects:[{kind:'resourceCard',target:'self'},resource]}))return true;
        return keys(a,['trigger','title','costs','targetResolver','effect','ifYouDo'])&&a.trigger==='event'&&same(a.costs,[])&&
            same(a.targetResolver,{owner:'self',type:'unit',zone:'anyAttackable',predicate:{kind:'query',name:'hasSomeTrait',receiver:reference('card'),args:[constant('imperial')]},promptTitle:'Defeat a friendly Imperial unit'})&&
            same(a.effect,{kind:'defeat',target:'chosen'})&&keys(a.ifYouDo,['title','costs','effect'])&&same(a.ifYouDo.costs,[])&&same(a.ifYouDo.effect,resource);
    }
    if(out.baseClass==='NonLeaderUnitCard'&&a.trigger==='bounty'&&a.bounty&&keys(a,['trigger','title','costs','optional','effect','bounty'])&&same(a.costs,[])&&a.optional===true&&same(a.effect,resource))return true;
    if(out.baseClass!=='NonLeaderUnitCard'||a.trigger!=='whenPlayed'||!same(a.costs,[]))return false;
    if(keys(a,['trigger','title','costs','effect'])) {
        return same(a.effect,{kind:'sequence',effects:[conditional(fewer('hand'),{kind:'draw',target:'ownPlayer',amount:constant(1)}),conditional(fewer('resources'),resource)]});
    }
    if(!keys(a,['trigger','title','costs','optional','effect'])||a.optional!==true)return false;
    if(same(a.effect,resource))return true;
    const units={kind:'binary',op:'>=',left:{kind:'query',name:'arenaUnitCount',receiver:reference('player'),args:[constant(true)]},right:constant(3)};
    return same(a.effect,conditional(fewer('resources'),resource))||same(a.effect,conditional(units,resource));
}

function target(node, ctx, fallback = 'self') {
    if (!node) return fallback;
    const ordered=orderedArenaTarget(node,ctx);
    if(ordered)return ordered;
    const otherBase=otherBaseTarget(node,ctx);
    if(otherBase)return otherBase;
    const e = expr(node, { ...ctx, allowArenaList: true });
    const stringify = (v) => v.kind === 'ref' ? v.path.join('.') : null;
    const mappings = { source: 'self', target: 'chosen', 'player.base': 'ownBase', 'opponent.base': 'enemyBase', player: 'ownPlayer', opponent: 'enemyPlayer' };
    if (mappings[stringify(e)]) return mappings[stringify(e)];
    if (e.kind === 'array') {
        const paths = e.items.map(stringify);
        if (paths.length === 2 && paths.includes('player.base') && paths.includes('opponent.base')) return 'bothBases';
        if (paths.length === 2 && paths.includes('player') && paths.includes('opponent')) return 'bothPlayers';
    }
    return e;
}
// Kessel has exactly two players. With a scalar base resolver the complement
// of the selected base is one base; keep that identity in the target register.
function otherBaseTarget(input,ctx) {
    const node=unwrap(input);
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.name.text!=='filter')return null;
    const map=unwrap(node.expression.expression);
    if(!ts.isCallExpression(map)||!ts.isPropertyAccessExpression(map.expression)||map.expression.name.text!=='map')return null;
    const players=unwrap(map.expression.expression);
    if(!ts.isCallExpression(players)||!ts.isPropertyAccessExpression(players.expression)||players.expression.name.text!=='getPlayers')return null;
    if(!ctx.otherBaseTargetScope||!ctx.callbackStage)fail(ctx,node,'other-base-target-scope');
    for(const call of [node,map,players])if(call.questionDotToken||call.expression.questionDotToken)fail(ctx,call,'optional-other-base-target');
    if(node.arguments.length!==1||map.arguments.length!==1||players.arguments.length)fail(ctx,node,'other-base-target-arity');
    const game=expr(players.expression.expression,{...ctx,arenaReceiver:true});
    if(game.kind!=='ref'||game.path.join('.')!=='game')fail(ctx,node,'other-base-target-receiver');
    for(const callback of [map.arguments[0],node.arguments[0]]){
        const fn=unwrap(callback);
        if((!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))||fn.parameters.length!==1)fail(ctx,fn,'other-base-target-callback');
    }
    const mapped=expr(map.arguments[0],ctx,['card']);
    const predicate=expr(node.arguments[0],ctx,['card']);
    const ref=(e,name)=>e.kind==='ref'&&e.path.join('.')===name;
    if(!ref(mapped,'card.base')||predicate.kind!=='binary'||predicate.op!=='!=='||!ref(predicate.left,'card')||!ref(predicate.right,'target'))fail(ctx,node,'other-base-target-shape');
    return {kind:'conditional',condition:{kind:'binary',op:'===',left:reference('target'),right:reference('player','base')},then:reference('opponent','base'),else:reference('player','base')};
}
// This grammar is mirrored by raw-IR admission. modifierTarget is the
// affected card, card is an arena-query candidate, and source/target keep their
// existing meanings. Live values cannot depend on effective stats or keywords.
function liveStatBounds(value, ctx, node, depth = 0) {
    if (depth > 24) fail(ctx,node,'live-stat-expression-bound');
    const rejected = () => fail(ctx,node,'unsupported-live-stat-expression');
    if (value.kind === 'const') {
        if (!Number.isSafeInteger(value.value) || Math.abs(value.value) > 1000) rejected();
        return [value.value,value.value];
    }
    if(value.kind==='attachmentCount'&&value.receiver.kind==='ref'&&['modifierTarget','source','target','card'].includes(value.receiver.path[0]))return [0,256];
    if(value.kind==='binary'&&value.op==='??'&&value.right.kind==='const'&&value.right.value===0&&value.left.kind==='memberPath'&&value.left.steps.slice(-2).map(s=>s.property).join('.')==='upgrades.length')return [0,256];
    if (value.kind === 'ref') {
        const p=value.path.join('.');
        if (p === 'modifierTarget.damage') return [0,65535];
        if (['modifierTarget.controller.resources.length','player.resources.length','opponent.resources.length','player.hand.length','opponent.hand.length'].includes(p)) return [0,64];
        rejected();
    }
    if (value.kind === 'unary' && value.op === '-') {
        const [lo,hi]=liveStatBounds(value.value,ctx,node,depth+1);return [-hi,-lo];
    }
    if (value.kind === 'binary' && ['+','-','*'].includes(value.op)) {
        const [a,b]=liveStatBounds(value.left,ctx,node,depth+1),[c,d]=liveStatBounds(value.right,ctx,node,depth+1);
        const values=value.op==='+'?[a+c,b+d]:value.op==='-'?[a-d,b-c]:[a*c,a*d,b*c,b*d];
        const bounds=[Math.min(...values),Math.max(...values)];
        if (bounds.some(n=>!Number.isSafeInteger(n)||Math.abs(n)>1000000)) fail(ctx,node,'live-stat-value-bound');
        return bounds;
    }
    if (value.kind === 'query' && ['arenaUnitCount','resourcePairs'].includes(value.name)) {
        const path=value.receiver.kind==='ref'?value.receiver.path.join('.'):'';
        if (!['player','opponent','modifierTarget.controller'].includes(path)) rejected();
        if (value.name==='resourcePairs') {if(value.args.length) rejected();return [0,32];}
        if (value.args.length!==1) rejected();
        const check=(v, exclusion=false, nesting=0)=> {
            if(!v||typeof v!=='object'||nesting>24) fail(ctx,node,'live-stat-expression-bound');
            if(v.kind==='const') {
                if (!['boolean','string','number'].includes(typeof v.value) || (typeof v.value==='number' && (!Number.isSafeInteger(v.value)||Math.abs(v.value)>1000))) rejected();
                return;
            }
            if(v.kind==='ref') {
                const path=v.path.join('.');
                if (['modifierTarget','source'].includes(path) && exclusion) return;
                if (['card','card.damage','card.exhausted','card.unique','card.zoneName','card.zone.name'].includes(path)) return;
                if(v.path.some(p=>['power','hp','remainingHp'].includes(p))) fail(ctx,node,'constant-stat-dependency');
                fail(ctx,node,'live-stat-arena-capture');
            }
            if(v.kind==='query') {
                if(['getPower','getHp','hasSomeKeyword','hasEveryKeyword','getNumericKeywordTotal'].includes(v.name)) fail(ctx,node,'constant-stat-dependency');
                if (!['isUnit','isTokenUnit','hasSomeTrait','hasSomeAspect'].includes(v.name) || v.receiver.kind!=='ref' || v.receiver.path.join('.')!=='card') rejected();
                const arity=['isUnit','isTokenUnit'].includes(v.name)?0:1;
                if(v.args.length!==arity || v.args.some(a=>a.kind!=='const'&&!(a.kind==='array'&&a.items.every(x=>x.kind==='const')))) rejected();
                if(arity && v.args.some(a=>a.kind==='const'?typeof a.value!=='string':a.items.some(x=>typeof x.value!=='string'))) rejected();
                return;
            }
            if(v.kind==='binary') {
                if(!['&&','||','===','!==','>','>=','<','<='].includes(v.op)) rejected();
                check(v.left,v.op==='!==',nesting+1);check(v.right,v.op==='!==',nesting+1);return;
            }
            if(v.kind==='unary'&&v.op==='!') {check(v.value,v.value.kind==='ref',nesting+1);return;}
            rejected();
        };
        check(value.args[0]);
        return [0,255];
    }
    rejected();
}
const MODIFIER_KEYWORDS = new Set(['sentinel','saboteur','overwhelm','grit','raid','restore','ambush','shielded','hidden','support']);
function modifierEffects(input, ctx, duration, targetValue) {
    let raw = unwrap(input);
    while (ts.isIdentifier(raw) && ctx.constants.has(raw.text)) raw = unwrap(readConstant(raw.text, ctx));
    const nodes = ts.isArrayLiteralExpression(raw) ? raw.elements : [raw];
    if (!nodes.length || nodes.length > 8) fail(ctx, raw, 'modifier-list-bound');
    const effects = nodes.flatMap((input) => {
        const call = unwrap(input);
        if (!ts.isCallExpression(call)) fail(ctx, call, 'modifier-is-not-helper');
        const parts = access(call.expression, ctx), kind = parts[2];
        if (parts.length !== 3 || !['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0]) || parts[1] !== 'ongoingEffects' || !['modifyStats','gainKeyword','loseKeyword'].includes(kind)) fail(ctx, call, 'unsupported-modifier:' + parts.join('.'));
        if (call.arguments.length !== 1) fail(ctx, call, 'modifier-arity');
        let value = unwrap(call.arguments[0]);
        while (ts.isIdentifier(value) && ctx.constants.has(value.text)) value = unwrap(readConstant(value.text, ctx));
        const dynamic = ts.isArrowFunction(value) || ts.isFunctionExpression(value);
        if (kind === 'modifyStats') {
            if (dynamic && ['phase','attack'].includes(duration)) fail(ctx,value,'dynamic-ongoing-stat-modifier');
            const lowered = dynamic ? lambda(value,{...ctx,liveStats:true},['modifierTarget','context']) : {node:value,ctx};
            const obj = object(lowered.node,lowered.ctx);
            // Every modifier field is semantic; descriptive-property exemptions
            // used by ordinary ability objects do not apply to live stat objects.
            for (const [key,v] of obj.props) if (!['power','hp'].includes(key)) fail(obj.ctx,v,'unsupported-stat-modifier-field:'+key);
            const out={kind,target:targetValue,duration};
            for (const stat of ['power','hp']) {
                if (!obj.props.has(stat)) fail(ctx,value,'incomplete-stat-modifier');
                let v=unwrap(obj.props.get(stat));
                while(ts.isIdentifier(v)&&obj.ctx.constants.has(v.text))v=unwrap(readConstant(v.text,obj.ctx));
                if(ts.isArrowFunction(v)||ts.isFunctionExpression(v))fail(ctx,v,'function-valued-stat-modifier');
                if(dynamic) {out[stat]=expr(v,obj.ctx);liveStatBounds(out[stat],obj.ctx,v);}
                else if(duration === 'persistent') { const n=literal(v,obj.ctx); if(!Number.isSafeInteger(n))fail(ctx,v,'noninteger-stat-modifier'); out[stat]=constant(n); }
                else out[stat]=expr(v,obj.ctx);
            }
            return [out];
        }
        if (kind === 'loseKeyword') {
            const names=literal(value,ctx), list=Array.isArray(names)?names:[names];
            if(!list.length||list.length>8||list.some(k=>!MODIFIER_KEYWORDS.has(k)))fail(ctx,value,'unsupported-modifier-keyword');
            return list.map(keyword=>({kind,target:targetValue,duration,keyword}));
        }
        if(dynamic&&['phase','attack'].includes(duration))fail(ctx,value,'dynamic-ongoing-keyword-modifier');
        const lowered=dynamic?lambda(value,ctx,['card','context']):{node:value,ctx};
        let keyword,amount=constant(1);
        if(ts.isObjectLiteralExpression(lowered.node)) {
            const obj=object(lowered.node,lowered.ctx);
            if(obj.props.has('ability'))return [bountyGrant(obj,duration,targetValue,{ability,literal,fail})];
            knownProperties(obj,['keyword','amount']);
            if(!obj.props.has('keyword'))fail(ctx,value,'missing-modifier-keyword');
            keyword=literal(obj.props.get('keyword'),obj.ctx);
            if(obj.props.has('amount'))amount=expr(obj.props.get('amount'),obj.ctx);
            if(['raid','restore'].includes(keyword)!==obj.props.has('amount'))fail(ctx,value,'invalid-keyword-amount');
        } else {keyword=literal(lowered.node,lowered.ctx);if(['raid','restore'].includes(keyword))fail(ctx,value,'missing-numeric-keyword-amount');}
        if(!MODIFIER_KEYWORDS.has(keyword))fail(ctx,value,'unsupported-modifier-keyword');
        return [{kind,target:targetValue,duration,keyword,amount}];
    });
    if(effects.length>8)fail(ctx,raw,'modifier-list-bound');
    if(['phase','attack'].includes(duration)&&ts.isArrayLiteralExpression(raw))return {kind:duration==='phase'?'phaseModifiers':'attackModifiers',effects};
    return effects.length===1?effects[0]:{kind:'simultaneous',effects};
}
function effect(input, context, defaultTarget = 'self') {
    let { node, ctx } = lambda(input, context);
    node = unwrap(node);
    if (ts.isIdentifier(node) && ctx.constants.has(node.text)) return effect(readConstant(node.text, ctx), ctx, defaultTarget);
    if (!ts.isCallExpression(node)) fail(ctx, node, 'effect-is-not-helper-call');
    const maximum=maximumHelperCall(node,ctx,{ts,unwrap,fail});if(maximum)return maximum;
    const parts = access(node.expression, ctx);
    if (parts.length !== 3 || !['AbilityHelper', 'abilityHelper', 'StaticAbilityHelper'].includes(parts[0]) || parts[1] !== 'immediateEffects') fail(ctx, node, 'unsupported-effect-call:' + parts.join('.'));
    const kind = parts[2];
    const pendingInventory=pendingInventoryEffect(node,ctx,defaultTarget,{ts,unwrap,fail});if(pendingInventory)return pendingInventory;
    if(kind!=='heal'&&ctx.otherBaseTargetScope)ctx={...ctx,otherBaseTargetScope:false};
    if(kind!=='damage'&&ctx.damageRecipientScope)ctx={...ctx,damageRecipientScope:false};
    if (node.arguments.length > 1) fail(ctx, node, 'effect-helper-arity');
    if (['simultaneous', 'sequential'].includes(kind)) {
        if(node.arguments.length===1&&!ts.isArrayLiteralExpression(unwrap(node.arguments[0]))&&kind==='simultaneous')return finiteHandGroupFactory(node.arguments[0],ctx);
        if (node.arguments.length !== 1 || !ts.isArrayLiteralExpression(unwrap(node.arguments[0]))) fail(ctx, node, 'invalid-effect-sequence');
        return { kind: kind === 'sequential' ? 'sequence' : 'simultaneous', effects: unwrap(node.arguments[0]).elements.map((e) => effect(e, ctx, defaultTarget)) };
    }
    let obj = node.arguments[0] ? object(node.arguments[0], ctx) : { props: new Map(), ctx, node };
    if (kind === 'capture') {
        // Representation only: the Rust engine has no capture zone/lifecycle yet.
        for(const [key,value]of obj.props)if(!['target','captor','fromOutOfPlay'].includes(key))fail(obj.ctx,value,'unsupported-capture-field:'+key);
        const fromOutOfPlay=obj.props.has('fromOutOfPlay')?literal(obj.props.get('fromOutOfPlay'),obj.ctx):false;
        if(typeof fromOutOfPlay!=='boolean')fail(obj.ctx,obj.node,'capture-out-of-play-boolean');
        return {kind:'capture',target:target(obj.props.get('target'),obj.ctx,defaultTarget),
            captor:obj.props.has('captor')?expr(obj.props.get('captor'),obj.ctx):reference('source'),
            fromOutOfPlay,targetType:'nonLeaderUnit'};
    }
    if (kind === 'discardCardsFromOwnHand') {
        // The wrapper changes who chooses, not who is targeted. Its inherited
        // PlayerTargetSystem default is the opponent, despite the helper name.
        for (const [key, value] of obj.props) if (!['amount', 'target', 'cardTypeFilter'].includes(key)) fail(obj.ctx, value, 'unsupported-hand-discard-field:' + key);
        if (obj.props.has('cardTypeFilter') && literal(obj.props.get('cardTypeFilter'), obj.ctx) !== 'any') fail(obj.ctx, obj.props.get('cardTypeFilter'), 'hand-discard-type-filter');
        const perPlayer = perPlayerHandDiscard(obj,{ts,unwrap,expr,literal,fail,unshadowedEnum});
        if (perPlayer) return perPlayer;
        const amount = obj.props.has('amount') ? literal(obj.props.get('amount'), obj.ctx) : 1;
        if (!Number.isSafeInteger(amount) || amount < 1 || amount > 2) fail(obj.ctx, obj.props.get('amount') || node, 'hand-discard-amount-bound');
        const playerTarget = target(obj.props.get('target'), obj.ctx, 'enemyPlayer');
        if (!['ownPlayer', 'enemyPlayer'].includes(playerTarget)) fail(obj.ctx, obj.props.get('target') || node, 'hand-discard-player-target');
        return { kind:'discardOwnHand', amount:constant(amount), target:playerTarget };
    }
    if (kind === 'discardFromDeck') {
        for (const [key, value] of obj.props) if (!['amount', 'target'].includes(key)) fail(obj.ctx, value, 'unsupported-deck-discard-field:' + key);
        // DiscardFromDeckSystem supplies no amount default; the inherited
        // PlayerTargetSystem default targets the opponent.
        if (!obj.props.has('amount')) fail(ctx, node, 'deck-discard-missing-amount');
        const amount = literal(obj.props.get('amount'), obj.ctx);
        if (!Number.isSafeInteger(amount) || amount < 0 || amount > 5) fail(obj.ctx, obj.props.get('amount'), 'deck-discard-amount-bound');
        const playerTarget = target(obj.props.get('target'), obj.ctx, 'enemyPlayer');
        if (!['ownPlayer', 'enemyPlayer'].includes(playerTarget)) fail(obj.ctx, obj.props.get('target') || node, 'deck-discard-player-target');
        return { kind:'discardDeck', amount:constant(amount), target:playerTarget };
    }
    const tokenTypes = { createBattleDroid:'battleDroid', createCloneTrooper:'cloneTrooper', createXWing:'xwing', createTieFighter:'tieFighter', createSpy:'spy', createBeast:'beast', createMandalorian:'mandalorian' };
    if (Object.hasOwn(tokenTypes, kind)) {
        // Token factory wrappers append their fixed type; caller-supplied type,
        // entry effects and event captures require their own execution contract.
        for (const [key, value] of obj.props) if (!['amount', 'target', 'entersReady'].includes(key)) fail(obj.ctx, value, 'unsupported-token-field:' + key);
        const dynamicAmount = obj.props.has('amount') ? resourceTokenAmount(obj.props.get('amount'), obj.ctx, {ts,unwrap,expr,literal,fail,unshadowedEnum}) : null;
        const amount = dynamicAmount || (obj.props.has('amount') ? literal(obj.props.get('amount'), obj.ctx) : 1);
        if (!dynamicAmount && (!Number.isSafeInteger(amount) || amount < 1 || amount > 5)) fail(obj.ctx, obj.props.get('amount') || node, 'token-amount-bound');
        const entersReady = obj.props.has('entersReady') ? literal(obj.props.get('entersReady'), obj.ctx) : false;
        if (typeof entersReady !== 'boolean') fail(obj.ctx, obj.props.get('entersReady') || node, 'token-readiness-boolean');
        const tokenTarget = target(obj.props.get('target'), obj.ctx, 'ownPlayer');
        if (!['ownPlayer', 'enemyPlayer'].includes(tokenTarget)) fail(obj.ctx, obj.props.get('target') || node, 'token-player-target');
        return { kind:dynamicAmount?'createTokenUnitsDynamic':'createTokenUnit', tokenType:tokenTypes[kind], amount:dynamicAmount||constant(amount), target:tokenTarget, entersReady };
    }
    if (['forThisPhaseCardEffect','forThisAttackCardEffect'].includes(kind)) {
        // Lasting stat modifiers have their values captured when the immediate
        // system resolves. A function passed to modifyStats itself is different:
        // the engine keeps reevaluating it as board state changes.
        const allowed = ['target', 'effect', 'title', 'ongoingEffectTargetDescription'];
        for (const [key, value] of obj.props) if (!allowed.includes(key) && !DISPLAY_FIELDS.has(key)) fail(obj.ctx, value, 'unsupported-field:' + key);
        if (!obj.props.has('effect')) fail(ctx, node, kind==='forThisAttackCardEffect'?'attack-effect-without-modifier':'phase-effect-without-modifier');
        const duration=kind==='forThisAttackCardEffect'?'attack':'phase';
        return modifierEffects(obj.props.get('effect'), obj.ctx, duration, target(obj.props.get('target'), obj.ctx, defaultTarget));
    }
    if (kind === 'conditional') {
        knownProperties(obj, ['condition', 'onTrue', 'onFalse']);
        if (!obj.props.has('condition')) fail(ctx, node, 'conditional-without-condition');
        return { kind, condition: expr(obj.props.get('condition'), obj.ctx), onTrue: obj.props.has('onTrue') ? effect(obj.props.get('onTrue'), obj.ctx, defaultTarget) : { kind: 'noAction' }, onFalse: obj.props.has('onFalse') ? effect(obj.props.get('onFalse'), obj.ctx, defaultTarget) : { kind: 'noAction' } };
    }
    if (kind === 'distributeHealingAmong' || kind === 'distributeDamageAmong') {
        knownProperties(obj, ['amountToDistribute', 'controller', 'cardTypeFilter', 'zoneFilter', 'maxTargets', 'canChooseNoTargets', 'canDistributeLess', 'cardCondition']);
        const spec = targetResolverProperties(obj);
        return { kind: kind === 'distributeHealingAmong' ? 'distributeHealing' : 'distributeDamage', amount: expr(obj.props.get('amountToDistribute'), obj.ctx), ...(obj.props.has('maxTargets') ? { maxTargets: literal(obj.props.get('maxTargets'), obj.ctx) } : {}), canChooseNoTargets: obj.props.has('canChooseNoTargets') ? literal(obj.props.get('canChooseNoTargets'), obj.ctx) : false, canDistributeLess: obj.props.has('canDistributeLess') ? literal(obj.props.get('canDistributeLess'), obj.ctx) : kind === 'distributeHealingAmong', targetResolver: spec };
    }
    if (kind === 'selectCard') {
        knownProperties(obj, ['cardTypeFilter', 'controller', 'zoneFilter', 'cardCondition', 'optional', 'mode', 'numCards', 'canChooseNoCards', 'immediateEffect', 'choosingPlayer']);
        const spec = targetResolverProperties(obj);
        if (obj.props.has('activePromptTitle')) {
            const node = unwrap(obj.props.get('activePromptTitle'));
            if (!(ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateExpression(node) && node.templateSpans.every(span => titleHelper(span.expression, obj.ctx) !== null))) fail(obj.ctx, node, 'inline-prompt-title');
            const title = displayText(node, obj.ctx);
            if (!title.length || title.length > 1024) fail(obj.ctx, node, 'inline-prompt-title');
            spec.promptTitle = title;
        }
        return { kind: 'select', targetResolver: spec, effect: effect(obj.props.get('immediateEffect'), obj.ctx, 'chosen') };
    }
    if (kind === 'useTheForce' || kind === 'theForceIsWithYou') {
        if (defaultTarget !== 'self') fail(ctx, node, 'force-inherited-target');
        knownProperties(obj, []);
        return { kind: kind === 'useTheForce' ? 'useForce' : 'gainForce', target: 'ownPlayer' };
    }
    if (kind === 'resourceCard') {
        knownProperties(obj, ['target']);
        return { kind, target: (obj.props.has('target')&&ownTopDeckResourceTarget(obj.props.get('target'),obj.ctx)) || target(obj.props.get('target'), obj.ctx, defaultTarget) };
    }
    if (kind === 'giveWeakness') {
        knownProperties(obj, ['amount', 'target']);
        return { kind, target: target(obj.props.get('target'), obj.ctx, defaultTarget), amount: obj.props.has('amount') ? expr(obj.props.get('amount'), obj.ctx) : constant(1) };
    }
    if (!BASIC_EFFECTS.has(kind)) fail(ctx, node, 'immediateEffect:' + kind);
    if (kind === 'damage') obj = normalizeDamageDefaultSource(obj, { ts, unwrap, expr });
    knownProperties(obj, ['amount', 'target', 'isCost', 'type', 'damageType', 'isIndirect', 'cannotBePrevented']);
    if (obj.props.has('isCost') && literal(obj.props.get('isCost'), obj.ctx)) fail(ctx, node, 'effect-is-cost');
    if (obj.props.has('type') && literal(obj.props.get('type'), obj.ctx) !== 'ability') fail(ctx, node, 'non-ability-damage');
    if (obj.props.has('damageType') || obj.props.has('isIndirect') || obj.props.has('cannotBePrevented')) fail(ctx, node, 'modified-damage-semantics');
    const result = { kind, target: target(obj.props.get('target'), obj.ctx, kind === 'draw' || kind === 'readyResources' ? 'ownPlayer' : defaultTarget) };
    if (obj.props.has('amount')) {
        let amount=unwrap(obj.props.get('amount')),amountContext=obj.ctx;
        // A readonly alias does not change DamageSystem's callback argument.
        // Retain each initializer's lexical context and capture-stage checks.
        while(kind==='damage'&&ts.isIdentifier(amount)&&amountContext.constants.has(amount.text)){
            const initializer=readConstant(amount.text,amountContext);
            amountContext=constantContext(initializer,amountContext);amount=unwrap(initializer);
        }
        if(kind==='damage'&&(ts.isArrowFunction(amount)||ts.isFunctionExpression(amount))){
            if(!obj.ctx.damageRecipientScope||result.target!=='chosen'||amount.parameters.length!==1)fail(obj.ctx,amount,'damage-recipient-callback-scope');
            const value=expr(amount,amountContext,['target']);
            // This bounded family reads the actual recipient parameter. A
            // captured outer target may lower to the same ref but is a snapshot.
            const body=unwrap(amount.body),parameter=amount.parameters[0].name;
            if(!ts.isIdentifier(parameter)||!ts.isBinaryExpression(body)||body.operatorToken.kind!==ts.SyntaxKind.MinusToken||!ts.isPropertyAccessExpression(unwrap(body.left))||unwrap(body.left).name.text!=='remainingHp'||!ts.isIdentifier(unwrap(unwrap(body.left).expression))||unwrap(unwrap(body.left).expression).text!==parameter.text)fail(obj.ctx,amount,'damage-recipient-callback-shape');
            if(value.kind!=='binary'||value.op!=='-'||value.left.kind!=='ref'||value.left.path.join('.')!=='target.remainingHp'||value.right.kind!=='const'||value.right.value!==1)fail(obj.ctx,amount,'damage-recipient-callback-shape');
            result.amount=value;
        }else {
            // Only expression result positions matter here. Arena-query
            // predicate arguments remain callbacks with their own typed roles.
            const callableValue=(node,context)=>{
                node=unwrap(node);
                if(ts.isIdentifier(node)&&context.constants.has(node.text)){const initializer=readConstant(node.text,context);return callableValue(initializer,constantContext(initializer,context));}
                if(ts.isArrowFunction(node)||ts.isFunctionExpression(node))return true;
                if(ts.isConditionalExpression(node))return callableValue(node.condition,context)||callableValue(node.whenTrue,context)||callableValue(node.whenFalse,context);
                if(ts.isBinaryExpression(node))return callableValue(node.left,context)||callableValue(node.right,context);
                if(ts.isPrefixUnaryExpression(node))return callableValue(node.operand,context);
                if(ts.isArrayLiteralExpression(node))return node.elements.some(e=>callableValue(e,context));
                if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.expression.getText(context.source)==='Math')return node.arguments.some(e=>callableValue(e,context));
                return false;
            };
            if(kind==='damage'&&callableValue(amount,amountContext))fail(obj.ctx,amount,'damage-callable-amount-composite');
            result.amount=expr(obj.props.get('amount'),obj.ctx);
        }
    }
    else if (['draw', 'giveShield', 'giveExperience', 'readyResources'].includes(kind)) result.amount = constant(1);
    else if (['damage', 'heal'].includes(kind)) fail(ctx, node, 'missing-effect-amount:' + kind);
    return result;
}
// This factory snapshots a resource comparison before any child event. Neither
// supported child changes resources, and group branches prepare before events.
function finiteHandGroupFactory(input,context) {
    const factory=unwrap(input);
    if(!ts.isArrowFunction(factory)||factory.parameters.length!==1||!ts.isIdentifier(factory.parameters[0].name)||!ts.isArrayLiteralExpression(unwrap(factory.body)))fail(context,factory,'simultaneous-factory-shape');
    if(['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(factory.parameters[0].name.text))fail(context,factory,'simultaneous-factory-helper-shadow');
    const {node,ctx}=lambda(factory,context);if(node.elements.length!==2)fail(ctx,node,'simultaneous-factory-bound');
    const effects=node.elements.map(n=>effect(n,ctx));
    const [draw,conditional]=effects;
    const unit=e=>e?.kind==='const'&&e.value===1;
    const ref=(e,p)=>e?.kind==='ref'&&e.path.join('.')===p;
    if(draw.kind!=='draw'||draw.target!=='ownPlayer'||!unit(draw.amount)||conditional.kind!=='conditional'||conditional.onTrue.kind!=='discardOwnHand'||conditional.onTrue.target!=='enemyPlayer'||!unit(conditional.onTrue.amount)||conditional.onFalse.kind!=='noAction'||conditional.condition.kind!=='binary'||conditional.condition.op!=='>'||!ref(conditional.condition.left,'opponent.resources.length')||!ref(conditional.condition.right,'player.resources.length'))fail(ctx,node,'simultaneous-factory-effects');
    const second=unwrap(node.elements[1]);if(!ts.isCallExpression(second)||second.arguments.length!==1)fail(ctx,second,'simultaneous-factory-condition');
    const objectNode=unwrap(second.arguments[0]);if(!ts.isObjectLiteralExpression(objectNode))fail(ctx,objectNode,'simultaneous-factory-condition');
    const condition=objectNode.properties.find(p=>ts.isPropertyAssignment(p)&&p.name.getText(ctx.source)==='condition');
    const raw=condition&&unwrap(condition.initializer),parameter=factory.parameters[0].name.text;
    if(!raw||!ts.isBinaryExpression(raw)||raw.operatorToken.kind!==ts.SyntaxKind.GreaterThanToken||JSON.stringify(access(raw.left,ctx))!==JSON.stringify([parameter,'player','opponent','resources','length'])||JSON.stringify(access(raw.right,ctx))!==JSON.stringify([parameter,'player','resources','length']))fail(ctx,raw||node,'simultaneous-factory-snapshot');
    return {kind:'simultaneous',effects};
}
function targetResolverProperties(obj) {
    const { props, ctx, node } = obj;
    const rawType = props.has('cardTypeFilter') ? literal(props.get('cardTypeFilter'), ctx) : 'any';
    const typeNames = { unit: 'unit', 'nonLeaderUnit': 'nonLeaderUnit', 'non-leader unit': 'nonLeaderUnit', basicUnit: 'basicUnit', base: 'base', any: 'any', upgrade: 'upgrade', event: 'event', leaderUnit: 'leaderUnit', tokenUnit: 'tokenUnit', token: 'token', basicUpgrade: 'basicUpgrade', tokenUpgrade: 'tokenUpgrade', nonTokenUnit: 'nonTokenUnit', nonUnit: 'nonUnit' };
    let type = Array.isArray(rawType) ? null : typeNames[rawType];
    // Engine arrays are unions. Only this exact union can use the runtime's
    // attackable inventory; discard/hand inventories may contain other types.
    if (Array.isArray(rawType) && rawType.length === 2 && rawType.includes('unit') && rawType.includes('base')) {
        const zone = props.has('zoneFilter') ? literal(props.get('zoneFilter'), ctx) : 'anyAttackable';
        if (['anyAttackable', 'anyArena', 'groundArena', 'spaceArena', 'base'].includes(zone)) type = 'any';
    }
    if (!type) fail(ctx, props.get('cardTypeFilter') || node, 'cardTypeFilter:' + rawType);
    const ownerRaw = props.has('controller') ? literal(props.get('controller'), ctx) : 'any';
    const owner = { self: 'self', opponent: 'opponent', any: 'any' }[ownerRaw];
    if (!owner) fail(ctx, props.get('controller') || node, 'target-controller:' + ownerRaw);
    const zoneRaw = props.has('zoneFilter') ? literal(props.get('zoneFilter'), ctx) : 'anyAttackable';
    const zones = { anyAttackable: 'anyAttackable', anyArena: 'arena', groundArena: 'arena', spaceArena: 'arena', base: 'base', hand: 'hand', discard: 'discard', resource: 'resource', deck: 'deck', any: 'any' };
    if (!zones[zoneRaw]) fail(ctx, props.get('zoneFilter') || node, 'target-zone:' + zoneRaw);
    const result = { owner, type, zone: zones[zoneRaw] };
    if (zoneRaw === 'groundArena' || zoneRaw === 'spaceArena') result.arena = zoneRaw === 'groundArena' ? 'ground' : 'space';
    if (props.has('optional')) result.optional = literal(props.get('optional'), ctx);
    if (props.has('cardCondition')) {
        const capturedTarget=(n,bound=new Set(),seen=new Set())=>{
            if(ts.isArrowFunction(n)||ts.isFunctionExpression(n)) {const local=new Set(bound);for(const p of n.parameters)if(ts.isIdentifier(p.name))local.add(p.name.text);return capturedTarget(n.body,local,seen);}
            if(ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n)) {
                let root=n;while(ts.isPropertyAccessExpression(root)||ts.isElementAccessExpression(root))root=root.expression;
                if(ts.isIdentifier(root)&&!bound.has(root.text)&&ctx.scope.has(root.text)) {
                    const binding=ctx.scope.get(root.text);
                    if(binding.kind==='ref'&&!isEventResultAccess(n,ctx,{ts,unwrap})) {const parts=normalizeRef([...binding.path,...access(n,ctx).slice(1)]);if(['target','targets'].includes(parts[0]) && !ctx.namedTargets)fail(ctx,n,'target-resolver-outer-target-capture');}
                }
            }
            if(ts.isIdentifier(n)&&!bound.has(n.text)&&ctx.constants.has(n.text)&&!(ts.isPropertyAccessExpression(n.parent)&&n.parent.name===n)) {
                if(seen.has(n.text))fail(ctx,n,'cyclic-target-resolver-binding');const next=new Set(seen);next.add(n.text);capturedTarget(readConstant(n.text,ctx),bound,next);
            }
            ts.forEachChild(n,child=>capturedTarget(child,bound,seen));
        };
        capturedTarget(props.get('cardCondition'));
        result.predicate = (ctx.aspectIncludesAlias&&aspectMembershipPredicate(props.get('cardCondition'),ctx))||expr(props.get('cardCondition'), {...ctx,powerMaximumStage:unwrap(props.get('cardCondition'))}, ['card', 'context']);
    }
    if (props.has('mode')) result.mode = literal(props.get('mode'), ctx);
    // A selected player needs a typed player target register. Falling back to
    // ownPlayer loses the user's choice (e.g. a ready-resources continuation).
    if (result.mode === 'player') fail(ctx, props.get('mode'), 'unsupported-player-target-resolver');
    if (props.has('numCards')) result.count = expr(props.get('numCards'), ctx);
    if (props.has('canChooseNoCards')) result.optional = !!result.optional || literal(props.get('canChooseNoCards'), ctx);
    else if (['upTo','upToVariable'].includes(result.mode)) result.optional = true;
    if (props.has('choosingPlayer') && literal(props.get('choosingPlayer'), ctx) !== 'self') fail(ctx, node, 'opponent-choice-resolver');
    return result;
}
function costs(input, ctx) {
    if (!input) return [];
    const nodes = ts.isArrayLiteralExpression(unwrap(input)) ? unwrap(input).elements : [input];
    return nodes.map((n) => {
        n = unwrap(n);
        if (!ts.isCallExpression(n)) fail(ctx, n, 'cost-is-not-helper');
        const parts = access(n.expression, ctx);
        if (parts[1] !== 'costs') fail(ctx, n, 'invalid-cost-helper');
        const kind = parts.at(-1);
        if (kind === 'exhaustSelf' && !n.arguments.length) return { kind };
        if (kind === 'abilityActivationResourceCost' && n.arguments.length === 1) return { kind: 'resources', amount: expr(n.arguments[0], ctx) };
        fail(ctx, n, 'cost:' + kind);
    });
}
// Recognize the complete guarded attack-membership predicate, rather than
// exposing arbitrary activeAttack target collections to scalar expressions.
function defendingAttackerPredicate(input,context) {
    let found=false;const search=n=>{if(ts.isPropertyAccessExpression(n)&&n.name.text==='getAllTargets')found=true;ts.forEachChild(n,search);};search(input);
    if(!found)return null;
    const fn=unwrap(input);
    if((!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))||fn.parameters.length!==2||fn.parameters.some(p=>!ts.isIdentifier(p.name))||fn.parameters[0].name.text===fn.parameters[1].name.text)fail(context,input,'defending-attacker-predicate');
    const {node,ctx}=lambda(fn,context,['card','context']);
    const terms=[];const flatten=n=>{n=unwrap(n);if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.AmpersandAmpersandToken){flatten(n.left);flatten(n.right);}else terms.push(n);};flatten(node);
    const card=fn.parameters[0].name.text,actual=fn.parameters[1].name.text;
    const call=(n,name)=>ts.isCallExpression(n)&&!n.questionDotToken&&ts.isPropertyAccessExpression(n.expression)&&!n.expression.questionDotToken&&n.expression.name.text===name;
    if(terms.length!==4)fail(ctx,node,'defending-attacker-predicate');
    for(let i=0;i<3;i++)if(!call(terms[i],['isUnit','isInPlay','isAttacking'][i])||terms[i].arguments.length||!ts.isIdentifier(terms[i].expression.expression)||terms[i].expression.expression.text!==card)fail(ctx,terms[i],'defending-attacker-predicate');
    const includes=terms[3],targets=call(includes,'includes')&&unwrap(includes.expression.expression);
    const attack=targets&&call(targets,'getAllTargets')&&unwrap(targets.expression.expression);
    const arg=includes.arguments?.length===1&&unwrap(includes.arguments[0]);
    const parent=arg&&ts.isPropertyAccessExpression(arg)&&!arg.questionDotToken&&arg.name.text==='parentUnit';
    const sourceArg=parent?unwrap(arg.expression):arg;
    if(!targets||targets.arguments.length||!attack||!ts.isPropertyAccessExpression(attack)||attack.questionDotToken||attack.name.text!=='activeAttack'||!ts.isIdentifier(attack.expression)||attack.expression.text!==card||!sourceArg||!ts.isPropertyAccessExpression(sourceArg)||sourceArg.questionDotToken||sourceArg.name.text!=='source'||!ts.isIdentifier(sourceArg.expression)||sourceArg.expression.text!==actual)fail(ctx,node,'defending-attacker-predicate');
    const queries=['isUnit','isInPlay','isAttacking'].map(name=>({kind:'query',name,receiver:reference('card'),args:[]}));
    return {predicate:{kind:'binary',op:'&&',left:{kind:'binary',op:'&&',left:queries[0],right:queries[1]},right:queries[2]},condition:{kind:'query',name:'isDefending',receiver:parent?reference('source','parentUnit'):reference('source'),args:[]},parentDefense:!!parent};
}
function parentDefenseHelperBinding(node,ctx) {
    let setup=node;while(setup&&!ts.isMethodDeclaration(setup))setup=setup.parent;
    if(!setup||setup.name.getText(ctx.source)!=='setupCardAbilities'||setup.parameters.length!==2||setup.parameters.some(p=>!ts.isIdentifier(p.name))||setup.parameters[0].name.text===setup.parameters[1].name.text)fail(ctx,node,'parent-defense-setup-binding');
    const raw=unwrap(node);
    if(!ts.isCallExpression(raw)||raw.arguments.length!==1||!ts.isObjectLiteralExpression(unwrap(raw.arguments[0])))fail(ctx,node,'parent-defense-helper-binding');
    const stats=object(raw.arguments[0],ctx);
    if(stats.props.size!==2||literal(stats.props.get('power'),stats.ctx)!==-1||literal(stats.props.get('hp'),stats.ctx)!==0)fail(ctx,node,'parent-defense-literal-stats');
    const root=setup.parameters[1].name.text,parts=access(raw.expression,ctx);
    if(parts.join('.')!==root+'.ongoingEffects.modifyStats')fail(ctx,node,'parent-defense-helper-binding');
    for(let a=node.parent;a&&a!==setup;a=a.parent)if(a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===root)||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===root)))fail(ctx,node,'parent-defense-helper-binding');
}
function parentDefenseSourceShape(out) {
    const a=out.abilities.card[0],same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
    const q=name=>({kind:'query',name,receiver:reference('card'),args:[]});
    const predicate={kind:'binary',op:'&&',left:{kind:'binary',op:'&&',left:q('isUnit'),right:q('isInPlay')},right:q('isAttacking')};
    return out.baseClass==='UpgradeCard'&&out.abilities.card.length===1&&!out.abilities.leader.length&&!out.abilities.unit.length&&
        same(out.attachCondition,{kind:'unary',op:'!',value:{kind:'query',name:'hasSomeTrait',receiver:reference('card'),args:[constant('vehicle')]}})&&
        same(a,{trigger:'constant',title:a.title,costs:[],effect:{kind:'modifyStats',target:'chosen',duration:'persistent',power:constant(-1),hp:constant(0)},condition:{kind:'query',name:'isDefending',receiver:reference('source','parentUnit'),args:[]},targetResolver:{owner:'opponent',type:'unit',zone:'arena',predicate}});
}
function aspectMembershipPredicate(input,context,depth=0) {
    if(depth>24)fail(context,input,'aspect-membership-alias-bound');
    const alias=unwrap(input);
    if(ts.isIdentifier(alias)&&context.constants.has(alias.text)){const initializer=readConstant(alias.text,context);return aspectMembershipPredicate(initializer,constantContext(initializer,context),depth+1);}
    let includes=false;const visit=n=>{if(ts.isPropertyAccessExpression(n)&&n.name.text==='includes'&&ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='aspects')includes=true;ts.forEachChild(n,visit);};visit(input);
    if(!includes)return null;
    const fn=unwrap(input);
    if((!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name))fail(context,input,'aspect-membership-predicate');
    const {node,ctx}=lambda(fn,context,['card','context']),n=unwrap(node),card=fn.parameters[0].name.text;
    const call=(x,name)=>ts.isCallExpression(x)&&!x.questionDotToken&&ts.isPropertyAccessExpression(x.expression)&&!x.expression.questionDotToken&&x.expression.name.text===name;
    if(!ts.isBinaryExpression(n)||n.operatorToken.kind!==ts.SyntaxKind.AmpersandAmpersandToken)fail(ctx,n,'aspect-membership-predicate');
    const left=unwrap(n.left),right=unwrap(n.right);
    if(!call(left,'isUnit')||left.arguments.length||!ts.isIdentifier(left.expression.expression)||left.expression.expression.text!==card||!call(right,'includes')||!ts.isPropertyAccessExpression(right.expression.expression)||right.expression.expression.name.text!=='aspects'||right.expression.expression.questionDotToken||!ts.isIdentifier(right.expression.expression.expression)||right.expression.expression.expression.text!==card)fail(ctx,n,'aspect-membership-predicate');
    return expr(n,ctx);
}
function unshadowedEnum(node,ctx,root) {
    for(let a=node.parent;a;a=a.parent)if(a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===root)||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===root)))fail(ctx,node,'syntax-alias-enum-binding');
    if(ctx.scope.has(root)||ctx.constants.has(root))fail(ctx,node,'syntax-alias-enum-binding');
}
function constantAbility(input, context, validatedObject = null) {
    const obj = validatedObject || object(input, context);
    knownProperties(obj, ['title', 'condition', 'ongoingEffect', 'matchTarget', 'targetController', 'targetCardTypeFilter', 'targetZoneFilter']);
    const { props, ctx, node } = obj;
    if (!props.has('ongoingEffect')) fail(ctx, node, 'constant-without-effect');
    const result = { trigger:'constant', title:props.has('title')?displayText(props.get('title'),ctx):'', costs:[], effect:normalizeKeywordCopy(props.get('ongoingEffect'),ctx,{ts,unwrap,fail})||modifierEffects(props.get('ongoingEffect'),ctx,'persistent','self') };
    let attack=null;
    if(props.has('matchTarget')){try{attack=defendingAttackerPredicate(props.get('matchTarget'),ctx);}catch(error){if(!(error instanceof Unsupported))throw error;}}
    if(attack) {
        if([...props.keys()].some(k=>!['title','targetController','matchTarget','ongoingEffect'].includes(k))||!props.has('targetController')||literal(props.get('targetController'),ctx)!=='opponent')fail(ctx,node,'defending-attacker-constant');
        unshadowedEnum(props.get('targetController'),ctx,'RelativePlayer');
        if(JSON.stringify(result.effect)!==JSON.stringify({kind:'modifyStats',target:'self',duration:'persistent',power:constant(-1),hp:constant(0)}))fail(ctx,node,'defending-attacker-modifier');
        if(attack.parentDefense)parentDefenseHelperBinding(props.get('ongoingEffect'),ctx);
        result.condition=attack.condition;
    } else if (props.has('condition')) result.condition = expr(props.get('condition'), ctx);
    // OngoingCardEffect defaults to source-only when no targeting filters exist.
    // Any targeting field switches to friendly arena units, including matchTarget.
    if (['matchTarget', 'targetController', 'targetCardTypeFilter', 'targetZoneFilter'].some((key) => props.has(key))) {
        const resolverProps = new Map();
        resolverProps.set('controller', props.get('targetController') || ts.factory.createStringLiteral('self'));
        resolverProps.set('cardTypeFilter', props.get('targetCardTypeFilter') || ts.factory.createStringLiteral('unit'));
        resolverProps.set('zoneFilter', props.get('targetZoneFilter') || ts.factory.createStringLiteral('anyArena'));
        if (props.has('matchTarget')&&!attack) resolverProps.set('cardCondition', props.get('matchTarget'));
        result.targetResolver = targetResolverProperties({ props: resolverProps, ctx, node });
        if(attack)result.targetResolver.predicate=attack.predicate;
        if (result.targetResolver.zone !== 'arena') fail(ctx, node, 'constant-target-zone');
        const assignTarget = (e) => { if(e.effects)e.effects.forEach(assignTarget);else e.target='chosen'; }; assignTarget(result.effect);
    }
    const keywordModifier = (e) => e.effects ? e.effects.some(keywordModifier) : ['gainKeyword','loseKeyword','copyKeywords'].includes(e.kind);
    const modifiesKeywords = keywordModifier(result.effect);
    const check = (value) => {
        if (!value || typeof value !== 'object') return;
        if ((value.kind === 'query' && (['getPower','getHp'].includes(value.name) || (modifiesKeywords && ['hasSomeKeyword','hasEveryKeyword','getNumericKeywordTotal'].includes(value.name)))) || (value.kind === 'ref' && value.path.some((p) => ['power', 'hp', 'remainingHp'].includes(p)))) fail(ctx, node, 'constant-stat-dependency');
        Object.values(value).forEach(check);
    };
    if(!powerMaximumGrant(result)){check(result.condition); check(result.targetResolver); check(result.effect);}
    return result;
}
// These are grant templates owned by an upgrade definition. At declaration,
// runtime captures each physical grantor and binds the body to its host.
// Keep this provenance separate from attachedContext used by live constants.
function gainedHostOnAttack(input,ctx,setup,call) {
    const reject=()=>fail(ctx,call,'registrar:addGainOnAttackAbilityTargetingAttached');
    const raw=unwrap(input);
    if(call.questionDotToken||!ts.isObjectLiteralExpression(raw)||setup.modifiers?.some(m=>m.kind===ts.SyntaxKind.StaticKeyword)||setup.name.getText(ctx.source)!=='setupCardAbilities'||setup.parameters.length!==2||setup.parameters.some(p=>!ts.isIdentifier(p.name))||setup.parameters[0].name.text===setup.parameters[1].name.text)reject();
    const obj=object(raw,ctx),keys=(obj,names)=>[...obj.props.keys()].sort().join(',')===[...names].sort().join(',');
    const helperRoot=setup.parameters[1].name.text;
    const helper=(input,category,name)=>{
        const n=unwrap(input);
        if(!ts.isCallExpression(n)||n.questionDotToken||n.arguments.length!==1||access(n.expression,ctx).join('.')!==helperRoot+'.'+category+'.'+name)reject();
        for(let a=n.parent;a&&a!==setup;a=a.parent)if(a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===helperRoot)||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===helperRoot)))reject();
        return n.arguments[0];
    };
    const plainObject=n=>{n=unwrap(n);if(!ts.isObjectLiteralExpression(n))reject();return object(n,ctx);};
    const simpleCallback=(input,count)=>{
        const fn=unwrap(input);
        if((!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))||fn.parameters.length!==count||fn.parameters.some(p=>!ts.isIdentifier(p.name))||new Set(fn.parameters.map(p=>p.name.text)).size!==count)reject();
        // lambda validates synchrony and plain bindings. Keep only direct bodies;
        // block aliases and captured earlier values need a separate grammar.
        lambda(fn,ctx);
        if(ts.isBlock(fn.body))reject();
        return {body:unwrap(fn.body),names:fn.parameters.map(p=>p.name.text)};
    };
    const pathIs=(n,parts)=>{n=unwrap(n);try{return access(n,ctx).join('.')===parts.join('.');}catch(error){if(!(error instanceof Unsupported))throw error;return false;}};
    const other=(input,unit)=>{
        const {body,names:[card,context]}=simpleCallback(input,2);
        const exclusion=n=>ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.ExclamationEqualsEqualsToken&&pathIs(n.left,[card])&&pathIs(n.right,[context,'source']);
        if(!unit){if(!exclusion(body))reject();return;}
        if(!ts.isBinaryExpression(body)||body.operatorToken.kind!==ts.SyntaxKind.AmpersandAmpersandToken||!exclusion(unwrap(body.left)))reject();
        const right=unwrap(body.right);if(!ts.isCallExpression(right)||right.questionDotToken||right.arguments.length||!pathIs(right.expression,[card,'isUnit']))reject();
    };
    const title=unwrap(obj.props.get('title')||raw);
    if(!(ts.isStringLiteral(title)||ts.isNoSubstitutionTemplateLiteral(title))||!title.text.length||title.text.length>1024)reject();
    let shape;
    if(keys(obj,['title','optional','targetResolver'])&&literal(obj.props.get('optional'),ctx)===true) {
        const resolver=plainObject(obj.props.get('targetResolver'));
        if(keys(resolver,['cardCondition','cardTypeFilter','immediateEffect'])) {
            const type=resolver.props.get('cardTypeFilter');unshadowedEnum(type,ctx,'WildcardCardType');
            if(literal(type,ctx)!=='unit')reject();other(resolver.props.get('cardCondition'),false);
            const heal=plainObject(helper(resolver.props.get('immediateEffect'),'immediateEffects','heal'));
            if(!keys(heal,['amount'])||literal(heal.props.get('amount'),ctx)!==3)reject();shape='forceHeal';
        } else if(keys(resolver,['cardCondition','immediateEffect'])) {
            other(resolver.props.get('cardCondition'),true);
            const phase=plainObject(helper(resolver.props.get('immediateEffect'),'immediateEffects','forThisPhaseCardEffect'));
            if(!keys(phase,['effect']))reject();
            const stats=plainObject(helper(phase.props.get('effect'),'ongoingEffects','modifyStats'));
            if(!keys(stats,['power','hp'])||literal(stats.props.get('power'),ctx)!==3||literal(stats.props.get('hp'),ctx)!==0)reject();shape='nonVehiclePhase';
        } else reject();
    } else if(keys(obj,['title','immediateEffect'])) {
        const factory=helper(obj.props.get('immediateEffect'),'immediateEffects','discardCardsFromOwnHand');
        const {body,names:[context]}=simpleCallback(factory,1),discard=plainObject(body);
        if(!keys(discard,['amount','target'])||literal(discard.props.get('amount'),ctx)!==1||!pathIs(discard.props.get('target'),[context,'player']))reject();shape='discard';
    } else reject();
    // The first cohort has no setup snapshots or live gain conditions. Verify
    // its whole registration and attachment syntax before emitting provenance.
    const statements=setup.body.statements;
    if(statements.length!==(shape==='discard'?1:2))reject();
    if(shape!=='discard') {
        const first=statements[0];
        if(!ts.isExpressionStatement(first)||!ts.isCallExpression(first.expression)||first.expression.questionDotToken||first.expression.arguments.length!==1||!pathIs(first.expression.expression,[setup.parameters[0].name.text,'setAttachCondition']))reject();
        const {body,names:[context]}=simpleCallback(first.expression.arguments[0],1);
        const trait=shape==='forceHeal'?'force':'vehicle';let query=body;
        if(shape==='nonVehiclePhase'){if(!ts.isPrefixUnaryExpression(query)||query.operator!==ts.SyntaxKind.ExclamationToken)reject();query=unwrap(query.operand);}
        if(!ts.isCallExpression(query)||query.questionDotToken||query.arguments.length!==1||!pathIs(query.expression,[context,'attachTarget','hasSomeTrait']))reject();
        unshadowedEnum(query.arguments[0],ctx,'Trait');if(literal(query.arguments[0],ctx)!==trait)reject();
    }
    const result=ability(input,ctx,'onAttack');result.gainedHost=true;
    return result;
}
function gainedHostSourceShape(out) {
    if(out.baseClass!=='UpgradeCard'||out.abilities.card.length!==1||out.abilities.leader.length||out.abilities.unit.length)return false;
    const a=out.abilities.card[0],same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
    const base={trigger:'onAttack',title:a.title,costs:[]},other={kind:'binary',op:'!==',left:reference('card'),right:reference('source')};
    const attach=trait=>({kind:'query',name:'hasSomeTrait',receiver:reference('card'),args:[constant(trait)]});
    return same(a,{...base,optional:true,targetResolver:{owner:'any',type:'unit',zone:'anyAttackable',predicate:other},effect:{kind:'heal',target:'chosen',amount:constant(3)},gainedHost:true})&&same(out.attachCondition,attach('force')) ||
        same(a,{...base,optional:true,targetResolver:{owner:'any',type:'any',zone:'anyAttackable',predicate:{kind:'binary',op:'&&',left:other,right:{kind:'query',name:'isUnit',receiver:reference('card'),args:[]}}},effect:{kind:'modifyStats',target:'chosen',duration:'phase',power:constant(3),hp:constant(0)},gainedHost:true})&&same(out.attachCondition,{kind:'unary',op:'!',value:attach('vehicle')}) ||
        same(a,{...base,effect:{kind:'discardOwnHand',amount:constant(1),target:'ownPlayer'},gainedHost:true})&&!out.attachCondition;
}
function attachedConstantAbility(input,context,host) {
    const obj=object(input,context);const allowed=host?['title','condition','matchTarget','ongoingEffect']:['title','condition','ongoingEffect'];
    for(const [key,value]of obj.props)if(!allowed.includes(key))fail(obj.ctx,value,'unsupported-attached-constant-field:'+key);
    const result=constantAbility(input,obj.ctx,obj);result.attachedContext=host?'host':'upgrade';
    const check=e=>{if(e.effects){e.effects.forEach(check);return;}if(e.kind!=='gainKeyword'||!['grit','sentinel','restore','raid','overwhelm'].includes(e.keyword)||e.amount.kind!=='const'||!Number.isSafeInteger(e.amount.value)||e.amount.value<1||e.amount.value>2)fail(obj.ctx,obj.node,'attached-constant-installed-keyword-only');if(!host)e.target='parent';};check(result.effect);
    return result;
}
function attachedKeywordAbility(input, context) {
    const obj = object(input, context);
    // Bounty grants retain the entire nested collection registration as IR.
    // Other keywords retain the narrower installed-keyword shape.
    if(obj.props.has('ability')) {
        for(const [key,value]of obj.props)if(!['keyword','ability','gainCondition'].includes(key))fail(obj.ctx,value,'unsupported-attached-bounty-field:'+key);
        const grant={...obj,props:new Map([...obj.props].filter(([key])=>key!=='gainCondition'))};
        const result={trigger:'constant',title:'Give Bounty to the attached unit',costs:[],effect:bountyGrant(grant,'persistent','parent',{ability,literal,fail})};
        if(obj.props.has('gainCondition'))result.condition=expr(obj.props.get('gainCondition'),{...obj.ctx,attachedParentTitle:true});
        return result;
    }
    // Every extra installed-keyword field changes semantics and is rejected.
    for (const [key, value] of obj.props) if (!['keyword', 'amount', 'gainCondition'].includes(key)) fail(obj.ctx, value, 'unsupported-attached-keyword-field:' + key);
    if (!obj.props.has('keyword')) fail(obj.ctx, obj.node, 'missing-modifier-keyword');
    const keywordProps = ts.factory.createObjectLiteralExpression([...obj.props].filter(([key]) => key !== 'gainCondition').map(([key, value]) => ts.factory.createPropertyAssignment(key, value)));
    const helper = ts.factory.createCallExpression(ts.factory.createPropertyAccessExpression(ts.factory.createPropertyAccessExpression(ts.factory.createIdentifier('AbilityHelper'), 'ongoingEffects'), 'gainKeyword'), undefined, [keywordProps]);
    // Synthetic helper nodes retain the real source span for fail-closed diagnostics.
    ts.setTextRange(keywordProps, obj.node);
    ts.setTextRange(helper, obj.node);
    const props = new Map([['ongoingEffect', helper]]);
    if (obj.props.has('gainCondition')) props.set('condition', obj.props.get('gainCondition'));
    const result = constantAbility(input, { ...obj.ctx, attachedParentTitle:true }, { ...obj, ctx:{...obj.ctx,attachedParentTitle:true}, props });
    result.effect.target = 'parent';
    const keyword = result.effect.keyword;
    const amount = result.effect.amount;
    if (amount.kind !== 'const' || !Number.isSafeInteger(amount.value) || amount.value < 0 || amount.value > 5) fail(obj.ctx, obj.node, 'attached-keyword-amount-bound');
    result.title = `Give ${keyword[0].toUpperCase() + keyword.slice(1)}${['raid', 'restore'].includes(keyword) ? ' ' + amount.value : ''} to the attached unit`;
    return result;
}
function attachPredicate(input, ctx) {
    const value = expr(input, { ...ctx, attachPredicate:true });
    const visit = (v) => {
        if (!v || typeof v !== 'object') return;
        if (v.kind === 'ref' && v.path[0] === 'attachTarget') v.path = ['card', ...v.path.slice(1)];
        Object.values(v).forEach(visit);
    };
    visit(value);
    return value;
}
function namedResolvers(input, context) {
    if(context.namedTargets?.size) fail(context,input,'nested-named-resolver-scope');
    const obj=object(input,context);
    if(!obj.props.size||obj.props.size>4) fail(obj.ctx,obj.node,'named-target-slot-bound');
    const descriptors=[];const slots=new Map();const dependents=new Set();
    for(const [name,value] of obj.props) {
        if(!/^[A-Za-z_][A-Za-z0-9_]{0,31}$/.test(name)||['__proto__','prototype','constructor'].includes(name)) fail(obj.ctx,value,'invalid-named-target-name');
        const resolver=object(value,obj.ctx);
        const allowed=['cardTypeFilter','controller','zoneFilter','cardCondition','optional','mode','numCards','canChooseNoCards','immediateEffect','choosingPlayer','dependsOn','activePromptTitle'];
        for(const [key,v]of resolver.props)if(!allowed.includes(key))fail(resolver.ctx,v,'unsupported-named-resolver-field:'+key);
        const mode=resolver.props.has('mode')?literal(resolver.props.get('mode'),resolver.ctx):'single';
        if(!['single','exactly'].includes(mode))fail(resolver.ctx,value,'named-target-mode');
        if(mode==='exactly'&&(!resolver.props.has('numCards')||literal(resolver.props.get('numCards'),resolver.ctx)!==1))fail(resolver.ctx,value,'named-target-exactly-one');
        if(mode==='single'&&resolver.props.has('numCards'))fail(resolver.ctx,value,'named-target-single-count');
        for(const key of ['optional','canChooseNoCards'])if(resolver.props.has(key)&&typeof literal(resolver.props.get(key),resolver.ctx)!=='boolean')fail(resolver.ctx,value,'named-target-optionality');
        const chooser=resolver.props.has('choosingPlayer')?literal(resolver.props.get('choosingPlayer'),resolver.ctx):'self';
        if(!['self','opponent'].includes(chooser))fail(resolver.ctx,value,'named-target-chooser');
        let dependsOn;
        if(resolver.props.has('dependsOn')) {
            dependsOn=literal(resolver.props.get('dependsOn'),resolver.ctx);
            if(!slots.has(dependsOn))fail(resolver.ctx,value,'named-target-backward-dependency');
            if(dependents.has(dependsOn))fail(resolver.ctx,value,'named-target-multiple-dependents');
            dependents.add(dependsOn);
        }
        const scope=new Map(slots);scope.set(name,{array:mode==='exactly'});
        const ctx={...resolver.ctx,namedTargets:scope};
        const props=new Map(resolver.props);props.delete('choosingPlayer');props.delete('dependsOn');
        const spec=targetResolverProperties({...resolver,props,ctx});
        if(!['any','unit','nonLeaderUnit','leaderUnit','tokenUnit','nonTokenUnit','basicUnit','base'].includes(spec.type)||!['arena','anyAttackable','base'].includes(spec.zone))fail(ctx,value,'named-target-card-domain');
        if(resolver.props.has('activePromptTitle'))spec.promptTitle=displayText(resolver.props.get('activePromptTitle'),ctx);
        const entry={name,targetResolver:spec,chooser,...(dependsOn?{dependsOn}:{})};
        if(resolver.props.has('immediateEffect'))entry.effect=effect(resolver.props.get('immediateEffect'),ctx,'chosen');
        const ancestorNames=new Set();let ancestor=dependsOn;
        while(ancestor){ancestorNames.add(ancestor);ancestor=descriptors.find(d=>d.name===ancestor)?.dependsOn;}
        const check=(v)=>{
            if(!v||typeof v!=='object')return;
            if(v.kind==='ref'&&v.path[0]==='namedTargets'&&v.path[1]!==name&&!ancestorNames.has(v.path[1]))fail(ctx,value,'named-target-undeclared-dependency');
            if(v.kind==='ref'&&['event','targets','target'].includes(v.path[0]))fail(ctx,value,'named-target-unsupported-capture');
            Object.values(v).forEach(check);
        };
        check(entry);
        const checkEffect=e=>{
            if(!e)return;
            if(e.kind==='simultaneous'||e.kind==='phaseModifiers') {if(!e.effects.length||e.effects.length>8)fail(ctx,value,'named-effect-bound');e.effects.forEach(checkEffect);return;}
            if(!['damage','heal','ready','exhaust','defeat','capture','giveShield','giveExperience','modifyStats','gainKeyword','loseKeyword'].includes(e.kind))fail(ctx,value,'unsupported-named-effect:'+e.kind);
        };
        checkEffect(entry.effect);
        descriptors.push(entry);slots.set(name,{array:mode==='exactly'});
    }
    return {entries:descriptors,ctx:{...obj.ctx,namedTargets:slots}};
}
function normalizeSingletonExhaust(result) {
    const keys=(v,expected)=>Object.keys(v||{}).sort().join(',')===[...expected].sort().join(',');
    const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
    if(!keys(result,['trigger','title','costs','targetResolvers','effect','ifYouDo'])||result.trigger!=='event'||!same(result.costs,[])||!same(result.effect,{kind:'noAction'})||result.targetResolvers.length!==1)return result;
    const slot=result.targetResolvers[0],next=result.ifYouDo;
    if(!keys(slot,['name','targetResolver','chooser','effect'])||slot.chooser!=='self'||!same(slot.targetResolver,{owner:'self',type:'unit',zone:'anyAttackable'})||!same(slot.effect,{kind:'exhaust',target:'chosen'})||!keys(next,['title','costs','effect'])||!same(next.costs,[]))return result;
    const target={kind:'ref',path:['namedTargets',slot.name]};
    if(!same(next.effect,{kind:'simultaneous',effects:[{kind:'giveExperience',target,amount:{kind:'const',value:2}},{kind:'giveShield',target,amount:{kind:'const',value:1}}]}))return result;
    result.targetResolver=slot.targetResolver;result.effect=slot.effect;delete result.targetResolvers;
    next.effect.effects.forEach(effect=>{effect.target='chosen';});return result;
}
function ability(input, context, trigger = null, validatedObject = null) {
    if (!validatedObject) {
        const initiative = normalizeInitiativeUpTo(input, context, trigger, { ts, unwrap, object, literal, displayText, contextTitle, expr, effect });
        if (initiative) return initiative;
    }
    const obj = validatedObject || object(input, context);
    const {props,node}=obj;let ctx=obj.ctx;
    if(ctx.namedTargets?.size && (props.has('targetResolver')||props.has('targetResolvers')||props.has('cost'))) fail(ctx,node,'named-continuation-targeting-or-cost');
    knownProperties(obj, ['title', 'optional', 'cost', 'condition', 'immediateEffect', 'targetResolver', 'targetResolvers', 'ifYouDo', 'ifYouDoCondition', 'then', 'thenCondition', 'limit', 'optionalMakesAbilityNonCancelable','attackerMustSurvive']);
    const result = { ...(trigger ? { trigger } : {}), title: props.has('title') ? displayText(props.get('title'), ctx) : '', costs: costs(props.get('cost'), ctx) };
    if(props.has('contextTitle'))result.contextTitle=contextTitle(props.get('contextTitle'),ctx);
    if(props.has('attackerMustSurvive')){const value=literal(props.get('attackerMustSurvive'),ctx);if(trigger!=='whenAttackEnds'||typeof value!=='boolean')fail(ctx,node,'attack-end-survival-field');result.attackerMustSurvive=value;}
    if (props.has('optional')) result.optional = literal(props.get('optional'), ctx);
    if (props.has('condition')) result.condition = expr(props.get('condition'), ctx);
    if (props.has('optionalMakesAbilityNonCancelable')) result.optionalMakesAbilityNonCancelable = literal(props.get('optionalMakesAbilityNonCancelable'), ctx);
    if (props.has('limit')) fail(ctx, props.get('limit'), 'activation-limit');
    if(props.has('targetResolvers')) {
        if(props.has('targetResolver')||props.has('immediateEffect'))fail(ctx,node,'multiple-target-system-properties');
        if(props.has('cost')||props.has('limit'))fail(ctx,node,'named-target-cost-or-limit');
        const named=namedResolvers(props.get('targetResolvers'),ctx);
        result.targetResolvers=named.entries;ctx=named.ctx;
        // Sentinel for absence of a direct parent system. An absent child
        // system remains absent because it controls dependency feasibility.
        result.effect={kind:'noAction'};
        if(!named.entries.some(e=>e.effect)&&!props.has('then')&&!props.has('ifYouDo'))fail(ctx,node,'named-ability-without-effect');
    }
    if (props.has('targetResolver')) {
        if (props.has('immediateEffect')) fail(ctx, node, 'both-targeted-and-direct-effects');
        const resolver = object(props.get('targetResolver'), ctx);
        knownProperties(resolver, ['cardTypeFilter', 'controller', 'zoneFilter', 'cardCondition', 'optional', 'mode', 'numCards', 'canChooseNoCards', 'immediateEffect', 'choosingPlayer']);
        const aspectIncludesAlias=trigger==='whenDefeated'&&result.optional===true&&result.costs.length===0&&!props.has('condition')&&!props.has('then')&&!props.has('ifYouDo');
        result.targetResolver = targetResolverProperties({...resolver,ctx:{...resolver.ctx,aspectIncludesAlias}});
        if (resolver.props.has('immediateEffect')) {
            const otherBaseTargetScope=result.targetResolver.type==='base'&&!result.targetResolver.mode&&!result.targetResolver.count;
            const damageRecipientScope=['unit','nonLeaderUnit','tokenUnit'].includes(result.targetResolver.type)&&!result.targetResolver.mode&&!result.targetResolver.count;
            result.effect = effect(resolver.props.get('immediateEffect'), {...resolver.ctx,otherBaseTargetScope,damageRecipientScope}, 'chosen');
        }
    }
    if (props.has('immediateEffect')) {
        if (result.effect) fail(ctx, node, 'both-targeted-and-direct-effects');
        result.effect = effect(props.get('immediateEffect'), ctx, result.targetResolver ? 'chosen' : 'self');
    }
    if (!result.effect) fail(ctx, node, 'ability-without-effect');
    for (const branch of ['then', 'ifYouDo']) if (props.has(branch)) result[branch] = ability(props.get(branch), ctx);
    if(result.targetResolver&&ctx.powerMaximumSource&&props.has('targetResolver')) {
        const resolver=object(props.get('targetResolver'),ctx);
        if(resolver.props.has('activePromptTitle'))result.targetResolver.promptTitleParts=contextTitle(resolver.props.get('activePromptTitle'),{...resolver.ctx,powerMaximumTitle:true});
    }
    if(result.targetResolver&&ctx.reductionHelpers?.size&&props.has('targetResolver')) {
        const resolver=object(props.get('targetResolver'),ctx);
        if(resolver.props.has('activePromptTitle'))result.targetResolver.promptTitleParts=contextTitle(resolver.props.get('activePromptTitle'),resolver.ctx);
    }
    if(result.targetResolver&&(ctx.inventoryUsage?.size||props.has('attackerMustSurvive'))&&props.has('targetResolver')){const resolver=object(props.get('targetResolver'),ctx);if(resolver.props.has('activePromptTitle'))result.targetResolver.promptTitleParts=contextTitle(resolver.props.get('activePromptTitle'),resolver.ctx);}
    // A top-deck continuation event retains its explicit selector prompt. Other
    // scalar source families keep their existing representation byte for byte.
    if(result.targetResolver&&result.ifYouDo?.effect?.kind==='resourceCard'&&result.ifYouDo.effect.target==='ownTopDeck') {
        const resolver=object(props.get('targetResolver'),ctx);
        if(resolver.props.has('activePromptTitle')) {
            const n=unwrap(resolver.props.get('activePromptTitle'));
            const staticSpan=span=>{
                const parts=access(span.expression,resolver.ctx);
                let shadowed=false;for(let ancestor=span.expression.parent;ancestor;ancestor=ancestor.parent){if(ancestor.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===parts[0]))shadowed=true;}
                if(shadowed||resolver.ctx.scope.has(parts[0])||resolver.ctx.constants.has(parts[0]))fail(resolver.ctx,span.expression,'top-deck-resource-prompt-binding');
                return titleHelper(span.expression,resolver.ctx)!==null;
            };
            if(!(ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n)||ts.isTemplateExpression(n)&&n.templateSpans.every(staticSpan)))fail(resolver.ctx,n,'top-deck-resource-prompt');
            result.targetResolver.promptTitle=displayText(n,resolver.ctx);
        }
    }
    for (const condition of ['thenCondition', 'ifYouDoCondition']) if (props.has(condition)) result[condition] = expr(props.get(condition), ctx);
    // A factory closes over the previous effect context. Flattening a captured
    // target into the current-target register would change its value after a new
    // resolver runs (Dooku's readied-unit cost must not become the base's cost).
    // The current IR has no capture slots; reject ambiguous lifetimes explicitly.
    const factory = unwrap(input);
    if (ts.isArrowFunction(factory) || ts.isFunctionExpression(factory)) {
        let selection = false, multipleEffects = false, capturedTarget = false, capturedEvent = false, dynamicReference = false;
        const visit = (value) => {
            if (!value || typeof value !== 'object') return;
            if (value.targetResolver) selection = true;
            if (value.then || value.ifYouDo || (value.effects && value.effects.length > 1)) multipleEffects = true;
            if (value.kind === 'ref' && !(value.path[0]==='namedTargets'&&value.path.length===2)) dynamicReference = true;
            if (value.kind === 'ref' && ['target', 'targets'].includes(value.path[0])) capturedTarget = true;
            if (value.kind === 'ref' && value.path[0] === 'event') capturedEvent = true;
            Object.values(value).forEach(visit);
        };
        visit(result);
        if (capturedTarget && (selection || multipleEffects)) fail(ctx, factory, 'continuation-factory-target-lifetime', 'Factory target capture requires a separate value slot before subsequent targeting or effects');
        if (capturedEvent && multipleEffects) fail(ctx, factory, 'continuation-factory-event-lifetime', 'Factory event capture requires a separate value slot before subsequent effects');
        if (dynamicReference && multipleEffects) fail(ctx, factory, 'continuation-factory-value-lifetime', 'Factory values require capture slots before a sequence can change their referenced state');
    }
    let createsTokens = false, discardsHand = false, discardsDeck = false, capturesTokenEvent = false;
    const tokenCapture = (value) => {
        if (!value || typeof value !== 'object') return;
        if (value.kind === 'createTokenUnit') createsTokens = true;
        if (value.kind === 'discardOwnHand') discardsHand = true;
        if (value.kind === 'discardDeck') discardsDeck = true;
        if (value.kind === 'ref' && value.path[0] === 'event') capturesTokenEvent = true;
        Object.values(value).forEach(tokenCapture);
    };
    tokenCapture(result);
    if(ctx.namedTargets) {
        const check=v=> {
            if(!v||typeof v!=='object')return;
            if(v.kind==='ref'&&v.path[0]==='event')fail(ctx,node,'named-event-payload-capture');
            Object.values(v).forEach(check);
        };
        check(result);
    }
    if (createsTokens && capturesTokenEvent) fail(ctx, node, 'token-event-result-capture');
    if (discardsDeck && capturesTokenEvent) fail(ctx, node, 'deck-discard-event-result-capture');
    if (discardsHand && capturesTokenEvent) fail(ctx, node, 'hand-discard-event-result-capture');
    return normalizeSingletonExhaust(result);
}
// Standard hooks are one registration with alternative event names. Expand only
// static self hooks; event watchers and shared activation state need separate IR.
function standardTriggeredAbilities(input, context, validatedObject = null) {
    const obj = validatedObject || object(input, context);
    if (!obj.props.has('when')) fail(obj.ctx, obj.node, 'trigger-alias-without-when');
    const whenProperty = obj.node.properties.find(p => ts.isPropertyAssignment(p) && (p.name.text === 'when'));
    if (!whenProperty) fail(obj.ctx, obj.node, 'trigger-alias-static-when');
    const whenNode = unwrap(obj.props.get('when'));
    if (!ts.isObjectLiteralExpression(whenNode)) fail(obj.ctx, whenNode, 'trigger-alias-static-when');
    const when = object(whenNode, obj.ctx);
    if (!when.props.size || when.props.size > 3) fail(when.ctx, when.node, 'trigger-alias-hook-count');
    for (const [hook, value] of when.props) {
        if (!['whenPlayed', 'onAttack', 'whenDefeated'].includes(hook)) fail(when.ctx, value, 'trigger-alias-hook:' + hook);
        if (unwrap(value).kind !== ts.SyntaxKind.TrueKeyword) fail(when.ctx, value, 'trigger-alias-true-only');
    }
    if (obj.props.has('cost')) fail(obj.ctx, obj.props.get('cost'), 'trigger-alias-cost');
    const props = new Map(obj.props);
    props.delete('when');
    return [...when.props.keys()].map((trigger) => ability(input, obj.ctx, trigger, { ...obj, props }));
}
// Coordinate grants this definition while its printed keyword is active.
// Runtime retains the marker separately from condition: attack registration
// is captured before resolution, whereas constant gates remain live.
function coordinateAbility(input, context) {
    const obj = object(input, context);
    if (!obj.props.has('type')) fail(obj.ctx, obj.node, 'coordinate-without-type');
    const type = literal(obj.props.get('type'), obj.ctx);
    const props = new Map(obj.props);
    props.delete('type');
    let result;
    if (type === 'constant') result = constantAbility(input, obj.ctx, { ...obj, props });
    else if (type === 'triggered') {
        if (!props.has('when')) fail(obj.ctx, obj.node, 'coordinate-without-when');
        const when = object(props.get('when'), obj.ctx);
        if (when.props.size !== 1 || !when.props.has('onAttack')) fail(obj.ctx, obj.node, 'coordinate-trigger');
        [result] = standardTriggeredAbilities(input, obj.ctx, { ...obj, props });
    } else fail(obj.ctx, obj.props.get('type'), 'coordinate-type:' + type);
    return { ...result, coordinate: true };
}
function loadEnums(sourceRoot) {
    const out = {};
    const root = path.join(sourceRoot, 'game/core');
    if (!fs.existsSync(root)) return out;
    for (const relative of fs.readdirSync(root, { recursive: true }).filter((f) => f.endsWith('.ts')).sort()) {
        const file = path.join(root, relative);
        const sf = ts.createSourceFile(file, fs.readFileSync(file, 'utf8'), ts.ScriptTarget.Latest, true);
        function walk(n) {
            if (ts.isEnumDeclaration(n)) for (const member of n.members) {
                if (member.initializer && (ts.isStringLiteral(member.initializer) || ts.isNumericLiteral(member.initializer))) out[n.name.text + '.' + member.name.getText(sf)] = ts.isStringLiteral(member.initializer) ? member.initializer.text : Number(member.initializer.text);
            }
            ts.forEachChild(n, walk);
        }
        walk(sf);
    }
    return out;
}
function compileSource(text, file, enums = {}) {
    const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
    const ctx = { source, file, enums, scope: new Map(), constants: new Map(), constantScopes: new Map(), constantEnvs: new Map(), constantStages: new Map(), bindings: [], usedConstants: new Set(), inventoryUsage:new Set() };
    const diagnostics = [];
    const out = { sourcePath: file, sourceHash: hash(text), abilities: { leader: [], unit: [], card: [] }, diagnostics };
    if (source.parseDiagnostics.length) {
        for (const diagnostic of source.parseDiagnostics) {
            const location = source.getLineAndCharacterOfPosition(diagnostic.start || 0);
            diagnostics.push({ code: 'INVALID_TYPESCRIPT', pattern: 'typescript-parse-error', file, line: location.line + 1, column: location.character + 1, message: ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n') });
        }
    }
    for (const statement of source.statements) {
        if (ts.isImportDeclaration(statement) || ts.isClassDeclaration(statement) || ts.isInterfaceDeclaration(statement) || ts.isTypeAliasDeclaration(statement) || ts.isEmptyStatement(statement)) continue;
        diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'module-statement:' + ts.SyntaxKind[statement.kind], file, line: source.getLineAndCharacterOfPosition(statement.getStart()).line + 1, column: 1, message: 'Module-level executable behavior requires conversion' });
    }
    const classes = source.statements.filter(ts.isClassDeclaration);
    if (classes.length !== 1) return { ...out, status: 'unsupported', diagnostics: [{ code: 'UNSUPPORTED_SOURCE', pattern: 'expected-one-card-class', file, line: 1, column: 1, message: `Found ${classes.length} classes` }] };
    const cls = classes[0];
    out.className = cls.name?.text;
    out.baseClass = cls.heritageClauses?.flatMap((c) => c.types.map((t) => t.expression.getText(source)))[0] || null;
    // JS uses the last own member with a given name. A grant template cannot
    // retain an earlier identity/setup body after an overriding member replaces
    // it. Check all member kinds before generic method traversal can skip them.
    let hasGainedHostRegistrar=false;
    const findGrant=n=>{if(ts.isPropertyAccessExpression(n)&&n.name.text==='addGainOnAttackAbilityTargetingAttached'||ts.isElementAccessExpression(n)&&n.argumentExpression&&(ts.isStringLiteral(n.argumentExpression)||ts.isNoSubstitutionTemplateLiteral(n.argumentExpression))&&n.argumentExpression.text==='addGainOnAttackAbilityTargetingAttached')hasGainedHostRegistrar=true;ts.forEachChild(n,findGrant);};findGrant(cls);
    const memberName=member=>member.name&&(ts.isIdentifier(member.name)||ts.isStringLiteral(member.name)||ts.isNoSubstitutionTemplateLiteral(member.name))?member.name.text:null;
    const checkGainedHostSlots=()=>{
        if(cls.members.some(m=>m.name&&ts.isComputedPropertyName(m.name)))fail(ctx,cls,'gained-host-computed-member');
        for(const name of ['getImplementationId',...Object.keys(SIDE_METHODS)]) {
            const members=cls.members.filter(m=>memberName(m)===name);
            if((['getImplementationId','setupCardAbilities'].includes(name)&&members.length!==1)||(name!=='setupCardAbilities'&&name in SIDE_METHODS&&members.length)||members.length>1||members.some(m=>!ts.isMethodDeclaration(m)))fail(ctx,cls,'gained-host-class-slot:'+name);
        }
    };
    if(hasGainedHostRegistrar) {
        try {checkGainedHostSlots();}
        catch(error) {if(!(error instanceof Unsupported))throw error;diagnostics.push(error.diagnostic);out.status='unsupported';return out;}
    }
    ctx.powerMaximumSource=hasRawLivePowerMaximum(cls,{ts,unwrap});
    ctx.maximumHelpers=out.baseClass==='LeaderUnitCard'?inspectMaximumHelpers(cls,ctx,{ts,unwrap,literal,unshadowedEnum}):new Map();
    ctx.physicalAttachmentHelpers=inspectPhysicalAttachmentHelpers(cls,ctx,{ts,unwrap,literal,unshadowedEnum});
    ctx.resultHelpers=inspectResultHelpers(cls,ctx,{ts,unwrap});
    ctx.defeatedUnitCountHelpers=new Map();
    if(out.baseClass==='EventCard')for(const member of cls.members){const helper=defeatedUnitCountHelper(member,{ts,unwrap});if(helper&&cls.members.filter(m=>m.name?.getText(source)===member.name.getText(source)).length===1)ctx.defeatedUnitCountHelpers.set(member.name.text,helper);}
    ctx.reductionHelpers=inspectReductionHelpers(cls,{...ctx,baseClass:out.baseClass},{ts,unwrap,expr,fail});
    ctx.arenaDamageHelpers=new Map();
    if(['EventCard','NonLeaderUnitCard'].includes(out.baseClass))for(const member of cls.members){
        const damage=arenaDamageHelper(member);
        const damageClass=damage&&((out.baseClass==='EventCard'&&damage.multiplier===2)||(out.baseClass==='NonLeaderUnitCard'&&damage.multiplier===1));
        const helper=damageClass?damage:out.baseClass==='NonLeaderUnitCard'?(spaceCountComparisonHelper(member)||filteredArenaCountHelper(member)):null;
        if(helper&&cls.members.filter(m=>m.name?.getText(source)===member.name.getText(source)).length===1)ctx.arenaDamageHelpers.set(member.name.text,helper);
    }
    for (const member of cls.members) {
        if (ts.isMethodDeclaration(member) && member.name.getText(source) === 'getImplementationId') {
            const returned = member.body?.statements.find(ts.isReturnStatement)?.expression;
            try {
                if (isNonSynchronous(member)) fail(ctx,member,'non-synchronous-implementation-identity');
                if (hasDecorators(member) || member.parameters.some(p=>!ts.isIdentifier(p.name)||unsupportedParameterBinding(p))) fail(ctx,member,'unsupported-implementation-identity-binding');
                const id = object(returned, ctx);
                out.id = literal(id.props.get('id'), ctx);
                out.internalName = literal(id.props.get('internalName'), ctx);
            } catch (e) { if (e instanceof Unsupported) diagnostics.push(e.diagnostic); else throw e; }
        }
    }
    if (!out.id || !out.internalName) diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'missing-implementation-identity', file, line: 1, column: 1, message: 'Card does not expose a constant implementation identity' });
    const supportedBases = new Set(['NonLeaderUnitCard', 'LeaderUnitCard', 'BaseCard', 'EventCard', 'UpgradeCard', 'TokenUpgradeCard', 'TokenUnitCard']);
    if (!supportedBases.has(out.baseClass)) diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'custom-inheritance:' + out.baseClass, file, line: 1, column: 1, message: 'Inherited custom behavior requires conversion' });
    for (const member of cls.members) {
        const name = member.name?.getText(source);
        if (name === 'getImplementationId') continue;
        if (ts.isPropertyDeclaration(member)) {
            if (name === 'overrideNotImplemented') diagnostics.push({ code: 'ENGINE_UNIMPLEMENTED', pattern: 'engine-override-not-implemented', file, line: source.getLineAndCharacterOfPosition(member.getStart()).line + 1, column: 1, message: 'Engine marks this definition unimplemented' });
            else diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'class-state:' + name, file, line: source.getLineAndCharacterOfPosition(member.getStart()).line + 1, column: 1, message: 'Class-local persistent state requires conversion' });
            continue;
        }
        if (!ts.isMethodDeclaration(member)) {
            diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'class-member:' + ts.SyntaxKind[member.kind], file, line: source.getLineAndCharacterOfPosition(member.getStart()).line + 1, column: 1, message: 'Additional class behavior requires conversion' });
            continue;
        }
        if (!(name in SIDE_METHODS)) {
            if(ctx.resultHelpers.get(name)?.member===member || ctx.maximumHelpers.get(name)?.member===member || ctx.defeatedUnitCountHelpers.get(name)?.member===member || ctx.reductionHelpers.get(name)?.member===member || ctx.arenaDamageHelpers.get(name)?.member===member || ctx.physicalAttachmentHelpers.get(name)?.member===member)continue;
            diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'method:' + name, file, line: source.getLineAndCharacterOfPosition(member.getStart()).line + 1, column: 1, message: 'Additional class method requires conversion' });
            continue;
        }
        if (isNonSynchronous(member)) {
            diagnostics.push(new Unsupported(ctx,member,'non-synchronous-setup-method').diagnostic);
            continue;
        }
        if (hasDecorators(member) || member.parameters.some(p=>!ts.isIdentifier(p.name)||unsupportedParameterBinding(p))) {
            diagnostics.push(new Unsupported(ctx,member,'unsupported-setup-parameter-binding').diagnostic);
            continue;
        }
        const local = { ...ctx, constants: new Map() };
        for (const statement of member.body?.statements ?? []) {
            try {
                if (ts.isVariableStatement(statement)) {
                    if (!(statement.declarationList.flags & ts.NodeFlags.Const)) fail(local, statement, 'mutable-setup-binding');
                    for (const d of statement.declarationList.declarations) {
                        if (!ts.isIdentifier(d.name) || !d.initializer) fail(local, d, 'complex-setup-binding');
                        bindConstant(local, d.name.text, d.initializer);
                    }
                    continue;
                }
                if (!ts.isExpressionStatement(statement) || !ts.isCallExpression(statement.expression)) fail(local, statement, 'setup-statement:' + ts.SyntaxKind[statement.kind]);
                const call = statement.expression;
                const parts = access(call.expression, local);
                const registration = parts.at(-1);
                if (parts.length !== 2 || parts[0] !== member.parameters[0]?.name.getText(source) || (!TRIGGERS[registration] && registration !== 'addConstantAbility' && registration !== 'addGainOnAttackAbilityTargetingAttached' && registration !== 'addTriggeredAbility' && registration !== 'addBountyAbility' && registration !== 'addCoordinateAbility' && registration !== 'addGainKeywordTargetingAttached' && registration !== 'setAttachCondition' && registration !== 'addConstantAbilityTargetingAttached' && registration !== 'addGainConstantAbilityTargetingAttached')) fail(local, call, 'registrar:' + registration);
                if (call.arguments.length !== 1) fail(local, call, 'registration-arity');
                if (['addGainKeywordTargetingAttached', 'setAttachCondition'].includes(registration)) {
                    if (!['UpgradeCard', 'TokenUpgradeCard'].includes(out.baseClass) || name !== 'setupCardAbilities') fail(local, call, 'attached-source-side');
                    if (registration === 'setAttachCondition') {
                        if (out.attachCondition) fail(local, call, 'duplicate-attach-condition');
                        out.attachCondition = attachPredicate(call.arguments[0], local);
                    } else out.abilities.card.push(attachedKeywordAbility(call.arguments[0], local));
                }
                else if (['addConstantAbilityTargetingAttached','addGainConstantAbilityTargetingAttached'].includes(registration)) {
                    if(out.baseClass!=='UpgradeCard'||name!=='setupCardAbilities')fail(local,call,'attached-source-side');
                    out.abilities.card.push(attachedConstantAbility(call.arguments[0],local,registration==='addGainConstantAbilityTargetingAttached'));
                }
                else if(registration==='addGainOnAttackAbilityTargetingAttached') {
                    try{out.abilities.card.push(gainedHostOnAttack(call.arguments[0],local,member,call));}
                    catch(error){if(!(error instanceof Unsupported))throw error;fail(local,call,'registrar:addGainOnAttackAbilityTargetingAttached');}
                }
                else if (registration === 'addCoordinateAbility') {
                    if (out.baseClass !== 'NonLeaderUnitCard' || name !== 'setupCardAbilities') fail(local, call, 'coordinate-source-side');
                    if (out.abilities.card.some(a => a.coordinate)) fail(local, call, 'coordinate-multiple-definitions');
                    out.abilities.card.push(coordinateAbility(call.arguments[0], local));
                }
                else if (registration === 'addBountyAbility') {
                    if(out.baseClass!=='NonLeaderUnitCard'||name!=='setupCardAbilities')fail(local,call,'bounty-source-side');
                    out.abilities.card.push(bountyAbility(call.arguments[0],local,{ability,fail}));
                }
                else if (registration === 'addTriggeredAbility') out.abilities[SIDE_METHODS[name]].push(...standardTriggeredAbilities(call.arguments[0], local));
                else out.abilities[SIDE_METHODS[name]].push(registration === 'addConstantAbility' ? constantAbility(call.arguments[0], local) : ability(call.arguments[0], local, TRIGGERS[registration]));
            } catch (e) { if (e instanceof Unsupported) diagnostics.push(e.diagnostic); else throw e; }
        }
    }
    for (const binding of ctx.bindings) if (!ctx.usedConstants.has(binding.node)) {
        const location = source.getLineAndCharacterOfPosition(binding.node.getStart(source));
        diagnostics.push({ code: 'UNSUPPORTED_SOURCE', pattern: 'unused-source-binding', file, line: location.line + 1, column: location.character + 1, message: `Unused binding ${binding.name} may contain unconverted behavior` });
    }
    for(const [name,helper]of ctx.reductionHelpers)if(!helper.used)diagnostics.push(new Unsupported(ctx,helper.member,'unused-reduction-helper:'+name).diagnostic);
    for(const [name,helper]of ctx.maximumHelpers)if(!helper.used)diagnostics.push(new Unsupported(ctx,helper.member,'unused-maximum-helper:'+name).diagnostic);
    for(const [name,helper]of ctx.resultHelpers)if(!helper.used)diagnostics.push(new Unsupported(ctx,helper.member,'unused-result-helper:'+name).diagnostic);
    for(const [name,helper]of ctx.defeatedUnitCountHelpers)if(!helper.used)diagnostics.push(new Unsupported(ctx,helper.member,'unused-resolved-defeat-helper:'+name).diagnostic);
    for(const [name,helper]of ctx.arenaDamageHelpers)if(!helper.used)diagnostics.push(new Unsupported(ctx,helper.member,'unused-arena-damage-helper:'+name).diagnostic);
    if(out.abilities.card.some(a=>a.gainedHost)) {
        try{
            // Validate provenance regardless of how the registrar callee was spelled.
            checkGainedHostSlots();
            if(source.statements.some(s=>s!==cls&&(!ts.isImportDeclaration(s)||!ts.isStringLiteral(s.moduleSpecifier)||!['../../../AbilityHelper','../../../core/card/AbilityRegistrationInterfaces','../../../core/card/UpgradeCard','../../../core/Constants'].includes(s.moduleSpecifier.text)||!s.importClause)))fail(ctx,cls,'gained-host-module-shape');
            const imported=(name,suffix)=>{
                const bindings=[];
                for(const s of source.statements)if(ts.isImportDeclaration(s)&&s.importClause){
                    const c=s.importClause;if(c.name?.text===name)bindings.push(false);
                    if(c.namedBindings&&ts.isNamespaceImport(c.namedBindings)&&c.namedBindings.name.text===name)bindings.push(false);
                    if(c.namedBindings&&ts.isNamedImports(c.namedBindings))for(const e of c.namedBindings.elements)if(e.name.text===name)bindings.push(!c.isTypeOnly&&!e.isTypeOnly&&(e.propertyName?.text||e.name.text)===name&&s.moduleSpecifier.text==='../../..'+suffix);
                }
                return bindings.length===1&&bindings[0];
            };
            if(!imported('UpgradeCard','/core/card/UpgradeCard'))fail(ctx,cls,'gained-host-import-binding');
            const usesNamespace=(node,name)=>ts.isPropertyAccessExpression(node)&&ts.isIdentifier(node.expression)&&node.expression.text===name||ts.forEachChild(node,child=>usesNamespace(child,name));
            for(const name of ['Trait','WildcardCardType'])if(usesNamespace(cls,name)&&!imported(name,'/core/Constants'))fail(ctx,cls,'gained-host-import-binding');
            const identities=cls.members.filter(m=>memberName(m)==='getImplementationId'),identity=identities[0];
            if(!cls.modifiers?.some(m=>m.kind===ts.SyntaxKind.ExportKeyword)||!cls.modifiers.some(m=>m.kind===ts.SyntaxKind.DefaultKeyword)||cls.modifiers.some(m=>m.kind===ts.SyntaxKind.DeclareKeyword)||hasDecorators(cls)||identities.length!==1||!identity||!ts.isMethodDeclaration(identity)||identity.modifiers?.some(m=>m.kind===ts.SyntaxKind.StaticKeyword)||identity.parameters.length||identity.body?.statements.length!==1||!ts.isReturnStatement(identity.body.statements[0]))fail(ctx,cls,'gained-host-identity-shape');
            const idValue=unwrap(identity.body.statements[0].expression);
            if(!ts.isObjectLiteralExpression(idValue)||idValue.properties.length!==2||idValue.properties.some(p=>!ts.isPropertyAssignment(p)||!['id','internalName'].includes(p.name.getText(source))||!(ts.isStringLiteral(unwrap(p.initializer))||ts.isNoSubstitutionTemplateLiteral(unwrap(p.initializer)))))fail(ctx,identity,'gained-host-identity-shape');
            topDeckContinuationNamespaces(cls,ctx);if(!gainedHostSourceShape(out))fail(ctx,cls,'gained-host-source-shape');
        }
        catch(error){if(!(error instanceof Unsupported))throw error;diagnostics.push(error.diagnostic);out.abilities={leader:[],unit:[],card:[]};}
    }
    const parentDefense=value=>!!value&&typeof value==='object'&&((value.kind==='query'&&value.name==='isDefending'&&JSON.stringify(value.receiver)===JSON.stringify(reference('source','parentUnit')))||Object.values(value).some(parentDefense));
    if(parentDefense(out.abilities)) {
        try{topDeckContinuationNamespaces(cls,ctx);if(!parentDefenseSourceShape(out))fail(ctx,cls,'parent-defense-source-shape');}
        catch(error){if(!(error instanceof Unsupported))throw error;diagnostics.push(error.diagnostic);out.abilities={leader:[],unit:[],card:[]};}
    }
    const containsTop=value=>!!value&&typeof value==='object'&&(value.target==='ownTopDeck'||Object.values(value).some(containsTop));
    if(containsTop(out.abilities)&&(out.baseClass==='EventCard'||out.abilities.card.some(a=>a.effect?.kind==='sequence'))){
        try{topDeckContinuationNamespaces(cls,ctx);}catch(error){if(!(error instanceof Unsupported))throw error;diagnostics.push(error.diagnostic);out.abilities={leader:[],unit:[],card:[]};}
    }
    if(containsTop(out.abilities)&&!topDeckResourceSourceShape(out)){
        diagnostics.push(new Unsupported(ctx,cls,'top-deck-resource-source-shape').diagnostic);
        out.abilities={leader:[],unit:[],card:[]};
    }
    out.status = diagnostics.length ? 'unsupported' : 'supported';
    return out;
}
module.exports = { SCHEMA, VERSION, ts, hash, compileSource, loadEnums, expr, ability, effect, Unsupported };
