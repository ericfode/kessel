'use strict';

// Overrides are spread last by LeaderUnitCard's constructor. In particular a
// supplied condition replaces the printed resource-threshold condition.
function deploymentOverride(member,ctx,api){
 const {ts,object,expr,costs,fail}=api;
 if(!ts.isMethodDeclaration(member)||member.name.getText(ctx.source)!=='deployActionAbilityProps')return null;
 if(member.parameters.length!==1||!ts.isIdentifier(member.parameters[0].name)||member.parameters[0].initializer||member.parameters[0].dotDotDotToken||member.parameters[0].questionToken||member.typeParameters?.length||member.asteriskToken||ts.getDecorators(member)?.length||member.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)||member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0])||!member.body.statements[0].expression)fail(ctx,member,'deploy-override-shape');
 const scope=new Map(ctx.scope);scope.set(member.parameters[0].name.text,{kind:'ref',path:['abilityHelper']});
 const obj=object(member.body.statements[0].expression,{...ctx,scope});
 for(const[k,v]of obj.props)if(!['condition','cost'].includes(k))fail(obj.ctx,v,'deploy-override-field:'+k);
 const value={kind:'leaderDeploymentOverride',evaluation:'leaderConstructorAfterSuper',merge:'spreadLastOverDefaultDeployAction',defaultAction:{limit:'sharedEpicDeployLimit',requiresConfirmation:true,zone:'base',effect:'deployLeader'},properties:{}};
 if(obj.props.has('condition'))value.properties.condition=expr(obj.props.get('condition'),obj.ctx);
 if(obj.props.has('cost'))value.properties.costs=costs(obj.props.get('cost'),obj.ctx);
 return value;
}
module.exports={deploymentOverride};
