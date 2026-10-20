'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyCrossArena,planCrossArenaProbes}=require('./cross-arena-probes');
const entry=()=>structuredClone(catalogue.cards.find(e=>e.setCode==='ASH_037'));
test('full cross-arena recipe includes intrinsic Support and both copied-permission recipient arenas',()=>{
    const e=entry(),p=planCrossArenaProbes(e,{catalogue});assert.ok(p);
    for(const owner of [0,1])for(const recipientArena of ['ground','space'])for(const scenario of ['normal','cross-unit-only','ground-sentinel','space-sentinel','both-sentinels','decline'])assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.mode==='support'&&c.probe.recipientArena===recipientArena&&c.probe.scenario===scenario));
    for(const owner of [0,1])assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario==='saboteur'));
    for(const c of p.cases)assert.ok(c.assertionStages.includes(c.probe.terminalFatigue?'retained-snapshot-terminal-export':c.probe.expectedRecipientSelectable===false?'pre-grant-recipient-exclusion':'post-attack-permission-cleanup'));
    const renamed=entry();renamed.setCode='CUSTOM_001';renamed.className='Other';renamed.card.title='Other';assert.ok(classifyCrossArena(renamed));
});
test('reject partial printed permission, malformed Support, identity aliases and foreign companion semantics',()=>{
    for(const mutate of [e=>e.baseClass='LeaderUnitCard',e=>e.card.types.push('token'),e=>e.card.keywords=[],e=>e.keywordValues={},e=>e.keywordValues.support=2,e=>e.card.pilotText='Pilot',e=>e.card.deployBox='Deploy',e=>e.abilities.other=[],e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.card[0].optional=true,e=>e.abilities.card[0].condition={},e=>e.abilities.card[0].effect.effects.pop(),e=>e.abilities.card[0].effect.effects.reverse(),e=>e.abilities.card[0].effect.effects[0].duration='phase',e=>e.abilities.card[0].effect.effects[0].target='chosen',e=>e.abilities.card[0].effect.effects[0].permission='canAttackSpaceArenaFromGroundArena']){const e=entry();mutate(e);assert.equal(classifyCrossArena(e),null);}
});
test('no construction donor produces no invented coverage',()=>{
    const p=planCrossArenaProbes(entry());assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('construction-donors')));
});

test('intrinsic cannot-base recipient is excluded before copied permission, and cancellation grants persist until real attacks',()=>{
    const {wicketDonor}=require('./cross-arena-probes');const donor=catalogue.cards.find(e=>e.setCode==='ASH_034');assert.ok(wicketDonor(donor));
    const p=planCrossArenaProbes(entry(),{catalogue});assert.equal(p.cases.length,46);
    for(const owner of [0,1]) {
        const c=p.cases.find(c=>c.probe.owner===owner&&c.probe.scenario==='wicket-only-cross-defender');assert.ok(c);assert.equal(c.probe.expectedRecipientSelectable,false);assert.equal(c.probe.wicketCode,'ASH_034');assert.ok(c.setupCardIds.includes('ASH_034'));assert.ok(!c.setupCardIds.includes('SOR_072'));
        for(const recipientArena of ['ground','space'])for(const scenario of ['defender-cancel','multiple-cancel-same-recipient']){const c=p.cases.find(c=>c.probe.owner===owner&&c.probe.recipientArena===recipientArena&&c.probe.scenario===scenario);assert.ok(c);assert.equal(c.probe.cancelDefender,true);if(scenario==='multiple-cancel-same-recipient'){assert.equal(c.probe.multipleCancellations,true);assert.equal(c.probe.resourceCount,30);}}
    }
    assert.ok(p.cases.every(c=>!c.setupCardIds.includes('SOR_072')));
    for(const mutate of [e=>e.abilities.card[0].effect.modifier.effectType='canAttackBase',e=>e.card.keywords=[],e=>e.abilities.card.push({})]){const bad=structuredClone(donor);mutate(bad);assert.equal(wicketDonor(bad),false);}
});

test('natural terminal cancelled-grant witnesses use both seats and ordinary play through natural regroups',()=>{
    const p=planCrossArenaProbes(entry(),{catalogue});assert.equal(p.cases.length,46);
    for(const owner of [0,1]){const c=p.cases.find(c=>c.probe.owner===owner&&c.probe.terminalFatigue);assert.ok(c);assert.equal(c.probe.mode,'support');assert.equal(c.probe.recipientArena,'ground');assert.equal(c.probe.scenario,'support-cancel-terminal-fatigue');assert.equal(c.probe.cancelDefender,true);assert.equal(c.probe.resourceCount,12);assert.ok(c.assertionStages.includes('legal-pass-only-through-natural-regroups'));assert.ok(!c.assertionStages.includes('post-attack-permission-cleanup'));}
});
