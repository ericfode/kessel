# CI scope and reuse

CI runs on pull requests, pushes to `main`, and manual dispatch. Branch pushes do
not duplicate PR runs. New PR commits cancel superseded runs; `main` runs finish
so their dependency caches remain available to later PRs. There is no matrix.

All four required check names remain stable. Every job checks out the commit and
runs `.github/scripts/ci-scope.js`. Only a PR whose merge commit has the exact
event base/head parents, and whose entire diff consists of explicitly listed
documentation files, can omit the expensive commands. The job reports success
with the selection reason in its step summary. The Node job always tests the
selector. Missing history, an invalid event, an empty diff, or any unknown path
runs all ordinary gates. Push and manual runs also run all ordinary gates. There are no
workflow-level path filters that could leave required checks pending.

The documentation allowlist lives in the selector. Do not generalize it to all
Markdown: `crates/kessel-wasm/PACKAGING.md` is a package input, and generated
coverage files and reference manifests must be checked. Any CI configuration or
selector change runs all ordinary gates. Renames inspect both old and new paths.

Ordinary runs retain workspace library, binary, integration and documentation tests,
Node tests, corpus hash/inventory verification, pinned-engine compiler/generated-data
checks, and both Node and browser Wasm builds, adapter tests, packing and package
verification. Runtime changes still require the card job because `cards:check` runs
the native release admission generator. Unknown changed files never narrow these
gates; the documentation selector and its finite allowlist are unchanged.

Only two whole accepted-corpus test bodies are deferred to the 200-card checkpoint:

- Rust `every_reference_matches_current_programs_and_immutable_engine_recordings`
  in `crates/kessel/tests/reference_conformance.rs`. The manifest-reader test and
  any future siblings in that target still run.
- Wasm `all reference cases match current Wasm programs and immutable engine recordings`
  in `crates/kessel-wasm/js/test/reference-conformance.test.js`. Future siblings in
  that file still run.

`.github/scripts/ci-tests.js` binds the Rust exclusion to the exact compiled
reference target, then proves the deferred function name occurs exactly once in
Cargo's complete workspace test listing. A collision in another target fails
before runtime tests; it is never silently skipped. The final command remains
Cargo's default workspace test invocation with `--skip NAME --exact`, preserving
workspace feature unification, doctests and target selection. Default
`--no-run` JSON artifacts validate the target inventory and compile-only examples.
New packages and integration targets are included; an unknown runnable kind fails.

Wasm enumerates the same test files as the existing glob, including new files,
and scopes its exact anchored name filter to the one reference file. Registration-
only evaluation of that file records declarations without calling test bodies;
missing/duplicate identities or an unsupported registration harness fail. Similarly
named tests and new siblings still run, and the same name in another Node file
also runs. Real tiny Cargo and Node harnesses exercise these rules, including
workspace feature unification, suffix names, collisions, doctests and examples.

Fixed-fixture coverage is ordinary work: keep Rust
`generic_runtime_tests::generated_programs_match_all_committed_engine_traces`,
all Node/browser core tests, and all adapter tests, including the sixteen expanded
engine games. The terminal-actor test also runs with a genuine checked recording,
pool, catalogue and built package. By default the runner selects the committed
Hera/Grievous capped-damage case with a mandatory postwinner choice; it rejects
missing or ambiguous evidence. An explicit targeted recording must supply all four
`KESSEL_TERMINAL_{TRACE,POOL,CATALOGUE,PACKAGE}` paths. A skipped test does not
establish terminal ingress coverage. Targeted new and affected real-engine
comparisons remain required alongside these ordinary gates.

The full commands remain available unchanged:

```sh
cargo test --locked --workspace
npm run wasm:build -- --install-tools
npm run wasm:test
npm run wasm:pack
npm run wasm:pack:check
```

Run the full clean rebuild and full accepted native/Wasm corpus only at each 200
unique newly reviewed and runtime-admitted cards, using concrete review and
integration packets rather than admission totals or repeated recordings. Dispatch
that check to a background worker on the exact immutable checkpoint revision with
a separate checkout and fresh target/build directories. Preserve its invocation,
input hashes, results and failures; later commits cannot relabel that checkpoint.
There is no automatic counter, committed cadence journal or changed-file waiver.
The existing jobs for an already pushed head must finish unchanged; future cadence
changes do not cancel or bypass them.

The pinned Rust cache action stores dependency builds rather than workspace
binaries or test results. Native optimized tests, native release admission and
Wasm release builds use separate job/profile keys, with compiler and Cargo input
fingerprints. Only successful `main` push jobs save these caches. PRs can read
base-branch caches without publishing caches for other branches. Cargo's locked
builds and all ordinary gates still run on a cache hit; workspace recompilation is expected.
The action sets `CARGO_INCREMENTAL=0`, avoiding throwaway incremental artifacts
on ephemeral runners while retaining the optimized dev profile's assertions and
overflow checks. Cold-build timings therefore also reflect that setting.
The Node version follows `.nvmrc`. The Wasm CLI key uses its exact build-script
pin and runner OS/architecture, and the build verifies the installed version.

The existing engine cache retains the pinned engine's dependencies, build and
downloaded metadata. Its build enables the real engine helper test. The card API
is live, so a cache miss can reveal metadata drift and fail `cards:check`. Do not
regenerate committed catalog files just to make such a failure pass. A fully
immutable metadata snapshot and a platform-aware engine cache migration need a
separate reproducibility change; changing that cache's identity here would force
a fresh API download unrelated to the CI savings.

Wasm tarballs and their receipt remain available for seven days. Uploading already
compressed tarballs without another compression pass reduces redundant work.

To validate workflow changes locally:

```sh
node --test .github/scripts/ci-scope.test.js .github/scripts/ci-tests.test.js
actionlint .github/workflows/ci.yml
git diff --check
```

These standard GitHub-hosted jobs in the public repository are free under
[GitHub's Actions billing policy](https://docs.github.com/en/billing/managing-billing-for-github-actions/about-billing-for-github-actions).
Runner minutes, elapsed time and artifact storage are separate quantities;
fewer runner minutes here do not restore a private-repository minute allowance.
Dependency cache gains depend on hits and compiler cost; targeted replay time is unchanged.
