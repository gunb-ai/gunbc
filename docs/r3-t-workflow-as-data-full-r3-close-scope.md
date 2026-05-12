# T-Workflow-As-Data — FULL R3-close scope (operator elevation 2026-05-12)

**Author**: deep-wolf-155 (PM)
**Authority**: operator directive 2026-05-12 (Brian; transcript-cited at gunbc#846 / Director inbox routing) — elevate T-CI-Workflow-As-Data to FULL R3-close criterion (not just demonstration). Operator design ratification: emission target is a **modeled toggle field on `ci.dag`**, not a hard-coded emitter choice.
**Routing**: Director (zesty-bear-812) ratifies new gate set + lane absorption; Substrate Mgr (warm-wolf-698) absorbs Slice 4-8 into existing T-WAD lane; Verification Mgr (clever-tern-670) absorbs affected-set integration (natural extension of PR #2713).
**Parent program**: T-Workflow-As-Data lane in `docs/r3-program-plan.md` §4.4 + §1.8 gates 53-58 + 63.
**Status**: PROPOSED; Director ratification pending.

---

## §0. What changes vs existing T-WAD plan

**Existing scope** (per `r3-program-plan.md` §1.8 gate #56): `ci_workflow_modeled_as_dag` = "at least one workflow as `.dag` data executes through evaluator." Slice 3 demonstration; consumes Slices 1+2 substrate.

**FULL R3-close scope** (operator-ratified 2026-05-12):
1. **ALL** CI workflow authored as `.dag` (not just one demo workflow)
2. **Hand-authored `ci.yml` DELETED** — replaced by static-regen artifact OR thin shim invoking compiled binary
3. **EmissionTarget toggle** — same `ci.dag` emits multiple target shapes (YamlStatic, BinaryShim, PythonShim, …); choice is a modeled field, not a build-system decision
4. **Affected-set integration** — Layer 2 path-regex bridge (PR #2718/#2721/#2727) dissolved; CI selection consumes affected-set lens output (PR #2713 ✓ merged)
5. **Cost dimension on test nodes** — `slow-test-exemptions.txt` dissolved; slow-test ratchet derives from Cost dimension structurally

## §1. Proposed §1.8 gate additions (Director ratifies)

Proposed new gates (or scope-expansion of existing #56):

| Gate ID (proposed) | Predicate-family | Lane | Pass condition |
|---|---|---|---|
| **#56 (expanded)** | demonstration | T-WAD | ALL ci.yml workflow content modeled as `.dag` (not just demo); current `ci.yml` rebuilds from `ci.dag` via emitter |
| **NEW: `workflow_emission_target_toggle_proven`** | demonstration | T-WAD | Same `ci.dag` emits both YamlStatic + BinaryShim outputs; toggle exercises both emitter paths; equivalence at workflow-semantics level verified |
| **NEW: `ci_yml_dissolved`** | state-check | T-WAD | Hand-authored `.github/workflows/ci.yml` deleted; replaced by static-regen artifact OR thin shim (≤10 lines invoking compiled binary or python-shim) |
| **NEW: `ci_uses_affected_set_selection`** | demonstration | T-WAD + T-Verification | CI selection consumes `affected_set` lens output (per `docs/design-affected-set-lens.md`); Layer 2 path-regex `if:` gates removed from ci.yml |
| **NEW: `test_cost_dimension_landed`** | substrate-shape + state-check | T-WAD + Debt-Paydown | `Cost` dimension declared on test nodes (`Step` carrier or test-as-data substrate); `scripts/slow-test-exemptions.txt` deleted; slow-test ratchet derives from Cost dimension |

Alternative shape: scope-expand existing #56 + #63 (`substrate_gap_workflow_scheduling_closed`) to encompass all five. Director decides which framing is cleaner (gate-additive vs scope-expansion).

## §2. Architectural shape (operator-ratified 2026-05-12)

**`EmissionTarget` as modeled field on workflow node** (open enum, same shape as `Dimension`):

```dag
type EmissionTarget = YamlStatic | BinaryShim | PythonShim | InlineGunbc | ...
type Workflow<Trigger, Steps, Resources> {
  trigger:        Trigger
  steps:          Steps
  resources:      Resources
  emission_target: EmissionTarget    // NEW — modeled field, default YamlStatic
  ...
}
```

**Emitter dispatch** (parallel to `05_emit_*.dag` consolidation thesis per `project_eliminate_emit_lang_files.md`):

- Single emitter reads `emission_target` from `ci.dag`
- Dispatches to YamlStatic emitter (regenerate `ci.yml`) OR BinaryShim emitter (compile binary + emit thin shim YAML) OR PythonShim (emit Python harness + thin shim YAML)
- Same `ci.dag` semantically equivalent across targets at workflow level
- A/B comparison is apples-to-apples (toggle one field, re-emit)

**Why BinaryShim is the affected-set unlock**: Static YAML can't naturally express "compute affected-set at PR time, dispatch matrix dynamically." BinaryShim makes runtime decisions structural — binary reads git diff, computes `affected_set = ⋃ over dim of reachable(Δ, dim)`, dispatches matrix accordingly.

## §3. Slice expansion

Existing slices (already in T-WAD plan):
- **Slice 1**: substrate carriers (`WorkflowSecret<Name>` + `Cron<Schedule>` refinement) — brief at `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md`; **HELD** on auto-spawn fix per #2075
- **Slice 2**: timing carriers (`TimingMeasurement` + `TimingObservationSet` + `WorkflowObservationAnchor` + `TimingBudget`); gates #54 #55; gated on T-LBP COMPLETE
- **Slice 3**: demonstration (gate #56) — bound to neat-badger-30; idle, gated on Slices 1+2

**New slices added by FULL elevation**:
- **Slice 4**: `EmissionTarget` field + YamlStatic emitter — gated on Slice 1 substrate + ci.dag scaffold
- **Slice 5**: BinaryShim emitter — gated on Slice 4 (or parallel); proves toggle
- **Slice 6**: Cost dimension on test nodes + slow-test-exemptions.txt dissolution — gated on Phase 3 Cluster M settling (test-as-data substrate stable)
- **Slice 7**: Affected-set integration via BinaryShim — gated on Slice 5 + PR #2713 (✓ merged)
- **Slice 8**: ci.yml deletion + emission-target toggle finalized — gated on Slices 4-7 + acceptance verification

## §4. Dependency graph

```
External prereqs (already in flight):
  - Auto-spawn fix (cluster ctrl#217) → unblocks Slice 1
  - T-LBP COMPLETE → unblocks Slice 2
  - Phase 3 Cluster M settling (#2715/#2716, calm-newt-602 + silent-swift-300) → unblocks Slice 6
  - Affected-set lens (PR #2713) ✓ MERGED

Critical path:
  Slice 1 → Slice 3 → Slice 4 ──┐
                                ├─→ Slice 7 → Slice 8
                       Slice 5 ─┤              ↑
  Slice 2 ──────────────────────┘              │
                                               │
  Phase 3 Cluster M → Slice 6 ─────────────────┘

Parallelizable (start NOW, independent of held substrate):
  - Emitter-dispatch architecture canvas (forward-looking design)
  - ci.dag scaffold first-draft (uses existing carriers at dsl/extdeps/github/actions.dag)
  - r3-program-plan.md §1.8 gate-addition canonical-ledger update (PM-direct)
```

## §5. Realistic R3-close timing

Assumes:
- Auto-spawn fix lands within 1 week → Slice 1 dispatches
- T-LBP COMPLETE within 2-3 weeks → Slice 2 dispatches
- Phase 3 Cluster M settles within 2-4 weeks → Slice 6 unblocks
- Slices 4-5 take 1-2 weeks each (emitter implementation)
- Slice 7 (affected-set integration) takes 1-2 weeks
- Slice 8 (ci.yml deletion + acceptance) takes 1 week

**Estimated R3-close-with-FULL**: 6-10 weeks from 2026-05-12. Tight but achievable if external gates resolve on schedule. Critical-path risk: T-LBP COMPLETE timing + Phase 3 Cluster M settling.

## §6. Immediate parallel work (PM-direct work-items)

Independent of held substrate; dispatchable now:

**WI-1 (PM auto-spawn)**: T-WAD emitter-dispatch architecture canvas
- Output: `docs/design-ci-workflow-emitter-dispatch.md`
- Scope: How `emission_target` field routes to multiple emitters; consumer pattern; relationship to existing `05_emit_*.dag` consolidation thesis; YamlStatic semantics vs BinaryShim semantics; how affected-set output is consumed at runtime
- Authority: forward-looking; Mgr-tier preliminary (warm-wolf-698 + Director ratify final)

**WI-2 (PM auto-spawn)**: T-WAD ci.dag scaffold first-draft
- Output: `dsl/extdeps/github/ci.dag` (new file)
- Scope: captures current `.github/workflows/ci.yml` jobs + steps + deps + runner selectors as `Workflow<Trigger, Steps, Resources>` instance; placeholder `emission_target: YamlStatic` field
- Authority: forward-looking; consumes existing carriers at `dsl/extdeps/github/actions.dag`; substrate-compatible at HEAD

(Skipping a separate WI for §1.8 gate-addition — PM can author proposal directly in this scoping doc; Director ratifies via this doc's review.)

## §7. Routing requests

1. **Director (zesty-bear-812)**: Ratify FULL R3-close scope; ratify new gate set (or scope-expansion of #56/#63); decide owner-Mgr for Slice 4-8 (warm-wolf-698 absorb OR new T-CI-WAD Mgr spawn)
2. **Substrate Mgr (warm-wolf-698)**: Absorb Slice 4-5 (emitters) + Slice 8 (ci.yml deletion) into T-WAD lane scope; review WI-1 canvas + WI-2 ci.dag scaffold when drafted
3. **Verification Mgr (clever-tern-670)**: Absorb Slice 7 (affected-set integration into CI emitter) — natural extension of PR #2713 affected-set lens ownership; coordinate with Substrate Mgr at slice handoff
4. **Debt-Paydown Mgr (zesty-boar-261)**: Absorb Slice 6 sub-component (slow-test-exemptions.txt dissolution) — falls into existing slow-test ratchet ownership

## §8. Cross-references

- `docs/r3-program-plan.md` §1.4 (Class 4 closure) + §1.8 (gates 53-58, 63) + §4.4 (T-WAD lane)
- `docs/design-affected-set-lens.md` — affected-set lens substrate (PR #2713 ✓ merged); Slice 7 consumes
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 1 substrate carrier brief
- `docs/ci-pipeline-optimization.md` — pre-v3 historical optimization (background context; not active program)
- `docs/briefs/r3-ci-layer-2-pm-prestaged-mgr-fill-template.md` — Layer 2 path-regex bridge (PR #2721 ✓ merged); dissolution target for Slice 7
- `project_eliminate_emit_lang_files.md` (memory) — omni-emission consolidation thesis; inherited architecture for workflow-target dispatch

---

**End of scoping doc.**
