'use strict';

// Compiler-only source class methods. These explicit contracts do not imply
// executable support and never replace an unrecognized member with an empty one.
function classMethodContract(member,ctx,baseClass,api){
 const {ts,unwrap,expr,fail}=api;
 if(baseClass!=='NonLeaderUnitCard'||!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||!['getUpgradeHp','getUpgradePower','canAttach','checkIsAttachable'].includes(member.name.text))return null;
 const name=member.name.text;
 if(member.typeParameters?.length||member.asteriskToken||member.questionToken||ts.getDecorators(member)?.length||member.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)||member.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length||ts.getDecorators(p)?.length)||new Set(member.parameters.map(p=>p.name.text)).size!==member.parameters.length)fail(ctx,member,'class-contract-method-shape');
 if(member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0]))fail(ctx,member,'class-contract-return-body');
 const value=member.body.statements[0].expression;
 if(['getUpgradeHp','getUpgradePower'].includes(name)){
  if(member.parameters.length||!value)fail(ctx,member,'class-contract-upgrade-stat');
  const n=api.literal(value,ctx);if(!Number.isSafeInteger(n)||Math.abs(n)>1000)fail(ctx,value,'class-contract-upgrade-stat-bound');
  return {kind:'classMethodOverride',name,evaluation:'eachMethodCall',parameters:[],body:[{kind:'return',value:{kind:'const',value:n}}]};
 }
 if(name==='checkIsAttachable'){
  if(member.parameters.length||value)fail(ctx,member,'class-contract-attachment-check');
  return {kind:'classMethodOverride',name,evaluation:'eachMethodCall',parameters:[],body:[{kind:'returnUndefined'}],replacesInheritedAssertion:true};
 }
 if(member.parameters.length!==3||!value)fail(ctx,member,'class-contract-attachment-parameters');
 const scope=new Map(ctx.scope);const roots=['card','context','attachmentController'];member.parameters.forEach((p,i)=>scope.set(p.name.text,{kind:'ref',path:[roots[i]]}));
 const predicate=expr(value,{...ctx,scope,callbackStage:member});
 return {kind:'classMethodOverride',name,evaluation:'eachMethodCall',parameters:[{name:'card',type:'card'},{name:'context',type:'abilityContext'},{name:'attachmentController',type:'player'}],body:[{kind:'return',value:predicate}],replacesInheritedAttachmentConditionAndTypeGate:true};
}
function classSystem(kind,obj,defaultTarget,api){
 if(kind!=='flipDoubleSidedLeader')return null;
 for(const[key,value]of obj.props)if(key!=='target')api.fail(obj.ctx,value,'flip-leader-field:'+key);
 return {kind:'flipDoubleSidedLeader',target:api.target(obj.props.get('target'),obj.ctx,defaultTarget),targetTypes:['leader'],eligibility:'isDoubleSidedLeaderThenInheritedCardTargetSystem',event:'onLeaderFlipped',handler:[{kind:'assertCardPredicate',name:'isDoubleSidedLeader',expected:true,failure:'throw'},{kind:'assertCardPredicate',name:'isDeployableLeader',expected:false,failure:'throw'},{kind:'invokeClassContract',method:'flipLeader'}]};
}
module.exports={classMethodContract,classSystem};
