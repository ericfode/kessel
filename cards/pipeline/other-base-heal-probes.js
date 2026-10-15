'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "TS26_042",
  "sourcePath": "game/cards/07_TS26/units/ReliefFrigate.ts",
  "sourceHash": "3206877280818e87690d2017e036a00a23540d39dde6618f7dfdd58d0a84b236",
  "className": "ReliefFrigate",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "Relief Frigate",
    "subtitle": "",
    "cost": 5,
    "hp": 7,
    "power": 3,
    "text": "When Played: Choose a base. Heal 3 damage from each other base.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "9458579327",
    "aspects": [
      "vigilance"
    ],
    "traits": [
      "republic",
      "vehicle",
      "capital ship"
    ],
    "arena": "space",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "TS26",
      "number": 42
    },
    "setCodes": [
      {
        "set": "TS26",
        "number": 42
      }
    ],
    "internalName": "relief-frigate"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Choose a base. Heal 3 damage from each other base",
        "costs": [],
        "targetResolver": {
          "owner": "any",
          "type": "base",
          "zone": "anyAttackable"
        },
        "effect": {
          "kind": "heal",
          "target": {
            "kind": "conditional",
            "condition": {
              "kind": "binary",
              "op": "===",
              "left": {
                "kind": "ref",
                "path": [
                  "target"
                ]
              },
              "right": {
                "kind": "ref",
                "path": [
                  "player",
                  "base"
                ]
              }
            },
            "then": {
              "kind": "ref",
              "path": [
                "opponent",
                "base"
              ]
            },
            "else": {
              "kind": "ref",
              "path": [
                "player",
                "base"
              ]
            }
          },
          "amount": {
            "kind": "const",
            "value": 3
          }
        }
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": null,
  "piloting": null
};
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planOtherBaseHealProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=[];
 for(const sourceSeat of [1,2])for(const [ownDamage,enemyDamage]of [[5,5],[0,5],[5,0],[0,0],[1,5],[5,1]])for(const chosenBase of ['own','enemy']) {
  const name='relief-chosen-'+chosenBase+'-damage-'+ownDamage+'-'+enemyDamage+'-seat'+sourceSeat;
  const obligation=entry.setCode+':other-base-heal:'+name;
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds:['ASH_019','SOR_014'],behaviorIds:['other-base-heal','chosen-vs-effective-target','paid-entry'],assertionStages:['paid-entry','exact-base-menu','opposite-base-healing','terminal-continuation'],coverageObligationIds:[obligation],probe:{family:'relief-frigate',cardId:entry.setCode,sourceSeat,chosenBase,ownDamage,enemyDamage}});
 }
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'other-base-heal',dimension:c.name}));
 const uncovered=[['source-departure-and-replacement','No actual source departure, unique replacement, or controller-transfer interleaving; source itself is nonunique.'],['healing-restrictions-and-concurrent-modifiers','No construction covers every healing restriction, prevention or concurrent modifier priority.'],['more-than-two-players','Runtime is explicitly two-player; additional bases in multiplayer games are outside this supported domain.'],['fatigue-boundaries','No source draw, discard, resource or deck-dependent operation; fatigue is irrelevant to the new heal rule and receives no coverage credit.']].map(([dimension,reason])=>{const id=entry.setCode+':other-base-heal:'+dimension;obligations.push({id,path:'other-base-heal',dimension});return{id,path:'other-base-heal',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planOtherBaseHealProbes};
