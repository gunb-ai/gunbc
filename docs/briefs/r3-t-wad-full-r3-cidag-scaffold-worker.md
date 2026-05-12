# Worker brief — T-WAD FULL R3 ci_emission.dag substrate scaffold

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-2; operator FULL elevation 2026-05-12; Director ratification msg_5cbdad24 + (c-refined) ratification msg_237bde05 / msg_f9fd669e 2026-05-12.
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698 lane-absorbed Slices 4-5/8 per Director); this WI lands the projection-function substrate that enables Slice 4 emitter implementation.
**Closure predicate**: `dsl/gunbc/ci_emission.dag` declares `EmissionTarget` open enum + `project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow` projection function + `gunbc_ci_yml_workflow` pinned-projection data binding; downstream Slice 4 YamlStatic emitter consumes the projection; downstream Slice 8 deletes hand-authored `.github/workflows/ci.yml`.

## Substrate-shape ratification anchor

The (c-refined) shape was ratified by Director at msg_237bde05 + msg_f9fd669e and self-corrected to via PR #2749 §7 (warm-wolf-698). The earlier (a) shape (`EmissionTarget?` field on `dsl/extdeps/github/actions.dag::Workflow`) was RETRACTED at Director msg_b4151f45 + codex BLOCKING #9970 on PR #2749 for INVARIANTS P1 violation (CI logic placed on platform carrier).

**The ratified shape**: `EmissionTarget` is a parameter to a projection function in `dsl/gunbc/`, NOT a field on any carrier. The function's invocation pin produces the `Workflow` value the emitter consumes.

## Output

**Create NEW file `dsl/gunbc/ci_emission.dag`** declaring:

1. **`EmissionTarget` open enum** (sum-type with named arms; OPEN per design):

   ```
   EmissionTarget = YamlStatic
                  | BinaryShim
                  | PythonShim
                  | InlineGunbc
                  | ...           // open enum — additional targets may be added
   ```

   Each arm is a tag indicating the emission strategy. Initial set covers the 4 strategies named in PM scoping doc §3. Open-enum discipline: the projection function must total-handle the declared arms; unknown arms compile-fail (per fail-closed discipline `feedback_fail_closed_discipline`).

2. **`project_github_actions` projection function declaration**:

   ```
   project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow
   ```

   - Input domain: `(ci_workflow_dag: CIWorkflowDag, target: EmissionTarget)`
   - Output codomain: `Workflow` (the `dsl/extdeps/github/actions.dag::Workflow` platform carrier)
   - **This WI lands the DECLARATION + signature**, not the per-arm implementation. Per-arm bodies (YamlStatic projection, BinaryShim projection, etc.) are Slice 4+ work owned by Substrate Mgr.
   - Leave function body as `// TODO Slice 4-5: per-arm projection implementations` with structural total-handling skeleton (match on `target` arms; each arm body marked TODO).

3. **`gunbc_ci_yml_workflow` pinned-projection data binding**:

   ```
   gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)
   ```

   - `ci_workflow_dag` is the input — sourced from existing `dsl/gunbc/ci.dag` `CIPipeline` value or a new `CIWorkflowDag` carrier (see "CIWorkflowDag dependency sequencing" below)
   - This is the **invocation pin** that defines the canonical YAML-emission point for downstream Slice 4
   - The pinned data binding is THE Workflow value Slice 4's YamlStatic emitter walks to produce `.github/workflows/ci.yml`-equivalent output

## CIWorkflowDag dependency sequencing

The projection function takes a `CIWorkflowDag` input. Two valid sourcing paths:

**Path (a) — reuse existing `dsl/gunbc/ci.dag` CIPipeline value**: pass the gate-centric `CIPipeline` from PR #2371 as the input domain; the projection function unpacks gates + pipeline structure into Workflow shape. **No new substrate type needed**.

**Path (b) — introduce new `CIWorkflowDag` carrier**: declares a richer CI-logic shape that captures jobs/steps/needs/triggers at gunbc-substrate level (NOT platform-level), then projects into the `Workflow` platform carrier via the function. **Substrate-shape addition — belongs to Mgr canvas per `feedback_substrate_shape_belongs_in_mgr_canvas`**.

**Disposition**: Path (a) preferred for this WI — minimal substrate addition, leverages existing `CIPipeline` authority. If Path (a) cannot express ci.yml fully (e.g., needs steps/runners that `CIPipeline` doesn't carry), **STOP and PING Substrate Mgr** to ratify Path (b) carrier shape via canvas. Do NOT introduce `CIWorkflowDag` without Mgr ratification.

## Scope boundaries (DO / DON'T)

**DO**:
- Create NEW file `dsl/gunbc/ci_emission.dag` (the projection-substrate file).
- Declare `EmissionTarget` open enum with 4 named arms + open marker.
- Declare `project_github_actions` function signature with TODO-marked total-handling skeleton.
- Declare `gunbc_ci_yml_workflow` pinned-projection data binding (invocation pin with `YamlStatic`).
- Reference existing `Workflow` carrier from `dsl/extdeps/github/actions.dag` as the codomain type.
- Cite existing `CIPipeline` from `dsl/gunbc/ci.dag` as the input source if Path (a).

**DON'T**:
- Do NOT add fields to `dsl/extdeps/github/actions.dag` carriers (Workflow/Job/Step) — this is the INVARIANTS P1 violation the (c-refined) shape resolves.
- Do NOT extend `dsl/gunbc/ci.dag` with new substrate types — extension belongs to Mgr canvas tier.
- Do NOT implement per-arm projection bodies — those are Slice 4-5 work owned by Substrate Mgr after canvas ratification.
- Do NOT modify `.github/workflows/ci.yml` — Slice 8 deletes; this WI only lands the projection substrate.
- Do NOT introduce a `CIWorkflowDag` carrier without Substrate Mgr ratification (see "CIWorkflowDag dependency sequencing" above).

## Acceptance gates

1. New file `dsl/gunbc/ci_emission.dag` exists with valid `.dag` syntax per existing v3 parser.
2. `EmissionTarget` open enum declared with 4 named arms (`YamlStatic`, `BinaryShim`, `PythonShim`, `InlineGunbc`) + open-enum marker.
3. `project_github_actions: (CIWorkflowDag, EmissionTarget) -> Workflow` function signature declared with structural total-handling skeleton (match on `target`; each arm body TODO-marked).
4. `gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)` pinned-projection data binding declared.
5. NO modifications to `dsl/extdeps/github/actions.dag` (INVARIANTS P1).
6. NO new fields on existing `dsl/gunbc/ci.dag` carriers (`CIGate` / `CIPipeline` / `GateSource`).
7. NO new substrate types introduced without Mgr ratification (per `feedback_substrate_shape_belongs_in_mgr_canvas`).
8. `cargo test --workspace` green (no breakage of existing tests including `t_ci_workflow_as_data_demo`).
9. `cargo clippy --all-targets -- -D warnings` clean.
10. `cargo fmt --all --check` clean.

## STOP / PING criteria

- **STOP** if existing `CIPipeline` from `dsl/gunbc/ci.dag` cannot supply enough input data for the projection function to produce a faithful `Workflow` value — surface to Substrate Mgr; Path (b) requires canvas-tier ratification.
- **STOP** if `EmissionTarget` open-enum declaration requires substrate-shape features not yet supported by current v3 surface (e.g., open-enum vocabulary missing) — surface to Substrate Mgr.
- **STOP** if `project_github_actions` function signature requires substrate features not yet supported (e.g., function-as-projection declaration vocabulary missing) — surface to Substrate Mgr; canvas-tier decision.
- **PING** PM (deep-wolf-155) on PR-open for review-routing.
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification + Slice 4 emitter dispatch routing.
- **COORDINATE** with sibling still-heron-763 (WI-1 emitter-dispatch canvas at PR #2749) — the canvas declares the (c-refined) shape this WI implements; if canvas re-ratifies the shape, this WI must follow.

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent); §1 gate `project_github_actions_landed` is the closure-gate for this WI.
- `docs/briefs/r3-t-wad-full-r3-emitter-dispatch-canvas-worker.md` — sibling WI-1 brief (declares the (c-refined) substrate shape this WI implements).
- PR #2749 (still-heron-763 WI-1 canvas) §7 — (c-refined) shape self-correction; THIS WI implements that shape.
- `dsl/extdeps/github/actions.dag:1-12` — platform-carrier scope header ("platform constraints, not CI logic (that lives in gunbc/ci.dag)"); the discriminator that grounds the (c-refined) shape.
- `dsl/extdeps/github/actions.dag:21` — `Workflow` carrier (codomain type for `project_github_actions`).
- `dsl/gunbc/ci.dag` — existing gate-centric CI substrate (Path (a) input source).
- `feedback_audit_extdeps_header_for_logic_vs_platform_discriminator.md` — the discipline that catches (a)-shape mis-placements at canvas-authoring time.

## Sequencing

- Dispatch-ready NOW (existing `dsl/extdeps/github/actions.dag::Workflow` + existing `dsl/gunbc/ci.dag::CIPipeline` suffice for declaration scope).
- Slice 1 substrate LANDED (PR #2160) — `WorkflowSecret` + `CronSchedule` available; consumed downstream by Slice 4 emitter, not this WI.
- Slice 3 demo LANDED (PR #2371) — `t_ci_workflow_as_data_demo.dag` exists; this WI extends scope from demo-of-evaluation into projection-substrate-declaration.
- This WI lands the projection-function substrate; Slice 4 implements per-arm bodies; Slice 8 dissolves hand-authority over `.github/workflows/ci.yml` (per renamed gate `ci_yml_hand_authority_dissolved` — see scope doc §1 fix 2026-05-12).

## Propagated substrate-fidelity concerns (Slice 4-5 — Substrate Mgr canvas)

The following concerns are flagged by briansrls BLOCKING inline reviews on PR #2744 (2026-05-12T06:58:55Z) against the EARLIER brief content (when scope was "compose full ci.yml against existing carriers"). The CURRENT brief scope (declaration-only) does NOT make these claims, but the underlying substrate-fidelity concerns propagate to Slice 4-5 per-arm projection body work and must be addressed by Substrate Mgr (warm-wolf-698) canvas-tier before YamlStatic / BinaryShim / PythonShim implementations land.

1. **Concurrency carrier absence**: current `.github/workflows/ci.yml` uses top-level `concurrency:` field; `dsl/extdeps/github/actions.dag::Workflow` does NOT model `concurrency`. P1/P2 fidelity requires extension (e.g., `Workflow.concurrency: Concurrency?` with `Concurrency { group, cancel_in_progress }`).
2. **`ready_for_review` PR activity absence**: current ci.yml uses `pull_request.types: [opened, synchronize, reopened, ready_for_review]`; `PullRequestActivity` enum in actions.dag may not carry `ready_for_review` arm. Audit + extend if missing.
3. **Trigger fidelity — NO fabrication**: current ci.yml declares only `push` and `pull_request` triggers — NOT `schedule`. Slice 4 YamlStatic emitter MUST emit faithfully (push + PR only); `Schedule` trigger arm exists in the carrier but is NOT instantiated for current ci.yml. **No fabrication of triggers absent from source.**
4. **Step body + action input completeness as MUST (not SHOULD / NICE-TO-HAVE)**: Slice 4 ci.yml-equivalent regeneration requires executable step bodies (`RunStep.run`, `RunStep.shell`, `RunStep.env`) and complete action references (`UsesStep.uses: ActionRef`, `UsesStep.with: Map<String, String>`) as MUST acceptance. P1 modeling faithfulness fails if these are partial.

These concerns are NOT acceptance gates for THIS WI (declaration-only scope). They are propagated to Slice 4-5 canvas as MUST-address-before-per-arm-body-PRs.

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive + Director ratification msg_5cbdad24 + (c-refined) ratification msg_237bde05 / msg_f9fd669e; revised after codex BLOCKING #9970 on PR #2749 (INVARIANTS P1: EmissionTarget belongs in gunbc-substrate projection function, not as field on actions.dag::Workflow).
