'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "SEC_130",
  "sourcePath": "game/cards/06_SEC/events/FerrixUprising.ts",
  "sourceHash": "f3ce506e83a22f02da20ab855db30df86889e64d0be51c490f6c87db1a294d6b",
  "className": "FerrixUprising",
  "baseClass": "EventCard",
  "card": {
    "title": "Ferrix Uprising",
    "subtitle": "",
    "cost": 4,
    "hp": null,
    "power": null,
    "text": "Deal damage to a unit equal to twice the number of units you control in its arena.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "8492928317",
    "aspects": [
      "command"
    ],
    "traits": [
      "tactic"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "SEC",
      "number": 130
    },
    "setCodes": [
      {
        "set": "SEC",
        "number": 130
      }
    ],
    "internalName": "ferrix-uprising"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Deal damage to a unit equal to twice the number of units you control in its arena.",
        "costs": [],
        "targetResolver": {
          "owner": "any",
          "type": "unit",
          "zone": "anyAttackable"
        },
        "effect": {
          "kind": "damage",
          "target": "chosen",
          "amount": {
            "kind": "binary",
            "op": "*",
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
                        "kind": "ref",
                        "path": [
                          "target",
                          "zoneName"
                        ]
                      }
                    },
                    "right": {
                      "kind": "binary",
                      "op": "||",
                      "left": {
                        "kind": "binary",
                        "op": "===",
                        "left": {
                          "kind": "ref",
                          "path": [
                            "target",
                            "zoneName"
                          ]
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
                          "kind": "ref",
                          "path": [
                            "target",
                            "zoneName"
                          ]
                        }
                      }
                    }
                  }
                }
              ]
            },
            "right": {
              "kind": "const",
              "value": 2
            }
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
    "name": "SEC_130-enemy-ground-count-ordinary-token-leader-exclude-space-enemy-p1",
    "arena": "groundArena",
    "count": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyOther",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token",
        "token": "battle-droid",
        "owner": "own"
      },
      {
        "key": "enemyToken",
        "token": "xwing",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-enemy-ground-count-ordinary-token-leader-exclude-space-enemy-p2",
    "arena": "groundArena",
    "count": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyOther",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token",
        "token": "battle-droid",
        "owner": "own"
      },
      {
        "key": "enemyToken",
        "token": "xwing",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-enemy-space-count-ordinary-two-tokens-exclude-ground-p1",
    "arena": "spaceArena",
    "count": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "JTL_212",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyOther",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token1",
        "token": "xwing",
        "owner": "own"
      },
      {
        "key": "token2",
        "token": "tie-fighter",
        "owner": "own"
      },
      {
        "key": "enemyToken",
        "token": "battle-droid",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-enemy-space-count-ordinary-two-tokens-exclude-ground-p2",
    "arena": "spaceArena",
    "count": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "JTL_212",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyOther",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token1",
        "token": "xwing",
        "owner": "own"
      },
      {
        "key": "token2",
        "token": "tie-fighter",
        "owner": "own"
      },
      {
        "key": "enemyToken",
        "token": "battle-droid",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-friendly-ground-self-counts-p1",
    "arena": "groundArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-friendly-ground-self-counts-p2",
    "arena": "groundArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-friendly-space-self-counts-p1",
    "arena": "spaceArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-friendly-space-self-counts-p2",
    "arena": "spaceArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-mixed-zero-ground-still-legal-p1",
    "arena": "groundArena",
    "count": 0,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemySpace",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-mixed-zero-ground-still-legal-p2",
    "arena": "groundArena",
    "count": 0,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemySpace",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-mixed-zero-space-shield-preserved-p1",
    "arena": "spaceArena",
    "count": 0,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 1
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyGround",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-mixed-zero-space-shield-preserved-p2",
    "arena": "spaceArena",
    "count": 0,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 1
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemyGround",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-all-zero-no-target-no-confirmation-p1",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "enemySpace",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-all-zero-no-target-no-confirmation-p2",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "enemySpace",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-positive-damage-shield-p1",
    "arena": "groundArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 2,
        "shields": 1
      },
      {
        "key": "friend",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-positive-damage-shield-p2",
    "arena": "groundArena",
    "count": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 2,
        "shields": 1
      },
      {
        "key": "friend",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-lethal-ground-p1",
    "arena": "groundArena",
    "count": 2,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token",
        "token": "battle-droid",
        "owner": "own"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-lethal-ground-p2",
    "arena": "groundArena",
    "count": 2,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "token",
        "token": "battle-droid",
        "owner": "own"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-native-token-target-lethal-p1",
    "arena": "spaceArena",
    "count": 1,
    "units": [
      {
        "key": "friend",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "subject",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-native-token-target-lethal-p2",
    "arena": "spaceArena",
    "count": 1,
    "units": [
      {
        "key": "friend",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "subject",
        "token": "tie-fighter",
        "owner": "enemy"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-deployed-leader-target-counts-p1",
    "arena": "groundArena",
    "count": 1,
    "deployedLeader": true,
    "target": "leader",
    "units": [
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "leader"
      }
    ]
  },
  {
    "name": "SEC_130-deployed-leader-target-counts-p2",
    "arena": "groundArena",
    "count": 1,
    "deployedLeader": true,
    "target": "leader",
    "units": [
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "leader"
      }
    ]
  },
  {
    "name": "SEC_130-prepayment-cancel-p1",
    "arena": "spaceArena",
    "count": 1,
    "cancel": true,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "JTL_212",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-prepayment-cancel-p2",
    "arena": "spaceArena",
    "count": 1,
    "cancel": true,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "JTL_212",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-empty-unit-domain-play-p1",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "noEffect": true,
    "cancel": false,
    "units": [],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-empty-unit-domain-play-p2",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "noEffect": true,
    "cancel": false,
    "units": [],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-empty-unit-domain-cancel-p1",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "noEffect": true,
    "cancel": true,
    "units": [],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  },
  {
    "name": "SEC_130-empty-unit-domain-cancel-p2",
    "arena": "groundArena",
    "count": 0,
    "noMenu": true,
    "noEffect": true,
    "cancel": true,
    "units": [],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "SEC_130"
    ],
    "plan": [
      {
        "code": "SEC_130",
        "target": "subject"
      }
    ]
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planFerrixUprisingProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':ferrix-uprising:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,'SOR_014','SOR_023',...(design.units??[]).map(u=>u.code),'TOKEN_BATTLE_DROID','TOKEN_TIE_FIGHTER','TOKEN_XWING','TOKEN_SHIELD'])].sort(),behaviorIds:['chosen-target-arena-unit-count','ordinary-damage','exact-zero-target-feasibility'],assertionStages:['prepayment-targeting','real-event-payment','exact-count-and-damage','lethal-or-shield-cleanup','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'ferrix-uprising',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'ferrix-uprising',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['interleaved-count-changes','No unrelated reaction changes arena membership between target selection and property generation.'],['all-damage-replacements','Witnesses cover ordinary damage, native Shield and lethal cleanup, not every external restriction/replacement.']].map(([dimension,reason])=>{const id=entry.setCode+':ferrix-uprising:'+dimension;obligations.push({id,path:'ferrix-uprising',dimension});return{id,path:'ferrix-uprising',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planFerrixUprisingProbes};
