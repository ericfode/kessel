'use strict';
const test=require('node:test'), assert=require('node:assert/strict');
const {classifyExhaustResources,planExhaustResourcesProbes}=require('../pipeline/exhaust-resources-probes');
const {planCardProbes}=require('../pipeline/probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('actual complete source IR shapes plan both owners and physical resource boundaries',()=>{
    for(const id of ['ASH_216','TWI_185']) {
        const e=entry(id),p=planExhaustResourcesProbes(e,{catalogue});
        assert.ok(p); assert.equal(p.sourceHash,e.sourceHash);
        for(const owner of [0,1]) for(const readyResources of [0,1,2]) {
            const c=p.cases.find(c=>c.probe.owner===owner && c.probe.readyResources===readyResources && c.probe.trigger!=='when-played');
            assert.ok(c); assert.equal(c.probe.resourceLayout.filter(x=>!x).length,readyResources);
        }
        assert.deepEqual(planCardProbes(e,{catalogue}),p);
        assert.ok(p.uncovered.some(x=>x.id.endsWith('controller-changed-last-known-information')));
    }
});
test('on-attack companion has real play ground space empty entering source enemy-only and decline cases',()=>{
    const p=planExhaustResourcesProbes(entry('TWI_185'),{catalogue});
    for(const owner of [0,1]) {
        const cases=p.cases.filter(c=>c.probe.owner===owner);
        assert.ok(cases.some(c=>c.probe.trigger==='on-attack' && c.probe.decline));
        for(const targetArena of ['ground','space']) assert.ok(cases.some(c=>c.probe.trigger==='when-played' && c.probe.targetArena===targetArena));
        for(const scenario of ['no-units','entering-source','enemy-only','mixed-exhausted','all-exhausted']) assert.ok(cases.some(c=>c.probe.trigger==='when-played' && c.probe.scenario===scenario));
    }
});
test('classification is metadata and full IR based rather than card identity',()=>{
    const e=entry('ASH_216');e.setCode='CUSTOM_999';e.className='AnotherScout';e.card.title='Another';
    assert.ok(classifyExhaustResources(e));
});
test('unknown fields, companion bodies, dynamic amounts and incompatible metadata fail closed',()=>{
    for(const id of ['ASH_216','TWI_185']) for(const mutate of [
        e=>e.card.types.push('token'), e=>e.baseClass='LeaderUnitCard',e=>e.card.pilotText='Pilot',e=>e.piloting={},
        e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'whenPlayed',effect:{kind:'draw'}}),
        e=>e.abilities.card.find(a=>a.effect.kind==='exhaustResources').effect.amount={kind:'binary',op:'+',left:{kind:'const',value:0},right:{kind:'const',value:1}},
        e=>e.abilities.card.find(a=>a.effect.kind==='exhaustResources').effect.extra=true,
        e=>e.abilities.card.find(a=>a.effect.kind==='exhaustResources').effect.allowCostAdjustments=true,
        e=>e.abilities.card.find(a=>a.effect.kind==='exhaustResources').condition={kind:'const',value:true},
        e=>e.abilities.card.find(a=>a.effect.kind==='exhaustResources').effect.target='chosenPlayer',
    ]) {const e=entry(id);mutate(e);assert.equal(classifyExhaustResources(e),null);assert.equal(planExhaustResourcesProbes(e,{catalogue}),null);}
    const e=entry('TWI_185'); e.abilities.card[0].targetResolver.owner='any';assert.equal(classifyExhaustResources(e),null);
});
test('missing donors are explicitly uncovered',()=>{
    const p=planExhaustResourcesProbes(entry('ASH_216'),{catalogue:[]});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(x=>x.id.endsWith('construction-donors')));
});

test('played exhaustion boundaries explicitly distinguish ineffective selection from no feasible choice for each owner',()=>{
    const p=planExhaustResourcesProbes(entry('TWI_185'),{catalogue});
    for(const owner of [0,1]) {
        for(const scenario of ['mixed-exhausted','all-exhausted']) {
            const c=p.cases.find(c=>c.probe.owner===owner && c.probe.scenario===scenario);
            assert.ok(c); assert.equal(c.probe.trigger,'when-played');
            assert.equal(c.probe.decline,false); assert.equal(c.probe.targetArena,'ground');
            assert.deepEqual(c.probe.resourceLayout,[true,false,true,false]);
            assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));
        }
    }
});
