# Testgen: Test Generation from Proof Obligations

> **Goal**: Generate *all non-tautological tests* deducible from graph structure
> and the contract tower, with minimal user input.

---

## Core Principle: Tests = Unproven Proof Obligations

The framework has a strong static story: nodes are pure, boundaries are
structural, and conditional execution is structural (Branch + optional
cardinality), not ad-hoc metadata/guards.

The validation model has four levels:

| Level | What it proves | Static? |
|-------|---------------|---------|
| **L1** | Cardinality satisfaction | Yes (edge creation) |
| **L2** | Type equality | Yes (validate_dag) |
| **L3** | Predicate entailment | Sometimes (Unknown → test) |
| **L4** | Witness compatibility | Not yet (future infra) |

The anti-tautology rule:

> Only generate tests for obligations that are **not fully discharged** by
> compile-time validation, plus runtime/executor invariants that cannot be
> statically guaranteed.

---

## Proven by Construction (NO Tests Generated)

These are statically verified. Generating tests for them would be tautological:

| Property | How it's proven |
|----------|----------------|
| **Acyclicity** | DAG structure is acyclic by construction |
| **Edge type compatibility** | `validate_dag()` / `DagBuilder::add_edge` |
| **Edge cardinality compatibility** | Compile-time checked at edge creation |
| **Cycle detection** | `DagBuilder` enforces |

---

## Obligation Buckets

The testgen produces tests organized into 4 buckets. Each obligation is either
discharged statically (no test) or remains Unknown/RuntimeOnly (test generated).

### Bucket A — Execution Semantics (framework-level)

Not tautological because they validate the **executor / boundary model** against
the graph. Static typing cannot prove "DryRun truly intercepts all transports."

| Obligation | Test | Tier |
|-----------|------|------|
| **DryRun completes** | Run full workflow in DryRun, verify no crash | 0 |
| **Transport interception** | All transport executors were intercepted | 0 |
| **Determinism** (same inputs → same outputs) | `execute_single_node` twice, compare | 1 |

Note: "Determinism" was previously called "Idempotency" — that's a misnomer.
Idempotency is `f(f(x)) == f(x)`. What we test is determinism / referential
transparency: `f(x) == f(x)`.

### Bucket B — Contract Obligations (graph-specific, high value)

Tests only generated when the type/contract system can't fully prove something.

| Obligation | Trigger | Tier |
|-----------|---------|------|
| **Edge predicate entailment** | L3 returns Unknown | 3 |
| **Witness compatibility** | L4 witnesses exist | 3 |
| **Node contract compliance** | Always (implementation can be wrong) | 1 |

This is *the* "not tautological" class: it checks semantic compatibility when
the proof engine can't decide. Gets much stronger once "Types as DAGs" enables
witness generation.

### Bucket C — Scenario Coverage (N+1, not 2^N)

Mechanically derived from graph structure.

| Obligation | Count | Tier |
|-----------|-------|------|
| **All transports succeed** | 1 | 0 |
| **Single transport failure** | N (per transport executor) | 0 |
| **Skip-path propagation** | Per transport with downstream nodes | 0 |
| **Guard/skip branch coverage** | Per guarded input port | 0 |

The guard/skip branch coverage is a high-value addition: for each node with a
guarded input, generate two scenarios (guard passes → executes, guard fails →
skip). The executor implements `skip ⇒ all outputs are Value::Skipped`, and
importantly, transport nodes should NOT be "intercepted" when they weren't
executed.

### Bucket D — Resource Hygiene (structural + simulation)

Split into two categories:

**D.1: Structural (graph-declared, capability-grant model)**

| Obligation | What it proves | Tier |
|-----------|---------------|------|
| **Resource inputs connected** | Every `resource:*`/`tool:*` input has an edge | 0 |
| **Resource owner valid** | Provider is a valid env/acquisition node | 0 |
| **No orphan resources** | Acquired resources are consumed by someone | 0 |
| **Resource conflict absence** | `detect_conflicts(dag, accesses)` returns empty | 0 |
| **Contention handling** | Consumer handles failed acquisition | 0 |

**D.2: MockSpec-based simulation (existing infrastructure)**

| Obligation | What it proves | Tier |
|-----------|---------------|------|
| **Resource acquisition** | `acquire()` succeeds or fails per behavior | 0 |
| **Lease timeout** | `should_timeout(elapsed)` behaves correctly | 0 |

Note: The old tests "resources actually used" and "skipped nodes don't acquire"
required runtime tracking. With the capability-grant model, resource needs are
**structural** — declared via ports and edges. For tool acquisition ordering,
see "No speculative tool acquisition" below.

---

## Reality Tiers

What can be generated today vs what needs infrastructure.

### Tier 0 — Today, No New Infrastructure

| Test | What it does |
|------|-------------|
| DryRun completes | Smoke test: workflow runs without crash |
| Boundary interception | All transport executors mocked in DryRun |
| Success + per-boundary failure | N+1 scenarios |
| Guard toggle scenarios | Each guarded input: pass/fail |
| Resource connectivity | All resource inputs have edges |
| Resource owner validity | Providers are env/owner nodes |
| Resource simulation | MockSpec-based acquire/timeout tests |
| Resource conflict absence | `detect_conflicts` returns empty |

### Tier 1 — Small Infrastructure (mostly present)

| Test | Requires |
|------|----------|
| Per-node determinism | `execute_single_node` + baseline-derived inputs |
| Node contract compliance | `execute_single_node` + valid input generation |
| Output conforms to ports | Runtime output validation in executor |

Key insight: `execute_single_node` **already exists** in the codebase. Per-node
testing doesn't require inventing a new execution model.

Low-effort trick for determinism:
1. Run a baseline DryRun once
2. Reconstruct each node's input map from (DAG edges + upstream outputs)
3. Call `execute_single_node` multiple times, assert stable outputs

### Tier 2 — IR/Runtime Changes

| Test | Requires |
|------|----------|
| No speculative tool acquisition | Move tool acquisition after skip checks |
| Tool hygiene generalized to resources | Resource event logging |
| Empty/large collection handling | Per-node isolation + mock generation |
| Optional input presence | Per-node isolation |

### Tier 3 — Types as DAGs / Contract Witnesses

| Test | Requires |
|------|----------|
| Edge predicate entailment (L3) | Witness generation from contract predicates |
| Witness compatibility (L4) | Type registry with default witnesses |
| Boundary value fuzzing | Per-type boundary strategy registry |

---

## Unified Resource Model

All resources follow the **capability grant pattern**: acquired by an owner node
and flow downstream via explicit edges.

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

### No Speculative Tool Acquisition

The executor currently acquires tools BEFORE checking skip guards on downstream
consumer nodes. This means a skipped node can still trigger tool acquisition
upstream — exactly the "no speculative acquisition" property we want to enforce.

**Recommended fix**: Move tool acquisition after skip checks (or make it
conditional on non-skip). Then generate a test:

- Scenario: upstream forces `consumer_node.skip=true`
- Assert: no tool acquisition occurred (or at least no "check/install" ops ran)

This is the best stepping stone to broader "resource hygiene" tests.

---

## Obligation IR

The testgen is built around a small internal IR in `obligation.rs`:

```rust
pub enum Obligation {
    // Bucket A
    TransportInterceptable { node_id },
    DryRunCompletion,
    PureNodeDeterminism { node_id },

    // Bucket B
    EdgePredicateEntailment { from_node, to_node, entailment: EntailmentStatus },
    WitnessCompatibility { node_id, port_name, type_id },
    NodeContractCompliance { node_id },

    // Bucket C
    AllTransportsSucceed,
    SingleTransportFailure { node_id },
    SkipPathPropagation { trigger_node },
    GuardBranchCoverage { node_id, guard_port },

    // Bucket D
    ResourceInputConnected { node_id, port_name },
    ResourceOwnerValid { node_id },
    ResourceOrphan { node_id, port_name },
    ResourceConflictAbsence { conflicts },
    ResourceContentionHandling { resource_port },
    ResourceSimulation { resource_id },
}
```

The generator:
1. Runs the validator/analyzer
2. Collects only obligations that are **Unknown / RuntimeOnly**
3. Produces tests

This gives "ALL deducible non-tautological tests" by construction.

---

## What's Structural vs What Needs Annotation

### Structural (derivable from graph)

| Property | How we know |
|----------|-------------|
| Transport executors | Input port has type `TransportRequest` |
| Tool consumers | Input port has type `ToolHandle` |
| Tool environment | Output port has type `ToolHandle` |
| Resource consumers | Input port name starts with `resource:` or `tool:` |
| Guard branches | Input port has a guard predicate |
| Downstream nodes | Follow edges from a given node |
| Pure vs I/O nodes | No `TransportRequest` / `ToolHandle` = pure |

### Minimal User Input

| Input | Purpose | Already exists? |
|-------|---------|----------------|
| Type registry with default witnesses | Boundary fuzzing, L3/L4 tests | Partially (TypeRegistry) |
| `Mockable::mock_outputs()` | Fallback witness values | Yes |
| `Mockable::cardinality_inputs()` | Optional/empty cases | Yes |
| `Mockable::error_cases()` | Known failure shapes | Yes |
| `MockSpec` | Per-tool boundary/transport mocks | Yes |

Everything else — success/failure semantics, skip flows, resource hygiene —
is derived from structure + contracts, not from port-name conventions.

---

## Test Count: Honest Estimates

For a typical CI graph with T=5 transports, G=3 guarded nodes, R=3 resources:

| Tier | Tests | What |
|------|-------|------|
| 0 | ~15-20 | DryRun, interception, N+1 failures, guard toggles, resource wiring |
| 1 | ~10-15 | Per-node determinism, contract compliance |
| 2 | ~5-10 | Tool hygiene, collection handling |
| 3 | ~30+ | Contract witnesses, boundary fuzzing |

Don't claim 120 tests from a graph that simply doesn't have Many/coercions/
resources declared. The honest near-term number is ~15-20 tests per DAG with
zero manual MockSpec.

---

## Acceptance Criteria

Formal criteria for evaluating testgen completeness. Each criterion has a
status and evidence pointer.

### Proven (met today)

| # | Criterion | Evidence |
|---|-----------|----------|
| AC-1 | **Auto-discovery covers all compilable modules** — every `.dag` file with `func` items gets testgen treatment with zero manual input | 29 generated files from `discover_compilable_modules()`. Only 1 module skipped (`examples/deployment.dag` — ambiguous resource binding). |
| AC-2 | **Obligation bucketing is complete** — all 4 buckets (A-D) produce tests | Generated file headers show `A=N, B=N, C=N, D=N` obligation counts per module. Example: `tools/testgen.dag` → `388 obligations (100 discharged, 5 INVALID, 283 testable: A=83, B=172, C=28, D=0)`. |
| AC-3 | **Anti-tautology filtering** — only Unknown/RuntimeOnly obligations generate tests; statically-proven properties (L1 cardinality, L2 type equality) are discharged | `collect_obligations()` returns discharged count ≥ 25% of total obligations across all modules. |
| AC-4 | **Scale** — testgen produces meaningful test volume from structure alone | 5,874 test functions across 157K lines of generated code, from 29 modules, with zero manual `MockSpec`. |
| AC-5 | **DryRun smoke coverage** — every generated module has a DryRun completion test | Bucket A `DryRunCompletion` obligation emitted for every module with transport nodes. |
| AC-6 | **Transport failure scenarios** — N+1 scenarios (all-succeed + per-boundary failure) | Bucket C `SingleTransportFailure` obligations per transport node, plus `AllTransportsSucceed`. |
| AC-7 | **Skip-path propagation** — inject `Value::Skipped`, verify downstream nodes skip | Bucket C `SkipPathPropagation` tests generated for each transport with downstream dependents. |
| AC-8 | **Guard branch coverage** — two-scenario tests for Bool guards | Bucket C `GuardBranchCoverage` per guarded input port. |
| AC-9 | **Probe-observer integration** — structural integration chains from probes to terminal observers | Bucket B generates probe→observer chain tests. Example: `tools/testgen.dag` → 72 probes, 12 observers, 95 integration tests. |
| AC-10 | **WrapScalar coercion coverage** — `__deps` fan-in coercion edges tested | Bucket B generates `coercion_wraps_*` tests for each `__deps` port with WrapScalar edges. |
| AC-11 | **MockCorpus infrastructure** — DryRun-derived per-node I/O corpus builder works | `build_corpus()` extracts `NodeIdentity`→`CorpusExample` from `ExecutionLog`. Proven in `corpus_builder.rs` integration tests. |
| AC-12 | **Fidelity ladder types** — cost-tiered transport resolution model complete | `FidelityLevel`, `FidelityLadder`, `canonical_ladders()`, `node_max_fidelity()` all pass 310 lines of tests. |
| AC-13 | **Type witness enrichment** — corpus mutation via `contract::witnesses_checked()` | `enrich_corpus_with_type_witnesses()` implemented and tested. Max 50 examples per node, 3 base cases varied per port. |

### Partial (infrastructure exists, not fully wired)

| # | Criterion | Gap | Reconciliation |
|---|-----------|-----|---------------|
| AC-14 | **Per-node corpus execution** — execute each node against corpus examples, assert outputs | `build_corpus_section()` is a stub (asserts `!dag.nodes.is_empty()` only). `corpus_tests: false` in `TestConfig`. | Wire `execute_single_node` + corpus examples → assert. No design decision needed — straightforward implementation (BB-2). |
| AC-15 | **Adjacent-pair window tests** — 2-node windows through real executor | `build_adjacent_pair_section()` is a stub. `adjacent_pair_tests: false`. | Wire `window_subdag` extraction + executor. Blocked on `__deps` MixedInput resolution for windows crossing fan-in nodes (BB-3). |
| AC-16 | **Fidelity ladder variant generation** — same corpus, different transport resolution per tier | `build_fidelity_ladder_section()` emits TODO placeholders. Only PureMock (XS) is real. | S+ tiers blocked on virtual I/O infrastructure (in-memory FS, mock HTTP server). Not a testgen design issue — infrastructure dependency (BB-6). |

### Not Started

| # | Criterion | Dependency |
|---|-----------|-----------|
| AC-17 | **Cross-workflow consistency** — nodes appearing in 2+ workflows assert compatible outputs | BB-5. Needs corpus data from multiple workflows + output comparison logic. |
| AC-18 | **L3/L4 contract witnesses** — boundary fuzzing from type-derived witnesses | Tier 3. Needs type registry with default witnesses + predicate generation. |
| AC-19 | **Property-based simulation** — IoContract generators + validators exercised per-node | `Simulator` types exist but not integrated into testgen codegen pipeline. |

---

## Gap Reconciliation

### Self-reconcilable (no design decisions needed)

| Gap | What to do | Effort |
|-----|-----------|--------|
| **BB-2 (corpus execution)** | Set `corpus_tests: true`, implement `build_corpus_section()` body: iterate corpus examples, call `execute_single_node`, assert `Expectation` (ExactOutputs/TypeContractOnly). | M |
| **BB-4 (enrichment wiring)** | Call `enrich_corpus_with_type_witnesses()` in the `AutoGenerate` path after `build_corpus()`, passing `result.dsl_type_registry`. | S |
| **BB-5 (cross-workflow)** | Set `cross_workflow_tests: true`, implement body: group corpus by `NodeIdentity`, for nodes in 2+ workflows assert output types/shapes match. | S |
| **Observability gaps** | Extend `auto_mock_spec()` fallback matchers to cover more terminal node types beyond `IdentityCallableOp`. | S |
| **`examples/deployment.dag` skip** | Provide a default profile or relax `enforce_profile_for_bound_uses()` for auto-testgen compilation (IS-3 follow-up). | S |

### Requires infrastructure work (no design decisions, but non-trivial)

| Gap | What's needed | Effort |
|-----|--------------|--------|
| **BB-3 (`__deps` MixedInput)** | Teach `window_subdag` to handle fan-in ports: either (a) include all fan-in sources in the window, or (b) mock the external fan-in edges. Option (b) is more general — mock external inputs at the window boundary. | M |
| **BB-6 S+ fidelity tiers** | Build virtual I/O backends: in-memory filesystem (S-tier File/Shell), in-memory HTTP mock server (S-tier Rest/Http). Then wire into fidelity ladder codegen. | L |
| **ConditionalMock** | Add predicate-based response selection to `MockSequence`. Low priority — `MockSequence` (ordered) covers current needs. | S |
| **Simulator integration** | Wire `IoContract` generators into testgen codegen: emit property-based test loops per node with contracts. | M |

### Needs design decision

None of the remaining gaps require a design decision from you. All gaps are
implementation work following established patterns. The testgen model (obligation
buckets, anti-tautology, auto-discovery, fidelity ladders) is architecturally
complete — what remains is filling in stub bodies and wiring infrastructure.

---

## Implementation Status

### Done

- [x] `DagAnalysis` — structural analysis (transport detection, tool env, guards, pure nodes)
- [x] `ObligationSet` — proof obligation collection and bucketing
- [x] `collect_obligations()` — analyze DAG → produce obligations
- [x] `check_predicate_entailment()` — L3 entailment checking (conservative)
- [x] Bucket A codegen: DryRun completion, transport interception
- [x] Bucket B codegen: probe-observer integration chains, WrapScalar coercion coverage
- [x] Bucket C codegen: all-succeed, single-failure scenarios
- [x] Bucket C codegen: skip-path propagation (inject `Value::Skipped`, verify downstream)
- [x] Bucket C codegen: guard branch coverage (Bool guards: two-scenario test, non-Bool: structured comments)
- [x] Bucket D codegen: resource connectivity, conflict absence, simulation
- [x] Obligation summary in generated test header
- [x] Anti-tautology filtering (only Unknown/RuntimeOnly → tests)
- [x] Guard obligations decoupled from transport presence (C.4 always emitted)
- [x] `build::guarded()` helper for creating guarded ports in tests
- [x] Auto-discovery pipeline (`discover_compilable_modules` → compile → `auto_mock_spec` → generate)
- [x] MockCorpus builder (`build_corpus` from DryRun `ExecutionLog`)
- [x] Type witness enrichment (`enrich_corpus_with_type_witnesses`)
- [x] Fidelity ladder types and canonical ladders
- [x] `NodeExample` / `OutputMatcher` per-node I/O specification
- [x] `Simulator` / `IoContract` types for property-based testing

### Next Steps

- [ ] BB-2: Per-node corpus execution via `execute_single_node` with corpus-derived inputs
- [ ] BB-3: Adjacent-pair window tests (requires `__deps` MixedInput resolution)
- [ ] BB-4: Wire `enrich_corpus_with_type_witnesses()` into auto-testgen pipeline
- [ ] BB-5: Cross-workflow consistency tests (group by `NodeIdentity`, compare outputs)
- [ ] BB-6: S+ fidelity tier variant generation (blocked on virtual I/O infrastructure)
- [ ] Guard branch tests for non-Bool types (requires per-node isolation / Tier 1)
- [ ] Tool/resource acquisition instrumentation + ordering fix (skip → no tool acquire)
- [ ] Contract-tower witnesses for true boundary fuzzing (L3/L4)
- [ ] Per-type boundary strategy registry for edge case generation
- [ ] Simulator/IoContract integration into testgen codegen pipeline

### Known Issues / Follow-ups

- [ ] `types_compatible()` in `gunbc-test/composition.rs` treats `Any` as universally
      compatible at L2 (line 38). This is correct for wiring validation, but means
      L3 entailment checking is the *only* thing preventing `Any → ConstrainedType`
      edges from silently passing. If L3 is ever bypassed or disabled, these edges
      would go untested. Consider: should `types_compatible` return a
      `CompatibilityResult` with a warning for `Any` source?
- [ ] Hard-stop mode for Invalid obligations: optionally refuse to generate the
      normal test suite when `has_invalids()` is true (emit `compile_error!` or
      a single failing test that lists all invalids). Currently, invalid tests
      appear alongside normal tests.
- [ ] SubDag interface validation as an IR-level validator (`validate_subdag_interfaces`)
      that testgen can call. Currently only checked during `lower()` (too late).
      See DAG Pattern Audit findings 1, 2, 6.
- [ ] Pattern config observability: Repeat/While/Poll config fields (retry policy,
      classifier, max_iterations, interval/timeout) are stored but never lowered into
      IR structure. Testgen can't verify them until config is IR-observable.
- [ ] `T::default()` contract gap: pattern internals depend on `T::default()` for
      merge/unpack/controller ops with no trait-level contract. Needs `T: PatternInternalOps`
      or equivalent — not a testgen problem but affects test correctness.
- [ ] `__deps` MixedInput: windows spanning fan-in ports (WrapScalar coercion) are
      silently skipped by `window_has_mixed_inputs()`. Individual coercion edges are
      tested (Bucket B), but integration chains through those nodes are not.

---

## Recent Additions (2026-02)

### Auto-Generation from Types + DAG Structure (Primary Model)

As of 2026-02, testgen uses **auto-discovery** as its primary model. Every
compilable `.dag` file (any file with `func` items) gets full testgen treatment
with zero manual input:

```
discover_compilable_modules(dsl/) → compile → auto_mock_spec() → generate_target()
```

- `auto_mock_spec(dag, name)` generates a complete `MockSpec` from DAG
  structure + types alone — no inline `test` blocks, no `#[testgen_target]`
  annotations, no manual fixtures.
- `generate_target(config, dag, spec)` emits Rust test code from the
  auto-generated MockSpec.
- `build_testgen_graph_auto()` is the registered testgen builder. It discovers
  all compilable modules and creates content upsert chains for each.

**Inline test blocks** (in `.dag` files) are optional overrides — they layer
fixture-specific mocks and assertions on top of auto-generated defaults.
`#[testgen_target]` inventory registrations are also optional; auto-discovery
supersedes them for all compilable modules.

**Coverage**: 29 generated test files, 5,874 test functions, 157K lines of
generated code. Only 1 module skipped (`examples/deployment.dag` — ambiguous
resource binding requiring profile resolution). Coverage is automatic — as DSL
gaps close, skipped modules will automatically get tests too.

### MockSpec Requirement (Safety Net)

DAGs with transport nodes still **require** a MockSpec when using `TestGenerator`
directly. The auto-discovery path always provides one via `auto_mock_spec()`, so
this panic is unreachable through the standard pipeline:

```rust
// In TestGenerator::generate_test_module()
if !analysis.transport_executors.is_empty() && self.mock_spec.is_none() {
    panic!("DAG '{}' has transport nodes but no MockSpec provided", module_name);
}
```

### NodeExample / OutputMatcher

Per-node I/O specification for generating example-based tests:

```rust
pub struct NodeExample {
    pub node_id: String,
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, OutputMatcher>,
    pub description: Option<String>,
}

pub enum OutputMatcher {
    Exact(Value),           // Exact equality
    Contains(String),       // String contains substring
    NonEmpty,               // Non-empty string/list
    Satisfies { predicate } // Custom predicate
    Any,                    // Any value OK
}
```

Usage in MockSpec:

```rust
MockSpec::new("llm")
    .node_example(
        NodeExample::new("prepare")
            .input("question", Value::Str("What is 2+2?".into()))
            .output("system_prompt", OutputMatcher::non_empty())
            .output("messages", OutputMatcher::contains("2+2"))
            .description("basic arithmetic question")
    )
```

### Testgen Registry (Legacy, Superseded by Auto-Discovery)

> **Note**: The inventory-based registry is superseded by auto-discovery
> (`build_testgen_graph_auto`). New modules do NOT need `#[testgen_target]`
> registrations — auto-discovery generates tests for all compilable `.dag` files.

DAGs can optionally register via `#[testgen_target(...)]` for custom generate
functions or override behavior:

```rust
#[gunbc_testgen_registry_macros::testgen_target(
    name = "ci",
    output = "gunbc-dag/src/ci/generated_tests.rs",
    module = "ci_generated_tests",
    builder(crate::build_ci_graph().unwrap()),
    signature(crate::ci_signature()),
    flow_tests
)]
pub fn ci_mock_spec() -> MockSpec { ... }
```

### Makefile Integration

```makefile
testgen:
    cargo run -p gunbc-dag --bin gunbc-testgen --release

testgen-check:
    cargo run -p gunbc-dag --bin gunbc-testgen --release -- --mode=verify
```

Staleness detection via content hash in generated file header:

```rust
// Generated tests for ci_generated_tests DAG.
// DO NOT EDIT - regenerate with: make testgen
// Content hash: 0x1a2b3c4d...
```

### Flow Verification Tests

When `flow_tests: true`, generates end-to-end tests using MockSpec terminal outputs:

```rust
#[test]
fn test_flow_ci() {
    let dag = crate::build_ci_graph().unwrap();
    let mocks = mock_spec().to_boundary_mocks();
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))
        .expect("DryRun execution should succeed");

    // Verify terminal outputs from MockSpec
    let entry = log.get("report").expect("node 'report' should be in log");
    assert_eq!(
        entry.outputs.get("overall_success"),
        &Value::Bool(true),
        "flow verification: report.overall_success mismatch"
    );
}
```

---

## Simulator: Property-Based I/O Testing

The `Simulator` type enables property-based testing with constrained random inputs
and output range validation.

### Core Types

```rust
pub struct Simulator {
    pub description: String,
    generator: Option<Arc<dyn Fn() -> Value>>,   // Generate random valid values
    validator: Option<Arc<dyn Fn(&Value) -> Result<(), String>>>, // Check value in range
}

pub struct IoContract {
    pub name: String,
    pub input: HashMap<String, Simulator>,   // Input generators
    pub output: HashMap<String, Simulator>,  // Output validators
}
```

### Built-in Simulators

| Simulator | Description |
|-----------|-------------|
| `non_empty_string()` | Random non-empty string |
| `boolean()` | Random bool |
| `exit_code()` | Random 0-255 |
| `success_exit_code()` | Always 0 |
| `failure_exit_code()` | Random 1-255 |
| `int_range(min, max)` | Random int in range |
| `json_object()` | Random JSON object |
| `one_of(values)` | Random from allowed set |
| `any()` | Any value (no constraint) |

### Usage Pattern

```rust
// Define I/O contract for a node
let contract = IoContract::new("parse_exit_code")
    .input("exit_code", Simulator::exit_code())     // Generate 0-255
    .output("success", Simulator::boolean());        // Output must be bool

// Property test: for all valid inputs, outputs satisfy contract
for _ in 0..100 {
    let inputs = contract.generate_inputs();
    let outputs = execute_single_node("parse_exit_code", inputs);
    assert!(contract.validate_outputs(&outputs).is_ok());
}
```

### Integration with MockSpec (Planned)

```rust
MockSpec::new("ci")
    .node_contract("parse_build",
        IoContract::new("parse_build")
            .input("response", Simulator::shell_response())
            .output("success", Simulator::boolean())
            .output("exit_code", Simulator::exit_code())
    )
```

### Next Steps for Simulators

- [ ] Add `shell_response()` and `transport_response()` simulators
- [ ] Add `node_contract()` method to MockSpec
- [ ] Generate property tests from IoContracts in testgen
- [ ] Support "paired" simulators (output depends on input characteristics)
- [ ] Shrinking for counterexample minimization
