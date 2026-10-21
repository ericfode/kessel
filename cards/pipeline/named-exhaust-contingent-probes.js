'use strict';
const crypto=require('node:crypto');
function canonical(v){if(v===null)return 'null';if(Array.isArray(v)){if(Reflect.ownKeys(v).length!==v.length+1||!v.every((_,i)=>Object.hasOwn(v,i)))throw Error('Non-JSON array');return '['+v.map(canonical).join(',')+']';}if(typeof v==='object'){if(![Object.prototype,null].includes(Object.getPrototypeOf(v))||Reflect.ownKeys(v).some(k=>typeof k!=='string'))throw Error('Non-JSON object');return '{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}';}if(typeof v==='number'){if(!Number.isFinite(v))throw Error('Non-JSON number');return Object.is(v,-0)?'-0':String(v);}if(['string','boolean'].includes(typeof v))return JSON.stringify(v);throw Error('Non-JSON value');}
function fingerprint(v){try{return crypto.createHash('sha256').update(canonical(v)).digest('hex');}catch{return null;}}
function profile(entry){if(!entry||typeof entry!=='object'||Array.isArray(entry))return null;const e=structuredClone(entry);for(const k of ['setCode','className','id','internalName'])delete e[k];if(e.card)for(const k of ['id','internalName','setId','setCodes'])delete e.card[k];return e;}
// Closed, complete source/metadata/IR/diagnostic fingerprints, not runtime admission.
const reviewedSource='58109be2162e2efeb3be51fb0ea43c0947a96a85e02f4666558ef25260872d49',reviewedDonors={
  "SOR_095": "27b538f7ef020583c610281275a75f2187beb15ef5946d59a18809d53d686d3d",
  "JTL_069": "6611c0d90ba427510603108630d7417569bce9356c7bcda444dcde3ee1cf1d8f",
  "SOR_014": "d1ca8396901fac3eb4eafafb55439deeadf8b385f6e195faa781a786568c2bac",
  "SOR_023": "a694a128d3fb4bb0a4e465dcf906cb2afd3a936c7d99ee1f8cf53db1a7cb41b6",
  "TOKEN_ADVANTAGE": "be7663af45ebe560c97cbe5b1305863b4dcdf6bd4d594ef333643b3fd1fb7234"
};
const scenarios=["ready-ground","ready-space","mixed-arenas","friendly-exhausted-only","enemy-exhausted-only","both-exhausted-only","select-exhausted-friendly","select-exhausted-enemy","select-exhausted-both","friendly-empty","enemy-empty","both-empty","cancel-friendly","cancel-enemy","attack-expiry","defense-expiry","cancel-no-effect"];

function classifyNamedExhaustContingent(entry){try{return typeof entry?.setCode==='string'&&fingerprint(profile(entry))===reviewedSource?{family:'named-exhaust-contingent'}:null;}catch{return null;}}
function planNamedExhaustContingentProbes(entry,options={}){
 const shape=classifyNamedExhaustContingent(entry);if(!shape)return null;const catalogue=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[],result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
 function obligation(name,reason){const id=entry.setCode+':named-exhaust-contingent:'+name;result.obligations.push({id,path:'named-exhaust-contingent',dimension:name});if(reason)result.uncovered.push({id,path:'named-exhaust-contingent',reason});return id;}
 function donor(code){const rows=catalogue.filter(e=>e.setCode===code);return rows.length===1&&fingerprint(rows[0])===reviewedDonors[code];}
 obligation('source-review','Source-only construction does not prove native admission, parent-event outcome capture or complete event composition.');
 for(const owner of [0,1])for(const scenario of scenarios){const name='owner-'+owner+'-'+scenario,setupCardIds=['SOR_014','SOR_023','TOKEN_ADVANTAGE'];if(!['both-empty','cancel-no-effect'].includes(scenario))setupCardIds.push(['ready-space','defense-expiry'].includes(scenario)?'JTL_069':'SOR_095');if(scenario==='mixed-arenas')setupCardIds.push('JTL_069');setupCardIds.sort();
  if(!setupCardIds.every(donor)){obligation(name,'Required full native token/default/quiet unit profile missing, changed or ambiguous');continue;}const id=obligation(name),probe={...shape,cardId:entry.setCode,owner,scenario,resourceCount:12,sourceArena:'ground',leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'SOR_023',enemyBaseCode:'SOR_023'};
  result.cases.push({id:'auto/'+entry.setCode+'/named-exhaust-contingent-'+name,name:'named-exhaust-contingent-'+name,kind:'probe',focusCardIds:[entry.setCode],setupCardIds,behaviorIds:['named-exhaust-contingent','exhaust','advantage'],assertionStages:['construction','payment','named-selection','parent-outcome','contingent','terminal'],coverageObligationIds:[id],probe});
 }
 for(const name of ['recipient-departure-before-child','control-change-between-bind-and-child','replacement-event-parent-outcome','competing-trigger-during-parent-window'])obligation(name,'No actual legal source construction established; finite gap, no source-only or admission credit.');return result;
}
module.exports={classifyNamedExhaustContingent,planNamedExhaustContingentProbes};
