'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { compileSource, loadEnums } = require('../compiler');
const engine = path.resolve(__dirname, '../../forceteki/server');
const enums = loadEnums(engine);
function compile(body, base = 'NonLeaderUnitCard') {
    return compileSource(`export default class Fresh extends ${base} {
        getImplementationId() { return {id:'fresh-entry-effect',internalName:'fresh-entry-effect'}; }
        setupCardAbilities(registrar, AbilityHelper) { ${body} }
    }`, 'fresh.ts', enums);
}
const con = value => ({kind:'const',value});
const ref = (...path) => ({kind:'ref',path});
const supported = (body,base) => {const c=compile(body,base);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];};
const immediate = (helper) => `registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.${helper}});`;
const rejection = (body, pattern, base) => {
    const card = compile(body, base);
    assert.equal(card.status, 'unsupported');
    if (pattern) assert.ok(card.diagnostics.some(d => d.pattern === pattern), JSON.stringify(card.diagnostics));
};
test('Mandalorian factory preserves inherited exact defaults and scalar overrides', () => {
    assert.deepEqual(compile(immediate('createMandalorian()')).abilities.card[0].effect,
        {kind:'createTokenUnit',tokenType:'mandalorian',amount:{kind:'const',value:1},target:'ownPlayer',entersReady:false});
    const card = compile(immediate('createMandalorian(ctx=>({amount:2,entersReady:true,target:ctx.player.opponent}))'));
    assert.equal(card.status, 'supported');
    assert.equal(card.abilities.card[0].effect.target, 'enemyPlayer');
    for (const field of ['tokenType:"spy"', 'isCost:true', 'unknown:true']) rejection(immediate(`createMandalorian(ctx=>({${field}}))`));
    for(const [field,amount,controllers] of [['enterPlayEffect:AbilityHelper.immediateEffects.draw()',1,'ownPlayer'],['amount:6',6,'ownPlayer'],['target:[ctx.player,ctx.player.opponent]',1,{kind:'array',items:[ref('player'),ref('opponent')]}]]) {
        const effect=supported(immediate(`createMandalorian(ctx=>({${field}}))`)).effect;
        const expected={kind:'createTokenUnitSystem',tokenType:'mandalorian',amount:con(amount),controllers,entersReady:false,creationTiming:'eventUpdate',entryTiming:'contingentEvents'};
        if(field.startsWith('enterPlay'))Object.assign(expected,{entryEffects:[{kind:'tokenEntryEffect',targetBinding:'enteringUnit',forceEventTarget:true,system:{kind:'draw',target:'ownPlayer',amount:con(1)}}],entryEffectTiming:'afterEntryBeforeStateCheck'});
        assert.deepEqual(effect,expected);
    }
    const captured=supported('registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.createMandalorian(),then:ctx=>({immediateEffect:AbilityHelper.immediateEffects.damage({amount:ctx.events[0].amount})})});').then.effect.amount;
    assert.deepEqual({...captured,contextBinding:{...captured.contextBinding,id:undefined}}, {kind:'dataPath',root:['context'],contextBinding:{id:undefined,role:'then',bindAt:'callbackInvocation',readAt:'expressionEvaluation',reference:'lexicalContextObject'},steps:[{key:'events',optional:false},{index:con(0),optional:false},{key:'amount',optional:false}]});
});
test('deck discard keeps legacy defaults and explicit general-system amount guards', () => {
    for (const amount of [0,1,3,5]) {
        const card = compile(immediate(`discardFromDeck({amount:${amount}})`));
        assert.equal(card.status,'supported');
        assert.deepEqual(card.abilities.card[0].effect,{kind:'discardDeck',amount:{kind:'const',value:amount},target:'enemyPlayer'});
    }
    assert.equal(compile(immediate('discardFromDeck(ctx=>({amount:1,target:ctx.player}))')).abilities.card[0].effect.target,'ownPlayer');
    rejection(immediate('discardFromDeck()'),'deck-discard-missing-amount');
    for(const field of ['amount:1,isCost:true','amount:1,unknown:true']) rejection(immediate(`discardFromDeck(ctx=>({${field}}))`));
    const expected=(amount,target='enemyPlayer')=>({kind:'discardDeckSystem',target,amount,assertAmountNonNegative:true,selection:'topCardsInDeckOrder',amountClamp:'availableDeckSize',events:['discardSpecificCardPerCard','overallDiscardEvent'],zeroAmount:'emitOverallEventAndReturnFromAllPlayerLoop',overallEventCondition:'alwaysTrue',mustFullyResolveRequiresFullAmount:true});
    for(const [field,amount,target] of [['amount:-1',{kind:'unary',op:'-',value:con(1)}],['amount:6',con(6)],['amount:1.5',con(1.5)],['amount:ctx.source.power',ref('source','power')],['amount:1,target:[ctx.player,ctx.player.opponent]',con(1),'bothPlayers']])assert.deepEqual(supported(immediate(`discardFromDeck(ctx=>({${field}}))`)).effect,expected(amount,target));
    const factory=supported(immediate('discardFromDeck(ctx=>({amount:1,target:ctx.source}))')).effect;
    assert.equal(factory.kind,'systemPropertiesFactory');assert.equal(factory.invocationPolicy,'eachNativePropertyGeneration');
    assert.deepEqual(factory.parameters,[{root:'attackBinding1_0',role:'systemPropertiesContext'}]);
    assert.deepEqual(factory.captures.map(({name,...capture})=>capture),[{role:'staticProperty',value:ref('attackBinding1_0','source'),evaluateAt:'propertyFactoryInvocation',order:0}]);
    assert.deepEqual(factory.effect,expected(con(1),ref('systemFactoryCaptures',factory.captures[0].name)));
    const event=supported('registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.discardFromDeck({amount:1}),ifYouDo:ctx=>({condition:()=>ctx.events[0].card.isUnit(),immediateEffect:AbilityHelper.immediateEffects.draw({amount:1})})});').ifYouDo.condition;
    assert.equal(event.name,'isUnit');assert.deepEqual({...event.receiver,contextBinding:undefined},{kind:'eventResult',contextBinding:undefined,collection:'events',eventIndex:0,path:['card'],optionalAccess:[]});
    assert.equal(event.receiver.contextBinding.role,'ifYouDo');assert.equal(event.receiver.contextBinding.reference,'lexicalContextObject');
});
test('attached keyword retains lexical upgrade source and attach candidate independently', () => {
    const card=compile(`registrar.setAttachCondition(ctx=>!ctx.attachTarget.hasSomeTrait(Trait.Vehicle));
        registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Restore,amount:2,gainCondition:ctx=>ctx.source.parentCard.hasSomeTrait(Trait.Force)});`, 'UpgradeCard');
    assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));
    assert.deepEqual(card.attachCondition.value.receiver,{kind:'ref',path:['card']});
    const ability=card.abilities.card[0];
    assert.equal(ability.effect.target,'parent');
    assert.equal(ability.title,'Give Restore 2 to the attached unit');
    assert.deepEqual(ability.condition.receiver,{kind:'ref',path:['source','parentCard']});
    for(const field of ['ability:{immediateEffect:AbilityHelper.immediateEffects.draw()}','unknown:true','title:"ignored"','amount:ctx=>2']) rejection(`registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Sentinel,${field}});`,null,'UpgradeCard');
    const bounty=supported('registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Bounty,ability:{title:"draw",immediateEffect:AbilityHelper.immediateEffects.draw()}});','UpgradeCard').effect;
    assert.equal(bounty.kind,'gainBounty');assert.equal(bounty.target,'parent');assert.equal(bounty.abilitySource,'recipient');
    assert.deepEqual(bounty.ability.effect,{kind:'draw',target:'ownPlayer',amount:con(1)});assert.deepEqual(bounty.ability.bounty.events,['onCardDefeated','onCardCaptured']);assert.equal(bounty.ability.bounty.contextPlayer,'collector');
    rejection('registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Sentinel});','attached-source-side');
    rejection('registrar.setAttachCondition(ctx=>true);registrar.setAttachCondition(ctx=>false);','duplicate-attach-condition','UpgradeCard');
    rejection('registrar.setAttachCondition(ctx=>{ctx.source.damage=1;return true;});',null,'UpgradeCard');
    rejection('registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Sentinel}); registrar.addActionAbility({immediateEffect:AbilityHelper.immediateEffects.unmodeled()});',null,'UpgradeCard');
});
test('full sources preserve token result, discarded-card identity and recipient Bounty contracts', () => {
    const sources = [
        ['game/cards/08_ASH/units/DuchesssProtector.ts',true],
        ['game/cards/08_ASH/events/FoundlingRescue.ts',true],
        ['game/cards/08_ASH/events/BuyTime.ts',true],
        ['game/cards/07_LAW/units/BraccaShipbreaker.ts',true],
        ['game/cards/07_LAW/units/BT-1Blastomech.ts',true],
        ['game/cards/01_SOR/upgrades/Protector.ts',true],
        ['game/cards/02_SHD/upgrades/DeathMark.ts',true]
    ];
    for(const [file,supported] of sources) {
        const card=compileSource(fs.readFileSync(path.join(engine,file),'utf8'),file,enums);
        assert.equal(card.status,supported?'supported':'unsupported',file+JSON.stringify(card.diagnostics));
        if(file.endsWith('BuyTime.ts')){const target=card.abilities.card[0].then.effect.target;assert.equal(target.collection,'resolvedEvents');assert.deepEqual(target.path,['generatedTokens']);assert.equal(target.eventIndex,0);assert.equal(target.contextBinding.role,'then');assert.deepEqual(target.optionalAccess,[2]);}
        if(file.endsWith('FoundlingRescue.ts')){assert.deepEqual(card.abilities.card[0].effect.effects.map(x=>x.kind),['select','createTokenUnit']);}
        if(file.endsWith('BT-1Blastomech.ts')){assert.equal(card.abilities.card[0].ifYouDo.ifYouDoCondition.name,'hasSomeAspect');assert.deepEqual(card.abilities.card[0].ifYouDo.ifYouDoCondition.receiver.path,['card']);}
        if(file.endsWith('DeathMark.ts')){assert.equal(card.abilities.card[0].effect.kind,'gainBounty');assert.equal(card.abilities.card[0].effect.ability.effect.amount.value,2);}
    }
});
