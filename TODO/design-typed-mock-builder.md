# Typed Mock Builder: Impossible by Construction

**Status:** Design
**Depends on:** Phase 8 (done), Phase 9 concepts

## Problem

Current mock specification is disconnected from DAG definition:

```rust
// DAG defines ports with types
let dag = build_gist_dag();  // execute_gist.url: String, execute_gist.response: Json

// MockSpec defined separately - can have wrong types or miss ports
let spec = MockSpec::new("gist")
    .boundary("execute_gist", "url", Value::Int(42))  // WRONG TYPE
    // forgot execute_gist.response - not caught until testgen
```

**Current mitigation:** Validate at testgen time, panic with error messages.

**Problem with validation:** It's still possible to write incorrect code. We catch it
late (at test generation, not at mock definition). Better error messages help, but
don't make the bug impossible.

## Design Principle

> "Make invalid states unrepresentable"

If a DAG has boundary ports, constructing a MockSpec for that DAG should
**require** providing mocks for those ports, and the types should be checked
**at the point of mock construction**, not at testgen time.

## Proposed Design

### 1. DAG Produces Mock Requirements

```rust
impl<T> Dag<T> {
    /// Extract mock requirements from DAG structure.
    ///
    /// This analyzes boundaries, transport nodes, and resource ports to
    /// determine what mocks are needed for testing this DAG.
    pub fn mock_requirements(&self) -> MockRequirements {
        let boundaries = detect_boundaries(self);

        MockRequirements {
            dag_name: self.name.clone(),
            boundary_slots: boundaries.boundary_ports.iter()
                .map(|(node_id, port_name)| {
                    let node = self.get_node(node_id).unwrap();
                    let port = node.outputs.iter()
                        .find(|p| &p.name == port_name).unwrap();
                    MockSlot {
                        node_id: node_id.clone(),
                        port_name: port_name.clone(),
                        type_id: port.type_id.clone(),
                        cardinality: port.cardinality,
                        required: true,  // boundaries are always required
                    }
                })
                .collect(),
            transport_slots: self.find_transport_nodes()
                .into_iter()
                .flat_map(|node| node.outputs.iter().map(move |p| /* ... */))
                .collect(),
        }
    }
}
```

### 2. MockRequirements Enforces Completeness

```rust
pub struct MockRequirements {
    dag_name: String,
    boundary_slots: Vec<MockSlot>,
    transport_slots: Vec<MockSlot>,
    filled: HashSet<(NodeId, PortName)>,  // tracks what's been provided
}

pub struct MockSlot {
    node_id: NodeId,
    port_name: PortName,
    type_id: TypeId,
    cardinality: Cardinality,
    required: bool,
}

impl MockRequirements {
    /// Set a boundary mock. Validates type at call site.
    pub fn boundary(
        mut self,
        node: &str,
        port: &str,
        value: impl Into<Value>,
    ) -> Result<Self, MockTypeError> {
        let slot = self.find_slot(node, port)?;
        let value = value.into();

        // Type check happens HERE, not at testgen
        self.validate_type(&slot, &value)?;

        self.filled.insert((slot.node_id.clone(), slot.port_name.clone()));
        self.mocks.push(BoundaryMock { node, port, value });
        Ok(self)
    }

    /// Typed helpers for common patterns
    pub fn boundary_str(self, node: &str, port: &str, value: &str) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Str(value.to_string()))
    }

    pub fn boundary_json(self, node: &str, port: &str, value: serde_json::Value) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Json(value))
    }

    pub fn transport_response(self, node: &str, port: &str, response: TransportResponse) -> Result<Self, MockTypeError> {
        // Only accepts TransportResponse for transport ports
        self.boundary(node, port, Value::Response(response))
    }

    /// Build MockSpec. Fails if required slots are unfilled.
    pub fn build(self) -> Result<MockSpec, MockIncompleteError> {
        let unfilled: Vec<_> = self.boundary_slots.iter()
            .chain(self.transport_slots.iter())
            .filter(|s| s.required && !self.filled.contains(&(s.node_id.clone(), s.port_name.clone())))
            .collect();

        if !unfilled.is_empty() {
            return Err(MockIncompleteError {
                dag_name: self.dag_name,
                missing: unfilled.iter().map(|s| format!("{}.{}", s.node_id.0, s.port_name.0)).collect(),
            });
        }

        Ok(MockSpec { /* ... */ })
    }
}
```

### 3. Co-located DAG + Mock Definition

Instead of separate files:
```
lib/tools/gist/src/
├── graph.rs       # DAG definition
├── graph_mock.rs  # MockSpec definition (separate, can drift)
```

The mock requirements come from the DAG:
```rust
// In graph.rs
pub fn build_gist_dag() -> Dag<GistGraphOp> { /* ... */ }

pub fn gist_mock_spec() -> MockSpec {
    let dag = build_gist_dag();

    dag.mock_requirements()
        .boundary_str("execute_gist", "url", "https://gist.github.com/mock/123")
        .unwrap()
        .boundary_json("execute_gist", "response", json!({ /* ... */ }))
        .unwrap()
        // Missing required mock → compile error or early panic
        .build()
        .expect("all required mocks provided")
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MockTypeError {
    #[error("unknown mock slot {node}.{port}")]
    UnknownSlot { node: String, port: String },

    #[error("type mismatch for {node}.{port}: expected {expected}, got {actual}")]
    TypeMismatch {
        node: String,
        port: String,
        expected: TypeId,
        actual: String,
    },

    #[error("cardinality mismatch for {node}.{port}: expected {expected:?}, got {actual} values")]
    CardinalityMismatch {
        node: String,
        port: String,
        expected: Cardinality,
        actual: usize,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("incomplete mock spec for {dag_name}: missing {}", missing.join(", "))]
pub struct MockIncompleteError {
    dag_name: String,
    missing: Vec<String>,
}
```

## Benefits

1. **Impossible to forget mocks** - `build()` fails if required slots unfilled
2. **Type errors at definition site** - not at testgen, not at test runtime
3. **Co-location** - DAG and mocks defined together, can't drift
4. **IDE support** - typed helpers enable autocomplete
5. **Removes testgen validation** - the current `find_missing_transport_mocks` becomes unnecessary

## Migration Path

1. Add `Dag::mock_requirements()` method
2. Add `MockRequirements` builder type
3. Keep existing `MockSpec::new()` path for backwards compatibility
4. Migrate tool mock specs one by one to new pattern
5. Eventually deprecate/remove `MockSpec::new()` direct construction
6. Remove testgen validation panics (now impossible to hit)

## Relation to Phase 9 DagSpec

This is a stepping stone to DagSpec. Once mock requirements are co-located:

```rust
// Current proposal
pub fn gist_mock_spec() -> MockSpec {
    let dag = build_gist_dag();
    dag.mock_requirements()
        .boundary_str(...)
        .build()
}

// Future DagSpec
pub struct DagSpec<T> {
    builder: fn() -> Dag<T>,
    mock_spec: MockSpec,  // Built from requirements, guaranteed complete
    signature: Option<DagSignature>,
}
```

## Open Questions

1. **Error handling:** Should typed helpers return `Result` or panic?
   - `Result` is more Rusty, but verbose
   - Panic matches current `MockSpec::boundary()` pattern
   - Could have both: `boundary()` returns Result, `boundary_unchecked()` panics

2. **Cardinality validation:** Should we validate mock value cardinality?
   - `Value::List([a, b])` for a `Cardinality::ONE` port should fail
   - This is caught later in execution, but could catch at mock construction

3. **Optional slots:** Some boundaries might have default mocks (from `Mockable` trait)
   - Should these be pre-filled in requirements?
   - Or should `build()` auto-fill from `Mockable::mock_outputs()`?

4. **Compile-time vs runtime:** TypeId is a runtime string, so full compile-time safety
   is hard. Is runtime checking at mock construction acceptable?
   - It's still "by construction" in that you can't construct a bad MockSpec
   - The error happens at the call site, not at testgen or test runtime
