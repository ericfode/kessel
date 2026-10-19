'use strict';
const same = require('node:util').isDeepStrictEqual;
const keys = (value, allowed) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).every(key => allowed.includes(key));
// This is the complete retained helper contract, not a match on one descendant leaf.
const selectedResourceEffect = {
    kind: 'selectionSystem', system: 'selectPlayer', name: 'target',
    resolver: {kind:'playerSelection',mode:'player',chooser:'self',resultBinding:'target',defaults:'pinnedTargetResolver',contextBinding:'target',choosingPlayerOverride:'contextOverrideFirst',delayOpponentBeforeCosts:true,resolverName:'target'},
    innerSystem: {kind:'exhaustResources',target:{kind:'ref',path:['selectedPlayer']},amount:{kind:'const',value:1},paymentType:'exhaustResources',isCost:false,allowCostAdjustments:false},
    contract:'SelectPlayerSystem.selectedPlayerDefaultTarget',legality:'innerSystemCanAffectPlayer',
};
function classifySelectionExhaust(entry) {
    const card = entry?.card;
    if (entry?.status !== 'supported' || entry.baseClass !== 'NonLeaderUnitCard' || !same(card?.types,['unit']) || !['ground','space'].includes(card.arena) ||
        !Number.isInteger(card.cost) || card.cost < 0 || card.cost > 8 || !Number.isInteger(card.power) || card.power < 0 || card.power > 32 || !Number.isInteger(card.hp) || card.hp < 1 || card.hp > 32 ||
        typeof card.unique !== 'boolean' || entry.attachCondition || entry.piloting || card.pilotText || card.pilotBox || card.pilotingBox || card.deployBox || card.epicAction ||
        card.upgradePower != null || card.upgradeHp != null || !same(card.keywords,[]) || !same(entry.keywordValues ?? {},{})) return null;
    if (!keys(entry.abilities,['leader','unit','card']) || !same(entry.abilities.leader,[]) || !same(entry.abilities.unit,[]) || entry.abilities.card?.length !== 1) return null;
    const ability = entry.abilities.card[0];
    if (!keys(ability,['trigger','title','costs','effect']) || ability.trigger !== 'action' || typeof ability.title !== 'string' || !ability.title.trim() || ability.title.length > 256 ||
        !same(ability.costs,[{kind:'exhaustSelf'}]) || !same(ability.effect,selectedResourceEffect)) return null;
    return {family:'selection-exhaust'};
}
function resourceKeywordDonor(entry) {
    const c=entry?.card;
    if(entry?.status!=='supported' || entry.baseClass!=='NonLeaderUnitCard' || !same(c?.types,['unit']) || c.arena!=='ground' ||
        !Number.isInteger(c.hp) || c.hp<1 || c.hp>32 || !Number.isInteger(c.power) || c.power<0 || c.power>32 ||
        !Number.isInteger(c.cost) || c.cost<0 || c.cost>8 || typeof c.unique!=='boolean' || entry.piloting || entry.attachCondition || c.pilotText || c.pilotBox || c.pilotingBox || c.deployBox || c.epicAction ||
        c.upgradeHp!=null || c.upgradePower!=null || !same(c.keywords,['raid','overwhelm']) || !same(entry.keywordValues??{},{}) ||
        !keys(entry.abilities,['card','leader','unit']) || !same(entry.abilities.leader,[]) || !same(entry.abilities.unit,[]) || entry.abilities.card?.length!==2) return false;
    const abilities=entry.abilities.card;
    if(!abilities.every(a=>typeof a.title==='string' && a.title.trim())) return false;
    return same(abilities.map(({title,...a})=>a),[
        {trigger:'constant',costs:[],effect:{kind:'gainKeyword',target:'self',duration:'persistent',keyword:'raid',amount:{kind:'ref',path:['player','exhaustedResourceCount']}}},
        {trigger:'constant',costs:[],effect:{kind:'gainKeyword',target:'self',duration:'persistent',keyword:'overwhelm',amount:{kind:'const',value:1}},condition:{kind:'binary',op:'===',left:{kind:'ref',path:['player','readyResourceCount']},right:{kind:'const',value:0}}},
    ]);
}
function planSelectionExhaustProbes(entry, options = {}) {
    const shape = classifySelectionExhaust(entry);
    if (!shape) return null;
    const definitions=Array.isArray(options.catalogue)?options.catalogue:options.catalogue?.cards??[];
    const aura=definitions.filter(resourceKeywordDonor).sort((a,b)=>a.setCode.localeCompare(b.setCode))[0];
    const result = {schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    const obligation = (dimension,reason) => {
        const id=`${entry.setCode}:selection-exhaust:${dimension}`;
        result.obligations.push({id,path:'selection-exhaust',dimension});
        if (reason) result.uncovered.push({id,path:'selection-exhaust',reason});
        return id;
    };
    function add(name,fields) {
        const id=obligation(name);
        const probe={...shape,cardId:entry.setCode,sourceArena:entry.card.arena,owner:0,chosenPlayer:'self',ownReady:2,enemyReady:2,sourceReady:true,scenario:'normal',...fields};
        result.cases.push({id:`auto/${entry.setCode}/selection-exhaust-${name}`,name:`selection-exhaust-${name}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:probe.interactionCode?[probe.interactionCode]:[],behaviorIds:['selection-exhaust','paid-nested-player-selection','physical-resource-order'],assertionStages:['construction','initial-action-legality','paid-source-at-menu','exact-player-menu','selected-resource-identities','observation'],coverageObligationIds:[id],probe});
    }
    for (const owner of [0,1]) {
        for (const chosenPlayer of ['self','opponent']) {
            for (const [ownReady,enemyReady] of [[2,2],[1,2],[2,1],[0,2],[2,0]]) add(`owner-${owner}-${chosenPlayer}-ready-${ownReady}-${enemyReady}`,{owner,chosenPlayer,ownReady,enemyReady});
            add(`owner-${owner}-${chosenPlayer}-reverse-order`,{owner,chosenPlayer,scenario:'reverse-order'});
            add(`owner-${owner}-${chosenPlayer}-both-zero`,{owner,chosenPlayer,ownReady:0,enemyReady:0,scenario:'both-zero'});
        }
        add(`owner-${owner}-prepay-cancel`,{owner,scenario:'cancel'});
        add(`owner-${owner}-source-exhausted`,{owner,sourceReady:false,scenario:'source-exhausted'});
    }
    if(aura) for(const owner of [0,1]) for(const chosenPlayer of ['self','opponent']) add(`owner-${owner}-${chosenPlayer}-live-resource-aura`,{owner,chosenPlayer,ownReady:1,enemyReady:1,scenario:'live-resource-keywords',interactionCode:aura.setCode,interactionSourceHash:aura.sourceHash});
    else obligation('live-resource-sensitive-aura','No reviewed bounded actual resource-sensitive aura construction donor');
    for (const [dimension,reason] of [
        ['source-review','Finite oracle constructions do not establish complete pinned source lowering; independent source review required'],
        ['foreign-controller','Current exporter does not support arbitrary foreign-controlled source states'],
        ['native-credit-not-consumed','Current exporter omits native Credit state; resource-effect payment flags require runtime tests and source review'],
        ['competing-trigger-and-replacement','No exhaustive replacement or competing trigger construction'],
    ]) obligation(dimension,reason);
    return result;
}
module.exports={classifySelectionExhaust,planSelectionExhaustProbes,resourceKeywordDonor};
