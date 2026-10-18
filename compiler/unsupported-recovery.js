'use strict';
// Narrow legacy parsers and complete source contracts share diagnostic names.
// A miss such as non-path-reference, a callback statement or an unsupported
// legacy field must still reach the existing query/program/factory validator.
// Those validators preserve lexical captures and reject their own unknown
// fields; classifying every legacy miss as terminal bypasses that validation.
// Only invariant violations that cannot become valid in a wider representation
// stop recovery here. Snapshot timing remains owned by the construction frames
// and lexical checks at their original call sites.
const invariantFailures=new Set([
    'syntax-alias-enum-binding',
    'unsupported-setup-parameter-binding',
    'setup-helper-binding',
    'optional-registrar-call',
    'non-synchronous-implementation-identity',
    'non-synchronous-setup-method',
    'non-synchronous-callback',
    'unsupported-callback-parameter-binding',
    'callable-display-title',
    'function-valued-card-condition',
    'attached-registration-properties-must-be-object',
    'attached-keyword-amount-bound',
    'title-helper-amount',
    'title-helper-string',
    'incomplete-stat-modifier',
    'function-valued-stat-modifier',
    'noninteger-stat-modifier',
    'live-stat-expression-bound',
    'live-stat-value-bound',
]);
function isSpecializedFence(error) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    return invariantFailures.has(pattern)||pattern.startsWith('unsupported-stat-modifier-field:');
}
function recoversToContract(error,patterns) {
    if(!error||!error.diagnostic)return false;
    const pattern=error.diagnostic.pattern||'';
    return patterns.includes(pattern)||!isSpecializedFence(error);
}
module.exports={isSpecializedFence,recoversToContract};
