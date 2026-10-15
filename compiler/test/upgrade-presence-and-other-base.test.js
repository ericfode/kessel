'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={LOF_244:'05_LOF/units/JediVector.ts',SEC_095:'06_SEC/units/TheedSecurity.ts',TS26_042:'07_TS26/units/ReliefFrigate.ts',JTL_144:'04_JTL/events/NoDisintegrations.ts'};
function source(id){const file='game/cards/'+files[id];return {text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function full(id){const {text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(text,file){const c=compileSource(text,file,enums);assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
const upgrade={kind:'query',name:'isUpgrade',receiver:{kind:'ref',path:['card']},args:[]};

test('upgrade existence preserves independent friendly trait and opposing unfiltered queries',()=>{
 const vector=full('LOF_244');assert.equal(vector.abilities.card.length,2);
 const [jedi,saber]=vector.abilities.card;assert.equal(jedi.condition.name,'hasSomeArenaUnit');
 assert.deepEqual(jedi.condition.args[0].right.right.right,{kind:'ref',path:['source']});
 assert.equal(saber.condition.name,'hasSomeArenaCard');assert.deepEqual(saber.condition.receiver,{kind:'ref',path:['player']});
 assert.deepEqual(saber.condition.args[0].right,upgrade);assert.equal(saber.condition.args[0].left.right.args[0].value,'lightsaber');
 for(const ability of [jedi,saber])assert.deepEqual(ability.effect,{kind:'modifyStats',target:'self',duration:'persistent',power:{kind:'const',value:1},hp:{kind:'const',value:0}});
 const security=full('SEC_095').abilities.card[0];assert.equal(security.trigger,'whenPlayed');
 assert.deepEqual(security.targetResolver,{owner:'any',type:'unit',zone:'anyAttackable'});
 assert.deepEqual(security.effect.condition,{kind:'query',name:'hasSomeArenaCard',receiver:{kind:'ref',path:['opponent']},args:[{kind:'binary',op:'&&',left:{kind:'const',value:true},right:upgrade}]});
 assert.deepEqual(security.effect.onTrue,{kind:'giveExperience',target:'chosen',amount:{kind:'const',value:1}});
});
test('upgrade alias rejects unsupported filters, optional calls, scopes and snapshots',()=>{
 const {text,file}=source('SEC_095');for(const replacement of [
  'hasSomeArenaUpgrade?.()', 'hasSomeArenaUpgrade({}, {})', 'hasSomeArenaUpgrade({type:"unit"})',
  'hasSomeArenaUpgrade({controller:"self"})','hasSomeArenaUpgrade({condition:c=>true})',
  'hasSomeArenaUpgrade({arena:"groundArena"})','hasSomeArenaUpgrade({trait:[]})','hasSomeArenaUpgrade({trait:""})'
 ])reject(text.replace('hasSomeArenaUpgrade()',replacement),file);
 reject(text.replace('context.player.opponent.hasSomeArenaUpgrade()','context.source.hasSomeArenaUpgrade()'),file);
 reject(text.replace('condition: (context) => context.player.opponent.hasSomeArenaUpgrade()', 'condition: (context) => {const saved=context.player.opponent.hasSomeArenaUpgrade();return other=>saved;}'),file);
 const captured=`export default class Probe extends NonLeaderUnitCard {getImplementationId(){return{id:'probe',internalName:'probe'}} setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.conditional(context=>{const saved=context.player.hasSomeArenaUpgrade();const alias=saved;return {condition:later=>alias,onTrue:AbilityHelper.immediateEffects.draw()};})});}}`;
 const snapshot=compileSource(captured,'snapshot.ts',enums);assert.equal(snapshot.status,'unsupported');assert.ok(snapshot.diagnostics.some(d=>d.pattern==='arena-query-snapshot-capture'),JSON.stringify(snapshot.diagnostics));
 const controller=`export default class Probe extends NonLeaderUnitCard {getImplementationId(){return{id:'probe',internalName:'probe'}} setupCardAbilities(registrar,AbilityHelper){const owner=this.controller;registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.conditional({condition:context=>owner.hasSomeArenaUpgrade(),onTrue:AbilityHelper.immediateEffects.draw()})});}}`;
 const receiver=compileSource(controller,'controller.ts',enums);assert.equal(receiver.status,'unsupported');assert.ok(receiver.diagnostics.some(d=>d.pattern==='arena-controller-snapshot-capture'),JSON.stringify(receiver.diagnostics));
});
test('pinned Player helper forces upgrade type and requesting controller across both arenas',()=>{
 const {Player}=require(path.resolve(root,'../build/server/game/core/Player.js'));
 let captured;const receiver={game:{allArenas:{hasSomeCard:filter=>{captured=filter;return true;}}}};
 assert.equal(Player.prototype.hasSomeArenaUpgrade.call(receiver),true);
 assert.equal(captured.controller,receiver);assert.equal(captured.type,'upgrade');
 Player.prototype.hasSomeArenaUpgrade.call(receiver,{trait:'lightsaber',controller:'other',type:'unit'});
 assert.equal(captured.controller,receiver);assert.equal(captured.type,'upgrade');assert.equal(captured.trait,'lightsaber');
});
test('Relief Frigate keeps selected base separate from opposite healing recipient',()=>{
 const ability=full('TS26_042').abilities.card[0];assert.equal(ability.trigger,'whenPlayed');
 assert.deepEqual(ability.targetResolver,{owner:'any',type:'base',zone:'anyAttackable'});
 assert.deepEqual(ability.effect,{kind:'heal',amount:{kind:'const',value:3},target:{kind:'conditional',condition:{kind:'binary',op:'===',left:{kind:'ref',path:['target']},right:{kind:'ref',path:['player','base']}},then:{kind:'ref',path:['opponent','base']},else:{kind:'ref',path:['player','base']}}});
});
test('opposite-base lowering rejects general player lists and missing scalar base selection',()=>{
 const {text,file}=source('TS26_042');for(const changed of [
  text.replace('getPlayers()','getPlayers(1)'),text.replace('getPlayers()','getPlayers?.()'),
  text.replace('.map((x) => x.base)', '.map(async (x) => x.base)'),
  text.replace('.map((x) => x.base)', '.map((...x) => x.base)'),
  text.replace('.map((x) => x.base)', '.map((x = sideEffect()) => x.base)'),
  text.replace('.map((x) => x.base)', '.map((x) => x.hand)'),
  text.replace('x !== context.target','x !== context.source'),text.replace('x !== context.target','x === context.target'),
  text.replace('cardTypeFilter: CardType.Base','cardTypeFilter: WildcardCardType.Unit'),
  text.replace('cardTypeFilter: CardType.Base','cardTypeFilter: CardType.Base, mode:"upTo", numCards:2'),
  text.replace('targetResolver: {\n                cardTypeFilter: CardType.Base,\n                immediateEffect:', 'immediateEffect:').replace(/\n            }\n        }\);/,'\n        });')
 ])reject(changed,file);
});
test('direct damage callback binds the recipient rather than the ability context',()=>{
 const ability=full('JTL_144').abilities.card[0];assert.equal(ability.trigger,'event');
 assert.deepEqual(ability.targetResolver,{owner:'any',type:'nonLeaderUnit',zone:'anyAttackable'});
 assert.deepEqual(ability.effect,{kind:'damage',target:'chosen',amount:{kind:'binary',op:'-',left:{kind:'ref',path:['target','remainingHp']},right:{kind:'const',value:1}}});
});
test('recipient damage callback rejects arbitrary bindings, destinations and captures',()=>{
 const {text,file}=source('JTL_144');for(const changed of [
  text.replace('(target) =>','async (target) =>'),text.replace('(target) =>','(...target) =>'),
  text.replace('(target) =>','(target = sideEffect()) =>'),text.replace('(target) =>','(target,context) =>'),
  text.replace('target.remainingHp - 1','target.remainingHp - 2'),text.replace('target.remainingHp - 1','target.getPower() - 1'),
  text.replace('target.remainingHp - 1','context.source.remainingHp - 1'),
  text.replace('amount: (target)', 'target:context.source, amount: (target)'),
  text.replace('WildcardCardType.NonLeaderUnit','CardType.Base'),
  text.replace('cardTypeFilter: WildcardCardType.NonLeaderUnit','cardTypeFilter: WildcardCardType.NonLeaderUnit, mode:"upTo", numCards:2')
 ])reject(changed,file);
});
test('readonly damage callback aliases preserve recipient role across chains and reject context captures',()=>{
 const {text,file}=source('JTL_144');
 const aliased=(declarations,name='amountCallback')=>text.replace('registrar.setEventAbility',declarations+'\n        registrar.setEventAbility').replace('(target) => target.remainingHp - 1',name);
 const expected=full('JTL_144').abilities.card[0].effect.amount;
 for(const declarations of [
  'const amountCallback = recipient => recipient.remainingHp - 1;',
  'const original = recipient => recipient.remainingHp - 1; const amountCallback = original;',
  'const original = recipient => recipient.remainingHp - 1; const intermediate = original; const amountCallback = intermediate;'
 ]){
  const card=compileSource(aliased(declarations),file,enums);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));assert.deepEqual(card.abilities.card[0].effect.amount,expected);
 }
 for(const declarations of [
  'const amountCallback = context => context.target.remainingHp - 1;',
  'const amountCallback = context => context.source.remainingHp - 1;',
  'const original = context => context.target.remainingHp - 1; const amountCallback = original;',
  'const amountCallback = async recipient => recipient.remainingHp - 1;',
  'const amountCallback = (...recipient) => recipient.remainingHp - 1;',
  'const amountCallback = (recipient=sideEffect()) => recipient.remainingHp - 1;',
  'const first=second; const second=first; const amountCallback=first;'
 ])reject(aliased(declarations),file);
 for(const composite of [
  'true ? (context=>context.target.remainingHp-1) : (recipient=>recipient.remainingHp-1)',
  '(context=>context.source.remainingHp-1) || (recipient=>recipient.remainingHp-1)',
  'null ?? (context=>context.target.remainingHp-1)',
  'Math.max((context=>context.target.remainingHp-1),1)',
  '((context=>context.target.remainingHp-1) as any)',
  '((true ? (context=>context.target.remainingHp-1) : (recipient=>recipient.remainingHp-1)) as any)'
 ]){
  reject(text.replace('(target) => target.remainingHp - 1',composite),file);
  reject(aliased('const value='+composite+';const amountCallback=value;'),file);
 }
 const wrapped=text.replace('(target) => target.remainingHp - 1','((recipient=>recipient.remainingHp-1) as any)');
 const wrappedCard=compileSource(wrapped,file,enums);assert.equal(wrappedCard.status,'supported',JSON.stringify(wrappedCard.diagnostics));assert.deepEqual(wrappedCard.abilities.card[0].effect.amount,expected);
 const captured=text.replace('AbilityHelper.immediateEffects.damage({\n                    amount: (target) => target.remainingHp - 1\n                })','AbilityHelper.immediateEffects.damage(context=>{const saved=context.target.remainingHp;const amountCallback=recipient=>saved-1;return {amount:amountCallback};})');
 assert.notEqual(captured,text);reject(captured,file);
 const across=text.replace('registrar.setEventAbility','const saved=this.controller.getArenaUnits().length;const amountCallback=recipient=>saved-1;registrar.setEventAbility').replace('(target) => target.remainingHp - 1','amountCallback');
 reject(across,file);
});
