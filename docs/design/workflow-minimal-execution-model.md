# Workflow Minimal Execution Model

Status: Draft
Owner: `gunbc` workflow/runtime
Scope: `make ci`, `make test-all`, and shared generation/lint/build/test orchestration

## 1. Problem Statement

`gunbc` has the right low-level primitives (typed DAGs, resource manifests, effect metadata), but top-level workflow execution is still modeled as imperative target chaining.

Current consequences:

1. Redundant compile/generator work across `build`, `verify-fix`, and test targets.
2. Repeated freshness checks and repeated tool startup overhead in one command.
3. Runtime behavior encoded in Make target composition instead of a typed execution model.
4. Slow no-op and warm-path commands despite a medium-sized repo.

User-facing requirement:

1. Commands should usually return in seconds on warm state.
2. No hidden fallback/deprecated path behavior.
3. Workflows should naturally avoid redundant work by construction.

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
3. Add changed-input routing as a first-class planner input, not ad-hoc Make glue.

## 4. Design Goals

1. Single planner computes minimal executable frontier once per run.
2. Deterministic materialization keys per workflow node.
3. Explicit invalidation causes (input/content/env/toolchain/policy) recorded in run ledger.
4. Zero implicit fallback paths.
5. Stable latency targets with observable SLOs.

## 5. Non-Goals

1. Replacing Make in one step. Make remains a thin command shim.
2. Rewriting all tool DAGs immediately.
3. Distributed remote cache/execution in phase 1.

## 6. Core Model

Introduce a typed workflow spec and execution ledger.

```rust
pub struct WorkflowSpec {
    pub id: String,                    // e.g. "ci", "test-all"
    pub nodes: Vec<WorkflowNode>,      // topologically sortable
    pub edges: Vec<WorkflowEdge>,
}

pub struct WorkflowNode {
    pub id: String,                    // stable ID, no dynamic string synthesis
    pub op: WorkflowOp,                // typed operation enum
    pub effect: Effect,                // PURE/READ/WRITE_DETERMINISTIC/WRITE
    pub claims: Vec<ResourceClaim>,    // resource concurrency constraints
    pub inputs: Vec<NodeInput>,        // typed dependency set for keying
    pub outputs: Vec<NodeOutput>,      // declared artifacts
}

pub struct MaterializationKey {
    pub node_id: String,
    pub digest: String,                // sha256 of normalized key payload
}

pub struct RunLedgerEntry {
    pub node_id: String,
    pub key: String,
    pub status: NodeStatus,            // hit/miss/executed/failed/skipped
    pub reason: Option<MissReason>,    // why work executed
    pub duration_ms: u64,
}
```

`WorkflowOp` must be typed and closed. No generic shell-string fallback operation in the steady-state model.

## 7. Materialization Key Contract

For each node:

`key = H(op_version, declared_inputs, upstream_output_keys, env_projection, toolchain_fingerprint, policy_version)`

Where:

1. `declared_inputs`: content hashes from `ResourceDef`/file sets/type-stable params.
2. `upstream_output_keys`: keys of producing nodes, not timestamps.
3. `env_projection`: explicit allowlist (`RUSTFLAGS`, selected auth/runtime flags, etc.).
4. `toolchain_fingerprint`: `rustc --version`, cargo lock digest, relevant build profile.
5. `policy_version`: workflow policy and planner version hash.

No mtime-only keys in the planner core.

## 8. Planner Algorithm

1. Load `WorkflowSpec`.
2. Load previous `RunLedger`.
3. Compute current `MaterializationKey` for every node.
4. Mark node dirty if:
   1. no prior entry,
   2. key differs,
   3. any declared output missing,
   4. prior run failed.
5. Compute transitive dirty closure over dependents.
6. Schedule only dirty closure, preserving resource claims and concurrency limits.
7. Persist full `RunLedger` with hit/miss reasons and timings.

Result:

1. No-op warm run executes zero functional nodes and returns quickly.
2. Single-source edits execute minimal downstream closure.

## 9. Executor Semantics

Executor must satisfy all before starting a node:

1. Dependencies succeeded.
2. Required resources available by capacity/claim.
3. Node admitted by max concurrency budget.

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

1. Add `WorkflowSpec` and `RunLedger` types.
2. Add deterministic key computation library.
3. Add planner-only dry-run command to show execute set and reasons.

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

## 15. Immediate Next Design Artifacts

1. `workflow_spec.rs`: first-class schema for `ci` and `test-all`.
2. `workflow_key.rs`: canonical key payload normalization and hashing.
3. `workflow_planner.rs`: dirty-closure + scheduling plan.
4. `workflow_ledger.rs`: stable on-disk format and versioning policy.
5. `docs/design/workflow-key-schema.md`: concrete key fields and miss reasons.

---

This design keeps the current DAG/resource architecture, but moves orchestration from imperative Make composition into a typed planner with explicit keys, explicit invalidation, and strict fail-closed semantics.
