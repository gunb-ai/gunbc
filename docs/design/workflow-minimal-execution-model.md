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
5. if full result payload persistence is undesirable, store a typed bounded
   summary/reference as the canonical `result` payload and keep that stable.

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

## 15. Immediate Next Design Artifacts

1. `workflow_spec.rs`: `WorkflowSpec { dag: Dag<WorkflowUnit> }` for `ci` and `test-all`.
2. `workflow_key.rs`: canonical key payload normalization and hashing.
3. `workflow_planner.rs`: dirty-closure + scheduling plan with typed miss reasons.
4. `workflow_ledger.rs`: stable on-disk format, versioning policy, atomic persistence.
5. `docs/design/workflow-key-schema.md`: concrete key fields and miss reasons.

---

This design keeps the current DAG/resource architecture, but moves orchestration from imperative Make composition into a typed planner with explicit keys, explicit invalidation, and strict fail-closed semantics.
