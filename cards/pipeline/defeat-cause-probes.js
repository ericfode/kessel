'use strict';
const same = require('node:util').isDeepStrictEqual;
// Closed, source-qualified raw profile: identity is not the behavior selector.
// Keep helper AST, lexical bindings, metadata, intrinsics and all registrations.
function sourceProfile(entry) {
    const e=structuredClone(entry);
    for(const key of ['setCode','id','internalName','className']) delete e[key];
    if(e.card) for(const key of ['id','internalName','setId','setCodes']) delete e.card[key];
    return e;
}
const reviewedSource = {
  "setCode": "ASH_191",
  "sourcePath": "game/cards/08_ASH/units/ShinHatisFiendFighterCompactAndAgile.ts",
  "sourceHash": "c9e170fc79a7b952b8a9c9a3a3d9172fe08963a1fb69d8cb060d1223e1b5c69d",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenDefeated",
        "title": "Give 2 Advantage tokens to a unit, or 3 instead if this unit wasn't defeated by combat damage",
        "costs": [],
        "contextTitle": [
          {
            "kind": "const",
            "value": "Give "
          },
          {
            "kind": "historyHelperCall",
            "name": "getAdvantageTokenAmount",
            "args": [
              {
                "kind": "historyAbilityContext",
                "binding": {
                  "id": "callback:1033:0",
                  "role": "contextTitle",
                  "bindAt": "callbackInvocation",
                  "readAt": "expressionEvaluation",
                  "reference": "lexicalContextObject"
                }
              }
            ]
          },
          {
            "kind": "const",
            "value": " Advantage tokens to a unit"
          }
        ],
        "optional": true,
        "targetResolver": {
          "owner": "any",
          "type": "unit",
          "zone": "anyAttackable"
        },
        "effect": {
          "kind": "giveAdvantage",
          "target": "chosen",
          "amount": {
            "kind": "historyHelperCall",
            "name": "getAdvantageTokenAmount",
            "args": [
              {
                "kind": "historyAbilityContext",
                "binding": {
                  "id": "callback:1313:0",
                  "role": "properties:giveAdvantage",
                  "bindAt": "callbackInvocation",
                  "readAt": "expressionEvaluation",
                  "reference": "lexicalContextObject"
                }
              }
            ]
          },
          "createdBy": "self",
          "batch": "singleCreationEvent",
          "tokenOwner": "recipientController"
        }
      }
    ]
  },
  "className": "ShinHatisFiendFighterCompactAndAgile",
  "baseClass": "NonLeaderUnitCard",
  "id": "0834554190",
  "internalName": "shin-hatis-fiend-fighter#compact-and-agile",
  "historyHelpers": [
    {
      "kind": "historyHelper",
      "name": "getAdvantageTokenAmount",
      "parameters": [
        "data:helper:getAdvantageTokenAmount:0"
      ],
      "body": {
        "kind": "dataBlock",
        "statements": [
          {
            "kind": "dataReturn",
            "value": {
              "kind": "conditional",
              "condition": {
                "kind": "binary",
                "op": "===",
                "left": {
                  "kind": "dataPath",
                  "root": [
                    "data:helper:getAdvantageTokenAmount:0"
                  ],
                  "steps": [
                    {
                      "key": "event",
                      "optional": false
                    },
                    {
                      "key": "defeatSource",
                      "optional": false
                    },
                    {
                      "key": "type",
                      "optional": false
                    }
                  ]
                },
                "right": {
                  "kind": "const",
                  "value": "attack"
                }
              },
              "then": {
                "kind": "const",
                "value": 2
              },
              "else": {
                "kind": "const",
                "value": 3
              }
            }
          }
        ]
      }
    }
  ],
  "status": "supported",
  "card": {
    "title": "Shin Hati's Fiend Fighter",
    "subtitle": "Compact and Agile",
    "cost": 2,
    "hp": 1,
    "power": 3,
    "text": "When Defeated: You may give 2 Advantage tokens to a unit. If this unit wasn't defeated by combat damage, you may give 3 Advantage tokens to that unit instead.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": "“Combat damage” is only the damage dealt during the “End attack” step of an attack.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "0834554190",
    "aspects": [
      "cunning",
      "villainy"
    ],
    "traits": [
      "vehicle",
      "fighter"
    ],
    "arena": "space",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "ASH",
      "number": 191
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 191
      }
    ],
    "internalName": "shin-hatis-fiend-fighter#compact-and-agile"
  },
  "dataHash": "df791173cf7935b8f6fcf1a3c8d14bd569462ac530357b76927b4db330bd4daf",
  "keywordValues": {},
  "requiredCapabilities": [
    "cardType:unit",
    "ir:const",
    "ir:giveAdvantage",
    "ir:historyAbilityContext",
    "ir:historyHelperCall",
    "targetType:unit",
    "targetZone:anyAttackable",
    "trigger:whenDefeated"
  ],
  "diagnostics": []
};
const reviewedDonors = [
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
    "setCode": "LAW_253",
    "id": "0494601180",
    "internalName": "alliance-xwing",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Alliance X-Wing",
      "subtitle": null,
      "cost": 2,
      "hp": 3,
      "power": 2,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0494601180",
      "aspects": [
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
        "set": "LAW",
        "number": 253
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 237
        },
        {
          "set": "LAW",
          "number": 253
        }
      ],
      "internalName": "alliance-xwing"
    },
    "dataHash": "a36e1f7d4df998f48b2f7dbfef9c7c7b04d12111b515a9004548c03612834f9c",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TWI_174",
    "sourcePath": "game/cards/01_SOR/events/OpenFire.ts",
    "sourceHash": "e99017d7da9c5604552433078263b58582eb20bfd810662dc5b4510e406aa7a6",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Deal 4 damage to a unit.",
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
              "kind": "const",
              "value": 4
            }
          }
        }
      ]
    },
    "className": "OpenFire",
    "baseClass": "EventCard",
    "id": "8148673131",
    "internalName": "open-fire",
    "status": "supported",
    "card": {
      "title": "Open Fire",
      "subtitle": null,
      "cost": 3,
      "hp": null,
      "power": null,
      "text": "Deal 4 damage to a unit.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8148673131",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "tactic"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "TWI",
        "number": 174
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 172
        },
        {
          "set": "TWI",
          "number": 174
        }
      ],
      "internalName": "open-fire"
    },
    "dataHash": "fd9827786392769d33914048b3709cc018853225b8b48d890d4248341baebf57",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:const",
      "ir:damage",
      "targetType:unit",
      "targetZone:anyAttackable",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TWI_077",
    "sourcePath": "game/cards/01_SOR/events/Vanquish.ts",
    "sourceHash": "87c0e7489800419b83fb685a75b828f5d3c8dacc44102754e555614e0ff13d40",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Defeat a non-leader unit",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "nonLeaderUnit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "defeat",
            "target": "chosen"
          }
        }
      ]
    },
    "className": "Vanquish",
    "baseClass": "EventCard",
    "id": "6472095064",
    "internalName": "vanquish",
    "status": "supported",
    "card": {
      "title": "Vanquish",
      "subtitle": null,
      "cost": 5,
      "hp": null,
      "power": null,
      "text": "Defeat a non-leader unit.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6472095064",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "tactic"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "TWI",
        "number": 77
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 78
        },
        {
          "set": "TWI",
          "number": 77
        }
      ],
      "internalName": "vanquish"
    },
    "dataHash": "b451ebbca14e50aafd986e503df080a01850535d6ab6424215b7aa9af9b81ed2",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:defeat",
      "targetType:nonLeaderUnit",
      "targetZone:anyAttackable",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "LOF_144",
    "sourcePath": "game/cards/05_LOF/units/JediStarfighter.ts",
    "sourceHash": "5027aa1eafc13aee892d22ebd489c4c465bcef9d2728c02130e3f85a0afd9eed",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "Deal 1 damage to a space unit",
          "costs": [],
          "optional": true,
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "arena",
            "arena": "space"
          },
          "effect": {
            "kind": "damage",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 1
            }
          }
        }
      ]
    },
    "className": "JediStarfighter",
    "baseClass": "NonLeaderUnitCard",
    "id": "3363314608",
    "internalName": "jedi-starfighter",
    "status": "supported",
    "card": {
      "title": "Jedi Starfighter",
      "subtitle": "",
      "cost": 2,
      "hp": 4,
      "power": 1,
      "text": "On Attack: You may deal 1 damage to a space unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3363314608",
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "jedi",
        "republic",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "LOF",
        "number": 144
      },
      "setCodes": [
        {
          "set": "LOF",
          "number": 144
        }
      ],
      "internalName": "jedi-starfighter"
    },
    "dataHash": "fbbafbcdc9b6ddc8fc63bdbeee3bd103df7eda907e3b8e8eb9a804e641e5588d",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit",
      "ir:const",
      "ir:damage",
      "targetType:unit",
      "targetZone:arena",
      "trigger:onAttack"
    ],
    "diagnostics": []
  },
  {
    "setCode": "SHD_037",
    "sourcePath": "game/cards/02_SHD/units/SupremeLeaderSnokeShadowRuler.ts",
    "sourceHash": "7f15a401fed97b0662d7be1b1502cc3650592ab474fa7713cf9b4ede0c4b5567",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "Each enemy non-leader unit gets -2/-2",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "chosen",
            "duration": "persistent",
            "power": {
              "kind": "const",
              "value": -2
            },
            "hp": {
              "kind": "const",
              "value": -2
            }
          },
          "targetResolver": {
            "owner": "opponent",
            "type": "nonLeaderUnit",
            "zone": "arena"
          }
        }
      ]
    },
    "className": "SupremeLeaderSnokeShadowRuler",
    "baseClass": "NonLeaderUnitCard",
    "id": "3731235174",
    "internalName": "supreme-leader-snoke#shadow-ruler",
    "status": "supported",
    "card": {
      "title": "Supreme Leader Snoke",
      "subtitle": "Shadow Ruler",
      "cost": 8,
      "hp": 6,
      "power": 6,
      "text": "Each enemy non-leader unit gets –2/–2.",
      "deployBox": null,
      "epicAction": null,
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3731235174",
      "aspects": [
        "villainy",
        "vigilance"
      ],
      "traits": [
        "force",
        "first order",
        "official"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SHD",
        "number": 37
      },
      "setCodes": [
        {
          "set": "SHD",
          "number": 37
        }
      ],
      "internalName": "supreme-leader-snoke#shadow-ruler"
    },
    "dataHash": "255e5256db1124d322e532cf4682be07c231f1b11cfba3901da72cb2b747dafb",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit",
      "ir:const",
      "ir:modifyStats",
      "targetType:nonLeaderUnit",
      "targetZone:arena",
      "trigger:constant"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TOKEN_ADVANTAGE",
    "sourcePath": "game/cards/08_ASH/tokens/Advantage.ts",
    "sourceHash": "1280861b9a94572ecce2c1d62fb5fd93c11b08609495c3d9f60e656d153e3119",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "status": "unsupported",
    "id": "5844562972",
    "internalName": "advantage",
    "card": {
      "title": "Advantage",
      "subtitle": "",
      "cost": 0,
      "hp": 0,
      "power": 1,
      "text": "When attached unit's attack or defense ends: Defeat this upgrade.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "When Attack Ends/When Defense Ends abilities trigger at the same time as units deal combat damage.\n",
      "upgradePower": 1,
      "upgradeHp": 0,
      "id": "5844562972",
      "aspects": [],
      "traits": [
        "innate"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "ASH"
      },
      "internalName": "advantage"
    },
    "dataHash": "9a81c2d083caa13d2a5a173bdeedf76e837d62fb413496e8539338b42d71ce94",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:token",
      "cardType:upgrade"
    ],
    "diagnostics": [
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "expected-one-card-class",
        "file": "game/cards/08_ASH/tokens/Advantage.ts",
        "line": 1,
        "column": 1,
        "message": "Found 2 classes"
      }
    ]
  }
];
// Whole actual assembled donor definitions, including diagnostics (strict-profile V5).
reviewedDonors.push(...[
  {
    "setCode": "IC27_175",
    "sourcePath": "game/cards/01_SOR/events/ShootFirst.ts",
    "sourceHash": "83d0376dd417c27ad82af2959d8f0489dbcd0720c587ab0f91a5a770f9a76098",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Attack with a unit. It gets +1/+0 for this attack and deals its combat damage before the defender.",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "initiateAttack",
            "properties": {
              "kind": "attackProperties",
              "properties": {
                "target": "chosen",
                "attackerCondition": {
                  "kind": "attackCallback",
                  "parameters": [
                    {
                      "root": "attackBinding1_0",
                      "role": "candidate"
                    },
                    {
                      "root": "attackBinding1_1",
                      "role": "systemContext"
                    }
                  ],
                  "body": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "ref",
                      "path": [
                        "attackBinding1_0",
                        "controller"
                      ]
                    },
                    "right": {
                      "kind": "ref",
                      "path": [
                        "attackBinding1_1",
                        "player"
                      ]
                    }
                  }
                },
                "attackerLastingEffects": [
                  {
                    "kind": "attackLastingEntry",
                    "evaluation": "beforeDefenderSelection",
                    "effect": {
                      "kind": "modifyStats",
                      "target": {
                        "kind": "ref",
                        "path": [
                          "attackParticipant"
                        ]
                      },
                      "duration": "attack",
                      "power": {
                        "kind": "const",
                        "value": 1
                      },
                      "hp": {
                        "kind": "const",
                        "value": 0
                      }
                    }
                  },
                  {
                    "kind": "attackLastingEntry",
                    "evaluation": "beforeDefenderSelection",
                    "effect": {
                      "kind": "dealsCombatDamageFirst",
                      "target": {
                        "kind": "ref",
                        "path": [
                          "attackParticipant"
                        ]
                      },
                      "duration": "attack"
                    }
                  }
                ]
              }
            },
            "initiationProperty": true
          }
        }
      ]
    },
    "className": "ShootFirst",
    "baseClass": "EventCard",
    "id": "8297630396",
    "internalName": "shoot-first",
    "status": "supported",
    "card": {
      "title": "Shoot First",
      "subtitle": null,
      "cost": 1,
      "hp": null,
      "power": null,
      "text": "Attack with a unit. It gets +1/+0 for this attack and deals its combat damage before the defender. (If the defender is defeated, it deals no combat damage.)",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": "You must attack with a unit, if able. Units must be ready in order to attack.\n“Combat damage” is only the damage dealt during the “deal combat damage” step of an attack.\nIf the defender is defeated by the attacked, it does not deal combat damage back. If it survives and has Grit, it deals bonus damage from Grit when dealing combat damage back.\n(ERRATA) Templating update: “Defender” becomes “defending unit”.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8297630396",
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
        "set": "IC27",
        "number": 175
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 217
        },
        {
          "set": "IC27",
          "number": 175
        }
      ],
      "internalName": "shoot-first"
    },
    "dataHash": "c21b040f3ac2cb092d7c09891e767f865a257451cc67d1a00cc521f129d799ca",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:===",
      "cardType:event",
      "ir:attackCallback",
      "ir:attackLastingEntry",
      "ir:attackProperties",
      "ir:binary",
      "ir:const",
      "ir:dealsCombatDamageFirst",
      "ir:initiateAttack",
      "ir:modifyStats",
      "ir:ref",
      "ref:attackBinding1_0.controller",
      "ref:attackBinding1_1.player",
      "ref:attackParticipant",
      "targetType:any",
      "targetZone:anyAttackable",
      "trigger:event"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_197",
    "sourcePath": "game/cards/08_ASH/units/ExecutorFinalDestructionOfTheAlliance.ts",
    "sourceHash": "6f83587cb55e7e605378f5113da752cd33a8b033130a9b8fa719eaa70ba0fd45",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "This unit gets +1/+0 for each upgrade on other friendly units",
          "costs": [],
          "effect": {
            "kind": "constantEffectRegistration",
            "properties": {},
            "modifier": {
              "kind": "ongoingContract",
              "scope": "card",
              "effectType": "modifyStats",
              "target": "constantMatch",
              "duration": "persistent",
              "valueMode": "dynamic",
              "value": {
                "kind": "ongoingValueFactory",
                "callback": {
                  "kind": "pureCallback",
                  "parameters": [
                    {
                      "slot": "data:callback:819:0",
                      "argument": "targetCard",
                      "bindingPattern": {
                        "name": "target"
                      }
                    }
                  ],
                  "body": {
                    "kind": "pureBlock",
                    "statements": [
                      {
                        "kind": "pureBindings",
                        "bindings": [
                          {
                            "slot": "data:local:855",
                            "mutable": false,
                            "value": {
                              "kind": "ongoingMember",
                              "receiver": {
                                "kind": "pureCollectionCall",
                                "method": "flatMap",
                                "receiver": {
                                  "kind": "ongoingQuery",
                                  "name": "getArenaUnits",
                                  "receiver": {
                                    "kind": "dataPath",
                                    "root": [
                                      "data:callback:819:0"
                                    ],
                                    "steps": [
                                      {
                                        "key": "controller",
                                        "optional": false
                                      }
                                    ]
                                  },
                                  "args": [
                                    {
                                      "kind": "record",
                                      "properties": {
                                        "otherThan": {
                                          "kind": "dataPath",
                                          "root": [
                                            "data:callback:819:0"
                                          ],
                                          "steps": []
                                        }
                                      }
                                    }
                                  ],
                                  "optionalReceiver": false,
                                  "optionalCall": false,
                                  "continuesOptionalChain": false
                                },
                                "args": [
                                  {
                                    "kind": "pureCallback",
                                    "parameters": [
                                      {
                                        "slot": "data:callback:953:0",
                                        "argument": "item",
                                        "bindingPattern": {
                                          "name": "x"
                                        }
                                      }
                                    ],
                                    "body": {
                                      "kind": "pureReturn",
                                      "value": {
                                        "kind": "dataPath",
                                        "root": [
                                          "data:callback:953:0"
                                        ],
                                        "steps": [
                                          {
                                            "key": "upgrades",
                                            "optional": false
                                          }
                                        ]
                                      }
                                    },
                                    "capture": "lexicalReferences",
                                    "invokeAt": "nativeCallbackInvocation"
                                  }
                                ],
                                "optionalReceiver": false,
                                "optionalCall": false
                              },
                              "member": "length",
                              "optionalReceiver": false,
                              "continuesOptionalChain": false
                            },
                            "bindingPattern": {
                              "name": "upgradeCountOnOtherFriendlyUnits"
                            },
                            "evaluateAt": "declaration"
                          }
                        ]
                      },
                      {
                        "kind": "pureReturn",
                        "value": {
                          "kind": "dataRecord",
                          "fields": {
                            "power": {
                              "kind": "dataPath",
                              "root": [
                                "data:local:855"
                              ],
                              "steps": []
                            },
                            "hp": {
                              "kind": "const",
                              "value": 0
                            }
                          }
                        }
                      }
                    ]
                  },
                  "capture": "lexicalReferences",
                  "invokeAt": "nativeCallbackInvocation"
                }
              }
            },
            "duration": "persistent",
            "context": "refreshFromSourceController",
            "scopeDefaults": "pinnedOngoingCardOrPlayerEffect",
            "dependencyPolicy": "liveFixedPointRequired"
          }
        },
        {
          "trigger": "whenPlayed",
          "title": "Give an Advantage token to each other friendly unit",
          "costs": [],
          "effect": {
            "kind": "giveAdvantage",
            "target": {
              "kind": "arenaList",
              "receiver": {
                "kind": "ref",
                "path": [
                  "player"
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
                      "kind": "ref",
                      "path": [
                        "source"
                      ]
                    }
                  },
                  "right": {
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
                }
              }
            },
            "amount": {
              "kind": "const",
              "value": 1
            },
            "createdBy": "self",
            "batch": "singleCreationEvent",
            "tokenOwner": "recipientController"
          }
        }
      ]
    },
    "className": "ExecutorFinalDestructionOfTheAlliance",
    "baseClass": "NonLeaderUnitCard",
    "id": "6139617442",
    "internalName": "executor#final-destruction-of-the-alliance",
    "status": "supported",
    "card": {
      "title": "Executor",
      "subtitle": "Final Destruction of the Alliance",
      "cost": 8,
      "hp": 12,
      "power": 5,
      "text": "This unit gets +1/+0 for each upgrade on other friendly units.\nWhen Played: Give an Advantage token to each other friendly unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6139617442",
      "aspects": [
        "cunning",
        "villainy"
      ],
      "traits": [
        "imperial",
        "vehicle",
        "capital ship"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 197
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 197
        }
      ],
      "internalName": "executor#final-destruction-of-the-alliance"
    },
    "dataHash": "65550cf13bcb9cf21ee436b9351f01067a1f9174bcd2c4bdabdcdb0136454703",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:!==",
      "binary:&&",
      "binary:||",
      "cardType:unit",
      "ir:arenaList",
      "ir:binary",
      "ir:const",
      "ir:constantEffectRegistration",
      "ir:dataPath",
      "ir:dataRecord",
      "ir:giveAdvantage",
      "ir:ongoingContract",
      "ir:ongoingMember",
      "ir:ongoingQuery",
      "ir:ongoingValueFactory",
      "ir:pureBindings",
      "ir:pureBlock",
      "ir:pureCallback",
      "ir:pureCollectionCall",
      "ir:pureReturn",
      "ir:record",
      "ir:ref",
      "ir:unary",
      "ref:card",
      "ref:player",
      "ref:source",
      "trigger:constant",
      "trigger:whenPlayed",
      "unary:!"
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
  }
]);
function classifyDefeatCause(entry) {
    if(!entry || typeof entry!=='object' || Array.isArray(entry) || typeof entry.setCode!=='string') return null;
    return same(sourceProfile(entry),sourceProfile(reviewedSource)) ? {family:'defeat-cause',sourceArena:'space'} : null;
}
function planDefeatCauseProbes(entry, options={}) {
    const shape=classifyDefeatCause(entry);if(!shape) return null;
    const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name,reason){const id=entry.setCode+':defeat-cause:'+name;result.obligations.push({id,path:'defeat-cause',dimension:name});if(reason)result.uncovered.push({id,path:'defeat-cause',reason});return id;}
    obligation('source-review','A construction plan cannot prove full source/helper lowering or captured-cause execution; independent source-bound review required');
    function donor(code){const rows=catalogue.filter(e=>e.setCode===code);return rows.length===1 && same(rows[0],reviewedDonors.find(e=>e.setCode===code));}
    if(!donor('TOKEN_ADVANTAGE')){obligation('native-advantage-definition','Complete pinned native Advantage metadata/source/lifecycle definition absent, changed or ambiguous');return result;}
    const causeSetup={combat:['LAW_253'],'direct-damage':['TWI_174'],'direct-defeat':['TWI_077'],'attack-ability-damage':['LOF_144'],'stat-death':['SHD_037']};
    for(const owner of [0,1]) for(const cause of Object.keys(causeSetup)) for(const variant of ['own','enemy','decline']) {
        const name='owner-'+owner+'-'+cause+'-'+variant;
        const setupCardIds=['SOR_095',...causeSetup[cause],'TOKEN_ADVANTAGE'].sort();
        if(!setupCardIds.every(donor)){obligation(name,'Complete pinned admitted construction donor absent, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,cause,recipient:variant==='enemy'?'opponent':'self',decline:variant==='decline',resourceCount:24,expectedAmount:cause==='combat'?2:3,recipientCode:'SOR_095'};
        if(cause==='combat')probe.combatCode='LAW_253';
        if(cause==='direct-damage')probe.activationCode='TWI_174';
        if(cause==='direct-defeat')probe.activationCode='TWI_077';
        if(cause==='attack-ability-damage')probe.attackerCode='LOF_144';
        if(cause==='stat-death')probe.statCode='SHD_037';
        result.cases.push({id:'auto/'+entry.setCode+'/defeat-cause-'+name,name:'defeat-cause-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['defeat-cause','when-defeated','advantage'],assertionStages:['construction','activation','observation','phase-boundary','recipient-attack-end','terminal'],coverageObligationIds:[id],probe});
    }
    // Append only: original thirty source-derived objects above remain unchanged.
    for(const owner of [0,1])for(const cause of ['first-strike-incoming','first-strike-return','first-strike-shield']) {
        const name='owner-'+owner+'-'+cause+'-own';
        const shield=cause==='first-strike-shield',returnWindow=cause==='first-strike-return';
        const setupCardIds=['SOR_095','IC27_175',returnWindow?'ASH_197':'LAW_253','TOKEN_ADVANTAGE',...(shield?['TOKEN_SHIELD']:[])].sort();
        if(!setupCardIds.every(donor)){obligation(name,'Complete pinned admitted construction donor absent, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,cause,recipient:'self',decline:false,resourceCount:24,expectedAmount:shield?0:2,recipientCode:'SOR_095',firstStrikeCode:'IC27_175'};
        if(returnWindow)probe.defenderCode='ASH_197';else probe.attackerCode='LAW_253';
        result.cases.push({id:'auto/'+entry.setCode+'/defeat-cause-'+name,name:'defeat-cause-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['defeat-cause','when-defeated','advantage','first-strike-combat-cause'],assertionStages:['construction','activation','first-strike-step','attack-cause-attribution',...(shield?['shield-cancels-first-strike-no-death-trigger']:['observation','phase-boundary','recipient-attack-end']),'terminal'],coverageObligationIds:[id],probe});
    }
    for(const name of ['missing-or-null-cause','unique-rule-cause','source-controller-change','nested-or-competing-death'])obligation(name,'No actual source-card construction established; missing/unknown imported cause must reject rather than default to noncombat');
    return result;
}
module.exports={classifyDefeatCause,planDefeatCauseProbes};
