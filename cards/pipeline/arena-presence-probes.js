"use strict";
const same = require("node:util").isDeepStrictEqual;
const keys = (v, allowed) => v && typeof v === "object" && !Array.isArray(v) && Object.keys(v).every(k => allowed.includes(k));
const strings = v => Array.isArray(v) && v.every(x => typeof x === "string");
const noPilot = e => e?.piloting == null && ["pilotText", "pilotBox", "pilotingBox"].every(k => e.card?.[k] == null);
const source = e => e?.status === "supported" && typeof e.sourcePath === "string" && e.sourcePath.length > 0 && /^[a-f0-9]{64}$/.test(e.sourceHash ?? "");
const shapes = {
  "aspect-union-xp": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "costs": [],
          "effect": {
            "kind": "giveExperience",
            "target": "self",
            "amount": {
              "kind": "binary",
              "op": "+",
              "left": {
                "kind": "binary",
                "op": "+",
                "left": {
                  "kind": "binary",
                  "op": "+",
                  "left": {
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
                                "value": "vigilance"
                              }
                            ]
                          }
                        }
                      ]
                    },
                    "then": {
                      "kind": "const",
                      "value": 1
                    },
                    "else": {
                      "kind": "const",
                      "value": 0
                    }
                  },
                  "right": {
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
                                "value": "command"
                              }
                            ]
                          }
                        }
                      ]
                    },
                    "then": {
                      "kind": "const",
                      "value": 1
                    },
                    "else": {
                      "kind": "const",
                      "value": 0
                    }
                  }
                },
                "right": {
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
                              "value": "aggression"
                            }
                          ]
                        }
                      }
                    ]
                  },
                  "then": {
                    "kind": "const",
                    "value": 1
                  },
                  "else": {
                    "kind": "const",
                    "value": 0
                  }
                }
              },
              "right": {
                "kind": "binary",
                "op": "+",
                "left": {
                  "kind": "binary",
                  "op": "+",
                  "left": {
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
                        }
                      ]
                    },
                    "then": {
                      "kind": "const",
                      "value": 1
                    },
                    "else": {
                      "kind": "const",
                      "value": 0
                    }
                  },
                  "right": {
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
                                "value": "heroism"
                              }
                            ]
                          }
                        }
                      ]
                    },
                    "then": {
                      "kind": "const",
                      "value": 1
                    },
                    "else": {
                      "kind": "const",
                      "value": 0
                    }
                  }
                },
                "right": {
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
                              "value": "villainy"
                            }
                          ]
                        }
                      }
                    ]
                  },
                  "then": {
                    "kind": "const",
                    "value": 1
                  },
                  "else": {
                    "kind": "const",
                    "value": 0
                  }
                }
              }
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "space",
      "cost": 4,
      "hp": 1,
      "power": 1,
      "text": "When Played: Give an Experience token to this unit for each different aspect among units you control.",
      "deployBox": null,
      "epicAction": "",
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [],
      "traits": [
        "vehicle",
        "transport"
      ],
      "aspects": [
        "command",
        "heroism"
      ]
    },
    "variants": [
      "source-self-union",
      "duplicate-icons-enemy-exclusion",
      "six-aspect-union",
      "real-deployed-token"
    ]
  },
  "base-threshold-draw": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenDefeated",
          "costs": [],
          "effect": {
            "kind": "draw",
            "target": "ownPlayer",
            "amount": {
              "kind": "binary",
              "op": "+",
              "left": {
                "kind": "conditional",
                "condition": {
                  "kind": "binary",
                  "op": ">=",
                  "left": {
                    "kind": "ref",
                    "path": [
                      "player",
                      "base",
                      "damage"
                    ]
                  },
                  "right": {
                    "kind": "const",
                    "value": 15
                  }
                },
                "then": {
                  "kind": "const",
                  "value": 1
                },
                "else": {
                  "kind": "const",
                  "value": 0
                }
              },
              "right": {
                "kind": "conditional",
                "condition": {
                  "kind": "binary",
                  "op": ">=",
                  "left": {
                    "kind": "ref",
                    "path": [
                      "opponent",
                      "base",
                      "damage"
                    ]
                  },
                  "right": {
                    "kind": "const",
                    "value": 15
                  }
                },
                "then": {
                  "kind": "const",
                  "value": 1
                },
                "else": {
                  "kind": "const",
                  "value": 0
                }
              }
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "unit"
      ],
      "unique": true,
      "arena": "ground",
      "cost": 2,
      "hp": 3,
      "power": 3,
      "text": "When Defeated: For each player with 15 or more damage on their base, draw a card.",
      "deployBox": null,
      "epicAction": null,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [],
      "traits": [
        "mandalorian",
        "trooper"
      ],
      "aspects": [
        "aggression",
        "aggression"
      ]
    },
    "variants": [
      "both-below",
      "own-threshold",
      "enemy-threshold",
      "both-threshold"
    ]
  },
  "force-card-stats": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "self",
            "duration": "persistent",
            "power": {
              "kind": "const",
              "value": 1
            },
            "hp": {
              "kind": "const",
              "value": 1
            }
          },
          "condition": {
            "kind": "query",
            "name": "hasSomeArenaCard",
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
                }
              }
            ]
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "space",
      "cost": 2,
      "hp": 3,
      "power": 1,
      "text": "While you control a Force unit or a Force upgrade, this unit gets +1/+1.",
      "deployBox": null,
      "epicAction": null,
      "rules": "Padawan Starfighter gets +1/+1 while you control a Force unit or a Force upgrade.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [],
      "traits": [
        "jedi",
        "republic",
        "vehicle",
        "fighter"
      ],
      "aspects": [
        "vigilance"
      ]
    },
    "variants": [
      "own-force-unit",
      "own-upgrade-enemy-host",
      "enemy-upgrade-enemy-host",
      "actual-removal-lethal"
    ]
  },
  "sith-leader-ready": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "costs": [],
          "optional": true,
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
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
                          "value": "sith"
                        }
                      ]
                    }
                  },
                  "right": {
                    "kind": "binary",
                    "op": "&&",
                    "left": {
                      "kind": "query",
                      "name": "isLeader",
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
                      "name": "isDeployed",
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
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {
      "overwhelm": 1
    },
    "printed": {
      "types": [
        "unit"
      ],
      "unique": true,
      "arena": "ground",
      "cost": 5,
      "hp": 7,
      "power": 4,
      "text": "Overwhelm\nWhen Played: If you control a Sith leader unit, you may ready this unit.",
      "deployBox": null,
      "epicAction": "",
      "rules": "If the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\nShields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [
        "overwhelm"
      ],
      "traits": [
        "force",
        "sith"
      ],
      "aspects": [
        "villainy"
      ]
    },
    "variants": [
      "own-sith-accept",
      "own-sith-decline",
      "front-and-enemy-negative",
      "own-nonsith-negative"
    ]
  },
  "leader-draw-grit": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "costs": [],
          "optional": true,
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
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
                      "name": "isLeader",
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
                      "name": "isDeployed",
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
            },
            "onTrue": {
              "kind": "draw",
              "target": "ownPlayer",
              "amount": {
                "kind": "const",
                "value": 1
              }
            },
            "onFalse": {
              "kind": "noAction"
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {
      "grit": 1
    },
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "ground",
      "cost": 5,
      "hp": 5,
      "power": 3,
      "text": "Grit (This unit gets +1/+0 for each damage on it.)\n\nOn Attack: If you control a leader unit, you may draw a card.",
      "deployBox": null,
      "epicAction": null,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [
        "grit"
      ],
      "traits": [
        "fringe"
      ],
      "aspects": [
        "vigilance"
      ]
    },
    "variants": [
      "own-leader-draw-grit",
      "own-leader-decline",
      "own-front-negative",
      "enemy-leader-negative"
    ]
  },
  "leader-buff-overwhelm": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "costs": [],
          "targetResolver": {
            "owner": "self",
            "type": "unit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
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
                      "name": "isLeader",
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
                      "name": "isDeployed",
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
            },
            "onTrue": {
              "kind": "modifyStats",
              "target": "chosen",
              "duration": "phase",
              "power": {
                "kind": "const",
                "value": 2
              },
              "hp": {
                "kind": "const",
                "value": 2
              }
            },
            "onFalse": {
              "kind": "noAction"
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {
      "overwhelm": 1
    },
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "ground",
      "cost": 5,
      "hp": 5,
      "power": 5,
      "text": "Overwhelm (When attacking an enemy unit, deal excess damage to the opponent's base.)\n\nOn Attack: If you control a leader unit, give a friendly unit +2/+2 for this phase.",
      "deployBox": null,
      "epicAction": null,
      "rules": "Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [
        "overwhelm"
      ],
      "traits": [
        "trooper"
      ],
      "aspects": [
        "command"
      ]
    },
    "variants": [
      "own-leader-self-overwhelm",
      "own-leader-other-arena",
      "own-front-negative",
      "enemy-leader-negative"
    ]
  },
  "leader-damage-saboteur": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "anyAttackable",
            "predicate": {
              "kind": "binary",
              "op": "||",
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
                  "kind": "binary",
                  "op": "===",
                  "left": {
                    "kind": "ref",
                    "path": [
                      "card",
                      "zoneName"
                    ]
                  },
                  "right": {
                    "kind": "const",
                    "value": "groundArena"
                  }
                }
              },
              "right": {
                "kind": "query",
                "name": "isBase",
                "receiver": {
                  "kind": "ref",
                  "path": [
                    "card"
                  ]
                },
                "args": []
              }
            }
          },
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
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
                      "name": "isLeader",
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
                      "name": "isDeployed",
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
            },
            "onTrue": {
              "kind": "damage",
              "target": "chosen",
              "amount": {
                "kind": "const",
                "value": 2
              }
            },
            "onFalse": {
              "kind": "noAction"
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {
      "saboteur": 1
    },
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "ground",
      "cost": 2,
      "hp": 1,
      "power": 2,
      "text": "Saboteur (When this unit attacks, ignore Sentinel and defeat the defender's Shields.)\n\nOn Attack: If you control a leader unit, deal 2 damage to a ground unit or a base.",
      "deployBox": null,
      "epicAction": null,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [
        "saboteur"
      ],
      "traits": [
        "rebel"
      ],
      "aspects": [
        "aggression"
      ]
    },
    "variants": [
      "own-leader-enemy-base",
      "own-leader-ground-target",
      "own-front-negative",
      "enemy-leader-negative"
    ]
  },
  "leader-exhaust-raid": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "costs": [],
          "optional": true,
          "targetResolver": {
            "owner": "any",
            "type": "nonLeaderUnit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
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
                      "name": "isLeader",
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
                      "name": "isDeployed",
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
    "attachCondition": null,
    "baseClass": "NonLeaderUnitCard",
    "keywordValues": {
      "raid": 1
    },
    "printed": {
      "types": [
        "unit"
      ],
      "unique": false,
      "arena": "space",
      "cost": 2,
      "hp": 3,
      "power": 1,
      "text": "Raid 1 (This unit gets +1/+0 while attacking.)\n\nOn Attack: If you control a leader unit, you may exhaust a non-leader unit.",
      "deployBox": null,
      "epicAction": null,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "keywords": [
        "raid"
      ],
      "traits": [
        "fringe",
        "vehicle",
        "fighter"
      ],
      "aspects": [
        "cunning"
      ]
    },
    "variants": [
      "own-leader-exhaust",
      "own-leader-decline",
      "own-front-negative",
      "enemy-leader-negative"
    ]
  }
};
function classifyArenaPresence(entry) {
    const card = entry?.card;
    if (!source(entry) || !card || !noPilot(entry) || !strings(card.traits) || !strings(card.aspects)
        || !strings(card.keywords) || typeof card.unique !== "boolean" || !keys(entry.abilities, ["leader", "unit", "card"])) return null;
    const abilities = structuredClone(entry.abilities);
    for (const side of ["leader", "unit", "card"]) {
        if (!Array.isArray(abilities[side])) return null;
        for (const ability of abilities[side]) {
            if (!ability || typeof ability.title !== "string" || !ability.title.length || Buffer.byteLength(ability.title) > 1024) return null;
            delete ability.title;
        }
    }
    const mode = Object.keys(shapes).find(name => {
        const shape = shapes[name];
        return entry.baseClass === shape.baseClass && same(abilities, shape.abilities)
            && same(entry.attachCondition ?? null, shape.attachCondition) && same(entry.keywordValues, shape.keywordValues)
            && Object.entries(shape.printed).every(([key, value]) => same(card[key] ?? null, value));
    });
    if (!mode) return null;
    return {family: "arena-presence", mode, sourceHp: card.hp, sourcePower: card.power, sourceCost: card.cost,
        intrinsic: structuredClone(entry.keywordValues), titles: entry.abilities.card.map(a => a.title)};
}
function selectDonors(catalogue) {
    const defs = catalogue.cards.slice().sort((a, b) => a.setCode.localeCompare(b.setCode));
    const quiet = require("./arena-query-probes").quiet;
    const base = require("./base-zone-probes");
    const first = predicate => defs.find(predicate);
    const ground = first(e => quiet(e) && e.card.arena === "ground" && e.card.power === 1 && e.card.hp >= 5);
    const space = first(e => quiet(e) && e.card.arena === "space" && e.card.power === 1 && e.card.hp >= 3);
    const force = first(e => quiet(e) && e.card.traits.includes("force") && e.card.power === 1 && e.card.hp >= 4);
    const unique = first(e => quiet(e, true) && e.card.arena === "ground" && e.card.hp >= 7);
    const doubled = first(e => quiet(e, true) && e.card.aspects.length === 2 && e.card.aspects[0] === e.card.aspects[1] && e.card.aspects[0] === "aggression");
    const leader = first(base.ordinaryLeader);
    const sith = first(e => require("./base-zone-probes").classifyBaseZone(e)?.mode === "leader-raid" && e.card.traits.includes("sith"));
    const fire = first(e => base.damageEvent(e, 4));
    const small = first(e => base.damageEvent(e, 2));
    const returning = first(e => {
        const shape = require("./upgrade-return-probes").classifyUpgradeReturn(e);
        return shape && !shape.sourceUpgrade && !shape.conditional && !shape.nonUnique && !shape.discard && shape.threshold === 3
            && e.baseClass === "NonLeaderUnitCard" && same(e.card.keywords, []) && same(e.keywordValues, {});
    });
    const forceUpgrade = first(e => source(e) && noPilot(e) && e.baseClass === "UpgradeCard" && same(e.card.types, ["upgrade"])
        && e.card.unique === false && e.card.cost === 1 && e.card.upgradePower === 1 && e.card.upgradeHp === 3
        && same(e.card.traits, ["force"]) && same(e.card.keywords, []) && same(e.keywordValues, {})
        && same(e.abilities, {leader: [], unit: [], card: []}) && same(e.attachCondition,
            {kind: "query", name: "hasSomeTrait", receiver: {kind: "ref", path: ["card"]}, args: [{kind: "const", value: "force"}]}));
    const luke = first(e => source(e) && noPilot(e) && e.baseClass === "NonLeaderUnitCard" && same(e.card.types, ["unit"])
        && e.card.title === "Luke Skywalker" && e.card.traits.includes("force") && !e.card.traits.includes("vehicle")
        && e.card.unique === true && e.card.hp === 6 && e.card.power === 6 && e.card.arena === "ground"
        && same(e.keywordValues, {restore: 2}) && same(e.card.keywords, ["restore"]) && e.attachCondition == null
        && same(e.abilities.leader, []) && same(e.abilities.unit, []) && e.abilities.card.length === 1
        && (() => {const ability = e.abilities.card[0]; return keys(ability, ["trigger", "title", "costs", "optional", "targetResolver", "effect"])
            && ability.trigger === "whenPlayed" && ability.optional === true && same(ability.costs, [])
            && same(ability.targetResolver, {owner: "any", type: "unit", zone: "arena", arena: "ground"})
            && same(ability.effect, {kind: "damage", target: "chosen", amount: {kind: "const", value: 3}});})());
    const xp = first(require("./arena-token-probes").nativeExperience);
    const shield = first(require("./arena-damage-probes").nativeShield);
    const beast = first(require("./weakness-continuation-probes").nativeBeast);
    const aspectUnits = ["vigilance", "command", "aggression", "cunning", "heroism", "villainy"].map(aspect => first(e => quiet(e) && e.card.aspects.includes(aspect)));
    const selected = {ground, space, force, unique, doubled, leader, sith, fire, small, returning, forceUpgrade, luke, xp, shield, beast};
    if (Object.values(selected).some(e => !e) || aspectUnits.some(e => !e)) return null;
    return {...Object.fromEntries(Object.entries(selected).map(([role, e]) => [role, e.setCode])), aspectUnits: [...new Set(aspectUnits.map(e => e.setCode))]};
}

function planArenaPresenceProbes(entry, options = {}) {
    const shape = classifyArenaPresence(entry);
    if (!shape) return null;
    const catalogue = Array.isArray(options.catalogue) ? {cards: options.catalogue} : options.catalogue;
    const result = {schema: "swu-card-probes/v1", cardId: entry.setCode, sourceHash: entry.sourceHash, cases: [], obligations: [], uncovered: []};
    const obligation = (dimension, reason) => {
        const id = `${entry.setCode}:arena-presence:${dimension}`;
        result.obligations.push({id, path: "arena-presence", dimension});
        if (reason) result.uncovered.push({id, path: "arena-presence", reason});
        return id;
    };
    if (!catalogue?.cards) {obligation("catalogue-donors", "No full source/native catalogue supplied"); return result;}
    const donors = selectDonors(catalogue);
    if (!donors) {obligation("full-source-donors", "No complete bounded donor set"); return result;}
    const retained = {
        "aspect-union-xp": ["source-self-union", "duplicate-icons-enemy-exclusion", "six-aspect-union"],
        "base-threshold-draw": shapes[shape.mode].variants,
        "force-card-stats": shapes[shape.mode].variants,
        "sith-leader-ready": shapes[shape.mode].variants,
        "leader-draw-grit": ["own-leader-draw-grit", "own-leader-decline", "own-front-negative"],
        "leader-buff-overwhelm": ["own-leader-self-overwhelm", "own-front-negative"],
        "leader-damage-saboteur": ["own-leader-enemy-base", "own-front-negative"],
        "leader-exhaust-raid": ["own-leader-exhaust", "own-leader-decline", "own-front-negative"]
    }[shape.mode];
    for (const variant of shapes[shape.mode].variants) if (!retained.includes(variant)) obligation(`deferred-${variant}`, "Source-derived boundary has no constructed case; no coverage credit");
    for (const variant of retained) for (const sourceSeat of [1, 2]) {
        const dimension = `${variant}-seat${sourceSeat}`, id = obligation(dimension);
        const probe = {...shape, cardId: entry.setCode, sourceSeat, variant, resourceCount: 24, donors};
        result.cases.push({id: `auto/${entry.setCode}/arena-presence-${dimension}`, name: `arena-presence-${dimension}`, kind: "probe",
            source: {module: "../pipeline/probe-fixture.js", scenario: `arena-presence-${dimension}`}, focusCardIds: [entry.setCode], setupCardIds: [...new Set(Object.values(donors).flat())].sort(),
            behaviorIds: ["arena-presence", "live-controller-scope", "actual-source-lifecycle"], assertionStages: ["construction", "activation", "observation"],
            coverageObligationIds: [id], probe});
    }
    for (const dimension of ["captured-source-departure-and-unique", "all-deck-fatigue-boundaries", "exotic-controller-and-replacement", "all-concurrent-modifier-priorities"])
        obligation(dimension, "No actual full source construction for this boundary; partial/primitive diagnostics remain separate evidence");
    return result;
}
module.exports = {classifyArenaPresence, planArenaPresenceProbes, noPilot, source, selectDonors};
