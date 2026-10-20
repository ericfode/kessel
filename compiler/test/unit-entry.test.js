'use strict';
const test=require('node:test'), assert=require('node:assert/strict'), fs=require('node:fs'), path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const enums=loadEnums(path.resolve(__dirname,'../../forceteki/server'));
const source=fs.readFileSync(path.resolve(__dirname,'../../forceteki/server/game/cards/08_ASH/units/OutcastMercenaryStarship.ts'),'utf8');
const compile=text=>compileSource(text,'game/cards/08_ASH/units/OutcastMercenaryStarship.ts',enums);
test('unit entry lowers physical event-card and live title into a closed binding',()=>{
 const out=compile(source);assert.equal(out.status,'supported');const a=out.abilities.card[0];assert.deepEqual(a.effect.target,{kind:'ref',path:['unitEntryCard']});assert.deepEqual(a.contextTitle[1],{kind:'ref',path:['unitEntryCard','title']});assert.equal(a.hooks[0].event,'onUnitEntersPlay');assert.equal(a.hooks[0].predicate.left.steps[1].key,'controller');assert.equal(a.optional,undefined);assert.equal(a.effect.duration,'phase');assert.deepEqual(a.effect.power,{kind:'const',value:1});
});
test('unit-entry lowering does not erase optional paths or substitute a different event card field',()=>{
 for(const text of [source.replace('target: context.event.card','target: context.event?.card'),source.replace('context.event.card.title','context.event.card.subtitle')]) {const out=compile(text);assert.ok(!JSON.stringify(out.abilities).includes('"path":["unitEntryCard"]'));}
});
