# R2 Pure Bootstrap - Tier 3 Mirror Dissolution Worker Pack

**Status:** Draft dispatch pack for R2 Pure Bootstrap Manager.

**Owner:** Pure Bootstrap Manager.

**Program authority:** `docs/r2-structure.md` Pure Bootstrap Manager scope:
post-R1 Pure Bootstrap work only. This pack covers Tier 3 mirror
dissolutions for termination, computation, induction, and effect-carrier Rust
mirrors. It does **not** authorize R1 PB census-reduction work; R1 owns
`pb_hand_rust_at_shim_floor`, `pb_compiler_std_ratchet_zero`,
`pb_rust_tests_outside_residual_zero`, and `lens_producer_files_remaining` per
ROADMAP gate authority.

## Authority Audit Receipt

1. **Substrate exists?** Yes for the carrier surfaces: `src/v3/std/termination.dag`,
   `src/v3/std/computation.dag`, `src/v3/std/induction.dag`, and
   `src/v3/std/effects.dag` exist. Rust mirrors remain in
   `src/v3/compiler/src/dag.rs`, `src/v3/compiler/src/dag/effects.rs`, and
   `src/v3/compiler/src/workflow_idempotency.rs`. The live dissolution trigger
   for termination/computation/induction is evaluated std block bodies instead
   of `ArrowBody::Unparsed`.
2. **Existing brief?** No existing brief in `docs/briefs/` owns the Tier 3 mirror
   dissolution pack as R2 PB work. Related carrier-port history lives in
   `docs/design-substrate-carrier-port-program.md`; this pack consumes that
   history and does not reopen the porting lanes.
3. **Design-doc recommendation matches?** Yes. `docs/r2-structure.md` names
   termination / computation / induction / effect-carrier Rust mirrors as R2 PB
   Tier 3 mirror dissolutions. `docs/design-substrate-carrier-port-program.md`
   says E-T/E-C/E-I carrier surfaces have landed; remaining work is consumer /
   evaluation wiring, not new carrier invention.
4. **Citations live?** Verified against current HEAD after PR #898 merged.
   Stable symbol anchors are used instead of line-number-only authority:
   `DescentEvidence`, `SizeBound`, `SubValueRelation`, `WorkflowEffect`, and
   `lane2_workflow_idempotency_report`.
5. **Carrier dissolves the bridge?** The bridge being dissolved is executable
   Rust mirroring of `.dag` std carriers. A valid implementation must replace
   mirror use with evaluated/reflected `.dag` values or prove a narrower
   terminal Rust boundary. Hand-deleting mirrors without an evaluated consumer
   path does not dissolve the bridge.

## Shared Worker Rules

- Start with an audit PR if the implementation path is not immediately
  mechanical. The minimum useful output is an implementation-ready plan with
  live symbol anchors, explicit blockers, and a test receipt list.
- Do not add new substrate shapes as a convenience. If the plan requires a new
  `ValueBody` variant, new connective, or broader std-body evaluation surface,
  STOP and escalate to Pure Bootstrap Manager. The manager will route to
  Substrate / Grounding / Director as needed.
- Do not duplicate R1 PB census-reduction authority. If SG-0 non-test or test
  census entries fall as a side effect, report the delta; do not expand scope to
  chase unrelated census entries.
- Preserve fail-closed behavior. Unknown descent, unsupported workflow shapes,
  and unparsed std bodies must stay explicit; no fallback may fabricate semantic
  evidence.
- Every implementation PR must report SG-0 deltas from
  `src/v3/compiler/tests/integration/sg0_census_test.rs` and run the narrow
  test receipt plus `make fmt` / `make v3` unless the worker explains why those
  commands are unavailable.

## Worker 1 - Termination Mirror Audit / Dissolution Plan

**Scope:** `src/v3/compiler/src/dag.rs` termination mirror block:
`DescentEvidence`, `RankingDimension`, `PositiveDescentAmount`,
`ProportionalDivisor`, `DescentSource`, `TerminationProof`, `ProofEdge`, and
lattice / Peano helpers.

**Authority:** `src/v3/std/termination.dag`.

**Task:**

1. Audit which Rust mirror items are terminal carrier spellings versus executable
   scaffold caused by std block bodies lowering as `ArrowBody::Unparsed`.
2. Identify the smallest path to evaluate or reflect the `.dag` authority for
   lattice helpers (`merge_evidence`, `join_evidence`, `evidence_rank`,
   `optional_evidence_meet`, `map_evidence_merge_at`) without changing their
   fail-closed behavior.
3. Produce an implementation-ready plan, or implement a narrow slice if the
   evaluated path already exists.

**STOP-AND-ESCALATE:**

- The plan needs new std-body evaluation support beyond existing v3 lowering.
- Structural parameter references are required to replace `String` bridges.
- Peano materialization caps drift from the `.dag` authority.

**Receipts:** `m2_substrate_inhabitance_test` termination rows, SG-0 census
delta, and any new direct `.dag` evaluation receipt.

## Worker 2 - Computation Mirror Audit / Dissolution Plan

**Scope:** `src/v3/compiler/src/dag.rs` computation mirror block:
`SizeBound`, `CallPattern`, `ShrinkFactor`, `IterationPrimitive`,
`LoweringTarget`, `lower_call_pattern`, and bound/profile helper functions.

**Authority:** `src/v3/std/computation.dag`, with termination Peano carriers from
`src/v3/std/termination.dag`.

**Task:**

1. Audit the Rust computation mirror and classify terminal carrier spelling
   versus executable scaffold.
2. Identify the evaluated `.dag` path for `lower_call_pattern`,
   `size_bound_param`, `is_constant_bound`, `constant_bound_value`, and
   related helpers.
3. Keep `kernel_algebra_profile` out of this slice unless the required
   map-shaped carrier already exists; that mirror is separately gated on a
   future `ValueBody::Map` substrate lane.

**STOP-AND-ESCALATE:**

- `kernel_algebra_profile` requires `ValueBody::Map` or equivalent substrate
  work.
- `Forever` constant-bound semantics would change from iteration-bound / repeat
  cap semantics.
- The path requires changing termination mirror behavior first.

**Receipts:** computation rows in `m2_substrate_inhabitance_test`, any
cost/computation helper tests touched, SG-0 census delta.

## Worker 3 - Induction Mirror Audit / Dissolution Plan

**Scope:** `src/v3/compiler/src/dag.rs` induction mirror block:
`RecursionShape`, `InductiveField`, `SubValueRelation`, `CallDescentEvidence`,
and per-call descent evidence producer boundaries.

**Authority:** `src/v3/std/induction.dag`; related producer history in
`docs/design-substrate-carrier-port-program.md` Lane E-P.

**Task:**

1. Audit which Rust induction mirrors exist only so native DAG lenses can emit
   `std.induction::SubValueRelation` while std block bodies remain unparsed.
2. Identify whether generated/reflected lens execution can construct
   `SubValueRelation` directly today.
3. Preserve the E-P side-table decision unless the consuming cost/complexity
   lens proves a different storage shape is required.

**STOP-AND-ESCALATE:**

- The worker needs to widen `TransformNode` or attach new substrate fields.
- Parameter-name refs or structural `ParamRef` evidence are required before the
  mirror can dissolve.
- Cost/complexity consumer wiring is needed to prove the dissolution and would
  exceed this worker slice.

**Receipts:** E-P per-call descent evidence tests, induction rows in
`m2_substrate_inhabitance_test`, SG-0 census delta.

## Worker 4 - Effect-Carrier Mirror Audit / Dissolution Plan

**Scope:** `src/v3/compiler/src/dag/effects.rs` and
`src/v3/compiler/src/workflow_idempotency.rs`: `WorkflowEffect`,
`OperationEffect`, `CompositionVerdict`, unsupported detail/report carriers,
and the host-boundary functions that mirror `std.effects`.

**Authority:** `src/v3/std/effects.dag` and the existing Stage 2b / Stage 2e
idempotency and parallelism lens receipts.

**Task:**

1. Audit which effect-carrier Rust items are terminal typed boundaries versus
   transitional mirrors kept because emitted `.dag` lens modules are not called
   directly by the crate API.
2. Identify the narrowest path to replace `workflow_idempotency.rs` projection
   bodies with direct emitted/evaluated `std.effects` or lens calls, if that
   surface exists.
3. Preserve explicit unsupported variants for branch, loop, parallel, and
   missing-workflow cases.

**STOP-AND-ESCALATE:**

- Direct `.dag` / emitted lens invocation from crate API is missing and would
  require Grounding or emitter work.
- Constructor validation would become weaker than the current `ElementRef` /
  `BoolPortRef` boundaries.
- Unsupported workflow shapes would collapse to `None`, default success, or a
  string-only diagnostic.

**Receipts:** Stage 2b idempotency tests, Stage 2e parallelism tests,
`m2_lens_idempotency_migration_test`, SG-0 census delta.

## Reporting

Workers report one of:

- **Implementation PR ready:** mirror use reduced or replaced, receipts green,
  SG-0 delta included.
- **Audit PR ready:** implementation path and blockers are concrete, with live
  anchors and test receipt plan.
- **STOP:** substrate/evaluation/emitter capability missing; include the exact
  missing surface and the smallest upstream lane that would unblock it.
