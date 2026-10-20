"use strict";
const {isDeepStrictEqual: same} = require("node:util");
// Complete source envelope: no unknown registration, keyword, or native lifecycle
// is hidden by selecting the one leaf this constructor knows how to exercise.
const expected = {
  "setCode": "ASH_041",
  "sourcePath": "game/cards/08_ASH/units/OutcastMercenaryStarship.ts",
  "sourceHash": "672ab37c6f6edd626674463466651427e444157d065ea4ff3e46544d2e7238d6",
  "className": "OutcastMercenaryStarship",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Outcast",
    "subtitle": "Mercenary Starship",
    "cost": 2,
    "hp": 4,
    "power": 1,
    "text": "When a friendly unit enters play (including this one): It gets +1/+0 for this phase.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "5394891260",
    "aspects": [
      "aggression",
      "cunning"
    ],
    "traits": [
      "underworld",
      "vehicle",
      "transport"
    ],
    "arena": "space",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "ASH",
      "number": 41
    },
    "setCodes": [
      {
        "set": "ASH",
        "number": 41
      }
    ],
    "internalName": "outcast#mercenary-starship"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "eventHooks",
        "title": "Give that unit +1/+0 for this phase",
        "costs": [],
        "contextTitle": [
          {
            "kind": "const",
            "value": "Give "
          },
          {
            "kind": "ref",
            "path": [
              "unitEntryCard",
              "title"
            ]
          },
          {
            "kind": "const",
            "value": " +1/+0 for this phase"
          }
        ],
        "effect": {
          "kind": "modifyStats",
          "target": {
            "kind": "ref",
            "path": [
              "unitEntryCard"
            ]
          },
          "duration": "phase",
          "power": {
            "kind": "const",
            "value": 1
          },
          "hp": {
            "kind": "const",
            "value": 0
          }
        },
        "hooks": [
          {
            "event": "onUnitEntersPlay",
            "predicate": {
              "kind": "binary",
              "op": "===",
              "left": {
                "kind": "dataPath",
                "root": [
                  "event"
                ],
                "argumentBinding": {
                  "id": "callback:846:0",
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
        ]
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": null,
  "piloting": null
};
const normalize = e => JSON.parse(JSON.stringify({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null}));
function classifyUnitEntry(entry) {
  return entry?.status === "supported" && !entry.diagnostics?.length && same(normalize(entry), expected)
    ? {family:"unit-entry", event:"onUnitEntersPlay", target:"physical-event-card"} : null;
}
function planUnitEntryProbes(entry, {catalogue} = {}) {
  if (!classifyUnitEntry(entry)) return null;
  const cases = [], obligations = [], uncovered = [];
  for (const sourceSeat of [1,2]) for (const [variant, setupCardIds, stages] of constructions) {
    const name = variant + "-seat" + sourceSeat;
    const missing = setupCardIds.filter(id => !isUnitEntryDonor(catalogue?.cards?.find(e => e.setCode === id)));
    const id = entry.setCode + ":unit-entry:" + name;
    obligations.push({id,path:"unit-entry",dimension:name});
    const rejected = setupCardIds.filter(id => Object.hasOwn(rejectedDonors, id));
    if (rejected.length) {uncovered.push({id,path:"unit-entry",reason:"Source-engine diagnostic construction only; runtime donor rejected: "+rejected.map(id=>id+" ("+rejectedDonors[id]+")").join(", ")});continue;}
    if (missing.length) {uncovered.push({id,path:"unit-entry",reason:"Missing or unaudited complete construction donors: "+missing.join(", ")});continue;}
    cases.push({id:"auto/"+entry.setCode+"/"+name,name,kind:"probe",
      source:{module:"../pipeline/unit-entry-probe-fixture.js",scenario:name},
      focusCardIds:[entry.setCode],setupCardIds,
      behaviorIds:["unit-entry","physical-event-card","phase-stats"],
      assertionStages:stages,coverageObligationIds:[id],
      probe:{family:"unit-entry",cardId:entry.setCode,sourceSeat,scenario:variant}});
  }
  for (const dimension of ["rescue","piloting-no-entry","control-change-no-entry","simultaneous-source-departure","foreign-owner-source","enemy-controlled-owned-source"]) {
    const id = entry.setCode+":unit-entry:"+dimension;
    obligations.push({id,path:"unit-entry",dimension});
    uncovered.push({id,path:"unit-entry",reason:"Requires audited printed construction and supported strict ownership/lifecycle protocol; raw diagnostic oracle alone does not establish runtime parity."});
  }
  return {schema:"swu-card-probes/v1",cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
const donorShapes = {
  "SOR_095": {
    "setCode": "SOR_095",
    "sourcePath": null,
    "sourceHash": null,
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
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "TWI_237": {
    "setCode": "TWI_237",
    "sourcePath": "game/cards/03_TWI/events/DroidDeployment.ts",
    "sourceHash": "78099d889d73f4f69f9363b6dcc6e8b32e6b086e13d17c933979ad6cbb08632c",
    "className": "DroidDeployment",
    "baseClass": "EventCard",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "ASH_111": {
    "setCode": "ASH_111",
    "sourcePath": "game/cards/08_ASH/units/ChildrenOfTheWatch.ts",
    "sourceHash": "eec2dadc32de3f3fc73866527569fbd32931544bca67248b5df79b36533fc15c",
    "className": "ChildrenOfTheWatch",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Children of the Watch",
      "subtitle": "",
      "cost": 6,
      "hp": 3,
      "power": 3,
      "text": "When Played: Create 2 Mandalorian tokens.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5980742717",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "mandalorian"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 111
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 111
        }
      ],
      "internalName": "children-of-the-watch"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Create 2 Mandalorian tokens",
          "costs": [],
          "effect": {
            "kind": "createTokenUnit",
            "tokenType": "mandalorian",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "SHD_037": {
    "setCode": "SHD_037",
    "sourcePath": "game/cards/02_SHD/units/SupremeLeaderSnokeShadowRuler.ts",
    "sourceHash": "7f15a401fed97b0662d7be1b1502cc3650592ab474fa7713cf9b4ede0c4b5567",
    "className": "SupremeLeaderSnokeShadowRuler",
    "baseClass": "NonLeaderUnitCard",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "SEC_105": {
    "setCode": "SEC_105",
    "sourcePath": "game/cards/06_SEC/events/RenewedFriendship.ts",
    "sourceHash": "7359c2654c3e48a828332e243f323f8de4d83b8c291a3139566de0d728eeda5b",
    "className": "RenewedFriendship",
    "baseClass": "EventCard",
    "card": {
      "title": "Renewed Friendship",
      "subtitle": "",
      "cost": 4,
      "hp": null,
      "power": null,
      "text": "Return a unit from your discard pile to your hand.\nCreate 2 Spy tokens.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1744140192",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "innate"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "SEC",
        "number": 105
      },
      "setCodes": [
        {
          "set": "SEC",
          "number": 105
        }
      ],
      "internalName": "renewed-friendship"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Return a unit from your discard pile to your hand. Create 2 Spy tokens",
          "costs": [],
          "effect": {
            "kind": "simultaneous",
            "effects": [
              {
                "kind": "select",
                "targetResolver": {
                  "owner": "self",
                  "type": "basicUnit",
                  "zone": "discard"
                },
                "effect": {
                  "kind": "returnToHand",
                  "target": "chosen"
                }
              },
              {
                "kind": "createTokenUnit",
                "tokenType": "spy",
                "amount": {
                  "kind": "const",
                  "value": 2
                },
                "target": "ownPlayer",
                "entersReady": false
              }
            ]
          }
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "JTL_121": {
    "setCode": "JTL_121",
    "sourcePath": "game/cards/04_JTL/events/Salvage.ts",
    "sourceHash": "bcbf68f248e12c53e39a6e20125e9a6d6c96ed485645b5eba012075fef111109",
    "className": "Salvage",
    "baseClass": "EventCard",
    "card": {
      "title": "Salvage",
      "subtitle": "",
      "cost": 0,
      "hp": null,
      "power": null,
      "text": "Play a Vehicle unit from your discard pile (paying its cost). Then, deal 1 damage to it.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "Abilities that let you play a card require you to pay that card’s cost unless specified otherwise.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2614693321",
      "aspects": [
        "command"
      ],
      "traits": [
        "supply"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "JTL",
        "number": 121
      },
      "setCodes": [
        {
          "set": "JTL",
          "number": 121
        }
      ],
      "internalName": "salvage"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Play a Vehicle unit from your discard pile (paying its cost)",
          "costs": [],
          "targetResolver": {
            "owner": "self",
            "type": "any",
            "zone": "discard",
            "predicate": {
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
          "effect": {
            "kind": "playCardSystem",
            "helper": "playCardFromOutOfPlay",
            "target": "chosen",
            "ignoredRequirements": [
              "phase"
            ],
            "optional": false,
            "entersReady": false,
            "playType": "playFromOutOfPlay",
            "playAsType": "unit",
            "nested": false,
            "canPlayFromAnyZone": true
          },
          "then": {
            "title": "Deal 1 damage to the played unit",
            "costs": [],
            "effect": {
              "kind": "damage",
              "target": "chosen",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "LAW_043": {
    "setCode": "LAW_043",
    "sourcePath": "game/cards/07_LAW/events/ShadowCloaking.ts",
    "sourceHash": "5e8c4e3134886c781c698033dd5050287e0c55fca2ac1a1388e147a1e579ac66",
    "className": "ShadowCloaking",
    "baseClass": "EventCard",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "ASH_060": {
    "setCode": "ASH_060",
    "sourcePath": "game/cards/08_ASH/units/CobbVanthLetMeHandleThis.ts",
    "sourceHash": "afb4756a04a702f17a8592ad7a951ade0aeebd556f7a2f85b931db4ecde78365",
    "className": "CobbVanthLetMeHandleThis",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Cobb Vanth",
      "subtitle": "Let Me Handle This",
      "cost": 4,
      "hp": 6,
      "power": 2,
      "text": "Grit\nWhen you play another unit: You may deal 2 damage to this unit. If you do, give a Shield token to that unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2100091585",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "official"
      ],
      "arena": "ground",
      "keywords": [
        "grit"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 60
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 60
        }
      ],
      "internalName": "cobb-vanth#let-me-handle-this"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Deal 2 damage to this unit to give a Shield token to that unit",
          "costs": [],
          "contextTitle": [
            {
              "kind": "const",
              "value": "Deal 2 damage to "
            },
            {
              "kind": "ref",
              "path": [
                "source",
                "title"
              ]
            },
            {
              "kind": "const",
              "value": " to give a Shield token to "
            },
            {
              "kind": "dataPath",
              "root": [
                "context"
              ],
              "contextBinding": {
                "id": "callback:805:0",
                "role": "contextTitle",
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
                  "key": "card",
                  "optional": false
                },
                {
                  "key": "title",
                  "optional": false
                }
              ]
            },
            {
              "kind": "const",
              "value": ""
            }
          ],
          "optional": true,
          "effect": {
            "kind": "damage",
            "target": "self",
            "amount": {
              "kind": "const",
              "value": 2
            }
          },
          "ifYouDo": {
            "title": "Give a Shield token to [value]",
            "costs": [],
            "effect": {
              "kind": "giveShield",
              "target": {
                "kind": "dataPath",
                "root": [
                  "context"
                ],
                "contextBinding": {
                  "id": "callback:1392:0",
                  "role": "ifYouDo",
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
                    "key": "card",
                    "optional": false
                  }
                ]
              },
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
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
                        "id": "callback:966:0",
                        "bindAt": "callbackInvocation",
                        "readAt": "expressionEvaluation"
                      },
                      "steps": [
                        {
                          "key": "cardTypeWhenInPlay",
                          "optional": false
                        }
                      ]
                    },
                    "right": {
                      "kind": "const",
                      "value": "basicUnit"
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
                        "id": "callback:966:0",
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
                      "id": "callback:966:0",
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
    "keywordValues": {
      "grit": 1
    },
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "SOR_014": {
    "setCode": "SOR_014",
    "sourcePath": "game/cards/01_SOR/leaders/SabineWrenGalvanizedRevolutionary.ts",
    "sourceHash": "b1d4463b24bdf9485f4b3528bde3cb1c7c328a4d1e6facdfa9923c2faf0484a4",
    "className": "SabineWrenGalvanizedRevolutionary",
    "baseClass": "LeaderUnitCard",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "TWI_226": {
    "setCode": "TWI_226",
    "sourcePath": "game/cards/01_SOR/events/Waylay.ts",
    "sourceHash": "93c1da09d37785bcfad71f02ae59ca9a556ecef2c002b61e251d205fbf62ea41",
    "className": "Waylay",
    "baseClass": "EventCard",
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
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "TOKEN_BATTLE_DROID": {
    "setCode": "TOKEN_BATTLE_DROID",
    "sourcePath": "game/cards/03_TWI/tokens/BattleDroid.ts",
    "sourceHash": "dc0c56c80105d04930c35405295126f330c935c7cc56c7c6d78da044b129eb93",
    "className": "BattleDroid",
    "baseClass": "TokenUnitCard",
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
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "TOKEN_MANDALORIAN": {
    "setCode": "TOKEN_MANDALORIAN",
    "sourcePath": "game/cards/08_ASH/tokens/Mandalorian.ts",
    "sourceHash": "8f6beea828b30ca108831ac3ec8ffbe7a5f7178808aabfc9ca521d1185fbeba9",
    "className": "Mandalorian",
    "baseClass": "TokenUnitCard",
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
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "keywordValues": {
      "shielded": 1
    },
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  },
  "TOKEN_SHIELD": {
    "setCode": "TOKEN_SHIELD",
    "sourcePath": "game/cards/01_SOR/tokens/Shield.ts",
    "sourceHash": "39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f",
    "className": "Shield",
    "baseClass": "TokenUpgradeCard",
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
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null,
    "status": "unsupported",
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
  "TOKEN_SPY": {
    "setCode": "TOKEN_SPY",
    "sourcePath": "game/cards/06_SEC/tokens/Spy.ts",
    "sourceHash": "aae6ac20865fac5b7c5eace59f33b781c75a578c07cf494bdc28b337c5d42f1f",
    "className": "Spy",
    "baseClass": "TokenUnitCard",
    "card": {
      "title": "Spy",
      "subtitle": "",
      "cost": 0,
      "hp": 2,
      "power": 0,
      "text": "Raid 2 (This unit gets +2/+0 while attacking.)",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6665455613",
      "aspects": [],
      "traits": [
        "official"
      ],
      "arena": "ground",
      "keywords": [
        "raid"
      ],
      "types": [
        "token",
        "unit"
      ],
      "setId": {
        "set": "SEC"
      },
      "internalName": "spy"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "keywordValues": {
      "raid": 2
    },
    "attachCondition": null,
    "piloting": null,
    "status": "supported",
    "diagnostics": []
  }
};
const rejectedDonors = Object.freeze({SEC_105:"unverified simultaneous return-to-hand and Spy creation",JTL_121:"unimplemented playCardSystem",ASH_060:"unimplemented card-play event target callbacks"});
const donorCodes = Object.freeze(Object.keys(donorShapes));
function isUnitEntryDonor(entry) {return !!entry && same({...normalize(entry),status:entry.status,diagnostics:entry.diagnostics},donorShapes[entry.setCode]);}
const constructions = [
  [
    "self-entry",
    [],
    [
      "self-entry",
      "exact-event-target",
      "phase-expiry"
    ]
  ],
  [
    "friendly-entry",
    [
      "SOR_095"
    ],
    [
      "friendly-power-four",
      "exact-event-target"
    ]
  ],
  [
    "enemy-entry",
    [
      "SOR_095"
    ],
    [
      "enemy-power-three",
      "no-grant"
    ]
  ],
  [
    "multiple-entrants-next",
    [
      "TWI_237",
      "TOKEN_BATTLE_DROID"
    ],
    [
      "resolve-next",
      "distinct-physical-targets"
    ]
  ],
  [
    "multiple-entrants-all",
    [
      "TWI_237",
      "TOKEN_BATTLE_DROID"
    ],
    [
      "resolve-all",
      "distinct-physical-targets"
    ]
  ],
  [
    "unique-keep-incoming",
    [],
    [
      "capture-before-uniqueness",
      "old-source-survives-capture",
      "incoming-power-three"
    ]
  ],
  [
    "unique-keep-old",
    [],
    [
      "capture-before-uniqueness",
      "dead-incoming-receives-two-effects",
      "survivor-power-one"
    ]
  ],
  [
    "dead-mandalorian-batch",
    [
      "ASH_111",
      "SHD_037",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    [
      "entry-before-stat-death",
      "offboard-physical-targets",
      "shield-no-resurrection"
    ]
  ],
  [
    "repeated-dead-token-batch",
    [
      "ASH_111",
      "SHD_037",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    [
      "four-distinct-token-identities",
      "one-effect-per-entry"
    ]
  ],
  [
    "renewed-friendship-replay",
    [
      "SEC_105",
      "TWI_226",
      "TOKEN_SPY"
    ],
    [
      "dead-incoming-effects",
      "clear-on-discard-to-hand",
      "fresh-self-entry-power-two"
    ]
  ],
  [
    "salvage-replay",
    [
      "JTL_121",
      "TWI_226"
    ],
    [
      "dead-incoming-effects",
      "clear-on-discard-to-arena",
      "fresh-self-entry-power-two"
    ]
  ],
  [
    "renewed-friendship-expired",
    [
      "SEC_105",
      "TWI_226",
      "TOKEN_SPY"
    ],
    [
      "offboard-phase-expiry",
      "fresh-self-entry-power-two"
    ]
  ],
  [
    "salvage-expired",
    [
      "JTL_121",
      "TWI_226"
    ],
    [
      "offboard-phase-expiry",
      "fresh-self-entry-power-two"
    ]
  ],
  [
    "phase-expiry",
    [
      "SOR_095"
    ],
    [
      "power-four-before-regroup",
      "power-three-after-regroup"
    ]
  ],
  [
    "source-departure",
    [
      "SOR_095",
      "TWI_226"
    ],
    [
      "source-departure-keeps-recipient-effect"
    ]
  ],
  [
    "recipient-reentry",
    [
      "SOR_095",
      "TWI_226"
    ],
    [
      "same-physical-recipient",
      "old-effect-clears",
      "one-new-grant"
    ]
  ],
  [
    "leader-deploy",
    [
      "SOR_014"
    ],
    [
      "deploy-emits-unit-entry",
      "leader-power-three"
    ]
  ],
  [
    "buffed-combat",
    [
      "SOR_095",
      "LAW_043",
      "TOKEN_SHIELD"
    ],
    [
      "readied-by-printed-event",
      "four-base-damage"
    ]
  ],
  [
    "mixed-trigger-order-outcast-first",
    [
      "SOR_095",
      "ASH_060",
      "TOKEN_SHIELD"
    ],
    [
      "exact-context-titles",
      "outcast-before-cobb",
      "power-four-and-shield"
    ]
  ],
  [
    "mixed-trigger-order-cobb-first",
    [
      "SOR_095",
      "ASH_060",
      "TOKEN_SHIELD"
    ],
    [
      "exact-context-titles",
      "cobb-before-outcast",
      "power-four-and-shield"
    ]
  ]
];
module.exports = {classifyUnitEntry,planUnitEntryProbes,isUnitEntryDonor,donorCodes};
