'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const RAW_CASES=[
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-force-force",
    "name": "generated-token-continuation-owner-0-force-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-force:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "force-force",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-force-force",
    "name": "generated-token-continuation-owner-1-force-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-force:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "force-force",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-force-mando",
    "name": "generated-token-continuation-owner-0-force-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-mando:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "force-mando",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-force-mando",
    "name": "generated-token-continuation-owner-1-force-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-mando:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "force-mando",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-mando-force",
    "name": "generated-token-continuation-owner-0-mando-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:mando-force:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "mando-force",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-mando-force",
    "name": "generated-token-continuation-owner-1-mando-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:mando-force:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "mando-force",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-mando-mando",
    "name": "generated-token-continuation-owner-0-mando-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:mando-mando:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "mando-mando",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-mando-mando",
    "name": "generated-token-continuation-owner-1-mando-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:mando-mando:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "mando-mando",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-both-force",
    "name": "generated-token-continuation-owner-0-both-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:both-force:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "both-force",
      "resourceCount": 24,
      "force": true,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-both-force",
    "name": "generated-token-continuation-owner-1-both-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:both-force:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "both-force",
      "resourceCount": 24,
      "force": true,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-both-mando",
    "name": "generated-token-continuation-owner-0-both-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:both-mando:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "both-mando",
      "resourceCount": 24,
      "force": true,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-both-mando",
    "name": "generated-token-continuation-owner-1-both-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:both-mando:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "both-mando",
      "resourceCount": 24,
      "force": true,
      "mando": true,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-neither-force",
    "name": "generated-token-continuation-owner-0-neither-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:neither-force:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "neither-force",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-neither-force",
    "name": "generated-token-continuation-owner-1-neither-force",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:neither-force:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "neither-force",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-neither-mando",
    "name": "generated-token-continuation-owner-0-neither-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:neither-mando:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "neither-mando",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-neither-mando",
    "name": "generated-token-continuation-owner-1-neither-mando",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:neither-mando:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "neither-mando",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-force-clip",
    "name": "generated-token-continuation-owner-0-force-clip",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-clip:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "force-clip",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 2,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-force-clip",
    "name": "generated-token-continuation-owner-1-force-clip",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-clip:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "force-clip",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 2,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-snoke-newborn-death",
    "name": "generated-token-continuation-owner-0-snoke-newborn-death",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:snoke-newborn-death:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "snoke-newborn-death",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": true,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-snoke-newborn-death",
    "name": "generated-token-continuation-owner-1-snoke-newborn-death",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:snoke-newborn-death:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "snoke-newborn-death",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": true,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-repeat-regroup-combat",
    "name": "generated-token-continuation-owner-0-repeat-regroup-combat",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:repeat-regroup-combat:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "repeat-regroup-combat",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": true,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-repeat-regroup-combat",
    "name": "generated-token-continuation-owner-1-repeat-regroup-combat",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:repeat-regroup-combat:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "repeat-regroup-combat",
      "resourceCount": 24,
      "force": false,
      "mando": true,
      "snoke": false,
      "lifecycle": true,
      "enemyTraits": false,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-force-healthy",
    "name": "generated-token-continuation-owner-0-force-healthy",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-healthy:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "force-healthy",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 0,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-force-healthy",
    "name": "generated-token-continuation-owner-1-force-healthy",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:force-healthy:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "force-healthy",
      "resourceCount": 24,
      "force": true,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": false,
      "branch": "force",
      "baseDamage": 0,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-0-enemy-traits-negative",
    "name": "generated-token-continuation-owner-0-enemy-traits-negative",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:enemy-traits-negative:owner-0"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 0,
      "scenario": "enemy-traits-negative",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": true,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  },
  {
    "id": "auto/ASH_257/generated-token-continuation-owner-1-enemy-traits-negative",
    "name": "generated-token-continuation-owner-1-enemy-traits-negative",
    "kind": "probe",
    "focusCardIds": [
      "ASH_257"
    ],
    "setupCardIds": [
      "ASH_061",
      "ASH_131",
      "ASH_190",
      "ASH_201",
      "ASH_242",
      "ASH_261",
      "HMW_082",
      "HMW_093",
      "HMW_116",
      "HMW_128",
      "HMW_135",
      "HMW_155",
      "HMW_163",
      "HMW_174",
      "HMW_178",
      "HMW_219",
      "SHD_037",
      "SHD_169",
      "SOR_014",
      "SOR_023",
      "TOKEN_ADVANTAGE",
      "TOKEN_MANDALORIAN",
      "TOKEN_SHIELD"
    ],
    "behaviorIds": [
      "generated-token-continuation"
    ],
    "assertionStages": [
      "construction",
      "normal-paid-play",
      "conditional-modal",
      "physical-sequential-recipient",
      "terminal-completion"
    ],
    "coverageObligationIds": [
      "ASH_257:generated-token-continuation:enemy-traits-negative:owner-1"
    ],
    "probe": {
      "family": "generated-token-continuation",
      "cardId": "ASH_257",
      "owner": 1,
      "scenario": "enemy-traits-negative",
      "resourceCount": 24,
      "force": false,
      "mando": false,
      "snoke": false,
      "lifecycle": false,
      "enemyTraits": true,
      "branch": "mando",
      "baseDamage": 9,
      "leaderCode": "SOR_014",
      "enemyLeaderCode": "SOR_014",
      "baseCode": "SOR_023",
      "enemyBaseCode": "SOR_023",
      "deck": [
        {
          "code": "ASH_257",
          "count": 3
        },
        {
          "code": "HMW_128",
          "count": 3
        },
        {
          "code": "SHD_169",
          "count": 3
        },
        {
          "code": "SHD_037",
          "count": 3
        },
        {
          "code": "ASH_061",
          "count": 3
        },
        {
          "code": "ASH_131",
          "count": 3
        },
        {
          "code": "ASH_190",
          "count": 3
        },
        {
          "code": "ASH_201",
          "count": 3
        },
        {
          "code": "ASH_242",
          "count": 3
        },
        {
          "code": "ASH_261",
          "count": 3
        },
        {
          "code": "HMW_082",
          "count": 3
        },
        {
          "code": "HMW_093",
          "count": 3
        },
        {
          "code": "HMW_116",
          "count": 3
        },
        {
          "code": "HMW_135",
          "count": 3
        },
        {
          "code": "HMW_155",
          "count": 3
        },
        {
          "code": "HMW_163",
          "count": 3
        },
        {
          "code": "HMW_174",
          "count": 2
        }
      ]
    }
  }
];
const PROFILES={
  "ASH_257": {
    "entryHash": "910359df227aa64b1d88fa87dcfa302b9ebde0735e7048f4a92a64c77f4b9346",
    "metadata": {
      "setCode": "ASH_257",
      "title": "Choose Your Path",
      "subtitle": "",
      "cost": 2,
      "hp": null,
      "power": null,
      "text": "Choose one:\nIf you control a Force unit, heal 5 damage from your base.\nIf you control a Mandalorian unit, create a Mandalorian token and give an Advantage token to it.\n",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "2613847155",
      "aspects": [
        "heroism"
      ],
      "traits": [
        "force",
        "mandalorian"
      ],
      "keywords": [],
      "types": [
        "event"
      ],
      "setId": {
        "set": "ASH",
        "number": 257
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 257
        }
      ],
      "internalName": "choose-your-path"
    }
  },
  "ASH_061": {
    "entryHash": "f6a033f123707b8cd7f0431599b43617fa2e088092586b0fc67343ca8d482c9b",
    "metadata": {
      "setCode": "ASH_061",
      "title": "Strike Team Vanguard",
      "subtitle": "",
      "cost": 4,
      "hp": 5,
      "power": 5,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1029172801",
      "aspects": [
        "vigilance",
        "heroism"
      ],
      "traits": [
        "rebel",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 61
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 61
        }
      ],
      "internalName": "strike-team-vanguard"
    }
  },
  "ASH_131": {
    "entryHash": "1fcfd58c506b4e94f313d01deacffd6cf2fbc4c2bb180bd7bb1d46d296961c2d",
    "metadata": {
      "setCode": "ASH_131",
      "title": "Dinosaur Turtle",
      "subtitle": "",
      "cost": 6,
      "hp": 7,
      "power": 7,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9890561214",
      "aspects": [
        "command"
      ],
      "traits": [
        "creature"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 131
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 131
        }
      ],
      "internalName": "dinosaur-turtle"
    }
  },
  "ASH_190": {
    "entryHash": "5d22cbe617157548f41e5cc0d44365242c093ed7a66e8135cd52e106e41c0b93",
    "metadata": {
      "setCode": "ASH_190",
      "title": "Peridea Bandit",
      "subtitle": "",
      "cost": 2,
      "hp": 1,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4344365271",
      "aspects": [
        "cunning",
        "villainy"
      ],
      "traits": [
        "fringe"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 190
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 190
        }
      ],
      "internalName": "peridea-bandit"
    }
  },
  "ASH_201": {
    "entryHash": "173783137608a82a1eae6b38accd237d6484517f350c4fe9f399ec4dd08a7c08",
    "metadata": {
      "setCode": "ASH_201",
      "title": "Open Circle Ace",
      "subtitle": "",
      "cost": 1,
      "hp": 2,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9620848758",
      "aspects": [
        "cunning",
        "heroism"
      ],
      "traits": [
        "republic",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 201
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 201
        }
      ],
      "internalName": "open-circle-ace"
    }
  },
  "ASH_242": {
    "entryHash": "d0a10a6e6307b625451b06765d5f5b8304f68d9e3e2cc3edd510c564eee4abd5",
    "metadata": {
      "setCode": "ASH_242",
      "title": "Death Trooper Squad",
      "subtitle": "",
      "cost": 4,
      "hp": 4,
      "power": 5,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9011599796",
      "aspects": [
        "villainy"
      ],
      "traits": [
        "imperial",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 242
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 242
        }
      ],
      "internalName": "death-trooper-squad"
    }
  },
  "ASH_261": {
    "entryHash": "e62a861a7625bd2c9f6222abcf1ca31168d7a493d2bf5af30188a863a5325ab2",
    "metadata": {
      "setCode": "ASH_261",
      "title": "Noti Mobile Pod",
      "subtitle": "",
      "cost": 3,
      "hp": 4,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5556203656",
      "aspects": [],
      "traits": [
        "vehicle",
        "transport"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "ASH",
        "number": 261
      },
      "setCodes": [
        {
          "set": "ASH",
          "number": 261
        }
      ],
      "internalName": "noti-mobile-pod"
    }
  },
  "HMW_082": {
    "entryHash": "d4b667f1930c70b6bc1eac9ec62dd96121b44c3baba5ef9d0e9a32fb0a683e81",
    "metadata": {
      "setCode": "HMW_082",
      "title": "Nala Se",
      "subtitle": "Silent Survivor",
      "cost": 2,
      "hp": 7,
      "power": 1,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5691187672",
      "aspects": [
        "vigilance",
        "vigilance"
      ],
      "traits": [
        "kaminoan"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 82
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 82
        }
      ],
      "internalName": "nala-se#silent-survivor"
    }
  },
  "HMW_093": {
    "entryHash": "5c1f7ec438f2e2ad83ea97ab207ad8bdf99f80be4d2dc04e4734925f2b8477ee",
    "metadata": {
      "setCode": "HMW_093",
      "title": "Coastal Catamarans",
      "subtitle": "",
      "cost": 7,
      "hp": 8,
      "power": 8,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "1226993616",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "vehicle",
        "transport"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 93
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 93
        }
      ],
      "internalName": "coastal-catamarans"
    }
  },
  "HMW_116": {
    "entryHash": "b16e84dce80f70da340f5c2c87580793ce7367f8b516f24ff63680a2ec281e69",
    "metadata": {
      "setCode": "HMW_116",
      "title": "Ewok Brigade",
      "subtitle": "",
      "cost": 2,
      "hp": 4,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8382448205",
      "aspects": [
        "command",
        "heroism"
      ],
      "traits": [
        "ewok"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 116
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 116
        }
      ],
      "internalName": "ewok-brigade"
    }
  },
  "HMW_128": {
    "entryHash": "ce5b5159056a22ff31c7ec06d689caede18bc64edacef70b39e72efff0d73027",
    "metadata": {
      "setCode": "HMW_128",
      "title": "Mace Windu",
      "subtitle": "Austere Centrist",
      "cost": 3,
      "hp": 5,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0886129762",
      "aspects": [
        "command",
        "command"
      ],
      "traits": [
        "force",
        "jedi"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 128
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 128
        }
      ],
      "internalName": "mace-windu#austere-centrist"
    }
  },
  "HMW_135": {
    "entryHash": "e74522cc77ce7cbf0d454a4948ff5b12684f882268757a157a63761a4bbbdc83",
    "metadata": {
      "setCode": "HMW_135",
      "title": "Arena Reek",
      "subtitle": "Fixture of Ceremony",
      "cost": 3,
      "hp": 4,
      "power": 4,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0212754007",
      "aspects": [
        "command"
      ],
      "traits": [
        "creature"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 135
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 135
        }
      ],
      "internalName": "arena-reek#fixture-of-ceremony"
    }
  },
  "HMW_155": {
    "entryHash": "9db7facb8ee661bc3c4181c8c07d9f1629e2b1ea0f7371c63e68b66522002c19",
    "metadata": {
      "setCode": "HMW_155",
      "title": "Filthy Dianoga",
      "subtitle": "",
      "cost": 4,
      "hp": 5,
      "power": 5,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "7827651708",
      "aspects": [
        "aggression",
        "villainy"
      ],
      "traits": [
        "creature"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 155
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 155
        }
      ],
      "internalName": "filthy-dianoga"
    }
  },
  "HMW_163": {
    "entryHash": "9021b8be120211bfeb9d2b7a357fe033bd16642d59602c7a18093acad5a0b48a",
    "metadata": {
      "setCode": "HMW_163",
      "title": "Champions of Endor",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4516882765",
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "ewok"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 163
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 163
        }
      ],
      "internalName": "champions-of-endor"
    }
  },
  "HMW_174": {
    "entryHash": "68d86d06abc3e805c3ac3c424aa6ddeb2258d405ffb99b651323680584894d19",
    "metadata": {
      "setCode": "HMW_174",
      "title": "Maul",
      "subtitle": "Only Revenge Remains",
      "cost": 4,
      "hp": 6,
      "power": 6,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3460896271",
      "aspects": [
        "aggression",
        "aggression"
      ],
      "traits": [
        "force",
        "underworld"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 174
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 174
        }
      ],
      "internalName": "maul#only-revenge-remains"
    }
  },
  "HMW_178": {
    "entryHash": "c59421061ea586b6387d6e587b7af8001adbbb3dd1eff18a13c2e958e64dc87c",
    "metadata": {
      "setCode": "HMW_178",
      "title": "Desperate Nantex",
      "subtitle": "",
      "cost": 2,
      "hp": 3,
      "power": 2,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "5508530442",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "separatist",
        "vehicle",
        "fighter"
      ],
      "arena": "space",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 178
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 178
        }
      ],
      "internalName": "desperate-nantex"
    }
  },
  "HMW_219": {
    "entryHash": "8eb4414f5a1f69fe1013ee0b4bcb644aacec9e84165ccf0a863b105ff35abc96",
    "metadata": {
      "setCode": "HMW_219",
      "title": "Jaxxon",
      "subtitle": "I Can Hear You Breathing!",
      "cost": 1,
      "hp": 3,
      "power": 3,
      "text": "",
      "deployBox": null,
      "epicAction": "",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "9427322990",
      "aspects": [
        "cunning",
        "cunning"
      ],
      "traits": [
        "underworld"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "HMW",
        "number": 219
      },
      "setCodes": [
        {
          "set": "HMW",
          "number": 219
        }
      ],
      "internalName": "jaxxon#i-can-hear-you-breathing"
    }
  },
  "SOR_014": {
    "entryHash": "d1ca8396901fac3eb4eafafb55439deeadf8b385f6e195faa781a786568c2bac",
    "metadata": {
      "setCode": "SOR_014",
      "title": "Sabine Wren",
      "subtitle": "Galvanized Revolutionary",
      "cost": 4,
      "hp": 5,
      "power": 2,
      "text": "Action [exhaust]: Deal 1 damage to each base.",
      "deployBox": "On Attack: Deal 1 damage to each enemy base.",
      "epicAction": "Epic Action: If you control 4 or more resources, deploy this leader.",
      "unique": true,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "4841169874",
      "aspects": [
        "aggression",
        "heroism"
      ],
      "traits": [
        "mandalorian",
        "rebel",
        "spectre"
      ],
      "arena": "ground",
      "keywords": [],
      "types": [
        "leader"
      ],
      "setId": {
        "set": "SOR",
        "number": 14
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 14
        }
      ],
      "internalName": "sabine-wren#galvanized-revolutionary"
    }
  },
  "SOR_023": {
    "entryHash": "a694a128d3fb4bb0a4e465dcf906cb2afd3a936c7d99ee1f8cf53db1a7cb41b6",
    "metadata": {
      "setCode": "SOR_023",
      "title": "Command Center",
      "subtitle": null,
      "cost": null,
      "hp": 30,
      "power": null,
      "text": null,
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "0507674993",
      "aspects": [
        "command"
      ],
      "traits": [
        "death star"
      ],
      "keywords": [],
      "types": [
        "base"
      ],
      "setId": {
        "set": "SOR",
        "number": 23
      },
      "setCodes": [
        {
          "set": "SOR",
          "number": 23
        }
      ],
      "internalName": "command-center"
    }
  },
  "SHD_169": {
    "entryHash": "4aa250c8d001af0fb2880a516ea4cdf362575c79bf8e511f8bc8fdc5d8e95888",
    "metadata": {
      "setCode": "SHD_169",
      "title": "Clan Challengers",
      "subtitle": "",
      "cost": 5,
      "hp": 6,
      "power": 3,
      "text": "Raid 3 (This unit gets +3/+0 while attacking.)\nWhile this unit is upgraded, it gains Overwhelm. (When attacking an enemy unit, deal excess damage to the opponent's base.)",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": "Shields prevent all damage from a unit with Overwhelm. There is no excess damage since the defender is not defeated.\nIf the defender is defeated during an attack before combat damage is dealt, all combat damage is considered excess damage for Overwhelm.\n",
      "upgradePower": null,
      "upgradeHp": null,
      "id": "3487311898",
      "aspects": [
        "aggression"
      ],
      "traits": [
        "mandalorian",
        "trooper"
      ],
      "arena": "ground",
      "keywords": [
        "raid",
        "overwhelm"
      ],
      "types": [
        "unit"
      ],
      "setId": {
        "set": "SHD",
        "number": 169
      },
      "setCodes": [
        {
          "set": "SHD",
          "number": 169
        }
      ],
      "internalName": "clan-challengers"
    }
  },
  "SHD_037": {
    "entryHash": "972f1fb4d3c3643b2194496bb04dd02d774297d50bd6ea1a63c2b56203ed5299",
    "metadata": {
      "setCode": "SHD_037",
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
    }
  },
  "TOKEN_MANDALORIAN": {
    "entryHash": "228b031938f4233d811ade2b8d27dbcedb892be0bc2e016f26dcaa8d7ed2be10",
    "metadata": {
      "setCode": "TOKEN_MANDALORIAN",
      "title": "Mandalorian",
      "subtitle": "",
      "cost": 0,
      "hp": 2,
      "power": 2,
      "text": "Shielded (When you create this token, give a Shield token to it.)",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": null,
      "upgradePower": null,
      "upgradeHp": null,
      "id": "8192010342",
      "aspects": [
        "vigilance"
      ],
      "traits": [
        "mandalorian"
      ],
      "arena": "ground",
      "keywords": [
        "shielded"
      ],
      "types": [
        "token",
        "unit"
      ],
      "setId": {
        "set": "ASH"
      },
      "internalName": "mandalorian"
    }
  },
  "TOKEN_ADVANTAGE": {
    "entryHash": "be7663af45ebe560c97cbe5b1305863b4dcdf6bd4d594ef333643b3fd1fb7234",
    "metadata": {
      "setCode": "TOKEN_ADVANTAGE",
      "title": "Advantage",
      "subtitle": "",
      "cost": 0,
      "hp": 0,
      "power": 1,
      "text": "When attached unit's attack or defense ends: Defeat this upgrade.",
      "deployBox": null,
      "epicAction": "",
      "unique": false,
      "rules": "When Attack Ends/When Defense Ends abilities trigger at the same time as units deal combat damage.\n",
      "upgradePower": 1,
      "upgradeHp": 0,
      "id": "5844562972",
      "aspects": [],
      "traits": [
        "innate"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "ASH"
      },
      "internalName": "advantage"
    }
  },
  "TOKEN_SHIELD": {
    "entryHash": "a8a34f754731a5dc9de991d5476b9491466190ac7521e0c7192c9210fc5538a9",
    "metadata": {
      "setCode": "TOKEN_SHIELD",
      "title": "Shield",
      "subtitle": null,
      "cost": 0,
      "hp": 0,
      "power": 0,
      "text": "If damage would be dealt to attached unit, prevent that damage. If you do, defeat a Shield token on it.",
      "deployBox": null,
      "epicAction": null,
      "unique": false,
      "rules": null,
      "upgradePower": 0,
      "upgradeHp": 0,
      "id": "8752877738",
      "aspects": [],
      "traits": [
        "armor"
      ],
      "keywords": [],
      "types": [
        "token",
        "upgrade"
      ],
      "setId": {
        "set": "SOR"
      },
      "internalName": "shield"
    }
  }
};
const HELPER_INPUTS=[
  {
    "file": "server/AnonymousSpectator.ts",
    "sha256": "5a4ac2ea5d70301fab51f05ec8b0ff1074a83827c157be48ed4c19e8f4285f2f"
  },
  {
    "file": "server/env.ts",
    "sha256": "e6e06ac06b0a1f88619960f26711c7cf6d11a5b6a9ad3fe443380e77c6e29e18"
  },
  {
    "file": "server/game/abilities/EpicActionAbility.ts",
    "sha256": "a940e9db8d8112dd49e970d03279becff44307444c48c93c32a4030f800f6e0e"
  },
  {
    "file": "server/game/abilities/keyword/AmbushAbility.ts",
    "sha256": "afe5b72e495c3f4822e76a65f49e5ce239c40358b3f5ee5bfb915a3762ecc11f"
  },
  {
    "file": "server/game/abilities/keyword/BountyAbility.ts",
    "sha256": "182378c1ce7841d48018c00e81493b40dd05c343ff1aa1eb1f48539b5e9d219e"
  },
  {
    "file": "server/game/abilities/keyword/exploit/ExploitCostAdjuster.ts",
    "sha256": "fd16e7577ffd78fc237d70d2fce5ed2bb7fb9904468e9406df56b8a99da0b4cd"
  },
  {
    "file": "server/game/abilities/keyword/exploit/MergedExploitCostAdjuster.ts",
    "sha256": "2dc083172e27beb757a69fb683ede698625509c51054cf84262a6a0bef09c446"
  },
  {
    "file": "server/game/abilities/keyword/PlotAbility.ts",
    "sha256": "484e4461484bf965639f57a88137a83555b31a14043b58e01822e5bd35464fb5"
  },
  {
    "file": "server/game/abilities/keyword/RestoreAbility.ts",
    "sha256": "12a0b53abd82f49830025dbd5833f0ff4e21a7023a223f1a71325327663e49d4"
  },
  {
    "file": "server/game/abilities/keyword/SaboteurDefeatShieldsAbility.ts",
    "sha256": "cd131ade6595b0572075e942546641f64c64cee8833656ed274c7864a8a09cf7"
  },
  {
    "file": "server/game/abilities/keyword/ShieldedAbility.ts",
    "sha256": "61297990f8690c10d803c2afe8937513086fa45f34dfbc2f2b2a801e22667567"
  },
  {
    "file": "server/game/abilities/keyword/SupportAbility.ts",
    "sha256": "451dc0b24da1722ca22ba3090530e3154ecb52170f2c409066f5c74e6d270d11"
  },
  {
    "file": "server/game/AbilityHelper.ts",
    "sha256": "6ef4ef44e1f9a1fbf9e834fac10b2a34a260eb1cd31b80c6fa7400d1e8aa9f0f"
  },
  {
    "file": "server/game/actions/InitiateAttackAction.ts",
    "sha256": "1fe125e5074fc0a5b01bea2fe754f7c3acbdff6efb32aa7e326cb2381b402585"
  },
  {
    "file": "server/game/actions/PlayEventAction.ts",
    "sha256": "72ea580183ad952dc2eccb7021888f9f782a599696a022a22e3cbfcbfef1be10"
  },
  {
    "file": "server/game/actions/PlayUnitAction.ts",
    "sha256": "ea74cd48fb06718239b845f81636a963f36947f7eaaf5d64e7df164062f52b1a"
  },
  {
    "file": "server/game/actions/PlayUpgradeAction.ts",
    "sha256": "262d6b3d040b78f79df4283d9e2ed74d41244bb7a7afe92bf586f7ef3d6c65e9"
  },
  {
    "file": "server/game/core/ability/AbilityContext.ts",
    "sha256": "fdf25428b713a5f5b622a7b78d5ea6a970659f3a9f07471dde5a78cc10d1f8df"
  },
  {
    "file": "server/game/core/ability/AbilityLimit.ts",
    "sha256": "ae922d1364425aa274b8b1f6f64fd8b8f8a721a069b2e1b7efd6a1c9f9f2720b"
  },
  {
    "file": "server/game/core/ability/abilityTargets/CardTargetResolver.ts",
    "sha256": "4562c7a69b37610ca99dce69b001a89efcd6a09d85fd9c3dadeeba8b56694a85"
  },
  {
    "file": "server/game/core/ability/abilityTargets/DropdownListTargetResolver.ts",
    "sha256": "c2a0ae12e5bdf2953ae23a8f7a3779cbbdabacac2a606863859d43650eedf967"
  },
  {
    "file": "server/game/core/ability/abilityTargets/NumberTargetResolver.ts",
    "sha256": "ddee1ad36a9769ef7d34a746d50e2ce0f9761cae13a52b95d8df76e935773aed"
  },
  {
    "file": "server/game/core/ability/abilityTargets/PlayerTargetResolver.ts",
    "sha256": "03ea15460efb900de87b737441570b0bac46bf0551674829b7d7b768f65de2fb"
  },
  {
    "file": "server/game/core/ability/abilityTargets/SelectChoice.ts",
    "sha256": "efece4c20e7f27ff9830858201ea526973b17849df57c31330608fd43b7198de"
  },
  {
    "file": "server/game/core/ability/abilityTargets/SelectTargetResolver.ts",
    "sha256": "604bdad218ff832df7bb1fb61785b6186b53d9d510b62bcc2f1e0958385c6baf"
  },
  {
    "file": "server/game/core/ability/abilityTargets/TargetResolver.ts",
    "sha256": "f573dacad7b6b64120ab80a99c865094572265c858e345505487e7567fcac211"
  },
  {
    "file": "server/game/core/ability/AbilityTypes.ts",
    "sha256": "62c4e3c514b5cae7829b4d692f8ba559f84e35af0de51868f2737f9e1f777bd7"
  },
  {
    "file": "server/game/core/ability/ActionAbility.ts",
    "sha256": "7ec8a16ee3c5a7fae9f2f6a2e3960b0fb2f4ccb30150066ee13867768b404676"
  },
  {
    "file": "server/game/core/ability/CardAbility.ts",
    "sha256": "15f2b262769f92522840bca83fab2095001c602112cfe760209351de1f6d334c"
  },
  {
    "file": "server/game/core/ability/CardAbilityStep.ts",
    "sha256": "523dd116f740ef9a67b2667731644a6279727daf7ba37c493ca41786083d4b3e"
  },
  {
    "file": "server/game/core/ability/ConstantAbility.ts",
    "sha256": "5ae2dd2a2e9f396b9edd80e2b6218e40eb85ca60231940aa3b07fec84135ddca"
  },
  {
    "file": "server/game/core/ability/DamageModificationAbility.ts",
    "sha256": "f36d435694c420a2ada9a7e13b52221dc854fe354cb42cb77b808150adfef4be"
  },
  {
    "file": "server/game/core/ability/EventAbility.ts",
    "sha256": "cdcbf49379801e5e855a50836fc1cbd076119a66ea560057f7c341752f49ae1a"
  },
  {
    "file": "server/game/core/ability/KeywordHelpers.ts",
    "sha256": "32d654fb2a6ad6fe0d8bc825b68d6030218c482e0038f101fe3d61874fd60c9f"
  },
  {
    "file": "server/game/core/ability/KeywordInstance.ts",
    "sha256": "40b77310ad29283aed0b72f79f7e0daf4a387572610f3dfd26f7b231c6d2f6e0"
  },
  {
    "file": "server/game/core/ability/PlayCardAction.ts",
    "sha256": "2334a3da66d419c92472c756d2a9b14709ca1af4402f50e7273ac80092b21039"
  },
  {
    "file": "server/game/core/ability/PlayerAction.ts",
    "sha256": "4bae0fa92df52ecd3669815c6a758d54a8945af43b77c8478dbafea59e327e78"
  },
  {
    "file": "server/game/core/ability/PlayerOrCardAbility.ts",
    "sha256": "3ac8b6a26dd978cf818934574ffc3519d879cf01c415d66f793da6434cca71f0"
  },
  {
    "file": "server/game/core/ability/PreEnterPlayAbility.ts",
    "sha256": "ad1ccc8332bd55eb11892a3875bf90be65650f29352ed29951656a4c20d81970"
  },
  {
    "file": "server/game/core/ability/ReplacementAbilityBase.ts",
    "sha256": "7afe3a74654bc8c20756b21cdef3b30e38aa4ac3660f7ddf6d71a992c46d89c7"
  },
  {
    "file": "server/game/core/ability/ReplacementEffectAbility.ts",
    "sha256": "f556d371a2fce9102770831f2e32578bec596bb8bba775aaad25391e04878328"
  },
  {
    "file": "server/game/core/ability/ReplacementEffectContext.ts",
    "sha256": "742a9a1c72ab008605e8cd9f3020d7e4e99eb756af14fd2a322795798cc52fc0"
  },
  {
    "file": "server/game/core/ability/TriggeredAbility.ts",
    "sha256": "f9c08ce8158b482356554128bdc2f7112ff289c599bdb30235a412483304bdfd"
  },
  {
    "file": "server/game/core/ability/TriggeredAbilityContext.ts",
    "sha256": "b1b8945be4bdad1e6ea5d4f0bd905a4a8536ee319e3a447feb23632d60b1c7da"
  },
  {
    "file": "server/game/core/actionTimer/ByoyomiTimer.ts",
    "sha256": "37f3e3822087765f131ff0a0773e5c63a9ef02a18df449e28917b63e569b98ba"
  },
  {
    "file": "server/game/core/actionTimer/GameActionTimer.ts",
    "sha256": "2983ea2f1390ff7bf9f1f3b7e8b907e1f29a5b5facdc37abc8b6077cf4936fcf"
  },
  {
    "file": "server/game/core/actionTimer/IActionTimer.ts",
    "sha256": "cb7d2edd34df82639d581972acb8fa56be623635ae716ef4062b300948238db0"
  },
  {
    "file": "server/game/core/actionTimer/IByoyomiTimer.ts",
    "sha256": "fe2866abad4c6300de69d87889a189694467ca490bf100d6b8c34d061fc8a96d"
  },
  {
    "file": "server/game/core/actionTimer/NoopActionTimer.ts",
    "sha256": "b8b9324066b1f8d842830fdd1159b19a55e89f2b8a02908a816d4092a42b8e9e"
  },
  {
    "file": "server/game/core/actionTimer/SimpleActionTimer.ts",
    "sha256": "7efe467a0577208da3f182db0ff9a8e208e53edea78c7d3601757fd25ea045c8"
  },
  {
    "file": "server/game/core/attack/Attack.ts",
    "sha256": "53fe24e9538ace421a48d4d9a42e0be2d97ace387a5da7e88becdf17fece3cc6"
  },
  {
    "file": "server/game/core/attack/AttackFlow.ts",
    "sha256": "274a19f3de35ae9a04b71a97e249ad303ac2b3f4c3a092d6d1c86da2fa7f4e7f"
  },
  {
    "file": "server/game/core/attack/AttackHelpers.ts",
    "sha256": "a31ad807c0a7809442ec432db02165e5cc7a499e494965b241c232af6d12d2ce"
  },
  {
    "file": "server/game/core/card/AbilityRegistrationInterfaces.ts",
    "sha256": "58642e6fa319340d84c89738e1f713f94db6277a0bd78f3fef399d652b7c76d0"
  },
  {
    "file": "server/game/core/card/BaseCard.ts",
    "sha256": "a3fa02e9c172546acb0e1a1f9edc2693e93d1895cd419fcbe2af46e7313b4de3"
  },
  {
    "file": "server/game/core/card/baseClasses/InPlayCard.ts",
    "sha256": "78f7325e62b289868f8a64b7645ae0ac30408e622921b4a0c8ec0935130e16e1"
  },
  {
    "file": "server/game/core/card/baseClasses/PlayableOrDeployableCard.ts",
    "sha256": "a0bc737e78e20428bebb90df26c2b82bdef07b1d5177259cdd7e8d3901c9ca8d"
  },
  {
    "file": "server/game/core/card/Card.ts",
    "sha256": "16a127880aaf931c0cb040a6d9f6c9521c982a6fe77442663908aebe4012533c"
  },
  {
    "file": "server/game/core/card/CardHelpers.ts",
    "sha256": "d391d292761fffdc9e2747f4ac165ae2a6bcd55da791e27c696bac208d86f225"
  },
  {
    "file": "server/game/core/card/CardInterfaces.ts",
    "sha256": "33be7e2c58f1ae9420ce333ee124cfbf0f60b0e055ca47fed96841f5290961cf"
  },
  {
    "file": "server/game/core/card/DoubleSidedLeaderCard.ts",
    "sha256": "f63dd9302b7a6eeb8614cfbda89f625c6e972675daacd1117cf99cb2cd4c18b0"
  },
  {
    "file": "server/game/core/card/EventCard.ts",
    "sha256": "020fc560f6cd97a679ae49ad9c6be2ce433596df6458a3e3ec29461cf590a3bf"
  },
  {
    "file": "server/game/core/card/LeaderUnitCard.ts",
    "sha256": "9fa85dff3be802e9a22b20598d7e3bc04726c0dfca61dee0d560cf8bebb4d36d"
  },
  {
    "file": "server/game/core/card/NonLeaderUnitCard.ts",
    "sha256": "3d423f3bee911cc0ba0669dadab5cb4850313e076c02f1bf14bd908d1ebc7394"
  },
  {
    "file": "server/game/core/card/propertyMixins/ActionAbilityRegistration.ts",
    "sha256": "729ed633ec7e6eaede9e246d807581cf5b6de57e358cfee26792aa631b7be105"
  },
  {
    "file": "server/game/core/card/propertyMixins/AllAbilityTypeRegistrations.ts",
    "sha256": "c4d4509eff8125b5d4f6d4fe3043e3ddac224574a14d7db8df0f65e5200013a3"
  },
  {
    "file": "server/game/core/card/propertyMixins/ConstantAbilityRegistration.ts",
    "sha256": "411b7a85facebc888e5d5ae6bf380e62a8d61ce13987dc4064387400f1ea0042"
  },
  {
    "file": "server/game/core/card/propertyMixins/Cost.ts",
    "sha256": "6e0c7b9238b5ee255fcd563f87f60a17398c92cddd0bede7778e7bf864acf6a5"
  },
  {
    "file": "server/game/core/card/propertyMixins/Damage.ts",
    "sha256": "9682bd683401a57afa05f7f6ba95a5a4457eaff08ca54ee46786cfc64f4c2931"
  },
  {
    "file": "server/game/core/card/propertyMixins/LeaderProperties.ts",
    "sha256": "ddcb8ee342042a7084bc5668f981c619539e7b755c0170aac85702d7efed98a8"
  },
  {
    "file": "server/game/core/card/propertyMixins/PreEnterPlayAbilityRegistration.ts",
    "sha256": "cb4b4e2c43f311a52bcd77c7341cada14adf51703a42886530b29c35c2b62ad4"
  },
  {
    "file": "server/game/core/card/propertyMixins/PrintedHp.ts",
    "sha256": "ed75baa3bcea7b7c201f165ad973e743116fb8908e62e0b04cb3f035d6bd077c"
  },
  {
    "file": "server/game/core/card/propertyMixins/PrintedPower.ts",
    "sha256": "c43f201e919607a6abb46874ed0a2fd543f24c7cec9d7ed76cfc025efeb9a5cc"
  },
  {
    "file": "server/game/core/card/propertyMixins/StandardAbilitySetup.ts",
    "sha256": "cba8505dfdc3ade4084d6dd8f22232d704023dd1fb2ef5d64c751c92a687e0d0"
  },
  {
    "file": "server/game/core/card/propertyMixins/Token.ts",
    "sha256": "3f503f6ec00df17b9b048c288b527495ae26be2710ccf67bbc95ba471d2776a7"
  },
  {
    "file": "server/game/core/card/propertyMixins/TriggeredAbilityRegistration.ts",
    "sha256": "0feb23d92b2d67c1c2f31368c5fe1c99f4a23d4a7d506db419b70a0da2d783f3"
  },
  {
    "file": "server/game/core/card/propertyMixins/UnitProperties.ts",
    "sha256": "b165069fc0d17bc3dbc216120dacf6780574b1d823a94c4e559a71ec830ab2e2"
  },
  {
    "file": "server/game/core/card/TokenCards.ts",
    "sha256": "b82d4654f1c01e3d780c0f93df38cef20a457a7164c282819389de576d600297"
  },
  {
    "file": "server/game/core/card/UpgradeCard.ts",
    "sha256": "79e74d72c0187be164ea8a8d6bdf8f0904a89cd4378170ffdaf673e55a99e738"
  },
  {
    "file": "server/game/core/cardSelector/BaseCardSelector.ts",
    "sha256": "1321721dd06565c2ab5ff73fd752280dbdab58f37798de5380488f0ff4f2a81a"
  },
  {
    "file": "server/game/core/cardSelector/BetweenVariableXYCardSelector.ts",
    "sha256": "00d1e747447b3fe0b48be0eb748db8784c1e8e7f0bd94a7bfbe100eea912bfd5"
  },
  {
    "file": "server/game/core/cardSelector/CardSelectorFactory.ts",
    "sha256": "9239d76875ce0f5a4c669dec3b5bafc0557583ccee8460d8a810be0cc1809ae6"
  },
  {
    "file": "server/game/core/cardSelector/ExactlyVariableXCardSelector.ts",
    "sha256": "9eeb01e4d626ac04426d6014570b481d90b0903d211e36f35f52055d18f0ac74"
  },
  {
    "file": "server/game/core/cardSelector/ExactlyXCardSelector.ts",
    "sha256": "7aa0413d3ba982f8b82f34325ec85bc380cf97915d117e3e1042d07484a66c4b"
  },
  {
    "file": "server/game/core/cardSelector/MaxStatCardSelector.ts",
    "sha256": "e29ea27d6cbba26425ffb120471ba5767ae8e6141d1e0d32969fdd9bf448d58c"
  },
  {
    "file": "server/game/core/cardSelector/SingleCardSelector.ts",
    "sha256": "53a7eedc646809100a30c7bea8e59d6e4d9c8fad6fb99db07e337257f908b4b2"
  },
  {
    "file": "server/game/core/cardSelector/UnlimitedCardSelector.ts",
    "sha256": "376766cfd409627bbc319f0cabb6a47294461d6ac7c0654b12b63124e858cf40"
  },
  {
    "file": "server/game/core/cardSelector/UpToVariableXCardSelector.ts",
    "sha256": "5aaba593421b0207319a4eab137e8b1416b9aab1b7f0e9959ec0f2e46ab6f7f0"
  },
  {
    "file": "server/game/core/cardSelector/UpToXCardSelector.ts",
    "sha256": "7d8151f9c57c8b7dc79c2431c7c3335135006a4d7d8e1f61dfcd5cfd3063c074"
  },
  {
    "file": "server/game/core/chat/ChatHelpers.ts",
    "sha256": "32208e84ec22d10878227a5c5e82bbf20b71cdbb858d800004bc05217b861e72"
  },
  {
    "file": "server/game/core/chat/GameChat.ts",
    "sha256": "d7a41784e8d2e995fe090ba30907307ca71689b69e5bdab099205969bcebdd9d"
  },
  {
    "file": "server/game/core/Constants.ts",
    "sha256": "f8c2e76ea4307aed95a617bc1a354ec91fe745fb64a93c345e6e2b77bd916648"
  },
  {
    "file": "server/game/core/cost/CostAdjuster.ts",
    "sha256": "e3c51fb77b69a3fda39cd3f66b5b5880001d93d2adaff6e0de9ad66847b1d424"
  },
  {
    "file": "server/game/core/cost/CostAdjusterFactory.ts",
    "sha256": "3aa00dc02b0a5c13baf4e38948ac30a65f9cf83c41ae4f943c109249ad8b926b"
  },
  {
    "file": "server/game/core/cost/CostAdjusterWithGameSteps.ts",
    "sha256": "ae17f62e9af9aa07cd1eb2bc69ea8851886e37374b3759f7764f894802791fb1"
  },
  {
    "file": "server/game/core/cost/CostHelpers.ts",
    "sha256": "63965da7b3190bd0bf94f6b7d67a0faefe872878511ba4b1a0028d138c34ecdf"
  },
  {
    "file": "server/game/core/cost/CostInterfaces.ts",
    "sha256": "6cc158e532fe39ca8f4ad848bbdaea2940b5ddf0236c7e762af202b5ce8ee869"
  },
  {
    "file": "server/game/core/cost/CostPaymentRecovery.ts",
    "sha256": "78c2474fd33517a618816feb311a7639a129118a11d6c4207a8c375681a7959d"
  },
  {
    "file": "server/game/core/cost/DamageUnitsCostAdjuster.ts",
    "sha256": "8eefeea15a6835f679d52fcf72cd72bd12ab27a3fdf41849946f311b1332aeeb"
  },
  {
    "file": "server/game/core/cost/DefeatCreditTokensCostAdjuster.ts",
    "sha256": "dccf8fb181472b8343741fdf773be3f3f11d5894de5d4bd30be02fc6af136497"
  },
  {
    "file": "server/game/core/cost/DefeatResourcesCostAdjuster.ts",
    "sha256": "be8a522f080424cc5b81272192ce1341cbe8c2d7be00620a921047de9da250c1"
  },
  {
    "file": "server/game/core/cost/evaluation/AdjustedCostEvaluator.ts",
    "sha256": "86f2e51d8d4ef97bd6e29f475c3f64581ee6c6af0c90ce5e9545fc42a7f1c770"
  },
  {
    "file": "server/game/core/cost/evaluation/DynamicOpportunityCost.ts",
    "sha256": "a83481565042eb51a16b00cd35c1b0d7dc4014763de226dc5fb0be641d23737a"
  },
  {
    "file": "server/game/core/cost/evaluation/SimpleAdjustedCost.ts",
    "sha256": "16c45f2c638236096b137808b6e254cf972d8fcd8da711d8e94593c751640e00"
  },
  {
    "file": "server/game/core/cost/ExhaustUnitsCostAdjuster.ts",
    "sha256": "f3525c1e89b0a1160f7fcc516e2ba77d83a8a866bd86358da8607a8cec0194c5"
  },
  {
    "file": "server/game/core/cost/FreeCostAdjuster.ts",
    "sha256": "3245b944ffffd988f0505ef75820b5815e84c6a5eb5e4370943ee1c2f6782e69"
  },
  {
    "file": "server/game/core/cost/GameSystemCost.ts",
    "sha256": "5a338dcdc2d2e8cd9560bf94cd842108a0b6164219ed9177cc261b9c101f2a23"
  },
  {
    "file": "server/game/core/cost/ICost.ts",
    "sha256": "08983892c616aaa9c12c3a948c97f6225f6efe64c01082fb7722ccc1ba251c25"
  },
  {
    "file": "server/game/core/cost/IgnoreAspectCostAdjuster.ts",
    "sha256": "aee5dd69d5fb83980dae6471262d7f273d4e7ae5fd9ae24f49ee02b31d19839f"
  },
  {
    "file": "server/game/core/cost/IgnoreWildcardAspectsCostAdjuster.ts",
    "sha256": "6773d9b70a9ceb906b1ed53f6f8477dcc807b7802a2a29e53713ed41443f1678"
  },
  {
    "file": "server/game/core/cost/IncreaseCostAdjuster.ts",
    "sha256": "c4d670f899a3c675341ed8632a4b850216903f0ea7ad06facdc5eef2a6731161"
  },
  {
    "file": "server/game/core/cost/MetaActionCost.ts",
    "sha256": "f6d7af50c3a9f047177e522a8e876988ea425eaf196b49f3f2023f2fb190bff7"
  },
  {
    "file": "server/game/core/cost/ModifyPayStageCostAdjuster.ts",
    "sha256": "4ebaf2e7daeb444db8c439398d94b9070ea6e650655e322f69dc0290c5363600"
  },
  {
    "file": "server/game/core/cost/SimpleCostAdjuster.ts",
    "sha256": "e5798984195cca365d0d565c62c2db634937e906e1ddb7f22de97ed61a9c0c2e"
  },
  {
    "file": "server/game/core/cost/TargetedCostAdjuster.ts",
    "sha256": "4a71b1b7ab7851e9c347899742a4a6e417596aecfc25ac3083a2e57250d2b186"
  },
  {
    "file": "server/game/core/DiscordDispatcher.ts",
    "sha256": "9d4edd264bd90f3ab3e718cc0e5942dbc97033afae28addb5075d23394012ebb"
  },
  {
    "file": "server/game/core/event/EventRegistrar.ts",
    "sha256": "d3f9f931017d6c07601db3f90cb1a796b4171497ff3286a043feca5c58b1843f"
  },
  {
    "file": "server/game/core/event/EventWindow.ts",
    "sha256": "ee180d966d995d9ffc52df8f22a8c669f2b4e113ef69815a107127aec5fae674"
  },
  {
    "file": "server/game/core/event/GameEvent.ts",
    "sha256": "f557162ce465ed42f1e1d559d9070b804b2c8cefb95e0bab85fb1943950376ca"
  },
  {
    "file": "server/game/core/event/LastKnownInformation.ts",
    "sha256": "5d9f692e70f7dbdbe981021d90b6b4c7583c8c9fb08a85ca41211bc0ced4b60b"
  },
  {
    "file": "server/game/core/Game.ts",
    "sha256": "bfb2fa16f3381b1c3f32ea8240b32392e40217a533cac3e603f7645bf214071c"
  },
  {
    "file": "server/game/core/GameInterfaces.ts",
    "sha256": "039c5a7922c4b936f3c0a8371eb542bb5350ce8222de19c5031b4335d515df6f"
  },
  {
    "file": "server/game/core/GameObject.ts",
    "sha256": "229e22adef243a3a055b87c6468143a1755e87cc1b0f552ff49cbe953e80895e"
  },
  {
    "file": "server/game/core/GameObjectBase.ts",
    "sha256": "6a2df2ce8ab5bc25205965ceb70fecf49aee8503e67347895d3b553b95d04012"
  },
  {
    "file": "server/game/core/GameObjectUtils.ts",
    "sha256": "092ab95caaf4322f3eec02c239d04efdffaa33dc82c406e92bed04d10fb37c3f"
  },
  {
    "file": "server/game/core/GamePipeline.ts",
    "sha256": "77a5c526113b09de65cf1f482a5a76a781cb2c5be3d4cbe7efd362a0bca810d6"
  },
  {
    "file": "server/game/core/gameSteps/AbilityResolver.ts",
    "sha256": "2fb772983a114b7694967d3f101b3c7c94285c7a3183e900c73c19688b8cdb30"
  },
  {
    "file": "server/game/core/gameSteps/abilityWindow/ReplacementEffectWindow.ts",
    "sha256": "b9449a037241b05bfe03e3f26d555cb79529ef97751487451425c816a75c202b"
  },
  {
    "file": "server/game/core/gameSteps/abilityWindow/TriggeredAbilityWindow.ts",
    "sha256": "db373bfda90c4c0b76ea91d40c1c6eee4618a7b15ebfecc3b97508889ece75f1"
  },
  {
    "file": "server/game/core/gameSteps/abilityWindow/TriggeredAbilityWindowTitle.ts",
    "sha256": "1bb1006a53b81d31064cdaed7d2c16e19fb2c96c554f2747c6f80356caf6ff80"
  },
  {
    "file": "server/game/core/gameSteps/abilityWindow/TriggerWindowBase.ts",
    "sha256": "b487ee8572f825f9dd237d34fa0781c3ce6429b01d7136548158fa8f8d8f4aba"
  },
  {
    "file": "server/game/core/gameSteps/ActionWindow.ts",
    "sha256": "e4e7a96c12b5cfe7ec334da845fcb3834a36d038186adac850c0e0f8ae9fbd29"
  },
  {
    "file": "server/game/core/gameSteps/BaseStep.ts",
    "sha256": "8e83914eca5be6e0ecd0302026cf73dfff835086617009b3b2e96078f4b6e360"
  },
  {
    "file": "server/game/core/gameSteps/BaseStepWithPipeline.ts",
    "sha256": "fc8596bb287d27edad19801bc30c47fccdb55d0343910b0b44749888515d25aa"
  },
  {
    "file": "server/game/core/gameSteps/IStep.ts",
    "sha256": "ea8a786dbd8dd2d193d4da419272464ac40684a1bb73285e28306a4d00a4ed24"
  },
  {
    "file": "server/game/core/gameSteps/phases/ActionPhase.ts",
    "sha256": "b2f8f2d37c9f3e9e38fbb6bd2ea8eb13ced608b957eb4791289804470bc99ad7"
  },
  {
    "file": "server/game/core/gameSteps/phases/Phase.ts",
    "sha256": "8b2b634da690306bc80beaf7a1e83b68bd29f5bf53e2f213ab06c5be10ed7b86"
  },
  {
    "file": "server/game/core/gameSteps/phases/RegroupPhase.ts",
    "sha256": "6d76049cf87f731a9731173b79f506420bdc3b7fab92e511af3be5a95fd802ac"
  },
  {
    "file": "server/game/core/gameSteps/phases/SetupPhase.ts",
    "sha256": "8ddcc201bd1a37598c49bc7687a3bf1b4e20bc70ec65cf23d6ffd330527a2b42"
  },
  {
    "file": "server/game/core/gameSteps/PromptInterfaces.ts",
    "sha256": "0a8d4dabceecb84e6e260ab834ceba710ec8bfc22ca34692e8099f21d55d5974"
  },
  {
    "file": "server/game/core/gameSteps/prompts/AllPlayerPrompt.ts",
    "sha256": "f845d0bbe57320300247c0532d7953c909748535a591e2b6e6b8ac17f0842a91"
  },
  {
    "file": "server/game/core/gameSteps/prompts/BatchTriggerResolutionPrompt.ts",
    "sha256": "b1188db4b29da7787b2897f5c7713651ca56fd23eca5a2ff95da1a890e117a22"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DisplayCardPrompt.ts",
    "sha256": "3483b813e086b9e275d9885ebe9ea9acea7572ea8f96b7043d29ab57f032569a"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DisplayCardsBasicPrompt.ts",
    "sha256": "53c25b63c03fca2a19daa7df4d81a1dd7e9d3a3868bbcbc93336c55ab5d96553"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DisplayCardsForSelectionPrompt.ts",
    "sha256": "1214d814af292f4a72502f79a804253c9bcf96db45c7efac5486d7eb476a1343"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DisplayCardsWithButtonsPrompt.ts",
    "sha256": "6fef989b1eabf4323f9a15c9a8a214e650346cf252d073fcd64c27073a8027c8"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DistributeAmongTargetsPrompt.ts",
    "sha256": "63b7534056d83395f9550849994381969c49de3a6af3515685d88b3fbb30f6d2"
  },
  {
    "file": "server/game/core/gameSteps/prompts/DropdownListPrompt.ts",
    "sha256": "af1168d0345d40d0094078693d1dff8d0a36f0e04cda92fa8f7f2e4a8cf54c75"
  },
  {
    "file": "server/game/core/gameSteps/prompts/GameOverPrompt.js",
    "sha256": "917b4379706944ae2740caa86bf7893166f43e8558a96e853af7f7e491856e2b"
  },
  {
    "file": "server/game/core/gameSteps/prompts/HandlerMenuMultipleSelectionPrompt.js",
    "sha256": "5d361f7efadb05615467bb2110b5fa5105ac0c2225eabbad2f04c589717e475e"
  },
  {
    "file": "server/game/core/gameSteps/prompts/HandlerMenuPrompt.js",
    "sha256": "4f550211e5ccfd7daf6b75c4e817834f8ccb0ae770201e426f15b12258613da0"
  },
  {
    "file": "server/game/core/gameSteps/prompts/MenuPrompt.js",
    "sha256": "6c9cb0905b17ede17670cf3c37847d8d9f4b117cea3356d27652ad3196079075"
  },
  {
    "file": "server/game/core/gameSteps/prompts/MulliganPrompt.ts",
    "sha256": "64592892c830c7c8015e18b5f5e965236bce8523c809b48d41c51f15c5e9c52e"
  },
  {
    "file": "server/game/core/gameSteps/prompts/NumberPrompt.ts",
    "sha256": "20091123dca024271439217d5d64b3cffde35b89cf48e1bc06d76b0ec809db76"
  },
  {
    "file": "server/game/core/gameSteps/prompts/OptionalTriggerPrompt.ts",
    "sha256": "afcdd9c923e59c05b4e006e7c0597361023e7a2b2e8370dc6da30a89f90194bb"
  },
  {
    "file": "server/game/core/gameSteps/prompts/PassDelayPrompt.ts",
    "sha256": "d1274a3876554f17c39ffd6e7ebe7cc016db2e9ff261082fa1f46f155abc6c95"
  },
  {
    "file": "server/game/core/gameSteps/prompts/ResourcePrompt.ts",
    "sha256": "218d7324e40708bae20444d007ea6c24ab6f5c623679d1380f3055e8943f9c57"
  },
  {
    "file": "server/game/core/gameSteps/prompts/SelectCardPrompt.ts",
    "sha256": "4d99bf8d0b03b929f1ca87863587204d82daf16f66e1970ec3eff676c0577a21"
  },
  {
    "file": "server/game/core/gameSteps/prompts/TriggeredAbilityResolutionPrompt.ts",
    "sha256": "2f38b70c00c3acbb82623e9984228dc699f4cbc961847da92b4ecc78a5a958f3"
  },
  {
    "file": "server/game/core/gameSteps/prompts/UiPrompt.ts",
    "sha256": "e0341220b4f546bed64d29a1dc81b45479db4c40d8e5ad3e84d48673634f2742"
  },
  {
    "file": "server/game/core/gameSteps/prompts/UndoConfirmationPrompt.js",
    "sha256": "c6dc8ba3c55ec36615547eea7ad921e75befe0070da8ec94ec12fc755dfffa9c"
  },
  {
    "file": "server/game/core/gameSteps/prompts/VariableResourcePrompt.ts",
    "sha256": "b71fd36b0c500342244249f4a4bac45152c6057131233792acb07de3ebe0afc6"
  },
  {
    "file": "server/game/core/gameSteps/SimpleStep.ts",
    "sha256": "639e12243d01984f19770c69f6abef6a1a4ccd7d691e703726a5122cb809ccdf"
  },
  {
    "file": "server/game/core/gameSystem/AggregateSystem.ts",
    "sha256": "f9eb1fd9c0f807207dcb6944c129422d88133d8f1f8eaabdf100bca57d9ec9cf"
  },
  {
    "file": "server/game/core/gameSystem/CardTargetSystem.ts",
    "sha256": "e87cfa4984927681af8dfb06bfa2a242f00ec867bf28271db9442c436f245d63"
  },
  {
    "file": "server/game/core/gameSystem/GameSystem.ts",
    "sha256": "25d6049630d2544f084b67a994953069254781f0356450d91c7ec0049173101f"
  },
  {
    "file": "server/game/core/gameSystem/LastingEffectPropertiesBase.ts",
    "sha256": "f3504b6dab6df829a91119a7ee69207bccdcf2d06ce6578c55eaa5ded1cc6277"
  },
  {
    "file": "server/game/core/gameSystem/PlayerTargetSystem.ts",
    "sha256": "ac3b63da8d7c697f39c3472d6f10f253ecbafa851ec56acba8a878d706dbd313"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/AdditionalPhaseEffect.ts",
    "sha256": "154efa8351ce3a9a1768ebfda4cc4c8295758c4dece17499aef8748a1686bce9"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/CanAttackMultipleUnitsSimultaneously.ts",
    "sha256": "5df075c7b0b6ee1f70c49129b95b070f6145247570bcaefa96e99fdf12eedb08"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/CloneUnitEffect.ts",
    "sha256": "b66717031e728a72a04d6849eb3febef980d5bc0a3af85acd2d8b3e751d085ab"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/CopyStandardTriggeredAbilitiesEffect.ts",
    "sha256": "2f11a088db9fe8dbb729a48f5a1fc6259cc959d06951b989a80401eacaeb9d3a"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/DetachedOngoingEffectValueWrapper.ts",
    "sha256": "5f7336e202b976d42ce2092e5a694c0e6c95d93d8c63d1c42d20db9e28ebd56f"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/DynamicOngoingEffectImpl.ts",
    "sha256": "6b684f2a5077c18a6e98240e13092169ad9306b2839c8264aa115f7527b4bd4a"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/GainAbility.ts",
    "sha256": "2a832168f31653686ecaabbdcd8ee6fb7860860b4fed13b78dbb07d5864e470c"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/GainAbilityDescription.ts",
    "sha256": "a8f87b1a76fb1391fd2ac1357b2e7019aec92d9a6dd6a82b25558f07f1915d02"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/GainKeyword.ts",
    "sha256": "116434579850ee9e31281a9b88873c6fee43d457b0ef7d45ac351da20eb21826"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/GainNonKeywordAbilitiesFromUnitEffect.ts",
    "sha256": "a7f283be1a295fc1c1893e462716a94d9f75c061ce792a3fd0629cea9bb102d0"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/IndirectDamageModifier.ts",
    "sha256": "03cae011299fde034f15b79c45965b074a01f098c02e39ee99db8a090ec3e0e6"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/MustAttackProperties.ts",
    "sha256": "2d5265825b442bf58e730f8de42a7adbd64fa4fb9153c75dc7d1352b1e4cdd74"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/NumericKeywordMultiplier.ts",
    "sha256": "dac1e3305eb4abd1e169c6f2cfbb2c07fdb0f935b162e966eb3a6659e4944f52"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/OngoingEffectImpl.ts",
    "sha256": "8634df62a322df606b7cacc2d00b1c76860b1fef9ecb623982d6d57f3a768b66"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/OngoingEffectValueWrapper.ts",
    "sha256": "c3d6449525adcb0035df60e8f8f2cd19c681626fd5141b49c3a5942505b0b483"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/PilotLimitModifier.ts",
    "sha256": "485191c86ebf17e15792e1ddbf0dd21924f93b39a525122b487abe77cd7582fd"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/PlayFromDiscardProperties.ts",
    "sha256": "3c8f28f8121ad88d49a3857a9124d7113f3157149a6f6ebe0b10ae522a540013"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/PrintedAttributesOverride.ts",
    "sha256": "7ac8cda6aff0d096d29d628619decb9ca694c74bc1945eff49eb037a911183e3"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/ProvidedAspects.ts",
    "sha256": "2198eedc445bed76c7f2f39e9d556780575228aafa953a9449dda833433967bd"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/ReplaceKeyword.ts",
    "sha256": "c66ce236cfd78bf928646bcf8c159399758805ea32115664652cc5dfa9a4ced4"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/Restriction.ts",
    "sha256": "16b962974c43a79b8482e54d6a4622dd618ca9070f1972c958a3ac893273f3e9"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/StartingHandSizeModifier.ts",
    "sha256": "ab1863f44ac189571d760805f05fcf4fb7b2e0b272cc5f6686f3d07a516cd424"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/StaticOngoingEffectImpl.ts",
    "sha256": "07a3c6bbd574f9408c3ba72d7cf95107f8f5a78d88d9e1aba0f63f15c9dce8b4"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/StatsModifier.ts",
    "sha256": "69f1886a75cf5fc890d23eacf5360296fe7cc38c95ba758cb8eca8c82a169bf0"
  },
  {
    "file": "server/game/core/ongoingEffect/effectImpl/StatsModifierWrapper.ts",
    "sha256": "b0ef9c54c989602d64b6141caad38080c69e7d59c1504532d4859e6e68d08a67"
  },
  {
    "file": "server/game/core/ongoingEffect/IConstantAbility.ts",
    "sha256": "aa50539b0559a777547c1b5b71ba2989bd6269c0e3e0ba5ecb1c5cad4f92ca31"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingAllCardsForPlayerEffect.ts",
    "sha256": "5f0ecf7984f5310e3b1d1bb858ef377d1b39dbe095d9e935717d45aeeb43ef20"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingCardEffect.ts",
    "sha256": "0ea847ba86549ad7900a3513eb0029b3c4b466dd9fef434a75abf0a829e9b968"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingEffect.ts",
    "sha256": "dbe2714c8f8180687fb8ba19ec5049ea6e077733f01c3dd86e559d3fc10bae34"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingEffectBuilder.ts",
    "sha256": "98409e0de9db73134fd80d9ef1d3ae64120df9ca369d3ea5797b53406763a0fb"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingEffectEngine.ts",
    "sha256": "bdf123885833bee1a98fa2c0e8577d43c87239efa632cc023d986d17cdc5648c"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingEffectSource.ts",
    "sha256": "4e2dfbda96b527410e32fd11c283c72712fba0e91c3438f9026581c48c620502"
  },
  {
    "file": "server/game/core/ongoingEffect/OngoingPlayerEffect.ts",
    "sha256": "7809d7dbb253a9db1fddadf96a05017e0f4a7c8a0edad3574a10df2b932a4dae"
  },
  {
    "file": "server/game/core/PlayableZone.ts",
    "sha256": "3fa80e3687c39a7b1fadc3cec86855e13d0406d4af93deb85bf1e2c3e613526d"
  },
  {
    "file": "server/game/core/Player.ts",
    "sha256": "be965431f458e9f3a693624b6353782817c92f1940c9e8d6a9339a31545489bd"
  },
  {
    "file": "server/game/core/playerEffect/UnitsEnterPlayReadyForPlayer.ts",
    "sha256": "e2b0c72916ea11d353a3f4390b36fcac14c05c2ca53dd8a7dbba33c7dec59bce"
  },
  {
    "file": "server/game/core/PlayerPromptState.ts",
    "sha256": "b90b5f253e67f3808e3c725977638308b50cc9c01a0c50359f80cb7430bb2819"
  },
  {
    "file": "server/game/core/Randomness.ts",
    "sha256": "9fe6f7b9e0f287234df14ea7f1553d469bba51b925e8e8e282c5d3afbe96170e"
  },
  {
    "file": "server/game/core/snapshot/container/MetaSnapshotArray.ts",
    "sha256": "3555b26b88482a96f5f56617bc171b0b33b84bb561d65642ab1a1f93e5066a76"
  },
  {
    "file": "server/game/core/snapshot/container/SnapshotArray.ts",
    "sha256": "2a34972487aecab7386f53a605493fefac6aa0bad827157df966ebbfdb99ac2a"
  },
  {
    "file": "server/game/core/snapshot/container/SnapshotContainerBase.ts",
    "sha256": "ef88c2e82ef08c7125ab2a04367c72cb550ae1fb1870961db1dc618a608303b1"
  },
  {
    "file": "server/game/core/snapshot/container/SnapshotHistoryMap.ts",
    "sha256": "1c707557ce01d8f4e775f6b254012190b211de7099e3237ab46583eead0a3ce4"
  },
  {
    "file": "server/game/core/snapshot/container/SnapshotMap.ts",
    "sha256": "e8607b8812862e6980b6a0f9b29f05a112e82754d8c3444d9364f5c21a49b53b"
  },
  {
    "file": "server/game/core/snapshot/GameStateManager.ts",
    "sha256": "c70d1d1a1ee7eb3195c2d865036febde85314a50b12de6b3e69b7a812f25c57d"
  },
  {
    "file": "server/game/core/snapshot/SnapshotFactory.ts",
    "sha256": "5244fa91022040f5f0d6704d222ee23fa22ac30bb04f604c9f7c9cb9feae0241"
  },
  {
    "file": "server/game/core/snapshot/SnapshotInterfaces.ts",
    "sha256": "1581c0124853904292e398726ccb8cbac73c68fda3a39889e41a44cce0a45758"
  },
  {
    "file": "server/game/core/snapshot/SnapshotManager.ts",
    "sha256": "93e0e9cb68ad4baae58f5b8247e1decd5a523b8b6e8a025e91dc14ea3eb5efe2"
  },
  {
    "file": "server/game/core/snapshot/UndoLimit.ts",
    "sha256": "6662fd34d36dbb42e77e08130caeca34dc0463c0db2d5fbed39831a83766d816"
  },
  {
    "file": "server/game/core/stateWatcher/StateWatcher.ts",
    "sha256": "d57b2709ad15214f62bad0f713cdd30645ce146cfb67c3b4eab5f00ec3323698"
  },
  {
    "file": "server/game/core/stateWatcher/StateWatcherRegistrar.ts",
    "sha256": "4eacdad29b0961316d894b2730c8894df2c1c69c6b75f6b8513ed46ef7dd704c"
  },
  {
    "file": "server/game/core/utils/Contract.ts",
    "sha256": "8a6e9953de29a0200b72fdc8412652060457d9e0ff742d7ad460920f07d4c9a6"
  },
  {
    "file": "server/game/core/utils/EnumHelpers.ts",
    "sha256": "48668c6b99174dadb89475c1089371d9a0de9a93417728d070eaf3d204c5b33c"
  },
  {
    "file": "server/game/core/utils/Helpers.ts",
    "sha256": "262150d3260cb5ed4011855f8431f0b908d6c9a11a36dddeabea96e5b298a4d2"
  },
  {
    "file": "server/game/core/utils/TextHelper.ts",
    "sha256": "a7dbb4a7949063ca2540660d31cb3847a814979325411e7b14f94f96c0c77ad1"
  },
  {
    "file": "server/game/core/utils/TypeHelpers.ts",
    "sha256": "3306711ccad075e61689bfa7e1cce36dad7c9f792aa60adf83c71407ca0a0311"
  },
  {
    "file": "server/game/core/zone/AllArenasZone.ts",
    "sha256": "ceffb9b9acf458a04e7b00f9eb73af55137bb83bc4373a0907b644e75af907b9"
  },
  {
    "file": "server/game/core/zone/BaseZone.ts",
    "sha256": "5a2ab89f4073af4815489465966f2d0679d449938acc9b754cc0e9823328b67a"
  },
  {
    "file": "server/game/core/zone/CaptureZone.ts",
    "sha256": "09b7ed85e7c8ba23b077c37ef72d9f998d4c46865041632cf5039ffc78974140"
  },
  {
    "file": "server/game/core/zone/ConcreteArenaZone.ts",
    "sha256": "5127344fbacb34ea3700c51554aa715f6d5f41363ef5bdc56acdcd15cddd09c2"
  },
  {
    "file": "server/game/core/zone/ConcreteOrMetaArenaZone.ts",
    "sha256": "790cc3b515c9646ccc5a72575f22a35d3f143c2acc9a4c0a55dc3e2c159b5a96"
  },
  {
    "file": "server/game/core/zone/DeckZone.ts",
    "sha256": "1d15abd9d5ce5d5cf891da7d20477d14ef0f7936beac734f72cfc23ecf28040a"
  },
  {
    "file": "server/game/core/zone/DiscardZone.ts",
    "sha256": "bba2d38693e9686a32f64683e3fb1d95b49277e3ff148e5836982aff4e67f520"
  },
  {
    "file": "server/game/core/zone/GroundArenaZone.ts",
    "sha256": "11988edc0d6970fe130364096c893a528b95754de512c87eb9c6ed96e7ad311d"
  },
  {
    "file": "server/game/core/zone/HandZone.ts",
    "sha256": "1993c167a6ed2647ad8ca1d2304203fe7cdde40d65ecdda643fe2e081b600e7d"
  },
  {
    "file": "server/game/core/zone/OutsideTheGameZone.ts",
    "sha256": "dc4302da7d41c894b45999d7dcacc6bf366855e9f6eca9d6d0040fe547822820"
  },
  {
    "file": "server/game/core/zone/PlayerZone.ts",
    "sha256": "a91c45425a4ca3b56ec7235f1029dc5b42e358330d14136dba82892640d2c54e"
  },
  {
    "file": "server/game/core/zone/ResourceZone.ts",
    "sha256": "2ac26780548f562d1dff9922a0fff20c6c9654a26dec4bbf3785b3a61cd6b4c1"
  },
  {
    "file": "server/game/core/zone/SimpleZone.ts",
    "sha256": "d53e835cc1d2e8a3e2c368cca1ff69e911045a1befb7b1c5c75101635febf0c2"
  },
  {
    "file": "server/game/core/zone/SpaceArenaZone.ts",
    "sha256": "038a7f5a0fc52d9c0abfb385eab4e38a9f23de46cf06fe89c8eb22a3735a5e0e"
  },
  {
    "file": "server/game/core/zone/ZoneAbstract.ts",
    "sha256": "760961384091c58b4159adf03a45b9c79721badca98cc9fc1fa790aa8d1e883b"
  },
  {
    "file": "server/game/costs/AbilityResourceCost.ts",
    "sha256": "4d864a90c8b575ebb9543e7949c326b1e49efc1f92d1e2c81565a1bda59dd8ec"
  },
  {
    "file": "server/game/costs/CardEffectResourcePayment.ts",
    "sha256": "f368561202ceaaca192ab5a8232b67ea4ac71feb9e64b1ed7c3f9843218ba072"
  },
  {
    "file": "server/game/costs/CostLibrary.ts",
    "sha256": "f65a923096444f82dfdc81c0b8bd6fbd600fa50a0ab87ff00dc0620ee2841786"
  },
  {
    "file": "server/game/costs/PlayCardResourceCost.ts",
    "sha256": "42290f4a0381c657a6bf52dc26bf597c32a879d39449ddc40cde82ebaa97e85a"
  },
  {
    "file": "server/game/costs/ResourceCost.ts",
    "sha256": "2131c70e5d0614a2910a04516b4c92bdaa31a0f050ad27cc06c8c0d3529d0eb5"
  },
  {
    "file": "server/game/gameSystems/AllCardsForPlayerLastingEffectSystem.ts",
    "sha256": "754188ada5760acf89c7e93509d6af6bbb78e05d9fdcce79d2db0b2b6dcacd9d"
  },
  {
    "file": "server/game/gameSystems/AttachUpgradeSystem.ts",
    "sha256": "0376f9c652e3e25ce3fce6a91a8c182c578d29ff7985eab6da68e7a9b34a63bf"
  },
  {
    "file": "server/game/gameSystems/AttackStepsSystem.ts",
    "sha256": "f6209f9b7ec9e8e3fbf30e193294a1df466d02e81988d544edd0a74d26988b38"
  },
  {
    "file": "server/game/gameSystems/CaptureSystem.ts",
    "sha256": "12ff68e767b29ebf46a09bcd6881ebfd6b0e597cf5c6b57b124ea2394ec2bafb"
  },
  {
    "file": "server/game/gameSystems/CardAttackLastingEffectSystem.ts",
    "sha256": "3b99856085eac6415bd7014f214ffe2df3b70afb7c436ccebbcbe30fa9cde932"
  },
  {
    "file": "server/game/gameSystems/CardEffectResourcePaymentSystem.ts",
    "sha256": "bd9506094637a561e50b124e94d94c3412196c4a8b1ab4f8057c96d368d67efa"
  },
  {
    "file": "server/game/gameSystems/CardLastingEffectSystem.ts",
    "sha256": "d47af53160ce8d7a83413097e8e923d0aed4528110ce2d14410c2beca98a5167"
  },
  {
    "file": "server/game/gameSystems/CardPhaseLastingEffectSystem.ts",
    "sha256": "11093bf36c75a916ee9bfc3d62dd96f4e9a2704613326f816662493ba022fff1"
  },
  {
    "file": "server/game/gameSystems/CardRoundLastingEffectSystem.ts",
    "sha256": "222056f6076589e4407f5448c4f68b4d56381413e40fed8f90fe36db1e460dbc"
  },
  {
    "file": "server/game/gameSystems/CardWhileSourceInPlayLastingEffectSystem.ts",
    "sha256": "f0590b24f07ba541bbab2762060461ea0b96d98ea45126a4cf0f6bebb6c24255"
  },
  {
    "file": "server/game/gameSystems/ChooseModalEffectsSystem.ts",
    "sha256": "63c438af7dbf7c900cc6db0bea1a5e59196befbb7b56878fa1c1cb665c02dd5b"
  },
  {
    "file": "server/game/gameSystems/ChooseNumberSystem.ts",
    "sha256": "c1e8f17f936be0a6e395ff7bfca3906921f971abc25bccb27e46beb39c9a4324"
  },
  {
    "file": "server/game/gameSystems/ClaimCounterSystem.ts",
    "sha256": "04267f094ec04b930b2f6f427c11fb35f22cac67caf2d0ad383b66a71ea816c6"
  },
  {
    "file": "server/game/gameSystems/ClaimPlanCounterPutOnBottomSystem.ts",
    "sha256": "b4a55087aa0bd08c6f7b7055963d40b0d7745f6bdb829a190c268f94ad5492bf"
  },
  {
    "file": "server/game/gameSystems/CollectBountySystem.ts",
    "sha256": "899f11af3210fd4e32e6110369cbfac5a6682fdd8fef8af7ba844ff1eb64cfc2"
  },
  {
    "file": "server/game/gameSystems/ConditionalSystem.ts",
    "sha256": "2f66600363c5b04c395c4461bc2e03050577a03e1bc383efa282daddb1dd594a"
  },
  {
    "file": "server/game/gameSystems/CreateCreditTokenSystem.ts",
    "sha256": "072b205df0ea591988796e4713ed2ceec6fcfd31c103226a653c966dc56fda16"
  },
  {
    "file": "server/game/gameSystems/CreateForceTokenSystem.ts",
    "sha256": "f9dbd1956cb590a312d03b919db8fba0e24ebc5bb35dbc9c60977bc8fb0f9e41"
  },
  {
    "file": "server/game/gameSystems/CreateTokenUnitSystem.ts",
    "sha256": "178e8bbbf971ec80cdc608cb2b4559ecb3ca5e2fcfdecab288adc6e56ea70fbd"
  },
  {
    "file": "server/game/gameSystems/DamageModificationSystem.ts",
    "sha256": "41060cf1b604b2ecf8bad751a7026c7b0f2a0015ab32ad5fd8a27aefbc5f2cf4"
  },
  {
    "file": "server/game/gameSystems/DamageSystem.ts",
    "sha256": "e7c8f249933b14ef9424a292531e1f976d494eb4656b21e040b14c699f00a83c"
  },
  {
    "file": "server/game/gameSystems/DefeatBaseSystem.ts",
    "sha256": "0c76c158e5696c5b6bde3012719f3ff29956358adc1847153a17152e04957036"
  },
  {
    "file": "server/game/gameSystems/DefeatCardSystem.ts",
    "sha256": "3e7d5c860539aba5ffd0b9272601000faf53180d01b6f9e520451cd7e7539b87"
  },
  {
    "file": "server/game/gameSystems/DelayedEffectSystem.ts",
    "sha256": "ef62a6f825ef2726b27ed6404f3d694317fffce6209326d0ff0087fa8eaee3f7"
  },
  {
    "file": "server/game/gameSystems/DeployAndAttachPilotLeaderSystem.ts",
    "sha256": "09688c5a25b1464cad53e9c55665f258bacc545f2dedf33359e4ced7ef192fec"
  },
  {
    "file": "server/game/gameSystems/DeployLeaderSystem.ts",
    "sha256": "11257c941c2a27fd7fb8427286f98774da6b4c6a0cb6fe2ea6804333be376fda"
  },
  {
    "file": "server/game/gameSystems/DetachPilotSystem.ts",
    "sha256": "b8268cdef81b350da3d3ee6ad5ce30c9a9fee36176afc04c251c9170584ab0d3"
  },
  {
    "file": "server/game/gameSystems/DiscardCardsFromHandSystem.ts",
    "sha256": "60b701f1a9f59c59683fcc3973636b6147b58ad6cf62f87191afe3e8708c3d03"
  },
  {
    "file": "server/game/gameSystems/DiscardEntireHandSystem.ts",
    "sha256": "ca90ce948a0490ec2c3ae7dfdd2fad221ff570029279fb9b42f8401e1760048e"
  },
  {
    "file": "server/game/gameSystems/DiscardFromDeckSystem.ts",
    "sha256": "2e133054c25595bec03e2bfbb895a8fe1625b46344512bfc2f3c57ab4297838f"
  },
  {
    "file": "server/game/gameSystems/DiscardSpecificCardSystem.ts",
    "sha256": "20a42ed816129dd6bc614257df203fbb2d8252e72ebacef917905731e882a774"
  },
  {
    "file": "server/game/gameSystems/DiscloseAspectsSystem.ts",
    "sha256": "ab7468a24891e6687ce670d91a27cacadb53a7714356def519df373ffbacb932"
  },
  {
    "file": "server/game/gameSystems/DistributeAmongTargetsSystem.ts",
    "sha256": "7eff9809c0628280f63f42e68cce357919b38498f36525caa0840f327f78c787"
  },
  {
    "file": "server/game/gameSystems/DistributeDamageSystem.ts",
    "sha256": "c3135a7ec8e0df3653fb1390233f7cc254c301db7cf69f8f56cfc3ab1189df6e"
  },
  {
    "file": "server/game/gameSystems/DistributeHealingSystem.ts",
    "sha256": "216ac434002d5e0335dca86d7da0dec87ec241a33cd08ac5053f011fcbe26ddb"
  },
  {
    "file": "server/game/gameSystems/DistributeIndirectDamageToCardsSystem.ts",
    "sha256": "240d5fec49158bcb5ce1a10bc0bc7cea6e4cd2fc7d709537c8e8b83bb01f65ca"
  },
  {
    "file": "server/game/gameSystems/DistributeTokenUpgradeSystem.ts",
    "sha256": "4f92e71450125e5cf6d9a341819297431d05a35b388a11b9b8ea10d605e76238"
  },
  {
    "file": "server/game/gameSystems/DrawSpecificCardSystem.ts",
    "sha256": "63845aa73e26a23c10ccb64611b78f355863e6636fa82fa6dab6fe6d162b74b7"
  },
  {
    "file": "server/game/gameSystems/DrawSystem.ts",
    "sha256": "eef13a62a127b3a04bfea16329c03b9d06e5d0d2ec71270c91e142f553de8db8"
  },
  {
    "file": "server/game/gameSystems/ExecuteHandlerSystem.ts",
    "sha256": "01fec9eccc12af2a38315ad455d9b9edfe4d1102ed4b41e242bfd29072be1e1c"
  },
  {
    "file": "server/game/gameSystems/ExhaustOrReadySystem.ts",
    "sha256": "1d237a892c1b106c655ba5242183b139bcf9fde22dbfb8541be1cb043c823cab"
  },
  {
    "file": "server/game/gameSystems/ExhaustResourcesSystem.ts",
    "sha256": "dee0aec70a94d5e2830984e3ae303109e3d6230970b3fe8238b88bcb67553717"
  },
  {
    "file": "server/game/gameSystems/ExhaustSystem.ts",
    "sha256": "c023e6eb17139e2db35b4ed163a3908097cedbce11454ef794fc00f2a95b5dd0"
  },
  {
    "file": "server/game/gameSystems/FlipAndAttachPilotLeaderSystem.ts",
    "sha256": "1d93ff449ce9aa3b0d187be56018a0cd1d573c36025483e5232532ffcb63d580"
  },
  {
    "file": "server/game/gameSystems/FlipDoubleSidedLeaderSystem.ts",
    "sha256": "515f115501fe7e732d3af8c105b13de7c69a97f1a8df3c6142d0d4923db47808"
  },
  {
    "file": "server/game/gameSystems/FrameworkDefeatCardSystem.ts",
    "sha256": "c8974173c60fb52ebcfa259543f5b6e99784c4c7c38892a45c38e2daaab035a4"
  },
  {
    "file": "server/game/gameSystems/GameSystemLibrary.ts",
    "sha256": "2cba31deb4ad198180d1a21b3a8a52585aea3399520dfb61ccb48bcb73f9525b"
  },
  {
    "file": "server/game/gameSystems/GiveTokenUpgradeSystem.ts",
    "sha256": "b8048ebcc0daf229a546b5b4e3a72919cc501cf0dd0883ad5afad3d0d24968b8"
  },
  {
    "file": "server/game/gameSystems/HealSystem.ts",
    "sha256": "b5ae63a9e24f02389937944c8ff8e16a08e62bae347902333f306c2206aa5617"
  },
  {
    "file": "server/game/gameSystems/helpers/LastingEffectSystemHelpers.ts",
    "sha256": "97431c96abff88f56673ad92d939f6e5dc73e12f7fd4775399275f6af7695ae1"
  },
  {
    "file": "server/game/gameSystems/IncreaseAbilityDamageSystem.ts",
    "sha256": "e820bd82ff61237f01756be8336b8be8a9045a92171e6b1dc0bab9ea0b52ab96"
  },
  {
    "file": "server/game/gameSystems/IndirectDamageToPlayerSystem.ts",
    "sha256": "405585c8df6e54c1e3b308d452042123decbfb248f8f716cb21543a39e77d03c"
  },
  {
    "file": "server/game/gameSystems/InitiateAttackSystem.ts",
    "sha256": "2d444d7f1ffe37663d6b4e67709e03f5ff1ad56ab4e67537ce8551021436597c"
  },
  {
    "file": "server/game/gameSystems/LookAtSystem.ts",
    "sha256": "9a4dcc026b8ec2d4668ed66791a8ce25e8889c66be2f83366144c50704f5fe86"
  },
  {
    "file": "server/game/gameSystems/LookMoveDeckCardsTopOrBottomSystem.ts",
    "sha256": "61a7ed408beb90fe2d5459788c31e61d728eeb95fd6ad1a2fda2d8ea055b6c61"
  },
  {
    "file": "server/game/gameSystems/LoseGameSystem.ts",
    "sha256": "f52e8bcec135aace450bf135259b13990a0600c37ea9410a850b706b234f25c3"
  },
  {
    "file": "server/game/gameSystems/MoveCardSystem.ts",
    "sha256": "fffa3f43ec7e0461a76762d973fc57ec0835831629cfeef2fd1299959dbfa973"
  },
  {
    "file": "server/game/gameSystems/MoveUnitBetweenArenasSystem.ts",
    "sha256": "f041c444dc5cc3266b70212dc5c0c83e505587c81cfe00c870879a12655902b7"
  },
  {
    "file": "server/game/gameSystems/NoActionSystem.ts",
    "sha256": "f7b66c6939f7d90668e394eecb39ebab4b1a1f0da67ed1a7d2301be243a8e699"
  },
  {
    "file": "server/game/gameSystems/OptionalSystem.ts",
    "sha256": "8a79c9e8c6bda025318f67d86b9030230f87a438bc34ddf132858943f0423af8"
  },
  {
    "file": "server/game/gameSystems/PayCardPrintedCostSystem.ts",
    "sha256": "99bbc8555b1133261e621d33ba6eee2143a95a91809270dd33cffa0b81da9b44"
  },
  {
    "file": "server/game/gameSystems/PlayCardSystem.ts",
    "sha256": "4a58b41e2b3d78b113e4c09d647b478d67416f095458daf0c377a088c398186e"
  },
  {
    "file": "server/game/gameSystems/PlayerLastingEffectSystem.ts",
    "sha256": "e9c4e95e489323e96520fdbc5f2fa53b96586c5c4d86e534c7cb5295ade9fc8f"
  },
  {
    "file": "server/game/gameSystems/PlayerPhaseLastingEffectSystem.ts",
    "sha256": "72a6300522e254e21420db0799fa9231ebd33dd04c60ef79f4b28de8f48319f3"
  },
  {
    "file": "server/game/gameSystems/PlayMultipleCardsFromDeckSystem.ts",
    "sha256": "31fe649016cff221c8083695f4e3257369b0ff147db39864a1e8262d83e7b622"
  },
  {
    "file": "server/game/gameSystems/PlayMultipleCardsFromDiscardSystem.ts",
    "sha256": "1909d44c3e9ea1f91b9df2d8441b23d3ec7e7cff76b931c0a0a6607ad894a342"
  },
  {
    "file": "server/game/gameSystems/PlayMultipleCardsFromHandSystem.ts",
    "sha256": "87e27ceebda38fd9f67dfb580da5315b19a33413ad2298b7f418e7caebdb66bf"
  },
  {
    "file": "server/game/gameSystems/PutIntoPlaySystem.ts",
    "sha256": "8781a3f5924a1d9e82712478c4c8bec1ae60125ada3321d600b98153bc68d1b2"
  },
  {
    "file": "server/game/gameSystems/RandomSelectionSystem.ts",
    "sha256": "625fdd9409b5f25eb37d62e26d9cdfa537a5272a873cd1b73c3c94cded88e18f"
  },
  {
    "file": "server/game/gameSystems/ReadyResourcesSystem.ts",
    "sha256": "1219409e6383dc654e63caf6080c2dcbb7ed7604cfb580aeea64497e2e634e02"
  },
  {
    "file": "server/game/gameSystems/ReadySystem.ts",
    "sha256": "2df3e68b0c2da3b5c51a048e4b981aa89001ce13c612abf288b0a8364acdc134"
  },
  {
    "file": "server/game/gameSystems/ReplacementEffectSystem.ts",
    "sha256": "241278f4ae72fa935571f110c5c131123c825bd6802f3adaaa3b47261472d449"
  },
  {
    "file": "server/game/gameSystems/RescueSystem.ts",
    "sha256": "11d26ae5ffe729df5fb44defdf0a4852db20063d15264b3fc458a8584e2c36e5"
  },
  {
    "file": "server/game/gameSystems/ResourceCardSystem.ts",
    "sha256": "1de4a91d3d1a508fb3a7bd219ac54513062a27f772dda5382a44865de01e5f74"
  },
  {
    "file": "server/game/gameSystems/RevealAndDrawSystem.ts",
    "sha256": "7345fb55458d4cdd1bf40498bab02d641dc840dbbd4ea4b218d59c85db0d2a84"
  },
  {
    "file": "server/game/gameSystems/RevealSystem.ts",
    "sha256": "78600d4d360fdf9cdb3856c8ecc3a90d0ad72386e5468e89f729f350c2da427e"
  },
  {
    "file": "server/game/gameSystems/SearchDeckSystem.ts",
    "sha256": "815ace9432bbddb74f98bd19798be8ef1b7d2a459127c542b71a2c1c3827b0a1"
  },
  {
    "file": "server/game/gameSystems/SelectCardSystem.ts",
    "sha256": "21359f9d0fbefbbc24051df817f1c40b2757e736a67d52513cc075ac86882219"
  },
  {
    "file": "server/game/gameSystems/SelectPlayerSystem.ts",
    "sha256": "c59591fca9cd7203689a5d6a92318f8ba20c996adb4d4ec39b0aa3a9df5d2b74"
  },
  {
    "file": "server/game/gameSystems/SequentialSystem.ts",
    "sha256": "e8cf2c1a1865b612ebe77173d53536d42e8ec2c56b7c8f0df730a3cff45cb1ee"
  },
  {
    "file": "server/game/gameSystems/ShuffleDeckSystem.ts",
    "sha256": "0e169660450c50e9788d3fecdd954f3853962bcb2ffcec59a7ba25dd45f15623"
  },
  {
    "file": "server/game/gameSystems/SimultaneousOrSequentialSystem.ts",
    "sha256": "da172bd6d07c6f54c68e48bd26cd54028efa7ee4bbe9790b08d5b136c616b6dc"
  },
  {
    "file": "server/game/gameSystems/SimultaneousSystem.ts",
    "sha256": "ddc2f93ac1cc09fd3da9658982d86b0021c8bf06aa80e156446af6b8bd17424f"
  },
  {
    "file": "server/game/gameSystems/TakeControlOfCreditTokenSystem.ts",
    "sha256": "358eb9ca2ac0b03a22ffe50f98ac56afde2b0672985defa54dc29f36b1dae7e8"
  },
  {
    "file": "server/game/gameSystems/TakeControlOfResourceSystem.ts",
    "sha256": "dce40190a940b2cca730f503e6c5a086eb39d58a854d1b7b37f807c736b5a881"
  },
  {
    "file": "server/game/gameSystems/TakeControlOfUnitSystem.ts",
    "sha256": "681faf5cb5688e2d2e6e6a1bb7a062e9e092ff7390a70b20792fe1d3462117ae"
  },
  {
    "file": "server/game/gameSystems/UseOnAttackSystem.ts",
    "sha256": "bd26876ca37cf412c221cc4c97a7b2a0272eafb46b6829e0ba760a1f6a32989a"
  },
  {
    "file": "server/game/gameSystems/UseTheForceSystem.ts",
    "sha256": "9d0ade06b8d6d49faf554f228acde3c093d3328deba00133a7a18c1f1bedbc07"
  },
  {
    "file": "server/game/gameSystems/UseWhenDefeatedSystem.ts",
    "sha256": "ef0c877aa389b1f824fc703c69445c0bbd781b61d46eb9b1816a78d8377899f6"
  },
  {
    "file": "server/game/gameSystems/UseWhenPlayedSystem.ts",
    "sha256": "f34e6ea7047d8c3d2213c423a9e20ec073b62b58b3594740af8558d3753fc8b3"
  },
  {
    "file": "server/game/gameSystems/ViewCardSystem.ts",
    "sha256": "97873d0bda289e890484d9e202c367c09e2538966cac87fee4ea4354b6606766"
  },
  {
    "file": "server/game/gameSystems/WhenSourceLeavesPlayDelayedEffectSystem.ts",
    "sha256": "890deca66ec1c5d4a1b24a86a6b33281fc00bb54aa37634eef2b7cb2195b4560"
  },
  {
    "file": "server/game/gameSystems/WinGameSystem.ts",
    "sha256": "8b19c80c975ad8e7d2eeb9d46491ed3c3dc5e0fcd0152f089414a7fd90ecc7fd"
  },
  {
    "file": "server/game/IDamageOrDefeatSource.ts",
    "sha256": "0e086bc897895e2e3154a739b4a8e3f180d21c9b507167b8da54c9637fd01334"
  },
  {
    "file": "server/game/Interfaces.ts",
    "sha256": "8e72c7cab5aab6fc9a31b08924249691ccfc88683a64f4163261e32e1d40c657"
  },
  {
    "file": "server/game/ongoingEffects/CardCannot.ts",
    "sha256": "c32437fa9742d20d1eeed2261e4916298d0e4c59ac63165e0d4e50316706ca96"
  },
  {
    "file": "server/game/ongoingEffects/ModifyCost.ts",
    "sha256": "d0b6f4c71d3cff3e8f5169aaecd65cbaf830697f0c4cb1ae601a625d9056301a"
  },
  {
    "file": "server/game/ongoingEffects/OngoingEffectLibrary.ts",
    "sha256": "80ec23b1e4910814de9dba4264dc1f49945bc74524bc27fbb0b6b59c0f76fe30"
  },
  {
    "file": "server/game/ongoingEffects/PlayerCannot.ts",
    "sha256": "fd8935c58a0dcc8d91905eeedac29273af57c6fd35b40b2fd11e49ebf279992b"
  },
  {
    "file": "server/game/ongoingEffects/RestrictionDsl.ts",
    "sha256": "a864664891473f5fbb92e9c0a7a27006f81ae8597759449adf590da3d0847083"
  },
  {
    "file": "server/game/ongoingEffects/UnitsEnterPlayReady.ts",
    "sha256": "fa87fb7d580c40cfaa62617d59ccf09f357aa92ea5ac1256312cb3061872e9db"
  },
  {
    "file": "server/game/stateWatchers/ActionsThisPhaseWatcher.ts",
    "sha256": "7df0e537368830f0c7a74ff309bfd24327a2f6079995c6eac1394be72e3e7271"
  },
  {
    "file": "server/game/stateWatchers/AttacksThisPhaseWatcher.ts",
    "sha256": "35151505ccc6d0ebb071cebc937829a133fe430eee1ebfdeff343cf641a215f9"
  },
  {
    "file": "server/game/stateWatchers/BasesHealedThisPhaseWatcher.ts",
    "sha256": "be669c72df846bb5e2ac4e5582abb43e03f7704f73c760ba504447b21ea210f4"
  },
  {
    "file": "server/game/stateWatchers/CardsDefeatedThisPhaseWatcher.ts",
    "sha256": "5f8ea5555a385e25959d0fea6f16fd9db3b0a8aa88530a48d5d7fbfb1a999de2"
  },
  {
    "file": "server/game/stateWatchers/CardsDiscardedThisPhaseWatcher.ts",
    "sha256": "9577e86d5d74f4c953aa20c94b7a4266495c4e6dc6a1cf8fea51d88350cd3769"
  },
  {
    "file": "server/game/stateWatchers/CardsDrawnThisPhaseWatcher.ts",
    "sha256": "c15873da871568ca7d6aa733d18f3f83517cc324e924e400cf1e64d0259e0c25"
  },
  {
    "file": "server/game/stateWatchers/CardsEnteredPlayThisPhaseWatcher.ts",
    "sha256": "0657c46ca78e437e7cad7440164a7c6bd233e7dc24ad665eb38ab72702d66d2c"
  },
  {
    "file": "server/game/stateWatchers/CardsLeftPlayThisPhaseWatcher.ts",
    "sha256": "ac17d2d8f50f99c805a4e9f6b215287fadff0e3a3dda81b8e040e33486c243ed"
  },
  {
    "file": "server/game/stateWatchers/CardsPlayedThisPhaseWatcher.ts",
    "sha256": "3f033140d7e36ba211882acda63eed62e491a96472d06b6be0ae3e1b01ea1c3b"
  },
  {
    "file": "server/game/stateWatchers/DamageDealtThisPhaseWatcher.ts",
    "sha256": "70eb2c4420c33db81e49a86b22586cdc5fde3b18ae199b38cb808eaca2a29fe3"
  },
  {
    "file": "server/game/stateWatchers/ForceUsedThisPhaseWatcher.ts",
    "sha256": "62e9fe6e63d53866404587c0e9bbb871505711247759dd9f0fe689c3e426fdee"
  },
  {
    "file": "server/game/stateWatchers/LeadersDeployedThisPhaseWatcher.ts",
    "sha256": "7f6085f6ac3c7ea655757581485e2e5ef6fe465c991f701c1a5b53b1d4c20f5e"
  },
  {
    "file": "server/game/stateWatchers/StateWatcherLibrary.ts",
    "sha256": "a0bf3675f57dc9c39d702d546931ac1a5bc3492e2d5f7097b93c2c26eca3254e"
  },
  {
    "file": "server/game/stateWatchers/TokensCreatedThisPhaseWatcher.ts",
    "sha256": "c009d22d10f84fcb3b2571bacc2df0f46b3bb716eb35d6c90faa97f8f0d4417d"
  },
  {
    "file": "server/game/stateWatchers/UnitsDamagedThisPhaseWatcher.ts",
    "sha256": "f19c413a48ebe7410f902ed20affd65f2c5b870d47647ada5ac2873240ab0e7f"
  },
  {
    "file": "server/game/stateWatchers/UnitsHealedThisPhaseWatcher.ts",
    "sha256": "c98537b45c9cc834866cbd18b2f0af64c8318c14f0c11979c2f6801f4fcb5609"
  },
  {
    "file": "server/game/TargetInterfaces.ts",
    "sha256": "81fa6d703fe008a23bd5064ee9c00ad56e8e22b0b38bb1a664ff6362d92a6e62"
  },
  {
    "file": "server/gamenode/GameNodeConfig.ts",
    "sha256": "61f0e5a2604177df089591fc1c34e0b119d6eaac358b37a4fd0dda48c6053995"
  },
  {
    "file": "server/gamenode/GameServer.ts",
    "sha256": "b8a14a2d38c6040d9ce8b0f7d3eaaf4334780d508e0eac1dd0bb3781d9a9284f"
  },
  {
    "file": "server/gamenode/index.ts",
    "sha256": "58911ee64fb25b2b95806fbc4b6b3a23ad8f69da86ea3422638192edd17075b9"
  },
  {
    "file": "server/gamenode/Lobby.ts",
    "sha256": "6c206efe06d3b95c1183bf5808f21ca3f6f35dcd8b59fe28d67334eb00eebde4"
  },
  {
    "file": "server/gamenode/MatchmakingRules.ts",
    "sha256": "3a6bef9da6cfb2085eb9976ad8260d21ad8ac444bceeb3ccf3674a1b597d7f55"
  },
  {
    "file": "server/gamenode/QueueHandler.ts",
    "sha256": "1c94173174d811c9a64c4fba45c267aeec546f7ec157181ae4d40725033a203e"
  },
  {
    "file": "server/gameStatistics/GameStatisticsTracker.ts",
    "sha256": "fd00b648f9d75cf7449be94480985279187ac2c103833567d1c470558147d5ca"
  },
  {
    "file": "server/logger.ts",
    "sha256": "17fa2e6be41d52bc5b3974f58676dd9a8e5d09cf51fdf34cfbc2d35391d9266e"
  },
  {
    "file": "server/middleware/AuthMiddleWare.ts",
    "sha256": "f228750d44c27f98257f945057578867d35c25cb5258793eada63292507ae9a0"
  },
  {
    "file": "server/services/DynamoDBInterfaces.ts",
    "sha256": "f9e0328bff304ad5369878c8f5b7e0747a45c2f382c8aa106650a53cd4f92bdc"
  },
  {
    "file": "server/services/DynamoDBInterfaceSchemas.ts",
    "sha256": "a87ff34de0cb72b8c97a95b7237187e627f8879fe59a056272423e5b9d8ea557"
  },
  {
    "file": "server/services/DynamoDBService.ts",
    "sha256": "8cc296ec53c1b5014a46ade77bb8fe2ff6118b63794e1e9d26b55e3b24f1e629"
  },
  {
    "file": "server/Settings.ts",
    "sha256": "7123f3ddcd2499ed92309704524ed6094aec3e2ef7d20901c6702ff8088a2272"
  },
  {
    "file": "server/socket.js",
    "sha256": "9cd745fc6e4e7b7270e09d1826b8643f5f4316b1d37abff73576b9def8b4c89d"
  },
  {
    "file": "server/Spectator.ts",
    "sha256": "227240e676de8f2c293dbd72f3f5cd29ae5df322f8cfe9651cd4f0de2f6a6062"
  },
  {
    "file": "server/Util.ts",
    "sha256": "de7940776a32a811004fda92b26102f1af11ae07919b17c885f37b2afdf93691"
  },
  {
    "file": "server/utils/authUtils.ts",
    "sha256": "debba0cb54a904d25379a291b4714008e2ed7bc0fe0eae55e534c5ec93dcc4c4"
  },
  {
    "file": "server/utils/cardData/CardDataGetter.ts",
    "sha256": "567b63e6d3a919b8b8d95ef8aa187cb8f72e3168db5132553579ab75bb4b93ab"
  },
  {
    "file": "server/utils/cardData/CardDataInterfaces.ts",
    "sha256": "cec1e457194c8219193defc1cf5419e7cf5f4842a500b10ebc3cdd36105a744d"
  },
  {
    "file": "server/utils/cardData/ISynchronousCardDataGetter.ts",
    "sha256": "f9c71dc681a52efb094f1e63022c8a3d33638b936c95d5cd8ca25de5ec13f728"
  },
  {
    "file": "server/utils/cardData/LocalFolderCardDataGetter.ts",
    "sha256": "735f78b8f6dad2fb137fc75fdc438a2ace5d20ed561aa5a54d6c434e314b7ab0"
  },
  {
    "file": "server/utils/cardData/UnitTestCardDataGetter.ts",
    "sha256": "0e5bc7db65283dbd943f45fae556891e0982f0c02c0f48fcc01664c0286e6c90"
  },
  {
    "file": "server/utils/cosmetics/CosmeticsCache.ts",
    "sha256": "6f6c987f0f06b096f5893327cc49c8d9b1f49a2f67b4b61993fd7f332497bdca"
  },
  {
    "file": "server/utils/cosmetics/CosmeticsInterfaces.ts",
    "sha256": "848d2e0927cfe7fb934d385638b7f908b076051abf318716bda3a1abd626cd4e"
  },
  {
    "file": "server/utils/cosmetics/CosmeticsService.ts",
    "sha256": "b73094e97cefd53fc62e67a4c7fdac7d813a3366587ea84014f4bff8e280df9e"
  },
  {
    "file": "server/utils/deck/Deck.ts",
    "sha256": "d51994a4d9ff3e64ae76adfe65c7942e4de7ef31cfa14f781ffbec23f8cdf051"
  },
  {
    "file": "server/utils/deck/DeckFetchError.ts",
    "sha256": "384ebcf78e20ea3799085cec43a8652b7401a3db1a24f8f00732ad30508d0455"
  },
  {
    "file": "server/utils/deck/DeckInterfaces.ts",
    "sha256": "be54b1273156b51ee2d5d9fa8765f35a2d6c20862cef4e55f710f4e4b85122a6"
  },
  {
    "file": "server/utils/deck/DeckLinkResolver.ts",
    "sha256": "65b546a9e21203f01233620d882ccf1f9032fc681f3d5b898652fe7aa7f9eb29"
  },
  {
    "file": "server/utils/deck/DeckService.ts",
    "sha256": "f79ef902209b55ce3b82e20927d9b5a94cac527c144a300012014c72da82a640"
  },
  {
    "file": "server/utils/deck/DeckValidator.ts",
    "sha256": "a98cd80e98b20166f9c2c9a88750703607ff00cfdba7ee9eb6ad75dfa2883b30"
  },
  {
    "file": "server/utils/deck/MeleeDeckFetcher.ts",
    "sha256": "02dd0f07db334ebd47a8a78c2ff506616017a26b1b8e857e168897607729edc0"
  },
  {
    "file": "server/utils/deck/SwuDbDeckFetcher.ts",
    "sha256": "b31a268909c8fff96cbb9320a2905728ba60418bd9f37342f0ae9e4bee95ea06"
  },
  {
    "file": "server/utils/deck/SwuSetData.ts",
    "sha256": "047cf36ffb59a89ec67c4d37ae65e568a73e1a98f2d8970d97155f18a4c02d07"
  },
  {
    "file": "server/utils/GameServerMetrics.ts",
    "sha256": "fbfeafbc6ce584aeb4a029d337f9b56a19211e126e3443c941562976f1094a50"
  },
  {
    "file": "server/utils/IHttpClient.ts",
    "sha256": "a2cf26a0918e3140688c10d263c4754af80c4f719d301016902158ea9ed8ed05"
  },
  {
    "file": "server/utils/IScheduler.ts",
    "sha256": "6455e320736e77c3c21f08ff4153b30e15112b656e95bc7dfaf7bce979f7707a"
  },
  {
    "file": "server/utils/logger.js",
    "sha256": "b679b73d359387a7ee55aa7d12e5ee6f0915ebaed08a9507ee10c42ced0282a4"
  },
  {
    "file": "server/utils/ModActionService.ts",
    "sha256": "f86df355b66b4347f4414b4a8851d764a6528eadde4464c74db1ebbec411b4b6"
  },
  {
    "file": "server/utils/profanityFilter/KarabastProfanityMatcher.ts",
    "sha256": "9b5d3dc0e053cf3f3af804a8337513e9551cf2b2f93be5fa6a742ec3e2d3b5ab"
  },
  {
    "file": "server/utils/profanityFilter/ProfanityFilter.ts",
    "sha256": "6abbcb457a26f6c2373d2fabce1ed66630521beee3dfe7d6f5b9d72bbf88c0f1"
  },
  {
    "file": "server/utils/profanityFilter/termLists/BasicProfanityList.ts",
    "sha256": "14ac06e74b86edd10a5d0732c88da442e7384ef040ef4cf5f8ac1e05da8d90d3"
  },
  {
    "file": "server/utils/profiler.ts",
    "sha256": "3a2533ec4405e4d0506680e800a69ba6bda859ce9f094635628bad4fc11ce135"
  },
  {
    "file": "server/utils/RealHttpClient.ts",
    "sha256": "7d63742efcc98633a3f62fc7ed22dbffb2d231af54362786b3b8d7eae9218659"
  },
  {
    "file": "server/utils/RealScheduler.ts",
    "sha256": "6ab3cdd4f928192e909d61fc7376463bda8c3172d96f149413fde7425159bf01"
  },
  {
    "file": "server/utils/ServerRoleUsersCache.ts",
    "sha256": "4621720a896d431301bbd7228397c1b510a46e29f33bfa72170b877724515f48"
  },
  {
    "file": "server/utils/ServerSettingsCache.ts",
    "sha256": "e98108430fcf8bf9be08b1ab7d6691c5996d4be16ae20c7662e9f216bd0c82c4"
  },
  {
    "file": "server/utils/statHandlers/StatHandlerTypes.ts",
    "sha256": "2c60843b818fdf449a7c11cb1466012bec812b4bd893f4feccba59a7ff86a964"
  },
  {
    "file": "server/utils/statHandlers/SwuBaseHandler.ts",
    "sha256": "32831f38c4f0f358e60b5066445da6579c4a03ad03cda623e3657b963228df91"
  },
  {
    "file": "server/utils/statHandlers/SwuStatsHandler.ts",
    "sha256": "854c0fb015a958f77da13b07572d0c40616744576a64e298600c07355417a2ff"
  },
  {
    "file": "server/utils/stats/statsMessages.ts",
    "sha256": "54f124e3392522c7c84f0fe38718778f5dab69956481a87cb828768514234a1a"
  },
  {
    "file": "server/utils/TimedCache.ts",
    "sha256": "584555aff59ad4e2bd50ac7ffefd72aa005ff9a189f35818e2c386b6e094deee"
  },
  {
    "file": "server/utils/user/User.ts",
    "sha256": "84d6875ee09d01f41b5347e67c8b5053670c40777994ee1efce82295cb9b8100"
  },
  {
    "file": "server/utils/user/UserFactory.ts",
    "sha256": "e561b52c5e5c24ff5a9b9f70ef10fcb1dcb36d41803bdf4eca4d987a6c3596af"
  }
];
const canonical=v=>Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
const semanticHash=v=>crypto.createHash('sha256').update(JSON.stringify(canonical(v))).digest('hex');
function helperEnvelope(catalogue){assert.ok(catalogue&&typeof catalogue==='object'&&!Array.isArray(catalogue)&&Array.isArray(catalogue.cards)&&Array.isArray(catalogue.inputs),'Complete assembled catalogue including inputs required');for(const row of catalogue.inputs){assert.ok(row&&typeof row==='object'&&!Array.isArray(row));assert.deepEqual(Object.keys(row).sort(),['file','sha256']);assert.ok(typeof row.file==='string'&&row.file.length&&/^[a-f0-9]{64}$/.test(row.sha256));}const rows=catalogue.inputs.filter(r=>r.file.startsWith('server/')&&!r.file.startsWith('server/game/cards/'));assert.deepEqual(rows,HELPER_INPUTS,'Complete ordered415 non-card server input rows');return rows;}
function validateCatalogue(catalogue){helperEnvelope(catalogue);for(const[id,p]of Object.entries(PROFILES)){const rows=catalogue.cards.filter(e=>e.setCode===id);assert.equal(rows.length,1,'Exactly one complete entry '+id);assert.equal(semanticHash(rows[0]),p.entryHash,'Whole source/profile/defaults/diagnostics '+id);}return true;}

function classifyGeneratedTokenContinuation(entry,catalogue){try{validateCatalogue(catalogue);return entry?.setCode==='ASH_257'&&semanticHash(entry)===PROFILES.ASH_257.entryHash;}catch{return false;}}
function constructGeneratedTokenContinuationProbes(entry,catalogue){if(entry?.setCode!=='ASH_257')return null;try{validateCatalogue(catalogue);assert.equal(semanticHash(entry),PROFILES.ASH_257.entryHash);return{cardId:'ASH_257',family:'generated-token-continuation',cases:structuredClone(RAW_CASES),uncovered:[],sourceOnly:true,admissionCredit:0};}catch(e){return{cardId:'ASH_257',family:'generated-token-continuation',cases:[],uncovered:[{reason:e.message}],sourceOnly:true,admissionCredit:0};}}

// Formal coverage describes the existing finite recipes, without changing their
// source-derived assertions. Repeated IDs explicitly denote one obligation.
function planGeneratedTokenContinuationProbes(entry,catalogue) {
    const result = constructGeneratedTokenContinuationProbes(entry,catalogue);
    if (!result) return null;
    result.schema = 'swu-card-probes/v1';
    result.sourceHash = entry.sourceHash;
    const path = 'abilities.card.0', prefix = entry.setCode + ':generated-token-continuation:';
    const obligations = new Map();
    for (const recipe of result.cases)
        for (const id of recipe.coverageObligationIds) {
            assert.ok(typeof id === 'string' && id.startsWith(prefix) && id.length > prefix.length);
            if (!obligations.has(id)) obligations.set(id, { id, path, dimension: id.slice(prefix.length) });
        }
    result.uncovered = result.uncovered.map((gap, i) => ({
        ...gap, id: gap.id ?? prefix + 'construction-profile-' + i, path: gap.path ?? path,
    }));
    for (const [dimension, reason] of [["source-interaction-review", "Finite named recipes require independent review of the complete pinned card and helper sources and interactions; they do not establish whole-source conformance."], ["foreign-controller", "No named recipe constructs owner/controller divergence through a control-changing effect."], ["replacement-callback-phase", "No named recipe establishes arbitrary replacement or cancellation between event construction, payment, callback evaluation and resolution."], ["pending-import-capacity", "No named recipe imports a pending continuation or exhausts native token/program storage capacity."], ["generated-token-replacement", "The newborn-death recipe is finite; arbitrary replacement of token creation and the resolvedEvents generatedTokens callback remains unwitnessed."]]) {
        result.uncovered.push({ id: prefix + dimension, path, reason });
    }
    for (const gap of result.uncovered) {
        assert.ok(!obligations.has(gap.id), 'A coverage gap must not claim a witnessed recipe');
        obligations.set(gap.id, { id: gap.id, path: gap.path, dimension: gap.id.slice(prefix.length) });
    }
    result.obligations = [...obligations.values()];
    return result;
}
module.exports={classifyGeneratedTokenContinuation,planGeneratedTokenContinuationProbes,validateCatalogue,helperEnvelope,RAW_CASES,PROFILES,HELPER_INPUTS};
