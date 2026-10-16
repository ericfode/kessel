'use strict';
const {compileDoubleSidedLeaderContract}=require('./double-sided-leader-contract');
const {compileNativeTokenLifecycle}=require('./native-token-lifecycle');
const {compileNativeTokenGraph}=require('./native-token-graphs');
const {compileTokenInheritance}=require('./native-token-base');

const fs = require('fs');
const path = require('path');
const { SCHEMA, VERSION, hash, compileSource, loadEnums, ts } = require('./compiler');
const { parseKeywords } = require('./keywords');
const { isPlainUpgrade } = require('./plain-upgrade');
const { splitCatalog, loadCatalog } = require('./load-catalog');

const repository = path.resolve(__dirname, '..');
const codeFor = (card) => card.setId?.number == null ? `TOKEN_${card.internalName.replace(/[^a-z0-9]+/gi, '_').toUpperCase()}` : `${card.setId.set}_${String(card.setId.number).padStart(3, '0')}`;
function files(root, extension) {
    return fs.readdirSync(root, { recursive: true }).filter((f) => f.endsWith(extension)).sort();
}
function requiredCapabilities(abilities, card, attachCondition) {
    const capabilities = new Set();
    const walk = (value) => {
        if (Array.isArray(value)) return value.forEach(walk);
        if (!value || typeof value !== 'object') return;
        if (value.trigger) capabilities.add(`trigger:${value.trigger}`);
        if (value.kind) capabilities.add(`ir:${value.kind}`);
        if (value.kind === 'query' || value.kind === 'collection') capabilities.add(`${value.kind}:${value.name}`);
        if (value.kind === 'binary' || value.kind === 'unary') capabilities.add(`${value.kind}:${value.op}`);
        if (value.kind === 'ref') capabilities.add(`ref:${value.path.join('.')}`);
        if (value.targetResolver) {
            if(value.targetResolver.type)capabilities.add(`targetType:${value.targetResolver.type}`);
            if(value.targetResolver.zone)capabilities.add(`targetZone:${value.targetResolver.zone}`);
            if (value.targetResolver.mode) capabilities.add(`targetMode:${value.targetResolver.mode}`);
        }
        for (const [k, v] of Object.entries(value)) {
            if (['then', 'ifYouDo', 'thenCondition', 'ifYouDoCondition', 'optionalMakesAbilityNonCancelable'].includes(k)) capabilities.add(`ability:${k}`);
            walk(v);
        }
    };
    walk(abilities);
    walk(attachCondition);
    for (const keyword of card.keywords ?? []) capabilities.add(`keyword:${keyword}`);
    for (const type of card.types ?? []) capabilities.add(`cardType:${type}`);
    return [...capabilities].sort();
}
function keywordValues(card) {
    const values = {};
    const parsedKeywords = new Map(['Sentinel', 'Raid', 'Overwhelm', 'Saboteur', 'Restore', 'Ambush', 'Grit', 'Shielded', 'Coordinate', 'Hidden', 'Support'].map(name => [name.toLowerCase(), name]));
    // Card's constructor parses text for units; leaders parse deployBox and
    // text separately. Metadata supplies expected names, including mentions
    // that do not become intrinsic keywords without an own-line text match.
    const texts = card.types?.includes('leader') ? [card.deployBox, card.text] : [card.text];
    for (const keyword of new Set(card.keywords ?? [])) {
        const name = parsedKeywords.get(keyword);
        if (!name) continue; // Unsupported printed kinds still fail runtime admission.
        const numeric = keyword === 'raid' || keyword === 'restore';
        for (const text of texts) {
            const regex = new RegExp(`(?:^|\\n)${name}${numeric ? ' ([0-9]+)' : ''}`, 'g');
            const matches = [...String(text ?? '').matchAll(regex)];
            if (matches.length > 1) throw new Error(`Multiple printed ${keyword} instances in one side of ${card.internalName ?? 'card'}`);
            if (!matches.length) continue;
            const amount = numeric ? Number(matches[0][1]) : 1;
            values[keyword] = numeric ? (values[keyword] ?? 0) + amount : 1;
        }
    }
    return values;
}

function compileCatalog({ source = path.join(repository, 'forceteki/server'), cards = path.join(repository, 'forceteki/test/json') } = {}) {
    const sourceDir = path.join(source, 'game/cards');
    const cardDir = path.join(cards, 'Card');
    if (!fs.existsSync(sourceDir)) throw new Error(`Missing engine card source: ${sourceDir}; initialize the pinned submodule or pass --source`);
    if (!fs.existsSync(cardDir)) throw new Error(`Missing downloaded card data: ${cardDir}; run npm --prefix forceteki run get-cards`);
    const enums = loadEnums(source);
    const sharedClassContracts=new Map();
    const doubleSided=compileDoubleSidedLeaderContract(source,enums);if(doubleSided)sharedClassContracts.set(doubleSided.className,doubleSided);
    const sharedBases=new Map();
    for(const relative of ['05_LOF/common/LofCommonForceBase.ts','07_LAW/common/LAWCommonBase.ts']) {
        const sharedPath=path.join(sourceDir,relative),sourcePath=path.posix.join('game/cards',relative);
        if(!fs.existsSync(sharedPath))continue;
        const definition=compileSource(fs.readFileSync(sharedPath,'utf8'),sourcePath,enums,{abstractDefinition:true});
        sharedBases.set(definition.className,definition);
    }
    const sources = [];
    const byId = new Map();
    const unmatchedSources = [];
    // The compiler's defaults depend on engine helper semantics as well as card
    // registrations. Hash server TypeScript and JavaScript, including legacy
    // prompt/socket helpers that participate in the reference engine runtime.
    const inputs = [...files(source, '.ts'), ...files(source, '.js')].sort().map((relative) => ({ file: `server/${relative.split(path.sep).join('/')}`, sha256: hash(fs.readFileSync(path.join(source, relative))) }));
    for (const relative of files(sourceDir, '.ts')) {
        const file = path.join(sourceDir, relative);
        const text = fs.readFileSync(file, 'utf8');
        const sourcePath = path.posix.join('game/cards', relative.split(path.sep).join('/'));
        if (relative === 'Index.ts' || relative.split(path.sep).includes('common')) {
            unmatchedSources.push({ sourcePath, sourceHash: hash(text), role: relative === 'Index.ts' ? 'registry' : 'shared-base' });
            continue;
        }
        let converted;
        try { converted = compileSource(text, sourcePath, enums, {sharedBases,sharedClassContracts}); }
        catch (error) {
            // A compiler defect is a build failure, never an "unsupported" card.
            throw new Error(`Compiler failed on ${sourcePath}: ${error.stack || error}`);
        }
        const lifecycle=require('./source-exclusions').sourceExclusion(sourcePath)?null:compileNativeTokenLifecycle(text,sourcePath,source,enums)||compileNativeTokenGraph(text,sourcePath,source,enums,compileTokenInheritance);
        if(lifecycle) {
            unmatchedSources.push(lifecycle);
            // A complete staged lifecycle can recover identity for a native file
            // with multiple classes. It does not replace the active rejection or
            // promote that card to executable/support status.
            if(!converted.id && lifecycle.role==='staged-native-token-lifecycle' && lifecycle.status==='lifecycle-complete'
                && lifecycle.sourcePath===converted.sourcePath && lifecycle.sourceHash===converted.sourceHash
                && typeof lifecycle.identity?.id==='string' && typeof lifecycle.identity?.internalName==='string') {
                converted={...converted,id:lifecycle.identity.id,internalName:lifecycle.identity.internalName};
            }
        }
        sources.push(converted);
        if (converted.id) {
            if (byId.has(converted.id)) throw new Error(`Duplicate source identity ${converted.id}: ${sourcePath}`);
            byId.set(converted.id, converted);
        }
    }
    const mockFile = path.join(cards, '_mockCardNames.json');
    const mockText = fs.existsSync(mockFile) ? fs.readFileSync(mockFile, 'utf8') : '[]';
    const mocks = new Set(JSON.parse(mockText));
    inputs.push({ file: 'data/_mockCardNames.json', sha256: hash(mockText) });
    const definitions = [];
    let mockCount = 0;
    const sourceUsed = new Set();
    const setCodes = new Set();
    for (const file of files(cardDir, '.json')) {
        const text = fs.readFileSync(path.join(cardDir, file), 'utf8');
        inputs.push({ file: `data/Card/${file}`, sha256: hash(text) });
        const card = JSON.parse(text);
        if (mocks.has(card.internalName)) { mockCount++; continue; }
        const setCode = codeFor(card);
        if (setCodes.has(setCode)) throw new Error(`Duplicate card set code ${setCode}`);
        setCodes.add(setCode);
        const found = byId.get(card.id);
        let compiled;
        if (found) {
            sourceUsed.add(found.id);
            compiled = structuredClone(found);
            if (found.internalName !== card.internalName) throw new Error(`Source/data identity mismatch: ${found.sourcePath} (${found.internalName}) vs ${card.internalName}`);
        } else {
            const keywordsOnly = parseKeywords(card);
            const plainBase = card.types.includes('base') && !card.text && !card.epicAction;
            const vanilla = card.types.includes('unit') && keywordsOnly !== null && !card.types.includes('leader');
            const plainUpgrade = isPlainUpgrade(card);
            const supported = plainBase || vanilla || plainUpgrade;
            compiled = { id: card.id, internalName: card.internalName, ...(plainUpgrade ? { baseClass: 'UpgradeCard' } : {}), sourcePath: null, sourceHash: null, status: supported ? 'supported' : 'unsupported', abilities: { leader: [], unit: [], card: [] }, diagnostics: supported ? [] : [{ code: 'MISSING_IMPLEMENTATION', pattern: 'no-source-for-rules-text', file: `data/Card/${file}`, line: 1, column: 1, message: 'No source implementation and card is not a plain base or supported keyword-only unit' }] };
        }
        const definition = { setCode, ...compiled, card, dataHash: hash(text), keywordValues: keywordValues(card) };
        definition.requiredCapabilities = requiredCapabilities(definition.abilities, card, definition.attachCondition);
        definitions.push(definition);
    }
    definitions.sort((a, b) => a.setCode.localeCompare(b.setCode));
    for (const source of sources) if (!sourceUsed.has(source.id)) unmatchedSources.push({ sourcePath: source.sourcePath, sourceHash: source.sourceHash, role: 'source-without-card-data', id: source.id, internalName: source.internalName, status: source.status, diagnostics: source.diagnostics });
    inputs.sort((a, b) => a.file.localeCompare(b.file));
    // Hash compiler implementation only — generated output and tests must not feedback into provenance.
    const compilerFiles = files(__dirname, '.js').filter((f) => !f.startsWith('generated' + path.sep) && !f.startsWith('test' + path.sep)).map((file) => ({ file, sha256: hash(fs.readFileSync(path.join(__dirname, file))) }));
    const inputHash = hash(JSON.stringify(inputs));
    const compilerHash = hash(JSON.stringify(compilerFiles));
    return { schema: SCHEMA, compilerVersion: VERSION, typescriptVersion: ts.version, sourceTreeHash: inputHash, compilerHash, inputs, sourceSupport: unmatchedSources, cards: definitions, inventory: { knownCards: definitions.length, mockCardsExcluded: mockCount, sourceFiles: files(sourceDir, '.ts').length, sourceDefinitions: sources.length, definitionsWithSource: sourceUsed.size } };
}
function coverage(catalog) {
    const patterns = new Map();
    const unsupported = [];
    const converted = [];
    for (const card of catalog.cards) {
        if (card.status === 'supported') converted.push(card.setCode);
        else {
            unsupported.push({ setCode: card.setCode, internalName: card.internalName, sourcePath: card.sourcePath, diagnostics: card.diagnostics });
            for (const diagnostic of card.diagnostics) {
                const entry = patterns.get(diagnostic.pattern) || { pattern: diagnostic.pattern, occurrences: 0, cards: [] };
                entry.occurrences++;
                if (!entry.cards.includes(card.setCode)) entry.cards.push(card.setCode);
                patterns.set(entry.pattern, entry);
            }
        }
    }
    return { schema: 'swu-card-compiler-coverage/v1', sourceTreeHash: catalog.sourceTreeHash, compilerHash: catalog.compilerHash, ...catalog.inventory, converted: converted.length, sourceAstConverted: catalog.cards.filter((card) => card.status === 'supported' && card.sourcePath !== null).length, dataOnlyConverted: catalog.cards.filter((card) => card.status === 'supported' && card.sourcePath === null).length, unconverted: unsupported.length, runtimeReady: null, runtimeReadyNote: 'Source compilation is independent of runtime capability validation. The Rust importer validates each operation before accepting a card; this report does not claim executable coverage.', convertedCards: converted, blockers: [...patterns.values()].sort((a, b) => b.cards.length - a.cards.length || a.pattern.localeCompare(b.pattern)), unsupportedCards: unsupported };
}
function reportMarkdown(report) {
    return ['# Card source conversion coverage', '', `Known card definitions: **${report.knownCards}**. IR-converted definitions: **${report.converted}** (${report.sourceAstConverted} from source AST; ${report.dataOnlyConverted} metadata-only plain cards). Explicitly blocked definitions: **${report.unconverted}**. Excluded test mocks: ${report.mockCardsExcluded}.`, '', 'These counts measure bounded IR conversion, not runtime acceptance. Metadata-only conversion handles plain bases, fully parsed keyword-only units, and plain stat-only upgrades using the generic UpgradeCard fallback. The Rust runtime separately validates its supported operation set. Unsupported definitions retain their source locations and must not be loaded as no-op cards.', '', '| Unsupported source pattern | Affected cards | Occurrences |', '|---|---:|---:|', ...report.blockers.map((b) => `| ${b.pattern.replaceAll('|', '\\|')} | ${b.cards.length} | ${b.occurrences} |`), '', 'Full card-by-card diagnostics and converted IDs are in [coverage.json](coverage.json). Every discovered card is represented across the concern-split catalogue shards (`manifest.json`, `cards.json`, `diagnostics.json`, `inputs.json`).', ''].join('\n');
}
function args(argv) {
    const options = { source: path.join(repository, 'forceteki/server'), cards: path.join(repository, 'forceteki/test/json'), out: path.join(__dirname, 'generated'), check: false, strict: false };
    for (let i = 0; i < argv.length; i++) {
        const flag = argv[i];
        if (flag === '--check') options.check = true;
        else if (flag === '--strict') options.strict = true;
        else if (['--source', '--cards', '--out'].includes(flag) && argv[i + 1] && !argv[i + 1].startsWith('--')) options[flag.slice(2)] = path.resolve(argv[++i]);
        else throw new Error(`Unknown or incomplete argument ${flag}`);
    }
    return options;
}
function catalogOutputs(catalog, report) {
    const shards = splitCatalog(catalog);
    const outputs = {};
    for (const [name, value] of Object.entries(shards)) outputs[name] = JSON.stringify(value, null, 2) + '\n';
    outputs['coverage.json'] = JSON.stringify(report, null, 2) + '\n';
    outputs['coverage.md'] = reportMarkdown(report);
    return outputs;
}

function main() {
    const options = args(process.argv.slice(2));
    const catalog = compileCatalog(options);
    const report = coverage(catalog);
    const outputs = catalogOutputs(catalog, report);
    // Round-trip guard: shards must rebuild the exact in-memory catalogue.
    const assembled = (() => {
        const temporary = fs.mkdtempSync(path.join(require('os').tmpdir(), 'swu-catalog-assemble-'));
        try {
            for (const [name, content] of Object.entries(outputs)) {
                if (!name.endsWith('.json') || name.startsWith('coverage')) continue;
                fs.writeFileSync(path.join(temporary, name), content);
            }
            return loadCatalog(temporary);
        } finally {
            fs.rmSync(temporary, { recursive: true, force: true });
        }
    })();
    // Field order on card objects may differ (diagnostics is reattached last); values must match.
    const { canonical } = require('../cards/inventory');
    if (JSON.stringify(canonical(assembled)) !== JSON.stringify(canonical(catalog))) {
        throw new Error('Concern-split catalogue shards do not round-trip to the unified catalogue');
    }
    if (options.check) {
        for (const [name, content] of Object.entries(outputs)) {
            const file = path.join(options.out, name);
            if (!fs.existsSync(file) || fs.readFileSync(file, 'utf8') !== content) throw new Error(`Stale generated card compiler artifact: ${file}; run node compiler/build.js`);
        }
        if (fs.existsSync(path.join(options.out, 'catalog.json'))) {
            throw new Error(`Legacy monolith still present: ${path.join(options.out, 'catalog.json')}; run node compiler/build.js`);
        }
    } else {
        fs.mkdirSync(options.out, { recursive: true });
        for (const [name, content] of Object.entries(outputs)) fs.writeFileSync(path.join(options.out, name), content);
        const legacy = path.join(options.out, 'catalog.json');
        if (fs.existsSync(legacy)) fs.unlinkSync(legacy);
    }
    console.log(JSON.stringify({ known: report.knownCards, converted: report.converted, unconverted: report.unconverted, runtimeReady: report.runtimeReady, sourceFiles: report.sourceFiles, sourceHash: report.sourceTreeHash, compilerHash: report.compilerHash, checked: options.check, layout: 'concern-split' }));
    if (options.strict && report.unconverted) { console.error(`Strict conversion incomplete: ${report.unconverted} cards have explicit blockers`); process.exitCode = 2; }
}
if (require.main === module) { try { main(); } catch (error) { console.error(error.stack || String(error)); process.exitCode = 1; } }
module.exports = { compileCatalog, coverage, requiredCapabilities, keywordValues, codeFor, args, catalogOutputs, splitCatalog, loadCatalog };
