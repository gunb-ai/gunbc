# gunbc Thesis

This is the parent document. Everything else — ROADMAP, INVARIANTS,
MODELING, architecture, design docs — serves this thesis.

## What gunbc is

gunbc is a **causal engine**. Its job is to validate that a program
has a consistent, coherent cause-and-effect flow from source (inputs,
declarations, intent) to drain (outputs, behavior, emitted code).

Traditional compilers work bottom-up: the developer has high-level
intent, and the compiler's job is to align machine code to that
intent. The question is "can I make this execute?" gunbc inverts
this. The question is **"is what you said sound?"** The compiler
validates intent against itself — every declaration must be
consistent with every other declaration, every data flow must have
a valid source and a valid drain, every computation must terminate
with a proven bound. If the answer is yes, emission is mechanical
translation. The emitted code is a consequence of the validated
intent, not an interpretation of it.

**If it compiles, the intent is sound and will execute as declared.**

The only possible failure after compilation is an external reality
mismatch — you declared access to a resource you don't actually
have, or an external service returned something outside its declared
contract. These are facts the compiler cannot verify because they
exist outside the program's causal graph. If those facts are
structured in the language (service declarations, transport
contracts), the compiler can validate them too.

## Why this works

`.dag` is a closed system. All data is finite (Bit/Word64). All
iteration is bounded (fold/descend/repeat). Composition preserves
boundedness. In a closed system, correctness properties —
termination, type safety, exhaustiveness, ownership, complexity
bounds — are **consequences of the model**, like conservation laws
in physics. They don't require separate analysis passes. They
emerge from the structure.

This is what makes the causal engine possible. In an open system
(Turing-complete, unbounded iteration, implicit coercions), you
cannot validate all causal links — some are undecidable. In a
closed system, every link is checkable, so the compiler can prove
the entire causal chain from source to drain.

## Correctness dimensions

Correctness is not one property — it is many orthogonal dimensions:
termination, type safety, ownership, side effects, purity,
idempotence, space bounds. In traditional systems these are separate
tools (type checker, linter, static analyzer, profiler) that you
opt into. In gunbc, they are **inescapable properties of the
system**, like conservation laws in physics. You don't opt into
gravity.

Every dimension is:
1. **Declared in `std/`** as a structural type with lattice
   operations (meet, join, top, bottom)
2. **Computed at binding sites** during inference — no separate
   analysis pass
3. **Carried through the IR** on bindings, from computation to
   consumption
4. **Enforced universally** — all code is subject to all dimensions,
   no escape hatch, no wrapper functions

The compiler doesn't have "a complexity pass" and "an ownership
pass." It has one mechanism that reads whatever dimensions `std/`
declares and enforces them all uniformly. Adding a new dimension
means declaring a lattice in `std/` and its binding-site rule.
The compiler carries it generically. Cost of change: one file.

Current dimensions and status:

| Dimension | Declared in | Lattice? | Carried on bindings? | Enforced? |
|-----------|------------|----------|---------------------|-----------|
| Type safety | std/types.dag | N/A (structural) | TypeBinding.resolved | Yes (blocking) |
| Termination | std/termination.dag | BoundedLattice | TypeBinding.provenance + ExprCall.descent_evidence | Partial (424 violations, non-blocking) |
| Ownership | ownership.dag | Not yet | Not yet (separate pass) | Partial (SharedError blocks) |
| Side effects | std/behavioral.dag | Not yet | Not yet | No (declared, not consumed) |
| Purity | (not declared) | — | — | No |
| Idempotence | std/behavioral.dag | Not yet | Not yet | No (declared, not consumed) |
| Space bounds | (not declared) | — | — | No |

The architecture is: **as dimensions move from "separate pass" to
"lattice on bindings," the compiler gets more correct without
getting more complex.** Each dimension dissolved into the binding
mechanism is one fewer analysis pass, one fewer set of heuristics,
one fewer source of reconstruction bugs.

## What falls out

### Zero bugs

If every causal link from source to drain is validated, there are
no bugs. A bug is a broken causal link — a field that doesn't
exist, a branch that isn't handled, a computation that doesn't
terminate, a type that doesn't match. The compiler checks every
link. What it can't check statically, it generates tests for.

Three tiers of the zero-bug guarantee:

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
| Coercion completeness | Fail-closed inhabitant lookup | **partial** (fail-closed where implemented; coercion engine design incomplete) |

**Gating item:** CX gate. Once 424 → 0 and the gate is blocking,
every function that compiles is proven to terminate. This is the
single biggest remaining item in Tier 1.

**Note:** Tier 1 status claims reflect what the compiler enforces
today, not aspirational targets. "DONE" means the diagnostic exists
and blocks compilation. Items marked "partial" have gaps documented
in their design docs.

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

### Tier 3: Verification from structure

In a causal engine, structure and behavior are coupled. The `.dag`
source IS the behavior specification — not a separate oracle. The
compiler has both the intent (declarations) and the output (emitted
code). Verification is: **does the emitted code faithfully translate
the `.dag` evaluation?**

The compiler can generate witness values for any type (all data is
finite, all types have known inhabitants). For any function
`f(x, y)`, the compiler can evaluate it at the `.dag` level for
generated inputs, emit it to each target, execute the emitted code,
and compare results. The `.dag` source is the oracle. No
hand-written tests needed.

This is not "testing" in the traditional sense. It is **emission
verification** — proving that the mechanical translation is
faithful.

| Test level | What it proves | Status |
|------------|---------------|--------|
| L0: Structural tests from data | Coercion mappings are complete and consistent | DONE |
| L1: Pipeline unit tests | Compiler stages produce correct output | DONE (388 tests) |
| L2: Bootstrap self-hosting | Compiler can compile itself | DONE |
| L3: Syntax validity | Emitted code parses in target language | DONE |
| L4: Semantic correctness | Emitted code executes, matches `.dag` evaluation | **not implemented** |
| L5: Cross-language equivalence | Same `.dag` → same behavior in Rust/Python/Go | **not implemented** |
| L6: Exhaustive form coverage | Every structural form compiles to every target | **not implemented** |
| L7: Algebraic law verification | fold/map/filter obey their declared laws | **not implemented** |

**Gating items:** L4 (semantic correctness) is the critical gap.
The compiler can evaluate `.dag` functions directly (closed,
decidable, finite). The emitted code must agree. Until L4 is
gated, "emission is mechanical translation" is unverified.

---

## What else falls out

These are not separate features. They are consequences of the
causal engine being designed correctly.

### Frontend/backend agnosticism

The causal engine validates the causal graph — it does not care
what syntax produced it or what language consumes it. The IR
(Node + Edge, fold/descend/repeat) is the invariant. Anything
that can express basic truths — types, lists, functions, even
structured English — can in principle be ingested. Anything that
can represent the primitives can be emitted to.

This is not a primary goal — it is a side effect of designing the
causal system properly. If the compiler's only job is to validate
causal consistency, and the IR captures all the structure, then
the frontend and backend are just projections of the same graph.
`.dag` syntax is one frontend. Rust/Python/Go are three backends.
The set is open in both directions.

### Omni-emission: one intent graph, many artifacts

A single `.dag` program can describe an entire system — API
server, frontend, database schema, CLI tool, deployment config.
Different subgraphs of the intent emit to different targets.
The emission topology is itself part of the declared intent:

```dag
service OrderAPI via rest::server(lang: Rust, port: 8080) { ... }
service OrderUI  via web::frontend(lang: TypeScript) { ... }
type   OrderSchema via sql::migration(target: Postgres) { ... }
```

The compiler validates the full causal graph across all
artifacts — the Rust API server and the TypeScript frontend
agree on types because they derive from the same declarations.
The compiler owns the glue: serialization contracts, shared
type definitions, API surface consistency. Each artifact is a
projection of the validated intent onto a specific target.

Emission is independent of intent. You declare what the system
does; separately, you declare what artifacts it becomes. The
compiler handles everything in between.

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
/ (direction — start here)
  THESIS.md .............. this file — the goal
  ROADMAP.md ............. current state and work plan
  INVARIANTS.md .......... rules that protect the thesis
  MODELING.md ............ how to extend the language safely

docs/ (project-wide design — read for understanding)
  architecture.md ........ substrate design (Node + Edge)
  algebraic-type-spec.md . type system semantics
  coercion-design.md ..... type coercion algebra (Tier 1, DONE)

src/v2/ (compiler implementation — read when working)
  DESIGN.md .............. compiler design principles
  cx-design.md ........... complexity analysis (Tier 1 gating item)
  cx-computation-model.md  CX core model and evidence system
  cx-violation-triage.md . CX violation snapshot
  ownership-design.md .... ownership proofs (Tier 1 + parallelism)
  compiler-laws.md ....... compiler structural laws
  CM.md .................. concept model gaps
  CM-inventory.md ........ heuristic inventory

src/v2/tests/ (testing — read for verification)
  testing-strategy.md .... generated tests (Tier 3)
```

Every doc has a "Part of" header linking up to this file.
Browse top-down: start at THESIS, drill into ROADMAP for
current state, then into the relevant design doc for details.

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

The causal engine is real when a user can declare their intent:

```dag
type Order { customer: String  amount: Float  status: OrderStatus }
type OrderStatus = Pending | Approved | Declined | Refunded

service OrderService {
  fn create_order(req: CreateOrderRequest) -> Order via rest::post("/orders")
  fn get_order(id: String) -> Order via rest::get("/orders/{id}")
}
```

...and the compiler:
1. **Validates** every causal link — types, fields, transports,
   termination, ownership (Tier 1)
2. **Proves** that no internal operation can fail at runtime (Tier 2)
3. **Generates** tests that verify the declared behavior matches
   actual behavior (Tier 3)
4. **Emits** to any target language as mechanical translation

The only possible failure is external: the REST endpoint doesn't
exist, the network is down, the upstream service violates its
contract. Everything inside the causal graph is proven sound.
