# Card source conversion coverage

Known card definitions: **2605**. IR-converted definitions: **2568** (2121 from source AST; 447 metadata-only plain cards). Explicitly blocked definitions: **37**. Excluded test mocks: 0.

These counts measure bounded IR conversion, not runtime acceptance. Metadata-only conversion handles plain bases, fully parsed keyword-only units, and plain stat-only upgrades using the generic UpgradeCard fallback. The Rust runtime separately validates its supported operation set. Unsupported definitions retain their source locations and must not be loaded as no-op cards.

| Unsupported source pattern | Affected cards | Occurrences |
|---|---:|---:|
| no-source-for-rules-text | 26 | 26 |
| quarantined-source | 5 | 5 |
| class-member:GetAccessor | 3 | 3 |
| custom-inheritance:TokenCard | 2 | 2 |
| class-member:Constructor | 1 | 1 |
| class-state:highPriorityRemoval | 1 | 1 |
| expected-one-card-class | 1 | 1 |
| method:isCreditToken | 1 | 1 |
| method:isExperience | 1 | 1 |
| method:isForceToken | 1 | 1 |
| method:isShield | 1 | 1 |
| method:isWeakness | 1 | 1 |
| method:takeControl | 1 | 1 |
| registrar:addDamageModificationAbility | 1 | 1 |

Full card-by-card diagnostics and converted IDs are in [coverage.json](coverage.json). Every discovered card is represented across the concern-split catalogue shards (`manifest.json`, `cards.json`, `diagnostics.json`, `inputs.json`).
