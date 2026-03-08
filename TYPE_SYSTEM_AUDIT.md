# Type System Hardcoding Audit — DAG-Compositional Migration Plan

**Generated:** 2026-03-08  
**Scope:** Every file in `/workspace/src` where types are hardcoded, string-matched, or special-cased rather than being driven by DAG structure.

---

## Summary Metrics

| Pattern | Files | Total Occurrences | Severity |
|---------|-------|-------------------|----------|
| String-literal type names (`"String"`, `"Bool"`, etc.) | 70+ | ~1,500+ | **Critical** |
| `TypeId::*()` typed constructors | 1 | 19 | Low (already centralized) |
| `TypeCategory::*` enum | 1 | 20 | Medium |
| `SemanticCarrierKind` / `SemanticCarrierClass` | 7 | 70 | **High** |
| `ValueBacking::*` enum | 6 | 101 | **High** |
| `PlatformRepr` / `TypeShape` | 5 | 42 | Low (already DAG-structural) |
| `type_lib::*` factory functions | 7 | 185 | Medium |
| `map_abstract_type` / type mapping tables | 11 | 45 | **High** |
| `WrapperKind::*` (outside type_op.rs) | 5 | 69 | Medium |
| `BaseType::*` enum | 2 | 21 | Low |

**Total hardcoded type-system sites: ~2,072 occurrences across ~80 files**

---

## Phase 0: Foundation (`00_foundation/ir/src/`)

### 1. `types.rs` — The Hardcoding Epicenter

**Lines affected:** ~280 (lines 892–1190)  
**Occurrences:** 23 string literals + 19 TypeId constructors + 20 TypeCategory + 43 SemanticCarrierKind/Class + 17 ValueBacking

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `TypeId::bool()`, `::string()`, etc. (12 constructors) | 899–940 | Named constructors producing `TypeId("Bool")` etc. | **Easy** — Replace with registry lookups or `TypeId::from_dag(dag)`. These are convenience aliases, not logic. |
| `TypeId::category()` match | 945–959 | Classifies by string: `"Bool" \| "String" \| ...` → Primitive, `starts_with("List<")` → Container | **Medium** — Replace with `TypeShape` extraction from registered DAG. The DAG *already* carries this info. |
| `semantic_carrier_kind_for_type_id()` giant match | 1060–1118 | 50+ string arms mapping type names → SemanticCarrierKind | **Hard** — This is the single largest hardcoded table. Must be replaced with metadata annotations on type DAGs (e.g., `TypeOp::Meta(SemanticCarrier(kind))`). |
| `value_compatible_with_type_id()` | 1199–1253 | Runtime value↔type compatibility via string dispatch | **Hard** — Must be replaced with DAG-structural backing queries. Already partially delegated to `TypeRegistry::value_backing()`. |
| `value_backing_for_type_id()` | 1183–1188 | Free function using cached `TypeRegistry` | **Easy** — Already delegates to registry. Just need to deprecate. |
| `parse_map_type_id()`, `parse_unary_generic_type_id()` | 987–1041 | String parsing of `Map<K,V>`, `List<T>`, `Optional<T>` | **Medium** — Replace with `TypeExpr` AST or `TypeShape` container variants. |

### 2. `type_registry.rs` — Registration & Lookup

**Lines affected:** ~300 (lines 279–567 for `register_primitives` + `register_core_types`)  
**Occurrences:** 102 `type_lib::` calls, 164 string literals, 17 WrapperKind, 46 ValueBacking

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `register_primitives()` | 286–295 | Hardcodes 8 primitive types by name | **Easy** — Already the single source of truth. Could be generated from a `.dag` file. |
| `register_core_types()` | 300–567 | Hardcodes ~80 type registrations | **Medium** — This is intentionally centralized. Migration: generate from `types.dag` definitions. |
| `value_backing()` method | 882–963 | Maps TypeId → ValueBacking via string match + coercion path + SemanticCarrierKind fallback | **Hard** — Has three cascading fallback strategies, all string-based. Replace with `TypeShape` structural matching. |
| `parse_wrapper_kind()` | 71–79 | String → WrapperKind mapping | **Easy** — Small, localized. Could be a method on the TypeExpr AST. |
| `render_type_expr()` | 165–183 | WrapperKind → string for type expressions | **Easy** — Inverse of parse, small and localized. |

### 3. `type_op.rs` — Type DAG Operations

**Lines affected:** ~50  
**Occurrences:** 19 BaseType, 5 PlatformRepr, 2 WrapperKind

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `BaseType` enum definition | 337–362 | Defines fundamental data shapes (Unit, Bool, Int, Float, String, Bytes, Json, Secret, List, Option, Map, Named) | **Already DAG-structural** — This is the correct abstraction. BaseType should become the canonical way backends discover type shapes. |
| `WrapperKind` enum definition | 387–401 | Container wrapper kinds (Optional, List, NonEmptyList, Set, NonEmptySet, Map) | **Already DAG-structural** — Used correctly in `TypeOp::Wrap(kind)`. |
| `PlatformRepr` struct | 86–96 | Machine representation hints (bits, signed, float, discrete) | **Already DAG-structural** — This is the Phase 2 design for backends. |

### 4. `type_shape.rs` — Structural Classification (The Target)

**Lines affected:** ~70  
**Occurrences:** 24 PlatformRepr, 4 WrapperKind

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `type_shape()` extractor | 101–177 | Walks `Dag<TypeOp>` → `TypeShape` enum | **N/A — This IS the target.** TypeShape is what all backends should switch to. Currently unused by emit backends. |

### 5. `contract.rs` — Contract Tower

**Lines affected:** ~80  
**Occurrences:** 56 type_lib calls, 33 WrapperKind, 10 PlatformRepr, 23 string literals

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `cardinality()` | 43–58 | WrapperKind → Cardinality mapping | **Already DAG-structural** — Correct approach. |
| `base_type()` | 64–95 | Extracts base type name from Identity node ports | **Low** — Port type names are still strings, but this reads the DAG. |
| `TypeContract::can_safely_coerce_to_with()` | (large) | Predicate entailment + base-type upcast checking | **Medium** — Uses string base-type names for upcast checks. Should use coercion DAG paths. |

### 6. `type_lib.rs` — Type DAG Factory

**Lines affected:** ~100  
**Occurrences:** 6 type_lib self-references, 14 string type names

| Site | Lines | What It Does | Migration Difficulty |
|------|-------|-------------|---------------------|
| `identity("String")`, `identity("Bool")`, etc. | 58–95 | Factory functions building primitive type DAGs | **Easy** — These ARE the DAG constructors. The string names flow into port type annotations. |
| Container constructors (`optional()`, `list()`, etc.) | 306–448 | Build container type DAGs with WrapperKind | **Already DAG-structural.** |

### 7. Other foundation files

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `system_model.rs` | 34 strings, 9 ValueBacking, 5 WrapperKind | System model type mapping for codegen bridge | **Medium** — Uses ValueBacking → Rust type string mapping. Should use TypeShape. |
| `builder.rs` | 70 string literals | Test helpers using `"String"`, `"Int"` in test DAGs | **Low** — Test code only, not production logic. |
| `validate.rs` | 84 string literals | Mostly in tests; some in validation logic for type compatibility | **Low-Medium** — Production validation uses TypeRegistry; strings are mostly in test fixtures. |
| `coerce.rs` | 2 string literals | Type coercion helpers | **Easy** |
| `value.rs` | 5 string literals, 1 ValueBacking | Value type names | **Easy** |
| `value_bridge.rs` | 5 string literals | Value↔type bridge | **Easy** |
| `signature.rs` | 40 string literals | Operation signatures with hardcoded port types | **Medium** — Port types are string-based throughout. |
| `dag.rs` | 6 string literals, 3 type_lib | DAG structure with type annotations on ports | **Low** |
| `entrypoint.rs` | 10 string literals | DAG entrypoint with type annotations | **Low** |
| `boundary.rs` | 6 string literals | Boundary crossing types | **Low** |
| `typed_io.rs` | 9 string literals | Typed I/O port definitions | **Low** |
| `codegen_bridge.rs` | 10 string literals, 1 map_abstract_type | Bridge to codegen with type mapping | **Medium** |
| `symbols.rs` | 15 string literals | Symbol table type annotations | **Low** |
| `patterns/*.rs` | ~116 string literals total | Pattern port type annotations across 8 pattern files | **Medium** — All use string type names for port definitions. |
| `transport/*.rs` | ~22 string literals | Transport type annotations | **Low** |
| `resource/*.rs` | ~21 string literals | Resource type annotations | **Low** |
| `language/traits/type_system.rs` | 22 strings, 2 type matching | Language type system trait with hardcoded type names | **Medium** |
| `language/categories/turing.rs` | 9 strings, 2 map_abstract_type | Turing-complete language type mapping | **Medium** |

---

## Phase 1: Surfaces (`01_surfaces/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `codegen/src/testgen/codegen.rs` | 123 strings, 2 SemanticCarrierKind | Test generation with hardcoded type expectations | **High** — Largest single file outside foundation. Generates test code with string type names. |
| `codegen/src/testgen/obligation.rs` | 14 strings | Test obligation types | **Medium** |
| `codegen/src/testgen/analyze.rs` | 8 strings | Test analysis with type inspection | **Medium** |
| `codegen/src/testgen/probe_observer.rs` | 5 strings | Probe observer type matching | **Low** |
| `codegen/src/testgen/mock_corpus.rs` | 6 strings | Mock corpus type annotations | **Low** |
| `codegen/src/tool_discovery.rs` | 3 strings, 1 type matching | Tool discovery type checks | **Low** |
| `codegen/src/fidelity.rs` | 1 string | Fidelity check | **Low** |
| `codegen/src/cli_gen.rs` | 14 `.as_str()`, 2 type matching | CLI codegen with type-name string matching | **Medium** |
| `cli/src/lib.rs` | 10 strings | CLI type annotations | **Low** |
| `workflow/src/schema.rs` | 5 strings | Workflow schema types | **Low** |
| `workflow/src/planner.rs` | 2 strings | Workflow planner types | **Low** |
| `workflow/src/coordination.rs` | 2 strings | Coordination types | **Low** |
| `daglang-cli/src/compile/tests.rs` | 8 strings | Compiler test fixtures | **Low** |
| `daglang-cli/src/pipeline.rs` | 2 strings | Pipeline type annotations | **Low** |

---

## Phase 2: Pipeline (`02_pipeline/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `daglang-driver/src/lib.rs` | 8 strings | Driver type annotations | **Low** |
| `daglang-driver/src/pipeline.rs` | 3 `.as_str()` | Pipeline string matching | **Low** |

---

## Phase 3: Source / Syntax (`03_source/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `daglang-syntax/src/lib.rs` | 12 strings | Syntax-level type names (parsing) | **Medium** — Parser hardcodes primitive type names for literal inference. |
| `daglang-syntax/src/parser.rs` | 3 strings, 12 `.as_str()` | Parser type name matching | **Medium** — Tokenizer/parser needs to know primitive names for type expressions. |

---

## Phase 4: Semantics / Typecheck (`04_semantics/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `daglang-typecheck/src/lib.rs` | 34 strings, 15 `.as_str()`, 2 type_lib | **Critical hotspot.** Hardcodes primitive type names for: literal typing (`Int`, `Float`, `String`, `Bool`, `Unit`), operator return types, built-in function signatures, and the `PRIMITIVE_TYPES` list. | **High** — The `primitive_types()` function (lines 2275-2283) is a hardcoded list. The `builtin_functions()` table (lines ~1880-1990) hardcodes ~15 function signatures with string type names. Literal type inference (lines 2941-2945) matches on literal variant to produce type name strings. |
| `daglang-typecheck/src/tests.rs` | 7 strings | Test fixtures | **Low** |

---

## Phase 5: Graph / Lower (`05_graph/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `daglang-lower/src/lib.rs` | **58 strings, 345 `.as_str()`, 5 type matching** | **THE LARGEST HOTSPOT.** The lowerer builds DAG nodes with hardcoded port types throughout: `Port::scalar("path", "String")`, `Port::scalar("skip", "Bool")`, `Port::scalar("result", "Unit")`, etc. Also does string matching for type decisions: `type_id == "Secret"`, `ty == "Bool"`. | **Very Hard** — ~8,500 lines with pervasive string type usage. Every node construction uses string type names for ports. Secret detection is string-based. The transport/auth/GCP lowering paths use dozens of hardcoded `"String"`, `"Bool"`, `"Int"` type annotations. |
| `daglang-lower/src/tests.rs` | 11 strings, 21 `.as_str()` | Test fixtures | **Low** |
| `daglang-lower/src/eval.rs` | 3 `.as_str()` | Expression evaluation type matching | **Low** |
| `daglang-lower/src/expr.rs` | 5 `.as_str()` | Expression lowering type matching | **Low** |

---

## Phase 6: Artifacts / Derive (`06_artifacts/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `daglang-derive/src/lib.rs` | 12 strings, 11 `.as_str()` | Derive macro with type-name string matching for code generation | **Medium** |

---

## Phase 7: Emit (`07_emit/daglang-emit/src/`)

### Critical: Type Mapping Tables

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `type_mapping.rs` | **30 map_abstract_type, 20 strings** | Central type mapping tables: `RUST_TYPE_MAPPING` and `GO_TYPE_MAPPING`. Each has ~12 `PrimitiveMapping` entries with hardcoded DSL name → target language type string. | **High** — This is THE place where DSL types get turned into target language types. Must be replaced with `TypeShape` → target type mapping. The `map_abstract_type()` function also does string parsing for `List<T>`, `Optional<T>`, `Map<K,V>` generics. |
| `type_codegen.rs` | 16 strings, 10 `.as_str()`, 2 map_abstract_type | DSL→Rust code IR bridge. `map_primitive()` delegates to `RUST_TYPE_MAPPING` table but also special-cases `"List"` → `"Vec"` and `"Map"` → `"HashMap"`. `type_expr_to_static_rust()` special-cases `"String"` → `"&'static str"`. `is_default_compatible()` checks `name == "Bool"`. | **Medium** — Mostly delegates to type_mapping tables already. |
| `lower_to_ir.rs` | 12 strings, 1 `.as_str()`, 3 map_abstract_type | `map_to_rust_type()` at line 533 has its own inline string match: `"String" \| "Path" → "String"`, `"Bool" → "bool"`, `"Int" → "i64"`. Duplicates type_mapping.rs logic. | **High** — Redundant mapping that should be eliminated. |
| `lower_rust.rs` | 7 strings, 2 `.as_str()`, 2 map_abstract_type | Rust backend uses `map_to_rust_type()` which delegates to `RUST_TYPE_MAPPING`. | **Medium** — Already mostly centralized. |
| `lower_go.rs` | 8 strings, 2 `.as_str()`, 2 map_abstract_type | Go backend uses `map_to_go_type()` which delegates to `GO_TYPE_MAPPING`. | **Medium** — Already mostly centralized. |
| `lower_c.rs` | 12 strings, 1 map_abstract_type | C backend has inline match at line 587: `"String" \| "Path" → CType::Ptr(...)`, `"Bool" → CType::Int(...)`, `"Int" → CType::Int(Fixed(64))`, etc. Does NOT use `type_mapping.rs` tables. | **High** — Completely independent type mapping. Must be unified. |
| `computation.rs` | 30 strings | Computation plan nodes with hardcoded port types | **Medium** — All `Port::scalar("path", "String")` style. |
| `plan.rs` | 22 strings, 26 `.as_str()` | Plan nodes with hardcoded port types and string matching | **Medium** |
| `service_emit.rs` | 9 strings, 3 `.as_str()`, 1 map_abstract_type | Service emission with type mapping | **Medium** |
| `dag_emit.rs` | 2 strings | DAG emission | **Low** |
| `test_gen.rs` | 16 ValueBacking, 4 `.as_str()` | Test generation uses ValueBacking for mock value type selection | **Medium** |
| `test_mock_emit.rs` | 4 strings, 5 `.as_str()` | Test mock emission | **Low** |
| `rust_exec_runtime.rs` | 3 strings, 9 `.as_str()` | Rust runtime execution codegen | **Low** |
| `render_rust.rs` | 4 strings | Rust rendering | **Low** |
| `render_c.rs` | 1 string | C rendering | **Low** |
| `fn_codegen.rs` | 1 string, 6 `.as_str()` | Function codegen | **Low** |
| `backend_harness.rs` | 1 string | Backend harness | **Low** |

---

## Phase 8: Materialize (`08_materialize/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `resolve/src/resolve.rs` | 42 strings, 11 `.as_str()` | Resolution engine with type-name string matching | **Medium** |
| `resolve/src/service_ops/service_ops_impl.rs` | 20 strings, 19 `.as_str()`, 5 type matching | Service operation implementations with type matching | **Medium** |
| `transport/src/system_models.rs` | 6 strings, 5 `.as_str()` | System model type mapping | **Low** |
| `transport/src/test_backend.rs` | 9 `.as_str()` | Test backend with type matching | **Low** |

---

## Phase 9: Execute (`09_execute/exec/src/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `execute/tests.rs` | 72 strings, 3 `.as_str()` | Execution test fixtures with hardcoded type names | **Low** — Test code only. |
| `execute/mod.rs` | 14 `.as_str()` | Execution engine with type matching | **Medium** |
| `freshness.rs` | 5 strings, 3 `.as_str()` | Freshness checking with type matching | **Low** |
| `helpers.rs` | 4 `.as_str()` | Helper functions | **Low** |
| `frame_build.rs` | 6 `.as_str()` | Frame building with type matching | **Low** |
| `topo.rs` | 3 `.as_str()` | Topological sorting | **Low** |
| `display.rs` | 5 `.as_str()` | Display formatting | **Low** |
| `render.rs` | 1 `.as_str()` | Rendering | **Low** |
| `intercept.rs` | 1 `.as_str()` | Intercept handling | **Low** |

---

## Phase 10: Test (`10_test/`)

| File | Occurrences | What It Does | Migration Difficulty |
|------|-------------|-------------|---------------------|
| `test/src/auto_mock.rs` | 3 strings, 12 ValueBacking, 29 `.as_str()`, 2 type matching | Auto-mock generation uses ValueBacking enum + string matching extensively | **Medium** |
| `test/src/window.rs` | 32 strings, 1 `.as_str()` | Window test with type annotations | **Low** |
| `test/src/mock_requirements.rs` | 8 strings, 12 `.as_str()` | Mock requirements with type matching | **Medium** |
| `test/src/corpus.rs` | 3 strings, 6 SemanticCarrierKind | Corpus with semantic carrier checks | **Medium** |
| `test/src/composition.rs` | 5 strings | Composition tests | **Low** |
| `test/src/mock_spec.rs` | 4 strings | Mock spec types | **Low** |
| `test/src/boundary.rs` | 2 strings | Boundary tests | **Low** |
| `test/src/fermi.rs` | 11 `.as_str()` | Fermi estimation test | **Low** |

---

## Top 10 Hardest Migration Targets (Ranked)

1. **`05_graph/daglang-lower/src/lib.rs`** — ~8,500 lines, 58 string type names, 345 `.as_str()` calls. Every node in every lowering path constructs ports with string type names. `type_id == "Secret"` for secret detection. Pervasive.

2. **`00_foundation/ir/src/types.rs` → `semantic_carrier_kind_for_type_id()`** — 50+ string match arms classifying type names into semantic carrier kinds. The largest single match statement.

3. **`07_emit/daglang-emit/src/type_mapping.rs`** — Central type mapping tables that ALL backends depend on. Two independent static tables (Rust, Go) with ~12 entries each.

4. **`07_emit/daglang-emit/src/lower_c.rs`** — Independent inline type mapping that doesn't use the shared tables. Must be unified.

5. **`07_emit/daglang-emit/src/lower_to_ir.rs`** — Duplicate type mapping at line 533 that should delegate to shared tables.

6. **`04_semantics/daglang-typecheck/src/lib.rs`** — Hardcoded primitive types list, literal type inference, and 15 builtin function signatures.

7. **`01_surfaces/codegen/src/testgen/codegen.rs`** — 123 string type occurrences in test generation.

8. **`00_foundation/ir/src/type_registry.rs` → `value_backing()`** — Three cascading fallback strategies for determining runtime value backing, all string-based.

9. **`00_foundation/ir/src/type_registry.rs` → `register_core_types()`** — 80+ type registrations by string name. Not wrong per se, but should be generated from `.dag` definitions.

10. **`00_foundation/ir/src/types.rs` → `TypeId::category()`** — String-based category classification that should use TypeShape.

---

## Migration Strategy Recommendations

### Phase A: Eliminate Duplicate Mappings (Low risk, high value)
- Unify `lower_to_ir.rs` line 533 mapping to use `type_mapping::RUST_TYPE_MAPPING`
- Unify `lower_c.rs` line 587 mapping to use a new `C_TYPE_MAPPING` table in `type_mapping.rs`
- Deprecate `value_backing_for_type_id()` free function in favor of `TypeRegistry::value_backing()`

### Phase B: TypeShape Adoption in Backends (Medium risk, high value)
- Emit backends (`lower_rust.rs`, `lower_go.rs`, `lower_c.rs`) switch from `map_abstract_type(string)` to `match TypeShape` extracted from the type registry
- `type_mapping.rs` tables get replaced by `TypeShape → target_type` functions
- This eliminates the largest source of drift between backends

### Phase C: Semantic Carrier → DAG Metadata (High risk, high value)
- Add `TypeOp::Meta(SemanticCarrier(kind))` to type DAGs during registration
- Replace `semantic_carrier_kind_for_type_id()` string match with DAG metadata query
- This is the hardest change because it touches the type registry initialization, validation, and all callers

### Phase D: Lowerer Port Types (Highest risk)
- The lowerer's use of string type names in port annotations is deeply structural
- Strategy: Ports should carry `TypeId` (already do) which resolves through the registry
- The registry already exists; the issue is that lowerer constructs ports inline with string names
- Could introduce `Port::typed(name, &TypeId)` alongside existing `Port::scalar(name, "String")`

### Phase E: Typecheck Primitives (Medium risk)
- Replace hardcoded `primitive_types()` list with registry query
- Replace literal type inference with a lookup table driven by registry
- Replace builtin function signatures with registered type information

---

## What's Already Right (DAG-Structural)

The codebase already has the correct abstractions in place:

1. **`TypeOp` enum** — Types as DAG operations is the right model
2. **`TypeShape` extractor** — Structural classification from DAGs exists but is unused by backends
3. **`PlatformRepr`** — Machine representation hints are DAG-structural
4. **`WrapperKind`** — Container classification is DAG-structural
5. **`BaseType` enum** — Fundamental shape classification is correct
6. **`TypeContract`** — Contract extraction from DAGs works
7. **`TypeRegistry`** — Central registry with DAG storage and coercion paths
8. **`Cardinality`** — Already fully interval-based, no enum special-casing
9. **`ContentEncoding` lattice** — Already has proper lattice algebra

The gap is that backends and downstream code bypass these abstractions and match on type name strings directly. The infrastructure for DAG-compositional types exists; it just isn't wired through yet.
