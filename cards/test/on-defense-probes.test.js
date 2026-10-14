'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyOnDefense,planOnDefenseProbes}=require('../pipeline/on-defense-probes');
const catalogue=require('../../compiler/generated/catalog.json'),ids=['LOF_047','SHD_035','SOR_196','TS26_024','TWI_049','TWI_083'];
const plan=id=>planOnDefenseProbes(catalogue.cards.find(e=>e.setCode===id),{catalogue});
test('complete defense family accepts supported scalar source shapes and rejects unmodeled fields',()=>{
    for(const id of ids){const raw=catalogue.cards.find(e=>e.setCode===id);assert.ok(classifyOnDefense(raw));const e=structuredClone(raw);e.abilities.card[0].ifYouDo={effect:{kind:'draw',target:'ownPlayer',amount:{kind:'const',value:1}}};assert.equal(classifyOnDefense(e),null);}
    for(const id of ['SOR_196','TS26_024','TWI_049','TWI_083','SEC_187']){const e=structuredClone(catalogue.cards.find(e=>e.setCode===id));e.abilities.card[0].optional=true;assert.equal(classifyOnDefense(e),null);}
    const spaceToken=structuredClone(catalogue.cards.find(e=>e.setCode==='TWI_049'));spaceToken.abilities.card[0].effect.tokenType='tieFighter';assert.equal(classifyOnDefense(spaceToken),null);spaceToken.abilities.card[0].effect.tokenType='notAToken';assert.equal(classifyOnDefense(spaceToken),null);
    const missing=structuredClone(catalogue.cards.find(e=>e.setCode==='LOF_047'));delete missing.abilities.card[0].effect;assert.equal(classifyOnDefense(missing),null);missing.abilities.card[0].effect=catalogue.cards.find(e=>e.setCode==='LOF_047').abilities.card[0].effect;missing.abilities.card[0].optional='true';assert.equal(classifyOnDefense(missing),null);
    const e=structuredClone(catalogue.cards.find(e=>e.setCode==='SHD_035'));e.abilities.card[0].targetResolver.predicate={kind:'const',value:true};assert.equal(classifyOnDefense(e),null);
});
test('the defense-only adapter leaves Hidden lifecycle explicit with or without intrinsic extraction',()=>{
    const entry=catalogue.cards.find(e=>e.setCode==='SEC_187');
    assert.deepEqual(entry.keywordValues,{hidden:1});
    for(const keywordValues of [entry.keywordValues,{}]){
        const p=planOnDefenseProbes({...entry,keywordValues},{catalogue});
        assert.ok(p.uncovered.some(o=>o.id.endsWith(':hidden-lifecycle')));
    }
});
test('defense probes construct owner priorities, real attack keyword donors, lethal capture and cross-arena selectors',()=>{
    for(const id of ids){const p=plan(id);assert.deepEqual(p,plan(id));for(const variant of ['restore-attacker-first','restore-defender-first','saboteur-attacker-first','saboteur-defender-first','onattack-lethal-attacker-first','onattack-lethal-defender-first','actual-ambush'])assert.ok(p.cases.some(c=>c.name.endsWith(variant)),id+':'+variant);}
    assert.ok(plan('SHD_035').cases.some(c=>c.name.endsWith('experience-other-arena-opponent')));
    assert.ok(plan('SOR_196').cases.some(c=>c.name.endsWith('already-ready-restore')));
});
