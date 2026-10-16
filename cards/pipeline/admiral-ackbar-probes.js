'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "SOR_097",
  "sourcePath": "game/cards/01_SOR/units/AdmiralAckbarBrilliantStrategist.ts",
  "sourceHash": "c943cb25c9a7a5f6bef3e22877cca8362441d53abd2c9cc3fe9fb81b780dc1cd",
  "className": "AdmiralAckbarBrilliantStrategist",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Admiral Ackbar",
    "subtitle": "Brilliant Strategist",
    "cost": 3,
    "hp": 4,
    "power": 1,
    "text": "Restore 1 (When this unit attacks, heal 1 damage from your base.)\n\nWhen Played: You may deal damage to a unit equal to the number of units you control in its arena. ",
    "deployBox": null,
    "epicAction": null,
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "0827076106",
    "aspects": [
      "command",
      "heroism"
    ],
    "traits": [
      "rebel",
      "official"
    ],
    "arena": "ground",
    "keywords": [
      "restore"
    ],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "SOR",
      "number": 97
    },
    "setCodes": [
      {
        "set": "SOR",
        "number": 97
      }
    ],
    "internalName": "admiral-ackbar#brilliant-strategist"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Deal damage to a unit equal to the number of units your control in its arena.",
        "costs": [],
        "optional": true,
        "targetResolver": {
          "owner": "any",
          "type": "unit",
          "zone": "anyAttackable"
        },
        "effect": {
          "kind": "damage",
          "target": "chosen",
          "amount": {
            "kind": "query",
            "name": "arenaUnitCount",
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
                  "op": "||",
                  "left": {
                    "kind": "unary",
                    "op": "!",
                    "value": {
                      "kind": "ref",
                      "path": [
                        "target",
                        "zoneName"
                      ]
                    }
                  },
                  "right": {
                    "kind": "binary",
                    "op": "||",
                    "left": {
                      "kind": "binary",
                      "op": "===",
                      "left": {
                        "kind": "ref",
                        "path": [
                          "target",
                          "zoneName"
                        ]
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
                        "kind": "ref",
                        "path": [
                          "target",
                          "zoneName"
                        ]
                      }
                    }
                  }
                }
              }
            ]
          }
        }
      }
    ]
  },
  "keywordValues": {
    "restore": 1
  },
  "attachCondition": null,
  "piloting": null
};
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
const variants=[
  [
    "ground-token-leader-restore",
    {
      "target": "enemy",
      "expectedCount": 4,
      "deployOwn": true,
      "restore": true,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent"
        },
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        },
        {
          "key": "token",
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid"
        },
        {
          "key": "other",
          "code": "JTL_212",
          "owner": "self"
        },
        {
          "key": "excluded",
          "code": "IBH_007",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "space-native-tokens",
    {
      "target": "enemy",
      "expectedCount": 3,
      "units": [
        {
          "key": "enemy",
          "code": "LOF_119",
          "owner": "opponent"
        },
        {
          "key": "friendly",
          "code": "JTL_212",
          "owner": "self"
        },
        {
          "key": "xwing",
          "code": "TOKEN_XWING",
          "owner": "self",
          "token": "xwing"
        },
        {
          "key": "tie",
          "code": "TOKEN_TIE_FIGHTER",
          "owner": "self",
          "token": "tie-fighter"
        },
        {
          "key": "excluded",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "enemyGround",
          "code": "SEC_028",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "friendly-ground",
    {
      "target": "friendly",
      "expectedCount": 2,
      "units": [
        {
          "key": "friendly",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "excluded",
          "code": "JTL_212",
          "owner": "self"
        },
        {
          "key": "enemy",
          "code": "IBH_007",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "friendly-space",
    {
      "target": "friendly",
      "expectedCount": 1,
      "units": [
        {
          "key": "friendly",
          "code": "LOF_119",
          "owner": "self"
        },
        {
          "key": "excluded",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "enemy",
          "code": "JTL_212",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "self-only",
    {
      "target": "source",
      "expectedCount": 1,
      "units": []
    }
  ],
  [
    "mixed-zero-space",
    {
      "target": "enemy",
      "expectedCount": 0,
      "units": [
        {
          "key": "enemy",
          "code": "LOF_119",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "mixed-zero-shield",
    {
      "target": "enemy",
      "expectedCount": 0,
      "units": [
        {
          "key": "enemy",
          "code": "LOF_119",
          "owner": "opponent",
          "shield": true
        }
      ]
    }
  ],
  [
    "ground-shield",
    {
      "target": "enemy",
      "expectedCount": 2,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent",
          "shield": true
        },
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "space-shield",
    {
      "target": "enemy",
      "expectedCount": 1,
      "units": [
        {
          "key": "enemy",
          "code": "LOF_119",
          "owner": "opponent",
          "shield": true
        },
        {
          "key": "friendly",
          "code": "JTL_212",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "enemy-lethal",
    {
      "target": "enemy",
      "expectedCount": 4,
      "units": [
        {
          "key": "enemy",
          "code": "SEC_028",
          "owner": "opponent"
        },
        {
          "key": "one",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "two",
          "code": "IBH_007",
          "owner": "self"
        },
        {
          "key": "three",
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "friendly-lethal",
    {
      "target": "friendly",
      "expectedCount": 4,
      "units": [
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        },
        {
          "key": "two",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "three",
          "code": "IBH_007",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "self-lethal",
    {
      "target": "source",
      "expectedCount": 4,
      "units": [
        {
          "key": "one",
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "key": "two",
          "code": "IBH_007",
          "owner": "self"
        },
        {
          "key": "three",
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "decline-and-restore",
    {
      "target": "enemy",
      "decline": true,
      "restore": true,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent"
        },
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "own-deployed-leader",
    {
      "target": "leader",
      "expectedCount": 2,
      "deployOwn": true,
      "units": [
        {
          "key": "enemy",
          "code": "JTL_212",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "enemy-deployed-leader",
    {
      "target": "enemyLeader",
      "expectedCount": 1,
      "deployEnemy": true,
      "units": [
        {
          "key": "friendly",
          "code": "JTL_212",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "unique-keep-old",
    {
      "target": "enemy",
      "expectedCount": 2,
      "keepUnique": "old",
      "restore": true,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent"
        },
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "unique-keep-new",
    {
      "target": "enemy",
      "expectedCount": 2,
      "keepUnique": "new",
      "restore": true,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent"
        },
        {
          "key": "friendly",
          "code": "SEC_028",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "native-ground-selected",
    {
      "target": "native",
      "expectedCount": 2,
      "units": [
        {
          "key": "native",
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "native-space-selected",
    {
      "target": "native",
      "expectedCount": 1,
      "units": [
        {
          "key": "native",
          "code": "TOKEN_TIE_FIGHTER",
          "owner": "self",
          "token": "tie-fighter"
        }
      ]
    }
  ],
  [
    "decline-damaged-shield",
    {
      "target": "enemy",
      "decline": true,
      "units": [
        {
          "key": "enemy",
          "code": "HMW_082",
          "owner": "opponent",
          "shield": true,
          "damage": 2
        }
      ]
    }
  ]
];

function planAdmiralAckbarProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(const [variant,fields]of variants){
  const name='ackbar-'+variant+'-seat'+sourceSeat,id=entry.setCode+':admiral-ackbar:'+name;
  const setupCardIds=[...new Set(['SOR_014','ASH_021',...fields.units.map(u=>u.code),...(fields.units.some(u=>u.shield)?['TOKEN_SHIELD']:[])])];
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['chosen-arena-live-own-unit-count','optional-when-played','source-on-entry-count','intrinsic-restore','unique-identity'],assertionStages:['paid-source-entry','exact-optional-unit-menu','query-binding-and-count','damage-shield-and-defeat','unique-and-restore','terminal-continuation'],coverageObligationIds:[id],probe:{family:'admiral-ackbar',cardId:entry.setCode,sourceSeat,...fields}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'admiral-ackbar',dimension:c.name}));
 const uncovered=[['all-modifiers-replacements-and-controller-transfer','No complete source construction establishes every concurrent HP/power modifier, damage replacement, source departure or controller-transfer capture.'],['all-unique-and-death-trigger-interactions','Actual keep-old/keep-new copies are exercised; all uniqueness/death-trigger replacement interactions remain uncredited.'],['fatigue','No new draw effect exists in this source; fatigue during normal terminal continuation is not credited as an Ackbar-specific assertion.'],['all-domain-zero','A live played Ackbar counts itself in ground and supplies a positive damage domain; no unsupported offboard/all-domain-zero approximation is credited.']].map(([dimension,reason])=>{const id=entry.setCode+':admiral-ackbar:'+dimension;obligations.push({id,path:'admiral-ackbar',dimension});return{id,path:'admiral-ackbar',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planAdmiralAckbarProbes};
