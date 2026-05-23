# Changelog

## [0.1.1] - 2026-05-23

### Fixed

- docs.rs build for v0.1.0 failed with `Error: writing compile_commands.json
  / Caused by: Read-only file system (os error 30)` because `build.rs` wrote
  `compile_commands.json` into the crate root, which the docs.rs sandbox
  mounts read-only. The per-build `compile_commands.json` emission is removed
  in favor of a committed `.clangd` at the crate root that gives clangd the
  flags needed for files in `src/` (where the wrapper lives) and suppresses
  diagnostics under `vendor/` (vendor code is not maintained here). build.rs
  now writes nothing outside `OUT_DIR`, so docs.rs is happy without any
  env-var gating.

## [0.1.0] - 2026-05-23

Initial release.

- Vendored [bsalita/ddss](https://github.com/bsalita/ddss) (a DDS 2.9.0 fork
  with the `CalcAllTablesPBNx` dynamic batch API and a persistent internal
  thread pool) as a git submodule, pinned to a commit on the fork's `develop`
  branch.
- bindgen-generated bindings to ddss's `dll.h`, including the fork-specific
  `CalcAllTablesPBNx`.
- `cc`-based build of the 27 ddss `src/*.cpp` translation units with
  `-DDDS_THREADS_STL` (std::thread backend) and C++17.
- Smoke tests covering the legacy `CalcDDtable` API and the new
  `CalcAllTablesPBNx` API on the four-13-card-straight-flushes deal.
- `compile_commands.json` emission for clangd.
- `debug-dump` Cargo feature gating ddss's `dump.txt`-on-error behavior.
- Additional Cargo features `debug-top-level`, `debug-ab-stats`,
  `debug-tt-stats`, `debug-timing`, `debug-moves`, each gating the
  corresponding `DDS_*` macro in the ddss vendor and emitting per-thread
  `.txt` diagnostic files into the cwd. All off by default.
- `.gitignore` covers editor scratch (`.vscode/`, `.cache/`) and the
  per-thread DDS dump filenames produced by the debug features.
