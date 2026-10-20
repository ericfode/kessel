"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const {loadCatalog} = require("../../compiler/load-catalog");
const {classifyUnitEntry, planUnitEntryProbes, isUnitEntryDonor, donorCodes} = require("../pipeline/unit-entry-probes");
const catalogue = loadCatalog(process.env.SWU_PROBE_TEST_CATALOGUE);
const copy = x => JSON.parse(JSON.stringify(x));
// The committed shard predates the closed callback lowering. Compiler tests own
// lowering against source; this dependency-free suite tests its explicit IR output.
function typedEntry() {
  const e = copy(catalogue.cards.find(e => e.setCode === "ASH_041"));
  e.abilities.card[0].effect.target = {kind:"ref",path:["unitEntryCard"]};
  e.abilities.card[0].contextTitle[1] = {kind:"ref",path:["unitEntryCard","title"]};
  return e;
}
test("complete typed unit-entry source constructs both seats and meaningful boundary cases", () => {
  const entry = typedEntry();
  assert.deepEqual(classifyUnitEntry(entry), {family:"unit-entry",event:"onUnitEntersPlay",target:"physical-event-card"});
  const plan = planUnitEntryProbes(entry,{catalogue});
  assert.equal(plan.cases.length,28);
  assert.equal(new Set(plan.cases.map(c=>c.id)).size,28);
  for(const scenario of ["unique-keep-incoming","unique-keep-old","dead-mandalorian-batch","repeated-dead-token-batch","phase-expiry","multiple-entrants-next","multiple-entrants-all"])
    assert.deepEqual(plan.cases.filter(c=>c.probe.scenario===scenario).map(c=>c.probe.sourceSeat),[1,2]);
  for(const c of plan.cases) {
    assert.equal(c.probe.family,"unit-entry");
    assert.ok(c.assertionStages.length);
    assert.deepEqual(c.coverageObligationIds,[plan.obligations.find(o=>o.id===c.coverageObligationIds[0]).id]);
    assert.equal(c.source.module,"../pipeline/unit-entry-probe-fixture.js");
  }
  assert.ok(plan.uncovered.some(o=>o.id.endsWith(":simultaneous-source-departure")));
  assert.ok(plan.uncovered.some(o=>o.id.endsWith(":foreign-owner-source")));
  assert.equal(plan.uncovered.filter(o=>o.reason.includes("runtime donor rejected")).length,12);
  assert.ok(!plan.cases.some(c=>c.setupCardIds.some(id=>["SEC_105","JTL_121","ASH_060"].includes(id))));
});
test("physical target, source controller, and live title are distinct closed bindings", () => {
  const mutations = [
    e=>e.abilities.card[0].effect.target.path=["source"],
    e=>e.abilities.card[0].effect.target.path=["target"],
    e=>e.abilities.card[0].contextTitle[1].path=["source","title"],
    e=>e.abilities.card[0].hooks[0].predicate.left.steps[1].key="owner",
    e=>e.abilities.card[0].hooks[0].event="onCardPlayed",
    e=>e.abilities.card[0].optional=true,
    e=>e.abilities.card[0].limit={max:1},
    e=>e.abilities.card[0].effect.power.value=2,
    e=>e.abilities.card[0].effect.hp.value=1,
    e=>e.abilities.card[0].effect.duration="attack",
    e=>e.abilities.card[0].targetResolver={type:"unit"},
    e=>e.abilities.card[0].hooks.push(copy(e.abilities.card[0].hooks[0])),
    e=>e.abilities.unit.push(copy(e.abilities.card[0])),
    e=>e.abilities.card[0].unreviewed=true,
  ];
  for(const mutate of mutations) {const e=typedEntry();mutate(e);assert.equal(classifyUnitEntry(e),null);assert.equal(planUnitEntryProbes(e,{catalogue}),null);}
});
test("full metadata and intrinsic lifecycle audit rejects partial or incompatible cards", () => {
  for(const mutate of [e=>e.baseClass="LeaderUnitCard",e=>e.sourceHash="0".repeat(64),e=>e.card.types.push("leader"),e=>e.card.keywords.push("Plot"),e=>e.keywordValues.support=1,e=>e.card.pilotText="Piloting",e=>e.card.unique=false,e=>e.card.hp=5,e=>e.card.text="Other text",e=>e.card.rules="Extra rules",e=>e.card.deployBox="Deploy",e=>e.piloting={},e=>e.card.extraLifecycle=true,e=>e.status="unsupported",e=>e.diagnostics=[{}]]) {
    const e=typedEntry();mutate(e);assert.equal(classifyUnitEntry(e),null);
  }
});
test("construction donors require their actual complete source, metadata and intrinsic lifecycles", () => {
  for(const id of donorCodes) {
    const original=catalogue.cards.find(e=>e.setCode===id);assert.equal(isUnitEntryDonor(original),true,id);
    for(const mutate of [e=>e.sourceHash="0".repeat(64),e=>e.card.keywords.push("Plot"),e=>e.abilities.card.push({trigger:"action"}),e=>e.card.text="Changed",e=>e.piloting={extra:true}]) {
      const e=copy(original);mutate(e);assert.equal(isUnitEntryDonor(e),false,id);
    }
  }
  const c=copy(catalogue);c.cards=c.cards.filter(e=>e.setCode!=="SHD_037");
  const plan=planUnitEntryProbes(typedEntry(),{catalogue:c});
  assert.equal(plan.cases.length,24);
  assert.equal(plan.uncovered.filter(o=>o.reason.includes("SHD_037")).length,4);
  assert.equal(planUnitEntryProbes(typedEntry()).cases.length,6); // self entry and unique choices have no external donors
});
