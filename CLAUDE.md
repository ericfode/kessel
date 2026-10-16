# CLAUDE.md

This file guides Claude Code (and Codex, through `AGENTS.md`) when working in this repository.

## What this repo is

Kessel is a deterministic Rust rules simulator and endgame prover for an explicit subset of Star
Wars Unlimited. The same core is compiled to WebAssembly, and a card compiler lowers forceteki
TypeScript card definitions into the data-only ability IR that the simulator runs. Every
supported card is checked against complete games recorded from the forceteki engine. Unsupported
cards are rejected with a reason and never approximated. `README.md` has the layout and commands.

It was `bot/omni-rs` (with `bot/omni-wasm`, `bot/card-compiler` and `bot/cards`) in
[swu_bot](https://github.com/ericfode/swu_bot), moved here with history. Kessel must not depend on
swu_bot or on [Holocron](https://github.com/ericfode/holocron). Holocron depends on Kessel and keeps
the engine-driving side in `lab/kessel-reference/` (reference game scenarios and generator, probe
fixtures, the card pipeline runner) and `bots/omni/rs/` (trace and pool exporters), next to its game
player and bot harness. The arena (swu_bot) vendors Kessel's Node Wasm package in `vendor/kessel`.

## Layout

- `crates/kessel/`: library `kessel` and binaries `kessel` and `kessel-pimc`. `build.rs`
  assembles the concern-split catalogue under `compiler/generated/` (`manifest.json`,
  `cards.json`, `diagnostics.json`, `inputs.json`) into Rust ahead of time.
  `data/pool.json` is the strict 100-card pool. `tests/` holds the strict, compiled and
  reference conformance suites.
- `crates/kessel-wasm/`: wasm-bindgen crate, build/pack scripts, and `js/` (Node and browser
  adapter, checker, prover).
- `compiler/`: `compiler.js` (TypeScript AST to IR), `build.js`, `keywords.js`, and
  `generated/` (committed catalog, coverage, runtime admission and runtime pool).
- `cards/`: catalog CLI (`cli.js`), experiment planners (`pipeline/`), and the reference corpus
  (`reference/fixtures/`: content-addressed manifest, traces and pools) with `reference/check.js`.
- `validation/`: old measurement receipts. Keep them as recorded; do not rewrite paths in them.
- `tools/`: measurement report, runtime admission report, search benchmark.
- `.agents/skills/swu-card-conversion/SKILL.md`: the card conversion workflow (catalog, named
  cases, reference recordings in Holocron, review). Read it before converting cards.
- `forceteki/`: git submodule (`ericfode/forceteki`). The compiler reads its card source and
  its downloaded card JSON.

## Constraints

- **Do not modify anything inside `forceteki/` without asking the user first.** Moving the
  submodule pin is a normal change in this repo; editing its source is not.
- Schema and trace identifiers (`omni-rs-trace-v1`, `omni-rs-pool-v1`,
  `omni-rs-catalogue-report-v1`, `omni-wasm-v1`, `omni-wasm-build-v1`) keep their old names.
  Reference traces and pools are content-addressed by their bytes, and
  `npm run cards:reference -- verify` checks every hash and the exact file inventory.
- `compiler/build.js` hashes every `compiler/*.js` file into `compilerHash`. After any compiler
  change, run `npm run cards:compile` and commit `compiler/generated/`; `cards:check` fails
  otherwise.
- Unsupported rules stay explicit rejections. Never add an approximation that the reference
  corpus cannot check.
- Keep tokens, keys and `.env` files out of Git. Development needs no service credentials.

## Commands

```sh
cargo build --release                 # target/release/kessel, kessel-pimc
cargo test                            # all crates; reference_conformance is slow in debug
npm test                              # cards/ and tools/ tests on committed data, no forceteki
npm run setup                         # submodule + forceteki deps (--ignore-scripts) + card data
npm run cards:test                    # compiler and card tests (needs setup)
npm run cards:check                   # committed compiler/generated/* are current (needs setup)
npm run cards:compile                 # regenerate compiler/generated/*
npm run cards:reference -- verify     # corpus hashes and inventory
npm run wasm:build -- --install-tools && npm run wasm:test
```

`SWU_CARD_JSON` points the Rust card loader (`crates/kessel/src/cards.rs`) at a card JSON
directory; the default is `forceteki/test/json`. One compiler test loads a built forceteki
(`forceteki/build/server`) and skips when it is absent.

New reference recordings are made in Holocron (`npm run kessel:reference -- generate`, or
`npm run kessel:pipeline -- run`, with `KESSEL_DIR` pointing here) and imported here with
`npm run cards:reference -- import --corpus DIR`.
