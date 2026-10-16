'use strict';
const {bindAttackCallback,callbackValue}=require('./attack-initiation');
const {playerCostModifier}=require('./cost-resource-systems');
const {playerPhaseModifier}=require('./player-phase-effects');
const history=require('./event-history');
function lastingContract(kind,input,context,defaultTarget,api) {
 if(!['playerLastingEffect','cardLastingEffect','whileSourceInPlayCardEffect','allCardsForPlayerLastingEffect'].includes(kind))return null;
 const {ts,unwrap,object,literal,expr,target,modifierEffects,fail,contextTitle}=api,node=unwrap(input);
 const callbackContext={...context,ongoingExpression:true};
 const binding=ts.isArrowFunction(node)||ts.isFunctionExpression(node)?bindAttackCallback(node,callbackContext,['systemContext'],api):null;
 const obj=object(binding?binding.lowered.node:node,binding?binding.lowered.ctx:callbackContext),{props,ctx}=obj;
 const scope=kind==='playerLastingEffect'?'player':kind==='allCardsForPlayerLastingEffect'?'allCardsForPlayer':'card';
 const fields=['target','effect','duration','until','condition','ability','title','ongoingEffectDescription','ongoingEffectTargetDescription',...(scope==='allCardsForPlayer'?['matchCondition','cardTargetMode']:[])];
 for(const[k,v]of props)if(!fields.includes(k))fail(ctx,v,'lasting-contract-field:'+k);
 const duration=kind==='whileSourceInPlayCardEffect'?'whileSourceInPlay':props.has('duration')?literal(props.get('duration'),ctx):null;
 if(!['persistent','untilEndOfAttack','untilEndOfPhase','untilEndOfRound','whileSourceInPlay','custom'].includes(duration))fail(ctx,obj.node,'lasting-contract-duration');
 if(!props.has('effect'))fail(ctx,obj.node,'lasting-contract-missing-effect');
 const recipient=target(props.get('target'),ctx,scope==='card'?defaultTarget:'ownPlayer');
 const raw=unwrap(props.get('effect')),nodes=ts.isArrayLiteralExpression(raw)?raw.elements:[raw];
 const effects=nodes.map(n=>{
  if(scope!=='card'){const e=playerCostModifier(n,ctx,api)||playerPhaseModifier(n,ctx,api);if(e)return e;}
  return modifierEffects(n,{...ctx,explicitOngoingContract:true},duration,scope==='card'?'lastingCard':'lastingPlayer');
 });
 const out={kind:'lastingEffectContract',scope,target:recipient,duration,effects,applicationContext:'retainSourceAbility',effectContext:'pinnedOngoingFrameworkContext',sourceZoneFilter:scope==='card'?'any':'anyArena',isLastingEffect:true,cardApplicability:scope==='card'?'suppressGainWhenBlankAndCheckCannotApplyLastingEffects':'unfiltered',requiresSourceInPlayAtApplication:kind==='whileSourceInPlayCardEffect',sourceLifetime:duration==='whileSourceInPlay'?'expireWhenSourceOutOfPlayDuringEffectResolution':'independent'};
 if(props.has('condition'))out.condition=callbackValue(props.get('condition'),ctx,['ongoingContext'],api);
 if(props.has('ability'))out.ability=expr(props.get('ability'),ctx);
 if(props.has('until')){const hooks=history.hooks(obj.node,{...ctx,eventHistory:true},{...api,object:(input,context)=>{const o=object(input,context);if(o.props.has('until')){const props=new Map(o.props);props.set('when',props.get('until'));return {...o,props};}return o;}});if(!hooks)fail(ctx,props.get('until'),'lasting-contract-custom-hooks');out.until=hooks;}
 if(duration==='custom'&&!props.has('until'))fail(ctx,obj.node,'lasting-contract-custom-without-until');
 if(scope==='allCardsForPlayer'){out.cardTargetMode=props.has('cardTargetMode')?literal(props.get('cardTargetMode'),ctx):'onlyOwned';if(props.has('matchCondition'))out.matchCondition=callbackValue(props.get('matchCondition'),ctx,['candidate'],api);out.controllerFilter='relativeToSourceControllerAtApplication';}
 if(props.has('title'))out.titleParts=contextTitle(props.get('title'),ctx);
 return binding?{kind:'lastingPropertiesFactory',parameters:binding.parameters,body:out}:out;
}
module.exports={lastingContract};
