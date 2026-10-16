'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { compileSource, loadEnums } = require('../compiler');
const root = path.resolve(process.env.SWU_ENGINE_SOURCE || path.join(__dirname, '../../forceteki/server'));
const enums = loadEnums(root);
const query = receiver => ({ kind: 'query', name: 'isTokenUnit', receiver: { kind: 'ref', path: [receiver] }, args: [] });
function compile(condition) {
    return compileSource(`export default class Probe extends NonLeaderUnitCard {
        getImplementationId(){return {id:'probe',internalName:'probe'};}
        setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({
            title:'Ready a token unit',targetResolver:{cardCondition:(card,context)=>${condition},
            immediateEffect:AbilityHelper.immediateEffects.ready()}});}
    }`, 'probe.ts', enums);
}
function full(file) {
    const relative = 'game/cards/' + file;
    return compileSource(fs.readFileSync(path.join(root, relative), 'utf8'), relative, enums);
}
test('token-unit predicates retain the scalar candidate, source and outer target identities', () => {
    for (const [expression, receiver] of [['card.isTokenUnit()', 'card'], ['context.source.isTokenUnit()', 'source'], ['context.target.isTokenUnit()', 'target']]) {
        const card = compile(expression);
        assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
        assert.deepEqual(card.abilities.card[0].targetResolver.predicate, query(receiver));
    }
});
test('token-unit query rejects optionality and arguments', () => {
    for (const expression of ['card?.isTokenUnit()', 'card.isTokenUnit?.()', 'card.isTokenUnit(1)', 'card.isTokenUnit(...[])']) {
        const card = compile(expression);
        assert.equal(card.status, 'unsupported', expression);
        assert.ok(card.diagnostics.length, expression);
    }
});
test('First Order TIE Fighter retains friendly live token-unit presence and Raid 1', () => {
    // Pinned source: presence of a friendly token UNIT grants Raid, not mere
    // presence of a Shield/Experience token and not an opponent's token unit.
    const card = full('04_JTL/units/FirstOrderTIEFighter.ts');
    assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
    assert.equal(card.abilities.card.length, 1);
    const ability = card.abilities.card[0];
    assert.deepEqual(ability.condition, { kind: 'query', name: 'hasSomeArenaUnit', receiver: { kind: 'ref', path: ['player'] }, args: [{ kind: 'binary', op: '&&', left: { kind: 'const', value: true }, right: query('card') }] });
    assert.deepEqual(ability.effect, { kind: 'gainKeyword', target: 'self', duration: 'persistent', keyword: 'raid', amount: { kind: 'const', value: 1 } });
});
test('Kraken retains token creation and only friendly token-unit phase recipients', () => {
    const card = full('03_TWI/units/KrakenConfederateTactician.ts');
    assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
    assert.equal(card.abilities.card.length, 2);
    const [play, attack] = card.abilities.card;
    assert.equal(play.trigger, 'whenPlayed');
    assert.deepEqual(play.effect, { kind: 'createTokenUnit', tokenType: 'battleDroid', amount: { kind: 'const', value: 2 }, target: 'ownPlayer', entersReady: false });
    assert.equal(attack.trigger, 'onAttack');
    assert.deepEqual(attack.effect.target, { kind: 'arenaList', receiver: { kind: 'ref', path: ['player'] }, predicate: { kind: 'binary', op: '&&', left: { kind: 'const', value: true }, right: query('card') } });
    assert.equal(attack.effect.duration, 'phase');
    assert.deepEqual([attack.effect.power, attack.effect.hp], [{ kind: 'const', value: 1 }, { kind: 'const', value: 1 }]);
});
test('general token query contracts retain noncandidate and collection receivers', () => {
    for (const [expression, receiver] of [
        ['context.player.isTokenUnit()', {kind:'ref',path:['player']}],
        ['context.player.base.isTokenUnit()', {kind:'ref',path:['player','base']}],
        ['context.source.parentUnit.isTokenUnit()', {kind:'ref',path:['source','parentUnit']}],
        ['[card].isTokenUnit()', {kind:'array',items:[{kind:'ref',path:['card']}]}]
    ]) {
        const card=compile(expression);
        assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));
        assert.deepEqual(card.abilities.card[0].targetResolver.predicate, {
            kind:'cardQueryContract',operation:'isTokenUnit',receiver,evaluation:'expressionEvaluation'
        });
    }
});
test('Private Manufacturing retains exact hand selection and Koska retains phase watcher', () => {
    const manufacturing=full('03_TWI/events/PrivateManufacturing.ts');
    assert.equal(manufacturing.status,'supported',JSON.stringify(manufacturing.diagnostics));
    const sequence=manufacturing.abilities.card[0].effect;
    assert.deepEqual(sequence.effects[0],{kind:'draw',target:'ownPlayer',amount:{kind:'const',value:2}});
    assert.deepEqual(sequence.effects[1].condition.args[0].right,query('card'));
    const selected=sequence.effects[1].onFalse;
    assert.equal(selected.kind,'select');
    assert.partialDeepStrictEqual(selected.targetResolver,{kind:'cardSelection',mode:'exactly',chooser:'self',
        zones:{kind:'const',value:'hand'},controller:{kind:'const',value:'self'},numCards:{kind:'const',value:2},mustChangeGameState:'mustFullyResolve'});
    assert.deepEqual(selected.effect,{kind:'moveToBottomOfDeck',target:'chosen'});
    const koska=full('08_ASH/units/KoskaReevesWarriorOfMandalore.ts');
    assert.equal(koska.status,'supported',JSON.stringify(koska.diagnostics));
    assert.deepEqual(koska.abilities.card[0].condition.args[0].right,query('card'));
    assert.equal(koska.abilities.card[1].effect.condition.kind,'historyQuery');
    assert.equal(koska.abilities.card[1].effect.condition.binding,koska.stateWatchers[0].binding);
    assert.equal(koska.stateWatchers[0].resetEvent,'onPhaseEnded');
    assert.deepEqual(koska.abilities.card[1].effect.onTrue,{kind:'createTokenUnit',tokenType:'mandalorian',amount:{kind:'const',value:1},target:'ownPlayer',entersReady:false});
});
