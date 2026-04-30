# R2 PR-A.3 Follow-On Test Surface

**Status:** TEST PLAN - docs-only follow-on surface while Substrate owns the
PR-A.3 carrier implementation. This brief does not declare carriers, regenerate
bootstrap snapshots, or edit evaluator Rust.

**Parent authority:** [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md),
[`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md),
and parser prerequisite [PR #1286](https://github.com/gunb-ai/gunbc/pull/1286).

## Scope Boundary

Substrate owns the declarations in `src/v3/std/runtime.dag` and the generated
bootstrap snapshots for the PR-A.3 eager-baseline carriers. Worker A owns the
follow-on verification surface that should land after the Substrate carrier PR
is merged.

Worker A must not edit carrier declarations for this slice. If a test cannot
be written without changing `runtime.dag` or generated bootstrap files, stop
and wait for the Substrate carrier PR.

## Carrier Shape To Verify

The post-Substrate tests must validate exactly these declarations:

```dag
type EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }
type InputEvaluationOrder = LeftFirst
type EvalStateKey { state: EvalStateStack }
type EvalMemoKey { program: DeclarationId, node: NodeId, state_key: EvalStateKey, strategy: EvalStrategy }
```

The test surface must consume the declarations from the generated full
bootstrap DAG. It must not compile an alternate local fixture that could hide a
bootstrap-wiring miss.

## Required Focused Tests

Add focused carrier tests in
[`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs)
after the Substrate carrier PR lands:

- `runtime_eval_strategy_has_closed_eager_baseline_shape`
  - `EvalStrategy` exists in the full bootstrap.
  - Its source span file is `src/v3/std/runtime.dag`.
  - It lowers as `TypeConnective::Disj`.
  - Its only variant label is `ApplicativeOrder`.
  - `ApplicativeOrder` has field `input_order: InputEvaluationOrder`.
- `runtime_input_evaluation_order_is_left_first_only`
  - `InputEvaluationOrder` exists in the full bootstrap.
  - Its source span file is `src/v3/std/runtime.dag`.
  - It lowers as `TypeConnective::Disj`.
  - Its only variant label is `LeftFirst`.
- `runtime_eval_state_key_uses_structural_state_stack`
  - `EvalStateKey` exists in the full bootstrap.
  - Its source span file is `src/v3/std/runtime.dag`.
  - Field `state` resolves to the live `EvalStateStack` declaration.
- `runtime_eval_memo_key_uses_closed_strategy_and_structural_state`
  - `EvalMemoKey` exists in the full bootstrap.
  - Its source span file is `src/v3/std/runtime.dag`.
  - Field `program` resolves to `DeclarationId`.
  - Field `node` resolves to `NodeId`.
  - Field `state_key` resolves to `EvalStateKey`.
  - Field `strategy` resolves to `EvalStrategy`.

These tests should reuse local helpers already present in
`m2_substrate_inhabitance_test.rs` (`find_named`, `conj_field_by_id`, and the
existing `TypeConnective` assertions) rather than introducing a parallel test
harness.

## Negative Assertions

The same focused test slice must assert the deferred/fake inhabitants are still
absent from the generated full bootstrap:

- no `NormalOrder`;
- no `RightFirst`;
- no `EvalThunk`;
- no `RuntimeEvalStrategy`, `RuntimeInputEvaluationOrder`, or other renamed
  product/alias workaround for the locked carrier names.

The `Value` coproduct must also remain unchanged: no `ClosureValue` and no
strategy or memo-key inhabitants inside observable `Value`.

## Parser Prerequisite Coverage

PR #1286 owns parser support and parser regressions for the one-variant sum
syntax. Worker A's follow-on tests do not need to duplicate parser unit tests;
they must prove the runtime carriers survive the full path:

1. authored in `src/v3/std/runtime.dag`;
2. parsed as the intended one-inhabitant sums / records;
3. lowered into the generated full bootstrap;
4. reachable by downstream evaluator tests through declaration lookup.

## Non-Goals

- No edits to `src/v3/std/runtime.dag`.
- No generated bootstrap snapshots in this docs-only planning slice.
- No `EvalThunk`, `NormalOrder`, `RightFirst`, or parallel input order.
- No body evaluator Rust.
- No TC2 strict-equality strengthening.
- No new observable `Value` variants.
