'use strict';
const same=require('node:util').isDeepStrictEqual;
function sourceProfile(entry){const e=structuredClone(entry);for(const key of ['setCode','id','internalName','className'])delete e[key];if(e.card)for(const key of ['id','internalName','setId','setCodes'])delete e.card[key];return e;}
// Full raw source and helper projection, including mandatory per-event identity
// predicate and native/intrinsic context. Card identity is not the classifier.
const reviewedSource={
  "setCode": "ASH_204",
  "sourcePath": "game/cards/08_ASH/units/BladeThreeBaneOfTheDevastator.ts",
  "sourceHash": "17006e846d35f5aa945d2212d5978f7dd0f1cf9d0e23a7ee4795670b0cde2bdd",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "eventHooks",
        "title": "Give an Advantage token to this unit",
        "costs": [],
        "effect": {
          "kind": "giveAdvantage",
          "target": "self",
          "amount": {
            "kind": "const",
            "value": 1
          },
          "createdBy": "self",
          "batch": "singleCreationEvent",
          "tokenOwner": "recipientController"
        },
        "hooks": [
          {
            "event": "onDamageDealt",
            "predicate": {
              "kind": "binary",
              "op": "===",
              "left": {
                "kind": "dataPath",
                "root": [
                  "event"
                ],
                "argumentBinding": {
                  "id": "callback:761:0",
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
                  "player",
                  "base"
                ]
              }
            }
          }
        ]
      }
    ]
  },
  "className": "BladeThreeBaneOfTheDevastator",
  "baseClass": "NonLeaderUnitCard",
  "id": "7634552408",
  "internalName": "blade-three#bane-of-the-devastator",
  "status": "supported",
  "card": {
    "title": "Blade Three",
    "subtitle": "Bane of the Devastator",
    "cost": 3,
    "hp": 4,
    "power": 2,
    "text": "When your base is dealt damage: Give an Advantage token to this unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "7634552408",
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
      "set": "ASH",
      "number": 204
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 204
      }
    ],
    "internalName": "blade-three#bane-of-the-devastator"
  },
  "dataHash": "00cfafeaa37cef184c2ce75f115dc271aba491a9d535642c1ac51fe6d12b97b6",
  "keywordValues": {},
  "requiredCapabilities": [
    "binary:===",
    "cardType:unit",
    "ir:binary",
    "ir:const",
    "ir:dataPath",
    "ir:giveAdvantage",
    "ir:ref",
    "ref:player.base",
    "trigger:eventHooks"
  ],
  "diagnostics": []
};
const reviewedDonors=[
  {
    "setCode": "ASH_025",
    "id": "6815244860",
    "internalName": "emperors-observatory",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Emperor's Observatory",
      "subtitle": "",
      "cost": null,
      "hp": 30,
      "power": null,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6815244860",
      "aspects": [
        "cunning"
      ],
      "traits": [
        "pillio"
      ],
      "keywords": [],
      "types": [
        "base"
      ],
      "setId": {
        "set": "ASH",
        "number": 25
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 25
        }
      ],
      "internalName": "emperors-observatory"
    },
    "dataHash": "7a69da368d5bd80906a3a2829e772d6a1bc27a549f3c3068b22439fcff7ee47d",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:base"
    ],
    "diagnostics": []
  },
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
    "setCode": "LOF_254",
    "id": "2971660620",
    "internalName": "porg",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Porg",
      "subtitle": "",
      "cost": 0,
      "hp": 1,
      "power": 1,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2971660620",
      "aspects": [],
      "traits": [
        "creature"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "LOF",
        "number": 254
      },
      "setCodes": [
        {
          "set": "LOF",
          "number": 254
        }
      ],
      "internalName": "porg"
    },
    "dataHash": "d87257b2fc19c9b8967340f43fa12aacf6f99bc3d77ef70ccf62eb540b76de64",
    "keywordValues": {},
    "requiredCapabilities": [
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
    "setCode": "IBH_063",
    "id": "0581968676",
    "internalName": "snowtrooper",
    "sourcePath": null,
    "sourceHash": null,
    "status": "supported",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "card": {
      "title": "Snowtrooper",
      "subtitle": "",
      "cost": 1,
      "hp": 3,
      "power": 1,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0581968676",
      "aspects": [
        "aggression",
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
        "set": "IBH",
        "number": 63
      },
      "setCodes": [
        {
          "set": "IBH",
          "number": 63
        },
        {
          "set": "IBH",
          "number": 77
        },
        {
          "set": "IBH",
          "number": 87
        }
      ],
      "internalName": "snowtrooper"
    },
    "dataHash": "b0098ed28549e70134c30e609f7740ca2ec17d4b4c362dc728af7e2a578361dc",
    "keywordValues": {},
    "requiredCapabilities": [
      "cardType:unit"
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
    "setCode": "ASH_151",
    "sourcePath": "game/cards/08_ASH/events/OperationCinder.ts",
    "sourceHash": "cf245ad766b2c283f3df98dd3748c5883f040c89a0abe3220ff02dca7b7fea7c",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Deal 5 damage to your base",
          "costs": [],
          "effect": {
            "kind": "damage",
            "target": "ownBase",
            "amount": {
              "kind": "const",
              "value": 5
            }
          },
          "then": {
            "title": "Deal 5 damage to each unit",
            "costs": [],
            "effect": {
              "kind": "damage",
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
              },
              "amount": {
                "kind": "const",
                "value": 5
              }
            }
          }
        }
      ]
    },
    "className": "OperationCinder",
    "baseClass": "EventCard",
    "id": "6062959089",
    "internalName": "operation-cinder",
    "status": "supported",
    "card": {
      "title": "Operation Cinder",
      "subtitle": "",
      "cost": 6,
      "hp": null,
      "power": null,
      "text": "Deal 5 damage to your base. Then, deal 5 damage to each unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "All damage dealt to units by Operation Cinder is dealt simultaneously.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6062959089",
      "aspects": [
        "aggression",
        "villainy"
      ],
      "traits": [
        "disaster",
        "plan"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "ASH",
        "number": 151
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 151
        }
      ],
      "internalName": "operation-cinder"
    },
    "dataHash": "2731fbda4351987a37248e3d72038b9053430786b21f663008f48e5c2c7fe9b2",
    "keywordValues": {},
    "requiredCapabilities": [
      "ability:then",
      "cardType:event",
      "ir:arenaList",
      "ir:const",
      "ir:damage",
      "ir:ref",
      "ref:game",
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
function classifyBladeBaseDamage(entry){if(!entry||typeof entry!=='object'||Array.isArray(entry)||typeof entry.setCode!=='string')return null;return same(sourceProfile(entry),sourceProfile(reviewedSource))?{family:'blade-base-damage',sourceArena:'space'}:null;}
function planBladeBaseDamageProbes(entry,options={}){
    const shape=classifyBladeBaseDamage(entry);if(!shape)return null;
    const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name,reason){const id=entry.setCode+':blade-base-damage:'+name;result.obligations.push({id,path:'blade-base-damage',dimension:name});if(reason)result.uncovered.push({id,path:'blade-base-damage',reason});return id;}
    function donor(code){const rows=catalogue.filter(e=>e.setCode===code);return rows.length===1&&same(rows[0],reviewedDonors.find(e=>e.setCode===code));}
    obligation('source-review','Construction does not prove full source lowering, damage-event capture, terminal completion or native Advantage lifecycle; independent source-bound review required');
    const fillerCodes=['ASH_061','ASH_131','ASH_190','ASH_201','ASH_242','ASH_261','HMW_082','HMW_093','HMW_116','HMW_128','HMW_135','HMW_155','HMW_163','HMW_174','HMW_178','HMW_219','HMW_258'];
    const common=['ASH_025','SOR_014','SOR_023','TOKEN_ADVANTAGE',...fillerCodes];
    const scenarios={
        'paid-play-damage1':[],
        'damage3':['SOR_095'],
        'separate-events':['LOF_254'],
        'enemy-base-negative':['SOR_095'],
        'friendly-unit-negative':['SOR_095'],
        'shield-cancel-unit':['SOR_095','TOKEN_SHIELD'],
        'zero-base-combat':['SHD_037','IBH_063'],
        'source-gone-before':['TWI_226'],
        'source-gone-within-cinder':['ASH_151'],
        'terminal-own-base':[],
        'advantage-attack-expiry':['LAW_043','TOKEN_SHIELD'],
        'advantage-phase-persistence':[]
    };
    for(const owner of [0,1])for(const [scenario,extra]of Object.entries(scenarios)){
        const name='owner-'+owner+'-'+scenario,setupCardIds=[...new Set([...common,...extra])].sort();
        if(!setupCardIds.every(donor)){obligation(name,'Required complete native token, default leader/base or admitted scenario donor is missing, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario,resourceCount:12,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'ASH_025',enemyBaseCode:'ASH_025',fillerCodes:[...fillerCodes]};
        result.cases.push({id:'auto/'+entry.setCode+'/blade-base-damage-'+name,name:'blade-base-damage-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['blade-base-damage','on-damage-dealt','advantage'],assertionStages:['construction','activation','observation','terminal'],coverageObligationIds:[id],probe});
    }
    for(const name of ['cancelled-own-base-damage','indirect-and-overwhelm','two-events-one-parent-window','source-controller-transfer','mixed-effective-trigger-owners','advantage-defense-expiry'])obligation(name,'No standard actual source-card construction in the sealed design; retained unsupported ASH070 diagnostics add no coverage or admission credit');
    return result;
}
module.exports={classifyBladeBaseDamage,planBladeBaseDamageProbes};
