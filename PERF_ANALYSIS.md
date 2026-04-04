# Build & Test Performance Analysis

_Branch: `perf/build-analysis` — 2026-04-04_

## Timing Summary

All times are wall-clock from macOS. Container overhead is ~15s constant per `cargo` invocation (Colima: 4 CPU, 6GB RAM, 8GB swap).

| Operation | Wall time | Inside container | Notes |
|-----------|-----------|-----------------|-------|
| `cargo check` (cold) | 22s | 8.4s | Includes dependency compilation |
| `cargo check` (incremental) | 15s | 1.1s | After touching one file |
| `cargo clippy` | 24s | 8.7s | |
| **`cargo test` (all)** | **4m41s** | **266s** | 285 pass, 22 ignored |
| → pipeline tests | 3m44s | 225s | 175 tests (**85% of total**) |
| → source_audit | 15s | 0.16s | 35 tests |
| → infer_semantics | 15s | 0.00s | 30 tests |
| → bootstrap | 15s | 0.03s | 1 active test |

## Bottleneck #1: Pipeline Tests (85% of test time)

190 pipeline tests make 178 `compile_dag`/`compile_multi` calls. Each runs the full pipeline from scratch:

```
import resolution → tokenize → parse → normalize → resolve → infer → emit
```

Average: ~1.3s per pipeline test.

**Key finding: single-threaded is FASTER than multi-threaded.**

| Threads | Time | Delta |
|---------|------|-------|
| 1 | 197s | baseline |
| 4 | 220s | +12% slower |
| default | 225s | +14% slower |

This signals memory/cache contention in the 4-CPU / 6GB container. The Rc-heavy allocation pattern and per-test full pipeline re-runs thrash memory.

**Root cause:** No cross-test compilation caching. Common imports (std.types, std.algebra, etc.) are re-tokenized, re-parsed, re-resolved, re-inferred, and re-emitted in every single test. The module_index (file→path lookup) is cached via OnceLock, but no compiled artifacts are reused.

## Bottleneck #2: Container Overhead (15s constant tax)

Every `cargo` invocation pays ~15s:
- Colima VM startup/connection
- Toolchain sync check ("downloading 5 components")
- Docker layer negotiation

This means `cargo check` after a one-line edit costs 15s regardless of the 1.1s actual compilation.

## Bottleneck #3: Generated File Sizes

| File | Size | Lines |
|------|------|-------|
| `v2_compiler_parse.rs` | 356KB | 11,032 |
| `v2_compiler_emit_rust.rs` | 284KB | 4,550 |
| `v2_compiler_infer.rs` | 164KB | 3,364 |
| `v2_compiler_complexity.rs` | 148KB | 3,199 |
| **Total stage0** | | **41,926** |

`v2_compiler_parse.rs` at 356KB is 2.2x the next-largest file and dominates cold build time.

## Improvement Opportunities

### 1. Cross-test compilation cache (highest impact)

Cache tokenize+parse+resolve results for imported modules across tests. Each pipeline test re-compiles the same imports. A shared cache keyed on `(module_path, content_hash)` would avoid re-doing work.

**Estimated savings:** 60-80% of pipeline test time (120-180s).

### 2. Force single-threaded test execution

Set `--test-threads=1` for pipeline tests. Already measured: saves ~30s.

### 3. Reduce container overhead

The 15s tax dominates the inner loop for incremental development. Options:
- Keep a warm container with cargo daemon
- Cache toolchain sync more aggressively
- Run cargo natively for quick checks

### 4. Split large generated modules

`v2_compiler_parse.rs` at 11K lines could be split at the generator level to improve incremental compile granularity.

## Using the Complexity Analyzer

`complexity.dag` defines symbolic cost algebra with CostShape, SizeExpr, CostExpr, and Work/Span metrics. To identify which pipeline _stage_ dominates per-test cost, the complexity analyzer's `method_cost_shape` dispatch could be instrumented to emit per-stage timings. The 19 intrinsic methods already have CostShape entries — extending this to pipeline stages would answer "is it inference or emission that's slow?"
