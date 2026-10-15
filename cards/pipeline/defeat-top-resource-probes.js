'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "ASH_103",
  "sourcePath": "game/cards/08_ASH/events/LongLiveTheEmpire.ts",
  "sourceHash": "a6d8735e00f70842b2982d42b0321d36c617d28dbd2a3f962874e56fb0b92024",
  "className": "LongLiveTheEmpire",
  "baseClass": "EventCard",
  "card": {
    "title": "Long Live the Empire",
    "subtitle": "",
    "cost": 2,
    "hp": null,
    "power": null,
    "text": "Defeat a friendly Imperial unit. If you do, resource the top card of your deck.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "Resources enter play exhausted.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9020748123",
    "aspects": [
      "command",
      "villainy"
    ],
    "traits": [
      "imperial"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "ASH",
      "number": 103
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 103
      }
    ],
    "internalName": "long-live-the-empire"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Defeat a friendly Imperial unit. If you do, resource the top card of your deck",
        "costs": [],
        "targetResolver": {
          "owner": "self",
          "type": "unit",
          "zone": "anyAttackable",
          "predicate": {
            "kind": "query",
            "name": "hasSomeTrait",
            "receiver": {
              "kind": "ref",
              "path": [
                "card"
              ]
            },
            "args": [
              {
                "kind": "const",
                "value": "imperial"
              }
            ]
          },
          "promptTitle": "Defeat a friendly Imperial unit"
        },
        "effect": {
          "kind": "defeat",
          "target": "chosen"
        },
        "ifYouDo": {
          "title": "Resource the top card of your deck",
          "costs": [],
          "effect": {
            "kind": "resourceCard",
            "target": "ownTopDeck"
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
    "name": "ASH_103-ground-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-ground-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-space-seat1",
    "target": "SOR_225",
    "arena": "space",
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-space-seat2",
    "target": "SOR_225",
    "arena": "space",
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-mixed-selector-ground-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "mixed": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-mixed-selector-ground-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "mixed": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-mixed-selector-space-seat1",
    "target": "SOR_225",
    "arena": "space",
    "mixed": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-mixed-selector-space-seat2",
    "target": "SOR_225",
    "arena": "space",
    "mixed": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-prepay-cancel-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "mixed": true,
    "cancel": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-prepay-cancel-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "mixed": true,
    "cancel": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-no-friendly-imperial-seat1",
    "none": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-no-friendly-imperial-seat2",
    "none": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-no-eligible-cancel-seat1",
    "none": true,
    "cancel": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-no-eligible-cancel-seat2",
    "none": true,
    "cancel": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-empty-deck-defeat-still-resolves-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "deckSize": 0,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-empty-deck-defeat-still-resolves-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "deckSize": 0,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-last-card-no-fatigue-seat1",
    "target": "SOR_225",
    "arena": "space",
    "deckSize": 1,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-last-card-no-fatigue-seat2",
    "target": "SOR_225",
    "arena": "space",
    "deckSize": 1,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-event-top-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "topCode": "TWI_124",
    "sourceSeat": 1,
    "deckSize": 3
  },
  {
    "name": "ASH_103-event-top-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "topCode": "TWI_124",
    "sourceSeat": 2,
    "deckSize": 3
  },
  {
    "name": "ASH_103-upgrade-top-seat1",
    "target": "SOR_128",
    "arena": "ground",
    "topCode": "SOR_057",
    "sourceSeat": 1,
    "deckSize": 3
  },
  {
    "name": "ASH_103-upgrade-top-seat2",
    "target": "SOR_128",
    "arena": "ground",
    "topCode": "SOR_057",
    "sourceSeat": 2,
    "deckSize": 3
  },
  {
    "name": "ASH_103-death-draw-two-cards-seat1",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 2,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-two-cards-seat2",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 2,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-accept-seat1",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-accept-seat2",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-decline-seat1",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "decline": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-decline-seat2",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "decline": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-last-card-fatigue-seat1",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 1,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-last-card-fatigue-seat2",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 1,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-empty-fatigue-seat1",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 0,
    "sourceSeat": 1,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-death-draw-empty-fatigue-seat2",
    "target": "JTL_063",
    "arena": "space",
    "deathDraw": true,
    "deckSize": 0,
    "sourceSeat": 2,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-deployed-imperial-leader-seat1",
    "leader": true,
    "target": "SOR_007",
    "arena": "ground",
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-deployed-imperial-leader-seat2",
    "leader": true,
    "target": "SOR_007",
    "arena": "ground",
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-native-token-not-imperial-seat1",
    "none": true,
    "token": true,
    "sourceSeat": 1,
    "deckSize": 3,
    "topCode": "HMW_082"
  },
  {
    "name": "ASH_103-native-token-not-imperial-seat2",
    "none": true,
    "token": true,
    "sourceSeat": 2,
    "deckSize": 3,
    "topCode": "HMW_082"
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planDefeatTopResourceProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':defeat-top-resource:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,'SOR_007','SOR_023','SOR_128','SOR_225','HMW_082',design.topCode,...(design.target?[design.target]:[]),...(design.token?['TOKEN_TIE_FIGHTER']:[])])].sort(),behaviorIds:['friendly-imperial-unit-selector','defeat-result-resource-continuation','death-triggers-after-continuation','exact-resource-identity-order-and-readiness'],assertionStages:['actual-paid-event-or-prepay-cancel','exact-friendly-imperial-selector-and-title','defeat-then-own-top-deck-resource','death-draw-physical-card-order','natural-terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'defeat-top-resource',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'defeat-top-resource',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['external-defeat-replacement','Normal actual unit defeat and undeployed leader result witnessed; external replacement/prevention not exhaustively constructed.'],['external-resource-replacements','No exhaustive external movement restrictions, resource-capacity or controlled-card transfer interleavings; runtime boundary tests and independent review remain required.']].map(([dimension,reason])=>{const id=entry.setCode+':defeat-top-resource:'+dimension;obligations.push({id,path:'defeat-top-resource',dimension});return{id,path:'defeat-top-resource',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planDefeatTopResourceProbes};
