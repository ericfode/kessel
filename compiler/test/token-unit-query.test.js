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
test('token-unit query rejects optionality, arguments and unsupported receivers', () => {
    for (const expression of ['card?.isTokenUnit()', 'card.isTokenUnit?.()', 'card.isTokenUnit(1)', 'card.isTokenUnit(...[])', 'context.player.isTokenUnit()', 'context.player.base.isTokenUnit()', 'context.source.parentUnit.isTokenUnit()', '[card].isTokenUnit()']) {
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
test('query lowering does not admit partial Private Manufacturing or watcher-based Koska Reeves', () => {
    for (const [file, remaining] of [
        ['03_TWI/events/PrivateManufacturing.ts', 'unsupported-field:mustChangeGameState'],
        ['08_ASH/units/KoskaReevesWarriorOfMandalore.ts', 'method:setupStateWatchers']
    ]) {
        const card = full(file);
        assert.equal(card.status, 'unsupported');
        assert.ok(card.diagnostics.some(d => d.pattern === remaining), JSON.stringify(card.diagnostics));
        assert.ok(!card.diagnostics.some(d => d.pattern === 'unsupported-query:isTokenUnit'));
    }
});
