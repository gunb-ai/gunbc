# R2 PR-B Body Evaluator — Eager Baseline (Design / Closure Deferral)

**Status:** PROPOSAL — docs-only design lock for the R2 Evaluator body-execution
lane. This brief authors the first body-evaluator implementation slice and the
boundary at which full evaluator execution must defer to R3. No Rust execution
code lands in this brief; no new substrate carriers are declared here.

R2 closure target for body evaluation is **closed-with-residuals**: the
substrate carrier surface and the deterministic eager baseline are designed in
R2; full executable evaluation across all strategies and the witness-construction
surface remain R3 residuals.

**Parent authority:**
[`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — PR-B row in
"PR breakdown" calls out body evaluator + witness construction; this brief is
the design slice for the body half.
**Substrate authority:**
[`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
§3 (interpreter shape, frame discipline, evaluation rules).
**Carrier authorities (live):**
[`docs/briefs/r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md)
+ [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) for `Value`,
`NamedField`, `EvalFrame`, `EvalStateStack`, `EvalStrategy`,
`InputEvaluationOrder`, `EvalStateKey`, and `EvalMemoKey`. The PR-A.3 audit
remains the design rationale for strategy / memoization decisions:
[`docs/briefs/r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md).

## Decision

PR-B's body-evaluator lane is sequenced into one R2-shippable slice and a
named R3 residual:

- **R2 slice — PR-B.0 (this brief, docs only):** lock the deterministic eager
  evaluation rule over bounded `.dag` bodies, name its carrier prerequisites,
  and define the residual boundary.
- **R2 slice — PR-B.1 (next body-evaluator implementation slice):** Rust eager
  body-evaluator over `Value` / `EvalFrame` / `EvalStateStack` plus the closed
  `EvalStrategy` / `EvalMemoKey` carrier boundary, behind a fail-closed boundary
  that errors on every non-eager / non-bounded case.
- **R3 residual:** lazy / normal-order evaluation, TC2 evaluation-order
  independence proof, witness construction surface (PR-B's other half),
  cross-target equivalence harness execution against the body evaluator
  (PR-D's executable form), and **`LoopBound::Descent` execution** —
  fail-closed in PR-B.1, gated on a separate descent-execution slice that
  lowers `std.termination` `DescentEvidence` to bounded iteration.

This brief covers PR-B.0 only. PR-B.1 no longer waits on carrier declaration;
it still needs its worker dispatch / Rust implementation slice.

## Live prerequisites (carriers available today)

| Carrier                                | Authority                                                                 | Status                                |
|----------------------------------------|---------------------------------------------------------------------------|----------------------------------------|
| `Value` / `NamedField`                 | `src/v3/std/runtime.dag` (PR-A.1, #1243)                                  | LANDED on `main`                       |
| `EvalFrame { bindings: Map<PortId, Value> }` | `src/v3/std/runtime.dag` (PR-A.2, #1255)                            | LANDED on `main`                       |
| `EvalStateStack { frames: List<EvalFrame> }` | `src/v3/std/runtime.dag` (PR-A.2, #1255)                             | LANDED on `main`                       |
| `EvalStrategy` / `InputEvaluationOrder` | `src/v3/std/runtime.dag` (PR-A.3 carrier completion, #1544)              | LANDED in this PR                       |
| `EvalStateKey` / `EvalMemoKey`         | `src/v3/std/runtime.dag` (PR-A.3 carrier completion, #1544)              | LANDED in this PR                       |

PR-B.0 reads these as the substrate it will execute over. PR-B.1 may now consume
the closed eager strategy and structural memo-key carriers from `runtime.dag`;
it must not invent local no-memo, string-key, or name-only substitutes.

## Carrier boundary now closed (formerly blocking PR-B.1)

PR-A.3 carrier completion declares the PR-B.1 strategy / memo-key boundary in
[`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag):

- `EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }`
  (PR-A.3 audit §EvalStrategy).
- `InputEvaluationOrder = LeftFirst` (eager baseline; `RightFirst` / parallel
  are R3 expansions per the audit's "semantic expansion" gate).
- `EvalStateKey { state: EvalStateStack }` — structural reachable-state
  identity for memoization keying (audit §Memoization Boundary).
- `EvalMemoKey { program, node, state_key, strategy }` — declared as a closed
  carrier; cached `Value` results only after successful evaluation, never
  diagnostics or partial state (audit §Memoization Boundary).

`EvalThunk` is **not** a PR-B.1 prerequisite — eager-only baseline does not
need a lazy boundary. `EvalThunk` is part of the R3 residual.

## First implementation target — PR-B.1 scope

PR-B.1 authors the **deterministic eager body evaluator over bounded `.dag`
bodies** in Rust, consuming the carriers above as existing substrate. Scope is
bounded by construction:

1. **Inputs.** A `NodeId` for the body, an initial `EvalStateStack` containing
   one `EvalFrame` with the parameter bindings the caller supplied at the
   `Transform(Callable(...))` site (per
   `docs/design-pb-runtime-interpreter.md` §3.2 Transform rule).
2. **Behavior dispatch.** The five `Behavior` variants from
   `src/v3/std/substrate.dag` (`Value` / `Transform` / `Branch` / `Loop` /
   `Bind`), with the eager rules from
   [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
   §3.2:
   - `Value` → return the carried `Value` directly.
   - `Transform(t)` → eager-evaluate inputs left-to-right (`LeftFirst`), then
     apply `t.target`. `Callable(decl_id)` over `ArrowBody::UserDefined(body)`
     pushes a fresh `EvalFrame`, binds parameters by `PortId`, evaluates the
     body, and pops the frame. `FieldProject` / `Operator` are total over the
     primitive inhabitants.
   - `Branch` → eager-evaluate the scrutinee, select the path, evaluate the
     selected path body in a fresh frame containing the path binding.
   - `Loop` → eager-evaluate the init once, then dispatch on the two
     variants of `LoopBound` per `src/v3/std/substrate.dag` and
     [`docs/design-mutual-recursion-lowering.md`](../design-mutual-recursion-lowering.md).
     The variants are non-optional dispatch arms (Facts-Flow-Forward;
     dropping either erases substrate proof state):
     - `LoopBound::Cardinality { count: PortId }` — executable in PR-B.1.
       Read `count` through the declared runtime/value path; if the
       cardinality witness is available, execute finite iteration in a
       fresh frame containing the accumulator binding for that iteration;
       otherwise fail closed with a `Diagnostic`.
     - `LoopBound::Descent { cluster: ClusterId }` — **named fail-closed
       residual at PR-B.1**: every `Descent` loop emits a `Diagnostic`,
       not a hang and not a silent skip. Dissolution trigger: a separate
       descent-execution slice that consumes `std.termination`
       `DescentEvidence` and lowers it to bounded iteration. PR-B.1 must
       not silently broaden to `Descent` execution; that rule belongs in
       its own slice with its own design lock.
   - `Bind` → register the binding in the current frame; no body execution at
     the Bind site (function-form bodies enter from `Transform(Callable(...))`).
3. **Strategy.** PR-B.1 reads
   `EvalStrategy = ApplicativeOrder { input_order: LeftFirst }` only.
   Any other inhabitant of `EvalStrategy` is a fail-closed `Diagnostic` per
   `feedback_fail_closed_discipline`. PR-B.1 introduces no `NormalOrder`
   handling; `EvalThunk` is unreachable in PR-B.1.
4. **Memoization.** PR-B.1 may consume `EvalMemoKey` to cache completed
   `Value` results, but it must not cache diagnostics or partial state, and
   it must not key on anything other than the declared
   `{program, node, state_key, strategy}` carrier.
5. **Termination.** PR-B.1 inherits `feedback_decidability_invariant`:
   evaluation must terminate on every well-formed bounded `.dag` body.
   `Loop` is bounded by `LoopBound`; recursion is bounded by termination
   evidence (`std.termination` `DescentEvidence`). Evaluation that would
   require unbounded execution is a fail-closed `Diagnostic`, not a hang.
6. **Errors.** Every detectable problem is a `Diagnostic` (P3 / fail-closed
   discipline). Non-evaluable `ArrowBody` variants
   (`ExternalRealization` dispatching to a host-bound implementation handled
   outside this evaluator, plus `Pending` / `NoBody` / `Unparsed`) are
   `Diagnostic`s at the call site, never panics or silent `None`s.

## Out of scope (R2 / PR-B.1)

These are **not** in PR-B.0 design lock and **not** in PR-B.1 implementation:

- **Lazy / normal-order evaluation.** Requires `EvalThunk` carrier and a
  declared evaluation rule per the PR-A.3 audit's "semantic expansion" gate;
  R3 residual.
- **TC2 evaluation-order independence proof.** The slice-0 hook
  (`src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`,
  predicate `Compiles`) stays deferred. TC2 strengthens to a strict
  differential / lens-equality claim only when at least two executable
  strategies coexist (eager + lazy or eager + alternate input order).
  PR-B.1 lands a single strategy, so TC2 cannot strengthen yet. R3 residual.
- **Witness construction surface.** PR-B's "witness construction" half (per
  `r2-evaluator-manager.md` PR-B row) is a separate design slice and a
  separate worker brief; this PR-B brief is the body-execution half only.
  R3 residual unless explicitly promoted earlier.
- **Cross-target equivalence harness execution.** PR-D slice 0
  ([`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md))
  landed structural primitives; executing the body evaluator inside that
  harness against Rust / Python / Go targets is gated on PR-B.1 plus a
  `ForAllTargets` lens runner. R3 residual.
- **First-class closures with captured environments.** Forbidden P1
  substrate change per `r2-pr-a-runtime-value-model.md` — `ClosureValue`
  may not be added to `Value`, and `Callable(DeclarationId)` is the only
  callable identity. PR-B.1 enforces this by construction (it never
  observes a closure inhabitant, because none exists in the substrate).

## R3 residual reason

The former PR-A.3 carrier absence is closed: `EvalStrategy`,
`InputEvaluationOrder`, `EvalStateKey`, and `EvalMemoKey` live in
`src/v3/std/runtime.dag`, so PR-B.1 may read the closed eager strategy and
structural memo-key boundary rather than inventing a local no-memo or
string-key convention.

Full evaluator execution still cannot ship in R2 because:

1. **Execution semantics for non-eager strategies undeclared.** `NormalOrder`
   has no rule, no thunk capture point, and no TC2 comparison obligation
   (PR-A.3 audit §EvalStrategy). Until those land, the evaluator cannot
   honor any inhabitant of `EvalStrategy` other than `ApplicativeOrder /
   LeftFirst`, which means TC2 cannot strengthen and PR-D harness execution
   cannot run two strategies for differential receipts.
2. **Witness construction surface unauthored.** The witness half of PR-B has
   no design lock yet. Even with PR-B.1 landed, witness emission for
   downstream consumers (R3 lens producers, cross-target reflection
   consumers) is not in scope here.

R3 closure for body evaluation requires (in order): PR-A.3 implementation
carriers → PR-B.1 eager Rust evaluator → lazy / `EvalThunk` slice →
TC2 strengthened predicate → witness construction worker brief and slice →
PR-D harness running the evaluator across targets.

## Acceptance gates (this brief, PR-B.0)

- ✅ Live prerequisites named with authority paths and merge IDs.
- ✅ Formerly blocking PR-A.3 carriers named as live `runtime.dag` substrate,
  each linked to the audit section that authored its design rationale.
- ✅ First implementation target is exactly **deterministic eager body
  evaluator over bounded `.dag` bodies**, no lazy / no normal-order, no TC2
  proof, no witness construction.
- ✅ R3 residual reason is named with three causal gaps, not a
  schedule excuse.
- ✅ Docs-only PR. No new substrate, no Rust execution code, no new test
  fixtures.

## Out of scope (this brief, PR-B.0)

- Authoring PR-B.1 as a worker dispatch brief. PR-B.1 may now consume the
  strategy / memoization carriers; this brief states the scope, not the
  dispatch instructions.
- Editing `r2-evaluator-manager.md` PR-table rows beyond a status / cross-ref
  note pointing to this brief.
- Any change to PR-A.3 audit content. PR-A.3 implementation is owned by
  whichever worker the manager dispatches; this brief consumes the audit's
  decisions, not amends them.
