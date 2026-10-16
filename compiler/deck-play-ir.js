'use strict';
const { adjustmentProperties } = require('./cost-resource-systems');

// Declarative deck/play systems. Callback values keep their evaluation role;
// selection-dependent systems use symbolic selected/remaining card defaults.
// These are pending-executor operations, never aliases of draw or ordinary play.
function deckPlayEffect(kind, obj, defaultTarget, api) {
    const {ts, unwrap, expr, literal, effect, target, object, fail, contextTitle} = api;
    const c = value => ({kind:'const',value});
    const check = names => {for(const [key,value] of obj.props)if(!names.includes(key))fail(obj.ctx,value,'unsupported-'+kind+'-field:'+key);};
    const derived = (input, role, params=['context']) => {
        const n=unwrap(input), value=expr(input,params.includes('selectedCards')?{...obj.ctx,pendingSelection:true}:obj.ctx,params);
        return ts.isArrowFunction(n)||ts.isFunctionExpression(n)
            ?{kind:'systemValue',evaluation:role,value}:value;
    };
    const predicate = (input,role) => derived(input,role,['card','context']);
    const prompts = result => {
        for(const key of ['title','activePromptTitle','noSelectedCardsButtonText'])if(obj.props.has(key))result[key+'Parts']=contextTitle(obj.props.get(key),obj.ctx);
    };
    if(kind==='playMultipleCardsFromDeck') {
        const extra=['playAsType','playedCardEnterPlayEffect'];
        const searchObj={...obj,ctx:{...obj.ctx,pendingSelection:true},props:new Map([...obj.props].filter(([key])=>!extra.includes(key)))};
        const search=deckPlayEffect('deckSearch',searchObj,defaultTarget,api);
        const play={kind:'playCardSystem',helper:'playCard',playType:'playFromOutOfPlay',nested:true,adjustCost:{kind:'costAdjuster',adjustment:'free'},playAsType:obj.props.has('playAsType')?literal(obj.props.get('playAsType'),obj.ctx):'any',target:'searchSelected'};
        if(obj.props.has('playedCardEnterPlayEffect')) {const raw=unwrap(obj.props.get('playedCardEnterPlayEffect'));play.enterPlayEffect=ts.isArrayLiteralExpression(raw)?{kind:'simultaneous',effects:raw.elements.map(n=>effect(n,obj.ctx,'playedCard'))}:effect(raw,obj.ctx,'playedCard');}
        return {...search,kind,selectionOrder:'playInSelectedOrder',selectedCardsImmediateEffect:play};
    }
    if(['playMultipleCardsFromHand','playMultipleCardsFromDiscard'].includes(kind)) {
        check(['maxCards','playAsType','adjustCost','attachTargetCondition','activePromptTitle','cardCondition','cardTypeFilter']);
        const forwarded=new Map([...obj.props].filter(([key])=>['playAsType','adjustCost','attachTargetCondition'].includes(key)));
        const play=deckPlayEffect('playCard',{...obj,props:forwarded},'selectedPlayCard',api);
        play.nested=true;play.playType=kind==='playMultipleCardsFromHand'?'playFromHand':'playFromOutOfPlay';
        if(play.attachTargetCondition)play.attachContext='multiplePlaySystemContext';
        const result={kind,zone:kind==='playMultipleCardsFromHand'?'hand':'discard',controller:'self',maxCards:obj.props.has('maxCards')?derived(obj.props.get('maxCards'),'multiplePlayMaximum'):null,maximumDefault:'unbounded',selectionMode:'single',optional:true,nextSelection:'afterFullResolutionAndTriggers',stopOn:'declineOrNoLegalTargetOrMaximum',perCardPlay:play};
        if(obj.props.has('cardCondition'))result.cardCondition=predicate(obj.props.get('cardCondition'),'multiplePlaySelection');
        if(obj.props.has('cardTypeFilter'))result.cardTypeFilter=literal(obj.props.get('cardTypeFilter'),obj.ctx);
        prompts(result);return result;
    }
    if(kind==='deckSearch') {
        check(['target','targetMode','searchCount','searchWholeDeck','canChooseFewer','selectCount','shuffleWhenDone','title','activePromptTitle','choosingPlayer','selectedCardsImmediateEffect','remainingCardsImmediateEffect','chooseNothingImmediateEffect','cardCondition','multiSelectCondition']);
        const result={kind,target:target(obj.props.get('target'),obj.ctx,'ownPlayer'),targetMode:'upTo',searchWholeDeck:c(false),selectCount:c(1),shuffleWhenDone:c(false),cardCondition:c(true),multiSelectCondition:c(true),remainingCardsDefault:'bottomInRandomOrderUnlessWholeDeck',wholeDeckAlwaysShuffles:true,completionOrder:['remaining','selected','shuffle'],selectedContextField:'selectedPromptCards'};
        for(const key of ['targetMode'])if(obj.props.has(key))result[key]=literal(obj.props.get(key),obj.ctx);
        for(const key of ['searchCount','searchWholeDeck','canChooseFewer','selectCount','shuffleWhenDone'])if(obj.props.has(key))result[key]=derived(obj.props.get(key),'searchProperties:'+key);
        if(!obj.props.has('searchCount')&&!obj.props.has('searchWholeDeck'))fail(obj.ctx,obj.node,'search-count-or-whole-deck-required');
        if(obj.props.has('choosingPlayer'))result.choosingPlayer=expr(obj.props.get('choosingPlayer'),obj.ctx);
        if(obj.props.has('cardCondition'))result.cardCondition=predicate(obj.props.get('cardCondition'),'searchCardFilter');
        if(obj.props.has('multiSelectCondition'))result.multiSelectCondition=derived(obj.props.get('multiSelectCondition'),'searchMultiSelectFilter',['card','selectedCards','context']);
        for(const [key,binding] of [['selectedCardsImmediateEffect','searchSelected'],['remainingCardsImmediateEffect','searchRemaining'],['chooseNothingImmediateEffect',defaultTarget]])if(obj.props.has(key))result[key]=effect(obj.props.get(key),obj.ctx,binding);
        prompts(result);return result;
    }
    if(kind==='lookAt') {
        check(['target','player','useDisplayPrompt','activePromptTitle','message','messageArgs']);
        const result={kind,target:target(obj.props.get('target'),obj.ctx,defaultTarget),player:obj.props.has('player')?expr(obj.props.get('player'),obj.ctx):{kind:'ref',path:['player']},interactMode:'viewOnly',useDisplayPrompt:obj.props.has('useDisplayPrompt')?literal(obj.props.get('useDisplayPrompt'),obj.ctx):null};
        if(obj.props.has('message'))result.messageParts=contextTitle(obj.props.get('message'),obj.ctx);
        if(obj.props.has('messageArgs'))result.messageArgs=derived(obj.props.get('messageArgs'),'viewMessage',['viewCards']);
        prompts(result);return result;
    }
    if(kind==='lookAtAndSelectCard') {
        check(['target','player','useDisplayPrompt','activePromptTitle','canChooseFewer','maxCards','cardCondition','immediateEffect','noSelectedCardsButtonText']);
        const result={kind,target:target(obj.props.get('target'),obj.ctx,defaultTarget),player:obj.props.has('player')?expr(obj.props.get('player'),obj.ctx):{kind:'ref',path:['player']},useDisplayPrompt:obj.props.has('useDisplayPrompt')?literal(obj.props.get('useDisplayPrompt'),obj.ctx):true,maxCards:c(1),canChooseFewer:c(true),cardCondition:c(true),selectedContextField:'selectedPromptCards'};
        for(const key of ['maxCards','canChooseFewer'])if(obj.props.has(key))result[key]=derived(obj.props.get(key),'viewSelection:'+key);
        if(obj.props.has('cardCondition'))result.cardCondition=predicate(obj.props.get('cardCondition'),'viewSelectionFilter');
        if(obj.props.has('immediateEffect'))result.immediateEffect=effect(obj.props.get('immediateEffect'),obj.ctx,'viewSelected');
        prompts(result);return result;
    }
    if(['playCard','playCardFromHand','playCardFromOutOfPlay'].includes(kind)) {
        check(['target','ignoredRequirements','entersReady','playType','playAsType','adjustCost','nested','canPlayFromAnyZone','exploitValue','attachTargetCondition','enterPlayEffect','optional']);
        if(kind!=='playCard'&&(obj.props.has('playType')||obj.props.has('optional')))fail(obj.ctx,obj.node,'play-wrapper-owned-properties');
        const result={kind:'playCardSystem',helper:kind,target:target(obj.props.get('target'),obj.ctx,defaultTarget),ignoredRequirements:['phase'],optional:false,entersReady:false,playType:kind==='playCardFromOutOfPlay'?'playFromOutOfPlay':'playFromHand',playAsType:'any',nested:false,canPlayFromAnyZone:false};
        for(const key of ['ignoredRequirements','optional','entersReady','playType','playAsType','nested','canPlayFromAnyZone'])if(obj.props.has(key)){try{result[key]=literal(obj.props.get(key),obj.ctx);}catch(error){if(!error.diagnostic||!obj.ctx.pendingSystemConstruction)throw error;result[key]={kind:'systemPropertyValue',property:key,value:expr(obj.props.get(key),obj.ctx),readAt:'nativePropertyGeneration',invalidValue:'nativeAssertionOrSwitchContract'};}}
        if(obj.props.has('exploitValue'))result.exploitValue=derived(obj.props.get('exploitValue'),'playExploitValue');
        if(obj.props.has('attachTargetCondition'))result.attachTargetCondition=derived(obj.props.get('attachTargetCondition'),'playAttachFilter',['card','context']);
        if(obj.props.has('adjustCost')) {
            const cost=object(obj.props.get('adjustCost'),obj.ctx);
            if(!cost.props.has('costAdjustType'))fail(cost.ctx,cost.node,'play-cost-adjuster-type');
            const adjustment=literal(cost.props.get('costAdjustType'),cost.ctx);
            result.adjustCost=adjustmentProperties({...cost,props:new Map([...cost.props].filter(([key])=>key!=='costAdjustType'))},adjustment,api);
        }
        if(obj.props.has('enterPlayEffect')){const n=unwrap(obj.props.get('enterPlayEffect'));result.enterPlayEffect=ts.isArrayLiteralExpression(n)?{kind:'simultaneous',effects:n.elements.map(e=>effect(e,obj.ctx,'playedCard'))}:effect(n,obj.ctx,'playedCard');}
        return result;
    }
    if(['drawSpecificCard','revealAndDraw','reveal','moveToBottomOfDeck','moveToTopOfDeck','shuffleDeck'].includes(kind)) {
        const allowed={drawSpecificCard:['target','switch','switchTarget','shuffle','changePlayer'],revealAndDraw:['target','useDisplayPrompt','promptedPlayer','resolutionMode'],reveal:['target','useDisplayPrompt','promptedPlayer'],moveToBottomOfDeck:['target','changePlayer'],moveToTopOfDeck:['target','changePlayer'],shuffleDeck:['target']}[kind];check(allowed);
        const result={kind,target:target(obj.props.get('target'),obj.ctx,kind==='shuffleDeck'?'ownPlayer':defaultTarget)};
        for(const [key,value] of obj.props)if(key!=='target')result[key]=key==='switchTarget'?expr(value,obj.ctx):literal(value,obj.ctx);
        if(kind==='drawSpecificCard')for(const key of ['switch','shuffle','changePlayer'])result[key]??=false;
        return result;
    }
    if(kind==='chooseModalEffects') {
        check(['target','amountOfChoices','choices','canChooseSameOptionMoreThanOnce','activePromptTitle']);
        if(!obj.props.has('choices')||!obj.props.has('amountOfChoices'))fail(obj.ctx,obj.node,'modal-missing-choices-or-count');
        let choices;try{const raw=object(obj.props.get('choices'),obj.ctx);choices=[...raw.props].map(([label,value])=>({label,effect:effect(value,raw.ctx,defaultTarget)}));}catch(error){if(!error.diagnostic)throw error;choices=require('./view-deck-systems').modalChoiceDefinitions(obj.props.get('choices'),obj.ctx,api,defaultTarget);}
        const result={kind,target:target(obj.props.get('target'),obj.ctx,defaultTarget),amountOfChoices:derived(obj.props.get('amountOfChoices'),'modalChoiceCount'),canChooseSameOptionMoreThanOnce:obj.props.has('canChooseSameOptionMoreThanOnce')?literal(obj.props.get('canChooseSameOptionMoreThanOnce'),obj.ctx):false,choices};
        if(obj.props.has('activePromptTitle'))result.activePromptTitleParts=contextTitle(obj.props.get('activePromptTitle'),obj.ctx);
        return result;
    }
    return null;
}

function objectSystemBatch(input,ctx,mode,defaultTarget,api) {
    const {ts,unwrap,object,literal,effect,fail}=api;
    const n=unwrap(input),body=ts.isArrowFunction(n)||ts.isFunctionExpression(n)?unwrap(n.body):n;
    if(!ts.isObjectLiteralExpression(body))return null;
    const obj=object(input,ctx);
    for(const [key,value]of obj.props)if(!['gameSystems','resolutionMode'].includes(key))fail(obj.ctx,value,'unsupported-system-batch-field:'+key);
    if(!obj.props.has('gameSystems'))fail(obj.ctx,obj.node,'system-batch-missing-systems');
    const systems=unwrap(obj.props.get('gameSystems'));if(!ts.isArrayLiteralExpression(systems))fail(obj.ctx,systems,'system-batch-array');
    const resolutionMode=obj.props.has('resolutionMode')?literal(obj.props.get('resolutionMode'),obj.ctx):'someGameSystemsMustBeLegal';
    if(!['someGameSystemsMustBeLegal','allGameSystemsMustBeLegal','alwaysResolve'].includes(resolutionMode))fail(obj.ctx,obj.node,'system-batch-resolution-mode');
    return {kind:'systemBatch',mode,resolutionMode,effects:systems.elements.map(n=>effect(n,obj.ctx,defaultTarget))};
}

function deckExpression(input,ctx,api) {
    const {ts,unwrap,expr,fail}=api,n=unwrap(input);
    const isDeck=n=>ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['getTopCardOfDeck','getTopCardsOfDeck'].includes(n.expression.name.text);
    if(isDeck(n)) {
        const many=n.expression.name.text==='getTopCardsOfDeck';
        if(n.questionDotToken||n.expression.questionDotToken||n.arguments.length!==(many?1:0))fail(ctx,n,'deck-query-shape');
        const receiver=expr(n.expression.expression,ctx);
        return {kind:'deckQuery',operation:many?'topCards':'topCard',receiver,args:n.arguments.map(a=>expr(a,ctx)),order:'topFirst',empty:many?'emptyList':'null',...(many?{countAssertion:'positiveNonZero'}:{})};
    }
    if(ts.isPropertyAccessExpression(n)&&isDeck(unwrap(n.expression)))return {kind:'deckMember',receiver:expr(n.expression,ctx),field:n.name.text,optional:!!n.questionDotToken};
    return null;
}
module.exports={deckPlayEffect,objectSystemBatch,deckExpression};
