'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const{compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server')),enums=loadEnums(root);
const files={LOF_235:'05_LOF/units/HK87AssassinDroid.ts',SOR_039:'01_SOR/units/AT-ATSuppressor.ts',TWI_173:'03_TWI/events/BloodSport.ts'};
function source(id){const file='game/cards/'+files[id];return{text:fs.readFileSync(path.join(root,file),'utf8'),file};}
function full(id){const{text,file}=source(id),c=compileSource(text,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));return c;}
function reject(text,file,pattern){const c=compileSource(text,file,enums);assert.ok(!c.diagnostics.some(d=>d.pattern==='typescript-parse-error'),JSON.stringify(c.diagnostics));assert.equal(c.status,'unsupported',JSON.stringify(c.abilities));if(pattern)assert.ok(c.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(c.diagnostics));}
function mutations(id,pairs){const{text,file}=source(id);for(const[from,to]of pairs){assert.ok(text.includes(from),from);reject(text.replace(from,to),file);}}
const ground={kind:'binary',op:'&&',left:{kind:'const',value:true},right:{kind:'binary',op:'||',left:{kind:'unary',op:'!',value:{kind:'const',value:'groundArena'}},right:{kind:'binary',op:'||',left:{kind:'binary',op:'===',left:{kind:'const',value:'groundArena'},right:{kind:'const',value:'anyArena'}},right:{kind:'binary',op:'===',left:{kind:'ref',path:['card','zoneName']},right:{kind:'const',value:'groundArena'}}}}};
test('three complete sources retain one operation and exact controller partition order',()=>{
 for(const[id,trigger,kind,controllerOrder]of[['LOF_235','whenDefeated','damage','playerFirst'],['SOR_039','whenPlayed','exhaust','opponentFirst'],['TWI_173','event','damage','seatOrder']]){
  const c=full(id),a=c.abilities.card[0];assert.equal(c.abilities.card.length,1);assert.deepEqual(c.abilities.unit,[]);assert.deepEqual(c.abilities.leader,[]);assert.equal(a.trigger,trigger);assert.deepEqual(a.costs,[]);
  const effect={kind,target:{kind:'arenaList',receiver:{kind:'ref',path:['game']},predicate:ground,controllerOrder}};if(kind==='damage')effect.amount={kind:'const',value:2};assert.deepEqual(a.effect,effect);assert.equal(a.then,undefined);assert.equal(a.ifYouDo,undefined);
 }
});
test('same-stage readonly list aliases and callback alpha renames preserve exact order',()=>{
 const{text,file}=source('SOR_039');for(const changed of [text.replaceAll('opponentGroundUnits','enemy').replaceAll('friendlyGroundUnits','friendly'),text.replace('return { target: opponentGroundUnits.concat(friendlyGroundUnits) };','const alias=opponentGroundUnits;const second=alias;return { target: second.concat(friendlyGroundUnits) };')])assert.deepEqual(compileSource(changed,file,enums).abilities,full('SOR_039').abilities);
 const blood=source('TWI_173');assert.deepEqual(compileSource(blood.text.replaceAll('(units, player)','(acc, actor)').replace('units.concat(player.getArenaUnits','acc.concat(actor.getArenaUnits'),blood.file,enums).abilities,full('TWI_173').abilities);
 const hk=source('LOF_235');const reverse=hk.text.replace('context.player.getArenaUnits({ arena: ZoneName.GroundArena }).concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))','context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }).concat(context.player.getArenaUnits({ arena: ZoneName.GroundArena }))');assert.equal(compileSource(reverse,hk.file,enums).abilities.card[0].effect.target.controllerOrder,'opponentFirst');
});
test('concat grammar rejects overlap, altered filters, arbitrary collections and optional paths',()=>{
 mutations('LOF_235',[
 ['context.player.opponent.getArenaUnits','context.player.getArenaUnits'],['.concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))','.concat()'],['.concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))','.concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }), context.player.getArenaUnits({ arena: ZoneName.GroundArena }))'],['.concat(','.concat?.('],['getArenaUnits({ arena: ZoneName.GroundArena })','getArenaUnits?.({ arena: ZoneName.GroundArena })'],['ZoneName.GroundArena','ZoneName.SpaceArena'],['arena: ZoneName.GroundArena','arena: ZoneName.GroundArena, condition:card=>true'],['arena: ZoneName.GroundArena','arena: ZoneName.GroundArena, controller:context.player'],['getArenaUnits({ arena: ZoneName.GroundArena })','getArenaUnits()'],['getArenaUnits({ arena: ZoneName.GroundArena })','getArenaUpgrades({ arena: ZoneName.GroundArena })'],['.concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))','.concat(...context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))']
 ]);
});
test('seat-order reducer accepts only the exact finite player concatenation grammar',()=>{
 mutations('TWI_173',[
 ['getPlayers()','getPlayers(1)'],['getPlayers()','getPlayers?.()'],['getPlayers()','getPlayers().reverse()'],['.reduce(','.reduceRight('],['(units, player) =>','async (units, player) =>'],['(units, player) =>','(units, ...player) =>'],['(units, player) =>','(units, player=sideEffect()) =>'],['(units, player) =>','(units, player, index) =>'],['(units, player) =>','(units, units) =>'],['units.concat(player.getArenaUnits','units.concat(context.player.getArenaUnits'],['units.concat(player.getArenaUnits','other.concat(player.getArenaUnits'],['units.concat(player.getArenaUnits','units.concat?.(player.getArenaUnits'],[')), [])',')), [context.source])'],[')), [])',')), {})'],['arena: ZoneName.GroundArena','arena: ZoneName.GroundArena, otherThan:context.source'],['ZoneName.GroundArena','ZoneName.SpaceArena']
 ]);
 const{text,file}=source('TWI_173');const body='units.concat(player.getArenaUnits({ arena: ZoneName.GroundArena }))';reject(text.replace('(units, player) => '+body,'function(units,player){return '+body+';}'),file);reject(text.replace('(units, player) => '+body,'(units,player)=>{return '+body+';}'),file);
});
function hkBlock(text,declarations){return text.replace('damage((context) => ({','damage((context) => {'+declarations+'return ({').replace('\n            }))','\n            }); })');}
test('GroundArena namespace shadow and readonly arena aliases cannot alter arena membership',()=>{
 const{text,file}=source('LOF_235');reject(text.replaceAll('context','ZoneName'),file,'ordered-arena-shadowed-enum');
 for(const declarations of ['const saved=ZoneName.GroundArena;','const first=ZoneName.GroundArena;const saved=first;'])reject(hkBlock(text,declarations).replaceAll('arena: ZoneName.GroundArena','arena: saved'),file,'ordered-arena-static-filter');
 const shadowed=hkBlock(text,'const saved=ZoneName.GroundArena;').replaceAll('arena: ZoneName.GroundArena','arena: saved').replaceAll('context','ZoneName');reject(shadowed,file,'ordered-arena-static-filter');
 const blood=source('TWI_173');for(const changed of [blood.text.replace('(units, player)','(ZoneName, player)').replace('units.concat','ZoneName.concat'),blood.text.replace('(units, player)','(units, ZoneName)').replace('player.getArenaUnits','ZoneName.getArenaUnits'),blood.text.replaceAll('context','ZoneName')])reject(changed,blood.file,'ordered-arena-shadowed-enum');
});
test('list snapshots cannot move across callbacks or hide captured controller aliases',()=>{
 const{text,file}=source('SOR_039');
 const original='return { target: opponentGroundUnits.concat(friendlyGroundUnits) };';
 for(const declaration of ['const alias=opponentGroundUnits;','const first=opponentGroundUnits;const alias=first;'])reject(text.replace('immediateEffects.exhaust((context) => {','immediateEffects.conditional((context) => {').replace(original,declaration+'return {condition:later=>true,onTrue:AbilityHelper.immediateEffects.exhaust(later=>({target:alias.concat(friendlyGroundUnits)}))};'),file,'arena-query-snapshot-capture');
 const hk=source('LOF_235');for(const declarations of ['const owner=context.player;','const first=context.player;const owner=first;'])reject(hkBlock(hk.text,declarations).replace('context.player.getArenaUnits','owner.getArenaUnits'),hk.file,'ordered-arena-direct-controller');
 reject(hkBlock(hk.text,'const saved=context;').replaceAll('context.player','saved.player'),hk.file,'ordered-arena-direct-controller');
});
test('ordered list is target-only and default old list shapes omit the new field',()=>{
 const{text,file}=source('LOF_235');const target='context.player.getArenaUnits({ arena: ZoneName.GroundArena }).concat(context.player.opponent.getArenaUnits({ arena: ZoneName.GroundArena }))';
 reject(text.replace('amount: 2','amount: '+target),file);reject(text.replace(target,target+'.length'),file);reject(text.replace(target,target+'.filter(card=>true)'),file);
 const ordinary=text.replace(target,'context.game.getArenaUnits({arena:ZoneName.GroundArena})');const c=compileSource(ordinary,file,enums);assert.equal(c.status,'supported',JSON.stringify(c.diagnostics));const t=c.abilities.card[0].effect.target;assert.deepEqual(t,{kind:'arenaList',receiver:{kind:'ref',path:['game']},predicate:ground});assert.equal(Object.hasOwn(t,'controllerOrder'),false);
});
