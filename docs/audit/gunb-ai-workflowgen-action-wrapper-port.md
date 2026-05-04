# Audit: gunb.ai workflowgen — action-wrapper & constraint port survey

**Dispatch:** T-Workflow-As-Data prep (survey / catalog only).
**Scope:** Read-only inspection of `gunb-ai/gunb.ai` — especially `tools/ci/workflowgen/actions/`, `tools/ci/workflowgen/constraints/`, and how generated workflows consume wrappers. **No** workflow YAML edits, **no** substrate carriers, **no** CI timing / job-DAG design beyond noting wrapper dependencies.

**Surveyed revision:** shallow clone `gunb-ai/gunb.ai` @ `085febfb7279215039c6ce9fd15659726d7c1e3e` (oneline: `085febf Fix Docker push to GitHub Container Registry (#918)`).

**Generator entrypoints:** `tools/ci/workflowgen/main.go` (`workflowgen` binary), `workflows.Registry()`, `GeneratedActions()` in `main.go`.

---

## 1. Inventory — typed action wrappers

### 1.A Generated composite actions (first-class “typed wrappers”)

`main.go:GeneratedActions()` registers **exactly one** composite action today:

| Action dir (output) | Go authority | Version pinning | Primary inputs | Primary outputs | Secrets / env | Cache behavior | Permissions / assumptions |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `.github/actions/setup-bazel` | `tools/ci/workflowgen/actions/setup_bazel.go` → `actions.SetupBazel()` | **Not** a marketplace action; YAML is generated from Go. Nested `uses:` inside the composite pin third-party actions (below). | Many inputs with defaults: `build-user`, `enable-docker`, `bazelisk-home`, `bazel-cache-dir`, `bazel-disk-cache-dir`, `bazel-repo-cache-dir`, `build-mode`, `disk-cache-policy`, `skip-disk-cache` (deprecated), `secret-envs`, `gcp-workload-identity-provider`, `gcp-service-account`, `gcp-secrets-project`, `gcp-secrets-prefix`. | `bazel-ci-configs`, `disk-cache-enabled`, `disk-cache-reason`, `disk-free-gb` (all from `disk-cache-policy` step outputs). | `secret-envs` drives Secret Manager fetch; WIF inputs for GCP; steps write `GITHUB_ENV` (e.g. `BAZELISK_HOME`, disk-cache flags). | **Inside composite:** `actions/cache@v4` for bazelisk home, repo cache, disk cache; `CachedBinarySteps(BinarySecrets)` before secret load; ordering explicitly documented to avoid slow `bazel run` fallback. | Assumes Ubuntu-style runner with `sudo`, GitHub-hosted disk layout for optional cleanup; optional Docker socket; GCP auth steps use `google-github-actions/auth@v3` + `setup-gcloud@v2`. Smoke workflow (`setup_bazel_smoke.go`) documents permission regressions. |

**Identity:** the composite is the repo-local `ActionRef` `schema.ActionSetupBazel` = `./.github/actions/setup-bazel` (`schema/workflow.go`).

### 1.B Centralized marketplace action pins (`schema.Action*` constants)

`tools/ci/workflowgen/schema/workflow.go` defines pinned `ActionRef` strings reused across workflows/shared helpers:

| Constant | Pin |
| --- | --- |
| `ActionCheckout` | `actions/checkout@v4` |
| `ActionCache` | `actions/cache@v4` |
| `ActionUploadArtifact` | `actions/upload-artifact@v4` |
| `ActionDownloadArtifact` | `actions/download-artifact@v4` |
| `ActionDockerLogin` | `docker/login-action@v3` |
| `ActionDockerMetadata` | `docker/metadata-action@v5` |
| `ActionDockerSetupQEMU` | `docker/setup-qemu-action@v3` |
| `ActionDockerSetupBuildx` | `docker/setup-buildx-action@v3` |
| `ActionDockerBuildPush` | `docker/build-push-action@v6` |
| `ActionFreeDiskSpace` | `thiagokokada/free-disk-space@main` |
| `ActionCreateGitHubAppToken` | `actions/create-github-app-token@v1` |

**Note:** `@main` on `free-disk-space` is a moving pin (policy / supply-chain consideration for any future extdeps modeling).

### 1.C Ad hoc pins in workflow packages (not routed through `schema.Action*`)

| Location | `uses` | Role |
| --- | --- | --- |
| `workflows/stylelint_util.go` | `actions/setup-node@v4` | Node for stylelint |
| `workflows/stale_branch_cleanup.go` | `actions/github-script@v7` | Branch cleanup automation |
| `workflows/live_tests.go` | `google-github-actions/auth@v3`, `setup-gcloud@v2`, `aws-actions/configure-aws-credentials@v4` | Cloud auth matrix |
| `workflows/gcp_deploy.go` | same GCP pair + `actions/upload-artifact@v4` | Deploy / artifacts |
| `workflows/promote_heavy_image.go` | `docker/setup-buildx-action@v3` (+ `schema.ActionDockerLogin` elsewhere in file) | Buildx promotion path |
| `actions/setup_bazel.go` | `google-github-actions/auth@v3`, `setup-gcloud@v2` (as `schema.ActionRef("…")`) | Secret Manager + gcloud inside composite |
| `shared/steps.go` | GCP pair in helper(s) | Shared auth plumbing |
| `shared/backend_change_util.go` | `google-github-actions/auth@v3` | IAP / backend checks |
| `shared/secret_deps.go` | `actions/create-github-app-token@v1` | Token minting pattern |

### 1.D Reusable workflow calls (not marketplace actions, but “typed” `uses`)

Examples: `workflows/scheduled.go`, `workflows/presubmit.go` use `uses: ./.github/workflows/<name>.yml` for `bazel-ci`, `live-tests`, `devcontainer-tests`, etc. These are **generated** reusable workflows, not external marketplace wrappers — still part of the port surface because they form a **nested workflow dependency graph** (see `workflows/workflows_test.go` path-closure tests).

### 1.E “Ops DAG” resilient operations (`shared/predefined_ops.go`, `shared/resilient_op.go`, …)

Not GitHub Actions wrappers, but **typed CI setup DAG nodes** (bazelisk download, APT locks, bazel-diff JAR, etc.) composed into shell emitted by generators. These are strong candidates for a future **workflow-as-data** carrier (behavior + locks + exports), distinct from marketplace `uses:` rows.

---

## 2. Catalog — constraints wrappers

`tools/ci/workflowgen/constraints/` contains **one** substantive constraint module:

| Constraint | File | What it models | Workflow validity effect |
| --- | --- | --- | --- |
| **Bazel analysis cache / build-config mixing** | `bazel_cache.go` | `AnalysisCacheConstraint` over `BazelBuildConfig` (`standard` vs `coverage`); infers config from `step.Run` text (`bazel coverage`, flags, etc.). | `ValidateJobAnalysisCache` flags steps in the **same job** that may both run with different configs **without** mutually exclusive `if:` conditions — expensive analysis-cache thrash / incorrect assumptions. |
| **Tests** | `bazel_cache_test.go`, `workflows/caching_test.go` | Golden / documentation tests for constraint behavior. | CI for workflowgen itself — not emitted workflow constraints. |

**Runner selection** lives in `shared/runner_selector.go` / `shared/constants.go` (global config from `main.go` `-self-hosted` flag) — affects **which runner labels** jobs get, not YAML validity of `uses:` pins.

**Secret dependency graph** (`shared/secret_deps.go`, `dag/secrets.go`) models **which secrets must be materialized before which steps** — adjacent to constraints, but closer to “policy DAG” than Bazel cache rules.

---

## 3. Mapping table — gunb.ai → gunbc (candidate extdeps / workflow carrier / policy)

Rows are **candidates** for T-Workflow-As-Data / extdeps alignment, not commitments.

| gunb.ai surface | gunbc candidate | Notes / open questions |
| --- | --- | --- |
| `schema.Workflow` / `schema.Job` / `schema.Step` | `dsl/extdeps/github/actions.dag` (`Workflow`, `Job`, `Step`, `UsesStep`, …) plus a required `uses`-target extension | gunbc already has a **platform-shaped** Actions model, but its current `ActionRef { owner, repo, ref }` only represents marketplace-style action refs. It cannot represent the local composite action (`./.github/actions/setup-bazel`) or reusable workflow calls (`./.github/workflows/<name>.yml`) catalogued above. The port therefore needs a `uses` target coproduct or sibling carrier for `MarketplaceAction`, `LocalCompositeAction`, and `ReusableWorkflowCall`, in addition to any typed marketplace-action catalog. |
| `ActionRef` constants (`checkout@v4`, `cache@v4`, …) | New `extdeps.github.marketplace_actions` or rows in `extdeps` registry | Centralize pins + semver policy (`@main` risk on `free-disk-space`). Open: who owns version bumps — generator vs substrate? |
| `actions/setup_bazel` composite | **Workflow carrier** slice + small **GCP extdeps** (`extdeps/cloud/gcp/*` already exists) | Composite is large shell + nested actions; port likely **structured steps** referencing extdeps for **Secret Manager**, **WIF**, **cache keys** — not a 1:1 YAML string lift. |
| `google-github-actions/auth`, `setup-gcloud` | `extdeps/cloud/gcp/*` + thin “CI auth step” carrier | Map WIF inputs to existing GCP STS / IAM facts; keep **non-secret** parameter shapes in .dag. |
| `actions/create-github-app-token` | `extdeps/github/*` (expand beyond REST pulls/gists) | Today token story is documented in Go comments (`workflowgen_util.go`); need explicit **App auth** extdep or defer until gunbc CI needs it. |
| Docker build/push stack (`docker/*-action@v*`) | Defer or **extdeps** “container registry ops” | Heavily gunb.ai release-train specific; gunbc may only need a subset for future publishing lanes. |
| `aws-actions/configure-aws-credentials` | `extdeps/cloud/...` (future AWS lane) or **defer** | Only appears in `live_tests.go` paths. |
| `AnalysisCacheConstraint` | **Policy / constraint candidate** | Fits “workflow validity beyond YAML schema” — could become a **lint lens** over a workflow-as-data IR (same job, incompatible Bazel modes). |
| `secret_deps` / job DAG | **Policy candidate** (partial) | Maps secret readiness; overlaps future “evidence DAG” for CI — keep separate from marketplace `uses`. |
| `shared/predefined_ops` (`ResilientOp`, locks, exports) | **Workflow carrier** (ops DAG) | Natural mid-layer between `.dag` intent and YAML emission; extdeps for **URLs/versions** (`tools/extdeps/versions` in gunb.ai) already mirror gunbc’s “versions in one place” pattern. |
| `BinarySecrets`, `BinaryHeal`, `BazeliskCacheSteps` | Mix: **extdeps** (binary URLs, versions) + **carrier** (cache key algebra) | gunbc `cargo` / tool pins live in different infra; reuse **pattern** (hashFiles inputs, cache dir conventions) more than literal paths. |

---

## 4. Classification summary

| Bucket | Members (high level) |
| --- | --- |
| **Direct `.dag` extdeps candidate** | Pinned marketplace actions (`schema.Action*` table + ad hoc `uses` strings); external URLs in bazelisk / bazel-diff download scripts (`versions` package); GCP/AWS auth action **parameters** (not the YAML itself). |
| **Workflow carrier candidate** | `schema.CompositeAction` / `schema.Workflow` emission pipeline; `ResilientOp` / `OpsDAG`; `setup-bazel` step sequence as structured data; reusable workflow `uses: ./.github/workflows/...` graph. |
| **Policy / constraint candidate** | `constraints/bazel_cache.go` analysis-cache rule; `secret_deps` ordering rules; permissions merge helpers (`schema` permissions aggregation). |
| **Defer** | Docker promote heavy image path specifics; AWS live-test matrix; `thiagokokada/free-disk-space@main` pin policy unless gunbc adopts same host hygiene. |

---

## 5. Wrapper patterns worth lifting to reusable substrate / extdeps

1. **Single pin registry:** `schema/workflow.go` constants are the right idea; avoid scattering `"actions/foo@vN"` in individual workflow files when adding gunbc workflowgen — prefer one module-level registry (extdeps or generated).

2. **Composite-as-data:** `schema.CompositeAction` + `RenderAction` already treat actions as **structured** values (inputs/outputs/steps). That is the template for a future gunbc “workflow carrier” without stringly YAML.

3. **Cache + ownership choreography:** `BazeliskCacheSteps` and `CachedBinarySteps` encode **ordering + cache-hit env + sudo chmod** invariants — these are easy to get wrong in ad hoc YAML; a substrate-level “cache mount step” pattern (or extdeps facts about runner image layout) would reduce duplication.

4. **Disk / mode policy:** `DiskCachePolicy` + probe script in `setup_bazel.go` couples **runner disk truth** to **Bazel flags** — good candidate for a named policy object in workflow-as-data (vs long `run: |` blocks).

5. **Constraint hooks:** `ValidateJobAnalysisCache` shows value in **post-generation validation** over the IR — mirror as a gunbc pass over workflow DAG before render.

6. **Single IR for theory vs emit:** gunb.ai keeps **one** generated composite today, but marketplace pins still split across `schema` constants and ad hoc `uses` strings. gunbc’s `extdeps/github/actions.dag` is already a declarative “theory” of Actions. Long-term, **one IR** should feed validation, constraints, and YAML render to avoid drift between those surfaces.

---

## 6. Follow-ups (out of scope for this slice)

- Land this brief on `main` via normal PR path when the worktree is clean (commit from a non-detached branch if the agent worktree was mid-rebase).

---

## 7. Reply hook (dashboard)

When the brief is merged or PR is opened, reply on **inbox #1130** with the PR URL or commit SHA pointing at `docs/audit/gunb-ai-workflowgen-action-wrapper-port.md`.
