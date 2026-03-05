# Compilation Pipeline: `.dag` Source to Execution

> Design doc mapping the full compiler pipeline, data shapes at each stage,
> known gaps, and the two runtime paths (interpreted vs compiled).
>
> **Date**: 2026-03-05

---

## Why This Matters

The entire purpose of gunbc is **moving contradiction discovery from runtime to
static analysis**. The compiler pipeline is not a build tool — it IS the product.
If a `.dag` definition validates, its wiring is correct, its types are sound, and
its execution intent is unambiguous. Contradictions that would surface as opaque
runtime failures in a hand-wired system are caught at compile time as typed
diagnostics pointing to the source location.

Every piece of "forgiveness" in the pipeline — every silent drop, every auto-fill,
every swallowed error — is a direct failure of this mission. It moves a
contradiction back to runtime, which is exactly what the compiler exists to prevent.

---

## Four Invariants

These are the hard rules for the compiler pipeline. Every design decision,
refactor, and new feature must satisfy all four.

### 1. Binary Logic — Every signal is PASS or FAIL

No ternary states. No "maybe." No "deferred." Every stage either succeeds
with correct output or fails with a diagnostic. There are exactly two
outcomes, never three.

**Violations to eliminate** (audit 2026-03-05, 9 found):

| Signal | Location | The forbidden third state |
|--------|----------|--------------------------|
| `lossy: true` | parser `FnBody` | Not-parsed-but-not-error |
| `allow_unresolved_references` | typecheck options | Not-checked-but-not-rejected |
| `lower_warn()` + `continue` | lowerer pattern expansion | Not-lowered-but-not-error |
| `skip_verification` | driver | Not-verified-but-not-rejected |
| `Value::Skipped` | executor | Not-a-value-but-not-an-error |
| `has_edge_to_port()` → skip | lowerer wiring | Not-wired-but-not-failed |
| `param_source` without edges | lowerer | Not-connected-but-not-error |
| `synthesize_expr_compute()` → None | lowerer RT4c | Not-wired-but-not-failed |
| `propagate_skipped()` conflation | executor | Guard-skip conflated with absent |

### 2. Minimalism — No redundancy between stages

Each stage adds new information. No stage restates what a previous stage
already computed. If the same data appears in two stages, one is redundant.

**Redundancies to eliminate** (audit 2026-03-05, 11 found):

| What | Canonical owner | Restated in | Fix |
|------|-----------------|-------------|-----|
| `path`, `module_path`, `ast` | `ResolvedModule` | `TypedModule` (copied) | TypedModule references by index |
| Callable signatures | `TypedCallableSignature` | Node ports (re-derived from AST) | Derive ports from signatures |
| Type constraints | Typecheck (rich) | Lower (erased to strings) | Carry `TypeId` on ports |
| `output_paths` | Lowered DAG | Driver (re-extracted) | Compute once during lower |
| `pipeline_params` | TypedProject | Driver (re-scanned) | Attach during typecheck |
| `dsl_type_registry` | TypedProject | Driver (re-extracted) | Compute once during typecheck |
| `available_profiles` | TypedProject | Driver (re-scanned) | Attach during typecheck |
| `data_values` | TypedProject | Driver (re-evaluated) | Evaluate once during lower |
| Transport triplet structure | Lowerer (`Node.kind`) | Derive (re-parsed from port strings) | Use `Node.kind` directly |
| Loop pattern structure | Lowerer (created SubDag) | Executor (`detect_loop_pattern()`) | Stamp `LoopInfo` on SubDag |
| `inferred_entrypoints` | Lowered DAG topology | Driver (re-inferred) | Compute once during lower |

### 3. Resolve Early — Don't defer decisions

If information is available at Stage N, resolve it at Stage N. Every deferral
creates an ambiguous intermediate state that later stages must handle with
fallbacks — which violates Invariant 1 (binary logic).

**Deferrals to eliminate** (audit 2026-03-05, 9 remaining):

| Decision | Deferred from | To | Fix |
|----------|--------------|-----|-----|
| Selective import validation | Parse | Typecheck | Validate names against module symbols |
| Auth scheme validation | Typecheck | Lower/Resolve | Validate enum in typecheck |
| Missing required input detection | Lower | Executor | Validate after lower: every required port has an edge |
| Transport resource ports (`res:file`) | Lower | Resolve | Emit resource ports during lower |
| Default list ports (`[]`) | Lower | Executor (auto-fill) | Wire `MakeLiteral([])` during lower |
| Default callable params | Lower (post-pass) | Lower (post-pass) | Wire inline during call lowering |
| Profile binding wiring | Lower | Resolve | Require `--profile` at lower time |
| `param_source` edge wiring | Lower | Lower (post-pass) | Wire during initial lowering |
| ~~Variant constructor resolution~~ | ~~Lower~~ | ~~Executor~~ | **DONE** (VariantConstruct) |

### 4. Clear Contracts — Every stage is a pure function with typed I/O

Each pipeline stage is a **pure function**: explicit inputs, explicit outputs, no
hidden state, no ambient globals, no mutation of shared structures. The function
signature IS the contract. If you can read the signature, you know exactly what
the stage needs and what it produces.

This serves as a strong **boundary between participants** — whether those are
humans, agents, or teams. Everyone knows what they need to produce, provide,
and accept at all times. No implicit coupling, no "you need to also call X
first" that isn't visible in the types.

**Contract violations to eliminate** (audit 2026-03-05):

| Violation | Location | Problem |
|-----------|----------|---------|
| Typecheck takes ownership of `ModuleGraph` | `typecheck_module_graph(graph: ModuleGraph)` | Moves input, later stages can't reference it |
| TypedModule copies ModuleGraph fields | `TypedModule { path, module_path, imports, ast }` | Duplicates input instead of referencing |
| CompileOutput is a grab-bag | 11 fields, many re-extracted | Output conflates multiple concerns |
| Execute has 4 progressively wider entry points | `execute` → `..._with_mode` → `..._and_inputs` → `..._and_detail` | Growing parameter lists hide the real contract |
| ExternResolver is stringly-typed | `resolve(module: &str, name: &str) -> Option<DynOp>` | Runtime string lookup instead of validated ID |
| LowerError is ad-hoc | `{ node_id: Option<String>, reason: String }` | No structured error, no span, Option for required context |
| verify_dag has no output type | Validation only, doesn't produce `VerifiedDag` | No type-level proof that verification happened |
| Driver re-extracts stage outputs | `extract_output_paths()`, `infer_entrypoints()` after lower | Re-walks structures that earlier stages already computed |

---

## Architectural Principles

These principles follow from the four invariants above.

### The Tautology Rule

A correct compiler pipeline is a chain of **lossless semantic translations**.
Each stage restates the exact same truth, translated into a vocabulary suited for
the next stage. You never lose information, you never guess, and you never
implicitly invent meaning.

In a perfectly tautological compiler:
- The **parser** is a strict historian — it records exactly what the user wrote.
- The **typechecker** is an absolute gatekeeper — if it passes, soundness is proven.
- The **lowerer** makes implicit intent explicit — every data flow is a wire.
- The **executor** is maximally dumb — it blindly traverses a verified graph.

When bugs appear, it is because a stage broke the tautology by:
(1) lying by omission (dropping data), (2) lying by invention (auto-filling),
or (3) swallowing errors (pretending invalid state is valid).

The strategy is to **delete all the "forgiveness"**: search for `lossy`, `skip`,
`allow_unresolved`, `lower_warn`, and `unwrap_or_else` in Stages 1-4.

### Per-Stage Tautologies

| Stage | Tautology | Guarantee |
|-------|-----------|-----------|
| Parse | `AST == Source Text` | 1:1 structural record. No drops, no substitutions. |
| Module Resolve | `ModuleGraph == Physical Reality` | Every import resolves to a real file, or it's an error. |
| Typecheck | `TypedProject == Sound Program` | If it passes, all references resolve and all types match. |
| Lower | `DAG == Explicit Execution Intent` | Every data flow is an edge. No magic materialization. |
| Resolve/Execute | `Execution == Blind Traversal` | Executor only knows what the graph tells it. |

### Push Magic to the Left

Implicit runtime behavior should be reified into explicit compiler constructs.
Every piece of "forgiveness" in the runtime is a contradiction that escaped
static analysis — exactly the thing the project exists to prevent.

---

## Implementation Patterns

Concrete Rust patterns that enforce the three invariants across the pipeline.

### Verdict<T> — one result type for all stages

Every stage API on the main compilation path returns `Verdict<T>`:

```rust
type Verdict<T> = Result<T, Diagnostics>;

struct Diagnostics {
    errors: Vec<Diagnostic>,   // non-empty => FAIL
    warnings: Vec<Diagnostic>, // never affects PASS/FAIL
}
```

- **PASS**: `Ok(T)` — downstream can assume all invariants hold.
- **FAIL**: `Err(Diagnostics)` — downstream MUST NOT run. No "continue with partial."

Lossy/IDE helpers are separate APIs (`parse_for_ide`, `typecheck_partial`), never
reachable from `compile_strict`.

### The Strict Pipeline

The primary compilation path is a single function where every `?` is a PASS/FAIL
boundary:

```rust
fn compile_strict(context: Context, opts: Options) -> Verdict<CompileOutput> {
    let graph    = discover_module_graph(context)?;           // Parse + Resolve
    let externs  = build_extern_registry(&opts.externs);      // ExternId table
    let typed    = typecheck_strict(&graph, &externs, &opts.tc)?; // Sound + externs resolved
    let dag      = lower_strict(&typed, &opts.lower)?;        // Explicit DAG or FAIL
    let verified = verify_dag(&dag, &typed.type_registry)?;   // Wiring correct or FAIL
    let derived  = derive_artifacts(&verified)?;               // Obligations, manifest
    let emitted  = emit_with_options(&verified, &derived, opts.emit)?;

    Ok(CompileOutput { verified, derived, emitted, ... })
}
```

And the interpreted runtime path:

```rust
let realized = realize_for_backend(&verified)?;              // expand ServiceCall → triplets (total)
let resolved = verified.dag.map_bodies(|op| resolver.resolve(op)); // topology-preserving
let result   = execute_with_mode(&resolved, mode)?;          // no auto-fills
```

### NodeOutcome — execution result per node

Replace `Value::Skipped` with a per-node execution outcome:

```rust
enum NodeOutcome {
    Executed(HashMap<String, Value>),  // ran, produced outputs
    Skipped,                            // guard evaluated to false
    Failed(ExecError),                  // real error, propagate
}
```

The executor keeps `HashMap<NodeId, NodeOutcome>` instead of mapping ports to
`Value::Skipped`. When a node is `Skipped`, its outputs don't exist — downstream
nodes are **transitively bypassed** (never scheduled, not "given Skipped values"):

```rust
if node.control_edge_evaluates_false() {
    outcomes.insert(node.id, NodeOutcome::Skipped);
    for dependent in node.downstream_dependents() {
        outcomes.insert(dependent, NodeOutcome::Skipped);
    }
    continue;
}
```

Downstream nodes that need actual values despite an upstream skip must have
explicit default values wired by the lowerer. The executor never invents data.

### NodeOrigin — traceability from IR to source

Every node in the lowered DAG carries its origin:

```rust
enum NodeOrigin {
    UserCode { span: Span },
    PatternExpansion { instance_id: PatternInstanceId, local_idx: u8 },
}
```

Pattern instances are tracked in the DAG metadata:

```rust
struct PatternInstance {
    kind: PatternKind,       // ServiceCall, ContentUpsert, Loop, Branch
    source_span: Span,
    node_ids: Vec<NodeId>,
}
```

### side_effecting annotation for DryRun

The lowerer stamps each node with `side_effecting: bool`. In `DryRun(Strict)`,
encountering a `side_effecting` node without an explicit mock is
`ExecError::StrictDryRunBlocked { node_id }` — not a silent skip.

### One owner per kind of truth

| Truth | Owner | Referenced by |
|-------|-------|---------------|
| Module facts (paths, imports, files) | `ModuleGraph` | `TypedProject.graph` (by ref) |
| Typing facts (signatures, constraints) | `TypedProject` | `Dag<LoweredOp>` port types |
| Extern symbol signatures | `ExternRegistry` | Typecheck (validation), Lower (`ExternId`) |
| Execution topology | `Dag<LoweredOp>` | `VerifiedDag<LoweredOp>` wrapper |
| Verification proof | `VerifiedDag<LoweredOp>` | Resolve + Emit (required input) |
| Backend expansion | `RealizedDag<LoweredOp>` | Executor / emitter (provably total) |
| Runtime bindings | `Dag<DynOp>` / `EmissionBundle` | Executor / compiled binary |

### CompileDb arena (target architecture)

The minimalism invariant implies a shared database rather than per-stage output
types that copy earlier fields. The target architecture:

```rust
struct CompileDb {
    files: FileTable,           // file contents + spans
    parsed: ParsedTable,        // SourceFile per FileId
    modules: ModuleTable,       // ModuleId, ModulePath, FileId, imports
    symbols: SymbolTable,       // DefId, name -> DefId
    types: TypeTable,           // TypeId arena, constraints
    externs: ExternRegistry,    // ExternId -> ExternSymbol (signature + module)
    lowered: Option<Dag<LoweredOp>>,
    verified: Option<VerifiedDag<LoweredOp>>,
}
```

Each stage mutates the shared database:

```rust
fn parse_all(db: &mut CompileDb) -> Verdict<()> { ... }
fn resolve_modules(db: &mut CompileDb) -> Verdict<()> { ... }
fn typecheck(db: &mut CompileDb) -> Verdict<()> { ... }
fn lower(db: &mut CompileDb) -> Verdict<()> { ... }
fn verify(db: &mut CompileDb) -> Verdict<()> { ... }
```

No `TypedProject` duplicating `path/module_path/imports/ast`. Those are already
in `db.modules` + `db.parsed`. Typed info references IDs. This eliminates the
"which copy is authoritative?" class of bugs entirely.

> **Note**: This is the long-term target. The incremental path starts with the
> ECS overlay pattern (below), which achieves the same property locally.

### ECS overlay pattern (incremental step)

`TypedProject` doesn't copy the AST — it holds a reference to `ModuleGraph` and
overlays proofs onto AST node IDs:

```rust
struct TypedProject<'a> {
    graph: &'a ModuleGraph,                          // canonical module facts
    signatures: HashMap<SymbolId, TypedItemSignature>, // proven types (NEW info)
}
```

No field in `TypedProject` duplicates a field in `ModuleGraph`. The AST lives
in exactly one place in memory.

### ExternRegistry — push extern validation left

Today extern symbol resolution happens at Stage 5a (Resolve), meaning unknown
externs aren't detected until you try to run the graph. The fix: validate extern
calls in typecheck, carry typed IDs through lowering.

```rust
struct ExternRegistry {
    symbols: Vec<ExternSymbol>,
}

struct ExternSymbol {
    module: String,
    name: String,
    signature: TypedCallableSignature,
    id: ExternId,
}
```

Typecheck consumes `ExternRegistry` and proves every `extern_call` resolves to
an `ExternId` with a matching signature. The lowerer carries `ExternId`, not
`(module, name)` strings. Stage 5a resolve becomes total — it cannot fail on
missing externs because typecheck already proved they exist.

---

### Preserve High-Level Intent Through Lowering (Decision vs Realization)

The lowerer currently expands service calls into 3-node transport triplets and
content_upsert into 5-node chains. This destroys semantic intent and creates
deduplication conflicts when multiple callables reference the same service.

There's an apparent tension between "resolve early" and "preserve intent," but
the two are compatible if you split **decision** from **mechanical realization**:

- **Decision** (happens early): typecheck proves the service op exists, args
  match, auth is present, transport kind is known. All choices are made.
- **Realization** (happens in backend): expand the fully-resolved ServiceCall
  node into transport triplets. This is a provably total mechanical expansion
  that cannot fail because all decisions were already made.

The canonical DAG keeps `ServiceCall` as a single node with all decisions
resolved:

```rust
LoweredOp::ServiceCall {
    op: ServiceOpId,              // resolved in typecheck
    args: Vec<ArgBinding>,        // fully bound (no silent drops)
    result_ports: Vec<PortSpec>,  // typed outputs
    transport_kind: TransportKind,
    auth: AuthSpec,               // proven present / validated
    origin: Origin,
}
```

Backend expansion produces `RealizedDag<LoweredOp>` — a separate type from
`VerifiedDag<LoweredOp>`:

```rust
VerifiedDag<LoweredOp>  // canonical, high-level intent preserved
    ↓ backend expand (provably total)
RealizedDag<LoweredOp>  // transport triplets materialized
```

Canonical verification happens once on `VerifiedDag`. Backend expansion is a
total function (cannot fail) because all choices were already made.

### Carry Types Forward

The typechecker proves types, but the lowerer erases them into untyped
`LoweredExpr`. Ports should carry type information so that:

- Edge validation can check `from_port.type == to_port.type` at wire time
- The emitter can generate typed code without reconstructing type info
- Runtime type mismatches trace directly to the source location

Types proven by Stage 3 should be carried as far right as possible. If the
parser found an AST node, keep it. If the typechecker proved a type, attach it
to the DAG port. The truth should accumulate, never erode.

### One Canonical IR

All front-end phases (Parse → Module Resolve → Typecheck → Lower) are a
semantics-preserving normalization into one canonical object:

> **`Dag<LoweredOp>` + extern symbol table + data values**

Both the interpreted and compiled paths are realizations of this same object.
The interpreted path binds symbols to `DynOp` and traverses the graph. The
compiled path emits source code that, when compiled, traverses the same graph.

This means:
- Verification happens once, on the canonical IR, before either path runs.
- Both paths must produce identical outputs for the same inputs (the parity test).
- A `VerifiedDag<T>` type wrapper should gate entry to both Resolve and Emit —
  making verification a structural requirement, not a boolean flag.

### Resolution Preserves Topology

The Resolve stage (`Dag<LoweredOp>` → `Dag<DynOp>`) must be a **pure
relabeling**: it swaps the `body` of each node from symbolic to executable,
but never adds, removes, or rewires nodes or edges. Graph topology is fixed
after lowering. If resolution needs to change the graph, that change belongs
in the lowerer.

This invariant should be enforced structurally via `Dag<T>.map_bodies()`:

```rust
impl<T> Dag<T> {
    fn map_bodies<U>(self, f: impl FnMut(T) -> U) -> Dag<U> {
        // same nodes, same edges, same ports — only body labels change
    }
}
```

This makes it impossible for resolution to accidentally modify topology. The
test `assert_eq!(resolved.nodes.len(), lowered.nodes.len())` becomes redundant
because the type system prevents it.

### Pattern Instance Tracking

When the lowerer expands patterns (service call → 3-node triplet, content_upsert
→ 5-node chain, loop → unpack/body/pack), it should tag the generated nodes
with a backref to the original pattern:

```
PatternInstance { kind: ServiceCall, source_span: Span, inner_node_ids: Vec<NodeId> }
```

This gives:
- Better error messages ("this transport node was generated from service call at line 42")
- Verification can recognize "this 5-node chain IS a content_upsert" structurally
- Emit can reconstruct high-level intent without reverse-engineering the graph

### Skipping as Control Flow, Not Magic Values

`Value::Skipped` is currently a variant in the `Value` enum, meaning every value
consumer must handle "what if this is Skipped?" This is a type-level hack.

The tautological alternative: skipping is a **per-node control edge effect**.
A node with a guard that evaluates to false is not executed, and its outputs
are never materialized. Downstream nodes that depend on those outputs must
either:

- Be part of the same skip group (transitively skipped via control edges), or
- Have explicit default values wired by the lowerer.

This eliminates an entire class of "what does Skipped mean here?" bugs and
makes the skip behavior visible in the graph structure, not hidden in values.

### Stages Add Information, Never Restate It

Each pipeline stage should add new information on top of the previous stage's
output, not copy it:

- `ModuleGraph` owns module paths, file paths, imports, and raw ASTs.
- `TypedProject` adds type signatures and validation — it references modules
  by index into `ModuleGraph`, not by copying their fields.
- `Dag<LoweredOp>` adds execution topology — it doesn't carry raw ASTs.

When stages restate information, it creates synchronization bugs and makes it
unclear which copy is authoritative.

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
validate_structural_primitive_wiring()     [unconditional] ← TARGET: merge into verify()
         |
verify_dag()                               [conditional: skip_verification] ← TARGET: mandatory
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
