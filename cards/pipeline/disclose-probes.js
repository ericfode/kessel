'use strict';
const same = require('node:util').isDeepStrictEqual;
const {source, noPilot} = require('./arena-presence-probes');
const shapes = {
  "death-xp": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenDefeated",
          "title": "Disclose Vigilance to give an Experience token to a unit",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "vigilance"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Give an Experience token to a unit",
            "costs": [],
            "targetResolver": {
              "owner": "any",
              "type": "unit",
              "zone": "anyAttackable"
            },
            "effect": {
              "kind": "giveExperience",
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
    "printed": {
      "cost": 2,
      "hp": 2,
      "power": 2,
      "text": "When Defeated: You may disclose Vigilance (reveal a card from your hand with this aspect icon). If you do, give an Experience token to a unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "republic",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ]
    }
  },
  "play-draw": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Disclose Vigilance to draw a card",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "vigilance"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Draw a card",
            "costs": [],
            "effect": {
              "kind": "draw",
              "target": "ownPlayer",
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
    "printed": {
      "cost": 4,
      "hp": 4,
      "power": 3,
      "text": "When Played: You may disclose Vigilance (reveal a card from your hand with this aspect icon). If you do, draw a card.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "fringe",
        "vehicle",
        "transport"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ]
    }
  },
  "event-defeat": {
    "baseClass": "EventCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Disclose Vigilance, Vigilance. If you do, defeat a damaged non-leader unit",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "vigilance",
              "vigilance"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Defeat a damaged non-leader unit",
            "costs": [],
            "targetResolver": {
              "owner": "any",
              "type": "nonLeaderUnit",
              "zone": "anyAttackable",
              "predicate": {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "query",
                  "name": "isNonLeaderUnit",
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
              "kind": "defeat",
              "target": "chosen"
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 4,
      "hp": null,
      "power": null,
      "text": "You may disclose VigilanceVigilance (reveal cards from your hand with these aspect icons among them). If you do, defeat a damaged non-leader unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "law"
      ],
      "keywords": [],
      "types": [
        "event"
      ]
    }
  },
  "death-draw": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenDefeated",
          "title": "Disclose Command, Command, Heroism to draw a card",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "command",
              "command",
              "heroism"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Draw a card",
            "costs": [],
            "effect": {
              "kind": "draw",
              "target": "ownPlayer",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
          }
        }
      ]
    },
    "keywordValues": {
      "restore": 1
    },
    "printed": {
      "cost": 2,
      "hp": 4,
      "power": 2,
      "text": "Restore 1 (When this unit attacks, heal 1 damage from your base.)\nWhen Defeated: You may disclose CommandCommandHeroism (reveal cards from your hand with these aspect icons among them). If you do, draw a card.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "separatist",
        "official"
      ],
      "arena": "ground",
      "keywords": [
        "restore"
      ],
      "types": [
        "unit"
      ]
    }
  },
  "defense-base-heal": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onDefense",
          "title": "Disclose Command, Heroism to heal 1 damage from your base",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "command",
              "heroism"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Heal 1 damage from your base",
            "costs": [],
            "effect": {
              "kind": "heal",
              "target": "ownBase",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
          }
        }
      ]
    },
    "keywordValues": {
      "sentinel": 1
    },
    "printed": {
      "cost": 4,
      "hp": 5,
      "power": 4,
      "text": "Sentinel\nWhen this unit is attacked: You may disclose CommandHeroism. If you do, heal 1 damage from your base.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": "“When this unit is attacked” triggers at the same time as “On Attack” abilities.\nOn Defense abilities trigger when this unit is chosen as the defender for an attack, in the same window as the attacker's On Attack abilities.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "naboo",
        "republic",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [
        "sentinel"
      ],
      "types": [
        "unit"
      ]
    }
  },
  "play-death-sentinel": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Disclose Command to give a friendly unit Sentinel for this phase",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "command"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Give a friendly unit Sentinel for this phase",
            "costs": [],
            "targetResolver": {
              "owner": "self",
              "type": "unit",
              "zone": "anyAttackable"
            },
            "effect": {
              "kind": "gainKeyword",
              "target": "chosen",
              "duration": "phase",
              "keyword": "sentinel",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
          }
        },
        {
          "trigger": "whenDefeated",
          "title": "Disclose Command to give a friendly unit Sentinel for this phase",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "command"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Give a friendly unit Sentinel for this phase",
            "costs": [],
            "targetResolver": {
              "owner": "self",
              "type": "unit",
              "zone": "anyAttackable"
            },
            "effect": {
              "kind": "gainKeyword",
              "target": "chosen",
              "duration": "phase",
              "keyword": "sentinel",
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
    "printed": {
      "cost": 6,
      "hp": 7,
      "power": 5,
      "text": "When Played/When Defeated: You may disclose Command (reveal a card from your hand with this aspect icon). If you do, give a friendly unit Sentinel for this phase.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "command"
      ],
      "traits": [
        "naboo",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [
        "sentinel"
      ],
      "types": [
        "unit"
      ]
    }
  },
  "play-three-spies": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Disclose Aggression, Aggression, Villainy to create 3 Spy tokens ",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "aggression",
              "aggression",
              "villainy"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Create 3 Spy tokens",
            "costs": [],
            "effect": {
              "kind": "createTokenUnit",
              "tokenType": "spy",
              "amount": {
                "kind": "const",
                "value": 3
              },
              "target": "ownPlayer",
              "entersReady": false
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 7,
      "hp": 6,
      "power": 6,
      "text": "When Played: You may disclose AggressionAggressionVillainy (reveal cards from your hand with these aspect icons among them). If you do, create 3 Spy tokens.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "aggression",
        "villainy"
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
      ]
    }
  },
  "death-ready-spy": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenDefeated",
          "title": "Disclose Aggression, Heroism to create a Spy token and ready it",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "aggression",
              "heroism"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Create a Spy token and ready it",
            "costs": [],
            "effect": {
              "kind": "createTokenUnit",
              "tokenType": "spy",
              "amount": {
                "kind": "const",
                "value": 1
              },
              "target": "ownPlayer",
              "entersReady": true
            }
          }
        }
      ]
    },
    "keywordValues": {
      "hidden": 1
    },
    "printed": {
      "cost": 2,
      "hp": 2,
      "power": 3,
      "text": "Hidden\nWhen Defeated: You may disclose AggressionHeroism (reveal cards from your hand with these aspect icons among them). If you do, create a Spy token and ready it.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "rebel"
      ],
      "arena": "ground",
      "keywords": [
        "hidden"
      ],
      "types": [
        "unit"
      ]
    }
  },
  "death-discard": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenDefeated",
          "title": "Disclose Aggression, Aggression, Heroism to make your opponent discards 2 cards from their hand",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "aggression",
              "aggression",
              "heroism"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Your opponent discards 2 cards from their hand",
            "costs": [],
            "effect": {
              "kind": "discardOwnHand",
              "amount": {
                "kind": "const",
                "value": 2
              },
              "target": "enemyPlayer"
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 5,
      "hp": 3,
      "power": 5,
      "text": "When Defeated: You may choose an opponent and disclose AggressionAggressionHeroism (reveal cards from your hand with these aspect icons among them). If you do, that opponent discards 2 cards from their hand.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "rebel",
        "vehicle",
        "transport"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ]
    }
  },
  "event-two-spies": {
    "baseClass": "EventCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Create a Spy token. You may disclose Aggression. If you do, create another Spy token",
          "costs": [],
          "effect": {
            "kind": "createTokenUnit",
            "tokenType": "spy",
            "amount": {
              "kind": "const",
              "value": 1
            },
            "target": "ownPlayer",
            "entersReady": false
          },
          "then": {
            "title": "Disclose Aggression to create another Spy token",
            "costs": [],
            "effect": {
              "kind": "disclose",
              "target": "ownPlayer",
              "aspects": [
                "aggression"
              ],
              "mode": "all"
            },
            "ifYouDo": {
              "title": "Create another Spy token",
              "costs": [],
              "effect": {
                "kind": "createTokenUnit",
                "tokenType": "spy",
                "amount": {
                  "kind": "const",
                  "value": 1
                },
                "target": "ownPlayer",
                "entersReady": false
              }
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 3,
      "hp": null,
      "power": null,
      "text": "Create a Spy token.\nYou may disclose Aggression (reveal a card from your hand with this aspect icon). If you do, create another Spy token.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "aggression"
      ],
      "traits": [
        "plan"
      ],
      "keywords": [],
      "types": [
        "event"
      ]
    }
  },
  "event-damage": {
    "baseClass": "EventCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "title": "Disclose Aggression, Aggression. If you do, deal 5 damage to a unit",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "aggression",
              "aggression"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Deal 5 damage to a unit",
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
                "value": 5
              }
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 4,
      "hp": null,
      "power": null,
      "text": "You may disclose AggressionAggression (reveal cards from your hand with these aspect icons among them). If you do, deal 5 damage to a unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "aggression"
      ],
      "traits": [
        "law"
      ],
      "keywords": [],
      "types": [
        "event"
      ]
    }
  },
  "attack-ready-resources": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "Disclose Cunning, Cunning, Villainy to ready 2 resources",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "cunning",
              "cunning",
              "villainy"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Ready 2 resources",
            "costs": [],
            "effect": {
              "kind": "readyResources",
              "target": "ownPlayer",
              "amount": {
                "kind": "const",
                "value": 2
              }
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "printed": {
      "cost": 3,
      "hp": 3,
      "power": 3,
      "text": "On Attack: You may disclose CunningCunningVillainy (reveal cards from your hand with these aspect icons among them). If you do, ready 2 resources.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "cunning",
        "villainy"
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
      ]
    }
  },
  "attack-sentinel": {
    "baseClass": "NonLeaderUnitCard",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "onAttack",
          "title": "Disclose Heroism, Heroism to give a unit Sentinel for this phase",
          "costs": [],
          "effect": {
            "kind": "disclose",
            "target": "ownPlayer",
            "aspects": [
              "heroism",
              "heroism"
            ],
            "mode": "all"
          },
          "ifYouDo": {
            "title": "Give a unit Sentinel for this phase",
            "costs": [],
            "targetResolver": {
              "owner": "any",
              "type": "unit",
              "zone": "anyAttackable"
            },
            "effect": {
              "kind": "gainKeyword",
              "target": "chosen",
              "duration": "phase",
              "keyword": "sentinel",
              "amount": {
                "kind": "const",
                "value": 1
              }
            }
          }
        }
      ]
    },
    "keywordValues": {
      "restore": 1
    },
    "printed": {
      "cost": 1,
      "hp": 4,
      "power": 0,
      "text": "Restore 1\nOn Attack: You may disclose HeroismHeroism (reveal cards from your hand with these aspect icons among them). If you do, give a unit Sentinel for this phase.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "aspects": [
        "heroism"
      ],
      "traits": [
        "fringe",
        "droid"
      ],
      "arena": "ground",
      "keywords": [
        "restore",
        "sentinel"
      ],
      "types": [
        "unit"
      ]
    }
  }
};
function bindDisclosureContracts(value) {
    if (!value || typeof value !== 'object') return;
    if (value.kind === 'disclose') Object.assign(value, {
        aspectMultiplicity: true, selectedCardsEventField: 'disclosedCards',
        selectionContext: 'targetPlayer', triggeredNoMatchPolicy: 'maskWithPauseWithoutEvent'
    });
    for (const child of Object.values(value)) if (typeof child === 'object') bindDisclosureContracts(child);
}
for (const shape of Object.values(shapes)) bindDisclosureContracts(shape.abilities);
function classifyDisclose(entry) {
    if (!source(entry) || !noPilot(entry) || entry.attachCondition != null || !entry.card || !entry.abilities ||
        Object.keys(entry.abilities).some(k => !['leader', 'unit', 'card'].includes(k))) return null;
    const mode = Object.keys(shapes).find(mode => {
        const s = shapes[mode];
        return entry.baseClass === s.baseClass && same(entry.abilities, s.abilities) &&
            same(entry.keywordValues, s.keywordValues) &&
            Object.entries(s.printed).every(([key, value]) => same(entry.card[key] ?? null, value));
    });
    if (!mode) return null;
    const a = entry.abilities.card[0], disclose = a.effect.kind === 'disclose' ? a.effect : a.then.effect;
    return {family: 'disclose', mode, aspects: disclose.aspects.slice(), trigger: a.trigger,
        sourceArena: entry.card.arena, sourcePower: entry.card.power, sourceHp: entry.card.hp,
        keywords: {...entry.keywordValues}, sourceCost: entry.card.cost};
}
function selectDiscloseDonors(catalogue, entry, shape) {
    const defs = catalogue.cards.slice().sort((a, b) => a.setCode.localeCompare(b.setCode));
    const {quiet} = require('./arena-query-probes'), {ordinaryLeader, damageEvent} = require('./base-zone-probes');
    const units = defs.filter(e => quiet(e));
    const revealUnits = defs.filter(e => quiet(e) || quiet(e, true));
    const first = f => defs.find(f);
    const ground = units.find(e => e.card.arena === 'ground' && e.card.hp >= 7);
    const space = units.find(e => e.card.arena === 'space' && e.card.hp >= 7);
    const neutral = units.find(e => e.card.arena === 'ground' && !e.card.aspects.length && e.card.hp >= 3 && e.card.power === 1);
    const attacker = units.find(e => e.card.arena === (shape.sourceArena ?? 'ground') && e.card.power === 1);
    const singles = Object.fromEntries([...new Set(shape.aspects)].map(aspect => [aspect, units.find(e => same(e.card.aspects, [aspect]))]));
    const fire = first(e => damageEvent(e, 2));
    const defeat = first(e => source(e) && noPilot(e) && e.baseClass === 'EventCard' && same(e.card.types, ['event']) &&
        same(e.keywordValues, {}) && !e.card.keywords?.length && e.abilities.card?.length === 1 &&
        same(Object.keys(e.abilities).sort(), ['card','leader','unit']) && !e.abilities.leader.length && !e.abilities.unit.length &&
        Object.keys(e.abilities.card[0]).every(k => ['trigger','title','costs','targetResolver','effect'].includes(k)) &&
        e.abilities.card[0].trigger === 'event' && same(e.abilities.card[0].costs, []) &&
        same(e.abilities.card[0].effect, {kind:'defeat',target:'chosen'}) &&
        same(e.abilities.card[0].targetResolver, {owner:'any',type:'unit',zone:'anyAttackable'}));
    const leader = first(ordinaryLeader), spy = first(require('./arena-query-probes').nativeSpy);
    const xp = first(require('./arena-token-probes').nativeExperience);
    if ([ground,space,neutral,attacker,fire,defeat,leader,spy,xp,...Object.values(singles)].some(e => !e)) return null;
    // These selected cards are revealed only. A second copy of the real source
    // supplies the positive single-card case without inventing card metadata.
    const required = Object.fromEntries([...new Set(shape.aspects)].map(a => [a, shape.aspects.filter(x => x === a).length]));
    const single = revealUnits.find(e => Object.entries(required).every(([a,n]) => e.card.aspects.filter(x => x === a).length >= n));
    return {ground:ground.setCode, space:space.setCode, neutral:neutral.setCode, attacker:attacker.setCode,
        fire:fire.setCode, defeat:defeat.setCode, leader:leader.setCode, spy:spy.setCode, xp:xp.setCode,
        single:single?.setCode ?? null, singles:Object.fromEntries(Object.entries(singles).map(([a,e]) => [a,e.setCode]))};
}
function planDiscloseProbes(entry, options = {}) {
    const shape = classifyDisclose(entry);
    if (!shape) return null;
    const result = {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    const obligation = (dimension, reason) => {
        const id = entry.setCode + ':disclose:' + dimension;
        result.obligations.push({id,path:'disclose',dimension});
        if (reason) result.uncovered.push({id,path:'disclose',reason});
        return id;
    };
    if (shape.mode === 'attack-distributed-heal') {
        obligation('distribution-four', 'Four-point healing distribution is quarantined pending a complete independently asserted constructor.');
        return result;
    }
    const catalogue = Array.isArray(options.catalogue) ? {cards:options.catalogue} : options.catalogue;
    const donors = catalogue?.cards && selectDiscloseDonors(catalogue, entry, shape);
    if (!donors) {
        obligation('source-donor-contract', 'Complete pinned quiet unit, aspect, token and real activation donors unavailable');
        return result;
    }
    const variants = ['positive','decline','impossible'];
    if (shape.aspects.length > 1) variants.push('split-toggle');
    if (shape.mode === 'play-death-sentinel') variants.push('death-positive');
    for (const variant of variants) for (const sourceSeat of [1,2]) {
        const dimension = variant + '-seat' + sourceSeat, id = obligation(dimension);
        const probe = {...shape,cardId:entry.setCode,variant,sourceSeat,donors,resourceCount:24};
        const setup = Object.values(donors).flatMap(value => typeof value === 'string' ? [value] : value ? Object.values(value) : []);
        result.cases.push({id:'auto/'+entry.setCode+'/disclose-'+dimension,name:'disclose-'+dimension,kind:'probe',
            source:{module:'../pipeline/probe-fixture.js',scenario:'disclose-'+dimension},
            focusCardIds:[entry.setCode],setupCardIds:[...new Set(setup)].sort(),
            behaviorIds:['disclose-all','aspect-multiset','opponent-acknowledgement',shape.trigger],
            assertionStages:['construction','paid-source','hand-selection','opponent-acknowledgement','continuation'],
            coverageObligationIds:[id],probe});
    }
    for (const dimension of ['unique-source-departure','multi-owner-death-order','token-leader-recipients','replacement-cancels-reveal','fatigue-winner-latch','phase-expiry','all-selection-order-permutations']) {
        obligation(dimension, 'No complete source-card construction in this bounded family; separate diagnostics do not establish it.');
    }
    return result;
}
module.exports = {classifyDisclose,selectDiscloseDonors,planDiscloseProbes};
