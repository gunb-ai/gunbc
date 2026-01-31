# Testgen: Automatic Test Generation from Graph Structure

> **Goal**: Exploit the fact that all nodes are pure/hermetic and all workflows
> are graphs to automatically generate comprehensive tests with minimal user input.

---

## Core Insight

The system has properties that enable automatic test generation:

1. **Nodes are pure** — same inputs → same outputs, no side effects
2. **I/O is isolated** — only transport executor nodes do I/O
3. **Everything is a graph** — structure is analyzable at build time
4. **Types are explicit** — ports declare types and cardinalities
5. **Resources flow as capabilities** — declared via input ports, passed via edges

These properties mean we can **derive tests from graph structure**, not require
manual test specifications.

---

## Unified Resource Model

All resources follow the **capability grant pattern**: they are acquired by an
owner node and flow downstream via explicit edges. This makes resource needs
visible in graph structure.

### Resource Types

| Type | Description | Example |
|------|-------------|---------|
| `ToolHandle` | Acquired tool binary | clippy, cargo, buck2 |
| `Lock` | Exclusive access | `cargo:build` (one build at a time) |
| `Lease` | Time-bounded access | API rate limit window |
| `SharedLock` | Concurrent read access | Read-only file access |
| `Budget` | Spend limit / ledger | API call quota, money |

### Pattern

```
env_node ──resource:X──▶ consumer_node ──resource:X──▶ sub_consumer
   │                          │
   │ (acquires)               │ (uses, may pass down)
```

1. **Owner node** acquires the resource (I/O boundary)
2. **Consumer node** declares need via input port: `port("resource:X", "Lock")`
3. **Edge** connects owner to consumer
4. **Subtree** can pass resource further down

### Benefits

- **Visible in graph** — resource dependencies are edges, not hidden
- **DryRun intercepts owner** — mock resources naturally
- **Structural tests** — verify all resource inputs have edges
- **Scoped access** — only nodes in the subtree can use the resource

### Budgets (Future)

Budgets represent finite resources like API quotas or money:

```rust
// Conceptual - exact design TBD
Budget {
    limit: 1000,           // total allowed
    ledger: LedgerHandle,  // tracks spend
    lease: LeaseHandle,    // time-bounded access
}
```

A node receiving a budget can spend from it and pass the (reduced) budget
downstream. The ledger tracks cumulative spend. Tests can verify:
- Budget flows to nodes that need it
- Spend stays within limits (via mock ledger)

---

## Test Classes

### Level 0: Zero User Input

These tests are generated purely from graph analysis. No `MockSpec`, no
annotations, no user input required.

#### Structural Tests

| Test | What it verifies |
|------|------------------|
| **Graph builds** | `build_*_graph()` succeeds |
| **DryRun completes** | Full workflow runs without crash |
| **Transports intercepted** | All transport executors are mocked in DryRun |
| **Pure nodes executed** | All pure nodes actually run |

```rust
#[test]
fn test_dryrun_completes() {
    let dag = build_ci_graph().unwrap();
    let mocks = auto_mocks_from_types(&dag);
    let result = execute_with_mode(&dag, ExecutionMode::DryRun(mocks));
    assert!(result.is_ok(), "DryRun should complete");
}

#[test]
fn test_all_transports_intercepted() {
    let dag = build_ci_graph().unwrap();
    let log = dryrun(&dag);

    for node_id in find_transport_executors(&dag) {
        let entry = log.get(&node_id).unwrap();
        assert!(entry.was_intercepted, "transport {} should be intercepted", node_id);
    }
}
```

#### Cardinality / Set Algebra Tests

| Test | What it verifies |
|------|------------------|
| **Edge cardinality compatibility** | `Many → One` requires explicit reduction |
| **Output matches declared** | Node declaring `One` doesn't output `Many` |
| **Empty collection handling** | `Many` inputs handle `[]` |
| **Large collection handling** | `Many` inputs handle 1000+ items |
| **Optional input presence** | Nodes work with/without optional inputs |

```rust
#[test]
fn test_cardinality_compatibility() {
    let dag = build_ci_graph().unwrap();

    for edge in &dag.edges {
        let source_card = get_output_cardinality(&dag, &edge.source);
        let target_card = get_input_cardinality(&dag, &edge.target);

        let compatible = match (source_card, target_card) {
            (One, One) => true,
            (One, Many) => true,     // auto-wrap
            (Many, Many) => true,
            (Many, One) => false,    // needs explicit reduce
            (Optional, One) => false, // needs unwrap
            _ => true,
        };

        assert!(compatible, "edge {:?} has incompatible cardinality", edge);
    }
}

#[test]
fn test_handles_empty_collections() {
    let dag = build_ci_graph().unwrap();

    for node in &dag.nodes {
        for port in node.inputs.iter().filter(|p| p.cardinality == Many) {
            let inputs = valid_inputs(&node).with(&port.name, Value::StrList(vec![]));
            let result = execute_node_isolated(&node, &inputs);

            assert!(
                result.is_ok() || result.is_handled_error(),
                "node {} cannot handle empty collection on port {}",
                node.id, port.name
            );
        }
    }
}
```

#### Resource Tests

Resources follow the capability grant model — they're declared as input ports
and flow via edges. This makes resource tests **structural** (analyzable from
graph alone) rather than requiring runtime tracking.

| Test | What it verifies |
|------|------------------|
| **Resource inputs have edges** | Every `resource:*` / `tool:*` input has an incoming edge |
| **Resource owner exists** | The edge source is a valid resource provider |
| **No orphan resources** | Resources acquired by owner are consumed by someone |
| **Contention handling** | Nodes handle failed acquisition gracefully (DryRun) |

```rust
#[test]
fn test_all_resource_inputs_have_edges() {
    let dag = build_ci_graph().unwrap();

    for node in &dag.nodes {
        for port in &node.inputs {
            let is_resource = port.name.0.starts_with("resource:")
                           || port.name.0.starts_with("tool:");
            if is_resource {
                assert!(
                    dag.has_edge_to(&node.id, &port.name),
                    "node {} declares {} but no edge provides it",
                    node.id, port.name
                );
            }
        }
    }
}

#[test]
fn test_resource_owners_are_env_nodes() {
    let dag = build_ci_graph().unwrap();

    for edge in &dag.edges {
        let is_resource = edge.source.port.0.starts_with("resource:")
                       || edge.source.port.0.starts_with("tool:");
        if is_resource {
            let source_node = dag.get_node(&edge.source.node).unwrap();
            assert!(
                is_resource_owner(source_node),
                "resource {} provided by {} which is not a resource owner",
                edge.source.port, edge.source.node
            );
        }
    }
}

#[test]
fn test_resource_contention_handling() {
    let dag = build_ci_graph().unwrap();

    // For each resource, test that consumer handles acquisition failure
    for resource in find_resources(&dag) {
        let mocks = contend_resource(&dag, &resource);
        let log = dryrun_with(&dag, mocks);

        // Workflow should complete (graceful degradation) or fail cleanly
        assert!(
            log.completed() || log.has_clean_failure(),
            "resource contention for {} caused ungraceful failure",
            resource
        );
    }
}
```

**Note:** The old tests "resources actually used" and "skipped nodes don't
acquire" required runtime tracking of resource acquisition. With the capability
grant model, resource needs are **structural** — if a node has a `tool:clippy`
input port, it declared that need. Whether it *uses* the capability when
skipped is an implementation detail of the node, not testable from graph
structure alone.

#### Type Tests

| Test | What it verifies |
|------|------------------|
| **Type coercions handled** | Receiver handles coerced values |
| **Boundary values** | Empty strings, zero, MAX_INT, etc. |

```rust
#[test]
fn test_type_coercions_handled() {
    let dag = build_ci_graph().unwrap();

    for edge in find_coercing_edges(&dag) {
        let value = generate_value_of_type(&edge.source_type);
        let inputs = hashmap! { edge.target_port => value };
        let result = execute_node_isolated(&edge.target_node, &inputs);

        assert!(result.is_ok(), "node {} cannot handle coercion", edge.target_node);
    }
}

#[test]
fn test_type_boundary_values() {
    let dag = build_ci_graph().unwrap();

    for node in &dag.nodes {
        for port in &node.inputs {
            for value in boundary_values(&port.type_id) {
                let inputs = valid_inputs(&node).with(&port.name, value.clone());
                let result = execute_node_isolated(&node, &inputs);

                assert!(
                    result.is_ok() || result.is_handled_error(),
                    "node {} panics on boundary value {:?} for {}",
                    node.id, value, port.name
                );
            }
        }
    }
}

fn boundary_values(type_id: &str) -> Vec<Value> {
    match type_id {
        "String" => vec![
            Value::Str("".into()),           // empty
            Value::Str(" \n\t".into()),      // whitespace
            Value::Str("a".repeat(10000)),   // large
        ],
        "Int" => vec![
            Value::Int(0),
            Value::Int(-1),
            Value::Int(i64::MAX),
            Value::Int(i64::MIN),
        ],
        "StrList" => vec![
            Value::StrList(vec![]),
            Value::StrList(vec!["".into()]),
            Value::StrList(vec!["a".into(); 1000]),
        ],
        _ => vec![],
    }
}
```

#### Property Tests

| Test | What it verifies |
|------|------------------|
| **Idempotency** | Same inputs → same outputs |
| **Determinism** | No timestamp/random dependencies |
| **No hidden state** | Isolated execution gives same results |

```rust
#[test]
fn test_pure_nodes_idempotent() {
    let dag = build_ci_graph().unwrap();

    for node in find_pure_nodes(&dag) {
        let inputs = valid_inputs(&node);
        let result1 = execute_node_isolated(&node, &inputs);
        let result2 = execute_node_isolated(&node, &inputs);

        assert_eq!(result1.outputs, result2.outputs, "node {} not idempotent", node.id);
    }
}

#[test]
fn test_deterministic_outputs() {
    let dag = build_ci_graph().unwrap();

    for node in find_pure_nodes(&dag) {
        let inputs = valid_inputs(&node);
        let results: Vec<_> = (0..5)
            .map(|_| execute_node_isolated(&node, &inputs).outputs)
            .collect();

        assert!(
            results.windows(2).all(|w| w[0] == w[1]),
            "node {} produces non-deterministic outputs",
            node.id
        );
    }
}
```

#### Lease/Timeout Tests

| Test | What it verifies |
|------|------------------|
| **Lease timeout handling** | Nodes handle expired leases |
| **Completion within lease** | Nodes finish before lease expires (or renew) |

```rust
#[test]
fn test_lease_timeout_handling() {
    let dag = build_ci_graph().unwrap();

    for (node, resource) in nodes_with_leases(&dag) {
        let mocks = expire_lease_immediately(&resource);
        let result = execute_node_with_mocks(&node, mocks);

        assert!(
            result.is_timeout_error() || result.has_output("timeout"),
            "node {} doesn't handle lease timeout",
            node.id
        );
    }
}
```

---

### Level 1: Convention-Based

These tests use naming conventions to infer semantics. No manual annotation,
just follow conventions.

#### Conventions

| Convention | Meaning |
|------------|---------|
| `exit_code: 0` | Shell command succeeded |
| `exit_code: non-zero` | Shell command failed |
| `*_success`, `overall_success` | Boolean success indicator |
| `skip` output | Node was skipped |
| `error` output | Node encountered error |

#### Success/Failure Path Tests

```rust
#[test]
fn test_success_path() {
    let dag = build_ci_graph().unwrap();
    let mocks = success_mocks(&dag);  // all exit_code: 0
    let log = dryrun_with(&dag, mocks);

    // Find outputs named *_success and verify true
    for (node, port) in find_success_outputs(&dag) {
        assert_eq!(
            log.get(&node).outputs[&port],
            Value::Bool(true),
            "success path: {}.{} should be true",
            node, port
        );
    }
}
```

#### Auto-Generated Failure Tests

For each transport executor, generate a test where it fails:

```rust
// Auto-generated for EACH transport executor node
#[test]
fn test_failure_at_execute_build() {
    let dag = build_ci_graph().unwrap();
    let mocks = fail_at(&dag, "execute_build");  // exit_code: 1
    let log = dryrun_with(&dag, mocks);

    // Terminal success should be false
    for (node, port) in find_success_outputs(&dag) {
        if is_terminal(&dag, &node) {
            assert_eq!(log.get(&node).outputs[&port], Value::Bool(false));
        }
    }
}

#[test]
fn test_failure_at_execute_test() { /* same pattern */ }

#[test]
fn test_failure_at_execute_clippy() { /* same pattern */ }
```

#### Skip Propagation Tests

```rust
#[test]
fn test_skip_propagation_when_build_fails() {
    let dag = build_ci_graph().unwrap();
    let mocks = fail_at(&dag, "execute_build");
    let log = dryrun_with(&dag, mocks);

    // Downstream nodes should be skipped
    for node in downstream_of(&dag, "execute_build") {
        if has_skip_output(&dag, &node) {
            assert_eq!(
                log.get(&node).outputs.get("skip"),
                Some(&Value::Bool(true)),
                "node {} should be skipped when upstream fails",
                node
            );
        }
    }
}
```

---

### Level 2: Integration Tests (Free from DryRun)

DryRun executes the full pure node chain with mocked I/O. This is a free
integration test.

```rust
#[test]
fn test_ci_workflow_integration() {
    let dag = build_ci_graph().unwrap();
    let mocks = realistic_success_mocks(&dag);
    let log = dryrun_with(&dag, mocks);

    // Workflow completed
    assert!(log.completed());

    // Hit all expected nodes
    assert!(log.get("report").is_some());

    // Terminal state is correct
    assert_eq!(
        log.get("report").outputs["overall_success"],
        Value::Bool(true)
    );
}

#[test]
fn test_ci_workflow_handles_build_failure() {
    let dag = build_ci_graph().unwrap();
    let mocks = fail_at(&dag, "execute_build");
    let log = dryrun_with(&dag, mocks);

    // Workflow should still complete (graceful failure)
    assert!(log.completed());

    // But report failure
    assert_eq!(
        log.get("report").outputs["overall_success"],
        Value::Bool(false)
    );
}
```

---

## Test Count Estimation

For a typical DAG with:
- 5 transport executors (T=5)
- 3 resources (R=3)
- 10 pure nodes (N=10)
- 8 edges (E=8)
- 3 Many-cardinality inputs (M=3)
- 2 Optional inputs (O=2)

### Tier 1: Implementable Today (no new infrastructure)

| Test Class | Count | Value |
|------------|-------|-------|
| Graph builds | 1 | High |
| DryRun completes | 1 | High |
| Transports intercepted | 1 | Medium |
| Pure nodes executed | 1 | Medium |
| Resource inputs have edges | 1 | High |
| Resource owners are valid | 1 | High |
| Success path | 1 | High |
| Individual failures | T | High |
| Skip propagation | T | High |
| Resource contention | R | Medium |
| **Subtotal** | **~15** | |

### Tier 2: Needs Infrastructure

| Test Class | Count | Requires |
|------------|-------|----------|
| Output matches declared cardinality | P | Runtime cardinality tracking |
| Empty/large collection handling | M×2 | `execute_node_isolated()` |
| Optional input presence | O×2 | `execute_node_isolated()` |
| Boundary values | P×~3 | Smart mock generation |
| Idempotency | N | `execute_node_isolated()` |
| Type coercions | C | Coercion detection |
| **Subtotal** | **~50** | |

### Tier 3: Compile-Time (already enforced)

| Test Class | Notes |
|------------|-------|
| Edge cardinality compatibility | `DagBuilder::add_edge` enforces |
| Type compatibility | Compile-time via port types |
| Cycle detection | `DagBuilder` enforces |

**Realistic near-term: ~15 tests per DAG with zero manual MockSpec.**

With infrastructure investment (node isolation, smart mocks): ~65 tests.

---

## What's Structural vs What Needs Annotation

### Structural (derivable from graph)

| Property | How we know |
|----------|-------------|
| Transport executors | Input port has type `TransportRequest` |
| Tool consumers | Input port has type `ToolHandle` |
| Resource consumers | Input port name starts with `resource:` or `tool:` |
| Success outputs | Output port name matches `*_success` (convention) |
| Skip outputs | Output port name is `skip` |
| Downstream nodes | Follow edges from a given node |
| Pure vs I/O nodes | No `TransportRequest` input = pure |

### Needs User Input

| Need | Example | When Required |
|------|---------|---------------|
| Custom success criteria | "success means file was created" | Non-convention outputs |
| Specific skip rules | "if A fails, B skips but C runs" | Complex skip logic |
| Expected output values | `report.overall_success == true` | Value assertions beyond Bool |
| Resource semantics | "cargo:build is exclusive" | Resource type info |

For custom scenarios, users can optionally provide `MockSpec` with
`expected_outputs`. But **basic structural coverage requires zero user input**.

---

## Implementation Phases

### Phase 1: Core Infrastructure (Tier 1 tests)

Enables ~15 tests per DAG with no manual MockSpec.

- [ ] `auto_mocks_from_types(dag)` — generate valid mocks from port types
- [ ] `find_transport_executors(dag)` — identify nodes to intercept
- [ ] `find_pure_nodes(dag)` — identify nodes that execute
- [ ] `find_resource_ports(dag)` — find `resource:*` and `tool:*` inputs
- [ ] `success_mocks(dag)` — all transports succeed (exit_code: 0)
- [ ] `fail_at(dag, node)` — one transport fails
- [ ] `contend_resource(dag, resource)` — simulate resource contention

### Phase 2: Structural Tests

- [ ] DryRun smoke test
- [ ] Transport interception test
- [ ] Pure node execution test
- [ ] Resource inputs have edges test
- [ ] Resource owners are valid env nodes test

### Phase 3: Convention-Based Flow Tests

- [ ] `find_success_outputs(dag)` — ports named `*_success`
- [ ] `downstream_of(dag, node)` — find nodes downstream of a given node
- [ ] Success path test
- [ ] Per-transport failure tests (auto-generated for each transport)
- [ ] Skip propagation tests
- [ ] Resource contention tests

### Phase 4: Node Isolation Infrastructure (Tier 2 tests)

Enables ~50 additional tests but requires significant infrastructure.

- [ ] `execute_node_isolated(node, inputs)` — run single node outside DAG
- [ ] `valid_inputs(node)` — generate type-appropriate inputs for any node
- [ ] Smart mock generation for complex types (`TransportResponse`, etc.)

### Phase 5: Property Tests (requires Phase 4)

- [ ] Idempotency tests
- [ ] Optional input presence tests
- [ ] Empty/large collection handling tests
- [ ] Boundary value tests

### Phase 6: Integration Tests

- [ ] Full workflow integration tests (success path)
- [ ] Failure-mode integration tests (one per transport)

---

## Migration from Current System

The current testgen produces low-value tests:
- Mock spec self-consistency (tautological)
- Input expectation counts (fragile snapshots)
- Trivial resource acquire (tests mock framework)

**Migration plan:**
1. Implement Phase 1-2 (auto-mocks, structural tests)
2. Run both old and new tests in parallel
3. Once new tests provide coverage, remove old generators
4. Keep hand-written `graph_mock.rs` tests (they test mock specs themselves)

---

## Appendix: Helper Functions

```rust
/// Generate type-appropriate mock value
fn mock_value_for_type(type_id: &str) -> Value {
    match type_id {
        "String" => Value::Str("<mock>".into()),
        "Bool" => Value::Bool(true),
        "Int" => Value::Int(0),
        "StrList" => Value::StrList(vec!["<mock>".into()]),
        "TransportResponse" => Value::Response(TransportResponse::Shell(
            ShellResponse { exit_code: 0, stdout: "<mock>".into(), stderr: "".into() }
        )),
        "ToolHandle" => Value::ToolHandle(ToolHandle::mock("mock-tool")),
        "Lock" => Value::Lock(LockHandle::mock("mock-lock")),
        _ => Value::Str("<mock>".into()),
    }
}

/// Generate mocks for all transport executor outputs
fn auto_mocks_from_types(dag: &Dag) -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();

    // Mock transport executors
    for node in find_transport_executors(dag) {
        for port in &node.outputs {
            mocks.set_value(&node.id, &port.name, mock_value_for_type(&port.type_id));
        }
    }

    // Mock resource/tool providers (env nodes)
    for node in find_resource_owners(dag) {
        for port in &node.outputs {
            mocks.set_value(&node.id, &port.name, mock_value_for_type(&port.type_id));
        }
    }

    mocks
}

/// Find nodes that consume TransportRequest (transport executors)
fn find_transport_executors(dag: &Dag) -> Vec<&Node> {
    dag.nodes.iter()
        .filter(|n| n.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"))
        .collect()
}

/// Find nodes that consume ToolHandle (tool consumers)
fn find_tool_consumers(dag: &Dag) -> Vec<&Node> {
    dag.nodes.iter()
        .filter(|n| n.inputs.iter().any(|p| p.type_id.0 == "ToolHandle"))
        .collect()
}

/// Find nodes that output resources (env/owner nodes)
fn find_resource_owners(dag: &Dag) -> Vec<&Node> {
    dag.nodes.iter()
        .filter(|n| n.outputs.iter().any(|p| {
            p.name.0.starts_with("resource:") ||
            p.name.0.starts_with("tool:") ||
            p.type_id.0 == "ToolHandle" ||
            p.type_id.0 == "Lock"
        }))
        .collect()
}

/// Find nodes that don't do I/O (pure nodes)
fn find_pure_nodes(dag: &Dag) -> Vec<&Node> {
    let transport = find_transport_executors(dag);
    let tool_consumers = find_tool_consumers(dag);
    let resource_owners = find_resource_owners(dag);

    dag.nodes.iter()
        .filter(|n| !transport.contains(n)
                 && !tool_consumers.contains(n)
                 && !resource_owners.contains(n))
        .collect()
}

/// Find all resource input ports in the DAG
fn find_resource_ports(dag: &Dag) -> Vec<(&NodeId, &PortName)> {
    dag.nodes.iter()
        .flat_map(|n| n.inputs.iter().map(move |p| (&n.id, &p.name)))
        .filter(|(_, p)| p.0.starts_with("resource:") || p.0.starts_with("tool:"))
        .collect()
}

/// Find outputs named *_success (convention for success indicators)
fn find_success_outputs(dag: &Dag) -> Vec<(&NodeId, &PortName)> {
    dag.nodes.iter()
        .flat_map(|n| n.outputs.iter().map(move |p| (&n.id, &p.name)))
        .filter(|(_, p)| p.0.ends_with("_success") || p.0 == "success")
        .collect()
}

/// Generate success mocks (all transports succeed)
fn success_mocks(dag: &Dag) -> BoundaryMocks {
    auto_mocks_from_types(dag)  // default is exit_code: 0
}

/// Generate mocks where one transport fails
fn fail_at(dag: &Dag, node_id: &str) -> BoundaryMocks {
    let mut mocks = success_mocks(dag);

    // Override the specified node to fail
    mocks.set_value(node_id, "response", Value::Response(
        TransportResponse::Shell(ShellResponse {
            exit_code: 1,
            stdout: "".into(),
            stderr: "mock failure".into(),
        })
    ));

    mocks
}

/// Generate mocks where a resource is contended (acquisition fails)
fn contend_resource(dag: &Dag, resource: &str) -> BoundaryMocks {
    let mut mocks = success_mocks(dag);

    // Find the owner of this resource and mock it as contended
    for node in find_resource_owners(dag) {
        for port in &node.outputs {
            if port.name.0 == resource {
                mocks.set_value(&node.id, &port.name, Value::ResourceContended(
                    format!("{} is held by another process", resource)
                ));
            }
        }
    }

    mocks
}
```
