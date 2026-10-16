'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={SOR_097:'01_SOR/units/AdmiralAckbarBrilliantStrategist.ts',JTL_135:'04_JTL/units/SpecialForcesTIEFighter.ts'};
function source(id){const file='game/cards/'+files[id];return{text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function compiled(id){const {text,file}=source(id),card=compileSource(text,file,enums);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));return card;}
function rejects(text,file){const c=compileSource(text,file,enums);assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
function mutations(id,pairs){const {text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);rejects(text.replace(from,to),file);}}
test('Ackbar damage uses unscaled friendly count in selected target arena',()=>{
 const a=compiled('SOR_097').abilities.card[0];assert.equal(a.trigger,'whenPlayed');assert.equal(a.optional,true);assert.deepEqual(a.targetResolver,{owner:'any',type:'unit',zone:'anyAttackable'});assert.equal(a.effect.target,'chosen');assert.equal(a.effect.amount.kind,'query');assert.equal(a.effect.amount.name,'arenaUnitCount');assert.deepEqual(a.effect.amount.receiver,{kind:'ref',path:['player']});
 const p=a.effect.amount.args[0];assert.deepEqual(p.right.right.right,{kind:'binary',op:'===',left:{kind:'ref',path:['card','zoneName']},right:{kind:'ref',path:['target','zoneName']}});
 const {text,file}=source('SOR_097');let renamed=text.replaceAll('getDamageFromContext','getCountFromContext').replaceAll('arenaName','selectedArena').replaceAll('const arena =','const friendlyUnits =').replace('return arena.length','return friendlyUnits.length');assert.deepEqual(compileSource(renamed,file,enums).abilities,compiled('SOR_097').abilities);
});
test('TIE comparison uses both controlled space inventories and source ready default',()=>{
 const a=compiled('JTL_135').abilities.card[0];assert.equal(a.trigger,'whenPlayed');assert.equal(a.optional,undefined);assert.equal(a.effect.kind,'conditional');assert.equal(a.effect.condition.op,'<');
 for(const[key,owner]of[['left','player'],['right','opponent']]){const q=a.effect.condition[key];assert.equal(q.name,'arenaUnitCount');assert.deepEqual(q.receiver,{kind:'ref',path:[owner]});assert.deepEqual(q.args[0].right.right.right,{kind:'binary',op:'===',left:{kind:'ref',path:['card','zoneName']},right:{kind:'const',value:'spaceArena'}});}
 assert.deepEqual(a.effect.onTrue,{kind:'ready',target:'self'});assert.deepEqual(a.effect.onFalse,{kind:'noAction'});
 const {text,file}=source('JTL_135');assert.deepEqual(compileSource(text.replaceAll('controlsLessUnitsInSpaceArena','controlsFewerSpaceUnits'),file,enums).abilities,compiled('JTL_135').abilities);
});
test('bounded unit helpers reject body effects, invalid calls and unused methods',()=>{
 mutations('SOR_097',[
 ['const arenaName =','sideEffect(); const arenaName ='],['return arena.length;','const unused = sideEffect(); return arena.length;'],['amount: this.getDamageFromContext(context)','amount: 1'],['this.getDamageFromContext(context)','other.getDamageFromContext(context)'],['this.getDamageFromContext(context)','this.getDamageFromContext(context.target)'],['this.getDamageFromContext(context)','this.getDamageFromContext(context, context)']
 ]);
 mutations('JTL_135',[
 ['return context.player.getArenaUnits','sideEffect(); return context.player.getArenaUnits'],['this.controlsLessUnitsInSpaceArena(context)','this.controlsLessUnitsInSpaceArena(context.source)'],['this.controlsLessUnitsInSpaceArena(context)','this.controlsLessUnitsInSpaceArena()'],['this.controlsLessUnitsInSpaceArena(context)','true']
 ]);
});
function nodes(value,kind) {if(!value||typeof value!=='object')return [];return [...(value.kind===kind?[value]:[]),...Object.values(value).flatMap(v=>nodes(v,kind))];}
function programMutation(id,from,to) {
 const {text,file}=source(id);assert.ok(text.includes(from),from);const c=compileSource(text.replace(from,to),file,enums);
 assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));
 const program=c.privatePrograms[0];assert.equal(program.kind,'privateProgram');assert.equal(program.source,'physicalAbilitySource');
 const call=id==='SOR_097'?c.abilities.card[0].effect.amount:c.abilities.card[0].effect.condition;
 assert.equal(call.kind,'privateProgramCall');assert.equal(call.name,program.name);assert.equal(call.invokeAt,'expressionEvaluation');
 assert.equal(call.args[0].binding.reference,'lexicalContextObject');
 return {c,program,calls:nodes(program,'dataCall').filter(n=>n.method==='getArenaUnits')};
}
test('general private programs preserve changed arena helper values and optional calls',()=>{
 let p=programMutation('SOR_097','const arenaName =','let arenaName =');
 assert.equal(p.program.body.statements[0].bindings[0].mutable,true);
 p=programMutation('SOR_097','return arena.length;','return arena.length * 2;');
 assert.partialDeepStrictEqual(p.program.body.statements.at(-1),{kind:'pureReturn',value:{kind:'binary',op:'*',right:{kind:'const',value:2}}});
 p=programMutation('SOR_097','context.target.zoneName','context.source.zoneName');
 assert.deepEqual(p.program.body.statements[0].bindings[0].value.steps,[{key:'source',optional:false},{key:'zoneName',optional:false}]);
 p=programMutation('SOR_097','context.player.getArenaUnits','context.player.opponent.getArenaUnits');
 assert.deepEqual(p.calls[0].receiver.steps,[{key:'player',optional:false},{key:'opponent',optional:false}]);
 p=programMutation('SOR_097','getArenaUnits({ arena: arenaName })','getArenaUnits({ arena: arenaName, condition:card=>true })');
 assert.equal(p.calls[0].args[0].fields.condition.kind,'pureCallback');assert.deepEqual(p.calls[0].args[0].fields.condition.body,{kind:'pureReturn',value:{kind:'const',value:true}});
 p=programMutation('SOR_097','getArenaUnits({ arena: arenaName })','getArenaUnits?.({ arena: arenaName })');assert.equal(p.calls[0].optionalCall,true);
 for(const op of ['<=','>']){p=programMutation('JTL_135',' < context.player.opponent',` ${op} context.player.opponent`);assert.equal(p.program.body.statements[0].value.op,op);}
 p=programMutation('JTL_135','arena: ZoneName.SpaceArena','arena: ZoneName.GroundArena');
 assert.deepEqual(p.calls.map(c=>c.args[0].fields.arena.value),['groundArena','spaceArena']);
 p=programMutation('JTL_135','getArenaUnits({ arena: ZoneName.SpaceArena })','getArenaUnits({ arena: ZoneName.SpaceArena, condition:card=>true })');assert.equal(p.calls[0].args[0].fields.condition.kind,'pureCallback');
 p=programMutation('JTL_135','getArenaUnits({ arena: ZoneName.SpaceArena })','getArenaUnits?.({ arena: ZoneName.SpaceArena })');assert.equal(p.calls[0].optionalCall,true);assert.equal(p.calls[1].optionalCall,false);
 for(const id of ['SOR_097','JTL_135']){p=programMutation(id,'extends NonLeaderUnitCard','extends EventCard');assert.equal(p.c.baseClass,'EventCard');}
});
test('all helper methods require plain synchronous private context binding and lexical card this',()=>{
 for(const[id,name]of[['SOR_097','getDamageFromContext'],['JTL_135','controlsLessUnitsInSpaceArena']]){
  const {text,file}=source(id),method='private '+name;
  for(const replacement of ['public '+name,'private static '+name,'private async '+name,'private *'+name,'@sideEffect() private '+name])rejects(text.replace(method,replacement),file);
  for(const binding of ['...context','context = sideEffect()','{player:context}','context, other']){
   const from=id==='SOR_097'?'context: AbilityContext): number':'context) {';
   const to=id==='SOR_097'?binding+'): number':binding+') {';
   rejects(text.replace(from,to),file);
  }
  rejects(text.replace('this.'+name+'(context)','this.'+name+'?.(context)'),file);
  rejects(text.replace('this.'+name+'(context)','this.'+name+'(context.player)'),file);
 }
 const ack=source('SOR_097');rejects(ack.text.replace('damage((context) => ({ amount: this.getDamageFromContext(context) }))','damage(function(context) { return {amount:this.getDamageFromContext(context)}; })'),ack.file);
 const tie=source('JTL_135');rejects(tie.text.replace('(context) => this.controlsLessUnitsInSpaceArena(context)','function(context) { return this.controlsLessUnitsInSpaceArena(context); }'),tie.file);
 rejects(tie.text.replace('(context) => this.controlsLessUnitsInSpaceArena(context)','async (context) => this.controlsLessUnitsInSpaceArena(context)'),tie.file);
});
test('helper actual argument aliases and saved-stage contexts remain fail closed',()=>{
 for(const[id,name]of[['SOR_097','getDamageFromContext'],['JTL_135','controlsLessUnitsInSpaceArena']]){const {text,file}=source(id);rejects(text.replace('this.'+name+'(context)','(() => {const alias=context;return this.'+name+'(alias);})()'),file);rejects(text.replace('registrar.addWhenPlayedAbility','const saved=this.controller; registrar.addWhenPlayedAbility').replace('this.'+name+'(context)','this.'+name+'(saved)'),file);}
});

test('space helper context cannot shadow the enum namespace it reads',()=>{
 const {text,file}=source('JTL_135');
 rejects(text.replace('private controlsLessUnitsInSpaceArena(context)','private controlsLessUnitsInSpaceArena(ZoneName)').replace('return context.player.getArenaUnits','return ZoneName.player.getArenaUnits').replace('< context.player.opponent','< ZoneName.player.opponent'),file);
});

test('helper result snapshots follow direct-query callback-stage guards through aliases',()=>{
 const {text,file}=source('JTL_135');
 const original=`AbilityHelper.immediateEffects.conditional({
                condition: (context) => this.controlsLessUnitsInSpaceArena(context),
                onTrue: AbilityHelper.immediateEffects.ready(),
            })`;
 assert.ok(text.includes(original));
 const factory=(declarations,condition)=>text.replace(original,`AbilityHelper.immediateEffects.conditional(context=>{${declarations} return {condition:${condition},onTrue:AbilityHelper.immediateEffects.ready()};})`);
 for(const declarations of [
  'const saved=this.controlsLessUnitsInSpaceArena(context);',
  'const initial=this.controlsLessUnitsInSpaceArena(context); const saved=initial;',
  'const initial=this.controlsLessUnitsInSpaceArena(context); const second=initial; const saved=second;'
 ]){
  const rejected=compileSource(factory(declarations,'()=>saved'),file,enums);
  assert.equal(rejected.status,'unsupported');assert.ok(rejected.diagnostics.some(d=>d.pattern==='arena-damage-helper-call-scope'),JSON.stringify(rejected.diagnostics));
  const sameStage=compileSource(factory(declarations,'saved'),file,enums);assert.equal(sameStage.status,'supported',JSON.stringify(sameStage.diagnostics));assert.deepEqual(sameStage.abilities,compiled('JTL_135').abilities);
 }
 rejects(factory('const captured=context;','later=>this.controlsLessUnitsInSpaceArena(captured)'),file);
 rejects(factory('const captured=context; const second=captured;','later=>this.controlsLessUnitsInSpaceArena(second)'),file);
 rejects(factory('const owner=context.player; const captured=owner;','later=>this.controlsLessUnitsInSpaceArena(captured)'),file);
 rejects(factory('const receiver=this;','later=>receiver.controlsLessUnitsInSpaceArena(later)'),file);
 rejects(factory('const receiver=this; const second=receiver;','later=>second.controlsLessUnitsInSpaceArena(later)'),file);
});
test('unscaled damage helper result retains same-stage value and rejects later callback capture',()=>{
 const {text,file}=source('SOR_097');const original='damage((context) => ({ amount: this.getDamageFromContext(context) }))';assert.ok(text.includes(original));
 const sameStage=text.replace(original,'damage(context=>{const initial=this.getDamageFromContext(context);const amount=initial;return {amount};})');const card=compileSource(sameStage,file,enums);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));assert.deepEqual(card.abilities,compiled('SOR_097').abilities);
 const later=text.replace(original,'damage(context=>{const initial=this.getDamageFromContext(context);const amount=initial;return {amount: later=>amount};})');rejects(later,file);
});
