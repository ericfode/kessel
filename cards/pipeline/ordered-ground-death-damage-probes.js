'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "LOF_235",
  "sourcePath": "game/cards/05_LOF/units/HK87AssassinDroid.ts",
  "sourceHash": "add5c3778a8b5f07bbff3f3ea9bbce937a17d94d6c0838f642def3497a25c21c",
  "className": "HK87AssassinDroid",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "HK-87 Assassin Droid",
    "subtitle": "",
    "cost": 5,
    "hp": 4,
    "power": 4,
    "text": "When Defeated: Deal 2 damage to each ground unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "All damage dealt by HK-87 Assassin Droid's \"When Defeated\" ability is dealt simultaneously.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9160311421",
    "aspects": [
      "villainy"
    ],
    "traits": [
      "imperial",
      "droid"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "LOF",
      "number": 235
    },
    "setCodes": [
      {
        "set": "LOF",
        "number": 235
      }
    ],
    "internalName": "hk87-assassin-droid"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenDefeated",
        "title": "Deal 2 damage to each ground unit",
        "costs": [],
        "effect": {
          "kind": "damage",
          "target": {
            "kind": "arenaList",
            "receiver": {
              "kind": "ref",
              "path": [
                "game"
              ]
            },
            "predicate": {
              "kind": "binary",
              "op": "&&",
              "left": {
                "kind": "const",
                "value": true
              },
              "right": {
                "kind": "binary",
                "op": "||",
                "left": {
                  "kind": "unary",
                  "op": "!",
                  "value": {
                    "kind": "const",
                    "value": "groundArena"
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "||",
                  "left": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "const",
                      "value": "groundArena"
                    },
                    "right": {
                      "kind": "const",
                      "value": "anyArena"
                    }
                  },
                  "right": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "ref",
                      "path": [
                        "card",
                        "zoneName"
                      ]
                    },
                    "right": {
                      "kind": "const",
                      "value": "groundArena"
                    }
                  }
                }
              }
            },
            "controllerOrder": "playerFirst"
          },
          "amount": {
            "kind": "const",
            "value": 2
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
    "name": "LOF_235-interleaved-survivors-seat1",
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_093"
      },
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-interleaved-survivors-seat2",
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_093"
      },
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-own-ground-empty-seat1",
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-own-ground-empty-seat2",
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-opponent-ground-empty-seat1",
    "units": [
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-opponent-ground-empty-seat2",
    "units": [
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-no-ground-recipients-after-source-death-seat1",
    "units": [
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-no-ground-recipients-after-source-death-seat2",
    "units": [
      {
        "key": "spaceOwn",
        "owner": "self",
        "code": "LOF_119"
      },
      {
        "key": "spaceEnemy",
        "owner": "opponent",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-native-ground-deployed-leaders-space-excluded-seat1",
    "units": [
      {
        "key": "enemyToken",
        "owner": "opponent",
        "code": "TOKEN_CLONE_TROOPER",
        "token": "cloneTrooper",
        "arena": "ground"
      },
      {
        "key": "ownLeader",
        "owner": "self",
        "code": "SOR_007",
        "deployedLeader": true,
        "arena": "ground"
      },
      {
        "key": "ownToken",
        "owner": "self",
        "code": "TOKEN_BATTLE_DROID",
        "token": "battleDroid",
        "arena": "ground"
      },
      {
        "key": "enemyLeader",
        "owner": "opponent",
        "code": "SOR_007",
        "deployedLeader": true,
        "arena": "ground"
      },
      {
        "key": "spaceToken",
        "owner": "self",
        "code": "TOKEN_XWING",
        "token": "xwing",
        "arena": "space"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-native-ground-deployed-leaders-space-excluded-seat2",
    "units": [
      {
        "key": "enemyToken",
        "owner": "opponent",
        "code": "TOKEN_CLONE_TROOPER",
        "token": "cloneTrooper",
        "arena": "ground"
      },
      {
        "key": "ownLeader",
        "owner": "self",
        "code": "SOR_007",
        "deployedLeader": true,
        "arena": "ground"
      },
      {
        "key": "ownToken",
        "owner": "self",
        "code": "TOKEN_BATTLE_DROID",
        "token": "battleDroid",
        "arena": "ground"
      },
      {
        "key": "enemyLeader",
        "owner": "opponent",
        "code": "SOR_007",
        "deployedLeader": true,
        "arena": "ground"
      },
      {
        "key": "spaceToken",
        "owner": "self",
        "code": "TOKEN_XWING",
        "token": "xwing",
        "arena": "space"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-both-owner-shields-auto-physical-removal-seat1",
    "shields": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082",
        "shields": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082",
        "shields": 2
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-both-owner-shields-auto-physical-removal-seat2",
    "shields": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082",
        "shields": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082",
        "shields": 2
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-both-owner-lethal-discard-order-seat1",
    "lethal": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "SEC_028",
        "damage": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "SEC_028",
        "damage": 2
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_128",
        "damage": 3
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_128",
        "damage": 3
      },
      {
        "key": "space",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-both-owner-lethal-discard-order-seat2",
    "lethal": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "SEC_028",
        "damage": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "SEC_028",
        "damage": 2
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_128",
        "damage": 3
      },
      {
        "key": "f2",
        "owner": "self",
        "code": "HMW_128",
        "damage": 3
      },
      {
        "key": "space",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-opposing-second-hk-full-wave-before-chain-seat1",
    "chain": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "LOF_235",
        "damage": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-opposing-second-hk-full-wave-before-chain-seat2",
    "chain": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "LOF_235",
        "damage": 2
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-both-owner-vanguard-death-trigger-after-wave-seat1",
    "vanguard": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "SOR_108"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "SOR_108"
      },
      {
        "key": "xpRecipient",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-both-owner-vanguard-death-trigger-after-wave-seat2",
    "vanguard": true,
    "units": [
      {
        "key": "o1",
        "owner": "opponent",
        "code": "SOR_108"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "SOR_108"
      },
      {
        "key": "xpRecipient",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-cody-aura-damage-cap-before-removal-seat1",
    "aura": true,
    "units": [
      {
        "key": "cody",
        "owner": "self",
        "code": "TWI_114",
        "damage": 2
      },
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "fragile",
        "owner": "self",
        "code": "TOKEN_CLONE_TROOPER",
        "token": "cloneTrooper",
        "arena": "ground",
        "damage": 1
      },
      {
        "key": "filler",
        "owner": "self",
        "code": "HMW_082"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-cody-aura-damage-cap-before-removal-seat2",
    "aura": true,
    "units": [
      {
        "key": "cody",
        "owner": "self",
        "code": "TWI_114",
        "damage": 2
      },
      {
        "key": "o1",
        "owner": "opponent",
        "code": "HMW_082"
      },
      {
        "key": "fragile",
        "owner": "self",
        "code": "TOKEN_CLONE_TROOPER",
        "token": "cloneTrooper",
        "arena": "ground",
        "damage": 1
      },
      {
        "key": "filler",
        "owner": "self",
        "code": "HMW_082"
      }
    ],
    "sourceSeat": 2
  },
  {
    "name": "LOF_235-actual-combat-death-in-active-attack-seat1",
    "combat": true,
    "units": [
      {
        "key": "defender",
        "owner": "opponent",
        "code": "HMW_128"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "space",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 1
  },
  {
    "name": "LOF_235-actual-combat-death-in-active-attack-seat2",
    "combat": true,
    "units": [
      {
        "key": "defender",
        "owner": "opponent",
        "code": "HMW_128"
      },
      {
        "key": "f1",
        "owner": "self",
        "code": "HMW_082"
      },
      {
        "key": "o2",
        "owner": "opponent",
        "code": "HMW_093"
      },
      {
        "key": "space",
        "owner": "self",
        "code": "LOF_119"
      }
    ],
    "sourceSeat": 2
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planOrderedGroundDeathDamageProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':ordered-ground-death-damage:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,'SOR_007','ASH_019','SHD_079',...(design.units??[]).map(u=>u.code),'TOKEN_SHIELD'])].sort(),behaviorIds:['source-controller-first-ground-union','single-window-simultaneous-damage','deferred-lethal-aura-and-deathchains','exact-native-shield-identities'],assertionStages:['actual-paid-focus-and-death','ordered-damage-events','shield-or-deferred-lethal-cleanup','nested-death-and-combat-context','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'ordered-ground-death-damage',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'ordered-ground-death-damage',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['external-damage-replacements','Finite witnesses cover native Shield; no exhaustive other prevention/replacement or captured-controller interleavings.'],['capacity-and-all-death-combinations','Native capacity/order-import boundaries require runtime tests; finite source traces do not cover every nested death topology.']].map(([dimension,reason])=>{const id=entry.setCode+':ordered-ground-death-damage:'+dimension;obligations.push({id,path:'ordered-ground-death-damage',dimension});return{id,path:'ordered-ground-death-damage',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planOrderedGroundDeathDamageProbes};
