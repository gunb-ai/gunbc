# R3 Lane 1 Readiness Re-Audit After PR-E E5

**Status:** PROPOSAL — research-only addendum to
`docs/briefs/r3-v-l4-l7-direct-readiness-audit.md`. No substrate edits, no
runner changes, no fixture authoring, and no new `TestPredicate` variants.

**Disposition:** **Option B — still gated.** Lane 1 slice 1 is closer because
the eager body evaluator entry point now exists, but the L4 implementation
worker is not dispatch-ready until W1 wires `rust_emit_output` and
`dag_eval_output` into `DifferentialEquals`.

## HEAD State

| Surface | HEAD evidence | Readiness effect |
|---|---|---|
| PR-A.2 frames | `src/v3/std/runtime.dag:76-90` declares `EvalFrame { bindings: Map<PortId, Value> }` and `EvalStateStack { frames: List<EvalFrame> }`. | Still landed; not a blocker. |
| PR-A.3 eager strategy | `src/v3/std/runtime.dag:93-103` declares `EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }` and `InputEvaluationOrder = LeftFirst`. | Eager/left-first strategy identity is live. |
| PR-A.3 memo carriers | `src/v3/std/runtime.dag` declares `EvalStateKey { state: EvalStateStack }` and `EvalMemoKey { program: DeclarationId, node: NodeId, state_key: EvalStateKey, strategy: EvalStrategy }`. | Structural memo identity is live. The former no-memo eager carve-out gate is closed; W1 producer wiring remains the blocker. |
| Body evaluator | `src/v3/compiler/src/lib.rs:547-553` exposes `pub fn evaluate_body(...) -> Result<Value, EvalError>`. Tests cover the shell at `src/v3/compiler/src/lib.rs:820-829` and branch dispatch at `src/v3/compiler/src/lib.rs:1344-1373`. | The old "no body evaluator entry point" blocker is closed. |
| W1 producer fixture | `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag:19-23` still defines `rust_emit_output` / `dag_eval_output` as `miss_int_lookup()` placeholders, and the fixture header at `:1-4` says runner support is intentionally NYI until wiring lands. | The structural fixture exists, but producers are not real. |
| W1 runner extension | `src/v3/compiler/src/test_runner.rs:2336-2344` still accepts only `(v3_program_cost, v2_oracle_cost)` for `DifferentialEquals`; any other pairing returns `NotYetImplemented`. | `rust_emit_output` / `dag_eval_output` remain unwired. This is the blocking gate. |
| L7 law enum | `src/v3/std/verification.dag:99-107` declares `AlgebraicLawKind = Associativity | Commutativity | Identity`; no `Distributivity` inhabitant exists. | Same enum surface as #1392/#1419. |
| L7 runner | `src/v3/compiler/src/test_runner.rs:2376-2444` wires `Associativity` and `Commutativity`, returns `NotYetImplemented` for `Identity`, and routes non-enum / future laws through P1 framing. | L7 early receipts can continue; full `l7_algebraic_laws_witnessed` remains gated on identity-edge and missing-law substrate work. |

## Fire Criteria Re-Evaluation

The four #1392 fire criteria now evaluate as:

1. **W1 producers landed:** **No.** `DifferentialEquals` still hard-codes the
   Lane-E cost pair, so the L4 `(rust_emit_output, dag_eval_output)` row cannot
   execute.
2. **`dag_eval_output` backed by real evaluator:** **Partially.** `evaluate_body`
   exists and has E5-era runtime coverage, but no `dag_eval_output` producer
   calls it from `TestRunner`.
3. **Memo gap closed or explicitly deferred:** **Closed.** `EvalStateKey` /
   `EvalMemoKey` declarations now exist with structural state and closed
   strategy fields; W1 producer wiring remains independently required.
4. **Worker brief preserves failure taxonomy and fixture path:** **Yes.** The
   existing fixture path remains
   `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`, and
   the failure taxonomy from #1392 still applies.

## Implementation Dispatch Decision

Lane 1 slice 1 should **not** dispatch as a plain L4 implementation worker yet.
The next dispatchable unit is narrower: **W1 runner-extension implementation**
for `DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind)`.
That worker can now cite `evaluate_body` as the oracle substrate for
`dag_eval_output`, but it must still author the producer dispatch and decide the
explicit no-memo eager scope.

Recommended trigger for re-engagement:

- `test_runner.rs::eval_differential_equals` accepts the
  `(rust_emit_output, dag_eval_output)` lineage pair without hitting the
  `NotYetImplemented` branch; and
- `dag_eval_output` invokes the real eager evaluator path rather than fixture
  lookup or placeholder `miss_int_lookup()`; and
- `EvalMemoKey` is consumed as the declared structural memo identity if W1 uses
  memoization; no string/name-only state key is introduced.

Until then, cool-crab / Lane 1 implementation should stay on hold. The concrete
blocker is no longer evaluator critical mass; it is the W1 producer bridge
between the existing `DifferentialEquals` fixture and the now-landed evaluator.
