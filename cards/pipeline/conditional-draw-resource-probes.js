'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "LAW_083",
  "sourcePath": "game/cards/07_LAW/units/BrokenHornVizagosPride.ts",
  "sourceHash": "00415235b2b9794c9b2af643a935a327b2d73edaee529c8467a95ba3531c1b02",
  "className": "BrokenHornVizagosPride",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Broken Horn",
    "subtitle": "Vizago's Pride",
    "cost": 5,
    "hp": 4,
    "power": 5,
    "text": "When Played: If you have fewer cards in hand than an opponent, draw a card. If you control fewer resources than an opponent, resource the top card of your deck.",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": "If there are multiple opponents, the controlling player chooses which one will be \u201can opponent.\u201d\nResources enter play exhausted.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9574762282",
    "aspects": [
      "aggression",
      "cunning"
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
    ],
    "setId": {
      "set": "LAW",
      "number": 83
    },
    "setCodes": [
      {
        "set": "LAW",
        "number": 83
      }
    ],
    "internalName": "broken-horn#vizagos-pride"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "If you have fewer cards in hand than your opponent, draw a card. If you control fewer resources than your opponent, resource the top card of your deck",
        "costs": [],
        "effect": {
          "kind": "sequence",
          "effects": [
            {
              "kind": "conditional",
              "condition": {
                "kind": "binary",
                "op": "<",
                "left": {
                  "kind": "ref",
                  "path": [
                    "player",
                    "hand",
                    "length"
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "opponent",
                    "hand",
                    "length"
                  ]
                }
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
            },
            {
              "kind": "conditional",
              "condition": {
                "kind": "binary",
                "op": "<",
                "left": {
                  "kind": "ref",
                  "path": [
                    "player",
                    "resources",
                    "length"
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "opponent",
                    "resources",
                    "length"
                  ]
                }
              },
              "onTrue": {
                "kind": "resourceCard",
                "target": "ownTopDeck"
              },
              "onFalse": {
                "kind": "noAction"
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
};
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
const variants=[
  [
    "hand-less-resources-less",
    {
      "ownMarkers": 1,
      "enemyResources": 13
    }
  ],
  [
    "hand-less-resources-equal",
    {
      "ownMarkers": 1,
      "enemyResources": 12
    }
  ],
  [
    "hand-less-resources-greater",
    {
      "ownMarkers": 1,
      "enemyResources": 11
    }
  ],
  [
    "hand-equal-resources-less",
    {
      "ownMarkers": 2,
      "enemyResources": 13
    }
  ],
  [
    "hand-equal-resources-equal",
    {
      "ownMarkers": 2,
      "enemyResources": 12
    }
  ],
  [
    "hand-equal-resources-greater",
    {
      "ownMarkers": 2,
      "enemyResources": 11
    }
  ],
  [
    "hand-greater-resources-less",
    {
      "ownMarkers": 3,
      "enemyResources": 13
    }
  ],
  [
    "hand-greater-resources-equal",
    {
      "ownMarkers": 3,
      "enemyResources": 12
    }
  ],
  [
    "hand-greater-resources-greater",
    {
      "ownMarkers": 3,
      "enemyResources": 11
    }
  ],
  [
    "deck0-tt",
    {
      "deckCodes": []
    }
  ],
  [
    "deck0-tf",
    {
      "deckCodes": [],
      "enemyResources": 12
    }
  ],
  [
    "deck0-ft",
    {
      "deckCodes": [],
      "ownMarkers": 2
    }
  ],
  [
    "deck0-ff",
    {
      "deckCodes": [],
      "ownMarkers": 2,
      "enemyResources": 12
    }
  ],
  [
    "deck1-tt",
    {
      "deckCodes": [
        "HMW_082"
      ]
    }
  ],
  [
    "deck1-ft-upgrade",
    {
      "deckCodes": [
        "ASH_087"
      ],
      "ownMarkers": 2
    }
  ],
  [
    "deck2-tt-event-upgrade",
    {
      "deckCodes": [
        "SHD_156",
        "ASH_087"
      ]
    }
  ],
  [
    "deck2-tf-upgrade-unit",
    {
      "deckCodes": [
        "ASH_087",
        "HMW_082"
      ],
      "enemyResources": 12
    }
  ],
  [
    "types-upgrade-carrier",
    {
      "deckCodes": [
        "ASH_087",
        "JTL_119",
        "TWI_229"
      ]
    }
  ],
  [
    "less-enemy-exhausted",
    {
      "enemyReady": 0
    }
  ],
  [
    "equal-own-only-five-ready",
    {
      "enemyResources": 12,
      "ownMarkers": 2,
      "ownReady": 5
    }
  ],
  [
    "greater-own-only-five-ready",
    {
      "enemyResources": 11,
      "ownMarkers": 2,
      "ownReady": 5
    }
  ],
  [
    "hand0-after-payment",
    {
      "ownMarkers": 0,
      "enemyMarkers": 1
    }
  ],
  [
    "both-hand0",
    {
      "ownMarkers": 0,
      "enemyMarkers": 0
    }
  ],
  [
    "fatigue26",
    {
      "deckCodes": [],
      "baseDamage": 26
    }
  ],
  [
    "fatal27",
    {
      "deckCodes": [],
      "baseDamage": 27
    }
  ],
  [
    "fatal29",
    {
      "deckCodes": [],
      "baseDamage": 29
    }
  ],
  [
    "fatal-draw-only",
    {
      "deckCodes": [],
      "baseDamage": 29,
      "enemyResources": 12
    }
  ],
  [
    "unique-keep-old",
    {
      "keepUnique": "old"
    }
  ],
  [
    "unique-keep-new",
    {
      "keepUnique": "new"
    }
  ]
];

function planConditionalDrawResourceProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(const [variant,fields]of variants){
 const name='conditional-draw-resource-'+variant+'-seat'+sourceSeat,id=entry.setCode+':conditional-draw-resource:'+name;
 const probe={family:'conditional-draw-resource',cardId:entry.setCode,sourceSeat,ownMarkers:1,enemyMarkers:2,ownResources:12,enemyResources:13,deckCodes:['HMW_082','SHD_156','ASH_087'],enemyDeckCodes:['IBH_008','SEC_044'],...fields};
 const setupCardIds=[...new Set(['SOR_014','ASH_025','TWI_170','IBH_007','LOF_112','SEC_028',...probe.deckCodes,...probe.enemyDeckCodes])].filter(c=>c!==entry.setCode);
 cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['mandatory-sequential-conditionals','hand-after-paid-entry','total-resource-comparison','draw-before-live-top-resource','separate-child-event-windows','empty-fatigue-and-terminal','captured-unique-source'],assertionStages:['paid-space-source','draw-and-resource-physical-identities','intermediate-window-order','exact-hand-resource-deck-state','fatigue-and-unique-boundaries','terminal-continuation'],coverageObligationIds:[id],probe});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'conditional-draw-resource',dimension:c.name}));
 const uncovered=[['replacements-and-interleavings','Arbitrary draw/resource restrictions, replacement, controller-transfer and topdeck mutation interactions remain uncredited.'],['postwinner-mandatory-step','Fatal empty draw leaves no resource target; terminal observation cannot establish an unobservable postwinner resource step.'],['global-triggers-and-capacity','Actual keep-old/new uniqueness is witnessed; all other trigger, multiplayer, hand/deck/resource capacity interactions remain uncredited.']].map(([dimension,reason])=>{const id=entry.setCode+':conditional-draw-resource:'+dimension;obligations.push({id,path:'conditional-draw-resource',dimension});return{id,path:'conditional-draw-resource',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planConditionalDrawResourceProbes};
