# R1 Testgen Manager Brief

## Orient before reading

- Product direction: [PR #672](https://github.com/gunb-ai/gunbc/pull/672)
  — `docs/thesis/compositional-modeling.md`. This manager's slice
  is load-bearing for the story doc's "tests fall out of the
  modeling" claim. Testgen is the runner that makes the release
  gates (including all the `[ext]` gates from the other four
  managers) actually evaluate as `.dag` programs. Lens API is the
  user-authored counterpart — letting consumers of the language
  extend the evaluation surface without compiler patches.
- Coordination context: [R1 Director Brief](r1-director-brief.md).
- Scope authority: [`THESIS.md`](../../THESIS.md) +
  [`ROADMAP.md`](../../ROADMAP.md). This brief does not author R1
  scope; it sequences and coordinates what those docs already name.

## Slice

This manager owns two lanes:

- **`T-TestGen`** (`ROADMAP.md:51`) — testgen runner, service
  simulation, first-class `TestClaim`. Size **L**. DB-15 follow-up
  (`ROADMAP.md:235`).
  - `testgen_structural_coverage` — `[ext]` gate.
  - `testgen_mock_backed_integration_safe` — `[ext]` gate;
    requires `MockBackedInvariant` wiring.
  - `testgen_manual_claim_is_first_class` — `[ext]` gate.
- **`T-LensAPI`** (`ROADMAP.md:52`, lens capability honesty pass
  at `:333`) — user-authored lenses + composition. Size **M-L**.
  - `user_authored_lens_compiles` — `[Day 1]` gate.
  - `lens_composition_associative` — `[ext]` gate; requires
    `AlgebraicLaw` predicate.
  - `lens_output_is_queryable_data` — `[ext]` gate.

## Framing question this manager answers

**Can the release gates evaluate as `.dag` programs end-to-end —
schema runner executes each predicate, `MockBackedInvariant` wires
service simulation, and user-authored lenses compose so the
evaluation surface is itself first-class `.dag`?**

Today:
- DB-15 schema landed (`ROADMAP.md:235`); generated runner
  execution remains follow-up. Translation: you can write a
  `TestClaim` in `.dag` and it compiles; you just can't yet
  *evaluate* it without the runner.
- The majority of R1 release gates are `[ext]` — they compile only
  after T-TestGen's schema extensions land (`ROADMAP.md:59`). That
  makes this lane the gate-enabling lane for R1.
- `MockBackedInvariant` service simulation is not yet wired.
- User-authored lenses (T-LensAPI) have DAY-1 compile support but
  composition / `AlgebraicLaw` / queryable-output are `[ext]`.

The ask: land the runner, wire service simulation, land lens
composition so the release-gate majority evaluates and user lenses
compose. When this closes, the R1 release gates themselves
(authored by other managers) become checkable.

## Sequence + dispatch

- **Day 1.** T-LensAPI `user_authored_lens_compiles` dispatches.
  `[Day 1]` gate; no schema extension needed. This gets user-
  authored lens compilation working early so consumers can start
  writing lenses while the evaluation surface is built.
- **Day 1.** T-TestGen runner foundation dispatches. Landing order
  for the three sub-deliverables is a lane-owner decision; the
  manager-level sequence is: runner first (so predicates have
  *something* to run against), then structural coverage, then
  mock-backed integration.
- **As runner lands.** `testgen_structural_coverage` predicate is
  the first to become evaluable. This evaluates "did testgen
  enumerate the structural transitions at a boundary?" — the
  claim from story-doc Part 6.
- **After runner foundation.** `testgen_mock_backed_integration_safe`
  dispatches. Requires `MockBackedInvariant` wiring — the
  non-trivial piece.
- **After runner foundation.** `testgen_manual_claim_is_first_class`
  dispatches. Manually-authored `TestClaim` values evaluate
  alongside generated ones.
- **Parallel (LensAPI).** `lens_composition_associative` requires
  `AlgebraicLaw` predicate in the T-TestGen schema extension.
  Coordinate internally so the two lanes land the shared extension
  once.
- **Parallel (LensAPI).** `lens_output_is_queryable_data`
  dispatches independently; don't block on compositional work.

## Hand-off points

- **Sideways to Self-hosting Manager.** T-TestGen runner readiness
  is the hand-off signal for T-PB-B landing. Self-hosting drafts
  `.dag` `TestClaim` declarations during Day-1, waits on this
  signal, then converts pipeline / contract tests. Notify
  Self-hosting when the runner lands. **Landed batch (runner-backed):**
  `docs/briefs/t-pb-b-1.md` + `src/v3/compiler/tests/dag/` —
  `t_pb_b_1_dag_runner_test` evaluates the landed suites (PR #736); the
  former compile-smoke-only `t_pb_b_1_tests_dag_smoke_test` is **retired**
  (redundant with the runner test — *Decisions log* 2026-04-24).
  **Pre–Rust-deletion checklist (single source — edit here only; do
  not fork a competing numbered list in other briefs):** (1) **[done,
  PR #736]** Runner accepts v3 `.dag` modules under
  `src/v3/compiler/tests/dag/` with the same `compile_to_dag`
  entrypoint as today’s integration harness — proven by
  `t_pb_b_1_dag_runner_test`. (2) **[done, PR #736, scoped-NO]**
  `requires: []` vs `empty()` for `List<ResourceReference>`: **not
  equivalent in `data` bodies today.** Runner path must stay on `[]`;
  `empty()` is rejected by M1(2.8) class-5. Receipt pinned by
  `test_runner_data_bodies_reject_requires_empty_call_today`. Receipt
  paragraph in `docs/briefs/t-pb-b-1.md`. (3) **[done for Day-1 predicate
  shapes, PR #736]** M1(2.8) predicate-inlining constraints for
  `PortHasState` / `CostBounded`: inlined on each `TestClaim` record in
  `t_pb_b_1_contract_port_cost.dag`; standalone
  `data _: TestPredicate = …` bodies remain NYI under the same
  class-5 restriction. Receipt paragraph in `docs/briefs/t-pb-b-1.md`.
  (4) **First Rust deletion batch — partial sign-off (2026-04-24).** The
  three `#[test]` functions in the former `t_pb_b_1_tests_dag_smoke_test.rs`
  (redundant compile-only coverage for the landed `tests/dag/*.dag` modules)
  are **deleted**; replacement coverage and the `#[test]` → `TestClaim`
  mapping are recorded in the *Decisions log* below. **Still open** for the
  remainder of the inventory until a follow-on sign-off entry lands
  (same four-point guard). Target shape for the **next** slice: one thin
  `Compiles`-only duplicate (or `thesis_validation_test.rs` subset) whose
  claim is already covered by a landed `.dag` suite evaluated under
  `t_pb_b_1_dag_runner_test`. Candidate inventory (under
  `src/v3/compiler/tests/integration/`, registered in `integration.rs`
  and enumerated in `sg0_census_test.rs`):
  - **Excluded from first batch: `pipe_desugar.rs`.** Despite the
    `expect("compiles")` surface, `pipe_desugars_unary_call_by_injecting_the_left_value`
    and `pipe_desugars_multi_arg_call_with_first_arg_injection` assert
    structural facts — `Transform` target name, `call.inputs.len()`,
    injected `LiteralBits::Int` order, and the port's declared return
    `TypeShape`. The `.dag` analogue (`t_pb_b_1_pipeline_smoke.dag`,
    `Compiles` + source text only) does not cover those facts. Holding
    this file until structural-query predicates (target/arity/argument
    order/return shape) land in the T-TestGen schema and a `.dag`
    suite exercises both unary and multi-arg shapes.
  - `thesis_validation_test.rs` — mixed shapes (`Compiles`,
    `PortHasState`, `CostBounded`, `FailsWithDiagnostic` via
    `rendered_rust_diagnostic`). The `Compiles` / `PortHasState` /
    `CostBounded` subset maps onto the landed runner predicates; the
    rendered-diagnostic subset does **not** yet have a runner-backed
    counterpart and must be excluded from the first batch. Any split
    of this file needs a receipt listing which `#[test]` functions
    move and which stay.
  **Explicit Testgen sign-off — guard before any deletion PR opens:**
  1. Receipts (1)–(3) reviewed on main and still green (`cargo test
     -p v3-compiler t_pb_b_1_dag_runner_test` passes; the class-5 pin
     `test_runner_data_bodies_reject_requires_empty_call_today` still
     passes). 2. For each Rust `#[test]` proposed for deletion,
     name the `.dag` `TestClaim` (suite + claim name) that covers it
     and confirm `TestRunner::run_suite` returns `ClaimResult::Pass`
     on today's runner — no inference by filename. 3. PR #735
     (symbolic cost / `Lookup`) either merged or explicitly determined
     not to shift `CostBounded` witnesses for the candidate claims
     (per the cost-witness note in `docs/briefs/t-pb-b-1.md`). 4.
     Testgen manager records the sign-off in this brief's *Decisions
     log* **before** the self-hosting manager opens the deletion PR;
     without that entry, the self-hosting lane stays parked on
     `ROADMAP.md` T-PB-B lane work (`pb_rust_tests_outside_residual_zero`)
     and does not delete Rust tests.
     Out of scope today: deleting tests whose shape is rendered-diagnostic
     only, shared-predicate (`let pred_*: TestPredicate`) factored, or
     `let`+`empty()` backed — those depend on runner seams / M1(2.8)
     class-5 lift that have not landed.
- **Sideways to Surface Manager.** `emit_omni_demo_fixtures_green`
  and the three `emit_*` gates under T-Emit require `ExecuteCommand`
  + `ForAllTargets` predicates — these are T-TestGen `[ext]`
  extensions. Coordinate predicate shape with Surface early so
  T-Emit lane-owners aren't blocked.
- **Sideways to Release Manager.** T-Demo's `fixture_compiler_nerd_canonical`
  gate includes lens-output demos (`[ext]` on lens-output) — that's
  a T-LensAPI `lens_output_is_queryable_data` consumer. Notify
  Release when that lane lands.
- **Sideways to Substrate Manager.** When E-P lands (Substrate
  manager's sub-lane E-P), v3 cells produce `SubValueRelation`
  per-call. That's what `testgen_structural_coverage` should
  evaluate against — v3 authority rather than v2 oracle.
  Coordinate with Substrate on that cutover once E-P is near
  closure.
- **Up to director.** Any schema extension that looks like it
  needs to grow beyond what `ROADMAP.md:59` lists (new predicate
  kind, new `TestClaim` shape) is a scope question. Flag for
  director before landing.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-TestGen:**
- [x] Schema extensions landed — `ExecuteCommand`, `ForAllTargets`,
      `LensOutputEquals`, `DifferentialEquals`, `AlgebraicLaw` variants
      added to `TestPredicate` (PR #678, merged 2026-04-24)
- [x] Runner foundation — schema predicates execute structurally
      (PR #688, merged 2026-04-24)
- [x] `testgen_structural_coverage` gate compiles + evaluates
      (PR #720, merged 2026-04-24)
- [x] `MockBackedInvariant` wiring
      (PR #722, merged 2026-04-24 — runner dispatches on `MockBackedInvariant`,
      resolves both `DeclarationRef` edges, returns `NotYetImplemented(String)`
      naming the unresolved mock-simulation step)
- [ ] `testgen_mock_backed_integration_safe` gate compiles + evaluates
- [x] `testgen_manual_claim_is_first_class` gate compiles + evaluates
      (PR #707, merged 2026-04-24)

**T-LensAPI:**
- [x] `user_authored_lens_compiles` gate (Day-1) passes
      (PR #679, merged 2026-04-24)
- [x] `AlgebraicLaw` predicate runner evaluation wired
      (PR #728, merged 2026-04-24 — `Associativity` initial dispatch;
      **dissolved by PR #741** — `declaration_is_binary_int_add_associativity_witness`
      deleted, associativity now evaluates via D1 `int_associativity_holds_all_triples`
      over the lens-application primitive)
- [x] `lens_composition_associative` gate compiles + evaluates
      (PR #728, merged 2026-04-24 — witness + `r1_gates.dag` suite;
      PR #741 swapped the Rust operator recognizer for D1 lens application)
- [x] `lens_output_is_queryable_data` gate compiles + evaluates
      (PR #741, merged 2026-04-24 — D1 `apply_lens_declaration` primitive
      + D2 real `eval_lens_output_equals` apply/compare. The prior `NotYetImplemented`
      thin-receipt shape is gone; `compile_to_dag(&claim.source, …)` is **still called**
      on the evaluation path (intentional — reflects the program `Dag` for lens input
      and pairs the canonical lens `Dag` for P2 `id_space` alignment), fail-closed
      on tokenize/parse/inference per P3. `r1_lens_output_input_from_program` sentinel
      reflects `Dag.nodes` until first-class `Dag` literals land. Closes ROADMAP
      "Scheduled cleanups: LensOutputEquals runner and R1 gate fixtures" items 1–3
      (split fixture folded back into `r1_gates.dag` via `r1_gates.template.dag`
      + `build.rs` splice). **Follow-on open (ROADMAP §77 item 1):** retire the
      two parallel `compile_to_dag` call sites when `DeclarationRef` resolves
      executable lens + inputs structurally from one authority. **Follow-on open
      (ROADMAP §77 item 3):** dissolve fixture-local `named_function_count` /
      `count_named_bind` stubs once `DeclarationRef` can resolve the lens from
      `program_dag` alone)

**Schema extensions owned here that other managers consume:**
- [x] `ExecuteCommand` predicate (Surface T-Emit consumer) — landed PR #678
- [x] `ForAllTargets` predicate (Surface T-Emit consumer) — landed PR #678
- [x] `LensOutputEquals` predicate (Substrate T-LaneE consumer
      — `complexity_merge_sort_is_nlogn`) — landed PR #678
- [x] `DifferentialEquals` predicate (Substrate T-LaneE consumer
      — `complexity_v3_matches_v2_oracle`) — landed PR #678

Decisions log (append as they happen):

- 2026-04-24 (Lane B / #763): **Pre–Rust-deletion checklist item (4) — first slice signed
  off (redundant T-PB-B-1 compile-smoke).** Receipts (1)–(3) re-verified
  green (`cargo test -p v3-compiler t_pb_b_1_dag_runner_test`,
  `test_runner_data_bodies_reject_requires_empty_call_today`); PR #735
  merged on main (`cf59f2288` — `CostBounded` witness for the candidate
  claims is stable). **Deleted Rust tests** (file
  `t_pb_b_1_tests_dag_smoke_test.rs` removed) and **replacement**:
  `t_pb_b_1_dag_runner_test::lower` keeps the **empty module diagnostics**
  compile-smoke receipt for each declaring `tests/dag/*.dag` harness, then
  `TestRunner::run_suite` proves embedded `TestClaim` predicates (`ClaimResult::Pass`
  for every claim — orthogonal to the outer harness diagnostic check). **Implementation
  (current `lower` in `t_pb_b_1_dag_runner_test.rs`):** only `Ok(dag)` returns, with an
  explicit `assert!(dag.diagnostics().is_empty())` on that arm; `Err(Semantic(_))` **panics**
  (not a success path) — the declaring harness must compile with an empty module
  diagnostic table before `run_suite` runs.
  - `t_pb_b_1_pipeline_smoke_dag_lowers_cleanly` →
    `t_pb_b_1_dag_runner_test::t_pb_b_1_pipeline_smoke_suite_passes_through_runner`
    / suite `suite_pipeline_pipe_unary` / claim `claim_pipe_unary_compiles`
    ("pipe unary call program compiles").
  - `t_pb_b_1_contract_diagnostic_smoke_dag_lowers_cleanly` →
    `t_pb_b_1_dag_runner_test::t_pb_b_1_contract_diagnostic_smoke_suite_passes_through_runner`
    / suite `suite_contract_diagnostic_negatives` / claim `claim_bool_int_rejected`
    ("Bool annotation rejects Int literal").
  - `t_pb_b_1_contract_port_cost_dag_lowers_cleanly` →
    `t_pb_b_1_dag_runner_test::t_pb_b_1_contract_port_cost_suite_passes_through_runner`
    / suite `suite_contract_port_and_cost` / claims `claim_answer_resolved`,
    `claim_answer_cost_bounded`. **Follow-on** first-batch work
    (`thesis_validation_test.rs` subset; `pipe_desugar.rs` still excluded)
  remains under the 2026-04-24 inventory + four-point guard until a
  separate log entry names each `#[test]` → `TestClaim` pair.
- 2026-04-24 (PR #751): **Pre–Rust-deletion checklist item (4) prepped, not signed
  off.** Candidate inventory narrowed — `pipe_desugar.rs` **excluded**
  from first batch (asserts `Transform` target / input arity / literal
  order / return shape; `t_pb_b_1_pipeline_smoke.dag` only witnesses
  `Compiles`, so deletion would drop structural coverage until
  structural-query predicates land). First-batch candidate is the
  `Compiles`/`PortHasState`/`CostBounded` subset of
  `thesis_validation_test.rs`; four-point sign-off guard recorded in *Hand-off points →
  Sideways to Self-hosting Manager*. **No Rust deletion lands without a
  sign-off entry in this log naming the specific `#[test]` → `.dag`
  `TestClaim` mapping.** Self-hosting manager should continue to treat
  item (4) as parked until that entry appears.
- 2026-04-24: **T-LensAPI lane effectively closed** via PR #741 (D1+D2+D3+D4
  bundle). `apply_lens_declaration` lens-application primitive + frame-based
  `EvalCtx` + authoritative `std.list.fold` dispatch land as D1;
  `eval_lens_output_equals` and `eval_algebraic_law_for_claim_program` now
  consume D1 directly (D2, D3). D4 folds the split
  `r1_lens_output_equals_gate.dag` back into `r1_gates.dag` via a new
  `r1_gates.template.dag` + `build.rs` splice-at-build-time. One Rust
  structural recognizer deleted (`declaration_is_binary_int_add_associativity_witness`);
  the `eval_lens_output_equals` NYI-path thin receipt is replaced by real
  evaluation — `compile_to_dag` call sites remain on the evaluation path,
  intentional and load-bearing (program `Dag` for reflection, canonical lens
  pairing for P2 `id_space` alignment); ROADMAP §77 item 1 keeps an open
  follow-on to retire the parallel compile paths when `DeclarationRef`
  resolves lens + inputs from one authority.
  **Supersedes #740** (wise-koi-316 AlgebraicLaw direction, archived without
  merging). ROADMAP "Scheduled cleanups: LensOutputEquals runner and R1
  gate fixtures" items 1–3 all dissolved in the same PR. Lane non-goals
  called out in PR body: `reflect_behavior` still lossy for
  Transform/Branch/Loop variants; only `AlgebraicLawKind::Associativity`
  is evaluated (other kinds stay `NotYetImplemented`).
- 2026-04-24: **T-LensAPI `lens_composition_associative` initial dispatch**
  via PR #728 (Rust operator-shape recognizer), later dissolved by PR #741.
- 2026-04-24: T-PB-B / Testgen **pre–Rust-deletion** coordination
  checklist consolidated in this brief (Hand-off → Self-hosting);
  `docs/briefs/t-pb-b-1.md` now points here instead of duplicating
  numbered items.
- 2026-04-24: `ForAllTargets` self-referential variant dissolved to
  `{ command, args, expect_exit_code }` to preserve bounded-kernel
  invariant (Node is the only recursive type).

Open questions for director:

- _(none today)_

Cross-manager notifications queued:

- **Surface Manager**: `ExecuteCommand` + `ForAllTargets` predicates
  are on main (PR #678). T-Emit lane-owners can now author gates
  against those predicate shapes.
- **Substrate Manager**: `LensOutputEquals` + `DifferentialEquals`
  predicates are on main (PR #678). T-LaneE consumers can reference
  them once the runner lands.
- **Self-hosting Manager**: Runner foundation landed (PR #688,
  2026-04-24). T-PB-B is unblocked — begin `.dag` TestClaim
  conversion of pipeline / contract tests. ⬅ **SEND NOW**
