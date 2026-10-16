'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "HMW_136",
  "sourcePath": "game/cards/09_HMW/units/LifetreeCaravan.ts",
  "sourceHash": "ac4d9146dcfd1c82f0f63ad8bac0edc1ff3aa84552c608284e52a18d48457532",
  "className": "LifetreeCaravan",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Lifetree Caravan",
    "subtitle": "",
    "cost": 3,
    "hp": 1,
    "power": 2,
    "text": "When Played: If you control 3 or more units (including this one), you may resource the top card of your deck.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "Resources enter play exhausted.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "5032370330",
    "aspects": [
      "command"
    ],
    "traits": [
      "ewok"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "HMW",
      "number": 136
    },
    "setCodes": [
      {
        "set": "HMW",
        "number": 136
      }
    ],
    "internalName": "lifetree-caravan"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Resource the top card of your deck",
        "costs": [],
        "optional": true,
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
                  "player"
                ]
              },
              "args": [
                {
                  "kind": "const",
                  "value": true
                }
              ]
            },
            "right": {
              "kind": "const",
              "value": 3
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
const designs=[
  {
    "name": "HMW_136-source-alone-count-one-p1",
    "units": [],
    "count": 1,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-source-alone-count-one-p2",
    "units": [],
    "count": 1,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-one-other-ground-count-two-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 2,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-one-other-ground-count-two-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 2,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-entering-source-is-third-ground-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-entering-source-is-third-ground-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-four-own-units-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 4,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-four-own-units-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 4,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-two-space-units-count-p1",
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-two-space-units-count-p2",
    "units": [
      {
        "key": "a",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-mixed-arenas-count-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-mixed-arenas-count-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "LOF_119",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-exhausted-units-count-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-exhausted-units-count-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-ready-enemies-and-base-frontleader-excluded-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "f",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 2,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-ready-enemies-and-base-frontleader-excluded-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "f",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "g",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 2,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-offboard-hand-deck-resources-excluded-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "initialHandCodes": [
      "HMW_093",
      "LOF_119"
    ],
    "count": 2,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-offboard-hand-deck-resources-excluded-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "initialHandCodes": [
      "HMW_093",
      "LOF_119"
    ],
    "count": 2,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-native-ground-and-space-unit-tokens-count-p1",
    "units": [],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid"
      },
      {
        "key": "fighter",
        "name": "tieFighter",
        "arena": "space"
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-native-ground-and-space-unit-tokens-count-p2",
    "units": [],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid"
      },
      {
        "key": "fighter",
        "name": "tieFighter",
        "arena": "space"
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-enemy-native-ground-and-space-tokens-excluded-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid",
        "owner": "enemy"
      },
      {
        "key": "fighter",
        "name": "tieFighter",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "count": 2,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-enemy-native-ground-and-space-tokens-excluded-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "droid",
        "name": "battleDroid",
        "owner": "enemy"
      },
      {
        "key": "fighter",
        "name": "tieFighter",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "count": 2,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-own-native-token-third-unit-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "clone",
        "name": "cloneTrooper",
        "exhausted": true
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-own-native-token-third-unit-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "clone",
        "name": "cloneTrooper",
        "exhausted": true
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-own-actual-deployed-leader-counts-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "actualLeader": "own",
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader"
      },
      {
        "code": "HMW_136"
      }
    ],
    "count": 3,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-own-actual-deployed-leader-counts-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "actualLeader": "own",
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader"
      },
      {
        "code": "HMW_136"
      }
    ],
    "count": 3,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-opposing-actual-deployed-leader-excluded-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "actualLeader": "enemy",
    "enemyLeader": true,
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "HMW_136"
      }
    ],
    "count": 2,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-opposing-actual-deployed-leader-excluded-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "actualLeader": "enemy",
    "enemyLeader": true,
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "HMW_136"
      }
    ],
    "count": 2,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-optional-pass-preserves-top-card-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "decline": true,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-optional-pass-preserves-top-card-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "decline": true,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-empty-deck-no-fatigue-or-prompt-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "emptyOwnDeck": true,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-empty-deck-no-fatigue-or-prompt-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "emptyOwnDeck": true,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-empty-deck-and-count-two-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 2,
    "emptyOwnDeck": true,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-empty-deck-and-count-two-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 2,
    "emptyOwnDeck": true,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-captured-dead-source-live-three-resources-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "snoke",
        "code": "SHD_037",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 3,
    "sourceGone": true,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-captured-dead-source-live-three-resources-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "c",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "snoke",
        "code": "SHD_037",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 3,
    "sourceGone": true,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-captured-dead-source-live-two-no-resource-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "snoke",
        "code": "SHD_037",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 2,
    "sourceGone": true,
    "ownSeat": 0,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-captured-dead-source-live-two-no-resource-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "snoke",
        "code": "SHD_037",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "count": 2,
    "sourceGone": true,
    "ownSeat": 1,
    "leaderCode": "SOR_007",
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-aspect-penalty-real-payment-five-p1",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "leaderCode": "SOR_014",
    "baseCode": "ASH_019",
    "payment": 5,
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  },
  {
    "name": "HMW_136-aspect-penalty-real-payment-five-p2",
    "units": [
      {
        "key": "a",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false
      },
      {
        "key": "b",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false
      }
    ],
    "count": 3,
    "leaderCode": "SOR_014",
    "baseCode": "ASH_019",
    "payment": 5,
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "HMW_136"
      }
    ],
    "actualScriptedCardIds": [
      "HMW_136"
    ],
    "family": "arena-top-resource"
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planArenaTopResourceProbes(entry){
if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
const tokens={battleDroid:'TOKEN_BATTLE_DROID',cloneTrooper:'TOKEN_CLONE_TROOPER',tieFighter:'TOKEN_TIE_FIGHTER',xwing:'TOKEN_XWING'};
const cases=designs.map(design=>{const id=entry.setCode+':arena-top-resource:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_007',design.baseCode??'SOR_023',...(design.units??[]).map(t=>t.code),...design.plan.map(p=>p.code),...(design.initialHandCodes??[]),...(design.tokens??[]).map(t=>tokens[t.name])])].sort(),behaviorIds:['optional-when-played-top-resource','live-own-both-arena-unit-count','same-top-physical-id-exhausted-resource-append','captured-dead-source-live-query'],assertionStages:['actual-paid-focus-play-and-native-entry','exact-optional-trigger-pass-menu-or-no-effect-skip','physical-deck-resource-movement-without-allocation-draw-or-fatigue','both-owner-zone-and-count-exclusions','terminal-continuation'],coverageObligationIds:[id],probe:{family:'arena-top-resource',scenarioName:design.name,design}};});
const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'arena-top-resource',dimension:c.name}));
const uncovered=[['source-semantics','Finite experiments require independent complete card/helper lowering and runtime timing review; count and current top card are live at ability resolution.'],['external-resource-replacements-and-control','No approximation or credit for arbitrary resource prevention/replacement, capture, piloting or unsupported controller changes. Native identity, controller, owner and exhausted entry are asserted.'],['capacity-and-hidden-order','Finite focus games do not cover every MAX_CARDS/resource capacity or malformed hidden-zone import; runtime exact deck/resource ID movement and bounds require separate focused validation. No new ID, draw or fatigue is asserted in actual games.']].map(([dimension,reason])=>{const id=entry.setCode+':arena-top-resource:'+dimension;obligations.push({id,path:'arena-top-resource',dimension});return{id,path:'arena-top-resource',reason};});
return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};}
module.exports={planArenaTopResourceProbes};
