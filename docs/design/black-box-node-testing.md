# Black-Box Node Testing: Cross-Workflow Mock Accumulation

> **Goal**: Generalize testgen's per-workflow baseline capture into a
> cross-workflow corpus keyed by stable node identity, so that
> every node is tested against every input context it appears in —
> without manual MockSpec authoring.

**Status**: Draft — February 2026
**Depends on**: [`testgen.md`](testgen.md), [`integration-testgen.md`](integration-testgen.md)

---

## Problem

Every node in the DAG is a black box with typed input/output ports. Today, test obligations are derived per-workflow: a node's mock values come from its MockSpec in one specific workflow context. But the same node (e.g., `shared.dag_util::doc`) appears in many workflows — pragma, gist, bootstrap, makegen — each feeding it different upstream values. A bug in one wiring context (e.g., port name vs parameter name mismatch) goes undetected if only one workflow's MockSpec exercises that node.

**Concrete example**: `FnBodyDelegate` broke `shared.dag_util::doc` in pragma/gist/bootstrap but not makegen (which used a direct evaluation path). No generated test caught it because each workflow tested the node with its own DryRun mocks, and the DryRun passthrough always succeeded.

## Insight

A node doesn't care which workflow it's in. It receives values on input ports and produces values on output ports. The **set of valid inputs** for a node is the **union of all upstream outputs across all workflows** that feed it. Similarly, the **set of expected outputs** is the union of all downstream inputs that consume from it.

If we accumulate (input, output) mock pairs across all workflows and test every node against every accumulated pair, we get cross-workflow regression coverage without writing manual integration tests.

## Model

### Core Abstraction: The Node Mock Corpus

For each unique node identity `N` (identified by module + callable name), build a **mock corpus** of paired, provenance-tracked cases:

```rust
/// Stable node identity across workflows
struct NodeIdentity {
    module: String,       // e.g., "shared.dag_util"
    callable: String,     // e.g., "doc"
}

/// One test input + what we're allowed to assert about its output
struct CorpusExample {
    provenance: Provenance,
    inputs: HashMap<PortName, Value>,
    expectation: Expectation,
}

struct Provenance {
    workflow: String,                  // module path + entry func
    profile: Option<String>,           // binding mode (unit_test, local, etc.)
    node_instance: NodeId,             // instance id in that DAG
    subdag_path: Vec<NodeId>,          // [] for top-level, [parent, ...] for nested
    seed_kind: SeedKind,               // how this example was produced
}

enum SeedKind {
    WorkflowObserved,    // captured from baseline DryRun
    ExplicitMockSpec,    // human-curated NodeExample
    TypeDerived,         // generated from type DAG witnesses
    PropertyBased,       // fuzz/proptest
}

/// What can we assert about outputs for this example?
enum Expectation {
    /// Ground-truth outputs known to be deterministic + stable (pure nodes only)
    ExactOutputs(HashMap<PortName, Value>),
    /// Human-provided matchers (strongest non-exact assertions)
    OutputMatchers(HashMap<PortName, OutputMatcher>),
    /// Only assert type/shape + no-crash invariants (default for most sources)
    TypeContractOnly,
    /// Negative boundary: expect validation error (separate test family)
    ExpectValidationError,
}

struct MockCorpus {
    examples: Vec<CorpusExample>,
}
```

Each example pairs inputs with **exactly one expectation** — no split bags,
no ambiguity about what to assert. Provenance tracks origin (workflow, profile,
subdag path, seed kind) for debugging ("this failing case came from
`tools/pragma.dag` entry `render_clippy_toml`, seed: WorkflowObserved") and
stable test naming (`test_node_{id}_workflow_{workflow}_input_{hash}`).

**Sources** (each maps to a specific `Expectation`):

1. **Workflow-observed** (`SeedKind::WorkflowObserved`): Execute each workflow
   in DryRun, capture per-node inputs from upstream baseline outputs (same
   logic as `apply_window_inputs` for a 1-node window). DryRun is a
   **provenance extractor**: it provides realistic *inputs* but outputs
   captured from DryRun are only used as expectations when the node is
   deterministic and pure. Expectation assignment:
   - Pure + deterministic node → `ExactOutputs` (re-execute in Real mode, compare)
   - Effectful/boundary node → `TypeContractOnly` (output shape + no-crash only)

2. **Explicit NodeExamples** (`SeedKind::ExplicitMockSpec`): Human-curated
   `NodeExample` entries from MockSpec. These carry the strongest assertions.
   Expectation: `OutputMatchers` (when matchers provided) or `ExactOutputs`
   (when exact outputs specified).

3. **Type-derived** (`SeedKind::TypeDerived`, positive boundary only): For each
   input port with type `T`, generate guard-satisfying boundary values from
   `T`'s constraints. Expectation: always `TypeContractOnly` — these inputs
   have no known-correct output, only type/shape invariants.
   - `String @non_empty` → `"x"`, `"hello world"` (only valid values)
   - `Int @range(min: 0, max: 5)` → `0`, `1`, `3`, `5` (in-range only)
   - `List<T>` → `[]`, `[one]`, `[one, two, three]` (Fermi cardinality)
   - `Bool` → `true`, `false`
   - `Option<T>` → `Unit`, `boundary_values(T)[0]`
   - Records → anchored mutation of field witnesses (see below)
   - Sum types → one value per variant

4. **Property-based** (`SeedKind::PropertyBased`): For structurally-generatable
   types, proptest/fuzz with valid inputs. Expectation: `TypeContractOnly`
   (crash-freedom + output validates against declared type). Gated by cost.

**Invalid-value testing** (negative boundary) is a separate test family
using `Expectation::ExpectValidationError`: generate values that violate port
guards/refinements, and assert the system **rejects** them at the
injection/typecheck boundary. These are not mixed into the main corpus — they
test the guard infrastructure, not node logic.

### Accumulation Across Workflows

```
         ┌─────────┐     ┌─────────┐     ┌──────────┐
         │ pragma  │     │  gist   │     │ makegen  │
         └────┬────┘     └────┬────┘     └────┬─────┘
              │               │               │
              ▼               ▼               ▼
         ┌────────────────────────────────────────┐
         │         Compile all workflows          │
         │    (DryRun / MockSpec / type-derived)   │
         └────────────────────┬───────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │     Per-Node Mock Corpus Builder       │
         │                                         │
         │  For each node N across all workflows:  │
         │    collect CorpusExamples               │
         │    assign Expectation per seed kind     │
         │    merge with type-derived witnesses    │
         │    deduplicate by (workflow, hash(in))  │
         └────────────────────┬───────────────────┘
                              │
                              ▼
         ┌────────────────────────────────────────┐
         │        Test Obligation Generator        │
         │                                         │
         │  For each CorpusExample:                │
         │    execute node N with example.inputs   │
         │    assert per example.expectation       │
         └────────────────────────────────────────┘
```

### Pure vs Effectful: Same Model, Different Mock Depth

The model treats every node identically. The distinction between pure (`fn`) and effectful (`func`) nodes is only about **what the output port carries**:

| Node kind | Input ports | Output ports | Mock strategy |
|-----------|-------------|--------------|---------------|
| Pure `fn` | Parameter values | Computed result | Execute node, assert output |
| Effectful `func` | Parameter values + resource handles | Result + transport side-effects | Mock transport, assert result |
| Transport executor | `TransportRequest` | `TransportResponse` | Intercept, return mock response |

A pure node `fn doc(sections: List<DocumentSection>) -> Document` is tested exactly like an effectful node — feed inputs, check outputs. The transport executor is just a node whose output happens to come from an external system (mocked in hermetic tests, real in live tests).

### Adjacent Mock Pairs

For a node `B` with input port `x`, the **adjacent upstream mock** is the output value of whatever node `A` feeds port `x` — accumulated across all workflows where `A → B` exists:

```
Workflow 1:  [render_header] --header_text--> [doc]
Workflow 2:  [render_config] --config_doc---> [doc]
Workflow 3:  [render_policy] --policy_doc---> [doc]

Adjacent mock pairs for doc.sections:
  { sections: <render_header's output> }    // from pragma workflow
  { sections: <render_config's output> }    // from bootstrap workflow
  { sections: <render_policy's output> }    // from makegen workflow
```

For node `B`'s outputs, the **adjacent downstream mock** is the expected input of whatever node `C` consumes from `B`:

```
[doc] --document--> [render_document_section]
[doc] --document--> [render_document_line]

Adjacent downstream assertions for doc.return:
  output must satisfy render_document_section's input type (Document)
  output must satisfy render_document_line's input type (Document)
```

This gives us **inter-node contract testing** without full-workflow execution.

---

## Anti-Tautology Rule

> Only generate assertions that prove something **not already proven** by
> compile-time validation or existing DryRun interception tests.

Corpus-derived tests must avoid asserting "DryRun returns the mock I configured."
This means:

- **Pure nodes**: Can assert stronger properties (structural shape, or exact output
  when an explicit `OutputMatcher` exists).
- **Effectful/boundary nodes**: Assert *structural* properties only (output ports
  exist, type/shape compatible, node was intercepted) — not exact mock values.
- **Exact-output assertions**: Opt-in only, via `NodeExample` / `OutputMatcher`
  or deterministic pure nodes with `ExactOutputs`. Never the default for
  effectful/boundary nodes.

### DryRun Capture Semantics

> DryRun is a **provenance extractor**: it provides realistic *inputs* (and
> sometimes outputs), but outputs captured from DryRun are only used as
> expectations when they are known to be computed/stable. Otherwise they seed
> `TypeContractOnly` tests.

This prevents the tautology where a test asserts "DryRun returned the mock
I configured." For pure nodes executed in `ExecutionMode::Real`, DryRun-captured
outputs serve as ground truth (re-execution should produce the same result).
For effectful nodes, DryRun outputs are discarded — only the *inputs* extracted
from upstream baseline values are used.

---

## Normalization, Redaction, and Size Policy

Once we capture values flowing through ports, some will be nondeterministic,
environment-dependent, large, or sensitive. Define policy now to avoid
retrofitting later.

**Normalization** (applied before dedup and storage):
- Canonicalize map key ordering (sorted)
- Canonicalize paths: replace tempdir prefixes with `<TMP>`, home with `<HOME>`,
  cwd with `<CWD>`
- Replace known nondeterministic fields (timestamps, UUIDs) with stable
  placeholders if the field type is annotated `@nondeterministic` or detected
  heuristically

**Redaction** (applied before storage, never reversed):
- If a port type is `Secret`, `Credential`, or a `@brand("Secret*")` type,
  store only a `<REDACTED>` placeholder. These ports require explicit
  MockSpec seeds to test — never auto-generated.
- Resource handles (`Handle`, `tool:*` ports): store placeholder, not value.

**Size limits**:
- Cap stored `Value` serialization at 64 KB per port. Larger values stored as
  fixtures (files) with content hash reference, not inline Rust literals.
- `max_examples_per_node`: default 50. Beyond this, sample by provenance
  diversity (prefer examples from distinct workflows over duplicates from one).

---

## Test Generation Strategy

### Level 1a: Execution + Output Shape (default for all nodes, XS cost)

For each node `N` and each `CorpusExample` in its corpus, verify the node
executes and produces well-shaped output. The assertion level is determined
by `example.expectation`:

```rust
#[test]
fn test_node_dag_util_doc_workflow_pragma() {
    let example = corpus.example("shared.dag_util::doc", "pragma");
    let outputs = execute_single_node("shared.dag_util::doc", example.inputs);

    match &example.expectation {
        TypeContractOnly => {
            // Assert required ports exist + type/shape-compatible
            assert!(outputs.contains_key("return"));
            assert_matches!(outputs["return"], Value::Map(_));
        }
        ExactOutputs(expected) => {
            // Pure + deterministic: assert actual == expected
            assert_eq!(outputs, *expected);
        }
        OutputMatchers(matchers) => {
            for (port, matcher) in matchers {
                assert!(matcher.check(&outputs[port]));
            }
        }
        _ => {}
    }
}
```

For **pure nodes**, `execute_single_node` uses `ExecutionMode::Real` (no mocks
needed — the node is pure computation). For **effectful nodes**, it uses
`ExecutionMode::DryRun` with the workflow's boundary mocks.

The `ExactOutputs` and `OutputMatchers` branches are only reached when the
corpus contains examples with those expectations (human-curated or
deterministic-pure). `TypeContractOnly` is the default — this preserves
anti-tautology: exact assertions come from intentional human input or
provably stable computation.

### Level 2: Adjacent Pair Tests (Hermetic, S cost)

Adjacent pair tests must execute through the **window/subgraph executor**, not
manual port-map feeding. This is critical: the motivating bug (param name vs
port name mismatch) lives in the executor wiring, not in the values themselves.

For each workflow edge instance, capture an `EdgeExample` during DryRun:

```rust
struct EdgeExample {
    provenance: Provenance,
    edge: Edge,                        // A.out_port → B.in_port
    a_inputs: HashMap<PortName, Value>,
    b_other_inputs: HashMap<PortName, Value>,  // B's non-edge inputs
}
```

Then test using `Window::from_nodes` — the same executor path workflows use:

```rust
#[test]
fn test_edge_render_header_to_doc_pragma() {
    let edge = corpus.edge("render_header", "doc", "pragma");

    // Build a 2-node window through the real executor wiring
    let window = Window::from_nodes(&dag, &[edge.a_id, edge.b_id]);
    let window_inputs = edge.a_inputs.clone();
    window_inputs.extend(edge.b_other_inputs.clone());

    // Execute the 2-node subgraph via the same executor pipeline
    let outputs = execute_window(&window, window_inputs, ExecutionMode::Real);

    // Assert B executed successfully through the real wiring
    assert!(outputs.contains_key("return"));
    assert!(!matches!(outputs["return"], Value::Skipped));
}
```

This catches **runtime wiring mismatches**: port name mapping, value passing
through the executor, param→port translation. Manual `hashmap!` feeding would
bypass exactly the plumbing where bugs live.

**Scope**: Start with edges where **both nodes are pure** (no transport mocking
complexity). Extend to mixed edges after the pure-only path is proven.

### Level 3: Chain Tests (Hermetic, M cost)

Existing probe-observer model, enriched with corpus mock pairs. For each
(probe, observer) pair in each workflow, extract the subgraph and execute the
chain with corpus-derived inputs at the probe.

### Level 4: Cross-Workflow Regression (Hermetic, S cost)

For each node that appears in multiple workflows, verify consistent behavior.
**Only for nodes classified as deterministic** (pure + no resources + no
time/env reads). Otherwise "expected differences" are really "implicit
dependencies not modeled yet" — useful signal but disruptive to gate on.

```rust
#[test]
fn test_node_dag_util_doc_cross_workflow_consistency() {
    let cases = corpus.cases_for("shared.dag_util::doc");
    for case in &cases {
        let outputs = execute_single_node("shared.dag_util::doc", case.inputs.clone());
        assert!(outputs.contains_key("return"),
            "node doc failed in workflow {}", case.workflow);
        // Same output shape across all workflows
        assert_matches!(outputs["return"], Value::Map(_),
            "node doc wrong shape in workflow {}", case.workflow);
    }
}
```

**This is the test that would have caught the FnBodyDelegate regression**: the
pragma/gist workflows would have failed while makegen succeeded, immediately
surfacing the inconsistency.

### Window Tests: Complementary, Not Superseded

Windowed segment tests prove a **different invariant** than adjacent-pair tests:

> "subgraph extraction + input injection + execution yields the same outputs as
> the full graph run under the same boundary mocks"

That's a decomposition/infrastructure correctness property. Adjacent-pair tests
prove local inter-node compatibility. Both are valuable:

| Test type | What it proves |
|-----------|---------------|
| Window | Subgraph runner correctness (decomposition invariant) |
| Adjacent pair | Local inter-node wiring compatibility |
| Probe-observer chain | End-to-end value propagation through subgraphs |
| Cross-workflow | Same node identity behaves consistently everywhere |

---

## The Type DAG as a Compositional Source of Free Mock Pairs

The type system itself is a DAG. Types compose from primitives through refinements, containers, records, and sums — and **every node in that type DAG should generate test obligations automatically**. This is the key insight: the same set-algebraic composition that defines types also defines the space of valid test inputs.

### Type DAG Structure

Types are represented as `Dag<TypeOp>` — the same DAG substrate used for workflow graphs. Each type is a composition of operations:

```
TypeOp::Identity      — base type (String, Int, Bool, ...)
TypeOp::Validate      — predicate refinement (@non_empty, @range, @pattern)
TypeOp::Wrap          — container (List, Set, Optional, Map)
TypeOp::Product       — record (named typed fields)
TypeOp::Coproduct     — sum type (tagged union of variants)
TypeOp::Brand         — nominal wrapper (TextFilePath = FilePath @content(Text))
TypeOp::Transform     — coercion between base types
```

The `TypeLayer` decomposition (`contract.rs:458`) recursively walks this DAG:

```
List<Optional<Int @range(0,100)>>
  ├─ Layer 0: Wrap(List)           → cardinality [0,∞)
  ├─ Layer 1: Wrap(Optional)       → cardinality [0,1]
  └─ Layer 2: Identity(Int)        → base "Int"
       └─ Validate(InRange(0,100)) → predicate refinement
```

**Every node in this decomposition contributes test obligations:**

| Type DAG node | Test obligation | Free mock pairs |
|---------------|----------------|-----------------|
| `Wrap(List)` | Cardinality coverage | `[]`, `[one]`, `[one, two, three]` (Fermi: 0, 1, 3) |
| `Wrap(Optional)` | Presence/absence | `Unit` (None), `Some(inner_witness)` |
| `Identity(Int)` | Base type witness | `1` (default scalar) |
| `Validate(InRange(0,100))` | Boundary lattice | `-1`, `0`, `50`, `100`, `101` |
| `Product(fields)` | Per-field + cross-product | Each field's witnesses combined |
| `Coproduct(variants)` | Per-variant coverage | One witness per variant arm |
| `Brand("X", inner)` | Inner type witnesses | Delegates to inner type |

### Existing Infrastructure (Already Built)

The witness generation pipeline is fully operational in `core/ir/src/contract.rs`:

- **`witnesses(type_dag)`** (line 154): Generates `Vec<BoundaryWitness>` from any type DAG. Each witness is a `(count, Value)` pair representing a boundary point.

- **`TypeLayer::from_type_dag()`** (line 477): Recursively decomposes types into layers. Each layer has cardinality, base type, predicates, wrapper, inner type, coproduct arms, and product fields.

- **`cross_product_witnesses(type_dag, depth)`** (line 547): For nested types, generates cross-product of inner witnesses at each layer up to a depth limit.

- **`predicate_boundary_witnesses(pred, base)`** (line 669): Generates values at predicate lattice transition points (below-min, at-min, mid, at-max, above-max).

- **`variant_witnesses(type_id, registry)`** (line 308): For coproduct types, generates one witness per variant arm.

- **`scalar_witness_for_base(base, preds)`** (line 279): Generates refined scalar witnesses (respects `@non_empty`, `@range`, `@pattern` etc.).

- **`fermi_test_cases(cardinality)`** in `cardinality.rs`: Generates boundary counts `[0, 1, 3]` for ZeroOrMore, `[1, 2]` for OneOrMore, etc.

### How Type Nodes Generate Free Mock Pairs

For a port with type `List<DocumentSection>` where `DocumentSection = { title: String, lines: List<String> }`:

```
Type decomposition:
  Wrap(List) → cardinality [0,∞)
    Product({ title: String, lines: List<String> })
      Identity(String) → "example"
      Wrap(List) → cardinality [0,∞)
        Identity(String) → "example"

Generated mock pairs (free, no author input):
  count=0: []                                           // empty list
  count=1: [{ title: "example", lines: [] }]            // one section, no lines
  count=1: [{ title: "example", lines: ["example"] }]   // one section, one line
  count=3: [{ ... }, { ... }, { ... }]                  // Fermi "many"

Predicate-refined (if @non_empty on title):
  count=1: [{ title: "x", lines: [] }]                  // minimal non-empty
```

Each generated mock pair is a valid input set for any node with a `List<DocumentSection>` input port. **The type DAG literally tells us what to test** — we just walk its nodes.

### Cardinality as the Primary Fuzzing Dimension

Cardinality (`core/ir/src/types.rs:33`) is the most productive fuzzing dimension because it's:

1. **Universal**: Every port has a cardinality (scalars are `[1,1]`, optionals are `[0,1]`, lists are `[0,∞)`, non-empty lists are `[1,∞)`)
2. **Boundary-rich**: The `boundary_values()` method generates out-of-range values too (for negative testing)
3. **Compositionally free**: A `List<Optional<T>>` combines list cardinality `[0,∞)` with optional cardinality `[0,1]` — the cross-product generates cases like "list with 3 elements, 2 present and 1 None"
4. **Bug-productive**: Most real bugs involve edge cases: empty collections, missing optionals, single-element lists where code assumes many. Cardinality-driven testing targets exactly these.

The Fermi constant (`FERMI_MANY = 3`) keeps the count small while still exercising multi-element behavior. Combined with `allows_count()` for validity checking, the system generates only valid boundary inputs — no wasted test runs.

```
Port: sections: List<DocumentSection>
Cardinality: [0,∞) → Fermi cases: [0, 1, 3]

Test input sets (free):
  { sections: [] }                               // 0 elements: does node handle empty?
  { sections: [witness] }                         // 1 element: does node handle single?
  { sections: [witness, witness, witness] }       // 3 elements: does node handle many?

Port: prefix: String?
Cardinality: [0,1] → cases: [0, 1]

Test input sets (free):
  { prefix: Unit }                                // 0: does node handle None?
  { prefix: "example" }                           // 1: does node handle Some?
```

### Set-Algebraic Composition of Test Obligations

The type algebra composes with set operations. For a node with multiple typed
input ports, the test obligation space is:

```
node inputs:
  port_a: T_a    → witnesses(T_a) = {a1, a2, a3}
  port_b: T_b    → witnesses(T_b) = {b1, b2}
  port_c: T_c?   → witnesses(T_c?) = {Unit, c1}

Full cross-product (exhaustive):
  |{a1,a2,a3}| × |{b1,b2}| × |{Unit,c1}| = 12 test cases
```

Full cross-products are wasteful and create unrealistic input combinations
(ports often have **semantic correlations** not expressed in the type DAG).
The default strategy is **anchored mutation**:

**Anchored mutation** (default):
1. Pick 1-3 **base** input maps from workflow-observed examples (realistic correlations)
2. For each base, vary **one port at a time** across its witness set while
   holding other ports fixed at their base values
3. Always include the unmodified base cases

```
Base case (from pragma workflow):
  { port_a: a_real, port_b: b_real, port_c: c_real }

Anchored mutations:
  { port_a: a1,     port_b: b_real, port_c: c_real }  // vary a only
  { port_a: a2,     port_b: b_real, port_c: c_real }
  { port_a: a3,     port_b: b_real, port_c: c_real }
  { port_a: a_real, port_b: b1,     port_c: c_real }  // vary b only
  { port_a: a_real, port_b: b2,     port_c: c_real }
  { port_a: a_real, port_b: b_real, port_c: Unit   }  // vary c only
  { port_a: a_real, port_b: b_real, port_c: c1     }

Total: 1 base + 7 mutations = 8 test cases (not 12)
```

This gets realistic correlations, excellent bug yield (single-port boundary
bugs are the most common class), and a predictable upper bound:
`bases × sum(witnesses_per_port)` instead of `product(witnesses_per_port)`.

**Pairwise cross-product**: Opt-in per node (for nodes where port interactions
are known to matter). Reduces N^k to N^2 but still generates unrealistic
combinations. Capped by `max_test_cases_per_node`.

Additional pruning:
- **Workflow-prioritized**: Workflow-observed inputs always included as bases;
  type-derived fill remaining single-port boundary gaps.
- **Fermi-bounded**: Cap at `FERMI_MANY` (3) elements per collection dimension.

### Semantic Carrier Boundary

Types classified as `SemanticCarrier` (TransportRequest, Credential, etc.) are NOT auto-generated. The `SemanticCarrierKind` enum (`types.rs:608`) explicitly partitions the type universe:

- **Structural** (auto-generatable): String, Bool, Int, Float, List, Set, Map, Optional, refined primitives (Url, FilePath, etc.)
- **TransportRequest/Response**: Require `@mock_response` or `Mockable::mock_outputs()`
- **Credential/Secret/Handle**: Require explicit seeds from `MockSpec`

This preserves the anti-tautology invariant: semantic values always come from intentional sources. The type DAG generates free mock pairs only for the structural portion — which is exactly the portion where cardinality and refinement constraints make the generation sound.

---

## Transport-Defined Fidelity Ladders

### The Missing Layer: Transport-Scoped Test Policy

The existing test system has `FermiCost` (XS/S/M/L/XL) and `TestClass` (Unit/Hermetic/Integration), but these are assigned per-test-function. What's missing is the connection between **transport type** and **fidelity level** — the idea that a Filesystem transport at XS means "pure mock", at S means "virtual filesystem", at M means "sandboxed real filesystem", etc.

Each `TransportKind` should define its own **fidelity ladder**: a mapping from Fermi tier to execution strategy. Nodes that depend on a transport inherit that transport's ladder. The same black-box test logic runs at every fidelity level — same inputs, but the transport boundary resolves differently.

### Per-Transport Fidelity Ladders

```
TransportKind::File
  XS: Pure mock          — DryRun intercept, return Value::Str("mock content")
  S:  Virtual filesystem  — in-memory fs (tempfile + HashMap), hermetic
  M:  Sandboxed real fs   — real tempdir, real read/write, cleaned up after
  L:  Real filesystem     — actual working directory, not sandboxed
  XL: Remote filesystem   — network-mounted fs, cloud storage

TransportKind::Shell
  XS: Pure mock           — DryRun intercept, return ShellResponse::ok("")
  S:  Recorded replay     — replay captured stdout/stderr from prior real runs
  M:  Sandboxed shell     — real shell in isolated env (nsjail/bubblewrap/chroot)
  L:  Real shell          — actual shell execution, local machine
  XL: Remote shell        — SSH/container execution on remote host

TransportKind::Rest (HTTP API)
  XS: Pure mock           — DryRun intercept, return mock JSON
  S:  Contract stub       — local HTTP server validating request schema
  M:  Recorded replay     — replay captured HTTP exchanges (VCR/cassette style)
  L:  Real API (staging)  — actual API calls to staging/sandbox environment
  XL: Real API (prod)     — actual API calls to production, gated by secrets

TransportKind::Http
  XS: Pure mock           — DryRun intercept
  S:  Local server        — localhost HTTP server with canned responses
  M:  Proxy replay        — mitmproxy-style recorded exchanges
  L:  Real HTTP           — actual network calls
  XL: Real HTTP (cross-region) — latency-sensitive, multi-hop

TransportKind::Tcp
  XS: Pure mock           — DryRun intercept
  S:  Loopback            — localhost TCP, hermetic
  M:  Local network       — real TCP on local interface
  L:  Real network        — actual remote TCP connections
  XL: Real network (adverse) — high-latency, packet-loss simulation

TransportKind::LocalDirect
  XS: Pure mock           — DryRun intercept (identity passthrough)
  S:  Real execution      — in-process, no I/O boundary
  (no higher tiers — LocalDirect is inherently hermetic)
```

### How Nodes Inherit Transport Tiers

A node's **maximum available fidelity** is determined by the transports it depends on. The executor resolves this transitively:

```
fn render_makefile_content(...)        → pure fn, no transport
  max_fidelity: XS (no transport dependency, always hermetic)

fn write_makefile(content, path)       → uses Filesystem.write
  max_fidelity: XL (limited by Filesystem ladder)
  at XS: content validated against type, write intercepted
  at S:  content written to virtual fs, read back, diffed
  at M:  content written to tempdir, verified on disk

func clippy_lint(paths) uses clippy    → uses Shell transport
  max_fidelity: XL (limited by Shell ladder)
  at XS: shell command intercepted, return mock ShellResponse
  at S:  replay recorded clippy output
  at M:  run clippy in sandboxed env with fixture crate

func create_gist(content) uses github  → uses REST transport
  max_fidelity: XL (limited by REST ladder)
  at XS: HTTP request intercepted, return mock JSON
  at S:  local stub server validates request schema
  at L:  real GitHub API call (requires GITHUB_TOKEN)
```

When a node depends on **multiple transports**, its fidelity at tier T is the **meet** (minimum) of all its transports' capabilities at tier T. If a node uses both File (available at S) and REST (only mock at XS), the node can only run at XS unless the REST transport is also available at S.

### Test Policy: Fidelity Tiers as Test Sets

Each Fermi tier defines a test set. `make test-all` runs `≤ S` by default (the hermetic ceiling):

```
make test-all                     → runs XS + S tests (hermetic, cheap)
GUNBC_TEST_MAX_COST=M make test-all  → runs XS + S + M tests (sandboxed I/O)
GUNBC_TEST_MAX_COST=L make test-all  → runs XS + S + M + L tests (real local I/O)
GUNBC_TEST_MAX_COST=XL make test-all → runs everything (real remote I/O, needs secrets)
```

At each tier, the **same black-box test logic** executes — same mock corpus inputs, same output assertions. Only the transport resolution changes:

```rust
// Generated test for write_makefile at tier S
#[test]
fn test_write_makefile_tier_s() {
    let inputs = corpus.inputs_for("write_makefile", "makegen");
    let fs = VirtualFilesystem::new();  // S-tier: virtual fs
    let outputs = execute_node_with_transport("write_makefile", inputs, &fs);
    // Same assertions as XS tier
    assert!(outputs.contains_key("return"));
    // Additional S-tier assertion: content was written to virtual fs
    assert_eq!(fs.read("Makefile"), expected_content);
}

// Generated test for write_makefile at tier M
#[test]
#[cfg(feature = "sandboxed_tests")]
fn test_write_makefile_tier_m() {
    let inputs = corpus.inputs_for("write_makefile", "makegen");
    let tempdir = tempfile::tempdir().unwrap();  // M-tier: sandboxed real fs
    let outputs = execute_node_with_transport("write_makefile", inputs, &tempdir);
    assert!(outputs.contains_key("return"));
    // Additional M-tier assertion: file exists on disk
    assert!(tempdir.path().join("Makefile").exists());
}
```

### Transport Fidelity as a Type-Level Concept

The fidelity ladder should be a first-class concept in the type system, not a testgen-only concern. Each transport defines its ladder as data:

```dag
resource Filesystem {
  kind: Capability

  fidelity {
    XS: PureMock         // DryRun intercept
    S:  VirtualFs         // in-memory hermetic
    M:  SandboxedFs       // real tempdir
    L:  RealFs            // unsandboxed local
    XL: RemoteFs          // network-mounted
  }

  capability read { @file(READ, "{path}") }
  capability write { @file(WRITE, "{path}") }
}
```

This makes the fidelity ladder **structural** — the compiler can:
1. Infer a node's max fidelity from its transport dependencies
2. Generate test variants at each tier automatically
3. Gate test execution by `GUNBC_TEST_MAX_COST` (already exists)
4. Report coverage per tier ("100% at XS, 80% at S, 40% at M, 0% at L")

### Composition with Black-Box Model

The black-box mock corpus and the fidelity ladder compose orthogonally:

```
                    ┌──────────────────────────────────┐
                    │       Mock Corpus (per node)      │
                    │  workflow-observed + type-derived  │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────┴───────────────────┐
                    │     For each input set:           │
                    │                                   │
              ┌─────┴─────┐  ┌─────┴─────┐  ┌────┴────┐
              │   Tier XS  │  │  Tier S   │  │ Tier M  │  ...
              │ Pure mock  │  │ Virtual   │  │Sandboxed│
              │ (DryRun)   │  │ (hermetic)│  │(real IO)│
              └────────────┘  └───────────┘  └─────────┘
                    │               │              │
              ┌─────┴─────┐  ┌─────┴─────┐  ┌────┴────┐
              │ Assert     │  │ Assert +  │  │Assert + │
              │ output     │  │ side-     │  │ real    │
              │ shape      │  │ effects   │  │ verify  │
              └────────────┘  └───────────┘  └─────────┘
```

At XS, assertions are structural (output shape matches type). At S+, assertions
can verify side effects (file written, HTTP request well-formed). At L+,
assertions verify real-world outcomes (file exists on disk, API returned 200).

### Side-Effect Assertion Sources

At tiers S+, the test must assert something about side effects — but "expected
content" has to come from somewhere. The assertion source rule:

> At S+, side-effect assertions are derived from (in priority order):
> 1. Explicit `OutputMatcher` / `BoundaryMock` from MockSpec
> 2. Structural invariants (file exists, request schema valid, response
>    status 2xx, content non-empty)
> 3. Equality against a stable computed output port in the same test window
>    (e.g., the `content` input port value should match what was written)
>
> Never require stable golden outputs at S+ unless an explicit matcher exists.

This prevents the fidelity ladder from requiring golden snapshots everywhere
— structural assertions at S are sufficient for regression detection, and
explicit matchers handle the cases where exact content matters.

### Default Test Policy

By definition:
- **XS + S are hermetic** — no external dependencies, deterministic, cheap
- **M is sandboxed** — real I/O but contained (tempdir, nsjail, local stub servers)
- **L is local-live** — real I/O on local machine, may need tools installed
- **XL is remote-live** — real I/O to external services, needs secrets/credentials

`make test-all` runs `≤ S` because that's the hermetic ceiling. Anything above S requires explicit opt-in via `GUNBC_TEST_MAX_COST`.

The existing `max_cost_from_env()` in `core/test/src/fermi.rs` already defaults to `FermiCost::S` locally and `FermiCost::XL` in CI — this is exactly the right behavior. The missing piece is generating the per-tier test variants from the transport fidelity ladder.

---

## Implementation Plan

### Phase 1: Mock Corpus Builder

Build the cross-workflow accumulator. Piggyback on the existing baseline
DryRun that testgen already performs per workflow for window tests:

1. Compile all DSL tools (already happens in testgen)
2. For each compiled workflow, execute baseline DryRun
3. Operate on `lower(&dag).dag` (the flattened DAG), not the high-level DAG — this makes SubDag interior nodes (loop bodies, branch arms) visible without bespoke recursion
4. For each node in the lowered DAG, derive its inputs from upstream baseline outputs (same logic as `apply_window_inputs` for a 1-node window)
5. Build `CorpusExample` with `Provenance { workflow, profile, node_instance, subdag_path, seed_kind: WorkflowObserved }` and assign `Expectation` (ExactOutputs for pure+deterministic, TypeContractOnly otherwise)
6. Apply normalization (canonicalize maps, redact secrets, normalize paths)
7. Group by `NodeIdentity` (module + callable name), dedup by `(workflow, hash(inputs))`

**Output**: `HashMap<NodeIdentity, MockCorpus>`

**Files**:
- New: `core/codegen/src/testgen/mock_corpus.rs`
- Modify: `core/codegen/src/testgen/codegen.rs` (consume corpus in test generation)

### Phase 2: Per-Node Black-Box Test Generation

Extend testgen to emit Level 1a + 1b tests. **Start with pure nodes only**
(no transport mocking complexity):

1. **Level 1a** (all nodes): For each `ObservedCase`, execute node with case inputs, assert output ports exist and are type/shape-compatible. Use `ExecutionMode::Real` for pure nodes, `DryRun` for effectful nodes.

2. **Level 1b** (explicit matchers only): For each `NodeExample` with `OutputMatcher`, execute node and assert matcher. These are the only exact-output tests.

3. Pure nodes with no observed cases still get tested with type-derived inputs (Phase 4).

**Files**:
- Modify: `core/codegen/src/testgen/codegen.rs` (new test bucket)
- Modify: `core/test/src/mock_spec.rs` (corpus integration)

### Phase 3: Adjacent Pair Test Generation

Extend testgen to emit Level 2 tests. **Start with edges where both nodes are
pure** — no transport mocking in micro-tests:

1. Capture `EdgeExample` per workflow edge during DryRun (upstream inputs, downstream other-inputs, edge mapping)
2. For each pure→pure edge, generate a 2-node window test via `Window::from_nodes`
3. Execute the window through the **real executor wiring** (not manual port-map feeding) — this tests the same param→port translation path that workflows use
4. Assert downstream executes successfully, no unexpected Skipped, output validates

Extend to mixed edges (pure→effectful, effectful→pure) after pure-only path
is proven.

**Files**:
- Modify: `core/codegen/src/testgen/codegen.rs` (adjacent pair section)
- Modify: `core/codegen/src/testgen/mock_corpus.rs` (EdgeExample capture)
- Reuse: `core/test/src/window.rs` (Window::from_nodes for 2-node windows)

### Phase 4: Type-Derived Boundary Values (Mostly Exists)

The core infrastructure is already built. Wire it into the corpus:

1. For each node's input ports, resolve `TypeId` → `Dag<TypeOp>` via `TypeRegistry`
2. Call `witnesses(type_dag)` to get `Vec<BoundaryWitness>` (already handles cardinality, predicates, containers, brands)
3. Call `cross_product_witnesses(type_dag, depth)` for nested types
4. Call `variant_witnesses(type_id, registry)` for coproduct ports
5. Merge type-derived witnesses into the corpus alongside workflow-observed values
6. Cross-product across ports (Fermi-bounded pairwise to avoid combinatorial explosion)

**Already exists**:
- `contract::witnesses()` — full boundary witness generation from type DAGs
- `contract::cross_product_witnesses()` — nested type cross-products
- `contract::predicate_boundary_witnesses()` — lattice transition points
- `contract::variant_witnesses()` — coproduct arm coverage
- `cardinality::fermi_test_cases()` — bounded Fermi counts

**New work**:
- Bridge from port `TypeId` → `TypeRegistry` → `witnesses()` in corpus builder
- Anchored mutation: vary one port at a time from observed base cases
- Pairwise cross-product as opt-in per node (for known port-interaction bugs)
- Merge/dedup with workflow-observed values; `max_test_cases_per_node = 50`

**Files**:
- Modify: `core/codegen/src/testgen/mock_corpus.rs` (merge type-derived values from existing `contract::witnesses`)

### Phase 5: Cross-Workflow Consistency Tests

Generate Level 4 tests that verify nodes behave consistently across workflows:

1. For each node appearing in 2+ workflows, generate a consistency test
2. All workflow-specific inputs must produce structurally compatible outputs
3. Flag nodes where one workflow succeeds and another fails

**Files**:
- Modify: `core/codegen/src/testgen/codegen.rs` (cross-workflow section)

### Phase 6: Transport Fidelity Ladders

Define per-transport fidelity tiers and generate tiered test variants:

1. Add `FidelityLadder` type: `Vec<(FermiCost, FidelityLevel)>` per `TransportKind`
2. Define canonical ladders for File, Shell, Rest, Http, Tcp, LocalDirect
3. Extend DSL `resource` blocks with optional `fidelity { }` declarations
4. Compute per-node max fidelity from transport dependencies (transitive meet)
5. Generate test variants at each tier up to max fidelity
6. Gate by existing `GUNBC_TEST_MAX_COST` / `max_cost_from_env()`

**Already exists**:
- `TransportKind` enum (6 variants) in `core/ir/src/transport/behavior.rs`
- `FermiCost` enum with `PartialOrd` (natural tier ordering) in `core/test/src/fermi.rs`
- `max_cost_from_env()` gating with CI/local defaults
- `guard_test_with_env()` for cost + secrets + requirements

**New work**:
- `FidelityLadder` type and canonical definitions per transport
- `FidelityLevel` enum (PureMock, VirtualIo, Sandboxed, RealLocal, RealRemote)
- Node-level max fidelity inference from transport dependency graph
- Test variant generation: same inputs, different transport resolution per tier
- DSL syntax for `fidelity { }` block in resource definitions

**Files**:
- New: `core/test/src/fidelity.rs` (FidelityLadder, FidelityLevel, canonical ladders)
- Modify: `core/ir/src/transport/behavior.rs` (associate ladder with TransportKind)
- Modify: `core/codegen/src/testgen/codegen.rs` (tiered test generation)
- Modify: `dsl/std/resources.dag` (fidelity declarations on resources)

---

## Relationship to Existing Test Buckets

| Existing bucket | Black-box extension |
|----------------|-------------------|
| **A (Execution)** | Level 1a per-node tests extend single-workflow DryRun with cross-workflow inputs |
| **B (Contract)** | Level 2 adjacent pairs prove edge contracts at runtime |
| **C (Scenario)** | Level 4 cross-workflow catches wiring regressions |
| **D (Resource)** | Unchanged — resource hygiene is structural |
| **Probe-Observer** | Level 3 chain tests enrich with corpus mock pairs |
| **Windowed** | Complementary — windows prove decomposition correctness; Level 2 proves wiring |
| **Live flow** | Subsumed by fidelity ladder tiers L/XL with same corpus inputs |

---

## Worked Example: The FnBodyDelegate Regression

The bug: `FnBodyDelegate` was applied to all fn nodes in the DAG execution path. It evaluated fn bodies using parameter names (`sections`), but the DAG executor provided values keyed by port names (different convention). This caused `shared.dag_util::doc` to fail with "unbound variable: sections" in pragma/gist/bootstrap — but not makegen (which used a direct evaluation path).

**How black-box testing catches this**:

1. **Corpus accumulation**: The corpus builder compiles all 4 workflows (pragma, gist, bootstrap, makegen). Each DryRun captures I/O for the `doc` node.

2. **Per-node test**: For each workflow's input set, execute the `doc` node and assert output. The pragma input set would fail → test catches the regression.

3. **Cross-workflow consistency**: The consistency test notices makegen succeeds but pragma/gist/bootstrap fail → immediate signal that something is workflow-specific (which it shouldn't be for a pure fn).

4. **Adjacent pair test**: The `render_header → doc` edge test in pragma would fail because `render_header` produces output that `doc` can't consume (port naming mismatch) → catches the wiring bug.

## Worked Example: Type-Derived Boundary Values

Consider `fn apply_prefix(prefix: String?, cmd: String) -> String`:

**Type-derived input sets**:
```
{ prefix: Unit,     cmd: ""          }  // Option=None, String=empty
{ prefix: Unit,     cmd: "x"         }  // Option=None, String=minimal
{ prefix: Unit,     cmd: "cargo fmt" }  // Option=None, String=realistic
{ prefix: "RUSTFLAGS=\"-D warnings\"", cmd: "" }  // Option=Some, String=empty
{ prefix: "RUSTFLAGS=\"-D warnings\"", cmd: "cargo fmt" }  // Option=Some, realistic
```

**Workflow-observed input sets** (from makegen):
```
{ prefix: Unit,                          cmd: "@cargo fmt" }
{ prefix: "RUSTFLAGS=\"-D warnings\"",   cmd: "cargo run -p gunbc-app ..." }
{ prefix: "GUNBC_TEST_MAX_COST=XL",      cmd: "@RUSTFLAGS=\"-D warnings\" cargo test ..." }
```

**Union** gives 8 distinct input sets. Each generates a test. The parser block-expression bug (empty record for multi-statement match arms) would have been caught by the third workflow-observed input — `prefix: "GUNBC_TEST_MAX_COST=XL"` triggers the wildcard match arm that was silently broken.

---

## Cost Model

### Per-Fidelity Tier Breakdown

| Level | Tier XS (mock) | Tier S (virtual) | Tier M (sandboxed) | Tier L (real local) |
|-------|---------------|-----------------|-------------------|-------------------|
| 1 (per-node) | ~10 × N_nodes | ~10 × N_transport_nodes | ~10 × N_transport_nodes | ~10 × N_transport_nodes |
| 2 (adjacent) | ~3 × N_edges | ~3 × N_transport_edges | ~3 × N_transport_edges | — |
| 3 (chain) | ~50 tests | ~50 tests | — | — |
| 4 (cross-workflow) | ~50 tests | — | — | — |

### Default Run (`make test-all`, ≤ S)

For the current codebase (~200 unique nodes, ~400 edges, ~50 shared, ~80 transport nodes):
- Level 1 XS: ~2000 tests (<1s total)
- Level 1 S: ~800 tests (<3s total, virtual I/O)
- Level 2 XS: ~1200 tests (<5s total)
- Level 2 S: ~300 tests (<2s total, virtual I/O)
- Level 3: ~50 tests (<1s total)
- Level 4: ~50 tests (<1s total)
- **Total ≤ S: ~4400 tests, <13s**

All hermetic, all deterministic, all auto-generated.

### Opt-In Tiers

```
GUNBC_TEST_MAX_COST=M  → adds ~1100 sandboxed tests (~30s)
GUNBC_TEST_MAX_COST=L  → adds ~800 real-local tests (~2min, needs tools)
GUNBC_TEST_MAX_COST=XL → adds ~200 remote tests (~5min, needs secrets)
```

The key property: **test count scales linearly with tier**, not exponentially. Each tier adds transport nodes × corpus size tests, not a combinatorial explosion. Higher tiers reuse the same corpus inputs — only the transport resolution changes.

---

## Design Decisions (Resolved)

1. **Corpus storage**: Recompute each testgen run. DryRun is fast, avoids
   staleness. Optional: cache behind an env flag keyed by a compile hash
   (perf optimization only, never for correctness).

2. **Test naming**: `test_node_{id}_workflow_{workflow}_input_{hash}` — readable
   and stable across minor workflow changes. Consider table-driven per-node
   tests (one `#[test]` fn iterating over examples) to reduce Rust compile
   overhead vs thousands of `#[test]` fns.

3. **Failure diagnostics**: Port-level diffs on failure. Include provenance in
   every panic message: workflow, node instance path (incl. subdag), input
   hash, which port failed validation, actual vs expected type.

4. **Partial corpus**: Nodes in only 1 workflow still get Level 1a tests
   (TypeContractOnly) plus type-derived anchored mutations. Level 4
   (cross-workflow) is skipped. If a required port is a semantic carrier with
   no seed: **skip with a diagnostic** ("needs explicit MockSpec seed"),
   don't silently degrade.

5. **SubDag visibility**: Corpus builder operates on `lower(&dag).dag` — the
   flattened DAG where loop bodies and branch arms are visible as ordinary
   nodes. Instrument execution so capture includes nested nodes with a
   `subdag_path` in provenance. Reuses the same lowering path that window
   tests already use.

6. **Multi-port generation**: Default is **anchored mutation** (vary one port
   at a time from observed base cases). Pairwise cross-product is opt-in per
   node. `max_test_cases_per_node = 50` as a safety valve.

7. **Type DAG registry**: Ensure lowered IR carries (or can reconstruct) the
   full type DAG/registry including DSL-defined record/sum types. If type info
   is missing for a port, don't silently degrade — emit a diagnostic and rely
   on workflow-observed inputs only for that port.

8. **Profile-aware provenance**: `profile: Option<String>` in `Provenance`.
   When profiles change interface bindings (stub vs real), corpus cases from
   different profiles may produce different values for the same node. Never
   mix stub-profile and real-profile examples in a single consistency test.

---

## Incremental Landing Strategy

To avoid breaking existing testgen, land incrementally:

1. **Corpus from existing baselines**: During the per-workflow baseline DryRun that testgen already performs for window tests, accumulate per-node `ObservedCase` entries. No new test generation yet — just build the data structure.

2. **Level 1a for pure nodes only**: Generate `execute_single_node` + shape assertions for pure nodes using `ExecutionMode::Real`. This directly addresses the "Tier 1 infra" gap that generated tests already allude to in comments.

3. **Level 2 for pure→pure edges only**: Two-node chain tests without transport mocking complexity.

4. **Type-derived boundary cases**: Wire `contract::witnesses()` into corpus. Ensure all generated values are guard-satisfying (use same guard-check pipeline that testgen already uses).

5. **Level 4 cross-workflow for deterministic nodes only**: Gate on `is_pure && no_resource_deps && no_env_reads`. Nodes with implicit context dependencies will produce "expected differences" that are useful signal but disruptive to gate on until the dependency model is complete.

---

## Diagnostic Dashboard

Once the corpus exists, surface these metrics as engineering signal:

| Metric | What it reveals |
|--------|----------------|
| Top 20 nodes by cross-workflow input diversity | Most-reused nodes (highest value for corpus testing) |
| Nodes with inconsistent output shape across workflows | Implicit context dependencies not modeled as ports |
| Nodes with highest failure rate in Level 1a | Broken evaluation / wiring bugs |
| Nodes with no corpus cases (only type-derived inputs) | Nodes not exercised by any workflow (dead code candidates) |
| Pure nodes with no adjacent-pair coverage | Missing edge tests (coverage gap) |
