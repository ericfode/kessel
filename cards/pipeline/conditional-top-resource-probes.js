'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "JTL_164",
  "sourcePath": "game/cards/04_JTL/units/ChamSyndullaRallyingRyloth.ts",
  "sourceHash": "b8014f9e9ea792cd561823ca4e82e88186cb934ddf3fd06285bca77938bc7563",
  "className": "ChamSyndullaRallyingRyloth",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Cham Syndulla",
    "subtitle": "Rallying Ryloth",
    "cost": 4,
    "hp": 4,
    "power": 5,
    "text": "When Played: If an opponent controls more resources than you, you may put the top card of your deck into play as a resource.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": "A card put into play as a resource enters play exhausted.\n(ERRATA) Templating update: “Put into play as a resource” becomes “resource”. \"Controls more\" becomes \"control fewer\".\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "4019449999",
    "aspects": [
      "aggression"
    ],
    "traits": [
      "rebel",
      "twi'lek"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "JTL",
      "number": 164
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 164
      }
    ],
    "internalName": "cham-syndulla#rallying-ryloth"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Put the top card of your deck into play as a resource",
        "costs": [],
        "optional": true,
        "effect": {
          "kind": "conditional",
          "condition": {
            "kind": "binary",
            "op": "<",
            "left": {
              "kind": "ref",
              "path": [
                "player",
                "resources",
                "length"
              ]
            },
            "right": {
              "kind": "ref",
              "path": [
                "opponent",
                "resources",
                "length"
              ]
            }
          },
          "onTrue": {
            "kind": "resourceCard",
            "target": "ownTopDeck"
          },
          "onFalse": {
            "kind": "noAction"
          }
        }
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": null,
  "piloting": null
};
const designs = [
  {
    "name": "JTL_164-fewer-accept-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-fewer-accept-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-fewer-decline-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "decline": true,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-fewer-decline-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "decline": true,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-more-false-seat1",
    "ownResources": 9,
    "enemyResources": 8,
    "condition": false,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-more-false-seat2",
    "ownResources": 9,
    "enemyResources": 8,
    "condition": false,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-fewer-opponent-all-spent-still-true-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "enemyExhausted": 9,
    "condition": true,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-fewer-opponent-all-spent-still-true-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "enemyExhausted": 9,
    "condition": true,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-equal-only-four-ready-false-seat1",
    "ownResources": 8,
    "enemyResources": 8,
    "ownExhausted": 4,
    "condition": false,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-equal-only-four-ready-false-seat2",
    "ownResources": 8,
    "enemyResources": 8,
    "ownExhausted": 4,
    "condition": false,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-more-only-four-ready-false-seat1",
    "ownResources": 9,
    "enemyResources": 8,
    "ownExhausted": 5,
    "condition": false,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-more-only-four-ready-false-seat2",
    "ownResources": 9,
    "enemyResources": 8,
    "ownExhausted": 5,
    "condition": false,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-eligible-empty-deck-no-event-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "deckSize": 0,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-eligible-empty-deck-no-event-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "deckSize": 0,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-last-card-resource-without-fatigue-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "deckSize": 1,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-last-card-resource-without-fatigue-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "deckSize": 1,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-event-top-identity-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "topCode": "TWI_124",
    "sourceSeat": 1,
    "deckSize": 3
  },
  {
    "name": "JTL_164-event-top-identity-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "topCode": "TWI_124",
    "sourceSeat": 2,
    "deckSize": 3
  },
  {
    "name": "JTL_164-upgrade-top-identity-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "topCode": "SOR_057",
    "sourceSeat": 1,
    "deckSize": 3
  },
  {
    "name": "JTL_164-upgrade-top-identity-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "topCode": "SOR_057",
    "sourceSeat": 2,
    "deckSize": 3
  },
  {
    "name": "JTL_164-unique-keep-old-incoming-dead-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "unique": "old",
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-unique-keep-old-incoming-dead-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "unique": "old",
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-unique-keep-incoming-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "unique": "incoming",
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-unique-keep-incoming-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": true,
    "unique": "incoming",
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-prior-own-carrier-fewer-to-equal-seat1",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": false,
    "ramp": "own",
    "deckSize": 8,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-prior-own-carrier-fewer-to-equal-seat2",
    "ownResources": 8,
    "enemyResources": 9,
    "condition": false,
    "ramp": "own",
    "deckSize": 8,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "JTL_164-prior-opponent-carrier-equal-to-fewer-seat1",
    "ownResources": 8,
    "enemyResources": 8,
    "condition": true,
    "ramp": "opponent",
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "deckSize": 3
  },
  {
    "name": "JTL_164-prior-opponent-carrier-equal-to-fewer-seat2",
    "ownResources": 8,
    "enemyResources": 8,
    "condition": true,
    "ramp": "opponent",
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "deckSize": 3
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planConditionalTopResourceProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':conditional-top-resource:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,'SOR_014','SOR_023',design.topCode,...(design.ramp?['JTL_119']:[])])].sort(),behaviorIds:['strict-total-resource-comparison','optional-top-deck-resource','captured-unique-source','exact-resource-identity-order-and-readiness'],assertionStages:['actual-paid-entry','live-condition-after-prior-real-ramp','exact-optional-or-no-effect-menu','exact-own-top-deck-resource','natural-regroup-and-terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'conditional-top-resource',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'conditional-top-resource',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['same-window-resource-changes','Prior real Carrier changes both comparison directions; no unsupported Bounty/Ambush same-window resource change is included.'],['external-resource-replacements','No exhaustive external movement restrictions, resource-capacity or controlled-card transfer interleavings; runtime boundary tests and independent review remain required.']].map(([dimension,reason])=>{const id=entry.setCode+':conditional-top-resource:'+dimension;obligations.push({id,path:'conditional-top-resource',dimension});return{id,path:'conditional-top-resource',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planConditionalTopResourceProbes};
