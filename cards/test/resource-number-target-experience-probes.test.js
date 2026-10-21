'use strict';const test=require('node:test'),assert=require('node:assert/strict'),m=require('../pipeline/resource-number-target-experience-probes'),catalog=require('../../compiler/load-catalog').loadCatalog(require('node:path').join(__dirname,'../../compiler/generated')),all=catalog.cards,entry=all.find(c=>c.setCode==='SEC_040');
test('complete source entry plans34 exact predeclared probe recipes',()=>{const p=m.planResourceNumberTargetExperienceProbes(entry,{catalogue:catalog});assert.equal(p.cases.length,34);assert.deepEqual(p.cases,m.RAW_CASES);assert.ok(p.cases.every(c=>c.kind==='probe'&&c.coverageObligationIds.length));assert.ok(p.cases.some(c=>c.probe.amount>6));});
test('whole-source and metadata mutations reject without partial family cases',()=>{for(const change of [e=>e.card.types=['unit'],e=>e.card.keywords=['support'],e=>e.extra=true,e=>e.diagnostics=['unknown'],e=>e.abilities.card[0].extra=true]){const e=structuredClone(entry);change(e);assert.equal(m.classifyResourceNumberTargetExperience(e),false);assert.equal(m.planResourceNumberTargetExperienceProbes(e,{catalogue:catalog}),null);}});
test('complete native Experience and donor profiles required, no metadata enrichment',()=>{for(const change of [a=>a.filter(e=>e.setCode!=='TOKEN_EXPERIENCE'),a=>{a.find(e=>e.setCode==='TOKEN_EXPERIENCE').card.hp=2;return a;},a=>[...a,a.find(e=>e.setCode==='ASH_061')]])assert.equal(m.planResourceNumberTargetExperienceProbes(entry,{catalogue:{...catalog,cards:change(structuredClone(all))}}).cases.length,0);});

test('strict ordered415 helper envelope and complete options API fail closed',()=>{assert.equal(m.planResourceNumberTargetExperienceProbes(entry,all).cases.length,0);for(const mutate of [c=>c.inputs.find(i=>i.file==='server/game/core/gameSteps/AbilityResolver.ts').sha256='0'.repeat(64),c=>c.inputs.push({file:'server/game/unknown.ts',sha256:'0'.repeat(64)}),c=>c.inputs.push(null),c=>c.inputs=c.inputs.filter(i=>!i.file.startsWith('server/')||i.file.startsWith('server/game/cards/')),c=>c.inputs.reverse(),c=>c.inputs.push(structuredClone(c.inputs.find(i=>i.file==='server/game/core/gameSteps/AbilityResolver.ts'))),c=>c.inputs.push({file:17,sha256:'0'.repeat(64)}),c=>c.inputs.push({file:'server/missing.ts'}),c=>c.inputs[0].unknown=true]){const c=structuredClone(catalog);mutate(c);assert.equal(m.planResourceNumberTargetExperienceProbes(entry,{catalogue:c}).cases.length,0);}});
test('leader-only both-seat standard cases retain number payment without XP recipient',()=>{const cases=m.RAW_CASES.filter(c=>c.probe.leaderOnly);assert.deepEqual(cases.map(c=>c.probe.owner),[0,1]);for(const c of cases){assert.equal(c.probe.amount,7);assert.equal(c.probe.readyAfterPlay,19);assert.equal(c.probe.noUnits,true);assert.ok(c.setupCardIds.includes('SOR_014'));assert.equal(c.kind,'probe');}});

// Exercise the public dispatcher and real coverage accounting, including evidence
// that must not be promoted to an oracle witness or a three-backend match.
test('public planner formal contract preserves finite evidence and explicit source gaps', () => {
    const catalogue = require('../../compiler/load-catalog').loadCatalog(require('node:path').join(__dirname, '../../compiler/generated'));
    const focus = catalogue.cards.find(e => e.setCode === 'SEC_040');
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
