 'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(__dirname,'../../forceteki/server'),file='game/cards/08_ASH/units/HeroicPurrgil.ts',source=fs.readFileSync(path.join(root,file),'utf8'),enums=loadEnums(root);
const query={kind:'query',name:'isAttackingUsingAmbush',receiver:{kind:'ref',path:['source']},args:[]};
test('actual complete Ambush condition lowers as one closed source predicate',()=>{const e=compileSource(source,file,enums);assert.equal(e.status,'supported');assert.deepEqual(e.abilities.card[0].condition,query);assert.deepEqual(e.abilities.card[0].effect,{kind:'modifyStats',target:'self',duration:'persistent',power:{kind:'const',value:2},hp:{kind:'const',value:0}});});
test('nearby optional flag reads never acquire the reviewed predicate',()=>{for(const [before,after]of [['activeAttack?.isAmbush','activeAttack.isAmbush'],['source.isAttacking()','source.isDefending()'],['source.activeAttack','target.activeAttack'],['isAmbush','isOverwhelm'],['source.isAttacking() && context.source.activeAttack?.isAmbush','source.activeAttack?.isAmbush']]){const e=compileSource(source.replace(before,after),file,enums);assert.ok(e.status!=='supported'||JSON.stringify(e.abilities).indexOf('isAttackingUsingAmbush')<0,after);}});

test('all complete catalogue entries remain identical except reviewed Ambush condition and capabilities',()=>{
 const {canonical}=require('../../cards/inventory'),baseline=require('../load-catalog').loadCatalog(),fresh=require('../build').compileCatalog();
 assert.equal(fresh.cards.length,2605);assert.equal(baseline.cards.length,2605);
 for(const actual of fresh.cards){const expected=structuredClone(baseline.cards.find(e=>e.setCode===actual.setCode));
  if(actual.setCode==='ASH_207'){expected.abilities.card[0].condition=query;expected.requiredCapabilities=['cardType:unit','ir:const','ir:modifyStats','ir:query','ir:ref','keyword:ambush','query:isAttackingUsingAmbush','ref:source','trigger:constant'];}
  assert.deepEqual(canonical(actual),canonical(expected),actual.setCode);
 }
});
