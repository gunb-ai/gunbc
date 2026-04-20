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
   *Example:* `ListBuild` = FreeMonoid introduction via
   `concat`/`empty` from `std/algebra.dag`; iteration forms
   (`map`/`filter`/`fold`) = FreeMonoid catamorphism from
   `std/algebra.dag`; arithmetic `+`/`−`/`×` = operations gained
   by types inhabiting `Ring` / `OrderedRing` in
   `std/algebra.dag`.

   **What does NOT belong here.** Structural projection on
   `Conj` (field access: `p.first`) and case discrimination on
   `Disj` (match arms) are *substrate primitives*, not algebra
   inhabitance operations. They are intrinsic to the shape via
   the universal mapping property — a `Pair` doesn't "inhabit"
   a ProductAlgebra to gain projection, because being a `Conj`
   IS the product structure; there is no room to inhabit
   anything. Similarly, each `Disj` has its own eliminator
   shape (variable variant count, variable payload types), so
   there is no shared "DisjAlgebra" users inhabit. Projection
   and case are compiler substrate primitives expressed through
   `Behavior::Transform`'s target and `Behavior::Branch`'s
   Path/arm machinery respectively — see
   `docs/substrate-reflection-design.md` §3.6 and §3.7 for the
   option walks and committed substrate forms. **Intuitive
   categorical framing can drift from what the codebase actually
   declares;** verify against current `dsl/std/algebra.dag`
   before recommending an "algebraic form" decompression.

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

