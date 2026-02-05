# Typed Mock Builder: Impossible by Construction

**Status:** Implemented ✅
**Depends on:** Phase 8 (done), Phase 9 concepts

## Implementation Summary

The typed mock builder pattern has been implemented and the first tool (gist) has been migrated.

**Core files:**
- `core/test/src/mock_requirements.rs` - MockRequirements type and extract_mock_requirements()
- `lib/tools/gist/src/graph_mock.rs` - First migrated MockSpec

## Problem (Solved)

Current mock specification was disconnected from DAG definition:

```rust
// DAG defines ports with types
let dag = build_gist_dag();  // execute_gist.url: String, execute_gist.response: Json

// MockSpec defined separately - can have wrong types or miss ports
let spec = MockSpec::new("gist")
    .boundary("execute_gist", "url", Value::Int(42))  // WRONG TYPE
    // forgot execute_gist.response - not caught until testgen
```

**Solution:** `extract_mock_requirements()` analyzes the DAG and creates typed slots that validate mocks at construction time.

## Implemented Design

### 1. MockRequirements from DAG

```rust
// In core/test/src/mock_requirements.rs
pub fn extract_mock_requirements<T>(dag: &Dag<T>, name: &str) -> MockRequirements {
    let boundaries = detect_boundaries(dag);

    // Analyze DAG structure to find:
    // - Boundary ports (unconnected outputs)
    // - Transport executor outputs (connected or not)
    // - Resource/environment outputs (connected or not)

    // Create MockRequirements with typed slots
}
```

### 2. Type-Checked Mock Setting

```rust
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
        self.validate_type(&slot, &value)?;
        // ... add to filled set and mock list
        Ok(self)
    }

    // Typed helpers
    pub fn boundary_str(self, node: &str, port: &str, value: &str) -> Result<Self, MockTypeError>
    pub fn boundary_json(self, node: &str, port: &str, value: serde_json::Value) -> Result<Self, MockTypeError>
    pub fn transport_response(self, node: &str, port: &str, response: TransportResponse) -> Result<Self, MockTypeError>
}
```

### 3. Migration Example (gist)

```rust
// Old pattern:
pub fn gist_mock_spec(mode: &GistMode) -> MockSpec {
    MockSpec::new("gist")
        .boundary("fs_env", "fs:write", mock_fs_handle())  // Could have wrong type
        .boundary("execute_gist", "response", Value::Json(...))  // Could forget
}

// New pattern:
pub fn gist_mock_spec(mode: &GistMode) -> MockSpec {
    let dag = build_gist_graph(mode.clone(), vec![], false)
        .expect("gist graph should build");

    extract_mock_requirements(&dag, "gist")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs:write mock should match type")  // Type-checked!
        .transport_response("execute_gist", "response", mock_response())
        .expect("execute_gist response should match type")
        .build_unchecked()
}
```

## Type Compatibility

The following type compatibilities are implemented:

| Value Type | Port TypeId | Compatible |
|------------|-------------|------------|
| String | String | ✓ |
| Int | Int | ✓ |
| Int | Timestamp | ✓ (Timestamp serializes as Int) |
| Map | ToolHandle, AuthToken, FilesystemHandle, Platform | ✓ (Map-backed types) |
| Response | TransportResponse | ✓ |
| Json | Any | ✓ (Json is flexible) |
| Any | Any | ✓ |
| Skipped | Any | ✓ (Skipped is always compatible) |

## MockSlotKind

Three kinds of mock slots are detected:

1. **Boundary** - Unconnected output ports (world writes)
2. **Transport** - Transport executor outputs (consume TransportRequest)
3. **Resource** - Environment/resource node outputs (emit capability tokens)

## Error Handling

Errors at mock construction time (not testgen time):

```rust
pub enum MockTypeError {
    UnknownSlot { node: String, port: String },
    TypeMismatch { node: String, port: String, expected: String, actual: String },
    CardinalityMismatch { node: String, port: String, expected: Cardinality, actual: usize },
}

pub struct MockIncompleteError {
    dag_name: String,
    missing: Vec<String>,  // List of node.port that are unfilled
}
```

## Benefits Realized

1. **Impossible to forget mocks** - `build()` fails if required slots unfilled
2. **Type errors at definition site** - not at testgen, not at test runtime
3. **Co-location** - DAG is built first, mocks extracted from its structure
4. **IDE support** - typed helpers enable autocomplete
5. **Test validates pattern** - `test_typed_builder_catches_type_errors` proves type checking works

## Migration Status

| Tool | Status | Notes |
|------|--------|-------|
| gist | ✅ Migrated | Full pattern: DAG → extract → type-check → build |
| deps | ✅ Migrated | Transport + resource mocks only |
| makegen | ✅ Migrated | Transport node with multiple outputs |
| bootstrap | ✅ Migrated | Multiple transport nodes (scan, makefile, gitignore) |
| ci | ✅ Migrated | Complex graph with transport + resource + CliTool nodes |
| review | ✅ Migrated | Two graphs: inline and diff, both with LLM transport |
| llm-ops | ⏳ Pending | Multiple providers, may need different approach |

## Relation to Phase 9 DagSpec

This is a stepping stone to DagSpec. Once all mock specs are migrated:

```rust
// Current approach
pub fn gist_mock_spec() -> MockSpec {
    let dag = build_gist_dag();
    extract_mock_requirements(&dag, "gist")
        .boundary_str(...)
        .build_unchecked()
}

// Future DagSpec
pub struct DagSpec<T> {
    builder: fn() -> Dag<T>,
    mock_spec: MockSpec,  // Built from requirements, guaranteed complete
    signature: Option<DagSignature>,
}
```

## Resolved Questions

1. **Error handling:** Using `Result` with `.expect()` at call sites - Rusty and explicit
2. **Cardinality validation:** Not yet implemented, but slot has cardinality info
3. **Optional slots:** All slots from boundaries are required; `build_unchecked()` panics if missing
4. **Compile-time vs runtime:** Runtime checking at mock construction is acceptable - error happens at definition site
