'use strict';
const {callbackValue}=require('./attack-initiation');
function constantContract(input,context,api,validated=null) {
 const {object,knownProperties,modifierEffects,displayText,literal,fail}=api,obj=validated||object(input,context),{props,ctx}=obj;
 knownProperties(obj,['title','condition','matchTarget','ongoingEffect','sourceZoneFilter','targetController','targetCardTypeFilter','targetZoneFilter']);
 if(!props.has('ongoingEffect'))fail(ctx,obj.node,'constant-contract-missing-modifier');
 const properties={};const callbackContext={...ctx,ongoingExpression:true};
 for(const k of ['condition','matchTarget'])if(props.has(k))properties[k]=callbackValue(props.get(k),callbackContext,k==='condition'?['ongoingContext']:['candidate','ongoingContext'],api);
 for(const k of ['sourceZoneFilter','targetController','targetCardTypeFilter','targetZoneFilter'])if(props.has(k))properties[k]=literal(props.get(k),ctx);
 const modifier=modifierEffects(props.get('ongoingEffect'),{...ctx,explicitOngoingContract:true},'persistent','constantMatch');
 return {trigger:'constant',title:props.has('title')?displayText(props.get('title'),ctx):'',costs:[],effect:{kind:'constantEffectRegistration',properties,modifier,duration:'persistent',context:'refreshFromSourceController',scopeDefaults:'pinnedOngoingCardOrPlayerEffect',dependencyPolicy:'liveFixedPointRequired'}};
}
function attachedConstantContract(input,context,host,api) {
 const {object,knownProperties,displayText,modifierEffects}=api,obj=object(input,context),{props,ctx}=obj;
 knownProperties(obj,['title','condition','matchTarget','ongoingEffect',...(host?['gainCondition']:[])]);
 const grant={kind:'attachedConstantContract',target:'parent',mode:host?'gainConstant':'direct',targetController:'any',targetTypes:['unit','base'],targetZone:'anyAttackable',requiresGrantorInPlay:true,sourceBinding:host?'attachedHost':'upgrade',gainAbilitySourceBinding:'upgrade'};
 if(host) {
  const next=new Map(props);next.delete('gainCondition');grant.program=constantContract(input,ctx,api,{...obj,props:next});
  if(props.has('gainCondition'))grant.gainCondition=callbackValue(props.get('gainCondition'),{...ctx,ongoingExpression:true},['grantorContext'],api);
 } else {
  grant.modifier=modifierEffects(props.get('ongoingEffect'),{...ctx,explicitOngoingContract:true},'persistent','parent');
  if(props.has('condition'))grant.condition=callbackValue(props.get('condition'),{...ctx,ongoingExpression:true},['grantorContext'],api);
  if(props.has('matchTarget'))grant.matchCondition=callbackValue(props.get('matchTarget'),{...ctx,ongoingExpression:true},['candidate','grantorContext'],api);
 }
 return {trigger:'constant',title:props.has('title')?displayText(props.get('title'),ctx):'',costs:[],effect:grant};
}
module.exports={constantContract,attachedConstantContract};
