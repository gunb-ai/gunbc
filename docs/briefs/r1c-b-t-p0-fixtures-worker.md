# R1C-B — T-P0 fixture authoring `(S, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-B. Closes 3 unwired T-P0 gates under strict reading of [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md). Reports to R1 Closure Manager.

## Read first

- **[`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md)** — T-P0 row: `p0_repeat_string_correct` [Day 1, structural receipt pending per ROADMAP] · interim `p0_repeat_string_v2_oracle_rust_bridge` (`.dag` + v2-oracle integration) · `p0_no_fabrication_sentinel` [ext] · `p0_rest_ops_aligned` [ext]. The Day-1 gate uses today's DB-15 schema; the two `[ext]` gates require T-TestGen schema extensions per ROADMAP authority.
- **[`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md)** — manager scope; verify dependency on R1C-A (schema extensions) at brief authoring before dispatching the `[ext]` gate fixtures.
- **[`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag)** — current R1 gate fixture file. Existing TestClaim authoring patterns to mirror for the new fixtures.
- **[`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs)** — runner-dispatch table; new gate predicates need dispatch arms here.
- **`feedback_construction_over_ratchets`** — fixtures should ground in observable behavior, not author parallel asserts.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Frame

T-P0 features already work. `repeat_string_correct` is verified by integration test against the v2 oracle. The `no_fabrication_sentinel` and `rest_ops_aligned` features are similarly landed but lack `.dag` TestClaim wrappers. This brief authors three fixtures + runner-dispatch wiring.

The Day-1 structural fixture (`p0_repeat_string_correct`) is pending modeled evaluation in v3 (see ROADMAP T-P0 note). The interim bridge (`p0_repeat_string_v2_oracle_rust_bridge`) uses existing DB-15 schema (`OutputEquals` on a lowered literal + v2-oracle integration). The two `[ext]` fixtures depend on R1C-A's predicate-shape work IF they need new predicates beyond the DB-15 surface; **verify at brief authoring** by reading the integration test for each feature to determine the predicate shape needed.

## Three consumer-side requirements

1. **Author structural `p0_repeat_string_correct` TestClaim** in `r1_gates.dag` (or sibling fixture file) — a `.dag` receipt that evaluates **modeled** `repeat_string` behavior (not a self-referential literal witness). Until that lands, `p0_repeat_string_v2_oracle_rust_bridge` in `r1_gates.template.dag` is an explicitly named **interim** coupling (literal + `OutputEquals` + `p0_std_render_repeat_string_test` v2 oracle); **dissolution** when the structural gate supersedes it (comment in template). Predicate options for the structural gate: `Compiles` and/or `OutputEquals` once the claim program can surface a computed witness. Wire runner dispatch in `test_runner.rs` as needed. Existing integration test `tests/integration/p0_std_render_repeat_string_test.rs` is the reference for input + expected output. **Day-1 unblocked** for the interim bridge (uses DB-15 schema).

2. **Audit `p0_no_fabrication_sentinel` predicate shape.** Read the existing P0 sentinel-fix code in `src/` (search for sentinel-removal evidence). Determine: does this need a new predicate (e.g., `NoSentinelPresent`) that R1C-A scopes? OR can it be expressed in existing DB-15 surface (e.g., `FailsWithDiagnostic` if the fabrication check is a compile-error path; `Compiles` if the absence-of-sentinel is structural)? **Branch on audit:**
   - **DB-15 surface suffices:** author fixture immediately, wire runner dispatch.
   - **New predicate needed:** STOP and coordinate with R1C-A worker (predicate shape lands there); this brief blocks until R1C-A scopes.

3. **Audit `p0_rest_ops_aligned` predicate shape.** Same audit shape as #2. The REST_OPS alignment feature is structural (transports align with operations). Determine predicate: existing `Compiles` / `OutputEquals` may suffice, or a new structural-coverage predicate may be needed. **Branch on audit similarly to #2.**

## Slice — fixture-by-fixture

1. PR-1: interim `p0_repeat_string_v2_oracle_rust_bridge` (literal witness + `OutputEquals` + integration + `test_runner` suite); structural `p0_repeat_string_correct` remains follow-up until v3 can witness modeled `repeat_string`.
2. Audit step for `[ext]` predicates — captured in PR-1 description or follow-up issue. Determines whether PR-2 + PR-3 dispatch immediately or block on R1C-A.
3. PR-2: `p0_no_fabrication_sentinel` fixture (post-audit; possibly post-R1C-A schema landing).
4. PR-3: `p0_rest_ops_aligned` fixture (post-audit; possibly post-R1C-A schema landing).

Bundle PR-1 + PR-2 + PR-3 if all three are DB-15-suffices; split if any blocks on R1C-A.

## Acceptance

- [ ] `p0_repeat_string_correct` structural `.dag` TestClaim + runner dispatch + gate evaluates `Pass` (interim `p0_repeat_string_v2_oracle_rust_bridge` landed until superseded).
- [ ] `p0_no_fabrication_sentinel` `.dag` TestClaim authored + runner dispatch + gate evaluates `Pass`.
- [ ] `p0_rest_ops_aligned` `.dag` TestClaim authored + runner dispatch + gate evaluates `Pass`.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status table updated to "R1C-B: 3/3 gates green."

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **Audit reveals `[ext]` gates need a new predicate shape not in R1C-A's scope** → STOP. Escalate to R1 Closure Manager (cross-lane scoping coordination); R1C-A worker may need scope expansion.
- **Existing P0 feature regressed** (oracle test fails after fixture authoring) → STOP immediately. The fixture authoring is supposed to be additive; behavioral regression indicates the feature wasn't actually closed.
- **DB-8 fixed-point drifts** → STOP. Re-author fixtures structurally rather than ratcheting.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-B.
- Gate authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) T-P0 row.
- Schema dependency: [`docs/briefs/r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md) (when authored — possibly).
- Existing integration tests: `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs` + analogous P0 tests.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
