'use strict';
// Bounded whole-source upgrade-presence witnesses; complete independent source review remains required.
const same=require('node:util').isDeepStrictEqual;
const shapes=[
  {
    "setCode": "LOF_244",
    "sourcePath": "game/cards/05_LOF/units/JediVector.ts",
    "sourceHash": "4652e90baa0aa083ce5e6b16e2b4176299f30b2ac0f3da28e2a83aed39df512e",
    "className": "JediVector",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Jedi Vector",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 1,
      "text": "This unit gets +1/+0 if you control another Jedi unit and +1/+0 if you control a Lightsaber upgrade.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0035741177",
      "aspects": [
        "heroism"
      ],
      "traits": [
        "jedi",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "LOF",
        "number": 244
      },
      "setCodes": [
        {
          "set": "LOF",
          "number": 244
        }
      ],
      "internalName": "jedi-vector"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "constant",
          "title": "While you control another Jedi unit, this unit gets +1/+0",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "self",
            "duration": "persistent",
            "power": {
              "kind": "const",
              "value": 1
            },
            "hp": {
              "kind": "const",
              "value": 0
            }
          },
          "condition": {
            "kind": "query",
            "name": "hasSomeArenaUnit",
            "receiver": {
              "kind": "ref",
              "path": [
                "player"
              ]
            },
            "args": [
              {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "const",
                    "value": true
                  },
                  "right": {
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
                        "value": "jedi"
                      }
                    ]
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "||",
                  "left": {
                    "kind": "unary",
                    "op": "!",
                    "value": {
                      "kind": "ref",
                      "path": [
                        "source"
                      ]
                    }
                  },
                  "right": {
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
                  }
                }
              }
            ]
          }
        },
        {
          "trigger": "constant",
          "title": "While you control a Lightsaber upgrade, this unit gets +1/+0",
          "costs": [],
          "effect": {
            "kind": "modifyStats",
            "target": "self",
            "duration": "persistent",
            "power": {
              "kind": "const",
              "value": 1
            },
            "hp": {
              "kind": "const",
              "value": 0
            }
          },
          "condition": {
            "kind": "query",
            "name": "hasSomeArenaCard",
            "receiver": {
              "kind": "ref",
              "path": [
                "player"
              ]
            },
            "args": [
              {
                "kind": "binary",
                "op": "&&",
                "left": {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "const",
                    "value": true
                  },
                  "right": {
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
                        "value": "lightsaber"
                      }
                    ]
                  }
                },
                "right": {
                  "kind": "query",
                  "name": "isUpgrade",
                  "receiver": {
                    "kind": "ref",
                    "path": [
                      "card"
                    ]
                  },
                  "args": []
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
  },
  {
    "setCode": "SEC_095",
    "sourcePath": "game/cards/06_SEC/units/TheedSecurity.ts",
    "sourceHash": "7124bce4998e88cc7f1c4b8dc8f650c1703bd2fc20b43157f92c01defe05397c",
    "className": "TheedSecurity",
    "baseClass": "NonLeaderUnitCard",
    "card": {
      "title": "Theed Security",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 2,
      "text": "When Played: If an opponent controls an upgrade, give an Experience token to a unit.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9857313649",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "naboo",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SEC",
        "number": 95
      },
      "setCodes": [
        {
          "set": "SEC",
          "number": 95
        }
      ],
      "internalName": "theed-security"
    },
    "abilities": {
      "leader": [],
      "unit": [],
      "card": [
        {
          "trigger": "whenPlayed",
          "title": "Give an Experience token to a unit",
          "costs": [],
          "targetResolver": {
            "owner": "any",
            "type": "unit",
            "zone": "anyAttackable"
          },
          "effect": {
            "kind": "conditional",
            "condition": {
              "kind": "query",
              "name": "hasSomeArenaCard",
              "receiver": {
                "kind": "ref",
                "path": [
                  "opponent"
                ]
              },
              "args": [
                {
                  "kind": "binary",
                  "op": "&&",
                  "left": {
                    "kind": "const",
                    "value": true
                  },
                  "right": {
                    "kind": "query",
                    "name": "isUpgrade",
                    "receiver": {
                      "kind": "ref",
                      "path": [
                        "card"
                      ]
                    },
                    "args": []
                  }
                }
              ]
            },
            "onTrue": {
              "kind": "giveExperience",
              "target": "chosen",
              "amount": {
                "kind": "const",
                "value": 1
              }
            },
            "onFalse": {
              "kind": "noAction"
            }
          }
        }
      ]
    },
    "keywordValues": {},
    "attachCondition": null,
    "piloting": null
  }
];
const designs=[
  {
    "cardId": "LOF_244",
    "variant": "paid-no-upgrade-self-and-enemy-jedi-excluded",
    "enemyJedi": true
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-own-saber-friendly-ground-host",
    "upgrade": "saber",
    "upgradeOwner": "self",
    "hostOwner": "self",
    "hostArena": "ground"
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-own-saber-enemy-space-host",
    "upgrade": "saber",
    "upgradeOwner": "self",
    "hostOwner": "opponent",
    "hostArena": "space"
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-enemy-saber-friendly-ground-host",
    "upgrade": "saber",
    "upgradeOwner": "opponent",
    "hostOwner": "self",
    "hostArena": "ground"
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-own-non-lightsaber-upgrade",
    "upgrade": "ordinary",
    "upgradeOwner": "self",
    "hostOwner": "self",
    "hostArena": "space"
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-own-token-upgrades-are-not-lightsabers",
    "tokens": [
      "shield",
      "experience",
      "weakness"
    ],
    "upgradeOwner": "self",
    "hostOwner": "self",
    "hostArena": "ground"
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-offboard-lightsaber-excluded",
    "offboardSaber": true
  },
  {
    "cardId": "LOF_244",
    "variant": "paid-native-force-in-base-excluded",
    "force": true
  },
  {
    "cardId": "LOF_244",
    "variant": "actual-saber-attach-return-keeps-independent-jedi",
    "otherJedi": true,
    "lifecycle": "saber"
  },
  {
    "cardId": "LOF_244",
    "variant": "actual-other-jedi-leaves-keeps-saber",
    "otherJedi": true,
    "upgrade": "saber",
    "upgradeOwner": "self",
    "hostOwner": "opponent",
    "hostArena": "space",
    "lifecycle": "jedi"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-no-upgrade-no-experience"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-own-upgrade-on-enemy-space-host-no-experience",
    "upgrade": "ordinary",
    "upgradeOwner": "self",
    "hostOwner": "opponent",
    "hostArena": "space"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-enemy-upgrade-on-friendly-ground-host-self-target",
    "upgrade": "ordinary",
    "upgradeOwner": "opponent",
    "hostOwner": "self",
    "hostArena": "ground",
    "chosen": "source"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-enemy-upgrade-space-host-enemy-target",
    "upgrade": "ordinary",
    "upgradeOwner": "opponent",
    "hostOwner": "opponent",
    "hostArena": "space",
    "chosen": "enemy"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-opponent-shield-counts",
    "tokens": [
      "shield"
    ],
    "upgradeOwner": "opponent",
    "hostOwner": "opponent",
    "hostArena": "ground"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-opponent-experience-counts",
    "tokens": [
      "experience"
    ],
    "upgradeOwner": "opponent",
    "hostOwner": "opponent",
    "hostArena": "space"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-opponent-weakness-counts",
    "tokens": [
      "weakness"
    ],
    "upgradeOwner": "opponent",
    "hostOwner": "opponent",
    "hostArena": "ground"
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-opponent-shield-native-unit-target-includes-deployed-leader",
    "tokens": [
      "shield"
    ],
    "upgradeOwner": "opponent",
    "hostOwner": "opponent",
    "hostArena": "ground",
    "chosen": "token",
    "extraTargetTypes": true
  },
  {
    "cardId": "SEC_095",
    "variant": "paid-opponent-native-force-in-base-excluded",
    "force": true
  }
];
const donors={
  "leader": "SOR_014",
  "ground": "SEC_028",
  "space": "LOF_119",
  "jedi": "LOF_093",
  "saber": "LOF_053",
  "ordinary": "SOR_057",
  "returning": "SHD_209",
  "bounce": "TWI_226"
};
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planUpgradePresenceProbes(entry,options={}){
 if(entry.status!=='supported'||entry.diagnostics?.length||!shapes.some(s=>same(s,normalize(entry))))return null;
 const obligations=[],cases=[];
 for(const design of designs.filter(d=>d.cardId===entry.setCode))for(const sourceSeat of[1,2]){
  const name='upgrade-presence-'+design.variant+'-seat'+sourceSeat,id=entry.setCode+':upgrade-presence:'+design.variant+'-seat'+sourceSeat;
  obligations.push({id,path:'upgrade-presence',dimension:design.variant+'-seat'+sourceSeat});
  cases.push({id:'auto/'+entry.setCode+'/'+name,name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,...Object.values(donors),'TOKEN_SHIELD','TOKEN_EXPERIENCE','TOKEN_WEAKNESS','TOKEN_THE_FORCE','TOKEN_BATTLE_DROID'])].sort(),behaviorIds:['upgrade-presence','live-controller-scope','actual-paid-source'],assertionStages:['construction','actual-paid-play','exact-query-and-target-outcome','terminal-continuation'],coverageObligationIds:[id],probe:{...design,family:'upgrade-presence',sourceSeat,resourceCount:24,donors}});
 }
 const uncovered=[['source-semantics','Finite witnesses do not prove completeness of source lowering'],['unconstructed-controller-transfer-capture-pilots','Controller transfers,captured sources,pilots and concurrent replacement/priority interactions are not constructed']].map(([dimension,reason])=>{const id=entry.setCode+':upgrade-presence:'+dimension;obligations.push({id,path:'upgrade-presence',dimension});return{id,path:'upgrade-presence',reason};});
 return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planUpgradePresenceProbes};
