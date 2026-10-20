'use strict';
const {isDeepStrictEqual:same}=require('node:util');
const {createHash}=require('node:crypto');
const FOCUS={
  "abilities": {
    "card": [
      {
        "costs": [],
        "effect": {
          "amount": {
            "kind": "const",
            "value": 5
          },
          "kind": "damage",
          "target": "ownBase"
        },
        "then": {
          "costs": [],
          "effect": {
            "amount": {
              "kind": "const",
              "value": 5
            },
            "kind": "damage",
            "target": {
              "kind": "arenaList",
              "predicate": {
                "kind": "const",
                "value": true
              },
              "receiver": {
                "kind": "ref",
                "path": [
                  "game"
                ]
              }
            }
          },
          "title": "Deal 5 damage to each unit"
        },
        "title": "Deal 5 damage to your base",
        "trigger": "event"
      }
    ],
    "leader": [],
    "unit": []
  },
  "baseClass": "EventCard",
  "card": {
    "aspects": [
      "aggression",
      "villainy"
    ],
    "cost": 6,
    "deployBox": null,
    "epicAction": "",
    "hp": null,
    "id": "6062959089",
    "internalName": "operation-cinder",
    "keywords": [],
    "power": null,
    "rules": "All damage dealt to units by Operation Cinder is dealt simultaneously.\n",
    "setCodes": [
      {
        "number": 151,
        "set": "ASH"
      }
    ],
    "setId": {
      "number": 151,
      "set": "ASH"
    },
    "subtitle": "",
    "text": "Deal 5 damage to your base. Then, deal 5 damage to each unit.",
    "title": "Operation Cinder",
    "traits": [
      "disaster",
      "plan"
    ],
    "types": [
      "event"
    ],
    "unique": false,
    "upgradeHp": null,
    "upgradePower": null
  },
  "className": "OperationCinder",
  "dataHash": "2731fbda4351987a37248e3d72038b9053430786b21f663008f48e5c2c7fe9b2",
  "diagnostics": [],
  "id": "6062959089",
  "internalName": "operation-cinder",
  "keywordValues": {},
  "requiredCapabilities": [
    "ability:then",
    "cardType:event",
    "ir:arenaList",
    "ir:const",
    "ir:damage",
    "ir:ref",
    "ref:game",
    "trigger:event"
  ],
  "setCode": "ASH_151",
  "sourceHash": "cf245ad766b2c283f3df98dd3748c5883f040c89a0abe3220ff02dca7b7fea7c",
  "sourcePath": "game/cards/08_ASH/events/OperationCinder.ts",
  "status": "supported"
};
// Reviewed admitted setup profiles: semantic identity is independent of source-byte identity.
const DONORS = {
  "ground": {
    "semanticFingerprint": "e0efbf36198692bc808fea91f1e7b8213b6731d9551a5cd71b7b91920ba6a69d",
    "sourcePath": null,
    "sourceByteHash": null
  },
  "space": {
    "semanticFingerprint": "ae3b0a9c89d10c42adc92e10c8f50e16303570c1d917ae707625999ff5c18994",
    "sourcePath": null,
    "sourceByteHash": null
  },
  "shield": {
    "semanticFingerprint": "957aebc6e792150be2a9f97c8f4ac7cdf72b9220ac7948382a7ecc6a2dd2a203",
    "sourcePath": "game/cards/01_SOR/tokens/Shield.ts",
    "sourceByteHash": "39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f"
  },
  "aura": {
    "semanticFingerprint": "0bc935f5f073675ba6159ddae2bb1880278bea78e996ebe2ae7237a0ff70932d",
    "sourcePath": "game/cards/04_JTL/units/VictorLeaderLeadingFromTheFront.ts",
    "sourceByteHash": "143c730115cdb25d9f7d47590d4fbf9fbc09fba40d2c68ea8732c7333781773f"
  },
  "auraRecipient": {
    "semanticFingerprint": "40aea9562e2d8c71445f84e5423fe97e5bccc2be9bc9ef25124cba84d7cf8017",
    "sourcePath": "game/cards/08_ASH/units/B-WingRearguard.ts",
    "sourceByteHash": "422da43280995d4d5938c9fffc7d02df22a5dfac254a7bbe658e6629ad11abda"
  },
  "deathHeal": {
    "semanticFingerprint": "b4b327ecb12b299fe2bc4ef42eeee47b025966a99dc1fc0a7232036ee30a603a",
    "sourcePath": "game/cards/07_LAW/units/ImperialDoorTechnician.ts",
    "sourceByteHash": "0232e2034907e2563a7f82c5b9777df66d536d255272af79ce0c319c53f37482"
  },
  "deathDamage": {
    "semanticFingerprint": "b7e13ccad9f7dee803b5252fc82674620c7f5940750ef9bbf6c92a53e0c09a40",
    "sourcePath": "game/cards/07_LAW/units/CavernAngelsXWing.ts",
    "sourceByteHash": "2f70a8dc6c9ddf156a5ce45239ea675af417a36ac011a3dec243c883da6dd388"
  },
  "deathDraw": {
    "semanticFingerprint": "f5f9ef3eb03978d21cec7bb1dff48dc2641cee177b4669c71b1b0ca3392148db",
    "sourcePath": "game/cards/01_SOR/units/StarWingScout.ts",
    "sourceByteHash": "6a825694c1745fa7ce25b0709e91192921d83125a28e1d18c0c2aa6d73fc161a"
  },
  "leader": {
    "semanticFingerprint": "0cd2730edab5027f435bf38187a6dc4c1c3d4fd52e19a2440fc43a7afc40d84b",
    "sourcePath": "game/cards/01_SOR/leaders/SabineWrenGalvanizedRevolutionary.ts",
    "sourceByteHash": "b1d4463b24bdf9485f4b3528bde3cb1c7c328a4d1e6facdfa9923c2faf0484a4"
  }
};
const canonical = v => Array.isArray(v) ? v.map(canonical) : v && typeof v === 'object' ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])])) : v;
function semanticProfile(e) {
    if (!e || typeof e !== 'object') return null;
    const copy = { ...e };
    for (const key of ['sourceHash','sourcePath','dataHash','setCode','diagnostics']) delete copy[key];
    return copy;
}
function semanticFingerprint(e) { return createHash('sha256').update(JSON.stringify(canonical(semanticProfile(e)))).digest('hex'); }
function classifySelfBaseArenaDamage(e) {
    // Every printed field, intrinsic keyword, registration and source byte identity is bound.
    // The selected closed shape is literal five, mandatory ownBase then game arenaList(true).
    if (!e || !same(e.diagnostics ?? [], []) || !same(semanticProfile(e), semanticProfile(FOCUS)) ||
        e.setCode !== FOCUS.setCode || e.dataHash !== FOCUS.dataHash || e.sourcePath !== FOCUS.sourcePath || e.sourceHash !== FOCUS.sourceHash) return null;
    return { family: 'self-base-arena-damage', trigger: 'event', amount: 5, baseTarget: 'ownBase', receiver: 'game', continuation: 'then' };
}
function matchDonor(e, role) {
    const expected = DONORS[role];
    return !!expected && !!e && semanticFingerprint(e) === expected.semanticFingerprint &&
        e.sourceHash === expected.sourceByteHash && e.sourcePath === expected.sourcePath;
}
function planSelfBaseArenaDamageProbes(entry, options = {}) {
    const shape = classifySelfBaseArenaDamage(entry);
    if (!shape) return null;
    const defs = Array.isArray(options.catalogue) ? options.catalogue : options.catalogue?.cards ?? [];
    const admitted = options.admittedCardIds ? new Set(options.admittedCardIds) : null;
    const donor = Object.fromEntries(Object.keys(DONORS).map(role => [role, defs.find(e => matchDonor(e, role) && (!admitted || admitted.has(e.setCode)))]));
    const result = { schema: 'swu-card-probes/v1', cardId: entry.setCode, sourceHash: entry.sourceHash, cases: [], obligations: [], uncovered: [] };
    const obligation = (dimension, reason) => {
        const id = entry.setCode + ':self-base-arena-damage:' + dimension;
        result.obligations.push({ id, path: 'abilities.card.0', dimension });
        if (reason) result.uncovered.push({ id, path: 'abilities.card.0', reason });
        return id;
    };
    const missing = Object.keys(donor).filter(role => !donor[role]);
    if (missing.length) {
        obligation('construction-donors', 'Missing complete admitted structural donor roles: ' + missing.join(', '));
        return result;
    }
    const row = (role, owner, shields = 0) => ({ code: donor[role].setCode, owner, arena: donor[role].card.arena, exhausted: true,
        attachments: Array.from({ length: shields }, () => ({ code: donor.shield.setCode, token: 'shield' })) });
    const specs = [
        { scenario: 'empty', targets: [], dimensions: ['own-base-first','empty-arena','paid-event-play'] },
        { scenario: 'mixed-shields-global-order', targets: [row('ground','opponent'),row('space','self',2),row('ground','self'),row('space','opponent')], dimensions: ['global-physical-order','all-primary-damage-before-departure','exact-shield-survivor'] },
        { scenario: 'multi-owner-shield-menus', targets: [row('ground','self',2),row('space','opponent',2),row('ground','opponent',2),row('space','self',2)], dimensions: ['replacement-owner-order','mandatory-resolve-next-all','exact-multi-shield-survivors'] },
        { scenario: 'aura-death', targets: [row('aura','self'),row('auraRecipient','self'),row('ground','opponent')], dimensions: ['aura-after-primary-batch','secondary-stat-death'] },
        { scenario: 'own-base-lethal-death-heal', baseDamage: 25, targets: [row('deathHeal','self'),row('ground','opponent')], dimensions: ['winner-latched-before-units','mandatory-healing-after-winner','terminal-export'] },
        { scenario: 'winner-latched-then-mandatory-choice', baseDamage: 25, enemyBaseDamage: 28, targets: [row('deathDamage','self'),row('ground','opponent')], dimensions: ['mandatory-choice-after-winner','both-bases-lethal-first-winner','terminal-choice-export'] },
        { scenario: 'enemy-base-lethal-death-trigger', enemyBaseDamage: 28, targets: [row('deathDamage','self'),row('ground','opponent')], dimensions: ['opponent-base-only-real-death-trigger','source-player-winner'] },
        { scenario: 'death-draw-order', targets: [row('deathDraw','self'),row('deathHeal','self'),row('ground','opponent')], dimensions: ['mandatory-death-draw','death-trigger-order','draw-after-primary-batch'] }
    ];
    for (const sourceSeat of [1,2]) for (const spec of specs) {
        const coverageObligationIds = spec.dimensions.map(name => obligation(name + '-seat-' + sourceSeat));
        const probe = { ...shape, cardId: entry.setCode, sourceSeat, scenario: spec.scenario, initiative: 'self', leaderCode: donor.leader.setCode, enemyLeaderCode: donor.leader.setCode,
            resourceCount: 24, targets: spec.targets, baseDamage: spec.baseDamage ?? 0, enemyBaseDamage: spec.enemyBaseDamage ?? 0 };
        const used = new Set([probe.leaderCode,...spec.targets.map(t => t.code),...spec.targets.flatMap(t => t.attachments.map(a => a.code))]);
        const donorProvenance = Object.entries(donor).filter(([,e]) => used.has(e.setCode)).map(([role,e]) => ({ role, cardId: e.setCode, semanticFingerprint: semanticFingerprint(e), sourcePath: e.sourcePath, sourceByteHash: e.sourceHash }));
        result.cases.push({ id: 'auto/' + entry.setCode + '/self-base-arena-' + spec.scenario + '-seat-' + sourceSeat,
            name: 'self-base-arena-' + spec.scenario + '-seat-' + sourceSeat, kind: 'probe', focusCardIds: [entry.setCode], setupCardIds: [...used].sort(),
            source: { module: '../pipeline/probe-fixture.js', scenario: spec.scenario }, behaviorIds: ['damage','then','arena-list','event'],
            assertionStages: ['construction','normal-play','resolved-list','terminal'], coverageObligationIds, donorProvenance, probe });
    }
    // Proposed assertions are distinct from oracle execution, admission and conformance credit.
    return result;
}
module.exports = { classifySelfBaseArenaDamage, planSelfBaseArenaDamageProbes, semanticFingerprint, matchDonor };
