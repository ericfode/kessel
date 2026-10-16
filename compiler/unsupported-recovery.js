'use strict';
// Query/ongoing/property-factory contracts fill vocabulary gaps. They must not
// erase a specialized fence that already rejected the shape.
//
// Only vocabulary-gap diagnostics may recover into a contract. Field/arity/
// capture/shape fences stay closed — including unsupported-field:*, which the
// specialized helpers raise for unknown properties.
//
// Intentional narrow-path misses recover at their call sites (attached constant,
// gained-host → piloting, live-stat → ongoing, constant → contract), not here.
function isSpecializedFence(error) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    if(pattern.startsWith('unsupported-query:')||pattern.startsWith('expression:'))return false;
    if(pattern==='arena-query-receiver')return false;
    if(pattern==='constant-without-effect'||pattern==='expected-declarative-object')return false;
    return pattern.length>0;
}
function recoversToContract(error,patterns) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    return patterns.includes(pattern)||!isSpecializedFence(error);
}
module.exports={isSpecializedFence,recoversToContract};
