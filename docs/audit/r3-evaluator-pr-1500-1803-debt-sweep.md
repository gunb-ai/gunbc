# R3 Evaluator PR #1500-#1803 Debt Sweep

**Status:** Phase 3 audit receipt for the R3 Evaluator lane. This receipt
packages the Phase 2 PR-history sweep for merged PRs #1500 through #1803 and
is scoped to evaluator-lane debt introductions only.

**Authority:** R3 Evaluator Manager dispatch at
[gunbc#1752 #issuecomment-4389863379](https://github.com/gunb-ai/gunbc/issues/1752#issuecomment-4389863379),
building on the accepted Phase 2 sweep posted at
[gunbc#1743 #issuecomment-4387269008](https://github.com/gunb-ai/gunbc/issues/1743#issuecomment-4387269008)
and manager feedback at
[gunbc#1752 #issuecomment-4387313543](https://github.com/gunb-ai/gunbc/issues/1752#issuecomment-4387313543).

**Methodology:** PR metadata and changed-file lists were checked with
`gh pr list` / `gh pr view`; evaluator-path candidates were spot-checked with
`gh pr diff` and local `rg` against `src/v3/compiler/src/lib.rs`, the E5/E6
briefs, `docs/r3-structure.md`, and `ROADMAP.md`. This is a lane-scoped audit,
not a global debt count.

## Readout

Within merged PRs #1500-#1803, **#1715 is the only true production evaluator
behavior expansion**. It expands the body evaluator in `src/v3/compiler/src/lib.rs`
for E6-G0c by executing existing `TransformTarget::FieldProject` and
Arrow/UserDefined `TransformTarget::Callable` paths through existing evaluator
entry points.

#1715 is disciplined:

- it reuses existing `TransformTarget` shapes;
- it reuses existing `EvalError` shapes;
- it adds no substrate carrier;
- it adds no new dispatch variant;
- it provides implementation evidence for the E6-G0c FieldProject +
  Arrow/UserDefined Callable blocker, anchored in the live
  `eval_transform_node` FieldProject and Arrow/UserDefined Callable arms.

#1799 is correctly classified as a STOP/receipt, not new evaluator debt:
`LoopBound::Descent` remains active and fail-closed as
`EvalError::LoopBoundDescentResidual` until substrate termination proof
authority exists.

#1540, #1568, and #1598 are real hand-Rust or scaffold surfaces, but they sit
outside evaluator execution and should not be charged to PR-E.

## Chronological Rows

| PR | Title | Merged at | Touched evaluator path? | Introduced hand-Rust? | Introduced bridge/debt row? | Dissolution trigger active/retired? | Authority cross-ref |
|---:|---|---|---|---|---|---|---|
| #1503 | feat(evaluator): PR-E E7 -- public-API integration tests for analyze_complexity | 2026-05-02 12:27 UTC | Test-only `src/v3/compiler/tests/integration.rs`; no production evaluator code. | Test-only hand-Rust. | PR body names runner/substrate/Bool bridge work, but no production evaluator bridge found. | Active follow-on; not retired by this PR. | `docs/briefs/r3-evaluator-dispatch.md` §E7; ROADMAP runner/TestClaim debt rows. |
| #1505 | test(evaluator): E7 analyze_complexity root selection coverage | 2026-05-02 13:20 UTC | Test-only integration coverage. | Test-only hand-Rust. | No evaluator bridge/debt row found. | N/A. | `docs/briefs/r3-evaluator-dispatch.md` §E7. |
| #1516 | docs(evaluator): E6 post-blocker gate packet | 2026-05-02 19:07 UTC | Docs only. | No. | No code debt; records E6-G0/G1 boundaries. | Active gate packet; later partially advanced by #1715. | `docs/briefs/r3-pr-e6-post-blocker-gate-packet.md`; `docs/briefs/r3-evaluator-dispatch.md` §E6. |
| #1540 | feat(v3-compiler): PB-1 bin-shim main.rs shell + ProcessExit host mirror | 2026-05-02 20:17 UTC | `lib.rs` only exports PB/bin-shim modules; no `pub mod evaluator` behavior change. | Yes, production PB/bin-shim modules, outside evaluator execution. | PB bin-shim host mirror debt, outside evaluator path. | Active/managed by PB bin-shim lane. | PB bin-shim briefs; ROADMAP PB/bin-shim rows. |
| #1568 | feat(r3): add MethodTemplateContract projection oracle for row 85 | 2026-05-03 10:56 UTC | `lib.rs` only exports `pb_method_template_projection`; no evaluator behavior. | Yes; PR body admits two new hand-Rust files to SG-0 census. | Yes: Row 85 projection oracle hand-Rust with director sign-off. | Active: dissolution trigger is testgen covering reflected-Dag structural assertions over std/row authorities. | `docs/decisions/r3-row85-method-template-read-surface.md`; ROADMAP row 85 / PB-zero method-template projection scope. |
| #1598 | feat(r3): Gap 4 build-step producer -- bounded MethodTemplateContract Map adapter | 2026-05-04 00:44 UTC | `lib.rs` only exports `pb_method_template_projection_dag_emit`; no evaluator behavior. | Yes, producer/adapter hand-Rust. | Yes, bounded Gap 4 adapter; PR body rejects v2->v3.std import bridge and second hand-authored Map authority. | Active until legacy `Single`-template leaf migration / generated structural producer replaces adapter. | ROADMAP MethodTemplateContract / CollectionOps algebra-reframe rows; row 85 adjacency. |
| #1664 | F5: deactivate v3 service parser authority | 2026-05-04 22:55 UTC | `lib.rs` listed, but parser/table/generated-manifest authority churn; no evaluator-symbol patch found. | Generated/parser authority changes, not evaluator hand-Rust. | Not evaluator-lane debt. | N/A for evaluator. | ROADMAP F5 service parser authority row. |
| #1661 | keen-swift-519 | 2026-05-04 23:59 UTC | `lib.rs` patch only reorders/exports `LAYER1_DIAGNOSTIC_KIND_LABELS`; no evaluator behavior. | No evaluator hand-Rust. | No evaluator bridge/debt row found. | N/A. | Diagnostics/verification authority, outside PR-E evaluator execution. |
| #1715 | feat(v3): E6-G0c -- evaluator execution for FieldProject and Callable | 2026-05-05 00:12 UTC | Yes. `eval_transform_node` executes `TransformTarget::FieldProject` and Arrow/UserDefined `TransformTarget::Callable`; adds `eval_callable_body_in_pushed_frame` and focused tests. | Yes, production evaluator Rust. | No new debt row per PR receipt; no census shift, no new substrate carriers, no new `TransformTarget` variants, no new `EvalError` variants. | Implementation evidence for E6-G0c FieldProject / Arrow-UserDefined Callable support. The Phase 3 compile should verify against current `eval_transform_node`; this receipt is not standalone retirement authority. Active residuals remain: G1.b runtime-sourced / Indirect callable dispatch, declared lens-instance data, `DimensionReport` lifting, and `LoopBound::Descent`. | `docs/briefs/r3-pr-e6-lens-fold-readiness-audit.md` G0c / residual sections; `docs/briefs/r3-evaluator-dispatch.md` §E6; current `src/v3/compiler/src/lib.rs` `eval_transform_node` FieldProject and Arrow/UserDefined Callable arms; PR receipt references #1532. |
| #1719 | test(v3): refresh parse corpus manifest | 2026-05-05 00:43 UTC | `lib.rs` touched inside evaluator but patch is formatting of Callable binding collection; no behavior change. | No new hand-Rust surface beyond existing #1715 code. | No evaluator bridge/debt row found. | N/A; housekeeping after parser-manifest drift. | Parser corpus manifest authority. |
| #1725 | docs(evaluator): E6-G0d constructor runtime execution brief | 2026-05-05 04:03 UTC | Docs only. | No. | No code debt; queues constructor runtime execution boundary. | Active at merge time. Post-range PR #1813 implements E6-G0d constructor Callable runtime support; this row is evidence for the Phase 3 compile, not standalone retirement authority. | `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md`; PR #1813 (`feat(v3-eval): E6-G0d constructor Callable runtime execution`); current `src/v3/compiler/src/lib.rs` `eval_transform_node` Callable constructor arms; current `src/v3/compiler/src/lower.rs` E6-G0d constructor helper anchors. |
| #1799 | sharp-ibex-91 -- E5 LoopBound::Descent STOP packet | 2026-05-06 05:27 UTC | Docs/test CI only; no evaluator code. | No production hand-Rust. | No new debt; records STOP boundary for existing Descent residual. | Active STOP: `LoopBound::Descent` remains `EvalError::LoopBoundDescentResidual` until termination-evidence authority exists. | `docs/briefs/r3-pr-e5-loopbound-descent-stop-packet.md`; `docs/briefs/r3-evaluator-dispatch.md` §E5; `src/v3/compiler/src/lib.rs` `eval_loop` / `EvalError::LoopBoundDescentResidual`. |

## Active Evaluator Residuals

The sweep leaves these evaluator-lane residuals active:

- `LoopBound::Descent` execution, blocked on substrate termination proof
  authority and currently fail-closed as `LoopBoundDescentResidual`;
- G1.b runtime-sourced / Indirect callable dispatch, the X1.b S1/S3-shaped
  runtime callee path distinct from G1.a static-representative/report
  production;
- declared lens-instance data plus `Witness` / `OptionalDiagnostic` /
  `DimensionReport` lifting through the body evaluator;
- broader runner/TestClaim hand-Rust pressure from E7 and verification-facing
  tests, which should feed the debt sweep but is not production evaluator
  behavior expansion in this PR range.

## Handoff To Phase 3 Debt Compile

The broader `docs/audit/r3-debt-sweep-2026-05-06.md` Phase 3 compile should
consume these lane-specific rows without treating this receipt as a global
count:

- **#1715** as the only production evaluator behavior expansion in #1500-#1803,
  with no new evaluator debt row. The Phase 3 compile may use #1715 plus the
  current `eval_transform_node` FieldProject and Arrow/UserDefined Callable
  arms as E6-G0c support evidence, but this #1500-#1803 receipt does not by
  itself close that blocker.
- **#1799** as an active STOP receipt for `LoopBound::Descent`, not a new debt
  introduction.
- **#1503 / #1505** as test-only E7 pressure feeding the runner/TestClaim
  hand-Rust pattern, not production evaluator behavior expansion.
- **#1540 / #1568 / #1598** as non-evaluator hand-Rust or scaffold surfaces to
  be charged, if at all, to PB / method-template / build-step lanes rather than
  PR-E.
- **#1725** as a docs-only E6-G0d boundary that was active at merge time. The
  Phase 3 compile may use post-range PR #1813 plus the current
  `eval_transform_node` Callable constructor arms / E6-G0d lowerer helpers as
  retirement evidence, but this #1500-#1803 receipt does not by itself close
  that residual.

## Local Verification

- `git status --short --branch` passed before authoring this receipt.
- `cargo --version` passed before authoring this receipt.
- No evaluator implementation files were edited.
