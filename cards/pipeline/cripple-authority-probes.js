'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "SHD_156",
  "sourcePath": "game/cards/02_SHD/events/CrippleAuthority.ts",
  "sourceHash": "f2abd62ba1930201f44fe9615dbc5f4cf0ce2b0fc9f714688c519d0ca03b763d",
  "className": "CrippleAuthority",
  "baseClass": "EventCard",
  "card": {
    "title": "Cripple Authority",
    "subtitle": "",
    "cost": 2,
    "hp": null,
    "power": null,
    "text": "Draw a card. Each opponent who controls more resources than you discards a card from their hand.",
    "deployBox": null,
    "epicAction": null,
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "3228620062",
    "aspects": [
      "heroism",
      "aggression"
    ],
    "traits": [
      "trick"
    ],
    "keywords": [],
    "types": [
      "event"
    ],
    "setId": {
      "set": "SHD",
      "number": 156
    },
    "setCodes": [
      {
        "set": "SHD",
        "number": 156
      }
    ],
    "internalName": "cripple-authority"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "event",
        "title": "Draw a card. Each opponent who controls more resources than you discards a card from their hand",
        "costs": [],
        "effect": {
          "kind": "simultaneous",
          "effects": [
            {
              "kind": "draw",
              "target": "ownPlayer",
              "amount": {
                "kind": "const",
                "value": 1
              }
            },
            {
              "kind": "conditional",
              "condition": {
                "kind": "binary",
                "op": ">",
                "left": {
                  "kind": "ref",
                  "path": [
                    "opponent",
                    "resources",
                    "length"
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "player",
                    "resources",
                    "length"
                  ]
                }
              },
              "onTrue": {
                "kind": "discardOwnHand",
                "target": "enemyPlayer",
                "amount": {
                  "kind": "const",
                  "value": 1
                }
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
    "above-first",
    {}
  ],
  [
    "above-last",
    {
      "discardIndex": 1
    }
  ],
  [
    "equal",
    {
      "enemyResources": 12
    }
  ],
  [
    "below",
    {
      "enemyResources": 11
    }
  ],
  [
    "above-single-hand",
    {
      "enemyHand": 1
    }
  ],
  [
    "above-empty-hand",
    {
      "enemyHand": 0
    }
  ],
  [
    "equal-empty-hand",
    {
      "enemyResources": 12,
      "enemyHand": 0
    }
  ],
  [
    "below-empty-hand",
    {
      "enemyResources": 11,
      "enemyHand": 0
    }
  ],
  [
    "above-own-empty-after-play",
    {
      "ownMarker": false
    }
  ],
  [
    "above-one-card-deck",
    {
      "deckSize": 1
    }
  ],
  [
    "above-empty-deck",
    {
      "deckSize": 0
    }
  ],
  [
    "equal-empty-deck",
    {
      "enemyResources": 12,
      "deckSize": 0
    }
  ],
  [
    "below-empty-deck",
    {
      "enemyResources": 11,
      "deckSize": 0
    }
  ],
  [
    "above-opponent-empty-deck",
    {
      "enemyDeckSize": 0
    }
  ],
  [
    "above-opponent-exhausted",
    {
      "enemyExhausted": 13
    }
  ],
  [
    "equal-own-exhausted",
    {
      "enemyResources": 12,
      "ownExhausted": 10
    }
  ],
  [
    "below-own-exhausted",
    {
      "enemyResources": 11,
      "ownExhausted": 10
    }
  ],
  [
    "above-fatal27",
    {
      "deckSize": 0,
      "fatal": true,
      "baseDamage": 27
    }
  ],
  [
    "above-fatal29",
    {
      "deckSize": 0,
      "fatal": true,
      "baseDamage": 29
    }
  ],
  [
    "equal-fatal29",
    {
      "enemyResources": 12,
      "deckSize": 0,
      "fatal": true,
      "baseDamage": 29
    }
  ],
  [
    "below-fatal27",
    {
      "enemyResources": 11,
      "deckSize": 0,
      "fatal": true,
      "baseDamage": 27
    }
  ],
  [
    "above-fatal-empty-hands",
    {
      "enemyHand": 0,
      "ownMarker": false,
      "deckSize": 0,
      "fatal": true,
      "baseDamage": 29
    }
  ]
];
function planCrippleAuthorityProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(const [variant,fields]of variants){
  const name='cripple-'+variant+'-seat'+sourceSeat,id=entry.setCode+':cripple-authority:'+name;
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds:['SOR_014','ASH_023','IBH_007','TWI_170'],behaviorIds:['finite-property-factory','simultaneous-draw-discard','opponent-hand-chooser','preparation-before-resolution'],assertionStages:['paid-source','pre-draw-discard-prompt','exact-group-event-order','fatigue-and-winner','terminal-continuation'],coverageObligationIds:[id],probe:{family:'cripple-authority',cardId:entry.setCode,sourceSeat,ownResources:12,enemyResources:13,enemyHand:2,...fields}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'cripple-authority',dimension:c.name}));
 const uncovered=[['external-resource-change-and-controller-transfer','Exact source group changes hands/deck only; external resource-changing captures/controller-transfer interleavings have no full-source construction.'],['all-replacements-and-restrictions','No construction establishes every simultaneous draw/discard replacement or restriction.'],['multiplayer','Engine factory iterates one explicit opponent in this pinned source; runtime is two-player only.'],['arbitrary-factory-capture','Only finite exact resource-comparison factory is planned; arbitrary captured locals, mutable aliases, extra leaves and resource-changing helpers stay rejected.']].map(([dimension,reason])=>{const id=entry.setCode+':cripple-authority:'+dimension;obligations.push({id,path:'cripple-authority',dimension});return{id,path:'cripple-authority',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planCrippleAuthorityProbes};
