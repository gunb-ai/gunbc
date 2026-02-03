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

**TODO 1.1: Require MockSpec for transport nodes** ✅
- [x] In `TestGenerator::generate()`, fail if DAG has transport nodes but no MockSpec
- [x] Error message: "DAG has N transport nodes but no MockSpec provided"
- *Already implemented in `codegen.rs:145` — panics with detailed guidance.*

**TODO 1.2: Add staleness check** ✅
- [x] Embed content hash in generated test file header — *`codegen.rs`: body hashed before header emission*
- [x] Add `testgen --check` mode that verifies files match — *already implemented in `testgen.rs:298`*
- [x] `make test` runs `testgen-check` before `cargo test` — *via `MetaTarget.extra_deps`*

**TODO 1.3: Auto-discover DAGs** ✅ (partial)
- [x] Refactored `testgen.rs` to single registration site with generic `generate_target<T>`
- [x] Eliminated 7 duplicate builder functions — each target is now 3 lines
- [x] Consolidated two-site registration into one: removed `all_testgen_dags()` from
  `codegen/registry.rs`, inlined `TestgenTargetDef` metadata directly alongside builder
  closures in `testgen.rs`. Adding a new target is now a single addition in one place.
- [x] Added `target!()` macro: each expression written once, macro both calls it
  and stringifies it (with `crate::` replacement via `to_crate_path()`). ~5 lines per target.
- [ ] Full auto-discovery deferred — see Phase 6 (Registry-Driven Testgen) below

### Phase 2: I/O Examples on Nodes

**TODO 2.1: Add `NodeExample` type** ✅
- *Already implemented in `mock_spec.rs:644` — includes `NodeExample`, `OutputMatcher`,
  builder pattern, and `to_check_code()` for assertion generation.*

```rust
// Already exists:
pub struct NodeExample {
    pub node_id: String,
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, OutputMatcher>,
    pub description: Option<String>,
}
```

**TODO 2.2: Extend Node builder** ✅
- [x] Added `examples: Vec<NodeIoExample>` field to `Node<T>` (serde skip if empty)
- [x] Added `with_example()` and `with_described_example()` builder methods
- [x] `NodeIoExample` type in `gunbc-ir` uses exact `Value` matching (serializable)
- [x] Testgen collects examples from both `Node.examples` AND `MockSpec.node_examples`
- *Node examples use exact match; MockSpec examples use rich `OutputMatcher`*

**TODO 2.3: Generate per-node tests** ✅
- [x] For each node with examples, generate a test
- [x] Test calls `execute_single_node(example.inputs)` and checks `example.outputs`
- [x] One test per example (nodes can have multiple examples)
- *Already implemented in `codegen.rs:1430` — `generate_node_example_tests()`*

### Phase 3: Makefile Integration

**TODO 3.1: Add testgen to Makefile** ✅
- *Already in Makefile: `make testgen` and `make testgen-check` targets exist.*

**TODO 3.2: Upsert pattern** ✅
- [x] `make test` depends on `testgen-check` — fails if generated tests are stale
- [x] Error message directs user to run `make testgen`

### Phase 4: Stop Committing Generated Tests

**TODO 4.1: Add to .gitignore** ✅
- [x] Added `TESTGEN_CATEGORY` to gitignore generation: `**/generated_tests*.rs`
- *Generated by bootstrap via `derive_categories()` in `gitignore.rs`*

**TODO 4.2: Generate on build** ✅
- [x] `make build` depends on `testgen` (via `render_core_targets`)
- [x] Generated tests are build artifacts, regenerated before compilation

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

### Phase 6: Merge Testgen into Codegen

Roll `gunbc-testgen` into `gunbc-codegen` so there's one code generation crate
that owns the registry, the test generator, and the output pipeline.

**Current state**: `gunbc-testgen` and `gunbc-codegen` are independent crates
that share no code. Both depend on `gunbc-ir`. The testgen binary in `gunbc-dag`
is the only thing that pulls them together. This means:
- Two separate code generators doing the same kind of work
- The registry (`ToolDef`) lives in codegen but can't reference test generation
- The testgen binary duplicates codegen's FileWriter/arg-parsing infrastructure
- Adding a DAG requires edits in both systems

```
Current dependency graph:

gunbc-codegen ──→ gunbc-ir      gunbc-testgen ──→ gunbc-ir
     │            gunbc-clippy        │            gunbc-test
     │                                │
     └──── both consumed by ──────────┘
              gunbc-dag (testgen binary)
```

**Merged state**: codegen gains testgen's 4 files + `gunbc-test` dep. The
testgen binary becomes `gunbc-codegen testgen`. The registry can directly
reference `TestGenerator`.

```
After merge:

gunbc-codegen ──→ gunbc-ir
     │            gunbc-test    (was testgen dep)
     │            gunbc-clippy
     │
     ├── codegen (CLI main.rs generation)
     ├── daggen  (graph.rs generation)
     ├── cigen   (CI YAML generation)
     ├── testgen (test file generation)  ← NEW subcommand
     └── registry (single source of truth for all DAGs)
```

**TODO 6.1: Move testgen modules into codegen** ✅
- [x] Moved `core/testgen/src/{analyze,obligation,codegen}.rs` → `core/codegen/src/testgen/`
- [x] Added `gunbc-test` dependency to codegen's Cargo.toml
- [x] Exported `testgen` module from codegen (`gunbc_codegen::testgen::*`)
- [x] Updated `gunbc-dag/Cargo.toml` to drop `gunbc-testgen` dep
- [x] Deleted `core/testgen/` crate, removed from workspace
- [x] All 40 tests pass in merged crate

**TODO 6.2: Rewrite testgen binary to use shared infrastructure** ✅
- [x] Testgen binary now uses `FileWriter` for all I/O (generate, check, dry-run)
- [x] Eliminated ~150 lines of reimplemented file I/O and staleness checking
- [x] Binary stays in `gunbc-dag/src/bin/testgen.rs` (needs DAG builder references)
- [ ] Future: could become `gunbc-codegen testgen` subcommand if circular dep is resolved

**TODO 6.3: Wire registry to test generation** ✅

The circular dep means the registry can't hold actual function references to
DAG builders. The `target!()` macro already achieves single-site registration
in testgen.rs — each expression is written once, the string form is derived
via `stringify!`. Adding a registry layer would reintroduce two-site duplication.

What the registry CAN do: advertise which tools have testgen, so Makefile/CI
generation can derive testgen targets automatically.

- [x] Added `.testgen(TestgenTargetDef)` to bootstrap and makegen in `all_tools()`
- [x] These configs are metadata-only (no function references)
- [x] Added `tool_testgen_targets()` to collect testgen configs from all tools
- [ ] Makefile generation reads `ToolDef.testgen` to auto-generate targets
- [ ] CI generation reads it to know what to check
- [x] testgen.rs remains the authority for actual generation (via `target!()`)

**TODO 6.4: Registry for non-tool DAGs**
- [ ] Library DAGs (llm-ops) need a `LibraryDagDef` or similar
- [ ] Multi-mock: llm-ops has 4 MockSpec variants for 1 DAG builder
- [ ] `all_testgen_dags()` collects from ToolDef.testgen + LibraryDagDef
- [ ] Makefile/CI derive targets from this combined list

**Design note — why two sites are unavoidable:**

The `target!()` macro in testgen.rs IS the single-site registration for test
generation. It holds both the metadata (via stringify) and the actual function
call. The registry can't replace it because:

1. `gunbc-codegen` can't depend on `gunbc-dag` (circular dependency)
2. Function references can't be stored as data in the registry
3. `stringify!` derives the string form from the expression, eliminating manual
   string duplication

The registry's role is to advertise testgen targets to OTHER consumers (Makefile,
CI), not to drive testgen itself. This is the same pattern as ToolDef.boundaries —
the registry stores CLI boundary metadata for codegen, while the actual DAG builder
functions live in the tool crates.

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

- Testgen module: `core/codegen/src/testgen/`
- Testgen binary: `gunbc-dag/src/bin/testgen.rs`
- MockSpec: `core/test/src/mock_spec.rs`
- Obligation model: `core/codegen/src/testgen/obligation.rs`
- Tool registry: `core/codegen/src/registry.rs`
