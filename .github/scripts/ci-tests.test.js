'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawnSync } = require('node:child_process');
const gates = require('./ci-tests');
const script = path.join(__dirname, 'ci-tests.js');
function write(root, file, text) { const p = path.join(root, file); fs.mkdirSync(path.dirname(p), { recursive: true }); fs.writeFileSync(p, text); }
function invoke(mode, root, env = {}) {
    const childEnv = { ...process.env, ...env };
    if (mode === 'rust') childEnv.CARGO_TARGET_DIR = path.join(root, 'target');
    delete childEnv.NODE_TEST_CONTEXT; // A fresh subprocess runner, not recursive node:test.
    return spawnSync(process.execPath, ['-e', `require(${JSON.stringify(script)}).main(${JSON.stringify(mode)},${JSON.stringify(root)})`],
        { encoding: 'utf8', env: childEnv, maxBuffer: 32 * 1024 * 1024 });
}
function success(result) { assert.equal(result.status, 0, result.stdout + result.stderr); }
function lines(file) { return fs.existsSync(file) ? fs.readFileSync(file, 'utf8').trim().split(/\r?\n/) : []; }
const markRust = name => `{use std::io::Write;writeln!(std::fs::OpenOptions::new().create(true).append(true).open(std::env::var("CI_TEST_LOG").unwrap()).unwrap(),"${name}").unwrap();}`;
const markNode = name => `require('node:fs').appendFileSync(process.env.CI_TEST_LOG,${JSON.stringify(name + '\n')});`;

test('real Cargo discovery retains workspace libraries, binaries, new integrations, doctests and exact skip siblings', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ordinary cargo ')), log = path.join(root, 'ran.txt');
    try {
        write(root, 'Cargo.toml', '[workspace]\nmembers=["crates/kessel","future"]\nresolver="2"\n');
        write(root, 'crates/kessel/Cargo.toml', '[package]\nname="kessel"\nversion="0.0.0"\nedition="2021"\n[features]\nenhanced=[]\n');
        write(root, 'future/Cargo.toml', '[package]\nname="future"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nkessel={path="../crates/kessel",features=["enhanced"]}\n');
        write(root, 'future/src/lib.rs', `#[test] fn future_package() ${markRust('future-package')}`);
        write(root, 'crates/kessel/src/lib.rs', `/// \`\`\`\n/// ${markRust('doctest')}\n/// \`\`\`\npub fn documented() {}\n#[test] fn workspace_feature_union() {assert!(cfg!(feature="enhanced"));${markRust('workspace-features')}}\n#[test] fn ordinary_lib() ${markRust('lib')}\n#[cfg(test)] mod generic_runtime_tests { #[test] fn generated_programs_match_all_committed_engine_traces() ${markRust('fixed-fixtures')} }`);
        write(root, 'crates/kessel/src/main.rs', `fn main() {} #[test] fn ordinary_binary() ${markRust('bin')}`);
        write(root, 'crates/kessel/examples/quiet.rs', 'fn main() {}');
        const reference = `#[test] fn ${gates.rustCheckpoint}() { panic!("HEAVY_BODY_EXECUTED"); }\n#[test] fn ${gates.rustCheckpoint}_suffix() ${markRust('exact-suffix')}\n#[test] fn reference_manifest_reader_accepts_both_formats_and_rejects_damage() ${markRust('reader')}`;
        write(root, 'crates/kessel/tests/reference_conformance.rs', reference);
        write(root, 'crates/kessel/tests/future.rs', `#[test] fn ordinary_future_target() ${markRust('future-target')}\n#[test] fn new_integration() {assert!(std::env::var("CI_FAIL_FUTURE").is_err());${markRust('future-integration')}}`);
        const lock = spawnSync('cargo', ['generate-lockfile', '--offline'], { cwd: root, encoding: 'utf8' }); success(lock);
        const result = invoke('rust', root, { CI_TEST_LOG: log }); success(result);
        const ran = lines(log);
        for (const name of ['workspace-features', 'lib', 'fixed-fixtures', 'bin', 'reader', 'exact-suffix', 'future-target', 'future-integration', 'future-package', 'doctest']) assert.ok(ran.includes(name), name + '\n' + result.stdout);
        assert.equal(ran.filter(name => name === 'doctest').length, 1, 'Preflight listing must not execute doctest bodies');
        assert.ok(fs.existsSync(path.join(root, 'target/debug/examples/quiet')), 'Cargo default compile-only example retained');
        assert.doesNotMatch(result.stdout + result.stderr, /HEAVY_BODY_EXECUTED/);
        const failure = invoke('rust', root, { CI_TEST_LOG: log, CI_FAIL_FUTURE: '1' });
        assert.notEqual(failure.status, 0, 'New integration failures must fail ordinary CI');
        assert.match(failure.stdout + failure.stderr, /new_integration/);
        write(root, 'crates/kessel/tests/collision.rs', `#[test] fn ${gates.rustCheckpoint}() ${markRust('collision-must-not-silently-skip')}`);
        const collision = invoke('rust', root, { CI_TEST_LOG: log }); assert.notEqual(collision.status, 0); assert.match(collision.stdout + collision.stderr, /collides/); assert.ok(!lines(log).includes('collision-must-not-silently-skip'));
        fs.unlinkSync(path.join(root, 'crates/kessel/tests/collision.rs'));
        write(root, 'crates/kessel/tests/reference_conformance.rs', reference.replace(gates.rustCheckpoint + '()', 'renamed_checkpoint()'));
        const missing = invoke('rust', root, { CI_TEST_LOG: log });
        assert.notEqual(missing.status, 0); assert.match(missing.stdout + missing.stderr, /checkpoint test identity/);
        assert.doesNotMatch(missing.stdout + missing.stderr, /HEAVY_BODY_EXECUTED/);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('real Node harness runs future files and reference siblings, scopes exact names, and retains terminal input', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ordinary node ')), log = path.join(root, 'ran.txt');
    try {
        const prelude = "const test=require('node:test');\n";
        write(root, 'crates/kessel-wasm/test.js', prelude + `test('fixed Node/browser tests',()=>{${markNode('core')}});`);
        write(root, 'crates/kessel-wasm/js/test/adapter.test.js', prelude + `test('adapter',()=>{${markNode('adapter')}});`);
        write(root, 'crates/kessel-wasm/js/test/future.test.js', prelude + `test(${JSON.stringify(gates.wasmCheckpoint)},()=>{${markNode('same-name-other-file')}});test('future test',()=>{require('node:assert/strict').ok(!process.env.CI_FAIL_FUTURE);${markNode('future')}});`);
        write(root, 'crates/kessel-wasm/js/test/terminal-actor.test.js', prelude + `test('terminal resolver actor ingress survives Wasm checkpoint and restore',()=>{require('node:assert/strict').ok(process.env.KESSEL_TERMINAL_TRACE);${markNode('terminal')}});`);
        const reference = prelude + `test(${JSON.stringify(gates.wasmCheckpoint)},()=>{throw Error('HEAVY_BODY_EXECUTED')});\ntest(${JSON.stringify(gates.wasmCheckpoint + ' suffix')},()=>{${markNode('exact-suffix')}});\ntest('new reference sibling',()=>{${markNode('reference-sibling')}});`;
        write(root, 'crates/kessel-wasm/js/test/reference-conformance.test.js', reference);
        const env = { CI_TEST_LOG: log };
        for (const name of ['KESSEL_TERMINAL_TRACE', 'KESSEL_TERMINAL_POOL', 'KESSEL_TERMINAL_CATALOGUE', 'KESSEL_TERMINAL_PACKAGE']) { write(root, name, '{}'); env[name] = path.join(root, name); }
        const result = invoke('wasm', root, env); success(result);
        for (const name of ['core', 'adapter', 'future', 'same-name-other-file', 'terminal', 'exact-suffix', 'reference-sibling']) assert.ok(lines(log).includes(name), name + '\n' + result.stdout);
        assert.doesNotMatch(result.stdout + result.stderr, /HEAVY_BODY_EXECUTED/);
        const failure = invoke('wasm', root, { ...env, CI_FAIL_FUTURE: '1' }); assert.notEqual(failure.status, 0); assert.match(failure.stdout + failure.stderr, /future test/);
        write(root, 'crates/kessel-wasm/js/test/reference-conformance.test.js', reference + `test(${JSON.stringify(gates.wasmCheckpoint)},()=>{throw Error('DUPLICATE_HEAVY')});`);
        const duplicate = invoke('wasm', root, env); assert.notEqual(duplicate.status, 0); assert.match(duplicate.stdout + duplicate.stderr, /checkpoint test identity/); assert.doesNotMatch(duplicate.stdout + duplicate.stderr, /HEAVY_BODY_EXECUTED|DUPLICATE_HEAVY/);
        write(root, 'crates/kessel-wasm/js/test/reference-conformance.test.js', reference.replace(JSON.stringify(gates.wasmCheckpoint), "'renamed checkpoint'"));
        const missing = invoke('wasm', root, env); assert.notEqual(missing.status, 0); assert.match(missing.stdout + missing.stderr, /checkpoint test identity/);
        write(root, 'crates/kessel-wasm/js/test/reference-conformance.test.js', reference + "test.before(()=>{});");
        const unknown = invoke('wasm', root, env); assert.notEqual(unknown.status, 0); assert.match(unknown.stdout + unknown.stderr, /Unknown Wasm registration harness/);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('unsupported runnable Cargo kinds, missing artifacts and ambiguous artifacts fail red', () => {
    const root = '/workspace', target = { name: 'reference_conformance', kind: ['test'], src_path: root + '/crates/kessel/tests/reference_conformance.rs', test: true }, p = { id: 'p', manifest_path: root + '/crates/kessel/Cargo.toml', targets: [target] }, metadata = { workspace_members: ['p'], packages: [p] }, a = { reason: 'compiler-artifact', package_id: 'p', profile: { test: true }, target, executable: '/exe' };
    assert.throws(() => gates.rustPlan(root, metadata, []), /omitted|not compiled/);
    assert.throws(() => gates.rustPlan(root, metadata, [a, { ...a, executable: '/other' }]), /Ambiguous/);
    const unknown = { name: 'future', kind: ['unknown'], src_path: '/workspace/future', test: true };
    for (const kind of [['unknown'], ['rlib', 'unknown']]) { const future = { ...unknown, kind }; assert.throws(() => gates.rustPlan(root, { ...metadata, packages: [{ ...p, targets: [target, future] }] }, [a, { ...a, target: future }]), /Unknown runnable/); }
});

test('public CLI rejects unknown modes and ignored extra options',()=>{for(const args of [['unknown'],['wasm','--ignored']]){const result=spawnSync(process.execPath,[script,...args],{encoding:'utf8'});assert.notEqual(result.status,0);assert.match(result.stderr,/Use ci-tests.js/);}});
