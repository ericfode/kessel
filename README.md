# Kessel

Kessel is a deterministic Rust rules simulator and endgame prover for an explicit subset of
Star Wars Unlimited, with the same core compiled to WebAssembly, and a compiler that lowers
[forceteki](https://github.com/ericfode/forceteki) card definitions into the data-only rules IR
the simulator runs. Every supported card is checked against complete games recorded from the
forceteki engine. Unsupported cards are rejected with a reason; they are never approximated.

It started as `bot/omni-rs` in [swu_bot](https://github.com/ericfode/swu_bot) and was moved here
with its history (`git filter-repo`). swu_bot depends on Kessel; Kessel does not depend on swu_bot.

## Layout

| Path | What it is |
| --- | --- |
| `crates/kessel/` | The simulator, prover and trace checker (library `kessel`, binaries `kessel` and `kessel-pimc`). `data/pool.json` is the strict 100-card pool; `tests/` holds the conformance suites and their fixtures. [README](crates/kessel/README.md) |
| `crates/kessel-wasm/` | The WebAssembly build, its packaging scripts, and the JavaScript adapter (`js/`) for Node and browsers. [README](crates/kessel-wasm/README.md) |
| `compiler/` | Card compiler: forceteki TypeScript card source plus card data to the ability IR. `generated/` holds the committed catalog, coverage and runtime admission. [README](compiler/README.md) |
| `cards/` | Card catalog CLI, experiment planners, and the accepted reference corpus (`reference/fixtures/`: 2,108 engine recordings and their 41 pools) with its native/Wasm checker. [README](cards/README.md) |
| `validation/` | Measurement receipts from earlier validation runs (`omni-rs-1`, `omni-wasm-1`, `omni-wasm-2`), kept as recorded. |
| `tools/` | Measurement report (`omni-rs-report.js`), runtime admission report (`card-runtime-report.js`) and the equal-work search benchmark (`omni-search-bench.js`). |
| `forceteki/` | Git submodule, pinned to the same commit as swu_bot. The compiler reads its card source; its card data is downloaded at setup. |

Trace and schema identifiers inside data files (`omni-rs-trace-v1`, `omni-rs-pool-v1`,
`omni-wasm-v1`, ...) keep their original names: committed recordings and pools are
content-addressed by those bytes.

## Requirements

- Rust stable (`cargo`); nothing else is needed to build and test the crates.
- Node 22 (`.nvmrc`) for the compiler, card tooling and Wasm adapter tests.
- For the Wasm build: the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` 0.2.129
  (`npm run wasm:build -- --install-tools` installs both).

## Build and test

```sh
cargo build --release          # target/release/kessel, target/release/kessel-pimc
cargo test                     # unit tests + strict, compiled and reference conformance (slow in debug)
npm test                       # card catalog, planners and tools (committed data only, no forceteki needed)
```

The compiler tests and the staleness check need the forceteki source, its TypeScript
dependency and the card data, which `npm run setup` installs (network: the npm registry and
`admin.starwarsunlimited.com`, fetched by forceteki's own `get-cards`):

```sh
npm run setup                  # submodule, forceteki npm deps (no scripts), card data
npm run cards:test             # compiler and card tests
npm run cards:check            # committed compiler/generated/* match a fresh compile
npm run cards:compile          # rewrite compiler/generated/* after compiler or pin changes
```

WebAssembly:

```sh
npm run wasm:build -- --install-tools
npm run wasm:test
npm run wasm:pack && npm run wasm:pack:check
```

Card browsing: `npm run cards -- summary`, `npm run cards -- show TWI_124`,
`npm run cards:reference -- verify`, `npm run cards:reference -- list --card TWI_124`.

## What stays in swu_bot

Recording new reference games plays bots through the forceteki engine with swu_bot's headless
harness (`bot/env.js`, the bot registry, `bot/omni/rs/state.js`). That side stays in swu_bot:
the reference scenarios and generator (`bot/cards/reference/scenarios/`, `generate.js`, `index.js`),
the probe fixtures and pipeline runner (`bot/cards/pipeline/*-probe-fixture.js`, `run.js`,
`prepared.js`, `cli.js`, `fanout.js`), and the trace and pool generators in `bot/omni/rs/`.
swu_bot generates corpora and Kessel checks and imports them
(`npm run cards:reference -- import --corpus DIR`). `fromEnvGame` in the Wasm adapter takes the
forceteki state exporter as an argument for the same reason.

## Card data

Official card data is not fetched by the build of the crates, but the committed generated
files contain it: `compiler/generated/catalog.json` and `runtime-pool.json`, the reference pools in
`cards/reference/fixtures/pools/`, `crates/kessel/tests/compiled-fixtures/pool.json` and
`crates/kessel/data/pool.json` carry card titles, stats and rules text derived from forceteki's
`get-cards` download. Keep the repository private, or move these to build-time generation,
before publishing it.
