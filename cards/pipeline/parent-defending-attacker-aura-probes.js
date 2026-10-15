'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "SOR_071",
  "sourcePath": "game/cards/01_SOR/upgrades/Electrostaff.ts",
  "sourceHash": "eba341177494fdaa6ef24ac4da31fab05d056ec264584b91aa05a1825935e1f3",
  "className": "Electrostaff",
  "baseClass": "UpgradeCard",
  "card": {
    "title": "Electrostaff",
    "subtitle": null,
    "cost": 2,
    "hp": 2,
    "power": 2,
    "text": "Attach to a non-VEHICLE unit. \n\nWhile attached unit is defending, the attacker gets –1/–0.",
    "deployBox": null,
    "epicAction": null,
    "unique": false,
    "rules": null,
    "upgradePower": 2,
    "upgradeHp": 2,
    "id": "1323728003",
    "aspects": [
      "vigilance"
    ],
    "traits": [
      "item",
      "weapon"
    ],
    "keywords": [],
    "types": [
      "upgrade"
    ],
    "setId": {
      "set": "SOR",
      "number": 71
    },
    "setCodes": [
      {
        "set": "SOR",
        "number": 71
      }
    ],
    "internalName": "electrostaff"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "constant",
        "title": "While attached unit is defending, the attacker gets -1/-0",
        "costs": [],
        "effect": {
          "kind": "modifyStats",
          "target": "chosen",
          "duration": "persistent",
          "power": {
            "kind": "const",
            "value": -1
          },
          "hp": {
            "kind": "const",
            "value": 0
          }
        },
        "condition": {
          "kind": "query",
          "name": "isDefending",
          "receiver": {
            "kind": "ref",
            "path": [
              "source",
              "parentUnit"
            ]
          },
          "args": []
        },
        "targetResolver": {
          "owner": "opponent",
          "type": "unit",
          "zone": "arena",
          "predicate": {
            "kind": "binary",
            "op": "&&",
            "left": {
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
                "kind": "query",
                "name": "isInPlay",
                "receiver": {
                  "kind": "ref",
                  "path": [
                    "card"
                  ]
                },
                "args": []
              }
            },
            "right": {
              "kind": "query",
              "name": "isAttacking",
              "receiver": {
                "kind": "ref",
                "path": [
                  "card"
                ]
              },
              "args": []
            }
          }
        }
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": {
    "kind": "unary",
    "op": "!",
    "value": {
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
          "value": "vehicle"
        }
      ]
    }
  },
  "piloting": null
};
const designs=[
  {
    "tag": "ground-power-0",
    "attacker": "IBH_016",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "IBH_016",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "IBH_016",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-ground-power-0-p1"
  },
  {
    "tag": "ground-power-0",
    "attacker": "IBH_016",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "IBH_016",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "IBH_016",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-ground-power-0-p2"
  },
  {
    "tag": "ground-power-1",
    "attacker": "HMW_082",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-ground-power-1-p1"
  },
  {
    "tag": "ground-power-1",
    "attacker": "HMW_082",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-ground-power-1-p2"
  },
  {
    "tag": "ground-power-2",
    "attacker": "HMW_116",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-ground-power-2-p1"
  },
  {
    "tag": "ground-power-2",
    "attacker": "HMW_116",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-ground-power-2-p2"
  },
  {
    "tag": "ground-power-8",
    "attacker": "HMW_093",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-ground-power-8-p1"
  },
  {
    "tag": "ground-power-8",
    "attacker": "HMW_093",
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-ground-power-8-p2"
  },
  {
    "tag": "enemy-host-upgrade-owner-attacker",
    "enemyHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-enemy-host-upgrade-owner-attacker-p1"
  },
  {
    "tag": "enemy-host-upgrade-owner-attacker",
    "enemyHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-enemy-host-upgrade-owner-attacker-p2"
  },
  {
    "tag": "enemy-host-large-attacker",
    "enemyHost": true,
    "attacker": "HMW_093",
    "host": "HMW_082",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-enemy-host-large-attacker-p1"
  },
  {
    "tag": "enemy-host-large-attacker",
    "enemyHost": true,
    "attacker": "HMW_093",
    "host": "HMW_082",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-enemy-host-large-attacker-p2"
  },
  {
    "tag": "space-creature-host",
    "host": "LOF_121",
    "space": true,
    "attacker": "LOF_119",
    "enemyHost": false,
    "arena": "spaceArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "LOF_121",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-space-creature-host-p1"
  },
  {
    "tag": "space-creature-host",
    "host": "LOF_121",
    "space": true,
    "attacker": "LOF_119",
    "enemyHost": false,
    "arena": "spaceArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "LOF_121",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-space-creature-host-p2"
  },
  {
    "tag": "enemy-space-creature-host",
    "host": "LOF_121",
    "space": true,
    "enemyHost": true,
    "attacker": "LOF_119",
    "arena": "spaceArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "LOF_121",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-enemy-space-creature-host-p1"
  },
  {
    "tag": "enemy-space-creature-host",
    "host": "LOF_121",
    "space": true,
    "enemyHost": true,
    "attacker": "LOF_119",
    "arena": "spaceArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "LOF_121",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-enemy-space-creature-host-p2"
  },
  {
    "tag": "native-beast-host",
    "tokenHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "host",
        "name": "beast",
        "owner": "own",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-native-beast-host-p1"
  },
  {
    "tag": "native-beast-host",
    "tokenHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "host",
        "name": "beast",
        "owner": "own",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-native-beast-host-p2"
  },
  {
    "tag": "enemy-native-beast-host",
    "tokenHost": true,
    "enemyHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "host",
        "name": "beast",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-enemy-native-beast-host-p1"
  },
  {
    "tag": "enemy-native-beast-host",
    "tokenHost": true,
    "enemyHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "host",
        "name": "beast",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "own",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-enemy-native-beast-host-p2"
  },
  {
    "tag": "actual-deployed-leader-host",
    "leaderHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader"
      },
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-actual-deployed-leader-host-p1"
  },
  {
    "tag": "actual-deployed-leader-host",
    "leaderHost": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader"
      },
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-actual-deployed-leader-host-p2"
  },
  {
    "tag": "host-is-attacker",
    "hostAttacks": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "host",
        "actor": "own",
        "target": "attacker"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-host-is-attacker-p1"
  },
  {
    "tag": "host-is-attacker",
    "hostAttacks": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "host",
        "actor": "own",
        "target": "attacker"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-host-is-attacker-p2"
  },
  {
    "tag": "unrelated-unit-defender",
    "target": "unrelated",
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-unrelated-unit-defender-p1"
  },
  {
    "tag": "unrelated-unit-defender",
    "target": "unrelated",
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-unrelated-unit-defender-p2"
  },
  {
    "tag": "unrelated-base-defender",
    "target": "ownBase",
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-unrelated-base-defender-p1"
  },
  {
    "tag": "unrelated-base-defender",
    "target": "ownBase",
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-unrelated-base-defender-p2"
  },
  {
    "tag": "other-arena-attack",
    "otherArena": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "otherArena",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-other-arena-attack-p1"
  },
  {
    "tag": "other-arena-attack",
    "otherArena": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "otherArena",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-other-arena-attack-p2"
  },
  {
    "tag": "attachment-cancel",
    "cancel": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [],
    "ownSeat": 0,
    "name": "SOR_071-attachment-cancel-p1"
  },
  {
    "tag": "attachment-cancel",
    "cancel": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [],
    "ownSeat": 1,
    "name": "SOR_071-attachment-cancel-p2"
  },
  {
    "tag": "onattack-defeats-upgrade-and-later-unrelated-attack",
    "attacker": "JTL_168",
    "onAttackRemove": true,
    "later": true,
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_168",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-onattack-defeats-upgrade-and-later-unrelated-attack-p1"
  },
  {
    "tag": "onattack-defeats-upgrade-and-later-unrelated-attack",
    "attacker": "JTL_168",
    "onAttackRemove": true,
    "later": true,
    "host": "HMW_082",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_168",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-onattack-defeats-upgrade-and-later-unrelated-attack-p2"
  },
  {
    "tag": "host-combat-death",
    "host": "HMW_116",
    "attacker": "HMW_093",
    "combatDeath": true,
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-host-combat-death-p1"
  },
  {
    "tag": "host-combat-death",
    "host": "HMW_116",
    "attacker": "HMW_093",
    "combatDeath": true,
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_093",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_093",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-host-combat-death-p2"
  },
  {
    "tag": "prior-real-confiscate-upgrade",
    "removeBefore": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-prior-real-confiscate-upgrade-p1"
  },
  {
    "tag": "prior-real-confiscate-upgrade",
    "removeBefore": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-prior-real-confiscate-upgrade-p2"
  },
  {
    "tag": "actual-bounce-and-reattach",
    "bounce": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "replacementHost",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "SHD_209",
        "target": "upgrade"
      },
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 0,
    "name": "SOR_071-actual-bounce-and-reattach-p1"
  },
  {
    "tag": "actual-bounce-and-reattach",
    "bounce": true,
    "host": "HMW_082",
    "attacker": "HMW_116",
    "enemyHost": false,
    "arena": "groundArena",
    "focus": "SOR_071",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "electrostaff-parent-defense",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_116",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "vehicle",
        "code": "IBH_056",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "otherArena",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "replacementHost",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "SHD_209",
        "target": "upgrade"
      },
      {
        "code": "SOR_071",
        "target": "host"
      },
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "host"
      }
    ],
    "actualScriptedCardIds": [
      "SOR_071"
    ],
    "actualAbilityCardIds": [
      "SOR_071"
    ],
    "ownSeat": 1,
    "name": "SOR_071-actual-bounce-and-reattach-p2"
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planParentDefendingAttackerAuraProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{const obligationId=entry.setCode+':electrostaff-parent-defense:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode,design.baseCode,...design.units.map(u=>u.code),...design.plan.map(s=>s.code),...(design.tokens.length?['TOKEN_BEAST']:[])])].sort(),behaviorIds:['upgrade-owner-relative-parent-defense','nonVehicle-attachment-and-printed-stats','active-attacker-power','physical-upgrade-departure-and-reattachment'],assertionStages:['actual-paid-upgrade-or-prepay-cancel','exact-legal-nonVehicle-host-domain','upgrade-controller-independent-of-host','actual-declared-attack-and-damage','source-cache-and-framework-refresh','attack-teardown-and-terminal'],coverageObligationIds:[obligationId],probe:{family:'electrostaff-parent-defense',scenarioName:design.name,design}};});
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'electrostaff-parent-defense',dimension:c.name}));
 const uncovered=[['source-semantics','Finite actual witnesses require independent full-source/lowering review.'],['external-control-transfer','Upgrade-owner/host-owner separation witnessed both seats; control-transfer/replacement/multiple simultaneous attack interleavings not exhaustive.'],['source-cache-boundaries','Actual OnAttack removal and deferred combat-death boundaries differ; runtime cache tests and independent review must preserve these stages rather than treating all defeat observers identically.']].map(([dimension,reason])=>{const id=entry.setCode+':electrostaff-parent-defense:'+dimension;obligations.push({id,path:'electrostaff-parent-defense',dimension});return{id,path:'electrostaff-parent-defense',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planParentDefendingAttackerAuraProbes};
