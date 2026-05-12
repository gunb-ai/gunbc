# Worker brief — T-WAD FULL R3 emitter-dispatch architecture canvas

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-1; operator FULL elevation 2026-05-12; Director ratification msg_5cbdad24 + (c-refined) substrate-shape ratification msg_237bde05 + msg_f9fd669e + msg_2a68a4b5 2026-05-12.
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698); this design canvas; final Mgr-tier ratification routes through warm-wolf-698, with Director co-sign at #828 if cross-tier confirmation needed.
**Closure predicate**: design canvas authored at `docs/design-ci-workflow-emitter-dispatch.md` + Mgr-ratified per-target emission semantics; downstream Slice 4-5 emitter implementation work consumes this canvas.

## Substrate-shape ratification anchor

**The (c-refined) substrate-shape is LOCKED** (post-2026-05-12 cascade):

- `WorkflowRuntime` is an invocation-time parameter to `project_github_actions: (CIWorkflowDag, WorkflowRuntime) -> Workflow`, **NOT a field on any carrier**.
- `WorkflowRuntime` lives in gunbc namespace (`dsl/gunbc/ci_emission.dag` — NEW file landed by WI-2).
- `dsl/extdeps/github/actions.dag::Workflow` stays platform-faithful and UNMODIFIED.
- The earlier option (a) "field on Workflow" was **RETRACTED** at Director msg_b4151f45 + codex BLOCKING #9970 on PR #2749 (INVARIANTS P1 violation — CI logic on platform carrier).
- Option (b) "field on CIPipeline" was **SUPERSEDED** — `CIPipeline { name, gates: List<CIGate> }` is flat without edge structure; cannot serve as projection input per warm-wolf-698 msg_27d99080. Replaced by `CIWorkflowDag` (PR #2736) as input domain.
- Option (c) "wrapper node" was **SUPERSEDED** by (c-refined) projection function — parameter-as-data preserves single-substrate-fact + cost-of-change ≤ 1.

**This brief does NOT reopen the placement question**. The canvas evaluates per-target emission semantics on top of the ratified substrate-shape, not the substrate-shape itself.

## Output

`docs/design-ci-workflow-emitter-dispatch.md` — new design canvas matching the shape of `docs/design-affected-set-lens.md` (single-authority, locked-design). Canvas should consume the ratified (c-refined) shape as given, not re-derive it.

## Scope

Design the emitter-dispatch architecture for T-Workflow-As-Data FULL R3-close on top of the locked (c-refined) substrate:

1. **Per-target emission semantics** (the substantive canvas surface):
   - **YamlStatic** (Slice 4 deliverable): body of `project_github_actions(ci_workflow_dag, YamlStatic)` walks `CIWorkflowDag` (PR #2736) and emits a `Workflow` value whose deterministic YAML encoding is **semantically equivalent** to current `.github/workflows/ci.yml` (same triggers / jobs / steps / runners / conditions / permissions / matrix as consumed by GitHub Actions); one-time migration rewrites committed file to canonical encoding (non-semantic facts — comments, whitespace, key ordering — discarded per P1: `Workflow` is single authority for CI semantics; load-bearing comments must migrate into substrate as modeled facts, not into parallel byte-authority); regression-guard gate compares on-disk artifact byte-wise to fresh projection output (internal byte-identity: projection-output ↔ committed-artifact, NOT external: substrate ↔ legacy hand-authored content); regenerated on workflow edit; consumer is GH Actions runner directly.
   - **BinaryShim** (Slice 5): body of `project_github_actions(ci_workflow_dag, BinaryShim)` emits a Workflow value pointing to a compiled binary (Rust target) entry-point + thin shim YAML (≤10 lines invoking binary); binary reads diff at PR-time, computes affected-set, dispatches matrix dynamically. Slice 7 affected-set integration consumes.
   - **PythonShim** (DESIGN-ONLY per briansrls BLOCKING on PR #2744 2026-05-12T10:46:59Z; held out of initial enum until real consumer exists): same shim model, Python runtime substrate (Python target emit pattern reused); sketch only — NO enum variant or emitter arm lands until a real runtime consumer exists (symmetric treatment with InlineGunbc per INVARIANTS P5 / Pure Bootstrap).
   - **InlineGunbc** (DESIGN-ONLY per codex BLOCKING #9970 + PR #2746 §5.4; held out of initial enum until real consumer exists): gunbc-as-runtime-orchestrator longer-term option; sketch only — NO enum variant or emitter arm lands until a real runtime consumer exists.

2. **Emitter dispatch composition** — how per-arm projection bodies compose with existing `05_emit_rust.dag` / `_python.dag` / `_go.dag` consolidation thesis (per memory `project_eliminate_emit_lang_files.md`); how the projection function's pinned invocation (`gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)`) flows into the existing emission pipeline.

3. **Affected-set integration shape** — how `BinaryShim` arm consumes `affected_set(Dag_before, Dag_after)` from `docs/design-affected-set-lens.md`; what the runtime decision surface looks like (matrix-spec emission, job-skip emission, etc.); cross-tier coordination with Slice 7 owner (Verification Mgr clever-tern-670).

4. **Equivalence claim** — what does "same `CIWorkflowDag` emits semantically equivalent output across workflow runtimes" mean precisely; test claim shape for `workflow_runtime_open_enum_landed` + `project_github_actions_landed` gates (per scope doc §1).

5. **GH Actions expression-substrate consumption** — per Director ratification of PR #2751 (warm-wolf-698 expression-substrate canvas): canvas should reference the ratified `Expression = OpaqueString(String)` 🟡 YELLOW shape + 5-site uniform migration (RunnerSpec / Job.if_condition / ConcurrencySpec.group / Step.with[k] / Step.env[k]); YamlStatic emission = "emit Expression::OpaqueString variant verbatim to YAML"; BinaryShim/PythonShim = "evaluate Expression at runtime" (opaque-passthrough until dissolution fires).

6. **Carrier reuse audit** — confirm existing `dsl/extdeps/github/actions.dag` carriers (concrete `Workflow` `:21` / `Job` `:110` / `Step` `:147`) + ratified Expression carrier (PR #2751) are sufficient for YamlStatic emitter. **5 carrier gaps total** identified per WI-2 brief Propagated Substrate-Fidelity Concerns section + warm-wolf-698 inventory: **4 small extdeps-fidelity substrate-prereq PRs** (`Workflow.concurrency` / `PullRequestActivity.ReadyForReview` / `Push.paths` Optional / `WorkflowPermissions.*` Optional) PLUS **1 substrate-shape canvas + Expression carrier** for `RunnerSpec` expression-syntax (PR #2751 canvas RATIFIED at msg_168005e1; Expression carrier introduction is a separate substrate-prereq PR authored by warm-wolf-698 post-#2751-merge). The 5th gap is canvas-tier, not Slice-4-extdeps-fidelity.

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent); §2 has the LOCKED (c-refined) substrate-shape statement
- `docs/briefs/r3-t-wad-full-r3-cidag-scaffold-worker.md` — sibling WI-2 brief (declares projection-function substrate in NEW file `dsl/gunbc/ci_emission.dag` — NOT extension of `dsl/gunbc/ci.dag`); this canvas plugs into the projection-function-shape that WI-2 lands
- `docs/design-affected-set-lens.md` — affected-set lens substrate (Slice 7 consumer)
- `docs/design-emission-model.md` — existing emission architecture (the model this canvas extends)
- `docs/design-clean-emission-contract.md` — emission contract authority
- PR #2749 §7 (warm-wolf-698 substrate-shape comparison canvas) — (c-refined) ratification anchor
- PR #2736 (neat-badger-30) — `CIWorkflowDag` carrier introduction; projection input domain
- PR #2751 (warm-wolf-698 GH Actions expression substrate canvas) — `Expression` 🟡 YELLOW substrate; Director-ratified per msg_168005e1
- `dsl/extdeps/github/actions.dag` — **canonical platform carriers**: concrete `Workflow` (`:21`), `Job` (`:110`), `Step` (`:147`), `WorkflowSecret` (`:102`)
- `dsl/gunbc/ci.dag` — **existing gate-centric CI substrate** (PR #2371); canvas does NOT extend this file (per (c-refined) shape); WI-2 lands the projection-function substrate in NEW file `dsl/gunbc/ci_emission.dag`
- memory: `project_eliminate_emit_lang_files.md` — omni-emission consolidation thesis

## Acceptance gates

1. Canvas authored at `docs/design-ci-workflow-emitter-dispatch.md` per scope §1-6 above
2. Each per-target emitter arm in the initial enum (`YamlStatic`, `BinaryShim`) has explicit emission semantics + acceptance contract
3. `PythonShim` AND `InlineGunbc` documented as DESIGN-ONLY future targets with explicit "no enum variant / emitter arm lands until real runtime consumer exists" exclusion rule (symmetric treatment per INVARIANTS P5)
4. Substrate-shape placement is NOT re-evaluated — (c-refined) shape is consumed as given; canvas explicitly cites PR #2749 §7 as placement authority
5. Cross-references to `design-affected-set-lens.md` §5 (CI integration sketch) explicit — canvas resolves the "CI integration deferred" placeholder
6. Cross-references to PR #2751 (Expression substrate) for expression-site emission contracts
7. No parallel-representation debt: canvas does NOT extend `dsl/gunbc/ci.dag` or modify `dsl/extdeps/github/actions.dag` substrate-shape outside the (c-refined)-ratified surface
8. Cargo / clippy / fmt clean (no code changes expected; doc-only PR)

## STOP / PING criteria

- **STOP** if canvas reveals the (c-refined) shape doesn't actually compose with the existing emission pipeline — surface to warm-wolf-698 + Director for shape re-evaluation (would constitute substrate-shape re-ratification request)
- **STOP** if per-target emission body shape requires substantive new substrate carriers beyond the 5 already-tracked gaps (4 small substrate-prereq PRs + 1 Expression carrier via PR #2751 canvas) — surface to Substrate Mgr warm-wolf-698
- **STOP** if the emitter-dispatch shape conflicts with `project_eliminate_emit_lang_files.md` consolidation thesis — surface; this canvas should EXTEND that thesis, not contradict
- **STOP** if affected-set integration shape requires substantive negotiation with Verification Mgr clever-tern-670 — surface for cross-tier ratification (Slice 7 owner)
- **PING** PM (deep-wolf-155) on PR-open for review-routing
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification routing (lane owner)
- **PING** Verification Mgr (clever-tern-670) on PR-open if BinaryShim affected-set-integration shape is non-trivial
- **COORDINATE** with sibling WI-2 worker cool-carp-720 — WI-2 lands the projection-function-substrate this canvas plugs into

## Sequencing

- Canvas authoring is dispatch-ready NOW (Slice 1 substrate LANDED via PR #2160; (c-refined) shape ratified at PR #2749; Expression substrate ratified at PR #2751 — all upstream)
- Canvas merge is the gate to dispatch Slice 4 (YamlStatic emitter implementation) + Slice 5 (BinaryShim emitter implementation) per-arm-body briefs
- Canvas should reference but NOT depend on Slice 2 (timing carriers) — emitter dispatch is orthogonal to timing instrumentation

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive + Director ratification msg_5cbdad24; revised 2026-05-12 per codex BLOCKING on PR #2744 (2026-05-12T07:49:06Z) — substrate-shape questions LOCKED to (c-refined) per ratification cascade; no longer "OPEN QUESTION"; WI-2 lands NEW file `dsl/gunbc/ci_emission.dag` not extension of `dsl/gunbc/ci.dag`.
