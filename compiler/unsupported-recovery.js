'use strict';
// Query/ongoing/property-factory contracts fill vocabulary gaps. They must not
// erase a specialized fence that already rejected the shape.
function isSpecializedFence(error) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    if(pattern.startsWith('unsupported-query:')||pattern.startsWith('expression:'))return false;
    if(pattern==='arena-query-receiver')return false;
    if(pattern.startsWith('unsupported-field:')||pattern.startsWith('unsupported-attached-constant-field:'))return false;
    if(pattern==='constant-without-effect'||pattern==='expected-declarative-object')return false;
    return pattern.length>0;
}
module.exports={isSpecializedFence};
