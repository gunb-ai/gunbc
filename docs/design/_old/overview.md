# gunbc Design Overview

> **Goal**: Compile-time proof of workflow correctness. If it compiles, it works.

## Philosophy

### Hierarchy of Guarantees (Strongest to Weakest)

| Level | Method | When | Example |
|-------|--------|------|---------|
| **1. Impossible by Structure** | Type system prevents invalid states | Compile | Can't wire `String` to `Int` |
| **2. Impossible by Generation** | Code generation only produces valid code | Build | Generated CLI always has correct args |
| **3. Validated at Compile** | Explicit checks during compilation | Build | `validate_dag()` catches cardinality mismatches |
| **4. Validated at Runtime** | Checks during execution | Run | Mock spec constraint checking |
| **5. Tested** | Unit/integration tests | Test | Boundary interception works |

**Preference**: 1 > 2 > 3 > 4 > 5. We push guarantees as early as possible.

---

## What We Have

### 1. Fractal IR (Intermediate Representation)

Every workflow is a DAG of nodes. Nodes can be:
- **Opaque**: Leaf operations with inputs/outputs
- **SubDag**: Nested DAG (fractal composition)

```rust
// core/ir/src/node.rs
pub enum NodeBody<T> {
    Opaque(T),                    // Leaf operation
    SubDag(Box<Dag<T>>),          // Nested DAG
}

pub struct Node<T> {
    pub id: NodeId,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub body: NodeBody<T>,
}
```

**Guarantee Level**: Structure (1) — A node IS either opaque or subdag, no invalid states.

### 2. Set-Theoretic Cardinality

Every port has a cardinality that describes data flow:

```rust
// core/ir/src/types.rs
pub enum Cardinality {
    Zero,        // ∅ (signal, no data)
    One,         // Exactly one (scalar, required)
    ZeroOrOne,   // Optional (nullable)
    ZeroOrMore,  // List (may be empty)
    OneOrMore,   // Non-empty list
}
```

**Guarantee Level**: Compile-time validation (3)

```rust
// Compile error: ZeroOrMore cannot satisfy OneOrMore
dag.add_edge(Edge::new("filter", "out", "process", "in"));
//                      ↑ ZeroOrMore        ↑ OneOrMore
// ERROR: "output might be empty but input requires non-empty"
```

### 3. Structural World I/O Detection

World reads/writes are **deduced from graph structure**, not annotated:

```rust
// core/ir/src/boundary.rs
pub fn detect_boundaries<T>(dag: &Dag<T>) -> BoundaryInfo {
    // Output ports with no downstream edges = world writes
    // Detected structurally, impossible to forget annotation
}

// core/ir/src/entrypoint.rs  
pub fn detect_entrypoints<T>(dag: &Dag<T>) -> EntrypointInfo {
    // Input ports with no upstream edges = world reads
}
```

**Guarantee Level**: Impossible by structure (1) — If a port has no edge, it's automatically a boundary. You can't "forget" to mark it.

### 4. Compile-Time DAG Validation

```rust
// core/ir/src/validate.rs
pub fn validate_dag<T>(dag: &Dag<T>) -> Result<(), ValidationResult> {
    // Type compatibility on edges
    // Cardinality satisfaction on edges  
    // Cycle detection
    // Port existence
    // SubDag interface matching
}
```

**Guarantee Level**: Compile-time validation (3)

| Check | Error |
|-------|-------|
| `String → Int` | `TypeMismatch` |
| `ZeroOrMore → OneOrMore` | `CardinalityMismatch` |
| `A → B → A` | `CycleDetected` |
| `edge("X", "typo", ...)` | `PortNotFound` |

### 5. Lowering (SubDag Flattening)

SubDags are flattened to a single execution graph:

```rust
// core/exec/src/lower.rs
pub fn lower<T: Clone>(dag: &Dag<T>) -> Result<Dag<T>, LowerError> {
    // Flattens SubDag nodes into parent
    // Rewires edges through boundaries
    // Preserves all guarantees
}
```

**Guarantee Level**: Impossible by generation (2) — Lowered DAG is always valid if input was valid.

### 6. Mock Specifications

Each tool declares what its boundaries output when mocked:

```rust
// lib/tools/gist/src/graph_mock.rs
pub fn gist_mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("execute_transport", "url", 
            Value::Str("https://gist.github.com/mock/123".into()))
        .expects_input("repo_path", InputConstraint::Any)
        .resource_lock("fs:read")
        .resource_lease("github:api_token", 5000)
}
```

**Guarantee Level**: Runtime validation (4) — Mock values checked against constraints.

### 7. Resource Simulation

```rust
// core/test/src/mock_spec.rs
pub enum ResourceType {
    Lock,                           // Exclusive mutex
    Lease { duration_ms: u64 },     // Time-bounded
    SharedLock { max_holders },     // Read lock
    PoolSlot { pool_size },         // Connection pool
}

pub enum ResourceBehavior {
    AcquireSucceeds,
    FailAcquire { error: String },
    DelayAcquire { ms: u64 },
    LeaseExpires,
}
```

**Guarantee Level**: Tested (5) — Resource behavior verified by generated tests.

---

## Evidence / Code Samples

### Cardinality Satisfaction (Compile-Time)

```rust
// This compiles:
dag.add_node(Node::opaque("source", vec![], 
    vec![non_empty_list("items", "StrList")], op));  // OneOrMore
dag.add_node(Node::opaque("sink", 
    vec![list("items", "StrList")], vec![], op));    // ZeroOrMore
dag.add_edge(edge("source", "items", "sink", "items"));
// ✓ OneOrMore satisfies ZeroOrMore

// This fails to compile (validate_dag error):
dag.add_node(Node::opaque("filter", vec![...], 
    vec![list("out", "StrList")], op));              // ZeroOrMore
dag.add_node(Node::opaque("process", 
    vec![non_empty_list("in", "StrList")], vec![], op)); // OneOrMore
dag.add_edge(edge("filter", "out", "process", "in"));
// ✗ ERROR: CardinalityMismatch - "output might be empty but input requires non-empty"
```

### Boundary Detection (Structural)

```rust
// lib/tools/gist/src/graph.rs
pub fn build_gist_graph() -> Dag<GistGraphOp> {
    // ...
    dag.add_node(Node::opaque("execute_transport", 
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse"), 
             port("url", "String")],  // ← No downstream edge
        GistGraphOp::Transport(TransportOps::Execute)));
    // ...
}

// Boundary detected automatically:
let boundaries = detect_boundaries(&dag);
assert!(boundaries.is_boundary_node(&"execute_transport".into()));
// No annotation needed — structure implies boundary
```

### Generated Test (Runtime Verification)

```rust
// Generated by testgen - verifies dry-run actually intercepts
#[test]
fn test_boundary_execute_transport_mockable() {
    let dag = build_gist_graph(vec![], false);
    let mut mocks = BoundaryMocks::new();
    mocks.set_value("execute_transport", "url", 
        Value::Str("https://gist.github.com/mock/123".into()));
    
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();
    assert!(log.get("execute_transport").unwrap().was_intercepted);
}
```

---

## What We're Aiming For

### Near-Term: Complete Compile-Time Proof

| Property | Current | Target |
|----------|---------|--------|
| Type compatibility | ✓ Compile | ✓ |
| Cardinality satisfaction | ✓ Compile | ✓ |
| Boundary detection | ✓ Structural | ✓ |
| Effect purity | Implicit | Explicit in type |
| Idempotency | Declared | Verified |
| Resource requirements | Runtime | Compile |

### Long-Term: Impossibility by Structure

**Current**: Validation catches errors
```rust
validate_dag(&dag)?;  // Runtime check, returns error
```

**Target**: Type system prevents errors
```rust
// Can't even construct invalid DAG
let dag: Dag<Pure> = ...;  // Type enforces purity
dag.add_edge::<Satisfies<OneOrMore, ZeroOrMore>>(...);  // Type-level cardinality
```

### Proof Hierarchy Goal

```
                    ┌─────────────────────────────────┐
                    │     IMPOSSIBLE BY STRUCTURE     │
                    │   (Type system prevents it)     │
                    └─────────────────────────────────┘
                                   ↑
                    ┌─────────────────────────────────┐
                    │    IMPOSSIBLE BY GENERATION     │
                    │  (Codegen only makes valid)     │
                    └─────────────────────────────────┘
                                   ↑
        ════════════ COMPILE TIME BOUNDARY ════════════
                                   ↑
                    ┌─────────────────────────────────┐
                    │     VALIDATED AT COMPILE        │  ← We are here
                    │   (validate_dag catches it)     │
                    └─────────────────────────────────┘
                                   ↑
                    ┌─────────────────────────────────┐
                    │     VALIDATED AT RUNTIME        │
                    │  (Mock spec checks, etc.)       │
                    └─────────────────────────────────┘
                                   ↑
                    ┌─────────────────────────────────┐
                    │           TESTED                │
                    │   (Integration, e2e tests)      │
                    └─────────────────────────────────┘
```

---

## Behaviors Already Guaranteed

### Compile-Time (validate_dag)

| Guarantee | Evidence |
|-----------|----------|
| No type mismatches | `ValidationError::TypeMismatch` |
| Cardinality satisfaction | `ValidationError::CardinalityMismatch` |
| No cycles | `ValidationError::CycleDetected` |
| No dangling edges | `ValidationError::PortNotFound` |
| SubDag interfaces match | `ValidationError::SubDagInterfaceMismatch` |

### Structural (Impossible to Violate)

| Guarantee | How |
|-----------|-----|
| Boundaries detected | No downstream edge = boundary |
| Entrypoints detected | No upstream edge = entrypoint |
| Fractal composition | SubDag IS a DAG, recursively |
| Lowering preserves validity | Output DAG always valid |

### Generated Tests

| Test | Verifies |
|------|----------|
| `test_boundary_X_mockable` | Dry-run intercepts world writes |
| `test_mock_spec_self_consistent` | MockSpec has all declared mocks |
| `test_resource_X_acquire` | Resource simulation works |
| `test_resource_X_timeout` | Lease expiration handled |

---

## Integration / E2E Testing Support

### Execution Modes

```rust
// core/exec/src/lib.rs
pub enum ExecutionMode {
    Real,                    // Actually execute world I/O
    DryRun(BoundaryMocks),   // Intercept boundaries with mocks
    Simulate(SimConfig),     // Full simulation with timing
}
```

### Dry-Run Testing

```rust
// Intercepts all boundaries, uses mock values
let mocks = BoundaryMocks::new()
    .set_value("execute_transport", "url", mock_url);
let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))?;
assert!(log.all_boundaries_intercepted());
```

### Chain Validation (Cross-Tool)

```rust
// Verify A's mock output satisfies B's input expectations
let result = validate_chain(&gist_mock_spec(), &deploy_mock_spec(), &port_mapping);
assert!(result.is_ok());
```

### Resource Simulation

```rust
// Test with lock contention
let spec = ci_mock_spec()
    .resource_lock_fails("cargo:build", "Another build in progress");
let resource = spec.get_resource("cargo:build").unwrap();
assert!(matches!(resource.acquire(), ResourceAcquireResult::Failed(_)));
```

### Real-World Testing

```rust
// Actually execute (for integration tests)
let log = execute_with_mode(&dag, ExecutionMode::Real)?;
// Boundaries execute for real, results logged
```

---

## Tools with Full Mock Coverage

| Tool | Boundaries | Resources | MockSpec |
|------|------------|-----------|----------|
| gist | `execute_transport` | `fs:read`, `github:api_token` | ✓ |
| deps | `execute_installs` | `pkg:manager`, `sudo:elevation` | ✓ |
| makegen | `write_makefile` | `fs:Makefile` | ✓ |
| viz | `execute_transport` | `fs:viz-data.json` | ✓ |
| bootstrap | `write_files` | `fs:Makefile`, `fs:.gitignore` | ✓ |
| ci | `report` | `cargo:build`, `cargo:test`, `cargo:clippy` | ✓ |
| buck2 | `execute_transport` | `fs:Cargo.toml`, `fs:BUCK` | ✓ |

---

## File Structure

```
gunbc/
├── core/
│   ├── ir/           # DAG types, validation, boundary detection
│   ├── exec/         # Execution, lowering, dry-run
│   ├── test/         # MockSpec, resource simulation
│   ├── testgen/      # Test code generation
│   └── codegen/      # CLI/entrypoint generation
├── lib/
│   ├── primitives/   # ReadFiles, WriteFiles, etc.
│   ├── transport/    # HTTP, File, Shell transports
│   └── tools/        # gist, deps, makegen, viz, ci, buck2, bootstrap
└── docs/
    └── design/       # This document
```

---

## TODO: Thorough Review

- [ ] Audit all tools for complete MockSpec coverage
- [ ] Verify all cardinality annotations are correct
- [ ] Check lowering preserves all invariants
- [ ] Review SubDag boundary wiring
- [ ] Ensure generated tests match current guarantees
- [ ] Document any validation gaps
- [ ] Identify opportunities to move runtime checks to compile-time
- [ ] Review resource simulation completeness
