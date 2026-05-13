---
status: dispatchable (worker brief; ratified shape per canvas PR #2831 Director-ratified Q1=B 2026-05-13 via PM msg_dbc2e5e0 relaying msg_4b13e93f)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #63 `substrate_gap_workflow_scheduling_closed`
parent canvas: PR #2831 / `docs/briefs/r3-substrate-gate-63-workflow-scheduling-canvas.md` — Q1=B RATIFIED with PythonShim carve-out
ratification anchor: PM msg_dbc2e5e0 relaying Director msg_4b13e93f
---

# Gate #63 — workflow_scheduling closure worker brief

## §0. Status — DISPATCH-READY (canvas-merge-gated)

Director ratified Candidate B with PythonShim carve-out per PM msg_dbc2e5e0. Worker dispatch gates on **PR #2831 (canvas) merging**.

Key structural Director-ratified framing (verbatim per relay): "The 5 sibling failures ARE in-scope: they're substrate-compile failures in workflow-as-data ... directly load-bearing for 'modeled as .dag data' criterion. **Cannot be triaged out as 'separately-scoped'; the criterion requires structural-validity-of-the-substrate.**"

The snappy-bear-502 isolation-pass finding is structurally insufficient — passes-in-isolation does NOT establish substrate-validity-of-CIWorkflowDag. Closure requires the substrate compiles + the gate-criterion test passes in the broader `--include-ignored` run + `#[ignore]` is removed.

## §1. Ratified scope (Q1=B with PythonShim carve-out)

- Close YamlStatic + BinaryShim execution paths (both substrate landed)
- **PythonShim out-of-scope** — Tier 1 closure covers 2 of 3 arms
- Closure receipt: at least one **real (non-demo) CI workflow** executes through evaluator → `DimensionReport<TimingMeasurement>`
- All 5 sibling substrate-compile failures resolved
- `#[ignore]` removed at `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581`

## §2. The 5 in-scope sibling failures

Per snappy-bear-502 audit (msg_140d9bc7) + Director structural framing:

1. `dsl/gunbc/ci_emission.dag` unresolved `CIWorkflowDag` / unknown type
2. `gunbc_ci_emission_binary_shim_workflow` opaque body
3. `dsl/gunbc/ci_github_actions_workflow.dag` opaque body + `concurrency` field type mismatch
4. `ci_workflow_as_data_demo_pins_*` topology/command tests fail
5. `gunbc_ci_emission_substrate_compiles` + `gunbc_ci_github_actions_workflow_authority_compiles` fail

(Items 4-5 may be multiple sub-tests; worker audits + enumerates concretely.)

**PythonShim placeholder opaque body** is OUT-OF-SCOPE per PythonShim carve-out — that one specific failure can stay if it's PythonShim-attributable. Worker must distinguish PythonShim failures (out-of-scope) from YamlStatic/BinaryShim failures (in-scope).

## §3. Q3 `#[ignore]` history (Director-verified)

Anchor commit: `73969f4a9` "test(ci): anchor ignored timing demo ignore to ROADMAP P5 deferral" (Brian Searls, 2026-05-12T22:23Z). `#[ignore]` was explicitly anchored to T-WAD substrate-landing arc; removal IS the planned closure receipt. Substrate is now landed; `#[ignore]` removal is the structural closure path.

## §4. Phase A — Diagnose + fix the 5 in-scope failures

1. Run `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test -- --include-ignored --nocapture` and capture full diagnostic output (BuildBuddy invocation cite in PR body)
2. For each of the 5 failures (or 4 if PythonShim-only):
   - Diagnose root cause (unresolved type / opaque body / type mismatch)
   - Identify whether YamlStatic-side, BinaryShim-side, or PythonShim-side (the latter is out-of-scope)
   - Fix at the substrate level — do NOT mask via `#[ignore]` or `#[cfg]` (anti-pattern §6.2)
3. After each fix, re-run the failing test in isolation to confirm green before moving to next
4. Final state: all in-scope failures pass; PythonShim-attributable failures may remain (document explicitly in PR body)

## §5. Phase B — Real CI workflow evaluator receipt

The criterion text "CI workflow modeled as .dag data executes through evaluator" requires evidence beyond the existing demo. Per Director Q1=B:
- At least ONE real (non-demo) CI workflow `.dag` value must execute through evaluator producing `DimensionReport<TimingMeasurement>`
- The repo's own `.github/workflows/ci.yml`-equivalent `CIWorkflowDag` is the natural candidate (`dsl/gunbc/ci.dag:191-200` canonical `ci_workflow_dag`)
- New hermetic test asserts: real CIWorkflowDag → evaluator → DimensionReport<TimingMeasurement> produces non-empty report

## §6. Phase C — `#[ignore]` removal + ratchet

1. Remove `#[ignore]` at `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581`
2. Verify the gate-criterion test (`ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator`) passes under normal `cargo test --workspace` invocation (not just `--ignored`)
3. Add the new Phase-B real-workflow evaluator test alongside (mirrors existing structure)

## §7. Phase D — §1.8 row #63 ledger update

After Phases A+B+C land + tests green:
- Update `docs/r3-program-plan.md` §1.8 row #63 from DECLARED (or CANVAS_RATIFIED if PM ledger-maintenance landed first) → **CONSUMER_LANDED**
- Cite this PR + canvas PR #2831 + Director msg_4b13e93f in the row
- Document PythonShim carve-out explicitly: "2 of 3 WorkflowRuntime arms (YamlStatic + BinaryShim) close gate #63 Tier 1; PythonShim arm R4-deferred"

## §8. Phase E — Cross-Mgr informational notice (Mgr-owned, NOT worker)

Already authored by Mgr (warm-wolf-698) per Q4 ratification. Worker not responsible for this phase; it's recorded here for completeness.

## §9. STOP conditions

1. **Substrate-compile fix surfaces substrate-shape question** (e.g., `CIWorkflowDag` carrier shape is wrong; `concurrency` field type needs structural redesign) — **STOP** and surface to Mgr. Substrate-shape questions go through canvas-tier ratification, not direct-author fix.
2. **PythonShim failure cannot be cleanly distinguished from YamlStatic/BinaryShim failure** — if a failure has cross-arm dependencies that break the carve-out, **STOP** — surface to Mgr for scope re-ratification.
3. **Director-anchored `#[ignore]` commit at `73969f4a9` has been amended/replaced since 2026-05-12T22:23Z** — git log audit at HEAD before authoring; if commit history has changed, **STOP** and surface.
4. **`feedback_load_bearing_ratchet_preservation` violation tempted** — if Phase A authoring tempts adding `#[ignore]` to mask any other failure, **STOP** — anti-pattern §6.2 (silent-mask preservation) fires.
5. **No real CI workflow available for Phase B** — if `dsl/gunbc/ci.dag:191-200` canonical `ci_workflow_dag` is not evaluator-ready, **STOP** — substrate-shape question.
6. **Parallel-authority risk** — if T-Lens-Self-Application Mgr (swift-deer-459) is in-flight on a competing DimensionReport producer route that touches the same evaluator surface, **STOP** and coordinate before authoring.

## §10. Anti-patterns (4 Director-ratified per canvas §8)

PR body MUST cite verbatim + assert receipt-of-compliance:

1. **Closure declared without `#[ignore]` removal** — fail-closed-discipline; un-ignore IS the closure receipt
2. **Silent-mask preservation** — NO new `#[ignore]` added to mask Phase-A failures (§P5 atomic-migration violation)
3. **Parallel-authority on CIWorkflowDag execution path** — if T-Lens-Self-Application has a competing DimensionReport producer shape, surface (do not duplicate)
4. **Demo-bound closure pretending to be production** — Phase B real-workflow receipt prevents this; PR body explicitly cites "non-demo" evidence

## §11. Verification

- `cargo test --workspace` green (NOT just `cargo test -p v3-compiler --test integration`)
- `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test -- --include-ignored --nocapture` shows ≥ 7 passed (was 3 passed, 5 failed; minus PythonShim-attributable remainder)
- Gate-criterion test runs WITHOUT `--ignored` flag (Phase C un-ignore)
- New Phase-B test asserts real CIWorkflowDag → DimensionReport receipt
- PR body cites:
  - Gate #63 closure (Phase D ledger update)
  - Canvas PR #2831 + Director disposition (PM msg_dbc2e5e0) verbatim Q1=B + PythonShim carve-out
  - 4 anti-patterns receipt-of-compliance (§10)
  - 5 in-scope failure resolutions (or 4 if PythonShim-only) — concrete file:line citations
  - `#[ignore]` removal at `t_ci_workflow_as_data_demo_test.rs:581`
  - Director-anchored history commit `73969f4a9` cite

## §12. Out of scope

- **PythonShim arm closure** — R4-deferred per Director carve-out
- **T-Lens-Self-Application substrate work** — OR-semantics; not a prereq, but if competing-shape-risk surfaces, coordinate
- **Cost-lens or other lens consumers of DimensionReport** beyond TimingMeasurement — separate scope
- **Doc-drift sweep on `docs/r3-program-plan.md` §1.7 vs §1.8 row alignment** — separate (Wave-2 doc-drift batch tracked)
- **CI workflow Python integration** — out-of-scope; PythonShim arm carve-out

## §13. Reference

- Canvas: PR #2831 / `docs/briefs/r3-substrate-gate-63-workflow-scheduling-canvas.md`
- Director ratification: PM msg_dbc2e5e0 (relaying Director msg_4b13e93f)
- snappy-bear-502 audit: msg_140d9bc7 + correction msg_cef1340b
- BuildBuddy invocations: `2e1d435a-a6fe-...` (5 failures), `9f22cbce-66ff-...` (isolation pass)
- `#[ignore]`-anchor commit: `73969f4a9` (2026-05-12T22:23Z)
- Gate #63 row: `docs/r3-program-plan.md:291`
- Class 4 framing: `docs/r3-program-plan.md:77` + §4.4
- CIWorkflowDag canonical: `dsl/gunbc/ci.dag:191-200`
- WorkflowRuntime arms: `dsl/gunbc/ci_emission.dag:27`
- Demo entrypoint: `src/v3/std/t_ci_workflow_as_data_demo.dag:206-210`
- Gate-criterion test: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581-582`

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Dispatch gate**: PR #2831 (canvas) merged.
