# R3 T-V-L5-Corpus Scaffold Notes

**Status:** PROPOSAL — standby pre-staging notes only. No implementation dispatch; no substrate changes. Parent authority is [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md), which stays sequentially gated on Lane 1 corpus existing plus Shape A target grounding.

## Purpose

These notes freeze the first L5 corpus and runner-shape decisions that can be made before Lane 2 dispatch:

- `l5_cross_target_consistency` compares emitted Rust/Python/Go runtime behavior for the same `.dag` program.
- The comparison is cross-target algebraic equivalence, not target-vs-`.dag` eval and not byte identity.
- Lane 2 consumes Lane 1 corpus **program identity**, not Lane 1 evidence.

Partial coverage below is staging evidence only. It does not close Lane 2.

## Seed Corpus Shape

The Lane 1 `add_then_branch` seed generalizes to L5 if the target emitters can materialize the same observable `Int` result for Rust, Python, and Go:

```dag
fn add_then_branch(x: Int, y: Int) -> Int =
  match true {
    True => x + y
    False => x
  }

let l5_out: Int = add_then_branch(1, 2)
```

Why this remains the right first seed:

- It exercises arithmetic, function call, branch lowering, and output binding.
- It avoids IO, effects, unicode/string library behavior, floats, host clocks, filesystem paths, and target-specific standard-library calls.
- It yields an algebraic `Int` value where Rust/Python/Go equivalence is value equality, not stdout byte equality.
- It is the same semantic program as the Lane 1 seed while preserving the L5 distinction: L5 compares target outputs to each other, not to evaluator output.

If one target lacks branch lowering at dispatch, the fallback seed should be `let l5_out: Int = 1 + 2`, but that is weaker and should be recorded as a temporary narrow seed, not the preferred corpus shape.

## ForAllTargets Runner Path

`TestPredicate::ForAllTargets` exists in `src/v3/std/verification.dag` as a scaffold with `(command, args, expect_exit_code)`, but it is not wired in `test_runner.rs` today. `run_claim` has explicit arms for `ExecuteCommand`, `DifferentialEquals`, `AlgebraicLaw`, and others; unknown variants fall through to `NotYetImplemented("TestPredicate::<name> is not wired in the Rust runner yet")`.

That means L5 has a parallel runner-extension dependency to the L4 `DifferentialEquals` finding:

- `ForAllTargets` can be the existing substrate envelope.
- The runner still needs producer dispatch for each grounded Shape A target.
- The runner must compile/emit the same `TestClaim.source` to Rust, Python, and Go, execute each artifact hermetically, capture the named output, and compare algebraic values.

The current raw command triple is not enough by itself because it checks process exit. L5 needs target-output observation. The eventual runner should either interpret the command as a target harness that emits structured observations, or route through target-specific execution facts from the language spec / runner tables. Do not introduce a new `TestPredicate` variant from this lane.

## Concrete Observation Contract

The first strict L5 row should define:

- `TestClaim.source` as the single `.dag` program authority.
- A named output bind such as `l5_out`.
- A frozen target set derived from the Shape A grounding ledger at dispatch.
- Per-target producer results normalized into a structural value domain: `Int`, `Bool`, simple records, and later lists of those values.
- Failure taxonomy: emit failure, target compile failure, target run failure, observation parse failure, and cross-target mismatch.

For the seed row, expected value can be implicit in the cross-target equality relation; a target-independent oracle value is optional evidence, not the L5 authority. Adding a `.dag` evaluator oracle would turn the row into L4.

## Critical-Path Consumption From Lane 1

Lane 2 consumes the Lane 1 corpus **programs**:

- source text;
- fixture/module naming convention;
- output-bind convention;
- corpus classification metadata, once Lane 1 has it.

Lane 2 does not consume:

- L4 `DifferentialEquals` receipts;
- evaluator output;
- target-vs-eval pass/fail evidence;
- L7 algebraic-law witness rows.

This preserves the categorical split: L4 proves each target matches `.dag` evaluation, while L5 proves targets agree with each other. L5 passing cannot make an L4 receipt redundant, and an L4 receipt is not an L5 input except through the shared corpus program identity.

## Coverage Progression

1. **Slice 1 — seed row:** one `ForAllTargets` L5 row over `add_then_branch`, once Rust/Python/Go emit and run surfaces are ready enough to observe `l5_out`.
2. **Slice 2 — primitive values:** add `Bool`, additional `Int` arithmetic, simple records/conjunctions, and branch variants that avoid target-library semantics.
3. **Slice 3 — collection values:** add lists/maps only after all target runtimes agree on structural observation format, not host string formatting.
4. **Slice 4 — user-program corpus:** import stable programs from the Lane 1 certification corpus and classify each by observable value shape.
5. **Slice 5 — strict fire:** fire `l5_cross_target_consistency` only when the curated corpus is broad enough to represent the accepted certification surface and every materialized row passes for the frozen target set.

Each new program adds evidence; none of the early slices closes Lane 2 alone.

## Standby-Time Decisions

Can freeze now:

- seed program shape;
- shared corpus-program identity rule from Lane 1;
- output-bind convention;
- requirement that comparison is algebraic value equivalence;
- no new predicate variants.

Must wait for dispatch:

- exact target set from the grounding ledger;
- `ForAllTargets` runner implementation;
- output observation format per target;
- corpus breadth threshold for strict `l5_cross_target_consistency`.

## Non-Claims

- No target-vs-`.dag` eval comparison is claimed here.
- No L4 receipt is consumed as proof for L5.
- No byte-equal stdout criterion is accepted as the L5 semantic comparison.
- No partial seed row closes Lane 2.
