'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {classifyPlayerPhaseEffect}=require('../pipeline/player-phase-effect-probes');
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('whole source registrations distinguish standard-payment discount and printed-power entry matcher',()=>{
    assert.deepEqual(classifyPlayerPhaseEffect(entry('ASH_237')),{family:'player-phase-effect',kind:'mouse-cost'});
    assert.deepEqual(classifyPlayerPhaseEffect(entry('ASH_248')),{family:'player-phase-effect',kind:'neel-ready'});
    for(const id of ['ASH_237','ASH_248']){const e=entry(id);e.setCode='CUSTOM_001';e.className='Renamed';e.card.title='Renamed';assert.ok(classifyPlayerPhaseEffect(e));}
});
test('reject companions, mixed card types, incorrect keyword metadata, and incomplete trigger registrations',()=>{
    for(const id of ['ASH_237','ASH_248'])for(const mutate of [e=>e.card.types.push('token'),e=>e.baseClass='LeaderUnitCard',e=>e.card.arena='space',e=>e.card.cost=99,e=>e.card.power=2,e=>e.card.traits=[],e=>e.card.aspects=[],e=>e.card.pilotText='Pilot',e=>e.card.deployBox='Deploy',e=>e.attachCondition={},e=>e.piloting={},e=>e.abilities.other=[],e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].condition={},e=>e.abilities.card[0].then={},e=>e.abilities.card[0].costs.push({kind:'exhaustSelf'}),e=>e.card.keywords.push('hidden'),e=>e.keywordValues.hidden=1,e=>e.sourceHash='unverified']){const e=entry(id);mutate(e);assert.equal(classifyPlayerPhaseEffect(e),null);}
    const n=entry('ASH_248');n.abilities.card.pop();assert.equal(classifyPlayerPhaseEffect(n),null);
});
test('Mouse requires all defaults, cloned one-use limit, matching type/trait and standard payment flags',()=>{
    for(const mutate of [e=>e.target='enemyPlayer',e=>e.duration='persistent',e=>e.effects.push(structuredClone(e.effects[0])),e=>e.effects[0].stage='standard_1',e=>e.effects[0].matchAbilityCosts=true,e=>e.effects[0].matchCardEffectResourcePayments=true,e=>e.effects[0].cardTypeFilter='card',e=>e.effects[0].limit.clonePerEffectApplication=false,e=>e.effects[0].limit.max=2,e=>e.effects[0].match.timing='onCardPlayed',e=>e.effects[0].match.value.receiver.path=['source'],e=>e.effects[0].match.value.args[0].value='rebel',e=>e.effects[0].amount.value=2]){const e=entry('ASH_237');mutate(e.abilities.card[0].effect);assert.equal(classifyPlayerPhaseEffect(e),null);}
});
test('Neel requires both exact matchers, printed power, played entry and consumption timing; bound alpha-renaming stays local',()=>{
    for(const mutate of [e=>e.target='enemyPlayer',e=>e.effects[0].entryType='created',e=>e.effects[0].matchTiming='onCardPlayed',e=>e.effects[0].limitIncrementTiming='eventResolved',e=>e.effects[0].limit.clonePerApplication=false,e=>e.effects[0].match.parameters[0].role='source',e=>e.effects[0].match.body.left.name='getPower',e=>e.effects[0].match.body.left.receiver.path=['source'],e=>e.effects[0].match.body.right.value=2]){const e=entry('ASH_248');mutate(e.abilities.card[1].effect);assert.equal(classifyPlayerPhaseEffect(e),null);}
    const alpha=entry('ASH_248');for(const a of alpha.abilities.card){a.effect.effects[0].match.parameters[0].root='entering';a.effect.effects[0].match.body.left.receiver.path=['entering'];}assert.ok(classifyPlayerPhaseEffect(alpha));
});
test('construction donors require whole reviewed metadata/registrations and current exact admitted pool rows',()=>{
    const {reviewedDonor,admittedDonor}=require('../pipeline/player-phase-effect-probes');
    const pool=require('../../compiler/generated/runtime-pool.json');
    for(const [role,id]of Object.entries({mouseLeader:'IBH_053',neelLeader:'SEC_005',base:'SOR_023',imperial:'IBH_075',imperialZero:'IBH_063',nonmatching:'LOF_254',ready:'SOR_169',killer:'ASH_061',printedOne:'IBH_007',printedTwo:'HMW_116',printedZero:'SEC_216',alreadyReady:'HMW_208',tokenCreator:'TWI_237',buff:'SOR_242',debuff:'SHD_037',battleDroid:'TOKEN_BATTLE_DROID'})){
        const e=entry(id);assert.ok(reviewedDonor(e,role),role);assert.ok(admittedDonor(e,pool),id);
        const bad=structuredClone(e);bad.abilities.card.push({trigger:'whenPlayed',effect:{kind:'noAction'}});assert.equal(reviewedDonor(bad,role),false);
        const meta=structuredClone(e);meta.card.traits=['unreviewed'];assert.equal(reviewedDonor(meta,role),false);assert.equal(admittedDonor(meta,pool),false);
        assert.equal(admittedDonor(e,{cards:[]}),false);
    }
});
test('both complete family plans retain concrete steps, both seats and the actual distinguishing boundaries',()=>{
    const {planPlayerPhaseEffectProbes:plan}=require('../pipeline/player-phase-effect-probes');
    const mouse=plan(entry('ASH_237'),{catalogue}),neel=plan(entry('ASH_248'),{catalogue});assert.equal(mouse.cases.length,12);assert.equal(neel.cases.length,18);
    for(const p of [mouse,neel])for(const owner of [0,1])assert.equal(p.cases.filter(c=>c.probe.owner===owner).length,p.cases.length/2);
    for(const p of [mouse,neel])for(const c of p.cases){assert.ok(c.probe.steps.length>=3);assert.equal(c.probe.steps[0].type,'paid-play');assert.equal(c.probe.steps[0].card,'source');assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));assert.ok(c.setupCardIds.includes(c.probe.donors.leader));}
    const mice=mouse.cases.find(c=>c.probe.scenario==='sequential-mice').probe;assert.deepEqual(mice.steps[1],{type:'paid-play',card:'secondMouse',expect:{discount:1,paid:0,activeCost:1}});assert.ok(mouse.uncovered.some(o=>o.id.endsWith('two-active-discounts')));
    const cancel=mouse.cases.find(c=>c.probe.scenario==='raid-evaluation-cancel').probe;assert.equal(cancel.steps.find(s=>s.type==='cancel-play').card,'readyEvent');assert.ok(mouse.uncovered.some(o=>o.id.endsWith('matching-unit-prepayment-cancel')));
    const stack=neel.cases.find(c=>c.probe.scenario==='wp-attack-stack').probe;assert.deepEqual(stack.steps.find(s=>s.expect?.allReadyUsed===2).expect,{ready:true,activeReady:0,allReadyUsed:2});
    const typed=neel.cases.find(c=>c.probe.scenario==='token-deploy-exclusion');assert.ok(typed.setupCardIds.includes('TOKEN_BATTLE_DROID'));assert.equal(typed.probe.steps.find(s=>s.type==='deploy').expect.activeReady,1);
    for(const p of [mouse,neel])assert.ok(p.uncovered.some(o=>o.id.endsWith('replacement-after-event-generation')));
});
test('missing or altered reviewed runtime donors yield explicit uncovered construction rather than a partial standard matrix',()=>{
    const {planPlayerPhaseEffectProbes:plan}=require('../pipeline/player-phase-effect-probes');
    for(const id of ['ASH_237','ASH_248']){const missing=plan(entry(id),{catalogue,runtimePool:{cards:[]}});assert.equal(missing.cases.length,0);assert.ok(missing.uncovered.some(o=>o.id.includes('construction-')));}
    const c={cards:catalogue.cards.map(e=>{if(e.setCode!=='IBH_075')return e;const bad=structuredClone(e);bad.abilities.card.push({});return bad;})};
    assert.equal(plan(entry('ASH_237'),{catalogue:c}).cases.length,0);
    const renamed=entry('ASH_237');renamed.setCode='CUSTOM_001';const p=plan(renamed,{catalogue});assert.equal(p.cases.length,12);assert.ok(p.cases.every(c=>c.probe.sourceCode==='CUSTOM_001'&&c.probe.donors.secondMouse==='CUSTOM_001'));
});
