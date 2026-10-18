'use strict';

const fs = require('node:fs');
const { execFileSync } = require('node:child_process');

// This is an audited list of documentation with no build or test consumers.
// In particular, Wasm PACKAGING.md is a package input and does not belong here.
const documentation = new Set([
    'README.md',
    'AGENTS.md',
    'CLAUDE.md',
    'compiler/README.md',
    'cards/README.md',
    'cards/PIPELINE.md',
    'crates/kessel/benchmarks/README.md',
]);

const full = (reason) => ({ run: true, reason });

function classifyPaths(paths) {
    if (!Array.isArray(paths) || paths.length === 0 || paths.some((file) => typeof file !== 'string' || file.length === 0)) {
        return full('No reliable, nonempty changed-file list; running all checks.');
    }
    if (!paths.every((file) => documentation.has(file))) {
        return { ...full('Changes include runtime, build, test, or unaudited paths.'), changedFiles: paths.length };
    }
    return { run: false, reason: 'Only audited documentation changed.', changedFiles: paths.length };
}

function inspectScope({ env = process.env, cwd = process.cwd() } = {}) {
    if (env.GITHUB_EVENT_NAME !== 'pull_request') {
        return full('Full checks are required for pushes, manual runs, and other events.');
    }
    try {
        const event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, 'utf8'));
        const base = event?.pull_request?.base?.sha;
        const head = event?.pull_request?.head?.sha;
        const sha = env.GITHUB_SHA;
        const validSha = (value) => typeof value === 'string' && /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/.test(value);
        if (![base, head, sha].every(validSha)) {
            return full('PR base, head, or merge SHA is missing or malformed; running all checks.');
        }
        const git = (...args) => execFileSync('git', args, {
            cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
            stdio: ['ignore', 'pipe', 'pipe'],
        });
        if (git('rev-parse', '--verify', 'HEAD').trim() !== sha) {
            return full('Checkout does not match the event merge SHA; running all checks.');
        }
        const parents = git('show', '--no-patch', '--format=%P', 'HEAD').trim().split(/\s+/);
        if (parents.length !== 2 || parents[0] !== base || parents[1] !== head
            || git('rev-parse', '--verify', 'HEAD^1').trim() !== base
            || git('rev-parse', '--verify', 'HEAD^2').trim() !== head) {
            return full('Checkout parents do not match the PR base and head; running all checks.');
        }
        // Compare the tested merge tree to its verified base. Disabling rename
        // detection exposes both the deleted source and added destination path.
        const diff = git('diff', '--name-only', '--no-renames', '-z', 'HEAD^1', 'HEAD');
        if (!diff.endsWith('\0')) return full('No reliable, nonempty changed-file list; running all checks.');
        return classifyPaths(diff.slice(0, -1).split('\0'));
    } catch {
        return full('Could not verify PR metadata or Git history; running all checks.');
    }
}

function main({ env = process.env, cwd = process.cwd() } = {}) {
    const decision = inspectScope({ env, cwd });
    const summary = `### CI scope\n\n${decision.run ? 'Run all checks.' : 'Skip build and test work.'} ${decision.reason}\n`;
    // Publish the decision last: an output/summary write failure fails this step,
    // so an incomplete selector cannot authorize a successful documentation skip.
    if (env.GITHUB_STEP_SUMMARY) fs.appendFileSync(env.GITHUB_STEP_SUMMARY, summary);
    if (env.GITHUB_OUTPUT) fs.appendFileSync(env.GITHUB_OUTPUT, `run=${decision.run}\n`);
    console.log(summary.trim());
    return decision;
}

if (require.main === module) {
    try { main(); }
    catch (error) { console.error(`Cannot publish CI scope: ${error.message}`); process.exitCode = 1; }
}

module.exports = { classifyPaths, inspectScope, main };
