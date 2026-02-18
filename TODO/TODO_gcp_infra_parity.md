# GCP Infrastructure Parity: gunbc vs gunb.ai

**Status**: In Progress (planning + phased implementation tracker)
**Date**: 2026-02-14
**DSL Alignment**: Domain parity backlog; target DSL consumer after core migration tracks stabilize
**Track**: E — Domain Parity

## Context

gunb.ai has a mature GCP infrastructure-as-code system with:
- Typed specs for 15+ resource types across 6 environments
- DAG-based plan/apply semantics with dependency ordering
- Bootstrap for initial project setup (WIF, SAs, IAM)
- Secret management with rotation, direnv, and caching
- Full compute stack (VMs, MIGs, load balancers, Cloud Run)
- Multi-project support (compute, secrets, CI)

gunbc currently models:
- 5 GCP service interfaces (Resource Manager, IAM, Secret Manager, WIF, Storage)
- 1 project spec (`GUNBAI_SECRETS`) with expanded namespace/service-account catalog (8 namespaces)
- Credential + upsert graphs for Secret Manager
- Discovery ops (list projects/SAs/secrets/buckets/WIF)
- IAM ensure pattern (auto-grant secretAccessor role)
- Self-healing OAuth (try-refresh → gcloud-auth → retry)

This TODO tracks the work needed to reach parity with gunb.ai's core
infrastructure modeling, adapted to gunbc's Rust/DAG architecture.

---

## Phase 1: Service Account & IAM Lifecycle (HIGH — blocks everything)

gunb.ai has full SA CRUD with roles, self-roles, service account users, and
WIF bindings. gunbc only has list/get/impersonate.

### 1.1 Service Account CRUD

**Gap**: gunbc cannot create, update, or delete service accounts.

- [x] Add `create_service_account(project, account_id, display_name)` to `IamService` trait
- [x] Add `update_service_account(project, email, display_name)` to `IamService` trait
- [x] Add `delete_service_account(project, email)` to `IamService` trait
- [x] Add `IamRest` implementations + tests
- [x] Add `MethodMeta` constants for each method

**Ref**: `gunb.ai/tools/infra/gcloud/admin.go` — `CreateServiceAccount`, `EnsureServiceAccount`

### 1.2 Service Account IAM Bindings

**Gap**: gunbc can only manage project-level IAM. SA-level IAM (who can
impersonate) is missing.

- [x] Add `get_service_account_iam_policy(project, email)` to `IamService`
- [x] Add `set_service_account_iam_policy(project, email, policy)` to `IamService`
- [x] Add ops: `PrepareEnsureSaIamBinding` / `CheckAndPrepareSaIamBinding` / `ParseSetSaIamBinding`
- [x] Generalize `add_ensure_iam_nodes()` to work for both project-level and SA-level bindings

**Ref**: `gunb.ai/tools/infra/dag/builder.go:258-300` — SA IAM binding ops

### 1.3 SA Spec in Project Spec

**Gap**: `project_spec.rs` has `ServiceAccountSpec` with name + roles but
no display_name, self_roles, service_account_users, or WIF bindings.

- [x] Add `display_name: &'static str` to `ServiceAccountSpec`
- [x] Add `self_roles: &'static [&'static str]` (roles the SA has on itself, e.g. tokenCreator)
- [ ] Add `service_account_users: &'static [&'static str]` (members who can act-as)
- [x] Add `wif_bindings` to `ServiceAccountSpec`
- [ ] Add `WifBindingSpec { pool: &str, provider: &str, attribute: &str }`
- [ ] Add derived method `wif_member(&self, project_number: &str) -> String`

**Ref**: `gunb.ai/tools/infra/spec/spec.go:94-130` — `ServiceAccountSpec`

### 1.4 Expand SA Catalog

**Gap**: gunbc only has 2 SAs (dev-secrets, ci-secrets). gunb.ai has ~8
SAs with distinct roles.

- [x] Model the full SA catalog needed for gunbc's infrastructure:
  - `gunbai-dev-secrets` — dev secret access (exists)
  - `gunbai-ci-secrets` — CI secret access (exists)
  - `gunbai-deployer` — GitHub Actions deployment (compute.admin, iam.serviceAccountAdmin, etc.)
  - `gunbai-ci-runner` — CI runner VM operations
  - Other SAs as needed for the compute/CI stack
- [x] Add roles, self-roles, WIF bindings to each SA spec
- [x] Add tests for derived values (emails, WIF binding/member projections)

**Ref**: `gunb.ai/tools/infra/spec/spec.go:670-750` — SA definitions per env

---

## Phase 2: WIF Bootstrap (HIGH — needed for CI)

gunb.ai has a full bootstrap command that creates WIF pools, providers,
and initial SA bindings. gunbc only has WIF discovery (list/get).

### 2.1 WIF Pool/Provider CRUD

**Gap**: gunbc can discover WIF pools/providers but cannot create or update them.

- [x] Add `create_pool(project, pool_id, display_name)` to `WorkloadIdentityService`
- [x] Add `create_provider(project, pool_id, provider_id, config)` to `WorkloadIdentityService`
- [x] Add `update_provider(project, pool_id, provider_id, config)` to `WorkloadIdentityService`
- [x] Add `WifProviderConfig` struct with OIDC issuer URI, attribute mapping, attribute condition
- [x] Add REST implementations + tests

**Ref**: `gunb.ai/tools/infra/gcloud/admin.go` — `CreateWorkloadIdentityPool`, `CreateWorkloadIdentityProvider`

### 2.2 Bootstrap DAG

**Gap**: No equivalent to `infra bootstrap` — initial project setup must
be done manually.

- [ ] Create `build_wif_bootstrap_dag()` that:
  1. Creates WIF pool (idempotent)
  2. Creates/updates WIF provider with GitHub OIDC config
  3. Creates service accounts from spec
  4. Grants project-level IAM roles
  5. Grants SA-level IAM roles (act-as, tokenCreator)
  6. Creates WIF principal bindings on SAs
- [ ] Add CLI entrypoint: `make bootstrap` or `make infra-bootstrap`
- [ ] Output GitHub Actions configuration variables (WIF provider, SA emails)

**Ref**: `gunb.ai/tools/infra/cmd/infra/main.go:888-1085` — bootstrap command

### 2.3 WIF Spec

**Gap**: `WifConfig` in project_spec.rs only has pool_id and provider_id.

- [x] Add OIDC issuer URI (e.g., `https://token.actions.githubusercontent.com`)
- [x] Add attribute mapping: `google.subject = assertion.sub`, `attribute.repository = assertion.repository`
- [x] Add attribute condition (e.g., `assertion.repository_owner == 'org-name'`)
- [x] Derive full resource names from project number + pool + provider

**Ref**: `gunb.ai/tools/infra/spec/spec.go:59-64, 122-130`

---

## Phase 3: Secret Manager Full Lifecycle (MEDIUM)

gunbc has Secret Manager CRUD and an upsert graph. gunb.ai adds rotation,
direnv caching, and multi-project secret management.

### 3.1 Secret Rotation

**Gap**: `SecretSpec` has `rotation: RotationHandler` enum but no runtime
rotation logic.

- [x] Implement `rotate_secret()` logic per handler type:
  - `Manual` — prompt / instructions only
  - `GitHubPat` — generate new PAT via GitHub API
  - `None` — skip
- [x] Add `max_age: Option<Duration>` to `SecretSpec` for rotation warnings
- [x] Add `check_secret_age()` op that compares version create time against max_age
- [ ] Add CLI entrypoint: `make rotate-secrets` or integrate into preflight

**Ref**: `gunb.ai/tools/secrets/cmd/secrets/rotate.go`

### 3.2 Secret Provisioning DAG

**Gap**: gunbc can upsert individual secrets but has no DAG that
provisions ALL secrets from the spec.

- [x] Create `build_secrets_provision_dag()` that iterates `KNOWN_SECRETS`:
  1. For each secret: check existence → create if missing
  2. Grant IAM access to appropriate SAs
  3. Report status (created, exists, missing value)
- [ ] Add CLI entrypoint: `make ensure-secrets`
- [ ] Integrate into `make bootstrap` flow

Update (2026-02-18):
- Added `build_secrets_provision_dag(...)` in
  `lib/cloud-ops/src/secret_provision_graph.rs`, composing one upsert sub-DAG
  per active secret from the canonical project spec.

**Ref**: `gunb.ai/tools/infra/dag/builder.go:1630-1680` — secret provisioning

### 3.3 Secret Fetch & Export

**Gap**: No local secret-loading mechanism (direnv, shell exports).

- [ ] Add `secrets fetch` command that reads secrets and emits `export VAR=value` lines
- [ ] Add direnv integration (`.envrc` hook + cache file)
- [ ] Add TTL-based caching to avoid re-fetching on every shell prompt
- [ ] Support partial fetch (only missing env vars)

Update (2026-02-18):
- Added `render_direnv_exports(...)` in `lib/cloud-ops/src/secret_exports.rs`
  to render shell export lines from fetched prefixed secret values with
  namespace-aware env-name mapping and shell escaping.

**Ref**: `gunb.ai/scripts/secrets/direnv_lib.sh`, `gunb.ai/tools/secrets/cmd/secrets/fetch.go`

---

## Phase 4: Environment Modeling (MEDIUM)

gunb.ai has 6 environments with distinct configs. gunbc only has dev and ci.

### 4.1 Environment Config Struct

**Gap**: `NamespaceSpec` is minimal — no region, zone, domain, contact email.

- [x] Expand `NamespaceSpec` or create `EnvironmentConfig`:
  - `project: &str` — GCP project ID (may differ from secrets project)
  - `project_number: &str`
  - `region: &str` — e.g., `us-central1`
  - `zone: &str` — e.g., `us-central1-a`
  - `domain: Option<&str>` — e.g., `dev.gunb.ai`
  - `name_prefix: &str` — resource name prefix
  - `secrets_project: &str` — may be separate project
  - `secrets_prefix: &str` — e.g., `dev-`
- [x] Derive all resource names from config (SA emails, secret IDs, etc.)
- [x] Add tests for name derivation across environments

**Ref**: `gunb.ai/tools/infra/spec/spec.go:66-92` — `EnvironmentConfig`

### 4.2 Additional Environments

**Gap**: gunbc only has dev and ci.

- [x] Add `test` namespace/environment (separate from dev)
- [x] Add `prod` namespace/environment (no prefix)
- [x] Design strategy: same project or multi-project?
- [x] Update `GUNBAI_SECRETS` spec with new namespaces
- [x] Ensure `to_cloud_secret_config()` works for all environments

Update (2026-02-18):
- Current strategy remains **same-project-by-default** (`gunbai-secrets`) but
  per-namespace fields now model explicit environment project/secrets project
  values so future multi-project splits are data-only changes.

**Ref**: `gunb.ai/tools/infra/spec/spec.go:624-670` — environment definitions

---

## Phase 5: Infrastructure Spec & Plan/Apply (MEDIUM-LOW)

gunb.ai has a full `InfraSpec` with plan/apply semantics. gunbc only has
upsert patterns for individual resources.

### 5.1 InfraSpec Type

**Gap**: No unified infrastructure spec — individual resources are modeled
separately.

- [x] Create `InfraSpec` struct that aggregates all resource specs:
  ```rust
  pub struct InfraSpec {
      pub environment: &'static str,
      pub config: EnvironmentConfig,
      pub service_accounts: &'static [ServiceAccountSpec],
      pub secrets: &'static [SecretSpec],
      pub wif: WifConfig,
      // Future: buckets, cloud_run, compute, etc.
  }
  ```
- [x] Create `DEV_SPEC`, `CI_SPEC`, etc. as compile-time constants
- [x] Add `InfraSpec::validate()` for cross-resource consistency checks

Update (2026-02-18):
- Added `lib/cloud-ops/src/infra_spec.rs`:
  - `EnvironmentConfig`
  - `InfraSpec`
  - static specs: `DEV_SPEC`, `CI_SPEC`, `TEST_SPEC`, `PROD_SPEC`
  - `InfraSpec::validate()` consistency checks + tests.

**Ref**: `gunb.ai/tools/infra/spec/spec.go:568-600` — `InfraSpec`

### 5.2 Plan/Apply DAG Builder

**Gap**: No plan/apply workflow — only individual upsert patterns.

- [ ] Create `build_infra_plan_dag(spec: &InfraSpec)` that:
  1. Discovers current state (SAs, secrets, IAM bindings)
  2. Compares against spec
  3. Outputs planned changes (create, update, delete)
- [ ] Create `build_infra_apply_dag(spec: &InfraSpec)` that:
  1. Runs plan
  2. Executes changes with proper dependency ordering
  3. Reports results
- [ ] Add CLI entrypoints: `make infra-plan`, `make infra-apply`
- [ ] Add `--target=ID` and `--skip=ID` filtering

**Ref**: `gunb.ai/tools/infra/cmd/infra/main.go` — plan/apply commands

### 5.3 Infrastructure Graph Visualization

**Gap**: No DOT graph output for infrastructure dependency visualization.

- [ ] Add `infra graph --env=dev` command that outputs DOT format
- [x] Visualize resource dependencies (SA → IAM → secret → compute)
- [ ] Integrate with existing DAG rendering (`PlainStructuredRenderer`)

Update (2026-02-18):
- Added `render_infra_spec_dot(&InfraSpec)` in `lib/cloud-ops/src/infra_graph.rs`
  to emit DOT with environment, WIF, service-account, secret, and rotation
  dependency edges.

**Ref**: `gunb.ai/tools/infra/cmd/infra/main.go` — `graph` subcommand

---

## Phase 6: Compute Stack (LOW — future)

gunb.ai models a full compute stack. gunbc doesn't need all of this yet
but should have the service interfaces ready.

### 6.1 Compute Engine Service Interface

- [ ] Add `ComputeService` trait:
  - `list_instance_templates(project)`
  - `create_instance_template(project, spec)`
  - `delete_instance_template(project, name)`
  - `list_instance_groups(project, zone)`
  - `create_managed_instance_group(project, zone, spec)`
  - `update_managed_instance_group(project, zone, name, spec)`
- [ ] Add REST implementation using `compute.googleapis.com/v1`

**Ref**: `gunb.ai/tools/infra/gcloud/admin.go` — compute operations

### 6.2 Cloud Run Service Interface

- [ ] Add `CloudRunService` trait:
  - `list_services(project, region)`
  - `create_service(project, region, spec)`
  - `update_service(project, region, name, spec)`
  - `delete_service(project, region, name)`
- [ ] Add REST implementation using `run.googleapis.com/v2`

**Ref**: `gunb.ai/tools/infra/spec/spec.go:338-368` — `CloudRunServiceSpec`

### 6.3 Load Balancer / Network

- [ ] Health checks service interface
- [ ] Backend services service interface
- [ ] URL maps service interface
- [ ] Forwarding rules service interface
- [ ] SSL certificate management

**Ref**: `gunb.ai/tools/infra/spec/spec.go:132-328`

### 6.4 GCS Bucket CRUD

**Gap**: gunbc has list/get/get_iam_policy for buckets but no create/update.

- [ ] Add `create_bucket(project, name, config)` to `StorageService`
- [ ] Add `update_bucket(name, config)` to `StorageService`
- [ ] Add `set_bucket_iam_policy(bucket, policy)` to `StorageService`
- [ ] Add `BucketConfig` struct (location, storage_class, versioning, access control)

**Ref**: `gunb.ai/tools/infra/spec/spec.go:390-418` — `GCSBucketSpec`

---

## Phase 7: CLI & Developer Experience (MEDIUM)

### 7.1 Unified Infra CLI

- [ ] Create `gunbc-infra` binary with subcommands:
  - `bootstrap` — initial project setup
  - `plan --env=ENV` — show planned changes
  - `apply --env=ENV` — apply changes
  - `spec --env=ENV` — show current spec
  - `graph --env=ENV` — DOT dependency graph
- [ ] Add Makefile targets: `infra-plan`, `infra-apply`, `infra-bootstrap`

### 7.2 Enhanced Login Flow

**Gap**: `make login` only does `gcloud auth login --update-adc`.

- [ ] Expand `make login` to also:
  - Verify ADC file exists and is valid
  - Check SA impersonation works
  - Verify secret access works
  - Report which secrets are accessible
  - Set up direnv if available

**Ref**: `gunb.ai/tools/secrets/cmd/secrets/login.go`

### 7.3 Status / Health Check

- [ ] Add `make infra-status` that:
  - Checks auth status (ADC valid? token fresh?)
  - Lists accessible projects
  - Verifies SA impersonation
  - Reports secret access status
  - Checks IAM bindings

---

## Phase 8: Multi-Project Support (LOW)

### 8.1 Project Registry

**Gap**: gunbc only models one project (`gunbai-secrets`). gunb.ai uses
3 projects (auto, secrets, CI).

- [ ] Create `ProjectRegistry` with multiple `ProjectSpec` entries
- [ ] Support cross-project references (e.g., SA in project A accesses secrets in project B)
- [ ] Add project-level IAM management for cross-project access

### 8.2 Compute Project

- [ ] Define compute project spec (VMs, load balancers)
- [ ] Wire compute project SAs to secrets project access
- [ ] Model cross-project WIF bindings

---

## Priority Summary

| Phase | Priority | Blocks | Est. Effort |
|-------|----------|--------|-------------|
| 1. SA & IAM Lifecycle | HIGH | Phase 2, 5 | 2-3 days |
| 2. WIF Bootstrap | HIGH | CI setup | 2-3 days |
| 3. Secret Full Lifecycle | MEDIUM | Dev UX | 2-3 days |
| 4. Environment Modeling | MEDIUM | Multi-env | 1-2 days |
| 5. InfraSpec + Plan/Apply | MEDIUM-LOW | Operations | 3-5 days |
| 6. Compute Stack | LOW | Production | 5+ days |
| 7. CLI & Dev UX | MEDIUM | Developer flow | 2-3 days |
| 8. Multi-Project | LOW | Scale | 2-3 days |

**Recommended path**: Phase 1 → 2 → 7.2 → 3.2 → 4 → 5 → rest

This gets SA management + WIF bootstrap + enhanced login working first,
then expands to full secret provisioning, environment modeling, and
plan/apply semantics.
