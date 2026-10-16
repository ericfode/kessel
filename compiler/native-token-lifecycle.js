'use strict';

// A separate, complete token lifecycle program. Card entries and native adapter
// gates remain untouched. Unsupported lifecycle members reject the entire stage.
const path=require('path');
const fs=require('fs');
const crypto=require('crypto');
const ts=require(require.resolve('typescript',{paths:[path.resolve(__dirname,'../forceteki')]}));
const hash=s=>crypto.createHash('sha256').update(s).digest('hex');
const constant=value=>({kind:'const',value});
function compileNativeTokenLifecycle(text,file,sourceRoot,enums){
    const source=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true);
    const classes=source.statements.filter(ts.isClassDeclaration);
    if(classes.length!==1||source.parseDiagnostics.length)return null;
    const cls=classes[0],base=cls.heritageClauses?.[0]?.types?.[0]?.expression?.getText(source);
    if(!['TokenCard','TokenUpgradeCard'].includes(base))return null;
    const imports=new Map();
    for(const statement of source.statements){
        if(ts.isImportDeclaration(statement)){
            const bindings=statement.importClause?.namedBindings;
            if(bindings&&ts.isNamedImports(bindings))for(const e of bindings.elements){if(imports.has(e.name.text))return null;imports.set(e.name.text,{name:e.propertyName?.text??e.name.text,module:statement.moduleSpecifier.text,typeOnly:!!(statement.importClause.isTypeOnly||e.isTypeOnly)});}
        }else if(statement!==cls)return null;
    }
    let currentFile=file;
    const bound=(name,suffix)=>{const v=imports.get(name);return v&&!v.typeOnly&&v.name===name&&path.posix.normalize(path.posix.join(path.posix.dirname(currentFile),v.module))==='game'+suffix;};
    if(!bound(base,'/core/card/TokenCards'))return null;
    const reject=()=>{throw new Error('unsupported-native-lifecycle');};
    let parameter=null;
    const expr=n=>{
        while(ts.isParenthesizedExpression(n))n=n.expression;
        if(n.kind===ts.SyntaxKind.TrueKeyword)return constant(true);
        if(n.kind===ts.SyntaxKind.FalseKeyword)return constant(false);
        if(n.kind===ts.SyntaxKind.NullKeyword)return constant(null);
        if(n.kind===ts.SyntaxKind.ThisKeyword)return {kind:'ref',path:['source']};
        if(ts.isIdentifier(n)&&n.text===parameter)return {kind:'ref',path:['newController']};
        if(ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n))return constant(n.text);
        if(ts.isTemplateExpression(n))return {kind:'textTemplate',parts:[constant(n.head.text),...n.templateSpans.flatMap(s=>[expr(s.expression),constant(s.literal.text)])]};
        if(ts.isPropertyAccessExpression(n)&&!n.questionDotToken){
            if(ts.isIdentifier(n.expression)&&['ZoneName','TokenUpgradeName'].includes(n.expression.text)){
                if(!bound(n.expression.text,'/core/Constants'))reject();const value=enums[n.expression.text+'.'+n.name.text];if(value===undefined)reject();return constant(value);
            }
            const parent=expr(n.expression);
            if(parent.kind!=='ref'||!['zone','name','zoneName','controller','owner','internalName'].includes(n.name.text))reject();
            return {kind:'ref',path:[...parent.path,n.name.text]};
        }
        if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsEqualsEqualsToken)return {kind:'binary',op:'===',left:expr(n.left),right:expr(n.right)};
        reject();
    };
    const block=body=>{
        if(!body||body.statements.length>8)reject();
        return body.statements.map(s=>{
            if(ts.isReturnStatement(s)&&s.expression)return {kind:'return',value:expr(s.expression)};
            if(ts.isIfStatement(s)&&!s.elseStatement&&ts.isBlock(s.thenStatement))return {kind:'if',condition:expr(s.expression),onTrue:block(s.thenStatement)};
            if(!ts.isExpressionStatement(s))reject();
            const n=s.expression;
            if(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.EqualsToken){const target=expr(n.left);if(target.kind!=='ref'||!['source.controller','source.owner','source.zone'].includes(target.path.join('.')))reject();return {kind:'writeTokenState',target,value:expr(n.right)};}
            if(!ts.isCallExpression(n)||n.questionDotToken||n.typeArguments?.length||!ts.isPropertyAccessExpression(n.expression)||n.expression.questionDotToken)reject();
            const callee=n.expression;
            if(ts.isIdentifier(callee.expression)&&callee.expression.text==='Contract'&&callee.name.text==='assertTrue'&&bound('Contract','/core/utils/Contract')&&[1,2].includes(n.arguments.length))return {kind:'assert',condition:expr(n.arguments[0]),failure:'throw',...(n.arguments[1]?{message:expr(n.arguments[1])}:{})};
            const receiver=expr(callee.expression);
            if(receiver.kind==='ref'&&receiver.path.join('.')==='source'&&callee.name.text==='moveTo'&&n.arguments.length===1)return {kind:'moveToken',destination:expr(n.arguments[0]),inheritedDispatch:'Card.moveTo'};
            if(receiver.kind==='ref'&&receiver.path.join('.')==='source.zone'&&callee.name.text==='removeCard'&&n.arguments.length===1&&n.arguments[0].kind===ts.SyntaxKind.ThisKeyword)return {kind:'removeTokenFromCurrentZone'};
            reject();
        });
    };
    try{
        const methods=[];let identity=null;
        for(const member of cls.members){
            const name=member.name?.getText(source);
            if(name==='getImplementationId'){if(!ts.isMethodDeclaration(member)||member.parameters.length||member.body?.statements.length!==1||!ts.isReturnStatement(member.body.statements[0]))reject();const value=member.body.statements[0].expression;if(!value||!ts.isObjectLiteralExpression(value)||value.properties.length!==2)reject();identity={};for(const p of value.properties){if(!ts.isPropertyAssignment(p)||!['id','internalName'].includes(p.name.getText(source))||!ts.isStringLiteral(p.initializer))reject();identity[p.name.getText(source)]=p.initializer.text;}if(!identity.id||!identity.internalName)reject();continue;}
            if(member.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword)||ts.getDecorators(member)?.length)reject();
            if(ts.isGetAccessorDeclaration(member)&&name==='tokenName'&&member.parameters.length===0){methods.push({name,kind:'getter',body:block(member.body)});continue;}
            if(!ts.isMethodDeclaration(member)||member.typeParameters?.length||member.asteriskToken)reject();
            if(/^is(?:ForceToken|CreditToken|Experience|Weakness)$/.test(name)&&member.parameters.length===0){const body=block(member.body);if(body.length!==1||body[0].kind!=='return'||body[0].value.value!==true)reject();methods.push({name,kind:'typePredicate',body});continue;}
            if(name==='takeControl'&&member.parameters.length===1){const p=member.parameters[0];if(!ts.isIdentifier(p.name)||p.initializer||p.questionToken||p.dotDotDotToken||p.modifiers?.length||p.type?.getText(source)!=='Player')reject();parameter=p.name.text;methods.push({name,kind:'tokenControlMutation',parameters:[{name:'newController',type:'player'}],body:block(member.body)});parameter=null;continue;}
            reject();
        }
        // These inherited overrides are source programs too, and are bound to
        // their full helper files rather than inferred from token names.
        if(!identity||base==='TokenUpgradeCard'&&!methods.some(m=>m.name==='tokenName'))reject();
        const tokenFile='game/core/card/propertyMixins/Token.ts',tokenText=fs.readFileSync(path.join(sourceRoot,tokenFile),'utf8');
        const tokenSource=ts.createSourceFile(tokenFile,tokenText,ts.ScriptTarget.Latest,true);
        const fn=tokenSource.statements.find(n=>ts.isFunctionDeclaration(n)&&n.name?.text==='AsToken');
        const inner=fn?.body?.statements.find(ts.isClassDeclaration);
        if(!fn||!inner||fn.parameters.length!==1||fn.parameters[0].name.getText(tokenSource)!=='BaseClass'||inner.heritageClauses?.[0]?.types?.[0]?.expression.getText(tokenSource)!=='BaseClass'||fn.body.statements.length!==2||!ts.isReturnStatement(fn.body.statements[1])||fn.body.statements[1].expression?.getText(tokenSource)!=='AsToken')reject();
        const inherited=[];
        imports.clear();currentFile=tokenFile;
        for(const statement of tokenSource.statements)if(ts.isImportDeclaration(statement)&&statement.importClause?.namedBindings&&ts.isNamedImports(statement.importClause.namedBindings))for(const e of statement.importClause.namedBindings.elements)imports.set(e.name.text,{name:e.propertyName?.text??e.name.text,module:statement.moduleSpecifier.text,typeOnly:!!(statement.importClause.isTypeOnly||e.isTypeOnly)});
        // Local Contract/ZoneName imports in Token.ts have the same semantics.
        for(const m of inner.members){if(!ts.isMethodDeclaration(m)||m.parameters.length||!['removeFromGame','isToken','isPlayable'].includes(m.name.getText(tokenSource)))reject();inherited.push({name:m.name.getText(tokenSource),body:block(m.body)});}
        if(inherited.length!==3)reject();
        if(base==='TokenUpgradeCard')inherited.push({name:'isTokenUpgrade',body:[{kind:'return',value:constant(true)}]});
        const parentFile='game/core/card/TokenCards.ts',parentText=fs.readFileSync(path.join(sourceRoot,parentFile),'utf8');
        const parentSource=ts.createSourceFile(parentFile,parentText,ts.ScriptTarget.Latest,true);
        const parent=parentSource.statements.find(n=>ts.isClassDeclaration(n)&&n.name?.text===base);
        if(!parent||parent.members.some(m=>!(ts.isPropertyDeclaration(m)&&m.name.getText(parentSource)==='state')&&!(base==='TokenUpgradeCard'&&(ts.isMethodDeclaration(m)&&m.name.getText(parentSource)==='isTokenUpgrade'||ts.isGetAccessorDeclaration(m)&&m.name.getText(parentSource)==='tokenName'))))reject();
        if(base==='TokenUpgradeCard'){const marker=parent.members.find(m=>m.name?.getText(parentSource)==='isTokenUpgrade');if(!marker||marker.parameters.length||marker.body?.statements.length!==1||marker.body.statements[0].expression?.kind!==ts.SyntaxKind.TrueKeyword)reject();}
        return {role:'staged-native-token-lifecycle',sourcePath:file,sourceHash:hash(text),className:cls.name?.text,identity,status:'lifecycle-complete',activeCardEntryUnchanged:true,baseClass:base,inheritedBase:'InPlayCard',tokenMixin:{name:'AsToken',stateRegistration:'stateBase',methods:inherited},methods,dependencies:[{sourcePath:tokenFile,sourceHash:hash(tokenText)},{sourcePath:parentFile,sourceHash:hash(parentText)}]};
    }catch(error){if(error.message==='unsupported-native-lifecycle')return null;throw error;}
}
module.exports={compileNativeTokenLifecycle};
