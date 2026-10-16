'use strict';
const {bindAttackCallback,callbackValue}=require('./attack-initiation');
function compileRoundCardEffect(input,context,defaultTarget,api) {
    const {ts,unwrap,object,target,modifierEffects,fail}=api,node=unwrap(input);
    const binding=ts.isArrowFunction(node)||ts.isFunctionExpression(node)?bindAttackCallback(node,context,['systemContext'],api):null;
    const obj=object(binding?binding.lowered.node:node,binding?binding.lowered.ctx:context),{props,ctx}=obj;
    for(const [key,value]of props)if(!['target','effect','condition','title','ongoingEffectDescription','ongoingEffectTargetDescription','effectArgs'].includes(key))fail(ctx,value,'round-card-effect-field:'+key);
    if(!props.has('effect'))fail(ctx,obj.node,'round-card-effect-missing-modifier');
    const scan=n=>{if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['modifyStats','gainKeyword'].includes(n.expression.name.text)&&n.arguments[0]&&(ts.isArrowFunction(unwrap(n.arguments[0]))||ts.isFunctionExpression(unwrap(n.arguments[0]))))fail(ctx,n,'round-live-modifier-requires-binding');ts.forEachChild(n,scan);};scan(props.get('effect'));
    const recipient=target(props.get('target'),ctx,defaultTarget);
    const result={kind:'roundCardEffect',target:recipient,duration:'round',modifier:modifierEffects(props.get('effect'),ctx,'round',recipient)};
    if(props.has('condition'))result.condition=callbackValue(props.get('condition'),ctx,['effectContext'],api);
    return binding?{kind:'roundCardPropertiesFactory',parameters:binding.parameters,body:result}:result;
}
module.exports={compileRoundCardEffect};
