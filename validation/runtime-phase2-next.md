# Local runtime capability batch

Validated source: `8b7d9e5e8ce4628f71e0e20980403b7020c537a9` on `codex/rust-card-batch-20261015`.
Incremental base: `91e3bfb81ce4518746552b76833f25c6b9f74071`. Accepted source checkout stays at
`943b7c56412580b3d67a1336f239c54085dc6ebb`; its preexisting changes were preserved.
Pinned engine source: `2ebed8ca2dce46ced42ca90de470e0f427c596dd`.

## Result and evidence

Runtime admissions **1083 → 1136**, zero losses: **52 ordinary converted cards + native Advantage**.
The compiler catalogue is unchanged: 2422/2458 converted, consisting of2040 AST and382 data-only;
1131 converted entries admitted,1291 converted entries rejected, plus5 admitted native adapters.
No deck-search admissions: the new strict diagnostics retain random-bottom/shuffle blockers.

The ordinary additions span7 conditional ongoing sources,2 live trait sources,7 attached trigger
aliases,4 attached trait/constant sources,14 bounded native attacks,13 Advantage-compatible sources,
and5 global defeat hooks. `phase2-next/new-card-identities.json` records all53 exact identities,
complexity families, source paths/hashes and coverage. Every source hash was rechecked.
LAW_210 has actual-card negative evidence and a positive synthetic Jabba-title dependency fixture;
the other51 ordinary additions have actual-card positive state-changing fixtures. Native Advantage
has physical identity/controller/import/lifecycle fixtures. These finite witnesses are not parity proof.

**711/711 ordinary Rust tests pass** (84.925s). Original strict+compiled Rust gates pass3/3
(28.586s), covering6 strict and16 compiled existing traces. Compiler AST freshness passes
(1.821s; no JS engine execution). Fresh Rust census/pool matches committed output byte for byte
(0.665s). Complete logs remain under `target/rust-loop-logs/` and outer `phase2-next/` artifacts.
The fast runner hashes are unchanged and assertions remain enabled.

One independent brief LOW review assessed `fb311ac821287bc07318aab1bafaff74cf8ac12f`.
It found a P2 missing complete-source-context guard for native attacks/Advantage. Commits5b1e4d6
and8b7d9e5 apply the shared helper guard while validating supported watcher declarations exactly.
Actual-entry mutations reject unknown helpers/watchers;20 Advantage focused checks,1 chronology
legacy check, and the711 aggregate checks pass. No second independent review was run.
An initial mixed Gideon fixture had a wrong controller assumption and was replaced with the faithful
actual Twins + Droid Cohort fixture; physical chronology assertions remain for both orders/seats/backends.
Earlier aggregate failures and rework logs were retained, including an obsolete snapshot fixture and
the first overly broad watcher guard. Final results above supersede those failed attempts.

## Layout and bounded measurements

State **28,528 bytes** (+320 from28208:64 native-attack storage +256 net physical registration
chronology). Card36; resolution context76; SupportGrant188; SupportState6096 bytes.
Release example build24.282s; benchmark5.441s. Seven measured rounds exclude one warmup.
State-copy median562.546ns; size-only20472-byte control418.521ns. Four500-node search workload
medians: strict14.548ms, compiled282.052ms, Support-pending49.818ms, retained-grant46.526ms.
28 measured searches +4warmups;14000 measured nodes and47803 decisions. Different State/hash
representation and separate run conditions prevent attributing a causal speedup to one change.
No sustained hourly card-conversion rate is claimed.

## Timing and limitations

Recorded batch start `2026-10-16T03:05:14.252889+00:00`; reviewed/checkpoint completion `2026-10-16T04:07:49.985136+00:00`.
Observed wall span **3755.732s** (62m35.7s), including parallel authoring,
integration, test/rework and capacity backoff. Setup/source inspection, implementation, independent
review/rework, and exclusive waits were not continuously instrumented; separate exact durations
cannot be recovered. Command wall times and each author's receipts are preserved; they overlap
and must not be summed as exclusive phases. Model-visible output bytes were not instrumented.
Saved command stdout/stderr sizes, source/patch/binary hashes are measured separately.

JS reference engines/cross-engine comparisons were not run. **Parity remains unverified.**
Status: accepted locally after review fixes and Rust checks; parent integration/source-engine parity
remain pending. Quarantine/stopped-cohort bodies and source checkout were not modified.
No merges, deployments, installs, purchases, secrets, access changes or quota resets.

Automatic approval rejected the requested GitHub push because the exact external destination
was not explicitly confirmed. No push ran; no indirect workaround was attempted. The pending
question names `https://github.com/ericfode/kessel.git`, branch `codex/rust-card-batch-20261015`.

Additional eligible work continues in isolated phase3 attack-timing and defeat-history lanes;
this report records a completed local batch, not whole-engine completion.
