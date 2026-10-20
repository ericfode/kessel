'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { createSimulator } = require('../node');
const { records, checkRecords } = require('../check');

// Run against a selected source-derived terminal-continuation recording and a
// freshly built package. Keeping recordings external preserves corpus ownership.
test('terminal resolver actor ingress survives Wasm checkpoint and restore', {
    skip: !process.env.KESSEL_TERMINAL_TRACE,
}, async () => {
    const rows = records(process.env.KESSEL_TERMINAL_TRACE);
    const simulator = await createSimulator({
        poolPath: process.env.KESSEL_TERMINAL_POOL,
        cataloguePath: process.env.KESSEL_TERMINAL_CATALOGUE,
        packagePath: process.env.KESSEL_TERMINAL_PACKAGE,
    });
    let game;
    let terminalChoices = 0;
    const completedResourceActors = new Set();
    try {
        assert.equal(checkRecords(simulator, rows).matched, true);
        for (const row of rows) {
            if (row.type === 'state') game ??= simulator.loadState(row.state);
            if (row.type !== 'decision') continue;
            if (row.prompt.type !== 'resource') completedResourceActors.clear();
            if (game.over) {
                terminalChoices++;
                assert.equal(game.actor, null);
                const before = game.snapshot();
                const winner = game.winner;
                const wrongActor = row.actor === 'p1' ? 'p2' : 'p1';
                if (row.prompt.type !== 'resource' || completedResourceActors.has(wrongActor)) {
                    assert.throws(() => game.legalActions(wrongActor), /actor mismatch/);
                    assert.throws(() => game.apply(wrongActor, row.chosen), /actor mismatch/);
                } else {
                    assert.ok(game.legalActions(wrongActor).length > 0,
                        'The other unfinished resource chooser is independently valid');
                }
                assert.deepEqual(game.snapshot(), before);
                const checkpoint = game.mark(row.actor);
                const actions = game.legalActions(row.actor);
                assert.ok(actions.some(a => a.key === row.chosen));
                game.apply(row.actor, row.chosen);
                const after = game.snapshot();
                assert.equal(game.winner, winner);
                assert.equal(game.actor, null);
                game.restore(row.actor, checkpoint);
                assert.deepEqual(game.snapshot(), before);
                assert.deepEqual(game.legalActions(row.actor), actions);
                game.apply(row.actor, row.chosen);
                assert.deepEqual(game.snapshot(), after);
                game.releaseMark(checkpoint);
            } else game.apply(row.actor, row.chosen);
            if (row.prompt.type === 'resource' && JSON.parse(row.chosen)[2] === 'done') {
                completedResourceActors.add(row.actor);
            }
        }
        assert.ok(terminalChoices > 0, 'Recording must exercise a latched-winner chooser');
    } finally { game?.free(); simulator.free(); }
});
