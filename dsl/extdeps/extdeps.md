# External Dependency Fidelity Invariant

Every `extdeps/` module models a real external system. The model must be grounded
in the actual API specification — not an idea of the spec, but the spec itself.

## The Invariant

**An extdeps module implements a specification, not an abstraction of one.**

This means:

1. **Cite the source.** Every module header must link to the upstream API
   documentation it models. If you can't link to a spec, you're inventing one.

2. **Use real names.** Field names, enum values, error codes, and endpoint paths
   must match the upstream API. Don't rename `html_url` to `url` or `APPROVED`
   to `Approved` because it looks nicer.

3. **Declare the version.** If the API has versioning (date-based, semver, path-
   based), the module must state which version it targets.

4. **Model what exists.** Types should describe the actual API response/request
   shapes. Don't add fields the API doesn't return. Don't omit fields it does.

5. **Operations match endpoints.** Service operations must correspond to real
   HTTP methods + paths with correct request/response types.

## Compositional Modeling: From TCP to GitHub Gists

The power of domain-driven extdeps is **compositional separation of concerns**.
Each layer defines "what X is" without leaking how it's used. A higher layer
imports from a lower layer, never the reverse. The system already demonstrates
this pattern — here's how it works, traced from the bottom up.

### Layer 0: Universal Primitives

These modules define vocabulary that applies everywhere — no external system
knowledge, just math and structure.

```
std/types.dag        "What is a refined type? A branded type? A sum type?"
std/coordination.dag "What is a CAS mechanism? A lease? A delivery guarantee?"
shared/behavioral.dag "What is a side effect? Determinism? A failure mode?"
std/rate_limit.dag   "What is a rate limit? A backoff strategy? A retry trigger?"
std/errors.dag       "What are provider error envelope shapes?"
std/fermi.dag        "What is an order of magnitude?"
```

Example — `std/coordination.dag` defines CAS without knowing GCS exists:

```dag
type CasMechanism = GenerationBased | ETagBased | VersionId | RowVersion
```

Example — `std/errors.dag` defines provider error envelope shapes without knowing individual operations:

```dag
type GitHubErrorShape {
  message: String
  documentation_url: String?
}
```

Example — `std/rate_limit.dag` defines retry semantics without knowing any API:

```dag
type BackoffStrategy
  = Exponential { base_ms: Milliseconds, max_ms: Milliseconds }
  | Linear { step_ms: Milliseconds }
  | JitteredExponential { base_ms: Milliseconds, max_ms: Milliseconds }

type RetryPolicy {
  max_attempts: Int
  backoff: BackoffStrategy
  retry_on: List<RetryTrigger>
}
```

**Key property**: nothing here references any external system. These are
tautological definitions — "a CAS mechanism is one of: generation-based,
ETag-based, version ID, or row version." True by definition.

### Layer 1: Cloud Transport Abstractions

These modules define what cloud APIs share — authentication schemes, endpoints,
credentials — without naming a specific provider.

```
extdeps/cloud/cloud.dag  "What is a cloud provider?"
```

```dag
// How cloud APIs authenticate requests. Each provider uses a subset.
type CloudAuthScheme
  = BearerToken
  | SigV4 { region: String, service: String }
  | ApiKey
  | OidcToken { audience: String }

type ServiceEndpoint {
  base_url: String
  version: String
  regional: Bool
}

type RateLimitPolicy {
  requests_per_minute: Int
  burst: Int?
  scope: String
}
```

This layer composes Layer 0: `RateLimitPolicy` is a simpler projection of
`std/rate_limit.dag`'s `RateLimit` — cloud-specific but not provider-specific.

### Layer 2: Provider Core

Each provider's interface module instantiates Layer 1 with real values from the
actual API documentation.

```
extdeps/cloud/gcp/gcp.dag     "What is GCP?" (OAuth2, scopes, endpoints)
extdeps/cloud/aws/aws.dag     "What is AWS?" (ARN, SigV4, regions)
extdeps/cloud/azure/azure.dag "What is Azure?" (tenants, subscriptions)
```

GCP example (`cloud/gcp/gcp.dag`) — real OAuth2 scopes, real endpoint URLs, real ADC path:

```dag
// Spec: https://cloud.google.com/iam/docs/reference/rest
// Spec: https://cloud.google.com/storage/docs/json_api/v1

data gcp_scopes: List<GcpScope> = [
  { scope: "https://www.googleapis.com/auth/cloud-platform", label: "Full access" },
  { scope: "https://www.googleapis.com/auth/iam", label: "IAM management" }
]

data gcp_api_endpoints: Map<String, String> = {
  storage: "https://storage.googleapis.com/storage/v1",
  iam: "https://iam.googleapis.com/v1",
  secret_manager: "https://secretmanager.googleapis.com/v1",
  cloud_run: "https://run.googleapis.com/v2"
}
```

AWS example (`cloud/aws/aws.dag`) — real ARN structure, real SigV4 parameters:

```dag
// Spec: https://docs.aws.amazon.com/IAM/latest/UserGuide/reference-arns.html

type AwsArn {
  partition: AwsPartition
  service: String
  region: String
  account_id: String
  resource: String
}

type AwsPartition = AwsStandard | GovCloud | AwsChina
```

**Key property**: these are facts about the provider, sourced from its
documentation. `"https://storage.googleapis.com/storage/v1"` is not an
abstraction — it's the actual endpoint URL from the GCP docs.

**Naming convention**: each folder's interface module matches the folder name
(`secrets/secrets.dag`, `cloud/cloud.dag`, `cloud/gcp/gcp.dag`). This
eliminates ambiguous `core.dag` files and makes the module path tautological:
`extdeps.cloud.gcp.gcp` is "the GCP module in cloud/gcp".

### Layer 3: Provider Services

Individual API services compose Layer 2 (provider core) with shared
(behavioral vocabulary) to model specific endpoints.

```
extdeps/cloud/gcp/secret_manager.dag  "What is GCP Secret Manager?"
extdeps/cloud/gcp/iam.dag             "What is GCP IAM?"
extdeps/cloud/aws/s3.dag              "What is S3?"
extdeps/github/gists.dag              "What is the GitHub Gists API?"
```

Secret Manager example — types match the real API, operations match real
endpoints, behaviors composed from `shared/behavioral.dag`:

```dag
// Spec: https://cloud.google.com/secret-manager/docs/reference/rest
// API version: v1

// Types match API response schema
type SmVersionState = SmEnabled | SmDisabled | SmDestroyed

type GcpSecretVersion {
  name: String         // "projects/*/secrets/*/versions/*"
  state: SmVersionState
  create_time: String  // RFC 3339
}

// Operations match real REST endpoints
service gcp.SecretManager {
  AccessVersion {
    transport: rest
    method: GET
    path: "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access"
    auth: BearerToken
    input: { project_id: String, secret: String, version: String }
    output: { payload: SecretPayload }
    error: GcpErrorShape
  }
}

// Behaviors composed from shared/behavioral.dag vocabulary
data access_version_behavior: OperationBehavior = {
  side_effects: ReadOnly,
  idempotent: true,
  determinism: Deterministic,
  confidence: Documented,
  // ...
}
```

### Layer 4: Cross-Provider Abstractions

Some concerns span providers. These compose Layer 3 services without
duplicating provider-specific details.

```
extdeps/secrets/secrets.dag            "What is a secret?" (universal)
extdeps/secrets/gcp_secret_manager.dag  GCP implementation of secret operations
extdeps/secrets/vault.dag              HashiCorp Vault implementation
```

The universal model:

```dag
type SecretLifecycle = Active | Disabled | Destroyed | PendingDeletion

type SecretVersion {
  id: String
  state: SecretLifecycle
  created_at: String
}
```

Each provider module maps its API to this vocabulary. GCP's `SmEnabled` maps
to `Active`. Vault's lease-based lifecycle maps to the same states. The
universal model is not invented — it's the intersection of what all providers
actually provide.

### Layer 5: Tool Integration

Tool modules compose Layer 3/4 services into workflows. They never define
types that belong in lower layers.

```
gunbc/tools/gist.dag  Composes github/gists + github/auth + git + shell
tools/bootstrap.dag  Composes gitignore + make + shell
```

```dag
module gunbc.tools.gist

import extdeps.github.gists { Gist, GistFile }
import extdeps.github.auth { github_token }
import extdeps.git { ... }

func gist(branch: String?, public: Bool?) -> { url: String }
  uses fs: Filesystem(mode: ReadWrite)
  uses net: Network(mode: ReadWrite)
{
  token = github_token()
  // ... compose services ...
}
```

### The Layer Diagram

```
Layer 5  gunbc/tools/gist.dag ─── "Do the thing"
           │  imports
Layer 4  secrets/secrets.dag ────────── "What is a secret?" (universal)
           │  imports
Layer 3  cloud/gcp/secret_manager ──── "What is GCP Secret Manager?" (spec)
         github/gists.dag ──────────── "What is the Gists API?" (spec)
           │  imports
Layer 2  cloud/gcp/gcp.dag ──────────── "What is GCP?" (provider facts)
         github/github.dag ──────────── "What is GitHub?" (provider facts)
           │  imports
Layer 1  cloud/cloud.dag ──────────── "What is a cloud provider?" (abstract)
           │  imports
Layer 0  std/errors.dag ──────────── "What is an HTTP error?"
         shared/behavioral.dag ──── "What is idempotency?"
         std/coordination.dag ────── "What is CAS?"
         std/rate_limit.dag ──────── "What is a retry policy?"
```

**Each layer only knows about layers below it.** `std/errors.dag` doesn't know
GitHub exists. `cloud/cloud.dag` doesn't know GCP exists. `cloud/gcp/gcp.dag`
doesn't know Secret Manager exists. This is separation of concerns through
composition, not abstraction.

### Existing Examples of Pristine Composition

**fermi → fidelity → test_policy** (the magnitude chain):

```
std/fermi.dag           Layer 0: FermiDepth ordinal + comparison + max
    ↓ imports
std/fidelity.dag        Layer 1: TransportClass → FermiDepth mapping
    ↓ imports                    classify_transports() composes fermi_max_of()
config/test_policy.dag  Layer 2: Repo-specific budget gate
                                 classify_from_facts() consumes fidelity
```

Each module is independently testable. `fermi.dag` knows nothing about
transports. `fidelity.dag` knows nothing about repo policy. The composition
is causal: transport cost → magnitude → budget check.

**behavioral → cloud services → coordination providers**:

```
shared/behavioral.dag           Shared layer: OperationBehavior schema
    ↓ imported by
cloud/gcp/secret_manager.dag   Layer 3: access_version_behavior data
coordination/gcs.dag            Layer 3: gcs_put_behavior data
    ↓ imported by
coordination/*.dag              Layer 4: wires behaviors to resource interfaces
```

Every service declares its behavior using the same vocabulary. A new service
doesn't need new behavior types — it instantiates existing ones with real
values from the API spec.

**errors → provider error shapes → service operations**:

```
std/errors.dag                  Layer 0: GitHubErrorShape, GcpErrorShape, etc.
    ↓ imported by
cloud/gcp/secret_manager.dag   Layer 3: error: GcpErrorShape in service ops
github/gists.dag                Layer 3: error: GitHubErrorShape in service ops
```

Error shapes are defined once per provider in `std/errors.dag` (sourced from
the actual API error envelope documentation), then referenced by every service
operation. When GitHub changes its error format, one type changes.

### Why This Matters

When we add a new external dependency — say, Stripe — we don't invent types.
We read the Stripe API documentation and transcribe:

1. **Layer 0**: `RetryPolicy` and `OperationBehavior` already exist.
   Stripe-specific error envelopes should be transcribed from Stripe docs, not
   squeezed through a generic HTTP error shape.

2. **Layer 2**: `extdeps/stripe/core.dag` — cite https://docs.stripe.com/api,
   declare API version `2024-12-18.acacia`, define `StripeAuthScheme = ApiKey`.

3. **Layer 3**: `extdeps/stripe/charges.dag` — model `POST /v1/charges` with
   real request/response types from the docs. Compose `OperationBehavior`
   with Stripe-specific idempotency key semantics.

4. **Layer 5**: `tools/billing.dag` — import and compose.

No new abstractions needed. The compositional model absorbs new dependencies
by instantiation, not by modification.

## Three-Layer Versioning Model

Adopted from `gunb.ai/tools/extdeps/`:

```
Layer 1: Upstream API version
  The version published by the provider.
  Examples: GitHub REST API 2022-11-28, GCP Secret Manager v1,
            Anthropic API 2023-06-01

Layer 2: Our extdeps snapshot
  The .dag module — our locked understanding of the upstream contract.
  Git-versioned. Contains: which API version + endpoints + behaviors
  we depend on.

Layer 3: Code binding
  Rust resolve/extern layer that wires DSL types to runtime operations.
  Adapts based on extdeps configuration.
```

When an upstream API version changes, the extdeps module is the single place
to update. Downstream consumers (tools, workflows) see the change automatically.

## Module Header Template

Every extdeps module should follow this pattern:

```dag
// extdeps/github/gists.dag -- GitHub Gists API
//
// Spec: https://docs.github.com/en/rest/gists/gists
// API version: 2022-11-28 (X-GitHub-Api-Version header)
// Endpoints modeled:
//   POST /gists                    (Create)
//   GET  /gists/{gist_id}          (Get)
//   PATCH /gists/{gist_id}         (Update)
//   DELETE /gists/{gist_id}        (Delete)
//
// Fields and types match the API response schema documented at the
// spec URL above. Deviations are noted inline with // DEVIATION: comments.

module extdeps.github.gists
```

For non-REST specs (file formats, CLI tools, protocols):

```dag
// extdeps/build/make.dag -- GNU Make
//
// Spec: https://www.gnu.org/software/make/manual/make.html
// Version: GNU Make 4.x
// Sections referenced:
//   §5.2   Recipe Echoing (@ prefix)
//   §5.5   Errors in Recipes (- prefix)
//   §5.7.1 Force Targets (+ prefix)
```

## Fidelity Grading

Each module is graded on how well it implements the upstream spec:

| Grade | Meaning | Criteria |
|-------|---------|----------|
| **A** | Spec-faithful | Cites spec URL. Field names match API. Version declared. Operations match endpoints. |
| **B** | Mostly right | Types are accurate but spec URL missing, or minor field gaps. |
| **C** | Idea of the spec | Describes what the API conceptually does without grounding in the actual contract. Made-up field names, missing versioning. |
| **D** | Wrong or dead | Models a different system than claimed, or has no consumers. |

## Current Grades

### Cloud Providers

| Module | Lines | Grade | Spec URL | Notes |
|--------|-------|-------|----------|-------|
| `cloud/cloud.dag` | 67 | A | — | Abstract vocabulary, not a specific API |
| `cloud/gcp/gcp.dag` | 196 | A | Inline endpoints | OAuth2 scopes, API base URLs, ADC path all spec-correct |
| `cloud/gcp/iam.dag` | 224 | A- | Inline endpoints | GenerateAccessToken path correct; missing GenerateIdToken |
| `cloud/gcp/secret_manager.dag` | 211 | A- | Inline endpoint | AccessVersion/CreateSecret/AddVersion correct; missing List |
| `cloud/gcp/sts.dag` | 180 | A- | Inline endpoint | RFC 8693 grant types correct |
| `cloud/gcp/storage.dag` | 188 | B+ | **Missing** | CAS/generation model correct; no REST ops wired |
| `cloud/gcp/cloud_run.dag` | 165 | B | **Missing** | Types good; no REST ops wired to transport |
| `cloud/gcp/pubsub.dag` | 132 | B | **Missing** | Behavioral data good; no REST ops wired |
| `cloud/aws/aws.dag` | 79 | A | — | ARN, SigV4, STS all match real spec |
| `cloud/aws/iam.dag` | 117 | A | **Missing** | Trust policy, principal types, managed policies all correct |
| `cloud/aws/lambda.dag` | 104 | A | **Missing** | Runtimes, limits (10240 MB, 900s) match real API |
| `cloud/aws/s3.dag` | 126 | A | **Missing** | ETag CAS, storage classes, versioning all correct |
| `cloud/aws/secrets_manager.dag` | 110 | A | **Missing** | Version stages, rotation config, 64KB limit correct |
| `cloud/aws/sqs.dag` | 122 | A | **Missing** | FIFO semantics, dedup, DLQ all correct |
| `cloud/azure/azure.dag` | 78 | A | Inline endpoints | Subscription states, identity model correct |
| `cloud/azure/identity.dag` | 116 | A | **Missing** | RBAC scopes, built-in roles correct |
| `cloud/azure/key_vault.dag` | 108 | A | **Missing** | Key types, soft delete, 25KB limit correct |
| `cloud/azure/blob_storage.dag` | 117 | A | **Missing** | Blob types, tiers, ETag preconditions correct |
| `cloud/azure/container_apps.dag` | 127 | B+ | **Missing** | Dapr, scaling rules good; minor gaps |
| `cloud/azure/service_bus.dag` | 135 | A | **Missing** | Session, peek-lock, DLQ all correct |

### GitHub

| Module | Lines | Grade | Spec URL | Notes |
|--------|-------|-------|----------|-------|
| `github/github.dag` | 70 | B | **Missing** | No API version header. Pagination oversimplified |
| `github/auth.dag` | 38 | C | **Missing** | Not GitHub auth — GCP credential pipeline |
| `github/gists.dag` | 146 | B | Mock only | `files` field wrong type (List vs Map). Missing GET/DELETE |
| `github/issues.dag` | 445 | B- | Mock only | Event types incomplete. `user` fields are String not object |
| `github/pull_requests.dag` | 459 | B- | Mock only | Missing `merged_at`. Review state case wrong |

### LLM

| Module | Lines | Grade | Spec URL | Notes |
|--------|-------|-------|----------|-------|
| `llm/llm.dag` | 48 | A | — | Universal message protocol, provider-agnostic |
| `llm/anthropic.dag` | 166 | A | Inline endpoint | API version 2023-06-01 correct. Model IDs current |
| `llm/openai.dag` | 281 | B+ | Inline endpoint | Contains fictional future models (o4-mini, gpt-5.2) |
| `llm/auth.dag` | 28 | B | — | GCP-specific credential bridge, not universal |
| `llm/pricing.dag` | 181 | B | **Missing** | Stale model names (claude-sonnet-4-5 should be 4-6) |

### Secrets

| Module | Lines | Grade | Spec URL | Notes |
|--------|-------|-------|----------|-------|
| `secrets/secrets.dag` | 95 | A | — | Universal vocabulary |
| `secrets/env_file.dag` | 129 | A | — | .env format rules accurate |
| `secrets/gcp_secret_manager.dag` | 192 | A | **Missing** | Operations, status codes, rate limits all correct |
| `secrets/github_secrets.dag` | 166 | A | **Missing** | libsodium encryption, write-only semantics correct |
| `secrets/vault.dag` | 196 | A | **Missing** | KV2 CAS, lease lifecycle, auth methods correct |

### Tools and Formats

| Module | Lines | Grade | Spec URL | Notes |
|--------|-------|-------|----------|-------|
| `git.dag` | 261 | A | **Missing** | Object model, merge strategies, diff format correct |
| `shell.dag` | 54 | A | **Missing** | POSIX find/which/printenv correct |
| `cron_schedule_model.dag` | 81 | A | Module header | POSIX.1-2017 XCU `crontab` five-field grammar; spec URL + edition in header |
| `cargo.dag` | 216 | A | **Missing** | Package, target, profile, feature model correct |
| `build/make.dag` | 127 | A | Cited | GNU Make manual URL. Recipe prefixes §5.2/5.5/5.7.1 |
| `yaml.dag` | 92 | A | **Missing** | Indent/kv/list/comment rules correct |
| `tools/gh_cli.dag` | 127 | B+ | **Missing** | CLI wrapper model, not REST API |
| `tools/package_managers.dag` | 160 | A | **Missing** | Platform matrix, install semantics correct |
| `tools/rust_toolchain.dag` | 146 | A | **Missing** | Channels, components, editions, targets correct |

### Rendering (internal, not external specs)

| Module | Lines | Grade | Notes |
|--------|-------|-------|-------|
| `build/render.dag` | 246 | A | Internal — shared build target rendering |
| `build/targets.dag` | 79 | A | Internal — shared schema types |
| `build/make_render.dag` | 51 | A | Internal — Make-specific rendering |
| `build/justfile_render.dag` | 34 | A | Internal — Just-specific rendering |
| `gitignore_render.dag` | 38 | A | Internal — gitignore rendering |
| `ci/github_actions.dag` | 186 | B | Schema correct but no spec URL |
| `ci/github_actions_render.dag` | 109 | A | Internal — YAML rendering |
| `ci/gitlab_ci.dag` | 64 | B | Schema correct but no spec URL |
| `ci/gitlab_ci_render.dag` | 53 | A | Internal — YAML rendering |
| `ci/script.dag` | 42 | A | Internal — generic script abstraction |
| `clippy.dag` | 69 | A | Internal — clippy domain model |

### Other

| Module | Lines | Grade | Notes |
|--------|-------|-------|-------|
| `gunbc.dag` | 151 | A | Self-referential — models this system |
| `coordination/coordination.dag` | 64 | A | Abstract vocabulary |
| `coordination/gcs.dag` | 145 | B+ | CAS model correct; no GCS spec URL |
| `coordination/postgres.dag` | 145 | B | Behavioral correct; no Postgres spec URL |
| `coordination/sqlite.dag` | 105 | B | Behavioral correct; no SQLite spec URL |
| `devenv/devcontainers.dag` | 73 | B | Schema correct; no devcontainer spec URL |
| `api/gcp_ops.dag` | 127 | C+ | Fictional retry triggers (PermissionNotYetPropagated) |
| `api/github_ops.dag` | 127 | B | Operational constraints; no spec URLs |

## Aggregate

```
Total extdeps modules: 79
  Grade A:   42  (53%)
  Grade B+:   9  (11%)
  Grade B:   13  (16%)
  Grade B-:   2   (3%)
  Grade C+:   1   (1%)
  Grade C:    1   (1%)
  Ungraded:  11  (14%)  (rendering internals)

Modules citing spec URLs:  ~8 of 79 (10%)
Modules with inline endpoint URLs: ~12 of 79 (15%)
Modules with no external reference: ~59 of 79 (75%)
```

## Priority Fixes

### P0: Add spec URLs to all A-grade modules

These modules are already accurate — they just don't cite their source.
Adding a header comment with the spec URL is trivial and massively increases
auditability. Follow the gunb.ai pattern: every module header lists the
upstream documentation it implements.

Targets (28 modules):
- `cloud/aws/*` — cite https://docs.aws.amazon.com/ per service
- `cloud/azure/*` — cite https://learn.microsoft.com/en-us/rest/api/ per service
- `secrets/*` — cite provider-specific docs
- `git.dag` — cite https://git-scm.com/docs or Pro Git
- `cargo.dag` — cite https://doc.rust-lang.org/cargo/reference/
- `shell.dag` — cite POSIX or coreutils docs
- `yaml.dag` — cite https://yaml.org/spec/1.2.2/
- `tools/rust_toolchain.dag` — cite https://rust-lang.github.io/rustup/

### P1: Fix GitHub modules (B/B- → A)

The GitHub modules describe the API conceptually but don't implement the spec:
- Add `X-GitHub-Api-Version: 2022-11-28` to all service configs
- Add `Accept: application/vnd.github+json` header
- Fix `files` field in gists (Map, not List)
- Fix `user` fields (GitHubUser object, not String)
- Add missing operations (GET/DELETE for gists, LIST/UPDATE for PRs)
- Add `merged_at`, `merged_by` to PullRequest
- Fix Review state enum case (APPROVED not Approved)

### P2: Fix data staleness

- `llm/pricing.dag` — reconcile model names with anthropic.dag (4-5 → 4-6)
- `llm/openai.dag` — remove fictional models (o4-mini, gpt-5.2, gpt-5.1-codex)
- `api/gcp_ops.dag` — replace fictional retry triggers with real HTTP status codes

### P3: Wire missing GCP REST operations

Cloud Run, Storage, Pub/Sub have excellent behavioral data but no transport
wiring. These are B-grade because the types are right but the operations
aren't connected to real endpoints.

## Design Influence

The gunb.ai extdeps system (proto-based contracts with machine-readable
documentation, three-layer versioning, capability-based contracts, drift
detection) is the gold standard. This DSL system achieves the same goals
through a different mechanism:

| Concern | gunb.ai | gunbc |
|---------|---------|-------|
| Contract format | Protobuf messages | DAG type declarations |
| Spec citation | Proto comment blocks with URLs | Module header comments with URLs |
| Versioning | Contract version constants | Module header version declaration |
| Capabilities | GCloudCapability + GCloudPrerequisite | OperationBehavior data |
| Drift detection | JSON snapshot comparison | Obligation contract tests |
| Code binding | Go resolver functions | Rust extern_ops + resolve |

The substrate differs. The invariant is the same: **model the spec, not an
idea of the spec.**
