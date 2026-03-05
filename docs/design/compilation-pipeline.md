# Compilation Pipeline: `.dag` Source to Execution

> Design doc mapping the full compiler pipeline, data shapes at each stage,
> known gaps, and the two runtime paths (interpreted vs compiled).
>
> **Date**: 2026-03-05

---

## Overview

```
                          .dag source files
                                |
                    +-----------+-----------+
                    |                       |
              [INTERPRETED]            [COMPILED]
              (primary path)         (emit path)
                    |                       |
              Parse (syntax)          Parse (syntax)
                    |                       |
           Module Resolve             Module Resolve
                    |                       |
              Typecheck                Typecheck
                    |                       |
                Lower                    Lower
                    |                       |
              Resolve (DynOp)            Emit
                    |                    /  |  \
              Execute (graph)      Rust  Go  C  MIPS
                    |                    |
              HashMap<String,Value>   rustc / gcc / as
                                         |
                                    native binary
```

There are two paths from `.dag` source to running code:

1. **Interpreted** (primary today) — the DAG is compiled to an in-memory graph of
   executable operations (`Dag<DynOp>`) and run directly by the graph executor.
   No codegen step. Used by all current tool binaries.

2. **Compiled** (emit path) — the DAG is lowered, then code is emitted as
   Rust/Go/C/MIPS source. The emitted code is compiled by the target toolchain
   into a native binary. Currently Phase 1 for Rust; service transport code
   exists for all four backends; orchestration/dataflow logic is scaffold-only
   for non-Rust backends.

Both paths share stages 1-4 (Parse through Lower). They diverge after lowering.

---

## Stage 1: Parse

```
Raw .dag text  ──>  SourceFile (untyped AST)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-syntax` |
| **Entry** | `parse(source: &str) -> Result<SourceFile, Vec<ParseError>>` |
| **Input** | Raw `.dag` file contents (string) |
| **Output** | `SourceFile { module_path, imports, items: Vec<Item> }` |

The lexer tokenizes, the parser builds an untyped AST. No semantic analysis.
The `Item` enum carries 19+ declaration forms (TypeDef, FnDef, FuncDef,
ServiceDef, PipelineDef, DataDef, ExternFuncDecl, etc.). Expressions,
statements, type expressions, and pipe methods are all represented.

### Data shape

```
SourceFile
  +-- module_path: Option<ModulePath>    "module tools.makegen"
  +-- imports: Vec<Import>               "import std.render { Span }"
  +-- items: Vec<Item>
        +-- Item::TypeDef { name, body: TypeBody }
        +-- Item::FnDef { name, params, return_type, body }
        +-- Item::FuncDef { name, inputs, outputs, uses, body }
        +-- Item::ServiceDef { name, config, operations }
        +-- Item::DataDef { name, type_expr, value: Expr }
        +-- Item::PipelineDef { name, stages }
        +-- ...
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| Multi-statement brace blocks silently discarded | High | `consume_brace_block_expr()` returns empty Record for complex lambdas with let bindings. No diagnostic emitted. |
| Interface type definitions dropped | High | `parse_interface_def()` parses `type` items inside interfaces but discards the result (`let _ = self.parse_type_def()`). |
| Parse error truncation in recovery mode | Medium | `parse_body_lossy()` truncates accumulated errors and returns empty body with `lossy: true`. Original errors lost. |
| PipeMethod registry `.expect()` | Low | `registry_entry()` panics if a PipeMethod variant lacks a registry entry. Compile-time invariant, runtime panic. |

---

## Stage 2: Module Resolve

```
SourceFile + filesystem  ──>  ModuleGraph (all transitive imports)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-resolve` |
| **Entry** | `discover_module_graph_for_context(context) -> Result<ModuleGraph, ResolveError>` |
| **Input** | DSL root directory + optional target file |
| **Output** | `ModuleGraph { modules: Vec<ResolvedModule> }` |

Walks the filesystem from the DSL root. Parses each `.dag` file to extract its
`module` path and `import` declarations. Resolves all transitive imports.
Builds a dependency-ordered list. No type checking yet.

### Data shape

```
ModuleGraph
  +-- modules: Vec<ResolvedModule>
        +-- path: PathBuf              "/home/.../dsl/tools/makegen.dag"
        +-- module_path: ModulePath    ["tools", "makegen"]
        +-- imports: Vec<ModulePath>   [["std", "patterns"], ["extdeps", "make"]]
        +-- dependencies: Vec<usize>   [3, 7, 12]  (indices into modules vec)
        +-- ast: SourceFile
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| Unresolved imports silently dropped | High | If an import target doesn't exist on disk, the dependency is quietly omitted. `ResolveError::UnresolvedImport` is defined but never returned. |
| Out-of-bounds dependency indices silently skipped | Medium | `topo_sort()` skips indices >= module count. Could mask bugs in dependency building. |
| "unknown" module path fallback | Medium | Files outside configured roots get module path `["unknown"]`, making multiple such files indistinguishable. |
| Hyphen-to-underscore normalization | Low | `my-tool.dag` becomes module `my_tool` implicitly. Undocumented. |

---

## Stage 3: Typecheck

```
ModuleGraph  ──>  TypedProject (validated signatures)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-typecheck` |
| **Entry** | `typecheck_module_graph_with_options(graph, options) -> Result<TypedProject, Vec<TypeError>>` |
| **Input** | `ModuleGraph` (all modules with raw ASTs) |
| **Output** | `TypedProject { modules: Vec<TypedModule> }` |

Validates type references, field access, refinement constraints, interface
conformance, generics, function signatures, service operations, and pipeline
stage dependencies. The AST is preserved — typecheck annotates it with
signatures but does not transform the tree.

### Data shape

```
TypedProject
  +-- modules: Vec<TypedModule>
        +-- path: PathBuf
        +-- module_path: ModulePath
        +-- imports: Vec<ModulePath>
        +-- ast: SourceFile              (same AST, now validated)
        +-- signatures: Vec<TypedItemSignature>
              +-- Fn(TypedCallableSignature)
              +-- Func(TypedCallableSignature)
              +-- Service { name, operations }
              +-- Interface { name, capabilities }
              +-- Pipeline { name, stages, stage_names }
              +-- ...
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| Debug `eprintln!` in production | Medium | ~5 `eprintln!("[DEBUG callable_body] ...")` statements fire on type mismatches, polluting stderr. |
| Unresolved references allowed by flag | High | When `allow_unresolved_references=true`, missing call targets are silently skipped. Downstream lowering fails with cryptic messages. |
| Generic type constraints not fully validated | Medium | Generic instantiation arity is checked, but constraint satisfaction and higher-order function types are not. |
| Pipe method argument types unchecked | Medium | Pipe method call arguments are not validated against method signatures. |

---

## Stage 4: Lower

```
TypedProject  ──>  Dag<LoweredOp> (graph IR)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-lower` |
| **Entry** | `lower_with_config(typed, config) -> Result<Dag<LoweredOp>, LowerError>` |
| **Input** | `TypedProject` |
| **Output** | `Dag<LoweredOp>` — nodes, edges, ports |

This is the big transform. Typed AST items become a **graph of nodes connected
by typed edges**. The major expansions:

- **Service calls** become 3-node transport triplets:
  `prepare_transport` / `execute_transport` / `parse_transport`
- **`content_upsert` pattern** becomes a 5-node chain:
  `prepare_read` / `execute_read` / `compare` / `prepare_write` / `execute_write`
- **`fn` bodies** become `LoweredFnBody` (expression tree attached to the node)
- **Collection ops** (map/filter/fold/split/zip) become `CollectionOpKind` nodes
- **Loops** become `LoopUnpack` / body SubDag / `LoopPack`
- **Data declarations** are evaluated to `serde_json::Value` at lower time

### Data shape

```
Dag<LoweredOp>
  +-- nodes: Vec<Node<LoweredOp>>
  |     +-- id: NodeId
  |     +-- body: NodeBody<LoweredOp>
  |     |     +-- Opaque(LoweredOp)          -- leaf node
  |     |     +-- SubDag(Dag<LoweredOp>)     -- nested graph (loops, branches)
  |     +-- inputs: Vec<Port>
  |     +-- outputs: Vec<Port>
  |
  +-- edges: Vec<Edge>
        +-- from_node / from_port
        +-- to_node / to_port
        +-- kind: EdgeKind { Data, Control, Resource }

LoweredOp variants:
  +-- Callable { module, kind, name, fn_body?, service_metadata? }
  +-- Primitive { module, name, kind: PrimitiveOpKind }
  +-- Collection { module, callable, kind: CollectionOpKind }
  +-- Pipeline { module, name, stages }
  +-- Pattern(PatternOp)      -- LoopUnpack, LoopPack, BranchMerge
  +-- ExternCall { symbol }
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| Silent node drops in pattern expansion | High | When a pattern body node expansion fails, the node is silently omitted via `lower_warn()`. Downstream port references fail with obscure errors. |
| Unsupported expression types silently skipped | High | Unknown expression types in pattern bodies are dropped with a warning (only visible with `DAGLANG_LOWER_WARNINGS=1`). |
| Service call arguments silently dropped | High | Unresolved service call arguments are silently skipped — no error unless executor fails later. |
| `param_source` nodes lack incoming edges | Medium | Created for service call args referencing callable params, but have no data wiring from caller scope. |
| Default arg JSON conversion failures silent | Medium | `expr_to_json_literal()` returning `None` silently skips default injection. |
| Type information erased | Medium | Lowering converts typed AST to untyped `LoweredExpr`. Runtime type mismatches are possible and hard to trace. |
| Transport node deduplication conflicts | Medium | One prepare/execute/parse triplet per service operation per module. Multiple callables calling the same service within one module conflict. |

---

## Stage 5a: Resolve (interpreted path)

```
Dag<LoweredOp>  ──>  Dag<DynOp> (executable operations)
```

| | |
|---|---|
| **Crate** | `core/resolve` |
| **Entry** | `resolve_lowered_dag_with(dag, resolver) -> Result<Dag<DynOp>, ResolveError>` |
| **Input** | `Dag<LoweredOp>` + `&dyn ExternResolver` |
| **Output** | `Dag<DynOp>` where `DynOp = Arc<dyn Executable>` |

Walks every node and maps symbolic `LoweredOp` to concrete executable
operations. This is where the graph becomes runnable.

### Resolution mapping

```
LoweredOp variant              Resolved DynOp
-----------------------------  -----------------------------------------
Callable + fn_body             FnBodyCallableOp (evaluates fn body)
Callable, no fn_body           DeclaredOutputCallableOp (passthrough)
Primitive(GetField)            GetFieldOp
Primitive(MakeLiteral)         MakeLiteralOp
Primitive(Conditional)         ConditionalOp
Primitive(MatchDispatch)       MatchDispatchOp
Primitive(...)                 ... (27 total PrimitiveOpKind variants)
Collection(Map/Filter/Fold)    MapOp / FilterOp / FoldOp / SplitOp / ZipOp
Pattern(LoopUnpack/Pack/...)   PatternOp (kept as-is)
ExternCall { symbol }          ExternResolver::resolve(module, name)
Service transport nodes        GenericRestPrepareOp, GenericShellParseOp, etc.
```

The `ExternResolver` trait is the app-specific extension point:
```rust
trait ExternResolver: Send + Sync {
    fn resolve(&self, module: &str, name: &str) -> Option<DynOp>;
}
```

`gunbc-dag` provides `GunbcExternResolver` with ~10 extern implementations
(render_tree, build_snapshot_content, discover_tools, render_bootstrap_*, etc.).

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| DryRunStrictness not implemented | High | Declared in code but behavior comment says "Phase 1: strictness stored but behavior unchanged (always lenient)". Can't detect modeling gaps. |
| FnBodyCallableOp swallows errors | Medium | Evaluation failures return `Value::Skipped` for all outputs instead of propagating the error. Silent degradation. |
| Missing auth scheme defaults to "none" | Medium | `unwrap_or_else(|| "none".to_string())` when auth_scheme is absent. Could mask configuration errors. |
| Callable passthrough for unwired outputs | Low | Func/Pattern callables without fn_body have optional outputs to avoid hard errors from unwired `__out:*` ports. |

---

## Stage 5b: Emit (compiled path)

```
Dag<LoweredOp> + DerivedArtifacts  ──>  EmissionBundle (source files)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-emit` |
| **Entry** | `emit_rust_bundle(dag, derived) -> EmissionBundle` |
| **Input** | `Dag<LoweredOp>` + metadata (obligations, manifest) |
| **Output** | Source files for target language |

### Backend status

| Backend | Service transport | Callable orchestration | CLI generation | Status |
|---------|------------------|----------------------|----------------|--------|
| **Rust** | Prepare + parse | Phase 1 scaffold | Full | Primary |
| **Go** | Prepare + parse | Generic stubs | N/A | Service transport only |
| **C** | Prepare + parse | `static void` stubs | N/A | Service transport only |
| **MIPS** | Prepare + parse | Label stubs | N/A | Service transport only |

Service `execute` (HTTP send, shell exec) is **intentionally delegated to
runtime** in all backends — not a gap.

### Per-module output structure

```
Emitted for each .dag module:
  types/       Record and sum type definitions
  fn/          Pure function bodies
  transport/   Service transport triplets (prepare/execute/parse)
  func/        DAG orchestrator (topo-scheduled)
  cli/         CLI arg parser from func inputs
  test/        Test harness (4-bucket obligations)
  mock/        MockSpec from service declarations
  manifest/    ProgressManifest (static topology)
  makefile/    Makefile target rule
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| `skip_verification` defaults true | High | Full DAG verification is bypassed because the lowerer produces benign unwired ports (`param_source` nodes). |
| REST/Lambda exposure `generate()` unimplemented | Medium | `rest_gen.rs` and `lambda_gen.rs` have typed params but `generate()` returns `todo!()`. |
| PureRender callables have no exec-runtime handler | Medium | Exec-runtime can't classify callables with fn bodies (RF-E5). |
| Layer 2 Rust native is scaffold only | Medium | `emit_rust_bundle()` produces main.rs + manifest but no real dataflow orchestration. |

---

## Stage 6: Execute

```
Dag<DynOp>  ──>  HashMap<String, Value>
```

| | |
|---|---|
| **Crate** | `core/exec` |
| **Entry** | `execute_with_mode(dag, mode) -> Result<HashMap<String, Value>, ExecError>` |
| **Input** | `Dag<DynOp>` (resolved, executable) |
| **Output** | Final outputs (nodes with no outgoing edges) |

### Execution model

```
1. Topological sort (respect data + control edges)
2. For each node in topo order:
   a. Gather inputs from upstream node outputs (via edge wiring)
   b. Check intercepts:
      - DryRun? Mock transport nodes
      - BoundaryMock? Inject pre-set values
      - Guard failed? Skip node
   c. Call node.body.execute(inputs) -> HashMap<String, Value>
   d. Store outputs in frame for downstream consumption
3. Handle special patterns:
   - LoopUnpack: fan out elements, execute body per element, LoopPack collects
   - BranchMerge: reconcile diverging paths
   - SubDag: recursive execution of nested graphs
4. Return final outputs
```

### Execution modes

| Mode | Behavior |
|------|----------|
| `Real` | All nodes execute, actual I/O performed |
| `DryRun(Lenient)` | Transport nodes intercepted with mocks, pure nodes execute normally |
| `DryRun(Strict)` | **Not implemented** — declared but behaves as Lenient |

### Data in flight: `Value`

```
Value enum (core/ir):
  +-- Str(String)
  +-- Int(i64)
  +-- Float(f64)
  +-- Bool(bool)
  +-- List(Vec<Value>)
  +-- Map(HashMap<String, Value>)
  +-- Bytes(Vec<u8>)
  +-- Secret(String)
  +-- Json(serde_json::Value)
  +-- Unit
  +-- Skipped
```

### Known gaps

| Issue | Severity | Detail |
|-------|----------|--------|
| Auto-filled optional list inputs | Low | Unwired optional list ports silently become `Value::List(vec![])`. Hides wiring bugs. |
| Loop body transport auto-mocking | Low | In DryRun, loop body transport nodes are always auto-mocked, even without explicit mocks. Undocumented. |
| Guard skip not in structured log | Low | Skipped nodes logged to observer but not in ExecutionLog details. |
| `Value::Skipped` propagation | Low | Skipped values in loops cause empty iterations. Correct but hard to debug. |

---

## Driver Coordination

```
.dag file  ──>  CompileOutput (everything)
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-driver` |
| **Entry** | `compile_from_context_with_options(context, options) -> Result<CompileOutput, CompileError>` |

The driver orchestrates the full pipeline:

```
discover_module_graph_for_context()        Module resolution
         |
typecheck_module_graph_with_options()      Type validation
         |
lower_with_config()                        DAG construction
         |
validate_structural_primitive_wiring()     [unconditional]
         |
verify_dag()                               [conditional: skip_verification]
         |
extract_output_paths()                     From content_upsert + @outputs
         |
derive_artifacts()                         ProgressManifest + TestObligations
         |
emit_with_options(target, layer)           Code generation
         |
CompileOutput {
  lowered_dag,              Dag<LoweredOp>
  derived,                  DerivedArtifacts (manifest, obligations)
  emitted,                  EmissionBundle (source files)
  output_paths,             Vec<String> (generated file paths)
  inferred_entrypoints,     Vec<InferredEntrypoint>
  dsl_type_registry,        TypeRegistry (sum/product types)
  data_values,              HashMap<String, serde_json::Value>
  available_profiles,       Vec<String>
  receipt,                  Option<CompileReceipt> (SHA-256)
}
```

---

## Binary Integration (`gunbc-dag`)

The final step for the interpreted path — wiring the compiled DAG into a
runnable binary:

```rust
// One-shot: parse, typecheck, lower, resolve
let dag: Dag<DynOp> = build_dsl_graph("tools/makegen.dag")?;

// Execute with mode selection + freshness + display
run_tool(dag, ExecutionMode::Real, options);
```

### Key components

| File | Purpose |
|------|---------|
| `dsl_builder.rs` | `build_dsl_graph()` — full pipeline in one call |
| `resolve.rs` | `GunbcExternResolver` — app-specific extern resolution |
| `extern_ops.rs` | ~10 extern symbol implementations |
| `tool_runner.rs` | `run_tool()` — freshness + execute + display |
| `tool_graphs.rs` | Per-tool graph builders (makegen, codegen, pragma, etc.) |

---

## End-to-End Worked Example: `makegen`

```
dsl/tools/makegen.dag
  |
  | [Parse]
  v
SourceFile {
  module: tools.makegen,
  imports: [std.patterns, extdeps.make, config.build_targets],
  items: [
    fn render_target(name, deps, recipe) -> String { ... },
    fn render_core_targets(test_cmd, ...) -> String { ... },
    func render_makefile_content(workflows) -> { content: String } uses fs: Filesystem { ... },
  ]
}
  |
  | [Module Resolve]
  v
ModuleGraph { modules: [
  { tools.makegen, imports: [std.patterns, extdeps.make, config.build_targets] },
  { std.patterns, imports: [...] },
  { extdeps.make, imports: [] },
  { config.build_targets, imports: [] },
]}
  |
  | [Typecheck]
  v
TypedProject { modules: [
  { tools.makegen, signatures: [
      Fn("render_target", params: [name: String, deps: List<String>, recipe: String]),
      Fn("render_core_targets", params: [test_cmd: String, ...]),
      Func("render_makefile_content", inputs: [workflows: List<ToolTarget>], outputs: [content: String]),
  ]},
  ...
]}
  |
  | [Lower]
  v
Dag<LoweredOp> {
  nodes: [
    Node("prepare_read_makefile",     Primitive { GetField }),
    Node("execute_read_makefile",     Callable { file_read }),
    Node("compare_makefile_content",  Primitive { Eq }),
    Node("render_makefile_content",   Callable { fn_body: Some(LoweredFnBody { ... }) }),
    Node("execute_makefile_transport",Callable { file_write }),
  ],
  edges: [
    prepare_read:path -> execute_read:path,
    execute_read:content -> compare:existing,
    render_content:content -> compare:expected,
    compare:changed -> execute_write:guard,
    render_content:content -> execute_write:content,
  ]
}
  |
  | [Resolve]                              [Emit]
  v                                        v
Dag<DynOp> {                           EmissionBundle {
  GetFieldOp,                            types/make_target.rs,
  FileReadOp,                            fn/render_target.rs,
  EqOp,                                  transport/file_write.rs,
  FnBodyCallableOp { body },             cli/makegen.rs,
  GenericFileWriteOp,                    test/makegen_test.rs,
}                                      }
  |
  | [Execute]
  v
{ "content": Value::Str("# Generated Makefile\n...") }
  |
  | [Write to disk if changed]
  v
Makefile
```

---

## Consolidated Gap Analysis

### Critical (affects correctness)

| # | Stage | Issue | Impact |
|---|-------|-------|--------|
| C1 | Module Resolve | Unresolved imports silently dropped | Missing dependencies cause cryptic downstream errors |
| C2 | Lower | Silent node drops in pattern expansion | Downstream port references fail with obscure errors |
| C3 | Lower | Service call args silently dropped | Unresolved arguments cause runtime failures, not compile errors |
| C4 | Resolve | DryRunStrictness not implemented | Can't detect modeling gaps in dry-run testing |
| C5 | Emit | `skip_verification` defaults true | Benign unwired ports pass; can't enable strict mode yet |

### Significant (affects usability / debuggability)

| # | Stage | Issue | Impact |
|---|-------|-------|--------|
| S1 | Parse | Multi-stmt brace blocks return empty Record | Complex lambdas silently lose content |
| S2 | Parse | Interface type definitions discarded | Types in interfaces are parsed then dropped |
| S3 | Typecheck | `eprintln!` debug output in production | Pollutes stderr on type mismatches |
| S4 | Typecheck | Unresolved references allowed by flag | Errors deferred to lowering with worse messages |
| S5 | Lower | Type information erased during lowering | Runtime type mismatches hard to trace to source |
| S6 | Resolve | FnBodyCallableOp swallows errors as Skipped | Silent degradation instead of failure |
| S7 | Emit | REST/Lambda exposure generate() is todo!() | Exposure types exist but codegen unimplemented |
| S8 | Emit | PureRender callables lack exec-runtime handler | Can't emit fn bodies in exec-runtime (RF-E5) |

### Low priority

| # | Stage | Issue |
|---|-------|-------|
| L1 | Parse | PipeMethod registry `.expect()` — runtime panic if invariant violated |
| L2 | Module Resolve | "unknown" fallback module path — ambiguous for out-of-root files |
| L3 | Lower | Default arg JSON conversion failures silent |
| L4 | Resolve | Missing auth scheme defaults to "none" |
| L5 | Execute | Unwired optional list ports auto-filled with empty list |
| L6 | Execute | Loop body transport auto-mocked (undocumented) |

---

## Crate Ownership Map

```
core/daglang/
  daglang-syntax/       Stage 1: Parse          (.dag text -> SourceFile)
  daglang-resolve/      Stage 2: Module Resolve  (filesystem -> ModuleGraph)
  daglang-typecheck/    Stage 3: Typecheck       (ModuleGraph -> TypedProject)
  daglang-lower/        Stage 4: Lower           (TypedProject -> Dag<LoweredOp>)
  daglang-emit/         Stage 5b: Emit           (Dag<LoweredOp> -> source files)
  daglang-driver/       Orchestration            (full pipeline coordination)
  daglang-derive/       Artifact derivation      (obligations, manifest, properties)

core/resolve/           Stage 5a: Resolve        (Dag<LoweredOp> -> Dag<DynOp>)
core/exec/              Stage 6: Execute         (Dag<DynOp> -> results)
core/ir/                Shared IR types          (Node, Edge, Port, Value, Dag<T>)
core/codegen/           CLI/binary generation    (entrypoints, tool discovery)
core/infra/             Leaf utilities           (hashing, freshness, manifest)

gunbc-dag/              Binary integration       (extern resolver, tool runner)
```
