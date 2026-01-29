# TODO — gunbc Roadmap

**Last updated**: January 2026

This TODO is derived from reconciling gunbc with the-gunbai (theory/spec) and
gunb.ai (runtime patterns). See `docs/design/reconciliation.md` for context.

---

## Philosophy

gunbc is the **compiler** that should eventually subsume the runtime capabilities
of gunb.ai. The approach:

1. **Now**: Get DAG/behavioral modeling rock-solid at compile time
2. **Later**: Add JIT execution for dynamic node addition

If dependencies are correctly modeled in the DAG, parallelization is free —
it falls out of the structure (like buck2).

---

## Phase 1: Contracts & Validation

The foundation. Catch wiring errors at compile time, not runtime.

### 1.1 Provides/Requires Contract System
**Status**: Not started  
**Priority**: High

Add explicit contract declarations to nodes:

```rust
struct NodeContract {
    provides: Vec<PrerequisiteId>,   // What this node establishes
    requires: Vec<PrerequisiteId>,   // What must exist before running
}
```

Tasks:
- [ ] Define `PrerequisiteId` type with namespaces (`data:`, `cap:`, `state:`)
- [ ] Add `contract: Option<NodeContract>` to `Node<T>`
- [ ] Implement contract validation pass in compiler
- [ ] Verify all `requires` are satisfied by prior nodes' `provides`
- [ ] Add contract inference from edge structure (optional convenience)

### 1.2 Full Validation Pipeline
**Status**: ✅ Complete  
**Priority**: High

Validation pipeline implemented in `gunbc-ir/src/validate.rs`:

- [x] Acyclicity check (cycle detection with path reporting)
- [x] Type agreement on edges
- [x] Port saturation (`check_port_saturation_lowered` for post-lowering)
- [x] Sub-DAG interface agreement (boundary ports match)
- [x] Duplicate node ID detection
- [x] Edge reference validation (node/port existence)
- [ ] Guard completeness (Skipped propagation) - deferred
- [ ] Contract satisfaction (Provides/Requires) - see 1.1
- [ ] Pattern conformance (slots bound correctly) - deferred

### 1.3 Structured Validation Errors
**Status**: ✅ Complete  
**Priority**: Medium

Implemented in `gunbc-ir/src/validate.rs`:

```rust
enum ValidationError {
    TypeMismatch { from_node, from_port, to_node, to_port, expected, actual },
    CycleDetected { nodes: Vec<String> },
    UnconnectedInput { node, port },
    DuplicateNodeId(String),
    NodeNotFound(String),
    PortNotFound { node, port },
    SubDagInterfaceMismatch { node, port },
}
```

---

## Phase 2: Effect & Property Classification

Make node properties explicit so the executor knows what's safe.

### 2.1 Effect Classification
**Status**: Not started (structural boundary detection exists)  
**Priority**: High

Add explicit effect bits to nodes:

```rust
enum Effect {
    Pure,       // No external I/O, deterministic
    Read,       // Reads external state
    Write,      // Modifies external state
}
```

Tasks:
- [ ] Define `Effect` enum
- [ ] Add `effect: Effect` to opaque node metadata
- [ ] Derive effect for SubDag nodes (union of children)
- [ ] Use in executor for parallel safety decisions

### 2.2 Idempotency Classification
**Status**: Implicit in Upsert pattern  
**Priority**: High

Make idempotency explicit:

```rust
enum Idempotency {
    Idempotent,           // Safe to retry
    IdempotentWithKey,    // Safe if key unchanged
    NotIdempotent,        // Not safe to retry
}
```

Tasks:
- [ ] Define `Idempotency` enum
- [ ] Add to node metadata
- [ ] Use for retry logic in executor
- [ ] Validate Upsert pattern nodes have correct idempotency

### 2.3 Property Claims with Verification
**Status**: Not started  
**Priority**: Medium

From the-gunbai V2 spec: every claim needs verification binding.

```rust
struct PropertyClaim {
    property: Property,
    verified_by: VerificationStrategy,
}

enum VerificationStrategy {
    Test(TestRef),
    StaticAnalysis,
    Assumed { reason: &'static str },
}
```

Tasks:
- [ ] Design PropertyClaim system
- [ ] Integrate with testgen for automatic test generation
- [ ] Add verification status tracking

---

## Phase 3: Executor Model

Replace batch "waves" with continuous work-queue execution.

### 3.1 Work-Queue Executor
**Status**: Current executor is sequential  
**Priority**: High

The executor should:
1. Track ready set (all dependencies satisfied)
2. Execute ready nodes (parallel, as many as resources allow)
3. On completion, update ready set
4. Repeat until done

```rust
struct Executor {
    pending: HashSet<NodeId>,
    ready: VecDeque<NodeId>,
    running: HashSet<NodeId>,
    completed: HashMap<NodeId, NodeResult>,
}
```

Tasks:
- [ ] Design executor interface
- [ ] Implement ready-set tracking with dependency graph
- [ ] Add async execution (tokio)
- [ ] Implement completion callbacks that update ready set
- [ ] Add resource limits (max concurrent nodes)

### 3.2 Parallel-Aware Codegen
**Status**: Not started  
**Priority**: Medium

Codegen should emit code that leverages the work-queue model:

Tasks:
- [ ] Emit DAG structure (nodes + edges)
- [ ] Emit executor instantiation with the DAG
- [ ] Generated CLIs use async runtime

### 3.3 Progress & Observability
**Status**: Not started  
**Priority**: Medium

From gunb.ai/the-gunbai: progress events for observability.

```rust
enum ProgressEvent {
    RunStarted { dag_id: DagId },
    NodeQueued { node: NodeId },
    NodeRunning { node: NodeId },
    NodeCompleted { node: NodeId, duration_ms: u64 },
    NodeFailed { node: NodeId, error: String },
    RunCompleted { duration_ms: u64 },
}
```

Tasks:
- [ ] Define event types
- [ ] Add event sink to executor
- [ ] Implement terminal renderer (like current progress display)

---

## Phase 4: Type System Evolution

**Decision**: Keep types simple. Use testing for semantic correctness.

### 4.1 Type Registry with Lanes
**Status**: ❌ Dropped  
**Reason**: Prefer testing over type discrimination. `TypeId(String)` is sufficient.

### 4.2 Coarse "ish" Types
**Status**: ❌ Dropped  
**Reason**: Same as above - test semantic correctness, don't over-type.

**Current approach**:
- `TypeId(String)` for identity
- Validation catches `"String" != "Int"` mismatches
- Tests validate semantic correctness

---

## Phase 5: JIT & Dynamic Execution

For when we need to add nodes at runtime.

### 5.1 JIT Executor Design
**Status**: Not started  
**Priority**: Future

The current model: DAG → lower → codegen → compile → execute (AOT)

JIT model: DAG → lower → interpret directly at runtime

Tasks:
- [ ] Design JIT interpreter interface
- [ ] Implement bytecode/IR format for fast interpretation
- [ ] Add hot-path compilation for frequently-run subgraphs
- [ ] Support dynamic node insertion

### 5.2 Dynamic Subgraph Execution
**Status**: Mentioned in SPEC.md §7  
**Priority**: Future

A node may produce a `Dag<T>` as output, which the executor inlines and runs.

Tasks:
- [ ] Implement DAG-as-output type
- [ ] Add executor support for inline expansion
- [ ] Handle loss of static analysis for dynamic subgraphs

---

## Phase 6: Integrations (from gunb.ai)

Eventually port gunb.ai's integration capabilities as gunbc primitives.

### 6.1 Cursor Integration
**Status**: Not started  
**Priority**: Future

Tasks:
- [ ] Design Cursor ops as primitives
- [ ] `cursor_apply_change` - apply code changes
- [ ] `cursor_run_tests` - run tests in workspace
- [ ] Add approval gate support

### 6.2 GitHub Integration
**Status**: Not started  
**Priority**: Future

Tasks:
- [ ] `github_fetch_pr_diff` - read PR diff
- [ ] `github_open_pr` - create pull request
- [ ] `github_add_comment` - comment on PR

### 6.3 Approval Gates
**Status**: Not started  
**Priority**: Future

From gunb.ai: pause execution, await human decision.

```rust
enum GateKind {
    Soft,  // Warning, continue anyway
    Hard,  // Must approve to continue
}
```

---

## Completed

- [x] Core IR types (Node, Dag, Edge, Port, Value, Guard)
- [x] Boundary and entrypoint detection
- [x] Pattern builders (Upsert, Atomic, Transaction, Loop, Branch)
- [x] Sequential execution engine
- [x] Dry-run via boundary interception
- [x] Transport layer types (Shell, File, Http, etc.)
- [x] CLI codegen
- [x] DAG codegen
- [x] Tools: gist, buck2, viz, makegen, deps, ci, bootstrap
- [x] **SubDag boundary wiring** in lowering (edges rewired to inner nodes)
- [x] **Full validation pipeline** (types, cycles, SubDag interface, duplicates)
- [x] **Structured validation errors** (typed error enum)

---

## Design Documents

- `docs/design/reconciliation.md` - Relationship between gunbc/the-gunbai/gunb.ai
- `docs/design/executor-model.md` - Work-queue executor design
- `docs/design/contracts.md` - Provides/Requires contract system (TODO)
