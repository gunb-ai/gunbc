> Part of: [THESIS.md](../THESIS.md)
> Informed by: [v2-compiler-audit.md](v2-compiler-audit.md),
> [compiler-ideal-vs-actual.md](compiler-ideal-vs-actual.md),
> [binding-model-proposal.md](binding-model-proposal.md),
> [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md)

# v3 Compiler Design

A fresh compiler written in `.dag`, compiled by the existing v1 Rust
host, designed to enforce the thesis structurally — making violations
impossible by construction, not caught after the fact.

---

## Why v3, not incremental v2 reform

The v2 compiler has three structural escapes from the thesis. Each
one is load-bearing — downstream code depends on the escape. Fixing
them incrementally means changing the foundation while the building
stands on it.

| Escape | Where in v2 | Why it's structural |
|--------|-------------|---------------------|
| General recursion instead of `descend` | 0 uses of `descend`; 204 fuel/depth sites; 5,489-line CX verification | The language allows self-calls that should be expressed as primitives. CX exists to retroactively prove what should be syntactic. |
| Node as god object (18 fields, one type for all stages) | Every stage receives same `Node`; parser fills ~5 fields, zeros 13 | No type-level boundary between "this fact doesn't exist yet" and "this fact is none." Stages can reach into facts that aren't theirs. |
| ExprData as universal dispatch key (22 variants, 451 match sites) | Every stage matches on surface syntax | Cost of change: ~12 files per new expression. Stages dispatch on syntax instead of structural properties (edges, dimensions). |

v3 makes these three escapes impossible by construction.

---

## Thesis claims → structural requirements

Each claim from THESIS.md maps to a concrete requirement on v3.

### Claim 1: "All iteration is bounded (fold/descend/repeat)"

**Requirement:** `fold`, `descend`, and `repeat` are the ONLY ways
to iterate. No function calls itself by name. No mutual recursion
cycles. The computation graph is an unwinding across a clock axis
bounded by the input structure.

**v2 divergence:** `.dag` allows `self(...)` and named recursive
calls. `descend` has 0 uses. The compiler walks Node trees via
direct recursion + manual fuel counters.

**v3 design decision:** [OPEN — see question Q1 below]

### Claim 2: "Correctness dimensions are computed at binding sites, carried on bindings, enforced universally"

**Requirement:** `TypeBinding` carries all dimension values.
No separate analysis passes. The compiler reads proof strategies
from `std/` and executes them generically during inference.

**v2 divergence:** `TypeBinding` carries only `{ name, resolved }`.
Provenance is computed and discarded. CX reconstructs it via 33
heuristics. Ownership is a separate name-keyed pass.

**v3 design decision:** TypeBinding carries dimensions from day one:

```dag
type TypeBinding {
  name: String
  resolved: Node           // the type
  provenance: SubValueRelation  // how this value relates to its source
  ownership: OwnershipKind      // owned/borrowed/shared
  // future dimensions added here — one field per dimension
}
```

Dimensions are computed at the binding site during the single
Prove stage. No downstream reconstruction.

### Claim 3: "Emission is mechanical translation"

**Requirement:** The emitter reads `LanguageSpec` + dimension
proofs from bindings. It never decides. Adding a target language
= adding a spec file.

**v2 divergence:** Rust emitter is 5,894 lines, separate from
the unified path. Python (666) and Go (689) are unified.

**v3 design decision:** One emitter from day one. Every target
reads the same `LanguageSpec` structure. Ownership-aware emission
reads `TypeBinding.ownership`, not a separate pass result.

### Claim 4: "The program IS the dependency graph"

**Requirement:** The IR is a DAG. Edges carry structural
properties (SubValueRelation, UsageEdge). Stages reason about
edges, not about surface syntax forms.

**v2 divergence:** ExprData (22 surface-syntax variants) is the
dispatch key at 451 sites. Stages reason about "is this an
ExprForEach or an ExprLambda" instead of "what SVR does this
edge carry."

**v3 design decision:** [OPEN — see question Q3 below]

### Claim 5: "Cost of change: 1 file"

**Requirement:** Adding a new expression form, a new dimension,
or a new target language touches 1 file.

**v2 divergence:** New expression form → ~12 files. New dimension
→ new analysis pass. New target → new emitter file.

**v3 design decision:**
- New expression form: parser + desugaring to existing primitives.
  Downstream stages see only the 2 fundamental binding forms.
- New dimension: declare lattice in `std/`, add field to
  TypeBinding, add column to SVR-keyed dimension table.
- New target: add spec file in `extdeps/languages/`.

---

## v3 pipeline

```
Stage 1: Parse
  source text → ParseTree (spans, tokens, surface structure)

Stage 2: Resolve
  List<ParseTree> → ResolvedGraph (modules, imports, types, topo order)

Stage 3: Prove
  ResolvedGraph → ProvenGraph (dimension values on every binding edge)
  ONE stage. Reads proof strategies from std/. Executes generically.

Stage 4: Emit
  ProvenGraph → target language files
  Reads LanguageSpec. Reads dimension proofs from bindings.
```

### Stage 1: Parse (~5,000 lines, mostly reusable from v2)

The tokenizer and parser are largely correct in v2. Surface syntax
is stable. What changes:

- Parse produces a **parse-specific tree**, not the universal Node.
  Parse tree nodes carry only what the parser knows: spans, token
  text, children, surface structure. No `inferred`, no
  `is_self_recursive`, no `descent_evidence`.
- ExprData is a **parse-time** concept only. It's the surface
  syntax discriminant. It does NOT flow into later stages as the
  dispatch key.
- Desugaring happens here or at the Parse→Resolve boundary:
  `for-each` → `fold`, match arm bindings → `let` + field access,
  lambda variants → single boundary-crossing form.

### Stage 2: Resolve (~3,000 lines)

Module graph construction, name resolution, type resolution, generic
expansion, cycle detection, function signature resolution. Merges
v2's `03_resolve`, `03_normalize`, `04_resolve`, `04_sigs`,
`04_cycle`, `04_patterns`, `04_lookup`, and parts of `04_infer`.

Output: `ResolvedGraph` where every name is resolved, every type
is expanded, every binding site is identified. Bindings exist but
dimension values are not yet computed.

### Stage 3: Prove (~3,500 lines)

The core of v3. ONE stage that:
1. Walks the resolved graph using `descend` (structural traversal)
2. At each binding site, computes all dimension values:
   - **Type:** resolve expression type (from v2 inference)
   - **Provenance:** compute SVR from edge position + expression
   - **Ownership:** compute from SVR + UsageEdge
   - **Effects:** compose from operation declarations
3. Carries dimension values on `TypeBinding`
4. For each proof strategy in `std/`, executes it:
   - Traverse the relevant graph (`fold` over SCCs, bindings, etc.)
   - Compose with the dimension's algebra (lattice meet/join)
   - Check the gate (does the proof construct?)
5. Returns `ProvenGraph` with proofs attached

What dissolves from v2:
- `complexity.dag` (5,489 lines) → proof construction from
  `std/termination.dag`, no heuristic classification
- `ownership.dag` (635 lines) → dimension on bindings, no
  separate pass, no string matching
- `classify_*` system in `04_infer.dag` (~1,200 lines) → SVR
  computed once at binding creation
- `DescentContext`, `lambda_param_provenance` side-channels → gone

### Stage 4: Emit (~5,000 lines for all targets)

One emitter, parameterized by `LanguageSpec`. Reads dimension proofs
from `TypeBinding`. Walks `ProvenGraph` using `descend`.

What changes from v2:
- Rust emitter (5,894 lines) → same unified path as Python/Go
- Ownership-aware emission reads `TypeBinding.ownership`
- No separate `build_shared_types` / `build_ownership_results`
  reconstruction

---

## v3 IR: stage-indexed types

v2 uses one `Node` type everywhere (18 fields). v3 uses
stage-specific types:

```dag
type ParsedNode {
  span: SourceSpan
  ident_span: SourceSpan?
  surface: SurfaceForm       // what the parser saw (replaces ExprData for parse)
  children: List<ParsedNode>
  params: List<ParsedNode>
  body: ParsedNode?
  // NO: inferred, is_self_recursive, descent_evidence, ownership
}

type ResolvedNode {
  span: SourceSpan
  resolved_type: ResolvedType
  children: List<ResolvedNode>
  params: List<ResolvedNode>
  body: ResolvedNode?
  bindings: List<Binding>     // binding sites identified
  // NO: descent_evidence, ownership — that's Stage 3
}

type ProvenNode {
  span: SourceSpan
  resolved_type: ResolvedType
  children: List<ProvenNode>
  params: List<ProvenNode>
  body: ProvenNode?
  bindings: List<TypeBinding> // with dimension values
  proofs: DimensionProofs     // termination, ownership, effects
}
```

Each stage boundary is a function:
```
parse:   source text → ParsedNode tree
resolve: ParsedNode tree → ResolvedNode graph
prove:   ResolvedNode graph → ProvenNode graph
emit:    ProvenNode graph → target files
```

The types enforce: you cannot read `proofs` from a `ResolvedNode`
(field doesn't exist). You cannot read `resolved_type` from a
`ParsedNode`. Stage boundaries are structural, not optional.

---

## What v3 reuses from v2

| Component | Reuse strategy |
|-----------|---------------|
| Tokenizer (01_tokenize.dag, 523 lines) | Copy with minimal changes (output tokens, not Node) |
| Parser surface (02_parse.dag, 4,828 lines) | Adapt to produce ParsedNode instead of Node. Pratt parsing, item dispatch reusable. |
| Module resolution (03_resolve.dag, 462 lines) | Reuse module graph, imports, topo sort. Adapt types. |
| `std/` types (algebra, computation, termination, induction, effects) | Import directly — these are thesis-aligned |
| `extdeps/languages/` specs | Import directly |
| Language spec builders (languages.dag, 1,163 lines) | Reuse with minor adaptation |
| Coercion tables (coercion.dag, 297 lines) | Reuse — data-driven, thesis-aligned |
| Test corpus (all existing .dag programs) | v3 output diffed against v2 output |
| v1 Rust compiler | Bootstrap host: compiles v3.dag → v3 binary |

| Component | Must rewrite |
|-----------|-------------|
| 00_core.dag (Node, ExprData, core tables) | Replace with stage-indexed IR |
| 04_infer.dag (5,470 lines) | Replace with Prove stage |
| complexity.dag (5,489 lines) | Replace with proof construction from std/ |
| ownership.dag (635 lines) | Replace with dimension on bindings |
| 05_emit_rust.dag (5,894 lines) | Replace with unified emitter |
| compile.dag (1,066 lines) | New pipeline orchestration |

Rough estimate: ~7,000 lines reusable, ~20,000 lines rewritten,
target total ~18,000-22,000 lines.

---

## Bootstrap path

```
v3.dag source ──(v1 Rust compiler)──▶ v3 binary (first generation)
     │
     │  v3 binary compiles v3.dag source
     ▼
v3 stage0 .rs ──(cargo/rustc)──▶ v3 binary (second generation)
     │
     │  second generation compiles v3.dag source
     ▼
v3 stage0 .rs ──(identical to above? fixed point!)──▶ v3 binary
```

The v1 compiler is the bootstrap host. It already compiles `.dag`
to Rust. v3 is a `.dag` program. v1 compiles v3.dag, producing a
Rust binary that IS the v3 compiler. That binary compiles v3.dag
again, producing stage0 Rust code. If stage0 is identical on the
second pass, bootstrap is achieved.

v2 serves as the **test oracle** during development. For any `.dag`
input, v2 and v3 should produce semantically equivalent output
(identical after normalization). Divergences are bugs in v3.

---

## Open design questions

### Q1: How does v3 enforce bounded computation?

Three options:

**Option A: Language-level restriction.** `.dag` does not allow
`self(...)` calls or named recursive calls. All iteration goes
through `fold`/`descend`/`repeat`. This is the purest thesis
alignment but requires rewriting every recursive function in v2
(and in all `.dag` programs) to use primitives.

**Option B: Compiler-level restriction.** `.dag` syntax allows
`self(...)` but the v3 compiler REJECTS it unless the call
matches a recognized bounded pattern (child accessor → descend,
collection shrink → fold, arithmetic descent → repeat). This is
v2's CX but as a hard gate from day one, not an advisory.

**Option C: Structural lowering.** The parser/resolver recognizes
recursion patterns and LOWERS them to `fold`/`descend`/`repeat`
before the Prove stage. The programmer writes `self(node.left)`
and the compiler sees `descend`. If it can't lower, it's a
compile error. Proof construction works on the lowered form.

Each option has trade-offs for ergonomics, migration, and the
bootstrap path (v3 must compile itself — and v3 uses tree
traversal extensively).

### Q2: How does v3 traverse its own IR?

The compiler IS a `.dag` program that walks trees. In v2, this
is done via direct recursion (the escape). In v3, if recursion
is restricted, the compiler must walk its own IR using `descend`.

This is the self-referential test: can the compiler enforce bounded
computation on itself? If v3 walks IR via `descend`, and `descend`
is bounded by tree depth, and the compiler can prove `descend`
terminates — then the compiler proves its own termination.

This may require `descend` to be richer than a simple "visit
children" — the compiler needs to carry scope, accumulate results,
and make decisions at each node. `descend` might need to be more
like a `fold` over the tree structure (accumulator + per-node
function + structural descent guarantee).

### Q3: What replaces ExprData as the dispatch key?

Three options:

**Option A: Edge properties.** Stages dispatch on the SVR and
UsageEdge of bindings, not on expression forms. A function call
with `IteratedSubValue` is treated the same whether the surface
syntax was `fold`, `for-each`, or `list.map`. This is the purest
thesis alignment but may be too abstract for practical compilation.

**Option B: Reduced expression forms.** Instead of 22 variants,
v3 has ~8 fundamental forms (literal, variable, application,
abstraction, product, coproduct, projection, injection). Surface
syntax desugars to these. Downstream stages match on ~8 forms
instead of ~22. This is Lambda Calculus + algebraic data types.

**Option C: ExprData stays but is parse-only.** The parse tree
has ExprData (22 variants). The Resolve stage normalizes to a
smaller set. The Prove and Emit stages never see ExprData — they
see resolved structural nodes with dimension values on edges.

### Q4: What does v3 target first?

Building the full 4-target emitter from day one is expensive.
Options:

**Option A: Rust only.** Get to bootstrap (v3 compiles itself to
Rust) as fast as possible. Add Python/Go/Dag later.

**Option B: Dag only.** v3 emits `.dag` (identity emission) first,
proving the pipeline is correct. Then add Rust for bootstrap.

**Option C: Rust + interpreter.** Bootstrap + `dag run` from
day one.

### Q5: What is the minimum viable v3?

The smallest v3 that proves the thesis:
- Parses `.dag` source
- Resolves modules
- Computes SVR on every binding
- Proves termination from SVR (CX gate = 0 on its own source)
- Emits Rust
- Bootstraps (compiles itself)

Everything else (ownership, effects, Python/Go, omni-emission)
is layered on after bootstrap.

---

## Execution sequence

1. **Write v3 core IR** (stage-indexed types, TypeBinding with
   dimensions)
2. **Adapt tokenizer + parser** to produce ParsedNode
3. **Write Resolve stage** (adapt from v2 module resolution +
   type resolution)
4. **Write Prove stage** (the new thing — SVR on bindings,
   proof construction from std/)
5. **Write Emit stage** (unified emitter, Rust target only)
6. **Bootstrap** (v1 compiles v3, v3 compiles v3, fixed point)
7. **Validate** (diff v3 output against v2 output on full corpus)
8. **Layer ownership** (add OwnershipKind to TypeBinding)
9. **Layer effects** (add EffectShape to TypeBinding)
10. **Add Python/Go targets** (port unified emitter specs)
11. **Delete v2** (when v3 passes all tests and bootstraps cleanly)
