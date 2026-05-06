# R3 Evaluator PR #1275-#1500 Debt Sweep

**Status:** audit receipt, 2026-05-06. This is a source/PR-history sweep, not
an implementation brief.

**Scope:** merged PRs #1275-#1500 that touched
`src/v3/compiler/src/lib.rs` evaluator surfaces:
`evaluator::Value`, `eval_value`, `eval_port`, `eval_node`, `eval_branch`,
`eval_loop`, `eval_bind`, and `eval_transform_node`.

**Verification inputs:** local `git log --all -- src/v3/compiler/src/lib.rs`,
`gh pr view` for the PR rows below, live `src/v3/compiler/src/lib.rs`, and
the current R3 routing docs:

- `docs/r3-program-plan.md` §10.3, especially
  `Q-EVAL-Descent-Termination-Contract`,
  `Q-EVAL-Lens-Fold-First-Slice`, and
  `Q-Pattern-A-First-Slice-Subscope`.
- `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md`.
- `docs/briefs/r3-pr-e6-lens-fold-readiness-audit.md`.
- `ROADMAP.md` Pattern B / `test_runner.rs` predicate-authority debt.

## Summary

The PRs that materially built the R2 evaluator body surface are legitimate
hand-Rust capacity slices, not accidental parallel substrate authority rows:
#1374, #1387, #1407, #1426, #1476, and #1496.

The material residual classifications are:

- #1426 introduced an explicit Bool literal-vs-variant branch identity gap.
  That gap is retired by #1467/#1473 through shared
  `Dag::bool_runtime_variant_id` authority.
- #1476 keeps `LoopBound::Descent` fail-closed. This remains active and routes
  through the Substrate termination-proof carrier before Evaluator consume-side
  wiring.
- #1407/#1496 left callable pressure for later E6 work. The non-Arrow
  constructor `Callable` residual is now retired by E6-G0d / #1813. The
  remaining E-series callable pressure is G1.a static lens fold/report
  production; G1.b generic dispatch remains held on X1.b S1/S3.
- #1484 is not a body-evaluator regression. `SymbolicCostExprEquals` remains
  a runner/test-predicate gap tracked outside the `lib.rs` body evaluator.
- #1459 and #1466 touched `lib.rs` via broader generated/substrate fallout,
  but this sweep found no evidence promoting either into evaluator-lane causal
  history.

## Chronological Rows

| PR | Merged-at | Evaluator-path touch | Hand-Rust / debt classification | Current dissolution state |
|---|---:|---|---|---|
| #1374 `feat(evaluator): PR-E E2 EvalFrame lookup and Bind environment` | 2026-05-01T07:04:37Z | `EvalFrame` / evaluator stack lookup helpers in `lib.rs`. | Hand-Rust capacity slice inside the existing evaluator module. PR body explicitly excludes runtime carrier changes, list fallback, and body evaluation. | Consumed by later E1/E4/E5/Bind slices. No new debt row found. |
| #1387 `feat(evaluator): PR-E E1 value behavior execution` | 2026-05-01T08:51:37Z | `Behavior::Value`, `eval_value`, `eval_port`, `eval_node`, and `evaluate_body`. | Hand-Rust evaluator capacity. Uses the Rust runtime `Value` mirror and constructs only `LiteralValue` in this slice. | Active foundation under the broader PB-zero hand-Rust retirement trajectory. |
| #1407 `feat(evaluator): PR-E E3 eager Transform application` | 2026-05-01T20:04:31Z | `eval_transform_node` for supported operator targets. | Hand-Rust capacity plus explicit unsupported transform fences. These are fail-closed residuals, not hidden bridge carriers. | FieldProject and Arrow/UserDefined callable execution later retired by E6-G0c. Non-Arrow constructor `Callable` is retired by E6-G0d / #1813. Static lens fold/report production remains G1.a; generic dispatch remains G1.b held on X1.b S1/S3. |
| #1425 `fix(v3): P0 DB-8 receipt.json key pin + SG-0 census` | 2026-05-01T20:04:41Z | Touched `lib.rs`, but not evaluator-lane behavior. | Receipt-key / SG-0 census work outside the evaluator lane. | Not an evaluator debt row. |
| #1426 `feat(evaluator): PR-E E4 Branch arm coverage` | 2026-05-01T21:29:41Z | `Behavior::Branch`, `eval_branch`, `Value::VariantValue`, branch frame/payload path execution. | Hand-Rust capacity plus explicit Bool literal-vs-variant identity gap. The PR deferred Bool `if` evaluation to shared substrate identity instead of adding a local map. | Retired by #1467/#1473 for evaluator and `lens_apply` Bool matching. |
| #1459 `keen-swift-519` | 2026-05-02T03:49:09Z | Touched `lib.rs` amid broad generated/substrate changes. | No evaluator-specific hand-Rust/debt introduction identified from PR metadata available in this sweep. | Do not promote into evaluator causal history without narrower file/body evidence. |
| #1466 `feat(std): T-Numeric-Construction Slice 3` | 2026-05-02T01:41:30Z | Touched `lib.rs` through regenerated/bootstrap fallout. | Numeric-construction substrate/std work, not evaluator semantics. | Not an evaluator debt row. |
| #1467 `fix(v3): reify Bool branch scrutinees through shared variant identity` | 2026-05-02T01:15:31Z | Evaluator Bool branch reification; also `dag.rs` and `lens_apply.rs`. | Retires bridge risk by centralizing on `Dag::bool_runtime_variant_id`; does not introduce a local Bool declaration-id map. | Retires #1426 Bool branch identity gap. |
| #1473 `test(evaluator): cover Bool branch reification` | 2026-05-02T03:48:42Z | Evaluator tests in `lib.rs`. | Test-only hand-Rust; adds fail-closed coverage when Bool authority is missing. | Reinforces #1467 retirement. |
| #1476 `feat(evaluator): PR-E E5 loop cardinality execution` | 2026-05-02T03:49:06Z | `Behavior::Loop`, cardinality execution, accumulator frame threading. | Hand-Rust capacity. Keeps `LoopBound::Descent` as a typed fail-closed residual. | Active. `docs/r3-program-plan.md` §10.3 routes this through `Q-EVAL-Descent-Termination-Contract`; evaluator consumes the Substrate proof token after that carrier lands. |
| #1484 `feat(evaluator): PR-E E7 symbolic-cost-only analyze_complexity wrapper` | 2026-05-02T04:57:42Z | Touched `lib.rs`; primary wrapper is in `dimension.rs`. | Thin wrapper over the existing symbolic-cost analyzer. No parallel analyzer and no new variants per PR body. | `SymbolicCostExprEquals` remains a runner/test-predicate gap, tracked in `docs/r3-program-plan.md` V2 and ROADMAP Pattern B, not as a body-evaluator regression. |
| #1496 `feat(evaluator): PR-E Bind callable entry` | 2026-05-02T08:33:41Z | `Behavior::Bind`, `eval_bind`, callable frame entry prerequisite. | Hand-Rust capacity. PR body excludes E6 lens fold, runner work, substrate changes, and new runtime carriers. | Consumed by E6-G0c/G0d and the G1 sequence. Current live pressure is G1.a static lens fold/report production; G1.b generic dispatch remains held on X1.b S1/S3. |

## Live Follow-Up Map

- `LoopBound::Descent`: active fail-closed evaluator residual. Owner path is
  Substrate termination-proof carrier, then Evaluator consume-side wiring.
- E6 callable/report path: constructor runtime execution is retired by #1813.
  Next executable evaluator slice is G1.a static lens fold/report production.
  G1.b generic dispatch stays deferred behind X1.b S1/S3.
- Runner/test-predicate authority: `SymbolicCostExprEquals` and related
  predicate execution gaps live under the R3 plan V2 and ROADMAP Pattern B
  runner-freeze rows, not in the #1275-#1500 `lib.rs` body-evaluator sweep.
