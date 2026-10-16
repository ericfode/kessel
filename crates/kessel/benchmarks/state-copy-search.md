# State copying and bounded native search

Run the Rust-only measurement example from the repository root:

```sh
cargo run --offline --locked --release -p kessel --example state_search_bench -- \
  --rounds 7 --nodes 500 --copy-iterations 200000 > /tmp/state-search.jsonl
```

The example prints JSON lines. The first row reports actual Rust layouts and the
unchanged Support capacities (32 grants, 80 copied program IDs per grant). Each
subsequent round reports two copy measurements and four actual `Search::solve`
measurements. Round zero warms up the workloads; exclude it when summarizing.
Workload order rotates between rounds. Search uses depth 6, a 512-entry table,
and the requested cumulative action-window node limit, without a time limit.

The copy microbenchmark preallocates 64 positions, then repeatedly copies a
complete `State` between distinct source/destination slots. Non-inlined assignment
and `black_box` prevent scalar-only or dead-store measurement. Allocation,
initialization, the final checksum, and JSON output are outside the timed region.
The 20,472-byte array is a size-only control, **not an older semantic State**.
Both rings fit in typical modern CPU caches; these numbers do not measure cold
DRAM copies, allocation, hashing, or game-rule execution.

The actual search workloads are:

- `strict_midgame`: state index 30 in the committed strict-pool
  `heuristic-vs-heuristic-1` recording.
- `compiled_midgame`: state index 30 in the committed compiled-effects
  `effect-interactions-4001` recording, using its committed pool and the current
  generated catalogue. The tool explicitly supplies an empty version-1 Support
  payload because this older recording predates Support and its selected pool
  contains no Support cards. This is an input compatibility annotation.
- `support_pending_attack`: a constructed round-6 board, validated through
  snapshot/import before paying for ASH_154, choosing an existing friendly
  attacker, and stopping at its native attack-target prompt. The saved Support
  continuation is pending.
- `support_retained_grant`: the same board after the legal "Pass attack" input,
  retaining one grant. Snapshot/import equality of legal actions and semantic
  hash is checked before measurement. These two positions are Rust-derived
  legal scenarios, not engine recordings.

The example only reads selected recorded states; it does not replay games.
It rejects more than 32 rounds, 20,000 nodes per solve, or two million copies per
round. Zero copy iterations skips the microbenchmark. A node-limited incomplete
proof is an expected measurement result, not a win or loss assertion.

For comparisons, preserve release binaries built with the same toolchain/profile,
freeze their catalogue/pool/trace bytes, and run them sequentially in alternating
order. Exclude warmups and compare state hashes, results, completion, completed
depth, nodes, decisions and transpositions before interpreting timings. Report
actual work counts and process wall time separately from `SolveResult.ms`, which
measures the search itself. Do not run builds or other benchmarks concurrently.
Shared-host interference and the small workload set limit generalization.
