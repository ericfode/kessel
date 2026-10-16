'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(__dirname,'../../forceteki/server'),enums=loadEnums(root);
function compile(predicate,field='matchTarget') {
    return compileSource(`export default class FreshAttack extends NonLeaderUnitCard {
        getImplementationId(){return{id:'fresh-attack',internalName:'fresh-attack'};}
        setupCardAbilities(registrar,AbilityHelper){registrar.addConstantAbility({${field}:${field==='matchTarget'?'(unit,ctx)':'ctx'}=>${predicate},ongoingEffect:AbilityHelper.ongoingEffects.modifyStats({power:-1,hp:0})});}
    }`,'fresh-attack.ts',enums);
}
function positive(text,field){const c=compile(text,field);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];}
function negative(text,field){assert.equal(compile(text,field).status,'unsupported',text);}
function nodes(value,kind) {
    if(!value||typeof value!=='object')return [];
    return [...(value.kind===kind?[value]:[]),...Object.values(value).flatMap(v=>nodes(v,kind))];
}
const ref=(...path)=>({kind:'ref',path});
const live=card=>({kind:'liveActiveAttack',card:ref(card),optionalReceiver:false});
const member=(receiver,name)=>({kind:'attackMember',receiver,member:name,optionalReceiver:false,continuesOptionalChain:false});
test('attack queries preserve legacy scalar receivers and general call contracts',()=>{
    assert.deepEqual(positive('ctx.source.isDefending()','condition').condition,{kind:'query',name:'isDefending',receiver:ref('source'),args:[]});
    assert.deepEqual(positive('unit.isUnit()&&unit.isAttacking()').targetResolver.predicate.right,{kind:'query',name:'isAttacking',receiver:ref('card'),args:[]});
    assert.deepEqual(positive('ctx.target.isDefending()','condition').condition.receiver,ref('target'));
    for(const [text,name,path,args,optionalReceiver,optionalCall] of [
        ['unit.isAttacking(true)','isAttacking',['candidate'],[{kind:'const',value:true}],false,false],
        ['unit.isDefending(ctx.source)','isDefending',['candidate'],['contextSource'],false,false],
        ['unit.isDefending({unknown:true})','isDefending',['candidate'],[{kind:'record',properties:{unknown:{kind:'const',value:true}}}],false,false],
        ['unit?.isDefending()','isDefending',['candidate'],[],true,false],
        ['unit.isDefending?.()','isDefending',['candidate'],[],false,true],
        ['ctx.player.isAttacking()','isAttacking',['ongoingContext','player'],[],false,false],
        ['unit.controller.isDefending()','isDefending',['candidate','controller'],[],false,false],
        ['unit.activeAttack.isDefending()','isDefending',null,[],false,false]
    ]) {
        const effect=positive(text).effect;assert.equal(effect.kind,'constantEffectRegistration');
        const callback=effect.properties.matchTarget;assert.equal(callback.kind,'attackCallback');
        const root=role=>callback.parameters.find(p=>p.role===role).root;
        const receiver=path?ref(root(path[0]),...path.slice(1)):live(root('candidate'));
        assert.deepEqual(callback.body,{kind:'ongoingQuery',name,receiver,args:args.map(a=>a==='contextSource'?ref(root('ongoingContext'),'source'):a),optionalReceiver,optionalCall,continuesOptionalChain:optionalReceiver||optionalCall});
    }
});
test('live attack reads preserve guard expression order, identity and optional steps',()=>{
    const a=positive('unit.isUnit()&&unit.isInPlay()&&unit.isDefending()&&unit.activeAttack.attacker===ctx.source');
    assert.deepEqual(a.targetResolver.predicate.right.left,member(live('card'),'attacker'));
    assert.deepEqual(a.targetResolver.predicate.right.right,ref('source'));
    for(const [text,topOp] of [
        ['unit.activeAttack.attacker===ctx.source','==='],
        ['unit.activeAttack.attacker===ctx.source&&unit.isDefending()','&&'],
        ['!unit.isDefending()&&unit.activeAttack.attacker===ctx.source','&&'],
        ['unit.isDefending()||unit.activeAttack.attacker===ctx.source','||'],
        ['(unit.isDefending()||ctx.source.isDefending())&&unit.activeAttack.attacker===ctx.source','&&'],
        ['ctx.source.isDefending()&&unit.activeAttack.attacker===ctx.source','&&']
    ]) {
        const predicate=positive(text).targetResolver.predicate;assert.equal(predicate.op,topOp);
        assert.deepEqual(nodes(predicate,'attackMember'),[member(live('card'),'attacker')]);
        assert.deepEqual(nodes(predicate,'ref').at(-1),text.endsWith('isDefending()')?ref('card'):ref('source'));
    }
    const optional=positive('unit.isDefending()&&unit.activeAttack?.attacker===ctx.source').targetResolver.predicate.right.left;
    assert.deepEqual(optional,{kind:'memberPath',receiver:live('card'),steps:[{property:'attacker',optional:true}]});
    assert.deepEqual(positive('unit.isDefending()&&unit.activeAttack.attacker.controller===ctx.player').targetResolver.predicate.right.left,member(member(live('card'),'attacker'),'controller'));
    assert.deepEqual(positive('unit.isDefending()&&unit.activeAttack.isAmbush').targetResolver.predicate.right,member(live('card'),'isAmbush'));
    assert.deepEqual(positive('unit.isDefending()&&ctx.source.activeAttack.attacker===ctx.source').targetResolver.predicate.right.left,member(live('source'),'attacker'));
    const targets=positive('unit.isDefending()&&unit.activeAttack.getAllTargets().includes(ctx.source)').targetResolver.predicate.right;
    assert.equal(targets.kind,'attackValueQuery');assert.equal(targets.name,'includes');assert.deepEqual(targets.args,[ref('source')]);
    assert.partialDeepStrictEqual(targets.receiver,{kind:'attackValueQuery',name:'getAllTargets',receiver:live('card'),args:[]});
    const target=positive('unit.isDefending()&&unit.activeAttack.targetIsUnit(c=>c.damage>0)').targetResolver.predicate.right;
    assert.equal(target.name,'targetIsUnit');assert.deepEqual(target.receiver,live('card'));
    assert.deepEqual(target.args[0].body.left,ref(target.args[0].parameters[0].root,'damage'));
    const inner=positive('unit.isDefending()&&ctx.player.hasSomeArenaUnit({condition:inner=>inner.activeAttack.attacker===ctx.source})').targetResolver.predicate.right;
    assert.equal(inner.name,'hasSomeArenaUnit');assert.deepEqual(inner.args[0].right.left,member(live('card'),'attacker'));
    for(const text of ['unit.damage=1','unknownUnit.isDefending()','unit.isDefending('])negative(text);
});
test('complete attack sources retain scalar lowering or their explicit additional contracts',()=>{
    const cases=[["game/cards/08_ASH/leaders/GroguCharmingCompanion.ts", false], ["game/cards/08_ASH/units/ScionShuttleAtMorgansBidding.ts", true], ["game/cards/08_ASH/units/PalaceChefDroid.ts", true], ["game/cards/08_ASH/units/HeroicPurrgil.ts", false], ["game/cards/08_ASH/units/MarroksFiendFighterFormidablePursuer.ts", false], ["game/cards/09_HMW/units/BatcherLoyalHound.ts", true], ["game/cards/09_HMW/units/AwakenedExogorth.ts", true], ["game/cards/09_HMW/units/BlockadeShip.ts", true], ["game/cards/04_JTL/units/GoldLeaderFastestShipInTheFleet.ts", true], ["game/cards/04_JTL/units/RetrofittedAirspeeder.ts", false], ["game/cards/07_LAW/units/LandoCalrissianEyesOpen.ts", true], ["game/cards/05_LOF/units/JediGuardian.ts", true], ["game/cards/06_SEC/units/CassianAndorLayLow.ts", false], ["game/cards/06_SEC/units/MirajScintelTheWeakDeserveToKneel.ts", false], ["game/cards/06_SEC/units/VelSarthaOnePathOneChoice.ts", true], ["game/cards/02_SHD/leaders/MoffGideonFormidableCommander.ts", false], ["game/cards/02_SHD/units/ConcordDawnInterceptors.ts", true], ["game/cards/02_SHD/units/JangoFettRenownedBountyHunter.ts", false], ["game/cards/02_SHD/units/EnfysNestMarauder.ts", false], ["game/cards/01_SOR/leaders/JynErsoResistingOppression.ts", false], ["game/cards/01_SOR/upgrades/Electrostaff.ts", true], ["game/cards/01_SOR/units/FirstLegionSnowtrooper.ts", false], ["game/cards/01_SOR/units/StrafingGunship.ts", false], ["game/cards/03_TWI/units/CloneDiveTrooper.ts", true]];
    const expanded={
        GroguCharmingCompanion:{kind:'deployLeaderSystem'},HeroicPurrgil:{kind:'liveActiveAttack',optionalReceiver:false},
        MarroksFiendFighterFormidablePursuer:{kind:'ref',path:['card','activeAttack','attacker']},
        RetrofittedAirspeeder:{kind:'attackValueQuery',name:'targetIsUnit'},CassianAndorLayLow:{kind:'damageModificationRegistration'},
        MirajScintelTheWeakDeserveToKneel:{kind:'attackCollection',name:'some'},MoffGideonFormidableCommander:{kind:'attackLastingEntry'},
        JangoFettRenownedBountyHunter:{kind:'liveKeywordDependentGrant'},EnfysNestMarauder:{kind:'attackMember',member:'isAmbush'},
        JynErsoResistingOppression:{kind:'attackLastingEntry'},FirstLegionSnowtrooper:{kind:'ref',path:['card','activeAttack','attacker']},
        StrafingGunship:{kind:'attackPermission'}
    };
    for(const [file,scalar]of cases){
        const c=compileSource(fs.readFileSync(path.join(root,file),'utf8'),file,enums);
        assert.equal(c.status,'supported',file+JSON.stringify(c.diagnostics));
        if(!scalar){const expected=expanded[path.basename(file,'.ts')];assert.ok(expected,file);
            assert.ok(nodes(c.abilities,expected.kind).some(node=>{try{assert.partialDeepStrictEqual(node,expected);return true;}catch{return false;}}),file+JSON.stringify(c.abilities));}
    }
});
