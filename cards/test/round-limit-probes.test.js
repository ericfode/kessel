'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyRoundLimit,planRoundLimitProbes}=require('../pipeline/round-limit-probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const ids=['ASH_047','ASH_032','ASH_160','HMW_062','SOR_115'];
const modes=['played-upgrade','nonlethal-damage','enemy-ground-attack','enemy-weakness-defeat','other-unique-defeat'];
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
test('five complete assembled source profiles retain exact per-registration round scope and native lifecycles',()=>{
    for(const [i,id]of ids.entries()){const e=entry(id),shape=classifyRoundLimit(e);assert.equal(shape.family,'round-limit');assert.equal(shape.mode,modes[i]);assert.deepEqual(e.diagnostics,[]);const limited=e.abilities.card.find(a=>a.limit);assert.deepEqual(limited.limit,{kind:'abilityLimit',factory:'perRound',args:[1],scope:'registration',sharedAcrossHooks:true});assert.ok(planRoundLimitProbes(e,{catalogue}).uncovered.some(o=>o.id.endsWith('source-review')));}
    assert.deepEqual(entry('ASH_160').keywordValues,{hidden:1});assert.deepEqual(entry('SOR_115').keywordValues,{ambush:1});assert.equal(entry('HMW_062').abilities.card.length,2);
});
test('limit cardinality/reset/scope, raw predicates, companion registrations and metadata changes fail closed',()=>{
    const changes=[e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='game/other.ts',e=>e.baseClass='LeaderUnitCard',e=>e.card.types.push('token'),e=>e.card.cost=0,e=>e.card.hp=1,e=>e.status='unsupported',e=>e.piloting={},e=>e.attachCondition={},e=>e.card.pilotText='Pilot',e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'whenPlayed'}),e=>e.abilities.card.find(a=>a.limit).limit.args=[2],e=>e.abilities.card.find(a=>a.limit).limit.factory='perPhase',e=>e.abilities.card.find(a=>a.limit).limit.scope='source',e=>e.abilities.card.find(a=>a.limit).limit.sharedAcrossHooks=false,e=>delete e.abilities.card.find(a=>a.limit).limit,e=>e.abilities.card.find(a=>a.limit).hooks[0].predicate={kind:'const',value:true},e=>e.abilities.card.find(a=>a.limit).collectiveTrigger=true,e=>e.extra=true];
    for(const id of ids)for(const change of changes){const e=entry(id);change(e);assert.equal(classifyRoundLimit(e),null);assert.equal(planRoundLimitProbes(e,{catalogue}),null);}
    for(const id of ['ASH_160','SOR_115'])for(const change of [e=>e.keywordValues={},e=>e.card.keywords=[]]){const e=entry(id);change(e);assert.equal(classifyRoundLimit(e),null);}
    const nuvo=entry('HMW_062');nuvo.abilities.card.shift();assert.equal(classifyRoundLimit(nuvo),null);
    for(const id of ['ASH_169','ASH_208','ASH_204','ASH_070'])assert.equal(classifyRoundLimit(entry(id)),null);
});
test('source profiles are not card-code exceptions; object insertion order is not semantic identity',()=>{
    const reorder=v=>Array.isArray(v)?v.map(reorder):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).reverse().map(([k,v])=>[k,reorder(v)])):v;
    for(const id of ids){const e=entry(id);e.setCode='CUSTOM_999';e.id=e.card.id='other';e.internalName=e.card.internalName='other';e.className='EquivalentSource';e.card.setId={set:'CUSTOM',number:999};e.card.setCodes=[e.card.setId];assert.ok(classifyRoundLimit(e));assert.deepEqual(planRoundLimitProbes(reorder(entry(id)),{catalogue:reorder(catalogue.cards)}),planRoundLimitProbes(entry(id),{catalogue}));}
});
test('all assembled classifiers agree with final five-family central routing and remain engine/parser independent',()=>{
    const {planCardProbes}=require('../pipeline/probes'),before=JSON.stringify(catalogue),selected=[],counts=[14,28,10,12,32];
    for(const e of catalogue.cards){const shape=classifyRoundLimit(e),direct=planRoundLimitProbes(e,{catalogue});assert.equal(Boolean(shape),Boolean(direct),e.setCode);if(!direct)continue;selected.push(e.setCode);const index=ids.indexOf(e.setCode);assert.notEqual(index,-1);assert.equal(shape.family,'round-limit');assert.equal(shape.mode,modes[index]);assert.equal(direct.cases.length,counts[index]);assert.ok(direct.cases.every(c=>c.probe.family==='round-limit'&&c.probe.mode===modes[index]));assert.deepEqual(planCardProbes(e,{catalogue}),direct,'central complete routing '+e.setCode);}
    assert.equal(catalogue.cards.length,2605);assert.deepEqual(selected.sort(),[...ids].sort());
    for(const id of ['ASH_047','ASH_032']){const changed=entry(id);changed.sourceHash='0'.repeat(64);assert.equal(classifyRoundLimit(changed),null);assert.equal(planRoundLimitProbes(changed,{catalogue}),null);const fallback=planCardProbes(changed,{catalogue});assert.equal(fallback.cases.length,0);assert.ok(fallback.uncovered.some(o=>o.path==='source'));}
    assert.equal(JSON.stringify(catalogue),before);assert.ok(!Object.keys(require.cache).some(p=>p.includes('/forceteki/')||p.includes('/typescript/')));
});

test('raw constructions bind both seats, exact fixture keys and all default/filler dependencies',()=>{
    const keys=['baseCode','cardId','enemyBaseCode','enemyLeaderCode','family','fillerCodes','leaderCode','mode','owner','resourceCount','scenario'];
    for(const id of ids){const p=planRoundLimitProbes(entry(id),{catalogue});for(const c of p.cases){assert.deepEqual(Object.keys(c.probe).sort(),c.probe.interaction==='at-limit-siblings'?[...keys,'interaction','choice'].sort():Object.hasOwn(c.probe,'interaction')?[...keys,'interaction'].sort():keys);assert.equal(c.name,'round-limit-'+id.toLowerCase().replaceAll('_','-')+'-owner-'+c.probe.owner+'-'+c.probe.scenario+(Object.hasOwn(c.probe,'choice')?'-remaining-'+c.probe.choice:''));assert.equal(c.probe.resourceCount,24);for(const d of ['SOR_014','SOR_023',...c.probe.fillerCodes])assert(c.setupCardIds.includes(d));assert.deepEqual(c.assertionStages,['construction','activation','observation','terminal']);assert.equal(p.cases.filter(x=>x.probe.scenario===c.probe.scenario&&x.probe.choice===c.probe.choice).length,2);}}
});
test('complete donor/native/default fingerprints reject metadata changes, duplicate rows and aliases',()=>{
    const gar=entry('ASH_047'),keeper=entry('ASH_032');assert(planRoundLimitProbes(gar,{catalogue}).cases.length>0);
    for(const mutate of [c=>{c.cards.find(x=>x.setCode==='IC27_052').card.extra=true;},c=>{c.cards.push(structuredClone(c.cards.find(x=>x.setCode==='IC27_052')));},c=>{c.cards.find(x=>x.setCode==='IC27_052').setCode='IC27_052_ALIAS';}]){const c=structuredClone(catalogue);mutate(c);assert.equal(planRoundLimitProbes(gar,{catalogue:c}).cases.length,0);assert.deepEqual(planRoundLimitProbes(keeper,{catalogue:c}),planRoundLimitProbes(keeper,{catalogue}));}
    for(const code of ['TOKEN_MANDALORIAN','TOKEN_SHIELD','SOR_014','SOR_023','ASH_061']){const c=structuredClone(catalogue);c.cards.find(x=>x.setCode===code).card.extra='unknown';assert.equal(planRoundLimitProbes(gar,{catalogue:c}).cases.length,0);}
});
test('finite verified table covers all five whole-card lifecycles and separates actual registration interactions',()=>{
    const counts=[14,28,10,12,32];for(const [i,id]of ids.entries())assert.equal(planRoundLimitProbes(entry(id),{catalogue}).cases.length,counts[i]);
    const has=(id,name)=>assert.equal(planRoundLimitProbes(entry(id),{catalogue}).cases.filter(c=>c.probe.scenario===name).length,2);
    has('ASH_047','reentry');has('ASH_047','opposing-copies');has('ASH_032','bases-none');has('ASH_032','multiple-copies-round-reset');has('ASH_160','paid-hidden-first-second-round-reset');has('ASH_160','already-ready-no-consume');has('HMW_062','paid-wp-first-second-death');has('HMW_062','captured-source-cinder-death');has('SOR_115','paid-ambush-first-second');has('SOR_115','nonunique-then-captured-mass-death');
    for(const code of ['TOKEN_WEAKNESS','ASH_151']){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===code).extra='unknown';assert(planRoundLimitProbes(entry('HMW_062'),{catalogue:c}).cases.length<12);}
    const changed=structuredClone(catalogue);changed.cards.find(e=>e.setCode==='SHD_037').card.keywords.push('unexpected');assert.equal(planRoundLimitProbes(entry('SOR_115'),{catalogue:changed}).cases.length,30);
});
test('extension preserves original62 non-name/id fields and adds exactly34 closed typed interactions',()=>{
    const crypto=require('node:crypto'),oldCounts=[14,16,8,10,14],hashes=['4e7db6e66db15df40d9bee49db714c02e5e204f965928b7b2808d490ea6082dd','62e6e938b7ab0467f06229985ff109ad9c09057a4bb5bc1e4ee29ec4040a92bb','a29bdda22d4afbef165ff2350698dcfda42cbb362f33f0a66681a7f97d09c45c','8c3ba6454bb810ad58310f785f2d53ab260e37d06375a42c78f344c48f4c3a25','1e18827fd0881c48ceca0b88ccb90c96b3ea6d74296acdba9c1540ef2b814c71'];
    const interactions=[];for(const [i,id]of ids.entries()){const p=planRoundLimitProbes(entry(id),{catalogue}),prefix=p.cases.slice(0,oldCounts[i]);assert.equal(crypto.createHash('sha256').update(JSON.stringify(prefix.map(c=>{const legacy=structuredClone(c),current=id.toLowerCase().replaceAll('_','-');legacy.name=legacy.name.replace('round-limit-'+current,'round-limit-'+id);legacy.id=legacy.id.replace('/round-limit-'+current,'/round-limit-'+id);return legacy;}))).digest('hex'),hashes[i]);assert(prefix.every(c=>!Object.hasOwn(c.probe,'interaction')));interactions.push(...p.cases.slice(oldCounts[i]));assert(p.uncovered.some(g=>g.id.endsWith('pending-old-registration-after-reentry')));}
    assert.equal(interactions.length,34);assert.deepEqual([...new Set(interactions.map(c=>c.probe.interaction))].sort(),['at-limit-siblings','distributed-damage-order','leader-unit','mixed-death-order','multiplicity','shared-declaration','zero-combat']);assert.equal(new Set(interactions.map(c=>c.name)).size,34);
});
test('new interaction guards bind staged leaders, companion focus sources and whole native/donor profiles without changing old62',()=>{
    const targets=[['SOR_014','ASH_032'],['ASH_153','SOR_115'],['SHD_037','ASH_032'],['IBH_063','ASH_032'],['ASH_072','ASH_160'],['HMW_062','ASH_032'],['TOKEN_WEAKNESS','SOR_115']];
    for(const [code,id]of targets){const c=structuredClone(catalogue);c.cards.find(e=>e.setCode===code).card.unreviewed=true;const before=planRoundLimitProbes(entry(id),{catalogue}),after=planRoundLimitProbes(entry(id),{catalogue:c});assert(after.cases.length<before.cases.length,code);assert(after.uncovered.some(x=>x.reason.includes('interaction source')),code);}
    const c=structuredClone(catalogue);c.cards.find(e=>e.setCode==='ASH_153').card.unreviewed=true;for(const id of ids){const old=planRoundLimitProbes(entry(id),{catalogue}).cases.filter(c=>!Object.hasOwn(c.probe,'interaction'));assert.deepEqual(planRoundLimitProbes(entry(id),{catalogue:c}).cases.filter(c=>!Object.hasOwn(c.probe,'interaction')),old);}
});

test('all96 automatic names and IDs satisfy the unchanged canonical reference grammar',()=>{
    const {definition}=require('../reference/library');let count=0;const seen=new Set();
    for(const id of ids)for(const c of planRoundLimitProbes(entry(id),{catalogue}).cases){
        assert.match(c.name,/^[a-z0-9-]+$/);assert.match(c.id,/^auto\/[A-Z][A-Z0-9]{2,3}_\d{3}\/[a-z0-9-]+$/);assert.equal(c.id.split('/')[2],c.name);assert(!seen.has(c.id));seen.add(c.id);
        const immutable={...c,source:{module:'../pipeline/probe-fixture.js'}};assert.equal(definition(c.id,{cases:[]},immutable),immutable);count++;
    }assert.equal(count,96);
});
test('distributed six appends exactly two source protocols while all86 prior case objects remain exact',()=>{
    const crypto=require('node:crypto'),counts=[14,22,10,12,28],hashes=['b2fe575504c53054757e172e3652e15457e6baed2e74f8b45c7be3cc9deb4eaf','19cc7eec1fc526ea5bf8d9d6e2c8e18db8a591ee2411ad58d2f5635d8fea7955','f779066ff42776d75d611b8d0d8b80e1e8c1ede9a53182b6df120329a77bec42','a16e9f2ec6fd0693202c09bd40a3ddb3097a03fc276dfb16eb54734bc8b2493a','747ed78767db830042ed0c762e1bcb84c56239c76f47012c4d369dd901dd966a'];
    for(const [i,id]of ids.entries()){const prefix=planRoundLimitProbes(entry(id),{catalogue}).cases.slice(0,counts[i]);assert.equal(crypto.createHash('sha256').update(JSON.stringify(prefix)).digest('hex'),hashes[i]);}
    const added=planRoundLimitProbes(entry('ASH_032'),{catalogue}).cases.filter(c=>c.probe.interaction==='distributed-damage-order');assert.equal(added.length,2);assert.deepEqual(added.map(c=>c.probe.owner),[0,1]);for(const c of added){assert.equal(c.probe.scenario,'distributed-six-keeper-order');for(const id of ['SOR_135','ASH_032','ASH_061','ASH_131'])assert(c.setupCardIds.includes(id));}
    const donor=entry('SOR_135');assert.equal(donor.card.cost,8);assert.deepEqual(donor.card.aspects,['aggression','villainy']);assert.deepEqual(donor.card.keywords,['overwhelm']);assert.deepEqual(donor.diagnostics,[]);
    for(const mutate of [e=>e.card.cost=7,e=>e.card.aspects=[],e=>e.card.keywords=[],e=>e.sourceHash='0'.repeat(64),e=>e.abilities.card.push({}),e=>e.card.unreviewed=true]){const c=structuredClone(catalogue);mutate(c.cards.find(e=>e.setCode==='SOR_135'));const p=planRoundLimitProbes(entry('ASH_032'),{catalogue:c});assert.equal(p.cases.length,22);assert(p.cases.every(c=>c.probe.interaction!=='distributed-damage-order'));}
});
test('at-limit siblings add only8 typed protocols preserving every original88 fullcaseobject',()=>{
    const crypto=require('node:crypto'),counts=[14,24,10,12,28],hashes=['b2fe575504c53054757e172e3652e15457e6baed2e74f8b45c7be3cc9deb4eaf','ba759f39f84302b990291d20a781c39e2c8651b0aa32902b793512d4630160f5','f779066ff42776d75d611b8d0d8b80e1e8c1ede9a53182b6df120329a77bec42','a16e9f2ec6fd0693202c09bd40a3ddb3097a03fc276dfb16eb54734bc8b2493a','747ed78767db830042ed0c762e1bcb84c56239c76f47012c4d369dd901dd966a'];
    const added=[];for(const [i,id]of ids.entries()){const cases=planRoundLimitProbes(entry(id),{catalogue}).cases;assert.equal(crypto.createHash('sha256').update(JSON.stringify(cases.slice(0,counts[i]))).digest('hex'),hashes[i]);added.push(...cases.filter(c=>c.probe.interaction==='at-limit-siblings'));}
    assert.equal(added.length,8);for(const id of ['ASH_032','SOR_115'])for(const owner of [0,1])for(const choice of ['next','all']){const c=added.find(c=>c.probe.cardId===id&&c.probe.owner===owner&&c.probe.choice===choice);assert(c);assert(c.name.endsWith('-remaining-'+choice));assert(c.setupCardIds.includes(id==='ASH_032'?'SOR_135':'SOR_043'));}
    assert.equal(entry('IBH_007').card.hp,5);assert.equal(entry('IBH_007').card.power,1);assert.equal(entry('IBH_007').card.text,'');assert.deepEqual(entry('IBH_007').card.keywords,[]);assert.deepEqual(entry('IBH_007').card.setCodes,[{set:'IBH',number:7},{set:'IBH',number:43},{set:'IBH',number:47}]);
    for(const mutate of [e=>e.card.hp=4,e=>e.card.text='new ability',e=>e.card.setCodes.pop(),e=>e.extra=true]){const c=structuredClone(catalogue);mutate(c.cards.find(e=>e.setCode==='IBH_007'));const p=planRoundLimitProbes(entry('ASH_032'),{catalogue:c});assert.equal(p.cases.length,24);assert(p.cases.every(c=>c.probe.interaction!=='at-limit-siblings'));}
});
