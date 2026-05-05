# R3 PR-E E5 LoopBound Descent STOP Packet

**Status:** STOP for implementation. `git-metadata-unavailable`.

This is a docs-only audit packet for R3 Evaluator E5. It packages the accepted
`LoopBound::Descent` residual audit for Director/Substrate coordination. It does
not implement descent execution, change parser/lowerer/runtime code, widen
substrate carriers, add runner behavior, or alter evaluator strategy.

## Dispatch Boundary

Cardinality loop execution is closed. The live evaluator already executes
`LoopBound::Cardinality { count }` with eager accumulator threading, fail-closed
count decoding, and balanced frame handling.

The remaining E5 surface is exactly:

```text
LoopBound::Descent { cluster: ClusterId, measure: PortId }
```

Current behavior must remain fail-closed until a termination-evidence authority
exists. `LoopBound::Descent` must not be reinterpreted as a cardinality loop,
defaulted to zero, widened with new evaluator-local fields, or executed by
locally inferring proof facts in `eval_loop`.

## Current Source References

- [`docs/briefs/r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md)
  records that cardinality `Behavior::Loop` execution is live and descent
  execution remains deferred.
- [`src/v3/compiler/src/lib.rs`](../../src/v3/compiler/src/lib.rs) defines
  `EvalError::LoopBoundDescentResidual { node, cluster, measure }` as the E5
  fail-closed residual.
- [`src/v3/compiler/src/lib.rs`](../../src/v3/compiler/src/lib.rs) implements
  `eval_loop` for `LoopBound::Cardinality`, returning
  `LoopBoundDescentResidual` immediately for `LoopBound::Descent`.
- [`src/v3/compiler/src/lib.rs`](../../src/v3/compiler/src/lib.rs) contains the
  focused evaluator tests for zero iterations, accumulator threading,
  missing/non-integer/negative cardinality counts, descent residual behavior,
  and stack restoration.
- [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) defines
  `MemberDescent`, `IntraClusterCall`, `Cluster`, and
  `LoopBound = Cardinality | Descent { cluster, measure }`.
- [`src/v3/compiler/src/lower.rs`](../../src/v3/compiler/src/lower.rs)
  materializes cluster members and intra-cluster calls, pushes `Dag.clusters`,
  and lowers mutual recursion to `LoopBound::Descent { cluster, measure }`.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) carries the
  Rust mirror for the termination vocabulary: `DescentEvidence`,
  `RankingDimension`, `PositiveDescentAmount`, `DescentSource`,
  `TerminationProof`, and `ProofEdge`.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) also exposes
  `CallDescentEvidence` / `per_call_descent_evidence`, currently a
  Callable-transform side table whose documented coverage is first-slice direct
  self-call arithmetic evidence, with broader callable cases failing closed to
  `SubValueUnknown`.

## Available Facts

- `LoopBound` already preserves the two distinct authorities: an explicit
  runtime count port for cardinality, or a descent cluster plus runtime measure
  port for mutual recursion.
- Lowering can construct `Dag.clusters` entries with
  `NonSingletonList<MemberDescent>` and `NonEmptyList<IntraClusterCall>`.
- `MemberDescent { param: ParamRef }` records the per-member descent parameter
  witness.
- `IntraClusterCall { transform: TransformRef }` records the authoritative
  transform handle for each intra-cluster call.
- Termination vocabulary exists in std and Rust mirrors, including the descent
  evidence lattice and proof/witness carrier shapes.
- E-P has a named per-call evidence side table intended to be broadened as the
  single callable-edge evidence authority.

## Missing Executable Contract

E5 does not yet have an evaluator-consumable contract that maps:

```text
(Dag, ClusterId, measure: PortId)
```

to a validated proof that the descent cluster is executable.

The missing authority must answer at least:

- which `Dag.clusters[cluster]` entry is authoritative for the loop;
- how each `IntraClusterCall.transform` is checked against the cluster's
  `MemberDescent.param` facts;
- what termination evidence is sufficient for runtime execution;
- how evidence is keyed to the `measure` port;
- how uncertified or incomplete evidence fails closed without executing;
- how the evaluator schedules or bounds descent execution once the proof is
  discharged.

The current `per_call_descent_evidence` surface is not enough by itself. It is
Callable-transform oriented, not keyed to `ClusterId`; its documented current
coverage is first-slice direct self-call arithmetic evidence; and it intentionally
returns `SubValueUnknown` for broader callable cases.

## STOP Decision

STOP implementation until termination-evidence authority is assigned.

Do not:

- widen `LoopBound`;
- add evaluator-local proof inference;
- reinterpret `Descent` as cardinality;
- add runner behavior;
- change evaluator strategy carriers;
- reopen cardinality-loop execution;
- fold this into G1 or generic lens-fold work;
- edit parser, lowerer, runtime, or substrate carriers as part of this audit.

## Proposed Authority Shape For Resume

A resumable descent execution slice should first define a Substrate-owned or
Director-assigned proof query that the evaluator can consume directly.

Suggested shape:

```text
descent_execution_proof(
  dag: Dag,
  cluster: ClusterId,
  measure: PortId,
) -> Result<DescentExecutionProof, DescentResidual>
```

The proof query should consume existing facts:

- `Dag.clusters[cluster]`;
- every `IntraClusterCall.transform` in that cluster;
- each cluster member's `MemberDescent.param`;
- the existing `std.termination` / Rust mirror evidence vocabulary;
- any broadened per-call evidence authority needed to certify the cluster.

The proof result should be fail-closed:

- certified clusters may execute through `eval_loop`;
- missing, unknown, incomplete, or non-strict evidence returns a typed residual
  or diagnostic without executing;
- no evaluator-local fallback fabricates proof facts.

Once that authority exists, E5 can replace the current
`LoopBoundDescentResidual` branch in `eval_loop` with a narrow consumer of the
proof query and focused tests proving certified descent executes while
uncertified descent remains fail-closed.

