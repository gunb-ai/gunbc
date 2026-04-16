# Computation Model and Rendering — Design

> **Parent docs:** `THESIS.md`, `INVARIANTS.md` §"Facts Flow
> Forward", `src/v3/SELF_HOSTING.md` §14.7.
>
> **Purpose:** rendering decisions for each target language
> derive from: .dag's computation model (the source facts),
> the target's execution model (the target facts), and a
> per-callable parameter contract (`Borrowed | Consumed`).
> This doc defines each layer and how they compose.
>
> **Status:** design for review.

---

## §1. Source facts: .dag's computation model

Four a priori facts. Not design choices — structural
consequences of the language's primitives.

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

| Fact | Guarantee |
|---|---|
| `Immutable` | Sharing is always safe. No value changes after creation. |
| `Pure` | Output depends only on inputs. Evaluation order flexible. .dag's own primitives are pure; `ExternalRealization` calls are bounded by declared effects (L2 M3). |
| `ExplicitDAG` | All dependencies are `produced_by` edges. No hidden state. |
| `Bounded` | Every computation terminates. |

**What .dag does NOT have:**
- **Ownership.** Values have no owner. Sharing is free (F1).
- **Rust-style borrow scopes.** .dag DOES have binding regions
  (Bind bodies, Loop bodies) that matter for use-count and
  last-use reasoning. But these are structural nesting, not
  target-language lifetime scopes.
- **References.** No borrows, no pointers. Values are values.
- **Clone.** No distinction between a value and a copy.
  Immutable values are identical to their copies.

---

## §2. Target facts: execution models

Each target declares its execution model — how it makes
values exist at runtime.

```dag
type MemoryModel
  = ValueOnly          // SPICE, English, YAML
  | GarbageCollected   // Go, Python, Java
  | RefCounted         // Swift, C++ shared_ptr
  | OwnershipBased     // Rust, C++ unique_ptr

type TargetExecutionModel {
  memory: MemoryModel
}
```

The memory model determines which rendering questions the
emitter must answer:

| Memory model | Ownership decisions? | Sharing/wrapping? | Iterator/container? |
|---|---|---|---|
| ValueOnly | No | No | No |
| GarbageCollected | **No** | Yes (type mapping, boxing) | Yes |
| RefCounted | **No** (refcount handles it) | Yes (wrapping policy) | Yes |
| OwnershipBased | **Yes** | Yes | Yes |

The ownership DECISION dissolves for non-ownership targets.
Type mapping, syntax rendering, container representation, and
iterator shape remain real work for any programming-language
target — those are `LanguageSpec` concerns, not ownership
concerns.

---

## §3. The primary ownership fact: `Borrowed | Consumed`

For ownership-based targets, the emitter needs one fact per
callable parameter: does the callee borrow or consume it?

```dag
type ParameterContract = Borrowed | Consumed

type CallableContract {
  callable: DeclarationId
  params: List<ParameterContract>
}
```

**`Borrowed`:** the callee inspects the value and discards it.
The caller can pass a reference. The value is unchanged after
the call.

**`Consumed`:** the callee takes ownership of the value. The
caller must pass an owned value (move or clone). The callee
may embed it in the return value, drop it, send it somewhere,
or transform it — the caller doesn't know or care.

### §3.1 Why `Borrowed | Consumed`, not `Crosses | Stays`

The earlier design used "scope boundary crossing" as the
primary fact. This fails on at least three cases:

| Callable | What happens | Crosses/Stays says | Correct answer |
|---|---|---|---|
| `id(x) -> x` | Param returned directly, no construct site | Stays (no construct) | **Consumed** — callee needs ownership to return |
| `drop(x) -> Unit` | Param consumed and dropped | Stays (not in return) | **Consumed** — callee needs ownership to drop |
| `send(x, channel)` | Param sent to external, not returned | Stays (not in return) | **Consumed** — callee takes ownership |
| `is_empty(x) -> Bool` | Param inspected only | Stays | **Borrowed** — correct |

`Crosses/Stays` conflates "flows to return value" with
"needs ownership." They overlap for pure .dag functions
(where consumption always means embedding in return), but
diverge for external callables that consume without returning.

`Borrowed | Consumed` is the actual fact the emitter needs.
"Flows to return" is a derived sub-fact useful for .dag body
analysis, not the primary contract.

### §3.2 Two authorities by callable kind

**.dag functions (UserDefined body):** derive from the body
DAG. A parameter is `Consumed` if:
- It flows to the function's return port, OR
- It flows to a construct site (record field, list cons)
  that is itself consumed

A parameter is `Borrowed` if it's only reachable from the
return port through read sites (function calls to borrowing
callees, match scrutinees, field access, comparisons).

**No annotation on Arrow.** The body is the sole authority
for .dag callables. Adding a contract field to Arrow would
create a parallel representation of a derivable fact.

**ExternalRealization:** declared in the realization spec.

```dag
data rust_cons: CallableRealization = {
  strategy: ListCons
  param_contract: [Consumed, Consumed]
}

data rust_is_empty: CallableRealization = {
  strategy: ListIsEmpty
  param_contract: [Borrowed]
}

data rust_fold: CallableRealization = {
  strategy: ListFold
  param_contract: [Borrowed, Consumed, Borrowed]
  // list: borrowed, init: consumed (becomes acc), fn: borrowed
}
```

### §3.3 The four-fixture pressure test

Any correct ownership model must handle these four without
special casing:

```dag
fn id(x: Int) -> Int = x                    // Consumed: returned directly
fn drop(x: Int) -> Int = 0                  // Borrowed: param unused (.dag)
                                             // Consumed: if ExternalRealization drops
fn wrap(x: Int) -> Box<Int> = { value: x }  // Consumed: embedded in record
fn is_empty(list: List<Int>) -> Bool =       // Borrowed: inspected only
  match list { Empty => true, Cons(p) => false }
```

| Callable | .dag derivation | Contract | Rust rendering for caller |
|---|---|---|---|
| `id(x)` | x flows to return | Consumed | pass `T` (move) |
| `drop(x)` | x doesn't flow anywhere | Borrowed | pass `&T` |
| `wrap(x)` | x flows to record field in return | Consumed | pass `T` (move or clone) |
| `is_empty(x)` | x flows through match (read), not to return | Borrowed | pass `&T` |

One fact, four cases, zero special handling.

---

## §4. Rendering at the use site

The source and target models determine **which facts matter**
for a given emission (§1, §2). Within the selected fact
surface, the per-callable contract (§3), `is_copy`, and the
target's `OwnershipPolicy` are real designed facts — they
don't fall out automatically. The models are a gate, not a
complete derivation.

The emitter makes the ownership decision at each **use site**
(the edge between a port and its consumer), not at port lookup
time. The current emitter clones at `render_port_with_locals`
— this must change to use-site rendering.

### §4.0 Emitter API gap (tracked)

The current emitter's rendering primitive is
`render_port(port_id) → String`, which returns
`(name).clone()` for any bound name. This is port-centric —
it cannot consume edge-level facts (which consumer? what
contract?) because it doesn't know the consumer at render
time.

The correct primitive is:
`render_input_use(consumer_node, input_slot) → String`

This knows both the port AND the consumer, so it can look up
the consumer's callable contract, determine Borrowed vs
Consumed for this specific input slot, and render accordingly.

Port-level rendering cannot consume edge-level facts by
construction. This is an emitter-API refactor tracked for
Phase 1.

### §4.1 What the emitter reads per use site

1. **ParameterContract** from §3 (is this parameter Borrowed
   or Consumed?)
2. **is_copy** from the type's realization (is cloning free?)
3. **is_last_consumed_use** from the dependency index (is this
   the final Consumed edge for this port?)

### §4.2 Rendering table for Rust

| Contract | is_copy | Last consumed use? | Rendering |
|----------|---------|---------------------|-----------|
| Borrowed | any | n/a | `&value` |
| Consumed | true | any | `value` (Copy, free) |
| Consumed | false | yes | `value` (move, free) |
| Consumed | false | no | `value.clone()` |

For Go/Python: always `value`. The ownership question doesn't
arise.

### §4.3 Copy type derivation

- **Leaf types:** declared in realization spec (Int → true,
  String → false)
- **Compound types (Conj):** derived. Copy iff ALL fields Copy.
  Not declared — prevents drift.
- **Recursive types:** default non-Copy. Rust's Copy requires
  Sized; recursive types need indirection (Box, Vec), which
  is not Copy.

### §4.4 Target rendering model

```dag
type OwnershipPolicy {
  borrow_syntax: String      // Rust: "&{V}"
  move_syntax: String        // Rust: "{V}"
  clone_syntax: String       // Rust: "{V}.clone()"
  copy_syntax: String        // Rust: "{V}" (same as move for Copy)
}
```

---

## §5. Dependency index (universal)

The dependency reverse-index is always computed, regardless
of target. It's a universal structural fact from F3
(ExplicitDAG).

```dag
type ConsumerEdge {
  port: PortId
  consumer: NodeId
}

type DependencyFacts {
  consumers: List<PortConsumers>
}

fn compute_dependencies(dag: Dag) -> DependencyFacts
```

Consumers of DependencyFacts:
- **Ownership** (§4): last-consumed-use determination
- **Parallelism** (§6): independence detection
- **Complexity**: critical path / cost accounting
- **Dead code**: consumer count = 0 → skip

---

## §6. Parallelism (orthogonal to ownership)

Reads DependencyFacts only. No ownership facts. No target
spec. Two ports with no transitive dependency path are
independent. For pure .dag code, independent operations are
always safe to parallelize (F1 + F2).

**Fold decomposition:** does the body's per-element work
transitively depend on the `acc` parameter? Acc-independent
work is a parallelizable map.

**Sharing primitive refinement:** if two consumers run on
different threads, Rust needs `Arc<T>` (atomic) instead of
`Rc<T>` (non-atomic). Parallelism and ownership are two
independent facts that compose at rendering.

---

## §7. The pipeline

```dag
fn compile(source: String, file: String, spec: LanguageSpec) -> String {
  let dag         = parse(source, file) |> lower |> infer
  let deps        = compute_dependencies(dag)        // universal
  let contracts   = compute_contracts(dag, spec)     // per-callable
  let parallelism = detect_parallelism(dag, deps)    // universal
  let complexity  = compute_complexity(dag, deps, contracts)

  // Target-conditional: only OwnershipBased targets need this
  let ownership   = match spec.execution.memory {
    OwnershipBased -> compute_ownership(dag, deps, contracts, spec)
    _              -> no_ownership_decisions()
  }

  emit(dag, ownership, complexity, parallelism, spec)
}
```

---

## §8. Validated result

The read-vs-construct classification (a proxy for
Borrowed/Consumed at the immediate level) dropped generated
lens clones from 72 → 6. The 6 remaining:

| Clone | Root cause | Fix |
|---|---|---|
| 3× fold acc | `rust_fold.param_contract` not declared yet — init is Consumed | Declare `[Borrowed, Consumed, Borrowed]` |
| 2× PortId | `is_copy` not derived yet — PortId is Copy | Declare leaf Copy types |
| 1× SourceSpan | Non-Copy Consumed at a record construction | Correct — necessary clone |

After fixes: **1 clone.**

---

## §9. Verification

Tests verify the facts, not symptoms.

1. **ParameterContract per callable.** The four-fixture test
   (§3.3): `id` → [Consumed], `drop` → [Borrowed],
   `wrap` → [Consumed], `is_empty` → [Borrowed].

2. **Additional callables.** `cons` → [Consumed, Consumed].
   `fold` → [Borrowed, Consumed, Borrowed].

3. **is_copy derivation.** Int → true. PortId → true.
   SourceSpan → false. { a: Int, b: Int } → true.
   { a: Int, b: String } → false. List<T> → false (recursive).

4. **Rendering parity.** Generated lens matches handwritten
   oracle.

5. **Roundtrip compilation.** Every generated artifact compiles
   with rustc.

6. **Clone-count pinning.** ~6 at Phase 1, 1 at Phase 2.

---

## §10. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | `std/computation_model.dag`. `TargetExecutionModel` in specs. Emitter gates ownership on target memory. Conservative default: all params Borrowed (safe). `is_copy` on leaf types. Move emitter from port-lookup cloning to use-site rendering. Clone count ~6. |
| **Phase 2** | L2 | `ParameterContract` analysis: body-walk for .dag callables, `param_contract` declared for ExternalRealization. `is_copy` composition. Last-consumed-use tracking. Clone count → 1. |
| **Phase 3** | L2+ | Parallelism Rc/Arc. Complexity reads contracts for clone cost. |
| **Phase 4** | L3 | Self-analysis. Multi-target: same DAG → Rust + Go. Go emits without ownership stage. |

---

## §11. When this doc updates

- `std/computation_model.dag` lands → §1 implemented
- Phase 1 lands → clone count pinned
- Four-fixture pressure test green → §3.3 validated
- Phase 2 lands → contracts verified, clone → 1
- Multi-target lands → gating validated
- All phases → doc archives
