# R3 Evaluator Dispatch Brief

**Status:** PR-E dispatch brief for R3-Evaluator implementation. This brief
composes the locked PR-A through PR-D design surfaces into bounded worker
slices. It is a dispatch authority, not a new substrate design.

**Read first:**
- [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md) — runtime
  `Value`, `EvalFrame`, `EvalStateStack`, strategy, and memoization boundary.
- [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md)
  and [`r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md)
  — deterministic eager body-evaluator rules.
- [`../design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) —
  PB-Runtime mirror of the evaluator runtime model.
- [`../design-reflection-completeness.md`](../design-reflection-completeness.md)
  and [`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md)
  — complete reflection input and PR-C dissolution gates.
- [`../design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)
  and [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
  — cross-target equivalence policy and harness primitives.
- [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md)
  and [`r2-evaluator-test-runner-authority-ratchet.md`](r2-evaluator-test-runner-authority-ratchet.md)
  — runner-extension debt and dissolution discipline.

## Dispatch Rule

Workers ship small implementation slices that consume existing carriers and
design locks. If a slice needs a missing carrier, a new `TestPredicate`
variant, a new observable `Value` inhabitant, or a convention-only runner
observation, the worker must STOP+PING and route the fact through
[`INVARIANTS.md`](../../INVARIANTS.md) §P1. Do not invent local Rust-side
authority to keep a slice moving.

Every hand-Rust runner addition must name its dissolution target: PR-B body
evaluator, PB-Runtime generated tests, T-LensProducer-Retirement,
T-FixedPoint, or a specific substrate carrier. Runner work is transitional
debt, not a second test-predicate language.

## Implementation Slices

### E1 — Value-Behavior Eager Execution

Implement the smallest evaluator execution path: `Behavior::Value` to runtime
`Value::LiteralValue`, with no frame mutation.

**Inputs:** runtime `Value` carrier; program DAG node lookup; PR-B.1
fail-closed catalog.

**Acceptance:** a focused test proves a bounded `.dag` body containing a
literal `Value` node evaluates to the expected runtime `Value`. Unsupported
`Transform`, `Branch`, `Loop`, and `Bind` paths fail closed.

**STOP+PING:** if the current Rust API cannot construct or compare the runtime
`Value` carrier without adding a parallel carrier or string parser.

### E2 — EvalFrame Lookup And Bind Environment

Implement evaluator-internal frame lookup and top-frame binding updates over
`EvalStateStack` and `Map<PortId, Value>`.

**Inputs:** `EvalFrame`, `EvalStateStack`, `Map<PortId, Value>` discipline from
PR-A.2 and PR-B.1.

**Acceptance:** tests cover innermost-frame-first lookup, top-frame-only write,
duplicate bind pre-check, and unbound-port diagnostics.

**STOP+PING:** if implementation would replace `Map<PortId, Value>` with
`List<EvalBinding>` or allow cross-frame mutation.

### E3 — Transform Application, Eager Left-First

Implement `Transform` evaluation for the smallest closed set: evaluate inputs
left-to-right under `ApplicativeOrder { input_order: LeftFirst }`, then dispatch
to already-supported callable or primitive operation surfaces.

**Inputs:** `EvalStrategy`, `InputEvaluationOrder`, PR-A.3 memo/strategy audit,
and PR-B.1 Transform checklist.

**Acceptance:** tests cover left-first input evaluation, arity mismatch
diagnostics, unsupported operator fail-closed behavior, and no lazy/thunk path.

**STOP+PING:** if PR-A.3 memo-key carriers are still missing and the slice would
need memoization to be observable. Eager no-memo execution is allowed; fake
strategy variants are not.

### E4 — Branch Arm Coverage

Implement `Branch` evaluation using resolved `BranchPattern` tag equality and
payload binding.

**Inputs:** PR-B.1 Branch rule, complete reflection shape, and existing
`BranchPattern` substrate declarations.

**Acceptance:** tests cover resolved variant match, payload binding into a fresh
frame, no-match fail-closed diagnostic, and `UnresolvedVariant` fail-closed
diagnostic.

**STOP+PING:** if the implementation needs wildcard/catch-all semantics not
present in the substrate.

### E5 — Loop Bounded Cardinality

Implement only `LoopBound::Cardinality` with deterministic accumulator
threading. `LoopBound::Descent` remains a named fail-closed residual until a
descent-execution slice consumes `std.termination` evidence.

**Readiness receipt:** [`r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md)
records historical prep and the remaining **`LoopBound::Descent`** gap. **Live
`main`:** `evaluator::eval_loop` executes **`LoopBound::Cardinality`** loops with
accumulator threading; **`LoopBound::Descent`** remains
**`LoopBoundDescentResidual`** until a descent slice consumes `std.termination`
evidence.

**Inputs:** PR-B.1 Loop rule, `LoopBound`, runtime `Value`, and frame discipline.

**Acceptance:** tests cover zero iterations, multiple iterations, accumulator
threading, missing/non-integer count diagnostics, and `Descent` fail-closed
behavior.

**STOP+PING:** if a worker needs to execute descent evidence or widen loop-bound
semantics in this slice.

### E6 — Lens Fold Completion

Deepen `fold_lens_over_reflected_program` beyond reflect-then-apply into the
full `Lens<C>` / `DimensionReport<C>` fold once E1-E5 provide enough body
execution semantics.

**Readiness receipt:** [`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md)
records the current **E6** blocker after **E3**, **E4**, **cardinality E5**,
and **Bind callable entry** landed on `main`: the evaluator can execute the
five L1 program behaviors, but generic `Lens<C>` still lacks live
function-field execution (`TransformTarget::Callable` / field projection),
declared lens-instance data, report/witness lifting, descent execution, and a
structural fold-scope decision. E6 must not duplicate `lens_apply.rs`'s
`FieldValue` interpreter or fabricate `DimensionReport<C>` values to keep
moving.

**Post-blocker gate packet:** [`r3-pr-e6-post-blocker-gate-packet.md`](r3-pr-e6-post-blocker-gate-packet.md)
names the next actionable sequence as E6-G0: evaluator field/call API support
for projected `Lens<C>` fields, followed by E6-G1: `Witness` /
`OptionalDiagnostic` / `DimensionReport` lifting over the existing carriers.
Descent stays an explicit residual unless termination evidence is executable.

**Inputs:** PR-C complete reflection, PR-E reflect/apply slice, PR-B evaluator
execution, and `docs/design-lens-framework.md`.

**Acceptance:** at least one complexity lens and one fail-closed validation lens
produce a `DimensionReport<C>` through the generic fold path without per-lens
Rust projection.

**STOP+PING:** if this would add a new observable `Value` variant, bypass
PB-Runtime body semantics, or introduce per-lens Rust dispatch.

### E7 — Witness Construction

Materialize `Witness::Inhabits` / `Witness::Violates` and diagnostic reports
from evaluator results for representative lenses.

**Inputs:** `src/v3/std/dimensions.dag`, diagnostic-kind Q6.5 two-layer
authority, PR-B evaluator semantics, and PR-C reflection gates.

**Acceptance:** tests cover complexity, tenant-flow, and IFC-style outcomes
with typed diagnostics and no string parsing of `Witness.reason`.

**STOP+PING:** if a lens-local diagnostic kind requires substrate-owned closed
sum extension instead of the Q6.5 lens-instance path.

### E8 — Runner Extension Follow-Ons

Implement the runner-extension workstreams only under the boundaries in
[`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md):

- PR-B.2 / W1: `DifferentialEquals` lineage producers, gated on typed
  observation carrier and PR-B body execution for `dag_eval_output`.
- PR-B.3 / W2: `AlgebraicLaw` runner support for `Commutativity`; `Identity`
  waits on the identity-element edge; `Distributivity` routes through P1.
- PR-B.4 / W3: `ForAllTargets` per-target dispatch, gated on structural
  observation carrier and target producer availability.

**Continuation readiness:** [`r3-pr-e8-runner-extensions-continuation-readiness.md`](r3-pr-e8-runner-extensions-continuation-readiness.md)
records the current post-bundle state: `AlgebraicLaw::Associativity` and
`AlgebraicLaw::Commutativity` are already wired, while W1/W3 still lack
declared producer identity / observation-channel authority for executable
runner work.

**W1 blocker/proposal:** [`r3-pr-e8-w1-output-producer-contract-blocker.md`](r3-pr-e8-w1-output-producer-contract-blocker.md)
records the accepted STOP+PING for
`DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind)`:
`dag_eval_output` is plausible through no-memo eager `evaluate_body`, but
`rust_emit_output` still needs an explicit producer identity contract and typed
observation normalization before runner code can land.

**STOP+PING:** no convention-only stdout parsing, no new `TestPredicate`
variants, and no permanent expansion of `test_runner.rs` authority without a
dissolution hook.

### E9 — Cross-Target Harness Consumption

Consume [`../design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)
for R3 L5 planning and strict receipts after LanguageSpec, Shape A grounding,
L4/L7 corpus, and structural observation prerequisites are live.

**Readiness receipt:** [`r3-pr-e9-cross-target-harness-consumption-readiness.md`](r3-pr-e9-cross-target-harness-consumption-readiness.md)
records the current implementation blocker: PR-D slice 0 + slice 1 are live,
but strict `ForAllTargets` consumption still waits on LanguageSpec, all Shape A
target grounding, L4/L7 corpus seed availability, and typed structural
observation.

**Acceptance:** PR-D slice 2 may wire existing `ForAllTargets` claims only when
all dependencies are cited together. Until then, only docs/audit updates or
blocked-row authoring are allowed.

**STOP+PING:** if implementation would enumerate targets, add predicate
variants, or compare bytes/strings instead of typed semantic observations.

## Parallelization

E1 and E2 can run in parallel if their write sets are disjoint. E3 depends on
enough of E1/E2 to evaluate inputs and bind parameters. E4 can begin after E2.
E5 depends on E1/E2 and should not absorb descent execution. E6 and E7 wait for
the body evaluator spine. E8 W2 can proceed independently of PR-A.3 body
execution for `Commutativity`; W1/W3 wait on typed observation carriers. E9 is
mostly R3 planning until its external gates land.

## Structural Acceptance

The existing manager gates remain authoritative:

- `evaluator_runtime_value_model_landed`
- `evaluator_body_evaluator_correctly_executes_std_termination`
- `evaluator_lens_application_complete_reflection`
- `evaluator_witness_construction_per_lens_correct`
- `evaluator_cross_target_equivalence_harness_primitives_landed`
- `evaluation_order_independent_lens_results` (deferred TC2 strict equality)

This brief does not add a new `TestClaim` variant. If a slice needs a new
acceptance predicate, route it through P1 and the manager brief before coding.

## R3 Unblock Signal

R3-Evaluator is ready to report lane progress when E1-E5 provide a minimal
eager body evaluator, E6 consumes complete reflection without per-lens Rust
projection, and E7 can materialize witnesses for representative lenses. Runner
extensions and cross-target receipts may advance in parallel, but they do not
replace the evaluator runtime spine.
