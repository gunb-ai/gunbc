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

## The core abstraction: dependency modeling

A `.dag` program is a dependency graph. Every binding declares
what it depends on. Every function declares its inputs. Every
`fold`/`descend`/`repeat` declares its data source and its
accumulator. **The program IS the dependency graph.** There is
no separate "scheduler" or "orchestrator" that reads the program
and decides execution order — the dependency structure IS the
execution order.

This has a fundamental consequence: **parallelism is not a
feature. It is the default.** Two bindings with no data
dependency between them are independent. The compiler can see
this from the dependency graph — no annotation, no `async`, no
thread pool configuration. Independent computations execute
concurrently because nothing constrains them to be sequential.
Sequential execution is what requires justification (a data
dependency), not parallel execution.

This is the same insight as MapReduce: the programmer declares
data dependencies (map function, reduce function, key partitioning),
and the framework handles distribution. But MapReduce is a
library that reads these declarations at runtime. `.dag` is a
language where the compiler reads them at compile time. The
compiler knows the full dependency graph, the cost of each node
(CX), whether the combining function is associative (algebra
inhabitant), and whether any node has side effects (effect shape).
It can emit the optimal schedule for any target — single-threaded,
multi-threaded, distributed, or pipelined — from the same source.

**Using .dag as a sequential scripting language is technically
possible but misses the point.** The language exists to model
dependencies. A program that threads all state through a single
sequential chain (do A, then B, then C, then D) is a valid
program, but it's an impoverished use of the abstraction. The
equivalent of writing MapReduce code that puts everything in one
partition.

The CI pipeline is the canonical example. Writing 8 gates as a
sequential list of shell commands is using .dag as a scripting
language. Writing the same 8 gates with their actual data
dependencies (build → compile → [check_dsl, check_diag, check_fresh,
check_perf]) is using .dag as dependency modeling software. The
compiler reads the graph, sees that `lint` and `tests` and
`check_l1` have no dependency on `build`, and can execute them
concurrently. No wave computation algorithm needed — the
dependency graph IS the wave computation.

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

### Structural decompression

In a closed system we define, every categorical classification
(coproduct / enum / sum type) is a compression artifact — a
t-shirt size laid over a richer coordinate space.

A t-shirt size (S/M/L/XL) compresses a continuous space (chest ×
length × shoulder) into a discrete tag because garment manufacturers
don't own both sides of the transaction: they don't know who buys
the shirts, the communication cost of full dimensions exceeds the
benefit, and discrete sizing enables economies of scale. The tags
are lossy — two shirts labeled "M" can differ by inches — and the
lossiness is acceptable because the downstream consumer (a shopper)
can't measure themselves precisely anyway.

**In a closed generated-code system, every economic reason for
categorical compression disappears.** We are the only manufacturer
(the compiler generates the code) and the only consumer (the
compiler reads its own model). Communication cost is zero — the
model generates the access code. The consumer (generated code) can
read any coordinate precisely. So there is no reason to compress.

**Structural decompression** is the practice of replacing categorical
tags with their underlying coordinate-level facts. The name encodes
the insight: the coproduct is COMPRESSED structure. Decompressing it
recovers the facts that were compressed away.

The test: when encountering a coproduct, ask **"what's the
coordinate space this is a tag system over?"** If you can name the
space, the coproduct decompresses into the coordinate-level model.
If you genuinely can't — the variants truly are an unordered set
with no common coordinates — it's a terminal at the user-input
boundary (an identifier, a literal, a source span — input from
outside the closed world that we don't define).

Four decompression patterns, depending on what kind of compression
is hiding:

1. **Fact placement** — a coproduct mixes concerns from different
   locations in the substrate. Decompress by placing each fact
   where its consumer naturally reads it.
   *Example:* `InferredNode = Resolved | CompilerError | TypeVariable`
   → type goes on Port, errors go in diagnostic table, unification
   is algorithm-internal.

2. **Variant-is-data** — variants differ in WHAT (which value) but
   not in HOW (same structure). Compress to one structural shape
   with the variation as data in a table.
   *Example:* keywords → single `Keyword` shape with identity in
   Token.text. TransformRule variants → rule-table entries.

3. **Algebraic form** — variants trace to introduction or
   elimination forms of algebraic structures already declared in
   `std/`. The algebra declarations generate the dispatch.
   *Example:* `Construct` = Product introduction.
   `FieldAccess` = Product elimination. `ListBuild` = FreeMonoid
   introduction. All from `std/algebra.dag`.

4. **Dimensional** — a flat N-variant coproduct hides an
   M-dimensional record. Replace the flat enum with a record
   whose fields are the dimensions.
   *Example:* 6 delimiter tokens → `Delimiter { shape: BracketShape,
   side: Side }` where `BracketShape = Curly | Round | Square` and
   `Side = Open | Close`.

**Priority: decompress single-consumer coproducts first.** In a
generated-code system, decompression cost is constant (model
editing) while deferral cost grows monotonically (new consumers
bolt on because the coproduct exists). `ExprData` started as the
parser's discriminator — one consumer. It accreted to 22 variants
with 665 match arms across 7 consumers because each new consumer
was easier to bolt on than to redesign. "Single consumer" is a
temporal accident, not a design property.

**Lossy compression is a correctness bug.** When downstream code
reconstructs information that a coproduct compressed away, the
disease is active. v2's `TypeBinding = {name, resolved}` discarded
provenance during inference. `complexity.dag` spent 5,000 lines
rebuilding it from heuristics. Structural decompression isn't just
cleaner — it's required when the compressed-away facts are needed
downstream.

### Why decompression always works in a closed system

A coproduct is a compressed reference to a reality the modeler
has already defined. In an open system, the reality may not be
fully owned by the modeler, so the compression can be irreducible
— natural kinds (species, elements) resist decomposition because
the underlying space is discovered, not defined. The modeler
is a consumer of a reality they don't control.

In a closed system we define, we own the reality by construction.
Every coproduct is a compressed reference to something we wrote
down somewhere — either elsewhere in the substrate, in data tables,
in `std/` vocabulary, or in coordinate spaces we can name.
Dissolution is replacing the compressed reference with a pointer
to the richer source.

The four decompression patterns are four ways of finding the
richer source:
1. **Fact placement** → richer source elsewhere in the DAG
2. **Variant-is-data** → richer source in a data table
3. **Algebraic form** → richer source in `std/` declarations
4. **Dimensional** → richer source in a coordinate space

The only genuinely irreducible coproducts are those whose richer
source is outside the system — user-chosen names, literal values,
source spans — where the user is the author of the reality and
we are a consumer. Everything else dissolves.

This means the real design work is not writing types — it is
writing richer sources. Every `std/` addition (Terminal, function
space, intro/elim declarations, lens algebras) is a richer source
that enables dissolution of compressed coproducts elsewhere. The
additions are not "new types to build" — they are "sources to
point at so the rest of the system stops pointing at compressions."

**Corollary:** when the agent finds a coproduct it thinks can't
dissolve, exactly two possibilities exist: (a) it's terminal input
from the user (outside the closed world), or (b) the richer source
hasn't been written yet. Case (a) is the stopping rule. Case (b)
is unfinished work pretending to be a terminal.

**The sustainability test:** when the system grows by one concept,
how many files need editing? If the answer is more than the
declaration file, there's a categorical compression somewhere that
is forcing consumers to enumerate variants instead of reading
coordinates. Find it. Decompress it.

### Epistemic stacking: every concept grounds in primitives

Everything in the system is a node in an ontological DAG rooted at
a minimal set of primitives. No concept is defined in isolation;
every concept is a composition, and the composition is traceable
all the way down to primitives whose only role is to be
irreducible. "Dependency modeling software" is therefore true in
two senses at once: the program is a runtime data-flow graph *and*
the system itself is a conceptual derivation graph. Every
declaration's meaning comes from walking its composition back to
primitives.

The root primitives are the minimal set that every other concept
composes from. Per `MODELING.md` §"Foundational primitive:
truth-valued structure" and §M9, the roots are:

- **Classical logic** — the truth-valued carrier. `Bit = Disj {
  On: Atom; Off: Atom }`, the two-element Disj with Atom variants,
  declared in `dsl/std/logic.dag`. This is the foundational
  carrier Word8/16/32/64 build from, which Int/Nat/Float build
  from, which everything user-facing eventually walks down to.
- **Conj (product constructor)** — labeled product, logical AND,
  the structural primitive for "these children present together."
- **Disj (coproduct constructor)** — labeled coproduct, logical
  OR, the structural primitive for "exactly one of these
  alternatives." Bit uses this over two Atoms; everything else
  follows.

Those three (Classical Bit + Conj + Disj) are the actual roots.
Every other named structure in the project — `Word64`, `Int`,
`String`, `List<T>`, `Monoid<T>`, `Ring<T>`, `BooleanAlgebra`,
`FreeMonoid<T>`, `Order`, `CIWorkflow<Step>` — is a Layer 1 (or
higher) composition on top of these roots. The algebras in
`dsl/std/algebra.dag` are NOT roots; they are the first-floor
library of compositional structures user types inhabit.

Examples of how Layer 1 composes from the roots:

- **Magma<T>** is a Conj with one `op` child (an Arrow with three
  T-references). Closure as a Conj-of-Arrow pattern.
- **Monoid<T>** is Magma plus an `identity` child (an Atom
  reference to the T parameter). Identity as a Conj-of-Arrow-plus-
  Atom pattern.
- **Ring<T>** is Monoid-shaped with more labeled children (`add`,
  `mul`, `zero`, `one`, `negate`). All Conj-of-Arrow-plus-Atom.
- **BooleanAlgebra** is a Conj over Bit — meet/join/complement as
  Arrow children whose inputs are Bit-typed. `Bool = Classical`,
  not a new primitive.
- **FreeMonoid<T>** is the recursive Disj `Empty | Cons { head:
  T; tail: FreeMonoid<T> }` — the canonical "generate from
  nothing" shape that roots strings, lists, syntax trees. Built
  from Disj + Conj + Atom references. Not a primitive.

The chain from roots to user code walks: roots (Classical + Conj
+ Disj) → Layer 1 algebras (Magma/Monoid/Ring/Lattice/FreeMonoid/
BooleanAlgebra in `algebra.dag`) → concrete types (Int = Instantiation
of OrderedRing<Word64>, String = Instantiation of FreeMonoid<Char>,
etc. in `types.dag`) → user types and workflows. Every layer
composes the layer below.

The thesis aligns with `MODELING.md` on this point: the
composition layer (Node, children, edges) is the substrate; the
foundation (logical algebra / Classical) is the interpretation
that gives the substrate meaning. The algebras are built on top,
not at the root.

Concrete types attach to this chain by **inhabitance**:

```
Bool     inhabits BooleanAlgebra
Nat      inhabits CommutativeSemiring
Int      inhabits OrderedRing
String   inhabits FreeMonoid<Char>
List<T>  inhabits FreeMonoid<T>
Set<A>   inhabits BooleanAlgebra<A>    (via A -> Bool encoding)
Map<K,V> inhabits PartialFunction<K,V>
```

`Int.add` is not declared anywhere as a primitive — it falls out
because Int walks to OrderedRing and OrderedRing has an `add` field.
Adding `Int256 = OrderedRing<Word256>` costs one declaration and
inherits all the arithmetic. The cost-of-change test in its purest
form.

**Load-bearing for codegen.** Emission is not a separate act from
concept construction — it is the same walk in the opposite
direction. To emit code for any expression, the compiler walks the
expression's concept chain down to primitives, and the primitives
are the points where it hands off to target-language realizations.
When a concept is *grounded* (has a full chain to primitives) the
emission is mechanical. When a concept is *ungrounded* (an opaque
intrinsic, a bolt-on annotation, a string label, a runtime-native
helper) the emitter has nothing to walk and must contain a special
case. Every special case in the emitter is evidence of an
ungrounded concept upstream. **The epistemic chain IS the emission
algorithm; breaking the chain breaks codegen.**

The same walk projects to any target and to any domain. A user who
declares `service shell.Exec { operation Run { input {...} output
{...} transport shell { argv: [...] } } }` is composing
declarations (service, operation, input, output, transport) that
themselves ground in the same algebraic primitives Int does. The
compiler's job is identical: walk the chain, hand off to target
realizations at the primitive boundary. There is no "math modeling
path" and a separate "domain modeling path" — they share one
substrate and one emission walk.

**Domain compositional models encoding other compositional models
are the direct consequence.** A user can declare their own
meta-model — `type CIWorkflow<Step>`, `type ControlPlane<Resource>`,
`type Understanding<System> { behaviors: List<Behavior>, ... }` —
and instantiations of that meta-model project onto code the same
way `Int` does. The substrate does not distinguish math compositions
from domain compositions; **both compose from the same three
substrate roots (Classical + Conj + Disj)** via the ordinary
composition mechanism. What's "primitive" in gunbc is structural,
not subjective: a primitive is one of the three substrate roots
(`MODELING.md`'s foundational primitives), or a user-input boundary
Atom (identifiers, literals, source spans from outside the closed
world). Everything else, math or domain, is a composition. This
is the property that lets gunbc target dependency surface area of
arbitrary depth: the deeper the domain's compositional structure,
the more load the epistemic chain carries, and the more leverage
the user gets per declaration.

**Corollary: no concept is opaque.** When a declaration resists
decomposition, exactly two cases are possible:

1. It bottoms out at a genuine primitive — a logical/algebraic root,
   or a user-input boundary (identifiers, literals, source spans
   from outside the closed world).
2. It hasn't been composed yet; its composition is unfinished work.

There is no third case. "Opaque intrinsic," "runtime-native helper,"
"bolt-on analyzer" are all case 2 — unfinished composition
masquerading as a primitive. The modeling discipline rejects them
not for style but because they break the epistemic chain codegen
walks.

**The substrate test.** For any candidate Declaration shape in the
compiler, the check is: *can it host `dsl/std/algebra.dag` as-is?*
— `Magma<T>`, `Monoid<T>`, `Ring<T>`, the parameterization over
`T`, the inhabitance relation connecting `Int` to
`OrderedRing<Word64>`, and the `fn(T, T) -> T` morphism shape that
appears in every algebra's `op` field. If the shape hosts this, it
naturally hosts every domain compositional model built the same
way. If the shape cannot host it, the project is carrying a
parallel representation somewhere — a hardcoded primitives table,
a special-cased variant, a Rust-native type system — that breaks
the epistemic chain and will force every downstream consumer to
compensate. The test is not "what is convenient to ship first"; it
is "does the substrate preserve the chain?"

**This principle must not be dropped.** It is the reason codegen
can be mechanical, the reason new lenses are cheap to add, the
reason user-defined correctness dimensions work the same as
built-in ones, and the reason the project's cost of change stays
proportional to conceptual complexity instead of exploding with
consumer count. Every design decision that flattens the chain —
even "temporarily" — accumulates migration debt that compounds,
because each flattening becomes another ungrounded concept
downstream consumers must work around.

Lineage: this principle traces to the intersubjectivity and
epistemic-stacking work in the `definitely-not-agi` papers; see
`docs/design-lineage.md` §"Stage 4: definitely-not-agi" for origin.

### The substrate: two coordinated shapes

The epistemic stacking section above tests any candidate compiler
substrate against a concrete question: *can it host the Node-tree
form of `dsl/std/algebra.dag`?* The thesis commits here to two
coordinated substrate shapes that pass the test. Neither is
invented in this document; both are inherited from earlier design
work. `MODELING.md` already gives the architecture explicitly:

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts       (what flows through)
Foundation:         logical algebra                 (why it's sound)
```

The thesis's "type substrate" is the composition layer of that
architecture — `service`, `fn`, `type`, and `operation` are
surface sugar on top of it, and any compiler representation that
treats them as distinct substrate-level kinds (e.g.,
`DeclKind::Service`, `DeclKind::Operation`) is carrying the
parallel-representation disease the modeling discipline forbids.
See also `docs/design-lineage.md` §"Stage 3: gunbc v2" and
§"Stage 5: v3 spec," and the core-invariant memory
`feedback_compiler_is_dag_processor.md`. The thesis pins both
substrates down together because the epistemic chain rests on
them, and M1 substrate decisions need a single authoritative
statement to measure against.

**Types are Node trees with connectives.** Every type declaration
— a primitive, a record, a sum type, a function signature, a
parameterized generic, a service, an algebra, a correctness
dimension, a user-defined domain meta-model — is a Node with:

- A **connective**: `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`
- **Children**: labeled (`Conj`, `Disj`) or positional
  (`Cardinality`) edges to other Nodes in the type substrate
- An optional **inhabits** edge: a reference to an algebra
  declaration whose children become derived accessors on this Node
  via the §"Epistemic stacking" mechanism

Connective meanings:

- **Atom.** Terminal at the user-input boundary — identifiers,
  literals, source spans from outside the closed world. Atoms
  carry a payload (the literal bits or identifier name). Every
  other connective decomposes; Atoms do not.
- **Conj.** Product / record with named children. Hosts type
  declarations (`Monoid<T> { op: fn(T, T) -> T; identity: T }`),
  services (`service shell.Exec { operation Run { ... } }`),
  operation definitions, user-defined domain meta-models,
  correctness dimension declarations, and **value construction**
  like `transport shell { argv: ["sh", "-lc", "{script}"] }` —
  which is a Conj whose fields are bound to specific values, with
  an optional `inhabits` edge pointing at the meta-type the Conj
  satisfies.
- **Disj.** Coproduct / sum with labeled alternatives. Hosts
  explicit sum types (`type Result<T, E> = Ok(T) | Err(E)`), exit
  patterns (`exit { 0 => Unit; nonzero => String }`), and the
  elimination forms consumed by the computation substrate's Branch
  behavior.
- **Arrow.** Function signature from input Nodes to an output
  Node. The `op: fn(T, T) -> T` field inside `Magma<T>` is an
  Arrow Node with three positional type children. An Arrow
  declaration's **body** is a typed edge to either a sub-DAG of
  L1 behaviors (for user functions) or a realization declaration
  in `src/v3/spec/` (for primitives). See §"Two
  groundings" below for how the user-defined vs external-
  realization distinction works; the concrete `ArrowBody` enum
  is defined in `src/v3/compiler/src/dag.rs` with four variants
  (`UserDefined`, `ExternalRealization`, `Pending`, `Unparsed`).
  In neither case is the body a name-based lookup — both are
  structural typed edges.
- **Cardinality.** Repetition over a child Node with a (possibly
  symbolic) count. `argv: ["sh", "-lc", "{script}"]` is a
  `Cardinality(3)` with three String-Atom children. `List<T>` is
  `Cardinality(n)` over `T`. Bounded iteration falls out of
  `List<T>` having a finite `n`. Unifies v2's Required / Optional
  (presence) with repetition (lists, arrays) in one connective.
- **Instantiation.** Specialization of a parameterized template
  declaration by binding its template parameters to concrete
  types. `Int64 = OrderedRing<Word64>` is an Instantiation whose
  `template` is the `OrderedRing` declaration and whose
  `arguments` list contains a single binding `T := Word64`.
  Substitution happens lazily at walk time via a substitution
  stack: walkers push the arguments on entering the Instantiation,
  look up template-parameter references inside the template body,
  and pop on exit. This matches C++ template instantiation
  exactly — a template is a parameterized declaration (`Monoid<T>`,
  analogous to `template<typename T> class Monoid`), an
  Instantiation is the specialized form with concrete arguments
  (`Monoid<Int>`, analogous to `Monoid<int>`).

**Instantiation is ONLY for type parameterization.** Earlier drafts
of this section conflated three different things under the
"Instance" label: type parameterization (`OrderedRing<Word64>`),
value construction (`transport shell { argv: [...] }`), and
inhabitance witness (`Int inhabits OrderedRing`). These are
three different structural relationships and the conflation was
a coproduct hiding in plain prose. They resolve as follows:

- **Type parameterization** → `Instantiation` connective, as
  described above. Template parameters (the `T` atoms in the
  parameterized declaration) bind to concrete types at the
  Instantiation site.
- **Value construction** → plain `Conj` with the field values as
  children, and an optional `inhabits` edge pointing at the
  meta-type the Conj should satisfy. `transport shell { argv:
  [...] }` is a Conj with one labeled child `argv` (a Cardinality
  literal) and an `inhabits` edge to the `transport` meta-type.
  No Instantiation involved — it's just record construction with
  a type tag.
- **Inhabitance witness** → for pure aliases like `Int =
  OrderedRing<Word64>`, Instantiation IS the inhabitance. Int's
  declaration has connective `Instantiation` directly; there is
  no separate "Int declaration with an inhabits edge pointing at
  an Instantiation Node." The collapse is honest because Int has
  no structure of its own beyond the inhabitance. The optional
  `Declaration.inhabits` slot is reserved for the rare case of a
  primary-structure declaration (Conj/Disj/Atom) that
  additionally inhabits an algebra — like a hypothetical Money
  record that also inhabits a CurrencyLattice.

**Computation is five L1 behaviors.** Every function body is a
sub-DAG of Nodes in a separate substrate with exactly five
primitive behaviors:

- **Value.** A literal carried into the graph on a Port with no
  inputs.
- **Transform.** Apply a `FunctionRef` (a cross-substrate edge to
  an Arrow declaration in the type substrate) to input Ports,
  producing an output Port.
- **Branch.** Select between sub-DAGs based on a Bit-valued
  (Disj-eliminated) input Port; outputs are unified into a single
  output Port. Exhaustiveness is structural: a Branch whose cases
  don't cover the input's Disj is a compile-time unbound-child
  error.
- **Loop.** Bounded iteration over a source Port whose type is a
  `Cardinality` Node in the type substrate, with an accumulator
  Port and a body sub-DAG. Termination is structural rather than
  behavioral because the source's cardinality is finite.
- **Bind.** Name a sub-DAG's output Port so downstream Transforms
  can reference it.

No sixth behavior. M0 validated these five under three reviewer
rounds and never hit a stop signal (see
`src/v3/M0_RETROSPECTIVE.md`); the five-behavior claim is the
project's strongest empirical substrate commitment and is not
revisited here.

**The two substrates compose via FunctionRef and Arrow-body.** A
Transform Node in the computation substrate holds a FunctionRef,
which is a typed edge to an Arrow declaration in the type
substrate. The Arrow declaration's `body` child is a Node
reference into the computation substrate (for user functions) or
into `extdeps/` (for primitives). So the two substrates meet at
exactly two points: the `Transform → Arrow` lookup on one side
and the `Arrow → body` lookup on the other. Inference resolves
types across this boundary; emission projects across it. There is
no third substrate.

**The load-bearing test: host `dsl/std/algebra.dag`.** For any
candidate compiler-internal Declaration shape, the check is: *can
it host the Node-tree form of `std/algebra.dag` as-is?*
Specifically, the shape must carry four things:

1. **Parameterization over T.** `Magma<T>`, `Monoid<T>`, `Ring<T>`
   — type declarations whose children reference a free type
   parameter. The substrate must represent `T` as a child Node
   (probably an Atom with a parameter marker) that gets bound at
   Instantiation time, not as a string name with ad-hoc substitution.

2. **Structural children.** `Monoid<T>` as a Conj with children
   `{ op: Arrow(T, T, T); identity: T }`. The substrate must carry
   these children as typed Node edges, not as serialized metadata
   on a nominal declaration.

3. **The `fn(T, T) -> T` morphism shape.** An Arrow Node with
   three positional `T`-references. The substrate must represent
   function types as first-class Node shapes, not as separate
   Function declarations with inline signatures disconnected from
   the algebra they belong to.

4. **The inhabitance relation.** `Int inhabits OrderedRing<Word64>`
   — realized as an Instantiation node whose `template` is
   `OrderedRing` and whose `arguments` are `[T := Word64]`. Int's
   declaration IS this Instantiation; no separate edge needed for
   the pure-alias case. The substrate must carry this relation
   structurally so that `Int.add` resolves by walking `Int →
   Instantiation → OrderedRing (template) → add child →
   Arrow(T, T, T)` with `T := Word64` substituted from the
   arguments stack at each parameter reference — with no separate
   `add` declaration attached directly to Int.

If the substrate hosts `algebra.dag` through this walk, it
automatically hosts `shell.dag`, domain meta-models, user-defined
correctness dimensions, `service OrderAPI via rest::server`
declarations, and every other compositional declaration in the
project — because they are all Node-tree compositions in the
same substrate with the same six connectives. **If it cannot
host `algebra.dag`, the substrate is too narrow** and is carrying
parallel-representation debt that will force special-case handling
in every lens and every emission target. The test is not "what is
convenient to ship first"; it is "does the substrate preserve the
epistemic chain?"

**Substrate extension is a C1-class stop signal.** Adding a
seventh connective to the type substrate, or a sixth L1 behavior
to the computation substrate, is a substrate-level commitment
that requires the same discipline as any coproduct extension:
*all four dissolution patterns from §"Structural decompression"
must be attempted, and extension is only valid if all four fail
with structural arguments.* "A new kind of thing we need" is
never sufficient — the new thing must be shown to not decompose
into existing connectives or behaviors. This stop signal applies
to both substrates symmetrically.

**Future candidate (not committed): unified substrate.** A
stronger redesign that dissolves the five L1 behaviors into
patterns over the type substrate — treating Value as an Atom
carrying a literal, Transform as an Instantiation of an Arrow,
Branch as an Instantiation of Disj elimination, Loop as an
Instantiation of `fold`, Bind as a local-scoped declaration —
has been discussed as a candidate for concept-unification applied
to the substrate itself. **The thesis does NOT commit to this
collapse.** M0's validation of the five behaviors as substrate
primitives stands; revisiting them requires new failure pressure,
not just aesthetic or theoretical preference. Recorded here as a
candidate for future consideration only.

### Compositional layering: below-boundary opacity by construction

The two-substrate design is load-bearing only if it gives the
project a property the substrate by itself can't provide: **layers
compose such that below-boundary changes are invisible to
consumers**. That property is the reason to generate code rather
than write it by hand, and it is the reason the substrate is worth
the discipline it demands. Without it, the substrate is just a
different storage layout; with it, the substrate is a composition-
preserving guarantee. This section pins the claim.

**The networking analogy is exact.** The OSI stack is the proof-
of-concept: HTTP does not know or care what transport it is running
over. You can swap TCP for QUIC, add TLS between the transport and
application layers, or change the link layer from Ethernet to WiFi
to fiber to cellular, and application code — literal HTTP requests
from a literal browser — is structurally unaware. HTTP has **no
API** for asking "which transport are we running over right now?"
Below-boundary details are not merely hidden, they are *unnameable*
to consumers. That unnameability is what makes the internet scale
N+M instead of N×M: each layer can evolve independently because
no layer above it can depend on below-boundary internals.

**gunbc's compositional modeling claims the same property for
general software composition.** A `rest / http / service`
composition in `.dag` should have the same layering guarantees as
`HTTP / TCP / IP`. You can insert a retry layer between `rest` and
`http`. You can swap `http` for a different transport. You can
replace `service`'s implementation entirely. Application code
**cannot observe the change** because the compiler never exposes
below-boundary identifiers to it. The user edits one layer; every
other layer keeps working without modification. This is what the
thesis means when it says "the cost of change is 1."

**Generated code is the mechanism.** Hand-written code embeds
composition assumptions at every call site: function names,
parameter shapes, ownership conventions, error channels, import
paths. Every line is a reference to specific names in specific
layers. A layer swap requires editing every consumer because every
consumer spelled out the composition manually. Generated code
inverts this: the compiler walks the structural graph and emits
target-language code from the walk. The consumer's `.dag` source
declares *intent*; the compiler resolves *references* by walking
the substrate; the emitted code is the result of that walk. When
an intermediate layer changes internally, the walk produces the
same result because the walk goes through structural edges, not
through memorized names.

**The test: rename any below-boundary identifier.** The cleanest
empirical check of the layering claim is to pick an identifier
that's internal to some layer (below its public boundary), rename
it, and see whether any consumer's generated output changes. If
consumers produce identical output, the layer was opaque. If any
consumer's output differs — or worse, if consumers fail to compile
— the layering leaked and the compiler was reaching below the
boundary to read the identifier by name. This is the **rename
test**, and it is the cheapest invariant check the compiler can
run.

**Empirical validation (2026-04-15).** The weather example in
`dsl/examples/weather/` was compiled to Rust with v2, then the
`Float` declaration chain in `dsl/std/float.dag` was edited three
ways. For each variant, `diff -r` was run against the baseline:

1. **Insert an intermediate layer.** `Float = PreciseScalar =
   Float64 = Field<Word64>` instead of `Float = Float64 =
   Field<Word64>`. Weather.dag unchanged. Generated Rust
   **byte-identical** to the baseline. The compiler walked the
   algebraic chain and did not notice the new intermediate alias.
   ✅ Layering holds.
2. **Rename internal layers below the boundary.** `Float64 →
   BinaryFloat64` and `Float32 → BinaryFloat32`, keeping `Float`
   as the boundary alias. Weather.dag unchanged. Generated Rust
   **byte-identical** to the baseline. The compiler walked
   through the renamed internal names and still produced `f64`.
   ✅ Layering holds.
3. **Rename the boundary identifier itself.** `Float → FloatingPoint`
   in std/float.dag, weather.dag updated to use the new name via
   an explicit `import`. The consumer update is expected: consumers
   depend on boundary names. The question is whether anything ELSE
   breaks. Answer: **yes**. Generated Rust is structurally
   different — fields change from `celsius: f64` to
   `celsius: Box<FloatingPoint>`, `Temperature` loses its `Copy`
   derive, every use site gains `Rc<Temperature>` wrappers, and
   function signatures return `FloatingPoint` instead of `f64`.
   The leak: v2's inference and emission have a **fast path** for
   types whose canonical name appears in `kernel_type_set` (a
   string-keyed map in `dsl/std/types.dag`), and a slow path for
   everything else. Renaming a primitive moves it from fast path
   to slow path without warning. ⚠ Layering leaks.

**The leak class.** What the weather experiment found in v2 is a
general pattern: **canonical-primitive-name rosters**. These are
tables keyed on `String` that map "known primitive names" to
behavioral decisions — which types get efficient emission, which
variants participate in fast-path dispatch, which declarations
count as kernel types. They appear in every compiler that stages
up from minimal parsing: the simplest way to recognize a primitive
is by its name, and adding a name table is cheap. The table
**dissolves** below-boundary opacity because it makes the primitive
name observable to the compiler's decision logic. Rename the
primitive and the dispatch changes even though the primitive's
algebraic structure is identical.

The fix is not to remove the distinction between fast and slow
paths — primitives really do map to different target representations,
and that mapping is the point of the language spec. The fix is to
**key the distinction on structural identity (a `DeclarationId`)
rather than on a string name**. A language spec declares "this
realization targets the declaration at `DeclarationId(X)`" via a
typed edge; the compiler walks the edge at resolution time; renaming
the declaration doesn't change its identity, and the walk still
finds the right realization. `kernel_type_set`'s replacement is the
dissolution of the canonical-name roster in favor of typed edges
from realizations to the declarations they realize.

**This is the same pattern the review cycle kept finding.**
Rounds 5–7 of M1(2.6) spent most of the milestone eliminating
name-keyed dispatch at the inference layer. M1(2.7)'s big fix
closed fourteen gaps, most of them instances of the same pattern.
PR-B of M1(3) shipped with a fresh version of the same pattern at
the emit/language-spec layer (`lookup("Int", "")`,
`lookup("Bool", "")`, `match variant.as_str() { "True" => ... }`).
Different layer, same leak. The root cause in every case is that
the primitive's identity was carried as a string, and the compiler
made a behavioral decision by string comparison. Layer opacity is
the invariant that would have caught every one of those cases at
introduction time if it had been enforced structurally from M0.

**Relationship to other thesis sections.**

- **Epistemic stacking** says concepts ground in primitives via
  a structural DAG. Layer opacity is the *runtime consequence* of
  epistemic stacking: if consumers walked the stack structurally,
  renaming any concept in the stack should not change the walk's
  result for consumers above it. A leak in layer opacity is
  always a leak in epistemic stacking — somewhere, a consumer
  short-circuited the walk by reading a name instead of following
  the structural edges.
- **The substrate: two coordinated shapes** provides the storage
  that makes the walk possible. The two substrates carry typed
  edges between declarations and between computation nodes, and
  those edges are what the walk traverses. Without typed edges,
  the walk collapses into name lookups.
- **Two groundings** (following this section) extends the layering
  claim to the target world: static grounding means concepts
  decompose to primitives structurally inside gunbc, realization
  grounding means primitives map to target representations
  structurally in the language spec. Both groundings must respect
  layer opacity — the static decomposition must not leak internal
  identifiers up to consumers, and the realization must not leak
  target-internal names (e.g., Rust's `i64`) into compiler
  decision logic.
- **Omni-emission** (far below) is what layer opacity enables at
  scale: one intent graph, many target artifacts, with no target-
  specific logic in compiler code. If any target emission has a
  hardcoded list of primitive names it knows about, adding a new
  target requires Rust edits; if every target's knowledge lives in
  a language spec referenced by typed edges, adding a new target
  is a single spec-file edit.

**Enforcement: lenses, not grep.** Layer opacity is a structural
query over a DAG — "walk every consumer site, flag any that reads
a below-boundary identifier by name" — and that makes it the
paradigmatic case for a **lens**. A lens is a pure reader over
the substrate that answers a structural question, and lenses are
the thesis's intended extensibility point for invariant
enforcement. The layer-opacity lens takes a `BoundarySpec` (which
declarations count as below-boundary for this application) and
returns a list of violations. Applied to compiler source with
boundary `= dsl/std/**`, it returns every place in the compiler
where a user-facing std/ identifier is read by name. Applied to
a user project with boundary `= project's internal layer`, it
returns violations in the user's own composition. **Same
mechanism, parameterized application.**

Lenses are the structural answer to invariant enforcement, and
layer opacity is the first invariant the project should use them
for. Grep gates were the earlier proposal in this document; they
are superseded by `INVARIANTS.md` §"Layer opacity" pointing at a
lens with opt-in application. The rename test remains as a
regression safety net, but the primary enforcement is the lens
output.

**Lenses as the general enforcement mechanism.** The layer-opacity
case generalizes. Most testable invariants over a DAG can be
stated as "walk the DAG, flag sites that violate property X" —
which is the shape of a lens. Compositional layering, scaffold
boundaries, fail-closed discipline, structural duplication,
epistemic grounding termination, dataflow reachability — all are
lens-shaped. The thesis's early framing of lenses as "cost,
ownership, complexity" undersold them. Lenses are **the
invariant-enforcement primitive**: adding a new invariant becomes
adding a new lens, applied to whichever files or folders should
be subject to it, with no substrate changes required. The review
cycle's historical cost came from re-implementing per-round
invariant checks by hand; the lens library is the structural
answer that replaces the hand-work with a standing query.

**Operational commitment.** Every consumer of the substrate —
lens, emitter, interpreter, future analysis tools — must pass
the layer-opacity lens for every identifier below its consumed
boundary. The lens is cheap to run (walk the DAG, return a list
of violations), general (same lens handles compiler source, user
projects, or any DAG), and catches the leak class that has
historically accounted for the largest share of review-round
findings. The rename test is the regression smoke test; the
layer-opacity lens is the primary enforcement; substrate-level
structural constraints (e.g., a `DisplayName` type without `Eq`)
are the long-term target that makes the lens's findings impossible
to construct in the first place.

Layer opacity is the property the substrate exists to provide.
Without it, the substrate is structurally interesting but
operationally indistinguishable from any other storage layout. With
it, the substrate delivers what the thesis promises: independent
evolution of layers, one-file edits for new targets, and
compositional modeling that matches the OSI stack's empirical
scalability proof.

### Two groundings: static validation vs efficient realization

Every concept in gunbc has **two groundings** that must both be
present, in the thesis's end-state, and must be consistent with
each other. These are different things — they have different
shapes, different purposes, and different completeness
requirements. Earlier drafts of the thesis blurred them; this
section pins the distinction down.

**Milestone scope note.** The thesis's top-level claim ("if it
compiles, the intent is sound and will execute as declared")
applies when both groundings are in place. **At M1(2.5), only
the static grounding is validated.** Realization grounding — the
target-language mapping that makes concepts emittable — lands at
M1(3)+ as language-spec declarations in
`src/v3/spec/`. During the M1(2.5) → M3 transition,
some primitive Arrows carry a scaffolded `Pending` realization
state: their concept decomposes completely via inhabitance, but
their target-language realization is declared later. A CI
ratchet (tracked in `INVARIANTS.md` §"No short-term solutions"
exception 2) requires that the Pending count monotonically
decreases and every Pending Arrow resolves to `UserDefined` or
`ExternalRealization` before M3 completes. The top-level executability claim is
therefore **milestone-conditional**: fully satisfied once M3's
realization declarations land, not before.

**Grounding #1: static grounding (`.dag` level).** Every concept
walks all the way down to the root primitives (Classical Bit +
Conj + Disj + Atom) via structural composition. The chain is
complete, deep, and bounded — every step is a substrate edge,
and every walk terminates at the roots. There are no escape
hatches, no opaque intrinsics, no concepts whose decomposition
is unfinished. This is the grounding §"Epistemic stacking"
insists on and the grounding §"The substrate"'s load-bearing
test validates.

The purpose of static grounding is **correctness proof**. It
answers questions like: Is `Int.add` a valid concept? Does it
type-check? Does it terminate? Does adding `Int256` work without
touching the compiler? Every answer comes from walking the
decomposition chain; nothing depends on the target world. Static
grounding is pure, closed-system, compile-time-only.

**Grounding #2: realization grounding (target level).** When the
compiler emits to a target language, every concept maps to the
nearest efficient target primitive via a language spec
declaration. This grounding is **shallow and target-specific**.
It is not a decomposition — it is a projection onto the target's
native capabilities. Concrete examples:

- `.dag Int64` realizes as Rust `i64`, NOT as a struct of 64
  individual Bit nodes. The compiler does not emit carry-
  propagation loops to add two integers — it emits `a + b` using
  the target's native machine instruction.
- `.dag Bool` realizes as Rust `bool`, not as a two-variant enum
  that dispatches through match arms for every operation.
- `.dag String` realizes as Rust `String`, not as a `FreeMonoid<
  Char>` traversal.
- `.dag List<T>` realizes as Rust `Vec<T>`, not as a cons-cell
  chain.

The language spec (`src/v3/spec/rust.dag` and friends)
is the **mapping from `.dag` concepts to efficient target
primitives**. The emitter uses this mapping directly; it does
not walk down the decomposition chain and reconstruct. The
target's native primitives are the realization boundary — the
point where the compiler hands off to the target world.

**The two groundings have different shapes and different
completeness rules:**

| Property | Static grounding | Realization grounding |
|---|---|---|
| Depth | Deep (walks to Classical + Conj + Disj) | Shallow (one hop to target primitive) |
| Purpose | Correctness proof | Efficient execution |
| Completeness | Must be total | Must exist for every concept the target emits |
| Time | Compile-time, closed-system | Compile-time declaration, target-time execution |
| Substrate impact | None (concept decomposition) | Language spec declarations only |
| Consistency requirement | Type-preserving with declared laws | Semantically equivalent to static chain (L4 verifies) |

**The two groundings must be consistent by construction + verified
by L4.** The language spec's realization claim is structurally
consistent with the `.dag` declaration's algebraic laws: Rust's
`i64` satisfies the OrderedRing axioms modulo two's-complement
overflow (which is a handled special case with explicit bounds
on safe inputs). L4 verification (see §"Tier 3: Verification
from structure") tests this by generating witness values,
evaluating them at the `.dag` level via the deep chain, emitting
them via the shallow chain to the target, and comparing results.
When L4 says a concept's static and realization groundings agree,
the language spec's realization claim is certified.

**Why this distinction resolves the "ungrounded concept"
question.** Earlier drafts of §"Epistemic stacking" declared that
every opaque intrinsic, runtime-native helper, or bolt-on
analyzer is "unfinished composition." That reading is correct for
the static grounding — at the `.dag` level, every concept must
decompose completely. But it's wrong if you apply it to
realization: `.dag Int.add` at M1(2.5) has its realization
binding in a **`Pending`** state (one of the four `ArrowBody`
variants defined in `src/v3/compiler/src/dag.rs`) because its
target-world mapping lives in `src/v3/spec/rust.dag` — which
will be declared as
an ordinary Declaration reachable via a typed edge, not a
name-based lookup. That is NOT an ungrounded concept; it is
a **concept whose static grounding is complete and whose
realization grounding edge is pending declaration loading.**
The stop signal applies to static grounding (no concept without
a compositional chain), not to realization grounding (which is
deliberately target-specific and shallow by design, but still
structural when it lands).

**For M1(2.5):** primitive Arrows in `dsl/std/algebra.dag` carry
`body: Pending` — a scaffolded bootstrap state. Their static
grounding walks through inhabitance (e.g., `Int.add` walks
through `Instantiation(OrderedRing, [T := Word64])`); their
realization grounding will be declared in the Rust language spec
when that work lands, after which `Pending` resolves to either
`UserDefined(NodeId)` or `ExternalRealization(DeclarationId)`
— both structural edges. A CI ratchet enforces that no `Pending`
remains once the M3 extdeps work completes. The C-checkpoint for
"ungrounded concept" fires only when a concept cannot be walked
through its static chain — which `Int.add` can, via
`OrderedRing<Word64>`.

### Target realization efficiency

A direct consequence of the two-groundings distinction: **the
cost complexity of a concept is different at the `.dag` level
versus at the target level, and the compiler can compute both.**

**`.dag`-level complexity** is declared via CX (the complexity
dimension — see §"Correctness dimensions"). `Int.add` at the
algebra level is O(1) because it's a single field access on
`OrderedRing.add`. The CX claim is independent of any target.

**Target-level complexity** is declared in the language spec as
**realization cost** per primitive. Concrete examples:

- `rust.dag` declares: `Int64.add realizes as i64 + i64, cost
  O(1), ~1 machine instruction, ~1ns on commodity hardware`.
- `rust.dag` declares: `Int256.add realizes as carry-propagated
  quad-word addition, cost O(4), ~4 machine instructions`
  (assuming no native 256-bit hardware).
- `python.dag` declares: `Int.add realizes as Python int
  addition, cost O(1) amortized for small ints, O(log n) for big
  ints, with GC overhead`.
- `spice.dag` (if it exists as a language spec) declares: `Bool.and
  realizes as AND gate, cost O(1) gate delay`.

Target-level complexity **composes** with `.dag`-level CX. For a
`.dag` program that does `k × Int.add`, the target cost is `k ×
realization_cost(Int.add, target)`. The compiler walks the
declared program, composes `.dag` CX with language-spec
realization costs per primitive, and yields per-target
complexity bounds for any function. No execution required.

**Why this matters for omni-emission.** With cost complexity as
a first-class thesis claim, the question "which target is fastest
for this workflow?" becomes a compile-time question the compiler
can answer statically. `create_order` in Rust: 2μs median.
Python: 180μs median. Go: 3μs median. Target selection becomes
a cost-aware optimization with bounded complexity estimates,
not an execution experiment.

**Why this is not speculative.** `.dag` already commits to CX as
a correctness dimension (see §"Correctness dimensions"). The
language spec already exists as a declaration pattern (see
§"Concept unification" → "coercion = emission"). Composing the
two requires adding a `realization_cost` field per primitive in
the language spec, and a composition rule in the CX analysis
pass. Both are small additions to existing mechanisms. The
thesis makes the claim here; implementation falls out when the
emitter lands.

**The sustainability consequence.** Adding a new target language
automatically yields target-specific cost analysis for every
existing workflow — zero new work per workflow. Cost-aware target
selection is a free consequence of declaring realization costs
in the language spec.

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
are declarations in `src/v3/spec/` and
`dsl/extdeps/transports/`. Each spec is itself a Node-tree
composition in the same substrate, declaring: what primitive
shapes the target has, what its syntax is, how each connective
in the type substrate projects onto target constructs, and how
service/transport bindings map to target API calls. Adding a new
target — say, Swift for iOS clients — means writing a Swift
language spec in `src/v3/spec/swift.dag`. **Zero
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
compiler reads a language spec from `src/v3/spec/` and
emits target source code via the single emitter. Examples: Rust,
Python, Go, TypeScript, Swift, and potentially hardware
description languages (Verilog, VHDL, Chisel) where the target
executes the compiled program.

**Shape A is the target architecture, not the current reality.**
At the time of writing, the LanguageSpec mechanism is partial:
materialization strategy, sharing × serialization coupling, and
type-decoration selection are not fully modeled in LanguageSpec
and still leak into per-target emitter code (see `src/v2/*.dag`
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
  in `src/v3/spec/`. Applies to every workflow
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
`src/v3/spec/`, and the spec declares — as ordinary
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

## Thesis claims — complete list

Every claim the thesis makes, in one place. The ROADMAP tracks
progress toward each. If a claim isn't here, the project doesn't
claim it. If a claim IS here but the ROADMAP has no track for it,
that's a gap.

**Core abstraction:**
- .dag is dependency modeling software. The program IS a dependency
  graph. Parallelism is the default; sequential execution requires
  a data dependency to justify it.

**Tier 1 — Structural correctness (impossible to write the bug):**
- Type mismatches, field typos, non-exhaustive matches, bare
  container types, circular dependencies, stale imports, cross-target
  drift — all caught at compile time.
- CX gate: every recursive function terminates with a proven bound.
- Coercion = emission: the compiler reads a target spec and
  translates. No separate coercion engine.
- Ownership: the compiler proves no aliased mutation in emitted code.

**Tier 2 — Runtime safety (proven safe or total):**
- Division by zero, integer overflow, out-of-bounds, force-unwrap,
  partial functions — either proven safe at compile time or made
  total. No partial functions in the runtime.

**Tier 3 — Verification from structure:**
- L4: emitted code executes and matches .dag evaluation.
- L5: same .dag produces same behavior in Rust/Python/Go.
- L6: every structural form compiles to every target.
- L7: operations obey declared algebraic laws.

**Concept unifications:**
- Coercion cost = complexity.
- Coercion = emission.
- Target language spec = transport spec = interpreter runtime.
- Idempotency + cancellation + redundancy = algebraic simplification.

**Epistemic stacking (load-bearing for codegen — must not be dropped):**
- Every concept is a node in an ontological DAG rooted at a minimal
  set of primitives. No concept is opaque.
- Root primitives: Magma (closure), Monoid, BooleanAlgebra (classical
  logic), FreeMonoid<T> (free constructions). Declared in
  `dsl/std/algebra.dag`.
- Concrete types attach by inhabitance (Int inhabits OrderedRing,
  String inhabits FreeMonoid<Char>, etc.). Operations fall out; they
  are never declared separately.
- The epistemic chain IS the emission algorithm. Every emitter
  special case is evidence of an ungrounded concept upstream.
- Math primitives and domain primitives share one substrate. A user
  declares `type CIWorkflow<Step>` the same way `Int` is declared,
  and it projects onto code the same way.
- Substrate test for any candidate Declaration shape: can it host
  `dsl/std/algebra.dag` as-is? If not, the shape is too narrow.

**Substrate shape (two coordinated substrates — must not be flattened):**
- Types are Node trees with six connectives: `Atom | Conj | Disj |
  Arrow | Cardinality | Instantiation`. `service`, `fn`, `type`,
  `operation` are surface sugar over this layer
  (`MODELING.md` §"Composition layer"). `Instantiation` matches
  C++ template-instantiation vocabulary and is ONLY used for type
  parameterization; value construction (`transport shell
  { argv: [...] }`) uses plain Conj with an optional inhabits tag.
- Computation is five L1 behaviors: `Value | Transform | Branch |
  Loop | Bind`. Validated by M0 under three reviewer rounds; the
  stop signal never fired.
- Composition: Transform holds a FunctionRef to an Arrow declaration
  in the type substrate; the Arrow's body is a sub-DAG of L1
  behaviors (for user functions) or a realization declaration in
  `extdeps/` (for primitives).
- Substrate extension is a C1-class stop signal (seventh connective
  or sixth behavior) — all four dissolution patterns from §"Structural
  decompression" must fail with structural arguments before extension
  is allowed.
- Future candidate (NOT committed): unified substrate dissolving the
  five behaviors into patterns over Node. Recorded for future
  consideration only; revisiting requires new failure pressure, not
  aesthetic preference.

**Free consequences (fall out when Tiers 1-2 close):**
- Automatic parallelism from dependency graph.
- Automatic memoization from purity + cost.
- Space bound proofs from CX.
- Cross-language optimization from shared cost algebra.

**Omni-emission (1:1 effort applied to full-stack systems):**
- One workflow declaration projects onto every layer of a real
  application: DB schema, backend service, API client, frontend
  form, documentation — from one source.
- Coherence between layers is structural, not checked — drift is
  impossible because every layer derives from the same Node
  tree (directly via compiler emission for Shape A targets, or
  indirectly via Shape B user programs walking typed values).
- **Shape A — compiler language targets**: programming languages
  (Rust, Python, Go, TypeScript, Swift, HDLs). Compiler emits
  directly via a language spec in `src/v3/spec/`.
  Adding a new Shape A target costs one language spec. `O(1)`
  per target; zero compiler/emitter changes.
- **Shape B — user-program artifacts**: YAML configs, Terraform
  HCL, Kubernetes manifests, SPICE netlists, natural-language
  docs, SQL schemas, JSON Schema, OpenAPI specs. Emitted by
  `.dag` programs walking typed values via `concat`/`fold`/`match`.
  Per ROADMAP Track 16: these are not compiler render targets;
  they are user code generating target strings. Adding a Shape B
  target is writing one reusable `.dag` emitter program.
- Target-level cost complexity composes with `.dag`-level CX via
  language-spec realization costs — the compiler can compute
  per-target complexity bounds statically (see §"Target
  realization efficiency").
- The two-shape distinction follows ROADMAP Track 16's explicit
  decision: the compiler emits programming languages; everything
  else is user code.
- Cost scaling: Shape A is `O(1)` per language target; Shape B
  is `O(1)` per artifact class. Neither is `O(N × M)`. Effort
  scales with conceptual content, not with layer or target count.
- Emission is independent of intent. You declare what the system
  does; separately, you declare what artifacts it becomes.

**Meta-process modeling:**
- Bootstrap, CI, dev process modeled as .dag workflows.
- `dag run` is the primary execution path.
- Adding a CI gate, a Node field, or a target language requires
  editing one .dag file.

**Modeling discipline:**
- Every declared type has at least one structural consumer.
- Every service boundary uses typed enums, not String/Bool proxies.
- No fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`). Missing
  facts are compile-time errors, not runtime strings.
- No duplicate record shapes. One type per concept.

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
