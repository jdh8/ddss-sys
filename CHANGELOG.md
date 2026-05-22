# Changelog

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
