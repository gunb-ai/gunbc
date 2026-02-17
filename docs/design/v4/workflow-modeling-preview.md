# Workflow Modeling Preview

**Status**: Complete (Phase 0.5 deliverable)
**Companion**: [`dsl-roadmap.md`](./dsl-roadmap.md)

This document is the Phase 0.5 modeling preview required by the DSL roadmap. For each
workflow targeted in Phases 1-4, it captures:

1. **Builder shape** — the hand-wired Rust graph (current state)
2. **DSL shape** — the compiled `.dag` output (target state)
3. **1:1 mappings** — nodes that correspond directly
4. **Compiler insertions** — nodes the compiler adds that don't appear in the `.dag` source
5. **Manifest differences** — progress model divergences
6. **Status and remaining gaps**

The source of truth for compile/parity status is the workflow fixture contract suite in
`daglang-cli/tests/workflow_contracts.rs`.

---

## Global Status

- All `.dag` files typecheck under whole-root compilation (`daglang obligations dsl --format json`).
- Deterministic parity scaffolds lower dependency-closure module scopes, so parity reports
  carry meaningful structural deltas for all proving workflows.
- Four exact-match normalized parity gates pass: makegen, GCP credential, gist (3 modes), CI.

---

## S1 — Makegen (`tools/makegen.dag`) — Phase 1 Proving Workflow

### DSL source (5 lines of logic)

```
func makegen(registry: ToolRegistry) -> { written: Bool } {
  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")
  return { written: result.written }
}
```

### Builder shape (from `gunbc-dag/src/makegen/graph.rs`, ~137 lines)

```
load_registry ──→ render_makefile ──→ prepare_read_makegen ──→ execute_read_makegen
                        │                                              │
                        │                                              ▼
                        ├──→ prepare_write_makegen ←── compare_makegen_content
                        │              │
                        ▼              ▼
                    (content)    execute_makegen_transport
                                       ▲
                                       │
fs_env ────────────────────────────────┘
```

8 nodes, 10 edges. Pattern: `content_upsert` (prepare_read → execute_read → compare → prepare_write → execute_write).

### Compiled DSL shape (from `makegen_canonical_ir.json`)

9 nodes, 11 edges. Identical topology to the builder after normalization:

| DSL node | Builder node | Match |
|---|---|---|
| `tools.makegen::render_makefile` | `render_makefile` | 1:1 (callable) |
| `tools.makegen::makegen` | — | DSL wrapper (stripped in parity) |
| `load_registry` | `load_registry` | 1:1 (pattern-expanded) |
| `fs_env` | `fs_env` | 1:1 (pattern-expanded) |
| `prepare_read_makegen` | `prepare_read_makegen` | 1:1 (pattern-expanded) |
| `execute_read_makegen` | `execute_read_makegen` | 1:1 (transport) |
| `compare_makegen_content` | `compare_makegen_content` | 1:1 (pattern-expanded) |
| `prepare_write_makegen` | `prepare_write_makegen` | 1:1 (pattern-expanded) |
| `execute_makegen_transport` | `execute_makegen_transport` | 1:1 (transport) |

### Compiler insertions

- The `content_upsert` pattern call expands to the 5-node read/compare/write chain automatically.
- `fs_env` and `load_registry` are inserted as root environment nodes.
- The wrapper `makegen` callable node is added by the DSL func declaration; stripped by normalized parity.
- Collection ops (`map`/`join` in `render_makefile` body) can optionally lower to `MapNode`/`JoinNode` IR nodes for data-parallel execution (visible via `--emit-collection-nodes`).

### Manifest comparison

| Field | Builder | Compiled | Match |
|---|---|---|---|
| `total_nodes` | 8 | 9 (8 after stripping wrapper) | Yes |
| Waves | 4 (`[fs_env, load_registry]` → `[render, prepare_read]` → `[execute_read]` → `[compare, prepare_write]` → `[execute_write]`) | 4 (same) | Yes |
| SubDag boundaries | none | none | Yes |
| Scatter points | none | none | Yes |

### Parity status

- **Exact match**: `compare_makegen_topology` returns `is_exact_match() == true`.
- **Execution**: compile → resolve → `execute_dag()` produces valid Makefile output (DryRun + real mode).
- **Parity gates**: `makegen_compiled_vs_builder_topology_parity`, IR snapshot in `makegen_canonical_ir.json`.
- **Gaps**: None. This is the fully proven baseline.

---

## S2 — GCP Credential Chain (`cloud/gcp/credential.dag`) — Phase 2 Proving Workflow

### DSL source (~25 lines)

```
func acquire_gcp_secret(runtime, project, secret_name, ...) -> { token: AccessToken }
  provides auth: AuthContext
{
  cred = credential_chain(runtime: runtime, audience: audience, ...)
  return { token: cred.token }
}
```

The actual complexity lives in `std.patterns::credential_chain`, which the DSL imports.
The hand-wired builder has ~1,688 lines across `lib/gcp-ops/src/graph.rs`.

### Builder shape (15 canonical nodes, GitHub runtime variant)

```
                              net_env
                                │
prepare_github_oidc ──→ execute_github_oidc ──→ parse_github_oidc
                                                       │
                              prepare_sts ──→ execute_sts ──→ parse_sts
                                                                  │
                           should_impersonate ←───────────────────┘
                                │
              prepare_impersonate ──→ execute_impersonate ──→ parse_impersonate
                                                                    │
                prepare_secret_access ──→ execute_secret_access ──→ parse_secret_access
                                                                         │
                                                               build_credential
```

Pattern: 4 sequential transport triplets (OIDC → STS → impersonate → secret access), plus conditional impersonation and credential assembly.

### Compiled DSL shape

15 canonical nodes after normalization. The compiled graph is projected into the same canonical shape
via `normalize_gcp_credential_candidate()`.

| DSL lowered node | Builder node | Match |
|---|---|---|
| `prepare_github_oidc` | `prepare_github_oidc` | 1:1 (transport triplet) |
| `execute_github_oidc` | `execute_github_oidc` | 1:1 |
| `parse_github_oidc` | `parse_github_oidc` | 1:1 |
| `prepare_sts` | `prepare_sts` | 1:1 |
| `execute_sts` | `execute_sts` | 1:1 |
| `parse_sts` | `parse_sts` | 1:1 |
| `should_impersonate` | `should_impersonate` | 1:1 |
| `prepare_impersonate` | `prepare_impersonate` | 1:1 |
| `execute_impersonate` | `execute_impersonate` | 1:1 |
| `parse_impersonate` | `parse_impersonate` | 1:1 |
| `prepare_secret_access` | `prepare_secret_access` | 1:1 |
| `execute_secret_access` | `execute_secret_access` | 1:1 |
| `parse_secret_access` | `parse_secret_access` | 1:1 |
| `net_env` | `net_env` | 1:1 |
| `build_credential` | `build_credential` | 1:1 |

### Compiler insertions

- Service calls (`gcp.SecretManager.AccessSecret()`, etc.) each expand to prepare/execute/parse transport triplets.
- `net_env` is a resource acquisition node inserted for `provides auth: AuthContext`.
- `should_impersonate` is a conditional branch node inserted from `when service_account` guard.
- The DSL `credential_chain` pattern encapsulates the 4-triplet chain; the compiler flattens it.

### Manifest comparison

| Field | Builder | Compiled | Match |
|---|---|---|---|
| Transport triplets | 4 (OIDC, STS, impersonate, secret) | 4 | Yes |
| Resource lifecycle | `net_env` acquire | `net_env` acquire | Yes |
| SubDag boundaries | none (flat after lowering) | none | Yes |
| Semantic annotations | implicit | `@idempotent`, `@readonly`, `@permissions` survive lowering | Enhanced |

### Parity status

- **Exact match**: `compare_gcp_credential_topology` uses `compare_ir()` (deep structural comparison) and returns exact match.
- **Parity gates**: `gcp_credential_normalized_parity_can_reach_exact_match`, `gcp_credential_normalized_parity_report_is_deterministic`.
- **Semantic preservation**: `ServiceCallMetadata` carries hermeticity/idempotency/permissions through lowering.
- **Gaps**: AWS and Azure credential chains compile but don't yet have exact-parity gates against legacy builder shapes (the legacy builders differ per provider).

---

## S4 — Gist Snapshot (`tools/gist.dag`) — Phase 3 Proving Workflow

### DSL source (59 lines, 3 modes)

```
func gist_snapshot(base_ref: CommitSha?) -> { url: Url }
  uses fs: Filesystem(mode: Read)
{
  ctx = branch_context()
  files = git.Core.LsFiles()
  read_result = read_text_files(paths: files.files)
  markdown = render_snapshot(files: read_result.files)
  result = share_content(markdown: markdown, branch: ctx.branch, base_ref: base_ref)
  return { url: result.url }
}
```

The hand-wired builder is ~1,449 lines across `lib/tools/gist/src/graph.rs`.

### Builder shape (snapshot mode, ~15 nodes)

```
fs_env ──→ list_files (triplet: prepare → execute → parse)
                │
                ▼
         read_files_loop ──→ collect_file_contents ──→ render_markdown
                                                            │
           branch_resolution (SubDag: 2 triplets)           │
                       │                                    │
                       └──→ gist_upload (SubDag: cred chain + upload) ←─┘
```

Patterns: transport triplets (git ls-files), LoopBuilder (per-file read), SubDag composition (branch resolution, gist upload with credential chain).

### Compiled DSL shape (per mode)

The compiler produces mode-specific graphs. After normalization, canonical nodes are:

**Snapshot**: `fs_env`, `list_files`, `read_files_loop`, `collect_file_contents`, `render_markdown`, `branch_resolution`, `gist_upload`

**Diff**: `fs_env`, `diff`, `render_markdown`, `branch_resolution`, `gist_upload`

**Recent**: `fs_env`, `diff`, `rev_list`, `render_markdown`, `branch_resolution`, `gist_upload`

| Node | Builder | DSL compiled | Notes |
|---|---|---|---|
| `fs_env` | resource node | resource node | 1:1 |
| `list_files` | 3-node triplet | 3-node triplet | 1:1 (snapshot only) |
| `read_files_loop` | LoopBuilder | `for` loop lowering | 1:1 |
| `collect_file_contents` | pure transform | pure callable | 1:1 |
| `render_markdown` | pure callable | `fn` callable | 1:1 |
| `branch_resolution` | SubDag (2 triplets) | SubDag (composed from `shared.gist_modes`) | 1:1 |
| `gist_upload` | SubDag (cred chain + upload) | SubDag (composed from `share_content`) | 1:1 |

### Compiler insertions

- `for file in files` → LoopUnpack + body SubDag + LoopPack nodes.
- Service calls in composed SubDags → transport triplets.
- `uses fs: Filesystem(mode: Read)` → `fs_env` resource acquisition.
- Scatter points inserted for loop progress counters.
- Collection ops in `render_snapshot` (`map`/`join`) optionally lower to `MapNode`/`JoinNode`.

### Manifest comparison

| Field | Builder | Compiled | Match |
|---|---|---|---|
| SubDag boundaries | `branch_resolution`, `gist_upload` | Same | Yes |
| Scatter points | loop at `read_files_loop` | loop at `for` expansion | Yes |
| Loop progress | manual `[n/N]` | auto-derived from collection nodes | Enhanced |
| Composition depth | 2 levels (gist → branch/upload → cred chain) | Same | Yes |

### Parity status

- **Exact match**: all 3 modes pass via `compare_gist_topology` with `GistParityMode::{Snapshot,Diff,Recent}`.
- **Parity gates**: `gist_snapshot_normalized_parity_can_reach_exact_match`, `gist_diff_normalized_parity_can_reach_exact_match`, `gist_recent_normalized_parity_can_reach_exact_match`.
- **Composition**: `gist_dependency_closure_lowering_reuses_shared_credential_chain` verifies composition wiring through `shared.gist_modes` into `std.patterns::credential_chain`.
- **Compression**: 59 DSL lines vs 1,449 Rust lines.
- **Gaps**: Parity still relies on normalization (stripping DSL-specific wrapper nodes); reducing normalization assumptions is ongoing.

---

## S5 — CI Pipeline (`pipelines/ci.dag`) — Phase 4 Proving Workflow

### DSL source (~90 lines, 12 stages)

```
pipeline ci {
  stage cloud_env { ... }
  stage codegen_stage [after cloud_env] { ... }
  stage bootstrap_stage [after codegen_stage, when codegen_result.success] { ... }
  stage generate [after codegen_stage, after bootstrap_stage] { parallel { ... } }
  stage build_stage [after generate] { ... }
  stage test_stage [after build_stage, when build_result.success] { ... }
  stage lint_stage [after build_stage, when build_result.success] { ... }
  stage guardrails [after generate] { ... }
  stage verify [after generate] { parallel { ... } }
  stage report [after test_stage, after lint_stage, after guardrails, after verify] { ... }
}
```

The hand-wired builder is ~920 lines in `gunbc-dag/src/ci/graph.rs`.

### Builder shape (23 canonical nodes)

```
cloud_env_status
       │
   codegen (inlined: fs_env, execute, stamp_write)
       │
       ├──→ deps_exists
       ├──→ bootstrap ──────────────────────────────────┐
       ├──→ pragma ──────────────────────────────────────┤
       ├──→ testgen ──────────────────────────────────────┤
       │                                                  │
       ├──→ build ──→ test ──→──┐                         │
       │         └──→ clippy ──→┤                         │
       │                        │                         │
       ├──→ guardrail_check ────┤                         │
       │                        │                         │
       ├──→ verify_makegen ─────┤                         │
       ├──→ verify_deps ────────┤                         │
       ├──→ verify_bootstrap ───┤                         │
       ├──→ verify_testgen ─────┤                         │
       ├──→ verify_pragma ──────┤                         │
       │                        │                         │
       └──→ aggregate_verify ───┤                         │
                                └──→ report ←─────────────┘
```

23 nodes. Stage ordering: cloud → codegen → (deps, bootstrap, pragma, testgen) → build → (test, lint) + guardrails + verify → report.

### Compiled DSL shape

23 canonical nodes after normalization, matching the builder topology.

| DSL stage | Canonical nodes | Builder | Match |
|---|---|---|---|
| `cloud_env` | `cloud_env_status` | `cloud_env_status` | 1:1 |
| `codegen_stage` | `codegen_exists`, `prepare_codegen_command`, `execute_codegen`, `parse_codegen_result`, `fs_env`, `prepare_stamp_write`, `execute_stamp_write` | Same set (inlined SubDag) | 1:1 |
| `bootstrap_stage` | `bootstrap` | `bootstrap` | 1:1 |
| `generate` | `pragma`, `testgen` | `pragma`, `testgen` | 1:1 (parallel) |
| `build_stage` | `build` | `build` | 1:1 |
| `test_stage` | `test` | `test` | 1:1 |
| `lint_stage` | `clippy_lint` | `clippy_lint` | 1:1 |
| `guardrails` | `guardrail_check` | `guardrail_check` | 1:1 |
| `verify` | 5 verify nodes + `aggregate_verify_results` | Same | 1:1 |
| `report` | `report` | `report` | 1:1 |

### Compiler insertions

- `pipeline` syntax → stage ordering edges from `after` clauses.
- `parallel { ... }` → parallel group annotation in the manifest.
- `when` guards → conditional execution edges.
- The bootstrap constraint (`after codegen_stage`) is now first-class in the DSL rather than implicit.
- Stage groups are derived into the ProgressManifest for collapsible section rendering.

### Manifest comparison

| Field | Builder | Compiled | Match |
|---|---|---|---|
| Total obligations | 133 | 133 | Yes |
| Stage groups | manual wiring | auto-derived from `stage` blocks | Enhanced |
| Bootstrap constraint | implicit ordering | explicit `after codegen_stage` | Enhanced |
| Parallel groups | manual | auto-derived from `parallel { }` | Enhanced |
| Collapsible sections | manual stage groups | manifest-driven | Enhanced |

### Parity status

- **Exact match**: `compare_ci_topology` projects both graphs into 23-node canonical shape.
- **Parity gates**: `ci_pipeline_normalized_parity_can_reach_exact_match`, `ci_pipeline_normalized_parity_report_is_deterministic`.
- **Obligation count**: `total_obligations: 133` verified via workflow fixture `s5_ci_pipeline.json`.
- **Gaps**: Stage group rendering is structural; runtime stage-progress events not yet wired.

---

## S3 — Tool Install Upsert (`tools/bootstrap.dag`)

### Shape comparison

| Aspect | Builder | DSL |
|---|---|---|
| Pattern | content_upsert x2 (Makefile + .gitignore) | `content_upsert` x2 via pattern import |
| Root node | crate directory scan | `shell.Find.ListDirs(path: "crates")` |
| Parallel chains | 2 independent upsert chains | Same (compiler infers independence) |
| Node count | ~16 | ~16 after expansion |

### Status

- Compiles and emits obligations contract.
- Deterministic parity scaffold is wired but not yet zero-delta.
- **Next**: tighten parity scaffold to exact match.

---

## S6 — LLM Review (`examples/abstract_services.dag`)

### Shape comparison

This workflow introduces the **interface/service/bind** three-layer model. No legacy builder
exists for the abstract service pattern — it is new capability.

| Aspect | Details |
|---|---|
| Interfaces declared | `Storage`, `LLM`, `AbstractQueue<T>`, `DurableStorage` |
| Concrete services | `gcs.Storage`, `aws.S3`, `local.FileStore`, `openai.ChatGPT`, `gcp.VertexAI` |
| Funcs against interfaces | `save_artifact(uses store: Storage)`, `review_code(uses llm: LLM)`, `review_and_store(uses llm + store)` |
| `@contract` annotations | Storage: 3, LLM: 2 |

### Status

- Compiles and emits obligations.
- Interface resolution verifies `uses store: Storage` resolves to concrete service per provider hint.
- **Next**: parity and execution coverage for interface-backed flows.

---

## Clippy (`tools/clippy.dag`)

### Shape comparison

| Aspect | Builder (~186 lines) | DSL (~52 lines) |
|---|---|---|
| Pattern | upsert (check → install → resolve) | `upsert` pattern import |
| Resource | `Clippy` capability | `resource Clippy { check, install, resolve }` |
| Post-upsert | `cargo.Build.Clippy()` | Same service call |
| Node count | ~6 | ~6 after expansion |

### Status

- Compiles and emits obligations.
- Deterministic parity scaffold wired.
- **Next**: tighten to zero-delta parity.

---

## Deps (`tools/deps.dag`)

### Shape comparison

| Aspect | Builder | DSL (~64 lines) |
|---|---|---|
| Install mode | read manifest → loop install per dep | `for dep in platform_deps { ... }` |
| Generate mode | render + content_upsert | `content_upsert(content, path)` |
| Platform filtering | runtime conditional | `fn select_platform_deps` with `filter` + `match` |

### Status

- Compiles and emits obligations.
- **Next**: tighten parity scaffold; add execution bridge.

---

## Build (`tools/build.dag`)

### Shape comparison

| Aspect | Builder (~13 nodes) | DSL (~41 lines) |
|---|---|---|
| Build | cargo build | `cargo.Build.Build()` |
| Post-build | test + clippy (parallel) | `[after build, when build.success]` parallel |
| Aggregate | custom aggregation | `aggregate_results(stages)` via shared helper |

### Status

- Compiles. Deterministic parity scaffold wired.
- **Next**: tighten to zero-delta parity.

---

## Codegen (`tools/codegen.dag`)

### Shape comparison

| Aspect | Builder (~9 nodes) | DSL (~31 lines) |
|---|---|---|
| Check | stamp file freshness | `shell.Codegen.Check()` |
| Execute | conditional codegen run | `[when !check.needed]` guard |
| Stamp | write stamp on success | `fs.write(path, content) [when run.success]` |

### Status

- Compiles. Deterministic parity scaffold wired.
- **Next**: tighten to zero-delta parity.

---

## Pragma (`tools/pragma.dag`)

### Shape comparison

| Aspect | Builder (~19 nodes) | DSL (~69 lines) |
|---|---|---|
| Chains | 3 parallel content_upsert | 3 `content_upsert()` calls (compiler infers parallelism) |
| Render fns | inline rendering | 3 `fn` with `filter |> map |> join` chains |
| Collection ops | opaque | `FilterNode → MapNode → JoinNode` in IR |

### Status

- Compiles. Deterministic parity scaffold wired.
- **Next**: tighten to zero-delta parity.

---

## Docgen (`tools/docgen.dag`)

### Shape comparison

| Aspect | Builder (~54 nodes) | DSL (~82 lines) |
|---|---|---|
| File reads | 13 parallel reads | 13 `fs.read()` calls (compiler infers parallelism) |
| Render | template substitution | `fn render_ab_workflows_doc` with pipe chains |
| Output | content_upsert | `content_upsert(content, path)` |

### Status

- Compiles. Deterministic parity scaffold wired.
- **Next**: tighten to zero-delta parity.

---

## Auth (`services/shell.dag`)

### Shape comparison

Auth is a service module, not a workflow func. It declares shell-based operations that
other workflows consume. Parity is verified transitively through downstream caller tests.

### Status

- Compiles and emits obligations.
- **Next**: validate through downstream workflow parity/execution tests.

---

## Credential — AWS (`cloud/aws/credential.dag`)

### Shape comparison

Mirrors the GCP credential chain structure but with OIDC → STS as the AWS variant.

| Aspect | Builder | DSL |
|---|---|---|
| Triplets | OIDC, STS, assume-role, secret | Same (via `credential_chain` pattern) |
| Provider-specific | `AwsSessionCredentials` type | `required_scopes`, `role_arn` params |

### Status

- Compiles and emits obligations. Provider resource fixture validates interface-contract verification obligations.
- **Next**: tighten to exact-parity against legacy builder shape.

---

## Credential — Azure (`cloud/azure/credential.dag`)

### Shape comparison

Uses federated identity → Azure AD token exchange.

| Aspect | Builder | DSL |
|---|---|---|
| Triplets | federated identity, AD token, secret | Same (via `credential_chain` pattern) |
| Provider-specific | `AzureAccessToken` type | `tenant_id`, `client_id` params |

### Status

- Compiles and emits obligations. Provider resource fixture validates interface-contract verification obligations.
- **Next**: tighten to exact-parity; restore richer federated-flow modeling.

---

## S8 — Infra Bootstrap (`infra/core.dag`)

### Shape comparison

This is new capability (no legacy builder). Defines abstract infrastructure interfaces:

| Interface | Operations | `@contract` annotations |
|---|---|---|
| `ObjectStorage` | Get, Put, Delete, List | 3 behavioral contracts |
| `Compute` | Deploy, Scale, Status | 2 |
| `SecretStore` | Get, Put, Delete | 2 |
| `Identity` | Authenticate, Authorize | 1 |
| `Queue<T>` | Publish, Pull, Ack | 1 |

Concrete implementations exist in `infra/gcp/`, `infra/aws/`, `infra/azure/`.

### Status

- Expands and obligations derive.
- Interface resolution and contract-driven test generation implemented.
- **Next**: runtime contract test execution.

---

## S9 — Cross-Cloud Deployment (`examples/deployment.dag`)

### Shape comparison

New capability. Demonstrates cross-provider composition:

| Aspect | Details |
|---|---|
| Providers | GCP + AWS + Azure in one func |
| `uses` clauses | `store: ObjectStorage` (resolved per provider hint) |
| Credential chains | Each provider resolves independently |
| Interface contracts | `@contract` tests pass for all 3 providers' `ObjectStorage` |

### Status

- Compiles and emits obligations with interface-contract verification targets.
- Lowering regressions cover provider-hint portability and cross-provider credential composition.
- **Next**: parity harness coverage and execution bridge checks.

---

## Summary: Compression Ratios

| Workflow | Builder (Rust lines) | DSL (lines) | Ratio |
|---|---|---|---|
| makegen | ~137 | 5 (+ fn) | 27:1 |
| GCP credential | ~1,688 | ~25 | 67:1 |
| gist (3 modes) | ~1,449 | 59 | 24:1 |
| CI pipeline | ~920 | ~90 | 10:1 |
| clippy | ~186 | ~52 | 3.5:1 |
| bootstrap | ~16 nodes | ~45 | 3:1 (est.) |
| pragma | ~19 nodes | ~69 | 3:1 (est.) |
| docgen | ~54 nodes | ~82 | 3:1 (est.) |

Average compression across proving workflows: **~25:1**.

---

## Summary: Parity Gate Status

| Workflow | Exact parity | Normalized parity | Obligations | Execution |
|---|---|---|---|---|
| S1 makegen | Yes | Yes | Yes | Yes (DryRun + real) |
| S2 GCP credential | Yes | Yes | Yes | Pending |
| S4 gist (3 modes) | Yes | Yes | Yes | Pending |
| S5 CI pipeline | Yes | Yes | Yes (133) | Pending |
| S3 bootstrap | Pending | Scaffold | Yes | Pending |
| S6 abstract services | N/A (new) | N/A | Yes | Pending |
| Clippy | Pending | Scaffold | Yes | Pending |
| Deps | Pending | Scaffold | Yes | Pending |
| Build | Pending | Scaffold | Yes | Pending |
| Codegen | Pending | Scaffold | Yes | Pending |
| Pragma | Pending | Scaffold | Yes | Pending |
| Docgen | Pending | Scaffold | Yes | Pending |
| Auth | N/A (service) | N/A | Yes | Transitive |
| AWS credential | Pending | Scaffold | Yes | Pending |
| Azure credential | Pending | Scaffold | Yes | Pending |
| S8 infra | N/A (new) | N/A | Yes | Pending |
| S9 cross-cloud | N/A (new) | N/A | Yes | Pending |
