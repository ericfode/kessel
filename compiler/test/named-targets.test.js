'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(__dirname,'../../forceteki/server'),enums=loadEnums(root);
function compile(resolvers,extra='') {
    return compileSource(`export default class FreshNamed extends EventCard {
        getImplementationId(){return{id:'fresh-named',internalName:'fresh-named'};}
        setupCardAbilities(registrar,AbilityHelper){registrar.setEventAbility({title:'Named targets',targetResolvers:{${resolvers}},${extra}});}
    }`,'fresh-named.ts',enums);
}
function positive(text,extra){const c=compile(text,extra);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];}
function negative(text,extra){const c=compile(text,extra);if(c.status==='supported')return;assert.equal(c.status,'unsupported',text+extra);}
const damage=(amount=2)=>`immediateEffect:AbilityHelper.immediateEffects.damage({amount:${amount}})`;
test('named resolver order, chooser, labels and backward identity refs remain structural',()=>{
    const a=positive(`enemy:{choosingPlayer:RelativePlayer.Opponent,controller:RelativePlayer.Opponent,cardTypeFilter:WildcardCardType.Unit,activePromptTitle:'Opponent chooses',${damage(3)}},friendly:{controller:RelativePlayer.Self,cardTypeFilter:WildcardCardType.Unit,dependsOn:'enemy',cardCondition:(card,ctx)=>card!==ctx.targets.enemy,${damage(1)}}`);
    assert.deepEqual(a.effect,{kind:'noAction'});
    assert.deepEqual(a.targetResolvers.map(e=>e.name),['enemy','friendly']);
    assert.equal(a.targetResolvers[0].chooser,'opponent');assert.equal(a.targetResolvers[1].chooser,'self');
    assert.equal(a.targetResolvers[0].targetResolver.promptTitle,'Opponent chooses');
    assert.equal(a.targetResolvers[1].dependsOn,'enemy');
    assert.deepEqual(a.targetResolvers[1].targetResolver.predicate.left,{kind:'ref',path:['card']});
    assert.deepEqual(a.targetResolvers[1].targetResolver.predicate.right,{kind:'ref',path:['namedTargets','enemy']});
    assert.equal(a.targetResolvers[0].effect.target,'chosen');
    assert.equal(a.targetResolvers[1].effect.amount.value,1);
});
test('parent with no child system preserves dependency feasibility and explicit target override',()=>{
    const a=positive(`parent:{cardTypeFilter:WildcardCardType.Unit},child:{dependsOn:'parent',cardTypeFilter:WildcardCardType.Unit,cardCondition:(card,ctx)=>card!==ctx.targets.parent,immediateEffect:AbilityHelper.immediateEffects.damage(ctx=>({amount:2,target:ctx.player.opponent.base}))}`);
    assert.equal(Object.hasOwn(a.targetResolvers[0],'effect'),false);
    assert.equal(a.targetResolvers[1].effect.target,'enemyBase');
    const current=positive(`self:{cardTypeFilter:WildcardCardType.Unit,cardCondition:(card,ctx)=>card===ctx.targets.self,${damage()}}`);
    assert.deepEqual(current.targetResolvers[0].targetResolver.predicate.right,{kind:'ref',path:['namedTargets','self']});
});
test('Exactly one retains array selection and lexical property/index reads',()=>{
    const a=positive(`unit:{mode:TargetMode.Exactly,numCards:1,canChooseNoCards:true,cardTypeFilter:WildcardCardType.Unit,${damage()}}`);
    assert.equal(a.targetResolvers[0].targetResolver.mode,'exactly');
    assert.deepEqual(a.targetResolvers[0].targetResolver.count,{kind:'const',value:1});
    assert.equal(a.targetResolvers[0].targetResolver.optional,true);
    for(const [reference,last] of [['ctx.targets.unit.power','power'],['ctx.targets.unit[0]',0],['ctx.targets.unit.length','length']]) {
        const entry=positive(`unit:{mode:TargetMode.Exactly,numCards:1,cardTypeFilter:WildcardCardType.Unit,immediateEffect:AbilityHelper.immediateEffects.damage(ctx=>({amount:${reference}}))}`).targetResolvers[0];
        assert.equal(entry.targetResolver.kind,'cardSelection');assert.equal(entry.targetResolver.mode,'exactly');assert.equal(entry.targetResolver.numCards.value,1);
        const factory=entry.effect;assert.equal(factory.kind,'systemPropertiesFactory');assert.equal(factory.captures.length,1);
        assert.deepEqual(factory.captures[0].value,{kind:'ref',path:[factory.parameters[0].root,'targets','unit',last]});
        assert.equal(factory.captures[0].evaluateAt,'propertyFactoryInvocation');assert.deepEqual(factory.effect.amount,{kind:'ref',path:['systemFactoryCaptures',factory.captures[0].name]});
    }
});
test('lexical named identity survives continuation multiple token leaves without target substitution',()=>{
    const a=positive(`friendlyUnit:{controller:RelativePlayer.Self,cardTypeFilter:WildcardCardType.Unit,immediateEffect:AbilityHelper.immediateEffects.exhaust()}`,`ifYouDo:parent=>({title:'Tokens',immediateEffect:AbilityHelper.immediateEffects.simultaneous([AbilityHelper.immediateEffects.giveExperience({amount:2,target:parent.targets.friendlyUnit}),AbilityHelper.immediateEffects.giveShield({target:parent.targets.friendlyUnit})])})`);
    assert.equal(a.targetResolvers,undefined);
    assert.deepEqual(a.targetResolver,{owner:'self',type:'unit',zone:'anyAttackable'});
    assert.deepEqual(a.effect,{kind:'exhaust',target:'chosen'});
    assert.deepEqual(a.ifYouDo.effect,{kind:'simultaneous',effects:[{kind:'giveExperience',target:'chosen',amount:{kind:'const',value:2}},{kind:'giveShield',target:'chosen',amount:{kind:'const',value:1}}]});
    const named=positive(`friendlyUnit:{cardTypeFilter:WildcardCardType.Unit,immediateEffect:AbilityHelper.immediateEffects.exhaust()}`,`ifYouDo:parent=>({title:'Tokens',immediateEffect:AbilityHelper.immediateEffects.simultaneous([AbilityHelper.immediateEffects.giveExperience({amount:2,target:parent.targets.friendlyUnit}),AbilityHelper.immediateEffects.giveShield({target:parent.targets.friendlyUnit})])})`);
    assert.equal(named.targetResolver,undefined);
    assert.equal(named.targetResolvers[0].targetResolver.owner,'any');
    for(const e of named.ifYouDo.effect.effects)assert.deepEqual(e.target,{kind:'ref',path:['namedTargets','friendlyUnit']});
    negative(`unit:{cardTypeFilter:WildcardCardType.Unit,${damage()}}`,`ifYouDo:ctx=>({immediateEffect:AbilityHelper.immediateEffects.damage({amount:ctx.events[0].totalDistributed})})`);
    const continued=positive(`unit:{cardTypeFilter:WildcardCardType.Unit,${damage()}}`,`ifYouDo:ctx=>({targetResolver:{cardTypeFilter:WildcardCardType.Unit,immediateEffect:AbilityHelper.immediateEffects.damage({amount:1,target:ctx.targets.unit})}})`).ifYouDo;
    assert.equal(continued.kind,'abilityFactory');assert.deepEqual(continued.parameters,[{root:'abilityBinding1',role:'continuationContext'}]);assert.deepEqual(continued.body.effect,{kind:'damage',target:{kind:'ref',path:['abilityBinding1','targets','unit']},amount:{kind:'const',value:1}});
});
test('named lowering distinguishes explicit selector/cost/system contracts from malformed fields and graphs',()=>{
    for(const field of ['unknown:true','onSelectionSetChanged:()=>true','optional:1','canChooseNoCards:1']) negative(`unit:{cardTypeFilter:WildcardCardType.Unit,${field},${damage()}}`);
    negative(`unit:{cardTypeFilter:WildcardCardType.Unit,${damage()}}`,'immediateEffect:AbilityHelper.immediateEffects.draw({amount:1})');
    negative(`unit:{cardTypeFilter:WildcardCardType.Unit,${damage()}}`,'targetResolver:{immediateEffect:AbilityHelper.immediateEffects.draw({amount:1})}');
    const costed=positive(`unit:{cardTypeFilter:WildcardCardType.Unit,${damage()}}`,'cost:AbilityHelper.costs.exhaustSelf()');
    assert.deepEqual(costed.costs,[{kind:'exhaustSelf'}]);assert.deepEqual(costed.namedTargetTiming,{contract:'PlayerOrCardAbility.namedTargetResolversWithCosts',selfPreTarget:true,opponentAfterPayment:true,sharedLimitIdentity:true});
    for(const [field,expected] of [['mustChangeGameState:true',{mustChangeGameState:true}],['condition:ctx=>true',{condition:{kind:'const',value:true}}],['choosingPlayer:ctx=>ctx.player',{chooser:{kind:'ref',path:['player']}}],['controller:ctx=>ctx.player',{controller:{kind:'ref',path:['player']}}],['mode:TargetMode.UpTo,numCards:1',{mode:'upTo',numCards:{kind:'const',value:1}}],['mode:TargetMode.Exactly,numCards:2',{mode:'exactly',numCards:{kind:'const',value:2}}],['numCards:1',{numCards:{kind:'const',value:1}}],['zoneFilter:ZoneName.Hand',{zones:{kind:'const',value:'hand'}}]]) {
        const resolver=positive(`unit:{cardTypeFilter:WildcardCardType.Unit,${field},${damage()}}`).targetResolvers[0].targetResolver;
        assert.equal(resolver.kind,'cardSelection');assert.equal(resolver.resultBinding,'unit');assert.equal(resolver.contextBinding,'target');assert.equal(resolver.delayOpponentBeforeCosts,true);
        assert.deepEqual(Object.fromEntries(Object.keys(expected).map(key=>[key,resolver[key]])),expected,field);
    }
    negative(`a:{dependsOn:'b',${damage()}},b:{${damage()}}`);
    negative(`a:{dependsOn:'a',${damage()}}`);
    negative(`a:{${damage()}},b:{dependsOn:'a',${damage()}},c:{dependsOn:'a',${damage()}}`);
    negative(`a:{cardCondition:(card,ctx)=>card!==ctx.targets.b,${damage()}},b:{${damage()}}`);
    const backward=positive(`a:{${damage()}},b:{cardCondition:(card,ctx)=>card!==ctx.targets.a,${damage()}}`).targetResolvers;
    assert.deepEqual(backward.map(x=>x.name),['a','b']);assert.deepEqual(backward[1].targetResolver.predicate,{kind:'binary',op:'!==',left:{kind:'ref',path:['selectionCandidate']},right:{kind:'ref',path:['namedTargets','a']}});
    const five=positive(`a:{${damage()}},b:{${damage()}},c:{${damage()}},d:{${damage()}},e:{${damage()}}`).targetResolvers;
    assert.deepEqual(five.map(x=>x.name),['a','b','c','d','e']);for(const entry of five){assert.equal(entry.targetResolver.resultBinding,entry.name);assert.deepEqual(entry.effect,{kind:'damage',target:'chosen',amount:{kind:'const',value:2}});}
    negative(`'01':{${damage()}}`);negative(`constructor:{${damage()}}`);
    negative(`a:{${damage()},immediateEffect:AbilityHelper.immediateEffects.damage({amount:1})}`);
    negative(`a:{immediateEffect:AbilityHelper.immediateEffects.damage({amount:1,source:ctx.source})}`);
    for(const [effect,expected] of [['payResources({amount:1})',{kind:'registrationSystem',system:'payResources',contextTarget:'chosen',defaults:'pinnedSystem',properties:{amount:{kind:'const',value:1}}}],['playCard()',{kind:'playCardSystem',helper:'playCard',target:'chosen',ignoredRequirements:['phase'],optional:false,entersReady:false,playType:'playFromHand',playAsType:'any',nested:false,canPlayFromAnyZone:false}],['giveAdvantage()',{kind:'registrationSystem',system:'giveAdvantage',contextTarget:'chosen',defaults:'pinnedSystem',properties:{}}]])assert.deepEqual(positive(`a:{immediateEffect:AbilityHelper.immediateEffects.${effect}}`).targetResolvers[0].effect,expected);
    negative(`a:{cardCondition:(card,ctx)=>card!==ctx.targets.unknown,${damage()}}`);
    const factory=positive(`a:{immediateEffect:AbilityHelper.immediateEffects.damage(ctx=>({amount:ctx.target.power}))}`).targetResolvers[0].effect;
    assert.equal(factory.kind,'systemPropertiesFactory');assert.deepEqual(factory.captures[0].value,{kind:'ref',path:[factory.parameters[0].root,'target','power']});assert.equal(factory.captures[0].evaluateAt,'propertyFactoryInvocation');assert.deepEqual(factory.effect.amount,{kind:'ref',path:['systemFactoryCaptures',factory.captures[0].name]});
});
test('all seventeen complete pinned card sources convert without dropping named effects',()=>{
const files=["game/cards/09_HMW/units/C-3POCaptivatingStoryteller.ts", "game/cards/09_HMW/units/WreckerWreckingTheEmpire.ts", "game/cards/04_JTL/events/CoordinatedFront.ts", "game/cards/07_LAW/units/BosskJoinOurMerryBand.ts", "game/cards/07_LAW/units/MillenniumFalconDodgingPatrols.ts", "game/cards/07_LAW/units/GovernorsShuttle.ts", "game/cards/07_LAW/units/AerieCloudRiderDropship.ts", "game/cards/05_LOF/events/CalmInTheStorm.ts", "game/cards/01_SOR/units/DeathTrooper.ts", "game/cards/01_SOR/events/AsteroidSanctuary.ts", "game/cards/07_TS26/units/ZitonMojSowingChaos.ts", "game/cards/07_TS26/events/Fervor.ts", "game/cards/07_TS26/events/Mislead.ts", "game/cards/03_TWI/events/UnlimitedPower.ts", "game/cards/03_TWI/events/GrenadeStrike.ts", "game/cards/03_TWI/events/MercilessContest.ts", "game/cards/03_TWI/events/HeroesOnBothSides.ts"];
let namedCount=0,scalarCount=0;
for(const file of files){
 const c=compileSource(fs.readFileSync(path.join(root,file),'utf8'),file,enums);assert.equal(c.status,'supported',file+JSON.stringify(c.diagnostics));
 if(file.endsWith('/CalmInTheStorm.ts')){const a=c.abilities.card[0];assert.equal(a.targetResolvers,undefined);assert.deepEqual(a.targetResolver,{owner:'self',type:'unit',zone:'anyAttackable'});assert.deepEqual(a.effect,{kind:'exhaust',target:'chosen'});assert.deepEqual(a.ifYouDo.effect,{kind:'simultaneous',effects:[{kind:'giveExperience',target:'chosen',amount:{kind:'const',value:2}},{kind:'giveShield',target:'chosen',amount:{kind:'const',value:1}}]});scalarCount++;}
 else {const a=[...c.abilities.card,...c.abilities.unit,...c.abilities.leader].find(a=>a.targetResolvers);assert.ok(a&&a.targetResolvers.length);assert.deepEqual(a.effect,{kind:'noAction'});assert.ok(a.targetResolvers.every(e=>e.effect));namedCount++;}
}
assert.equal(namedCount,16);assert.equal(scalarCount,1);
});
