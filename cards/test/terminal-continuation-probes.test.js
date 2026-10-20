'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const {planTerminalContinuationProbes:plan}=require('../pipeline/terminal-continuation-probes');
const focus=['ASH_179','JTL_104','LAW_033','ASH_260','JTL_063'];
const entry=(id,c=catalogue)=>structuredClone(c.cards.find(e=>e.setCode===id));
const all=c=>focus.map(id=>plan(entry(id,c),{catalogue:c}));
test('seven complete actual terminal continuation paths plan both activation seats',()=>{
 const plans=all(catalogue);assert.deepEqual(plans.map(p=>p.cases.length),[2,2,4,2,4]);assert.equal(plans.flatMap(p=>p.cases).length,14);
 const scenarios=['rancor-two-hits','raddus-death','hounds-choose','hounds-pass','watermonger-committed-discard','shuttle-trigger','shuttle-pass'];
 for(const scenario of scenarios)assert.deepEqual(plans.flatMap(p=>p.cases).filter(c=>c.probe.scenario===scenario).map(c=>c.probe.sourceSeat),[1,2]);
 for(const p of plans){assert.equal(p.newAdmissionCredit,0);assert.match(p.sourceAdmissionReceipt.reportSha256,/^[a-f0-9]{64}$/);for(const c of p.cases){assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));assert.equal(c.probe.family,'terminal-continuation');assert.equal(c.probe.resourceCount,24);assert.equal(c.probe.leaderCode,'SOR_014');assert.equal(c.focusCardIds[0],p.cardId);assert.ok(c.assertionStages.includes('latched-winner-public-prompt'));}}
});
test('mandatory damage and death continuations preserve exact distinct source assertions',()=>{
 const rancor=plan(entry('ASH_179'),{catalogue}).cases[0];assert.deepEqual(rancor.expected.targetDamageEvents,[5,5]);assert.equal(rancor.expected.optionalPass,false);assert.equal(rancor.expected.targetFinalDamage,10);assert.equal(rancor.probe.targets[0].attachments.length,10);assert.equal(rancor.expected.winner,'opponent');
 const raddus=plan(entry('JTL_104'),{catalogue}).cases[0];assert.equal(raddus.probe.targets[0].damage,3);assert.equal(raddus.probe.focusTargetIndex,0);assert.equal(raddus.expected.targetFinalDamage,13);assert.equal(raddus.expected.focusFinalZone,'discard');assert.equal(raddus.expected.optionalPass,false);assert.equal(raddus.probe.activationCode,'ASH_151');assert.equal(raddus.probe.sourceInPlay,undefined);
});
test('optional postwinner pass branches retain their physical targets and chooser',()=>{
 const hounds=plan(entry('LAW_033'),{catalogue});for(const c of hounds.cases){assert.equal(c.expected.optionalPass,true);assert.equal(c.probe.sourceInPlay,true);assert.equal(c.probe.sourceReady,true);assert.equal(c.expected.targetFinalZone,c.probe.passOptional?'groundArena':'discard');assert.equal(c.probe.enemyDamage,29);}
 const shuttle=plan(entry('JTL_063'),{catalogue});for(const c of shuttle.cases){assert.equal(c.probe.focusOwner,'opponent');assert.equal(c.probe.targets[c.probe.focusTargetIndex].code,'JTL_063');assert.equal(c.expected.terminalPromptOwner,'opponent');assert.equal(c.expected.winner,'self');assert.deepEqual(c.expected.exactButtons,[{arg:'trigger',label:'Trigger'},{arg:'pass',label:'Pass'}]);assert.equal(c.expected.focusOwnerHandDelta,c.probe.passOptional?0:1);}
});
test('prewinner optional draw commits a mandatory postwinner discard',()=>{
 const water=plan(entry('ASH_260'),{catalogue});for(const c of water.cases){assert.equal(c.probe.emptyDeck,true);assert.equal(c.probe.acceptBeforeWinner,true);assert.equal(c.expected.mandatoryHandDiscard,1);assert.equal(c.expected.optionalPass,false);assert.equal(c.expected.ownBaseFinalDamage,30);}
});
test('current Cinder setup dependency stays visible without inventing an admission',()=>{
 for(const id of ['JTL_104','JTL_063']){const p=plan(entry(id),{catalogue}),receipt=p.sourceAdmissionReceipt.sources.ASH_151;assert.equal(receipt.valid,true);assert.ok(p.cases.every(c=>c.setupCardIds.includes('ASH_151')));assert.equal(p.runtimeReady,receipt.runtimeAdmitted);if(!receipt.runtimeAdmitted)assert.ok(p.uncovered.some(o=>o.id.endsWith('cinder-setup-admission')));}
});
test('complete source, data, metadata and nested ability mutations fail closed',()=>{
 for(const id of focus)for(const mutate of [e=>e.sourceHash='0'.repeat(64),e=>e.dataHash='1'.repeat(64),e=>e.card.text+=' changed',e=>e.abilities.card.push({trigger:'whenPlayed',effect:{kind:'noAction'}}),e=>e.card.types.push('leader'),e=>e.unreviewed={x:1}]){const e=entry(id);mutate(e);const p=plan(e,{catalogue});assert.equal(p.cases.length,0,id);assert.ok(p.uncovered.some(o=>o.id.endsWith('source-or-admission')));}
 for(const id of ['ASH_151','LOF_256','TOKEN_EXPERIENCE','SOR_014','ASH_019']){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===id).card.power=99;const p=plan(entry('JTL_104',c),{catalogue:c});assert.equal(p.cases.length,0,id);}
});
test('recursive property order is semantic while arrays and every source value remain significant',()=>{
 const reorder=x=>Array.isArray(x)?x.map(reorder):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).reverse().map(k=>[k,reorder(x[k])])):x;
 const before=JSON.stringify(catalogue),original=all(catalogue),reordered=all(reorder(catalogue));assert.deepEqual(reordered,original);assert.equal(JSON.stringify(catalogue),before);
 const c=structuredClone(catalogue),e=c.cards.find(e=>e.setCode==='JTL_104');e.card.aspects.reverse();assert.equal(plan(entry('JTL_104',c),{catalogue:c}).cases.length,0);
});
test('unreviewed aliases and unrelated cards do not acquire terminal fixture coverage',()=>{const e=entry('ASH_179');e.setCode='CUSTOM_001';assert.equal(plan(e,{catalogue}),null);assert.equal(plan(entry('ASH_061'),{catalogue}),null);});

test('paid Glory then Cinder expiry adds exactly two source-bound regroup continuations',()=>{
 const p=plan(entry('ASH_151'),{catalogue});assert.equal(p.cases.length,2);assert.deepEqual(p.cases.map(c=>c.probe.sourceSeat),[1,2]);
 for(const c of p.cases){assert.equal(c.probe.scenario,'expiry-cavern-regroup');assert.equal(c.probe.activationCode,'ASH_151');assert.equal(c.probe.preparationCode,'SHD_130');assert.equal(c.probe.targets[0].code,'LAW_189');assert.equal(c.probe.targets[0].attachments.length,1);assert.equal(c.probe.targets[0].attachments[0].token,'experience');assert.equal(c.probe.targets[0].exhausted,true);assert.equal(c.probe.ownDamage,0);assert.equal(c.probe.enemyDamage,28);assert.equal(c.probe.emptyDecks,true);assert.equal(c.probe.initiallyExhaustResource,true);assert.equal(c.expected.targetDamageBeforeExpiry,5);assert.equal(c.expected.targetHpBeforeExpiry,6);assert.equal(c.expected.prewinnerDeathPhase,null);assert.equal(c.expected.prewinnerDeathPromptTitle,'Deal 2 damage to a base');assert.equal(c.expected.promptTitle,'Select between 0 and 1 cards to resource');assert.equal(c.expected.winner,'self');assert.equal(c.expected.ownBaseFinalDamage,11);assert.equal(c.expected.enemyBaseFinalDamage,30);assert.deepEqual(c.expected.postWinnerResourceSeats,['p1','p2']);assert.deepEqual(c.expected.regroupDrawAmounts,[2,2]);assert.equal(c.expected.roundDelta,1);assert.ok(c.setupCardIds.includes('SHD_130'));}
 const moment=entry('SHD_130');assert.deepEqual(moment.abilities.card[0].effect,{kind:'modifyStats',target:'chosen',duration:'phase',power:{kind:'const',value:4},hp:{kind:'const',value:4}});assert.equal(p.sourceAdmissionReceipt.sources.SHD_130.runtimeAdmitted,true);
});
test('six ordinary phase regressions use an existing base and award no source ability witness',()=>{
 const p=plan(entry('ASH_019'),{catalogue});assert.equal(p.cases.length,6);for(const initiative of [1,2])for(const losing of ['p1','p2','both']){const c=p.cases.find(c=>c.probe.initiativeSeat===initiative&&c.id.includes('fatigue-'+losing+'-seat-'));assert.ok(c);assert.deepEqual(c.focusCardIds,['ASH_019']);assert.ok(c.behaviorIds.includes('regroup-runtime'));assert.ok(c.assertionStages.includes('actual-phase-pass'));assert.ok(!c.assertionStages.includes('real-paid-play-or-attack'));assert.equal(c.probe.behaviorOnly,true);assert.equal(c.probe.constructionCode,'SOR_095');assert.equal(c.probe.activationCode,null);assert.equal(c.probe.scenario,'ordinary-regroup-fatigue');assert.equal(c.probe.sourceSeat,initiative);assert.equal(c.probe.resourceCount,12);assert.equal(c.probe.emptyDecks,true);assert.equal(c.probe.initiallyExhaustLeaders,true);assert.deepEqual(c.probe.initialBaseDamageAbsolute,losing==='both'?[25,25]:losing==='p1'?[25,0]:[0,25]);assert.equal(c.expected.winnerAbsolute,losing==='both'?'draw':losing==='p1'?'p2':'p1');assert.deepEqual(c.expected.baseFinalDamageAbsolute,losing==='both'?[30,30]:losing==='p1'?[30,6]:[6,30]);assert.deepEqual(c.expected.resourceCountDelta,[1,1]);assert.deepEqual(c.expected.handCountDelta,[-1,-1]);assert.equal(c.expected.resourceSelectionPerSeat,1);assert.equal(c.expected.allResourcesReady,true);assert.equal(c.expected.allUnitsReady,true);assert.equal(c.expected.allLeadersReady,true);assert.equal(c.expected.finalActor,null);assert.deepEqual(c.expected.phaseSequence,['action','regroup','action']);}assert.equal(p.newAdmissionCredit,0);
});
test('new Glory and ordinary donor variants reject full-definition drift',()=>{
 for(const mutate of [e=>e.sourceHash='a'.repeat(64),e=>e.dataHash='b'.repeat(64),e=>e.card.text+=' extra',e=>e.abilities.card[0].effect.hp.value=5]){const c=structuredClone(catalogue);mutate(c.cards.find(e=>e.setCode==='SHD_130'));assert.equal(plan(entry('ASH_151',c),{catalogue:c}).cases.length,0);assert.equal(all(c).flatMap(p=>p.cases).length,14);}
 const c=structuredClone(catalogue);c.cards.find(e=>e.setCode==='SOR_095').card.power=99;assert.equal(plan(entry('ASH_019',c),{catalogue:c}).cases.length,0);
 assert.equal(plan(entry('SHD_208'),{catalogue}),null,'Unadmitted Final Showdown remains engine-only');
});
test('all22 supplementary cases preserve all original14 full plan objects',()=>{
 const oldPlans=all(catalogue),regroup=['ASH_151','ASH_019'].map(id=>plan(entry(id),{catalogue}));assert.equal([...oldPlans,...regroup].flatMap(p=>p.cases).length,22);for(const p of regroup){assert.equal(p.newAdmissionCredit,0);assert.equal(p.runtimeReady,true);}
 const reverse=x=>Array.isArray(x)?x.map(reverse):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).reverse().map(k=>[k,reverse(x[k])])):x;const reordered=reverse(catalogue);for(const id of ['ASH_151','ASH_019'])assert.deepEqual(plan(entry(id,reordered),{catalogue:reordered}),plan(entry(id),{catalogue}));
});
