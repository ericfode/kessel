'use strict';

// Exact compiler-only systems for token entry effects and extended resourcing.
const constant=value=>({kind:'const',value});
function fields(obj,names,api) {
    for(const [key,value]of obj.props)if(!names.includes(key))api.fail(obj.ctx,value,'resource-token-field:'+key);
}
function resourceTarget(input,ctx,api) {
    const {ts,unwrap,expr,fail}=api,n=unwrap(input);
    if(!ts.isCallExpression(n)||!ts.isPropertyAccessExpression(n.expression)||!['getTopCardOfDeck','getTopCardsOfDeck'].includes(n.expression.name.text))return null;
    const plural=n.expression.name.text==='getTopCardsOfDeck';
    if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==(plural?1:0)||!ctx.callbackStage)fail(ctx,n,'resource-deck-target-shape');
    const receiver=expr(n.expression.expression,{...ctx,arenaReceiver:true});
    if(receiver.kind!=='ref')fail(ctx,n,'resource-deck-target-controller');
    return {kind:'query',name:plural?'topDeckCards':'topDeckCard',receiver,args:plural?[expr(n.arguments[0],ctx)]:[]};
}
function extendedResourceSystem(obj,defaultTarget,api) {
    const {props,ctx}=obj;
    if(!props.has('readyResource')&&!props.has('targetPlayer')&&!(props.has('target')&&api.ts.isCallExpression(api.unwrap(props.get('target')))&&api.unwrap(props.get('target')).expression.name?.text==='getTopCardsOfDeck'))return null;
    fields(obj,['target','readyResource','targetPlayer'],api);
    let readyResource=false,dynamicReadyResource=false;
    if(props.has('readyResource')){
        const input=props.get('readyResource'),n=api.unwrap(input);
        if(api.ts.isArrowFunction(n)||api.ts.isFunctionExpression(n))api.fail(ctx,n,'resource-readiness-is-not-property-value');
        try{readyResource=api.literal(input,ctx);}catch(error){if(!error.diagnostic)throw error;readyResource=api.expr(input,ctx);dynamicReadyResource=true;}
    }
    let targetPlayer='self',dynamicTargetPlayer=false;
    if(props.has('targetPlayer')){
        try{targetPlayer=api.literal(props.get('targetPlayer'),ctx);}catch(error){if(!error.diagnostic)throw error;targetPlayer=api.expr(props.get('targetPlayer'),ctx);dynamicTargetPlayer=true;}
        const n=api.unwrap(props.get('targetPlayer'));if(api.ts.isArrowFunction(n)||api.ts.isFunctionExpression(n))api.fail(ctx,n,'resource-target-player-is-not-relative-value');
    }
    if((!dynamicReadyResource&&typeof readyResource!=='boolean')||(!dynamicTargetPlayer&&!['self','opponent'].includes(targetPlayer)))api.fail(ctx,obj.node,'resource-system-property-type');
    return {...(dynamicTargetPlayer?{resourceControllingPlayer:{kind:'conditional',condition:{kind:'binary',op:'===',left:targetPlayer,right:constant('self')},then:{kind:'ref',path:['player']},else:{kind:'ref',path:['opponent']}},controllerEvaluation:'resourceEventPropertyGeneration'}:{}),kind:'resourceCardSystem',target:props.has('target')?(resourceTarget(props.get('target'),ctx,api)||api.target(props.get('target'),ctx,defaultTarget)):defaultTarget,targetPlayer,readyResource,...(dynamicReadyResource?{readinessPropertyContract:{evaluateAt:'eachNativePropertyGeneration',undefined:'retain',condition:'nativeTruthiness',captureAt:'ResourceCardSystem.updateEventBeforeParentUpdate',capturedTargets:'completePropertiesTargetList',contingent:'ReadySystemForEveryCapturedTargetAfterResourceEvent',eventHandler:'assertPlayableThenTakeControlOrMoveToResource'}}:{}),readyTiming:'contingentAfterResourceEvent',readinessTargets:'completeSystemTargetList'};
}
function extendedTokenSystem(kind,obj,defaultTarget,api) {
    const types={createBattleDroid:'battleDroid',createCloneTrooper:'cloneTrooper',createXWing:'xwing',createTieFighter:'tieFighter',createSpy:'spy',createBeast:'beast',createMandalorian:'mandalorian'};
    if(!Object.hasOwn(types,kind))return null;
    const {props,ctx,node}=obj,{ts,unwrap}=api;
    fields(obj,['amount','target','entersReady','enterPlayEffect'],api);
    const target=props.has('target')?api.target(props.get('target'),ctx,'ownPlayer'):(ctx.selectedPlayerTarget||'ownPlayer');
    let fixedAmount=null;
    if(!props.has('amount'))fixedAmount=1;
    else {try {const value=api.literal(props.get('amount'),ctx);if(typeof value==='number')fixedAmount=value;}catch(error){if(!error.diagnostic)throw error;}}
    let legacyDynamic=false;
    if(props.has('amount')&&fixedAmount===null){try{legacyDynamic=!!api.resourceTokenAmount(props.get('amount'),ctx,{ts,unwrap,expr:api.expr,literal:api.literal,fail:api.fail,unshadowedEnum:api.unshadowedEnum});}catch(error){if(!error.diagnostic)throw error;}}
    const extended=props.has('enterPlayEffect')||typeof target!=='string'||!['ownPlayer','enemyPlayer'].includes(target)||fixedAmount!==null&&(fixedAmount<1||fixedAmount>5)||props.has('amount')&&fixedAmount===null&&!legacyDynamic;
    if(!extended)return null;
    const amount=props.has('amount')?api.expr(props.get('amount'),ctx):constant(1);
    let n=props.has('amount')&&unwrap(props.get('amount'));const seen=new Set();
    while(n&&ts.isIdentifier(n)&&ctx.constants.has(n.text)){if(seen.has(n.text))api.fail(ctx,n,'token-amount-alias-cycle');seen.add(n.text);n=unwrap(ctx.constants.get(n.text));}
    if(n&&(ts.isArrowFunction(n)||ts.isFunctionExpression(n)))api.fail(ctx,n,'token-amount-is-not-number');
    // Keep the specialized 1..5 fence for plain fixed amounts. Extended mode
    // is for dynamic/controller/entry shapes, not for admitting amount 0.
    if(fixedAmount!==null&&(!Number.isSafeInteger(fixedAmount)||fixedAmount<1||fixedAmount>64))api.fail(ctx,node,'token-system-fixed-amount-bound');
    const entersReady=props.has('entersReady')?api.literal(props.get('entersReady'),ctx):false;
    if(typeof entersReady!=='boolean')api.fail(ctx,node,'token-system-readiness-type');
    // Preserve source array order rather than compressing a player array to a
    // set-like target label. Default targets are acting player unless overridden.
    let controllers=props.has('target')?api.expr(props.get('target'),{...ctx,arenaReceiver:true}):target;
    if(!ctx.callbackStage&&controllers?.kind==='ref'&&['source.controller','source.controller.opponent'].includes(controllers.path.join('.')))controllers={kind:'registrationSnapshot',value:controllers};
    const result={kind:'createTokenUnitSystem',tokenType:types[kind],amount,controllers,entersReady,creationTiming:'eventUpdate',entryTiming:'contingentEvents'};
    if(props.has('enterPlayEffect')) {
        const raw=unwrap(props.get('enterPlayEffect')),nodes=ts.isArrayLiteralExpression(raw)?raw.elements:[raw];
        if(!nodes.length||nodes.length>8)api.fail(ctx,raw,'token-entry-effect-bound');
        result.entryEffects=nodes.map(n=>({kind:'tokenEntryEffect',targetBinding:'enteringUnit',forceEventTarget:true,system:api.effect(n,ctx,'enteringUnit')}));
        result.entryEffectTiming='afterEntryBeforeStateCheck';
    }
    return result;
}
// The old runtime-facing top-deck form retains its reviewed whole-source shape.
// Other fully lowered source programs get an explicit compiler-only system, so
// expanding compilation cannot implicitly expand executable runtime support.
function extendTopDeckResources(abilities) {
    const visit=value=>{
        if(!value||typeof value!=='object')return;
        if(value.kind==='resourceCard'&&value.target==='ownTopDeck') {
            value.kind='resourceCardSystem';value.target={kind:'query',name:'topDeckCard',receiver:{kind:'ref',path:['player']},args:[]};
            value.targetPlayer='self';value.readyResource=false;value.readyTiming='contingentAfterResourceEvent';value.readinessTargets='completeSystemTargetList';
        }
        Object.values(value).forEach(visit);
    };
    visit(abilities);
}
function ignoreAspectAbility(input,ctx,registration,api) {
    const obj=api.object(input,ctx),props=new Map(obj.props);
    props.set('costAdjustType',api.ts.factory.createStringLiteral(registration==='addIgnoreAllAspectPenaltiesAbility'?'ignoreAllAspects':'ignoreSpecificAspect'));
    return api.adjustCostAbility(input,ctx,api,{...obj,props});
}
module.exports={extendedResourceSystem,extendedTokenSystem,extendTopDeckResources,ignoreAspectAbility};
