'use strict';

const {bindAttackCallback}=require('./attack-initiation');
// The target unit and destination player are distinct captured properties. In
// particular, an owner's regroup return is not a return to the old controller.
function compileControlChange(input,context,defaultTarget,api) {
    const {ts,unwrap,object,expr,target,literal,fail}=api,node=unwrap(input);
    const factory=ts.isArrowFunction(node)||ts.isFunctionExpression(node);
    const binding=factory?bindAttackCallback(node,context,['systemContext'],api):null;
    const obj=object(binding?binding.lowered.node:node,binding?binding.lowered.ctx:context),{props,ctx}=obj;
    for(const [key,value]of props)if(!['target','newController','excludeLeaderUnit'].includes(key))fail(ctx,value,'control-change-field:'+key);
    if(!props.has('newController'))fail(ctx,obj.node,'control-change-missing-controller');
    const newController=expr(props.get('newController'),ctx);
    if(newController.kind!=='ref'||!['player','opponent','owner','controller'].includes(newController.path.at(-1)))fail(ctx,props.get('newController'),'control-change-player-reference');
    const excludeLeaderUnit=props.has('excludeLeaderUnit')?literal(props.get('excludeLeaderUnit'),ctx):true;
    if(typeof excludeLeaderUnit!=='boolean')fail(ctx,obj.node,'control-change-exclude-leader-boolean');
    const effect={kind:'takeControlOfUnit',target:target(props.get('target'),ctx,defaultTarget),newController,excludeLeaderUnit,
        requiresInPlay:true,tokenUpgradeController:'newController',leaderControllerChange:'replaceWithFrameworkDefeat'};
    return binding?{kind:'controlPropertiesFactory',parameters:binding.parameters,body:effect}:effect;
}
module.exports={compileControlChange};
