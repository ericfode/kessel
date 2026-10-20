'use strict';
// Closed source profile. Finite source-engine designs are construction evidence,
// never runtime admission or a claim that unobserved interactions are covered.
const same=require('node:util').isDeepStrictEqual;
const {createHash}=require('node:crypto');
const reviewedSource={
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
};
const reviewedNeeds=[
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
    "setCode": "SOR_178",
    "sourcePath": "game/cards/01_SOR/units/CartelSpacer.ts",
    "sourceHash": "43fd273cccc57082cd1e1469d8cd2ca38ab84ad426c71b942984d1eb0e6a5f16",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "If you control another Cunning unit, exhaust an enemy unit that costs 4 or less",
          "costs": [],
          "targetResolver": {
            "owner": "opponent",
            "type": "any",
            "zone": "anyAttackable",
            "predicate": {
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
                "op": "<=",
                "left": {
                  "kind": "ref",
                  "path": [
                    "card",
                    "cost"
                  ]
                },
                "right": {
                  "kind": "const",
                  "value": 4
                }
              }
            }
          },
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaUnit",
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
                    "kind": "binary",
                    "op": "&&",
                    "left": {
                      "kind": "const",
                      "value": true
                    },
                    "right": {
                      "kind": "query",
                      "name": "hasSomeAspect",
                      "receiver": {
                        "kind": "ref",
                        "path": [
                          "card"
                        ]
                      },
                      "args": [
                        {
                          "kind": "const",
                          "value": "cunning"
                        }
                      ]
                    }
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
              ]
            },
            "onTrue": {
              "kind": "exhaust",
              "target": "chosen"
            },
            "onFalse": {
              "kind": "noAction"
            }
          }
        }
      ]
    },
    "className": "CartelSpacer",
    "baseClass": "NonLeaderUnitCard",
    "id": "3802299538",
    "internalName": "cartel-spacer",
    "status": "supported",
    "card": {
      "title": "Cartel Spacer",
      "subtitle": null,
      "cost": 2,
      "hp": 3,
      "power": 2,
      "text": "When Played: If you control another [Cunning] unit, exhaust an enemy unit that costs 4 or less.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": "Abilities that refer to a card’s cost always refer to its printed cost, regardless of modifiers.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3802299538",
      "aspects": [
        "cunning",
        "villainy"
      ],
      "traits": [
        "underworld",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SOR",
        "number": 178
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 178
        }
      ],
      "internalName": "cartel-spacer"
    },
    "dataHash": "cd7f2b8eea76c118e20516eeba389cf832cee7bb270efc2e35ae3f0614ab3935",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:!==",
      "binary:&&",
      "binary:<=",
      "binary:||",
      "cardType:unit",
      "ir:binary",
      "ir:conditional",
      "ir:const",
      "ir:exhaust",
      "ir:noAction",
      "ir:query",
      "ir:ref",
      "ir:unary",
      "query:hasSomeArenaUnit",
      "query:hasSomeAspect",
      "query:isUnit",
      "ref:card",
      "ref:card.cost",
      "ref:player",
      "ref:source",
      "targetType:any",
      "targetZone:anyAttackable",
      "trigger:whenPlayed",
      "unary:!"
    ],
    "diagnostics": []
  },
  {
    "setCode": "IC27_052",
    "id": "9059202647",
    "internalName": "resilient",
    "baseClass": "UpgradeCard",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Resilient",
      "subtitle": null,
      "cost": 1,
      "hp": 3,
      "power": 0,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": 0,
      "upgradeHp": 3,
      "id": "9059202647",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "innate"
      ],
      "keywords": [],
      "types": [
        "upgrade"
      ],
      "setId": {
        "set": "IC27",
        "number": 52
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 69
        },
        {
          "set": "SHD",
          "number": 70
        },
        {
          "set": "IC27",
          "number": 52
        }
      ],
      "internalName": "resilient"
    },
    "dataHash": "77ba946015e189f48f9a773420d3ad627b6b98018b9cab9360dfdcfffb120766",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:upgrade"
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
    "setCode": "TOKEN_EXPERIENCE",
    "sourcePath": "game/cards/01_SOR/tokens/Experience.ts",
    "sourceHash": "fccf00d13061bfc15447101a0dc82fc360f9b5d2e28a9da678684e8c93bc9d67",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "className": "Experience",
    "baseClass": "TokenUpgradeCard",
    "id": "2007868442",
    "internalName": "experience",
    "status": "unsupported",
    "card": {
      "title": "Experience",
      "subtitle": null,
      "cost": 0,
      "hp": 1,
      "power": 1,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": 1,
      "upgradeHp": 1,
      "id": "2007868442",
      "aspects": [],
      "traits": [
        "learned"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "SOR"
      },
      "internalName": "experience"
    },
    "dataHash": "cc1d7a3bf3344d8a95afdbba8607659ec1edd07e5282bb9eb9d3497b0f7f332b",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:token",
      "cardType:upgrade"
    ],
    "diagnostics": [
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "method:isExperience",
        "file": "game/cards/01_SOR/tokens/Experience.ts",
        "line": 12,
        "column": 1,
        "message": "Additional class method requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "class-member:GetAccessor",
        "file": "game/cards/01_SOR/tokens/Experience.ts",
        "line": 16,
        "column": 1,
        "message": "Additional class behavior requires conversion"
      }
    ]
  }
];
const rawDesigns=[
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "empty-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 5,
        "sourceHp": 12,
        "ownAdvantageRecipients": [],
        "sourceExhausted": true
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "empty-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 5,
        "sourceHp": 12,
        "ownAdvantageRecipients": [],
        "sourceExhausted": true
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "both-arenas-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "advantageBatchCount": 1,
        "advantageGeneratedCount": 2,
        "ownAdvantageRecipients": [
          "marine",
          "spacer"
        ],
        "enemyUpgradeCount": 0,
        "sourceUpgradeCount": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "both-arenas-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "advantageBatchCount": 1,
        "advantageGeneratedCount": 2,
        "ownAdvantageRecipients": [
          "marine",
          "spacer"
        ],
        "enemyUpgradeCount": 0,
        "sourceUpgradeCount": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "mixed-inventory-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [
        {
          "host": "marine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "own"
        },
        {
          "host": "marine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "marine",
          "code": "IC27_052",
          "owner": "own"
        },
        {
          "host": "spacer",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "enemyMarine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "enemy"
        },
        {
          "host": "enemyMarine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 11,
        "marineUpgradeCount": 4,
        "spacerUpgradeCount": 2,
        "enemyUpgradeCount": 2
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "mixed-inventory-normal-play",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [
        {
          "host": "marine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "own"
        },
        {
          "host": "marine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "marine",
          "code": "IC27_052",
          "owner": "own"
        },
        {
          "host": "spacer",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "enemyMarine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "enemy"
        },
        {
          "host": "enemyMarine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 11,
        "marineUpgradeCount": 4,
        "spacerUpgradeCount": 2,
        "enemyUpgradeCount": 2
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "shield-consumption-live-recompute",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [
        {
          "host": "marine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "own"
        },
        {
          "host": "marine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "marine",
          "code": "IC27_052",
          "owner": "own"
        },
        {
          "host": "spacer",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "enemyMarine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "enemy"
        },
        {
          "host": "enemyMarine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "TWI_174",
          "target": "marine"
        }
      ],
      "expected": {
        "sourcePower": 10,
        "sourcePowerAfterPlay": 11,
        "marineDamage": 0,
        "marineShieldCount": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "shield-consumption-live-recompute",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [
        {
          "host": "marine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "own"
        },
        {
          "host": "marine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "marine",
          "code": "IC27_052",
          "owner": "own"
        },
        {
          "host": "spacer",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "own"
        },
        {
          "host": "enemyMarine",
          "token": "experience",
          "code": "TOKEN_EXPERIENCE",
          "owner": "enemy"
        },
        {
          "host": "enemyMarine",
          "token": "shield",
          "code": "TOKEN_SHIELD",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "TWI_174",
          "target": "marine"
        }
      ],
      "expected": {
        "sourcePower": 10,
        "sourcePowerAfterPlay": 11,
        "marineDamage": 0,
        "marineShieldCount": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "advantage-attack-expiry",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "LAW_043",
          "target": "marine"
        },
        {
          "code": "SOR_095",
          "from": "marine",
          "target": "enemyBase"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "sourcePowerAfterPlay": 7,
        "sourcePowerBeforeAttack": 8,
        "marineAdvantageCount": 0,
        "marineShieldCount": 1,
        "spacerAdvantageCount": 1,
        "enemyBaseDamage": 4
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "advantage-attack-expiry",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "LAW_043",
          "target": "marine"
        },
        {
          "code": "SOR_095",
          "from": "marine",
          "target": "enemyBase"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "sourcePowerAfterPlay": 7,
        "sourcePowerBeforeAttack": 8,
        "marineAdvantageCount": 0,
        "marineShieldCount": 1,
        "spacerAdvantageCount": 1,
        "enemyBaseDamage": 4
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "ordinary-upgrade-self-exclusion",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "IC27_052",
          "target": "executor"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "sourceHp": 15,
        "sourceUpgradeCount": 1
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "ordinary-upgrade-self-exclusion",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "IC27_052",
          "target": "executor"
        }
      ],
      "expected": {
        "sourcePower": 7,
        "sourceHp": 15,
        "sourceUpgradeCount": 1
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "host-defeat-attachment-cleanup",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "TWI_174",
          "target": "marine"
        }
      ],
      "expected": {
        "sourcePower": 6,
        "marineZone": "discard",
        "marineAttachmentCountAfterCleanup": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "host-defeat-attachment-cleanup",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "TWI_174",
          "target": "marine"
        }
      ],
      "expected": {
        "sourcePower": 6,
        "marineZone": "discard",
        "marineAttachmentCountAfterCleanup": 0
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "unique-keep-old",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 10,
        "survivor": "old",
        "defeated": "incoming",
        "survivorAdvantageCount": 1,
        "marineAdvantageCount": 2,
        "spacerAdvantageCount": 2,
        "totalFocusPlayCost": 24,
        "capturedDeadIncomingWhenPlayed": true
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "unique-keep-old",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 10,
        "survivor": "old",
        "defeated": "incoming",
        "survivorAdvantageCount": 1,
        "marineAdvantageCount": 2,
        "spacerAdvantageCount": 2,
        "totalFocusPlayCost": 24,
        "capturedDeadIncomingWhenPlayed": true
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 0,
    "scenario": "unique-keep-incoming",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 9,
        "survivor": "incoming",
        "defeated": "old",
        "survivorAdvantageCount": 0,
        "marineAdvantageCount": 2,
        "spacerAdvantageCount": 2,
        "totalFocusPlayCost": 24
      }
    }
  },
  {
    "family": "executor-upgrade-count",
    "cardId": "ASH_197",
    "owner": 1,
    "scenario": "unique-keep-incoming",
    "resourceCount": 30,
    "leaderCode": "SOR_014",
    "baseCode": "SOR_023",
    "baseDamage": 0,
    "expectedFocusPlayCost": 12,
    "design": {
      "units": [
        {
          "key": "marine",
          "code": "SOR_095",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "spacer",
          "code": "SOR_178",
          "owner": "own",
          "exhausted": false
        },
        {
          "key": "enemyMarine",
          "code": "SOR_095",
          "owner": "enemy",
          "exhausted": false
        }
      ],
      "initialAttachments": [],
      "plan": [
        {
          "code": "ASH_197"
        },
        {
          "code": "ASH_197"
        }
      ],
      "expected": {
        "sourcePower": 9,
        "survivor": "incoming",
        "defeated": "old",
        "survivorAdvantageCount": 0,
        "marineAdvantageCount": 2,
        "spacerAdvantageCount": 2,
        "totalFocusPlayCost": 24
      }
    }
  }
];
const copy=value=>JSON.parse(JSON.stringify(value));
const stable=value=>Array.isArray(value)?value.map(stable):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,stable(value[key])])):value;
const semanticHash=value=>createHash('sha256').update(JSON.stringify(stable(value))).digest('hex');
function classifyExecutorUpgradeCount(entry){return same(entry,reviewedSource)?{family:'executor-upgrade-count',arena:'space',unique:true}:null;}
function planExecutorUpgradeCountProbes(entry,options={}){
 if(!classifyExecutorUpgradeCount(entry))return null;
 const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
 const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[],needs:reviewedNeeds.map(e=>({cardId:e.setCode,wholeEntrySemanticHash:semanticHash(e),sourcePath:e.sourcePath??null,sourceHash:e.sourceHash??null,native:e.card.types.includes('token')}))};
 const obligation=(dimension,reason)=>{const id=entry.setCode+':executor-upgrade-count:'+dimension;result.obligations.push({id,path:'executor-upgrade-count',dimension});if(reason)result.uncovered.push({id,path:'executor-upgrade-count',reason});return id;};
 const missing=reviewedNeeds.filter(e=>!catalogue.some(c=>same(c,e)));
 for(const need of missing)obligation('dependency-'+need.setCode,'Complete reviewed donor/native definition missing or changed: '+need.setCode);
 if(missing.length)return result;
 for(const original of rawDesigns){const probe=copy(original),name=probe.scenario+'-seat'+probe.owner,id=obligation(name);const setup=[probe.leaderCode,probe.baseCode,'TOKEN_ADVANTAGE',...probe.design.units.map(u=>u.code),...probe.design.initialAttachments.map(a=>a.code),...probe.design.plan.filter(p=>!p.from&&p.code!==entry.setCode).map(p=>p.code)];if(probe.scenario==='advantage-attack-expiry')setup.push('TOKEN_SHIELD');result.cases.push({id:'auto/'+entry.setCode+'/executor-upgrade-count-'+name,name:'executor-upgrade-count-'+name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds:[...new Set(setup)].sort(),behaviorIds:['live-physical-upgrade-count','complete-when-played-advantage-batch','physical-source-exclusion'],assertionStages:['actual-focus-play-normal-cost','resolved-batch-and-attachment-identity','live-power-after-attachment-mutation','terminal-continuation'],coverageObligationIds:[id],probe});}
 for(const [dimension,reason] of [['source-review','Exact full source/helper lowering and independent review remain required; raw source games are not native/Wasm conformance.'],['capacity-and-coinciding-deaths','No capacity overflow or simultaneous multi-host/source lethal-set witness; reject unsupported capacity rather than truncate.'],['controller-transfer','No actual legal controller-transfer gameplay. Foreign-owner attachment diagnostics are deliberately excluded from these standard cases.'],['leader-and-native-unit-hosts','Complete source includes deployed leaders/native unit hosts; no corresponding source-game construction is included here.'],['replacement-blanking-and-late-entry','Arbitrary token replacement, source blanking and entry during replacement window remain uncovered.']])obligation(dimension,reason);
 return result;
}
module.exports={classifyExecutorUpgradeCount,planExecutorUpgradeCountProbes};
