'use strict';
// Query/ongoing/property-factory contracts fill vocabulary gaps. They must not
// erase a specialized fence that already rejected the shape.
//
// Only vocabulary-gap diagnostics and intentional narrow-path misses may recover
// into a contract. Field/arity/capture/shape fences stay closed — including
// unsupported-field:*, which the specialized helpers raise for unknown properties.
//
// Intentional misses: legacy helpers reject first with a dedicated pattern, then
// the caller recovers into the broader contract / piloting path. Those patterns
// are not shape fences.
function isSpecializedFence(error) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    if(pattern.startsWith('unsupported-query:')||pattern.startsWith('expression:'))return false;
    if(pattern==='arena-query-receiver')return false;
    if(pattern==='constant-without-effect'||pattern==='expected-declarative-object')return false;
    if(pattern==='attached-constant-installed-keyword-only')return false;
    if(pattern==='registrar:addGainOnAttackAbilityTargetingAttached')return false;
    if(pattern==='live-stat-arena-capture')return false;
    if(pattern==='non-path-reference')return false;
    if(pattern==='unimplemented-context-field:game')return false;
    return pattern.length>0;
}
module.exports={isSpecializedFence};
