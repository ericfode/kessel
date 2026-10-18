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
runs the full checks. Push and manual runs always run everything. There are no
workflow-level path filters that could leave required checks pending.

The documentation allowlist lives in the selector. Do not generalize it to all
Markdown: `crates/kessel-wasm/PACKAGING.md` is a package input, and generated
coverage files and reference manifests must be checked. Any CI configuration or
selector change runs the full checks. Renames inspect both old and new paths.

Full runs retain the workspace Rust tests (including strict, compiled,
architecture, proof-boundary and complete reference conformance), Node tests,
corpus hash/inventory verification, pinned-engine compiler/generated-data checks,
and Wasm tests, packing and package verification. Runtime changes also require
the card job because `cards:check` runs the native release admission generator.
No individual cards, corpus recordings or test cases are selected out.

The pinned Rust cache action stores dependency builds rather than workspace
binaries or test results. Native optimized tests, native release admission and
Wasm release builds use separate job/profile keys, with compiler and Cargo input
fingerprints. Only successful `main` push jobs save these caches. PRs can read
base-branch caches without publishing caches for other branches. Cargo's locked
builds and all tests still run on a cache hit; workspace recompilation is expected.
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
node --test .github/scripts/ci-scope.test.js
actionlint .github/workflows/ci.yml
git diff --check
```

These standard GitHub-hosted jobs in the public repository are free under
[GitHub's Actions billing policy](https://docs.github.com/en/billing/managing-billing-for-github-actions/about-billing-for-github-actions).
Runner minutes, elapsed time and artifact storage are separate quantities;
fewer runner minutes here do not restore a private-repository minute allowance.
Dependency cache gains depend on hits and compiler cost; replay time is unchanged.
