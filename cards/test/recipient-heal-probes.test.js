'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyRecipientHeal,donorValid,planRecipientHealProbes}=require('../pipeline/recipient-heal-probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('complete source profile plans both seats including unique departure and Sentinel domains',()=>{
 const p=planRecipientHealProbes(entry('ASH_065'),{catalogue});assert.equal(p.cases.length,16);assert.equal(p.sourceHash,entry('ASH_065').sourceHash);
 for(const owner of [0,1])for(const scenario of ['mixed','all-undamaged','source-only','keep-old','keep-new','sentinel','saboteur','attachments'])assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario===scenario));
 assert.match(p.donorAdmissionReceipt.reportHash,/^[a-f0-9]{64}$/);
 assert.equal(p.donorAdmissionReceipt.donors.shield.implementation,'nativeAdapter');
 for(const c of p.cases)assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));
 for(const d of ['capture-before-mutation','one-state-sweep','large-capacity','heal-listeners-and-replacements','source-and-helper-review','foreign-control'])assert.ok(p.uncovered.some(x=>x.id.endsWith(d)));
});
test('source shape is structural with alpha-consistent healed-card binding',()=>{
 const e=entry('ASH_065');e.setCode='CUSTOM_001';e.className='OtherShip';e.card.title='Other';let cb=e.abilities.card[0].effect.amount.callback;cb.parameters[0].root='attackBinding9_2';cb.body.path[0]='attackBinding9_2';assert.ok(classifyRecipientHeal(e));
});
test('reject unknown source fields, companion abilities, callback timing, scope and binding mutants',()=>{
 for(const mutate of [e=>e.sourceSupport={},e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.other=[],e=>e.card.types.push('token'),e=>e.card.unique=false,e=>e.card.arena='ground',e=>e.card.keywords.push('shielded'),e=>e.keywordValues.sentinel=2,e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].then={},e=>e.abilities.card[0].effect.extra=true,e=>e.abilities.card[0].effect.target.receiver.path=['game'],e=>e.abilities.card[0].effect.amount.extra=true,e=>e.abilities.card[0].effect.amount.evaluateAt='handler',e=>e.abilities.card[0].effect.amount.legalityComparison='value',e=>e.abilities.card[0].effect.amount.callback.parameters[0].role='attacker',e=>e.abilities.card[0].effect.amount.callback.body.path[1]='hp',e=>e.abilities.card[0].effect.amount.callback.body.path[0]='source',e=>e.abilities.card[0].effect.amount.callback.extra=true]){const e=entry('ASH_065');mutate(e);assert.equal(classifyRecipientHeal(e),null);}
});
test('donor full keywords, token native metadata and deployed leader source are guarded',()=>{
 for(const [id,role]of [['HMW_093','ground'],['ASH_061','ground2'],['JTL_069','space'],['ASH_141','saboteur'],['SEC_005','leader'],['TOKEN_BEAST','token'],['TOKEN_SHIELD','shield'],['TOKEN_EXPERIENCE','experience']]){assert.ok(donorValid(entry(id),role),id);const bad=entry(id);bad.card.keywords.push('hidden');assert.equal(donorValid(bad,role),false);}
 const bad=entry('SEC_005');bad.abilities.leader[0].effect.amount.value=3;assert.equal(donorValid(bad,'leader'),false);
 const fake=entry('HMW_093');fake.setCode='UNADMITTED_999';const p=planRecipientHealProbes(entry('ASH_065'),{catalogue:{cards:[...catalogue.cards,fake]},recipientHealDonors:{ground:fake.setCode}});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('donor-admission')));
});
