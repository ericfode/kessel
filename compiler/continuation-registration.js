'use strict';

// Pending select-unless grammar. The default is automatic when the unless system
// cannot fully/partially resolve; showUnresolvable changes the displayed choices,
// not that automatic-default boundary or the identity of the chooser.
function unlessResolver(obj,api) {
    const {literal,knownProperties,object,expr,effect,contextTitle,fail}=api,{props}=obj;
    if(!props.has('mode')||literal(props.get('mode'),obj.ctx)!=='selectUnless')return null;
    knownProperties(obj,['mode','choosingPlayer','unlessEffect','defaultEffect','highlightCards','showUnresolvable','optional']);
    const ctx={...obj.ctx,eventHistory:true,pendingRegistration:true};
    if(!props.has('unlessEffect')||!props.has('defaultEffect'))fail(ctx,obj.node,'select-unless-two-effects');
    const compileOption=node=>{const option=object(node,ctx);knownProperties(option,['effect','promptButtonText']);if(!option.props.has('effect')||!option.props.has('promptButtonText'))fail(ctx,node,'select-unless-option');return {label:literal(option.props.get('promptButtonText'),option.ctx),effect:effect(option.props.get('effect'),option.ctx,'self')};};
    const result={kind:'unlessSelection',mode:'selectUnless',resultBinding:'select',contract:'SelectTargetResolver.selectUnless',choosingPlayerOverride:'contextOverrideFirst',chooser:props.has('choosingPlayer')?expr(props.get('choosingPlayer'),ctx):{kind:'const',value:'self'},unlessEffect:compileOption(props.get('unlessEffect')),defaultEffect:compileOption(props.get('defaultEffect')),autoDefaultWhenUnlessCannot:'mustFullyOrPartiallyResolve'};
    if(props.has('highlightCards'))result.highlightCards=expr(props.get('highlightCards'),ctx);
    if(props.has('activePromptTitle'))result.promptTitleParts=contextTitle(props.get('activePromptTitle'),ctx);
    for(const key of ['showUnresolvable','optional'])if(props.has(key)){const v=literal(props.get(key),ctx);if(typeof v!=='boolean')fail(ctx,obj.node,'select-unless-boolean:'+key);result[key]=v;}
    return result;
}
module.exports={unlessResolver};
