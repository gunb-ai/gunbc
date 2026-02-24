# DSL Golden Examples

Standalone `.dag` files that define what the gunbc DSL looks like for every real workflow. These are **spec examples** (golden targets for the parser), not runnable code -- the compiler doesn't exist yet.

## Purpose

1. **Parser golden targets** -- the first thing the compiler must parse
2. **Parity test anchors** -- each `.dag` file maps 1:1 to a Rust graph builder
3. **Modeling validation** -- prove the DSL syntax handles all real patterns

## Reading order

Start with the foundation, then work outward:

```
std/          Foundation: types, resources, patterns
services/     Pure declarations: typed I/O + transport annotations
cloud/        Credential acquisition per provider (GCP, AWS, Azure)
infra/        Multi-cloud infrastructure (abstract interfaces + provider impls)
  core.dag    Abstract interfaces: ObjectStorage, Compute, SecretStore, Identity, Queue
  spec.dag    Provider-neutral spec types
  gcp/        GCP provider: services, resources, config
  aws/        AWS provider: services, resources, config
  azure/      Azure provider: services, resources, config
shared/       Composition helpers + utility library
tools/        Funcs that compose everything above
pipelines/    Multi-stage composition of tools
examples/     Forward-looking proposals (services, types, deployment, test generation)
```

### Recommended first read

1. `std/types.dag` -- see all the types
2. `std/resources.dag` -- Filesystem, Network, Clock, AuthContext
3. `std/patterns.dag` -- content_upsert, credential_chain (the reusable shapes)
4. `shared/dag_util.dag` -- utility helpers (aggregate, report, stage construction)
5. `tools/makegen.dag` -- simplest tool (~30 lines, uses content_upsert)
6. `tools/gist.dag` -- complex tool (4 modes, loops, service composition)
7. `infra/core.dag` -- abstract interfaces (ObjectStorage, Compute, SecretStore, Identity, Queue)
8. `infra/spec.dag` -- provider-neutral specs (reconciliation, fingerprinting, cost estimation)
9. `infra/gcp/resources.dag` -- GCP resources implementing abstract interfaces
10. `infra/aws/resources.dag` -- AWS resources implementing abstract interfaces
11. `infra/azure/resources.dag` -- Azure resources implementing abstract interfaces
12. `examples/deployment.dag` -- multi-cloud composition, provider selection, cross-provider
13. `examples/integration_tests.dag` -- six-tier test generation model
14. `pipelines/ci.dag` -- everything wired together

## Syntax overview

Three levels of "function" in the DSL, all using C-style signatures:

```
// Pure function (no I/O, no side effects)
fn render(items: List<Item>) -> String { ... }

// Effectful function (can call services, use resources)
func gist_upload(md: String, branch: String) -> { url: String }
  uses net: Network
  uses auth: AuthContext
{ ... }

// Reusable DAG template (compile-time expansion)
pattern content_upsert(content: String, path: String) -> { written: Bool }
  uses fs: Filesystem(mode: ReadWrite)
{ ... }
```

## Rust mapping

Each `.dag` file replaces a specific Rust graph builder:

| DSL file | Rust file | Rust LOC |
|---|---|---|
| `tools/makegen.dag` | `gunbc-dag/src/makegen/graph.rs` | 220 |
| `tools/gist.dag` | `lib/tools/gist/src/graph.rs` | 1,449 |
| `tools/dag_viz.dag` | `gunbc-dag/src/dag_viz/graph.rs` | 1,347 |
| `tools/clippy.dag` | `lib/tools/clippy/src/graph.rs` | 186 |
| `tools/deps.dag` | `lib/tools/deps/src/graph.rs` | ~200 |
| `tools/bootstrap.dag` | `gunbc-dag/src/bootstrap/graph.rs` | ~300 |
| `tools/codegen.dag` | `gunbc-dag/src/codegen/graph.rs` | ~200 |
| `tools/testgen.dag` | `gunbc-dag/src/testgen_dag/graph.rs` | ~200 |
| `tools/pragma.dag` | `gunbc-dag/src/pragma/graph.rs` | ~300 |
| `tools/build.dag` | `gunbc-dag/src/build/graph.rs` | ~250 |
| `tools/docgen.dag` | `gunbc-dag/src/docgen/graph.rs` | ~500 |
| `cloud/gcp/credential.dag` | `lib/gcp-ops/src/graph.rs` | 1,700+ |
| `pipelines/ci.dag` | `gunbc-dag/src/ci/graph.rs` | ~600 |

**Total: ~7,500 lines of Rust graph builders replaced by ~700 lines of DSL.**

## File inventory

### `std/` -- Standard library

- **`types.dag`** -- Primitives, refinement types, domain enums, records. Everything from `Unit` to `DagTopology`.
- **`resources.dag`** -- `Filesystem`, `Network`, `Clock`, `AuthContext`. Resource lifecycle declarations with capabilities.
- **`patterns.dag`** -- `content_upsert`, `upsert`, `emit`, `credential_chain`, `transaction`, `retry`. C-style signatures: `pattern name(params) -> { outputs } { body }`.

### `services/` -- Service declarations

- **`git.dag`** -- `git.Core`: CurrentBranch, RemoteBranches, LsFiles, Diff, RevList, Show
- **`cargo.dag`** -- `cargo.Build`: Build, Test, Clippy, Doc, Run
- **`shell.dag`** -- `gcloud.Auth`, `oauth2.Google`, `shell.Find`, `shell.Codegen`, `rustup.Component`, `shell.Which`
- **`github/gist.dag`** -- `github.Gist`: Create (REST API + mock_response)
- **`gcp/secret_manager.dag`** -- `gcp.SecretManager`: AccessVersion, CreateSecret, AddVersion
- **`gcp/iam.dag`** -- `gcp.IAM`: GenerateAccessToken; `gcp.ResourceManager`: Get/SetIamPolicy
- **`gcp/sts.dag`** -- `gcp.STS`: Exchange; `github.OIDC`: GetToken; `gcp.Metadata`: GetIdentityToken

### `cloud/` -- Cloud provider credential acquisition

- **`gcp/credential.dag`** -- `acquire_gcp_secret` func. Wraps `credential_chain` pattern for GCP WIF -> STS -> impersonation.
- **`aws/credential.dag`** -- `acquire_aws_credentials`, `acquire_aws_secret`. AWS OIDC -> STS AssumeRoleWithWebIdentity -> session credentials.
- **`azure/credential.dag`** -- `acquire_azure_credentials`, `acquire_azure_secret`. Azure federated identity -> AD token exchange.

### `infra/` -- Multi-cloud infrastructure as resources

- **`core.dag`** -- Abstract infrastructure interfaces: `ObjectStorage`, `Compute`, `SecretStore`, `Identity`, `Queue<T>`. Provider-neutral capabilities with `@contract` behavioral annotations. Also defines `CloudConfig` (sum type: GcpConfig | AwsConfig | AzureConfig) and `InfraEnvironment`.
- **`spec.dag`** -- Provider-neutral spec types: `ReconcileAction`/`ReconcileResult`, `ResourceFingerprint`, `SecretSpec`, `ComputeSpec`, `StorageSpec`, `IdentitySpec`, `QueueSpec`, `InfraSpec`. No provider-specific types.
- **`gcp/services.dag`** -- GCP API service declarations for infra lifecycle: `gcp.Storage`, `gcp.CloudRun`, `gcp.SecretManager.Lifecycle`, `gcp.IAM.Lifecycle`, `gcp.WIF`, `gcp.PubSub`.
- **`gcp/resources.dag`** -- GCP resources implementing abstract interfaces: `GcsBucket : ObjectStorage`, `CloudRunService : Compute`, `ManagedSecret : SecretStore`, `GcpServiceAccount : Identity`, `WifProvider` (GCP-specific), `PubSubTopic : Queue<Bytes>`.
- **`gcp/config.dag`** -- GCP environment configs (`dev_config`, `ci_config`, `prod_config`), GCP-specific spec types (`GcpWifSpec`, `GcpServiceAccountSpec`).
- **`aws/services.dag`** -- AWS API service declarations: `aws.S3`, `aws.Lambda`, `aws.SecretsManager`, `aws.IAM`, `aws.STS`, `aws.SQS`.
- **`aws/resources.dag`** -- AWS resources implementing abstract interfaces: `S3Bucket : ObjectStorage`, `LambdaFunction : Compute`, `AwsSecret : SecretStore`, `AwsIamRole : Identity`, `SqsQueue : Queue<String>`.
- **`aws/config.dag`** -- AWS environment configs (`dev_config`, `ci_config`, `prod_config`).
- **`azure/services.dag`** -- Azure API service declarations: `azure.BlobStorage`, `azure.ContainerApps`, `azure.KeyVault`, `azure.ManagedIdentity`, `azure.ServiceBus`, `azure.Authorization`.
- **`azure/resources.dag`** -- Azure resources implementing abstract interfaces: `BlobContainer : ObjectStorage`, `ContainerApp : Compute`, `KeyVaultSecret : SecretStore`, `AzureManagedIdentity : Identity`, `ServiceBusQueue : Queue<String>`.
- **`azure/config.dag`** -- Azure environment configs (`dev_config`, `ci_config`, `prod_config`).

### `shared/` -- Composition helpers

- **`dag_util.dag`** -- Common utility functions: `aggregate_results`, `all_succeeded`, `format_report`, `stage_result`, `stage_from_output`, `generated_header`. Extracted from duplicated logic across tool files.
- **`gist_modes.dag`** -- `branch_context`, `resolve_recent_base`, `share_content`. The shared scaffolding for the snapshot/diff/recent mode pattern. Both `gist.dag` and `dag_viz.dag` import from here.

### `tools/` -- Tool funcs

- **`makegen.dag`** -- Simplest: `render_makefile` fn + `content_upsert` pattern. Uses `generated_header` from dag_util.
- **`gist.dag`** -- 4 modes: `gist_upload`, `gist_snapshot`, `gist_diff`, `gist_recent`. Composes `shared/gist_modes.dag` for mode scaffolding.
- **`dag_viz.dag`** -- 4 modes: `dag_viz_snapshot`, `dag_viz_diff`, `dag_viz_recent`, `dag_viz_save`. Same composition pattern as gist.
- **`clippy.dag`** -- `upsert` pattern (check/install/resolve) + lint run
- **`deps.dag`** -- Dependency install (platform-aware loop) + config generation. Uses `generated_header` from dag_util.
- **`bootstrap.dag`** -- Workspace scan + 2 parallel content_upserts (Makefile, .gitignore). Uses `generated_header`.
- **`codegen.dag`** -- Conditional execution: check stamp, run if stale, write stamp
- **`testgen.dag`** -- Dynamic parallel: N targets, each with independent content_upsert
- **`pragma.dag`** -- 3 parallel content_upserts (clippy.toml, allowlist, policy). Uses `generated_header`.
- **`build.dag`** -- `cargo build` then parallel `cargo test` + `cargo clippy`. Uses `aggregate_results` and `stage_from_output` from dag_util.
- **`docgen.dag`** -- Read 13 source files, render single doc, content_upsert

### `pipelines/` -- Multi-stage pipelines

- **`ci.dag`** -- 12-stage CI pipeline composing all tools with `after` dependencies, parallel groups, conditional execution (`when`), and aggregate reporting. Uses `aggregate_results`, `format_report`, `stage_result`, `stage_from_output` from dag_util.

### `examples/` -- Forward-looking proposals

These explore language extensions not yet finalized in the spec. Each file is self-contained with design rationale and open questions.

- **`abstract_services.dag`** -- Three-layer service abstraction: `interface` (abstract contract with `@contract` behavioral specs), `service X : Interface` (concrete implementation), and funcs written against interfaces. Shows Storage, LLM, Queue interfaces with GCS, S3, OpenAI, VertexAI implementations.
- **`rich_types.dag`** -- Generic types (`Result<T, E>`, `Page<T>`), sum types with payloads (`DeployEvent`), type bounds/interfaces (`Serializable`, `Comparable`), branded types (`UserId`, `TeamId`), and bounded generics. Includes phasing recommendation (Phase 1-4).
- **`deployment.dag`** -- Multi-cloud infrastructure composition. Shows abstract interface usage, provider-specific resources, cross-provider composition (GCP + AWS), provider selection via config, GCP/AWS full bootstrap, and multi-cloud integration test generation.
- **`integration_tests.dag`** -- Six-tier test generation for infra-backed funcs: hermetic unit, node contracts, scenario coverage (guard paths), resource hygiene, live integration (real GCP), and end-to-end. Shows how `@mock_response` in resource acquire blocks + probe-observer model generates all test tiers automatically.

## Multi-cloud infrastructure

The key architectural decision: cloud infrastructure is modeled as first-class `resource` declarations where `acquire` IS the ensure/upsert pattern. Abstract interfaces define capabilities; provider-specific resources implement them.

```
// Abstract interface (infra/core.dag)
interface ObjectStorage {
  capability read(key: NonEmptyStr) -> { content: Bytes, found: Bool }
  capability write(key: NonEmptyStr, content: Bytes) -> { ok: Bool }
  @contract: read(k) after write(k, v) => { content: v, found: true }
}

// GCP implementation (infra/gcp/resources.dag)
resource GcsBucket implements ObjectStorage {
  config { name: NonEmptyStr, project: ProjectId }
  acquire { check -> create -> resolve }  // GCP-specific ensure DAG
}

// AWS implementation (infra/aws/resources.dag)
resource S3Bucket implements ObjectStorage {
  config { name: NonEmptyStr, region: String }
  acquire { HeadBucket -> CreateBucket }   // AWS-specific ensure DAG
}

// Business logic targets the INTERFACE
func store_artifact(key: NonEmptyStr, content: Bytes) -> { ok: Bool }
  uses store: ObjectStorage
{
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}
```

Provider selection happens at compile time via `CloudConfig` (sum type: GcpConfig | AwsConfig | AzureConfig). The compiler inserts the correct acquire DAG, threads resource handles, detects conflicts, and generates integration tests from `@mock_response` annotations.

Five abstract interfaces, three providers, 15 concrete resources:

| Interface | GCP | AWS | Azure |
|---|---|---|---|
| ObjectStorage | GcsBucket | S3Bucket | BlobContainer |
| Compute | CloudRunService | LambdaFunction | ContainerApp |
| SecretStore | ManagedSecret | AwsSecret | KeyVaultSecret |
| Identity | GcpServiceAccount | AwsIamRole | AzureManagedIdentity |
| Queue | PubSubTopic | SqsQueue | ServiceBusQueue |

See `infra/core.dag` for the interface definitions, `infra/{gcp,aws,azure}/resources.dag` for implementations, and `examples/deployment.dag` for multi-cloud composition.

## Collection operations as IR nodes

A key design property: collection operations (`map`, `filter`, `fold`, `join`, etc.) inside `fn` bodies are **not** compiled as opaque function calls. The compiler lowers them to IR-level collection nodes (`MapNode`, `FilterNode`, `FoldNode`, `JoinNode`, etc.) whose inner transforms are scalar functions.

This means every program is a complete dataflow graph -- nothing is hidden. The executor can parallelize `MapNode` across workers, stream `MapNode -> FilterNode` without materializing intermediates, and fuse trivial adjacent maps into single passes.

Two kinds of parallelism are visible in the IR:
- **Task-parallel**: func-level `for` loops (each iteration has I/O)
- **Data-parallel**: `fn`-level `|> map/filter/fold` (each element is a pure transform)

See `dsl-design.md` section 4.2 for the full two-tier model.

## Design doc

These files are the concrete instantiation of the language spec in `docs/design/v4/dsl-design.md`. The spec defines the grammar; these files prove it works for every real workflow.
