'use strict';
// Exact reviewed full-card source shapes; only ordinary leader setups are differential eligible.
const same = require('node:util').isDeepStrictEqual;
const shapes = [
  {
    "setCode": "ASH_113",
    "sourcePath": "game/cards/08_ASH/units/MandalorianFlagshipCapturedFromTheEmpire.ts",
    "sourceHash": "46bf7414c5f6e9f9b566fa98c3e77b98a9760c0f62fce0e1f091785cb1cea09c",
    "className": "MandalorianFlagshipCapturedFromTheEmpire",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Mandalorian Flagship",
      "subtitle": "Captured from the Empire",
      "cost": 7,
      "hp": 8,
      "power": 4,
      "text": "While you control a leader unit, this unit gains Ambush. (When you play this unit, it may attack an enemy unit.)\nThis unit gets +1/+0 for each other friendly Mandalorian unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": "Ambush is an ability that triggers at the same time as “When Played” abilities. The unit does not have to be ready in order to attack.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "7405707601",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "mandalorian",
        "vehicle",
        "capital ship"
      ],
      "arena": "space",
      "keywords": [
        "ambush"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 113
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 113
        }
      ],
      "internalName": "mandalorian-flagship#captured-from-the-empire"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While you control a leader unit, this unit gains Ambush",
          "costs": [],
          "effect": {
            "kind": "gainKeyword",
            "target": "self",
            "duration": "persistent",
            "keyword": "ambush",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
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
                  "kind": "const",
                  "value": true
                },
                "right": {
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
                    "name": "isLeader",
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
            ]
          }
        },
        {
          "trigger": "constant",
          "title": "This unit gets +1/+0 for each other friendly Mandalorian unit",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "self",
            "duration": "persistent",
            "power": {
              "kind": "query",
              "name": "arenaUnitCount",
              "receiver": {
                "kind": "ref",
                "path": [
                  "modifierTarget",
                  "controller"
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
                          "value": "mandalorian"
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
                          "modifierTarget"
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
                          "modifierTarget"
                        ]
                      }
                    }
                  }
                }
              ]
            },
            "hp": {
              "kind": "const",
              "value": 0
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null
  },
  {
    "setCode": "ASH_240",
    "sourcePath": "game/cards/08_ASH/units/MandalorianSuperCommandos.ts",
    "sourceHash": "84e8f2400501f701a6139f247f864b7650e41f22748159ddd1cecf396a00c38f",
    "className": "MandalorianSuperCommandos",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Mandalorian Super Commandos",
      "subtitle": "",
      "cost": 3,
      "hp": 5,
      "power": 2,
      "text": "While you control a leader unit, this unit gets +2/+0.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3523847566",
      "aspects": [
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
        "number": 240
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 240
        }
      ],
      "internalName": "mandalorian-super-commandos"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While you control a leader unit, this unit gets +2/+0",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "self",
            "duration": "persistent",
            "power": {
              "kind": "const",
              "value": 2
            },
            "hp": {
              "kind": "const",
              "value": 0
            }
          },
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
                  "kind": "const",
                  "value": true
                },
                "right": {
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
                    "name": "isLeader",
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
            ]
          }
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null
  }
];
const scenarios = [
  {
    "name": "ASH_113-paid-base-leader-ordinary-token-negative-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SEC_028"
    ],
    "design": {
      "name": "ASH_113-paid-base-leader-ordinary-token-negative-p1",
      "family": "arena-leader-predicate-negative",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "ordinaryGround",
          "code": "SEC_028",
          "owner": "own"
        },
        {
          "key": "ordinarySpace",
          "code": "JTL_212",
          "owner": "own"
        }
      ],
      "initialTokens": [
        {
          "key": "groundToken",
          "token": "battle-droid"
        },
        {
          "key": "spaceToken",
          "token": "tie-fighter"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-base-leader-ordinary-token-negative-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SEC_028"
    ],
    "design": {
      "name": "ASH_113-paid-base-leader-ordinary-token-negative-p2",
      "family": "arena-leader-predicate-negative",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "ordinaryGround",
          "code": "SEC_028",
          "owner": "own"
        },
        {
          "key": "ordinarySpace",
          "code": "JTL_212",
          "owner": "own"
        }
      ],
      "initialTokens": [
        {
          "key": "groundToken",
          "token": "battle-droid"
        },
        {
          "key": "spaceToken",
          "token": "tie-fighter"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-opposing-only-deployed-leader-negative-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SOR_014"
    ],
    "design": {
      "name": "ASH_113-paid-opposing-only-deployed-leader-negative-p1",
      "family": "arena-leader-controller-boundary",
      "focusCardId": "ASH_113",
      "enemyLeaderAlias": true,
      "units": [
        {
          "key": "enemySpace",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader",
          "actor": "enemy"
        },
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-opposing-only-deployed-leader-negative-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SOR_014"
    ],
    "design": {
      "name": "ASH_113-paid-opposing-only-deployed-leader-negative-p2",
      "family": "arena-leader-controller-boundary",
      "focusCardId": "ASH_113",
      "enemyLeaderAlias": true,
      "units": [
        {
          "key": "enemySpace",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader",
          "actor": "enemy"
        },
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-actual-ground-deploy-then-defeat-live-update-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SOR_014",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_113-actual-ground-deploy-then-defeat-live-update-p1",
      "family": "arena-leader-live-deploy-loss",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader"
        },
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_113-actual-ground-deploy-then-defeat-live-update-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "JTL_212",
      "SOR_014",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_113-actual-ground-deploy-then-defeat-live-update-p2",
      "family": "arena-leader-live-deploy-loss",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader"
        },
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_240-paid-base-leader-ordinary-token-negative-p1",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "JTL_212",
      "SEC_028"
    ],
    "design": {
      "name": "ASH_240-paid-base-leader-ordinary-token-negative-p1",
      "family": "arena-leader-predicate-negative",
      "focusCardId": "ASH_240",
      "units": [
        {
          "key": "ordinaryGround",
          "code": "SEC_028",
          "owner": "own"
        },
        {
          "key": "ordinarySpace",
          "code": "JTL_212",
          "owner": "own"
        }
      ],
      "initialTokens": [
        {
          "key": "groundToken",
          "token": "battle-droid"
        },
        {
          "key": "spaceToken",
          "token": "tie-fighter"
        }
      ],
      "plan": [
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-paid-base-leader-ordinary-token-negative-p2",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "JTL_212",
      "SEC_028"
    ],
    "design": {
      "name": "ASH_240-paid-base-leader-ordinary-token-negative-p2",
      "family": "arena-leader-predicate-negative",
      "focusCardId": "ASH_240",
      "units": [
        {
          "key": "ordinaryGround",
          "code": "SEC_028",
          "owner": "own"
        },
        {
          "key": "ordinarySpace",
          "code": "JTL_212",
          "owner": "own"
        }
      ],
      "initialTokens": [
        {
          "key": "groundToken",
          "token": "battle-droid"
        },
        {
          "key": "spaceToken",
          "token": "tie-fighter"
        }
      ],
      "plan": [
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-paid-opposing-only-deployed-leader-negative-p1",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "SOR_014"
    ],
    "design": {
      "name": "ASH_240-paid-opposing-only-deployed-leader-negative-p1",
      "family": "arena-leader-controller-boundary",
      "focusCardId": "ASH_240",
      "enemyLeaderAlias": true,
      "units": [],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader",
          "actor": "enemy"
        },
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-paid-opposing-only-deployed-leader-negative-p2",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "SOR_014"
    ],
    "design": {
      "name": "ASH_240-paid-opposing-only-deployed-leader-negative-p2",
      "family": "arena-leader-controller-boundary",
      "focusCardId": "ASH_240",
      "enemyLeaderAlias": true,
      "units": [],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader",
          "actor": "enemy"
        },
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-actual-ground-deploy-then-defeat-live-update-p1",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "SOR_014",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_240-actual-ground-deploy-then-defeat-live-update-p1",
      "family": "arena-leader-live-deploy-loss",
      "focusCardId": "ASH_240",
      "units": [
        {
          "key": "subject",
          "code": "ASH_240",
          "owner": "own"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader"
        },
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_240-actual-ground-deploy-then-defeat-live-update-p2",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240",
      "SOR_014",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_240-actual-ground-deploy-then-defeat-live-update-p2",
      "family": "arena-leader-live-deploy-loss",
      "focusCardId": "ASH_240",
      "units": [
        {
          "key": "subject",
          "code": "ASH_240",
          "owner": "own"
        }
      ],
      "plan": [
        {
          "code": "SOR_014",
          "from": "leader"
        },
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-attack-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082",
      "JTL_212"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-attack-p1",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-attack-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082",
      "JTL_212"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-attack-p2",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-decline-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082",
      "JTL_212"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-decline-p1",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-decline-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082",
      "JTL_212"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-decline-p2",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        },
        {
          "key": "victim",
          "code": "JTL_212",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-no-space-target-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-no-space-target-p1",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-paid-ground-leader-ambush-no-space-target-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "HMW_082"
    ],
    "design": {
      "name": "ASH_113-paid-ground-leader-ambush-no-space-target-p2",
      "family": "arena-leader-paid-ambush",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "ground",
          "code": "HMW_082",
          "owner": "enemy"
        }
      ],
      "plan": [
        {
          "code": "ASH_113"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-paid-friendly-ground-leader-positive-p1",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240"
    ],
    "design": {
      "name": "ASH_240-paid-friendly-ground-leader-positive-p1",
      "family": "arena-leader-paid-entry",
      "focusCardId": "ASH_240",
      "deployedLeader": true,
      "units": [],
      "plan": [
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_240-paid-friendly-ground-leader-positive-p2",
    "focusCardId": "ASH_240",
    "setupCardIds": [
      "ASH_240"
    ],
    "design": {
      "name": "ASH_240-paid-friendly-ground-leader-positive-p2",
      "family": "arena-leader-paid-entry",
      "focusCardId": "ASH_240",
      "deployedLeader": true,
      "units": [],
      "plan": [
        {
          "code": "ASH_240"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_240"
      ],
      "activationKind": "paid-entry"
    }
  },
  {
    "name": "ASH_113-live-other-friendly-ground-space-token-count-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "ASH_124",
      "ASH_240",
      "SOR_077",
      "TWI_226"
    ],
    "design": {
      "name": "ASH_113-live-other-friendly-ground-space-token-count-p1",
      "family": "arena-leader-independent-mandalorian-count",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "groundMandalorian",
          "code": "ASH_240",
          "owner": "own"
        },
        {
          "key": "spaceMandalorian",
          "code": "ASH_124",
          "owner": "own"
        },
        {
          "key": "enemyMandalorian",
          "code": "ASH_240",
          "owner": "enemy"
        }
      ],
      "initialTokens": [
        {
          "key": "mandalorianToken",
          "token": "mandalorian"
        }
      ],
      "plan": [
        {
          "code": "SOR_077",
          "target": "groundMandalorian"
        },
        {
          "code": "TWI_226",
          "target": "spaceMandalorian"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_113-live-other-friendly-ground-space-token-count-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "ASH_124",
      "ASH_240",
      "SOR_077",
      "TWI_226"
    ],
    "design": {
      "name": "ASH_113-live-other-friendly-ground-space-token-count-p2",
      "family": "arena-leader-independent-mandalorian-count",
      "focusCardId": "ASH_113",
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "groundMandalorian",
          "code": "ASH_240",
          "owner": "own"
        },
        {
          "key": "spaceMandalorian",
          "code": "ASH_124",
          "owner": "own"
        },
        {
          "key": "enemyMandalorian",
          "code": "ASH_240",
          "owner": "enemy"
        }
      ],
      "initialTokens": [
        {
          "key": "mandalorianToken",
          "token": "mandalorian"
        }
      ],
      "plan": [
        {
          "code": "SOR_077",
          "target": "groundMandalorian"
        },
        {
          "code": "TWI_226",
          "target": "spaceMandalorian"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_113-leader-loss-keeps-other-mandalorian-power-p1",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "ASH_240",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_113-leader-loss-keeps-other-mandalorian-power-p1",
      "family": "arena-leader-count-independent-from-condition",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "other",
          "code": "ASH_240",
          "owner": "own"
        }
      ],
      "plan": [
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 0,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  },
  {
    "name": "ASH_113-leader-loss-keeps-other-mandalorian-power-p2",
    "focusCardId": "ASH_113",
    "setupCardIds": [
      "ASH_113",
      "ASH_240",
      "SOR_077"
    ],
    "design": {
      "name": "ASH_113-leader-loss-keeps-other-mandalorian-power-p2",
      "family": "arena-leader-count-independent-from-condition",
      "focusCardId": "ASH_113",
      "deployedLeader": true,
      "units": [
        {
          "key": "subject",
          "code": "ASH_113",
          "owner": "own"
        },
        {
          "key": "other",
          "code": "ASH_240",
          "owner": "own"
        }
      ],
      "plan": [
        {
          "code": "SOR_077",
          "target": "leader"
        }
      ],
      "ownSeat": 1,
      "baseDamage": 0,
      "actualScriptedCardIds": [
        "ASH_113"
      ],
      "activationKind": "staged-live-constant-observation"
    }
  }
];
const normalize = e => ({ setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null });
function planLeaderPredicateProbes(entry) {
    if (entry.status !== 'supported' || entry.diagnostics?.length || !shapes.some(s => same(s, normalize(entry)))) return null;
    const selected = scenarios.filter(s => s.focusCardId === entry.setCode);
    const cases = selected.map(s => {
        const id = entry.setCode + ':arena-leader:' + s.name;
        return { id:'auto/'+entry.setCode+'/'+s.name.toLowerCase().replaceAll('_','-'), name:s.name, kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:s.name},focusCardIds:[entry.setCode],setupCardIds:s.setupCardIds,behaviorIds:['arena-leader-predicate','live-controller-scope'],assertionStages:['paid-entry-or-staged-live-constant','source-assertions','terminal-continuation'],coverageObligationIds:[id],probe:{family:'arena-leader-predicate',scenarioName:s.name,design:s.design} };
    });
    const obligations = cases.map(c=>({id:c.coverageObligationIds[0],path:'arena-leader',dimension:c.name}));
    const uncovered = [
        ['source-semantics','Finite cases do not prove completeness of source lowering'],
        ['printed-space-leader','Six engine-only space-leader observations are outside the admitted setup pool and excluded from differential evidence'],
        ['controller-capture-pilot-and-simultaneous-departure','No controller-transfer,piloted/captured-source or simultaneous leader-departure dependency-cycle construction'],
    ].map(([dimension,reason])=>{const id=entry.setCode+':arena-leader:'+dimension;obligations.push({id,path:'arena-leader',dimension});return{id,path:'arena-leader',reason};});
    return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports = {planLeaderPredicateProbes};
