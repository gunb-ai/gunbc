# v2 Compiler Audit — Stage-by-Stage Structure

> Generated 2026-04-13. Purpose: lay out the shape of every file in `src/v2/`
> so we can identify simplification and alignment opportunities against the
> thesis/roadmap.

---

## Pipeline overview

```
.dag source
  → 01_tokenize    (String → List<Token>)
  → 02_parse       (List<Token> → Node tree)
  → 03_resolve     (Node trees → ModuleGraph with imports/topo order)
  → 03_normalize   (structural checks, bare container diagnostics)
  → 04_infer       (type inference, provenance, emit info — 13 sub-files)
  → 05_emit        (shared emitter + per-target backends)
  + complexity     (symbolic cost analysis, termination proofs)
  + ownership      (binding usage, move/clone decisions)
  + coercion       (target-language type mapping)
  → output files
```

Orchestration lives in `compile.dag`. Support files: `languages.dag`,
`artifact.dag`, `trace.dag`, `effect_derivation.dag`, `runtime_rust.dag`,
`compiler_tests_rust.dag`.

---

## 00_core.dag — Shared IR (~1,703 lines)

**Role:** Compiler domain model. Not a pipeline stage — types used everywhere.

**Key types:**
- `Token`, `TokenShape` — lexical tokens (keywords as one shape + text)
- `Node` — the central IR node (name, ident, spans, children, connective,
  params, inferred, body, transport, properties, type_annotation, expr_data…)
- `ExprData` — large coproduct of expression forms (literals, calls, match,
  if, binops, etc.)
- `InferredNode` — `Resolved | CompilerError | TypeVariable`
- `CompilerDiagnostic` — diagnostic variants
- `CompileResult`, `TextFile` — pipeline I/O
- `DeclaredFuncSig`, `DeclaredFuncEnv` — function signatures pre-inference
- `InternTable`, `InternResult` — string interning
- `LineCol`, `NewlineIndex` — source location

**Key functions:** Node constructors (`make_param_node`, `make_field_node`,
`make_expr_node`…), `authored_name_at` / `node_name_span`, diagnostic
helpers, expr projection helpers (`binop_left`, `match_scrutinee`…),
transport builders, `build_newline_index` / `source_text_at`, intern
operations.

**Notable patterns:**
- Large **hand-maintained `data` maps**: `expr_child_roles`, wrapper roles,
  complexity-related name sets (`is_tree_size_preserving`, etc.)
- Keywords are **not** closed in `TokenShape` (open set via syntax spec)
- `authored_name_at` special-cases kernel spans (`<kernel:Name>`)
- `Node.name` still exists (Theme 2 target: delete it)

---

## 01_tokenize.dag — Tokenizer (~523 lines)

**Role:** Pure `String → List<Token>`. Newlines preserved. String
interpolation and escape handling.

**Key types:** `TokenizerState`, `TokPos`, `ScanResult`, `SourceRef`
(file + text + pre-split `source_chars`).

**Key functions:** `tokenize` → `tokenize_loop` → `scan_next_token` →
`scan_ident`, `scan_number`, `scan_string`, `scan_str_cont`,
`scan_string_body`; `skip_spaces_and_comments`; `process_escapes`.

**Notable patterns:**
- `data single_punct` map for one-char punctuation
- `dag_keyword_set` import + `is_keyword_text` O(1) lookup
- `interp_depth` stack for `{`/`}` inside interpolated strings
- `fuel` on the main loop to bound work
- UTF-8 handled via code-point list, not per-call string scans

**Assessment:** Clean, focused, self-contained. Minimal simplification needed.

---

## 02_parse.dag — Parser (~4,828 lines)

**Role:** Recursive-descent parser over `List<Token>`. Pratt parsing for
expressions. Produces `Node` modules/items/expressions. First error halts.

**Key types:** `ParseContext` (`source_indices`, `intern_table`),
`ParseResult`, many flat result records (`TokenResult`, `ExprResult`,
`ItemResult`…), `ExpectedToken`, `BindingPower`.

**Key functions:** Entry `parse` → `parse_module` → `parse_imports` /
`parse_items` / `parse_item` / `parse_item_by_form`; top-level parsers for
types, fn, func, service, resource, data, transports (REST, shell, file),
operations; expression Pratt parsing (`parse_expr` / `parse_expr_bp` /
`parse_expr_loop`); shared `advance`, `expect`, `eat`, `parse_error`.

**Notable patterns:**
- Very large set of `is_*_shape` / `tok_is_*` helpers (explicit token
  classification)
- Imports `dag_syntax_spec`, `dag_non_name_keywords`
- `ItemForm`-driven dispatch from `v2.compiler.languages`
- Recovery nodes via `parse_recovery_expr`
- Interning integrated via context / pre-intern

**Assessment:** Large but structurally sound — it's a parser. The size
is proportional to the surface syntax. `ItemForm` dispatch from
`languages.dag` is good alignment with Theme 4.

---

## 03_resolve.dag — Module Resolution (~462 lines)

**Role:** Parsed modules → `ModuleGraph`: duplicate detection, import
resolution, export checks, cycle detection, topological ordering.

**Key types:** `ModuleGraph`, `ResolvedModule`, `ResolvedImport`,
`DepEdge`, `ResolveAccum`, `TopoResult`, `KahnDrainState`.

**Key functions:** `resolve_modules` (phases: duplicates → index + export
sets → per-module `resolve_module_imports` → `topological_sort` → build
sorted `ResolvedModule` list); `get_exported_names`; Kahn's algorithm
with fuel.

**Notable patterns:**
- **Bootstrap bridge:** implicit edges from `std.types` when present but
  not imported; `topo_sort_key` forces `std.types` first
- Kahn uses fuel and dedupes zero-in-degree batch with a map
- `map_has` evaluator quirk documented in comments

**Assessment:** Clean graph algorithm. The bootstrap bridge is necessary
complexity, well-documented.

---

## 03_normalize.dag — Structural Checks (~91 lines)

**Role:** Post-resolve, pre-infer structural checks. Currently focused
on bare container types (e.g. `List` with no type args).

**Key functions:** `check_bare_containers` (recursive), `normalize_graph`
(walks modules).

**Notable patterns:**
- Delegates arity expectations to `container_expected_arity` from
  `std.types` (authority outside this file)
- Does **not** rewrite the graph — same `ModuleGraph` returned with
  extra diagnostics

**Assessment:** Tiny, focused. Could grow as more structural checks land.

---

## 04_infer.dag — Main Inference (~5,470 lines)

**Role:** The big one. Namespace/type reconciliation, `build_type_env`,
`typecheck`/`typecheck_modules`, expression inference (`infer_expr`),
blocks, patterns, properties, transport, provenance/SubValueRelation
annotation, output provenance for functions.

**Key types:** `ItemContribution`, `ModuleContext`, `InferScope`,
`InferResult`, `BlockInferState`, `TypedItemResult`, `ArmInferResult`,
`DescentContext`, many small result carriers.

**Key functions:** `infer_block_stmts`, `merge_scope_from_imports`,
inductive-field helpers, scope helpers (`extend_scope*`), `infer_expr`
(very large), `infer_record_lit`, provenance classification
(`classify_binding_provenance`, `classify_let_value`,
`classify_argument`, `classify_body_provenance`), descent annotation
(`annotate_descent`, `annotate_descent_evidence`), output provenance
(`infer_output_provenance`, `populate_output_provenance`),
`infer_property_values`, `infer_transport_node`, `infer_item`,
`build_type_env`, `build_emit_graph_info`, `typecheck_modules`.

**Notable patterns:**
- **Massive central file** — largest in the compiler
- Heavy `match` on `expr_data` and expression shapes
- Integrates induction/SubValueRelation for descent and ownership metadata
- Duplicates small helpers (`is_type_variable`, `type_variable_node`)
  that also appear in sibling modules
- **Triple/quadruple classification system** (Theme 1 target: collapse to
  single SVR computation at binding creation)
- `lambda_param_provenance` on `InferScope` as side-channel

**Assessment:** Primary simplification target. The classify_* system is
the heart of the CX gap (340 violations). Theme 1 collapses this.

---

## 04_resolve.dag — Type Resolution (~992 lines)

**Role:** Resolve type Node trees and expression trees against `TypeEnv`:
expand named types, handle generics/slots, aliases, recursive boundaries.

**Key types:** `NodeResolveResult`, `AliasKind` (`AliasParameterized |
AliasLeaf | AliasPassthrough`).

**Key functions:** `resolve_node` / `resolve_node_bounded`,
`substitute_type_slots`, `classify_alias`, `resolve_alias_target`,
`resolve_expr_types`, `resolve_item_types`.

**Notable patterns:**
- Large structural recursion
- Alias classification avoids parser limitations
- Container types explicitly excluded from generic "use site" expansion
- `resolve_expr_types` is a big expression-shaped dispatcher

**Assessment:** Solid structural work. Alias classification is a
necessary bridge.

---

## 04_types.dag — Type Helpers (~992 lines)

**Role:** Pure helpers on Node-as-types: constructors, predicates,
compatibility/equality, algebra/kernel enrichment, template substitution,
literal inference, binop inference.

**Key types:** `AlgebraFieldMatch`, `BinOpInferred`.

**Key functions:** `resolved_type`, `node_is_*_collection`,
`is_product_type` / `is_coproduct_type` / `is_fully_resolved`,
`make_container_type` / `make_map_type` / `make_callable_type`,
`enrich_kernel_type`, `instantiate_algebra_type` / `unify_template` /
`apply_type_substitution`, `node_type_compatible` / `node_type_equals*`,
`infer_literal_node`, `infer_binop_type_node`.

**Notable patterns:**
- String/name-driven checks via `authored_name_at` and `is_container_type`
- Algebra profiles/templates from `std.algebra`
- Binop inference uses `AlgebraFieldKind` lists

**Assessment:** Vocabulary file for stage 04. Theme 2 (delete Node.name)
will affect the name-driven checks here.

---

## 04_sigs.dag — Signature Resolution (~262 lines)

**Role:** SCC-aware resolution of function signatures from
`DeclaredFuncSig` to `ResolvedFuncSig` using call graph and topological
"ready" peeling.

**Key types:** `ResolvedFuncSig`, `ResolvedFuncEnv`, `CallEdge`.

**Key functions:** `collect_func_call_edges`, `func_reaches_self`,
`topo_resolve_loop`, `resolve_func_sigs`.

**Notable patterns:**
- Recursive graph walk for calls
- Cycle members must have return annotations (`MissingAnnotation`)
- Fuel-bounded recursion in `topo_resolve_loop`

**Assessment:** Clean, well-scoped algorithm.

---

## 04_env.dag — Type Environment (~133 lines)

**Role:** `TypeEnv` and `TypeBinding`; lookup; recursive-type and
inductive-field maps; merging envs from imports/parents.

**Key types:** `TypeEnv`, `TypeBinding` (name, resolved — the narrow
binding that Theme 1 wants to enrich with provenance).

**Key functions:** `lookup_type`, `merge_envs`, `is_recursive_type`,
`inductive_fields_for`, `put_inductive_field`.

**Notable patterns:**
- Thin data layer
- `merge_envs` picks first `intern_table` intentionally (performance)
- **`TypeBinding` is the bottleneck** — only carries name + resolved type,
  no provenance/ownership (Theme 1/3 target)

**Assessment:** Small file, big leverage point. Enriching `TypeBinding`
is the structural fix for Themes 1 and 3.

---

## 04_items.dag — Item Classification (~150 lines)

**Role:** Item classification, metadata (`ItemInfo`), `TypedModule` /
`TypedGraph` / `ResolvedGraph` shapes.

**Key types:** `ItemKind` (`FnItem | … | OtherItem`), `ItemInfo`,
`TypedModule`, `TypedGraph`, `ResolvedGraph`.

**Key functions:** `item_kind`, `variant_locals_from_items`.

**Notable patterns:**
- `item_kind` is a heuristic ordering of Node fields (connective,
  transport, body, uses, params, type_annotation)

**Assessment:** Small bridging file.

---

## 04_lookup.dag — Field/Method Lookup (~346 lines)

**Role:** Field and method resolution on types; scrutinee normalization;
Tier 0 structural methods + Tier 1 service methods.

**Key types:** `KnownMethodResolution`, `MethodFieldResult`.

**Key functions:** `lookup_field_type_node`,
`resolve_scrutinee_type_node`, `field_summary_for_type`,
`lookup_structural_method`, `resolve_known_method_node`,
`check_service_method_call_node`.

**Notable patterns:**
- Optional unwrapping and `field == "value"` special case
- `enrich_kernel_type` + `kernel_algebra_profile` for algebra metadata
- Explicit two-tier dispatch (structural then service)

**Assessment:** Reasonable. The `field == "value"` special case is a
string-keyed pattern that Theme 2 would clean up.

---

## 04_method.dag — Builtin Functions (~113 lines)

**Role:** Builtin free-function return types and small type constructors.

**Key functions:** `builtin_function_registry`,
`infer_builtin_call_type`, `resolve_builtin_call_type`.

**Notable patterns:**
- **Hand-maintained** `map_insert` chain for builtin names → return
  Node types
- Documented as bridge to be replaced by real `.dag` definitions
- Stage0 caches registry in Rust

**Assessment:** Known technical debt, explicitly documented.

---

## 04_access.dag — Index/Slice Checks (~129 lines)

**Role:** Type checks for index and slice operations.

**Key functions:** `check_index_access_node`, `check_slice_access_node`.

**Notable patterns:**
- String vs Map vs ordered element collections
- Slices restricted to strings

**Assessment:** Small, focused, clean.

---

## 04_patterns.dag — Pattern Matching (~242 lines)

**Role:** Pattern subject classification, variant/field lookup for
patterns, match exhaustiveness (coproducts and Optional).

**Key types:** `PatternSubject` (`PatternResolved | PatternDynamic |
PatternLookupBlocked`).

**Key functions:** `pattern_subject_from_node`, `lookup_variant_in_type`,
`lookup_field_in_variant`, `check_match_exhaustiveness`.

**Notable patterns:**
- Optional modeled as Some/None names for exhaustiveness
- Bool literals map to True/False
- `synthesize_optional_some_variant` called out as bridge

**Assessment:** Reasonable. The Optional synthesis is necessary for now.

---

## 04_service.dag — Service Inference (~250 lines)

**Role:** Detect typed service calls, collect transitive service
dependencies, registry lookups.

**Key functions:** `is_typed_service_call_receiver` (A–Z first char
heuristic), `collect_typed_service_calls`,
`expand_transitive_services`, `check_service_method_call_node`.

**Notable patterns:**
- **ASCII A–Z heuristic** on field name for service detection
- Fixpoint expansion with pass limit
- Service path strings as Map keys

**Assessment:** The A–Z heuristic is a naming convention, not structural.
Could be improved but low priority.

---

## 04_cycle.dag — Type Cycle Detection (~156 lines)

**Role:** Kahn-style cycle detection on type dependency graphs.

**Key functions:** `detect_type_cycles_kahn`, `kahn_remove_loop`,
`kahn_cycle_drain`.

**Notable patterns:**
- Pure graph algorithms; self-edges handled separately
- Fuel parameter on drain

**Assessment:** Clean algorithm, well-isolated.

---

## 04_emit_info.dag — Emit Info Precomputation (~398 lines)

**Role:** Precompute `EmitGraphInfo`: type summaries, struct/enum field
summaries, variant maps.

**Key types:** `TypeRepr` (`StructRepr | EnumRepr { unit_only }`),
`TypeSummary`, `EmitGraphInfo` (large struct: summaries, recursion,
sharing, ownership indices, variant maps…).

**Key functions:** `build_type_summary`, `add_emit_item_summary`,
`derive_variant_to_enum`, `build_struct_field_summaries`,
`build_enum_field_summaries`.

**Notable patterns:**
- Enum variants with fields get separate `TypeSummary` entries
- Ambiguous variant names get `""` sentinel
- Filters exclude `"Dynamic"` and type variables from field_type_map

**Assessment:** Bridge between inference and emission. The sentinel
pattern and Dynamic filter are string-keyed heuristics.

---

## 05_emit.dag — Shared Emitter (~3,003 lines)

**Role:** Target-agnostic emitter: imports, types, literals,
`RenderTarget`-dispatched helpers, unified expression/TCO/service/transport
rendering for Python/Go (and building blocks Rust uses).

**Key types:** `EmitResult`, `BlockEmitState`, `TcoFrame`,
`BackendCapability` / `BackendInfo`, `ServiceFieldSet`.

**Key functions:** `emit_simple_expr`, `emit_unified_typed_expr`,
`emit_literal`, `emit_node_type`, `render_node_type`,
`emit_container`, service helpers (`compute_service_fields`,
`effective_operation_transport`), naming helpers (`to_snake`,
`to_pascal`, `apply_naming_case`), JSON helpers.

**Notable patterns:**
- Central `match target` / `language_spec(target)` for keywords and
  templates
- No single `emit()` entry — pipeline dispatches to per-target backends
- Heavy reuse of string templates and `RenderTarget` branching
- Coercion hooks via `v2.compiler.coercion`

**Assessment:** Theme 4 target. The `match target` branching is what
the single-emitter work aims to eliminate by reading LanguageSpec only.

---

## 05_emit_rust.dag — Rust Backend (~5,894 lines)

**Role:** Full Rust backend: types, serde, services, ownership-aware
params, TCO, `lib.rs` / `v2_rt` / `compiler_tests` wiring.

**Key functions:** `emit_rust` (entry), `build_shared_types`,
`build_ownership_results`, `emit_module` / `emit_module_full`,
struct/enum/func emitters, `emit_rust_expr_*`, TCO stack, `emit_v2_rt_module`,
`emit_compiler_tests_module`, `emit_lib_rs_from_files`,
`render_rust_type`.

**Notable patterns:**
- **Largest emitter** — not on the unified path (Python/Go are)
- Imports ownership module directly
- Duplicates helpers that exist in `05_emit.dag`
- String-heavy Rust source generation

**Assessment:** The big outlier. 5,894 lines vs Python 666 / Go 689.
Phase 6 of the single-emitter work (blocked on LS-4 borrow model) is
what brings this in line. Theme 3 (ownership) and Theme 4 (emission as
data) both target this file.

---

## 05_emit_python.dag — Python Backend (~666 lines)

**Role:** Python backend: dataclasses, `__init__.py`, `requirements.txt`,
tests, aiohttp-style service bodies. Delegates most expression work to
`emit_unified_*` from `05_emit.dag`.

**Key functions:** `emit_python`, `emit_py_module`, `emit_py_typed_item`,
`emit_py_typed_expr`, `emit_py_service_def`, `emit_py_test_file`.

**Notable patterns:**
- Thin — delegates to unified emitter
- Parallel structure to Go

**Assessment:** Good alignment with Theme 4. Shows what Rust should
look like after Phase 6.

---

## 05_emit_go.dag — Go Backend (~689 lines)

**Role:** Go backend: `go.mod`, package naming, struct/interface-style
sums. Same unified-emit pattern as Python.

**Key functions:** `emit_go`, `emit_go_module`, `emit_go_typed_item`,
`emit_go_typed_expr`, `emit_go_service_def`, `emit_go_test_file`.

**Notable patterns:**
- **Near-mirror of Python** file layout
- Go-specific multi-return/error flavor
- `go_mock_expr_uses_fmt` — string split heuristic for `fmt` import

**Assessment:** Good alignment with Theme 4. The `fmt` heuristic is
minor tech debt.

---

## complexity.dag — Complexity Analysis (~5,489 lines)

**Role:** Symbolic work/span complexity, cost interning, parser
termination (progress witnesses, SCCs, call graphs), recursion contexts,
`build_complexity_report`.

**Key types:** `SizeExpr`, `CostExpr`, `Certainty`,
`ComplexitySummary`, `CostInternTable`, `CallEdge`, `SccInfo`,
`ParserProgressEdge` / `ParserProgressEnv`, `FuncEntry`,
`RecursionContext`, `ComplexityReport`, `ComplexityViolation`.

**Key functions:** `get_or_compute_summary`,
`build_complexity_report`, parser progress subsystem, SCC and
self-call analysis.

**Notable patterns:**
- Imports std algebra, termination, computation, graph, parse witnesses
- Evidence-heavy, many internal accumulators
- `annotate_descent_evidence` has ~33 heuristics (340 CX violations)
- `classify_self_call_evidence` is the CX fallback

**Assessment:** Second-largest file. Theme 1's provenance-on-bindings
work directly targets the heuristic classification system here. Steps
1-4 in the CX roadmap aim to delete the ad-hoc classification functions
and replace them with proof constructors reading TypeBinding.provenance.

---

## ownership.dag — Ownership Analysis (~635 lines)

**Role:** Per-function binding usage (Consumed/Read/Threaded/Projected),
fan-out vs semantic consumers, movable Rc vs move, fold accumulator
unwrap proofs.

**Key types:** `EdgeKind`, `BindingUsage`, `OwnershipDecision`,
`FoldAccUnwrapProof`, `OwnershipProof`.

**Key functions:** `walk_expr`, `make_decision`, `build_movable_set`,
`build_read_only_params`, fold helpers, `analyze_ownership`.

**Notable patterns:**
- Explicit **no-heuristic** stance in header comments
- Branch merge uses max fan-out by binding
- Fold detection via string name matching ("init" arg, method name "fold")
- **Name-keyed** `BindingUsage` (Theme 3 target: move to TypeBinding)

**Assessment:** Theme 3 target. The name-keyed approach and fold string
matching are what gets replaced by SVR on bindings. The explicit
no-heuristic stance is good — the string matching is a data-flow gap,
not intentional heuristics.

---

## coercion.dag — Type Coercion (~297 lines)

**Role:** Target dispatch over `TypeCheckpoint` / `InhabitantDecl` /
`CallableRepr` from extdeps + std.coercion; cast syntax; auto-generated
coercion test entries.

**Key functions:** `target_checkpoints`, `target_inhabitants`,
`coerce_primitive_type`, `render_cast`, `lookup_inhabitant`,
`extract_coercion_tests`.

**Notable patterns:**
- `match target` for Rust/Python/Go/Dag
- Data tables live in `extdeps/languages/*/types.dag`
- `coerce_primitive_type` documents **fail-open** on miss (transitional)
- Tests are **data-driven** from the same tables

**Assessment:** Good structure. The fail-open is documented transitional
debt. The data-driven test generation is good alignment with the thesis.

---

## languages.dag — Language Specifications (~1,163 lines)

**Role:** Compiler-facing `LanguageSpec` built from extdeps constants;
per-target spec functions and accessors.

**Key types:** `LanguageSpec` (large record), `SharingStrategy`,
`BlockSyntax`, `ForEachSyntax`, `NamingCase`, `TcoSyntax`,
`ServiceMethodStrategy`, many more.

**Key functions:** `rust_spec`, `python_spec`, `go_spec`, `dag_spec`,
`language_spec_for_target`, `target_keyword`, `binop_symbol`.

**Notable patterns:**
- **Hand-assembled** records per language from extdep constants
- Bootstrap can't import all of `std.languages` directly, so some
  types are mirrored
- Central place for string templates consumed by emit

**Assessment:** This is the data that Theme 4 wants emitters to read
instead of making decisions. Good existing structure, just needs to
be the *only* authority (no inline target knowledge in emitters).

---

## Support files

### effect_derivation.dag (~66 lines)
Pure forwarding bridge — re-exports `std.effects` and `std.http_path`
into `v2.compiler` for stage0/tests. No logic.

### artifact.dag (~113 lines)
`RenderTarget` enum, artifact/boundary/plan model. Mostly data modeling.
Single-artifact assumption documented.

### trace.dag (~223 lines)
Trace/SpanMapping/SourceMap for runtime traces. Pure functional updates.
No compiler I/O.

### runtime_rust.dag (~279 lines)
Single authority for generated `v2_rt.rs` text: concat, strings,
collections, Rc helpers, scanners, unicode, filesystem. Large string
concatenation of Rust source.

### compiler_tests_rust.dag (~1,260 lines)
Generated `compiler_tests.rs` harness. Hardcoded lists of `dsl/...`
paths for self-compile and gist closures. Ties coercion registry to
generated Rust assertions.

---

## Size summary

| File | Lines | Theme alignment |
|------|------:|-----------------|
| 00_core.dag | 1,703 | Theme 2 (Node.name), Theme 4 (core tables) |
| 01_tokenize.dag | 523 | Clean |
| 02_parse.dag | 4,828 | Theme 1 (output provenance) |
| 03_resolve.dag | 462 | Clean |
| 03_normalize.dag | 91 | Clean |
| 04_infer.dag | 5,470 | **Theme 1** (classify_* system, provenance) |
| 04_resolve.dag | 992 | — |
| 04_types.dag | 992 | Theme 2 (name-driven checks) |
| 04_sigs.dag | 262 | Clean |
| 04_env.dag | 133 | **Theme 1** (TypeBinding is the bottleneck) |
| 04_items.dag | 150 | — |
| 04_lookup.dag | 346 | Theme 2 (string-keyed field lookup) |
| 04_method.dag | 113 | Known debt (hand-maintained registry) |
| 04_access.dag | 129 | Clean |
| 04_patterns.dag | 242 | — |
| 04_service.dag | 250 | Minor (A-Z heuristic) |
| 04_cycle.dag | 156 | Clean |
| 04_emit_info.dag | 398 | Minor (sentinel patterns) |
| 05_emit.dag | 3,003 | **Theme 4** (match target branching) |
| 05_emit_rust.dag | 5,894 | **Theme 3+4** (ownership + single emitter) |
| 05_emit_python.dag | 666 | Good (unified path) |
| 05_emit_go.dag | 689 | Good (unified path) |
| complexity.dag | 5,489 | **Theme 1** (340 CX violations, heuristics) |
| ownership.dag | 635 | **Theme 3** (name-keyed, string fold detection) |
| coercion.dag | 297 | Good (data-driven) |
| languages.dag | 1,163 | **Theme 4** (spec authority) |
| effect_derivation.dag | 66 | Bridge |
| artifact.dag | 113 | Data model |
| trace.dag | 223 | Clean |
| runtime_rust.dag | 279 | Support |
| compiler_tests_rust.dag | 1,260 | Support |
| compile.dag | 1,066 | Orchestration |
| **Total** | **~37,812** | |

---

## Where the mass is

The top 5 files account for ~24,684 lines (65% of the compiler):

1. **05_emit_rust.dag** (5,894) — Rust-specific emission, not on unified path
2. **complexity.dag** (5,489) — CX analysis with heuristic classification
3. **04_infer.dag** (5,470) — main inference with triple classification system
4. **02_parse.dag** (4,828) — parser (proportional to surface syntax)
5. **05_emit.dag** (3,003) — shared emitter with target branching

Files 1-3 are the primary simplification targets. File 4 is large but
structurally justified. File 5 shrinks as language specs become the sole
authority.

---

## Gap analysis: proposals vs current compiler

The docs on this branch (`compiler-ideal-vs-actual.md`,
`binding-model-proposal.md`, `binding-unification-design.md`,
`dag-vocabulary-reconciliation.md`, `compiler-reduction-plan.md`)
lay out a clear first-principles architecture. Here's how the
current v2 compiler compares.

### The ideal (from the proposals)

4 stages, not 5+:
```
Parse → Resolve → Prove → Emit
```

- **Parse:** source text → Node tree with spans.
- **Resolve:** modules connected, names resolved, generics expanded.
- **Prove:** ONE stage. Walks graph, at each binding site computes
  all dimension values (SVR, ownership, effects) using each dimension's
  `compose` function. Carries values on bindings. For each proof
  strategy in `std/`, executes: traverse graph, compose with algebra,
  check gate.
- **Emit:** reads LanguageSpec + proofs. Never decides. Mechanical.

The compiler doesn't know about complexity, ownership, or effects.
These are dimension facts in `std/`. The compiler reads proof
strategies and executes them: `fold` over graph, compose with
algebra, check gate. One mechanism, N dimensions.

### The actual (current v2)

6+ conceptual stages across 32 files:
```
Tokenize → Parse → Resolve → Normalize → Infer → CX → Ownership → Emit
```

- **Infer** is the bottleneck: 5,470 lines. It types expressions,
  AND classifies provenance, AND annotates descent evidence, AND
  builds emit info. These are tangled.
- **CX** (complexity.dag, 5,489 lines) is a **parallel re-derivation
  engine**. It reconstructs SVR from AST structure because bindings
  don't carry it. 33 heuristics, 340 violations.
- **Ownership** (635 lines) is a **separate pass** that walks the
  typed body after inference, using string name matching for folds.
- Infer → CX → Ownership are three stages that the proposal says
  should be one ("Prove").

### The specific gaps

**1. TypeBinding is too narrow (04_env.dag)**

```dag
// Current — only carries type
type TypeBinding {
  name: String
  resolved: Node
}

// Proposed — carries everything
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // Theme 1
  ownership: OwnershipKind       // Theme 3
  // future: effects, user-defined dimensions
}
```

This is the single structural bottleneck. Everything downstream
(classify_*, annotate_descent_*, ownership string matching) exists
because TypeBinding doesn't carry the facts forward.

**2. The classify_* system (04_infer.dag)**

~1,200 lines of code that re-derive SubValueRelation from AST:
- `classify_binding_provenance` (~30 lines) — at bind time
- `classify_let_value` (~100 lines) — for descent evidence
- `classify_argument` (~170 lines) — for call arguments
- `classify_body_provenance` (~150 lines) — for output provenance
- `annotate_descent` (~260 lines) — walk body to build sub_value_vars
- `DescentContext` threading (~100 lines) — parallel SVR map
- Various fallbacks (~100+ lines)

The proposal: compute SVR once at binding creation, carry it on
TypeBinding.provenance. All of the above become thin readers.

**3. CX heuristics (complexity.dag)**

`classify_self_call_evidence` and `collect_evidence_incremental`
reconstruct what SVR on bindings would provide directly. ~380 lines
of reconstruction dissolve.

**4. Ownership string matching (ownership.dag)**

- `fname == "fold"` / `mname == "fold"` to detect fold patterns
- `a.name == "init"` to find fold accumulator
- Duplicate ExprCall/ExprMethodCall fold logic

The proposal: fold passes `PreservedValue` SVR to accumulator and
`IteratedSubValue` to element. Ownership reads SVR from bindings.
String matching dissolves.

**5. Rust emitter isolation (05_emit_rust.dag)**

5,894 lines — not on the unified path. Python (666) and Go (689)
delegate to `emit_unified_*`. Rust has its own everything because
it needs ownership facts. The proposal: once ownership is a
dimension on bindings, the Rust emitter reads the same proofs as
Python/Go, just with different LanguageSpec rendering.

### The numbers (from compiler-reduction-plan.md)

| Category | Current | Dissolves | Remaining |
|----------|--------:|----------:|----------:|
| Parse | 4,827 | ~280 | ~4,547 |
| Resolve | 552 | ~91 | ~461 |
| Inference | 9,643 | ~1,600 | ~8,043 |
| Proof (CX+ownership) | 6,124 | ~525 | ~5,599 |
| Emission | 10,252 | ~320 | ~9,932 |
| Infrastructure | 6,680 | ~100 | ~6,580 |
| **Total** | **38,078** | **~2,916** | **~35,162** |

~2,900 lines exist ONLY because facts aren't on edges. That's ~8%.
The remaining ~35,000 is the actual compiler for the thesis.

The ideal target is ~24,730 lines across ~13 files (from
compiler-ideal-vs-actual.md). The gap between 35,000 and 25,000
is largely the Rust emitter and dimension facts that should move
from compiler to `std/`.

---

## Three questions about 00_core.dag

### Q1: Why are Token/TokenShape in core?

**They shouldn't need to be.** Actual consumers:

| File | Usage |
|------|-------|
| 01_tokenize.dag | Producer (defines tokens) |
| 02_parse.dag | Consumer (reads tokens) |
| compile.dag | Threads `List<Token>` between tokenize → parse |
| compiler_tests_rust.dag | Emits test assertions about token shapes |
| 04_infer.dag | **Comments only** — mentions Token for illustration |
| 05_emit_rust.dag | **Comments only** — mentions TokenShape for name collision context |
| effect_derivation.dag | Different type — `UrlPathToken` from `std.http_path` |

Token/TokenShape are consumed by exactly 2 pipeline stages (tokenize,
parse) plus the pipeline orchestrator and test generator. They could
live in `01_tokenize.dag` and be imported by the others. The reason
they're in core is the flat-module convention: everything is defined
in `00_core` and imported everywhere. This is a modularity gap —
later stages don't need to know about tokens.

### Q2: Node defined up front — shouldn't IR be later-stage?

**This is the fundamental design tension.** Currently, `Node` is the
universal IR from parse through emit. The parser directly constructs
`Node` values (70 calls to `make_expr_node`, `make_param_node`,
`make_field_node`, etc. in `02_parse.dag`).

The consequence: `Node` has 18 fields, most of which are irrelevant
at parse time but must be set anyway:
- `inferred` — always `none` at parse time
- `is_self_recursive` — always `false` at parse time
- `has_non_tail_self_call` — always `false` at parse time
- `return_cardinality` — always `Required` at parse time
- `expr_data` — filled by parser, refined by inference

The proposals don't directly challenge this — the "parse → resolve →
prove → emit" model still uses Node as the universal IR. But the
question is valid: should the parser produce a thinner tree (just
syntax) that later stages enrich? The current approach means every
stage sees every field, even when they're irrelevant.

The `make_*` constructors in `00_core.dag` paper over this — they
set defaults for fields the parser doesn't care about. But it means
the Node type is doing double duty as both "parsed syntax tree" and
"typed/annotated IR." A stage-specific IR would be cleaner but would
require conversion at each stage boundary.

### Q3: ExprData — the branching multiplier

`ExprData` is a 22-variant coproduct on every Node:

```dag
type ExprData
  = NoExprData          // non-expression nodes (types, items, etc.)
  | ExprLiteral         // 42, "hello", true
  | ExprError           // parse/semantic errors
  | ExprVar             // variable reference (carries VarBindingKind)
  | ExprFieldAccess     // x.field
  | ExprCall            // f(args) (carries CallSemantics, descent_evidence)
  | ExprMethodCall      // x.method(args) (carries MethodSemantics)
  | ExprMatch           // match scrutinee { arms }
  | ExprIf              // if cond { then } else { else }
  | ExprLet             // let x = expr
  | ExprRecordLit       // Foo { field: value }
  | ExprListLit         // [a, b, c]
  | ExprBinOp           // a + b (carries BinOp, AlgebraFieldKind)
  | ExprUnaryOp         // !x, -x
  | ExprLambda          // (params) => body (carries LambdaSemantics)
  | ExprStringInterp    // "hello {name}"
  | ExprBlock           // { stmts; expr }
  | ExprCast            // x as Type
  | ExprForEach         // for x in list { body }
  | ExprIndex           // x[i]
  | ExprSlice           // x[start..end]
  | ExprReturn          // return expr
```

**The branching cost:** every downstream consumer must `match expr_data`
over these variants. The counts:

| File | `match expr_data` sites |
|------|------------------------:|
| 04_infer.dag | 98 |
| complexity.dag | 100 |
| 05_emit_rust.dag | 64 |
| 04_resolve.dag | 21 |
| 05_emit.dag | 20 |
| 04_types.dag | 16 |
| ownership.dag | 14 |
| 04_service.dag | 8 |
| 05_emit_python.dag | 6 |
| 05_emit_go.dag | 6 |
| 02_parse.dag | 63 |
| 00_core.dag | 35 |
| **Total** | **~451** |

**451 match sites.** Adding a 23rd ExprData variant touches every one.

**The proposal's answer: binding unification (7 → 2).**

Three of these variants are candidates for desugaring:
- `ExprForEach` → desugars to fold (ExprCall + ExprLambda)
- `ExprLambda` (3 sub-variants via LambdaSemantics) → one form
- Match arm bindings → ExprLet + field access

Per the branching count above, `ExprForEach` + `ExprLambda` appear:
- 15 times in `04_infer.dag`
- 14 times in `complexity.dag`
- 10 times in `ownership.dag`
- 12 times in `05_emit_rust.dag`
- 7 times in `05_emit.dag` + `05_emit_python.dag` + `05_emit_go.dag`

~58 match arms across the compiler. Desugaring eliminates those arms
in CX, ownership, and emission (the downstream consumers). Inference
keeps them for error quality (Option B: desugar late).

**But the deeper issue:** ExprData is a surface-syntax discriminant
pasted onto the universal IR. The thesis says Node + edges are the
primitives. ExprData is 22 things pretending to be one thing. Each
variant carries different metadata types (VarBindingKind, CallSemantics,
MethodSemantics, LambdaSemantics, BinOp, AlgebraFieldKind) — these
are stapled onto ExprData variants rather than being structural
properties of edges.

The `compiler-ideal-vs-actual.md` doc envisions the compiler not
knowing about individual expression forms — it reads proof strategies
and executes them generically. That's a more radical departure than
binding unification: it would mean ExprData variants stop being the
dispatch key for downstream stages, and SVR-on-edges becomes the
dispatch key instead.

---

## Key structural patterns across the compiler

### Construct-discard-reconstruct (ROADMAP diagnosis)
- **Producer:** `04_infer.dag` computes provenance/descent at binding time
- **Bottleneck:** `TypeBinding` only carries `{ name, resolved }` (04_env.dag)
- **Consumers reconstruct:** `complexity.dag` (33 heuristics),
  `ownership.dag` (string name matching), `05_emit_rust.dag` (compensates)

### String-keyed authority (Theme 2 target)
- `Node.name` reads throughout (~15 remaining per ROADMAP)
- `authored_name_at` in `04_types.dag`, `04_lookup.dag`
- `find_child_named` for algebra field dispatch
- Fold detection via method name "fold" in `ownership.dag`
- Builtin registry in `04_method.dag`

### Per-target branching (Theme 4 target)
- `match target` in `05_emit.dag`, `coercion.dag`
- `rust_spec()` / `python_spec()` / `go_spec()` in `languages.dag`
- Rust emitter is entirely separate (5,894 lines vs 666/689)

### Hand-maintained tables (Theme 4 / core table dissolution)
- `expr_child_roles` in `00_core.dag`
- `node_field_roles` in `00_core.dag`
- `function_size_effects` (complexity-related)
- `builtin_function_registry` in `04_method.dag`
- `single_punct` in `01_tokenize.dag` (justified — it's a tokenizer)
