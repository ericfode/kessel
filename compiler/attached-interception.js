'use strict';
const replacement=require('./replacement-registration');
const {callbackValue}=require('./attack-initiation');
const REGISTRATIONS=new Set(['addReplacementEffectAbilityTargetingAttached','addDamageModificationAbilityTargetingAttached']);
function attachedInterception(name,input,context,api) {
 if(!REGISTRATIONS.has(name))return null;
 const damage=name==='addDamageModificationAbilityTargetingAttached';
 const obj=api.object(input,{...context,eventHistory:true}),props=new Map(obj.props);props.delete('gainCondition');
 const program=replacement.registration(damage?'addDamageModificationAbility':'addReplacementEffectAbility',input,obj.ctx,api,{...obj,props});
 const out={trigger:'constant',title:program.title?`Give “${program.title}” to the attached unit`:'Give an ability to the attached unit',costs:[],effect:{kind:'attachedInterceptionRegistration',registration:{type:damage?'damageModification':'replacementEffect',program},target:'parent',targetController:'any',targetTypes:['unit','base'],targetZone:'anyAttackable',requiresGrantorInPlay:true,sourceBinding:'attachedHost',controllerBinding:'hostController',gainAbilitySourceBinding:'upgrade',printedAbility:false,grantImplementation:damage?'gainDamageModificationAbility':'gainAbility',grantConditionContext:'upgrade',gainAbilityStorage:'sharedPropertiesAndPerRecipientUuid',unapply:'removeExactGainedUuidForRecipient'}};
 if(obj.props.has('gainCondition'))out.effect.gainCondition={kind:'shortCircuitGrantCondition',sourceMustBeInPlay:true,predicate:callbackValue(obj.props.get('gainCondition'),{...obj.ctx,ongoingExpression:true},['upgradeContext'],api)};
 return out;
}
module.exports={REGISTRATIONS,attachedInterception};
