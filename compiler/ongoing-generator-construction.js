'use strict';

// Used only in a system-construction graph. The helper call creates generators
// now; game/source/effect properties are supplied by the ongoing framework later.
function ongoingModifierConstruction(input,context,api) {
 const {ts,unwrap,access,modifierEffects}=api,call=unwrap(input);
 if(!ts.isCallExpression(call))return null;
 let p;try{p=access(call.expression,context);}catch(error){if(!error.diagnostic)throw error;return null;}
 const helper=p.length===3&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(p[0])&&p[1]==='ongoingEffects';
 const builder=p.length===3&&p[0]==='OngoingEffectBuilder'&&p[2]==='static';
 if(!helper&&!builder)return null;
 const duration={kind:'ongoingApplicationProperty',property:'duration'};
 const target={kind:'ongoingApplicationTarget'};
 return {kind:'ongoingGeneratorConstruction',generators:modifierEffects(call,{...context,explicitOngoingContract:true},duration,target),evaluation:'constructorExpression',capture:'lexicalValuesAndPhysicalObjectIdentity',argumentsEvaluation:'leftToRightAtConstruction',generatorInvocation:{parameters:['game','source','ongoingProperties'],targetBinding:target,durationBinding:duration,valueFactories:'invokeOnGeneratorInvocation'},resultType:'ongoingEffectGenerator'};
}
module.exports={ongoingModifierConstruction};
