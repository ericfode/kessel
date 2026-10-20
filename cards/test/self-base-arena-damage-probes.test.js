'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const p = require('../pipeline/self-base-arena-damage-probes');
// Committed data only: this test must never load or invoke the source compiler.
const directory = path.resolve(__dirname, '../../compiler/generated');
const cards = JSON.parse(fs.readFileSync(path.join(directory, 'cards.json'))).cards;
const diagnostics = new Map(JSON.parse(fs.readFileSync(path.join(directory, 'diagnostics.json'))).cards.map(e => [e.setCode, e.diagnostics]));
const catalogue = { cards: cards.map(e => ({ ...e, diagnostics: diagnostics.get(e.setCode) ?? [] })) };
const get = id => structuredClone(catalogue.cards.find(e => e.setCode === id));
const plan = () => p.planSelfBaseArenaDamageProbes(get('ASH_151'), { catalogue });
test('complete literal-five self-base then global-unit source classifies; aliases do not', () => {
    assert.equal(p.classifySelfBaseArenaDamage(get('ASH_151')).amount, 5);
    assert.equal(p.classifySelfBaseArenaDamage(null), null);
    for (const change of [e => e.setCode = 'ASH_999', e => e.sourceHash = 'f'.repeat(64), e => e.card.text += ' Draw a card.', e => e.dataHash = 'f'.repeat(64),
        e => e.baseClass = 'NonLeaderUnitCard', e => e.card.types = ['event','leader'], e => e.abilities.unit.push({}), e => e.abilities.card.push({}),
        e => e.keywordValues.support = 1, e => e.card.keywords.push('plot'), e => e.card.cost = 5, e => e.piloting = null, e => e.stateWatchers = []]) {
        const e = get('ASH_151'); change(e); assert.equal(p.classifySelfBaseArenaDamage(e), null);
    }
});
test('all unreviewed headers, continuations, targets, amounts and order fields reject', () => {
    for (const change of [a => a.optional = true, a => a.costs.push({kind:'exhaustSelf'}), a => a.targetResolver = {}, a => a.condition = {},
        a => a.ifYouDo = a.then, a => a.then.then = {}, a => a.effect.target = 'enemyBase', a => a.effect.target = ['ownBase','enemyBase'],
        a => a.effect.amount.value = 0, a => a.effect.amount.value = 4, a => a.then.effect.amount.value = 4,
        a => a.then.effect.amount = {kind:'ref',path:['source','cost']}, a => a.then.effect.target.receiver.path = ['player'],
        a => a.then.effect.target.predicate.value = false, a => a.then.effect.target.controllerOrder = 'seatOrder']) {
        const e = get('ASH_151'); change(e.abilities.card[0]); assert.equal(p.classifySelfBaseArenaDamage(e), null);
    }
});
test('both seats have paid-play, global-order, replacement and terminal obligations', () => {
    const result = plan(); assert.equal(result.cases.length, 16); assert.deepEqual(result.uncovered, []);
    assert.equal(new Set(result.cases.map(c => c.id)).size, 16);
    for (const seat of [1,2]) {
        const cases = result.cases.filter(c => c.probe.sourceSeat === seat); assert.equal(cases.length, 8);
        const names = cases.map(c => c.probe.scenario);
        for (const name of ['empty','mixed-shields-global-order','multi-owner-shield-menus','aura-death','own-base-lethal-death-heal','winner-latched-then-mandatory-choice','enemy-base-lethal-death-trigger','death-draw-order']) assert.ok(names.includes(name));
        assert.ok(cases.find(c => c.probe.scenario === 'multi-owner-shield-menus').probe.targets.every(t => t.attachments.length === 2));
        for (const c of cases) { assert.equal(c.probe.resourceCount, 24); assert.ok(c.coverageObligationIds.length >= 2); assert.deepEqual(c.focusCardIds, ['ASH_151']); }
    }
});
test('setup donors are admitted complete semantic profiles with separate byte hashes', () => {
    const admitted = new Set(JSON.parse(fs.readFileSync(path.join(directory, 'runtime-admission.json'))).runtimeAdmittedCards.map(e => e.setCode));
    for (const c of plan().cases) for (const binding of c.donorProvenance) {
        assert.ok(admitted.has(binding.cardId)); const e = get(binding.cardId);
        assert.equal(p.semanticFingerprint(e), binding.semanticFingerprint); assert.equal(e.sourceHash, binding.sourceByteHash);
        assert.ok(p.matchDonor(e, binding.role));
        const sourceChanged = structuredClone(e); sourceChanged.sourceHash = 'e'.repeat(64);
        assert.equal(p.semanticFingerprint(sourceChanged), binding.semanticFingerprint); assert.equal(p.matchDonor(sourceChanged, binding.role), false);
        const changed = structuredClone(e); changed.card.text += ' Extra behavior.'; assert.equal(p.matchDonor(changed, binding.role), false);
    }
});
test('missing or unadmitted donor remains an explicit uncovered obligation', () => {
    const missing = p.planSelfBaseArenaDamageProbes(get('ASH_151'), { catalogue: {cards: catalogue.cards.filter(e => e.setCode !== 'LAW_189')} });
    assert.equal(missing.cases.length, 0); assert.match(missing.uncovered[0].reason, /deathDamage/);
    const unadmitted = p.planSelfBaseArenaDamageProbes(get('ASH_151'), {catalogue, admittedCardIds: []});
    assert.equal(unadmitted.cases.length, 0); assert.ok(unadmitted.uncovered.length);
});
test('generated definitions preserve full content in reference-library round trip', () => {
    const library = require('../reference/library');
    for (const c of plan().cases) assert.deepEqual(library.definition(c.id, {cases: []}, c), c);
});
