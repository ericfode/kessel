'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {compileSource,loadEnums}=require('../compiler');
const root=path.resolve(process.env.SWU_ENGINE_SOURCE||path.join(__dirname,'../../forceteki/server'));
const enums=loadEnums(root);
const catalogue=require('../load-catalog').loadCatalog();
function source(id){const entry=catalogue.cards.find(c=>c.setCode===id);return {file:entry.sourcePath,text:fs.readFileSync(path.join(root,entry.sourcePath),'utf8')};}
function reject(text,file,pattern){const result=compileSource(text,file,enums);assert.equal(result.status,'unsupported',file);assert.ok(result.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(result.diagnostics));return result;}

test('full pinned conditions and property factories reject async or generator semantics',()=>{
    for(const id of ['LOF_060','LAW_147','SHD_157','ASH_066','ASH_114','SEC_104']){
        const {text,file}=source(id);
        assert.equal(compileSource(text,file,enums).status,'supported',id+' baseline');
        const async=text.replace('(context) =>','async (context) =>');
        assert.notEqual(async,text,id+' mutation must target actual callback');
        reject(async,file,'non-synchronous-callback');
    }
    const {text,file}=source('LOF_060');
    const condition='(context) => context.player.hasSomeArenaCard({ trait: Trait.Force })';
    for(const replacement of [
        'function*(context){return context.player.hasSomeArenaCard({trait:Trait.Force});}',
        'async function(context){return context.player.hasSomeArenaCard({trait:Trait.Force});}',
        'async function*(context){return context.player.hasSomeArenaCard({trait:Trait.Force});}'
    ]){const changed=text.replace(condition,replacement);assert.notEqual(changed,text);reject(changed,file,'non-synchronous-callback');}
});

test('all recognized setup methods reject generator and async registration bodies',()=>{
    for(const id of ['LOF_060','LAW_147','SHD_157','ASH_066','ASH_114','SEC_104']){
        const {text,file}=source(id);
        for(const changed of [text.replace('setupCardAbilities','*setupCardAbilities'),text.replace('setupCardAbilities','async setupCardAbilities')]){
            const result=reject(changed,file,'non-synchronous-setup-method');
            assert.equal(result.abilities.card.length,0,'generator setup must not emit unexecuted registrations');
        }
    }
    for(const method of ['setupLeaderSideAbilities','setupLeaderUnitSideAbilities','setupLeaderBackSideAbilities']){
        const text=`export default class Probe extends LeaderUnitCard {getImplementationId(){return{id:'probe',internalName:'probe'}} *${method}(registrar,AbilityHelper){registrar.addWhenPlayedAbility({title:'Draw',immediateEffect:AbilityHelper.immediateEffects.draw()});}}`;
        reject(text,'leader.ts','non-synchronous-setup-method');
    }
});

test('implementation identity cannot be extracted from Promise or generator iterator',()=>{
    const {text,file}=source('LOF_060');
    for(const changed of [text.replace('getImplementationId','async getImplementationId'),text.replace('getImplementationId','*getImplementationId')]){
        const result=reject(changed,file,'non-synchronous-implementation-identity');
        assert.ok(!result.id);
        assert.ok(!result.internalName);
    }
});

test('pinned callbacks reject rest arrays and ignored default initializers before scalar binding',()=>{
    for(const id of ['LOF_060','LAW_147','SHD_157','ASH_066','ASH_114','SEC_104']){
        const {text,file}=source(id);
        for(const binding of ['(...context) =>','(context = sideEffect()) =>']){
            const changed=text.replace('(context) =>',binding);
            assert.notEqual(changed,text,id+' callback mutation must apply');
            reject(changed,file,'unsupported-callback-parameter-binding');
        }
    }
    const {text,file}=source('LOF_060');
    reject(text.replace('(context) =>','({player}) =>'),file,'destructured-callback-parameter');
});

test('setup registration receivers and identity calls reject rest/default/decorator bindings',()=>{
    const {text,file}=source('LOF_060');
    for(const replacement of [
        'setupCardAbilities(...registrar: INonLeaderUnitAbilityRegistrar[])',
        'setupCardAbilities(registrar = sideEffect(), AbilityHelper: IAbilityHelper)',
        'setupCardAbilities(registrar: INonLeaderUnitAbilityRegistrar, AbilityHelper = sideEffect())',
        'setupCardAbilities(@wrap registrar: INonLeaderUnitAbilityRegistrar, AbilityHelper: IAbilityHelper)'
    ]){
        const changed=text.replace(/setupCardAbilities\([^)]*\)/,replacement);
        const result=reject(changed,file,'unsupported-setup-parameter-binding');
        assert.equal(result.abilities.card.length,0);
    }
    reject(text.replace('public override setupCardAbilities','@wrap public override setupCardAbilities'),file,'unsupported-setup-parameter-binding');
    for(const replacement of ['getImplementationId(context = sideEffect())','getImplementationId(...context)','getImplementationId(@wrap context)']){
        const changed=text.replace('getImplementationId()',replacement);
        const result=reject(changed,file,'unsupported-implementation-identity-binding');
        assert.ok(!result.id);
        assert.ok(!result.internalName);
    }
    reject(text.replace('protected override getImplementationId','@wrap protected override getImplementationId'),file,'unsupported-implementation-identity-binding');
});
