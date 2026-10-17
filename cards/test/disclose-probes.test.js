'use strict';
const test=require('node:test'), assert=require('node:assert/strict');
const {classifyDisclose,planDiscloseProbes}=require('../pipeline/disclose-probes');
const {planCardProbes}=require('../pipeline/probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const ids=['SEC_059','SEC_062','SEC_076','SEC_094','SEC_098','SEC_120','SEC_141','SEC_148','SEC_153','SEC_181','SEC_182','SEC_190','SEC_248'];
const entry=id=>catalogue.cards.find(e=>e.setCode===id);

test('all thirteen bounded disclosure shapes construct both owners and real optional branches',()=>{
    for(const id of ids){
        const e=entry(id);
        assert.equal(e.status, 'supported', id);
        assert.equal(classifyDisclose(e)?.family,'disclose',id);
        const plan=planDiscloseProbes(e,{catalogue});
        assert.ok(plan);
            assert.ok(plan.cases.length>=6,id);
            assert.deepEqual(new Set(plan.cases.map(c=>c.probe.sourceSeat)),new Set([1,2]));
            assert.ok(['positive','decline','impossible'].every(v=>plan.cases.some(c=>c.probe.variant===v)));
            assert.deepEqual(planCardProbes(e,{catalogue}),plan,'Central planner uses complete disclosure construction');
            for(const c of plan.cases){
                assert.deepEqual(c.focusCardIds,[id]);
                assert.ok(c.setupCardIds.includes(c.probe.donors.spy));
                assert.equal(c.source.module,'../pipeline/probe-fixture.js');
            }
    }
});

test('classification rejects incomplete source metadata and additional effects rather than covering a partial card',()=>{
    const base=entry('SEC_062');if(base.status!=='supported')return;
    for(const change of [
        e=>e.baseClass='LeaderUnitCard',
        e=>e.card.types=['unit','token'],
        e=>e.card.hp++,
        e=>e.card.pilotText='Pilot',
        e=>e.abilities.card[0].costs.push({kind:'exhaustSelf'}),
        e=>e.abilities.card[0].condition={kind:'const',value:true},
        e=>e.abilities.card[0].effect.mode='some',
        e=>delete e.abilities.card[0].effect.aspectMultiplicity,
        e=>e.abilities.card[0].effect.selectionContext='sourcePlayer',
        e=>e.abilities.card[0].effect.selectedCardsEventField='cards',
        e=>e.abilities.card[0].effect.triggeredNoMatchPolicy='instant',
        e=>e.abilities.card[0].effect.aspects.push('vigilance'),
        e=>e.abilities.card[0].ifYouDo.effect.amount.value=2,
        e=>e.abilities.card.push({...e.abilities.card[0]}),
        e=>e.abilities.unit=[{trigger:'whenPlayed',effect:{kind:'draw'}}],
        e=>e.card.keywords=['sentinel']
    ]){
        const e=structuredClone(base);change(e);assert.equal(classifyDisclose(e),null);
    }
});

test('missing exact native dependency remains an uncovered construction',()=>{
    const e=entry('SEC_141');if(e.status!=='supported')return;
    const withoutSpy={...catalogue,cards:catalogue.cards.filter(c=>c.setCode!=='TOKEN_SPY')};
    const plan=planDiscloseProbes(e,{catalogue:withoutSpy});
    assert.equal(plan.cases.length,0);
    assert.ok(plan.uncovered.some(x=>x.id.endsWith(':source-donor-contract')));
});

test('source identity is metadata, while opponent discard is part of the complete shape', () => {
    const e=structuredClone(entry('SEC_153'));
    e.setCode='TST_999'; e.id='another-id'; e.internalName='other-name';
    assert.equal(classifyDisclose(e)?.mode,'death-discard');
    e.abilities.card[0].ifYouDo.effect.target='ownPlayer';
    assert.equal(classifyDisclose(e),null);
});
