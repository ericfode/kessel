'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const {compileSource} = require('../compiler');

function source(formal = 'registrar', declaration = '') {
    return `export default class EnumProbe extends NonLeaderUnitCard {
        getImplementationId(){return{id:'enum-probe',internalName:'enum-probe'}}
        setupCardAbilities(${formal}, AbilityHelper){${declaration}
            ${formal}.addConstantAbility({targetController:RelativePlayer.Opponent,
                condition:ctx=>ctx.source.getPower()>2,
                ongoingEffect:AbilityHelper.ongoingEffects.modifyStats({power:1,hp:0})});}
    }`;
}
const enums = {'RelativePlayer.Opponent':'opponent'};
test('generic constant contract retains an unshadowed enum controller', () => {
    const card = compileSource(source(), 'enum-probe.ts', enums);
    assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
    const effect = card.abilities.card[0].effect;
    assert.equal(effect.kind, 'constantEffectRegistration');
    assert.equal(effect.properties.targetController, 'opponent');
});
test('generic constant fallback cannot reinterpret formal or local enum shadows', () => {
    for (const text of [source('RelativePlayer'), source('registrar', 'const RelativePlayer={Opponent:"self"};')]) {
        const card = compileSource(text, 'enum-probe.ts', enums);
        assert.equal(card.status, 'unsupported', JSON.stringify(card.abilities));
        assert.ok(card.diagnostics.some(d=>d.pattern==='syntax-alias-enum-binding'), JSON.stringify(card.diagnostics));
    }
});
