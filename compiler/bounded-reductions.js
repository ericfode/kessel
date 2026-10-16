'use strict';
// Fixed, pure reductions. This is an AST bridge, never a JavaScript evaluator.
function apiTools(api) {
 const {ts,unwrap}=api;
 const path=n=>{n=unwrap(n);if(ts.isIdentifier(n))return [n.text];if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){const p=path(n.expression);return p&&[...p,n.name.text];}return null;};
 const callback=(n,count)=>{n=unwrap(n);return ts.isArrowFunction(n)&&!n.asteriskToken&&!n.modifiers?.length&&n.parameters.length===count&&n.parameters.every(p=>ts.isIdentifier(p.name)&&!p.dotDotDotToken&&!p.initializer&&!p.questionToken&&!p.modifiers?.length&&!ts.getDecorators(p)?.length)&&new Set(n.parameters.map(p=>p.name.text)).size===count&&!ts.isBlock(n.body)?n:null;};
 const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
 return {path,callback,same};
}
function normalizeBoundedReduction(node,ctx,api) {
 const {ts,unwrap,expr,fail}=api,{path,callback,same}=apiTools(api);
 if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression)||node.expression.name.text!=='reduce')return null;
 const reject=()=>fail(ctx,node,'bounded-arena-reduction');
 if(node.questionDotToken||node.expression.questionDotToken||node.arguments.length!==2||!ts.isNumericLiteral(unwrap(node.arguments[1]))||Number(unwrap(node.arguments[1]).text)!==0)reject();
 const fn=callback(node.arguments[0],2);if(!fn)reject();
 const [acc,item]=fn.parameters.map(p=>p.name.text),body=unwrap(fn.body);let base=unwrap(node.expression.expression),name;
 if(ts.isBinaryExpression(body)&&body.operatorToken.kind===ts.SyntaxKind.PlusToken&&same(path(body.left),[acc])) {
  if(same(path(body.right),[item,'damage']))name='arenaDamageSum';
  if(same(path(body.right),[item,'upgrades','length']))name='arenaUpgradeSum';
 } else if(ts.isConditionalExpression(body)&&ts.isBinaryExpression(unwrap(body.condition))&&unwrap(body.condition).operatorToken.kind===ts.SyntaxKind.GreaterThanToken&&same(path(unwrap(body.condition).left),[acc])&&same(path(unwrap(body.condition).right),[item])&&same(path(body.whenTrue),[acc])&&same(path(body.whenFalse),[item])) {
  if(!ts.isCallExpression(base)||base.questionDotToken||!ts.isPropertyAccessExpression(base.expression)||base.expression.questionDotToken||base.expression.name.text!=='map'||base.arguments.length!==1)reject();
  const mapper=callback(base.arguments[0],1);if(!mapper)reject();const mapped=unwrap(mapper.body);
  if(!ts.isCallExpression(mapped)||mapped.questionDotToken||mapped.arguments.length||!same(path(mapped.expression),[mapper.parameters[0].name.text,'getPower']))reject();
  base=unwrap(base.expression.expression);name='arenaPowerMax';
 }
 if(!name||!ts.isCallExpression(base)||base.questionDotToken||base.arguments.length||!ts.isPropertyAccessExpression(base.expression)||base.expression.questionDotToken||base.expression.name.text!=='getArenaUnits')reject();
 const receiver=expr(base.expression.expression,{...ctx,arenaReceiver:true});
 if(receiver.kind!=='ref'||!['player','opponent'].includes(receiver.path.join('.')))reject();
 return {kind:'query',name,receiver,args:[]};
}
function inspectReductionHelpers(cls,ctx,api) {
 const {ts,unwrap}=api,result=new Map();
 if(ctx.baseClass!=='EventCard')return result;
 for(const m of cls.members) {
  if(!ts.isMethodDeclaration(m)||!ts.isIdentifier(m.name)||!/^get[A-Z]\w*Amount$/.test(m.name.text)||m.modifiers?.length!==1||m.modifiers[0].kind!==ts.SyntaxKind.PrivateKeyword||m.asteriskToken||m.questionToken||m.typeParameters?.length||ts.getDecorators(m)?.length||m.parameters.length!==1||!ts.isIdentifier(m.parameters[0].name)||m.parameters[0].initializer||m.parameters[0].dotDotDotToken||m.parameters[0].questionToken||m.parameters[0].modifiers?.length||ts.getDecorators(m.parameters[0])?.length||m.body?.statements.length!==1||cls.members.filter(x=>x.name?.getText(ctx.source)===m.name.text).length!==1)continue;
  const r=m.body.statements[0];if(!ts.isReturnStatement(r)||!r.expression)continue;
  // Validate in a private lexical scope; no setup locals or receiver aliases.
  const child={...ctx,scope:new Map([[m.parameters[0].name.text,{kind:'ref',path:['context']}]]),constants:new Map()};
  try {const reduction=normalizeBoundedReduction(unwrap(r.expression),child,api);if(reduction?.name==='arenaDamageSum'&&reduction.receiver.path.join('.')==='player')result.set(m.name.text,{member:m,reduction,used:false});}catch(error){if(error.diagnostic)continue;throw error;}
 }
 return result;
}
function reductionHelperCall(node,ctx,helper,api) {
 const {ts,fail}=api;
 for(let p=node.parent;p;p=p.parent)if(ts.isFunctionExpression(p))fail(ctx,node,'reduction-helper-nonlexical-this');
 if(!ctx.callbackStage||node.questionDotToken||node.expression.questionDotToken||node.expression.expression.kind!==ts.SyntaxKind.ThisKeyword||node.arguments.length!==1||!ts.isIdentifier(node.arguments[0])||ctx.constants.has(node.arguments[0].text)||ctx.scope.get(node.arguments[0].text)?.path.join('.')!=='context')fail(ctx,node,'reduction-helper-call-scope');
 helper.used=true;return helper.reduction;
}
function powerMaximumGrant(a) {
 const q=(name,path)=>({kind:'query',name,receiver:{kind:'ref',path},args:[]});
 const expected={trigger:'constant',title:a.title,costs:[],effect:{kind:'gainKeyword',target:'chosen',duration:'persistent',keyword:'overwhelm',amount:{kind:'const',value:1}},targetResolver:{owner:'self',type:'unit',zone:'arena',predicate:{kind:'binary',op:'&&',left:q('isUnit',['card']),right:{kind:'binary',op:'===',left:q('getPower',['card']),right:q('arenaPowerMax',['player'])}}}};
 return JSON.stringify(a)===JSON.stringify(expected);
}
module.exports={normalizeBoundedReduction,inspectReductionHelpers,reductionHelperCall,powerMaximumGrant};
