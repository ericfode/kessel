'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
for(const [file,trigger]of [['game/cards/01_SOR/units/EmperorPalpatineMasterOfTheDarkSide.ts','whenPlayed'],['game/cards/05_LOF/units/TheLegacyRunDoomedDebris.ts','whenDefeated']]){
 const source=fs.readFileSync(path.join(root,file),'utf8');
 test(`complete distributed damage source ${trigger}`,()=>{const c=compileSource(source,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.deepEqual(c.abilities,{leader:[],unit:[],card:[{trigger,title:'Deal 6 damage divided as you choose among enemy units',costs:[],effect:{kind:'distributeDamage',amount:{kind:'const',value:6},canChooseNoTargets:false,canDistributeLess:false,targetResolver:{owner:'opponent',type:'unit',zone:'anyAttackable'}}}]});});
 test(`distributed damage compiler retains semantically distinct overrides ${trigger}`,()=>{for(const [before,after,field,value]of [['amountToDistribute: 6','amountToDistribute: 5','amount',{kind:'const',value:5}],['canChooseNoTargets: false','canChooseNoTargets: true','canChooseNoTargets',true],['amountToDistribute: 6','maxTargets: 2, amountToDistribute: 6','maxTargets',2]]){const c=compileSource(source.replace(before,after),file,enums);assert.equal(c.status,'supported');assert.deepEqual(c.abilities.card[0].effect[field],value);}const c=compileSource(source.replace('amountToDistribute: 6','unknownField: true, amountToDistribute: 6'),file,enums);assert.equal(c.status,'unsupported');});
}

test('fresh compileCatalog and shard loadCatalog preserve all60 interaction plans', () => {
    const fresh = require('../build').compileCatalog();
    const catalogue = require('../load-catalog').loadCatalog();
    const { planDistributedDamageProbes } = require('../../cards/pipeline/distributed-damage-probes');
    const entry = id => structuredClone(catalogue.cards.find(e => e.setCode === id));
    for (const id of ['SOR_135','LOF_213']) {
        const baseline = planDistributedDamageProbes(entry(id), { catalogue });
        const compiled = planDistributedDamageProbes(fresh.cards.find(e => e.setCode === id), { catalogue: fresh });
        assert.equal(compiled.cases.length, id === 'SOR_135' ? 32 : 28);
        assert.deepEqual(compiled.cases, baseline.cases);
        assert.deepEqual(compiled.interactionDonorAdmissionReceipt, baseline.interactionDonorAdmissionReceipt);
    }
});
