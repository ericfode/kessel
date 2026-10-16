'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={SEC_130:'06_SEC/events/FerrixUprising.ts',SHD_156:'02_SHD/events/CrippleAuthority.ts'};
function source(id){const file='game/cards/'+files[id];return {text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function compile(text,file){return compileSource(text,file,enums);}
function full(id){const {text,file}=source(id),card=compile(text,file);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));return card;}
function rejects(text,file){const card=compile(text,file);assert.equal(card.status,'unsupported',JSON.stringify(card.abilities));}
function represented(text,file,check){const card=compile(text,file);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));check(card);return card;}
function mutations(id,pairs){const {text,file}=source(id);for(const [from,to,check]of pairs){assert.ok(text.includes(from),from);if(check)represented(text.replace(from,to),file,check);else rejects(text.replace(from,to),file);}}
function privateContract(card,{mutable=false,multiplier=2,arena='target',owner=['player'],optional=false,condition=false,name='getDamageFromContext',base='EventCard'}={}){
 assert.equal(card.baseClass,base);const call=card.abilities.card[0].effect.amount;
 assert.equal(call.kind,'privateProgramCall');assert.equal(call.name,name);assert.equal(call.source,'physicalAbilitySource');assert.equal(call.invokeAt,'expressionEvaluation');
 const program=card.privatePrograms.find(p=>p.name===name),[first,second,last]=program.body.statements;
 assert.equal(program.source,'physicalAbilitySource');assert.equal(first.bindings[0].mutable,mutable);
 assert.deepEqual(first.bindings[0].value.steps.map(s=>s.key),[arena,'zoneName']);
 const query=second.bindings[0].value;assert.equal(query.method,'getArenaUnits');assert.deepEqual(query.receiver.steps.map(s=>s.key),owner);assert.equal(query.optionalCall,optional);
 assert.deepEqual(query.args[0].fields.arena.root,[first.bindings[0].slot]);
 if(condition)assert.deepEqual(query.args[0].fields.condition.body,{kind:'pureReturn',value:{kind:'const',value:true}});
 assert.equal(last.value.op,'*');assert.deepEqual(last.value.right,{kind:'const',value:multiplier});assert.deepEqual(last.value.left.root,[second.bindings[0].slot]);
}
function factoryContract(card,{mode='simultaneous',draw=1,drawOwner=['player'],discardOwner=['player','opponent'],op='>',resource='resources',deferred=false}={}){
 const f=card.abilities.card[0].effect;assert.equal(f.kind,'systemArrayFactory');assert.equal(f.mode,mode);assert.equal(f.invocationPolicy,'eachNativePropertyGeneration');
 assert.equal(f.constructionOrder,'readonlyDeclarationsThenArrayLeftToRight');assert.equal(f.callbacks,'deferredWithLexicalBindings');
 const root=f.parameters[0].root,[drawing,conditional]=f.effects;
 assert.deepEqual(f.captures[0].value,{kind:'ref',path:[root,...drawOwner]});assert.equal(f.captures[0].evaluateAt,'arrayFactoryInvocation');
 assert.deepEqual(drawing.target,{kind:'ref',path:['systemFactoryCaptures',f.captures[0].name]});assert.equal(drawing.amount.value,draw);
 const predicate=deferred?conditional.condition:f.captures[1].value;
 assert.equal(predicate.op,op);assert.deepEqual(predicate.left.path,deferred?['opponent','resources','length']:[root,'player','opponent',...(resource==='resources'?['resources','length']:[resource])]);assert.deepEqual(predicate.right.path.slice(1),['player','resources','length']);
 const target=f.captures.at(-1);assert.deepEqual(target.value.path,[root,...discardOwner]);assert.deepEqual(conditional.onTrue.target,{kind:'ref',path:['systemFactoryCaptures',target.name]});assert.equal(conditional.onTrue.actualChooser,'affectedPlayer');
}

test('pure helper preserves selected target arena independently from counted card identity',()=>{
 const ability=full('SEC_130').abilities.card[0];assert.equal(ability.trigger,'event');assert.deepEqual(ability.targetResolver,{owner:'any',type:'unit',zone:'anyAttackable'});
 assert.equal(ability.effect.kind,'damage');assert.equal(ability.effect.target,'chosen');
 const amount=ability.effect.amount;assert.equal(amount.op,'*');assert.deepEqual(amount.right,{kind:'const',value:2});assert.equal(amount.left.name,'arenaUnitCount');assert.deepEqual(amount.left.receiver,{kind:'ref',path:['player']});
 const predicate=JSON.stringify(amount.left.args);assert.ok(predicate.includes('"card","zoneName"'));assert.ok(predicate.includes('"target","zoneName"'));assert.ok(!predicate.includes('"source"'));
 const {text,file}=source('SEC_130');const renamed=text.replaceAll('getDamageFromContext','getArenaDamageFromContext').replaceAll('arenaName','chosenArena').replaceAll('const arena =','const friendly =').replace('return arena.length','return friendly.length');assert.deepEqual(compile(renamed,file).abilities,full('SEC_130').abilities);
});
test('helper programs preserve arithmetic and binding while impure or malformed helpers fail closed',()=>{
 mutations('SEC_130',[
 ['private getDamageFromContext','public getDamageFromContext'],['private getDamageFromContext','private static getDamageFromContext'],['private getDamageFromContext','private async getDamageFromContext'],['private getDamageFromContext','private *getDamageFromContext'],['private getDamageFromContext','@sideEffect() private getDamageFromContext'],
 ['context: AbilityContext): number','...context: AbilityContext): number'],['context: AbilityContext): number','context: AbilityContext = sideEffect()): number'],['context: AbilityContext): number','context: AbilityContext, other: AbilityContext): number'],
 ['const arenaName =','let arenaName =',c=>privateContract(c,{mutable:true})],['const arenaName =','sideEffect(); const arenaName ='],['return arena.length * 2;','const unused = sideEffect(); return arena.length * 2;'],['return arena.length * 2;','return arena.length * 3;',c=>privateContract(c,{multiplier:3})],['context.target.zoneName','context.source.zoneName',c=>privateContract(c,{arena:'source'})],['context.player.getArenaUnits','context.player.opponent.getArenaUnits',c=>privateContract(c,{owner:['player','opponent']})],['getArenaUnits({ arena: arenaName })','getArenaUnits({ arena: arenaName, condition: card => true })',c=>privateContract(c,{condition:true})],['getArenaUnits({ arena: arenaName })','getArenaUnits?.({ arena: arenaName })',c=>privateContract(c,{optional:true})],['this.getDamageFromContext(context)','this.getDamageFromContext(context.target)'],['this.getDamageFromContext(context)','this.getDamageFromContext(context, context)'],['this.getDamageFromContext(context)','this.getDamageFromContext?.(context)'],['this.getDamageFromContext(context)','other.getDamageFromContext(context)'],['amount: this.getDamageFromContext(context)','amount: 2'],['extends EventCard','extends NonLeaderUnitCard',c=>privateContract(c,{base:'NonLeaderUnitCard'})]
 ]);
 const {text,file}=source('SEC_130');represented(text.replaceAll('getDamageFromContext','getPower'),file,c=>privateContract(c,{name:'getPower'}));
 const declaration=text.slice(text.indexOf('    private getDamageFromContext'),text.indexOf('    public override setupCardAbilities'));rejects(text.replace(declaration,declaration+declaration),file);
});
test('finite simultaneous factory preserves draw order, opponent chooser and resource comparison snapshot',()=>{
 const effect=full('SHD_156').abilities.card[0].effect;
 assert.deepEqual(effect,{kind:'simultaneous',effects:[{kind:'draw',target:'ownPlayer',amount:{kind:'const',value:1}},{kind:'conditional',condition:{kind:'binary',op:'>',left:{kind:'ref',path:['opponent','resources','length']},right:{kind:'ref',path:['player','resources','length']}},onTrue:{kind:'discardOwnHand',target:'enemyPlayer',amount:{kind:'const',value:1}},onFalse:{kind:'noAction'}}]});
});
test('factory groups preserve ordering, arithmetic, recipients and deferred conditions',()=>{
 mutations('SHD_156',[
 ['(context) => [','async (context) => ['],['(context) => [','(...context) => ['],['(context) => [','(context = sideEffect()) => ['],['(context) => [','(context, extra) => [',c=>{const f=c.abilities.card[0].effect;assert.equal(f.kind,'systemArrayProgram');assert.deepEqual(f.callback.parameters.map(p=>p.argument),['systemFactoryContext','argument1']);assert.equal(f.callback.body.value.kind,'validatedSystemArray');assert.equal(f.callback.body.value.value.evaluationOrder,'leftToRight');const root=f.callback.parameters[0].slot,[draw,conditional]=f.callback.body.value.value.items;assert.equal(draw.effect.kind,'draw');assert.equal(draw.effect.amount.value,1);assert.deepEqual(draw.captures[0].value.root,[root]);assert.deepEqual(draw.captures[0].value.steps,[{key:'player',optional:false}]);assert.equal(draw.captures[0].evaluateAt,'constructorArgumentEvaluation');const comparison=conditional.captures[0].value;assert.equal(comparison.op,'>');assert.deepEqual(comparison.left.root,[root]);assert.deepEqual(comparison.left.steps.map(s=>s.key),['player','opponent','resources','length']);assert.deepEqual(comparison.right.steps.map(s=>s.key),['player','resources','length']);assert.equal(conditional.effect.onTrue.actualChooser,'affectedPlayer');assert.deepEqual(conditional.captures[1].value.steps.map(s=>s.key),['player','opponent']);}],
 ['immediateEffects.simultaneous(','immediateEffects.sequential(',c=>factoryContract(c,{mode:'sequential'})],['amount: 1','amount: 2',c=>factoryContract(c,{draw:2})],['target: context.player,','target: context.player.opponent,',c=>factoryContract(c,{drawOwner:['player','opponent']})],['target: context.player.opponent,','target: context.player,',c=>factoryContract(c,{discardOwner:['player']})],['resources.length > context.player.resources.length','resources.length >= context.player.resources.length',c=>factoryContract(c,{op:'>='})],['condition: context.player.opponent.resources.length','condition: later => later.player.opponent.resources.length',c=>factoryContract(c,{deferred:true})],['context.player.opponent.resources.length','context.player.opponent.readyResourceCount',c=>factoryContract(c,{resource:'readyResourceCount'})]
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

test('factory blocks and functions preserve construction; composites, aliases and generators fail closed',()=>{
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
  if(replacement.startsWith('function(context)')||replacement.startsWith('context => {'))represented(changed,file,c=>factoryContract(c));
  else rejects(changed,file);
 }
});
