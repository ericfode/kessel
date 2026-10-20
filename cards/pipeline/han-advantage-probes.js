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
const effect={kind:'simultaneous',effects:[{kind:'damage',target:'self',amount:{kind:'const',value:3}},{kind:'select',targetResolver:{owner:'any',type:'unit',zone:'anyAttackable',promptTitle:'Give 3 Advantage tokens to a unit'},effect:{kind:'simultaneous',effects:[{kind:'giveAdvantage',target:'chosen',amount:{kind:'const',value:3},createdBy:'self',batch:'singleCreationEvent',tokenOwner:'recipientController'}]}}]};
function classifyHanAdvantage(e) {
    if(!ordinaryUnit(e,['saboteur'],{saboteur:1})||!e.card.unique||e.abilities.card?.length!==1)return null;
    if(typeof e.sourcePath!=='string'||!e.sourcePath||!/^[a-f0-9]{64}$/.test(e.sourceHash??''))return null;
    const a=e.abilities.card[0];
    if(!keys(a,['trigger','title','costs','effect'])||a.trigger!=='whenPlayed'||typeof a.title!=='string'||!a.title.trim()||a.title.length>256||!same(a.costs,[])||!same(a.effect,effect))return null;
    return {family:'han-advantage'};
}
function nativeAdvantage(e) {
    return e?.setCode==='TOKEN_ADVANTAGE'&&e.status==='unsupported'&&e.sourcePath==='game/cards/08_ASH/tokens/Advantage.ts'&&e.sourceHash==='1280861b9a94572ecce2c1d62fb5fd93c11b08609495c3d9f60e656d153e3119'&&
        !e.attachCondition&&!e.piloting&&same(e.abilities,{leader:[],unit:[],card:[]})&&same(e.keywordValues,{})&&
        same(e.card,{title:'Advantage',subtitle:'',cost:0,hp:0,power:1,text:"When attached unit's attack or defense ends: Defeat this upgrade.",deployBox:null,epicAction:'',unique:false,rules:'When Attack Ends/When Defense Ends abilities trigger at the same time as units deal combat damage.\n',upgradePower:1,upgradeHp:0,id:'5844562972',aspects:[],traits:['innate'],keywords:[],types:['token','upgrade'],setId:{set:'ASH'},internalName:'advantage'});
}
function planHanAdvantageProbes(entry,options={}) {
    const shape=classifyHanAdvantage(entry);if(!shape)return null;
    const defs=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(dimension,reason){const id=`${entry.setCode}:han-advantage:${dimension}`;result.obligations.push({id,path:'han-advantage',dimension});if(reason)result.uncovered.push({id,path:'han-advantage',reason});return id;}
    const vanilla=e=>({...e,baseClass:e.baseClass??(!e.sourcePath?'NonLeaderUnitCard':undefined)});
    const quiet=defs.filter(e=>ordinaryUnit(vanilla(e),[],{})&&!e.card.unique&&same(e.abilities.card,[])).sort((a,b)=>a.setCode.localeCompare(b.setCode));
    const ground=quiet.find(e=>e.card.arena==='ground'&&e.card.hp>=4&&e.card.power>=1),space=quiet.find(e=>e.card.arena==='space'&&e.card.hp>=2);
    const sentinel=defs.filter(e=>ordinaryUnit(vanilla(e),['sentinel'],{sentinel:1})&&!e.card.unique&&same(e.abilities.card,[])&&e.card.arena==='ground').sort((a,b)=>a.setCode.localeCompare(b.setCode))[0];
    const advantage=defs.find(nativeAdvantage),shield=defs.find(require('./arena-damage-probes').nativeShield);
    if(!advantage||!shield){obligation('native-token-dependencies','Missing reviewed native Advantage or Shield definition; cannot construct actual token witnesses');return result;}
    if(!ground||!space){obligation('construction-donors','Missing bounded quiet ground/space combat donors');return result;}
    function add(name,fields={}){const id=obligation(name),probe={...shape,cardId:entry.setCode,sourceArena:entry.card.arena,owner:0,scenario:'friendly',groundCode:ground.setCode,spaceCode:space.setCode,...fields};result.cases.push({id:`auto/${entry.setCode}/han-advantage-${name}`,name:`han-advantage-${name}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[...new Set([ground.setCode,space.setCode,advantage.setCode,shield.setCode,...(probe.setupCodes??[])])].sort(),behaviorIds:['han-advantage','native-advantage-three','simultaneous-self-damage','saboteur'],assertionStages:['construction','mandatory-exact-target-menu','damage-before-selection-boundary','single-native-creation-event','three-attachment-event-identities','recipient-controller-ownership','observation'],coverageObligationIds:[id],probe});}
    for(const owner of [0,1])for(const scenario of ['friendly','enemy-ground','enemy-space','self','keep-old','keep-new','append-order'])add(`owner-${owner}-${scenario}`,{owner,scenario});
    if(sentinel)for(const owner of [0,1])add(`owner-${owner}-saboteur-combat`,{owner,scenario:'saboteur-combat',sentinelCode:sentinel.setCode,setupCodes:[sentinel.setCode]});
    else obligation('saboteur-combat','Missing bounded ordinary intrinsic Sentinel donor');
    for(const [dimension,reason] of [
        ['source-review','Finite constructions require complete independent source/helper review'],
        ['foreign-controller','No arbitrary foreign-controlled source or recipient export construction'],
        ['native-token-replacement','No exhaustive native token creation replacement construction'],
        ['shield-self-on-entry','Actual Cobb Vanth entry-Shield witness requires unsupported companion eventHooks; engine diagnostic only, no differential coverage'] ,
        ['leader-recipient','No complete bounded deployed-leader construction donor supplied'],
        ['simultaneous-lethal-self','Fresh ordinary source survives damage3; synthetic preloaded damage is diagnostic only'],
    ])obligation(dimension,reason);
    return result;
}
module.exports={classifyHanAdvantage,planHanAdvantageProbes,ordinaryUnit,nativeAdvantage};
