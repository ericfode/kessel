'use strict';
// Math.max(empty) is -Infinity. Keep its comparison boolean and its display
// string separate; never replace the source's unseeded maximum with zero.
function isMaximumCall(node,api) {
 const {ts,unwrap}=api;node=unwrap(node);
 return ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Math'&&node.expression.name.text==='max';
}
function maximumOwner(input,ctx,api) {
 const {ts,unwrap,expr,fail,unshadowedEnum}=api,node=unwrap(input);
 if(!isMaximumCall(node,api))return null;
 const reject=()=>fail(ctx,node,'live-power-maximum-shape');
 if(!ctx.callbackStage||node.questionDotToken||node.expression.questionDotToken||node.typeArguments?.length||node.arguments.length!==1||!ts.isSpreadElement(node.arguments[0]))reject();
 unshadowedEnum(node,ctx,'Math');
 for(const statement of ctx.source.statements)if(ts.isImportDeclaration(statement)&&statement.importClause) {
  const clause=statement.importClause,binding=clause.namedBindings;
  if(clause.name?.text==='Math'||binding&&(ts.isNamespaceImport(binding)?binding.name.text==='Math':binding.elements.some(element=>element.name.text==='Math')))reject();
 }
 const map=unwrap(node.arguments[0].expression);
 if(!ts.isCallExpression(map)||map.questionDotToken||!ts.isPropertyAccessExpression(map.expression)||map.expression.questionDotToken||map.expression.name.text!=='map'||map.typeArguments?.length||map.arguments.length!==1)reject();
 const callback=unwrap(map.arguments[0]);
 if(!ts.isArrowFunction(callback)||callback.modifiers?.length||callback.typeParameters?.length||callback.parameters.length!==1||!ts.isIdentifier(callback.parameters[0].name)||callback.parameters[0].dotDotDotToken||callback.parameters[0].initializer||callback.parameters[0].questionToken||callback.parameters[0].modifiers?.length||ts.getDecorators(callback.parameters[0])?.length)reject();
 const body=unwrap(callback.body),parameter=callback.parameters[0].name.text;
 if(!ts.isCallExpression(body)||body.questionDotToken||body.typeArguments?.length||body.arguments.length||!ts.isPropertyAccessExpression(body.expression)||body.expression.questionDotToken||body.expression.name.text!=='getPower'||!ts.isIdentifier(body.expression.expression)||body.expression.expression.text!==parameter)reject();
 const arena=unwrap(map.expression.expression);
 if(!ts.isCallExpression(arena)||arena.questionDotToken||arena.typeArguments?.length||arena.arguments.length||!ts.isPropertyAccessExpression(arena.expression)||arena.expression.questionDotToken||arena.expression.name.text!=='getArenaUnits')reject();
 const receiver=expr(arena.expression.expression,{...ctx,arenaReceiver:true});
 if(receiver.kind!=='ref'||receiver.path.join('.')!=='player')reject();
 return receiver;
}
function normalizeLivePowerMaximum(node,ctx,api) {
 const {ts,unwrap,expr,fail}=api;
 if(ts.isBinaryExpression(node)&&node.operatorToken.kind===ts.SyntaxKind.LessThanToken&&isMaximumCall(node.right,api)) {
  if(!ctx.powerMaximumStage||ctx.powerMaximumStage!==ctx.callbackStage)fail(ctx,node,'live-power-maximum-comparison-stage');
  const receiver=maximumOwner(node.right,ctx,api),left=unwrap(node.left);
  if(!ts.isCallExpression(left)||left.questionDotToken||left.arguments.length||left.typeArguments?.length||!ts.isPropertyAccessExpression(left.expression)||left.expression.questionDotToken||left.expression.name.text!=='getPower')fail(ctx,node,'live-power-maximum-candidate');
  const candidate=expr(left.expression.expression,ctx);
  if(candidate.kind!=='ref'||candidate.path.join('.')!=='card')fail(ctx,node,'live-power-maximum-candidate');
  const power=root=>({kind:'query',name:'getPower',receiver:{kind:'ref',path:[root]},args:[]});
  return {kind:'query',name:'hasSomeArenaUnit',receiver,args:[{kind:'binary',op:'>',left:power('card'),right:power('target')}]};
 }
 if(ctx.powerMaximumTitle&&isMaximumCall(node,api))return {kind:'query',name:'arenaPowerMaximumText',receiver:maximumOwner(node,ctx,api),args:[]};
 return null;
}
function hasRawLivePowerMaximum(node,api) {
 const {ts,unwrap}=api;let found=false;
 const walk=n=>{if(isMaximumCall(n,api)&&n.arguments.length===1&&ts.isSpreadElement(n.arguments[0])) {const map=unwrap(n.arguments[0].expression);if(ts.isCallExpression(map)&&ts.isPropertyAccessExpression(map.expression)&&map.expression.name.text==='map'&&ts.isCallExpression(unwrap(map.expression.expression))&&ts.isPropertyAccessExpression(unwrap(map.expression.expression).expression)&&unwrap(map.expression.expression).expression.name.text==='getArenaUnits')found=true;}ts.forEachChild(n,walk);};walk(node);return found;
}
module.exports={normalizeLivePowerMaximum,hasRawLivePowerMaximum};
