# SDLC Operational Readiness Map

Single-page reference for every credential, endpoint, GCP resource, and env var
the SDLC pipeline touches. Covers both the **local dev** path (env vars, file
ledgers) and the **prod/cloud_run** path (GCP Secret Manager, GCS, PubSub,
Cloud Run).

---

## 1. GCP Projects

| Project | ID | Number | Purpose |
|---------|-----|--------|---------|
| gunbai-auto | `gunbai-auto` | `314501921854` | WIF pool host, Cloud Run, GCS buckets, PubSub |
| gunbai-secrets | `gunbai-secrets` | `582015116396` | Secret Manager (all namespaces) |

Source: `lib/cloud-ops/src/project_spec.rs`

---

## 2. Credential Resolution Architecture

Two completely different credential paths depending on profile:

### `local` profile — env var passthrough
```
env("GITHUB_TOKEN")  -->  std::env::var("GITHUB_TOKEN")  -->  IssueProvider
env("CODEX_API_KEY")  -->  std::env::var("CODEX_API_KEY")  -->  AgentProvider
ANTHROPIC_API_KEY  -->  read from env directly by LLM stages (no profile binding)
```

### `cloud_run` profile — GCP Secret Manager via DAG
```
secret("github-token")
  --> BindSecretName (sets secret.name = "github-token")
  --> CloudRuntimeKind dispatch:
      GitHubActions:  OIDC token from ACTIONS_ID_TOKEN_REQUEST_URL
      CloudMetadata:  metadata server at http://metadata.google.internal
      LocalDev:       ADC from ~/.config/gcloud/application_default_credentials.json
  --> STS exchange (sts.googleapis.com) --> Google access token
  --> SA impersonation (iamcredentials.googleapis.com) --> SA token
  --> Secret Manager access (secretmanager.googleapis.com) --> base64 payload
  --> Credential struct --> DAG consumer
```

Source:
- DAG wiring: `lib/gcp-ops/src/graph.rs` (build_gcp_secret_manager_credential_graph)
- Ops: `lib/gcp-ops/src/ops.rs` (PrepareGitHubOidcRequest, PrepareStsExchange, PrepareImpersonate, PrepareSecretAccess)
- Dispatch: `lib/cloud-ops/src/graph.rs` (build_cloud_secret_manager_credential_graph_from_config)
- Config resolution: `lib/cloud-ops/src/config_loader.rs` (graph_cloud_config, detect_runtime)

---

## 3. Workload Identity Federation (GitHub Actions -> GCP)

| Field | Value |
|-------|-------|
| Pool | `github-pool` in `gunbai-auto` (314501921854) |
| Provider | `github` |
| OIDC Issuer | `https://token.actions.githubusercontent.com` |
| Attribute condition | `assertion.repository_owner == 'gunb-ai'` |
| Provider resource name | `projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github` |

Provisioned by: `gunbc-infra bootstrap`

Source: `lib/cloud-ops/src/project_spec.rs:341-351`

---

## 4. Service Accounts (per namespace)

All SAs live in project `gunbai-secrets`.
Format: `gunbai-{ns}-secrets@gunbai-secrets.iam.gserviceaccount.com`

| Namespace | SA Name | Prefix | Role | WIF Binding |
|-----------|---------|--------|------|-------------|
| dev | `gunbai-dev-secrets` | `dev-` | `roles/secretmanager.secretAccessor` | github-pool/gunb-ai/gunbc |
| ci | `gunbai-ci-secrets` | `ci-` | same | same |
| test | `gunbai-test-secrets` | `test-` | same | same |
| staging | `gunbai-staging-secrets` | `staging-` | same | same |
| prod | `gunbai-prod-secrets` | `` (empty) | same | same |
| review | `gunbai-review-secrets` | `review-` | same | same |
| llm | `gunbai-llm-secrets` | `llm-` | same | same |
| ops | `gunbai-ops-secrets` | `ops-` | same | same |
| sandbox | `gunbai-sandbox-secrets` | `sandbox-` | same | same |

**Prod uses empty prefix** -- secrets are stored under their canonical names
(e.g., `github-token` not `prod-github-token`).

Provisioned by: `gunbc-infra bootstrap`

---

## 5. Secret Manager Catalog

Secrets in `gunbai-secrets`, prefixed by namespace (except prod which has no prefix):

| Env Var | Secret ID | Requirement | Rotation | Scopes |
|---------|-----------|-------------|----------|--------|
| `GITHUB_TOKEN` | `github-token` | Optional* | GitHubPat (90d) | `repo`, `read:org`, `gist` |
| `ANTHROPIC_API_KEY` | `anthropic-api-key` | Required | Manual | -- |
| `OPENAI_API_KEY` | `openai-api-key` | Optional | Manual | -- |
| `CODEX_API_KEY` | `codex-api-key` | Required | Manual | -- |

*`GITHUB_TOKEN` is "Optional" in the catalog but **required** for both profiles'
IssueProvider.

**Secret naming by namespace:**

| Namespace | github-token | anthropic-api-key | codex-api-key |
|-----------|-------------|-------------------|---------------|
| dev | `dev-github-token` | `dev-anthropic-api-key` | `dev-codex-api-key` |
| ci | `ci-github-token` | `ci-anthropic-api-key` | `ci-codex-api-key` |
| prod | `github-token` | `anthropic-api-key` | `codex-api-key` |

Provisioned by: `gunbc-infra apply --env <namespace>` (creates Secret Manager
entries via upsert DAG; values must be populated manually or via rotation).

Source: `lib/cloud-ops/src/project_spec.rs:543-579`

---

## 6. Infrastructure Provisioning Status

### Automated via `gunbc-infra`

| Resource | Command | Implementation |
|----------|---------|----------------|
| WIF pool + provider | `gunbc-infra bootstrap` | `lib/cloud-ops/src/infra_bootstrap.rs` |
| Service accounts (all namespaces) | `gunbc-infra bootstrap` | same |
| IAM role bindings (secretAccessor) | `gunbc-infra bootstrap` | same |
| WIF principal bindings | `gunbc-infra bootstrap` | same |
| Secret Manager entries (create + version) | `gunbc-infra apply --env <ns>` | `lib/cloud-ops/src/infra_plan_apply.rs` |

### NOT automated (manual provisioning required for cloud_run)

| Resource | Declared In | Provisioning |
|----------|-------------|--------------|
| GCS bucket: `gunbai-auto-sdlc-claims` | `dsl/infra/sdlc/deploy.dag` | Manual: `gsutil mb` or console |
| GCS bucket: `gunbai-auto-sdlc-outcomes` | `dsl/infra/sdlc/deploy.dag` | Manual |
| GCS bucket: `gunbai-auto-sdlc-artifacts` | `dsl/infra/sdlc/deploy.dag` | Manual |
| GCS bucket: `gunbai-auto-sdlc-ledger` | `infra/sdlc/cloud-run-service.yaml` | Manual (GCS Fuse mount) |
| PubSub topic: `sdlc-signals` | `dsl/profiles/sdlc.dag` | Manual (profile still uses stub) |
| PubSub subscription: `sdlc-worker` | `dsl/profiles/sdlc.dag` | Manual (profile still uses stub) |
| Cloud Run service: `gunbc-sdlc-worker` | `infra/sdlc/cloud-run-service.yaml` | Manual: `gcloud run deploy` |
| Cloud Scheduler job | `infra/sdlc/cloud-scheduler-job.yaml` | Manual: `gcloud scheduler jobs create` |
| Container image | Referenced in Cloud Run YAML | No build/push pipeline |
| Scheduler -> Cloud Run IAM | Not modeled | Manual |

**Note**: `dsl/infra/sdlc/deploy.dag` declares functions (`deploy_ledger_bucket`,
`deploy_worker_service`, etc.) but these are DAG specifications without a Rust
executor -- the imperative provisioning logic only covers secrets and WIF today.

---

## 7. Environment Variables

### `local` profile -- required env vars

| Env Var | Profile Binding | Used By |
|---------|----------------|---------|
| `GITHUB_TOKEN` | `env("GITHUB_TOKEN")` | IssueProvider (GitHub Issues API) |
| `CODEX_API_KEY` | `env("CODEX_API_KEY")` | AgentProvider (Codex CLI subprocess) |
| `ANTHROPIC_API_KEY` | None (read from env directly) | Design/DesignReview/CodeReview stages |

### `cloud_run` profile -- no env vars needed

All credentials resolved via `secret()` expressions through the GCP DAG.
Runtime detection (`detect_runtime()`) selects the token acquisition path.

### Optional overrides (both profiles)

| Env Var | Default | Purpose |
|---------|---------|---------|
| `SDLC_PROFILE` | `local` | Select deployment profile |
| `SDLC_WORKER_ID` | `worker-{pid}-{epoch}` | Worker identity for claim ownership |
| `SDLC_REPO_OWNER` | `gunb-ai` | GitHub org/owner |
| `SDLC_REPO_NAME` | `gunbc` | GitHub repo name |
| `SDLC_LLM_PROVIDER` | `anthropic` | LLM provider selection |
| `SDLC_LLM_MODEL` | `claude-sonnet-4-20250514` | LLM model ID |

### Cloud config resolution (precedence order)

1. `GUNBC_CLOUD_CONFIG_PATH` (explicit path)
2. `GUNBC_CLOUD_CONFIG_JSON` (inline JSON)
3. `GUNBC_CLOUD_CONFIG_TOML` (inline TOML)
4. Legacy `GCP_*` env vars
5. Profile file `.gunbc/config-{profile}.toml`
6. Default: `dev` namespace with `LocalDev` runtime

Source: `lib/cloud-ops/src/config_loader.rs:238-285`

---

## 8. Network Endpoints (per stage)

### GitHub API (`api.github.com`)
- **All stages** (via IssueProvider): `GET/PATCH /repos/{owner}/{repo}/issues/{number}`, labels, comments
- **Implementing -> CodeReview**: `POST /repos/{owner}/{repo}/pulls` (PR create)
- **CodeReview**: `GET /repos/{owner}/{repo}/pulls/{number}` (PR diff/files)
- Credential: `github-token` from Secret Manager (cloud_run) or `GITHUB_TOKEN` env (local)

### Anthropic API (`api.anthropic.com`)
- **Design** (Idea -> Design): `POST /v1/messages`
- **DesignReview** (Design -> DesignReview): `POST /v1/messages`
- **CodeReview** (CodeReview -> Testing): `POST /v1/messages`
- Credential: `anthropic-api-key` from Secret Manager (cloud_run) or `ANTHROPIC_API_KEY` env (local)

### Codex CLI (local subprocess)
- **Implementing** (Accepted -> Implementing): spawns `codex` binary in CWD
- Credential: `codex-api-key` from Secret Manager (cloud_run) or `CODEX_API_KEY` env (local)

### GCP APIs (cloud_run credential resolution)
- `sts.googleapis.com` -- STS token exchange (WIF)
- `iamcredentials.googleapis.com` -- SA impersonation
- `secretmanager.googleapis.com` -- secret payload fetch
- `storage.googleapis.com` -- GCS for claims/outcomes/artifacts
- `pubsub.googleapis.com` -- signal delivery (when implemented)
- `run.googleapis.com` -- Cloud Run service (deployment only)

---

## 9. Local File Paths (`local` profile)

All under project root, created automatically:

| Path | Purpose |
|------|---------|
| `target/sdlc/claims/` | FileClaimStore -- JSON claim records |
| `target/sdlc/outcomes/` | FileOutcomeLedger -- JSON outcome records |
| `target/sdlc/signals/` | FileSignalStore -- JSON signal records |
| `target/sdlc/intake/ledger.json` | Intake ledger (intent -> issue mappings) |
| `target/sdlc/claims/ledger.json` | Claim ledger (worker ownership) |
| `target/sdlc/artifacts/ledger.json` | Artifact ledger (provisional -> canonical) |
| `target/sdlc/run_state/ledger.json` | Run state ledger (per-stage execution status) |
| `target/sdlc/issue_transport/ledger.json` | Issue transport ledger (GitHub API call log) |
| `target/sdlc/agent/ledger.json` | Agent ledger (Codex agent status) |
| `target/sdlc/drain_flag` | Drain flag file (worker graceful shutdown) |

---

## 10. GCP Resources (cloud_run profile)

| Resource | Type | Project | Region | Notes |
|----------|------|---------|--------|-------|
| `gunbai-auto-sdlc-claims` | GCS Bucket | gunbai-auto | us-central1 | Generation-based CAS for claims |
| `gunbai-auto-sdlc-outcomes` | GCS Bucket | gunbai-auto | us-central1 | Outcome ledger |
| `gunbai-auto-sdlc-artifacts` | GCS Bucket | gunbai-auto | us-central1 | Artifact storage |
| `gunbai-auto-sdlc-ledger` | GCS Bucket | gunbai-auto | us-central1 | GCS Fuse mount for Cloud Run |
| `sdlc-signals` | PubSub Topic | gunbai-auto | -- | At-least-once signal delivery |
| `sdlc-worker` | PubSub Subscription | gunbai-auto | -- | Worker signal pull |
| `gunbc-sdlc-worker` | Cloud Run Service | gunbai-auto | us-central1 | Stateless worker fleet |

Source: `dsl/profiles/sdlc.dag:92-114`, `dsl/infra/sdlc/deploy.dag`, `infra/sdlc/cloud-run-service.yaml`

---

## 11. Stage-by-Stage Execution Map

| Stage Transition | APIs Called | Credentials Needed | Artifacts Produced |
|-----------------|------------|-------------------|-------------------|
| **Intake** | GitHub Issues (create/find) | github-token | Issue created, intake ledger entry |
| **Idea -> Design** | Anthropic Messages | anthropic-api-key, github-token | Design document (issue comment) |
| **Design -> DesignReview** | Anthropic Messages, GitHub Issues | anthropic-api-key, github-token | Review feedback (issue comment) |
| **DesignReview -> Accepted** | GitHub Issues (labels) | github-token | `awaiting_approval=true`, manual gate |
| **Accepted -> Implementing** | Codex CLI (spawn) | codex-api-key, github-token | Agent branch, code changes |
| **Implementing -> CodeReview** | Codex (poll), GitHub PRs (create) | codex-api-key, github-token | Pull request |
| **CodeReview -> Testing** | Anthropic Messages, GitHub PRs (diff) | anthropic-api-key, github-token | Review verdict (PR comment) |
| **Testing -> Done** | `cargo test`, `cargo clippy`, GitHub Issues | github-token | Test results, issue closed |

---

## 12. Operational Gaps

### GAP-1: `codex-api-key` missing from KNOWN_SECRETS -- RESOLVED
Added to `KNOWN_SECRETS` in `project_spec.rs`. Both provisioning (`gunbc-infra apply`)
and runtime credential DAG now know about this secret.

### GAP-2: ANTHROPIC_API_KEY not wired through cloud_run profile
The `local` profile reads `ANTHROPIC_API_KEY` from env, and the LLM stages read
it directly from env. The `cloud_run` profile does NOT have a `secret("anthropic-api-key")`
binding for the LLM stages -- they bypass the profile system entirely.
**Impact**: In cloud_run, LLM stages would need `ANTHROPIC_API_KEY` set as a Cloud Run
env var (from Secret Manager via Cloud Run secret mounting), or the LLM graph needs
to use `graph_cloud_config()` to resolve from Secret Manager.
**Current state**: The `llm-ops` graph (`lib/llm-ops/src/graph.rs`) DOES use
`graph_cloud_config()` → `build_cloud_secret_manager_credential_graph`, so the
DAG path works for LLM. The gap is that the profile doesn't document this.

### GAP-3: `.env.example` template -- RESOLVED
Created at repo root. Clearly scoped to local profile only.

### GAP-4: DesignReview pauses for manual approval
Intentional human gate. Worker yields, releases claim. Resume via
`gunbc-sdlc await-approval --intake-key <key>` or manual label change.

### GAP-5: Codex agent runs in CWD
Must invoke `gunbc-sdlc worker` from repo root.

### GAP-6: Testing stage tests current checkout
Runs `cargo test`/`cargo clippy` on CWD, not PR branch. Fine for local dry-run;
Cloud Run worker would need to checkout PR head.

### GAP-7: Intent file -- RESOLVED
Created `TODO/feature-intent-markdown.yaml`.

### GAP-8: Cloud Scheduler not modeled in code
YAML template exists at `infra/sdlc/cloud-scheduler-job.yaml` (cron: `*/5 * * * *`),
but no automated provisioning. Manual `gcloud scheduler jobs create` needed.

### GAP-9: GCS/PubSub/Cloud Run provisioning not automated
DAG specs exist in `dsl/infra/sdlc/deploy.dag` but the imperative Rust executors
only cover secrets and WIF bootstrap. Storage, messaging, and compute resources
require manual provisioning before cloud_run profile works.

### GAP-10: PubSub SignalStore still uses stub
The `cloud_run` profile declares `gcp.PubSubSignalStore` but the profile source
comments note `TODO: pubsub` and `TODO: gcs` for SignalStore and ArtifactStore.
The concrete implementations may not exist yet.

---

## 13. Prod Deployment Sequence (zero to running)

```
# Phase 1: Identity infrastructure (automated)
gunbc-infra bootstrap                    # WIF pool, provider, SAs, IAM bindings

# Phase 2: Secret entries (automated, values manual)
gunbc-infra apply --env prod             # Creates Secret Manager entries
# Then manually populate secret values:
#   gcloud secrets versions add github-token --data-file=- < token.txt
#   gcloud secrets versions add anthropic-api-key --data-file=- < key.txt
#   gcloud secrets versions add codex-api-key --data-file=- < key.txt

# Phase 3: Storage (manual)
gsutil mb -p gunbai-auto -l us-central1 gs://gunbai-auto-sdlc-claims
gsutil mb -p gunbai-auto -l us-central1 gs://gunbai-auto-sdlc-outcomes
gsutil mb -p gunbai-auto -l us-central1 gs://gunbai-auto-sdlc-artifacts
gsutil mb -p gunbai-auto -l us-central1 gs://gunbai-auto-sdlc-ledger

# Phase 4: Messaging (manual, when PubSub impl ready)
gcloud pubsub topics create sdlc-signals --project=gunbai-auto
gcloud pubsub subscriptions create sdlc-worker \
  --topic=sdlc-signals --project=gunbai-auto

# Phase 5: Compute (manual)
# Build and push container image
# Deploy Cloud Run service from infra/sdlc/cloud-run-service.yaml
# Create Cloud Scheduler job from infra/sdlc/cloud-scheduler-job.yaml
```

---

## 14. Local Dry-Run Checklist

### Prerequisites
1. [ ] `cargo build -p gunbc-dag --bin gunbc-sdlc` compiles clean
2. [ ] `GITHUB_TOKEN` set (PAT with `repo` scope)
3. [ ] `ANTHROPIC_API_KEY` set
4. [ ] `CODEX_API_KEY` set
5. [ ] `codex` binary on PATH
6. [ ] Intent YAML file ready (`TODO/feature-intent-markdown.yaml`)
7. [ ] Working directory is repo root

### Execution sequence
```bash
# 1. Build
cargo build -p gunbc-dag --bin gunbc-sdlc

# 2. Intake -- creates GitHub issue, initializes ledgers
cargo run -p gunbc-dag --bin gunbc-sdlc -- intake --intent TODO/feature-intent-markdown.yaml

# 3. Worker -- processes first ready stage (Idea -> Design)
cargo run -p gunbc-dag --bin gunbc-sdlc -- worker --profile local

# 4. Repeat worker for subsequent stages
#    (DesignReview will pause for approval)
cargo run -p gunbc-dag --bin gunbc-sdlc -- worker --profile local

# 5. After manual approval:
cargo run -p gunbc-dag --bin gunbc-sdlc -- worker --profile local

# 6. Continue until Done
```

### Dry-run mode (no side effects)
```bash
cargo run -p gunbc-dag --bin gunbc-sdlc -- worker --dry-run --profile local
```

---

## 15. Summary

| # | Action | Priority | Profile | Status |
|---|--------|----------|---------|--------|
| 1 | Add `codex-api-key` to `KNOWN_SECRETS` | P0 | Both | Done |
| 2 | Set env vars for local dry-run | P0 | local | Manual |
| 3 | Ensure `codex` on PATH | P0 | local | Manual |
| 4 | Write intent YAML | P0 | Both | Done |
| 5 | Create `.env.example` | P2 | local | Done |
| 6 | Run `gunbc-infra bootstrap` | P0 | cloud_run | Manual |
| 7 | Run `gunbc-infra apply --env prod` | P0 | cloud_run | Manual |
| 8 | Populate secret values in Secret Manager | P0 | cloud_run | Manual |
| 9 | Create GCS buckets | P0 | cloud_run | Manual |
| 10 | Create PubSub topic/subscription | P1 | cloud_run | Blocked (stub impl) |
| 11 | Deploy Cloud Run service | P1 | cloud_run | Manual |
| 12 | Create Cloud Scheduler job | P2 | cloud_run | Manual |
| 13 | Verify build | P0 | Both | Pending |
