'use strict';
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const same=require('node:util').isDeepStrictEqual;
const keys=(v,allowed)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).every(k=>allowed.includes(k));
const hash=v=>crypto.createHash('sha256').update(v).digest('hex');
const DEFAULT_DONORS={ground:'HMW_093',ground2:'ASH_061',space:'JTL_069',leader:'SEC_005',attacker:'JTL_069',saboteur:'ASH_141',token:'TOKEN_BEAST',shield:'TOKEN_SHIELD',experience:'TOKEN_EXPERIENCE'};
function metadata(e){const c=e?.card;return !!c&&Number.isInteger(c.cost)&&c.cost>=0&&c.cost<=16&&Number.isInteger(c.hp)&&c.hp>=1&&c.hp<=32&&Number.isInteger(c.power)&&c.power>=0&&c.power<=32&&typeof c.unique==='boolean'&&Array.isArray(c.aspects)&&c.aspects.every(x=>['heroism','villainy','vigilance','command','aggression','cunning'].includes(x))&&Array.isArray(c.traits)&&c.traits.every(x=>typeof x==='string'&&x.length>0)&&!e.piloting&&!e.attachCondition&&!c.pilotText&&!c.pilotBox&&!c.pilotingBox&&c.upgradePower==null&&c.upgradeHp==null;}
function classifyRecipientHeal(e){
 if(!keys(e,['setCode','sourcePath','sourceHash','abilities','className','baseClass','id','internalName','status','card','dataHash','keywordValues','requiredCapabilities','diagnostics','attachCondition','piloting'])||e?.status!=='supported'||e.baseClass!=='NonLeaderUnitCard'||!metadata(e)||!same(e.card.types,['unit'])||e.card.arena!=='space'||e.card.unique!==true||e.card.deployBox||e.card.epicAction||!same(e.card.keywords,['sentinel'])||!same(e.keywordValues,{sentinel:1})||!keys(e.abilities,['leader','unit','card'])||!same(e.abilities.leader,[])||!same(e.abilities.unit,[])||e.abilities.card?.length!==1)return null;
 const a=e.abilities.card[0],f=a.effect,m=f?.amount,cb=m?.callback,param=cb?.parameters?.[0];
 if(!keys(a,['trigger','title','costs','effect'])||a.trigger!=='whenPlayed'||typeof a.title!=='string'||!a.title.trim()||a.title.length>256||!same(a.costs,[])||!keys(f,['kind','target','amount'])||f.kind!=='heal'||!same(f.target,{kind:'arenaList',receiver:{kind:'ref',path:['player']},predicate:{kind:'const',value:true}})||!keys(m,['kind','callback','evaluateAt','legalityComparison'])||m.kind!=='healRecipientAmount'||m.evaluateAt!=='addPropertiesToEachHealEvent'||m.legalityComparison!=='nativePropertyFunctionComparedToZero'||!keys(cb,['kind','parameters','body'])||cb.kind!=='attackCallback'||cb.parameters?.length!==1||!keys(param,['root','role'])||param.role!=='healedCard'||typeof param.root!=='string'||!/^attackBinding\d+_\d+$/.test(param.root)||!same(cb.body,{kind:'ref',path:[param.root,'damage']}))return null;
 return {family:'recipient-heal'};
}
function noAbilities(e){return keys(e.abilities,['leader','unit','card'])&&same(e.abilities,{leader:[],unit:[],card:[]});}
function donorValid(e,role){
 if(!e)return false;
 if(role==='shield'||role==='experience') {const n=role==='shield'?0:1;return e.baseClass==='TokenUpgradeCard'&&same(e.card.types,['token','upgrade'])&&same(e.card.keywords,[])&&same(e.keywordValues,{})&&noAbilities(e)&&e.card.power===n&&e.card.hp===n&&e.card.upgradePower===n&&e.card.upgradeHp===n&&!e.piloting&&!e.attachCondition&&e.card.cost===0&&!e.card.unique&&same(e.card.aspects,[])&&same(e.card.traits,[role==='shield'?'armor':'learned']);}
 if(e.status!=='supported'||!metadata(e))return false;
 if(role==='leader')return e.baseClass==='LeaderUnitCard'&&same(e.card.types,['leader'])&&e.card.arena==='ground'&&e.card.unique&&e.card.hp>=2&&same(e.card.keywords,['restore'])&&same(e.keywordValues,{restore:4})&&same(e.abilities.unit,[])&&same(e.abilities.card,[])&&e.card.deployBox==='Restore 4'&&same(e.abilities.leader,[{trigger:'action',title:'Heal up to 2 damage from a unit. If you do, deal that much damage to your base',costs:[{kind:'exhaustSelf'}],effect:{kind:'distributeHealing',amount:{kind:'const',value:2},maxTargets:1,canChooseNoTargets:true,canDistributeLess:true,targetResolver:{owner:'any',type:'unit',zone:'anyAttackable'}},ifYouDo:{title:'Deal that much damage to your base',costs:[],effect:{kind:'damage',target:'ownBase',amount:{kind:'ref',path:['event','totalDistributed']}}}}]);
 if(!noAbilities(e)||e.card.deployBox||e.card.epicAction||e.card.unique)return false;
 if(role==='token')return e.baseClass==='TokenUnitCard'&&same(e.card.types,['token','unit'])&&e.card.arena==='ground'&&e.card.hp>=2&&same(e.card.keywords,[])&&same(e.keywordValues,{})&&same(e.card.aspects,[])&&same(e.card.traits,['creature']);
 if(!same(e.card.types,['unit'])||!(e.baseClass==='NonLeaderUnitCard'||(!e.baseClass&&!e.sourcePath)))return false;
 if(role==='saboteur')return e.card.arena==='space'&&e.card.power>=1&&same(e.card.keywords,['saboteur'])&&same(e.keywordValues,{saboteur:1});
 return e.card.arena===(role==='ground'||role==='ground2'?'ground':'space')&&e.card.hp>=5&&e.card.power>=1&&!e.card.text&&same(e.card.keywords,[])&&same(e.keywordValues,{});
}
function planRecipientHealProbes(entry,options={}){
 const shape=classifyRecipientHeal(entry);if(!shape)return null;
 const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
 function obligation(d,reason){const id=`${entry.setCode}:recipient-heal:${d}`;result.obligations.push({id,path:'recipient-heal',dimension:d});if(reason)result.uncovered.push({id,path:'recipient-heal',reason});return id;}
 const defs=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
 // Independent runtime admission is necessary: compiler status alone is insufficient.
 const reportPath=path.resolve(__dirname,'../../compiler/generated/runtime-admission.json');
 const bytes=fs.readFileSync(reportPath),report=JSON.parse(bytes),admitted=new Map(report.runtimeAdmittedCards.map(e=>[e.setCode,e]));
 const donorCodes={...DEFAULT_DONORS,...options.recipientHealDonors};
 const donorEntries=Object.fromEntries(Object.entries(donorCodes).map(([r,c])=>[r,defs.find(e=>e.setCode===c)]));
 const bad=Object.keys(DEFAULT_DONORS).filter(r=>!donorValid(donorEntries[r],r)||!admitted.has(donorCodes[r])||admitted.get(donorCodes[r]).internalName!==donorEntries[r]?.internalName);
 result.donorAdmissionReceipt={schema:'recipient-heal-donor-admission/v1',reportPath:'compiler/generated/runtime-admission.json',reportHash:hash(bytes),catalogueHash:report.catalogueHash,donors:Object.fromEntries(Object.entries(donorEntries).map(([r,e])=>[r,{code:donorCodes[r],sourceHash:e?.sourceHash??null,dataHash:e?.dataHash??null,entryHash:e?hash(JSON.stringify(e)):null,implementation:admitted.get(donorCodes[r])?.implementation??null}]))};
 if(bad.length){obligation('donor-admission','Missing, structurally incompatible, or independently unadmitted setup donors: '+bad.join(','));return result;}
 for(const owner of [0,1])for(const scenario of ['mixed','all-undamaged','source-only','keep-old','keep-new','sentinel','saboteur','attachments']){
 const id=obligation(`owner-${owner}-${scenario}`);result.cases.push({id:`auto/${entry.setCode}/recipient-heal-owner-${owner}-${scenario}`,name:`recipient-heal-owner-${owner}-${scenario}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set(Object.values(donorCodes))].sort(),behaviorIds:['recipient-heal','mandatory-paid-when-played','friendly-arena-scope','intrinsic-sentinel'],assertionStages:['construction','normal-paid-play','unique-choice','recipient-damage','enemy-and-base-unchanged','sentinel-attack-domain','observation'],coverageObligationIds:[id],probe:{...shape,cardId:entry.setCode,owner,scenario,donorCodes}});
 }
 for(const [d,reason]of [
 ['source-and-helper-review','Independent full source, helper defaults, callback capture, event conditions and metadata review remains required'],
 ['heal-listeners-and-replacements','No admitted onDamageHealed listener/replacement interaction is constructed; no listener support credit'],
 ['foreign-control','Foreign owner/controller divergence is outside supported exporter state'],
 ['capture-before-mutation','Distinct recipient final damage does not distinguish captured event amounts from live reevaluation; requires engine diagnostic and native test'],
 ['one-state-sweep','Plain wounded units do not distinguish one post-primary state sweep from per-recipient sweeps; no actual printed lethal aura witness'],
 ['large-capacity','No normal legal greater-than-32 recipient production witness; staged forty-token engine diagnostic and native capacity tests are separate evidence'],
 ['deployed-leader-and-token-production','Mixed case stages already deployed leader and token population; it witnesses healing scope, not their normal creation or deployment'],
 ['event-order-and-success-result','Final-board counts do not prove emitted event order or zero-result success; engine event diagnostics required'],
 ])obligation(d,reason);
 return result;
}
module.exports={classifyRecipientHeal,donorValid,planRecipientHealProbes};
