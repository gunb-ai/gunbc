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

## Implementation Status

### Done

- [x] `DagAnalysis` — structural analysis (transport detection, tool env, guards, pure nodes)
- [x] `ObligationSet` — proof obligation collection and bucketing
- [x] `collect_obligations()` — analyze DAG → produce obligations
- [x] `check_predicate_entailment()` — L3 entailment checking (conservative)
- [x] Bucket A codegen: DryRun completion, transport interception
- [x] Bucket C codegen: all-succeed, single-failure scenarios
- [x] Bucket C codegen: skip-path propagation (inject `Value::Skipped`, verify downstream)
- [x] Bucket C codegen: guard branch coverage (Bool guards: two-scenario test, non-Bool: structured comments)
- [x] Bucket D codegen: resource connectivity, conflict absence, simulation
- [x] Obligation summary in generated test header
- [x] Anti-tautology filtering (only Unknown/RuntimeOnly → tests)
- [x] Guard obligations decoupled from transport presence (C.4 always emitted)
- [x] `build::guarded()` helper for creating guarded ports in tests

### Next Steps

- [ ] Per-node execution harness using `execute_single_node` with baseline-derived inputs
- [ ] Guard branch tests for non-Bool types (requires per-node isolation / Tier 1)
- [ ] Tool/resource acquisition instrumentation + ordering fix (skip → no tool acquire)
- [ ] Contract-tower witnesses for true boundary fuzzing (L3/L4)
- [ ] Per-type boundary strategy registry for edge case generation

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
