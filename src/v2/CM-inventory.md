# CM Heuristic Inventory

Companion to [`CM.md`](CM.md). Catalogues every known heuristic site
across compiler `.dag` files, mapped to the three ontological categories
(Signal/Algebra/Reduction) with modeling implications.

**Purpose:** Validate whether the Signal/Algebra/Reduction ontology
(CM.md §Foundational ontology) actually dissolves the heuristic
forests, or whether it's a dead end. Each entry includes the
fundamental question being asked and what changes with the ontology.

**Legend:**
- **S** = Signal (assertion, state, epistemic — "does this hold?")
- **A** = Algebra (structure, combination — "how does this compose?")
- **R** = Reduction (transformation, application — "what happens?")
- **fail-open** = returns a default (Unit, empty string, false) on miss instead of erroring
- **fabrication** = creates data that doesn't exist upstream

---

## Summary

| File | Sites | S | A | R | Fail-open | Key theme |
|------|-------|---|---|---|-----------|-----------|
| 00_core.dag | 4 | 2 | 0 | 2 | 0 | Transport identity by string |
| 02_parse.dag | 26 | 17 | 0 | 9 | 4 | Grammar keyword dispatch (mostly legitimate) |
| 03_normalize.dag | 1 | 1 | 0 | 0 | 0 | Container arity |
| 03_resolve.dag | 5 | 2 | 1 | 2 | 0 | Bootstrap module ordering hack |
| 04_items.dag | 2 | 0 | 1 | 1 | 0 | Item classification forest #1 |
| 04_resolve.dag | 16 | 10 | 4 | 2 | 4 | Alias/generic resolution; many fail-open |
| 04_sigs.dag | 3 | 3 | 0 | 0 | 0 | Function identification |
| 04_types.dag | 21 | 13 | 7 | 1 | 5 | Type equality/compatibility; most fail-open |
| 04_lookup.dag | 2 | 0 | 2 | 0 | 0 | Field access via connective |
| 04_infer.dag | 8 | 3 | 1 | 4 | 2 | Item classification #3, method dispatch |
| 04_method.dag | 2 | 1 | 0 | 1 | 1 | Builtin function registry |
| 04_access.dag | 5 | 4 | 1 | 0 | 3 | Index access type dispatch |
| 04_cycle.dag | 4 | 2 | 1 | 1 | 1 | SCC/dependency graph |
| 04_emit_info.dag | 6 | 2 | 1 | 3 | 2 | Item classification #2, type summary |
| 05_emit.dag | 2 | 0 | 1 | 1 | 0 | Type rendering dispatch |
| 05_emit_rust.dag | 8 | 1 | 2 | 5 | 1 | Method dispatch, nullary detection |
| 05_emit_go.dag | 4 | 1 | 1 | 2 | 2 | Item dispatch, index dispatch |
| 05_emit_python.dag | 4 | 1 | 1 | 2 | 2 | Item dispatch, index dispatch |
| ownership.dag | 10 | 6 | 0 | 4 | 2 | Fold/threaded by string name |
| coercion.dag | 5 | 1 | 2 | 2 | 2 | Container-to-algebra hardcoded table |
| languages.dag | 5 | 2 | 0 | 3 | 2 | Target dispatch, fabricated error markers |
| complexity.dag | 4 | 1 | 1 | 2 | 0 | Expression identity, method classification |
| compile.dag | 3 | 1 | 0 | 2 | 1 | CX disabled, diagnostic fabrication |
| trace.dag | 1 | 1 | 0 | 0 | 1 | Stack underflow |
| runtime_rust.dag | 4 | 4 | 0 | 0 | 4 | Bounds-miss defaults |
| **Total** | **~150** | **~79** | **~27** | **~44** | **~39** | |

---

## Ontological analysis

### Signal sites (~79) — "does this hold?"

The largest category. Most are:
1. **Name-based membership checks** — `n.name == "Callable"`, `is_container_type(name:)`, magic name allowlists
2. **Fail-open fabrication** — `None => unit_type`, `None => ""`, `None => false`
3. **Field-presence assertions** — `body != none`, `transport != none`, `params |> count > 0`

**Modeling implication:** Field-presence assertions (category 3) are
already correct — `body != none` IS the structural fact. The problem
is that consumers can't reach these fields at the boundary (they're
stripped or re-keyed by side-tables). The fix is making existing fields
(`body`, `transport`, `connective`) reachable at every consumption
site, not adding derived `has_reduction?` wrappers (that would
duplicate the existing authority). Name-based checks (category 1)
dissolve when consumers read the structural fact the name approximates
(e.g., algebra profile instead of `"List"` string). Fail-open sites
(category 2) need fail-closed boundaries, not ontological modeling.

### Algebra sites (~27) — "how does this compose?"

Almost all are connective dispatch:
1. **Conj/Disj/NoConnective branching** — field access path, type summary, type equality
2. **Container arity** — `children |> count == 1` vs `== 2` for element vs keyed collections
3. **Container-to-algebra mapping** — hardcoded string tables (coercion.dag)

**Modeling implication:** Connective IS the existing authority.
Consumers that re-derive "product or coproduct?" should read
connective directly (or its existing downstream authority,
`TypeSummary.repr`). Adding a new TypeShape enum would duplicate
connective — the fix is surfacing connective/TypeSummary.repr
through stage boundaries. Container arity (2) should come from the
existing algebra profile (which already carries parameter counts).
The hardcoded container-to-algebra table (3) is already identified
as needing derivation from algebra declarations.

### Reduction sites (~44) — "what happens?"

Three clusters:
1. **Method/function name dispatch** — 13-way algebra method dispatch, builtin registry, "fold" special-casing
2. **Call/application classification** — call tier, nullary detection, binding kind
3. **Target language dispatch** — per-backend rendering, keyword/primitive coercion

**Modeling implication:** The `method_def` Node and `AlgebraFieldTemplate`
already carry method identity structurally. Method name dispatch
(category 1) dissolves when consumers read from these existing
authorities instead of matching on name strings — not by adding a
parallel AlgebraMethodKind enum (duplicate representation). The "fold"
special cases in ownership.dag dissolve when ownership reads the
existing algebra framework's structural facts about fold. Call
classification (2) dissolves when inference normalizes ExprVar→ExprCall
for nullary functions (making the expression IR the authority). Target
dispatch (3) is inherent — different targets need different rendering —
but fail-open fallbacks (fabricated error markers) should be
fail-closed.

---

## Per-file detail

### 00_core.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 1038-1041 | `t.name == "rest"` / `"shell"` / `"file"` / `"local"` | R | What transport kind? | Transport kind = ReductionKind (external variant). Already uses named constants — close to structural. |
| 1060-1092 | Transport property accessors by string key | R | What transport field? | Transport fields become typed product, not string-keyed bag. |

### 02_parse.dag

Parser keyword dispatch is largely *legitimate* — the grammar IS defined
by keywords. The ontological concerns are:

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 745-753 | `"acquire"` / `"release"` hardcoded exclusion | S | Is this name a keyword? | Should be in SyntaxSpec keyword data, not hardcoded |
| 844-862 | `"skip_newlines"` / `"with"` parser helper names | S | Is this a progress witness? | Parser progress should be structural (advance/expect = witness by construction) |
| 2275-2304 | `"config"` / `"transport"` / `"operation"` ident dispatch | R | What service entry? | ServiceEntryKind enum derived from SyntaxSpec |
| 2358-2387 | `"rest"` / `"shell"` / `"file"` transport dispatch | R | What transport kind? | Same as 00_core: should be ReductionKind |
| 2605-2663 | 8+ keyword/ident cascade in op body | R | What operation entry? | OperationEntryKind from SyntaxSpec |
| 2773-2797 | `"Refined"` hardcoded + `is_container_type` | S | Human-readable type name? | Structural: containers have algebra, Refined has constraint |
| 3394-3395 | `"let"` / `"return"` keyword dispatch | R | What statement? | Legitimate grammar dispatch |
| 3733-3738 | `"match"` / `"if"` / `"for"` / `"fn"` etc. | R | What expression? | Legitimate grammar dispatch |
| 3843-3854 | Uppercase-start casing heuristic | S | Record literal or variable + block? | Grammar ambiguity; SyntaxSpec should resolve |

### 03_resolve.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 327-332 | `name == "std.types"` sort key override | R | Module order? | Import graph should determine order structurally |
| 343-368 | `name != "std.types" && name != "std.algebra"` implicit dep | R | Which modules get std.types? | Implicit import = structural graph edge, not name exception |

### 04_items.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 83-112 | `inferred_to_outputs`: connective + name checks | A | How many output fields? | Algebra (Product: expand fields, Coproduct: wrap) |
| 114-131 | `item_kind`: 7-branch priority chain | R | What emission category? | Consumers read existing fields (`body`, `transport`, `connective`) directly — no derived wrapper |

### 04_resolve.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 137-158 | `is_user_generic_use_site`: 5 checks | A | Parameterized alias or container? | Algebra arity on type declaration; no name check needed |
| 208-213 | `classify_alias`: 3 variants | A | Alias shape? | Structural: has children? has inferred? |
| 241 | `n.name == "Refined"` special case | S | Refined type? | Refined = structural constraint (AND with predicate), not name |
| 380-401 | `node_is_keyed_collection` dispatch | A | Map-specific resolution? | Algebra arity: keyed = arity 2 |
| 384-390 | Map child fallbacks: `None => string_type` | S | Missing key/value type? | **Fail-closed:** missing type = diagnostic, not fabricated String |
| 454 | `"Callable" || "Dynamic" || "Error"` allowlist | S | Suppress diagnostic? | These should be structural: Callable = function type, Error = error state |
| 473 | `None => unit_type` | S | Missing inferred? | **Fail-closed:** missing inference = diagnostic |

### 04_sigs.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 67-69 | `params > 0 && body != none` | S | Is this a function? | These ARE the existing structural facts — the check is already correct, just needs to be reachable at all sites |
| 119-236 | `dsig.inferred != none` repeated | S | Has return annotation? | Legitimate presence check |

### 04_types.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 70-77 | `rt_type` Unit fallback | S | Missing type? | **Fail-closed:** error/variable/untyped = diagnostic |
| 89-97 | `is_container_type` / child count | S+A | Collection kind? | Algebra arity from profile |
| 103-106 | `container_expected_arity` name lookup | S | Under-parameterized? | Arity from algebra declaration |
| 258-267 | `n.name == "Refined"` unwrap | S | Peel Refined? | Structural: Refined = product with constraint child |
| 274-321 | `node_type_shape` classification cascade | A | Full shape label? | TypeShape enum computed once, not re-derived |
| 326-367 | `node_type_compatible` 8+ branches | S+A | Types compatible? | Structural comparison on TypeShape; fail-open errors → fail-closed |
| 371-396 | `prefer_specific_type` Unit detection | A | Which type is more specific? | Algebra: Unit is terminal (bottom of specificity) |
| 447-452 | Leaf-vs-struct name equality | S | Cross-shape equality? | **Structural identity, not name:** leaf "Foo" ≠ struct Foo unless declaration links them |
| 616-633 | `binop_algebra_field` mapping | A | BinOp → algebra method? | Clean enum dispatch; NullCoalesce `""` is the fail-open |
| 681 | `normed.name == string_type.name` | S | String iterates to what? | Algebra: String inhabits FreeMonoid<Char>, element type = Char |
| 729-740 | `"T"/"K"/"V"` + `"Char"/"Ordering"` magic names | S | Placeholder or concrete? | Should derive from declaration (has_type_param on the algebra) |

### 04_lookup.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 209-237 | `field_summary_for_type`: connective dispatch | A | Field access path? | TypeShape: Product → field lookup, Coproduct → variant, Leaf → none |
| 272-293 | `lookup_structural_method`: connective dispatch | A | Has methods? | Algebra: Conj has declared methods; else enrich from algebra profile |

### 04_infer.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 842-903 | Method result type: 7-way name match | R | What type does this method return? | Existing `AlgebraFieldTemplate` already carries return-type structure — surface it |
| 1008-1070 | Field access: 5-branch cascade | A | Struct/enum/container/optional? | TypeShape dispatch |
| 1130-1189 | Call tier: 5-branch cascade | R | Direct call / method / builtin? | ReductionKind: DirectApplication / MethodApplication / BuiltinRule |
| 1215-1258 | Built-in refinement: 4-way name match | R | Special return type? | Builtin = named rewrite rule; return type from rule declaration |
| 1642-1670 | Lambda element threading: 5 branches | R | Callable shape? | Reduction: lambda = anonymous rewrite rule; shape from declaration |
| 2270-2326 | Type env registration: 5-branch | S+A | What type entity? | Node decomposition: Algebra(connective) + Reduction(body/transport) |
| 2526-2527 | `is_zero_arg_func` | R | Nullary? | Redex test: arity = 0 ⟹ constant redex |
| 2972-3050 | `is_item_constant` | S+A | Constant value? | No reduction + no parameters = constant |

### 04_method.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 70-108 | `builtin_function_registry`: 24-entry name→type map | R | Builtin return type? | Each builtin is a named rewrite rule; return type from rule declaration. Self-identified as "BRIDGE" / "duplicate authority" |
| 117-122 | Unknown builtin → Unit | S | Missing builtin? | **Fail-closed:** unknown builtin = diagnostic |

### 04_access.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 81-109 | Index access: 3-way type dispatch | A | String / Map / List? | Algebra: String = FreeMonoid<Char>, Map = PartialFunction, List = FreeMonoid<T>. Index semantics from algebra profile |
| 100 | `normed.name == "List"` | S | List specifically? | Algebra: indexing from FreeMonoid profile (List, NonEmptyList, etc.) |
| 97-105 | Fallbacks → unit_type | S | Malformed index? | **Fail-closed** |

### 04_emit_info.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 65-70 | `n.name != "Unit"` | S | Type alias? | Unit is structural terminal; shouldn't be detected by name |
| 75-104 | `classify_typed_item`: 9-branch | R | Emission kind? | Node decomposition: Algebra + Reduction → pattern match |
| 107-111 | `None => TypedItemUnhandled` | S | Unknown item? | **Fail-closed:** impossible by construction with Node decomposition |
| 127-159 | Service field scanning + fabrication | R+S | Service facts? | Transport facts from Reduction; fail-open fabrication → fail-closed |
| 389-422 | `build_type_summary`: connective dispatch | A | Struct or enum? | TypeShape |
| ~420 | `n.name == "Callable"` for has_fn | S | Has function fields? | Callable = Reduction type. Structural, not name |

### 04_cycle.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 32 | Self-loop exclusion | A | Graph structure | Legitimate graph algorithm |
| 115,124 | `None => 0` in-degree fallback | S | Missing entry? | Safe default for Kahn's algorithm (missing = 0 deps) |
| 143-148 | Self-ref detection by name equality | S | Self-referencing type? | Structural: node references itself (graph edge, not name) |

### 05_emit.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 922-930 | `n.name == "Callable"` → special TypeRendering | R | Function type? | Callable is the type of a Reduction. Structural, not name-based |
| 1095-1191 | `render_type_base`: 7-branch dispatch | A+R | Target syntax for type? | TypeRendering should carry Algebra (Product/Coproduct/Leaf) + Reduction (Callable). String names dissolve |

### 05_emit_rust.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 1685-1750 | Nullary detection: 3-tier | R | Append `()`? | Is-redex: structural on expression |
| 2286-2338 | Index dispatch: string/map/list | A | Index syntax? | Algebra profile: FreeMonoid → integer index, PartialFunction → key lookup |
| 2853-2901 | Method dispatch: 13-way name match | R | Rendering template? | Read existing `AlgebraFieldTemplate` + `method_def` structural facts |
| 2944-2957 | Match scrutinee Rc: 4 branches | A | Ownership at match? | Sharing from TypeSummary |
| 3101-3123 | Field optionality: 3 fallbacks | A | Optional field? | Cardinality from type declaration |
| 3170-3239 | Variant parent + struct name: 8 fallback paths | S | Type identity? | Structural: variant → parent is a graph edge, not name lookup |
| 3881-3897 | Shell return type: 4 branches | A | Transport output type? | Reduction output type from transport declaration |

### 05_emit_go.dag / 05_emit_python.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 371-406 / 363-406 | `emit_typed_item`: kind dispatch + `""` | R+S | Item rendering? | Node decomposition; `""` fail-open → impossible |
| 609 / 591 | Ignores `binding_kind` entirely | R | Nullary? | **Bug:** is-redex not consulted. With structural redex, impossible to ignore |
| 822-843 / 801-824 | Index dispatch: string-like check | A | Index syntax? | Algebra profile |
| 373 / 366 | `None => false` unit_only | S | Unit-only enum? | Fail-open → structural |

### ownership.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 212-220, 240-241 | `fname == "fold"` / `mname == "fold"` | R | Threaded semantics? | Read existing algebra framework facts — fold's structural properties are already declared in `AlgebraFieldTemplate` |
| 225, 254 | `a.name == "init"` | S | Fold accumulator arg? | Fold's accumulator position from algebra declaration |
| 360-378 | Consumer count branching | S | Ownership class? | Legitimate count-based classification |
| 390-395 | `LocalValueBinding` match | S | Movable? | Legitimate variant classification |
| 429-502 | `terminal.name == acc_type_name`, `""` sentinel | S | Fold body constructs acc? | Name-based type matching + empty-string sentinel → structural type comparison |

### coercion.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 42-80 | RenderTarget match (4 sites) | R | Per-target data? | Legitimate target dispatch |
| 93-98 | `None => dag_name` passthrough | R | Unknown primitive? | **Fail-closed:** unknown primitive = diagnostic |
| 121-138 | 14-entry container-to-algebra string table | A | Container algebra? | **Dissolves:** derive from algebra profile on the type declaration |
| 170-171 | Hardcoded keyed/element container name lists | A | Container kind? | **Dissolves:** derive from algebra arity |

### languages.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 382-389 | Dag → `rust_spec()` fallback | R | Dag language spec? | Dag should have its own spec or explicit absence |
| 391-398 | `None => key` keyword passthrough | R | Unknown keyword? | **Fail-closed** |
| 400-407 | Fabricated error markers | R | Unknown primitive? | **Fail-closed** |
| 431-435 | `is_value_type`: all non-Rust → false | S | Value type? | Should derive from language extdeps |
| 439-446 | `is_string_like`: Dag → false | S | String-like? | Same |

### complexity.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 566-580 | `extract_base_var_name`: ExprData match | R | Variable identity? | Expression identity from IR, not re-extraction |
| 737-765 | `classify_call_progress`: witness dispatch | R | Call progress kind? | Reduction: call pattern → iteration primitive (already in computation.dag) |
| 1393-1500+ | `max_path_self_calls_with_cont`: ExprData | R | Recursive call analysis? | Same: structural reduction analysis |
| distributed | `mname == "first" || mname == "last"` | S | Shrinking method? | Algebra: first/last = FreeMonoid accessor with known size contract |

### compile.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 155-162 | RenderTarget dispatch | R | Which emitter? | Legitimate |
| 650-657 | Hardcoded "error" severity, null category | S | Diagnostic metadata? | Should carry actual severity from upstream |
| 819-821 | CX disabled | R | Skip complexity? | Temporary; CX track will re-enable |

### trace.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 79-85 | Empty stack on underflow | S | Stack empty? | Legitimate defensive check (trace is auxiliary) |

### runtime_rust.dag

| Lines | Heuristic | Cat | Question | With ontology |
|-------|-----------|-----|----------|---------------|
| 40-57 | char_at / substring: `""` on bounds miss | S | Out of bounds? | Runtime semantics choice (not a compiler heuristic) |
| 207-211 | code_point: 0 / from_code_point: `""` | S | Invalid codepoint? | Same — runtime behavior, not compiler modeling |

---

## Cross-cutting patterns

### Pattern 1: Fail-open fabrication (~39 sites)

The most pervasive anti-pattern. When data is missing, the site
fabricates a default (unit_type, empty string, false) instead of
producing a diagnostic. This allows ungrounded claims to propagate.

**Ontological fix:** Fail-open is not solved by the Signal/Algebra/
Reduction ontology directly. It requires fail-closed boundaries:
every lookup returns `Result<T, Diagnostic>`, never `T` with a
silent default. The ontology helps indirectly: with structural
decomposition, there are fewer "what kind?" questions to ask,
so fewer opportunities for miss → fabrication.

### Pattern 2: Re-derived classification (~25 sites across MM-1)

Four independent forests classify Nodes into item kinds. Each
inspects the same fields with subtly different priority.

**Ontological fix:** Direct dissolution. Node decomposition into
Signal/Algebra/Reduction makes classification a pattern match:
`{connective: NoConnective, body: present, transport: absent}` → FnItem.
Computed once, carried structurally. The four forests merge.

### Pattern 3: Name-as-identity (~30 sites)

String names used as semantic authority: `"Callable"`, `"Refined"`,
`"List"`, `"fold"`, `"first"`, `"last"`, magic type param names.

**Ontological fix:** Partial dissolution. Each name stands for
a structural fact:
- `"Callable"` = the type of a Reduction (function type)
- `"Refined"` = Product with a constraint child
- `"List"` = FreeMonoid with arity 1
- `"fold"` = catamorphic operation (structural facts in `AlgebraFieldTemplate`)
- `"first"/"last"` = FreeMonoid accessor with known size contract
Once the structural fact has a typed representation, the name
becomes rendering sugar (how it's displayed), not identity.

### Pattern 4: Lossy boundaries (~10 sites in emit)

TypeRendering erases connective. EmitGraphInfo partially erases
item identity. Downstream re-infers what was lost.

**Fix:** TypeRendering must stop erasing connective. The existing
connective and TypeSummary.repr authorities should flow through
to the rendering layer. The 97-line `render_type_base` dispatch
collapses when it can read the existing authority directly.

### Pattern 5: Duplicate authority (~8 sites)

Same fact derived independently in multiple places:
`builtin_function_registry` vs algebra templates, `item_kind` vs
`classify_typed_item`, `is_zero_arg_func` vs `is_zero_arg_callable_ref`.

**Fix:** Single authority per fact — consume existing structure, never
duplicate. Algebra operations come from the existing algebra profile.
Item identity comes from the existing Node fields. Nullary call status
comes from ExprVar→ExprCall normalization (making the expression IR
the sole authority). Each duplicate authority should be deleted in
favor of the existing one.
