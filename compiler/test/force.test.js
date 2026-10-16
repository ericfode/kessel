'use strict';
const test=require('node:test'),assert=require('node:assert/strict');const {compileSource}=require('../compiler');
const compile=(effect,resolver=false)=>compileSource(`export default class ForceUnit extends NonLeaderUnitCard {getImplementationId(){return{id:'force-unit',internalName:'force-unit'};}setupCardAbilities(registrar,AbilityHelper){registrar.addWhenPlayedAbility({title:'Force',${resolver?`targetResolver:{cardTypeFilter:'unit',immediateEffect:${effect}}`:`immediateEffect:${effect}`}});}}`,'force-unit.ts');
test('Force lowering distinguishes own-player defaults, inherited UseForce targets and explicit constructor properties',()=>{
 const inherited=binding=>({kind:'inheritedSystemTarget',binding,fallback:'nativeUseForceTargets'});
 const use=(target,isCost=false)=>({kind:'useForceSystem',target,nativeDefaultTargets:'contextPlayerOnlyWhenHasTheForce',isCost:{kind:'const',value:isCost},cannotBeCancelled:{kind:'const',value:false},nativeSystem:'UseTheForceSystem',legalTarget:'player',requiresForce:'costOrRequiredStateChange',eventUpdateAssertion:'targetPlayerHasTheForce',event:'onCardLeavesPlay',card:'targetPlayer.baseZone.forceToken',lastKnownInformation:'forceTokenAtEventUpdate',destination:'outsideTheGame',contingentEvent:'onForceUsed'});
 for(const helper of ['useTheForce','theForceIsWithYou']){
  const effect=`AbilityHelper.immediateEffects.${helper}()`;const ordinary=compile(effect);assert.equal(ordinary.status,'supported');assert.deepEqual(ordinary.abilities.card[0].effect,{kind:helper==='useTheForce'?'useForce':'gainForce',target:'ownPlayer'});
  for(const nested of [false,true]){const c=compile(nested?`AbilityHelper.immediateEffects.conditional({condition:context=>true,onTrue:${effect}})`:effect,true);if(helper==='theForceIsWithYou'){assert.equal(c.status,'unsupported');assert.ok(c.diagnostics.some(d=>d.pattern==='force-inherited-target'));}else{assert.equal(c.status,'supported');assert.deepEqual(c.abilities.card[0].targetResolver,{owner:'any',type:'unit',zone:'anyAttackable'});assert.deepEqual(c.abilities.card[0].effect,nested?{kind:'conditional',condition:{kind:'const',value:true},onTrue:use(inherited('chosen')),onFalse:{kind:'noAction'}}:use(inherited('chosen')));}}
  const explicit=compile(`AbilityHelper.immediateEffects.${helper}({target:context=>context.player})`);assert.equal(explicit.status,'supported');
  if(helper==='useTheForce')assert.deepEqual(explicit.abilities.card[0].effect,use({kind:'ref',path:['player']}));
  else {const e=explicit.abilities.card[0].effect;assert.equal(e.kind,'nativeForceTokenSystem');assert.equal(e.target,'ownPlayer');assert.equal(e.targetOverride,'explicitConstructorProperty');assert.equal(e.constructor,'CreateForceTokenSystem');assert.equal(e.tokenType,'the-force');assert.equal(e.eventName,'onTokensCreated');assert.equal(e.eventGeneration.update.append,'existingForceTokenIdentity');assert.equal(e.executionCapability,'pendingNativeForceTokenSystem');}
  const cost=compile(`AbilityHelper.immediateEffects.${helper}({isCost:true})`);if(helper==='useTheForce'){assert.equal(cost.status,'supported');assert.deepEqual(cost.abilities.card[0].effect,use(inherited(null),true));}else{assert.equal(cost.status,'unsupported');assert.ok(cost.diagnostics.some(d=>d.pattern==='unsupported-field:isCost'));}
  const malformed=compile(`AbilityHelper.immediateEffects.${helper}({},{})`);assert.equal(malformed.status,'unsupported');assert.ok(malformed.diagnostics.some(d=>d.pattern==='effect-helper-arity'));
 }
});
const fs=require('node:fs'),path=require('node:path');const {loadEnums}=require('../compiler');
test('the three pinned source registrations retain use success and gain defaults',()=>{
 const root=path.resolve(__dirname,'../../forceteki/server'),enums=loadEnums(root);
 function find(dir,fragment){for(const e of fs.readdirSync(dir,{withFileTypes:true})){const file=path.join(dir,e.name);if(e.isDirectory()){const hit=find(file,fragment);if(hit)return hit;}else if(e.name.endsWith('.ts')&&fs.readFileSync(file,'utf8').includes(fragment))return file;}}
 for(const [fragment,kind,optional] of [['kiadimundi#we-must-push-on','useForce',true],['vernestra-rwoh#precocious-knight','useForce',true],['youngling-padawan','gainForce',false]]){
  const file=find(path.join(root,'game/cards'),fragment);assert.ok(file,fragment);const c=compileSource(fs.readFileSync(file,'utf8'),file,enums);assert.equal(c.status,'supported');const a=c.abilities.card[0];assert.equal(a.effect.kind,kind);assert.equal(a.effect.target,'ownPlayer');assert.equal(!!a.optional,optional);if(kind==='useForce')assert.ok(a.ifYouDo);
 }
});
