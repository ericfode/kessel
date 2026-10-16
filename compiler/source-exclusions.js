'use strict';

// Explicit user-requested stopped cohort. Apply before parsing source; these
// definitions are excluded from conversion and execution, independently of
// which generic compiler capabilities become available.
const excluded = [
  {
    "setCode": "HMW_250",
    "sourcePath": "game/cards/09_HMW/units/ImperialCavalry.ts",
    "id": "1175708178",
    "internalName": "imperial-cavalry"
  },
  {
    "setCode": "JTL_172",
    "sourcePath": "game/cards/04_JTL/upgrades/TwinLaserTurret.ts",
    "id": "9981313319",
    "internalName": "twin-laser-turret"
  },
  {
    "setCode": "SEC_246",
    "sourcePath": "game/cards/06_SEC/events/ContemptForCulture.ts",
    "id": "8056594889",
    "internalName": "contempt-for-culture"
  },
  {
    "setCode": "SHD_074",
    "sourcePath": "game/cards/02_SHD/upgrades/VambraceGrappleshot.ts",
    "id": "3525325147",
    "internalName": "vambrace-grappleshot"
  },
  {
    "setCode": "SOR_121",
    "sourcePath": "game/cards/01_SOR/upgrades/HardpointHeavyBlaster.ts",
    "id": "3987987905",
    "internalName": "hardpoint-heavy-blaster"
  }
];

function sourceExclusion(file, sourceHash) {
    const entry=excluded.find(entry=>entry.sourcePath===file);
    if(!entry)return null;
    return {id:entry.id,internalName:entry.internalName,sourcePath:file,sourceHash,
        status:'unsupported',abilities:{leader:[],unit:[],card:[]},
        diagnostics:[{code:'QUARANTINED',pattern:'quarantined-source',file,line:1,column:1,
            message:'Excluded from conversion by explicit user instruction'}]};
}

module.exports={sourceExclusion,excluded};
