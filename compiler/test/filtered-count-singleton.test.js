'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={LOF_121:'05_LOF/units/ThePurgilKingLeadingTheJourney.ts',JTL_199:'04_JTL/units/BladeSquadronBWing.ts',LOF_054:'05_LOF/events/CalmInTheStorm.ts'};
function source(id){const file='game/cards/'+files[id];return{text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function full(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(text,file){const c=compileSource(text,file,enums);assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
function mutations(id,pairs){const{text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);reject(text.replace(from,to),file);}}
test('Purrgil filtered count reads each friendly candidate remaining HP across both arenas',()=>{
 const a=full('LOF_121').abilities.card[0];assert.equal(a.trigger,'whenPlayed');assert.deepEqual(a.effect,{kind:'draw',target:'ownPlayer',amount:{kind:'query',name:'arenaUnitCount',receiver:{kind:'ref',path:['player']},args:[{kind:'binary',op:'&&',left:{kind:'const',value:true},right:{kind:'binary',op:'>=',left:{kind:'ref',path:['card','remainingHp']},right:{kind:'const',value:7}}}]}});
 const{text,file}=source('LOF_121');for(const changed of [text.replaceAll('getDrawAmount','getCardAmount').replaceAll('(card) => card.remainingHp','(target) => target.remainingHp'),text.replace('(card) => card.remainingHp','(context) => context.remainingHp')])assert.deepEqual(compileSource(changed,file,enums).abilities,full('LOF_121').abilities);
});
test('B-Wing filtered count preserves opponent controller, exhausted status and threshold',()=>{
 const e=full('JTL_199').abilities.card[0].effect;assert.equal(e.kind,'conditional');assert.equal(e.condition.op,'>=');assert.deepEqual(e.condition.right,{kind:'const',value:3});const q=e.condition.left;assert.equal(q.name,'arenaUnitCount');assert.deepEqual(q.receiver,{kind:'ref',path:['opponent']});assert.deepEqual(q.args[0].right,{kind:'binary',op:'&&',left:{kind:'query',name:'isUnit',receiver:{kind:'ref',path:['card']},args:[]},right:{kind:'binary',op:'===',left:{kind:'ref',path:['card','exhausted']},right:{kind:'const',value:true}}});assert.deepEqual(e.onTrue,{kind:'select',targetResolver:{owner:'any',type:'unit',zone:'anyAttackable'},effect:{kind:'giveShield',target:'chosen',amount:{kind:'const',value:1}}});assert.deepEqual(e.onFalse,{kind:'noAction'});
 const{text,file}=source('JTL_199');assert.deepEqual(compileSource(text.replaceAll('opponentHasAtLeastThreeExhausted','opponentHasEnoughExhausted').replaceAll('exhaustedUnits','filteredUnits'),file,enums).abilities,full('JTL_199').abilities);
});
test('filtered helper grammar rejects invalid calls and side effects',()=>{
 mutations('LOF_121',[
 ['.filter((card) => card.remainingHp >= 7)','.filter((card) => card.remainingHp >= 7, sideEffect())'],['return context.player','sideEffect(); return context.player'],['return context.player','const unused=sideEffect(); return context.player']
 ]);
 mutations('JTL_199',[
 ['const exhaustedUnits','sideEffect(); const exhaustedUnits'],['return exhaustedUnits.length >= 3','const unused=sideEffect(); return exhaustedUnits.length >= 3']
 ]);
});
function nodes(value,kind) {if(!value||typeof value!=='object')return [];return [...(value.kind===kind?[value]:[]),...Object.values(value).flatMap(v=>nodes(v,kind))];}
function represented(id,from,to) {
 const {text,file}=source(id);assert.ok(text.includes(from),from);const c=compileSource(text.replace(from,to),file,enums);
 assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));
 const program=(c.privatePrograms||c.historyHelpers)[0];
 const call=id==='LOF_121'?c.abilities.card[0].effect.amount:c.abilities.card[0].effect.condition;
 assert.equal(call.name,program.name);assert.equal(call.kind,program.kind==='privateProgram'?'privateProgramCall':'historyHelperCall');
 return {program,arena:nodes(program,'dataCall').find(c=>c.method==='getArenaUnits')};
}
test('general helper programs preserve list calls, predicate operands and thresholds',()=>{
 let p=represented('LOF_121','getArenaUnits()','getArenaUnits({})');assert.deepEqual(p.arena.args,[{kind:'dataRecord',fields:{}}]);
 p=represented('LOF_121','getArenaUnits()','getArenaUnits?.()');assert.equal(p.arena.optionalCall,true);
 p=represented('LOF_121','getArenaUnits()','getArenaUnits(1)');assert.deepEqual(p.arena.args,[{kind:'const',value:1}]);
 p=represented('LOF_121','context.player.getArenaUnits','context.player.opponent.getArenaUnits');assert.deepEqual(p.arena.receiver.steps.map(s=>s.key),['player','opponent']);
 for(const [to,op,value] of [['remainingHp > 7','>',7],['remainingHp >= 6','>=',6]]) {
  p=represented('LOF_121','remainingHp >= 7',to);const predicate=nodes(p.program,'binary')[0];assert.equal(predicate.op,op);assert.deepEqual(predicate.right,{kind:'const',value});
 }
 p=represented('LOF_121','(card) => card.remainingHp >= 7','(card) => context.source.remainingHp >= 7');
 const read=nodes(p.program,'binary')[0].left;assert.deepEqual(read.root,p.program.parameters);assert.deepEqual(read.steps.map(s=>s.key),['source','remainingHp']);
 p=represented('LOF_121','.filter(','.map(');assert.equal(nodes(p.program,'pureCollectionCall')[0].method,'map');
 p=represented('JTL_199','const exhaustedUnits','let exhaustedUnits');assert.equal(p.program.body.statements[0].bindings[0].mutable,true);
 for(const [to,op,value] of [['return exhaustedUnits.length > 3','>',3],['return exhaustedUnits.length >= 2','>=',2]]) {
  p=represented('JTL_199','return exhaustedUnits.length >= 3',to);assert.partialDeepStrictEqual(p.program.body.statements.at(-1),{kind:'pureReturn',value:{kind:'binary',op,right:{kind:'const',value}}});
 }
 p=represented('JTL_199','card.exhausted === true','card.exhausted === false');assert.deepEqual(nodes(p.program,'pureCallback')[0].body.value.right.right,{kind:'const',value:false});
 p=represented('JTL_199','card.isUnit() && card.exhausted','card.isUnit() || card.exhausted');assert.equal(nodes(p.program,'pureCallback')[0].body.value.op,'||');
 p=represented('JTL_199','card.isUnit()','card.isUpgrade()');assert.equal(nodes(p.program,'pureCallback')[0].body.value.left.method,'isUpgrade');
 p=represented('JTL_199','context.player.opponent.getArenaUnits','context.player.getArenaUnits');assert.deepEqual(p.arena.receiver.steps.map(s=>s.key),['player']);
 p=represented('JTL_199','getArenaUnits()','getArenaUnits({})');assert.deepEqual(p.arena.args,[{kind:'dataRecord',fields:{}}]);
});
test('general helper callbacks retain index and destructured parameter bindings',()=>{
 for(const id of ['LOF_121','JTL_199']) {
  let p=represented(id,'(card) =>','(card,other) =>');let callback=nodes(p.program,'pureCallback')[0];
  assert.deepEqual(callback.parameters.map(p=>p.argument),['item','index']);assert.deepEqual(callback.parameters.map(p=>p.bindingPattern),[{name:'card'},{name:'other'}]);
  p=represented(id,'(card) =>','({remainingHp:card}) =>');callback=nodes(p.program,'pureCallback')[0];
  assert.deepEqual(callback.parameters[0].bindingPattern,{kind:'object',entries:[{key:'remainingHp',binding:{name:'card'}}]});
  const root=callback.parameters[0].slot;assert.ok(nodes(callback,'dataPath').some(p=>p.root.join('.')===root+'.remainingHp'));
 }
});
test('filtered helper callback and method bindings fail closed',()=>{
 for(const[id,name]of[['LOF_121','getDrawAmount'],['JTL_199','opponentHasAtLeastThreeExhausted']]){
  const{text,file}=source(id);for(const prefix of ['public ','private static ','private async ','@sideEffect() private ','private *'])reject(text.replace('private '+name,prefix+name),file);
  for(const replacement of ['async (card) =>','(...card) =>','(card=sideEffect()) =>'])reject(text.replace('(card) =>',replacement),file);
  for(const binding of ['...context','context=sideEffect()','{player:context}','context, other']){const from=id==='LOF_121'?'context: AbilityContext)':'context) {';reject(text.replace(from,binding+(id==='LOF_121'?')':') {')),file);}
  reject(text.replace('this.'+name+'(context)','this.'+name+'?.(context)'),file);reject(text.replace('this.'+name+'(context)','other.'+name+'(context)'),file);reject(text.replace('this.'+name+'(context)','this.'+name+'(context.source)'),file);reject(text.replace('this.'+name+'(context)','this.'+name+'(context, context)'),file);reject(text.replace('this.'+name+'(context)','1'),file);
 }
 const p=source('LOF_121');reject(p.text.replace('draw((context) => ({ amount: this.getDrawAmount(context) }))','draw(function(context){return {amount:this.getDrawAmount(context)};})'),p.file);
 const b=source('JTL_199');reject(b.text.replace('(context) => this.opponentHasAtLeastThreeExhausted(context)','function(context){return this.opponentHasAtLeastThreeExhausted(context);}'),b.file);
});
test('filtered helper result snapshots retain existing direct query stage policy',()=>{
 const{text,file}=source('JTL_199');const from='condition: (context) => this.opponentHasAtLeastThreeExhausted(context),';
 for(const bindings of ['const saved=this.opponentHasAtLeastThreeExhausted(context);','const first=this.opponentHasAtLeastThreeExhausted(context);const saved=first;','const first=this.opponentHasAtLeastThreeExhausted(context);const second=first;const saved=second;']){
  const changed=text.replace('conditional({','conditional(context=>{'+bindings+'return {').replace(from,'condition: later=>saved,').replace('\n            })','\n            };})');const c=compileSource(changed,file,enums);assert.equal(c.status,'unsupported');assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.ok(c.diagnostics.some(d=>d.pattern==='arena-damage-helper-call-scope'),JSON.stringify(c.diagnostics));
 }
 const p=source('LOF_121');const local=p.text.replace('draw((context) => ({ amount: this.getDrawAmount(context) }))','draw(context=>{const saved=this.getDrawAmount(context);return {amount:saved};})');const c=compileSource(local,p.file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.deepEqual(c.abilities,full('LOF_121').abilities);
});
test('singleton exhaust normalization preserves one chosen identity and token event order',()=>{
 const a=full('LOF_054').abilities.card[0];assert.equal(a.targetResolvers,undefined);assert.deepEqual(a.targetResolver,{owner:'self',type:'unit',zone:'anyAttackable'});assert.deepEqual(a.effect,{kind:'exhaust',target:'chosen'});assert.deepEqual(a.ifYouDo.effect,{kind:'simultaneous',effects:[{kind:'giveExperience',target:'chosen',amount:{kind:'const',value:2}},{kind:'giveShield',target:'chosen',amount:{kind:'const',value:1}}]});
 const{text,file}=source('LOF_054');assert.deepEqual(compileSource(text.replaceAll('friendlyUnit','selectedUnit'),file,enums).abilities,full('LOF_054').abilities);
});
test('singleton normalization does not widen chooser, count, flags or continuation topology',()=>{
 const{text,file}=source('LOF_054');const unchanged=changed=>{const c=compileSource(changed,file,enums);if(c.status==='supported'){const a=c.abilities.card[0];assert.ok(a.targetResolvers,'unbounded source was normalized');assert.equal(a.targetResolver,undefined);}};
 for(const[from,to]of [
 ['controller: RelativePlayer.Self','controller: RelativePlayer.Opponent'],['cardTypeFilter: WildcardCardType.Unit','optional:true, cardTypeFilter: WildcardCardType.Unit'],['cardTypeFilter: WildcardCardType.Unit','choosingPlayer:RelativePlayer.Opponent, cardTypeFilter: WildcardCardType.Unit'],['cardTypeFilter: WildcardCardType.Unit','mode:TargetMode.Exactly,numCards:1,cardTypeFilter: WildcardCardType.Unit'],['cardTypeFilter: WildcardCardType.Unit','zoneFilter:ZoneName.GroundArena,cardTypeFilter: WildcardCardType.Unit'],['immediateEffects.exhaust()','immediateEffects.ready()'],['amount: 2','amount: 3'],['ifYouDoContext.targets.friendlyUnit','ifYouDoContext.source'],['ifYouDo: (ifYouDoContext) => ({','ifYouDo: (ifYouDoContext) => ({ optional:true,'],['ifYouDo: (ifYouDoContext) => ({','then: (ifYouDoContext) => ({']
 ]){assert.ok(text.includes(from));unchanged(text.replace(from,to));}
});
