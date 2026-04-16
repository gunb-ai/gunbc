# Dependency, Scope, and Target Rendering — Design

> **Parent docs:** `THESIS.md` (omni-emission), `INVARIANTS.md`
> §"Facts Flow Forward", `src/v3/SELF_HOSTING.md` §14.7.
>
> **Purpose:** the DAG carries two structural facts — dependency
> edges and scope hierarchy. These are universal, computed once,
> target-agnostic. Whether a target language CONSUMES these facts
> depends on its reference model. Value-only and GC targets
> ignore scope crossings entirely. Only ownership-based targets
> (Rust, C++ unique_ptr) need the scope-crossing analysis.
>
> **Key thesis connection:** the same DAG emits to many targets.
> Ownership isn't a mandatory pipeline stage — it's a fact that
> only some targets consult. The core pipeline stays minimal.
>
> **Status:** design for review.

---

## §1. The universal structural facts

Two facts are always computed from the DAG. They are
target-agnostic and spec-independent.

### §1.1 Dependency edges

The DAG's `produced_by` edges + behavior input lists define
a complete dependency graph. Created at lowering. Never lost.

```dag
module std.dependency

type ConsumerEdge {
  port: PortId
  consumer: NodeId
}

type DependencyFacts {
  consumers: List<PortConsumers>
}

fn compute_dependencies(dag: Dag) -> DependencyFacts
```

This is a reverse index of `produced_by`. For each port, who
reads it. Built by scanning all behaviors' input lists. Pure
structural fact.

### §1.2 Scope hierarchy and boundary crossings

Every value belongs to the scope it was produced in. Functions
nest let-blocks nest lambda bodies nest expression scopes.
An edge from producer to consumer either:

- **Stays within scope** — sibling expressions, inline
  consumption. The value is alive for the entire consumer's
  evaluation.
- **Crosses outward** — stored in a return value, embedded in
  a longer-lived structure, passed to a callee that returns
  it. The value needs to live BEYOND its producer's scope.

```dag
module std.scope

type ScopeCrossing = WithinScope | CrossesOutward

type EdgeScope {
  port: PortId
  consumer: NodeId
  crossing: ScopeCrossing
}

type ScopeFacts {
  edges: List<EdgeScope>
}

fn compute_scope_facts(dag: Dag) -> ScopeFacts
```

`crossing` is derivable: compare the producer's scope nesting
with the consumer's. If the consumer is in an outer (or
different) scope, the value crosses outward.

**Transitive crossings.** If a value is passed to a callee
that embeds it in its return value, the value transitively
crosses the callee's scope boundary. For .dag callables, this
is derivable from the body DAG — does the parameter port
reach the return port? For `ExternalRealization` callables,
it's declared in the realization spec:

```dag
// Per-parameter: does the callee cause this value to escape?
type ParameterScope = Contained | Escaping

// rust.dag
data rust_cons: CallableRealization = {
  strategy: ListCons
  param_scope: [Escaping, Escaping]   // head + tail stored in return
}

data rust_is_empty: CallableRealization = {
  strategy: ListIsEmpty
  param_scope: [Contained]            // list inspected and discarded
}

data rust_fold: CallableRealization = {
  strategy: ListFold
  param_scope: [Contained, Escaping, Contained]
  // list: contained, init: escaping (becomes acc), fn: contained
}
```

---

## §2. Target reference models

Whether a target language CONSUMES scope facts depends on its
reference model. Four classes:

| Target class | Examples | Scope-crossing decision |
|---|---|---|
| **Value-only** | English, SPICE, YAML, SQL | Doesn't exist. No logical references. Just emit values. |
| **GC** | Go, Python, Java, JavaScript | Trivial. Always "reference." GC extends lifetimes. |
| **Refcount** | Swift, C++ shared_ptr | Always "shared reference." Refcount at boundaries. |
| **Ownership** | Rust, C++ unique_ptr | The scope-crossing question matters. Borrow within scope, own at crossings. |

Three of four classes dissolve the question entirely. The
entire borrow/move/clone discussion only applies to the
bottom row.

```dag
type ReferenceModel
  = ValueOnly            // no references — emit values directly
  | GarbageCollected     // runtime manages lifetimes
  | RefCounted           // shared ownership via refcount
  | OwnershipBased {     // scope crossings require ownership transfer
      crossing_policy: CrossingPolicy
    }

type CrossingPolicy {
  within_scope: String    // Rust: "&{V}" (borrow)
  crossing_copy: String   // Rust: "*{V}" (deref Copy type)
  crossing_move: String   // Rust: "{V}" (move at last use)
  crossing_clone: String  // Rust: "{V}.clone()" (clone non-Copy)
}
```

Target declarations:

```dag
// rust.dag
data rust_reference_model: ReferenceModel = OwnershipBased {
  crossing_policy: {
    within_scope: "&{V}"
    crossing_copy: "*{V}"
    crossing_move: "{V}"
    crossing_clone: "{V}.clone()"
  }
}

// go.dag
data go_reference_model: ReferenceModel = GarbageCollected

// python.dag
data python_reference_model: ReferenceModel = GarbageCollected

// spice.dag
data spice_reference_model: ReferenceModel = ValueOnly

// english.dag
data english_reference_model: ReferenceModel = ValueOnly
```

---

## §3. The rendering decision

For each consumer edge, the emitter reads:

1. The target's `ReferenceModel`
2. If `OwnershipBased`: the edge's `ScopeCrossing` from §1.2
3. If crossing + ownership: `is_copy` and `is_last_use`

```dag
fn render_edge(
  edge: EdgeScope,
  model: ReferenceModel,
  is_copy: Bool,
  is_last_use: Bool
) -> String =
  match model {
    ValueOnly        -> edge.port     // just the value
    GarbageCollected -> edge.port     // just the value
    RefCounted       -> rc(edge.port) // wrap in shared ref
    OwnershipBased { crossing_policy: policy } ->
      match edge.crossing {
        WithinScope    -> policy.within_scope     // &value
        CrossesOutward ->
          if is_copy then policy.crossing_copy    // *value
          else if is_last_use then policy.crossing_move  // value
          else policy.crossing_clone              // value.clone()
      }
  }
```

**For value-only and GC targets, scope analysis is never
consulted.** The emitter checks the reference model first.
If it's not `OwnershipBased`, skip scope facts entirely.
The core pipeline doesn't bake ownership reasoning into every
path.

### §3.1 Copy type derivation

- **Leaf types:** `is_copy` declared in realization spec
  (Int → true, Bool → true, PortId → true, String → false)
- **Compound types (Conj):** derived. A record is Copy iff
  ALL fields are Copy. Not declared — prevents drift.

### §3.2 Last-use determination

A crossing edge is the "last use" if no subsequent consumer
of the same port also has `CrossesOutward`. Derived from
evaluation order + the consumer list in `DependencyFacts`.

---

## §4. Parallelism — orthogonal to scope

Parallelism reads `DependencyFacts` (Layer 1) only. It does
NOT read scope facts. Two ports with no transitive dependency
path are independent. In a pure language, independent
operations are always safe to parallelize.

**Fold decomposition:** which body nodes depend on `acc`?
Acc-independent = parallelizable map. Acc-dependent =
sequential reduce.

**Parallelism refines the SHARING PRIMITIVE, not whether
sharing is needed:**

| Sharing context | Rust primitive |
|---|---|
| Single-threaded sharing | `Rc<T>` (non-atomic) |
| Cross-thread sharing | `Arc<T>` (atomic) |
| No sharing (last use) | move |

Parallelism and scope are two independent facts that compose
at the rendering layer. Scope tells you WHETHER to own.
Parallelism tells you HOW to share (Rc vs Arc).

---

## §5. The pipeline composition

```dag
fn compile(source: String, file: String, spec: LanguageSpec) -> String {
  let dag         = parse(source, file) |> lower |> infer
  let deps        = compute_dependencies(dag)       // always
  let scope       = compute_scope_facts(dag)        // always
  let parallelism = detect_parallelism(dag, deps)   // always (structural)

  // Target-conditional: only if spec.reference_model is OwnershipBased
  let ownership   = if needs_ownership(spec) then
                      compute_ownership(dag, scope, spec)
                    else
                      trivial_ownership()  // GC/value: no decisions

  let complexity  = compute_complexity(dag, deps, ownership)
  emit(dag, ownership, complexity, parallelism, spec)
}
```

The core pipeline computes universal facts (deps, scope,
parallelism). Ownership rendering is gated on the target's
reference model. For Go, Python, SPICE, English — the
ownership stage is a no-op.

---

## §6. Validated result and remaining work

### §6.1 What's proven

The read-vs-construct classification (a proxy for scope
crossing) dropped generated lens clones from 72 → 6. This
validates the direction: values that stay within scope should
be borrowed, values that cross outward need ownership.

### §6.2 The 6 remaining clones

| Clone | Root cause | Fix |
|---|---|---|
| 3x fold accumulator | Rust's fold takes acc by value (Escaping parameter). Emitter doesn't read `param_scope` yet. | `rust_fold.param_scope: [Contained, Escaping, Contained]` — emitter renders `mut acc` for Escaping fold params. |
| 2x PortId deref | PortId is Copy. `.clone()` works but `*value` is correct. | `is_copy: true` on PortId in realization spec. |
| 1x SourceSpan | Non-Copy value at a CrossesOutward edge (record construction). Genuinely necessary. | None — this clone is correct. |

**After fixes: 1 clone.** The single remaining clone is a
non-Copy value that crosses a scope boundary. The model
correctly identifies it as necessary.

---

## §7. Verification

The model claims: if `ScopeCrossing` is correctly computed per
edge and `is_copy` is correctly derived per type, the emitter
produces correct code. Tests verify the facts.

1. **ScopeCrossing per edge.** Function argument to
   non-escaping callee → `WithinScope`. Record field →
   `CrossesOutward`. Return value → `CrossesOutward`. Argument
   to `cons` → `CrossesOutward` (transitive via `param_scope`).

2. **is_copy derivation.** Int → true. PortId → true.
   SourceSpan → false. { a: Int, b: Int } → true.
   { a: Int, b: String } → false.

3. **Rendering parity.** Generated lens matches handwritten
   oracle.

4. **Roundtrip compilation.** Every generated artifact compiles
   with rustc (Rust's borrow checker rejects incorrect scope
   analysis).

5. **Clone-count pinning.** ~6 at Phase 1, 1 at Phase 2.

---

## §8. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | `std/scope.dag` types. `compute_scope_facts` with direct crossings (record fields, return values). Conservative default for callees (treat as Contained unless declared Escaping). `is_copy` on leaf types. Emitter reads `ReferenceModel`, renders `&T` for WithinScope on Rust target. Clone count ~6. |
| **Phase 2** | L2 | Transitive crossings (param_scope on callables). Body-walk derivation for .dag callables. Declared for ExternalRealization. `is_copy` composition for compound types. Last-use tracking. Clone count → 1. |
| **Phase 3** | L2+ | Parallelism sharing class (Rc vs Arc). Complexity reads scope + ownership for accurate cost. Dead-code from DependencyFacts. |
| **Phase 4** | L3 | Self-analysis. Multi-target validation (same DAG emitted to Rust + Go, Rust has ownership decisions, Go has none). |

---

## §9. When this doc updates

- Phase 1 lands → clone count pinned at ~6
- Phase 2 lands → transitive crossings verified, clone → 1
- Multi-target lands → ownership-free emission validated
- All phases → doc archives
