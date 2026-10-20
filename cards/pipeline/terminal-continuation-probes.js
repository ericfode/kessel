'use strict';
const fs=require('node:fs'),path=require('node:path'),{createHash}=require('node:crypto');
const {canonical}=require('../inventory');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
// Complete entry semantic identities guard source, metadata and every registration.
// They deliberately do not replace byte-exact catalogue/recording identities.
const PROFILES={
  "SHD_130": "a2588b11a0c6abafe2d073b960225b2f0b5d4fdb885b195594de9732633817f4",
  "ASH_179": "a527b0322cd14a30f067ae7f6209a7a6d3810d524594aa4bc2e38b442a56f89c",
  "JTL_104": "5322ade2796494afec9884f09200040bd856cfca48b75b8758800e92ee63e9e8",
  "LAW_033": "2b1b6927cd6d9340d8ccf0511d43c2651d9d478054def809443116b5c3c5366c",
  "ASH_260": "1020eb5a439ab58cdb13201224f50dc5e55e178cde31dec25d5c992375eeb34f",
  "JTL_063": "eae13421284c563d59d3803ae41d337a2552da6516f05ea8b77f072a3e3daa28",
  "ASH_151": "f624d88b12117c24756f2412e788ec4896bc7080537dd60d2a205393d5d09dd5",
  "LAW_189": "ed4acaa92cb6689e4c4d092b12677b23875a03cb156721ac1c3a2d8e6249cc75",
  "LOF_256": "0f4eee168738d683eb179c31ab575580b130ea4d46a585e4967e54fb5ef1c6a9",
  "SOR_095": "27b538f7ef020583c610281275a75f2187beb15ef5946d59a18809d53d686d3d",
  "TOKEN_EXPERIENCE": "275c2c34e4ea190b9c681ea857f3f8bb13877e2373f6309cea717b3cf4eeb9f8",
  "SOR_014": "d1ca8396901fac3eb4eafafb55439deeadf8b385f6e195faa781a786568c2bac",
  "ASH_019": "76d8ac90ca6b8ec90407f30d525ee3a244dfef8bf0959104ccd6558213238083"
};
const FOCUS=['ASH_179','JTL_104','LAW_033','ASH_260','JTL_063','ASH_151','ASH_019'];
function planTerminalContinuationProbes(entry,options={}) {
 if(!entry||!FOCUS.includes(entry.setCode))return null;
 const cards=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
 const defs=new Map(cards.map(e=>[e.setCode,e]));
 const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[],newAdmissionCredit:0};
 const obligation=(dimension,reason)=>{const id=entry.setCode+':terminal-continuation:'+dimension;result.obligations.push({id,path:'terminal-continuation',dimension});if(reason)result.uncovered.push({id,path:'terminal-continuation',reason});return id;};
 const cinder=['JTL_104','JTL_063','ASH_151'].includes(entry.setCode);
 const required=[entry.setCode,'SOR_014','ASH_019',...(entry.setCode==='ASH_179'||entry.setCode==='JTL_104'?['LOF_256','TOKEN_EXPERIENCE']:[]),...(entry.setCode==='LAW_033'?['SOR_095']:[]),...(cinder?['ASH_151']:[]),...(entry.setCode==='JTL_063'?['LAW_189']:[]),...(entry.setCode==='ASH_151'?['SHD_130','LAW_189','TOKEN_EXPERIENCE']:[]),...(entry.setCode==='ASH_019'?['SOR_095']:[])];
 const reportPath=path.resolve(__dirname,'../../compiler/generated/runtime-admission.json'),bytes=fs.readFileSync(reportPath),report=JSON.parse(bytes),admitted=new Map(report.runtimeAdmittedCards.map(e=>[e.setCode,e]));
 result.sourceAdmissionReceipt={schema:'terminal-continuation-source-admission/v1',reportPath:'compiler/generated/runtime-admission.json',reportSha256:hash(bytes),sources:{}};
 const bad=[];
 for(const code of required){const e=code===entry.setCode?entry:defs.get(code),valid=!!e&&hash(JSON.stringify(canonical(e)))===PROFILES[code],native=admitted.get(code),runtimeAdmitted=valid&&native?.internalName===e.internalName;
  result.sourceAdmissionReceipt.sources[code]={entrySemanticSha256:PROFILES[code],sourceHash:e?.sourceHash??null,dataHash:e?.dataHash??null,valid,runtimeAdmitted:!!runtimeAdmitted};
  if(!valid||(!runtimeAdmitted&&code!=='ASH_151'))bad.push(code);
 }
 if(bad.length){obligation('source-or-admission','Missing, mutated or independently unadmitted complete definitions: '+bad.join(','));return result;}
 if(cinder&&!result.sourceAdmissionReceipt.sources.ASH_151.runtimeAdmitted)obligation('cinder-setup-admission','Plans remain intended coverage only: Operation Cinder must be runtime admitted in the final prepared pool before these cases execute. Current admission receipt does not establish that dependency.');
 result.runtimeReady=required.every(code=>result.sourceAdmissionReceipt.sources[code].runtimeAdmitted);
 function add(scenario,sourceSeat,fields,expectation){const oid=obligation(scenario+'-seat-'+sourceSeat),probe={family:'terminal-continuation',cardId:entry.setCode,sourceSeat,focusOwner:entry.setCode==='JTL_063'?'opponent':'self',focusInPlay:['JTL_104','JTL_063','LAW_033'].includes(entry.setCode),activationCode:cinder?'ASH_151':entry.setCode,scenario,initiative:'self',resourceCount:24,leaderCode:'SOR_014',enemyLeaderCode:'SOR_014',baseCode:'ASH_019',enemyBaseCode:'ASH_019',...fields};
  result.cases.push({id:'auto/'+entry.setCode+'/terminal-continuation-'+scenario+'-seat-'+sourceSeat,name:'terminal-continuation-'+scenario+'-seat-'+sourceSeat,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:required.filter(code=>code!==entry.setCode).sort(),behaviorIds:probe.behaviorOnly?['terminal-continuation','regroup-runtime','latched-winner','postwinner-public-prompts']:['terminal-continuation','latched-winner','postwinner-public-prompts'],assertionStages:probe.behaviorOnly?['source-bound-construction','actual-phase-pass','regroup-draw-batch','latched-winner-resource-card-selection-and-confirm','physical-resource-movement','ready-resources-units-leaders','next-round-with-fixed-winner','terminal-export-boundary']:['source-bound-construction','real-paid-play-or-attack','actual-lethal-event','latched-winner-public-prompt','accepted-postwinner-input','mandatory-resolution','stable-winner-and-completed-board','terminal-export-boundary'],coverageObligationIds:[oid],probe,expected:expectation});
 }
 const experience=()=>Array.from({length:10},()=>({code:'TOKEN_EXPERIENCE',token:'experience'}));
 for(const seat of [1,2])switch(entry.setCode){
 case 'ASH_019':for(const losing of ['p1','p2','both'])add('ordinary-regroup-fatigue-'+losing,seat,{scenario:'ordinary-regroup-fatigue',constructionCode:'SOR_095',activationCode:null,behaviorOnly:true,initiativeSeat:seat,initiative:seat===1?'self':'opponent',fatigueLethalSeats:losing==='both'?['p1','p2']:[losing],initialBaseDamageAbsolute:losing==='both'?[25,25]:losing==='p1'?[25,0]:[0,25],targets:[{code:'SOR_095',owner:'self',arena:'ground',exhausted:true},{code:'SOR_095',owner:'opponent',arena:'ground',exhausted:true}],resourceCount:12,emptyDecks:true,initiallyExhaustResource:true,initiallyExhaustLeaders:true},{winnerAbsolute:losing==='both'?'draw':losing==='p1'?'p2':'p1',baseFinalDamageAbsolute:losing==='both'?[30,30]:losing==='p1'?[30,6]:[6,30],terminalPromptOwner:'both',promptTitle:'Select between 0 and 1 cards to resource',resourceCountDelta:[1,1],handCountDelta:[-1,-1],allResourcesReady:true,allUnitsReady:true,allLeadersReady:true,roundDelta:1,finalActor:null,postWinnerResourceSeats:['p1','p2'],resourceSelectionPerSeat:1,regroupDrawAmounts:[2,2],phaseSequence:['action','regroup','action']});break;
 case 'ASH_151':add('expiry-cavern-regroup',seat,{preparationCode:'SHD_130',targets:[{code:'LAW_189',owner:'self',arena:'space',exhausted:true,attachments:[{code:'TOKEN_EXPERIENCE',token:'experience'}]}],ownDamage:0,enemyDamage:28,emptyDecks:true,initiallyExhaustResource:true,targetIndex:0},{winner:'self',terminalPromptOwner:'both',promptTitle:'Select between 0 and 1 cards to resource',prewinnerDeathPromptTitle:'Deal 2 damage to a base',prewinnerDeathPhase:null,targetFinalZone:'discard',targetDamageBeforeExpiry:5,targetHpBeforeExpiry:6,ownBaseFinalDamage:11,enemyBaseFinalDamage:30,roundDelta:1,allResourcesReady:true,postWinnerResourceSeats:['p1','p2'],regroupDrawAmounts:[2,2],phaseSequence:['action','regroup','action']});break;
 case 'ASH_179':add('rancor-two-hits',seat,{targets:[{code:'LOF_256',owner:'opponent',arena:'ground',attachments:experience()}],ownDamage:25,targetIndex:0},{winner:'opponent',terminalPromptOwner:'self',promptTitle:'Deal 5 damage to an enemy ground unit. Then, deal 5 damage to the same unit.',targetIndices:[0],optionalPass:false,targetDamageEvents:[5,5],targetFinalDamage:10,ownBaseFinalDamage:30});break;
 case 'JTL_104':add('raddus-death',seat,{targets:[{code:'JTL_104',owner:'self',arena:'space',damage:3},{code:'LOF_256',owner:'opponent',arena:'ground',attachments:experience()}],ownDamage:25,focusTargetIndex:0,targetIndex:1},{winner:'opponent',terminalPromptOwner:'self',promptTitle:'Deal 8 damage to an enemy unit',targetIndices:[1],optionalPass:false,focusFinalZone:'discard',targetFinalDamage:13,ownBaseFinalDamage:30});break;
 case 'LAW_033':for(const passOptional of [false,true])add(passOptional?'hounds-pass':'hounds-choose',seat,{sourceInPlay:true,sourceArena:'space',sourceReady:true,targets:[{code:'SOR_095',owner:'opponent',arena:'ground'}],enemyDamage:29,targetIndex:0,passOptional},{winner:'self',terminalPromptOwner:'self',promptTitle:'Defeat a unit with less than 4 power',targetIndices:[0],optionalPass:true,targetFinalZone:passOptional?'groundArena':'discard',enemyBaseFinalDamage:30});break;
 case 'ASH_260':add('watermonger-committed-discard',seat,{targets:[],ownDamage:29,emptyDeck:true,acceptBeforeWinner:true},{winner:'opponent',terminalPromptOwner:'self',promptTitle:"Choose a card to discard for Mos Espa Watermonger's effect",optionalPass:false,mandatoryHandDiscard:1,ownBaseFinalDamage:30});break;
 case 'JTL_063':for(const passOptional of [false,true])add(passOptional?'shuttle-pass':'shuttle-trigger',seat,{targets:[{code:'LAW_189',owner:'self',arena:'space'},{code:'JTL_063',owner:'opponent',arena:'space'}],enemyDamage:28,focusTargetIndex:1,passOptional},{winner:'self',terminalPromptOwner:'opponent',promptTitle:'You may trigger this ability',optionalPass:true,exactButtons:[{arg:'trigger',label:'Trigger'},{arg:'pass',label:'Pass'}],focusFinalZone:'discard',focusOwnerHandDelta:passOptional?0:1,enemyBaseFinalDamage:30,ownBaseFinalDamage:5});break;
 }
 obligation('source-and-helper-review','Full printed source, helper defaults, complete postwinner event ordering and final native/Wasm differential review remain required.');
 obligation('nonterminal-serialization','These terminal traces do not authorize exporting or importing unsupported pending nonterminal resolvers or unit owner/controller divergence.');
 return result;
}
module.exports={planTerminalContinuationProbes};
