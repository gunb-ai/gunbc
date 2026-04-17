> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md)

# P1-L1: L1.5 tail — ownership Phase 2 + ignore hygiene

**Phase:** 1 (Foundation)
**Time budget:** ~1 week (one full implementer-week; split into ~5 commits)
**Status:** Plan. No code changes yet.

---

## Motivation

Half A (PR #489) and Half B (PR #490) close the L1.5 ownership substrate
but leave six `#[ignore]`d tests that are explicitly gated on
*"Phase 2 lands"* or *"Go target lands"*. These are not gaps — they're
forward references that pay off in days, not milestones.

Meanwhile, new coproducts landed in Half B (`ComputationModel`,
`TargetExecutionModel`, `ParameterDisposition`, `MemoryModel`,
`ScopeModel`, etc.) and each needs a 🟢/🟡/🔴 receipt per modeling
discipline §4. Some may be missing; audit closes it.

This lane is the "sweep up" before P2 starts.

---

## Scope

Five work items, roughly ordered:

### 1. ParameterContract::Consumed → pass-by-value in emit_rust
Today emit_rust renders every parameter as `&T` (borrowed). Half B's
`ParameterDisposition` substrate fact now distinguishes `Borrowed` from
`Consumed`, but the renderer still treats all params as borrowed.

**Change:** `render_function_declaration` in `emit_rust.rs` reads each
parameter's `ParameterDispositionBinding` from the `CallableRealization`
and emits `T` (by-value) when `Consumed`, `&T` when `Borrowed`.

**Unignores:**
- `four_fixture_id_phase2_consumed_contract`
- `four_fixture_wrap_phase2_consumed_contract`

### 2. Go cross-target test activation
Four tests in `four_fixture_regression_test.rs` are `#[ignore]`d pending
"Lane 3 / Track 2 (go.dag) lands". Half B landed `go.dag` and
`emit_go.rs`. Verify the fixtures roundtrip through Go; unignore.

**Unignores (×4):**
- `four_fixture_id_cross_target_go_placeholder`
- `four_fixture_drop_cross_target_go_placeholder`
- `four_fixture_wrap_cross_target_go_placeholder`
- `four_fixture_is_empty_cross_target_go_placeholder`

### 3. Receipts audit on new coproducts
For every `type X = A | B | ...` declaration added in Half A or Half B
(across `src/v3/std/computation_model.dag`, `src/v3/spec/rust.dag`,
`src/v3/spec/go.dag`, `src/v3/spec/python.dag`, `src/v3/std/emit_model.dag`),
verify the declaration carries a 🟢/🟡/🔴 receipt with one-line
justification.

**Command:** `grep -nE "^type [A-Z].*=" src/v3/std/*.dag src/v3/spec/*.dag`
→ for each hit, adjacent comment block must contain a receipt glyph.

### 4. m1_3_emit_rust_test compile-once coverage
Half A applied the compile-once-run-many pattern to the three lens
migration tests. `m1_3_emit_rust_test.rs` still has ~10 rustc roundtrip
tests that each compile a fresh wrapper. Where tests share wrapper
shape, fold them into a single harness call (same pattern as
`RustcHarness` in `tests/common/`).

**Target:** cut CI time on m1_3_emit_rust_test from ~12s to ~4s.

### 5. Delete `#[allow(warnings)]` from test wrappers
The emitted-code `#[allow(warnings, clippy::all)]` attributes currently
in `m2_lens_cost_migration_test.rs`, `m2_lens_provenance_migration_test.rs`,
`m2_lens_unused_parameters_migration_test.rs`, and `m1_3_emit_rust_test.rs`
were band-aids. P1-L2 will solve the underlying warnings structurally.

**Do NOT remove these attributes in this lane.** P1-L2 owns that
transition. This item is listed so implementers don't propagate more
`#[allow]` attributes during the Consumed work above.

---

## Out-of-scope

- Warning fixes at the emit_rust source level — that's P1-L2.
- Any new substrate facts — if Consumed rendering reveals a gap in
  `rust.dag`'s `CallableRealization`, escalate (see below).
- Any emit_go.rs or emit_python.rs refactoring — those consolidate in P2.
- New target languages.

---

## Direction

The lane's goal is **close visible gates, don't open new ones**. Every
change should reduce the count of `#[ignore]`d tests without adding
new scaffolding.

Work items are independent. The likely sequence: Consumed rendering
first (biggest item), Go unignores second (should be mechanical
verification), receipts audit third, m1_3 perf fourth. The #[allow]
note is informational for the working implementer.

---

## Escalation criteria

Stop work and surface if:

1. **Consumed rendering needs new substrate facts** — e.g., lifetime
   annotations, move-out tracking, borrow-checker escape hatches. The
   ownership substrate should be sufficient. If the renderer needs new
   facts to emit correct Rust, that's a real gap and deserves a
   separate design pass, not a tactical extension here.

2. **Go cross-target tests fail for reasons other than "not yet wired"**
   — e.g., emit_go produces broken Go, semantics differ from Rust. Those
   are Half B debt (B8 ListFold scoping, etc.) that should have been
   fixed before landing. Surface to Half B owners, don't patch here.

3. **Receipts audit reveals 5+ missing receipts on live substrate**
   — that indicates modeling-discipline drift across the whole codebase,
   not just new work. Surface as a standing concern; this lane only
   catches Half A/B additions.

4. **m1_3 compile-once harness requires refactoring
   `tests/common/RustcHarness`** — the harness is shared infrastructure.
   Changes there affect all migration tests. Surface before touching.

---

## Acceptance gates

Lane is done when all five hold:

- `cargo test -p v3-compiler` passes with **6 fewer** `#[ignore]`d
  tests than at lane start.
- Every `type X = ... | ...` declaration in `src/v3/std/` and
  `src/v3/spec/` has a 🟢/🟡/🔴 receipt.
- m1_3_emit_rust_test runs in <5s local (was ~12s).
- No new `#[allow(...)]` attributes added anywhere.
- CI green on half-a-migrations after merge OR on a fresh PR against
  main.

---

## Dependencies

- **Requires:** Half A (#489) and Half B (#490) merged to main.
- **Blocks:** P1-L2 (clean-emission invariant uses the post-Consumed
  renderer as its starting point).
- **Does not block:** P1-L3 (design-only, independent).

---

## Estimate

- Consumed rendering: 2 days (including test verification)
- Go unignores: 0.5 day (mechanical)
- Receipts audit: 0.5 day
- m1_3 compile-once: 1 day
- Integration + PR prep: 1 day

Total: ~5 implementer-days.
