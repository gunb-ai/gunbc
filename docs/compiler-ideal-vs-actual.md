> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md)
> See also: [compiler-reduction-plan.md](compiler-reduction-plan.md),
> [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md)

# Compiler: Ideal vs Actual (32-file audit)

For each `.dag` file in `src/v2/`: what would the thesis-pure,
perfectly modeled version look like? Then the delta from reality.

**Methodology:** "Ideal" means: no reconstruction, no string
matching for structural facts, SVR on every edge, no parallel
classifications, no hand-maintained tables, no dual representations.
Every concept exists once. Every fact flows forward. Every proof
is constructed, not validated.

---

## Stage 1: Lexing

### `01_tokenize.dag` (522 lines, 5 types, 27 fns, 1 data)

**Ideal:** A function `tokenize(source, file) -> List<Token>`.
Tokens carry shape + text + span. Keywords resolved from a shared
set. String interpolation handled via nesting depth. Pure, no
reconstruction.

**Delta:**
- Two-char operator dispatch is an `if` chain, not a table. Minor.
- `should_start_interpolation` is 4 ad-hoc character checks.
  Ideally this would be data in the syntax spec.
- `single_punct` table is already data-driven. Good.
- `source_chars: List<Int>` duplicates `source: String` for O(1)
  indexing. Implementation detail, not modeling debt.

**Verdict: ~90% ideal.** Small file, mostly clean. No structural
debt. Low priority.

---

## Stage 2: Parsing

### `02_parse.dag` (4,827 lines, 72 types, 281 fns)

**Ideal:** A recursive-descent + Pratt parser driven by `SyntaxSpec`
data. One entry: `parse(tokens, source_indices) -> ParseResult`.
Surface syntax produces Nodes. No semantic analysis. No string
matching beyond what the spec provides. Types are only
parse-specific state (position, diagnostics, context).

**Delta:**
- **72 `*Result` types** for threading `{tokens, ctx, value, errors}`.
  Ideally a generic `ParseStep<T> = { tokens, ctx, value, errors }`
  or the parser uses a mutable-style state monad. Each result type
  is identical in shape, differing only in the `value` field type.
  The 72 types are the cost of not having parametric types in emission.
- **~54 `is_*_shape` + ~30 `tok_is_*` predicates** are one-line
  wrappers. A single `token_has_shape(t, shape) -> Bool` would
  replace all of them. ~80 lines.
- **Hardcoded keyword strings** in `parse_stmt`, `parse_primary`,
  `parse_op_body_entries`. These overlap with `dag_keyword_set` and
  `SyntaxSpec.item_forms`. Should be spec-driven dispatch.
- **`ParserResultWitness` + `ParserHelperIdentity` + `ParserCallIdentity`**
  (~100 lines) — static analysis for CX parser progress proofs.
  This is a CX concern, not a parser concern. Should live in
  `complexity.dag` or a parser-analysis module.
- **`is_uppercase_start` heuristic** for record literals vs variables.
  Should be resolved structurally (after name resolution knows
  whether a name refers to a type or a value), not by convention.
- **`parse_op_body_entries`** skips unknown `ident:` pairs silently.
  Ideal: fail-closed on unknown fields.

**Verdict: ~70% ideal.** The recursive descent structure is correct.
The debt is in predicate duplication, hardcoded strings, and the
72-type pattern. The witness machinery is misplaced. The uppercase
heuristic is a modeling gap.

---

## Stage 3: Module resolution

### `03_resolve.dag` (461 lines, 10 types, 13 fns)

**Ideal:** Validate module graph: check for duplicates, resolve
imports to targets, validate exports, topological sort. Pure
graph algorithm. No type information.

**Delta:**
- Clean. The 10 types are graph-algorithm state (DepEdge,
  TopoResult, KahnDrainState). The 13 functions are the algorithm.
- Module names are `String`-keyed in the graph. Ideally would use
  `InternTable` IDs, but this is a bootstrap-era trade-off that's
  documented and tracked (Theme 2).

**Verdict: ~95% ideal.** Minimal file, correct separation.

### `03_normalize.dag` (91 lines, 1 type, 2 fns)

**Ideal:** Bare container check (`List` without `<T>`) as part
of resolve, not a separate stage.

**Delta:**
- Separate file/module for 2 functions and 1 type. Should merge
  into `03_resolve.dag`.

**Verdict: 100% ideal in logic, wrong file.** Merge candidate.

---

## Stage 4: Type inference

### `04_env.dag` (133 lines, 2 types, 12 fns)

**Ideal:** `TypeBinding { name, resolved, provenance }` and
`TypeEnv` as the shared scope data structure. Lookup and merge
functions. SVR on every binding from construction.

**Delta:**
- `TypeBinding` already has `provenance: SubValueRelation`. Good.
- `TypeEnv.bindings` is `Map<String, TypeBinding>` — name-keyed.
  Ideally keyed by structural ID (InternTable). Tracked (Theme 2).
- `merge_envs` was the perf bug site (fact re-derivation, now
  fixed). The fix is correct but the function's existence shows
  the boundary-merge pattern that KF-2 should catch.

**Verdict: ~85% ideal.** The types are right. The keying is wrong
(String vs structural ID). The merge function is a boundary
discipline concern.

### `04_types.dag` (992 lines, 2 types, 52 fns)

**Ideal:** Pure Node vocabulary: structural predicates, type
constructors, comparisons, algebra instantiation. No TypeEnv
dependency. Functions are total (take a Node, return structural
facts).

**Delta:**
- Clean separation: no TypeEnv, no InferScope. Good.
- `algebra_field_kind_name` (line ~350) converts `AlgebraFieldKind`
  back to `String` for child lookup. This is the Track 6 remainder
  — structural child IDs would eliminate this.
- `resolved_type_name` reads `Node.name` as fallback. Tracked
  (Theme 2, Node.name deletion).
- Several functions do `authored_name_at` with string comparison
  for field identity. Same Theme 2 issue.

**Verdict: ~80% ideal.** Pure vocabulary separation is correct.
String-based identity (Node.name, field names) is the debt.

### `04_resolve.dag` (992 lines, 12 types, 19 fns)

**Ideal:** Walk TypeEnv, expand named type references, substitute
generics, resolve aliases. Pure tree rewriting driven by TypeEnv.

**Delta:**
- 12 types are all `{node, diagnostics}` result pairs. Same
  72-type pattern as the parser — cost of no parametric results.
- `AliasKind = AliasParameterized | AliasLeaf | AliasPassthrough`
  is a classification that could be structural from the type
  definition rather than computed during resolution.
- `resolve_node_bounded` has a `fuel` parameter for recursion
  depth — termination guard. In the ideal, this is structural:
  the type graph is acyclic (04_cycle.dag verifies), so resolution
  terminates by construction.

**Verdict: ~85% ideal.** The resolution logic is correct. The
fuel parameter is a non-constructive termination guard. The alias
classification could be structural.

### `04_patterns.dag` (242 lines, 3 types, 12 fns)

**Ideal:** Pattern matching: subject extraction, variant/field
lookup, exhaustiveness checking. Structural from the coproduct
definition (Coproduct → set of variants → check coverage).

**Delta:**
- `check_match_exhaustiveness` is correct and structural. Good.
- `lookup_variant_in_type` does string-name comparison for variant
  names. Ideally structural (variant IDs, not name strings).
- `NodeLookupStatus` carries error strings. Ideally carries
  typed diagnostics.

**Verdict: ~85% ideal.** Exhaustiveness is the right algorithm.
String-based variant lookup is the debt.

### `04_lookup.dag` (346 lines, 2 types, 13 fns)

**Ideal:** Scope lookup: locals, func sigs, structural methods,
service methods. Pure dispatch on resolved types.

**Delta:**
- `lookup_structural_method` dispatches on type shape (collection,
  map, string, etc.) to find algebra methods. This is correct —
  it reads from algebra declarations.
- String-keyed lookup throughout (`map_get(scope.locals, name)`).
  Theme 2 debt.
- `resolve_scrutinee_type_node` unwraps Optional for match — a
  small heuristic that should be structural.

**Verdict: ~85% ideal.** Correct separation. String keying is the
debt.

### `04_items.dag` (150 lines, 5 types, 3 fns)

**Ideal:** Item classification (fn, service, type, data, resource)
and the TypedModule/TypedGraph/ResolvedGraph boundary types.

**Delta:**
- `ItemKind` is a useful coproduct: `FnItem | ServiceItem | TypeItem | DataItem | ResourceItem`. Correct.
- `ItemInfo { kind, module_name }` — good, carries qualified identity.
- `ResolvedGraph` includes `emit_graph_info` — this is the
  infer→emit boundary contract. Correct placement.

**Verdict: ~95% ideal.** Small file, clean types, correct boundary.

### `04_access.dag` (129 lines, 2 types, 5 fns)

**Ideal:** Index and slice access type checking. Should be
structural: `container[key]` checks key type against container's
declared key type.

**Delta:**
- Clean and small. The 5 functions check index/slice access.
- `keyed_collection_parts` extracts key/value types from Map/List.
  Correct structural approach.
- **Merge candidate:** could fold into `04_resolve.dag` or
  `04_types.dag` without losing clarity.

**Verdict: ~95% ideal.** Correct but could merge for fewer files.

### `04_service.dag` (250 lines, 3 types, 12 fns)

**Ideal:** Service graph: collect typed service calls from module,
expand transitive service dependencies, check service method calls
against declared operations.

**Delta:**
- `collect_typed_service_calls` walks the typed tree to find service
  references. This is a post-inference pass. Ideally, service usage
  would be recorded DURING inference (when the service method call
  is typed), not re-walked after.
- `UniqueAccum` for dedup — a fold accumulator for unique service
  collection. Clean pattern.
- `check_service_method_call_node` validates against declared ops.
  Correct.

**Verdict: ~80% ideal.** The post-inference re-walk is avoidable.
Service usage should be recorded during inference as a side-product.

### `04_sigs.dag` (262 lines, 5 types, 9 fns)

**Ideal:** Function signature resolution: collect call graph,
topological sort for mutual recursion, resolve signatures in
dependency order.

**Delta:**
- `ResolvedFuncSig` carries `output_provenance: List<Map<String, SubValueRelation>>`.
  This is SVR on function outputs — the right structure.
- `CallEdge { caller, callee }` uses `String` names. Should be
  structural IDs. Theme 2 debt.
- `topo_resolve_loop` handles mutual recursion by SCC processing.
  Correct algorithm.

**Verdict: ~85% ideal.** Correct algorithm. String-keyed edges.

### `04_method.dag` (113 lines, 0 types, 8 fns)

**Ideal:** Should not exist. Builtin functions should be declared
as data in `std/` or `extdeps/`, not hardcoded in the compiler.

**Delta:**
- `builtin_function_registry` is a hand-maintained list of built-in
  function return types (count, first, last, reverse, etc.).
- `infer_builtin_call_type` dispatches by function name string.
- This is the definition of "modeling debt" — builtins should be
  `.dag` declarations with typed signatures, not compiler-internal
  string dispatch.

**Verdict: ~20% ideal.** This file should dissolve entirely into
data declarations in `std/primitives.dag` or similar. The compiler
should read builtin signatures from declarations, not a hardcoded
registry.

### `04_cycle.dag` (156 lines, 1 type, 7 fns)

**Ideal:** Type dependency cycle detection via Kahn's algorithm.
Feeds into `04_resolve.dag` so resolution doesn't expand cycles
infinitely.

**Delta:**
- Correct algorithm. Clean separation.
- `KahnState` is an internal fold accumulator. Fine.
- **Merge candidate:** could fold into `04_resolve.dag` or
  `04_sigs.dag` since it's only consumed by type resolution.

**Verdict: ~95% ideal.** Correct, could merge for fewer files.

### `04_emit_info.dag` (398 lines, 4 types, 19 fns)

**Ideal:** Precompute structural facts about types for emission:
type summaries (product vs coproduct, fields, generics), variant
maps, recursion/sharing analysis. Pure over the typed graph.

**Delta:**
- `EmitGraphInfo` is a large Product with many `Map<String, ...>`
  fields for ownership, fold-eligible, read-only params, etc.
  Some of these should be computed during inference (Theme 3),
  not at the emit boundary.
- `TypeSummary` is well-structured. Good.
- `derive_variant_to_enum` is correctly derived, not hardcoded. Good.
- `build_type_summary` reads from resolved Nodes. Correct.

**Verdict: ~80% ideal.** The type summaries are right. The
ownership-related fields on EmitGraphInfo should move to inference
(Theme 3). The boundary should carry proven facts, not compute
them at emit time.

### `04_infer.dag` (5,470 lines, 30 types, 101 fns)

**Ideal:** The single inference pass: walk the parsed Node tree,
attach SVR + resolved type to every binding/edge, construct proofs
(dimension values) at each binding site, thread proofs through the
IR. One pass. No reconstruction. Every fact computed once, stored
on the binding, consumed downstream.

For each binding site: read `BindingSurface` from the parser,
determine SVR (caller provides for parameters, compose for
let-bindings), store on `TypeBinding.provenance`. Done.

For each function: collect `output_provenance` from body's return
expressions by reading the bindings in the body. No re-derivation.

**Delta:**
- **`classify_argument` (~170 lines):** Re-derives SVR from AST.
  Should read `binding.provenance`. DISSOLVES.
- **`classify_let_value` (~100 lines):** Re-derives SVR for let
  expressions. Should read from the computed SVR at binding
  creation. DISSOLVES.
- **`classify_binding_provenance` (~30 lines):** The one place
  that actually computes SVR at binding time. This is the function
  that STAYS — but it should be the ONLY classification function,
  not one of four.
- **`classify_body_provenance` (~150 lines):** Re-derives SVR for
  output provenance. Should read from bindings in the body.
  DISSOLVES.
- **`annotate_descent` (~260 lines):** Walks the body to build
  `sub_value_vars` — a parallel SVR map. Should be unnecessary
  when SVR is on every binding. DISSOLVES.
- **`DescentContext` type + threading (~100 lines):** Carries the
  parallel `sub_value_vars`, `size_aliases`, `per_field_vars`.
  These are all SVR facts that should be on bindings. DISSOLVES.
- **`lambda_param_provenance` on InferScope (~30 lines):** A
  side-channel for fold element provenance. Should be the SVR
  that the fold call site attaches to the element parameter.
  DISSOLVES.
- **`build_call_evidence` (~30 lines):** Assembles descent evidence
  per call by classifying arguments. Should read SVR from argument
  bindings. SIMPLIFIES to binding reads.
- **`read_arg_provenance` with fallback (~100 lines):** Reads
  binding provenance, falls back to `classify_argument`. The
  fallback DISSOLVES when binding provenance is always set.
- **`SizeExpr = ParamSize | DividedSize` (~30 lines):** Descent
  size alias tracking inside `DescentContext`. DISSOLVES with
  context.
- **~25 `*Result` / `*Accum` types (~300 lines):** All are
  `{value, diagnostics}` pairs. Cost of no parametric types.
  Not modeling debt per se, but type inflation.
- **30 types total:** Ideally ~10 (InferScope, InferResult, the
  essential boundary types, and nothing else).

**Verdict: ~65% ideal.** The inference skeleton is correct. ~1,200
lines of classification/reconstruction code exists only because
SVR isn't consistently read from bindings. This is the single
largest reduction target.

---

## Stage 5: Proof construction

### `complexity.dag` (5,489 lines, 27 types, 159 fns)

**Ideal:** Read SVR from edges. For each self-recursive function:
compose SVR across call arguments into a `TerminationProof`. If
the proof constructs, the function terminates. If not, diagnostic.

The cost algebra (SizeExpr, CostExpr, ComplexitySummary) composes
costs from iteration bounds. SCC analysis groups mutually recursive
functions. Parser progress model proves parser termination.

**Delta:**
- **`classify_self_call_evidence` (~60 lines):** Parallel
  classification of self-call arguments. Should read annotated
  `descent_evidence` from ExprCall. DISSOLVES.
- **`collect_evidence_incremental` (~200 lines):** Incremental
  let/match/if evidence threading. Should read SVR from bindings
  in each scope. DISSOLVES.
- **`construct_termination_proof` fallback (~70 lines):** When
  annotated evidence is insufficient, constructs proof from
  heuristics. DISSOLVES when evidence is always strong.
- **Hardcoded tables:** `function_size_effects` in 00_core.dag
  (consumed here), `is_child_accessor_in_model`,
  `is_tree_size_reducing`, `is_property_contraction` — hardcoded
  function name lists. Should be structural from type declarations
  (type has recursive field → accessor is tree-size-reducing).
  DISSOLVES.
- **Parser progress model (~500 lines):** Specific to proving
  parser functions terminate by tracking position advancement.
  This is correct and necessary — parser functions have a different
  termination argument (position monotonically advances) that
  isn't captured by generic SVR. STAYS.
- **Cost algebra (~400 lines):** SizeExpr, CostExpr composition,
  simplification. This is the actual work CX does. STAYS.
- **SCC analysis (~300 lines):** Tarjan/Kahn for call graph
  components. Essential algorithm. STAYS.
- **`SizeExpr` name collision** with 04_infer.dag. Rename to
  `SymbolicSize` or similar.

**Verdict: ~70% ideal.** The cost algebra, SCC, and parser progress
are real work. ~330 lines of parallel classification dissolves.
~500 lines of parser progress is CX-specific but correct.

### `ownership.dag` (635 lines, 7 types, 19 fns)

**Ideal:** Read SVR + UsageEdge from edges. For each binding:
derive `OwnershipKind` from the dimension table. For each function:
construct `OwnershipProof` (fan-out per binding, last-use
identification, fold accumulator eligibility). No string matching.
No separate AST walk.

**Delta:**
- **`fname == "fold"` / `mname == "fold"` string matching (~45
  lines):** Detect fold calls by method name string. Should be
  structural from `MethodSemantics`. DISSOLVES.
- **`a.name == "init"` string matching (~10 lines):** Find fold
  accumulator argument by name. Should be structural from
  parameter position. DISSOLVES.
- **Duplicate ExprCall/ExprMethodCall handling (~60 lines):** Two
  copies of the fold detection logic. Should be one path that reads
  call structure. DISSOLVES.
- **`analyze_ownership` as separate pass (~30 lines):** Walks the
  typed body after inference. Should be computed during inference
  as a dimension. DISSOLVES.
- **`walk_expr` (~170 lines):** The core AST walk recording
  Consumed/Read/Projected/Threaded per use. This STAYS — but it
  should run during inference, not after.
- **`BindingUsage { name, binding_kind, consumers }`:** Name-keyed.
  Should be structural-ID-keyed. Theme 2 debt.
- **`FoldAccUnwrapProof`:** Specific to Rust emission
  (`Rc::try_unwrap`). Should be derivable from OwnershipKind +
  AccumulatorContract, not a separate type.

**Verdict: ~70% ideal.** The ownership analysis logic is correct.
The debt is string matching, separate pass, and Rust-specific proof
types.

---

## Stage 6: Emission

### `05_emit.dag` (3,003 lines, 12 types, 156 fns)

**Ideal:** Target-agnostic emission kernel. Reads `LanguageSpec`
for syntax/semantics, reads proven facts (type, provenance,
ownership) from IR, mechanical translation. No decisions — only
spec lookups and structural pattern rendering.

**Delta:**
- `emit_shared_expr` dispatches 22 ExprData variants. Each variant
  has a handler. This is inherently necessary — different expression
  forms emit differently. STAYS.
- `ExprCategory` (6 variants) classifies expressions into coarse
  groups. Useful for TCO/block structure analysis. Correct.
- `FuncBodyShape` (3 variants) classifies function body structure.
  Correct.
- `TcoExprShape` (6 variants) classifies TCO-relevant expression
  shapes. Correct.
- **`BlockEmitState { lines, scope }`:** Threads scope through
  block emission. Correct pattern.
- **Test extraction** (`extract_test_projections`): walks typed
  graph for testable operations. Could be structural (mark
  operations as testable during inference), but the current
  approach is a clean post-pass.
- **Import derivation** (`derive_module_imports`): walks typed
  graph for used types. Post-pass — could be structural (record
  imports during inference), but correct.

**Verdict: ~85% ideal.** Well-factored via LanguageSpec. The post-
pass patterns (test extraction, import derivation) are correct but
could be more structural. No major debt.

### `05_emit_rust.dag` (5,894 lines, 5 types, 246 fns)

**Ideal:** Rust-specific emission: module structure, derives, Rc
patterns, cargo config, runtime embedding, compiler test emission.
Reads LanguageSpec + ownership proof + type summaries. No decisions
beyond what the specs and proofs dictate.

**Delta:**
- **Ownership-aware emission (~500 lines):** `build_shared_types`,
  `build_ownership_results`, `RcPatternAnalysis`, `RcMatchAnalysis`.
  This reconstructs ownership facts at emit time. When ownership
  is a dimension on bindings (Theme 3), the emitter reads proven
  facts. ~200 lines SIMPLIFY, ~100 DISSOLVE.
- **`VarBindingKind` matching (~20 lines):** Checks `FunctionValueBinding`
  and `VariantValueBinding` for emission decisions. These are type
  questions, not binding classifications. DISSOLVES (read from type).
- **5,894 lines is large.** Much of it is inherently Rust-specific:
  derive macros, module structure, cargo config, runtime wiring,
  `#[cfg(test)]` generation. This doesn't dissolve — Rust is a
  complex target.

**Verdict: ~75% ideal.** The size is inherent to Rust complexity.
The ownership reconstruction is the main debt (~300 lines). The
VarBindingKind dependency dissolves.

### `05_emit_go.dag` (689 lines, 0 types, 34 fns)

**Ideal:** Go-specific emission, fully driven by LanguageSpec.
Types, functions, services, error tuples. Reads specs, no decisions.

**Delta:**
- Already largely LanguageSpec-driven (Phase 5 unification). Good.
- No per-language types (0 types). Good — all via shared types.
- Some residual Go-specific patterns (multi-return, interface
  assertions) that are spec-data but complex to express.

**Verdict: ~90% ideal.** Close to the thesis target.

### `05_emit_python.dag` (666 lines, 0 types, 33 fns)

**Ideal:** Python-specific emission, fully driven by LanguageSpec.
Dataclasses, type hints, async patterns. Reads specs, no decisions.

**Delta:**
- Already largely LanguageSpec-driven. Good.
- No per-language types. Good.
- `emit_init_py` is Python module structure boilerplate. Necessary.

**Verdict: ~90% ideal.** Close to the thesis target.

---

## Infrastructure

### `00_core.dag` (1,702 lines, 33 types, 159 fns, 24 data)

**Ideal:** The IR type definitions and pure accessors. No tables,
no heuristics. Types carry structural facts. Accessors are
structural (by position, not name).

**Delta:**
- **`Node` has `name: String`** — the tracked debt (Theme 2). Should
  be structural identity via InternTable or positional.
- **`VarBindingKind` (4 variants):** Dissolves into type information
  + edge position. See reconciliation.
- **`Connective = Conj | Disj | NoConnective | Arrow`:** Should
  dissolve into `Product`/`Coproduct` from constructors.dag. 313
  consumer sites.
- **`Cardinality = Required | CardOptional`:** Variant name conflict
  with constructors.dag's `Optional`.
- **`NodeFieldRole` (3 variants):** Should be derived from
  StructuralEdge, not hand-maintained.
- **`expr_child_roles` (50 lines):** Hand-maintained table mapping
  ExprData variant names (strings!) to child positions. Should be
  derived from the type definition.
- **`wrapper_child_roles` (12 lines):** Same pattern.
- **`function_size_effects` (9 lines):** Hand-maintained table
  mapping function names to size effects. Should be structural
  contracts on the functions.
- **`node_field_roles` (7 lines):** Hand-maintained. Should derive
  from edge vocabulary.
- **Transport config key constants (12 lines):** `transport_url_key`,
  `transport_path_key`, etc. Stringly-typed config lookup. Should
  be structural fields.
- **Kernel type sentinels** (`unit_type`, `bool_type`, etc.): These
  are fine — canonical Node instances for primitive types.
- **`InternTable` + `InternResult`:** Good — structural identity.
- **`NewlineIndex` + `LineCol`:** Good — span infrastructure.
- **159 fns:** Many are accessor helpers (`arg_value`, `arm_body`,
  `field_access_base`, etc.) that read children by position using
  `expr_child_roles`. If child positions were structural (typed
  fields on expression types), these would be field reads, not
  function calls. ~60 accessors could dissolve.

**Verdict: ~60% ideal.** The IR types are mostly correct. The
debt is: Node.name, string-keyed tables, string-keyed accessors,
Connective, VarBindingKind, and the accessor function pattern
(should be typed field reads).

### `compile.dag` (1,065 lines, 5 types, 60 fns)

**Ideal:** Pipeline orchestration: wire stages together, gate on
diagnostics, delegate to artifact plan. Plus a clean serialization
layer for the Dag artifact format.

**Delta:**
- Pipeline wiring (~200 lines) is clean. Good.
- JSON serialization of typed graph (~400 lines) is large but
  mechanical. Could be a separate file.
- `emit_artifact` matches on `RenderTarget` to dispatch. Correct.
- `compile_to_resolved` stops after ownership. Correct API.

**Verdict: ~85% ideal.** Clean orchestration. The JSON chunk is
large but correct.

### `languages.dag` (1,163 lines, 29 types, 22 fns)

**Ideal:** `LanguageSpec` as a pure data aggregation of per-target
language facts from `extdeps/languages/*/`. One entry per language
feature per target. The emitter reads this, never decides.

**Delta:**
- 29 types define the LanguageSpec shape: `BlockSyntax`,
  `ForEachSyntax`, `ExpressionSemantics`, `SharingStrategy`,
  `VariantPatternSyntax`, `TcoSyntax`, etc. These are the right
  types — each names a language dimension.
- `rust_spec()`, `python_spec()`, `go_spec()`, `dag_spec()` build
  LanguageSpec instances from extdeps data. Correct.
- **Size is inherent:** each target has ~50 configuration points.
  4 targets × ~50 points = ~200 data items. Plus the 29 types.
  Plus accessors. 1,163 lines is reasonable.

**Verdict: ~90% ideal.** This file IS the thesis ("emission reads
specs") instantiated for language features. Close to target.

### `coercion.dag` (297 lines, 2 types, 22 fns)

**Ideal:** Map .dag algebra types to target language type strings
using data from `std/coercion.dag` and `extdeps/languages/*/types`.
Pure lookup.

**Delta:**
- `target_checkpoints`, `target_inhabitants`, `target_callable`,
  etc. dispatch on `RenderTarget` and lookup from per-language
  type tables. Correct data-driven approach.
- `CoercionTestEntry` supports test extraction. Good.
- `can_cast` and `render_cast` — cast legality and rendering.
  Correct.

**Verdict: ~90% ideal.** Data-driven coercion lookup. Clean.

### `artifact.dag` (113 lines, 8 types, 2 fns)

**Ideal:** Artifact model: RenderTarget, ArtifactPlan, Boundary.
Small, stable contract.

**Delta:**
- `RenderTarget = Rust | Python | Go | Dag`. Correct.
- `ArtifactPlan`, `Artifact`, `PartitionRule`. Correct for future
  multi-artifact story.
- `default_artifact_plan` is trivially correct.

**Verdict: ~98% ideal.** Small, clean, forward-looking.

### `runtime_rust.dag` (279 lines, 0 types, 9 fns)

**Ideal:** Single source of truth for the Rust `v2_rt` runtime
shim. String constants assembled into one `rust_runtime_source()`.

**Delta:**
- Pure string assembly. No imports. No types.
- Content is Rust source code as string literals. Necessarily ugly
  but correct — single authority.

**Verdict: ~95% ideal.** It is what it is — runtime shim text.

### `compiler_tests_rust.dag` (1,260 lines, 0 types, 27 fns)

**Ideal:** Extract test cases from the typed graph and emit Rust
test functions. Driven by type structure (what's testable) and
coercion data (how to construct test values).

**Delta:**
- Large but mechanical. Test extraction + Rust test formatting.
- Could be more data-driven (test templates from LanguageSpec)
  but correct as-is.

**Verdict: ~80% ideal.** Correct, could be more data-driven.

### `effect_derivation.dag` (66 lines, 0 types, 4 fns)

**Ideal:** Should not exist as a separate file. Re-exports
`std.effects` functions for stage0 visibility.

**Delta:**
- 4 re-export functions. Pure bootstrap artifact.
- DISSOLVES when the build system can directly expose `std.effects`
  to stage0 tests.

**Verdict: ~0% ideal as a concept, 100% necessary as bootstrap
scaffolding.** Dissolves when build improves.

### `trace.dag` (223 lines, 7 types, 13 fns)

**Ideal:** Runtime trace/debug contract. Orthogonal to compilation.
Correct to exist, correct to be separate.

**Delta:**
- `SpanMapping`, `TraceEvent`, `TraceFrame`, `Trace`, `TraceFilter`,
  `ReproCase`, `SourceMap`. Clean types for runtime observability.
- Not on the compilation hot path.

**Verdict: ~95% ideal.** Clean, orthogonal, correct.

---

## Summary: distance from ideal

| File | Lines | Verdict | Primary debt |
|------|------:|---------|-------------|
| 01_tokenize | 522 | ~90% | Minor (op table, interpolation) |
| 02_parse | 4,827 | ~70% | Predicate duplication, hardcoded keywords, 72-type pattern, witness misplacement |
| 03_resolve | 461 | ~95% | String-keyed module names |
| 03_normalize | 91 | Wrong file | Merge into 03_resolve |
| 04_env | 133 | ~85% | String-keyed bindings |
| 04_types | 992 | ~80% | String-based identity |
| 04_resolve | 992 | ~85% | Fuel guard, alias classification |
| 04_patterns | 242 | ~85% | String-based variant lookup |
| 04_lookup | 346 | ~85% | String-keyed scope |
| 04_items | 150 | ~95% | Clean |
| 04_access | 129 | ~95% | Merge candidate |
| 04_service | 250 | ~80% | Post-inference re-walk |
| 04_sigs | 262 | ~85% | String-keyed call edges |
| **04_method** | **113** | **~20%** | **Hardcoded builtins — should dissolve into std/ data** |
| 04_cycle | 156 | ~95% | Merge candidate |
| 04_emit_info | 398 | ~80% | Ownership fields should move to inference |
| **04_infer** | **5,470** | **~65%** | **~1,200 lines of classification/reconstruction** |
| **complexity** | **5,489** | **~70%** | **~330 lines parallel classification + hardcoded tables** |
| **ownership** | **635** | **~70%** | **String matching, separate pass, Rust-specific proofs** |
| 05_emit | 3,003 | ~85% | Post-pass patterns |
| 05_emit_rust | 5,894 | ~75% | Ownership reconstruction (~300 lines) |
| 05_emit_go | 689 | ~90% | Close to target |
| 05_emit_python | 666 | ~90% | Close to target |
| **00_core** | **1,702** | **~60%** | **Node.name, string tables, Connective, VarBindingKind, ~60 accessor fns** |
| compile | 1,065 | ~85% | JSON chunk size |
| languages | 1,163 | ~90% | Inherent size |
| coercion | 297 | ~90% | Clean |
| artifact | 113 | ~98% | Clean |
| runtime_rust | 279 | ~95% | Inherent |
| compiler_tests | 1,260 | ~80% | Could be more data-driven |
| effect_derivation | 66 | Bootstrap | Dissolves when build improves |
| trace | 223 | ~95% | Clean |

### The 5 files furthest from ideal

1. **04_method.dag (~20%)** — hardcoded builtins, should be data
2. **00_core.dag (~60%)** — Node.name, string tables, accessors
3. **04_infer.dag (~65%)** — classification/reconstruction
4. **complexity.dag (~70%)** — parallel classification
5. **ownership.dag (~70%)** — string matching, separate pass

### Cross-cutting debt themes

1. **String-based identity** (Node.name, name-keyed maps, string
   comparison for field/variant/function names) — affects ~20 files.
   This is Theme 2. The fix is structural IDs via InternTable.

2. **Fact reconstruction** (classify_*, annotate_descent, parallel
   classification in CX, ownership re-walk) — affects 3 files.
   This is Theme 1. The fix is SVR on every edge.

3. **Hand-maintained tables** (expr_child_roles, function_size_effects,
   builtin_function_registry) — affects 2 files. The fix is
   structural: types carry their own facts.

4. **No parametric result types** (72 parser Result types, 25 infer
   Result types) — affects 2 files. The fix is generic emission
   for `WithDiag<T>` or similar.

5. **Connective duplication** (313 sites matching Conj/Disj vs
   Product/Coproduct) — affects many files. Tracked in reconciliation.
