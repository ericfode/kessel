# Working on Kessel

Read `CLAUDE.md` (or `README.md`) for the layout, constraints and commands; it applies to
Codex work too.

- Use Rust stable and Node 22 (`.nvmrc`). `cargo build --release` and `cargo test` need
  nothing else: the generated catalog and every pool are committed.
- `npm test` runs the card catalog, planner and tool tests on committed data. `npm run setup`
  installs the pinned `forceteki` submodule's dependencies (with `--ignore-scripts`) and
  downloads card data; after that `npm run cards:test` runs the compiler tests and
  `npm run cards:check` confirms `compiler/generated/` is current.
- The Wasm build uses `npm run wasm:build -- --install-tools`, then `npm run wasm:test`.
- Keep changes to the `forceteki` submodule's source out of this repo's work unless the
  user authorizes them. Dependency installs, card downloads and generated builds are
  ordinary setup operations, not engine source edits.
- Recording new reference games needs Holocron's game player; that work happens in
  Holocron's `lab/kessel-reference/`, and its output is imported here with
  `npm run cards:reference -- import`. Kessel depends on neither Holocron nor swu_bot.
- `.agents/skills/swu-card-conversion/SKILL.md` is the card conversion workflow.
- Basic development needs no service credentials. Keep tokens and `.env` files out of Git.

## Dependency and event boundaries

- `search.rs` is the bounded two-player graph prover. `swu_search.rs` implements
  its `Game` interface for the SWU `Engine`, including reply ordering, UI
  cancellation and simultaneous resource-choice policy. Keep SWU state and card
  rules out of the prover. Run `cargo test --locked -p kessel --test proof_boundary`:
  it compiles the real prover with only `seat.rs`, not the simulator/model/IR.
- Run `cargo test --locked -p kessel --test architecture_boundaries` when changing
  source contracts or scalar execution. CI's workspace test runs it too; no daemon,
  model, credential or network service is involved.
- `admission.rs`, `activation_contract.rs` and `selector_contract.rs` describe
  source/registration envelopes. They must not import runtime `Engine`/`State`,
  simulator, search or validation code. Family restrictions and provenance proofs
  remain admission responsibilities; standalone guards must validate their inputs.
- Scalar execution resolves target/amount expressions before `apply_scalar_effect`.
  Prepared windows pass captured values to that same mutation owner. That handler
  must not reselect targets, evaluate original expressions or decode `PreparedEvent`.
  Keep capture, replacement, primaries, contingents and recomputation order explicit.
- When changing a stateful flow, explain its phases and ownership top down next to
  the real functions, including why timing boundaries differ. A new type/helper is
  useful only if it reduces interacting responsibilities or repeated semantics.
- The architecture tests guard selected direct source edges and evaluator calls;
  they do not resolve arbitrary aliases, macro expansions or transitive behavior.
  Review those boundaries when moving code. Do not treat a passing check as a
  coupling score or replace necessary raw-header/provenance checks with defaults.

## Cursor Cloud specific instructions

- There is no dev server. `cargo test`, `npm test`, and `kessel check` on a compiled fixture are the end-to-end checks.
- `wasm-bindgen-cli` 0.2.129 (pinned by `crates/kessel-wasm`) needs rustc 1.86 or newer. A machine whose `rustc --version` is older should `rustup toolchain install stable --profile minimal` and `rustup default stable` before `npm run wasm:build -- --install-tools`.
- `CARGO_PROFILE_DEV_OPT_LEVEL=2 cargo test --locked --workspace` matches CI. Reference conformance is the slow test.
- `npm run cards:check` rebuilds the catalog from the pinned forceteki sources and the card JSON `get-cards` just downloaded. The official card API can add or renumber cards after `compiler/generated/` was committed; the check then fails while `npm run cards:test` still passes. Leave the committed catalog alone unless the regeneration is intentional.
