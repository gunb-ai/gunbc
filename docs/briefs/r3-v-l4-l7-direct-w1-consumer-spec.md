# R3 Lane 1 W1 Consumer-Side Implementation Spec

**Status:** PROPOSAL - research-only consumer-side implementation worker brief
spec. This note pre-authors the Lane 1 slice-1 dispatch shape against the
landed W1 producer contract docs. It does not authorize substrate edits, new
`TestPredicate` variants, runner changes, fixture rewrites, or implementation
commits.

**Parent lane:** [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)
and the Lane 1 readiness re-audit in
[`r3-v-l4-l7-direct-readiness-reaudit-post-e5.md`](r3-v-l4-l7-direct-readiness-reaudit-post-e5.md).

**W1 contract authority:** [`r3-pr-e8-w1-output-producer-contract-blocker.md`](r3-pr-e8-w1-output-producer-contract-blocker.md)
from PR #1485. PR #1485 landed the producer contract docs only; W1 runner
implementation is still R2-Evaluator territory.

## Current Live State

The Lane 1 L4 skeleton fixture is already present at
`src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`. It
authors `r3_verification_l4_emit_eval_match_skeleton` with
`DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind {
output_ref: l4_out })`. The fixture still defines both producers as
`miss_int_lookup()` placeholders, and its header states the row intentionally
returns `NotYetImplemented` until the `(rust_emit_output, dag_eval_output)`
runner wiring lands. The runner still accepts only the historical Lane-E cost
lineage pair; any other pairing, including W1, remains deferred.

The body evaluator entry exists: `src/v3/compiler/src/lib.rs:547-553` exposes
`pub fn evaluate_body(...) -> Result<Value, EvalError>`. That closes the old
"no evaluator entry point" blocker, but it does not by itself make
`dag_eval_output` a producer. W1 must still connect the `ProgramOutputBind` row
to the eager evaluator path.

## W1 Contract Summary

The W1 contract names two acceptable producer-identity routes: transitional
DeclarationRef-name dispatch for exactly `rust_emit_output` /
`dag_eval_output`, or a durable P1 producer role / marker surface. Lane 1 slice
1 should consume whichever W1 implementation actually lands. It must not
introduce a new predicate, producer enum, or fixture-local authority. The
existing `DifferentialEquals` + `ProgramOutputBind` shape remains the consumer
surface.

The observation domain is likewise contract-bound. Durable W1 should read a
typed observation-channel/value-kind carrier. The narrow slice-1 carve-out may
parse emitted Rust stdout as a single trimmed `Int`, but only for this L4 Int
fixture and only with the W1 dissolution target named. Unknown producer pairs,
unsupported value shapes, failed Rust execution, stdout parse failures,
evaluator errors, and value mismatches must all fail closed with typed
`ClaimResult` outcomes.

## Post-W1 Implementation Worker Brief

When W1 implementation lands, dispatch the Lane 1 slice-1 worker with this
concrete target:

- **Fixture:** `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`.
- **Suite:** `r3_verification_l4_emit_eval_skeleton_suite`.
- **Claim:** `r3_verification_l4_emit_eval_match_skeleton`.
- **Predicate row:** keep
  `DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind { output_ref: l4_out })`.
- **Program shape:** keep the existing minimal branch + fold source in
  `TestClaim.source` unless W1's landed producer contract requires a smaller
  fixture. Any fixture source change must preserve the claim name.
- **Eval side:** `dag_eval_output` invokes `evaluate_body` directly over the
  `ProgramOutputBind.output_ref` body using the landed eager strategy. If memo
  identity remains absent, the implementation must name the no-memo eager
  carve-out at the call site.
- **Rust side:** `rust_emit_output` emits the same claim program to Rust,
  compiles/runs it through the W1 path, captures the contract-authorized
  observation channel, and normalizes to the same typed value domain as
  `dag_eval_output`.
- **Comparison:** `DifferentialEquals` compares typed values, not stdout bytes or
  target source text.

Failure taxonomy: emit failure, Rust compile/run failure, observation parse or
normalization failure, evaluator failure, typed value mismatch, and unsupported
producer/value shape must each fail closed with stage attribution. Do not
fallback to success, stdout-byte comparison, or fixture-local lookup behavior.

## Test Infrastructure

Use the merged skeleton harness pattern:
`src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs`
already compiles the L4 fixture through `OnceLock`-backed `cached_compile` and
asserts `ClaimResult` by variant shape. The implementation worker should reuse
that amortization pattern and avoid pinning diagnostic prose. Once W1 is live,
the L4 test should switch from `ClaimResult::NotYetImplemented(_)` to the
appropriate executable result for the skeleton claim, with any negative-path
tests asserting structured result categories rather than raw message substrings.

## Re-Engagement Trigger

Do not dispatch the implementation worker from this document alone. Re-engage
only when W1 lands on main and then verify, in that landed tree, that:

1. `eval_differential_equals` accepts `(rust_emit_output, dag_eval_output)`
   without hitting the legacy cost-only `NotYetImplemented` branch.
2. `dag_eval_output` calls the real eager evaluator path instead of fixture
   `miss_int_lookup()` behavior.
3. The no-memo eager carve-out is explicitly recorded, or memo identity carriers
   have landed.
4. `rust_emit_output` has a declared producer identity and observation-channel
   rule matching the W1 contract.

If any item fails, keep Lane 1 slice 1 in standby and route the gap back to
R2-Evaluator / W1 ownership. Verification owns the consuming fixture and
coverage requirement; it must not patch around missing producer authority.

## Non-Claims

- This spec does not claim W1 implementation is landed.
- This spec does not close `l4_emit_eval_match`.
- This spec does not authorize a new substrate role, observation carrier, or
  `TestPredicate` variant.
- This spec does not widen W1 beyond the Rust single-target L4 slice-1 row.
- This spec does not alter Lane 2 L5 or L7 algebraic-law dispatch.
