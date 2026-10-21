'use strict';
const same = require('node:util').isDeepStrictEqual;
const keys = (value, allowed) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).every(key => allowed.includes(key));
const constant = value => ({kind:'const',value});
const mouseEffect = {
    kind:'phasePlayerCostEffect',target:'ownPlayer',duration:'phase',effects:[{
        kind:'costAdjuster',adjustment:'decrease',stage:'standard_0',cardTypeFilter:'unit',
        matchAbilityCosts:false,matchCardEffectResourcePayments:false,
        limit:{kind:'costAdjusterLimit',scope:'perPlayerPerGame',max:1,clonePerEffectApplication:true},
        match:{kind:'boundCallback',parameters:['adjustedCard'],timing:'costEligibility',value:{kind:'query',name:'hasSomeTrait',receiver:{kind:'ref',path:['adjustedCard']},args:[constant('imperial')]}},amount:constant(1),
    }],
};
function readyEffect(binding) {
    return {kind:'phasePlayerEffect',target:'ownPlayer',duration:'phase',effects:[{
        kind:'unitsEnterPlayReady',cardTypeFilter:'unit',
        match:{kind:'attackCallback',parameters:[{root:binding,role:'enteringUnit'}],body:{kind:'binary',op:'<=',left:{kind:'query',name:'getPrintedPower',receiver:{kind:'ref',path:[binding]},args:[]},right:constant(1)}},
        limit:{kind:'playerModifierLimit',scope:'perPlayerPerGame',max:1,clonePerApplication:true},
        entryType:'played',matchTiming:'beforeUnitEntry',limitIncrementTiming:'successfulMatcherApplication',
    }]};
}
function ordinary(entry) {
    const c=entry?.card;
    return entry?.status==='supported' && entry.baseClass==='NonLeaderUnitCard' && same(c?.types,['unit']) && c.arena==='ground' &&
        typeof entry.sourcePath==='string' && entry.sourcePath.length>0 && /^[a-f0-9]{64}$/.test(entry.sourceHash??'') &&
        Number.isInteger(c.cost) && c.cost>=0 && c.cost<=8 && Number.isInteger(c.power) && c.power>=0 && c.power<=32 &&
        Number.isInteger(c.hp) && c.hp>=1 && c.hp<=32 && typeof c.unique==='boolean' &&
        !entry.piloting && !entry.attachCondition && !c.pilotText && !c.pilotBox && !c.pilotingBox && !c.deployBox && !c.epicAction &&
        c.upgradePower==null && c.upgradeHp==null && keys(entry.abilities,['card','unit','leader']) && same(entry.abilities.unit,[]) && same(entry.abilities.leader,[]);
}
function ability(a,trigger,effect) {
    return keys(a,['trigger','title','costs','effect']) && a.trigger===trigger && typeof a.title==='string' && !!a.title.trim() && a.title.length<=256 && same(a.costs,[]) && same(a.effect,effect);
}
function classifyPlayerPhaseEffect(entry) {
    if(!ordinary(entry)) return null;
    const c=entry.card,all=entry.abilities.card;
    if(!Array.isArray(all))return null;
    if(!c.unique && c.cost===1 && c.hp===1 && c.power===1 && same(c.traits,['imperial','droid']) && same(c.aspects,['villainy']) && same(c.keywords,['raid']) && same(entry.keywordValues,{raid:1}) && all.length===1 && ability(all[0],'whenPlayed',mouseEffect))
        return {family:'player-phase-effect',kind:'mouse-cost'};
    if(!c.unique || c.cost!==1 || c.hp!==4 || c.power!==1 || !same(c.traits,['fringe']) || !same(c.aspects,['heroism']) || !same(c.keywords,[]) || !same(entry.keywordValues??{},{}) || all.length!==2) return null;
    // Only alpha-renaming the one bound entering-unit parameter is allowed.
    // A free ref, a captured source, or a different callback timing is not this matcher.
    const binding=all[0]?.effect?.effects?.[0]?.match?.parameters?.[0]?.root;
    if(typeof binding!=='string' || !/^[A-Za-z_][A-Za-z0-9_]{0,63}$/.test(binding))return null;
    const effect=readyEffect(binding);
    if(!ability(all[0],'whenPlayed',effect) || !ability(all[1],'onAttack',effect))return null;
    return {family:'player-phase-effect',kind:'neel-ready'};
}


// Complete reviewed construction envelopes, without card identity or display labels.
// These donors are setup dependencies; their presence does not admit the focus card.
const donorRecipes = {
  "mouseLeader": {
    "baseClass": "LeaderUnitCard",
    "metadata": {
      "types": [
        "leader"
      ],
      "arena": "ground",
      "cost": 6,
      "power": 3,
      "hp": 8,
      "unique": true,
      "aspects": [
        "aggression",
        "villainy"
      ],
      "traits": [
        "force",
        "imperial",
        "sith"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": "On Attack: Deal 2 damage to a base.",
      "epicAction": "Epic Action: If you control 6 or more resources, deploy this leader. (Flip him, ready him, and move him to the ground arena.)"
    },
    "keywordValues": {},
    "abilities": {
      "leader": [
        {
          "trigger": "action",
          "costs": [
            {
              "kind": "resources",
              "amount": {
                "kind": "const",
                "value": 1
              }
            },
            {
              "kind": "exhaustSelf"
            }
          ],
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "base"
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
      ],
      "unit": [
        {
          "trigger": "onAttack",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "base"
          },
          "effect": {
            "kind": "damage",
            "target": "chosen",
            "amount": {
              "kind": "const",
              "value": 2
            }
          }
        }
      ],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "neelLeader": {
    "baseClass": "LeaderUnitCard",
    "metadata": {
      "types": [
        "leader"
      ],
      "arena": "ground",
      "cost": 5,
      "power": 0,
      "hp": 8,
      "unique": true,
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "mandalorian",
        "official"
      ],
      "keywords": [
        "restore"
      ],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": "Restore 4",
      "epicAction": "Epic Action: If you control 5 or more resources, deploy this leader."
    },
    "keywordValues": {
      "restore": 4
    },
    "abilities": {
      "leader": [
        {
          "trigger": "action",
          "costs": [
            {
              "kind": "exhaustSelf"
            }
          ],
          "effect": {
            "kind": "distributeHealing",
            "amount": {
              "kind": "const",
              "value": 2
            },
            "maxTargets": 1,
            "canChooseNoTargets": true,
            "canDistributeLess": true,
            "targetResolver": {
              "owner": "any",
              "type": "unit",
              "zone": "anyAttackable"
            }
          },
          "ifYouDo": {
            "title": "Deal that much damage to your base",
            "costs": [],
            "effect": {
              "kind": "damage",
              "target": "ownBase",
              "amount": {
                "kind": "ref",
                "path": [
                  "event",
                  "totalDistributed"
                ]
              }
            }
          }
        }
      ],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "base": {
    "baseClass": null,
    "metadata": {
      "types": [
        "base"
      ],
      "cost": null,
      "power": null,
      "hp": 30,
      "unique": false,
      "aspects": [
        "command"
      ],
      "traits": [
        "death star"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "imperial": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "space",
      "cost": 2,
      "power": 2,
      "hp": 2,
      "unique": false,
      "aspects": [
        "villainy"
      ],
      "traits": [
        "imperial",
        "vehicle",
        "fighter"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "imperialZero": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 1,
      "power": 1,
      "hp": 3,
      "unique": false,
      "aspects": [
        "aggression",
        "villainy"
      ],
      "traits": [
        "imperial",
        "trooper"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "nonmatching": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 0,
      "power": 1,
      "hp": 1,
      "unique": false,
      "aspects": [],
      "traits": [
        "creature"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "ready": {
    "baseClass": "EventCard",
    "metadata": {
      "types": [
        "event"
      ],
      "cost": 2,
      "power": null,
      "hp": null,
      "unique": false,
      "aspects": [
        "aggression"
      ],
      "traits": [
        "tactic"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
          "costs": [],
          "targetResolver": {
            "owner": "any",
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
                  "kind": "query",
                  "name": "getPower",
                  "receiver": {
                    "kind": "ref",
                    "path": [
                      "card"
                    ]
                  },
                  "args": []
                },
                "right": {
                  "kind": "const",
                  "value": 3
                }
              }
            }
          },
          "effect": {
            "kind": "ready",
            "target": "chosen"
          }
        }
      ]
    },
    "attachCondition": null,
    "piloting": null
  },
  "killer": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 4,
      "power": 5,
      "hp": 5,
      "unique": false,
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "rebel",
        "trooper"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "printedOne": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 2,
      "power": 1,
      "hp": 5,
      "unique": false,
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "rebel"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "printedTwo": {
    "baseClass": null,
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 2,
      "power": 2,
      "hp": 4,
      "unique": false,
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "ewok"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  },
  "printedZero": {
    "baseClass": "NonLeaderUnitCard",
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 2,
      "power": 0,
      "hp": 5,
      "unique": false,
      "aspects": [
        "cunning"
      ],
      "traits": [
        "imperial",
        "official"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "action",
          "costs": [
            {
              "kind": "exhaustSelf"
            }
          ],
          "effect": {
            "kind": "selectionSystem",
            "system": "selectPlayer",
            "name": "target",
            "resolver": {
              "kind": "playerSelection",
              "mode": "player",
              "chooser": "self",
              "resultBinding": "target",
              "defaults": "pinnedTargetResolver",
              "contextBinding": "target",
              "choosingPlayerOverride": "contextOverrideFirst",
              "delayOpponentBeforeCosts": true,
              "resolverName": "target"
            },
            "innerSystem": {
              "kind": "exhaustResources",
              "target": {
                "kind": "ref",
                "path": [
                  "selectedPlayer"
                ]
              },
              "amount": {
                "kind": "const",
                "value": 1
              },
              "paymentType": "exhaustResources",
              "isCost": false,
              "allowCostAdjustments": false
            },
            "contract": "SelectPlayerSystem.selectedPlayerDefaultTarget",
            "legality": "innerSystemCanAffectPlayer"
          }
        }
      ]
    },
    "attachCondition": null,
    "piloting": null
  },
  "alreadyReady": {
    "baseClass": "NonLeaderUnitCard",
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 1,
      "power": 1,
      "hp": 3,
      "unique": true,
      "aspects": [
        "cunning",
        "heroism"
      ],
      "traits": [
        "force",
        "fringe"
      ],
      "keywords": [
        "raid"
      ],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": ""
    },
    "keywordValues": {
      "raid": 1
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "constantEffectRegistration",
            "properties": {
              "condition": {
                "kind": "attackCallback",
                "parameters": [
                  {
                    "root": "attackBinding1_0",
                    "role": "ongoingContext"
                  }
                ],
                "body": {
                  "kind": "binary",
                  "op": "===",
                  "left": {
                    "kind": "ref",
                    "path": [
                      "attackBinding1_0",
                      "game",
                      "roundNumber"
                    ]
                  },
                  "right": {
                    "kind": "const",
                    "value": 1
                  }
                }
              },
              "sourceZoneFilter": "any"
            },
            "modifier": {
              "kind": "ongoingContract",
              "scope": "card",
              "effectType": "entersPlayReady",
              "target": "constantMatch",
              "duration": "persistent",
              "valueMode": "static",
              "value": {
                "kind": "undefined"
              },
              "builder": "static"
            },
            "duration": "persistent",
            "context": "refreshFromSourceController",
            "scopeDefaults": "pinnedOngoingCardOrPlayerEffect",
            "dependencyPolicy": "liveFixedPointRequired"
          }
        }
      ]
    },
    "attachCondition": null,
    "piloting": null
  },
  "tokenCreator": {
    "baseClass": "EventCard",
    "metadata": {
      "types": [
        "event"
      ],
      "cost": 2,
      "power": null,
      "hp": null,
      "unique": false,
      "aspects": [
        "villainy"
      ],
      "traits": [
        "supply"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "event",
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
    "attachCondition": null,
    "piloting": null
  },
  "buff": {
    "baseClass": "NonLeaderUnitCard",
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 4,
      "power": 4,
      "hp": 4,
      "unique": true,
      "aspects": [
        "heroism"
      ],
      "traits": [
        "rebel",
        "official"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "chosen",
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
                    "value": "rebel"
                  }
                ]
              }
            }
          }
        }
      ]
    },
    "attachCondition": null,
    "piloting": null
  },
  "debuff": {
    "baseClass": "NonLeaderUnitCard",
    "metadata": {
      "types": [
        "unit"
      ],
      "arena": "ground",
      "cost": 8,
      "power": 6,
      "hp": 6,
      "unique": true,
      "aspects": [
        "villainy",
        "vigilance"
      ],
      "traits": [
        "force",
        "first order",
        "official"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
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
    "attachCondition": null,
    "piloting": null
  },
  "battleDroid": {
    "baseClass": "TokenUnitCard",
    "metadata": {
      "types": [
        "token",
        "unit"
      ],
      "arena": "ground",
      "cost": 0,
      "power": 1,
      "hp": 1,
      "unique": false,
      "aspects": [
        "villainy"
      ],
      "traits": [
        "separatist",
        "droid",
        "trooper"
      ],
      "keywords": [],
      "upgradePower": null,
      "upgradeHp": null,
      "deployBox": null,
      "epicAction": null
    },
    "keywordValues": {},
    "abilities": {
      "leader": [],
      "unit": [],
      "card": []
    },
    "attachCondition": null,
    "piloting": null
  }
};
function donorShape(entry) {
    const metadata={};
    for(const key of ['types','arena','cost','power','hp','unique','aspects','traits','keywords','upgradePower','upgradeHp','pilotText','pilotBox','pilotingBox','deployBox','epicAction'])if(entry.card?.[key]!==undefined)metadata[key]=entry.card[key];
    return {baseClass:entry.baseClass??null,metadata,keywordValues:entry.keywordValues??{},abilities:Object.fromEntries(Object.entries(entry.abilities??{}).map(([key,list])=>[key,Array.isArray(list)?list.map(({title,...rest})=>rest):list])),attachCondition:entry.attachCondition??null,piloting:entry.piloting??null};
}
function reviewedDonor(entry,role) {
    return !!entry && (entry.status==='supported'||role==='battleDroid'&&entry.status==='unsupported'&&entry.sourceHash==="dc0c56c80105d04930c35405295126f330c935c7cc56c7c6d78da044b129eb93") && Object.hasOwn(donorRecipes,role) && same(donorShape(entry),donorRecipes[role]);
}
function admittedDonor(entry,pool) {
    const rows=Array.isArray(pool)?pool:pool?.cards??[];
    const row=rows.find(row=>row.setCode===entry?.setCode);
    if(!row)return false;
    const {setCode,...metadata}=row;
    return same(metadata,entry.card);
}


// Small actual-card protocols proposed by the engine author. These parameters
// describe experiments; no case count or expectation establishes an oracle pass.
const templates = {
  "mouse-cost": [
    {
      "scenario": "matching",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "discount": 1,
            "activeCost": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "discount": 0,
            "activeCost": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "nonmatching-zero",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "nonmatch",
          "expect": {
            "discount": 0,
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "zero",
          "expect": {
            "discount": 1,
            "paid": 0,
            "activeCost": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "discount": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "sequential-mice",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "secondMouse",
          "expect": {
            "discount": 1,
            "paid": 0,
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "discount": 1,
            "activeCost": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "discount": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "source-death",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "attack",
          "card": "killer",
          "target": "source",
          "expect": {
            "sourceZone": "discard",
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "discount": 1,
            "activeCost": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "discount": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "phase-expiry",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "pass-phase"
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "discount": 0,
            "activeCost": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "raid-evaluation-cancel",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "evaluate",
          "card": "first",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "cancel-play",
          "card": "readyEvent",
          "expect": {
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "readyEvent",
          "expect": {
            "discount": 0,
            "activeCost": 1
          },
          "targets": [
            "source"
          ]
        },
        {
          "type": "attack",
          "card": "source",
          "target": "enemyBase",
          "expect": {
            "power": 2,
            "baseDamage": 2,
            "activeCost": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "discount": 1,
            "activeCost": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching standard adjusters consume at actual payment, including effective zero",
        "evaluation and prepayment event cancellation do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    }
  ],
  "neel-ready": [
    {
      "scenario": "printed-boundary",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "high",
          "expect": {
            "ready": false,
            "activeReady": 1,
            "printedPower": 2
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "printedPower": 1
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false,
            "activeReady": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "printed-zero",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "zero",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "printedPower": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false,
            "activeReady": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "wp-attack-stack",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "readyEvent",
          "expect": {
            "activeReady": 1
          },
          "targets": [
            "source"
          ]
        },
        {
          "type": "attack",
          "card": "source",
          "target": "enemyBase",
          "expect": {
            "power": 1,
            "baseDamage": 1,
            "activeReady": 2
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "allReadyUsed": 2
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false,
            "activeReady": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "source-death",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "attack",
          "card": "killer",
          "target": "source",
          "expect": {
            "sourceZone": "discard",
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "phase-expiry",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "pass-phase"
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": false,
            "activeReady": 0
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "buff-printed-one",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "printedPower": 1,
            "livePower": 2
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false,
            "livePower": 2
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "debuff-printed-two",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "high",
          "expect": {
            "ready": false,
            "activeReady": 1,
            "printedPower": 2,
            "livePower": 0
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "printedPower": 1,
            "livePower": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "already-ready",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "already",
          "expect": {
            "ready": true,
            "activeReady": 0,
            "printedPower": 1
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    },
    {
      "scenario": "token-deploy-exclusion",
      "steps": [
        {
          "type": "paid-play",
          "card": "source",
          "expect": {
            "activeReady": 1
          }
        },
        {
          "type": "paid-play",
          "card": "tokenEvent",
          "expect": {
            "activeReady": 1,
            "createdTokens": 2
          }
        },
        {
          "type": "deploy",
          "card": "leader",
          "expect": {
            "activeReady": 1,
            "printedPower": 0
          }
        },
        {
          "type": "paid-play",
          "card": "first",
          "expect": {
            "ready": true,
            "activeReady": 0
          }
        },
        {
          "type": "paid-play",
          "card": "follow",
          "expect": {
            "ready": false
          }
        }
      ],
      "expectedBoundaries": [
        "all matching printedPower<=1 Unit Played grants consume during entry event construction before preResolve",
        "already ready entry consumes; created token and deployed leader entries do not consume",
        "player effect survives source departure and expires at phase end"
      ]
    }
  ]
};
const donorRoles = {
    'mouse-cost':{leader:'mouseLeader',first:'imperial',follow:'imperial',zero:'imperialZero',nonmatch:'nonmatching',secondMouse:'focus',readyEvent:'ready',killer:'killer'},
    'neel-ready':{leader:'neelLeader',first:'printedOne',follow:'printedOne',zero:'printedZero',high:'printedTwo',readyEvent:'ready',killer:'killer',buff:'buff',debuff:'debuff',already:'alreadyReady',tokenEvent:'tokenCreator'},
};
function planPlayerPhaseEffectProbes(entry,options={}) {
    const shape=classifyPlayerPhaseEffect(entry);if(!shape)return null;
    const definitions=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const pool=options.runtimePool??require('../../compiler/generated/runtime-pool.json');
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(dimension,reason){const id=entry.setCode+':player-phase-effect:'+dimension;result.obligations.push({id,path:'player-phase-effect',dimension});if(reason)result.uncovered.push({id,path:'player-phase-effect',reason});return id;}
    const roleMap=donorRoles[shape.kind],selected={};
    for(const role of new Set([...Object.values(roleMap).filter(role=>role!=='focus'),'base',...(shape.kind==='neel-ready'?['battleDroid']:[])])){
        selected[role]=definitions.filter(e=>reviewedDonor(e,role)&&admittedDonor(e,pool)).sort((a,b)=>a.setCode.localeCompare(b.setCode))[0];
        if(!selected[role])obligation('construction-'+role,'Missing complete reviewed and currently runtime-admitted construction donor; no standard case or native admission inferred');
    }
    if(Object.values(selected).every(Boolean)) {
        const donors=Object.fromEntries(Object.entries(roleMap).map(([name,role])=>[name,role==='focus'?entry.setCode:selected[role].setCode]));
        for(const owner of [0,1])for(const template of templates[shape.kind]) {
            const short=shape.kind==='mouse-cost'?'mouse':'neel',name='player-phase-'+short+'-'+owner+'-'+template.scenario;
            const id=obligation('owner-'+owner+'-'+template.scenario);
            const probe={family:shape.family,kind:shape.kind,owner,sourceCode:entry.setCode,donors:{...donors},expectedBoundaries:structuredClone(template.expectedBoundaries),scenario:template.scenario,steps:structuredClone(template.steps)};
            result.cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,...Object.values(donors),selected.base.setCode,...(shape.kind==='neel-ready'?[selected.battleDroid.setCode]:[])])].sort(),behaviorIds:['player-phase-effect',shape.kind,'actual-payment-and-entry-boundaries'],assertionStages:['actual-paid-source-play','exact-legal-actions-and-prompts','physical-resource-payment-receipts','player-effect-identity-and-remaining-use',...(shape.kind==='neel-ready'?['entry-event-construction-before-pre-resolution']:['actual-resource-payment-commit']),...template.steps.map(step=>'step-'+step.type),'observation'],coverageObligationIds:[id],probe});
        }
    }
    for(const [dimension,reason]of [
        ['source-review','Closed raw plans and finite constructions require independent full pinned-source/helper review'],
        ['oracle-native-wasm','These are proposed source-derived experiments; actual engine and native/Wasm results must be recorded separately'],
        ['replacement-after-event-generation','No actual witness establishes cancellation or replacement after unit entry event construction; consumption timing must not be moved to resolution'],
        ['foreign-controller','No arbitrary owner/controller-divergence construction'],
        ...(shape.kind==='mouse-cost'?[
            ['two-active-discounts','Sequential paid Mice consume the prior Imperial discount before creating one fresh grant; no standard two-active-grant source construction established'],
            ['matching-unit-prepayment-cancel','Actual prepayment cancellation uses Keep Fighting event; matching Imperial unit cancellation before payment remains unwitnessed'],
            ['ability-and-effect-resource-payments','Source flags exclude these payment types; no actual companion construction witnesses both exclusions'],
        ]:[['blocked-before-entry-generation','No actual prevented-entry construction separates payment from absence of entry event generation']]),
    ])obligation(dimension,reason);
    return result;
}
module.exports={classifyPlayerPhaseEffect,planPlayerPhaseEffectProbes,reviewedDonor,admittedDonor};
