'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(__dirname,'../../forceteki/server'),enums=loadEnums(root);
function compile(modifier,fields='') {
    return compileSource(`export default class FreshDynamic extends NonLeaderUnitCard {
        getImplementationId(){return {id:'fresh-dynamic',internalName:'fresh-dynamic'};}
        setupCardAbilities(registrar,AbilityHelper){registrar.addConstantAbility({${fields}ongoingEffect:AbilityHelper.ongoingEffects.modifyStats(${modifier})});}
    }`,'fresh-dynamic.ts',enums);
}
function supported(modifier,fields=''){const c=compile(modifier,fields);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];}
function rejected(modifier,fields=''){const c=compile(modifier,fields);assert.equal(c.status,'unsupported',modifier);return c;}
const literal=value=>({kind:'const',value});
const data=(root,...keys)=>({kind:'dataPath',root:[root],steps:keys.map(key=>({key,optional:false}))});
const binary=(op,left,right)=>({kind:'binary',op,left,right});
const query=(name,receiver,args=[])=>({kind:'ongoingQuery',name,receiver,args,optionalReceiver:false,optionalCall:false,continuesOptionalChain:false});
function dynamic(modifier,fields='') {
 const effect=supported(modifier,fields).effect;assert.equal(effect.kind,'constantEffectRegistration');
 assert.equal(effect.dependencyPolicy,'liveFixedPointRequired');assert.equal(effect.modifier.kind,'ongoingContract');
 assert.equal(effect.modifier.valueMode,'dynamic');const callback=effect.modifier.value.callback;
 assert.equal(callback.capture,'lexicalReferences');assert.equal(callback.invokeAt,'nativeCallbackInvocation');
 return {effect,callback,fields:callback.body.value?.fields};
}
function nodes(value,kind) {if(!value||typeof value!=='object')return [];return [...(value.kind===kind?[value]:[]),...Object.values(value).flatMap(v=>nodes(v,kind))];}
test('persistent callback binds affected card independently and keeps live scalar arithmetic',()=>{
    const a=supported('(unit,ctx)=>({power:unit.damage*2+ctx.player.hand.length,hp:unit.controller.resources.length})');
    assert.deepEqual(a.effect.power.left.left,{kind:'ref',path:['modifierTarget','damage']});
    assert.deepEqual(a.effect.power.right,{kind:'ref',path:['player','hand','length']});
    assert.deepEqual(a.effect.hp,{kind:'ref',path:['modifierTarget','controller','resources','length']});
    assert.equal(a.effect.duration,'persistent');
    assert.deepEqual(supported('unit=>({power:-unit.damage,hp:0})').effect.power.value,{kind:'ref',path:['modifierTarget','damage']});
    assert.deepEqual(supported('{power:1,hp:-1}').effect.power,{kind:'const',value:1});
});
test('resource pair expression lowers only exact floor of total resources divided by two',()=>{
    for(const [text,receiver]of [['unit.controller',['modifierTarget','controller']],['ctx.player',['player']],['ctx.player.opponent',['opponent']]]) {
        const a=supported(`(unit,ctx)=>({power:Math.floor(${text}.resources.length/2),hp:0})`);
        assert.deepEqual(a.effect.power,{kind:'query',name:'resourcePairs',receiver:{kind:'ref',path:receiver},args:[]});
    }
    // Broader numeric forms use an explicit dynamic contract; they are not
    // silently normalized to the bounded resourcePairs operation.
    for(const [text,expected] of [
        ['Math.floor(unit.damage/2)',root=>({kind:'math',name:'floor',args:[binary('/',data(root,'damage'),literal(2))]})],
        ...[3,0].map(divisor=>[`Math.floor(unit.controller.resources.length/${divisor})`,root=>({kind:'math',name:'floor',args:[binary('/',data(root,'controller','resources','length'),literal(divisor))]})]),
        ['Math.floor(unit.controller.resources.length)',root=>({kind:'math',name:'floor',args:[data(root,'controller','resources','length')]})],
        ['Math.ceil(unit.controller.resources.length/2)',root=>({kind:'math',name:'ceil',args:[binary('/',data(root,'controller','resources','length'),literal(2))]})],
        ['unit.controller.resources.length/2',root=>binary('/',data(root,'controller','resources','length'),literal(2))],
        ['Math.floor(unit.controller.resources.length/2,3)',root=>({kind:'math',name:'floor',args:[binary('/',data(root,'controller','resources','length'),literal(2)),literal(3)]})]
    ]) {const d=dynamic(`unit=>({power:${text},hp:0})`);assert.deepEqual(d.fields.power,expected(d.callback.parameters[0].slot));}

});
test('nested arena predicate keeps candidate, source and modifier target distinct',()=>{
    const a=supported(`(unit,ctx)=>({power:unit.controller.getArenaUnits({otherThan:unit,condition:card=>card.zone.name===ZoneName.SpaceArena&&card.damage>0}).length,hp:ctx.player.getArenaUnits({otherThan:ctx.source,trait:Trait.Ewok}).length})`);
    assert.deepEqual(a.effect.power.receiver,{kind:'ref',path:['modifierTarget','controller']});
    const refs=[];const visit=v=>{if(!v||typeof v!=='object')return;if(v.kind==='ref')refs.push(v.path.join('.'));Object.values(v).forEach(visit);};visit(a);
    assert.ok(refs.includes('card.zoneName'));assert.ok(refs.includes('modifierTarget'));assert.ok(refs.includes('source'));
    const token=supported('(_,ctx)=>({power:ctx.player.getArenaUnits({condition:card=>card.isTokenUnit()&&!card.exhausted}).length,hp:0})');
    assert.equal(token.effect.power.args[0].right.left.name,'isTokenUnit');
    for(const [condition,expected] of [
        ['card.name===unit.name',(card,unit)=>binary('===',data(card,'name'),data(unit,'name'))],
        ['card.damage>unit.damage',(card,unit)=>binary('>',data(card,'damage'),data(unit,'damage'))],
        ['card.getPower()===0',card=>binary('===',query('getPower',data(card)),literal(0))],
        ['card.hp>1',card=>binary('>',data(card,'hp'),literal(1))],
        ['card.hasSomeKeyword(KeywordName.Sentinel)',card=>query('hasSomeKeyword',data(card),[literal('sentinel')])],
        ['ctx.target===card',(card,unit,context)=>binary('===',data(context,'target'),data(card))]
    ]) {
        const d=dynamic(`(unit,ctx)=>({power:ctx.player.getArenaUnits({condition:card=>${condition}}).length,hp:0})`);
        const q=d.fields.power.receiver;assert.equal(d.fields.power.member,'length');assert.equal(q.name,'getArenaUnits');
        const predicate=q.args[0].properties.condition.callback;
        assert.equal(predicate.parameters[0].argument,'candidate');
        assert.notEqual(predicate.parameters[0].slot,d.callback.parameters[0].slot);
        assert.deepEqual(predicate.body.value,expected(predicate.parameters[0].slot,d.callback.parameters[0].slot,d.callback.parameters[1].slot));
    }
});
test('dynamic modifier contracts retain fields, lexical reads and phase duration',()=>{
    for(const [record,keys] of [['power:unit.damage,hp:0,unknown:true',['power','hp','unknown']],['power:unit.damage,hp:0,effect:"hidden"',['power','hp','effect']],['power:unit.damage',['power']]]) {
        const d=dynamic(`unit=>({${record}})`);assert.deepEqual(Object.keys(d.fields),keys);
        assert.deepEqual(d.fields.power,data(d.callback.parameters[0].slot,'damage'));
        if(d.fields.unknown)assert.deepEqual(d.fields.unknown,literal(true));
        if(d.fields.effect)assert.deepEqual(d.fields.effect,literal('hidden'));
    }
    let d=dynamic('unit=>({power:()=>unit.damage,hp:0})');
    assert.equal(d.fields.power.kind,'pureCallback');assert.deepEqual(d.fields.power.body.value,data(d.callback.parameters[0].slot,'damage'));
    d=dynamic('unit=>({power:unit.power,hp:0})');assert.deepEqual(d.fields.power,data(d.callback.parameters[0].slot,'power'));
    for(const [text,path]of [['unit.upgrades.length',['modifierTarget']],['unit.controller.base.upgrades.length',['modifierTarget','controller','base']]]) {
        assert.deepEqual(supported(`unit=>({power:${text},hp:0})`).effect.power,{kind:'attachmentCount',receiver:{kind:'ref',path},limit:256});
    }
    d=dynamic('unit=>({power:unit.controller.discardZone.cards.filter(c=>c.isUnit()).length,hp:0})');
    assert.equal(d.fields.power.member,'length');const filter=d.fields.power.receiver;assert.equal(filter.kind,'pureCollectionCall');assert.equal(filter.method,'filter');
    assert.deepEqual(filter.receiver,data(d.callback.parameters[0].slot,'controller','discardZone','cards'));
    assert.deepEqual(filter.args[0].body.value,query('isUnit',data(filter.args[0].parameters[0].slot)));
    d=dynamic('(unit,ctx)=>({power:ctx.target.damage,hp:0})');assert.deepEqual(d.fields.power,data(d.callback.parameters[1].slot,'target','damage'));
    d=dynamic('(unit,ctx)=>({power:ctx.events[0].totalDistributed,hp:0})');
    assert.deepEqual(d.fields.power,{kind:'dataPath',root:[d.callback.parameters[1].slot],steps:[{key:'events',optional:false},{index:literal(0),optional:false},{key:'totalDistributed',optional:false}]});
    d=dynamic('unit=>({power:unit.damage*1000,hp:0})');assert.deepEqual(d.fields.power,binary('*',data(d.callback.parameters[0].slot,'damage'),literal(1000)));
    d=dynamic('unit=>({power:unit.damage,hp:0})','condition:ctx=>ctx.source.getPower()>0,');assert.equal(d.effect.properties.condition.body.left.name,'getPower');
    d=dynamic('unit=>({power:unit.damage,hp:0})','matchTarget:card=>card.remainingHp>0,');const match=d.effect.properties.matchTarget;assert.deepEqual(match.body.left,{kind:'ref',path:[match.parameters[0].root,'remainingHp']});
    rejected('unit=>{unit.damage=2;return {power:unit.damage,hp:0};}');
    const phase=compileSource(`export default class Phase extends NonLeaderUnitCard{getImplementationId(){return{id:'phase',internalName:'phase'};}setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.forThisPhaseCardEffect({effect:AbilityHelper.ongoingEffects.modifyStats(unit=>({power:unit.damage,hp:0}))})});}}`,'phase.ts',enums);
    assert.equal(phase.status,'supported');const effect=phase.abilities.card[0].effect;
    assert.partialDeepStrictEqual(effect,{kind:'ongoingContract',duration:'phase',valueMode:'dynamic',effectType:'modifyStats'});
    assert.deepEqual(effect.value.callback.body.value.fields.power,data(effect.value.callback.parameters[0].slot,'damage'));
});
test('complete live sources retain scalar or explicit ongoing and attachment contracts',()=>{
    const files=['09_HMW/units/WroshyrRebel.ts','09_HMW/units/ChiefChirpaDefiantElder.ts','04_JTL/units/DQarCargoFrigate.ts','04_JTL/units/CloneCombatSquadron.ts','05_LOF/units/PazVizslaUnyieldingWarrior.ts','06_SEC/leaders/GovernorPryceTyrantOfLothal.ts','06_SEC/units/KinoLoyYouAnswerToMe.ts','01_SOR/units/97thLegionKeepingThePeaceOnSullust.ts','07_TS26/units/GeneralGrievousCrushThem.ts'];
    for(const file of files){const source=path.join(root,'game/cards',file);const c=compileSource(fs.readFileSync(source,'utf8'),source,enums);assert.equal(c.status,'supported',file+JSON.stringify(c.diagnostics));}
    const extra=[
        ['08_ASH/units/KelleranBeqWhereAreTheOthers.ts','ongoingQuery','getPower'],
        ['09_HMW/units/CarrionSpikeHarbingerOfTyranny.ts','memberPath',null],
        ['04_JTL/units/SwarmingVultureDroid.ts','dataPath',null],
        ['05_LOF/units/AxeWovesAccomplishedWarrior.ts','attachmentCount',null],
        ['05_LOF/units/CaptainEnochCaptainOfTheGuard.ts','pureCollectionCall','filter']
    ];
    for(const [file,kind,name] of extra){
        const source=path.join(root,'game/cards',file),c=compileSource(fs.readFileSync(source,'utf8'),source,enums);
        assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));const found=nodes(c.abilities,kind);assert.ok(found.length,file);
        if(name)assert.ok(found.some(n=>n.name===name||n.method===name),file);
        if(kind==='memberPath')assert.deepEqual(found[0].steps,[{property:'base',optional:true},{property:'upgrades',optional:false},{property:'length',optional:false}]);
        if(kind==='attachmentCount')assert.deepEqual(found,[{kind:'attachmentCount',receiver:{kind:'ref',path:['modifierTarget']},limit:256},{kind:'attachmentCount',receiver:{kind:'ref',path:['modifierTarget']},limit:256}]);
        if(file.includes('SwarmingVultureDroid')) {
            const callback=c.abilities.card[0].effect.modifier.value.callback;
            const predicate=nodes(callback,'ongoingQuery').find(n=>n.name==='getArenaUnits').args[0].properties.condition.callback;
            assert.deepEqual(predicate.body.value,binary('===',data(predicate.parameters[0].slot,'name'),data(callback.parameters[0].slot,'name')));
        }
    }
});
