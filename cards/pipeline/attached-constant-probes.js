"use strict";
const same = require("node:util").isDeepStrictEqual;
const keys = (v, allowed) => v && typeof v === "object" && !Array.isArray(v) && Object.keys(v).every(k => allowed.includes(k));
const strings = v => Array.isArray(v) && v.every(x => typeof x === "string");
const noPilot = e => e?.piloting == null && ["pilotText", "pilotBox", "pilotingBox"].every(k => e.card?.[k] == null);
const source = e => e?.status === "supported" && typeof e.sourcePath === "string" && e.sourcePath.length > 0 && /^[a-f0-9]{64}$/.test(e.sourceHash ?? "");
const shapes = {
  "parent-title-sentinel": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
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
            "kind": "binary",
            "op": "===",
            "left": {
              "kind": "ref",
              "path": [
                "source",
                "parentCard",
                "title"
              ]
            },
            "right": {
              "kind": "const",
              "value": "Luke Skywalker"
            }
          }
        }
      ]
    },
    "attachCondition": {
      "kind": "unary",
      "op": "!",
      "value": {
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
    "baseClass": "UpgradeCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "upgrade"
      ],
      "unique": true,
      "arena": null,
      "cost": 3,
      "hp": 3,
      "power": 3,
      "text": "Attach to a non-Vehicle unit.\nIf attached unit is Luke Skywalker, he gains Sentinel. (Enemy units in this arena must attack a Sentinel when they attack you.)",
      "deployBox": null,
      "epicAction": "",
      "rules": null,
      "upgradePower": 3,
      "upgradeHp": 3,
      "keywords": [
        "sentinel"
      ],
      "traits": [
        "item",
        "weapon",
        "lightsaber"
      ],
      "aspects": [
        "vigilance",
        "heroism"
      ]
    },
    "variants": [
      "friendly-luke-sentinel",
      "enemy-luke-sentinel",
      "other-title-vehicle-negative",
      "actual-removal"
    ]
  },
  "parent-grit": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "gainKeyword",
            "target": "parent",
            "duration": "persistent",
            "keyword": "grit",
            "amount": {
              "kind": "const",
              "value": 1
            }
          },
          "attachedContext": "upgrade"
        }
      ]
    },
    "attachCondition": {
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
    "baseClass": "UpgradeCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "upgrade"
      ],
      "unique": false,
      "arena": null,
      "cost": 3,
      "hp": 0,
      "power": 0,
      "text": "Attach to a non-leader unit.\nAttached unit gains Grit.",
      "deployBox": null,
      "epicAction": "",
      "rules": null,
      "upgradePower": 0,
      "upgradeHp": 0,
      "keywords": [
        "grit"
      ],
      "traits": [
        "innate"
      ],
      "aspects": [
        "vigilance"
      ]
    },
    "variants": [
      "friendly-grit-combat",
      "enemy-grit-combat",
      "leader-attach-negative",
      "actual-detach-loss"
    ]
  },
  "parent-exhausted-sentinel": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
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
    "attachCondition": null,
    "baseClass": "UpgradeCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "upgrade"
      ],
      "unique": false,
      "arena": null,
      "cost": 3,
      "hp": 3,
      "power": 1,
      "text": "While attached unit is exhausted, it gains Sentinel. (Enemy units in this arena must attack a Sentinel when they attack you.)",
      "deployBox": null,
      "epicAction": "",
      "rules": null,
      "upgradePower": 1,
      "upgradeHp": 3,
      "keywords": [
        "sentinel"
      ],
      "traits": [
        "innate"
      ],
      "aspects": [
        "vigilance"
      ]
    },
    "variants": [
      "friendly-exhausted-sentinel",
      "enemy-exhausted-sentinel",
      "ready-exhaust-transition",
      "actual-detach-hp-loss"
    ]
  },
  "host-restore-condition": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "gainKeyword",
            "target": "self",
            "duration": "persistent",
            "keyword": "restore",
            "amount": {
              "kind": "const",
              "value": 2
            }
          },
          "condition": {
            "kind": "binary",
            "op": "||",
            "left": {
              "kind": "query",
              "name": "hasSomeTrait",
              "receiver": {
                "kind": "ref",
                "path": [
                  "source"
                ]
              },
              "args": [
                {
                  "kind": "const",
                  "value": "force"
                }
              ]
            },
            "right": {
              "kind": "binary",
              "op": "===",
              "left": {
                "kind": "ref",
                "path": [
                  "source",
                  "title"
                ]
              },
              "right": {
                "kind": "const",
                "value": "Sabine Wren"
              }
            }
          },
          "attachedContext": "host"
        }
      ]
    },
    "attachCondition": {
      "kind": "unary",
      "op": "!",
      "value": {
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
    "baseClass": "UpgradeCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "upgrade"
      ],
      "unique": true,
      "arena": null,
      "cost": 2,
      "hp": 2,
      "power": 2,
      "text": "Attach to a non-Vehicle unit.\nIf attached unit is Sabine Wren or a Force unit, it gains Restore 2.",
      "deployBox": null,
      "epicAction": "",
      "rules": null,
      "upgradePower": 2,
      "upgradeHp": 2,
      "keywords": [
        "restore"
      ],
      "traits": [
        "item",
        "weapon",
        "lightsaber"
      ],
      "aspects": [
        "command",
        "heroism"
      ]
    },
    "variants": [
      "friendly-force-restore",
      "enemy-sabine-restore",
      "neither-vehicle-negative",
      "actual-detach-loss"
    ]
  },
  "ready-host-aura": {
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "simultaneous",
            "effects": [
              {
                "kind": "gainKeyword",
                "target": "chosen",
                "duration": "persistent",
                "keyword": "overwhelm",
                "amount": {
                  "kind": "const",
                  "value": 1
                }
              },
              {
                "kind": "gainKeyword",
                "target": "chosen",
                "duration": "persistent",
                "keyword": "raid",
                "amount": {
                  "kind": "const",
                  "value": 1
                }
              },
              {
                "kind": "gainKeyword",
                "target": "chosen",
                "duration": "persistent",
                "keyword": "restore",
                "amount": {
                  "kind": "const",
                  "value": 1
                }
              }
            ]
          },
          "condition": {
            "kind": "unary",
            "op": "!",
            "value": {
              "kind": "ref",
              "path": [
                "source",
                "exhausted"
              ]
            }
          },
          "targetResolver": {
            "owner": "self",
            "type": "unit",
            "zone": "arena",
            "predicate": {
              "kind": "binary",
              "op": "&&",
              "left": {
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
              },
              "right": {
                "kind": "binary",
                "op": "===",
                "left": {
                  "kind": "ref",
                  "path": [
                    "card",
                    "controller"
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
          },
          "attachedContext": "host"
        }
      ]
    },
    "attachCondition": {
      "kind": "ref",
      "path": [
        "card",
        "unique"
      ]
    },
    "baseClass": "UpgradeCard",
    "keywordValues": {},
    "printed": {
      "types": [
        "upgrade"
      ],
      "unique": true,
      "arena": null,
      "cost": 3,
      "hp": 2,
      "power": 2,
      "text": "Attach to a <uq> unit.\nAttached unit gains: \u201cWhile this unit is ready, each other friendly unit gains Overwhelm, Raid 1, and Restore 1.\u201d",
      "deployBox": null,
      "epicAction": "",
      "rules": "Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n",
      "upgradePower": 2,
      "upgradeHp": 2,
      "keywords": [
        "overwhelm",
        "raid",
        "restore"
      ],
      "traits": [
        "innate"
      ],
      "aspects": [
        "command",
        "heroism"
      ]
    },
    "variants": [
      "friendly-ready-aura-combat",
      "enemy-ready-controller",
      "host-exhaust-ready-transition",
      "actual-removal"
    ]
  }
};
function classifyAttachedConstant(entry) {
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
    return {family: "attached-constant", mode, sourceHp: card.hp, sourcePower: card.power, sourceCost: card.cost,
        intrinsic: structuredClone(entry.keywordValues), titles: entry.abilities.card.map(a => a.title)};
}
function planAttachedConstantProbes(entry, options = {}) {
    const shape = classifyAttachedConstant(entry);
    if (!shape) return null;
    const catalogue = Array.isArray(options.catalogue) ? {cards: options.catalogue} : options.catalogue;
    const result = {schema: "swu-card-probes/v1", cardId: entry.setCode, sourceHash: entry.sourceHash, cases: [], obligations: [], uncovered: []};
    const obligation = (dimension, reason) => {
        const id = `${entry.setCode}:attached-constant:${dimension}`;
        result.obligations.push({id, path: "attached-constant", dimension});
        if (reason) result.uncovered.push({id, path: "attached-constant", reason});
        return id;
    };
    if (!catalogue?.cards) {obligation("catalogue-donors", "No full source/native catalogue supplied"); return result;}
    const donors = require("./arena-presence-probes").selectDonors(catalogue);
    if (!donors) {obligation("full-source-donors", "No complete bounded donor set"); return result;}
    const retained = shapes[shape.mode].variants.slice(0, 3);
    for (const variant of shapes[shape.mode].variants) if (!retained.includes(variant)) obligation(`deferred-${variant}`, "Source-derived boundary has no constructed case; no coverage credit");
    for (const variant of retained) for (const sourceSeat of [1, 2]) {
        const dimension = `${variant}-seat${sourceSeat}`, id = obligation(dimension);
        const probe = {...shape, cardId: entry.setCode, sourceSeat, variant, resourceCount: 24, donors};
        result.cases.push({id: `auto/${entry.setCode}/attached-constant-${dimension}`, name: `attached-constant-${dimension}`, kind: "probe",
            source: {module: "../pipeline/probe-fixture.js", scenario: `attached-constant-${dimension}`}, focusCardIds: [entry.setCode], setupCardIds: [...new Set(Object.values(donors).flat())].sort(),
            behaviorIds: ["attached-constant", "live-controller-scope", "actual-source-lifecycle"], assertionStages: ["construction", "activation", "observation"],
            coverageObligationIds: [id], probe});
    }
    for (const dimension of ["captured-source-departure-and-unique", "all-deck-fatigue-boundaries", "exotic-controller-and-replacement", "all-concurrent-modifier-priorities"])
        obligation(dimension, "No actual full source construction for this boundary; partial/primitive diagnostics remain separate evidence");
    return result;
}
module.exports = {classifyAttachedConstant, planAttachedConstantProbes, noPilot, source};
