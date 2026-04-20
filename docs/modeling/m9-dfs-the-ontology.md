### M9: DFS the ontology — every construct attaches to first principles

The `std/` library is an **ontology** — a connected DAG of concepts
rooted in first-principles logic. Not just algebra (structures with
operations), but the complete inventory of what exists and how things
relate: logic, construction, algebra, iteration, termination,
discrimination, coercion. Every concept in the codebase traces back
through this ontology to `Classical` (True/False).

```
Classical (logic.dag)
├── Bit → Word8..Word64 (bit.dag)
│   ├── Int = Word64 + OrderedRing witness (integer.dag)
│   ├── Float = Word64 + ApproximateField witness (float.dag)
│   └── Bool = Classical itself
├── Product / Coproduct (constructors.dag)
│   ├── Node = recursive Product with Coproduct discriminant (00_core.dag)
│   └── every .dag type
├── Monoid → Semiring → Ring → Field (algebra.dag)
│   ├── FreeMonoid<T> → List, String (algebra.dag, string_type.dag)
│   ├── PartialFunction<K,V> → Map (algebra.dag)
│   ├── BooleanAlgebra<T> → Set (algebra.dag)
│   └── Lattice → BoundedLattice (algebra.dag)
│       └── DescentEvidence = BoundedLattice (termination.dag)
├── fold / descend / repeat (iteration.dag)
│   └── every loop, every recursion
└── Ordering = Less | Equal | Greater (algebra.dag)
    └── well-founded orderings → termination proofs (termination.dag)
```

**The methodology:** when implementing or changing code, think in terms
of DFS through the ontology. Start from the concept you need, walk DOWN
to its root. The root tells you what the concept ACTUALLY IS. Then walk
back UP from the closest existing concept in `std/` to find your
attachment point. The ontology has branches beyond algebra:

| Branch | Root concept | std/ file | What it covers |
|--------|-------------|-----------|----------------|
| **Logic** | Truth/Falseness | `logic.dag` | Propositions, connectives, entailment |
| **Construction** | Product/Coproduct | `constructors.dag` | Type forming, records, enums |
| **Algebra** | Monoid → Ring → Field | `algebra.dag` | Operations that emerge from structure |
| **Iteration** | fold/descend/repeat | `iteration.dag` | All bounded computation |
| **Termination** | Well-founded orderings | `termination.dag` | Proof that computation halts |
| **Observation** | Pattern discrimination | (needs `discrimination.dag`) | Matching, case analysis |
| **Coercion** | Algebraic sidecast | `coercion.dag` | Cross-language type mapping |

If your concept doesn't fit any branch, you've likely found a new
branch of the ontology — add it to std/ with an external authority.

**The process:**
1. "I need a cost expression type." → DFS down: what IS cost? It's a
   value in a semiring (add, multiply, zero, one) with a lattice join
   (max). → Walk up from `std/algebra.dag` Semiring + Lattice. Found.
   Don't invent CostExpr; use the existing algebraic structure.
2. "I need a progress tracking type." → DFS down: what IS progress?
   It's an ordering: strict decrease, same, or unknown. → Walk up from
   `std/termination.dag` DescentEvidence. Found. Don't invent
   ProgressKind; it's the same BoundedLattice.
3. "I need a parse result type." → DFS down: what IS a parse result?
   It's a value + state + errors. → Walk up: this is a state monad
   (threaded state) with error accumulation (writer). → If std/ has
   no monad type, ADD it with authority citation (Moggi 1989). Then
   use it everywhere instead of defining 36 bespoke result types.

**The test for any new type:**
- Can you point to its parent in the concept DAG?
- Does that parent already exist in std/?
- If yes: import and compose. If no: add it with an external authority
  citation, THEN import and compose.
- If you can't find ANY parent: you've likely invented an abstraction
  rather than discovered a concept. Reconsider.

**Why this works:** concepts rooted in first principles NEVER need
refactoring — they are what everything else refactors TOWARD. If
something competes with a concept in the DAG, the competing thing is
what needs to change, not the concept. A Semiring will always be a
Semiring. A BoundedLattice will always be a BoundedLattice. Code
grounded in these is permanent.

**Worked examples from the pipeline audit:**

| Ad-hoc type | DFS root | std/ attachment point | Cost of not doing DFS |
|---|---|---|---|
| CostExpr (7 variants) | Semiring + Lattice | `std/algebra.dag` line 145 | 11 walker functions, 30 CX violations |
| SizeExpr (5 variants) | CommutativeMonoid + Lattice | sub-algebra of CostExpr | Separate type doubling walker code |
| ProgressKind (3 variants) | BoundedLattice | `std/termination.dag` line 57 | Duplicate of DescentEvidence |
| 36 parse result types | State × Writer monad | needs std/ addition | 36 types instead of 1 |
| 22 resolve/infer result types | Writer monad | needs std/ addition | 22 types instead of 1 |
| AlgebraTypeTemplate (9 variants) | Free algebra of type constructors | Node (universal carrier) | Separate recursive type |
| InferScope ≅ ModuleContext | Product type (context) | same concept | 2 types for 1 concept |

**When you find a concept not in the DAG:** add it to `std/` with an
external authority citation. The citation is the proof that you
discovered something real, not invented something ad-hoc. Examples:
- `std/termination.dag` cites Floyd (1967), Lee/Jones/Ben-Amram (2001)
- `std/algebra.dag` cites ring theory, lattice theory
- `std/iteration.dag` cites catamorphism theory
- A new `std/discrimination.dag` would cite pattern calculus, tree automata
- A new `std/graph.dag` would cite Cormen et al., Tarjan (1972)

