# Computation Model, Dependency, and Rendering — Design

> **Parent docs:** `THESIS.md`, `INVARIANTS.md`, `SELF_HOSTING.md`
> §14.7.
>
> **Purpose:** model the a priori facts about .dag's computation
> model and each target's execution model. Ownership, scope,
> parallelism, and complexity EMERGE from the composition of
> these two models. Nothing is designed — everything derives.
>
> **Status:** design for review.

---

## §1. The root facts

Four a priori facts about .dag. Not design choices — structural
consequences of the language having no mutation primitive, no
side-effect primitive, explicit data-flow edges, and bounded
iteration primitives.

```dag
module std.computation_model

type Mutability = Immutable | Mutable
type Purity = Pure | Effectful
type Structure = ExplicitDAG | Arbitrary
type Iteration = Bounded | Unbounded

type ComputationModel {
  mutability: Mutability
  purity: Purity
  structure: Structure
  iteration: Iteration
}

data dag_model: ComputationModel = {
  mutability: Immutable
  purity: Pure
  structure: ExplicitDAG
  iteration: Bounded
}
```

### What each fact guarantees

| Fact | Guarantee | What it dissolves |
|---|---|---|
| `Immutable` | Sharing is always safe. No value can be modified after creation. | Data races, aliasing bugs, defensive cloning |
| `Pure` | Output depends only on inputs. Evaluation order is flexible. `.dag`'s own primitives are pure; `ExternalRealization` calls are bounded by their declared effects (effect annotations, L2 M3). | Effect ordering, scope-as-correctness-concern |
| `ExplicitDAG` | All dependencies are `produced_by` edges. No hidden state. | Alias analysis, dependency discovery, escape reconstruction |
| `Bounded` | Every computation terminates. | Halting problem, unbounded cost |

### What's NOT in the root facts

These concepts do NOT exist in .dag:
- **Ownership.** Values have no owner. They exist from
  production to last use. Sharing is free (F1).
- **Scope.** The DAG has structural nesting (Bind bodies, Loop
  bodies). Lexical scope is a target-language concept; the
  emitter maps DAG structural nesting onto the target's scope
  primitives.
- **References.** No borrows, no pointers, no indirection.
  Values are values.
- **Clone.** No distinction between "the value" and "a copy of
  the value." Immutable values are identical to their copies.

These concepts appear only when the .dag DAG is mapped to a
target language that has them.

---

## §2. Target execution models

Each target language has its own execution model — how it
makes values exist at runtime, how it manages their lifetime,
and whether it has scope-based resource constraints.

```dag
module std.target_model

type MemoryModel
  = ValueOnly          // SPICE, English, YAML: no runtime memory management
  | GarbageCollected   // Go, Python, Java: runtime extends lifetimes
  | RefCounted         // Swift, C++ shared_ptr: shared ownership via refcount
  | OwnershipBased     // Rust, C++ unique_ptr: scope-based resource management

type ScopeModel
  = NoScope            // SPICE, English: no lexical nesting
  | LexicalScoping     // Most programming languages: { } blocks, functions

type TargetExecutionModel {
  memory: MemoryModel
  scope: ScopeModel
}
```

Target declarations:

```dag
data rust_execution: TargetExecutionModel = {
  memory: OwnershipBased
  scope: LexicalScoping
}

data go_execution: TargetExecutionModel = {
  memory: GarbageCollected
  scope: LexicalScoping
}

data python_execution: TargetExecutionModel = {
  memory: GarbageCollected
  scope: LexicalScoping
}

data spice_execution: TargetExecutionModel = {
  memory: ValueOnly
  scope: NoScope
}

data english_execution: TargetExecutionModel = {
  memory: ValueOnly
  scope: NoScope
}
```

---

## §3. What emerges from the composition

The rendering for any `.dag` program on any target is
determined by: `dag_model × TargetExecutionModel`. No
per-target analysis is designed — it falls out.

### §3.1 Sharing safety (from F1: Immutable)

`dag_model.mutability == Immutable` → sharing is always safe
for ALL targets. No target ever needs to worry about data
races or aliasing when rendering .dag code. This is universal.

Consequence: the ONLY question at a sharing point is
"what mechanism does this target use for sharing?" — not
"is sharing safe?"

### §3.2 Evaluation flexibility (from F2: Pure)

`dag_model.purity == Pure` → the emitter can choose any
evaluation order that respects data dependencies. The emitter
creates the target's scope structure as a rendering decision.
Scope is not a property of the .dag program — it's a property
of how the emitter renders it.

Consequence: "scope boundary crossing" is a rendering fact,
not a source fact. The emitter knows where it creates scopes
because it creates them.

### §3.3 Explicit independence (from F3: ExplicitDAG)

`dag_model.structure == ExplicitDAG` → two ports with no
transitive `produced_by` path are independent. This is
structural. No analysis needed.

Consequence: parallelism opportunities are visible by
inspection. The emitter reads the DAG, sees independent
subgraphs, renders them in parallel if the target supports it.

### §3.4 Total complexity (from F4: Bounded)

`dag_model.iteration == Bounded` → every function has a
computable cost bound. The longest dependency chain is the
critical path.

### §3.5 The composition table

| Source model | Target memory | What the emitter does |
|---|---|---|
| Immutable + Pure | **ValueOnly** | Emit values directly. No sharing, no scope, no ownership. The simplest case. |
| Immutable + Pure | **GarbageCollected** | Emit values. GC extends lifetimes. Scope exists but crossings are free. |
| Immutable + Pure | **RefCounted** | Emit shared references. Refcount at creation. Scope crossings handled by refcount. |
| Immutable + Pure | **OwnershipBased** | Emit borrows within scope, ownership transfer at scope crossings. The ONLY case that needs crossing analysis. |

Three of four target classes produce trivial rendering. The
entire ownership/scope discussion exists for one class.

---

## §4. Rendering for ownership-based targets

This section only applies when `target.memory == OwnershipBased`
(currently: Rust).

### §4.1 Scope boundaries are emitter-created

The emitter maps .dag's DAG to Rust's scope structure. For
v3's current emission strategy (sequential let bindings within
a function body), a value's scope is the function body it's
emitted in. A value crosses a scope boundary when:

- It's stored in a record/list that's returned from the function
- It's passed to a callee that stores it in ITS return value
  (transitive crossing)

These are rendering facts — the emitter knows them because it
creates the scope structure.

### §4.2 The per-callable crossing declaration

For each callable, the emitter needs to know: does this callee
cause its parameter to cross a scope boundary (by embedding it
in the return value)?

**For .dag functions:** derivable from the body DAG. Walk from
the return port backward. If a parameter is reachable through
a construct site (record field, list cons), the parameter
crosses. No annotation on Arrow — the body is the authority.

**For ExternalRealization:** declared in the realization spec.

```dag
type ParameterCrossing = Stays | Crosses

data rust_cons: CallableRealization = {
  strategy: ListCons
  param_crossing: [Crosses, Crosses]
}

data rust_is_empty: CallableRealization = {
  strategy: ListIsEmpty
  param_crossing: [Stays]
}

data rust_fold: CallableRealization = {
  strategy: ListFold
  param_crossing: [Stays, Crosses, Stays]
}
```

### §4.3 Rendering at each edge

The Rust emitter declares its rendering policy:

```dag
type OwnershipPolicy {
  stays_in_scope: String       // "&{V}" — borrow
  crosses_copy: String         // "*{V}" — deref Copy type
  crosses_move: String         // "{V}" — move at last use
  crosses_clone: String        // "{V}.clone()" — clone non-Copy
}
```

The decision per edge:

| Crosses? | is_copy | Last crossing use? | Rendering |
|----------|---------|---------------------|-----------|
| No | any | n/a | `&value` (borrow) |
| Yes | true | any | `*value` (deref, free) |
| Yes | false | yes | `value` (move, free) |
| Yes | false | no | `value.clone()` (O(size)) |

### §4.4 Copy type derivation

- Leaf types: declared in realization spec (`Int → true`,
  `String → false`)
- Compound types: derived. Record is Copy iff ALL fields are
  Copy. Not declared — prevents drift.
- Recursive types: default to non-Copy. Rust's `Copy` requires
  `Sized` with no indirection; recursive types need indirection
  (`Box`, `Vec`), which is not Copy.

---

## §5. Parallelism (orthogonal)

Parallelism reads the DAG structure directly (F3: ExplicitDAG).
It does NOT read scope facts or ownership facts. Two ports with
no transitive dependency path are independent. Period.

**Fold decomposition:** does the fold body's per-element
computation reach the `acc` parameter? If not, the fold
contains a map (parallelizable).

**Parallelism refines the sharing PRIMITIVE for ownership
targets:** if two consumers run on different threads, Rust
needs `Arc<T>` (atomic) instead of `Rc<T>` (non-atomic).
Parallelism and scope are two independent facts that compose
at rendering.

---

## §6. The pipeline

```dag
fn compile(source: String, file: String, spec: LanguageSpec) -> String {
  let dag         = parse(source, file) |> lower |> infer
  let deps        = compute_dependencies(dag)       // universal (F3)
  let parallelism = detect_parallelism(dag, deps)   // universal (F1+F3)
  let complexity  = compute_complexity(dag, deps)    // universal (F4)

  // Target-conditional:
  let rendering   = match spec.execution.memory {
    OwnershipBased -> compute_ownership_rendering(dag, deps, spec)
    _              -> trivial_rendering()
  }

  emit(dag, rendering, complexity, parallelism, spec)
}
```

Universal facts (deps, parallelism, complexity) are always
computed. Ownership rendering is gated on the target's memory
model. For Go, Python, SPICE, English — no ownership stage.

---

## §7. Validated result

The read-vs-construct classification (a proxy for scope
crossing) dropped generated lens clones from 72 → 6. The 6
remaining:

| Clone | Root cause | Fix |
|---|---|---|
| 3× fold acc | `rust_fold.param_crossing` not declared yet | Declare `[Stays, Crosses, Stays]` |
| 2× PortId | `is_copy` not derived yet | Declare leaf Copy, derive compound |
| 1× SourceSpan | Non-Copy value at a genuine scope crossing | Correct — necessary clone |

After fixes: **1 clone.** The model correctly identifies it
as the only genuinely necessary ownership transfer.

---

## §8. Verification

The model claims: rendering is fully determined by
`ComputationModel × TargetExecutionModel × DAG structure`.
Tests verify the root facts and their composition.

1. **Root facts hold.** .dag programs compile with
   `dag_model = { Immutable, Pure, ExplicitDAG, Bounded }`.
   .dag's own primitives enforce these facts. Programs with
   cycles or unbounded iteration are rejected at compile time.
   `ExternalRealization` calls are bounded by their declared
   effects (the purity guarantee applies to .dag primitives;
   external calls carry effect annotations, L2 M3).

2. **ParameterCrossing per callable.** `cons` → [Crosses,
   Crosses]. `is_empty` → [Stays]. `fold` → [Stays, Crosses,
   Stays]. Derived for .dag callables, declared for external.

3. **is_copy derivation.** Int → true. PortId → true.
   SourceSpan → false. { a: Int, b: Int } → true.
   { a: Int, b: String } → false.

4. **Rendering parity.** Generated lens matches handwritten
   oracle on all fixtures.

5. **Cross-target validation.** Same .dag program emitted to
   Rust (ownership decisions) and Go (no ownership decisions)
   produces behaviorally equivalent code.

6. **Clone-count pinning.** ~6 at Phase 1, 1 at Phase 2.

---

## §9. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | Declare `ComputationModel` in `std/`. Build consumer index in emitter. Emitter reads `TargetExecutionModel` and gates ownership stage. Conservative default for callables (all params Stay). `is_copy` on leaf types. Clone count ~6. |
| **Phase 2** | L2 | `ParameterCrossing` analysis. Body-walk derivation for .dag callables. Declared for ExternalRealization. `is_copy` composition. Last-crossing-use tracking. Clone count → 1. |
| **Phase 3** | L2+ | Parallelism rendering (Rc vs Arc). Complexity reads crossing facts for clone cost. Multi-target validation. |
| **Phase 4** | L3 | Self-analysis. Clone count zero. Same DAG → Rust + Go → both correct. |

---

## §10. When this doc updates

- `std/computation_model.dag` lands → §1 is implemented
- Phase 1 lands → clone count pinned
- Phase 2 lands → crossing analysis verified
- Multi-target → §3.5 composition validated empirically
- All phases → doc archives
