'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const same = require('node:util').isDeepStrictEqual;
const { canonical } = require('../inventory');
const sha256 = x => crypto.createHash('sha256').update(x).digest('hex');
const semanticSha256 = x => sha256(JSON.stringify(canonical(x)));
const keys = (v, allowed) => v && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).every(k => allowed.includes(k));
const C = value => ({ kind: 'const', value });
const DISCARD = { kind: 'discardSpecificCard', target: 'chosen' };
const KEEPER_PROMPT = 'Choose 2 cards named Keeper of Skara Nal from your hand to discard';
const QIRA_ABILITIES = [
    { trigger: 'constant', title: 'This unit gets -1/-0 for each card in your hand.', costs: [], effect: { kind: 'modifyStats', target: 'self', duration: 'persistent', power: { kind: 'binary', op: '*', left: { kind: 'unary', op: '-', value: C(1) }, right: { kind: 'ref', path: ['player','hand','length'] } }, hp: C(0) } },
    { trigger: 'whenPlayed', title: 'Discard a card from your hand to deal 3 damage to a unit', costs: [], optional: true, targetResolver: { owner: 'self', type: 'any', zone: 'hand' }, effect: DISCARD,
        ifYouDo: { title: 'Deal 3 damage to a unit', costs: [], targetResolver: { owner: 'any', type: 'unit', zone: 'anyAttackable' }, effect: { kind: 'damage', target: 'chosen', amount: C(3) } } },
];
const KEEPER_ABILITIES = [{ trigger: 'onAttack', title: 'Discard 2 cards named Keeper of Skara Nal from your hand. If you do, this unit gets +15/+0 and gains Overwhelm for this attack.', costs: [], optional: true,
    targetResolver: { owner: 'self', type: 'any', zone: 'hand', promptTitle: KEEPER_PROMPT, predicate: { kind: 'binary', op: '===', left: { kind: 'ref', path: ['card','title'] }, right: C('Keeper of Skara Nal') }, mode: 'exactly', count: C(2) }, effect: DISCARD,
    ifYouDo: { title: 'This unit gets +15/+0 and gains Overwhelm for this attack', costs: [], effect: { kind: 'attackModifiers', effects: [
        { kind: 'modifyStats', target: 'self', duration: 'attack', power: C(15), hp: C(0) },
        { kind: 'gainKeyword', target: 'self', duration: 'attack', keyword: 'overwhelm', amount: C(1) },
    ] } },
}];
function classifyChosenHandDiscard(e) {
    const c = e?.card;
    if (!keys(e, ['setCode','sourcePath','sourceHash','abilities','className','baseClass','id','internalName','status','card','dataHash','keywordValues','requiredCapabilities','diagnostics','attachCondition','piloting']) ||
        e.status !== 'supported' || e.baseClass !== 'NonLeaderUnitCard' || !c || !same(c.types, ['unit']) || c.arena !== 'ground' || c.unique !== true ||
        !Array.isArray(c.aspects) || !c.aspects.every(x => ['heroism','villainy','vigilance','command','aggression','cunning'].includes(x)) || !Array.isArray(c.traits) || !c.traits.every(x => typeof x === 'string' && x.length) ||
        e.attachCondition || e.piloting || c.pilotText || c.pilotBox || c.pilotingBox || c.deployBox || c.epicAction || c.upgradePower != null || c.upgradeHp != null ||
        typeof e.sourcePath !== 'string' || !e.sourcePath || !/^[a-f0-9]{64}$/.test(e.sourceHash ?? '') || !/^[a-f0-9]{64}$/.test(e.dataHash ?? '') ||
        !keys(e.abilities, ['leader','unit','card']) || !same(e.abilities.leader, []) || !same(e.abilities.unit, [])) return null;
    if (c.text === 'This unit gets –1/–0 for each card in your hand.\nWhen Played: You may discard a card from your hand. If you do, deal 3 damage to a unit.' && c.cost === 7 && c.hp === 7 && c.power === 9 && same(c.keywords, []) && same(e.keywordValues, {}) && same(e.abilities.card, QIRA_ABILITIES))
        return { family: 'chosen-hand-discard', profile: 'qira', trigger: 'whenPlayed' };
    // Metadata mentions gained Overwhelm; the native intrinsic table must contain Restore only.
    if (c.text === 'Restore 2\nOn Attack: You may discard 2 cards named Keeper of Skara Nal from your hand. If you do, this unit gets +15/+0 and gains Overwhelm for this attack.' && c.cost === 6 && c.hp === 8 && c.power === 5 && c.title === 'Keeper of Skara Nal' && same(c.keywords, ['restore','overwhelm']) && same(e.keywordValues, { restore: 2 }) && same(e.abilities.card, KEEPER_ABILITIES))
        return { family: 'chosen-hand-discard', profile: 'keeper', trigger: 'onAttack' };
    return null;
}
const DONOR_PROFILES = {
    repair: { code: 'JTL_075', entrySemanticSha256: 'bb4537b3b348a79fad1542aab276863b25a6eb73de5d9089ff093c500505ddb8' },
    ground: { code: 'SOR_095', entrySemanticSha256: '27b538f7ef020583c610281275a75f2187beb15ef5946d59a18809d53d686d3d' },
    space: { code: 'SOR_112', entrySemanticSha256: '8da69e297c020302bb1a87af57639462b3c207ecb56e798a61d4598f01802bae' },
    draw: { code: 'ASH_087', entrySemanticSha256: '9527c6c036399d875479ce85a1374117d1a74ef58e2bdc4e29caf2e09968f052' },
    plating: { code: 'ASH_086', entrySemanticSha256: '65ffaa78baae51e0923509ece4d9ff927b39cced640df79b3ecb6f15752ff843' },
    shield: { code: 'TOKEN_SHIELD', entrySemanticSha256: 'a8a34f754731a5dc9de991d5476b9491466190ac7521e0c7192c9210fc5538a9' },
};
function planChosenHandDiscardProbes(entry, options = {}) {
    const shape = classifyChosenHandDiscard(entry);
    if (!shape) return null;
    const result = { schema: 'swu-card-probes/v1', cardId: entry.setCode, sourceHash: entry.sourceHash, cases: [], obligations: [], uncovered: [] };
    function obligation(dimension, reason) {
        const id = `${entry.setCode}:chosen-hand-discard:${dimension}`;
        result.obligations.push({ id, path: 'chosen-hand-discard', dimension });
        if (reason) result.uncovered.push({ id, path: 'chosen-hand-discard', reason });
        return id;
    }
    const defs = Array.isArray(options.catalogue) ? options.catalogue : options.catalogue?.cards ?? [];
    const reportBytes = fs.readFileSync(path.resolve(__dirname, '../../compiler/generated/runtime-admission.json'));
    const report = JSON.parse(reportBytes), admitted = new Map(report.runtimeAdmittedCards.map(e => [e.setCode,e]));
    const donorCodes = Object.fromEntries(Object.entries(DONOR_PROFILES).map(([role,p]) => [role,options.chosenHandDiscardDonors?.[role] ?? p.code]));
    const donors = Object.fromEntries(Object.entries(donorCodes).map(([role,code]) => [role,defs.find(e => e.setCode === code)]));
    const bad = Object.keys(DONOR_PROFILES).filter(role => !donors[role] || semanticSha256(donors[role]) !== DONOR_PROFILES[role].entrySemanticSha256 || admitted.get(donorCodes[role])?.internalName !== donors[role]?.internalName);
    result.donorAdmissionReceipt = { schema: 'chosen-hand-discard-donors/v1', reportByteSha256: sha256(reportBytes), catalogueHash: report.catalogueHash,
        donors: Object.fromEntries(Object.entries(DONOR_PROFILES).map(([role,p]) => [role,{ code: donorCodes[role], entrySemanticSha256: donors[role] ? semanticSha256(donors[role]) : null, expectedEntrySemanticSha256: p.entrySemanticSha256, sourceHash: donors[role]?.sourceHash ?? null, dataHash: donors[role]?.dataHash ?? null, implementation: admitted.get(donorCodes[role])?.implementation ?? null }])) };
    if (bad.length) { obligation('donor-admission', 'Missing, mutated or independently unadmitted complete setup donors: ' + bad.join(',')); return result; }
    const row = (role, owner = 'opponent', extra = {}) => ({ code: donorCodes[role], owner, arena: donors[role].card.arena, ...extra });
    const mixed = () => [row('ground','self',{exhausted:true}),row('ground','opponent',{exhausted:false}),row('space','self',{exhausted:false}),row('space','opponent',{exhausted:true})];
    function add(scenario, sourceSeat, extra = {}) {
        const id = obligation(`${scenario}-seat-${sourceSeat}`);
        const probe = { ...shape, cardId: entry.setCode, sourceArena: entry.card.arena, sourceSeat, scenario, resourceCount: 30, donorCodes, targets: mixed(),
            handCodes: shape.profile === 'qira' ? [donorCodes.repair,donorCodes.ground,donorCodes.space] : [entry.setCode,entry.setCode,donorCodes.repair],
            discardIndexes: shape.profile === 'qira' ? [0] : [0,1], targetIndex: 1, baseDamage: shape.profile === 'keeper' ? 5 : 0, triggerOrder: 'custom-first', ...extra };
        result.cases.push({ id: `auto/${entry.setCode}/chosen-hand-discard-${scenario}-seat-${sourceSeat}`, name: `chosen-hand-discard-${scenario}-seat-${sourceSeat}`, kind: 'probe', focusCardIds: [entry.setCode],
            setupCardIds: [...new Set([...probe.targets.map(t => t.code),...probe.targets.flatMap(t => (t.attachments ?? []).map(a => a.code)),...probe.handCodes.filter(c => c !== entry.setCode), ...(probe.waitDraw ? [donorCodes.draw] : []), ...(probe.shielded ? [donorCodes.plating,donorCodes.shield] : [])])].sort(),
            behaviorIds: ['chosen-hand-discard',shape.profile,'physical-card-selection','last-event-success',...(shape.profile === 'keeper' ? ['exactly-two','restore','attack-modifiers'] : ['live-hand-power','damage3'])],
            assertionStages: ['construction','normal-paid-play','exact-prompt-actions','ordered-physical-discard','if-you-do-gate',...(shape.profile === 'keeper' ? ['restore-order','attack-power-and-keyword','combat','expiry'] : ['live-hand-power','secondary-unit-target','damage']), 'observation'], coverageObligationIds: [id], probe });
    }
    for (const sourceSeat of [1,2]) {
        if (shape.profile === 'qira') {
            for (const [scenario,targetIndex] of [['self','self'],['own-ground',0],['enemy-ground',1],['own-space',2],['enemy-space',3]]) add(`damage-${scenario}`,sourceSeat,{ targetIndex });
            add('no-choose-nothing',sourceSeat,{ assertNoDone:true,decline:'passAbility',discardIndexes:[] });
            add('decline-pass',sourceSeat,{ decline:'passAbility',discardIndexes:[] });
            add('empty-hand',sourceSeat,{ handCodes:[],discardIndexes:[] });
            add('single-card',sourceSeat,{ handCodes:[donorCodes.repair] });
            add('large-hand',sourceSeat,{ handCodes:Array(10).fill(donorCodes.repair),targetIndex:'self' });
            add('self-draw',sourceSeat,{ waitDraw:'self',drawCode:donorCodes.draw,targetIndex:'self',handCodes:[donorCodes.repair,donorCodes.draw,donorCodes.space] });
            add('opponent-draw',sourceSeat,{ waitDraw:'opponent',drawCode:donorCodes.draw,opponentHandCodes:[donorCodes.draw],targetIndex:'self' });
        } else {
            add('restore-first',sourceSeat,{ triggerOrder:'restore-first' });
            add('custom-first',sourceSeat);
            add('decline-done',sourceSeat,{ decline:'done',discardIndexes:[] });
            add('decline-pass',sourceSeat,{ decline:'passAbility',discardIndexes:[] });
            add('zero-matching',sourceSeat,{ handCodes:[donorCodes.repair],discardIndexes:[] });
            add('one-matching',sourceSeat,{ handCodes:[entry.setCode,donorCodes.repair],discardIndexes:[] });
            add('undamaged-base',sourceSeat,{ baseDamage:0 });
            add('toggle-three',sourceSeat,{ handCodes:[entry.setCode,entry.setCode,entry.setCode,donorCodes.repair],toggleSequence:[0,1,0,2],discardIndexes:[1,2] });
            add('shield',sourceSeat,{ shielded:true,defenderUpgradeCode:donorCodes.plating,opponentHandCodes:[donorCodes.plating] });
            add('attack-base',sourceSeat,{ targetIndex:'base' });
            add('terminal-attack-cleanup',sourceSeat,{ targetIndex:'base',enemyBaseDamage:10,terminalAttackCleanup:true });
        }
    }
    if (shape.profile === 'qira') for (const sourceSeat of [1,2]) for (const keep of ['old','new']) {
        add(`unique-keep-${keep}`,sourceSeat,{
            targets:[...mixed(),{code:entry.setCode,owner:'self',arena:entry.card.arena}],
            oldIndex:4,keep,targetIndex:1,
        });
        result.cases.at(-1).assertionStages.splice(-1,0,'unique-before-hand-picker','physical-distinct-hand-candidates','captured-incoming-damage-source','survivor-live-hand-power','current-live-unit-domain');
    }
    for (const [dimension, reason] of [
        ['source-helper-independent-review','Independent complete card source, helper defaults, full metadata and intrinsic keyword review remains required'],
        ['discard-replacements-and-last-event-failure','No admitted discard replacement or last-event failure donor; exact result behavior needs dedicated runtime mutation tests'],
        ['ownership-controller-divergence','Global supported state requires original unit owner equal controller; arbitrary foreign ownership is unsupported'],
        ['pending-selector-import','Standard replay covers action windows and exact decision sets; arbitrary pending hand-picker import is unsupported; native copy/hash tests are separate'],
        ['same-title-other-subtitle','No independently admitted alternate Keeper subtitle donor is supplied; title predicate is source-bound but actual alternate-subtitle game remains uncovered'],
        ['deck-construction-legality','Constructed decks and repeated physical same-code copies are unvalidated fixture setup, not tournament deck-legality evidence'],
        ['unique-source-departure','No unique keep-old/keep-new source departure during selected-hand continuation is currently constructed'],
        ['stacked-attack-grants','No separately granted attack-duration boost/Overwhelm stacking source is currently constructed'],
        ['retained-terminal-attack-grant','Normal lethal base-attack witnesses reach complete attack cleanup; they do not prove terminal import/export of a retained attack and its modifiers'],
        ['automatic-single-target-setting','Engine reference uses autoSingleTarget:false; handler-menu optimization under other settings remains uncovered'],
        ['subsequent-optional-action','No second printed chosen-hand trigger after a decline is currently constructed to expose stale success across distinct activations'],
    ]) {
        if (dimension === 'unique-source-departure' && shape.profile === 'qira') continue;
        obligation(dimension,reason);
    }
    return result;
}
module.exports = { classifyChosenHandDiscard, planChosenHandDiscardProbes, KEEPER_PROMPT };
