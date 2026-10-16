'use strict';

// Assembles the unified swu-card-ir/v1 catalogue from concern-split shards.
// Accepts a legacy monolith file or a generated/ directory (or its manifest.json).
const fs = require('node:fs');
const path = require('node:path');

const MANIFEST_SCHEMA = 'swu-card-ir-manifest/v1';
const INPUTS_SCHEMA = 'swu-card-ir-inputs/v1';
const DIAGNOSTICS_SCHEMA = 'swu-card-diagnostics/v1';
const CARDS_SCHEMA = 'swu-card-ir-cards/v1';
const IR_SCHEMA = 'swu-card-ir/v1';

const GENERATED = path.resolve(__dirname, 'generated');

function readJson(file) {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isDirectoryCatalog(target) {
    if (fs.existsSync(target) && fs.statSync(target).isDirectory()) return true;
    if (path.basename(target) === 'manifest.json' && fs.existsSync(target)) return true;
    return false;
}

function catalogDir(target = GENERATED) {
    if (path.basename(target) === 'manifest.json') return path.dirname(target);
    return target;
}

/** Split a unified catalogue object into byte-stable concern shards. */
function splitCatalog(catalog) {
    if (catalog.schema !== IR_SCHEMA) throw new Error(`splitCatalog expects ${IR_SCHEMA}`);
    const diagnosticsCards = [];
    const cards = catalog.cards.map((entry) => {
        const { diagnostics, ...rest } = entry;
        if (Array.isArray(diagnostics) && diagnostics.length) {
            diagnosticsCards.push({ setCode: entry.setCode, diagnostics });
        }
        return rest;
    });
    return {
        'manifest.json': {
            schema: MANIFEST_SCHEMA,
            irSchema: IR_SCHEMA,
            compilerVersion: catalog.compilerVersion,
            typescriptVersion: catalog.typescriptVersion,
            sourceTreeHash: catalog.sourceTreeHash,
            compilerHash: catalog.compilerHash,
            sourceSupport: catalog.sourceSupport,
            inventory: catalog.inventory,
            files: {
                cards: 'cards.json',
                inputs: 'inputs.json',
                diagnostics: 'diagnostics.json',
            },
        },
        'inputs.json': {
            schema: INPUTS_SCHEMA,
            compilerHash: catalog.compilerHash,
            sourceTreeHash: catalog.sourceTreeHash,
            inputs: catalog.inputs,
        },
        'cards.json': {
            schema: CARDS_SCHEMA,
            irSchema: IR_SCHEMA,
            compilerHash: catalog.compilerHash,
            sourceTreeHash: catalog.sourceTreeHash,
            cards,
        },
        'diagnostics.json': {
            schema: DIAGNOSTICS_SCHEMA,
            compilerHash: catalog.compilerHash,
            sourceTreeHash: catalog.sourceTreeHash,
            cards: diagnosticsCards,
        },
    };
}

/** Reassemble shards (or a legacy monolith) into the unified catalogue object. */
function loadCatalog(target = GENERATED) {
    const resolved = path.resolve(target);
    if (!isDirectoryCatalog(resolved)) {
        const value = readJson(resolved);
        if (value.schema !== IR_SCHEMA || !Array.isArray(value.cards)) {
            throw new Error(`Not a ${IR_SCHEMA} catalogue: ${resolved}`);
        }
        return value;
    }
    const dir = catalogDir(resolved);
    const manifest = readJson(path.join(dir, 'manifest.json'));
    if (manifest.schema !== MANIFEST_SCHEMA) throw new Error(`Bad catalogue manifest schema in ${dir}`);
    const files = manifest.files || {};
    const cardsFile = readJson(path.join(dir, files.cards || 'cards.json'));
    const inputsFile = readJson(path.join(dir, files.inputs || 'inputs.json'));
    const diagnosticsFile = readJson(path.join(dir, files.diagnostics || 'diagnostics.json'));
    if (cardsFile.schema !== CARDS_SCHEMA) throw new Error(`Bad cards shard schema in ${dir}`);
    if (inputsFile.schema !== INPUTS_SCHEMA) throw new Error(`Bad inputs shard schema in ${dir}`);
    if (diagnosticsFile.schema !== DIAGNOSTICS_SCHEMA) throw new Error(`Bad diagnostics shard schema in ${dir}`);
    for (const [label, hash] of [
        ['cards', cardsFile.compilerHash],
        ['inputs', inputsFile.compilerHash],
        ['diagnostics', diagnosticsFile.compilerHash],
    ]) {
        if (hash !== manifest.compilerHash) throw new Error(`${label} shard compilerHash mismatch in ${dir}`);
    }
    const byCode = new Map(diagnosticsFile.cards.map((entry) => [entry.setCode, entry.diagnostics]));
    const cards = cardsFile.cards.map((entry) => ({
        ...entry,
        diagnostics: byCode.get(entry.setCode) ?? [],
    }));
    return {
        schema: IR_SCHEMA,
        compilerVersion: manifest.compilerVersion,
        typescriptVersion: manifest.typescriptVersion,
        sourceTreeHash: manifest.sourceTreeHash,
        compilerHash: manifest.compilerHash,
        inputs: inputsFile.inputs,
        sourceSupport: manifest.sourceSupport,
        cards,
        inventory: manifest.inventory,
    };
}

/** Deterministic pretty bytes — same shape the old catalog.json used. */
function catalogBytes(target = GENERATED) {
    return Buffer.from(JSON.stringify(loadCatalog(target), null, 2) + '\n', 'utf8');
}

/** Compact JSON text for Wasm / Engine loaders that want a single string. */
function catalogJsonText(target = GENERATED) {
    return JSON.stringify(loadCatalog(target));
}

/** Resolve a --catalogue path: directory, manifest, or legacy monolith file. */
function resolveCatalogueTarget(target) {
    if (target == null) return GENERATED;
    return path.resolve(target);
}

function defaultGenerated() {
    return GENERATED;
}

module.exports = {
    MANIFEST_SCHEMA,
    INPUTS_SCHEMA,
    DIAGNOSTICS_SCHEMA,
    CARDS_SCHEMA,
    IR_SCHEMA,
    splitCatalog,
    loadCatalog,
    catalogBytes,
    catalogJsonText,
    catalogDir,
    isDirectoryCatalog,
    resolveCatalogueTarget,
    defaultGenerated,
};
