'use strict';
// Exact full source shape; recipient-bound amount semantics use the existing damage runtime.
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "JTL_144",
  "sourcePath": "game/cards/04_JTL/events/NoDisintegrations.ts",
  "sourceHash": "dc9d4cef2163188cdfb69013de20d1767575699b2e1fecf73b90f6e05642102c",
  "className": "NoDisintegrations",
  "baseClass": "EventCard",
  "card": {
    "title": "No Disintegrations",
    "subtitle": "",
    "cost": 3,
    "hp": null,
    "power": null,
    "text": "Deal damage to a non-leader unit equal to 1 less than its remaining HP.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "4033634907",
    "aspects": [
      "aggression",
      "villainy"
    ],
    "traits": [
      "plan"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "JTL",
      "number": 144
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 144
      }
    ],
    "internalName": "no-disintegrations"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Deal damage to a non-leader unit equal to 1 less than its remaining HP",
        "costs": [],
        "targetResolver": {
          "owner": "any",
          "type": "nonLeaderUnit",
          "zone": "anyAttackable"
        },
        "effect": {
          "kind": "damage",
          "target": "chosen",
          "amount": {
            "kind": "binary",
            "op": "-",
            "left": {
              "kind": "ref",
              "path": [
                "target",
                "remainingHp"
              ]
            },
            "right": {
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
  "piloting": null
};
const designs = [
  {
    "name": "JTL_144-undamaged-ground-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "undamaged-ground",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-damaged-ground-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "damaged-ground",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-damaged-space-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "damaged-space",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "JTL_212",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-shield-positive-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "shield-positive",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "own",
        "damage": 1,
        "shields": 1
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-mixed-zero-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "mixed-zero",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-mixed-zero-shield-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "mixed-zero-shield",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "own",
        "damage": 0,
        "shields": 1
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-cancel-own",
    "family": "recipient-remaining-hp-damage",
    "mode": "cancel",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-undamaged-ground-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "undamaged-ground",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-damaged-ground-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "damaged-ground",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-damaged-space-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "damaged-space",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "JTL_212",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-shield-positive-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "shield-positive",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 1
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-mixed-zero-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "mixed-zero",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-mixed-zero-shield-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "mixed-zero-shield",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "enemy",
        "damage": 0,
        "shields": 1
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "effective",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-cancel-enemy",
    "family": "recipient-remaining-hp-damage",
    "mode": "cancel",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-all-zero-shield-0",
    "family": "recipient-remaining-hp-zero-domain",
    "mode": "play",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "SOR_225",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-all-zero-shield-1",
    "family": "recipient-remaining-hp-zero-domain",
    "mode": "play",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "ASH_190",
        "owner": "own",
        "damage": 0,
        "shields": 1
      },
      {
        "key": "other",
        "code": "SOR_225",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "JTL_144"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-no-nonleader-play",
    "family": "recipient-remaining-hp-no-target",
    "mode": "play",
    "deployedLeader": true,
    "units": [],
    "plan": [
      {
        "code": "JTL_144"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-no-nonleader-cancel",
    "family": "recipient-remaining-hp-no-target",
    "mode": "cancel",
    "deployedLeader": true,
    "units": [],
    "plan": [
      {
        "code": "JTL_144"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-native-battle-droid",
    "family": "recipient-remaining-hp-native-token",
    "mode": "play",
    "deployedLeader": true,
    "units": [
      {
        "key": "effective",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "subject",
        "token": "battle-droid",
        "owner": "own"
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-native-xwing",
    "family": "recipient-remaining-hp-native-token",
    "mode": "play",
    "deployedLeader": true,
    "units": [
      {
        "key": "effective",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "initialTokens": [
      {
        "key": "subject",
        "token": "xwing",
        "owner": "own"
      }
    ],
    "plan": [
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-live-hp-boost-own",
    "family": "recipient-remaining-hp-live-stat",
    "mode": "play",
    "targetOwner": "own",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "TWI_124",
        "target": "subject"
      },
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  },
  {
    "name": "JTL_144-live-hp-boost-enemy",
    "family": "recipient-remaining-hp-live-stat",
    "mode": "play",
    "targetOwner": "enemy",
    "deployedLeader": true,
    "units": [
      {
        "key": "subject",
        "code": "SEC_028",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_268",
        "owner": "own",
        "damage": 0,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "TWI_124",
        "target": "subject"
      },
      {
        "code": "JTL_144",
        "target": "subject"
      }
    ],
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_144"
    ]
  }
];
const normalize = e => ({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planRemainingHpDamageProbes(entry) {
    if (entry.status !== 'supported' || entry.diagnostics?.length || !same(shape, normalize(entry))) return null;
    const specs = designs.flatMap(s => [0,1].map(ownSeat => ({...s,name:s.name+'-p'+(ownSeat+1),ownSeat})));
    const cases = specs.map(design => {
        const obligationId = entry.setCode+':remaining-hp:'+design.name;
        return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_014',...(design.units??[]).map(u=>u.code),...design.plan.map(p=>p.code)])].sort(),behaviorIds:['recipient-remaining-hp-damage','exact-zero-target-feasibility'],assertionStages:['prepayment-targeting','real-event-payment','recipient-source-assertions','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'remaining-hp-damage',scenarioName:design.name,design}};
    });
    const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'recipient-remaining-hp',dimension:c.name}));
    const uncovered=[
        ['source-semantics','Finite named witnesses do not prove completeness of lowering'],
        ['intervening-hp-or-damage-change','No unrelated effect changes recipient HP/damage between selection and damage-event preparation; source callback evaluation time remains separately reviewed'],
        ['nonpositive-remaining-hp','No lethal arena unit is fabricated; nonpositive remaining-HP recipients are removed by state checks in the admitted pool'],
    ].map(([dimension,reason])=>{const id=entry.setCode+':remaining-hp:'+dimension;obligations.push({id,path:'recipient-remaining-hp',dimension});return{id,path:'recipient-remaining-hp',reason};});
    return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planRemainingHpDamageProbes};
