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

### The deeper motivation

A compiler's job is to **store tautology** — to convert uncertainty into certainty
as early as possible, so that everything downstream can operate with confidence.
The preference for collapsing signals to binary logic (true or false) isn't just
an engineering style choice — it IS the compiler. A compiler that stores
tautology early means programs built on top of it don't have to guess, hedge, or
check. They can trust the ground they're standing on.

This mirrors how good coordination works in any high-stakes domain. The four
invariants below are communication norms applied to code:

- **Binary signals** — say yes or no. "Maybe" forces the receiver to guess,
  which is where misunderstanding comes from.
- **No repetition** — one source of truth. Repetition creates divergence.
- **Resolve ambiguity early** — the longer it sits, the more assumptions
  accumulate on top of it. Surface contradictions immediately.
- **Explicit contracts** — state your expectations. Don't make people infer
  what you need.

These aren't universal truths. In exploratory contexts (brainstorming, early
prototyping) you'd want softer boundaries and more tolerance for ambiguity. But
a compiler pipeline is past that phase — it's the thing that needs to be
maximally trustworthy so everything built on top of it can afford to be
creative. Think of it as the scientific method applied to program construction:
conservative, incremental, falsifiable progress. Make your claims precise enough
that they can be proven wrong quickly, rather than vaguely enough that failure is
slow and confusing.

### The obligation boundary

Today, when the compiler can't prove a property statically, it generates a
**proof obligation** — a test that testgen emits to verify the property at
runtime. This is the right escape hatch: you can't prove everything statically.
But the obligation boundary should move **leftward** over time. Each improvement
to the compiler should let it prove more, so testgen generates less.

The goal is not to eliminate testgen. It's to make testgen's job **precise**.
Today testgen is defensive — it infers providers from node ID strings, probe-
executes downstream nodes to guess response shapes, and falls back to "kitchen
sink" mock responses when it can't figure out what a service returns. This
defensiveness exists because the compiler pipeline loses information that the
DSL author declared:

- `service github.Gist { ... }` — the compiler knows the provider, but this
  metadata is lost after lowering. Testgen re-infers it from `"github.Gist.Create/execute"`.
- `response { 200 => GistResponse, 404 => NotFound }` — the compiler parses
  this, but the response type map doesn't flow through the IR. Testgen probe-
  executes downstream to find which response shape "fits."
- `readonly`, `idempotent` — the compiler validates these, but testgen re-
  derives them via BFS graph walk (`CallableProperties`) instead of reading
  from verified metadata.

Every piece of metadata that testgen re-derives is a piece of tautology the
compiler failed to store. The fix is to preserve service metadata, response
contracts, and behavioral properties through the lowered IR so testgen can
trust them directly.

### Pain points that motivated this doc

These are real bugs from the last few weeks — each one is a compiler pipeline
failure that manifested as an opaque runtime error:

| Bug | Root cause | What the compiler should have done |
|-----|-----------|-----------------------------------|
| `make gist` DNS error | Lowerer creates transport nodes for unreachable match arms (no guards) | Lower match arms into guarded SubDags; transport nodes inside dead arms never execute |
| `make gist` `audience=true` | Lowerer doesn't inject literal nodes for default parameters | Emit `MakeLiteral("sigstore")` for omitted args with defaults (**fixed**: Bridge 2b) |
| `gunbc-ci` false failure | Lowerer silently drops complex return expressions (`BinOp`, `Match` → None) | Return `LowerError` instead of silently dropping |
| `make gist` 401 | No credential wiring from service config to transport execute node | Validate that every required port has a producer after lowering |
| SDLC won't compile | Unresolved imports silently dropped by module resolver | Return `ResolveError::UnresolvedImport` instead of skipping |
| Testgen kitchen sink mocks | Service provider metadata lost after lowering | Preserve provider + response contract through IR |

Every one of these would have been caught by the four invariants: binary PASS/FAIL
(no silent drops), minimalism (no re-derivation), resolve early (validate at
lower time), clear contracts (metadata flows through typed interfaces).

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

**Open or partial contract gaps** (audit 2026-03-06):

| Gap | Current state | Remaining mismatch |
|-----|---------------|--------------------|
| TypedProject ownership | `typecheck_module_graph()` now borrows `&ModuleGraph` | `TypedModule` still clones `path`, `module_path`, `imports`, and `ast` instead of referencing canonical module storage |
| Lower-stage outputs | Main compile path now consumes `LowerOutput` | Helper and embedded compile flows still rely on some legacy extraction helpers, so lower-stage ownership is not fully canonical yet |
| Diagnostics | Shared `daglang-contract::Diagnostic` exists | `span`/`file` remain optional, and `TypeError`/`LowerError` still flow through `Spanned*` wrappers instead of a single diagnostic path |
| Verification gate | `VerifiedDag<T>` exists and gates compile output | `VerifiedDag::from_verified()` remains an escape hatch, so proof is not yet forced at every construction site |
| Runtime bindings | `RuntimeBindings` centralizes extern registration | Bindings are still keyed by `(module, name)` strings and bridge `ExternResolver`; ExternId-keyed total linking is still target-state work |

---

## Architectural Principles

These principles follow from the four invariants above. The first three
(binary logic, minimalism, resolve early) constrain the semantics. The fourth
(clear contracts) constrains the interfaces.

### When You Can't Store Tautology, Fail Helpfully

A compiler's PASS path stores tautology — certainty for downstream consumers.
But the FAIL path is equally important. When the compiler finds a contradiction,
the error message is its **last act of usefulness** before stopping. A good
error message transfers enough information for the human to resolve the
contradiction themselves.

Every diagnostic must answer three questions:

1. **What went wrong?** — the contradiction, in structured terms (expected X, got Y)
2. **Where?** — the source location (`.dag` file, line, column) that caused it
3. **How to fix it?** — a concrete suggestion, or at minimum, what the stage
   needed that it didn't get

This means each stage is responsible for emitting errors that carry the I/O
necessary for earlier stages to correlate to the actual failure. A lowerer error
that says `"missing required input for compare_content"` is useless. A lowerer
error that says `"tools/gist.dag:42: service call gist.Create is missing
required argument 'description' (expected: String) — add description: "..." to
the call site"` lets the user fix it immediately.

**Current diagnostic quality** (audit 2026-03-05):

| Stage | Spans | Structured context | Help/suggestion | Verdict |
|-------|-------|-------------------|-----------------|---------|
| Parse | Yes | No (message string) | No | Has location, lacks structure |
| Module Resolve | No | Yes (module paths) | No | Has context, no location |
| Typecheck | **No** | Yes (35+ variants) | No | Rich context, completely blind to source |
| Lower | **No** | Yes (25+ variants) | Partial (ad-hoc in Display) | Rich context, no location |
| Verify (SubDag) | No | Yes (8 variants, available ports) | No | Structural detail, no source link |
| Derive | No | No (single String variant) | No | Minimal |
| Emit | No | Yes (3 variants) | No | Adequate |
| Resolve (core) | No | No (node_id + reason string) | No | Worst: unstructured, no location |
| Execute | No | Yes (layered, 7 layer types, classification) | No | Richest runtime context, no source link |

The pattern is clear: **source location is generated at parse time and
immediately lost**. Every subsequent stage operates blind to where in the `.dag`
file the problem originated. The fix is to thread `Span` from parse through
typecheck and lowering, and attach it to every error.

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

Target end state: every lowered node carries source origin. Current branch has
the `NodeOrigin` field with `Unknown` as the backward-compatible default, but
most lowerer stamping is still deferred until spans are threaded through the
lowerer context.

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

### Contracts crate (enforced boundaries)

Create a small `daglang-contracts` crate containing **only** the shared interface
types (`Verdict`, `Diagnostics`, `Diagnostic`, `Span`, `FileId`, `ModuleId`,
`ExternId`, `TypeId`, etc.) and the function signatures for each stage. Every stage
crate depends on it. No implementation code.

This turns "style preference for pure functions" into an enforced boundary. If a
stage needs something not in its declared input, it must add it to the contract —
making the dependency visible to everyone.

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
// Current implementation (core/daglang/daglang-typecheck/src/extern_registry.rs)
struct ExternRegistry {
    entries: HashMap<(String, String), ExternSignature>,
}

struct ExternSignature {
    pub module: String,
    pub name: String,
    pub input_count: usize,
    pub output_count: usize,
}

// Target: ExternId-keyed total linking (not yet implemented)
// struct ExternSymbol { module, name, signature: TypedCallableSignature, id: ExternId }
```

Typecheck consumes `ExternRegistry` and validates extern declarations exist with
matching arities. **Current state**: keyed by `(module, name)` string tuples.
**Target**: `ExternId`-keyed total linking where the lowerer carries `ExternId`
not strings, and Stage 5a resolve becomes provably total.

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

## Target Stage Interfaces

Every pipeline stage is a pure function: `fn(Input) -> Verdict<Output>`. The
signature IS the contract. No hidden state, no ambient globals, no progressively
wider parameter lists.

### Shared Types

> Current-state note (2026-03-06): this section describes the **target end
> state**, not a claim that every detail is already implemented today. The
> branch is still transitional on mandatory located diagnostics, canonical
> `TypedProject` ownership, full `NodeOrigin` stamping, ExternId-keyed runtime
> bindings, and a standalone `SourceBundle` ingest stage.

```rust
/// The one result type. PASS = Ok, FAIL = Err. No third state.
type Verdict<T> = Result<T, Diagnostics>;

struct Diagnostics {
    errors: Vec<Diagnostic>,    // non-empty => FAIL
}

/// Every error answers: what went wrong, where, and how to fix it.
struct Diagnostic {
    code: &'static str,         // machine-readable (e.g. "E0401")
    message: String,            // human-readable: the contradiction
    span: Span,                 // source location — NOT optional for user-facing errors
    file: PathBuf,              // which .dag file
    context: DiagnosticContext, // structured: what was expected vs what was found
    help: Option<String>,       // concrete suggestion for resolving the contradiction
    related: Vec<RelatedSpan>,  // secondary locations (e.g. "first defined here")
}

/// Structured context so tooling can act on errors programmatically.
enum DiagnosticContext {
    /// Type mismatch: expected X, got Y
    TypeMismatch { expected: String, got: String },
    /// Missing required item (import, arg, port, field)
    Missing { kind: &'static str, name: String, available: Vec<String> },
    /// Duplicate definition
    Duplicate { name: String, first: Span },
    /// Unsupported feature
    Unsupported { feature: String },
    /// Generic (escape hatch for rare cases)
    Note(String),
}

struct RelatedSpan {
    span: Span,
    file: PathBuf,
    label: String,              // "first defined here", "imported from here", etc.
}

/// Interned identity types — no stringly-typed resolution after typecheck.
/// Each stage introduces IDs for its domain; later stages carry IDs, not strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId(u32);        // Source ingest: stable within one compilation
struct ModuleId(u32);      // Module resolve: index into ModuleGraph.modules
struct DefId(u32);         // Typecheck: any named definition
struct CallableId(u32);    // Typecheck: fn/func/pattern/extern func
struct ServiceOpId(u32);   // Typecheck: service operation
struct ExternId(u32);      // Typecheck: extern func (via ExternRegistry)
struct TypeId(u32);        // Typecheck: type arena entry
struct DataId(u32);        // Lower: evaluated data declaration
struct PatternInstanceId(u32); // Lower: expanded pattern (triplet, chain, loop)

/// Newtype: proof that verification passed. Only constructible via verify().
/// Current implementation (core/ir/src/verified.rs): simple newtype wrapper.
struct VerifiedDag<T>(Dag<T>);
// Target: add proof: DagProof field recording what was checked and when.

/// Target architecture (not yet implemented):
/// Newtype for backend-expanded DAG. Provably total expansion of VerifiedDag.
// struct RealizedDag<T> { dag: Dag<T> }
```

### Diagnostic Contracts Per Stage

Every stage emits `Diagnostic` on FAIL. The table below specifies the minimum
information each stage's diagnostics must carry. "Help" means a concrete
suggestion for resolving the contradiction.

| Stage | Must carry span | Must carry context | Must carry help | Example |
|-------|----------------|-------------------|-----------------|---------|
| Parse | Yes (from lexer) | `Note` (syntax description) | Yes: expected token | `"expected '}' to close block opened at line 12"` |
| Module Resolve | Yes (import statement span) | `Missing { kind: "module", available }` | Yes: similar module names | `"import std.rendor not found — did you mean std.render?"` |
| Typecheck | Yes (AST node span from parse) | `TypeMismatch`, `Missing`, `Duplicate` | Yes: expected type/name | `"tools/gist.dag:42: expected String, got Int for arg 'id' — change to String or cast"` |
| Lower | Yes (AST span via TypedProject → ModuleGraph) | `Missing { kind: "port" }`, structural | Yes: what to wire | `"tools/gist.dag:42: service call gist.Create missing required arg 'description' (String)"` |
| Verify | Yes (via NodeOrigin on lowered nodes) | `Missing { kind: "edge" }`, structural | Yes: what's unwired | `"node compare_content has unwired input 'expected_content' — add content source or default"` |
| Derive | Yes (via node metadata) | Structural | No (internal) | Rare — structural invariant violations |
| Emit | Yes (via node metadata) | `Unsupported` | Yes: what's not supported | `"REST exposure codegen not yet implemented for func 'serve' at line 8"` |
| Execute | No (runtime) | Layered (`ErrorLayer` stack) | Yes: what mock/config is missing | `"DryRun: unmocked transport node 'execute_gist_create' — add mock or run in Real mode"` |

Current implementation note: treat this as the intended contract, not the fully
landed one. `Diagnostic::new()` is still location-free, `TypeError` and
`LowerError` still convert through `SpannedTypeError` / `SpannedLowerError`,
and most lowered nodes still default `NodeOrigin::Unknown`.

The key insight is still correct: **spans should flow rightward through the
pipeline**. Parse creates them. Module resolve attaches them to import
statements. Typecheck reads them from AST nodes (which live in `ModuleGraph`,
referenced by `TypedProject`). The target end state is for lower to attach them
as `NodeOrigin::UserCode { span }` on every lowered node so verify and emit can
consume them directly. The current branch has not fully completed that
propagation yet.

### Stage 0: Ingest (impurity isolation)

```rust
fn ingest_sources(vfs: &dyn Vfs, roots: &[PathBuf], entry: EntrySpec) -> Verdict<SourceBundle>
```

- **In**: `Vfs` (filesystem trait) + DSL roots + entry spec
- **Out**: `SourceBundle` — immutable snapshot of all source files
- **Contract**: This is the **only impure stage**. Everything after ingest is a pure
  function of the `SourceBundle`. No subsequent stage reads the filesystem.

```rust
trait Vfs: Send + Sync {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn list_files(&self, root: &Path, ext: &str) -> io::Result<Vec<PathBuf>>;
}

struct SourceBundle {
    files: Vec<SourceUnit>,             // stable: index == FileId.0
    by_path: HashMap<PathBuf, FileId>,
    entry: EntrySpec,
}

struct SourceUnit {
    id: FileId,
    path: PathBuf,
    text: Arc<str>,
}
```

Isolating filesystem access to Stage 0 means the rest of the pipeline can be tested
with synthetic inputs — no filesystem mocking needed in parser, typechecker, or
lowerer tests. Currently module resolve interleaves parsing and filesystem reads;
this separation makes it a pure graph construction over already-parsed ASTs.

### Stage 1: Parse

```rust
fn parse(source: &str) -> Verdict<SourceFile>
```

- **In**: raw `.dag` text
- **Out**: `SourceFile` — faithful structural record with spans on every node
- **Contract**: `AST == Source Text`. No drops, no substitutions, no lossy recovery.
  Every AST node carries a `Span`. Unsupported syntax is represented as
  `Expr::ErrorNode { span }`, never silently discarded.

### Stage 2: Module Resolve

```rust
fn discover_module_graph(context: &DriverContext) -> Verdict<ModuleGraph>
```

- **In**: DSL root paths + optional target file
- **Out**: `ModuleGraph` — all modules in dependency order, with parsed ASTs
- **Contract**: every `import` resolves to exactly one module on disk, or FAIL.
  No `["unknown"]` fallbacks, no silently dropped imports, no OOB index skips.

```rust
struct ModuleGraph {
    modules: Vec<ResolvedModule>,   // dependency-ordered (leaves first)
}

struct ResolvedModule {
    file: FileId,                   // index into source file table
    path: PathBuf,                  // filesystem path
    module_path: ModulePath,        // semantic path (e.g. ["tools", "makegen"])
    ast: SourceFile,                // parsed AST (owned here, referenced later)
    dependencies: Vec<ModuleId>,    // indices into modules vec
}
```

### Stage 3: Typecheck

```rust
fn typecheck(graph: &ModuleGraph, externs: &ExternRegistry) -> Verdict<TypedProject>
```

- **In**: `&ModuleGraph` (borrowed, not moved) + `ExternRegistry`
- **Out**: `TypedProject` — type signatures overlaid on module graph, by reference
- **Contract**: if PASS, every reference resolves, every type is sound, every
  extern call matches a registered signature. No `allow_unresolved_references`.

```rust
struct TypedProject<'a> {
    graph: &'a ModuleGraph,                              // borrowed, not copied
    signatures: HashMap<SymbolId, TypedItemSignature>,   // proven types
    type_registry: TypeRegistry,                         // all type definitions
    available_profiles: Vec<String>,                     // discovered during check
}

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

`TypedProject` has **no** `path`, `module_path`, `imports`, or `ast` fields.
Those live in `graph` and are accessed via `ModuleId` indices.

### Stage 4: Lower

```rust
fn lower(typed: &TypedProject, config: &LowerConfig) -> Verdict<LowerOutput>
```

- **In**: `&TypedProject` + lowering config (profile, entry module)
- **Out**: `LowerOutput` — canonical DAG + all derived metadata computed during lowering
- **Contract**: every required input port has a producer edge. No `lower_warn()`,
  no silent arg drops, no `param_source` without wiring. Every port carries a type.
  Defaults are explicit `MakeLiteral` nodes, not runtime auto-fills.

```rust
struct LowerOutput {
    dag: Dag<LoweredOp>,                               // canonical IR
    output_paths: Vec<String>,                          // from content_upsert + @outputs
    data_values: HashMap<String, serde_json::Value>,    // evaluated data declarations
    entrypoints: Vec<InferredEntrypoint>,                // structurally inferred
}
```

These fields are computed **during** lowering (not re-extracted by the driver
afterward). The driver never walks the DAG to re-derive information that
lowering already computed.

### Stage 5: Verify

```rust
fn verify(dag: Dag<LoweredOp>, types: &TypeRegistry) -> Verdict<VerifiedDag<LoweredOp>>
```

- **In**: `Dag<LoweredOp>` (consumed) + type registry
- **Out**: `VerifiedDag<LoweredOp>` — the only way to construct this type
- **Contract**: all structural invariants proven. Every required port has a
  producer. Every edge is type-compatible. Every pattern instance is well-formed.
  `validate_structural_primitive_wiring()` is merged into this single step.

`VerifiedDag` is the gate to both runtime paths. You cannot call `resolve()`
or `emit()` without one. `skip_verification` cannot exist.

### Stage 6: Derive

```rust
fn derive(verified: &VerifiedDag<LoweredOp>) -> Verdict<DerivedArtifacts>
```

- **In**: `&VerifiedDag<LoweredOp>`
- **Out**: `DerivedArtifacts` — manifest, obligations, transport triplets, properties
- **Contract**: derives from the verified DAG only. Uses `Node.kind` for
  transport classification, not port string heuristics.

### Stage 7a: Resolve (interpreted path)

```rust
fn resolve(
    verified: &VerifiedDag<LoweredOp>,
    bindings: &RuntimeBindings,
) -> Verdict<Dag<DynOp>>
```

- **In**: `&VerifiedDag<LoweredOp>` + runtime bindings (total mapping `ExternId → DynOp`)
- **Out**: `Dag<DynOp>` — executable graph, same topology
- **Contract**: pure relabeling. Same nodes, same edges, same ports. Only body
  labels change from symbolic to executable. Implemented via `dag.map_bodies()`.
  Cannot fail on missing externs (typecheck proved they exist, bindings are total).

```rust
struct RuntimeBindings {
    externs: HashMap<ExternId, DynOp>,   // total: every ExternId has a binding
    // no Option, no fallback, no "none" default
}
```

### Stage 7b: Emit (compiled path)

```rust
fn emit(
    verified: &VerifiedDag<LoweredOp>,
    derived: &DerivedArtifacts,
    config: &EmitConfig,
) -> Verdict<EmissionBundle>
```

- **In**: `&VerifiedDag<LoweredOp>` + derived artifacts + emit config (target, layer)
- **Out**: `EmissionBundle` — source files for target language
- **Contract**: emits from verified DAG and derived artifacts only. Unsupported
  features return `EmitError::UnsupportedFeature`, never `todo!()`.

### Stage 8: Execute

```rust
fn execute(dag: &Dag<DynOp>, config: &ExecConfig) -> Verdict<ExecResult>
```

- **In**: `&Dag<DynOp>` + execution config (mode, mocks, detail level)
- **Out**: `ExecResult` — outputs + execution log
- **Contract**: blind traversal. The executor only knows what the graph tells it.
  No auto-fills, no invented values, no `Value::Skipped`. A node either executes
  (producing outputs) or is transitively bypassed (downstream never scheduled).

```rust
struct ExecConfig {
    mode: ExecutionMode,             // Real | DryRun(mocks, strictness)
    detail: LogDetailLevel,          // how much to log
}

// Current implementation returns Result<ExecutionLog, ExecError>.
// Target: ExecResult bundling outputs + log (not yet implemented).
// struct ExecResult {
//     outputs: HashMap<String, Value>,
//     log: ExecutionLog,
// }
```

One entry point. No `execute_with_mode`, `execute_with_mode_and_inputs`,
`execute_with_mode_and_inputs_and_detail` progression. The config struct carries
all options.

### The Full Pipeline (composed)

```rust
fn compile(vfs: &dyn Vfs, opts: &CompileOptions) -> Verdict<CompileOutput> {
    let sources  = ingest_sources(vfs, &opts.roots, &opts.entry)?;  // only impure stage
    let parsed   = parse_all(&sources)?;
    let graph    = resolve_modules(&parsed)?;
    let externs  = build_extern_registry(&opts.externs);
    let typed    = typecheck(&graph, &externs)?;
    let lowered  = lower(&typed, &opts.lower)?;
    let verified = verify(lowered.dag, &typed.type_registry)?;
    let derived  = derive(&verified)?;
    let emitted  = emit(&verified, &derived, &opts.emit)?;

    Ok(CompileOutput { graph, typed, verified, lowered, derived, emitted })
}
```

And the interpreted runtime path:

```rust
let bindings = build_runtime_bindings(&externs, &resolver);  // total
let resolved = resolve(&verified, &bindings)?;
let result   = execute(&resolved, &exec_config)?;
```

Every `?` is a PASS/FAIL boundary. Every stage takes typed inputs and produces
typed outputs. No hidden coupling. The signatures are the documentation.

### Contract Traceability

| What flows between stages | Type | Owner | Consumed by |
|---------------------------|------|-------|-------------|
| Source text → AST | `SourceFile` | Parse | Module Resolve (stored in `ResolvedModule`) |
| Module graph | `ModuleGraph` | Module Resolve | Typecheck (by ref), Lower (via TypedProject) |
| Type signatures | `TypedProject<'a>` | Typecheck | Lower |
| Extern symbol table | `ExternRegistry` | Builder (pre-typecheck) | Typecheck (validation), Resolve (binding) |
| Canonical IR | `LowerOutput` | Lower | Verify (dag), Derive (after verify), Emit |
| Verification proof | `VerifiedDag<LoweredOp>` | Verify | Resolve, Emit, Derive |
| Derived metadata | `DerivedArtifacts` | Derive | Emit |
| Executable graph | `Dag<DynOp>` | Resolve | Execute |
| Runtime bindings | `RuntimeBindings` | Builder (pre-resolve) | Resolve |
| Execution result | `ExecResult` | Execute | Caller |

---

## Strict Pipeline Diagram

This is the target end state: isolate filesystem impurity once, produce one
canonical verified artifact, realize to backend (mechanical, total), link/emit
as consumers of verified truth (no guessing, no rewiring).

```text
                ┌────────────────────────────────────────────────┐
                │                IMPURE BOUNDARY                 │
                │  (filesystem / environment access is explicit) │
                └────────────────────────────────────────────────┘

   VFS / FS snapshot
         |
         v
┌───────────────────┐   SourceBundle (FileId, path, text, hash, line_index)
│ Stage 0: Ingest   │───────────────────────────────────────────────────────┐
│ (read files once) │                                                       │
└───────────────────┘                                                       │
                                                                            │
                ┌────────────────────────────────────────────────┐          │
                │               PURE COMPILER CORE               │          │
                │     fn(Input) -> Verdict<Output> everywhere    │          │
                └────────────────────────────────────────────────┘          │

         v
┌───────────────────┐   ParsedBundle (AST per file; spans everywhere)
│ Stage 1: ParseAll │───────────────────────────────────────────────────────┐
└───────────────────┘                                                       │
         v                                                                    │
┌────────────────────────┐  ModuleGraph (ModuleId graph + topo order)
│ Stage 2: ResolveModules│───────────────────────────────────────────────────┐
└────────────────────────┘                                                   │
         v                                                                    │
┌───────────────────┐   TypedProgram (IDs resolved; TypeRegistry; signatures)
│ Stage 3: Typecheck│───────────────────────────────────────────────────────┐
└───────────────────┘                                                       │
         v                                                                    │
┌───────────────────┐   LoweredProgram (Dag<LoweredOp> + meta + consts)
│ Stage 4: Lower    │───────────────────────────────────────────────────────┐
└───────────────────┘                                                       │
         v                                                                    │
┌───────────────────┐   VerifiedProgram (VerifiedDag<LoweredOp> + proof)
│ Stage 4.5: Verify │───────────────────────────────────────────────────────┐
└───────────────────┘                                                       │
         |                                                                    │
         | (canonical gate: only VerifiedProgram crosses this boundary)       │
         |                                                                    │
         +--------------------+------------------------------+----------------+
                              |                              |
                              v                              v
                    ┌───────────────────┐          ┌────────────────────────┐
                    │ Stage 5: Derive   │          │ Stage 5: Realize       │
                    │ (manifest, tests, │          │ (total mechanical       │
                    │ obligations, etc) │          │ expansion)              │
                    └───────────────────┘          └────────────────────────┘
                              |                              |
                              v                              v
                   DerivedArtifacts                   RealizedDag<LoweredOp>
                              |                              |
                              |                    +---------+----------+
                              |                    |                    |
                              v                    v                    v
                    ┌───────────────────┐   ┌──────────────────┐  ┌─────────────────┐
                    │ Stage 6b: Emit    │   │ Stage 6a: Link    │  │ Stage 6a: Link  │
                    │ (compiled path)   │   │ (interpreted path)│  │ (compiled runtime│
                    └───────────────────┘   └──────────────────┘  │  if embedded)    │
                              |                    |              └─────────────────┘
                              v                    v
                      EmissionBundle          Dag<DynOp>
                              |                    |
                              v                    v
                        toolchain            ┌───────────────┐
                              |              │ Stage 7: Exec  │
                              v              │ (blind traversal│
                        native binary        │ no autofills)   │
                                             └───────────────┘
                                                   |
                                                   v
                                              ExecResult
```

**Coherence choice**: canonical lowering keeps high-level intent where it
matters; backend realization handles mechanical expansion. This eliminates
the "service triplets created too early" conflict and makes testgen/derive
*trust* metadata instead of re-inferring it.

---

## Overview (legacy — see strict pipeline above for target)

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
| **Input** | `&ModuleGraph` (all modules with raw ASTs) |
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
| TypedModule duplicates module facts | High | `TypedModule` still clones `path`, `module_path`, `imports`, and `ast` from `ResolvedModule` instead of referencing canonical module storage. |
| Relaxed utility mode exists | Medium | Some helper paths still typecheck with `allow_unresolved_imports = true` for partial tooling flows; the strict compile path is fail-closed, but the typecheck contract is not uniform yet. |
| Generic type constraints not fully validated | Medium | Generic instantiation arity is checked, but constraint satisfaction and higher-order function types are not. |
| Pipe method argument types unchecked | Medium | Pipe method call arguments are not validated against method signatures. |

---

## Stage 4: Lower

```
TypedProject  ──>  LowerOutput { dag, output_paths, inferred_entrypoints, data_values }
```

| | |
|---|---|
| **Crate** | `core/daglang/daglang-lower` |
| **Entry** | `lower_to_output_with_config(typed, config) -> Result<LowerOutput, LowerError>` |
| **Input** | `TypedProject` |
| **Output** | `LowerOutput` — DAG plus canonical lower-stage products |

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

`lower_with_config()` still exists as a compatibility wrapper for callers that
only need the DAG, but the main compile path now consumes the bundled
`LowerOutput`.

### Data shape

```
LowerOutput
  +-- dag: Dag<LoweredOp>
  |     +-- nodes: Vec<Node<LoweredOp>>
  |     |     +-- id: NodeId
  |     |     +-- body: NodeBody<LoweredOp>
  |     |     |     +-- Opaque(LoweredOp)          -- leaf node
  |     |     |     +-- SubDag(Dag<LoweredOp>)     -- nested graph (loops, branches)
  |     |     +-- inputs: Vec<Port>
  |     |     +-- outputs: Vec<Port>
  |     |
  |     +-- edges: Vec<Edge>
  |           +-- from_node / from_port
  |           +-- to_node / to_port
  |           +-- kind: EdgeKind { Data, Control, Resource }
  +-- output_paths: Vec<String>
  +-- inferred_entrypoints: Vec<InferredEntrypoint>
  +-- data_values: HashMap<String, Json>

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
| Service calls still expand early | High | Lowering still realizes service operations as transport triplets today; the `VerifiedDag -> Realize -> RealizedDag` split is documented target architecture, not current runtime truth. |
| NodeOrigin stamping is partial | Medium | `NodeOrigin` exists on every node, but most lowerer sites still leave the default `Unknown` origin until span-threading work lands. |
| Lowering still erases some semantic structure | Medium | `LoweredExpr` and transport triplets still collapse higher-level intent that the target design wants to preserve until backend realization. |
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

`gunbc-app` provides `GunbcExternResolver` with ~10 extern implementations
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

## Binary Integration (`gunbc-app`)

The final step for the interpreted path — wiring the compiled DAG into a
runnable binary:

```rust
// One-shot: parse, typecheck, lower, resolve
let dag: Dag<DynOp> = gunbc_resolve::builder::build_dsl_graph(
    "tools/makegen.dag",
    &gunbc_app::extern_ops::GunbcExternResolver,
    gunbc_resolve::BuildOpts::default(),
)?
.dag;

// Execute with mode selection + freshness + display
run_tool(dag, ExecutionMode::Real, options);
```

### Key components

| File | Purpose |
|------|---------|
| `core/resolve/src/builder.rs` | `build_dsl_graph()` — full pipeline in one call |
| `gunbc-app/src/extern_ops.rs` | `GunbcExternResolver` plus ~13 app-specific extern symbol implementations |
| `tool_runner.rs` | `run_tool()` — freshness + execute + display |
| `core/codegen/src/tool_discovery.rs` | Structural tool discovery from DSL |

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

gunbc-app/              Binary integration       (extern resolver, tool runner)
```

---

## Stage Acceptance Criteria

Each stage has PASS guarantees, FAIL conditions, and required tests. These are
the contracts that make stages independently shippable.

### Stage 0: IngestSources

**PASS guarantees**: reads each `.dag` file exactly once into `SourceBundle`,
assigns stable `FileId`, stores `(path, text, hash)`, builds line index.
**FAIL**: file not readable, entry not found, duplicate path.
**Tests**: `ingest_reads_all_files_under_roots`, `ingest_fails_on_missing_entry`,
`ingest_hash_stable_for_identical_text`, `line_index_maps_offsets_to_line_col`.
**Wiring**: no stage after ingest touches filesystem.
`grep -R "std::fs::" core/daglang/daglang-*` should only find it in ingest/driver/VFS.

### Stage 1: ParseAll

**PASS guarantees**: AST is faithful (no silent drops), every node carries Span,
strict path never enters lossy recovery.
**FAIL**: syntax errors; unsupported construct → FAIL with NYI diagnostic.
**Tests**: `parse_interface_type_defs_preserved`, `parse_multi_stmt_block_represented_or_fails`,
`parse_reports_span_for_common_errors`, `parse_strict_never_sets_lossy`.
**Wiring**: `grep -R "lossy" core/daglang/daglang-syntax` shows strict path never uses it.

### Stage 2: ResolveModules

**PASS guarantees**: every import resolves to exactly one module, no silent drop,
no `["unknown"]` fallback, topo order stored (no repeated topo sorts downstream).
**FAIL**: unresolved import (with span), duplicate module path, import cycle.
**Tests**: `module_resolve_fails_on_unresolved_import_with_span`,
`module_resolve_detects_cycle_and_reports_cycle_path`,
`module_resolve_rejects_unknown_module_path`, `module_resolve_topo_order_is_valid`.
**Wiring**: typecheck takes `&ModuleGraph`, never re-walks filesystem.

### Stage 3: Typecheck

**PASS guarantees**: all names resolve, all types match, auth scheme validated,
extern calls validated → `ExternId`, no debug prints.
**FAIL**: unresolved symbol, type mismatch, invalid auth, extern mismatch.
**Tests**: `typecheck_fails_on_unresolved_reference_with_span`,
`typecheck_validates_pipe_method_arg_types`, `typecheck_validates_auth_scheme`,
`typecheck_extern_calls_resolve_to_extern_id`, `typecheck_has_no_debug_eprintln`.
**Wiring**: `grep -R "allow_unresolved_references" ...` returns no hits in strict pipeline.

### Stage 4: Lower

**PASS guarantees**: explicit edges for every required value, default params as
literal nodes, no `param_source` without edges, no `lower_warn()`, computes
`output_paths`/`entrypoints`/`data_values` once.
**FAIL**: pattern expansion failure, unsupported expr, missing service call arg.
**Tests**: `lower_fails_on_pattern_expansion_failure`, `lower_fails_on_missing_service_call_arg`,
`lower_injects_default_param_literals`, `lower_sets_node_origin_for_all_nodes`.
**Wiring**: `grep -R "lower_warn" core/daglang/daglang-lower` → 0 hits.

### Stage 4.5: Verify

**PASS guarantees**: returns `VerifiedDag<T>` (type-level proof), proves all
structural invariants, stores topo order for reuse.
**FAIL**: missing wiring, dangling reference, type mismatch on edge.
**Tests**: `verify_fails_on_missing_required_input_edge`, `verify_fails_on_dangling_edge`,
`verify_produces_topo_order_reused_downstream`.
**Wiring**: emit/link accept **only** Verified input. `skip_verification` removed.

### Stage 5: DeriveArtifacts

**PASS guarantees**: pure function of VerifiedProgram, uses `Node.kind` not
port string heuristics, obligations include provenance (ratchet metric).
**Tests**: `derive_uses_node_kind_not_port_strings`, `derive_emits_obligations_with_provenance`.

### Stage 5: RealizeForBackend

**PASS guarantees**: total (cannot fail if input is Verified), expands ServiceCall
into backend-specific forms, tags with PatternInstance for diagnostics.
**Tests**: `realize_service_call_expands_to_triplet_with_pattern_instance`,
`realize_preserves_dataflow_and_origin`, `realize_is_deterministic`.

### Stage 6a: Link

**PASS guarantees**: topology-preserving via `map_bodies()`, binds IDs → DynOp
via `RuntimeBindings` (total, no Option).
**Tests**: `link_is_topology_preserving_by_construction`, `link_never_looks_up_by_string`.
**Wiring target**: CP-47 ends with ExternId-keyed `RuntimeBindings` and no
string lookup. Current branch still ships a bridge form: `RuntimeBindings`
exists, but it is keyed by `(module, name)` and implements `ExternResolver` for
incremental migration.

### Stage 6b: Emit

**PASS guarantees**: no `todo!()`, uses VerifiedProgram + DerivedArtifacts,
unsupported features → `EmitError::UnsupportedFeature` with span.
**Tests**: `emit_returns_unsupported_feature_error_instead_of_todo`,
`emit_orchestration_topo_order_matches_verified_topo` (after CP-10).

### Stage 7: Execute

**PASS guarantees**: blind traversal, no implicit defaults, no `Value::Skipped`,
`DryRun(Strict)` blocks unmocked side effects.
**Tests**: `exec_dryrun_strict_blocks_unmocked_side_effect_node`,
`exec_has_single_entrypoint`, `exec_does_not_autofill_optional_list_inputs`.

---

## Phase Gates (ratchet — never regress)

### Gate A (after Foundation + Silent Failure Elimination)

* `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
* Strict pipeline: binary PASS/FAIL, no silent drops
* `grep -R "lower_warn" core/daglang | wc -l` → 0 (strict path)
* `grep -R "lossy:" core/daglang | wc -l` → 0 (strict path)
* `grep -R "allow_unresolved_references" core/daglang | wc -l` → 0 (strict path)

### Gate B (after Strict Verification)

* Verification is mandatory in driver
* VerifiedDag gates emit/link
* `grep -R "skip_verification" core/daglang | wc -l` → 0 (or constant false)
* `grep -R "validate_structural_primitive_wiring" core/daglang` → only inside verify

### Gate D (after Interface Cleanup)

* Stage APIs are pure at boundaries (no fs reads except ingest)
* Execute has single entrypoint
* `grep -R "std::env::var\\(\"DAGLANG_" core/daglang | wc -l` → 0
* `grep -R "execute_with_mode" core/exec | wc -l` → 0

### Gate Parity (after D.3)

* Parity harness runs on at least one representative `.dag`
* Outputs match: interpreted vs compiled

---

## Wiring Demands (no half-migrations)

### CP-58: `daglang-contracts` crate

* All stage crates depend on it
* `grep -R "type Verdict" core/daglang | wc -l` → 1 (in contracts)
* `grep -R "struct Diagnostic" core/daglang | wc -l` → 1 (in contracts)

### CP-23: VerifiedDag gate

* `emit(...)` requires `&VerifiedDag<_>` or `&VerifiedProgram`
* `link(...)` requires verified input
* Old callers that skip verify won't compile

### CP-40 + CP-47: ExternRegistry + RuntimeBindings

* Typecheck validates externs → `ExternId`
* Link binds `ExternId → DynOp` using total mappings
* `grep -R "trait ExternResolver" -n` → 0 (except legacy compat)

### CP-34 + CP-56: Remove stringly re-derivation

* `grep -R "detect_loop_pattern" -n` → 0 (after CP-35)
* No provider inference from node IDs in testgen
