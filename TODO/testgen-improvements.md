# Testgen Improvements

**Status**: TODO
**Date**: 2026-02-02

## Problem Statement

Current testgen has several deficiencies:

1. **Manual target registration** - must add each DAG to `all_targets()` in `testgen.rs`
2. **No staleness detection** - generated tests can drift from DAG/MockSpec
3. **No auto-regeneration** - `make test` doesn't regenerate, easy to forget
4. **MockSpecs are optional** - no enforcement, easy to skip
5. **Only tests wiring** - doesn't test node I/O behavior
6. **Committed generated files** - can go stale in git

## Desired Model

**DAG definition = test specification**

When you define a DAG node, you also specify its I/O contract. Testgen generates tests that verify the contract.

```rust
// Current: you define DAG, then separately write unit tests
dag.add_node(Node::opaque("prepare_prompt", inputs, outputs, ReviewOps::PrepareReviewPrompt));

// Desired: I/O examples are part of the node definition
dag.add_node(Node::opaque("prepare_prompt", inputs, outputs, ReviewOps::PrepareReviewPrompt)
    .with_example(
        inputs! { "artifact" => "fn foo() {}", "criteria" => security_criteria() },
        outputs! { "question" => contains("security"), "system_prompt" => non_empty() },
    )
);
```

Then testgen generates:
```rust
#[test]
fn test_prepare_prompt_example_0() {
    let inputs = hashmap! { "artifact" => "fn foo() {}", "criteria" => ... };
    let result = ReviewOps::PrepareReviewPrompt.execute(inputs).unwrap();
    assert!(result["question"].as_str().unwrap().contains("security"));
    assert!(!result["system_prompt"].as_str().unwrap().is_empty());
}
```

## Key Insight

**Mocks and I/O examples serve the same purpose** - they specify what a node produces. The difference:

| | Transport Nodes | Pure Nodes |
|---|---|---|
| **What** | Mock response (external I/O) | Expected output (computed) |
| **Why** | Can't run real I/O in tests | Verify business logic |
| **How** | `MockSpec::boundary()` | `Node::with_example()` |

Both should be **required**, not optional.

## Implementation Plan

### Phase 1: Enforce MockSpecs

**TODO 1.1: Require MockSpec for transport nodes**
- [ ] In `TestGenerator::generate()`, fail if DAG has transport nodes but no MockSpec
- [ ] Error message: "DAG has N transport nodes but no MockSpec provided"

**TODO 1.2: Add staleness check**
- [ ] Embed DAG hash in generated test file header
- [ ] Add `testgen --check` mode that verifies hashes match
- [ ] `make test` runs `testgen --check` before `cargo test`

**TODO 1.3: Auto-discover DAGs**
- [ ] Scan workspace for `pub fn build_*_graph()` functions
- [ ] Generate target list automatically instead of hardcoding
- [ ] Or: require DAGs to register via a manifest file

### Phase 2: I/O Examples on Nodes

**TODO 2.1: Add `NodeExample` type**
```rust
pub struct NodeExample {
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, OutputMatcher>,
}

pub enum OutputMatcher {
    Exact(Value),
    Contains(String),
    NonEmpty,
    Satisfies(fn(&Value) -> bool),
}
```

**TODO 2.2: Extend Node builder**
```rust
impl Node<T> {
    pub fn with_example(self, inputs: HashMap<String, Value>, outputs: HashMap<String, OutputMatcher>) -> Self;
    pub fn examples(&self) -> &[NodeExample];
}
```

**TODO 2.3: Generate per-node tests**
- [ ] For each node with examples, generate a test
- [ ] Test calls `node.op.execute(example.inputs)` and checks `example.outputs`
- [ ] One test per example (nodes can have multiple examples)

### Phase 3: Makefile Integration

**TODO 3.1: Add testgen to Makefile**
```makefile
testgen:
    @cargo run -p gunbc-dag --bin gunbc-testgen

testgen-check:
    @cargo run -p gunbc-dag --bin gunbc-testgen -- --check

test: testgen-check build
    @cargo test
```

**TODO 3.2: Upsert pattern**
- [ ] `make test` checks staleness first
- [ ] If stale, fail with "run `make testgen` to regenerate"
- [ ] Or: auto-regenerate (configurable)

### Phase 4: Stop Committing Generated Tests

**TODO 4.1: Add to .gitignore**
```
**/generated_tests*.rs
```

**TODO 4.2: Generate on build**
- [ ] `make build` includes `testgen`
- [ ] Generated tests are build artifacts, not source

**Alternative**: Keep committing but with strict staleness checks in CI.

### Phase 5: Windowed Segment Testing

Test arbitrary sub-segments of a DAG pipeline automatically. Instead of testing nodes
in isolation (Phase 2), execute contiguous windows of nodes and verify inter-node
integration.

**Motivation**: Per-node testing catches I/O bugs within a single node, but misses
integration bugs where the composition `B → C` breaks due to subtle mismatches in
how values flow between nodes. Windowed testing covers the space between unit tests
(single node) and full-DAG smoke tests (everything).

**Core idea**: A full DryRun captures every intermediate value at every port. These
captured values can seed any arbitrary window — no additional mocks needed for pure
nodes, and transport nodes are already intercepted by DryRun.

**TODO 5.1: Define `Window` type**
```rust
pub struct Window {
    /// Entry-point nodes of this window (inputs severed and injected)
    pub entry_nodes: Vec<NodeId>,
    /// Exit-point nodes (outputs captured and verified)
    pub exit_nodes: Vec<NodeId>,
    /// All nodes in the interior (executed normally)
    pub interior: Vec<NodeId>,
}
```

**TODO 5.2: Window enumeration**
- [ ] Given a DAG, enumerate all valid windows (contiguous sub-DAG slices)
- [ ] For a linear DAG of n nodes, this is O(n^2) windows (sliding window)
- [ ] For branching DAGs, enumerate by cut boundaries — sever incoming edges at
      entry nodes, capture outgoing edges at exit nodes
- [ ] Optionally limit window size (e.g., max 5 nodes) to control test explosion

**TODO 5.3: DryRun value capture**
- [ ] Run full DryRun with existing MockSpec → `ExecutionLog`
- [ ] Extract per-port values at window entry boundaries from the log
- [ ] These become the injected inputs for the windowed execution

**TODO 5.4: Windowed execution**
- [ ] For each window, construct a sub-DAG or use `input_mock()` injection at
      the severed entry edges
- [ ] Execute the window with DryRun mode (transport nodes still intercepted)
- [ ] Pure nodes execute for real — this is where integration bugs surface
- [ ] Compare window exit-port values against the full DryRun expected values

**TODO 5.5: Test generation**
- [ ] For each enumerated window, generate a test function
- [ ] Test name encodes the window: `test_window_B_through_D`
- [ ] Inputs derived from DryRun log, outputs verified against DryRun log
- [ ] Consider grouping by window size or DAG region to organize output

**Open design questions for Phase 5:**
- **Test explosion**: O(n^2) can be large. Should we sample windows, use a max
  window size, or generate all and let the user filter?
- **Branching semantics**: For fan-out/fan-in, what constitutes a valid window?
  Likely: any connected sub-DAG where all severed incoming edges have DryRun values.
- **Transport within windows**: If a window interior contains a transport node, it
  gets DryRun-intercepted. This tests the pure logic around it but not the transport
  itself — which is the right tradeoff.
- **Depends on**: Phase 2 (I/O examples) is nice-to-have but not required. The
  DryRun log provides all the mock values. Phase 1 (MockSpec enforcement) is
  a prerequisite since we need a valid DryRun to seed windows.

## Open Questions

1. **Commit or not?** Generated tests as source vs build artifact
   - Pro commit: visible in PR diffs, works without build step
   - Pro .gitignore: no stale tests, cleaner history

2. **Auto-discover vs manifest?** How to find DAGs
   - Auto-discover: magic, might miss some
   - Manifest: explicit, more work

3. **Example syntax?** How verbose should `with_example()` be?
   - Macro-based: `inputs! { ... }` - concise but magic
   - Builder: `.input("foo", val).output("bar", matcher)` - verbose but clear

4. **Inheritance?** If DAG A embeds DAG B as subdag, do B's examples run?
   - Probably yes - verify subdags work in context

## Success Criteria

After implementation:

1. **Can't forget tests** - testgen auto-discovers DAGs, fails if no MockSpec
2. **Can't have stale tests** - staleness check in `make test`
3. **I/O is tested** - node examples generate unit tests
4. **Minimal ceremony** - just add `.with_example()` to node, tests appear

## References

- Current testgen: `core/testgen/`
- Current targets: `gunbc-dag/src/bin/testgen.rs`
- MockSpec: `core/test/src/mock_spec.rs`
- Obligation model: `core/testgen/src/obligation.rs`
