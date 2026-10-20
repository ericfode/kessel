'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyExecutorUpgradeCount,planExecutorUpgradeCountProbes}=require('../pipeline/executor-upgrade-count-probes');
// Committed data only: no compiler/parser, simulator or engine import.
const catalogue=require('../../compiler/load-catalog').loadCatalog().cards;
const entry=catalogue.find(e=>e.setCode==='ASH_197');
const clone=structuredClone;
const plan=()=>planExecutorUpgradeCountProbes(entry,{catalogue});
test('complete source profile keeps both registrations and unique ordinary metadata',()=>{
 assert.deepEqual(classifyExecutorUpgradeCount(entry),{family:'executor-upgrade-count',arena:'space',unique:true});
 for(const mutate of [e=>e.baseClass='LeaderUnitCard',e=>e.card.types=['unit','leader'],e=>e.card.unique=false,e=>e.card.cost=7,e=>e.card.hp=11,e=>e.card.power=6,e=>e.card.arena='ground',e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.attachCondition={},e=>e.card.keywords=['support'],e=>e.keywordValues.support=1,e=>e.abilities.card.pop(),e=>e.abilities.card.push(clone(e.abilities.card[1])),e=>e.abilities.unit=[{}],e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='other.ts',e=>e.unknown=true]){const e=clone(entry);mutate(e);assert.equal(classifyExecutorUpgradeCount(e),null);}
});
test('dynamic physical count and paired creation bindings reject partial or broadened AST',()=>{
 for(const mutate of [e=>e.abilities.card[0].effect.properties.targetController='opponent',e=>e.abilities.card[0].effect.modifier.value.callback.parameters[0].argument='abilityContext',e=>e.abilities.card[0].effect.modifier.value.callback.body.statements[0].bindings[0].value.receiver.method='map',e=>e.abilities.card[0].effect.modifier.value.callback.body.statements[1].value.fields.hp.value=1,e=>e.abilities.card[1].effect.amount.value=2,e=>e.abilities.card[1].effect.tokenOwner='creator',e=>e.abilities.card[1].effect.batch='perTarget',e=>e.abilities.card[1].effect.target.receiver.path=['opponent'],e=>e.abilities.card[1].effect.target.predicate={kind:'const',value:true}]){const e=clone(entry);mutate(e);assert.equal(planExecutorUpgradeCountProbes(e,{catalogue}),null);}
});
test('eighteen concrete plans retain exact source-derived both-seat expectations',()=>{
 const p=plan();assert.equal(p.cases.length,18);assert.equal(new Set(p.cases.map(c=>c.id)).size,18);
 for(const scenario of ['empty-normal-play','both-arenas-normal-play','mixed-inventory-normal-play','shield-consumption-live-recompute','advantage-attack-expiry','ordinary-upgrade-self-exclusion','host-defeat-attachment-cleanup','unique-keep-old','unique-keep-incoming'])assert.deepEqual(p.cases.filter(c=>c.probe.scenario===scenario).map(c=>c.probe.owner),[0,1]);
 for(const c of p.cases){assert.equal(c.probe.family,'executor-upgrade-count');assert.equal(c.probe.expectedFocusPlayCost,12);assert.equal(c.probe.resourceCount,30);assert.equal(c.probe.leaderCode,'SOR_014');assert.equal(c.probe.baseCode,'SOR_023');assert.equal(c.probe.baseDamage,0);assert.equal(c.probe.design.plan[0].code,entry.setCode);assert.equal(new Set(c.setupCardIds).size,c.setupCardIds.length);assert(!JSON.stringify(c.probe).includes('FOREIGN'));}
 const get=name=>p.cases.find(c=>c.probe.scenario===name).probe.design;
 assert.equal(get('mixed-inventory-normal-play').expected.sourcePower,11);assert.equal(get('mixed-inventory-normal-play').initialAttachments.length,6);
 assert.equal(get('shield-consumption-live-recompute').expected.sourcePower,10);
 assert.equal(get('advantage-attack-expiry').expected.sourcePowerBeforeAttack,8);assert.equal(get('advantage-attack-expiry').expected.marineShieldCount,1);
 assert.equal(get('ordinary-upgrade-self-exclusion').expected.sourceHp,15);
 assert.equal(get('unique-keep-old').expected.sourcePower,10);assert(get('unique-keep-old').expected.capturedDeadIncomingWhenPlayed);
 assert.equal(get('unique-keep-incoming').expected.sourcePower,9);assert.equal(get('unique-keep-incoming').expected.survivorAdvantageCount,0);
});
test('every donor and native definition is a whole-entry dependency; missing or changed never emits partial cases',()=>{
 const p=plan();assert.equal(p.needs.length,10);assert.deepEqual(p.needs.filter(n=>n.native).map(n=>n.cardId).sort(),['TOKEN_ADVANTAGE','TOKEN_EXPERIENCE','TOKEN_SHIELD']);
 for(const need of p.needs){assert.match(need.wholeEntrySemanticHash,/^[a-f0-9]{64}$/);const missing=planExecutorUpgradeCountProbes(entry,{catalogue:catalogue.filter(e=>e.setCode!==need.cardId)});assert.equal(missing.cases.length,0);assert(missing.uncovered.some(x=>x.id.endsWith(need.cardId)));const changed=clone(catalogue);changed.find(e=>e.setCode===need.cardId).extraSemanticField=true;assert.equal(planExecutorUpgradeCountProbes(entry,{catalogue:changed}).cases.length,0);}
 assert.equal(planExecutorUpgradeCountProbes(entry).cases.length,0);
});
test('plans are repeatable independent values and keep genuine uncovered obligations',()=>{
 const a=plan(),b=plan();assert.deepEqual(a,b);a.cases[0].probe.design.expected.sourcePower=999;a.cases[2].probe.design.units[0].code='wrong';assert.deepEqual(plan(),b);
 for(const suffix of ['source-review','capacity-and-coinciding-deaths','controller-transfer','leader-and-native-unit-hosts','replacement-blanking-and-late-entry'])assert(b.uncovered.some(x=>x.id.endsWith(suffix)));
 assert.equal(b.obligations.length,23);assert.equal(b.uncovered.length,5);
});

test('assembled complete catalogue includes diagnostics and selects only full Executor profile',()=>{assert.deepEqual(entry.diagnostics,[]);assert.equal(catalogue.filter(e=>classifyExecutorUpgradeCount(e)).length,1);assert.equal(plan().cases.length,18);const missing=clone(entry);delete missing.diagnostics;assert.equal(classifyExecutorUpgradeCount(missing),null);});
