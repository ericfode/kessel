'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyDefeatCause,planDefeatCauseProbes}=require('../pipeline/defeat-cause-probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('complete source/helper construction covers both owners and all observed causes/recipient choices',()=>{
    const e=entry('ASH_191'),p=planDefeatCauseProbes(e,{catalogue});assert.ok(p);assert.equal(p.cases.length,36);assert.equal(new Set(p.cases.map(c=>c.id)).size,36);assert.equal(p.sourceHash,e.sourceHash);
    for(const owner of [0,1])for(const cause of ['combat','direct-damage','direct-defeat','attack-ability-damage','stat-death']){
        const rows=p.cases.filter(c=>c.probe.owner===owner&&c.probe.cause===cause);assert.equal(rows.length,3);assert.ok(rows.some(c=>c.probe.recipient==='self'&&!c.probe.decline));assert.ok(rows.some(c=>c.probe.recipient==='opponent'&&!c.probe.decline));assert.ok(rows.some(c=>c.probe.decline));assert.ok(rows.every(c=>c.probe.expectedAmount===(cause==='combat'?2:3)));
    }
    for(const c of p.cases){assert.equal(c.probe.family,'defeat-cause');assert.equal(c.probe.sourceArena,'space');assert.equal(c.probe.resourceCount,24);assert.equal(c.probe.recipientCode,'SOR_095');assert.ok(c.setupCardIds.includes('TOKEN_ADVANTAGE'));assert.deepEqual(c.focusCardIds,['ASH_191']);assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));if(c.probe.expectedAmount>0)assert.ok(c.assertionStages.includes('recipient-attack-end'));assert.ok(c.assertionStages.includes('terminal'));}
    const attack=p.cases.find(c=>c.probe.cause==='attack-ability-damage');assert.equal(attack.probe.attackerCode,'LOF_144');assert.equal(attack.probe.expectedAmount,3);
    for(const gap of ['source-review','missing-or-null-cause','unique-rule-cause','source-controller-change','nested-or-competing-death'])assert.ok(p.uncovered.some(o=>o.id.endsWith(gap)));
});
test('identity does not select behavior; raw source profile and helper AST do',()=>{
    const e=entry('ASH_191');e.setCode='CUSTOM_999';e.id=e.card.id='new-physical-definition';e.internalName=e.card.internalName='another-definition';e.className='SameReviewedBehavior';e.card.setId={set:'CUSTOM',number:999};e.card.setCodes=[e.card.setId];
    assert.ok(classifyDefeatCause(e));const p=planDefeatCauseProbes(e,{catalogue});assert.equal(p.cases.length,36);assert.ok(p.cases.every(c=>c.probe.cardId==='CUSTOM_999'));
});
test('metadata, source, intrinsic, helper binding/title/amount and complete registration changes reject',()=>{
    const mutations=[e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='game/cards/other.ts',e=>e.baseClass='LeaderUnitCard',e=>e.status='unsupported',e=>e.card.hp=2,e=>e.card.types=['unit','token'],e=>e.card.arena='ground',e=>e.card.cost=0,e=>e.keywordValues={restore:1},e=>e.card.keywords=['restore'],e=>e.piloting={},e=>e.attachCondition={},e=>e.card.pilotText='Pilot',e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'whenPlayed'}),e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].effect.target='self',e=>e.abilities.card[0].effect.createdBy='opponent',e=>e.abilities.card[0].effect.amount.name='differentHelper',e=>e.abilities.card[0].contextTitle[1].name='other',e=>e.historyHelpers[0].parameters.push('extra'),e=>e.historyHelpers[0].body.statements[0].value.then.value=3,e=>e.historyHelpers[0].body.statements[0].value.condition.left.steps[1].key='currentAttack',e=>e.extra=true];
    for(const change of mutations){const e=entry('ASH_191');change(e);assert.equal(classifyDefeatCause(e),null);assert.equal(planDefeatCauseProbes(e,{catalogue}),null);}
    for(const id of ['LOF_255','ASH_206','ASH_228'])assert.equal(classifyDefeatCause(entry(id)),null);
});
test('complete native Advantage dependency is required; unsupported missing metadata gives no construction credit',()=>{
    for(const change of [e=>e.sourceHash='0'.repeat(64),e=>e.card.types=['upgrade'],e=>e.card.upgradePower=9,e=>e.card.text='Other token lifecycle']){const defs=structuredClone(catalogue.cards);change(defs.find(e=>e.setCode==='TOKEN_ADVANTAGE'));const p=planDefeatCauseProbes(entry('ASH_191'),{catalogue:defs});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(x=>x.id.endsWith('native-advantage-definition')));}
    assert.equal(planDefeatCauseProbes(entry('ASH_191'),{catalogue:[]}).cases.length,0);
});
test('missing/changed/ambiguous cause donors omit affected cases without silently changing setup',()=>{
    for(const id of ['LAW_253','TWI_174','TWI_077','LOF_144','SHD_037']){
        for(const defs of [catalogue.cards.filter(e=>e.setCode!==id),catalogue.cards.map(e=>e.setCode===id?{...e,sourceHash:'0'.repeat(64)}:e),[...catalogue.cards,entry(id)]]){const p=planDefeatCauseProbes(entry('ASH_191'),{catalogue:defs});assert.equal(p.cases.filter(c=>!c.probe.cause.startsWith('first-strike-')).length,24);assert.equal(p.uncovered.filter(x=>x.reason.includes('construction donor')&&!x.id.includes('first-strike-')).length,6);}
    }
    assert.equal(planDefeatCauseProbes(entry('ASH_191'),{catalogue:catalogue.cards.filter(e=>e.setCode!=='SOR_095')}).cases.length,0);
});
test('object insertion order is immaterial but donor arrays/metadata remain exact',()=>{
    const reorder=v=>Array.isArray(v)?v.map(reorder):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).reverse().map(([k,v])=>[k,reorder(v)])):v;
    assert.deepEqual(planDefeatCauseProbes(reorder(entry('ASH_191')),{catalogue:reorder(catalogue.cards)}),planDefeatCauseProbes(entry('ASH_191'),{catalogue}));
});

test('first-strike attack and return windows retain combat cause; Shield cancels trigger; source-only donor profiles exact',()=>{
 const p=planDefeatCauseProbes(entry('ASH_191'),{catalogue});
 for(const owner of [0,1])for(const cause of ['first-strike-incoming','first-strike-return','first-strike-shield']){const rows=p.cases.filter(c=>c.probe.owner===owner&&c.probe.cause===cause);assert.equal(rows.length,1);const c=rows[0];assert.equal(c.probe.firstStrikeCode,'IC27_175');assert.equal(c.probe.expectedAmount,cause==='first-strike-shield'?0:2);assert.equal(c.probe.decline,false);assert.equal(c.probe.recipient,'self');assert.ok(c.setupCardIds.includes(cause==='first-strike-return'?'ASH_197':'LAW_253'));if(cause==='first-strike-shield')assert.ok(c.setupCardIds.includes('TOKEN_SHIELD'));}
 for(const id of ['IC27_175','ASH_197','TOKEN_SHIELD']){const defs=structuredClone(catalogue.cards),bad=defs.find(e=>e.setCode===id);bad.diagnostics=[{code:'UNREVIEWED'}];const next=planDefeatCauseProbes(entry('ASH_191'),{catalogue:defs});assert.equal(next.cases.filter(c=>!c.probe.cause.startsWith('first-strike-')).length,30);assert.ok(next.uncovered.some(x=>x.id.includes('first-strike-')));}
});
