'use strict';
const {compileCallback}=require('./private-programs');
function abilityProtocolProperties(obj,api){
 const {props,ctx,node}=obj,result={};
 if(props.has('optionalButtonTextOverride')){
  if(!props.has('optional')||api.literal(props.get('optional'),ctx)!==true)api.fail(ctx,node,'optional-button-requires-optional-ability');
  const text=api.literal(props.get('optionalButtonTextOverride'),ctx);if(typeof text!=='string')api.fail(ctx,node,'optional-button-text-string');
  result.optionalButtonTextOverride=text;result.optionalPassProtocol={argument:'passAbility',chooser:'playerChoosingOptionalOrSelf',cancelled:true,resolutionCommitted:true,fallbackWhenEmpty:'nativePassOrPassAttack'};
 }
 if(props.has('customConfirmation')){
  const callback=compileCallback(props.get('customConfirmation'),ctx,api,['abilityContext']);if(!callback||callback.parameters.length>1)api.fail(ctx,node,'custom-confirmation-callback');
  result.customConfirmation={kind:'abilityConfirmation',callback,evaluateAt:'afterEarlyTargetsAndCancelOrPassBeforeInitiateAbilityEvent',skipPrompt:'falsyCallbackResult',chooser:'contextPlayer',choices:[{label:'Continue',action:'continue'},{label:'Cancel',action:'setResolverCancelled'}],commitOnCancel:false};
 }
 return result;
}
function attachedCoordinate(obj,api){
 if(!obj.props.has('keyword')||api.literal(obj.props.get('keyword'),obj.ctx)!=='coordinate'||!obj.props.has('ability'))return null;
 for(const [key,value]of obj.props)if(!['keyword','ability','gainCondition'].includes(key))api.fail(obj.ctx,value,'attached-coordinate-field:'+key);
 let gainCondition=null;if(obj.props.has('gainCondition')){gainCondition=compileCallback(obj.props.get('gainCondition'),obj.ctx,api,['grantorContext']);if(!gainCondition||gainCondition.parameters.length>1)api.fail(obj.ctx,obj.node,'attached-coordinate-gain-condition');}
 const program=api.coordinateAbility(obj.props.get('ability'),obj.ctx);
 return {trigger:'constant',title:'Give Coordinate to the attached unit',costs:[],effect:{kind:'attachedCoordinateKeywordRegistration',target:'parent',duration:'persistent',targetController:'any',targetZone:'anyAttackable',targetTypes:['unit','base'],requiresGrantorInPlay:true,requiresParentInPlay:true,grantor:'physicalUpgrade',keywordInstanceOwner:'recipient',refreshKeywordAbilities:'onApplyAndUnapplyWhenRecipientUnitInPlay',coordinateGate:'recipientControllerHasAtLeastThreeArenaUnitsAndKeywordInstanceNotBlank',sourceBinding:'recipient',gainedAbilitySource:'recipientCoordinateConstant',gainedPrintedAbility:false,keywordRefresh:'rebuildWhileInPlayKeywordConstants',ability:program,...(obj.props.has('gainCondition')?{gainCondition}:{})}};
}
module.exports={abilityProtocolProperties,attachedCoordinate};
