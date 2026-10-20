'use strict';
const same=require('node:util').isDeepStrictEqual,crypto=require('node:crypto');
// Semantic fingerprints preserve every field, array position, value and type;
// JSON object insertion order is not an execution identity. No parser/engine.
function canonical(v){
    if(v===null)return 'null';
    if(Array.isArray(v))return '['+v.map(canonical).join(',')+']';
    if(typeof v==='object'){
        if(![Object.prototype,null].includes(Object.getPrototypeOf(v))||Reflect.ownKeys(v).some(k=>typeof k!=='string'))throw new Error('Non-JSON object');
        return '{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}';
    }
    if(typeof v==='number'){if(!Number.isFinite(v))throw new Error('Non-JSON number');return Object.is(v,-0)?'-0':String(v);}
    if(typeof v==='string'||typeof v==='boolean')return JSON.stringify(v);
    throw new Error('Non-JSON value');
}
function fingerprint(v){try{return crypto.createHash('sha256').update(canonical(v)).digest('hex');}catch{return null;}}
function profile(entry){const e=structuredClone(entry);for(const key of ['setCode','id','internalName','className'])delete e[key];if(e.card)for(const key of ['id','internalName','setId','setCodes'])delete e.card[key];return e;}
const reviewedProfiles=[
  {
    "setCode": "ASH_169",
    "sourcePath": "game/cards/08_ASH/units/AxeWovesUndaunted.ts",
    "sourceHash": "a70d68ce72653355bfd9038e5b80b0c77ab0ee9190d31e2c9d7da8363979f296",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Give an Advantage token to Axe Woves, Undaunted",
          "costs": [],
          "contextTitle": [
            {
              "kind": "const",
              "value": "Give an Advantage token to "
            },
            {
              "kind": "ref",
              "path": [
                "source",
                "title"
              ]
            },
            {
              "kind": "const",
              "value": ""
            }
          ],
          "collectiveTrigger": true,
          "effect": {
            "kind": "giveAdvantage",
            "target": "self",
            "amount": {
              "kind": "const",
              "value": 1
            },
            "createdBy": "self",
            "batch": "singleCreationEvent",
            "tokenOwner": "recipientController"
          },
          "hooks": [
            {
              "event": "onCardsDrawn",
              "predicate": {
                "kind": "binary",
                "op": "===",
                "left": {
                  "kind": "dataPath",
                  "root": [
                    "event"
                  ],
                  "argumentBinding": {
                    "id": "callback:874:0",
                    "bindAt": "callbackInvocation",
                    "readAt": "expressionEvaluation"
                  },
                  "steps": [
                    {
                      "key": "player",
                      "optional": false
                    }
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "player"
                  ]
                }
              }
            }
          ]
        }
      ]
    },
    "className": "AxeWovesUndaunted",
    "baseClass": "NonLeaderUnitCard",
    "id": "9992086344",
    "internalName": "axe-woves#undaunted",
    "status": "supported",
    "card": {
      "title": "Axe Woves",
      "subtitle": "Undaunted",
      "cost": 3,
      "hp": 4,
      "power": 2,
      "text": "When you draw 1 or more cards (including during the regroup phase): Give an Advantage token to this unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9992086344",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "mandalorian"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 169
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 169
        }
      ],
      "internalName": "axe-woves#undaunted"
    },
    "dataHash": "ebdc12b6804c6e9246595ebe04cc2960f7fbc8cbcf76d917cae923988fd0c652",
    "keywordValues": {},
    "requiredCapabilities": [
      "binary:===",
      "cardType:unit",
      "ir:binary",
      "ir:const",
      "ir:dataPath",
      "ir:giveAdvantage",
      "ir:ref",
      "ref:player",
      "ref:source.title",
      "trigger:eventHooks"
    ],
    "diagnostics": []
  },
  {
    "setCode": "ASH_208",
    "sourcePath": "game/cards/08_ASH/units/SabineWrenILearnedTheHardWay.ts",
    "sourceHash": "1b3d7e38064f55a936e034d8787f35b0f1fd7c02f85e5282eef3f2f1de6d76f4",
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "eventHooks",
          "title": "Exhaust a ground unit",
          "costs": [],
          "optional": true,
          "collectiveTrigger": true,
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "arena",
            "arena": "ground"
          },
          "effect": {
            "kind": "exhaust",
            "target": "chosen"
          },
          "hooks": [
            {
              "event": "onUpgradeAttached",
              "predicate": {
                "kind": "binary",
                "op": "===",
                "left": {
                  "kind": "dataPath",
                  "root": [
                    "event"
                  ],
                  "argumentBinding": {
                    "id": "callback:856:0",
                    "bindAt": "callbackInvocation",
                    "readAt": "expressionEvaluation"
                  },
                  "steps": [
                    {
                      "key": "parentCard",
                      "optional": false
                    }
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "source"
                  ]
                }
              }
            }
          ]
        }
      ]
    },
    "className": "SabineWrenILearnedTheHardWay",
    "baseClass": "NonLeaderUnitCard",
    "id": "6033198879",
    "internalName": "sabine-wren#i-learned-the-hard-way",
    "status": "supported",
    "card": {
      "title": "Sabine Wren",
      "subtitle": "I Learned the Hard Way",
      "cost": 5,
      "hp": 5,
      "power": 4,
      "text": "Shielded (When you play this unit, give a Shield token to her.)\nWhen 1 or more upgrades attach to this unit (including from Shielded): You may exhaust a ground unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "6033198879",
      "aspects": [
        "cunning",
        "heroism"
      ],
      "traits": [
        "jedi",
        "mandalorian",
        "spectre"
      ],
      "arena": "ground",
      "keywords": [
        "shielded"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 208
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 208
        }
      ],
      "internalName": "sabine-wren#i-learned-the-hard-way"
    },
    "dataHash": "04bb2956a763fc66e072c5ab25367906e0706343b149ed8b9ea89d61e86297d9",
    "keywordValues": {
      "shielded": 1
    },
    "requiredCapabilities": [
      "binary:===",
      "cardType:unit",
      "ir:binary",
      "ir:dataPath",
      "ir:exhaust",
      "ir:ref",
      "keyword:shielded",
      "ref:source",
      "targetType:unit",
      "targetZone:arena",
      "trigger:eventHooks"
    ],
    "diagnostics": []
  }
];
const reviewedDonors={
  "ASH_061": "f6a033f123707b8cd7f0431599b43617fa2e088092586b0fc67343ca8d482c9b",
  "ASH_131": "1fcfd58c506b4e94f313d01deacffd6cf2fbc4c2bb180bd7bb1d46d296961c2d",
  "ASH_190": "5d22cbe617157548f41e5cc0d44365242c093ed7a66e8135cd52e106e41c0b93",
  "ASH_201": "173783137608a82a1eae6b38accd237d6484517f350c4fe9f399ec4dd08a7c08",
  "ASH_242": "d0a10a6e6307b625451b06765d5f5b8304f68d9e3e2cc3edd510c564eee4abd5",
  "ASH_261": "e62a861a7625bd2c9f6222abcf1ca31168d7a493d2bf5af30188a863a5325ab2",
  "HMW_082": "d4b667f1930c70b6bc1eac9ec62dd96121b44c3baba5ef9d0e9a32fb0a683e81",
  "HMW_093": "5c1f7ec438f2e2ad83ea97ab207ad8bdf99f80be4d2dc04e4734925f2b8477ee",
  "HMW_116": "b16e84dce80f70da340f5c2c87580793ce7367f8b516f24ff63680a2ec281e69",
  "HMW_128": "ce5b5159056a22ff31c7ec06d689caede18bc64edacef70b39e72efff0d73027",
  "HMW_135": "e74522cc77ce7cbf0d454a4948ff5b12684f882268757a157a63761a4bbbdc83",
  "HMW_155": "9db7facb8ee661bc3c4181c8c07d9f1629e2b1ea0f7371c63e68b66522002c19",
  "HMW_163": "9021b8be120211bfeb9d2b7a357fe033bd16642d59602c7a18093acad5a0b48a",
  "HMW_174": "68d86d06abc3e805c3ac3c424aa6ddeb2258d405ffb99b651323680584894d19",
  "HMW_178": "c59421061ea586b6387d6e587b7af8001adbbb3dd1eff18a13c2e958e64dc87c",
  "HMW_219": "8eb4414f5a1f69fe1013ee0b4bcb644aacec9e84165ccf0a863b105ff35abc96",
  "HMW_258": "bb9a2eddf056f27a18763ccc6aeeabcbbd4e3d97287249fe6ca00d307bec03a8",
  "SOR_014": "d1ca8396901fac3eb4eafafb55439deeadf8b385f6e195faa781a786568c2bac",
  "SOR_023": "a694a128d3fb4bb0a4e465dcf906cb2afd3a936c7d99ee1f8cf53db1a7cb41b6",
  "SOR_095": "27b538f7ef020583c610281275a75f2187beb15ef5946d59a18809d53d686d3d",
  "SOR_178": "d70e91da87c650d7c3ccc91ad3b6c67f36a04ece2570dd224764053daf5f22e4",
  "TWI_175": "cf875f9ee08c1da4c60c4b192bff19e93f274259b5c27f2590325579f27c8e0d",
  "TWI_226": "abca6b40e5fe7d36f6c4caa12ae501cc3ccfabaa26970aeee550fff3c727d80e",
  "IC27_184": "f8a03546457954045b8c2bda94c19c69dce3ad292a092e689775627d8d132989",
  "IC27_052": "fec00f15679fe14298b2d95d5ec70c80eee187c77e2cd9d590a4eb62b324fb12",
  "TOKEN_ADVANTAGE": "be7663af45ebe560c97cbe5b1305863b4dcdf6bd4d594ef333643b3fd1fb7234",
  "TOKEN_SHIELD": "a8a34f754731a5dc9de991d5476b9491466190ac7521e0c7192c9210fc5538a9",
  "TOKEN_EXPERIENCE": "275c2c34e4ea190b9c681ea857f3f8bb13877e2373f6309cea717b3cf4eeb9f8",
  "HMW_097": "156bc2cebbf7722fdd95e9d297ca7d682888dbe439ef9870937f4939e3d9d0ac",
  "TOKEN_WEAKNESS": "e2044669df19b9534392e1b1ccfe31a1fa3cdccedd63c39bfdecd50594e0b823"
};
const fillerCodes=["ASH_061","ASH_131","ASH_190","ASH_201","ASH_242","ASH_261","HMW_082","HMW_093","HMW_116","HMW_128","HMW_135","HMW_155","HMW_163","HMW_174","HMW_178","HMW_219","HMW_258"];
function classifyCollectiveTrigger(entry){
    if(!entry||typeof entry!=='object'||Array.isArray(entry)||typeof entry.setCode!=='string')return null;
    const i=reviewedProfiles.findIndex(p=>same(profile(entry),profile(p)));
    return i<0?null:{family:'collective-trigger',mode:i===0?'draw':'attachment'};
}
function planCollectiveTriggerProbes(entry,options={}){
    const shape=classifyCollectiveTrigger(entry);if(!shape)return null;
    const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name,reason){const id=entry.setCode+':collective-trigger:'+name;result.obligations.push({id,path:'collective-trigger',dimension:name});if(reason)result.uncovered.push({id,path:'collective-trigger',reason});return id;}
    function donor(code){const rows=catalogue.filter(e=>e.setCode===code);return rows.length===1&&fingerprint(rows[0])===reviewedDonors[code];}
    obligation('source-review','Planner shapes do not establish source lowering, ability/window collective identity, intrinsic lifecycle or completed native/Wasm comparisons');
    const common=['SOR_014','SOR_023','SOR_095','SOR_178',...fillerCodes,...(shape.mode==='draw'?['TOKEN_ADVANTAGE']:['TOKEN_SHIELD','TOKEN_EXPERIENCE'])];
    const scenarios=shape.mode==='draw'?{
        'paid-play-draw3':['TWI_175'],
        'sequential-draw-windows':['TWI_175'],
        'opponent-draw-negative':['TWI_175'],
        'natural-regroup-draw':[],
        'empty-deck-fatigue':['TWI_175'],
        'source-gone-before-draw':['TWI_226','TWI_175'],
        'both-owners-regroup':[]
    }:{
        'shielded-play-own-ground':[],
        'shielded-play-enemy-ground':[],
        'shielded-decline':[],
        'shielded-space-negative':[],
        'shielded-exhausted-ground':[],
        'multi-xp-one-window':['IC27_184'],
        'sequential-upgrade-windows':['IC27_052'],
        'other-parent-negative':['IC27_184'],
        'phase-reset-second-window':['IC27_052'],
        'source-gone-before-attach':['TWI_226','IC27_184'],
        'unique-keep-old':[],
        'unique-keep-incoming':[]
    };
    for(const owner of [0,1])for(const [scenario,extra]of Object.entries(scenarios)){
        const name='owner-'+owner+'-'+scenario,setupCardIds=[...new Set([...common,...extra])].sort();
        if(!setupCardIds.every(donor)){obligation(name,'A complete admitted donor, native token, default leader/base or filler definition is absent, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario,resourceCount:24,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'SOR_023',enemyBaseCode:'SOR_023',fillerCodes:[...fillerCodes]};
        result.cases.push({id:'auto/'+entry.setCode+'/collective-trigger-'+name,name:'collective-trigger-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['collective-trigger',shape.mode==='draw'?'on-cards-drawn':'on-upgrade-attached'],assertionStages:['construction','activation','observation','terminal'],coverageObligationIds:[id],probe});
    }
    // Append the actual paid-Dire-Prowess interaction after all original cases;
    // the original thirty-eight case objects and their ordering stay intact.
    if(shape.mode==='attachment')for(const scenario of ['weakness-source-death','same-action-sequential-attachment-windows'])for(const owner of [0,1]){
        const name='owner-'+owner+'-'+scenario;
        const setupCardIds=[...new Set([...common,'HMW_097','TOKEN_WEAKNESS'])].sort();
        if(!setupCardIds.every(donor)){obligation(name,'A complete admitted Dire Prowess/Weakness or common construction dependency is absent, changed or ambiguous');continue;}
        const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario,resourceCount:24,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'SOR_023',enemyBaseCode:'SOR_023',fillerCodes:[...fillerCodes]};
        result.cases.push({id:'auto/'+entry.setCode+'/collective-trigger-'+name,name:'collective-trigger-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['collective-trigger','on-upgrade-attached'],assertionStages:['construction','activation','observation','terminal'],coverageObligationIds:[id],probe});
    }
    const gaps=shape.mode==='draw'?['single-action-sequential-draw-windows','missing-event-window-fallback','source-controller-transfer','competing-nested-draw-windows']:['competing-distinct-abilities-one-source','missing-event-window-fallback','source-controller-transfer'];
    for(const name of gaps)obligation(name,'No sourced actual admitted construction; separate external actions are not proof of separate windows within one action, and synthetic events receive no gameplay credit');
    return result;
}
module.exports={classifyCollectiveTrigger,planCollectiveTriggerProbes};
