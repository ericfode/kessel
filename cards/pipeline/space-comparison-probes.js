'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "JTL_135",
  "sourcePath": "game/cards/04_JTL/units/SpecialForcesTIEFighter.ts",
  "sourceHash": "00e9be851873e2afb15af348cd142f8dbfdd0b0e1c228747677c1c4d67cfdc43",
  "className": "SpecialForcesTIEFighter",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Special Forces TIE Fighter",
    "subtitle": "",
    "cost": 2,
    "hp": 3,
    "power": 2,
    "text": "When Played: If an opponent controls more space units than you, ready this unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9595057518",
    "aspects": [
      "aggression",
      "villainy"
    ],
    "traits": [
      "first order",
      "vehicle",
      "fighter"
    ],
    "arena": "space",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "JTL",
      "number": 135
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 135
      }
    ],
    "internalName": "special-forces-tie-fighter"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Ready this unit",
        "costs": [],
        "effect": {
          "kind": "conditional",
          "condition": {
            "kind": "binary",
            "op": "<",
            "left": {
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
                    "op": "||",
                    "left": {
                      "kind": "unary",
                      "op": "!",
                      "value": {
                        "kind": "const",
                        "value": "spaceArena"
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
                          "value": "spaceArena"
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
                          "value": "spaceArena"
                        }
                      }
                    }
                  }
                }
              ]
            },
            "right": {
              "kind": "query",
              "name": "arenaUnitCount",
              "receiver": {
                "kind": "ref",
                "path": [
                  "opponent"
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
                    "op": "||",
                    "left": {
                      "kind": "unary",
                      "op": "!",
                      "value": {
                        "kind": "const",
                        "value": "spaceArena"
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
                          "value": "spaceArena"
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
                          "value": "spaceArena"
                        }
                      }
                    }
                  }
                }
              ]
            }
          },
          "onTrue": {
            "kind": "ready",
            "target": "self"
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
const designs=[
  {
    "name": "JTL_135-less-after-self-entry-p1",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-less-after-self-entry-p2",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-equal-only-after-self-entry-p1",
    "ownCount": 1,
    "enemyCount": 1,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-equal-only-after-self-entry-p2",
    "ownCount": 1,
    "enemyCount": 1,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-more-no-enemy-space-p1",
    "ownCount": 1,
    "enemyCount": 0,
    "units": [],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-more-no-enemy-space-p2",
    "ownCount": 1,
    "enemyCount": 0,
    "units": [],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-equal-existing-own-space-p1",
    "ownCount": 2,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-equal-existing-own-space-p2",
    "ownCount": 2,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-less-existing-exhausted-space-p1",
    "ownCount": 2,
    "enemyCount": 3,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": true
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-less-existing-exhausted-space-p2",
    "ownCount": 2,
    "enemyCount": 3,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": true
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-more-existing-own-space-p1",
    "ownCount": 3,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-more-existing-own-space-p2",
    "ownCount": 3,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-own-ground-distractors-excluded-p1",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "HMW_268",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "ground",
        "token": "battle-droid"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-own-ground-distractors-excluded-p2",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "HMW_268",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "ground",
        "token": "battle-droid"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-enemy-ground-distractors-excluded-p1",
    "ownCount": 1,
    "enemyCount": 0,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "ground",
        "token": "battle-droid",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-enemy-ground-distractors-excluded-p2",
    "ownCount": 1,
    "enemyCount": 0,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "ground",
        "token": "battle-droid",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-own-space-tokens-count-p1",
    "ownCount": 3,
    "enemyCount": 2,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "one",
        "token": "xwing"
      },
      {
        "key": "two",
        "token": "tie-fighter"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-own-space-tokens-count-p2",
    "ownCount": 3,
    "enemyCount": 2,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "one",
        "token": "xwing"
      },
      {
        "key": "two",
        "token": "tie-fighter"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-enemy-space-tokens-count-p1",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [],
    "initialTokens": [
      {
        "key": "one",
        "token": "xwing",
        "owner": "enemy"
      },
      {
        "key": "two",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-enemy-space-tokens-count-p2",
    "ownCount": 1,
    "enemyCount": 2,
    "units": [],
    "initialTokens": [
      {
        "key": "one",
        "token": "xwing",
        "owner": "enemy"
      },
      {
        "key": "two",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-same-printed-copy-counts-p1",
    "ownCount": 2,
    "enemyCount": 2,
    "units": [
      {
        "key": "copy",
        "code": "JTL_135",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-same-printed-copy-counts-p2",
    "ownCount": 2,
    "enemyCount": 2,
    "units": [
      {
        "key": "copy",
        "code": "JTL_135",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-deployed-ground-leader-excluded-p1",
    "ownCount": 1,
    "enemyCount": 1,
    "deployedLeader": true,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-deployed-ground-leader-excluded-p2",
    "ownCount": 1,
    "enemyCount": 1,
    "deployedLeader": true,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-base-side-leader-excluded-p1",
    "ownCount": 1,
    "enemyCount": 1,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-base-side-leader-excluded-p2",
    "ownCount": 1,
    "enemyCount": 1,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-live-recount-after-ambush-defeat-p1",
    "ownCount": 1,
    "enemyCount": 2,
    "ambush": true,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "victim",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "other",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  },
  {
    "name": "JTL_135-live-recount-after-ambush-defeat-p2",
    "ownCount": 1,
    "enemyCount": 2,
    "ambush": true,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "victim",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "initialTokens": [
      {
        "key": "other",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_135"
    ],
    "plan": [
      {
        "code": "JTL_135"
      }
    ]
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planSpaceComparisonProbes(entry){
if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
const tokens={'battle-droid':'TOKEN_BATTLE_DROID','tie-fighter':'TOKEN_TIE_FIGHTER',xwing:'TOKEN_XWING'};
const cases=designs.map(design=>{const obligationId=entry.setCode+':space-comparison:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_014','SOR_023',...(design.units??[]).map(u=>u.code),...(design.initialTokens??[]).map(t=>tokens[t.token])])].sort(),behaviorIds:['space-unit-comparison','entering-source-count','conditional-source-ready','live-count-at-resolution'],assertionStages:['real-paid-unit-entry','exact-space-count','source-ready-event-or-false-noop','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'space-comparison',scenarioName:design.name,design}};});
const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'space-comparison',dimension:c.name}));
const uncovered=[['source-semantics','Finite witnesses need independent full helper/source review'],['printed-space-leader','HMW004 is compiler/runtime unsupported; four separate engine-only setup observations establish source counting but are excluded from this planner and conformance credit'],['already-ready-entry','No admitted external play-ready grant constructed for this source; no artificial postcapture readiness mutation'],['external-control-transfer','No capture/piloting/control-transfer interleavings constructed'],['arbitrary-helper-shapes','Only bounded exact pure source helper lowered; arbitrary snapshots, side effects, callback binding and class hooks remain rejected']].map(([dimension,reason])=>{const id=entry.setCode+':space-comparison:'+dimension;obligations.push({id,path:'space-comparison',dimension});return{id,path:'space-comparison',reason};});
return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};}
module.exports={planSpaceComparisonProbes};
