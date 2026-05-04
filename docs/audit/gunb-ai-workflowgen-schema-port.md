# gunb.ai workflowgen schema port survey

Status: survey/catalog only. This brief does not author substrate carriers or change workflow behavior.

Source inspected: `gunb-ai/gunb.ai` at `085febfb7279215039c6ce9fd15659726d7c1e3e`, focused on `tools/ci/workflowgen/schema/`, `tools/ci/workflowgen/workflows/`, `tools/ci/workflowgen/actions/`, and `.github/workflows/`.

## Schema Inventory

`tools/ci/workflowgen/schema/workflow.go` defines a typed Go model for GitHub Actions workflow YAML and keeps rendering separate in `render.go`.

| Concept | Go schema surface | Observed shape |
| --- | --- | --- |
| Workflow | `Workflow` | `Name`, optional `Concurrency`, `On`, `Permissions`, optional `Defaults`, `Env`, ordered `[]JobEntry`. |
| Trigger set | `On` | Optional `Push`, `PullRequest`, `WorkflowCall`, `WorkflowDispatch`, `WorkflowRun`, and repeated `ScheduleTrigger`. |
| Push trigger | `PushTrigger` | `Branches`, `Tags`, `Paths`. |
| Pull request trigger | `PullRequestTrigger` | `Types`, `Branches`, `Paths`. |
| Workflow call | `WorkflowCallTrigger` | Ordered inputs, secrets, outputs for reusable workflows. |
| Workflow dispatch | `WorkflowDispatch` | Ordered manual inputs. |
| Workflow run | `WorkflowRunTrigger` | Workflow names, branches, event types, plus a non-rendered `Conclusion` note. |
| Schedule | `ScheduleTrigger` plus `Schedule` builder | Raw cron string with helpers for daily, daily-at, hourly, every-N-hours, every-N-minutes, weekly, weekdays, and raw cron. |
| Permissions | `Permissions map[PermissionScope]PermissionLevel` | Scopes: `actions`, `contents`, `pull-requests`, `packages`, `id-token`; levels: `read`, `write`. `MergePermissions` and `PermissionsFromJobs` compute workflow ceilings. |
| Defaults | `Defaults`, `RunDefaults` | Workflow-level `defaults.run.shell` and `defaults.run.working-directory`. |
| Env/default data | `EnvMap map[string]string` | String-valued env maps rendered in sorted key order. Expressions remain strings. |
| Concurrency | `Concurrency` | `group` string and `cancel-in-progress` bool; false is omitted by renderer. |
| Job | `JobEntry`, `JobID`, `Job` | Ordered job id plus job. Fields cover `name`, `runs-on`, `if`, `needs`, permissions, env, container, outputs, steps, reusable `uses`, `with`, and `secrets`. |
| Dependencies | `Job.Needs []JobID` | One need renders as scalar; multiple needs render as a sequence. |
| Reusable workflow job | `Job.Uses`, `Job.With map[string]any`, `Job.Secrets` | Used by caller jobs that invoke generated reusable workflows. |
| Runner | `RunnerType` | GitHub-hosted `ubuntu-24.04`, `ubuntu-22.04`; self-hosted profiles `bazel-rbe`, `bazel-local`, `quick-checks`, `Medium` map to label lists. |
| Condition | `Condition string` | Opaque GitHub Actions expression string. |
| Secrets mode | `SecretsMode` | `inherit` or empty. Individual workflow-call secrets are separate `WorkflowSecret` entries. |
| Container | `Container`, `Credentials` | Image, registry credentials, env, options, and host volume strings. |
| Step | `Step`, `StepID`, `ActionRef` | `name`, `id`, `if`, `uses`, string `with`, `run`, `shell`, env, working directory, and `continue-on-error`. |
| Action references | `ActionRef` constants | Common refs include checkout, cache, artifact upload/download, docker actions, app-token action, and local `./.github/actions/setup-bazel`. |
| Composite action | `CompositeAction` in `action.go` | `name`, `description`, ordered inputs/outputs, and `CompositeRuns{Using, Steps}`. |
| Matrix | Not present in schema | No `strategy` or `matrix` carrier exists in `workflow.go`; no generated workflow output used `strategy`/`matrix` in the inspected set. |

Renderer notes:
- `Render` emits generated-file headers, then builds ordered YAML nodes.
- Workflow/job/step/env map keys are stabilized where maps are used.
- Permission rendering has an explicit scope order: actions, contents, pull-requests, packages, id-token.
- `With map[string]any` is intentionally looser for reusable workflow job inputs, while step `With` is `map[string]string`.

## Generated Workflow Catalog

`tools/ci/workflowgen/workflows/registry.go` is the generated workflow registry. It emits:

| Generated workflow | Trigger shape | Job shape and dependencies | Permissions/secrets/env highlights |
| --- | --- | --- | --- |
| `bazel-ci.yml` | `workflow_call` with typed inputs and outputs. | Single containerized `bazel-tests` job on `${{ inputs.runner_type }}`. | Workflow permissions include contents/pull-requests/packages read and id-token write. Uses GHCR container credentials from `secrets.GITHUB_TOKEN`, setup-bazel, artifacts, coverage, and context payload env. |
| `coverage-triage.yml` | Daily schedule plus manual dispatch. | Reusable `coverage` job calls `bazel-ci.yml`; `backend-change` needs `coverage`. | Default permissions, `secrets: inherit` for reusable call, Google auth/setup-gcloud, artifact download, OaaS dispatch env. |
| `devcontainer-tests.yml` | `workflow_call` plus manual dispatch. | Single host `test-devcontainer` job. | Default permissions, job outputs, devcontainer build/start/verification steps. |
| `flake-triage.yml` | Daily schedule plus manual dispatch. | Containerized `flake-detection`; `backend-change` needs `flake-detection`. | Default permissions, GHCR credentials, setup-bazel, artifact upload/download, Google auth, OaaS dispatch env. |
| `gcp-deploy-dev.yml` | Weekly schedule plus manual dispatch inputs. | Single host `deploy-dev` job. | Contents read and id-token write. Uses Secret Manager auth, remote build secret loading, WIF, Docker/Artifact Registry, infra plan/apply, artifacts. |
| `gcp-deploy-test.yml` | Daily schedule plus manual dispatch. | Single host `deploy-test` job. | Same deploy pattern as dev with test tag/config. |
| `live-tests.yml` | `workflow_call` plus manual dispatch. | Single containerized `live-tests` job. | Default permissions, GHCR/public heavy container, Google WIF, AWS OIDC, setup-bazel, live integration test env. |
| `main-breaks.yml` | Schedule plus manual dispatch. | `check-failure`; `collect-context` needs it; `remediation` needs `collect-context`. | Adds `actions: read` to default permissions. Uses workflow-run inspection, setup-bazel, artifact handoff, Google auth, OaaS dispatch. |
| `presubmit.yml` | Push, pull request, manual dispatch. | Router/heal graph plus reusable `bazel-ci`, coverage variants, image smoke/publish, devcontainer, live tests. Needs connect router/heal to downstream jobs; reusable jobs use `secrets: inherit`. | Workflow permissions are computed from jobs and include contents/packages write, pull-requests read, id-token write. Heavy jobs use containers and setup-bazel. |
| `promote-heavy-image.yml` | Manual dispatch with choice/string inputs. | Single host `promote` job. | Job-level contents read, packages/id-token write. Uses docker buildx/login and GHCR promotion steps. |
| `setup-bazel-smoke.yml` | Pull request paths plus manual dispatch. | `smoke-container` and `smoke-host` independent jobs. | Contents read and id-token write. Container and host variants exercise local `setup-bazel`. |
| `stale-branch-cleanup.yml` | Daily schedule plus manual dispatch inputs. | Single host `cleanup` job. | Contents write, pull-requests read. Uses `actions/github-script@v7` with `secrets.GITHUB_TOKEN`. |
| `style-lint.yml` | `workflow_call`. | Single containerized `lint` job on self-hosted quick labels. | Workflow env `STYLE_LINT_TS`; job permissions contents write/packages read/id-token write; setup-bazel, Node, Python, renderdag cache/build, auto-format. |

Generated composite action:

| Generated action | Source | Shape |
| --- | --- | --- |
| `.github/actions/setup-bazel/action.yml` | `tools/ci/workflowgen/actions/setup_bazel.go` via `schema.CompositeAction` | Composite action with typed inputs/outputs and ordered steps. Recurring step families include disk cleanup, workspace safety, setup-bazel binary build/provisioning, Bazelisk cache, Secret Manager binary cache, auth timestamping, and exported cache/config outputs. |

Other workflow files in `.github/workflows/`:
- `ci-bootstrap.yml` is explicitly manual and non-generated because workflowgen corruption could otherwise break the generated recovery path.
- `extdeps-triage.yml` lacks the generated header and is not listed in `workflows.Registry()`.

Recurring emitted components:
- Trigger families: `workflow_call`, `workflow_dispatch`, `schedule`, `push`, `pull_request`, and generated `workflow_run` support in schema. Generated outputs heavily use reusable workflow calls and schedules.
- Job families: single host jobs, containerized Bazel jobs, reusable workflow caller jobs, backend/OaaS dispatch jobs after artifact handoff, deployment jobs, lint/auto-format jobs, and smoke-test pairs.
- Step families: checkout, setup-bazel local action, cache/cache-save, artifact upload/download, Google auth/setup-gcloud, AWS OIDC, Docker buildx/login/build-push, disk cleanup/storage reports, Bazel test/coverage/build, payload/artifact preparation, and guarded auto-commit/push.
- Secrets/env usage: `secrets.GITHUB_TOKEN`, `secrets: inherit`, Google WIF vars/service accounts, Secret Manager-loaded values, GHCR credentials, OaaS dispatch env, Bazel cache env, and workflow/job env maps. Expressions are currently stored as strings.
- Dependencies: explicit `needs` is common for router/downstream CI, artifact handoff, backend remediation, and publish-after-build. No generated matrix usage was observed.

## Port Mapping

Candidate `.dag` names below are survey labels only, not authored carriers.

| gunb.ai Go schema concept | Candidate gunbc `.dag` carrier/type | Portable now? | Notes/open questions |
| --- | --- | --- | --- |
| `Workflow` | `GithubWorkflow` record | Portable now | Straight record with ordered jobs; decide whether renderer order is structural or producer-owned. |
| `JobEntry` / `JobID` | `WorkflowJobEntry`, `WorkflowJobId` | Portable now | Job ids should likely be nominal/newtyped strings. |
| `On` | `WorkflowTriggerSet` sum/record | Needs substrate decision | Optional many-trigger record is portable; exact trigger union and unsupported GitHub events need policy. |
| `PushTrigger`, `PullRequestTrigger` | `PushTrigger`, `PullRequestTrigger` | Portable now | Lists of branches/tags/paths/types are direct. |
| `WorkflowCallTrigger` | `ReusableWorkflowInterface` | Needs substrate decision | Inputs/secrets/outputs are portable, but expression-typed output values need a decision. |
| `WorkflowDispatch` | `ManualDispatch` | Portable now | Ordered inputs map cleanly. |
| `WorkflowRunTrigger` | `WorkflowRunTrigger` | Needs substrate decision | `Conclusion` is intentionally non-rendered in Go; substrate should separate modeled-but-non-emitted fields from rendered schema. |
| `ScheduleTrigger` / `Schedule` builder | `CronSchedule`, plus producer helpers | Needs substrate decision | Raw cron is portable; helper constructors could be producer functions rather than carriers. |
| `Permissions` | `PermissionSet`, `PermissionScope`, `PermissionLevel` | Portable now | Finite scopes/levels are explicit in Go. Merge/ceiling logic is behavior and should remain producer or rule layer. |
| `Defaults` / `RunDefaults` | `WorkflowDefaults`, `RunDefaults` | Portable now | Direct optional record. |
| `EnvMap` | `EnvMap` / `EnvBinding` | Needs substrate decision | String map is portable, but values often contain GitHub expressions and secret references. |
| `Concurrency` | `WorkflowConcurrency` | Portable now | Group string and bool; decide if expression-valued group is just string or typed expression. |
| `Job` | `WorkflowJob` sum or record | Needs substrate decision | Go uses one record for executable and reusable jobs. `.dag` may want a sum separating `runs-on/steps` from `uses/with/secrets`. |
| `RunnerType` | `RunnerRef` / `RunnerProfile` | Needs substrate decision | Known runner profiles are portable; self-hosted label expansion may belong to a producer, not carrier data. |
| `Condition` | `GithubExpr` or opaque `String` | Needs substrate decision | Current Go treats conditions as strings. A port should decide whether GitHub expression syntax is opaque or parsed. |
| `SecretsMode` / `WorkflowSecret` | `WorkflowSecretMode`, `WorkflowSecretDecl` | Needs substrate decision | `inherit` is simple; actual secret authority/redaction remains outside this survey. |
| `Container` / `Credentials` | `JobContainer`, `ContainerCredentials` | Needs substrate decision | Shape is direct, but credential expressions and volume policy need substrate boundaries. |
| `Step` | `WorkflowStep` sum: `RunStep` / `UsesStep` | Needs substrate decision | Go permits both `uses` and `run` fields in one record; substrate may want exclusive variants. |
| `ActionRef` | `ActionRef` | Portable now | Constants can port as data, but version pinning/local refs need policy. |
| `CompositeAction` | `GithubCompositeAction` | Portable now | Mirrors generated `action.yml`; shares `Step` type. |
| `InputType` / action inputs | `WorkflowInputType`, `WorkflowInput`, `ActionInput` | Portable now | Finite input types and ordered entries map cleanly. |
| `With map[string]any` | `WorkflowInputValue` | Needs substrate decision | `any` supports bool/string/number for reusable workflow `with`; needs typed scalar/expression decision. |
| Matrix | `WorkflowMatrix` | Needs substrate decision | Not present in current Go schema or generated outputs; include only if future workflow port scope needs it. |

## Portable Now vs Needs Decision

Portable now:
- Ordered workflow/job/action entry structure.
- Finite permission scopes and read/write levels.
- Push/pull request branch/path/tag/type filters.
- Workflow and action input declarations, including string/boolean/number/choice input types.
- Workflow defaults, concurrency record shape, action refs as data, and composite action shape.
- Job ids, step ids, output names, and other nominal string identifiers.

Needs substrate decision:
- GitHub expression language: conditions, `${{ }}` env values, output values, `with` values, concurrency groups, and secret references.
- Whether executable jobs and reusable workflow caller jobs are one record or disjoint variants.
- Whether steps are a permissive record or an exclusive `run`/`uses` sum.
- Secret authority, inheritance, redaction, and Secret Manager/WIF boundaries.
- Runner profile expansion and self-hosted label authority.
- Container credentials and host volume policy.
- Cron helper constructors versus raw cron carrier semantics.
- Modeled-but-not-rendered fields such as `WorkflowRunTrigger.Conclusion`.
- Matrix/strategy support, since it is absent today but likely part of a fuller GitHub Actions model.
- Permission merge/ceiling computation as producer behavior versus declarative substrate rule.

## Single-source Patterns To Preserve

- `workflows.Registry()` is the canonical generated workflow list; comments state both workflowgen and heal should use it to avoid duplication.
- `GeneratedActions()` is the canonical generated composite-action list.
- Generated YAML carries a standard header with source path, regeneration command, and freshness check command.
- `workflowgen -check` compares rendered bytes to checked-in YAML and reports drift.
- Workflow/action definitions are pure Go value construction; `main.go` owns I/O and flag handling.
- Renderers encode deterministic order through ordered entry slices and sorted map keys.
- Shared helpers centralize repeated workflow fragments: default permissions, shell defaults, runner selection, Bazel env, heavy containers, checkout/setup steps, backend dispatch jobs, cached binary steps, and artifact contracts.
- Tests assert schema rendering, permission merging, registry coverage, generated freshness, and recurring workflow contracts; this is the main guard against YAML drift.
