'use strict';

// Compiler-only IR: preserve the lexical AbilityContext object and the time of
// the expression read. A future executor must bind these callback IDs; reading
// its current numeric result register is not an implementation of this node.
function contextBinding(node, ctx, { ts, fail }) {
    if (!ts.isIdentifier(node) || ctx.constants.has(node.text) || ctx.scope.get(node.text)?.path?.join('.') !== 'context') fail(ctx,node,'event-result-context-binding');
    for (let parent=node.parent;parent;parent=parent.parent) {
        if (!parent.parameters) continue;
        const index=parent.parameters.findIndex(p=>ts.isIdentifier(p.name)&&p.name.text===node.text);
        if(index<0)continue;
        const parameter=parent.parameters[index];
        if(parameter.initializer||parameter.dotDotDotToken||parameter.questionToken)fail(ctx,parameter,'event-result-context-parameter');
        let role='callback';
        const owner=parent.parent;
        if(ts.isPropertyAssignment(owner))role=owner.name.text||'callback';
        else if(ts.isCallExpression(owner)&&ts.isPropertyAccessExpression(owner.expression))role='properties:'+owner.expression.name.text;
        return {id:`callback:${parent.getStart(ctx.source)}:${index}`,role,bindAt:'callbackInvocation',readAt:'expressionEvaluation',reference:'lexicalContextObject'};
    }
    fail(ctx,node,'event-result-context-origin');
}
function pathOf(input,{ts,unwrap}) {
    const node=unwrap(input);
    if(ts.isIdentifier(node))return [node.text];
    if(ts.isPropertyAccessExpression(node)&&!node.questionDotToken){const p=pathOf(node.expression,{ts,unwrap});return p&&[...p,node.name.text];}
    return null;
}
function chain(input,{ts,unwrap}) {
    const node=unwrap(input);
    if(ts.isIdentifier(node))return {root:node,segments:[]};
    if(ts.isPropertyAccessExpression(node)) {const p=chain(node.expression,{ts,unwrap});return p&&{...p,segments:[...p.segments,{key:node.name.text,optional:!!node.questionDotToken}]};}
    if(ts.isElementAccessExpression(node)&&node.argumentExpression&&ts.isNumericLiteral(unwrap(node.argumentExpression))) {
        const key=Number(unwrap(node.argumentExpression).text);if(!Number.isSafeInteger(key)||key<0||key>31)return null;
        const p=chain(node.expression,{ts,unwrap});return p&&{...p,segments:[...p.segments,{key,optional:!!node.questionDotToken}]};
    }
    return null;
}
// Grouping can terminate optional-chain short circuiting. Keep this unverified
// syntax rejected rather than flattening it into an ungrouped event read.
function groupedOptionalEventAccess(input,api) {
    const {ts}=api;
    if(ts.isParenthesizedExpression(input)) {
        const parsed=chain(input.expression,api);
        return !!parsed?.segments.some(segment=>segment.optional);
    }
    return (ts.isPropertyAccessExpression(input)||ts.isElementAccessExpression(input))
        &&groupedOptionalEventAccess(input.expression,api);
}
function normalizeEventResultExpression(input,ctx,api) {
    const {ts,unwrap,expr,fail,unshadowedEnum}=api,node=unwrap(input);
    const parsed=chain(node,api);
    if(parsed&&['events','resolvedEvents'].includes(parsed.segments[0]?.key)) {
        if(groupedOptionalEventAccess(node,api))fail(ctx,node,'event-result-parenthesized-optional-boundary');
        if(!ctx.callbackStage)fail(ctx,node,'event-result-evaluation-stage');
        const [collection,index,...path]=parsed.segments;
        if(!index||typeof index.key!=='number'||!path.length)fail(ctx,node,'event-result-explicit-index-and-field');
        const keys=path.map(p=>p.key);
        // Preserve the existing exact scalar distribution-result IR and its
        // admitted programs. New identity/list/replacement reads use explicit IR.
        if(collection.key==='events'&&index.key===0&&keys.length===1&&keys[0]==='totalDistributed'&&!parsed.segments.some(s=>s.optional))return null;
        const allowed=keys.length===1&&['card','generatedTokens','willDefeat','availableExcessDamage','totalDistributed'].includes(keys[0])||
            keys.length===2&&keys[0]==='card'&&['printedCost','cost','title','type','aspects'].includes(keys[1])||
            keys.length===2&&keys[0]==='generatedTokens'&&typeof keys[1]==='number'||
            keys.length===2&&keys[0]==='lastKnownInformation'&&['power','hp','cost','card','arena','controller','parentCard','title','exhausted','type'].includes(keys[1])||
            keys.length===3&&keys[0]==='individualEvents'&&typeof keys[1]==='number'&&keys[2]==='card';
        if(!allowed)fail(ctx,node,'event-result-field-path');
        return {kind:'eventResult',contextBinding:contextBinding(parsed.root,ctx,api),collection:collection.key,eventIndex:index.key,path:keys,optionalAccess:parsed.segments.map((s,i)=>s.optional?i:null).filter(i=>i!==null)};
    }
    if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)) {
        const call=node.expression;
        if(call.name.text==='includes') {
            const p=chain(call.expression,api);
            if(p&&['events','resolvedEvents'].includes(p.segments[0]?.key)) {
                if(node.arguments.length!==1)fail(ctx,node,'event-result-membership-arity');
                return {kind:'resultIncludes',receiver:expr(call.expression,ctx),value:expr(node.arguments[0],ctx),optional:!!node.questionDotToken||!!call.questionDotToken};
            }
        }
        if(ts.isIdentifier(call.expression)&&call.expression.text==='EnumHelpers'&&['isUnit','isLeaderUnit','isNonLeaderUnit','isUpgrade'].includes(call.name.text)) {
            const p=node.arguments.length===1&&chain(node.arguments[0],api);
            if(p&&['events','resolvedEvents'].includes(p.segments[0]?.key)) {
                if(node.questionDotToken||call.questionDotToken)fail(ctx,node,'event-result-type-call');
                unshadowedEnum(call.expression,ctx,'EnumHelpers');
                return {kind:'resultCardType',test:call.name.text,value:expr(node.arguments[0],ctx)};
            }
        }
    }
    return null;
}
function inspectResultHelpers(cls,ctx,api) {
    const {ts,unwrap}=api,helpers=new Map();
    const path=n=>pathOf(n,api)?.join('.');
    const call=(n,name)=>ts.isCallExpression(n)&&!n.questionDotToken&&path(n.expression)===name;
    for(const member of cls.members) {
        if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||member.asteriskToken||member.questionToken||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||ts.canHaveDecorators(member)&&ts.getDecorators(member)?.length||member.parameters.length!==1||!ts.isIdentifier(member.parameters[0].name)||member.parameters[0].initializer||member.parameters[0].dotDotDotToken||member.parameters[0].questionToken)continue;
        if(cls.members.filter(m=>m.name?.getText(ctx.source)===member.name.text).length!==1)continue;
        const parameter=member.parameters[0].name.text,statements=member.body?.statements;
        if(statements?.length===1&&ts.isReturnStatement(statements[0])&&statements[0].expression) {
            const reduce=unwrap(statements[0].expression);
            if(!ts.isCallExpression(reduce)||reduce.questionDotToken||!ts.isPropertyAccessExpression(reduce.expression)||reduce.expression.questionDotToken||reduce.expression.name.text!=='reduce'||reduce.arguments.length!==2||!ts.isNumericLiteral(reduce.arguments[1])||reduce.arguments[1].text!=='0')continue;
            const filter=unwrap(reduce.expression.expression),reducer=unwrap(reduce.arguments[0]);
            if(!call(filter,parameter+'.resolvedEvents.filter')||filter.arguments.length!==1||!ts.isArrowFunction(filter.arguments[0])||filter.arguments[0].modifiers?.length||filter.arguments[0].parameters.length!==1||!ts.isIdentifier(filter.arguments[0].parameters[0].name)||filter.arguments[0].parameters[0].initializer||filter.arguments[0].parameters[0].dotDotDotToken||filter.arguments[0].parameters[0].questionToken)continue;
            const predicate=filter.arguments[0],event=predicate.parameters[0].name.text,test=unwrap(predicate.body);
            if(event===parameter||event==='EventName'||!ts.isBinaryExpression(test)||test.operatorToken.kind!==ts.SyntaxKind.EqualsEqualsEqualsToken||path(test.left)!==event+'.name'||path(test.right)!=='EventName.OnDamageDealt')continue;
            if(!ts.isArrowFunction(reducer)||reducer.modifiers?.length||reducer.parameters.length!==2||reducer.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken))continue;
            const [total,item]=reducer.parameters.map(p=>p.name.text);if(new Set([parameter,total,item,'EventName']).size!==4)continue;
            const sum=unwrap(reducer.body);if(!ts.isBinaryExpression(sum)||sum.operatorToken.kind!==ts.SyntaxKind.PlusToken||path(sum.left)!==total)continue;
            const fallback=unwrap(sum.right);if(!ts.isBinaryExpression(fallback)||fallback.operatorToken.kind!==ts.SyntaxKind.QuestionQuestionToken||path(fallback.left)!==item+'.damageDealt'||!ts.isNumericLiteral(fallback.right)||fallback.right.text!=='0')continue;
            helpers.set(member.name.text,{kind:'resolvedDamageSum',member,parameter,used:false});continue;
        }
        if(statements?.length===3&&ts.isReturnStatement(statements[2])&&statements[2].expression) {
            const decl=s=>ts.isVariableStatement(s)&&(s.declarationList.flags&ts.NodeFlags.Const)&&s.declarationList.declarations.length===1&&ts.isIdentifier(s.declarationList.declarations[0].name)?s.declarationList.declarations[0]:null;
            const base=decl(statements[0]),discarded=decl(statements[1]);if(!base?.initializer||!discarded?.initializer||path(base.initializer)!==parameter+'.player.base.aspects')continue;
            const d=chain(discarded.initializer,api);if(!d||d.root.text!==parameter||JSON.stringify(d.segments.map(s=>s.key))!==JSON.stringify(['events',0,'card','aspects'])||d.segments.some(s=>s.optional))continue;
            const some=unwrap(statements[2].expression);if(!call(some,base.name.text+'.some')||some.arguments.length!==1)continue;
            const predicate=unwrap(some.arguments[0]);if(!ts.isArrowFunction(predicate)||predicate.modifiers?.length||predicate.parameters.length!==1||!ts.isIdentifier(predicate.parameters[0].name)||predicate.parameters[0].initializer||predicate.parameters[0].dotDotDotToken)continue;
            const aspect=predicate.parameters[0].name.text,includes=unwrap(predicate.body);
            if(new Set([parameter,base.name.text,discarded.name.text,aspect]).size!==4||!call(includes,discarded.name.text+'.includes')||includes.arguments.length!==1||path(includes.arguments[0])!==aspect)continue;
            helpers.set(member.name.text,{kind:'discardSharesBaseAspect',member,parameter,used:false});
        }
    }
    return helpers;
}
function normalizeResultHelperCall(node,ctx,api) {
    const {ts,fail}=api;
    if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
    const helper=ctx.resultHelpers?.get(node.expression.name.text);if(!helper)return null;
    if(node.questionDotToken||node.expression.questionDotToken||node.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||node.arguments.length!==1||!ts.isIdentifier(node.arguments[0]))fail(ctx,node,'result-helper-call');
    for(let parent=node.parent;parent;parent=parent.parent)if(ts.isFunctionExpression(parent))fail(ctx,node,'result-helper-lexical-this');
    const binding=contextBinding(node.arguments[0],ctx,api);helper.used=true;
    if(helper.kind==='resolvedDamageSum')return {kind:'eventResult',contextBinding:binding,collection:'resolvedEvents',reduction:{operation:'sum',eventName:'onDamageDealt',field:'damageDealt',missing:0,seed:0}};
    return {kind:'resultSharesBaseAspect',card:{kind:'eventResult',contextBinding:binding,collection:'events',eventIndex:0,path:['card'],optionalAccess:[]},base:{kind:'contextRef',contextBinding:binding,path:['player','base']}};
}
function isEventResultAccess(node,ctx,api) {
    const p=chain(node,api);return !!p&&ctx.scope.get(p.root.text)?.path?.join('.')==='context'&&['events','resolvedEvents'].includes(p.segments[0]?.key);
}
module.exports={contextBinding,normalizeEventResultExpression,inspectResultHelpers,normalizeResultHelperCall,isEventResultAccess};
