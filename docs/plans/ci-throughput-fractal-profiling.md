# CI throughput — fractal profiling plan

Owner: calm-carp-204 ("CI profiling"). Started 2026-06-26.
Purpose: drive CI wall-clock down by *attributing and killing* cost, not by
raising caps or scheduling bloat efficiently. A small compiler should not need
8 GiB or 20 minutes; both numbers are suspects until decomposed.

## Why this is worth it (the displaced cost)

CI wall-clock (~15-25 min) is paid on every push, on every PR, by every session
in the fleet. It is the single largest recurring time-tax we control. The OOM fix
bought *green* by running the floor *narrow* (width 1-3), so "green" and "fast"
diverged — the slowness is partly the direct cost of the safety fix. The fix is
not "tolerate it" but "make each shard cheap enough that wide is also safe."

## The method: fractal profiling

§2 deep-decomposition applied to cost. At each level:
1. **Measure the whole** by execution (wall-clock for time, phase-peak VmHWM for space).
2. **Attribute to sub-parts** with a control — never "this is probably X."
3. **Recurse into the largest** contributor.
4. **Stop** only at a grounded-irreducible cost or a *named bug*.

Two axes, because they trade off: **TIME** (wall-clock) and **SPACE** (RSS).
Narrow-for-memory buys time-cost; that is why a space win is also a time win.

### Honesty invariants (how we keep ourselves honest)

- **No "baseline."** A number is a suspect until decomposed to grounded-irreducible
  or a named bug. "2 GiB/shard" and "20 min" are suspects, not floors.
- **Phase-peak, not alloc-backtrace.** Peak RSS is the *sum* of live allocations;
  the biggest single allocation is not the peak driver. (This already misled us
  once — the tracking allocator pointed at the cache when the precompute was the
  cause.) Use phase RSS checkpoints (VmHWM) bracketing each stage.
- **By-execution + control for every attribution.** A win is the *measured* number
  dropping, with a discriminating control — never a projection.
- **Don't mask.** Never raise a cap, widen scheduling, or relax a budget to cover
  an un-attributed cost. That converts a measurable bug into permanent overhead.
- **Each level names a numeric target and a falsifier.**

## The decomposition (levels)

### L0 — CI wall-clock (root)
`run = queue + build + max(floor_job, rust_tests_job)` (the two jobs run in parallel).
Instrument one full run; attribute the ~20 min across queue / build / floor-run /
slow-tests. **Target:** know where the time goes. **Falsifier:** the parts sum to
observed wall-clock.

### L1 — the floor job
`floor = build + claim_executor_run`. The run = `base_resolve + width × per_shard_marginal`
(the cgroup counts total RSS = shared base counted once + per-shard deltas).
Attribute base vs marginal vs scheduling. **Target:** separate the shared-base cost
from the per-shard cost (decides whether base-sharing or per-shard reduction is the
bigger lever). **Falsifier:** `base + width×marginal` reproduces measured peak across
two widths.

### L2 — per-shard resolve (the 2 GiB suspect) — PRIORITY
Attribute one shard's peak RSS to:
- **(a) env-merge duplication** — measured: ~73% of a resolved graph is duplicated
  merged closure (`type_env` 54% + `func_env` 19%). This is Layer A's target.
- **(b) base-resolve redundancy** — *hypothesis*: each discovery shard resolves its
  witness against the full `dsl + src/v2` roots; if `std`/`extdeps` is re-resolved
  per witness instead of resolved once and shared, that is the *same §2 irrelevant-
  work pattern* as the precompute bug (#5833) — the whole-tree cost paid N times.
  Potentially a *bigger* lever than (a); not yet measured.
- **(c) genuine** witness-closure cost.

Method: phase-peak VmHWM around `build_type_env` / `merge_envs` / the import-closure
resolve; `Rc::ptr_eq` probes for sharing; a small-closure witness vs a large-closure
witness to separate base from marginal. **Target:** per-shard 2 GiB → ~200-300 MB.
**Falsifier:** `a + b + c` sums to the measured 2 GiB, *and* landing Layer A drops the
measured number by (a)'s predicted share (by execution, not projection).

### L3 — rust_tests slow cases (separate fat chunk)
Individual nextest cases run >480-615s (parse-table memo, typescript typecheck,
floor-skip discovery). Attribute which dominate; recurse into the worst.
**Target:** top-5 wall-clock tests named + worst root-caused. **Falsifier:** sum of
slow tests ≈ `rust_tests` wall-clock − build.

### L4 — build + cache
`cargo build -p v1-compiler --release --bins` + sccache (flaky — the original
complaint). Attribute cold vs warm; sccache hit-rate. (Layer B helps the *resolve*
cache, not rustc.) **Target:** warm-build floor + sccache reliability number.

## Sequencing (do not pour effort downstream of an un-measured level)

1. **L0+L1+L2 profiling pass FIRST** — attribute the 2 GiB and the wall-clock. This
   is "Move 1." No scheduling or restructure effort before this lands a number.
2. **Layer A** — kills the env-dup share of per-shard (in flight; the load-bearing
   spine flip is operator-gated). Re-measure L2 to confirm the predicted drop.
3. **Base-resolve sharing fix** — *only if* L2 confirms redundancy (b). Could be the
   biggest single lever; would be a new piece of work.
4. **Resource-aware scheduler** (Node A-D, currently UNOWNED — gentle-newt-542
   archived) — *only after* per-shard is honest. `width = memory / per_shard` gives
   wide parallelism for free once a shard is ~300 MB; resurrecting it on a 2 GiB
   bloated shard just schedules bloat efficiently.
5. **L3 slow-tests + L4 build/cache** — independent of the floor; can run in parallel.

## Status ledger (kept current)

- L2(a) env-dup: **measured** (~73%); fix = Layer A (guard landed+proven; spine flip
  operator-gated).
- L2(b) base-resolve redundancy: **unmeasured hypothesis** — Move 1 confirms or kills it.
- Floor OOM *for green*: budget fix #5836 (merged) + emit-host isolation #5837 (in CI)
  — narrow-but-green. Throughput (wide) is what this plan is about.
- Resource-aware scheduler: **unowned** (owner archived).
- L3 slow tests, L4 build/cache: **unowned**, unmeasured.
