'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const {compileSource} = require('../compiler');
const source = amount => `export default class AttachedProbe extends UpgradeCard {
    getImplementationId(){return{id:'attached-probe',internalName:'attached-probe'}}
    setupCardAbilities(registrar,AbilityHelper){registrar.addGainKeywordTargetingAttached({keyword:KeywordName.Sentinel${amount === undefined ? "" : ",amount:"+amount}});}
}`;
test('attached keyword amount contracts reject explicitly rather than throwing a compiler TypeError', () => {
    const enums = {'KeywordName.Sentinel':'sentinel'};
    const valid = compileSource(source(), 'attached-probe.ts', enums);
    assert.equal(valid.status, 'supported', JSON.stringify(valid.diagnostics));
    assert.equal(valid.abilities.card[0].effect.target, 'parent');
    assert.equal(valid.abilities.card[0].effect.amount.value, 1);
    const invalid = compileSource(source('ctx=>2'), 'attached-probe.ts', enums);
    assert.equal(invalid.status, 'unsupported');
    assert.ok(invalid.diagnostics.some(d=>d.pattern==='attached-keyword-amount-bound'));
});
