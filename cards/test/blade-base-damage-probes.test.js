'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyBladeBaseDamage,planBladeBaseDamageProbes}=require('../pipeline/blade-base-damage-probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
const fillers=['ASH_061','ASH_131','ASH_190','ASH_201','ASH_242','ASH_261','HMW_082','HMW_093','HMW_116','HMW_128','HMW_135','HMW_155','HMW_163','HMW_174','HMW_178','HMW_219','HMW_258'];
test('sealed twenty-four standard scenarios use explicit paid-play base/defaults and ordered full deck dependencies',()=>{
    const p=planBladeBaseDamageProbes(entry('ASH_204'),{catalogue});assert.equal(p.cases.length,24);assert.equal(new Set(p.cases.map(c=>c.id)).size,24);
    for(const owner of [0,1])for(const scenario of ['paid-play-damage1','damage3','separate-events','enemy-base-negative','friendly-unit-negative','shield-cancel-unit','zero-base-combat','source-gone-before','source-gone-within-cinder','terminal-own-base','advantage-attack-expiry','advantage-phase-persistence'])assert.ok(p.cases.some(c=>c.probe.owner===owner&&c.probe.scenario===scenario));
    for(const c of p.cases){assert.equal(c.probe.family,'blade-base-damage');assert.equal(c.probe.resourceCount,12);assert.equal(c.probe.sourceArena,'space');assert.equal(c.probe.leaderCode,'SOR_014');assert.equal(c.probe.enemyLeaderCode,'SOR_014');assert.equal(c.probe.baseCode,'ASH_025');assert.equal(c.probe.enemyBaseCode,'ASH_025');assert.deepEqual(c.probe.fillerCodes,fillers);assert.ok(['SOR_014','SOR_023','ASH_025','TOKEN_ADVANTAGE',...fillers].every(id=>c.setupCardIds.includes(id)));assert.ok(!c.setupCardIds.includes('ASH_070'));assert.deepEqual(c.focusCardIds,['ASH_204']);assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));assert.ok(c.assertionStages.includes('terminal'));}
    assert.ok(p.cases.find(c=>c.probe.scenario==='source-gone-within-cinder').setupCardIds.includes('ASH_151'));
    for(const gap of ['cancelled-own-base-damage','indirect-and-overwhelm','two-events-one-parent-window','source-controller-transfer','mixed-effective-trigger-owners','advantage-defense-expiry'])assert.ok(p.uncovered.some(o=>o.id.endsWith(gap)));
});
test('whole raw source identity predicate, mandatory self grant and metadata remain closed',()=>{
    const changes=[e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='game/other.ts',e=>e.card.types=['unit','token'],e=>e.baseClass='LeaderUnitCard',e=>e.status='unsupported',e=>e.card.cost=2,e=>e.card.hp=5,e=>e.card.power=4,e=>e.card.arena='ground',e=>e.card.aspects=[],e=>e.keywordValues={restore:2},e=>e.card.keywords=['restore'],e=>e.piloting={},e=>e.attachCondition={},e=>e.card.pilotText='Pilot',e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'whenDefeated'}),e=>e.abilities.card[0].optional=true,e=>e.abilities.card[0].effect.amount.value=3,e=>e.abilities.card[0].effect.target='chosen',e=>e.abilities.card[0].effect.tokenOwner='creator',e=>e.abilities.card[0].hooks[0].event='onDamageTaken',e=>e.abilities.card[0].hooks[0].predicate.right.path=['source','controller','base'],e=>e.abilities.card[0].hooks[0].predicate.left.steps[0].key='amount',e=>e.abilities.card[0].hooks[0].predicate.op='!==',e=>e.abilities.card[0].then={effect:{kind:'noAction'}},e=>e.extra=true];
    for(const change of changes){const e=entry('ASH_204');change(e);assert.equal(classifyBladeBaseDamage(e),null);assert.equal(planBladeBaseDamageProbes(e,{catalogue}),null);}
    for(const id of ['ASH_070','ASH_191','ASH_206'])assert.equal(classifyBladeBaseDamage(entry(id)),null);
});
test('card code is not a behavior exception and object key order does not replace semantics',()=>{
    const e=entry('ASH_204');e.setCode='CUSTOM_999';e.id=e.card.id='another';e.internalName=e.card.internalName='other';e.className='EquivalentSource';e.card.setId={set:'CUSTOM',number:999};e.card.setCodes=[e.card.setId];assert.ok(classifyBladeBaseDamage(e));assert.equal(planBladeBaseDamageProbes(e,{catalogue}).cases.length,24);
    const reorder=v=>Array.isArray(v)?v.map(reorder):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).reverse().map(([k,v])=>[k,reorder(v)])):v;
    assert.deepEqual(planBladeBaseDamageProbes(reorder(entry('ASH_204')),{catalogue:reorder(catalogue.cards)}),planBladeBaseDamageProbes(entry('ASH_204'),{catalogue}));
});
test('missing malformed or ambiguous native/default/filler dependencies cannot produce a partial pool plan',()=>{
    for(const code of ['TOKEN_ADVANTAGE','SOR_014','SOR_023','ASH_025','ASH_061','HMW_258'])for(const defs of [catalogue.cards.filter(e=>e.setCode!==code),catalogue.cards.map(e=>e.setCode===code?{...e,sourceHash:'0'.repeat(64)}:e),[...catalogue.cards,entry(code)]]){const p=planBladeBaseDamageProbes(entry('ASH_204'),{catalogue:defs});assert.equal(p.cases.length,0);assert.equal(p.uncovered.filter(o=>o.reason.includes('missing, changed or ambiguous')).length,24);}
});
test('specific real donor/source alias changes omit only dependent scenarios',()=>{
    for(const [code,remaining]of [['SOR_095',16],['LOF_254',22],['TOKEN_SHIELD',20],['SHD_037',22],['IBH_063',22],['TWI_226',22],['ASH_151',22],['LAW_043',22]]){const defs=catalogue.cards.filter(e=>e.setCode!==code);const p=planBladeBaseDamageProbes(entry('ASH_204'),{catalogue:defs});assert.equal(p.cases.length,remaining,code);}
    const defs=structuredClone(catalogue.cards);defs.find(e=>e.setCode==='SOR_014').card.setCodes.reverse();
    // Alter the actual alias content, not just a singleton array's order.
    defs.find(e=>e.setCode==='SOR_014').card.setCodes.push({set:'BAD',number:14});
    assert.equal(planBladeBaseDamageProbes(entry('ASH_204'),{catalogue:defs}).cases.length,0);
});
test('planner cases do not share mutable filler arrays and committed-data tests load no engine/parser',()=>{
    const p=planBladeBaseDamageProbes(entry('ASH_204'),{catalogue});p.cases[0].probe.fillerCodes.reverse();assert.deepEqual(p.cases[1].probe.fillerCodes,fillers);assert.ok(!Object.keys(require.cache).some(p=>p.includes('/forceteki/')||p.includes('/typescript/')));
});
