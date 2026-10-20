'use strict';
const same = require('node:util').isDeepStrictEqual;
// Construction is source-qualified: a different lowered shape requires new
// source review rather than silently inheriting the Flock oracle contract.
const reviewedSource = {
  "setCode": "LOF_255",
  "sourcePath": "game/cards/05_LOF/units/CuriousFlock.ts",
  "sourceHash": "d189a10344162e862a142895c83717c70a0a63dd8fe2f35bb4ea4c283d9a25bd",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Pay up to 6 resources. For each resource paid this way, give an Experience token to this unit",
        "costs": [],
        "targetResolver": {
          "kind": "numberSelection",
          "mode": "chooseNumber",
          "min": {
            "kind": "const",
            "value": 0
          },
          "max": {
            "kind": "math",
            "name": "min",
            "args": [
              {
                "kind": "const",
                "value": 6
              },
              {
                "kind": "ref",
                "path": [
                  "player",
                  "readyResourceCount"
                ]
              }
            ]
          },
          "chooser": {
            "kind": "const",
            "value": "self"
          },
          "resultBinding": "select",
          "resolverName": "target",
          "namedResultStorage": "selects[name].choice",
          "contextSelectAssignment": "onlyWhenNameIsTarget",
          "resultRepresentation": "decimalString",
          "rangeAssertions": [
            "integerMinimum",
            "integerMaximum",
            "maximumAtLeastMinimum"
          ],
          "sameEndpoints": "automaticStringSelection",
          "logSelection": true,
          "defaults": "NumberTargetResolver"
        },
        "effect": {
          "kind": "sequence",
          "effects": [
            {
              "kind": "payUnadjustedResources",
              "target": "ownPlayer",
              "amount": {
                "kind": "selectionNumberCoercion",
                "operation": "parseInt",
                "args": [
                  {
                    "kind": "ref",
                    "path": [
                      "select"
                    ]
                  }
                ],
                "semantics": "javascriptParseInt",
                "invalidResult": "NaN"
              },
              "paymentType": "exhaustResources",
              "isCost": true,
              "allowCostAdjustments": false
            },
            {
              "kind": "giveExperience",
              "target": "self",
              "amount": {
                "kind": "selectionNumberCoercion",
                "operation": "parseInt",
                "args": [
                  {
                    "kind": "ref",
                    "path": [
                      "select"
                    ]
                  }
                ],
                "semantics": "javascriptParseInt",
                "invalidResult": "NaN"
              }
            }
          ]
        }
      }
    ]
  },
  "className": "CuriousFlock",
  "baseClass": "NonLeaderUnitCard",
  "id": "4974236883",
  "internalName": "curious-flock",
  "status": "supported",
  "card": {
    "title": "Curious Flock",
    "subtitle": "",
    "cost": 1,
    "hp": 1,
    "power": 1,
    "text": "When Played: Pay up to 6 resources. For each resource paid this way, give an Experience token to this unit.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": "Curious Flock's \"When Played\" ability is not reduced by effects that reduce the cost of cards.\n",
    "upgradePower": null,
    "upgradeHp": null,
    "id": "4974236883",
    "aspects": [],
    "traits": [
      "creature"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "LOF",
      "number": 255
    },
    "setCodes": [
      {
        "set": "LOF",
        "number": 255
      }
    ],
    "internalName": "curious-flock"
  },
  "dataHash": "2b23d127e1ed9dafe8d17f23482952b08ed197472f6bbe81fbd71bdcc0800a63",
  "keywordValues": {},
  "requiredCapabilities": [
    "cardType:unit",
    "ir:const",
    "ir:giveExperience",
    "ir:math",
    "ir:numberSelection",
    "ir:payUnadjustedResources",
    "ir:ref",
    "ir:selectionNumberCoercion",
    "ir:sequence",
    "ref:player.readyResourceCount",
    "ref:select",
    "targetMode:chooseNumber",
    "trigger:whenPlayed"
  ],
  "diagnostics": []
};
const reviewedSnoke = {
  "setCode": "SHD_037",
  "sourcePath": "game/cards/02_SHD/units/SupremeLeaderSnokeShadowRuler.ts",
  "sourceHash": "7f15a401fed97b0662d7be1b1502cc3650592ab474fa7713cf9b4ede0c4b5567",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "constant",
        "title": "Each enemy non-leader unit gets -2/-2",
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
  "className": "SupremeLeaderSnokeShadowRuler",
  "baseClass": "NonLeaderUnitCard",
  "id": "3731235174",
  "internalName": "supreme-leader-snoke#shadow-ruler",
  "status": "supported",
  "card": {
    "title": "Supreme Leader Snoke",
    "subtitle": "Shadow Ruler",
    "cost": 8,
    "hp": 6,
    "power": 6,
    "text": "Each enemy non-leader unit gets –2/–2.",
    "deployBox": null,
    "epicAction": null,
    "unique": true,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "3731235174",
    "aspects": [
      "villainy",
      "vigilance"
    ],
    "traits": [
      "force",
      "first order",
      "official"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "SHD",
      "number": 37
    },
    "setCodes": [
      {
        "set": "SHD",
        "number": 37
      }
    ],
    "internalName": "supreme-leader-snoke#shadow-ruler"
  },
  "dataHash": "255e5256db1124d322e532cf4682be07c231f1b11cfba3901da72cb2b747dafb",
  "keywordValues": {},
  "requiredCapabilities": [
    "cardType:unit",
    "ir:const",
    "ir:modifyStats",
    "targetType:nonLeaderUnit",
    "targetZone:arena",
    "trigger:constant"
  ],
  "diagnostics": []
};
const reviewedExperience = {
  "setCode": "TOKEN_EXPERIENCE",
  "sourcePath": "game/cards/01_SOR/tokens/Experience.ts",
  "sourceHash": "fccf00d13061bfc15447101a0dc82fc360f9b5d2e28a9da678684e8c93bc9d67",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": []
  },
  "className": "Experience",
  "baseClass": "TokenUpgradeCard",
  "id": "2007868442",
  "internalName": "experience",
  "status": "unsupported",
  "card": {
    "title": "Experience",
    "subtitle": null,
    "cost": 0,
    "hp": 1,
    "power": 1,
    "text": null,
    "deployBox": null,
    "epicAction": null,
    "unique": false,
    "rules": null,
    "upgradePower": 1,
    "upgradeHp": 1,
    "id": "2007868442",
    "aspects": [],
    "traits": [
      "learned"
    ],
    "keywords": [],
    "types": [
      "token",
      "upgrade"
    ],
    "setId": {
      "set": "SOR"
    },
    "internalName": "experience"
  },
  "dataHash": "cc1d7a3bf3344d8a95afdbba8607659ec1edd07e5282bb9eb9d3497b0f7f332b",
  "keywordValues": {},
  "requiredCapabilities": [
    "cardType:token",
    "cardType:upgrade"
  ],
  "diagnostics": [
    {
      "code": "UNSUPPORTED_SOURCE",
      "pattern": "method:isExperience",
      "file": "game/cards/01_SOR/tokens/Experience.ts",
      "line": 12,
      "column": 1,
      "message": "Additional class method requires conversion"
    },
    {
      "code": "UNSUPPORTED_SOURCE",
      "pattern": "class-member:GetAccessor",
      "file": "game/cards/01_SOR/tokens/Experience.ts",
      "line": 16,
      "column": 1,
      "message": "Additional class behavior requires conversion"
    }
  ]
};
function classifyVariablePayment(entry) {
    return same(entry, reviewedSource) ? {family:'variable-payment',sourceArena:'ground'} : null;
}
function planVariablePaymentProbes(entry, options = {}) {
    const shape = classifyVariablePayment(entry);
    if (!shape) return null;
    const catalogue = Array.isArray(options.catalogue) ? options.catalogue : options.catalogue?.cards ?? [];
    const snoke = catalogue.find(e => same(e, reviewedSnoke));
    const result = {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name, reason) {
        const id = entry.setCode+':variable-payment:'+name;
        result.obligations.push({id,path:'variable-payment',dimension:name});
        if(reason) result.uncovered.push({id,path:'variable-payment',reason});
        return id;
    }
    obligation('source-review','Construction plans do not establish full pinned source/compiler/runtime semantics; independent source-bound review required');
    if (!catalogue.some(e => same(e, reviewedExperience))) {
        obligation('native-experience-definition','Pinned complete native Experience definition missing or changed');
        return result;
    }
    function add(owner, paymentAmount, readyAfterPlay, scenario) {
        const name='owner-'+owner+'-'+scenario+'-pay-'+paymentAmount+'-ready-'+readyAfterPlay;
        const id=obligation(name);
        const probe={...shape,cardId:entry.setCode,owner,scenario,paymentAmount,readyAfterPlay,resourceCount:12};
        if(scenario==='captured-source-dead') probe.snokeCode='SHD_037';
        result.cases.push({id:'auto/'+entry.setCode+'/variable-payment-'+name,name:'variable-payment-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:scenario==='captured-source-dead'?['SHD_037','TOKEN_EXPERIENCE']:['TOKEN_EXPERIENCE'],behaviorIds:['variable-payment','when-played','unadjusted-resources','experience'],assertionStages:['construction','activation','observation'],coverageObligationIds:[id],probe});
    }
    for(const owner of [0,1]) {
        for(const amount of [0,1,3,6]) add(owner,amount,9,'normal');
        for(const [amount,ready] of [[0,0],[2,2],[6,6]]) add(owner,amount,ready,'normal');
        if(snoke) add(owner,3,9,'captured-source-dead');
        else obligation('owner-'+owner+'-captured-source-dead','Pinned complete admitted Snoke source donor missing or changed');
    }
    for(const name of ['credit-and-cost-reducer-interaction','pending-number-state-import','number-prompt-live-resource-change','experience-replacement-event'])
        obligation(name,'No actual source-card construction established for this interaction');
    return result;
}
module.exports={classifyVariablePayment,planVariablePaymentProbes};
