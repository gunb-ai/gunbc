# CM: Compiler Concept Modeling

The compiler works but doesn't model what it does. Every stage
re-derives structural facts that earlier stages already knew, producing
heuristic if-else forests that get shuffled around but never eliminated.
This document catalogs the missing models, their symptoms, and design
directions. The goal is to spend time getting the modeling right.

See also: `DESIGN.md` (principles), `ROADMAP.md §CM` (summary table).

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

## Acceptance criteria (uncheateable)

These count actual heuristic sites in .dag source. Moving a heuristic
behind a helper doesn't change the count — the helper still contains
the interrogation. You can only reach zero by modeling the concept.

### MM-1 acceptance

```bash
# Zero classification forests
grep -c 'classify_typed_item\|classify_item' src/v2/*.dag  # target: 0
# Zero raw structural interrogation in emit
grep -cE '\.(connective|body|transport|uses|type_annotation|properties) [!=]=' src/v2/05_emit*.dag  # target: 0
# Zero fail-open fallbacks
# (manual audit: no None => TypedItemUnhandled, None => "", None => false for missing facts)
# Zero name-keyed side-tables
grep -c 'Map<String, TypedItemKind>\|Map<String, FunctionSignature>\|Map<String, ServiceFieldSet>\|Map<String, ResourceDefinition>' src/v2/*.dag  # target: 0
```

### MM-2 acceptance

```bash
# Zero connective interpretation in emit
grep -cE '\.connective ==' src/v2/05_emit*.dag  # target: 0
# Type rendering dispatch is exhaustive match, not if-else forest
# (manual audit: 05_emit.dag type rendering is <20 lines, matches on a sum type)
```

### MM-3 acceptance

```bash
# Zero method name dispatch
grep -cE 'method_def\.name|method_name.*==' src/v2/05_emit*.dag src/v2/04_infer.dag src/v2/complexity.dag  # target: 0
# Nullary invocation handled structurally in all 3 backends
# (test: emit a zero-arg function ref in Go/Python/Rust, all produce "()")
```

### Current baseline (2026-04-05)

| Metric | Count | Target |
|--------|-------|--------|
| Structural interrogation in emit | 55 | 0 |
| `.connective ==` in emit | 24 | 0 |
| Method name dispatch (emit + infer + complexity) | 21 | 0 |
| Item classification forests | 4 | 0 |
| Fail-open fallbacks | 8 | 0 |

---

## Principles for resolution

1. **Model the concept, don't classify the node.** The question "is this
   a function?" is already wrong — it's an interpretation. The stable
   facts are "has body", "has transport", "has params". If emit needs a
   classification, it should be a consequence of structural facts, not
   a side-table lookup.

2. **Compute once, carry structurally.** If a fact is derived during
   inference, it must reach emit without re-derivation. Either on the
   item itself or in a boundary type that's fail-closed by construction.

3. **No fail-open fallbacks.** If a lookup can miss, the map isn't
   total. Either make it total (every item guaranteed present) or make
   the miss impossible by carrying the fact structurally.

4. **Name keys are identity proxies.** Every `Map<String, X>` keyed by
   item name is a proxy for structural identity. The dissolution is
   carrying X on the item itself, not improving the key.

5. **String dispatch is a missing enum.** Every `method_def.name == "fold"`
   is a missing `AlgebraMethodKind` variant. Every `t.name == "rest"`
   is a missing `TransportKind` variant (partially fixed).

6. **The graph represents semantics, emission only renders.** If the
   emitter makes a semantic decision (nullary call, clone vs move,
   method dispatch), that decision belongs upstream.
