'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync, spawnSync } = require('node:child_process');
const test = require('node:test');
const { classifyPaths, inspectScope } = require('./ci-scope');

function fixture(t, change = () => {}) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'kessel-ci-scope-'));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const repository = path.join(directory, 'repository');
    fs.mkdirSync(repository);
    const git = (...args) => execFileSync('git', [
        '-c', 'user.name=CI scope test', '-c', 'user.email=ci-scope@example.invalid',
        '-c', 'commit.gpgSign=false', '-c', 'core.hooksPath=/dev/null', ...args,
    ], { cwd: repository, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
    const write = (name, content) => {
        const file = path.join(repository, name);
        fs.mkdirSync(path.dirname(file), { recursive: true });
        fs.writeFileSync(file, content);
    };
    git('init', '--initial-branch=main', '--template=');
    write('README.md', 'Documentation\n');
    write('AGENTS.md', 'Instructions\n');
    write('src/runtime.rs', 'Runtime implementation\n');
    git('add', '.');
    git('commit', '-m', 'Base');
    const base = git('rev-parse', 'HEAD');
    git('switch', '-c', 'feature');
    change({ repository, git, write });
    git('add', '-A');
    git('commit', '--allow-empty', '-m', 'PR change');
    const head = git('rev-parse', 'HEAD');
    git('switch', 'main');
    git('merge', '--no-ff', '--no-edit', 'feature');
    const event = { pull_request: { base: { sha: base }, head: { sha: head } } };
    const eventPath = path.join(directory, 'event.json');
    const saveEvent = () => fs.writeFileSync(eventPath, JSON.stringify(event));
    saveEvent();
    const env = {
        GITHUB_EVENT_NAME: 'pull_request', GITHUB_EVENT_PATH: eventPath,
        GITHUB_SHA: git('rev-parse', 'HEAD'),
    };
    return { directory, repository, git, event, eventPath, env, saveEvent };
}

test('only the exact audited documentation paths qualify', () => {
    const paths = ['README.md', 'AGENTS.md', 'CLAUDE.md', 'compiler/README.md',
        'cards/README.md', 'cards/PIPELINE.md', 'crates/kessel/benchmarks/README.md'];
    assert.equal(classifyPaths(paths).run, false);
    for (const file of ['docs/guide.md', 'readme.md', './README.md', 'README.md\n',
        'crates/kessel-wasm/PACKAGING.md', 'crates/kessel/src/lib.rs',
        'compiler/generated/cards.json', 'cards/reference/fixtures/manifest.json.gz',
        'Cargo.lock', '.github/workflows/ci.yml', '.github/scripts/ci-scope.js']) {
        assert.equal(classifyPaths(['README.md', file]).run, true, file);
    }
    for (const paths of [undefined, null, [], 'README.md', [''], [null]]) {
        assert.equal(classifyPaths(paths).run, true);
    }
});

test('a verified PR merge with only documentation changes may skip', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    assert.deepEqual(inspectScope({ env: f.env, cwd: f.repository }), {
        run: false, reason: 'Only audited documentation changed.', changedFiles: 1,
    });
});

test('a depth-two checkout contains enough history for a verified skip', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    const shallow = path.join(f.directory, 'shallow');
    execFileSync('git', ['clone', '--depth=2', '--no-local', '--branch=main', f.repository, shallow], {
        encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
    });
    assert.equal(execFileSync('git', ['rev-parse', '--is-shallow-repository'], { cwd: shallow, encoding: 'utf8' }).trim(), 'true');
    assert.equal(inspectScope({ env: f.env, cwd: shallow }).run, false);
});

test('deleted audited documentation is still documentation-only', (t) => {
    const f = fixture(t, ({ repository }) => fs.unlinkSync(path.join(repository, 'AGENTS.md')));
    assert.equal(inspectScope({ env: f.env, cwd: f.repository }).run, false);
});

test('renaming runtime code into an allowed document exposes both paths', (t) => {
    const f = fixture(t, ({ repository, git }) => {
        fs.mkdirSync(path.join(repository, 'compiler'));
        git('mv', 'src/runtime.rs', 'compiler/README.md');
    });
    const decision = inspectScope({ env: f.env, cwd: f.repository });
    assert.deepEqual(decision, {
        run: true, reason: 'Changes include runtime, build, test, or unaudited paths.', changedFiles: 2,
    });
});

test('runtime, CI, package Markdown and generated changes require full checks', (t) => {
    for (const file of ['src/runtime.rs', '.github/workflows/ci.yml',
        'crates/kessel-wasm/PACKAGING.md', 'compiler/generated/cards.json']) {
        const f = fixture(t, ({ write }) => write(file, 'Changed input\n'));
        const decision = inspectScope({ env: f.env, cwd: f.repository });
        assert.equal(decision.run, true, file);
        assert.equal(decision.changedFiles, 1, file);
    }
});

test('an empty diff never authorizes a skip', (t) => {
    const f = fixture(t);
    assert.equal(inspectScope({ env: f.env, cwd: f.repository }).run, true);
});

test('non-PR events always run all checks without relying on event metadata', () => {
    for (const event of [undefined, 'push', 'workflow_dispatch', 'pull_request_target']) {
        assert.equal(inspectScope({ env: { GITHUB_EVENT_NAME: event }, cwd: os.tmpdir() }).run, true);
    }
});

test('missing, unreadable and malformed events all select full checks', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    for (const eventPath of [undefined, path.join(f.directory, 'missing'), f.directory]) {
        assert.equal(inspectScope({ env: { ...f.env, GITHUB_EVENT_PATH: eventPath }, cwd: f.repository }).run, true);
    }
    for (const data of ['{broken', 'null', '{}', '{"pull_request":{"base":{},"head":{}}}']) {
        fs.writeFileSync(f.eventPath, data);
        assert.equal(inspectScope({ env: f.env, cwd: f.repository }).run, true, data);
    }
});

test('missing or malformed SHAs cannot authorize a skip', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    for (const sha of [undefined, '', 'main', 'a'.repeat(39)]) {
        assert.equal(inspectScope({ env: { ...f.env, GITHUB_SHA: sha }, cwd: f.repository }).run, true);
    }
    for (const side of ['base', 'head']) {
        const original = f.event.pull_request[side].sha;
        for (const sha of [undefined, '', 'main', 123]) {
            f.event.pull_request[side].sha = sha;
            f.saveEvent();
            assert.equal(inspectScope({ env: f.env, cwd: f.repository }).run, true, side);
        }
        f.event.pull_request[side].sha = original;
    }
});

test('merge SHA and both parent identities must match the PR event', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    assert.equal(inspectScope({ env: { ...f.env, GITHUB_SHA: f.event.pull_request.head.sha }, cwd: f.repository }).run, true);
    for (const side of ['base', 'head']) {
        const original = f.event.pull_request[side].sha;
        f.event.pull_request[side].sha = '0'.repeat(40);
        f.saveEvent();
        assert.equal(inspectScope({ env: f.env, cwd: f.repository }).run, true, side);
        f.event.pull_request[side].sha = original;
    }
    f.saveEvent();
    f.git('checkout', '--detach', f.event.pull_request.head.sha);
    assert.equal(inspectScope({ env: { ...f.env, GITHUB_SHA: f.event.pull_request.head.sha }, cwd: f.repository }).run, true);
});

test('missing Git history or a repository error selects full checks', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    assert.equal(inspectScope({ env: f.env, cwd: f.directory }).run, true);
    const shallow = path.join(f.directory, 'shallow');
    execFileSync('git', ['clone', '--depth=1', '--no-local', '--branch=main', f.repository, shallow], {
        encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
    });
    assert.equal(inspectScope({ env: f.env, cwd: shallow }).run, true);
});

test('CLI appends runner output and a human-readable scope explanation', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    const output = path.join(f.directory, 'output');
    const summary = path.join(f.directory, 'summary');
    fs.writeFileSync(output, 'previous=value\n');
    for (const eventName of ['pull_request', 'push']) {
        const child = spawnSync(process.execPath, [path.join(__dirname, 'ci-scope.js')], {
            cwd: f.repository, encoding: 'utf8',
            env: { ...process.env, ...f.env, GITHUB_EVENT_NAME: eventName, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: summary },
        });
        assert.equal(child.status, 0, child.stderr);
    }
    assert.equal(fs.readFileSync(output, 'utf8'), 'previous=value\nrun=false\nrun=true\n');
    assert.match(fs.readFileSync(summary, 'utf8'), /Only audited documentation changed/);
    assert.match(fs.readFileSync(summary, 'utf8'), /Full checks are required/);
});

test('a publishing failure cannot produce a successful skip output', (t) => {
    const f = fixture(t, ({ write }) => write('README.md', 'Updated documentation\n'));
    const output = path.join(f.directory, 'output');
    const child = spawnSync(process.execPath, [path.join(__dirname, 'ci-scope.js')], {
        cwd: f.repository, encoding: 'utf8',
        env: { ...process.env, ...f.env, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: f.directory },
    });
    assert.equal(child.status, 1);
    assert.equal(fs.existsSync(output), false);
});
