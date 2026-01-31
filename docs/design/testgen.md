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
5. **Resources are declared** — nodes declare what they acquire

These properties mean we can **derive tests from graph structure**, not require
manual test specifications.

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

| Test | What it verifies |
|------|------------------|
| **Resources actually used** | Declared resources are accessed |
| **Skipped nodes don't acquire** | `skip=true` → no resource acquisition |
| **Resources released** | All acquired resources are released |
| **Contention handling** | Nodes handle failed acquisition gracefully |

```rust
#[test]
fn test_resources_actually_used() {
    let dag = build_ci_graph().unwrap();

    for node in &dag.nodes {
        for resource in node.declared_resources() {
            let log = execute_node_isolated(&dag, &node);

            assert!(
                log.resource_accessed(&resource),
                "node {} declares resource {} but never uses it",
                node.id, resource
            );
        }
    }
}

#[test]
fn test_skipped_nodes_dont_acquire_resources() {
    let dag = build_ci_graph().unwrap();

    for node in nodes_with_skip_input(&dag) {
        let log = execute_node_with(&node, inputs! { "skip" => true });

        assert!(
            log.resources_acquired().is_empty(),
            "node {} acquired resources despite being skipped",
            node.id
        );
    }
}
```

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
- 15 input ports (P=15)
- 4 skippable nodes (S=4)
- 8 edges (E=8)
- 3 Many-cardinality inputs (M=3)
- 2 Optional inputs (O=2)
- 2 leased resources (L=2)

| Test Class | Count | Formula |
|------------|-------|---------|
| Graph builds | 1 | |
| DryRun completes | 1 | |
| Transports intercepted | 1 | |
| Pure nodes executed | 1 | |
| Edge cardinality | 8 | E |
| Output matches declared | 15 | P |
| Empty collection handling | 3 | M |
| Large collection handling | 3 | M |
| Optional input presence | 4 | O × 2 |
| Resources actually used | 3 | R |
| Skipped nodes don't acquire | 4 | S |
| Resources released | 1 | |
| Contention handling | 3 | R |
| Lease timeout | 2 | L |
| Type coercions | ~2 | C |
| Boundary values | ~30 | P × ~2 |
| Idempotency | 10 | N |
| Determinism | 10 | N |
| Success path | 1 | |
| Individual failures | 5 | T |
| Skip propagation | 5 | T |
| Integration (success) | 1 | |
| Integration (failures) | 5 | T |
| **Total** | **~120** | |

**~120 tests automatically generated with zero manual MockSpec.**

---

## What Still Needs User Input

Some tests require semantic knowledge the graph doesn't encode:

| Need | Example |
|------|---------|
| Custom success criteria | "success means file was created" |
| Business rule verification | "if A fails, B and C should be skipped but D should still run" |
| Non-convention outputs | success indicator not named `*_success` |

For these, users can optionally provide `MockSpec` with `expected_outputs`.
But this is **opt-in enrichment**, not required for basic coverage.

---

## Implementation Phases

### Phase 1: Auto-Mock Infrastructure

- [ ] `auto_mocks_from_types(dag)` — generate valid mocks from port types
- [ ] `find_transport_executors(dag)` — identify nodes to intercept
- [ ] `find_pure_nodes(dag)` — identify nodes that execute
- [ ] `valid_inputs(node)` — generate type-appropriate inputs for a node
- [ ] `execute_node_isolated(node, inputs)` — run single node

### Phase 2: Structural Tests

- [ ] Generate Level 0 tests for all DAGs
- [ ] DryRun smoke, transport interception, pure node execution

### Phase 3: Cardinality Tests

- [ ] `get_cardinality(dag, port)` — extract cardinality from port
- [ ] `find_coercing_edges(dag)` — edges with type coercion
- [ ] Generate cardinality compatibility tests
- [ ] Generate empty/large collection tests
- [ ] Generate optional input tests

### Phase 4: Resource Tests

- [ ] Track resource access in execution log
- [ ] Generate "resources actually used" tests
- [ ] Generate "skipped nodes don't acquire" tests
- [ ] Generate contention handling tests

### Phase 5: Property Tests

- [ ] Generate idempotency tests
- [ ] Generate determinism tests
- [ ] Generate boundary value tests

### Phase 6: Convention-Based Flow Tests

- [ ] `find_success_outputs(dag)` — ports named `*_success`
- [ ] `success_mocks(dag)` — all transports succeed
- [ ] `fail_at(dag, node)` — one transport fails
- [ ] Generate success path test
- [ ] Generate per-transport failure tests
- [ ] Generate skip propagation tests

### Phase 7: Integration Tests

- [ ] Generate full workflow integration tests
- [ ] Generate failure-mode integration tests

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
        _ => Value::Str("<mock>".into()),
    }
}

/// Generate mocks for all transport executor outputs
fn auto_mocks_from_types(dag: &Dag) -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();

    for node in find_transport_executors(dag) {
        for port in &node.outputs {
            mocks.set_value(&node.id, &port.name, mock_value_for_type(&port.type_id));
        }
    }

    mocks
}

/// Find nodes that consume TransportRequest (transport executors)
fn find_transport_executors(dag: &Dag) -> Vec<&Node> {
    dag.nodes.iter()
        .filter(|n| n.inputs.iter().any(|p| p.type_id == "TransportRequest"))
        .collect()
}

/// Find nodes that don't consume TransportRequest (pure nodes)
fn find_pure_nodes(dag: &Dag) -> Vec<&Node> {
    dag.nodes.iter()
        .filter(|n| !n.inputs.iter().any(|p| p.type_id == "TransportRequest"))
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
```
