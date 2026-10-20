'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const same = require('node:util').isDeepStrictEqual;
const { canonical } = require('../inventory');
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const semanticSha256 = entry => sha256(JSON.stringify(canonical(entry)));
const keys = (value,allowed) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).every(k=>allowed.includes(k));
function classifyAmbushContext(entry) {
    const c=entry?.card;
    if (!keys(entry,['setCode','sourcePath','sourceHash','abilities','className','baseClass','id','internalName','status','card','dataHash','keywordValues','requiredCapabilities','diagnostics','attachCondition','piloting']) ||
        !keys(c,['title','subtitle','cost','hp','power','text','deployBox','epicAction','unique','rules','upgradePower','upgradeHp','id','aspects','traits','arena','keywords','types','setId','setCodes','internalName','pilotText','pilotBox','pilotingBox']) ||
        entry.status!=='supported' || entry.baseClass!=='NonLeaderUnitCard' || !c || !same(c.types,['unit']) || c.arena!=='space' || c.unique!==false || c.cost!==5 || c.hp!==6 || c.power!==3 ||
        c.text!=='Ambush (When you play this unit, it may attack an enemy unit.)\nWhile attacking using Ambush, this unit gets +2/+0.' || !same(c.keywords,['ambush']) || !same(entry.keywordValues,{ambush:1}) ||
        !Array.isArray(c.aspects) || !c.aspects.every(a=>['heroism','villainy','vigilance','command','aggression','cunning'].includes(a)) || !Array.isArray(c.traits) || !c.traits.every(t=>typeof t==='string'&&t.length) ||
        c.pilotText || c.pilotBox || c.pilotingBox || c.deployBox || c.epicAction || c.upgradeHp!=null || c.upgradePower!=null || entry.piloting || entry.attachCondition ||
        typeof entry.sourcePath!=='string' || !entry.sourcePath || !/^[a-f0-9]{64}$/.test(entry.sourceHash??'') || !/^[a-f0-9]{64}$/.test(entry.dataHash??'') ||
        !keys(entry.abilities,['leader','unit','card']) || !same(entry.abilities.leader,[]) || !same(entry.abilities.unit,[]) || entry.abilities.card?.length!==1) return null;
    const a=entry.abilities.card[0];
    if (!same(a,{
        trigger:'constant',title:'While attacking with Ambush, this unit gets +2/+0',costs:[],
        effect:{kind:'modifyStats',target:'self',duration:'persistent',power:{kind:'const',value:2},hp:{kind:'const',value:0}},
        condition:{kind:'query',name:'isAttackingUsingAmbush',receiver:{kind:'ref',path:['source']},args:[]},
    })) return null;
    return {family:'ambush-context',trigger:'constant',bonusPower:2};
}
const DONORS={
    frigate:{code:'JTL_069',entrySemanticSha256:'6611c0d90ba427510603108630d7417569bce9356c7bcda444dcde3ee1cf1d8f'},
    sentinel:{code:'ASH_117',entrySemanticSha256:'cbe68b1051e407cab4733445d2b90a69fb270f87eca3017cd6d9b8573b6c7383'},
    fleet:{code:'LAW_164',entrySemanticSha256:'33b92b9e7c6676f6e1eba9fe4248088a0334ba7772cbb12ac2f30cb946f9c935'},
    legacy:{code:'LOF_213',entrySemanticSha256:'e69eb09f0bc9f7896eabdce8b6c6b031cc5b26b5b13d080572516c0d5c4d7ecd'},
};
function planAmbushContextProbes(entry,options={}) {
    const shape=classifyAmbushContext(entry);if(!shape)return null;
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(dimension,reason){const id=`${entry.setCode}:ambush-context:${dimension}`;result.obligations.push({id,path:'ambush-context',dimension});if(reason)result.uncovered.push({id,path:'ambush-context',reason});return id;}
    const defs=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const reportBytes=fs.readFileSync(path.resolve(__dirname,'../../compiler/generated/runtime-admission.json')),report=JSON.parse(reportBytes),admitted=new Map(report.runtimeAdmittedCards.map(e=>[e.setCode,e]));
    const donorCodes=Object.fromEntries(Object.entries(DONORS).map(([role,p])=>[role,options.ambushContextDonors?.[role]??p.code]));
    const donors=Object.fromEntries(Object.entries(donorCodes).map(([role,code])=>[role,defs.find(e=>e.setCode===code)]));
    const bad=Object.keys(DONORS).filter(role=>!donors[role]||semanticSha256(donors[role])!==DONORS[role].entrySemanticSha256||admitted.get(donorCodes[role])?.internalName!==donors[role]?.internalName);
    result.donorAdmissionReceipt={schema:'ambush-context-donors/v1',reportByteSha256:sha256(reportBytes),catalogueHash:report.catalogueHash,donors:Object.fromEntries(Object.entries(DONORS).map(([role,p])=>[role,{code:donorCodes[role],entrySemanticSha256:donors[role]?semanticSha256(donors[role]):null,expectedEntrySemanticSha256:p.entrySemanticSha256,sourceHash:donors[role]?.sourceHash??null,dataHash:donors[role]?.dataHash??null,implementation:admitted.get(donorCodes[role])?.implementation??null}]))};
    if(bad.length){obligation('donor-admission','Missing, mutated or independently unadmitted complete setup donors: '+bad.join(','));return result;}
    const row=(role,owner='opponent')=>({code:donorCodes[role],owner,arena:donors[role].card.arena});
    function add(scenario,sourceSeat,targets,extra={}) {
        const id=obligation(`${scenario}-seat-${sourceSeat}`),probe={...shape,cardId:entry.setCode,sourceArena:entry.card.arena,sourceSeat,scenario,resourceCount:30,targets,defenderIndex:0,laterNormal:true,...extra};
        result.cases.push({id:`auto/${entry.setCode}/ambush-context-${scenario}-seat-${sourceSeat}`,name:`ambush-context-${scenario}-seat-${sourceSeat}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set(targets.map(t=>t.code))].sort(),
            behaviorIds:['ambush-context','native-ambush','source-physical-attack-context','conditional-persistent-power'],assertionStages:['construction','normal-paid-play','entry-exhausted','ambush-offer-or-no-target-boundary',...(!probe.decline&&scenario!=='no-enemy'?['exact-attack-target-domain','ambush-combat-power']:[]),...(probe.deathDistribution?['pending-death-context','exact-death-distribution']:[]),'attack-cleanup',...(probe.laterNormal?['later-normal-power']:[]),...(probe.cancelRetry?['normal-cancel-retry']:[]),'observation'],coverageObligationIds:[id],probe});
    }
    for(const sourceSeat of [1,2]) {
        add('accept',sourceSeat,[row('frigate')]);
        add('decline',sourceSeat,[row('frigate')],{decline:true});
        add('cancel-retry',sourceSeat,[row('frigate')],{decline:true,cancelRetry:true});
        add('sentinel',sourceSeat,[row('frigate'),row('sentinel')],{defenderIndex:1,sentinelIndex:1});
        add('no-enemy',sourceSeat,[row('frigate','self')],{defenderIndex:null});
        add('attacker-death',sourceSeat,[row('fleet')],{laterNormal:false});
        add('defender-death',sourceSeat,[row('legacy'),row('frigate','self')],{deathDistribution:[{index:'source',amount:2},{index:1,amount:4}]});
        add('defender-death-lethal',sourceSeat,[row('legacy'),row('frigate','self')],{deathDistribution:[{index:'source',amount:6}],laterNormal:false});
    }
    for(const sourceSeat of [1,2]) {
        add('mirror-defender',sourceSeat,[{code:entry.setCode,owner:'opponent',arena:entry.card.arena}],{assertDefenderNoBonus:true});
        result.cases.at(-1).assertionStages.splice(-1,0,'defending-focus-power','physical-role-context');
    }
    for(const [dimension,reason]of[
        ['independent-source-helper-review','Independent full source, helper defaults, complete metadata and native Ambush lifecycle review remains required'],
        ['copied-constant-abilities','No admitted natural route granting Support to this space donor and copying its conditional constant is constructed; arbitrary donor export remains unsupported'],
        ['foreign-ownership','Supported state requires equal physical owner/controller; arbitrary control changes remain globally unsupported'],
        ['cross-arena-attacks','No modifier granting cross-arena attack permission to this source is constructed'],
        ['ambush-base-permission','Ordinary Ambush excludes bases; no printed modifier allowing this source to attack bases using Ambush is constructed'],
        ['terminal-pending-context','Finite natural terminal games do not prove arbitrary direct import of a terminal pending death selector/retained attack frame'],
        ['multiple-nested-death-frames','One actual Legacy death selector is exercised; deeper simultaneous/nested attack frame stacks require separate runtime/source-context tests'],
    ])obligation(dimension,reason);
    return result;
}
module.exports={classifyAmbushContext,planAmbushContextProbes};
