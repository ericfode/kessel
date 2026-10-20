'use strict';
const same=require('node:util').isDeepStrictEqual;
const keys=(x,allowed)=>x&&typeof x==='object'&&!Array.isArray(x)&&Object.keys(x).every(k=>allowed.includes(k));
function ordinaryUnit(e,keywords,keywordValues) {
    const c=e?.card;
    return e?.status==='supported'&&e.baseClass==='NonLeaderUnitCard'&&same(c?.types,['unit'])&&['ground','space'].includes(c.arena)&&
        Number.isInteger(c.cost)&&c.cost>=0&&c.cost<=8&&Number.isInteger(c.power)&&c.power>=0&&c.power<=32&&Number.isInteger(c.hp)&&c.hp>=1&&c.hp<=32&&typeof c.unique==='boolean'&&
        !e.attachCondition&&!e.piloting&&!c.pilotText&&!c.pilotBox&&!c.pilotingBox&&!c.deployBox&&!c.epicAction&&c.upgradeHp==null&&c.upgradePower==null&&same(c.keywords,keywords)&&same(e.keywordValues??{},keywordValues)&&
        keys(e.abilities,['card','unit','leader'])&&same(e.abilities.unit,[])&&same(e.abilities.leader,[]);
}
const effect={kind:'simultaneous',effects:[{kind:'attackPermission',permission:'canAttackGroundArenaFromSpaceArena',target:'self',duration:'persistent'},{kind:'attackPermission',permission:'canAttackSpaceArenaFromGroundArena',target:'self',duration:'persistent'}]};
function classifyCrossArena(e) {
    if(!ordinaryUnit(e,['support'],{support:1})||!e.card.unique||e.abilities.card?.length!==1)return null;
    if(typeof e.sourcePath!=='string'||!e.sourcePath||!/^[a-f0-9]{64}$/.test(e.sourceHash??''))return null;
    const a=e.abilities.card[0];
    if(!keys(a,['trigger','title','costs','effect'])||a.trigger!=='constant'||typeof a.title!=='string'||!a.title.trim()||a.title.length>256||!same(a.costs,[])||!same(a.effect,effect))return null;
    return {family:'cross-arena'};
}
function wicketDonor(e) {
    if(!ordinaryUnit(e,['saboteur'],{saboteur:1})||e.card.arena!=='ground'||!e.card.unique||e.abilities.card?.length!==1)return false;
    const a=e.abilities.card[0];
    return keys(a,['trigger','title','costs','effect'])&&a.trigger==='constant'&&typeof a.title==='string'&&!!a.title.trim()&&same(a.costs,[])&&same(a.effect,{kind:'constantEffectRegistration',properties:{},modifier:{kind:'ongoingContract',scope:'card',effectType:'cannotAttackBase',target:'constantMatch',duration:'persistent',valueMode:'static',value:{kind:'undefined'}},duration:'persistent',context:'refreshFromSourceController',scopeDefaults:'pinnedOngoingCardOrPlayerEffect',dependencyPolicy:'liveFixedPointRequired'});
}
function planCrossArenaProbes(entry,options={}) {
    const shape=classifyCrossArena(entry);if(!shape)return null;
    const defs=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(dimension,reason){const id=`${entry.setCode}:cross-arena:${dimension}`;result.obligations.push({id,path:'cross-arena',dimension});if(reason)result.uncovered.push({id,path:'cross-arena',reason});return id;}
    const vanilla=e=>({...e,baseClass:e.baseClass??(!e.sourcePath?'NonLeaderUnitCard':undefined)});
    const candidates=defs.filter(e=>same(e.abilities?.card,[])&&!e.card?.unique).sort((a,b)=>a.setCode.localeCompare(b.setCode));
    const find=(arena,keyword)=>candidates.find(e=>e.card.arena===arena&&ordinaryUnit(vanilla(e),keyword?[keyword]:[],keyword?{[keyword]:1}:{}));
    const ground=find('ground'),space=find('space'),groundSentinel=find('ground','sentinel'),spaceSentinel=find('space','sentinel'),saboteur=find('ground','saboteur');
    if(!ground||!space||!groundSentinel||!spaceSentinel||!saboteur){obligation('construction-donors','Missing complete quiet, Sentinel or Saboteur construction donor');return result;}
    const wicket=defs.filter(wicketDonor).sort((a,b)=>a.setCode.localeCompare(b.setCode))[0];
    function add(name,fields={}){const id=obligation(name),probe={...shape,cardId:entry.setCode,sourceArena:entry.card.arena,owner:0,mode:'printed',scenario:'normal',recipientArena:'ground',groundCode:ground.setCode,spaceCode:space.setCode,groundSentinelCode:groundSentinel.setCode,spaceSentinelCode:spaceSentinel.setCode,saboteurCode:saboteur.setCode,...fields};result.cases.push({id:`auto/${entry.setCode}/cross-arena-${name}`,name:`cross-arena-${name}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set([ground.setCode,space.setCode,groundSentinel.setCode,spaceSentinel.setCode,saboteur.setCode,...(probe.setupCodes??[])])].sort(),behaviorIds:['cross-arena','support-copied-constant','ordinary-attack-legality'],assertionStages:probe.terminalFatigue?['paid-source-play','defender-cancel-retains-copied-permission','forty-stable-post-cancel-action-windows','legal-pass-only-through-natural-regroups','round-twenty-empty-decks-fatigue-draw','retained-snapshot-terminal-export','observation']:probe.cancelDefender?['paid-source-play','defender-cancel-retains-copied-permission',...(probe.multipleCancellations?['second-paid-source-play','old-donor-uniqueness-discard','two-copied-snapshot-identities']:[]),'action-window-export','later-actual-cross-arena-attack','post-attack-permission-cleanup','observation']:probe.expectedRecipientSelectable===false?['construction','paid-source-play','pre-grant-recipient-exclusion','no-permission-grant','observation']:['construction','pre-grant-target-legality','exact-defender-menu','actual-combat','support-snapshot-lifecycle','post-attack-permission-cleanup','observation'],coverageObligationIds:[id],probe});}
    for(const owner of [0,1]) {
        for(const scenario of ['normal','ground-sentinel','space-sentinel','both-sentinels'])add(`owner-${owner}-printed-${scenario}`,{owner,scenario});
        for(const recipientArena of ['ground','space'])for(const scenario of ['normal','cross-unit-only','ground-sentinel','space-sentinel','both-sentinels','decline'])add(`owner-${owner}-support-${recipientArena}-${scenario}`,{owner,mode:'support',recipientArena,scenario});
        if(wicket)add(`owner-${owner}-support-ground-pre-grant-exclusion`,{owner,mode:'support',recipientArena:'ground',scenario:'wicket-only-cross-defender',wicketCode:wicket.setCode,expectedRecipientSelectable:false,setupCodes:[wicket.setCode]});
        for(const recipientArena of ['ground','space']) {
            add(`owner-${owner}-support-${recipientArena}-defender-cancel`,{owner,mode:'support',recipientArena,scenario:'defender-cancel',cancelDefender:true});
            add(`owner-${owner}-support-${recipientArena}-multiple-cancel-same-recipient`,{owner,mode:'support',recipientArena,scenario:'multiple-cancel-same-recipient',cancelDefender:true,multipleCancellations:true,resourceCount:30});
        }
        add(`owner-${owner}-support-saboteur`,{owner,mode:'support',scenario:'saboteur',recipientArena:'ground'});
    }
    // Keep the prior case objects and order stable; this is a new terminal
    // boundary reached by ordinary play and natural deck exhaustion.
    for(const owner of [0,1])add(`owner-${owner}-support-cancel-terminal-fatigue`,{owner,mode:'support',recipientArena:'ground',scenario:'support-cancel-terminal-fatigue',cancelDefender:true,terminalFatigue:true,resourceCount:12});
    if(!wicket)obligation('pre-grant-exclusion','Missing complete reviewed intrinsic ground cannot-attack-base donor');
    for(const [dimension,reason] of [
        ['source-review','Complete printed and copied Support registration requires independent pinned-source review'],
        ['donor-return-reentry','Paid second-source uniqueness witnesses discarded donor retaining copied snapshot; donor return/reentry remains unwitnessed'],
        ['recipient-departure','No actual companion construction exercises recipient departure/reentry during Support; full-card lifecycle validation pending'],
        ['uniqueness-keep-old','Paid second-source uniqueness witnesses keep-new and two retained snapshots; keep-old incoming-source defeat remains unwitnessed'],
        ['foreign-controller','No arbitrary foreign-controlled recipient construction'],
        ['pending-support-import','Pending Support target resolver serialization is outside current action-window import API; no pending resolver restoration coverage claimed'],
        ['surviving-recipient-next-attack','Actual later normal attack checked only for surviving recipients without Sentinel obstruction; not every combat branch'],
        ['space-pre-grant-exclusion','Space Entrenched rejection remains engine-only diagnostic because attachment donor is not runtime admitted; excluded from standard plans'],
    ])obligation(dimension,reason);
    return result;
}
module.exports={classifyCrossArena,planCrossArenaProbes,wicketDonor};
