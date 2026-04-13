# v2 Retrospective: What Worked, What Didn't, What v3 Needs

> Part of: [THESIS.md](../THESIS.md)

## Executive summary

v2 achieved its primary goal: **glue code generation from dependency
modeling.** Given a DAG dependency in extdeps, the compiler generates
the integration code needed, eliminating glue bugs. The pipeline is
structurally sound — pure stages, threaded diagnostics, structural
types, no intermediate string IR in emission.

The problems are not in what v2 does. They're in how additional
concerns (complexity, ownership, effects) were added — bolted on
as afterthoughts rather than designed into the substrate. Each one
required a separate reconstruction pass because the IR doesn't carry
the facts they need.

**The v3 question:** can the substrate be designed so that adding
these concerns is additive (declare a lattice in std/, write a
binding-site rule) rather than reconstructive (write a 5,000-line
analysis pass that re-derives facts the IR already computed and
discarded)?

---

## 1. What v2 got right

### 1a. Glue code generation works

The core claim is validated: declare dependencies in .dag, emit
correct integration code. Cross-target drift is impossible because
all targets derive from the same declarations. This eliminates an
entire class of bugs (field typos, stale imports, type mismatches
in generated code).

### 1b. Pure pipeline architecture

Each stage is a pure function: `Input → { value, diagnostics }`.
Diagnostics thread through without loss. No mutable global state
between stages. This makes the pipeline testable, composable, and
comprehensible.

### 1c. Self-hosting validates the model

The compiler compiles itself. This is the strongest possible
validation: the .dag language is expressive enough to describe
its own compiler. 393 tests pass. Bootstrap converges.

### 1d. Structural type system

Types are values (Node trees with connectives), not registry
lookups. Inference is reconciliation — finding the structural
intersection of constraints — not Hindley-Milner unification.
This is the right design for a dependency modeling language.

### 1e. Bounded computation model

The three-primitive iteration model (fold/descend/repeat) makes
decidability a construction, not a check. All programs terminate.
This is foundational and correct.

### 1f. Interpreter proves the IR is complete

`dag run` executes validated IR directly. The interpreter reads
the same transport specs as the emitter. This proves the IR is a
complete computational description — emission is a projection, not
an interpretation.

---

## 2. What went wrong

### 2a. Ideas piled on without design

The v2 development pattern was: build the core → realize the
structure enables something powerful → start shoehorning it in.
Complexity, ownership, effects — each was a genuine insight
(this closed system SHOULD give us these properties for free),
but each was added without first asking: **what substrate change
makes this emergent rather than reconstructed?**

The result: each concern became a separate analysis pass that
reconstructs facts the pipeline already computed and discarded.

### 2b. Node became a god struct

The invariant says "Node is the only recursive type." The intent
was: Node trees are the IR, the compiler operates on Node trees.
In practice, Node accumulated 17 fields mixing structural facts
(children, connective, cardinality) with compiler state (inferred,
is_self_recursive, has_non_tail_self_call, expr_data). This creates
pressure: every new concern wants a field on Node, because Node
is the only thing that flows through the pipeline.

**The root cause:** the insight "everything is a Node" is correct
at the IR level but was applied too literally at the implementation
level. Node should be structural substrate. Compiler-derived facts
should live in external indexed tables, keyed by Node identity.

Current Node fields and their proper homes:

| Field | Structural? | Proper home |
|-------|------------|-------------|
| children, connective, cardinality | Yes | Node (substrate) |
| params, body, return_cardinality | Yes | Node (substrate) |
| span, ident_span | Yes | Node (substrate) |
| ident | Yes | Node (identity) |
| name | No | Delete (use span + source_text_at) |
| inferred | No | CompilerTable<NodeId, InferredNode> |
| is_self_recursive | No | CompilerTable<NodeId, RecursionInfo> |
| has_non_tail_self_call | No | CompilerTable<NodeId, RecursionInfo> |
| expr_data | Maybe | Could argue either way — it's structural (what kind of expression) but also compiler-derived |
| type_annotation | Yes | Node (substrate, authored) |
| transport, uses | Yes | Node (substrate, authored) |
| properties | Yes | Node (substrate, authored) |
| match_pattern | Maybe | Structural, but arguably a compiler concern |

### 2c. TypeBinding is too narrow (construct-discard-reconstruct)

This is the single most impactful design mistake in v2.

```
type TypeBinding { name: String, resolved: Node }
```

Inference computes rich structural facts about every binding:
- **Provenance:** is this value a sub-value of a parameter? Which
  field? How was it derived? (SubValueRelation)
- **Ownership:** who else reads this? What's the fan-out? (OwnershipEdge)
- **Effects:** does computing this value have side effects? (EffectShape)
- **Cost:** how expensive is this computation? (CostExpression)

All of these are computed... and then discarded. Only the resolved
type survives into TypeBinding. Every downstream consumer (CX,
ownership, emission) must reconstruct the facts it needs.

**The numbers tell the story:**
- complexity.dag: 5,130 lines, 33 reconstruction heuristics, 420+
  violations that exist purely because CX can't see provenance
- ownership.dag: 576 lines, string-name matching for fold
  accumulators, blocked on stable binding identity
- 05_emit*.dag: three separate files with scattered per-language
  heuristics that compensate for missing structural facts

### 2d. Complexity was shoehorned, not designed

The insight was correct: in a decidable language with bounded
iteration, complexity is a consequence of the model. You should
get it for free.

The implementation was wrong: complexity was added as a post-hoc
analysis pass that walks the resolved IR and tries to reconstruct
descent evidence. This required:
- 33 heuristic reconstruction points
- 13 string name matches (variable names, method names, field names)
- 8 hardcoded lookup tables
- 5 type introspection callbacks
- 4 thread-through parameter maps

**What "for free" actually requires:** if complexity facts flow
through bindings (like types do), the complexity of any expression
is determined at its binding site — the same place types are
determined. No separate pass. No reconstruction.

### 2e. Ownership was bolted on

Same pattern as complexity. The insight was correct (ownership is
critical for performance, and in a closed system it's derivable).
The implementation was a separate pass that re-walks the IR.

Three layers were identified:
1. Last-use elision — blocked on stable binding identity
2. Post-TCO ownership — done but narrow
3. Borrow propagation — needs LanguageSpec design

Layer 3 is where 90% of the wins live, and it's blocked on a design
that was never done because ownership was an afterthought.

### 2f. The thesis outran the implementation

THESIS.md describes an architecture where:
- Adding a correctness dimension = declare a lattice in std/ + a
  binding-site rule
- User-defined dimensions work the same as built-in ones
- One mechanism enforces all dimensions uniformly

v2 has none of this. Each dimension is a bespoke pass. There's no
generic dimension mechanism. User-defined dimensions are impossible.

The thesis is the right goal. But it was written as if the
implementation supported it, when actually the implementation needs
structural changes to enable it.

---

## 3. The structural gap

### 3a. What the thesis requires vs. what v2 provides

| Thesis requirement | v2 reality |
|---|---|
| Dimensions flow through bindings | TypeBinding has {name, resolved} — no room |
| One mechanism, many dimensions | Separate passes for each dimension |
| Adding a dimension = 1 file | Adding complexity took 5,130 lines |
| User-defined dimensions | Impossible without generic mechanism |
| Coercion = emission | 3 separate emitter files with heuristics |
| Node is pure substrate | Node has 17 fields mixing substrate + compiler state |

### 3b. The construct-discard-reconstruct pattern

This is the architectural anti-pattern. It appears in every concern
that was bolted on:

```
                       ┌─ compute fact ─┐
                       │                │
Inference stage ──────►│  TypeBinding   │──────► downstream
                       │  {name, type}  │        consumer
                       │                │
                       └─ discard fact ─┘
                              │
                              ▼
                    downstream must
                    RECONSTRUCT the fact
                    from type + heuristics
```

Every instance follows the same pattern:
1. Inference computes a rich fact
2. TypeBinding discards everything except the resolved type
3. Downstream pass reconstructs the fact via heuristics
4. Heuristics are incomplete → violations that should be zero

**Instances in v2:**

| Fact | Computed in | Discarded at | Reconstructed in | Cost |
|------|-----------|-------------|-----------------|------|
| Provenance (SubValueRelation) | infer (let-binding, field access, match arm) | TypeBinding | complexity.dag (33 heuristics) | 420+ violations |
| Ownership (fan-out, last-use) | ownership.dag | emit boundary | 05_emit_rust.dag (string key tables) | blocked features |
| Effect shape | declared in std/effects.dag | never carried on bindings | nowhere (not consumed) | no enforcement |
| Cost | declared in std/primitives.dag | never carried on bindings | nowhere (not consumed) | no enforcement |

---

## 4. What v3 needs to be different

### 4a. Extensible bindings (the dimension slot)

The fundamental change: TypeBinding must carry an open set of
dimension values, not just {name, resolved}.

```
type TypeBinding {
  name: String              // identity (or ident:Int after Node.name deletion)
  resolved: Node            // structural type
  dimensions: DimensionSet  // lattice values for each declared dimension
}

type DimensionSet {
  // keyed by dimension declaration in std/
  // each value is a lattice element for that dimension
  provenance: SubValueRelation     // Track 1
  ownership: OwnershipFact         // Stream B
  effect: EffectShape              // std/effects.dag
  cost: CostBound                  // std/primitives.dag
  // ... user-defined dimensions added here
}
```

**The key insight:** the dimension set doesn't need to be a fixed
struct. It can be a type-indexed map — the compiler reads what's
declared in std/ and carries values for each. Adding a dimension
doesn't change TypeBinding's struct definition.

### 4b. Binding-site rules (the computation point)

Each dimension declares how its value is computed at binding sites.
This replaces separate analysis passes.

```dag
// In std/termination.dag (authority for the provenance dimension)
fn provenance_at_let(binding_expr: Node, env: TypeEnv) -> SubValueRelation {
  // this IS the classification, done at the binding site
  // not reconstructed downstream
}

fn provenance_at_match_arm(pattern: MatchPattern, scrutinee: Node) -> SubValueRelation {
  // computed here, carried forward, consumed by CX without reconstruction
}
```

**The binding-site rule IS the analysis.** There's no separate
complexity pass because provenance is already on the binding by the
time CX reads it. CX becomes a consumer (read provenance, compose,
check descent) rather than a reconstructor (re-walk IR, guess
provenance from string names).

### 4c. Node as pure substrate

Node should carry only structural facts (the authored .dag source):

```dag
type Node {
  ident: Int                       // unique identity
  span: SourceSpan                 // source location
  children: List<Node>             // connective children
  connective: Connective           // Conj | Disj | Arrow | None
  params: List<Node>               // function parameters
  body: Node?                      // function/data body
  return_cardinality: Cardinality  // Required | Optional
  type_annotation: Node?           // authored type ascription
  transport: Node?                 // service transport config
  uses: List<Node>                 // resource uses
  properties: List<Node>           // field initialization
  match_pattern: MatchPattern?     // pattern matching
}
```

Compiler-derived facts live in external tables:
```dag
// Compiler tables, not Node fields
CompilerTable<NodeId, InferredNode>     // replaces Node.inferred
CompilerTable<NodeId, ExprData>         // replaces Node.expr_data
CompilerTable<NodeId, RecursionInfo>    // replaces is_self_recursive + has_non_tail_self_call
CompilerTable<NodeId, DimensionSet>     // NEW — carries all dimension values
```

This decouples the substrate (what the user wrote) from compiler
analysis (what the compiler derived). Node stays bounded. Compiler
tables grow without touching Node.

### 4d. Generic dimension mechanism

The compiler shouldn't know about specific dimensions. It should:
1. Read dimension declarations from std/
2. At each binding site, call the dimension's binding-site rule
3. Store the result in the binding's DimensionSet
4. At enforcement points, check the dimension's constraint

```
For each dimension D declared in std/:
  D.lattice: { meet, join, top, bottom }
  D.binding_rule: (binding_site, env) → D.element
  D.enforcement: (element, constraint) → diagnostic?
```

Adding a new dimension = declare these three things. The compiler
carries it. Cost of change: one file in std/.

### 4e. Single emitter architecture

v2 has three emitter files (rust, go, python) with scattered
decisions. The thesis says emission = coercion = mechanical
translation from specs.

v3 needs one emitter that reads LanguageSpec data:
- Type rendering rules → spec data
- Expression syntax rules → spec data
- Ownership/sharing model → spec data (LS-4)
- Operator dispatch → spec data (already done in v2)

The emitter is a generic translator: `(IR, LanguageSpec) → text`.
Adding a language = adding a spec file, not adding an emitter file.

---

## 5. Risk assessment: what could go wrong in v3

### 5a. Over-generalization

The generic dimension mechanism could become its own complexity.
A type-indexed DimensionSet with open extension needs careful design
to avoid becoming a stringly-typed map. The key constraint: each
dimension value is a specific lattice type with specific operations,
not an `Any`.

### 5b. Performance of carrying dimensions

Every binding carries values for every dimension. If there are 5
dimensions and 10,000 bindings, that's 50,000 lattice values flowing
through the pipeline. This may need lazy computation (compute
dimension value only when first consumed) or stratified evaluation
(dimensions computed in dependency order, not all at once).

### 5c. Bootstrap complexity

v3 is a compiler rewrite. Self-hosting means v2 must compile v3's
bootstrap, then v3 compiles itself. The transition needs a clear
migration strategy — not a big bang.

### 5d. The expr_data question

ExprData (21 variants: ExprCall, ExprMatch, ExprLiteral, etc.)
classifies what kind of expression a node represents. Is this
structural (part of the authored source) or compiler-derived?
Arguments for both. Getting this wrong affects the substrate
boundary.

### 5e. Premature abstraction

The generic dimension mechanism must be designed from concrete
instances. CX and ownership are the two known dimensions. Effects
and cost are declared but not consumed. User-defined dimensions
are theoretical. Designing the generic mechanism from only two
concrete instances risks premature abstraction.

**Mitigation:** implement provenance-on-bindings (Track 1) and
ownership-on-bindings in v2 first. Let the concrete instances
teach us what the generic mechanism needs to look like. v3 is
the extraction, not the invention.

---

## 6. Recommended path

### 6a. Don't rewrite — extract

v3 should not be a ground-up rewrite. The v2 pipeline is
structurally sound (pure stages, threaded diagnostics, structural
types). The substrate (Node + Edge + connectives) is correct.
The bounded computation model is correct.

What needs to change is narrow:
1. TypeBinding gains dimension slots
2. Node sheds compiler fields to external tables
3. Binding-site rules replace analysis passes
4. Emission consolidates from 3 files to 1

Each of these can be done incrementally within v2's architecture.
"v3" is the name for the state after these changes land, not a
separate codebase.

### 6b. Sequence by dependency

```
Step 1: Provenance on bindings (Track 1 — already in progress)
  → validates the "carry facts through bindings" pattern
  → dissolves 33 CX heuristics, 420 violations → 0
  → teaches us what the binding-site rule looks like

Step 2: Ownership on bindings (same pattern)
  → second dimension, same mechanism
  → validates that the pattern generalizes
  → dissolves string-key matching in emission

Step 3: Extract the generic mechanism
  → two concrete instances → one abstraction
  → DimensionSet, binding-site rules, generic enforcement
  → this IS v3 (not a rewrite, an extraction)

Step 4: Effects and cost as dimension instances
  → declare lattices in std/
  → write binding-site rules
  → compiler carries them automatically
  → validates user-defined dimensions will work

Step 5: Single emitter
  → with dimensions on bindings, emitter has all facts
  → consolidate 3 files to 1
  → LanguageSpec data drives all decisions

Step 6: Node cleanup
  → remove inferred, expr_data, recursion flags
  → external CompilerTables
  → Node is pure substrate
```

### 6c. The validation criterion

v3 is real when adding a new correctness dimension requires:
1. One lattice declaration in std/
2. One binding-site rule
3. Zero compiler changes

If user-defined dimensions work the same as built-in ones, the
mechanism is general. If they require special compiler support,
iterate on the mechanism until they don't.

---

## 7. Concrete v3 spec sketch (src/v3/spec.dag target)

This is the shape, not the implementation. The implementation
emerges from the extraction path (6a-6c).

```dag
// --- Substrate ---

type Node {
  ident: Int
  span: SourceSpan
  children: List<Node>
  connective: Connective        // Conj | Disj | Arrow | NoConnective
  params: List<Node>
  body: Node?
  return_cardinality: Cardinality
  type_annotation: Node?
  transport: Node?
  uses: List<Node>
  properties: List<Node>
  match_pattern: MatchPattern?
}

// --- Compiler tables (external to Node) ---

type CompilerState {
  inferred: Table<NodeId, InferredNode>
  expr_data: Table<NodeId, ExprData>
  recursion: Table<NodeId, RecursionInfo>
  bindings: Table<BindingId, TypeBinding>
}

// --- Extensible bindings ---

type TypeBinding {
  ident: Int
  resolved: Node
  dimensions: DimensionValues   // carries all dimension lattice values
}

// --- Dimension mechanism ---

type Dimension<L: Lattice> {
  lattice: LatticeOps<L>                           // meet, join, top, bottom
  compute_at_binding: fn(BindingSite, Env) -> L     // the binding-site rule
  check: fn(L, Constraint) -> Diagnostic?           // enforcement
}

// --- Built-in dimensions (instances of the generic mechanism) ---

dimension Provenance: Dimension<SubValueRelation> {
  lattice: sub_value_lattice
  compute_at_binding: classify_provenance       // replaces 33 CX heuristics
  check: verify_structural_descent              // replaces CX pass
}

dimension Ownership: Dimension<OwnershipFact> {
  lattice: ownership_lattice
  compute_at_binding: classify_ownership        // replaces ownership.dag pass
  check: verify_no_aliased_mutation             // replaces ownership diagnostics
}

dimension Effects: Dimension<EffectShape> {
  lattice: effect_lattice                       // already in std/effects.dag
  compute_at_binding: classify_effects
  check: verify_effect_safety
}

// --- User-defined dimensions (same mechanism) ---
// This is the test of the architecture.

dimension SecurityLevel: Dimension<SecLevel> {
  lattice: { meet: min, join: max, top: Secret, bottom: Public }
  compute_at_binding: classify_security
  check: verify_no_declassification
}
```

---

## 8. Exhaustive inventory: what stays, what goes

### 8a. By the numbers

| Metric | Value |
|--------|-------|
| Total .dag files in src/v2/ | 32 |
| Total .dag lines | 38,075 |
| Generated Rust (stage0/) | 55,632 lines across 62 files |
| Structural lines (keep) | ~17,500 (46%) |
| Reconstruction heuristic lines (rewrite) | ~20,575 (54%) |
| Distinct reconstruction sites | 60+ |

### 8b. Per-file breakdown

| File | Lines | Structural | Reconstruction | Keep % |
|------|-------|-----------|----------------|--------|
| 00_core.dag | 1,702 | 1,702 | 0 | 100% |
| complexity.dag | 5,489 | ~390 | ~5,099 | 8% |
| 04_infer.dag | 5,470 | ~2,200 | ~3,270 | 46% |
| 05_emit_rust.dag | 5,894 | ~4,125 | ~1,769 | 70% |
| 05_emit.dag | 3,003 | ~2,100 | ~900 | 70% |
| 02_parse.dag | 4,824 | ~1,930 | ~2,894 | 40% |
| ownership.dag | 635 | 635 | 0 | 100% |
| compile.dag | 1,065 | 1,065 | 0 | 100% |
| languages.dag | 1,163 | ~1,047 | ~116 | 90% |
| 04_types.dag | 992 | ~694 | ~298 | 70% |
| 04_resolve.dag | 992 | ~595 | ~397 | 60% |
| All other .dag | ~6,846 | ~5,017 | ~1,832 | ~73% |

### 8c. Reconstruction anti-pattern inventory

**Hardcoded lookup tables (8 maps + 6 query functions):**
- `expr_child_roles` — 50 lines mapping ExprData variants to child positions
- `wrapper_child_roles` — 12 lines mapping wrapper node children
- `function_size_effects` — 9 lines mapping function names to tree-size effects
- `node_field_roles` — 7 lines mapping field names to structural roles
- `is_child_accessor_in_model()`, `child_roles_for_variant()`,
  `is_tree_size_preserving()`, `is_tree_size_reducing()`,
  `is_property_contraction()`, `is_sub_value_field()` — query wrappers

**String-based name matching (45+ sites):**
- complexity.dag: `param_node_type_expr(n: param).name == "ParserState"`,
  field name checks for `"state"`, `"tokens"`, method name checks
- 04_infer.dag: `func_name == "empty_map"`, `func_name == "lookup"`,
  `method_name == "map_keys"`, `mname == "count"`, `mname == "skip"`,
  `type_name.value == "Some"`, `field_type_name == "List"/"Set"/"Map"`
- ownership.dag: `fname == "fold"`, `authored_name_at(...) == "init"`
- 05_emit_rust.dag: field name matches for `"naming"`, `"from_key"`,
  `"exit_nonzero"`, variant name `"SnakeCase"`, method name `"skip"`

**Classification/reconstruction functions (30+ major functions):**
- complexity.dag: `classify_recursion_pattern()` (35+ lines),
  `classify_self_call_evidence()`, `classify_scc_recursion_pattern()`,
  `classify_parser_scc_recursion_pattern()`, `is_child_descent_expr()`,
  `is_tree_size_preserving_wrapper()`, `is_sub_value_extractor()`,
  `is_list_shrink_expr()`, `is_tokens_consuming_call()`,
  `is_tokens_input_expr()`, `is_descent_arg()`
- 04_infer.dag: `classify_field_recursion()`, `classify_argument()` (80+ lines),
  `classify_binding_provenance()`, `classify_let_value()`,
  `classify_call_via_provenance()`, `classify_call_arg_provenance()`,
  `classify_body_provenance()` (60+ lines), `annotate_descent()`,
  `annotate_pattern_parent_enums()`

### 8d. The worst offenders

1. **complexity.dag** — 92% reconstruction. The cost algebra (SizeExpr,
   CostExpr, composition rules) is 390 lines of real domain logic.
   The other 5,099 lines are heuristic dimension guessing, pattern
   matching on expressions, and fallback searches. Line 2861:
   "Unknown type: fallback to trying all dimensions."

2. **04_infer.dag classify_argument()** — 80+ lines that walk expression
   trees to reconstruct descent evidence that was available at the
   binding site. This single function embodies the construct-discard-
   reconstruct pattern: inference computed the SubValueRelation,
   TypeBinding discarded it, classify_argument() reconstructs it.

3. **02_parse.dag** — 60% heuristic. Integer position indexing creates
   opaque tokens that CX can't prove terminate. 132 CX violations
   exist purely because the parser representation defeats structural
   descent proofs. Stream D (list consumption) is mechanically done
   but CX can't see through helper returns without output provenance.

---

## 9. Strategic question: refactor v2 or redesign v3?

Two paths to the same goal. The question is cost.

### Path A: Incremental refactor (current plan)

Continue Track 1 (provenance on bindings) → ownership on bindings →
extract generic dimension mechanism → single emitter → Node cleanup.

**Pros:**
- Working compiler throughout. No bootstrap gap.
- Each step is testable against the 393-test suite.
- Hard-won edge cases (service reconciliation, pattern coverage,
  variant disambiguation) are preserved.
- 46% of the code (17,500 lines) stays as-is.

**Cons:**
- Track 1 has been in progress for months. S1-S7 partially done.
  Refactoring while maintaining backward compatibility is slow.
- Each step touches multiple stages simultaneously (infer creates
  bindings, CX reads them, emit consumes them). Cross-cutting changes
  in a 38K-line self-hosted compiler are expensive.
- The "refactor while running" constraint means you can never make
  a breaking substrate change — you must maintain both old and new
  paths until the migration is complete.
- Risk of permanent incrementalism: always refactoring, never arriving.

**Estimated cost:** Track 1 alone was estimated at ~1,500 lines net
dissolution. At current velocity, that's weeks of agent sessions.
Full path (through single emitter) is months.

### Path B: Clean redesign (spec-first)

Write src/v3/spec.dag as a complete specification. Design the
substrate (Node, TypeBinding, DimensionSet, CompilerTable) with all
known concerns from day one. Then implement the pipeline against
the spec, porting the 17,500 structural lines and rewriting the
20,575 reconstruction lines as clean consumers.

**Pros:**
- No backward compatibility constraint. Breaking substrate changes
  are free.
- The spec can be reviewed and validated before implementation.
  Design mistakes are caught in the spec, not in the 10th refactor
  session.
- Complexity, ownership, effects are designed in from the start.
  No reconstruction heuristics. No classify_argument().
- 46% port + 54% clean rewrite of ~20K lines may be faster than
  incrementally migrating ~20K lines while keeping them working.
- The spec IS the documentation. No divergence between design docs
  and implementation.

**Cons:**
- Bootstrap gap: v2 must compile v3's first stage0, then v3 must
  self-host. The transition is a project.
- Risk of losing hard-won edge cases that aren't documented but are
  encoded in the 20K lines of heuristics. Some of those heuristics
  capture real complexity that a clean design must also handle.
- Risk of over-design: spending weeks on a spec that turns out to
  need revision once implementation starts.
- The 17,500 "structural" lines may not port cleanly if the substrate
  changes enough.

**Estimated cost:** Spec design (1-2 weeks of focused sessions).
Implementation (~3-4 weeks porting structural code + writing clean
dimension consumers). Bootstrap (~1 week).

### Path B.1: Hybrid — spec then implement inside v2

Write the spec (the thinking work). But implement it as a v2
refactor, not a separate codebase. The spec guides which substrate
changes to make and in what order. But the changes land in v2,
preserving the test suite and bootstrap.

This gets the design clarity of Path B with the continuity of Path A.
The spec answers "what is the target state?" The implementation is
still incremental, but now each increment knows exactly where it's
going.

**This may be the right path.** The problem with Track 1 wasn't
incrementalism — it was incrementalism without a complete target
spec. Each step discovered new design questions that weren't
answered yet. A spec that resolves all design questions up front
makes the incremental path predictable.

---

## 10. Summary: the three lessons

1. **Design concerns into the substrate, don't bolt them on.**
   Complexity, ownership, effects — each was a correct insight
   about what a closed system should provide. Each was added without
   the substrate change that would make it emergent. The result:
   reconstruction passes that duplicate work the pipeline already did.

2. **The IR must carry facts from computation to consumption.**
   TypeBinding {name, resolved} is too narrow. Every downstream
   consumer that needs more than the type must reconstruct. The fix:
   dimension slots on bindings, computed at binding sites, consumed
   downstream without reconstruction.

3. **Generalize from concrete instances, not from theory.**
   The thesis describes a beautiful generic dimension mechanism.
   v2 tried to get there by bolting on specific instances. v3 should
   get there by implementing provenance and ownership concretely on
   bindings, observing the pattern, and extracting the generic
   mechanism. The abstraction follows the instances, not the other
   way around.
