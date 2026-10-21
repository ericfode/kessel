'use strict';
const {isDeepStrictEqual:same}=require('node:util');
// These are complete concern-shard entries, including diagnostics, helper bodies,
// raw metadata and intrinsic lifecycle. Unknown fields are never stripped to fit.
// The source hash also binds the activePromptTitle callback omitted by raw IR.
const reviewedSource={
  "setCode": "ASH_035",
  "sourcePath": "game/cards/08_ASH/units/TatooineRepulsorTrain.ts",
  "sourceHash": "7b34e5913965588ea82110944db0b13aefe73d978abfaf1f343eaf774b68e727",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "constant",
        "title": "This unit can't be attacked while you control 2 or more exhausted units",
        "costs": [],
        "effect": {
          "kind": "cardRestriction",
          "target": "self",
          "duration": "persistent",
          "restriction": "beAttacked"
        },
        "condition": {
          "kind": "binary",
          "op": ">=",
          "left": {
            "kind": "historyHelperCall",
            "name": "getFriendlyExhaustedUnitCount",
            "args": [
              {
                "kind": "historyAbilityContext",
                "binding": {
                  "id": "callback:839:0",
                  "role": "condition",
                  "bindAt": "callbackInvocation",
                  "readAt": "expressionEvaluation",
                  "reference": "lexicalContextObject"
                }
              }
            ]
          },
          "right": {
            "kind": "const",
            "value": 2
          }
        }
      },
      {
        "trigger": "onAttack",
        "title": "Deal 2 damage to a ground unit for each friendly exhausted unit",
        "costs": [],
        "targetResolver": {
          "owner": "any",
          "type": "unit",
          "zone": "arena",
          "arena": "ground"
        },
        "effect": {
          "kind": "damage",
          "target": "chosen",
          "amount": {
            "kind": "historyHelperCall",
            "name": "getAttackDamage",
            "args": [
              {
                "kind": "historyAbilityContext",
                "binding": {
                  "id": "callback:1467:0",
                  "role": "properties:damage",
                  "bindAt": "callbackInvocation",
                  "readAt": "expressionEvaluation",
                  "reference": "lexicalContextObject"
                }
              }
            ]
          }
        }
      }
    ]
  },
  "className": "TatooineRepulsorTrain",
  "baseClass": "NonLeaderUnitCard",
  "id": "9629363501",
  "internalName": "tatooine-repulsor-train",
  "historyHelpers": [
    {
      "kind": "historyHelper",
      "name": "getFriendlyExhaustedUnitCount",
      "parameters": [
        "data:helper:getFriendlyExhaustedUnitCount:0"
      ],
      "body": {
        "kind": "dataBlock",
        "statements": [
          {
            "kind": "dataReturn",
            "value": {
              "kind": "dataMember",
              "receiver": {
                "kind": "scopedArenaUnitsQuery",
                "receiver": {
                  "kind": "dataPath",
                  "root": [
                    "data:helper:getFriendlyExhaustedUnitCount:0"
                  ],
                  "steps": [
                    {
                      "key": "player",
                      "optional": false
                    }
                  ]
                },
                "filter": {
                  "condition": {
                    "kind": "queryCallback",
                    "parameters": [
                      {
                        "root": "attackBinding1_0",
                        "role": "candidate"
                      }
                    ],
                    "body": {
                      "kind": "ref",
                      "path": [
                        "attackBinding1_0",
                        "exhausted"
                      ]
                    }
                  }
                },
                "type": "unit",
                "controller": "receiverIdentity",
                "order": "groundThenSpacePhysicalOrder",
                "fieldCombination": "and",
                "attributeArrayCombination": "or",
                "predicateOrder": [
                  "aspect",
                  "keyword",
                  "trait",
                  "type",
                  "otherThan",
                  "condition"
                ],
                "evaluation": "expressionEvaluation"
              },
              "key": "length",
              "optional": false
            }
          }
        ]
      }
    },
    {
      "kind": "historyHelper",
      "name": "getAttackDamage",
      "parameters": [
        "data:helper:getAttackDamage:0"
      ],
      "body": {
        "kind": "dataBlock",
        "statements": [
          {
            "kind": "dataReturn",
            "value": {
              "kind": "binary",
              "op": "*",
              "left": {
                "kind": "const",
                "value": 2
              },
              "right": {
                "kind": "historyHelperCall",
                "name": "getFriendlyExhaustedUnitCount",
                "args": [
                  {
                    "kind": "dataPath",
                    "root": [
                      "data:helper:getAttackDamage:0"
                    ],
                    "steps": []
                  }
                ]
              }
            }
          }
        ]
      }
    }
  ],
  "status": "supported",
  "card": {
    "title": "Tatooine Repulsor Train",
    "subtitle": "",
    "cost": 7,
    "hp": 7,
    "power": 8,
    "text": "This unit can't be attacked while you control 2 or more exhausted units (unless it gains Sentinel).\nOn Attack: Deal 2 damage to a ground unit for each friendly exhausted unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "(ERRATA) On Attack: Choose a ground unit. Deal 2 damage to it for each friendly exhausted unit.\nIf Tatooine Repulsor Train gains Sentinel, it can be attacked, even while you control 2 or more exhausted units.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9629363501",
    "aspects": [
      "command",
      "aggression"
    ],
    "traits": [
      "underworld",
      "vehicle",
      "speeder"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "ASH",
      "number": 35
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 35
      }
    ],
    "internalName": "tatooine-repulsor-train"
  },
  "dataHash": "257442c8045023339c9656ca70d8d6ee8eb3878502a1cc503618a887ba3596c5",
  "keywordValues": {},
  "requiredCapabilities": [
    "binary:>=",
    "cardType:unit",
    "ir:binary",
    "ir:cardRestriction",
    "ir:const",
    "ir:damage",
    "ir:historyAbilityContext",
    "ir:historyHelperCall",
    "targetType:unit",
    "targetZone:arena",
    "trigger:constant",
    "trigger:onAttack"
  ],
  "diagnostics": []
};
const reviewedDonors=[
  {
    "setCode": "SOR_014",
    "sourcePath": "game/cards/01_SOR/leaders/SabineWrenGalvanizedRevolutionary.ts",
    "sourceHash": "b1d4463b24bdf9485f4b3528bde3cb1c7c328a4d1e6facdfa9923c2faf0484a4",
    "abilities": {
      "leader": [
        {
          "trigger": "action",
          "title": "Deal 1 damage to each base",
          "costs": [
            {
              "kind": "exhaustSelf"
            }
          ],
          "effect": {
            "kind": "damage",
            "target": "bothBases",
            "amount": {
              "kind": "const",
              "value": 1
            }
          }
        }
      ],
      "unit": [
        {
          "trigger": "onAttack",
          "title": "Deal 1 damage to each enemy base",
          "costs": [],
          "effect": {
            "kind": "damage",
            "target": "enemyBase",
            "amount": {
              "kind": "const",
              "value": 1
            }
          }
        }
      ],
      "card": []
    },
    "className": "SabineWrenGalvanizedRevolutionary",
    "baseClass": "LeaderUnitCard",
    "id": "4841169874",
    "internalName": "sabine-wren#galvanized-revolutionary",
    "status": "supported",
    "card": {
      "title": "Sabine Wren",
      "subtitle": "Galvanized Revolutionary",
      "cost": 4,
      "hp": 5,
      "power": 2,
      "text": "Action [exhaust]: Deal 1 damage to each base.",
      "deployBox": "On Attack: Deal 1 damage to each enemy base.",
      "epicAction": "Epic Action: If you control 4 or more resources, deploy this leader.",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4841169874",
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "mandalorian",
        "rebel",
        "spectre"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "leader"
      ],
      "setId": {
        "set": "SOR",
        "number": 14
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 14
        }
      ],
      "internalName": "sabine-wren#galvanized-revolutionary"
    },
    "dataHash": "0a8fd54a19249cd0651285a0634386aac9339700e7eac8a15bd4e248f2ed8973",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:leader",
      "ir:const",
      "ir:damage",
      "ir:exhaustSelf",
      "trigger:action",
      "trigger:onAttack"
    ],
    "diagnostics": []
  },
  {
    "setCode": "SOR_023",
    "id": "0507674993",
    "internalName": "command-center",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Command Center",
      "subtitle": null,
      "cost": null,
      "hp": 30,
      "power": null,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0507674993",
      "aspects": [
        "command"
      ],
      "traits": [
        "death star"
      ],
      "keywords": [],
      "types": [
        "base"
      ],
      "setId": {
        "set": "SOR",
        "number": 23
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 23
        }
      ],
      "internalName": "command-center"
    },
    "dataHash": "db72ddb5ebf5b34c366b8848366e96ec775213d4b2b9a416f929bab5c899ea5b",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:base"
    ],
    "diagnostics": []
  },
  {
    "setCode": "SOR_095",
    "id": "4317911650",
    "internalName": "battlefield-marine",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Battlefield Marine",
      "subtitle": null,
      "cost": 2,
      "hp": 3,
      "power": 3,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4317911650",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "rebel",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SOR",
        "number": 95
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 95
        }
      ],
      "internalName": "battlefield-marine"
    },
    "dataHash": "a8a099891788d72015624df432a47ff6e46d0e118baa98f96741dd9605c15e23",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_061",
    "id": "1029172801",
    "internalName": "strike-team-vanguard",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Strike Team Vanguard",
      "subtitle": "",
      "cost": 4,
      "hp": 5,
      "power": 5,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1029172801",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "rebel",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 61
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 61
        }
      ],
      "internalName": "strike-team-vanguard"
    },
    "dataHash": "cc298cba68428c319e40f970b7ea451838c4e3873ebe05706419e5ddee12e4c8",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_131",
    "id": "9890561214",
    "internalName": "dinosaur-turtle",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Dinosaur Turtle",
      "subtitle": "",
      "cost": 6,
      "hp": 7,
      "power": 7,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9890561214",
      "aspects": [
        "command"
      ],
      "traits": [
        "creature"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 131
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 131
        }
      ],
      "internalName": "dinosaur-turtle"
    },
    "dataHash": "89d4614e7fc53d71b1d97329d0f617756aba52452ff1371fd82eccda14dd823f",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "JTL_069",
    "id": "4789935295",
    "internalName": "munificent-frigate",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Munificent Frigate",
      "subtitle": "",
      "cost": 5,
      "hp": 7,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4789935295",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "separatist",
        "vehicle",
        "capital ship"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "JTL",
        "number": 69
      },
      "setCodes": [
        {
          "set": "JTL",
          "number": 69
        }
      ],
      "internalName": "munificent-frigate"
    },
    "dataHash": "b4b6ef3130b29a9b34911bfed32dc06a04967a89e7bab8f34596e758b9408068",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_033",
    "sourcePath": "game/cards/08_ASH/units/GrandAdmiralThrawnOrchestratingHisReturn.ts",
    "sourceHash": "5df21103804076a77412275e414cc9a0a35c1500a4f10e93fbcd2e72518e62d7",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenAttackEnds",
          "title": "If the defending unit was defeated, ready this unit",
          "costs": [],
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "historyAttackCheck",
              "test": "defenderWasDefeated",
              "attack": {
                "kind": "dataPath",
                "root": [
                  "context"
                ],
                "contextBinding": {
                  "id": "callback:1427:0",
                  "role": "condition",
                  "bindAt": "callbackInvocation",
                  "readAt": "expressionEvaluation",
                  "reference": "lexicalContextObject"
                },
                "steps": [
                  {
                    "key": "event",
                    "optional": false
                  },
                  {
                    "key": "attack",
                    "optional": false
                  }
                ]
              },
              "watcher": "cardsDefeatedThisPhase",
              "binding": "cardsDefeatedThisPhaseWatcher"
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
    "className": "GrandAdmiralThrawnOrchestratingHisReturn",
    "baseClass": "NonLeaderUnitCard",
    "stateWatchers": [
      {
        "kind": "phaseHistory",
        "binding": "cardsDefeatedThisPhaseWatcher",
        "watcher": "cardsDefeatedThisPhase",
        "sharedScope": "game",
        "registerAt": "setupStateWatchers",
        "resetEvent": "onPhaseEnded",
        "implementation": "CardsDefeatedThisPhaseWatcher",
        "queryContract": "pinnedWatcherApi"
      }
    ],
    "id": "3881321989",
    "internalName": "grand-admiral-thrawn#orchestrating-his-return",
    "status": "supported",
    "card": {
      "title": "Grand Admiral Thrawn",
      "subtitle": "Orchestrating His Return",
      "cost": 7,
      "hp": 7,
      "power": 5,
      "text": "Support (When you play this unit, you may attack with another unit. It gains this unit's other abilities for this attack.)\nWhen Attack Ends: If the defending unit was defeated, ready this unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": "When Attack Ends abilities trigger at the same time as units deal combat damage.\nIf you use Thrawn's Support ability to attack with another unit and the defending unit is defeated by that attack, the attacking unit readies.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3881321989",
      "aspects": [
        "command",
        "aggression",
        "villainy"
      ],
      "traits": [
        "imperial",
        "official"
      ],
      "arena": "ground",
      "keywords": [
        "support"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 33
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 33
        }
      ],
      "internalName": "grand-admiral-thrawn#orchestrating-his-return"
    },
    "dataHash": "04572f621e89a17496b6b65154d5cf4c082a4726593ff7aaa82d3b890fcbb59e",
    "keywordValues": {
      "support": 1
    },
    "requiredCapabilities": [
      "cardType:unit",
      "ir:conditional",
      "ir:dataPath",
      "ir:historyAttackCheck",
      "ir:noAction",
      "ir:ready",
      "keyword:support",
      "trigger:whenAttackEnds"
    ],
    "diagnostics": []
  },
  {
    "setCode": "SEC_071",
    "sourcePath": "game/cards/06_SEC/upgrades/DisciplesDevotion.ts",
    "sourceHash": "5a5be79574585d52bf7d2509f033657bbf258fc002ec346a98f1d92aa39c2d0c",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While attached unit is exhausted, it gains Sentinel",
          "costs": [],
          "effect": {
            "kind": "gainKeyword",
            "target": "parent",
            "duration": "persistent",
            "keyword": "sentinel",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
          "condition": {
            "kind": "ref",
            "path": [
              "source",
              "parentUnit",
              "exhausted"
            ]
          },
          "attachedContext": "upgrade"
        }
      ]
    },
    "className": "DisciplesDevotion",
    "baseClass": "UpgradeCard",
    "id": "0217687208",
    "internalName": "disciples-devotion",
    "status": "supported",
    "card": {
      "title": "Disciples' Devotion",
      "subtitle": "",
      "cost": 3,
      "hp": 3,
      "power": 1,
      "text": "While attached unit is exhausted, it gains Sentinel. (Enemy units in this arena must attack a Sentinel when they attack you.)",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": 1,
      "upgradeHp": 3,
      "id": "0217687208",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "innate"
      ],
      "keywords": [
        "sentinel"
      ],
      "types": [
        "upgrade"
      ],
      "setId": {
        "set": "SEC",
        "number": 71
      },
      "setCodes": [
        {
          "set": "SEC",
          "number": 71
        }
      ],
      "internalName": "disciples-devotion"
    },
    "dataHash": "d9cecbe63376c5de8994b1b0dccbcd72580d2a35cd433e3abf0830b8bae3f2b0",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:upgrade",
      "ir:const",
      "ir:gainKeyword",
      "ir:ref",
      "keyword:sentinel",
      "ref:source.parentUnit.exhausted",
      "trigger:constant"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_173",
    "id": "4513422222",
    "internalName": "shydopp-pirate-skiff",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Shydopp Pirate Skiff",
      "subtitle": "",
      "cost": 4,
      "hp": 5,
      "power": 4,
      "text": "Saboteur (When this unit attacks, ignore Sentinel and defeat the defender's Shields.)",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4513422222",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "underworld",
        "vehicle",
        "transport"
      ],
      "arena": "ground",
      "keywords": [
        "saboteur"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 173
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 173
        }
      ],
      "internalName": "shydopp-pirate-skiff"
    },
    "dataHash": "49908af86e33bbad31b31016cbb3c8367ca31ab58adedc6dffba4bc994bdf535",
    "keywordValues": {
      "saboteur": 1
    },
    "requiredCapabilities": [
      "cardType:unit",
      "keyword:saboteur"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TWI_226",
    "sourcePath": "game/cards/01_SOR/events/Waylay.ts",
    "sourceHash": "93c1da09d37785bcfad71f02ae59ca9a556ecef2c002b61e251d205fbf62ea41",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Return a non-leader unit to its owner's hand",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "nonLeaderUnit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "returnToHand",
            "target": "chosen"
          }
        }
      ]
    },
    "className": "Waylay",
    "baseClass": "EventCard",
    "id": "7202133736",
    "internalName": "waylay",
    "status": "supported",
    "card": {
      "title": "Waylay",
      "subtitle": null,
      "cost": 3,
      "hp": null,
      "power": null,
      "text": "Return a non-leader unit to its owner's hand.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": "Abilities that return a card to hand must choose a card in play unless otherwise specified.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "7202133736",
      "aspects": [
        "cunning"
      ],
      "traits": [
        "trick"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "TWI",
        "number": 226
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 222
        },
        {
          "set": "TWI",
          "number": 226
        }
      ],
      "internalName": "waylay"
    },
    "dataHash": "15cb99f5f353192a55265d6554ceb5f6fd749304bc413cb76ea5232393b6a35d",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:returnToHand",
      "targetType:nonLeaderUnit",
      "targetZone:anyAttackable",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "LAW_043",
    "sourcePath": "game/cards/07_LAW/events/ShadowCloaking.ts",
    "sourceHash": "5e8c4e3134886c781c698033dd5050287e0c55fca2ac1a1388e147a1e579ac66",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Ready a unit and give a Shield token to it",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "simultaneous",
            "effects": [
              {
                "kind": "ready",
                "target": "chosen"
              },
              {
                "kind": "giveShield",
                "target": "chosen",
                "amount": {
                  "kind": "const",
                  "value": 1
                }
              }
            ]
          }
        }
      ]
    },
    "className": "ShadowCloaking",
    "baseClass": "EventCard",
    "id": "6384323164",
    "internalName": "shadow-cloaking",
    "status": "supported",
    "card": {
      "title": "Shadow Cloaking",
      "subtitle": "",
      "cost": 5,
      "hp": null,
      "power": null,
      "text": "Ready a unit and give a Shield token to it.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6384323164",
      "aspects": [
        "vigilance",
        "aggression",
        "villainy"
      ],
      "traits": [
        "tactic"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "LAW",
        "number": 43
      },
      "setCodes": [
        {
          "set": "LAW",
          "number": 43
        }
      ],
      "internalName": "shadow-cloaking"
    },
    "dataHash": "f5c98e2e261021f06e62a13eb10db60f6408334215ea7cdbd4f19cb087c7d215",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:const",
      "ir:giveShield",
      "ir:ready",
      "ir:simultaneous",
      "targetType:unit",
      "targetZone:anyAttackable",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TWI_237",
    "sourcePath": "game/cards/03_TWI/events/DroidDeployment.ts",
    "sourceHash": "78099d889d73f4f69f9363b6dcc6e8b32e6b086e13d17c933979ad6cbb08632c",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Create 2 Battle Droid tokens",
          "costs": [],
          "effect": {
            "kind": "createTokenUnit",
            "tokenType": "battleDroid",
            "amount": {
              "kind": "const",
              "value": 2
            },
            "target": "ownPlayer",
            "entersReady": false
          }
        }
      ]
    },
    "className": "DroidDeployment",
    "baseClass": "EventCard",
    "id": "6826668370",
    "internalName": "droid-deployment",
    "status": "supported",
    "card": {
      "title": "Droid Deployment",
      "subtitle": "",
      "cost": 2,
      "hp": null,
      "power": null,
      "text": "Create 2 Battle Droid tokens.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6826668370",
      "aspects": [
        "villainy"
      ],
      "traits": [
        "supply"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "TWI",
        "number": 237
      },
      "setCodes": [
        {
          "set": "TWI",
          "number": 237
        }
      ],
      "internalName": "droid-deployment"
    },
    "dataHash": "6bfff5e99a6545d94af1bed83eb717c01096cea5ed06a7fd81b218aa611635b4",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:const",
      "ir:createTokenUnit",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TOKEN_BATTLE_DROID",
    "sourcePath": "game/cards/03_TWI/tokens/BattleDroid.ts",
    "sourceHash": "dc0c56c80105d04930c35405295126f330c935c7cc56c7c6d78da044b129eb93",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "className": "BattleDroid",
    "baseClass": "TokenUnitCard",
    "id": "3463348370",
    "internalName": "battle-droid",
    "status": "supported",
    "card": {
      "title": "Battle Droid",
      "subtitle": "",
      "cost": 0,
      "hp": 1,
      "power": 1,
      "text": "",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3463348370",
      "aspects": [
        "villainy"
      ],
      "traits": [
        "separatist",
        "droid",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "token",
        "unit"
      ],
      "setId": {
        "set": "TWI"
      },
      "internalName": "battle-droid"
    },
    "dataHash": "3913537dac211f4ab208d27763ab2b35bbc8c9ac692009e39f55fddbada947ac",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:token",
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TOKEN_SHIELD",
    "sourcePath": "game/cards/01_SOR/tokens/Shield.ts",
    "sourceHash": "39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "className": "Shield",
    "baseClass": "TokenUpgradeCard",
    "id": "8752877738",
    "internalName": "shield",
    "status": "unsupported",
    "card": {
      "title": "Shield",
      "subtitle": null,
      "cost": 0,
      "hp": 0,
      "power": 0,
      "text": "If damage would be dealt to attached unit, prevent that damage. If you do, defeat a Shield token on it.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": 0,
      "upgradeHp": 0,
      "id": "8752877738",
      "aspects": [],
      "traits": [
        "armor"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "SOR"
      },
      "internalName": "shield"
    },
    "dataHash": "3fe37b5b4c214e1a6971c2ea5e7729a7cad9bf983a384c20b56c404cbf41208b",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:token",
      "cardType:upgrade"
    ],
    "diagnostics": [
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "class-state:highPriorityRemoval",
        "file": "game/cards/01_SOR/tokens/Shield.ts",
        "line": 10,
        "column": 1,
        "message": "Class-local persistent state requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "class-member:Constructor",
        "file": "game/cards/01_SOR/tokens/Shield.ts",
        "line": 19,
        "column": 1,
        "message": "Additional class behavior requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "method:isShield",
        "file": "game/cards/01_SOR/tokens/Shield.ts",
        "line": 29,
        "column": 1,
        "message": "Additional class method requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "class-member:GetAccessor",
        "file": "game/cards/01_SOR/tokens/Shield.ts",
        "line": 33,
        "column": 1,
        "message": "Additional class behavior requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "registrar:addDamageModificationAbility",
        "message": "registrar:addDamageModificationAbility",
        "file": "game/cards/01_SOR/tokens/Shield.ts",
        "line": 38,
        "column": 9
      }
    ]
  },
  {
    "setCode": "ASH_190",
    "id": "4344365271",
    "internalName": "peridea-bandit",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Peridea Bandit",
      "subtitle": "",
      "cost": 2,
      "hp": 1,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4344365271",
      "aspects": [
        "cunning",
        "villainy"
      ],
      "traits": [
        "fringe"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 190
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 190
        }
      ],
      "internalName": "peridea-bandit"
    },
    "dataHash": "6060f76bfc42af449e87bf123e659c8797b9e1090e418155631eabca9beef83f",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  }
];
const constructions=Object.freeze([
  {
    "scenario": "paid-play7",
    "references": [],
    "assertionStages": [
      "paid-play",
      "printed-seven-payment",
      "self-enters-exhausted"
    ]
  },
  {
    "scenario": "count0-ready",
    "references": [
      "SOR_095"
    ],
    "assertionStages": [
      "count-zero",
      "can-be-attacked"
    ]
  },
  {
    "scenario": "count1-self",
    "references": [
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage"
    ]
  },
  {
    "scenario": "count2-ground",
    "references": [
      "ASH_061",
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage"
    ]
  },
  {
    "scenario": "count2-space",
    "references": [
      "JTL_069",
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "both-arenas"
    ]
  },
  {
    "scenario": "mixed-arenas",
    "references": [
      "ASH_061",
      "JTL_069",
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "both-arenas"
    ]
  },
  {
    "scenario": "enemy-negative",
    "references": [
      "ASH_061",
      "JTL_069",
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "exclude-enemy-units"
    ]
  },
  {
    "scenario": "resource-negative",
    "references": [
      "SOR_095",
      "ASH_131"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "exclude-resources-and-undeployed-leaders"
    ]
  },
  {
    "scenario": "friendly-ground-target",
    "references": [
      "SOR_095",
      "ASH_131",
      "ASH_061"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "friendly-physical-target"
    ]
  },
  {
    "scenario": "enemy-ground-target",
    "references": [
      "SOR_095",
      "ASH_131",
      "ASH_061"
    ],
    "assertionStages": [
      "current-count",
      "current-controller",
      "exhaust-before-on-attack",
      "live-prompt",
      "captured-damage",
      "enemy-physical-target"
    ]
  },
  {
    "scenario": "source-self-lethal",
    "references": [
      "ASH_061",
      "JTL_069",
      "ASH_190"
    ],
    "assertionStages": [
      "source-is-ground-target",
      "captured-self-damage",
      "source-death-before-combat",
      "no-combat-after-source-death"
    ]
  },
  {
    "scenario": "paid-sentinel-override",
    "references": [
      "SEC_071",
      "ASH_061",
      "SOR_095"
    ],
    "assertionStages": [
      "paid-upgrade-five",
      "live-sentinel",
      "sentinel-overrides-cannot-be-attacked"
    ]
  },
  {
    "scenario": "saboteur-no-override",
    "references": [
      "ASH_173",
      "ASH_061"
    ],
    "assertionStages": [
      "saboteur",
      "cannot-be-attacked-without-sentinel"
    ]
  },
  {
    "scenario": "saboteur-sentinel-override",
    "references": [
      "ASH_173",
      "SEC_071",
      "ASH_061"
    ],
    "assertionStages": [
      "paid-upgrade-five",
      "saboteur",
      "sentinel-overrides-cannot-be-attacked"
    ]
  },
  {
    "scenario": "paid-support-ready",
    "references": [
      "ASH_033",
      "ASH_131",
      "SOR_095"
    ],
    "assertionStages": [
      "paid-support-nine",
      "native-support-attack",
      "train-and-donor-counted",
      "friendly-on-attack-damage",
      "combat-defender-death",
      "copied-attack-end-ready",
      "live-count-and-restriction-drop"
    ]
  },
  {
    "scenario": "depart-reentry",
    "references": [
      "TWI_226",
      "ASH_061",
      "ASH_131",
      "SOR_095"
    ],
    "assertionStages": [
      "paid-waylay-five",
      "physical-departure",
      "live-count-drops",
      "physical-reentry",
      "live-count-restored"
    ]
  },
  {
    "scenario": "live-ready",
    "references": [
      "LAW_043",
      "TOKEN_SHIELD",
      "ASH_061"
    ],
    "assertionStages": [
      "paid-ready-nine",
      "ready-counted-friendly-unit",
      "shield-token-is-not-unit",
      "live-count-drops",
      "immediate-restriction-removal"
    ]
  },
  {
    "scenario": "live-exhaust",
    "references": [
      "LAW_043",
      "TOKEN_SHIELD",
      "ASH_061"
    ],
    "assertionStages": [
      "paid-ready-nine",
      "normal-friendly-unit-attack",
      "actual-exhaustion",
      "live-count-restored",
      "immediate-restriction-restoration"
    ]
  },
  {
    "scenario": "token-count",
    "references": [
      "TWI_237",
      "TOKEN_BATTLE_DROID"
    ],
    "assertionStages": [
      "paid-token-event-four",
      "two-actual-exhausted-token-units",
      "tokens-counted",
      "prompt-and-prepared-damage"
    ]
  },
  {
    "scenario": "deployed-leader-count",
    "references": [],
    "assertionStages": [
      "actual-leader-deployment",
      "actual-leader-unit-attack",
      "deployed-exhausted-leader-counted",
      "train-exhaust-before-on-attack"
    ]
  },
  {
    "scenario": "undeployed-leader-negative",
    "references": [],
    "assertionStages": [
      "actual-base-leader-action",
      "base-leader-exhaustion",
      "undeployed-leader-excluded",
      "train-exhaust-before-on-attack"
    ]
  }
]);
function classifyExhaustedLiveCount(entry){return same(entry,reviewedSource)?{family:'exhaustedLiveCount',sourceArena:'ground'}:null;}
function planExhaustedLiveCountProbes(entry,options={}){
 const shape=classifyExhaustedLiveCount(entry);if(!shape)return null;
 const cards=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
 const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
 const obligation=(name,reason)=>{const id=entry.setCode+':exhausted-live-count:'+name;result.obligations.push({id,path:'exhausted-live-count',dimension:name});if(reason)result.uncovered.push({id,path:'exhausted-live-count',reason});return id;};
 const donor=code=>{const entries=cards.filter(e=>e.setCode===code);return entries.length===1&&same(entries[0],reviewedDonors.find(e=>e.setCode===code));};
 for(const owner of [0,1])for(const construction of constructions){
  const scenario=construction.scenario,name='owner-'+owner+'-'+scenario,setupCardIds=[...new Set(['SOR_014','SOR_023','ASH_131','SOR_095',...construction.references])].sort();
  const missing=setupCardIds.filter(code=>!donor(code));
  if(missing.length){obligation(name,'Missing, changed or ambiguous complete construction donor: '+missing.join(', '));continue;}
  const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario,resourceCount:24,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'SOR_023',enemyBaseCode:'SOR_023'};
  result.cases.push({id:'auto/'+entry.setCode+'/exhausted-live-count-'+name,name:'exhausted-live-count-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['exhausted-live-count','current-controlled-arena-query','on-attack-damage','cannot-be-attacked'],assertionStages:[...construction.assertionStages,'terminal-drain'],coverageObligationIds:[id],probe});
 }
 obligation('source-and-timing-review','Plans are source-only constructions; require exact paired helper/raw-source projection, distinct live prompt versus prepared damage reads, independent source-bound review and actual native/Wasm comparisons before complete-card acceptance. No phase-history projection is justified.');
 for(const gap of gaps)obligation(gap.dimension,gap.reason);
 return result;
}
const gaps=[
  {
    "dimension": "attached-unit-as-upgrade-exclusion",
    "reason": "Native Piloting play lifecycle is not supported for the proposed donor; attachment import alone cannot establish this source construction."
  },
  {
    "dimension": "control-change-and-capacity",
    "reason": "No actual current-controller transfer or physical-capacity boundary recorded; equal owner/controller fixtures do not prove divergent ownership."
  },
  {
    "dimension": "prompt-versus-property-interleaving",
    "reason": "Prompt and effect property factory are distinct source reads. No actual same-window count-changing interleaving between those reads is established."
  }
];
module.exports={classifyExhaustedLiveCount,planExhaustedLiveCountProbes};
