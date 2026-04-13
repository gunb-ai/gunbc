> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md)
> See also: [compiler-reduction-plan.md](compiler-reduction-plan.md),
> [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md)

# Compiler: First-Principles Architecture

What stages does the thesis REQUIRE? Not "what do we have" — what
MUST exist?

---

## The thesis, restated as a compiler contract

The compiler takes .dag source and constructs a proof that the
program's causal graph is sound. If the proof constructs, emission
is mechanical. The proof has N dimensions, each declared in std/
as a BoundedLattice. The compiler doesn't know which dimensions
exist — it reads them from std/ and applies ONE generic mechanism.

**The compiler should not know about complexity.** It should not
know about ownership. It should not know about effects. These are
dimension FACTS declared in .dag source (std/). The compiler knows
how to process dimensions generically:

1. Read a `BoundedLattice<D>` declaration from std/
2. At each binding site, compute `D` using the lattice's compose
   function and the edge's `SubValueRelation`
3. Carry `D` through the IR on bindings
4. At gates, check the constraint (dimension-specific, but the
   MECHANISM of checking is generic)

`complexity.dag` (5,489 lines) as a compiler file is evidence that
the generic mechanism doesn't work yet. The cost algebra, SCC
analysis, parser progress model, termination proof construction —
these are all facts about the TERMINATION dimension, not compiler
infrastructure. They should be .dag functions in std/ that the
generic mechanism calls.

Same for `ownership.dag` (635 lines). OwnershipKind, UsageEdge,
fold detection, fan-out counting — facts about the OWNERSHIP
dimension. The compiler doesn't "run an ownership pass." It runs
the generic dimension mechanism, and ownership is one of the
dimensions it reads from std/.

---

## Required stages (first principles)

### Stage 1: Parse

**Input:** source text.
**Output:** Node tree with spans.

This must exist. A language needs a parser. The parser produces
the syntactic structure. It does NOT do semantic analysis.

**Minimal file set:**
- `01_tokenize.dag` — lexer
- `02_parse.dag` — recursive descent + Pratt

**Current state:** These files are ~90% and ~70% ideal respectively.
The parser has mechanical debt (predicate duplication, hardcoded
keywords, witness machinery in the wrong place) but the stage
itself is correct.

### Stage 2: Resolve

**Input:** Node trees from multiple files.
**Output:** A module graph with resolved names, types, and imports.
All names point to their declarations. Cycles detected.

This must exist. Modules need to be connected, names need to be
resolved, generic types need to be substituted.

**Minimal file set:**
- `resolve.dag` — module graph + name resolution + type resolution
  + cycle detection

**Current state:** Split across 03_resolve (461), 03_normalize (91),
04_resolve (992), 04_types (992), 04_patterns (242), 04_lookup (346),
04_items (150), 04_access (129), 04_service (250), 04_sigs (262),
04_method (113), 04_cycle (156), 04_env (133) = **4,317 lines across
13 files.**

The question: how many of these 13 files are genuinely distinct
concerns vs. artifacts of incremental development?

**Genuinely distinct:**
- Module graph resolution (imports, exports, topo sort) — 03_resolve
- Type tree resolution (expand named refs, generics, aliases) — 04_resolve
- Pure type vocabulary (structural predicates, constructors) — 04_types
- Pattern matching / exhaustiveness — 04_patterns
- Function signature resolution (call graph, mutual recursion) — 04_sigs
- Service graph (collect service deps, validate ops) — 04_service

**Should merge or dissolve:**
- 03_normalize (91) → merge into resolve (2 functions)
- 04_access (129) → merge into type resolution (index/slice checks)
- 04_method (113) → dissolve entirely (hardcoded builtins → std/ data)
- 04_cycle (156) → merge into resolve or sigs (cycle detection)
- 04_lookup (346) → merge into the proof stage (scope lookup during
  proof construction, not a separate concern)
- 04_items (150) → merge into resolve (item classification)
- 04_env (133) → this is TypeBinding + TypeEnv. These types stay,
  but the file could merge into the proof stage.

**Ideal file count: 4-5 files** (module resolve, type resolve, type
vocabulary, patterns, service graph) instead of 13.

### Stage 3: Prove

**Input:** Resolved module graph with typed nodes.
**Output:** The same graph with dimension values on every edge.
TerminationProof per recursive function. OwnershipProof per
function. Effect composition per workflow. Diagnostics for any
proof that fails to construct.

**This is the radical claim: prove is ONE stage, not three.**

The current compiler has three separate passes:
- `04_infer.dag` (5,470) — type inference + provenance
- `complexity.dag` (5,489) — termination proofs
- `ownership.dag` (635) — ownership analysis

The thesis says these should be ONE generic dimension mechanism
applied to all declared dimensions. The compiler doesn't know
about complexity or ownership. It reads dimension declarations
from std/ and applies the mechanism:

```
for each dimension D declared in std/:
  for each binding in the resolved graph:
    D(binding) = compose_D(edge.svr, source_D)
  for each function:
    check_D_gate(function)  // e.g., terminates? ownership sound?
```

**What moves from compiler to std/:**

| Currently in compiler | Should be in std/ | Why |
|----------------------|-------------------|-----|
| Cost algebra (SizeExpr, CostExpr, ~400 lines) | `std/computation.dag` (partially there) | Facts about cost, not compiler infrastructure |
| SCC analysis (~300 lines) | `std/graph.dag` (partially there) | Graph algorithm, not dimension-specific |
| Parser progress model (~500 lines) | `std/parser_progress.dag` (new) | Facts about parser termination |
| Termination proof construction (~200 lines) | `std/termination.dag` (partially there) | Proof rules for the termination dimension |
| Ownership rules (fold detection, fan-out, ~200 lines) | `std/ownership.dag` (proposed) | Facts about the ownership dimension |
| Evidence classification (~330 lines) | DISSOLVES | Reconstruction that SVR eliminates |

**What stays in the compiler:**
- The generic dimension mechanism (~500 lines estimated):
  read lattice declarations, compute at binding sites, carry
  through IR, check gates
- Expression typing (~2,000 lines from 04_infer.dag):
  the actual type inference (resolve expression types, check
  compatibility, produce typed nodes)
- Scope management (~500 lines): locals, imports, func sigs
- Diagnostic construction (~200 lines)

**Ideal file set:**
- `prove.dag` — the generic dimension mechanism + expression
  typing + scope management

**Current: 11,594 lines across 3 files + 13 sub-files.**
**Ideal: ~3,200 lines in 1 file** (the generic mechanism +
expression typing) **+ dimension facts in std/ (~1,600 lines).**

The 5,489-line complexity.dag largely dissolves: ~1,400 lines
move to std/ as dimension facts, ~330 lines dissolve (reconstruction),
and the remaining ~3,700 lines of SCC/cost/parser-progress become
std/ graph algorithms and dimension data. The 635-line ownership.dag
moves to std/. The 5,470-line 04_infer.dag sheds ~1,200 lines
of reconstruction and ~2,000 lines become the generic mechanism.

### Stage 4: Emit

**Input:** Resolved graph with dimension values proven on every edge.
**Output:** Target language source files.

This must exist. Emission reads LanguageSpec + dimension proofs
and produces code. It never decides — it reads.

**Minimal file set:**
- `emit.dag` — shared emission kernel (expression rendering,
  block structure, TCO, imports)
- `emit_rust.dag` — Rust-specific (derives, Rc, cargo, runtime)
- `emit_go.dag` — Go-specific
- `emit_python.dag` — Python-specific
- `languages.dag` — LanguageSpec data
- `coercion.dag` — type realization data

**Current state:** These files are ~75-90% ideal. The main debt
is ownership reconstruction in the Rust emitter (~300 lines),
which dissolves when ownership proofs are on bindings.

**Ideal: same file count, ~300 fewer lines.**

### Infrastructure

**Must exist:**
- `core.dag` — Node, ExprData, IR types. But smaller: no
  VarBindingKind, no Connective (use Product/Coproduct), no
  hand-maintained tables, no ~60 accessor functions (structural
  child access instead)
- `compile.dag` — pipeline orchestration
- `artifact.dag` — RenderTarget, ArtifactPlan

**Should dissolve:**
- `04_method.dag` → builtins become std/ data declarations
- `effect_derivation.dag` → bootstrap artifact
- `compiler_tests_rust.dag` → could be data-driven from specs

**Moves to std/:**
- `runtime_rust.dag` → arguably an extdeps concern (Rust runtime),
  not a compiler concern
- `trace.dag` → runtime contract, could be std/ or extdeps

---

## The minimal compiler

```
COMPILER (src/v2/):              DIMENSION FACTS (dsl/std/):
                                 
  core.dag        ~1,000         algebra.dag (BoundedLattice)
  tokenize.dag      ~500         induction.dag (SubValueRelation)
  parse.dag       ~4,500         termination.dag (DescentEvidence, proofs)
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
**Ideal: ~24,730 lines across ~13 files + dimension facts in std/.**

The ~13,000 line reduction comes from:
- ~1,200 lines of reconstruction in infer (SVR eliminates)
- ~5,500 lines of complexity.dag moving to std/ dimension facts
  (~1,400 lines) + dissolving (~330 lines) + becoming graph
  algorithms in std/ (~3,700 lines of SCC/cost/parser-progress)
- ~635 lines of ownership.dag moving to std/
- ~1,200 lines of sub-file merges (13 → 5 resolve files)
- ~1,000 lines of core.dag reduction (tables, accessors, VarBindingKind)
- ~300 lines of Rust emitter ownership reconstruction
- ~500 lines of misc (04_method dissolution, predicate dedup, etc.)

The total .dag code doesn't decrease by 13,000 — much of it MOVES
to std/ rather than disappearing. But the COMPILER shrinks from
38K to ~25K, and the compiler becomes generic over dimensions
rather than hardcoding complexity/ownership/effects.

---

## The key architectural question

**Does the generic dimension mechanism actually work?**

The dimensions-design.md says yes. But no dimension is fully
implemented via the generic mechanism today. SubValueRelation is
closest (it's on TypeBinding, it has a lattice, it has composition).
But even SubValueRelation is partially reconstructed downstream
(the triple classification problem).

The proof that the mechanism works: implement ONE dimension
end-to-end via the generic mechanism. SubValueRelation is the
candidate. When SVR flows through every edge via the generic
mechanism, and CX reads it without reconstruction, that proves
the mechanism. Then ownership and effects are the same mechanism
with different lattice declarations.

If the mechanism DOESN'T work (some dimension needs compiler-
specific logic that can't be expressed as a lattice + compose),
then the architecture is wrong and we need to understand why.

---

## Execution: what to do

1. **Build the generic dimension mechanism** — this is the
   architectural bet. One mechanism that reads any
   BoundedLattice<D> from std/ and processes it at binding sites.
   Test with SubValueRelation (already 80% there).

2. **Move complexity facts to std/** — cost algebra, proof rules,
   parser progress → std/computation.dag, std/termination.dag,
   std/graph.dag. The compiler calls them; doesn't contain them.

3. **Move ownership facts to std/** — OwnershipKind, UsageEdge,
   composition rules → std/ownership.dag.

4. **Merge resolve files** — 13 files → 4-5 files.

5. **Reduce core.dag** — dissolve tables, Connective, VarBindingKind.

Each step is testable: the 394 existing tests must still pass.
The dimension mechanism is proven when CX violations = 0 via
the generic path (not via complexity.dag heuristics).
