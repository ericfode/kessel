'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyArenaDefeatAttack,planArenaDefeatAttackProbes,donorProfileValid}=require('../pipeline/arena-defeat-attack-probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('closed global other-space mandatory attack has both-seat actual construction matrix',()=>{
 const e=entry('ASH_083'),p=planArenaDefeatAttackProbes(e,{catalogue});assert.equal(p.sourceHash,e.sourceHash);assert.equal(p.cases.length,24);
 for(const sourceSeat of [1,2])for(const scenario of ['mixed-global-order','source-only','space-defender-removed','attachments','sentinel-domain','saboteur-bypass','death-trigger-order','terminal-base','own-copy-included','enemy-copy-included','on-defense-capture','on-defense-terminal'])assert.ok(p.cases.some(c=>c.probe.sourceSeat===sourceSeat&&c.probe.scenario===scenario));
 for(const c of p.cases){assert.ok(c.assertionStages.includes('normal-paid-source-play'));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));assert.ok(c.setupCardIds.includes('TOKEN_SHIELD'));assert.ok(c.setupCardIds.includes('TOKEN_EXPERIENCE'));}
 assert.equal(p.donorAdmissionReceipt.donors.TOKEN_SHIELD.implementation,'nativeAdapter');
 for(const d of ['phantom-protected-survivor','state-based-death-capture','full-source-and-helper-review','capacity-and-dynamic-membership','foreign-control'])assert.ok(p.uncovered.some(x=>x.id.endsWith(d)));
});
test('recognition depends on complete semantic source shape not identity',()=>{
 const e=entry('ASH_083');e.setCode='CUSTOM_777';e.className='OtherCreature';e.card.title='Other';e.internalName='other';assert.ok(classifyArenaDefeatAttack(e));
});
test('reject partial source or incorrect metadata, cost, trigger, intrinsic or target predicate',()=>{
 const mutations=[e=>e.sourceSupport={},e=>e.abilities.card.push({}),e=>e.abilities.unit.push({}),e=>e.abilities.other=[],e=>e.card.types.push('token'),e=>e.baseClass='LeaderUnitCard',e=>e.card.cost=11,e=>e.card.hp=14,e=>e.card.power=16,e=>e.card.unique=true,e=>e.card.arena='ground',e=>e.card.keywords.push('overwhelm'),e=>e.keywordValues.sentinel=2,e=>e.card.traits.push('vehicle'),e=>e.piloting={},e=>e.card.pilotText='Pilot',e=>e.abilities.card[0].trigger='whenPlayed',e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].costs.push({kind:'exhaustSelf'}),e=>e.abilities.card[0].then={},e=>e.abilities.card[0].effect.extra=true,e=>e.abilities.card[0].effect.target.receiver.path=['player'],e=>e.abilities.card[0].effect.target.predicate.right={kind:'const',value:true},e=>e.abilities.card[0].effect.target.predicate.right.left.value={kind:'const',value:false},e=>e.abilities.card[0].effect.target.predicate.right.right.right.path=['card'],e=>e.abilities.card[0].effect.target.predicate.left.right.right.right.right.value='groundArena'];
 for(const mutate of mutations){const e=entry('ASH_083');mutate(e);assert.equal(classifyArenaDefeatAttack(e),null);assert.equal(planArenaDefeatAttackProbes(e,{catalogue}),null);}
 for(const id of ['ASH_065','TWI_185','LAW_144'])assert.equal(classifyArenaDefeatAttack(entry(id)),null);
});
test('full donor profiles and independent native admission fail closed on raw profile mutants',()=>{
 const base=planArenaDefeatAttackProbes(entry('ASH_083'),{catalogue});
 for(const code of Object.keys(base.donorAdmissionReceipt.donors)){
 const e=entry(code);assert.ok(donorProfileValid(e,code));e.card.keywords.push('hidden');assert.equal(donorProfileValid(e,code),false);
 const cards=catalogue.cards.map(x=>x.setCode===code?e:x),p=planArenaDefeatAttackProbes(entry('ASH_083'),{catalogue:cards});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(x=>x.id.endsWith('setup-donor-admission')));
 }
 const p=planArenaDefeatAttackProbes(entry('ASH_083'),{catalogue:[]});assert.equal(p.cases.length,0);
});
