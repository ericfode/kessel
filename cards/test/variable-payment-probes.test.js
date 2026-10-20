'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const {classifyVariablePayment,planVariablePaymentProbes} = require('../pipeline/variable-payment-probes');
const catalogue = require('../../compiler/load-catalog').loadCatalog();
const entry = id => structuredClone(catalogue.cards.find(e => e.setCode === id));
test('actual reviewed Flock source constructs bounded number/payment boundaries on both seats',()=>{
    const e=entry('LOF_255'), plan=planVariablePaymentProbes(e,{catalogue});
    assert.ok(classifyVariablePayment(e));assert.equal(plan.sourceHash,e.sourceHash);assert.equal(plan.cases.length,16);
    assert.equal(new Set(plan.cases.map(c=>c.id)).size,16);
    for(const owner of [0,1]) {
        const cases=plan.cases.filter(c=>c.probe.owner===owner);
        for(const n of [0,1,3,6]) assert.ok(cases.some(c=>c.probe.scenario==='normal'&&c.probe.paymentAmount===n&&c.probe.readyAfterPlay===9));
        for(const [n,ready] of [[0,0],[2,2],[6,6]]) assert.ok(cases.some(c=>c.probe.scenario==='normal'&&c.probe.paymentAmount===n&&c.probe.readyAfterPlay===ready));
        const dead=cases.find(c=>c.probe.scenario==='captured-source-dead');assert.equal(dead.probe.paymentAmount,3);assert.equal(dead.probe.snokeCode,'SHD_037');
    }
    for(const c of plan.cases){assert.equal(c.probe.family,'variable-payment');assert.equal(c.probe.cardId,'LOF_255');assert.equal(c.probe.sourceArena,'ground');assert.equal(c.probe.resourceCount,12);assert.ok(c.probe.readyAfterPlay+e.card.cost<=c.probe.resourceCount);assert.ok(c.probe.paymentAmount<=Math.min(6,c.probe.readyAfterPlay));assert.ok(c.setupCardIds.includes('TOKEN_EXPERIENCE'));assert.deepEqual(c.focusCardIds,['LOF_255']);assert.ok(c.coverageObligationIds.every(id=>plan.obligations.some(o=>o.id===id)));}
    assert.ok(plan.uncovered.some(o=>o.id.endsWith('source-review')));assert.ok(plan.uncovered.some(o=>o.id.endsWith('credit-and-cost-reducer-interaction')));
});
test('similar number-payment sources remain outside the complete Flock construction contract',()=>{
    for(const id of ['HMW_036','SEC_040']) {assert.equal(classifyVariablePayment(entry(id)),null);assert.equal(planVariablePaymentProbes(entry(id),{catalogue}),null);}
});
test('source, metadata, intrinsic and complete IR changes fail closed independently',()=>{
    const mutations=[
        e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='game/cards/other.ts',e=>e.status='unsupported',e=>e.baseClass='LeaderUnitCard',
        e=>e.card.types=['token','unit'],e=>e.card.hp=2,e=>e.card.cost=0,e=>e.card.aspects=['vigilance'],e=>e.card.unique=true,
        e=>e.card.keywords=['restore'],e=>e.keywordValues={restore:2},e=>e.piloting={},e=>e.attachCondition={},e=>e.card.pilotText='pilot',
        e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'onAttack'}),e=>e.abilities.card[0].optional=true,
        e=>e.abilities.card[0].then={effect:{kind:'noAction'}},e=>e.abilities.card[0].targetResolver.max.args[0].value=7,
        e=>e.abilities.card[0].targetResolver.sameEndpoints='prompt',e=>e.abilities.card[0].targetResolver.chooser.value='opponent',
        e=>e.abilities.card[0].effect.effects.reverse(),e=>e.abilities.card[0].effect.effects[0].allowCostAdjustments=true,
        e=>e.abilities.card[0].effect.effects[0].isCost=false,e=>e.abilities.card[0].effect.effects[1].target='chosen',
        e=>e.abilities.card[0].effect.effects[1].amount.args[0].path=['other'],e=>e.unreviewed=true,
    ];
    for(const mutate of mutations){const e=entry('LOF_255');mutate(e);assert.equal(classifyVariablePayment(e),null);assert.equal(planVariablePaymentProbes(e,{catalogue}),null);}
});
test('object property order is irrelevant, while arrays and types retain their exact contract',()=>{
    const reorder=v=>Array.isArray(v)?v.map(reorder):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).reverse().map(([k,v])=>[k,reorder(v)])):v;
    assert.deepEqual(planVariablePaymentProbes(reorder(entry('LOF_255')),{catalogue}),planVariablePaymentProbes(entry('LOF_255'),{catalogue}));
});
test('changed or missing native Experience refuses construction instead of claiming token outcomes',()=>{
    for(const mutation of [e=>e.card.hp=9,e=>e.sourceHash='0'.repeat(64),e=>e.card.types=['unit']]){const defs=structuredClone(catalogue.cards);mutation(defs.find(e=>e.setCode==='TOKEN_EXPERIENCE'));const p=planVariablePaymentProbes(entry('LOF_255'),{catalogue:defs});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(x=>x.id.endsWith('native-experience-definition')));}
    const p=planVariablePaymentProbes(entry('LOF_255'),{catalogue:[]});assert.equal(p.cases.length,0);
});
test('changed or absent Snoke keeps normal cases and marks each death boundary uncovered',()=>{
    for(const defs of [catalogue.cards.filter(e=>e.setCode!=='SHD_037'),catalogue.cards.map(e=>e.setCode==='SHD_037'?{...e,sourceHash:'0'.repeat(64)}:e)]){const p=planVariablePaymentProbes(entry('LOF_255'),{catalogue:defs});assert.equal(p.cases.length,14);assert.ok(p.cases.every(c=>c.probe.scenario==='normal'));for(const owner of [0,1])assert.ok(p.uncovered.some(x=>x.id.endsWith('owner-'+owner+'-captured-source-dead')));}
});
