# R1C-A — T-TestGen schema extensions `(M-L, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-A. Critical-path enabler — unblocks R1C-D (T-PB census-as-`.dag`). Reports to R1 Closure Manager.

**Closure status:** **COMPLETE on main.** Aggregated receipts live in [`r1-closure-manager.md`](r1-closure-manager.md) (R1C-A row + working-state table): Sub-B **PR #939**; mock-backed gate **`3a18fa80b`** + `r1_mock_backed_invariant_gate.dag`; Sub-A list lowering tied to substrate/list work (e.g. `5bf0ec8d0`). The **Pre-author audit receipt** section below is historical context from brief authoring time.

## Read first

- **[`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md)** — manager scope + lane structure + dependency edge `R1C-A → R1C-D`.
- **[`docs/briefs/r2-manager-brief-authority-matrix.md`](r2-manager-brief-authority-matrix.md)** — local review checklist (status consistency + pre-author verification invariants apply).
- **[`docs/escalation-paths.md`](../escalation-paths.md)** — escalation channel + decision-artifact discipline.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Pre-author audit receipt (per matrix verification invariant)

Pre-author grep against main HEAD `407a8bcb1` established:

- **`MockBackedInvariant` runner** lives at `src/v3/compiler/src/test_runner.rs:1432-1448`. Currently returns `NotYetImplemented` when `claim.requires.is_empty()`. Reason: *"`TestClaim.requires` is empty — DB-15 mock obligations attach only on `requires` as `ResourceReference` edges; hermetic subject/invariant application succeeded but is not a mock-backed receipt until at least one obligation is declared (M1(2.8): list bodies in fixture `TestClaim` data are not expressible yet)."* The runner expects `requires: List<ResourceReference>` populated; blocker is two-part — (a) M1(2.8) data-body shape + (b) downstream materialization once non-empty.
- **`ResourceReference` schema** already exists at `src/v3/std/resources.dag:25-26` (`type ResourceReference { target: DeclarationRef }`); `TestClaim.requires: List<ResourceReference>` is already declared in the DB-15 schema at `src/v3/std/verification.dag:187`. No new schema required for `MockBackedInvariant` itself.
- **M1(2.8) data-body restriction** at `src/v3/compiler/src/lower.rs:2446` rejects list-literal data bodies as `ValueBody::Unparsed` and routes to `reject_user_unparsed_scaffolds` at `:2404-2453`. Diagnostic message: *"data `{name}` has an opaque body — M1(2.8) user code cannot yet use record / list / map literals inside data bodies (see DOWNSTREAM_REQUIREMENTS.md class-5 gap #3)."* This is a compiler-side limitation, not a substrate gap; lowering must learn to handle list literals in data bodies. Per ROADMAP T-Substrate row, the broader `ValueBody::List` substrate work is owned by T-Substrate ValueBody-list/sum (PR #790; R2 sub-lane); R1C-A's narrower scope is **the data-body lowering path that consumes the substrate's list-shape**.
- **Predicate dispatch inventory** in the match block at `test_runner.rs:1424-1448` has all 10 predicates wired (Compiles / FailsWithDiagnostic / OutputEquals / PortHasState / CostBounded / LensOutputEquals / DifferentialEquals / AlgebraicLaw / ExecuteCommand wired across `:1424-1431`; `MockBackedInvariant` arm at `:1432-1448`). None are stubs except `MockBackedInvariant` which is ext-gated on `requires`.
- **Census authority** at `src/v3/compiler/tests/integration/sg0_census_test.rs:166-297`:
  - `EXPECTED_HAND_AUTHORED_NON_TEST` (lines 166-206) — 41 file paths.
  - `EXPECTED_HAND_AUTHORED_TEST` (lines 213-288) — 77 test file paths.
  - `EXPECTED_HAND_AUTHORED_FRAGMENTS` (line 297) — 1 entry.
  - Drift-detection test `sg0_v3_hand_authored_census` at `:387-451` — panics with narrative on mismatch.
- **For the 6 PB-census gate predicates** (`pb_hand_rust_at_shim_floor`, `lens_producer_files_remaining`, `pb_self_compile_fixed_point`, `pb_compiler_std_ratchet_zero`, `pb_test_file_generated_from_dag`, `pb_rust_tests_outside_residual_zero`): **no `.dag` predicate shape exists**. ROADMAP line 65 names T-TestGen as scoping authority for `[ext]` predicate shapes; R1C-A authors them from scratch.
- **Existing `.dag` test fixtures reading external state**: `r1_gates.dag` and `t_demo_fixtures.dag` have **no template** for fixtures that read file lists or census state. R1C-D will author the first such fixtures *after* R1C-A's predicate shapes land.

## Frame — three coupled sub-deliverables

R1C-A is the critical-path enabler for R1 closure under strict reading. It bundles three sub-deliverables that share the T-TestGen scoping authority pattern:

### Sub-deliverable A — M1(2.8) list-body lowering for `data` declarations (compiler work)

`data <name>: List<T> = [el1, el2, ...]` must lower successfully rather than fail at `lower.rs:2446`. This is the predecessor to all `MockBackedInvariant.requires`-populated fixtures.

**Scope:**
- Extend `lower_data_decl` (or analogous lowering path) to recognize list-literal value bodies and lower them as `ValueBody::List` (the substrate variant landed via PR #790 / T-Substrate ValueBody-list/sum work — verify substrate state at brief-dispatch time per `feedback_thesis_gate_state_drift`).
- Retire the `ValueBody::Unparsed` rejection path for list-shaped bodies; preserve rejection for shapes substrate doesn't yet carry.
- Scope: **list-shape only** (per the named class-5 gap #3). Map / record literals are out-of-scope for this sub-deliverable; they remain rejected pending separate substrate work.

**Acceptance:**
- A `.dag` fixture program declaring `data my_refs: List<ResourceReference> = [some_ref, other_ref]` lowers to a `Dag` whose `value_body` is `ValueBody::List(...)`, not `Unparsed`.
- Existing `reject_user_unparsed_scaffolds` rejection still fires for non-list bodies (regression test).
- DB-8 fixed-point converges bit-identically.

### Sub-deliverable B — Predicate-shape scoping for 6 PB-census gates (schema extension)

T-TestGen owns scoping the predicate shape per ROADMAP line 65: *"T-TestGen also owns scoping the predicate shape for `[ext]` gates that other lanes consume; currently includes `lens_producer_files_remaining` for T-PB-A (enumeration declared in `sg0_census_test.rs` at scoping time)."* No `.dag` predicate shape exists for any of the 6 census gates today. R1C-A authors them.

**Scope (per-gate predicate shape):**

| Gate | Predicate shape (proposed) | Reads |
|---|---|---|
| `pb_hand_rust_at_shim_floor` | `CensusBoundCheck { authority: DeclarationRef, list_constant: Symbol, bound: Int }` | `EXPECTED_HAND_AUTHORED_NON_TEST.len() ≤ bound` |
| `lens_producer_files_remaining` | `CensusSubsetCount { authority: DeclarationRef, list_constant: Symbol, subset_predicate: ... }` | count of `EXPECTED_HAND_AUTHORED_NON_TEST` files matching lens-producer pattern |
| `pb_self_compile_fixed_point` | `FixedPointConverges { compile_target: Path, expected: SnapshotRef }` | bootstrap snapshot equality |
| `pb_compiler_std_ratchet_zero` | `RatchetZero { authority: DeclarationRef, ratchet_kind: ConsolidationRatchet }` | compiler-local types-not-in-positive-set count = 0 |
| `pb_test_file_generated_from_dag` | `GeneratedFromDag { authority: DeclarationRef, generated_paths: List<Path> }` | test-file partition matches generated set |
| `pb_rust_tests_outside_residual_zero` | `CensusBoundCheck { authority: DeclarationRef, list_constant: Symbol, bound: Int }` | `EXPECTED_HAND_AUTHORED_TEST.len() = 0` (per cascade promotion) |

The above shapes are **proposals** — finalize by reading each gate's definition + finding the minimal shape that closes it without over-generalizing. **Audit before authoring** (per matrix's pre-author verification invariant): grep `dsl/std/verification.dag` for existing predicate variant patterns; mirror the existing schema discipline.

**Scope rule:** keep predicate shapes minimal + closed-form. A predicate that needs general-purpose computation is a sign the gate is mis-scoped; escalate per STOP discipline below.

**Acceptance:**
- 6 predicate variants authored in `dsl/std/verification.dag` (or sibling) following the existing predicate-variant pattern.
- Each variant has a runner-dispatch arm in `src/v3/compiler/src/test_runner.rs` (alongside the existing 10).
- Variants compile cleanly; no `[ext]` blocker remains for the 6 census gates' predicate-shape requirement.

### Sub-deliverable C — `MockBackedInvariant` minimal-demo fixture (closure receipt)

Once Sub-deliverable A lands, `testgen_mock_backed_integration_safe` is unblocked. Author the minimal demo fixture: a `TestClaim` with non-empty `requires: List<ResourceReference>` that the runner materializes into mock obligations and evaluates.

**Scope:**
- Author one `TestClaim` fixture in `r1_gates.dag` (or sibling) with `predicate: MockBackedInvariant` and a non-empty `requires` list referencing two declared `ResourceReference` instances.
- Runner evaluates the fixture and returns `Pass` (mock-backed receipt landed).

**Acceptance:**
- `testgen_mock_backed_integration_safe` gate evaluates `Pass`.
- Fixture is the minimal possible demo (~5-10 lines `.dag`); not a comprehensive mock-coverage suite.

## Slice — A → B → C, in order

A blocks B (predicate shapes have a `data ... = [list, of, things]` body for census-list references) AND blocks C (MockBackedInvariant fixture has a `requires: List<ResourceReference>` body). B blocks R1C-D (R1C-D's gate fixtures use B's predicate shapes).

1. **PR-A (Sub-deliverable A):** lower.rs list-body lowering. Standalone deliverable; gate verification: `cargo test --workspace --exclude v2-compiler-tests` clean; DB-8 converges; `reject_user_unparsed_scaffolds` regression passes.
2. **PR-B (Sub-deliverable B):** predicate variants in `dsl/std/verification.dag` (mirrored to `src/v3/std/verification.dag`) + runner dispatch arms. Depends on PR-A landing (predicate variants reference list-shaped data).
3. **PR-C (Sub-deliverable C):** `MockBackedInvariant` minimal-demo fixture. Depends on PR-A (fixture body uses list literal). Closes `testgen_mock_backed_integration_safe`.

3 PRs, sequential. Bundle PR-B + PR-C if PR-A lands cleanly + the worker has bandwidth (reduces review cycles).

## Acceptance (R1C-A as a whole)

- [ ] Sub-deliverable A: `data: List<T> = [...]` lowers to `ValueBody::List`, not `Unparsed`. Regression test for non-list bodies still rejecting.
- [ ] Sub-deliverable B: 6 predicate variants in test schema + runner dispatch arms; all compile + dispatch cleanly.
- [ ] Sub-deliverable C: `testgen_mock_backed_integration_safe` gate evaluates `Pass` (closes the `[ext]` gate listed in ROADMAP line 65).
- [ ] All R1 gates remain green (`complexity_merge_sort_is_nlogn`, `complexity_merge_sort_v3_matches_v2_oracle`, `lane_e_bundled_witness_host_emit_parity`, etc.).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status updated to "R1C-A: 3/3 sub-deliverables landed; closes mock gate; unblocks R1C-D."
- [ ] R1C-D worker brief signaled as unblocked (cross-manager queue → R1 Closure Manager).

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **Sub-deliverable A reveals that `ValueBody::List` substrate variant is not yet landed on main** (per `feedback_thesis_gate_state_drift` — verify at brief-dispatch time, not at authoring) → STOP. Escalate to R2 Substrate Manager (post-spawn) or to Director (pre-spawn) for substrate landing. R1C-A cannot ship Sub-deliverable A without the substrate.
- **Sub-deliverable B's predicate-shape proposals turn out to need substrate work beyond schema extension** (e.g., a predicate variant requires a new substrate connective to express census comparison) → STOP. Per `feedback_compiler_is_dag_processor` + `feedback_construction_over_ratchets`, substrate-connective extension is C1-class; escalate to Director.
- **Sub-deliverable C's `MockBackedInvariant` minimal demo regresses an existing R1 gate** → STOP immediately. R1 gate green is non-negotiable.
- **Audit reveals the proposed predicate shapes for the 6 PB-census gates conflict with the existing 10-predicate dispatch arm convention in `test_runner.rs`** → STOP. Surface for design clarification rather than diverging the predicate-variant pattern unilaterally.
- **DB-8 fixed-point drifts** → STOP immediately.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-A.
- Authority matrix categorization: Category 1 (worker brief) per [`docs/briefs/r2-manager-brief-authority-matrix.md`](r2-manager-brief-authority-matrix.md).
- Runner authority: `src/v3/compiler/src/test_runner.rs:1423-1448` (predicate dispatch + `MockBackedInvariant` `NotYetImplemented` reason).
- Schema authority: `src/v3/std/resources.dag:25-26` (`ResourceReference`); `src/v3/std/verification.dag:187` (`TestClaim.requires` field); `dsl/std/verification.dag:36` (`TestClaim` type declaration; DSL-side authority).
- Lowering authority: `src/v3/compiler/src/lower.rs:2446` (`ValueBody::Unparsed` rejection); `:2404-2453` (`reject_user_unparsed_scaffolds`).
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs:166-297` (`EXPECTED_HAND_AUTHORED_*` constants); `:387-451` (drift test).
- Gate authority: [`ROADMAP.md` lines 65-68](../../ROADMAP.md) (T-TestGen + T-PB-A + T-PB-B `[ext]` predicate names).
- Substrate dependency (Sub-deliverable A): T-Substrate ValueBody-list/sum (PR #790; R2 sub-lane).
- Downstream consumer: [R1C-D worker brief](r1c-d-t-pb-census-as-dag-worker.md) — depends on Sub-deliverable B's predicate shapes.
- Discipline anchors: `feedback_construction_over_ratchets`, `feedback_audit_adjacent_authority_first`, `feedback_thesis_gate_state_drift`, `feedback_verify_thesis_claims`, `feedback_compiler_is_dag_processor`.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
