'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {compileSource, loadEnums} = require('../compiler');
const root = path.resolve(process.env.SWU_ENGINE_SOURCE || path.join(__dirname, '../../forceteki/server'));
const enums = loadEnums(root);
const leaderPredicate = {kind:'binary', op:'&&', left:{kind:'query',name:'isUnit',receiver:{kind:'ref',path:['card']},args:[]},right:{kind:'query',name:'isLeader',receiver:{kind:'ref',path:['card']},args:[]}};
function source(condition) {
    return `export default class Probe extends NonLeaderUnitCard {
        getImplementationId(){return{id:'probe',internalName:'probe'}}
        setupCardAbilities(registrar,AbilityHelper){registrar.addConstantAbility({
            condition:context=>${condition},ongoingEffect:AbilityHelper.ongoingEffects.modifyStats({power:2,hp:0})});}
    }`;
}
function compile(condition) {return compileSource(source(condition),'probe.ts',enums);}
function full(file) {const p='game/cards/08_ASH/units/'+file+'.ts';return compileSource(fs.readFileSync(path.join(root,p),'utf8'),p,enums);}

test('synchronous arena candidate lowers leader-unit identity to existing unit and leader tests',()=>{
    const card=compile('context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit()})');
    assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));
    assert.deepEqual(card.abilities.card[0].condition.args[0].right,leaderPredicate);
    const list=compile('context.player.getArenaUnits({condition:c=>c.isLeaderUnit()}).length>0');
    assert.equal(list.status,'supported',JSON.stringify(list.diagnostics));
    const filtered=compile('context.player.getArenaUnits().some(c=>c.isLeaderUnit())');
    assert.equal(filtered.status,'supported',JSON.stringify(filtered.diagnostics));
    assert.deepEqual(filtered.abilities.card[0].condition.predicate,leaderPredicate);
});
test('only synchronous scalar arena candidates qualify, with no optionality or arguments',()=>{
    for(const condition of [
        'context.source.isLeaderUnit()', 'context.target.isLeaderUnit()',
        'context.player.hasSomeArenaUnit({condition:c=>context.source.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:c=>context.target.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:c=>c.parentUnit.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:c=>c?.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit?.()})',
        'context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit(1)})',
        'context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit(...[])})',
        'context.player.hasSomeArenaUnit({condition:async c=>c.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:(c=sideEffect())=>c.isLeaderUnit()})',
        'context.player.hasSomeArenaUnit({condition:function*(c){return c.isLeaderUnit()}})',
        'context.player.hasSomeArenaUnit({condition:(c,other)=>c.isLeaderUnit()})'
    ]) assert.equal(compile(condition).status,'unsupported',condition);
    const selector=source('true').replace('condition:context=>true,','matchTarget:c=>c.isLeaderUnit(),');
    assert.equal(compileSource(selector,'selector.ts',enums).status,'unsupported');
    const synchronous=source('context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit()})');
    for(const enclosing of [
        synchronous.replace('condition:context=>','condition:async context=>'),
        synchronous.replace('setupCardAbilities(', 'async setupCardAbilities('),
        synchronous.replace('setupCardAbilities(', '*setupCardAbilities('),
        synchronous.replace('condition:context=>context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit()})','condition:function*(context){return context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit()})}'),
        synchronous.replace('condition:context=>context.player.hasSomeArenaUnit({condition:c=>c.isLeaderUnit()})','condition:async context=>context.player.getArenaUnits().some(c=>c.isLeaderUnit())')
    ]) assert.equal(compileSource(enclosing,'async-ancestor.ts',enums).status,'unsupported');
});
test('two complete sources retain independent constant effects and friendly live count',()=>{
    const commandos=full('MandalorianSuperCommandos');
    assert.equal(commandos.status,'supported',JSON.stringify(commandos.diagnostics));
    assert.equal(commandos.abilities.card.length,1);
    assert.deepEqual(commandos.abilities.card[0].condition.args[0].right,leaderPredicate);
    assert.deepEqual(commandos.abilities.card[0].effect,{kind:'modifyStats',target:'self',duration:'persistent',power:{kind:'const',value:2},hp:{kind:'const',value:0}});
    const flagship=full('MandalorianFlagshipCapturedFromTheEmpire');
    assert.equal(flagship.status,'supported',JSON.stringify(flagship.diagnostics));
    assert.equal(flagship.abilities.card.length,2);
    assert.deepEqual(flagship.abilities.card[0].condition.args[0].right,leaderPredicate);
    assert.deepEqual(flagship.abilities.card[0].effect,{kind:'gainKeyword',target:'self',duration:'persistent',keyword:'ambush',amount:{kind:'const',value:1}});
    const power=flagship.abilities.card[1].effect.power;
    assert.equal(power.name,'arenaUnitCount');
    assert.deepEqual(power.receiver,{kind:'ref',path:['modifierTarget','controller']});
    assert.equal(power.args[0].left.right.args[0].value,'mandalorian');
    assert.deepEqual(power.args[0].right.right.right,{kind:'ref',path:['modifierTarget']});
});
test('pinned engine leader-unit guard follows deployed unit status',()=>{
    const build=path.resolve(root,'../build/server/game/core/card');
    const {LeaderUnitCard}=require(path.join(build,'LeaderUnitCard.js'));
    const {Card}=require(path.join(build,'Card.js'));
    const {NonLeaderUnitCard}=require(path.join(build,'NonLeaderUnitCard.js'));
    const {TokenUnitCard}=require(path.join(build,'TokenCards.js'));
    for(const deployed of [false,true]) {
        const receiver=Object.create(LeaderUnitCard.prototype);
        receiver.isUnit=()=>deployed;
        assert.equal(receiver.isLeaderUnit(),deployed);
        assert.equal(receiver.isLeader()&&receiver.isUnit(),deployed);
    }
    assert.equal(Card.prototype.isLeaderUnit.call({}),false);
    // Arena unit mixins use attached-leader presence for both guards. Piloting
    // remains outside this runtime's admitted pool; this checks engine identity.
    for(const Class of [NonLeaderUnitCard,TokenUnitCard]) for(const attached of [false,true]) {
        const receiver=Object.create(Class.prototype);
        receiver.isLeaderAttachedToThis=()=>attached;
        receiver.isUnit=()=>true;
        assert.equal(receiver.isLeaderUnit(),attached);
        assert.equal(receiver.isLeaderUnit(),receiver.isUnit()&&receiver.isLeader());
    }
});
