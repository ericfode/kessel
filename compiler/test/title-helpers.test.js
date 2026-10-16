'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const path=require('node:path');
const {spawnSync}=require('node:child_process');
const {compileSource,loadEnums}=require('../compiler');
const engine=path.resolve(__dirname,'../../forceteki');
const enums=loadEnums(path.join(engine,'server'));
function compile(fragment,context=false) {
    return compileSource('export default class Example extends NonLeaderUnitCard { getImplementationId(){return {id:"new-title",internalName:"new-title"};} setupCardAbilities(registrar,AbilityHelper){registrar.addOnAttackAbility({title:`Pay '+fragment+'`,'+(context?'contextTitle:ctx=>`Pay '+fragment+' to ${ctx.source.title}`,':'')+'immediateEffect:AbilityHelper.immediateEffects.draw()});}}','title.ts',enums);
}
function oracle(expression,mode) {
    const helper=path.join(engine,'build/server/game/core/utils/TextHelper.js');
    const constants=path.join(engine,'build/server/game/core/Constants.js');
    const result=spawnSync(process.execPath,['-e',`const {TextHelper}=require(${JSON.stringify(helper)});const {KeywordName}=require(${JSON.stringify(constants)});process.stdout.write(JSON.stringify(${expression}));`],{env:{...process.env,NODE_ENV:mode},encoding:'utf8'});
    assert.equal(result.status,0,result.stderr);
    return JSON.parse(result.stdout);
}
test('resource labels match pinned headless helper, including zero and singular',()=>{
    for(const amount of [0,1,2,99]) {
        const label=oracle(`TextHelper.resource(${amount})`,'test');
        const card=compile('${TextHelper.resource('+amount+')}',true);
        assert.equal(card.status,'supported');
        assert.equal(card.abilities.card[0].title,'Pay '+label);
        assert.equal(card.abilities.card[0].contextTitle[1].value,label);
        assert.equal(oracle(`TextHelper.resource(${amount})`,'production'),`{resource:${amount}}`);
    }
});
test('bare and object keyword labels match real helper without dropping numeric zero',()=>{
    for(const input of ['KeywordName.Raid','KeywordName.Ambush','{keyword:KeywordName.Raid}','{keyword:KeywordName.Restore,amount:0}','{keyword:KeywordName.Raid,amount:3}']) {
        const label=oracle(`TextHelper.keyword(${input})`,'test');
        const card=compile('${TextHelper.keyword('+input+')}',true);
        assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));
        assert.equal(card.abilities.card[0].title,'Pay '+label);
        assert.equal(card.abilities.card[0].contextTitle[1].value,label);
        assert.match(oracle(`TextHelper.keyword(${input})`,'production'),/^\{keyword:/);
    }
});
test('title helpers retain formatter values and reject malformed arity, records and unbound references',()=>{
    for(const [call,pattern] of [['resource()','text-program-formatter-arity'],['resource(1,2)','text-program-formatter-arity'],['resource(ctx.source.damage)','unbound-reference:ctx'],['keyword()','text-program-formatter-arity'],['keyword(KeywordName.Raid,1)','text-program-formatter-arity'],['keyword({keyword:KeywordName.Raid,amount:-1})','expression:ObjectLiteralExpression'],['keyword({keyword:KeywordName.Raid,amount:ctx.source.damage})','expression:ObjectLiteralExpression'],['keyword({keyword:KeywordName.Raid,extra:1})','expression:ObjectLiteralExpression'],['keyword({amount:1})','expression:ObjectLiteralExpression']]) {
        const card=compile('${TextHelper.'+call+'}',true);assert.equal(card.status,'unsupported',call);assert.ok(card.diagnostics.some(d=>d.pattern===pattern),JSON.stringify(card.diagnostics));
    }
    for(const [call,formatter,value] of [['resource(-1)','resource',{kind:'unary',op:'-',value:{kind:'const',value:1}}],['resource(1.5)','resource',{kind:'const',value:1.5}],['keyword("unknown")','keyword',{kind:'const',value:'unknown'}]]) {
        const card=compile('${TextHelper.'+call+'}',true);assert.equal(card.status,'supported');assert.equal(card.abilities.card[0].title,'Pay [value]');assert.deepEqual(card.abilities.card[0].contextTitle,[{kind:'const',value:'Pay '},{kind:'textFormatter',formatter,value,environment:'test',resourcePlural:'amountNotStrictlyOne',keyword:'capitalizeNameAndOptionalAmountProperty',trait:'titleCaseEachSpaceSeparatedWord',aspect:'capitalize',evaluateAt:'stringExpressionEvaluation'},{kind:'const',value:' to '},{kind:'ref',path:['source','title']},{kind:'const',value:''}]);
    }
});
test('real title-helper sources retain payment, damage provenance and unless-choice contracts',()=>{
    const fs=require('node:fs');
    const lower=file=>{const relative='game/cards/'+file;const card=compileSource(fs.readFileSync(path.join(engine,'server',relative),'utf8'),relative,enums);assert.equal(card.status,'supported',JSON.stringify(card.diagnostics));return card.abilities.card[0];};
    for(const [file,amount,nextKind] of [['07_LAW/units/DoggedPursuers.ts',1,'damage'],['01_SOR/units/MiningGuildTieFighter.ts',2,'draw']]){const a=lower(file);assert.equal(a.optional,true);assert.deepEqual(a.effect,{kind:'payAdjustedResources',target:'ownPlayer',amount:{kind:'const',value:amount},paymentType:'cardEffectPayment',isCost:false,canCancelDuringPayment:false,allowCostAdjustments:true});assert.equal(a.ifYouDo.effect.kind,nextKind);}
    const volley=lower('09_HMW/events/VolleyFire.ts'),damage=volley.targetResolvers[1].effect;assert.equal(volley.targetResolvers[1].dependsOn,'friendlyUnit');assert.equal(damage.kind,'damagePropertiesFactory');assert.equal(damage.parameters[0].role,'systemContext');assert.equal(damage.body.kind,'damageFromCard');assert.deepEqual(damage.body.source,{kind:'ref',path:[damage.parameters[0].root,'targets','friendlyUnit']});assert.deepEqual(damage.body.amount.left.receiver,damage.body.source);assert.equal(damage.body.sourceCard,'sourceOrContextSourceAtEventCreation');
    const choice=lower('02_SHD/events/LookTheOtherWay.ts').targetResolvers[1];assert.equal(choice.dependsOn,'targetUnit');assert.equal(choice.targetResolver.kind,'choiceSelection');assert.equal(choice.targetResolver.mode,'selectUnless');assert.equal(choice.targetResolver.defaultEffect.effect.kind,'exhaust');assert.equal(choice.targetResolver.unlessEffect.effect.system,'payResources');assert.equal(choice.targetResolver.unlessEffect.effect.properties.target.slot,'targetUnit');assert.deepEqual(choice.targetResolver.unlessEffect.effect.properties.target.path,['controller']);assert.deepEqual(choice.targetResolver.unlessEffect.effect.properties.amount,{kind:'const',value:2});assert.deepEqual(choice.effect,{kind:'resolveChoice',resultBinding:'opponentsChoice'});
});