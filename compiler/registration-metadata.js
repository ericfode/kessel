'use strict';

// Source-zone scope and trigger ownership are registration data. Neither is a
// target-resolver filter, and the activating player may differ from controller.
function metadata(obj,api,fields) {
    const {literal,expr,fail}=api,out={};
    for(const key of fields)if(obj.props.has(key)) {
        const node=obj.props.get(key);
        if(key==='canBeTriggeredBy')out[key]=expr(node,{...obj.ctx,eventHistory:true});
        else {
            const value=literal(node,obj.ctx);
            if(['showUnresolvable','appendOverrideTitle','collectiveTrigger'].includes(key)) {if(typeof value!=='boolean')fail(obj.ctx,node,'registration-boolean:'+key);}
            else if(typeof value!=='string'&&!(Array.isArray(value)&&value.every(x=>typeof x==='string')))fail(obj.ctx,node,'registration-zone-filter:'+key);
            out[key]=value;
        }
    }
    return out;
}
module.exports={metadata};
