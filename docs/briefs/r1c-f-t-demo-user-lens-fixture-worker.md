# R1C-F — T-Demo `demo_user_authored_lens_rejects_violating_program` fixture `(S, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-F. Closes 1 unwired T-Demo gate. Reports to R1 Closure Manager.

## Read first

- **[`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md)** — T-Demo row: `demo_user_authored_lens_rejects_violating_program` [ext]. Operationalizes `THESIS.md §"User-defined dimensions"` — proves the ceiling of what gunbc can prove is user-extensible, not compiler-baked.
- **[`THESIS.md §"User-defined dimensions"`](../../THESIS.md)** — thesis claim authority. *"A user writes a lens in `.dag` — e.g., 'max external HTTP calls per workflow' — and the compiler validates every program against it using the same mechanism it uses for built-in dimensions."*
- **[`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag)** — current R1 gate fixtures; existing `user_authored_lens_compiles` gate (lines ~91-97) is the GREEN T-LensAPI dependency. Pattern to mirror.
- **[`src/v3/compiler/tests/t_demo/t_demo_fixtures.dag`](../../src/v3/compiler/tests/t_demo/t_demo_fixtures.dag)** — T-Demo fixture file. Existing T-Demo claims (`fixture_compiler_nerd_canonical`, `fixture_integration_canonical`, `impossible_bug_class_suite_r1`) are the pattern — this brief adds the user-lens demo claim alongside.
- **[`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs)** — runner dispatch.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`THESIS.md`](../../THESIS.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Frame

T-LensAPI's `user_authored_lens_compiles` gate is GREEN — the user-authored lens infrastructure works. The T-Demo gate `demo_user_authored_lens_rejects_violating_program` is the **demo half** of that capability: a fixture program + a user-authored lens that rejects the program, exercising the full proof loop.

The fixture is per-thesis: ~20 lines of `.dag` (the lens) + a violating program + the expected diagnostic. THESIS gives an example: `"max external HTTP calls per workflow"` — a workflow with too many external calls violates the lens; the compiler rejects.

## Single deliverable

Author `demo_user_authored_lens_rejects_violating_program` TestClaim in `t_demo_fixtures.dag` (or sibling). Components:

1. **The user-authored lens** (~20 lines `.dag`): a structural analysis declared in user space (not compiler-baked). Concrete shape per THESIS example: `max_external_http_calls` lens that walks workflow declarations + counts external calls + asserts ≤ N.
2. **The violating program** (~10-20 lines `.dag`): a workflow that exceeds the lens's threshold.
3. **The TestClaim**: predicate likely `FailsWithDiagnostic` (asserts the lens rejects the violating program with a specific diagnostic). Alternative: `LensOutputEquals` if the lens output structure carries the violation receipt.

## Slice — single PR

1. Read THESIS §"User-defined dimensions" for the canonical lens example shape.
2. Author the lens declaration as a separate `.dag` file (or inline in fixture file per existing T-Demo patterns).
3. Author the violating program.
4. Author the TestClaim referencing both (lens + program); pick predicate.
5. Wire runner dispatch.
6. Verify gate evaluates `Pass` (i.e., the lens did reject as expected).

## Acceptance

- [ ] User-authored lens declaration authored (in `.dag`, ~20 lines, structural).
- [ ] Violating program authored (compiles structurally; would emit but for the lens rejection).
- [ ] `demo_user_authored_lens_rejects_violating_program` TestClaim authored + runner dispatch + gate evaluates `Pass` (lens rejects violating program as expected).
- [ ] Demo artifact authored per `docs/r2-structure.md` §"Demo discipline" (running fixture + 1-paragraph "what this demonstrates").
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status table updated to "R1C-F: 1/1 gates green."

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **The user-authored lens infrastructure (T-LensAPI `user_authored_lens_compiles`) regresses while authoring the demo** → STOP. The demo brief is built on the GREEN T-LensAPI dependency; regression there indicates the dependency wasn't actually held.
- **The lens example chosen turns out to require a substrate capability not present** (e.g., the canonical "max external HTTP calls" example requires effect-carrier introspection that v3 doesn't yet have on user surface) → STOP. Escalate per `feedback_audit_adjacent_authority_first`; pick a different lens example that grounds in current substrate, or surface for design clarification.
- **The TestClaim predicate shape doesn't exist yet** (e.g., the gate needs a structural-rejection-with-diagnostic predicate not in DB-15) → STOP. Escalate to R1 Closure Manager (cross-lane to R1C-A schema scoping).
- **DB-8 fixed-point drifts** → STOP.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-F.
- Thesis claim: [`THESIS.md §"User-defined dimensions"`](../../THESIS.md).
- T-LensAPI dependency (GREEN): `user_authored_lens_compiles` at `r1_gates.dag:91-97`.
- T-Demo fixture pattern: [`src/v3/compiler/tests/t_demo/t_demo_fixtures.dag`](../../src/v3/compiler/tests/t_demo/t_demo_fixtures.dag).
- Gate authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) T-Demo row.
- Demo discipline: [`docs/r2-structure.md §"Demo discipline"`](../r2-structure.md).
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
