'use strict';

// Compiler-only contracts for payment, token creation and self play-cost
// adjustment. No callback is executed and none of these new operations implies
// runtime admission.
const constant = value => ({ kind: 'const', value });
const reference = (...path) => ({ kind: 'ref', path });
function exactFields(obj, fields, api) {
    for (const [key, value] of obj.props) if (!fields.includes(key)) api.fail(obj.ctx, value, 'cost-resource-field:' + key);
}
function callback(input, ctx, parameters, timing, api) {
    const {ts, unwrap, expr, fail} = api;
    const n = unwrap(input);
    if(ts.isIdentifier(n)&&ctx.constants.has(n.text)){let alias=n;const seen=new Set();while(ts.isIdentifier(alias)&&ctx.constants.has(alias.text)){if(seen.has(alias.text))fail(ctx,n,'cost-callback-alias-cycle');seen.add(alias.text);alias=unwrap(ctx.constants.get(alias.text));}if(ts.isArrowFunction(alias)||ts.isFunctionExpression(alias))fail(ctx,n,'cost-callback-alias');}
    if(ts.isFunctionExpression(n))fail(ctx,n,'nonlexical-cost-callback');
    if (!ts.isArrowFunction(n) && !ts.isFunctionExpression(n)) return expr(input, ctx);
    if (n.parameters.length > parameters.length || n.typeParameters?.length || n.parameters.some(p => !ts.isIdentifier(p.name) || p.initializer || p.dotDotDotToken || p.questionToken || p.modifiers?.length || ts.getDecorators(p)?.length) || new Set(n.parameters.map(p => p.name.text)).size !== n.parameters.length) fail(ctx, n, 'cost-resource-callback-binding');
    return {kind: 'boundCallback', parameters: parameters.slice(0, n.parameters.length), timing, value: expr(n, {...ctx, costAdjustmentScope:true}, parameters)};
}
function decreaseCostAbility(input, ctx, api) {
    const obj = api.object(input, ctx);
    exactFields(obj, ['title', 'condition', 'amount', 'attachTargetCondition', 'playType', 'matchAbilityCosts', 'matchCardEffectResourcePayments'], api);
    const effect = {
        kind: 'decreaseSelfPlayCost', sourceZone: 'any', targetPlayers: 'any',
        matchSourceIdentity: true, matchPrintedType: true, stage: 'standard_0',
        amount: obj.props.has('amount') ? callback(obj.props.get('amount'), obj.ctx, ['adjustedCard', 'adjusterPlayer', 'context'], 'costAdjustment', api) : constant(null),
        matchAbilityCosts: false, matchCardEffectResourcePayments: false
    };
    // The underlying adjuster has no amount default. Do not invent zero or one.
    if (!obj.props.has('amount')) api.fail(obj.ctx, obj.node, 'cost-decrease-missing-amount');
    for (const key of ['matchAbilityCosts', 'matchCardEffectResourcePayments']) if (obj.props.has(key)) {
        const value=api.literal(obj.props.get(key),obj.ctx);if(typeof value!=='boolean')api.fail(obj.ctx,obj.props.get(key),'cost-adjustment-boolean');effect[key]=value;
    }
    if(obj.props.has('playType'))effect.playType=api.literal(obj.props.get('playType'),obj.ctx);
    if(obj.props.has('attachTargetCondition'))effect.attachTargetCondition=callback(obj.props.get('attachTargetCondition'),obj.ctx,['attachTarget','context','adjusterSource'],'costAttachmentEligibility',api);
    const result = {trigger: 'costAdjustment', title: obj.props.has('title') ? api.displayText(obj.props.get('title'),obj.ctx) : '', costs: [], effect};
    if(obj.props.has('condition'))result.condition=callback(obj.props.get('condition'),obj.ctx,['context'],'constantCondition',api);
    return result;
}
function scalarProperty(input,ctx,api) {
    let n=api.unwrap(input);const seen=new Set();
    while(api.ts.isIdentifier(n)&&ctx.constants.has(n.text)){if(seen.has(n.text))api.fail(ctx,n,'cyclic-numeric-property');seen.add(n.text);n=api.unwrap(ctx.constants.get(n.text));}
    if(api.ts.isArrowFunction(n)||api.ts.isFunctionExpression(n))api.fail(ctx,n,'numeric-property-is-callback');
    return api.expr(input,ctx);
}
function playerTarget(input, ctx, fallback, api) {
    if(!input&&ctx.selectedPlayerTarget)return ctx.selectedPlayerTarget;
    if (input) {
        const n=api.unwrap(input),{ts}=api;
        if(ts.isCallExpression(n)&&!n.questionDotToken&&ts.isPropertyAccessExpression(n.expression)&&!n.expression.questionDotToken&&n.expression.name.text==='getPlayers'&&n.arguments.length===0) {
            const receiver=api.expr(n.expression.expression,{...ctx,arenaReceiver:true});
            if(receiver.kind==='ref'&&receiver.path.join('.')==='game')return {kind:'collection',name:'players',receiver,args:[]};
        }
    }
    const result=api.target(input,ctx,fallback);
    if(typeof result==='string'&&!['ownPlayer','enemyPlayer','bothPlayers'].includes(result))api.fail(ctx,input,'cost-resource-player-target');
    if(!ctx.callbackStage&&result?.kind==='ref'&&['source.controller','source.controller.opponent'].includes(result.path.join('.')))return {kind:'registrationSnapshot',value:result};
    return result;
}
function costResourceSystem(kind, obj, defaultTarget, api) {
    const {props,ctx,node}=obj;
    if(kind==='payCardPrintedCost') {
        exactFields(obj,['player','target'],api);
        if(!props.has('player'))api.fail(ctx,node,'printed-cost-missing-payer');
        return {kind:'payCardPrintedCost',target:api.target(props.get('target'),ctx,defaultTarget),payer:playerTarget(props.get('player'),ctx,null,api),paymentType:'cardEffectPayment',amountSource:'firstTargetCostAtPayment',zeroCostRequiresNoPayment:true,canCancelDuringPayment:false,allowCostAdjustments:true};
    }
    if(kind==='distributeAdvantageAmong') {
        exactFields(obj,['amountToDistribute','controller','cardTypeFilter','zoneFilter','maxTargets','canChooseNoTargets','canDistributeLess','cardCondition','player','activePromptTitle'],api);
        if(!props.has('amountToDistribute')||!props.has('canChooseNoTargets'))api.fail(ctx,node,'token-distribution-required-property');
        const canChooseNoTargets=api.literal(props.get('canChooseNoTargets'),ctx),canDistributeLess=props.has('canDistributeLess')?api.literal(props.get('canDistributeLess'),ctx):false;
        if(typeof canChooseNoTargets!=='boolean'||typeof canDistributeLess!=='boolean'||canDistributeLess&&!canChooseNoTargets)api.fail(ctx,node,'token-distribution-optionality');
        const chooser=props.has('player')?api.literal(props.get('player'),ctx):'self';if(!['self','opponent'].includes(chooser))api.fail(ctx,node,'token-distribution-chooser');
        const result={kind:'distributeTokenUpgrade',tokenType:'advantage',amount:callback(props.get('amountToDistribute'),ctx,['context'],'distributionQuantity',api),targetResolver:api.targetResolverProperties(obj),canChooseNoTargets,canDistributeLess,chooser,creationEvents:'perRecipient'};
        if(props.has('maxTargets'))result.maxTargets=api.literal(props.get('maxTargets'),ctx);
        if(props.has('activePromptTitle'))result.promptTitleParts=api.contextTitle(props.get('activePromptTitle'),ctx);
        return result;
    }
    if(kind==='forThisPhasePlayerEffect') {
        exactFields(obj,['effect','target','condition','title','ongoingEffectDescription','ongoingEffectTargetDescription'],api);
        if(!props.has('effect'))api.fail(ctx,node,'player-phase-missing-modifier');
        const raw=api.unwrap(props.get('effect')),nodes=api.ts.isArrayLiteralExpression(raw)?raw.elements:[raw];
        if(!nodes.length||nodes.length>8)api.fail(ctx,node,'player-phase-modifier-bound');
        const effects=nodes.map(n=>{const e=playerCostModifier(n,ctx,api);if(!e)api.fail(ctx,n,'unsupported-player-cost-modifier');return e;});
        return {kind:'phasePlayerCostEffect',target:playerTarget(props.get('target'),ctx,'ownPlayer',api),duration:'phase',effects,...(props.has('condition')?{condition:callback(props.get('condition'),ctx,['context'],'lastingEffectCondition',api)}:{})};
    }
    if(kind==='giveAdvantage') {
        exactFields(obj,['amount','target','createdBy','highPriorityRemoval'],api);
        const createdBy=props.has('createdBy')?api.literal(props.get('createdBy'),ctx):'self';
        if(!['self','opponent'].includes(createdBy))api.fail(ctx,node,'token-created-by');
        // highPriorityRemoval is deliberately ignored by the engine for Advantage.
        if(props.has('highPriorityRemoval')&&typeof api.literal(props.get('highPriorityRemoval'),ctx)!=='boolean')api.fail(ctx,node,'token-removal-priority-boolean');
        return {kind:'giveAdvantage',target:api.target(props.get('target'),ctx,defaultTarget),amount:props.has('amount')?scalarProperty(props.get('amount'),ctx,api):constant(1),createdBy,batch:'singleCreationEvent',tokenOwner:'recipientController'};
    }
    if(kind==='createCreditToken') {
        exactFields(obj,['amount','target'],api);
        return {kind:'createCreditToken',target:playerTarget(props.get('target'),ctx,'ownPlayer',api),amount:props.has('amount')?scalarProperty(props.get('amount'),ctx,api):constant(1),destination:'base',creationTiming:'eventUpdate'};
    }
    if(['payResources','payResourcesWithoutAdjustment','exhaustResources'].includes(kind)) {
        exactFields(obj,['amount','target'],api);
        if(!props.has('amount'))api.fail(ctx,node,'resource-payment-missing-amount');
        return {kind:kind==='payResources'?'payAdjustedResources':kind==='payResourcesWithoutAdjustment'?'payUnadjustedResources':'exhaustResources',target:playerTarget(props.get('target'),ctx,'enemyPlayer',api),amount:scalarProperty(props.get('amount'),ctx,api),paymentType:kind==='payResources'?'cardEffectPayment':'exhaustResources',isCost:kind==='payResourcesWithoutAdjustment',...(kind==='payResources'?{canCancelDuringPayment:false,allowCostAdjustments:true}: {allowCostAdjustments:false})};
    }
    return null;
}
function additionalCost(call, kind, ctx, api) {
    const {ts,unwrap,expr,object,fail}=api;
    if(call.questionDotToken||!ts.isPropertyAccessExpression(call.expression)||call.expression.questionDotToken)fail(ctx,call,'optional-cost-helper');
    if(kind==='discardCardsFromOwnHand') {
        if(call.arguments.length!==2)fail(ctx,call,'cost-helper-arity');
        return {kind:'discardHandCost',amount:expr(call.arguments[0],ctx),target:playerTarget(call.arguments[1],ctx,null,api),timing:'abilityCostPayment',mustFullyResolve:true,chooser:'affectedPlayer',cardTypeFilter:'any'};
    }
    if(kind==='dealDamageSpecific'||kind==='defeatSpecific') {
        const damage=kind==='dealDamageSpecific';if(call.arguments.length!==(damage?2:1))fail(ctx,call,'cost-helper-arity');
        return {kind:'directSystemCost',system:{kind:damage?'damage':'defeat',target:api.target(call.arguments[damage?1:0],ctx,'self'),isCost:true,...(damage?{amount:expr(call.arguments[0],ctx),damageType:'ability'}:{})},timing:'abilityCostPayment',mustFullyResolve:true};
    }
    const simple={returnSelfToHandFromPlay:{kind:'directSystemCost',system:{kind:'returnToHand',target:'self',isCost:true,destination:'hand'}},useTheForce:{kind:'useForceCost',target:'ownPlayer'},defeatSelf:{kind:'defeatSelfCost',target:'self'},exhaustFriendlyUnit:null,readyEnemyUnit:null};
    if(Object.hasOwn(simple,kind)&&simple[kind]) {
        if(call.arguments.length)fail(ctx,call,'cost-helper-arity');
        return {...simple[kind],timing:'abilityCostPayment',mustFullyResolve:true};
    }
    if(kind==='discardFromOwnDeck') {
        if(call.arguments.length>1)fail(ctx,call,'cost-helper-arity');
        return {kind:'discardOwnDeckCost',amount:call.arguments.length?expr(call.arguments[0],ctx):constant(1),target:'ownPlayer',timing:'abilityCostPayment',mustFullyResolve:true};
    }
    const selectorKinds={defeat:'defeat',discardCardFromOwnHand:'discardSpecificCard',returnToHandFromPlay:'returnToHand',exhaustFriendlyUnit:'exhaust',readyEnemyUnit:'ready',dealDamage:'damage'};
    if(!Object.hasOwn(selectorKinds,kind))return null;
    const amount=kind==='dealDamage'?expr(call.arguments[0],ctx):null;
    const propertyIndex=kind==='dealDamage'?1:0;
    if(call.arguments.length>(kind==='dealDamage'?2:1)||kind==='dealDamage'&&call.arguments.length!==2||['exhaustFriendlyUnit','readyEnemyUnit'].includes(kind)&&call.arguments.length)fail(ctx,call,'cost-helper-arity');
    const obj=call.arguments[propertyIndex]?object(call.arguments[propertyIndex],ctx):{props:new Map(),ctx,node:call};
    exactFields(obj,['controller','cardTypeFilter','zoneFilter','cardCondition','activePromptTitle'],api);
    const props=new Map(obj.props);
    const put=(name,value)=>props.set(name,ts.factory.createStringLiteral(value));
    if(!props.has('controller'))put('controller','self'); // MetaActionCost payment default
    if(kind==='discardCardFromOwnHand'){put('zoneFilter','hand');put('controller','self');}
    if(kind==='returnToHandFromPlay')put('zoneFilter','anyArena');
    if(kind==='exhaustFriendlyUnit'||kind==='readyEnemyUnit'){put('controller',kind==='readyEnemyUnit'?'opponent':'self');put('cardTypeFilter','unit');}
    const selector=api.targetResolverProperties({...obj,props});
    const titles={defeat:'Choose a card to defeat',discardCardFromOwnHand:'Choose a card to discard',returnToHandFromPlay:'Choose a card to return to hand',exhaustFriendlyUnit:'Choose a friendly unit to exhaust',readyEnemyUnit:'Choose an exhausted enemy unit to ready',dealDamage:null};
    const promptTitle=kind==='dealDamage'?[constant('Choose a card to deal '),amount,constant(' damage to')]:[constant(titles[kind])];
    // MetaActionCost supplies its helper title, but an explicit property wins
    // because GameSystem merges caller properties after additionalProperties.
    return {kind:'selectCardCost',selector,effect:{kind:selectorKinds[kind],target:'costSelection',isCost:true,...(amount?{amount,damageType:'ability'}:{})},timing:'abilityCostPayment',selectionName:'cost',chooser:'ownPlayer',mustFullyResolve:true,promptTitleParts:obj.props.has('activePromptTitle')?api.contextTitle(obj.props.get('activePromptTitle'),obj.ctx):promptTitle};
}
// Additional collection counts for cost callbacks use separate roots for the
// played card, adjuster's source player, and the ability context's player.
function costCollectionCount(node,ctx,api) {
    const {ts,unwrap,expr,object,literal,fail}=api;
    if(!ctx.costAdjustmentScope||!ts.isPropertyAccessExpression(node)||node.questionDotToken||node.name.text!=='length')return null;
    const call=unwrap(node.expression);
    if(!ts.isCallExpression(call)||call.questionDotToken||!ts.isPropertyAccessExpression(call.expression)||call.expression.questionDotToken||!['getInPlayCards','getArenaUpgrades'].includes(call.expression.name.text))return null;
    if(!ctx.callbackStage||call.arguments.length>1)fail(ctx,node,'cost-collection-shape');
    const receiver=expr(call.expression.expression,{...ctx,arenaReceiver:true});
    if(receiver.kind!=='ref'||!['adjusterPlayer','adjusterPlayer.opponent','player','opponent'].includes(receiver.path.join('.')))fail(ctx,node,'cost-collection-receiver');
    const obj=call.arguments.length?object(call.arguments[0],ctx):{props:new Map(),ctx,node:call};
    exactFields(obj,['type','trait','condition'],api);
    const filter={kind:'cardFilter'};
    for(const k of ['type','trait'])if(obj.props.has(k))filter[k]=literal(obj.props.get(k),obj.ctx);
    if(obj.props.has('condition'))filter.predicate=expr(obj.props.get('condition'),obj.ctx,['card']);
    return {kind:'query',name:call.expression.name.text==='getInPlayCards'?'controlledInPlayCount':'controlledArenaUpgradeCount',receiver,args:[filter]};
}


function costLimit(input,ctx,api) {
    const {ts,unwrap,access,literal,fail}=api,n=unwrap(input);
    if(!ts.isCallExpression(n)||n.questionDotToken)fail(ctx,n,'cost-adjuster-limit');
    const parts=access(n.expression,ctx);
    if(parts.length===3&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])&&parts[1]==='limit'&&parts[2]==='unlimited'&&n.arguments.length===0)return {kind:'costAdjusterLimit',scope:'unlimited',clonePerEffectApplication:true,tracksUsagePerPlayer:true};
    if(n.arguments.length!==1)fail(ctx,n,'cost-adjuster-limit');
    if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='limit'||!['perPhase','perRound','perGame','perPlayerPerGame'].includes(parts[2]))fail(ctx,n,'cost-adjuster-limit');
    const max=literal(n.arguments[0],ctx);if(!Number.isSafeInteger(max)||max<1||max>64)fail(ctx,n,'cost-adjuster-limit-bound');
    return {kind:'costAdjusterLimit',scope:parts[2],max,clonePerEffectApplication:true};
}
function adjustmentProperties(obj,adjustment,api) {
    const {props,ctx,node}=obj;
    const base=['cardTypeFilter','match','limit','playType','attachTargetCondition','matchAbilityCosts','matchCardEffectResourcePayments'];
    const specific={decrease:['amount'],increase:['amount'],free:[],modifyPayStage:['payStageAmount'],exhaustUnits:['canExhaustUnitCondition'],defeatResources:['amountPerResource','readyResourcesOnly'],damageUnits:['damagePerUnit','amountPerUnit','canDamageUnitCondition'],ignoreAllAspects:[],ignoreSpecificAspect:['ignoredAspect'],ignoreWildcardAspects:['wildcardAspects','ignoreCount']};
    if(!Object.hasOwn(specific,adjustment))api.fail(ctx,node,'unsupported-cost-adjustment-type:'+adjustment);
    exactFields(obj,[...base,...specific[adjustment]],api);
    const stages={decrease:'standard_0',increase:'increase_8',free:'standard_0',modifyPayStage:'payStage_5',exhaustUnits:'exhaustUnits_6',defeatResources:'defeatResources_4',damageUnits:'damageUnits_3',ignoreAllAspects:'standard_0',ignoreSpecificAspect:'standard_0',ignoreWildcardAspects:'ignoreWildcard_1'};
    const result={kind:'costAdjuster',adjustment,stage:stages[adjustment],cardTypeFilter:props.has('cardTypeFilter')?api.literal(props.get('cardTypeFilter'),ctx):'any',matchAbilityCosts:false,matchCardEffectResourcePayments:false};
    for(const key of ['matchAbilityCosts','matchCardEffectResourcePayments','readyResourcesOnly'])if(props.has(key)){
        const v=api.literal(props.get(key),ctx);if(typeof v!=='boolean')api.fail(ctx,node,'cost-adjustment-boolean');result[key]=v;
    }
    if(adjustment==='defeatResources'&&!props.has('readyResourcesOnly'))result.readyResourcesOnly=false;
    for(const key of ['playType','ignoredAspect'])if(props.has(key))result[key]=api.literal(props.get(key),ctx);
    if(adjustment==='ignoreWildcardAspects') {
        if(!props.has('wildcardAspects')||!props.has('ignoreCount'))api.fail(ctx,node,'wildcard-aspect-required-property');
        const set=api.unwrap(props.get('wildcardAspects')),ts=api.ts;
        if(!ts.isNewExpression(set)||!ts.isIdentifier(set.expression)||set.expression.text!=='Set'||set.arguments?.length!==1||set.typeArguments?.length)api.fail(ctx,set,'wildcard-aspect-set-shape');
        if(ctx.scope.has('Set')||ctx.constants.has('Set'))api.fail(ctx,set,'wildcard-aspect-set-binding');
        for(let n=set.parent;n;n=n.parent)if(n.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text==='Set'))api.fail(ctx,set,'wildcard-aspect-set-binding');
        if(ctx.source.statements.some(n=>ts.isImportDeclaration(n)&&n.importClause&&(n.importClause.name?.text==='Set'||n.importClause.namedBindings?.name?.text==='Set'||n.importClause.namedBindings?.elements?.some(e=>e.name.text==='Set'))))api.fail(ctx,set,'wildcard-aspect-set-binding');
        const values=api.literal(set.arguments[0],ctx),ignoreCount=api.literal(props.get('ignoreCount'),ctx);
        if(!Array.isArray(values)||values.length>64||values.some(v=>!['vigilance','command','aggression','cunning','heroism','villainy'].includes(v))||!Number.isSafeInteger(ignoreCount)||ignoreCount<0||ignoreCount>64)api.fail(ctx,node,'wildcard-aspect-value-bound');
        result.wildcardAspects={kind:'aspectSet',values:values.filter((v,i)=>values.indexOf(v)===i)};result.ignoreCount=ignoreCount;
    }
    if(props.has('limit'))result.limit=costLimit(props.get('limit'),ctx,api);
    if(props.has('match'))result.match=callback(props.get('match'),ctx,['adjustedCard','adjusterSource'],'costEligibility',api);
    if(props.has('attachTargetCondition'))result.attachTargetCondition=callback(props.get('attachTargetCondition'),ctx,['attachTarget','context','adjusterSource'],'costAttachmentEligibility',api);
    if(['decrease','increase'].includes(adjustment)) {
        if(!props.has('amount'))api.fail(ctx,node,'cost-adjustment-missing-amount');
        result.amount=callback(props.get('amount'),ctx,['adjustedCard','adjusterPlayer','context'],'costAdjustment',api);
    }
    if(adjustment==='modifyPayStage') {
        if(!props.has('payStageAmount'))api.fail(ctx,node,'cost-adjustment-missing-pay-stage-amount');
        result.payStageAmount=callback(props.get('payStageAmount'),ctx,['remainingCost'],'payStage',api);
    }
    for(const key of ['canExhaustUnitCondition','canDamageUnitCondition'])if(props.has(key))result[key]=callback(props.get(key),ctx,['paymentUnit','context'],'paymentTargetEligibility',api);
    for(const key of ['amountPerResource','damagePerUnit','amountPerUnit'])if(specific[adjustment].includes(key)) {
        if(!props.has(key))api.fail(ctx,node,'cost-adjustment-missing-'+key);
        result[key]=scalarProperty(props.get(key),ctx,api);
    }
    return result;
}
function playerCostModifier(input,ctx,api) {
    const {ts,unwrap,access,object}=api,n=unwrap(input);
    if(!ts.isCallExpression(n))return null;
    const parts=access(n.expression,ctx);
    const types={decreaseCost:'decrease',increaseCost:'increase',forFree:'free',modifyPayStageCost:'modifyPayStage',canExhaustUnitsInsteadOfResources:'exhaustUnits',canDefeatResourcesToDecreaseCost:'defeatResources',ignoreAllAspectPenalties:'ignoreAllAspects',ignoreSpecificAspectPenalties:'ignoreSpecificAspect'};
    if(parts.length!==3||!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(parts[0])||parts[1]!=='ongoingEffects'||!Object.hasOwn(types,parts[2]))return null;
    if(n.questionDotToken||n.arguments.length!==1)api.fail(ctx,n,'cost-modifier-arity');
    return adjustmentProperties(object(n.arguments[0],ctx),types[parts[2]],api);
}
function playerCostConstant(obj,api) {
    if(!obj.props.has('ongoingEffect'))return null;
    const modifier=playerCostModifier(obj.props.get('ongoingEffect'),obj.ctx,api);if(!modifier)return null;
    exactFields(obj,['title','condition','ongoingEffect','targetController','targetCardTypeFilter','targetZoneFilter','sourceZoneFilter'],api);
    const targetController=obj.props.has('targetController')?api.literal(obj.props.get('targetController'),obj.ctx):'self';
    if(!['self','opponent','any'].includes(targetController))api.fail(obj.ctx,obj.node,'player-cost-controller');
    const result={trigger:'constantPlayerCostAdjustment',title:obj.props.has('title')?api.displayText(obj.props.get('title'),obj.ctx):'',costs:[],targetController,effect:modifier};
    for(const key of ['targetCardTypeFilter','targetZoneFilter','sourceZoneFilter'])if(obj.props.has(key))result[key]=api.literal(obj.props.get(key),obj.ctx);
    if(obj.props.has('condition'))result.condition=callback(obj.props.get('condition'),obj.ctx,['context'],'constantCondition',api);
    return result;
}
function adjustCostAbility(input,ctx,api,validatedObject=null) {
    const obj=validatedObject||api.object(input,ctx),props=new Map(obj.props);
    if(!props.has('costAdjustType'))api.fail(obj.ctx,obj.node,'self-adjustment-missing-type');
    const type=api.literal(props.get('costAdjustType'),obj.ctx);props.delete('costAdjustType');props.delete('title');props.delete('condition');
    const effect=adjustmentProperties({...obj,props},type,api);
    const result={trigger:'costAdjustment',title:obj.props.has('title')?api.displayText(obj.props.get('title'),obj.ctx):'',costs:[],effect:{kind:'adjustSelfPlayCost',sourceZone:'any',targetPlayers:'any',matchSourceIdentity:true,matchPrintedType:true,adjuster:effect}};
    if(obj.props.has('condition'))result.condition=callback(obj.props.get('condition'),obj.ctx,['context'],'constantCondition',api);
    return result;
}
module.exports={playerTarget,scalarProperty,decreaseCostAbility,adjustCostAbility,costResourceSystem,additionalCost,costCollectionCount,playerCostConstant,playerCostModifier,adjustmentProperties,costCallback:callback};
