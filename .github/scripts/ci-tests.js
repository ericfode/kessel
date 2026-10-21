'use strict';

// Ordinary gates defer two named whole-corpus bodies, not their targets or files.
// Keep the default Cargo/Node discovery domains: new tests must run or fail red.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');

const rustCheckpoint = 'every_reference_matches_current_programs_and_immutable_engine_recordings';
const wasmCheckpoint = 'all reference cases match current Wasm programs and immutable engine recordings';
const referenceFile = 'crates/kessel-wasm/js/test/reference-conformance.test.js';
const terminalCase = 'auto/ASH_031/attack-base-history-owner-0-hera-grievous-capped-defender-departure';

function run(command, args, root, { capture = false, env = process.env } = {}) {
    console.log(`> ${command} ${args.map(a => JSON.stringify(a)).join(' ')}`);
    const result = spawnSync(command, args, { cwd: root, env, encoding: 'utf8', maxBuffer: 128 * 1024 * 1024 });
    if (result.stderr) process.stderr.write(result.stderr);
    if (!capture && result.stdout) process.stdout.write(result.stdout);
    if (result.error) throw result.error;
    assert.equal(result.status, 0, `${command} failed (${result.status ?? result.signal})${capture ? '\n' + result.stdout : ''}`);
    return result.stdout;
}
function validateCargoTarget(target) {
    if (target.kind.length && target.kind.every(kind => ['lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'].includes(kind))) return;
    if (target.kind.length === 1 && ['bin', 'test', 'example', 'bench'].includes(target.kind[0])) return;
    throw new Error(`Unknown runnable Cargo target: ${JSON.stringify(target)}`);
}
function targetKey(packageId, target) {
    return JSON.stringify([packageId, target.name, target.kind, path.resolve(target.src_path)]);
}
function rustPlan(root, metadata, artifacts) {
    const members = new Set(metadata.workspace_members);
    const packages = metadata.packages.filter(p => members.has(p.id));
    const targets = new Map(packages.flatMap(p => p.targets.map(t => [targetKey(p.id, t), { package: p, target: t }])));
    const checkpoint = [...targets.values()].filter(({ package: p, target: t }) =>
        path.resolve(p.manifest_path) === path.join(root, 'crates/kessel/Cargo.toml') &&
        t.name === 'reference_conformance' && t.kind.length === 1 && t.kind[0] === 'test' &&
        path.resolve(t.src_path) === path.join(root, 'crates/kessel/tests/reference_conformance.rs'));
    assert.equal(checkpoint.length, 1, 'Expected exactly one known Rust checkpoint target');
    const runnable = new Map();
    for (const a of artifacts) {
        if (a.reason !== 'compiler-artifact' || !members.has(a.package_id) || !a.profile?.test || !a.executable) continue;
        const key = targetKey(a.package_id, a.target), known = targets.get(key);
        assert.ok(known, `Unrecognized workspace test artifact: ${key}`);
        validateCargoTarget(known.target); // Validate every kind, not just the checkpoint.
        if (runnable.has(key)) assert.equal(runnable.get(key).executable, a.executable, 'Ambiguous Cargo test artifact');
        runnable.set(key, { ...known, executable: a.executable });
    }
    for (const [key, { target }] of targets) {
        // Cargo legitimately omits targets whose required features are disabled.
        if (target.test && !(target['required-features']?.length)) assert.ok(runnable.has(key), `Cargo omitted an expected test target: ${key}`);
    }
    const special = targetKey(checkpoint[0].package.id, checkpoint[0].target);
    assert.ok(runnable.has(special), 'Rust checkpoint target was not compiled');
    return [...runnable.entries()].map(([key, a]) => ({
        checkpoint: key === special, executable: a.executable,
    }));
}
function runRust(root) {
    root = fs.realpathSync(root);
    const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], root, { capture: true }));
    // Cargo itself decides the default build set, including compile-only examples.
    const output = run('cargo', ['test', '--locked', '--workspace', '--no-run', '--message-format=json'], root, { capture: true });
    const messages = output.split(/\r?\n/).filter(Boolean).map(line => JSON.parse(line));
    for (const message of messages) if (message.reason === 'compiler-message' && message.message?.rendered) process.stderr.write(message.message.rendered);
    const plan = rustPlan(root, metadata, messages);
    const checkpoint = plan.filter(target => target.checkpoint);
    assert.equal(checkpoint.length, 1, 'Ambiguous compiled Rust checkpoint target');
    const identity = run(checkpoint[0].executable, ['--list', '--format', 'terse'], root, { capture: true });
    const count = listed => listed.split(/\r?\n/).filter(line => line === rustCheckpoint + ': test').length;
    assert.equal(count(identity), 1, 'Missing or ambiguous Rust checkpoint test identity');
    // Listing the default workspace executes no bodies and includes doctests.
    // Refuse a collision rather than silently skipping the same name elsewhere.
    const listed = run('cargo', ['test', '--locked', '--workspace', '--', '--list', '--format', 'terse'], root, { capture: true });
    assert.equal(count(listed), 1, 'Rust checkpoint test name collides with another workspace test');
    run('cargo', ['test', '--locked', '--workspace', '--', '--skip', rustCheckpoint, '--exact'], root);
    console.log(`Ordinary Rust: default workspace harnesses/doctests; one unique named whole-corpus body deferred.`);
}

function wasmFiles(root) {
    const directory = path.join(root, 'crates/kessel-wasm/js/test');
    const files = ['crates/kessel-wasm/test.js', ...fs.readdirSync(directory).filter(n => !n.startsWith('.') && n.endsWith('.test.js')).sort().map(n => 'crates/kessel-wasm/js/test/' + n)];
    for (const file of files) assert.ok(fs.statSync(path.join(root, file)).isFile(), `Not a test file: ${file}`);
    assert.equal(files.filter(f => f === referenceFile).length, 1, 'Missing Wasm checkpoint test file');
    return files;
}
function terminalEnvironment(root, env) {
    const names = ['KESSEL_TERMINAL_TRACE', 'KESSEL_TERMINAL_POOL', 'KESSEL_TERMINAL_CATALOGUE', 'KESSEL_TERMINAL_PACKAGE'];
    if (names.some(name => env[name])) {
        for (const name of names) assert.ok(env[name] && fs.existsSync(env[name]), `Explicit terminal input is missing: ${name}`);
        return env;
    }
    const lib = require(path.join(root, 'cards/reference/library'));
    const rows = lib.corpus().recordings.filter(row => row.caseId === terminalCase);
    assert.equal(rows.length, 1, 'Expected one reviewed terminal-continuation recording');
    assert.ok(rows[0].witness?.terminalCompletion?.postWinnerChoices > 0, 'Selected terminal case needs committed postwinner choices');
    const selected = { ...env,
        KESSEL_TERMINAL_TRACE: lib.checkedArtifact(lib.defaultCorpus, 'trace', rows[0].traceSha256),
        KESSEL_TERMINAL_POOL: lib.checkedArtifact(lib.defaultCorpus, 'pool', rows[0].poolSha256),
        KESSEL_TERMINAL_CATALOGUE: path.join(root, 'compiler/generated'),
        KESSEL_TERMINAL_PACKAGE: path.join(root, 'crates/kessel-wasm/pkg/node/kessel_wasm.js'),
    };
    for (const name of names) assert.ok(fs.existsSync(selected[name]), `Terminal input unavailable: ${name}`);
    return selected;
}
// Node 22 filters names out of TAP rather than listing skipped registrations.
// Evaluate this one closed CommonJS registration envelope without running bodies.
function wasmRegistrations(root) {
    const file = path.join(root, referenceFile), names = [];
    const allowed = new Set(['node:test', 'node:assert/strict', 'node:path', '../../../../cards/reference/library', '../node', '../check']);
    let registration;
    const declare = (name, ...args) => {
        assert.equal(typeof name, 'string', 'Unknown Wasm test registration name');
        assert.ok(args.some(arg => typeof arg === 'function'), 'Unknown Wasm test registration shape');
        names.push(name);
    };
    registration = new Proxy(declare, { get(_target, key) {
        if (key === 'test' || key === 'default') return registration;
        throw new Error('Unknown Wasm registration harness member: ' + String(key));
    } });
    const load = require('node:module').createRequire(file), module = { exports: {} };
    const sandbox = { module, exports: module.exports, __filename: file, __dirname: path.dirname(file),
        require(name) {
            assert.ok(allowed.has(name), 'Unknown Wasm checkpoint registration import: ' + name);
            return name === 'node:test' ? registration : load(name);
        } };
    require('node:vm').runInNewContext(fs.readFileSync(file, 'utf8'), sandbox, {
        filename: file, timeout: 1000, microtaskMode: 'afterEvaluate',
    });
    return names;
}
function runWasm(root, env = process.env) {
    root = fs.realpathSync(root);
    const files = wasmFiles(root);
    assert.equal(wasmRegistrations(root).filter(name => name === wasmCheckpoint).length, 1,
        'Missing or ambiguous Wasm checkpoint test identity');
    const testEnv = files.includes('crates/kessel-wasm/js/test/terminal-actor.test.js') ? terminalEnvironment(root, env) : env;
    const tested = run(process.execPath, ['--test', '--test-reporter=tap', ...files.filter(file => file !== referenceFile)], root, { env: testEnv });
    if (files.includes('crates/kessel-wasm/js/test/terminal-actor.test.js')) {
        const terminal = tested.split(/\r?\n/).filter(line => /^ok [0-9]+ - terminal resolver actor ingress survives Wasm checkpoint and restore(?:$| #)/.test(line));
        assert.equal(terminal.length, 1, 'Missing or ambiguous terminal-actor result');
        assert.doesNotMatch(terminal[0], /# (?:SKIP|TODO)/, 'Terminal ingress requires a non-skipped real recording');
    }
    // Scope this exact name to its file: names in other files are never waived.
    const escaped = wasmCheckpoint.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    run(process.execPath, ['--test', '--test-skip-pattern=^' + escaped + '$', referenceFile], root, { env: testEnv });
    console.log(`Ordinary Wasm: ${files.length} files; one named whole-corpus body deferred.`);
}
function main(mode, root = path.resolve(__dirname, '../..')) {
    assert.ok(Number(process.versions.node.split('.')[0]) >= 22, 'CI gates require Node 22 or newer');
    if (mode === 'rust') return runRust(root);
    if (mode === 'wasm') return runWasm(root);
    throw new Error('Use ci-tests.js rust or ci-tests.js wasm');
}
if (require.main === module) {
    try {
        assert.equal(process.argv.length, 3, 'Use ci-tests.js rust or ci-tests.js wasm; no extra options');
        main(process.argv[2]);
    } catch (error) { console.error(error); process.exitCode = 1; }
}
module.exports = { rustCheckpoint, wasmCheckpoint, rustPlan, wasmFiles, terminalEnvironment, runRust, runWasm, wasmRegistrations, main };
