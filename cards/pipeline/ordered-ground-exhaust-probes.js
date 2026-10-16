'use strict';
const same=require('node:util').isDeepStrictEqual;
const shape={
  "setCode": "IC27_027",
  "sourcePath": "game/cards/01_SOR/units/AT-ATSuppressor.ts",
  "sourceHash": "6f07a2716913f9f408fc34599cc1dc2cce9d7979237bc664fe0c80f93ceaeeeb",
  "className": "ATATSuppressor",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "AT-AT Suppressor",
    "subtitle": null,
    "cost": 8,
    "hp": 8,
    "power": 8,
    "text": "When Played: Exhaust all ground units.",
    "deployBox": null,
    "epicAction": null,
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "6663619377",
    "aspects": [
      "villainy",
      "vigilance"
    ],
    "traits": [
      "imperial",
      "vehicle",
      "walker"
    ],
    "arena": "ground",
    "keywords": [],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "SOR",
      "number": 39
    },
    "setCodes": [
      {
        "set": "SOR",
        "number": 39
      }
    ],
    "internalName": "atat-suppressor"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenPlayed",
        "title": "Exhaust all ground units",
        "costs": [],
        "effect": {
          "kind": "exhaust",
          "target": {
            "kind": "arenaList",
            "receiver": {
              "kind": "ref",
              "path": [
                "game"
              ]
            },
            "predicate": {
              "kind": "binary",
              "op": "&&",
              "left": {
                "kind": "const",
                "value": true
              },
              "right": {
                "kind": "binary",
                "op": "||",
                "left": {
                  "kind": "unary",
                  "op": "!",
                  "value": {
                    "kind": "const",
                    "value": "groundArena"
                  }
                },
                "right": {
                  "kind": "binary",
                  "op": "||",
                  "left": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "const",
                      "value": "groundArena"
                    },
                    "right": {
                      "kind": "const",
                      "value": "anyArena"
                    }
                  },
                  "right": {
                    "kind": "binary",
                    "op": "===",
                    "left": {
                      "kind": "ref",
                      "path": [
                        "card",
                        "zoneName"
                      ]
                    },
                    "right": {
                      "kind": "const",
                      "value": "groundArena"
                    }
                  }
                }
              }
            },
            "controllerOrder": "opponentFirst"
          }
        }
      }
    ]
  },
  "keywordValues": {},
  "attachCondition": null,
  "piloting": null
};
const designs=[
  {
    "name": "IC27_027-thirty-five-ground-recipients-no-truncation-p1",
    "capacity": true,
    "units": [
      {
        "key": "f0",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e0",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f1",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f3",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e3",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f4",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e4",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f5",
        "code": "HMW_116",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e5",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f6",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e6",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f7",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e7",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f8",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e8",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f9",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e9",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f10",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e10",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f11",
        "code": "HMW_116",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e11",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f12",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e12",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f13",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e13",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f14",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e14",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f15",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e15",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f16",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e16",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-thirty-five-ground-recipients-no-truncation-p2",
    "capacity": true,
    "units": [
      {
        "key": "f0",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e0",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f1",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f3",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e3",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f4",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e4",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f5",
        "code": "HMW_116",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e5",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f6",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e6",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f7",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e7",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f8",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e8",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f9",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e9",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f10",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e10",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f11",
        "code": "HMW_116",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e11",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f12",
        "code": "ASH_061",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e12",
        "code": "ASH_061",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f13",
        "code": "ASH_131",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e13",
        "code": "ASH_131",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f14",
        "code": "ASH_242",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e14",
        "code": "ASH_242",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f15",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e15",
        "code": "ASH_261",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f16",
        "code": "HMW_093",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e16",
        "code": "HMW_093",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-interleaved-ready-arrivals-p1",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-interleaved-ready-arrivals-p2",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-reverse-interleaved-ready-arrivals-p1",
    "units": [
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-reverse-interleaved-ready-arrivals-p2",
    "units": [
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-only-ready-ground-p1",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-only-ready-ground-p2",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-only-ready-ground-p1",
    "units": [
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-only-ready-ground-p2",
    "units": [
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-mixed-ready-cancelled-opponent-and-own-p1",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f3",
        "code": "HMW_135",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e3",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-mixed-ready-cancelled-opponent-and-own-p2",
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f3",
        "code": "HMW_135",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e3",
        "code": "HMW_116",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-all-ground-already-exhausted-p1",
    "noEffect": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-all-ground-already-exhausted-p2",
    "noEffect": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": true,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": true,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-source-alone-no-effective-exhaust-p1",
    "noEffect": true,
    "units": [],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-source-alone-no-effective-exhaust-p2",
    "noEffect": true,
    "units": [],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-only-space-distractors-p1",
    "noEffect": true,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "space",
        "name": "xwing",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-only-space-distractors-p2",
    "noEffect": true,
    "units": [
      {
        "key": "f",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "space",
        "name": "xwing",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ready-native-ground-tokens-and-space-exclusion-p1",
    "units": [
      {
        "key": "fSpace",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "eSpace",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "fToken",
        "name": "battleDroid",
        "exhausted": false
      },
      {
        "key": "eToken",
        "name": "cloneTrooper",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "fSpaceToken",
        "name": "xwing",
        "arena": "space",
        "exhausted": false
      },
      {
        "key": "eSpaceToken",
        "name": "tieFighter",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ready-native-ground-tokens-and-space-exclusion-p2",
    "units": [
      {
        "key": "fSpace",
        "code": "JTL_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "eSpace",
        "code": "LOF_119",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "fToken",
        "name": "battleDroid",
        "exhausted": false
      },
      {
        "key": "eToken",
        "name": "cloneTrooper",
        "owner": "enemy",
        "exhausted": false
      },
      {
        "key": "fSpaceToken",
        "name": "xwing",
        "arena": "space",
        "exhausted": false
      },
      {
        "key": "eSpaceToken",
        "name": "tieFighter",
        "arena": "space",
        "owner": "enemy",
        "exhausted": false
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-exhausted-native-ground-tokens-cancel-p1",
    "units": [
      {
        "key": "ready",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "fToken",
        "name": "battleDroid",
        "exhausted": true
      },
      {
        "key": "eToken",
        "name": "cloneTrooper",
        "owner": "enemy",
        "exhausted": true
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-exhausted-native-ground-tokens-cancel-p2",
    "units": [
      {
        "key": "ready",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "tokens": [
      {
        "key": "fToken",
        "name": "battleDroid",
        "exhausted": true
      },
      {
        "key": "eToken",
        "name": "cloneTrooper",
        "owner": "enemy",
        "exhausted": true
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-predeployed-leader-ready-p1",
    "deployedLeader": true,
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-predeployed-leader-ready-p2",
    "deployedLeader": true,
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-actual-deploy-ready-leader-p1",
    "actualLeader": "own",
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-actual-deploy-ready-leader-p2",
    "actualLeader": "own",
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-actual-deploy-ready-leader-p1",
    "actualLeader": "enemy",
    "enemyLeader": true,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-actual-deploy-ready-leader-p2",
    "actualLeader": "enemy",
    "enemyLeader": true,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-actual-deploy-attack-exhausted-leader-p1",
    "actualLeader": "own",
    "attackLeader": true,
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "target": "enemyBase"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-own-actual-deploy-attack-exhausted-leader-p2",
    "actualLeader": "own",
    "attackLeader": true,
    "units": [
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "target": "enemyBase"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-actual-deploy-attack-exhausted-leader-p1",
    "actualLeader": "enemy",
    "enemyLeader": true,
    "attackLeader": true,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy",
        "target": "ownBase"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-enemy-actual-deploy-attack-exhausted-leader-p2",
    "actualLeader": "enemy",
    "enemyLeader": true,
    "attackLeader": true,
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy"
      },
      {
        "code": "SOR_014",
        "from": "leader",
        "actor": "enemy",
        "target": "ownBase"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ordinary-second-copy-no-uniqueness-p1",
    "units": [
      {
        "key": "copy",
        "code": "IC27_027",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ordinary-second-copy-no-uniqueness-p2",
    "units": [
      {
        "key": "copy",
        "code": "IC27_027",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-existing-native-shields-preserved-p1",
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 2
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 1
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-existing-native-shields-preserved-p2",
    "units": [
      {
        "key": "f",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 2
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 1
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-bounce-and-replay-physical-arrival-p1",
    "reentry": true,
    "units": [
      {
        "key": "late",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "other",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "TWI_226",
        "target": "late"
      },
      {
        "code": "HMW_082",
        "from": "late"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-bounce-and-replay-physical-arrival-p2",
    "reentry": true,
    "units": [
      {
        "key": "late",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "other",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "TWI_226",
        "target": "late"
      },
      {
        "code": "HMW_082",
        "from": "late"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-one-shot-regroup-readies-again-p1",
    "regroup": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-one-shot-regroup-readies-again-p2",
    "regroup": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-live-sentinel-and-raid-conditions-p1",
    "liveKeywords": true,
    "units": [
      {
        "key": "guardian",
        "code": "HMW_259",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "sage",
        "code": "LOF_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-live-sentinel-and-raid-conditions-p2",
    "liveKeywords": true,
    "units": [
      {
        "key": "guardian",
        "code": "HMW_259",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "sage",
        "code": "LOF_212",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "enemy",
        "code": "HMW_082",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-matched-aspect-printed-payment-eight-p1",
    "leaderCode": "SOR_007",
    "baseCode": "ASH_019",
    "matchedPayment": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-matched-aspect-printed-payment-eight-p2",
    "leaderCode": "SOR_007",
    "baseCode": "ASH_019",
    "matchedPayment": true,
    "units": [
      {
        "key": "f1",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e1",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "f2",
        "code": "ASH_261",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e2",
        "code": "IBH_007",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-actual-calm-native-experience-shield-preserved-p1",
    "calmBefore": true,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "LOF_054",
        "target": "recipient"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-actual-calm-native-experience-shield-preserved-p2",
    "calmBefore": true,
    "units": [
      {
        "key": "recipient",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "e",
        "code": "HMW_268",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      }
    ],
    "plan": [
      {
        "code": "LOF_054",
        "target": "recipient"
      },
      {
        "code": "IC27_027"
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ambush-first-dead-source-live-array-p1",
    "ambush": true,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "bendu",
        "code": "LOF_170",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  },
  {
    "name": "IC27_027-ambush-first-dead-source-live-array-p2",
    "ambush": true,
    "units": [
      {
        "key": "wedge",
        "code": "SOR_100",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "bendu",
        "code": "LOF_170",
        "owner": "enemy",
        "exhausted": false,
        "shields": 0
      },
      {
        "key": "friend",
        "code": "HMW_082",
        "owner": "own",
        "exhausted": false,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseDamage": 0,
    "plan": [
      {
        "code": "IC27_027"
      }
    ],
    "actualScriptedCardIds": [
      "IC27_027"
    ],
    "family": "ordered-ground-exhaust"
  }
];
const normalize=e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planOrderedGroundExhaustProbes(entry){
if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
const tokens={battleDroid:'TOKEN_BATTLE_DROID',cloneTrooper:'TOKEN_CLONE_TROOPER',tieFighter:'TOKEN_TIE_FIGHTER',xwing:'TOKEN_XWING'};
const cases=designs.map(design=>{const obligationId=entry.setCode+':ordered-ground-exhaust:'+design.name;return{id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,design.leaderCode??'SOR_014',design.baseCode??'SOR_023',...(design.units??[]).map(u=>u.code),...design.plan.map(p=>p.code),...(design.tokens??[]).map(t=>tokens[t.name]),...((design.units??[]).some(u=>u.shields)||design.calmBefore?['TOKEN_SHIELD']:[]),...(design.calmBefore?['TOKEN_EXPERIENCE']:[])])].sort(),behaviorIds:['opponent-first-ground-exhaust','one-event-window','cancelled-already-exhausted-source','live-controller-inventory'],assertionStages:['real-paid-unit-entry','generated-event-array-order-and-cancellation','successful-event-order-one-window','ground-only-state-and-native-attachments','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'ordered-ground-exhaust',scenarioName:design.name,design}};});
const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'ordered-ground-exhaust',dimension:c.name}));
const uncovered=[['source-semantics','Finite witnesses require independent complete pinned source/helper and compiler/runtime review'],['ordering-public-state-sensitivity','Source read-only observers assert exact opponent-first full generated array and successful event order. Exhaust ordering is public-state unobservable in these admitted setups; do not credit trace sensitivity to reversing controller partitions. Root runtime vector/one-window/import-order gates remain separate requirements'],['external-exhaust-replacements','Unsupported KyloLightsaber/Mythosaur replacement mechanics and arbitrary restrictions are not approximated or credited'],['control-piloting-and-ready-entry','No capture/piloting/control-transfer or unsupported ordinary unit entersReady constructor; normal source inclusion is proved by its generated then canceled event'],['runtime-capacity-and-legacy-import','Both seats witness35generated/34successful recipients beyond group32, but MAX_CARDS/capacity and missing/malformed authoritative arenaUnitOrder must also fail closed in root runtime tests']].map(([dimension,reason])=>{const id=entry.setCode+':ordered-ground-exhaust:'+dimension;obligations.push({id,path:'ordered-ground-exhaust',dimension});return{id,path:'ordered-ground-exhaust',reason};});
return{schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};}
module.exports={planOrderedGroundExhaustProbes};
