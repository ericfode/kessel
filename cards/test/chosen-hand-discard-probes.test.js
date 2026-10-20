'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { loadCatalog } = require('../../compiler/load-catalog');
const { classifyChosenHandDiscard, planChosenHandDiscardProbes, KEEPER_PROMPT } = require('../pipeline/chosen-hand-discard-probes');
// Ordinary unit tests use authoritative committed shards, without engine/compiler dependencies.
const catalogue = loadCatalog();
const entry = id => structuredClone(catalogue.cards.find(e => e.setCode === id));
const plan = id => planChosenHandDiscardProbes(entry(id), { catalogue });
test('complete source-derived profiles construct both seats with physical hand and target boundaries', () => {
    for (const [id,profile,count] of [['ASH_226','qira',28],['HMW_041','keeper',22]]) {
        const p = plan(id); assert.ok(p); assert.equal(p.cases.length,count); assert.equal(p.sourceHash,entry(id).sourceHash);
        for (const seat of [1,2]) {
            assert.ok(p.cases.some(c => c.probe.sourceSeat === seat));
            for (const scenario of [profile==='qira'?'no-choose-nothing':'decline-done','decline-pass']) { const c=p.cases.find(c=>c.probe.sourceSeat===seat&&c.probe.scenario===scenario);assert.ok(c);assert.deepEqual(c.probe.discardIndexes,[]); }
        }
        for (const c of p.cases) { assert.equal(c.probe.profile,profile); assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id))); assert.ok(c.probe.handCodes.every(code=>code===id||c.setupCardIds.includes(code))); }
        assert.match(p.donorAdmissionReceipt.reportByteSha256,/^[a-f0-9]{64}$/);
        assert.equal(p.donorAdmissionReceipt.donors.shield.implementation,'nativeAdapter');
    }
});
test('Qi live stats include every unit target scope, hand size and actual draw-hook constructions', () => {
    const p=plan('ASH_226');
    const noDone=p.cases.find(c=>c.probe.scenario==='no-choose-nothing').probe;assert.equal(noDone.assertNoDone,true);assert.equal(noDone.decline,'passAbility');assert.ok(p.cases.every(c=>c.probe.decline!=='done'));
    for (const seat of [1,2]) for (const scenario of ['damage-self','damage-own-ground','damage-enemy-ground','damage-own-space','damage-enemy-space','empty-hand','single-card','large-hand','self-draw','opponent-draw']) assert.ok(p.cases.some(c=>c.probe.sourceSeat===seat&&c.probe.scenario===scenario));
    assert.equal(p.cases.find(c=>c.probe.scenario==='large-hand').probe.handCodes.length,10);
    const draw=p.cases.find(c=>c.probe.scenario==='self-draw').probe;assert.ok(draw.handCodes.includes(draw.drawCode));
    assert.deepEqual(p.cases.find(c=>c.probe.scenario==='opponent-draw').probe.opponentHandCodes,['ASH_087']);
});
test('Keeper exact2 toggles, Restore orders, Shield and terminal cleanup are separate bounded cases', () => {
    const e=entry('HMW_041');assert.equal(e.abilities.card[0].targetResolver.promptTitle,KEEPER_PROMPT);
    const p=plan('HMW_041');
    for (const seat of [1,2]) for (const scenario of ['restore-first','custom-first','zero-matching','one-matching','undamaged-base','toggle-three','shield','attack-base','terminal-attack-cleanup']) assert.ok(p.cases.some(c=>c.probe.sourceSeat===seat&&c.probe.scenario===scenario));
    const toggle=p.cases.find(c=>c.probe.scenario==='toggle-three').probe;assert.deepEqual(toggle.toggleSequence,[0,1,0,2]);assert.deepEqual(toggle.discardIndexes,[1,2]);assert.equal(toggle.handCodes.filter(x=>x==='HMW_041').length,3);
    const shield=p.cases.find(c=>c.probe.scenario==='shield');assert.equal(shield.probe.defenderUpgradeCode,'ASH_086');assert.ok(shield.setupCardIds.includes('TOKEN_SHIELD'));
});
test('profile identity is source structure; missing Keeper prompt and partial intrinsic metadata reject', () => {
    const q=entry('ASH_226');q.setCode='CUSTOM_001';q.className='Other';q.card.title='Other';assert.ok(classifyChosenHandDiscard(q));
    const k=entry('HMW_041');k.setCode='CUSTOM_002';k.className='Other';assert.ok(classifyChosenHandDiscard(k));
    delete k.abilities.card[0].targetResolver.promptTitle;assert.equal(classifyChosenHandDiscard(k),null);
    const original=loadCatalog().cards.find(e=>e.setCode==='HMW_041');if (!original.abilities.card[0].targetResolver.promptTitle) assert.equal(classifyChosenHandDiscard(original),null);
});
test('complete envelopes reject hidden fields, changed selection and continuation semantics', () => {
    const shared=[e=>e.extra={},e=>e.card.unique=false,e=>e.card.types.push('leader'),e=>e.card.power++,e=>e.card.hp++,e=>e.card.cost++,e=>e.card.text+=' Plot',e=>e.card.arena='space',e=>e.piloting={},e=>e.card.deployBox='Deploy',e=>e.card.keywords.push('plot'),e=>e.keywordValues.overwhelm=1,e=>e.abilities.unit.push({}),e=>e.abilities.card.push({}),e=>e.abilities.extra=[],e=>e.sourceHash='',e=>e.dataHash=''];
    const ability=[a=>a.optional=false,a=>a.costs.push({kind:'exhaustSelf'}),a=>a.targetResolver.owner='opponent',a=>a.targetResolver.zone='discard',a=>a.targetResolver.extra=true,a=>a.effect.extra=true,a=>a.effect.target='ownPlayer',a=>a.ifYouDo.optional=true,a=>a.ifYouDo.then={}];
    for(const id of ['ASH_226','HMW_041']) {
        for(const mutate of shared){const e=entry(id);mutate(e);assert.equal(classifyChosenHandDiscard(e),null);}
        for(const mutate of ability){const e=entry(id);mutate(e.abilities.card[id==='ASH_226'?1:0]);assert.equal(classifyChosenHandDiscard(e),null);}
    }
    for(const mutate of [a=>a.targetResolver.mode='upTo',a=>a.targetResolver.count.value=1,a=>a.targetResolver.promptTitle='Wrong',a=>a.targetResolver.predicate.right.value='Other',a=>a.targetResolver.predicate.left.path[1]='internalName',a=>a.ifYouDo.effect.effects.reverse(),a=>a.ifYouDo.effect.effects[0].duration='phase',a=>a.ifYouDo.effect.effects[0].power.value=14,a=>a.ifYouDo.effect.effects.push({})]){const e=entry('HMW_041');mutate(e.abilities.card[0]);assert.equal(classifyChosenHandDiscard(e),null);}
});
test('donors preserve complete semantics across catalogue loading and recursive key order', () => {
    const reorder=x=>Array.isArray(x)?x.map(reorder):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).reverse().map(k=>[k,reorder(x[k])])):x;
    const shuffled=reorder(catalogue),loaded=loadCatalog();
    for(const id of ['ASH_226','HMW_041']) {
        const p=plan(id),r=planChosenHandDiscardProbes(reorder(entry(id)),{catalogue:shuffled});assert.deepEqual(r.cases,p.cases);assert.deepEqual(r.donorAdmissionReceipt,p.donorAdmissionReceipt);
        const loadedDonors=planChosenHandDiscardProbes(entry(id),{catalogue:loaded});assert.equal(loadedDonors.cases.length,p.cases.length);
    }
});
test('actual donor mutations and unadmitted aliases cannot fabricate constructions', () => {
    for(const id of ['JTL_075','SOR_095','SOR_112','ASH_087','ASH_086','TOKEN_SHIELD']) for(const mutate of [e=>e.card.traits.push('unreviewed'),e=>e.sourceHash='0'.repeat(64),e=>e.dataHash='0'.repeat(64)]){
        const cards=structuredClone(catalogue.cards);mutate(cards.find(e=>e.setCode===id));const p=planChosenHandDiscardProbes(entry('ASH_226'),{catalogue:{cards}});assert.equal(p.cases.length,0);assert.ok(p.uncovered.some(u=>u.id.endsWith('donor-admission')));
    }
    const fake=entry('SOR_095');fake.setCode='FAKE_001';const p=planChosenHandDiscardProbes(entry('ASH_226'),{catalogue:{cards:[...catalogue.cards,fake]},chosenHandDiscardDonors:{ground:fake.setCode}});assert.equal(p.cases.length,0);
});

test('Qi unique choices preserve captured played source and physical own hand in both seats', () => {
    const p=plan('ASH_226');
    for(const sourceSeat of [1,2]) for(const keep of ['old','new']) {
        const c=p.cases.find(c=>c.probe.sourceSeat===sourceSeat&&c.probe.scenario===`unique-keep-${keep}`),q=c.probe;
        assert.ok(c);assert.equal(q.keep,keep);assert.equal(q.oldIndex,4);
        assert.deepEqual(q.targets[q.oldIndex],{code:'ASH_226',owner:'self',arena:'ground'});
        assert.ok(q.handCodes.length>=2);assert.equal(new Set(q.handCodes).size,q.handCodes.length);
        assert.deepEqual(q.discardIndexes,[0]);assert.equal(q.handCodes[0],'JTL_075');assert.equal(q.targetIndex,1);
        for(const stage of ['unique-before-hand-picker','physical-distinct-hand-candidates','captured-incoming-damage-source','survivor-live-hand-power','current-live-unit-domain'])assert.ok(c.assertionStages.includes(stage));
    }
    assert.ok(!p.uncovered.some(u=>u.id.endsWith('unique-source-departure')));
});
