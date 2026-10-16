'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "TWI_173",
  "sourcePath": "game/cards/03_TWI/events/BloodSport.ts",
  "sourceHash": "6c838c0824990e4b510ab824e48f65a467f4eae7ec4583b637d5d6c3b57d6ee4",
  "className": "BloodSport",
  "baseClass": "EventCard",
  "card": {
    "title": "Blood Sport",
    "subtitle": "",
    "cost": 3,
    "hp": null,
    "power": null,
    "text": "Deal 2 damage to each ground unit.",
    "deployBox": null,
    "epicAction": null,
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "2103133661",
    "aspects": [
      "aggression"
    ],
    "traits": [
      "fringe"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "TWI",
      "number": 173
    },
    "setCodes": [
      {
        "set": "TWI",
        "number": 173
      }
    ],
    "internalName": "blood-sport"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Deal 2 damage to each ground unit",
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
                    "kind": "const",
                    "value": "groundArena"
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "||",
                  "left": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "const",
                      "value": "groundArena"
                    },
                    "right": {
                      "kind": "const",
                      "value": "anyArena"
                    }
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
                }
              }
            },
            "controllerOrder": "seatOrder"
          },
          "amount": {
            "kind": "const",
            "value": 2
          }
        }
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": null,
  "piloting": null
};
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
const variants=[
  [
    "interleaved",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "LOF_119",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "JTL_212",
          "seat": 2,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "reverse-arrivals",
    {
      "targets": [
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "JTL_212",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "LOF_119",
          "seat": 1,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "only-p1",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "only-p2",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "empty-play",
    {
      "targets": [],
      "empty": true
    }
  ],
  [
    "space-only-play",
    {
      "targets": [
        {
          "code": "LOF_119",
          "seat": 1,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "JTL_212",
          "seat": 2,
          "exhausted": true
        }
      ],
      "empty": true
    }
  ],
  [
    "empty-cancel",
    {
      "targets": [],
      "empty": true,
      "cancel": true
    }
  ],
  [
    "shield-p1",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "shield-p2",
    {
      "targets": [
        {
          "code": "SEC_028",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "all-shielded",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        }
      ]
    }
  ],
  [
    "double-shields-source-first",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            },
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            },
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        }
      ],
      "priorityFirst": "source"
    }
  ],
  [
    "double-shields-other-first",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            },
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "HMW_082",
          "seat": 2,
          "exhausted": true,
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            },
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        }
      ],
      "priorityFirst": "other"
    }
  ],
  [
    "native-units",
    {
      "targets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "seat": 1,
          "exhausted": true,
          "token": "battle-droid"
        },
        {
          "code": "TOKEN_CLONE_TROOPER",
          "seat": 2,
          "exhausted": true,
          "token": "clone-trooper"
        },
        {
          "code": "TOKEN_CLONE_TROOPER",
          "seat": 1,
          "exhausted": true,
          "token": "clone-trooper"
        },
        {
          "code": "TOKEN_BATTLE_DROID",
          "seat": 2,
          "exhausted": true,
          "token": "battle-droid"
        },
        {
          "code": "TOKEN_XWING",
          "seat": 1,
          "exhausted": true,
          "token": "xwing",
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "TOKEN_TIE_FIGHTER",
          "seat": 2,
          "exhausted": true,
          "token": "tie-fighter"
        }
      ]
    }
  ],
  [
    "mixed-lethal",
    {
      "targets": [
        {
          "code": "SEC_028",
          "seat": 1,
          "exhausted": true,
          "damage": 2
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true,
          "damage": 3
        },
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "actual-deployed-leaders",
    {
      "targets": [
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "LOF_119",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "JTL_212",
          "seat": 1,
          "exhausted": true
        }
      ],
      "deploySeats": [
        2,
        1
      ]
    }
  ],
  [
    "lethal-deployed-leaders",
    {
      "targets": [
        {
          "code": "SOR_014",
          "seat": 2,
          "exhausted": true,
          "deployedLeader": true,
          "damage": 3
        },
        {
          "code": "HMW_082",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SOR_014",
          "seat": 1,
          "exhausted": true,
          "deployedLeader": true,
          "damage": 3
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        }
      ]
    }
  ],
  [
    "death-births-source-first",
    {
      "targets": [
        {
          "code": "TWI_032",
          "seat": 1,
          "exhausted": true,
          "damage": 1
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "TWI_032",
          "seat": 2,
          "exhausted": true,
          "damage": 1
        },
        {
          "code": "LOF_119",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "JTL_212",
          "seat": 2,
          "exhausted": true
        }
      ],
      "deathBirths": true,
      "priorityFirst": "source"
    }
  ],
  [
    "death-births-other-first",
    {
      "targets": [
        {
          "code": "TWI_032",
          "seat": 1,
          "exhausted": true,
          "damage": 1
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "TWI_032",
          "seat": 2,
          "exhausted": true,
          "damage": 1
        },
        {
          "code": "LOF_119",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "JTL_212",
          "seat": 2,
          "exhausted": true
        }
      ],
      "deathBirths": true,
      "priorityFirst": "other"
    }
  ],
  [
    "capacity-34",
    {
      "targets": [
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_008",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "IBH_055",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "LOF_112",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_028",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "SEC_044",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_044",
          "seat": 2,
          "exhausted": true
        },
        {
          "code": "SEC_044",
          "seat": 1,
          "exhausted": true
        },
        {
          "code": "SEC_044",
          "seat": 2,
          "exhausted": true
        }
      ],
      "expectedGroundCount": 34
    }
  ]
];

function planOrderedGroundEventDamageProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(let index=0;index<variants.length;index++){const [variant,fields]=variants[index],initiativeSeat=index%2?3-sourceSeat:sourceSeat,name='ordered-ground-'+variant+'-source'+sourceSeat+'-initiative'+initiativeSeat,id=entry.setCode+':ordered-ground-event-damage:'+name;
  const setupCardIds=[...new Set(['SOR_014','ASH_023','TOKEN_SHIELD','TOKEN_BATTLE_DROID',...fields.targets.map(t=>t.code),...fields.targets.flatMap(t=>(t.attachments??[]).map(a=>a.code))])];
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['physical-seat-ordered-arena-union','shared-window-preventable-damage','shield-identities-and-priority','deferred-defeats-and-newborn-exclusion'],assertionStages:['actual-event-payment-or-cancel','captured-physical-union-order','exact-shield-and-damage-events','lethal-departure-and-trigger-order','newborn-identity-and-damage','terminal-continuation'],coverageObligationIds:[id],probe:{family:'ordered-ground-event-damage',cardId:entry.setCode,sourceSeat,initiativeSeat,...fields}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'ordered-ground-event-damage',dimension:c.name}));
 const uncovered=[['arbitrary-restrictions-replacements-and-controller-transfer','Exact physical-seat union, native prevention and selected death arrivals are constructed; arbitrary damage replacement/restriction, reentry, transfer and aura-driven departure combinations remain uncredited.'],['fatal-base-and-fatigue-death-interactions','This source only deals unit damage; fatal-base/deck-fatigue effects from other death donors are not included in the selected token-birth scenarios.'],['all-attachment-cleanup','Native Shield identities and deployed-leader departures are asserted; all ordinary/piloting attachment cleanup variants remain uncredited.'],['multiplayer-and-capacity','Two physical players and bounded native identities are exercised; multiplayer and global card-capacity arithmetic stay uncredited.']].map(([dimension,reason])=>{const id=entry.setCode+':ordered-ground-event-damage:'+dimension;obligations.push({id,path:'ordered-ground-event-damage',dimension});return{id,path:'ordered-ground-event-damage',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planOrderedGroundEventDamageProbes};
