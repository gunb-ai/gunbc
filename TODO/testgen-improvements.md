# Testgen Improvements

**Status**: In Progress
**Date**: 2026-02-02
**Updated**: 2026-02-06

## Problem Statement

Current testgen has several deficiencies:

1. **Manual target registration** - historically required adding each DAG to a list (now resolved via registry auto-discovery)
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

// Desired: I/O examples are part of the mock spec
let spec = MockSpec::new("review")
    .node_example(
        NodeExample::new("prepare_prompt")
            .input("artifact", Value::Str("fn foo() {}".into()))
            .input("criteria", Value::Str("security".into()))
            .output("question", OutputMatcher::contains("security"))
            .output("system_prompt", OutputMatcher::non_empty())
    );
```

Then testgen generates:
```rust
#[test]
fn test_example_prepare_prompt_0() {
    let dag = build_review_graph();
    let mut inputs = std::collections::HashMap::new();
    inputs.insert("artifact".to_string(), Value::Str("fn foo() {}".into()));
    inputs.insert("criteria".to_string(), Value::Str("security".into()));
    let outputs = gunbc_exec::execute_single_node(&dag, "prepare_prompt", inputs, ExecutionMode::Real)
        .expect("node 'prepare_prompt' should execute successfully");
    assert!(outputs.get("question").unwrap().as_str().map(|s| s.contains("security")).unwrap_or(false));
    assert!(!outputs.get("system_prompt").unwrap().as_str().map(|s| s.is_empty()).unwrap_or(false));
}
```

## Key Insight

**Mocks and I/O examples serve the same purpose** - they specify what a node produces. The difference:

| | Transport Nodes | Pure Nodes |
|---|---|---|
| **What** | Mock response (external I/O) | Expected output (computed) |
| **Why** | Can't run real I/O in tests | Verify business logic |
| **How** | `MockSpec::boundary()` | `MockSpec::node_example()` |

Both should be **required**, not optional.

**Design decision**: Examples live on `MockSpec` (via `.node_example()`) rather than on `Node` directly. This keeps `Node<T>` generic and free of test concerns, while `MockSpec` already serves as the test specification.

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

**TODO 1.3: Auto-discover DAGs** ✅
- [x] Added `gunbc-testgen-registry` + `gunbc-testgen-registry-macros` using `inventory`
- [x] `#[testgen_target(...)]` on each MockSpec registers a `TestgenTarget`
- [x] Testgen binary iterates the registry (`iter_targets()`) and generates from `target.generate`
- [x] Removed manual target lists (`all_testgen_targets`, `library_testgen_targets`) and `target!()` macro
- [x] Added a test (`gunbc-dag/tests/mock_spec_registration.rs`) that enforces
      every `pub fn ...mock_spec -> MockSpec` has `#[testgen_target]` (or `skip`)

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
- *Implemented*: `core/test/src/mock_spec.rs:644-708`

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

### Phase 4: Generated Test Strategy

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

**TODO 5.1: Define `Window` type** ✅ (added in `gunbc-test::window`)
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

**TODO 5.2: Window enumeration** ✅ (topo-slice windows + connectivity + mixed-input filter)
- [x] Given a DAG, enumerate all valid windows (contiguous sub-DAG slices)
- [x] For a linear DAG of n nodes, this is O(n^2) windows (sliding window)
- [x] For branching DAGs, enumerate by cut boundaries — sever incoming edges at
      entry nodes, capture outgoing edges at exit nodes
- [x] Optionally limit window size (e.g., max 5 nodes) to control test explosion

**TODO 5.3: DryRun value capture** ✅ (baseline log per test; caching TBD)
- [x] Run full DryRun with existing MockSpec → `ExecutionLog`
- [x] Extract per-port values at window entry boundaries from the log
- [x] These become the injected inputs for the windowed execution

**TODO 5.4: Windowed execution** ✅ (window sub-DAG + injected inputs)
- [x] For each window, construct a sub-DAG or use `input_mock()` injection at
      the severed entry edges
- [x] Execute the window with DryRun mode (transport nodes still intercepted)
- [x] Pure nodes execute for real — this is where integration bugs surface
- [x] Compare window exit-port values against the full DryRun expected values

**TODO 5.5: Test generation** ✅ (auto-generated; default max window size = 5)
- [x] For each enumerated window, generate a test function
- [x] Test name encodes the window: `test_window_B_through_D`
- [x] Inputs derived from DryRun log, outputs verified against DryRun log
- [x] Consider grouping by window size or DAG region to organize output

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

Registry is now the single source of truth for testgen. Each MockSpec is
annotated with `#[testgen_target(...)]`, which registers a `TestgenTarget`
in an `inventory` registry. The testgen binary iterates the registry and
generates tests directly from those entries.

- [x] Added `gunbc-testgen-registry` + `gunbc-testgen-registry-macros`
- [x] Removed `.testgen(...)` from `ToolDef` and deleted `all_testgen_targets()`
- [x] Testgen binary uses `iter_targets()` (no manual list)
- [ ] Optional: Makefile/CI could consume the registry if explicit targets are needed

**Design note — linking requirement:**

Inventory only includes crates linked into the binary. `gunbc-dag/src/bin/testgen.rs`
force-links all crates that register testgen targets.

## Open Questions

1. ~~**Commit or not?**~~ → Commit with staleness checks (decided above)

2. **Auto-discover vs manifest?** Resolved via `inventory` + `#[testgen_target]`
   - Explicit annotation + registry auto-discovery, no manual list
   - Caveat: crates must be linked into the testgen binary to register

3. ~~**Example syntax?**~~ → Builder pattern on `NodeExample` (decided)
   - `NodeExample::new("node").input("port", val).output("port", matcher).description("...")`

4. **Inheritance?** If DAG A embeds DAG B as subdag, do B's examples run?
   - Probably yes - verify subdags work in context

## Success Criteria

After implementation:

1. **Can't forget tests** - testgen auto-discovers DAGs via registry ✅, fails if no MockSpec ✅, panics if pure nodes lack examples ✅
2. **Can't have stale tests** - staleness check in `make test` ✅, deterministic output ✅
3. **I/O is tested** - node examples generate unit tests ✅, all 7 targets have examples ✅
4. **Minimal ceremony** - just add `.node_example()` to MockSpec, tests appear ✅

### Phase 7: Enforcement & Coverage

**TODO 7.1: Enforcement for pure nodes** ✅
- [x] Added `skipped_node_examples: Vec<String>` field to `MockSpec`
- [x] Added `skip_node_example()` builder method for explicit opt-out
- [x] Added enforcement check in `generate_test_module()`: panics when pure nodes
  have no examples (from MockSpec or Node) and aren't explicitly skipped
- [x] Error message lists uncovered nodes with guidance on `.node_example()` or
  `.skip_node_example()`
- *Implemented in `codegen.rs:176-242`*

**TODO 7.2: Add node_examples to all MockSpecs** ✅
- [x] **makegen**: `load_registry` (tool_count ≥ 2, tool_names non-empty),
  `render_makefile` (contains "gist"), skip `prepare_file_write`
- [x] **bootstrap**: `prepare_scan_workspace` (request non-empty),
  `parse_scan_result` (skipped response propagation), `generate_makefile`
  (contains header), `generate_gitignore` (contains header), skip
  `prepare_makefile_write`, `prepare_gitignore_write`
- [x] **CI**: `report` (2 examples: all-pass → SUCCESS, build-fail → FAILURE),
  `parse_deps_exists`/`parse_codegen_exists` (skipped response propagation),
  `parse_codegen_result`/`parse_build`/`parse_test` (skip=true path with exact
  outputs), all `prepare_*` nodes (boolean output checks), `parse_clippy_lint`,
  skip `prepare_deps_exists`
- [x] **LLM** (4 variants): `prepare` (provider/model/messages → request + echoed
  provider), `parse` (skipped response propagation)

**TODO 7.3: Fix codegen bugs found during coverage** ✅
- [x] `to_check_code()` Exact matcher: `assert_eq!` → `assert_eq!(*...)` (deref)
- [x] `to_check_code()` Contains matcher: added `{:?}` format placeholder for
  value argument
- [x] `to_check_code()` all matchers: added trailing semicolons
- [x] `value_to_rust_literal()`: added `Value::Skipped` support
- [x] `generate_node_example_tests()`: sorted HashMap iteration for deterministic
  output (both inputs and outputs)

## Remaining Work

- **TODO 1.3**: Auto-discover DAGs (eliminate hardcoded builder map)

### Phase 8: Absorb Manual graph_mock.rs Tests

49 hand-written tests across 8 `graph_mock.rs` files follow patterns
that testgen already generates (or could generate with small additions).
Goal: make `graph_mock.rs` files **data-only** (MockSpec + NodeExamples
+ resources) and delete the `#[cfg(test)]` blocks.

**graph_mock.rs test counts (after consolidation cleanup):**

| File | Tests | Kept | Patterns |
|------|-------|------|----------|
| `bootstrap/graph_mock.rs` | 5→1 | 1 | typed builder (Pattern E) |
| `ci/graph_mock.rs` | 6→1 | 1 | typed builder (Pattern E) |
| `makegen/graph_mock.rs` | 5→1 | 1 | typed builder (Pattern E) |
| `lib/llm-ops/graph_mock.rs` | 9→5 | 5 | content validation, secret/lease |
| `lib/tools/gist/graph_mock.rs` | 9→5 | 5 | URL validity, mode-specific (Pattern B), typed builder (Pattern E) |
| `lib/tools/buck2/graph_mock.rs` | 7 | 7 | boundary presence (not yet cleaned) |
| `lib/tools/deps/graph_mock.rs` | 4→3 | 3 | content check, lease type, typed builder |
| `lib/review/graph_mock.rs` | 4→2 | 2 | utility, typed builder (Pattern E) |

**Patterns to absorb** (maps to testgen features needed):

**Pattern A: "MockSpec has boundary X" (presence checks)**
Most common. Tests that `mock_spec.boundaries.get("node").is_some()`.
Already covered by testgen Bucket A (transport interception).
*Safe to delete now* — if a boundary mock is missing, the generated
DryRun test will panic at execution time.

**Pattern B: "mock value has property" (content/URL checks)**
Tests that mock fixture values contain substrings, start with URLs, etc.
Two options:
1. Express as `NodeExample` outputs (preferred — tests node behavior,
   not fixture data)
2. Add matcher layer on mock fixtures themselves if fixtures are
   treated as golden data

**Pattern C: "validate_chain(spec, spec, empty_mapping)" (self-chain)**
Tests that a MockSpec chains with itself. Already generated by testgen
(chain validation tests in Bucket C).
*Safe to delete now* — exact same assertion generated automatically.

**Pattern D: "resource exists / lease expiration"**
Tests for resource configuration. Testgen already emits lease expiration
tests for `ResourceType::Lease`.
*Safe to delete once generated resource tests confirmed equivalent.*

**Pattern E: "signature matches DAG"**
Tests that DAG signature validates. Not yet generated by testgen.
Requires new testgen assertion (see TODO 8.2 below).

**TODO 8.1: Add transport-mock coverage assertion to testgen** ✅
- [x] Walk DAG analysis, find transport executor nodes
- [x] Assert MockSpec provides mocks for all output ports used downstream
- [x] This replaces per-tool boundary presence tests (Pattern A)
- [x] Subsumes ~25 of the 49 tests
- *Implemented in `codegen.rs:188-223` — panics if connected transport outputs lack mocks*

**TODO 8.2: Add signature validation assertion to testgen** ✅
- [x] If a `TestgenTargetDef` includes a signature, emit a test that
      calls `signature.validate(&dag)`
- [ ] Optionally: `infer_signature(&dag)` matches declared signature
- [ ] This replaces per-tool signature tests (Pattern E, consolidation §7 Pattern 3)

**TODO 8.3: Add mock-value type compatibility assertion to testgen** ✅
- [x] For each mock value in MockSpec, assert `Value` type is compatible
      with the DAG port's `type_id` (and cardinality)
- [x] Contract-level check: "mock is Bool-typed" not "mock == Bool(true)"
- [x] Catches type drift between MockSpec and DAG port definitions
- *Implemented in `codegen.rs:230-254` + `find_mock_type_mismatches()` helper*

**TODO 8.4: Delete redundant graph_mock.rs tests**
- [x] Delete Pattern A tests (boundary presence) — 22 tests deleted (10 earlier + 2 gist + 10 consolidation round: 3 bootstrap + 3 ci + 3 makegen + 2 deps + 4 review)
- [x] Delete Pattern C tests (self-chain) — 3 tests deleted (1 llm-ops + 2 gist)
- [x] Delete Pattern D tests (resource presence) — 11 tests deleted (4 earlier + 1 gist + 6 consolidation round: 1 bootstrap + 2 ci + 1 makegen + 1 deps + 1 llm-ops)
- [ ] Migrate Pattern B tests to NodeExamples — then delete
- [ ] Delete Pattern E tests — once TODO 8.2 lands
- [ ] Goal: graph_mock.rs files contain only `pub fn mock_spec()` + data

### Phase 9: DagSpec End-State

Unify DAG builder location, MockSpec, signature, and testgen registration
into a single `DagSpec` definition. This is the concrete form of
"DAG definition = test specification."

**Current**: Adding a new tool DAG requires edits in 2-3 places:
1. Builder function in tool crate
2. MockSpec in `graph_mock.rs` with `#[testgen_target(...)]`
3. (optional) signature tests in `graph.rs`

**Goal**: One `DagSpec` per DAG that carries everything:

```rust
pub struct DagSpec<T> {
    /// Builds the DAG
    pub builder: fn() -> Result<Dag<T>>,
    /// Test specification (mocks, examples, resources)
    pub mock_spec: MockSpec,
    /// Expected interface contract (optional)
    pub signature: Option<DagSignature>,
    /// Testgen configuration
    pub testgen: TestgenTargetDef,
}
```

**Blocked on**: Phase 8 (absorbing manual tests first), resolving the
circular dependency between `gunbc-codegen` and tool crates (builder
functions live in tool crates, DagSpec would need to reference them).

**TODO 9.1: Design DagSpec type**
- [ ] Define what fields DagSpec carries
- [ ] Resolve circular dep (likely: DagSpec metadata in codegen,
      builder fn reference stays in tool crate via registration)

**TODO 9.2: Migrate targets to DagSpec**
- [ ] Convert `#[testgen_target]` registrations to DagSpec instances
- [ ] Each tool crate exports a `dag_specs()` function
- [ ] Testgen, Makefile gen, and CI gen all consume DagSpec

---

## Phase 10: Credential Lifecycle Testing

Additions needed for auto-generating credential lifecycle tests.
See [TODO_credential_lifecycle.md](TODO_credential_lifecycle.md) for
the credential design that drives these requirements.

**Depends on**: Phase 5 (windowed testing), Phase 8 (test absorption)

**TODO 10.1: Add `ResourceType::Credential` to resource simulation** ✅
- [x] Add `ResourceType::Credential { expiry_ms, refreshable }` to `mock_spec.rs`
- [x] Add `ResourceBehavior::RefreshSucceeds { new_ttl_ms }`
- [x] Add `ResourceBehavior::RefreshFails { error }`
- [x] Add `ResourceBehavior::RevokeSucceeds`
- [x] Generate: acquire, use-while-valid, refresh, use-after-refresh,
      expire, revoke test sequence in Bucket D

**TODO 10.2: Add `MockSequence` / `ConditionalMock` to MockSpec** ✅
- [x] Add `MockSequence`: ordered list of responses per transport node
      (first call → response A, second call → response B)
- [ ] OR `ConditionalMock`: predicate on input values selects response
- [x] Wire into `BoundaryMocks` / DryRun interception in `execute.rs`

**TODO 10.3: Generate credential lifecycle suites** ✅
- [x] Detect `CredentialOp` nodes in DAG analysis
- [x] Generate Bucket C scenarios specific to credential flows:
      acquire-fails, use-with-expired, refresh-succeeds, revoke-then-use
- [x] Use Phase 5 windows to test sub-segments of the credential flow

---

## Phase 11: IR Completeness & Language Idioms

**See [design-codegen-quality.md](design-codegen-quality.md)** for the full
treatment of this topic. It applies to all code generation, not just testgen.

**Summary**: The test IR must model idiomatic Rust patterns. When clippy fires
on generated code, it usually reveals an IR modeling gap (e.g., missing
`Stmt::TailExpr` for implicit returns). The IR should be complete enough that
generated code passes all linters with no `#[allow(...)]` escapes.

**Case study**: The `needless_return` bug (2026-02-05) showed that `Stmt::Return`
was insufficient — we needed `Stmt::TailExpr` for Rust's idiomatic tail expression
pattern. Fixed by adding the variant and using `Stmt::tail()` in helper generation.

## References

- Testgen module: `core/codegen/src/testgen/`
- Testgen binary: `gunbc-dag/src/bin/testgen.rs`
- Testgen registry: `core/testgen-registry/`
- Testgen macro: `core/testgen-registry-macros/`
- MockSpec: `core/test/src/mock_spec.rs`
- Obligation model: `core/codegen/src/testgen/obligation.rs`
- Tool registry: `core/codegen/src/registry.rs`
- MetaTarget extra_deps: `gunbc-dag/src/makegen/registry.rs`
