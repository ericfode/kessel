'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "LOF_121",
  "sourcePath": "game/cards/05_LOF/units/ThePurgilKingLeadingTheJourney.ts",
  "sourceHash": "f6c3fb10e0b4de3ae46c8ddf02ed3be7c23d0bcc5e20f781c47f8701771c570e",
  "className": "ThePurgilKingLeadingTheJourney",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "The Purrgil King",
    "subtitle": "Leading The Journey",
    "cost": 8,
    "hp": 12,
    "power": 4,
    "text": "Restore 4\nWhen Played: Draw a card for each friendly unit with 7 or more remaining HP.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "2968188569",
    "aspects": [
      "command"
    ],
    "traits": [
      "creature"
    ],
    "arena": "space",
    "keywords": [
      "restore"
    ],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "LOF",
      "number": 121
    },
    "setCodes": [
      {
        "set": "LOF",
        "number": 121
      }
    ],
    "internalName": "the-purrgil-king#leading-the-journey"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Draw a card for each friendly unit with 7 or more remaining HP",
        "costs": [],
        "effect": {
          "kind": "draw",
          "target": "ownPlayer",
          "amount": {
            "kind": "query",
            "name": "arenaUnitCount",
            "receiver": {
              "kind": "ref",
              "path": [
                "player"
              ]
            },
            "args": [
              {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "const",
                  "value": true
                },
                "right": {
                  "kind": "binary",
                  "op": ">=",
                  "left": {
                    "kind": "ref",
                    "path": [
                      "card",
                      "remainingHp"
                    ]
                  },
                  "right": {
                    "kind": "const",
                    "value": 7
                  }
                }
              }
            ]
          }
        }
      }
    ]
  },
  "keywordValues": {
    "restore": 4
  },
  "attachCondition": null,
  "piloting": null
};
const designs = [
  {
    "name": "LOF_121-entering-self-only-p1",
    "count": 1,
    "ownSeat": 0
  },
  {
    "name": "LOF_121-entering-self-only-p2",
    "count": 1,
    "ownSeat": 1
  },
  {
    "name": "LOF_121-own-ground-exact-seven-p1",
    "count": 2,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-own-ground-exact-seven-p2",
    "count": 2,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-ground-six-enemy-space-ten-excluded-p1",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 1
      },
      {
        "key": "enemy",
        "code": "LOF_119",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-ground-six-enemy-space-ten-excluded-p2",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 1
      },
      {
        "key": "enemy",
        "code": "LOF_119",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-own-space-eight-p1",
    "count": 2,
    "units": [
      {
        "key": "recipient",
        "code": "LOF_119",
        "owner": "self",
        "damage": 2
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-own-space-eight-p2",
    "count": 2,
    "units": [
      {
        "key": "recipient",
        "code": "LOF_119",
        "owner": "self",
        "damage": 2
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-space-six-enemy-ground-seven-excluded-p1",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "LOF_119",
        "owner": "self",
        "damage": 4
      },
      {
        "key": "enemy",
        "code": "HMW_082",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-space-six-enemy-ground-seven-excluded-p2",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "LOF_119",
        "owner": "self",
        "damage": 4
      },
      {
        "key": "enemy",
        "code": "HMW_082",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-both-arenas-exhaustion-and-owner-p1",
    "count": 3,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0,
        "exhausted": true
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "low",
        "code": "SEC_028",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "enemy",
        "code": "LOF_119",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-both-arenas-exhaustion-and-owner-p2",
    "count": 3,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0,
        "exhausted": true
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "low",
        "code": "SEC_028",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "enemy",
        "code": "LOF_119",
        "owner": "opponent",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-live-hp-boost-five-to-seven-p1",
    "count": 2,
    "boosts": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 2
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-live-hp-boost-five-to-seven-p2",
    "count": 2,
    "boosts": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 2
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-live-hp-boost-four-to-six-p1",
    "count": 1,
    "boosts": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 3
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-live-hp-boost-four-to-six-p2",
    "count": 1,
    "boosts": 1,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 3
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-native-battle-droid-live-one-to-seven-p1",
    "count": 2,
    "boosts": 3,
    "units": [
      {
        "key": "recipient",
        "code": "TOKEN_BATTLE_DROID",
        "owner": "self",
        "damage": 0,
        "token": "battleDroid",
        "arena": "ground"
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-native-battle-droid-live-one-to-seven-p2",
    "count": 2,
    "boosts": 3,
    "units": [
      {
        "key": "recipient",
        "code": "TOKEN_BATTLE_DROID",
        "owner": "self",
        "damage": 0,
        "token": "battleDroid",
        "arena": "ground"
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-native-battle-droid-low-excluded-p1",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "TOKEN_BATTLE_DROID",
        "owner": "self",
        "damage": 0,
        "token": "battleDroid",
        "arena": "ground"
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-native-battle-droid-low-excluded-p2",
    "count": 1,
    "units": [
      {
        "key": "recipient",
        "code": "TOKEN_BATTLE_DROID",
        "owner": "self",
        "damage": 0,
        "token": "battleDroid",
        "arena": "ground"
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-undeployed-leader-seven-excluded-p1",
    "count": 1,
    "leaderCode": "SOR_007",
    "ownSeat": 0
  },
  {
    "name": "LOF_121-undeployed-leader-seven-excluded-p2",
    "count": 1,
    "leaderCode": "SOR_007",
    "ownSeat": 1
  },
  {
    "name": "LOF_121-actual-deployed-leader-seven-included-p1",
    "count": 2,
    "leaderCode": "SOR_007",
    "deploy": true,
    "ownSeat": 0
  },
  {
    "name": "LOF_121-actual-deployed-leader-seven-included-p2",
    "count": 2,
    "leaderCode": "SOR_007",
    "deploy": true,
    "ownSeat": 1
  },
  {
    "name": "LOF_121-deployed-leader-six-excluded-p1",
    "count": 1,
    "leaderCode": "SOR_007",
    "units": [
      {
        "key": "recipient",
        "code": "SOR_007",
        "owner": "self",
        "damage": 1,
        "deployedLeader": true,
        "arena": "ground"
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-deployed-leader-six-excluded-p2",
    "count": 1,
    "leaderCode": "SOR_007",
    "units": [
      {
        "key": "recipient",
        "code": "SOR_007",
        "owner": "self",
        "damage": 1,
        "deployedLeader": true,
        "arena": "ground"
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-unique-keep-incoming-count-one-p1",
    "count": 1,
    "unique": "incoming",
    "ownSeat": 0
  },
  {
    "name": "LOF_121-unique-keep-incoming-count-one-p2",
    "count": 1,
    "unique": "incoming",
    "ownSeat": 1
  },
  {
    "name": "LOF_121-unique-keep-old-six-count-zero-p1",
    "count": 0,
    "unique": "old",
    "ownSeat": 0
  },
  {
    "name": "LOF_121-unique-keep-old-six-count-zero-p2",
    "count": 0,
    "unique": "old",
    "ownSeat": 1
  },
  {
    "name": "LOF_121-unique-keep-old-six-other-seven-count-one-p1",
    "count": 1,
    "unique": "old",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-unique-keep-old-six-other-seven-count-one-p2",
    "count": 1,
    "unique": "old",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-partial-deck-draw-one-fatigue-six-p1",
    "count": 3,
    "deckSize": 1,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-partial-deck-draw-one-fatigue-six-p2",
    "count": 3,
    "deckSize": 1,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-empty-deck-fatigue-nine-p1",
    "count": 3,
    "deckSize": 0,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-empty-deck-fatigue-nine-p2",
    "count": 3,
    "deckSize": 0,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 1
  },
  {
    "name": "LOF_121-paid-entry-natural-phase-restore-four-p1",
    "count": 1,
    "restore": true,
    "ownSeat": 0
  },
  {
    "name": "LOF_121-paid-entry-natural-phase-restore-four-p2",
    "count": 1,
    "restore": true,
    "ownSeat": 1
  },
  {
    "name": "LOF_121-fatal-empty-deck-fatigue-nine-p1",
    "count": 3,
    "deckSize": 0,
    "baseDamage": 29,
    "fatal": true,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 0
  },
  {
    "name": "LOF_121-fatal-empty-deck-fatigue-nine-p2",
    "count": 3,
    "deckSize": 0,
    "baseDamage": 29,
    "fatal": true,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "self",
        "damage": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "self",
        "damage": 0
      }
    ],
    "ownSeat": 1
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planRemainingHpDrawProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':remaining-hp-draw:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_014','SOR_014',...(design.units??[]).map(u=>u.code),...(design.boosts?['TWI_124']:[]),'TOKEN_BATTLE_DROID'])].sort(),behaviorIds:['live-remaining-hp-filtered-count','own-draw-identities-and-fatigue','unique-physical-source','printed-restore-4'],assertionStages:['paid-source-and-live-modifiers','post-unique-filtered-count','exact-own-draw-and-fatigue','restore-on-attack','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'remaining-hp-draw',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'remaining-hp-draw',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['external-draw-replacements','Finite witnesses do not establish every external draw replacement/restriction or interleaved triggered ability.'],['all-live-hp-effects','Phase HP boosts and damage distinguish remaining HP; no exhaustive aura/experience/capture/controller-transfer combinations.']].map(([dimension,reason])=>{const id=entry.setCode+':remaining-hp-draw:'+dimension;obligations.push({id,path:'remaining-hp-draw',dimension});return{id,path:'remaining-hp-draw',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planRemainingHpDrawProbes};
