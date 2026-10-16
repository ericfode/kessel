'use strict';

const TYPES={createBattleDroid:'battleDroid',createCloneTrooper:'cloneTrooper',createXWing:'xwing',createTieFighter:'tieFighter',createSpy:'spy',createBeast:'beast',createMandalorian:'mandalorian'};
// A private construction program already evaluated this const record exactly
// once. Validate its source shape, then bind its stored fields; never expand or
// re-evaluate that initializer at any of the seven constructor call sites.
function tokenRecordConstruction(call,ctx,kind,api){
 if(!Object.hasOwn(TYPES,kind)||call.arguments.length!==1)return null;
 const {ts,unwrap,fail}=api,input=unwrap(call.arguments[0]);
 if(!ts.isIdentifier(input)||!ctx.pureProgram||!ctx.constructionProgram)return null;
 const reference=ctx.scope.get(input.text);
 if(reference?.kind!=='ref'||reference.path.length!==1||!reference.path[0].startsWith('data:local:')||ctx.programLocals?.get(reference.path.join('.'))!==false)return null;
 let declaration=null;
 for(let ancestor=input.parent;ancestor;ancestor=ancestor.parent){
  if(!ts.isBlock(ancestor))continue;
  const matches=ancestor.statements.filter(ts.isVariableStatement).flatMap(s=>s.declarationList.declarations.filter(d=>ts.isIdentifier(d.name)&&d.name.text===input.text).map(d=>({statement:s,declaration:d})));
  if(matches.length){if(matches.length!==1)fail(ctx,input,'token-constructor-record-shadow');const found=matches[0];if(!(found.statement.declarationList.flags&ts.NodeFlags.Const)||!found.declaration.initializer||found.declaration.getStart(ctx.source)>=input.getStart(ctx.source))fail(ctx,input,'token-constructor-record-declaration');declaration=found.declaration;break;}
 }
 if(!declaration||reference.path[0]!=='data:local:'+declaration.getStart(ctx.source))fail(ctx,input,'token-constructor-record-binding');
 const initializer=unwrap(declaration.initializer);
 if(!ts.isObjectLiteralExpression(initializer))fail(ctx,initializer,'token-constructor-record-object');
 const fields=[];
 for(const property of initializer.properties){if(!ts.isPropertyAssignment(property)||!ts.isIdentifier(property.name)||!['amount','entersReady','target','enterPlayEffect'].includes(property.name.text)||fields.includes(property.name.text))fail(ctx,property,'token-constructor-record-field');fields.push(property.name.text);}
 if(fields.length!==4)fail(ctx,initializer,'token-constructor-record-complete-fields');
 const field=name=>({kind:'capturedRecordField',record:reference,property:name,readAt:'nativeConstructorArgumentEvaluation'});
 return {kind:'nativeTokenUnitConstructionFromRecord',tokenType:TYPES[kind],record:reference,recordOrigin:{binding:'data:local:'+declaration.getStart(ctx.source),evaluation:'originalPureProgramDeclaration',identity:'existingReadonlyRecord',initializerEvaluationAtUse:'forbidden'},constructor:{nativeClass:'CreateTokenUnitSystem',allocation:'nativeSystemInstance',wrapper:'GameSystem.appendToPropertiesOrPropertyFactory',properties:{kind:'copyCapturedRecordWithFixedProperties',source:reference,sourcePropertyOrder:fields,append:{tokenType:TYPES[kind]},copy:'ownEnumerableValuesInNativeOrder',valueIdentity:'retainPhysicalObjectsAndExistingSystems'}},fieldBindings:{amount:{value:field('amount'),domain:'nativeNumber'},entersReady:{value:field('entersReady'),domain:'booleanOrUndefined',coercion:'none'},target:{value:field('target'),domain:'playerOrPlayerArrayOrUndefined',identity:'capturedPlayerObjects'},enterPlayEffect:{kind:'existingNativeSystemValue',value:field('enterPlayEffect'),domain:'gameSystemOrArrayOrUndefined',identity:'retainInstancesAndCapturedCallbacks',reconstruction:'forbidden'}},nativePropertyGeneration:{mergeOrder:['contextDefaultTarget','systemDefaults','additionalProperties','storedConstructorProperties'],defaults:{amount:1,entersReady:false,target:'contextPlayer'},presentUndefinedFields:'overwriteDefaults',targetNormalization:'wrapScalarThenFilterBoolean'},creationEventContract:'CreateTokenUnitSystem',entryEffectHandling:'forwardExistingSystemInstancesThroughCreationEventToPutIntoPlay',executionCapability:'pendingNativeTokenRecordConstructor'};
}
// Explicit target overrides the enclosing resolver's default. The persistent
// Force token is reused, not allocated. Even a never-selected record arm still
// constructs this system when the enclosing thirteen-entry record is built.
function explicitForceTokenSystem(kind,obj,api){
 if(kind!=='theForceIsWithYou'||!obj.props.has('target'))return null;
 for(const [key,value]of obj.props)if(key!=='target')api.fail(obj.ctx,value,'force-token-property:'+key);
 return {kind:'nativeForceTokenSystem',target:api.target(obj.props.get('target'),obj.ctx,'ownPlayer'),targetOverride:'explicitConstructorProperty',eventName:'onTokensCreated',tokenType:'the-force',constructor:'CreateForceTokenSystem',defaultTarget:'contextPlayer',eventGeneration:{parent:'PlayerTargetSystem',multiplicity:'oneEventPerLegalTarget',update:{generatedTokens:'freshEmptyArrayForEachEvent',iterate:'completeGeneratedPropertiesTargetListForEveryEvent',skip:'targetPlayerAlreadyHasTheForce',lookup:{zone:'targetPlayerOutsideTheGame',predicate:'isForceToken',assertCount:1,assertMessage:'There should be exactly one Force token in the outside the game zone for player {player.name}.'},append:'existingForceTokenIdentity'}},handler:{iterate:'event.generatedTokens',operation:'moveTo',zone:'base'},canAffect:{generatePropertiesAdditionalProperties:'omittedInNativeOverride',rejectWhen:{kind:'and',terms:[{kind:'or',terms:['properties.isCost','mustChangeGameStateNotNone']},'contextPlayerHasTheForce']},then:'PlayerTargetSystem.canAffectInternal'},executionCapability:'pendingNativeForceTokenSystem'};
}
module.exports={tokenRecordConstruction,explicitForceTokenSystem};
