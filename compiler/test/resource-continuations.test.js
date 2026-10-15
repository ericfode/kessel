'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={ASH_103:'08_ASH/events/LongLiveTheEmpire.ts',LAW_083:'07_LAW/units/BrokenHornVizagosPride.ts'};
const resource={kind:'resourceCard',target:'ownTopDeck'},ref=(...path)=>({kind:'ref',path}),con=value=>({kind:'const',value});
const fewer=zone=>({kind:'binary',op:'<',left:ref('player',zone,'length'),right:ref('opponent',zone,'length')});
const cond=(condition,onTrue)=>({kind:'conditional',condition,onTrue,onFalse:{kind:'noAction'}});
function source(id){const file='game/cards/'+files[id];return{file,text:fs.readFileSync(path.join(root,file),'utf8')};}
function compile(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function rejected(text,file,pattern='top-deck-resource-source-shape'){const c=compileSource(text,file,enums);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));return c;}
function mutate(id,pairs){const{text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);rejected(text.replace(from,to),file);}}
test('full Imperial defeat event preserves chosen identity, prompt and ifYouDo top-deck movement',()=>{
 const c=compile('ASH_103'),a=c.abilities.card[0];assert.equal(c.baseClass,'EventCard');assert.deepEqual(c.abilities.leader,[]);assert.deepEqual(c.abilities.unit,[]);assert.equal(c.abilities.card.length,1);
 assert.deepEqual(a,{trigger:'event',title:'Defeat a friendly Imperial unit. If you do, resource the top card of your deck',costs:[],targetResolver:{owner:'self',type:'unit',zone:'anyAttackable',predicate:{kind:'query',name:'hasSomeTrait',receiver:ref('card'),args:[con('imperial')]},promptTitle:'Defeat a friendly Imperial unit'},effect:{kind:'defeat',target:'chosen'},ifYouDo:{title:'Resource the top card of your deck',costs:[],effect:resource}});
});
test('full mandatory unit registration preserves live hand then resource comparisons and draw defaults',()=>{
 const c=compile('LAW_083'),a=c.abilities.card[0];assert.equal(c.baseClass,'NonLeaderUnitCard');assert.deepEqual(Object.keys(a),['trigger','title','costs','effect']);assert.equal(a.trigger,'whenPlayed');assert.deepEqual(a.costs,[]);assert.deepEqual(a.effect,{kind:'sequence',effects:[cond(fewer('hand'),{kind:'draw',target:'ownPlayer',amount:con(1)}),cond(fewer('resources'),resource)]});
});
test('event gate rejects changed controller, predicate, defeat, continuation and headers',()=>{
 mutate('ASH_103',[
 ['RelativePlayer.Self','RelativePlayer.Opponent'],['WildcardCardType.Unit','WildcardCardType.NonLeaderUnit'],['Trait.Imperial),','Trait.Rebel),'],['immediateEffects.defeat()','immediateEffects.exhaust()'],['ifYouDo:','then:'],["title: 'Resource the top card of your deck',","title: 'Resource the top card of your deck', optional:true,"],['extends EventCard','extends NonLeaderUnitCard'],['targetResolver: {','optional:true, targetResolver: {'],['targetResolver: {','condition:context=>true, targetResolver: {'],['targetResolver: {','cost:AbilityHelper.costs.exhaustSelf(), targetResolver: {'],['controller: RelativePlayer.Self,','controller: RelativePlayer.Self, zoneFilter:"groundArena",']
 ]);
});
test('unit gate rejects changed comparisons, sequence order, effects and optionality',()=>{
 mutate('LAW_083',[
 ['hand.length < context.player.opponent.hand.length','hand.length <= context.player.opponent.hand.length'],['resources.length < context.player.opponent.resources.length','resources.length <= context.player.opponent.resources.length'],['resources.length < context.player.opponent.resources.length','readyResourceCount < context.player.opponent.readyResourceCount'],['immediateEffects.draw()','immediateEffects.draw({amount:2})'],['immediateEffects.draw()','immediateEffects.draw({target:RelativePlayer.Opponent})'],['immediateEffects.sequential','immediateEffects.simultaneous'],['registrar.addWhenPlayedAbility({','registrar.addWhenPlayedAbility({optional:true,'],['addWhenPlayedAbility','addWhenDefeatedAbility'],['extends NonLeaderUnitCard','extends EventCard']
 ]);
 const{text,file}=source('LAW_083');rejected(text.replace('player.hand.length','player.resources.length').replace('opponent.hand.length','opponent.resources.length'),file);
});
test('new contexts preserve same-stage aliases and reject earlier property/query snapshots',()=>{
 for(const id of Object.keys(files)){
 const{text,file}=source(id);const original=id==='ASH_103'?'(context) => ({ target: context.player.getTopCardOfDeck() })':'(context) => ({\n                        target: context.player.getTopCardOfDeck()\n                    })';assert.ok(text.includes(original));
 for(const body of ['const top=context.player.getTopCardOfDeck();return {target:top};','const first={target:context.player.getTopCardOfDeck()};const saved=first;return saved;']){const c=compileSource(text.replace(original,'context=>{'+body+'}'),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.deepEqual(c.abilities,compile(id).abilities);}
 for(const[decl,value]of [['const saved=context.player.getTopCardOfDeck();','{target:saved}'],['const first={target:context.player.getTopCardOfDeck()};const saved=first;','saved']]){
 // A nested resource factory must not requery a captured earlier physical card.
 const effect='AbilityHelper.immediateEffects.conditional(outer=>{'+decl.replaceAll('context','outer')+'return {condition:true,onTrue:AbilityHelper.immediateEffects.resourceCard(context=>('+value+'))};})';
 const prefix='AbilityHelper.immediateEffects.resourceCard('+original+')';assert.ok(text.includes(prefix));rejected(text.replace(prefix,effect),file,'arena-query-snapshot-capture');
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
 for(const ns of ['Trait','RelativePlayer','WildcardCardType']){rejected(text.replaceAll('registrar',ns),file,'top-deck-resource-enum-binding');}
 rejected(text.replace('(card) => card.hasSomeTrait(Trait.Imperial)','(Trait) => Trait.hasSomeTrait(Trait.Imperial)'),file,'top-deck-resource-enum-binding');
 rejected(text.replace('AbilityHelper: IAbilityHelper)','AbilityHelper: IAbilityHelper, Trait: any)'),file,'top-deck-resource-enum-binding');
});
