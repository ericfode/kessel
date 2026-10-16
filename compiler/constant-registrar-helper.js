'use strict';

// A private helper that performs exactly one constant registration is a source
// template, not a runtime method. Bind its scalar keyword argument at each call.
function inspectConstantRegistrarHelpers(cls,ctx,api) {
    const {ts,unwrap}=api,helpers=new Map();
    for(const member of cls.members) {
        if(!ts.isMethodDeclaration(member)||!ts.isIdentifier(member.name)||member.modifiers?.length!==1||member.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||ts.getDecorators(member)?.length||member.asteriskToken||member.typeParameters?.length||member.parameters.length!==3||member.parameters.some(p=>!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length||ts.getDecorators(p)?.length)||new Set(member.parameters.map(p=>p.name.text)).size!==3||member.body?.statements.length!==1)continue;
        if(cls.members.filter(m=>m.name?.getText(ctx.source)===member.name.text).length!==1)continue;
        const statement=member.body.statements[0];if(!ts.isExpressionStatement(statement))continue;
        const call=unwrap(statement.expression);
        if(!ts.isCallExpression(call)||call.questionDotToken||call.arguments.length!==1||!ts.isPropertyAccessExpression(call.expression)||call.expression.questionDotToken||!ts.isIdentifier(call.expression.expression)||call.expression.expression.text!==member.parameters[0].name.text||call.expression.name.text!=='addConstantAbility'||!ts.isObjectLiteralExpression(unwrap(call.arguments[0])))continue;
        if(!['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(member.parameters[1].name.text))continue;
        let helperMismatch=false;const inspect=n=>{if(ts.isPropertyAccessExpression(n)&&ts.isIdentifier(n.expression)&&['AbilityHelper','abilityHelper','StaticAbilityHelper'].includes(n.expression.text)&&n.expression.text!==member.parameters[1].name.text)helperMismatch=true;ts.forEachChild(n,inspect);};inspect(call.arguments[0]);if(helperMismatch)continue;
        helpers.set(member.name.text,{member,parameters:member.parameters.map(p=>p.name.text),input:call.arguments[0],used:false});
    }
    return helpers;
}
function compileConstantRegistrarCall(call,ctx,setup,api) {
    const {ts,access,literal,fail,bindConstant,constantAbility}=api;
    const parts=access(call.expression,ctx),helper=parts.length===2&&parts[0]==='this'&&ctx.constantRegistrarHelpers?.get(parts[1]);
    if(!helper)return null;
    if(call.questionDotToken||call.arguments.length!==3||setup.parameters.length!==2||!ts.isIdentifier(call.arguments[0])||call.arguments[0].text!==setup.parameters[0].name.text||!ts.isIdentifier(call.arguments[1])||call.arguments[1].text!==setup.parameters[1].name.text)fail(ctx,call,'constant-registrar-helper-arguments');
    const keyword=literal(call.arguments[2],ctx);
    if(!['ambush','grit','hidden','overwhelm','saboteur','sentinel','shielded','support'].includes(keyword))fail(ctx,call,'constant-registrar-keyword');
    const child={...ctx,constants:new Map(ctx.constants),scope:new Map(ctx.scope),constantTemplateSource:true};
    bindConstant(child,helper.parameters[2],call.arguments[2]);
    helper.used=true;
    return constantAbility(helper.input,child);
}
module.exports={inspectConstantRegistrarHelpers,compileConstantRegistrarCall};
