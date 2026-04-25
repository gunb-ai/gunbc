# PB-1-b — `STAGED_FILES` data-bootstrap worker brief `(M)`

> **Worker brief** for PB-1 sub-lane b. Reports through Zero-Floor
> Program Manager (`stern-swift-335`) per
> [`docs/briefs/pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md)
> (now PB-1 brief amended 2026-04-25 for 0-floor framing per merged
> [#770](https://github.com/gunb-ai/gunbc/pull/770)).

## Read first

- **[`docs/briefs/pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md)** — the PB-1 program brief. Read in full. This worker brief does not re-author PB-1's work definition; it dispatches sub-lane PB-1-b specifically and inherits the program brief's scope, acceptance, and STOP-AND-ESCALATE conditions.
- [`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs) `:100-115` — **the proof-of-concept already in place**: PB-1-a landed via the `std_fixture_bootstrap_snapshot()` pattern (not as a separate retired brief; it's the shape PB-1-a converged on in R1 era). PB-1-b extends the same pattern.
- [`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs) `:131-150` — `load_runtime_bootstrap_authorities` still parses `STAGED_FILES` + `V3_SPECS` + `COMPILER_FILES` at runtime via `load_fixtures`. PB-1-b retires the `STAGED_FILES` portion of this; PB-1-c/d retire the other two.
- [`src/v3/compiler/build.rs`](../../src/v3/compiler/build.rs) — current build-script-time generation pattern. Reuse for PB-1-b's snapshot.
- [PR #780](https://github.com/gunb-ai/gunbc/pull/780) — proven shape for build-script-time `.dag` → generated-Rust pattern (different lane, but the discipline carries over: SG-0 producer-owned-partition cementing, REGEN_OUTPUTS registration).

## Slice — `STAGED_FILES` data-bootstrap

`STAGED_FILES` enumerates `src/v3/std/*.dag` (per `build.rs:35`); these are runtime-parsed today via `load_runtime_bootstrap_authorities`. PB-1-b migrates this slice to the same shape as PB-1-a's `std_fixture_bootstrap_snapshot()`:

- Build-script-time: extend `build.rs` to compile the `src/v3/std/*.dag` files into a generated snapshot module (e.g., `staged_fixture_bootstrap_snapshot.rs` or extension of an existing snapshot module — worker's call which fits cleaner).
- Runtime: replace the `STAGED_FILES`-side branch of `load_runtime_bootstrap_authorities` with a call to the generated snapshot loader, the same way `Dag::std_fixture_bootstrap_snapshot()` works.
- No change to `V3_SPECS` / `COMPILER_FILES` in this sub-lane — they continue runtime-parsing under PB-1-c/d.

## Inheritance from PB-1 brief

This worker brief inherits **all of**:

- Acceptance criteria from [`pb-1-data-driven-bootstrap.md` §"Acceptance"](pb-1-data-driven-bootstrap.md), specifically:
  - `cargo test -p v3-compiler` passes; bootstrap still primes Dag correctly.
  - **DB-8 `self_host_fixed_point` converges bit-identically.** No-compromise gate.
  - SG-0 census updated as needed: any retired hand-Rust off the list, generated snapshot files in REGEN_OUTPUTS partition.
- STOP-AND-ESCALATE conditions from the program brief, especially the bit-identical DB-8 convergence requirement.
- Non-goals — no `tokenize.rs` / `parse.rs` / `lower.rs` deletion **at the file level** in this sub-lane (those are separate PB-4/5/6 lanes per the 2026-04-25 amendment); not a binary-blob format; no `.dag` user-facing syntax change.

## What's different post-amendment

The merged [#770](https://github.com/gunb-ai/gunbc/pull/770) inverted three PB-1 non-goals under 0-floor:
- Tokenize/parse/lower/infer/emit retirement is in scope for **the program** (under PB-4/5/6/etc.), no longer non-goal — **but still not PB-1-b's work**. PB-1-b stays narrow.
- `Dag` runtime format is allowed to evolve as `substrate.dag` evolves under PB-Substrate. PB-1-b's generated snapshot must keep pace with whatever `Dag` shape exists at landing time.
- `Dag` builder API is no longer a fixed surface. Same caveat — PB-1-b uses whatever exists.

The retained non-goals (constructor-emission over binary blob; no `.dag` user-facing syntax change) are PB-1-b's bounded scope.

## Acceptance (PB-1-b-specific deltas to inherited program acceptance)

- [ ] `STAGED_FILES`-side runtime parse retired in `load_runtime_bootstrap_authorities`; generated snapshot loader replaces it.
- [ ] Generated snapshot module lands at compile time via `build.rs`; runs without manual invocation.
- [ ] Generated module registered in `REGEN_OUTPUTS` (SG-0 partition cementing — same pattern as PB-Substrate v2 #780 inherits).
- [ ] **`Dag::new()` measurably faster on the `STAGED_FILES`-attributable portion.** Worker measures pre/post and surfaces in PR description.
- [ ] DB-8 `self_host_fixed_point` converges bit-identically.
- [ ] All inherited program-brief acceptance criteria.

## STOP-AND-ESCALATE (delta to inherited)

Surface to Zero-Floor Manager.

- **If the generated snapshot's serialization shape diverges from `std_fixture_bootstrap_snapshot()`'s** — STOP. PB-1-a's pattern is the precedent; divergence means the pattern doesn't generalize and that's its own design call.
- **If `STAGED_FILES`'s `.dag` files contain constructs the current compiler can't fully self-emit** — STOP. Same condition as in the program brief; surface the gap.
- **If DB-8 fixed-point drifts** — STOP immediately. Same no-compromise gate.
- **If the generated snapshot module's size triggers rustc OOM or build-time issues** — STOP. Per program brief: propose split or alternate format.

## Reporting

- Single PR. Title pattern: `feat(v3): PB-1-b — STAGED_FILES data-bootstrap snapshot (extends PB-1-a pattern; closes runtime-parse on src/v3/std/*.dag)`.
- PR description: cite this brief + the program brief; cite the bootstrap-time speedup measurement; cite SG-0 census deltas (any).
- On merge: Zero-Floor Manager confirms PB-1-b closure to Director; PB-1-c (V3_SPECS) becomes the next dispatch.

## Cross-manager note

No cross-manager signal needed. PB-1-b is internal to PB-1 program scope; bootstrap snapshot shape doesn't alter `Dag` runtime format in a way that affects Grounding or other managers.
