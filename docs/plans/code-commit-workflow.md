# Provider-agnostic code-commit workflow — one gate roster, CI + pre-push as realization handlers

> DESIGN refs: §2 (one roster, N realization handlers), §3 (interface shape vs transport vs workflow policy — `Gate`/`LocalTidyCheck` are channel shapes; `ci.yml` and `.githooks/pre-push` are transports), §5 (witnessed lift — the unified roster must project byte-for-byte to the existing authorities before any rewire), §6 (model-before-implement; this doc is the authority frame). **#5924 is frozen** — `local_tidy_spec.dag`, `githooks_pre_push_emit.dag`, `ci_spec.dag`, `ci_gates.dag`, and `floor_effect_gate_witness.dag` are not edited in P1; the lift is additive and witnessed.

## 1. Problem — two rosters for one commit workflow

Gunbc's commit-time checks are modeled twice today:

- **CI floor** — `gunbc.ci_spec.Gate` roster (`gunbc_ci_floor_gates`, `gunbc_ci_rust_job_gates`) realized by GitHub Actions via `gunbc.ci_workflow` → `ci_yaml_emit` → `.github/workflows/ci.yml`.
- **Local pre-push** — `gunbc.local_tidy_spec.LocalTidyCheck` roster (`local_tidy_checks`) realized by git via `gunbc.githooks_pre_push_emit` → `.githooks/pre-push` (#5924).

The overlap is real (`GeneratedArtifactDriftGate` appears on both surfaces; `RustMonolithGate` subsumes `cargo fmt` on CI while pre-push carries a dedicated `LocalTidyCargoFmtCheck` slice; doc-reachability witnesses are pre-push-only). Two rosters = §3 nicknaming risk: adding a check means editing two authorities and hoping they stay aligned.

## 2. Authority — `gunbc.code_commit_workflow`

One workflow model lifts both rosters into `commit_gate_roster: List<CommitCheckEnrollment>` where each row names:

1. **`check: CommitCheckKind`** — reuses existing channel shapes (`CommitSpecGate { gate: Gate }`, `CommitWitnessClaim { entry, check_fns }`, `CommitCargoFmtCheck`) without minting parallel gate enums.
2. **`surfaces: List<CommitWorkflowSurface>`** — provider-agnostic enrollment (`GithubActionsCiJob`, `GithubActionsRustTestsJob`, `GitPrePushHook`).

Projection functions derive the legacy authorities:

- `project_ci_floor_gates(roster)` → `gunbc_ci_floor_gates`
- `project_ci_rust_job_gates(roster)` → `gunbc_ci_rust_job_gates`
- `project_local_tidy_checks(roster)` → `local_tidy_checks`

P1 lands the roster + projections + witnesses (`code_commit_workflow_witness_test.dag`). **No rewire** of `ci_spec` or `local_tidy_spec` until projections are green — then P2 makes those modules thin projections of `commit_gate_roster` (dissolving the dual-authority fork).

## 3. Realization handlers — transport, not policy

Per §3, the *dispatch* that selects a realization is itself realization. `commit_workflow_realization_bindings` records which handler materializes each surface:

| surface | handler | committed artifact |
| --- | --- | --- |
| `GithubActionsCiJob` + `GithubActionsRustTestsJob` | `CiYamlHandler` → `gunbc.ci_workflow` / `ci_yaml_emit` | `.github/workflows/ci.yml` |
| `GitPrePushHook` | `GitPrePushScriptHandler` → `gunbc.githooks_pre_push_emit` | `.githooks/pre-push` |

Both artifacts are already drift-gated under `gunbc.generated_artifact` (`CiYamlArtifact`, `GithooksPrePushArtifact`). P1 documents the binding; P3 rewires emit modules to walk `commit_gate_roster` per surface instead of importing parallel rosters (the #5924 emit logic becomes a handler arm, not the authority).

### 3.1 Honest channel fork (not collapsed in P1)

`RustMonolithGate` (fmt+clippy+nextest) and `CommitCargoFmtCheck` (fmt-only, auto-fix on HEAD-branch push) are **different channel shapes** for overlapping policy. The unified roster keeps both rows on their respective surfaces rather than pretending one Gate subsumes the other — collapsing them is a P4 policy decision, not a modeling shortcut.

## 4. Sequencing

1. **P1 (this slice)** — `code_commit_workflow.dag` roster + projections + witnesses; design plan; **zero edits** to #5924 files.
2. **P2** — `ci_spec.dag` gate lists become `project_ci_*_gates(commit_gate_roster)`; `local_tidy_spec.dag` becomes `project_local_tidy_checks(commit_gate_roster)` + trigger-glob helpers stay (path policy is workflow-layer, not gate identity).
3. **P3** — `ci_workflow.dag` and `githooks_pre_push_emit.dag` consume per-surface projections from the roster (handler emit, not roster authority).
4. **P4 (optional)** — unify `CommitWitnessClaim` channel with floor witness discovery (the #5924 parked follow-on: one floor-check concept for gate-dispatch vs discovered-witness).

## 5. Witness contract

- `roster_lifts_ci_floor_gates` — ordered gate list matches `gunbc_ci_floor_gates`.
- `roster_lifts_ci_rust_job_gates` — matches `gunbc_ci_rust_job_gates`.
- `roster_lifts_local_tidy_checks` — matches `local_tidy_checks` (#5924 roster, untouched).
- Realization bindings resolve to `artifact_path` for `CiYamlArtifact` and `GithooksPrePushArtifact`.

## Dissolution trigger (DESIGN §6)

Delete when `commit_gate_roster` is the sole roster authority (P2 rewired `ci_spec` + `local_tidy_spec`), both realization handlers emit from per-surface projections (P3), and the lift witnesses are redundant because the legacy projection functions are gone.
