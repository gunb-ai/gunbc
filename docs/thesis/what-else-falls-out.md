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

A single `.dag` program can describe an entire system —
frontend client, middleware, backend service, database schema,
simulation netlist, API documentation, infrastructure config —
with the compiler projecting different subgraphs onto different
targets. The **coherence across targets is structural, not
checked**: every artifact is a walk over the same Node tree
through a different language spec, so drift between layers is
structurally impossible.

**A concrete full-stack example.** Consider an order-management
workflow:

```dag
// Shared domain — declared once, projected everywhere.
type Money = Field<Word64>              // inhabits Field → arithmetic free
type OrderItem { sku: String; qty: Int; price: Money }
type Order {
  customer_id: CustomerId
  items: List<OrderItem>
  total: Money
  status: OrderStatus
}
type OrderStatus = Pending | Confirmed | Shipped | Delivered | Cancelled

// The workflow — a composition over shared types.
workflow create_order(draft: Order) -> OrderId {
  let validated = validate_items(draft.items)
  let stock_ok  = check_inventory(validated)
  let payment   = charge_payment(draft.customer_id, draft.total)
  let record    = insert_order(validated, payment)
  let _notify   = notify_customer(draft.customer_id, record)
  record.id
}

// Projection specs — bind subgraphs to target artifacts.
service OrderAPI via rest::server(lang: Rust, path: "/orders") {
  operation POST create_order
}

service OrderUI via web::frontend(lang: TypeScript, framework: React) {
  form_for  Order
  submit_to OrderAPI.create_order
}

persistence OrderRecord via sql::postgres(table: "orders") {
  schema_from Order
  constraints status
  operations  insert_order, select_by_id, update_status
}
```

From this one source, the compiler projects every layer of a
real application, coherent by construction:

| Layer | Target | Comes from |
|---|---|---|
| **DB migration** | Postgres DDL | `persistence OrderRecord` walks `Order`'s Node tree. `CREATE TABLE orders (...)` with a `CHECK (status IN (...))` constraint derived from the `OrderStatus` Disj variants. |
| **Backend struct + handler** | Rust | `struct Order { ... }` + `async fn create_order_handler(Json(draft): Json<Order>) -> ...` where the body is emission of the workflow's L1 behaviors. |
| **Backend service logic** | Rust | Each workflow step emitted from its own function body in `.dag`. Rust-specific error handling, `Rc` wrapping, and async insertions are projection-rule consequences of the Rust language spec. |
| **Client-side API binding** | TypeScript | `async function createOrder(draft: Order): Promise<OrderId> { ... }` — emitted from the `service OrderAPI` declaration by walking the REST transport spec with the TypeScript language spec. |
| **Client-side type definitions** | TypeScript | `interface Order { ... }` + `type OrderStatus = 'Pending' \| 'Confirmed' \| ...` — same Node tree, TypeScript projection rules (camelCase field rewrite, string-union for unit-variant Disj, `number` for `Money`'s Word64 carrier). |
| **Client-side form component** | React | `<form>` with fields for each `Order` child, dynamic list for `items`, dropdown for `status` — walked from `form_for Order` through the React spec's "Conj → form fields, Disj-of-units → dropdown, Cardinality<T> → dynamic list" rules. |

**Coherence is structural, not checked.** If you edit the `.dag`
source:

- Add `delivery_address: Address` to `Order` → the form gets a
  new input, the DB gets a new column, the Rust struct gets a new
  field, the TypeScript interface gets a new field, the API
  client serializes it, the migration script includes `ALTER
  TABLE`.
- Rename `status` to `state` → all six layers use `state`.
- Add a `Refunded` variant to `OrderStatus` → the dropdown has a
  new option, the Postgres `CHECK` constraint includes it, the
  Rust enum has a new variant, the TypeScript union has a new
  string literal, and any existing `match` on status that didn't
  handle `Refunded` is an exhaustiveness error at compile time.

**You cannot have drift between these layers** because they are
not separate artifacts that need synchronization. They are
projections of the same Node tree — same declarations, walked
through different language specs. Drift is not checked; it is
structurally impossible. Traditional full-stack projects have
"is the frontend interface in sync with the backend DTO in sync
with the DB schema?" as a recurring question with no structural
answer. In `.dag`, the question is dissolved: there is only one
`Order`, and asking if the frontend's Order matches the
backend's Order is like asking if `7` equals `7`.

**Targets are declarations, not compiler features.** The Rust,
TypeScript, React, Postgres, and REST specs in the example above
are declarations in `dsl/extdeps/languages/` and
`dsl/extdeps/transports/`. Each spec is itself a Node-tree
composition in the same substrate, declaring: what primitive
shapes the target has, what its syntax is, how each connective
in the type substrate projects onto target constructs, and how
service/transport bindings map to target API calls. Adding a new
target — say, Swift for iOS clients — means writing a Swift
language spec in `dsl/extdeps/languages/swift.dag`. **Zero
compiler changes, zero emitter changes, zero workflow changes,
zero risk of drift into existing targets.** The spec IS the
implementation.

**Two shapes of omni-emission: Shape A (compiler targets) vs
Shape B (user programs).** This is a load-bearing distinction per
`ROADMAP.md` §Track 16 and should not be blurred. The two shapes
have different mechanisms, different cost structures, and
different scope in the compiler core.

**Shape A — compiler language targets.** Programming languages
that execute the full computational semantics of `.dag`. The
compiler reads a language spec from `dsl/extdeps/languages/` and
emits target source code via the single emitter. Examples: Rust,
Python, Go, TypeScript, Swift, and potentially hardware
description languages (Verilog, VHDL, Chisel) where the target
executes the compiled program.

**Shape A is the target architecture, not the current reality.**
At the time of writing, the LanguageSpec mechanism is partial:
materialization strategy, sharing × serialization coupling, and
type-decoration selection are not fully modeled in LanguageSpec
and still leak into per-target emitter code (see `src/v1/*.dag`
emit phases for the current state). The thesis commitment is to
the end state — "adding a new Shape A target costs one spec file,
zero compiler changes" — and the work between M1 and M2 is
closing the gap between what LanguageSpec currently covers and
what it needs to cover to make that claim fully banked. Treat
"one spec per target" as the architectural target, not as an
already-achieved property.

**Shape B — user-program artifact generation.** Non-programming-
language artifacts that are OUTPUTS of `.dag` programs, not
compiled-to by the compiler. A `.dag` workflow walks a typed value
(a `Workflow`, a `Circuit`, a `Manifest`, an `Understanding`) and
emits strings via `concat`/`fold`/`match` — standard user-code
operations. The compiler emits the `.dag` PROGRAM to a Shape A
target (Rust, Python, Go); that program's OUTPUT is the Shape B
artifact. Examples: YAML configs, Terraform HCL, Kubernetes
manifests, CloudFormation templates, CI pipeline YAML (GitHub
Actions, GitLab, Jenkinsfile), SPICE netlists, natural-language
documentation, API reference material, SQL schemas, JSON Schema,
OpenAPI specs. Shape B targets are authored as `.dag` programs
that consume typed workflow values and produce the target
artifact as a string. Adding a new Shape B target costs one
`.dag` program that walks the appropriate typed value and emits
the target format — zero compiler changes, zero emitter changes.

**Why the distinction matters.** Per Track 16, treating YAML /
Terraform / SPICE / natural language as compiler render targets
would be a category error: it would grow the compiler core for
concerns that belong in user code. The compiler's job is to
emit executable code for programming languages; everything else
is a user program that runs in a Shape A language and produces
the artifact. This keeps the compiler core small and bounded to
"things that run computation."

**Both shapes produce coherent artifacts from the same source.**
Coherence-by-construction is preserved in both cases because both
derive from the same `.dag` declarations. A `create_order`
workflow might simultaneously produce:

- Rust backend (Shape A — compiler emits Rust directly).
- TypeScript frontend (Shape A — compiler emits TypeScript
  directly).
- Postgres schema migration SQL (Shape B — a `.dag` program
  walks the Order type and emits `CREATE TABLE` string via
  fold/match).
- Terraform infrastructure module (Shape B — a `.dag` program
  walks the service deployment spec and emits HCL strings).
- English API documentation (Shape B — a `.dag` program walks
  the service declarations and emits markdown strings).
- SPICE netlist for any analog circuit declarations (Shape B).

The `.dag` source is the single source of truth for all six
outputs; drift is structurally impossible because all six are
projections (direct or indirect) of the same Node tree.

**Cost scaling differs between the two shapes:**

- **Shape A:** cost of adding a new target = one language spec
  in `dsl/extdeps/languages/`. Applies to every workflow
  automatically. `O(1)` per new target.
- **Shape B:** cost of adding a new target = one `.dag` program
  that walks the appropriate typed value and emits the target
  format. Typically `~50-200 lines` of .dag per artifact class,
  reusable across workflows that share the same input type.
  `O(types × artifact classes)`, but the per-entry cost is small
  and the programs are themselves subject to all the usual
  correctness dimensions.

**The 1:1 effort property still holds**, with the two-shape
mechanism:

1. The user's declarations (types, workflows, services) are
   written once.
2. Shape A targets get compiler projection automatically.
3. Shape B targets get user-program projection, but the user
   programs are themselves reusable across workflows and are
   written in `.dag` (so they inherit all the correctness
   guarantees).

Cost scales with the number of conceptually distinct artifact
classes, not with the number of workflows × artifacts. Editing
the `.dag` source still reflows consistently across all six
output classes because both the compiler and the Shape B user
programs derive from the same typed declarations.

**The rule that makes Shape A cheap: treat every programming-
language target as an extdep.** The compiler does not know Rust,
or TypeScript, or Go natively. It reads a language spec from
`dsl/extdeps/languages/`, and the spec declares — as ordinary
`.dag` compositional modeling — how each connective in the type
substrate and each L1 behavior in the computation substrate
projects onto the target's constructs. Adding a new Shape A
target is writing a new spec against whatever the target
language provides. The spec is reusable across every workflow;
the workflows are reusable across every spec. For Shape A,
**N workflows × M language targets is handled by N + M
declarations, not N × M handwritten integrations.**

**Shape B uses the same mechanism at the user-code level.** A
`.dag` program that walks an `Order` value and emits Postgres
DDL is itself a reusable artifact — the same program works for
any record type that needs schema generation. Shape B emitters
are libraries, not compiler features. Writing a SPICE-netlist
emitter is writing a `.dag` program once; any circuit declaration
can feed into it.

**Cost-scaling consequence.** Adding a Shape A target is
`O(1)` — one language spec. Adding a Shape B target is
`O(1)` per artifact class — one `.dag` emitter program. Neither
is `O(N × M)` across workflows × targets. A team using `.dag`
omni-emission pays for:

1. Their workflow declarations (the conceptual content of their
   system).
2. Language specs for each target they want (reusable across
   workflows, written once).

They do **not** pay for:

- Synchronizing separate artifact codebases (frontend, backend,
  DB migration, docs).
- Maintaining API contracts between layers.
- Writing parsers, serializers, or type mappers for cross-layer
  communication.
- Keeping documentation in sync with implementation.
- Onboarding developers to N different toolchains — there is one.
- Adding a new target platform (cost = one language spec, once).

This is the 1:1 effort property applied to full-stack
development: effort scales with the system's conceptual content,
not with the number of layers, languages, or target environments
the system projects onto.

**Why this works (connection to §"Epistemic stacking").** The
coherence-by-construction and the cost-scaling properties both
rest on the same foundation: *every artifact emission is a walk
over the same Node tree.* The walk bottoms out at primitives —
the language spec's atomic realizations for each connective —
which is where the epistemic chain hands off to the target world.
Because the walk is deterministic and the Node tree is the
single source of truth, two projections cannot disagree about a
shared type. Because the walk's depth is proportional to
conceptual structure rather than consumer count, adding a new
target does not require rewriting existing ones. This is the
§"Epistemic stacking" substrate test applied to emission rather
than to type inhabitance.

**Why this works (connection to §"The substrate").** The
substrate has to host not just the domain types (`Order`,
`OrderStatus`, `Money`), but also the workflow declarations
(compositions of L1 behaviors over those types) AND the language
specs themselves (Node-tree declarations that describe target-
world shapes and projection rules). The substrate test "can it
host `dsl/std/algebra.dag` as-is?" is precisely what unlocks
this: the same connective set that holds `Monoid<T>` also holds
`service OrderAPI via rest::server { ... }`, the same
`Instantiation` connective that binds `T := Word64` for `Int64`
via type parameterization also has a Conj-with-inhabits-tag
counterpart for value construction like `transport shell
{ argv: [...] }` and `service OrderAPI via rest::server(lang:
Rust)`, and the same `inhabits` edge that connects `Int` to
`OrderedRing` connects `create_order` to
the workflow-projection rules in the Rust spec. **One substrate,
one walk, arbitrarily many targets.**

**Emission is independent of intent.** You declare what the
system does; separately, you declare what artifacts it becomes.
The compiler handles everything in between. And because both the
system declaration and the artifact declaration are compositional
Node trees in the same substrate, they evolve together
automatically: adding a workflow instantly becomes available to
every bound target; adding a target instantly applies to every
existing workflow; refactoring the workflow cascades to every
projection; refactoring a projection rule cascades to every
workflow that uses that target.

### Automatic parallelism (structural, not scheduled)

Parallelism in .dag is not scheduled — it is structural. The
program IS a dependency graph (see "The core abstraction" above).
Independent subexpressions have no ordering constraint. The
compiler reads the graph and emits concurrent execution for any
target that supports it.

Three specific patterns emerge from the bounded iteration model:

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

But these are specific instances of the general principle: **any
two expressions without a data dependency can execute concurrently.**
The compiler doesn't need special "parallel fold" or "parallel
descend" support — it needs to read the dependency graph and emit
independent subgraphs to concurrent execution units.

**What the compiler needs to know:**
- Provenance on bindings (Stream A) — which values depend on which
- Ownership proof (Stream B) — whether the accumulator is aliased
- CX gate closed — that the operation terminates
- Effect shape (std/effects.dag) — whether the operation has side
  effects that constrain ordering

**What the compiler does NOT need:**
- Explicit parallelism annotations (async, spawn, par_iter)
- A separate scheduling algorithm (wave computation, task graphs)
- Runtime thread pool configuration

These are all derivable from the dependency graph + the target's
concurrency model. The emitter reads the target spec to decide
HOW to express concurrency (OS threads, async tasks, distributed
workers, CI job dependencies). The .dag source says WHAT depends
on WHAT. The rest follows.

**The sustainability test:** adding a new concurrency target
(e.g., "emit as GitHub Actions jobs" or "emit as Kubernetes pods")
should require adding a target spec, not changing the source
program. The dependencies are the same regardless of whether
they execute on threads, CI runners, or cloud functions.

### Automatic memoization

A pure function with known cost and no side effects can be
memoized by the emitter. The compiler already knows:
- Whether the function is pure (no service calls, no mutation)
- Its complexity bound (CX)
- Its argument types (hashable or not)

Once these facts flow through bindings, the emitter can insert
memoization for expensive pure functions automatically.

### Incremental cross-run execution

The same purity + bounded execution + determinism + dependency-
graph commitments that enable within-run memoization (above) also
enable **cross-run** caching. A pure subexpression with hashable
inputs has a content-addressable result; across runs, the
compiler/runner can skip re-executing any subgraph whose inputs
hash to the same value as a prior run.

What the compiler/runner already knows:
- Each Node's structural identity (content hash from declaration)
- Each binding's transitive dependencies (Stream A provenance)
- Each operation's purity and CX (bounded execution → bounded
  per-result size; total cache store retention is a separate
  policy fact, not derived from CX)

Two consequences fall out:

- **Incremental execution** — when source changes between runs,
  only the dependent subtrees re-execute. The dependency graph
  already exists (per §"Automatic parallelism"); cross-run
  change-propagation is the same graph walked across two
  run-states.

- **Content-hash caching** — deterministic execution +
  content-addressable inputs means a pure expression's result
  caches by `hash(structural_form, input_hashes)` as the lookup
  key. Cache invalidation is precise (same structural inputs ⇒
  same result, by purity + determinism), not heuristic ("anything
  that depends on this file"). Hash is the lookup mechanism;
  semantic equivalence is structural equality of inputs, not
  hash equality (collisions are an implementation concern handled
  at lookup verification, not a load-bearing thesis claim).

Both are consequences of the existing dependency-graph + purity +
determinism substrate. Cross-run scope is the only thing that
changes from §"Automatic memoization".

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
latent hot spots. See [docs/perf/clone-elimination.md](../perf/clone-elimination.md)
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
