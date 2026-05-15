# CM: Compiler Concept Modeling

The compiler works but doesn't model what it does. Every stage
re-derives structural facts that earlier stages already knew, producing
heuristic if-else forests that get shuffled around but never eliminated.
This document catalogs the missing models, their symptoms, and design
directions. The goal is to spend time getting the modeling right.

See also: `DESIGN.md` (principles), `ROADMAP.md §CM` (summary table).

---

## Foundational ontology

The concept DAG (computation.dag §LAYERS) has two foundational
categories modeled as std/ primitives:

| Category | Captures | std/ home | Math grounding |
|----------|----------|-----------|----------------|
| **Signal** | What holds — assertion, state | logic.dag, bit.dag | Classical bivalent logic |
| **Algebra** | How things combine — structure, laws | algebra.dag | Abstract algebra (Monoid → Field) |

These are categorically distinct. A Bit is a signal (hi/lo). A
Monoid is a bag (elements accumulate under laws). Neither reduces
to the other. They compose (BooleanAlgebra is truth values with
algebraic structure) but remain independent primitives.

A third category is missing: **what happens.** Function calls,
transports, computation steps — these produce signals and consume
bags, but the transformation itself is neither. The compiler
*performs* this third category (that's what compilation is) but
doesn't *model* it.

### Working candidate: Reduction

From term rewriting systems (Knuth, Bendix 1970) and lambda calculus
(Church 1936). **β-reduction** (`(λx.M)N → M[N/x]`) formalizes
"a computation step" — a term matching a rule is replaced by its
result. This has well-developed theory: Church-Rosser (confluence),
strong normalization, completion algorithms.

Connection to the existing concept DAG:

| Existing concept | Reduction perspective |
|-----------------|----------------------|
| Function definition | A **rewrite rule**: `f(x) → body[x/param]` |
| Function call site | A **redex** (reducible expression): matches a rule's LHS |
| Function application | A **reduction step**: match, substitute, produce result |
| Normal form | Fully evaluated — no more redexes (within .dag evaluation) |
| fold/descend/repeat | **Rewrite strategies**: the order in which reductions are applied |
| Termination proofs | Standard rewrite-termination via ranking functions (already in termination.dag) |
| Compilation | **Translation** (functor), not reduction — see §Known gaps |

Reduction is not a concept invented for this compiler. It is how
lambda calculus formalizes "computation step" — the same way
Classical formalizes "assertion" and Monoid formalizes "combination."

### Implications for function calls (MM-3)

The ref-vs-call distinction becomes: **is this term a redex?**

| Current (name-based) | With reduction (structural) |
|-----------------------|----------------------------|
| `f(arg)` = ExprCall (surface syntax category) | `f(arg)` = redex (matches a rewrite rule) |
| `f` ref vs `f()` call — ambiguous for nullary fns | arity=0: redex (reducible now). arity>0: value (needs args) |
| 3 ad-hoc nullary detection sites in emit_rust | Redex-vs-value: structural property on the expression |
| 21 method name-dispatch sites | Method = named rewrite rule; dispatch on rule structure |
| Go/Python ignore binding_kind (bug) | All backends read the same structural fact |

Transport unification: an internal function is a rewrite rule with a
body (internal reduction). A transport is a rewrite rule grounded
externally (external reduction). Same concept, different grounding
source. This connects to Evidence in §Foundational principle below.

### Open questions

- **Is reduction the right grounding?** Alternatives considered:
  modus ponens (too narrow — only implication elimination),
  eval morphism (requires CCC infrastructure), cut rule (more
  general but less operational). Reduction chosen as working
  candidate for function application / computation steps.
  **Caveat:** compilation itself is translation (structure-preserving
  map between representations), not evaluation (reduction to normal
  form). The emitter is a functor, not a reducer. See §Known gaps.
- **std/ representation?** Possibly `std/reduction.dag` with
  Redex, RewriteRule, NormalForm. Must connect to Layers 4-6
  (iteration primitives are rewrite strategies, call patterns
  are redex classifications).
- **Relationship to Morphism?** The Morphism property
  (§Foundational principle) may be the static description
  ("this Node is a rewrite rule") while reduction is the dynamic
  description ("applying this rule is a computation step").

### Known gaps: where S/A/R doesn't help

Stress-testing against the full heuristic inventory (CM-inventory.md)
reveals three areas the ontology does not cover:

**Gap 1: Naming/Reference** — how you find the value

Variable lookup, scope threading, alias resolution. These are about
the relationship between a symbol and its referent, not about what
the value IS or how it composes or what happens to it.

| Problem | Math structure |
|---------|---------------|
| Variable lookup | Partial function application: `Name → Binding` |
| Scope/environment | Monoid action: environment acts on free variables |
| Alias resolution | Union-find: equivalence class representative |

In sequent calculus terms, S/A/R covers the right side of the
turnstile (`⊢ e : T`). Naming covers the left side — the context
`Γ` that makes the judgment meaningful.

**Gap 2: Multiplicity** — how many times a value is used

Fan-out counting, clone/move/borrow decisions, sharing strategy.
The entire `ownership.dag` computes which structural rules of linear
logic apply to each binding:

| Structural rule | Fan-out | ownership.dag |
|----------------|---------|---------------|
| Weakening | 0 | Dead code |
| Linearity | 1 | Move |
| Contraction | >1 | Rc + clone |

Math: linear logic (Girard 1987), substructural type theory. This
is a modality on the logic (the exponential `!A`), not a new
connective — it modifies HOW Signal/Algebra/Reduction work, rather
than being a peer of them.

**Gap 3: Maps between structures** — relating different algebras

Coercion, TypeRendering, type equality. These are about relationships
*between* algebraic structures, not operations *within* one:

| Problem | Math structure |
|---------|---------------|
| Coercion (Int → i64) | Functor: structure-preserving map between categories |
| TypeRendering | Forgetful functor: lossy projection with ad-hoc characterization |
| Type equality | Decidable congruence on Node |

**Crucially: compilation itself belongs here, not under Reduction.**
The emitter translates between representations (a functor from .dag
algebra to target algebra). It does not evaluate/reduce programs.
This aligns with the existing invariant: "Emission is translation,
not decision-making."

**Open:** Do these three gaps need their own std/ primitives, or are
they consequences of Signal/Algebra/Reduction applied at different
levels? Multiplicity may be a modality (orthogonal modifier).
Naming may be the application-of-partial-functions aspect of
Algebra. Functors may generalize Reduction to cross-structure maps.

### Where CM does NOT help: fail-open analysis

~39 sites in the inventory return fabricated defaults (unit_type,
empty string, false) when data is missing. Stress-testing these
against the CM analysis:

| Category | Sites | CM helps? |
|----------|-------|-----------|
| Upstream data genuinely missing | bare_map_node contamination (04_resolve, 04_access), builtin registry (04_method) | **Yes** — modeling eliminates the fallback path |
| Bridge-era sentinels | Callable/Dynamic/Error suppression, rt_type policy | **Indirectly** — dissolves when builtins become .dag definitions |
| Defensive: unreachable post-inference | resolve_optional_node `None => unit_type` | **Not via modeling** — path is unreachable; function signature is too wide |
| Defensive: classification exhaustiveness | TypedItemUnhandled | **Not via modeling** — if-else can't prove completeness |
| Runtime semantics | char_at `""` on out-of-bounds | **No** — deliberate behavior choice |

The genuine modeling gaps (~3 sites) trace to **one upstream root
cause**: `bare_map_node()` from inference not propagating expected
types through fold accumulators. This is the M2 open item already
on the ROADMAP. CM confirms but doesn't add to that diagnosis.

**However: "defensive programming" is itself a modeling question.**

The two defensive sites ARE structurally eliminable — at the
type-system level rather than the data level:

- `resolve_optional_node` accepts `InferredNode?` but is only
  called post-inference where `.inferred` is always `Some`. If
  the function required `InferredNode` (non-optional), or if
  post-inference Nodes were a distinct type with `.inferred`
  guaranteed, the None path would be unrepresentable.

- `TypedItemUnhandled` exists because an if-else chain can't
  prove exhaustiveness. If the connective × body × transport
  product space were a finite sum type with exhaustive match,
  the unhandled case would be unrepresentable.

The principle: every defensive guard is a place where the type
system can't prove what the programmer knows. The goal is not
"never write guards" but "push the proof into the structure so
the guard becomes a compiler error, not a runtime default." Some
guards are eliminable today (tighten function signatures); some
require richer type modeling (phase-indexed Nodes, finite product
decomposition); some are irreducible (runtime bounds checks on
external input).

### Node and DAG as derived structure

Node and DAG are treated as foundational ("Node is the only recursive
semantic authority" — ROADMAP). But they are not primitives. They are
compositions of the three foundational categories:

**Node** = Product of:
- **connective**: Signal (Conj | Disj | NoConnective — structural assertion)
- **children**: FreeMonoid<Node> (Algebra — bounded recursive collection)
- **fields** (params, type_annotation, properties): Product (Algebra — conjunction of facts)
- **body | transport**: Reduction (rewrite rule — internal or external, or absent)

**DAG** = FreeMonoid<Node> + PartialFunction<Node, FreeMonoid<Node>> where acyclic
- Nodes form a collection (FreeMonoid)
- Edges are adjacency (PartialFunction)
- Acyclicity is a refinement constraint (same mechanism as `where non_empty`)

The bounded kernel invariant restated: Node.children is FreeMonoid<Node>.
FreeMonoid is bounded by construction (finite list). The "only recursive
type" property falls out of using a bounded algebraic structure as the
carrier for self-reference — it's a theorem, not an axiom.

**Connection to MM-1:** If Node decomposes along Signal/Algebra/Reduction,
then item classification is already answered by the structure:

| Sugar | Signal (connective) | Algebra (fields) | Reduction |
|-------|---------------------|-------------------|-----------|
| `type` | Conj or Disj | structure fields | absent |
| `fn` | NoConnective | params, return | body (internal) |
| `service` | Conj | config, children | transport (external) |
| `resource` | Conj | capabilities | external identity |
| `data` | NoConnective | type constraint | body (ground term) |

The classification forests (MM-1) exist because the compiler treats
Node as opaque and re-derives this decomposition. If the decomposition
were the API — if consumers asked "does this have Reduction?" instead
of "is this a function?" — the forests dissolve.

### Validation: heuristic forests map to ontological categories

Every heuristic forest in the per-file inventory (§below) asks about
one or two of the three categories. The reason the questions are hard
is impedance mismatch: the Node API exposes individual fields (`body`,
`transport`, `connective`, `params`, `uses`...) but the questions are
ontological ("has Reduction?", "what kind of Algebra?"). The consumer
reconstructs the category from raw fields every time.

**MM-1 forests** — all four ask the same three questions:

| Question | Category | How they currently ask |
|----------|----------|----------------------|
| Has structure? | Algebra | `connective != NoConnective` |
| Has a rewrite rule? | Reduction | `body != none`, `transport != none` |
| What grounding? | Reduction + Signal | `uses |> count > 0`, `transport != none` |

The `item_kind` priority chain is: check Algebra (→ TypeItem), then
Reduction (→ ServiceItem/FnItem/FuncItem/DataItem). Four forests exist
because no one projection serves all consumers — but the source data
(Algebra? Reduction? What kind?) is identical across all four.

**MM-2 sites** — almost entirely Algebra:

| Site | Question | Category |
|------|----------|----------|
| `field_summary_for_type` | Conj → struct, Disj → enum | Algebra (product/coproduct) |
| `build_type_summary` | StructRepr vs EnumRepr | Algebra |
| `render_type_base` (97 lines) | 7-way string/field dispatch | **Lost Algebra** — connective erased by TypeRendering |
| `build_type_rendering` "Callable" | `n.name == "Callable"` | Reduction ("is this a function type?") |

The 97-line `render_type_base` exists because TypeRendering is a lossy
boundary: it flattened Algebra (connective) into string names and
optional fields. If TypeRendering carried Algebra structurally, most
branches collapse. The "Callable" special case is Reduction leaking
through — "is this the type of a rewrite rule?"

**MM-3 sites** — Algebra + Reduction:

| Site | Question | Category |
|------|----------|----------|
| 13-way method dispatch | What rendering for this operation? | Algebra (operation kind) |
| Method result type (7-way) | What type does this produce? | Algebra (map/filter/fold semantics) |
| Call tier classification | Direct call? Method? Built-in? | Reduction (what kind of application?) |
| Nullary detection (3 sites) | Append `()`? | Reduction (is this a redex?) |
| Transport predicates | rest/shell/file/local? | Reduction (what kind of external rule?) |

**Conclusion:** The three ontological categories are not speculative.
They are exactly the three axes every heuristic forest already asks
about. The forests exist because the compiler lacks the vocabulary.

Full heuristic-by-heuristic analysis with modeling implications:
[`CM-inventory.md`](CM-inventory.md)

---

## The three missing models

Everything below reduces to three structural gaps in the compiler IR.

### MM-1: Item identity ("what kind of thing is this Node?")

**The gap:** The parser produces uniform `Node` for types, functions,
services, resources, data defs. Every downstream stage re-derives item
kind from the same bag of fields: `{connective, body, transport, params,
children, uses, type_annotation, properties}`.

**Symptoms:** Four independent classification forests, three with
different output taxonomies:

| Stage | Location | Branches | Output |
|-------|----------|----------|--------|
| Item analysis | `04_items.dag:114-131` | 8 | `ItemKind` (FnItem/FuncItem/TypeItem/DataItem/ServiceItem/OtherItem) |
| Type env registration | `04_infer.dag:2270-2326` | 5 | Registration strategy (full type / alias / nominal / parameterized / resource) |
| Emit classification | `04_emit_info.dag:75-104` | 9 | `TypedItemKind` (Struct/Enum/TypeAlias/TypeDecl/Function/TransportFunction/DataDef/ServiceDef/ResourceDef/Unhandled) |
| Value constancy | `04_infer.dag:2972-3050` | 5 | Boolean (is_constant) |

Plus downstream consumption: name-keyed `Map<String, TypedItemKind>`,
name-keyed `Map<String, FunctionSignature>`, etc. — each with fail-open
fallbacks that fabricate defaults on miss.

**The question:** What are the irreducible structural facts about an
item? Not "is this a function?" (interpretation) but "does it have a
body? does it have a transport? does it have structure?" The fields
themselves ARE the facts. The four taxonomies are redundant
interpretations that exist because no single taxonomy felt complete.

**Design direction:** Stop classifying. The fields (`body`, `transport`,
`connective`, `params`, `uses`) ARE the structural facts. Consumers
should pattern-match on the facts they need directly — "does this have
a body?" not "is this a FnItem?" Adding a `ClassifiedItem` wrapper
would create a duplicate authority alongside the existing fields.
The fix is making the existing fields reachable at every consumption
site, not wrapping them in a new taxonomy.

**Subproblems:**
- `is_type_alias_return_node` special-cases string `"Unit"` (`04_emit_info.dag:69`)
- `TypedItemUnhandled` is a fail-open sentinel — should be impossible by construction
- ServiceFieldSet / FunctionSignature / ResourceDefinition stored in name-keyed side-tables

---

### MM-2: Type structure ("product, sum, or leaf?")

**The gap:** `Conj/Disj/NoConnective` is a primitive, but its
*interpretation* (product → struct rendering, sum → enum rendering,
leaf → primitive lookup) is re-derived at every site.

**Symptoms:** 277 `Conj/Disj/NoConnective` references across .dag files.
Key sites:

| Location | What it decides |
|----------|----------------|
| `04_lookup.dag:209-294` | Field access resolution path |
| `04_emit_info.dag:607-633` | TypeSummary (StructRepr vs EnumRepr) |
| `05_emit.dag:1095-1191` | 97-line type rendering dispatch |
| `04_types.dag:367-415` | Type equality |
| Every `emit_typed_item` | Struct vs enum code generation |
| `complexity.dag` | Structural analysis |

**Design direction:** The connective primitives stay — they're
foundational. The connective IS the authority. `TypeSummary.repr`
(StructRepr/EnumRepr) already exists as a downstream consumer of
connective — the question is whether all consumers should read
connective directly (the minimal model) or whether TypeSummary.repr
should become the single derived consumer that others go through.

Adding a new TypeShape enum would create a third authority alongside
connective and TypeSummary.repr — a duplicate representation
violation. The fix is to make connective reachable to every consumer
that currently re-derives it, not to add another interpretation layer.

---

### MM-3: Expression semantics ("what does this reference/call do?")

**The gap:** Method and function identity is a string. Every consumer
dispatches on name.

**Symptoms:**

| Location | Branches | Dispatch |
|----------|----------|----------|
| `05_emit_rust.dag:2853-2901` | 13 | Algebra method name (fold/map/filter/sort_by/...) |
| `04_infer.dag:842-903` | 7 | Collection method result type |
| `04_infer.dag:1215-1258` | 4 | Built-in call type refinement (lookup/map_get/...) |
| `04_infer.dag:1130-1189` | 5 | Call tier (known sig → method bridge → special case) |
| `complexity.dag` (distributed) | 5+ | Method classification for progress analysis |
| `05_emit_rust.dag:1748-1750` | 3 | Nullary function ref → append `()` |

**Subproblem — nullary invocation (from CG review):**

In DAG, referencing a nullary function IS invoking it — no `foo()` vs
`foo` distinction. Target languages need `()`. Currently the emitter
re-derives this from type names or registry lookups (`04_emit_rust.dag`
lines 1685-1750). Inference already knows at `04_infer.dag:970-977`:
`binding_kind = FunctionValueBinding` + `fsig.params |> count == 0`.

**Design direction:** Normalize ExprVar → ExprCall during inference
when the reference IS an invocation. The expression IR is the authority
for invocation semantics — the graph should represent what the program
DOES, not what the surface syntax LOOKS like. Inference already knows
(`binding_kind = FunctionValueBinding` + zero params); the fix is to
act on that knowledge by rewriting the expression node.

Adding `NullaryCallBinding` to `VarBindingKind` would encode invocation
semantics on binding classification (a workaround) instead of the
expression IR (the root cause). This violates Root-Cause Depth — the
fix belongs at the expression level, not the binding level.

Note: Go and Python emitters completely ignore `binding_kind` today —
they destructure with `_` and emit bare identifiers. They have the
same nullary bug. ExprVar→ExprCall normalization fixes all three
backends at once because the authority is in the expression, not the
binding.

**Design direction for method dispatch:** The `method_def` Node and
the algebra framework (`AlgebraProfile`, `AlgebraFieldTemplate`)
already carry method identity structurally. The fix is to surface
these existing authorities to downstream consumers (emit, complexity)
through the pipeline boundary, not to add a parallel
`AlgebraMethodKind` enum. Adding a new dispatch enum would duplicate
the existing method_def authority — a No-duplicate-representations
violation. The method_def Node IS the method identity.

---

## Per-file heuristic inventory

Dense reference for when we start fixing these.

### 04_emit_info.dag (662 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 65-70 | `is_type_alias_return_node`: `n.name != "Unit"` | MM-1: name-driven alias detection |
| 75-104 | `classify_typed_item`: 9-branch if-else | MM-1: item classification |
| 107-111 | `lookup_item_kind`: `None => TypedItemUnhandled` | MM-1: fail-open fallback |
| 127-152 | `classify_service_fields`: transport scanning | MM-1: service facts |
| 155-159 | `lookup_service_fields`: fabricates `{false,false,false,false}` | MM-1: fail-open |
| 607-633 | `build_type_summary`: connective dispatch | MM-2: type structure |

### 04_items.dag (250 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 114-131 | `classify_item`: 8-branch if-else | MM-1: item classification (duplicate #2) |
| 83-112 | Output derivation: connective + name checks | MM-2: type structure |

### 04_infer.dag (3159 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 842-903 | Collection method result type: 7-way name match | MM-3: method identity |
| 1008-1070 | Field access resolution: 5-branch cascade | MM-2: type structure |
| 1130-1189 | Call tier classification: 5-branch cascade | MM-3: call identity |
| 1215-1258 | Built-in call refinement: 4-way name match | MM-3: method identity |
| 1642-1670 | Lambda element type threading: 5 branches | MM-3: callable shape |
| 2270-2326 | Type env registration: 5-branch classification | MM-1: item classification (duplicate #3) |
| 2526-2527 | `is_zero_arg_func`: re-derived from params | MM-3: nullary invocation |
| 2972-3050 | `is_item_constant`: 5-branch value analysis | MM-1/MM-2: item + type structure |

### 05_emit_rust.dag (4598 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 1685-1750 | `emit_var_ref`/`emit_typed_expr_base`: 3-tier nullary detection | MM-3: nullary invocation |
| 2286-2338 | Index dispatch: string-like vs map vs list | MM-2: type structure (string-like) |
| 2853-2901 | Algebra method dispatch: 13-way name match | MM-3: method identity |
| 2944-2957 | Match scrutinee Rc handling: 4 branches | MM-2: ownership shape |
| 3101-3123 | Field optionality: 3 fallback attempts | MM-2: field metadata |
| 3170-3217 | Variant parent lookup: 4 fallback paths | MM-1: variant identity |
| 3219-3239 | Struct name from field set: heuristic matching | MM-1: type identity |
| 3881-3897 | Shell return type dispatch: 4 branches | MM-2: type structure |

### 05_emit.dag (1715 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 922-930 | `"Callable"` name check for type rendering | MM-2/MM-3: callable type |
| 1095-1191 | Type rendering dispatch: 97 lines, 7 branches | MM-2: type structure |

### 05_emit_go.dag / 05_emit_python.dag

| Lines (Go/Py) | Heuristic | Missing model |
|-------|-----------|---------------|
| 371-406 / 363-406 | `emit_*_typed_item`: kind dispatch + fail-open `""` | MM-1: item classification |
| 609 / 591 | `emit_*_expr_var`: ignores `binding_kind` entirely | MM-3: nullary invocation (bug?) |
| 822-843 / 801-824 | Index dispatch: string-like check | MM-2: type structure |
| 373 / 366 | `unit_only` lookup: `None => false` | MM-2: fail-open fallback |

### 00_core.dag (1441 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 1038-1041 | Transport predicates: `t.name == "rest"` etc. | MM-1: transport identity (4 string comparisons) |
| 1060-1092 | Transport property accessors: string keys | MM-1: transport structure |

### complexity.dag (4363 lines)

| Lines | Heuristic | Missing model |
|-------|-----------|---------------|
| 566-580 | `extract_base_var_name`: pattern-match on ExprData | MM-3: expression identity |
| 737-765 | `classify_call_progress`: witness-based dispatch | MM-3: call semantics |
| 1393-1500+ | `max_path_self_calls_with_cont`: ExprData dispatch | MM-3: expression semantics |
| distributed | `mname == "first" || mname == "last"` repeated | MM-3: method identity |

---

## Cross-cutting patterns found in open PRs / recent main

These aren't new problems — they're the same three models showing up in
every change.

**PR #321 (CG lane):** Moved `has_fn_fields` and `is_constant` into
TypeSummary (good — dissolves re-derivation). But `variant_to_enum` map
still uses name-key with empty-string sentinel. Name-string dispatch on
"Callable"/"Refined"/"Tuple" persists in 05_emit.dag.

**PR #318 (CX lane):** Added complexity helpers (`is_sub_value_extractor`,
`is_tree_size_preserving_wrapper`) that dispatch on callee name strings.
`fielded_variants` side-table keyed by `concat(parent, "::", name)`.

**Recent main commits:**
- `da10ff482`: ValueContext/is_constant precomputation — right CM
  direction. But `is_string_like` still dispatches on type name.
- `027c637f8`: Transport consolidation (ServiceFieldSet, TransportKind
  enum) — good structural fix. Corrected Go/Python op_children bug that
  existed because of incomplete interrogation.
- `ff7c7e2e1`: Fold accumulator unwrap lifted from emit to ownership —
  right direction (analysis in compile phase, not rendering phase).
  Eliminated 4 ad-hoc AST walks from emit.

**Pattern:** Every PR that touches emit either adds new heuristics or
moves existing ones upstream. The upstream moves are progress, but
without the target models defined, the heuristics just relocate.

---

## Modeling gaps in dsl/std vs compiler

The `dsl/std/` and `dsl/extdeps/` layers define structural facts that
the compiler partially uses:

**Used correctly:**
- Algebra profiles (`kernel_algebra_profile`) → single authority
- Container templates → via LanguageSpec
- Reserved words / escape conventions → imported by emit

**Re-declared or hardcoded:**
- Type/keyword mappings redeclared in `05_emit.dag` (import collisions block using extdeps directly)
- Sharing strategy hardcoded in `v2/languages.dag` (should live in extdeps/languages/rust/)
- Test conventions hardcoded in `v2/languages.dag` (configuration, not derivation)
- Serde attributes pass through three-layer fallback chain

**Not imported but should be:**
- Transport protocol contracts (TransportRequest/Response shapes in std/types.dag)
- Platform/target model (TargetTriple, RuntimePlatform)

---

## Acceptance criteria

Each criterion is a structural claim: "this class of mistake is
**unrepresentable**." Not "we haven't written it" but "the types and
module boundaries prevent it from being written."

**Governing constraint (from review):** Every acceptance criterion
must resolve to ONE clear end-state. The existing authorities
(Node fields, connective, method_def, AlgebraFieldTemplate) are the
ground truth. The fix is making them reachable at every consumption
site. If a boundary needs to carry facts to a consumer, it must be
exact and non-lossy — no stripping of data the consumer needs, no
new parallel interpretation types. This means:
- No new `ItemKind`/`TypeShape`/`AlgebraMethodKind` classification types
- No lossy boundary that drops distinctions downstream consumers need
- The existing structural data flows through stage boundaries intact

### MM-1: Item re-classification dissolves

**Problem:** Four classification forests re-derive item kind from
raw Node fields because the fields aren't reachable at the boundary.

**End-state:** The existing structural fields (`body`, `transport`,
`connective`, `params`, `uses`) flow through the infer→emit boundary
intact. Emit reads them directly — not from side-tables, not from a
parallel classification type, not re-derived from heuristics. The
four classification forests dissolve because every consumer reads the
same structural facts from the same Node.

**Constraint:** This does NOT mean emit performs ad-hoc semantic
interpretation of raw fields. The fields themselves are the semantic
facts (body = has internal evidence, transport = has external
grounding, connective = structural composition). Reading a field is
reading the authority, not re-deriving it.

**Edge cases (from analysis):** Some field combinations are
semantically ambiguous (empty service, resource with properties but
no capabilities, fn/interface/pattern producing identical shapes).
These should be resolved at the parser or normalizer — not left for
downstream to improvise. See §Arity boundaries below.

### MM-2: Connective re-interpretation dissolves

**Problem:** 57 sites re-derive "product or coproduct?" from
connective.

**End-state:** Connective is the authority. `TypeSummary.repr`
(StructRepr/EnumRepr) is the existing derived consumer for emit.
The fix is extending `TypeSummary.repr` to cover all type rendering
(not just type definitions). `TypeSummary.repr` is only safe as the
single authority if it is proven non-lossy — it must preserve every
distinction that lookup, types, complexity, and ownership need, not
just emit. If it collapses distinctions that other consumers need,
those consumers should read connective directly.

**Constraint:** No new TypeShape enum. Either connective or
TypeSummary.repr is the authority at each site, never both.

### MM-3: Method name dispatch dissolves

**Problem:** 21 sites dispatch on `method_def.name` strings.

**End-state:** The `method_def` Node and `AlgebraFieldTemplate`
already carry method identity (parameter types, return type, receiver
shape). Emit reads these structural facts instead of matching on
name strings. Nullary function invocation is ExprCall in the IR
(normalized during inference), not detected by type-name checks.

**Constraint:** The algebra framework IS the method identity
authority. The fix is surfacing `AlgebraFieldTemplate` facts through
the pipeline — not adding a parallel enum. For nullary calls, the
expression IR is the authority (ExprVar→ExprCall normalization).

---

## Arity boundaries: where the model is silent

Open-ended modeling (Node with optional fields, containers with
variable children) produces a combinatorial space much larger than
the set of semantically valid configurations. At the boundaries —
arity 0, arity > expected, ambiguous field combinations — the model
is silent and code improvises. Each improvisation is a future refactor.

### Node field combinations

Three keywords (`fn`, `interface`, `pattern`) produce identical field
shapes. The keyword dissolves after parsing — downstream can't
distinguish intent. Two independent classifiers (`item_kind` in
04_items.dag, `classify_typed_item` in 04_emit_info.dag) operate
on the same Node field space with different priority chains and
**can disagree** (resource with properties but no capabilities:
item_kind → OtherItem, classify_typed_item → ResourceDef).

Ambiguous combinations the parser produces but nobody specified:

| Combination | Parser accepts? | Semantic question |
|-------------|----------------|-------------------|
| Service with 0 operations | Yes | Valid (forward decl)? Or error? |
| Resource with 0 capabilities | Yes | Valid? Hits TypedItemUnhandled |
| fn with only type params, no value params | Yes | Body can't use params at value level |
| interface → identical shape as fn | Yes | Intent lost after parsing |

**Principle:** Each ambiguous combination should be resolved at the
parser or normalizer — not left for downstream to improvise. If the
combination is valid, its semantics must be specified. If invalid,
the parser should reject it (construction over ratchets).

### Container arity boundaries

`container_type_arity` covers List(1), Map(2), Set(1) but is silent
at the boundaries:

| Scenario | What happens | Should happen |
|----------|-------------|---------------|
| `Map<String>` (under-arity) | Silently filled to `Map<String, Unit>` | Diagnostic or expected-type propagation |
| `List<Int, String>` (over-arity) | Falls through all predicates | Diagnostic: arity mismatch |
| `Optional<Optional<T>>` | Collapses to `Optional<T>` silently | Document or diagnose (idempotent optionality) |
| Callable/lambda param mismatch | Excess gets TypeVariable | Diagnostic: arity mismatch |
| `index` on FreeMonoid | Declared as T (non-optional) | Should be T? (partial function) |

Types not in `container_type_arity` at all: Optional, Callable,
Tuple, FreeMonoid, PartialFunction, BooleanAlgebra. These have no
arity validation even for the common case.

**Connection to fail-open:** The `bare_map_node()` chain (§fail-open
analysis above) is a specific instance of this pattern. When arity
is unvalidated, under-parameterized types propagate through the
pipeline and force downstream fallbacks.

**Principle:** Arity constraints should be declared once (in the
algebra profile or type declaration) and enforced at normalization.
Over-arity and under-arity should produce diagnostics, not silent
patches. The algebra profile already carries parameter counts — the
normalizer should read them.

---

## Foundational principle: classification is not primitive

The heuristic forests exist because the compiler classifies nodes into
categories (type, function, service) that are not primitive concepts.
They are surface sugar — named compositions of more fundamental
properties. See `MODELING.md §four-layer model`:

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts        (what flows through nodes)
Foundation:         logical algebra                  (why it's sound)
```

`MODELING.md` already states: *"The sugar informs the parser what fields
to expect. It does not flow into the compiler core as identity."*

The compiler violates this. Surface identity (is-this-a-function,
is-this-a-service) leaks into every stage — not as an explicit concept,
but as implicit field-presence heuristics that re-derive it. The
classification forests are the compiler re-inventing surface sugar
because it doesn't have the underlying primitives.

### What the classifications are really asking

Every classification forest is asking about combinations of a few
orthogonal properties. These properties are already on Node but are
not named as concepts:

| Property | What it means | Foundation | Ontological category (§above) |
|----------|---------------|------------|-------------------------------|
| **Structure** | Node asserts a data shape (connective + children) | Conjunction/Disjunction — logical structure | Algebra |
| **Evidence** | Node provides backing for its claim (body = internal, transport = external) | Witness/proof — epistemic | Signal |
| **Morphism** | Node describes a mapping (params → return type) | Implication — logical entailment | Reduction (rewrite rule) |

Structure/Evidence/Morphism are **analysis vocabulary** — a way to
talk about what existing Node fields mean. They are NOT proposed as
stored metadata or a new boundary layer. The heuristic forests exist
because consumers re-derive these properties from raw Node fields
instead of reading the fields directly. The fix is making the fields
reachable, not adding a Structure/Evidence/Morphism annotation.

The surface sugar composes these:

| Sugar | Decomposition |
|-------|---------------|
| `type` | Structure, no evidence (the claim IS the structure) |
| `fn` | Morphism + internal evidence |
| `func` | Morphism + internal evidence + external binding |
| `service` | External evidence + structured children (operations) |
| `resource` | Structure + external identity (capabilities) |
| `data` | Internal evidence + type constraint (ground term) |

The downstream consumer never needs to know "is this a function?" — it
needs to know "does this have a morphism?" and "does this have internal
evidence?" Those are the actual questions. The classification is a
lossy proxy for them.

### The pattern: concepts are emergent, not primitive

This is the same pattern as recursion. "Recursion" was treated as a
concept that needed to be detected and classified (is this safe? is
this tail-recursive?). In reality, recursion is emergent from three
primitives: fold (structural descent), descend (child traversal), and
repeat (bounded iteration). Once the primitives were modeled, the
classification problem dissolved — you didn't need to ask "is this
safe recursion?" because the primitives were safe by construction.

The same principle applies here:

- **Item classification** (MM-1): "type/function/service" are emergent
  from structure/evidence/morphism. Model the primitives and the
  classifications become sugar that the parser uses and the compiler
  doesn't need.

- **Type structure** (MM-2): "product/sum/leaf" are emergent from
  connective (already primitive). The missing piece is a named
  interpretation that the compiler can consume without re-deriving.
  Connective IS the primitive — it just needs to be the vocabulary
  the compiler thinks in, rather than something it checks via
  `.connective == Conj`.

- **Method identity** (MM-3): "fold/map/filter" are emergent from
  algebraic structure. A fold is a monoid homomorphism. A map is a
  functor application. A filter is set comprehension. The algebra
  framework (`AlgebraFieldTemplate`) already carries this identity —
  the missing piece is surfacing it through the pipeline to consumers
  that currently dispatch on name strings instead.

### Design direction

The goal is not to carry classification forward from the parser, but
to model the underlying primitives so classification is unnecessary.
The compiler should work at the composition and semantic kernel layers,
never at the surface sugar layer. When it needs to dispatch, it
dispatches on primitives (structure? evidence? morphism?), not on
named compositions (type? function? service?).

This means the primitives need to be modeled in `dsl/std/` — the same
way connective, cardinality, and algebraic structure already are.
They become part of the shared concept library that the compiler both
generates code for and consumes. The compiler moves parts of itself
into higher-level abstractions that it itself processes.

### Practical principles

1. **Surface sugar does not flow past the parser.** If the compiler
   asks "is this a function?" it's using sugar as identity.

2. **Classify by dispatching on primitives, not named compositions.**
   The question is "does this have evidence?" not "is this a function?"

3. **If a concept keeps getting re-derived, it's a missing primitive.**
   The re-derivation is the compiler telling you what it needs modeled.

4. **Unrepresentability over discipline.** Design the types so the
   wrong question can't be asked, not so that people avoid asking it.

5. **The graph represents semantics, emission only renders.** If the
   emitter makes a semantic decision, that decision belongs upstream
   in the semantic kernel.

6. **Consume existing authorities, never duplicate them.** When a
   downstream consumer needs a fact, the fix is to surface the existing
   authority through the pipeline — not to derive a parallel
   representation. Adding a new TypeShape alongside connective, or a
   new AlgebraMethodKind alongside method_def, is `a + 0 = a`
   reimplemented as `identity_check(a) → a`. The mathematical minimum
   is one authority per fact. Ontological analysis (Signal/Algebra/
   Reduction) helps identify what the heuristics are asking; the fix
   is always "make the existing answer reachable," not "add a new
   answer."
