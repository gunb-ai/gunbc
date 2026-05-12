# T-Workflow-As-Data — FULL R3-close scope (operator elevation 2026-05-12)

**Author**: deep-wolf-155 (PM)
**Authority**: operator directive 2026-05-12 (Brian; transcript-cited at gunbc#846 / Director inbox routing) — elevate T-CI-Workflow-As-Data to FULL R3-close criterion (not just demonstration). Operator design ratification: emission target is **modeled data** (open enum, invocation-time projection parameter), not a hard-coded emitter choice. **Substrate-shape RATIFIED (c-refined)** per PR #2749 §7 + Director msg_237bde05 / msg_f9fd669e / msg_2a68a4b5: `EmissionTarget` is an invocation-time parameter to `project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow`, NOT a field on any carrier. See §2 for full (a)/(b)/(c) RETRACTED/SUPERSEDED treatment.
**Routing**: Director (zesty-bear-812) **RATIFIED** scope + framing at msg_5cbdad24 2026-05-12 ~06:35Z. Substrate Mgr (warm-wolf-698) lane-absorbs Slices 4-5/8; Verification Mgr (clever-tern-670) absorbs Slice 7 (affected-set integration); Debt-Paydown Mgr (zesty-boar-261) absorbs Slice 6 sub-component (slow-test-exemptions dissolution).
**Parent program**: T-Workflow-As-Data lane in `docs/r3-program-plan.md` §4.4 + §1.8 gates 53-58 + 63.
**Status**: **RATIFIED 2026-05-12** (Director). PR #2744 + downstream Mgr-tier briefs/canvases proceeding.

---

## §0. What changes vs existing T-WAD plan

**Existing scope** (per `r3-program-plan.md` §1.8 gate #56): `ci_workflow_modeled_as_dag` = "at least one workflow as `.dag` data executes through evaluator." Slice 3 demonstration; consumes Slices 1+2 substrate. **Slice 1 LANDED via PR #2160 + #2169** (`WorkflowSecret` + `CronSchedule` carriers — Slice 1 worker brief naming `CronExpression` was superseded by the landed `CronSchedule` model at `dsl/extdeps/cron_schedule_model.dag`; use `CronSchedule` as the grounded name). **Slice 3 demo LANDED via PR #2371** (`t_ci_workflow_as_data_demo.dag` + integration tests).

**FULL R3-close scope** (operator-ratified 2026-05-12; Director-ratified gate-additive framing msg_5cbdad24):
1. **ALL** CI workflow authored as `.dag` (not just one demo workflow) — `.github/workflows/ci.yml` content sourced from gunbc-substrate via projection function `project_github_actions(ci_workflow_dag, target) → Workflow`; WI-2 lands the projection-substrate (new file `dsl/gunbc/ci_emission.dag`); Slice 4-5 implements per-arm projection bodies that walk `CIWorkflowDag` (PR #2736) + emit the chosen target.
2. **Hand-authored `ci.yml` AUTHORITY DISSOLVED** — `.github/workflows/ci.yml` content authored by gunbc emission (not hand-edited); file present as full-emit artifact (YamlStatic) OR thin-shim entry-point (BinaryShim / PythonShim) OR absent (some emission targets); regression-guard test prevents hand-authored re-introduction. **Per briansrls BLOCKING #PR2744 2026-05-12**: NOT file deletion — P5 / Pure Bootstrap dissolves authority; YamlStatic and shim targets STILL require some `.github/workflows/ci.yml` artifact for GH Actions trigger discovery.
3. **EmissionTarget toggle** — same `CIWorkflowDag` (PR #2736 carrier) projects through `project_github_actions(ci_workflow_dag, target)` to multiple target shapes (YamlStatic, BinaryShim, PythonShim, …; `InlineGunbc` is DESIGN-ONLY future target per PR #2746 §5.4, NOT in initial enum); `EmissionTarget` is invocation-time projection parameter (modeled data), NOT carrier-time field. Substrate-shape LOCKED per (c-refined) ratification — see §2 for full treatment.
4. **Affected-set integration** — Layer 2 path-regex bridge (PR #2718/#2721/#2727) dissolved; CI selection consumes affected-set lens output (PR #2713 ✓ merged)
5. **Cost dimension on test nodes** — `slow-test-exemptions.txt` dissolved; slow-test ratchet derives from Cost dimension structurally

## §1. §1.8 gate additions — RATIFIED 2026-05-12 (gate-additive; revised post-(c-refined) ratification)

**Authority**: Director ratification (zesty-bear-812):
- Initial gate-additive framing + 4 NEW gates: msg_5cbdad24 2026-05-12 ~06:35Z
- Revised gate set per (c-refined) canvas ratification + clarification: msg_4f7f536d + msg_f9fd669e 2026-05-12 ~07:00Z (6 NEW gates with substrate-shape vs state-check family splits per kernel-modeling discipline)

**Framing**: GATE-ADDITIVE (NOT scope-expand #56). Reasoning: #56 was authored as demonstration-class (one workflow proves feasibility); scope-expanding to "ALL workflow" would invalidate demonstration semantics + force a post-hoc reclassification of #56's status meaning. Cleaner: #56 stays as demonstration anchor (promotable to PASSING when first .dag workflow lands — likely closing soon given PR #2371 landed the demo); add 6 NEW gates for full-R3 closure surface.

**Citation chain**: operator directive aligns with prior #846 c#4412330468 ("0 hand-Rust including tests AND stage0; bootstrap is data + self-generated") — `.github/workflows/ci.yml` is structurally identical to hand-Rust under that framing. Dissolution is load-bearing for R3 close.

**Substrate-shape ratified** (per PR #2749 (c-refined)): `EmissionTarget` sum-type lives in gunbc namespace; `project_github_actions: (CIWorkflowDag, EmissionTarget) → Workflow` lives in gunbc; extdeps.github.actions.Workflow unmodified. Emission choice is invocation-time data (projection parameter), NOT carrier-time data. See PR #2749 for the comparison canvas authority.

| Gate ID | Predicate-family | Lane | Pass condition |
|---|---|---|---|
| **#56** (unchanged) | demonstration | T-WAD | At least one workflow as `.dag` data executes through evaluator (existing scope — promotable when PR #2371 demo + integration is gate-PASSING) |
| **NEW: `ci_yml_hand_authority_dissolved`** | state-check | T-WAD | `.github/workflows/ci.yml` is no longer hand-authored CI logic — file is either (a) absent (some emission targets may not require a `.github/workflows/` artifact), (b) committed-emission-artifact byte-identical to `project_github_actions(ci_workflow_dag, target)` output with regression-guard test (YamlStatic), or (c) thin-shim entry-point invoking the emitted binary/python (BinaryShim / PythonShim); in NO case is it hand-edited CI logic. Regression guard test prevents hand-authored re-introduction. **Per briansrls BLOCKING #PR2744 inline review 2026-05-12T06:58:55Z**: deleting hand-maintenance ≠ deleting executable artifact; P5 / Pure Bootstrap dissolves authority, not file presence. |
| **NEW: `emission_target_open_enum_landed`** | substrate-shape | T-WAD | `EmissionTarget` sum-type with 3 initial arms (`YamlStatic \| BinaryShim \| PythonShim \| ...`) declared in gunbc namespace as open enum; `InlineGunbc` is DESIGN-ONLY per PR #2746 §5.4 — lands via separate substrate-prereq PR paired with a runtime consumer |
| **NEW: `project_github_actions_landed`** | substrate-shape | T-WAD | `project_github_actions: (CIWorkflowDag, EmissionTarget) → Workflow` projection function declared in gunbc namespace; consumes `extdeps.github.actions.Workflow` as output type; per (c-refined) substrate-shape per PR #2749 §7 |
| **NEW: `test_cost_dimension_landed`** | substrate-shape | T-WAD + Debt-Paydown | `Cost` dimension declared on test nodes (substrate-shape only — splits from state-check sibling below) |
| **NEW: `slow_test_exemptions_dissolved`** | state-check | T-WAD + Debt-Paydown | `scripts/slow-test-exemptions.txt` deleted; slow-test ratchet derives from `Cost` dimension structurally (state-check sibling of `test_cost_dimension_landed` per kernel-modeling discipline split) |
| **NEW: `ci_uses_affected_set_selection`** | state-check | T-WAD + T-Verification | `BinaryShim` emitter consumes affected-set lens output from PR #2713; Layer 2 path-regex `if:` gates removed from any remaining workflow files (cross-tier co-owned with clever-tern-670 Slice 7 work) |

## §2. Architectural shape (operator-ratified 2026-05-12; (c-refined) substrate-shape LOCKED 2026-05-12 per PR #2749 §7)

**`EmissionTarget` as modeled data** (open enum, same shape as `Dimension`):

```dag
type EmissionTarget = YamlStatic | BinaryShim | PythonShim | ...
// InlineGunbc DESIGN-ONLY per PR #2746 §5.4 — NOT in initial enum; lands when runtime consumer exists
```

**Substrate-shape RATIFIED** (per PR #2749 §7 + Director msg_237bde05 + msg_f9fd669e — (c-refined) shape; was option (c) wrapper, refined to projection-function):

**`EmissionTarget` is an invocation-time parameter to the projection function in gunbc-substrate**, NOT a field on any carrier:

```dag
// In dsl/gunbc/ci_emission.dag (NEW file, WI-2 lands it):
project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow

// Invocation pin (canonical YAML-emission point for Slice 4):
gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)
```

- **Input domain**: `CIWorkflowDag` (PR #2736 carrier; gate-dependency-bearing semantic source — flat `CIPipeline` is INSUFFICIENT per warm-wolf-698 msg_27d99080)
- **Output codomain**: `extdeps.github.actions.Workflow` (platform-faithful; UNMODIFIED — INVARIANTS P1)
- **Emission choice**: invocation-time argument (`target: EmissionTarget`); per-arm projection bodies are Slice 4-5 implementation

**Prior framings — RETRACTED / DISSOLVED**:

- **(a) Field on `Workflow` (extdeps platform carrier)** — RETRACTED 2026-05-12 at Director msg_b4151f45 + codex BLOCKING #9970 on PR #2749. INVARIANTS P1 violation: emission_target IS CI logic, not platform fact; cannot land on `dsl/extdeps/github/actions.dag` per its scope header ("platform constraints — not CI logic").
- **(b) Field on `CIPipeline`** — superseded; `CIPipeline { name, gates: List<CIGate> }` is flat without edge structure; cannot serve as projection input per warm-wolf-698 msg_27d99080. Replaced by `CIWorkflowDag` (PR #2736) input domain.
- **(c) Wrapper node** `WorkflowEmission { workflow, target }` — superseded by (c-refined) projection function. Wrapper introduces an extra carrier; projection function is the cleaner shape (parameter-as-data preserves single-substrate-fact + cost-of-change ≤ 1).

**Carrier hierarchy** (per `dsl/extdeps/github/actions.dag` authority — UNMODIFIED):

```
Workflow {                              // :21 — CONCRETE type, codomain for project_github_actions
  on: List<WorkflowTrigger>             // :23
  jobs: List<Job>                       // :24
  env, permissions, ...
}

Job {                                   // :110
  steps: List<Step>                     // :114
  needs: List<String>                   // :115
  runner, strategy, ...
}

Step = RunStep | UsesStep               // :147 — sum type
```

**Emitter dispatch** (parallel to `05_emit_*.dag` consolidation thesis per `project_eliminate_emit_lang_files.md`):

- Slice 4-5 emitter consumes the **pinned-projection invocation** `gunbc_ci_yml_workflow = project_github_actions(ci_workflow_dag, YamlStatic)` (or other `target` for BinaryShim / PythonShim / etc.)
- Per-arm projection bodies (Slice 4 = YamlStatic, Slice 5 = BinaryShim, future = PythonShim) walk `CIWorkflowDag` + emit the chosen target. `InlineGunbc` is DESIGN-ONLY (not in initial enum) per PR #2746 §5.4
- Same `ci_workflow_dag` value across all targets at workflow level; the emission STRATEGY is invocation-time
- A/B comparison is apples-to-apples (call `project_github_actions(ci_workflow_dag, BinaryShim)` vs `project_github_actions(ci_workflow_dag, YamlStatic)`)

**Why BinaryShim is the affected-set unlock**: Static YAML can't naturally express "compute affected-set at PR time, dispatch matrix dynamically." BinaryShim makes runtime decisions structural — binary reads git diff, computes `affected_set = ⋃ over dim of reachable(Δ, dim)`, dispatches matrix accordingly.

**Open expression-substrate question — DEFERRED to canvas (NOT blocking this scope)**: GH Actions expression syntax modeling (`runs-on: ${{ vars.CI_RUNNER || 'ubuntu-latest' }}` etc.) is a Slice 4-5 substrate-shape question routed to warm-wolf-698 canvas authority (PR #2751 — Expression 🟡 YELLOW recommendation; Director ratification pending). NOT a §1 gate.

## §3. Slice expansion

Existing T-WAD slices (state at 2026-05-12):

- **Slice 1**: substrate carriers (`WorkflowSecret { name: SecretName, scope: SecretScope }` + `CronSchedule`) — **LANDED via PR #2160 / #2169** (refined cron model). Brief at `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md`.
- **Slice 2**: timing carriers (`TimingMeasurement` + `TimingObservationSet` + `WorkflowObservationAnchor` + `TimingBudget`); gates #54 #55; gated on T-LBP COMPLETE
- **Slice 3**: demonstration (gate #56) — `t_ci_workflow_as_data_demo.dag` **LANDED via PR #2371** ("post-S4 carriers"); existing `dsl/gunbc/ci.dag` provides gate-centric CI intent. Gate #56 may already be eligible for PASSING promotion; separate state-check sweep.

**New slices added by FULL elevation** (Director-ratified absorption to lanes):

- **Slice 4**: YamlStatic projection-arm implementation — body of `project_github_actions(..., YamlStatic)` walks `CIWorkflowDag` and emits `Workflow` value byte-equivalent to current `.github/workflows/ci.yml` content; consumed by emission pipeline to produce the on-disk artifact → **Substrate Mgr (warm-wolf-698) lane**. Gated on WI-1 + WI-2 PR merges + PR #2736 (CIWorkflowDag) merge. NOTE: NOT an "EmissionTarget field landing" — per (c-refined) shape (§2), `EmissionTarget` is invocation-time projection parameter, not carrier-time data.
- **Slice 5**: BinaryShim emitter implementation → **Substrate Mgr (warm-wolf-698) lane**. Parallel to Slice 4; proves toggle.
- **Slice 6**: Cost dimension on test nodes + slow-test-exemptions.txt dissolution → **Debt-Paydown Mgr (zesty-boar-261) lane** (sub-component). Gated on Phase 3 Cluster M settling (test-as-data substrate stable).
- **Slice 7**: Affected-set integration via BinaryShim → **Verification Mgr (clever-tern-670) lane**. Gated on Slice 5 + PR #2713 (✓ merged).
- **Slice 8**: ci.yml hand-authority dissolution (hand-authored content replaced by emission artifact or thin-shim per chosen `EmissionTarget`) + emission-target toggle finalized + `ci_yml_hand_authority_dissolved` gate PASSING → **Substrate Mgr (warm-wolf-698) lane**. Gated on Slices 4-7 + acceptance verification. **NOTE** (per briansrls BLOCKING #PR2744 2026-05-12): NOT file-deletion — P5 / Pure Bootstrap dissolves AUTHORITY, and YamlStatic / BinaryShim / PythonShim emission paths all require some `.github/workflows/ci.yml` artifact (full-emit or thin-shim) for GH Actions trigger discovery.

## §4. Dependency graph

```
External prereqs:
  - T-LBP COMPLETE → unblocks Slice 2 (timing carriers)
  - Phase 3 Cluster M settling (#2715/#2716, calm-newt-602 + silent-swift-300) → unblocks Slice 6
  - PR #2713 (affected-set lens) ✓ MERGED
  - PR #2160 + #2169 (Slice 1 substrate) ✓ MERGED
  - PR #2371 (Slice 3 demo) ✓ MERGED

Critical path (forward):
  WI-1 canvas merge ───────┐
                           ├─→ Slice 4 (YamlStatic projection-arm) ──┐
  WI-2 ci_emission.dag ────┤                                          │
  (proj-fn substrate)      │                                          │
                           │                                          │
  PR #2736 CIWorkflowDag ──┤                                          │
                           ├─→ Slice 5 (BinaryShim projection-arm) ──┼─→ Slice 7 (affected-set integration) ──┐
                                                          │                                        │
                          Phase 3 Cluster M settles → Slice 6 (Cost dim) ─────────────────────────┤
                                                                                                  │
                                                                                                  ├─→ Slice 8 (ci.yml hand-authority dissolution)
                                                                                                  │
                                                                                                  └─→ all 6 NEW §1.8 gates PASSING

Parallelizable (started 2026-05-12 — dispatched under PM):
  - WI-1 emitter-dispatch architecture canvas (worker still-heron-763)
  - WI-2 new file dsl/gunbc/ci_emission.dag — declares EmissionTarget open enum + project_github_actions function signature + gunbc_ci_yml_workflow pinned-projection (worker cool-carp-720)
```

## §5. Realistic R3-close timing

Assumes:
- T-LBP COMPLETE within 2-3 weeks → Slice 2 dispatches
- Phase 3 Cluster M settles within 2-4 weeks → Slice 6 unblocks
- WI-1 + WI-2 land within 1 week (drafts in flight)
- Slices 4-5 take 1-2 weeks each (emitter implementation)
- Slice 7 (affected-set integration) takes 1-2 weeks
- Slice 8 (ci.yml hand-authority dissolution + acceptance) takes 1 week

**Estimated R3-close-with-FULL**: 6-10 weeks from 2026-05-12. Tight but achievable if external gates (T-LBP, Phase 3 Cluster M) resolve on schedule.

## §6. Immediate parallel work (PM-direct work-items — DISPATCHED 2026-05-12)

**WI-1**: T-WAD emitter-dispatch architecture canvas
- Output: `docs/design-ci-workflow-emitter-dispatch.md` (new)
- Worker: **still-heron-763** (PM auto-spawn 2026-05-12 ~06:30Z)
- Brief: `docs/briefs/r3-t-wad-full-r3-emitter-dispatch-canvas-worker.md`
- Independent of held substrate; Slice 1 LANDED

**WI-2**: NEW file `dsl/gunbc/ci_emission.dag` — projection-function substrate scaffold (per (c-refined) shape)
- Output: NEW file `dsl/gunbc/ci_emission.dag` declaring `EmissionTarget` open enum + `project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow` function signature + `gunbc_ci_yml_workflow` pinned-projection data binding
- Scope: DECLARATION + signature ONLY (per-arm projection bodies are Slice 4-5 owned by Substrate Mgr post-canvas-merge)
- Worker: **cool-carp-720** (PM auto-spawn 2026-05-12 ~06:30Z)
- Brief: `docs/briefs/r3-t-wad-full-r3-cidag-scaffold-worker.md`
- Dependency: PR #2736 (CIWorkflowDag carrier introduction); WI-2 sequences after PR #2736 merge OR rebases on `session/neat-badger-30`
- **Does NOT extend `dsl/gunbc/ci.dag`** — existing gate-centric CI substrate from PR #2371 stays untouched (per (c-refined) shape; INVARIANTS P1 boundary)
- **Does NOT modify `dsl/extdeps/github/actions.dag`** — platform carriers stay platform-faithful (per (c-refined) shape; INVARIANTS P1)

Both workers briefed with brief-pointer messages 2026-05-12 ~06:30Z. Canvas + extension PRs expected within ~1 week.

## §7. Routing — RATIFIED 2026-05-12

**Director (zesty-bear-812) ratified at msg_5cbdad24 2026-05-12 ~06:35Z**:

1. ✅ **FULL R3-close scope ratified** — operator directive aligns with #846 c#4412330468 framing; no R4 carve
2. ✅ **Gate framing: GATE-ADDITIVE** (NOT scope-expand #56); 4 NEW gates per §1 (subsequently revised to 6 NEW per msg_f9fd669e + briansrls BLOCKING fix 2026-05-12 — see §1 current table)
3. ✅ **Owner-Mgr decision: LANE-ABSORB to Substrate Mgr (warm-wolf-698)** with explicit T-CI-WAD program-tag on work-items; no dedicated T-CI-WAD Mgr spawn (avoids standing-Mgr-bottleneck per `feedback_standing_managers_need_owned_deliverables`)
4. ✅ **Sequencing endorsed** (Slice 2 gated T-LBP; Slice 6 gated Phase 3 Cluster M; WI-1/WI-2 dispatched immediately)

**Lane-absorption details** (Director-ratified):
- **Substrate Mgr (warm-wolf-698)**: absorbs Slices 4-5 (emitters) + Slice 8 (ci.yml hand-authority dissolution) into T-WAD lane with T-CI-WAD program-tag; consumes WI-1 canvas + WI-2 extension PR output
- **Verification Mgr (clever-tern-670)**: absorbs Slice 7 (affected-set integration in BinaryShim) — natural extension of PR #2713 ownership
- **Debt-Paydown Mgr (zesty-boar-261)**: absorbs Slice 6 sub-component (slow-test-exemptions.txt dissolution; Cost-dimension on tests integrates with Phase 3 Cluster M test-as-data substrate)

PM-relay to warm-wolf-698 + Director co-sign at #828 if cross-tier confirmation needed.

## §8. Cross-references

- `docs/r3-program-plan.md` §1.4 (Class 4 closure) + §1.8 (gates 53-58, 63) + §4.4 (T-WAD lane)
- `docs/design-affected-set-lens.md` — affected-set lens substrate (PR #2713 ✓ merged); Slice 7 consumes
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 1 substrate carrier brief (LANDED PR #2160)
- `dsl/extdeps/github/actions.dag` — canonical platform carriers (`Workflow:21` `Job:110` `Step:147` `WorkflowSecret:102`)
- `dsl/gunbc/ci.dag` — existing gate-centric CI intent declarations (PR #2371); WI-2 does NOT extend this file (per (c-refined) shape); CIPipeline is NOT used as projection input (flat without edge structure per warm-wolf-698 msg_27d99080)
- `dsl/gunbc/ci_emission.dag` — NEW file WI-2 creates (projection-function substrate; EmissionTarget enum + project_github_actions signature + pinned-projection binding)
- PR #2736 (neat-badger-30) — `CIWorkflowDag` carrier introduction; WI-2 sources projection input from this
- `dsl/extdeps/cron_schedule_model.dag` — CronSchedule carrier (LANDED via PR #2169)
- `docs/ci-pipeline-optimization.md` — pre-v3 historical optimization (background context; not active program)
- `docs/briefs/r3-ci-layer-2-pm-prestaged-mgr-fill-template.md` — Layer 2 path-regex bridge (PR #2721 ✓ merged); dissolution target for Slice 7
- `project_eliminate_emit_lang_files.md` (memory) — omni-emission consolidation thesis; inherited architecture for workflow-target dispatch
- `feedback_audit_adjacent_authority_first` (memory) — grep authority before authoring; rule of record for codex BLOCKING #9970 fix below

## §9. Acceptance-aggregator pattern pilot — Director scaffold LANDED (PR #2748); pilot row REVISED 2026-05-12

Director scaffolded acceptance-aggregator pattern doc at `docs/design-section-1-8-acceptance-aggregator-pattern.md` (PR #2748). Pattern semantics: new §1.8 Family value 'acceptance-aggregator' + new 'depends_on:' column (comma-separated #-refs to constituent rows). Status derived as lattice-meet (DECLARED < CONSUMER_LANDED < PASSING) — machine-checkable; never hand-set. Aggregator rows are VIEWS over constituents; do NOT inflate 97→96 honest-close arithmetic.

**T-CI-WAD pilot row** (Director-ratified at msg_4f7f536d + msg_f9fd669e 2026-05-12; depends_on revised 4→5→6 across Director iterations):

| Proposed | Predicate-family | Lane | depends_on (lattice-meet status) |
|---|---|---|---|
| `t_ci_wad_acceptance` (or PM-named equivalent) | acceptance-aggregator | T-CI-WAD | **#56 + 6 NEW** (from §1): #56, `ci_yml_hand_authority_dissolved`, `emission_target_open_enum_landed`, `project_github_actions_landed`, `test_cost_dimension_landed`, `slow_test_exemptions_dissolved`, `ci_uses_affected_set_selection` |

**Status**: PARKED pending PR #2748 ratification (currently 1/2 approvals). When PR #2748 ratifies, this pilot row inserts into §1.8 ledger as the canonical T-CI-WAD aggregator. Sibling clusters (Cluster M / Cluster F / Cluster K / T-V2-Retirement) get the same template at low marginal cost.

**Row is SHAPE-STABLE** post (c-refined) ratification: constituent gate-IDs finalized at Director msg_f9fd669e. depends_on revisions from #56+4→#56+5→#56+6 reflect canvas-driven gate-set refinement; aggregator semantics unchanged.

**Lattice-meet status derivation**:
- All constituents DECLARED → aggregator DECLARED
- All constituents CONSUMER_LANDED → aggregator CONSUMER_LANDED
- All constituents PASSING → aggregator PASSING
- ANY mix → aggregator status = minimum of constituents (per lattice-meet rule)

---

**End of scoping doc.**

— Authored by deep-wolf-155 (PM) 2026-05-12. Director ratification absorbed msg_5cbdad24. codex BLOCKING #9970 (carrier hierarchy: Workflow > Job > Step) fixed 2026-05-12 — earlier draft incorrectly stated `Workflow<Trigger, Steps, Resources>` generic + flat job→Step mapping; corrected to concrete `Workflow` per `dsl/extdeps/github/actions.dag:21` authority + nested Job-with-Steps hierarchy.
