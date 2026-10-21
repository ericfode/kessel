'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyAttackBaseHistory,planAttackBaseHistoryProbes,REVIEWED_HELPER_INPUTS,FILLER_CODES,semanticHash,BOUNDARY_RAW_CASES,BOUNDARY_RAW_CASES_SOURCE_SHA256,DEFENDER_DEPARTURE_RAW_CASES,DEFENDER_DEPARTURE_RAW_SOURCE_SHA256}=require('../pipeline/attack-base-history-probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
const plan=(id,c=catalogue,extra={})=>planAttackBaseHistoryProbes(entry(id),{catalogue:c,...extra});
test('three whole-card profiles retain declaration vs same-attack damage distinctions and both seats',()=>{
 for(const [id,count,kind]of [['ASH_119',36,'base-declaration'],['ASH_031',26,'self-combat-base-heal'],['ASH_144',30,'friendly-combat-base-advantage']]){const p=plan(id);assert.equal(p.cases.length,count);assert.equal(p.sourceHash,entry(id).sourceHash);for(const owner of [0,1])assert.equal(p.cases.filter(c=>c.probe.owner===owner).length,count/2);for(const c of p.cases){assert.equal(c.probe.kind,kind);assert.ok(c.probe.focusEntry.startsWith('normal-paid-play'));assert.equal(c.probe.fillerCodes.length,17);for(const code of FILLER_CODES)assert.ok(c.setupCardIds.includes(code));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));}for(const choice of ['keep-old','keep-new'])assert.equal(p.cases.filter(c=>c.probe.uniqueChoice===choice).length,2);}
 const g=plan('ASH_119');assert.ok(g.cases.some(c=>c.probe.scenario==='opponent-zero-power-base'&&c.probe.expectedTokenDelta===1&&c.probe.expectedCombatBaseDamage===0));assert.ok(g.cases.some(c=>c.probe.scenario==='overwhelm-without-base-declaration'&&c.probe.expectedTokenDelta===0));
 const h=plan('ASH_031');assert.ok(h.cases.some(c=>c.probe.scenario==='overwhelm-dies-public'&&c.probe.expectedFocusZone==='discard'&&c.probe.expectedBaseHeal===2));assert.ok(h.cases.some(c=>c.probe.scenario==='same-source-earlier-attack'&&c.probe.expectedSecondBaseHeal===0));
 const v=plan('ASH_144');assert.ok(v.cases.some(c=>c.probe.scenario==='overwhelm-attacker-dies-public'&&c.probe.expectedAdvantageDelta===1));assert.ok(v.cases.some(c=>c.probe.scenario==='opponent-base'&&c.probe.expectedAdvantageDelta===0));assert.ok(v.cases.some(c=>c.probe.scenario==='same-source-earlier-attack'&&c.probe.expectedSecondAdvantageDelta===0));
});
test('415 ordered helpers, full input-row shapes and intrinsic/native lifecycle envelope fail closed',()=>{
 assert.equal(REVIEWED_HELPER_INPUTS.length,415);assert.equal(planAttackBaseHistoryProbes(entry('ASH_119'),{catalogue:catalogue.cards}).cases.length,0);
 for(const mutate of [c=>c.inputs.reverse(),c=>c.inputs.push({...c.inputs[0]}),c=>c.inputs[0].extra=true,c=>c.inputs[0].sha256='wrong',c=>c.inputs.find(r=>r.file===REVIEWED_HELPER_INPUTS[0].file).sha256='0'.repeat(64),c=>delete c.sourceSupport,c=>c.sourceSupport.pop(),c=>c.sourceSupport[0].extra=true]){const c=structuredClone(catalogue);mutate(c);assert.equal(plan('ASH_119',c).cases.length,0);}
});
test('raw watcher, capture, title, costs and complete intrinsic metadata mutants are rejected',()=>{
 for(const id of ['ASH_119','ASH_031','ASH_144'])for(const mutate of [e=>e.sourceSupport={},e=>e.card.keywords.push('support'),e=>e.keywordValues.support=1,e=>e.card.cost++,e=>e.card.unique=false,e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.abilities.card.push({}),e=>e.stateWatchers[0].resetEvent='onRoundEnded',e=>e.stateWatchers[0].sharedScope='player',e=>e.abilities.card[0].optional=true]){const e=entry(id);mutate(e);assert.equal(classifyAttackBaseHistory(e),null);}
 const g=entry('ASH_119');g.abilities.card[0].effect.condition.args[0].fields.controller.path=['player'];assert.equal(classifyAttackBaseHistory(g),null);
 const h=entry('ASH_031');h.abilities.card[0].effect.onTrue.amount.value.operation='getDamageDealtToBaseByUnitThisPhase';assert.equal(classifyAttackBaseHistory(h),null);
 const v=entry('ASH_144');v.abilities.card[0].hooks[0].predicate.left.right.path=['opponent'];assert.equal(classifyAttackBaseHistory(v),null);
 assert.equal(classifyAttackBaseHistory(entry('ASH_162')),null);
});
test('all donor profiles and native token lifecycles require independent admission',()=>{
 for(const code of ['TOKEN_MANDALORIAN','TOKEN_SHIELD','TOKEN_ADVANTAGE','TOKEN_EXPERIENCE','ASH_181']){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===code).card.keywords.push('hidden');assert.equal(plan('ASH_119',c).cases.length,0);}
 const c=structuredClone(catalogue);c.cards.push(entry('ASH_119'));assert.equal(plan('ASH_119',c).cases.length,0);
 const report=require('../../compiler/generated/runtime-admission.json');assert.equal(plan('ASH_119',catalogue,{runtimeReport:{...report,runtimeAdmittedCards:report.runtimeAdmittedCards.filter(e=>e.setCode!=='TOKEN_SHIELD')}}).cases.length,0);
});
test('finite gaps explicitly retain hidden incarnation, foreign controller and protocol uncertainty',()=>{
 for(const id of ['ASH_119','ASH_031','ASH_144']){const p=plan(id);for(const d of ['hidden-zone-and-new-incarnation','noncombat-event-and-indirect-history','foreign-controller-and-multitarget','cancel-and-conditional-action-protocol','replacement-and-terminal-windows'])assert.ok(p.uncovered.some(o=>o.id.endsWith(d)));}
 const p=plan('ASH_031',catalogue,{engineRoot:'/nonexistent/history-engine'});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(o=>o.id.endsWith('physical-engine-source')));
});

test('literal deck definitions exclude native tokens leaders bases while pool dependencies retain all native lifecycle roles',()=>{
 const p=plan('ASH_119');for(const c of p.cases){assert.equal(c.probe.fillerCodes.length,17);for(const code of c.probe.fillerCodes){const e=catalogue.cards.find(e=>e.setCode===code);assert.ok(!e.card.types.some(t=>['token','leader','base'].includes(t)));}for(const code of ['TOKEN_MANDALORIAN','TOKEN_SHIELD','TOKEN_ADVANTAGE','TOKEN_EXPERIENCE','SOR_014','SOR_023'])assert.ok(c.setupCardIds.includes(code));}
});

test('append preserves every sealed68 full object and relative order while binding exact measured16',()=>{
 const baseline={ASH_119:[24,'9eeea5c845020efc21cdd836e309a0a01ec5712771d2556aa1b29741eefaf1c2'],ASH_031:[20,'00f47d042132864c86d576e7f63e2a69d39aadec67a6916d930f213ebbf6e3a8'],ASH_144:[24,'d13a0de1944f658e29a99d390a1f1faa0b5b094adeaf08bd6a0d6a59192dda42']};
 assert.equal(BOUNDARY_RAW_CASES_SOURCE_SHA256,'adaeb80d55a06b472733760f06f79b574adc254a0b019da831e9310eb112c4a6');
 assert.equal(semanticHash(BOUNDARY_RAW_CASES),'23e48a486eafcf68d0353a643f6002bd8a8a282b281010d10d5acfdf3861a360');
 for(const [id,[oldCount,oldHash]]of Object.entries(baseline)){const p=plan(id);assert.equal(semanticHash(p.cases.slice(0,oldCount)),oldHash);assert.deepEqual(p.cases.slice(oldCount,oldCount+BOUNDARY_RAW_CASES.filter(c=>c.probe.cardId===id).length),BOUNDARY_RAW_CASES.filter(c=>c.probe.cardId===id));for(const c of p.cases.slice(oldCount))for(const id of c.coverageObligationIds)assert.ok(p.obligations.some(o=>o.id===id));}
});
test('new exact cases distinguish preentry history Credit commitments capped damage and dead-target capture',()=>{
 for(const owner of [0,1]){const rows=BOUNDARY_RAW_CASES.filter(c=>c.probe.owner===owner);assert.equal(rows.length,8);const g=rows.filter(c=>c.probe.cardId==='ASH_119');assert.equal(g.length,6);assert.ok(g.some(c=>c.probe.scenario==='preentry-declaration'&&c.probe.expectedTokenDelta===1));assert.equal(g.filter(c=>c.probe.creditCount===1).length,5);assert.ok(!g.some(c=>c.probe.scenario==='credit-zero-skip'));for(const c of g.filter(c=>c.probe.creditChoice==='cancel'))assert.equal(c.probe.expectedTokenDelta,0);const v=rows.find(c=>c.probe.cardId==='ASH_144');assert.equal(v.probe.expectedHistoryAmount,1);assert.equal(v.probe.expectedAdvantageDelta,0);assert.equal(v.probe.expectedFocusZone,'discard');const h=rows.find(c=>c.probe.cardId==='ASH_031');assert.equal(h.probe.enemyBaseDamage,29);assert.equal(h.probe.expectedActualDamage,1);assert.equal(h.probe.expectedHistoryAmount,1);assert.equal(h.probe.expectedBaseHeal,1);assert.equal(h.probe.expectedWinner,'self');assert.ok(h.probe.fillerCodes.includes('HMW_128'));assert.ok(!h.probe.fillerCodes.includes('HMW_178'));}
});
test('new full donor profiles and helper sha array coercion remain blocking',()=>{
 for(const code of ['HMW_178','TOKEN_CREDIT']){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===code).card.keywords.push('hidden');assert.equal(plan('ASH_119',c).cases.length,0);}
 const c=structuredClone(catalogue);c.inputs[0].sha256=[c.inputs[0].sha256];assert.equal(plan('ASH_119',c).cases.length,0);const report=require('../../compiler/generated/runtime-admission.json');assert.equal(plan('ASH_119',catalogue,{runtimeReport:{...report,runtimeAdmittedCards:report.runtimeAdmittedCards.filter(e=>e.setCode!=='TOKEN_CREDIT')}}).cases.length,0);assert.equal(Object.keys(plan('ASH_119').sourceAdmissionReceipt.profiles).length,25);
});

test('defender-departure regressions bind exact measured V2 and genuine formal obligation links',()=>{
 assert.equal(DEFENDER_DEPARTURE_RAW_SOURCE_SHA256,'7eef2477f954cf7fd7574eea44c42f9dd1eae572009c07efdffb08e07b035bf2');
 assert.equal(semanticHash(DEFENDER_DEPARTURE_RAW_CASES),'3cda4fdca91e2daca9c9462ade28eeebe8be6c3049544a724c4539cf277aff11');
 for(const id of ['ASH_031','ASH_144']){const p=plan(id),raw=DEFENDER_DEPARTURE_RAW_CASES.filter(c=>c.probe.cardId===id);assert.equal(raw.length,4);assert.deepEqual(p.cases.slice(-4),raw);for(const owner of [0,1])assert.equal(raw.filter(c=>c.probe.owner===owner).length,2);for(const c of raw){assert.equal(c.probe.attackPrelude,'paid-focus-and-attacker-then-actual-combat-wound-before-upgrade');for(const oid of c.coverageObligationIds)assert.ok(p.obligations.some(o=>o.id===oid&&o.path==='attack-base-history'&&o.dimension===c.probe.scenario));for(const code of ['TWI_198','SEC_187','ASH_035'])assert.ok(c.setupCardIds.includes(code));for(const code of c.probe.fillerCodes){const e=catalogue.cards.find(e=>e.setCode===code);assert.ok(!e.card.types.some(t=>['token','leader','base'].includes(t)));}}}
 for(const owner of [0,1]){const rows=DEFENDER_DEPARTURE_RAW_CASES.filter(c=>c.probe.owner===owner);const hera=rows.filter(c=>c.probe.cardId==='ASH_031');assert.ok(hera.some(c=>c.probe.expectedHistoryAmount===5&&c.probe.expectedBaseHeal===5));assert.ok(hera.some(c=>c.probe.expectedHistoryAmount===1&&c.probe.expectedBaseHeal===1));const vane=rows.filter(c=>c.probe.cardId==='ASH_144');assert.ok(vane.some(c=>c.probe.expectedHistoryAmount===7&&c.probe.expectedAdvantageDelta===1));assert.ok(vane.some(c=>c.probe.expectedHistoryAmount===10&&c.probe.expectedAdvantageDelta===1));}
 assert.equal(plan('ASH_119').cases.length,36);assert.ok(!plan('ASH_119').cases.some(c=>DEFENDER_DEPARTURE_RAW_CASES.some(r=>r.id===c.id)));
});
test('new whole donor profiles are blocking only for Hera/Vane dependency scope',()=>{
 const unchanged=plan('ASH_119');for(const code of ['TWI_198','SEC_187','ASH_035']){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===code).card.keywords.push('support');assert.equal(plan('ASH_031',c).cases.length,0);assert.equal(plan('ASH_144',c).cases.length,0);assert.deepEqual(plan('ASH_119',c),unchanged);}
 assert.equal(Object.keys(plan('ASH_031').sourceAdmissionReceipt.profiles).length,28);assert.equal(Object.keys(plan('ASH_144').sourceAdmissionReceipt.profiles).length,28);assert.equal(Object.keys(unchanged.sourceAdmissionReceipt.profiles).length,25);
 const report=require('../../compiler/generated/runtime-admission.json'),removed={...report,runtimeAdmittedCards:report.runtimeAdmittedCards.filter(e=>e.setCode!=='TWI_198')};assert.equal(plan('ASH_031',catalogue,{runtimeReport:removed}).cases.length,0);assert.equal(plan('ASH_144',catalogue,{runtimeReport:removed}).cases.length,0);
});
