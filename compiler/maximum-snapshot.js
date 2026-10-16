'use strict';

// Match a complete synchronous helper program. Its locals belong to one
// ConditionalSystem property-factory evaluation; the scalar and sole physical
// recipient are captured before a nested selector is allowed to suspend.
function inspectMaximumHelpers(cls,ctx,{ts,unwrap,literal,unshadowedEnum}) {
 const result=new Map();
 if(ts.getDecorators?.(cls)?.length||cls.modifiers?.some(m=>m.kind===ts.SyntaxKind.DeclareKeyword))return result;
 const template=ts.createSourceFile('maximum-template.ts',`function helper(AbilityHelper) {
 return AbilityHelper.immediateEffects.conditional((context) => {
 const villainyUnits = context.player.getArenaUnits({ aspect: Aspect.Villainy });
 const maxPower = villainyUnits.map((x) => x.getPower()).reduce((p, c) => (p > c ? p : c), 0);
 return {
 condition: villainyUnits.filter((x) => x.getPower() === maxPower).length > 1,
 onTrue: AbilityHelper.immediateEffects.selectCard({
 cardTypeFilter: WildcardCardType.Unit, controller: RelativePlayer.Self,
 cardCondition: (card) => {return card.isUnit() && card.hasSomeAspect(Aspect.Villainy) && card.getPower() === maxPower;},
 immediateEffect: AbilityHelper.immediateEffects.giveExperience()
 }),
 onFalse: AbilityHelper.immediateEffects.giveExperience({ target: villainyUnits.find((x) => x.getPower() === maxPower) })
 };
 });
 }`,ts.ScriptTarget.Latest,true);
 const model=template.statements[0];
 function fingerprint(root,initial,expectedAspect) {
  let next=0;
  const walk=(node,bindings)=>{
   if(ts.isParenthesizedExpression(node))return walk(node.expression,bindings);
   if(ts.isIdentifier(node))return ['id',bindings.get(node.text)||node.text];
   if(ts.isStringLiteral(node)||ts.isNumericLiteral(node))return [node.kind,node.text];
   if(ts.isPropertyAccessExpression(node)) {
    if(ts.isIdentifier(node.expression)&&node.expression.text==='Aspect')return ['aspect',expectedAspect];
    return [node.kind,walk(node.expression,bindings),node.name.text,!!node.questionDotToken];
   }
   if(ts.isArrowFunction(node)) {
    const local=new Map(bindings),params=[];
    for(const p of node.parameters){if(!ts.isIdentifier(p.name)||p.initializer||p.dotDotDotToken||p.questionToken||p.modifiers?.length)return null;const id='$'+next++;local.set(p.name.text,id);params.push(id);}
    return [node.kind,params,node.modifiers?.map(m=>m.kind)||[],!!node.asteriskToken,walk(node.body,local)];
   }
   if(ts.isVariableDeclarationList(node))return [node.kind,node.flags&ts.NodeFlags.Const,...node.declarations.map(n=>walk(n,bindings))];
   if(ts.isVariableDeclaration(node)) {
    if(!ts.isIdentifier(node.name)||!node.initializer)return null;
    const id='$'+next++;bindings.set(node.name.text,id);return [node.kind,id,walk(node.initializer,bindings)];
   }
   if(ts.isBlock(node)){const local=new Map(bindings);return [node.kind,...node.statements.map(n=>walk(n,local))];}
   if(ts.isPropertyAssignment(node))return [node.kind,node.name.getText(),walk(node.initializer,bindings)];
   const children=[];ts.forEachChild(node,n=>{children.push(walk(n,bindings));});return [node.kind,...children];
  };
  return JSON.stringify(walk(root,new Map([[initial,'$helper']])));
 }
 const expected=fingerprint(model.body,'AbilityHelper','aspect');
 for(const member of cls.members) {
  if(!ts.isMethodDeclaration(member)||ts.getDecorators?.(member)?.length||!ts.isIdentifier(member.name)||!/^build[A-Z][A-Za-z0-9]*Ability$/.test(member.name.text)||!member.modifiers?.some(m=>m.kind===ts.SyntaxKind.PrivateKeyword)||member.modifiers.some(m=>[ts.SyntaxKind.StaticKeyword,ts.SyntaxKind.AsyncKeyword,ts.SyntaxKind.DeclareKeyword].includes(m.kind))||member.asteriskToken||member.parameters.length!==1||!ts.isIdentifier(member.parameters[0].name)||member.parameters[0].initializer||member.parameters[0].dotDotDotToken||member.parameters[0].questionToken||member.parameters[0].modifiers?.length||ts.getDecorators?.(member.parameters[0])?.length||!member.body||cls.members.filter(m=>m.name?.getText(ctx.source)===member.name.text).length!==1)continue;
  let valid=true,aspect=null;const enums=[];
  const visit=n=>{if(ts.isPropertyAccessExpression(n)&&ts.isIdentifier(n.expression)&&['Aspect','RelativePlayer','WildcardCardType'].includes(n.expression.text)){try{unshadowedEnum(n,ctx,n.expression.text);const value=literal(n,ctx);if(n.expression.text==='Aspect'){if(!['vigilance','command','aggression','cunning','heroism','villainy'].includes(value)||aspect!==null&&aspect!==value)valid=false;aspect=value;}enums.push(n);}catch(_){valid=false;}}ts.forEachChild(n,visit);};visit(member.body);
  if(!valid||aspect===null||enums.length!==4||fingerprint(member.body,member.parameters[0].name.text,'aspect')!==expected)continue;
  result.set(member.name.text,{member,aspect,used:false});
 }
 return result;
}
function maximumHelperCall(node,ctx,{ts,unwrap,fail}) {
 if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||!ctx.maximumHelpers?.has(node.expression.name.text))return null;
 const helper=ctx.maximumHelpers.get(node.expression.name.text);let setup=node;while(setup&&!ts.isMethodDeclaration(setup))setup=setup.parent;
 if(node.questionDotToken||node.expression.questionDotToken||node.arguments.length!==1||!setup||setup.parameters.length!==2||!ts.isIdentifier(setup.parameters[1].name)||!ts.isIdentifier(unwrap(node.arguments[0]))||unwrap(node.arguments[0]).text!==setup.parameters[1].name.text)fail(ctx,node,'maximum-helper-binding');
 const root=setup.parameters[1].name.text;
 for(let a=node.parent;a&&a!==setup;a=a.parent)if(ts.isFunctionExpression(a)||ts.isFunctionDeclaration(a)||a.parameters?.some(p=>ts.isIdentifier(p.name)&&p.name.text===root)||ts.isBlock(a)&&a.statements.some(st=>ts.isVariableStatement(st)&&st.declarationList.declarations.some(d=>ts.isIdentifier(d.name)&&d.name.text===root)))fail(ctx,node,'maximum-helper-binding');
 helper.used=true;return {kind:'maximumPowerExperience',aspect:helper.aspect};
}
module.exports={inspectMaximumHelpers,maximumHelperCall};
