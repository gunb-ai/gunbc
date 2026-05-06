# R3 PR-E E2 — Descent Execution Proof Consumer Worker

**Status:** pre-authored worker brief. Do not dispatch until the substrate
`descent_execution_proof` carrier/query lands or Director/Substrate explicitly
names an equivalent authority.

**Lane:** R3 Evaluator E5 residual closure / E2 consumer wiring.

**Dispatch trigger:** Substrate merges a proof authority with the shape:

```text
descent_execution_proof(
  dag: Dag,
  cluster: ClusterId,
  measure: PortId,
) -> Result<DescentExecutionProof, DescentResidual>
```

or an explicitly ratified equivalent that lets the evaluator distinguish
certified `LoopBound::Descent` execution from missing, unknown, incomplete, or
non-strict evidence without inferring proof facts locally.

## Source Authority

- [`r3-pr-e5-loopbound-descent-stop-packet.md`](r3-pr-e5-loopbound-descent-stop-packet.md)
  is the active STOP receipt.
- [`docs/r3-program-plan.md`](../r3-program-plan.md) EVAL-2 names the resume
  contract and fail-closed residual classes.
- `src/v3/compiler/src/lib.rs` currently returns
  `EvalError::LoopBoundDescentResidual { node, cluster, measure }` from
  `eval_loop` for `LoopBound::Descent`.
- `src/v3/std/substrate.dag` owns the live
  `LoopBound::Descent { cluster: ClusterId, measure: PortId }` shape plus the
  cluster/member/call topology carriers.

## Goal

Replace the evaluator's unconditional `LoopBound::Descent` residual with a
narrow consumer of the substrate proof query:

- certified descent loops execute through the existing loop evaluator path;
- uncertified descent loops continue to fail closed with a typed residual;
- the evaluator never computes, approximates, or repairs termination evidence.

This is a consumer slice. The proof producer, evidence lattice, cluster
coverage, and per-call evidence broadening remain Substrate-owned.

## Required Implementation Shape

In `eval_loop`, when `node.bound` is `LoopBound::Descent { cluster, measure }`:

1. Call the substrate-owned proof query with the current `Dag`, `cluster`, and
   `measure`.
2. If the query returns a certified proof token, execute the loop body using
   the existing evaluator frame/accumulator discipline. Reuse the existing
   cardinality-loop execution helpers where possible, but do not reinterpret
   descent as `LoopBound::Cardinality`.
3. If the query returns a residual such as missing, unknown, incomplete, or
   non-strict evidence, return a fail-closed `EvalError` carrying the existing
   `node`, `cluster`, and `measure` context plus the proof residual if the
   landed substrate type exposes one.
4. Preserve stack restoration behavior on success and failure.

If the landed substrate API does not expose an executable bound/schedule for
the certified loop, STOP. The evaluator cannot invent an iteration count or
termination schedule from proof existence alone.

## Hard Bars

Do not:

- widen `LoopBound`;
- add evaluator-local proof inference;
- call `per_call_descent_evidence` directly as a substitute for the proof
  query unless Substrate explicitly makes it the proof query;
- reinterpret `Descent` as cardinality or default to zero/one iteration;
- add runner behavior or `TestPredicate` arms;
- change parser, lowerer, or substrate carriers;
- add a second termination-evidence mirror in evaluator code;
- collapse residual classes into string matching.

Any of those needs a STOP back to Substrate/Director.

## Acceptance

The PR must include focused evaluator tests for:

1. Certified descent proof executes the loop body and returns the expected
   accumulator value.
2. Missing/unknown/incomplete/non-strict proof residuals fail closed without
   executing the body.
3. The old unconditional `LoopBoundDescentResidual` behavior is replaced only
   for certified proof tokens.
4. Stack/frame restoration remains correct after certified execution and after
   residual failure.
5. The evaluator does not call any local proof-inference helper or evidence
   side table directly unless that helper is the ratified substrate proof
   authority.

Validation should include the narrow evaluator test target that covers
`eval_loop` plus repository format/check commands required by touched files.

## Non-Goals

- Substrate proof carrier design.
- Per-call descent evidence broadening.
- Termination lens behavioral parity.
- TC3 producer/evaluation-step work.
- `LoopBound::Cardinality` refactors.
- Runner predicate changes.

## Handoff Notes

If the substrate authority lands under a different name than
`descent_execution_proof`, update this brief before dispatch with the exact
function/type names and residual variants. The dispatch decision should be
mechanical once the proof token gives the evaluator both permission and enough
execution information to run the descent loop without fabricating proof facts.
