'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={SEC_130:'06_SEC/events/FerrixUprising.ts',SHD_156:'02_SHD/events/CrippleAuthority.ts'};
function source(id){const file='game/cards/'+files[id];return {text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function compile(text,file){return compileSource(text,file,enums);}
function full(id){const {text,file}=source(id),card=compile(text,file);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));return card;}
function rejects(text,file){const card=compile(text,file);assert.equal(card.status,'unsupported',JSON.stringify(card.abilities));}
function mutations(id,pairs){const {text,file}=source(id);for(const [from,to]of pairs){assert.ok(text.includes(from),from);rejects(text.replace(from,to),file);}}
test('pure helper preserves selected target arena independently from counted card identity',()=>{
 const ability=full('SEC_130').abilities.card[0];assert.equal(ability.trigger,'event');assert.deepEqual(ability.targetResolver,{owner:'any',type:'unit',zone:'anyAttackable'});
 assert.equal(ability.effect.kind,'damage');assert.equal(ability.effect.target,'chosen');
 const amount=ability.effect.amount;assert.equal(amount.op,'*');assert.deepEqual(amount.right,{kind:'const',value:2});assert.equal(amount.left.name,'arenaUnitCount');assert.deepEqual(amount.left.receiver,{kind:'ref',path:['player']});
 const predicate=JSON.stringify(amount.left.args);assert.ok(predicate.includes('"card","zoneName"'));assert.ok(predicate.includes('"target","zoneName"'));assert.ok(!predicate.includes('"source"'));
 const {text,file}=source('SEC_130');const renamed=text.replaceAll('getDamageFromContext','getArenaDamageFromContext').replaceAll('arenaName','chosenArena').replaceAll('const arena =','const friendly =').replace('return arena.length','return friendly.length');assert.deepEqual(compile(renamed,file).abilities,full('SEC_130').abilities);
});
test('helper body purity, binding, hook protection and exact arithmetic fail closed',()=>{
 mutations('SEC_130',[
 ['private getDamageFromContext','public getDamageFromContext'],['private getDamageFromContext','private static getDamageFromContext'],['private getDamageFromContext','private async getDamageFromContext'],['private getDamageFromContext','private *getDamageFromContext'],['private getDamageFromContext','@sideEffect() private getDamageFromContext'],
 ['context: AbilityContext): number','...context: AbilityContext): number'],['context: AbilityContext): number','context: AbilityContext = sideEffect()): number'],['context: AbilityContext): number','context: AbilityContext, other: AbilityContext): number'],
 ['const arenaName =','let arenaName ='],['const arenaName =','sideEffect(); const arenaName ='],['return arena.length * 2;','const unused = sideEffect(); return arena.length * 2;'],['return arena.length * 2;','return arena.length * 3;'],['context.target.zoneName','context.source.zoneName'],['context.player.getArenaUnits','context.player.opponent.getArenaUnits'],['getArenaUnits({ arena: arenaName })','getArenaUnits({ arena: arenaName, condition: card => true })'],['getArenaUnits({ arena: arenaName })','getArenaUnits?.({ arena: arenaName })'],['this.getDamageFromContext(context)','this.getDamageFromContext(context.target)'],['this.getDamageFromContext(context)','this.getDamageFromContext(context, context)'],['this.getDamageFromContext(context)','this.getDamageFromContext?.(context)'],['this.getDamageFromContext(context)','other.getDamageFromContext(context)'],['amount: this.getDamageFromContext(context)','amount: 2'],['extends EventCard','extends NonLeaderUnitCard']
 ]);
 const {text,file}=source('SEC_130');rejects(text.replaceAll('getDamageFromContext','getPower'),file);
 const declaration=text.slice(text.indexOf('    private getDamageFromContext'),text.indexOf('    public override setupCardAbilities'));rejects(text.replace(declaration,declaration+declaration),file);
});
test('finite simultaneous factory preserves draw order, opponent chooser and resource comparison snapshot',()=>{
 const effect=full('SHD_156').abilities.card[0].effect;
 assert.deepEqual(effect,{kind:'simultaneous',effects:[{kind:'draw',target:'ownPlayer',amount:{kind:'const',value:1}},{kind:'conditional',condition:{kind:'binary',op:'>',left:{kind:'ref',path:['opponent','resources','length']},right:{kind:'ref',path:['player','resources','length']}},onTrue:{kind:'discardOwnHand',target:'enemyPlayer',amount:{kind:'const',value:1}},onFalse:{kind:'noAction'}}]});
});
test('factory grammar rejects altered groups, callable composites and callback bindings',()=>{
 mutations('SHD_156',[
 ['(context) => [','async (context) => ['],['(context) => [','(...context) => ['],['(context) => [','(context = sideEffect()) => ['],['(context) => [','(context, extra) => ['],
 ['immediateEffects.simultaneous(','immediateEffects.sequential('],['amount: 1','amount: 2'],['target: context.player,','target: context.player.opponent,'],['target: context.player.opponent,','target: context.player,'],['resources.length > context.player.resources.length','resources.length >= context.player.resources.length'],['condition: context.player.opponent.resources.length','condition: later => later.player.opponent.resources.length'],['context.player.opponent.resources.length','context.player.opponent.readyResourceCount']
 ]);
});
test('saved setup booleans and receiver aliases cannot erase factory snapshot stage',()=>{
 const {text,file}=source('SHD_156');
 for(const [declaration,replacement]of [
  ['const saved = this.controller.opponent.resources.length > this.controller.resources.length;','saved'],
  ['const owner = this.controller;','owner.opponent.resources.length > owner.resources.length'],
  ['const factory = context => [];','factory']
 ]){
  let changed=text.replace('registrar.setEventAbility',declaration+'\n        registrar.setEventAbility');
  changed=replacement==='factory'?changed.replace('(context) => [','factory || (context) => ['):changed.replace('context.player.opponent.resources.length > context.player.resources.length',replacement);
  rejects(changed,file);
 }
 const sec=source('SEC_130');rejects(sec.text.replace('amount: this.getDamageFromContext(context)','amount: (() => this.getDamageFromContext(context))()'),sec.file);
});

test('card helper requires lexical this through every callback ancestor',()=>{
 const {text,file}=source('SEC_130');
 const ordinary=text.replace('damage((context) => ({','damage(function(context) { return ({').replace('                }))','                }); })');
 rejects(ordinary,file);
 rejects(text.replace('damage((context) => ({','damage(function(outer) { return (context => ({').replace('                }))','                }))(outer); })'),file);
});

test('factory parameter cannot shadow invoked ability-helper namespace',()=>{
 const {text,file}=source('SHD_156');
 rejects(text.replace('(context) => [','(AbilityHelper) => [').replaceAll('context.player','AbilityHelper.player'),file);
});

test('well-formed factory aliases, composites, blocks and generator functions fail closed',()=>{
 const {text,file}=source('SHD_156');
 const start=text.indexOf('(context) => ['),end=text.indexOf('\n            )',start);
 const factory=text.slice(start,end);
 for(const replacement of [
  `function*(context) { return ${factory.slice(factory.indexOf('['))}; }`,
  `function(context) { return ${factory.slice(factory.indexOf('['))}; }`,
  `true ? (${factory}) : (${factory})`,
  `(${factory}) || (${factory})`,
  `context => { return ${factory.slice(factory.indexOf('['))}; }`,
  'savedFactory'
 ]){
  let changed=text.slice(0,start)+replacement+text.slice(end);
  if(replacement==='savedFactory')changed=changed.replace('registrar.setEventAbility',`const savedFactory = ${factory}; registrar.setEventAbility`);
  rejects(changed,file);
 }
});
