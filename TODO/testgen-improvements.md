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
