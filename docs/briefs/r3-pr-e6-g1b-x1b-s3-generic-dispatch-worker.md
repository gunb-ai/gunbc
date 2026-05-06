# R3 PR-E E6-G1.b — X1.b S3 Generic Dispatch Worker Brief

**Status:** pre-authored fire-later worker brief. Do not dispatch until the
trigger below fires. This brief exists so the Evaluator lane has a ready,
bounded worker packet for the generic `fold_lens<C>` / runtime-sourced callable
dispatch slice after the X1.b carrier chain lands.

**Assignment shape:** E4 / E6-G1.b generic dispatch continuation. The worker
consumes already-landed X1.b authority; it does not author substrate,
parser/lowerer, runner, or generic fold carrier changes.

## Live Source Authorities

- [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3
  `Q-Pattern-A-First-Slice-Subscope` and
  `Q-EVAL-Lens-Fold-First-Slice`: G1.a static representative is the accepted
  first executable lens/report slice; G1.b generic dispatch is deferred behind
  X1.b S1/S3.
- [`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md)
  §Deferred: G1.b — generic parametric `fold_lens<C>`: generic
  `fold_lens<C>(lens: Lens<C>, ...)` is blocked on X1.b S1 substrate carrier
  collapse, X1.b S3 evaluator `Indirect` evaluation, `DimensionReport<C>`
  construction conventions, and structural program-scope authority.
- [`x1b-evaluator-impact-audit.md`](x1b-evaluator-impact-audit.md): S1 is
  Substrate-owned `TransformDispatch` / `Indirect` / `input_ports()` authority;
  S3 is the first genuinely Evaluator-owned X1.b slice, consuming runtime-
  sourced callable dispatch rather than widening evaluator authority locally.
- [`r3-v-tc1-eta-equivalence-deeper-analysis.md`](r3-v-tc1-eta-equivalence-deeper-analysis.md)
  §Path A / §Path B: TC1 first slice uses G1.a static representative; Path B
  generic G1.b / X1.b remains deferred.
- [`r3-pr-e6-g1a-option3-feasibility-probe.md`](r3-pr-e6-g1a-option3-feasibility-probe.md)
  and [`r3-pr-e6-g1a-option3-static-lens-worker.md`](r3-pr-e6-g1a-option3-static-lens-worker.md),
  landed by PR #1844 / #1853: Option 3 narrows G1.a to an argument-opaque
  static representative and bars compiled-`Dag` reification,
  `lens_apply` whole-Dag reflection, scalar declaration-id encoding, and new
  evaluator `Value` variants. G1.b inherits those bars unless a later Director
  authority explicitly replaces them.
- [`r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md`](r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md)
  §Slice boundary: static G1.a and generic G1.b are separate; no generic
  `fold_lens<C>` claim before X1.b.

## Dispatch Trigger

Dispatch this worker only when all of the following are true:

1. **X1.b S1 has merged:** the substrate carrier collapse from
   `TransformNode.target + inputs` to the accepted dispatch authority has
   landed, including the runtime-sourced callable carrier (`Indirect` or the
   Director-ratified successor spelling), typed callee handle, and single
   input-port authority.
2. **X1.b S3 has merged or is explicitly assigned to this worker:** evaluator
   runtime-sourced callable dispatch is available through the named X1.b
   authority, or Director/Evaluator Manager explicitly reroutes S3 plus this
   G1.b consumer into one packet.
3. **G1.a outcome is available:** the static representative / report-production
   path has either landed, STOPed with a named replacement authority, or been
   explicitly superseded by Director. Do not let this generic slice erase or
   bypass the G1.a result.

If any trigger component is ambiguous, STOP and ask the Evaluator Manager for
the exact PR / symbol / carrier names before implementing.

## Worker Scope

Implement only the evaluator-side consumption needed for generic
`fold_lens<C>` call sites where the lens is a runtime value or parameter:

```text
fn fold_lens<C>(lens: Lens<C>, d: Dag) -> DimensionReport<C> = ...
// call sites such as:
// lens.read(d, behavior)
// lens.sequential.op(a, b)
// lens.branch(left, right)
// lens.iterate(acc, bound)
// lens.validate(d, composed)
```

The key distinction from G1.a is that these function fields are sourced from a
runtime `lens` value, not a top-level static `data` binding. The implementation
must consume the X1.b runtime-callee authority instead of pretending the
function fields are statically known declarations.

## Acceptance

The future implementation PR must demonstrate:

1. Runtime-sourced / Indirect callable dispatch is consumed through the named
   X1.b authority. The implementation must not inspect ad hoc strings,
   declaration-id scalar encodings, or a local host-Rust registry to recover
   the callee.
2. Static G1.a remains separate. Existing G1.a representative/report tests keep
   passing, and the PR body states that this slice extends the generic path
   rather than replacing the static representative.
3. A generic `Lens<C>` parameter or equivalent runtime-carried lens value can
   call at least one function field through the X1.b runtime-callee dispatch
   path.
4. Read-channel diagnostic lifting remains declared-authority-only:
   `Witness<C>::Violates { reason: String, at: Behavior }` is not fabricated
   into `DimensionFail.violations: List<Diagnostic>` unless an existing
   declared Diagnostic construction path is cited and exercised.
5. `Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>` construction
   still uses declared constructors and the existing evaluator value carriers.
6. Any `LoopBound::Descent` dependency remains fail-closed unless the E5
   descent proof consumer authority has landed and is explicitly in scope.
7. Tests fail if the generic call path is replaced by static-only
   `FieldProject` / `Callable` lowering or by a host-Rust lens callback.

## Hard Bars

Do not:

- add or use a host-Rust lens registry;
- route through `lens_apply.rs` whole-Dag reflection or treat
  `fold_lens_over_reflected_program` as the generic fold authority;
- add `eval_substrate_reify` or any compiled-`Dag` to evaluator-`Value`
  reification helper;
- encode `DeclarationId`, `DeclarationRef`, `ArrowPortRef`, or any callee
  handle as `Int`, `String`, or another scalar carrier;
- add evaluator `Value` variants;
- claim generic `fold_lens<C>` before runtime-sourced callee authority exists;
- add parser, lowerer, substrate carrier, or runner/TestPredicate behavior;
- widen `BinaryDimensionReportEquals` or any Verification predicate surface;
- implement G1.b by copying the G1.a static representative and hard-coding its
  field callees.

## STOP Conditions

STOP and report the exact missing authority if:

- the X1.b S1 carrier names differ from this brief and no Director-ratified
  replacement mapping is cited;
- X1.b S3 has not landed and the dispatch did not explicitly include S3;
- runtime-sourced callable dispatch is still fail-closed in the evaluator;
- the only available path is static-only `FieldProject` / `Callable` through a
  top-level `data` lens;
- generic `fold_lens<C>` would require new evaluator `Value` inhabitants,
  scalar declaration handles, or compiled-program reification;
- `DimensionReport<C>` construction requires a host-Rust report/witness mirror;
- read-channel `Violates` would need an undeclared String/Behavior-to-Diagnostic
  lift;
- G1.a has not landed or STOPed with a named replacement, and this worker would
  have to guess the report-production handoff shape.

## Non-Goals

- X1.b S1 substrate carrier authoring.
- X1.b lowerer production of runtime-callee dispatch.
- Parser syntax work.
- Runner, Verification predicate, or `BinaryDimensionReportEquals` changes.
- Whole-Dag reflection through `lens_apply.rs`.
- Static G1.a representative implementation or repair.
- E5 `LoopBound::Descent` execution.
- Any generic-fold claim that is not backed by the named X1.b runtime-callee
  authority.

## Worker Output

Open one implementation PR only after the dispatch trigger fires. The PR body
must cite the X1.b S1/S3 landing PRs or the explicit Director reroute, name the
G1.a outcome it builds on, list the runtime-sourced callable tests, and state
that no hard-bar surfaces were introduced.

If the trigger is not met, the correct output is a STOP note, not code.
