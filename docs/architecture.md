> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md)
> Drill down: [src/v1/DESIGN.md](../src/v1/DESIGN.md), [algebraic-type-spec.md](algebraic-type-spec.md)

# gunbc Architecture

This document describes the architectural thesis. See
[ROADMAP.md](../ROADMAP.md) for current state and milestone plan.

## Compiler Substrate: Node and Edge

The compiler has two substrate primitives. Everything above them —
including truth values, type structure, and cardinality — is
compositional modeling in `.dag`.

| Primitive | What it is |
|-----------|-----------|
| **Node** | The universal carrier. Identity, structure, composition. |
| **Edge** | A directed relationship from one node to another. Edges are outgoing only — a node knows what it points to (children), never what points at it (parents). An edge either connects to a target node, or it doesn't exist. There is no "edge to nothing." |

Edges are directed: parent → child. A node sees its outgoing edges
(children) but not its incoming edges (who references it). This is the
D in DAG. Derived properties like fan-out (how many things reference a
binding) are computed by graph traversal, not stored on nodes. The
pipeline walks forward — parse, resolve, infer, emit all follow edges
in the outgoing direction.

## Why Two Substrate Primitives Are Sufficient

All possible states of a slot on a node:

| Edge exists? | Target node exists? | Valid? | What it is |
|---|---|---|---|
| Yes | Yes | Valid | Slot filled — relationship holds |
| Yes | No | **Invalid** | An edge must connect to something. Edge-to-nothing = no edge. |
| No | (N/A) | Valid | Slot empty — no relationship |

No third state. The binary distinction (exists / doesn't exist) is
inherent in both Node and Edge — no separate truth primitive needed.
Classical logic (`True | False`) is the first modeled layer above
the substrate: the point where the binary distinction inherent in
node/edge existence gets a formal name. Bit is classical logic given
a hardware name. Everything else composes upward from there.

From Node and Edge, all structural properties emerge:

**Product (AND)** — a node where all child edges connect. "A Person
has a name AND an age" means both edges point to nodes.

**Coproduct (OR)** — a node where exactly one child edge connects.
"A Shape is Circle OR Square" means one edge points to a node.

**Cardinality** — an edge that exists or doesn't. Present = connects
to a node. Absent = no edge.

**Bit / Classical logic** — modeled in `.dag` as
`type Classical = True | False` (Layer 0 of the composition stack).
Users write boolean values and the language has full classical logic.
The compiler processes these structurally: a two-variant coproduct
where the truth value is carried by which edge is active.

## How This Looks in Practice

Every `.dag` type is a composition of Node + Edge:

```dag
// Product: a node where all child edges connect (AND)
type SourceSpan {
  file: FilePath        // edge to FilePath node — connected
  start: Int            // edge to Int node — connected
  end: Int              // edge to Int node — connected
}

// Coproduct: a node where exactly one child edge connects (OR)
// True and False are both nodes that exist — the distinction
// is WHICH edge is active (edge connectivity, not a truth primitive)
type Classical = True | False
type Bool = True | False

// Cardinality: an edge that may or may not connect
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: Timestamp?          // edge connects or doesn't
}

// Recursive coproduct: edges can point back into the structure
type Stack<T>
  = Empty                         // terminal — no child edges
  | Push { top: T, rest: Stack<T> }  // product inside a coproduct variant

// Collection algebras: named compositions
type List<element> = FreeMonoid<element>
type Map<key, value> = PartialFunction<key, value>
```

The compiler sees nodes and edges — not names, not truth values, not
product/coproduct categories. `SourceSpan` and `AccessToken` are both
"node with three child edges" to the compiler. The structural
difference (all-connected vs one-connected) is an edge connectivity
pattern, not a compiler-known enum.

## Where We Are (current compiler primitives)

| What | How the compiler knows it | Sites | Status |
|------|--------------------------|-------|--------|
| **Node** | Substrate type | — | Keep |
| **Edge (DAG)** | Substrate (outgoing children) | — | Keep |
| **Conj / Disj** | `connective` enum on Node | 81 dispatch, 114 construction | Bridge — dissolve into edge patterns |
| **Cardinality** | `return_cardinality` enum on Node | 38 dispatch, 142 construction | Bridge — dissolve into edge existence |
| **Bool** | `kernel_types` string list, fabricated `bool_type` node, `"Bool"` → `BooleanAlgebraProfile` map, `type_name == "Bool"` in emit | 5 name-based sites, 1 structural | **Not dissolved** — compiler knows Bool by name |
| **Int, String, Float** | Same as Bool: `kernel_types` list, type algebra profile map, emit default values | ~20 name-based sites each | **Not dissolved** — compiler knows these by name |
| **List, Map, Set** | `container_types` string list, `is_container_type()` | ~10 name-based sites | **Not dissolved** — compiler knows containers by name |

The compiler currently has **2 substrate primitives + 2 bridge enums +
~8 name-known types**. The direction is to dissolve everything except
Node and Edge.

## Structural Principles

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `List`) are human-readable labels for
structural compositions, not compiler-meaningful identifiers. The
compiler must not branch on node names for structural decisions.

**2. Compiler errors are orthogonal to the node graph.**

When inference fails, the result is not a node — it is a structurally
distinct failure. `InferredNode = Resolved { node } | CompilerError
{ message, span }`. Emit never sees error nodes.

**3. Syntactically distinct forms for the same operation normalize
before inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`/`MethodCall` bridging is complete, nodes
carry declared structural properties, and parameterized types carry
their declared arity of children.

## Languages as Coercion Targets

The compiler core does not know about Rust, Go, Python, SPICE, or
Verilog. It does not know about any specific target. Rendering is
**coercion** — finding the minimal complete representation of each
graph segment in the target's native capabilities.

This is the same problem as compiling floating point on a target
with no FPU:
- Target has native FPU → use it (efficient, zero overhead)
- Target has no FPU → synthesize from integer ops (correct, expensive)
- The compiler picks the minimal representation automatically

For graph patterns:
- Target has native branching (Rust `match`, Verilog `case`) → use it
- Target has no branching (pure analog SPICE) → synthesize from mux/
  comparator circuits (correct, more components)
- The cost algebra reports the difference

**The rendering model:**

A language plugin declares two things:
1. **Native capabilities** — what the target does efficiently
2. **Rendering table** — how each structural pattern maps to target
   constructs

The compiler walks the typed graph. For each node, it matches the
structural pattern (edge count + which edges connect) and looks up
the target's rendering. If the target has a native mapping → use it.
If not → synthesize from more primitive capabilities (and the cost
algebra reflects the overhead). If not even synthesizable → compile
error.

| Pattern (Node + Edge) | Rust (native) | SPICE analog | Verilog | English |
|---|---|---|---|---|
| Product (all edges) | struct | subcircuit ports | module ports | "X has Y and Z" |
| Coproduct (one edge) | enum/match | **mux** (synthesized) | case/mux | "X is Y or Z" |
| Cardinality | Option | **tri-state** | tri-state | "optionally" |
| Sequence | let bindings | wire chain | assign chain | "first X, then Y" |
| Function | fn | subcircuit | module | "given X, produce Y" |

**Any causal system qualifies.** The `.dag` graph is directed
relationships between entities — the same structure as circuits,
HDL, natural language, and serialization formats. The minimum: can
you model a directed connection between two things? If yes, you can
render a `.dag` program. The quality varies by how many patterns the
target handles natively vs synthesizes.

**Current reality:** 6,857 lines of language-specific code inside
`src/v1/` and 632 mentions of specific language names across 12
compiler files. These should all be zero.

**Challenge targets** (design validation — if the rendering model
works for these, it works for anything):
- **Verilog** — hardware: products are module ports, coproducts are
  muxes, sequences are assign chains
- **SPICE** — analog: products are subcircuit parameters, coproducts
  are synthesized from comparators + switches, cardinality is tri-state
- **English (Markdown)** — natural language: products are bullet lists,
  coproducts are "either/or", functions are paragraphs

Adding a language = writing `coerce.dag` (coercion rules declaring
the target's basis) + `render.dag` (trivial text output from
target-basis graph) in `dsl/extdeps/languages/`. Zero compiler
changes.

## Decidability

Every `.dag` program is decidable. Recursion, loops, and cyclic-looking
patterns are surface syntax sugar that decomposes into bounded iteration
over finite structure.

The language provides:
- `fold`, `map`, `filter`, `flat_map` — bounded by collection size
- `descend` — bounded by tree depth (structural descent)
- `repeat(bound: N)` — bounded by explicit count

The language does not provide `while(true)`, unbounded `loop`, or
unrestricted recursion. Undecidable programs are structurally
unrepresentable, not detected and rejected.

## Composition Stack

The foundation is formal logic, not hardware. Each layer builds on
the one below through composition. Everything reduces to classical
truth — the binary distinction inherent in node/edge existence.

| Layer | What | Built from | Example |
|-------|------|-----------|---------|
| 0 | **Classical logic** (`True \| False`) | Substrate (node existence = truth) | `std/logic.dag` |
| 1 | **Bit** | Classical logic given a hardware name | `std/bit.dag` — `type Bit = Classical` |
| 2 | **Machine words** (`Word32`, `Word64`) | Compositions of Bits | `std/bit.dag` — `type Word64 = Tuple<Bit, ..., Bit>` |
| 3 | **Algebraic types** (`Int`, `String`, `Float`) | Algebraic structures over machine words | `Int = OrderedRing<Word64>` |
| 4 | **Collections** (`List<A>`, `Set<A>`, `Map<K,V>`) | Algebraic structures with laws | `List = FreeMonoid` |
| 5 | **Structural compositions** + bounded iteration | Nodes + edges + algebras | `std/iteration.dag` |
| 6 | **Domain types** | Compositions of Layers 0-5 | Compiler, user programs |

Product/coproduct are not a layer — they are the composition
mechanism: how nodes at any layer combine through edges.

## Type coercion through the stack

Every type must reduce to classical truth through composition.
This is how new types coerce into target languages — the language
doesn't need to know what the type IS, only how to render the
structural patterns it decomposes into.

```
Bloobear                          -- user-defined Layer 6 type
  = Product { x: Wobble, y: Int } -- decomposes to product of fields
    → Wobble decomposes to...     -- each field decomposes further
    → Int = OrderedRing<Word64>   -- Layer 3: algebraic structure
      → Word64 = Tuple<Bit×64>   -- Layer 2: machine word
        → Bit = Classical         -- Layer 1: hardware name
          → True | False          -- Layer 0: classical truth
            → node exists or not  -- substrate
```

The target language renders each structural pattern it encounters
during decomposition: product → struct/subcircuit/bullet-list,
coproduct → enum/mux/"either-or", leaf → literal value. The language
never needs to know what "Bloobear" means — it renders the structure.

**If a new type CAN'T reduce to classical truth** (e.g., fuzzy logic
with truth values between 0 and 1, or quantum superposition), that's
a new Layer 0 — a genuinely different logical foundation. This is a
thesis-level change: the substrate may need to grow, and every
language plugin needs new rendering entries. This should be
extraordinarily rare.

See `docs/algebraic-type-spec.md` for the collection algebra and
denotational model.

## Guarantee Tiers

| Tier | Property | How enforced | Example |
|------|----------|-------------|---------|
| 1 | Structurally enforced | Violations don't compile | `ExpectedToken` enum — missing match arm = Rust error |
| 2 | Tested and gated | CI/test catches regressions | Scrambled-name tests — renamed types produce identical graph |
| 3 | Machinery exists, not gated | Report-only, dangerous | L1 ratchet script — runs but not in CI |
| 4 | Design exists, no machinery | Future | Parse-emit round-trip test |
| 5 | Fundamentally limited | Can't fully prove | Semantic correctness of emitted code |

All invariants should aspire to Tier 1 (unrepresentable violations).
Tier 3 items are the most dangerous — they give the illusion of coverage
without enforcement. Promoting Tier 3 to Tier 2 is always high-priority.

## Where We Will Be (target state)

| What | How the compiler knows it | Sites |
|------|--------------------------|-------|
| **Node** | Substrate type | — |
| **Edge (DAG)** | Substrate (outgoing children) | — |
| **Everything else** | `.dag` declarations the compiler reads structurally | 0 name-based sites |

- Product/coproduct, cardinality, and truth are compositional modeling
  above the substrate — edge connectivity patterns, not compiler enums.
- Names are opaque. Inference processes graph structure only.
- Zero language-specific code in the compiler core. Languages are
  coercion targets: `coerce.dag` (rules) + `render.dag` (trivial text)
  in `dsl/extdeps/languages/`. The compiler coerces the typed graph
  into the target's basis; the renderer produces text from the
  already-coerced graph.
- All `.dag` programs are decidable by construction.
- Ownership and complexity proofs wired into the pipeline.
- At least one real program compiles and runs end to end.

## Adding a new base concept

If the architecture is right, adding a new fundamental modeled concept
(a new Layer 0/1 type, beyond Bit/Bool) requires:

- **Add a `.dag` file to `std/`.** Define the type as a composition
  of the substrate primitives (Node + Edge).
- **Zero compiler changes.** The compiler reads structural properties
  from the `.dag` declaration. It doesn't know the concept by name.

If adding the concept requires compiler changes, the architecture has
failed — a structural fact is missing from the substrate.

**Test:** try to model the new concept as a `.dag` type. If you can
→ add the file, done. If you can't → the substrate needs to grow,
which is a thesis-level event requiring deep design review.

Examples: `type Probability = Float where range(0.0, 1.0)` (refinement,
zero compiler changes). `type Qubit = Superposition<Bit>` (new
constructor — would need substrate discussion). `type Signal =
Continuous<Float>` (new collection kind — would need substrate
discussion). The substrate should rarely grow; most concepts compose.
