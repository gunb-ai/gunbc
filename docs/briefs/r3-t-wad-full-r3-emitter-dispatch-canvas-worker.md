# Worker brief — T-WAD FULL R3 emitter-dispatch architecture canvas

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-1; operator FULL elevation 2026-05-12 (toggle design ratification).
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698) — this is forward-looking canvas; final ratification by Mgr + Director.
**Closure predicate**: design canvas authored + Director-ratified shape; downstream Slice 4-5 emitter implementation work consumes this canvas.

## Output

`docs/design-ci-workflow-emitter-dispatch.md` — new design canvas, ~400-800 lines target, matching the shape of `docs/design-affected-set-lens.md` (single-authority, locked-design).

## Scope

Design the emitter-dispatch architecture for `T-Workflow-As-Data` FULL R3-close:

1. **`EmissionTarget` modeled field shape** — open enum `EmissionTarget = YamlStatic | BinaryShim | PythonShim | InlineGunbc | ...`; where it lives in the carrier hierarchy (on `Workflow<>` carrier? sibling node?); default value if absent (`YamlStatic` recommended)
2. **Emitter dispatch semantics** — single emitter reads `emission_target` from `ci.dag`, dispatches to per-target emitter module; how this composes with existing `05_emit_rust.dag` / `_python.dag` / `_go.dag` consolidation thesis (per memory `project_eliminate_emit_lang_files.md`)
3. **Per-target semantics**:
   - **YamlStatic**: emit `.github/workflows/ci.yml` from `ci.dag`; deterministic; regenerated on workflow edit; consumer is GH Actions runner directly
   - **BinaryShim**: emit compiled binary (Rust target) + thin shim YAML (≤10 lines invoking binary); binary reads diff at PR-time, computes affected-set, dispatches matrix dynamically
   - **PythonShim**: same shim model, Python runtime substrate (Python target emit pattern reused)
   - **InlineGunbc**: gunbc-as-runtime-orchestrator (longer-term option; sketch only)
4. **Affected-set integration shape** — how `BinaryShim` emitter consumes `affected_set(Dag_before, Dag_after)` from `docs/design-affected-set-lens.md`; what the runtime decision surface looks like (matrix-spec emission, job-skip emission, etc.)
5. **Equivalence claim** — what does "same `ci.dag` emits semantically equivalent output across targets" mean precisely; what's the test claim
6. **Carrier dependencies** — what existing carriers at `dsl/extdeps/github/actions.dag` are reused vs new substrate needed (likely just `EmissionTarget` field addition + possibly `MatrixSpec` for dynamic dispatch)

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent)
- `docs/design-affected-set-lens.md` — affected-set lens substrate (Slice 7 consumer)
- `docs/design-emission-model.md` — existing emission architecture (the model this extends)
- `docs/design-clean-emission-contract.md` — emission contract authority
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 1 carrier baseline (`WorkflowSecret<Name>` + `Cron`)
- `dsl/extdeps/github/actions.dag` — existing workflow carriers (`Workflow`, `WorkflowTrigger`, `Step`, `MatrixStrategy`, `RunnerSpec`)
- `.github/workflows/ci.yml` — current hand-authored CI (the deletion target)
- memory: `project_eliminate_emit_lang_files.md` — omni-emission consolidation thesis

## Acceptance gates

1. Canvas authored at `docs/design-ci-workflow-emitter-dispatch.md` per scope §1-6 above
2. Each per-target emitter (`YamlStatic`, `BinaryShim`, `PythonShim`) has explicit emission semantics + acceptance contract
3. `EmissionTarget` field shape ratifiable by Mgr without further canvas iteration (Substrate-Principle-Audit shape per memory `feedback_substrate_principle_audit`)
4. Cross-references to `design-affected-set-lens.md` §5 (CI integration sketch) explicit — canvas resolves the "CI integration deferred to R4 full delivery" placeholder
5. No parallel-representation debt: greps cleanly against existing `dsl/extdeps/github/actions.dag` (canvas extends carriers, doesn't duplicate)
6. Cargo / clippy / fmt clean (no code changes expected; doc-only PR)

## STOP / PING criteria

- **STOP** if canvas needs to introduce new substrate-shape carrier beyond `EmissionTarget` field on existing `Workflow<>` — surface to Mgr (substrate-shape questions belong to Mgr canvas per `feedback_substrate_shape_belongs_in_mgr_canvas`)
- **STOP** if the emitter-dispatch shape conflicts with `project_eliminate_emit_lang_files.md` consolidation thesis — surface; this canvas should EXTEND that thesis, not contradict
- **PING** PM (deep-wolf-155) on PR-open for review-routing
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification routing
- **PING** Verification Mgr (clever-tern-670) on PR-open if BinaryShim affected-set-integration shape is non-trivial (clever-tern-670 owns affected-set lens at PR #2713)

## Sequencing

- This canvas authors are dispatch-ready NOW (independent of Slice 1 substrate hold; no new substrate-fact introduction)
- Director ratification of canvas shape is the gate to dispatch Slice 4 (YamlStatic emitter implementation) + Slice 5 (BinaryShim emitter implementation)
- Canvas should reference but NOT depend on Slice 2 (timing carriers) — emitter dispatch is orthogonal to timing instrumentation

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive.
