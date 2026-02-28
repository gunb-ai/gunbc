# Lowerer Pure-Function Refactor

**Status**: Proposed
**Date**: 2026-02-28
**Scope**: `core/daglang/daglang-lower/src/lib.rs` (8,300 lines) + related modules

## 1. Problem Statement

The lowerer (`lib.rs`) is an 8,300-line imperative mutation pipeline that threads
a mutable `DagBuilder` through 33 functions. Every phase mutates the builder
in-place, making it impossible to test phases independently, reason about data flow,
or compose phases differently for different compilation targets.

### Evidence of the problem

**Defect patterns traceable to imperative mutation:**
- **30 silent drops** (`_ => None`) — return expressions, wiring paths, transport specs
  all silently discard data when they can't handle an expression type. The `overall_success`
  postmortem (RT4a) was caused by exactly this: the lowerer dropped a complex return
  expression, nobody noticed, execution produced `Value::Skipped`.
- **105 conditional skips** vs 74 edge additions — the lowerer skips more edges than it
  creates, and many skips mask wiring failures.
- **22+ functions with >6 parameters** — the builder + N registry HashMaps get threaded
  through every call. Parameter lists of 8-11 are common.
- **1 side effect** (`eprintln!`) in the lowerer itself, plus the RT4c warning was
  emitted via `eprintln!` until this branch removed it.

**Structural smell: the 9-phase pipeline is sequential mutation, not composition.**

```
lower_typed_project_with_callable_scope:
  Phase 1: lower callables → builder.add_node (mutation)
  Phase 2: add scaffolding → builder.add_node (mutation)
  Phase 3: add transport triplets → builder.add_node + builder.add_edge (mutation)
  Phase 4: wire dependencies → builder.add_edge (mutation)
  Phase 5: wire service calls → builder.add_edge (mutation)
  Phase 6: wire auth → builder.add_edge (mutation)
  Phase 7: resource lifecycle → builder.add_node + builder.add_edge (mutation)
  Phase 8: wire resources → builder.add_edge (mutation)
  Phase 9: interface contracts → builder.add_node (mutation)
  Final: builder.into_dag() → stamp_node_kinds(&mut dag)
```

Each phase mutates `DagBuilder` which contains a `Dag<LoweredOp>` (nodes + edges)
plus deduplication sets. Because all phases share the same mutable state, you cannot:
- Test Phase 5 without running Phases 1-4 first
- Swap Phase 3 for a different transport strategy
- Emit intermediate results for debugging
- Parallelize independent phases

## 2. Target Architecture: Pure Function Chain

**Principle**: Each phase takes immutable input and returns new data. The driver
composes phases. Mutation is confined to the final assembly step.

```rust
fn lower(project: &TypedProject, options: &LowerOptions) -> Result<Dag<LoweredOp>, LowerError> {
    let context = LoweringContext::from_project(project, options)?;
    let callables = lower_callables(&context)?;
    let scaffolding = derive_scaffolding(&context, &callables)?;
    let transports = derive_transports(&context, &callables)?;
    let dependencies = derive_dependency_edges(&context, &callables)?;
    let service_edges = derive_service_call_edges(&context, &callables, &transports)?;
    let auth_edges = derive_auth_edges(&context, &callables, &transports)?;
    let resources = derive_resource_lifecycle(&context, &callables)?;
    let resource_edges = derive_resource_edges(&context, &callables, &resources)?;
    let contracts = derive_interface_contracts(&context, &callables)?;

    assemble_dag(LoweringParts {
        callables,
        scaffolding,
        transports,
        dependencies,
        service_edges,
        auth_edges,
        resources,
        resource_edges,
        contracts,
    })
}
```

### Key properties

1. **Each `derive_*` function is pure** — takes `&` references, returns owned data
2. **No `&mut` anywhere** except the final `assemble_dag` which builds the actual `Dag`
3. **Each phase returns typed intermediate data** — not raw nodes/edges, but domain types
4. **The driver is a linear chain** — easy to read, test, extend
5. **Errors are typed per-phase** — no catch-all `LowerError::Message(String)`

## 3. Phase Decomposition

### Phase 0: Context Construction (pure)

```rust
struct LoweringContext<'a> {
    project: &'a TypedProject,
    options: LowerOptions,
    endpoint_registry: EndpointRegistry,       // callable → endpoint mapping
    service_registry: ServiceEndpointRegistry,  // service → transport spec
    profile_bindings: Option<ProfileBindings>,  // interface → provider mapping
    data_values: HashMap<String, Value>,        // data declarations
}
```

Replaces the 8-11 parameter threads. Built once, passed by `&` reference everywhere.

### Phase 1: Lower Callables → `Vec<LoweredCallable>` (pure)

```rust
struct LoweredCallable {
    endpoint: LoweredEndpoint,
    node: Node<LoweredOp>,
    service_calls: Vec<ServiceCallSite>,
    control_flow: Vec<ControlFlowSite>,
    return_bindings: Vec<ReturnBinding>,
    uses_clauses: Vec<UsesClause>,
    provides_clauses: Vec<ProvidesClause>,
}
```

Each callable is lowered independently. No cross-callable mutation.

### Phase 2: Derive Scaffolding → `Vec<Node<LoweredOp>>` (pure)

Makegen scaffolding, param source nodes, literal source nodes.
These are additional nodes that don't depend on other phases.

### Phase 3: Derive Transports → `TransportManifest` (pure)

```rust
struct TransportManifest {
    triplets: Vec<TransportTriplet>,  // prepare → execute → parse
    scoped_triplets: Vec<ScopedTriplet>,  // branch/loop-scoped transports
}

struct TransportTriplet {
    prepare: Node<LoweredOp>,
    execute: Node<LoweredOp>,
    parse: Node<LoweredOp>,
    internal_edges: Vec<Edge>,
}
```

Transport triplets are derived from service call analysis. The new
`scope.rs` module (Scoped IR) already extracts the scope information —
this phase uses it to create scoped vs top-level triplets.

### Phase 4-6: Derive Edges → `Vec<DerivedEdge>` (pure)

```rust
struct DerivedEdge {
    source_node: NodeId,
    source_port: PortName,
    target_node: NodeId,
    target_port: PortName,
    kind: EdgeKind,
    provenance: EdgeProvenance,  // which phase created this edge
}
```

Each edge-derivation phase returns a `Vec<DerivedEdge>`. The `provenance`
field enables debugging ("why does this edge exist?") and validation
("did Phase 5 create edges it shouldn't have?").

### Phase 7-8: Derive Resources → `ResourceManifest` (pure)

```rust
struct ResourceManifest {
    lifecycle_nodes: Vec<Node<LoweredOp>>,  // acquire/release
    provide_nodes: Vec<Node<LoweredOp>>,
    edges: Vec<DerivedEdge>,
}
```

### Phase 9: Derive Contracts → `Vec<Node<LoweredOp>>` (pure)

Interface contract verification nodes.

### Assembly: `assemble_dag(parts) -> Dag<LoweredOp>` (mutating, but contained)

The only function that mutates. Takes all the pure intermediate data,
builds the `Dag` with deduplication, and stamps node kinds. This is the
only place `DagBuilder` (or its successor) lives.

## 4. Migration Strategy: Strangler Fig

Don't rewrite — extract phases one at a time from the imperative pipeline.

### Wave 1: Extract `LoweringContext` (S)

Create the context struct. Thread it through existing functions, replacing
the 8-11 parameter tuples. No behavior change — pure mechanical refactor.

**Acceptance**: All tests pass. `#[allow(clippy::too_many_arguments)]` count
drops from 22 to <10.

### Wave 2: Extract Phase 1 — `lower_callables()` → `Vec<LoweredCallable>` (M)

Make callable lowering return structured data instead of mutating the builder.
The main function creates nodes from `LoweredCallable` structs.

**Acceptance**: Each callable can be lowered in a unit test without a builder.

### Wave 3: Extract Phase 3 — `derive_transports()` → `TransportManifest` (M)

This is the biggest win. Transport triplet creation is currently interleaved
with edge wiring. Separating them makes both testable. The `scope.rs` module
is already infrastructure for this.

**Acceptance**: Transport analysis is a pure function. Branch-scoped transports
are derived from `ScopedBody`, not from ad-hoc `detect_*_branches_in_stmts`.

### Wave 4: Extract Phases 4-6 — `derive_*_edges()` → `Vec<DerivedEdge>` (L)

The edge phases are the messiest. They have the most silent drops and the most
complex parameter threading. Extracting them requires:
1. Making `resolve_return_expr_source` return a `ResolvedSource` enum (not `Option<(String, String)>`)
2. Making `wire_callable_return_outputs` return `Vec<DerivedEdge>` (not mutate builder)
3. Replacing all `_ => None` in wiring paths with typed `WiringGap` values

**Acceptance**: Zero silent drops in edge derivation. Every unwired expression
produces a `WiringGap` that flows to diagnostics.

### Wave 5: Extract Phases 7-9 — Resources + Contracts (S)

These are already relatively clean. Mechanical extraction.

### Wave 6: Delete `DagBuilder`, introduce `assemble_dag` (S)

Once all phases return pure data, `DagBuilder` becomes a simple `fold` over
node/edge collections. The deduplication logic moves into `assemble_dag`.

## 5. Defect Elimination

Each wave eliminates a class of defects:

| Wave | Defect Class | Mechanism |
|------|-------------|-----------|
| 1 | Parameter threading errors | Grouped into `LoweringContext` |
| 2 | Callable lowering coupling | Independent `LoweredCallable` per callable |
| 3 | Transport scope bugs | `ScopedBody` replaces ad-hoc detection |
| 4 | Silent drops in wiring | `ResolvedSource` enum replaces `Option` |
| 4 | Missing return edges | `WiringGap` diagnostic replaces `continue` |
| 5 | Resource lifecycle gaps | Pure derivation with exhaustive matching |
| 6 | Deduplication bugs | Single `assemble_dag` with clear semantics |

## 6. Concrete Targets from tasks.md

This refactor subsumes or enables several pending tasks:

| Task | How It's Addressed |
|------|-------------------|
| RT4a (complex return expressions) | Wave 4: `ResolvedSource::ComputeNode` variant |
| RT4b (passthrough missing-input diagnostic) | Wave 4: `WiringGap` type |
| RT4c (lowering completeness gate) | Wave 4: structured `LowerWarnings` in return type |
| RT38 (no panics in lowering) | Wave 1: `LowerError` variants replace panics |
| RT43 (nested field access) | Wave 4: `ResolvedSource::FieldChain` variant |
| RT82 (ban silent `_ => None`) | Wave 4: systematic replacement with typed errors |
| BT-E1 (transport node dedup) | Wave 3: global transport registry in `TransportManifest` |

## 7. File Size Targets

| File | Current | Target | Mechanism |
|------|---------|--------|-----------|
| `lib.rs` | 8,300 | ~2,000 | Extract phases to separate modules |
| `context.rs` (new) | — | ~300 | `LoweringContext` + construction |
| `callable.rs` (new) | — | ~800 | Phase 1: callable lowering |
| `transport.rs` (new) | — | ~600 | Phase 3: transport derivation |
| `wiring.rs` (new) | — | ~1,200 | Phases 4-6: edge derivation |
| `resource.rs` (new) | — | ~400 | Phases 7-8: resource lifecycle |
| `assembly.rs` (new) | — | ~300 | Final DAG construction |
| `scope.rs` (existing) | 583 | ~700 | Expanded for Phase 3 integration |
| `expr.rs` (existing) | 612 | ~800 | `ResolvedSource` enum + matching |
| `eval.rs` (existing) | 1,287 | 1,287 | No change needed |
| `spec.rs` (existing) | 176 | 176 | No change needed |
| `tests.rs` (existing) | 3,275 | ~4,000 | Per-phase test modules |
| **Total** | **~14,000** | **~12,000** | Net reduction from dedup + dead code |

## 8. Risk Mitigation

**Risk**: Snapshot tests break during extraction.
**Mitigation**: Each wave has a parity test: compile all `.dag` modules,
compare `Dag<LoweredOp>` output byte-for-byte with the imperative pipeline.

**Risk**: Performance regression from intermediate allocations.
**Mitigation**: Profile before/after. `Vec<DerivedEdge>` allocation is small
relative to the type-checking and parsing work. If needed, use arena allocation.

**Risk**: Wave 4 (edge derivation) is too large.
**Mitigation**: Break into sub-waves: (4a) dependency edges, (4b) service call
edges, (4c) auth edges, (4d) return wiring. Each sub-wave is independently
shippable.

## 9. Non-Goals

- **Changing the pipeline interface**: `lower_typed_project(&TypedProject) -> Result<Dag<LoweredOp>, LowerError>` stays the same.
- **Rewriting the emitter**: The emit layer is already relatively pure.
- **Changing the IR types**: `Dag<LoweredOp>`, `Node`, `Edge`, `Port` stay as-is.
- **Parallelizing phases**: The pure-function architecture enables this, but
  it's not a goal for this refactor. Sequential composition is fine.
