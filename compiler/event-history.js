'use strict';

// These names identify pinned engine history APIs. They are deliberately not
// lowered to arena queries: entries include departed cards and captured IDs/LKI.
const WATCHERS = {
    attacksThisPhase: ['AttacksThisPhaseWatcher', {getCurrentValue:[],getAttackers:['predicate'],cardDidAttack:['value'],getAttackersInPlay:['predicate'],someUnitAttackedControlledByPlayer:['options']}],
    actionsThisPhase: ['ActionsThisPhaseWatcher',{getCurrentValue:[],getActions:['predicate'],playerHasTakenAction:['value'],previousActionNumberForPlayer:['value']}],
    cardsDefeatedThisPhase: ['CardsDefeatedThisPhaseWatcher',{getCurrentValue:[],someUnitDefeatedThisPhase:['predicate'],getDefeatedUnitsControlledByPlayer:['value'],wasDefeatedThisPhase:['value','value?'],someDefeatedUnitControlledByPlayer:['value'],someDefeatedWhileAttackingUnitControlledByPlayer:['value'],playerDefeatedEnemyUnit:['value'],someUpgradeDefeatedThisPhase:['options?']}],
    cardsPlayedThisPhase: ['CardsPlayedThisPhaseWatcher',{getCurrentValue:[],getCardsPlayed:['predicate'],someCardPlayed:['predicate']}],
    cardsLeftPlayThisPhase: ['CardsLeftPlayThisPhaseWatcher',{getCurrentValue:[],getCardsLeftPlay:['options'],getLeftPlayEntry:['value','value?'],someCardLeftPlay:['options'],someUnitLeftPlay:['options'],someLeaderUnitLeftPlay:['options']}],
    cardsEnteredPlayThisPhase: ['CardsEnteredPlayThisPhaseWatcher',{getCurrentValue:[],getCardsEnteredPlay:['predicate'],someCardEnteredPlay:['predicate']}],
    damageDealtThisPhase: ['DamageDealtThisPhaseWatcher',{getCurrentValue:[],getDamageDealtByPlayer:['value','predicate?'],playerHasDealtDamage:['value','predicate?'],wasUnitDamagedThisPhase:['value'],unitHasDealtDamage:['value','predicate?'],unitHasDealtCombatDamageToBaseThisAttack:['value','context'],getDamageDealtToBaseByUnitThisAttack:['value','context'],getNonLeaderUnitsDealtCombatDamageByUnitThisAttack:['value','context']}],
    cardsDrawnThisPhase: ['CardsDrawnThisPhaseWatcher',{getCurrentValue:[],drawnCardsAmount:['value']}],
    cardsDiscardedThisPhase: ['CardsDiscardedThisPhaseWatcher',{getCurrentValue:[]}],
    leadersDeployedThisPhase: ['LeadersDeployedThisPhaseWatcher',{getCurrentValue:[],someLeaderDeployed:['predicate']}],
    tokensCreatedThisPhase: ['TokensCreatedThisPhaseWatcher',{getCurrentValue:[],getTokensCreated:['predicate'],someTokenCreated:['predicate']}],
    unitsHealedThisPhase: ['UnitsHealedThisPhaseWatcher',{getCurrentValue:[],wasHealedThisPhase:['value','value?']}],
    unitsDamagedThisPhase: ['UnitsDamagedThisPhaseWatcher',{getCurrentValue:[],wasDamagedThisPhase:['value','value?']}],
    basesHealedThisPhase: ['BasesHealedThisPhaseWatcher',{getCurrentValue:[],wasHealedThisPhase:['value']}],
    forceUsedThisPhase: ['ForceUsedThisPhaseWatcher',{getCurrentValue:[],countForceUsedThisPhase:['value']}]
};
function synchronous(n,{ts}) {return !n.asteriskToken&&!n.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)&&!(ts.canHaveDecorators(n)&&ts.getDecorators(n)?.length);}
function inspectWatchers(cls,ctx,api) {
    const {ts,unwrap}=api,bindings=new Map(),members=new Set();
    const setup=cls.members.filter(m=>m.name?.getText(ctx.source)==='setupStateWatchers');
    if(setup.length!==1||!ts.isMethodDeclaration(setup[0])||setup[0].modifiers?.some(m=>m.kind===ts.SyntaxKind.StaticKeyword)||!synchronous(setup[0],api)||setup[0].parameters.length!==2||setup[0].parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||ts.canHaveDecorators(p)&&ts.getDecorators(p)?.length))return {bindings,members};
    const method=setup[0],helper=method.parameters[1].name.text;
    for(const s of method.body?.statements||[]) {
        if(!ts.isExpressionStatement(s)||!ts.isBinaryExpression(s.expression)||s.expression.operatorToken.kind!==ts.SyntaxKind.EqualsToken)return {bindings:new Map(),members:new Set()};
        const {left,right}=s.expression;
        if(!ts.isPropertyAccessExpression(left)||left.questionDotToken||left.expression.kind!==ts.SyntaxKind.ThisKeyword||bindings.has(left.name.text))return {bindings:new Map(),members:new Set()};
        const r=unwrap(right);
        if(!ts.isCallExpression(r)||r.questionDotToken||r.arguments.length||!ts.isPropertyAccessExpression(r.expression)||r.expression.questionDotToken)return {bindings:new Map(),members:new Set()};
        const factory=r.expression.name.text,receiver=r.expression.expression,definition=WATCHERS[factory];
        if(!definition||!ts.isPropertyAccessExpression(receiver)||receiver.questionDotToken||receiver.name.text!=='stateWatchers'||!ts.isIdentifier(receiver.expression)||receiver.expression.text!==helper)return {bindings:new Map(),members:new Set()};
        const decl=cls.members.filter(m=>m.name?.getText(ctx.source)===left.name.text);
        if(decl.length!==1||!ts.isPropertyDeclaration(decl[0])||decl[0].initializer||decl[0].modifiers?.some(m=>m.kind===ts.SyntaxKind.StaticKeyword)||!synchronous(decl[0],api)||decl[0].type?.getText(ctx.source)!==definition[0])return {bindings:new Map(),members:new Set()};
        bindings.set(left.name.text,{factory,methods:definition[1]});members.add(decl[0]);
    }
    if(bindings.size)members.add(method);
    return {bindings,members};
}
function watcherDeclarations(watchers) {return [...watchers.bindings].map(([binding,w])=>({kind:'phaseHistory',binding,watcher:w.factory,sharedScope:'game',registerAt:'setupStateWatchers',resetEvent:'onPhaseEnded',implementation:WATCHERS[w.factory][0],queryContract:'pinnedWatcherApi'}));}
function inspectHelpers(cls,ctx,api) {
    const {ts}=api,result=new Map();let eventHook=false;const find=n=>{if(ts.isPropertyAccessExpression(n)&&['addTriggeredAbility','addDamageModificationAbility','addReplacementEffectAbility'].includes(n.name.text))eventHook=true;ts.forEachChild(n,find);};find(cls);if(!ctx.watchers?.bindings.size&&!eventHook)return result;
    for(const m of cls.members) {
        const name=m.name?.getText(ctx.source);
        if(!ts.isMethodDeclaration(m)||name==='getImplementationId'||name.startsWith('setup')||!synchronous(m,api)||m.modifiers?.some(x=>x.kind===ts.SyntaxKind.StaticKeyword)||m.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||ts.canHaveDecorators(p)&&ts.getDecorators(p)?.length)||cls.members.filter(x=>x.name?.getText(ctx.source)===name).length!==1)continue;
        let effect=false;const walk=n=>{if(ts.isPropertyAccessExpression(n)&&['immediateEffects','ongoingEffects','addKeywordCopyAbility'].includes(n.name.text))effect=true;ts.forEachChild(n,walk);};walk(m);
        if(!effect){const statements=m.body?.statements,returned=statements?.length===1&&ts.isReturnStatement(statements[0])&&statements[0].expression;const eventNames=new Set(Object.entries(ctx.enums).filter(([k])=>k.startsWith('EventName.')).map(([,v])=>v));const hookFactory=m.parameters.length===0&&returned&&ts.isObjectLiteralExpression(returned)&&returned.properties.length&&returned.properties.every(p=>ts.isPropertyAssignment(p)&&eventNames.has(p.name.text));result.set(name,{member:m,name,...(hookFactory?{hookFactory:returned,used:false}:{})});}
    }
    return result;
}
function statementProgram(node,local,label,api) {
    const {ts,expr,fail}=api;let id=0;
    const statement=(n,c)=> {
            if(ts.isBlock(n)){const block={...c,scope:new Map(c.scope),dataRoots:new Set(c.dataRoots)};return {kind:'dataBlock',statements:n.statements.map(s=>statement(s,block))};}
            if(ts.isReturnStatement(n)&&n.expression)return {kind:'dataReturn',value:expr(n.expression,c)};
            if(ts.isIfStatement(n))return {kind:'dataIf',condition:expr(n.expression,c),then:statement(n.thenStatement,{...c,scope:new Map(c.scope),dataRoots:new Set(c.dataRoots)}),...(n.elseStatement?{else:statement(n.elseStatement,{...c,scope:new Map(c.scope),dataRoots:new Set(c.dataRoots)})}:{})};
            if(ts.isSwitchStatement(n))return {kind:'dataSwitch',value:expr(n.expression,c),cases:n.caseBlock.clauses.map(cl=>({...ts.isCaseClause(cl)?{value:expr(cl.expression,c)}:{default:true},statements:cl.statements.map(st=>statement(st,c))}))};
            if(ts.isBreakStatement(n)&&!n.label)return {kind:'dataBreak'};
            if(ts.isVariableStatement(n)&&(n.declarationList.flags&ts.NodeFlags.Const))return {kind:'dataBindings',bindings:n.declarationList.declarations.map(d=>{if(!ts.isIdentifier(d.name)||!d.initializer)fail(c,d,'history-helper-local');const value=expr(d.initializer,c),binding='data:helper:'+label+':local:'+(id++);c.scope.set(d.name.text,{kind:'ref',path:[binding]});c.constants.delete(d.name.text);c.dataRoots.add(binding);return {binding,value,captureAt:'statementEvaluation'};})};
            fail(c,n,'history-helper-statement:'+ts.SyntaxKind[n.kind]);
        };
    return statement(node,local);
}
function helperDefinitions(ctx,api) {
    const {ts,expr,fail}=api,definitions=[];
    for(const [name,h]of ctx.historyHelpers||[]) {
        if(h.hookFactory){if(!h.used)fail(ctx,h.member,'unused-event-hook-factory');continue;}
        const roots=h.member.parameters.map((p,i)=>'data:helper:'+name+':'+i),scope=new Map(ctx.scope),constants=new Map(ctx.constants);
        h.member.parameters.forEach((p,i)=>{scope.set(p.name.text,{kind:'ref',path:[roots[i]]});constants.delete(p.name.text);});
        const local={...ctx,scope,constants,eventHistory:true,dataRoots:new Set(roots)};
        if(!h.member.body)fail(local,h.member,'history-helper-body');
        definitions.push({kind:'historyHelper',name,parameters:roots,body:statementProgram(h.member.body,local,name,api)});
    }
    return definitions;
}
function callbackProgram(input,ctx,api,names) {
    const {ts,unwrap}=api,n=unwrap(input);
    if(!ctx.eventHistory&&!ctx.watchers?.bindings.size||(!ts.isArrowFunction(n)&&!ts.isFunctionExpression(n))||!ts.isBlock(n.body)||!n.body.statements.some(s=>ts.isIfStatement(s)||ts.isSwitchStatement(s)))return null;
    if(!synchronous(n,api)||n.parameters.length>names.length||n.parameters.some(p=>p.initializer||p.dotDotDotToken))api.fail(ctx,n,'event-callback-program-binding');
    const roots=n.parameters.map((_,i)=>'data:callback:'+n.getStart(ctx.source)+':'+i),scope=new Map(ctx.scope),constants=new Map(ctx.constants);
    n.parameters.forEach((p,i)=>bindingPattern(p.name,[roots[i]],scope,constants,api,ctx));
    const local={...ctx,scope,constants,eventHistory:true,dataRoots:new Set([...(ctx.dataRoots||[]),...roots]),callbackStage:n};
    return {kind:'callbackProgram',bindings:roots.map((binding,i)=>({binding,argument:names[i]})),body:statementProgram(n.body,local,'callback:'+n.getStart(ctx.source),api)};
}
function hooks(input,ctx,api) {
    const {ts,unwrap,object,expr,fail}=api,obj=object(input,ctx);let whenNode=unwrap(obj.props.get('when'));
    if(ts.isCallExpression(whenNode)&&ts.isPropertyAccessExpression(whenNode.expression)&&whenNode.expression.expression.kind===ts.SyntaxKind.ThisKeyword){const helper=ctx.historyHelpers?.get(whenNode.expression.name.text);if(!helper?.hookFactory||whenNode.arguments.length||whenNode.questionDotToken||whenNode.expression.questionDotToken)fail(ctx,whenNode,'event-hook-factory');helper.used=true;whenNode=helper.hookFactory;}
    const when=object(whenNode,obj.ctx);
    if([...when.props].every(([k,v])=>['whenPlayed','onAttack','whenDefeated'].includes(k)&&unwrap(v).kind===ts.SyntaxKind.TrueKeyword))return null;
    if(!when.props.size||when.props.size>16)fail(ctx,when.node,'event-hook-count');
    const names=new Set(Object.entries(ctx.enums).filter(([k])=>k.startsWith('EventName.')).map(([,v])=>v));
    const aliases=new Set(['whenPlayed','whenPlayedUsingSmuggle','onAttack','onDefense','whenDefeated','whenAttackEnds']);
    return [...when.props].map(([event,value])=>{
        if(aliases.has(event)){if(unwrap(value).kind!==ts.SyntaxKind.TrueKeyword)fail(ctx,value,'event-hook-alias-value');return {alias:event,enabled:true};}
        if(!names.has(event))fail(ctx,value,'event-hook-name:'+event);
        const n=unwrap(value);if(!ts.isArrowFunction(n)&&!ts.isFunctionExpression(n))fail(ctx,n,'event-hook-predicate');
        if(!synchronous(n,api)||n.parameters.length>2)fail(ctx,n,'event-hook-predicate-binding');
        return {event,predicate:expr(value,{...obj.ctx,eventHistory:true},['event','context'])};
    });
}
function bindingPattern(pattern,base,scope,constants,api,ctx) {
    const {ts,fail}=api;
    if(ts.isIdentifier(pattern)){scope.set(pattern.text,{kind:'ref',path:base});constants.delete(pattern.text);return;}
    if(!ts.isObjectBindingPattern(pattern))fail(ctx,pattern,'history-callback-binding');
    for(const p of pattern.elements){if(p.dotDotDotToken||p.initializer)fail(ctx,p,'history-callback-binding');const key=p.propertyName||p.name;if(!ts.isIdentifier(key)&&!ts.isStringLiteral(key))fail(ctx,key,'history-callback-property');bindingPattern(p.name,[...base,key.text],scope,constants,api,ctx);}
}
function predicate(input,ctx,api,names=['historyEntry','historyIndex','historyArray']) {
    const {ts,unwrap,expr,fail}=api,n=unwrap(input);
    if((!ts.isArrowFunction(n)&&!ts.isFunctionExpression(n))||!synchronous(n,api)||n.parameters.length>names.length||n.parameters.some(p=>p.initializer||p.dotDotDotToken))fail(ctx,n,'history-predicate-shape');
    const scope=new Map(ctx.scope),constants=new Map(ctx.constants),id='data:'+n.getStart(ctx.source);
    const roots=names.map((name,i)=>id+':'+i),local={...ctx,scope,constants,eventHistory:true,dataRoots:new Set([...(ctx.dataRoots||[]),...roots])};
    n.parameters.forEach((p,i)=>bindingPattern(p.name,[roots[i]],scope,constants,api,ctx));
    let body=n.body;
    if(ts.isBlock(body)) {
        if(body.statements.length!==1||!ts.isReturnStatement(body.statements[0])||!body.statements[0].expression)return {kind:'dataLambda',bindings:roots.slice(0,n.parameters.length),body:statementProgram(body,local,'predicate:'+n.getStart(ctx.source),api),bodyKind:'program'};body=body.statements[0].expression;
    }
    return {kind:'dataLambda',bindings:roots.slice(0,n.parameters.length),body:expr(body,local)};
}
function path(input,ctx,api) {
    const {ts,unwrap}=api,n=unwrap(input);
    if(ts.isIdentifier(n)) {
        const b=ctx.scope.get(n.text);if(b?.kind==='ref'&&(['event','context'].includes(b.path[0])||ctx.dataRoots?.has(b.path[0])))return {root:b.path[0]==='context'?['context']:b.path,rootNode:n,steps:[]};return null;
    }
    if(ts.isPropertyAccessExpression(n)){const p=path(n.expression,ctx,api);return p&&{...p,steps:[...p.steps,{key:n.name.text,optional:!!n.questionDotToken}]};}
    if(ts.isElementAccessExpression(n)&&n.argumentExpression){const p=path(n.expression,ctx,api);return p&&{...p,steps:[...p.steps,{index:n.argumentExpression,optional:!!n.questionDotToken}]};}
    return null;
}
function origin(p,ctx,api) {
    const {ts,fail}=api;if(p.root[0].startsWith('data:'))return {};
    if(p.root[0]==='context')return {contextBinding:require('./event-result-context').contextBinding(p.rootNode,ctx,api)};
    for(let a=p.rootNode.parent;a;a=a.parent){if(!a.parameters)continue;const index=a.parameters.findIndex(x=>ts.isIdentifier(x.name)&&x.name.text===p.rootNode.text);if(index<0)continue;return {argumentBinding:{id:'callback:'+a.getStart(ctx.source)+':'+index,bindAt:'callbackInvocation',readAt:'expressionEvaluation'}};}
    fail(ctx,p.rootNode,'event-argument-origin');
}
function dataReceiver(n,ctx,api) {
    const p=path(n,ctx,api);return p&&(p.root[0]==='event'||p.root[0].startsWith('data:')||p.root[0]==='context'&&(p.steps[0]?.key==='event'||ctx.pendingResultArray&&['events','resolvedEvents'].includes(p.steps[0]?.key)||p.steps.some(s=>['activeAttack','currentAttack'].includes(s.key))||['game','replacementEffectWindow'].includes(p.steps[0]?.key)));
}
function pendingResultAccess(input,ctx,api) {
    const {ts,unwrap}=api,n=unwrap(input),p=path(n,ctx,api);
    if(p?.root[0]==='context'&&['events','resolvedEvents'].includes(p.steps[0]?.key)) {
        const [collection,index,...tail]=p.steps;if(!index?.index||!ts.isNumericLiteral(unwrap(index.index))||!tail.length)return true;
        const keys=tail.map(x=>x.index&&ts.isNumericLiteral(unwrap(x.index))?Number(unwrap(x.index).text):x.key);
        const known=keys.length===1&&['card','generatedTokens','willDefeat','availableExcessDamage','totalDistributed'].includes(keys[0])||keys.length===2&&keys[0]==='card'&&['printedCost','cost','title','type','aspects'].includes(keys[1])||keys.length===2&&keys[0]==='generatedTokens'&&typeof keys[1]==='number'||keys.length===2&&keys[0]==='lastKnownInformation'&&['power','hp','cost','card','arena','controller','parentCard','title','exhausted','type'].includes(keys[1])||keys.length===3&&keys[0]==='individualEvents'&&typeof keys[1]==='number'&&keys[2]==='card';
        return !known;
    }
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression))return pendingResultAccess(n.expression.expression,ctx,api);
    if(ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n))return pendingResultAccess(n.expression,ctx,api);
    return false;
}
function capturesHistory(input,ctx,api,seen=new Set()) {
    const {ts,unwrap}=api,n=unwrap(input);if(ts.isArrowFunction(n)||ts.isFunctionExpression(n))return false;
    if(pendingResultAccess(n,ctx,api))return true;
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)) {
        const r=n.expression.expression;
        if(ts.isPropertyAccessExpression(r)&&r.expression.kind===ts.SyntaxKind.ThisKeyword&&ctx.watchers?.bindings.has(r.name.text)||r.kind===ts.SyntaxKind.ThisKeyword&&ctx.historyHelpers?.has(n.expression.name.text))return true;
    }
    if(ts.isIdentifier(n)&&ctx.constants.has(n.text)&&!(ts.isPropertyAccessExpression(n.parent)&&n.parent.name===n)) {
        const value=ctx.constants.get(n.text);if(seen.has(value))return false;const next=new Set(seen);next.add(value);return capturesHistory(value,{...ctx,constants:ctx.constantEnvs?.get(value)||ctx.constants},api,next);
    }
    let result=false;ts.forEachChild(n,c=>{if(capturesHistory(c,ctx,api,seen))result=true;});return result;
}
function capturedHistory(input,ctx,api) {
    const {ts,unwrap,expr,fail,readConstant}=api,n=unwrap(input);
    const tail=[];let root=n;
    while(ts.isPropertyAccessExpression(root)||ts.isElementAccessExpression(root)){tail.unshift(ts.isPropertyAccessExpression(root)?{key:root.name.text,optional:!!root.questionDotToken}:{index:root.argumentExpression,optional:!!root.questionDotToken});root=root.expression;}
    if(!ts.isIdentifier(root)||!ctx.constants.has(root.text))return null;
    const initializer=ctx.constants.get(root.text);if(!capturesHistory(initializer,{...ctx,constants:ctx.constantEnvs?.get(initializer)||ctx.constants},api))return null;
    const trail=new Set(ctx.historyCaptureTrail||[]);if(trail.has(initializer))fail(ctx,n,'cyclic-history-capture');trail.add(initializer);readConstant(root.text,ctx);
    const stage=ctx.constantStages?.get(initializer),local={...ctx,scope:ctx.constantScopes?.get(initializer)||ctx.scope,constants:ctx.constantEnvs?.get(initializer)||ctx.constants,callbackStage:stage,historyCaptureTrail:trail};
    return {kind:'capturedHistory',binding:'const:'+initializer.getStart(ctx.source),captureAt:'declarationEvaluation',owner:stage?'callback:'+stage.getStart(ctx.source):'abilitySetup',value:expr(initializer,local),steps:tail.map(t=>t.index?{index:expr(t.index,ctx),optional:t.optional}:t)};
}
function normalize(input,ctx,api) {
    const {ts,unwrap,expr,fail,object,literal,unshadowedEnum}=api,n=unwrap(input);
    if(pendingResultAccess(n,ctx,api))ctx={...ctx,eventHistory:true,pendingResultArray:true};
    const captured=capturedHistory(n,ctx,api);if(captured)return captured;
    if(ctx.eventHistory&&Object.hasOwn(ctx.enums,n.getText(ctx.source))){let root=n;while(ts.isPropertyAccessExpression(root))root=root.expression;if(ts.isIdentifier(root))unshadowedEnum(root,ctx,root.text);return {kind:'const',value:ctx.enums[n.getText(ctx.source)]};}
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)) {
        const call=n.expression,name=call.name.text,r=call.expression;
        if(ts.isPropertyAccessExpression(r)&&r.expression.kind===ts.SyntaxKind.ThisKeyword&&ctx.watchers?.bindings.has(r.name.text)) {
            if(n.questionDotToken||call.questionDotToken||r.questionDotToken)fail(ctx,n,'optional-history-query');
            for(let a=n.parent;a;a=a.parent)if(ts.isFunctionExpression(a))fail(ctx,n,'history-query-this');
            const watcher=ctx.watchers.bindings.get(r.name.text),signature=watcher.methods[name];
            if(!signature||n.arguments.length>signature.length||n.arguments.length<signature.filter(x=>!x.endsWith('?')).length)fail(ctx,n,'history-query-signature:'+name);
            const args=n.arguments.map((a,i)=>{
                const type=signature[i].replace('?','');
                if(type==='predicate')return predicate(a,ctx,api);
                if(type==='options') {const o=object(a,ctx),fields={};for(const [key,value]of o.props){if(!['controller','filter'].includes(key))fail(ctx,value,'history-option:'+key);fields[key]=key==='filter'?predicate(value,o.ctx,api):expr(value,o.ctx);}return {kind:'historyOptions',fields};}
                if(type==='context'){const p=path(a,ctx,api);return p?.root[0].startsWith('data:')?{kind:'historyAbilityContext',value:expr(a,ctx)}:{kind:'historyAbilityContext',binding:require('./event-result-context').contextBinding(a,ctx,api)};}
                return expr(a,{...ctx,eventHistory:true});
            });
            return {kind:'historyQuery',watcher:watcher.factory,binding:r.name.text,operation:name,args,readAt:'expressionEvaluation'};
        }
        if(ctx.historyHelpers?.has(name)&&r.kind===ts.SyntaxKind.ThisKeyword) {
            const h=ctx.historyHelpers.get(name);if(h.hookFactory)fail(ctx,n,'event-hook-factory-value');if(n.questionDotToken||call.questionDotToken||n.arguments.length!==h.member.parameters.length)fail(ctx,n,'history-helper-call');
            for(let a=n.parent;a;a=a.parent)if(ts.isFunctionExpression(a))fail(ctx,n,'history-helper-this');
            return {kind:'historyHelperCall',name,args:n.arguments.map(a=>{const p=path(a,ctx,api);return p?.root[0]==='context'&&!p.steps.length?{kind:'historyAbilityContext',binding:require('./event-result-context').contextBinding(a,ctx,api)}:expr(a,{...ctx,eventHistory:true});})};
        }
        if(ctx.eventHistory&&ts.isIdentifier(r)&&r.text==='EnumHelpers'&&name==='asRelativePlayer'){unshadowedEnum(r,ctx,'EnumHelpers');if(n.arguments.length!==2||n.questionDotToken||call.questionDotToken)fail(ctx,n,'relative-player-query');return {kind:'relativePlayerQuery',player:expr(n.arguments[0],ctx),otherPlayer:expr(n.arguments[1],ctx)};}
        if(ctx.eventHistory&&name==='isReplacement'&&path(r,ctx,api)?.root[0]==='context'&&!path(r,ctx,api).steps.length){if(n.arguments.length||n.questionDotToken||call.questionDotToken)fail(ctx,n,'replacement-context-query');return {kind:'contextQuery',query:name,contextBinding:require('./event-result-context').contextBinding(r,ctx,api)};}
        if(ctx.eventHistory&&ts.isIdentifier(r)&&r.text==='Array'&&name==='from'){unshadowedEnum(r,ctx,'Array');if(n.arguments.length!==1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'replacement-array-from');return {kind:'arrayFrom',value:expr(n.arguments[0],ctx)};}
        if(ctx.eventHistory&&r.kind===ts.SyntaxKind.ThisKeyword&&name==='getObject'){if(n.arguments.length!==1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'history-object-lookup');return {kind:'historyObjectLookup',id:expr(n.arguments[0],ctx)};}
        if(ctx.eventHistory&&ts.isIdentifier(r)&&r.text==='Helpers'&&name==='pluralize'){unshadowedEnum(r,ctx,'Helpers');if(n.arguments.length!==3||n.questionDotToken||call.questionDotToken)fail(ctx,n,'history-pluralize');return {kind:'historyPluralize',count:expr(n.arguments[0],ctx),singular:literal(n.arguments[1],ctx),plural:literal(n.arguments[2],ctx)};}
        if(ctx.eventHistory&&['canBeInPlay','isNonTokenUnit','isAttached'].includes(name)&&!dataReceiver(r,ctx,api)){if(n.arguments.length||n.questionDotToken||call.questionDotToken)fail(ctx,n,'event-card-query-shape');return {kind:'dataCall',method:name,receiver:expr(r,ctx),args:[],optionalCall:false,optionalReceiver:false};}
        if(ctx.eventHistory&&ts.isIdentifier(r)&&r.text==='DefeatCardSystem'&&name==='defeatSourceCard'){unshadowedEnum(r,ctx,'DefeatCardSystem');if(n.arguments.length!==1||n.questionDotToken||call.questionDotToken)fail(ctx,n,'defeat-source-query');return {kind:'defeatSourceCard',event:expr(n.arguments[0],ctx)};}
        if(ts.isIdentifier(r)&&r.text==='AttackHelpers'&&['defenderWasDefeated','attackerSurvived'].includes(name)) {
            unshadowedEnum(r,ctx,'AttackHelpers');const w=n.arguments[1];if(n.questionDotToken||call.questionDotToken||n.arguments.length!==2||!ts.isPropertyAccessExpression(w)||w.expression.kind!==ts.SyntaxKind.ThisKeyword||ctx.watchers?.bindings.get(w.name.text)?.factory!=='cardsDefeatedThisPhase')fail(ctx,n,'history-attack-helper');
            return {kind:'historyAttackCheck',test:name,attack:expr(n.arguments[0],{...ctx,eventHistory:true}),watcher:'cardsDefeatedThisPhase',binding:w.name.text};
        }
        if(ctx.eventHistory&&(dataReceiver(r,ctx,api)||isHistorySyntax(r,ctx,api))) {
            const methods=new Set(['some','every','filter','find','map','flatMap','reduce','includes','indexOf','concat','slice','isAttackAction','getArenaCards','getTopCardsOfDeck','getTopCardOfDeck','getShieldsNotQueuedForDefeat','getArenaUnits','getLeader','getLeaderUnit','getLeaderCards','getLeaderSide','getPlayers','join','getPrintedPower','getPrintedHp','getTriggeredAbilities','isNonTokenUnit','getLegalTargets','getSingleTarget','getAllTargets','getDefendingPlayer','getAttacker','targetIsUnit','targetIsBase','isUnit','isBase','isLeader','isLeaderUnit','isNonLeaderUnit','isUpgrade','isToken','isTokenUnit','isTokenUpgrade','isShield','isExperience','isWeakness','isUpgraded','isDeployableLeader','isInPlay','isDeployed','isAttacking','isDefending','hasSomeTrait','hasEveryTrait','hasSomeAspect','hasEveryAspect','hasSomeKeyword','hasEveryKeyword','hasCost','getPower','getHp','getCost','getPrintedCost','canBeInPlay','canRegisterTriggeredAbilities','isAction','isActivated','isTriggered','isEvent','hasTargets','hasSome','has','isSmuggle','isWhenDefeated','isWhenPlayed','isOnAttack']);
            if(!methods.has(name))fail(ctx,n,'event-history-method:'+name);
            const args=n.arguments.map((a,i)=>{const v=unwrap(a);return ts.isArrowFunction(v)||ts.isFunctionExpression(v)?predicate(v,ctx,api,name==='reduce'?['accumulator','historyEntry','historyIndex','historyArray']:undefined):expr(a,{...ctx,allowArenaList:true});});
            return {kind:'dataCall',method:name,receiver:expr(r,ctx),args,optionalCall:!!n.questionDotToken,optionalReceiver:!!call.questionDotToken};
        }
        if(ctx.eventHistory&&ts.isIdentifier(r)&&r.text==='EnumHelpers'&&['isUnit','isUpgrade','isLeaderUnit','isNonLeaderUnit','isBase','isLeader','arenaName','isArena'].includes(name)) {
            unshadowedEnum(r,ctx,'EnumHelpers');return {kind:'enumTypeTest',test:name,args:n.arguments.map(a=>expr(a,ctx))};
        }
    }
    if(ctx.eventHistory) {
        if(ts.isIdentifier(n)&&n.text==='undefined'&&!ctx.constants.has(n.text)&&!ctx.scope.has(n.text))return {kind:'undefined'};
        const gamePath=path(n,ctx,api);if(gamePath?.root[0]==='context'&&(['game.currentPhase','game.actionNumber','game.roundNumber'].includes(gamePath.steps.map(s=>s.key).join('.'))||['target','targets'].includes(gamePath.steps[0]?.key)&&gamePath.steps.some(s=>s.optional)))return {kind:'dataPath',root:['context'],...origin(gamePath,ctx,api),steps:gamePath.steps};
        if(ts.isObjectLiteralExpression(n)){const o=object(n,ctx);return {kind:'dataRecord',fields:Object.fromEntries([...o.props].map(([k,v])=>[k,expr(v,ctx)]))};}
        if(ts.isTemplateExpression(n))return {kind:'dataTemplate',parts:[n.head.text,...n.templateSpans.flatMap(s=>[expr(s.expression,ctx),s.literal.text])]};
        const p=path(n,ctx,api);
        if(p&&(p.root[0]==='event'||p.root[0].startsWith('data:')||p.root[0]==='context'&&(p.steps[0]?.key==='event'||ctx.pendingResultArray&&['events','resolvedEvents'].includes(p.steps[0]?.key)||p.steps.some(s=>['activeAttack','currentAttack'].includes(s.key))||['game','replacementEffectWindow'].includes(p.steps[0]?.key)))) {
            const root=p.root,steps=p.steps;
            return {kind:'dataPath',root,...origin(p,ctx,api),steps:steps.map(s=>s.index?{index:expr(s.index,ctx),optional:s.optional}:s)};
        }
        if((ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n))&&(isHistorySyntax(n.expression,ctx,api)||ts.isIdentifier(n.expression)&&ctx.constants.has(n.expression.text)))return {kind:'dataMember',receiver:expr(n.expression,ctx),...(ts.isPropertyAccessExpression(n)?{key:n.name.text}:{index:expr(n.argumentExpression,ctx)}),optional:!!n.questionDotToken};
    }
    return null;
}
function isHistorySyntax(n,ctx,api) {
    const {ts,unwrap}=api;n=unwrap(n);
    if(dataReceiver(n,ctx,api))return true;
    if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)) {
        const r=n.expression.expression;
        if(ts.isIdentifier(r)&&r.text==='DefeatCardSystem'&&n.expression.name.text==='defeatSourceCard')return true;
        if(r.kind===ts.SyntaxKind.ThisKeyword&&ctx.historyHelpers?.has(n.expression.name.text))return true;
        if(ts.isPropertyAccessExpression(r)&&r.expression.kind===ts.SyntaxKind.ThisKeyword&&ctx.watchers?.bindings.has(r.name.text))return true;
        return isHistorySyntax(r,ctx,api);
    }
    if(ts.isPropertyAccessExpression(n)||ts.isElementAccessExpression(n))return isHistorySyntax(n.expression,ctx,api);
    return false;
}
function activationLimit(input,ctx,api) {
    const {ts,unwrap,literal,fail}=api,n=unwrap(input);
    if(!ts.isCallExpression(n)||n.questionDotToken||!ts.isPropertyAccessExpression(n.expression)||n.expression.questionDotToken||!ts.isPropertyAccessExpression(n.expression.expression)||n.expression.expression.name.text!=='limit'||n.expression.expression.questionDotToken)fail(ctx,n,'history-activation-limit');
    const root=n.expression.expression.expression;if(!ts.isIdentifier(root))fail(ctx,n,'history-limit-factory-binding');
    let bound=false;for(let a=root.parent;a;a=a.parent){if(!a.parameters)continue;const index=a.parameters.findIndex(p=>ts.isIdentifier(p.name)&&p.name.text===root.text);if(index<0)continue;bound=ts.isMethodDeclaration(a)&&index===1&&a.name?.getText(ctx.source).startsWith('setup');break;}if(!bound)fail(ctx,n,'history-limit-factory-binding');
    const method=n.expression.name.text,arity=method==='repeatable'?2:method==='unlimited'||method==='epicAction'?0:1;
    if(!['perPhase','perRound','perGame','perPlayerPerGame','repeatable','unlimited','epicAction'].includes(method)||n.arguments.length!==arity)fail(ctx,n,'history-activation-limit');
    const args=n.arguments.map(a=>literal(a,ctx));if(arity&&(!Number.isSafeInteger(args[0])||args[0]<1))fail(ctx,n,'history-activation-limit-count');
    return {kind:'abilityLimit',factory:method,args,scope:'registration',sharedAcrossHooks:true};
}
module.exports={callbackProgram,activationLimit,inspectWatchers,watcherDeclarations,inspectHelpers,helperDefinitions,hooks,normalize,predicate};
