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
