# Workflow Audit + Parallelization Plan

**Status**: Draft
**Date**: 2026-02-07

## Goal

Get a full, static, end-to-end view of all workflows (Makefile, CI, binaries, DAGs), identify theoretical complexity and parallelization misses, and outline a consolidation plan so workflows are fast, consistent, and DAG-driven.

## Scope

Static analysis only. No timing measurements or runtime profiling in this pass. The focus is on dependency structure, theoretical complexity, and missed parallelism.

## Inventory (Current Workflows)

**Entry Points / Orchestrators**
- `Makefile` (generated). `Makefile`
- `gunbc-ci` DAG. `gunbc-dag/src/ci/graph.rs`
- `gunbc-build` DAG. `gunbc-dag/src/build/graph.rs`
- `gunbc-codegen` binary (commit/rollback/codegen/cigen). `gunbc-dag/src/bin/codegen_cli.rs`
- `gunbc-codegen-dag` binary (codegen prep DAG). `gunbc-dag/src/codegen/graph.rs`
- `gunbc-testgen` DAG. `gunbc-dag/src/testgen_dag/graph.rs`
- `gunbc-makegen` DAG. `gunbc-dag/src/makegen/graph.rs`
- `gunbc-pragma` DAG. `gunbc-dag/src/pragma/graph.rs`
- `gunbc-bootstrap` DAG. `gunbc-dag/src/bootstrap/graph.rs`
- `gunbc-docgen` DAG. `gunbc-dag/src/docgen/graph.rs`

**Tool DAGs**
- `gunbc-gist` DAGs (snapshot/diff/recent). `lib/tools/gist/src/graph.rs`
- `gunbc-deps` DAGs (install + generate). `lib/tools/deps/src/graph.rs`
- `gunbc-clippy` DAG. `lib/tools/clippy/src/graph.rs`

**Cloud / LLM / Review DAGs**
- LLM chat completion DAG. `lib/llm-ops/src/graph.rs`
- Review DAGs (phase, inline, diff, multi-source). `lib/review/src/graph.rs`
- Cloud secret manager DAGs (provider-neutral). `lib/cloud-ops/src/graph.rs`
- GitHub credential lifecycle DAG. `lib/cloud-ops/src/github_credential_graph.rs`
- GCP WIF + Secret Manager DAGs. `lib/gcp-ops/src/graph.rs`
- AWS Secrets Manager DAG. `lib/aws-ops/src/graph.rs`
- Azure Key Vault DAG. `lib/azure-ops/src/graph.rs`

**Preflight (Lint-Upsert)**
- All binaries run a preflight: check manifest + tracked files, then run codegen/testgen/pragma + clippy if stale. `lib/transport/src/preflight.rs`

## Execution Model (Critical Bottleneck)

The executor runs **strict topological order, single-threaded**. DAG structure encodes parallelism but the runtime does not exploit it. `core/exec/src/execute.rs`

## Workflow Maps (Static)

### Makefile

**Dependency diagram (meta targets)**\n
```text
ensure-codegen
  └─> codegen
       └─> testgen
            └─> build
                 ├─> test
                 └─> test-all

ensure-codegen ─┬─> makegen (verify)
                ├─> bootstrap (verify)
                ├─> testgen (verify)
                └─> pragma (verify)

ensure-codegen ─┬─> pragma-check
                └─> clippy
```

**Steps (selected meta targets)**
1. `ensure-codegen`: `cargo run -p gunbc-dag --bin gunbc-codegen --release -- codegen`
2. `codegen`: `ensure-codegen` then `cargo run -p gunbc-dag --bin gunbc-codegen-dag --release`
3. `testgen`: `ensure-codegen` then `cargo run -p gunbc-dag --bin gunbc-testgen --release`
4. `pragma`: `cargo run -p gunbc-dag --bin gunbc-pragma --release`
5. `build`: `codegen` -> `testgen` -> `cargo build --all-targets`
6. `test`: `build` + `verify-fix` -> `cargo test`
7. `clippy`: `ensure-codegen` + `pragma-check` -> `cargo clippy --all-targets -- -D warnings`
8. `verify`: `ensure-codegen` -> run `makegen`, `bootstrap`, `testgen`, `pragma` in verify mode
9. Tool targets: `deps`, `gist`, `gist-diff`, `gist-recent`, `makegen`, `bootstrap`, `ci`, `build-all`

**Complexity (theoretical)**
- Dominated by Rust compile + test + clippy. Multiple `cargo run` invocations add compile/launch overhead even when incremental.

**Parallelization misses**
- Makefile target graph is serial. Independent tasks are not run concurrently.
- `verify` runs 4 generators sequentially even though they can be parallel after codegen.

### gunbc-ci DAG

**Dependency diagram (stage-level)**\n
```text
codegen
  ├─> bootstrap ─┐
  ├─> pragma ───┼─> verify ─┐
  └─> testgen ──┘           │
         └─> build ─┬─> test ─┤
                    └─> lint ─┤
testgen + pragma ─> guardrails ─┘
```

**Steps (stage-level)**
1. SetupDeps: check for `deps.toml`.
2. Prep: inline codegen DAG (exists -> run -> stamp).
3. Bootstrap, Pragma, Testgen: all depend on codegen.
4. Build: depends on codegen + testgen.
5. Test: depends on build.
6. Lint: depends on build + pragma.
7. Guardrails: depends on testgen + pragma.
8. Verify: depends on codegen + bootstrap + testgen + pragma.
9. Report: depends on test + lint + guardrails + verify.

**Complexity (theoretical)**
- Sum of codegen + bootstrap + pragma + testgen + build + test + clippy + guardrails + verify.

**Parallelization misses**
- Bootstrap, pragma, testgen can run in parallel after codegen.
- Test, lint, guardrails, verify can run in parallel after their deps.
- Executor serializes all nodes.

### gunbc-build DAG

**Dependency diagram**\n
```text
build ─┬─> test ─┐
       └─> clippy ─┤
                   └─> summary
```

**Steps**
1. Build
2. Test (depends on build)
3. Clippy (depends on build)
4. Summary

**Complexity (theoretical)**
- Build + max(test, clippy) + summary.

**Parallelization misses**
- Test and clippy are independent but serialized by executor.

### gunbc-codegen (commit/rollback/codegen/cigen)

**Commit path diagram**\n
```text
codegen (generate CLIs) -> update manifest -> cargo build -> bin setup
```

**Commit path**
1. Generate CLIs
2. Update codegen manifest
3. Cargo build (release)
4. Setup bin directory

**Codegen path**
1. Generate CLIs only

**Cigen path**
1. Generate CI YAMLs

**Complexity (theoretical)**
- Codegen + cargo build; dominated by build.

**Parallelization misses**
- Build cannot start until codegen finishes because generated sources are build inputs.
- No additional parallelism in this binary; best gains are from compile reuse.

### gunbc-codegen-dag

**Dependency diagram**\n
```text
exists-check -> (codegen if stale) -> stamp write
```

**Steps**
1. Check codegen outputs + manifest freshness
2. Run codegen if stale
3. Write stamp

**Complexity (theoretical)**
- O(1) existence checks, plus codegen if stale.

**Parallelization misses**
- Single chain; no intrinsic parallelism.

### gunbc-testgen

**Dependency diagram (per target, parallelizable)**\n
```text
generate_{t} -> prepare_read_{t} -> execute_read_{t} -> compare_{t} -> execute_write_{t}
             └-> prepare_write_{t} -------------------------------> (request)
```

**Steps**
1. Discover testgen targets from registry
2. Build DAG with N upsert chains (one per target)
3. For each target: generate -> read -> compare -> write

**Complexity (theoretical)**
- O(N) target generation + O(M) file read/compare/write.

**Parallelization misses**
- Each target chain is independent but executor is serial.

### gunbc-makegen

**Dependency diagram**\n
```text
load_registry -> render_makefile -> (read/compare/write upsert)
```

**Steps**
1. Load tool registry
2. Render Makefile
3. Content upsert: read -> compare -> write

**Complexity (theoretical)**
- O(T) registry size + O(1) file read/compare/write.

**Parallelization misses**
- Single chain; no intrinsic parallelism.

### gunbc-pragma

**Dependency diagram (three independent chains)**\n
```text
render_clippy   -> upsert clippy.toml
render_allowlist -> upsert allowlist
render_policy   -> upsert policy
```

**Steps**
1. Render clippy.toml
2. Render allowlist
3. Render policy
4. Each output has its own upsert chain

**Complexity (theoretical)**
- O(1) renders + 3 file read/compare/write chains.

**Parallelization misses**
- Three chains can run in parallel but are serialized by executor.

### gunbc-bootstrap

**Dependency diagram**\n
```text
scan_workspace
  ├─> generate_makefile -> upsert Makefile
  └─> generate_gitignore -> upsert .gitignore
```

**Steps**
1. Scan workspace (discover crates)
2. Generate Makefile content
3. Generate .gitignore content
4. Upsert Makefile (read/compare/write)
5. Upsert .gitignore (read/compare/write)

**Complexity (theoretical)**
- O(C) crate scan + 2 upsert chains.

**Parallelization misses**
- Two upsert chains are independent after scan.

### gunbc-docgen

**Dependency diagram**\n
```text
read_inputs (many in parallel) -> render_doc -> upsert doc
```

**Steps**
1. Read many files in parallel (via transport triplets)
2. Render doc
3. Upsert doc

**Complexity (theoretical)**
- O(R) file reads + one render + one upsert chain.

**Parallelization misses**
- All reads are independent but serialized by executor.

### gunbc-gist (tool)

**Snapshot mode**\n
```text
list_files -> read_files (loop) -> render -> create gist
```

**Diff mode**\n
```text
git diff -> render diff -> create gist
```

**Recent mode**\n
```text
rev-list -> git diff -> render diff -> create gist
```

**Snapshot mode**
1. List files
2. Read files (loop)
3. Render markdown
4. Create gist

**Diff mode**
1. Git diff
2. Render diff
3. Create gist

**Recent mode**
1. Rev-list (find base)
2. Git diff
3. Render diff
4. Create gist

**Complexity (theoretical)**
- Snapshot is O(F) file reads and content size. Diff is O(D) where D is diff size.

**Parallelization misses**
- File reads are independent but serialized by executor.

### gunbc-deps (tool)

**Dependency diagram**\n
```text
platform_env + load_manifest -> generate_scripts -> execute_installs
```

**Steps**
1. Platform env (resource)
2. Load manifest
3. Generate install script
4. Execute install script

**Complexity (theoretical)**
- Depends on manifest size + number of packages; execution cost dominated by installer.

**Parallelization misses**
- Script execution is monolithic; no parallelization in current model.

### gunbc-deps (generate deps.toml)

**Dependency diagram**\n
```text
load_tool_registry -> render_deps -> (read/compare/write upsert)
```

**Steps**
1. Load tool registry
2. Render deps.toml content
3. Content upsert: read -> compare -> write

**Complexity (theoretical)**
- O(T) registry size + O(1) file read/compare/write.

**Parallelization misses**
- Single chain; no intrinsic parallelism.

### gunbc-clippy (tool)

**Dependency diagram**\n
```text
check -> create -> resolve
```

**Steps**
1. Check for tool availability
2. Install tool if missing
3. Execute tool (clippy)

**Complexity (theoretical)**
- Dominated by tool install (if needed) and cargo clippy runtime.

**Parallelization misses**
- Single chain; no intrinsic parallelism.

### lib/llm-ops (LLM chat completion)

**Dependency diagram**\n
```text
cloud_env -> bind_secret -> cloud_credential (subdag)
prepare -> resolve_auth -> bind_secret -> cloud_credential -> execute -> parse
```

**Steps**
1. Resolve cloud env config and OIDC inputs
2. Prepare request
3. Resolve auth scheme
4. Bind secret name and acquire credential (subdag)
5. Execute transport (LLM API call)
6. Parse response

**Complexity (theoretical)**
- Dominated by network I/O (OIDC + secret manager + LLM API).

**Parallelization misses**
- Cloud credential acquisition and request preparation could overlap if inputs allow.
- Executor serializes all nodes.

### lib/review (review workflows)

**Dependency diagram (phase graph)**\n
```text
prepare_blob -> blob_fetch (transport) -> parse_blob -> prepare_prompt
prepare_prompt -> llm_execute (transport) -> parse_review
```

**Steps**
1. Prepare blob source (inline or remote)
2. Fetch blob via transport if needed
3. Prepare review prompt
4. LLM execute
5. Parse review response

**Complexity (theoretical)**
- Dominated by blob fetch + LLM API.

**Parallelization misses**
- Multi-source review can parallelize blob fetches.
- Executor serializes all nodes.

### lib/cloud-ops / provider DAGs

**Dependency diagram (provider-neutral)**\n
```text
resolve_config -> map_inputs -> provider_subdag
```

**Provider subdags (typical)**\n
```text
oidc_exchange -> access_token -> secret_fetch -> parse -> credential
```

**Complexity (theoretical)**
- Dominated by network calls (OIDC, token exchange, secret manager).

**Parallelization misses**
- Provider subdag chains are linear but could overlap with other independent tasks.

## Complexity + Bottlenecks (Cross-Cutting)

1. **Executor is serial**
- All DAG parallelism is theoretical only.

2. **Repeated `cargo` invocations**
- Many workflows run `cargo run` or `cargo clippy` in separate processes.
- Cost includes incremental compile + startup for each binary.

3. **Preflight O(n) scans on every binary**
- `git ls-files` + per-file mtime stat even on clean repos.
- When stale, preflight reads all tracked files to compute hash (O(n) read).

4. **Makefile duplicates orchestration logic**
- Makefile enforces `ensure-codegen` and calls binaries sequentially.
- CI DAG and preflight perform similar upsert logic separately.

5. **Content upsert chains re-read outputs**
- Makegen/pragma/bootstrap/docgen always read + compare outputs.
- No manifest-based fast path for those generated files.
6. **Network-heavy DAGs are serial**
- LLM/review/cloud graphs are fully serial even when substeps could overlap.

## Blocking Dependency: Resource Declaration + Purity Enforcement

**Expectation:** every node is pure, and all I/O is represented explicitly with resource access metadata.

**Current gap**
- Most nodes do not declare `res:*` resource inputs, so resource conflicts are invisible to the scheduler.
- `TransportRequest::Shell` is opaque and cannot be safely parallelized without explicit resource annotations.
- Preflight runs outside the DAG, bypassing resource modeling entirely.

**Why this blocks parallel execution**
- A parallel scheduler requires explicit resource access to avoid unsafe concurrent writes.
- Without resource declarations, the scheduler must conservatively serialize or risk data races.

**Required changes (by construction)**
1. **Declare resource access on every I/O node**
   - Transport nodes must declare `res:file:*` and `res:tool:*` inputs with `AccessMode`.
   - For content upsert chains, the generator already knows the output path — it should attach resource ports automatically.
   - For CLI tool ops, attach `res:tool:<id>` using the tool’s declared access mode.
2. **Make resource declarations mandatory**
   - DAG build should fail if any node performs I/O without declared resource access.
   - Add `detect_resource_conflicts()` + `validate_resource_ordering()` to build-time or CI checks.
3. **Move preflight into the DAG model**
   - Treat lint-upsert as a managed resource or DAG stage so it participates in resource conflict rules.

**Purity enforcement (tests / integration)**
- Add tests that prove purity “by construction”:
  - Unit tests: `derive_resource_accesses()` must succeed for all DAGs.
  - Unit tests: `detect_resource_conflicts()` returns empty for all DAGs.
  - Integration tests: “no transport I/O” for pure ops (only `TransportOps::Execute` nodes may emit I/O).
  - CI check: fail if any DAG introduces new nodes without resource declarations.

## Sandboxability Roadmap (Keep It Simple First)

**Goal:** make all DAG nodes naturally sandboxable and replayable by ensuring *all* I/O is explicit and centralized in transport boundaries.

### Phase 0: Strict purity boundaries (cheap, high leverage)
- **Rule**: Only transport-layer crates may do I/O (filesystem, network, process exec).
- **Enforcement**: clippy `disallowed_methods` for `std::fs`, `std::process::Command`, `reqwest/ureq`, `git2`, etc.
- **Policy**: allowlist only boundary crates (e.g., `lib/transport`, `lib/cloud-ops` if truly boundary, `lib/tools/*` if they wrap CLI execution).
- **Migration path**: start as warnings, then flip to deny after violations are eliminated.

### Phase 1: Resource declarations by construction
- Update core DAG patterns so they *always* add `res:*` ports for any I/O:
  - `add_transport_triplet*`
  - `add_content_upsert_chain`
  - `build_cli_upsert` (tool install + exec)
- Define resource ids consistently: `res:file:<path>`, `res:tool:<id>`, `res:api:<provider>`, `res:repo`, `res:target`, etc.

### Phase 2: Auto-registered resource tests (static purity)
- Add a `#[resource_test_target]` macro that registers a function pointer.
- Each DAG builder registers itself once; the test runner iterates all and runs:
  - `derive_resource_accesses()`
  - `detect_resource_conflicts()`
  - `validate_resource_wiring_recursive()`
- Integrate into CI (fast).

### Phase 3: Lightweight runtime file guard (optional)
- For test runs, snapshot mtime or hash for `res:file:*` before/after each node.
- If a node writes without declared write access, fail the test.
- Enabled only in tests or when `GUNBC_RESOURCE_GUARD=1`.

### Phase 4: Sandbox + durability/replay (longer-term)
- Record transport I/O operations (requests, responses, file writes).
- Enable deterministic replay for tests and retries (durability).
- Consider OS-level sandboxing later (ptrace/seccomp/containers) if needed.

## Resource Declaration Gap Audit (Per DAG)

**Global pattern gaps**
- `add_transport_triplet` / `add_skippable_transport_triplet` do **not** add `res:*` ports. Every `execute_*` node they create is missing explicit resource access.
- `add_content_upsert_chain` does **not** add `res:*` ports for read/write. Output paths are known but not modeled as resources.
- Resource IDs are **static** (derived from port names). Dynamic path resources (e.g., `--path` entrypoints) are not representable without a coarser `FilesystemHandle` or a new dynamic resource scheme.

**gunbc-ci**
- Missing resources on: `execute_deps_exists`, `execute_codegen_exists`, `execute_codegen`, `execute_stamp_write`, `execute_bootstrap`, `execute_pragma`, `execute_testgen`, `execute_build`, `execute_test`, `clippy_lint`, `execute_guardrail_check`, `execute_verify_check`.
- Suggested resources: `res:file:deps.toml` (read), `res:build:generated_cli` (write), `res:file:target/.codegen-stamp` (write), `res:tool:cargo` (exec), `res:tool:clippy` (exec), plus a coarse `res:workspace` or `res:target` lock for cargo build/test/clippy if we can’t model finer-grained conflicts.

**gunbc-build**
- Missing resources on: `execute_build`, `execute_test`, `execute_clippy`.
- Suggested: `res:tool:cargo` (exec), `res:target` (write). Decide policy on parallel test/clippy vs shared target dir.

**gunbc-codegen-dag**
- Missing resources on: `execute_codegen_exists`, `execute_codegen`, `execute_stamp_write`.
- Suggested: `res:build:generated_cli` (read/write), `res:file:target/.codegen-stamp` (write).

**gunbc-testgen**
- Missing resources on all `execute_read_*` and `execute_*_transport` nodes per target.
- Suggested: `res:file:<generated_tests_path>` read/write per target (or coarse `res:fs`).

**gunbc-makegen**
- Missing resources on `execute_read_makegen` and `execute_makegen_transport`.
- MockSpec already expects `fs:Makefile` but DAG does not declare it.
- Suggested: `res:file:Makefile` read/write (or `res:fs` if path is dynamic).

**gunbc-pragma**
- Missing resources on `execute_read_clippy`, `execute_clippy_transport`, `execute_read_allowlist`, `execute_allowlist_transport`, `execute_read_policy`, `execute_policy_transport`.
- MockSpec expects `fs:clippy.toml`, `fs:tools/disallowed-methods-allowlist.txt`, `fs:tools/pragma-lint-policy.txt` but DAG does not declare them.
- Suggested: per-file read/write resources.

**gunbc-bootstrap**
- Missing resources on `execute_read_makefile`, `execute_makefile_transport`, `execute_read_gitignore`, `execute_gitignore_transport`, plus `execute_scan_workspace`.
- Suggested: per-file read/write resources + coarse `res:workspace` read for scan.

**gunbc-docgen**
- Missing resources on all read triplets + doc write.
- Suggested: per-file read resources for inputs + `res:file:docs/ab-writing-workflows.md` write, or coarse `res:fs` read/write.

**gunbc-gist**
- Has `res:fs` and `res:clock` handles, but file reads + git commands + network requests are not resource-declared.
- Suggested: `res:repo` read (git), `res:file:*` read (snapshot), `res:api:github` write (gist creation), or coarse `res:fs` + `res:net`.

**gunbc-deps**
- Uses `res:platform` but manifest read + install script execution are not declared.
- Suggested: `res:file:deps.toml` read, `res:pkg:manager` exclusive for installs.

**gunbc-deps (generate deps.toml)**
- Missing resources on `execute_read_deps` / `execute_deps_transport`.
- Suggested: `res:file:deps.toml` read/write.

**gunbc-clippy**
- Missing resources on `check`, `create`, `resolve` nodes (CLI upsert).
- Suggested: `res:tool:clippy` exec, `res:tool:rustup` exec, and a coarse `res:toolchain` or `res:target` write lock if clippy writes target artifacts.

**lib/llm-ops**
- Missing network / API resources on `execute` transport node.
- Suggested: `res:api:<provider>` write (or `res:net` write), `res:credential` read already exists.

**lib/review**
- Missing resources for blob fetch + git diff and LLM transport nodes.
- Suggested: `res:repo` read for git ops, `res:file:*` or `res:blob` read for blob sources, `res:api:<provider>` write for LLM, plus credential resource read.

**lib/cloud-ops**
- Missing resources on OIDC/token/secret manager transport nodes.
- Suggested: `res:cloud:oidc` write, `res:cloud:secrets` write, `res:credential` write.

**lib/gcp-ops**
- Missing resources on metadata server + token exchange + secret fetch nodes.
- Suggested: `res:cloud:gcp:metadata` write, `res:cloud:gcp:sts` write, `res:cloud:gcp:secrets` write.

**lib/aws-ops**
- Missing resources on STS + Secrets Manager nodes.
- Suggested: `res:cloud:aws:sts` write, `res:cloud:aws:secrets` write.

**lib/azure-ops**
- Missing resources on metadata + Key Vault nodes.
- Suggested: `res:cloud:azure:metadata` write, `res:cloud:azure:keyvault` write.

**gunbc-dag workspace subdags**
- Workspace SubDag wrappers inherit resource gaps from their inner DAGs; no extra declarations.

## Parallelization Misses (Summary)

- CI: bootstrap, pragma, testgen can run in parallel after codegen.
- CI: test, lint, guardrails, verify can run in parallel after their deps.
- Build: test and clippy should run concurrently.
- Testgen: N upsert chains can run concurrently.
- Pragma: three upsert chains can run concurrently.
- Docgen: read triplets can run concurrently.
- Gist snapshot: per-file reads can run concurrently.
- Review multi-source: blob fetches can run concurrently.
- Cloud credential graphs: independent provider steps could overlap if explicitly modeled.

## Consolidation Opportunities (Design Direction)

- **Single canonical workflow registry**
  - Define workflows once and generate Makefile + CI + CLI wrappers.
  - Avoid divergent dependency graphs between Makefile and DAGs.

- **Promote preflight into the DAG model**
  - Represent lint-upsert as a managed resource with explicit inputs/outputs.
  - Schedule it alongside other resources instead of running in every binary.

- **Reduce `cargo` invocations**
  - Use one orchestrator binary per workflow, or `cargo build --bins` once then run binaries directly.

- **Add executor parallelism**
  - Ready-queue + worker pool with resource conflict detection.
  - Reuse existing resource access modeling to prevent races.

- **Fast-path freshness**
  - Use `git status --porcelain` + `HEAD` hash to skip per-file scans when clean.

## Consolidated Workflow Model Proposal (SSoT)

### Target

Define workflows once in a canonical registry and generate:
1. DAG builders (runtime graph shape)
2. CLI wrappers (`gunbc-*` binaries)
3. Makefile targets
4. CI workflow steps

### Proposed Registry Shape

```text
WorkflowSpec {
  id: "ci" | "build" | "codegen" | ...,
  mode: "normal" | "verify",
  steps: [StepSpec],
  edges: [(from_step, to_step)],
  resources: [ResourceContract],
  outputs: [WorkflowOutput],
}
```

```text
StepSpec {
  id: string,
  kind: "prepare" | "execute" | "parse" | "subworkflow",
  command_template?: string,
  skip_on_failure_of?: [step_id],
  verify_equivalent?: string,
}
```

### Generation Rules

1. `WorkflowSpec -> DAG`: generate node/edge assembly code with stable node IDs.
2. `WorkflowSpec -> CLI`: generate `--mode=verify` / normal command wiring from the same step definitions.
3. `WorkflowSpec -> Makefile`: generate targets and dependencies from the same edge set.
4. `WorkflowSpec -> CI`: generate CI stage order and command invocations from the same stage projection.

### Validation Gates

1. Golden tests for generated Makefile and CI snippets.
2. Contract tests for command arguments (`workflow_acceptance` style).
3. Registry completeness test: all shipping workflows appear in the registry.

## Parallel Executor Plan

### Scheduler

Use a deterministic ready-queue on top of topological ordering:
1. Track indegree for each node.
2. Push indegree-0 nodes into a ready queue.
3. Pop nodes in deterministic order (stable lexical `NodeId` tie-break).
4. Dispatch to worker pool when resource locks permit.

### Worker Pool

1. `N` workers, configurable (default: logical CPUs bounded by a sane cap).
2. Nodes execute independently once admitted by scheduler.
3. Completion updates downstream indegree and unlocks resources.

### Resource Conflict Gating

1. Use `derive_resource_accesses()` output as lock requests.
2. Read/Read compatible; any Write conflicts with Read/Write on same resource.
3. Admission control blocks conflicting nodes until lock release.
4. Missing resource declarations are hard errors in parallel mode.

### Failure + Skip Semantics

1. Preserve current skip propagation behavior.
2. Failed node marks dependents as skipped when configured.
3. No partial reordering after failure beyond already running nodes.

### Verification

1. Determinism test: same DAG + mocks produces stable terminal outputs across repeated runs.
2. Conflict test: intentionally conflicting nodes never overlap.
3. Throughput smoke test: synthetic independent nodes execute faster with `N>1`.

## Fast-Path Freshness Proposal

### Goal

Skip O(n) file scans in preflight when repo state is unchanged.

### Key

Cache tuple:
1. `HEAD` commit (`git rev-parse HEAD`)
2. Dirty bit + staged/unstaged summary (`git status --porcelain --untracked-files=no`)
3. Toolchain fingerprint (`rustc --version`, `cargo clippy --version`)

### Behavior

1. If key matches previous run and dirty bit is clean, skip tracked-file stat/hash walk.
2. If dirty or key missing/mismatched, fall back to current full scan/hash behavior.
3. If `git` unavailable, use existing conservative full-scan path.

### Cache Location

`target/.gunbc-preflight-cache.json` (ephemeral, local, ignored by VCS).

## Workflow Merge/Retire Decisions

1. Merge `ensure-codegen` semantics into canonical workflow preflight stage (retire standalone Makefile-only orchestration role).
2. Keep `gunbc-codegen-dag` as the canonical codegen freshness workflow; treat `gunbc-codegen` CLI as authoring/maintenance tool, not orchestration source.
3. Keep `makegen`, `bootstrap`, `testgen`, `pragma` as distinct subworkflows, but invoke via registry-driven composition (not hard-coded duplicated chains).
4. Keep CI `verify` stage as separate mode projection (`mode=verify`) from same registry steps.
5. Retire duplicated dependency wiring in handwritten Makefile/CI once generated outputs are authoritative.

## Implementation Roadmap

### Phase 1: Audit + Metrics

1. Land canonical `WorkflowSpec` types and registry loader.
2. Add completeness tests: every existing workflow has a registry entry.
3. Add baseline metrics hooks (node counts, critical path estimate, command counts).

### Phase 2: Consolidate

1. Generate Makefile + CI fragments from registry.
2. Generate CLI wrapper wiring from registry.
3. Keep old handwritten paths behind parity tests until outputs match.

### Phase 3: Parallel Runtime

1. Implement ready-queue + worker pool executor.
2. Enable resource lock gating from declared accesses.
3. Roll out behind feature flag, then make default after determinism/conflict tests pass.

## Tasks

- [x] Extend this doc with a dependency diagram for each workflow (ASCII or graph description). _(2026-02-14: completed in “Workflow Maps (Static)” sections for Makefile, CI, build, codegen, testgen, makegen, pragma, bootstrap, docgen, gist, deps, clippy, llm/review/cloud workflows.)_
- [x] Build a consolidated workflow model proposal (single source of truth for Makefile + CI + CLI). _(2026-02-14: completed in “Consolidated Workflow Model Proposal (SSoT)”.)_
- [x] Identify all `cargo` invocations across workflows and propose a single-build + multi-run strategy. _(2026-02-14: inventory + strategy below; CI Build stage switched to `cargo test --no-run` and acceptance-verified in `gunbc-dag/tests/workflow_acceptance.rs`.)_
- [x] Design a parallel executor plan (ready-queue, worker pool, resource conflict checks). _(2026-02-14: completed in “Parallel Executor Plan”.)_
- [x] Propose fast-path freshness detection (git HEAD/dirty state) to avoid per-file stat loops. _(2026-02-14: completed in “Fast-Path Freshness Proposal”.)_
- [x] Decide which workflows should be merged/retired (e.g., `ensure-codegen` vs preflight vs codegen DAG). _(2026-02-14: completed in “Workflow Merge/Retire Decisions”.)_
- [x] Draft an implementation roadmap: phase 1 (audit + metrics), phase 2 (consolidate), phase 3 (parallel runtime). _(2026-02-14: completed in “Implementation Roadmap”.)_
- [x] Add a “resource declaration gap” audit for each DAG (which nodes need `res:*` annotations). _(2026-02-14: completed in “Resource Declaration Gap Audit (Per DAG)”.)_
- [x] Add purity enforcement tests (derive_resource_accesses + detect_resource_conflicts). _(2026-02-14: registry-wide test runner active in `gunbc-dag/tests/resource_purity_checks.rs`.)_
- [x] Ensure every DAG builder is registered (testgen registry) so purity tests cover the entire codebase. _(2026-02-14: added source + runtime coverage gates in `gunbc-dag/tests/resource_registry_coverage.rs`; removed `resource_test_target(skip)` from canonical workflow builders and registered missing local/upsert variants.)_
- [x] Add clippy guardrails to forbid direct I/O in pure crates (only transport/boundary crates allowed). _(2026-02-14: enforced via root `clippy.toml` disallowed-methods policy.)_
- [x] Add `#[resource_test_target]` registry + test runner for codebase-wide purity checks. _(2026-02-14: registry implemented + CI guardrail now runs `resource_purity_checks`.)_
- [ ] Add optional runtime file guard for `res:file:*` during tests.
- [ ] Draft a sandbox + durability/replay RFC (record/replay transport I/O, deterministic tests).

## Workflow Update Task List (Start ASAP)

**Phase A: Define and enforce purity boundaries (mostly done)**
- [x] Inventory which crates are allowed to do I/O (transport/boundary only).
- [x] Add clippy `disallowed_methods` for `std::fs` (23 entries), `std::process::Command` (1 entry) — enforced via root `clippy.toml`. Infrastructure crate (`core/infra`) has empty `disallowed_methods` (allowed).
- [x] Add clippy `disallowed_methods` for `reqwest`, `ureq`, `git2` (if used) — remaining gap.
- [x] Clean up violations or move I/O into transport/boundary crates.

**Phase B: Resource declarations by construction (now)**
- [x] Update `add_transport_triplet*` to always attach `res:*` ports (file/network/tool).
- [x] Update `add_content_upsert_chain` to declare `res:file:*` read/write for outputs.
- [x] Update `build_cli_upsert` to declare `res:tool:*` and any `res:target`/`res:pkg` locks. _(2026-02-14: verified by unit tests in `core/ir/src/transport/cli.rs`)_
- [x] Normalize resource id naming (`res:file:<path>`, `res:tool:<id>`, `res:api:<provider>`, `res:repo`, `res:target`). _(2026-02-14: added canonical ID normalization in `derive_resource_accesses()` + resource mock alias compatibility; legacy `res:fs`/`res:net`/`res:pkg` continue to work.)_

**Phase C: Codebase-wide purity checks (now)**
- [x] Implement `#[resource_test_target]` registry (auto-register DAG builders).
- [x] Add a single test runner that iterates all registered DAGs and runs:
  - `derive_resource_accesses()`
  - `detect_resource_conflicts()`
  - `validate_resource_wiring_recursive()`
- [x] Wire into CI (fast, deterministic). _(2026-02-14: added `gunbc-dag/tests/resource_purity_checks.rs` and wired guardrail command to run `cargo test -p gunbc-dag --test resource_purity_checks` alongside `tools/check-disallowed-methods.sh`.)_

**Phase D: Workflow consolidation + parallelism (next)**
- [ ] Consolidate Makefile + CI + CLI to a single canonical workflow registry.
- [ ] Add ready-queue executor with resource conflict gating (parallelism).
- [ ] Add fast-path freshness check to skip full scans on clean repos.

## Acceptance Criteria (Current Workflow)

Target workflow states and verification gates:

1. CI build/test command contract
   - Build stage compiles test artifacts with `cargo test --no-run` (no standalone `cargo build` pass).
   - Test stage runs `cargo test` (execution stage).
   - Verify with: `cargo test -p gunbc-dag --test workflow_acceptance ci_build_stage_compiles_tests_without_running_them ci_test_stage_runs_tests_after_build`
   - Failure-state verify: `cargo test -p gunbc-dag --test workflow_acceptance ci_test_stage_skips_when_build_fails`

2. CI guardrail/purity contract
   - Guardrail stage runs both disallowed-method checks and registry-wide resource purity checks.
   - Verify with: `cargo test -p gunbc-dag --test workflow_acceptance ci_guardrail_stage_runs_disallowed_methods_and_resource_purity_checks`
   - Failure-state verify: `cargo test -p gunbc-dag --test workflow_acceptance ci_guardrail_stage_skips_when_upstream_fails`
   - Verify end-to-end with: `cargo test -p gunbc-dag guardrail_check`

3. CI verify-mode contract
   - Verify sub-stages (`makegen`, `bootstrap`, `testgen`, `pragma`) run with `--mode=verify`.
   - Verify with: `cargo test -p gunbc-dag --test workflow_acceptance ci_verify_stage_uses_verify_mode_commands`
   - Failure-state verify: `cargo test -p gunbc-dag --test workflow_acceptance ci_verify_stage_skips_when_prep_fails`

4. Resource purity always-on
   - Registry-wide purity test remains green and CI-callable.
   - Verify with: `cargo test -p gunbc-dag --test resource_purity_checks`

5. Builder registration coverage
   - Every public zero-arg `build_*graph*` builder is covered by non-skip `#[resource_test_target]` registration.
   - Runtime `iter_resource_tests()` includes non-skip source annotations for force-linked workflow crates.
   - Verify with: `cargo test -p gunbc-dag --test resource_registry_coverage`

## Cargo Inventory + Strategy (2026-02-14)

Current CI command inventory (from `gunbc-dag` CI prepare nodes):

1. Build stage: `cargo test --no-run` (compile test artifacts once)
2. Test stage: `cargo test` (execute tests)
3. Lint stage: `cargo clippy --all-targets -- -D warnings`
4. Verify stages: `cargo run -p gunbc-dag --bin gunbc-{makegen|bootstrap|testgen|pragma} -- --mode=verify`
5. Guardrail stage: `tools/check-disallowed-methods.sh` plus `cargo test -p gunbc-dag --test resource_purity_checks`

Single-build + multi-run strategy target:

1. Keep Build as compile-only (`cargo test --no-run`) to avoid a separate `cargo build` pass.
2. Keep Test as execution-only (`cargo test`), reusing artifacts from Build where possible.
3. Keep Lint independent (different analysis pipeline), but treat as non-blocking to test artifact reuse decisions.
4. Keep Verify as command-level freshness checks (`--mode=verify`) rather than full rebuilds.
5. Keep Guardrail purity check explicit until/unless test-stage gating guarantees deterministic execution order for purity assertions.

## Deferred: Full Sandbox + Durability (Roadmap)

- [ ] Record/replay transport I/O for deterministic tests and retries.
- [ ] Define durable I/O log schema (requests, responses, file writes, env).
- [ ] Optional OS-level sandboxing (ptrace/seccomp/containers) for strong guarantees.

## Notes

Key files for the audit:
- `Makefile`
- `core/exec/src/execute.rs`
- `gunbc-dag/src/ci/graph.rs`
- `gunbc-dag/src/build/graph.rs`
- `gunbc-dag/src/codegen/graph.rs`
- `gunbc-dag/src/testgen_dag/graph.rs`
- `gunbc-dag/src/makegen/graph.rs`
- `gunbc-dag/src/pragma/graph.rs`
- `gunbc-dag/src/bootstrap/graph.rs`
- `gunbc-dag/src/docgen/graph.rs`
- `lib/tools/gist/src/graph.rs`
- `lib/tools/deps/src/graph.rs`
- `lib/transport/src/preflight.rs`
