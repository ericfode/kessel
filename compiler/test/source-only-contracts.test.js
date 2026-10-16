'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fixtures = require('../../crates/kessel/tests/fixtures/source-only-contracts.json');
const {compileSource} = require('../compiler');

// The same exact source-derived IR is rejected by Rust's source_only_contracts
// integration test. Source preservation must not silently become execution.
for (const fixture of fixtures) test(`retained ${fixture.name} contract matches the Rust boundary fixture`, () => {
    const actual = compileSource(fixture.source, fixture.file);
    assert.equal(actual.status, 'supported', JSON.stringify(actual.diagnostics));
    assert.deepEqual(actual, fixture.compiled);
    assert.ok(JSON.stringify(actual.abilities).includes(`"kind":"${fixture.contract}"`));
});
