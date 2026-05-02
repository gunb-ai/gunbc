# PR-E E8 W1 - Output producer contract blocker

**Status:** BLOCKER / PROPOSAL - docs-only. This note converts the W1
STOP+PING for `DifferentialEquals(rust_emit_output, dag_eval_output,
ProgramOutputBind)` into a narrow producer-contract proposal. It does not
change `test_runner.rs`, rewrite fixtures, add substrate shapes, add
`TestPredicate` variants, or execute the L4 corpus.

**Parent authority:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E8 - Runner Extension Follow-Ons and
[`r3-pr-e8-runner-extensions-continuation-readiness.md`](r3-pr-e8-runner-extensions-continuation-readiness.md).

**Verification signal:** [PR #1482](https://github.com/gunb-ai/gunbc/pull/1482)
re-audits Lane 1 after PR-E E5 and identifies W1 runner-extension wiring as
the concrete gate before Lane 1 can consume the authored L4
`DifferentialEquals` row.

## Current Live State

`dag_eval_output` is plausibly available enough for a first no-memo eager
slice. The compiler exposes `evaluate_body` / `eval_node` over the PR-E body
evaluator spine, and the no-memo eager carve-out is an explicit candidate in
the Lane 1 re-audit. A future implementation can evaluate the bind named by
`ProgramOutputBind.output_ref` through the existing evaluator entry, with memo
semantics deferred for slice 1 and named in the runner arm's dissolution target.

`rust_emit_output` remains blocked. It requires emission, Rust compilation,
target execution, and normalization of the produced output into the same
typed value domain observed by `dag_eval_output`. Current main has
`ProgramObservation<Carrier> { observed: Carrier }`, but that envelope is
producer-neutral. It does not identify the producer, target language,
observation channel, exit-status policy, or parse / normalization rule.

The existing L4 fixture declarations for `rust_emit_output` and
`dag_eval_output` are still `miss_int_lookup()` stubs. Replacing those stubs
inside the runner by matching declaration names would make
`test_runner.rs` a second producer language unless the identity contract is
declared up front.

## Minimum Producer Contract

Before runner implementation, W1 needs an explicit answer to how a
`DeclarationRef` in `DifferentialEquals` becomes a supported producer.

Two acceptable contracts are possible:

1. **Transitional DeclarationRef-name contract.** The runner may recognize the
   declaration names `rust_emit_output` and `dag_eval_output` as a temporary
   producer identity contract only if the implementation comment points here,
   rejects any other declaration name fail-closed, and names the dissolution
   target. This is the smallest implementation unblock, but it must be
   explicitly marked as transitional debt rather than treated as substrate
   authority.
2. **Substrate producer role / marker contract.** A P1 substrate proposal adds
   producer roles or markers that declarations inhabit, so the runner selects
   producers by typed role instead of declaration spelling. This is the durable
   direction and avoids name-keyed dispatch, but it is not in scope for a
   docs-only runner-readiness PR.

Either path must keep the existing `DifferentialEquals` and
`ProgramOutputBind` surfaces. W1 does not require a new predicate variant,
new `ProgramInputRole`, fixture-local producer enum, or broad target
enumeration.

## Typed Observation Normalization Options

W1 also needs an explicit observation-channel rule for Rust output into
`ProgramObservation<Value>`.

Acceptable options:

1. **P1 typed observation-channel carrier.** A substrate proposal introduces
   the observation channel and expected value kind, for example stdout vs a
   declared file plus `ValueKind::Int` / `Bool` / record. The runner then reads
   the declared channel and normalizes into `ProgramObservation<Value>`.
2. **Transitional Int-only stdout parse carve-out.** The runner may capture
   emitted Rust stdout, require a single trimmed integer token, convert it to
   `Value::LiteralValue(LiteralBits::Int)`, and reject everything else
   fail-closed. This carve-out is acceptable only if it is explicitly
   authorized as W1 slice-1 debt, restricted to the L4 Int output fixture, and
   marked with its dissolution target.

The dissolution target for any Int-only stdout carve-out is the same one named
by the E8 bundle: PB-Runtime-generated target-language tests and a substrate
owned observation-channel/value-kind surface replace runner-local stdout
parsing. Once that surface lands, the runner must stop treating stdout parsing
as producer authority and read the typed observation fact instead.

## Implementation Fire Criteria

A W1 runner implementation may proceed only when all of these are true:

1. Producer identity is declared by either the explicit transitional
   DeclarationRef-name contract or a substrate role / marker surface.
2. Rust output observation is declared by either a typed observation-channel
   carrier or the explicitly scoped Int-only stdout parse carve-out.
3. `dag_eval_output` uses the real body evaluator entry with no-memo eager
   semantics stated at the call site; it does not call fixture
   `miss_int_lookup()` behavior.
4. Every new runner arm names its dissolution target:
   `rust_emit_output` dissolves into PB-Runtime generated target-language
   tests; `dag_eval_output` dissolves into PR-B eager evaluation plus witness
   construction.
5. Unknown producer pairs, unsupported value shapes, failed Rust execution,
   parse failures, and evaluator errors all remain fail-closed typed runner
   errors.

Until those criteria are met, `eval_differential_equals` should continue to
return `NotYetImplemented` for `(rust_emit_output, dag_eval_output)` rather
than inventing local name dispatch or stdout conventions.

## Explicit Non-Goals

- No `test_runner.rs` changes in this blocker PR.
- No fixture rewrites and no replacement of `miss_int_lookup()` stubs.
- No new `TestPredicate`, `Value`, observation carrier, or substrate role in
  this slice.
- No L5 corpus execution, broad target enumeration, E6/E7 witness work, or
  cross-target byte / string comparison.
- No permanent runner authority. W1 remains transitional runner debt with a
  named dissolution path.
