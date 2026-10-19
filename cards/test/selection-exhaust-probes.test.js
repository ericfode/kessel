'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifySelectionExhaust,planSelectionExhaustProbes,resourceKeywordDonor}=require('../pipeline/selection-exhaust-probes');
const {planCardProbes}=require('../pipeline/probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('whole retained selected-player action produces both-seat effective and ineffective target boundaries',()=>{
    const e=entry('SEC_216'),p=planSelectionExhaustProbes(e,{catalogue});
    assert.ok(p);assert.equal(p.sourceHash,e.sourceHash);assert.deepEqual(planCardProbes(e,{catalogue}),p);
    for(const owner of [0,1]) for(const chosenPlayer of ['self','opponent']) {
        for(const [ownReady,enemyReady] of [[2,2],[1,2],[2,1],[0,2],[2,0]]) assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.chosenPlayer===chosenPlayer&&c.probe.ownReady===ownReady&&c.probe.enemyReady===enemyReady&&c.probe.scenario==='normal'));
        for(const scenario of ['reverse-order','both-zero']) assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.chosenPlayer===chosenPlayer&&c.probe.scenario===scenario));
    }
    assert.equal(p.cases.length,36);
    for(const c of p.cases) {assert.ok(c.assertionStages.includes('paid-source-at-menu'));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));}
});
test('actual prepayment cancel and exhausted-source unavailability have separate constructions',()=>{
    const p=planSelectionExhaustProbes(entry('SEC_216'));
    for(const owner of [0,1]) {
        assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario==='cancel'&&c.probe.sourceReady));
        assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario==='source-exhausted'&&!c.probe.sourceReady));
    }
    assert.ok(!p.uncovered.some(x=>x.dimension==='cancel-protocol'));
    for(const d of ['foreign-controller','native-credit-not-consumed','live-resource-sensitive-aura']) assert.ok(p.uncovered.some(x=>x.id.endsWith(d)));
});
test('ordinary metadata and IR classify without source identity shortcuts',()=>{
    const e=entry('SEC_216');e.setCode='CUSTOM_001';e.className='Renamed';e.card.title='Renamed';e.card.text='';e.card.cost=3;e.card.hp=4;e.card.arena='space';assert.ok(classifySelectionExhaust(e));
});
test('reject complete-source companions and metadata outside ordinary unit domain',()=>{
    for(const mutate of [e=>e.abilities.card.push(structuredClone(e.abilities.card[0])),e=>e.abilities.unit.push({}),e=>e.abilities.other=[],e=>e.card.types.push('token'),e=>e.baseClass='LeaderUnitCard',e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.card.deployBox='Deploy',e=>e.card.keywords.push('hidden'),e=>e.keywordValues.hidden=1,e=>e.card.upgradeHp=1,e=>e.card.cost=99]) {
        const e=entry('SEC_216');mutate(e);assert.equal(classifySelectionExhaust(e),null);assert.equal(planSelectionExhaustProbes(e),null);
    }
});
test('reject raw wrapper, resolver, nested amount, target and cost mutations',()=>{
    const mutations=[a=>a.optional=false,a=>a.title='',a=>a.then={},a=>a.costs=[],a=>a.costs.push({kind:'resources',amount:{kind:'const',value:1}}),a=>a.effect.extra=true,a=>a.effect.target='enemyPlayer',a=>a.effect.name='other',a=>a.effect.system='selectCard',a=>a.effect.contract='other',a=>a.effect.legality='always',a=>a.effect.resolver.chooser='opponent',a=>a.effect.resolver.mode='multiplePlayers',a=>a.effect.resolver.resultBinding='selectedPlayer',a=>a.effect.resolver.contextBinding='other',a=>a.effect.resolver.delayOpponentBeforeCosts=false,a=>a.effect.resolver.extra=true,a=>a.effect.innerSystem.amount={kind:'binary',op:'+',left:{kind:'const',value:0},right:{kind:'const',value:1}},a=>a.effect.innerSystem.amount.value=2,a=>a.effect.innerSystem.target='ownPlayer',a=>a.effect.innerSystem.target.path=['target'],a=>a.effect.innerSystem.isCost=true,a=>a.effect.innerSystem.allowCostAdjustments=true,a=>a.effect.innerSystem.extra=true];
    for(const mutate of mutations) {const e=entry('SEC_216');mutate(e.abilities.card[0]);assert.equal(classifySelectionExhaust(e),null);}
    for(const id of ['ASH_216','TWI_185','JTL_250','SEC_235','LOF_255']) assert.equal(classifySelectionExhaust(entry(id)),null);
});

test('live keyword interaction uses a complete source-bound donor and rejects altered donor semantics',()=>{
    const donor=entry('HMW_117');assert.ok(resourceKeywordDonor(donor));
    const p=planSelectionExhaustProbes(entry('SEC_216'),{catalogue});
    for(const owner of [0,1]) for(const chosenPlayer of ['self','opponent']) {
        const c=p.cases.find(c=>c.probe.owner===owner&&c.probe.chosenPlayer===chosenPlayer&&c.probe.scenario==='live-resource-keywords');
        assert.ok(c);assert.equal(c.probe.ownReady,1);assert.equal(c.probe.enemyReady,1);
        assert.ok(c.setupCardIds.includes(donor.setCode));assert.equal(c.probe.interactionSourceHash,donor.sourceHash);
    }
    assert.ok(!p.uncovered.some(x=>x.id.endsWith('live-resource-sensitive-aura')));
    for(const mutate of [e=>e.abilities.card[0].effect.amount.path=['player','readyResourceCount'],e=>e.abilities.card[1].condition.right.value=1,e=>e.abilities.card.push({}),e=>e.card.types.push('token'),e=>e.abilities.card[0].then={}]) {
        const bad=structuredClone(donor);mutate(bad);assert.equal(resourceKeywordDonor(bad),false);
        const q=planSelectionExhaustProbes(entry('SEC_216'),{catalogue:[bad]});assert.ok(q.uncovered.some(x=>x.id.endsWith('live-resource-sensitive-aura')));
    }
});
