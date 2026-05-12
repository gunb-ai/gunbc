# Worker brief — T-WAD FULL R3 ci.dag extension (current ci.yml coverage)

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-2; operator FULL elevation 2026-05-12; Director ratification msg_5cbdad24 2026-05-12.
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698 lane-absorbed Slices 4-5/8 per Director); this is forward-looking scaffold; final ratification + emitter integration in Slice 4-5.
**Closure predicate**: `dsl/gunbc/ci.dag` extended to cover current `.github/workflows/ci.yml` content + Mgr-ratified shape; downstream Slice 4 YamlStatic emitter consumes; downstream Slice 8 deletes hand-authored `.github/workflows/ci.yml`.

## Output

**Extend existing `dsl/gunbc/ci.dag`** to cover full current `.github/workflows/ci.yml` content. (This file ALREADY EXISTS — it is the gate-centric CI declaration; landed via PR #2371. Current shape: `CIGate` / `CIPipeline` / `GateSource` + 4 instantiated gates [lint_gate / test_gate / l1_gate / compile_gates_gate]. The extension work is to express the FULL workflow content — jobs, steps, triggers, runner selection, matrix — composing against existing platform carriers at `dsl/extdeps/github/actions.dag`.)

## Scope

Compose the current `.github/workflows/ci.yml` against existing platform carriers from `dsl/extdeps/github/actions.dag` (authority — single-authority per `feedback_audit_adjacent_authority_first`):

**Carrier hierarchy** (per `dsl/extdeps/github/actions.dag`):

```
Workflow {                              // line :21 — concrete type, NOT generic
  name: String
  on: List<WorkflowTrigger>             // triggers
  jobs: List<Job>                       // line :24 — workflow contains JOBS
  env: Map<String, String>
  permissions: WorkflowPermissions?
}

Job {                                   // line :110
  id: String
  steps: List<Step>                     // line :114 — job contains STEPS
  needs: List<String>                   // line :115 — job-id references (NOT step-level)
  runner: RunnerSpec
  strategy: MatrixStrategy?
  ...
}

Step                                    // line :147 — RunStep | UsesStep sum
  = RunStep { run, shell, env, ... }
  | UsesStep { uses: ActionRef, with, ... }
```

**Mapping ci.yml → Workflow instance**:

1. **Top-level `on:`** → `Workflow.on: List<WorkflowTrigger>` (Push / PullRequest / Schedule / WorkflowDispatch variants from `:41`)
2. **Each `job:` in ci.yml** → `Job` element in `Workflow.jobs: List<Job>` (NOT a Step — Workflow contains Jobs, Jobs contain Steps)
3. **Per-job `needs:` field** → `Job.needs: List<String>` (list of job-id references; NOT step-level dependencies)
4. **Per-job `steps:` field** → `Job.steps: List<Step>` (each `run:` → RunStep; each `uses:` → UsesStep)
5. **Per-job `strategy.matrix:`** → `Job.strategy: MatrixStrategy?` (per `:222`)
6. **Per-job `runs-on:`** → `Job.runner: RunnerSpec` (HostedRunner / SelfHosted variants from `:132`); use existing pool labels (`gunbc-quick` / `gunbc-v3` etc.)
7. **`if:` conditions** → `Job.if_condition: String?` or `Step.if_condition: String?` per appropriate level
8. **Layer 1 skip** (`changes` job) → first `Job` in pipeline; its output binding drives `if:` conditions on downstream jobs
9. **Layer 2 cluster skip** (template per PR #2721 — NOT WIRED into ci.yml at HEAD): leave as `// TODO Slice 7` marker — Slice 7 affected-set integration replaces
10. **Slow-test ratchet** (`scripts/slow-test-exemptions.txt`): represented as `Step` with `RunStep.run: "scripts/check-slow-test-ratchet.sh"` for now — leave `// TODO Slice 6` marker for Cost-dimension migration
11. **Secrets references**: use existing `WorkflowSecret { name: SecretName, scope: SecretScope }` carrier (per actions.dag `:102` — LANDED in PR #2160, **NOT held** as earlier brief incorrectly stated)
12. **Cron triggers**: use existing `CronSchedule` carrier from `dsl/extdeps/cron_schedule_model.dag` (LANDED)

**Existing `gunbc/ci.dag` already declares** `CIGate` / `CIPipeline` / `GateSource` (gate-centric CI intent). The extension work integrates the platform-shape `Workflow` instance alongside (or composing through) the existing gate-centric declaration. **Open question for canvas/Mgr** (WI-1 may inform): does the new Workflow data go into `gunbc/ci.dag` as a sibling data, into a new file `dsl/gunbc/ci_workflow.dag`, or compose `CIPipeline` → `Workflow` via a derivation lens? Leave this in `// QUESTION ratify-via-Mgr` markers.

## Coverage scope

This is a FIRST DRAFT extending the existing `dsl/gunbc/ci.dag`. Coverage acceptance:

- **MUST**: All `jobs:` in `.github/workflows/ci.yml` represented as `Job` nodes (Workflow.jobs list)
- **MUST**: `needs:` graph faithful at Job-level (each Job.needs: List<String> matches ci.yml `needs:` field for that job)
- **MUST**: Triggers captured (push / PR / schedule trio as List<WorkflowTrigger>)
- **MUST**: Per-job runner pool selection correct (`gunbc-quick` / `gunbc-v3` / generic)
- **MUST**: Trigger conditions / runner labels validate against existing carriers (no new substrate types introduced)
- **SHOULD**: Matrix strategies for jobs that use them
- **SHOULD**: Steps within each Job captured (RunStep / UsesStep)
- **NICE-TO-HAVE**: complete `with:` / `uses:` action references per step
- **DEFERRED**: Layer 2 cluster `if:` regex (TODO Slice 7); slow-test-exemptions Cost-dim migration (TODO Slice 6); affected-set wiring (TODO Slice 7); emission_target field placement (TODO ratify via WI-1 canvas)

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent)
- `docs/briefs/r3-t-wad-full-r3-emitter-dispatch-canvas-worker.md` — sibling WI-1 brief (emitter-dispatch canvas; informs emission_target placement)
- `dsl/extdeps/github/actions.dag` — **canonical platform carriers** (authority for `Workflow` / `Job` / `Step` shapes; READ THIS FIRST — line numbers `:21` `:24` `:110` `:114` `:147` cited above)
- `dsl/gunbc/ci.dag` — **existing file to extend** (gate-centric declarations from PR #2371; do NOT duplicate)
- `dsl/extdeps/cron_schedule_model.dag` — `CronSchedule` carrier (Schedule trigger consumer)
- `dsl/extdeps/github/actions.dag:96-105` — `WorkflowSecret` + `SecretScope` carriers (LANDED PR #2160)
- `.github/workflows/ci.yml` — source content to translate

## Acceptance gates

1. `dsl/gunbc/ci.dag` extended (NOT replaced); valid `.dag` syntax per existing v3 parser
2. All ci.yml `jobs:` represented as Job nodes in a `Workflow` instance — count matches
3. `Job.needs` faithfully captures each job's `needs:` field as job-id references
4. Triggers captured (push / PR / schedule trio)
5. Per-job runner selection correct
6. No new substrate types introduced (extends only — substrate-shape additions belong to Substrate Mgr per `feedback_substrate_shape_belongs_in_mgr_canvas`)
7. TODO markers explicit for: Layer 2 cluster `if:` regex (Slice 7); slow-test-exemptions (Slice 6); affected-set wiring (Slice 7); emission_target placement (ratify via WI-1 canvas)
8. `cargo test --workspace` green (no breakage of existing tests including `t_ci_workflow_as_data_demo`)
9. `cargo clippy --all-targets -- -D warnings` clean
10. `cargo fmt --all --check` clean

## STOP / PING criteria

- **STOP** if existing carriers at `dsl/extdeps/github/actions.dag` are insufficient (e.g., need new step-shape, new trigger-shape) — surface to Substrate Mgr (substrate-shape additions belong to canvas-tier per `feedback_substrate_shape_belongs_in_mgr_canvas`)
- **STOP** if extending `gunbc/ci.dag` requires structural reframing of existing `CIGate` / `CIPipeline` declarations (e.g., they should compose under `Workflow` rather than parallel-represent) — surface to Mgr; this is a Mgr-canvas-tier decision
- **STOP** if `emission_target` field placement is forced before WI-1 canvas ratifies — leave `// QUESTION` markers, sibling worker (still-heron-763 / WI-1) is authoring the canvas in parallel
- **PING** PM (deep-wolf-155) on PR-open for review-routing
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification + Slice 4 YamlStatic emitter dispatch routing
- **COORDINATE** with sibling still-heron-763 (WI-1 emitter-dispatch canvas) if emission_target shape questions cross

## Sequencing

- Dispatch-ready NOW (existing carriers + existing ci.dag suffice for first-draft extension)
- Slice 1 substrate LANDED (PR #2160) — `WorkflowSecret` + `CronSchedule` available; no held substrate
- Slice 3 demo LANDED (PR #2371) — `t_ci_workflow_as_data_demo.dag` exists; this WI extends scope from demo→full
- Final shape ratified during Slice 4 YamlStatic emitter implementation (Substrate Mgr dispatch)
- Slice 4 YamlStatic emitter consumes this extended `ci.dag` and emits `.github/workflows/ci.yml`-equivalent; equivalence is the Slice 4 acceptance gate

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive + Director ratification msg_5cbdad24; codex BLOCKING #9970 fixed 2026-05-12 (carrier hierarchy: Workflow > Job > Step, not flat job→Step).
