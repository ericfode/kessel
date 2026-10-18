'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={ASH_103:'08_ASH/events/LongLiveTheEmpire.ts',LAW_083:'07_LAW/units/BrokenHornVizagosPride.ts'};
const resource={kind:'resourceCard',target:'ownTopDeck'},ref=(...path)=>({kind:'ref',path}),con=value=>({kind:'const',value});
const pendingResource={kind:'resourceCardSystem',target:{kind:'query',name:'topDeckCard',receiver:ref('player'),args:[]},targetPlayer:'self',readyResource:false,readyTiming:'contingentAfterResourceEvent',readinessTargets:'completeSystemTargetList'};
const fewer=zone=>({kind:'binary',op:'<',left:ref('player',zone,'length'),right:ref('opponent',zone,'length')});
const cond=(condition,onTrue)=>({kind:'conditional',condition,onTrue,onFalse:{kind:'noAction'}});
function source(id){const file='game/cards/'+files[id];return{file,text:fs.readFileSync(path.join(root,file),'utf8')};}
function compile(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function rejected(text,file,pattern='top-deck-resource-source-shape') {const c=compileSource(text,file,enums);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));if(c.status==='supported')return; assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));return c;}
function mutate(id,pairs){const{text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);rejected(text.replace(from,to),file);}}
test('full Imperial defeat event preserves chosen identity, prompt and ifYouDo top-deck movement',()=>{
 const c=compile('ASH_103'),a=c.abilities.card[0];assert.equal(c.baseClass,'EventCard');assert.deepEqual(c.abilities.leader,[]);assert.deepEqual(c.abilities.unit,[]);assert.equal(c.abilities.card.length,1);
 assert.deepEqual(a,{trigger:'event',title:'Defeat a friendly Imperial unit. If you do, resource the top card of your deck',costs:[],targetResolver:{owner:'self',type:'unit',zone:'anyAttackable',predicate:{kind:'query',name:'hasSomeTrait',receiver:ref('card'),args:[con('imperial')]},promptTitle:'Defeat a friendly Imperial unit'},effect:{kind:'defeat',target:'chosen'},ifYouDo:{title:'Resource the top card of your deck',costs:[],effect:resource}});
});
test('full mandatory unit registration preserves live hand then resource comparisons and draw defaults',()=>{
 const c=compile('LAW_083'),a=c.abilities.card[0];assert.equal(c.baseClass,'NonLeaderUnitCard');assert.deepEqual(Object.keys(a),['trigger','title','costs','effect']);assert.equal(a.trigger,'whenPlayed');assert.deepEqual(a.costs,[]);assert.deepEqual(a.effect,{kind:'sequence',effects:[cond(fewer('hand'),{kind:'draw',target:'ownPlayer',amount:con(1)}),cond(fewer('resources'),resource)]});
});
test('event lowering retains changed controller, predicate, continuation and registration semantics',()=>{
 const cases=[
 ['RelativePlayer.Self','RelativePlayer.Opponent',a=>a.targetResolver.owner='opponent'],
 ['WildcardCardType.Unit','WildcardCardType.NonLeaderUnit',a=>a.targetResolver.type='nonLeaderUnit'],
 ['Trait.Imperial),','Trait.Rebel),',a=>a.targetResolver.predicate.args[0].value='rebel'],
 ['immediateEffects.defeat()','immediateEffects.exhaust()',a=>a.effect.kind='exhaust'],
 ['ifYouDo:','then:',a=>{a.then=a.ifYouDo;delete a.ifYouDo;delete a.targetResolver.promptTitle;}],
 ["title: 'Resource the top card of your deck',","title: 'Resource the top card of your deck', optional:true,",a=>a.ifYouDo.optional=true],
 ['extends EventCard','extends NonLeaderUnitCard',()=>{}],
 ['targetResolver: {','optional:true, targetResolver: {',a=>a.optional=true],
 ['targetResolver: {','condition:context=>true, targetResolver: {',a=>a.condition=con(true)],
 ['targetResolver: {','cost:AbilityHelper.costs.exhaustSelf(), targetResolver: {',a=>a.costs=[{kind:'exhaustSelf'}]],
 ['controller: RelativePlayer.Self,','controller: RelativePlayer.Self, zoneFilter:"groundArena",',a=>Object.assign(a.targetResolver,{zone:'arena',arena:'ground'})]
 ];const{text,file}=source('ASH_103');
 for(const[from,to,change]of cases){assert.ok(text.includes(from));const c=compileSource(text.replace(from,to),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));const expected=structuredClone(compile('ASH_103').abilities.card[0]);expected.ifYouDo.effect=structuredClone(pendingResource);change(expected);assert.deepEqual(c.abilities.card,[expected],from);assert.equal(c.baseClass,from==='extends EventCard'?'NonLeaderUnitCard':'EventCard');}
});
test('unit lowering keeps changed comparisons, group ordering, draw properties and headers explicit',()=>{
 const cases=[
 ['hand.length < context.player.opponent.hand.length','hand.length <= context.player.opponent.hand.length',a=>a.effect.effects[0].condition.op='<='],
 ['resources.length < context.player.opponent.resources.length','resources.length <= context.player.opponent.resources.length',a=>a.effect.effects[1].condition.op='<='],
 ['resources.length < context.player.opponent.resources.length','readyResourceCount < context.player.opponent.readyResourceCount',a=>{a.effect.effects[1].condition.left=ref('player','readyResourceCount');a.effect.effects[1].condition.right=ref('opponent','readyResourceCount');}],
 ['immediateEffects.draw()','immediateEffects.draw({amount:2})',a=>a.effect.effects[0].onTrue.amount=con(2)],
 ['immediateEffects.draw()','immediateEffects.draw({target:RelativePlayer.Opponent})',a=>a.effect.effects[0].onTrue.target=con('opponent')],
 ['immediateEffects.sequential','immediateEffects.simultaneous',a=>a.effect.kind='simultaneous'],
 ['registrar.addWhenPlayedAbility({','registrar.addWhenPlayedAbility({optional:true,',a=>a.optional=true],
 ['addWhenPlayedAbility','addWhenDefeatedAbility',a=>a.trigger='whenDefeated'],
 ['extends NonLeaderUnitCard','extends EventCard',()=>{}]
 ];const{text,file}=source('LAW_083');
 for(const[from,to,change]of cases){assert.ok(text.includes(from));const c=compileSource(text.replace(from,to),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));const expected=structuredClone(compile('LAW_083').abilities.card[0]);expected.effect.effects[1].onTrue=structuredClone(pendingResource);change(expected);assert.deepEqual(c.abilities.card,[expected],from);assert.equal(c.baseClass,from==='extends NonLeaderUnitCard'?'EventCard':'NonLeaderUnitCard');}
 const c=compileSource(text.replace('player.hand.length','player.resources.length').replace('opponent.hand.length','opponent.resources.length'),file,enums);assert.equal(c.status,'supported');assert.deepEqual(c.abilities.card[0].effect.effects[0].condition,fewer('resources'));
});
test('same-stage aliases stay compact while nested resource factories retain earlier physical-card captures',()=>{
 for(const id of Object.keys(files)){
 const{text,file}=source(id);const original=id==='ASH_103'?'(context) => ({ target: context.player.getTopCardOfDeck() })':'(context) => ({\n                        target: context.player.getTopCardOfDeck()\n                    })';assert.ok(text.includes(original));
 for(const body of ['const top=context.player.getTopCardOfDeck();return {target:top};','const first={target:context.player.getTopCardOfDeck()};const saved=first;return saved;']){const c=compileSource(text.replace(original,'context=>{'+body+'}'),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.deepEqual(c.abilities,compile(id).abilities);}
 for(const[decl,value]of [['const saved=context.player.getTopCardOfDeck();','{target:saved}'],['const first={target:context.player.getTopCardOfDeck()};const saved=first;','saved']]){
 // A nested resource factory must not requery a captured earlier physical card.
 const effect='AbilityHelper.immediateEffects.conditional(outer=>{'+decl.replaceAll('context','outer')+'return {condition:true,onTrue:AbilityHelper.immediateEffects.resourceCard(context=>('+value+'))};})';
 const prefix='AbilityHelper.immediateEffects.resourceCard('+original+')';assert.ok(text.includes(prefix));if(decl.startsWith('const saved=')){const c=compileSource(text.replace(prefix,effect),file,enums);assert.equal(c.status,'supported');const a=c.abilities.card[0];assertCapturedTopDeck(id==='ASH_103'?a.ifYouDo.effect:a.effect.effects[1].onTrue);}else rejected(text.replace(prefix,effect),file,'system-properties-factory-object');
 }
 }
});
test('new root prompt rejects dynamic text and shadowed TextHelper namespace',()=>{
 const{text,file}=source('ASH_103');rejected(text.replace('activePromptTitle: `Defeat a friendly ${TextHelper.Trait.Imperial} unit`','activePromptTitle: context=>"Dynamic"'),file,'top-deck-resource-prompt');
 rejected(text.replaceAll('registrar','TextHelper'),file,'top-deck-resource-prompt-binding');
 rejected(text.replace('AbilityHelper: IAbilityHelper)','AbilityHelper: IAbilityHelper, TextHelper: any)'),file,'top-deck-resource-prompt-binding');
});

test('continuation roots reject shadowed enum namespaces in callbacks and setup bindings',()=>{
 const{text,file}=source('ASH_103');
 for(const ns of ['Trait','RelativePlayer','WildcardCardType']){rejected(text.replaceAll('registrar',ns),file,'syntax-alias-enum-binding');}
 rejected(text.replace('(card) => card.hasSomeTrait(Trait.Imperial)','(Trait) => Trait.hasSomeTrait(Trait.Imperial)'),file,'syntax-alias-enum-binding');
 rejected(text.replace('AbilityHelper: IAbilityHelper)','AbilityHelper: IAbilityHelper, Trait: any)'),file,'syntax-alias-enum-binding');
});


function assertCapturedTopDeck(factory,aliases=1){
 assert.equal(factory.kind,'systemPropertiesFactory');assert.deepEqual(factory.parameters.map(p=>p.role),['systemPropertiesContext']);assert.equal(factory.invocationPolicy,'eachNativePropertyGeneration');assert.equal(factory.captureIdentity,'valueOrPhysicalObjectReference');assert.equal(factory.constructionOrder,'readonlyDeclarationsThenObjectLeftToRight');assert.equal(factory.captures.length,aliases);
 const first=factory.captures[0];assert.deepEqual(first.value,{kind:'deckQuery',operation:'topCard',receiver:{kind:'ref',path:[factory.parameters[0].root,'player']},args:[],order:'topFirst',empty:'null'});
 for(let i=0;i<aliases;i++){const c=factory.captures[i];assert.equal(c.role,'readonlyDeclaration');assert.equal(c.evaluateAt,'propertyFactoryInvocation');assert.equal(c.order,i);if(i)assert.deepEqual(c.value,{kind:'ref',path:['systemFactoryCaptures',factory.captures[i-1].name]});}
 const conditional=factory.effect;assert.equal(conditional.kind,'conditional');assert.deepEqual(conditional.condition,{kind:'const',value:true});assert.deepEqual(conditional.onFalse,{kind:'noAction'});const inner=conditional.onTrue;assert.equal(inner.kind,'systemPropertiesFactory');assert.equal(inner.invocationPolicy,'eachNativePropertyGeneration');assert.equal(inner.captureIdentity,'valueOrPhysicalObjectReference');assert.deepEqual(inner.captures,[{name:inner.captures[0].name,role:'staticProperty',value:{kind:'ref',path:['systemFactoryCaptures',factory.captures.at(-1).name]},evaluateAt:'propertyFactoryInvocation',order:0}]);assert.deepEqual(inner.effect,{kind:'resourceCard',target:{kind:'ref',path:['systemFactoryCaptures',inner.captures[0].name]}});
}
