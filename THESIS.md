# gunbc Thesis: Zero Bugs by Construction

This is the parent document. Everything else — ROADMAP, INVARIANTS,
MODELING, architecture, design docs — serves this thesis.

## The claim

A `.dag` program that compiles has **zero bugs**. Not "fewer bugs" —
zero. The compiler either proves the program correct or refuses to
emit it. Generated tests cover what static analysis cannot.

This is achievable because `.dag` is a closed system: all data is
finite, all iteration is bounded, all composition preserves
boundedness. In a closed system, correctness properties are
**consequences of the model** — they don't require separate analysis.

## What "zero bugs" means concretely

Three tiers. A compiled `.dag` program must satisfy all three.

### Tier 1: Structural bugs — impossible by construction

These bugs cannot be written. The type system, exhaustiveness
checking, and structural descent proofs make them unrepresentable.

| Bug class | Mechanism | Status |
|-----------|-----------|--------|
| Field typos in generated code | Emitter derives names from declarations | DONE |
| Field typos in `.dag` source | FieldNotFound diagnostic | DONE |
| Non-exhaustive match | NonExhaustiveMatch diagnostic | DONE |
| Type mismatches | TypeMismatch diagnostic (branches, args, returns) | DONE |
| Bare container types | ArityMismatch diagnostic | DONE |
| Map key type mismatch | Infer-stage type check | DONE |
| Stale imports | UnresolvedType / MissingExport diagnostics | DONE |
| Circular dependencies | CircularDependency diagnostic | DONE |
| Cross-target drift | Single `.dag` declaration → all targets | DONE |
| Diamond dependency divergence | Module graph deduplicates imports | DONE |
| Non-termination | Structural descent proof (CX gate) | **424 violations → 0, then blocking** |
| Record literal completeness | Missing-field diagnostic | **partial** |
| Coercion completeness | Fail-closed inhabitant lookup | DONE |

**Gating item:** CX gate. Once 424 → 0 and the gate is blocking,
every function that compiles is proven to terminate. This is the
single biggest remaining item in Tier 1.

### Tier 2: Runtime safety — proven safe or total

These bugs compile today but crash at runtime. Closing them means
the compiled program cannot panic, trap, or produce silent wrong
data from safe operations.

| Bug class | Current state | Path to zero |
|-----------|--------------|--------------|
| Division by zero | Unchecked | Model divisor as NonZero or emit checked_div |
| Integer overflow | Wraps or panics (Rust-dependent) | Bounded arithmetic or checked ops |
| String/array out-of-bounds | Silently returns empty string | Require bounds proof or emit checked access |
| Optional force-unwrap | Unchecked panic | Require match/if-let to extract; no `.force()` |
| Partial functions | Some runtime helpers are partial | Make all runtime functions total |

**Design principle:** either prove the precondition at compile time
(refinement types) or make the operation total (return Option,
use checked arithmetic). No partial functions in the runtime.

### Tier 3: Logic bugs — generated tests as proof

The compiler cannot know your intent. But it can generate tests
from your declarations that verify behavioral contracts.

| Test level | What it proves | Status |
|------------|---------------|--------|
| L0: Structural tests from data | Coercion mappings are complete and consistent | DONE |
| L1: Pipeline unit tests | Compiler stages produce correct output | DONE (388 tests) |
| L2: Bootstrap self-hosting | Compiler can compile itself | DONE |
| L3: Syntax validity | Emitted code parses in target language | DONE |
| L4: Semantic correctness | Emitted code executes and produces correct results | **not implemented** |
| L5: Cross-language equivalence | Same `.dag` → same behavior in Rust/Python/Go | **not implemented** |
| L6: Exhaustive form coverage | Every structural form compiles to every target | **not implemented** |
| L7: Algebraic law verification | fold/map/filter obey their algebraic laws | **not implemented** |

**Gating items:** L4 (semantic correctness) is the critical gap.
A program can compile, pass L0-L3, and still compute the wrong
answer. L4 requires executing emitted code against oracles derived
from the `.dag` declarations.

---

## Free consequences

These are not separate features. They fall out of the closed model
once Tiers 1-3 are satisfied. They require no additional language
design — only that the compiler has enough information to apply them.

### Automatic parallelism (map-reduce)

If every operation is a fold/descend/repeat, complexity is known,
and there is no shared mutable state, then:

- **fold over partitioned data → map-reduce.** The compiler knows
  the fold's accumulator type, the element type, and the combining
  function. If the combining function is associative and commutative
  (declared via algebra inhabitant in `std/algebra.dag`), the fold
  can be partitioned across cores with no synchronization.

- **descend on tree children → parallel tree walk.** Each child
  subtree is independent (DAG property — no back-edges). The
  compiler can descend children in parallel and join results.

- **repeat with known bound → pipeline parallelism.** If successive
  iterations are independent (pure function, no accumulator
  mutation), iterations can overlap.

**Prerequisite:** provenance on bindings (Stream A) + ownership
proof (Stream B) + CX gate closed. The compiler needs to know
that the fold body is pure, that the accumulator isn't aliased,
and that the operation terminates.

### Automatic memoization

A pure function with known cost and no side effects can be
memoized by the emitter. The compiler already knows:
- Whether the function is pure (no service calls, no mutation)
- Its complexity bound (CX)
- Its argument types (hashable or not)

Once these facts flow through bindings, the emitter can insert
memoization for expensive pure functions automatically.

### Space bound proofs

If complexity is known and all data is finite, the maximum heap
allocation for any function call is computable at compile time.
This enables:
- Stack overflow prevention (known recursion depth)
- Memory budget enforcement (known allocation ceiling)
- Embedded/constrained deployment (prove program fits in N bytes)

### Cross-language optimization

Each target language has different performance characteristics
(Go's goroutines are cheap; Rust's async is zero-cost; Python's
GIL constrains parallelism). The compiler already models
`LanguageSpec` per target. With full cost information, the emitter
can choose target-specific strategies:
- Rust: inline small folds, parallelize large ones via Rayon
- Go: emit goroutines for parallel descents
- Python: emit multiprocessing for CPU-bound folds

---

## How the docs connect

```
THESIS.md (this file)
  ├── WHY: zero bugs by construction
  │
  ├── INVARIANTS.md — rules that protect the thesis
  │     (modeling faithfulness, root-cause depth, decidability, ...)
  │
  ├── MODELING.md — how to extend the language without breaking the thesis
  │     (concept DAG, DFS before defining, compositional modeling, ...)
  │
  ├── ROADMAP.md — current state and work plan
  │     (lanes, streams, tracks — all serving the thesis)
  │
  └── docs/
        ├── architecture.md — substrate design (Node + Edge)
        ├── cx-design.md — complexity analysis (Tier 1 gating item)
        ├── ownership-design.md — ownership proofs (Tier 1 + parallelism)
        ├── coercion-design.md — type coercion (Tier 1, DONE)
        ├── testing-strategy.md — generated tests (Tier 3)
        └── compiler-laws.md — algebraic laws (Tier 3 + parallelism)
```

Every design doc should state which tier it serves and what
"zero bugs" claim it advances.

---

## Current scoreboard

Updated manually. If this is stale, check ROADMAP.md for details.

```
Tier 1: Structural         ██████████████░░ ~85%
  CX gate:                 424 violations remaining (non-blocking)
  Record completeness:     partial

Tier 2: Runtime safety      ░░░░░░░░░░░░░░░ ~0%
  No runtime safety proofs yet

Tier 3: Generated tests     ████░░░░░░░░░░░ ~25%
  L0-L3 done, L4-L7 not started

Free consequences:          ░░░░░░░░░░░░░░░ blocked on Tiers 1-2
  Parallelism:             blocked (needs CX + ownership + purity)
  Memoization:             blocked (needs CX + purity)
  Space bounds:            blocked (needs CX)
```

---

## The test: when is it real?

The "zero bugs" claim becomes real when a user can write:

```dag
type Order { customer: String  amount: Float  status: OrderStatus }
type OrderStatus = Pending | Approved | Declined | Refunded

service OrderService {
  fn create_order(req: CreateOrderRequest) -> Order via rest::post("/orders")
  fn get_order(id: String) -> Order via rest::get("/orders/{id}")
}
```

...and the compiler:
1. **Refuses** to emit if any structural invariant is violated (Tier 1)
2. **Proves** that no runtime operation can panic (Tier 2)
3. **Generates** tests that verify the service behaves correctly (Tier 3)
4. **Parallelizes** independent operations automatically (free consequence)

No test is written by hand. No runtime crash is possible. No bug
class is left to developer discipline.
