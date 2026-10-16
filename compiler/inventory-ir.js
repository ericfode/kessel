'use strict';

// Portable inventory operations pending engine execution. These nodes retain
// their executing callback location; they are never memoized into a live query
// when the source saved an earlier list or scalar.
function normalizeInventoryExpression(input,ctx,{ts,unwrap,expr,fail}) {
 const n=unwrap(input);
 // Keep native token identity and physical attachment aliases distinct from
 // printed keywords. New aliases are pending executor operations.
 if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['hasExperience','hasWeakness','hasShield','isCreditToken'].includes(n.expression.name.text)){
  const name=n.expression.name.text;if(n.arguments.length)fail(ctx,n,'native-token-predicate-arity');if(!ctx.callbackStage)fail(ctx,n,'native-token-predicate-stage');
  return {kind:'nativeTokenPredicate',operation:name,receiver:expr(n.expression.expression,ctx),domain:name==='isCreditToken'?'cardIdentity':'physicalAttachments',tokenType:{hasExperience:'experience',hasWeakness:'weakness',hasShield:'shield',isCreditToken:'credit'}[name],optionalReceiver:!!n.expression.questionDotToken,optionalCall:!!n.questionDotToken};
 }

 const checked=(node)=>{if(!ctx.callbackStage)fail(ctx,n,'inventory-evaluation-stage');ctx.inventoryUsage?.add(node.kind);return node;};
 const path=(v,names)=>names.length===1?ts.isIdentifier(v)&&v.text===names[0]:ts.isPropertyAccessExpression(v)&&!v.questionDotToken&&v.name.text===names.at(-1)&&path(v.expression,names.slice(0,-1));
 const call=(v,name,arity)=>ts.isCallExpression(v)&&!v.questionDotToken&&ts.isPropertyAccessExpression(v.expression)&&!v.expression.questionDotToken&&v.expression.name.text===name&&v.arguments.length===arity;
 const unary=input=>{const fn=unwrap(input);if(!ts.isArrowFunction(fn)||fn.modifiers?.length||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name)||fn.parameters[0].initializer||fn.parameters[0].dotDotDotToken||fn.parameters[0].questionToken||fn.parameters[0].modifiers?.length||ts.isBlock(fn.body))return null;return fn;};
 const cardReceiver=input=>{const receiver=expr(input,ctx);return receiver.kind==='ref'&&['source','card','target','modifierTarget'].includes(receiver.path[0])?receiver:null;};
 // A chain short-circuits after any optional step. A parenthesized inner
 // expression is a separate receiver, preserving (a?.b).c versus a?.b.c.
 if(ts.isPropertyAccessExpression(n)&&!(ctx.attachedParentTitle&&n.questionDotToken&&n.name.text==='title'&&ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='parentCard')) {
  let root=n;const nodes=[];while(ts.isPropertyAccessExpression(root)){nodes.unshift(root);root=root.expression;}
  const firstOptional=nodes.findIndex(step=>!!step.questionDotToken);
  const boundary=ts.isParenthesizedExpression(root)&&root.getText(ctx.source).includes('?.');
  if(firstOptional>=0||boundary){
   // Resolve the mandatory prefix through the ordinary context/reference binder.
   // A bare context parameter is not itself a card or player value in this IR.
   const prefix=firstOptional>=0?nodes[firstOptional].expression:root;
   const remaining=firstOptional>=0?nodes.slice(firstOptional):nodes;
   const steps=remaining.map(step=>({property:step.name.text,optional:!!step.questionDotToken}));
   if(steps.length>12)fail(ctx,n,'inventory-member-path-bound');
   return checked({kind:'memberPath',receiver:expr(prefix,ctx),steps});
  }
 }
 if(ts.isPropertyAccessExpression(n)&&n.name.text==='length') {
  const collection=unwrap(n.expression);
  if(ts.isCallExpression(collection)&&ts.isPropertyAccessExpression(collection.expression)&&['getConstantAbilities','getActionAbilities','getTriggeredAbilities'].includes(collection.expression.name.text)) {
   if(collection.questionDotToken||collection.expression.questionDotToken||collection.arguments.length)fail(ctx,n,'ability-inventory-shape');
   const receiver=cardReceiver(collection.expression.expression);if(!receiver)fail(ctx,n,'ability-inventory-receiver');
   return checked({kind:'abilityInventoryLength',receiver,category:{getConstantAbilities:'constant',getActionAbilities:'action',getTriggeredAbilities:'triggered'}[collection.expression.name.text],limit:256});
  }
  if(ts.isPropertyAccessExpression(collection)&&collection.name.text==='upgrades'&&!collection.questionDotToken){const receiver=cardReceiver(collection.expression);if(receiver)return checked({kind:'attachmentCount',receiver,limit:256});}
  if(call(collection,'filter',1)) {
   const base=unwrap(collection.expression.expression),predicate=unary(collection.arguments[0]);
   if(predicate&&ts.isPropertyAccessExpression(base)&&!base.questionDotToken&&base.name.text==='upgrades'){
    const receiver=cardReceiver(base.expression);if(receiver)return checked({kind:'attachmentCount',receiver,predicate:expr(predicate,ctx,['card']),limit:256});
   }
   if(predicate&&call(base,'flatMap',1)) {
    const map=unary(base.arguments[0]),arena=unwrap(base.expression.expression);
    if(map&&path(unwrap(map.body),[map.parameters[0].name.text,'upgrades'])&&call(arena,'getArenaUnits',0)){
     const token=unwrap(predicate.body),name=predicate.parameters[0].name.text;
     if(call(token,'isExperience',0)&&path(token.expression,[name,'isExperience'])){
      const receiver=expr(arena.expression.expression,ctx);if(receiver.kind!=='ref'||receiver.path.length!==1||!['player','opponent'].includes(receiver.path[0]))fail(ctx,n,'arena-attachment-inventory-receiver');
      return checked({kind:'arenaAttachmentCount',receiver,tokenType:'experience',limit:256});
     }
    }
   }
  }
 }
 if(call(n,'some',1)) {
  const inventory=unwrap(n.expression.expression),fn=unary(n.arguments[0]);
  if(call(inventory,'getTriggeredAbilities',0)&&fn){const body=unwrap(fn.body),name=fn.parameters[0].name.text;if(ts.isPropertyAccessExpression(body)&&!body.questionDotToken&&['isWhenPlayed','isWhenDefeated'].includes(body.name.text)&&path(body.expression,[name])){const receiver=cardReceiver(inventory.expression.expression);if(!receiver)fail(ctx,n,'ability-inventory-receiver');return checked({kind:'abilityInventorySome',receiver,trigger:body.name.text==='isWhenPlayed'?'whenPlayed':'whenDefeated',limit:256});}}
 }
 return null;
}

function pendingInventoryEffect(node,ctx,defaultTarget,api) {
 const {ts,unwrap,fail}=api;
 const ref=(...path)=>({kind:'ref',path});
 const path=(v,names)=>names.length===1?ts.isIdentifier(v)&&v.text===names[0]:ts.isPropertyAccessExpression(v)&&!v.questionDotToken&&v.name.text===names.at(-1)&&path(v.expression,names.slice(0,-1));
 const call=(v,names,arity)=>ts.isCallExpression(v)&&!v.questionDotToken&&v.arguments.length===arity&&path(v.expression,names);
 if(!ts.isCallExpression(node)||!ts.isPropertyAccessExpression(node.expression))return null;
 const kind=node.expression.name.text;
 if(['useWhenPlayedAbility','useWhenDefeatedAbility'].includes(kind)) {if(!node.arguments.length)return {kind,target:defaultTarget};return require('./triggered-inventory-systems').compileTriggeredInventorySystem(node,ctx,defaultTarget,api);}
 if(kind!=='giveTokenUpgrade')return null;
 const reject=()=>fail(ctx,node,'token-upgrade-copy-factory');
 if(node.arguments.length!==1)reject();const fn=unwrap(node.arguments[0]);
 if(!ts.isArrowFunction(fn)||fn.modifiers?.length||fn.parameters.length!==1||!ts.isIdentifier(fn.parameters[0].name)||fn.parameters[0].initializer||fn.parameters[0].dotDotDotToken||fn.parameters[0].questionToken||!ts.isBlock(fn.body)||fn.body.statements.length!==3)reject();
 const context=fn.parameters[0].name.text,[binding,assertion,returned]=fn.body.statements;
 if(context==='Contract'||!ts.isVariableStatement(binding)||!(binding.declarationList.flags&ts.NodeFlags.Const)||binding.declarationList.declarations.length!==1||!ts.isExpressionStatement(assertion)||!ts.isReturnStatement(returned)||!returned.expression)reject();
 const declaration=binding.declarationList.declarations[0];if(!ts.isIdentifier(declaration.name)||!declaration.initializer||!path(unwrap(declaration.initializer),[context,'target']))reject();const token=declaration.name.text;
 if(token==='Contract'||!call(assertion.expression,['Contract','assertTrue'],1)||!call(unwrap(assertion.expression.arguments[0]),[token,'isTokenUpgrade'],0))reject();
 const properties=unwrap(returned.expression);if(!ts.isObjectLiteralExpression(properties)||properties.properties.length!==2||properties.properties.some(p=>!ts.isPropertyAssignment(p)||!ts.isIdentifier(p.name)))reject();
 const fields=new Map(properties.properties.map(p=>[p.name.text,unwrap(p.initializer)]));if(fields.size!==2||!fields.has('target')||!fields.has('tokenType')||!path(fields.get('target'),[token,'parentCard'])||!path(fields.get('tokenType'),[token,'tokenName']))reject();
 const imported=ctx.source.statements.filter(s=>ts.isImportDeclaration(s)&&s.moduleSpecifier.text==='../../../core/utils/Contract'&&s.importClause?.namedBindings&&ts.isNamedImports(s.importClause.namedBindings)&&s.importClause.namedBindings.elements.some(e=>e.name.text==='Contract'&&(!e.propertyName||e.propertyName.text==='Contract')));
 if(imported.length!==1||ctx.constants.has('Contract')||ctx.scope.has('Contract'))reject();
 return {kind:'giveTokenUpgrade',target:ref('target','parentCard'),tokenType:ref('target','tokenName'),assertions:[{kind:'query',name:'isTokenUpgrade',receiver:ref('target'),args:[]}]};
}
module.exports={normalizeInventoryExpression,pendingInventoryEffect};
