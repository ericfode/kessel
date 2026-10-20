'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const same = require('node:util').isDeepStrictEqual;
const { donorValid } = require('./recipient-heal-probes');
const { defeatEvent } = require('./arena-damage-probes');
const { canonical } = require('../inventory');
// Semantic identity ignores object insertion order; arrays and every JSON value remain significant.
const entrySemanticSha256 = entry => hash(JSON.stringify(canonical(entry)));
const keys = (v, allowed) => v && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).every(k => allowed.includes(k));
const hash = v => crypto.createHash('sha256').update(v).digest('hex');
const DEFAULT_DONORS = { ground: 'HMW_093', ground2: 'ASH_061', space: 'JTL_069', shield: 'TOKEN_SHIELD', defeat: 'SHD_079' };

// Closed complete-card envelope: distribution is not permission to ignore companion registrations.
function classifyDistributedDamage(e) {
    const c = e?.card;
    if (!keys(e, ['setCode','sourcePath','sourceHash','abilities','className','baseClass','id','internalName','status','card','dataHash','keywordValues','requiredCapabilities','diagnostics','attachCondition','piloting']) ||
        e.status !== 'supported' || e.baseClass !== 'NonLeaderUnitCard' || !c || !same(c.types, ['unit']) || !['ground','space'].includes(c.arena) ||
        !Number.isInteger(c.cost) || c.cost < 0 || c.cost > 16 || !Number.isInteger(c.hp) || c.hp < 1 || c.hp > 32 || !Number.isInteger(c.power) || c.power < 0 || c.power > 32 ||
        typeof c.unique !== 'boolean' || !Array.isArray(c.aspects) || !c.aspects.every(x => ['heroism','villainy','vigilance','command','aggression','cunning'].includes(x)) ||
        !Array.isArray(c.traits) || !c.traits.every(x => typeof x === 'string' && x.length) ||
        e.attachCondition || e.piloting || c.pilotText || c.pilotBox || c.pilotingBox || c.deployBox || c.epicAction || c.upgradePower != null || c.upgradeHp != null ||
        typeof e.sourcePath !== 'string' || !e.sourcePath || !/^[a-f0-9]{64}$/.test(e.sourceHash ?? '') ||
        !keys(e.abilities, ['leader','unit','card']) || !same(e.abilities.leader, []) || !same(e.abilities.unit, []) || e.abilities.card?.length !== 1) return null;
    const a = e.abilities.card[0];
    if (!keys(a, ['trigger','title','costs','effect']) || !['whenPlayed','whenDefeated'].includes(a.trigger) || typeof a.title !== 'string' || !a.title.trim() || a.title.length > 256 || !same(a.costs, []) ||
        !same(a.effect, { kind: 'distributeDamage', amount: { kind: 'const', value: 6 }, canChooseNoTargets: false, canDistributeLess: false, targetResolver: { owner: 'opponent', type: 'unit', zone: 'anyAttackable' } })) return null;
    const overwhelm = same(c.keywords, ['overwhelm']) && same(e.keywordValues, { overwhelm: 1 });
    if (!(same(c.keywords, []) && same(e.keywordValues, {})) && !overwhelm) return null;
    if (overwhelm && a.trigger !== 'whenPlayed') return null;
    return { family: 'distributed-damage', trigger: a.trigger, amount: 6, overwhelm };
}

function planDistributedDamageProbes(entry, options = {}) {
    const shape = classifyDistributedDamage(entry);
    if (!shape) return null;
    const result = { schema: 'swu-card-probes/v1', cardId: entry.setCode, sourceHash: entry.sourceHash, cases: [], obligations: [], uncovered: [] };
    function obligation(dimension, reason) {
        const id = `${entry.setCode}:distributed-damage:${dimension}`;
        result.obligations.push({ id, path: 'distributed-damage', dimension });
        if (reason) result.uncovered.push({ id, path: 'distributed-damage', reason });
        return id;
    }
    const defs = Array.isArray(options.catalogue) ? options.catalogue : options.catalogue?.cards ?? [];
    const donorCodes = { ...DEFAULT_DONORS, ...options.distributedDamageDonors };
    const donors = Object.fromEntries(Object.entries(donorCodes).map(([role, code]) => [role, defs.find(e => e.setCode === code)]));
    const bytes = fs.readFileSync(path.resolve(__dirname, '../../compiler/generated/runtime-admission.json'));
    const report = JSON.parse(bytes), admitted = new Map(report.runtimeAdmittedCards.map(e => [e.setCode, e]));
    const roles = Object.keys(DEFAULT_DONORS);
    const bad = roles.filter(role => !(role === 'defeat' ? donors[role] && defeatEvent(donors[role]) : donorValid(donors[role], role)) || !admitted.has(donorCodes[role]) || admitted.get(donorCodes[role]).internalName !== donors[role]?.internalName);
    result.donorAdmissionReceipt = { schema: 'distributed-damage-donor-admission/v1', reportPath: 'compiler/generated/runtime-admission.json', reportHash: hash(bytes), catalogueHash: report.catalogueHash,
        donors: Object.fromEntries(roles.map(role => { const e = donors[role]; return [role, { code: donorCodes[role], sourceHash: e?.sourceHash ?? null, dataHash: e?.dataHash ?? null, entryHash: e ? hash(JSON.stringify(e)) : null, implementation: admitted.get(donorCodes[role])?.implementation ?? null }]; })) };
    if (bad.length) { obligation('donor-admission', 'Missing, incompatible or independently unadmitted setup donors: ' + bad.join(',')); return result; }
    const row = (role, owner = 'opponent', extra = {}) => ({ code: donorCodes[role], owner, arena: donors[role].card.arena, ...extra });
    const shield = () => ({ code: donorCodes.shield, token: 'shield' });
    const friendly = () => row('ground', 'self');
    const scenarios = [
        ['no-target', [friendly()], []],
        ['single', [row('ground'), friendly()], [6]],
        ['low-hp', [row('ground2', 'opponent', { damage: donors.ground2.card.hp - 1 }), friendly()], [6]],
        ['split-2-4', [row('ground2'), row('space'), friendly()], [2,4]],
        ['split-1-2-3', [row('ground2'), row('space'), row('ground'), friendly()], [1,2,3]],
        ['six-ones', [row('ground2'), row('space'), row('ground'), row('ground2'), row('space'), row('ground'), friendly()], [1,1,1,1,1,1]],
        ['shield', [row('ground2', 'opponent', { attachments: [shield()] }), row('space'), friendly()], [2,4]],
        ['multiple-shields', [row('ground2', 'opponent', { attachments: [shield(),shield()] }), row('space', 'opponent', { attachments: [shield()] }), friendly()], [2,4]],
        ['all-shielded', [row('ground2', 'opponent', { attachments: [shield()] }), row('space', 'opponent', { attachments: [shield()] }), friendly()], [2,4]],
        ['split-lethal', [row('ground2', 'opponent', { damage: donors.ground2.card.hp - 2 }), row('space'), friendly()], [2,4]],
    ];
    function add(scenario, sourceSeat, targets, amounts, extra = {}) {
        const id = obligation(`${scenario}-seat-${sourceSeat}`);
        const probe = { ...shape, cardId: entry.setCode, sourceArena: entry.card.arena, sourceSeat, owner: sourceSeat - 1, scenario, resourceCount: 30, donorCodes,
            targets, assignments: amounts.map((amount, index) => ({ index, amount })),
            ...(shape.trigger === 'whenDefeated' ? { sourceInPlay: true, activationCode: donorCodes.defeat } : {}), ...extra };
        result.cases.push({ id: `auto/${entry.setCode}/distributed-damage-${scenario}-seat-${sourceSeat}`, name: `distributed-damage-${scenario}-seat-${sourceSeat}`, kind: 'probe', focusCardIds: [entry.setCode],
            setupCardIds: [...new Set([...targets.map(t => t.code), ...targets.flatMap(t => (t.attachments ?? []).map(a => a.code)), ...(probe.activationCode ? [probe.activationCode] : [])])].sort(),
            behaviorIds: ['distributed-damage', shape.trigger, 'enemy-unit-scope', 'mandatory-six', ...(shape.overwhelm ? ['overwhelm'] : [])],
            assertionStages: ['construction','normal-activation','exact-distribution-action-domain','assigned-damage','shield-identities','enemy-and-base-scope','observation'], coverageObligationIds: [id], probe });
    }
    for (const sourceSeat of [1,2]) for (const [scenario, targets, amounts] of scenarios) add(scenario, sourceSeat, structuredClone(targets), amounts);
    if (shape.trigger === 'whenPlayed' && entry.card.unique) for (const sourceSeat of [1,2]) for (const keep of ['old','new'])
        add(`unique-keep-${keep}`, sourceSeat, [{ code: entry.setCode, owner: 'self', arena: entry.card.arena }, row('ground')], [], { oldIndex: 0, keep, assignments: [{ index: 1, amount: 6 }] });
    if (shape.trigger === 'whenDefeated') for (const sourceSeat of [1,2]) {
        add('live-attack', sourceSeat, [row('space'), row('ground'), friendly()], [4,2], { liveAttack: true, attackTargetIndex: 0 });
        obligation(`foreign-control-seat-${sourceSeat}`,
            `Unsupported global state boundary: original unit owner must equal controller. The wire omits divergent physical ownership, so restoration to the physical owner's discard cannot be reproduced. Original engine-only diagnostic auto/${entry.setCode}/distributed-damage-foreign-control-seat-${sourceSeat} is preserved separately and earns no native/Wasm coverage.`);
    }
    if (shape.overwhelm) for (const sourceSeat of [1,2])
        add('overwhelm-combat', sourceSeat, [row('ground2'), row('space'), friendly()], [2,4], { combatAfterPlay: true, attackTargetIndex: 0 });
    // Additional interaction donors are pinned complete definitions. Keep this separate from
    // the original donor map so adding witnesses cannot rewrite existing case objects.
    const interactionProfiles = {
        leader: { code: 'SEC_005', entrySemanticSha256: 'ae1bc463696a3fc045eea668b5cf50bf5426a26e1dbf99eec43b6764031cfe91' },
        deathDraw: { code: 'ASH_116', entrySemanticSha256: '012a766ea8690b19a17d429ae0e22f04d4bc8b0c8cf4e970b19fbc81a2011f3d' },
        hpAura: { code: 'JTL_085', entrySemanticSha256: '757750d1bc4a2cb15705514d2a17e9db8fe3b1be243fb4a8c69246f3058c1334' },
    };
    result.interactionDonorAdmissionReceipt = { schema: 'distributed-damage-interaction-donors/v1', reportHash: hash(bytes), donors: {} };
    for (const [role, profile] of Object.entries(interactionProfiles)) {
        const e = defs.find(e => e.setCode === profile.code), native = admitted.get(profile.code);
        const valid = !!e && entrySemanticSha256(e) === profile.entrySemanticSha256 && !!native && native.internalName === e.internalName;
        result.interactionDonorAdmissionReceipt.donors[role] = { ...profile, valid, sourceHash: e?.sourceHash ?? null, dataHash: e?.dataHash ?? null, implementation: native?.implementation ?? null };
        if (!valid) { obligation(`interaction-donor-${role}`, `Missing, mutated or independently unadmitted complete interaction donor ${profile.code}`); continue; }
        for (const sourceSeat of [1,2]) {
            if (role === 'leader') add('deployed-leader', sourceSeat,
                [{ code: profile.code, owner: 'opponent', arena: 'ground', deployedLeader: true, damage: 4 }, row('space'), friendly()], [4,2],
                { leaderCode: profile.code, enemyLeaderCode: profile.code });
            if (role === 'deathDraw') add('death-draw', sourceSeat,
                [{ code: profile.code, owner: 'opponent', arena: 'ground' }, row('space'), friendly()], [2,4], { deathDrawIndex: 0 });
            if (role === 'hpAura') add('hp-aura-departure', sourceSeat,
                [row('space', 'opponent', { damage: 5 }), { code: profile.code, owner: 'opponent', arena: 'space' }, friendly()], [2,4],
                { auraIndex: 1, auraRecipientIndex: 0 });
        }
    }
    for (const [dimension, reason] of [
        ['source-and-helper-review','Independent complete source, helper-default, keyword and metadata review remains required'],
        ['meta-continuations','Actual cards have no continuation consuming distribution success; resolved-zero continuation and stale-result clearing require separate runtime/helper tests'],
        ['ownership-controller-boundary','Executable standard cases require equal original owner/controller; arbitrary unit control changes and restoration are unsupported globally'],
        ['additional-death-listeners','Ant Droid draw and Victor Leader aura departure are bounded witnesses; other lethal auras and death callbacks remain unexercised'],
        ['damage-increase-and-restrictions','No live pending ability-damage increase, receive-damage restriction, immunity or cannot-be-defeated hook donor; unsupported pool hooks must remain rejected'],
        ['leader-deployment-path','Leader recipient is staged already deployed; this does not exercise its ordinary deployment action'],
        ['attack-context-listeners','Normal attack cases do not include Raid/stat threshold or attack-end/death-listener interactions'],
        ['enumeration-overflow','Six targets fit the 4096 reference action bound; ten-target 5005-action rejection requires separate capacity test'],
    ]) obligation(dimension, reason);
    return result;
}
module.exports = { classifyDistributedDamage, planDistributedDamageProbes, DEFAULT_DONORS };
