# [SKETCH] Provider-agnostic commit workflow — unify two existing emitters over one gate roster

> **Status: design sketch (checkpoint 0)** — model frame for parent review *before* further substrate migration. DESIGN §6: priced in displaced cost, not elegance. **#5924 frozen** — `.githooks/pre-push` / `githooks_pre_push_emit.dag` / `local_tidy_spec.dag` are not edited until this sketch is approved and handler re-expression is witnessed byte-identical.

## 0. Grounded today — two emitters, one gate vocabulary

This is **not speculative**. Gunbc already has two registered `GeneratedArtifact` rows whose committed bytes are drift-gated:

| artifact | consumer (`RepoConsumer`) | emit handler | gate authority imported today |
| --- | --- | --- | --- |
| `CiYamlArtifact` → `.github/workflows/ci.yml` | `GithubActionsWorkflow` | `gunbc.ci_yaml_emit` ← `gunbc.ci_workflow` | `gunbc.ci_spec` (`gunbc_ci_floor_gates`, `gunbc_ci_rust_job_gates`, `CiSpec`) |
| `GithooksPrePushArtifact` → `.githooks/pre-push` | `GitProtocol` (`CommitRequired { consumer: GitProtocol }`) | `gunbc.githooks_pre_push_emit` | `gunbc.ci_spec.Gate` (via `GeneratedArtifactDriftGate`) + `local_tidy_spec` freshness slice (#5924) |

**Displaced cost:** two parallel emission paths maintain the same commit-time gate facts twice — `ci_workflow` hardwires the CI job graph while `githooks_pre_push_emit` hardwires a overlapping bash slice. Adding a gate today means touching both handlers *and* hoping `local_tidy_checks` stays aligned with `gunbc_ci_*_gates`. The project unifies **two existing emitters**, not a hypothetical Nth provider.

## 1. §3 spine — three facts, one workflow authority

| fact | owner (layer) | lives in sketch model as |
| --- | --- | --- |
| **(a) Interface shape** — what runs at commit-time: gate identity, witness entry/fn, exit semantics (`ProcessExit` pass/fail), freshness scoping (path globs) | **workflow** (`gunbc.commit_workflow` — new) | `CommitCheckRow { kind, exit_shape, freshness }` reusing `gunbc.ci_spec.Gate` + witness channel (no parallel gate enum) |
| **(b) Transport** — how the check is invoked on a provider surface | **realization handler** (peripheral) | `CommitWorkflowHandler` arms: `CiYamlHandler` (`ci_yaml_emit`), `GitPrePushScriptHandler` (`githooks_pre_push_emit`); a 3rd provider = **one new handler row**, not a workflow fork |
| **(c) Business policy** — which checks run on which provider, auto-stage-on-drift, HEAD-branch fmt auto-fix, diff merge-base for CI | **workflow** | `CommitWorkflowEnrollment { row, providers: List<RepoConsumer> }` + policy fields (`auto_stage_on_drift`, `ci_diff_policy`, …) |

**Reuse, don't re-coin:** `gunbc.ci_spec` keeps `Gate`, `CiSpec`, `DiffPolicy` as the gate vocabulary; `gunbc.local_tidy_spec` keeps path-glob helpers as **freshness policy** projections, not a second roster authority; `gunbc.generated_artifact` keeps `RepoConsumer` / `CommitPolicy` as the provider registry.

## 2. Model sketch — `gunbc.commit_workflow` (proposed module)

Single authority the two handlers project from:

- `CommitWorkflowSpec` — the workflow fact bundle: `checks: List<CommitCheckRow>`, `ci_policy: CiSpec` (existing type), `pre_push_policy: PrePushPolicy` (freshness + auto-fix flags extracted from #5924 emit, not the bash).
- `CommitCheckRow` — one roster line: `kind: CommitCheckKind` = `SpecGate { gate: Gate }` | `WitnessClaim { entry, fns }` | `CargoFmtSlice` (honest fork: CI runs fmt inside `RustMonolithGate`; pre-push runs fmt-only with auto-fix policy).
- `CommitCheckEnrollment` — policy row: `check: CommitCheckRow` + `providers: List<RepoConsumer>` (`GithubActionsWorkflow` | `GitProtocol`). **No speculative providers** — only the two consumers registered in `generated_artifact.dag` today; adding e.g. Buildkite is `providers += Buildkite` + one handler arm.

**Projections (legacy compatibility, fail-closed):**

- `project_ci_floor_gates(spec)` → `gunbc_ci_floor_gates` (GithubActionsWorkflow, floor job)
- `project_ci_rust_job_gates(spec)` → `gunbc_ci_rust_job_gates`
- `project_local_tidy_checks(spec)` → `local_tidy_checks` (GitProtocol only; witness→gate→fmt order preserved)
- `project_freshness_globs(spec, row)` → delegates to existing `local_tidy_trigger_globs` helpers (#5924, untouched)

## 3. Handler re-expression — same bytes, one workflow feed

Both handlers become **thin realization folds** over `CommitWorkflowSpec` + `RepoConsumer`:

1. **`ci_yaml_emit`** — unchanged public surface (`expected_ci_yml()`). Internals: `ci_workflow` builds `Workflow` from `project_ci_jobs(commit_workflow_spec, GithubActionsWorkflow)` instead of importing gate lists directly from `ci_spec` data rows. Checkpoint: `ci_yml_drifted(committed) == false` (existing `GeneratedArtifactDriftGate` / `ci_yaml_gate`).
2. **`githooks_pre_push_emit`** — unchanged public surface (`expected_githooks_pre_push_sh()`). Internals: bash emission walks `project_local_tidy_checks` + `project_freshness_globs` for `GitProtocol`. **#5924 hook bytes frozen** until projection wiring is witnessed; then swap authority import, not hand-edit bash.

Dispatch registry (extends `generated_artifact.dag`, no new artifact types):

| `RepoConsumer` | handler module | artifact |
| --- | --- | --- |
| `GithubActionsWorkflow` | `ci_yaml_emit` / `ci_workflow` | `CiYamlArtifact` |
| `GitProtocol` | `githooks_pre_push_emit` | `GithooksPrePushArtifact` |

### 3.1 What we are NOT doing (purity-trap guard)

- No speculative provider handlers (GitLab/Jenkins/…) — a 3rd provider is one `RepoConsumer` row + one handler arm.
- No new parallel gate enum — `Gate` from `ci_spec` stays the vocabulary.
- No pre-push rewrite in the same PR as the authority lift — #5924 emit stays byte-frozen until handler re-expression is proven.

## 4. Checkpoint — witnessed before any handler rewire

- **Roster lift:** `project_*` lists match `gunbc_ci_floor_gates`, `gunbc_ci_rust_job_gates`, `local_tidy_checks` (ordered).
- **Byte identity:** `expected_ci_yml()` and `expected_githooks_pre_push_sh()` unchanged across handler rewire PR (existing drift gates stay green).
- **Provider registry:** enrollments use only `GithubActionsWorkflow` + `GitProtocol` from `generated_artifact.dag`.

## 5. Sequencing (sketch → implement)

1. **Checkpoint 0 (this sketch)** — parent review; no #5924 edits.
2. **Checkpoint 1** — land `gunbc.commit_workflow` (`dsl/gunbc/commit_workflow.dag`) model + projection witnesses only (additive; `ci_spec`/`local_tidy_spec` still own data rows). Projections carry `commit_workflow_projection_scaffold` (`Scaffold { dissolves_to: SingleAuthority }`) — parallel-rep until checkpoint 2 inverts authority. **Do not merge checkpoint 1 as authority dissolution.**
3. **Checkpoint 2 — HOLD** (operator scope pending): `ci_spec` gate lists → projections. Load-bearing; do not touch until relay.
4. **Checkpoint 3 — HOLD** (operator scope pending): handler re-expression; mandatory byte-identity on `expected_ci_yml()` + `expected_githooks_pre_push_sh()`; #5924 hook bytes frozen (swap authority import only).

## Dissolution trigger (DESIGN §6)

Delete when `gunbc.commit_workflow` is the sole roster authority, both handlers emit from it byte-identically, and this sketch is superseded by the landed model.
