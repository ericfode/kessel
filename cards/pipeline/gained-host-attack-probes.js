'use strict';
const same=require('node:util').isDeepStrictEqual;
const shapes={
  "LOF_051": {
    "setCode": "LOF_051",
    "sourcePath": "game/cards/05_LOF/upgrades/JediHolocron.ts",
    "sourceHash": "e00930d436143c9b0063f23ff7962f54af35ebf99c76b844750c4afed0e5425c",
    "className": "JediHolocron",
    "baseClass": "UpgradeCard",
    "card": {
      "title": "Jedi Holocron",
      "subtitle": "",
      "cost": 1,
      "hp": 1,
      "power": 1,
      "text": "Attach to a Force unit. \nAttached unit gains: “On Attack: You may heal 3 damage from another unit.”",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": 1,
      "upgradeHp": 1,
      "id": "7377298352",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "jedi",
        "item"
      ],
      "keywords": [],
      "types": [
        "upgrade"
      ],
      "setId": {
        "set": "LOF",
        "number": 51
      },
      "setCodes": [
        {
          "set": "LOF",
          "number": 51
        }
      ],
      "internalName": "jedi-holocron"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "You may heal 3 damage from another unit.",
          "costs": [],
          "optional": true,
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "anyAttackable",
            "predicate": {
              "kind": "binary",
              "op": "!==",
              "left": {
                "kind": "ref",
                "path": [
                  "card"
                ]
              },
              "right": {
                "kind": "ref",
                "path": [
                  "source"
                ]
              }
            }
          },
          "effect": {
            "kind": "heal",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 3
            }
          },
          "gainedHost": true
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": {
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
          "value": "force"
        }
      ]
    },
    "piloting": null
  },
  "LOF_139": {
    "setCode": "LOF_139",
    "sourcePath": "game/cards/05_LOF/upgrades/BattleFury.ts",
    "sourceHash": "406b5b44dab9dcebb4217438b55ae0e377dff7504a59f4b4ddd67c613e4911b4",
    "className": "BattleFury",
    "baseClass": "UpgradeCard",
    "card": {
      "title": "Battle Fury",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 3,
      "text": "Attached unit gains: “On Attack: Discard a card from your hand.”",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": 3,
      "upgradeHp": 3,
      "id": "4256802093",
      "aspects": [
        "aggression",
        "villainy"
      ],
      "traits": [
        "innate"
      ],
      "keywords": [],
      "types": [
        "upgrade"
      ],
      "setId": {
        "set": "LOF",
        "number": 139
      },
      "setCodes": [
        {
          "set": "LOF",
          "number": 139
        }
      ],
      "internalName": "battle-fury"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "Discard a card from your hand",
          "costs": [],
          "effect": {
            "kind": "discardOwnHand",
            "amount": {
              "kind": "const",
              "value": 1
            },
            "target": "ownPlayer"
          },
          "gainedHost": true
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null
  }
};
const designs=[
  {
    "focus": "LOF_051",
    "tag": "heal-0",
    "damage": 0,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-heal-0-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-0",
    "damage": 0,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-heal-0-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-1",
    "damage": 1,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 1,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-heal-1-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-1",
    "damage": 1,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 1,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-heal-1-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-2",
    "damage": 2,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-heal-2-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-2",
    "damage": 2,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-heal-2-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-3",
    "damage": 3,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-heal-3-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-3",
    "damage": 3,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-heal-3-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-4",
    "damage": 4,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-heal-4-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "heal-4",
    "damage": 4,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-heal-4-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-other-recipient",
    "recipientEnemy": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-enemy-other-recipient-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-other-recipient",
    "recipientEnemy": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-enemy-other-recipient-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "cross-arena-other-recipient",
    "crossArena": true,
    "damage": 4,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "LOF_121",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-cross-arena-other-recipient-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "cross-arena-other-recipient",
    "crossArena": true,
    "damage": 4,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "LOF_121",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-cross-arena-other-recipient-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-own-recipient",
    "enemyHost": true,
    "host": "HMW_128",
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-enemy-host-own-recipient-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-own-recipient",
    "enemyHost": true,
    "host": "HMW_128",
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-enemy-host-own-recipient-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-enemy-recipient",
    "enemyHost": true,
    "recipientEnemy": true,
    "host": "HMW_128",
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-enemy-host-enemy-recipient-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-enemy-recipient",
    "enemyHost": true,
    "recipientEnemy": true,
    "host": "HMW_128",
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-enemy-host-enemy-recipient-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "optional-decline",
    "decline": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-optional-decline-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "optional-decline",
    "decline": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-optional-decline-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "healthy-chosen-with-damaged-other",
    "healthy": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "key": "healthy",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-healthy-chosen-with-damaged-other-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "healthy-chosen-with-damaged-other",
    "healthy": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "key": "healthy",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-healthy-chosen-with-damaged-other-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "only-host-damaged-not-eligible",
    "damage": 0,
    "hostDamage": 3,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-only-host-damaged-not-eligible-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "only-host-damaged-not-eligible",
    "damage": 0,
    "hostDamage": 3,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-only-host-damaged-not-eligible-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "two-independent-copies",
    "copies": 2,
    "damage": 5,
    "host": "HMW_128",
    "owner": "own",
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-two-independent-copies-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "two-independent-copies",
    "copies": 2,
    "damage": 5,
    "host": "HMW_128",
    "owner": "own",
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-two-independent-copies-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "two-copies-partial-then-no-effect",
    "copies": 2,
    "damage": 2,
    "host": "HMW_128",
    "owner": "own",
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-two-copies-partial-then-no-effect-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "two-copies-partial-then-no-effect",
    "copies": 2,
    "damage": 2,
    "host": "HMW_128",
    "owner": "own",
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-two-copies-partial-then-no-effect-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-captured-self-defeat",
    "host": "TWI_165",
    "hostDamage": 5,
    "captureKill": true,
    "enemyHost": true,
    "recipientEnemy": true,
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "TWI_165",
        "owner": "enemy",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "TWI_165",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-enemy-host-captured-self-defeat-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "enemy-host-captured-self-defeat",
    "host": "TWI_165",
    "hostDamage": 5,
    "captureKill": true,
    "enemyHost": true,
    "recipientEnemy": true,
    "owner": "enemy",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "TWI_165",
        "owner": "enemy",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "TWI_165",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-enemy-host-captured-self-defeat-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "printed-host-heal-coexists",
    "host": "HMW_261",
    "damage": 5,
    "printedHeal": true,
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_261",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_261",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-printed-host-heal-coexists-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "printed-host-heal-coexists",
    "host": "HMW_261",
    "damage": 5,
    "printedHeal": true,
    "owner": "own",
    "copies": 1,
    "heal": true,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_261",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_261",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-printed-host-heal-coexists-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "captured-host-self-defeat",
    "host": "TWI_165",
    "hostDamage": 5,
    "captureKill": true,
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "TWI_165",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "TWI_165",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-captured-host-self-defeat-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "captured-host-self-defeat",
    "host": "TWI_165",
    "hostDamage": 5,
    "captureKill": true,
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "TWI_165",
        "owner": "own",
        "damage": 5,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "TWI_165",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-captured-host-self-defeat-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "prior-upgrade-removed",
    "removeBefore": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-prior-upgrade-removed-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "prior-upgrade-removed",
    "removeBefore": true,
    "host": "HMW_128",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_128",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_128",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-prior-upgrade-removed-p2"
  },
  {
    "focus": "LOF_051",
    "tag": "actual-force-leader-host",
    "leaderHost": true,
    "host": "HMW_007",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "HMW_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "HMW_007",
        "from": "leader"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_007",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 0,
    "name": "LOF_051-actual-force-leader-host-p1"
  },
  {
    "focus": "LOF_051",
    "tag": "actual-force-leader-host",
    "leaderHost": true,
    "host": "HMW_007",
    "owner": "own",
    "copies": 1,
    "heal": true,
    "damage": 3,
    "leaderCode": "HMW_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "HMW_007",
        "from": "leader"
      },
      {
        "code": "LOF_051",
        "target": "host"
      },
      {
        "code": "HMW_007",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_051"
    ],
    "actualAbilityCardIds": [
      "LOF_051"
    ],
    "ownSeat": 1,
    "name": "LOF_051-actual-force-leader-host-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-0",
    "hand": 0,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-own-host-hand-0-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-0",
    "hand": 0,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-own-host-hand-0-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-0",
    "hand": 0,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-enemy-host-hand-0-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-0",
    "hand": 0,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-enemy-host-hand-0-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-1",
    "hand": 1,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-own-host-hand-1-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-1",
    "hand": 1,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-own-host-hand-1-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-1",
    "hand": 1,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-enemy-host-hand-1-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-1",
    "hand": 1,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-enemy-host-hand-1-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-2",
    "hand": 2,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-own-host-hand-2-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "own-host-hand-2",
    "hand": 2,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-own-host-hand-2-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-2",
    "hand": 2,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-enemy-host-hand-2-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-hand-2",
    "hand": 2,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-enemy-host-hand-2-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "two-copies-hand-two",
    "copies": 2,
    "hand": 2,
    "host": "HMW_082",
    "owner": "own",
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-two-copies-hand-two-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "two-copies-hand-two",
    "copies": 2,
    "hand": 2,
    "host": "HMW_082",
    "owner": "own",
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-two-copies-hand-two-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "two-copies-hand-one",
    "copies": 2,
    "hand": 1,
    "host": "HMW_082",
    "owner": "own",
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-two-copies-hand-one-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "two-copies-hand-one",
    "copies": 2,
    "hand": 1,
    "host": "HMW_082",
    "owner": "own",
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-two-copies-hand-one-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-draw-first",
    "host": "LAW_107",
    "hand": 1,
    "printedDraw": true,
    "order": "draw-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-printed-draw-first-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-draw-first",
    "host": "LAW_107",
    "hand": 1,
    "printedDraw": true,
    "order": "draw-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-printed-draw-first-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-draw-from-empty",
    "host": "LAW_107",
    "hand": 0,
    "printedDraw": true,
    "order": "draw-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-printed-draw-from-empty-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-draw-from-empty",
    "host": "LAW_107",
    "hand": 0,
    "printedDraw": true,
    "order": "draw-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-printed-draw-from-empty-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-captured-upgrade-defeat",
    "host": "JTL_168",
    "hand": 2,
    "removeCaptured": true,
    "enemyHost": true,
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "JTL_168",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-enemy-host-captured-upgrade-defeat-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-host-captured-upgrade-defeat",
    "host": "JTL_168",
    "hand": 2,
    "removeCaptured": true,
    "enemyHost": true,
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "JTL_168",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-enemy-host-captured-upgrade-defeat-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-discard-first",
    "host": "LAW_107",
    "hand": 1,
    "printedDraw": true,
    "order": "discard-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-printed-discard-first-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "printed-discard-first",
    "host": "LAW_107",
    "hand": 1,
    "printedDraw": true,
    "order": "discard-first",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LAW_107",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LAW_107",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-printed-discard-first-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "captured-upgrade-defeat",
    "host": "JTL_168",
    "hand": 2,
    "removeCaptured": true,
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "JTL_168",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-captured-upgrade-defeat-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "captured-upgrade-defeat",
    "host": "JTL_168",
    "hand": 2,
    "removeCaptured": true,
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "JTL_168",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "JTL_168",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-captured-upgrade-defeat-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "prior-upgrade-removed",
    "hand": 2,
    "removeBefore": true,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-prior-upgrade-removed-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "prior-upgrade-removed",
    "hand": 2,
    "removeBefore": true,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "SHD_262",
        "target": "upgrade"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-prior-upgrade-removed-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "native-beast-host",
    "hand": 2,
    "tokenHost": true,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-native-beast-host-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "native-beast-host",
    "hand": 2,
    "tokenHost": true,
    "host": "HMW_082",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-native-beast-host-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-native-beast-host",
    "hand": 2,
    "tokenHost": true,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-enemy-native-beast-host-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "enemy-native-beast-host",
    "hand": 2,
    "tokenHost": true,
    "enemyHost": true,
    "host": "HMW_082",
    "owner": "enemy",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_082",
        "from": "host",
        "actor": "enemy",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-enemy-native-beast-host-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "space-vehicle-host",
    "hand": 2,
    "host": "LOF_119",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-space-vehicle-host-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "space-vehicle-host",
    "hand": 2,
    "host": "LOF_119",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "SOR_011",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "host",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "LOF_119",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-space-vehicle-host-p2"
  },
  {
    "focus": "LOF_139",
    "tag": "actual-force-leader-host",
    "hand": 2,
    "leaderHost": true,
    "host": "HMW_007",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "HMW_007",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "HMW_007",
        "from": "leader"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_007",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 0,
    "name": "LOF_139-actual-force-leader-host-p1"
  },
  {
    "focus": "LOF_139",
    "tag": "actual-force-leader-host",
    "hand": 2,
    "leaderHost": true,
    "host": "HMW_007",
    "owner": "own",
    "copies": 1,
    "heal": false,
    "damage": 0,
    "leaderCode": "HMW_007",
    "baseCode": "SOR_026",
    "baseDamage": 0,
    "family": "gained-host-attack",
    "units": [
      {
        "key": "recipient",
        "code": "HMW_116",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "extra",
        "code": "IBH_063",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "nonForce",
        "code": "HMW_116",
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
      }
    ],
    "tokens": [],
    "plan": [
      {
        "code": "HMW_007",
        "from": "leader"
      },
      {
        "code": "LOF_139",
        "target": "host"
      },
      {
        "code": "HMW_007",
        "from": "host",
        "actor": "own",
        "target": "attackBase"
      }
    ],
    "actualScriptedCardIds": [
      "LOF_139"
    ],
    "actualAbilityCardIds": [
      "LOF_139"
    ],
    "ownSeat": 1,
    "name": "LOF_139-actual-force-leader-host-p2"
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planGainedHostAttackProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!Object.hasOwn(shapes,entry.setCode)||!same(shapes[entry.setCode],normalize(entry)))return null;
 const cases=designs.filter(d=>d.focus===entry.setCode).map(design=>{const obligationId=entry.setCode+':gained-host-attack:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode,design.baseCode,...design.units.map(u=>u.code),...design.plan.map(p=>p.code),...(design.tokens.length?['TOKEN_BEAST']:[])])].sort(),behaviorIds:['physical-host-source-and-controller','independent-gained-onattack-registrations','captured-grant-survives-departure',design.heal?'optional-other-unit-heal':'mandatory-host-player-hand-discard'],assertionStages:['actual-paid-upgrade-attachment','exact-host-identity-and-legal-target-domain','actual-host-attack-registration-window','printed-and-gained-trigger-order','source-capture-departure-and-live-resolution','complete-terminal-lifecycle'],coverageObligationIds:[obligationId],probe:{family:'gained-host-attack',scenarioName:design.name,design}};});
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'gained-host-attack',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source witnesses require independent full-source/lowering and shared-runtime review.'],['initial-stat-death','Captured printed-OnAttack host/grant departures witnessed, but initial attack-start stat death without printedOnAttack unconstructed.'],['control-transfer-replacements','Upgradepayer/hostcontroller separation both seats witnessed; later controltransfer/externalreplacement interleavings not exhaustive.'],...(entry.setCode==='LOF_051'?[['space-force-host','No admitted Force spacehost in setup pool; actual ground/deployedForcehost plus crossarena healtarget covered.']]:[])].map(([dimension,reason])=>{const id=entry.setCode+':gained-host-attack:'+dimension;obligations.push({id,path:'gained-host-attack',dimension});return{id,path:'gained-host-attack',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planGainedHostAttackProbes};
