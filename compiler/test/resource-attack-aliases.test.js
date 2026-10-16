'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root),files={SOR_226:'01_SOR/units/AdmiralMottiBrazenAndScornful.ts',JTL_054:'04_JTL/units/GoldLeaderFastestShipInTheFleet.ts',LAW_108:'07_LAW/units/LandoCalrissianEyesOpen.ts'};
const ref=(...path)=>({kind:'ref',path}),query=(name,receiver='card',args=[])=>({kind:'query',name,receiver:ref(receiver),args}),binary=(left,right)=>({kind:'binary',op:'&&',left,right});
function source(id){const file='game/cards/'+files[id];return{file,text:fs.readFileSync(path.join(root,file),'utf8')};}
function full(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function rejected(text,file,pattern) {const c=compileSource(text,file,enums);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));if(c.status==='supported')return; assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));}
test('Motti keeps optional defeat, any owner arena and Unit guard, lowers same-getter one-aspect membership',()=>{const a=full('SOR_226').abilities.card[0];assert.equal(a.trigger,'whenDefeated');assert.equal(a.optional,true);assert.deepEqual(a.targetResolver,{owner:'any',type:'any',zone:'arena',predicate:binary(query('isUnit'),query('hasSomeAspect','card',[{kind:'const',value:'villainy'}]))});assert.deepEqual(a.effect,{kind:'ready',target:'chosen'});});
test('Motti aspect alias rejects enum formal shadows and unsupported membership arguments',()=>{
 const{text,file}=source('SOR_226');
 for(const ns of ['Aspect','WildcardZoneName','TextHelper'])rejected(text.replaceAll('registrar',ns),file,'syntax-alias-enum-binding');
 rejected(text.replace('(card) => card.isUnit() && card.aspects.includes(Aspect.Villainy)','(Aspect) => Aspect.isUnit() && Aspect.aspects.includes(Aspect.Villainy)'),file,'syntax-alias-enum-binding');
 for(const[from,to]of [['includes(Aspect.Villainy)','includes(Aspect.Villainy,1)'],['includes(Aspect.Villainy)','includes(context.source)']]){rejected(text.replace(from,to),file);}
});
test('both aura full sources preserve exact candidate guard, source defense condition and persistent minus one',()=>{
 for(const id of ['JTL_054','LAW_108']){const c=full(id),a=c.abilities.card[0];assert.equal(c.baseClass,'NonLeaderUnitCard');assert.deepEqual(c.abilities.leader,[]);assert.deepEqual(c.abilities.unit,[]);assert.equal(c.abilities.card.length,1);assert.deepEqual(a.condition,query('isDefending','source'));assert.deepEqual(a.targetResolver,{owner:'opponent',type:'unit',zone:'arena',predicate:binary(binary(query('isUnit'),query('isInPlay')),query('isAttacking'))});assert.deepEqual(a.effect,{kind:'modifyStats',target:'chosen',duration:'persistent',power:{kind:'const',value:-1},hp:{kind:'const',value:0}});}
});
test('aura rejects altered identity, short circuit, target controller, guards, options and modifiers',()=>{
 for(const id of ['JTL_054','LAW_108']){const{text,file}=source(id);for(const[from,to]of [
 ['includes(context.source)','includes(context.target)'],['includes(context.source)','includes(card)'],['includes(context.source)','includes(context.source,1)'],['getAllTargets()','getAllTargets(1)'],['getAllTargets()','getAllTargets?.()'],['card.activeAttack','context.source.activeAttack'],['card.isAttacking() &&','card.isAttacking() ||'],['card.isInPlay() && ',''],['RelativePlayer.Opponent','RelativePlayer.Self'],['power: -1','power: -2'],['hp: 0','hp: 1'],['matchTarget: (card, context) =>','matchTarget: async (card, context) =>'],['matchTarget: (card, context) =>','matchTarget: (card, context=sideEffect()) =>'],['matchTarget: (card, context) =>','matchTarget: (card, ...context) =>'],["targetController: RelativePlayer.Opponent,","targetController: RelativePlayer.Opponent, condition:context=>true,"]
 ]){assert.ok(text.includes(from),from);rejected(text.replace(from,to),file);}
 rejected(text.replaceAll('registrar','RelativePlayer'),file,'syntax-alias-enum-binding');
 }
});
test('aura alpha formals preserve binding while source and candidate aliases are conservatively rejected',()=>{
 for(const id of ['JTL_054','LAW_108']){const{text,file}=source(id);assert.deepEqual(compileSource(text.replaceAll('card, context','candidate, actual').replaceAll('card.','candidate.').replaceAll('context.source','actual.source'),file,enums).abilities,full(id).abilities);
 rejected(text.replace('card.activeAttack.getAllTargets().includes(context.source)','card.activeAttack.getAllTargets().includes(saved.source)').replace('matchTarget: (card, context) =>','matchTarget: (card, context) => {const saved=context;return ').replace('includes(saved.source),','includes(saved.source);},'),file);
 }
});

test('Motti rejects callable-valued predicate operands rather than erasing their function identity',()=>{
 const{text,file}=source('SOR_226');for(const expression of ['(() => card.aspects.includes(Aspect.Villainy))','(true ? (() => card.aspects.includes(Aspect.Villainy)) : (() => false))'])rejected(text.replace('card.aspects.includes(Aspect.Villainy)',expression),file,'aspect-membership-predicate');
});

test('Motti readonly predicate aliases keep callback role and reject callable-result aliases',()=>{
 const{text,file}=source('SOR_226'),original='(card) => card.isUnit() && card.aspects.includes(Aspect.Villainy)';
 for(const callback of [original,'(card) => card.isUnit() && (() => card.aspects.includes(Aspect.Villainy))']){const changed=text.replace('registrar.addWhenDefeatedAbility({','const first='+callback+';const saved=first;registrar.addWhenDefeatedAbility({').replace('cardCondition: '+original,'cardCondition:saved');if(callback===original)assert.deepEqual(compileSource(changed,file,enums).abilities,full('SOR_226').abilities);else rejected(changed,file,'aspect-membership-predicate');}
});
