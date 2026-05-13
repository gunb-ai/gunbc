---
status: Mgr canvas (substrate-shape question for Director ratification; surfaced per feedback_substrate_shape_belongs_in_mgr_canvas after snappy-bear-502 audit msg_140d9bc7 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #63 `substrate_gap_workflow_scheduling_closed`
authority docs:
  - `docs/r3-program-plan.md:291` — gate #63 row (DECLARED 2026-05-06)
  - `docs/r3-program-plan.md:77` + §4.4 — Class 4 (workflow/scheduling) criterion: "CI workflow modeled as .dag data executes through evaluator + produces DimensionReport<TimingMeasurement>"
  - `docs/r3-structure.md` — T-Workflow-As-Data + T-Lens-Self-Application lane assignment
snappy-bear-502 audit anchor: msg_140d9bc7-6417-45bf-b541-41cba1ea98cd
---

# Gate #63 — workflow_scheduling substrate-shape canvas

## §0. Status

DECLARED at `docs/r3-program-plan.md:291` (NEW 2026-05-06). T-WAD Wave-1 substrate just landed extensively in this Substrate-mgr lane (PR #2774 Slice 4 YamlStatic, PR #2808 Slice 5 BinaryShim, PR #2798 Slice 7 affected-set, PR #2823 gate #62 FileAttachment). snappy-bear-502 was auto-spawned without authored brief and surfaced a clean STOP-condition audit revealing partial-landing state.

This canvas frames the **closure-scope substrate-shape question** for Director ratification before worker brief authoring.

## §1. Source authority (verbatim)

### Class 4 criterion (`docs/r3-program-plan.md:77`)

> CI workflow modeled as `.dag` data executes through evaluator

### Gate #63 row (`docs/r3-program-plan.md:291`)

> `substrate_gap_workflow_scheduling_closed` — substrate-gap-class — T-Workflow-As-Data + T-Lens-Self-Application — DECLARED (NEW 2026-05-06) — CI workflow as `.dag` data; substrate prereqs in §4.4

### Class 4 §4.4 (full closure criterion per snappy-bear-502 grep)

> "CI workflow modeled as .dag data executes through evaluator + produces DimensionReport<TimingMeasurement>"

The closure has **two conjuncts**: (a) execution-through-evaluator + (b) DimensionReport<TimingMeasurement> production.

## §2. State at HEAD — partial-landing inventory (snappy-bear-502 audit)

### What exists

- **CIWorkflowDag carrier**: `dsl/gunbc/ci.dag:120-125` with canonical `ci_workflow_dag` at `:191-200`; includes pipeline/edges + `github_actions_workflow` pinned transport
- **WorkflowRuntime 3-arm**: `dsl/gunbc/ci_emission.dag:27` (per ratified Slice 4 canvas) + `project_github_actions` at `:87-92`
  - YamlStatic arm: returns `dag.github_actions_workflow` (PR #2774 landed)
  - BinaryShim arm: concrete thin-shim Workflow body (PR #2808 landed)
  - PythonShim arm: placeholder data
- **Demo evaluator entrypoint**: `src/v3/std/t_ci_workflow_as_data_demo.dag:206-210` returns `DimensionReport<TimingMeasurement>` — the criterion (b) shape exists
- **Integration evaluator receipt**: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:582` exists

### What's the actual state (corrected per snappy-bear-502 msg_cef1340b 2026-05-13)

**The gate-criterion test PASSES in isolation:**
- Test: `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator`
- Command: `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator -- --ignored --exact --nocapture`
- Result: **1 passed in 5.26s** (BuildBuddy `9f22cbce-66ff-41e0-a172-c2e62a69dee0`)

**But sibling tests in the broader `--include-ignored` run fail:**
- BuildBuddy invocation `2e1d435a-a6fe-437e-9e47-aca68c1ed5a7`: 3 passed, 5 failed
- Failure modes (NOT the gate-criterion test itself):
  - `dsl/gunbc/ci_emission.dag` unresolved `CIWorkflowDag` / unknown type
  - `gunbc_ci_emission_binary_shim_workflow` opaque body
  - PythonShim placeholder opaque body
  - `dsl/gunbc/ci_github_actions_workflow.dag` opaque body + `concurrency` field type mismatch
  - `ci_workflow_as_data_demo_pins_*` topology/command tests fail
  - `gunbc_ci_emission_substrate_compiles` fails
  - `gunbc_ci_github_actions_workflow_authority_compiles` fails

**Reading**: the substrate-shape is **closer-to-closed than first thought**. The gate-criterion test itself passes; the `#[ignore]` masks a passing receipt, not a failing one. The 5 sibling failures are **separately-scoped substrate-debt** in adjacent tests/compiles, not the gate-criterion path.

**This materially shifts the candidate evaluation** (see §3 updates below).

## §3. Closure-scope question — three candidates

### Candidate A — minimal repair (un-ignore the passing test)

**Scope shifted per snappy-bear-502 msg_cef1340b correction**:

- Remove `#[ignore]` from `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` at `t_ci_workflow_as_data_demo_test.rs:581` (test ALREADY PASSES in isolation per BuildBuddy `9f22cbce-66ff-...`)
- Close gate #63 on the unblocked passing receipt
- The 5 sibling-test failures from `--include-ignored` broader run are treated as **separately-scoped substrate-debt** (not gate #63 closure-blockers)

Pros:
- **Smallest possible blast-radius** — single `#[ignore]` removal + test promotion
- Gate-criterion test is **already passing**; gate closure is administrative (un-ignore + ledger update)
- Sibling-test substrate-debt is preserved as known-unknowns separately
- No new substrate authoring needed

Cons:
- "Already passing" status depends on the demo `.dag` exemplifying the criterion correctly — needs sanity check that the demo encodes "CI workflow as .dag data" faithfully
- Sibling-test failures leave structural debt that may re-surface as separate gate failures
- Closes gate #63 narrowly — operator may have intended broader scope per "executes through evaluator" criterion text

### Candidate B — full CIWorkflowDag/projection path execution

Scope:
- Wire the full repo `.github/workflows/ci.yml`-equivalent CIWorkflowDag through evaluator (not just the synthetic demo)
- All 3 WorkflowRuntime arms (YamlStatic + BinaryShim + PythonShim) must execute end-to-end
- DimensionReport<TimingMeasurement> production for at least one real workflow execution
- Fix all 5 failures + remove `#[ignore]`

Pros:
- "Executes through evaluator" reads as production-grade, not demo-grade
- Closes Python placeholder + opaque body shapes structurally
- Forward-compatible with downstream T-Lens-Self-Application

Cons:
- Significantly larger scope (3-5x Candidate A)
- Risk of substrate-shape questions surfacing during implementation (cascades)
- May need cross-lane coordination with T-Lens-Self-Application Mgr
- Cost-of-change-1 satisfied only at the per-workflow level

### Candidate C — wait for T-Lens-Self-Application

Scope:
- Gate #63 closure waits on T-Lens-Self-Application landing first
- T-Lens-Self-Application provides the generic `DimensionReport` producer route that gate #63's evaluator path consumes
- Once T-Lens-Self-Application gates close, return to gate #63 with substrate-prereqs satisfied

Pros:
- Avoids re-doing work if T-Lens-Self-Application changes the DimensionReport producer shape
- Acknowledges the gate row's dual-lane assignment (T-WAD + T-Lens-Self-Application)
- Preserves the partial-landing as substrate-progress receipt

Cons:
- T-Lens-Self-Application lane status unknown at this canvas authoring; may be R4-bound
- Operator may have intended Class 4 to close IN-R3 (cf. operator framing from msg_4fd650b7 on gate #105: "we need to land this all in R3 please")
- Defers known-broken state (`#[ignore]`-masked 5 failures) instead of repairing

## §4. Key load-bearing finding — `#[ignore]` masking

The `#[ignore]` at `t_ci_workflow_as_data_demo_test.rs:581` is the substrate-closure-state load-bearing fact:
- If it was added when the demo was authored (pre-Wave-1 substrate landing), it represents **deferred fail-closed receipt** awaiting the substrate landing
- If it was added after a regression, it represents **silently-masked broken state**
- Worker audit at HEAD shows the substrate IS landed; therefore the `#[ignore]` should be removable IF Candidate A repair is in scope

**Per `feedback_load_bearing_ratchet_preservation` discipline**, `#[ignore]` masking on gate-criterion tests is a fail-closed-discipline anti-pattern; any closure-scope choice should plan for `#[ignore]` removal.

## §5. Cross-lane coupling — T-Lens-Self-Application

Gate row says lane = `T-Workflow-As-Data + T-Lens-Self-Application`. Two lane interpretations:

1. **AND semantics**: gate #63 requires both lanes' substrate to land before closure → Candidate C
2. **OR / contributory semantics**: either lane's substrate contributes to closure; closure proceeds when sufficient substrate is present → Candidate A or B

Worker audit didn't probe T-Lens-Self-Application substrate state. Canvas authoring at HEAD doesn't have visibility into that lane's progress. Director-tier disposition needed.

## §6. Practice 4 (coproduct dissolution) check

No new sum-type proposed in this canvas. All three candidates work with existing carriers (CIWorkflowDag + WorkflowRuntime + DimensionReport). Practice 4 GREEN/YELLOW/RED not applicable.

## §7. Cost-of-change accounting

| Candidate | Files edited to close gate #63 |
|---|---|
| A (demo repair) | ~3-5 (fix 5 specific failures + remove `#[ignore]`) |
| B (full path) | ~10-20 (close 3 WorkflowRuntime arms + repo workflow path + cross-cutting infra) |
| C (defer) | 0 (now); unknowable later |

## §8. Anti-patterns (Mgr-derived for reviewer enforcement)

1. **Closure declared without `#[ignore]` removal** — fail-closed-discipline (§4); any closure must un-ignore the gate-criterion test
2. **Silent-mask preservation** — adding more `#[ignore]` to mask new failures (§P5 atomic-migration violation)
3. **Parallel-authority on CIWorkflowDag execution path** — if T-Lens-Self-Application has a competing DimensionReport producer shape, canvas should surface, not duplicate
4. **Demo-bound closure pretending to be production** — if Candidate A repairs only demo evaluator but the criterion text intended broader scope, that's a P3 second-source-of-truth (demo passes; production doesn't)

## §9. Open questions for ratification

Director ratification on:

- **Q1 — Closure-scope candidate**: A (demo repair) / B (full path) / C (defer to T-Lens-Self-Application)
- **Q2 — Lane semantics**: T-WAD + T-Lens-Self-Application AND vs OR-semantics
- **Q3 — `#[ignore]` history**: was this `#[ignore]` planned-deferral (substrate-then-unignore) or regression-mask? Director may have grep visibility on the test's `#[ignore]` introduction commit; if so, that disambiguates A/B/C
- **Q4 — Cross-lane coordination**: does Substrate Mgr coordinate with T-Lens-Self-Application Mgr for closure, or is gate #63 substrate-lane-owned end-to-end?

## §10. Mgr recommendation (REVISED per snappy-bear-502 msg_cef1340b)

**Initial recommendation** (pre-correction): Candidate B with PythonShim carve-out.

**Revised recommendation** (post-correction): **Candidate A with sibling-debt receipt** — given that the gate-criterion test passes in isolation, gate #63 closure is administrative (un-ignore + ledger). Candidate B would re-author already-working substrate; that's `feedback_no_short_term_solutions` inverted (don't redo passing work).

Recommended **revised scope**:
- Q1=A: Un-ignore `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` (already passing per `9f22cbce-66ff-...`)
- Document the 5 sibling-test failures as **separately-scoped substrate-debt**; surface to Mgr for follow-on triage but NOT block gate #63 closure
- Close gate #63 on the unblocked passing receipt + ledger update
- Optional follow-on canvas: if criterion text "executes through evaluator" requires more than demo-grade evidence, surface a Tier-2 expansion canvas after gate closure

**Q2: OR-semantics** — both lanes contribute; gate closes when sufficient substrate is present.

**Q3: defer to Director** — needs `git log` history check on the `#[ignore]` introduction commit. If `#[ignore]` was added pre-substrate-landing as planned-deferral, Candidate A is the correct receipt. If post-substrate as regression-mask, Candidate A still works but the regression-mask history should be archived.

**Q4: Substrate-lane-owned** with informational cross-Mgr notification when authoring.

**Sibling-debt note**: the 5 failures from `--include-ignored` are separately worth tracking. Worker brief Phase D could include "surface sibling-debt audit document" with the 5 specific failure modes as a Mgr-tier follow-on triage item, not a gate #63 closure-blocker.

## §11. Reference

- snappy-bear-502 audit: msg_140d9bc7-6417-45bf-b541-41cba1ea98cd
- BuildBuddy diagnostic invocation: `2e1d435a-a6fe-437e-9e47-aca68c1ed5a7`
- Gate #63 row: `docs/r3-program-plan.md:291`
- Class 4 framing: `docs/r3-program-plan.md:77` + §4.4
- CIWorkflowDag: `dsl/gunbc/ci.dag:120-125`
- WorkflowRuntime: `dsl/gunbc/ci_emission.dag:27`
- Demo entrypoint: `src/v3/std/t_ci_workflow_as_data_demo.dag:206-210`
- `#[ignore]`-masked test: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581-582`

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
