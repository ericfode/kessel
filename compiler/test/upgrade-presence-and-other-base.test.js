'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={LOF_244:'05_LOF/units/JediVector.ts',SEC_095:'06_SEC/units/TheedSecurity.ts',TS26_042:'07_TS26/units/ReliefFrigate.ts',JTL_144:'04_JTL/events/NoDisintegrations.ts'};
function source(id){const file='game/cards/'+files[id];return {text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function full(id){const {text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(text,file){const c=compileSource(text,file,enums);assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
function represented(text,file){const c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c.abilities.card[0];}
function recipientAmount(ability,{field=['remainingHp'],subtract=1,method}={}){
 const amount=ability.effect.amount;assert.equal(amount.kind,'nativeDamageRecipientAmount');assert.equal(amount.evaluateAt,'addAbilityDamagePropertiesToEachEvent');assert.deepEqual(amount.arguments,['actualEventRecipient']);assert.equal(amount.eventResolution,'useStoredEventAmount');
 const callback=amount.callback;assert.equal(callback.parameters[0].argument,'damagedCard');assert.equal(callback.capture,'lexicalReferences');assert.equal(callback.invokeAt,'nativeCallbackInvocation');
 const value=callback.body.value;assert.equal(value.op,'-');assert.deepEqual(value.right,{kind:'const',value:subtract});
 const receiver=method?value.left.receiver:value.left;if(method){assert.equal(value.left.kind,'dataCall');assert.equal(value.left.method,method);assert.deepEqual(value.left.args,[]);}
 assert.deepEqual(receiver.root,[callback.parameters[0].slot]);assert.deepEqual(receiver.steps.map(s=>s.key),field);
}
function collectionRecipient(ability,{field='base',compare='target',op='!==',type='base',upTo=false,selector=true}={}){
 if(selector){assert.equal(ability.targetResolver.type,type);if(upTo){assert.equal(ability.targetResolver.mode,'upTo');assert.equal(ability.targetResolver.count.value,2);}}else assert.equal(ability.targetResolver,undefined);
 const f=ability.effect;assert.equal(f.kind,'systemPropertiesFactory');assert.equal(f.invocationPolicy,'eachNativePropertyGeneration');
 const capture=f.captures[0],query=capture.value;assert.equal(capture.evaluateAt,'propertyFactoryInvocation');assert.equal(query.kind,'constructionCollectionCall');assert.equal(query.method,'filter');
 assert.deepEqual(query.receiver.args,[{kind:'ref',path:['card',field]}]);assert.equal(query.receiver.receiver.kind,'gamePlayersQuery');assert.equal(query.receiver.receiver.order,'playersAndSpectatorsObjectValues');
 const predicate=query.args[0];assert.equal(predicate.body.op,op);assert.deepEqual(predicate.body.left.path,[predicate.parameters[0].root]);assert.deepEqual(predicate.body.right.path,[f.parameters[0].root,compare]);
 assert.deepEqual(f.effect.target,{kind:'ref',path:['systemFactoryCaptures',capture.name]});assert.equal(f.effect.kind,'heal');assert.equal(f.effect.amount.value,3);
}

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
test('upgrade alias preserves explicit query captures while malformed filters and receivers fail closed',()=>{
 const {text,file}=source('SEC_095');for(const replacement of [
  'hasSomeArenaUpgrade?.()', 'hasSomeArenaUpgrade({}, {})', 'hasSomeArenaUpgrade({type:"unit"})',
  'hasSomeArenaUpgrade({controller:"self"})'
 ])reject(text.replace('hasSomeArenaUpgrade()',replacement),file);
 reject(text.replace('context.player.opponent.hasSomeArenaUpgrade()','context.source.hasSomeArenaUpgrade()'),file);
 const nested=represented(text.replace('condition: (context) => context.player.opponent.hasSomeArenaUpgrade()', 'condition: (context) => {const saved=context.player.opponent.hasSomeArenaUpgrade();return other=>saved;}'),file).effect.condition;
 assert.equal(nested.kind,'pureCallback');const [binding,returned]=nested.body.statements;assert.equal(binding.bindings[0].evaluateAt,'declaration');assert.equal(binding.bindings[0].value.kind,'scopedArenaSomeQuery');assert.deepEqual(returned.value.body.value.root,[binding.bindings[0].slot]);assert.equal(returned.value.capture,'lexicalReferences');
 const captured=`export default class Probe extends NonLeaderUnitCard {getImplementationId(){return{id:'probe',internalName:'probe'}} setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.conditional(context=>{const saved=context.player.hasSomeArenaUpgrade();const alias=saved;return {condition:later=>alias,onTrue:AbilityHelper.immediateEffects.draw()};})});}}`;
 const snapshot=represented(captured,'snapshot.ts').effect;assert.equal(snapshot.kind,'systemPropertiesFactory');assert.equal(snapshot.invocationPolicy,'eachNativePropertyGeneration');const [saved,alias]=snapshot.captures;assert.equal(saved.value.kind,'scopedArenaSomeQuery');assert.equal(saved.value.type,'upgrade');assert.equal(saved.evaluateAt,'propertyFactoryInvocation');assert.deepEqual(alias.value,{kind:'ref',path:['systemFactoryCaptures',saved.name]});assert.deepEqual(snapshot.effect.condition,{kind:'ref',path:['systemFactoryCaptures',alias.name]});
 const controller=`export default class Probe extends NonLeaderUnitCard {getImplementationId(){return{id:'probe',internalName:'probe'}} setupCardAbilities(registrar,AbilityHelper){const owner=this.controller;registrar.addWhenPlayedAbility({immediateEffect:AbilityHelper.immediateEffects.conditional({condition:context=>owner.hasSomeArenaUpgrade(),onTrue:AbilityHelper.immediateEffects.draw()})});}}`;
 const receiver=compileSource(controller,'controller.ts',enums);assert.equal(receiver.status,'unsupported');assert.ok(receiver.diagnostics.some(d=>d.pattern==='arena-query-snapshot-capture'),JSON.stringify(receiver.diagnostics));
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
test('player collection lowering rejects malformed calls and callback parameters',()=>{
 const {text,file}=source('TS26_042');for(const changed of [
  text.replace('getPlayers()','getPlayers(1)'),text.replace('getPlayers()','getPlayers?.()'),
  text.replace('.map((x) => x.base)', '.map(async (x) => x.base)'),
  text.replace('.map((x) => x.base)', '.map((...x) => x.base)'),
  text.replace('.map((x) => x.base)', '.map((x = sideEffect()) => x.base)'),
 ])reject(changed,file);
});
test('direct damage callback binds the recipient rather than the ability context',()=>{
 const ability=full('JTL_144').abilities.card[0];assert.equal(ability.trigger,'event');
 assert.deepEqual(ability.targetResolver,{owner:'any',type:'nonLeaderUnit',zone:'anyAttackable'});
 assert.deepEqual(ability.effect,{kind:'damage',target:'chosen',amount:{kind:'binary',op:'-',left:{kind:'ref',path:['target','remainingHp']},right:{kind:'const',value:1}}});
});
test('recipient damage callback rejects malformed bindings and unbound context destinations',()=>{
 const {text,file}=source('JTL_144');for(const changed of [
  text.replace('(target) =>','async (target) =>'),text.replace('(target) =>','(...target) =>'),
  text.replace('(target) =>','(target = sideEffect()) =>'),text.replace('(target) =>','(target,context) =>'),
  text.replace('target.remainingHp - 1','context.source.remainingHp - 1'),
  text.replace('amount: (target)', 'target:context.source, amount: (target)'),
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
  '((true ? (context=>context.target.remainingHp-1) : (recipient=>recipient.remainingHp-1)) as any)'
 ]){
  reject(text.replace('(target) => target.remainingHp - 1',composite),file);
  reject(aliased('const value='+composite+';const amountCallback=value;'),file);
 }
 const wrapped=text.replace('(target) => target.remainingHp - 1','((recipient=>recipient.remainingHp-1) as any)');
 const wrappedCard=compileSource(wrapped,file,enums);assert.equal(wrappedCard.status,'supported',JSON.stringify(wrappedCard.diagnostics));assert.deepEqual(wrappedCard.abilities.card[0].effect.amount,expected);
 const captured=text.replace('AbilityHelper.immediateEffects.damage({\n                    amount: (target) => target.remainingHp - 1\n                })','AbilityHelper.immediateEffects.damage(context=>{const saved=context.target.remainingHp;const amountCallback=recipient=>saved-1;return {amount:amountCallback};})');
 assert.notEqual(captured,text);const factory=represented(captured,file).effect;assert.equal(factory.kind,'systemPropertiesFactory');const capture=factory.captures[0];assert.deepEqual(capture.value.path,[factory.parameters[0].root,'target','remainingHp']);assert.equal(capture.evaluateAt,'propertyFactoryInvocation');assert.deepEqual(factory.effect.amount.callback.body.value.left,{kind:'ref',path:['systemFactoryCaptures',capture.name]});assert.equal(factory.effect.amount.evaluateAt,'addAbilityDamagePropertiesToEachEvent');
 const across=text.replace('registrar.setEventAbility','const saved=this.controller.getArenaUnits().length;const amountCallback=recipient=>saved-1;registrar.setEventAbility').replace('(target) => target.remainingHp - 1','amountCallback');
 reject(across,file);
});

test('scoped upgrade filters preserve predicate, arena and empty trait operands',()=>{
 const {text,file}=source('SEC_095');
 for(const [replacement,key,expected] of [
 ['hasSomeArenaUpgrade({condition:c=>true})','condition',{kind:'const',value:true}],
 ['hasSomeArenaUpgrade({arena:"groundArena"})','arena',{kind:'const',value:'groundArena'}],
 ['hasSomeArenaUpgrade({trait:[]})','trait',{kind:'array',items:[]}],
 ['hasSomeArenaUpgrade({trait:""})','trait',{kind:'const',value:''}]
 ]){const q=represented(text.replace('hasSomeArenaUpgrade()',replacement),file).effect.condition;assert.equal(q.kind,'scopedArenaSomeQuery');assert.equal(q.type,'upgrade');assert.deepEqual(q.receiver,{kind:'ref',path:['opponent']});assert.equal(q.controller,'receiverIdentity');assert.equal(q.order,'groundThenSpacePhysicalOrder');if(key==='condition'){assert.equal(q.filter.condition.kind,'queryCallback');assert.equal(q.filter.condition.parameters[0].role,'candidate');assert.deepEqual(q.filter.condition.body,expected);}else assert.deepEqual(q.filter[key],expected);}
});
test('general player collection targets preserve mapped field, comparison, selection and capture timing',()=>{
 const {text,file}=source('TS26_042');for(const [changed,options] of [
 [text.replace('.map((x) => x.base)', '.map((x) => x.hand)'),{field:'hand'}],
 [text.replace('x !== context.target','x !== context.source'),{compare:'source'}],
 [text.replace('x !== context.target','x === context.target'),{op:'==='}],
 [text.replace('cardTypeFilter: CardType.Base','cardTypeFilter: WildcardCardType.Unit'),{type:'unit'}],
 [text.replace('cardTypeFilter: CardType.Base','cardTypeFilter: CardType.Base, mode:"upTo", numCards:2'),{upTo:true}],
 [text.replace('targetResolver: {\n                cardTypeFilter: CardType.Base,\n                immediateEffect:', 'immediateEffect:').replace(/\n            }\n        }\);/,'\n        });'),{selector:false}]
 ])collectionRecipient(represented(changed,file),options);
});
test('native recipient amount programs preserve arithmetic and selected recipient cardinality',()=>{
 const {text,file}=source('JTL_144');
 recipientAmount(represented(text.replace('target.remainingHp - 1','target.remainingHp - 2'),file),{subtract:2});
 recipientAmount(represented(text.replace('target.remainingHp - 1','target.getPower() - 1'),file),{method:'getPower',field:[]});
 const base=represented(text.replace('WildcardCardType.NonLeaderUnit','CardType.Base'),file);assert.equal(base.targetResolver.type,'base');recipientAmount(base);
 const multiple=represented(text.replace('cardTypeFilter: WildcardCardType.NonLeaderUnit','cardTypeFilter: WildcardCardType.NonLeaderUnit, mode:"upTo", numCards:2'),file);assert.equal(multiple.targetResolver.mode,'upTo');assert.equal(multiple.targetResolver.count.value,2);recipientAmount(multiple);
});
test('recipient formal names and readonly aliases retain object member paths',()=>{
 const {text,file}=source('JTL_144');for(const [declaration,field] of [
 ['const amountCallback = context => context.target.remainingHp - 1;','target'],
 ['const amountCallback = context => context.source.remainingHp - 1;','source'],
 ['const original = context => context.target.remainingHp - 1; const amountCallback = original;','target']
 ])recipientAmount(represented(text.replace('registrar.setEventAbility',declaration+'\n        registrar.setEventAbility').replace('(target) => target.remainingHp - 1','amountCallback'),file),{field:[field,'remainingHp']});
 const cast='((context=>context.target.remainingHp-1) as any)';
 recipientAmount(represented(text.replace('(target) => target.remainingHp - 1',cast),file),{field:['target','remainingHp']});
 recipientAmount(represented(text.replace('registrar.setEventAbility','const value='+cast+';const amountCallback=value;\n        registrar.setEventAbility').replace('(target) => target.remainingHp - 1','amountCallback'),file),{field:['target','remainingHp']});
});
