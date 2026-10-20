'use strict';
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const same=require('node:util').isDeepStrictEqual;
const hash=v=>crypto.createHash('sha256').update(v).digest('hex');
const keys=(v,allowed)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).every(k=>allowed.includes(k));
const ref=(...path)=>({kind:'ref',path}),constant=value=>({kind:'const',value}),binary=(op,left,right)=>({kind:'binary',op,left,right}),not=value=>({kind:'unary',op:'!',value});
// Preserve the entire getArenaUnits({otherThan:source,arena:space}) predicate,
// including its nullable-source arm. A same-trigger leaf match is insufficient.
const target={kind:'arenaList',receiver:ref('game'),predicate:binary('&&',binary('&&',constant(true),binary('||',not(constant('spaceArena')),binary('||',binary('===',constant('spaceArena'),constant('anyArena')),binary('===',ref('card','zoneName'),constant('spaceArena'))))),binary('||',not(ref('source')),binary('!==',ref('card'),ref('source'))))};
function classifyArenaDefeatAttack(e){
 if(!keys(e,['setCode','sourcePath','sourceHash','abilities','className','baseClass','id','internalName','status','card','dataHash','keywordValues','requiredCapabilities','diagnostics','attachCondition','piloting'])||e.status!=='supported'||e.baseClass!=='NonLeaderUnitCard')return null;
 const c=e.card;
 if(!c||!same(c.types,['unit'])||c.arena!=='space'||c.unique!==false||c.cost!==12||c.hp!==15||c.power!==15||!same(c.keywords,['sentinel'])||!same(e.keywordValues,{sentinel:1})||!same(c.aspects,['vigilance'])||!same(c.traits,['creature'])||c.pilotText||c.pilotBox||c.pilotingBox||e.piloting||e.attachCondition||c.deployBox||c.epicAction||c.upgradeHp!=null||c.upgradePower!=null)return null;
 if(!keys(e.abilities,['leader','unit','card'])||!same(e.abilities.leader,[])||!same(e.abilities.unit,[])||e.abilities.card?.length!==1)return null;
 const a=e.abilities.card[0];if(!keys(a,['trigger','title','costs','effect'])||a.trigger!=='onAttack'||typeof a.title!=='string'||!a.title.trim()||a.title.length>256||!same(a.costs,[])||!same(a.effect,{kind:'defeat',target}))return null;
 return {family:'arena-defeat-attack'};
}
const DONORS={ground:'ASH_061',space:'JTL_069',leader:'SEC_005',shield:'TOKEN_SHIELD',experience:'TOKEN_EXPERIENCE',ordinaryUpgrade:'IC27_052',deathDraw:'JTL_063',deathBase:'LAW_189',spaceSaboteur:'ASH_141',defense:'TS26_024'};
// Frozen reviewed setup profiles bind complete metadata, intrinsic keywords,
// source IR and native-adapter definitions; they never dispatch the focus card.
const PROFILES={
  "TS26_024": "390a8e2f2f76feb996d376318cd04253e2333dfa7bb7b4f428b6a6938dcd6020",
  "ASH_061": "f6a033f123707b8cd7f0431599b43617fa2e088092586b0fc67343ca8d482c9b",
  "JTL_069": "6611c0d90ba427510603108630d7417569bce9356c7bcda444dcde3ee1cf1d8f",
  "SEC_005": "ae1bc463696a3fc045eea668b5cf50bf5426a26e1dbf99eec43b6764031cfe91",
  "TOKEN_SHIELD": "a8a34f754731a5dc9de991d5476b9491466190ac7521e0c7192c9210fc5538a9",
  "TOKEN_EXPERIENCE": "275c2c34e4ea190b9c681ea857f3f8bb13877e2373f6309cea717b3cf4eeb9f8",
  "IC27_052": "fec00f15679fe14298b2d95d5ec70c80eee187c77e2cd9d590a4eb62b324fb12",
  "JTL_063": "eae13421284c563d59d3803ae41d337a2552da6516f05ea8b77f072a3e3daa28",
  "LAW_189": "ed4acaa92cb6689e4c4d092b12677b23875a03cb156721ac1c3a2d8e6249cc75",
  "ASH_141": "631176e41954132bf43b060d30f7ef5b8dae362f8b5c71a5496a10ebe63787f2"
};
function canonical(v){return Array.isArray(v)?v.map(canonical):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;}
function donorProfileValid(entry,code){return !!entry&&!!PROFILES[code]&&hash(JSON.stringify(canonical(entry)))===PROFILES[code];}
function planArenaDefeatAttackProbes(entry,options={}){
 const shape=classifyArenaDefeatAttack(entry);if(!shape)return null;
 const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
 const obligation=(dimension,reason)=>{const id=`${entry.setCode}:arena-defeat-attack:${dimension}`;result.obligations.push({id,path:'arena-defeat-attack',dimension});if(reason)result.uncovered.push({id,path:'arena-defeat-attack',reason});return id;};
 const cards=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[],defs=new Map(cards.map(e=>[e.setCode,e]));
 const bytes=fs.readFileSync(path.resolve(__dirname,'../../compiler/generated/runtime-admission.json')),report=JSON.parse(bytes),admitted=new Map(report.runtimeAdmittedCards.map(e=>[e.setCode,e]));
 const bad=[];result.donorAdmissionReceipt={schema:'arena-defeat-attack-donor-admission/v1',reportPath:'compiler/generated/runtime-admission.json',reportSha256:hash(bytes),catalogueHash:report.catalogueHash,donors:{}};
 for(const code of Object.values(DONORS)){const e=defs.get(code),valid=donorProfileValid(e,code),a=admitted.get(code),runtimeAdmitted=valid&&a?.internalName===e.internalName;result.donorAdmissionReceipt.donors[code]={entrySha256:PROFILES[code],sourceHash:e?.sourceHash??null,dataHash:e?.dataHash??null,valid,runtimeAdmitted:!!runtimeAdmitted,implementation:a?.implementation??null};if(!runtimeAdmitted)bad.push(code);}
 if(bad.length){obligation('setup-donor-admission','Missing, altered or independently unadmitted full setup profiles: '+bad.join(','));return result;}
 for(const sourceSeat of [1,2])for(const scenario of ['mixed-global-order','source-only','space-defender-removed','attachments','sentinel-domain','saboteur-bypass','death-trigger-order','terminal-base','own-copy-included','enemy-copy-included','on-defense-capture','on-defense-terminal']){
 const id=obligation(`seat-${sourceSeat}-${scenario}`);result.cases.push({id:`auto/${entry.setCode}/arena-defeat-attack-seat-${sourceSeat}-${scenario}`,name:`arena-defeat-attack-seat-${sourceSeat}-${scenario}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set(Object.values(DONORS))].sort(),behaviorIds:['arena-defeat-attack','global-other-space-units','mandatory-on-attack','intrinsic-sentinel'],assertionStages:['construction','normal-paid-source-play','natural-phase-ready','attack-declaration','defeat-event-window','ground-and-source-survive','attack-target-cleanup','observation'],coverageObligationIds:[id],probe:{...shape,cardId:entry.setCode,sourceSeat,scenario,donors:{...DONORS}}});
 }
 for(const [d,reason]of [
 ['full-source-and-helper-review','Independent full card/helper review must establish exact target evaluation, defeat restrictions, event conditions, attachment contingents, trigger capture, global arena order and terminal continuation'],
 ['phantom-protected-survivor','Pinned Phantom survives eligible enemy ability defeat, but its full source is not runtime admitted; engine-only diagnostic is separate and no native Phantom support is claimed'],
 ['state-based-death-capture','Ordinary recipient defeats and final state do not establish every ongoing-effect death sweep or captured trigger registration; source-bound oracle diagnostics and runtime tests remain distinct'],
 ['foreign-control','Foreign owner/controller divergence is outside current exporter support'],
 ['capacity-and-dynamic-membership','No legal maximum-capacity or arbitrary target-membership mutation witness; finite mixed boards do not prove unbounded ALL behavior'],
 ['other-replacements-and-listeners','No exhaustive external defeat restriction, replacement, nested attack, capture/rescue/reentry or competing listener interaction matrix'],
 ['donor-production','Setup leaders, attachments and preexisting units are staged populations; only focus source normal paid play and actual attacks are claimed'],
 ])obligation(d,reason);
 return result;
}
module.exports={classifyArenaDefeatAttack,planArenaDefeatAttackProbes,donorProfileValid};
