'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {loadCatalog}=require('../../compiler/load-catalog');
const {classifyExhaustedLiveCount,planExhaustedLiveCountProbes}=require('../pipeline/exhausted-live-count-probes');
const catalogue=loadCatalog(process.env.SWU_PROBE_TEST_CATALOGUE),source=catalogue.cards.find(e=>e.setCode==='ASH_035'),copy=structuredClone;
test('complete source and both private helper ASTs classify without engine/compiler execution',()=>{
 assert.deepEqual(classifyExhaustedLiveCount(source),{family:'exhaustedLiveCount',sourceArena:'ground'});
 assert.equal(source.historyHelpers.length,2);assert.equal(source.keywordValues&&Object.keys(source.keywordValues).length,0);
});
test('every source, metadata, intrinsic and unknown-field mutation rejects rather than sanitizing',()=>{
 for(const mutate of [e=>e.extra=true,e=>e.sourceHash='0'.repeat(64),e=>e.dataHash='0'.repeat(64),e=>e.baseClass='LeaderUnitCard',e=>e.status='unsupported',e=>e.diagnostics.push({code:'unknown'}),e=>e.card.types.push('leader'),e=>e.card.keywords.push('Plot'),e=>e.keywordValues.support=1,e=>e.piloting={},e=>e.card.unique=true,e=>e.card.cost=6,e=>e.card.rules=null,e=>e.card.text='different',e=>e.card.extra=true,e=>e.abilities.unit.push({trigger:'action'}),e=>e.abilities.card[0].optional=true,e=>e.abilities.card[0].condition.right.value=1,e=>e.abilities.card[0].effect.restriction='attack',e=>e.abilities.card[1].targetResolver.arena='space',e=>e.abilities.card[1].targetResolver.activePromptTitle='invented',e=>e.abilities.card[1].effect.amount.name='otherHelper',e=>e.historyHelpers.push(copy(e.historyHelpers[0])),e=>e.historyHelpers[0].body.statements[0].value.receiver.receiver.steps[0].key='opponent',e=>e.historyHelpers[0].body.statements[0].value.receiver.filter.condition.body.path[1]='ready',e=>e.historyHelpers[0].body.statements[0].value.receiver.type='nonLeaderUnit',e=>e.historyHelpers[0].body.statements[0].value.receiver.arena='ground',e=>e.historyHelpers[1].body.statements[0].value.left.value=3,e=>e.historyHelpers[1].body.statements.push({kind:'dataReturn',value:{kind:'const',value:2}})]){const e=copy(source);mutate(e);assert.equal(classifyExhaustedLiveCount(e),null);assert.equal(planExhaustedLiveCountProbes(e,{catalogue}),null);}
});
test('raw plans retain both seats, exact source references and explicit source-only gaps',()=>{
 const plan=planExhaustedLiveCountProbes(source,{catalogue});assert(plan.cases.length>0);assert.equal(plan.cases.length%2,0);assert.equal(new Set(plan.cases.map(c=>c.id)).size,plan.cases.length);
 for(const c of plan.cases){assert.equal(c.kind,'probe');assert.equal(c.probe.family,'exhaustedLiveCount');assert.equal(c.probe.resourceCount,24);assert.equal(c.probe.leaderCode,'SOR_014');assert.equal(c.probe.baseCode,'SOR_023');assert(c.setupCardIds.includes('SOR_014'));assert(c.setupCardIds.includes('SOR_023'));assert(c.assertionStages.includes('terminal-drain'));assert(c.focusCardIds.includes('ASH_035'));assert.equal(plan.cases.filter(x=>x.probe.scenario===c.probe.scenario).length,2);assert.deepEqual(plan.cases.filter(x=>x.probe.scenario===c.probe.scenario).map(x=>x.probe.owner),[0,1]);}
 assert(plan.uncovered.some(o=>o.id.endsWith(':source-and-timing-review')));
});
test('missing, changed, duplicate and partially enriched donors cannot construct claimed cases',()=>{
 const plan=planExhaustedLiveCountProbes(source,{catalogue});assert(plan.cases.length>0);
 for(const mutate of [c=>c.cards=c.cards.filter(e=>e.setCode!=='SOR_023'),c=>c.cards.find(e=>e.setCode==='SOR_023').extra=true,c=>c.cards.push(copy(c.cards.find(e=>e.setCode==='SOR_023'))),c=>delete c.cards.find(e=>e.setCode==='SOR_023').diagnostics]){const c=copy(catalogue);mutate(c);const p=planExhaustedLiveCountProbes(source,{catalogue:c});assert.equal(p.cases.length,0);assert(p.uncovered.some(o=>o.reason.includes('SOR_023')));}
});
test('planning is nonmutating and preserves every prior full-card plan',()=>{
 const {planCardProbes}=require('../pipeline/probes');const originals=catalogue.cards.filter(e=>e.setCode!=='ASH_035'),prior=originals.map(e=>planCardProbes(e,{catalogue})),input=copy(catalogue);
 planExhaustedLiveCountProbes(source,{catalogue});assert.deepEqual(catalogue,input);assert.deepEqual(originals.map(e=>planCardProbes(e,{catalogue})),prior);
});
test('paid Support and Sentinel construction donors retain complete native lifecycles',()=>{
 const plan=planExhaustedLiveCountProbes(source,{catalogue});
 for(const c of plan.cases){assert(c.setupCardIds.includes('ASH_131'));assert(c.setupCardIds.includes('SOR_095'));assert.equal('fillerCodes' in c.probe,false);}
 for(const [code,mutate,affected]of [['SEC_071',e=>e.keywordValues.sentinel=1,4],['ASH_033',e=>e.keywordValues.support=0,2]]){const c=copy(catalogue);mutate(c.cards.find(e=>e.setCode===code));const changed=planExhaustedLiveCountProbes(source,{catalogue:c});assert.equal(plan.cases.length-changed.cases.length,affected);assert(changed.uncovered.some(o=>o.reason.includes(code)));}
 const self=plan.cases.find(c=>c.probe.scenario==='source-self-lethal');for(const id of ['ASH_061','JTL_069','ASH_190'])assert(self.setupCardIds.includes(id));
});

test('printed Train seven and donor payments match Sabine plus Command Center aspect inventory',()=>{
 const leader=catalogue.cards.find(e=>e.setCode==='SOR_014'),base=catalogue.cards.find(e=>e.setCode==='SOR_023'),supplied=[...leader.card.aspects,...base.card.aspects];
 function cost(code){const e=catalogue.cards.find(e=>e.setCode===code),left=[...supplied];return e.card.cost+e.card.aspects.reduce((penalty,aspect)=>{const i=left.indexOf(aspect);if(i<0)return penalty+2;left.splice(i,1);return penalty;},0);}
 assert.equal(cost('ASH_035'),7);assert.equal(cost('SEC_071'),5);assert.equal(cost('ASH_033'),9);assert.equal(cost('LAW_043'),9);assert.equal(cost('TWI_237'),4);assert.equal(cost('TWI_226'),5);
});

test('ground-target cases disclose their exhausted helper and require its complete definition',()=>{
 const names=['friendly-ground-target','enemy-ground-target'];
 const cases=planExhaustedLiveCountProbes(source,{catalogue}).cases.filter(c=>names.includes(c.probe.scenario));
 assert.equal(cases.length,4);for(const c of cases)assert(c.setupCardIds.includes('ASH_061'));
 const missing=copy(catalogue);missing.cards=missing.cards.filter(e=>e.setCode!=='ASH_061');
 assert.equal(planExhaustedLiveCountProbes(source,{catalogue:missing}).cases.filter(c=>names.includes(c.probe.scenario)).length,0);
});
