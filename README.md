# ddss-sys

Generated bindings to [bsalita/ddss](https://github.com/bsalita/ddss), a
performance-oriented fork of [DDS](https://github.com/dds-bridge/dds), the C++
double dummy solver for contract bridge. The fork is based on DDS **2.9.0** and
its headline addition is `CalcAllTablesPBNx`, a dynamic-size batch API that
accepts any number of deals in one call and internally chunks and schedules
across a persistent thread pool. The author reports ~1.44× batched throughput
over the same-version upstream.

This is a separate crate from [`dds-bridge-sys`](https://crates.io/crates/dds-bridge-sys)
and is **not** a drop-in replacement for it:

- ddss is based on DDS 2.9 (global state + internal thread pool), not DDS 3.x.
  There is no `SolverContext` C++ class and no per-thread context C shim.
- ddss bumps `MAXNOOFTABLES` (40→1000) and `MAXNOOFBOARDS` (200→5000), so the
  legacy batch structs (`boards`, `boardsPBN`, `ddTablesDealsPBN`, `solvedBoards`,
  …) have larger sizes here than in upstream DDS and are not ABI-compatible.

Linking both `dds-bridge-sys` and `ddss-sys` into the same binary is not
supported — they export the same C symbols (`SetMaxThreads`, `SolveBoard`,
`CalcDDtable`, …) and will collide.

## Status

ddss is a single-maintainer fork (1 star, 0 forks at the time of writing).
The vendored submodule is pinned to a specific commit on the `develop` branch.
Treat this crate as **experimental**.

## Usage

The library needs manual initialization — call
[`SetMaxThreads`](https://docs.rs/ddss-sys/latest/ddss_sys/fn.SetMaxThreads.html)
once before any other API:

```rust
// 0 stands for automatic configuration based on available cores
unsafe { ddss_sys::SetMaxThreads(0) };
```

ddss inherits DDS 2.9's threading model: the legacy entry points are not
reentrant. If you call them from multiple threads, serialize the calls
with a mutex.

### The fork-specific batch API

```rust
// PBN deal: "<dealer>:<N hand> <E hand> <S hand> <W hand>",
// each hand as "spades.hearts.diamonds.clubs". The cards[80] buffer
// is null-terminated. Here N has all clubs, E all diamonds, S all
// hearts, W all spades.
const PBN: &[u8] = b"N:...AKQJT98765432 ..AKQJT98765432. .AKQJT98765432.. AKQJT98765432...\0";

let mut deal = ddss_sys::ddTableDealPBN::default();
assert!(PBN.len() <= deal.cards.len());
for (dst, &b) in deal.cards.iter_mut().zip(PBN.iter()) {
    *dst = b as _;
}

unsafe { ddss_sys::SetMaxThreads(0) };

let mut deals = [deal];
let mut trump_filter = [0; ddss_sys::DDS_STRAINS as usize]; // 0 = solve every strain
let mut results = [ddss_sys::ddTableResults::default(); 1];

let status = unsafe {
    ddss_sys::CalcAllTablesPBNx(
        deals.len() as i32,
        deals.as_mut_ptr(),
        -1, // no par; pass 0..=3 to compute par with the given vulnerability
        trump_filter.as_mut_ptr(),
        results.as_mut_ptr(),
        core::ptr::null_mut(), // par output (null is fine when mode == -1)
    )
};
assert_eq!(status, ddss_sys::RETURN_NO_FAULT as i32);
```

`CalcAllTablesPBNx` accepts batches of any size: it splits internally into
fixed-size chunks bounded by `MAXNOOFTABLES` per call to the worker.

## Cargo features

All features are off by default. The `debug-*` features each gate the
corresponding `DDS_*` C++ macro in the ddss vendor; enabling them causes the
solver to write `.txt` diagnostic files into the current working directory at
runtime. Intended for solver development, not production use.

- `debug-dump` — let DDS write `dump.txt` on solver errors
- `debug-top-level` — top-level AB call info → `toplevel*.txt` (per thread)
- `debug-ab-stats` — alpha-beta search stats → `ABstats*.txt` (per thread)
- `debug-tt-stats` — transposition-table memory usage → `TTstats*.txt` (per thread)
- `debug-timing` — function timings → `timer*.txt` (per thread)
- `debug-moves` — move-generation quality → `movestats*.txt` (per thread)

## License

Apache-2.0, matching upstream DDS and ddss.
