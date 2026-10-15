'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root),file='game/cards/01_SOR/upgrades/Electrostaff.ts',source=fs.readFileSync(path.join(root,file),'utf8');
const ref=(...path)=>({kind:'ref',path}),con=value=>({kind:'const',value}),query=name=>({kind:'query',name,receiver:ref('card'),args:[]}),binary=(left,right)=>({kind:'binary',op:'&&',left,right});
function compile(text=source){return compileSource(text,file,enums);}
function reject(text,pattern){const c=compile(text);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));return c;}
test('full Electrostaff keeps upgrade source, parent defense identity, opponent scope and printed attach predicate',()=>{const c=compile();assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));assert.equal(c.baseClass,'UpgradeCard');assert.deepEqual(c.attachCondition,{kind:'unary',op:'!',value:{kind:'query',name:'hasSomeTrait',receiver:ref('card'),args:[con('vehicle')]}});assert.deepEqual(c.abilities,{leader:[],unit:[],card:[{trigger:'constant',title:'While attached unit is defending, the attacker gets -1/-0',costs:[],effect:{kind:'modifyStats',target:'chosen',duration:'persistent',power:con(-1),hp:con(0)},condition:{kind:'query',name:'isDefending',receiver:ref('source','parentUnit'),args:[]},targetResolver:{owner:'opponent',type:'unit',zone:'arena',predicate:binary(binary(query('isUnit'),query('isInPlay')),query('isAttacking'))}}]});assert.equal(c.abilities.card[0].attachedContext,undefined);});
test('alpha callback formals preserve upgrade and candidate roles; ordinary synchronous callback is equivalent',()=>{
 const alpha=source.replace('(card, context) =>','(candidate, actual) =>').replaceAll('card.is','candidate.is').replaceAll('card.activeAttack','candidate.activeAttack').replace('context.source.parentUnit','actual.source.parentUnit');assert.deepEqual(compile(alpha).abilities,compile().abilities);
 const normal=source.replace('(card, context) => card.isUnit() && card.isInPlay() && card.isAttacking() && card.activeAttack.getAllTargets().includes(context.source.parentUnit)','function(card,context){return card.isUnit() && card.isInPlay() && card.isAttacking() && card.activeAttack.getAllTargets().includes(context.source.parentUnit);}');assert.deepEqual(compile(normal).abilities,compile().abilities);
});
test('parent defense rejects wrong identity, missing guards, optional calls, short circuit and binders',()=>{
 for(const[from,to]of [
 ['context.source.parentUnit','context.source.parentCard'],['context.source.parentUnit','context.target'],['context.source.parentUnit','card'],['context.source.parentUnit','context.source.parentUnit.parentUnit'],['context.source.parentUnit','context.source?.parentUnit'],['card.activeAttack','context.source.parentUnit.activeAttack'],['getAllTargets()','getAllTargets(1)'],['getAllTargets()','getAllTargets?.()'],['includes(context.source.parentUnit)','includes(context.source.parentUnit,1)'],['card.isInPlay() && ',''],['card.isAttacking() &&','card.isAttacking() ||'],['(card, context) =>','async (card, context) =>'],['(card, context) =>','(card, ...context) =>'],['(card, context) =>','(card, context=sideEffect()) =>'],['(card, context) =>','(card, card) =>'],['(card, context) =>','({card}, context) =>']
 ]){assert.ok(source.includes(from));reject(source.replace(from,to));}
});
test('whole source fence rejects changed class, attachment restriction, controller, sides and headers',()=>{
 for(const[from,to]of [
 ['extends UpgradeCard','extends NonLeaderUnitCard'],['extends UpgradeCard','extends TokenUpgradeCard'],['Trait.Vehicle','Trait.Jedi'],['!context.attachTarget.hasSomeTrait','context.attachTarget.hasSomeTrait'],['RelativePlayer.Opponent','RelativePlayer.Self'],['power: -1','power: -2'],['hp: 0','hp: 1'],['targetController: RelativePlayer.Opponent,','targetController: RelativePlayer.Opponent, condition:context=>true,'],['targetController: RelativePlayer.Opponent,','targetController: RelativePlayer.Opponent, targetZoneFilter:"groundArena",'],['registrar.addConstantAbility({','registrar.addWhenPlayedAbility({title:"Extra",immediateEffect:AbilityHelper.immediateEffects.draw()});registrar.addConstantAbility({'],['setupCardAbilities','setupLeaderSideAbilities']
 ]){assert.ok(source.includes(from));reject(source.replace(from,to));}
 const removed=source.replace('registrar.setAttachCondition((context) => !context.attachTarget.hasSomeTrait(Trait.Vehicle));','');reject(removed,'parent-defense-source-shape');
});
test('namespace and helper binding fences reject setup/formal aliases without trusting enum spelling',()=>{
 reject(source.replaceAll('registrar','Trait'),'top-deck-resource-enum-binding');
 reject(source.replaceAll('registrar','RelativePlayer'),'syntax-alias-enum-binding');
 reject(source.replace('AbilityHelper: IAbilityHelper)','AbilityHelper: IAbilityHelper, Trait:any)'),'parent-defense-setup-binding');
 reject(source.replace('AbilityHelper: IAbilityHelper','OtherHelper: IAbilityHelper'),'parent-defense-helper-binding');
 reject(source.replaceAll('registrar','AbilityHelper'),'parent-defense-setup-binding');
 const callback=source.replace('registrar.setAttachCondition((context) => !context.attachTarget.hasSomeTrait(Trait.Vehicle));','registrar.setAttachCondition((Trait) => !Trait.attachTarget.hasSomeTrait(Trait.Vehicle));');reject(callback,'top-deck-resource-enum-binding');
});
test('captured parent and callback composites reject without lowering a stale identity',()=>{
 const predicate='(card, context) => card.isUnit() && card.isInPlay() && card.isAttacking() && card.activeAttack.getAllTargets().includes(context.source.parentUnit)';
 for(const value of ['saved','(true?'+predicate+':'+predicate+')','(() => ('+predicate+'))']){let text=source;if(value==='saved')text=text.replace('registrar.addConstantAbility({','const saved='+predicate+';registrar.addConstantAbility({');reject(text.replace(predicate+',',value+','));}
 const captured=source.replace(predicate,'(card, context)=>{const host=context.source.parentUnit;return card.isUnit()&&card.isInPlay()&&card.isAttacking()&&card.activeAttack.getAllTargets().includes(host);}');reject(captured);
 const alias=source.replace('registrar.addConstantAbility({','const effect=AbilityHelper.ongoingEffects.modifyStats({power:-1,hp:0});registrar.addConstantAbility({').replace('ongoingEffect: AbilityHelper.ongoingEffects.modifyStats({ power: -1, hp: 0 })','ongoingEffect:effect');reject(alias,'parent-defense-helper-binding');
 for(const stats of ['{power:()=>-1,hp:0}','ctx=>({power:-1,hp:0})'])reject(source.replace('{ power: -1, hp: 0 }',stats));
});
test('damage replacement and optional parent-title siblings remain rejected as complete sources',()=>{
 for(const p of ['game/cards/08_ASH/upgrades/DeadlyVulnerability.ts','game/cards/09_HMW/upgrades/ChewbaccasBowcasterHandcraftedTradition.ts']){const c=compileSource(fs.readFileSync(path.join(root,p),'utf8'),p,enums);assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
});
