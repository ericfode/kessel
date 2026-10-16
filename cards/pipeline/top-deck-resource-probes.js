'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "JTL_119",
  "sourcePath": "game/cards/04_JTL/units/ResupplyCarrier.ts",
  "sourceHash": "e6852e63bb7c8508956a700c59d11467b5d70687dae082beabbc8c6cb17c476d",
  "className": "ResupplyCarrier",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Resupply Carrier",
    "subtitle": "",
    "cost": 6,
    "hp": 5,
    "power": 4,
    "text": "When Played: You may put the top card of your deck into play as a resource.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "A card put into play as a resource enters play exhausted.\n(ERRATA) Templating update: \u201cPut into play as a resource\u201d becomes \u201cresource\u201d.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "0926549684",
    "aspects": [
      "command"
    ],
    "traits": [
      "imperial",
      "vehicle",
      "capital ship"
    ],
    "arena": "space",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "JTL",
      "number": 119
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 119
      }
    ],
    "internalName": "resupply-carrier"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Put the top card of your deck into play as a resource",
        "costs": [],
        "optional": true,
        "effect": {
          "kind": "resourceCard",
          "target": "ownTopDeck"
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
    "unit-single",
    {
      "deckCodes": [
        "HMW_082"
      ]
    }
  ],
  [
    "unit-multiple",
    {
      "deckCodes": [
        "HMW_082",
        "SHD_156",
        "ASH_087"
      ]
    }
  ],
  [
    "event-top",
    {
      "deckCodes": [
        "SHD_156"
      ]
    }
  ],
  [
    "upgrade-top",
    {
      "deckCodes": [
        "ASH_087"
      ]
    }
  ],
  [
    "carrier-top-no-recursion",
    {
      "deckCodes": [
        "JTL_119"
      ]
    }
  ],
  [
    "token-maker-top-no-play",
    {
      "deckCodes": [
        "TWI_229"
      ],
      "arenaTargets": [
        {
          "code": "TOKEN_BATTLE_DROID",
          "owner": "self",
          "token": "battle-droid",
          "attachments": [
            {
              "code": "TOKEN_SHIELD",
              "token": "shield"
            }
          ]
        },
        {
          "code": "TOKEN_XWING",
          "owner": "opponent",
          "token": "xwing"
        }
      ]
    }
  ],
  [
    "pass-unit",
    {
      "deckCodes": [
        "HMW_082"
      ],
      "decline": true
    }
  ],
  [
    "pass-event",
    {
      "deckCodes": [
        "SHD_156"
      ],
      "decline": true
    }
  ],
  [
    "pass-upgrade",
    {
      "deckCodes": [
        "ASH_087"
      ],
      "decline": true
    }
  ],
  [
    "own-empty-enemy-present",
    {
      "deckCodes": []
    }
  ],
  [
    "own-present-enemy-empty",
    {
      "deckCodes": [
        "HMW_082"
      ],
      "enemyDeckCodes": []
    }
  ],
  [
    "both-empty",
    {
      "deckCodes": [],
      "enemyDeckCodes": []
    }
  ],
  [
    "exact-six-ready",
    {
      "deckCodes": [
        "HMW_082"
      ],
      "initialReady": 6
    }
  ],
  [
    "mixed-exhausted-resources",
    {
      "deckCodes": [
        "HMW_082"
      ],
      "initialReady": 10
    }
  ],
  [
    "penalty-eight",
    {
      "deckCodes": [
        "HMW_082"
      ],
      "penalty": true
    }
  ],
  [
    "natural-regroup-readies",
    {
      "deckCodes": [
        "HMW_082",
        "SHD_156",
        "ASH_087",
        "IBH_007"
      ],
      "waitForRegroup": true
    }
  ]
];

function planTopDeckResourceProbes(entry){
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];for(const sourceSeat of [1,2])for(const [variant,fields]of variants){const name='top-resource-'+variant+'-seat'+sourceSeat,id=entry.setCode+':top-deck-resource:'+name,enemyDeckCodes=fields.enemyDeckCodes??['IBH_007','HMW_082'];
  const setupCardIds=[...new Set(['SOR_014','ASH_021',...(fields.penalty?['ASH_019']:[]),...fields.deckCodes,...enemyDeckCodes,...(fields.arenaTargets??[]).flatMap(t=>[t.code,...(t.attachments??[]).map(a=>a.code)])])].filter(code=>code!==entry.setCode);
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['optional-paid-top-deck-resource','exact-physical-top-and-controller','exhausted-resource-order','hidden-unit-zone-eligibility','no-play-or-draw-side-effects'],assertionStages:['paid-source-space-entry','optional-trigger-pass-or-empty-skip','exact-resourced-top-and-prefix','deck-tail-and-opponent-state','natural-regroup-readiness','terminal-continuation'],coverageObligationIds:[id],probe:{family:'top-deck-resource',cardId:entry.setCode,sourceSeat,enemyDeckCodes,...fields}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'top-deck-resource',dimension:c.name}));
 const uncovered=[['arbitrary-topdeck-capture-and-controller-transfer','No source construction establishes every draw/topdeck mutation or controller-transfer replacement interleaving between legal check and resource resolution.'],['resource-replacements-and-capacity','Normal exhausted-resource entry and natural readiness are asserted; all replacement/restriction and global resource/card-capacity edges remain uncredited.'],['malformed-hidden-token-leader-base-states','Native tokens are not playable and cannot legally occupy a deck; malformed token/leader/base hidden states require separate raw validation diagnostics, not fabricated source scenarios.'],['fatigue','Empty topdeck lookup causes no draw/fatigue during focused resolution; later terminal-continuation fatigue is not credited as a new source effect.']].map(([dimension,reason])=>{const id=entry.setCode+':top-deck-resource:'+dimension;obligations.push({id,path:'top-deck-resource',dimension});return{id,path:'top-deck-resource',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planTopDeckResourceProbes};
