'use strict';
const same=require('node:util').isDeepStrictEqual,crypto=require('node:crypto');
function canonical(v){if(v===null)return'null';if(Array.isArray(v))return'['+v.map(canonical).join(',')+']';if(typeof v==='object'){if(![Object.prototype,null].includes(Object.getPrototypeOf(v))||Reflect.ownKeys(v).some(k=>typeof k!=='string'))throw new Error('Non-JSON object');return'{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}';}if(typeof v==='number'){if(!Number.isFinite(v))throw new Error('Non-JSON number');return Object.is(v,-0)?'-0':String(v);}if(typeof v==='string'||typeof v==='boolean')return JSON.stringify(v);throw new Error('Non-JSON value');}
function fingerprint(v){try{return crypto.createHash('sha256').update(canonical(v)).digest('hex');}catch{return null;}}
function profile(entry){const e=structuredClone(entry);for(const k of ['setCode','id','internalName','className'])delete e[k];if(e.card)for(const k of ['id','internalName','setId','setCodes'])delete e.card[k];return e;}
const reviewedProfiles=[
  {
    "setCode": "ASH_047",
    "sourcePath": "game/cards/08_ASH/units/GarSaxonCovetingPower.ts",
    "sourceHash": "6032937d56a8afdcc23ef689beb5401d85efd22460b2f2defef91089a17992c5",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Create a Mandalorian token",
          "costs": [],
          "optional": true,
          "limit": {
            "kind": "abilityLimit",
            "factory": "perRound",
            "args": [
              1
            ],
            "scope": "registration",
            "sharedAcrossHooks": true
          },
          "effect": {
            "kind": "createTokenUnit",
            "tokenType": "mandalorian",
            "amount": {
              "kind": "const",
              "value": 1
            },
            "target": "ownPlayer",
            "entersReady": false
          },
          "hooks": [
            {
              "event": "onCardPlayed",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:738:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "player",
                          "optional": false
                        }
                      ]
                    },
                    "right": {
                      "kind": "ref",
                      "path": [
                        "player"
                      ]
                    }
                  },
                  "right": {
                    "kind": "dataCall",
                    "method": "isUpgrade",
                    "receiver": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:738:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "card",
                          "optional": false
                        }
                      ]
                    },
                    "args": [],
                    "optionalCall": false,
                    "optionalReceiver": false
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "===",
                  "left": {
                    "kind": "dataPath",
                    "root": [
                      "event"
                    ],
                    "argumentBinding": {
                      "id": "callback:738:0",
                      "bindAt": "callbackInvocation",
                      "readAt": "expressionEvaluation"
                    },
                    "steps": [
                      {
                        "key": "attachTarget",
                        "optional": false
                      }
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
        }
      ]
    },
    "className": "GarSaxonCovetingPower",
    "baseClass": "NonLeaderUnitCard",
    "id": "6754905608",
    "internalName": "gar-saxon#coveting-power",
    "status": "supported",
    "card": {
      "title": "Gar Saxon",
      "subtitle": "Coveting Power",
      "cost": 3,
      "hp": 4,
      "power": 3,
      "text": "When you play an upgrade on this unit: You may create a Mandalorian token. Use this ability only once each round.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": "Attaching a token upgrade is not considered playing an upgrade and does not trigger Gar Saxon's ability.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6754905608",
      "aspects": [
        "vigilance",
        "villainy"
      ],
      "traits": [
        "mandalorian",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 47
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 47
        }
      ],
      "internalName": "gar-saxon#coveting-power"
    },
    "dataHash": "722f5a6e8ee1463257b972a5f9b8701e310faa637a45b3f7075393c896470560",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:&&",
      "binary:===",
      "cardType:unit",
      "ir:abilityLimit",
      "ir:binary",
      "ir:const",
      "ir:createTokenUnit",
      "ir:dataCall",
      "ir:dataPath",
      "ir:ref",
      "ref:player",
      "ref:source",
      "trigger:eventHooks"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_032",
    "sourcePath": "game/cards/08_ASH/units/RancorKeeper.ts",
    "sourceHash": "6f8746448d1845b9f91f796cbc9531e6f966957ac71264b0859861bd7b7a5b72",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Deal 1 damage to any number of bases",
          "costs": [],
          "limit": {
            "kind": "abilityLimit",
            "factory": "perRound",
            "args": [
              1
            ],
            "scope": "registration",
            "sharedAcrossHooks": true
          },
          "targetResolver": {
            "owner": "any",
            "type": "base",
            "zone": "anyAttackable",
            "mode": "unlimited",
            "optional": true
          },
          "effect": {
            "kind": "damage",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
          "hooks": [
            {
              "event": "onDamageDealt",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "unary",
                    "op": "!",
                    "value": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:787:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "willDefeat",
                          "optional": false
                        }
                      ]
                    }
                  },
                  "right": {
                    "kind": "dataCall",
                    "method": "isUnit",
                    "receiver": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:787:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "card",
                          "optional": false
                        }
                      ]
                    },
                    "args": [],
                    "optionalCall": false,
                    "optionalReceiver": false
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "===",
                  "left": {
                    "kind": "dataPath",
                    "root": [
                      "event"
                    ],
                    "argumentBinding": {
                      "id": "callback:787:0",
                      "bindAt": "callbackInvocation",
                      "readAt": "expressionEvaluation"
                    },
                    "steps": [
                      {
                        "key": "card",
                        "optional": false
                      },
                      {
                        "key": "controller",
                        "optional": false
                      }
                    ]
                  },
                  "right": {
                    "kind": "ref",
                    "path": [
                      "player"
                    ]
                  }
                }
              }
            }
          ]
        }
      ]
    },
    "className": "RancorKeeper",
    "baseClass": "NonLeaderUnitCard",
    "id": "1069133190",
    "internalName": "rancor-keeper",
    "status": "supported",
    "card": {
      "title": "Rancor Keeper",
      "subtitle": "",
      "cost": 2,
      "hp": 4,
      "power": 2,
      "text": "When a friendly unit is dealt damage and survives: Deal 1 damage to any number of bases. Use this ability only once each round.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "All damage dealt by Rancor Keeper's ability is dealt simultaneously.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1069133190",
      "aspects": [
        "vigilance",
        "aggression"
      ],
      "traits": [
        "underworld"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 32
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 32
        }
      ],
      "internalName": "rancor-keeper"
    },
    "dataHash": "8fad81fa1e86830b0b480e7dc418a68ef26704555369ed8c152f62809cef837e",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:&&",
      "binary:===",
      "cardType:unit",
      "ir:abilityLimit",
      "ir:binary",
      "ir:const",
      "ir:damage",
      "ir:dataCall",
      "ir:dataPath",
      "ir:ref",
      "ir:unary",
      "ref:player",
      "targetMode:unlimited",
      "targetType:base",
      "targetZone:anyAttackable",
      "trigger:eventHooks",
      "unary:!"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_160",
    "sourcePath": "game/cards/08_ASH/units/KachirhoMilitia.ts",
    "sourceHash": "4eb45048fce01d0b58d176e200af035b4d60525e105e28df84a3fd0443e25e17",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Ready Kachirho Militia",
          "costs": [],
          "limit": {
            "kind": "abilityLimit",
            "factory": "perRound",
            "args": [
              1
            ],
            "scope": "registration",
            "sharedAcrossHooks": true
          },
          "effect": {
            "kind": "ready",
            "target": "self"
          },
          "hooks": [
            {
              "event": "onAttackDeclared",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "binary",
                    "op": "!==",
                    "left": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:821:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "attackerLastKnownInformation",
                          "optional": false
                        },
                        {
                          "key": "controller",
                          "optional": false
                        }
                      ]
                    },
                    "right": {
                      "kind": "ref",
                      "path": [
                        "player"
                      ]
                    }
                  },
                  "right": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:821:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "attackerLastKnownInformation",
                          "optional": false
                        },
                        {
                          "key": "arena",
                          "optional": false
                        }
                      ]
                    },
                    "right": {
                      "kind": "const",
                      "value": "groundArena"
                    }
                  }
                },
                "right": {
                  "kind": "dataCall",
                  "method": "some",
                  "receiver": {
                    "kind": "dataCall",
                    "method": "getAllTargets",
                    "receiver": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:821:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "attack",
                          "optional": false
                        }
                      ]
                    },
                    "args": [],
                    "optionalCall": false,
                    "optionalReceiver": false
                  },
                  "args": [
                    {
                      "kind": "dataLambda",
                      "bindings": [
                        "data:1072:0"
                      ],
                      "body": {
                        "kind": "dataCall",
                        "method": "isBase",
                        "receiver": {
                          "kind": "dataPath",
                          "root": [
                            "data:1072:0"
                          ],
                          "steps": []
                        },
                        "args": [],
                        "optionalCall": false,
                        "optionalReceiver": false
                      }
                    }
                  ],
                  "optionalCall": false,
                  "optionalReceiver": false
                }
              }
            }
          ]
        }
      ]
    },
    "className": "KachirhoMilitia",
    "baseClass": "NonLeaderUnitCard",
    "id": "6989468664",
    "internalName": "kachirho-militia",
    "status": "supported",
    "card": {
      "title": "Kachirho Militia",
      "subtitle": "",
      "cost": 5,
      "hp": 6,
      "power": 4,
      "text": "Hidden (This unit can't be attacked if it was played this phase.)\nWhen an enemy ground unit attacks your base: Ready this unit. Use this ability only once each round.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "If a unit has Hidden and Sentinel at the same time, it can be attacked.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6989468664",
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "wookiee",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [
        "hidden"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 160
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 160
        }
      ],
      "internalName": "kachirho-militia"
    },
    "dataHash": "a53aef6ebc387be4935427a7cc531efa3b86d19d4dfbf6d54f19c3eb108c39f2",
    "keywordValues": {
      "hidden": 1
    },
    "requiredCapabilities": [
      "binary:!==",
      "binary:&&",
      "binary:===",
      "cardType:unit",
      "ir:abilityLimit",
      "ir:binary",
      "ir:const",
      "ir:dataCall",
      "ir:dataLambda",
      "ir:dataPath",
      "ir:ready",
      "ir:ref",
      "keyword:hidden",
      "ref:player",
      "trigger:eventHooks"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_062",
    "sourcePath": "game/cards/09_HMW/units/NuvoVindiBlueShadowPerfected.ts",
    "sourceHash": "33a11f826279ca2c6e8e1e3a5ead0fa3753d0548ffc9cbf54d7018c6a0d31af3",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Give a Weakness token to a unit",
          "costs": [],
          "optional": true,
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "giveWeakness",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 1
            }
          }
        },
        {
          "trigger": "eventHooks",
          "title": "Give a Weakness token to a unit",
          "costs": [],
          "optional": true,
          "limit": {
            "kind": "abilityLimit",
            "factory": "perRound",
            "args": [
              1
            ],
            "scope": "registration",
            "sharedAcrossHooks": true
          },
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "giveWeakness",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
          "hooks": [
            {
              "event": "onCardDefeated",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "enumTypeTest",
                    "test": "isUnit",
                    "args": [
                      {
                        "kind": "dataPath",
                        "root": [
                          "event"
                        ],
                        "argumentBinding": {
                          "id": "callback:1271:0",
                          "bindAt": "callbackInvocation",
                          "readAt": "expressionEvaluation"
                        },
                        "steps": [
                          {
                            "key": "lastKnownInformation",
                            "optional": false
                          },
                          {
                            "key": "type",
                            "optional": false
                          }
                        ]
                      }
                    ]
                  },
                  "right": {
                    "kind": "binary",
                    "op": "!==",
                    "left": {
                      "kind": "dataPath",
                      "root": [
                        "event"
                      ],
                      "argumentBinding": {
                        "id": "callback:1271:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "lastKnownInformation",
                          "optional": false
                        },
                        {
                          "key": "controller",
                          "optional": false
                        }
                      ]
                    },
                    "right": {
                      "kind": "ref",
                      "path": [
                        "player"
                      ]
                    }
                  }
                },
                "right": {
                  "kind": "dataCall",
                  "method": "some",
                  "receiver": {
                    "kind": "dataPath",
                    "root": [
                      "event"
                    ],
                    "argumentBinding": {
                      "id": "callback:1271:0",
                      "bindAt": "callbackInvocation",
                      "readAt": "expressionEvaluation"
                    },
                    "steps": [
                      {
                        "key": "lastKnownInformation",
                        "optional": false
                      },
                      {
                        "key": "upgrades",
                        "optional": false
                      }
                    ]
                  },
                  "args": [
                    {
                      "kind": "dataLambda",
                      "bindings": [
                        "data:1508:0"
                      ],
                      "body": {
                        "kind": "dataCall",
                        "method": "isWeakness",
                        "receiver": {
                          "kind": "dataPath",
                          "root": [
                            "data:1508:0"
                          ],
                          "steps": []
                        },
                        "args": [],
                        "optionalCall": false,
                        "optionalReceiver": false
                      }
                    }
                  ],
                  "optionalCall": false,
                  "optionalReceiver": true
                }
              }
            }
          ]
        }
      ]
    },
    "className": "NuvoVindiBlueShadowPerfected",
    "baseClass": "NonLeaderUnitCard",
    "id": "9689400738",
    "internalName": "nuvo-vindi#blue-shadow-perfected",
    "status": "supported",
    "card": {
      "title": "Nuvo Vindi",
      "subtitle": "Blue Shadow Perfected",
      "cost": 3,
      "hp": 4,
      "power": 1,
      "text": "When Played: You may give a Weakness token to a unit.\nWhen an enemy unit with a Weakness token on it is defeated: You may give a Weakness token to a unit. Use this ability only once each round.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9689400738",
      "aspects": [
        "vigilance",
        "villainy"
      ],
      "traits": [
        "separatist"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 62
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 62
        }
      ],
      "internalName": "nuvo-vindi#blue-shadow-perfected"
    },
    "dataHash": "71c2aa6d67e2f4117e75f957ea2c1c33b32ea802ef2ef89d52a2a04c3800cd19",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:!==",
      "binary:&&",
      "cardType:unit",
      "ir:abilityLimit",
      "ir:binary",
      "ir:const",
      "ir:dataCall",
      "ir:dataLambda",
      "ir:dataPath",
      "ir:enumTypeTest",
      "ir:giveWeakness",
      "ir:ref",
      "ref:player",
      "targetType:unit",
      "targetZone:anyAttackable",
      "trigger:eventHooks",
      "trigger:whenPlayed"
    ],
    "diagnostics": []
  },
  {
    "setCode": "SOR_115",
    "sourcePath": "game/cards/01_SOR/units/AgentKallusSeekingTheRebels.ts",
    "sourceHash": "9c20159591599b89b2f78b68c076ee9bb0321e1efb22dc631af847392318155b",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Draw a card",
          "costs": [],
          "optional": true,
          "limit": {
            "kind": "abilityLimit",
            "factory": "perRound",
            "args": [
              1
            ],
            "scope": "registration",
            "sharedAcrossHooks": true
          },
          "effect": {
            "kind": "draw",
            "target": "ownPlayer",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
          "hooks": [
            {
              "event": "onCardDefeated",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "enumTypeTest",
                    "test": "isUnit",
                    "args": [
                      {
                        "kind": "dataPath",
                        "root": [
                          "event"
                        ],
                        "argumentBinding": {
                          "id": "callback:826:0",
                          "bindAt": "callbackInvocation",
                          "readAt": "expressionEvaluation"
                        },
                        "steps": [
                          {
                            "key": "lastKnownInformation",
                            "optional": false
                          },
                          {
                            "key": "type",
                            "optional": false
                          }
                        ]
                      }
                    ]
                  },
                  "right": {
                    "kind": "dataPath",
                    "root": [
                      "event"
                    ],
                    "argumentBinding": {
                      "id": "callback:826:0",
                      "bindAt": "callbackInvocation",
                      "readAt": "expressionEvaluation"
                    },
                    "steps": [
                      {
                        "key": "card",
                        "optional": false
                      },
                      {
                        "key": "unique",
                        "optional": false
                      }
                    ]
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "!==",
                  "left": {
                    "kind": "dataPath",
                    "root": [
                      "event"
                    ],
                    "argumentBinding": {
                      "id": "callback:826:0",
                      "bindAt": "callbackInvocation",
                      "readAt": "expressionEvaluation"
                    },
                    "steps": [
                      {
                        "key": "card",
                        "optional": false
                      }
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
        }
      ]
    },
    "className": "AgentKallusSeekingTheRebels",
    "baseClass": "NonLeaderUnitCard",
    "id": "2649829005",
    "internalName": "agent-kallus#seeking-the-rebels",
    "status": "supported",
    "card": {
      "title": "Agent Kallus",
      "subtitle": "Seeking the Rebels",
      "cost": 5,
      "hp": 4,
      "power": 4,
      "text": "Ambush (After you play this unit, he may ready and attack an enemy unit.)\n\nWhen another unique unit is defeated: You may draw a card. Use this ability only once each round.",
      "deployBox": null,
      "epicAction": null,
      "unique": true,
      "rules": "If Agent Kallus is defeated simultaneously with other unique units, you still may draw a card.\nAmbush is an ability that triggers at the same time as “When Played” abilities. If there is no enemy unit that can be attacked, the unit does not ready.\n(ERRATA) Templating update: “Unique” becomes “<uq>”.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2649829005",
      "aspects": [
        "command"
      ],
      "traits": [
        "imperial",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [
        "ambush"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SOR",
        "number": 115
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 115
        }
      ],
      "internalName": "agent-kallus#seeking-the-rebels"
    },
    "dataHash": "0785ddee91ae02f4267f12f19ba6d69cf7a229887d4066f1f51d41801f71600d",
    "keywordValues": {
      "ambush": 1
    },
    "requiredCapabilities": [
      "binary:!==",
      "binary:&&",
      "cardType:unit",
      "ir:abilityLimit",
      "ir:binary",
      "ir:const",
      "ir:dataPath",
      "ir:draw",
      "ir:enumTypeTest",
      "ir:ref",
      "keyword:ambush",
      "ref:source",
      "trigger:eventHooks"
    ],
    "diagnostics": []
  }
];
const modes=["played-upgrade","nonlethal-damage","enemy-ground-attack","enemy-weakness-defeat","other-unique-defeat"];
const reviewedDonors={
  "ASH_061": "f6a033f123707b8cd7f0431599b43617fa2e088092586b0fc67343ca8d482c9b",
  "ASH_131": "1fcfd58c506b4e94f313d01deacffd6cf2fbc4c2bb180bd7bb1d46d296961c2d",
  "ASH_190": "5d22cbe617157548f41e5cc0d44365242c093ed7a66e8135cd52e106e41c0b93",
  "ASH_201": "173783137608a82a1eae6b38accd237d6484517f350c4fe9f399ec4dd08a7c08",
  "ASH_242": "d0a10a6e6307b625451b06765d5f5b8304f68d9e3e2cc3edd510c564eee4abd5",
  "ASH_261": "e62a861a7625bd2c9f6222abcf1ca31168d7a493d2bf5af30188a863a5325ab2",
  "HMW_082": "d4b667f1930c70b6bc1eac9ec62dd96121b44c3baba5ef9d0e9a32fb0a683e81",
  "HMW_093": "5c1f7ec438f2e2ad83ea97ab207ad8bdf99f80be4d2dc04e4734925f2b8477ee",
  "HMW_116": "b16e84dce80f70da340f5c2c87580793ce7367f8b516f24ff63680a2ec281e69",
  "HMW_128": "ce5b5159056a22ff31c7ec06d689caede18bc64edacef70b39e72efff0d73027",
  "HMW_135": "e74522cc77ce7cbf0d454a4948ff5b12684f882268757a157a63761a4bbbdc83",
  "HMW_155": "9db7facb8ee661bc3c4181c8c07d9f1629e2b1ea0f7371c63e68b66522002c19",
  "HMW_163": "9021b8be120211bfeb9d2b7a357fe033bd16642d59602c7a18093acad5a0b48a",
  "HMW_174": "68d86d06abc3e805c3ac3c424aa6ddeb2258d405ffb99b651323680584894d19",
  "HMW_178": "c59421061ea586b6387d6e587b7af8001adbbb3dd1eff18a13c2e958e64dc87c",
  "HMW_219": "8eb4414f5a1f69fe1013ee0b4bcb644aacec9e84165ccf0a863b105ff35abc96",
  "HMW_258": "bb9a2eddf056f27a18763ccc6aeeabcbbd4e3d97287249fe6ca00d307bec03a8",
  "SOR_014": "d1ca8396901fac3eb4eafafb55439deeadf8b385f6e195faa781a786568c2bac",
  "SOR_023": "a694a128d3fb4bb0a4e465dcf906cb2afd3a936c7d99ee1f8cf53db1a7cb41b6",
  "SOR_095": "27b538f7ef020583c610281275a75f2187beb15ef5946d59a18809d53d686d3d",
  "SOR_178": "d70e91da87c650d7c3ccc91ad3b6c67f36a04ece2570dd224764053daf5f22e4",
  "IC27_052": "fec00f15679fe14298b2d95d5ec70c80eee187c77e2cd9d590a4eb62b324fb12",
  "IC27_184": "f8a03546457954045b8c2bda94c19c69dce3ad292a092e689775627d8d132989",
  "ASH_151": "f624d88b12117c24756f2412e788ec4896bc7080537dd60d2a205393d5d09dd5",
  "SHD_037": "972f1fb4d3c3643b2194496bb04dd02d774297d50bd6ea1a63c2b56203ed5299",
  "LOF_164": "f7dfd7fab22e905db0e655e715054e1ebfe9d8d940d79b399edb2b42b50280c5",
  "TWI_174": "0d02abd4462778f21cd5759d9135763f5c2e2bb53677e7b04529677cb4d85266",
  "TWI_077": "e12af5884eb2f14a4d4dc085dae2bbd69814afa0b43d62278f9fe79008606a5a",
  "TWI_226": "abca6b40e5fe7d36f6c4caa12ae501cc3ccfabaa26970aeee550fff3c727d80e",
  "SOR_043": "3371eb7e1ff47519f2ac864b40aacd506997056edb8310f92457963cb35077a0",
  "ASH_036": "6b0f5aea837d2ebddeb65c9a201552a6f35ccc3fb94f24ce9003d275e0948e0f",
  "TOKEN_MANDALORIAN": "228b031938f4233d811ade2b8d27dbcedb892be0bc2e016f26dcaa8d7ed2be10",
  "TOKEN_SHIELD": "a8a34f754731a5dc9de991d5476b9491466190ac7521e0c7192c9210fc5538a9",
  "TOKEN_WEAKNESS": "e2044669df19b9534392e1b1ccfe31a1fa3cdccedd63c39bfdecd50594e0b823",
  "TOKEN_ADVANTAGE": "be7663af45ebe560c97cbe5b1305863b4dcdf6bd4d594ef333643b3fd1fb7234",
  "TOKEN_EXPERIENCE": "275c2c34e4ea190b9c681ea857f3f8bb13877e2373f6309cea717b3cf4eeb9f8",
  "ASH_032": "6c9a711a873901facaccbb1ce9175f407342d72e6bdc0f710f0d144e53ce7256",
  "ASH_072": "a71165479c1985a08c586c58781a60c8104c98cb10b56db4fb574893f7a8a0ec",
  "ASH_153": "bed1f320ac9550ee8a32f344a0d139ba75b37ab44d2dce3616867b029eda122a",
  "ASH_160": "854d4aa77bf934afdb11ec9efbe91caab433f6fde53f73968751715b2c21142b",
  "HMW_062": "2159d6b14eb5e2a616d2d1950c381012e81c1ac83c14d0e59fcbb7ae32d485e5",
  "IBH_063": "a0f23b97d6e48d728cd6cfa9e8470d3c6a8e6b42a74e8eb529c5dcd84a48ecee",
  "SOR_115": "3ebadf5e0d479ad418928c5987524c637390c951d4a3de7549a4971df2057b3f",
  "SOR_135": "3ab7a10134b4b54cd5544388b72e8449671475ba13d9fc39ae3c6d10466057bb",
  "IBH_007": "0e2363941d4598e3568215401fe98569fdae953332cb715d3dd107866a424314"
};
const fillerCodes=["ASH_061","ASH_131","ASH_190","ASH_201","ASH_242","ASH_261","HMW_082","HMW_093","HMW_116","HMW_128","HMW_135","HMW_155","HMW_163","HMW_174","HMW_178","HMW_219","HMW_258"];
function classifyRoundLimit(entry){if(!entry||typeof entry!=='object'||Array.isArray(entry)||typeof entry.setCode!=='string')return null;const i=reviewedProfiles.findIndex(p=>same(profile(p),profile(entry)));return i<0?null:{family:'round-limit',mode:modes[i]};}
// Only source-card constructions confirmed by the oracle author belong here.
// An empty family is deliberately uncovered, not a speculative standard case.
const constructions={
  "played-upgrade": [
    {
      "scenario": "paid-first-second",
      "donors": [
        "IC27_052",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "decline-then-use",
      "donors": [
        "IC27_052",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "round-reset",
      "donors": [
        "IC27_052",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "reentry",
      "donors": [
        "IC27_052",
        "TWI_226",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "token-other-host-negative",
      "donors": [
        "SOR_095",
        "IC27_184",
        "IC27_052",
        "TOKEN_EXPERIENCE",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "enemy-play-negative",
      "donors": [
        "IC27_052",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "opposing-copies",
      "donors": [
        "IC27_052",
        "TOKEN_MANDALORIAN",
        "TOKEN_SHIELD"
      ]
    }
  ],
  "nonlethal-damage": [
    {
      "scenario": "bases-both",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    },
    {
      "scenario": "bases-own",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    },
    {
      "scenario": "bases-enemy",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    },
    {
      "scenario": "bases-none",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    },
    {
      "scenario": "lethal-negative",
      "donors": [
        "SOR_095",
        "TWI_174"
      ]
    },
    {
      "scenario": "shield-then-survive",
      "donors": [
        "LOF_164",
        "TWI_174",
        "TOKEN_SHIELD"
      ]
    },
    {
      "scenario": "enemy-then-friendly",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    },
    {
      "scenario": "multiple-copies-round-reset",
      "donors": [
        "LOF_164",
        "TWI_174"
      ]
    }
  ],
  "enemy-ground-attack": [
    {
      "scenario": "paid-hidden-first-second-round-reset",
      "donors": [
        "SOR_095"
      ]
    },
    {
      "scenario": "predicate-negatives",
      "donors": [
        "SOR_095",
        "SOR_178",
        "LOF_164"
      ]
    },
    {
      "scenario": "already-ready-no-consume",
      "donors": [
        "SOR_095"
      ]
    },
    {
      "scenario": "multiple-copies",
      "donors": [
        "SOR_095"
      ]
    }
  ],
  "enemy-weakness-defeat": [
    {
      "scenario": "paid-wp-first-second-death",
      "donors": [
        "LOF_164",
        "SOR_095",
        "TWI_077",
        "TOKEN_WEAKNESS"
      ]
    },
    {
      "scenario": "decline-then-use",
      "donors": [
        "LOF_164",
        "SOR_095",
        "TWI_077",
        "TOKEN_WEAKNESS"
      ]
    },
    {
      "scenario": "round-reset",
      "donors": [
        "LOF_164",
        "SOR_095",
        "TWI_077",
        "TOKEN_WEAKNESS"
      ]
    },
    {
      "scenario": "predicate-negatives",
      "donors": [
        "LOF_164",
        "SOR_095",
        "TWI_077",
        "TOKEN_WEAKNESS"
      ]
    },
    {
      "scenario": "captured-source-cinder-death",
      "donors": [
        "LOF_164",
        "ASH_131",
        "ASH_151",
        "TOKEN_WEAKNESS"
      ]
    }
  ],
  "other-unique-defeat": [
    {
      "scenario": "paid-ambush-first-second",
      "donors": [
        "ASH_036",
        "TWI_077"
      ]
    },
    {
      "scenario": "draw-decline-then-use",
      "donors": [
        "ASH_036",
        "TWI_077"
      ]
    },
    {
      "scenario": "round-reset",
      "donors": [
        "ASH_036",
        "SHD_037",
        "TWI_077"
      ]
    },
    {
      "scenario": "nonunique-then-captured-mass-death",
      "donors": [
        "ASH_036",
        "SOR_095",
        "TWI_077",
        "SOR_043"
      ]
    },
    {
      "scenario": "ambush-no-target",
      "donors": []
    },
    {
      "scenario": "ambush-decline",
      "donors": [
        "SOR_095"
      ]
    },
    {
      "scenario": "self-death-negative",
      "donors": [
        "TWI_077"
      ]
    }
  ]
};
const interactionConstructions=[
  {
    "name": "round-limit-SOR_115-owner-0-simultaneous-unique-next",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-next",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-simultaneous-unique-next",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-next",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-simultaneous-unique-all",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-all",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-simultaneous-unique-all",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-all",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-simultaneous-unique-accept-first-next",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-accept-first-next",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-simultaneous-unique-accept-first-next",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-accept-first-next",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-simultaneous-unique-accept-first-all",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-accept-first-all",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-simultaneous-unique-accept-first-all",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "simultaneous-unique-accept-first-all",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "multiplicity"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-0-deployed-leader-nonlethal",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 0,
      "mode": "nonlethal-damage",
      "scenario": "deployed-leader-nonlethal",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-1-deployed-leader-nonlethal",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 1,
      "mode": "nonlethal-damage",
      "scenario": "deployed-leader-nonlethal",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-HMW_062-owner-0-deployed-leader-weakness-defeat",
    "probe": {
      "family": "round-limit",
      "cardId": "HMW_062",
      "owner": 0,
      "mode": "enemy-weakness-defeat",
      "scenario": "deployed-leader-weakness-defeat",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-HMW_062-owner-1-deployed-leader-weakness-defeat",
    "probe": {
      "family": "round-limit",
      "cardId": "HMW_062",
      "owner": 1,
      "mode": "enemy-weakness-defeat",
      "scenario": "deployed-leader-weakness-defeat",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-deployed-leader-unique-defeat",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "deployed-leader-unique-defeat",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-deployed-leader-unique-defeat",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "deployed-leader-unique-defeat",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "leader-unit"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_062",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_095",
      "SOR_115",
      "TOKEN_WEAKNESS",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-ordinary-registered-first",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "ordinary-registered-first",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "mixed-death-order"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_153",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_095",
      "SOR_115",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-ordinary-registered-first",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "ordinary-registered-first",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "mixed-death-order"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_153",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_095",
      "SOR_115",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-0-limited-registered-first",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 0,
      "mode": "other-unique-defeat",
      "scenario": "limited-registered-first",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "mixed-death-order"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_153",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_095",
      "SOR_115",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-SOR_115-owner-1-limited-registered-first",
    "probe": {
      "family": "round-limit",
      "cardId": "SOR_115",
      "owner": 1,
      "mode": "other-unique-defeat",
      "scenario": "limited-registered-first",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "mixed-death-order"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_153",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_095",
      "SOR_115",
      "TWI_174"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-0-zero-attacker-no-resolved-event",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 0,
      "mode": "nonlethal-damage",
      "scenario": "zero-attacker-no-resolved-event",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "zero-combat"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_063",
      "SHD_037",
      "SOR_014",
      "SOR_023",
      "SOR_095"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-1-zero-attacker-no-resolved-event",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 1,
      "mode": "nonlethal-damage",
      "scenario": "zero-attacker-no-resolved-event",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "zero-combat"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_063",
      "SHD_037",
      "SOR_014",
      "SOR_023",
      "SOR_095"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-0-zero-defender-no-event",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 0,
      "mode": "nonlethal-damage",
      "scenario": "zero-defender-no-event",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "zero-combat"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_063",
      "SHD_037",
      "SOR_014",
      "SOR_023",
      "SOR_095"
    ]
  },
  {
    "name": "round-limit-ASH_032-owner-1-zero-defender-no-event",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_032",
      "owner": 1,
      "mode": "nonlethal-damage",
      "scenario": "zero-defender-no-event",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "zero-combat"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_063",
      "SHD_037",
      "SOR_014",
      "SOR_023",
      "SOR_095"
    ]
  },
  {
    "name": "round-limit-ASH_160-owner-0-already-ready-shared-declaration",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_160",
      "owner": 0,
      "mode": "enemy-ground-attack",
      "scenario": "already-ready-shared-declaration",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "shared-declaration"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_072",
      "ASH_131",
      "ASH_160",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023"
    ]
  },
  {
    "name": "round-limit-ASH_160-owner-1-already-ready-shared-declaration",
    "probe": {
      "family": "round-limit",
      "cardId": "ASH_160",
      "owner": 1,
      "mode": "enemy-ground-attack",
      "scenario": "already-ready-shared-declaration",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "shared-declaration"
    },
    "setupCardIds": [
      "ASH_061",
      "ASH_072",
      "ASH_131",
      "ASH_160",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-0-distributed-six-keeper-order",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 0,
      "scenario": "distributed-six-keeper-order",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "distributed-damage-order"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-1-distributed-six-keeper-order",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 1,
      "scenario": "distributed-six-keeper-order",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "distributed-damage-order"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-0-keeper-three-siblings-remaining-next",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 0,
      "scenario": "keeper-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "next"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_007",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-0-keeper-three-siblings-remaining-all",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 0,
      "scenario": "keeper-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "all"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_007",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-1-keeper-three-siblings-remaining-next",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 1,
      "scenario": "keeper-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "next"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_007",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-ash-032-owner-1-keeper-three-siblings-remaining-all",
    "probe": {
      "family": "round-limit",
      "mode": "nonlethal-damage",
      "cardId": "ASH_032",
      "owner": 1,
      "scenario": "keeper-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "all"
    },
    "setupCardIds": [
      "ASH_032",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "IBH_007",
      "SOR_014",
      "SOR_023",
      "SOR_135"
    ]
  },
  {
    "name": "round-limit-sor-115-owner-0-kallus-three-siblings-remaining-next",
    "probe": {
      "family": "round-limit",
      "mode": "other-unique-defeat",
      "cardId": "SOR_115",
      "owner": 0,
      "scenario": "kallus-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "next"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-sor-115-owner-0-kallus-three-siblings-remaining-all",
    "probe": {
      "family": "round-limit",
      "mode": "other-unique-defeat",
      "cardId": "SOR_115",
      "owner": 0,
      "scenario": "kallus-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "all"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-sor-115-owner-1-kallus-three-siblings-remaining-next",
    "probe": {
      "family": "round-limit",
      "mode": "other-unique-defeat",
      "cardId": "SOR_115",
      "owner": 1,
      "scenario": "kallus-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "next"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  },
  {
    "name": "round-limit-sor-115-owner-1-kallus-three-siblings-remaining-all",
    "probe": {
      "family": "round-limit",
      "mode": "other-unique-defeat",
      "cardId": "SOR_115",
      "owner": 1,
      "scenario": "kallus-three-siblings",
      "resourceCount": 24,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "fillerCodes": [
        "ASH_061",
        "ASH_131",
        "ASH_190",
        "ASH_201",
        "ASH_242",
        "ASH_261",
        "HMW_082",
        "HMW_093",
        "HMW_116",
        "HMW_128",
        "HMW_135",
        "HMW_155",
        "HMW_163",
        "HMW_174",
        "HMW_178",
        "HMW_219",
        "HMW_258"
      ],
      "interaction": "at-limit-siblings",
      "choice": "all"
    },
    "setupCardIds": [
      "ASH_036",
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "HMW_258",
      "SOR_014",
      "SOR_023",
      "SOR_043",
      "SOR_115"
    ]
  }
];
function planRoundLimitProbes(entry,options={}){
    const shape=classifyRoundLimit(entry);if(!shape)return null;
    const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[],result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name,reason){const id=entry.setCode+':round-limit:'+name;result.obligations.push({id,path:'round-limit',dimension:name});if(reason)result.uncovered.push({id,path:'round-limit',reason});return id;}
    function donor(code){const rows=catalogue.filter(e=>e.setCode===code);return rows.length===1&&fingerprint(rows[0])===reviewedDonors[code];}
    obligation('source-review','Complete source/limit/helper and intrinsic lifecycle require independent source-bound review; planner classification is not runtime admission');
    const specs=constructions[shape.mode];
    if(!specs.length)obligation('actual-source-construction','Whole-card actual oracle construction/lifecycle not yet established; no standard cases or coverage credit');
    for(const spec of specs)for(const owner of [0,1]){
        const name=entry.setCode+'-owner-'+owner+'-'+spec.scenario,setupCardIds=[...new Set(['SOR_014','SOR_023',...fillerCodes,...spec.donors])].sort();
        if(!setupCardIds.every(donor)){obligation(name,'Complete admitted donor/native token/default/filler definition missing, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario:spec.scenario,resourceCount:24,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'SOR_023',enemyBaseCode:'SOR_023',fillerCodes:[...fillerCodes]};
        result.cases.push({id:'auto/'+entry.setCode+'/round-limit-'+name,name:'round-limit-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['round-limit',shape.mode],assertionStages:['construction','activation','observation','terminal'],coverageObligationIds:[id],probe});
    }
    // The immutable old62 cases above retain their full object identities.
    // These source protocols require fresh typed-wire recordings; old diagnostic traces are never relabelled.
    for(const spec of interactionConstructions.filter(s=>s.probe.cardId===entry.setCode)){
        const id=obligation('interaction-'+spec.name),setupCardIds=[...spec.setupCardIds];
        if(!setupCardIds.every(donor)){result.uncovered.push({id,path:'round-limit',reason:'Complete interaction source/native/default/filler dependency missing, changed or ambiguous'});continue;}
        result.cases.push({id:'auto/'+entry.setCode+'/'+spec.name,name:spec.name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['round-limit',shape.mode,spec.probe.interaction],assertionStages:['construction','activation','observation','terminal'],coverageObligationIds:[id],probe:structuredClone(spec.probe)});
    }
    obligation('pending-old-registration-after-reentry','No completed source protocol with an unresolved old registration surviving exit/reentry beside the new registration; ordinary reentry alone does not prove captured old-limit identity');
    for(const name of ['controller-change-registration-budget','gained-registration-clone-identity','pending-limit-import','committed-cost-cancellation'])obligation(name,'No completed actual source-card construction for this interaction; registration/player scope and OnRoundEnded reset cannot be replaced by source-phase bits');
    // Automatic reference case suffixes use the existing lowercase/hyphen grammar.
    // Provenance IDs, probe values and all other case fields retain their original identities.
    for(const c of result.cases){
        c.name=c.name.toLowerCase().replaceAll('_','-');
        const slash=c.id.lastIndexOf('/');c.id=c.id.slice(0,slash+1)+c.id.slice(slash+1).toLowerCase().replaceAll('_','-');
    }
    return result;
}
module.exports={classifyRoundLimit,planRoundLimitProbes};
