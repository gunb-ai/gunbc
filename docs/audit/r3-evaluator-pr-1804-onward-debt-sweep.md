# R3 Evaluator PR #1804-onward Debt Sweep

**Status:** Phase 4 audit receipt for the R3 Evaluator lane. This receipt
extends the cadence past the #1500-#1803 horizon and packages the post-#1803
evaluator-lane PR sweep called for by the Phase 4 handoff.

**End cursor:** merged PRs **#1804 through #2117** (latest merged PR at the
time of authoring, 2026-05-07). PRs after #2117 are out of scope for this
receipt.

**Authority:** Phase 4 audit handoff at
`docs/audit/r3-evaluator-phase4-audit-handoff.md` (#1855); Phase 3 sibling
receipts `docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md` (#1838) and
`docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md` (#1839); lane tracker
issue #1941; this evaluator-cadence work item issue #1973.

**Methodology:** Candidate PR set obtained via
`gh pr list --repo gunb-ai/gunbc --state merged --limit 500 --search
"merged:>=2026-05-06" --json number,title,mergedAt,files`, then filtered to
`number >= 1804` and to changed-file paths matching
`src/v3/compiler/src/(lib|lens_apply|test_runner).rs`,
`docs/briefs/r3-evaluator*`, `docs/briefs/r3-pr-e*`, `docs/briefs/x1b-*`, and
`docs/audit/r3-evaluator-*`. The `--limit 500` window covers all PR numbers
through #2117 with the `merged:>=2026-05-06` floor (default `gh pr list`
limit is 30; the explicit limit is load-bearing for completeness).
Date-floor sanity check: `gh pr list --repo gunb-ai/gunbc --state merged
--limit 1000 --search "merged:<2026-05-06" --json number,mergedAt -q '[.[]
| select(.number >= 1804 and .number <= 2117)] | length'` returned `0`,
confirming no PR with `1804 <= number <= 2117` merged before the floor.
The explicit `--limit 1000` is itself load-bearing here for the same
reason as the primary candidate-set query: without it, `gh` would default
to 30 rows and the check could vacuously return `0`.
Each row was verified with `gh pr view <N> --repo gunb-ai/gunbc`.
This is a lane-scoped audit, not a global debt count, and follows the
Phase 3 conservative-classification discipline: rows already accepted in
#1838 / #1839 are not retroactively reclassified here; instead they are
cross-referenced.

## Readout

Within merged PRs #1804-#2117, **#1813 is the only true production
evaluator behavior expansion**. It implements E6-G0d non-Arrow
`TransformTarget::Callable` support (record + variant constructor runtime
execution) inside `eval_transform_node` in `src/v3/compiler/src/lib.rs`.
The Phase 3 #1500-#1803 receipt already named #1813 as the implementing
landing for the #1725 docs-only E6-G0d boundary; this Phase 4 receipt
records #1813 itself as the production landing inside its own merge range.

#1813 is disciplined:

- it reuses existing `TransformTarget::Callable` shape (Arrow / Variant /
  Record peeling around the same arm);
- it reuses existing `EvalError::TransformArityMismatch`;
- it preserves the fail-closed "Callable target declaration is not an
  Arrow type" residual for other non-Arrow shapes;
- it adds no substrate carrier, no new dispatch variant, and no new
  runtime mirror.

All other evaluator-touching PRs in the range are docs-only briefs, audit
receipts, or citation hygiene — none expand production evaluator behavior.

## Chronological Rows

| PR | Title | Merged at | Touched evaluator path? | Introduced hand-Rust? | Introduced bridge/debt row? | Dissolution trigger active/retired? | Authority cross-ref |
|---:|---|---|---|---|---|---|---|
| #1813 | feat(v3-eval): E6-G0d constructor Callable runtime execution | 2026-05-06 05:31 UTC | Yes. `eval_transform_node` Callable arm in `src/v3/compiler/src/lib.rs` extended for non-Arrow Variant / Record constructor targets (Instantiation + resolved-atom peels; declaration-order field zip; arity ratchet). Also `lower.rs` rustfmt and constructor lowering helpers. | Yes, production evaluator Rust. | No new debt row per PR receipt; no new substrate carrier, no new `TransformTarget` variants, no new `EvalError` variants beyond reusing `TransformArityMismatch`. | Implementation evidence for E6-G0d constructor Callable runtime execution. The Phase 3 #1500-#1803 receipt already cites #1813 as the implementing landing for #1725's docs-only boundary; this row records the in-range landing itself, not standalone retirement authority. | `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md`; `docs/briefs/r3-evaluator-dispatch.md` §E6; current `src/v3/compiler/src/lib.rs` `eval_transform_node` Callable Variant / Record arms; cross-ref `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md` #1725 row. |
| #1823 | docs: replace evaluator brief line citations | 2026-05-06 07:29 UTC | Docs-only citation hygiene across `r2-pb-runtime-evaluator-convergence-matrix.md`, `t-impossiblebugs-nested-optional-flatten-design.md`, `x1b-evaluator-impact-audit.md`. | No. | No code debt; replaces stale `.md:NNN` line citations with stable anchors. | N/A (citation hygiene). | Director / PM citation-hygiene cadence; no evaluator behavior change. |
| #1826 | docs(evaluator): add E6-G1.a static lens fold worker brief | 2026-05-06 07:30 UTC | Docs only — `docs/briefs/r3-pr-e6-g1a-static-lens-fold-worker.md`. | No. | No code debt; queues G1.a static lens fold/report production. | Active queue item; tracks G1.a residual carried in Phase 4 handoff. | `docs/briefs/r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md`; Phase 4 handoff §"Live Residuals From #1838". |
| #1838 | [codex] Add R3 evaluator debt sweep receipt (#1275-#1500) | 2026-05-06 18:44 UTC | Docs only — `docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md`. | No. | Audit receipt itself; no new debt. | Active receipt; consumed by Phase 4 handoff. | `docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md`. |
| #1839 | [codex] add R3 evaluator PR debt sweep receipt (#1500-#1803) | 2026-05-06 18:54 UTC | Docs only — `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md`. | No. | Audit receipt itself; no new debt. | Active receipt; consumed by Phase 4 handoff and by this receipt. | `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md`. |
| #1844 | docs(evaluator): add E3 option3 feasibility probe | 2026-05-06 18:43 UTC | Docs only — `docs/briefs/r3-pr-e6-g1a-option3-feasibility-probe.md`. | No. | No code debt; probe doc for G1.a Option-3 path. | Active probe doc; superseded inside the same range by the Option-3 worker brief #1853. | `docs/briefs/r3-pr-e6-g1a-option3-feasibility-probe.md`. |
| #1853 | docs(evaluator): add E6 G1a option3 worker brief | 2026-05-06 18:55 UTC | Docs only — `docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md`. | No. | No code debt; queues Option-3 static lens worker for G1.a residual. | Active worker brief against G1.a residual. | Phase 4 handoff §"Live Residuals From #1838" G1.a row. |
| #1854 | docs(evaluator): add E2 descent proof consumer brief | 2026-05-06 19:17 UTC | Docs only — `docs/briefs/r3-pr-e2-descent-proof-consumer-worker.md`. | No. | No code debt; queues Descent proof consumer worker. | Active worker brief; held on Substrate termination proof carrier (Phase 4 handoff Descent residual). | `docs/briefs/r3-pr-e5-loop-readiness-audit.md`; Phase 4 handoff §"Live Residuals From #1838" Descent row. |
| #1855 | [codex] Add R3 evaluator Phase 4 audit handoff | 2026-05-06 19:00 UTC | Docs only — `docs/audit/r3-evaluator-phase4-audit-handoff.md`. | No. | Audit handoff itself; no new debt. | Active handoff; this receipt is its Phase 4 compile output. | `docs/audit/r3-evaluator-phase4-audit-handoff.md`. |
| #1877 | [codex] add E6 G1b generic dispatch worker brief | 2026-05-06 19:47 UTC | Docs only — `docs/briefs/r3-pr-e6-g1b-x1b-s3-generic-dispatch-worker.md`. | No. | No code debt; queues G1.b runtime-callee generic dispatch worker. | Active worker brief; held on X1.b S1/S3 (Phase 4 handoff G1.b row). | `docs/briefs/x1b-evaluator-impact-audit.md`; Phase 4 handoff §"Live Residuals From #1838" G1.b row. |
| #1905 | docs(evaluator): add phase5 post-e3 audit handoff | 2026-05-06 22:17 UTC | Docs only — `docs/audit/r3-evaluator-phase5-post-e3-closure-handoff.md`. | No. | Phase 5 audit boundary doc; no code debt. | Active handoff; defines next-cadence boundary after E3 closure. Does not retire any Phase 4 residual on its own. | `docs/audit/r3-evaluator-phase5-post-e3-closure-handoff.md`. |
| #1917 | docs(evaluator): add E8 W1 producer contract test plan | 2026-05-07 01:16 UTC | Docs only — `docs/briefs/r3-pr-e8-w1-producer-contract-test-plan-worker.md`. | No. | No code debt; queues E8 W1 `DifferentialEquals(rust_emit_output, dag_eval_output)` producer-contract test plan. | Active worker brief; sibling Evaluator session `gentle-owl-244` is the implementing lane. | Sibling session inbox #2148 (E8 W1 implementation). |
| #2079 | docs(r3): Substrate Mgr-tier briefs + Q-Reification Gate A receipt | 2026-05-07 07:22 UTC | Touches evaluator-cited briefs `r3-pr-e6-g1a-option3-static-lens-worker.md` and `r3-pr-e8-w1-producer-contract-test-plan-worker.md` plus Substrate-lane briefs and `docs/r3-program-plan.md`. | No. | Cross-lane brief refresh; no evaluator code debt. The evaluator-cited briefs remain queue items, not retirement evidence. | N/A as evaluator-lane debt; substrate-lane Q-Reification Gate A is out of scope here. | Substrate Mgr lane (out of scope for this receipt). |

## Active Evaluator Residuals (carried from Phase 4 handoff)

The Phase 4 handoff §"Live Residuals From #1838" enumerated four live
residuals. None are retired by any #1804-#2117 landing. Status updates
against in-range PRs:

- **G1.a static lens fold/report production** — held. Worker brief queued
  by #1826; Option-3 probe by #1844; Option-3 static lens worker by #1853
  (refreshed in #2079). No production evaluator landing in range.
- **G1.b generic dispatch (`fold_lens<C>`)** — held on X1.b S1/S3. Worker
  brief queued by #1877. No production evaluator landing in range; X1.b
  S1/S3 boundary unchanged.
- **Descent proof consumer (`LoopBound::Descent`)** — held on Substrate
  termination proof carrier. Consumer worker brief queued by #1854. The
  fail-closed `EvalError::LoopBoundDescentResidual` STOP from #1799 is
  unchanged.
- **SymbolicCost runner predicate (`SymbolicCostExprEquals`)** — held;
  remains outside `src/v3/compiler/src/lib.rs` body-evaluator closure and
  inside `test_runner.rs` predicate authority. No in-range PR touched
  `test_runner.rs`; no expansion.

A new evaluator queue item lands in range:

- **E8 W1 producer-contract test plan** (#1917) — queues
  `DifferentialEquals(rust_emit_output, dag_eval_output)` producer-contract
  implementation. Implementation lane is the sibling Evaluator session
  `gentle-owl-244` (inbox #2148); no in-range Rust landing.

## STOP Conditions Cleared

The Phase 4 handoff §"STOP Conditions" enumerated six STOP conditions for
this sweep. Each was checked against the in-range PR set:

- **Callable state contradiction:** none. #1813 expands Callable execution
  in a direction the evaluator-dispatch / readiness docs already declared
  as queued (E6-G0d worker brief #1725).
- **G1.a/G1.b boundary blur:** none. #1826 / #1853 stay scoped to G1.a
  static; #1877 stays scoped to G1.b generic dispatch held on X1.b.
- **Descent without proof carrier:** none. #1854 explicitly queues a
  proof-consumer worker; no PR claims Descent executable.
- **Runner predicate expansion:** none. No in-range PR touched
  `src/v3/compiler/src/test_runner.rs`.
- **New runtime mirror:** none. #1813 reuses existing `Value` shapes
  (`VariantValue` tag is the peeled template id; record fields zipped in
  declaration order) and adds no host-side mirror.
- **Unverifiable PR-range claim:** none. Every cited PR number resolved
  via `gh pr view <N> --repo gunb-ai/gunbc` during sweep authoring.

## Handoff

The next compile pass should consume:

- #1838 / `docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md` (Phase 3a);
- #1839 / `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md` (Phase 3b);
- #1855 / `docs/audit/r3-evaluator-phase4-audit-handoff.md` (Phase 4 boundary);
- this receipt for the #1804-#2117 slice (Phase 4 compile);
- #1905 / `docs/audit/r3-evaluator-phase5-post-e3-closure-handoff.md` for
  the next-cadence (post-E3) boundary.

Conservative-classification discipline preserved: no row from #1838 or
#1839 is reclassified here.

## Local Verification

- `gh pr view <N> --repo gunb-ai/gunbc` resolved every cited PR number in
  the chronological-rows table.
- `git status --short --branch` clean before authoring this receipt.
- No evaluator implementation files were edited; this receipt is
  docs/audit-only.
