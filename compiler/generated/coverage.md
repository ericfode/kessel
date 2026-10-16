# Card source conversion coverage

Known card definitions: **2461**. IR-converted definitions: **2284** (1900 from source AST; 384 metadata-only plain cards). Explicitly blocked definitions: **177**. Excluded test mocks: 21.

These counts measure bounded IR conversion, not runtime acceptance. Metadata-only conversion handles plain bases, fully parsed keyword-only units, and plain stat-only upgrades using the generic UpgradeCard fallback. The Rust runtime separately validates its supported operation set. Unsupported definitions retain their source locations and must not be loaded as no-op cards.

| Unsupported source pattern | Affected cards | Occurrences |
|---|---:|---:|
| no-source-for-rules-text | 26 | 26 |
| registrar:addGainOnAttackAbilityTargetingAttached | 17 | 17 |
| unused-source-binding | 17 | 39 |
| non-path-reference | 16 | 17 |
| attached-constant-installed-keyword-only | 13 | 15 |
| callback-statement:IfStatement | 7 | 7 |
| nonconstant-value | 7 | 7 |
| constant-target-zone | 5 | 5 |
| deck-discard-player-target | 5 | 5 |
| hand-discard-player-target | 5 | 5 |
| quarantined-source | 5 | 5 |
| unsupported-live-stat-expression | 5 | 6 |
| unimplemented-context-field:game | 4 | 4 |
| unsupported-attached-constant-field:gainCondition | 4 | 4 |
| arena-predicate-expression | 3 | 3 |
| arena-query-card-capture | 3 | 3 |
| arena-query-receiver | 3 | 3 |
| class-member:GetAccessor | 3 | 3 |
| complex-callback-binding | 3 | 3 |
| constant-stat-dependency | 3 | 4 |
| unimplemented-context-field:undefined | 3 | 3 |
| unsupported-hand-discard-field:random | 3 | 3 |
| callback-statement:ExpressionStatement | 2 | 3 |
| choice-external-capture-stage | 2 | 2 |
| custom-inheritance:TokenCard | 2 | 2 |
| private-program-assertion | 2 | 2 |
| unsupported-hand-discard-field:cardCondition | 2 | 2 |
| arena-card-query-filter:arena | 1 | 1 |
| arena-leader-query-scope | 1 | 1 |
| arena-list-scalar | 1 | 1 |
| arena-predicate-reference | 1 | 1 |
| attack-query-receiver | 1 | 1 |
| bounded-arena-reduction | 1 | 1 |
| class-member:Constructor | 1 | 1 |
| class-state:highPriorityRemoval | 1 | 1 |
| controlled-trait-arity | 1 | 1 |
| deck-discard-amount-bound | 1 | 1 |
| dynamic-attack-lasting-requires-construction-frame | 1 | 1 |
| event-history-method:createCopy | 1 | 1 |
| event-history-method:getOtherPlayer | 1 | 1 |
| event-history-method:isResource | 1 | 1 |
| expected-one-card-class | 1 | 1 |
| live-stat-arena-capture | 1 | 1 |
| live-stat-resource-pairs-shape | 1 | 1 |
| method:isCreditToken | 1 | 1 |
| method:isExperience | 1 | 1 |
| method:isForceToken | 1 | 1 |
| method:isShield | 1 | 1 |
| method:isWeakness | 1 | 1 |
| method:takeControl | 1 | 1 |
| mutable-callback-local | 1 | 1 |
| ordered-arena-ground-query | 1 | 1 |
| property-of-nonreference-local | 1 | 1 |
| registrar:addDamageModificationAbility | 1 | 1 |
| token-unit-query-receiver | 1 | 1 |
| unbound-reference:TextHelper | 1 | 1 |
| unimplemented-context-field:exhausted | 1 | 1 |
| unsupported-arena-collection:map | 1 | 2 |
| unsupported-attached-constant-field:matchTarget | 1 | 1 |
| unsupported-field:cost | 1 | 1 |
| view-deck-saved-binding-requires-construction-frame | 1 | 1 |
| waiting-title-saved-binding-requires-program | 1 | 1 |

Full card-by-card diagnostics and converted IDs are in [coverage.json](coverage.json). Every discovered card is represented in [catalog.json](catalog.json).
