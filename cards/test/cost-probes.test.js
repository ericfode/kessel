'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {planCostProbes,classifyCostShape}=require('../pipeline/cost-probes');
const catalogue=JSON.parse(fs.readFileSync(process.env.SWU_COST_PROBE_CATALOGUE??path.resolve(__dirname,'../../compiler/generated/catalog.json'),'utf8'));
const ids=['HMW_154','HMW_249','SEC_154','TS26_079'];
const plan=id=>planCostProbes(catalogue.cards.find(c=>c.setCode===id),{catalogue});
test('generic cost planners preserve card identity independence and finite uncovered boundaries',()=>{
    for(const id of ids){assert.ok(classifyCostShape(catalogue.cards.find(c=>c.setCode===id)),'Expected committed cost-query family '+id);const p=plan(id);assert.ok(p.cases.length>=7);assert.ok(p.cases.every(c=>c.probe.family==='cost-query'));const e=structuredClone(catalogue.cards.find(c=>c.setCode===id));e.setCode='SYN_999';assert.equal(planCostProbes(e,{catalogue}).cases.length,p.cases.length);assert.ok(p.uncovered.some(o=>o.id.endsWith('source-semantics')));}
});
test('cost shape classifiers reject unknown ability resolver and nested expression properties',()=>{
    for(const id of ids){const e=structuredClone(catalogue.cards.find(c=>c.setCode===id));const a=Object.values(e.abilities).flat()[0];if(a)a.unimplemented=true;else e.attachCondition.left.hidden=true;assert.equal(classifyCostShape(e),null);}
    const e=structuredClone(catalogue.cards.find(c=>c.setCode==='SEC_154'));e.abilities.card[0].targetResolver.chooser='opponent';assert.equal(classifyCostShape(e),null);
    const f=structuredClone(catalogue.cards.find(c=>c.setCode==='HMW_249'));f.abilities.card[0].targetResolver.predicate.left.receiver.path=['player'];assert.equal(classifyCostShape(f),null);
});
