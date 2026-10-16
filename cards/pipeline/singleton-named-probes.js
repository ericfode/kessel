'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "LOF_054",
  "sourcePath": "game/cards/05_LOF/events/CalmInTheStorm.ts",
  "sourceHash": "ea8ac7f475ab22d882de74f1d3b637bcd4cc24af2aab879af5e527c6b15a28cc",
  "className": "CalmInTheStorm",
  "baseClass": "EventCard",
  "card": {
    "title": "Calm in the Storm",
    "subtitle": "",
    "cost": 2,
    "hp": null,
    "power": null,
    "text": "Exhaust a friendly unit. If you do, give a Shield token and 2 Experience tokens to it.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9854991700",
    "aspects": [
      "vigilance",
      "heroism"
    ],
    "traits": [
      "force"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "LOF",
      "number": 54
    },
    "setCodes": [
      {
        "set": "LOF",
        "number": 54
      }
    ],
    "internalName": "calm-in-the-storm"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Exhaust a friendly unit. If you do, give a Shield token and 2 Experience tokens to it.",
        "costs": [],
        "effect": {
          "kind": "exhaust",
          "target": "chosen"
        },
        "ifYouDo": {
          "title": "Give a Shield token and 2 Experience tokens to it",
          "costs": [],
          "effect": {
            "kind": "simultaneous",
            "effects": [
              {
                "kind": "giveExperience",
                "target": "chosen",
                "amount": {
                  "kind": "const",
                  "value": 2
                }
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
        },
        "targetResolver": {
          "owner": "self",
          "type": "unit",
          "zone": "anyAttackable"
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
    "ready-ground",
    {
      "target": 0,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "IBH_007",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "ready-space",
    {
      "target": 0,
      "targets": [
        {
          "code": "LOF_119",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "HMW_082",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "mixed-choose-ready",
    {
      "target": 1,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "JTL_212",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "mixed-choose-exhausted",
    {
      "target": 0,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": true
        },
        {
          "code": "IBH_007",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "JTL_212",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "all-exhausted",
    {
      "target": 0,
      "allExhausted": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": true
        },
        {
          "code": "LOF_119",
          "owner": "self",
          "exhausted": true
        }
      ]
    }
  ],
  [
    "no-friendly-empty",
    {
      "noFriendly": true,
      "targets": []
    }
  ],
  [
    "no-friendly-enemy-only",
    {
      "noFriendly": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "opponent",
          "exhausted": false
        },
        {
          "code": "LOF_119",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "cancel-no-friendly",
    {
      "noFriendly": true,
      "cancel": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "cancel-target",
    {
      "target": 0,
      "cancel": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "LOF_119",
          "owner": "self",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "native-ground",
    {
      "target": 0,
      "targets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "exhausted": false,
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "native-space",
    {
      "target": 0,
      "targets": [
        {
          "code": "TOKEN_XWING",
          "owner": "self",
          "exhausted": false,
          "token": "xwing"
        }
      ]
    }
  ],
  [
    "deployed-leader",
    {
      "target": "leader",
      "deployOwn": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "existing-tokens-damage",
    {
      "target": 0,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": false,
          "damage": 4,
          "attachments": [
            {
              "code": "TOKEN_EXPERIENCE",
              "token": "experience"
            },
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "LOF_119",
          "owner": "self",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "penalty-payment",
    {
      "target": 0,
      "penalty": true,
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "exhausted": false
        }
      ]
    }
  ]
];

function planSingletonNamedProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(const [variant,fields]of variants){const name='singleton-'+variant+'-seat'+sourceSeat,id=entry.setCode+':singleton-named:'+name;
  const setupCardIds=[...new Set(['SOR_014','ASH_019',fields.penalty?'ASH_023':'ASH_019','TOKEN_SHIELD','TOKEN_EXPERIENCE',...fields.targets.map(t=>t.code),...fields.targets.flatMap(t=>(t.attachments??[]).map(a=>a.code))])];
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['singleton-named-captured-identity','exhaust-if-you-do','basic-vs-effective-target','simultaneous-native-token-order'],assertionStages:['prepayment-target-or-confirmation','cancel-or-printed-payment','exact-exhaust-success','native-token-events-and-ids','captured-target-stats-and-attachments','terminal-continuation'],coverageObligationIds:[id],probe:{family:'singleton-named',cardId:entry.setCode,sourceSeat,...fields}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'singleton-named',dimension:c.name}));
 const uncovered=[['arbitrary-exhaust-replacements-and-capture','No construction establishes every exhaust replacement/restriction or target departure/controller-transfer capture.'],['all-token-creation-and-attachment-replacements','Native tokens and existing attachments are asserted; all replacement/restriction combinations are uncredited.'],['fatigue','This source does not draw; terminal-continuation fatigue is not credited as a source-specific assertion.'],['larger-named-resolver-topologies','Only exact one-slot, no chooser/dependency/retarget topology with selected identity retained is planned; larger or mutable selector lifetimes stay rejected.']].map(([dimension,reason])=>{const id=entry.setCode+':singleton-named:'+dimension;obligations.push({id,path:'singleton-named',dimension});return{id,path:'singleton-named',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planSingletonNamedProbes};
