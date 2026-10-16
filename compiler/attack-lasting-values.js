'use strict';

// A value allocated by a properties factory can be a record, array or absent.
// Keep the allocated value's identity; this schema checks its source construction
// without rebuilding it at the later early/late attack-effect application step.
function attackLastingValue(input,context,participant,api) {
 const {ts,unwrap,expr,object,fail,modifierEffects}=api;
 const raw=unwrap(input),same=(a,b)=>a?.kind==='ref'&&b?.kind==='ref'&&a.path.join('.')===b.path.join('.');
 const origin=(input,ctx,seen=new Set())=>{
  const n=unwrap(input);if(seen.has(n))fail(ctx,n,'attack-lasting-construction-cycle');seen.add(n);
  if(!ts.isIdentifier(n))return n;
  if(n.text==='undefined'&&!ctx.scope.has(n.text)&&!ctx.constants.has(n.text))return n;
  if(ctx.constants.has(n.text))return origin(ctx.constants.get(n.text),ctx,seen);
  const reference=ctx.scope.get(n.text);
  if(reference){for(const binding of [...ctx.bindings||[]].reverse())if(binding.name===n.text&&same(ctx.constructionValues?.get(unwrap(binding.node)),reference))return origin(binding.node,ctx,seen);}
  fail(ctx,n,'attack-lasting-unknown-record-origin');
 };
 if(!ts.isConditionalExpression(raw)&&!ts.isIdentifier(raw))return null;
 const source=origin(raw,context);
 if(!context.constructionValues?.has(source)&&!context.scope.has(raw.text)&&!ts.isConditionalExpression(source)&&!(ts.isIdentifier(source)&&source.text==='undefined'))return null;
 if(!context.pendingSystemConstruction)fail(context,raw,'dynamic-attack-lasting-requires-construction-frame');
 const schema=(input,ctx,wholeProperty,depth=0)=>{
  if(depth>24)fail(ctx,input,'attack-lasting-schema-depth');
  const n=origin(input,ctx);
  if(ts.isIdentifier(n)&&n.text==='undefined'||n.kind===ts.SyntaxKind.NullKeyword){if(!wholeProperty)fail(ctx,n,'attack-lasting-nullish-array-entry');return {kind:'absentAttackLastingValue'};}
  if(ts.isConditionalExpression(n))return {kind:'attackLastingUnion',variants:[schema(n.whenTrue,ctx,wholeProperty,depth+1),schema(n.whenFalse,ctx,wholeProperty,depth+1)]};
  if(ts.isArrayLiteralExpression(n)){if(n.elements.length>16)fail(ctx,n,'attack-lasting-entry-bound');return {kind:'attackLastingArraySchema',items:n.elements.map(e=>schema(e,ctx,false,depth+1))};}
  if(ts.isArrowFunction(n)||ts.isFunctionExpression(n)){const binding=api.bindAttackCallback(n,ctx,['systemContext','attack'],api);return {kind:'attackLastingFactorySchema',parameterRoles:['systemContext','attack'],returns:schema(binding.lowered.node,binding.lowered.ctx,false,depth+1)};}
  const obj=object(n,ctx);
  for(const[k,v]of obj.props)if(!['effect','condition'].includes(k))fail(obj.ctx,v,'attack-lasting-record-field:'+k);
  if(!obj.props.has('effect'))fail(obj.ctx,obj.node,'attack-lasting-without-effect');
  // Validate the full generator vocabulary/callbacks; the actual constructor
  // values are retained in the captured record, so this is not an extra apply.
  modifierEffects(obj.props.get('effect'),{...obj.ctx,explicitOngoingContract:true},'attack',{kind:'ref',path:['attackParticipant']});
  if(obj.props.has('condition'))api.callbackValue(obj.props.get('condition'),obj.ctx,['attack','systemContext'],api);
  return {kind:'attackLastingRecordSchema',fields:{effect:'ongoingGeneratorOrGeneratorArray',...(obj.props.has('condition')?{condition:'attackAndContextPredicate'}:{})}};
 };
 const checked=schema(source,context,true);
 return {kind:'attackLastingValue',value:expr(raw,context),schema:checked,participant,valueEvaluation:'propertyGenerationRetainsAllocatedValue',normalization:'nullishToEmptyArrayOtherwiseAsArray',partition:{early:participant==='attacker'?'nonFactoryAndNullishCondition':'none',late:participant==='attacker'?'factoryOrNonNullishCondition':'everyEntryPerDefender'},earlyEvaluation:'beforeDefenderSelection',lateEvaluation:'afterAttackCreated',factoryArguments:['systemContext','attack'],conditionArguments:['attack','systemContext'],generatorApplication:{target:'attackParticipant',duration:'untilEndOfAttack',conditionComposition:'entryConditionAndExistingGeneratorCondition'}};
}
module.exports={attackLastingValue};
