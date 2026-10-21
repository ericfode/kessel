'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const {loadCatalog}=require('../../compiler/load-catalog');
const {classifyDamagedTargetPhaseHistory:classify,planDamagedTargetPhaseHistoryProbes:planner}=require('../pipeline/damaged-target-phase-history-probes');
const catalogue=loadCatalog(),entry=catalogue.cards.find(row=>row.setCode==='ASH_188');
const plan=(c=catalogue,e=entry)=>planner(e,{catalogue:c});
function mutated(change) {const c=structuredClone(catalogue);change(c);return c;}
test('complete pinned source uses physical damaged-target history rather than wounds or incarnation',()=>{
    assert.deepEqual(classify(entry),{family:'damaged-target-phase-history',kind:'galvanized-leap'});
    assert.equal(entry.card.cost,4);assert.deepEqual(entry.card.aspects,['aggression']);assert.deepEqual(entry.card.traits,['force']);assert.deepEqual(entry.keywordValues,{});
    const a=entry.abilities.card[0],q=a.targetResolver.predicate.right;
    assert.deepEqual([q.watcher,q.operation,q.readAt],['damageDealtThisPhase','wasUnitDamagedThisPhase','expressionEvaluation']);
    assert.deepEqual(q.args,[{kind:'ref',path:['card']}]);assert.deepEqual(a.effect,{kind:'ready',target:'chosen'});
});
test('whole unchanged source rejects unknown fields and every changed lifecycle/target boundary',()=>{
    for(const change of[
        e=>e.sourceHash='0'.repeat(64),e=>e.dataHash='0'.repeat(64),e=>e.status='unsupported',e=>e.className='Unknown',e=>e.baseClass='NonLeaderUnitCard',
        e=>e.card.cost++,e=>e.card.types=['event','unit'],e=>e.card.keywords=['plot'],e=>e.keywordValues.plot=0,e=>e.card.unknown=true,e=>e.unknown=true,
        e=>e.abilities.unit.push(e.abilities.card[0]),e=>e.abilities.card[0].optional=true,e=>e.abilities.card[0].targetResolver.owner='self',
        e=>e.abilities.card[0].targetResolver.predicate.right.operation='unitHasDealtDamage',e=>e.abilities.card[0].targetResolver.predicate.right.args[0].path=['card','inPlayId'],
        e=>e.abilities.card[0].targetResolver.predicate.right={kind:'ref',path:['card','damage']},e=>e.abilities.card[0].targetResolver.predicate.left.args=[{kind:'const',value:1}],
        e=>e.abilities.card[0].effect.kind='exhaust',e=>e.abilities.card[0].then={effect:{kind:'draw'}},e=>e.stateWatchers[0].resetEvent='onRoundEnded',
        e=>e.stateWatchers[0].sharedScope='player',e=>e.stateWatchers[0].unknown=true,e=>e.diagnostics.push({unreviewed:true})
    ]) {const e=structuredClone(entry);change(e);assert.equal(classify(e),null);assert.equal(planner(e,{catalogue}),null);}
    for(const e of[null,undefined,[],{}, {card:[]}])assert.equal(classify(e),null);
});
test('helper envelope rejects malformed rows before filtering, hashes and ordered multiplicity',()=>{
    for(const change of[
        c=>c.inputs.push(null),c=>c.inputs.push({sha256:'0'.repeat(64)}),c=>c.inputs.push({file:'outside',sha256:'bad'}),
        c=>c.inputs.push({...c.inputs[0],extra:1}),c=>c.inputs.push({...c.inputs[0]}),
        c=>c.inputs.push({file:'server/new-helper.ts',sha256:'0'.repeat(64)}),
        c=>{const rows=c.inputs.filter(r=>r.file.startsWith('server/')&&!r.file.startsWith('server/game/cards/'));rows[0].sha256='0'.repeat(64);},
        c=>c.inputs.reverse(),c=>{c.inputs=c.inputs.filter(r=>r.file!=='server/game/gameSystems/ReadySystem.ts');}
    ]) {const p=plan(mutated(change));assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(g=>g.id.endsWith(':helper-source-inputs')));}
    assert.equal(planner(entry,{catalogue:catalogue.cards}).cases.length,0);
});
test('full selected source/data inputs cannot be replaced by agreeing metadata profiles',()=>{
    for(const name of['server/game/cards/08_ASH/events/GalvanizedLeap.ts','data/Card/galvanized-leap.json']) {
        const row=catalogue.inputs.find(r=>r.file===name)||(name.startsWith('data/')&&catalogue.inputs.find(r=>r.sha256===entry.dataHash));assert.ok(row);
        for(const change of[c=>{c.inputs=c.inputs.filter(r=>r.file!==row.file);},c=>{c.inputs.find(r=>r.file===row.file).sha256='0'.repeat(64);}]){
            const p=plan(mutated(change));assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(g=>g.id.endsWith(':card-source-inputs')));
        }
    }
});
test('every literal deck, default and token dependency has an unchanged unique full definition',()=>{
    const good=plan();assert.ok(good.cases.length);
    for(const id of good.cases[0].setupCardIds) {
        for(const change of[c=>{c.cards.find(r=>r.setCode===id).card.unknown=true;},c=>{c.cards.push(structuredClone(c.cards.find(r=>r.setCode===id)));},c=>{c.cards=c.cards.filter(r=>r.setCode!==id);}]) {
            const p=plan(mutated(change));assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(g=>g.id.endsWith(':complete-construction-envelope')));
        }
    }
});
test('plans preserve caller inputs, exact formal obligation links and explicit remaining gaps',()=>{
    const before=JSON.stringify(catalogue),p=plan();assert.equal(JSON.stringify(catalogue),before);
    assert.equal(p.schema,'swu-card-probes/v1');assert.equal(p.cardId,'ASH_188');assert.equal(p.sourceHash,entry.sourceHash);
    const ids=new Set(p.obligations.map(o=>o.id));assert.equal(ids.size,p.obligations.length);assert.ok(p.uncovered.length);
    for(const c of p.cases){assert.equal(c.kind,'probe');assert.deepEqual(c.focusCardIds,['ASH_188']);assert.ok(c.coverageObligationIds.length);for(const id of c.coverageObligationIds)assert.ok(ids.has(id));assert.equal(c.probe.family,'damaged-target-phase-history');assert.match(c.id,/^auto\/ASH_188\//);}
    const again=plan();p.cases[0].probe.owner=99;assert.deepEqual(plan(),again);
    assert.ok(p.uncovered.some(g=>/native.*Wasm/.test(g.reason)));
});
test('both seats get complete50card literal decks and explicit positive, cancellation and interaction recipes',()=>{
    const scenarios=['own-ground-damage','enemy-ground-damage','own-space-damage','enemy-space-damage','healed-damage-history','bounce-replay-same-physical','no-history-play-anyway','no-history-cancel','shield-prevented-play-anyway','zero-power-no-event-play-anyway','phase-reset-play-anyway','ready-target-with-effective-alternative','leader-damage','target-cancel'];
    const p=plan();assert.equal(p.cases.length,28);
    for(const owner of[0,1])assert.deepEqual(p.cases.filter(c=>c.probe.owner===owner).map(c=>c.probe.scenario).sort(),[...scenarios].sort());
    for(const c of p.cases){const r=c.probe;assert.equal(r.deck.reduce((n,row)=>n+row.count,0),50);assert.equal(new Set(r.deck.map(row=>row.code)).size,17);assert.deepEqual(r.deck.map(row=>row.code),r.deckCardIds);
        for(const row of r.deck){assert.ok(row.count>=1&&row.count<=3);const e=catalogue.cards.find(e=>e.setCode===row.code);assert.ok(e);assert.ok(e.card.types.some(t=>['unit','event','upgrade'].includes(t)));assert.ok(!e.card.types.some(t=>['leader','base','token'].includes(t)));}
        assert.ok(c.setupCardIds.includes('SOR_014'));assert.ok(c.setupCardIds.includes('SOR_023'));assert.ok(c.setupCardIds.includes('TOKEN_SHIELD'));assert.deepEqual(c.setupCardIds,r.selectedCardIds);
        const focus=r.scripts.filter(s=>s.kind==='focus');assert.equal(focus.length,1);assert.equal(focus[0].cost,focus[0].choice==='cancel'?0:4);
        assert.ok(c.assertionStages.includes('terminal-completion'));assert.ok(c.assertionStages.includes('actual-damage-history'));
    }
    const by=s=>p.cases.find(c=>c.probe.owner===0&&c.probe.scenario===s).probe;
    assert.deepEqual(by('healed-damage-history').scripts.map(s=>s.card),['damage','repair','repair2',undefined]);
    assert.deepEqual(by('bounce-replay-same-physical').scripts.map(s=>s.card),['damage','waylay','target',undefined]);
    assert.ok(by('phase-reset-play-anyway').scripts.some(s=>s.kind==='phase'));
    assert.equal(by('ready-target-with-effective-alternative').unitSetup.find(u=>u.key==='target').exhausted,false);
    assert.equal(by('ready-target-with-effective-alternative').unitSetup.find(u=>u.key==='effective').exhausted,true);
    assert.ok(p.uncovered.some(g=>g.id.endsWith(':emitted-zero-damage')));
});
test('ability-damage targets survive actual printed4damage and full-heal uses two real Repair events',()=>{
    const p=plan(),damage=catalogue.cards.find(e=>e.setCode==='TWI_174').abilities.card[0].effect.amount.value,heal=catalogue.cards.find(e=>e.setCode==='JTL_075').abilities.card[0].effect.amount.value;
    assert.equal(damage,4);assert.equal(heal,3);
    for(const c of p.cases.filter(c=>['own-ground-damage','enemy-ground-damage','own-space-damage','enemy-space-damage','healed-damage-history','bounce-replay-same-physical'].includes(c.probe.scenario))){const target=c.probe.unitSetup.find(u=>u.key==='target'),def=catalogue.cards.find(e=>e.setCode===target.code);assert.ok(def.card.hp>damage);}
    const healed=p.cases.find(c=>c.probe.scenario==='healed-damage-history').probe;
    assert.equal(healed.scripts.filter(s=>s.kind==='play'&&s.card.startsWith('repair')).length,Math.ceil(damage/heal));
    const bounce=p.cases.find(c=>c.probe.scenario==='bounce-replay-same-physical').probe;
    assert.equal(bounce.unitSetup.find(u=>u.key==='target').code,'ASH_061');assert.equal(bounce.scripts.find(s=>s.card==='target').cost,6);
});
test('physical twin and previous-phase negatives can distinguish history from copied definition or current wounds',()=>{
    const p=plan();
    for(const c of p.cases.filter(c=>['own-ground-damage','enemy-ground-damage'].includes(c.probe.scenario))){
        const r=c.probe,target=r.unitSetup.find(u=>u.key==='target'),twin=r.unitSetup.find(u=>u.key==='twin');
        assert.ok(twin);assert.equal(twin.code,target.code);assert.equal(twin.owner,target.owner);assert.equal(twin.exhausted,true);
        assert.ok(!r.scripts.some(s=>s.target==='twin'));assert.equal(r.scripts.at(-1).target,'target');
    }
    for(const c of p.cases.filter(c=>c.probe.scenario==='phase-reset-play-anyway')){
        assert.deepEqual(c.probe.scripts.map(s=>s.kind),['play','phase','attack','focus']);
        assert.deepEqual(c.probe.scripts[2],{kind:'attack',card:'target',target:'enemy-base'});
        assert.equal(c.probe.scripts[3].choice,'play-anyway');
    }
});

test('transitive Advantage dependency retains full authoritative entry and physical inputs',()=>{
 for(const mode of ['missing','poison','duplicate']){const c=structuredClone(catalogue),i=c.cards.findIndex(c=>c.setCode==='TOKEN_ADVANTAGE');if(mode==='missing')c.cards.splice(i,1);if(mode==='poison')c.cards[i].card.rules='poison';if(mode==='duplicate')c.cards.push(structuredClone(c.cards[i]));assert.equal(plan(c).cases.length,0,mode);}
 const c=structuredClone(catalogue);c.inputs=c.inputs.filter(r=>!r.file.endsWith('/tokens/Advantage.ts'));assert.equal(plan(c).cases.length,0);assert.equal(plan().cases.length,28);
});
