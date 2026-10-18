'use strict';
const same = require('node:util').isDeepStrictEqual;
const keys = (x, allowed) => x && typeof x === 'object' && !Array.isArray(x) && Object.keys(x).every(k => allowed.includes(k));
function ordinary(e) {
    return e?.status === 'supported' && e.baseClass === 'NonLeaderUnitCard' && same(e.card?.types, ['unit']) &&
        ['ground','space'].includes(e.card.arena) && Number.isInteger(e.card.cost) && e.card.cost >= 0 && e.card.cost <= 8 &&
        Number.isInteger(e.card.hp) && e.card.hp > 0 && Number.isInteger(e.card.power) && e.card.power >= 0 &&
        !e.attachCondition && !e.piloting && !e.card.pilotText && !e.card.pilotBox && !e.card.pilotingBox && !e.card.deployBox && !e.card.epicAction &&
        !e.card.keywords?.length && !Object.keys(e.keywordValues ?? {}).length;
}
function classifyExhaustResources(entry) {
    if (!ordinary(entry) || !keys(entry.abilities, ['card','unit','leader']) || !same(entry.abilities.unit, []) || !same(entry.abilities.leader, [])) return null;
    const abilities = entry.abilities.card;
    if (!Array.isArray(abilities)) return null;
    const resource = abilities.find(a => a.effect?.kind === 'exhaustResources');
    if (!resource || !keys(resource, ['trigger','title','costs','effect','optional']) || typeof resource.title !== 'string' || !same(resource.costs, [])) return null;
    const death = resource.trigger === 'whenDefeated' && resource.optional === undefined;
    const attack = resource.trigger === 'onAttack' && resource.optional === true;
    if (!death && !attack) return null;
    const target = death ? 'ownPlayer' : 'enemyPlayer';
    if (!same(resource.effect, {kind:'exhaustResources', target, amount:{kind:'const',value:1}, paymentType:'exhaustResources',isCost:false,allowCostAdjustments:false})) return null;
    if (death && abilities.length !== 1) return null;
    if (attack) {
        if (abilities.length !== 2) return null;
        const played = abilities.find(a => a !== resource);
        if (!keys(played,['trigger','title','costs','optional','targetResolver','effect']) || played.trigger !== 'whenPlayed' || typeof played.title !== 'string' || played.optional !== true || !same(played.costs,[]) || !same(played.effect,{kind:'exhaust',target:'chosen'}) || !same(played.targetResolver,{owner:'opponent',type:'unit',zone:'anyAttackable'})) return null;
    }
    return {family:'exhaust-resources',trigger:death?'when-defeated':'on-attack',target,optional:attack};
}
function planExhaustResourcesProbes(entry, options = {}) {
    const shape = classifyExhaustResources(entry);
    if (!shape) return null;
    const defs = Array.isArray(options.catalogue) ? options.catalogue : options.catalogue?.cards ?? [];
    const quiet = defs.filter(e => ordinary({...e,baseClass:e.baseClass || (!e.card?.text ? 'NonLeaderUnitCard' : undefined)}) && !e.card.unique && Object.values(e.abilities ?? {}).flat().length === 0).sort((a,b)=>a.setCode.localeCompare(b.setCode));
    const ground = quiet.find(e=>e.card.arena==='ground' && e.card.power>=1);
    const space = quiet.find(e=>e.card.arena==='space');
    const result={schema:'swu-card-probes/v1',cardId:entry.setCode,sourceHash:entry.sourceHash,cases:[],obligations:[],uncovered:[]};
    function obligation(name, reason) { const id=`${entry.setCode}:exhaust-resources:${name}`; result.obligations.push({id,path:'exhaust-resources',dimension:name}); if(reason) result.uncovered.push({id,path:'exhaust-resources',reason}); return id; }
    obligation('source-review','IR construction does not establish completeness of pinned source lowering');
    if(!ground || !space) { obligation('construction-donors','Missing quiet ground/space combat construction donor'); return result; }
    function add(name, fields) {
        const id=obligation(name);
        const probe={...shape,cardId:entry.setCode,sourceArena:entry.card.arena,owner:0,readyResources:2,decline:false,targetArena:null,scenario:'normal',groundCode:ground.setCode,spaceCode:space.setCode,...fields};
        probe.resourceLayout=probe.readyResources===0?[true,true,true,true]:probe.readyResources===1?[true,false,true,true]:[true,false,true,false];
        result.cases.push({id:`auto/${entry.setCode}/exhaust-resources-${name}`,name:`exhaust-resources-${name}`,kind:'probe',focusCardIds:[entry.setCode],setupCardIds:[ground.setCode,space.setCode].sort(),behaviorIds:['exhaust-resources',probe.trigger],assertionStages:['construction','activation','observation'],coverageObligationIds:[id],probe});
    }
    for(const owner of [0,1]) {
        for(const readyResources of [0,1,2]) add(`owner-${owner}-ready-${readyResources}-physical-order`,{owner,readyResources});
        if(shape.optional) {
            add(`owner-${owner}-optional-decline`,{owner,decline:true});
            for(const targetArena of ['ground','space']) add(`owner-${owner}-when-played-${targetArena}`,{owner,trigger:'when-played',targetArena});
            for(const scenario of ['no-units','entering-source','enemy-only']) add(`owner-${owner}-when-played-${scenario}`,{owner,trigger:'when-played',scenario});
            for(const scenario of ['mixed-exhausted','all-exhausted']) add(`owner-${owner}-when-played-${scenario}`,{owner,trigger:'when-played',scenario,targetArena:'ground'});
            add(`owner-${owner}-when-played-decline`,{owner,trigger:'when-played',targetArena:'ground',decline:true});
        }
    }
    for(const dimension of ['controller-changed-last-known-information','competing-trigger-window','native-credit-not-consumed']) obligation(dimension,'No supported actual source-card construction for this interaction');
    return result;
}
module.exports={classifyExhaustResources,planExhaustResourcesProbes};
