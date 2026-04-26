# R1C-E — T-Emit `.dag` TestClaim wrappers `(S, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-E. Closes 3 unwired T-Emit gates by authoring `.dag` TestClaim wrappers around existing host harness using the PB-Runtime `ExecuteCommand` runner. Reports to R1 Closure Manager.

## Read first

- **[`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md)** — T-Emit row: `emit_rust_fixtures_rustc_green` [ext: `ExecuteCommand`] · `emit_generic_bounds_survive` [ext] · `emit_omni_demo_fixtures_green` [ext: `ForAllTargets` + `ExecuteCommand`].
- **PR #792 (PB-Runtime `ExecuteCommand` worker)** — landed the `ExecuteCommand` runner + `t_pb_b_1_execute_command_boundary` receipt + `TESTING.md` callout. The runner is the enabler for these `.dag` wrappers.
- **`src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs`** — existing host harness for `emit_rust_fixtures_rustc_green` (line ~1199) + `emit_generic_bounds_survive` (line ~379). Read for input fixtures + expected commands.
- **`src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs`** — existing host harness for `emit_omni_demo_fixtures_green` (line ~187). Multi-target (Rust/Python/Go).
- **[`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag)** — current R1 gate fixture file; existing TestClaim patterns to mirror.
- **[`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs)** — runner dispatch + existing `ExecuteCommand` predicate dispatch from PR #792.
- **`feedback_construction_over_ratchets`** — wrappers should ground in observable command-execution + exit-code, not author parallel asserts.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Frame

The T-Emit features ship — host integration harness verifies emitted Rust compiles via `rustc`, generic bounds survive, and the omni demo fixtures emit correctly across Rust/Python/Go. What's missing is `.dag` TestClaim wrappers that make these gates `.dag`-program-defined per the strict-reading R1 close criterion ("the release gate IS a `.dag` program" per THESIS).

PB-Runtime `ExecuteCommand` (PR #792) is the runner enabler. `.dag` TestClaims declare commands + expected exit-codes; the runner executes the bounded command and asserts the outcome. The host harness becomes the **input to** the `.dag` wrapper rather than the gate itself.

## Three consumer-side requirements

1. **Author `emit_rust_fixtures_rustc_green` `.dag` TestClaim wrapper.** Predicate: `ExecuteCommand` (or whatever PR #792 named the runner predicate). Command: `rustc --edition=... <emitted_fixture_path>` or analogous. Expected: exit-code 0 (rustc accepts emitted Rust). Reference the existing host harness's emit-then-rustc loop for fixture set + rustc invocation. Wire runner dispatch if not already covered by PR #792's generic dispatch.

2. **Author `emit_generic_bounds_survive` `.dag` TestClaim wrapper.** This gate verifies that emitted generic bounds carry `impl Fn + Clone` (or the equivalent target-specific bound). The host harness asserts via parsing emitted Rust output + checking bound survival. Whether the `.dag` wrapper uses `ExecuteCommand` or a structural-coverage predicate depends on the audit at brief authoring — `ExecuteCommand` running `grep` on emitted output is one path; a structural predicate is another. **Audit the gate semantics at brief authoring** to pick the simpler shape; prefer `ExecuteCommand` if it cleanly captures the assertion.

3. **Author `emit_omni_demo_fixtures_green` `.dag` TestClaim wrapper.** Multi-target: emit per omni fixture, run target-specific compiler (`rustc` / `python` / `go`), assert exit-code. The `[ext: ForAllTargets + ExecuteCommand]` tag suggests a `ForAllTargets` quantifier that PR #792 may or may not have shipped. **Verify at brief authoring** whether `ForAllTargets` exists; if not, this gate may need R1C-A schema extension OR can be expressed as 3 `ExecuteCommand` claims (one per target) without the quantifier.

## Slice — fixture-by-fixture

1. PR-1: `emit_rust_fixtures_rustc_green` (most direct; single command + exit-code).
2. PR-2: `emit_generic_bounds_survive` (audit-decided shape — `ExecuteCommand` + grep OR structural predicate).
3. PR-3: `emit_omni_demo_fixtures_green` (multi-target; possibly schema-dependent).

Bundle if all three are `ExecuteCommand`-shape; split if any needs schema work.

## Acceptance

- [ ] `emit_rust_fixtures_rustc_green` `.dag` TestClaim authored + runner dispatch + gate evaluates `Pass` (rustc accepts all emitted fixtures).
- [ ] `emit_generic_bounds_survive` `.dag` TestClaim authored + gate evaluates `Pass`.
- [ ] `emit_omni_demo_fixtures_green` `.dag` TestClaim authored + gate evaluates `Pass` (all targets accept emitted output).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status table updated to "R1C-E: 3/3 gates green."

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **`ForAllTargets` predicate doesn't exist yet** AND can't be expressed as 3 separate `ExecuteCommand` claims → STOP. Escalate to R1 Closure Manager (cross-lane to R1C-A schema scoping).
- **`ExecuteCommand` runner from PR #792 doesn't cover the bounded-execution shape needed** (e.g., the gate needs unbounded `output()` that PR #792 explicitly excluded for safety) → STOP. Surface the bounded-runner gap; do not bypass PR #792's safety discipline.
- **Emit regression discovered while authoring wrappers** (existing host-harness assertions fail) → STOP immediately. Wrapper authoring is additive; regression indicates the host harness wasn't actually green.
- **DB-8 fixed-point drifts** → STOP.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-E.
- Runner enabler: PR #792 (PB-Runtime `ExecuteCommand`).
- Existing host harness: `src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs` + `m1_5_emit_omni_demo_test.rs`.
- Gate authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) T-Emit row.
- Schema dependency: possibly [`r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md) for `ForAllTargets` quantifier.
- Discipline anchor: `feedback_construction_over_ratchets`.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
