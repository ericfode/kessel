'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {loadCatalog}=require('../../compiler/load-catalog');
const {classifyAmbushContext,planAmbushContextProbes}=require('../pipeline/ambush-context-probes');
// Unit tests require only authoritative committed catalogue shards, never the engine/compiler.
const catalogue=loadCatalog();
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
const plan=()=>planAmbushContextProbes(entry('ASH_207'),{catalogue});
test('complete source profile includes native Ambush and both seats for all18 material paths',()=>{
    const p=plan();assert.ok(p);assert.equal(p.cases.length,18);assert.equal(p.sourceHash,entry('ASH_207').sourceHash);
    for(const sourceSeat of [1,2])for(const scenario of ['accept','decline','cancel-retry','sentinel','no-enemy','attacker-death','defender-death','defender-death-lethal','mirror-defender']){
        const c=p.cases.find(c=>c.probe.sourceSeat===sourceSeat&&c.probe.scenario===scenario);assert.ok(c);assert.equal(c.probe.sourceArena,'space');assert.equal(c.probe.family,'ambush-context');assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));
        for(const t of c.probe.targets)assert.ok(t.damage===undefined||t.damage>=0,'No invalid negative damage setup');
    }
    for(const d of Object.values(p.donorAdmissionReceipt.donors)){assert.match(d.entrySemanticSha256,/^[a-f0-9]{64}$/);assert.ok(d.implementation);}
});
test('ordinary versus accepted Ambush protocol and Sentinel/no-enemy scopes stay distinct',()=>{
    const p=plan();
    const retry=p.cases.find(c=>c.probe.scenario==='cancel-retry');assert.equal(retry.probe.decline,true);assert.equal(retry.probe.cancelRetry,true);assert.equal(retry.probe.laterNormal,true);assert.ok(retry.assertionStages.includes('normal-cancel-retry'));
    const sentinel=p.cases.find(c=>c.probe.scenario==='sentinel').probe;assert.equal(sentinel.defenderIndex,1);assert.equal(sentinel.sentinelIndex,1);assert.equal(sentinel.targets[1].code,'ASH_117');
    const empty=p.cases.find(c=>c.probe.scenario==='no-enemy').probe;assert.equal(empty.defenderIndex,null);assert.ok(empty.targets.every(t=>t.owner==='self'));assert.equal(empty.laterNormal,true);
    assert.ok(!p.cases.find(c=>c.probe.scenario==='attacker-death').assertionStages.includes('later-normal-power'));
});
test('valid actual Legacy HP and allocations distinguish live nested attack context from cleanup',()=>{
    const p=plan();
    for(const seat of [1,2]){
        const live=p.cases.find(c=>c.probe.sourceSeat===seat&&c.probe.scenario==='defender-death').probe;
        assert.deepEqual(live.targets[0],{code:'LOF_213',owner:'opponent',arena:'space'});assert.equal(entry('LOF_213').card.hp,3);assert.deepEqual(live.deathDistribution,[{index:'source',amount:2},{index:1,amount:4}]);assert.equal(live.laterNormal,true);
        const lethal=p.cases.find(c=>c.probe.sourceSeat===seat&&c.probe.scenario==='defender-death-lethal').probe;assert.deepEqual(lethal.deathDistribution,[{index:'source',amount:6}]);assert.equal(lethal.laterNormal,false);
    }
});
test('classifier follows closed structural source template without focus ID dispatch',()=>{
    const e=entry('ASH_207');e.setCode='CUSTOM_001';e.className='Other';e.card.title='Other';e.internalName='other';assert.ok(classifyAmbushContext(e));
});
test('unknown fields, partial card metadata, changed query and legacy memberPath remain rejected',()=>{
    for(const mutate of [e=>e.other=true,e=>e.card.extra=true,e=>e.card.types.push('leader'),e=>e.card.unique=true,e=>e.card.arena='ground',e=>e.card.cost++,e=>e.card.hp++,e=>e.card.power++,e=>e.card.text+=' Support',e=>e.card.keywords.push('support'),e=>e.keywordValues.ambush=2,e=>e.keywordValues.other=1,e=>e.card.pilotText='Pilot',e=>e.piloting={},e=>e.attachCondition={},e=>e.card.deployBox='Deploy',e=>e.sourceHash='',e=>e.dataHash='',e=>e.abilities.extra=[],e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].then={},e=>e.abilities.card[0].costs.push({kind:'exhaustSelf'}),e=>e.abilities.card[0].condition.extra=false,e=>e.abilities.card[0].condition.name='isAttacking',e=>e.abilities.card[0].condition.receiver.path=['target'],e=>e.abilities.card[0].condition.args.push({kind:'const',value:true}),e=>e.abilities.card[0].condition={kind:'memberPath'},e=>e.abilities.card[0].effect.target='chosen',e=>e.abilities.card[0].effect.duration='attack',e=>e.abilities.card[0].effect.power.value=3,e=>e.abilities.card[0].effect.hp.value=1,e=>e.abilities.card[0].effect.extra=true]){const e=entry('ASH_207');mutate(e);assert.equal(classifyAmbushContext(e),null);}
});
test('whole donor semantics ignore object key order but retain every source/data/nested value',()=>{
    const reorder=x=>Array.isArray(x)?x.map(reorder):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).reverse().map(k=>[k,reorder(x[k])])):x;
    const p=plan(),r=planAmbushContextProbes(reorder(entry('ASH_207')),{catalogue:reorder(catalogue)});assert.deepEqual(r,p);
    for(const id of ['JTL_069','ASH_117','LAW_164','LOF_213'])for(const mutate of [e=>e.sourceHash='0'.repeat(64),e=>e.dataHash='0'.repeat(64),e=>e.card.traits.push('unreviewed')]){
        const cards=structuredClone(catalogue.cards);mutate(cards.find(e=>e.setCode===id));const invalid=planAmbushContextProbes(entry('ASH_207'),{catalogue:{cards}});assert.equal(invalid.cases.length,0);assert.ok(invalid.uncovered.some(u=>u.id.endsWith('donor-admission')));
    }
    const fake=entry('JTL_069');fake.setCode='FAKE_001';assert.equal(planAmbushContextProbes(entry('ASH_207'),{catalogue:{cards:[...catalogue.cards,fake]},ambushContextDonors:{frigate:fake.setCode}}).cases.length,0);
});

test('the same focus definition as defender receives no Ambush bonus from another attacker',()=>{
    const p=plan();
    for(const sourceSeat of [1,2]){
        const c=p.cases.find(c=>c.probe.sourceSeat===sourceSeat&&c.probe.scenario==='mirror-defender');assert.ok(c);
        assert.deepEqual(c.probe.targets,[{code:'ASH_207',owner:'opponent',arena:'space'}]);assert.equal(c.probe.assertDefenderNoBonus,true);assert.equal(c.probe.laterNormal,true);
        assert.ok(c.assertionStages.includes('defending-focus-power'));assert.ok(c.assertionStages.includes('physical-role-context'));
    }
});
