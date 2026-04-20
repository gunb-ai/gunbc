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

