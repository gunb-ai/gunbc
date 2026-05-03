# R3 T-V-L4-L7-Direct Readiness Audit

**Status:** PROPOSAL — research-only readiness artifact. No substrate edits,
fixture authoring, runner changes, or new `TestPredicate` variants. This brief
updates the standby Lane 1 dispatch surface after the current Evaluator state on
main, especially PR-E E2's `EvalFrame` / `Bind` environment progress.

**Parent authority:** [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)
and [`r3-v-l4-l7-direct-scaffold-notes.md`](r3-v-l4-l7-direct-scaffold-notes.md).
**Closure gates:** [`docs/r3-structure.md`](../r3-structure.md) L54-L56:
`l4_emit_eval_match` and `l7_algebraic_laws_witnessed`.

## HEAD Audit

| Surface | HEAD state | Dispatch consequence |
|---|---|---|
| PR-A.2 frame state | `src/v3/std/runtime.dag` declares `EvalFrame { bindings: Map<PortId, Value> }` and `EvalStateStack { frames: List<EvalFrame> }`; Rust side has `EvalFrame<V>` / `EvalStateStack<V>` helpers in `src/v3/compiler/src/lib.rs`. | Landed enough for environment shape; no longer a blanket Lane 1 blocker. |
| PR-A.3 strategy carriers | `EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }` and `InputEvaluationOrder = LeftFirst` are now declared in `runtime.dag`. | The eager/left-first strategy identity is live. |
| PR-A.3 memo carriers | `EvalStateKey { state: EvalStateStack }` and `EvalMemoKey { program: DeclarationId, node: NodeId, state_key: EvalStateKey, strategy: EvalStrategy }` are declared in `runtime.dag`. | Structural memo-key identity is live; this audit's former memo-carrier blocker is closed. |
| PR-B body evaluator | `r2-pr-b-body-evaluator-eager-baseline.md` and `r2-pr-b-1-eager-evaluator-implementation-seed.md` are design/seed briefs; no full eager evaluator entry point is visible on main. | `dag_eval_output` cannot be a real producer yet. |
| PR-E E2 environment | EvalFrame/Bind environment helpers landed on main via #1374. | Removes a frame API gap, but not the body-execution gap. |
| PR-B.2/W1 runner extension | `r2-pr-b-2-runner-extension-bundle.md` owns `rust_emit_output` + `dag_eval_output`. `test_runner.rs::eval_differential_equals` still accepts only `(v3_program_cost, v2_oracle_cost)`. | L4 fixture row should wait for W1 producer dispatch, not invent a local predicate or stdout convention. |

## Slice-1 Dispatch-Ready Shape

Lane 1 slice 1 should stay the single-target Rust L4 receipt proposed by the
scaffold notes, but fire only after the runner-extension prerequisite is real.

```dag
fn add_then_branch(x: Int, y: Int) -> Int =
  match true {
    True => x + y
    False => x
  }

let l4_out: Int = add_then_branch(1, 2)
```

The seed still fits HEAD: it exercises call, Int arithmetic, constant branch,
and a named output bind while avoiding lists, folds, effects, IO, external
calls, and target-library behavior. Keep the fixture path from the parent brief:
`src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`, suite
`r3_verification_l4_l7_direct_suite`, claim name `l4_emit_eval_match`.

The `DifferentialEquals` row should compare:

- `subject_ref: rust_emit_output`
- `oracle_ref: dag_eval_output`
- `input_ref: ProgramOutputBind { output_ref: l4_out }`

Failure taxonomy for the worker brief:

- **emit failure:** Rust emission/compile fails before execution;
- **target run failure:** emitted Rust artifact fails or exits outside the
  declared observation rule;
- **evaluator failure:** `.dag` body evaluation returns a fail-closed
  diagnostic;
- **value mismatch:** both producers return normalized values, but algebraic
  equality fails.

## Runner Extension Dependency

This audit confirms the runner-extension finding is still live and Evaluator-
owned. `r2-pr-b-2-runner-extension-bundle.md` W1 is the concrete absorption
candidate for L4: add `rust_emit_output` and `dag_eval_output` producer roles
behind the existing `DifferentialEquals` predicate. No Substrate-owned predicate
variant is needed for slice 1.

The open question is sequencing, not predicate shape. `rust_emit_output` can
land independently of body evaluation, but `dag_eval_output` requires a real
PR-B.1 eager evaluator and W1 producer wiring. The PR-A.3 structural memo-key
carriers are now declared, so this audit's former no-memo carve-out question is
no longer the blocking gate for `dag_eval_output`.

Cross-program flag: this is the same structural-observation/value-normalization
surface called out by the Lane 2 readiness work. The shared comparator should
remain the runner-side `runner_structural_values_equal` path rather than a
Lane-1-only comparison helper.

## L7 State

The scaffold notes are partially stale for L7. `AlgebraicLawKind` still declares
`Associativity`, `Commutativity`, and `Identity`; `Distributivity` is still not
an inhabitant. The runner now wires both `Associativity` and `Commutativity`
through bounded operational witness tables in `test_runner.rs`, while
`Identity` remains `NotYetImplemented` because no lens identity-element edge is
exposed on the algebra inhabitance.

Dispatch implication:

- A seed L7 fixture may now cover `Associativity` plus `Commutativity` for the
  same lens-composition witness surface, if the worker brief wants a stronger
  early receipt.
- That seed still does **not** close `l7_algebraic_laws_witnessed`; closure
  requires every algebra in `dsl/std/algebra.dag` to have runtime-constructed
  witnesses for each applicable law.
- `Identity` waits on the identity-element edge.
- `Distributivity` remains a substrate-fact-introduction candidate under
  `INVARIANTS.md` P1; do not encode it through another `AlgebraicLawKind` or a
  fixture-local oracle.

## Fire Criteria

Dispatch the implementation worker when all of these are true:

1. PR-B.2/W1 or an equivalent Evaluator slice lands `rust_emit_output` and
   `dag_eval_output` as declared producers for `DifferentialEquals`.
2. `dag_eval_output` is backed by a real eager body evaluator, not a fixture
   stub.
3. The memo-carrier gap is either closed by `EvalStateKey` / `EvalMemoKey` or
   explicitly deferred for no-memo eager slice 1.
4. The worker brief preserves the four failure classes above and the stable
   fixture path / claim name.

Until then, Lane 1 is no longer in generic standby, but it is still blocked on
the concrete Evaluator runner-extension and body-evaluator surfaces named here.
