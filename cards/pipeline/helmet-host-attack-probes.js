'use strict';
const same=require('node:util').isDeepStrictEqual;
// Complete source, card metadata, attachment restriction and gained HOST program.
// The source oracle uses HOST as source/player; the upgrade is physical provenance.
const shape={
  "setCode": "LAW_186",
  "sourcePath": "game/cards/07_LAW/upgrades/EnfysNestsHelmet.ts",
  "sourceHash": "bfacc2fce56f4d638aae84292e5851601105df245a53ffdfcf453a95cf53f2da",
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "onAttack",
        "title": "Give another unit +3/+0 for this phase",
        "costs": [],
        "optional": true,
        "targetResolver": {
          "owner": "any",
          "type": "any",
          "zone": "anyAttackable",
          "predicate": {
            "kind": "binary",
            "op": "&&",
            "left": {
              "kind": "binary",
              "op": "!==",
              "left": {
                "kind": "ref",
                "path": [
                  "card"
                ]
              },
              "right": {
                "kind": "ref",
                "path": [
                  "source"
                ]
              }
            },
            "right": {
              "kind": "query",
              "name": "isUnit",
              "receiver": {
                "kind": "ref",
                "path": [
                  "card"
                ]
              },
              "args": []
            }
          }
        },
        "effect": {
          "kind": "modifyStats",
          "target": "chosen",
          "duration": "phase",
          "power": {
            "kind": "const",
            "value": 3
          },
          "hp": {
            "kind": "const",
            "value": 0
          }
        },
        "gainedHost": true
      }
    ]
  },
  "diagnostics": [],
  "className": "EnfysNestsHelmet",
  "baseClass": "UpgradeCard",
  "id": "9472544840",
  "internalName": "enfys-nests-helmet",
  "attachCondition": {
    "kind": "unary",
    "op": "!",
    "value": {
      "kind": "query",
      "name": "hasSomeTrait",
      "receiver": {
        "kind": "ref",
        "path": [
          "card"
        ]
      },
      "args": [
        {
          "kind": "const",
          "value": "vehicle"
        }
      ]
    }
  },
  "status": "supported",
  "card": {
    "title": "Enfys Nest's Helmet",
    "subtitle": "",
    "cost": 2,
    "hp": 2,
    "power": 0,
    "text": "Attach to a non-Vehicle unit.\nAttached unit gains: “On Attack: You may give another unit +3/+0 for this phase.”",
    "deployBox": null,
    "epicAction": "",
    "unique": true,
    "rules": null,
    "upgradePower": 0,
    "upgradeHp": 2,
    "id": "9472544840",
    "aspects": [
      "aggression",
      "heroism"
    ],
    "traits": [
      "item",
      "armor"
    ],
    "keywords": [],
    "types": [
      "upgrade"
    ],
    "setId": {
      "set": "LAW",
      "number": 186
    },
    "setCodes": [
      {
        "set": "LAW",
        "number": 186
      }
    ],
    "internalName": "enfys-nests-helmet"
  },
  "dataHash": "81cdcc1572dcff87447e940b9e8a5ab05dfffb54df8c5b7e916d1e69ff7d5a96",
  "keywordValues": {},
  "requiredCapabilities": [
    "binary:!==",
    "binary:&&",
    "cardType:upgrade",
    "ir:binary",
    "ir:const",
    "ir:modifyStats",
    "ir:query",
    "ir:ref",
    "ir:unary",
    "query:hasSomeTrait",
    "query:isUnit",
    "ref:card",
    "ref:source",
    "targetType:any",
    "targetZone:anyAttackable",
    "trigger:onAttack",
    "unary:!"
  ]
};
const u=(key,code,owner='own',damage=0)=>({key,code,owner,damage}),specs=[],title='Give another unit +3/+0 for this phase';
function add(tag,fields={}){const hostOwner=fields.hostOwner??'own',space=!!fields.space,host=fields.hostCode??(space?'LOF_119':'HMW_082');let units=fields.units??[u('host',host,hostOwner,fields.hostDamage??0),u('friendlyGround','SEC_028'),u('enemyGround','IBH_007','enemy'),u('friendlySpace','JTL_212'),u('enemySpace','LOF_119','enemy')];
 const target=fields.target??'friendlyGround',actor=hostOwner==='enemy'?'enemy':undefined,attackBase=hostOwner==='enemy'?'ownBase':'enemyBase',common={family:'helmet-host-attack',focus:'LAW_186',tag,hostOwner,hostCode:host,space,target,leaderCode:'SOR_014',baseCode:'SOR_021',baseDamage:0,units,plan:[{code:'LAW_186',target:'host'},{code:host,from:'host',actor,target:attackBase}],...fields};specs.push(common);}
for(const hostOwner of ['own','enemy'])for(const space of [false,true])for(const target of ['friendlyGround','enemyGround','friendlySpace','enemySpace'])add(hostOwner+'-'+(space?'space':'ground')+'-'+target,{hostOwner,space,target});
for(const hostOwner of ['own','enemy']){add(hostOwner+'-decline',{hostOwner,decline:true});add(hostOwner+'-no-other-unit',{hostOwner,units:[u('host','HMW_082',hostOwner)],noOther:true});}
for(const token of ['battleDroid','tieFighter'])add('native-target-'+token,{tokens:[{key:'nativeTarget',name:token,owner:'enemy',arena:token==='tieFighter'?'space':'ground'}],target:'nativeTarget'});
for(const hostOwner of ['own','enemy'])add('deployed-leader-target-'+hostOwner,{hostOwner,deployedLeader:true,target:'leader'});
for(const hostOwner of ['own','enemy'])add('native-host-'+hostOwner,{hostOwner,hostToken:true,units:[u('friendlyGround','SEC_028'),u('enemyGround','IBH_007','enemy'),u('enemySpace','LOF_119','enemy')],tokens:[{key:'host',name:'battleDroid',owner:hostOwner}],hostCode:'HMW_082'});
for(const hostOwner of ['own','enemy'])add('unique-same-parent-'+hostOwner,{hostOwner,unique:'same-parent',plan:[{code:'LAW_186',target:'host'},{code:'LAW_186',target:'host'},{code:'HMW_082',from:'host',actor:hostOwner==='enemy'?'enemy':undefined,target:hostOwner==='enemy'?'ownBase':'enemyBase'}]});
for(const keep of ['old','new'])add('unique-different-parents-keep-'+keep,{unique:keep,units:[u('host','HMW_082'),u('secondHost','IBH_007','enemy'),u('friendlyGround','SEC_028')],plan:[{code:'LAW_186',target:'host'},{code:'LAW_186',target:'secondHost'},...(keep==='old'?[{code:'HMW_082',from:'host',target:'enemyBase'},{code:'IBH_007',from:'secondHost',actor:'enemy',target:'ownBase'}]:[{code:'IBH_007',from:'secondHost',actor:'enemy',target:'ownBase'},{code:'HMW_082',from:'host',target:'enemyBase'}])],activeHost:keep==='old'?'host':'secondHost'});
for(const hostOwner of ['own','enemy'])for(const order of ['remove-first','buff-first'])for(const hostDies of [false,true])add(hostOwner+'-'+order+'-'+(hostDies?'host-dies':'upgrade-dies'),{hostOwner,hostCode:'JTL_168',hostDamage:hostDies?4:0,removeOnAttack:true,hostDies,followup:!hostDies,order,units:[u('host','JTL_168',hostOwner,hostDies?4:0),u('friendlyGround','SEC_028'),u('enemyGround','IBH_007','enemy')],plan:[{code:'LAW_186',target:'host'},...(hostDies?[{code:'TWI_170',target:'host'}]:[]),{code:'JTL_168',from:'host',actor:hostOwner==='enemy'?'enemy':undefined,target:hostOwner==='enemy'?'ownBase':'enemyBase'}]});
for(const hostOwner of ['own','enemy'])for(const order of ['draw-first','buff-first'])add(hostOwner+'-printed-draw-'+order,{hostOwner,hostCode:'LAW_107',printedDraw:true,order});
for(const removal of ['upgrade','host'])for(const hostOwner of ['own','enemy'])add(hostOwner+'-after-buff-remove-'+removal,{hostOwner,hostCode:'SEC_028',hostDamage:2,afterRemoval:removal,plan:[{code:'LAW_186',target:'host'},{code:'SEC_028',from:'host',actor:hostOwner==='enemy'?'enemy':undefined,target:hostOwner==='enemy'?'ownBase':'enemyBase'},{code:removal==='upgrade'?'SHD_262':'SOR_077',target:removal==='upgrade'?'helmet':'host'}]});
add('cancel-attachment',{cancel:true});add('no-nonvehicle-attachment',{invalid:true,units:[u('host','ASH_261')],plan:[],initialHandCodes:['LAW_186']});
const scenarios=specs.flatMap(s=>[0,1].map(ownSeat=>({...s,ownSeat,name:'LAW_186-'+s.tag+'-p'+(ownSeat+1),actualScriptedCardIds:s.invalid||s.cancel?[]:['LAW_186'],actualAbilityCardIds:s.invalid||s.cancel?[]:['LAW_186']})));

function buildCases(){return scenarios.map(raw=>{const design=JSON.parse(JSON.stringify(raw));const name=design.name;const tokenCodes=(design.tokens??[]).map(t=>t.name==='tieFighter'?'TOKEN_TIE_FIGHTER':'TOKEN_BATTLE_DROID');return{id:'auto/LAW_186/'+name.toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,''),name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:['LAW_186'],setupCardIds:[...new Set([design.leaderCode,design.baseCode,...design.units.map(u=>u.code),...design.plan.map(s=>s.code),...(design.initialHandCodes??[]),...tokenCodes].filter(code=>code&&code!=='LAW_186'))],behaviorIds:['gained-host-source-controller','any-other-unit-phase-power','unique-upgrade-attachment','captured-grant-lifecycle','printed-on-attack-order'],assertionStages:['actual-cost-and-attachment-menu','host-declaration-and-physical-grantor','optional-target-or-pass','captured-context-and-trigger-order','phase-expiry-and-later-attack','complete-terminal-game'],coverageObligationIds:['LAW_186:helmet-host-attack:'+name],probe:{family:'helmet-host-attack',scenarioName:name,design}};});}
function planHelmetHostAttackProbes(entry){
 if(!entry||entry.status!=='supported'||entry.diagnostics?.length||!same(entry,shape))return null;
 const cases=buildCases();const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'helmet-host-attack',dimension:c.name}));
 const uncovered=[['control-change','Arbitrary control/owner divergence and controller changes after capture remain uncredited; enemy HOST owner/controller differs from upgrade payer.'],['other-grant-and-owner-order','Uniqueness prevents duplicate live Helmet grants. Nonunique grants, every competing OnDefense/keyword owner order and retroactive grant changes remain uncredited beyond the selected printed triggers and later attack without the removed grant.'],['capture-reentry-and-capacity','Capture/rescue/re-entry, arbitrary replacement/restriction and global capacity remain uncredited.']].map(([dimension,reason])=>{const id=entry.setCode+':helmet-host-attack:'+dimension;obligations.push({id,path:'helmet-host-attack',dimension});return{id,path:'helmet-host-attack',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planHelmetHostAttackProbes};
