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
const ref=(...path)=>({kind:'ref',path}),con=value=>({kind:'const',value}),deck={kind:'deckQuery',operation:'topCard',receiver:ref('player'),args:[],order:'topFirst',empty:'null'},pendingResource={kind:'resourceCardSystem',target:{kind:'query',name:'topDeckCard',receiver:ref('player'),args:[]},targetPlayer:'self',readyResource:false,readyTiming:'contingentAfterResourceEvent',readinessTargets:'completeSystemTargetList'};
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
test('query snapshots retain explicit transitive capture slots while unsupported property-object factories reject',()=>{
 for(const[declarations,target]of [
 ['const saved=context.player.getTopCardOfDeck();','{target:saved}'],
 ['const first=context.player.getTopCardOfDeck();const saved=first;','{target:saved}'],
 ['const saved={target:context.player.getTopCardOfDeck()};','saved'],
 ['const first={target:context.player.getTopCardOfDeck()};const second=first;const saved=second;','saved']
 ]){const{text,file}=carrierEffect('AbilityHelper.immediateEffects.conditional(context=>{'+declarations+'return {condition:true,onTrue:AbilityHelper.immediateEffects.resourceCard(later=>('+target+'))};})');if(declarations.includes('={target:'))reject(text,file,'system-properties-factory-object');else{const c=compileSource(text,file,enums);assert.equal(c.status,'supported');assertCapturedTopDeck(c.abilities.card[0].effect,declarations.includes('const first=')?2:1);}}
});
test('own deck lookup preserves readonly callable targets and rejects wrong receivers, arity, optionality and binders',()=>{
 mutations('JTL_119',[
 ['context.player.getTopCardOfDeck()','context.player.opponent.getTopCardOfDeck()'],['context.player.getTopCardOfDeck()','context.source.getTopCardOfDeck()'],['getTopCardOfDeck()','getTopCardOfDeck(1)'],['getTopCardOfDeck()','getTopCardOfDeck?.()'],['context.player.getTopCardOfDeck()','context?.player.getTopCardOfDeck()'],['context.player.getTopCardOfDeck()','context.player?.getTopCardOfDeck()'],['(context) =>','async (context) =>'],['(context) =>','(...context) =>'],['(context) =>','(context=sideEffect()) =>'],['(context) =>','({player:context}) =>']
 ]);
 for(const body of ['const saved=context;return {target:saved.player.getTopCardOfDeck()};','const first=context;const saved=first;return {target:saved.player.getTopCardOfDeck()};','const owner=context.player;return {target:owner.getTopCardOfDeck()};','const first=context.player;const owner=first;return {target:owner.getTopCardOfDeck()};']){const{text,file}=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>{'+body+'})');reject(text,file,'top-deck-resource-context');}
 const callable=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>{const lookup=()=>context.player.getTopCardOfDeck();return {target:lookup};})');const c=compileSource(callable.text,callable.file,enums);assert.equal(c.status,'supported');assert.deepEqual(c.abilities.card[0].effect,{kind:'resourceCard',target:deck});
});
test('top-deck lowering preserves helper, readiness, recipient, registration and live-condition variations',()=>{
 function check(id,from,to,change,base='NonLeaderUnitCard'){
  const{text,file}=source(id);assert.ok(text.includes(from));const c=compileSource(text.replace(from,to),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.equal(c.baseClass,base);const expected=structuredClone(full(id).abilities.card[0]);if(id==='JTL_119')expected.effect=structuredClone(pendingResource);else expected.effect.onTrue=structuredClone(pendingResource);change(expected);assert.deepEqual(c.abilities.card,[expected],from);
 }
 for(const kind of ['returnToHand','defeat'])check('JTL_119','immediateEffects.resourceCard','immediateEffects.'+kind,a=>a.effect={kind,target:deck});
 for(const[from,to,change,base]of [
 ['target: context.player.getTopCardOfDeck()','target: context.player.getTopCardOfDeck(), readyResource:true',a=>a.effect.readyResource=true],
 ['target: context.player.getTopCardOfDeck()','target: context.player.getTopCardOfDeck(), targetPlayer:RelativePlayer.Opponent',a=>a.effect.targetPlayer='opponent'],
 ['optional: true','optional: false',a=>a.optional=false],['optional: true,','',a=>delete a.optional],
 ['addWhenPlayedAbility','addWhenDefeatedAbility',a=>a.trigger='whenDefeated'],
 ['extends NonLeaderUnitCard','extends EventCard',()=>{},'EventCard'],['extends NonLeaderUnitCard','extends TokenUnitCard',()=>{},'TokenUnitCard'],
 ['optional: true,','optional:true, condition:context=>true,',a=>a.condition=con(true)],
 ['optional: true,','optional:true, cost:AbilityHelper.costs.exhaustSelf(),',a=>a.costs=[{kind:'exhaustSelf'}]],
 ['optional: true,','optional:true, then:{immediateEffect:AbilityHelper.immediateEffects.draw()},',a=>a.then={title:'',costs:[],effect:{kind:'draw',target:'ownPlayer',amount:con(1)}}]
 ])check('JTL_119',from,to,change,base);
 check('JTL_164','resources.length < context.player.opponent.resources.length','resources.length <= context.player.opponent.resources.length',a=>a.effect.condition.op='<=');
 check('JTL_164','resources.length < context.player.opponent.resources.length','readyResourceCount < context.player.opponent.readyResourceCount',a=>{a.effect.condition.left=ref('player','readyResourceCount');a.effect.condition.right=ref('opponent','readyResourceCount');});
 check('HMW_136','getArenaUnits().length >= 3','getArenaUnits().length > 3',a=>a.effect.condition.op='>');
 check('HMW_136','getArenaUnits().length >= 3','getArenaUnits().length >= 2',a=>a.effect.condition.right=con(2));
 check('HMW_136','c.player.getArenaUnits()','c.player.opponent.getArenaUnits()',a=>a.effect.condition.left.receiver=ref('opponent'));
 check('HMW_136','getArenaUnits()','getArenaUnits({condition:card=>card.isUnit()})',a=>a.effect.condition.left.args=[{kind:'binary',op:'&&',left:con(true),right:{kind:'query',name:'isUnit',receiver:ref('card'),args:[]}}]);
});
test('resource-zone selection and continuations remain represented without dropping sibling registrations',()=>{
 const lower=file=>{const c=compileSource(fs.readFileSync(path.join(root,file),'utf8'),file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];};
 const frontier=lower('game/cards/02_SHD/units/FrontierTrader.ts');assert.deepEqual(frontier.targetResolver,{owner:'self',type:'any',zone:'resource'});assert.deepEqual(frontier.effect,{kind:'returnToHand',target:'chosen'});assert.equal(frontier.optional,true);assert.equal(frontier.ifYouDo.optional,true);assert.deepEqual(frontier.ifYouDo.effect,pendingResource);
 const ima=lower('game/cards/09_HMW/units/ImaGunDiHoldingOutForResupply.ts');assert.equal(ima.trigger,'whenDefeated');assert.equal(ima.effect.kind,'conditional');assert.deepEqual(ima.effect.onTrue.targetResolver,{owner:'self',type:'any',zone:'hand',optional:true,promptTitle:'Resource a card from your hand'});assert.deepEqual(ima.effect.onTrue.effect,{kind:'resourceCard',target:'chosen'});assert.deepEqual(ima.ifYouDo.effect,pendingResource);
 const{text,file}=carrierEffect('AbilityHelper.immediateEffects.resourceCard(context=>({target:context.player.getTopCardOfDeck()}))');const extra=text.replace('registrar.addWhenPlayedAbility({','registrar.addWhenPlayedAbility({title:"Extra",immediateEffect:AbilityHelper.immediateEffects.draw()});registrar.addWhenPlayedAbility({');const c=compileSource(extra,file,enums);assert.equal(c.status,'supported');assert.equal(c.abilities.card.length,2);assert.deepEqual(c.abilities.card[0],{trigger:'whenPlayed',title:'Extra',costs:[],effect:{kind:'draw',target:'ownPlayer',amount:con(1)}});assert.deepEqual(c.abilities.card[1].effect,pendingResource);
});
function assertCapturedTopDeck(factory,aliases=1){
 assert.equal(factory.kind,'systemPropertiesFactory');assert.deepEqual(factory.parameters.map(p=>p.role),['systemPropertiesContext']);assert.equal(factory.invocationPolicy,'eachNativePropertyGeneration');assert.equal(factory.captureIdentity,'valueOrPhysicalObjectReference');assert.equal(factory.constructionOrder,'readonlyDeclarationsThenObjectLeftToRight');assert.equal(factory.captures.length,aliases);
 const first=factory.captures[0];assert.deepEqual(first.value,{kind:'deckQuery',operation:'topCard',receiver:{kind:'ref',path:[factory.parameters[0].root,'player']},args:[],order:'topFirst',empty:'null'});
 for(let i=0;i<aliases;i++){const c=factory.captures[i];assert.equal(c.role,'readonlyDeclaration');assert.equal(c.evaluateAt,'propertyFactoryInvocation');assert.equal(c.order,i);if(i)assert.deepEqual(c.value,{kind:'ref',path:['systemFactoryCaptures',factory.captures[i-1].name]});}
 const conditional=factory.effect;assert.equal(conditional.kind,'conditional');assert.deepEqual(conditional.condition,{kind:'const',value:true});assert.deepEqual(conditional.onFalse,{kind:'noAction'});const inner=conditional.onTrue;assert.equal(inner.kind,'systemPropertiesFactory');assert.equal(inner.invocationPolicy,'eachNativePropertyGeneration');assert.equal(inner.captureIdentity,'valueOrPhysicalObjectReference');assert.deepEqual(inner.captures,[{name:inner.captures[0].name,role:'staticProperty',value:{kind:'ref',path:['systemFactoryCaptures',factory.captures.at(-1).name]},evaluateAt:'propertyFactoryInvocation',order:0}]);assert.deepEqual(inner.effect,{kind:'resourceCard',target:{kind:'ref',path:['systemFactoryCaptures',inner.captures[0].name]}});
}
