'use strict';const test=require('node:test'),assert=require('node:assert/strict'),m=require('../pipeline/generated-token-continuation-probes'),C=require('../../compiler/load-catalog').loadCatalog(require('node:path').join(__dirname,'../../compiler/generated')),entry=C.cards.find(x=>x.setCode==='ASH_257');
const helpers=c=>c.inputs.filter(r=>r.file.startsWith('server/')&&!r.file.startsWith('server/game/cards/'));const mutations=[c=>{c.inputs=c.inputs.filter(r=>r!==helpers(c)[0]);},c=>c.inputs.push({file:'server/not-reviewed.ts',sha256:'a'.repeat(64)}),c=>c.inputs.push(structuredClone(helpers(c)[0])),c=>{const a=helpers(c),i=c.inputs.indexOf(a[0]),j=c.inputs.indexOf(a[1]);[c.inputs[i],c.inputs[j]]=[c.inputs[j],c.inputs[i]];},c=>{helpers(c)[0].sha256='b'.repeat(64);},c=>{helpers(c)[0].extra=true;},c=>{helpers(c)[0].file=null;},c=>c.inputs.push(null)];
test('full assembled catalogue emits26 exact source-only probes with obligations',()=>{assert.equal(m.classifyGeneratedTokenContinuation(entry,C),true);assert.deepEqual(m.planGeneratedTokenContinuationProbes(entry,C).cases,m.RAW_CASES);assert.equal(m.RAW_CASES.length,26);assert.equal(m.HELPER_INPUTS.length,415);assert.ok(m.RAW_CASES.every(c=>c.kind==='probe'&&c.coverageObligationIds.length));});
test('all415 helper rows must be present exact ordered and shaped before cases',()=>{for(const f of mutations){const c=structuredClone(C);f(c);assert.throws(()=>m.validateCatalogue(c));assert.equal(m.planGeneratedTokenContinuationProbes(entry,c).cases.length,0);}assert.equal(m.planGeneratedTokenContinuationProbes(entry,C.cards).cases.length,0);assert.equal(m.planGeneratedTokenContinuationProbes(entry,{cards:C.cards}).cases.length,0);});
test('complete focus/source/intrinsic/dependency guards reject malformed and unknown profiles',()=>{for(const f of [e=>e.card.types=['unit'],e=>e.card.keywords=['support'],e=>e.extra=true,e=>e.diagnostics=['unknown'],e=>e.abilities.card[0].extra=true]){const e=structuredClone(entry);f(e);assert.equal(m.planGeneratedTokenContinuationProbes(e,C).cases.length,0);}for(const id of ['TOKEN_MANDALORIAN','TOKEN_ADVANTAGE','TOKEN_SHIELD','HMW_128','SHD_169']){const c=structuredClone(C);c.cards=c.cards.filter(x=>x.setCode!==id);assert.equal(m.planGeneratedTokenContinuationProbes(entry,c).cases.length,0);}for(const id of ['TOKEN_MANDALORIAN','TOKEN_ADVANTAGE','TOKEN_SHIELD']){const c=structuredClone(C);c.cards.find(x=>x.setCode===id).card.hp=99;assert.equal(m.planGeneratedTokenContinuationProbes(entry,c).cases.length,0);}});

// Exercise the public dispatcher and real coverage accounting, including evidence
// that must not be promoted to an oracle witness or a three-backend match.
test('public planner formal contract preserves finite evidence and explicit source gaps', () => {
    const catalogue = require('../../compiler/load-catalog').loadCatalog(require('node:path').join(__dirname, '../../compiler/generated'));
    const focus = catalogue.cards.find(e => e.setCode === 'ASH_257');
    const plan = require('../pipeline/probes').planCardProbes(focus, { catalogue });
    const { coverage } = require('../pipeline/coverage');
    assert.equal(plan.schema, 'swu-card-probes/v1');
    assert.equal(plan.sourceHash, focus.sourceHash);
    assert.deepEqual(plan.cases, m.RAW_CASES);
    const declared = [...new Set(plan.cases.flatMap(c => c.coverageObligationIds))];
    assert.equal(new Set(plan.obligations.map(o => o.id)).size, plan.obligations.length);
    for (const id of declared) assert.equal(plan.obligations.filter(o => o.id === id).length, 1);
    for (const o of plan.obligations) assert.ok(o.path && o.dimension);
    assert.ok(plan.uncovered.length >= 5);
    const empty = coverage([plan])[0];
    assert.equal(empty.totals.generated, declared.length);
    assert.equal(empty.totals.matched, 0);
    for (const gap of plan.uncovered) {
        assert.ok(gap.id && gap.path && gap.reason);
        assert.equal(empty.obligations.find(o => o.id === gap.id).status, 'uncovered');
    }
    const c = plan.cases[0], id = c.coverageObligationIds[0], hash = 'a'.repeat(64);
    const observation = { caseId: c.id, coverageObligationIds: [id], traceSha256: hash, poolSha256: hash, provenance: { sourceCatalogueSha256: hash } };
    const status = (observations, comparisons = []) => coverage([plan], observations, comparisons)[0].obligations.find(o => o.id === id).status;
    assert.equal(status([{ ...observation, coverageObligationIds: [] }]), 'generated');
    assert.equal(status([{ ...observation, coverageObligationIds: [id + ':mutated'] }]), 'generated');
    assert.equal(status([{ ...observation, caseId: 'unplanned-case' }]), 'generated');
    assert.equal(status([observation]), 'oracle-exercised');
    const comparisons = ['native-auto', 'native-interpreter', 'wasm-auto'].map(runtime => ({caseId: c.id, runtime, matched: true, traceSha256: hash, poolSha256: hash, catalogueSha256: hash}));
    assert.equal(status([observation], comparisons.slice(1)), 'oracle-exercised');
    assert.equal(status([observation], comparisons.map(r => ({ ...r, catalogueSha256: 'b'.repeat(64) }))), 'oracle-exercised');
    assert.equal(status([observation], comparisons), 'matched');
    assert.equal(coverage([plan], [observation], comparisons)[0].totals.matched, 1);
    const broken = structuredClone(catalogue); broken.inputs.push(null);
    const rejected = require('../pipeline/probes').planCardProbes(focus, {catalogue: broken});
    assert.equal(rejected.cases.length, 0);
    assert.equal(coverage([rejected])[0].totals.matched, 0);
    assert.ok(rejected.uncovered.length);
});
