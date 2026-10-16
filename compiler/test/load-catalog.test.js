'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { compileCatalog } = require('../build');
const { splitCatalog, loadCatalog, catalogBytes } = require('../load-catalog');

test('concern-split shards round-trip to the unified catalogue', () => {
    const catalog = compileCatalog();
    const shards = splitCatalog(catalog);
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'swu-load-catalog-'));
    try {
        for (const [name, value] of Object.entries(shards)) {
            fs.writeFileSync(path.join(dir, name), JSON.stringify(value, null, 2) + '\n');
        }
        const { canonical } = require('../../cards/inventory');
        assert.deepEqual(canonical(loadCatalog(dir)), canonical(catalog));
        assert.equal(
            JSON.stringify(canonical(JSON.parse(catalogBytes(dir).toString('utf8')))),
            JSON.stringify(canonical(catalog)),
        );
        // Legacy monolith path still loads.
        const legacy = path.join(dir, 'legacy.json');
        fs.writeFileSync(legacy, JSON.stringify(catalog));
        assert.deepEqual(canonical(loadCatalog(legacy)), canonical(catalog));
    } finally {
        fs.rmSync(dir, { recursive: true, force: true });
    }
});
