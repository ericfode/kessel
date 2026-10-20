'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const {classifyCollectiveTrigger,planCollectiveTriggerProbes}=require('../pipeline/collective-trigger-probes');
const catalogue=require('../../compiler/load-catalog').loadCatalog();
const entry=id=>structuredClone(catalogue.cards.find(e=>e.setCode===id));
const fillers=['ASH_061','ASH_131','ASH_190','ASH_201','ASH_242','ASH_261','HMW_082','HMW_093','HMW_116','HMW_128','HMW_135','HMW_155','HMW_163','HMW_174','HMW_178','HMW_219','HMW_258'];
test('assembled complete Axe and Shielded Sabine profiles construct exact agreed both-seat scenarios',()=>{
    for(const [id,mode,n]of [['ASH_169','draw',14],['ASH_208','attachment',28]]){const e=entry(id),p=planCollectiveTriggerProbes(e,{catalogue});assert.equal(p.cases.length,n);assert.equal(p.sourceHash,e.sourceHash);assert.equal(new Set(p.cases.map(c=>c.id)).size,n);assert.ok(p.cases.every(c=>c.probe.mode===mode));for(const owner of [0,1])assert.equal(p.cases.filter(c=>c.probe.owner===owner).length,n/2);for(const c of p.cases){assert.equal(c.probe.family,'collective-trigger');assert.equal(c.probe.resourceCount,24);assert.equal(c.probe.leaderCode,'SOR_014');assert.equal(c.probe.enemyLeaderCode,'SOR_014');assert.equal(c.probe.baseCode,'SOR_023');assert.equal(c.probe.enemyBaseCode,'SOR_023');assert.deepEqual(c.probe.fillerCodes,fillers);assert.ok(['SOR_014','SOR_023','SOR_095','SOR_178',...fillers].every(d=>c.setupCardIds.includes(d)));assert.ok(c.coverageObligationIds.every(id=>p.obligations.some(o=>o.id===id)));assert.ok(c.assertionStages.includes('terminal'));}assert.ok(p.uncovered.some(o=>o.id.endsWith('missing-event-window-fallback')));}
    const axe=planCollectiveTriggerProbes(entry('ASH_169'),{catalogue});assert.ok(axe.cases.find(c=>c.probe.scenario==='paid-play-draw3').setupCardIds.includes('TWI_175'));assert.ok(axe.cases.some(c=>c.probe.scenario==='both-owners-regroup'));assert.ok(axe.uncovered.some(o=>o.id.endsWith('single-action-sequential-draw-windows')));
    const sabine=planCollectiveTriggerProbes(entry('ASH_208'),{catalogue});assert.ok(sabine.cases.find(c=>c.probe.scenario==='multi-xp-one-window').setupCardIds.includes('IC27_184'));assert.ok(sabine.cases.find(c=>c.probe.scenario==='sequential-upgrade-windows').setupCardIds.includes('IC27_052'));assert.ok(sabine.cases.every(c=>c.setupCardIds.includes('TOKEN_SHIELD')&&c.setupCardIds.includes('TOKEN_EXPERIENCE')));for(const name of ['unique-keep-old','unique-keep-incoming','shielded-exhausted-ground'])assert.ok(sabine.cases.some(c=>c.probe.scenario===name));
});
test('full collective field, raw event identity AST and intrinsic lifecycles fail closed',()=>{
    const mutations=[e=>e.sourceHash='0'.repeat(64),e=>e.sourcePath='game/other.ts',e=>e.status='unsupported',e=>e.baseClass='LeaderUnitCard',e=>e.card.types.push('token'),e=>e.card.cost=0,e=>e.card.hp=1,e=>e.card.power=0,e=>e.piloting={},e=>e.attachCondition={},e=>e.card.pilotText='Pilot',e=>e.abilities.card[0].collectiveTrigger=false,e=>delete e.abilities.card[0].collectiveTrigger,e=>e.abilities.card[0].hooks[0].predicate.op='!==',e=>e.abilities.card[0].hooks[0].predicate.left.steps[0].key='amount',e=>e.abilities.card[0].hooks[0].predicate.right.path=['opponent'],e=>e.abilities.card[0].effect.target='chosenPlayer',e=>e.abilities.card[0].then={effect:{kind:'noAction'}},e=>e.abilities.unit.push({}),e=>e.abilities.card.push({trigger:'whenPlayed'}),e=>e.extra=true];
    for(const id of ['ASH_169','ASH_208'])for(const change of mutations){const e=entry(id);change(e);assert.equal(classifyCollectiveTrigger(e),null);assert.equal(planCollectiveTriggerProbes(e,{catalogue}),null);}
    for(const change of [e=>e.keywordValues={},e=>e.card.keywords=[],e=>e.abilities.card[0].optional=false,e=>e.abilities.card[0].targetResolver.arena='space']){const e=entry('ASH_208');change(e);assert.equal(classifyCollectiveTrigger(e),null);}
    const axe=entry('ASH_169');axe.abilities.card[0].contextTitle[1].path=['player','title'];assert.equal(classifyCollectiveTrigger(axe),null);
});
test('source/profile matching uses behavior rather than card-ID exceptions',()=>{
    for(const id of ['ASH_169','ASH_208']){const e=entry(id);e.setCode='CUSTOM_999';e.id=e.card.id='other';e.internalName=e.card.internalName='other';e.className='SameReviewedClass';e.card.setId={set:'CUSTOM',number:999};e.card.setCodes=[e.card.setId];assert.ok(classifyCollectiveTrigger(e));assert.ok(planCollectiveTriggerProbes(e,{catalogue}).cases.every(c=>c.probe.cardId==='CUSTOM_999'));}
    for(const id of ['ASH_204','ASH_191','ASH_070','TWI_175'])assert.equal(classifyCollectiveTrigger(entry(id)),null);
});
test('native dependencies and default/filler complete fingerprints reject missing changed or ambiguous definitions',()=>{
    for(const [id,codes]of [['ASH_169',['TOKEN_ADVANTAGE','SOR_014','SOR_023','SOR_095','SOR_178','ASH_061','HMW_258']],['ASH_208',['TOKEN_SHIELD','TOKEN_EXPERIENCE','SOR_014','SOR_023']]])for(const code of codes)for(const defs of [catalogue.cards.filter(e=>e.setCode!==code),catalogue.cards.map(e=>e.setCode===code?{...e,sourceHash:'0'.repeat(64)}:e),[...catalogue.cards,entry(code)]])assert.equal(planCollectiveTriggerProbes(entry(id),{catalogue:defs}).cases.length,0);
});
test('actual producer and canonical alias guards isolate affected cases without fallback to reprints',()=>{
    const axe=planCollectiveTriggerProbes(entry('ASH_169'),{catalogue:catalogue.cards.filter(e=>e.setCode!=='TWI_175')});assert.equal(axe.cases.length,4);assert.ok(axe.cases.every(c=>['natural-regroup-draw','both-owners-regroup'].includes(c.probe.scenario)));
    const sabine=planCollectiveTriggerProbes(entry('ASH_208'),{catalogue:catalogue.cards.filter(e=>e.setCode!=='IC27_184')});assert.equal(sabine.cases.length,22);
    const defs=structuredClone(catalogue.cards);defs.find(e=>e.setCode==='IC27_184').aliasOf='LOF_239';assert.equal(planCollectiveTriggerProbes(entry('ASH_208'),{catalogue:defs}).cases.length,22);
});
test('semantic donor fingerprints ignore object order but preserve arrays, exact values and non-JSON unknown fields',()=>{
    const reorder=v=>Array.isArray(v)?v.map(reorder):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).reverse().map(([k,v])=>[k,reorder(v)])):v;
    for(const id of ['ASH_169','ASH_208'])assert.deepEqual(planCollectiveTriggerProbes(reorder(entry(id)),{catalogue:reorder(catalogue.cards)}),planCollectiveTriggerProbes(entry(id),{catalogue}));
    for(const change of [e=>e.extra=undefined,e=>e.card.cost='5',e=>e.card.cost=-0,e=>e.card.aspects.push('villainy')]){const defs=structuredClone(catalogue.cards);change(defs.find(e=>e.setCode==='TWI_175'));assert.equal(planCollectiveTriggerProbes(entry('ASH_169'),{catalogue:defs}).cases.length,4);}
});
test('assembled catalogue classifiers agree with final central composition and retain old families and fail-closed fallbacks',()=>{
    const {planCardProbes}=require('../pipeline/probes'),before=JSON.stringify(catalogue),selected=[];
    for(const e of catalogue.cards){
        const classification=classifyCollectiveTrigger(e),direct=planCollectiveTriggerProbes(e,{catalogue});
        assert.equal(Boolean(classification),Boolean(direct),e.setCode);
        if(!direct)continue;
        selected.push(e.setCode);
        const mode=e.setCode==='ASH_169'?'draw':'attachment',count=mode==='draw'?14:28;
        assert.equal(classification.family,'collective-trigger');assert.equal(classification.mode,mode);
        assert.equal(direct.cases.length,count);assert.ok(direct.cases.every(c=>c.probe.family==='collective-trigger'&&c.probe.mode===mode));
        assert.deepEqual(planCardProbes(e,{catalogue}),direct,'central dispatch must select complete direct family plan for '+e.setCode);
    }
    assert.equal(catalogue.cards.length,2605);assert.deepEqual(selected.sort(),['ASH_169','ASH_208']);
    // Existing composed families each retain their independent direct plan.
    for(const [id,module,exportName,count]of [
        ['ASH_191','defeat-cause-probes','planDefeatCauseProbes',36],
        ['ASH_197','executor-upgrade-count-probes','planExecutorUpgradeCountProbes',18],
        ['ASH_204','blade-base-damage-probes','planBladeBaseDamageProbes',24],
    ]){const e=entry(id),direct=require('../pipeline/'+module)[exportName](e,{catalogue});assert.equal(direct.cases.length,count);assert.deepEqual(planCardProbes(e,{catalogue}),direct,id);}
    // A changed source fingerprint must leave the generic, explicitly uncovered fallback.
    for(const id of ['ASH_169','ASH_208']){const e=entry(id);e.sourceHash='0'.repeat(64);assert.equal(classifyCollectiveTrigger(e),null);assert.equal(planCollectiveTriggerProbes(e,{catalogue}),null);const fallback=planCardProbes(e,{catalogue});assert.equal(fallback.cases.length,0);assert.ok(fallback.obligations.some(o=>o.dimension==='source-semantics'));assert.ok(fallback.uncovered.some(o=>o.path==='abilities.card.0'&&o.reason.startsWith('Unsupported probe construction:')));}
    assert.equal(JSON.stringify(catalogue),before);
    assert.ok(!Object.keys(require.cache).some(p=>p.includes('/forceteki/')||p.includes('/typescript/')));
});

test('actual Dire Prowess death and same-action window witnesses append four cases with exact closed Weakness dependencies',()=>{
    const p=planCollectiveTriggerProbes(entry('ASH_208'),{catalogue});
    assert.deepEqual(p.cases.slice(24).map(c=>[c.probe.scenario,c.probe.owner]),[['weakness-source-death',0],['weakness-source-death',1],['same-action-sequential-attachment-windows',0],['same-action-sequential-attachment-windows',1]]);
    for(const c of p.cases.slice(24)){assert.ok(c.setupCardIds.includes('HMW_097'));assert.ok(c.setupCardIds.includes('TOKEN_WEAKNESS'));assert.ok(c.setupCardIds.includes('TOKEN_SHIELD'));assert.ok(c.setupCardIds.includes('SOR_095'));assert.ok(c.assertionStages.includes('terminal'));}
    assert.ok(!p.uncovered.some(o=>o.id.endsWith('source-death-during-attachment-batch')));
    assert.ok(p.uncovered.some(o=>o.id.endsWith('competing-distinct-abilities-one-source')));
    // Exact metadata matters: Dire Prowess is an ordinary upgrade, not a unit.
    for(const id of ['HMW_097','TOKEN_WEAKNESS'])for(const defs of [catalogue.cards.filter(e=>e.setCode!==id),catalogue.cards.map(e=>e.setCode===id?{...e,card:{...e.card,types:['unit']}}:e),catalogue.cards.map(e=>e.setCode===id?{...e,extra:true}:e),[...catalogue.cards,entry(id)]]){
        const held=planCollectiveTriggerProbes(entry('ASH_208'),{catalogue:defs});assert.equal(held.cases.length,24);assert.deepEqual(held.cases,p.cases.slice(0,24));assert.equal(held.uncovered.filter(o=>o.reason.includes('Dire Prowess/Weakness')).length,4);
        assert.deepEqual(planCollectiveTriggerProbes(entry('ASH_169'),{catalogue:defs}),planCollectiveTriggerProbes(entry('ASH_169'),{catalogue}));
    }
});
