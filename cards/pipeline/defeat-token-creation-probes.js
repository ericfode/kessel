'use strict';
const same=require('node:util').isDeepStrictEqual;
const reviewedSource={
  "setCode": "ASH_028",
  "sourcePath": "game/cards/08_ASH/units/PazVizslaForABrighterFuture.ts",
  "sourceHash": "abb6d32d10d0a69a179075857626851bcf5ce2a315985c7033c2c9c4c898bd10",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenDefeated",
        "title": "If this unit wasn't defeated by combat damage, create 2 Mandalorian tokens",
        "costs": [],
        "effect": {
          "kind": "conditional",
          "condition": {
            "kind": "binary",
            "op": "!==",
            "left": {
              "kind": "ref",
              "path": [
                "event",
                "defeatSource",
                "type"
              ]
            },
            "right": {
              "kind": "const",
              "value": "attack"
            }
          },
          "onTrue": {
            "kind": "nativeTokenUnitEventFactory",
            "properties": {
              "tokenType": "mandalorian",
              "amount": {
                "kind": "const",
                "value": 2
              },
              "controllers": "ownPlayer",
              "entersReady": false
            },
            "defaults": {
              "enterPlayEffect": {
                "kind": "undefinedValue"
              }
            },
            "inputContext": {
              "kind": "nativeSystemInvocationContext",
              "scope": "currentAbilityInvocation",
              "source": "enclosingAbilitySource",
              "player": "enclosingAbilityPlayer",
              "triggerEvent": "context.event",
              "triggerEventIdentity": "retainOriginal",
              "propertyFactories": "nativeSystemPropertyGeneration"
            },
            "creationEvent": {
              "kind": "freshGameEvent",
              "name": "onTokensCreated",
              "identity": "newPerTargetPlayer",
              "targetIteration": "nativeTargetArrayOrder",
              "context": "inputContext",
              "fields": {
                "player": "capturedTargetPlayer",
                "amount": "generatedProperties.amount",
                "tokenType": "generatedProperties.tokenType",
                "entersReady": "generatedProperties.entersReady",
                "enterPlayEffect": "generatedProperties.enterPlayEffect"
              },
              "fieldBindingStage": "superUpdateEventAddPropertiesToEvent",
              "triggerEventRegister": "unchanged"
            },
            "updateEvent": {
              "after": "superUpdateEvent",
              "regenerateProperties": true,
              "generatedTokens": {
                "initialValue": [],
                "allocationStage": "eventUpdateBeforeResolution",
                "loop": {
                  "indexInitial": 0,
                  "condition": "indexLessThanGeneratedPropertiesAmount",
                  "increment": 1
                },
                "factory": "game.generateToken",
                "owner": "capturedTargetPlayer",
                "tokenType": "generatedProperties.tokenType"
              },
              "cancelledEventTokens": "retainUntilRoundCleanup"
            },
            "eventHandler": {
              "kind": "emptySourceHandler"
            },
            "contingentEventFactory": {
              "registeredAt": "eventUpdate",
              "input": "creationEvent",
              "iteration": "creationEvent.generatedTokensInOrder",
              "system": "PutIntoPlaySystem",
              "properties": {
                "controller": "capturedTargetPlayer",
                "target": "iteratedGeneratedToken",
                "entersReady": {
                  "kind": "logicalOr",
                  "left": "creationEvent.entersReady",
                  "right": "capturedTargetPlayerHasTokenUnitsEnterPlayReady"
                },
                "entryType": "created",
                "enterPlayEffect": "creationEvent.enterPlayEffect"
              },
              "context": "creationEvent.context",
              "contextEvent": "originalInputContextEvent",
              "order": "creationEvent.orderPlusOne"
            },
            "resultEvent": {
              "identity": "creationEvent",
              "generatedTokensField": "creationEvent.generatedTokens",
              "separateFrom": "inputContext.event",
              "continuationBinding": "existingResolvedEventContextProtocol"
            }
          },
          "onFalse": {
            "kind": "noAction"
          }
        }
      }
    ]
  },
  "className": "PazVizslaForABrighterFuture",
  "baseClass": "NonLeaderUnitCard",
  "id": "7107655724",
  "internalName": "paz-vizsla#for-a-brighter-future",
  "status": "supported",
  "card": {
    "title": "Paz Vizsla",
    "subtitle": "For a Brighter Future",
    "cost": 5,
    "hp": 7,
    "power": 4,
    "text": "Sentinel\nWhen Defeated: If this unit wasn't defeated by combat damage, create 2 Mandalorian tokens.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": "“Combat damage” is only the damage dealt during the “End attack” step of an attack.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "7107655724",
    "aspects": [
      "vigilance",
      "command",
      "heroism"
    ],
    "traits": [
      "mandalorian"
    ],
    "arena": "ground",
    "keywords": [
      "sentinel"
    ],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "ASH",
      "number": 28
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 28
      }
    ],
    "internalName": "paz-vizsla#for-a-brighter-future"
  },
  "dataHash": "07c813fa631018b2608e32d220498347299a7227e33068ee90d9855852584f0b",
  "keywordValues": {
    "sentinel": 1
  },
  "requiredCapabilities": [
    "binary:!==",
    "cardType:unit",
    "ir:binary",
    "ir:conditional",
    "ir:const",
    "ir:emptySourceHandler",
    "ir:freshGameEvent",
    "ir:logicalOr",
    "ir:nativeSystemInvocationContext",
    "ir:nativeTokenUnitEventFactory",
    "ir:noAction",
    "ir:ref",
    "ir:undefinedValue",
    "keyword:sentinel",
    "ref:event.defeatSource.type",
    "trigger:whenDefeated"
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
    "setCode": "TOKEN_MANDALORIAN",
    "sourcePath": "game/cards/08_ASH/tokens/Mandalorian.ts",
    "sourceHash": "8f6beea828b30ca108831ac3ec8ffbe7a5f7178808aabfc9ca521d1185fbeba9",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "className": "Mandalorian",
    "baseClass": "TokenUnitCard",
    "id": "8192010342",
    "internalName": "mandalorian",
    "status": "supported",
    "card": {
      "title": "Mandalorian",
      "subtitle": "",
      "cost": 0,
      "hp": 2,
      "power": 2,
      "text": "Shielded (When you create this token, give a Shield token to it.)",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8192010342",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "mandalorian"
      ],
      "arena": "ground",
      "keywords": [
        "shielded"
      ],
      "types": [
        "token",
        "unit"
      ],
      "setId": {
        "set": "ASH"
      },
      "internalName": "mandalorian"
    },
    "dataHash": "5b4ef3b80503a664b91f945e1558638f30ef4bf108631017b5d8fc5009798dda",
    "keywordValues": {
      "shielded": 1
    },
    "requiredCapabilities": [
      "cardType:token",
      "cardType:unit",
      "keyword:shielded"
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
    "setCode": "LAW_079",
    "sourcePath": "game/cards/07_LAW/units/K2SOLockingTheVault.ts",
    "sourceHash": "72d31cd665f1d613eb305fc4720b51e319642e11470de143c0e7f87feacffcae",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "Deal 3 damage to a damaged ground unit",
          "costs": [],
          "optional": true,
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "arena",
            "arena": "ground",
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
                "op": ">",
                "left": {
                  "kind": "ref",
                  "path": [
                    "card",
                    "damage"
                  ]
                },
                "right": {
                  "kind": "const",
                  "value": 0
                }
              }
            }
          },
          "effect": {
            "kind": "damage",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 3
            }
          }
        }
      ]
    },
    "className": "K2SOLockingTheVault",
    "baseClass": "NonLeaderUnitCard",
    "id": "6386443776",
    "internalName": "k2so#locking-the-vault",
    "status": "supported",
    "card": {
      "title": "K-2SO",
      "subtitle": "Locking the Vault",
      "cost": 5,
      "hp": 5,
      "power": 3,
      "text": "Ambush\nOn Attack: You may deal 3 damage to a damaged ground unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": "Ambush is an ability that triggers at the same time as “When Played” abilities. The unit does not have to be ready in order to attack.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6386443776",
      "aspects": [
        "aggression",
        "cunning",
        "heroism"
      ],
      "traits": [
        "rebel",
        "droid"
      ],
      "arena": "ground",
      "keywords": [
        "ambush"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "LAW",
        "number": 79
      },
      "setCodes": [
        {
          "set": "LAW",
          "number": 79
        }
      ],
      "internalName": "k2so#locking-the-vault"
    },
    "dataHash": "8400d238790e36e4e9314fdb54cae8b41c8261eca8d22a5c2a3fdedbc1327a54",
    "keywordValues": {
      "ambush": 1
    },
    "requiredCapabilities": [
      "binary:&&",
      "binary:>",
      "cardType:unit",
      "ir:binary",
      "ir:const",
      "ir:damage",
      "ir:query",
      "ir:ref",
      "keyword:ambush",
      "query:isUnit",
      "ref:card",
      "ref:card.damage",
      "targetType:any",
      "targetZone:arena",
      "trigger:onAttack"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_097",
    "sourcePath": "game/cards/09_HMW/upgrades/DireProwess.ts",
    "sourceHash": "b23073353cdfd617f163606e292c0cd7e76559bf22356c7c5e89c68719ac9fd7",
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
        }
      ]
    },
    "className": "DireProwess",
    "baseClass": "UpgradeCard",
    "id": "3828978526",
    "internalName": "dire-prowess",
    "status": "supported",
    "card": {
      "title": "Dire Prowess",
      "subtitle": "",
      "cost": 2,
      "hp": 1,
      "power": 1,
      "text": "When Played: You may give a Weakness token to a unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": 1,
      "upgradeHp": 1,
      "id": "3828978526",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "learned"
      ],
      "keywords": [],
      "types": [
        "upgrade"
      ],
      "setId": {
        "set": "HMW",
        "number": 97
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 97
        }
      ],
      "internalName": "dire-prowess"
    },
    "dataHash": "84068f3ac1e4df3a3e6f67cda0b8d7b4109a95c14fe1c12cfc2c5813f1c60725",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:upgrade",
      "ir:const",
      "ir:giveWeakness",
      "targetType:unit",
      "targetZone:anyAttackable",
      "trigger:whenPlayed"
    ],
    "diagnostics": []
  },
  {
    "setCode": "TOKEN_WEAKNESS",
    "sourcePath": "game/cards/09_HMW/tokens/Weakness.ts",
    "sourceHash": "dd546e4886757d77fffeb8be6e7381144fa7378b6493ce444aa87e4ba1202778",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "className": "Weakness",
    "baseClass": "TokenUpgradeCard",
    "id": "7816991190",
    "internalName": "weakness",
    "status": "unsupported",
    "card": {
      "title": "Weakness",
      "subtitle": "",
      "cost": 0,
      "hp": -1,
      "power": -1,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": -1,
      "upgradeHp": -1,
      "id": "7816991190",
      "aspects": [],
      "traits": [
        "condition"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "HMW"
      },
      "internalName": "weakness"
    },
    "dataHash": "6a8e3afbc95ff1ddbceb2edf8aafdfd558a8ce5b148c375e45f250792ae96fd7",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:token",
      "cardType:upgrade"
    ],
    "diagnostics": [
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "method:isWeakness",
        "file": "game/cards/09_HMW/tokens/Weakness.ts",
        "line": 12,
        "column": 1,
        "message": "Additional class method requires conversion"
      },
      {
        "code": "UNSUPPORTED_SOURCE",
        "pattern": "class-member:GetAccessor",
        "file": "game/cards/09_HMW/tokens/Weakness.ts",
        "line": 16,
        "column": 1,
        "message": "Additional class behavior requires conversion"
      }
    ]
  },
  {
    "setCode": "SOR_043",
    "sourcePath": "game/cards/01_SOR/events/SuperlaserBlast.ts",
    "sourceHash": "0c54236a10249d9c690b52ff9ce5ba7aa2b45746ff08892710ce94e48246ba81",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Defeat all units",
          "costs": [],
          "effect": {
            "kind": "defeat",
            "target": {
              "kind": "arenaList",
              "receiver": {
                "kind": "ref",
                "path": [
                  "game"
                ]
              },
              "predicate": {
                "kind": "const",
                "value": true
              }
            }
          }
        }
      ]
    },
    "className": "SuperlaserBlast",
    "baseClass": "EventCard",
    "id": "1353201082",
    "internalName": "superlaser-blast",
    "status": "supported",
    "card": {
      "title": "Superlaser Blast",
      "subtitle": null,
      "cost": 8,
      "hp": null,
      "power": null,
      "text": "Defeat all units.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1353201082",
      "aspects": [
        "villainy",
        "vigilance"
      ],
      "traits": [
        "tactic",
        "disaster"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "SOR",
        "number": 43
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 43
        }
      ],
      "internalName": "superlaser-blast"
    },
    "dataHash": "0476510074c7e07d6127589b3af5bfce4b02212eb103a3f96c68fc7c57e87652",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:event",
      "ir:arenaList",
      "ir:const",
      "ir:defeat",
      "ir:ref",
      "ref:game",
      "trigger:event"
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
  },
  {
    "setCode": "ASH_201",
    "id": "9620848758",
    "internalName": "open-circle-ace",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Open Circle Ace",
      "subtitle": "",
      "cost": 1,
      "hp": 2,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9620848758",
      "aspects": [
        "cunning",
        "heroism"
      ],
      "traits": [
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
        "set": "ASH",
        "number": 201
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 201
        }
      ],
      "internalName": "open-circle-ace"
    },
    "dataHash": "7a7c83e894e670c693a2e915324ba4eae814b6df398e4f05fcee03d0ace6bf84",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_242",
    "id": "9011599796",
    "internalName": "death-trooper-squad",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Death Trooper Squad",
      "subtitle": "",
      "cost": 4,
      "hp": 4,
      "power": 5,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9011599796",
      "aspects": [
        "villainy"
      ],
      "traits": [
        "imperial",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 242
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 242
        }
      ],
      "internalName": "death-trooper-squad"
    },
    "dataHash": "a7f39ea84e7cc4dbed97f4fec6e3f7fd9f883e7baacf05f85a605220dd0e4024",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_261",
    "id": "5556203656",
    "internalName": "noti-mobile-pod",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Noti Mobile Pod",
      "subtitle": "",
      "cost": 3,
      "hp": 4,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5556203656",
      "aspects": [],
      "traits": [
        "vehicle",
        "transport"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 261
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 261
        }
      ],
      "internalName": "noti-mobile-pod"
    },
    "dataHash": "716d04746e3d2eec962b92ca123e94221c549d8c313f69600daf5391b4b274ff",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_082",
    "id": "5691187672",
    "internalName": "nala-se#silent-survivor",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Nala Se",
      "subtitle": "Silent Survivor",
      "cost": 2,
      "hp": 7,
      "power": 1,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5691187672",
      "aspects": [
        "vigilance",
        "vigilance"
      ],
      "traits": [
        "kaminoan"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 82
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 82
        }
      ],
      "internalName": "nala-se#silent-survivor"
    },
    "dataHash": "0ac39ace4def5779d56a770c3ccb9e563f406fbbe756f477032197d8d3a3be90",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_093",
    "id": "1226993616",
    "internalName": "coastal-catamarans",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Coastal Catamarans",
      "subtitle": "",
      "cost": 7,
      "hp": 8,
      "power": 8,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1226993616",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "vehicle",
        "transport"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 93
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 93
        }
      ],
      "internalName": "coastal-catamarans"
    },
    "dataHash": "1fa4b66a37dd49ad51881778d42d7b16e7f6ce9234c1faa912976901366c3776",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_116",
    "id": "8382448205",
    "internalName": "ewok-brigade",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Ewok Brigade",
      "subtitle": "",
      "cost": 2,
      "hp": 4,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8382448205",
      "aspects": [
        "command",
        "heroism"
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
        "number": 116
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 116
        }
      ],
      "internalName": "ewok-brigade"
    },
    "dataHash": "4b287f57a4ba7eb94c31c57fe8a10d8aa5bed9f1db2972a0e2258b3f248aded4",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_128",
    "id": "0886129762",
    "internalName": "mace-windu#austere-centrist",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Mace Windu",
      "subtitle": "Austere Centrist",
      "cost": 3,
      "hp": 5,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0886129762",
      "aspects": [
        "command",
        "command"
      ],
      "traits": [
        "force",
        "jedi"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 128
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 128
        }
      ],
      "internalName": "mace-windu#austere-centrist"
    },
    "dataHash": "b5fc19cbe5a74d4b45f0634d2c61022cf49cec278e8e5b6420d8dc778af6fd1d",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_135",
    "id": "0212754007",
    "internalName": "arena-reek#fixture-of-ceremony",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Arena Reek",
      "subtitle": "Fixture of Ceremony",
      "cost": 3,
      "hp": 4,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0212754007",
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
        "set": "HMW",
        "number": 135
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 135
        }
      ],
      "internalName": "arena-reek#fixture-of-ceremony"
    },
    "dataHash": "f777d6aaa62d72173a00b387a5a620b1fa9c1ee5c4fd71e5fb8d1740bcd53996",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_155",
    "id": "7827651708",
    "internalName": "filthy-dianoga",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Filthy Dianoga",
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
      "id": "7827651708",
      "aspects": [
        "aggression",
        "villainy"
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
        "set": "HMW",
        "number": 155
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 155
        }
      ],
      "internalName": "filthy-dianoga"
    },
    "dataHash": "a911dae67e150e760a42ca617af2a5d76a8f2695e73c076f614310f00650a958",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_163",
    "id": "4516882765",
    "internalName": "champions-of-endor",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Champions of Endor",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4516882765",
      "aspects": [
        "aggression",
        "heroism"
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
        "number": 163
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 163
        }
      ],
      "internalName": "champions-of-endor"
    },
    "dataHash": "f4c6d7dca2c473b0fbaf1e6f1bbc2af868a4603d5b4148d211455ecdf7dd5ae1",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_174",
    "id": "3460896271",
    "internalName": "maul#only-revenge-remains",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Maul",
      "subtitle": "Only Revenge Remains",
      "cost": 4,
      "hp": 6,
      "power": 6,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3460896271",
      "aspects": [
        "aggression",
        "aggression"
      ],
      "traits": [
        "force",
        "underworld"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 174
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 174
        }
      ],
      "internalName": "maul#only-revenge-remains"
    },
    "dataHash": "a363184ebdf51d873de8cda19e52ad2f6e958a6f374e6aed54b54035a1ddec46",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_178",
    "id": "5508530442",
    "internalName": "desperate-nantex",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Desperate Nantex",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5508530442",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "separatist",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 178
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 178
        }
      ],
      "internalName": "desperate-nantex"
    },
    "dataHash": "b650d578fab77738145bf3551b1aeb10275578c72efb66a795a0c04e23e44983",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_219",
    "id": "9427322990",
    "internalName": "jaxxon#i-can-hear-you-breathing",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Jaxxon",
      "subtitle": "I Can Hear You Breathing!",
      "cost": 1,
      "hp": 3,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9427322990",
      "aspects": [
        "cunning",
        "cunning"
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
        "set": "HMW",
        "number": 219
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 219
        }
      ],
      "internalName": "jaxxon#i-can-hear-you-breathing"
    },
    "dataHash": "408c4b56d12f0b9bf958d1fc21788ffefb0f9822ff329eed727991cfd23b1520",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  },
  {
    "setCode": "HMW_258",
    "id": "9859570567",
    "internalName": "ryloth-revolutionary-rider",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Ryloth Revolutionary Rider",
      "subtitle": "",
      "cost": 3,
      "hp": 3,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9859570567",
      "aspects": [
        "heroism"
      ],
      "traits": [
        "creature",
        "twi'lek",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 258
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 258
        }
      ],
      "internalName": "ryloth-revolutionary-rider"
    },
    "dataHash": "88718007d28924e8bd937895819db65a4ddf77edfdf927136fd1c8a38a4bbad8",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
    ],
    "diagnostics": []
  }
];
const fillerCodes=["ASH_061","ASH_131","ASH_190","ASH_201","ASH_242","ASH_261","HMW_082","HMW_093","HMW_116","HMW_128","HMW_135","HMW_155","HMW_163","HMW_174","HMW_178","HMW_219","HMW_258"];
function profile(entry){const e=structuredClone(entry);for(const k of ['setCode','id','internalName','className'])delete e[k];for(const k of ['id','internalName','setId','setCodes'])delete e.card[k];return e;}
function classifyDefeatTokenCreation(entry){if(!entry||typeof entry!=='object'||Array.isArray(entry)||!entry.card||typeof entry.setCode!=='string')return null;return same(profile(entry),profile(reviewedSource))?{family:'defeat-token-creation',sourceArena:'ground'}:null;}
const causeDonors={combat:['ASH_131'], 'ability-damage':['TWI_174'],'direct-defeat':['TWI_077'],'attack-ability-damage':['TWI_174','LAW_079'],'framework-weakness':['TWI_174','HMW_097','SOR_095','TOKEN_WEAKNESS'],'unique-rule':[],'simultaneous-defeat':['SOR_043']};
function planDefeatTokenCreationProbes(entry,options={}){const shape=classifyDefeatTokenCreation(entry);if(!shape)return null;const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};function obligation(name,reason){const id=entry.setCode+':defeat-token-creation:'+name;result.obligations.push({id,path:'defeat-token-creation',dimension:name});if(reason)result.uncovered.push({id,path:'defeat-token-creation',reason});return id;}function donor(code){const rows=catalogue.filter(x=>x.setCode===code);return rows.length===1&&same(rows[0],reviewedDonors.find(x=>x.setCode===code));}
for(const owner of [0,1])for(const cause of Object.keys(causeDonors)){const suffix='defeat-token-creation-owner-'+owner+'-'+cause;const setupCardIds=[...new Set(['SOR_014','SOR_023','TOKEN_MANDALORIAN','TOKEN_SHIELD',...fillerCodes,...causeDonors[cause]])].sort();if(!setupCardIds.every(donor)){obligation(suffix,'Complete reviewed native token/default/filler/cause donor profile missing, ambiguous or changed');continue;}const id=obligation(suffix);result.cases.push({id:'auto/'+entry.setCode+'/'+suffix,name:suffix,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['defeat-token-creation','when-defeated','sentinel','shielded'],assertionStages:['construction','paid-play','sentinel-targets','captured-defeat','token-creation-events','physical-token-batch','terminal'],coverageObligationIds:[id],probe:{...shape,cardId:entry.setCode,owner,cause,resourceCount:24,leaderCode:'SOR_014',baseCode:'SOR_023'}});}
for(const [name,reason]of Object.entries({'source-review':'Plans and engine recordings do not establish native typed lowering, import/hash/copy correctness or independent review.','missing-null-cause':'No actual normal-route construction established; unsupported imported causes must reject.','source-controller-change':'No source-controlled change route constructed; captured context player must not be replaced with current chooser.','capacity-boundary':'Native-only capacity boundary not source-oracle admission evidence; no source capacity game constructed.','replacement-cancellation':'No actual creation replacement or cancellation route constructed.'}))obligation(name,reason);return result;}
module.exports={classifyDefeatTokenCreation,planDefeatTokenCreationProbes};
