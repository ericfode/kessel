'use strict';
const same=require('node:util').isDeepStrictEqual;
const shapes={
  "JTL_054": {
    "setCode": "JTL_054",
    "sourcePath": "game/cards/04_JTL/units/GoldLeaderFastestShipInTheFleet.ts",
    "sourceHash": "f9bce8de66c08d0a49186758a6f6055caeb7347249155bcf4374a9f6337220ce",
    "className": "GoldLeaderFastestShipInTheFleet",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Gold Leader",
      "subtitle": "Fastest Ship in the Fleet",
      "cost": 6,
      "hp": 5,
      "power": 5,
      "text": "Shielded (When you play this unit, give a Shield token to it.)\nWhile this unit is defending, the attacker gets –1/–0.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6300552434",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "rebel",
        "vehicle",
        "transport"
      ],
      "arena": "space",
      "keywords": [
        "shielded"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "JTL",
        "number": 54
      },
      "setCodes": [
        {
          "set": "JTL",
          "number": 54
        }
      ],
      "internalName": "gold-leader#fastest-ship-in-the-fleet"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While this unit is defending, the attacker gets -1/-0",
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
                "source"
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
    "keywordValues": {
      "shielded": 1
    },
    "attachCondition": null,
    "piloting": null
  },
  "LAW_108": {
    "setCode": "LAW_108",
    "sourcePath": "game/cards/07_LAW/units/LandoCalrissianEyesOpen.ts",
    "sourceHash": "ba3993ea7ce78a9749096cc8733d9cfd794b7604f12b6041b32a59abf89f43ba",
    "className": "LandoCalrissianEyesOpen",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Lando Calrissian",
      "subtitle": "Eyes Open",
      "cost": 5,
      "hp": 5,
      "power": 4,
      "text": "Sentinel (Enemy units in this arena must attack a Sentinel when they attack you.)\nWhile this unit is defending, the attacker gets –1/–0.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2254545413",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "rebel",
        "underworld"
      ],
      "arena": "ground",
      "keywords": [
        "sentinel"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "LAW",
        "number": 108
      },
      "setCodes": [
        {
          "set": "LAW",
          "number": 108
        }
      ],
      "internalName": "lando-calrissian#eyes-open"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While this unit is defending, the attacker gets -1/-0",
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
                "source"
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
    "keywordValues": {
      "sentinel": 1
    },
    "attachCondition": null,
    "piloting": null
  }
};
const designs=[
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "paid-entry-and-defense",
    "paid": true,
    "units": [
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
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 0,
    "name": "JTL_054-paid-entry-and-defense-p1",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "paid-entry-and-defense",
    "paid": true,
    "units": [
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
        "key": "otherArena",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 1,
    "name": "JTL_054-paid-entry-and-defense-p2",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-one-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_212",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_212",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "JTL_054-ordinary-one-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-one-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_212",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_212",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "JTL_054-ordinary-one-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-two-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "SHD_060",
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
      }
    ],
    "plan": [
      {
        "code": "SHD_060",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "ownSeat": 0,
    "name": "JTL_054-ordinary-two-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-two-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "SHD_060",
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
      }
    ],
    "plan": [
      {
        "code": "SHD_060",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "ownSeat": 1,
    "name": "JTL_054-ordinary-two-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-zero-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_052",
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
      }
    ],
    "plan": [
      {
        "code": "IBH_061",
        "target": "attacker"
      },
      {
        "code": "IBH_061",
        "target": "attacker"
      },
      {
        "code": "JTL_052",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "JTL_054-ordinary-zero-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-zero-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "JTL_052",
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
      }
    ],
    "plan": [
      {
        "code": "IBH_061",
        "target": "attacker"
      },
      {
        "code": "IBH_061",
        "target": "attacker"
      },
      {
        "code": "JTL_052",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "JTL_054-ordinary-zero-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-larger-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 0,
    "name": "JTL_054-ordinary-larger-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-larger-power",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 1,
    "name": "JTL_054-ordinary-larger-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-dies-in-simultaneous-combat",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 4,
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "combatDeath": true,
    "ownSeat": 0,
    "name": "JTL_054-source-dies-in-simultaneous-combat-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-dies-in-simultaneous-combat",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 4,
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "combatDeath": true,
    "ownSeat": 1,
    "name": "JTL_054-source-dies-in-simultaneous-combat-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-tieFighter-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "tieFighter",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "native": true,
    "ownSeat": 0,
    "name": "JTL_054-native-tieFighter-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-tieFighter-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "tieFighter",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "native": true,
    "ownSeat": 1,
    "name": "JTL_054-native-tieFighter-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-xwing-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "xwing",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "native": true,
    "ownSeat": 0,
    "name": "JTL_054-native-xwing-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-xwing-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "LOF_119",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "xwing",
        "owner": "enemy",
        "arena": "space"
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "native": true,
    "ownSeat": 1,
    "name": "JTL_054-native-xwing-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-unit-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 0,
    "name": "JTL_054-unrelated-unit-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-unit-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 1,
    "name": "JTL_054-unrelated-unit-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-base-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "target": "ownBase",
    "inactive": true,
    "ownSeat": 0,
    "name": "JTL_054-unrelated-base-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-base-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "target": "ownBase",
    "inactive": true,
    "ownSeat": 1,
    "name": "JTL_054-unrelated-base-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "other-arena-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 0,
    "name": "JTL_054-other-arena-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "other-arena-target",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 1,
    "name": "JTL_054-other-arena-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-is-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054",
        "from": "focus",
        "target": "unrelated"
      }
    ],
    "sourceAttacks": true,
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 0,
    "name": "JTL_054-source-is-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-is-attacker",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054",
        "from": "focus",
        "target": "unrelated"
      }
    ],
    "sourceAttacks": true,
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 1,
    "name": "JTL_054-source-is-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "friendly-attacker-unrelated-defender",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "target": "unrelated"
      }
    ],
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 0,
    "name": "JTL_054-friendly-attacker-unrelated-defender-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "friendly-attacker-unrelated-defender",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "target": "unrelated"
      }
    ],
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 1,
    "name": "JTL_054-friendly-attacker-unrelated-defender-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "target-selection-cancel",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "cancel": true,
    "ownSeat": 0,
    "name": "JTL_054-target-selection-cancel-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "target-selection-cancel",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "cancel": true,
    "ownSeat": 1,
    "name": "JTL_054-target-selection-cancel-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "onattack-defeats-source-and-later-unrelated-attack",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_144",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_144",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      },
      {
        "code": "LOF_119",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "sourceDeath": true,
    "expected": 0,
    "ownSeat": 0,
    "name": "JTL_054-onattack-defeats-source-and-later-unrelated-attack-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "onattack-defeats-source-and-later-unrelated-attack",
    "units": [
      {
        "key": "focus",
        "code": "JTL_054",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LOF_144",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_144",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      },
      {
        "code": "LOF_119",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "sourceDeath": true,
    "expected": 0,
    "ownSeat": 1,
    "name": "JTL_054-onattack-defeats-source-and-later-unrelated-attack-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-old",
    "paid": true,
    "unique": "old",
    "units": [
      {
        "key": "oldFocus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 0,
    "name": "JTL_054-unique-keep-old-p1",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-old",
    "paid": true,
    "unique": "old",
    "units": [
      {
        "key": "oldFocus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 1,
    "name": "JTL_054-unique-keep-old-p2",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-new",
    "paid": true,
    "unique": "new",
    "units": [
      {
        "key": "oldFocus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 0,
    "name": "JTL_054-unique-keep-new-p1",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "JTL_054",
    "arena": "spaceArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-new",
    "paid": true,
    "unique": "new",
    "units": [
      {
        "key": "oldFocus",
        "code": "JTL_054",
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
      }
    ],
    "plan": [
      {
        "code": "JTL_054"
      },
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 3,
    "ownSeat": 1,
    "name": "JTL_054-unique-keep-new-p2",
    "actualScriptedCardIds": [
      "JTL_054"
    ],
    "actualAbilityCardIds": [
      "JTL_054"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "paid-entry-and-defense",
    "paid": true,
    "units": [
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
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
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "LAW_108-paid-entry-and-defense-p1",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "paid-entry-and-defense",
    "paid": true,
    "units": [
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
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
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "LAW_108-paid-entry-and-defense-p2",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-one-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "LAW_108-ordinary-one-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-one-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "LAW_108-ordinary-one-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-two-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "ownSeat": 0,
    "name": "LAW_108-ordinary-two-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-two-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_116",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "ownSeat": 1,
    "name": "LAW_108-ordinary-two-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-zero-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "IBH_016",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "LAW_108-ordinary-zero-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-zero-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "IBH_016",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "LAW_108-ordinary-zero-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-larger-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "ASH_261",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "ASH_261",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 2,
    "ownSeat": 0,
    "name": "LAW_108-ordinary-larger-power-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "ordinary-larger-power",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "ASH_261",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "ASH_261",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 2,
    "ownSeat": 1,
    "name": "LAW_108-ordinary-larger-power-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-dies-in-simultaneous-combat",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "ASH_261",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "ASH_261",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 2,
    "combatDeath": true,
    "ownSeat": 0,
    "name": "LAW_108-source-dies-in-simultaneous-combat-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-dies-in-simultaneous-combat",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "ASH_261",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "ASH_261",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 2,
    "combatDeath": true,
    "ownSeat": 1,
    "name": "LAW_108-source-dies-in-simultaneous-combat-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-battleDroid-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "battleDroid",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "native": true,
    "ownSeat": 0,
    "name": "LAW_108-native-battleDroid-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-battleDroid-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "battleDroid",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "native": true,
    "ownSeat": 1,
    "name": "LAW_108-native-battleDroid-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-cloneTrooper-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "cloneTrooper",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "native": true,
    "ownSeat": 0,
    "name": "LAW_108-native-cloneTrooper-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "native-cloneTrooper-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "tokens": [
      {
        "key": "attacker",
        "name": "cloneTrooper",
        "owner": "enemy",
        "arena": "ground"
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 1,
    "native": true,
    "ownSeat": 1,
    "name": "LAW_108-native-cloneTrooper-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-unit-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_218",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_218",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 0,
    "name": "LAW_108-unrelated-unit-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-unit-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_218",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_218",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 1,
    "name": "LAW_108-unrelated-unit-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-base-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_218",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_218",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "target": "ownBase",
    "inactive": true,
    "ownSeat": 0,
    "name": "LAW_108-unrelated-base-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unrelated-base-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_218",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_218",
        "from": "attacker",
        "actor": "enemy",
        "target": "ownBase"
      }
    ],
    "target": "ownBase",
    "inactive": true,
    "ownSeat": 1,
    "name": "LAW_108-unrelated-base-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "other-arena-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 0,
    "name": "LAW_108-other-arena-target-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "other-arena-target",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
      }
    ],
    "plan": [
      {
        "code": "LOF_119",
        "from": "attacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "target": "unrelated",
    "inactive": true,
    "ownSeat": 1,
    "name": "LAW_108-other-arena-target-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-is-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108",
        "from": "focus",
        "target": "unrelated"
      }
    ],
    "sourceAttacks": true,
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 0,
    "name": "LAW_108-source-is-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "source-is-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108",
        "from": "focus",
        "target": "unrelated"
      }
    ],
    "sourceAttacks": true,
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 1,
    "name": "LAW_108-source-is-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "friendly-attacker-unrelated-defender",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "target": "unrelated"
      }
    ],
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 0,
    "name": "LAW_108-friendly-attacker-unrelated-defender-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "friendly-attacker-unrelated-defender",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "target": "unrelated"
      }
    ],
    "inactive": true,
    "target": "unrelated",
    "ownSeat": 1,
    "name": "LAW_108-friendly-attacker-unrelated-defender-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "target-selection-cancel",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "cancel": true,
    "ownSeat": 0,
    "name": "LAW_108-target-selection-cancel-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "target-selection-cancel",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "cancel": true,
    "ownSeat": 1,
    "name": "LAW_108-target-selection-cancel-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "onattack-defeats-source-and-later-unrelated-attack",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_057",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_057",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      },
      {
        "code": "HMW_082",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "sourceDeath": true,
    "expected": 2,
    "ownSeat": 0,
    "name": "LAW_108-onattack-defeats-source-and-later-unrelated-attack-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "onattack-defeats-source-and-later-unrelated-attack",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 4,
        "exhausted": false
      },
      {
        "key": "attacker",
        "code": "LAW_057",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "laterAttacker",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_057",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      },
      {
        "code": "HMW_082",
        "from": "laterAttacker",
        "actor": "enemy",
        "target": "unrelated"
      }
    ],
    "sourceDeath": true,
    "expected": 2,
    "ownSeat": 1,
    "name": "LAW_108-onattack-defeats-source-and-later-unrelated-attack-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-old",
    "paid": true,
    "unique": "old",
    "units": [
      {
        "key": "oldFocus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "LAW_108-unique-keep-old-p1",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-old",
    "paid": true,
    "unique": "old",
    "units": [
      {
        "key": "oldFocus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "LAW_108-unique-keep-old-p2",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-new",
    "paid": true,
    "unique": "new",
    "units": [
      {
        "key": "oldFocus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 0,
    "name": "LAW_108-unique-keep-new-p1",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_014",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "unique-keep-new",
    "paid": true,
    "unique": "new",
    "units": [
      {
        "key": "oldFocus",
        "code": "LAW_108",
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
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "LAW_108"
      },
      {
        "code": "HMW_082",
        "from": "attacker",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "expected": 0,
    "ownSeat": 1,
    "name": "LAW_108-unique-keep-new-p2",
    "actualScriptedCardIds": [
      "LAW_108"
    ],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "actual-deployed-leader-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "actualLeader": true,
    "expected": 1,
    "ownSeat": 0,
    "name": "LAW_108-actual-deployed-leader-attacker-p1",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  },
  {
    "focus": "LAW_108",
    "arena": "groundArena",
    "leaderCode": "SOR_007",
    "baseCode": "SOR_021",
    "baseDamage": 0,
    "family": "defending-attacker-aura",
    "tag": "actual-deployed-leader-attacker",
    "units": [
      {
        "key": "focus",
        "code": "LAW_108",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      },
      {
        "key": "unrelated",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "exhausted": false
      }
    ],
    "plan": [
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_007",
        "from": "leader",
        "actor": "enemy",
        "target": "focus"
      }
    ],
    "actualLeader": true,
    "expected": 1,
    "ownSeat": 1,
    "name": "LAW_108-actual-deployed-leader-attacker-p2",
    "actualScriptedCardIds": [],
    "actualAbilityCardIds": [
      "LAW_108"
    ]
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planDefendingAttackerAuraProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!shapes[entry.setCode]||!same(shapes[entry.setCode],normalize(entry)))return null;
 const tokens={battleDroid:'TOKEN_BATTLE_DROID',cloneTrooper:'TOKEN_CLONE_TROOPER',tieFighter:'TOKEN_TIE_FIGHTER',xwing:'TOKEN_XWING'};
 const cases=designs.filter(d=>d.focus===entry.setCode).map(design=>{const id=entry.setCode+':defending-attacker-aura:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode,design.baseCode,...design.units.map(t=>t.code),...design.plan.map(t=>t.code),...(design.tokens??[]).map(t=>tokens[t.name])])].sort(),behaviorIds:['opponent-attacker-of-exact-defending-source','live-attack-power-minus-one-floor-zero','intrinsic-shielded-or-sentinel-whole-card','source-departure-and-attack-cleanup'],assertionStages:['actual-paid-entry-or-explicit-staged-aura-state','exact-legal-attack-target-and-cancel-menu','read-only-declaration-combat-and-source-death-boundaries','normal-damage-physical-shield-and-unique-identities','settled-cleanup-and-terminal-continuation'],coverageObligationIds:[id],probe:{family:'defending-attacker-aura',scenarioName:design.name,design}};});
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'defending-attacker-aura',dimension:c.name}));
 const uncovered=[['source-equivalence','Finite single-target source games require independent full-card/helper and exact whole-predicate lowering review; no claim for concurrent/nested/multiple-target attacks, piloting, capture or unsupported controller transfer.'],['pre-framework-cached-source-death','Actual onCardDefeated observer retains cached reduced power after source departure; framework refresh before AttackDamageResolved/End and settled legal actions is asserted. No parity claim for arbitrary unimplemented observers/effects consuming stale stats between handler and framework refresh.'],['domain-and-import-capacity','Native attackers and actual deployed ground leader are exercised. Printed space leader HMW004 is unsupported and excluded; no synthetic leader-arena normalization, generalized hidden-state import or capacity proof.'],['whole-card-keyword-and-timing','Paid Gold native Shield and Lando Sentinel are independently asserted; staged unshielded Gold liveness games are ability witnesses only. Full selected-pool native-token dependency and raw metadata fences require coordinator runtime checks.']].map(([dimension,reason])=>{const id=entry.setCode+':defending-attacker-aura:'+dimension;obligations.push({id,path:'defending-attacker-aura',dimension});return{id,path:'defending-attacker-aura',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planDefendingAttackerAuraProbes};
