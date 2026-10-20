'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const catalogue = require('../../compiler/load-catalog').loadCatalog();
const { classifyDistributedDamage, planDistributedDamageProbes, DEFAULT_DONORS } = require('../pipeline/distributed-damage-probes');
const entry = id => structuredClone(catalogue.cards.find(e => e.setCode === id));
test('both complete actual card profiles plan both seats and allocation boundaries', () => {
    for (const [id, trigger, count] of [['SOR_135','whenPlayed',32],['LOF_213','whenDefeated',28]]) {
        const p = planDistributedDamageProbes(entry(id), { catalogue });
        assert.equal(p.cases.length, count);
        for (const sourceSeat of [1,2]) for (const scenario of ['no-target','single','low-hp','split-2-4','split-1-2-3','six-ones','shield','multiple-shields','all-shielded','split-lethal']) {
            const c = p.cases.find(c => c.probe.sourceSeat === sourceSeat && c.probe.scenario === scenario);
            assert.ok(c); assert.equal(c.probe.trigger, trigger);
            assert.equal(c.probe.assignments.reduce((sum, a) => sum + a.amount, 0), scenario === 'no-target' ? 0 : 6);
            for (const a of c.probe.assignments) assert.equal(c.probe.targets[a.index].owner, 'opponent');
            assert.equal(new Set(c.probe.assignments.map(a => a.index)).size, c.probe.assignments.length);
            assert.ok(c.coverageObligationIds.every(id => p.obligations.some(o => o.id === id)));
            if (trigger === 'whenDefeated') { assert.equal(c.probe.activationCode, 'SHD_079'); assert.equal(c.probe.sourceInPlay, true); }
        }
        assert.match(p.donorAdmissionReceipt.reportHash, /^[a-f0-9]{64}$/);
        assert.equal(p.donorAdmissionReceipt.donors.shield.implementation, 'nativeAdapter');
        for (const dimension of ['meta-continuations','ownership-controller-boundary','additional-death-listeners','damage-increase-and-restrictions','enumeration-overflow']) assert.ok(p.uncovered.some(o => o.id.endsWith(dimension)));
    }
});
test('classifier follows source structure rather than card identity', () => {
    for (const id of ['SOR_135','LOF_213']) { const e = entry(id); e.setCode = 'CUSTOM_001'; e.className = 'Other'; e.card.title = 'Other'; assert.ok(classifyDistributedDamage(e)); }
});
test('unknown companion semantics and distribution variants remain rejected', () => {
    const mutations = [e => e.other = true, e => e.abilities.extra = [], e => e.abilities.card.push({}), e => e.abilities.unit.push({}), e => e.card.types.push('leader'), e => e.card.pilotText = 'Pilot', e => e.attachCondition = {}, e => e.card.deployBox = 'Deploy', e => e.card.keywords.push('shielded'), e => e.keywordValues.extra = 1, e => e.sourceHash = '',
        e => e.abilities.card[0].optional = false, e => e.abilities.card[0].then = {}, e => e.abilities.card[0].costs.push({ kind: 'exhaustSelf' }), e => e.abilities.card[0].effect.maxTargets = 6, e => e.abilities.card[0].effect.source = 'self', e => e.abilities.card[0].effect.amount.value = 5, e => e.abilities.card[0].effect.amount.extra = 0, e => e.abilities.card[0].effect.canChooseNoTargets = true, e => e.abilities.card[0].effect.canDistributeLess = true, e => e.abilities.card[0].effect.targetResolver.owner = 'any', e => e.abilities.card[0].effect.targetResolver.zone = 'groundArena', e => e.abilities.card[0].effect.targetResolver.extra = false];
    for (const id of ['SOR_135','LOF_213']) for (const mutate of mutations) { const e = entry(id); mutate(e); assert.equal(classifyDistributedDamage(e), null); }
    const legacy = entry('LOF_213'); legacy.card.keywords = ['overwhelm']; legacy.keywordValues = { overwhelm: 1 }; assert.equal(classifyDistributedDamage(legacy), null);
});
test('donor mutations and independently unadmitted aliases fail closed', () => {
    for (const [role, id] of Object.entries(DEFAULT_DONORS)) {
        const cards = structuredClone(catalogue.cards); const donor = cards.find(e => e.setCode === id); donor.card.keywords.push('hidden');
        const p = planDistributedDamageProbes(entry('SOR_135'), { catalogue: { cards } }); assert.equal(p.cases.length, 0, role); assert.ok(p.uncovered.some(o => o.id.endsWith('donor-admission')));
    }
    const fake = entry('HMW_093'); fake.setCode = 'FAKE_001';
    const p = planDistributedDamageProbes(entry('LOF_213'), { catalogue: { cards: [...catalogue.cards, fake] }, distributedDamageDonors: { ground: fake.setCode } }); assert.equal(p.cases.length, 0);
});
test('low HP and shielded targets retain full six assignment and exact token requests', () => {
    const p = planDistributedDamageProbes(entry('SOR_135'), { catalogue });
    const low = p.cases.find(c => c.probe.scenario === 'low-hp').probe;
    assert.equal(entry(low.targets[0].code).card.hp - low.targets[0].damage, 1); assert.deepEqual(low.assignments, [{ index: 0, amount: 6 }]);
    const six = p.cases.find(c => c.probe.scenario === 'six-ones').probe; assert.equal(six.assignments.length, 6);
    const multi = p.cases.find(c => c.probe.scenario === 'multiple-shields').probe; assert.equal(multi.targets[0].attachments.length, 2); assert.equal(multi.targets[1].attachments.length, 1);
});

test('additional complete admitted interaction donors plan leader defeat, printed draw and HP aura departure', () => {
    for (const id of ['SOR_135','LOF_213']) {
        const p = planDistributedDamageProbes(entry(id), { catalogue });
        for (const sourceSeat of [1,2]) {
            const leader = p.cases.find(c => c.probe.sourceSeat === sourceSeat && c.probe.scenario === 'deployed-leader').probe;
            assert.equal(leader.targets[0].code, 'SEC_005'); assert.equal(leader.targets[0].deployedLeader, true);
            assert.equal(leader.targets[0].damage + leader.assignments[0].amount, 8);
            const draw = p.cases.find(c => c.probe.sourceSeat === sourceSeat && c.probe.scenario === 'death-draw').probe;
            assert.equal(draw.targets[draw.deathDrawIndex].code, 'ASH_116'); assert.equal(draw.assignments[0].amount, 2);
            const aura = p.cases.find(c => c.probe.sourceSeat === sourceSeat && c.probe.scenario === 'hp-aura-departure').probe;
            assert.equal(aura.targets[aura.auraIndex].code, 'JTL_085'); assert.equal(aura.targets[aura.auraRecipientIndex].damage, 5);
        }
        for (const role of ['leader','deathDraw','hpAura']) assert.equal(p.interactionDonorAdmissionReceipt.donors[role].valid, true);
    }
});
test('mutated interaction donor loses only its new constructions', () => {
    for (const [code,scenario] of [['SEC_005','deployed-leader'],['ASH_116','death-draw'],['JTL_085','hp-aura-departure']]) {
        const cards = structuredClone(catalogue.cards); cards.find(e => e.setCode === code).card.text += ' Changed';
        const p = planDistributedDamageProbes(entry('SOR_135'), { catalogue: { cards } });
        assert.equal(p.cases.length, 30); assert.ok(!p.cases.some(c => c.probe.scenario === scenario));
        assert.ok(p.cases.some(c => c.probe.scenario === 'single'));
    }
});

test('foreign physical ownership remains explicitly uncovered and never enters standard cases', () => {
    const p = planDistributedDamageProbes(entry('LOF_213'), { catalogue });
    assert.equal(p.cases.length, 28);
    assert.ok(p.cases.every(c => c.probe.scenario !== 'foreign-control' && !c.probe.foreignControl));
    for (const seat of [1,2]) {
        const u = p.uncovered.find(u => u.id === `LOF_213:distributed-damage:foreign-control-seat-${seat}`);
        assert.ok(u); assert.match(u.reason, /engine-only diagnostic/); assert.match(u.reason, /no native\/Wasm coverage/);
    }
});

test('recursive object key reordering preserves complete semantic donor identity', () => {
    const reorder = x => Array.isArray(x) ? x.map(reorder) : x && typeof x === 'object' ? Object.fromEntries(Object.keys(x).reverse().map(k => [k,reorder(x[k])])) : x;
    const reordered = reorder(catalogue);
    for (const id of ['SOR_135','LOF_213']) {
        const baseline = planDistributedDamageProbes(entry(id), { catalogue });
        const actual = planDistributedDamageProbes(reordered.cards.find(e => e.setCode === id), { catalogue: reordered });
        assert.deepEqual(actual.cases, baseline.cases);
        assert.deepEqual(actual.interactionDonorAdmissionReceipt, baseline.interactionDonorAdmissionReceipt);
        for (const donor of Object.values(actual.interactionDonorAdmissionReceipt.donors)) {
            assert.match(donor.entrySemanticSha256, /^[a-f0-9]{64}$/); assert.equal(donor.valid, true); assert.equal(donor.entryHash, undefined);
        }
    }
});
test('semantic donor fingerprints retain source/data identity and nested array values', () => {
    for (const [code,scenario] of [['SEC_005','deployed-leader'],['ASH_116','death-draw'],['JTL_085','hp-aura-departure']]) for (const mutate of [
        e => e.sourceHash = '0'.repeat(64), e => e.dataHash = '0'.repeat(64), e => e.card.aspects.reverse(), e => e.abilities.card.length ? e.abilities.card[0].effect.extra = true : e.abilities.leader[0].effect.extra = true,
    ]) {
        const cards = structuredClone(catalogue.cards); const donor = cards.find(e => e.setCode === code);
        const before = JSON.stringify(donor); mutate(donor); if (before === JSON.stringify(donor)) continue;
        const actual = planDistributedDamageProbes(entry('SOR_135'), { catalogue: { cards } });
        assert.ok(!actual.cases.some(c => c.probe.scenario === scenario), code);
        assert.equal(actual.cases.length, 30);
    }
});
