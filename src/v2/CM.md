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
| Normal form | Fully evaluated — no more redexes (= emitted target code) |
| fold/descend/repeat | **Rewrite strategies**: the order in which reductions are applied |
| Termination proofs | Standard rewrite-termination via ranking functions (already in termination.dag) |
| Compilation itself | **Reduction to normal form** in the target language |

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
  general but less operational). Reduction chosen because it has
  the strongest existing theory and connects naturally to
  compilation (which IS reduction to normal form).
- **std/ representation?** Possibly `std/reduction.dag` with
  Redex, RewriteRule, NormalForm. Must connect to Layers 4-6
  (iteration primitives are rewrite strategies, call patterns
  are redex classifications).
- **Relationship to Morphism?** The Morphism property
  (§Foundational principle) may be the static description
  ("this Node is a rewrite rule") while reduction is the dynamic
  description ("applying this rule is a computation step").

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

**Design direction:** Either (a) compute classification once and carry
it structurally to every consumer (e.g. `ClassifiedItem` wrapper), or
(b) stop classifying and let consumers pattern-match on the structural
facts directly. Option (b) is more aligned with "nodes are nodes" but
requires consumers to express their needs structurally.

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
foundational. What's missing is a cached interpretation layer. Options:
- `TypeShape` (Product/Coproduct/Scalar) computed once per type
- Or: let `TypeSummary.repr` (StructRepr/EnumRepr, already exists)
  become the single authority that all emit sites consume

`TypeSummary.repr` is already halfway there — the question is whether
non-emit consumers (lookup, types, complexity) should use it too, or
whether they need their own structural concept.

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

Two options discussed:
- **Option A:** Add `NullaryCallBinding` to `VarBindingKind`. Minimal,
  follows current architecture. Emit pattern-matches without type checks.
- **Option B:** Normalize ExprVar → ExprCall during inference when the
  reference IS an invocation. More principled (graph represents
  semantics, not surface syntax) but changes ExprVar/ExprCall partition.

Note: Go and Python emitters completely ignore `binding_kind` today —
they destructure with `_` and emit bare identifiers. They likely have
the same bug for nullary functions.

**Design direction for method dispatch:** Add `AlgebraMethodKind` enum
to `MethodSemantics`. This is localized to the algebra framework and
eliminates heuristic chains in both infer and emit. The algebra
framework already has `AlgebraProfile` — method kind is the missing
companion concept.

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

### MM-1: Item classification is unrepresentable in emit

**Claim:** Emit functions cannot access raw item structural fields
(`body`, `transport`, `uses`, `type_annotation`, `properties`) for
dispatch decisions. The boundary type carries pre-resolved item facts.
Classification forests can't exist because there's nothing to classify.

**Test:** The emit modules (`05_emit*.dag`) do not import or use these
fields for item-level dispatch. The boundary type between infer and
emit makes classification a consequence of the type, not a runtime
decision.

**What this implies for the design:** The boundary type must carry item
facts structurally. Either items arrive pre-classified (carry-on-item)
or the boundary type is rich enough that emit never asks "what kind?"
— it pattern-matches on what it received.

### MM-2: Connective interpretation is unrepresentable in emit

**Claim:** Emit functions cannot access `Connective` / `Conj` / `Disj` /
`NoConnective`. They receive a resolved type structure (product/sum/leaf
or equivalent) and dispatch on that.

**Test:** The emit modules do not import `Connective, Conj, Disj,
NoConnective`. Type rendering is an exhaustive match on a sum type,
not a multi-branch if-else that re-interprets connective.

**What this implies for the design:** Somewhere between infer and emit,
connective must be interpreted once into a concept that emit understands.
`TypeSummary.repr` (StructRepr/EnumRepr) is a partial version of this
but only covers type definitions, not all type rendering.

### MM-3: Method name dispatch is unrepresentable in emit

**Claim:** Emit functions cannot access `method_def.name` as a dispatch
key. They receive a method kind enum and dispatch on that.

**Test:** `AlgebraMethodSemantics` carries a `kind` field that is a
closed sum type. Emit functions match on kind, not name. Nullary
function invocation is a structural fact on the expression (either
`NullaryCallBinding` or normalized to `ExprCall`), not a type-name
check at render time.

**What this implies for the design:** `MethodSemantics` must carry an
`AlgebraMethodKind` enum. The algebra framework already computes method
identity — it just doesn't name it as a type. For nullary calls, the
IR must represent invocation semantics, not surface syntax.

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

Structure/Evidence/Morphism are the compiler-level manifestation of
the three foundational categories. The heuristic forests exist because
the compiler re-derives these from raw Node fields instead of consuming
them as named primitives.

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
  framework already knows this — the missing piece is naming the
  operational shape so dispatch doesn't fall back to string matching.
  Deeper: function application itself may be the missing primitive
  (§Foundational ontology). Methods are named rewrite rules;
  dispatch should be on rule structure, not name strings.

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
