# Workflow Minimal Execution Model

Status: Draft
Owner: `gunbc` workflow/runtime
Scope: `make ci`, `make test-all`, and shared generation/lint/build/test orchestration
Detailed design pack: `docs/design/workflow/wf1-wf4-dag-design-pack.md`

## 0. Document Contract

This document is the normative source for workflow execution semantics.

1. Canonical model authority lives here: typing, flattening, key/ledger, admission,
   execution semantics, no-fallback policy, and proof obligations.
2. `docs/design/workflow/wf1-wf4-dag-design-pack.md` is a derived WF1-WF4 review
   pack with workflow DAG visuals and ownership evidence.
3. If the two documents disagree, this document is authoritative.
4. Derived documents must not introduce alternate dependency/effect/claim semantics.

## 1. Problem Statement

`gunbc` has the right low-level primitives (typed DAGs, resource manifests, effect metadata), but top-level workflow execution is still modeled as imperative target chaining.

Current consequences:

1. Redundant compile/generator work across `build`, `verify-fix`, and test targets.
2. Repeated freshness checks and repeated tool startup overhead in one command.
3. Runtime behavior encoded in Make target composition instead of a typed execution model.
4. Slow no-op and warm-path commands despite a medium-sized repo.
5. Semantic drift risk: workflow meaning is split across multiple surfaces (Make
   composition + per-tool glue + freshness policy), so equivalence is not mechanically provable.

User-facing requirement:

1. Commands should usually return in seconds on warm state.
2. No hidden fallback/deprecated path behavior.
3. Workflows should naturally avoid redundant work by construction.

### 1.1 Missing Formal Object

What is missing is a canonical function:

`Plan(workflow_spec, declared_inputs, workspace_state, prior_global_ledger) -> (execution_plan, explanation)`

Without this function, we cannot prove determinism, minimality, or at-most-once
properties over end-to-end workflows.

### 1.2 Symptom vs Deficiency Mapping

1. Symptom: redundant compile/generator work across targets.
   Deficiency: no global flattening + dedup over a canonical resolved graph.
2. Symptom: repeated freshness checks and tool startup overhead.
   Deficiency: planning/execution happens per target chain, not once per resolved graph.
3. Symptom: hidden fallback/deprecated behavior.
   Deficiency: orchestration is not constrained by a closed typed fail-closed model.
4. Symptom: warm/no-op latency drift.
   Deficiency: no canonical key/ledger function to prove zero functional work.

### 1.3 Trust Boundary

Correctness/minimality guarantees are relative to declared inputs/outputs/effects.
Undeclared dependencies are model violations and must fail validation.

## 2. Existing Signals In This Repo

The model is already partially present:

1. Unified resource freshness model: `core/ir/src/resource/mod.rs`.
2. Resource definitions + content-key inputs: `core/ir/src/resource/defs.rs`.
3. Effect taxonomy with cacheability semantics: `core/ir/src/effect.rs`.
4. Freshness DAG composition primitive: `core/exec/src/freshness.rs`.
5. Current command-chain freshness planner (imperative): `lib/transport/src/freshness_policy.rs`.
6. Make orchestration that duplicates work across targets: `Makefile`.

Main gap:

1. There is no single typed planner for end-to-end workflows. Freshness is modeled, but orchestration still lives above the model in Make composition.

## 3. External Modeling Patterns To Reuse

Patterns observed in local sibling repos:

1. Producer-centric regeneration and explicit stamp contracts: `/home/briansrls/the-gunbai/Makefile`.
2. Changed-target routing for CI/debuggability: `/home/briansrls/gunb.ai/Makefile` (`router-debug` + CI router).
3. Resource-capacity-aware DAG execution contracts: `/home/briansrls/gunb.ai/docs/pkg-dag.md`.

Adaptation for `gunbc`:

1. Keep typed DAG + resource semantics.
2. Replace stamp-file orchestration with a typed key ledger.
3. Treat changed-input routing as an optimization hint unless proven complete; it
   must never reduce soundness of the computed execute set.

## 4. Design Goals

1. Single planner computes minimal executable frontier once per run.
2. Deterministic materialization keys per workflow node.
3. Explicit invalidation causes (input/content/env/toolchain/policy) recorded in run ledger.
4. Zero implicit fallback paths.
5. Stable latency targets with observable SLOs.

### 4.1 Formal Predicates

Define `Warm(S, L, W)` as:

1. every required node for workflow `W` has a ledger-consistent key,
2. all declared outputs exist, and
3. all declared outputs match expected content hashes.

Design target:

1. if `Warm(S, L, W)`, planner executes zero functional units (report/aggregate
   units may still run if policy marks them always-run).

### 4.2 Target Theorems

1. Deterministic planning: identical declared inputs/state/ledger produce identical plan.
2. Minimal execute set: execute set is the least sound set satisfying required outputs.
3. At-most-once execution: each `(WorkIdentity, MaterializationDigest)` executes at most once per run.

## 5. Non-Goals

1. Replacing Make in one step. Make remains a thin command shim.
2. Rewriting all tool DAGs immediately.
3. Distributed remote cache/execution in phase 1.

## 6. Core Model

Introduce a typed workflow spec and execution ledger while reusing the existing
`Dag<T>` model (no parallel graph abstraction).

```rust
pub struct WorkflowSpec {
    pub id: WorkflowId,                // e.g. "ci", "test-all"
    pub dag: Dag<WorkflowUnit>,        // canonical graph semantics
    pub policy_version: u32,
}

pub struct ProcessUnitRef {
    pub process_id: ProcessId,
    pub unit_id: NodeId,
}

pub struct WorkflowUnit {
    pub op: WorkflowOp,                // typed, closed operation enum
    // Effect/resource claims are derived from op + declared resource ports.
    // They are not independently authored fields.
}

pub enum WorkflowOp {
    InvokeProcessUnit(ProcessUnitRef),
    Aggregate(AggregateSpec),
    Report(ReportSpec),
}

pub struct WorkIdentity {
    pub process_id: ProcessId,
    pub unit_id: NodeId,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalKeyPayload {
    pub key_format_version: u32, // canonical encoding schema version
    pub op_version: u32,
    pub input_hashes: BTreeMap<PortName, Vec<ContentHash>>,
    pub upstream_keys: BTreeMap<PortName, Vec<MaterializationDigest>>,
    pub policy_version: u32,
}

pub struct MaterializationKey {
    pub work_id: WorkIdentity,         // context-free identity (no workflow node name)
    pub payload: CanonicalKeyPayload,
    pub digest: MaterializationDigest, // sha256(payload)
}

pub enum MissReason {
    NoPriorRun,
    InputChanged {
        port: PortName,
        old: Vec<ContentHash>,
        new: Vec<ContentHash>,
    },
    UpstreamKeyChanged {
        port: PortName,
        old: Vec<MaterializationDigest>,
        new: Vec<MaterializationDigest>,
    },
    OpVersionChanged { old: u32, new: u32 },
    PolicyVersionChanged { old: u32, new: u32 },
    OutputMissing { port: PortName },
    OutputTampered {
        port: PortName,
        expected: ContentHash,
        actual: ContentHash,
    },
    VolatileEffect { effect: Effect },
}

pub enum LedgerStatus {
    CachedHit { previous_run: RunId },
    Executed { reason: MissReason },
    Failed { reason: MissReason, error: String },
    Skipped { blocked_by: NodeId },
}

pub struct RunLedgerEntry {
    pub exec_node_id: NodeId,          // run-local explainability only
    pub work_id: WorkIdentity,         // global memoization identity
    pub key: MaterializationKey,
    pub status: LedgerStatus,
    pub output_hashes: BTreeMap<PortName, ContentHash>, // includes "result" when dataflow consumers exist
    pub duration_ms: u64,
}
```

`ProcessUnitRef` contract:

1. it resolves via a typed process registry to a `ProcessSpec` unit definition,
2. `CanonicalKeyPayload.op_version` for `InvokeProcessUnit(ProcessUnitRef)` is
   derived from that resolved unit's semantic version/digest,
3. semantic behavior changes in a process unit must change this semantic version/digest.

Model invariants:

1. `WorkflowSpec` wraps `Dag<WorkflowUnit>` and therefore reuses cycle checks,
   typed ports, and `EdgeKind`.
2. Downstream commit/readiness ordering is represented with `EdgeKind::Control`,
   not side tables or Make ordering.
3. `WorkflowOp` remains typed and closed. No shell-string fallback op in the
   steady-state model.
4. Effect/resource behavior is structural: derive from op + resource ports (for
   example via `derive_resource_accesses()`), not manually duplicated fields.
5. Key payload must preserve fan-in cardinality for multi-producer ports.

### 6.1 Global Canonicality (No Parallel Models)

To avoid modeling the same semantics in multiple forms, enforce one authority per
semantic fact:

1. graph topology and ordering live only in `Dag<WorkflowUnit>`,
2. executable unit meaning lives only in typed `WorkflowOp` + referenced process units,
3. resource/effect behavior is derived from typed ports/op class (not side tables),
4. cache causality lives only in `CanonicalKeyPayload` + `MissReason` ADTs,
5. orchestration node names are explainability labels, not cache identity.

Derived surfaces (`Makefile`, CLI wrappers, reports, dashboards) are projections.
They cannot author new dependencies, fallback semantics, or alternate claims.

### 6.2 Cross-Level Composition (Inter + Intra Process)

Three levels are modeled, but only one canonical execution graph is used at run
time:

1. level A: process-local DAG units (owned by process specs),
2. level B: workflow orchestration DAG units (`ci`, `test-all`),
3. level C: planner-resolved global DAG (A and B flattened with typed IDs).

Flattening contract:

1. every `InvokeProcessUnit(ProcessUnitRef)` resolves to typed process-owned units,
2. resolved nodes are normalized to stable typed IDs,
3. nodes with the same context-free `WorkIdentity` and equivalent key payload are
   executed once, regardless of whether they were reached from `ci` or `test-all`,
4. fan-out dependents consume the same committed output/key.

### 6.3 Proof Obligations (Mathematical Guarantees)

Planner validation/proof checks must hold before execution:

1. global DAG acyclicity (`Dag` check over resolved graph),
2. single-writer invariant for each declared output identity:
   concurrent writers are valid only if there is an ordering path between them,
3. at-most-once execution per `(WorkIdentity, MaterializationDigest)` in one run,
4. minimal dirty closure: execute set equals transitive closure of dirty roots,
5. no-ambient-dependency invariant for key computation,
6. projection equivalence: generated wrappers/projected views cannot change planner
   execute-set semantics.

Proof boundary:

1. guarantees are relative to declared resources/inputs/effects,
2. undeclared effectful behavior is a model violation and must fail validation.
3. opaque sub-process execution nodes are not allowed in planner execution DAG;
   flattening is required before scheduling.

## 7. Materialization Key Contract

For each node:

`key = H(op_version, canonical_inputs, upstream_output_keys, policy_version)`

Where:

1. `op_version`: semantic identity of the resolved unit (for process-invocation
   units, derived from resolved `ProcessSpec` semantic version/digest).
2. `canonical_inputs`: hashes from declared, wired input ports only.
3. Variability from env/toolchain must enter via explicit input resources/ports,
   not ambient OS probing inside key computation.
4. `upstream_output_keys`: keyed by consuming input `PortName` (not upstream node
   names), so cross-workflow orchestration naming does not change cache identity.
5. Multi-producer fan-in ports must preserve full contributor sets per port (no
   map overwrite). Contributor vectors are deterministically ordered.
6. `policy_version`: workflow policy/planner version hash.
7. `key_format_version`: canonical payload encoding version.
8. Key serialization is canonical + versioned (stable map ordering, deterministic
   vector ordering, and fixed encoder config).
9. Planning-time key computation is DAG-functional only: declared inputs +
   upstream digests. Planner must not read mutable produced artifacts while
   computing keys.

No mtime-only or ambient-probe keying in planner core.
Explainability comes from diffing `CanonicalKeyPayload` into typed `MissReason`.

### 7.1 Global Ledger Scope (Cross-Workflow Memoization)

Ledger storage is global for the workspace, not partitioned by workflow name:

1. canonical path: `.gunbc/workflow-ledger/global.ndjson` (or versioned equivalent),
2. index key: `(WorkIdentity, MaterializationDigest)`,
3. per-run metadata still records `exec_node_id` for explainability.

This prevents inter-workflow redundancy (`ci` and `test-all`) when the resolved
work identity and inputs are equivalent.

### 7.2 Output Store Contract (For Cached Rehydration)

`RunLedgerEntry.output_hashes` identifies output payloads in a content-addressed
store (CAS). Cached nodes are treated as committed only after output rehydration:

1. on `CachedHit`, planner/executor loads output payloads by hash from CAS,
2. rehydrated outputs are injected on outgoing dataflow edges exactly as if the
   node executed in this run,
3. missing CAS payload for expected hash is a hard miss/failure path, not silent skip.
4. when a node exposes `result` on dataflow edges, the typed `result` payload must
   also be materialized/re-hydratable via `output_hashes["result"]`.
5. strict/minimal default: persist a typed bounded summary/reference as the
   canonical `result` payload.
6. optional policy: additionally persist full typed result payload in CAS for
   selected units where required by diagnostics.

## 8. Planner Algorithm

1. Load `WorkflowSpec`.
2. Resolve/flatten process-unit references into the global typed DAG and deduplicate
   equivalent `WorkIdentity` nodes.
3. Validate single-writer ordering constraints on declared write claims.
4. Load previous global `RunLedger`.
5. Compute current `MaterializationKey` for every node from declared inputs and
   upstream digests only.
6. Mark node dirty if:
   1. no prior entry,
   2. key payload diff yields typed `MissReason`,
   3. any declared output missing (`OutputMissing`),
   4. any declared output hash mismatches ledger (`OutputTampered`),
   5. node effect is non-cacheable (`VolatileEffect`),
   6. prior run failed.
7. Rehydrate cached-hit node outputs from CAS before downstream readiness/dataflow.
8. Compute transitive dirty closure over dependents.
9. Schedule only dirty closure, preserving derived resource claims and concurrency limits.
10. Persist full global `RunLedger` with hit/miss reasons and timings.

Result:

1. No-op warm run executes zero functional nodes and returns quickly.
2. Single-source edits execute minimal downstream closure.

## 9. Executor Semantics

Executor must satisfy all before starting a node:

1. Dependencies committed (execution completed and required outputs/results are available).
2. Required resources available by capacity/claim derived from resource ports.
3. Node admitted by max concurrency budget.

### 9.2 Readiness/Dataflow Axioms

1. Control edges are completion gates (`commit`), not implicit success gates.
2. Readiness requires both:
   1. all required incoming control prerequisites committed, and
   2. all required dataflow inputs materialized (executed or rehydrated).
3. Missing required dataflow input at runtime after validation is an executor
   invariant violation (fail-closed).
4. Success-gated branching is explicit via typed guard units consuming `result`
   (not implicit control-edge semantics in this phase).

Strict default policy:

1. functional units are success-guarded by default,
2. report/aggregation units remain completion-gated for failure completeness.

Execution/reporting semantic split:

1. Domain failures (tests failed, lint findings) are data payloads on `result`
   and still commit, so downstream report/summary nodes run.
2. Execution failures (cannot spawn process, transport crash) are runtime failures
   that fail closed with explicit diagnostics.
3. Domain failures may be cacheable by policy. Default policy is explicit:
   replay cached domain failure or force rerun for selected ops.

### 9.1 Cached Domain Failure Policy

Because domain failures are modeled as committed results, they can be replayed
from cache when inputs are unchanged.

Policy surface:

1. default: `replay-domain-failure` (fast deterministic reruns),
2. override: `--force-run` (disable cache-read for selected/all workflow units),
3. op-level alternative: mark selected units `VolatileEffect` to always execute.

Resource behavior reuses existing `AccessMode` and lock semantics in `gunbc`.

## 10. Strict No-Fallback Policy

Rules:

1. Missing typed input mapping is a hard error.
2. Unknown node/output IDs are hard errors.
3. Unsupported input type at CLI boundary is a hard error.
4. Deprecated path aliases are rejected with explicit migration error.

This removes semantic drift from "best effort" runtime behavior.

## 11. Workflow Surface For Users

Keep existing UX shape:

1. `make ci`
2. `make test-all`

But map to planner entrypoints:

1. `cargo run -p gunbc-dag --bin gunbc-workflow -- ci`
2. `cargo run -p gunbc-dag --bin gunbc-workflow -- test-all`

Make becomes transport only, not scheduler.

## 12. Performance SLOs

Target warm-state SLOs:

1. `make ci` no-op: <= 5s.
2. `make test-all` no-op: <= 10s.
3. Single-file non-generator edit to unit-test completion: <= 30s median.

Planner must emit:

1. total nodes,
2. cache hits,
3. executed nodes,
4. critical path duration,
5. top N slow nodes,
6. miss reason histogram.

## 13. Migration Plan

### Phase A: Model Introduction

1. Add `WorkflowSpec` (wrapping `Dag<WorkflowUnit>`) and ADT `RunLedger` types.
2. Add deterministic key payload computation + digest library.
3. Add atomic ledger persistence contract (temp file + fsync + rename).
4. Add planner-only dry-run command to show execute set and typed reasons.

### Phase B: CI/Test-All Port

1. Model `ci` and `test-all` as specs.
2. Bind existing tool DAG invocations as typed `WorkflowOp` variants.
3. Keep behavior parity with current commands.

### Phase C: Remove Redundant Orchestration

1. Make `make ci` and `make test-all` call planner entrypoints.
2. Delete duplicated dependency chains for these targets from Make.
3. Keep legacy targets as wrappers only.

### Phase D: Hard Fail On Legacy Paths

1. Remove fallback orchestration hooks.
2. Enforce strict planner contract in CI.
3. Add regression tests for no-redundancy invariants.

## 14. Verification Plan

Must-pass checks:

1. Determinism: same tree + env + toolchain => identical plan and keys.
2. Soundness: changed input always invalidates required downstream nodes.
3. Minimality: unchanged graph executes zero functional nodes.
4. Safety: dependency/resource violations fail closed.
5. UX parity: command outputs still map to existing user workflows.

## 15. Extended Scope: All Tool Workflows

### 15.1 Motivation

Sections 1-14 define the minimal execution model for `ci` and `test-all`. The same model
must extend to **all** tool workflows in the workspace. Every tool workflow decomposes
into a small set of **capability requirements** — credentialing, network upload, filesystem
access, git state queries, codegen freshness, compilation, and pure computation. If each
capability is modeled and minimized correctly, redundant work is eliminated by construction
and performance follows.

### 15.2 Capability Taxonomy

Every tool workflow in the workspace draws from these capabilities. Each capability has
a natural minimization contract — a statement of the minimum work required for correctness.

#### 1. Git State

**What it provides**: branch name, HEAD ref, tree listing, diffs, rev-lists.

**Consumers**: gist (all modes), dag-viz (all modes), review, CI.

**Minimization contract**: Git state is a deterministic function of `.git` object store
contents. A git query's output is fully determined by its inputs (ref names, index state,
object hashes). Two invocations with identical git state must produce identical output.

| Query | Deterministic Input | Invalidation Signal |
|---|---|---|
| `git.CurrentBranch` | `.git/HEAD` content | HEAD changes (checkout, commit, rebase) |
| `git.LsFiles` | `.git/index` content hash | index changes (add, rm, checkout) |
| `git.Diff(base, head)` | `base` ref hash + `head` ref hash | either ref moves |
| `git.RevList(since)` | object store + `since` boundary | new commits within window |

**Current deficiency**: Every invocation re-runs every git query unconditionally.
No query result is retained or checked against prior state.

**Minimal model**: Each git query is a keyed unit. Key payload includes the
deterministic inputs above. Planner skips the query when inputs haven't changed.

#### 2. Filesystem Access

**What it provides**: file listing, file content reads, file writes (upsert pattern).

**Consumers**: gist-snapshot (read), bootstrap/makegen/pragma/testgen (read-compare-write),
deps (read manifest), codegen (write generated files).

**Minimization contract**: File reads are deterministic functions of file content hashes.
File writes (upsert pattern) are conditional: skip when generated content matches existing
file content.

| Operation | Deterministic Input | Invalidation Signal |
|---|---|---|
| Read file(s) | content hash per file | file content changes on disk |
| Write/upsert | generated content hash vs. existing content hash | generated content differs from existing |

**Current deficiency**: Snapshot mode re-reads every file on every invocation. Upsert
chains (bootstrap, makegen, pragma, testgen) do compare-before-write internally, but
the entire chain still runs from scratch including the generation step.

**Minimal model**: File content hashes are keyed inputs. Read nodes skip when content
hashes haven't changed. Upsert chains skip entirely (including generation) when the
generation inputs haven't changed — the content comparison is a consequence of keying,
not a substitute for it.

#### 3. Credentialing

**What it provides**: authenticated tokens for GitHub API, GCP services, LLM providers.

**Consumers**: gist (all modes), dag-viz (upload modes), review, LLM chat.

**Minimization contract**: Credential resolution is a function of:
1. runtime mode (`LocalDev` vs cloud),
2. credential source (env var, WIF chain, Secret Manager), and
3. token validity window (expiry timestamp).

Within a validity window, the resolved credential is immutable. Re-resolution
produces the same token until expiry or source change.

| Resolution Path | Inputs | Validity |
|---|---|---|
| Local env var (`GITHUB_TOKEN`) | env var content hash | until env changes |
| Cloud WIF chain | WIF provider + service account + audience | until token expiry (typically 1h) |
| GCP Secret Manager | project + secret name + WIF token | until secret version changes |
| LLM provider env var (`ANTHROPIC_API_KEY`) | env var content hash | until env changes |

**Current deficiency**: Every invocation re-runs the full credential chain from scratch.
`gist-recent` forces `GUNBC_CLOUD_CONFIG_REQUIRED=1`, triggering the entire WIF → OIDC →
Secret Manager chain on every call. Local-dev mode with `GITHUB_TOKEN` set still enters
the `credential_chain()` function and performs runtime detection.

**Minimal model**: Credential resolution is a keyed unit. Key payload includes runtime
mode + source identity + validity boundary. Within the validity window, the planner
resolves credentials from ledger (zero network calls). The local-dev path (env var
present) is the simplest case: key on env var hash, no cloud calls by construction.
The `GUNBC_CLOUD_CONFIG_REQUIRED=1` ambient env probe is replaced by an explicit
`runtime_mode` input port on the credential unit.

#### 4. Network Transport (Upload/API)

**What it provides**: HTTP requests to external services (GitHub Gist API, LLM APIs).

**Consumers**: gist (all modes), dag-viz (upload modes), review, LLM chat.

**Minimization contract**: Network transport is inherently effectful. Each call produces
a new external side effect (new gist, new API response). However, the *inputs* to the
transport are deterministic functions of upstream capability outputs.

| Transport | Inputs | Effect | Idempotency |
|---|---|---|---|
| `github.Gist.Create` | rendered markdown + credential + metadata | creates new gist | not idempotent (new gist each call) |
| LLM chat completion | prompt + credential + model config | new inference | not idempotent (non-deterministic) |

**Current deficiency**: Transport nodes re-execute unconditionally, but so does every
upstream node that produces the transport inputs. The transport itself is correctly
volatile, but its inputs are re-derived unnecessarily.

**Minimal model**: Transport nodes are marked `VolatileEffect` — always in the execute
set. But all upstream nodes that produce transport inputs (rendering, content collection,
credential resolution) are keyed and skip when their inputs are unchanged. The transport
node receives cached upstream outputs, so the only actual work is the HTTP call itself.

#### 5. Codegen Freshness

**What it provides**: generated CLI entrypoints, type stubs, and tool registrations.

**Consumers**: every tool target (via `ensure-codegen` Make prerequisite).

**Minimization contract**: Codegen output is a deterministic function of DSL source
files + codegen binary logic. If neither has changed, outputs are identical.

| Input | Source |
|---|---|
| DSL source files | `dsl/**/*.dag` content hashes |
| Codegen binary semantics | `gunbc-codegen` binary hash or semantic version |

**Current deficiency**: `ensure-codegen` runs `cargo run -p gunbc-dag --bin gunbc-codegen`
as a Make prerequisite for every tool target. This spawns a full Cargo compilation check
+ binary execution even when nothing has changed. Every `make gist`, `make bootstrap`,
`make deps`, etc. pays this cost.

**Minimal model**: Codegen is a keyed workflow unit in the planner. Key payload includes
DSL source content hashes + binary semantic version. Tool workflows declare codegen
freshness as a typed input dependency (not a Make ordering prerequisite). Planner
resolves codegen freshness via ledger lookup — no subprocess, no Cargo check.

#### 6. Compilation / Binary Dispatch

**What it provides**: built Rust binaries for tool execution.

**Consumers**: every `make <tool>` target (via `cargo run`).

**Minimization contract**: A compiled binary is a deterministic function of workspace
source files + Cargo dependency metadata + compiler version. If none have changed,
the existing binary is valid.

**Current deficiency**: Every `cargo run -p <pkg> --bin <bin>` invocation checks
compilation freshness of the entire workspace dependency tree. This is Cargo's
internal memoization, but it still costs 5-15s per invocation for a medium workspace.
Stacking `ensure-codegen` (one `cargo run`) + the tool itself (another `cargo run`)
means two full Cargo checks per tool invocation.

**Minimal model**: Binary freshness is a keyed unit. Key payload includes source
content hashes + `cargo metadata` dependency hashes + compiler version. Tool
invocation dispatches to the pre-built binary directly (via `build-release-bins`
or planner-managed binary path), bypassing `cargo run` entirely.

#### 7. Pure Computation

**What it provides**: markdown rendering, diff formatting, content aggregation,
template expansion.

**Consumers**: gist (render_snapshot/render_diff/render_recent), bootstrap
(GenerateMakefile/GenerateGitignore), makegen (RenderMakefile), pragma
(RenderClippy/RenderAllowlist/RenderPolicy), testgen (Generate_{name}).

**Minimization contract**: Pure functions are deterministic by definition. Same
inputs always produce same outputs. These are always cacheable.

**Current deficiency**: Every invocation re-runs every pure computation node, even
when inputs are identical to the last run.

**Minimal model**: Pure nodes are keyed on their input hashes. Always skip on
cache hit. No special treatment needed — the general keying model handles this.

### 15.3 Workflow-to-Capability Matrix

Which workflows require which capabilities:

| Workflow | Git | FS Read | FS Write | Credential | Network | Codegen | Compilation | Pure |
|---|---|---|---|---|---|---|---|---|
| **gist (snapshot)** | branch, ls-files | file contents | — | GitHub token | gist upload | yes | yes | render_snapshot |
| **gist (diff)** | branch, diff | — | — | GitHub token | gist upload | yes | yes | render_diff |
| **gist (recent)** | branch, rev-list, diff | — | — | GitHub token (cloud) | gist upload | yes | yes | render_recent |
| **dag-viz** | branch | — | — | GitHub token | gist upload | yes | yes | viz render |
| **dag-viz (diff)** | branch, diff | — | — | GitHub token | gist upload | yes | yes | viz render |
| **dag-viz (recent)** | branch, rev-list, diff | — | — | GitHub token (cloud) | gist upload | yes | yes | viz render |
| **dag-snapshot** | — | — | — | — | — | yes | yes | dag serialize |
| **bootstrap** | — | existing files | Makefile, .gitignore | — | — | yes | yes | generate |
| **makegen** | — | existing Makefile | Makefile | — | — | yes | yes | render |
| **pragma** | — | existing files | clippy.toml, allowlist, policy | — | — | yes | yes | render x3 |
| **testgen** | — | existing test files | test files x N | — | — | yes | yes | generate x N |
| **deps** | — | manifest | — | — | — | yes | yes | resolve, render |
| **build** | — | — | — | — | — | yes | yes | — |
| **clippy** | — | — | — | — | — | yes | yes | — |
| **review** | diff | — | — | LLM API key | LLM API | yes | yes | prompt, parse |
| **ci** | — | — | — | — | — | yes | yes | orchestration |
| **test-all** | — | — | — | — | — | yes | yes | orchestration |

Key observations:

1. **Codegen + Compilation are universal** — every workflow pays this cost. Minimizing
   these two capabilities has the highest leverage.
2. **Credentialing is shared** across gist, dag-viz (upload modes), and review — a
   single minimized credential unit serves all of them.
3. **Git state is shared** across gist and dag-viz mode families — identical queries
   with identical repo state should resolve from the same keyed unit.
4. **FS Write (upsert)** is only used by generator workflows (bootstrap, makegen,
   pragma, testgen) — all share the same content-upsert pattern.
5. **Network transport** is the only inherently volatile capability — everything
   upstream of it should be cacheable.

### 15.4 Per-Capability Minimization Applied to Gist

The gist family exercises five of the seven capabilities and is the primary focus
target. The three modes (snapshot, diff, recent) share a **base gist workflow** — the
DSL already factors this via `shared.gist_modes`. The design must preserve and
formalize this structure: a base workflow that handles credentialing, branch context,
and upload, composed with mode-specific content acquisition.

#### Base Gist Workflow (shared by all modes)

The base workflow is the invariant core. Every gist mode flows through it:

```
content acquisition (mode-specific)
        │
        ▼ markdown: String
┌──────────────────────────────────────────────┐
│  Base Gist Workflow                          │
│                                              │
│  branch_context()  ──────────┐               │
│                               ▼               │
│  credential.resolve  ───→  gist_upload()     │
│                               │               │
│                               ▼               │
│                        github.Gist.Create    │
└──────────────────────────────────────────────┘
        │
        ▼ url: Url
```

DSL source: `shared.gist_modes.share_content` → `gist_upload` → `credential_chain`
→ `github.Gist.Create`.

Base capability units (shared across all modes):

```
Capability        Unit                    Key Inputs                     Skip When
─────────────────────────────────────────────────────────────────────────────────────
Codegen           codegen.ensure          DSL hashes + binary version    DSL unchanged
Compilation       binary.ensure           source hashes + cargo meta     source unchanged
Git State         git.current_branch      .git/HEAD                      HEAD unchanged
Credential        credential.resolve      runtime_mode + source hash     within validity
Network           github.gist_create      markdown hash + credential     never (volatile)
```

These five units are identical across snapshot, diff, and recent. The global ledger
means running `make gist` and then `make gist-diff` reuses the base units (codegen,
compilation, branch context, credential) from the first invocation.

#### Mode-Specific Content Acquisition (augments base)

Each mode adds its own content acquisition capabilities upstream of the base workflow's
`markdown` input:

**Snapshot** — augments base with filesystem scan:

```
Capability        Unit                    Key Inputs                     Skip When
─────────────────────────────────────────────────────────────────────────────────────
Git State         git.ls_files            .git/index hash                index unchanged
FS Read           fs.read_files           file content hashes            files unchanged
Pure              gist.render_snapshot    upstream content hashes        inputs unchanged
```

DSL source: `gist_snapshot` → `git.LsFiles` → `read_text_files` → `render_snapshot`.

**Diff** — augments base with git diff:

```
Capability        Unit                    Key Inputs                     Skip When
─────────────────────────────────────────────────────────────────────────────────────
Git State         git.diff(base, HEAD)    base ref hash + HEAD hash      neither ref moved
Pure              gist.render_diff        upstream diff hash             inputs unchanged
```

DSL source: `gist_diff` → `git.Diff(base_ref)` → `render_diff`.

**Recent** — augments base with rev-list + per-commit diffs + cloud credential path:

```
Capability        Unit                    Key Inputs                     Skip When
─────────────────────────────────────────────────────────────────────────────────────
Git State         git.rev_list(since)     object store + since boundary  no new commits
Git State         git.diff(per commit)    commit hash + parent hash      refs unchanged
Pure              gist.render_recent      upstream diff hashes           inputs unchanged
Credential        credential.resolve      runtime_mode: Cloud + WIF cfg within TTL
  (cloud override)  ├── gcp.sts_exchange  WIF provider + OIDC token      within token TTL
                    ├── gcp.iam_impersonate  SA + STS token              within token TTL
                    └── gcp.secret_access  project + secret + IAM token  within token TTL
```

DSL source: `gist_recent` → `resolve_recent_base` → `git.RevList` → per-commit
`git.Diff` → `render_recent`. The credential unit is the same base unit but with
`runtime_mode: Cloud` input (replacing `GUNBC_CLOUD_CONFIG_REQUIRED=1`), which
triggers the WIF sub-chain.

#### Composition Summary

```
gist-snapshot = base gist + [git.ls_files, fs.read_files, render_snapshot]
gist-diff     = base gist + [git.diff, render_diff]
gist-recent   = base gist + [git.rev_list, git.diff×N, render_recent]
                           + credential.resolve(Cloud) override
```

The base workflow is not a separate `WorkflowSpec` — it's the shared set of capability
units that appear identically in all three modes. The global ledger's `WorkIdentity`-based
dedup (Section 6.2) means these shared units are executed once and reused across modes
without explicit cross-workflow wiring. dag-viz modes share the same base via
`shared.gist_modes`.

### 15.5 Full Tool Workflow Inventory

The planner model must cover all workflow families from the DAG audit
(`docs/dag-workflow-audit.md`):

| # | Workflow Family | Make Targets | DAG Source | Nodes | Mode Variants |
|---|---|---|---|---|---|
| 1 | Codegen | `codegen`, `ensure-codegen` | `gunbc-dag/src/codegen/graph.rs` | 8 | — |
| 2 | Bootstrap | `bootstrap`, `bootstrap-dry` | `gunbc-dag/src/bootstrap/graph.rs` | 15 | — |
| 3 | Build | `build-all`, `build-all-dry` | `gunbc-dag/src/build/graph.rs` | 10 | — |
| 4 | Makegen | `makegen`, `makegen-dry` | `gunbc-dag/src/makegen/graph.rs` | 7 | — |
| 5 | CI | `ci`, `ci-dry` | `gunbc-dag/src/ci/graph.rs` | many | — |
| 6 | Pragma | `pragma`, `pragma-dry` | `gunbc-dag/src/pragma/graph.rs` | 18 | — |
| 7 | Testgen | `testgen`, `testgen-check` | `gunbc-dag/src/testgen_dag/graph.rs` | Nx6 | — |
| 8 | **Gist** | `gist`, `gist-diff`, `gist-recent` (+dry) | `lib/tools/gist/src/graph.rs` | 17 | snapshot, diff, recent |
| 9 | Deps | `deps`, `deps-dry` | `lib/tools/deps/src/graph.rs` | 8 | install, generate |
| 10 | Clippy | `clippy`, `clippy-fix` | `lib/tools/clippy/src/graph.rs` | 3 | — |
| 11 | Review | (via `gunbc review`) | `lib/review/src/graph.rs` | 10-12 | phase, inline, diff, multi |
| 12 | LLM Chat | (embedded in review) | `lib/llm-ops/src/graph.rs` | 5 | — |
| 13 | DAG Viz | `dag-viz`, `dag-viz-diff`, `dag-viz-recent` (+dry) | shared gist_modes pattern | varies | snapshot, diff, recent |
| 14 | DAG Snapshot | `dag-snapshot`, `dag-snapshot-dry` | snapshot mode | varies | — |

### 15.6 Cross-Workflow Capability Sharing

The capability matrix (15.3) reveals that capabilities are shared across workflows.
The global ledger (Section 7.1) enables this sharing: identical capability units with
identical key payloads resolve from the same ledger entry regardless of which workflow
triggered them.

Concrete sharing relationships:

```
codegen.ensure          ← shared by ALL workflows (universal prerequisite)
binary.ensure           ← shared by ALL workflows (universal prerequisite)
git.current_branch      ← shared by gist (all), dag-viz (all)
git.ls_files            ← shared by gist-snapshot
git.diff(base, head)    ← shared by gist-diff, dag-viz-diff, review
git.rev_list(since)     ← shared by gist-recent, dag-viz-recent
credential.resolve(gh)  ← shared by gist (all), dag-viz (upload modes)
credential.resolve(llm) ← shared by review, LLM chat
```

This means: if `make gist-diff` has already resolved credentials and computed a diff,
then `make dag-viz-diff` with the same `base_ref` reuses both from the ledger. The
sharing is a consequence of context-free `WorkIdentity` (Section 6.2) — it requires
no explicit cross-workflow wiring.

### 15.7 Migration Approach

Minimization is applied per-capability, not per-workflow. Each capability gets a
keyed unit model, and workflows compose those units. The migration order follows
leverage (universal capabilities first):

1. **Codegen + Compilation** (universal) — eliminates the two capabilities that every
   single workflow pays for today. Highest leverage.
2. **Credentialing** — eliminates the most expensive single-capability cost (WIF chain).
   Unblocks fast gist/dag-viz/review invocations.
3. **Git State** — eliminates redundant git queries across gist/dag-viz/review.
4. **Filesystem** — eliminates redundant reads/upsert-checks in generator workflows.
5. **Pure Computation** — falls out naturally from general keying (no special work).
6. **Network Transport** — already correctly volatile; upstream minimization is the win.

### 15.8 Tool Workflow SLOs

SLOs are a verification mechanism for capability minimization, not the design goal
themselves. If each capability is minimized correctly, these budgets are satisfied
by construction:

| Target | Warm No-Op Budget | Notes |
|---|---|---|
| `make gist` / `gist-diff` / `gist-recent` | planner + ledger + upload | upload is the only real work |
| `make dag-viz` / variants | planner + ledger + upload | same pattern as gist |
| `make bootstrap` / `makegen` / `pragma` | planner + ledger | zero work on warm (upsert skips) |
| `make deps` | planner + ledger | zero work if manifest unchanged |
| `make build-all` | planner + ledger + cargo (if dirty) | cargo build time is external |
| `make ci` / `test-all` | ≤5s / ≤10s | already scoped in Section 12 |

## 16. Immediate Next Design Artifacts

1. `workflow_spec.rs`: `WorkflowSpec { dag: Dag<WorkflowUnit> }` for `ci`, `test-all`,
   and all tool workflows.
2. `workflow_key.rs`: canonical key payload normalization and hashing.
3. `workflow_planner.rs`: dirty-closure + scheduling plan with typed miss reasons.
4. `workflow_ledger.rs`: stable on-disk format, versioning policy, atomic persistence.
5. `docs/design/workflow-key-schema.md`: concrete key fields and miss reasons.
6. `docs/design/workflow/tool-workflow-design-pack.md`: concrete DAG views for gist,
   deps, bootstrap, makegen, pragma, dag-viz (analogous to `wf1-wf4-dag-design-pack.md`).

---

This design keeps the current DAG/resource architecture, but moves orchestration from imperative Make composition into a typed planner with explicit keys, explicit invalidation, and strict fail-closed semantics.
