'use strict';

// Overrides are spread last by LeaderUnitCard's constructor. In particular a
// supplied condition replaces the printed resource-threshold condition.
function deploymentOverride(member,ctx,api){
 const {ts,object,expr,costs,fail}=api;
 if(!ts.isMethodDeclaration(member)||member.name.getText(ctx.source)!=='deployActionAbilityProps')return null;
 if(member.parameters.length>1||member.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length||ts.getDecorators(p)?.length)||member.typeParameters?.length||member.asteriskToken||ts.getDecorators(member)?.length||member.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)||member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0])||!member.body.statements[0].expression)fail(ctx,member,'deploy-override-shape');
 const scope=new Map(ctx.scope);if(member.parameters.length)scope.set(member.parameters[0].name.text,{kind:'ref',path:['abilityHelper']});
 const obj=object(member.body.statements[0].expression,{...ctx,scope});
 for(const[k,v]of obj.props)if(!['condition','cost','limit'].includes(k))fail(obj.ctx,v,'deploy-override-field:'+k);
 const value={kind:'leaderDeploymentOverride',evaluation:'leaderConstructorAfterSuper',merge:'spreadLastOverDefaultDeployAction',defaultAction:{limit:'sharedEpicDeployLimit',requiresConfirmation:true,zone:'base',effect:'deployLeader'},properties:{}};
 if(obj.props.has('condition'))value.properties.condition=expr(obj.props.get('condition'),obj.ctx);
 if(obj.props.has('cost'))value.properties.costs=costs(obj.props.get('cost'),obj.ctx);
 if(obj.props.has('limit'))value.properties.limit=expr(obj.props.get('limit'),obj.ctx);
 return value;
}
module.exports={deploymentOverride};
