'use strict';
const {bindAttackCallback}=require('./attack-initiation');

// A narrowly recognized native property protocol, not a shorthand target.
// GameSystem's Object.assign consumes the returned object itself. Retain the
// physical object and the complete ordered own-property copy operation: assuming
// a Card has no colliding instance field would erase subclass behavior.
function cardObjectPropertiesSystem(call,context,kind,defaultTarget,api){
 const {ts,unwrap,expr,fail}=api;
 if(kind!=='ready'||call.arguments.length!==1)return null;
 const fn=unwrap(call.arguments[0]);if(!ts.isArrowFunction(fn)&&!ts.isFunctionExpression(fn))return null;
 let body=unwrap(fn.body);
 if(ts.isBlock(body)){if(body.statements.length!==1||!ts.isReturnStatement(body.statements[0])||!body.statements[0].expression)return null;body=unwrap(body.statements[0].expression);}
 if(!ts.isPropertyAccessExpression(body)||body.questionDotToken||body.name.text!=='target'||!ts.isIdentifier(body.expression)||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name)||body.expression.text!==fn.parameters[0].name.text)return null;
 if(fn.asteriskToken||fn.modifiers?.length||fn.typeParameters?.length||fn.parameters.some(p=>p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length||ts.getDecorators(p)?.length))fail(context,fn,'card-object-properties-callback-binding');
 const bound=bindAttackCallback(fn,context,['nativeSystemContext'],api);
 const returned=expr(bound.lowered.node,bound.lowered.ctx);
 return {kind:'readyFromNativeObjectProperties',parameters:bound.parameters,invocationPolicy:'eachNativePropertyGeneration',properties:{kind:'nativeCardValuePropertyMerge',initialTarget:{kind:'inheritedSystemTarget',binding:defaultTarget==='self'?null:defaultTarget,fallback:'sourceCard'},defaults:{isRegroupPhaseReadyStep:false},additionalProperties:'currentNativeCallAdditionalProperties',returnedObject:returned,returnedValueDomain:'selectedPhysicalCardOrCardListOrNullish',mergeOrder:['initialTarget','readySystemDefaults','additionalProperties','returnedObjectOwnEnumerableProperties'],copySemantics:{operation:'ECMAScriptObjectAssign',destination:'freshOrdinaryObject',nullishSource:'skip',keys:'ownPropertyKeysInNativeOrderIncludingSymbols',enumerability:'ownDescriptorEnumerableAtCopyTime',read:'GetReturnedObjectKeyForEachEnumerableOwnKey',write:'SetDestinationKeyWithThrowTrue',identity:'copyPropertyValuesAndPhysicalReferences',prototypeProperties:'notEnumerated',accessorAndExceptionEffects:'preserveNativeGetSetOrderAndCallerExceptionBoundary'},targetNormalization:{scalar:'wrapInArray',array:'keepArray',filter:'Boolean'}},consumedProperties:['target','isCost','optional','cannotBeCancelled','contingentSourceEvent'],event:'onCardReadied',targetTypes:['unit','event','upgrade','leader'],eligibility:{zone:'arenaOrResourceOrLeaderInBase',eventOrUpgrade:'resourceOnly',requires:'canBeExhaustedAndNotPendingDefeatUnitAndNoReadyRestriction',stateChange:'isCostOrMustChangeStateRequiresExhausted'},handler:'eventCardReady',eventTarget:'eachProcessedMergedTarget',sourceProvenance:'pinnedReadySystemAndGameSystemPropertyComposition'};
}
module.exports={cardObjectPropertiesSystem};
