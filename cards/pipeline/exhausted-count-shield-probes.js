'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "JTL_199",
  "sourcePath": "game/cards/04_JTL/units/BladeSquadronBWing.ts",
  "sourceHash": "e0148068fe58350d444362f5518c1677f0eb38b86df2f04d066e42a7b208ccdd",
  "className": "BladeSquadronBWing",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Blade Squadron B-Wing",
    "subtitle": "",
    "cost": 3,
    "hp": 3,
    "power": 3,
    "text": "When Played: If another player controls 3 or more exhausted units, give a Shield token to a unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "1965647391",
    "aspects": [
      "cunning",
      "heroism"
    ],
    "traits": [
      "rebel",
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
      "number": 199
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 199
      }
    ],
    "internalName": "blade-squadron-bwing"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Give a Shield token to a unit",
        "costs": [],
        "effect": {
          "kind": "conditional",
          "condition": {
            "kind": "binary",
            "op": ">=",
            "left": {
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
                    "op": "&&",
                    "left": {
                      "kind": "query",
                      "name": "isUnit",
                      "receiver": {
                        "kind": "ref",
                        "path": [
                          "card"
                        ]
                      },
                      "args": []
                    },
                    "right": {
                      "kind": "binary",
                      "op": "===",
                      "left": {
                        "kind": "ref",
                        "path": [
                          "card",
                          "exhausted"
                        ]
                      },
                      "right": {
                        "kind": "const",
                        "value": true
                      }
                    }
                  }
                }
              ]
            },
            "right": {
              "kind": "const",
              "value": 3
            }
          },
          "onTrue": {
            "kind": "select",
            "targetResolver": {
              "owner": "any",
              "type": "unit",
              "zone": "anyAttackable"
            },
            "effect": {
              "kind": "giveShield",
              "target": "chosen",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
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
    "name": "JTL_199-zero-opposing-units-p1",
    "count": 0,
    "units": [],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-zero-opposing-units-p2",
    "count": 0,
    "units": [],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-one-exhausted-ground-p1",
    "count": 1,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-one-exhausted-ground-p2",
    "count": 1,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-two-mixed-below-threshold-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-two-mixed-below-threshold-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-ground-self-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "focus",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-ground-self-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "focus",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-space-friendly-ground-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": "friend",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-space-friendly-ground-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": "friend",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-mixed-enemy-ground-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "a",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-mixed-enemy-ground-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "a",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-four-exhausted-enemy-space-recipient-p1",
    "count": 4,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "d",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "b",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-four-exhausted-enemy-space-recipient-p2",
    "count": 4,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "d",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "b",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-friendly-exhausted-distractors-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f",
        "code": "HMW_268",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "g",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "h",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-friendly-exhausted-distractors-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f",
        "code": "HMW_268",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "g",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "h",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-enemy-distractors-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "ready",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "readyGround",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-enemy-distractors-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "ready",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "readyGround",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-ready-enemies-do-not-count-p1",
    "count": 0,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-three-ready-enemies-do-not-count-p2",
    "count": 0,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-native-ground-space-tokens-count-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid",
        "owner": "enemy",
        "exhausted": true
      },
      {
        "key": "tie",
        "name": "tieFighter",
        "arena": "space",
        "owner": "enemy",
        "exhausted": true
      }
    ],
    "target": "tie",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-native-ground-space-tokens-count-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid",
        "owner": "enemy",
        "exhausted": true
      },
      {
        "key": "tie",
        "name": "tieFighter",
        "arena": "space",
        "owner": "enemy",
        "exhausted": true
      }
    ],
    "target": "tie",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-token-does-not-count-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "readyToken",
        "name": "xwing",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-token-does-not-count-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "readyToken",
        "name": "xwing",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-friendly-exhausted-tokens-do-not-count-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "ownToken",
        "name": "battleDroid",
        "exhausted": true
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-friendly-exhausted-tokens-do-not-count-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "ownToken",
        "name": "battleDroid",
        "exhausted": true
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-friendly-space-token-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "ownToken",
        "name": "xwing",
        "arena": "space",
        "exhausted": false
      }
    ],
    "target": "ownToken",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-friendly-space-token-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "ownToken",
        "name": "xwing",
        "arena": "space",
        "exhausted": false
      }
    ],
    "target": "ownToken",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-enemy-ground-token-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "readyToken",
        "name": "battleDroid",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "target": "readyToken",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-ready-enemy-ground-token-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "readyToken",
        "name": "battleDroid",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "target": "readyToken",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-already-shielded-enemy-stacks-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 1
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "a",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-already-shielded-enemy-stacks-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 1
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": "a",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-deployed-own-leader-recipient-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "deployedLeader": true,
    "target": "leader",
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-deployed-own-leader-recipient-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "JTL_135",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "deployedLeader": true,
    "target": "leader",
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-base-side-leader-does-not-count-p1",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-base-side-leader-does-not-count-p2",
    "count": 2,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-actual-opponent-deploy-attack-exhaust-counts-p1",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "enemyLeader": true,
    "target": "leader",
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy",
        "target": "ownBase"
      },
      {
        "code": "JTL_199"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-actual-opponent-deploy-attack-exhaust-counts-p2",
    "count": 3,
    "units": [
      {
        "key": "a",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "enemyLeader": true,
    "target": "leader",
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy",
        "target": "ownBase"
      },
      {
        "code": "JTL_199"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-live-recount-after-ambush-defeat-p1",
    "count": 3,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "victim",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ambush": true,
    "target": null,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  },
  {
    "name": "JTL_199-live-recount-after-ambush-defeat-p2",
    "count": 3,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "victim",
        "code": "JTL_212",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "c",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ambush": true,
    "target": null,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "JTL_199"
      }
    ],
    "actualScriptedCardIds": [
      "JTL_199"
    ],
    "family": "exhausted-count-shield"
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planExhaustedCountShieldProbes(entry){
if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
const tokens={battleDroid:'TOKEN_BATTLE_DROID',tieFighter:'TOKEN_TIE_FIGHTER',xwing:'TOKEN_XWING'};
const cases=designs.map(design=>{const obligationId=entry.setCode+':exhausted-count-shield:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_014','SOR_023','TOKEN_SHIELD',...(design.units??[]).map(u=>u.code),...(design.tokens??[]).map(t=>tokens[t.name])])].sort(),behaviorIds:['opposing-exhausted-unit-count','conditional-shield-selector','any-owner-unit-shield','live-count-at-resolution'],assertionStages:['real-paid-unit-entry','exact-count-threshold','exact-mandatory-unit-menu-or-false-noop','native-shield-owner-and-recipient','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'exhausted-count-shield',scenarioName:design.name,design}};});
const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'exhausted-count-shield',dimension:c.name}));
const uncovered=[['source-semantics','Finite witnesses require independent complete source/helper and compiler/runtime review'],['printed-space-leader','No printed space leader in admitted runtime pool; no synthetic arena normalization'],['external-control-or-replacement','No capture/piloting/control-transfer or Shield creation/attachment replacement interleaving constructed'],['empty-selector','Surviving entering source is itself a legal Unit Shield recipient; no unreachable empty menu manufactured'],['arbitrary-helper-shapes','Only bounded pure helper and current admitted execution semantics; snapshots, side effects, async and altered bindings remain compiler rejection obligations']].map(([dimension,reason])=>{const id=entry.setCode+':exhausted-count-shield:'+dimension;obligations.push({id,path:'exhausted-count-shield',dimension});return{id,path:'exhausted-count-shield',reason};});
return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};}
module.exports={planExhaustedCountShieldProbes};
