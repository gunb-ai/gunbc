# GCP Service Modeling: Spec, Bringup, Secrets, and Generated Tests

**Status**: Draft
**Date**: 2026-02-13
**Related**:
- `TODO/TODO_gcp_infra_parity.md` — phased checklist for infra porting
- `TODO/TODO_credential_lifecycle.md` — auth pattern architecture
- `docs/design/integration-testgen.md` — test tier taxonomy

## Motivation

gunbc models GCP services as low-level REST trait interfaces (`SecretManagerService`,
`IamService`, `WorkloadIdentityService`, `StorageService`, `ResourceManagerService`)
with DAG-based execution. This works for individual operations but lacks the
declarative spec layer that gunb.ai uses to make infrastructure reproducible,
testable, and self-documenting.

Three capabilities are needed:

1. **Service modeling** — port gunb.ai's spec-driven infrastructure (including
   bringup/bootstrap) so that `make infra-apply` provisions a full environment
   from typed Rust specs.

2. **Canonical secrets** — elevate secrets from ad-hoc DAG parameters to a
   first-class spec with rotation policy, environment prefixing, IAM bindings,
   and lifecycle status.

3. **Generated integration tests** — extend the probe-observer testgen system
   to cover GCP service graphs, including live tests gated by Fermi cost and
   secret availability.

This document is the unified design for all three.

---

## Part 1: Service Modeling

### What gunb.ai does

gunb.ai defines infrastructure as typed Go structs (`InfraSpec`) containing
every resource for an environment. Resources are reconciled via a DAG of
`Operation` nodes, each with `Check()` (idempotent state query) and `Execute()`
(mutating apply) functions. Key patterns:

- **Hardcoded specs** — all config is in Go code, not YAML/TOML files.
  `DevConfig()`, `TestConfig()`, `ProdConfig()` return typed structs.
- **Environment transformation** — `TestSpec()` starts from `DevSpec()` and
  renames resources (`dev-*` -> `test-*`).
- **Reconciliation** — check-then-apply with `ReconcileResult` tracking
  `Created | Updated | Unchanged | Deleted`.
- **Preflight** — validate IAM permissions before any mutation.
- **Wave parallelism** — compute topological waves, execute each wave in parallel.
- **Source resolution** — `bazel://`, `file://`, `registry://` schemes resolved
  before apply, injected into metadata.
- **Resource fingerprinting** — `managed-by: gunb-infra` and `spec-hash` labels
  for drift detection.

### What gunbc has today

- 5 GCP service traits (REST request builders with `MethodMeta`).
- `GcpOps` enum with prepare/execute/parse node types.
- Upsert graph for Secret Manager (check -> create -> add_version).
- Discovery graph for introspecting live GCP state.
- `ProjectSpec` with `NamespaceSpec` (dev, ci) and `ServiceAccountSpec`.
- IAM ensure pattern (auto-grant secretAccessor).
- No bootstrap. No plan/apply. No reconciliation. No fingerprinting.

### Design: `InfraSpec` in Rust

Port the spec-driven approach using compile-time constants.

```rust
// core/ir/src/infra/spec.rs

pub struct InfraSpec {
    pub environment: Environment,
    pub config: EnvironmentConfig,
    pub service_accounts: &'static [ServiceAccountSpec],
    pub secrets: &'static [SecretSpec],
    pub wif: WifSpec,
    pub iam_bindings: &'static [IamBindingSpec],
    // Future: buckets, compute, cloud_run
}

pub enum Environment {
    Dev,
    Ci,
    Test,
    Prod,
}

pub struct EnvironmentConfig {
    pub project: &'static str,
    pub project_number: &'static str,
    pub region: &'static str,
    pub zone: &'static str,
    pub secrets_project: &'static str,
    pub secrets_prefix: &'static str,
}

pub struct ServiceAccountSpec {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub project: Option<&'static str>,  // override config.project
    pub roles: &'static [&'static str],
    pub self_roles: &'static [&'static str],
    pub service_account_users: &'static [&'static str],
    pub wif_bindings: &'static [WifBindingSpec],
}

pub struct WifSpec {
    pub pool_id: &'static str,
    pub provider_id: &'static str,
    pub issuer_uri: &'static str,
    pub attribute_mapping: &'static [(&'static str, &'static str)],
    pub attribute_condition: Option<&'static str>,
}

pub struct WifBindingSpec {
    pub pool_id: &'static str,
    pub provider_id: &'static str,
    pub attribute: &'static str,
}

pub struct IamBindingSpec {
    pub resource: IamResource,
    pub role: &'static str,
    pub member: &'static str,
}

pub enum IamResource {
    Project,
    ServiceAccount(&'static str),
    Secret(&'static str),
    Bucket(&'static str),
}
```

Instances are compile-time constants:

```rust
pub static DEV_SPEC: InfraSpec = InfraSpec {
    environment: Environment::Dev,
    config: EnvironmentConfig {
        project: "gunbai-auto",
        project_number: "314501921854",
        region: "us-central1",
        zone: "us-central1-a",
        secrets_project: "gunbai-secrets",
        secrets_prefix: "dev-",
    },
    service_accounts: &[
        ServiceAccountSpec {
            name: "gunbai-dev-secrets",
            display_name: "Dev Secrets Accessor",
            // ...
        },
    ],
    secrets: &KNOWN_SECRETS,
    wif: WifSpec { /* ... */ },
    iam_bindings: &[],
};
```

### Design: Reconciliation

Each resource type gets a reconcile function that returns a structured result:

```rust
pub enum ReconcileAction {
    Created,
    Updated { changed_fields: Vec<String> },
    Unchanged,
    Deleted,
    Skipped { reason: String },
}

pub struct ReconcileResult {
    pub resource_kind: &'static str,
    pub resource_name: String,
    pub action: ReconcileAction,
}
```

The DAG builder generates check-then-apply node pairs per resource:

```
check_sa_deployer -> [skip if unchanged] -> apply_sa_deployer
check_sa_deployer -> [skip if unchanged] -> apply_sa_deployer_iam
```

### Design: Fingerprinting (Drift Detection)

Add `managed-by` and `spec-hash` labels to all created resources:

```rust
pub trait Fingerprinted {
    fn spec_hash(&self) -> String;
    fn labels(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("managed-by".into(), "gunbc".into());
        m.insert("spec-hash".into(), self.spec_hash());
        m
    }
}
```

Reconcile compares `spec_hash` on existing resources. If hash matches,
`Unchanged`. If hash differs, `Updated`. If no `managed-by` label, warn
(externally created resource).

### Design: Bootstrap DAG

Bootstrap handles first-time project setup. It is a separate DAG from
plan/apply because it creates prerequisites that other DAGs depend on
(WIF pool, initial SAs).

```
enable_apis
  -> create_wif_pool
  -> create_wif_provider
  -> create_sa(deployer)
  -> grant_project_roles(deployer)
  -> grant_wif_binding(deployer)
  -> create_sa(dev-secrets)
  -> grant_project_roles(dev-secrets)
  -> create_sa(ci-secrets)
  -> grant_project_roles(ci-secrets)
  -> grant_wif_binding(ci-secrets)
```

CLI entry: `make bootstrap` or `gunbc-infra bootstrap --project=X --github-repo=Y`

Output: prints the WIF provider path and SA emails needed for GitHub Actions
secrets configuration.

### Design: Plan/Apply CLI

```
gunbc-infra plan --env=dev     # check all resources, print planned changes
gunbc-infra apply --env=dev    # execute planned changes
gunbc-infra apply --env=dev --target=sa:deployer  # apply single resource
```

Plan output shows reconcile status per resource:

```
Environment: dev (gunbai-auto)

  [CREATE]    sa:gunbai-deployer
  [UNCHANGED] sa:gunbai-dev-secrets
  [UPDATE]    secret:dev-github-token  (changed: iam_bindings)
  [UNCHANGED] wif:github-pool/github

  3 unchanged, 1 create, 1 update, 0 delete
```

---

## Part 2: Canonical Secrets

### What gunb.ai does

Secrets are a first-class spec (`tools/secrets/spec/spec.go`) with:

- `EnvName` — canonical environment variable name
- `SecretID` — base Secret Manager ID (prefixed per environment)
- `Requirement` — Required or Optional
- `Status` — Active or Deleted (deleted kept for cleanup/pruning)
- `RotationHandler` — manual, github-pat, sa-key, none
- `MaxAgeDays` — rotation reminder interval
- `Scopes` — OAuth/API scopes (for PATs)
- Helper functions: `ActiveSecrets()`, `DevCISecrets()`, `RuntimeSecrets()`

Secrets are provisioned via a DAG that creates Secret Manager resources,
grants IAM access to appropriate SAs, and optionally uploads values from
environment variables.

### What gunbc has today

- `SecretSpec` in `ProjectSpec` with `name`, `env_name`, `rotation`, and
  per-namespace `NamespaceSecretSpec` (prefix, delimiter, version_selector).
- `CredentialPolicySpec` with profile-based intent-to-secret mapping.
- Secret Manager upsert graph (check -> create -> add_version).
- No canonical secret catalog. No rotation runtime. No fetch/export.
  No provisioning DAG that iterates the full catalog.

### Design: Canonical Secret Catalog

Expand `SecretSpec` to be the single source of truth:

```rust
pub struct SecretSpec {
    /// Canonical environment variable name (e.g., "GITHUB_TOKEN")
    pub env_name: &'static str,

    /// Base Secret Manager ID (e.g., "github-token")
    pub secret_id: &'static str,

    /// Required or Optional
    pub requirement: SecretRequirement,

    /// Active or Deleted
    pub status: SecretStatus,

    /// How this secret is rotated
    pub rotation: RotationHandler,

    /// Recommended max age before rotation
    pub max_age_days: Option<u32>,

    /// OAuth/API scopes (for PATs and tokens)
    pub scopes: &'static [&'static str],

    /// Human-readable description of what this secret is for
    pub description: &'static str,

    /// Which service accounts should have secretAccessor on this secret
    pub accessor_sas: &'static [&'static str],
}

pub enum SecretRequirement {
    Required,
    Optional,
}

pub enum SecretStatus {
    Active,
    Deleted,
}

pub enum RotationHandler {
    Manual,
    GitHubPat,
    ServiceAccountKey,
    None,
}
```

The canonical catalog:

```rust
pub static KNOWN_SECRETS: &[SecretSpec] = &[
    SecretSpec {
        env_name: "OPENAI_API_KEY",
        secret_id: "openai-api-key",
        requirement: SecretRequirement::Required,
        status: SecretStatus::Active,
        rotation: RotationHandler::Manual,
        max_age_days: None,
        scopes: &[],
        description: "OpenAI API key for LLM operations",
        accessor_sas: &["gunbai-dev-secrets", "gunbai-ci-secrets"],
    },
    SecretSpec {
        env_name: "GITHUB_TOKEN",
        secret_id: "github-token",
        requirement: SecretRequirement::Optional,
        status: SecretStatus::Active,
        rotation: RotationHandler::GitHubPat,
        max_age_days: Some(90),
        scopes: &["repo", "gist", "read:org", "workflow"],
        description: "GitHub PAT for gist/review/CI operations",
        accessor_sas: &["gunbai-dev-secrets", "gunbai-ci-secrets"],
    },
    // ...
];
```

### Design: Secret Provisioning DAG

A generated DAG that provisions all active secrets for an environment:

```
For each secret in KNOWN_SECRETS where status == Active:
  compose_name(prefix + secret_id)
    -> check_exists(project, composed_name)
    -> [create if missing](project, composed_name)
    -> ensure_iam(project, composed_name, accessor_sas)
```

All secrets are checked in parallel (wave 1), then created in parallel
(wave 2), then IAM bindings in parallel (wave 3).

CLI: `make ensure-secrets --env=dev`

### Design: Secret Fetch and Export

Port gunb.ai's parallel fetch with shell-safe export:

```
gunbc-secrets fetch --env=dev                  # fetch all dev secrets
gunbc-secrets fetch --env=dev --runtime        # minimal runtime subset
gunbc-secrets fetch --env=dev --format=shell   # export VAR="value" lines
gunbc-secrets fetch --env=dev --format=github  # GITHUB_ENV heredoc format
```

Implementation: a DAG that fetches secrets in parallel (one Secret Manager
access call per secret), collects results, and formats output.

Fetch uses the existing credential chain (`resolve_auth -> cloud_credential`)
to authenticate, then issues parallel `access_secret_version` calls.

### Design: Connecting Secrets to Credential Policy

The `CredentialPolicySpec` (from `TODO_credential_lifecycle.md`) binds
intents to secrets. The canonical catalog provides the source of truth
for what secrets exist and how they're named:

```
CredentialIntent("github.gist.create")
  -> CredentialPolicy resolves to SecretBinding { name: "github-token" }
  -> SecretSpec lookup: KNOWN_SECRETS.find(|s| s.secret_id == "github-token")
  -> Environment prefix: "dev-" + "github-token" = "dev-github-token"
  -> Secret Manager access: projects/gunbai-secrets/secrets/dev-github-token
```

The policy spec references secrets by `secret_id`; the catalog provides
metadata (scopes, rotation, requirement). This separation means:
- Adding a new secret = add to catalog + add IAM binding
- Using a secret in a new workflow = add intent binding to policy
- Rotating a secret = catalog says how, policy says where it's used

---

## Part 3: Generated Integration Tests

### How testgen works today

gunbc's testgen uses a **probe-observer model**:

- **Probes** — nodes with concrete injectable inputs (from `MockSpec`
  transport_mocks, boundary_mocks, input_mocks).
- **Observers** — nodes with developer-specified `OutputMatcher`s that
  are input-independent (chain-safe): `NonEmpty`, `IsString`, `IntGe(n)`.
- **Anti-tautology** — tests are only generated when matchers prove
  something non-trivial. `OutputMatcher::Any` is flagged as weak.

Intermediate observers serve dual roles: verify upstream chain, then
become probes for downstream tests. This gives compositional coverage:
if A->C and C->E both pass, A->E works by transitivity.

Test classes (`Unit`, `Hermetic`, `Integration`) and Fermi costs
(`XS` through `XL`) gate execution. Integration tests require
`GUNBC_TEST_MAX_COST >= M` and all declared secrets to be present.

### Design: MockSpec for GCP Service Graphs

Each GCP service graph needs a `MockSpec` that defines probes and observers
for both DryRun (hermetic) and live (integration) testing.

#### Secret Manager upsert graph

```rust
pub fn secret_upsert_mock_spec() -> MockSpec {
    MockSpec::new("gcp-secret-upsert")
        // Probes: entry points with concrete values
        .input_mock("compose_secret_name", "prefix", Value::Str("dev-".into()))
        .input_mock("compose_secret_name", "secret_id", Value::Str("test-secret".into()))
        .input_mock("prepare_secret_add_version", "secret_value",
                    Value::Secret(SecretString::new("mock-value")))

        // Transport mocks: REST responses
        .transport_mock("execute_secret_get", "response",
            Value::Response(TransportResponse::Rest(
                RestResponse::not_found())))
        .transport_mock("execute_secret_create", "response",
            Value::Response(TransportResponse::Rest(
                RestResponse::ok(json!({"name": "projects/p/secrets/dev-test-secret"})))))
        .transport_mock("execute_add_version", "response",
            Value::Response(TransportResponse::Rest(
                RestResponse::ok(json!({"name": "projects/p/secrets/dev-test-secret/versions/1"})))))

        // Observers: non-tautological output matchers
        .node_example(NodeExample::new("parse_secret_get")
            .output("exists", OutputMatcher::Exact(Value::Bool(false))))
        .node_example(NodeExample::new("parse_add_version")
            .output("version", OutputMatcher::NonEmpty))

        // Skip utility nodes from example enforcement
        .skip_node_example("compose_secret_name")
        .skip_node_example("prepare_secret_get")
        .skip_node_example("prepare_secret_create")
        .skip_node_example("prepare_secret_add_version")
}
```

This generates:
- Hermetic chain tests: `execute_secret_get -> parse_secret_get` (validates
  404 -> exists=false), `execute_add_version -> parse_add_version` (validates
  version output).
- Intermediate observer promotion: `parse_secret_get` becomes a probe for
  the conditional create path.

#### Bootstrap graph

```rust
pub fn bootstrap_mock_spec() -> MockSpec {
    MockSpec::new("gcp-bootstrap")
        // WIF pool creation
        .transport_mock("execute_create_pool", "response",
            Value::Response(TransportResponse::Rest(
                RestResponse::ok(json!({"name": "projects/p/locations/global/workloadIdentityPools/github-pool"})))))

        // SA creation
        .transport_mock("execute_create_sa", "response",
            Value::Response(TransportResponse::Rest(
                RestResponse::ok(json!({"email": "deployer@p.iam.gserviceaccount.com"})))))

        // Observers
        .node_example(NodeExample::new("parse_create_pool")
            .output("pool_name", OutputMatcher::Contains("github-pool")))
        .node_example(NodeExample::new("parse_create_sa")
            .output("email", OutputMatcher::Contains("deployer@")))
}
```

### Design: Live Integration Tests for GCP

Live tests use `TestClass::Integration` with `FermiCost::M` and require
real GCP credentials. They execute the DAG in `Real` mode against actual
Secret Manager / IAM APIs.

#### Gating

```rust
TestConfig {
    live_flow_tests: true,
    live_test_class: TestClass::Integration,
    live_fermi_cost: FermiCost::M,
    live_secrets: vec![
        "GCP_WIF_PROVIDER".into(),
        "GCP_SECRETS_PROJECT".into(),
    ],
}
```

The `guard()` function checks:
1. `GUNBC_TEST_MAX_COST >= M` (skips locally unless opted in)
2. All `live_secrets` are present as environment variables
3. If any check fails, test is skipped (not failed)

#### Live MockSpec

Live tests use `live_expected_outputs` instead of transport mocks:

```rust
MockSpec::new("gcp-secret-access-live")
    // No transport mocks — real HTTP
    // Input mocks still needed for DAG entry points
    .input_mock("compose_secret_name", "prefix",
        Value::Str(std::env::var("GCP_SECRETS_PREFIX").unwrap_or("dev-".into()).into()))
    .input_mock("compose_secret_name", "secret_id",
        Value::Str("openai-api-key".into()))

    // Live observers: verify real responses
    .live_expected_output("parse_secret_access", "secret_value",
        OutputMatcher::NonEmpty)
    .live_expected_output("build_credential", "credential",
        OutputMatcher::IsCredential)
```

#### What live tests prove

| Test | What it proves | Fermi cost |
|------|---------------|------------|
| Secret access (read) | WIF auth chain works end-to-end, secret exists and is readable | M |
| Secret upsert (write) | SA has secretmanager.secrets.create, upsert is idempotent | M |
| SA creation | SA CRUD works, IAM binding succeeds | L |
| Bootstrap | Full WIF + SA + IAM bringup from scratch (test project) | L |

### Design: Test Coverage Matrix for GCP Graphs

Each GCP graph should have coverage at three levels:

| Level | Mode | Class | Cost | What it tests |
|-------|------|-------|------|---------------|
| Bucket A | DryRun | Unit | XS | DAG structure (completion, interception) |
| Bucket B+C | DryRun + MockSpec | Hermetic | S | Node contracts, scenario paths |
| Probe-observer | DryRun + MockSpec | Hermetic | S | Non-tautological chain assertions |
| Live flow | Real | Integration | M-L | End-to-end against real GCP APIs |

Coverage enforcement: the probe-observer analysis enforces that every
terminal node reachable from a probe has an `OutputMatcher`. If not,
testgen fails with an actionable error listing uncovered terminals.

### Design: Test Infrastructure for New Service Graphs

When adding a new GCP service (e.g., Compute Engine), the workflow is:

1. **Define service trait** — `ComputeService` with method signatures + `MethodMeta`.
2. **Add ops** — `GcpOps::PrepareCreateInstance`, etc.
3. **Build graph** — `build_compute_instance_graph()`.
4. **Write MockSpec** — probes at transport boundaries, observers at parse nodes.
5. **Add `#[testgen_target]`** — registers for generated test emission.
6. **Add live test config** — `live_flow_tests: true` with required secrets.
7. **Run `make testgen`** — generates all test tiers automatically.

The generated tests automatically cover:
- DryRun completion (smoke)
- Transport interception (all REST calls mockable)
- Probe-observer chains (non-tautological assertions)
- Scenario coverage (success + per-node failure paths)
- Live integration (when secrets available)

---

## Part 4: Implementation Roadmap

### Phase 0: Canonical Secret Catalog (1-2 days)

Prerequisite for everything else. Establishes the source of truth.

- [ ] Create `core/ir/src/infra/mod.rs` module
- [ ] Define `SecretSpec`, `SecretRequirement`, `SecretStatus`, `RotationHandler`
- [ ] Define `KNOWN_SECRETS` catalog (port from gunb.ai `spec/spec.go`)
- [ ] Add helper methods: `active_secrets()`, `by_env_name()`, `by_secret_id()`
- [ ] Add `prefixed_name(prefix: &str) -> String` method
- [ ] Wire `SecretSpec` into existing `ProjectSpec` (replace inline definitions)
- [ ] Unit tests for catalog invariants (no duplicate env_names, no duplicate secret_ids)

### Phase 1: Secret Provisioning DAG + MockSpec (2-3 days)

Generates a DAG that ensures all secrets exist with proper IAM.

- [ ] Build `build_secrets_provision_dag(spec: &InfraSpec)` in `lib/gcp-ops/src/graph.rs`
- [ ] For each active secret: compose_name -> check_exists -> create_if_missing -> ensure_iam
- [ ] Write `MockSpec` with probes at transport boundaries
- [ ] Add observers at parse nodes (`exists`, `created`, `iam_bound`)
- [ ] Add `#[testgen_target]` registration
- [ ] Generated hermetic tests via `make testgen`
- [ ] Add live test config gated on `GCP_WIF_PROVIDER` + `GCP_SECRETS_PROJECT`
- [ ] CLI entrypoint: `make ensure-secrets`

### Phase 2: InfraSpec + Environment Configs (1-2 days)

- [ ] Define `InfraSpec`, `EnvironmentConfig`, `Environment` enum
- [ ] Define `ServiceAccountSpec` (full: display_name, self_roles, wif_bindings)
- [ ] Define `WifSpec`, `WifBindingSpec`, `IamBindingSpec`
- [ ] Create `DEV_SPEC`, `CI_SPEC` compile-time constants
- [ ] Add `InfraSpec::validate()` for cross-resource consistency
- [ ] Add derivation methods (SA emails, WIF member strings, secret full names)
- [ ] Unit tests for derivation across environments

### Phase 3: SA CRUD + IAM Service Expansion (2-3 days)

- [ ] Add `create_service_account`, `update_service_account`, `delete_service_account` to `IamService`
- [ ] Add SA-level IAM: `get_sa_iam_policy`, `set_sa_iam_policy`
- [ ] Add `MethodMeta` for each new method
- [ ] Add `GcpOps` variants: `PrepareCreateSa`, `ParseCreateSa`, etc.
- [ ] Build reconcile graph for SA lifecycle
- [ ] MockSpec with transport mocks for SA create/update responses
- [ ] Generated tests

### Phase 4: WIF Bootstrap DAG (2-3 days)

- [ ] Add `create_pool`, `create_provider`, `update_provider` to `WorkloadIdentityService`
- [ ] Build `build_bootstrap_dag(spec: &InfraSpec)` with full dependency chain:
  enable_apis -> create_wif_pool -> create_wif_provider -> create_sas -> grant_roles -> grant_wif_bindings
- [ ] MockSpec with probes at each transport boundary
- [ ] Observers validating pool name, provider config, SA emails, IAM status
- [ ] Generated hermetic tests
- [ ] CLI: `make bootstrap --project=X --github-repo=Y`
- [ ] Live test gated on admin-level GCP permissions (FermiCost::L)

### Phase 5: Plan/Apply + Reconciliation (3-4 days)

- [ ] Implement `ReconcileAction` enum and `ReconcileResult`
- [ ] Implement `Fingerprinted` trait with `spec_hash()` and `labels()`
- [ ] Build `build_infra_plan_dag(spec)` — check all resources, report status
- [ ] Build `build_infra_apply_dag(spec)` — execute reconciliation
- [ ] Add `--target` and `--skip` filtering
- [ ] Plan output formatting (CREATE/UPDATE/UNCHANGED/DELETE summary)
- [ ] CLI: `make infra-plan --env=dev`, `make infra-apply --env=dev`
- [ ] MockSpec covering create, update, and unchanged paths
- [ ] Generated tests including scenario coverage for mixed states

### Phase 6: Secret Fetch + Export (1-2 days)

- [ ] Build `build_secrets_fetch_dag(secrets: &[SecretSpec])` — parallel access
- [ ] Output formatters: shell export, GitHub Actions heredoc, JSON
- [ ] TTL caching for repeated fetches
- [ ] CLI: `make fetch-secrets --env=dev`
- [ ] Live integration test (fetch known secret, assert NonEmpty)

### Phase 7: Rotation (2 days, lower priority)

- [ ] `check_secret_age` op — compare version create_time against max_age_days
- [ ] Manual rotation: print instructions per handler type
- [ ] GitHub PAT rotation: browser OAuth flow (if applicable)
- [ ] SA key rotation: create key -> store -> delete old
- [ ] CLI: `make rotate-secret SECRET=github-token --env=dev`

---

## Open Decisions

1. **Spec location** — `core/ir/src/infra/` (near transport/credential types)
   vs `lib/gcp-ops/src/spec/` (near GCP implementations). Recommendation:
   `core/ir` because specs are provider-neutral (even if only GCP today).

2. **Test project isolation** — live integration tests need a GCP project.
   Options: use existing `gunbai-secrets` with test-prefixed resources, or
   create a dedicated `gunbc-test` project. Recommendation: test-prefixed
   resources in existing project (cheaper, simpler).

3. **Reconciliation depth** — full drift detection (compare all fields) vs
   hash-only (compare spec-hash label). Recommendation: hash-only initially,
   add field-level diff later if needed.

4. **Bootstrap idempotency** — should `make bootstrap` be safe to re-run?
   Yes, every operation uses check-before-create. Re-running on an already
   bootstrapped project should report all `Unchanged`.

5. **Secret catalog ownership** — should `KNOWN_SECRETS` live in `core/ir`
   (shared) or in the project spec (per-deployment)? Recommendation: the
   catalog of *what secrets exist* lives in `core/ir`; the *per-environment
   binding* (prefix, SA access) lives in the project spec.
