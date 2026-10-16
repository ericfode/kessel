'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "LAW_171",
  "sourcePath": "game/cards/07_LAW/events/Stockpile.ts",
  "sourceHash": "ac23f6020217fc6787838154dda000489d59e21a0b0805ce4cfad31aba658190",
  "className": "Stockpile",
  "baseClass": "EventCard",
  "card": {
    "title": "Stockpile",
    "subtitle": "",
    "cost": 6,
    "hp": null,
    "power": null,
    "text": "Resource this event and the top card of your deck.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "Resources enter play exhausted.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "8221105498",
    "aspects": [
      "command"
    ],
    "traits": [
      "supply"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "LAW",
      "number": 171
    },
    "setCodes": [
      {
        "set": "LAW",
        "number": 171
      }
    ],
    "internalName": "stockpile"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Resource this event and the top card of your deck",
        "costs": [],
        "effect": {
          "kind": "simultaneous",
          "effects": [
            {
              "kind": "resourceCard",
              "target": "self"
            },
            {
              "kind": "resourceCard",
              "target": "ownTopDeck"
            }
          ]
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
    "name": "LAW_171-empty-deck-self-only-seat1",
    "deckSize": 0,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-empty-deck-self-only-seat2",
    "deckSize": 0,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-one-unit-card-seat1",
    "deckSize": 1,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-one-unit-card-seat2",
    "deckSize": 1,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-two-card-deck-seat1",
    "deckSize": 2,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-two-card-deck-seat2",
    "deckSize": 2,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-ordinary-unit-top-seat1",
    "deckSize": 4,
    "sourceSeat": 1,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-ordinary-unit-top-seat2",
    "deckSize": 4,
    "sourceSeat": 2,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-native-arena-distractors-seat1",
    "nativeArena": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-native-arena-distractors-seat2",
    "nativeArena": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082",
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-event-top-seat1",
    "topCode": "TWI_124",
    "sourceSeat": 1,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-event-top-seat2",
    "topCode": "TWI_124",
    "sourceSeat": 2,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-upgrade-top-seat1",
    "topCode": "SOR_057",
    "sourceSeat": 1,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-upgrade-top-seat2",
    "topCode": "SOR_057",
    "sourceSeat": 2,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-token-creation-unit-not-played-seat1",
    "topCode": "JTL_090",
    "sourceSeat": 1,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-token-creation-unit-not-played-seat2",
    "topCode": "JTL_090",
    "sourceSeat": 2,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-token-creation-event-not-played-seat1",
    "topCode": "TWI_237",
    "sourceSeat": 1,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-token-creation-event-not-played-seat2",
    "topCode": "TWI_237",
    "sourceSeat": 2,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-second-stockpile-not-played-seat1",
    "topCode": "LAW_171",
    "sourceSeat": 1,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-second-stockpile-not-played-seat2",
    "topCode": "LAW_171",
    "sourceSeat": 2,
    "deckSize": 3,
    "resources": 8,
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-exact-six-resources-seat1",
    "resources": 6,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-exact-six-resources-seat2",
    "resources": 6,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-mixed-ready-resources-seat1",
    "resources": 8,
    "spent": 2,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082",
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-mixed-ready-resources-seat2",
    "resources": 8,
    "spent": 2,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082",
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-missing-aspect-pays-eight-seat1",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_029",
    "resources": 8,
    "cost": 8,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0
  },
  {
    "name": "LAW_171-missing-aspect-pays-eight-seat2",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_029",
    "resources": 8,
    "cost": 8,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0
  },
  {
    "name": "LAW_171-insufficient-payment-no-play-seat1",
    "resources": 5,
    "unaffordable": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  },
  {
    "name": "LAW_171-insufficient-payment-no-play-seat2",
    "resources": 5,
    "unaffordable": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082",
    "spent": 0,
    "cost": 6,
    "leaderCode": "SOR_007",
    "baseCode": "SOR_023"
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planSimultaneousResourceProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':simultaneous-resource:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode,design.baseCode,design.topCode,...(design.nativeArena?['TOKEN_TIE_FIGHTER','TOKEN_BATTLE_DROID']:[])])].sort(),behaviorIds:['fixed-two-simultaneous-resource-targets','event-self-stays-resource','top-deck-resource-no-play-or-draw','exact-resource-identity-order-and-readiness'],assertionStages:['actual-paid-event-or-unpayable-exclusion','actual-shared-event-window-prehandler-vector','ordered-resource-physical-identities','no-recursive-play-draw-or-nativebirth','natural-regroup-and-terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'simultaneous-resource',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'simultaneous-resource',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['simultaneous-runtime-record-proof','Read-only engine observer establishes source vector/window; simple public traces alone may match sequence. Runtime fixed2 record generation and semantic review must prove simultaneous execution.'],['external-resource-replacements','No exhaustive external movement restrictions, resource-capacity or controlled-card transfer interleavings; runtime boundary tests and independent review remain required.']].map(([dimension,reason])=>{const id=entry.setCode+':simultaneous-resource:'+dimension;obligations.push({id,path:'simultaneous-resource',dimension});return{id,path:'simultaneous-resource',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planSimultaneousResourceProbes};
