'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={JTL_119:'04_JTL/units/ResupplyCarrier.ts',JTL_164:'04_JTL/units/ChamSyndullaRallyingRyloth.ts',HMW_136:'09_HMW/units/LifetreeCaravan.ts'};
function source(id){const file='game/cards/'+files[id];return{text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function full(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(text,file,pattern){const c=compileSource(text,file,enums);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));}
function mutations(id,pairs){const{text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);reject(text.replace(from,to),file);}}
const resource={kind:'resourceCard',target:'ownTopDeck'};
test('three full source registrations preserve optionality, live conditions and ability source',()=>{
 for(const id of Object.keys(files)){const c=full(id),a=c.abilities.card[0];assert.equal(c.baseClass,'NonLeaderUnitCard');assert.equal(c.abilities.card.length,1);assert.deepEqual(c.abilities.leader,[]);assert.deepEqual(c.abilities.unit,[]);assert.equal(a.trigger,'whenPlayed');assert.equal(a.optional,true);assert.deepEqual(a.costs,[]);assert.equal(a.targetResolver,undefined);assert.equal(a.then,undefined);assert.equal(a.ifYouDo,undefined);}
 assert.deepEqual(full('JTL_119').abilities.card[0].effect,resource);
 assert.deepEqual(full('JTL_164').abilities.card[0].effect,{kind:'conditional',condition:{kind:'binary',op:'<',left:{kind:'ref',path:['player','resources','length']},right:{kind:'ref',path:['opponent','resources','length']}},onTrue:resource,onFalse:{kind:'noAction'}});
 assert.deepEqual(full('HMW_136').abilities.card[0].effect,{kind:'conditional',condition:{kind:'binary',op:'>=',left:{kind:'query',name:'arenaUnitCount',receiver:{kind:'ref',path:['player']},args:[{kind:'const',value:true}]},right:{kind:'const',value:3}},onTrue:resource,onFalse:{kind:'noAction'}});
});
function carrierEffect(replacement){const{text,file}=source('JTL_119');const original='AbilityHelper.immediateEffects.resourceCard((context) => ({\n                target: context.player.getTopCardOfDeck()\n            }))';assert.ok(text.includes(original));return{text:text.replace(original,replacement),file};}
test('same-stage query and property-object aliases preserve physical-target lookup role',()=>{
 for(const body of [
 'const top=context.player.getTopCardOfDeck();return {target:top};',
 'const first=context.player.getTopCardOfDeck();const second=first;const top=second;return {target:top};',
 'const saved={target:context.player.getTopCardOfDeck()};return saved;',
 'const first={target:context.player.getTopCardOfDeck()};const saved=first;return saved;'
 ]){const{text,file}=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>{'+body+'})'),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.deepEqual(c.abilities,full('JTL_119').abilities);}
 const alpha=source('JTL_119');assert.deepEqual(compileSource(alpha.text.replaceAll('context','actual'),alpha.file,enums).abilities,full('JTL_119').abilities);
 const normal=carrierEffect('AbilityHelper.immediateEffects.resourceCard(function(context){return {target:context.player.getTopCardOfDeck()};})');assert.deepEqual(compileSource(normal.text,normal.file,enums).abilities,full('JTL_119').abilities);
});
test('query and property-object snapshots reject direct and transitive cross-callback captures',()=>{
 for(const[declarations,target]of [
 ['const saved=context.player.getTopCardOfDeck();','{target:saved}'],
 ['const first=context.player.getTopCardOfDeck();const saved=first;','{target:saved}'],
 ['const saved={target:context.player.getTopCardOfDeck()};','saved'],
 ['const first={target:context.player.getTopCardOfDeck()};const second=first;const saved=second;','saved']
 ]){const{text,file}=carrierEffect('AbilityHelper.immediateEffects.conditional(context=>{'+declarations+'return {condition:true,onTrue:AbilityHelper.immediateEffects.resourceCard(later=>('+target+'))};})');reject(text,file,'arena-query-snapshot-capture');}
});
test('own deck lookup rejects wrong receiver, arguments, optional calls and callback binders',()=>{
 mutations('JTL_119',[
 ['context.player.getTopCardOfDeck()','context.player.opponent.getTopCardOfDeck()'],['context.player.getTopCardOfDeck()','context.source.getTopCardOfDeck()'],['getTopCardOfDeck()','getTopCardOfDeck(1)'],['getTopCardOfDeck()','getTopCardOfDeck?.()'],['context.player.getTopCardOfDeck()','context?.player.getTopCardOfDeck()'],['context.player.getTopCardOfDeck()','context.player?.getTopCardOfDeck()'],['(context) =>','async (context) =>'],['(context) =>','(...context) =>'],['(context) =>','(context=sideEffect()) =>'],['(context) =>','({player:context}) =>']
 ]);
 for(const body of ['const saved=context;return {target:saved.player.getTopCardOfDeck()};','const first=context;const saved=first;return {target:saved.player.getTopCardOfDeck()};','const owner=context.player;return {target:owner.getTopCardOfDeck()};','const first=context.player;const owner=first;return {target:owner.getTopCardOfDeck()};']){const{text,file}=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>{'+body+'})');reject(text,file,'top-deck-resource-context');}
 const callable=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>{const lookup=()=>context.player.getTopCardOfDeck();return {target:lookup};})');reject(callable.text,callable.file);
});
test('top deck lowering is resource-only and whole source registration remains bounded',()=>{
 mutations('JTL_119',[
 ['immediateEffects.resourceCard','immediateEffects.returnToHand'],['immediateEffects.resourceCard','immediateEffects.defeat'],['target: context.player.getTopCardOfDeck()','target: context.player.getTopCardOfDeck(), readyResource:true'],['target: context.player.getTopCardOfDeck()','target: context.player.getTopCardOfDeck(), targetPlayer:RelativePlayer.Opponent'],['optional: true','optional: false'],['optional: true,',''],['addWhenPlayedAbility','addWhenDefeatedAbility'],['extends NonLeaderUnitCard','extends EventCard'],['extends NonLeaderUnitCard','extends TokenUnitCard'],['optional: true,','optional:true, condition:context=>true,'],['optional: true,','optional:true, cost:AbilityHelper.costs.exhaustSelf(),'],['optional: true,','optional:true, then:{immediateEffect:AbilityHelper.immediateEffects.draw()},']
 ]);
 mutations('JTL_164',[
 ['resources.length < context.player.opponent.resources.length','resources.length <= context.player.opponent.resources.length'],['resources.length < context.player.opponent.resources.length','readyResourceCount < context.player.opponent.readyResourceCount']
 ]);
 mutations('HMW_136',[
 ['getArenaUnits().length >= 3','getArenaUnits().length > 3'],['getArenaUnits().length >= 3','getArenaUnits().length >= 2'],['c.player.getArenaUnits()','c.player.opponent.getArenaUnits()'],['getArenaUnits()','getArenaUnits({condition:card=>card.isUnit()})']
 ]);
});
test('deferred selector, group and continuation siblings remain full-source rejected',()=>{
 for(const file of ['game/cards/07_LAW/events/Stockpile.ts','game/cards/02_SHD/units/FrontierTrader.ts','game/cards/09_HMW/units/ImaGunDiHoldingOutForResupply.ts','game/cards/08_ASH/events/LongLiveTheEmpire.ts','game/cards/07_LAW/units/BrokenHornVizagosPride.ts']){const text=fs.readFileSync(path.join(root,file),'utf8');reject(text,file,'top-deck-resource-source-shape');assert.deepEqual(compileSource(text,file,enums).abilities,{leader:[],unit:[],card:[]});}
 const{text,file}=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>({target:context.player.getTopCardOfDeck()}))');const extra=text.replace('registrar.addWhenPlayedAbility({','registrar.addWhenPlayedAbility({title:"Extra",immediateEffect:AbilityHelper.immediateEffects.draw()});registrar.addWhenPlayedAbility({');reject(extra,file,'top-deck-resource-source-shape');
});
