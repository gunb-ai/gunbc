# Worker brief — T-WAD FULL R3 emitter-dispatch architecture canvas

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-1; operator FULL elevation 2026-05-12 (toggle design ratification); Director ratification msg_5cbdad24 2026-05-12 (Slice 4-5/8 lane-absorbed to Substrate Mgr warm-wolf-698).
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698); this is forward-looking design canvas; final Mgr-tier ratification routes through warm-wolf-698, with Director co-sign at #828 if cross-tier confirmation needed.
**Closure predicate**: design canvas authored + Mgr-ratified shape; downstream Slice 4-5 emitter implementation work consumes this canvas.

## Output

`docs/design-ci-workflow-emitter-dispatch.md` — new design canvas, ~400-800 lines target, matching the shape of `docs/design-affected-set-lens.md` (single-authority, locked-design).

## Scope

Design the emitter-dispatch architecture for T-Workflow-As-Data FULL R3-close:

1. **`EmissionTarget` open-enum shape** — `EmissionTarget = YamlStatic | BinaryShim | PythonShim | InlineGunbc | …`; default value if absent (`YamlStatic` recommended baseline; default-toggle behavior preserves current ci.yml-equivalent regen)
2. **Where `emission_target` is carried** — OPEN QUESTION; canvas should evaluate three placements and propose one:
   - **(a) Field on `Workflow`** (`dsl/extdeps/github/actions.dag:21`): platform-level — but actions.dag is "platform constraints, not CI logic" per its header; emission_target IS a CI logic choice, so this may violate the separation
   - **(b) Field on `CIPipeline`** (`dsl/gunbc/ci.dag`): CI-intent-level — emission_target is a transport choice the CI consumer makes; natural fit per actions.dag's "consumers: gunbc/ci.dag" framing
   - **(c) Wrapper node** (e.g., `WorkflowEmission { workflow: Workflow, target: EmissionTarget }`): preserves platform/CI separation cleanly; emission policy attaches to workflow-content via wrapper
   - Canvas recommendation expected; Mgr ratifies. Operator-ratified shape per 2026-05-12: "either could work; the choice was just a toggle/modeling decision in the .dag code itself" — substance is that the choice is **modeled data**, not where it lives.
3. **Emitter dispatch semantics** — single emitter reads `emission_target` from wherever it lives, dispatches to per-target emitter module; how this composes with existing `05_emit_rust.dag` / `_python.dag` / `_go.dag` consolidation thesis (per memory `project_eliminate_emit_lang_files.md`)
4. **Per-target emission semantics**:
   - **YamlStatic**: emit `.github/workflows/ci.yml` from extended `dsl/gunbc/ci.dag`; deterministic; regenerated on workflow edit; consumer is GH Actions runner directly. This is the Slice 4 deliverable.
   - **BinaryShim**: emit compiled binary (Rust target) + thin shim YAML (≤10 lines invoking binary); binary reads diff at PR-time, computes affected-set, dispatches matrix dynamically. This is Slice 5; Slice 7 affected-set integration consumes.
   - **PythonShim**: same shim model, Python runtime substrate (Python target emit pattern reused). Future; sketch only.
   - **InlineGunbc**: gunbc-as-runtime-orchestrator (longer-term option); sketch only.
5. **Affected-set integration shape** — how `BinaryShim` emitter consumes `affected_set(Dag_before, Dag_after)` from `docs/design-affected-set-lens.md`; what the runtime decision surface looks like (matrix-spec emission, job-skip emission, etc.); refer to Slice 7 owner (Verification Mgr clever-tern-670) for cross-tier coordination
6. **Equivalence claim** — what does "same `ci.dag` emits semantically equivalent output across targets" mean precisely; what's the test claim for `workflow_emission_target_toggle_proven` gate (substrate-shape, per Director ratification §1)
7. **Carrier reuse audit** — confirm existing `dsl/extdeps/github/actions.dag` carriers (concrete `Workflow` / `Job` / `Step` per actions.dag `:21` `:110` `:147`) are sufficient for YamlStatic emitter; identify any new substrate needed (likely minimal — possibly `EmissionTarget` enum + placement choice from §2)

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent)
- `docs/design-affected-set-lens.md` — affected-set lens substrate (Slice 7 consumer)
- `docs/design-emission-model.md` — existing emission architecture (the model this extends)
- `docs/design-clean-emission-contract.md` — emission contract authority
- `dsl/extdeps/github/actions.dag` — **canonical platform carriers**: concrete `Workflow` (`:21`), `Job` (`:110`), `Step` (`:147`), `WorkflowSecret` (`:102`); LANDED per PR #2160
- `dsl/gunbc/ci.dag` — **existing CI-intent declarations** (gate-centric: `CIGate` / `CIPipeline` / `GateSource`); landed PR #2371
- `dsl/extdeps/cron_schedule_model.dag` — `CronSchedule` carrier
- `.github/workflows/ci.yml` — current hand-authored CI transport shim (the dissolution target)
- memory: `project_eliminate_emit_lang_files.md` — omni-emission consolidation thesis (inherited architecture)

## Acceptance gates

1. Canvas authored at `docs/design-ci-workflow-emitter-dispatch.md` per scope §1-7 above
2. Each per-target emitter (`YamlStatic`, `BinaryShim`, `PythonShim`) has explicit emission semantics + acceptance contract
3. `EmissionTarget` placement (Workflow vs CIPipeline vs wrapper) explicitly evaluated; canvas recommends one with reasoning; Mgr-ratifiable shape per `feedback_substrate_principle_audit`
4. Cross-references to `design-affected-set-lens.md` §5 (CI integration sketch) explicit — canvas resolves the "CI integration deferred to R4 full delivery" placeholder
5. No parallel-representation debt: greps cleanly against existing `dsl/extdeps/github/actions.dag` + `dsl/gunbc/ci.dag` (canvas EXTENDS thinking, doesn't duplicate carriers)
6. Cargo / clippy / fmt clean (no code changes expected; doc-only PR)

## STOP / PING criteria

- **STOP** if canvas needs to introduce new substrate-shape carriers beyond `EmissionTarget` enum (e.g., new step-shape, new trigger-shape) — surface to Substrate Mgr warm-wolf-698 (substrate-shape questions belong to Mgr canvas per `feedback_substrate_shape_belongs_in_mgr_canvas`)
- **STOP** if the emitter-dispatch shape conflicts with `project_eliminate_emit_lang_files.md` consolidation thesis — surface; this canvas should EXTEND that thesis, not contradict
- **STOP** if affected-set integration shape requires substantive negotiation with Verification Mgr clever-tern-670 — surface for cross-tier ratification (Slice 7 owner)
- **PING** PM (deep-wolf-155) on PR-open for review-routing
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification routing (lane owner per Director ratification msg_5cbdad24)
- **PING** Verification Mgr (clever-tern-670) on PR-open if BinaryShim affected-set-integration shape is non-trivial

## Sequencing

- This canvas authoring is dispatch-ready NOW (Slice 1 substrate LANDED via PR #2160; no held prereq; no new substrate-fact introduction expected)
- Director ratification of canvas shape is the gate to dispatch Slice 4 (YamlStatic emitter implementation) + Slice 5 (BinaryShim emitter implementation)
- Canvas should reference but NOT depend on Slice 2 (timing carriers) — emitter dispatch is orthogonal to timing instrumentation
- Sibling worker cool-carp-720 (WI-2) is extending `dsl/gunbc/ci.dag` to full ci.yml coverage in parallel — coordinate if emission_target placement question crosses

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive + Director ratification msg_5cbdad24.
