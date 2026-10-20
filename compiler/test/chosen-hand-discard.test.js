'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {compileSource, loadEnums} = require('../compiler');
const root = path.resolve(__dirname, '../..');
const enums = loadEnums(path.join(root, 'forceteki/server'));
const names = ['08_ASH/units/QiraMasterOfTerasKasi.ts', '09_HMW/units/KeeperOfSkaraNalAwoken.ts'];
const sources = names.map(n => fs.readFileSync(path.join(root, 'forceteki/server/game/cards', n), 'utf8'));
const prompt = 'Choose 2 cards named Keeper of Skara Nal from your hand to discard';
function compile(source, index = 1) { return compileSource(source, 'game/cards/' + names[index], enums); }
function supported(source, index = 1) {
    const card = compile(source, index);
    assert.equal(card.status, 'supported', JSON.stringify(card.diagnostics));
    return card;
}
function rejected(source) { assert.equal(compile(source).status, 'unsupported'); }

test('pinned own-hand sources retain complete selector and continuation shapes', () => {
    const qira = supported(sources[0], 0), keeper = supported(sources[1]);
    assert.equal(qira.baseClass, 'NonLeaderUnitCard');
    assert.equal(qira.abilities.card.length, 2);
    const q = qira.abilities.card[1];
    assert.deepEqual(q.targetResolver, {owner:'self', type:'any', zone:'hand'});
    assert.equal(q.optional, true);
    assert.deepEqual(q.costs, []);
    assert.deepEqual(q.effect, {kind:'discardSpecificCard', target:'chosen'});
    assert.deepEqual(q.ifYouDo.targetResolver, {owner:'any', type:'unit', zone:'anyAttackable'});
    assert.deepEqual(q.ifYouDo.effect, {kind:'damage',target:'chosen',amount:{kind:'const',value:3}});
    const k = keeper.abilities.card[0];
    assert.deepEqual(k.targetResolver, {owner:'self',type:'any',zone:'hand',predicate:{kind:'binary',op:'===',left:{kind:'ref',path:['card','title']},right:{kind:'const',value:'Keeper of Skara Nal'}},mode:'exactly',count:{kind:'const',value:2},promptTitle:prompt});
    assert.equal(k.optional, true);
    assert.deepEqual(k.costs, []);
    assert.deepEqual(k.effect, q.effect);
    assert.deepEqual(k.ifYouDo.effect, {kind:'attackModifiers',effects:[{kind:'modifyStats',target:'self',duration:'attack',power:{kind:'const',value:15},hp:{kind:'const',value:0}},{kind:'gainKeyword',target:'self',duration:'attack',keyword:'overwhelm',amount:{kind:'const',value:1}}]});
    // Hidden-zone decline is a resolver lifecycle rule, not a source optional flag.
    assert.equal(Object.hasOwn(k.targetResolver, 'optional'), false);
});

test('literal prompt changes survive lowering for native complete-profile validation', () => {
    const changed = supported(sources[1].replace(prompt, 'Different reviewed label'));
    assert.equal(changed.abilities.card[0].targetResolver.promptTitle, 'Different reviewed label');
    const absent = supported(sources[1].replace(`activePromptTitle: '${prompt}',`, ''));
    assert.equal(Object.hasOwn(absent.abilities.card[0].targetResolver, 'promptTitle'), false);
});

test('hand-discard prompt rejects unsupported types and display siblings', () => {
    for (const value of ['42', 'false', 'null', "['label']", "{label:'text'}", "() => 'label'", "''"]) {
        rejected(sources[1].replace(`'${prompt}'`, value));
    }
    for (const field of ['waitingPromptTitle','contextTitle','effect','effectArgs','effectMessage','ongoingEffectDescription','inventedResolverField','appendToDefaultTitle','choosingPlayer','optional','canChooseNoCards','showUnresolvable']) {
        rejected(sources[1].replace('mode: TargetMode.Exactly,', `${field}: 'unsupported', mode: TargetMode.Exactly,`));
    }
});

test('hand-discard prompt cannot be transplanted into unsupported selector scopes', () => {
    for (const [before, after] of [
        ['RelativePlayer.Self','RelativePlayer.Opponent'],
        ['ZoneName.Hand','ZoneName.Discard'],
        ['TargetMode.Exactly','TargetMode.UpTo'],
        ['numCards: 2','numCards: 3'],
        ['zoneFilter: ZoneName.Hand,','zoneFilter: ZoneName.Hand, cardTypeFilter: "unit",'],
    ]) rejected(sources[1].replace(before, after));
    for (const field of ['unsupported: true', 'amount: 2', 'isCost: false', "activePromptTitle: 'nested label'", "waitingPromptTitle: 'nested label'"])
        rejected(sources[1].replace('discardSpecificCard()', `discardSpecificCard({ ${field} })`));
    rejected(sources[1].replace("mode: TargetMode.Exactly,", "onSelectionSetChanged: () => {}, mode: TargetMode.Exactly,"));
    rejected(sources[1].replace('discardSpecificCard()', 'discardSpecificCard({ target: context => context.targets.target[0] })'));
});

test('lowering depends on source structure rather than focus identity', () => {
    const renamed = supported(sources[1].replace('KeeperOfSkaraNalAwoken','UnrelatedUnit').replace('4533738134','new-physical-definition').replace('keeper-of-skara-nal#awoken','unrelated-unit'));
    assert.deepEqual(renamed.abilities, supported(sources[1]).abilities);
});
