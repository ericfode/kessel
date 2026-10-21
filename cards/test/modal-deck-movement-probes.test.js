'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyModalDeckMovement,planModalDeckMovementProbes,REVIEWED_HELPER_INPUTS}=require('../pipeline/modal-deck-movement-probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
const plan=(id,c=catalogue,extra={})=>planModalDeckMovementProbes(entry(id),{catalogue:c,...extra});
test('both complete card profiles plan literal physical identity and branch matrices',()=>{
 const top=plan('ASH_045'),discard=plan('ASH_133');assert.equal(top.cases.length,18);assert.equal(discard.cases.length,36);
 for(const owner of [0,1])for(const chosenDeck of ['self','opponent'])for(const branch of ['discard','leave'])assert.ok(top.cases.some(c=>c.probe.owner===owner&&c.probe.chosenDeck===chosenDeck&&c.probe.branch===branch&&c.probe.scenario==='normal'));
 for(const owner of [0,1])for(const trigger of ['when-played','on-attack'])for(const branch of ['bottom-heal','hand'])assert.ok(discard.cases.some(c=>c.probe.owner===owner&&c.probe.trigger===trigger&&c.probe.branch===branch&&c.probe.scenario==='normal'));
 for(const p of [top,discard])for(const c of p.cases){assert.ok(c.probe.ownDeck.every(d=>d.key&&d.code));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));}
 assert.equal(top.cases[0].probe.expectedFocusPlayCost,5);assert.equal(discard.cases[0].probe.expectedFocusPlayCost,8);
 assert.ok(top.cases.some(c=>c.probe.copyCount===2&&c.probe.activationCode==='SOR_043'));
 for(const owner of [0,1])for(const trigger of ['when-played','on-attack']){assert.ok(discard.cases.some(c=>c.probe.owner===owner&&c.probe.trigger===trigger&&c.probe.ownBaseDamage===0));const c=discard.cases.find(c=>c.probe.owner===owner&&c.probe.trigger===trigger&&c.probe.scenario==='duplicate-physical-discard');assert.equal(c.probe.targetKey,'duplicate-selected');assert.equal(c.probe.discard.filter(x=>x.code==='HMW_093').length,2);}
});
test('complete ordered415 helpers and every input row are required; cards alone cannot establish source execution',()=>{
 assert.equal(REVIEWED_HELPER_INPUTS.length,415);
 const good=plan('ASH_045');assert.equal(good.sourceAdmissionReceipt.helperInputs.length,415);assert.equal(good.sourceAdmissionReceipt.physicalSourcesVerified,false);
 assert.equal(planModalDeckMovementProbes(entry('ASH_045'),{catalogue:catalogue.cards}).cases.length,0);
 for(const mutate of [c=>c.inputs.reverse(),c=>c.inputs.push({...c.inputs[0]}),c=>c.inputs[0].extra=true,c=>c.inputs[0].sha256='x',c=>c.inputs.find(r=>r.file===REVIEWED_HELPER_INPUTS[0].file).sha256='0'.repeat(64),c=>c.inputs=c.inputs.filter(r=>r.file!==REVIEWED_HELPER_INPUTS[0].file),c=>{const i=c.inputs.findIndex(r=>r.file===REVIEWED_HELPER_INPUTS[0].file);[c.inputs[i],c.inputs[i+1]]=[c.inputs[i+1],c.inputs[i]];}]){const c=structuredClone(catalogue);mutate(c);const p=plan('ASH_045',c);assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(x=>x.id.endsWith('helper-source-envelope')));}
});
test('complete source and private method or selected modal target mutations fail closed',()=>{
 for(const id of ['ASH_045','ASH_133'])for(const mutate of [e=>e.sourceSupport={},e=>e.card.keywords.push('support'),e=>e.keywordValues.support=1,e=>e.card.cost++,e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.abilities.card[0].optional=true,e=>e.abilities.card.push({}),e=>e.abilities.card[0].targetResolver.extra=true]){const e=entry(id);mutate(e);assert.equal(classifyModalDeckMovement(e),null);}
 const e=entry('ASH_045');e.privatePrograms[0].defaultTargetParameter.captureSelectedTargetEarly=true;assert.equal(classifyModalDeckMovement(e),null);
 const d=entry('ASH_133');d.abilities.card[1].effect.choices.reverse();assert.equal(classifyModalDeckMovement(d),null);
});
test('source duplicates and donor admission cannot be laundered by matching one source row',()=>{
 for(const id of ['ASH_045','ASH_133']){const dup=structuredClone(catalogue);dup.cards.push(entry(id));assert.equal(plan(id,dup).cases.length,0);}
 const c=structuredClone(catalogue);c.cards.find(e=>e.setCode==='HMW_093').card.cost=8;assert.equal(plan('ASH_133',c).cases.length,0);
 const report=require('../../compiler/generated/runtime-admission.json');const removed={...report,runtimeAdmittedCards:report.runtimeAdmittedCards.filter(e=>e.setCode!=='SOR_043')};assert.equal(plan('ASH_045',catalogue,{runtimeReport:removed}).cases.length,0);
});
test('physical helper verification is opt-in and fails rather than asserting unavailable source evidence',()=>{
 const p=plan('ASH_045',catalogue,{engineRoot:'/nonexistent/modal-deck-engine'});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('physical-engine-source')));
 for(const id of ['ASH_045','ASH_133']){const p=plan(id);for(const name of ['cancel-and-unavailable-protocol','capture-versus-live-mutation','foreign-control-and-replacements','phase-and-terminal-interactions'])assert.ok(p.uncovered.some(o=>o.id.endsWith(name)));}
});

test('raw17-card construction dependencies and on-attack discard creation are actual paid paths',()=>{
 const p=plan('ASH_133');for(const c of p.cases){assert.equal(c.probe.fillerCodes.length,17);assert.equal(new Set(c.probe.fillerCodes).size,17);for(const code of c.probe.fillerCodes)assert.ok(c.setupCardIds.includes(code));if(c.probe.trigger==='on-attack'){assert.equal(c.probe.discardSetupTiming,'after-paid-defeat');assert.equal(c.probe.setupWounds,'hp-minus-four');assert.equal(c.probe.activationCode,'TWI_174');if(c.probe.scenario==='duplicate-physical-discard')assert.equal(c.probe.activationCopies,2);if(c.probe.scenario==='no-eligible')assert.equal(c.probe.activationCopies,0);}else assert.equal(c.probe.discardSetupTiming,'before-play');}
});

test('both decks empty creates exactly two additive no-choice death cases',()=>{const rows=plan('ASH_045').cases.filter(c=>c.probe.scenario==='both-decks-empty');assert.equal(rows.length,2);assert.deepEqual(rows.map(c=>c.probe.owner),[0,1]);for(const c of rows){assert.deepEqual(c.probe.ownDeck,[]);assert.deepEqual(c.probe.enemyDeck,[]);assert.equal(c.probe.trigger,'when-defeated');assert.equal(c.probe.expectedFocusPlayCost,5);assert.equal(c.probe.activationCode,'TWI_174');assert.equal(c.probe.chosenDeck,'self');assert.equal(c.probe.branch,'discard');assert.deepEqual(c.coverageObligationIds,['ASH_045:modal-deck-movement:owner-'+c.probe.owner+'-both-decks-empty']);}});
