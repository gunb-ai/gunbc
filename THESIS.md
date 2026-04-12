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

### Concept unification

In a closed system, apparently distinct concepts often collapse into
each other. This is not an optimization — it is a structural fact.
When two "different" mechanisms turn out to be the same mechanism
viewed from different angles, maintaining them separately is a dual
representation (INVARIANTS: "No duplicate representations," "No
parallel implementations").

Known unifications:
- **Coercion cost = complexity.** A type coercion is a .dag function.
  Its cost is whatever CX proves, not a separate lattice.
- **Coercion = emission.** Coercion is not a step before emission —
  it IS emission. The compiler reads a target spec and generates
  code. Whether that code is "a Rust struct" or "a SPICE subcircuit"
  or "an HTTP client" is determined by the spec, not by a separate
  coercion engine.
- **Target language spec = transport spec = interpreter runtime.**
  A Rust language spec, a REST transport spec, and the interpreter's
  execution model serve the same role: they declare **what the
  target is** — its primitives, its syntax, its capabilities — and
  the compiler translates mechanically. The emitter doesn't "know
  Rust" or "know REST." It reads the spec and translates.

  This unification has a concrete sustainability consequence: the
  interpreter does not have per-transport handlers. It reads the
  same transport specs as the emitter (`extdeps/transports/`). The
  transport spec says "shell means: construct argv, invoke subprocess,
  map stdout/stderr/exit to output fields." The emitter renders this
  as Rust source code. The interpreter renders this as a direct
  call to one of three platform primitives (process, HTTP, file).
  Adding a new transport (gRPC, WebSocket, etc.) means adding a
  spec in `extdeps/transports/` — zero compiler changes, zero
  emitter changes, zero interpreter changes.

  The same applies to language specs. Adding a new emission target
  (Swift, Kotlin, etc.) means adding a spec in `extdeps/languages/`
  — zero compiler changes. The spec IS the implementation.

  **The sustainability test:** when the system grows by one transport
  or one language, how many files need editing? The answer should
  be 1: the spec file. If it's more, there's a parallel list
  somewhere that will drift and break.

- **Idempotency + cancellation + redundancy = algebraic
  simplification.** These appear to be three distinct concepts:
  - Idempotency: `f ∘ f = f` (doing it twice = doing it once)
  - Cancellation: `f ∘ f⁻¹ = id` (doing and undoing = nothing)
  - Redundancy: `f₁ ∘ ... ∘ fₙ = g` where `cost(g) < cost(f₁∘...∘fₙ)`
  
  They are all instances of **one mechanism**: the compiler knows the
  algebraic laws on operations (group, monoid, lattice, involution)
  and simplifies compositions symbolically. Three right turns = one
  left turn is not a special case — it's the rotation group Z₄.
  `serialize ∘ deserialize = id` is not a special case — it's an
  inverse pair. The compiler has the algebra; simplification falls
  out. See `std/effects.dag` and `std/algebra.dag`.

**The test:** if adding a new concept requires a new mechanism rather
than being an instance of an existing mechanism, investigate whether
the new concept is really distinct. In a closed system, new concepts
should compose from existing ones. A parallel mechanism is evidence
of a missed unification.

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
| Termination | std/termination.dag | BoundedLattice | TypeBinding.provenance + ExprCall.descent_evidence | Partial (421 violations, non-blocking) |
| Coercion | (not a separate dimension — coercion IS emission; CX proves bounds on emission functions) | — | — | Partial (fail-closed where implemented) |
| Ownership | ownership.dag | Not yet | Not yet (separate pass) | Partial (SharedError blocks) |
| Side effects | std/behavioral.dag | Not yet | Not yet | No (declared, not consumed) |
| Purity | (not declared) | — | — | No |
| Idempotence | std/effects.dag | Lattice (derived from EffectShape) | Not yet | No (algebra declared, not consumed) |
| Space bounds | (not declared) | — | — | No |

The architecture is: **as dimensions move from "separate pass" to
"lattice on bindings," the compiler gets more correct without
getting more complex.** Each dimension dissolved into the binding
mechanism is one fewer analysis pass, one fewer set of heuristics,
one fewer source of reconstruction bugs.

### User-defined dimensions

The mechanism is not compiler-internal. If the architecture is
correct, users can declare their own correctness dimensions — the
compiler enforces them with the same machinery it uses for
termination and ownership.

Examples:
- **Security classification** — `Public | Internal | Secret` as a
  lattice. Secret data can't flow to a Public drain without a
  declassifier. Enforced at every binding.
- **Regulatory compliance** — `PHI | NonPHI` for HIPAA. Patient
  data can't flow to non-compliant storage.
- **Financial provenance** — every monetary computation carries
  provenance to its authorization source.

A user declares a lattice, attaches it to their types, and the
compiler enforces it universally. No special tooling. No
annotations. The same non-consensual enforcement that applies to
termination applies to their proprietary model.

**This is the test of the architecture.** If user-defined
dimensions work the same as built-in ones, the mechanism is
general. If they require special compiler support, the mechanism
is incomplete.

Design: [src/v2/dimensions-design.md](src/v2/dimensions-design.md)
— the general mechanism abstracted from CX and ownership.

## Error handling: show the correct code

When the compiler finds a broken causal link, it doesn't just
report the error — it shows the fix. Because the system is closed
and the compiler has full structural knowledge, it knows the finite
set of ways to make the code correct.

A diagnostic is not "error on line 42." It is:
- **What's wrong** — which causal link is broken
- **Why it's wrong** — the structural contradiction
- **How to fix it** — the literal corrected code, emitted to the
  terminal

This falls out of bidirectional emission. If the compiler can emit
`.dag` → Rust, it can emit "corrected `.dag`" → terminal. The
error diagnostic is emission targeted at the developer.

Examples:
- `NonExhaustiveMatch` → show the missing arms with placeholder
  bodies
- `ComplexityUnknown` → show which argument should be the sub-value
  and the corrected call
- `TypeMismatch` → enumerate the concrete options (change the
  branch, change the return type, widen the type)
- `FieldNotFound` → show the available fields, suggest the closest
  match

The compiler knows enough to solve the error, not just report it.
In many cases, only one fix is structurally valid — the compiler
can apply it automatically.

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
| Non-termination | Structural descent proof (CX gate) | **421 violations → 0, then blocking** |
| Non-idempotent workflow | Effect algebra composition (std/effects.dag) | **not started** — algebra declared, compiler consumption not wired |
| Record literal completeness | Missing-field diagnostic | **partial** |
| Coercion completeness | Fail-closed inhabitant lookup; coercion = emission (not a separate mechanism) | **partial** — schema + dispatch + per-language data done; single emitter (Lane C) not started |

**Gating items:** CX gate (421 → 0, then blocking) and emission
completeness (every .dag→target conversion is a declared .dag
function with CX-proven bounds). Coercion is not a separate gate
— it is emission. When the single emitter (Track 13) lands,
coercion completeness is a consequence.

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
| L1: Pipeline unit tests | Compiler stages produce correct output | DONE (393 tests) |
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

### Direct execution (interpreter)

`dag run foo.dag` — compile, validate, execute in one step. The
bounded kernel makes this safe: all programs terminate, all data
is finite, no mutation. A tree-walker over the post-validation IR.

Most users want: validate → run. Emission to Rust/Go/Python is a
**deployment optimization**, not the development workflow. The
interpreter proves that the validated IR is a complete computational
description — emission to other languages is a performance choice.

Service calls (shell, REST, file) execute via the same transport
specs the emitter reads. The interpreter doesn't have per-transport
handlers — it reads the spec and calls one of three platform
primitives (process, HTTP, file). This is the spec unification
in action: one declaration, two consumers (emitter and interpreter),
zero parallel code.

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

### Algebraic simplification (idempotency, cancellation, redundancy)

The compiler knows the algebraic laws on operations. From those
laws, three properties emerge without separate mechanisms:

**Idempotency** — `f ∘ f = f`. An operation whose effect is a
lattice meet on state is idempotent by the algebra. The compiler
derives this from the effect shape (`std/effects.dag`), not from
an annotation. `idempotent: Bool` on `OperationBehavior` dissolves.

- PUT /secrets/{name} → Map upsert → lattice meet → **idempotent**
- DELETE /instances/{id} → Map delete → lattice meet with ⊥ → **idempotent**
- POST /logs (no key) → List append → monoid → **not idempotent**

For workflows (infrastructure bringup, CI, deployment), the
compiler composes effects. A workflow is idempotent iff all its
operations have lattice effects. If one breaks the chain, the
compiler shows which one and why.

**Cancellation** — `f ∘ f⁻¹ = id`. Operations that are declared
inverses cancel. The compiler detects this from the algebraic
structure (group inverse, involution). Compile error: "these two
operations cancel — the result is equivalent to doing nothing."

**Redundant work** — `f₁ ∘ ... ∘ fₙ = g` where `cost(g) <
cost(f₁∘...∘fₙ)`. The compiler simplifies the composition using
algebraic laws and compares costs. If a cheaper equivalent exists,
compile error: "this sequence is equivalent to X, which costs less."

All three use the same mechanism: symbolic composition of operations
under their declared algebraic laws, followed by simplification.
The GPS analogy: three right turns = one left turn. The compiler
knows the rotation group and simplifies.

Three verification layers (same for all three):
1. **Compile time:** prove the property from algebraic laws
2. **Generated test:** verify the law holds against reality
   (e.g., `f(f(x)) == f(x)` for idempotency)
3. **Runtime receipt:** log when operations are no-ops

**Case study: we commit this bug against ourselves.** The
`merge_envs` function in the compiler (2026-04-12) was doing
`merge(a, a, a)` where all inputs were the same InternTable
(threaded from a single upstream authority). By idempotency
of merge, the result equals any input — but the compiler
didn't enforce this, so the runtime spent ~20 seconds per
self-compile iterating and rebuilding a table identical to
its inputs. A 6-line fix (read the first input instead of
merging) produced a 68× speedup on the reconcile stage.

This is KF-2 we're committing against ourselves. Every such
perf bug we hit is advance payment on KF-2's priority: if
the compiler enforced algebraic simplification at compile
time, merge_envs-class bugs would be compile errors, not
latent hot spots. See [docs/perf/clone-elimination.md](docs/perf/clone-elimination.md)
for the full case and the rules it teaches.

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

## What .dag catches that normal compilers don't

These are concrete examples of bugs and inefficiencies that .dag
rejects at compile time. A normal compiler (Rust, Go, Python, etc.)
would compile every one of these without complaint. .dag catches
them because the closed system gives the compiler enough algebraic
structure to prove they are wrong.

### Structural bugs (impossible to write)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| Non-terminating recursion (`check_type(resolve(name))` where resolved type is recursive) | Compiles fine. Stack overflow at runtime. Real bug in TypeScript, Rust, Haskell compilers. | CX demands structural descent proof. `resolve(name)` is a lookup, not descent — `SubValueUnknown`. Rejected. |
| Accidentally quadratic (`process(items)` inside `items |> map(...)`) | Compiles fine. O(n²) at runtime. | CX tracks cost composition. `fold(n, fold(n, ...))` = O(n²). If a cheaper equivalent exists (single fold), compile error. |
| Infinite mutual recursion (`f(n) → g(n) → f(n)`) | Compiles fine. Stack overflow at runtime. | CX analyzes SCCs. Neither call shows descent. Both rejected. |
| Recursion on sibling instead of child (`process(node)` instead of `process(child)`) | Compiles fine. Infinite loop at runtime. | CX sees `PreservedValue` (same node), not `StrictSubValue`. Rejected. |
| Work-list that grows unboundedly | Compiles fine. OOM or infinite loop at runtime. | `repeat(N)` requires explicit bound. No unbounded iteration primitive exists. |

### Redundant work (wasteful but compiles)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| `list |> reverse |> reverse` | Compiles fine. Wastes O(n) work. | `reverse` is an involution (`f ∘ f = id`). Composition simplifies to identity. Compile error: "equivalent to doing nothing." |
| `data |> serialize |> deserialize` | Compiles fine. Wastes serialization cost. | Declared inverse pair. Composition = identity. |
| `map(f) |> map(g)` (two passes) | Compiles fine. Two traversals where one suffices. | Map fusion law: `map(f) ∘ map(g) = map(f ∘ g)`. One pass is cheaper. |
| Clone a value used only once | Compiles fine (Rust requires it in some contexts). Wastes allocation + copy. | Ownership analysis: fan-out = 1. Last use can move. Clone is redundant. |
| Infrastructure bringup that re-provisions already-running services | Compiles fine. Wastes API calls and time. | Effect algebra: all operations are lattice meets (upsert). Workflow is idempotent — re-running is benign but the compiler can flag the redundancy. |

### Effect safety (silent bugs at runtime)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| Non-idempotent workflow marked as safe to retry | Compiles fine. Duplicates data on retry. | Effect algebra derives idempotency from effect shape. `POST /logs` (List append) is not idempotent. Compiler shows which operation breaks it. |
| Write-then-overwrite (dead effect) | Compiles fine. First write is wasted. | Effect composition: `upsert(k, v1) ∘ upsert(k, v2) = upsert(k, v2)`. First effect is subsumed. |
| `create_resource()` in a retry loop | Compiles fine. Creates duplicates on retry. | `POST` without key = `CreateEffect` = not idempotent. Compile error inside `repeat()` or retry context. |

### Complexity violations (wrong algorithm)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| O(n²) where O(n) suffices | Compiles fine. Slow at runtime. | CX proves cost. Algebraic simplification finds cheaper equivalent. KF-2 rejects. |
| Unbounded recursion depth | Compiles fine. Stack overflow at runtime on deep inputs. | CX proves depth bound from structural descent. No bound = rejected. |
| `fib(n-1) + fib(n-2)` (O(2ⁿ)) | Compiles fine. Exponential at runtime. | CX branching guard: multiple recursive calls with arithmetic descent = exponential. Rejected unless memoized or reformulated. |

Concrete `.dag` code examples with compiler errors:
[docs/error-examples.md](docs/error-examples.md) — serves as TDD
targets for the compiler. Each example is a test case: the .dag
code should compile today, and the error message is the acceptance
criterion for when the feature lands.

### The common pattern

Every row in every table above is the same mechanism: the compiler
has the algebraic structure (descent proofs, effect shapes, cost
algebra, inverse declarations), composes operations symbolically,
and checks whether the composition satisfies the required property.
No special-case analysis. No lint rules. No opt-in annotations.
The algebra does the work.

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
  error-examples.md ...... concrete .dag code + expected errors (TDD targets)

src/v2/ (compiler implementation — read when working)
  DESIGN.md .............. compiler design principles
  dimensions-design.md ... general correctness dimension mechanism
  cx-design.md ........... complexity (first dimension instance)
  cx-computation-model.md  CX core model and evidence system
  cx-violation-triage.md . CX violation snapshot
  ownership-design.md .... ownership (second dimension instance)
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
  CX gate:                 421 violations remaining (non-blocking)
  Coercion (= emission):   schema + dispatch + data done; single emitter (Lane C) not started
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
