# R3 Evaluator Phase 5 Post-E3 Closure Handoff

**Status:** handoff packet, 2026-05-06. Docs/audit only. This
packet does not implement evaluator behavior, edit substrate declarations,
touch generated manifests, alter CI, or change runner code.

**Phase 4 anchor:** PR #1855 merged on 2026-05-06 as
`docs/audit/r3-evaluator-phase4-audit-handoff.md`, carrying the post-#1803
evaluator queue and STOP criteria forward from the #1838 / #1839 receipts.

**Post-E3 anchor:** PR #1857 merged on 2026-05-06 as merge commit
`62bec567114ece95e0b3d00598a3eb3f2a28b079`. It landed
`src/v3/compiler/tests/integration/e6_g1a_option3_static_lens_test.rs`, the
E6-G1.a Option 3 mechanism demonstration authorized by PR #1853's
`docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md` and feasibility probe.

## Closed By #1857

#1857 closes only the narrow E6-G1.a Option 3 mechanism work:

- a static top-level `Lens<Int>` representative is consumed by evaluator
  execution;
- static function fields are exercised through `TransformTarget::Callable`;
- non-function lens fields are read through `TransformTarget::FieldProject`;
- `Witness<Int>`, `OptionalDiagnostic`, and `DimensionReport<Int>` are built
  through declared constructors and existing evaluator `Value` carriers;
- read-channel `Violates` remains fail-closed rather than fabricating a
  `String` / `Behavior` to `Diagnostic` lift.

This is a consumer-wiring mechanism demonstration. It is not a semantic
lens-over-`Dag` fold and it does not prove TC1 eta-equivalence against a
reflected program.

## Live Residuals After #1857

- **Q-Reification / reflected program carrier:** real lens-over-`Dag` folding
  remains deferred to `ReflectedProgram<T>` or an equivalent typed
  declaration-reference carrier. The Option 3 representative deliberately uses
  argument-opaque `Dag` / `Behavior` inputs and does not inspect reflected
  program structure.
- **G1.b generic dispatch:** generic `fold_lens<C>` remains held on X1.b
  S1/S3. The merged fire-later brief
  `docs/briefs/r3-pr-e6-g1b-x1b-s3-generic-dispatch-worker.md` is not an
  implementation-readiness claim; it says to dispatch only after the X1.b S1
  carrier and X1.b S3 evaluator runtime-callee authority have landed or been
  explicitly rerouted by Director.
- **E2 / Descent proof consumer:** `LoopBound::Descent` remains fail-closed
  until Substrate lands `descent_execution_proof` or a ratified equivalent.
  PR #1854's `docs/briefs/r3-pr-e2-descent-proof-consumer-worker.md` is a
  pre-authored consumer brief, not implementation readiness.
- **SymbolicCost runner predicate:** `SymbolicCostExprEquals` remains outside
  the body evaluator. Its closure is runner / Pattern B predicate-authority
  work, not a consequence of E6-G1.a Option 3.

## Next Audit Queue

**Queue name:** R3 Evaluator post-E3 closure authority sweep.

**Primary surface:** merged PRs after #1857 that touch or cite any of these
surfaces:

- `src/v3/compiler/tests/integration/e6_g1a_option3_static_lens_test.rs` or
  successor tests claiming broader lens/report coverage;
- `src/v3/compiler/src/lib.rs` evaluator dispatch, runtime `Value`, loop, or
  callable behavior;
- `src/v3/compiler/src/lens_apply.rs`, only when a PR tries to use it as an
  evaluator fold or reflection authority;
- `docs/briefs/r3-pr-e2-descent-proof-consumer-worker.md` and any Substrate
  PR that names the `descent_execution_proof` carrier;
- `docs/briefs/r3-pr-e6-g1b-x1b-s3-generic-dispatch-worker.md` and any X1.b
  S1/S3 landing or reroute;
- Q-Reification / `ReflectedProgram<T>` carrier docs or implementation PRs;
- `src/v3/compiler/src/test_runner.rs` predicate arms for
  `SymbolicCostExprEquals` or DimensionReport-typed Pattern A claims.

**Trigger to fire the queue:** start the sweep when one of these occurs:

1. a PR claims #1857 closed more than the E6-G1.a Option 3 mechanism demo;
2. X1.b S1/S3 lands or is explicitly bundled with G1.b generic dispatch;
3. Substrate lands the Descent proof carrier or Director names an equivalent;
4. Q-Reification / `ReflectedProgram<T>` carrier work lands or changes the
   accepted reflected-program fold boundary;
5. runner predicate work claims closure for `SymbolicCostExprEquals` or a
   DimensionReport-typed Pattern A predicate by citing the Option 3 test.

## STOP Conditions

Stop and route back to the Evaluator Manager instead of landing a local
classification if a sweep finds any of these:

- **Option 3 overclaim:** #1857 is cited as closing lens-over-`Dag` folding,
  TC1 eta-equivalence, generic `fold_lens<C>`, or reflected-program semantics.
- **E2 readiness overclaim:** Descent execution is treated as ready before a
  named Substrate proof carrier and residual taxonomy are present.
- **G1.b readiness overclaim:** generic runtime-sourced callable dispatch is
  treated as ready before X1.b S1/S3 lands or Director explicitly reroutes it.
- **Local reflection authority:** evaluator work routes through `lens_apply`,
  `eval_substrate_reify`, scalar declaration-reference encodings, or host-Rust
  registries to recover reflected-program or callee authority.
- **New runtime mirror:** a PR adds evaluator `Value`, witness, report, lens,
  declaration-reference, or diagnostic mirror state without same-PR authority
  and a dissolution trigger.
- **Runner authority blur:** a runner predicate arm is added or widened without
  its Pattern B / Pattern A authority row and producer contract.
- **Unverifiable authority:** a receipt relies on off-tree comments or stale
  packet names when an in-repo brief, audit file, PR, or code path should be
  the source of truth.

## Phase 5 Compile Handoff

Feed this packet into the next evaluator compile as the post-#1857 boundary:

- closed evidence: #1853 authorized and #1857 delivered the E6-G1.a Option 3
  static-lens consumer-wiring mechanism demonstration;
- still-open evidence: Q-Reification / `ReflectedProgram<T>`, E2 Descent proof
  consumer, G1.b generic dispatch behind X1.b S1/S3, and SymbolicCost runner
  predicate work.

Do not infer global debt counts from this packet. Its purpose is to keep the
post-E3 evaluator closure boundary precise so later sweeps do not silently turn
the narrow Option 3 mechanism into broader implementation readiness.
