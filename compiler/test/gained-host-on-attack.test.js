'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={LOF_051:'05_LOF/upgrades/JediHolocron.ts',LAW_186:'07_LAW/upgrades/EnfysNestsHelmet.ts',LOF_139:'05_LOF/upgrades/BattleFury.ts'};
const ref=(...path)=>({kind:'ref',path}),con=value=>({kind:'const',value}),other={kind:'binary',op:'!==',left:ref('card'),right:ref('source')};
function source(id){const file='game/cards/'+files[id];return{file,text:fs.readFileSync(path.join(root,file),'utf8')};}
function compile(id,text=source(id).text){return compileSource(text,source(id).file,enums);}
function full(id){const c=compile(id);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(id,text){const c=compile(id,text);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));}
function mutations(id,pairs){const{text}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);reject(id,text.replace(from,to));}}
const attach=trait=>({kind:'query',name:'hasSomeTrait',receiver:ref('card'),args:[con(trait)]});
test('three full sources emit explicit gained-host templates, never delegated constant context',()=>{
 for(const id of Object.keys(files)){const c=full(id);assert.equal(c.baseClass,'UpgradeCard');assert.deepEqual(c.abilities.leader,[]);assert.deepEqual(c.abilities.unit,[]);assert.equal(c.abilities.card.length,1);assert.equal(c.abilities.card[0].trigger,'onAttack');assert.equal(c.abilities.card[0].gainedHost,true);assert.equal(c.abilities.card[0].attachedContext,undefined);assert.deepEqual(c.abilities.card[0].costs,[]);}
 assert.deepEqual(full('LOF_051').abilities.card[0],{trigger:'onAttack',title:'You may heal 3 damage from another unit.',costs:[],optional:true,targetResolver:{owner:'any',type:'unit',zone:'anyAttackable',predicate:other},effect:{kind:'heal',target:'chosen',amount:con(3)},gainedHost:true});assert.deepEqual(full('LOF_051').attachCondition,attach('force'));
 assert.deepEqual(full('LAW_186').abilities.card[0],{trigger:'onAttack',title:'Give another unit +3/+0 for this phase',costs:[],optional:true,targetResolver:{owner:'any',type:'any',zone:'anyAttackable',predicate:{kind:'binary',op:'&&',left:other,right:{kind:'query',name:'isUnit',receiver:ref('card'),args:[]}}},effect:{kind:'modifyStats',target:'chosen',duration:'phase',power:con(3),hp:con(0)},gainedHost:true});assert.deepEqual(full('LAW_186').attachCondition,{kind:'unary',op:'!',value:attach('vehicle')});
 assert.deepEqual(full('LOF_139').abilities.card[0],{trigger:'onAttack',title:'Discard a card from your hand',costs:[],effect:{kind:'discardOwnHand',amount:con(1),target:'ownPlayer'},gainedHost:true});assert.equal(full('LOF_139').attachCondition,undefined);
});
test('alpha actual context, registrar and allowed helper names preserve template binding',()=>{
 for(const id of Object.keys(files)){const{text}=source(id);const alpha=text.replaceAll('context','actual').replaceAll('card, actual','candidate, actual').replaceAll('card !==','candidate !==').replaceAll('card.isUnit()','candidate.isUnit()').replaceAll('registrar','registrations');assert.deepEqual(compile(id,alpha).abilities,full(id).abilities);const helper=text.replace(' AbilityHelper: IAbilityHelper',' StaticAbilityHelper: IAbilityHelper').replace(' abilityHelper: IAbilityHelper',' StaticAbilityHelper: IAbilityHelper').replaceAll('AbilityHelper.immediateEffects','StaticAbilityHelper.immediateEffects').replaceAll('AbilityHelper.ongoingEffects','StaticAbilityHelper.ongoingEffects').replaceAll('abilityHelper.immediateEffects','StaticAbilityHelper.immediateEffects').replaceAll('abilityHelper.ongoingEffects','StaticAbilityHelper.ongoingEffects');assert.deepEqual(compile(id,helper).abilities,full(id).abilities);}
});
test('shared registrar boundary rejects extra semantic headers, side registrations, wrong classes and protocol titles',()=>{
 for(const id of Object.keys(files)){
 mutations(id,[['extends UpgradeCard','extends NonLeaderUnitCard'],['extends UpgradeCard','extends TokenUpgradeCard'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({gainCondition:context=>true,'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({condition:context=>true,'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({cost:[],'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({gainedHost:true,'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({attachedContext:"host",'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({when:{onAttack:true},'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({then:{immediateEffect:AbilityHelper.immediateEffects.draw()},'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({ifYouDo:{immediateEffect:AbilityHelper.immediateEffects.draw()},'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({contextTitle:context=>context.source.title,'],['setupCardAbilities','setupLeaderSideAbilities']]);
 const{text}=source(id),title=/title: '[^\n]+'/.exec(text)[0];reject(id,text.replace(title,'title: context=>"Dynamic"'));reject(id,text.replace(title,'title: `Dynamic ${sideEffect()}`'));
 const helper=id==='LAW_186'?'abilityHelper':'AbilityHelper';reject(id,text.replace('registrar.addGainOnAttackAbilityTargetingAttached({','registrar.addWhenPlayedAbility({title:"Extra",immediateEffect:'+helper+'.immediateEffects.draw()});registrar.addGainOnAttackAbilityTargetingAttached({'));
 }
});
test('actual host exclusion and attach restrictions reject wrong identities, types and callback values',()=>{
 mutations('LOF_051',[['Trait.Force','Trait.Jedi'],['WildcardCardType.Unit','WildcardCardType.NonLeaderUnit'],['card !== context.source','card !== context.target'],['card !== context.source','card !== context.source.parentUnit'],['card !== context.source','card === context.source'],['card !== context.source','(()=>card !== context.source)'],['card !== context.source','true?(()=>card !== context.source):(()=>false)'],['amount: 3','amount: 2'],['amount: 3','amount: ()=>3'],['optional: true','optional: false'],['(card, context) =>','(card, ...context) =>'],['(card, context) =>','async (card, context) =>']]);
 mutations('LAW_186',[['Trait.Vehicle','Trait.Force'],['!context.attachTarget','context.attachTarget'],['card !== context.source && card.isUnit()','card !== context.source || card.isUnit()'],['card !== context.source && card.isUnit()','card !== context.source && (()=>card.isUnit())'],['card !== context.source','card !== context.target'],['card.isUnit()','card.isUnit(1)'],['power: 3','power: ()=>3'],['power: 3','power: 2'],['hp: 0','hp: 1'],['forThisPhaseCardEffect','forThisAttackCardEffect'],['optional: true','optional: false']]);
});
test('mandatory own-hand discard preserves explicit host player and rejects callback/target ambiguity',()=>{
 mutations('LOF_139',[['amount: 1','amount: 2'],['amount: 1','amount: ()=>1'],['target: context.player','target: context.player.opponent'],['target: context.player','target: context.source'],['target: context.player','target: context.target'],['(context) =>','async (context) =>'],['(context) =>','(...context) =>'],['(context) =>','(context=sideEffect()) =>'],['(context) =>','({player:context}) =>'],['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached({optional:true,'],['amount: 1,','amount: 1, choosingPlayer:"opponent",']]);
 const{text}=source('LOF_139');reject('LOF_139',text.replace('amount: 1,\n                target: context.player','amount:1'));reject('LOF_139',text.replace('({\n                amount: 1,\n                target: context.player\n            })','{const player=context.player;return {amount:1,target:player};}'));
});
test('setup/import namespaces and helper provenance remain actual lexical bindings',()=>{
 for(const id of Object.keys(files)){const{text}=source(id),helper=id==='LAW_186'?'abilityHelper':'AbilityHelper';reject(id,text.replaceAll('registrar',helper));reject(id,text.replace(helper+': IAbilityHelper','WrongHelper: IAbilityHelper'));reject(id,text.replace(helper+': IAbilityHelper)',helper+': IAbilityHelper, extra:any)'));}
 for(const id of ['LOF_051','LAW_186']){const{text}=source(id);reject(id,text.replaceAll('registrar','Trait'));reject(id,text.replace('(context) => context.attachTarget','(Trait) => Trait.attachTarget').replace('(context) => !context.attachTarget','(Trait) => !Trait.attachTarget'));}
 reject('LOF_051',source('LOF_051').text.replaceAll('registrar','WildcardCardType'));
});
test('factories, readonly aliases and captured query objects cannot erase callback result or snapshot stages',()=>{
 for(const id of Object.keys(files)){const{text}=source(id);reject(id,text.replace('addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached(context=>({').replace(/\n        \}\);\n    \}/,'\n        }));\n    }'));
 const start=text.indexOf('registrar.addGainOnAttackAbilityTargetingAttached('),argStart=start+'registrar.addGainOnAttackAbilityTargetingAttached('.length,end=text.indexOf('\n        });',argStart);assert.ok(start>=0&&end>=0);const arg=text.slice(argStart,end+10);reject(id,text.slice(0,start)+'const saved='+arg+';\n        registrar.addGainOnAttackAbilityTargetingAttached(saved);'+text.slice(end+12));}
 const jedi=source('LOF_051').text;reject('LOF_051',jedi.replace('(card, context) => card !== context.source','(card, context)=>{const source=context.source;return card!==source;}'));
});
test('noncohort gained-attack sources recover through piloting; remaining sources stay rejected',()=>{
 const selected=new Set(Object.values(files).map(f=>'game/cards/'+f));let admitted=0,deferred=0;
 for(const relative of fs.readdirSync(path.join(root,'game/cards'),{recursive:true}).filter(f=>f.endsWith('.ts'))){const file='game/cards/'+relative,text=fs.readFileSync(path.join(root,file),'utf8');if(!text.includes('addGainOnAttackAbilityTargetingAttached')||selected.has(file))continue;const c=compileSource(text,file,enums);if(c.status==='supported'){assert.equal(c.abilities.card[0]?.effect?.kind,'attachedAbilityRegistration',file);admitted++;}else{assert.equal(c.status,'unsupported',file+JSON.stringify(c.diagnostics));deferred++;}}assert.equal(admitted,16);assert.equal(deferred,4);
});

test('new grant templates require instance setup and pure literal identity methods',()=>{
 for(const id of Object.keys(files)){mutations(id,[['public override setupCardAbilities','public static override setupCardAbilities'],['protected override getImplementationId','protected static override getImplementationId'],['getImplementationId() {','getImplementationId() {sideEffect();'],['getImplementationId () {','getImplementationId () {sideEffect();']].filter(([from])=>source(id).text.includes(from)));}
});

test('identity function values cannot be erased into literal implementation metadata',()=>{
 for(const id of Object.keys(files)){const{text}=source(id),returned=text.indexOf('return {'),end=text.indexOf('};',returned);assert.ok(returned>=0&&end>=0);reject(id,text.slice(0,returned)+text.slice(returned,end).replace('return {','return ()=>({')+'});'+text.slice(end+2));}
});

test('imported runtime namespaces and whole module statements cannot be substituted',()=>{
 for(const id of Object.keys(files)){const{text}=source(id);reject(id,'sideEffect();\n'+text);reject(id,text.replace("import { UpgradeCard }","import { EventCard as UpgradeCard }"));reject(id,text.replace("import { UpgradeCard }","import type { UpgradeCard }"));}
 mutations('LOF_051',[['{ Trait, WildcardCardType }','{ Aspect as Trait, WildcardCardType }'],['{ Trait, WildcardCardType }','{ Trait, CardType as WildcardCardType }']]);mutations('LAW_186',[['import { Trait }','import { Aspect as Trait }']]);
});

test('new registrar calls and attach setup require nonoptional direct invocation',()=>{
 for(const id of Object.keys(files))mutations(id,[['addGainOnAttackAbilityTargetingAttached({','addGainOnAttackAbilityTargetingAttached?.({']]);for(const id of ['LOF_051','LAW_186'])mutations(id,[['setAttachCondition(','setAttachCondition?.(']]);
});
test('duplicate or fake runtime namespace imports and side-effect imports reject',()=>{
 for(const id of Object.keys(files)){const{text}=source(id);reject(id,"import 'side-effect-module';\n"+text);reject(id,text.replace('import { UpgradeCard }','import { UpgradeCard, EventCard as UpgradeCard }'));}
 mutations('LOF_051',[["from '../../../core/Constants'","from 'fake/core/Constants'"],['{ Trait, WildcardCardType }','{ Trait, Aspect as Trait, WildcardCardType }']]);
});

test('new template source must export its concrete card class as the engine default',()=>{
 for(const id of Object.keys(files))mutations(id,[['export default class','export class'],['export default class','class'],['export default class','export declare class']]);
});

test('all identity member kinds and either declaration order reject duplicate identity slots',()=>{
 for(const id of Object.keys(files)){const{text}=source(id),identityStart=text.indexOf('    protected override getImplementationId'),setupStart=text.indexOf('    public override setupCardAbilities'),identity=text.slice(identityStart,setupStart).trim();assert.ok(identityStart>=0&&setupStart>=0);
 const variants=[identity,identity.replace('return {','sideEffect();return {'),'get getImplementationId(){sideEffect();return ()=>({id:"x",internalName:"x"});}','set getImplementationId(value){sideEffect();}','getImplementationId=()=>{sideEffect();return {id:"x",internalName:"x"};};',identity.replace('getImplementationId',"['getImplementationId']"),'["getImplementationId"]=()=>({id:"x",internalName:"x"});'];
 for(const extra of variants)for(const insertion of [identityStart,setupStart])reject(id,text.slice(0,insertion)+'    '+extra+'\n'+text.slice(insertion));
 const firstImpure=text.slice(0,identityStart)+text.slice(identityStart,setupStart).replace('return {','sideEffect();return {')+identity+'\n'+text.slice(setupStart);reject(id,firstImpure);
 }
});
test('setup slot overrides reject methods, accessors, properties and computed names gracefully',()=>{
 for(const id of Object.keys(files)){const{text}=source(id),start=text.indexOf('    public override setupCardAbilities');assert.ok(start>=0);
 for(const extra of ['setupCardAbilities(){}','setupCardAbilities(){sideEffect();}','get setupCardAbilities(){sideEffect();return ()=>{};}','set setupCardAbilities(value){sideEffect();}','setupCardAbilities=()=>{};','["setupCardAbilities"](){}','["setupCardAbilities"]=()=>{};']){reject(id,text.slice(0,start)+extra+'\n'+text.slice(start));const close=text.lastIndexOf('}');reject(id,text.slice(0,close)+extra+'\n'+text.slice(close));}
 for(const name of ['setupLeaderSideAbilities','setupUnitSideAbilities'])for(const extra of [name+'=()=>{};',name+'(){}'])reject(id,text.slice(0,start)+extra+'\n'+text.slice(start));
 }
});

test('static element-access registrar aliases retain the same class-slot fences',()=>{
 for(const id of Object.keys(files)){const{text}=source(id),computed=text.replace('registrar.addGainOnAttackAbilityTargetingAttached',"registrar['addGainOnAttackAbilityTargetingAttached']");assert.deepEqual(compile(id,computed).abilities,full(id).abilities);
 const close=computed.lastIndexOf('}');for(const member of ['setupCardAbilities(){}','getImplementationId(){sideEffect();return {id:"x",internalName:"x"};}','get getImplementationId(){return ()=>({id:"x",internalName:"x"});}','setupCardAbilities=()=>{};'])reject(id,computed.slice(0,close)+member+'\n'+computed.slice(close));
 }
});
