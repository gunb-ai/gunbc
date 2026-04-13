> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md)
> See also: [compiler-reduction-plan.md](compiler-reduction-plan.md),
> [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md),
> [binding-model-proposal.md](binding-model-proposal.md)

# Compiler: First-Principles Architecture

What stages does the thesis REQUIRE? Not "what do we have" — what
MUST exist? And how does every thesis requirement (termination,
ownership, effects, testgen, emission) emerge from composition of
a small set of primitives?

---

## Primitives: what the system is built from

The compiler — and the programs it validates — compose from a
small set of primitives. Everything else emerges.

| Primitive | Declared in | What it is |
|-----------|-----------|------------|
| `Classical = True \| False` | `std/logic.dag` | Bivalent truth — the foundation |
| `Product` (AND), `Coproduct` (OR) | `std/constructors.dag` | How things compose structurally |
| `Monoid`, `Lattice`, `BoundedLattice` | `std/algebra.dag` | Algebraic structure on values |
| `fold`, `descend`, `repeat` | `std/computation.dag` | Bounded iteration (the only computation) |
| Functions (`A -> B`) | Language primitive | Implication — "given A, produce B" |

Every concept in the system composes from these:

- `SubValueRelation` — a `BoundedLattice` (algebra primitive)
- `DescentEvidence` — a `BoundedLattice` (same mechanism)
- `OwnershipKind` — a `BoundedLattice` (same mechanism)
- `EffectShape` — composes via `Monoid` (sequential) + `Lattice` (branching)
- `TerminationProof` — `fold` over `List<ProofEdge>` (bounded iteration over finite evidence)
- SCC analysis — `fold` over the call graph (bounded iteration over finite edges)
- Cost composition — `Semiring` on `CostExpr` (algebra)
- Testgen — `fold` over type inhabitants (bounded iteration over finite types)
- Proof strategies — `fold` + `BoundedLattice` + `Function` (three primitives)

**The test:** can every proof the compiler constructs be expressed
as a composition of `fold`/`descend`/`repeat` over `BoundedLattice`
values along `Product`/`Coproduct` structures? If yes, the compiler
is an evaluator of these compositions — and CX can prove it
terminates, because bounded iteration over finite lattices always
terminates.

---

## The compiler contract

The compiler takes `.dag` source and constructs a proof that the
causal graph is sound. If the proof constructs, emission is
mechanical. The compiler doesn't know WHAT it's proving — it reads
proof strategies from `std/` and executes them generically.

**The compiler should not know about complexity.** It should not
know about ownership. It should not know about effects. These are
dimension facts declared in `.dag` source (`std/`). The compiler
knows how to execute proof strategies composed from primitives.

---

## Proof strategies: everything is fold + lattice + function

A proof strategy is:
1. A **graph structure** to traverse (call graph SCCs, use-site
   fan-out, workflow composition, type inhabitant enumeration)
2. A **composition algebra** to apply along edges (`BoundedLattice`
   meet/join, `Semiring` add/multiply, `Monoid` concat)
3. A **gate criterion** to check (does the composed value satisfy
   the required property?)

All three are `.dag` data declarations. The compiler reads them
and executes: `fold` over the graph, compose with the algebra,
check the gate. One mechanism.

### Termination (KF-1)

```
graph:   CallGraphSCCs (call graph, decomposed into SCCs)
compose: compose_descent (SubValueRelation lattice meet)
gate:    all recursive calls show structural descent
```

- `fold` over each SCC's edges
- `compose_descent` along each call edge (reads SVR from bindings)
- Gate: every dimension in the `TerminationProof` has `Strict` evidence

The SCC algorithm is `std/graph.dag`. The composition is
`std/induction.dag`. The gate is `std/termination.dag`. The
compiler calls them — doesn't contain them.

### Ownership

```
graph:   UseSiteGraph (per-binding fan-out of use sites)
compose: ownership_meet (OwnershipKind lattice meet)
gate:    no SharedError (fan-out > 1 without last-use identification)
```

- `fold` over each binding's use sites
- Classify each use as `UsageEdge` (Consumed/Read/Projected/Threaded)
- Derive `OwnershipKind` from SVR on the binding edge
- Gate: for each binding, fan-out is sound (sole owner or last-use elision)

The lattice is `std/ownership.dag`. The use-site classification
reads from the walk context (which Node field, what expression
form). The compiler walks generically.

### Effects (idempotency, safety)

```
graph:   WorkflowComposition (sequential/parallel operation sequence)
compose: compose_effects (EffectShape monoid/lattice)
gate:    workflow effect is lattice (idempotent) or explicitly non-idempotent
```

- `fold` over the workflow's operations
- `compose_effects` from `std/effects.dag`
- Gate: the composed effect satisfies the workflow's declared property

### Testgen (Tier 3)

```
graph:   TypedModuleGraph (all declared types and operations)
compose: type_inhabitants (generate witness values from type structure)
gate:    every type has inhabitants, every operation has test coverage
```

- `fold` over type declarations
- For each type, generate witness values (finite types have finite
  inhabitants — `Coproduct` = enumerate variants, `Product` = compose
  fields)
- For each operation with a mock, generate: call mock → parse response
  → verify against declared type

### Cross-language equivalence (KF-4)

```
graph:   FunctionGraph (all declared functions)
compose: evaluate in interpreter + emit to each target + execute
gate:    interpreter result == emitted result for all targets
```

- `fold` over functions with generated inputs
- Evaluate in `.dag` interpreter (the oracle)
- Emit to Rust/Python/Go, execute, compare
- Gate: all results match

---

## Required stages (first principles)

### Stage 1: Parse

**Input:** source text.
**Output:** Node tree with spans.

A language needs a parser. This must exist.

### Stage 2: Resolve

**Input:** Node trees from multiple files.
**Output:** Module graph with resolved names, types, imports.

Modules need to be connected, names resolved, generics substituted.
This must exist.

**Current 13 sub-files → ideal 4-5 files:**
- Module graph resolution (imports, exports, topo sort)
- Type tree resolution (expand named refs, generics, aliases)
- Pure type vocabulary (structural predicates, constructors)
- Pattern matching / exhaustiveness
- Function signature resolution (call graph, mutual recursion)

Merge candidates: 03_normalize, 04_access, 04_method (dissolves
into std/ data), 04_cycle, 04_lookup, 04_items, 04_env.

### Stage 3: Prove

**Input:** Resolved module graph.
**Output:** Graph with dimension values on every edge. Proof results
per function/workflow. Diagnostics for failed proof constructions.

This is ONE stage, not three. The compiler:
1. Walks the resolved graph
2. At each binding site, computes all dimension values (SVR,
   ownership, effects) using the dimension's `compose` function
3. Carries dimension values on bindings through the IR
4. For each proof strategy declared in `std/`, executes it:
   traverse the relevant graph, compose with the algebra, check gate

**What moves from compiler to std/:**

| Currently in compiler | Moves to std/ | Why |
|----------------------|--------------|-----|
| Cost algebra (SizeExpr, CostExpr) | `std/computation.dag` | Algebraic facts |
| SCC analysis | `std/graph.dag` | Graph algorithm |
| Parser progress model | `std/computation.dag` | Domain-specific termination |
| Termination proof construction | `std/termination.dag` | Proof rules |
| Ownership rules | `std/ownership.dag` | Dimension algebra |
| Evidence classification (~330 lines) | DISSOLVES | Reconstruction that SVR eliminates |

**What stays in the compiler:**
- The generic proof mechanism: read strategies, execute them
- Expression typing: resolve expression types, check compatibility
- Scope management: locals, imports, func sigs
- Diagnostic construction

### Stage 4: Emit

**Input:** Graph with proven dimension values on every edge.
**Output:** Target language source files.

Reads `LanguageSpec` + dimension proofs. Never decides. Mechanical
translation.

---

## Line count estimate

```
COMPILER (src/v2/):              DIMENSION FACTS (dsl/std/):

  core.dag        ~1,000         algebra.dag (BoundedLattice)
  tokenize.dag      ~500         induction.dag (SVR, composition)
  parse.dag       ~4,500         termination.dag (proofs, evidence)
  resolve.dag     ~3,000         computation.dag (cost algebra, SCC)
  prove.dag       ~3,200         graph.dag (graph algorithms)
  emit.dag        ~3,000         ownership.dag (OwnershipKind)
  emit_rust.dag   ~5,600         effects.dag (EffectShape)
  emit_go.dag       ~690
  emit_python.dag   ~670
  compile.dag     ~1,000
  languages.dag   ~1,160
  coercion.dag      ~300
  artifact.dag      ~110
                 -------
                 ~24,730
```

**Current: 38,078 lines across 32 files.**
**Ideal: ~24,730 lines across ~13 compiler files + facts in std/.**

---

## The self-referential closure

The compiler proving ITSELF terminates, using the same mechanism
it uses to prove user programs terminate. The compiler is a `.dag`
program. Its functions use `fold`/`descend`/`repeat`. Its data
is finite (`BoundedLattice` values on edges). CX can prove every
compiler function terminates — because the compiler IS the
mechanism it applies.

When the compiler compiles itself:
- Stage 1 parses .dag source (including the compiler's own source)
- Stage 2 resolves names (including its own modules)
- Stage 3 proves properties (including its own termination)
- Stage 4 emits Rust (producing stage0, which IS the compiler)

The bootstrap loop is closed at the SEMANTIC level: the compiler
doesn't just produce code that happens to work — it proves its
own code satisfies the same properties it enforces on user code.

---

## Execution: what to do

1. **Prove the mechanism on SVR** — implement one dimension
   end-to-end via the generic mechanism. SVR is the candidate
   (already 80% there). When SVR flows through every edge without
   reconstruction and CX reads it without heuristics, the
   mechanism is proven.

2. **Move dimension facts to std/** — cost algebra, proof rules,
   parser progress, ownership rules, effect composition. The
   compiler calls them; doesn't contain them.

3. **Consolidate resolve files** — 13 files → 4-5 files.

4. **Reduce core.dag** — dissolve tables, Connective, VarBindingKind.

5. **Build proof strategy framework** — the generic mechanism that
   reads strategies from std/ and executes them. This is the
   architectural bet.

Each step is testable: existing tests must still pass. The
mechanism is proven when CX violations = 0 via the generic path.
