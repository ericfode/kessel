'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "SOR_226",
  "sourcePath": "game/cards/01_SOR/units/AdmiralMottiBrazenAndScornful.ts",
  "sourceHash": "1af4970868a6afb887c1b71f0fc69390f3218d516d969887fbcb96d19afb8887",
  "className": "AdmiralMottiBrazenAndScornful",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Admiral Motti",
    "subtitle": "Brazen and Scornful",
    "cost": 2,
    "hp": 1,
    "power": 1,
    "text": "When Defeated: You may ready a [Villainy] unit.",
    "deployBox": null,
    "epicAction": null,
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9996676854",
    "aspects": [
      "villainy"
    ],
    "traits": [
      "imperial",
      "official"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "SOR",
      "number": 226
    },
    "setCodes": [
      {
        "set": "SOR",
        "number": 226
      }
    ],
    "internalName": "admiral-motti#brazen-and-scornful"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenDefeated",
        "title": "Ready a Villainy unit",
        "costs": [],
        "optional": true,
        "targetResolver": {
          "owner": "any",
          "type": "any",
          "zone": "arena",
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
        },
        "effect": {
          "kind": "ready",
          "target": "chosen"
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
    "self-ground",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "self-space",
    {
      "targets": [
        {
          "code": "IBH_075",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "opponent-ground",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "opponent-space",
    {
      "targets": [
        {
          "code": "IBH_075",
          "owner": "opponent"
        }
      ]
    }
  ],
  [
    "mixed-owner-arena-0",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        },
        {
          "code": "IBH_075",
          "owner": "self"
        },
        {
          "code": "SEC_080",
          "owner": "opponent"
        },
        {
          "code": "LAW_175",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 0
    }
  ],
  [
    "mixed-owner-arena-1",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        },
        {
          "code": "IBH_075",
          "owner": "self"
        },
        {
          "code": "SEC_080",
          "owner": "opponent"
        },
        {
          "code": "LAW_175",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 1
    }
  ],
  [
    "mixed-owner-arena-2",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        },
        {
          "code": "IBH_075",
          "owner": "self"
        },
        {
          "code": "SEC_080",
          "owner": "opponent"
        },
        {
          "code": "LAW_175",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 2
    }
  ],
  [
    "mixed-owner-arena-3",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        },
        {
          "code": "IBH_075",
          "owner": "self"
        },
        {
          "code": "SEC_080",
          "owner": "opponent"
        },
        {
          "code": "LAW_175",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 3
    }
  ],
  [
    "choose-ready-self",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "IBH_075",
          "owner": "self"
        }
      ],
      "chosenIndex": 0
    }
  ],
  [
    "choose-ready-opponent",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "opponent",
          "exhausted": false
        },
        {
          "code": "IBH_075",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 0
    }
  ],
  [
    "optional-decline",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        },
        {
          "code": "IBH_075",
          "owner": "self"
        },
        {
          "code": "SEC_080",
          "owner": "opponent"
        },
        {
          "code": "LAW_175",
          "owner": "opponent"
        }
      ],
      "decline": true
    }
  ],
  [
    "all-ready-no-effective",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self",
          "exhausted": false
        },
        {
          "code": "IBH_075",
          "owner": "opponent",
          "exhausted": false
        }
      ]
    }
  ],
  [
    "no-villainy",
    {
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self"
        },
        {
          "code": "IBH_007",
          "owner": "opponent"
        },
        {
          "code": "TOKEN_CLONE_TROOPER",
          "owner": "self",
          "token": "clone-trooper"
        },
        {
          "code": "TOKEN_XWING",
          "owner": "opponent",
          "token": "xwing"
        },
        {
          "code": "TOKEN_BEAST",
          "owner": "self",
          "token": "beast"
        }
      ]
    }
  ],
  [
    "only-source",
    {
      "targets": []
    }
  ],
  [
    "self-battle-droid",
    {
      "targets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "self-tie-fighter",
    {
      "targets": [
        {
          "code": "TOKEN_TIE_FIGHTER",
          "owner": "self",
          "token": "tie-fighter"
        }
      ]
    }
  ],
  [
    "opponent-battle-droid",
    {
      "targets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "owner": "opponent",
          "token": "battle-droid"
        }
      ]
    }
  ],
  [
    "opponent-tie-fighter",
    {
      "targets": [
        {
          "code": "TOKEN_TIE_FIGHTER",
          "owner": "opponent",
          "token": "tie-fighter"
        }
      ]
    }
  ],
  [
    "native-exclusions",
    {
      "targets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "owner": "opponent",
          "token": "battle-droid"
        },
        {
          "code": "TOKEN_CLONE_TROOPER",
          "owner": "self",
          "token": "clone-trooper"
        },
        {
          "code": "TOKEN_XWING",
          "owner": "opponent",
          "token": "xwing"
        },
        {
          "code": "TOKEN_BEAST",
          "owner": "self",
          "token": "beast"
        },
        {
          "code": "TOKEN_MANDALORIAN",
          "owner": "opponent",
          "token": "mandalorian"
        }
      ]
    }
  ],
  [
    "deployed-leader-self",
    {
      "targets": [
        {
          "code": "SOR_007",
          "owner": "self",
          "deployedLeader": true
        }
      ]
    }
  ],
  [
    "deployed-leader-opponent",
    {
      "targets": [
        {
          "code": "SOR_007",
          "owner": "opponent",
          "deployedLeader": true
        }
      ]
    }
  ],
  [
    "front-leader-excluded",
    {
      "targets": [
        {
          "code": "IBH_075",
          "owner": "self"
        }
      ]
    }
  ],
  [
    "attached-villainy-is-not-unit",
    {
      "targets": [
        {
          "code": "HMW_082",
          "owner": "self",
          "attachments": [
            {
              "code": "HMW_252",
              "owner": "self"
            }
          ]
        },
        {
          "code": "IBH_075",
          "owner": "opponent"
        }
      ],
      "chosenIndex": 1
    }
  ],
  [
    "hidden-villainy-not-unit-domain",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self",
          "zone": "hand"
        },
        {
          "code": "IBH_075",
          "owner": "opponent",
          "zone": "resource"
        },
        {
          "code": "IBH_055",
          "owner": "opponent",
          "zone": "discard"
        },
        {
          "code": "SEC_080",
          "owner": "self"
        }
      ],
      "chosenIndex": 3
    }
  ],
  [
    "opponent-actual-card-defeat",
    {
      "targets": [
        {
          "code": "IBH_075",
          "owner": "opponent"
        }
      ],
      "defeatEventOwner": "opponent"
    }
  ],
  [
    "combat-friendly-ground",
    {
      "targets": [
        {
          "code": "IBH_055",
          "owner": "self"
        }
      ],
      "deathMode": "combat"
    }
  ],
  [
    "combat-enemy-space",
    {
      "targets": [
        {
          "code": "IBH_075",
          "owner": "opponent"
        }
      ],
      "deathMode": "combat"
    }
  ],
  [
    "combat-no-eligible",
    {
      "targets": [],
      "deathMode": "combat"
    }
  ],
  [
    "combat-mutual-defeat-source-and-villainy-attacker",
    {
      "targets": [],
      "deathMode": "combat",
      "mutual": true
    }
  ],
  [
    "unique-keep-old",
    {
      "targets": [],
      "keepUnique": "old"
    }
  ],
  [
    "unique-keep-new",
    {
      "targets": [],
      "keepUnique": "new"
    }
  ]
];
function buildCases(){const cases=[];for(const sourceSeat of [1,2])for(const [name,fields]of variants){const n='aspect-ready-death-'+name+'-seat'+sourceSeat;cases.push({id:'auto/SOR_226/'+n,name:n,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:n},focusCardIds:['SOR_226'],setupCardIds:[...new Set(['SOR_007','ASH_019','SOR_077','HMW_082','IBH_063',...fields.targets.flatMap(t=>[t.code,...(t.attachments??[]).map(x=>x.code)])])],behaviorIds:['printed-aspect-unit-ready','either-owner-any-arena','optional-death-controller','basic-vs-effective-ready','native-and-deployed-domain','actual-paid-play-death'],assertionStages:['paid-unit-play','actual-source-death','exact-optional-title-and-target-domain','selected-ready-or-decline-or-skipped','unselected-identity-state','complete-terminal-game'],coverageObligationIds:['SOR_226:aspect-ready-death:'+n],probe:{family:'aspect-ready-death',cardId:'SOR_226',sourceSeat,deathMode:'event',chosenIndex:0,...fields}});}return cases;}
function planAspectReadyDeathProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=buildCases();
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'aspect-ready-death',dimension:c.name}));
 const uncovered=[['control-and-replacements','Controller/owner divergence, arbitrary Ready restrictions/replacements and multiplayer are not covered by these source constructions.'],['competing-trigger-and-reentry','Actual combat, Takedown and uniqueness deaths are witnessed; all other competing death responses, capture/rescue/reentry and stat-aura histories remain uncredited.'],['capacity-and-malformed-states','Global hand/arena/resource limits and malformed native/leader metadata require independent validation diagnostics.']].map(([dimension,reason])=>{const id=entry.setCode+':aspect-ready-death:'+dimension;obligations.push({id,path:'aspect-ready-death',dimension});return{id,path:'aspect-ready-death',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:structuredClone(cases),obligations,uncovered};
}
module.exports={planAspectReadyDeathProbes};
