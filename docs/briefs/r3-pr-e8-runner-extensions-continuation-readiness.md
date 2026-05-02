# PR-E E8 - Runner extensions continuation readiness audit

**Status:** READINESS AUDIT - docs-only. This note records the next E8
runner-extension state after the landed bundle and current runner wiring. It
does not authorize new `TestPredicate` variants, substrate changes, target
enumeration, stdout conventions, or broad `test_runner.rs` expansion.

**Parent authority:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E8 - Runner Extension Follow-Ons.

**Bundle authority:** [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md)
splits E8 into W1 `DifferentialEquals` lineage producers, W2 `AlgebraicLaw`,
and W3 `ForAllTargets` producer dispatch. Every runner arm must name its
dissolution target.

## Current State

Current main has one E8 implementation path already beyond the original bundle:

- `AlgebraicLaw::Associativity` and `AlgebraicLaw::Commutativity` are wired in
  `src/v3/compiler/src/test_runner.rs` through bounded operational witness
  tables. They are transitional runner checks, not substrate law-fact
  evaluation.
- `AlgebraicLaw::Identity` remains fail-closed as `NotYetImplemented` because no
  lens identity-element edge is exposed on the algebra inhabitance.
- `AlgebraicLawKind::Distributivity` is still absent from
  `src/v3/std/verification.dag`; any distributivity work remains a P1
  substrate-fact-introduction item, not an E8 runner workaround.
- `ProgramObservation<Carrier>` now exists in `src/v3/std/runtime.dag` as a
  producer-neutral observation envelope. It carries only the typed observed
  value; it does not name producer lineage, target language, stdout/stderr
  channel, exit status, or evaluator strategy.
- `DifferentialEquals` still only dispatches the cost lineage pair
  `v3_program_cost` / `v2_oracle_cost`. The L4 skeleton pair
  `rust_emit_output` / `dag_eval_output` is present as a fixture declaration
  shape, but the runner has no declared producer-dispatch authority for those
  lineages.
- `ForAllTargets` remains scaffolded on raw command / args / exit-code fields
  and has no strict structural-value runner path.

## First Unblocked Follow-On

No additional executable E8 runner slice is cleanly unblocked on current main.

The first non-substrate follow-on is a docs/test-plan slice that consumes the
live `ProgramObservation<Value>` carrier and specifies the missing producer
authority for W1 without implementing it:

1. **Producer identity contract:** name how a `DeclarationRef` in
   `DifferentialEquals` becomes a supported producer (`rust_emit_output`,
   `dag_eval_output`) without name-string dispatch or a new predicate variant.
2. **Observation channel contract:** name how emitted target output is captured
   and normalized into `ProgramObservation<Value>` without making stdout regexes
   a parallel substrate convention.
3. **Evaluator dependency:** require the specific eager evaluator entry point
   and supported `Value` surface needed for `dag_eval_output`.
4. **Dissolution hook:** tie both producers back to PR-B eager evaluator,
   PB-Runtime generated tests, and witness construction as listed in the bundle.

That follow-on can be docs-only now. A runner implementation should wait until
the producer identity and observation channel contracts are live and cited.
The W1-specific blocker/proposal is recorded in
[`r3-pr-e8-w1-output-producer-contract-blocker.md`](r3-pr-e8-w1-output-producer-contract-blocker.md).

## Workstream Gates

| Workstream | Current gate | Allowed next action |
|---|---|---|
| **W1 - `DifferentialEquals` producers** | `ProgramObservation<Value>` exists, but producer identity and observation channel authority do not. `dag_eval_output` also depends on enough eager evaluator semantics to execute the claim body. | Docs-only producer-contract/test-plan slice. Do not add name-keyed runner dispatch for `rust_emit_output` / `dag_eval_output`. |
| **W2 - `AlgebraicLaw`** | `Associativity` and `Commutativity` are already wired. `Identity` waits on a lens identity-element edge; `Distributivity` waits on P1 substrate routing. | Keep fail-closed wording/tests current; no new runner law until the substrate edge exists. |
| **W3 - `ForAllTargets`** | Existing predicate fields are command-shaped and exit-code-shaped. Strict L5 still needs typed target capability and structural observation. | Docs-only readiness updates; no target enumeration or raw-output comparison. |

## Explicit Non-Goals

- No `test_runner.rs` changes in this audit.
- No new `TestPredicate` or `AlgebraicLawKind` variants.
- No `Distributivity` encoding through another law or fixture-local convention.
- No Bool-as-Disj or stdout parsing bridge.
- No target enumeration or `ForAllTargets` execution.
- No new runtime `Value` variants or observation carriers.

## Readiness Verdict

E8 has no further executable runner-extension slice that satisfies the current
scope fence without adding local authority. The next useful step is a
producer-contract/readiness slice for W1 over `ProgramObservation<Value>`; actual
runner code waits on declared producer identity and observation-channel
authority.
