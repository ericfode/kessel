'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyHanAdvantage,planHanAdvantageProbes}=require('./han-advantage-probes');
const entry=()=>structuredClone(catalogue.cards.find(e=>e.setCode==='ASH_158'));
test('full mixed native Advantage3 registration retains mandatory selection, intrinsic Saboteur and both seats',()=>{
    const e=entry(),p=planHanAdvantageProbes(e,{catalogue});assert.ok(p);assert.equal(p.cases.length,16);
    for(const owner of [0,1])for(const scenario of ['friendly','enemy-ground','enemy-space','self','keep-old','keep-new','append-order','saboteur-combat'])assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario===scenario));
    for(const c of p.cases) {assert.ok(c.assertionStages.includes('single-native-creation-event'));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));}
    const renamed=entry();renamed.setCode='CUSTOM_001';renamed.className='Another';renamed.card.title='Another';assert.ok(classifyHanAdvantage(renamed));
});
test('reject partial metadata, companion abilities and mutations of whole simultaneous registration',()=>{
    for(const mutate of [e=>e.baseClass='LeaderUnitCard',e=>e.card.types.push('token'),e=>e.card.keywords=[],e=>e.keywordValues={},e=>e.card.unique=false,e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.abilities.other=[],e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.card[0].optional=true,e=>e.abilities.card[0].then={},e=>e.abilities.card[0].costs.push({kind:'exhaustSelf'}),e=>e.abilities.card[0].effect.effects.reverse(),e=>e.abilities.card[0].effect.effects[0].amount.value=2,e=>e.abilities.card[0].effect.effects[1].optional=true,e=>e.abilities.card[0].effect.effects[1].targetResolver.owner='self',e=>e.abilities.card[0].effect.effects[1].effect.effects[0].amount.value=2,e=>e.abilities.card[0].effect.effects[1].effect.effects[0].createdBy='chosen',e=>e.abilities.card[0].effect.effects[1].effect.effects[0].batch='multiple',e=>e.abilities.card[0].effect.effects[1].effect.effects[0].tokenOwner='sourceController']) {const e=entry();mutate(e);assert.equal(classifyHanAdvantage(e),null);}
});
test('missing construction donors and unconstructible interactions remain uncovered',()=>{
    const p=planHanAdvantageProbes(entry());assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('native-token-dependencies')));
    const full=planHanAdvantageProbes(entry(),{catalogue});for(const suffix of ['foreign-controller','shield-self-on-entry','simultaneous-lethal-self'])assert.ok(full.uncovered.some(o=>o.id.endsWith(suffix)));
});
test('native Advantage contract rejects missing and malformed pool dependencies',()=>{
    const {nativeAdvantage}=require('./han-advantage-probes');
    const token=catalogue.cards.find(e=>e.setCode==='TOKEN_ADVANTAGE');assert.ok(nativeAdvantage(token));
    for(const mutate of [e=>e.card.upgradeHp=1,e=>e.card.traits=[],e=>e.sourceHash='0'.repeat(64),e=>e.abilities.card.push({}),e=>e.card.types=['upgrade']]) {const bad=structuredClone(token);mutate(bad);assert.equal(nativeAdvantage(bad),false);const p=planHanAdvantageProbes(entry(),{catalogue:catalogue.cards.map(e=>e===token?bad:e)});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('native-token-dependencies')));}
});
