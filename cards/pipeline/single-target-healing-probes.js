'use strict';
const same = require('node:util').isDeepStrictEqual;
const shape = {
  "setCode": "JTL_071",
  "sourcePath": "game/cards/04_JTL/units/CR90ReliefRunner.ts",
  "sourceHash": "8b4bb1ab4dc73806873da53104bc555ec6845a76bd887178585b69dda23387eb",
  "className": "CR90ReliefRunner",
  "baseClass": "NonLeaderUnitCard",
  "card": {
    "title": "CR90 Relief Runner",
    "subtitle": "",
    "cost": 6,
    "hp": 6,
    "power": 4,
    "text": "Restore 2 (When this unit attacks, heal 2 damage from your base.)\nWhen Defeated: Heal up to 3 damage from a unit or base.",
    "deployBox": null,
    "epicAction": "",
    "unique": false,
    "rules": null,
    "upgradePower": null,
    "upgradeHp": null,
    "id": "7610382003",
    "aspects": [
      "vigilance"
    ],
    "traits": [
      "rebel",
      "vehicle",
      "capital ship"
    ],
    "arena": "space",
    "keywords": [
      "restore"
    ],
    "types": [
      "unit"
    ],
    "setId": {
      "set": "JTL",
      "number": 71
    },
    "setCodes": [
      {
        "set": "JTL",
        "number": 71
      }
    ],
    "internalName": "cr90-relief-runner"
  },
  "abilities": {
    "leader": [],
    "unit": [],
    "card": [
      {
        "trigger": "whenDefeated",
        "title": "Heal up to 3 damage from a unit or base",
        "costs": [],
        "effect": {
          "kind": "distributeHealing",
          "amount": {
            "kind": "const",
            "value": 3
          },
          "maxTargets": 1,
          "canChooseNoTargets": true,
          "canDistributeLess": true,
          "targetResolver": {
            "owner": "any",
            "type": "any",
            "zone": "anyAttackable"
          }
        }
      }
    ]
  },
  "keywordValues": {
    "restore": 2
  },
  "attachCondition": null,
  "piloting": null
};
const designs = [
  {
    "name": "JTL_071-own-ground-1-p1",
    "target": "subject",
    "amount": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-ground-1-p2",
    "target": "subject",
    "amount": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-ground-2-p1",
    "target": "subject",
    "amount": 2,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-ground-2-p2",
    "target": "subject",
    "amount": 2,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-ground-3-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-ground-3-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-1-p1",
    "target": "subject",
    "amount": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-1-p2",
    "target": "subject",
    "amount": 1,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-2-p1",
    "target": "subject",
    "amount": 2,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-2-p2",
    "target": "subject",
    "amount": 2,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-3-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-ground-3-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "LOF_119",
        "owner": "own",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-space-overallocate3-damage1-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-space-overallocate3-damage1-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "own",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-space-overallocate3-damage1-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-space-overallocate3-damage1-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 1,
        "shields": 0
      },
      {
        "key": "opposite",
        "code": "HMW_082",
        "owner": "own",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-base-three-p1",
    "target": "ownBase",
    "amount": 3,
    "baseDamage": 4,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-own-base-three-p2",
    "target": "ownBase",
    "amount": 3,
    "baseDamage": 4,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-base-three-p1",
    "target": "enemyBase",
    "amount": 3,
    "baseDamage": 4,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-enemy-base-three-p2",
    "target": "enemyBase",
    "amount": 3,
    "baseDamage": 4,
    "units": [
      {
        "key": "ground",
        "code": "HMW_082",
        "owner": "own",
        "damage": 2,
        "shields": 0
      },
      {
        "key": "space",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 2,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-undamaged-positive-zeroactual-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "own",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-undamaged-positive-zeroactual-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "own",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-all-undamaged-empty-p1",
    "target": null,
    "amount": 0,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-all-undamaged-empty-p2",
    "target": null,
    "amount": 0,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 0,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-all-undamaged-positive-p1",
    "target": "ownBase",
    "amount": 3,
    "units": [],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-all-undamaged-positive-p2",
    "target": "ownBase",
    "amount": 3,
    "units": [],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-emptychoice-with-damaged-targets-p1",
    "target": null,
    "amount": 0,
    "baseDamage": 4,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-emptychoice-with-damaged-targets-p2",
    "target": null,
    "amount": 0,
    "baseDamage": 4,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 4,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-shield-preserved-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 1
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-shield-preserved-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "subject",
        "code": "HMW_082",
        "owner": "own",
        "damage": 4,
        "shields": 1
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-native-token-undamaged-positive-p1",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "other",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
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
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-native-token-undamaged-positive-p2",
    "target": "subject",
    "amount": 3,
    "units": [
      {
        "key": "other",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
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
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-deployed-leader-p1",
    "target": "leader",
    "amount": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "leaderDamage": 0,
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-deployed-leader-p2",
    "target": "leader",
    "amount": 3,
    "deployedLeader": true,
    "units": [
      {
        "key": "other",
        "code": "LOF_119",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "leaderDamage": 0,
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "baseDamage": 0,
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-restore2-attack-before-death-p1",
    "target": "ownBase",
    "amount": 3,
    "restore": true,
    "baseDamage": 4,
    "units": [
      {
        "key": "ready",
        "code": "HMW_170",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 0,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "HMW_170",
        "from": "ready",
        "target": "focus"
      },
      {
        "code": "JTL_071",
        "from": "focus",
        "target": "enemyBase"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  },
  {
    "name": "JTL_071-restore2-attack-before-death-p2",
    "target": "ownBase",
    "amount": 3,
    "restore": true,
    "baseDamage": 4,
    "units": [
      {
        "key": "ready",
        "code": "HMW_170",
        "owner": "own",
        "damage": 0,
        "shields": 0
      },
      {
        "key": "other",
        "code": "HMW_082",
        "owner": "enemy",
        "damage": 3,
        "shields": 0
      }
    ],
    "ownSeat": 1,
    "baseCode": "ASH_019",
    "actualScriptedCardIds": [
      "JTL_071"
    ],
    "plan": [
      {
        "code": "JTL_071"
      },
      {
        "code": "HMW_170",
        "from": "ready",
        "target": "focus"
      },
      {
        "code": "JTL_071",
        "from": "focus",
        "target": "enemyBase"
      },
      {
        "code": "SHD_079",
        "target": "focus"
      }
    ]
  }
];
const normalize = e=>({setCode:e.setCode,sourcePath:e.sourcePath,sourceHash:e.sourceHash,className:e.className,baseClass:e.baseClass,card:e.card,abilities:e.abilities,keywordValues:e.keywordValues,attachCondition:e.attachCondition??null,piloting:e.piloting??null});
function planSingleTargetHealingProbes(entry) {
 if(entry.status!=='supported'||entry.diagnostics?.length||!same(shape,normalize(entry)))return null;
 const cases=designs.map(design=>{
  const obligationId=entry.setCode+':single-target-healing:'+design.name;
  return {id:'auto/'+entry.setCode+'/'+design.name.toLowerCase().replaceAll('_','-'),name:design.name,kind:'probe',source:{module:'../pipeline/probe-fixture.js',scenario:design.name},focusCardIds:[entry.setCode],setupCardIds:[...new Set([entry.setCode,'SOR_014','ASH_019','SHD_079','HMW_170',...(design.units??[]).map(u=>u.code),'TOKEN_BATTLE_DROID','TOKEN_TIE_FIGHTER','TOKEN_XWING','TOKEN_SHIELD'])].sort(),behaviorIds:['when-defeated-single-target-healing','complete-allocation-domain','actual-heal-vs-assignment','printed-restore-2'],assertionStages:['paid-focus-and-defeat','offboard-source-captured-trigger','exact-distribution-prompt','actual-heal-and-restore-order','terminal-continuation'],coverageObligationIds:[obligationId],probe:{family:'single-target-healing',scenarioName:design.name,design}};
 });
 const obligations=cases.map(c=>({id:c.coverageObligationIds[0],path:'single-target-healing',dimension:c.name}));
 const uncovered=[['source-semantics','Finite source-derived witnesses require independent complete source lowering review.'],['other-death-paths','Paid direct defeat captures the source; no exhaustive simultaneous combatdeath, capture or source replacement witnesses.'],['external-heal-replacements','No exhaustive BeHealed restriction or healing replacement priority witnesses.']].map(([dimension,reason])=>{const id=entry.setCode+':single-target-healing:'+dimension;obligations.push({id,path:'single-target-healing',dimension});return{id,path:'single-target-healing',reason};});
 return {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases,obligations,uncovered};
}
module.exports={planSingleTargetHealingProbes};
