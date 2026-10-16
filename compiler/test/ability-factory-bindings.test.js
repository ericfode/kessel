'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const {compileSource} = require('../compiler');

test('continuation block captures one arena list before constructing mutating effects', () => {
    const source = `export default class FactoryProbe extends NonLeaderUnitCard {
        getImplementationId(){return{id:'factory-probe',internalName:'factory-probe'}}
        setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({title:'Draw',
            immediateEffect:AbilityHelper.immediateEffects.draw(),then:ctx=>{
                const units=ctx.player.getArenaUnits();
                return {immediateEffect:AbilityHelper.immediateEffects.sequential([
                    AbilityHelper.immediateEffects.defeat({target:units}),
                    AbilityHelper.immediateEffects.ready({target:units})])};
            }});}
    }`;
    const card = compileSource(source, 'factory-probe.ts');
    assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
    const factory = card.abilities.card[0].then;
    assert.equal(factory.kind, 'abilityFactoryProgram');
    assert.equal(factory.evaluation, 'nativeAbilityFactoryInvocation');
    const callback = factory.callback;
    assert.equal(callback.invokeAt, 'nativeCallbackInvocation');
    const [declarations, returned] = callback.body.statements;
    assert.equal(callback.body.statements.length, 2);
    assert.equal(declarations.kind, 'pureBindings');
    assert.equal(declarations.bindings.length, 1);
    const binding = declarations.bindings[0];
    assert.equal(binding.evaluateAt, 'declaration');
    assert.deepEqual(binding.bindingPattern, {name:'units'});
    assert.equal(binding.value.kind, 'dataCall');
    assert.equal(binding.value.method, 'getArenaUnits');
    assert.equal(returned.kind, 'pureReturn');
    const effects = returned.value.ability.effect.effects;
    assert.deepEqual(effects.map(e=>e.kind), ['defeat','ready']);
    assert.deepEqual(effects.map(e=>e.target), [0,1].map(()=>({kind:'dataPath',root:[binding.slot],steps:[]})));
});
